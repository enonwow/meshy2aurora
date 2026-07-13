use std::collections::{HashMap, HashSet};

use super::binary_reader::{BinaryReader, checked_array_size};
use super::errors::{HEADER_INVALID, NODE_CYCLE, ParseError};
use super::types::{
    AnimationEventReport, AnimationReport, ArrayReport, ByteRangeReport, ControllerReport,
    Diagnostic, FaceReport, FileHeaderReport, InspectionReport, MeshReport, ModelReport,
    NodeReport, NodeTreeReport, ParserLimits, RawPointerReport, SkinReport, SkinVariant, Vec2,
    Vec3,
};

const FILE_HEADER_SIZE: usize = 0x0c;
const MODEL_HEADER_SIZE: usize = 0xe8;
const ANIMATION_HEADER_SIZE: usize = 0xc4;
const ANIMATION_EVENT_SIZE: usize = 0x24;
const NODE_HEADER_SIZE: usize = 0x70;
const MESH_NODE_SIZE: usize = 0x270;
const SKIN_LEGACY17_SIZE: usize = 0x2d4;
const SKIN_EXTENDED64_SIZE: usize = 0x330;
const ARRAY_HEADER_SIZE: usize = 0x0c;
const POINTER_SIZE: usize = 4;
const CONTROLLER_KEY_SIZE: usize = 0x0c;

const GEOMETRY_NAME_OFFSET: usize = 0x08;
const GEOMETRY_NAME_LENGTH: usize = 64;
const GEOMETRY_ROOT_OFFSET: usize = 0x48;
const GEOMETRY_NODE_COUNT_OFFSET: usize = 0x4c;

const NODE_INHERIT_OFFSET: usize = 0x18;
const NODE_NUMBER_OFFSET: usize = 0x1c;
const NODE_NAME_OFFSET: usize = 0x20;
const NODE_NAME_LENGTH: usize = 32;
const NODE_CHILDREN_OFFSET: usize = 0x48;
const NODE_CONTROLLER_KEYS_OFFSET: usize = 0x54;
const NODE_CONTROLLER_DATA_OFFSET: usize = 0x60;
const NODE_CONTENT_OFFSET: usize = 0x6c;

const FLAG_HEADER: u32 = 0x001;
const FLAG_LIGHT: u32 = 0x002;
const FLAG_EMITTER: u32 = 0x004;
const FLAG_CAMERA: u32 = 0x008;
const FLAG_REFERENCE: u32 = 0x010;
const FLAG_MESH: u32 = 0x020;
const FLAG_SKIN: u32 = 0x040;
const FLAG_ANIMMESH: u32 = 0x080;
const FLAG_DANGLY: u32 = 0x100;
const FLAG_AABB: u32 = 0x200;
const KNOWN_NODE_FLAGS: u32 = 0x3ff;
const SUPPORTED_NODE_FLAGS: u32 = FLAG_HEADER | FLAG_MESH | FLAG_SKIN;

const MESH_FACES_OFFSET: usize = 0x78;
const MESH_BOUNDS_MIN_OFFSET: usize = 0x84;
const MESH_BOUNDS_MAX_OFFSET: usize = 0x90;
const MESH_RADIUS_OFFSET: usize = 0x9c;
const MESH_AVERAGE_OFFSET: usize = 0xa0;
const MESH_DIFFUSE_OFFSET: usize = 0xac;
const MESH_AMBIENT_OFFSET: usize = 0xb8;
const MESH_SPECULAR_OFFSET: usize = 0xc4;
const MESH_SHININESS_OFFSET: usize = 0xd0;
const MESH_SHADOW_OFFSET: usize = 0xd4;
const MESH_BEAMING_OFFSET: usize = 0xd8;
const MESH_RENDER_OFFSET: usize = 0xdc;
const MESH_TRANSPARENCY_OFFSET: usize = 0xe0;
const MESH_RENDER_HINT_OFFSET: usize = 0xe4;
const MESH_TEXTURE0_OFFSET: usize = 0xe8;
const MESH_TILE_FADE_OFFSET: usize = 0x1e8;
const MESH_INDEX_COUNTS_OFFSET: usize = 0x204;
const MESH_RAW_INDEX_OFFSETS_OFFSET: usize = 0x210;
const MESH_UNKNOWN_RAW_OFFSET: usize = 0x21c;
const MESH_TYPE_OFFSET: usize = 0x224;
const MESH_START_RAW_OFFSET: usize = 0x228;
const MESH_VERTICES_RAW_OFFSET: usize = 0x22c;
const MESH_VERTEX_COUNT_OFFSET: usize = 0x230;
const MESH_TEXTURE_COUNT_OFFSET: usize = 0x232;
const MESH_UV0_RAW_OFFSET: usize = 0x234;
const MESH_NORMALS_RAW_OFFSET: usize = 0x244;
const MESH_COLORS_RAW_OFFSET: usize = 0x248;
const MESH_TEX_ANIM0_RAW_OFFSET: usize = 0x24c;

const SKIN_WEIGHTS_HEADER_OFFSET: usize = 0x270;
const SKIN_RAW_WEIGHTS_OFFSET: usize = 0x27c;
const SKIN_RAW_BONE_REFS_OFFSET: usize = 0x280;
const SKIN_NODE_TO_BONE_OFFSET: usize = 0x284;
const SKIN_MAP_COUNT_OFFSET: usize = 0x288;
const SKIN_Q_INV_OFFSET: usize = 0x28c;
const SKIN_T_INV_OFFSET: usize = 0x298;
const SKIN_CONSTANTS_OFFSET: usize = 0x2a4;
const SKIN_INLINE_MAPPING_OFFSET: usize = 0x2b0;

/// Inspects one binary MDL with the default product guardrails.
///
/// # Errors
///
/// Returns a stable [`ParseError`] when the input is malformed, unsupported at
/// the structural boundary, or exceeds a product guardrail.
pub fn inspect_binary_mdl(bytes: &[u8]) -> Result<InspectionReport, ParseError> {
    inspect_binary_mdl_with_limits(bytes, &ParserLimits::default())
}

/// Inspects one binary MDL with caller-supplied product guardrails.
///
/// # Errors
///
/// Returns a stable [`ParseError`] when the input is malformed or exceeds a
/// caller-supplied guardrail.
pub fn inspect_binary_mdl_with_limits(
    bytes: &[u8],
    limits: &ParserLimits,
) -> Result<InspectionReport, ParseError> {
    if bytes.len() > limits.max_input_bytes {
        return Err(ParseError::limit(
            0,
            format!(
                "input length {} exceeds product guardrail {}",
                bytes.len(),
                limits.max_input_bytes
            ),
        ));
    }
    if bytes.len() < FILE_HEADER_SIZE {
        return Err(ParseError::header(
            0,
            format!(
                "binary MDL file header requires {FILE_HEADER_SIZE} bytes, got {}",
                bytes.len()
            ),
        ));
    }

    let reader = BinaryReader::new(bytes);
    let binary_mdl_id = reader.read_u32(0, "bin_mdl_id")?;
    if binary_mdl_id != 0 {
        return Err(ParseError::new(
            HEADER_INVALID,
            0,
            format!("bin_mdl_id must be 0, got {binary_mdl_id}"),
        ));
    }

    let raw_data_offset = reader.read_u32(4, "p_start_mdx")?;
    let raw_data_size = reader.read_u32(8, "size_mdx")?;
    let core_length = raw_data_offset as usize;
    let raw_absolute = FILE_HEADER_SIZE
        .checked_add(core_length)
        .ok_or_else(|| ParseError::pointer(4, "raw data start overflow"))?;
    reader.read_slice(FILE_HEADER_SIZE, core_length, "core MDL block")?;
    reader.read_slice(raw_absolute, raw_data_size as usize, "appended MDX range")?;
    let declared_end = raw_absolute
        .checked_add(raw_data_size as usize)
        .ok_or_else(|| ParseError::pointer(8, "declared payload end overflow"))?;
    if declared_end != bytes.len() {
        return Err(ParseError::new(
            HEADER_INVALID,
            declared_end.min(bytes.len()),
            format!(
                "file length {} does not equal exact declared header/core/raw length {declared_end}",
                bytes.len()
            ),
        ));
    }
    if core_length < MODEL_HEADER_SIZE {
        return Err(ParseError::pointer(
            FILE_HEADER_SIZE,
            format!(
                "core MDL block length {core_length} is smaller than model header {MODEL_HEADER_SIZE}"
            ),
        ));
    }

    let mut context = ParseContext::new(
        &reader,
        limits,
        core_length,
        raw_absolute,
        raw_data_size as usize,
    );
    context.claim_core(0, MODEL_HEADER_SIZE, "model-header")?;
    let model_absolute = FILE_HEADER_SIZE;
    let model_name = reader.read_fixed_string(
        model_absolute + GEOMETRY_NAME_OFFSET,
        GEOMETRY_NAME_LENGTH,
        "model name",
    )?;
    let root_node_offset = reader.read_u32(
        model_absolute + GEOMETRY_ROOT_OFFSET,
        "model root node pointer",
    )?;
    let declared_node_count = read_declared_node_count(
        &reader,
        model_absolute + GEOMETRY_NODE_COUNT_OFFSET,
        root_node_offset,
        limits,
        "model",
    )?;

    let node_tree = parse_node_tree(
        &mut context,
        root_node_offset,
        declared_node_count,
        "base model root",
    )?;
    let (animation_pointers_header, animations) = parse_animations(&mut context, model_absolute)?;

    let core_end = FILE_HEADER_SIZE
        .checked_add(core_length)
        .ok_or_else(|| ParseError::pointer(4, "core range overflow"))?;
    let raw_end = raw_absolute
        .checked_add(raw_data_size as usize)
        .ok_or_else(|| ParseError::pointer(8, "raw range overflow"))?;

    Ok(InspectionReport {
        schema_version: 1,
        format: "nwn1-binary-mdl".to_owned(),
        byte_length: bytes.len(),
        file_header: FileHeaderReport {
            binary_mdl_id,
            mdx_start: raw_data_offset,
            mdx_size: raw_data_size,
            mdx_range_in_bounds: true,
            core_range: ByteRangeReport {
                start: FILE_HEADER_SIZE,
                length: core_length,
                end: core_end,
            },
            raw_range: ByteRangeReport {
                start: raw_absolute,
                length: raw_data_size as usize,
                end: raw_end,
            },
        },
        model: ModelReport {
            name: model_name,
            root_node_offset,
            geometry_type: reader.read_u32(model_absolute + 0x6c, "model geometry type")?,
            classification: reader.read_u8(model_absolute + 0x72, "model classification")?,
            fog: reader.read_u8(model_absolute + 0x73, "model fog")?,
            child_model_count: reader.read_u32(model_absolute + 0x74, "model child count")?,
            bounds_min: read_vec3(&reader, model_absolute + 0x88, "model bounds min")?,
            bounds_max: read_vec3(&reader, model_absolute + 0x94, "model bounds max")?,
            radius: reader.read_f32(model_absolute + 0xa0, "model radius")?,
            animation_scale: reader.read_f32(model_absolute + 0xa4, "model animation scale")?,
            supermodel_name: reader.read_fixed_string(
                model_absolute + 0xa8,
                64,
                "model supermodel name",
            )?,
            animation_pointers_header,
        },
        node_tree,
        animations,
        unsupported: context.unsupported,
        diagnostics: context.diagnostics,
    })
}

struct ParseContext<'reader, 'input> {
    reader: &'reader BinaryReader<'input>,
    limits: &'reader ParserLimits,
    core_length: usize,
    raw_absolute: usize,
    raw_length: usize,
    claims: Vec<CoreClaim>,
    unsupported: Vec<String>,
    diagnostics: Vec<Diagnostic>,
}

struct CoreClaim {
    start: usize,
    end: usize,
    kind: &'static str,
}

fn intentional_node_family_nesting(previous: &CoreClaim, candidate: &CoreClaim) -> bool {
    const NODE_FAMILY_KINDS: [&str; 3] = ["node-header", "mesh-node", "skin-node"];
    previous.start == candidate.start
        && NODE_FAMILY_KINDS.contains(&previous.kind)
        && NODE_FAMILY_KINDS.contains(&candidate.kind)
        && ((previous.start <= candidate.start && previous.end >= candidate.end)
            || (candidate.start <= previous.start && candidate.end >= previous.end))
}

impl<'reader, 'input> ParseContext<'reader, 'input> {
    fn new(
        reader: &'reader BinaryReader<'input>,
        limits: &'reader ParserLimits,
        core_length: usize,
        raw_absolute: usize,
        raw_length: usize,
    ) -> Self {
        Self {
            reader,
            limits,
            core_length,
            raw_absolute,
            raw_length,
            claims: Vec::new(),
            unsupported: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn claim_core(
        &mut self,
        offset: u32,
        length: usize,
        kind: &'static str,
    ) -> Result<(), ParseError> {
        if length == 0 {
            return Ok(());
        }
        let end = (offset as usize)
            .checked_add(length)
            .ok_or_else(|| ParseError::pointer(offset as usize, "core claim range overflow"))?;
        let candidate = CoreClaim {
            start: offset as usize,
            end,
            kind,
        };
        for previous in &self.claims {
            if candidate.start == previous.start
                && candidate.end == previous.end
                && candidate.kind == previous.kind
            {
                return Ok(());
            }
            let overlaps = candidate.start < previous.end && previous.start < candidate.end;
            if overlaps && !intentional_node_family_nesting(previous, &candidate) {
                return Err(ParseError::offset_type_conflict(
                    core_pointer(offset)?,
                    format!(
                        "core range 0x{:08x}..0x{:08x} ({}) overlaps 0x{:08x}..0x{:08x} ({})",
                        candidate.start,
                        candidate.end,
                        candidate.kind,
                        previous.start,
                        previous.end,
                        previous.kind
                    ),
                ));
            }
        }
        self.claims.push(candidate);
        Ok(())
    }

    fn core_absolute(
        &self,
        offset: u32,
        length: usize,
        context: &str,
    ) -> Result<usize, ParseError> {
        let relative_end = (offset as usize)
            .checked_add(length)
            .ok_or_else(|| ParseError::pointer(offset as usize, format!("{context} overflow")))?;
        if relative_end > self.core_length {
            return Err(ParseError::pointer(
                core_pointer(offset)?,
                format!(
                    "{context} escapes core MDL block ending at relative offset {}",
                    self.core_length
                ),
            ));
        }
        let absolute = core_pointer(offset)?;
        self.reader.read_slice(absolute, length, context)?;
        Ok(absolute)
    }

    fn raw_absolute(
        &self,
        pointer: i32,
        length: usize,
        context: &str,
        required: bool,
    ) -> Result<Option<usize>, ParseError> {
        if pointer == -1 {
            if required {
                return Err(ParseError::pointer(
                    self.raw_absolute,
                    format!("required {context} pointer is null"),
                ));
            }
            return Ok(None);
        }
        if pointer < -1 {
            return Err(ParseError::pointer(
                self.raw_absolute,
                format!("{context} has invalid negative raw pointer {pointer}"),
            ));
        }
        let relative = pointer as usize;
        let end = relative
            .checked_add(length)
            .ok_or_else(|| ParseError::pointer(relative, format!("{context} range overflow")))?;
        if end > self.raw_length {
            return Err(ParseError::pointer(
                self.raw_absolute.saturating_add(relative),
                format!(
                    "{context} escapes raw MDX block ending at relative offset {}",
                    self.raw_length
                ),
            ));
        }
        let absolute = self
            .raw_absolute
            .checked_add(relative)
            .ok_or_else(|| ParseError::pointer(relative, format!("{context} pointer overflow")))?;
        self.reader.read_slice(absolute, length, context)?;
        Ok(Some(absolute))
    }

    fn read_array_header(
        &mut self,
        header_absolute: usize,
        element_size: usize,
        context: &str,
        claim_kind: &'static str,
    ) -> Result<ArrayHeader, ParseError> {
        self.reader
            .read_slice(header_absolute, ARRAY_HEADER_SIZE, context)?;
        let pointer = self.reader.read_u32(header_absolute, context)?;
        let used = self.reader.read_u32(header_absolute + 4, context)? as usize;
        let allocated = self.reader.read_u32(header_absolute + 8, context)? as usize;
        if used > allocated {
            return Err(ParseError::header(
                header_absolute + 4,
                format!("{context} used count {used} exceeds allocated count {allocated}"),
            ));
        }
        let maximum_entries = self.limits.max_input_bytes / element_size.max(1);
        if allocated > maximum_entries {
            return Err(ParseError::limit(
                header_absolute + 8,
                format!(
                    "{context} allocated count {allocated} exceeds product guardrail {maximum_entries}"
                ),
            ));
        }
        if allocated == 0 {
            return Ok(ArrayHeader {
                pointer,
                used,
                allocated,
            });
        }
        if pointer == 0 {
            return Err(ParseError::pointer(
                header_absolute,
                format!("non-empty {context} array has a null pointer"),
            ));
        }
        let byte_length = checked_array_size(allocated, element_size, pointer as usize, context)?;
        self.core_absolute(pointer, byte_length, context)?;
        self.claim_core(pointer, byte_length, claim_kind)?;
        Ok(ArrayHeader {
            pointer,
            used,
            allocated,
        })
    }

    fn push_diagnostic(&mut self, diagnostic: Diagnostic) -> Result<(), ParseError> {
        if self.diagnostics.len() >= self.limits.max_diagnostics {
            return Err(ParseError::limit(
                diagnostic.offset,
                format!(
                    "diagnostic count would exceed product guardrail {}",
                    self.limits.max_diagnostics
                ),
            ));
        }
        self.unsupported.push(diagnostic.context.clone());
        self.diagnostics.push(diagnostic);
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct ArrayHeader {
    pointer: u32,
    used: usize,
    allocated: usize,
}

impl From<ArrayHeader> for ArrayReport {
    fn from(value: ArrayHeader) -> Self {
        Self {
            pointer: value.pointer,
            used: value.used,
            allocated: value.allocated,
        }
    }
}

fn read_declared_node_count(
    reader: &BinaryReader<'_>,
    count_absolute: usize,
    root_offset: u32,
    limits: &ParserLimits,
    context: &str,
) -> Result<usize, ParseError> {
    if root_offset == 0 {
        return Err(ParseError::pointer(
            count_absolute.saturating_sub(4),
            format!("{context} root node pointer is null"),
        ));
    }
    if (root_offset as usize) < MODEL_HEADER_SIZE {
        return Err(ParseError::pointer(
            count_absolute.saturating_sub(4),
            format!("{context} root node offset 0x{root_offset:08x} overlaps the model header"),
        ));
    }
    let declared = reader.read_u32(count_absolute, context)? as usize;
    if declared == 0 {
        return Err(ParseError::header(
            count_absolute,
            format!("{context} declared node count is zero while a root pointer is present"),
        ));
    }
    if declared > limits.max_nodes {
        return Err(ParseError::limit(
            count_absolute,
            format!(
                "{context} declared node count {declared} exceeds product guardrail {}",
                limits.max_nodes
            ),
        ));
    }
    Ok(declared)
}

fn parse_node_tree(
    context: &mut ParseContext<'_, '_>,
    root_offset: u32,
    declared_node_count: usize,
    tree_context: &str,
) -> Result<NodeTreeReport, ParseError> {
    let mut pending = vec![(root_offset, None, 0_usize)];
    let mut seen = HashSet::new();
    let mut flat_nodes = Vec::new();
    let mut max_depth = 0;

    while let Some((node_offset, parent_offset, depth)) = pending.pop() {
        if depth > context.limits.max_depth {
            return Err(ParseError::limit(
                core_pointer(node_offset)?,
                format!(
                    "{tree_context} node depth {depth} exceeds product guardrail {}",
                    context.limits.max_depth
                ),
            ));
        }
        if !seen.insert(node_offset) {
            return Err(ParseError::new(
                NODE_CYCLE,
                core_pointer(node_offset)?,
                format!("{tree_context} node pointer 0x{node_offset:08x} was visited twice"),
            ));
        }
        if flat_nodes.len() >= declared_node_count {
            return Err(ParseError::header(
                core_pointer(node_offset)?,
                format!(
                    "{tree_context} traversed node count exceeds declared budget {declared_node_count}"
                ),
            ));
        }

        let node = read_node(context, node_offset, parent_offset)?;
        let child_depth = depth
            .checked_add(1)
            .ok_or_else(|| ParseError::limit(0, "node depth overflow"))?;
        for &child in node.child_offsets.iter().rev() {
            pending.push((child, Some(node_offset), child_depth));
        }
        max_depth = max_depth.max(depth);
        flat_nodes.push(node);
    }

    let node_count = flat_nodes.len();
    let mut built = HashMap::with_capacity(node_count);
    for node in flat_nodes.into_iter().rev() {
        let mut children = Vec::with_capacity(node.child_offsets.len());
        for child_offset in node.child_offsets {
            children.push(built.remove(&child_offset).ok_or_else(|| {
                ParseError::header(
                    core_pointer(child_offset).unwrap_or_default(),
                    "child node was not available during tree assembly",
                )
            })?);
        }
        built.insert(
            node.offset,
            NodeReport {
                offset: node.offset,
                number: node.number,
                name: node.name,
                parent_offset: node.parent_offset,
                inherit_color: node.inherit_color,
                content_flags: node.content_flags,
                unsupported_families: node.unsupported_families,
                children_header: node.children_header,
                controller_keys_header: node.controller_keys_header,
                controller_data_header: node.controller_data_header,
                controllers: node.controllers,
                mesh: node.mesh,
                skin: node.skin,
                children,
            },
        );
    }
    let root = built.remove(&root_offset).ok_or_else(|| {
        ParseError::header(
            core_pointer(root_offset).unwrap_or_default(),
            "root node was not available during tree assembly",
        )
    })?;
    Ok(NodeTreeReport {
        declared_node_count,
        node_count,
        max_depth,
        roots: vec![root],
    })
}

struct FlatNode {
    offset: u32,
    number: u32,
    name: String,
    parent_offset: Option<u32>,
    inherit_color: u32,
    content_flags: u32,
    unsupported_families: Vec<String>,
    children_header: ArrayReport,
    controller_keys_header: ArrayReport,
    controller_data_header: ArrayReport,
    controllers: Vec<ControllerReport>,
    mesh: Option<MeshReport>,
    skin: Option<SkinReport>,
    child_offsets: Vec<u32>,
}

fn read_node(
    context: &mut ParseContext<'_, '_>,
    node_offset: u32,
    parent_offset: Option<u32>,
) -> Result<FlatNode, ParseError> {
    if (node_offset as usize) < MODEL_HEADER_SIZE {
        return Err(ParseError::pointer(
            core_pointer(node_offset)?,
            format!("node offset 0x{node_offset:08x} overlaps the model header"),
        ));
    }
    context.claim_core(node_offset, NODE_HEADER_SIZE, "node-header")?;
    let absolute = context.core_absolute(node_offset, NODE_HEADER_SIZE, "node header")?;
    let inherit_color = context
        .reader
        .read_u32(absolute + NODE_INHERIT_OFFSET, "node inherit color")?;
    let number = context
        .reader
        .read_u32(absolute + NODE_NUMBER_OFFSET, "node number")?;
    let name = context.reader.read_fixed_string(
        absolute + NODE_NAME_OFFSET,
        NODE_NAME_LENGTH,
        "node name",
    )?;
    let children = context.read_array_header(
        absolute + NODE_CHILDREN_OFFSET,
        POINTER_SIZE,
        "node children",
        "node-children",
    )?;
    if children.used > context.limits.max_nodes || children.allocated > context.limits.max_nodes {
        return Err(ParseError::limit(
            absolute + NODE_CHILDREN_OFFSET + 4,
            format!(
                "node children used/allocated counts {}/{} exceed node guardrail {}",
                children.used, children.allocated, context.limits.max_nodes
            ),
        ));
    }
    let child_offsets = read_core_u32_values(context, children, "node child pointer entry")?;
    let controllers = read_controllers(context, absolute)?;
    let content_flags = context
        .reader
        .read_u32(absolute + NODE_CONTENT_OFFSET, "node content flags")?;
    let mut unsupported_families = Vec::new();
    let unknown_flags = content_flags & !KNOWN_NODE_FLAGS;
    let unsupported_flags = content_flags & KNOWN_NODE_FLAGS & !SUPPORTED_NODE_FLAGS;
    if unknown_flags != 0 {
        let label = format!("unknown:0x{unknown_flags:08x}");
        unsupported_families.push(label.clone());
        context.push_diagnostic(Diagnostic::unknown_node_flags(
            absolute + NODE_CONTENT_OFFSET,
            format!("node {name:?} has unknown content flags 0x{unknown_flags:08x}"),
        ))?;
    }
    for (bit, family) in node_family_bits() {
        if unsupported_flags & bit != 0 {
            unsupported_families.push(family.to_owned());
            context.push_diagnostic(Diagnostic::unsupported_family(
                absolute + NODE_CONTENT_OFFSET,
                format!("node {name:?} uses deferred node family {family}"),
            ))?;
        }
    }

    let prefix_shifting_flags = FLAG_LIGHT | FLAG_EMITTER | FLAG_CAMERA | FLAG_REFERENCE;
    let mesh_decode_allowed = unknown_flags == 0 && content_flags & prefix_shifting_flags == 0;
    let (mesh, skin) = if mesh_decode_allowed && content_flags & FLAG_MESH != 0 {
        let mesh = read_mesh(context, node_offset)?;
        let skin = if content_flags & FLAG_SKIN != 0 {
            Some(read_skin(context, node_offset, mesh.vertex_count)?)
        } else {
            None
        };
        (Some(mesh), skin)
    } else {
        if mesh_decode_allowed && content_flags & FLAG_SKIN != 0 {
            return Err(ParseError::header(
                absolute + NODE_CONTENT_OFFSET,
                "skin node is missing the required mesh family flag",
            ));
        }
        (None, None)
    };

    Ok(FlatNode {
        offset: node_offset,
        number,
        name,
        parent_offset,
        inherit_color,
        content_flags,
        unsupported_families,
        children_header: children.into(),
        controller_keys_header: controllers.keys_header,
        controller_data_header: controllers.data_header,
        controllers: controllers.controllers,
        mesh,
        skin,
        child_offsets,
    })
}

struct ControllerBlockReport {
    keys_header: ArrayReport,
    data_header: ArrayReport,
    controllers: Vec<ControllerReport>,
}

fn read_controllers(
    context: &mut ParseContext<'_, '_>,
    node_absolute: usize,
) -> Result<ControllerBlockReport, ParseError> {
    let keys = context.read_array_header(
        node_absolute + NODE_CONTROLLER_KEYS_OFFSET,
        CONTROLLER_KEY_SIZE,
        "node controller keys",
        "controller-keys",
    )?;
    let data = context.read_array_header(
        node_absolute + NODE_CONTROLLER_DATA_OFFSET,
        4,
        "node controller data",
        "controller-data",
    )?;
    let data_absolute = if data.used == 0 {
        None
    } else {
        Some(context.core_absolute(
            data.pointer,
            checked_array_size(data.used, 4, data.pointer as usize, "controller data")?,
            "controller data",
        )?)
    };
    let mut data_values = Vec::with_capacity(data.used);
    if let Some(absolute) = data_absolute {
        for index in 0..data.used {
            data_values.push(
                context
                    .reader
                    .read_f32(absolute + index * 4, "controller data value")?,
            );
        }
    }
    if keys.used == 0 {
        return Ok(ControllerBlockReport {
            keys_header: keys.into(),
            data_header: data.into(),
            controllers: Vec::new(),
        });
    }
    let keys_absolute = context.core_absolute(
        keys.pointer,
        checked_array_size(
            keys.used,
            CONTROLLER_KEY_SIZE,
            keys.pointer as usize,
            "controller keys",
        )?,
        "controller keys",
    )?;
    let mut reports = Vec::with_capacity(keys.used);
    for index in 0..keys.used {
        let key_absolute = keys_absolute + index * CONTROLLER_KEY_SIZE;
        let controller_type = context.reader.read_i32(key_absolute, "controller type")?;
        let rows = context
            .reader
            .read_u16(key_absolute + 4, "controller row count")? as usize;
        let time_index = context
            .reader
            .read_u16(key_absolute + 6, "controller time index")? as usize;
        let data_index = context
            .reader
            .read_u16(key_absolute + 8, "controller data index")? as usize;
        let packed_byte = context.reader.read_u8(
            key_absolute + 10,
            "controller packed columns and interpolation flags",
        )?;
        let padding_byte = context
            .reader
            .read_u8(key_absolute + 11, "controller padding byte")?;
        let columns = usize::from(packed_byte & 0x0f);
        let interpolation_flags = packed_byte & 0xf0;
        if interpolation_flags & !0x10 != 0 {
            return Err(ParseError::controller(
                key_absolute + 10,
                format!(
                    "controller {controller_type} has unsupported interpolation flags 0x{interpolation_flags:02x} in packed byte 0x{packed_byte:02x}"
                ),
            ));
        }
        if columns == 0 {
            return Err(ParseError::controller(
                key_absolute + 10,
                format!(
                    "controller {controller_type} has invalid rows/columns {rows}/{columns} from packed byte 0x{packed_byte:02x}"
                ),
            ));
        }
        let semantics = controller_semantics(controller_type);
        if let Some((name, expected_columns)) = semantics
            && columns != expected_columns
        {
            return Err(ParseError::controller(
                key_absolute + 10,
                format!(
                    "common controller {name} ({controller_type}) requires {expected_columns} columns, got {columns}"
                ),
            ));
        }
        let controller_name = semantics.map(|(name, _)| name.to_owned());
        let decoded = interpolation_flags == 0;
        if !decoded {
            context.push_diagnostic(Diagnostic::unsupported_controller_interpolation(
                key_absolute + 10,
                format!(
                    "controller {controller_type} uses recognized deferred Bezier interpolation flags 0x{interpolation_flags:02x}"
                ),
            ))?;
        }
        let time_end = time_index.checked_add(rows).ok_or_else(|| {
            ParseError::controller_index(key_absolute + 6, "controller time range overflow")
        })?;
        if time_end > data_values.len() {
            return Err(ParseError::controller_index(
                key_absolute + 6,
                format!(
                    "controller {controller_type} time range {time_index}..{time_end} exceeds controller data length {}",
                    data_values.len()
                ),
            ));
        }
        if !decoded && rows > 0 && data_index >= data_values.len() {
            return Err(ParseError::controller_index(
                key_absolute + 8,
                format!(
                    "deferred controller {controller_type} data index {data_index} exceeds controller data length {}",
                    data_values.len()
                ),
            ));
        }
        if controller_name.is_none() {
            context.push_diagnostic(Diagnostic::unknown_controller(
                key_absolute,
                format!("controller type {controller_type} is not semantically known"),
            ))?;
        }
        let (times, values) = if decoded {
            let value_count = rows.checked_mul(columns).ok_or_else(|| {
                ParseError::controller(key_absolute + 10, "controller value count overflow")
            })?;
            let data_end = data_index.checked_add(value_count).ok_or_else(|| {
                ParseError::controller_index(key_absolute + 8, "controller data range overflow")
            })?;
            if data_end > data_values.len() {
                return Err(ParseError::controller_index(
                    key_absolute + 8,
                    format!(
                        "controller {controller_type} data range {data_index}..{data_end} exceeds controller data length {}",
                        data_values.len()
                    ),
                ));
            }
            let times = data_values[time_index..time_end].to_vec();
            let mut values = Vec::with_capacity(rows);
            for row in 0..rows {
                let start = data_index + row * columns;
                values.push(data_values[start..start + columns].to_vec());
            }
            (times, values)
        } else {
            (Vec::new(), Vec::new())
        };
        reports.push(ControllerReport {
            key_offset: u32::try_from(key_absolute - FILE_HEADER_SIZE).map_err(|_| {
                ParseError::controller(key_absolute, "controller key offset does not fit u32")
            })?,
            controller_type,
            controller_name,
            packed_byte,
            interpolation_flags,
            decoded,
            padding_byte,
            row_count: rows,
            time_index,
            data_index,
            column_count: columns,
            times,
            values,
        });
    }
    Ok(ControllerBlockReport {
        keys_header: keys.into(),
        data_header: data.into(),
        controllers: reports,
    })
}

fn controller_semantics(controller_type: i32) -> Option<(&'static str, usize)> {
    match controller_type {
        8 => Some(("position", 3)),
        20 => Some(("orientation", 4)),
        36 => Some(("scale", 1)),
        100 => Some(("selfIllumination", 3)),
        128 => Some(("alpha", 1)),
        _ => None,
    }
}

fn read_mesh(
    context: &mut ParseContext<'_, '_>,
    node_offset: u32,
) -> Result<MeshReport, ParseError> {
    context.claim_core(node_offset, MESH_NODE_SIZE, "mesh-node")?;
    let absolute = context.core_absolute(node_offset, MESH_NODE_SIZE, "mesh node")?;
    let vertex_count = context
        .reader
        .read_u16(absolute + MESH_VERTEX_COUNT_OFFSET, "mesh vertex count")?
        as usize;
    let texture_count = context
        .reader
        .read_u16(absolute + MESH_TEXTURE_COUNT_OFFSET, "mesh texture count")?
        as usize;
    let mut textures = Vec::with_capacity(4);
    for index in 0..4 {
        textures.push(context.reader.read_fixed_string(
            absolute + MESH_TEXTURE0_OFFSET + index * 64,
            64,
            "mesh texture resref",
        )?);
    }

    let faces_header = context.read_array_header(
        absolute + MESH_FACES_OFFSET,
        0x20,
        "mesh faces",
        "mesh-faces",
    )?;
    let faces = read_faces(context, faces_header, vertex_count)?;

    let index_counts_header = context.read_array_header(
        absolute + MESH_INDEX_COUNTS_OFFSET,
        4,
        "mesh index counts",
        "mesh-index-counts",
    )?;
    let raw_index_offsets_header = context.read_array_header(
        absolute + MESH_RAW_INDEX_OFFSETS_OFFSET,
        4,
        "mesh raw index offsets",
        "mesh-raw-index-offsets",
    )?;
    if index_counts_header.used != raw_index_offsets_header.used {
        return Err(ParseError::header(
            absolute + MESH_RAW_INDEX_OFFSETS_OFFSET + 4,
            format!(
                "mesh index count/offset arrays differ: {} != {}",
                index_counts_header.used, raw_index_offsets_header.used
            ),
        ));
    }
    let index_counts = read_core_u32_values(context, index_counts_header, "mesh index count")?;
    let raw_index_offsets =
        read_core_i32_values(context, raw_index_offsets_header, "mesh raw index offset")?;
    let raw_indices = read_raw_index_lists(
        context,
        &index_counts,
        &raw_index_offsets,
        vertex_count,
        "mesh raw indices",
    )?;
    let mut validated_raw_pointers = Vec::new();
    let unknown = read_and_validate_raw_pointer(
        context,
        absolute + MESH_UNKNOWN_RAW_OFFSET,
        "unknown0",
        usize::from(vertex_count > 0),
        false,
        &mut validated_raw_pointers,
    )?;
    let _ = unknown;
    read_and_validate_raw_pointer(
        context,
        absolute + MESH_START_RAW_OFFSET,
        "startMdx",
        usize::from(vertex_count > 0),
        vertex_count > 0,
        &mut validated_raw_pointers,
    )?;
    let vertices_pointer = read_and_validate_raw_pointer(
        context,
        absolute + MESH_VERTICES_RAW_OFFSET,
        "vertices",
        checked_array_size(vertex_count, 12, absolute, "mesh vertices")?,
        vertex_count > 0,
        &mut validated_raw_pointers,
    )?;
    let uv0_pointer = read_and_validate_raw_pointer(
        context,
        absolute + MESH_UV0_RAW_OFFSET,
        "uv0",
        checked_array_size(vertex_count, 8, absolute, "mesh UV0")?,
        vertex_count > 0,
        &mut validated_raw_pointers,
    )?;
    for index in 1..4 {
        read_and_validate_raw_pointer(
            context,
            absolute + MESH_UV0_RAW_OFFSET + index * 4,
            match index {
                1 => "uv1",
                2 => "uv2",
                _ => "uv3",
            },
            checked_array_size(vertex_count, 8, absolute, "mesh deferred UV")?,
            false,
            &mut validated_raw_pointers,
        )?;
    }
    let normals_pointer = read_and_validate_raw_pointer(
        context,
        absolute + MESH_NORMALS_RAW_OFFSET,
        "normals",
        checked_array_size(vertex_count, 12, absolute, "mesh normals")?,
        vertex_count > 0,
        &mut validated_raw_pointers,
    )?;
    read_and_validate_raw_pointer(
        context,
        absolute + MESH_COLORS_RAW_OFFSET,
        "colors",
        checked_array_size(vertex_count, 4, absolute, "mesh colors")?,
        false,
        &mut validated_raw_pointers,
    )?;
    for index in 0..6 {
        let element_size = if index == 5 { 4 } else { 12 };
        read_and_validate_raw_pointer(
            context,
            absolute + MESH_TEX_ANIM0_RAW_OFFSET + index * 4,
            match index {
                0 => "textureAnimation0",
                1 => "textureAnimation1",
                2 => "textureAnimation2",
                3 => "textureAnimation3",
                4 => "textureAnimation4",
                _ => "textureAnimation5",
            },
            checked_array_size(
                vertex_count,
                element_size,
                absolute,
                "mesh texture animation",
            )?,
            false,
            &mut validated_raw_pointers,
        )?;
    }

    let vertices = read_raw_vec3_values(context, vertices_pointer, vertex_count, "mesh vertices")?;
    let uv0 = read_raw_vec2_values(context, uv0_pointer, vertex_count, "mesh UV0")?;
    let normals = read_raw_vec3_values(context, normals_pointer, vertex_count, "mesh normals")?;
    Ok(MeshReport {
        textures,
        vertex_count,
        texture_count,
        bounds_min: read_vec3(
            context.reader,
            absolute + MESH_BOUNDS_MIN_OFFSET,
            "mesh bounds min",
        )?,
        bounds_max: read_vec3(
            context.reader,
            absolute + MESH_BOUNDS_MAX_OFFSET,
            "mesh bounds max",
        )?,
        radius: context
            .reader
            .read_f32(absolute + MESH_RADIUS_OFFSET, "mesh radius")?,
        average: finite_vec3(read_vec3(
            context.reader,
            absolute + MESH_AVERAGE_OFFSET,
            "mesh average",
        )?),
        diffuse: read_f32x3(
            context.reader,
            absolute + MESH_DIFFUSE_OFFSET,
            "mesh diffuse",
        )?,
        ambient: read_f32x3(
            context.reader,
            absolute + MESH_AMBIENT_OFFSET,
            "mesh ambient",
        )?,
        specular: read_f32x3(
            context.reader,
            absolute + MESH_SPECULAR_OFFSET,
            "mesh specular",
        )?,
        shininess: context
            .reader
            .read_f32(absolute + MESH_SHININESS_OFFSET, "mesh shininess")?,
        shadow: context
            .reader
            .read_u32(absolute + MESH_SHADOW_OFFSET, "mesh shadow")?,
        beaming: context
            .reader
            .read_u32(absolute + MESH_BEAMING_OFFSET, "mesh beaming")?,
        render: context
            .reader
            .read_u32(absolute + MESH_RENDER_OFFSET, "mesh render")?,
        transparency: context
            .reader
            .read_u32(absolute + MESH_TRANSPARENCY_OFFSET, "mesh transparency")?,
        render_hint: context
            .reader
            .read_u32(absolute + MESH_RENDER_HINT_OFFSET, "mesh render hint")?,
        tile_fade: context
            .reader
            .read_u32(absolute + MESH_TILE_FADE_OFFSET, "mesh tile fade")?,
        mesh_type: context
            .reader
            .read_u32(absolute + MESH_TYPE_OFFSET, "mesh type")?,
        start_mdx: context
            .reader
            .read_i32(absolute + MESH_START_RAW_OFFSET, "mesh start MDX")?,
        faces,
        index_counts,
        raw_index_offsets,
        raw_indices,
        vertices,
        uv0,
        normals,
        validated_raw_pointers,
    })
}

fn read_faces(
    context: &mut ParseContext<'_, '_>,
    header: ArrayHeader,
    vertex_count: usize,
) -> Result<Vec<FaceReport>, ParseError> {
    if header.used == 0 {
        return Ok(Vec::new());
    }
    let absolute = context.core_absolute(
        header.pointer,
        checked_array_size(header.used, 0x20, header.pointer as usize, "mesh faces")?,
        "mesh faces",
    )?;
    let mut faces = Vec::with_capacity(header.used);
    for index in 0..header.used {
        let face = absolute + index * 0x20;
        let vertex_indices = [
            context.reader.read_u16(face + 0x1a, "face vertex 0")?,
            context.reader.read_u16(face + 0x1c, "face vertex 1")?,
            context.reader.read_u16(face + 0x1e, "face vertex 2")?,
        ];
        if vertex_indices
            .iter()
            .any(|value| *value as usize >= vertex_count)
        {
            return Err(ParseError::header(
                face + 0x1a,
                format!(
                    "face vertex indices {vertex_indices:?} exceed vertex count {vertex_count}"
                ),
            ));
        }
        faces.push(FaceReport {
            normal: read_vec3(context.reader, face, "face normal")?,
            distance: context.reader.read_f32(face + 0x0c, "face distance")?,
            surface_id: context.reader.read_i32(face + 0x10, "face surface id")?,
            adjacent_faces: [
                context.reader.read_i16(face + 0x14, "face adjacent 0")?,
                context.reader.read_i16(face + 0x16, "face adjacent 1")?,
                context.reader.read_i16(face + 0x18, "face adjacent 2")?,
            ],
            vertex_indices,
        });
    }
    Ok(faces)
}

fn read_and_validate_raw_pointer(
    context: &ParseContext<'_, '_>,
    field_absolute: usize,
    field: &str,
    length: usize,
    required: bool,
    reports: &mut Vec<RawPointerReport>,
) -> Result<Option<usize>, ParseError> {
    let pointer = context.reader.read_i32(field_absolute, field)?;
    let absolute = context.raw_absolute(pointer, length, field, required)?;
    reports.push(RawPointerReport {
        field: field.to_owned(),
        pointer: (pointer != -1).then_some(pointer),
        validated_length: if pointer == -1 { 0 } else { length },
    });
    Ok(absolute)
}

fn validate_skin_bind_count(
    header: ArrayHeader,
    header_absolute: usize,
    map_count: usize,
    max_skin_bone_count: usize,
    context: &str,
) -> Result<(), ParseError> {
    if header.allocated > max_skin_bone_count {
        return Err(ParseError::limit(
            header_absolute + 8,
            format!(
                "{context} allocated count {} exceeds skin bone guardrail {max_skin_bone_count}",
                header.allocated
            ),
        ));
    }
    if header.used != map_count {
        return Err(ParseError::header(
            header_absolute + 4,
            format!(
                "{context} used count {} does not match skin node-to-bone count {map_count}",
                header.used
            ),
        ));
    }
    Ok(())
}

fn read_skin(
    context: &mut ParseContext<'_, '_>,
    node_offset: u32,
    vertex_count: usize,
) -> Result<SkinReport, ParseError> {
    let common_absolute = context.core_absolute(
        node_offset,
        SKIN_INLINE_MAPPING_OFFSET,
        "skin common prefix",
    )?;
    let map_pointer_signed = context.reader.read_i32(
        common_absolute + SKIN_NODE_TO_BONE_OFFSET,
        "skin node-to-bone pointer",
    )?;
    if map_pointer_signed < 0 {
        return Err(ParseError::skin_variant(
            common_absolute + SKIN_NODE_TO_BONE_OFFSET,
            format!("skin node-to-bone pointer {map_pointer_signed} matches no profile"),
        ));
    }
    let map_pointer = map_pointer_signed as u32;
    let legacy_pointer = node_offset
        .checked_add(SKIN_LEGACY17_SIZE as u32)
        .ok_or_else(|| ParseError::skin_variant(common_absolute, "legacy skin offset overflow"))?;
    let extended_pointer = node_offset
        .checked_add(SKIN_EXTENDED64_SIZE as u32)
        .ok_or_else(|| {
            ParseError::skin_variant(common_absolute, "extended skin offset overflow")
        })?;
    let legacy_matches = map_pointer == legacy_pointer;
    let extended_matches = map_pointer == extended_pointer;
    let (variant, profile_size, inline_count) = match (legacy_matches, extended_matches) {
        (true, false) => (SkinVariant::Legacy17, SKIN_LEGACY17_SIZE, 17),
        (false, true) => (SkinVariant::Extended64, SKIN_EXTENDED64_SIZE, 64),
        _ => {
            return Err(ParseError::skin_variant(
                common_absolute + SKIN_NODE_TO_BONE_OFFSET,
                format!(
                    "skin node-to-bone pointer 0x{map_pointer:08x} matches neither explicit skin profile"
                ),
            ));
        }
    };
    context.claim_core(node_offset, profile_size, "skin-node")?;
    let absolute = context.core_absolute(node_offset, profile_size, "skin profile header")?;
    let weights_header = context.read_array_header(
        absolute + SKIN_WEIGHTS_HEADER_OFFSET,
        4,
        "skin weights metadata",
        "skin-weights-metadata",
    )?;
    let map_count_signed = context
        .reader
        .read_i32(absolute + SKIN_MAP_COUNT_OFFSET, "skin node-to-bone count")?;
    if map_count_signed < 0 {
        return Err(ParseError::header(
            absolute + SKIN_MAP_COUNT_OFFSET,
            format!("skin node-to-bone count is negative: {map_count_signed}"),
        ));
    }
    let map_count = map_count_signed as usize;
    if map_count > context.limits.max_skin_bone_count {
        return Err(ParseError::limit(
            absolute + SKIN_MAP_COUNT_OFFSET,
            format!(
                "skin node-to-bone count {map_count} exceeds skin bone guardrail {}",
                context.limits.max_skin_bone_count
            ),
        ));
    }
    let map_byte_length =
        checked_array_size(map_count, 2, map_pointer as usize, "skin node-to-bone map")?;
    let map_absolute =
        context.core_absolute(map_pointer, map_byte_length, "skin node-to-bone map")?;
    context.claim_core(map_pointer, map_byte_length, "skin-node-to-bone-map")?;
    let mut node_to_bone_map = Vec::with_capacity(map_count);
    for index in 0..map_count {
        node_to_bone_map.push(
            context
                .reader
                .read_i16(map_absolute + index * 2, "skin node-to-bone entry")?,
        );
    }

    let q_header = context.read_array_header(
        absolute + SKIN_Q_INV_OFFSET,
        16,
        "skin inverse bone rotations",
        "skin-q-inverse",
    )?;
    validate_skin_bind_count(
        q_header,
        absolute + SKIN_Q_INV_OFFSET,
        map_count,
        context.limits.max_skin_bone_count,
        "skin inverse bone rotations",
    )?;
    let t_header = context.read_array_header(
        absolute + SKIN_T_INV_OFFSET,
        12,
        "skin inverse bone translations",
        "skin-t-inverse",
    )?;
    validate_skin_bind_count(
        t_header,
        absolute + SKIN_T_INV_OFFSET,
        map_count,
        context.limits.max_skin_bone_count,
        "skin inverse bone translations",
    )?;
    let constants_header = context.read_array_header(
        absolute + SKIN_CONSTANTS_OFFSET,
        4,
        "skin bone constants",
        "skin-bone-constants",
    )?;
    validate_skin_bind_count(
        constants_header,
        absolute + SKIN_CONSTANTS_OFFSET,
        map_count,
        context.limits.max_skin_bone_count,
        "skin bone constants",
    )?;
    let inverse_bone_rotations_raw = read_core_f32x4(context, q_header, "skin qBoneRefInv")?;
    let inverse_bone_translations = read_core_vec3(context, t_header, "skin tBoneRefInv")?;
    let bone_constants = read_core_i16x2(context, constants_header, "skin bone constants")?;
    let mut inline_mapping = Vec::with_capacity(inline_count);
    for index in 0..inline_count {
        inline_mapping.push(context.reader.read_i16(
            absolute + SKIN_INLINE_MAPPING_OFFSET + index * 2,
            "skin inline mapping",
        )?);
    }

    let raw_weights_pointer = context.reader.read_i32(
        absolute + SKIN_RAW_WEIGHTS_OFFSET,
        "skin raw weights pointer",
    )?;
    let raw_refs_pointer = context.reader.read_i32(
        absolute + SKIN_RAW_BONE_REFS_OFFSET,
        "skin raw bone refs pointer",
    )?;
    let vertex_weights = read_raw_f32x4(
        context,
        raw_weights_pointer,
        vertex_count,
        "skin raw vertex weights",
    )?;
    let bone_references = read_raw_u16x4(
        context,
        raw_refs_pointer,
        vertex_count,
        map_count,
        &vertex_weights,
        "skin raw bone references",
    )?;
    Ok(SkinReport {
        variant,
        node_offset,
        header_size: profile_size,
        node_to_bone_pointer: map_pointer,
        raw_weights_pointer,
        raw_refs_pointer,
        weights_header: weights_header.into(),
        q_header: q_header.into(),
        t_header: t_header.into(),
        constants_header: constants_header.into(),
        node_to_bone_map,
        inverse_bone_rotations_raw,
        inverse_bone_translations,
        bone_constants,
        inline_mapping,
        vertex_weights,
        bone_references,
    })
}

fn parse_animations(
    context: &mut ParseContext<'_, '_>,
    model_absolute: usize,
) -> Result<(ArrayReport, Vec<AnimationReport>), ParseError> {
    let header = context.read_array_header(
        model_absolute + 0x78,
        POINTER_SIZE,
        "model animation pointers",
        "animation-pointers",
    )?;
    let pointers = read_core_u32_values(context, header, "animation pointer")?;
    let mut reports = Vec::with_capacity(pointers.len());
    for pointer in pointers {
        context.claim_core(pointer, ANIMATION_HEADER_SIZE, "animation-header")?;
        let absolute = context.core_absolute(pointer, ANIMATION_HEADER_SIZE, "animation header")?;
        let root_pointer = context
            .reader
            .read_u32(absolute + GEOMETRY_ROOT_OFFSET, "animation root pointer")?;
        let declared = read_declared_node_count(
            context.reader,
            absolute + GEOMETRY_NODE_COUNT_OFFSET,
            root_pointer,
            context.limits,
            "animation",
        )?;
        let event_header = context.read_array_header(
            absolute + 0xb8,
            ANIMATION_EVENT_SIZE,
            "animation events",
            "animation-events",
        )?;
        let events = read_animation_events(context, event_header)?;
        let node_tree = parse_node_tree(context, root_pointer, declared, "animation root")?;
        reports.push(AnimationReport {
            offset: pointer,
            name: context.reader.read_fixed_string(
                absolute + GEOMETRY_NAME_OFFSET,
                GEOMETRY_NAME_LENGTH,
                "animation name",
            )?,
            geometry_array_50: context
                .read_array_header(
                    absolute + 0x50,
                    1,
                    "animation opaque geometry array 0x50",
                    "animation-geometry-array-50",
                )?
                .into(),
            geometry_array_5c: context
                .read_array_header(
                    absolute + 0x5c,
                    1,
                    "animation opaque geometry array 0x5c",
                    "animation-geometry-array-5c",
                )?
                .into(),
            runtime_68: context
                .reader
                .read_u32(absolute + 0x68, "animation opaque runtime field 0x68")?,
            runtime_6c: context
                .reader
                .read_u32(absolute + 0x6c, "animation opaque runtime field 0x6c")?,
            length: context
                .reader
                .read_f32(absolute + 0x70, "animation length")?,
            transition: context
                .reader
                .read_f32(absolute + 0x74, "animation transition")?,
            animation_root: context.reader.read_fixed_string(
                absolute + 0x78,
                64,
                "animation root name",
            )?,
            events_header: event_header.into(),
            events,
            node_tree,
        });
    }
    Ok((header.into(), reports))
}

fn read_animation_events(
    context: &mut ParseContext<'_, '_>,
    header: ArrayHeader,
) -> Result<Vec<AnimationEventReport>, ParseError> {
    if header.used == 0 {
        return Ok(Vec::new());
    }
    let absolute = context.core_absolute(
        header.pointer,
        checked_array_size(
            header.used,
            ANIMATION_EVENT_SIZE,
            header.pointer as usize,
            "animation events",
        )?,
        "animation events",
    )?;
    let mut events = Vec::with_capacity(header.used);
    for index in 0..header.used {
        let event = absolute + index * ANIMATION_EVENT_SIZE;
        events.push(AnimationEventReport {
            time: context.reader.read_f32(event, "animation event time")?,
            name: context
                .reader
                .read_fixed_string(event + 4, 32, "animation event name")?,
        });
    }
    Ok(events)
}

fn read_core_u32_values(
    context: &ParseContext<'_, '_>,
    header: ArrayHeader,
    value_context: &str,
) -> Result<Vec<u32>, ParseError> {
    if header.used == 0 {
        return Ok(Vec::new());
    }
    let absolute = context.core_absolute(
        header.pointer,
        checked_array_size(header.used, 4, header.pointer as usize, value_context)?,
        value_context,
    )?;
    let mut values = Vec::with_capacity(header.used);
    for index in 0..header.used {
        values.push(
            context
                .reader
                .read_u32(absolute + index * 4, value_context)?,
        );
    }
    Ok(values)
}

fn read_core_i32_values(
    context: &ParseContext<'_, '_>,
    header: ArrayHeader,
    value_context: &str,
) -> Result<Vec<i32>, ParseError> {
    if header.used == 0 {
        return Ok(Vec::new());
    }
    let absolute = context.core_absolute(
        header.pointer,
        checked_array_size(header.used, 4, header.pointer as usize, value_context)?,
        value_context,
    )?;
    let mut values = Vec::with_capacity(header.used);
    for index in 0..header.used {
        values.push(
            context
                .reader
                .read_i32(absolute + index * 4, value_context)?,
        );
    }
    Ok(values)
}

fn read_core_f32x4(
    context: &ParseContext<'_, '_>,
    header: ArrayHeader,
    value_context: &str,
) -> Result<Vec<[f32; 4]>, ParseError> {
    if header.used == 0 {
        return Ok(Vec::new());
    }
    let absolute = context.core_absolute(
        header.pointer,
        checked_array_size(header.used, 16, header.pointer as usize, value_context)?,
        value_context,
    )?;
    let mut values = Vec::with_capacity(header.used);
    for row in 0..header.used {
        let base = absolute + row * 16;
        values.push([
            context.reader.read_f32(base, value_context)?,
            context.reader.read_f32(base + 4, value_context)?,
            context.reader.read_f32(base + 8, value_context)?,
            context.reader.read_f32(base + 12, value_context)?,
        ]);
    }
    Ok(values)
}

fn read_core_vec3(
    context: &ParseContext<'_, '_>,
    header: ArrayHeader,
    value_context: &str,
) -> Result<Vec<Vec3>, ParseError> {
    if header.used == 0 {
        return Ok(Vec::new());
    }
    let absolute = context.core_absolute(
        header.pointer,
        checked_array_size(header.used, 12, header.pointer as usize, value_context)?,
        value_context,
    )?;
    let mut values = Vec::with_capacity(header.used);
    for row in 0..header.used {
        values.push(read_vec3(
            context.reader,
            absolute + row * 12,
            value_context,
        )?);
    }
    Ok(values)
}

fn read_core_i16x2(
    context: &ParseContext<'_, '_>,
    header: ArrayHeader,
    value_context: &str,
) -> Result<Vec<[i16; 2]>, ParseError> {
    if header.used == 0 {
        return Ok(Vec::new());
    }
    let absolute = context.core_absolute(
        header.pointer,
        checked_array_size(header.used, 4, header.pointer as usize, value_context)?,
        value_context,
    )?;
    let mut values = Vec::with_capacity(header.used);
    for row in 0..header.used {
        let base = absolute + row * 4;
        values.push([
            context.reader.read_i16(base, value_context)?,
            context.reader.read_i16(base + 2, value_context)?,
        ]);
    }
    Ok(values)
}

fn read_raw_vec3_values(
    context: &ParseContext<'_, '_>,
    absolute: Option<usize>,
    count: usize,
    value_context: &str,
) -> Result<Vec<Vec3>, ParseError> {
    let Some(absolute) = absolute else {
        return Ok(Vec::new());
    };
    let mut values = Vec::with_capacity(count);
    for index in 0..count {
        values.push(read_vec3(
            context.reader,
            absolute + index * 12,
            value_context,
        )?);
    }
    Ok(values)
}

fn read_raw_vec2_values(
    context: &ParseContext<'_, '_>,
    absolute: Option<usize>,
    count: usize,
    value_context: &str,
) -> Result<Vec<Vec2>, ParseError> {
    let Some(absolute) = absolute else {
        return Ok(Vec::new());
    };
    let mut values = Vec::with_capacity(count);
    for index in 0..count {
        let base = absolute + index * 8;
        values.push(Vec2 {
            x: context.reader.read_f32(base, value_context)?,
            y: context.reader.read_f32(base + 4, value_context)?,
        });
    }
    Ok(values)
}

fn read_raw_f32x4(
    context: &ParseContext<'_, '_>,
    pointer: i32,
    count: usize,
    value_context: &str,
) -> Result<Vec<[f32; 4]>, ParseError> {
    if count == 0 {
        context.raw_absolute(pointer, 0, value_context, false)?;
        return Ok(Vec::new());
    }
    let length = checked_array_size(count, 16, pointer.max(0) as usize, value_context)?;
    let absolute = context
        .raw_absolute(pointer, length, value_context, true)?
        .expect("required pointer was checked");
    let mut values = Vec::with_capacity(count);
    for index in 0..count {
        let base = absolute + index * 16;
        values.push([
            context.reader.read_f32(base, value_context)?,
            context.reader.read_f32(base + 4, value_context)?,
            context.reader.read_f32(base + 8, value_context)?,
            context.reader.read_f32(base + 12, value_context)?,
        ]);
    }
    Ok(values)
}

fn read_raw_index_lists(
    context: &ParseContext<'_, '_>,
    counts: &[u32],
    pointers: &[i32],
    vertex_count: usize,
    value_context: &str,
) -> Result<Vec<Vec<u16>>, ParseError> {
    let mut lists = Vec::with_capacity(counts.len());
    for (list_index, (&count, &pointer)) in counts.iter().zip(pointers).enumerate() {
        let count = usize::try_from(count).map_err(|_| {
            ParseError::pointer(
                pointer.max(0) as usize,
                format!("{value_context} count does not fit usize"),
            )
        })?;
        if count == 0 {
            let _ = context.raw_absolute(pointer, 0, value_context, false)?;
            lists.push(Vec::new());
            continue;
        }
        let length = checked_array_size(count, 2, pointer.max(0) as usize, "mesh raw index list")?;
        let absolute = context
            .raw_absolute(pointer, length, value_context, count > 0)?
            .ok_or_else(|| {
                ParseError::pointer(
                    pointer.max(0) as usize,
                    format!("{value_context} list {list_index} has no pointer"),
                )
            })?;
        let mut values = Vec::with_capacity(count);
        for index in 0..count {
            let value = context
                .reader
                .read_u16(absolute + index * 2, value_context)?;
            if value as usize >= vertex_count {
                return Err(ParseError::header(
                    absolute + index * 2,
                    format!("mesh raw index {value} exceeds vertex count {vertex_count}"),
                ));
            }
            values.push(value);
        }
        lists.push(values);
    }
    Ok(lists)
}

fn read_raw_u16x4(
    context: &ParseContext<'_, '_>,
    pointer: i32,
    count: usize,
    map_count: usize,
    weights: &[[f32; 4]],
    value_context: &str,
) -> Result<Vec<[u16; 4]>, ParseError> {
    if count == 0 {
        context.raw_absolute(pointer, 0, value_context, false)?;
        return Ok(Vec::new());
    }
    let length = checked_array_size(count, 8, pointer.max(0) as usize, value_context)?;
    let absolute = context
        .raw_absolute(pointer, length, value_context, true)?
        .expect("required pointer was checked");
    let mut values = Vec::with_capacity(count);
    for (index, weight_row) in weights.iter().enumerate().take(count) {
        let base = absolute + index * 8;
        let row = [
            context.reader.read_u16(base, value_context)?,
            context.reader.read_u16(base + 2, value_context)?,
            context.reader.read_u16(base + 4, value_context)?,
            context.reader.read_u16(base + 6, value_context)?,
        ];
        for (influence, value) in row.iter().enumerate() {
            if *value == u16::MAX {
                if weight_row[influence] != 0.0 {
                    return Err(ParseError::bone_ref(
                        base + influence * 2,
                        format!(
                            "bone reference 0xffff has non-zero weight {}",
                            weight_row[influence]
                        ),
                    ));
                }
            } else if *value as usize >= map_count {
                return Err(ParseError::bone_ref(
                    base + influence * 2,
                    format!("bone reference {value} exceeds skin node-to-bone count {map_count}"),
                ));
            }
        }
        values.push(row);
    }
    Ok(values)
}

fn read_vec3(
    reader: &BinaryReader<'_>,
    absolute: usize,
    context: &str,
) -> Result<Vec3, ParseError> {
    Ok(Vec3 {
        x: reader.read_f32(absolute, context)?,
        y: reader.read_f32(absolute + 4, context)?,
        z: reader.read_f32(absolute + 8, context)?,
    })
}

fn read_f32x3(
    reader: &BinaryReader<'_>,
    absolute: usize,
    context: &str,
) -> Result<[f32; 3], ParseError> {
    Ok([
        reader.read_f32(absolute, context)?,
        reader.read_f32(absolute + 4, context)?,
        reader.read_f32(absolute + 8, context)?,
    ])
}

fn finite_vec3(value: Vec3) -> Option<Vec3> {
    (value.x.is_finite() && value.y.is_finite() && value.z.is_finite()).then_some(value)
}

fn node_family_bits() -> [(u32, &'static str); 10] {
    [
        (FLAG_HEADER, "header"),
        (FLAG_LIGHT, "light"),
        (FLAG_EMITTER, "emitter"),
        (FLAG_CAMERA, "camera"),
        (FLAG_REFERENCE, "reference"),
        (FLAG_MESH, "mesh"),
        (FLAG_SKIN, "skin"),
        (FLAG_ANIMMESH, "animmesh"),
        (FLAG_DANGLY, "dangly"),
        (FLAG_AABB, "aabb"),
    ]
}

fn core_pointer(offset: u32) -> Result<usize, ParseError> {
    FILE_HEADER_SIZE
        .checked_add(offset as usize)
        .ok_or_else(|| ParseError::pointer(offset as usize, "core pointer overflow"))
}

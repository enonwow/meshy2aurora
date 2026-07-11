use std::collections::{HashMap, HashSet};

use super::binary_reader::{BinaryReader, checked_array_size};
use super::errors::{HEADER_INVALID, NODE_CYCLE, ParseError};
use super::types::{
    Diagnostic, FileHeaderReport, InspectionReport, ModelReport, NodeReport, NodeTreeReport,
    ParserLimits,
};

// Profile-A fixed-layout constants. Provenance:
// documentation/mdl-binary-crosswalk-codex.md and
// documentation/standalone-odpowiedz-codex.md (reference layout).
const FILE_HEADER_SIZE: usize = 0x0c;
const GEOMETRY_HEADER_SIZE: usize = 0x70;
const MODEL_HEADER_SIZE: usize = 0xe8;
const NODE_HEADER_SIZE: usize = 0x70;
const ARRAY_HEADER_SIZE: usize = 0x0c;
const POINTER_SIZE: usize = 4;

const GEOMETRY_NAME_OFFSET: usize = 0x08;
const GEOMETRY_NAME_LENGTH: usize = 64;
const GEOMETRY_ROOT_OFFSET: usize = 0x48;
const GEOMETRY_NODE_COUNT_OFFSET: usize = 0x4c;

const NODE_NUMBER_OFFSET: usize = 0x1c;
const NODE_NAME_OFFSET: usize = 0x20;
const NODE_NAME_LENGTH: usize = 32;
const NODE_GEOMETRY_OFFSET: usize = 0x40;
const NODE_PARENT_OFFSET: usize = 0x44;
const NODE_CHILDREN_OFFSET: usize = 0x48;
const NODE_CONTROLLER_KEYS_OFFSET: usize = 0x54;
const NODE_CONTROLLER_DATA_OFFSET: usize = 0x60;
const NODE_CONTENT_OFFSET: usize = 0x6c;
const CONTROLLER_KEY_SIZE: usize = 0x0c;
const CONTROLLER_DATA_SIZE: usize = 4;
const NODE_HEADER_CONTENT_FLAG: u32 = 0x001;

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
/// Returns a stable [`ParseError`] when the input is malformed, unsupported at
/// the structural boundary, or exceeds one of `limits`.
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

    let mdx_start = reader.read_u32(4, "p_start_mdx")?;
    let mdx_size = reader.read_u32(8, "size_mdx")?;
    let mdx_absolute = FILE_HEADER_SIZE
        .checked_add(mdx_start as usize)
        .ok_or_else(|| ParseError::pointer(4, "MDX start overflow"))?;
    reader.read_slice(mdx_absolute, mdx_size as usize, "appended MDX range")?;
    let core_length = mdx_start as usize;
    reader.read_slice(FILE_HEADER_SIZE, core_length, "core MDL block")?;

    if core_length < MODEL_HEADER_SIZE {
        return Err(ParseError::pointer(
            FILE_HEADER_SIZE,
            format!(
                "core MDL block length {core_length} is smaller than model header {MODEL_HEADER_SIZE}"
            ),
        ));
    }
    let model_name = reader.read_fixed_string(
        FILE_HEADER_SIZE + GEOMETRY_NAME_OFFSET,
        GEOMETRY_NAME_LENGTH,
        "model name",
    )?;
    let root_node_offset = reader.read_u32(
        FILE_HEADER_SIZE + GEOMETRY_ROOT_OFFSET,
        "model root node pointer",
    )?;
    if root_node_offset == 0 {
        return Err(ParseError::pointer(
            FILE_HEADER_SIZE + GEOMETRY_ROOT_OFFSET,
            "model root node pointer is null",
        ));
    }
    if (root_node_offset as usize) < MODEL_HEADER_SIZE {
        return Err(ParseError::pointer(
            FILE_HEADER_SIZE + GEOMETRY_ROOT_OFFSET,
            format!("root node offset 0x{root_node_offset:08x} overlaps the model header"),
        ));
    }

    let declared_node_count = reader.read_u32(
        FILE_HEADER_SIZE + GEOMETRY_NODE_COUNT_OFFSET,
        "model node count",
    )? as usize;
    if declared_node_count > limits.max_nodes {
        return Err(ParseError::limit(
            FILE_HEADER_SIZE + GEOMETRY_NODE_COUNT_OFFSET,
            format!(
                "declared node count {declared_node_count} exceeds product guardrail {}",
                limits.max_nodes
            ),
        ));
    }

    let mut state = TraversalState::new(&reader, limits, core_length);
    let root = state.parse_tree(root_node_offset)?;

    if state.node_count != declared_node_count {
        return Err(ParseError::header(
            FILE_HEADER_SIZE + GEOMETRY_NODE_COUNT_OFFSET,
            format!(
                "declared node count {declared_node_count} differs from traversed count {}",
                state.node_count
            ),
        ));
    }

    Ok(InspectionReport {
        schema_version: 1,
        format: "nwn1-binary-mdl".to_owned(),
        byte_length: bytes.len(),
        file_header: FileHeaderReport {
            binary_mdl_id,
            mdx_start,
            mdx_size,
            mdx_range_in_bounds: true,
        },
        model: ModelReport {
            name: model_name,
            root_node_offset,
        },
        node_tree: NodeTreeReport {
            node_count: state.node_count,
            max_depth: state.max_depth,
            roots: vec![root],
        },
        unsupported: state.unsupported,
        diagnostics: state.diagnostics,
    })
}

struct TraversalState<'reader, 'input> {
    reader: &'reader BinaryReader<'input>,
    limits: &'reader ParserLimits,
    core_length: usize,
    seen: HashSet<u32>,
    node_count: usize,
    max_depth: usize,
    unsupported: Vec<String>,
    diagnostics: Vec<Diagnostic>,
}

impl<'reader, 'input> TraversalState<'reader, 'input> {
    fn new(
        reader: &'reader BinaryReader<'input>,
        limits: &'reader ParserLimits,
        core_length: usize,
    ) -> Self {
        Self {
            reader,
            limits,
            core_length,
            seen: HashSet::new(),
            node_count: 0,
            max_depth: 0,
            unsupported: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn parse_tree(&mut self, root_offset: u32) -> Result<NodeReport, ParseError> {
        let mut pending = vec![(root_offset, None, 0_usize)];
        let mut flat_nodes = Vec::new();

        while let Some((node_offset, expected_parent, depth)) = pending.pop() {
            if depth > self.limits.max_depth {
                return Err(ParseError::limit(
                    core_pointer(node_offset)?,
                    format!(
                        "node depth {depth} exceeds product guardrail {}",
                        self.limits.max_depth
                    ),
                ));
            }
            if !self.seen.insert(node_offset) {
                return Err(ParseError::new(
                    NODE_CYCLE,
                    core_pointer(node_offset)?,
                    format!("node pointer 0x{node_offset:08x} was visited twice"),
                ));
            }
            if self.node_count >= self.limits.max_nodes {
                return Err(ParseError::limit(
                    core_pointer(node_offset)?,
                    format!(
                        "node traversal exceeds product guardrail {}",
                        self.limits.max_nodes
                    ),
                ));
            }

            let node = self.read_node(node_offset, expected_parent, depth)?;
            let child_depth = depth
                .checked_add(1)
                .ok_or_else(|| ParseError::limit(0, "node depth overflow"))?;
            for &child_offset in node.child_offsets.iter().rev() {
                pending.push((child_offset, Some(node_offset), child_depth));
            }
            flat_nodes.push(node);
        }

        let mut built = HashMap::with_capacity(flat_nodes.len());
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
                    content_flags: node.content_flags,
                    children,
                },
            );
        }

        built.remove(&root_offset).ok_or_else(|| {
            ParseError::header(
                core_pointer(root_offset).unwrap_or_default(),
                "root node was not available during tree assembly",
            )
        })
    }

    fn read_node(
        &mut self,
        node_offset: u32,
        expected_parent: Option<u32>,
        depth: usize,
    ) -> Result<FlatNode, ParseError> {
        if (node_offset as usize) < MODEL_HEADER_SIZE {
            return Err(ParseError::pointer(
                core_pointer(node_offset)?,
                format!("node offset 0x{node_offset:08x} overlaps the model header"),
            ));
        }
        let absolute = self.core_absolute(node_offset, NODE_HEADER_SIZE, "node header")?;
        let number = self
            .reader
            .read_u32(absolute + NODE_NUMBER_OFFSET, "node number")?;
        let name = self.reader.read_fixed_string(
            absolute + NODE_NAME_OFFSET,
            NODE_NAME_LENGTH,
            "node name",
        )?;
        let stored_parent = self
            .reader
            .read_u32(absolute + NODE_PARENT_OFFSET, "node parent pointer")?;
        let parent_offset = (stored_parent != 0).then_some(stored_parent);
        if parent_offset != expected_parent {
            return Err(ParseError::header(
                absolute + NODE_PARENT_OFFSET,
                format!(
                    "node parent {:?} differs from traversal parent {:?}",
                    parent_offset, expected_parent
                ),
            ));
        }

        let geometry_offset = self
            .reader
            .read_u32(absolute + NODE_GEOMETRY_OFFSET, "node geometry pointer")?;
        if geometry_offset != 0 {
            self.core_absolute(
                geometry_offset,
                GEOMETRY_HEADER_SIZE,
                "node geometry pointer",
            )?;
        }

        let children_header = absolute + NODE_CHILDREN_OFFSET;
        self.reader.read_slice(
            children_header,
            ARRAY_HEADER_SIZE,
            "node children array header",
        )?;
        let children_offset = self
            .reader
            .read_u32(children_header, "node children pointer")?;
        let children_count =
            self.reader
                .read_u32(children_header + 4, "node children used count")? as usize;
        let children_allocated = self
            .reader
            .read_u32(children_header + 8, "node children allocated count")?
            as usize;
        if children_count > children_allocated {
            return Err(ParseError::header(
                children_header + 4,
                format!(
                    "children used count {children_count} exceeds allocated count {children_allocated}"
                ),
            ));
        }
        if children_count > self.limits.max_nodes || children_allocated > self.limits.max_nodes {
            return Err(ParseError::limit(
                children_header + 4,
                format!(
                    "children used/allocated counts {children_count}/{children_allocated} exceed node guardrail {}",
                    self.limits.max_nodes
                ),
            ));
        }

        let child_pointers =
            self.read_child_pointers(children_offset, children_count, children_header)?;
        let content_flags = self
            .reader
            .read_u32(absolute + NODE_CONTENT_OFFSET, "node content flags")?;

        self.node_count += 1;
        self.max_depth = self.max_depth.max(depth);

        let deferred_content_flags = content_flags & !NODE_HEADER_CONTENT_FLAG;
        if deferred_content_flags != 0 {
            self.push_unsupported(
                absolute + NODE_CONTENT_OFFSET,
                format!(
                    "node {name:?} deferred content flags 0x{deferred_content_flags:08x}; payload parsing is outside M1A"
                ),
            )?;
        }

        self.validate_deferred_array(
            absolute + NODE_CONTROLLER_KEYS_OFFSET,
            CONTROLLER_KEY_SIZE,
            "node controller keys",
        )?;
        self.validate_deferred_array(
            absolute + NODE_CONTROLLER_DATA_OFFSET,
            CONTROLLER_DATA_SIZE,
            "node controller data",
        )?;

        Ok(FlatNode {
            offset: node_offset,
            number,
            name,
            parent_offset,
            content_flags,
            child_offsets: child_pointers,
        })
    }

    fn read_child_pointers(
        &self,
        children_offset: u32,
        children_count: usize,
        header_offset: usize,
    ) -> Result<Vec<u32>, ParseError> {
        if children_count == 0 {
            return Ok(Vec::new());
        }
        if children_offset == 0 {
            return Err(ParseError::pointer(
                header_offset,
                "non-empty children array has a null pointer",
            ));
        }

        let byte_length = checked_array_size(
            children_count,
            POINTER_SIZE,
            children_offset as usize,
            "node children pointer array",
        )?;
        let absolute =
            self.core_absolute(children_offset, byte_length, "node children pointer array")?;

        let mut pointers = Vec::with_capacity(children_count);
        for index in 0..children_count {
            let byte_offset = index
                .checked_mul(POINTER_SIZE)
                .and_then(|relative| absolute.checked_add(relative))
                .ok_or_else(|| {
                    ParseError::pointer(absolute, "node children pointer index overflow")
                })?;
            pointers.push(
                self.reader
                    .read_u32(byte_offset, "node child pointer entry")?,
            );
        }
        Ok(pointers)
    }

    fn validate_deferred_array(
        &mut self,
        header_offset: usize,
        element_size: usize,
        context: &str,
    ) -> Result<(), ParseError> {
        let pointer = self.reader.read_u32(header_offset, context)?;
        let used = self.reader.read_u32(header_offset + 4, context)? as usize;
        let allocated = self.reader.read_u32(header_offset + 8, context)? as usize;
        if used > allocated {
            return Err(ParseError::header(
                header_offset + 4,
                format!("{context} used count {used} exceeds allocated count {allocated}"),
            ));
        }
        let maximum_entries = self.limits.max_input_bytes / element_size;
        if allocated > maximum_entries {
            return Err(ParseError::limit(
                header_offset + 8,
                format!(
                    "{context} allocated count {allocated} exceeds product guardrail {maximum_entries}"
                ),
            ));
        }
        if used == 0 {
            return Ok(());
        }
        if pointer == 0 {
            return Err(ParseError::pointer(
                header_offset,
                format!("non-empty {context} array has a null pointer"),
            ));
        }

        let byte_length = checked_array_size(used, element_size, pointer as usize, context)?;
        let absolute = self.core_absolute(pointer, byte_length, context)?;
        self.reader.read_slice(absolute, byte_length, context)?;
        self.push_unsupported(
            absolute,
            format!("{context} contains {used} deferred entries"),
        )
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
        core_pointer(offset)
    }

    fn push_unsupported(&mut self, offset: usize, context: String) -> Result<(), ParseError> {
        if self.diagnostics.len() >= self.limits.max_diagnostics {
            return Err(ParseError::limit(
                offset,
                format!(
                    "diagnostic count would exceed product guardrail {}",
                    self.limits.max_diagnostics
                ),
            ));
        }
        self.unsupported.push(context.clone());
        self.diagnostics
            .push(Diagnostic::unsupported(offset, context));
        Ok(())
    }
}

struct FlatNode {
    offset: u32,
    number: u32,
    name: String,
    parent_offset: Option<u32>,
    content_flags: u32,
    child_offsets: Vec<u32>,
}

fn core_pointer(offset: u32) -> Result<usize, ParseError> {
    FILE_HEADER_SIZE
        .checked_add(offset as usize)
        .ok_or_else(|| ParseError::pointer(offset as usize, "core pointer overflow"))
}

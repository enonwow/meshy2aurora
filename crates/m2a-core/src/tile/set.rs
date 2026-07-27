use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::walkmesh::TileSurfaceV1;

const SET_SCHEMA_VERSION: u32 = 1;
const MAX_SET_BYTES: usize = 8 * 1024 * 1024;
const MAX_SET_SECTIONS: usize = 65_536;
const MAX_SET_ENTRIES: usize = 1_000_000;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSectionV1 {
    pub name: String,
    pub entries: Vec<(String, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetDocumentV1 {
    pub sections: Vec<SetSectionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TilesetGeneralV1 {
    pub name: String,
    pub set_type: String,
    pub version: String,
    pub display_name: String,
    pub unlocalized_name: String,
    pub interior: bool,
    pub has_height_transition: bool,
    pub transition: i32,
    pub env_map: String,
    pub border: String,
    pub default_terrain: String,
    pub floor: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileLightFlagsV1 {
    pub main: [bool; 2],
    pub source: [bool; 2],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileDescriptorV1 {
    pub tile_id: u32,
    pub model_resref: String,
    pub walkmesh_class_token: String,
    pub corner_terrain: [String; 4],
    pub corner_heights: [i32; 4],
    pub edge_crossers: [Option<String>; 4],
    pub path_node: String,
    pub visibility_node: Option<String>,
    pub visibility_orientation: i32,
    pub door_visibility_node: Option<String>,
    pub door_visibility_orientation: i32,
    pub orientation_quarter_turns: u8,
    pub image_map_2d: Option<String>,
    pub lights: TileLightFlagsV1,
    pub anim_loops: [bool; 3],
    pub doors: Vec<SetSectionV1>,
    pub sounds: Vec<SetSectionV1>,
}

impl TileDescriptorV1 {
    pub fn flat_v1(model_resref: &str, terrain: &str, surface: TileSurfaceV1) -> Self {
        let _ = surface;
        Self {
            tile_id: 0,
            model_resref: model_resref.to_owned(),
            // `WalkMesh` is an Aurora SET domain token, not the WOK resref
            // and not the corner-terrain label. The first audited static-tile
            // profile follows retail `tms01` and binds `msb01`.
            walkmesh_class_token: "msb01".to_owned(),
            corner_terrain: std::array::from_fn(|_| terrain.to_owned()),
            corner_heights: [0; 4],
            edge_crossers: std::array::from_fn(|_| None),
            path_node: String::new(),
            visibility_node: None,
            visibility_orientation: 0,
            door_visibility_node: None,
            door_visibility_orientation: 0,
            orientation_quarter_turns: 0,
            image_map_2d: Some(format!("{model_resref}_im")),
            lights: TileLightFlagsV1 {
                main: [false; 2],
                source: [false; 2],
            },
            anim_loops: [false; 3],
            doors: Vec::new(),
            sounds: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileGroupV1 {
    pub group_id: u32,
    pub rows: u32,
    pub columns: u32,
    pub tiles: Vec<Option<u32>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TilesetIrV1 {
    pub schema_version: u32,
    pub tileset_resref: String,
    pub document: SetDocumentV1,
    pub general: TilesetGeneralV1,
    pub terrain_types: Vec<SetSectionV1>,
    pub crosser_types: Vec<SetSectionV1>,
    pub primary_rules: Vec<SetSectionV1>,
    pub secondary_rules: Vec<SetSectionV1>,
    pub tiles: Vec<TileDescriptorV1>,
    pub groups: Vec<TileGroupV1>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileResourceBindingV1 {
    pub tile_id: u32,
    pub model_resref: String,
    pub wok_resref: String,
    pub walkmesh_class_token: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetArtifactV1 {
    pub payload: Vec<u8>,
    pub payload_sha256: String,
    pub inspection: TilesetIrV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileSetErrorV1 {
    pub schema_version: u32,
    pub code: String,
    pub path: String,
    pub message: String,
}

impl fmt::Display for TileSetErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

impl std::error::Error for TileSetErrorV1 {}

fn error(code: &str, path: impl Into<String>, message: impl Into<String>) -> TileSetErrorV1 {
    TileSetErrorV1 {
        schema_version: SET_SCHEMA_VERSION,
        code: code.to_owned(),
        path: path.into(),
        message: message.into(),
    }
}

pub fn minimal_static_tileset_v1(
    tileset_resref: &str,
    mut tile: TileDescriptorV1,
    interior: bool,
) -> Result<TilesetIrV1, TileSetErrorV1> {
    validate_resref(tileset_resref, "tilesetResref")?;
    validate_resref(&tile.model_resref, "tile.modelResref")?;
    if tile.tile_id != 0 {
        return Err(error(
            "TILE-SET-TILE-ID",
            "tile.tileId",
            "minimal V1 tileset requires exactly TILE0",
        ));
    }
    if tile.walkmesh_class_token.is_empty() || !tile.walkmesh_class_token.is_ascii() {
        return Err(error(
            "TILE-SET-WALKMESH-TOKEN",
            "tile.walkmeshClassToken",
            "WalkMesh is a non-empty ASCII domain token",
        ));
    }
    if tile.image_map_2d.is_none() {
        tile.image_map_2d = Some(format!("{}_im", tile.model_resref));
    }
    let general = TilesetGeneralV1 {
        name: tileset_resref.to_owned(),
        set_type: "SET".to_owned(),
        version: "V1.0".to_owned(),
        display_name: "-1".to_owned(),
        unlocalized_name: "Meshy2Aurora Static Tile V1".to_owned(),
        interior,
        has_height_transition: false,
        transition: 0,
        env_map: String::new(),
        border: tile.corner_terrain[0].clone(),
        default_terrain: tile.corner_terrain[0].clone(),
        floor: tile.corner_terrain[0].clone(),
    };
    let terrain = SetSectionV1 {
        name: "TERRAIN0".to_owned(),
        entries: vec![
            ("Name".to_owned(), tile.corner_terrain[0].clone()),
            ("StrRef".to_owned(), "-1".to_owned()),
        ],
    };
    let document = SetDocumentV1 {
        sections: vec![
            general_section(&general),
            SetSectionV1 {
                name: "GRASS".to_owned(),
                entries: vec![("Grass".to_owned(), "0".to_owned())],
            },
            count_section("TERRAIN TYPES", 1),
            terrain.clone(),
            count_section("CROSSER TYPES", 0),
            count_section("PRIMARY RULES", 0),
            count_section("SECONDARY RULES", 0),
            count_section("TILES", 1),
            tile_section(&tile),
            count_section("GROUPS", 0),
        ],
    };
    let tileset = TilesetIrV1 {
        schema_version: SET_SCHEMA_VERSION,
        tileset_resref: tileset_resref.to_owned(),
        document,
        general,
        terrain_types: vec![terrain],
        crosser_types: Vec::new(),
        primary_rules: Vec::new(),
        secondary_rules: Vec::new(),
        tiles: vec![tile],
        groups: Vec::new(),
    };
    validate_typed_tileset(&tileset)?;
    Ok(tileset)
}

pub fn parse_tileset_v1(bytes: &[u8]) -> Result<TilesetIrV1, TileSetErrorV1> {
    let document = parse_document(bytes)?;
    let sections = section_map(&document)?;
    let general_section = required_section(&sections, "GENERAL")?;
    let general = parse_general(general_section)?;
    if !general.set_type.eq_ignore_ascii_case("SET") {
        return Err(error(
            "TILE-SET-TYPE",
            "GENERAL.Type",
            "SET Type must be SET",
        ));
    }
    let terrain_types = parse_counted_sections(
        &sections,
        "TERRAIN TYPES",
        "TERRAIN",
        "TILE-SET-TERRAIN-COUNT-MISMATCH",
    )?;
    let crosser_types = parse_counted_sections(
        &sections,
        "CROSSER TYPES",
        "CROSSER",
        "TILE-SET-CROSSER-COUNT-MISMATCH",
    )?;
    let primary_rules = parse_counted_sections(
        &sections,
        "PRIMARY RULES",
        "PRIMARY RULE",
        "TILE-SET-PRIMARY-RULE-COUNT-MISMATCH",
    )?;
    let secondary_rules = parse_counted_sections(
        &sections,
        "SECONDARY RULES",
        "SECONDARY RULE",
        "TILE-SET-SECONDARY-RULE-COUNT-MISMATCH",
    )?;
    let tile_sections =
        parse_counted_sections(&sections, "TILES", "TILE", "TILE-SET-TILE-COUNT-MISMATCH")?;
    let mut tiles = Vec::with_capacity(tile_sections.len());
    for (tile_id, section) in tile_sections.iter().enumerate() {
        tiles.push(parse_tile(tile_id as u32, section, &document)?);
    }
    let group_sections = parse_counted_sections(
        &sections,
        "GROUPS",
        "GROUP",
        "TILE-SET-GROUP-COUNT-MISMATCH",
    )?;
    let mut groups = Vec::with_capacity(group_sections.len());
    for (group_id, section) in group_sections.iter().enumerate() {
        groups.push(parse_group(group_id as u32, section, tiles.len())?);
    }
    let tileset = TilesetIrV1 {
        schema_version: SET_SCHEMA_VERSION,
        tileset_resref: general.name.clone(),
        document,
        general,
        terrain_types,
        crosser_types,
        primary_rules,
        secondary_rules,
        tiles,
        groups,
    };
    validate_typed_tileset(&tileset)?;
    Ok(tileset)
}

pub fn write_tileset_v1(tileset: &TilesetIrV1) -> Result<SetArtifactV1, TileSetErrorV1> {
    validate_typed_tileset(tileset)?;
    let mut output = String::new();
    for (section_index, section) in tileset.document.sections.iter().enumerate() {
        if section_index != 0 {
            output.push('\n');
        }
        output.push('[');
        output.push_str(&section.name);
        output.push_str("]\n");
        for (key, value) in &section.entries {
            output.push_str(key);
            output.push('=');
            output.push_str(value);
            output.push('\n');
        }
    }
    let payload = output.into_bytes();
    let inspection = parse_tileset_v1(&payload)?;
    if inspection.document != tileset.document
        || inspection.tiles != tileset.tiles
        || inspection.groups != tileset.groups
    {
        return Err(error(
            "TILE-SET-SEMANTIC-DIFF",
            "payload",
            "parser -> writer -> parser semantic projection differs",
        ));
    }
    Ok(SetArtifactV1 {
        payload_sha256: sha256(&payload),
        payload,
        inspection,
    })
}

pub fn resolve_are_tile_v1(
    tileset: &TilesetIrV1,
    tile_id: u32,
) -> Result<TileResourceBindingV1, TileSetErrorV1> {
    let tile = tileset.tiles.get(tile_id as usize).ok_or_else(|| {
        error(
            "TILE-RESOLVER-TILE-ID-OOB",
            "ARE.Tile_ID",
            format!(
                "Tile_ID {tile_id} exceeds SET tile count {}",
                tileset.tiles.len()
            ),
        )
    })?;
    if tile.tile_id != tile_id {
        return Err(error(
            "TILE-RESOLVER-TILE-ID-MISMATCH",
            "ARE.Tile_ID",
            "SET tile sequence is not contiguous",
        ));
    }
    if tile
        .walkmesh_class_token
        .eq_ignore_ascii_case(&tile.model_resref)
    {
        // Equality is legal as text, but resolution never consumes it. Keep
        // the explicit fields separate so a future caller cannot substitute
        // SET.WalkMesh for the WOK resource key.
    }
    Ok(TileResourceBindingV1 {
        tile_id,
        model_resref: tile.model_resref.clone(),
        wok_resref: tile.model_resref.clone(),
        walkmesh_class_token: tile.walkmesh_class_token.clone(),
    })
}

fn parse_document(bytes: &[u8]) -> Result<SetDocumentV1, TileSetErrorV1> {
    if bytes.is_empty() || bytes.len() > MAX_SET_BYTES {
        return Err(error(
            "TILE-SET-SIZE",
            "set",
            format!("SET size must be in 1..={MAX_SET_BYTES} bytes"),
        ));
    }
    if bytes.contains(&0) || !bytes.is_ascii() {
        return Err(error(
            "TILE-SET-NON-ASCII",
            "set",
            "SET must contain non-NUL ASCII text",
        ));
    }
    let text = std::str::from_utf8(bytes).map_err(|source| {
        error(
            "TILE-SET-NON-ASCII",
            "set",
            format!("invalid ASCII/UTF-8: {source}"),
        )
    })?;
    let mut sections: Vec<SetSectionV1> = Vec::new();
    let mut entry_count = 0_usize;
    for (line_index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            if sections.len() >= MAX_SET_SECTIONS {
                return Err(error(
                    "TILE-SET-SECTION-LIMIT",
                    format!("set.lines[{line_index}]"),
                    "SET section count exceeds product guardrail",
                ));
            }
            let name = line[1..line.len() - 1].trim();
            if name.is_empty() || !name.is_ascii() {
                return Err(error(
                    "TILE-SET-SECTION-INVALID",
                    format!("set.lines[{line_index}]"),
                    "section name must be non-empty ASCII",
                ));
            }
            sections.push(SetSectionV1 {
                name: name.to_owned(),
                entries: Vec::new(),
            });
            continue;
        }
        let current = sections.last_mut().ok_or_else(|| {
            error(
                "TILE-SET-SECTION-MISSING",
                format!("set.lines[{line_index}]"),
                "key/value appears before the first section",
            )
        })?;
        let (key, value) = line.split_once('=').ok_or_else(|| {
            error(
                "TILE-SET-ENTRY-INVALID",
                format!("set.lines[{line_index}]"),
                "SET entry must use key=value",
            )
        })?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || !key.is_ascii() || !value.is_ascii() {
            return Err(error(
                "TILE-SET-ENTRY-INVALID",
                format!("set.lines[{line_index}]"),
                "SET key and value must be ASCII and key must be non-empty",
            ));
        }
        entry_count = entry_count
            .checked_add(1)
            .ok_or_else(|| error("TILE-SET-ENTRY-LIMIT", "set", "SET entry count overflow"))?;
        if entry_count > MAX_SET_ENTRIES {
            return Err(error(
                "TILE-SET-ENTRY-LIMIT",
                "set",
                "SET entry count exceeds product guardrail",
            ));
        }
        if current
            .entries
            .iter()
            .any(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        {
            return Err(error(
                "TILE-SET-DUPLICATE-KEY",
                format!("[{}].{key}", current.name),
                "duplicate SET key is ambiguous",
            ));
        }
        current.entries.push((key.to_owned(), value.to_owned()));
    }
    if sections.is_empty() {
        return Err(error(
            "TILE-SET-SECTION-MISSING",
            "set",
            "SET contains no sections",
        ));
    }
    Ok(SetDocumentV1 { sections })
}

fn section_map(document: &SetDocumentV1) -> Result<HashMap<String, &SetSectionV1>, TileSetErrorV1> {
    let mut map = HashMap::with_capacity(document.sections.len());
    for section in &document.sections {
        let key = section.name.to_ascii_uppercase();
        if map.insert(key, section).is_some() {
            return Err(error(
                "TILE-SET-DUPLICATE-SECTION",
                format!("[{}]", section.name),
                "duplicate SET section is ambiguous",
            ));
        }
    }
    Ok(map)
}

fn required_section<'a>(
    sections: &'a HashMap<String, &'a SetSectionV1>,
    name: &str,
) -> Result<&'a SetSectionV1, TileSetErrorV1> {
    sections.get(name).copied().ok_or_else(|| {
        error(
            "TILE-SET-SECTION-MISSING",
            format!("[{name}]"),
            "required SET section is missing",
        )
    })
}

fn parse_counted_sections(
    sections: &HashMap<String, &SetSectionV1>,
    count_section_name: &str,
    item_prefix: &str,
    mismatch_code: &str,
) -> Result<Vec<SetSectionV1>, TileSetErrorV1> {
    let count_section = required_section(sections, count_section_name)?;
    let count = parse_usize(
        require_value(count_section, "Count")?,
        &format!("[{count_section_name}].Count"),
    )?;
    if count > MAX_SET_SECTIONS {
        return Err(error(
            "TILE-SET-COUNT-LIMIT",
            format!("[{count_section_name}].Count"),
            "declared section count exceeds product guardrail",
        ));
    }
    let mut output = Vec::new();
    output.try_reserve_exact(count).map_err(|_| {
        error(
            "TILE-SET-ALLOCATION",
            format!("[{count_section_name}].Count"),
            "section vector allocation failed after count preflight",
        )
    })?;
    for index in 0..count {
        let name = format!("{item_prefix}{index}");
        output.push(
            required_section(sections, &name)
                .map_err(|_| {
                    error(
                        mismatch_code,
                        format!("[{count_section_name}].Count"),
                        format!("declared Count requires missing contiguous section [{name}]"),
                    )
                })?
                .clone(),
        );
    }
    if sections.keys().any(|name| {
        name.strip_prefix(item_prefix).is_some_and(|suffix| {
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
        }) && name
            .strip_prefix(item_prefix)
            .and_then(|suffix| suffix.parse::<usize>().ok())
            .is_some_and(|index| index >= count)
    }) {
        return Err(error(
            mismatch_code,
            format!("[{count_section_name}].Count"),
            "declared Count does not match contiguous item sections",
        ));
    }
    Ok(output)
}

fn parse_general(section: &SetSectionV1) -> Result<TilesetGeneralV1, TileSetErrorV1> {
    Ok(TilesetGeneralV1 {
        name: require_value(section, "Name")?.to_owned(),
        set_type: require_value(section, "Type")?.to_owned(),
        version: require_value(section, "Version")?.to_owned(),
        display_name: require_value(section, "DisplayName")?.to_owned(),
        unlocalized_name: require_value(section, "UnlocalizedName")?.to_owned(),
        interior: parse_bool(require_value(section, "Interior")?, "GENERAL.Interior")?,
        has_height_transition: parse_bool(
            require_value(section, "HasHeightTransition")?,
            "GENERAL.HasHeightTransition",
        )?,
        transition: parse_i32(require_value(section, "Transition")?, "GENERAL.Transition")?,
        env_map: require_value(section, "EnvMap")?.to_owned(),
        border: require_value(section, "Border")?.to_owned(),
        default_terrain: require_value(section, "Default")?.to_owned(),
        floor: require_value(section, "Floor")?.to_owned(),
    })
}

fn parse_tile(
    tile_id: u32,
    section: &SetSectionV1,
    document: &SetDocumentV1,
) -> Result<TileDescriptorV1, TileSetErrorV1> {
    let get = |name: &str| require_value(section, name);
    let optional = |name: &str| value(section, name).filter(|value| !value.is_empty());
    let door_count = optional("Doors")
        .map(|value| parse_usize(value, &format!("[{}].Doors", section.name)))
        .transpose()?
        .unwrap_or(0);
    let sound_count = optional("Sounds")
        .map(|value| parse_usize(value, &format!("[{}].Sounds", section.name)))
        .transpose()?
        .unwrap_or(0);
    let doors = associated_sections(document, tile_id, "DOOR", door_count)?;
    let sounds = associated_sections(document, tile_id, "SOUND", sound_count)?;
    Ok(TileDescriptorV1 {
        tile_id,
        model_resref: get("Model")?.to_owned(),
        walkmesh_class_token: get("WalkMesh")?.to_owned(),
        corner_terrain: [
            get("TopLeft")?.to_owned(),
            get("TopRight")?.to_owned(),
            get("BottomLeft")?.to_owned(),
            get("BottomRight")?.to_owned(),
        ],
        corner_heights: [
            parse_i32(get("TopLeftHeight")?, "TopLeftHeight")?,
            parse_i32(get("TopRightHeight")?, "TopRightHeight")?,
            parse_i32(get("BottomLeftHeight")?, "BottomLeftHeight")?,
            parse_i32(get("BottomRightHeight")?, "BottomRightHeight")?,
        ],
        edge_crossers: [
            optional("Top").map(str::to_owned),
            optional("Right").map(str::to_owned),
            optional("Bottom").map(str::to_owned),
            optional("Left").map(str::to_owned),
        ],
        path_node: get("PathNode")?.to_owned(),
        visibility_node: optional("VisibilityNode").map(str::to_owned),
        visibility_orientation: optional("VisibilityOrientation")
            .map(|value| parse_i32(value, "VisibilityOrientation"))
            .transpose()?
            .unwrap_or(0),
        door_visibility_node: optional("DoorVisibilityNode").map(str::to_owned),
        door_visibility_orientation: optional("DoorVisibilityOrientation")
            .map(|value| parse_i32(value, "DoorVisibilityOrientation"))
            .transpose()?
            .unwrap_or(0),
        orientation_quarter_turns: u8::try_from(parse_i32(get("Orientation")?, "Orientation")?)
            .map_err(|_| {
                error(
                    "TILE-SET-ORIENTATION",
                    format!("[{}].Orientation", section.name),
                    "orientation must be 0..3",
                )
            })?,
        image_map_2d: optional("ImageMap2D").map(str::to_owned),
        lights: TileLightFlagsV1 {
            main: [
                parse_bool(get("MainLight1")?, "MainLight1")?,
                parse_bool(get("MainLight2")?, "MainLight2")?,
            ],
            source: [
                parse_bool(get("SourceLight1")?, "SourceLight1")?,
                parse_bool(get("SourceLight2")?, "SourceLight2")?,
            ],
        },
        anim_loops: [
            parse_bool(get("AnimLoop1")?, "AnimLoop1")?,
            parse_bool(get("AnimLoop2")?, "AnimLoop2")?,
            parse_bool(get("AnimLoop3")?, "AnimLoop3")?,
        ],
        doors,
        sounds,
    })
}

fn associated_sections(
    document: &SetDocumentV1,
    tile_id: u32,
    kind: &str,
    count: usize,
) -> Result<Vec<SetSectionV1>, TileSetErrorV1> {
    let prefix = format!("TILE{tile_id}{kind}");
    let map = section_map(document)?;
    let mut output = Vec::with_capacity(count);
    for index in 0..count {
        output.push(required_section(&map, &format!("{prefix}{index}"))?.clone());
    }
    Ok(output)
}

fn parse_group(
    group_id: u32,
    section: &SetSectionV1,
    tile_count: usize,
) -> Result<TileGroupV1, TileSetErrorV1> {
    let rows = parse_usize(require_value(section, "Rows")?, "group.Rows")?;
    let columns = parse_usize(require_value(section, "Columns")?, "group.Columns")?;
    let count = rows.checked_mul(columns).ok_or_else(|| {
        error(
            "TILE-SET-GROUP-SIZE-OVERFLOW",
            format!("[{}]", section.name),
            "group Rows*Columns overflow",
        )
    })?;
    let mut tiles = Vec::with_capacity(count);
    for index in 0..count {
        let raw = require_value(section, &format!("Tile{index}"))?;
        let signed = parse_i32(raw, &format!("[{}].Tile{index}", section.name))?;
        if signed == -1 {
            tiles.push(None);
        } else {
            let tile = u32::try_from(signed).map_err(|_| {
                error(
                    "TILE-SET-GROUP-TILE-INVALID",
                    format!("[{}].Tile{index}", section.name),
                    "group tile must be -1 hole or a non-negative tile id",
                )
            })?;
            if tile as usize >= tile_count {
                return Err(error(
                    "TILE-SET-GROUP-TILE-OOB",
                    format!("[{}].Tile{index}", section.name),
                    "group references a missing tile",
                ));
            }
            tiles.push(Some(tile));
        }
    }
    Ok(TileGroupV1 {
        group_id,
        rows: rows as u32,
        columns: columns as u32,
        tiles,
    })
}

fn validate_typed_tileset(tileset: &TilesetIrV1) -> Result<(), TileSetErrorV1> {
    if tileset.schema_version != SET_SCHEMA_VERSION {
        return Err(error(
            "TILE-SET-SCHEMA",
            "tileset.schemaVersion",
            "TilesetIrV1 must use schema version 1",
        ));
    }
    if tileset.tiles.is_empty() {
        return Err(error(
            "TILE-SET-TILE-COUNT-MISMATCH",
            "tileset.tiles",
            "tileset must contain at least one tile",
        ));
    }
    let terrain_names = tileset
        .terrain_types
        .iter()
        .filter_map(|section| value(section, "Name"))
        .map(str::to_owned)
        .collect::<HashSet<_>>();
    for (index, tile) in tileset.tiles.iter().enumerate() {
        if tile.tile_id as usize != index {
            return Err(error(
                "TILE-SET-TILE-CONTIGUITY",
                format!("tileset.tiles[{index}].tileId"),
                "tile ids must be contiguous TILE0..TILE(count-1)",
            ));
        }
        validate_resref(
            &tile.model_resref,
            &format!("tileset.tiles[{index}].modelResref"),
        )?;
        if tile.orientation_quarter_turns > 3 {
            return Err(error(
                "TILE-SET-ORIENTATION",
                format!("tileset.tiles[{index}].orientationQuarterTurns"),
                "orientation must be in 0..3",
            ));
        }
        if tile.walkmesh_class_token.is_empty() || !tile.walkmesh_class_token.is_ascii() {
            return Err(error(
                "TILE-SET-WALKMESH-TOKEN",
                format!("tileset.tiles[{index}].walkmeshClassToken"),
                "WalkMesh must remain a non-empty ASCII domain token",
            ));
        }
        for (corner, terrain) in tile.corner_terrain.iter().enumerate() {
            if !terrain_names.contains(terrain) {
                return Err(error(
                    "TILE-SET-TERRAIN-REFERENCE",
                    format!("tileset.tiles[{index}].cornerTerrain[{corner}]"),
                    "tile corner terrain must reference a declared terrain type",
                ));
            }
        }
    }
    Ok(())
}

fn general_section(general: &TilesetGeneralV1) -> SetSectionV1 {
    SetSectionV1 {
        name: "GENERAL".to_owned(),
        entries: vec![
            ("Name".to_owned(), general.name.clone()),
            ("Type".to_owned(), general.set_type.clone()),
            ("Version".to_owned(), general.version.clone()),
            ("DisplayName".to_owned(), general.display_name.clone()),
            (
                "UnlocalizedName".to_owned(),
                general.unlocalized_name.clone(),
            ),
            (
                "Interior".to_owned(),
                bool_text(general.interior).to_owned(),
            ),
            (
                "HasHeightTransition".to_owned(),
                bool_text(general.has_height_transition).to_owned(),
            ),
            ("Transition".to_owned(), general.transition.to_string()),
            ("EnvMap".to_owned(), general.env_map.clone()),
            ("Border".to_owned(), general.border.clone()),
            ("Default".to_owned(), general.default_terrain.clone()),
            ("Floor".to_owned(), general.floor.clone()),
        ],
    }
}

fn tile_section(tile: &TileDescriptorV1) -> SetSectionV1 {
    let optional = |value: &Option<String>| value.clone().unwrap_or_default();
    SetSectionV1 {
        name: format!("TILE{}", tile.tile_id),
        entries: vec![
            ("Model".to_owned(), tile.model_resref.clone()),
            ("WalkMesh".to_owned(), tile.walkmesh_class_token.clone()),
            ("TopLeft".to_owned(), tile.corner_terrain[0].clone()),
            ("TopRight".to_owned(), tile.corner_terrain[1].clone()),
            ("BottomLeft".to_owned(), tile.corner_terrain[2].clone()),
            ("BottomRight".to_owned(), tile.corner_terrain[3].clone()),
            (
                "TopLeftHeight".to_owned(),
                tile.corner_heights[0].to_string(),
            ),
            (
                "TopRightHeight".to_owned(),
                tile.corner_heights[1].to_string(),
            ),
            (
                "BottomLeftHeight".to_owned(),
                tile.corner_heights[2].to_string(),
            ),
            (
                "BottomRightHeight".to_owned(),
                tile.corner_heights[3].to_string(),
            ),
            ("Top".to_owned(), optional(&tile.edge_crossers[0])),
            ("Right".to_owned(), optional(&tile.edge_crossers[1])),
            ("Bottom".to_owned(), optional(&tile.edge_crossers[2])),
            ("Left".to_owned(), optional(&tile.edge_crossers[3])),
            ("PathNode".to_owned(), tile.path_node.clone()),
            ("VisibilityNode".to_owned(), optional(&tile.visibility_node)),
            (
                "VisibilityOrientation".to_owned(),
                tile.visibility_orientation.to_string(),
            ),
            (
                "DoorVisibilityNode".to_owned(),
                optional(&tile.door_visibility_node),
            ),
            (
                "DoorVisibilityOrientation".to_owned(),
                tile.door_visibility_orientation.to_string(),
            ),
            (
                "Orientation".to_owned(),
                tile.orientation_quarter_turns.to_string(),
            ),
            ("ImageMap2D".to_owned(), optional(&tile.image_map_2d)),
            (
                "MainLight1".to_owned(),
                bool_text(tile.lights.main[0]).to_owned(),
            ),
            (
                "MainLight2".to_owned(),
                bool_text(tile.lights.main[1]).to_owned(),
            ),
            (
                "SourceLight1".to_owned(),
                bool_text(tile.lights.source[0]).to_owned(),
            ),
            (
                "SourceLight2".to_owned(),
                bool_text(tile.lights.source[1]).to_owned(),
            ),
            (
                "AnimLoop1".to_owned(),
                bool_text(tile.anim_loops[0]).to_owned(),
            ),
            (
                "AnimLoop2".to_owned(),
                bool_text(tile.anim_loops[1]).to_owned(),
            ),
            (
                "AnimLoop3".to_owned(),
                bool_text(tile.anim_loops[2]).to_owned(),
            ),
            ("Doors".to_owned(), tile.doors.len().to_string()),
            ("Sounds".to_owned(), tile.sounds.len().to_string()),
        ],
    }
}

fn count_section(name: &str, count: usize) -> SetSectionV1 {
    SetSectionV1 {
        name: name.to_owned(),
        entries: vec![("Count".to_owned(), count.to_string())],
    }
}

fn value<'a>(section: &'a SetSectionV1, key: &str) -> Option<&'a str> {
    section
        .entries
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.as_str())
}

fn require_value<'a>(section: &'a SetSectionV1, key: &str) -> Result<&'a str, TileSetErrorV1> {
    value(section, key).ok_or_else(|| {
        error(
            "TILE-SET-KEY-MISSING",
            format!("[{}].{key}", section.name),
            "required SET key is missing",
        )
    })
}

fn parse_bool(value: &str, path: &str) -> Result<bool, TileSetErrorV1> {
    match value {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(error(
            "TILE-SET-BOOLEAN",
            path,
            "boolean SET value must be 0 or 1",
        )),
    }
}

fn parse_i32(value: &str, path: &str) -> Result<i32, TileSetErrorV1> {
    value.parse::<i32>().map_err(|_| {
        error(
            "TILE-SET-INTEGER",
            path,
            "expected a signed decimal integer",
        )
    })
}

fn parse_usize(value: &str, path: &str) -> Result<usize, TileSetErrorV1> {
    value.parse::<usize>().map_err(|_| {
        error(
            "TILE-SET-COUNT",
            path,
            "expected a non-negative decimal count",
        )
    })
}

fn bool_text(value: bool) -> &'static str {
    if value { "1" } else { "0" }
}

fn validate_resref(value: &str, path: &str) -> Result<(), TileSetErrorV1> {
    if value.is_empty()
        || value.len() > 16
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(error(
            "TILE-RESREF-INVALID",
            path,
            "resref must contain 1..16 canonical lowercase ASCII letters, digits or underscore",
        ));
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

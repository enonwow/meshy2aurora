use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    erf::{ErfArchive, ErfFileType},
    gff::{
        GffDocumentV1, GffFileTypeV1, GffLimitsV1, GffStructV1, GffValueV1, GffWriterOptionsV1,
        read_gff_v32, write_gff_v32,
    },
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_erf_archive_v1, write_hak_v1},
    mdl::{NWN_EE_MAX_MESH_INDEX_COUNT_V1, NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1, inspect_binary_mdl},
    placeable::{
        ARE_RESOURCE_TYPE, FAC_RESOURCE_TYPE, GIC_RESOURCE_TYPE, GIT_RESOURCE_TYPE,
        IFO_RESOURCE_TYPE, ITP_RESOURCE_TYPE, MDL_RESOURCE_TYPE, PLACEABLES_2DA_RESOURCE_TYPE,
        PWK_RESOURCE_TYPE, PlaceablePlacementV1, StaticPlaceableIdentityV1, TGA_RESOURCE_TYPE,
        UTP_RESOURCE_TYPE, build_meshy_static_placeable_package_v1,
        build_meshy_static_placeable_package_v2, inspect_meshy_static_placeable_authoring_v1,
    },
    placeable_authoring::{
        PLACEABLE_AUTHORING_SCHEMA_VERSION_V1, PlaceableAuthoringDocumentV1,
        PlaceableAuthoringElementV1, PlaceableElementFlagsV1, PlaceableElementKindV1,
        PlaceableElementSourceV1, PlaceableElementTransformV1,
    },
    placeable_collision::inspect_ascii_placeable_walkmesh_v1,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const V1_CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\tlc-meshy-p20k-placeables-v1-20260725";
const SHADOW_V2_CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\tlc-meshy-p20k-placeables-shadow-v2-20260726";
const LOOT_WORKSTATIONS_V1_CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\tlc-meshy-loot-workstations-v1-20260726";
const LOOT_WORKSTATIONS_V2_CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\tlc-meshy-loot-workstations-v2-20260726";
const LOOT_WORKSTATIONS_V3_CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\tlc-meshy-loot-workstations-v3-20260726";
const LOOT_WORKSTATIONS_V4_CANONICAL_OUTPUT: &str =
    r"C:\Projects\meshy2aurora\proof-output\tlc-meshy-loot-workstations-v4-20260726";
const BASE_PLACEABLES_SHA256: &str =
    "b772eafec5e6b380ad41e163e2a52585f2ddcec1c5bd7acea230b7e1a618df90";
const TRIANGLE_COUNT: usize = 20_000;
const INDEX_COUNT: usize = TRIANGLE_COUNT * 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateKind {
    V1,
    ShadowV2,
    LootWorkstationsV1,
    LootWorkstationsV2,
    LootWorkstationsV3,
    LootWorkstationsV4,
}

#[derive(Clone, Copy, Debug)]
struct CandidateProfile {
    kind: CandidateKind,
    canonical_output: &'static str,
    lane: &'static str,
    module_resref: &'static str,
    module_file_name: &'static str,
    module_display_name: &'static str,
    area_resref: &'static str,
    area_name: &'static str,
    hak_resref: &'static str,
    hak_file_name: &'static str,
    change_under_test: &'static str,
}

const V1_PROFILE: CandidateProfile = CandidateProfile {
    kind: CandidateKind::V1,
    canonical_output: V1_CANONICAL_OUTPUT,
    lane: "TLC_MESHY_P20K_PLACEABLE_TRIO_V1",
    module_resref: "m2a_tlcm3_mod",
    module_file_name: "m2a_tlcm3_mod.mod",
    module_display_name: "Meshy2Aurora TLC Meshy P20K Trio",
    area_resref: "m2a_tlcm3_ar",
    area_name: "Meshy2Aurora M0 binary vertical-slice area",
    hak_resref: "m2a_tlcm3_hak",
    hak_file_name: "m2a_tlcm3_hak.hak",
    change_under_test: "original TLC Meshy P20K placeable trio",
};

const SHADOW_V2_PROFILE: CandidateProfile = CandidateProfile {
    kind: CandidateKind::ShadowV2,
    canonical_output: SHADOW_V2_CANONICAL_OUTPUT,
    lane: "TLC_MESHY_P20K_PLACEABLE_SHADOW_ADJACENCY_V2",
    module_resref: "m2a_tlcs2_mod",
    module_file_name: "m2a_tlcs2_mod.mod",
    module_display_name: "Meshy2Aurora TLC Shadow Adjacency V2",
    area_resref: "m2a_tlcs2_ar",
    area_name: "Meshy2Aurora TLC Shadow Adjacency Test",
    hak_resref: "m2a_tlcs2_hak",
    hak_file_name: "m2a_tlcs2_hak.hak",
    change_under_test: "position-welded face adjacency across UV and hard-normal render seams",
};

const LOOT_WORKSTATIONS_V1_PROFILE: CandidateProfile = CandidateProfile {
    kind: CandidateKind::LootWorkstationsV1,
    canonical_output: LOOT_WORKSTATIONS_V1_CANONICAL_OUTPUT,
    lane: "TLC_MESHY_LOOT_WORKSTATIONS_V1",
    module_resref: "m2a_tlcw1_mod",
    module_file_name: "m2a_tlcw1_mod.mod",
    module_display_name: "Meshy2Aurora TLC Loot Workstations V1",
    area_resref: "m2a_tlcw1_ar",
    area_name: "Meshy2Aurora TLC Loot Workstations",
    hak_resref: "m2a_tlcw1_hak",
    hak_file_name: "m2a_tlcw1_hak.hak",
    change_under_test: "three new Meshy multi-image loot-system workstations through the shared placeable pipeline",
};

const LOOT_WORKSTATIONS_V2_PROFILE: CandidateProfile = CandidateProfile {
    kind: CandidateKind::LootWorkstationsV2,
    canonical_output: LOOT_WORKSTATIONS_V2_CANONICAL_OUTPUT,
    lane: "TLC_MESHY_LOOT_WORKSTATIONS_V2_REACTOR_ALIGNMENT",
    module_resref: "m2a_tlcw2_mod",
    module_file_name: "m2a_tlcw2_mod.mod",
    module_display_name: "Meshy2Aurora TLC Loot Workstations V2",
    area_resref: "m2a_tlcw2_ar",
    area_name: "Meshy2Aurora TLC Loot Workstations V2",
    hak_resref: "m2a_tlcw2_hak",
    hak_file_name: "m2a_tlcw2_hak.hak",
    change_under_test: "authoring-only reactor correction: centered raised lower basin plus inward anchored lava stream",
};

const LOOT_WORKSTATIONS_V3_PROFILE: CandidateProfile = CandidateProfile {
    kind: CandidateKind::LootWorkstationsV3,
    canonical_output: LOOT_WORKSTATIONS_V3_CANONICAL_OUTPUT,
    lane: "TLC_MESHY_LOOT_WORKSTATIONS_V3_REACTOR_BASE",
    module_resref: "m2a_tlcw3_mod",
    module_file_name: "m2a_tlcw3_mod.mod",
    module_display_name: "Meshy2Aurora TLC Loot Workstations V3",
    area_resref: "m2a_tlcw3_ar",
    area_name: "Meshy2Aurora TLC Loot Workstations V3",
    hak_resref: "m2a_tlcw3_hak",
    hak_file_name: "m2a_tlcw3_hak.hak",
    change_under_test: "reactor base-only correction: original vertical lava, narrowed raised catch basin between the legs, and two preserved side-chain assemblies",
};

const LOOT_WORKSTATIONS_V4_PROFILE: CandidateProfile = CandidateProfile {
    kind: CandidateKind::LootWorkstationsV4,
    canonical_output: LOOT_WORKSTATIONS_V4_CANONICAL_OUTPUT,
    lane: "TLC_MESHY_LOOT_WORKSTATIONS_V4_LAVA_CATCH",
    module_resref: "m2a_tlcw4_mod",
    module_file_name: "m2a_tlcw4_mod.mod",
    module_display_name: "Meshy2Aurora TLC Loot Workstations V4",
    area_resref: "m2a_tlcw4_ar",
    area_name: "Meshy2Aurora TLC Loot Workstations V4",
    hak_resref: "m2a_tlcw4_hak",
    hak_file_name: "m2a_tlcw4_hak.hak",
    change_under_test: "reactor base-only correction: unchanged vertical lava lands with a safe interior margin in the widened lower catch basin",
};

const REACTOR_V2_BASIN_COMPONENTS: [u32; 16] = [
    31, 33, 34, 35, 45, 66, 74, 76, 84, 92, 144, 155, 170, 600, 602, 603,
];
const REACTOR_V3_BASE_COMPONENTS: [u32; 17] = [
    31, 33, 34, 35, 37, 45, 66, 74, 76, 84, 92, 144, 155, 170, 600, 602, 603,
];
const REACTOR_V4_BASE_COMPONENTS: [u32; 16] = [
    31, 33, 34, 35, 45, 66, 74, 76, 84, 92, 144, 155, 170, 600, 602, 603,
];
const REACTOR_LAVA_STREAM_COMPONENT: u32 = 766;
const REACTOR_BASIN_RAISE_METERS: f32 = 0.05;
const REACTOR_STREAM_ROTATION_X_DEGREES: f32 = 20.0;
const REACTOR_STREAM_SCALE_Y: f32 = 1.12;
const REACTOR_STREAM_PIVOT: [f32; 3] = [0.010_017_341, 0.326_376_91, 0.226_872_39];
const REACTOR_V3_BASE_TRANSLATION: [f32; 3] = [0.0, 0.05, 0.08];
const REACTOR_V3_BASE_SCALE: [f32; 3] = [0.54, 1.0, 1.05];
const REACTOR_V3_ROOT_SCALE: f32 = 2.5;
const REACTOR_V4_BASE_TRANSLATION: [f32; 3] = [0.0, 0.05, 0.14];
const REACTOR_V4_BASE_SCALE: [f32; 3] = [0.54, 1.0, 1.50];
const REACTOR_V4_MAX_STREAM_RADIAL_FRACTION: f32 = 0.60;
const REACTOR_V3_CHAIN_MIN_ABS_X: f32 = 0.190;
const REACTOR_V3_CHAIN_MAX_ABS_X: f32 = 0.290;
const REACTOR_V3_CHAIN_MIN_Y: f32 = 0.115;
const REACTOR_V3_CHAIN_MAX_Y: f32 = 0.470;
const REACTOR_V3_CHAIN_MAX_ABS_Z: f32 = 0.075;

#[derive(Clone, Copy, Debug)]
struct AssetSpec {
    style_name: &'static str,
    source_path: &'static str,
    source_sha256: &'static str,
    source_triangles: usize,
    meshy_task_kind: &'static str,
    meshy_task_id: &'static str,
    preview_task_id: &'static str,
    refine_task_id: &'static str,
    prompt: &'static str,
    model_resref: &'static str,
    texture_resref: &'static str,
    blueprint_resref: &'static str,
    object_tag: &'static str,
    display_name: &'static str,
    placement: PlaceablePlacementV1,
}

struct PreparedAsset {
    spec: AssetSpec,
    appearance_row: u32,
    source: Vec<u8>,
    mdl: Vec<u8>,
    pwk: Vec<u8>,
    texture: Vec<u8>,
    utp: Vec<u8>,
    git: Vec<u8>,
    gic: Vec<u8>,
    itp: Vec<u8>,
    pipeline_report: Vec<u8>,
    authoring_document: Option<Vec<u8>>,
    authoring_sha256: Option<String>,
    vertex_count: usize,
    adjacency_boundary_edge_count: usize,
    adjacency_linked_edge_count: usize,
}

fn main() -> ExitCode {
    match run() {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let command = parse(env::args().skip(1))?;
    let candidate = candidate_for_output(&command.output)?;
    if command.output.exists() {
        return Err(format!(
            "TLC-MESHY-P20K-OUTPUT-EXISTS: {}",
            command.output.display()
        ));
    }
    let base_placeables = read(&command.placeables_two_da, "BASE-PLACEABLES-2DA")?;
    require_hash(
        &base_placeables,
        BASE_PLACEABLES_SHA256,
        "BASE-PLACEABLES-2DA",
    )?;

    let specs = asset_specs(candidate);
    let mut current_two_da = base_placeables.clone();
    let mut prepared = Vec::with_capacity(specs.len());
    let mut shared_ifo = None;
    let mut shared_are = None;
    let mut shared_fac = None;

    for (index, spec) in specs.iter().copied().enumerate() {
        let source = read(Path::new(spec.source_path), "MESHY-SOURCE")?;
        require_hash(&source, spec.source_sha256, "MESHY-SOURCE")?;
        let identity = StaticPlaceableIdentityV1 {
            module_resref: candidate.module_resref.to_owned(),
            module_file_name: candidate.module_file_name.to_owned(),
            module_display_name: candidate.module_display_name.to_owned(),
            area_resref: candidate.area_resref.to_owned(),
            area_name: candidate.area_name.to_owned(),
            hak_resref: candidate.hak_resref.to_owned(),
            hak_file_name: candidate.hak_file_name.to_owned(),
            model_resref: spec.model_resref.to_owned(),
            texture_resref: spec.texture_resref.to_owned(),
            blueprint_resref: spec.blueprint_resref.to_owned(),
            object_tag: spec.object_tag.to_owned(),
            display_name: spec.display_name.to_owned(),
        };
        let authoring = match (candidate.kind, spec.style_name) {
            (CandidateKind::LootWorkstationsV2, "Upgrading Reactor") => {
                Some(upgrading_reactor_v2_authoring_document(&source)?)
            }
            (CandidateKind::LootWorkstationsV3, "Upgrading Reactor") => {
                Some(upgrading_reactor_v3_authoring_document(&source)?)
            }
            (CandidateKind::LootWorkstationsV4, "Upgrading Reactor") => {
                Some(upgrading_reactor_v4_authoring_document(&source)?)
            }
            _ => None,
        };
        let artifact = if let Some(document) = authoring.as_ref() {
            build_meshy_static_placeable_package_v2(
                &source,
                &current_two_da,
                &identity,
                spec.placement,
                7,
                document,
            )
        } else {
            build_meshy_static_placeable_package_v1(
                &source,
                &current_two_da,
                &identity,
                spec.placement,
                7,
            )
        }
        .map_err(|error| serde_json::to_string(&error).unwrap_or_else(|_| error.to_string()))?;
        let authoring_document = authoring
            .as_ref()
            .map(|document| pretty(document, "AUTHORING-DOCUMENT"))
            .transpose()?;
        let authoring_sha256 = artifact
            .report
            .authoring
            .as_ref()
            .map(|report| report.authoring_sha256.clone());
        let expected_row = 16_500 + index as u32;
        if artifact.report.appearance_row.value != expected_row {
            return Err(format!(
                "TLC-MESHY-P20K-APPEARANCE-ROW: {} expected {}, got {}",
                spec.model_resref, expected_row, artifact.report.appearance_row.value
            ));
        }

        let hak = ErfArchive::parse(&artifact.hak_payload)
            .map_err(|error| format!("TLC-MESHY-P20K-SINGLE-HAK-READBACK: {error}"))?;
        let module = ErfArchive::parse(&artifact.module_payload)
            .map_err(|error| format!("TLC-MESHY-P20K-SINGLE-MOD-READBACK: {error}"))?;
        let mdl = find(&hak, spec.model_resref, MDL_RESOURCE_TYPE)?.to_vec();
        let mdl_inspection = inspect_binary_mdl(&mdl)
            .map_err(|error| format!("TLC-MESHY-P20K-MDL-READBACK: {error}"))?;
        let mesh = mdl_inspection
            .node_tree
            .roots
            .first()
            .and_then(|root| root.children.first())
            .and_then(|node| node.mesh.as_ref())
            .ok_or_else(|| format!("TLC-MESHY-P20K-MDL-MESH-MISSING: {}", spec.model_resref))?;
        if mesh.faces.len() != TRIANGLE_COUNT
            || mesh.index_counts.as_slice() != [INDEX_COUNT as u32]
            || mesh.raw_indices.len() != 1
            || mesh.raw_indices[0].len() != INDEX_COUNT
        {
            return Err(format!(
                "TLC-MESHY-P20K-MDL-GEOMETRY-DIFF: {} vertices={} faces={} indices={:?}",
                spec.model_resref,
                mesh.vertex_count,
                mesh.faces.len(),
                mesh.index_counts
            ));
        }
        let next_two_da = find(&hak, "placeables", PLACEABLES_2DA_RESOURCE_TYPE)?.to_vec();
        let pwk = find(&hak, spec.model_resref, PWK_RESOURCE_TYPE)?.to_vec();
        let texture = find(&hak, spec.texture_resref, TGA_RESOURCE_TYPE)?.to_vec();
        let utp = find(&module, spec.blueprint_resref, UTP_RESOURCE_TYPE)?.to_vec();
        let git = find(&module, candidate.area_resref, GIT_RESOURCE_TYPE)?.to_vec();
        let gic = find(&module, candidate.area_resref, GIC_RESOURCE_TYPE)?.to_vec();
        let itp = find(&module, "placeablepalcus", ITP_RESOURCE_TYPE)?.to_vec();
        if index == 0 {
            shared_ifo = Some(find(&module, "module", IFO_RESOURCE_TYPE)?.to_vec());
            shared_are = Some(find(&module, candidate.area_resref, ARE_RESOURCE_TYPE)?.to_vec());
            shared_fac = Some(find(&module, "repute", FAC_RESOURCE_TYPE)?.to_vec());
        }
        let pipeline_report = pretty(&artifact.report, "SINGLE-PIPELINE-REPORT")?;
        current_two_da = next_two_da;
        prepared.push(PreparedAsset {
            spec,
            appearance_row: artifact.report.appearance_row.value,
            source,
            mdl,
            pwk,
            texture,
            utp,
            git,
            gic,
            itp,
            pipeline_report,
            authoring_document,
            authoring_sha256,
            vertex_count: mesh.vertex_count,
            adjacency_boundary_edge_count: mesh
                .faces
                .iter()
                .flat_map(|face| face.adjacent_faces)
                .filter(|adjacent| *adjacent < 0)
                .count(),
            adjacency_linked_edge_count: mesh
                .faces
                .iter()
                .flat_map(|face| face.adjacent_faces)
                .filter(|adjacent| *adjacent >= 0)
                .count(),
        });
    }

    let git = merge_instance_lists(
        &prepared
            .iter()
            .map(|asset| asset.git.as_slice())
            .collect::<Vec<_>>(),
        GffFileTypeV1::Git,
        "Placeable List",
    )?;
    let gic = merge_instance_lists(
        &prepared
            .iter()
            .map(|asset| asset.gic.as_slice())
            .collect::<Vec<_>>(),
        GffFileTypeV1::Gic,
        "Placeable List",
    )?;
    let itp = merge_palette_entries(
        &prepared
            .iter()
            .map(|asset| asset.itp.as_slice())
            .collect::<Vec<_>>(),
        7,
    )?;

    let mut hak_resources = vec![resource(
        "placeables",
        PLACEABLES_2DA_RESOURCE_TYPE,
        current_two_da.clone(),
    )];
    for asset in &prepared {
        hak_resources.extend([
            resource(
                asset.spec.model_resref,
                MDL_RESOURCE_TYPE,
                asset.mdl.clone(),
            ),
            resource(
                asset.spec.model_resref,
                PWK_RESOURCE_TYPE,
                asset.pwk.clone(),
            ),
            resource(
                asset.spec.texture_resref,
                TGA_RESOURCE_TYPE,
                asset.texture.clone(),
            ),
        ]);
    }
    let hak = write_hak_v1(&hak_resources, &HakWriterOptionsV1::default())
        .map_err(|error| format!("TLC-MESHY-P20K-COMBINED-HAK-WRITE: {error}"))?;
    let mut module_resources = vec![
        resource(
            "module",
            IFO_RESOURCE_TYPE,
            shared_ifo.ok_or_else(|| "TLC-MESHY-P20K-IFO-MISSING".to_owned())?,
        ),
        resource(
            "repute",
            FAC_RESOURCE_TYPE,
            shared_fac.ok_or_else(|| "TLC-MESHY-P20K-FAC-MISSING".to_owned())?,
        ),
        resource(
            candidate.area_resref,
            ARE_RESOURCE_TYPE,
            shared_are.ok_or_else(|| "TLC-MESHY-P20K-ARE-MISSING".to_owned())?,
        ),
        resource(candidate.area_resref, GIT_RESOURCE_TYPE, git.clone()),
        resource(candidate.area_resref, GIC_RESOURCE_TYPE, gic.clone()),
        resource("placeablepalcus", ITP_RESOURCE_TYPE, itp.clone()),
    ];
    module_resources.extend(prepared.iter().map(|asset| {
        resource(
            asset.spec.blueprint_resref,
            UTP_RESOURCE_TYPE,
            asset.utp.clone(),
        )
    }));
    let module = write_erf_archive_v1(
        ErfFileType::Module,
        &module_resources,
        &HakWriterOptionsV1::default(),
    )
    .map_err(|error| format!("TLC-MESHY-P20K-COMBINED-MOD-WRITE: {error}"))?;
    validate_combined_package(
        &hak.payload,
        &module.payload,
        &current_two_da,
        candidate.area_resref,
        &prepared,
    )?;

    let generated = command.output.join("generated");
    fs::create_dir_all(&generated)
        .map_err(|error| format!("TLC-MESHY-P20K-OUTPUT-CREATE: {error}"))?;
    let mut outputs = vec![
        (
            generated.join(candidate.module_file_name),
            module.payload.clone(),
        ),
        (generated.join(candidate.hak_file_name), hak.payload.clone()),
        (
            generated.join("base-placeables.2da"),
            base_placeables.clone(),
        ),
        (generated.join("placeables.2da"), current_two_da.clone()),
        (
            generated.join(format!("{}.git", candidate.area_resref)),
            git.clone(),
        ),
        (
            generated.join(format!("{}.gic", candidate.area_resref)),
            gic.clone(),
        ),
        (generated.join("placeablepalcus.itp"), itp.clone()),
    ];
    for asset in &prepared {
        outputs.extend([
            (
                generated.join(format!("{}-source.glb", asset.spec.model_resref)),
                asset.source.clone(),
            ),
            (
                generated.join(format!("{}.mdl", asset.spec.model_resref)),
                asset.mdl.clone(),
            ),
            (
                generated.join(format!("{}.pwk", asset.spec.model_resref)),
                asset.pwk.clone(),
            ),
            (
                generated.join(format!("{}.tga", asset.spec.texture_resref)),
                asset.texture.clone(),
            ),
            (
                generated.join(format!("{}.utp", asset.spec.blueprint_resref)),
                asset.utp.clone(),
            ),
            (
                generated.join(format!("{}-pipeline-report.json", asset.spec.model_resref)),
                asset.pipeline_report.clone(),
            ),
        ]);
        if let Some(document) = &asset.authoring_document {
            outputs.push((
                generated.join(format!("{}-authoring.json", asset.spec.model_resref)),
                document.clone(),
            ));
        }
    }
    for (path, bytes) in &outputs {
        write_new(path, bytes)?;
    }
    for (path, expected) in &outputs {
        if read(path, "OUTPUT-READBACK")? != *expected {
            return Err(format!(
                "TLC-MESHY-P20K-OUTPUT-MISMATCH: {}",
                path.display()
            ));
        }
    }

    let provenance = json!({
        "schemaVersion": 1,
        "provider": "Meshy",
        "profileId": "S1-static-prop/v1",
        "aiModel": "meshy-6",
        "requestedSourcePolycount": 21_500,
        "finalTriangleTarget": TRIANGLE_COUNT,
        "styleDirection": if matches!(
            candidate.kind,
            CandidateKind::LootWorkstationsV1
                | CandidateKind::LootWorkstationsV2
                | CandidateKind::LootWorkstationsV3
                | CandidateKind::LootWorkstationsV4
        ) {
            "industrial gothic dark-fantasy loot-system workstations; soot-stained black iron, oxidized brass, arcane energy, readable NWN-scale silhouettes"
        } else {
            "industrial gothic dark-fantasy city; soot-stained stone, black iron, oxidized brass and arcane civic seals"
        },
        "referencePolicy": "high-level direction only; no The Last City model, texture, UV or payload was copied",
        "assets": prepared.iter().map(asset_manifest).collect::<Vec<_>>(),
        "generation": {
            "taskKind": if matches!(
                candidate.kind,
                CandidateKind::LootWorkstationsV1
                    | CandidateKind::LootWorkstationsV2
                    | CandidateKind::LootWorkstationsV3
                    | CandidateKind::LootWorkstationsV4
            ) {
                "multi-image-to-3d"
            } else {
                "text-to-3d-preview-and-refine"
            },
            "candidateCreditsConsumed": 90
        }
    });
    let provenance_json = pretty(&provenance, "PROVENANCE")?;
    write_new(&generated.join("meshy-provenance.json"), &provenance_json)?;

    let handoff = json!({
        "schemaVersion": 1,
        "status": "built_pending_native_installation",
        "lane": candidate.lane,
        "changeUnderTest": candidate.change_under_test,
        "testModuleFileName": candidate.module_file_name,
        "toolsetModuleName": candidate.module_display_name,
        "areaName": candidate.area_name,
        "areaResref": candidate.area_resref,
        "orderedHakFiles": [candidate.hak_file_name],
        "orderedHakResrefs": [candidate.hak_resref],
        "placeables": prepared.iter().map(asset_manifest).collect::<Vec<_>>(),
        "geometryContract": {
            "placeableCount": prepared.len(),
            "meshCountPerPlaceable": 1,
            "triangleCountPerPlaceable": TRIANGLE_COUNT,
            "indexCountPerPlaceable": INDEX_COUNT,
            "perMeshTriangleLimit": NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1,
            "perMeshIndexLimit": NWN_EE_MAX_MESH_INDEX_COUNT_V1,
            "triangleHeadroomPerPlaceable": NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1 - TRIANGLE_COUNT
        },
        "collisionContract": {
            "format": "nwn1-ascii-pwk",
            "resourceType": PWK_RESOURCE_TYPE,
            "sameResrefAsMdl": true,
            "surfaceId": 7,
            "shape": "per-model conservative world-XY rectangle",
            "offlineReadback": "passed"
        },
        "outputs": {
            "moduleSha256": sha256(&module.payload),
            "hakSha256": sha256(&hak.payload),
            "placeables2daSha256": sha256(&current_two_da),
            "gitSha256": sha256(&git),
            "gicSha256": sha256(&gic),
            "itpSha256": sha256(&itp),
            "provenanceSha256": sha256(&provenance_json)
        },
        "componentStatuses": {
            "meshySourceIdentity": "passed",
            "exactTwentyThousandTriangleInputs": "passed",
            "sharedModelPipeline": "passed",
            "binaryMdlReadback": "passed",
            "shadowAdjacencyReadback": "passed",
            "asciiPwkReadback": "passed",
            "placeables2da": "passed",
            "utp": "passed",
            "gitGicThreeInstances": "passed",
            "customPaletteThreeEntries": "passed",
            "hakModPackaging": "passed",
            "nativeInstallation": "pending",
            "proof": "not_tested"
        },
        "nativeInstallation": {
            "status": "pending_post_materialization_hash_verified_install",
            "canonicalModule": generated.join(candidate.module_file_name),
            "canonicalHak": generated.join(candidate.hak_file_name),
            "destinationModule": PathBuf::from(r"C:\Users\enonw\Documents\Neverwinter Nights\modules").join(candidate.module_file_name),
            "destinationHak": PathBuf::from(r"C:\Users\enonw\Documents\Neverwinter Nights\hak").join(candidate.hak_file_name)
        },
        "modelVisibility": "not_tested",
        "proofCompleteness": "missing",
        "collisionRuntimeVerdict": "not_tested",
        "ownerProofRequired": true,
        "ownerTest": [
            "Open the exact module and Area named above.",
            "Confirm all three named Meshy objects are visible at their declared placements.",
            "Enable shadows and inspect each projected shadow for dense striped false silhouettes or disconnected shadow islands.",
            "Confirm the dismantler, purifier and upgrading reactor match their declared roles and readable silhouettes.",
            "Run Test Module without rebuilding or replacing the exact HAK.",
            "Walk into each object from several directions; each PWK must block the player."
        ],
        "startsToolset": false,
        "startsNwn": false,
        "materializationCount": 1
    });
    let handoff_json = pretty(&handoff, "HANDOFF")?;
    let handoff_path = command.output.join("ready-for-owner-proof.json");
    write_new(&handoff_path, &handoff_json)?;
    if read(&handoff_path, "HANDOFF-READBACK")? != handoff_json {
        return Err("TLC-MESHY-P20K-HANDOFF-READBACK".to_owned());
    }
    serde_json::to_string_pretty(&handoff)
        .map_err(|error| format!("TLC-MESHY-P20K-SUMMARY: {error}"))
}

fn upgrading_reactor_v2_authoring_document(
    source: &[u8],
) -> Result<PlaceableAuthoringDocumentV1, String> {
    let bootstrap = inspect_meshy_static_placeable_authoring_v1(source)
        .map_err(|error| serde_json::to_string(&error).unwrap_or_else(|_| error.to_string()))?;
    if bootstrap.inspection.render_node_count != 1
        || bootstrap.inspection.primitive_count != 1
        || bootstrap.inspection.connected_component_count != 810
    {
        return Err(format!(
            "TLC-W2-REACTOR-INVENTORY: expected 1 node, 1 primitive and 810 components; got {}, {}, {}",
            bootstrap.inspection.render_node_count,
            bootstrap.inspection.primitive_count,
            bootstrap.inspection.connected_component_count
        ));
    }
    let node = &bootstrap.inspection.nodes[0];
    let primitive = &node.primitives[0];
    let template = bootstrap
        .document
        .elements
        .first()
        .ok_or_else(|| "TLC-W2-REACTOR-AUTHORING-TEMPLATE-MISSING".to_owned())?;
    let basin_group_id = "group:reactor-lower-basin".to_owned();
    let radians = REACTOR_STREAM_ROTATION_X_DEGREES.to_radians();
    let stream_rotation = [(radians * 0.5).sin(), 0.0, 0.0, (radians * 0.5).cos()];
    let mut elements = Vec::with_capacity(primitive.components.len() + 1);

    for component in &primitive.components {
        let is_basin = REACTOR_V2_BASIN_COMPONENTS.contains(&component.component_index);
        let is_stream = component.component_index == REACTOR_LAVA_STREAM_COMPONENT;
        elements.push(PlaceableAuthoringElementV1 {
            id: component.element_id.clone(),
            name: if is_basin {
                format!("Lower basin C{}", component.component_index)
            } else if is_stream {
                "Anchored lava stream".to_owned()
            } else {
                format!("{} · P0 C{}", node.name, component.component_index)
            },
            kind: PlaceableElementKindV1::SourceComponent,
            source: Some(PlaceableElementSourceV1 {
                node_id: node.node_id,
                primitive_id: Some(primitive.primitive_id),
                component_index: Some(component.component_index),
            }),
            parent_id: is_basin.then(|| basin_group_id.clone()),
            transform: if is_stream {
                PlaceableElementTransformV1 {
                    translation: [0.0, 0.0, 0.0],
                    rotation_xyzw: stream_rotation,
                    scale: [1.0, REACTOR_STREAM_SCALE_Y, 1.0],
                    pivot: REACTOR_STREAM_PIVOT,
                }
            } else {
                template.transform.clone()
            },
            flags: template.flags,
            deleted: template.deleted,
        });
    }
    elements.push(PlaceableAuthoringElementV1 {
        id: basin_group_id,
        name: "Reactor lower basin".to_owned(),
        kind: PlaceableElementKindV1::Group,
        source: None,
        parent_id: None,
        transform: PlaceableElementTransformV1 {
            translation: [0.0, REACTOR_BASIN_RAISE_METERS, 0.0],
            ..PlaceableElementTransformV1::default()
        },
        flags: PlaceableElementFlagsV1::default(),
        deleted: false,
    });
    Ok(PlaceableAuthoringDocumentV1 {
        schema_version: PLACEABLE_AUTHORING_SCHEMA_VERSION_V1,
        source_sha256: bootstrap.document.source_sha256,
        elements,
    })
}

fn upgrading_reactor_v3_authoring_document(
    source: &[u8],
) -> Result<PlaceableAuthoringDocumentV1, String> {
    upgrading_reactor_owner_layout_authoring_document(
        source,
        &REACTOR_V3_BASE_COMPONENTS,
        REACTOR_V3_BASE_TRANSLATION,
        REACTOR_V3_BASE_SCALE,
        "TLC-W3",
    )
}

fn upgrading_reactor_v4_authoring_document(
    source: &[u8],
) -> Result<PlaceableAuthoringDocumentV1, String> {
    upgrading_reactor_owner_layout_authoring_document(
        source,
        &REACTOR_V4_BASE_COMPONENTS,
        REACTOR_V4_BASE_TRANSLATION,
        REACTOR_V4_BASE_SCALE,
        "TLC-W4",
    )
}

fn upgrading_reactor_owner_layout_authoring_document(
    source: &[u8],
    base_components: &[u32],
    base_translation: [f32; 3],
    base_scale: [f32; 3],
    diagnostic_prefix: &str,
) -> Result<PlaceableAuthoringDocumentV1, String> {
    let bootstrap = inspect_meshy_static_placeable_authoring_v1(source)
        .map_err(|error| serde_json::to_string(&error).unwrap_or_else(|_| error.to_string()))?;
    if bootstrap.inspection.render_node_count != 1
        || bootstrap.inspection.primitive_count != 1
        || bootstrap.inspection.connected_component_count != 810
    {
        return Err(format!(
            "{diagnostic_prefix}-REACTOR-INVENTORY: expected 1 node, 1 primitive and 810 components; got {}, {}, {}",
            bootstrap.inspection.render_node_count,
            bootstrap.inspection.primitive_count,
            bootstrap.inspection.connected_component_count
        ));
    }

    let node = &bootstrap.inspection.nodes[0];
    let primitive = &node.primitives[0];
    let template = bootstrap
        .document
        .elements
        .first()
        .ok_or_else(|| "TLC-W3-REACTOR-AUTHORING-TEMPLATE-MISSING".to_owned())?;
    let base_group_id = "group:reactor-lower-base".to_owned();
    let right_chain_group_id = "group:reactor-right-chain".to_owned();
    let left_chain_group_id = "group:reactor-left-chain".to_owned();
    let root_group_id = "group:reactor-physical-scale".to_owned();
    let mut elements = Vec::with_capacity(primitive.components.len() + 4);

    for component in &primitive.components {
        let is_base = base_components.contains(&component.component_index);
        let chain_side = reactor_v3_chain_side(component.bounds_min, component.bounds_max);
        let is_stream = component.component_index == REACTOR_LAVA_STREAM_COMPONENT;
        elements.push(PlaceableAuthoringElementV1 {
            id: component.element_id.clone(),
            name: if is_base {
                format!("Lower catch base C{}", component.component_index)
            } else if is_stream {
                "Original vertical lava stream".to_owned()
            } else if let Some(side) = chain_side {
                format!("{side} side chain/anchor C{}", component.component_index)
            } else {
                format!("{} · P0 C{}", node.name, component.component_index)
            },
            kind: PlaceableElementKindV1::SourceComponent,
            source: Some(PlaceableElementSourceV1 {
                node_id: node.node_id,
                primitive_id: Some(primitive.primitive_id),
                component_index: Some(component.component_index),
            }),
            parent_id: if is_base {
                Some(base_group_id.clone())
            } else {
                match chain_side {
                    Some("Right") => Some(right_chain_group_id.clone()),
                    Some("Left") => Some(left_chain_group_id.clone()),
                    _ => Some(root_group_id.clone()),
                }
            },
            // The owner explicitly rejected any lava edit. C766 and every
            // non-base component therefore retain identity transforms.
            transform: template.transform.clone(),
            flags: template.flags,
            deleted: template.deleted,
        });
    }

    elements.extend([
        PlaceableAuthoringElementV1 {
            id: base_group_id,
            name: "Lower catch base between the legs".to_owned(),
            kind: PlaceableElementKindV1::Group,
            source: None,
            parent_id: Some(root_group_id.clone()),
            transform: PlaceableElementTransformV1 {
                translation: base_translation,
                scale: base_scale,
                ..PlaceableElementTransformV1::default()
            },
            flags: PlaceableElementFlagsV1::default(),
            deleted: false,
        },
        PlaceableAuthoringElementV1 {
            id: right_chain_group_id,
            name: "Right body-to-base chain".to_owned(),
            kind: PlaceableElementKindV1::Group,
            source: None,
            parent_id: Some(root_group_id.clone()),
            transform: PlaceableElementTransformV1::default(),
            flags: PlaceableElementFlagsV1::default(),
            deleted: false,
        },
        PlaceableAuthoringElementV1 {
            id: left_chain_group_id,
            name: "Left body-to-base chain".to_owned(),
            kind: PlaceableElementKindV1::Group,
            source: None,
            parent_id: Some(root_group_id.clone()),
            transform: PlaceableElementTransformV1::default(),
            flags: PlaceableElementFlagsV1::default(),
            deleted: false,
        },
        PlaceableAuthoringElementV1 {
            id: root_group_id,
            name: "Reactor physical scale 1.50 m".to_owned(),
            kind: PlaceableElementKindV1::Group,
            source: None,
            parent_id: None,
            transform: PlaceableElementTransformV1 {
                scale: [REACTOR_V3_ROOT_SCALE; 3],
                ..PlaceableElementTransformV1::default()
            },
            flags: PlaceableElementFlagsV1::default(),
            deleted: false,
        },
    ]);

    Ok(PlaceableAuthoringDocumentV1 {
        schema_version: PLACEABLE_AUTHORING_SCHEMA_VERSION_V1,
        source_sha256: bootstrap.document.source_sha256,
        elements,
    })
}

fn reactor_stream_surface_radial_fraction(base_translation: [f32; 3], base_scale: [f32; 3]) -> f32 {
    const LIQUID_SURFACE_CENTER_X: f32 = 0.000_525_36;
    const LIQUID_SURFACE_CENTER_Z: f32 = 0.000_184_28;
    const LIQUID_SURFACE_HALF_X: f32 = 0.159_335_40;
    const LIQUID_SURFACE_HALF_Z: f32 = 0.161_398_71;
    const STREAM_CENTER_X: f32 = REACTOR_STREAM_PIVOT[0];
    const STREAM_CENTER_Z: f32 = REACTOR_STREAM_PIVOT[2];

    let surface_center_x = LIQUID_SURFACE_CENTER_X * base_scale[0] + base_translation[0];
    let surface_center_z = LIQUID_SURFACE_CENTER_Z * base_scale[2] + base_translation[2];
    let normalized_x =
        (STREAM_CENTER_X - surface_center_x) / (LIQUID_SURFACE_HALF_X * base_scale[0]);
    let normalized_z =
        (STREAM_CENTER_Z - surface_center_z) / (LIQUID_SURFACE_HALF_Z * base_scale[2]);
    normalized_x.hypot(normalized_z)
}

fn reactor_v3_chain_side(bounds_min: [f32; 3], bounds_max: [f32; 3]) -> Option<&'static str> {
    let inside_shared_slab = bounds_min[1] >= REACTOR_V3_CHAIN_MIN_Y
        && bounds_max[1] <= REACTOR_V3_CHAIN_MAX_Y
        && bounds_min[2] >= -REACTOR_V3_CHAIN_MAX_ABS_Z
        && bounds_max[2] <= REACTOR_V3_CHAIN_MAX_ABS_Z;
    if !inside_shared_slab {
        return None;
    }
    if bounds_min[0] >= REACTOR_V3_CHAIN_MIN_ABS_X && bounds_max[0] <= REACTOR_V3_CHAIN_MAX_ABS_X {
        Some("Right")
    } else if bounds_max[0] <= -REACTOR_V3_CHAIN_MIN_ABS_X
        && bounds_min[0] >= -REACTOR_V3_CHAIN_MAX_ABS_X
    {
        Some("Left")
    } else {
        None
    }
}

fn asset_specs(candidate: CandidateProfile) -> [AssetSpec; 3] {
    if matches!(
        candidate.kind,
        CandidateKind::LootWorkstationsV1
            | CandidateKind::LootWorkstationsV2
            | CandidateKind::LootWorkstationsV3
            | CandidateKind::LootWorkstationsV4
    ) {
        let mut specs = [
            AssetSpec {
                style_name: "Dismantling Station",
                source_path: r"C:\Projects\meshy2aurora\artifacts\meshy-tlc-workstations-20260726\tlc_disman01.glb",
                source_sha256: "86388b96b6c5c010153de279a8e0708a4f93488228d9e5633e6ffee1f98a3561",
                source_triangles: 20_337,
                meshy_task_kind: "multi-image-to-3d",
                meshy_task_id: "019f9dba-23f4-7f53-aad6-7fd42121acfc",
                preview_task_id: "",
                refine_task_id: "",
                prompt: "Industrial-gothic dismantling workstation with a heavy circular iron worktable, three articulated tool arms, orange cutting glow, chained braces, stable integrated base and a readable dark-fantasy isometric silhouette",
                model_resref: "m2a_tlcw1_dis",
                texture_resref: "m2a_tlcw1_dtx",
                blueprint_resref: "m2a_tlcw1_du",
                object_tag: "m2a_tlcw1_dismantler",
                display_name: "TLC Dismantling Station 20K",
                placement: PlaceablePlacementV1 {
                    x: 6.5,
                    y: 14.5,
                    z: 0.0,
                    bearing: 0.0,
                },
            },
            AssetSpec {
                style_name: "Purification Station",
                source_path: r"C:\Projects\meshy2aurora\artifacts\meshy-tlc-workstations-20260726\tlc_purify01.glb",
                source_sha256: "540eaa7cf3bce1583ff4d34013162ee399ee5ef30fd845eace8d50c83ca2316e",
                source_triangles: 20_151,
                meshy_task_kind: "multi-image-to-3d",
                meshy_task_id: "019f9dcb-f81b-78e8-83d5-99e5da50be51",
                preview_task_id: "",
                refine_task_id: "",
                prompt: "Industrial-gothic purification station with a reinforced circular basin, segmented black-iron body, amber fluid and runic filters, short pipes, side clamps and a stable integrated pedestal for a dark-fantasy isometric RPG",
                model_resref: "m2a_tlcw1_pur",
                texture_resref: "m2a_tlcw1_ptx",
                blueprint_resref: "m2a_tlcw1_pu",
                object_tag: "m2a_tlcw1_purifier",
                display_name: "TLC Purification Station 20K",
                placement: PlaceablePlacementV1 {
                    x: 10.0,
                    y: 14.5,
                    z: 0.0,
                    bearing: 0.0,
                },
            },
            AssetSpec {
                style_name: "Upgrading Reactor",
                source_path: r"C:\Projects\meshy2aurora\artifacts\meshy-tlc-workstations-20260726\tlc_upgrad01.glb",
                source_sha256: "cf2f6a7b43c6ccaf09ebb2d560a62bed0245ec4734b8c8df5d9b9e4dd0776ebd",
                source_triangles: 20_397,
                meshy_task_kind: "multi-image-to-3d",
                meshy_task_id: "019f9dcc-0f49-70a2-97f7-75962403ea27",
                preview_task_id: "",
                refine_task_id: "",
                prompt: "Open-topped spherical black-iron upgrading cauldron on integrated legs, heavy side chains connecting the body to the legs, visible molten fire inside, a bright lava stream flowing into a broad lower catch basin, industrial-gothic dark-fantasy isometric prop",
                model_resref: "m2a_tlcw1_upg",
                texture_resref: "m2a_tlcw1_utx",
                blueprint_resref: "m2a_tlcw1_uu",
                object_tag: "m2a_tlcw1_upgrader",
                display_name: "TLC Upgrading Reactor 20K",
                placement: PlaceablePlacementV1 {
                    x: 13.5,
                    y: 14.5,
                    z: 0.0,
                    bearing: 0.0,
                },
            },
        ];
        if candidate.kind == CandidateKind::LootWorkstationsV2 {
            specs[0].model_resref = "m2a_tlcw2_dis";
            specs[0].texture_resref = "m2a_tlcw2_dtx";
            specs[0].blueprint_resref = "m2a_tlcw2_du";
            specs[0].object_tag = "m2a_tlcw2_dismantler";
            specs[0].display_name = "TLC V2 Dismantling Station 20K";

            specs[1].model_resref = "m2a_tlcw2_pur";
            specs[1].texture_resref = "m2a_tlcw2_ptx";
            specs[1].blueprint_resref = "m2a_tlcw2_pu";
            specs[1].object_tag = "m2a_tlcw2_purifier";
            specs[1].display_name = "TLC V2 Purification Station 20K";

            specs[2].model_resref = "m2a_tlcw2_upg";
            specs[2].texture_resref = "m2a_tlcw2_utx";
            specs[2].blueprint_resref = "m2a_tlcw2_uu";
            specs[2].object_tag = "m2a_tlcw2_upgrader";
            specs[2].display_name = "TLC V2 Upgrading Reactor 20K";
        } else if candidate.kind == CandidateKind::LootWorkstationsV3 {
            specs[0].model_resref = "m2a_tlcw3_dis";
            specs[0].texture_resref = "m2a_tlcw3_dtx";
            specs[0].blueprint_resref = "m2a_tlcw3_du";
            specs[0].object_tag = "m2a_tlcw3_dismantler";
            specs[0].display_name = "TLC V3 Dismantling Station 20K";

            specs[1].model_resref = "m2a_tlcw3_pur";
            specs[1].texture_resref = "m2a_tlcw3_ptx";
            specs[1].blueprint_resref = "m2a_tlcw3_pu";
            specs[1].object_tag = "m2a_tlcw3_purifier";
            specs[1].display_name = "TLC V3 Purification Station 20K";

            specs[2].model_resref = "m2a_tlcw3_upg";
            specs[2].texture_resref = "m2a_tlcw3_utx";
            specs[2].blueprint_resref = "m2a_tlcw3_uu";
            specs[2].object_tag = "m2a_tlcw3_upgrader";
            specs[2].display_name = "TLC V3 Upgrading Reactor 20K";
        } else if candidate.kind == CandidateKind::LootWorkstationsV4 {
            specs[0].model_resref = "m2a_tlcw4_dis";
            specs[0].texture_resref = "m2a_tlcw4_dtx";
            specs[0].blueprint_resref = "m2a_tlcw4_du";
            specs[0].object_tag = "m2a_tlcw4_dismantler";
            specs[0].display_name = "TLC V4 Dismantling Station 20K";

            specs[1].model_resref = "m2a_tlcw4_pur";
            specs[1].texture_resref = "m2a_tlcw4_ptx";
            specs[1].blueprint_resref = "m2a_tlcw4_pu";
            specs[1].object_tag = "m2a_tlcw4_purifier";
            specs[1].display_name = "TLC V4 Purification Station 20K";

            specs[2].model_resref = "m2a_tlcw4_upg";
            specs[2].texture_resref = "m2a_tlcw4_utx";
            specs[2].blueprint_resref = "m2a_tlcw4_uu";
            specs[2].object_tag = "m2a_tlcw4_upgrader";
            specs[2].display_name = "TLC V4 Upgrading Reactor 20K";
        }
        return specs;
    }
    let mut specs = [
        AssetSpec {
            style_name: "Civic Reliquary",
            source_path: r"C:\Projects\meshy2aurora\test-assets\meshy\active\tlc-p20k-v1\final\tlc-civic-reliquary-20000.glb",
            source_sha256: "865fe8eaab2e2996354faf19491b7ccf341a3239a30762c78cadad3b3a0dc994",
            source_triangles: 21_550,
            meshy_task_kind: "text-to-3d",
            meshy_task_id: "019f9b1a-a953-77a2-8d0a-825d6fb8fa6a",
            preview_task_id: "019f9b19-50ba-7ec7-bcce-4452b1dd4e78",
            refine_task_id: "019f9b1a-a953-77a2-8d0a-825d6fb8fa6a",
            prompt: "A single freestanding industrial-gothic civic reliquary for a dark medieval fantasy city and classic isometric RPG, monumental soot-stained carved stone pedestal with heavy black iron braces, oxidized brass bands, inset cyan arcane seals, layered architectural crown, readable symmetrical silhouette, flat stable bottom, one watertight static prop, no character, no weapon, no background, no text, no floating parts, no thin chains, no glass, no transparency",
            model_resref: "m2a_tlcm_rel",
            texture_resref: "m2a_tlcm_brs",
            blueprint_resref: "m2a_tlcm_rel_u",
            object_tag: "m2a_tlcm_civic_reliquary",
            display_name: "TLC Meshy Civic Reliquary 20K",
            placement: PlaceablePlacementV1 {
                x: 6.5,
                y: 14.5,
                z: 0.0,
                bearing: 0.0,
            },
        },
        AssetSpec {
            style_name: "Aether Street Lamp",
            source_path: r"C:\Projects\meshy2aurora\test-assets\meshy\active\tlc-p20k-v1\final\tlc-aether-lamp-20000.glb",
            source_sha256: "310a6597f92bfcd391f44d293585417fa53abe3f1a576d93a7e0dc2c586b1ac7",
            source_triangles: 21_456,
            meshy_task_kind: "text-to-3d",
            meshy_task_id: "019f9b1c-1e20-7813-8711-d6cfc8eeab44",
            preview_task_id: "019f9b1b-4ed3-7f64-98c4-3cdccc268092",
            refine_task_id: "019f9b1c-1e20-7813-8711-d6cfc8eeab44",
            prompt: "A single freestanding industrial-gothic arcane street beacon for a dark medieval fantasy city and classic isometric RPG, tall soot-black iron and oxidized brass lamp tower, massive stone foot, enclosed amber aether crystal chamber protected by thick metal ribs, riveted civic ornament, readable vertical silhouette, flat stable bottom, one watertight static prop, no character, no weapon, no background, no text, no floating parts, no thin chains, no transparent glass",
            model_resref: "m2a_tlcm_lamp",
            texture_resref: "m2a_tlcm_amb",
            blueprint_resref: "m2a_tlcm_lamp_u",
            object_tag: "m2a_tlcm_aether_lamp",
            display_name: "TLC Meshy Aether Lamp 20K",
            placement: PlaceablePlacementV1 {
                x: 10.0,
                y: 14.5,
                z: 0.0,
                bearing: 0.0,
            },
        },
        AssetSpec {
            style_name: "Sewer Ward Barricade",
            source_path: r"C:\Projects\meshy2aurora\test-assets\meshy\active\tlc-p20k-v1\final\tlc-sewer-ward-20000.glb",
            source_sha256: "4192345b22327997708bad441fc70ce35be8db45331aa01dd928eeb432cc28c1",
            source_triangles: 22_037,
            meshy_task_kind: "text-to-3d",
            meshy_task_id: "019f9b1c-ec55-7849-9a31-c51986251f05",
            preview_task_id: "019f9b1b-a06b-77df-a4bc-476e112c4dc4",
            refine_task_id: "019f9b1c-ec55-7849-9a31-c51986251f05",
            prompt: "A single heavy industrial-gothic sewer ward barricade for a dark medieval fantasy city and classic isometric RPG, broad low soot-stained stone and black iron defensive block, oxidized brass rivets, layered armor plates, carved drainage grilles, inset green arcane ward seals, chunky readable silhouette, flat stable bottom, one watertight static prop, no character, no weapon, no background, no text, no floating parts, no thin rods, no chains, no glass, no transparency",
            model_resref: "m2a_tlcm_ward",
            texture_resref: "m2a_tlcm_iron",
            blueprint_resref: "m2a_tlcm_ward_u",
            object_tag: "m2a_tlcm_sewer_ward",
            display_name: "TLC Meshy Sewer Ward 20K",
            placement: PlaceablePlacementV1 {
                x: 13.5,
                y: 14.5,
                z: 0.0,
                bearing: 0.0,
            },
        },
    ];
    if candidate.kind == CandidateKind::ShadowV2 {
        specs[0].model_resref = "m2a_tlcs2_rel";
        specs[0].texture_resref = "m2a_tlcs2_brs";
        specs[0].blueprint_resref = "m2a_tlcs2_ru";
        specs[0].object_tag = "m2a_tlcs2_civic_reliquary";
        specs[0].display_name = "TLC Shadow V2 Civic Reliquary 20K";

        specs[1].model_resref = "m2a_tlcs2_lmp";
        specs[1].texture_resref = "m2a_tlcs2_amb";
        specs[1].blueprint_resref = "m2a_tlcs2_lu";
        specs[1].object_tag = "m2a_tlcs2_aether_lamp";
        specs[1].display_name = "TLC Shadow V2 Aether Lamp 20K";

        specs[2].model_resref = "m2a_tlcs2_wrd";
        specs[2].texture_resref = "m2a_tlcs2_irn";
        specs[2].blueprint_resref = "m2a_tlcs2_wu";
        specs[2].object_tag = "m2a_tlcs2_sewer_ward";
        specs[2].display_name = "TLC Shadow V2 Sewer Ward 20K";
    }
    specs
}

fn merge_instance_lists(
    payloads: &[&[u8]],
    expected_file_type: GffFileTypeV1,
    list_label: &str,
) -> Result<Vec<u8>, String> {
    let first = payloads
        .first()
        .ok_or_else(|| "TLC-MESHY-P20K-GFF-MERGE-EMPTY".to_owned())?;
    let mut merged = read_gff_v32(first, &GffLimitsV1::default())
        .map_err(|error| format!("TLC-MESHY-P20K-GFF-MERGE-READ: {error}"))?;
    if merged.file_type != expected_file_type {
        return Err("TLC-MESHY-P20K-GFF-MERGE-TYPE".to_owned());
    }
    let mut instances = Vec::with_capacity(payloads.len());
    for payload in payloads {
        let document = read_gff_v32(payload, &GffLimitsV1::default())
            .map_err(|error| format!("TLC-MESHY-P20K-GFF-MERGE-READ: {error}"))?;
        if document.file_type != expected_file_type {
            return Err("TLC-MESHY-P20K-GFF-MERGE-TYPE".to_owned());
        }
        let list = list_value(&document, list_label)?;
        if list.len() != 1 || list[0].struct_id != 9 {
            return Err(format!("TLC-MESHY-P20K-GFF-MERGE-SINGLETON: {list_label}"));
        }
        instances.push(list[0].clone());
    }
    *list_value_mut(&mut merged, list_label)? = instances;
    write_gff_v32(&merged, &GffWriterOptionsV1::default())
        .map(|artifact| artifact.payload)
        .map_err(|error| format!("TLC-MESHY-P20K-GFF-MERGE-WRITE: {error}"))
}

fn merge_palette_entries(payloads: &[&[u8]], palette_id: u8) -> Result<Vec<u8>, String> {
    let first = payloads
        .first()
        .ok_or_else(|| "TLC-MESHY-P20K-ITP-MERGE-EMPTY".to_owned())?;
    let mut merged = read_gff_v32(first, &GffLimitsV1::default())
        .map_err(|error| format!("TLC-MESHY-P20K-ITP-MERGE-READ: {error}"))?;
    let mut entries = Vec::with_capacity(payloads.len());
    for payload in payloads {
        let document = read_gff_v32(payload, &GffLimitsV1::default())
            .map_err(|error| format!("TLC-MESHY-P20K-ITP-MERGE-READ: {error}"))?;
        let category = find_palette_category(list_value(&document, "MAIN")?, palette_id)
            .ok_or_else(|| "TLC-MESHY-P20K-ITP-CATEGORY-MISSING".to_owned())?;
        let list = struct_list(category, "LIST")?;
        if list.len() != 1 {
            return Err("TLC-MESHY-P20K-ITP-ENTRY-SINGLETON".to_owned());
        }
        entries.push(list[0].clone());
    }
    let category = find_palette_category_mut(list_value_mut(&mut merged, "MAIN")?, palette_id)
        .ok_or_else(|| "TLC-MESHY-P20K-ITP-CATEGORY-MISSING".to_owned())?;
    *struct_list_mut(category, "LIST")? = entries;
    write_gff_v32(&merged, &GffWriterOptionsV1::default())
        .map(|artifact| artifact.payload)
        .map_err(|error| format!("TLC-MESHY-P20K-ITP-MERGE-WRITE: {error}"))
}

fn validate_combined_package(
    hak_bytes: &[u8],
    module_bytes: &[u8],
    expected_two_da: &[u8],
    area_resref: &str,
    assets: &[PreparedAsset],
) -> Result<(), String> {
    let hak = ErfArchive::parse(hak_bytes)
        .map_err(|error| format!("TLC-MESHY-P20K-COMBINED-HAK-READBACK: {error}"))?;
    let module = ErfArchive::parse(module_bytes)
        .map_err(|error| format!("TLC-MESHY-P20K-COMBINED-MOD-READBACK: {error}"))?;
    if hak.resources().len() != 1 + assets.len() * 3 || module.resources().len() != 6 + assets.len()
    {
        return Err("TLC-MESHY-P20K-RESOURCE-COUNT".to_owned());
    }
    if find(&hak, "placeables", PLACEABLES_2DA_RESOURCE_TYPE)? != expected_two_da {
        return Err("TLC-MESHY-P20K-2DA-READBACK-DIFF".to_owned());
    }
    for asset in assets {
        let mdl = find(&hak, asset.spec.model_resref, MDL_RESOURCE_TYPE)?;
        let inspection = inspect_binary_mdl(mdl)
            .map_err(|error| format!("TLC-MESHY-P20K-MDL-READBACK: {error}"))?;
        let mesh = inspection
            .node_tree
            .roots
            .first()
            .and_then(|root| root.children.first())
            .and_then(|node| node.mesh.as_ref())
            .ok_or_else(|| {
                format!(
                    "TLC-MESHY-P20K-MDL-MESH-MISSING: {}",
                    asset.spec.model_resref
                )
            })?;
        if mesh.faces.len() != TRIANGLE_COUNT
            || mesh.index_counts.as_slice() != [INDEX_COUNT as u32]
            || mesh.raw_indices.len() != 1
            || mesh.raw_indices[0].len() != INDEX_COUNT
        {
            return Err(format!(
                "TLC-MESHY-P20K-MDL-GEOMETRY-DIFF: {}",
                asset.spec.model_resref
            ));
        }
        let pwk = inspect_ascii_placeable_walkmesh_v1(find(
            &hak,
            asset.spec.model_resref,
            PWK_RESOURCE_TYPE,
        )?)
        .map_err(|error| format!("TLC-MESHY-P20K-PWK-READBACK: {error}"))?;
        if pwk.mesh_nodes.len() != 1
            || pwk.mesh_nodes[0].vertices.len() != 4
            || pwk.mesh_nodes[0].faces.len() != 2
            || pwk.mesh_nodes[0]
                .faces
                .iter()
                .any(|face| face.surface_id != 7)
        {
            return Err(format!(
                "TLC-MESHY-P20K-PWK-SEMANTIC-DIFF: {}",
                asset.spec.model_resref
            ));
        }
        find(&hak, asset.spec.texture_resref, TGA_RESOURCE_TYPE)?;
        find(&module, asset.spec.blueprint_resref, UTP_RESOURCE_TYPE)?;
    }
    let git = read_gff_v32(
        find(&module, area_resref, GIT_RESOURCE_TYPE)?,
        &GffLimitsV1::default(),
    )
    .map_err(|error| format!("TLC-MESHY-P20K-GIT-READBACK: {error}"))?;
    let instances = list_value(&git, "Placeable List")?;
    if instances.len() != assets.len() {
        return Err("TLC-MESHY-P20K-GIT-INSTANCE-COUNT".to_owned());
    }
    for (instance, asset) in instances.iter().zip(assets) {
        if field_value(instance, "Appearance") != Some(&GffValueV1::Dword(asset.appearance_row))
            || field_value(instance, "TemplateResRef")
                != Some(&GffValueV1::ResRef(asset.spec.blueprint_resref.to_owned()))
        {
            return Err(format!(
                "TLC-MESHY-P20K-GIT-IDENTITY-DIFF: {}",
                asset.spec.object_tag
            ));
        }
    }
    let gic = read_gff_v32(
        find(&module, area_resref, GIC_RESOURCE_TYPE)?,
        &GffLimitsV1::default(),
    )
    .map_err(|error| format!("TLC-MESHY-P20K-GIC-READBACK: {error}"))?;
    if list_value(&gic, "Placeable List")?.len() != assets.len() {
        return Err("TLC-MESHY-P20K-GIC-INSTANCE-COUNT".to_owned());
    }
    let itp = read_gff_v32(
        find(&module, "placeablepalcus", ITP_RESOURCE_TYPE)?,
        &GffLimitsV1::default(),
    )
    .map_err(|error| format!("TLC-MESHY-P20K-ITP-READBACK: {error}"))?;
    let category = find_palette_category(list_value(&itp, "MAIN")?, 7)
        .ok_or_else(|| "TLC-MESHY-P20K-ITP-CATEGORY-MISSING".to_owned())?;
    let entries = struct_list(category, "LIST")?;
    if entries.len() != assets.len() {
        return Err("TLC-MESHY-P20K-ITP-ENTRY-COUNT".to_owned());
    }
    for asset in assets {
        let count = entries
            .iter()
            .filter(|entry| {
                field_value(entry, "RESREF")
                    == Some(&GffValueV1::ResRef(asset.spec.blueprint_resref.to_owned()))
            })
            .count();
        if count != 1 {
            return Err(format!(
                "TLC-MESHY-P20K-ITP-IDENTITY-DIFF: {}",
                asset.spec.blueprint_resref
            ));
        }
    }
    Ok(())
}

fn asset_manifest(asset: &PreparedAsset) -> Value {
    json!({
        "styleName": asset.spec.style_name,
        "displayName": asset.spec.display_name,
        "modelResref": asset.spec.model_resref,
        "textureResref": asset.spec.texture_resref,
        "blueprintResref": asset.spec.blueprint_resref,
        "objectTag": asset.spec.object_tag,
        "appearanceRow": asset.appearance_row,
        "placement": asset.spec.placement,
        "meshy": {
            "taskKind": asset.spec.meshy_task_kind,
            "taskId": asset.spec.meshy_task_id,
            "previewTaskId": asset.spec.preview_task_id,
            "refineTaskId": asset.spec.refine_task_id,
            "prompt": asset.spec.prompt,
            "sourceTrianglesBeforeExactReduction": asset.spec.source_triangles,
            "finalGlbPath": asset.spec.source_path,
            "finalGlbSha256": sha256(&asset.source)
        },
        "geometry": {
            "meshCount": 1,
            "vertexCount": asset.vertex_count,
            "triangleCount": TRIANGLE_COUNT,
            "indexCount": INDEX_COUNT,
            "adjacencyBoundaryEdgeCount": asset.adjacency_boundary_edge_count,
            "adjacencyLinkedEdgeCount": asset.adjacency_linked_edge_count,
            "mdlSha256": sha256(&asset.mdl)
        },
        "authoring": asset.authoring_sha256.as_ref().map(|authoring_sha256| {
            if matches!(
                asset.spec.model_resref,
                "m2a_tlcw3_upg" | "m2a_tlcw4_upg"
            ) {
                let (base_components, base_translation, base_scale, stream_radial_fraction) =
                    if asset.spec.model_resref == "m2a_tlcw4_upg" {
                        (
                            REACTOR_V4_BASE_COMPONENTS.as_slice(),
                            REACTOR_V4_BASE_TRANSLATION,
                            REACTOR_V4_BASE_SCALE,
                            Some(reactor_stream_surface_radial_fraction(
                                REACTOR_V4_BASE_TRANSLATION,
                                REACTOR_V4_BASE_SCALE,
                            )),
                        )
                    } else {
                        (
                            REACTOR_V3_BASE_COMPONENTS.as_slice(),
                            REACTOR_V3_BASE_TRANSLATION,
                            REACTOR_V3_BASE_SCALE,
                            None,
                        )
                    };
                json!({
                    "schemaVersion": PLACEABLE_AUTHORING_SCHEMA_VERSION_V1,
                    "authoringSha256": authoring_sha256,
                    "lowerBaseComponentIds": base_components,
                    "lowerBaseTranslation": base_translation,
                    "lowerBaseScale": base_scale,
                    "wholeModelUniformScale": REACTOR_V3_ROOT_SCALE,
                    "targetPhysicalHeightMeters": 1.5,
                    "lavaStreamComponentId": REACTOR_LAVA_STREAM_COMPONENT,
                    "lavaStreamLocalTransform": "identity",
                    "lavaStreamSurfaceRadialFraction": stream_radial_fraction,
                    "lavaStreamSurfaceMaximumRadialFraction": if stream_radial_fraction.is_some() {
                        Some(REACTOR_V4_MAX_STREAM_RADIAL_FRACTION)
                    } else {
                        None
                    },
                    "sideChainGroups": 2,
                    "sideChainTransforms": "identity"
                })
            } else {
                json!({
                    "schemaVersion": PLACEABLE_AUTHORING_SCHEMA_VERSION_V1,
                    "authoringSha256": authoring_sha256,
                    "lowerBasinComponentIds": REACTOR_V2_BASIN_COMPONENTS,
                    "lowerBasinTranslation": [0.0, REACTOR_BASIN_RAISE_METERS, 0.0],
                    "lavaStreamComponentId": REACTOR_LAVA_STREAM_COMPONENT,
                    "lavaStreamPivot": REACTOR_STREAM_PIVOT,
                    "lavaStreamRotationXDegrees": REACTOR_STREAM_ROTATION_X_DEGREES,
                    "lavaStreamScaleY": REACTOR_STREAM_SCALE_Y
                })
            }
        }),
        "collision": {
            "pwkSha256": sha256(&asset.pwk),
            "offlineReadback": "passed"
        },
        "textureSha256": sha256(&asset.texture),
        "utpSha256": sha256(&asset.utp)
    })
}

fn list_value<'a>(
    document: &'a GffDocumentV1,
    label: &str,
) -> Result<&'a Vec<GffStructV1>, String> {
    document
        .root
        .fields
        .iter()
        .find_map(|field| match &field.value {
            GffValueV1::List(value) if field.label == label => Some(value),
            _ => None,
        })
        .ok_or_else(|| format!("TLC-MESHY-P20K-GFF-LIST-MISSING: {label}"))
}

fn list_value_mut<'a>(
    document: &'a mut GffDocumentV1,
    label: &str,
) -> Result<&'a mut Vec<GffStructV1>, String> {
    document
        .root
        .fields
        .iter_mut()
        .find_map(|field| match &mut field.value {
            GffValueV1::List(value) if field.label == label => Some(value),
            _ => None,
        })
        .ok_or_else(|| format!("TLC-MESHY-P20K-GFF-LIST-MISSING: {label}"))
}

fn struct_list<'a>(
    structure: &'a GffStructV1,
    label: &str,
) -> Result<&'a Vec<GffStructV1>, String> {
    structure
        .fields
        .iter()
        .find_map(|field| match &field.value {
            GffValueV1::List(value) if field.label == label => Some(value),
            _ => None,
        })
        .ok_or_else(|| format!("TLC-MESHY-P20K-GFF-STRUCT-LIST-MISSING: {label}"))
}

fn struct_list_mut<'a>(
    structure: &'a mut GffStructV1,
    label: &str,
) -> Result<&'a mut Vec<GffStructV1>, String> {
    structure
        .fields
        .iter_mut()
        .find_map(|field| match &mut field.value {
            GffValueV1::List(value) if field.label == label => Some(value),
            _ => None,
        })
        .ok_or_else(|| format!("TLC-MESHY-P20K-GFF-STRUCT-LIST-MISSING: {label}"))
}

fn find_palette_category(structures: &[GffStructV1], palette_id: u8) -> Option<&GffStructV1> {
    for structure in structures {
        if field_value(structure, "ID") == Some(&GffValueV1::Byte(palette_id)) {
            return Some(structure);
        }
        for field in &structure.fields {
            if let GffValueV1::List(children) = &field.value
                && let Some(found) = find_palette_category(children, palette_id)
            {
                return Some(found);
            }
        }
    }
    None
}

fn find_palette_category_mut(
    structures: &mut [GffStructV1],
    palette_id: u8,
) -> Option<&mut GffStructV1> {
    for structure in structures {
        if field_value(structure, "ID") == Some(&GffValueV1::Byte(palette_id)) {
            return Some(structure);
        }
        for field in &mut structure.fields {
            if let GffValueV1::List(children) = &mut field.value
                && let Some(found) = find_palette_category_mut(children, palette_id)
            {
                return Some(found);
            }
        }
    }
    None
}

fn field_value<'a>(structure: &'a GffStructV1, label: &str) -> Option<&'a GffValueV1> {
    structure
        .fields
        .iter()
        .find(|field| field.label == label)
        .map(|field| &field.value)
}

fn find<'a>(
    archive: &'a ErfArchive<'a>,
    resref: &str,
    resource_type: u16,
) -> Result<&'a [u8], String> {
    archive.find(resref, resource_type).map_err(|error| {
        format!("TLC-MESHY-P20K-RESOURCE-MISSING {resref}:{resource_type}: {error}")
    })
}

fn resource(resref: &str, resource_type: u16, payload: Vec<u8>) -> HakResourceInputV1 {
    HakResourceInputV1 {
        resref: resref.to_owned(),
        resource_type,
        payload,
    }
}

struct Command {
    placeables_two_da: PathBuf,
    output: PathBuf,
}

fn candidate_for_output(output: &Path) -> Result<CandidateProfile, String> {
    [
        V1_PROFILE,
        SHADOW_V2_PROFILE,
        LOOT_WORKSTATIONS_V1_PROFILE,
        LOOT_WORKSTATIONS_V2_PROFILE,
        LOOT_WORKSTATIONS_V3_PROFILE,
        LOOT_WORKSTATIONS_V4_PROFILE,
    ]
    .into_iter()
    .find(|candidate| output == Path::new(candidate.canonical_output))
    .ok_or_else(|| {
        format!(
            "TLC-MESHY-P20K-OUTPUT-IDENTITY: exact output must be {}, {}, {}, {}, {}, or {}",
            V1_PROFILE.canonical_output,
            SHADOW_V2_PROFILE.canonical_output,
            LOOT_WORKSTATIONS_V1_PROFILE.canonical_output,
            LOOT_WORKSTATIONS_V2_PROFILE.canonical_output,
            LOOT_WORKSTATIONS_V3_PROFILE.canonical_output,
            LOOT_WORKSTATIONS_V4_PROFILE.canonical_output
        )
    })
}

fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut placeables_two_da = None;
    let mut output = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let target = match argument.as_str() {
            "--placeables-2da" => &mut placeables_two_da,
            "--out" => &mut output,
            _ => {
                return Err(format!("TLC-MESHY-P20K-ARGUMENT: {argument}\n{}", usage()));
            }
        };
        if target.is_some() {
            return Err(format!("TLC-MESHY-P20K-ARGUMENT-DUPLICATE: {argument}"));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!("TLC-MESHY-P20K-ARGUMENT-MISSING: {argument}"));
        }
    }
    Ok(Command {
        placeables_two_da: PathBuf::from(placeables_two_da.ok_or_else(usage)?),
        output: PathBuf::from(output.ok_or_else(usage)?),
    })
}

fn usage() -> String {
    format!(
        "usage: materialize_tlc_meshy_p20k_placeables --placeables-2da <retail-table> --out <{}|{}|{}|{}|{}|{}>",
        V1_PROFILE.canonical_output,
        SHADOW_V2_PROFILE.canonical_output,
        LOOT_WORKSTATIONS_V1_PROFILE.canonical_output,
        LOOT_WORKSTATIONS_V2_PROFILE.canonical_output,
        LOOT_WORKSTATIONS_V3_PROFILE.canonical_output,
        LOOT_WORKSTATIONS_V4_PROFILE.canonical_output
    )
}

fn read(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    fs::read(path)
        .map_err(|error| format!("TLC-MESHY-P20K-{label}-READ {}: {error}", path.display()))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("TLC-MESHY-P20K-CREATE-NEW {}: {error}", path.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("TLC-MESHY-P20K-WRITE {}: {error}", path.display()))
}

fn require_hash(bytes: &[u8], expected: &str, label: &str) -> Result<(), String> {
    let actual = sha256(bytes);
    if actual != expected {
        return Err(format!(
            "TLC-MESHY-P20K-{label}-HASH: expected {expected}, got {actual}"
        ));
    }
    Ok(())
}

fn pretty(value: &impl serde::Serialize, label: &str) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(value)
        .map_err(|error| format!("TLC-MESHY-P20K-{label}-SERIALIZE: {error}"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn exact_meshy_trio_has_unique_runtime_identities_and_expected_rows() {
        for profile in [
            LOOT_WORKSTATIONS_V1_PROFILE,
            LOOT_WORKSTATIONS_V2_PROFILE,
            LOOT_WORKSTATIONS_V3_PROFILE,
            LOOT_WORKSTATIONS_V4_PROFILE,
        ] {
            let specs = asset_specs(profile);
            assert_eq!(specs.len(), 3);
            assert_eq!(
                specs
                    .iter()
                    .map(|spec| spec.model_resref)
                    .collect::<BTreeSet<_>>()
                    .len(),
                3
            );
            assert_eq!(
                specs
                    .iter()
                    .map(|spec| spec.blueprint_resref)
                    .collect::<BTreeSet<_>>()
                    .len(),
                3
            );
            assert!(
                specs
                    .iter()
                    .all(|spec| spec.source_triangles > TRIANGLE_COUNT)
            );
        }
        assert!(TRIANGLE_COUNT <= NWN_EE_MAX_MESH_TRIANGLE_COUNT_V1);
        assert!(INDEX_COUNT <= NWN_EE_MAX_MESH_INDEX_COUNT_V1);
    }

    #[test]
    fn reactor_alignment_keeps_the_basin_centered_and_lands_the_stream_inside_it() {
        let source_stream_bottom = [0.010_017_341_f32, 0.112_978_53, 0.226_872_39];
        let scaled_y = (source_stream_bottom[1] - REACTOR_STREAM_PIVOT[1]) * REACTOR_STREAM_SCALE_Y;
        let radians = REACTOR_STREAM_ROTATION_X_DEGREES.to_radians();
        let transformed_bottom_y = REACTOR_STREAM_PIVOT[1] + scaled_y * radians.cos();
        let transformed_bottom_z = REACTOR_STREAM_PIVOT[2] + scaled_y * radians.sin();
        let basin_surface_y = [
            0.051_654_562 + REACTOR_BASIN_RAISE_METERS,
            0.062_938_75 + REACTOR_BASIN_RAISE_METERS,
        ];
        let basin_surface_z = [-0.161_214_43_f32, 0.161_582_99];

        assert_eq!(REACTOR_V2_BASIN_COMPONENTS.len(), 16);
        assert_eq!(REACTOR_BASIN_RAISE_METERS, 0.05);
        assert!(
            (basin_surface_y[0]..=basin_surface_y[1]).contains(&transformed_bottom_y),
            "stream bottom Y {transformed_bottom_y} must intersect the raised basin surface"
        );
        assert!(
            (basin_surface_z[0]..=basin_surface_z[1]).contains(&transformed_bottom_z),
            "stream bottom Z {transformed_bottom_z} must land inside the centered basin"
        );
    }

    #[test]
    #[ignore = "reads the exact owned 15 MB Meshy reactor source"]
    fn exact_owned_reactor_builds_the_v2_authoring_document() {
        let source = fs::read(
            r"C:\Projects\meshy2aurora\artifacts\meshy-tlc-workstations-20260726\tlc_upgrad01.glb",
        )
        .expect("read exact owned reactor");
        require_hash(
            &source,
            "cf2f6a7b43c6ccaf09ebb2d560a62bed0245ec4734b8c8df5d9b9e4dd0776ebd",
            "TEST-REACTOR",
        )
        .expect("exact reactor hash");
        let document =
            upgrading_reactor_v2_authoring_document(&source).expect("build reactor authoring");

        assert_eq!(document.elements.len(), 811);
        assert_eq!(
            document
                .elements
                .iter()
                .filter(|element| element.parent_id.as_deref() == Some("group:reactor-lower-basin"))
                .count(),
            REACTOR_V2_BASIN_COMPONENTS.len()
        );
        let stream = document
            .elements
            .iter()
            .find(|element| {
                element
                    .source
                    .as_ref()
                    .and_then(|source| source.component_index)
                    == Some(REACTOR_LAVA_STREAM_COMPONENT)
            })
            .expect("lava stream component");
        assert_eq!(stream.transform.pivot, REACTOR_STREAM_PIVOT);
        assert_eq!(stream.transform.scale, [1.0, 1.12, 1.0]);
    }

    #[test]
    #[ignore = "reads the exact owned 15 MB Meshy reactor source"]
    fn exact_owned_reactor_builds_the_v3_base_only_authoring_document() {
        let source = fs::read(
            r"C:\Projects\meshy2aurora\artifacts\meshy-tlc-workstations-20260726\tlc_upgrad01.glb",
        )
        .expect("read exact owned reactor");
        require_hash(
            &source,
            "cf2f6a7b43c6ccaf09ebb2d560a62bed0245ec4734b8c8df5d9b9e4dd0776ebd",
            "TEST-REACTOR",
        )
        .expect("exact reactor hash");
        let document =
            upgrading_reactor_v3_authoring_document(&source).expect("build reactor authoring");

        assert_eq!(document.elements.len(), 814);
        assert_eq!(
            document
                .elements
                .iter()
                .filter(|element| element.parent_id.as_deref() == Some("group:reactor-lower-base"))
                .count(),
            REACTOR_V3_BASE_COMPONENTS.len()
        );
        assert_eq!(
            document
                .elements
                .iter()
                .filter(|element| element.parent_id.as_deref() == Some("group:reactor-right-chain"))
                .count(),
            186
        );
        assert_eq!(
            document
                .elements
                .iter()
                .filter(|element| element.parent_id.as_deref() == Some("group:reactor-left-chain"))
                .count(),
            194
        );
        let stream = document
            .elements
            .iter()
            .find(|element| {
                element
                    .source
                    .as_ref()
                    .and_then(|source| source.component_index)
                    == Some(REACTOR_LAVA_STREAM_COMPONENT)
            })
            .expect("lava stream component");
        assert_eq!(
            stream.parent_id.as_deref(),
            Some("group:reactor-physical-scale")
        );
        assert_eq!(
            stream.transform,
            PlaceableElementTransformV1::default(),
            "V3 must not move, rotate, scale, or re-pivot the lava stream"
        );
        let base = document
            .elements
            .iter()
            .find(|element| element.id == "group:reactor-lower-base")
            .expect("lower base group");
        assert_eq!(base.transform.translation, REACTOR_V3_BASE_TRANSLATION);
        assert_eq!(base.transform.scale, REACTOR_V3_BASE_SCALE);
        assert_eq!(
            base.parent_id.as_deref(),
            Some("group:reactor-physical-scale")
        );
        let root = document
            .elements
            .iter()
            .find(|element| element.id == "group:reactor-physical-scale")
            .expect("reactor physical-scale group");
        assert_eq!(root.transform.scale, [REACTOR_V3_ROOT_SCALE; 3]);

        let transformed_rim_max_x =
            0.191_083_97_f32 * REACTOR_V3_BASE_SCALE[0] * REACTOR_V3_ROOT_SCALE;
        let nearest_leg_inner_x = 0.105_288_78_f32 * REACTOR_V3_ROOT_SCALE;
        assert!(
            transformed_rim_max_x < nearest_leg_inner_x,
            "base rim must remain inside the nearest leg footprint"
        );
        let transformed_surface_max_z = (0.161_582_99_f32 * REACTOR_V3_BASE_SCALE[2]
            + REACTOR_V3_BASE_TRANSLATION[2])
            * REACTOR_V3_ROOT_SCALE;
        assert!(
            transformed_surface_max_z >= 0.239_554_82_f32 * REACTOR_V3_ROOT_SCALE,
            "unchanged stream endpoint must land over the transformed catch surface"
        );
    }

    #[test]
    fn v4_rejects_the_v3_rim_hit_and_requires_a_safe_interior_lava_target() {
        let rejected_v3_fraction = reactor_stream_surface_radial_fraction(
            REACTOR_V3_BASE_TRANSLATION,
            REACTOR_V3_BASE_SCALE,
        );
        let accepted_v4_fraction = reactor_stream_surface_radial_fraction(
            REACTOR_V4_BASE_TRANSLATION,
            REACTOR_V4_BASE_SCALE,
        );

        assert!(
            rejected_v3_fraction > 0.80,
            "V3 must be represented as a near-rim hit, got {rejected_v3_fraction}"
        );
        assert!(
            accepted_v4_fraction <= REACTOR_V4_MAX_STREAM_RADIAL_FRACTION,
            "V4 stream must land in the usable basin interior, got {accepted_v4_fraction}"
        );
        assert!(
            accepted_v4_fraction < rejected_v3_fraction,
            "V4 must improve the stream-to-basin alignment"
        );
    }

    #[test]
    #[ignore = "reads the exact owned 15 MB Meshy reactor source"]
    fn exact_owned_reactor_builds_the_v4_safe_lava_catch_authoring_document() {
        let source = fs::read(
            r"C:\Projects\meshy2aurora\artifacts\meshy-tlc-workstations-20260726\tlc_upgrad01.glb",
        )
        .expect("read exact owned reactor");
        require_hash(
            &source,
            "cf2f6a7b43c6ccaf09ebb2d560a62bed0245ec4734b8c8df5d9b9e4dd0776ebd",
            "TEST-REACTOR",
        )
        .expect("exact reactor hash");
        let document =
            upgrading_reactor_v4_authoring_document(&source).expect("build reactor authoring");

        assert_eq!(document.elements.len(), 814);
        assert_eq!(
            document
                .elements
                .iter()
                .filter(|element| element.parent_id.as_deref() == Some("group:reactor-lower-base"))
                .count(),
            REACTOR_V4_BASE_COMPONENTS.len()
        );
        assert_eq!(
            document
                .elements
                .iter()
                .filter(|element| element.parent_id.as_deref() == Some("group:reactor-right-chain"))
                .count(),
            186
        );
        assert_eq!(
            document
                .elements
                .iter()
                .filter(|element| element.parent_id.as_deref() == Some("group:reactor-left-chain"))
                .count(),
            194
        );

        for component_index in [REACTOR_LAVA_STREAM_COMPONENT, 809] {
            let lava = document
                .elements
                .iter()
                .find(|element| {
                    element
                        .source
                        .as_ref()
                        .and_then(|source| source.component_index)
                        == Some(component_index)
                })
                .expect("lava component");
            assert_eq!(
                lava.parent_id.as_deref(),
                Some("group:reactor-physical-scale")
            );
            assert_eq!(
                lava.transform,
                PlaceableElementTransformV1::default(),
                "V4 must not move, rotate, scale, or re-pivot lava component C{component_index}"
            );
        }

        let ground_ring = document
            .elements
            .iter()
            .find(|element| {
                element
                    .source
                    .as_ref()
                    .and_then(|source| source.component_index)
                    == Some(37)
            })
            .expect("ground ring component C37");
        assert_eq!(
            ground_ring.parent_id.as_deref(),
            Some("group:reactor-physical-scale")
        );
        assert_eq!(
            ground_ring.transform,
            PlaceableElementTransformV1::default(),
            "V4 must keep ground ring C37 with the legs instead of pulling it into the basin"
        );

        let base = document
            .elements
            .iter()
            .find(|element| element.id == "group:reactor-lower-base")
            .expect("lower base group");
        assert_eq!(base.transform.translation, REACTOR_V4_BASE_TRANSLATION);
        assert_eq!(base.transform.scale, REACTOR_V4_BASE_SCALE);
        assert_eq!(
            base.parent_id.as_deref(),
            Some("group:reactor-physical-scale")
        );

        let root = document
            .elements
            .iter()
            .find(|element| element.id == "group:reactor-physical-scale")
            .expect("reactor physical-scale group");
        assert_eq!(root.transform.scale, [REACTOR_V3_ROOT_SCALE; 3]);

        let transformed_rim_max_x =
            0.191_083_97_f32 * REACTOR_V4_BASE_SCALE[0] * REACTOR_V3_ROOT_SCALE;
        let nearest_leg_inner_x = 0.105_288_78_f32 * REACTOR_V3_ROOT_SCALE;
        assert!(
            transformed_rim_max_x < nearest_leg_inner_x,
            "base rim must remain inside the nearest leg footprint"
        );

        let stream_radial_fraction = reactor_stream_surface_radial_fraction(
            REACTOR_V4_BASE_TRANSLATION,
            REACTOR_V4_BASE_SCALE,
        );
        assert!(
            stream_radial_fraction <= REACTOR_V4_MAX_STREAM_RADIAL_FRACTION,
            "unchanged stream must land safely inside the transformed catch surface"
        );

        let transformed_surface_min_y =
            0.051_650_00_f32 * REACTOR_V4_BASE_SCALE[1] + REACTOR_V4_BASE_TRANSLATION[1];
        let transformed_surface_max_y =
            0.062_940_00_f32 * REACTOR_V4_BASE_SCALE[1] + REACTOR_V4_BASE_TRANSLATION[1];
        let stream_end_y = 0.112_978_53_f32;
        assert!(
            stream_end_y >= transformed_surface_min_y
                && stream_end_y <= transformed_surface_max_y + 0.000_1,
            "unchanged stream end must meet the raised liquid surface"
        );
    }
}

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use m2a_core::{
    erf::ErfArchive,
    hak::{HakResourceInputV1, HakWriterOptionsV1, write_hak_v1},
    model_pipeline::{
        ProceduralCreatureBuildOptionsV1, ProceduralCreatureProductIdentityV2,
        append_direct_creature_humanoid_appearance_row_v1,
        build_meshy_procedural_humanoid_product_with_options_v3,
    },
    profile_a::{CreatureSourceForwardV1, creature_source_forward_mapping_v1},
    proof_module::{
        BinaryCreatureModuleIdentityV1, BinaryCreatureOwnedFixtureV1,
        BinaryCreatureProfiledFixtureV2, BinaryCreatureRuntimeProfileV2, M0RuntimeDirectionV1,
        M0RuntimePositionV1, build_binary_creature_profile_matrix_module_v2,
    },
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const MODULE_RESREF: &str = "m2afacedemo1";
const AREA_RESREF: &str = "m2afacearea1";
const HAK_RESREF: &str = "m2afacehak1";
const TEXTURE_RESREF: &str = "m2afacetex1";
const MODULE_DISPLAY_NAME: &str = "Meshy2Aurora creature runtime-profile comparison";
const AREA_DISPLAY_NAME: &str = "Meshy2Aurora creature comparison area";

#[derive(Clone, Copy)]
struct VariantSpec {
    token: &'static str,
    display_axis: &'static str,
    forward: CreatureSourceForwardV1,
    model_resref: &'static str,
    creature_resref: &'static str,
    x: f32,
}

const VARIANTS: [VariantSpec; 4] = [
    VariantSpec {
        token: "positive_z",
        display_axis: "+Z",
        forward: CreatureSourceForwardV1::PositiveZ,
        model_resref: "m2afacepz1",
        creature_resref: "m2afcutcpz1",
        x: 4.5,
    },
    VariantSpec {
        token: "negative_z",
        display_axis: "-Z",
        forward: CreatureSourceForwardV1::NegativeZ,
        model_resref: "m2afacenz1",
        creature_resref: "m2afcutcnz1",
        x: 8.25,
    },
    VariantSpec {
        token: "positive_x",
        display_axis: "+X",
        forward: CreatureSourceForwardV1::PositiveX,
        model_resref: "m2afacepx1",
        creature_resref: "m2afcutcpx1",
        x: 11.75,
    },
    VariantSpec {
        token: "negative_x",
        display_axis: "-X",
        forward: CreatureSourceForwardV1::NegativeX,
        model_resref: "m2afacenx1",
        creature_resref: "m2afcutcnx1",
        x: 15.5,
    },
];

struct BuiltVariant {
    spec: VariantSpec,
    appearance_row: u16,
    model: Vec<u8>,
    report_json: Vec<u8>,
    summary_json: Vec<u8>,
    manifest_json: Vec<u8>,
    triangle_count: usize,
    determinant: f32,
}

struct PreparedVariant {
    spec: VariantSpec,
    input_appearance: Vec<u8>,
    expected_appearance: Vec<u8>,
    appearance_row: u16,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ByteBinding {
    file_name: String,
    byte_length: u64,
    sha256: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VariantBinding {
    source_forward: CreatureSourceForwardV1,
    source_axis: &'static str,
    forward_mapping: String,
    determinant: f32,
    model_resref: &'static str,
    creature_resref: &'static str,
    appearance_row: u16,
    triangle_count: usize,
    position: [f32; 3],
    orientation: [f32; 2],
    model: ByteBinding,
    report: ByteBinding,
    summary: ByteBinding,
    manifest: ByteBinding,
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
    let (source_path, appearance_path, output_directory) = parse_args(env::args().skip(1))?;
    if output_directory.exists() {
        return Err(format!(
            "CREATURE-FACING-DEMO-DESTINATION-EXISTS: {}",
            output_directory.display()
        ));
    }

    let source = read(&source_path, "SOURCE")?;
    let base_appearance = read(&appearance_path, "APPEARANCE")?;
    let source_binding = binding(source_path.display().to_string(), &source);
    let appearance_binding = binding(appearance_path.display().to_string(), &base_appearance);

    let mut current_appearance = base_appearance.clone();
    let mut prepared_variants = Vec::with_capacity(VARIANTS.len());
    for spec in VARIANTS {
        let appearance_label = format!("M2A_FACE_{}", spec.token.to_ascii_uppercase());
        let appended = append_direct_creature_humanoid_appearance_row_v1(
            &current_appearance,
            &appearance_label,
            spec.model_resref,
        )
        .map_err(|error| format!("CREATURE-FACING-DEMO-APPEARANCE-{}: {error}", spec.token))?;
        let appearance_row = appended.report.appended_row_index;
        let expected_appearance = appended.payload;
        prepared_variants.push(PreparedVariant {
            spec,
            input_appearance: current_appearance,
            expected_appearance: expected_appearance.clone(),
            appearance_row,
        });
        current_appearance = expected_appearance;
    }

    let built_results = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(prepared_variants.len());
        for prepared in prepared_variants {
            let source = source.as_slice();
            handles.push(scope.spawn(move || build_variant(source, prepared)));
        }
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| "CREATURE-FACING-DEMO-WORKER-PANIC".to_owned())?
            })
            .collect::<Result<Vec<_>, String>>()
    })?;

    let mut shared_texture: Option<Vec<u8>> = None;
    let mut built_variants = Vec::with_capacity(built_results.len());
    for (variant, texture) in built_results {
        if let Some(shared) = &shared_texture {
            if shared != &texture {
                return Err(format!(
                    "CREATURE-FACING-DEMO-TEXTURE-DIFF: {}",
                    variant.spec.token
                ));
            }
        } else {
            shared_texture = Some(texture);
        }
        built_variants.push(variant);
    }

    let texture =
        shared_texture.ok_or_else(|| "CREATURE-FACING-DEMO-TEXTURE-MISSING".to_owned())?;
    let mut resources = Vec::with_capacity(built_variants.len() + 2);
    for variant in &built_variants {
        resources.push(HakResourceInputV1 {
            resref: variant.spec.model_resref.to_owned(),
            resource_type: 2002,
            payload: variant.model.clone(),
        });
    }
    resources.extend([
        HakResourceInputV1 {
            resref: TEXTURE_RESREF.to_owned(),
            resource_type: 3,
            payload: texture.clone(),
        },
        HakResourceInputV1 {
            resref: "appearance".to_owned(),
            resource_type: 2017,
            payload: current_appearance.clone(),
        },
    ]);
    let hak = write_hak_v1(&resources, &HakWriterOptionsV1::default())
        .map_err(|error| format!("CREATURE-FACING-DEMO-HAK: {error}"))?;
    let hak_readback = ErfArchive::parse(&hak.payload)
        .map_err(|error| format!("CREATURE-FACING-DEMO-HAK-READBACK: {error:?}"))?;
    for resource in &resources {
        let readback = hak_readback
            .find(&resource.resref, resource.resource_type)
            .map_err(|error| format!("CREATURE-FACING-DEMO-HAK-RESOURCE: {error:?}"))?;
        if readback != resource.payload {
            return Err(format!(
                "CREATURE-FACING-DEMO-HAK-SEMANTIC-DIFF: {}:{}",
                resource.resref, resource.resource_type
            ));
        }
    }

    let module_identity = BinaryCreatureModuleIdentityV1 {
        module_resref: MODULE_RESREF.to_owned(),
        area_resref: AREA_RESREF.to_owned(),
        hak_resref: HAK_RESREF.to_owned(),
    };
    let fixtures = built_variants
        .iter()
        .map(|variant| BinaryCreatureProfiledFixtureV2 {
            fixture: BinaryCreatureOwnedFixtureV1 {
                id: format!("m2a_face_{}", variant.spec.token),
                template_resref: variant.spec.creature_resref.to_owned(),
                display_name: format!("Source {} to Aurora -Y", variant.spec.display_axis),
                appearance_row: variant.appearance_row,
                position: M0RuntimePositionV1 {
                    x: variant.spec.x,
                    y: 14.5,
                    z: 0.0,
                },
                orientation: M0RuntimeDirectionV1 { x: 0.0, y: -1.0 },
            },
            runtime_profile: BinaryCreatureRuntimeProfileV2::ActiveMonsterBaseline,
        })
        .collect::<Vec<_>>();
    let module = build_binary_creature_profile_matrix_module_v2(&module_identity, &fixtures)
        .map_err(|error| format!("CREATURE-FACING-DEMO-MODULE: {error}"))?;

    fs::create_dir(&output_directory).map_err(|error| {
        format!(
            "CREATURE-FACING-DEMO-OUTPUT-CREATE {}: {error}",
            output_directory.display()
        )
    })?;

    let mut variant_bindings = Vec::with_capacity(built_variants.len());
    for variant in &built_variants {
        let model_name = format!("{}.mdl", variant.spec.model_resref);
        let report_name = format!("inspection-{}.json", variant.spec.token);
        let summary_name = format!("summary-{}.json", variant.spec.token);
        let manifest_name = format!("conversion-manifest-{}.json", variant.spec.token);
        write_new(&output_directory.join(&model_name), &variant.model)?;
        write_new(&output_directory.join(&report_name), &variant.report_json)?;
        write_new(&output_directory.join(&summary_name), &variant.summary_json)?;
        write_new(
            &output_directory.join(&manifest_name),
            &variant.manifest_json,
        )?;
        variant_bindings.push(VariantBinding {
            source_forward: variant.spec.forward,
            source_axis: variant.spec.display_axis,
            forward_mapping: creature_source_forward_mapping_v1(variant.spec.forward).to_owned(),
            determinant: variant.determinant,
            model_resref: variant.spec.model_resref,
            creature_resref: variant.spec.creature_resref,
            appearance_row: variant.appearance_row,
            triangle_count: variant.triangle_count,
            position: [variant.spec.x, 14.5, 0.0],
            orientation: [0.0, -1.0],
            model: binding(model_name, &variant.model),
            report: binding(report_name, &variant.report_json),
            summary: binding(summary_name, &variant.summary_json),
            manifest: binding(manifest_name, &variant.manifest_json),
        });
    }

    let texture_name = format!("{TEXTURE_RESREF}.tga");
    let hak_name = format!("{HAK_RESREF}.hak");
    let module_name = format!("{MODULE_RESREF}.mod");
    write_new(&output_directory.join(&texture_name), &texture)?;
    write_new(
        &output_directory.join("appearance.2da"),
        &current_appearance,
    )?;
    write_new(&output_directory.join(&hak_name), &hak.payload)?;
    write_new(&output_directory.join(&module_name), &module.payload)?;

    let handoff = serde_json::json!({
        "schemaVersion": 1,
        "status": "ready_for_owner_proof",
        "purpose": "FOUR_WAY_CREATURE_SOURCE_FORWARD_DIAGNOSTIC_V1",
        "testModuleFileName": module_name,
        "moduleDisplayName": MODULE_DISPLAY_NAME,
        "areaDisplayName": AREA_DISPLAY_NAME,
        "moduleResref": MODULE_RESREF,
        "areaResref": AREA_RESREF,
        "hakResref": HAK_RESREF,
        "source": source_binding,
        "inputAppearanceTwoDa": appearance_binding,
        "module": binding(format!("{MODULE_RESREF}.mod"), &module.payload),
        "hak": binding(format!("{HAK_RESREF}.hak"), &hak.payload),
        "texture": binding(texture_name, &texture),
        "runtimeAppearanceTwoDa": binding("appearance.2da".to_owned(), &current_appearance),
        "hakEntryCount": hak.report.entry_count,
        "moduleReadback": module.readback,
        "variants": variant_bindings,
        "ownerProof": {
            "toolset": { "modelVisibility": "not_tested", "proofCompleteness": "missing" },
            "nwn": { "modelVisibility": "not_tested", "proofCompleteness": "missing" }
        },
        "startsToolset": false,
        "startsNwn": false
    });
    let handoff_bytes = serde_json::to_vec_pretty(&handoff)
        .map_err(|error| format!("CREATURE-FACING-DEMO-HANDOFF-JSON: {error}"))?;
    write_new(&output_directory.join("handoff.json"), &handoff_bytes)?;

    serde_json::to_string_pretty(&handoff)
        .map_err(|error| format!("CREATURE-FACING-DEMO-STDOUT-JSON: {error}"))
}

fn build_variant(
    source: &[u8],
    prepared: PreparedVariant,
) -> Result<(BuiltVariant, Vec<u8>), String> {
    let spec = prepared.spec;
    let identity = ProceduralCreatureProductIdentityV2 {
        model_resref: spec.model_resref.to_owned(),
        texture_resref: TEXTURE_RESREF.to_owned(),
        hak_resref: HAK_RESREF.to_owned(),
        appearance_label: format!("M2A_FACE_{}", spec.token.to_ascii_uppercase()),
    };
    let options = ProceduralCreatureBuildOptionsV1 {
        source_forward: spec.forward,
        ..ProceduralCreatureBuildOptionsV1::default()
    };
    let product = build_meshy_procedural_humanoid_product_with_options_v3(
        source,
        &prepared.input_appearance,
        &identity,
        &options,
    )
    .map_err(|error| format!("CREATURE-FACING-DEMO-{}: {error}", spec.token))?;

    let expected_mapping = creature_source_forward_mapping_v1(spec.forward);
    if product.summary.creature_source_forward != spec.forward
        || product.summary.creature_forward_mapping != expected_mapping
        || product.manifest.creature_source_forward != spec.forward
        || product.manifest.creature_forward_mapping != expected_mapping
        || product.report.conversion.policies.asset_forward_mapping != expected_mapping
        || (product.report.conversion.transform.determinant - 1.0).abs() > 1.0e-6
    {
        return Err(format!(
            "CREATURE-FACING-DEMO-CONTRACT-DIFF: {}",
            spec.token
        ));
    }
    if product.report.appearance.appended_row_index != prepared.appearance_row
        || product.appearance_two_da != prepared.expected_appearance
    {
        return Err(format!(
            "CREATURE-FACING-DEMO-APPEARANCE-DIFF: {}",
            spec.token
        ));
    }

    Ok((
        BuiltVariant {
            spec,
            appearance_row: prepared.appearance_row,
            model: product.model,
            report_json: product.report_json,
            summary_json: product.summary_json,
            manifest_json: product.manifest_json,
            triangle_count: product.report.geometry.triangle_count,
            determinant: product.report.conversion.transform.determinant,
        },
        product.texture,
    ))
}

fn parse_args(
    arguments: impl IntoIterator<Item = String>,
) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let mut source = None;
    let mut appearance = None;
    let mut output = None;
    let mut values = arguments.into_iter();
    while let Some(argument) = values.next() {
        let target = match argument.as_str() {
            "--source-glb" => &mut source,
            "--appearance-2da" => &mut appearance,
            "--out" => &mut output,
            _ => {
                return Err(format!(
                    "CREATURE-FACING-DEMO-ARGUMENT-UNKNOWN: {argument}\n{}",
                    usage()
                ));
            }
        };
        if target.is_some() {
            return Err(format!(
                "CREATURE-FACING-DEMO-ARGUMENT-DUPLICATE: {argument}"
            ));
        }
        *target = values.next().filter(|value| !value.starts_with("--"));
        if target.is_none() {
            return Err(format!("CREATURE-FACING-DEMO-ARGUMENT-MISSING: {argument}"));
        }
    }
    Ok((
        PathBuf::from(source.ok_or_else(usage)?),
        PathBuf::from(appearance.ok_or_else(usage)?),
        PathBuf::from(output.ok_or_else(usage)?),
    ))
}

fn usage() -> String {
    "usage: materialize_creature_facing_matrix --source-glb <canonical.glb> --appearance-2da <full appearance.2da> --out <new-directory>".to_owned()
}

fn read(path: &Path, role: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| {
        format!(
            "CREATURE-FACING-DEMO-{role}-READ {}: {error}",
            path.display()
        )
    })
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("CREATURE-FACING-DEMO-WRITE {}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("CREATURE-FACING-DEMO-WRITE {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("CREATURE-FACING-DEMO-SYNC {}: {error}", path.display()))
}

fn binding(file_name: String, bytes: &[u8]) -> ByteBinding {
    ByteBinding {
        file_name,
        byte_length: bytes.len() as u64,
        sha256: hex_sha256(bytes),
    }
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{VARIANTS, parse_args};

    #[test]
    fn command_requires_exact_inputs_once() {
        let args = [
            "--source-glb",
            "source.glb",
            "--appearance-2da",
            "appearance.2da",
            "--out",
            "output",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        assert!(parse_args(args.clone()).is_ok());
        let mut duplicate = args;
        duplicate.extend(["--out".to_owned(), "other".to_owned()]);
        assert!(parse_args(duplicate).is_err());
    }

    #[test]
    fn diagnostic_owns_four_unique_cardinal_variants() {
        let mut models = VARIANTS
            .iter()
            .map(|variant| variant.model_resref)
            .collect::<Vec<_>>();
        let mut creatures = VARIANTS
            .iter()
            .map(|variant| variant.creature_resref)
            .collect::<Vec<_>>();
        models.sort_unstable();
        creatures.sort_unstable();
        models.dedup();
        creatures.dedup();
        assert_eq!(models.len(), 4);
        assert_eq!(creatures.len(), 4);
    }
}

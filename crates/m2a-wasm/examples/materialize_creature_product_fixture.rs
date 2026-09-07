//! Offline fixture authoring for an already built Creature product. No native UI.
use m2a_core::proof_module::{
    BinaryCreatureModuleIdentityV1, BinaryCreatureProfiledFixtureV2,
    build_binary_creature_profile_matrix_module_named_v3,
};
use serde::Deserialize;
use std::{env, fs, io::Write, path::Path};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    module: BinaryCreatureModuleIdentityV1,
    module_name: String,
    area_name: String,
    fixtures: Vec<BinaryCreatureProfiledFixtureV2>,
}
fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        return Err("usage: product-report.json fixture-request.json output-directory".into());
    }
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&args[1]).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let request: Request = serde_json::from_slice(&fs::read(&args[2]).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if report["admissionV3"]["status"] != "PASS"
        || report["identity"]["hakResref"] != request.module.hak_resref
    {
        return Err("fixture product identity/admission mismatch".into());
    }
    let row = report["appearance"]["appendedRowIndex"]
        .as_u64()
        .ok_or("appearance row missing")?;
    if request.fixtures.is_empty()
        || request
            .fixtures
            .iter()
            .any(|f| u64::from(f.fixture.appearance_row) != row)
    {
        return Err("fixture appearance mismatch".into());
    }
    let artifact = build_binary_creature_profile_matrix_module_named_v3(
        &request.module,
        &request.fixtures,
        &request.module_name,
        &request.area_name,
        "Owned borzoi model with c_wolf inherited animations; awaiting owner visual test.",
    )
    .map_err(|e| e.to_string())?;
    if artifact.readback.fixtures != request.fixtures
        || artifact.readback.scene.ordered_hak_resrefs != [request.module.hak_resref]
    {
        return Err("module readback mismatch".into());
    }
    let out = Path::new(&args[3]);
    let path = out.join(format!("{}.mod", request.module.module_resref));
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    file.write_all(&artifact.payload)
        .map_err(|e| e.to_string())?;
    let meta = serde_json::json!({"moduleFile":path,"module_name":request.module_name,"area_name":request.area_name,"sha256":artifact.sha256,"readback":artifact.readback,"nativeRuntime":"NOT_TESTED"});
    let mut record = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(out.join("module-readback.json"))
        .map_err(|e| e.to_string())?;
    record
        .write_all(&serde_json::to_vec_pretty(&meta).unwrap())
        .map_err(|e| e.to_string())?;
    println!("MODULE_CREATED_AND_READ_BACK");
    Ok(())
}

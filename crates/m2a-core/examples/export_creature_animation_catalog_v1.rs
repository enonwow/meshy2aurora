use m2a_core::creature_animation_mapping::direct_creature_base_catalog_v1;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let states = direct_creature_base_catalog_v1()
        .into_iter()
        .map(|state| {
            json!({
                "stateId": state.state_id,
                "label": state.label,
                "description": state.description,
                "slot": state.slot,
                "gameplayFloor": state.gameplay_floor,
            })
        })
        .collect::<Vec<_>>();
    let contract = json!({
        "schemaVersion": 1,
        "profile": "DIRECT_CREATURE_S_L_BASE_42_CATALOG_V1",
        "supportedModelTypes": ["S", "L"],
        "playbackPolicy": "ENGINE_MANAGED",
        "states": states,
    });
    println!("{}", serde_json::to_string_pretty(&contract)?);
    Ok(())
}

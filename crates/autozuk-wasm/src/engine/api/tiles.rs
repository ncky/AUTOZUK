use serde_json::{json, Value};

use super::input::{get_i32, get_str, get_u32, parse_loadout, parse_pillar_config};
use crate::engine::{
    check_tile_excluded, hl_create_mob, parse_spawn_code, simulate_tile_summary,
    simulate_tile_summary_typed_with_scratch, tile_summary_json, with_cached_region, Point,
    SimScratch, TileOut, TileSimParams,
};

pub(crate) fn exclude_tiles_inner(input_json: &str) -> Result<Value, String> {
    let input: Value = serde_json::from_str(input_json).map_err(|e| e.to_string())?;
    let spawn_code = get_str(&input, "spawnCode")?;
    let pillar_config = parse_pillar_config(input.get("pillarConfig")).unwrap_or_default();
    let parsed = parse_spawn_code(spawn_code)?;
    let mut test_mobs = Vec::new();
    for spawn in parsed.spawns {
        if let Some(mob_type) = spawn.mob_type {
            test_mobs.push(hl_create_mob(mob_type, spawn.x, spawn.y, test_mobs.len()));
        }
    }
    let tiles = input
        .get("tiles")
        .and_then(Value::as_array)
        .ok_or_else(|| "tiles must be an array".to_string())?;
    with_cached_region(pillar_config, |region| {
        let mut excluded = Vec::new();
        let mut eligible = Vec::new();
        for tile in tiles {
            let x = get_i32(tile, "x")?;
            let y = get_i32(tile, "y")?;
            let out = TileOut { x, y };
            if check_tile_excluded(x, y, &test_mobs, region) {
                excluded.push(out);
            } else {
                eligible.push(out);
            }
        }
        Ok(json!({"error": null, "excluded": excluded, "eligible": eligible}))
    })
}

pub(crate) fn simulate_tile_inner(input_json: &str) -> Result<Value, String> {
    let input: Value = serde_json::from_str(input_json).map_err(|e| e.to_string())?;
    let spawn_code = get_str(&input, "spawnCode")?;
    let pillar_config = parse_pillar_config(input.get("pillarConfig")).unwrap_or_default();
    let tile_value = input
        .get("tile")
        .ok_or_else(|| "tile is required".to_string())?;
    let tile = Point {
        x: get_i32(tile_value, "x")?,
        y: get_i32(tile_value, "y")?,
    };
    let loadout = parse_loadout(
        input
            .get("loadout")
            .ok_or_else(|| "loadout is required".to_string())?,
    )?;
    let max_ticks = get_i32(&input, "maxTicks")?;
    let max_sims = get_i32(&input, "maxSims")?.max(0);
    let seed_base = get_u32(&input, "seedBase")?;
    let quick_prune = input
        .get("quickPrune")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let parsed = parse_spawn_code(spawn_code)?;
    with_cached_region(pillar_config, |region| {
        let summary = simulate_tile_summary(TileSimParams {
            parsed: &parsed,
            pillar_config,
            tile,
            loadout: &loadout,
            max_ticks,
            max_sims,
            seed_base,
            region,
            quick_prune,
        })?;
        Ok(json!({"error": null, "summary": summary}))
    })
}

pub(crate) fn simulate_tiles_inner(input_json: &str) -> Result<Value, String> {
    let input: Value = serde_json::from_str(input_json).map_err(|e| e.to_string())?;
    let spawn_code = get_str(&input, "spawnCode")?;
    let pillar_config = parse_pillar_config(input.get("pillarConfig")).unwrap_or_default();
    let loadout = parse_loadout(
        input
            .get("loadout")
            .ok_or_else(|| "loadout is required".to_string())?,
    )?;
    let max_ticks = get_i32(&input, "maxTicks")?;
    let max_sims = get_i32(&input, "maxSims")?.max(0);
    let seed_base = get_u32(&input, "seedBase")?;
    let quick_prune = input
        .get("quickPrune")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let tiles = input
        .get("tiles")
        .and_then(Value::as_array)
        .ok_or_else(|| "tiles must be an array".to_string())?;
    let parsed = parse_spawn_code(spawn_code)?;
    with_cached_region(pillar_config, |region| {
        let mut results = Vec::with_capacity(tiles.len());
        let mut scratch = SimScratch::with_capacity(max_sims as usize);
        for tile_value in tiles {
            let tile = Point {
                x: get_i32(tile_value, "x")?,
                y: get_i32(tile_value, "y")?,
            };
            let summary = match simulate_tile_summary_typed_with_scratch(
                TileSimParams {
                    parsed: &parsed,
                    pillar_config,
                    tile,
                    loadout: &loadout,
                    max_ticks,
                    max_sims,
                    seed_base,
                    region,
                    quick_prune,
                },
                &mut scratch,
            )? {
                Some(summary) => tile_summary_json(&summary),
                None => Value::Null,
            };
            results.push(json!({"tile": {"x": tile.x, "y": tile.y}, "summary": summary}));
        }
        Ok(json!({"error": null, "results": results}))
    })
}

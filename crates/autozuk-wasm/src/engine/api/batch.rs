use std::fmt::Write as _;

use serde_json::{json, Value};

use super::input::{get_i32, get_str, get_u32, parse_loadout, parse_pillar_config};
use crate::engine::{
    insert_ranked_tile_summary, parse_spawn_code, push_ranked_tile_summary_json,
    push_tile_summary_json, simulate_tile_summary_typed_with_scratch, with_cached_region, Point,
    RankedTileSummary, SimContext, SimScratch, TileSimParams, SIM_CONTEXT,
};

pub(crate) fn configure_sim_context_inner(input_json: &str) -> Result<Value, String> {
    let input: Value = serde_json::from_str(input_json).map_err(|e| e.to_string())?;
    let spawn_code = get_str(&input, "spawnCode")?;
    let pillar_config = parse_pillar_config(input.get("pillarConfig")).unwrap_or_default();
    let loadout = parse_loadout(
        input
            .get("loadout")
            .ok_or_else(|| "loadout is required".to_string())?,
    )?;
    let max_ticks = get_i32(&input, "maxTicks")?;
    let seed_base = get_u32(&input, "seedBase")?;
    let parsed = parse_spawn_code(spawn_code)?;
    SIM_CONTEXT.with(|cell| {
        *cell.borrow_mut() = Some(SimContext {
            parsed,
            pillar_config,
            loadout,
            max_ticks,
            seed_base,
            scratch: SimScratch::with_capacity(0),
        });
    });
    Ok(json!({"error": null}))
}

pub(crate) fn simulate_tiles_cached_inner(
    tile_coords: &[u8],
    max_sims: i32,
    quick_prune: bool,
) -> Result<String, String> {
    if !tile_coords.len().is_multiple_of(2) {
        return Err("tile coordinate buffer must contain x,y pairs".to_string());
    }
    SIM_CONTEXT.with(|cell| {
        let mut borrowed = cell.borrow_mut();
        let ctx = borrowed
            .as_mut()
            .ok_or_else(|| "simulation context is not configured".to_string())?;
        let SimContext {
            parsed,
            pillar_config,
            loadout,
            max_ticks,
            seed_base,
            scratch,
        } = ctx;
        with_cached_region(*pillar_config, |region| {
            let mut out = String::with_capacity(tile_coords.len() / 2 * 360 + 32);
            out.push_str("{\"error\":null,\"results\":[");
            scratch.ensure_capacity(max_sims.max(0) as usize);
            for (idx, pair) in tile_coords.chunks_exact(2).enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                let tile = Point {
                    x: pair[0] as i32,
                    y: pair[1] as i32,
                };
                let _ = write!(
                    out,
                    "{{\"tile\":{{\"x\":{},\"y\":{}}},\"summary\":",
                    tile.x, tile.y
                );
                match simulate_tile_summary_typed_with_scratch(
                    TileSimParams {
                        parsed,
                        pillar_config: *pillar_config,
                        tile,
                        loadout,
                        max_ticks: *max_ticks,
                        max_sims: max_sims.max(0),
                        seed_base: *seed_base,
                        region,
                        quick_prune,
                    },
                    scratch,
                )? {
                    Some(summary) => push_tile_summary_json(&mut out, &summary),
                    None => out.push_str("null"),
                }
                out.push('}');
            }
            out.push_str("]}");
            Ok(out)
        })
    })
}

pub(crate) fn simulate_tiles_top_cached_inner(
    tile_coords: &[u8],
    max_sims: i32,
    quick_prune: bool,
    limit: usize,
) -> Result<String, String> {
    if !tile_coords.len().is_multiple_of(2) {
        return Err("tile coordinate buffer must contain x,y pairs".to_string());
    }
    SIM_CONTEXT.with(|cell| {
        let mut borrowed = cell.borrow_mut();
        let ctx = borrowed
            .as_mut()
            .ok_or_else(|| "simulation context is not configured".to_string())?;
        let SimContext {
            parsed,
            pillar_config,
            loadout,
            max_ticks,
            seed_base,
            scratch,
        } = ctx;
        with_cached_region(*pillar_config, |region| {
            let tile_count = tile_coords.len() / 2;
            let limit = limit.min(tile_count);
            let mut top: Vec<RankedTileSummary> = Vec::with_capacity(limit);
            scratch.ensure_capacity(max_sims.max(0) as usize);
            for pair in tile_coords.chunks_exact(2) {
                let tile = Point {
                    x: pair[0] as i32,
                    y: pair[1] as i32,
                };
                let Some(summary) = simulate_tile_summary_typed_with_scratch(
                    TileSimParams {
                        parsed,
                        pillar_config: *pillar_config,
                        tile,
                        loadout,
                        max_ticks: *max_ticks,
                        max_sims: max_sims.max(0),
                        seed_base: *seed_base,
                        region,
                        quick_prune,
                    },
                    scratch,
                )?
                else {
                    continue;
                };
                insert_ranked_tile_summary(&mut top, limit, RankedTileSummary { tile, summary });
            }

            let mut out = String::with_capacity(top.len() * 360 + 32);
            out.push_str("{\"error\":null,\"results\":[");
            for (idx, item) in top.iter().enumerate() {
                if idx > 0 {
                    out.push(',');
                }
                push_ranked_tile_summary_json(&mut out, item);
            }
            out.push_str("]}");
            Ok(out)
        })
    })
}

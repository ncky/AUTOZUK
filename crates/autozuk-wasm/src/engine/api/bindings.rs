use serde_json::json;
use wasm_bindgen::prelude::*;

use super::batch::{
    configure_sim_context_inner, simulate_tiles_cached_inner, simulate_tiles_top_cached_inner,
};
use super::tiles::{exclude_tiles_inner, simulate_tile_inner, simulate_tiles_inner};

#[wasm_bindgen]
pub fn exclude_tiles(input_json: &str) -> String {
    match exclude_tiles_inner(input_json) {
        Ok(value) => value.to_string(),
        Err(error) => json!({"error": error, "excluded": [], "eligible": []}).to_string(),
    }
}

#[wasm_bindgen]
pub fn simulate_tile(input_json: &str) -> String {
    match simulate_tile_inner(input_json) {
        Ok(value) => value.to_string(),
        Err(error) => json!({"error": error, "summary": null}).to_string(),
    }
}

#[wasm_bindgen]
pub fn simulate_tiles(input_json: &str) -> String {
    match simulate_tiles_inner(input_json) {
        Ok(value) => value.to_string(),
        Err(error) => json!({"error": error, "results": []}).to_string(),
    }
}

#[wasm_bindgen]
pub fn configure_sim_context(input_json: &str) -> String {
    match configure_sim_context_inner(input_json) {
        Ok(value) => value.to_string(),
        Err(error) => json!({"error": error}).to_string(),
    }
}

#[wasm_bindgen]
pub fn simulate_tiles_cached(tile_coords: &[u8], max_sims: i32, quick_prune: bool) -> String {
    match simulate_tiles_cached_inner(tile_coords, max_sims, quick_prune) {
        Ok(value) => value,
        Err(error) => json!({"error": error, "results": []}).to_string(),
    }
}

#[wasm_bindgen]
pub fn simulate_tiles_top_cached(
    tile_coords: &[u8],
    max_sims: i32,
    quick_prune: bool,
    limit: usize,
) -> String {
    match simulate_tiles_top_cached_inner(tile_coords, max_sims, quick_prune, limit) {
        Ok(value) => value,
        Err(error) => json!({"error": error, "results": []}).to_string(),
    }
}

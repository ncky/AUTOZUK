mod constants;
pub(crate) use constants::{
    ARENA_CELLS, ARENA_W, ARENA_X_MAX, ARENA_X_MIN, ARENA_Y_MAX, ARENA_Y_MIN, BFS_DIRS,
    BLOOD_BARRAGE_AOE_OFFSETS, DAMAGE_BUCKETS, DEATH_ANIM_TICKS, LOS_WORDS_PER_ROW, MAX_MOB_IDS,
    MOB_TYPE_COUNT, NPC_LOS_TABLES, PLAYER_LOS_TABLES, SPAWN_LOCATIONS,
};
mod globals;
pub(crate) use globals::{REGION_CACHE, SIM_CONTEXT};
mod model;
pub(crate) use model::{
    event_i16, event_id, event_mob_type, event_style, AttackEvent, AttackStats,
    DelayedBlobletSpawn, Entity, Loadout, Mob, MobInit, MobProjectile, MobType, MonsterAttackTable,
    Mulberry32, ParsedSpawnCode, PillarConfig, Player, PlayerProjectile, Point, Prayer, Region,
    SimContext, SimResult, SimStatus, Spawn, State, Style, TileOut, WaveFlags,
};
mod mechanics;
pub(crate) use mechanics::{
    arena_idx, arena_point, chebyshev, closest_tile_to, collides_with_entities, collides_with_mobs,
    collision_math, dist_to_mob, key, mob_atk_speed, mob_has_dig, mob_has_flicker, mob_hp,
    mob_range, mob_size, mob_style, monster_projectile_delay, move_mob_to, player_projectile_delay,
    rebuild_mob_occupancy, sign, state_point_collides_with_mobs, update_mob_occupancy,
};
mod region;
pub(crate) use region::{
    can_use_secondary_melee, get_closest_face_tile, has_line_of_sight, is_within_melee_range,
    mob_has_los, npc_los_table_idx, osrs_walk_step, player_has_los, player_los_table_idx, raycast,
    with_cached_region,
};
mod spawn;
pub(crate) use spawn::{parse_spawn_code, wave_flags};
mod simulation;
pub(crate) use simulation::{
    aggro_is_unusable, find_respawn_location, has_dead_mob_for_revive, hl_create_mob,
    hl_init_state, hl_mob_attack, hl_move_mob, hl_player_attack_if_ready, hl_process_corpse_expiry,
    hl_process_delayed_bloblet_spawns, hl_process_pending_mob_deaths, hl_run_sim, hl_tick,
    is_under_mob, mob_idx_by_id, pop_dead_mob_for_revive, set_id_to_idx, set_mob_init,
    set_player_last_attacker, start_dig, SimRunParams,
};
mod scoring;
pub(crate) use scoring::{
    attack_style_idx, calc_sim_damage, check_tile_excluded, damage_bucket,
    insert_ranked_tile_summary, is_empty_wave, optimize_prayer, prayer_slot,
    push_ranked_tile_summary_json, push_tile_summary_json, simulate_tile_summary,
    simulate_tile_summary_typed_with_scratch, summarize_empty_wave_typed,
    summarize_no_attack_results_typed, tile_summary_json, DamageResult, MobProjectileQueue,
    PendingMobRemoval, PendingMobRemovalQueue, PendingPlayerHit, PendingPlayerHitQueue,
    PendingRecoil, PendingRecoilQueue, PlayerProjectileQueue, RankedTileSummary, SimScratch,
    TileSimParams, TileSummary,
};
mod api;

pub use api::{
    configure_sim_context, exclude_tiles, simulate_tile, simulate_tiles, simulate_tiles_cached,
    simulate_tiles_top_cached,
};

#[cfg(test)]
mod tests;

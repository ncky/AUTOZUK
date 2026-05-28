use serde_json::Value;

use crate::engine::{
    damage_bucket, hl_run_sim, is_empty_wave, optimize_prayer, summarize_empty_wave_typed,
    summarize_no_attack_results_typed, tile_summary_json, wave_flags, AttackEvent, DamageResult,
    Loadout, ParsedSpawnCode, PillarConfig, Point, Prayer, Region, SimResult, SimRunParams,
    SimStatus, DAMAGE_BUCKETS,
};

#[derive(Clone, Copy)]
pub(crate) struct TileSimParams<'a> {
    pub(crate) parsed: &'a ParsedSpawnCode,
    pub(crate) pillar_config: PillarConfig,
    pub(crate) tile: Point,
    pub(crate) loadout: &'a Loadout,
    pub(crate) max_ticks: i32,
    pub(crate) max_sims: i32,
    pub(crate) seed_base: u32,
    pub(crate) region: &'a Region,
    pub(crate) quick_prune: bool,
}

#[derive(Clone)]
pub(crate) struct TileSummary {
    pub(crate) avg_damage: f64,
    pub(crate) damage_buckets: [usize; DAMAGE_BUCKETS],
    pub(crate) damage_max: i32,
    pub(crate) over50_pct: f64,
    pub(crate) avg_ticks: f64,
    pub(crate) prayer: [Prayer; 4],
    pub(crate) invalid_pct: f64,
    pub(crate) total_sims: usize,
    pub(crate) death_pct: f64,
    pub(crate) marked_dead: bool,
}

pub(crate) struct RankedTileSummary {
    pub(crate) tile: Point,
    pub(crate) summary: TileSummary,
}

pub(crate) struct SimScratch {
    pub(crate) all_results: Vec<SimResult>,
    pub(crate) attack_log: Vec<AttackEvent>,
    pub(crate) best_damages: Vec<DamageResult>,
    pub(crate) candidate_damages: Vec<DamageResult>,
}

impl SimScratch {
    pub(crate) fn with_capacity(sim_cap: usize) -> Self {
        Self {
            all_results: Vec::with_capacity(sim_cap),
            attack_log: Vec::with_capacity(sim_cap.min(32) * 192),
            best_damages: Vec::with_capacity(sim_cap),
            candidate_damages: Vec::with_capacity(sim_cap),
        }
    }

    pub(crate) fn ensure_capacity(&mut self, sim_cap: usize) {
        let result_extra = sim_cap.saturating_sub(self.all_results.capacity());
        if result_extra > 0 {
            self.all_results.reserve_exact(result_extra);
        }
        let attack_cap = sim_cap.min(32) * 192;
        let attack_extra = attack_cap.saturating_sub(self.attack_log.capacity());
        if attack_extra > 0 {
            self.attack_log.reserve_exact(attack_extra);
        }
        let best_extra = sim_cap.saturating_sub(self.best_damages.capacity());
        if best_extra > 0 {
            self.best_damages.reserve_exact(best_extra);
        }
        let candidate_extra = sim_cap.saturating_sub(self.candidate_damages.capacity());
        if candidate_extra > 0 {
            self.candidate_damages.reserve_exact(candidate_extra);
        }
    }
}

pub(crate) fn simulate_tile_summary(params: TileSimParams<'_>) -> Result<Value, String> {
    Ok(match simulate_tile_summary_typed(params)? {
        Some(summary) => tile_summary_json(&summary),
        None => Value::Null,
    })
}

pub(crate) fn simulate_tile_summary_typed(
    params: TileSimParams<'_>,
) -> Result<Option<TileSummary>, String> {
    let mut scratch = SimScratch::with_capacity(params.max_sims.max(0) as usize);
    simulate_tile_summary_typed_with_scratch(params, &mut scratch)
}

pub(crate) fn simulate_tile_summary_typed_with_scratch(
    params: TileSimParams<'_>,
    scratch: &mut SimScratch,
) -> Result<Option<TileSummary>, String> {
    if params.max_sims <= 0 {
        return Ok(None);
    }
    if is_empty_wave(params.parsed, params.pillar_config) {
        return Ok(Some(summarize_empty_wave_typed(params.max_sims as usize)));
    }

    let flags = wave_flags(params.parsed);
    scratch.ensure_capacity(params.max_sims as usize);
    scratch.all_results.clear();
    scratch.attack_log.clear();
    scratch.best_damages.clear();
    scratch.candidate_damages.clear();
    for s in 0..params.max_sims {
        let seed = params.seed_base
            ^ ((params.tile.x as u32).wrapping_mul(73_856_093))
            ^ ((params.tile.y as u32).wrapping_mul(19_349_663))
            ^ ((s as u32).wrapping_mul(83_492_791));
        let result = hl_run_sim(
            SimRunParams {
                parsed: params.parsed,
                player_pos: params.tile,
                pillar_config: params.pillar_config,
                loadout: params.loadout,
                max_ticks: params.max_ticks,
                region: params.region,
                seed,
            },
            &mut scratch.attack_log,
        )?;
        scratch.all_results.push(result);
        if params.quick_prune
            && s == 2
            && scratch.all_results.len() >= 3
            && !scratch.attack_log.is_empty()
        {
            optimize_prayer(
                &scratch.all_results,
                &scratch.attack_log,
                flags,
                params.loadout,
                &mut scratch.best_damages,
                &mut scratch.candidate_damages,
            );
            let all_dead = scratch.best_damages.iter().all(|damage| damage.died);
            if all_dead {
                break;
            }
        }
        if params.quick_prune
            && s == 9
            && scratch.all_results.len() >= 10
            && !scratch.attack_log.is_empty()
        {
            optimize_prayer(
                &scratch.all_results,
                &scratch.attack_log,
                flags,
                params.loadout,
                &mut scratch.best_damages,
                &mut scratch.candidate_damages,
            );
            let mut total = 0;
            for damage in &scratch.best_damages {
                total += damage.damage;
            }
            let quick_avg = total as f64 / scratch.all_results.len() as f64;
            if quick_avg > 80.0 {
                break;
            }
        }
    }

    if scratch.all_results.is_empty() {
        return Ok(None);
    }

    if scratch.attack_log.is_empty() {
        return Ok(Some(summarize_no_attack_results_typed(
            &scratch.all_results,
            params.max_ticks,
        )));
    }

    let prayer = optimize_prayer(
        &scratch.all_results,
        &scratch.attack_log,
        flags,
        params.loadout,
        &mut scratch.best_damages,
        &mut scratch.candidate_damages,
    );
    let mut invalid_count = 0;
    let mut death_count = 0;
    let mut damage_count = 0usize;
    let mut damage_sum = 0i32;
    let mut over50 = 0usize;
    let mut max_damage = 100i32;
    let mut completion_tick_sum = 0i32;
    let mut completion_tick_count = 0usize;
    for (idx, r) in scratch.all_results.iter().enumerate() {
        if r.status == SimStatus::Invalid {
            invalid_count += 1;
            continue;
        }
        let res = scratch.best_damages[idx];
        if res.died {
            death_count += 1;
        }
        damage_count += 1;
        damage_sum += res.damage;
        if res.damage > 50 {
            over50 += 1;
        }
        max_damage = max_damage.max(res.damage);
        completion_tick_sum += r.completed_tick;
        completion_tick_count += 1;
    }
    let mut damage_buckets = [0usize; DAMAGE_BUCKETS];
    for (idx, r) in scratch.all_results.iter().enumerate() {
        if r.status == SimStatus::Invalid {
            continue;
        }
        let bucket = damage_bucket(scratch.best_damages[idx].damage, max_damage);
        damage_buckets[bucket] += 1;
    }
    let death_pct = if damage_count == 0 {
        0.0
    } else {
        death_count as f64 / damage_count as f64 * 100.0
    };
    let avg_dmg = if damage_count == 0 {
        999.0
    } else {
        damage_sum as f64 / damage_count as f64
    };
    let avg_ticks = if completion_tick_count == 0 {
        params.max_ticks as f64
    } else {
        completion_tick_sum as f64 / completion_tick_count as f64
    };
    let invalid_pct = invalid_count as f64 / scratch.all_results.len() as f64 * 100.0;
    Ok(Some(TileSummary {
        avg_damage: avg_dmg,
        damage_buckets,
        damage_max: max_damage,
        over50_pct: if damage_count == 0 {
            100.0
        } else {
            over50 as f64 / damage_count as f64 * 100.0
        },
        avg_ticks,
        prayer,
        invalid_pct,
        total_sims: scratch.all_results.len(),
        death_pct,
        marked_dead: death_pct > 30.0 || invalid_pct > 20.0,
    }))
}

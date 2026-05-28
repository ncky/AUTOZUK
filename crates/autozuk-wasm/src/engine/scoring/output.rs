use std::fmt::Write as _;

use serde_json::{json, Value};

use crate::engine::{Prayer, RankedTileSummary, SimResult, SimStatus, TileSummary, DAMAGE_BUCKETS};

pub(crate) fn tile_summary_json(summary: &TileSummary) -> Value {
    let prayer_json = summary.prayer.map(|p| p.as_str());
    json!({
        "avgDamage": summary.avg_damage,
        "damageBuckets": summary.damage_buckets,
        "damageMax": summary.damage_max,
        "over50Pct": summary.over50_pct,
        "avgTicks": summary.avg_ticks,
        "avgTime": format!("{:.1}", summary.avg_ticks * 0.6),
        "prayer": prayer_json,
        "invalidPct": summary.invalid_pct,
        "totalSims": summary.total_sims,
        "deathPct": summary.death_pct,
        "markedDead": summary.marked_dead
    })
}

pub(crate) fn push_tile_summary_json(out: &mut String, summary: &TileSummary) {
    let _ = write!(
        out,
        "{{\"avgDamage\":{},\"damageBuckets\":[",
        summary.avg_damage
    );
    for (idx, bucket) in summary.damage_buckets.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        let _ = write!(out, "{bucket}");
    }
    let _ = write!(
        out,
        "],\"damageMax\":{},\"over50Pct\":{},\"avgTicks\":{},\"avgTime\":\"{:.1}\",\"prayer\":[\"{}\",\"{}\",\"{}\",\"{}\"],\"invalidPct\":{},\"totalSims\":{},\"deathPct\":{},\"markedDead\":{}",
        summary.damage_max,
        summary.over50_pct,
        summary.avg_ticks,
        summary.avg_ticks * 0.6,
        summary.prayer[0].as_str(),
        summary.prayer[1].as_str(),
        summary.prayer[2].as_str(),
        summary.prayer[3].as_str(),
        summary.invalid_pct,
        summary.total_sims,
        summary.death_pct,
        summary.marked_dead
    );
    out.push('}');
}

pub(crate) fn push_ranked_tile_summary_json(out: &mut String, item: &RankedTileSummary) {
    let _ = write!(
        out,
        "{{\"tile\":{{\"x\":{},\"y\":{}}},\"summary\":",
        item.tile.x, item.tile.y
    );
    push_tile_summary_json(out, &item.summary);
    out.push('}');
}

pub(crate) fn summarize_empty_wave_typed(sample_count: usize) -> TileSummary {
    let mut damage_buckets = [0usize; DAMAGE_BUCKETS];
    damage_buckets[0] = sample_count;
    TileSummary {
        avg_damage: 0.0,
        damage_buckets,
        damage_max: 100,
        over50_pct: 0.0,
        avg_ticks: 1.0,
        prayer: [Prayer::Mage; 4],
        invalid_pct: 0.0,
        total_sims: sample_count,
        death_pct: 0.0,
        marked_dead: false,
    }
}

pub(crate) fn summarize_no_attack_results_typed(
    results: &[SimResult],
    max_ticks: i32,
) -> TileSummary {
    let mut invalid_count = 0usize;
    let mut valid_count = 0usize;
    let mut completion_tick_sum = 0i32;
    for result in results {
        if result.status == SimStatus::Invalid {
            invalid_count += 1;
            continue;
        }
        valid_count += 1;
        completion_tick_sum += result.completed_tick;
    }
    let avg_ticks = if valid_count == 0 {
        max_ticks as f64
    } else {
        completion_tick_sum as f64 / valid_count as f64
    };
    let invalid_pct = invalid_count as f64 / results.len() as f64 * 100.0;
    let mut damage_buckets = [0usize; DAMAGE_BUCKETS];
    damage_buckets[0] = valid_count;
    TileSummary {
        avg_damage: if valid_count == 0 { 999.0 } else { 0.0 },
        damage_buckets,
        damage_max: 100,
        over50_pct: if results.len() == invalid_count {
            100.0
        } else {
            0.0
        },
        avg_ticks,
        prayer: [Prayer::Mage; 4],
        invalid_pct,
        total_sims: results.len(),
        death_pct: 0.0,
        marked_dead: invalid_pct > 20.0,
    }
}

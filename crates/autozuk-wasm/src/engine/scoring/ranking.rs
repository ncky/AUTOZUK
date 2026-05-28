use std::cmp::Ordering;

use crate::engine::{
    ParsedSpawnCode, PillarConfig, Point, RankedTileSummary, TileSummary, DAMAGE_BUCKETS,
};

pub(crate) fn damage_bucket(damage: i32, max_damage: i32) -> usize {
    let max_damage = max_damage.max(1);
    (((damage as f64 / max_damage as f64) * DAMAGE_BUCKETS as f64).floor() as usize)
        .min(DAMAGE_BUCKETS - 1)
}

pub(crate) fn compare_tile_summary(
    a_tile: Point,
    a_summary: &TileSummary,
    b_tile: Point,
    b_summary: &TileSummary,
) -> Ordering {
    a_summary
        .avg_damage
        .partial_cmp(&b_summary.avg_damage)
        .unwrap_or(Ordering::Equal)
        .then_with(|| {
            a_summary
                .avg_ticks
                .partial_cmp(&b_summary.avg_ticks)
                .unwrap_or(Ordering::Equal)
        })
        .then_with(|| a_tile.x.cmp(&b_tile.x))
        .then_with(|| a_tile.y.cmp(&b_tile.y))
}

pub(crate) fn insert_ranked_tile_summary(
    top: &mut Vec<RankedTileSummary>,
    limit: usize,
    item: RankedTileSummary,
) {
    if limit == 0 {
        return;
    }
    let mut insert_at = top.len();
    while insert_at > 0
        && compare_tile_summary(
            item.tile,
            &item.summary,
            top[insert_at - 1].tile,
            &top[insert_at - 1].summary,
        ) == Ordering::Less
    {
        insert_at -= 1;
    }
    if insert_at >= limit {
        return;
    }
    top.insert(insert_at, item);
    if top.len() > limit {
        top.pop();
    }
}

pub(crate) fn is_empty_wave(parsed: &ParsedSpawnCode, pillar_config: PillarConfig) -> bool {
    parsed.spawns.iter().all(|spawn| spawn.mob_type.is_none())
        && (pillar_config.s || pillar_config.w || pillar_config.n)
}

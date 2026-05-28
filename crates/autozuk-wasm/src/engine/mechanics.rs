use crate::engine::{
    Entity, Loadout, Mob, MobType, Player, Point, State, Style, ARENA_CELLS, ARENA_W, ARENA_X_MAX,
    ARENA_X_MIN, ARENA_Y_MAX, ARENA_Y_MIN,
};

pub(crate) fn mob_size(t: MobType) -> i32 {
    match t {
        MobType::Mager => 4,
        MobType::Ranger => 3,
        MobType::Meleer => 4,
        MobType::Blob => 3,
        MobType::Bat => 2,
        MobType::Nibbler => 1,
        MobType::BlobletMage | MobType::BlobletRange | MobType::BlobletMelee => 1,
    }
}

pub(crate) fn mob_hp(t: MobType) -> i32 {
    match t {
        MobType::Mager => 220,
        MobType::Ranger => 125,
        MobType::Meleer => 75,
        MobType::Blob => 40,
        MobType::Bat => 25,
        MobType::Nibbler => 10,
        MobType::BlobletMage | MobType::BlobletRange | MobType::BlobletMelee => 15,
    }
}

pub(crate) fn mob_atk_speed(t: MobType) -> i32 {
    match t {
        MobType::Blob | MobType::Bat => 3,
        _ => 4,
    }
}

pub(crate) fn mob_range(t: MobType) -> i32 {
    match t {
        MobType::Meleer | MobType::Nibbler | MobType::BlobletMelee => 1,
        MobType::Bat => 4,
        _ => 15,
    }
}

pub(crate) fn mob_style(t: MobType) -> Style {
    match t {
        MobType::Mager | MobType::BlobletMage => Style::Magic,
        MobType::Ranger | MobType::Bat | MobType::BlobletRange => Style::Range,
        MobType::Meleer | MobType::Nibbler | MobType::BlobletMelee => Style::Melee,
        MobType::Blob => Style::Blob,
    }
}

pub(crate) fn mob_has_dig(t: MobType) -> bool {
    t == MobType::Meleer
}

pub(crate) fn mob_has_flicker(t: MobType) -> bool {
    t == MobType::Mager
}

pub(crate) fn collision_math(x: i32, y: i32, s: i32, x2: i32, y2: i32, s2: i32) -> bool {
    !(x > x2 + s2 - 1 || x + s - 1 < x2 || y - s + 1 > y2 || y < y2 - s2 + 1)
}

pub(crate) fn chebyshev(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (x2 - x1).abs().max((y2 - y1).abs())
}

pub(crate) fn key(x: i32, y: i32) -> usize {
    ((x << 6) | y) as usize
}

pub(crate) fn arena_idx(x: i32, y: i32) -> Option<usize> {
    if !(ARENA_X_MIN..=ARENA_X_MAX).contains(&x) || !(ARENA_Y_MIN..=ARENA_Y_MAX).contains(&y) {
        return None;
    }
    Some(arena_idx_unchecked(x, y))
}

pub(crate) fn arena_idx_unchecked(x: i32, y: i32) -> usize {
    ((y - ARENA_Y_MIN) as usize) * ARENA_W + (x - ARENA_X_MIN) as usize
}

pub(crate) fn arena_point(idx: usize) -> Point {
    Point {
        x: ARENA_X_MIN + (idx % ARENA_W) as i32,
        y: ARENA_Y_MIN + (idx / ARENA_W) as i32,
    }
}

pub(crate) fn sign(v: i32) -> i32 {
    if v > 0 {
        1
    } else if v < 0 {
        -1
    } else {
        0
    }
}

pub(crate) fn closest_tile_to(mob: &Mob, tx: i32, ty: i32) -> Point {
    Point {
        x: mob.x.max(tx.min(mob.x + mob.size - 1)),
        y: (mob.y - mob.size + 1).max(ty.min(mob.y)),
    }
}

pub(crate) fn dist_to_mob(px: i32, py: i32, mob: &Mob) -> i32 {
    let ct = closest_tile_to(mob, px, py);
    chebyshev(px, py, ct.x, ct.y)
}

pub(crate) fn collides_with_entities(x: i32, y: i32, s: i32, entities: &[Entity]) -> bool {
    entities
        .iter()
        .any(|e| collision_math(x, y, s, e.x, e.y, e.size))
}

pub(crate) fn collides_with_mobs(
    x: i32,
    y: i32,
    s: i32,
    mobs: &[Mob],
    exclude_idx: Option<usize>,
    skip_nibblers: bool,
) -> Option<usize> {
    let exclude_parent = exclude_idx.and_then(|idx| mobs.get(idx).and_then(|m| m.parent_blob_id));
    for (idx, m) in mobs.iter().enumerate() {
        if Some(idx) == exclude_idx || m.dead {
            continue;
        }
        if exclude_parent == Some(m.id) && m.dying > 0 {
            continue;
        }
        if skip_nibblers && m.mob_type == MobType::Nibbler {
            continue;
        }
        let collides = if s == 1 {
            x >= m.x && x < m.x + m.size && y <= m.y && y > m.y - m.size
        } else {
            collision_math(x, y, s, m.x, m.y, m.size)
        };
        if collides {
            return Some(idx);
        }
    }
    None
}

pub(crate) fn mob_occupancy_mask(idx: usize) -> u64 {
    if idx < 64 {
        1u64 << idx
    } else {
        0
    }
}

pub(crate) fn update_mob_occupancy(
    occupancy: &mut [u64; ARENA_CELLS],
    idx: usize,
    mob: &Mob,
    add: bool,
) {
    if mob.dead {
        return;
    }
    let mask = mob_occupancy_mask(idx);
    if mask == 0 {
        return;
    }
    for ox in 0..mob.size {
        for oy in 0..mob.size {
            if let Some(cell) = arena_idx(mob.x + ox, mob.y - oy) {
                if add {
                    occupancy[cell] |= mask;
                } else {
                    occupancy[cell] &= !mask;
                }
            }
        }
    }
}

pub(crate) fn rebuild_mob_occupancy(mobs: &[Mob], occupancy: &mut [u64; ARENA_CELLS]) {
    occupancy.fill(0);
    for (idx, mob) in mobs.iter().enumerate() {
        update_mob_occupancy(occupancy, idx, mob, true);
    }
}

pub(crate) fn move_mob_to(state: &mut State<'_>, idx: usize, x: i32, y: i32) {
    if state.mobs[idx].x == x && state.mobs[idx].y == y {
        return;
    }
    update_mob_occupancy(&mut state.mob_occupancy, idx, &state.mobs[idx], false);
    state.mobs[idx].x = x;
    state.mobs[idx].y = y;
    state.mobs[idx].los_checked_tick = -1;
    update_mob_occupancy(&mut state.mob_occupancy, idx, &state.mobs[idx], true);
}

pub(crate) fn state_point_collides_with_mobs(
    x: i32,
    y: i32,
    state: &State<'_>,
    exclude_idx: Option<usize>,
    skip_nibblers: bool,
) -> Option<usize> {
    let cell = arena_idx(x, y)?;
    let mut bits = state.mob_occupancy[cell];
    if let Some(idx) = exclude_idx {
        bits &= !mob_occupancy_mask(idx);
    }
    let exclude_parent =
        exclude_idx.and_then(|idx| state.mobs.get(idx).and_then(|m| m.parent_blob_id));
    while bits != 0 {
        let idx = bits.trailing_zeros() as usize;
        bits &= bits - 1;

        let Some(m) = state.mobs.get(idx) else {
            continue;
        };
        if m.dead {
            continue;
        }
        if exclude_parent == Some(m.id) && m.dying > 0 {
            continue;
        }
        if skip_nibblers && m.mob_type == MobType::Nibbler {
            continue;
        }
        return Some(idx);
    }
    None
}

pub(crate) fn delay_from_hit_tick_list(list: &[i32], dist: i32) -> i32 {
    let d = dist.max(1);
    let hit_tick = list[(d as usize).min(list.len()) - 1];
    hit_tick - 1
}

pub(crate) fn monster_projectile_origin(mob: &Mob) -> Point {
    match mob.mob_type {
        MobType::Mager => Point {
            x: mob.x + 2,
            y: mob.y - 2,
        },
        MobType::Bat => Point { x: mob.x, y: mob.y },
        MobType::Ranger | MobType::Blob => Point {
            x: mob.x + 1,
            y: mob.y - 1,
        },
        _ => Point { x: mob.x, y: mob.y },
    }
}

pub(crate) fn monster_projectile_distance(px: i32, py: i32, mob: &Mob) -> i32 {
    let origin = monster_projectile_origin(mob);
    chebyshev(px, py, origin.x, origin.y)
}

pub(crate) fn monster_projectile_delay(mob: &Mob, style: Style, player: &Player) -> i32 {
    if style == Style::Melee {
        return 1;
    }
    let origin_dist = monster_projectile_distance(player.x, player.y, mob);
    match mob.mob_type {
        MobType::Mager => delay_from_hit_tick_list(
            &[2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6],
            origin_dist,
        ),
        MobType::Ranger => delay_from_hit_tick_list(
            &[3, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 6, 6, 6, 6],
            origin_dist,
        ),
        MobType::Bat => delay_from_hit_tick_list(&[2, 2, 2, 3, 3], origin_dist),
        MobType::Blob => {
            if style == Style::Range {
                delay_from_hit_tick_list(
                    &[2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 5, 5, 5, 5, 6, 6],
                    origin_dist,
                )
            } else {
                delay_from_hit_tick_list(
                    &[2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6],
                    origin_dist,
                )
            }
        }
        MobType::BlobletRange => magic_delay(dist_to_mob(player.x, player.y, mob)),
        _ => {
            let edge_dist = dist_to_mob(player.x, player.y, mob);
            if style == Style::Range {
                ranged_delay(edge_dist)
            } else {
                magic_delay(edge_dist)
            }
        }
    }
}

pub(crate) fn ranged_delay(dist: i32) -> i32 {
    if dist <= 4 {
        2
    } else if dist <= 8 {
        3
    } else if dist <= 11 {
        4
    } else {
        5
    }
}

pub(crate) fn magic_delay(dist: i32) -> i32 {
    if dist <= 6 {
        2
    } else if dist <= 10 {
        3
    } else if dist <= 14 {
        4
    } else {
        5
    }
}

pub(crate) fn player_projectile_delay(loadout: &Loadout, px: i32, py: i32, target: &Mob) -> i32 {
    if loadout.atk_speed == 2 {
        return 2;
    }
    if loadout.is_blood_barrage {
        let sw_dist = chebyshev(px, py, target.x, target.y);
        if sw_dist <= 1 {
            2
        } else if sw_dist <= 3 {
            3
        } else if sw_dist <= 7 {
            4
        } else {
            5
        }
    } else {
        let dist = dist_to_mob(px, py, target);
        if dist <= 2 {
            2
        } else {
            3
        }
    }
}

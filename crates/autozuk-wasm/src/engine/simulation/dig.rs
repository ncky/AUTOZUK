use crate::engine::{collides_with_entities, collision_math, Mob, Player, Point, State};

pub(crate) fn start_dig(idx: usize, state: &mut State) {
    let s = state.mobs[idx].size;
    state.mobs[idx].frozen = 6;
    state.mobs[idx].dig_timer = 6;
    let px = state.player.x;
    let py = state.player.y;
    let loc = if !collides_with_entities(px - s + 1, py + s - 1, s, &state.region.entities) {
        Point {
            x: px - s + 1,
            y: py + s - 1,
        }
    } else if !collides_with_entities(px, py, s, &state.region.entities) {
        Point { x: px, y: py }
    } else if !collides_with_entities(px - s + 1, py, s, &state.region.entities) {
        Point {
            x: px - s + 1,
            y: py,
        }
    } else if !collides_with_entities(px, py + s - 1, s, &state.region.entities) {
        Point {
            x: px,
            y: py + s - 1,
        }
    } else {
        Point {
            x: px - 1,
            y: py + 1,
        }
    };
    state.mobs[idx].dig_location = Some(loc);
}

pub(crate) fn is_under_mob(mob: &Mob, player: &Player) -> bool {
    collision_math(mob.x, mob.y, mob.size, player.x, player.y, 1)
}

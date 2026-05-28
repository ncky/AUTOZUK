use crate::engine::{
    collision_math, key, mob_has_los, move_mob_to, sign, start_dig, state_point_collides_with_mobs,
    MobType, State,
};

pub(crate) fn hl_move_mob(idx: usize, state: &mut State) {
    if state.mobs[idx].has_dig && state.mobs[idx].dig_timer > 0 {
        state.mobs[idx].dig_timer -= 1;
        if state.mobs[idx].dig_timer == 0 {
            if let Some(loc) = state.mobs[idx].dig_location {
                move_mob_to(state, idx, loc.x, loc.y);
            }
            state.mobs[idx].attack_delay = 6;
            state.mobs[idx].frozen = 2;
            state.mobs[idx].dig_location = None;
            if state.player.aggro == Some(state.mobs[idx].id) {
                state.player.aggro = None;
            }
        }
        return;
    }

    let has_los = mob_has_los(state.region, &state.mobs[idx], &state.player);
    state.mobs[idx].had_los = state.mobs[idx].has_los;
    state.mobs[idx].has_los = has_los;
    state.mobs[idx].los_checked_tick = state.tick;
    if has_los {
        state.mobs[idx].no_los_ticks = 0;
    } else {
        state.mobs[idx].no_los_ticks += 1;
    }
    if state.mobs[idx].has_los || state.mobs[idx].frozen > 0 {
        return;
    }
    if state.mobs[idx].has_dig && !state.mobs[idx].has_los && state.mobs[idx].dig_timer == 0 {
        let attack_delay = state.mobs[idx].attack_delay;
        if (attack_delay <= -38 && state.rng.next_f64() < 0.1) || attack_delay <= -50 {
            start_dig(idx, state);
            return;
        }
    }

    let mut dx = state.mobs[idx].x + sign(state.player.x - state.mobs[idx].x);
    let mut dy = state.mobs[idx].y + sign(state.player.y - state.mobs[idx].y);
    if collision_math(
        state.mobs[idx].x,
        state.mobs[idx].y,
        state.mobs[idx].size,
        state.player.x,
        state.player.y,
        1,
    ) {
        if state.rng.next_f64() < 0.5 {
            dy = state.mobs[idx].y;
            dx = state.mobs[idx].x + if state.rng.next_f64() < 0.5 { 1 } else { -1 };
        } else {
            dx = state.mobs[idx].x;
            dy = state.mobs[idx].y + if state.rng.next_f64() < 0.5 { 1 } else { -1 };
        }
    } else if collision_math(
        dx,
        dy,
        state.mobs[idx].size,
        state.player.x,
        state.player.y,
        1,
    ) {
        dy = state.mobs[idx].y;
    }
    if state.mobs[idx].attack_delay > state.mobs[idx].atk_speed {
        return;
    }
    let x_off = dx - state.mobs[idx].x;
    let y_off = state.mobs[idx].y - dy;
    let both = hl_can_move(idx, x_off, y_off, state);
    let mut can_x = false;
    let mut can_y = false;
    if !both {
        if x_off != 0 {
            can_x = hl_can_move(idx, x_off, 0, state);
        }
        if !can_x && y_off != 0 {
            can_y = hl_can_move(idx, 0, y_off, state);
        }
    }
    if both {
        move_mob_to(state, idx, dx, dy);
    } else if can_x {
        let y = state.mobs[idx].y;
        move_mob_to(state, idx, dx, y);
    } else if can_y {
        let x = state.mobs[idx].x;
        move_mob_to(state, idx, x, dy);
    }
}

pub(crate) fn hl_can_move(idx: usize, x_off: i32, y_off: i32, state: &State) -> bool {
    if x_off == 0 && y_off == 0 {
        return true;
    }
    let mob = &state.mobs[idx];
    let s = mob.size;
    let dx = x_off;
    let dy = -y_off;
    let nx = mob.x + dx;
    let ny = mob.y + dy;
    let is_nib = mob.mob_type == MobType::Nibbler;
    if dx == -1 {
        for i in 0..s {
            if state.region.blocked[key(nx, ny - i)] != 0 {
                return false;
            }
            if !is_nib
                && state_point_collides_with_mobs(nx, ny - i, state, Some(idx), true).is_some()
            {
                return false;
            }
        }
    } else if dx == 1 {
        let rx = nx + s - 1;
        for i in 0..s {
            if state.region.blocked[key(rx, ny - i)] != 0 {
                return false;
            }
            if !is_nib
                && state_point_collides_with_mobs(rx, ny - i, state, Some(idx), true).is_some()
            {
                return false;
            }
        }
    }
    if dy == 1 {
        for i in 0..s {
            if state.region.blocked[key(nx + i, ny)] != 0 {
                return false;
            }
            if !is_nib
                && state_point_collides_with_mobs(nx + i, ny, state, Some(idx), true).is_some()
            {
                return false;
            }
        }
    } else if dy == -1 {
        let by = ny - s + 1;
        for i in 0..s {
            if state.region.blocked[key(nx + i, by)] != 0 {
                return false;
            }
            if !is_nib
                && state_point_collides_with_mobs(nx + i, by, state, Some(idx), true).is_some()
            {
                return false;
            }
        }
    }
    true
}

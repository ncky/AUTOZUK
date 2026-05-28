use crate::engine::{
    aggro_is_unusable, dist_to_mob, get_closest_face_tile, hl_mob_attack, hl_move_mob,
    hl_player_attack_if_ready, hl_process_corpse_expiry, hl_process_delayed_bloblet_spawns,
    hl_process_pending_mob_deaths, mob_idx_by_id, osrs_walk_step, player_has_los, Mob, Point,
    State,
};

// Phase order is gameplay-visible and must stay aligned with the JS simulator.
pub(crate) fn hl_tick(state: &mut State) {
    state.tick += 1;
    let tick = state.tick;
    hl_process_world_events(state, tick);
    hl_advance_mobs(state);
    hl_fire_ready_mob_attacks(state, tick);
    hl_apply_player_projectile_hits(state, tick);
    let player_was_hit = hl_advance_incoming_mob_projectiles(state);
    hl_update_auto_retaliate(state, tick, player_was_hit);
    hl_advance_player(state, tick);
}

pub(crate) fn hl_process_world_events(state: &mut State, tick: i32) {
    hl_process_corpse_expiry(state, tick);
    hl_process_pending_mob_deaths(state, tick);
    hl_process_delayed_bloblet_spawns(state, tick);
}

pub(crate) fn hl_advance_mobs(state: &mut State) {
    let move_len = state.mobs.len();
    for idx in 0..move_len {
        if state.mobs[idx].dead || state.mobs[idx].dying > 0 {
            continue;
        }
        if state.mobs[idx].stunned > 0 {
            state.mobs[idx].stunned -= 1;
            continue;
        }
        if state.mobs[idx].frozen > 0 {
            state.mobs[idx].frozen -= 1;
            continue;
        }
        hl_move_mob(idx, state);
    }
}

pub(crate) fn hl_fire_ready_mob_attacks(state: &mut State, tick: i32) {
    let attack_len = state.mobs.len();
    for idx in 0..attack_len {
        if state.mobs[idx].dead || state.mobs[idx].dying > 0 || state.mobs[idx].stunned > 0 {
            continue;
        }
        state.mobs[idx].attack_delay -= 1;
        hl_mob_attack(idx, state, tick);
    }
}

pub(crate) fn hl_apply_player_projectile_hits(state: &mut State, tick: i32) {
    let projectile_len = state.mobs.len();
    for idx in 0..projectile_len {
        if state.mobs[idx].dead || state.mobs[idx].dying > 0 {
            continue;
        }
        hl_apply_player_projectiles_to_mob(&mut state.mobs[idx], tick);
    }
}

pub(crate) fn hl_apply_player_projectiles_to_mob(mob: &mut Mob, tick: i32) {
    let mut hp = mob.hp;
    let mut pending = mob.pending_removal_tick;
    let mut dying_start_tick = mob.dying_start_tick;
    let projectiles = &mut mob.incoming_projectiles;
    let mut i = projectiles.len();
    while i > 0 {
        i -= 1;
        let mut p = projectiles.get(i);
        p.delay -= 1;
        if p.delay <= 0 {
            if pending.is_none() {
                hp -= p.damage;
                if hp <= 0 {
                    hp = 0;
                    pending = Some(tick + 1);
                    dying_start_tick = tick;
                }
            }
            projectiles.swap_remove(i);
        } else {
            projectiles.set(i, p);
        }
    }
    mob.hp = hp;
    mob.pending_removal_tick = pending;
    mob.dying_start_tick = dying_start_tick;
}

pub(crate) fn hl_advance_incoming_mob_projectiles(state: &mut State) -> bool {
    let mut arrived = false;
    {
        let incoming = &mut state.player.incoming_projectiles;
        let mut i = incoming.len();
        while i > 0 {
            i -= 1;
            let mut p = incoming.get(i);
            p.delay -= 1;
            if p.delay <= 0 {
                arrived = true;
                incoming.swap_remove(i);
            } else {
                incoming.set(i, p);
            }
        }
    }
    arrived
}

pub(crate) fn hl_update_auto_retaliate(state: &mut State, tick: i32, player_was_hit: bool) {
    if !state.player.auto_retaliate
        || !player_was_hit
        || !aggro_is_unusable(state, state.player.aggro, tick)
    {
        return;
    }
    let Some(target_id) = state.player.last_attacker else {
        return;
    };
    let Some(target_idx) = mob_idx_by_id(state, target_id) else {
        return;
    };
    let target = &state.mobs[target_idx];
    if !can_auto_retaliate_to(target) {
        return;
    }

    state.player.aggro = Some(target_id);
    let forced_delay = state.loadout.atk_speed / 2 + 1;
    if state.player.attack_delay < forced_delay {
        state.player.attack_delay = forced_delay;
    }
}

pub(crate) fn can_auto_retaliate_to(mob: &Mob) -> bool {
    !mob.dead && mob.dying == -1 && mob.pending_removal_tick.is_none()
}

pub(crate) fn hl_advance_player(state: &mut State, tick: i32) {
    state.player.attack_delay -= 1;
    hl_move_player_toward_aggro(state);
    hl_clear_expired_player_aggro(state, tick);
    hl_player_attack_if_ready(state, tick);
}

pub(crate) fn hl_move_player_toward_aggro(state: &mut State) {
    let Some(aggro_idx) = player_aggro_idx(state) else {
        return;
    };
    let aggro = &state.mobs[aggro_idx];
    if !can_move_toward_player_target(state, aggro) {
        return;
    }
    let Some(dest) = get_closest_face_tile(aggro, state.player.x, state.player.y, state.region)
    else {
        return;
    };
    hl_walk_player_toward(state, dest);
}

pub(crate) fn player_aggro_idx(state: &State<'_>) -> Option<usize> {
    state.player.aggro.and_then(|id| mob_idx_by_id(state, id))
}

pub(crate) fn can_move_toward_player_target(state: &State<'_>, target: &Mob) -> bool {
    if target.dead || target.dying > 0 || target.pending_removal_tick.is_some() {
        return false;
    }
    dist_to_mob(state.player.x, state.player.y, target) > state.player.range
        || !player_has_los(
            state.region,
            state.player.x,
            state.player.y,
            target,
            state.player.range,
        )
}

pub(crate) fn hl_walk_player_toward(state: &mut State<'_>, dest: Point) {
    if let Some(s1) = osrs_walk_step(state.player.x, state.player.y, dest.x, dest.y, state.region) {
        state.player.x = s1.x;
        state.player.y = s1.y;
        if state.player.x != dest.x || state.player.y != dest.y {
            if let Some(s2) =
                osrs_walk_step(state.player.x, state.player.y, dest.x, dest.y, state.region)
            {
                state.player.x = s2.x;
                state.player.y = s2.y;
            }
        }
    }
}

pub(crate) fn hl_clear_expired_player_aggro(state: &mut State, tick: i32) {
    if let Some(aggro_id) = state.player.aggro {
        if let Some(aggro_idx) = mob_idx_by_id(state, aggro_id) {
            let m = &state.mobs[aggro_idx];
            if m.dead || (m.dying > 0 && tick > m.dying_start_tick) {
                state.player.aggro = None;
            }
        } else {
            state.player.aggro = None;
        }
    }
}

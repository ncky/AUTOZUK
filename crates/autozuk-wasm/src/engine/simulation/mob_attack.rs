use crate::engine::{
    can_use_secondary_melee, dist_to_mob, event_i16, event_id, event_mob_type, event_style,
    find_respawn_location, has_dead_mob_for_revive, is_under_mob, mob_has_los, mob_idx_by_id,
    mob_style, monster_projectile_delay, pop_dead_mob_for_revive, set_player_last_attacker,
    update_mob_occupancy, AttackEvent, MobProjectile, MobType, State, Style,
};

pub(crate) fn hl_mob_attack(idx: usize, state: &mut State, tick: i32) {
    if state.mobs[idx].dead || state.mobs[idx].dying > 0 || state.mobs[idx].stunned > 0 {
        return;
    }
    let has_los = if state.mobs[idx].los_checked_tick == tick {
        state.mobs[idx].has_los
    } else {
        mob_has_los(state.region, &state.mobs[idx], &state.player)
    };
    state.mobs[idx].had_los = state.mobs[idx].has_los;
    state.mobs[idx].has_los = has_los;
    state.mobs[idx].los_checked_tick = tick;

    if state.mobs[idx].has_flicker {
        state.mobs[idx].flickering = state.mobs[idx].attack_delay == 1 && state.mobs[idx].has_los;
        if !state.mobs[idx].has_los
            || state.mobs[idx].attack_delay > 0
            || is_under_mob(&state.mobs[idx], &state.player)
        {
            return;
        }
        if state.rng.next_f64() < 0.1 && has_dead_mob_for_revive(state) {
            let Some(revive_id) = pop_dead_mob_for_revive(state) else {
                return;
            };
            if let Some(revive_idx) = mob_idx_by_id(state, revive_id) {
                let revive_hp = state.mobs[revive_idx].max_hp / 2;
                update_mob_occupancy(
                    &mut state.mob_occupancy,
                    revive_idx,
                    &state.mobs[revive_idx],
                    false,
                );
                if state.mobs[revive_idx].dead && state.dead_count > 0 {
                    state.dead_count -= 1;
                }
                {
                    let mob = &mut state.mobs[revive_idx];
                    mob.revived_once = true;
                    mob.hp = revive_hp;
                    mob.dead = false;
                    mob.dying = -1;
                    mob.pending_removal_tick = None;
                    mob.corpse_removal_tick = None;
                    mob.attack_delay = mob.atk_speed + 1;
                    mob.stunned = 0;
                    mob.frozen = 0;
                }
                let loc =
                    find_respawn_location(state.mobs[revive_idx].size, state.region, &state.mobs);
                state.mobs[revive_idx].x = loc.x;
                state.mobs[revive_idx].y = loc.y;
                update_mob_occupancy(
                    &mut state.mob_occupancy,
                    revive_idx,
                    &state.mobs[revive_idx],
                    true,
                );
                state.attacks.push(AttackEvent {
                    tick,
                    mob_id: event_id(Some(revive_id)),
                    target_mob_id: event_id(None),
                    mob_type: event_mob_type(Some(state.mobs[revive_idx].mob_type)),
                    style: event_style(None),
                    is_scan: false,
                    scan_tick: -1,
                    acc_roll: 0.0,
                    dmg_roll: 0.0,
                    dist_at_fire: event_i16(None),
                    hit_tick: event_i16(None),
                    is_player_attack: false,
                    player_dmg: 0,
                    is_revive: true,
                    revive_hp: event_i16(Some(revive_hp)),
                });
                state.mobs[idx].attack_delay = state.mobs[idx].atk_speed * 2;
                return;
            }
        }
        hl_fire_attack(idx, state, tick, None, None);
        state.mobs[idx].attack_delay = state.mobs[idx].atk_speed;
        return;
    }

    if state.mobs[idx].is_blob {
        if !state.mobs[idx].has_los && !state.mobs[idx].blob_scan_prayer {
            return;
        }
        if state.mobs[idx].has_los
            && (!state.mobs[idx].had_los
                || (!state.mobs[idx].blob_scan_prayer && state.mobs[idx].attack_delay <= 0))
        {
            state.mobs[idx].blob_scan_prayer = true;
            state.mobs[idx].attack_delay = state.mobs[idx].atk_speed;
            state.mobs[idx].last_scan_tick = Some(tick);
            state.mobs[idx].current_style = Some(if state.rng.next_f64() < 0.5 {
                Style::Magic
            } else {
                Style::Range
            });
            let mob_id = state.mobs[idx].id;
            state.attacks.push(AttackEvent {
                tick,
                mob_id: event_id(Some(mob_id)),
                target_mob_id: event_id(None),
                mob_type: event_mob_type(Some(MobType::Blob)),
                style: event_style(None),
                is_scan: true,
                scan_tick: tick,
                acc_roll: 0.0,
                dmg_roll: 0.0,
                dist_at_fire: event_i16(None),
                hit_tick: event_i16(None),
                is_player_attack: false,
                player_dmg: 0,
                is_revive: false,
                revive_hp: event_i16(None),
            });
            return;
        }
        if state.mobs[idx].blob_scan_prayer && state.mobs[idx].attack_delay <= 0 {
            let scan_tick = state.mobs[idx].last_scan_tick.unwrap_or(tick - 3);
            hl_fire_attack(idx, state, tick, Some(Style::Blob), Some(scan_tick));
            state.mobs[idx].blob_scan_prayer = false;
            state.mobs[idx].attack_delay = state.mobs[idx].atk_speed;
        }
        return;
    }

    if !state.mobs[idx].has_los
        || state.mobs[idx].attack_delay > 0
        || is_under_mob(&state.mobs[idx], &state.player)
    {
        return;
    }
    let style = mob_style(state.mobs[idx].mob_type);
    hl_fire_attack(idx, state, tick, Some(style), None);
    state.mobs[idx].attack_delay = state.mobs[idx].atk_speed;
}

pub(crate) fn hl_fire_attack(
    idx: usize,
    state: &mut State,
    tick: i32,
    style_or_blob: Option<Style>,
    scan_tick: Option<i32>,
) {
    let mut is_blob_attack = false;
    let mut style = match style_or_blob {
        Some(Style::Blob) => {
            is_blob_attack = true;
            None
        }
        Some(s) => Some(s),
        None => state.mobs[idx]
            .current_style
            .or(Some(mob_style(state.mobs[idx].mob_type))),
    };
    if style == Some(Style::Blob) {
        style = state.mobs[idx].current_style.or(Some(Style::Magic));
    }
    let mut projectile_style = if is_blob_attack {
        state.mobs[idx].current_style.unwrap_or(Style::Magic)
    } else {
        style.unwrap_or(Style::Magic)
    };
    if can_use_secondary_melee(&state.mobs[idx], &state.player) && state.rng.next_f64() < 0.5 {
        projectile_style = Style::Melee;
        style = Some(Style::Melee);
        is_blob_attack = false;
    }
    let delay = monster_projectile_delay(&state.mobs[idx], projectile_style, &state.player);
    let edge_dist = dist_to_mob(state.player.x, state.player.y, &state.mobs[idx]);
    let mob_id = state.mobs[idx].id;
    let mob_type = state.mobs[idx].mob_type;
    state
        .player
        .incoming_projectiles
        .push(MobProjectile { delay: delay + 1 });
    set_player_last_attacker(state, mob_id);
    state.attacks.push(AttackEvent {
        tick,
        mob_id: event_id(Some(mob_id)),
        target_mob_id: event_id(None),
        mob_type: event_mob_type(Some(mob_type)),
        style: event_style(if is_blob_attack { None } else { style }),
        is_scan: false,
        scan_tick: if is_blob_attack {
            scan_tick.unwrap_or(tick - 3)
        } else {
            -1
        },
        acc_roll: state.rng.next_f64(),
        dmg_roll: state.rng.next_f64(),
        dist_at_fire: event_i16(Some(edge_dist)),
        hit_tick: event_i16(Some(tick + delay)),
        is_player_attack: false,
        player_dmg: 0,
        is_revive: false,
        revive_hp: event_i16(None),
    });
}

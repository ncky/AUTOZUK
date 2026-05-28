use crate::engine::{
    event_i16, event_id, event_mob_type, event_style, mob_idx_by_id, player_has_los,
    player_projectile_delay, AttackEvent, Mob, MobType, PlayerProjectile, State,
    BLOOD_BARRAGE_AOE_OFFSETS,
};

pub(crate) fn hl_player_attack_if_ready(state: &mut State, tick: i32) {
    if let Some(target_id) = state.player.aggro {
        if let Some(target_idx) = mob_idx_by_id(state, target_id) {
            let can_attack = {
                let target = &state.mobs[target_idx];
                state.player.attack_delay <= 0
                    && target.pending_removal_tick.is_none()
                    && player_has_los(
                        state.region,
                        state.player.x,
                        state.player.y,
                        target,
                        state.player.range,
                    )
            };
            if can_attack {
                hl_player_attack(target_idx, state, tick);
            }
        }
    }
}

pub(crate) fn hl_player_attack(target_idx: usize, state: &mut State, tick: i32) {
    let target_type = state.mobs[target_idx].mob_type;
    let damage = roll_player_damage(state, target_type, true);
    queue_player_projectile(state, target_idx, tick, damage);

    if state.loadout.is_blood_barrage {
        hl_player_blood_barrage_splash(target_idx, state, tick);
    }

    state.player.attack_delay = state.loadout.atk_speed;
}

pub(crate) fn roll_player_damage(
    state: &mut State,
    mob_type: MobType,
    update_last_hit: bool,
) -> i32 {
    let acc_arr = state.loadout.player_acc[mob_type.idx()];
    let acc = if state.player.last_hit {
        acc_arr[0]
    } else {
        acc_arr[1]
    };
    let hit = state.rng.next_f64() < acc;
    if update_last_hit {
        state.player.last_hit = hit;
    }
    if hit {
        (state.rng.next_f64() * ((state.loadout.max_hit + 1) as f64)).floor() as i32
    } else {
        0
    }
}

pub(crate) fn queue_player_projectile(
    state: &mut State,
    target_idx: usize,
    tick: i32,
    damage: i32,
) {
    let delay = {
        let target = &state.mobs[target_idx];
        player_projectile_delay(state.loadout, state.player.x, state.player.y, target)
    };
    let target_id = state.mobs[target_idx].id;
    state.mobs[target_idx]
        .incoming_projectiles
        .push(PlayerProjectile { delay, damage });
    state
        .attacks
        .push(player_attack_event(tick, target_id, tick + delay, damage));
}

pub(crate) fn player_attack_event(
    tick: i32,
    target_id: usize,
    hit_tick: i32,
    damage: i32,
) -> AttackEvent {
    AttackEvent {
        tick,
        mob_id: event_id(None),
        target_mob_id: event_id(Some(target_id)),
        mob_type: event_mob_type(None),
        style: event_style(None),
        is_scan: false,
        scan_tick: -1,
        acc_roll: 0.0,
        dmg_roll: 0.0,
        dist_at_fire: event_i16(None),
        hit_tick: event_i16(Some(hit_tick)),
        is_player_attack: true,
        player_dmg: damage,
        is_revive: false,
        revive_hp: event_i16(None),
    }
}

pub(crate) fn hl_player_blood_barrage_splash(primary_idx: usize, state: &mut State, tick: i32) {
    let target_x = state.mobs[primary_idx].x;
    let target_y = state.mobs[primary_idx].y;
    let mut hit_count = 1;
    for (ox, oy) in BLOOD_BARRAGE_AOE_OFFSETS {
        if hit_count >= 9 {
            break;
        }
        let ax = target_x + ox;
        let ay = target_y + oy;
        for mob_idx in 0..state.mobs.len() {
            if mob_idx == primary_idx || !blood_barrage_can_splash(&state.mobs[mob_idx], ax, ay) {
                continue;
            }
            hit_count += 1;
            let mob_type = state.mobs[mob_idx].mob_type;
            let damage = roll_player_damage(state, mob_type, false);
            queue_player_projectile(state, mob_idx, tick, damage);
            break;
        }
    }
}

pub(crate) fn blood_barrage_can_splash(mob: &Mob, x: i32, y: i32) -> bool {
    !mob.dead && mob.dying <= 0 && mob.pending_removal_tick.is_none() && mob.x == x && mob.y == y
}

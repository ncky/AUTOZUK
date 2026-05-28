use crate::engine::{
    hl_create_mob, mob_idx_by_id, set_id_to_idx, set_mob_init, update_mob_occupancy,
    DelayedBlobletSpawn, MobType, State, DEATH_ANIM_TICKS,
};

pub(crate) fn hl_process_corpse_expiry(state: &mut State, tick: i32) {
    for idx in 0..state.mobs.len() {
        if state.mobs[idx].dead || state.mobs[idx].dying <= 0 {
            continue;
        }
        let remain = state.mobs[idx].corpse_removal_tick.unwrap_or(tick) - tick;
        if remain <= 0 {
            update_mob_occupancy(&mut state.mob_occupancy, idx, &state.mobs[idx], false);
            let mob = &mut state.mobs[idx];
            mob.dead = true;
            state.dead_count += 1;
            mob.dying = 0;
            mob.corpse_removal_tick = None;
            mob.pending_removal_tick = None;
        } else {
            state.mobs[idx].dying = remain;
        }
    }
}

pub(crate) fn hl_process_pending_mob_deaths(state: &mut State, tick: i32) {
    let len = state.mobs.len();
    for idx in 0..len {
        if state.mobs[idx].dead || state.mobs[idx].dying > 0 {
            continue;
        }
        if state.mobs[idx]
            .pending_removal_tick
            .map(|t| t <= tick)
            .unwrap_or(false)
        {
            {
                let mob = &mut state.mobs[idx];
                mob.pending_removal_tick = None;
                mob.dying = DEATH_ANIM_TICKS;
                mob.corpse_removal_tick = Some(tick + DEATH_ANIM_TICKS);
            }
            hl_on_death(idx, state, tick);
        }
    }
}

pub(crate) fn hl_spawn_bloblets_from_blob(state: &mut State, blob_id: usize) {
    let Some(blob_idx) = mob_idx_by_id(state, blob_id) else {
        return;
    };
    let blob_x = state.mobs[blob_idx].x;
    let blob_y = state.mobs[blob_idx].y;
    let mut bm = hl_create_mob(
        MobType::BlobletMage,
        blob_x + 2,
        blob_y - 2,
        state.id_counter,
    );
    state.id_counter += 1;
    bm.frozen = 1;
    bm.attack_delay = 4;
    bm.parent_blob_id = Some(blob_id);

    let mut br = hl_create_mob(
        MobType::BlobletRange,
        blob_x + 1,
        blob_y - 1,
        state.id_counter,
    );
    state.id_counter += 1;
    br.frozen = 1;
    br.attack_delay = 4;
    br.parent_blob_id = Some(blob_id);

    let mut bx = hl_create_mob(MobType::BlobletMelee, blob_x, blob_y, state.id_counter);
    state.id_counter += 1;
    bx.frozen = 1;
    bx.attack_delay = 4;
    bx.parent_blob_id = Some(blob_id);

    set_mob_init(
        &mut state.mob_init_hp,
        &mut state.mob_init_len,
        bm.id,
        bm.hp,
    );
    set_mob_init(
        &mut state.mob_init_hp,
        &mut state.mob_init_len,
        br.id,
        br.hp,
    );
    set_mob_init(
        &mut state.mob_init_hp,
        &mut state.mob_init_len,
        bx.id,
        bx.hp,
    );
    let bm_idx = state.mobs.len();
    set_id_to_idx(&mut state.id_to_idx, bm.id, bm_idx);
    state.mobs.push(bm);
    update_mob_occupancy(&mut state.mob_occupancy, bm_idx, &state.mobs[bm_idx], true);
    let br_idx = state.mobs.len();
    set_id_to_idx(&mut state.id_to_idx, br.id, br_idx);
    state.mobs.push(br);
    update_mob_occupancy(&mut state.mob_occupancy, br_idx, &state.mobs[br_idx], true);
    let bx_idx = state.mobs.len();
    set_id_to_idx(&mut state.id_to_idx, bx.id, bx_idx);
    state.mobs.push(bx);
    update_mob_occupancy(&mut state.mob_occupancy, bx_idx, &state.mobs[bx_idx], true);
}

pub(crate) fn hl_process_delayed_bloblet_spawns(state: &mut State, tick: i32) {
    if state.delayed_bloblet_spawns.is_empty() {
        return;
    }
    let mut write = 0usize;
    let len = state.delayed_bloblet_spawns.len();
    for read in 0..len {
        let item = state.delayed_bloblet_spawns[read];
        if item.tick <= tick {
            hl_spawn_bloblets_from_blob(state, item.blob_id);
        } else {
            state.delayed_bloblet_spawns[write] = item;
            write += 1;
        }
    }
    state.delayed_bloblet_spawns.truncate(write);
}

pub(crate) fn hl_on_death(idx: usize, state: &mut State, tick: i32) {
    let mob_type = state.mobs[idx].mob_type;
    let mob_id = state.mobs[idx].id;
    if state.mobs[idx].is_blob {
        state.delayed_bloblet_spawns.push(DelayedBlobletSpawn {
            tick: tick + 1,
            blob_id: mob_id,
        });
    }
    if !matches!(
        mob_type,
        MobType::BlobletMage | MobType::BlobletRange | MobType::BlobletMelee | MobType::Nibbler
    ) && !state.mobs[idx].revived_once
    {
        state.dead_mobs.push(mob_id);
    }
    if state.player.aggro == Some(mob_id) {
        state.player.aggro = None;
    }
    if state.player.last_attacker == Some(mob_id) {
        state.player.last_attacker = None;
    }
}

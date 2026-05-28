use crate::engine::{
    collides_with_entities, collides_with_mobs, key, mob_atk_speed, mob_has_dig, mob_has_flicker,
    mob_hp, mob_range, mob_size, rebuild_mob_occupancy, AttackEvent, Loadout, Mob, MobInit,
    MobProjectileQueue, MobType, Mulberry32, ParsedSpawnCode, PillarConfig, Player,
    PlayerProjectileQueue, Point, Region, State, ARENA_CELLS, MAX_MOB_IDS,
};

pub(crate) fn spawn_nibblers(
    mobs: &mut Vec<Mob>,
    region: &Region,
    rng: &mut Mulberry32,
    id_counter: &mut usize,
) {
    let mut positions = Vec::with_capacity(9);
    for x in 9..=11 {
        for y in 12..=14 {
            positions.push(Point { x, y });
        }
    }
    for i in (1..positions.len()).rev() {
        let j = (rng.next_f64() * ((i + 1) as f64)).floor() as usize;
        positions.swap(i, j);
    }
    let mut spawned = 0;
    for pos in positions {
        if spawned >= 3 {
            break;
        }
        if region.blocked[key(pos.x, pos.y)] == 0
            && collides_with_mobs(pos.x, pos.y, 1, mobs, None, false).is_none()
        {
            let mut nib = hl_create_mob(MobType::Nibbler, pos.x, pos.y, *id_counter);
            *id_counter += 1;
            nib.stunned = 0;
            mobs.push(nib);
            spawned += 1;
        }
    }
}

pub(crate) fn find_respawn_location(size: i32, region: &Region, mobs: &[Mob]) -> Point {
    for x in 16..23 {
        for y in 11..24 {
            if collides_with_mobs(x, y, size, mobs, None, false).is_none()
                && !collides_with_entities(x, y, size, &region.entities)
            {
                return Point { x, y };
            }
        }
    }
    Point { x: 11, y: 9 }
}

pub(crate) fn hl_create_mob(mob_type: MobType, x: i32, y: i32, id: usize) -> Mob {
    let hp = mob_hp(mob_type);
    Mob {
        id,
        mob_type,
        x,
        y,
        size: mob_size(mob_type),
        hp,
        max_hp: hp,
        atk_speed: mob_atk_speed(mob_type),
        range: mob_range(mob_type),
        attack_delay: 0,
        stunned: 0,
        frozen: 0,
        dead: false,
        dying: -1,
        dying_start_tick: -1,
        corpse_removal_tick: None,
        pending_removal_tick: None,
        revived_once: false,
        has_los: false,
        had_los: false,
        los_checked_tick: -1,
        is_blob: mob_type == MobType::Blob,
        blob_scan_prayer: false,
        last_scan_tick: None,
        has_dig: mob_has_dig(mob_type),
        dig_timer: 0,
        dig_location: None,
        has_flicker: mob_has_flicker(mob_type),
        flickering: false,
        incoming_projectiles: PlayerProjectileQueue::new(),
        no_los_ticks: 0,
        current_style: None,
        parent_blob_id: None,
        inf_num: 0,
    }
}

pub(crate) fn hl_create_player(x: i32, y: i32, loadout: &Loadout) -> Player {
    Player {
        x,
        y,
        attack_delay: 0,
        range: loadout.range,
        incoming_projectiles: MobProjectileQueue::new(),
        auto_retaliate: true,
        last_hit: true,
        aggro: None,
        last_attacker: None,
    }
}

pub(crate) fn set_mob_init(
    mob_init_hp: &mut [Option<MobInit>; MAX_MOB_IDS],
    mob_init_len: &mut usize,
    id: usize,
    hp: i32,
) {
    if id >= MAX_MOB_IDS {
        return;
    }
    mob_init_hp[id] = Some(MobInit { hp });
    *mob_init_len = (*mob_init_len).max(id + 1);
}

pub(crate) fn set_id_to_idx(id_to_idx: &mut [Option<usize>; MAX_MOB_IDS], id: usize, idx: usize) {
    if id < MAX_MOB_IDS {
        id_to_idx[id] = Some(idx);
    }
}

pub(crate) fn has_dead_mob_for_revive(state: &State<'_>) -> bool {
    state.dead_mob_cursor < state.dead_mobs.len()
}

pub(crate) fn pop_dead_mob_for_revive(state: &mut State<'_>) -> Option<usize> {
    if state.dead_mob_cursor >= state.dead_mobs.len() {
        state.dead_mobs.clear();
        state.dead_mob_cursor = 0;
        return None;
    }
    let mob_id = state.dead_mobs[state.dead_mob_cursor];
    state.dead_mob_cursor += 1;
    if state.dead_mob_cursor >= state.dead_mobs.len() {
        state.dead_mobs.clear();
        state.dead_mob_cursor = 0;
    }
    Some(mob_id)
}

pub(crate) fn hl_init_state<'a>(
    parsed: &ParsedSpawnCode,
    player_pos: Point,
    pillar_config: PillarConfig,
    loadout: &'a Loadout,
    region: &'a Region,
    seed: u32,
    attack_log: &'a mut Vec<AttackEvent>,
) -> State<'a> {
    let mut id_counter = 0usize;
    let mut rng = Mulberry32::new(seed);
    let mut mobs = Vec::with_capacity(parsed.spawns.len() + 12);
    for spawn in &parsed.spawns {
        if let Some(mob_type) = spawn.mob_type {
            let mut mob = hl_create_mob(mob_type, spawn.x, spawn.y, id_counter);
            id_counter += 1;
            mob.inf_num = spawn.inf_num;
            mobs.push(mob);
        }
    }
    if parsed.has_index_info {
        mobs.sort_by(|a, b| b.inf_num.cmp(&a.inf_num));
    }
    if !pillar_config.s && !pillar_config.w && !pillar_config.n {
        spawn_nibblers(&mut mobs, region, &mut rng, &mut id_counter);
    }
    let mut mob_init_hp = [None; MAX_MOB_IDS];
    let mut mob_init_len = 0usize;
    for mob in &mobs {
        set_mob_init(&mut mob_init_hp, &mut mob_init_len, mob.id, mob.hp);
    }
    let mut id_to_idx = [None; MAX_MOB_IDS];
    for (idx, mob) in mobs.iter().enumerate() {
        set_id_to_idx(&mut id_to_idx, mob.id, idx);
    }
    let mut mob_occupancy = [0; ARENA_CELLS];
    rebuild_mob_occupancy(&mobs, &mut mob_occupancy);
    State {
        region,
        mobs,
        mob_occupancy,
        id_to_idx,
        player: hl_create_player(player_pos.x, player_pos.y, loadout),
        tick: 0,
        dead_count: 0,
        dead_mobs: Vec::with_capacity(16),
        dead_mob_cursor: 0,
        id_counter,
        loadout,
        attacks: attack_log,
        mob_init_hp,
        mob_init_len,
        delayed_bloblet_spawns: Vec::with_capacity(8),
        rng,
    }
}

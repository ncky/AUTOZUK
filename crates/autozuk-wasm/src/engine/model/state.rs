use crate::engine::{
    AttackEvent, Loadout, Mob, MobInit, MobType, Mulberry32, Player, Region, ARENA_CELLS,
    MAX_MOB_IDS,
};

#[derive(Clone, Copy)]
pub(crate) struct DelayedBlobletSpawn {
    pub(crate) tick: i32,
    pub(crate) blob_id: usize,
}

pub(crate) struct State<'a> {
    pub(crate) region: &'a Region,
    pub(crate) mobs: Vec<Mob>,
    pub(crate) mob_occupancy: [u64; ARENA_CELLS],
    pub(crate) id_to_idx: [Option<usize>; MAX_MOB_IDS],
    pub(crate) player: Player,
    pub(crate) tick: i32,
    pub(crate) dead_count: usize,
    pub(crate) dead_mobs: Vec<usize>,
    pub(crate) dead_mob_cursor: usize,
    pub(crate) id_counter: usize,
    pub(crate) loadout: &'a Loadout,
    pub(crate) attacks: &'a mut Vec<AttackEvent>,
    pub(crate) mob_init_hp: [Option<MobInit>; MAX_MOB_IDS],
    pub(crate) mob_init_len: usize,
    pub(crate) delayed_bloblet_spawns: Vec<DelayedBlobletSpawn>,
    pub(crate) rng: Mulberry32,
}

#[derive(Clone)]
pub(crate) struct Spawn {
    pub(crate) mob_type: Option<MobType>,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) inf_num: i32,
}

pub(crate) struct ParsedSpawnCode {
    pub(crate) spawns: Vec<Spawn>,
    pub(crate) has_index_info: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct WaveFlags {
    pub(crate) has_mager: bool,
    pub(crate) has_ranger: bool,
    pub(crate) has_meleer: bool,
}

use crate::engine::{
    Loadout, MobProjectileQueue, MobType, ParsedSpawnCode, PillarConfig, PlayerProjectileQueue,
    Point, SimScratch, Style,
};

pub(crate) struct SimContext {
    pub(crate) parsed: ParsedSpawnCode,
    pub(crate) pillar_config: PillarConfig,
    pub(crate) loadout: Loadout,
    pub(crate) max_ticks: i32,
    pub(crate) seed_base: u32,
    pub(crate) scratch: SimScratch,
}

#[derive(Clone)]
pub(crate) struct Mob {
    pub(crate) id: usize,
    pub(crate) mob_type: MobType,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) size: i32,
    pub(crate) hp: i32,
    pub(crate) max_hp: i32,
    pub(crate) atk_speed: i32,
    pub(crate) range: i32,
    pub(crate) attack_delay: i32,
    pub(crate) stunned: i32,
    pub(crate) frozen: i32,
    pub(crate) dead: bool,
    pub(crate) dying: i32,
    pub(crate) dying_start_tick: i32,
    pub(crate) corpse_removal_tick: Option<i32>,
    pub(crate) pending_removal_tick: Option<i32>,
    pub(crate) revived_once: bool,
    pub(crate) has_los: bool,
    pub(crate) had_los: bool,
    pub(crate) los_checked_tick: i32,
    pub(crate) is_blob: bool,
    pub(crate) blob_scan_prayer: bool,
    pub(crate) last_scan_tick: Option<i32>,
    pub(crate) has_dig: bool,
    pub(crate) dig_timer: i32,
    pub(crate) dig_location: Option<Point>,
    pub(crate) has_flicker: bool,
    pub(crate) flickering: bool,
    pub(crate) incoming_projectiles: PlayerProjectileQueue,
    pub(crate) no_los_ticks: i32,
    pub(crate) current_style: Option<Style>,
    pub(crate) parent_blob_id: Option<usize>,
    pub(crate) inf_num: i32,
}

#[derive(Clone)]
pub(crate) struct Player {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) attack_delay: i32,
    pub(crate) range: i32,
    pub(crate) incoming_projectiles: MobProjectileQueue,
    pub(crate) auto_retaliate: bool,
    pub(crate) last_hit: bool,
    pub(crate) aggro: Option<usize>,
    pub(crate) last_attacker: Option<usize>,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct PlayerProjectile {
    pub(crate) delay: i32,
    pub(crate) damage: i32,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct MobProjectile {
    pub(crate) delay: i32,
}

use crate::engine::{MobType, Style, MOB_TYPE_COUNT};

#[derive(Clone, Copy)]
pub(crate) struct AttackStats {
    pub(crate) max: i32,
    pub(crate) acc: f64,
}

#[derive(Clone)]
pub(crate) struct MonsterAttackTable {
    pub(crate) normal: [Option<AttackStats>; MOB_TYPE_COUNT],
    pub(crate) melee: [Option<AttackStats>; MOB_TYPE_COUNT],
    pub(crate) blob_mage: Option<AttackStats>,
    pub(crate) blob_range: Option<AttackStats>,
    pub(crate) blob_melee: Option<AttackStats>,
}

impl MonsterAttackTable {
    pub(crate) fn resolve(&self, mob_type: MobType, style: Style) -> Option<AttackStats> {
        let idx = mob_type.idx();
        if style == Style::Melee {
            if let Some(stats) = self.melee[idx] {
                return Some(stats);
            }
        }
        if mob_type == MobType::Blob {
            return match style {
                Style::Magic => self.blob_mage,
                Style::Range => self.blob_range,
                Style::Melee => self.blob_melee.or(self.blob_range),
                Style::Blob => self.blob_range,
            };
        }
        self.normal[idx]
    }
}

#[derive(Clone)]
pub(crate) struct Loadout {
    pub(crate) atk_speed: i32,
    pub(crate) max_hit: i32,
    pub(crate) range: i32,
    pub(crate) starting_hp: i32,
    pub(crate) has_recoil: bool,
    pub(crate) has_ring_recoil: bool,
    pub(crate) has_echo_boots: bool,
    pub(crate) is_blood_barrage: bool,
    pub(crate) has_blood_sceptre: bool,
    pub(crate) player_acc: [[f64; 2]; MOB_TYPE_COUNT],
    pub(crate) monster_atk_fast: [[Option<AttackStats>; 3]; MOB_TYPE_COUNT],
}

use crate::engine::{MobType, Style, MAX_MOB_IDS};

#[derive(Clone)]
pub(crate) struct AttackEvent {
    pub(crate) tick: i32,
    pub(crate) mob_id: u16,
    pub(crate) target_mob_id: u16,
    pub(crate) mob_type: u8,
    pub(crate) style: u8,
    pub(crate) is_scan: bool,
    pub(crate) scan_tick: i32,
    pub(crate) acc_roll: f64,
    pub(crate) dmg_roll: f64,
    pub(crate) dist_at_fire: i16,
    pub(crate) hit_tick: i16,
    pub(crate) is_player_attack: bool,
    pub(crate) player_dmg: i32,
    pub(crate) is_revive: bool,
    pub(crate) revive_hp: i16,
}

pub(crate) const EVENT_NONE_ID: u16 = u16::MAX;
pub(crate) const EVENT_NONE_CODE: u8 = u8::MAX;
pub(crate) const EVENT_NONE_I16: i16 = i16::MIN;

pub(crate) fn event_id(value: Option<usize>) -> u16 {
    value.map(|v| v as u16).unwrap_or(EVENT_NONE_ID)
}

pub(crate) fn event_i16(value: Option<i32>) -> i16 {
    value.map(|v| v as i16).unwrap_or(EVENT_NONE_I16)
}

pub(crate) fn event_mob_type(value: Option<MobType>) -> u8 {
    value.map(|v| v.idx() as u8).unwrap_or(EVENT_NONE_CODE)
}

pub(crate) fn event_style(value: Option<Style>) -> u8 {
    match value {
        Some(Style::Magic) => 0,
        Some(Style::Range) => 1,
        Some(Style::Melee) => 2,
        Some(Style::Blob) => 3,
        None => EVENT_NONE_CODE,
    }
}

pub(crate) fn decode_mob_type(value: u8) -> Option<MobType> {
    match value {
        0 => Some(MobType::Mager),
        1 => Some(MobType::Ranger),
        2 => Some(MobType::Meleer),
        3 => Some(MobType::Blob),
        4 => Some(MobType::Bat),
        5 => Some(MobType::Nibbler),
        6 => Some(MobType::BlobletMage),
        7 => Some(MobType::BlobletRange),
        8 => Some(MobType::BlobletMelee),
        _ => None,
    }
}

pub(crate) fn decode_style(value: u8) -> Option<Style> {
    match value {
        0 => Some(Style::Magic),
        1 => Some(Style::Range),
        2 => Some(Style::Melee),
        3 => Some(Style::Blob),
        _ => None,
    }
}

impl AttackEvent {
    pub(crate) fn mob_id(&self) -> Option<usize> {
        (self.mob_id != EVENT_NONE_ID).then_some(self.mob_id as usize)
    }

    pub(crate) fn target_mob_id(&self) -> Option<usize> {
        (self.target_mob_id != EVENT_NONE_ID).then_some(self.target_mob_id as usize)
    }

    pub(crate) fn mob_type(&self) -> Option<MobType> {
        decode_mob_type(self.mob_type)
    }

    pub(crate) fn style(&self) -> Option<Style> {
        decode_style(self.style)
    }

    pub(crate) fn dist_at_fire(&self) -> Option<i32> {
        (self.dist_at_fire != EVENT_NONE_I16).then_some(self.dist_at_fire as i32)
    }

    pub(crate) fn hit_tick(&self) -> Option<i32> {
        (self.hit_tick != EVENT_NONE_I16).then_some(self.hit_tick as i32)
    }

    pub(crate) fn revive_hp(&self) -> Option<i32> {
        (self.revive_hp != EVENT_NONE_I16).then_some(self.revive_hp as i32)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct MobInit {
    pub(crate) hp: i32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SimStatus {
    Complete,
    Trapped,
    Invalid,
    Timeout,
}

pub(crate) struct SimResult {
    pub(crate) attack_start: usize,
    pub(crate) attack_end: usize,
    pub(crate) completed_tick: i32,
    pub(crate) status: SimStatus,
    pub(crate) mob_init_hp: [Option<MobInit>; MAX_MOB_IDS],
    pub(crate) mob_init_len: usize,
}

impl SimResult {
    pub(crate) fn attacks<'a>(&self, attack_log: &'a [AttackEvent]) -> &'a [AttackEvent] {
        &attack_log[self.attack_start..self.attack_end]
    }

    pub(crate) fn mob_init_hp(&self) -> &[Option<MobInit>] {
        &self.mob_init_hp[..self.mob_init_len]
    }
}

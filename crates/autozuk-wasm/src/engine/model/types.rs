use crate::engine::MOB_TYPE_COUNT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MobType {
    Mager,
    Ranger,
    Meleer,
    Blob,
    Bat,
    Nibbler,
    BlobletMage,
    BlobletRange,
    BlobletMelee,
}

impl MobType {
    pub(crate) fn idx(self) -> usize {
        match self {
            MobType::Mager => 0,
            MobType::Ranger => 1,
            MobType::Meleer => 2,
            MobType::Blob => 3,
            MobType::Bat => 4,
            MobType::Nibbler => 5,
            MobType::BlobletMage => 6,
            MobType::BlobletRange => 7,
            MobType::BlobletMelee => 8,
        }
    }

    pub(crate) fn from_code(ch: char) -> Option<Option<Self>> {
        match ch {
            'M' => Some(Some(MobType::Mager)),
            'R' => Some(Some(MobType::Ranger)),
            'X' => Some(Some(MobType::Meleer)),
            'B' => Some(Some(MobType::Blob)),
            'Y' => Some(Some(MobType::Bat)),
            'O' => Some(None),
            _ => None,
        }
    }

    pub(crate) fn js_key(self) -> &'static str {
        match self {
            MobType::Mager => "mager",
            MobType::Ranger => "ranger",
            MobType::Meleer => "meleer",
            MobType::Blob => "blob",
            MobType::Bat => "bat",
            MobType::Nibbler => "nibbler",
            MobType::BlobletMage => "blobletMage",
            MobType::BlobletRange => "blobletRange",
            MobType::BlobletMelee => "blobletMelee",
        }
    }

    pub(crate) fn all() -> [Self; MOB_TYPE_COUNT] {
        [
            MobType::Mager,
            MobType::Ranger,
            MobType::Meleer,
            MobType::Blob,
            MobType::Bat,
            MobType::Nibbler,
            MobType::BlobletMage,
            MobType::BlobletRange,
            MobType::BlobletMelee,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Style {
    Magic,
    Range,
    Melee,
    Blob,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Prayer {
    Mage,
    Range,
    Melee,
}

impl Prayer {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Prayer::Mage => "mage",
            Prayer::Range => "range",
            Prayer::Melee => "melee",
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct PillarConfig {
    pub(crate) s: bool,
    pub(crate) w: bool,
    pub(crate) n: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[derive(Clone, Copy)]
pub(crate) struct Entity {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) size: i32,
}

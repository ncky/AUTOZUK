pub(crate) const ARENA_X_MIN: i32 = 1;
pub(crate) const ARENA_X_MAX: i32 = 29;
pub(crate) const ARENA_Y_MIN: i32 = 1;
pub(crate) const ARENA_Y_MAX: i32 = 30;
pub(crate) const ARENA_W: usize = (ARENA_X_MAX - ARENA_X_MIN + 1) as usize;
pub(crate) const ARENA_H: usize = (ARENA_Y_MAX - ARENA_Y_MIN + 1) as usize;
pub(crate) const ARENA_CELLS: usize = ARENA_W * ARENA_H;
pub(crate) const LOS_WORDS_PER_ROW: usize = ARENA_CELLS.div_ceil(64);
pub(crate) const NPC_LOS_TABLES: usize = 4;
pub(crate) const PLAYER_LOS_TABLES: usize = 12;
pub(crate) const MOB_TYPE_COUNT: usize = 9;
pub(crate) const MAX_MOB_IDS: usize = 64;
pub(crate) const DAMAGE_BUCKETS: usize = 20;
pub(crate) const DEATH_ANIM_TICKS: i32 = 3;
pub(crate) const BFS_DIRS: [(i32, i32); 8] = [
    (-1, 0),
    (1, 0),
    (0, 1),
    (0, -1),
    (-1, 1),
    (1, 1),
    (-1, -1),
    (1, -1),
];
pub(crate) const BLOOD_BARRAGE_AOE_OFFSETS: [(i32, i32); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];
pub(crate) const SPAWN_LOCATIONS: [(i32, i32); 9] = [
    (2, 6),
    (23, 6),
    (4, 12),
    (24, 13),
    (17, 18),
    (6, 24),
    (24, 26),
    (2, 29),
    (16, 29),
];

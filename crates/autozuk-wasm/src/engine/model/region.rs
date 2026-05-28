use crate::engine::{
    arena_idx, arena_point, npc_los_table_idx, player_los_table_idx, raycast, Entity, Point,
    ARENA_CELLS, LOS_WORDS_PER_ROW, NPC_LOS_TABLES, PLAYER_LOS_TABLES,
};

pub(crate) struct Region {
    pub(crate) entities: Vec<Entity>,
    pub(crate) pillars: Vec<Entity>,
    pub(crate) blocked: [u8; 4096],
    pub(crate) los: Vec<u64>,
    pub(crate) npc_los: [Vec<u64>; NPC_LOS_TABLES],
    pub(crate) player_los: [Vec<u64>; PLAYER_LOS_TABLES],
    pub(crate) path_next: Vec<u16>,
}

impl Region {
    pub(crate) fn tile_los(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
        let (Some(a), Some(b)) = (arena_idx(x1, y1), arena_idx(x2, y2)) else {
            return raycast(self, x1, y1, x2, y2);
        };
        let word = a * LOS_WORDS_PER_ROW + (b >> 6);
        ((self.los[word] >> (b & 63)) & 1) != 0
    }

    pub(crate) fn path_next_step(&self, sx: i32, sy: i32, tx: i32, ty: i32) -> Option<Point> {
        let (Some(src), Some(dst)) = (arena_idx(sx, sy), arena_idx(tx, ty)) else {
            return None;
        };
        let next = self.path_next[src * ARENA_CELLS + dst];
        if next == 0 {
            return None;
        }
        Some(arena_point((next - 1) as usize))
    }

    pub(crate) fn npc_has_los(
        &self,
        size: i32,
        range: i32,
        mx: i32,
        my: i32,
        px: i32,
        py: i32,
    ) -> Option<bool> {
        let (Some(mob_idx), Some(player_idx)) = (arena_idx(mx, my), arena_idx(px, py)) else {
            return None;
        };
        let table_idx = npc_los_table_idx(size, range)?;
        let word = mob_idx * LOS_WORDS_PER_ROW + (player_idx >> 6);
        Some(((self.npc_los[table_idx][word] >> (player_idx & 63)) & 1) != 0)
    }

    pub(crate) fn player_has_los(
        &self,
        size: i32,
        range: i32,
        mx: i32,
        my: i32,
        px: i32,
        py: i32,
    ) -> Option<bool> {
        let (Some(mob_idx), Some(player_idx)) = (arena_idx(mx, my), arena_idx(px, py)) else {
            return None;
        };
        let table_idx = player_los_table_idx(size, range)?;
        let word = mob_idx * LOS_WORDS_PER_ROW + (player_idx >> 6);
        Some(((self.player_los[table_idx][word] >> (player_idx & 63)) & 1) != 0)
    }
}

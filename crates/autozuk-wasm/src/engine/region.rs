use crate::engine::{
    arena_idx, arena_point, chebyshev, closest_tile_to, collision_math, key, sign, Entity, Mob,
    MobType, PillarConfig, Player, Point, Region, ARENA_CELLS, ARENA_X_MAX, ARENA_X_MIN,
    ARENA_Y_MAX, ARENA_Y_MIN, BFS_DIRS, LOS_WORDS_PER_ROW, NPC_LOS_TABLES, PLAYER_LOS_TABLES,
    REGION_CACHE,
};

pub(crate) fn has_line_of_sight(
    region: &Region,
    source: Point,
    target: Point,
    size: i32,
    range: i32,
    is_npc: bool,
) -> bool {
    let bl = &region.blocked;
    if bl[key(source.x, source.y)] != 0 || bl[key(target.x, target.y)] != 0 {
        return false;
    }
    if collision_math(source.x, source.y, size, target.x, target.y, 1) {
        return false;
    }
    if range == 1 {
        let dx = target.x - source.x;
        let dy = target.y - source.y;
        return (dx < size && dx >= 0 && (dy == 1 || dy == -size))
            || (dy > -size && dy <= 0 && (dx == -1 || dx == size));
    }
    if is_npc {
        let closest = Point {
            x: source.x.max(target.x.min(source.x + size - 1)),
            y: (source.y - size + 1).max(target.y.min(source.y)),
        };
        return has_line_of_sight(region, target, closest, 1, range, false);
    }
    if (target.x - source.x).abs() > range || (target.y - source.y).abs() > range {
        return false;
    }
    region.tile_los(source.x, source.y, target.x, target.y)
}

pub(crate) fn raycast(region: &Region, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    raycast_blocked(&region.blocked, x1, y1, x2, y2)
}

pub(crate) fn raycast_blocked(bl: &[u8; 4096], x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let dx_abs = dx.abs();
    let dy_abs = dy.abs();
    if dx_abs == 0 && dy_abs == 0 {
        return true;
    }
    if dx_abs > dy_abs {
        let x_inc = if dx > 0 { 1 } else { -1 };
        let slope = (dy << 16) / dx_abs;
        let mut y = (y1 << 16) + 0x8000;
        if dy < 0 {
            y -= 1;
        }
        let mut x_tile = x1;
        while x_tile != x2 {
            x_tile += x_inc;
            let y_tile = y >> 16;
            if bl[key(x_tile, y_tile)] != 0 {
                return false;
            }
            y += slope;
            let ny = y >> 16;
            if ny != y_tile && bl[key(x_tile, ny)] != 0 {
                return false;
            }
        }
    } else {
        let y_inc = if dy > 0 { 1 } else { -1 };
        let slope = (dx << 16) / dy_abs;
        let mut x = (x1 << 16) + 0x8000;
        if dx < 0 {
            x -= 1;
        }
        let mut y_tile = y1;
        while y_tile != y2 {
            y_tile += y_inc;
            let x_tile = x >> 16;
            if bl[key(x_tile, y_tile)] != 0 {
                return false;
            }
            x += slope;
            let nx = x >> 16;
            if nx != x_tile && bl[key(nx, y_tile)] != 0 {
                return false;
            }
        }
    }
    true
}

pub(crate) fn mob_has_los(region: &Region, mob: &Mob, player: &Player) -> bool {
    if mob.range == 1 {
        return is_within_melee_range(mob, player.x, player.y);
    }
    if let Some(has_los) = region.npc_has_los(mob.size, mob.range, mob.x, mob.y, player.x, player.y)
    {
        return has_los;
    }
    let bl = &region.blocked;
    if bl[key(mob.x, mob.y)] != 0 || bl[key(player.x, player.y)] != 0 {
        return false;
    }
    if collision_math(mob.x, mob.y, mob.size, player.x, player.y, 1) {
        return false;
    }
    let tx = mob.x.max(player.x.min(mob.x + mob.size - 1));
    let ty = (mob.y - mob.size + 1).max(player.y.min(mob.y));
    if bl[key(tx, ty)] != 0 {
        return false;
    }
    if (tx - player.x).abs() > mob.range || (ty - player.y).abs() > mob.range {
        return false;
    }
    region.tile_los(player.x, player.y, tx, ty)
}

pub(crate) fn player_has_los(region: &Region, px: i32, py: i32, mob: &Mob, range: i32) -> bool {
    if let Some(has_los) = region.player_has_los(mob.size, range, mob.x, mob.y, px, py) {
        return has_los;
    }
    let cp = closest_tile_to(mob, px, py);
    let bl = &region.blocked;
    if bl[key(px, py)] != 0 || bl[key(cp.x, cp.y)] != 0 {
        return false;
    }
    if px == cp.x && py == cp.y {
        return false;
    }
    if (cp.x - px).abs() > range || (cp.y - py).abs() > range {
        return false;
    }
    region.tile_los(px, py, cp.x, cp.y)
}

pub(crate) fn is_within_melee_range(mob: &Mob, tx: i32, ty: i32) -> bool {
    let dx = tx - mob.x;
    let dy = ty - mob.y;
    let s = mob.size;
    (dx < s && dx >= 0 && (dy == 1 || dy == -s)) || (dy > -s && dy <= 0 && (dx == -1 || dx == s))
}

pub(crate) fn is_within_secondary_melee_range(mob: &Mob, player: &Player) -> bool {
    let ct = closest_tile_to(mob, player.x, player.y);
    chebyshev(player.x, player.y, ct.x, ct.y) == 1
}

pub(crate) fn can_use_secondary_melee(mob: &Mob, player: &Player) -> bool {
    matches!(
        mob.mob_type,
        MobType::Mager | MobType::Ranger | MobType::Blob
    ) && is_within_secondary_melee_range(mob, player)
}

pub(crate) fn get_closest_face_tile(mob: &Mob, px: i32, py: i32, region: &Region) -> Option<Point> {
    let s = mob.size;
    let mx = mob.x;
    let my = mob.y;
    let mut best_dist = i32::MAX;
    let mut best_man = i32::MAX;
    let mut best_tile: Option<(Point, bool)> = None;

    let mut check = |x: i32, y: i32, is_ns: bool| {
        if region.blocked[key(x, y)] != 0 {
            return;
        }
        let d = chebyshev(px, py, x, y);
        let m = (px - x).abs() + (py - y).abs();
        let best_is_ns = best_tile.map(|(_, v)| v).unwrap_or(false);
        if d < best_dist
            || (d == best_dist && is_ns && best_tile.is_some() && !best_is_ns)
            || (d == best_dist
                && is_ns == best_tile.map(|(_, v)| v).unwrap_or(is_ns)
                && m < best_man)
        {
            best_dist = d;
            best_man = m;
            best_tile = Some((Point { x, y }, is_ns));
        }
    };

    for x in mx..mx + s {
        check(x, my + 1, true);
    }
    for x in mx..mx + s {
        check(x, my - s, true);
    }
    if mx > ARENA_X_MIN {
        for y in my - s + 1..=my {
            check(mx - 1, y, false);
        }
    }
    if mx + s <= ARENA_X_MAX {
        for y in my - s + 1..=my {
            check(mx + s, y, false);
        }
    }
    best_tile.map(|(p, _)| p)
}

#[derive(Clone, Copy)]
pub(crate) struct BfsNode {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) parent: Option<usize>,
}

pub(crate) fn player_bfs(sx: i32, sy: i32, tx: i32, ty: i32, region: &Region) -> Option<Point> {
    if sx == tx && sy == ty {
        return None;
    }
    if let Some(step) = region.path_next_step(sx, sy, tx, ty) {
        return Some(step);
    }
    let bl = &region.blocked;
    let mut visited = [0u8; 4096];
    visited[key(sx, sy)] = 1;
    let mut queue = Vec::with_capacity(256);
    queue.push(BfsNode {
        x: sx,
        y: sy,
        parent: None,
    });
    let mut qi = 0usize;
    while qi < queue.len() && queue.len() < 2000 {
        let node = queue[qi];
        if node.x == tx && node.y == ty {
            let mut step_idx = qi;
            while let Some(parent_idx) = queue[step_idx].parent {
                let parent = queue[parent_idx];
                if parent.x == sx && parent.y == sy {
                    break;
                }
                step_idx = parent_idx;
            }
            let step = queue[step_idx];
            return Some(Point {
                x: step.x,
                y: step.y,
            });
        }
        for (d, (dx, dy)) in BFS_DIRS.iter().enumerate() {
            let nx = node.x + dx;
            let ny = node.y + dy;
            if !(ARENA_X_MIN..=ARENA_X_MAX).contains(&nx)
                || !(ARENA_Y_MIN..=ARENA_Y_MAX).contains(&ny)
            {
                continue;
            }
            let k = key(nx, ny);
            if visited[k] != 0 || bl[k] != 0 {
                continue;
            }
            if d >= 4 && (bl[key(node.x + dx, node.y)] != 0 || bl[key(node.x, node.y + dy)] != 0) {
                continue;
            }
            visited[k] = 1;
            queue.push(BfsNode {
                x: nx,
                y: ny,
                parent: Some(qi),
            });
        }
        qi += 1;
    }

    let dx = sign(tx - sx);
    let dy = sign(ty - sy);
    let nx = sx + dx;
    let ny = sy + dy;
    if dx != 0 && dy != 0 {
        if bl[key(nx, ny)] == 0 && bl[key(sx + dx, sy)] == 0 && bl[key(sx, sy + dy)] == 0 {
            return Some(Point { x: nx, y: ny });
        }
        if bl[key(sx + dx, sy)] == 0 {
            return Some(Point { x: sx + dx, y: sy });
        }
        if bl[key(sx, sy + dy)] == 0 {
            return Some(Point { x: sx, y: sy + dy });
        }
    } else if (dx != 0 || dy != 0) && bl[key(nx, ny)] == 0 {
        return Some(Point { x: nx, y: ny });
    }
    None
}

pub(crate) fn osrs_walk_step(sx: i32, sy: i32, tx: i32, ty: i32, region: &Region) -> Option<Point> {
    if sx == tx && sy == ty {
        return None;
    }
    let dx = tx - sx;
    let dy = ty - sy;
    let dx_a = dx.abs();
    let dy_a = dy.abs();
    let xs = sign(dx);
    let ys = sign(dy);
    let (nx, ny) = if dx_a > dy_a {
        (sx + xs, sy)
    } else if dy_a > dx_a {
        (sx, sy + ys)
    } else {
        (sx + xs, sy + ys)
    };
    let bl = &region.blocked;
    if dx_a == dy_a {
        if bl[key(nx, ny)] == 0 && bl[key(sx + xs, sy)] == 0 && bl[key(sx, sy + ys)] == 0 {
            return Some(Point { x: nx, y: ny });
        }
    } else if bl[key(nx, ny)] == 0 {
        return Some(Point { x: nx, y: ny });
    }
    player_bfs(sx, sy, tx, ty, region)
}

pub(crate) fn create_region(pillars: PillarConfig) -> Region {
    let mut entities = Vec::new();
    for x in ARENA_X_MIN - 1..=ARENA_X_MAX + 1 {
        entities.push(Entity {
            x,
            y: ARENA_Y_MIN - 1,
            size: 1,
        });
        entities.push(Entity {
            x,
            y: ARENA_Y_MAX + 1,
            size: 1,
        });
    }
    for y in ARENA_Y_MIN..=ARENA_Y_MAX {
        entities.push(Entity {
            x: ARENA_X_MIN - 1,
            y,
            size: 1,
        });
        entities.push(Entity {
            x: ARENA_X_MAX + 1,
            y,
            size: 1,
        });
    }

    let mut pillar_entities = Vec::new();
    for (enabled, x, y) in [(pillars.s, 11, 24), (pillars.w, 1, 10), (pillars.n, 18, 8)] {
        if enabled {
            let p = Entity { x, y, size: 3 };
            pillar_entities.push(p);
            entities.push(p);
        }
    }

    let mut blocked = [0u8; 4096];
    for e in &entities {
        let ex1 = e.x + e.size - 1;
        let ey0 = e.y - e.size + 1;
        for ex in e.x..=ex1 {
            for ey in ey0..=e.y {
                blocked[key(ex, ey)] = 1;
            }
        }
    }
    let los = build_los_graph(&blocked);
    let npc_los = build_npc_los_tables(&blocked, &los);
    let player_los = build_player_los_tables(&blocked, &los);
    let path_next = build_path_next_graph(&blocked);

    Region {
        entities,
        pillars: pillar_entities,
        blocked,
        los,
        npc_los,
        player_los,
        path_next,
    }
}

pub(crate) fn build_los_graph(blocked: &[u8; 4096]) -> Vec<u64> {
    let mut los = vec![0u64; ARENA_CELLS * LOS_WORDS_PER_ROW];
    for from in 0..ARENA_CELLS {
        let a = arena_point(from);
        if blocked[key(a.x, a.y)] != 0 {
            continue;
        }
        let row = from * LOS_WORDS_PER_ROW;
        for to in 0..ARENA_CELLS {
            let b = arena_point(to);
            if blocked[key(b.x, b.y)] != 0 {
                continue;
            }
            if raycast_blocked(blocked, a.x, a.y, b.x, b.y) {
                los[row + (to >> 6)] |= 1u64 << (to & 63);
            }
        }
    }
    los
}

pub(crate) fn graph_tile_los(los: &[u64], from: usize, to: usize) -> bool {
    let word = from * LOS_WORDS_PER_ROW + (to >> 6);
    ((los[word] >> (to & 63)) & 1) != 0
}

pub(crate) fn npc_los_table_idx(size: i32, range: i32) -> Option<usize> {
    match (size, range) {
        (1, 15) => Some(0),
        (2, 4) => Some(1),
        (3, 15) => Some(2),
        (4, 15) => Some(3),
        _ => None,
    }
}

pub(crate) fn build_npc_los_tables(
    blocked: &[u8; 4096],
    tile_los: &[u64],
) -> [Vec<u64>; NPC_LOS_TABLES] {
    [
        build_npc_los_table(blocked, tile_los, 1, 15),
        build_npc_los_table(blocked, tile_los, 2, 4),
        build_npc_los_table(blocked, tile_los, 3, 15),
        build_npc_los_table(blocked, tile_los, 4, 15),
    ]
}

pub(crate) fn build_npc_los_table(
    blocked: &[u8; 4096],
    tile_los: &[u64],
    size: i32,
    range: i32,
) -> Vec<u64> {
    let mut los = vec![0u64; ARENA_CELLS * LOS_WORDS_PER_ROW];
    for mob_idx in 0..ARENA_CELLS {
        let mob = arena_point(mob_idx);
        if blocked[key(mob.x, mob.y)] != 0 {
            continue;
        }
        let row = mob_idx * LOS_WORDS_PER_ROW;
        for player_idx in 0..ARENA_CELLS {
            let player = arena_point(player_idx);
            if blocked[key(player.x, player.y)] != 0 {
                continue;
            }
            if collision_math(mob.x, mob.y, size, player.x, player.y, 1) {
                continue;
            }
            let tx = mob.x.max(player.x.min(mob.x + size - 1));
            let ty = (mob.y - size + 1).max(player.y.min(mob.y));
            let Some(target_idx) = arena_idx(tx, ty) else {
                continue;
            };
            if blocked[key(tx, ty)] != 0 {
                continue;
            }
            if (tx - player.x).abs() > range || (ty - player.y).abs() > range {
                continue;
            }
            if graph_tile_los(tile_los, player_idx, target_idx) {
                los[row + (player_idx >> 6)] |= 1u64 << (player_idx & 63);
            }
        }
    }
    los
}

pub(crate) fn player_los_table_idx(size: i32, range: i32) -> Option<usize> {
    match (size, range) {
        (1, 5) => Some(0),
        (2, 5) => Some(1),
        (3, 5) => Some(2),
        (4, 5) => Some(3),
        (1, 8) => Some(4),
        (2, 8) => Some(5),
        (3, 8) => Some(6),
        (4, 8) => Some(7),
        (1, 10) => Some(8),
        (2, 10) => Some(9),
        (3, 10) => Some(10),
        (4, 10) => Some(11),
        _ => None,
    }
}

pub(crate) fn build_player_los_tables(
    blocked: &[u8; 4096],
    tile_los: &[u64],
) -> [Vec<u64>; PLAYER_LOS_TABLES] {
    [
        build_player_los_table(blocked, tile_los, 1, 5),
        build_player_los_table(blocked, tile_los, 2, 5),
        build_player_los_table(blocked, tile_los, 3, 5),
        build_player_los_table(blocked, tile_los, 4, 5),
        build_player_los_table(blocked, tile_los, 1, 8),
        build_player_los_table(blocked, tile_los, 2, 8),
        build_player_los_table(blocked, tile_los, 3, 8),
        build_player_los_table(blocked, tile_los, 4, 8),
        build_player_los_table(blocked, tile_los, 1, 10),
        build_player_los_table(blocked, tile_los, 2, 10),
        build_player_los_table(blocked, tile_los, 3, 10),
        build_player_los_table(blocked, tile_los, 4, 10),
    ]
}

pub(crate) fn build_player_los_table(
    blocked: &[u8; 4096],
    tile_los: &[u64],
    size: i32,
    range: i32,
) -> Vec<u64> {
    let mut los = vec![0u64; ARENA_CELLS * LOS_WORDS_PER_ROW];
    for mob_idx in 0..ARENA_CELLS {
        let mob = arena_point(mob_idx);
        let row = mob_idx * LOS_WORDS_PER_ROW;
        for player_idx in 0..ARENA_CELLS {
            let player = arena_point(player_idx);
            if blocked[key(player.x, player.y)] != 0 {
                continue;
            }
            let tx = mob.x.max(player.x.min(mob.x + size - 1));
            let ty = (mob.y - size + 1).max(player.y.min(mob.y));
            if blocked[key(tx, ty)] != 0 {
                continue;
            }
            if player.x == tx && player.y == ty {
                continue;
            }
            if (tx - player.x).abs() > range || (ty - player.y).abs() > range {
                continue;
            }
            let Some(target_idx) = arena_idx(tx, ty) else {
                continue;
            };
            if graph_tile_los(tile_los, player_idx, target_idx) {
                los[row + (player_idx >> 6)] |= 1u64 << (player_idx & 63);
            }
        }
    }
    los
}

pub(crate) fn build_path_next_graph(blocked: &[u8; 4096]) -> Vec<u16> {
    let mut table = vec![0u16; ARENA_CELLS * ARENA_CELLS];
    let mut visited = [0u8; ARENA_CELLS];
    let mut first_step = [0u16; ARENA_CELLS];
    let mut queue = [0usize; ARENA_CELLS];

    for src in 0..ARENA_CELLS {
        let src_point = arena_point(src);
        if blocked[key(src_point.x, src_point.y)] != 0 {
            continue;
        }
        visited.fill(0);
        first_step.fill(0);
        let mut head = 0usize;
        let mut tail = 0usize;
        visited[src] = 1;
        queue[tail] = src;
        tail += 1;

        while head < tail {
            let current = queue[head];
            head += 1;
            let point = arena_point(current);
            for (dir_idx, (dx, dy)) in BFS_DIRS.iter().enumerate() {
                let nx = point.x + dx;
                let ny = point.y + dy;
                let Some(next_idx) = arena_idx(nx, ny) else {
                    continue;
                };
                if visited[next_idx] != 0 || blocked[key(nx, ny)] != 0 {
                    continue;
                }
                if dir_idx >= 4
                    && (blocked[key(point.x + dx, point.y)] != 0
                        || blocked[key(point.x, point.y + dy)] != 0)
                {
                    continue;
                }
                visited[next_idx] = 1;
                first_step[next_idx] = if current == src {
                    (next_idx + 1) as u16
                } else {
                    first_step[current]
                };
                queue[tail] = next_idx;
                tail += 1;
            }
        }

        let row = src * ARENA_CELLS;
        table[row..row + ARENA_CELLS].copy_from_slice(&first_step);
    }
    table
}

pub(crate) fn pillar_cache_key(pillars: PillarConfig) -> u8 {
    (pillars.s as u8) | ((pillars.w as u8) << 1) | ((pillars.n as u8) << 2)
}

pub(crate) fn with_cached_region<R>(pillars: PillarConfig, f: impl FnOnce(&Region) -> R) -> R {
    let cache_key = pillar_cache_key(pillars);
    REGION_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let idx = if let Some(idx) = cache.iter().position(|(key, _)| *key == cache_key) {
            idx
        } else {
            cache.push((cache_key, create_region(pillars)));
            cache.len() - 1
        };
        f(&cache[idx].1)
    })
}

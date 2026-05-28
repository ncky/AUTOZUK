use crate::engine::{
    collision_math, has_line_of_sight, is_within_melee_range, Mob, MobProjectileQueue, MobType,
    Player, Point, Region,
};

pub(crate) fn check_tile_excluded(x: i32, y: i32, mobs: &[Mob], region: &Region) -> bool {
    for p in &region.pillars {
        if collision_math(p.x, p.y, p.size, x, y, 1) {
            return true;
        }
    }
    for m in mobs {
        if collision_math(m.x, m.y, m.size, x, y, 1) {
            return true;
        }
    }
    let fake_player = Player {
        x,
        y,
        attack_delay: 0,
        range: 1,
        incoming_projectiles: MobProjectileQueue::new(),
        auto_retaliate: true,
        last_hit: true,
        aggro: None,
        last_attacker: None,
    };
    let mut has_mager = false;
    let mut has_ranger = false;
    let mut has_meleer = false;
    for m in mobs {
        if matches!(
            m.mob_type,
            MobType::Mager | MobType::Ranger | MobType::Meleer
        ) {
            let has = if m.range == 1 {
                is_within_melee_range(m, fake_player.x, fake_player.y)
            } else {
                has_line_of_sight(
                    region,
                    Point { x: m.x, y: m.y },
                    Point { x, y },
                    m.size,
                    m.range,
                    true,
                )
            };
            if has {
                match m.mob_type {
                    MobType::Mager => has_mager = true,
                    MobType::Ranger => has_ranger = true,
                    MobType::Meleer => has_meleer = true,
                    _ => {}
                }
            }
        }
    }
    [has_mager, has_ranger, has_meleer]
        .into_iter()
        .filter(|has| *has)
        .count()
        >= 2
}

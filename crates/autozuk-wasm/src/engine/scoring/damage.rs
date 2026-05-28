use crate::engine::{
    AttackEvent, DamageResult, Loadout, MobInit, PendingMobRemoval, PendingMobRemovalQueue,
    PendingPlayerHit, PendingPlayerHitQueue, PendingRecoil, PendingRecoilQueue, Prayer, Style,
    MAX_MOB_IDS, MOB_TYPE_COUNT,
};

pub(crate) fn loadout_blood_heal_rate(loadout: &Loadout) -> f64 {
    if loadout.has_blood_sceptre {
        0.275
    } else {
        0.25
    }
}

pub(crate) fn loadout_blood_max_hp(loadout: &Loadout) -> i32 {
    if loadout.is_blood_barrage && loadout.has_blood_sceptre {
        108
    } else {
        99
    }
}

pub(crate) fn attack_style_idx(style: Style) -> usize {
    match style {
        Style::Magic => 0,
        Style::Range | Style::Blob => 1,
        Style::Melee => 2,
    }
}

pub(crate) fn prayer_slot(tick: i32) -> usize {
    (tick as usize) & 3
}

pub(crate) fn calc_sim_damage(
    attacks: &[AttackEvent],
    prayer_seq: &[Prayer; 4],
    loadout: &Loadout,
    mob_init_hp: &[Option<MobInit>],
) -> DamageResult {
    let has_recoil = loadout.has_recoil && !mob_init_hp.is_empty();
    if !has_recoil && !loadout.is_blood_barrage {
        return calc_sim_damage_no_recoil(attacks, prayer_seq, loadout);
    }
    let start_hp = loadout.starting_hp;
    let mut hp = start_hp;
    let max_hp = start_hp.max(loadout_blood_max_hp(loadout));
    let mut min_hp = start_hp;
    let mut died = false;
    let mut mob_hp = [0i32; MAX_MOB_IDS];
    let mut mob_exists = [false; MAX_MOB_IDS];
    if has_recoil {
        for (id, init) in mob_init_hp.iter().enumerate() {
            if id >= MAX_MOB_IDS {
                break;
            }
            if let Some(init) = init {
                mob_hp[id] = init.hp;
                mob_exists[id] = true;
            }
        }
    }
    let mut dead_mobs = [false; MAX_MOB_IDS];
    let mut echo_boots_cooldown = 0;
    let mut pending_recoil = PendingRecoilQueue::new();
    let mut pending_player_hits = PendingPlayerHitQueue::new();
    let mut pending_mob_removals = PendingMobRemovalQueue::new();

    for atk in attacks {
        apply_pending_player_hits(
            atk.tick,
            has_recoil,
            &mut pending_player_hits,
            &mut mob_hp,
            &mob_exists,
            &mut dead_mobs,
            &mut pending_mob_removals,
        );
        apply_pending_deaths(
            atk.tick,
            has_recoil,
            &mut pending_mob_removals,
            &mut dead_mobs,
        );
        if has_recoil && !pending_recoil.is_empty() {
            let mut i = pending_recoil.len();
            while i > 0 {
                i -= 1;
                let r = pending_recoil.get(i);
                if r.tick <= atk.tick {
                    if !dead_mobs.get(r.mob_id).copied().unwrap_or(false)
                        && mob_exists.get(r.mob_id).copied().unwrap_or(false)
                        && mob_hp[r.mob_id] > 0
                    {
                        mob_hp[r.mob_id] -= r.damage;
                        if mob_hp[r.mob_id] <= 0 {
                            schedule_mob_removal(
                                r.mob_id,
                                r.tick,
                                &dead_mobs,
                                &mut pending_mob_removals,
                            );
                        }
                    }
                    pending_recoil.swap_remove(i);
                }
            }
            apply_pending_deaths(
                atk.tick,
                has_recoil,
                &mut pending_mob_removals,
                &mut dead_mobs,
            );
        }

        if atk.is_player_attack {
            if loadout.is_blood_barrage && atk.player_dmg > 0 && hp < max_hp {
                hp = max_hp.min(
                    hp + ((atk.player_dmg as f64) * loadout_blood_heal_rate(loadout)).floor()
                        as i32,
                );
            }
            if has_recoil {
                if let Some(mob_id) = atk.target_mob_id() {
                    if mob_exists.get(mob_id).copied().unwrap_or(false) {
                        pending_player_hits.push(PendingPlayerHit {
                            tick: atk.hit_tick().unwrap_or(atk.tick),
                            mob_id,
                            damage: atk.player_dmg,
                        });
                    }
                }
            }
            continue;
        }

        if atk.is_revive {
            if has_recoil {
                if let Some(mob_id) = atk.mob_id() {
                    if mob_id >= dead_mobs.len() {
                        continue;
                    }
                    dead_mobs[mob_id] = false;
                    mob_hp[mob_id] = atk
                        .revive_hp()
                        .or_else(|| mob_init_hp.get(mob_id).and_then(|m| m.map(|v| v.hp / 2)))
                        .unwrap_or(0);
                    pending_mob_removals.retain(|r| r.mob_id != mob_id);
                    pending_player_hits.retain(|h| h.mob_id != mob_id);
                    pending_recoil.retain(|r| r.mob_id != mob_id);
                }
            }
            continue;
        }

        if atk.is_scan {
            continue;
        }
        let Some(mob_id) = atk.mob_id() else {
            continue;
        };
        if has_recoil && dead_mobs.get(mob_id).copied().unwrap_or(false) {
            continue;
        }
        let pray_on_tick = prayer_seq[prayer_slot(atk.tick)];
        let atk_style = match atk.style() {
            Some(style) => style,
            None => {
                let pray_on_scan = prayer_seq[prayer_slot(atk.scan_tick)];
                if pray_on_scan == Prayer::Mage {
                    Style::Range
                } else {
                    Style::Magic
                }
            }
        };
        let blocked = (atk_style == Style::Magic && pray_on_tick == Prayer::Mage)
            || (atk_style == Style::Range && pray_on_tick == Prayer::Range)
            || (atk_style == Style::Melee && pray_on_tick == Prayer::Melee);
        if blocked {
            continue;
        }
        let mob_type_idx = atk.mob_type as usize;
        if mob_type_idx >= MOB_TYPE_COUNT {
            continue;
        };
        let Some(stats) = loadout.monster_atk_fast[mob_type_idx][attack_style_idx(atk_style)]
        else {
            continue;
        };
        if atk.acc_roll < stats.acc {
            let dmg = (atk.dmg_roll * ((stats.max + 1) as f64)).floor() as i32;
            if dmg > 0 {
                hp -= dmg;
                if hp < min_hp {
                    min_hp = hp;
                }
                if has_recoil {
                    let recoil_tick = atk.hit_tick().unwrap_or(atk.tick + 1) + 1;
                    if loadout.has_ring_recoil {
                        let ring_dmg = ((dmg as f64) * 0.1 + 1.0).floor() as i32;
                        pending_recoil.push(PendingRecoil {
                            tick: recoil_tick,
                            mob_id,
                            damage: ring_dmg,
                        });
                    }
                    if loadout.has_echo_boots
                        && atk.dist_at_fire().unwrap_or(99) <= 1
                        && recoil_tick >= echo_boots_cooldown
                    {
                        pending_recoil.push(PendingRecoil {
                            tick: recoil_tick,
                            mob_id,
                            damage: 1,
                        });
                        echo_boots_cooldown = recoil_tick + 4;
                    }
                }
            }
            if hp <= 0 {
                died = true;
                break;
            }
        }
    }

    if has_recoil {
        apply_pending_player_hits(
            i32::MAX,
            has_recoil,
            &mut pending_player_hits,
            &mut mob_hp,
            &mob_exists,
            &mut dead_mobs,
            &mut pending_mob_removals,
        );
        pending_recoil.for_each(|r| {
            if !dead_mobs.get(r.mob_id).copied().unwrap_or(false)
                && mob_exists.get(r.mob_id).copied().unwrap_or(false)
                && mob_hp[r.mob_id] > 0
            {
                mob_hp[r.mob_id] -= r.damage;
                if mob_hp[r.mob_id] <= 0 {
                    schedule_mob_removal(r.mob_id, r.tick, &dead_mobs, &mut pending_mob_removals);
                }
            }
        });
    }

    let damage = if died {
        start_hp
    } else if loadout.is_blood_barrage {
        0.max(start_hp - min_hp)
    } else {
        0.max(start_hp - hp)
    };
    DamageResult { damage, died }
}

pub(crate) fn calc_sim_damage_no_recoil(
    attacks: &[AttackEvent],
    prayer_seq: &[Prayer; 4],
    loadout: &Loadout,
) -> DamageResult {
    let start_hp = loadout.starting_hp;
    let mut hp = start_hp;
    for atk in attacks {
        if atk.is_player_attack || atk.is_revive || atk.is_scan {
            continue;
        }
        if atk.mob_id().is_none() {
            continue;
        }
        let pray_on_tick = prayer_seq[prayer_slot(atk.tick)];
        let atk_style = match atk.style() {
            Some(style) => style,
            None => {
                let pray_on_scan = prayer_seq[prayer_slot(atk.scan_tick)];
                if pray_on_scan == Prayer::Mage {
                    Style::Range
                } else {
                    Style::Magic
                }
            }
        };
        let blocked = (atk_style == Style::Magic && pray_on_tick == Prayer::Mage)
            || (atk_style == Style::Range && pray_on_tick == Prayer::Range)
            || (atk_style == Style::Melee && pray_on_tick == Prayer::Melee);
        if blocked {
            continue;
        }
        let mob_type_idx = atk.mob_type as usize;
        if mob_type_idx >= MOB_TYPE_COUNT {
            continue;
        };
        let Some(stats) = loadout.monster_atk_fast[mob_type_idx][attack_style_idx(atk_style)]
        else {
            continue;
        };
        if atk.acc_roll < stats.acc {
            let dmg = (atk.dmg_roll * ((stats.max + 1) as f64)).floor() as i32;
            if dmg > 0 {
                hp -= dmg;
                if hp <= 0 {
                    return DamageResult {
                        damage: start_hp,
                        died: true,
                    };
                }
            }
        }
    }
    DamageResult {
        damage: 0.max(start_hp - hp),
        died: false,
    }
}

pub(crate) fn apply_pending_deaths(
    current_tick: i32,
    has_recoil: bool,
    pending_mob_removals: &mut PendingMobRemovalQueue,
    dead_mobs: &mut [bool],
) {
    if !has_recoil {
        return;
    }
    let mut i = pending_mob_removals.len();
    while i > 0 {
        i -= 1;
        let r = pending_mob_removals.get(i);
        if r.tick <= current_tick {
            if r.mob_id < dead_mobs.len() {
                dead_mobs[r.mob_id] = true;
            }
            pending_mob_removals.swap_remove(i);
        }
    }
}

pub(crate) fn schedule_mob_removal(
    mob_id: usize,
    hit_tick: i32,
    dead_mobs: &[bool],
    pending_mob_removals: &mut PendingMobRemovalQueue,
) {
    if dead_mobs.get(mob_id).copied().unwrap_or(false) {
        return;
    }
    let remove_tick = hit_tick + 1;
    if let Some(existing) = pending_mob_removals.first_mut(|r| r.mob_id == mob_id) {
        existing.tick = existing.tick.min(remove_tick);
    } else {
        pending_mob_removals.push(PendingMobRemoval {
            tick: remove_tick,
            mob_id,
        });
    }
}

pub(crate) fn apply_pending_player_hits(
    current_tick: i32,
    has_recoil: bool,
    pending_player_hits: &mut PendingPlayerHitQueue,
    mob_hp: &mut [i32],
    mob_exists: &[bool],
    dead_mobs: &mut [bool],
    pending_mob_removals: &mut PendingMobRemovalQueue,
) {
    if !has_recoil || pending_player_hits.is_empty() {
        return;
    }
    let mut i = pending_player_hits.len();
    while i > 0 {
        i -= 1;
        let h = pending_player_hits.get(i);
        if h.tick <= current_tick {
            if !dead_mobs.get(h.mob_id).copied().unwrap_or(false)
                && mob_exists.get(h.mob_id).copied().unwrap_or(false)
                && mob_hp[h.mob_id] > 0
            {
                mob_hp[h.mob_id] -= h.damage;
                if mob_hp[h.mob_id] <= 0 {
                    schedule_mob_removal(h.mob_id, h.tick, dead_mobs, pending_mob_removals);
                }
            }
            pending_player_hits.swap_remove(i);
        }
    }
}

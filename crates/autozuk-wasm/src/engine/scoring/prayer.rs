use crate::engine::{
    calc_sim_damage, prayer_slot, AttackEvent, DamageResult, Loadout, MobType, Prayer, SimResult,
    WaveFlags,
};

pub(crate) fn optimize_prayer(
    results: &[SimResult],
    attack_log: &[AttackEvent],
    flags: WaveFlags,
    loadout: &Loadout,
    best_damages: &mut Vec<DamageResult>,
    candidate_damages: &mut Vec<DamageResult>,
) -> [Prayer; 4] {
    best_damages.clear();
    candidate_damages.clear();
    let best_extra = results.len().saturating_sub(best_damages.capacity());
    if best_extra > 0 {
        best_damages.reserve_exact(best_extra);
    }
    let candidate_extra = results.len().saturating_sub(candidate_damages.capacity());
    if candidate_extra > 0 {
        candidate_damages.reserve_exact(candidate_extra);
    }

    let mut mager_votes = [0i32; 4];
    let mut ranger_votes = [0i32; 4];
    let mut meleer_votes = [0i32; 4];
    for result in results {
        let mut found_mager = false;
        let mut found_ranger = false;
        let mut found_meleer = false;
        for atk in result.attacks(attack_log) {
            if atk.is_scan {
                continue;
            }
            let slot = prayer_slot(atk.tick);
            match atk.mob_type() {
                Some(MobType::Mager) if flags.has_mager && !found_mager => {
                    mager_votes[slot] += 1;
                    found_mager = true;
                }
                Some(MobType::Ranger) if flags.has_ranger && !found_ranger => {
                    ranger_votes[slot] += 1;
                    found_ranger = true;
                }
                Some(MobType::Meleer) if flags.has_meleer && !found_meleer => {
                    meleer_votes[slot] += 1;
                    found_meleer = true;
                }
                _ => {}
            }
        }
    }

    let mut slots: [Option<Prayer>; 4] = [None, None, None, None];
    if flags.has_mager {
        if let Some(slot) = best_vote_slot(&mager_votes) {
            slots[slot] = Some(Prayer::Mage);
        }
    }
    if flags.has_ranger {
        if let Some(slot) = best_vote_slot(&ranger_votes) {
            if slots[slot].is_none() {
                slots[slot] = Some(Prayer::Range);
            }
        }
    }
    if flags.has_meleer {
        if let Some(slot) = best_vote_slot(&meleer_votes) {
            if slots[slot].is_none() {
                slots[slot] = Some(Prayer::Melee);
            }
        }
    }

    let mut unknowns = [0usize; 4];
    let mut unknown_count = 0usize;
    for (idx, p) in slots.iter().enumerate() {
        if p.is_none() {
            unknowns[unknown_count] = idx;
            unknown_count += 1;
        }
    }
    let combos = 1usize << unknown_count;
    let mut best_seq = [Prayer::Mage; 4];
    let mut best_dmg = f64::INFINITY;
    for c in 0..combos {
        let mut seq = [
            slots[0].unwrap_or(Prayer::Mage),
            slots[1].unwrap_or(Prayer::Mage),
            slots[2].unwrap_or(Prayer::Mage),
            slots[3].unwrap_or(Prayer::Mage),
        ];
        for (i, slot) in unknowns.iter().take(unknown_count).enumerate() {
            seq[*slot] = if ((c >> i) & 1) != 0 {
                Prayer::Range
            } else {
                Prayer::Mage
            };
        }
        let mut total = 0.0;
        candidate_damages.clear();
        let cutoff = best_dmg * results.len() as f64;
        let mut pruned = false;
        for result in results {
            let damage = calc_sim_damage(
                result.attacks(attack_log),
                &seq,
                loadout,
                result.mob_init_hp(),
            );
            total += damage.damage as f64;
            candidate_damages.push(damage);
            if total >= cutoff {
                pruned = true;
                break;
            }
        }
        if pruned {
            continue;
        }
        let avg = total / results.len() as f64;
        if avg < best_dmg {
            best_dmg = avg;
            best_seq = seq;
            std::mem::swap(best_damages, candidate_damages);
        }
    }
    best_seq
}

pub(crate) fn best_vote_slot(votes: &[i32; 4]) -> Option<usize> {
    let mut best = None;
    let mut best_count = 0;
    for (idx, count) in votes.iter().enumerate() {
        if *count > best_count {
            best_count = *count;
            best = Some(idx);
        }
    }
    best
}

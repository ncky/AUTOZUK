use crate::engine::State;

pub(crate) fn mob_idx_by_id(state: &State<'_>, id: usize) -> Option<usize> {
    state.id_to_idx.get(id).copied().flatten()
}

pub(crate) fn aggro_is_unusable(state: &State<'_>, aggro: Option<usize>, tick: i32) -> bool {
    match aggro.and_then(|id| mob_idx_by_id(state, id)) {
        None => true,
        Some(idx) => {
            let m = &state.mobs[idx];
            m.dead || (m.dying > 0 && tick > m.dying_start_tick)
        }
    }
}

pub(crate) fn can_set_last_attacker(state: &State<'_>, mob_id: usize) -> bool {
    match state.player.aggro {
        None => true,
        Some(id) if id == mob_id => true,
        Some(id) => mob_idx_by_id(state, id)
            .map(|idx| state.mobs[idx].dead)
            .unwrap_or(true),
    }
}

pub(crate) fn set_player_last_attacker(state: &mut State, mob_id: usize) {
    if can_set_last_attacker(state, mob_id) {
        state.player.last_attacker = Some(mob_id);
    }
}

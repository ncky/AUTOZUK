use crate::engine::{
    hl_init_state, hl_tick, AttackEvent, Loadout, MobType, ParsedSpawnCode, PillarConfig, Point,
    Region, SimResult, SimStatus, State,
};

#[derive(Clone, Copy)]
pub(crate) struct SimRunParams<'a> {
    pub(crate) parsed: &'a ParsedSpawnCode,
    pub(crate) player_pos: Point,
    pub(crate) pillar_config: PillarConfig,
    pub(crate) loadout: &'a Loadout,
    pub(crate) max_ticks: i32,
    pub(crate) region: &'a Region,
    pub(crate) seed: u32,
}

pub(crate) fn hl_run_sim(
    params: SimRunParams<'_>,
    attack_log: &mut Vec<AttackEvent>,
) -> Result<SimResult, String> {
    let attack_start = attack_log.len();

    let mut state = hl_init_state(
        params.parsed,
        params.player_pos,
        params.pillar_config,
        params.loadout,
        params.region,
        params.seed,
        attack_log,
    );
    for _ in 0..params.max_ticks {
        hl_tick(&mut state);
        if state.dead_count == state.mobs.len() {
            let completed_tick = state.tick;
            return Ok(finish_sim_result(
                state,
                attack_start,
                completed_tick,
                SimStatus::Complete,
            ));
        }
        if state.player.aggro.is_none() {
            let mut all_no_los = true;
            let mut trapped_big = [MobType::Mager; 3];
            let mut trapped_big_len = 0usize;
            for m in &state.mobs {
                if m.dead || m.dying > 0 {
                    continue;
                }
                if m.no_los_ticks < 20 {
                    all_no_los = false;
                    break;
                }
                if m.mob_type != MobType::Nibbler
                    && !matches!(
                        m.mob_type,
                        MobType::BlobletMage | MobType::BlobletRange | MobType::BlobletMelee
                    )
                {
                    if trapped_big_len < trapped_big.len() {
                        trapped_big[trapped_big_len] = m.mob_type;
                    }
                    trapped_big_len += 1;
                }
            }
            if all_no_los && state.dead_count < state.mobs.len() {
                let valid = check_trapped_valid(&trapped_big, trapped_big_len);
                let completed_tick = state.tick;
                return Ok(finish_sim_result(
                    state,
                    attack_start,
                    completed_tick,
                    if valid {
                        SimStatus::Trapped
                    } else {
                        SimStatus::Invalid
                    },
                ));
            }
        }
    }
    Ok(finish_sim_result(
        state,
        attack_start,
        params.max_ticks,
        SimStatus::Timeout,
    ))
}

pub(crate) fn finish_sim_result(
    state: State<'_>,
    attack_start: usize,
    completed_tick: i32,
    status: SimStatus,
) -> SimResult {
    SimResult {
        attack_start,
        attack_end: state.attacks.len(),
        completed_tick,
        status,
        mob_init_hp: state.mob_init_hp,
        mob_init_len: state.mob_init_len,
    }
}

pub(crate) fn check_trapped_valid(trapped: &[MobType; 3], trapped_len: usize) -> bool {
    if trapped_len == 0 {
        return true;
    }
    if trapped_len > 2
        || trapped
            .iter()
            .take(trapped_len)
            .any(|t| *t == MobType::Mager)
    {
        return false;
    }
    if trapped_len == 1 {
        return true;
    }
    let a = trapped[0];
    let b = trapped[1];
    matches!(
        (a, b),
        (MobType::Blob, MobType::Blob)
            | (MobType::Bat, MobType::Bat)
            | (MobType::Bat, MobType::Ranger)
            | (MobType::Ranger, MobType::Bat)
    )
}

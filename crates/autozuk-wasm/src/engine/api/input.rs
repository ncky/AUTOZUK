use serde_json::Value;

use crate::engine::{
    attack_style_idx, AttackStats, Loadout, MobType, MonsterAttackTable, PillarConfig, Style,
    MOB_TYPE_COUNT,
};

pub(crate) fn parse_pillar_config(value: Option<&Value>) -> Option<PillarConfig> {
    let value = value?;
    Some(PillarConfig {
        s: value.get("S").and_then(Value::as_bool).unwrap_or(false),
        w: value.get("W").and_then(Value::as_bool).unwrap_or(false),
        n: value.get("N").and_then(Value::as_bool).unwrap_or(false),
    })
}

pub(crate) fn parse_loadout(value: &Value) -> Result<Loadout, String> {
    let mut player_acc = [[0.0f64; 2]; MOB_TYPE_COUNT];
    let player_acc_value = value
        .get("playerAcc")
        .ok_or_else(|| "loadout.playerAcc is required".to_string())?;
    for mob_type in MobType::all() {
        let arr = player_acc_value
            .get(mob_type.js_key())
            .and_then(Value::as_array)
            .ok_or_else(|| format!("loadout.playerAcc.{} is required", mob_type.js_key()))?;
        if arr.len() < 2 {
            return Err(format!(
                "loadout.playerAcc.{} must have two values",
                mob_type.js_key()
            ));
        }
        player_acc[mob_type.idx()][0] = arr[0].as_f64().unwrap_or(0.0);
        player_acc[mob_type.idx()][1] = arr[1].as_f64().unwrap_or(0.0);
    }

    let monster_atk_value = value
        .get("monsterAtk")
        .ok_or_else(|| "loadout.monsterAtk is required".to_string())?;
    let mut monster_atk = MonsterAttackTable {
        normal: [None; MOB_TYPE_COUNT],
        melee: [None; MOB_TYPE_COUNT],
        blob_mage: None,
        blob_range: None,
        blob_melee: None,
    };
    for mob_type in MobType::all() {
        let Some(entry) = monster_atk_value.get(mob_type.js_key()) else {
            continue;
        };
        if mob_type == MobType::Blob {
            monster_atk.blob_mage = parse_attack_stats(entry.get("mage"))?;
            monster_atk.blob_range = parse_attack_stats(entry.get("range"))?;
            monster_atk.blob_melee = parse_attack_stats(entry.get("melee"))?;
        } else {
            monster_atk.normal[mob_type.idx()] = parse_attack_stats(Some(entry))?;
            monster_atk.melee[mob_type.idx()] = parse_attack_stats(entry.get("melee"))?;
        }
    }
    let monster_atk_fast = build_monster_attack_fast(&monster_atk);

    let starting_hp = value
        .get("startingHp")
        .and_then(Value::as_f64)
        .map(round_js)
        .unwrap_or(99)
        .clamp(1, 115);
    let has_recoil = value
        .get("hasRecoil")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Ok(Loadout {
        atk_speed: get_i32(value, "atkSpeed")?,
        max_hit: get_i32(value, "maxHit")?,
        range: get_i32(value, "range")?,
        starting_hp,
        has_recoil,
        has_ring_recoil: has_recoil
            && value
                .get("hasRingRecoil")
                .and_then(Value::as_bool)
                .unwrap_or(true),
        has_echo_boots: has_recoil
            && value
                .get("hasEchoBoots")
                .and_then(Value::as_bool)
                .unwrap_or(true),
        is_blood_barrage: value
            .get("isBloodBarrage")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        has_blood_sceptre: value
            .get("hasBloodSceptre")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        player_acc,
        monster_atk_fast,
    })
}

pub(crate) fn build_monster_attack_fast(
    monster_atk: &MonsterAttackTable,
) -> [[Option<AttackStats>; 3]; MOB_TYPE_COUNT] {
    let mut fast = [[None; 3]; MOB_TYPE_COUNT];
    for mob_type in MobType::all() {
        for style in [Style::Magic, Style::Range, Style::Melee] {
            fast[mob_type.idx()][attack_style_idx(style)] = monster_atk.resolve(mob_type, style);
        }
    }
    fast
}

pub(crate) fn parse_attack_stats(value: Option<&Value>) -> Result<Option<AttackStats>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(max) = value.get("max").and_then(Value::as_f64) else {
        return Ok(None);
    };
    let Some(acc) = value.get("acc").and_then(Value::as_f64) else {
        return Ok(None);
    };
    Ok(Some(AttackStats {
        max: round_js(max),
        acc,
    }))
}

pub(crate) fn get_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string"))
}

pub(crate) fn get_i32(value: &Value, key_name: &str) -> Result<i32, String> {
    value
        .get(key_name)
        .and_then(Value::as_f64)
        .map(round_js)
        .ok_or_else(|| format!("{key_name} must be a number"))
}

pub(crate) fn get_u32(value: &Value, key_name: &str) -> Result<u32, String> {
    value
        .get(key_name)
        .and_then(Value::as_u64)
        .map(|v| v as u32)
        .or_else(|| {
            value
                .get(key_name)
                .and_then(Value::as_i64)
                .map(|v| v as u32)
        })
        .or_else(|| {
            value
                .get(key_name)
                .and_then(Value::as_f64)
                .map(|v| v as u32)
        })
        .ok_or_else(|| format!("{key_name} must be a number"))
}

pub(crate) fn round_js(value: f64) -> i32 {
    (value + 0.5).floor() as i32
}

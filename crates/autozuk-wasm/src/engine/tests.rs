use serde_json::{json, Value};

use super::{
    api::simulate_tile_inner, damage_bucket, parse_spawn_code, prayer_slot, DAMAGE_BUCKETS,
};

const AYAK_LOADOUT: &str = r#"{
      "name":"Ayak","atkSpeed":3,"maxHit":39,"range":8,"startingHp":99,
      "hasRecoil":true,"hasRingRecoil":true,"hasEchoBoots":true,
      "playerAcc":{
        "nibbler":[0.9720,0.9990],"bat":[0.8500,0.9700],"blob":[0.6025,0.7894],
        "blobletMelee":[0.9830,0.9996],"blobletMage":[0.6966,0.8773],"blobletRange":[0.9830,0.9996],
        "meleer":[0.6796,0.8631],"ranger":[0.8325,0.9626],"mager":[0.4784,0.6379]
      },
      "monsterAtk":{
        "nibbler":{"max":4,"acc":1.0},"bat":{"max":19,"acc":0.0843},
        "blob":{"mage":{"max":29,"acc":0.6188},"range":{"max":29,"acc":0.1281},"melee":{"max":29,"acc":0.0756}},
        "blobletMelee":{"max":18,"acc":0.0577},"blobletMage":{"max":18,"acc":0.4088},"blobletRange":{"max":18,"acc":0.0799},
        "meleer":{"max":49,"acc":0.1530},"ranger":{"max":46,"acc":0.1873,"melee":{"max":19,"acc":0.0666}},
        "mager":{"max":70,"acc":0.8422,"melee":{"max":52,"acc":0.1745}}
      }
    }"#;

#[test]
fn simulates_known_tile_like_js_worker() {
    let input = json!({
        "tile": {"x": 15, "y": 15},
        "spawnCode": "MRYBXOOOO",
        "pillarConfig": {"S": true, "W": true, "N": true},
        "loadout": serde_json::from_str::<Value>(AYAK_LOADOUT).unwrap(),
        "maxTicks": 400,
        "maxSims": 20,
        "seedBase": 42
    });
    let out = simulate_tile_inner(&input.to_string()).unwrap();
    let summary = out.get("summary").unwrap();
    assert_eq!(summary.get("totalSims").unwrap().as_u64().unwrap(), 10);
    assert_eq!(
        summary.get("prayer").unwrap().as_array().unwrap(),
        &vec![json!("mage"), json!("mage"), json!("mage"), json!("range")]
    );
    assert!((summary.get("avgDamage").unwrap().as_f64().unwrap() - 90.5).abs() < f64::EPSILON);
}

#[test]
fn spawn_code_fills_missing_inference_numbers_in_order() {
    let parsed = parse_spawn_code("M1RX").unwrap();
    let inf_numbers: Vec<i32> = parsed
        .spawns
        .iter()
        .filter(|spawn| spawn.mob_type.is_some())
        .map(|spawn| spawn.inf_num)
        .collect();

    assert!(parsed.has_index_info);
    assert_eq!(inf_numbers, vec![1, 3, 2]);
}

#[test]
fn scoring_helpers_keep_bucket_and_slot_boundaries_stable() {
    assert_eq!(prayer_slot(0), 0);
    assert_eq!(prayer_slot(5), 1);
    assert_eq!(damage_bucket(0, 100), 0);
    assert_eq!(damage_bucket(1000, 100), DAMAGE_BUCKETS - 1);
}

use crate::engine::{MobType, ParsedSpawnCode, Spawn, WaveFlags, SPAWN_LOCATIONS};

pub(crate) fn parse_spawn_code(code: &str) -> Result<ParsedSpawnCode, String> {
    let code = code.trim().to_uppercase();
    if code.is_empty() {
        return Err("Enter a spawn code".to_string());
    }
    let chars: Vec<char> = code.chars().collect();
    let mut spawns = Vec::new();
    let mut i = 0usize;
    let mut pos = 0usize;
    while i < chars.len() && pos < 9 {
        let ch = chars[i];
        let mob_type =
            MobType::from_code(ch).ok_or_else(|| format!("Unknown '{}' at pos {}", ch, i + 1))?;
        i += 1;
        let mut inf_num = 0;
        if i < chars.len() && ('1'..='9').contains(&chars[i]) {
            inf_num = chars[i].to_digit(10).unwrap_or(0) as i32;
            i += 1;
        }
        let (x, y) = SPAWN_LOCATIONS[pos];
        spawns.push(Spawn {
            mob_type,
            x,
            y,
            inf_num,
        });
        pos += 1;
    }

    let has_index_info = spawns.iter().any(|s| s.mob_type.is_some() && s.inf_num > 0);
    if has_index_info {
        let mut used = [false; 10];
        let non_nothing = spawns.iter().filter(|s| s.mob_type.is_some()).count();
        for s in &spawns {
            if s.mob_type.is_some() && s.inf_num > 0 && (s.inf_num as usize) < used.len() {
                used[s.inf_num as usize] = true;
            }
        }
        let mut remaining = Vec::new();
        for (n, is_used) in used.iter().enumerate().take(non_nothing + 1).skip(1) {
            if !*is_used {
                remaining.push(n as i32);
            }
        }
        remaining.sort_by(|a, b| b.cmp(a));
        let mut ri = 0usize;
        for s in &mut spawns {
            if s.mob_type.is_some() && s.inf_num == 0 && ri < remaining.len() {
                s.inf_num = remaining[ri];
                ri += 1;
            }
        }
    }

    Ok(ParsedSpawnCode {
        spawns,
        has_index_info,
    })
}

pub(crate) fn wave_flags(parsed: &ParsedSpawnCode) -> WaveFlags {
    let mut flags = WaveFlags {
        has_mager: false,
        has_ranger: false,
        has_meleer: false,
    };
    for spawn in &parsed.spawns {
        match spawn.mob_type {
            Some(MobType::Mager) => flags.has_mager = true,
            Some(MobType::Ranger) => flags.has_ranger = true,
            Some(MobType::Meleer) => flags.has_meleer = true,
            _ => {}
        }
    }
    flags
}

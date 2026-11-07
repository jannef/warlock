//! The starter monsters (#1200): the first five levels are cut, level 1 hardest. A monster of
//! level 1 to 5 has [`hp_percent`] of the roster formulas' hit points (Toughness and `base_hp`
//! alike: the generated families' numbers here, the hand-written types in their files, which
//! the roster tests hold to the same share), and its riders stay under [`rider_cap`].

/// The share, in percent, of the roster formulas' hit points a monster of `level` has: 40 at
/// level 1, rising to 90 at level 5, the formulas' own from level 6.
pub fn hp_percent(level: u8) -> u32 {
    match level {
        0 | 6.. => 100,
        1 => 40,
        2 => 55,
        3 => 70,
        4 => 80,
        5 => 90,
    }
}

/// `value` cut to [`hp_percent`] of itself at `level`, rounded to the nearest.
pub fn cut(value: i64, level: u8) -> i64 {
    let percent = i64::from(hp_percent(level));
    (value * percent + 50).div_euclid(100)
}

/// The most chance, in percent, a rider (an `on_hit` status, hold, disease, or drain) of a
/// monster of `level` may have: 1 at level 1 to 5 at level 5; `None` from level 6. A rider that
/// defines its creature (a spider's venom, a leech's latch, a mimic's grip: every hold) may take
/// [`DEFINING_CAP`] instead from level 2.
pub fn rider_cap(level: u8) -> Option<u8> {
    match level {
        0 | 6.. => None,
        1 => Some(1),
        2 | 3 => Some(3),
        4 => Some(4),
        5 => Some(5),
    }
}

/// The most chance a defining rider of a monster of level 2 to 5 may have (none at level 1).
pub const DEFINING_CAP: u8 = 12;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{Affliction, Content, CreatureDef};
    use crate::stats::Stat;

    /// The statuses that define their creature, kept at up to [`DEFINING_CAP`] (#1200): the
    /// spiders' and the centipede's venom, the blood the leech draws. Every hold is defining too.
    const DEFINING: &[(&str, &str)] = &[
        ("barrow_spider", "poisoned"),
        ("giant_centipede", "poisoned"),
        ("giant_leech", "bleeding"),
        ("large_spider", "poisoned"),
    ];

    #[test]
    fn the_share_rises_to_the_formulas_own() {
        let shares: Vec<u32> = (0..=7).map(hp_percent).collect();
        assert_eq!(shares, [100, 40, 55, 70, 80, 90, 100, 100]);
        assert_eq!(cut(12, 1), 5);
        assert_eq!(cut(19, 5), 17);
        assert_eq!(cut(30, 6), 30);
    }

    #[test]
    fn starter_riders_are_rare_and_mild() {
        let content = Content::builtin().expect("content");
        let mut wrong = Vec::new();
        for def in content.creature_defs() {
            let Some(cap) = rider_cap(def.level) else {
                continue;
            };
            if !monster(&content, def) {
                continue;
            }
            let id = def.id.as_str();
            for hit in def.attack.iter().flat_map(|a| &a.on_hit) {
                let (name, defining) = match hit.what {
                    Affliction::Status {
                        status, severity, ..
                    } => {
                        assert!(severity <= 1, "{id}: a starter's status is one degree");
                        let name = content.status(status).id.as_str();
                        (name, DEFINING.contains(&(id, name)))
                    }
                    // A hold is its creature's own: a leech's latch, a mimic's grip.
                    Affliction::Hold { status, .. } => (content.status(status).id.as_str(), true),
                    Affliction::Disease(_) => ("a disease", false),
                    Affliction::Drain(_) => panic!("{id}: no drain before level 6"),
                };
                let most = if defining && def.level > 1 {
                    DEFINING_CAP
                } else {
                    cap
                };
                if hit.chance > most {
                    let level = def.level;
                    wrong.push(format!(
                        "{id}: {name} at {}%, {most}% at {level}",
                        hit.chance
                    ));
                }
            }
        }
        assert!(wrong.is_empty(), "riders over the cap: {wrong:#?}");
    }

    #[test]
    fn starters_are_soft() {
        let content = Content::builtin().expect("content");
        let mut wrong = Vec::new();
        for def in content.creature_defs() {
            let percent = hp_percent(def.level);
            if percent == 100 || !monster(&content, def) {
                continue;
            }
            let toughness = u32::from(def.stats.score(Stat::Toughness));
            if toughness > 20 * percent / 100 {
                wrong.push(format!(
                    "{}: Toughness {toughness} at {}",
                    def.id, def.level
                ));
            }
        }
        assert!(wrong.is_empty(), "starters too tough: {wrong:#?}");
    }

    /// Whether the type is a monster the cut is for: not the player, nor one of the people who
    /// fight no one (the villagers, the traders, the spirits).
    fn monster(content: &Content, def: &CreatureDef) -> bool {
        let people = ["merchants", "peaceful", "player", "spirits"];
        !(people.iter()).any(|p| def.faction.is_some() && def.faction == content.factions().id(p))
    }
}

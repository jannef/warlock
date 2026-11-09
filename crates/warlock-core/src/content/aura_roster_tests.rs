//! The bearers of auras, innate statuses, and cries (design `docs/design/monster-dread.md` §6,
//! #1128): each row of the sort as shipped, and nothing on a type the sort does not name.

use super::Content;
use crate::noise::SoundKind;
use crate::stats::Stat;

/// An aura row: the type, its status, its radius, whether it is heard, and its check's stat.
type AuraRow = (&'static str, &'static str, i32, bool, Option<Stat>);

const AURAS: &[AuraRow] = &[
    // The undead (`docs/dev/creatures/undead.md`).
    ("wraith", "sluggish", 2, false, Some(Stat::Toughness)),
    ("dread_wraith", "sluggish", 2, false, Some(Stat::Toughness)),
    ("wraith_lord", "sluggish", 2, false, Some(Stat::Toughness)),
    ("banshee", "frightened", 6, true, Some(Stat::Willpower)),
    ("gibbering_shade", "silenced", 3, false, None),
    (
        "washer_at_the_ford",
        "frightened",
        3,
        false,
        Some(Stat::Willpower),
    ),
    ("deathshade", "frightened", 3, false, Some(Stat::Willpower)),
    ("archlich", "frightened", 3, false, Some(Stat::Willpower)),
    ("demilich", "frightened", 3, false, Some(Stat::Willpower)),
    (
        "screaming_skull",
        "frightened",
        4,
        true,
        Some(Stat::Willpower),
    ),
    // The demons.
    ("hezrou", "poisoned", 2, false, Some(Stat::Toughness)),
    ("chasme", "drowsy", 3, true, Some(Stat::Willpower)),
    ("vrock", "frightened", 2, false, Some(Stat::Willpower)),
    ("nalfeshnee", "frightened", 3, false, Some(Stat::Willpower)),
    // The devils.
    ("bone_devil", "frightened", 2, false, Some(Stat::Willpower)),
    ("ice_devil", "sluggish", 2, false, Some(Stat::Toughness)),
    ("pit_fiend", "frightened", 3, false, Some(Stat::Willpower)),
    // The hags and the fey.
    ("bheur_hag", "sluggish", 2, false, Some(Stat::Toughness)),
    ("satyr", "drowsy", 3, true, Some(Stat::Willpower)),
    ("yeth_hound", "frightened", 2, false, Some(Stat::Willpower)),
];

const INNATE: &[(&str, &str)] = &[
    ("fire_salamander", "fire_cloaked"),
    ("balor", "fire_cloaked"),
    ("pit_fiend", "fire_cloaked"),
    ("barbed_devil", "barbed_hide"),
    ("ember_beetle", "acid_blood"),
];

const CRIES: &[(&str, SoundKind)] = &[
    ("banshee", SoundKind::Wail),
    ("screaming_skull", SoundKind::Scream),
    ("vrock", SoundKind::Scream),
    ("chasme", SoundKind::Buzzing),
    ("knocker", SoundKind::Banging),
    ("poltergeist", SoundKind::Banging),
    ("charnel_heap", SoundKind::Rattle),
    ("bone_dragon", SoundKind::Rattle),
    ("basilisk", SoundKind::Hiss),
    ("crocodile", SoundKind::Hiss),
    ("cockatrice", SoundKind::Hiss),
    ("hydra", SoundKind::Hiss),
    ("pixie", SoundKind::Giggle),
    ("nymph", SoundKind::Giggle),
    ("yeth_hound", SoundKind::Howl),
    ("bodach", SoundKind::Howl),
    ("barghest", SoundKind::Howl),
    ("black_shuck", SoundKind::Howl),
    ("church_grim", SoundKind::Howl),
    ("hound_of_the_hunt", SoundKind::Howl),
    ("warg", SoundKind::Howl),
    ("dire_warg", SoundKind::Howl),
    // The miscast's gates (#1295), heard through the walls.
    ("rift", SoundKind::Wail),
    ("hellmouth", SoundKind::Howl),
    ("gloom_gate", SoundKind::Howl),
    ("abyssal_gate", SoundKind::Scream),
    ("iron_gate", SoundKind::Banging),
    ("black_gate", SoundKind::Wail),
    ("dire_gate", SoundKind::Scream),
    ("arch_gate", SoundKind::Howl),
    ("lords_gate", SoundKind::Scream),
];

#[test]
fn the_auras_keep_to_the_sort() {
    let content = Content::builtin().expect("content");
    for def in &content.creatures {
        let row = AURAS.iter().find(|r| r.0 == def.id);
        assert_eq!(def.aura.is_some(), row.is_some(), "{}: an aura", def.id);
        let (Some(aura), Some(&(id, status, radius, heard, check))) = (&def.aura, row) else {
            continue;
        };
        assert_eq!(content.status(aura.status).id, status, "{id}: its status");
        assert_eq!(aura.radius, radius, "{id}: its radius");
        assert_eq!(aura.heard.is_some(), heard, "{id}: heard");
        assert_eq!(aura.check.map(|c| c.0), check, "{id}: its check");
        // The deeper the bearer, the harder its check; never past what a check may be.
        if let Some((_, difficulty)) = aura.check {
            assert!(
                (12..=24).contains(&difficulty),
                "{id}: difficulty {difficulty}"
            );
        }
        // A cold aura meets cold resistance.
        let cold = content.damage_type_id("cold");
        if status == "sluggish" {
            assert_eq!(aura.kind, cold, "{id}: the cold aura is cold");
        }
    }
}

#[test]
fn the_innate_statuses_and_the_cries_keep_to_the_sort() {
    let content = Content::builtin().expect("content");
    for def in &content.creatures {
        let mut innate: Vec<&str> = (def.innate.iter())
            .map(|&s| content.status(s).id.as_str())
            .collect();
        innate.sort_unstable();
        let mut want: Vec<&str> = (INNATE.iter())
            .filter(|r| r.0 == def.id)
            .map(|r| r.1)
            .collect();
        want.sort_unstable();
        assert_eq!(innate, want, "{}: innate statuses", def.id);
        // Every innate status strikes back: the aura's cousin for the thing that hurts to hit.
        for &s in &def.innate {
            assert!(
                content.status(s).retaliates.is_some(),
                "{}: retaliates",
                def.id
            );
        }
        let want = CRIES.iter().find(|r| r.0 == def.id).map(|r| r.1);
        assert_eq!(def.cry.map(|c| c.kind), want, "{}: its cry", def.id);
    }
}

//! Beast dens (spec §7 "Special rooms"; `mapgen/den.rs`): a teeming den whose sleepers are only
//! of its factions (wildlife, vermin, marsh), drawn only where the branch's population holds one,
//! with a hoard of bones and fresh corpses instead of items.

use crate::content::{Content, ContentFiles, Hoard, SpecialRoom, Swarm};
use crate::geom::Pos;

/// A rooms branch of `population` whose every special room is a beast den where one fits.
fn branch(population: &str) -> Content {
    let lair = format!(
        r#"
[[dungeon]]
id = "lair"
name = "Lair"
generator = "rooms"
depth = 1
size = [80, 21]
danger = 3
population = [{population}]
room_effects = {{ chance = 100, per_level = 1, favoured = {{ beast_den = 100 }} }}
"#
    );
    let mut files = ContentFiles::builtin();
    let dungeons = format!(
        "{}{lair}",
        include_str!("../../../../assets/data/dungeons.toml")
    );
    files.insert("data/dungeons.toml", &dungeons);
    Content::load(&files).expect("content")
}

#[test]
fn a_beast_den_is_data() {
    let content = Content::load(&ContentFiles::builtin()).expect("content");
    let id = content.room_effects().id("beast_den").expect("beast_den");
    let def = content.room_effect(id);
    assert_eq!(def.name, "beast den");
    assert!(def.needs_danger);
    assert_eq!(def.min_cells, 16);
    assert_eq!(def.weight, 5);
    let swarm = def.swarm.as_ref().expect("a swarm");
    assert_eq!(swarm.factions, ["wildlife", "vermin", "marsh"]);
    assert_eq!(swarm.hoard, Hoard::Corpses);
    let teeming = content.room_effects().id("teeming").expect("teeming");
    let plain = content.room_effect(teeming).swarm.as_ref().expect("swarm");
    assert!(plain.factions.is_empty());
    assert_eq!(plain.hoard, Hoard::Items, "the default");
}

#[test]
fn unknown_or_twice_named_factions_are_refused() {
    let shipped = include_str!("../../../../assets/data/room_effects.toml");
    let good = r#"factions = ["wildlife", "vermin", "marsh"]"#;
    assert!(shipped.contains(good), "the shipped beast den");
    for bad in [
        r#"factions = ["wildlife", "dragons"]"#,
        r#"factions = ["vermin", "vermin"]"#,
    ] {
        let mut files = ContentFiles::builtin();
        files.insert("data/room_effects.toml", &shipped.replace(good, bad));
        assert!(Content::load(&files).is_err(), "{bad}");
    }
    let mut files = ContentFiles::builtin();
    let text = shipped.replace(r#"hoard = "corpses""#, r#"hoard = "gold""#);
    files.insert("data/room_effects.toml", &text);
    assert!(Content::load(&files).is_err(), "an unknown hoard");
}

#[test]
fn a_generated_beast_den_holds_only_beasts_and_their_kills() {
    let content = branch(r#""bat", "giant_centipede", "kobold", "kobold_archer", "rat""#);
    let den = content.room_effects().id("beast_den").expect("beast_den");
    let bones = content.object_id("bones").expect("bones");
    let corpse = |o| {
        (0..content.creature_defs().len()).any(|i| {
            let id = crate::content::CreatureId(u16::try_from(i).expect("id"));
            content.corpse_of(id) == Some(o)
        })
    };
    let beasts = ["wildlife", "vermin", "marsh"].map(|f| content.factions().id(f));
    let (mut dens, mut corpses) = (0, 0);
    for seed in 0..20 {
        let make = |s| crate::mapgen::generate(&content, ("showcase", s), "lair:1", None);
        let level = make(seed).expect("lair:1");
        assert_eq!(make(seed).expect("again"), level, "deterministic");
        let Some(room) = level.rooms.iter().find(|r| r.effect == den) else {
            continue;
        };
        dens += 1;
        let inside: Vec<_> = (level.creatures.iter())
            .filter(|c| room.contains(c.pos))
            .collect();
        assert!(inside.iter().all(|c| c.asleep), "seed {seed}: all asleep");
        assert!((4..=16).contains(&inside.len()), "seed {seed}");
        for c in &inside {
            let faction = content.creature(c.creature).faction;
            assert!(beasts.contains(&faction), "seed {seed}: a beast");
        }
        let stacks = |room: &SpecialRoom| -> Vec<(Pos, crate::object::Stack)> {
            (level.objects.iter())
                .filter(|(p, _)| room.contains(**p))
                .flat_map(|(p, pile)| pile.iter().map(move |s| (*p, s.clone())))
                .collect()
        };
        let here = stacks(room);
        let piles: u32 = (here.iter())
            .filter(|(_, s)| s.object == bones)
            .map(|(_, s)| s.count)
            .sum();
        let hoard = Swarm::hoard(inside.len());
        assert_eq!(piles as usize, hoard, "seed {seed}: bones per piece");
        let kills: usize = here.iter().filter(|(_, s)| corpse(s.object)).count();
        assert!(kills <= hoard, "seed {seed}: at most a corpse a piece");
        assert!(
            here.iter().all(|(_, s)| s.died.is_none()),
            "seed {seed}: made fresh when the level is entered"
        );
        corpses += kills;
    }
    assert!(dens >= 10, "{dens} dens in 20 floors");
    assert!(corpses > 0, "corpses in the hoards");
}

#[test]
fn a_branch_without_beasts_never_gets_one() {
    let content = branch(r#""kobold", "kobold_archer", "kobold_shaman""#);
    let den = content.room_effects().id("beast_den").expect("beast_den");
    for seed in 0..20 {
        let level =
            crate::mapgen::generate(&content, ("showcase", seed), "lair:1", None).expect("lair:1");
        assert!(
            level.rooms.iter().all(|r| r.effect != den),
            "seed {seed}: no beast den"
        );
    }
}

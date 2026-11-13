//! A trap's damage by the level's danger (spec §8 "Traps", `[trap.grows]` in `data/traps.toml`,
//! `content::TrapGrows`, #1165): each trap's rungs by danger band; a level without a danger, and
//! every level below the first rung, the base; the base never kills a level-1 character at full
//! hit points; a door's and a chest's trap read the same rungs.

use super::*;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, TrapEffect};
use crate::dice::Roll;

/// A lit hall: the player beside the west wall, a trapped closed door east of them.
const HALL: &str = "\
##########
#@d......#
#........#
##########
";

/// Builtin content with the hall at `danger` (none at 0), its player of 1000 hit points without
/// a character (a run with one gives its own).
fn content_at(danger: u8) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    let header = if danger == 0 {
        String::new()
    } else {
        format!("danger = {danger}\npopulation = [\"kobold\"]\nspawn = {{ chance = 0 }}\n")
    };
    files.insert(
        "levels/test_rungs.toml",
        &format!(
            "name = \"Test\"\nambient = [200, 200, 200, 0]\n{header}map = '''\n{HALL}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"d\" = {{ terrain = \"door_closed\", trap = \"fireball\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_rungs.toml",
        "name = \"T\"\nstart_level = \"test_rungs\"\nhp = 1000\n",
    );
    Arc::new(Content::load(&files).expect("fixture content is valid"))
}

fn roll(text: &str) -> Roll {
    text.parse().expect("a roll")
}

/// What trap `id` takes off the player, sprung `times` on their cell, each time from full.
fn losses(w: &mut World, id: &str, times: usize) -> Vec<i32> {
    let content = w.content().clone();
    let kind = content.trap_id(id).expect("trap");
    let (player, here) = (w.player_id(), w.state.player().pos);
    let mut lost = Vec::new();
    for _ in 0..times {
        let full = w.state.player().max_hp(&content);
        w.player_mut().hp = full;
        let trap = crate::content::Trap::new(kind, content.trap(kind), false, None);
        w.state.current_mut().traps.insert(here, trap);
        w.spring_trap(here, player, &mut Vec::new());
        w.state.current_mut().traps.remove(&here);
        lost.push(full - w.state.player().hp);
    }
    lost
}

/// The builtin traps' damage by danger, at every band's edges: the base below the first rung
/// (and at 0, a level without a danger), each rung from its danger to the next's.
#[test]
fn each_trap_deals_its_rung_by_danger_band() {
    let content = Content::builtin().expect("builtin content");
    let table: [(&str, [&str; 7]); 6] = [
        ("arrow", ["1d2", "1d4", "1d6", "2d4", "2d6", "3d6", "4d6"]),
        ("spear", ["1d2", "1d5", "1d8", "2d6", "2d8", "3d8", "4d8"]),
        (
            "fireball",
            ["1d4", "2d4", "2d6", "3d6", "4d6", "6d6", "8d6"],
        ),
        ("frost", ["1d3", "1d6", "2d4", "2d6", "3d6", "4d6", "6d6"]),
        (
            "lightning_bolt",
            ["1d4", "2d4", "3d6", "4d6", "5d6", "7d6", "9d6"],
        ),
        ("pit", ["1d2", "1d4", "1d6", "2d4", "2d6", "3d6", "4d6"]),
    ];
    // The bands: 0 to 2, 3 to 5, 6 to 9, 10 to 14, 15 to 21, 22 to 29, 30 and on.
    let bands: [(u8, u8); 7] = [
        (0, 2),
        (3, 5),
        (6, 9),
        (10, 14),
        (15, 21),
        (22, 29),
        (30, 99),
    ];
    for (id, rungs) in table {
        let def = content.trap(content.trap_id(id).expect(id));
        for ((from, to), want) in bands.iter().zip(rungs) {
            for danger in [*from, *to] {
                assert_eq!(def.damage_at(danger), Some(roll(want)), "{id} at {danger}");
                let effect = def.effect_at(danger);
                assert_eq!(
                    effect.damage(),
                    Some(roll(want)),
                    "{id}'s effect at {danger}"
                );
                assert_eq!(effect.kind(), def.effect.kind());
            }
        }
        assert_eq!(def.effect.damage(), Some(roll(rungs[0])), "{id}: the base");
    }
    // What does no damage has no rungs.
    for id in ["alarm", "teleport"] {
        let def = content.trap(content.trap_id(id).expect(id));
        assert_eq!((def.damage_at(40), &def.grows), (None, &None), "{id}");
        assert_eq!(def.effect_at(40), def.effect);
    }
}

/// Sprung on a level, a trap deals its rung at the level's danger: a level without one, and one
/// below the first rung, the base.
#[test]
fn a_sprung_trap_reads_the_levels_danger() {
    for (danger, want) in [(0, "1d4"), (2, "1d4"), (3, "2d4"), (10, "3d6"), (30, "8d6")] {
        let mut w = World::new(content_at(danger), "test_rungs", None).expect("world");
        assert_eq!(w.danger(&w.state.current().id), danger);
        let want = roll(want);
        let lost = losses(&mut w, "fireball", 12);
        for l in &lost {
            assert!(
                (want.min()..=want.max()).contains(l),
                "danger {danger}: {l} of {want}"
            );
        }
        if danger == 30 {
            assert!(lost.iter().all(|l| *l > 4), "never the base's: {lost:?}");
        }
    }
}

/// A trapped door's trap reads the same rungs (a chest's goes off through the same
/// `spring_trap`): opened on a level of danger 30, its fireball is the deep one.
#[test]
fn a_doors_trap_reads_the_same_rungs() {
    let mut w = World::new(content_at(30), "test_rungs", None).expect("world");
    let full = w.state.player().hp;
    w.apply(&Action::Open(Dir::E));
    let lost = full - w.state.player().hp;
    assert!((8..=48).contains(&lost), "8d6: {lost}");
}

/// The most trap `def` can do in one going off at `danger`: its roll's best, doubled by a
/// critical for those that roll to hit.
fn worst(def: &crate::content::TrapDef, danger: u8) -> i32 {
    let best = def.damage_at(danger).map_or(0, |d| d.max());
    match def.effect {
        TrapEffect::Shot { attack, .. } | TrapEffect::Strike { attack } => {
            best * i32::from(attack.crit)
        }
        _ => best,
    }
}

/// The lowest rung cannot kill a level-1 character at full hit points (#1165, the owner: "we
/// can't make them too dead right away"): every race and class, on several seeds, takes every
/// damaging trap from full on a level without a danger and lives; and no trap's worst at that
/// rung, critical included, reaches the fewest hit points any of them started with.
#[test]
fn the_lowest_rung_never_kills_a_fresh_character() {
    let content = content_at(0);
    // A rigged trap (`data/rigs.toml`) lies only where its rigger set it, known to them.
    let rigged = |t: &&crate::content::TrapDef| t.id.starts_with("rigged_");
    let damaging: Vec<&str> = (content.trap_defs().iter())
        .filter(|t| t.effect.damage().is_some() && !rigged(t))
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(damaging.len(), 6);
    let mut fewest = i32::MAX;
    for race in &content.races().list {
        for class in content.classes() {
            for seed in 0..3u64 {
                let spec: CharacterSpec = format!("{},male,{}", race.id, class.id)
                    .parse()
                    .expect("a character");
                let mut w = World::start_run(
                    content.clone(),
                    "test_rungs",
                    Some(seed),
                    Some(&spec),
                    None,
                    true,
                )
                .expect("a run");
                assert_eq!(w.player_level(), 1);
                let full = w.state.player().max_hp(&content);
                fewest = fewest.min(full);
                for id in &damaging {
                    let lost = losses(&mut w, id, 4);
                    assert!(
                        lost.iter().all(|l| *l < full) && w.death().is_none(),
                        "{} {} (seed {seed}, {full} hp) dies of the {id}: {lost:?}",
                        race.id,
                        class.id
                    );
                }
            }
        }
    }
    for def in content.trap_defs().iter().filter(|t| !rigged(t)) {
        assert!(
            worst(def, 0) < fewest,
            "the {}'s worst, {}, against {fewest} hit points",
            def.id,
            worst(def, 0)
        );
    }
}

/// A malformed `[trap.grows]` is refused at load, saying what is wrong.
#[test]
fn bad_rungs_are_refused() {
    let refused = |grows: &str, kind: &str, words: &str| {
        let mut files = ContentFiles::builtin();
        let extra = match kind {
            "alarm" => "kind = \"alarm\"\nnoise = { sound = \"bell\", loudness = 300 }\n",
            _ => "kind = \"pit\"\ndamage = \"1d4\"\ndamage_type = \"bludgeoning\"\n",
        };
        let text = format!(
            "{}\n[[trap]]\nid = \"odd\"\nname = \"odd trap\"\nfg = \"#ffffff\"\n{extra}\
             [trap.grows]\ndamage = \"{grows}\"\n",
            include_str!("../../../../assets/data/traps.toml")
        );
        files.insert("data/traps.toml", &text);
        let e = Content::load(&files).expect_err(grows).to_string();
        assert!(e.contains(words), "{grows}: {e}");
    };
    refused("3 1d6", "pit", "`danger`, then rungs");
    refused("danger 3 1d6, 3 2d6", "pit", "dangers rise strictly");
    refused("danger 0 1d6", "pit", "1 to 99");
    refused("danger 100 1d6", "pit", "1 to 99");
    refused("danger 3 1d6-2", "pit", "can roll below 0");
    refused("danger 3 1x6", "pit", "bad roll");
    refused("danger 3 1d6", "alarm", "for a trap that does damage");
}

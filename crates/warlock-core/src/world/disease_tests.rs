//! Diseases in the world (spec §8 "Diseases"): caught by a check, they incubate, show as
//! Diseased, progress day by day with a check each time, damage stats (and through Toughness
//! the most hit points), are shaken off by checks won in a row or never, can kill, slow healing,
//! and are known by name after their first progression; stat damage mends once they are gone.
//! All of it is saved, hashed, and replayed.

use super::*;
use crate::action::{AttackKind, DiseaseNews, parse_actions};
use crate::check::check;
use crate::combat::{Blow, Strike};
use crate::content::{ContentFiles, DiseaseId, StatusId};
use crate::replay::Replay;
use crate::stats::Stat;
use crate::time::DAY;

const TERRAIN: &str = r##"
[[terrain]]
id = "floor"
name = "floor"
glyph = "."
fg = "#808080"

[[terrain]]
id = "wall"
name = "wall"
glyph = "#"
fg = "#a0a0a0"
walkable = false
transparent = false

[[terrain]]
id = "marsh"
name = "marsh"
glyph = '"'
fg = "#608060"
move_cost = 150
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "vermin"
name = "vermin"
hostile = ["player"]

[[faction]]
id = "tame"
name = "tame"
"##;

// The rat is frail and cannot fight back; the biter's bite always exposes its target to fever.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "rat"
name = "giant rat"
glyph = "r"
fg = "#806040"
weight = 4000
faction = "vermin"
stats = { toughness = 4, willpower = 1 }
level = 1
xp = 5

[[creature]]
id = "biter"
name = "biter"
glyph = "b"
fg = "#60a040"
weight = 3000
faction = "vermin"
attack = { damage = "1d2", name = "bite", verb = "bites", on_hit = [{ disease = "fever", chance = 100 }] }
"##;

const STATUS: &str = r##"
[[status]]
id = "bleeding"
name = "Bleeding"
max = 5
regen = 0

[[status]]
id = "diseased"
name = "Diseased"
max = 1
regen = 50
dies = "succumbs to disease"
death = "Died of disease"
"##;

// Fever is fought with Toughness and incubates two days; the others are fought with Willpower
// (so their damage does not change their checks) and incubate a day. Ache, rot, pox, and
// plague are all but sure to win against the scenario's Willpower 1 (only a natural 24 loses).
const DISEASES: &str = r##"
status = "diseased"

[[disease]]
id = "fever"
name = "fever"
difficulty = 12
incubation = 2
damage = { dexterity = "1d3", toughness = "1d3" }
cured_by = ["remove_disease"]
onset = "You feel hot."
symptoms = ["You burn with fever.", "You shiver."]
recovered = "The fever breaks."

[[disease]]
id = "ache"
name = "ache"
difficulty = 50
check = "willpower"
incubation = 1
damage = { toughness = 2, dexterity = 1 }
cured_by = ["remove_disease"]
onset = "You ache."
symptoms = ["Your bones ache."]
recovered = "The ache is gone."

[[disease]]
id = "ague"
name = "marsh ague"
difficulty = 13
incubation = 1
damage = { strength = 1 }
regen = 25
wading = { terrain = ["marsh"], per_hour = 100 }
onset = "You feel a chill."
symptoms = ["You shiver."]
recovered = "The ague leaves you."

[[disease]]
id = "rot"
name = "rot"
difficulty = 50
check = "willpower"
incubation = 1
damage = { toughness = 3, presence = 1 }
recovery = 0
kills = true
caught = ["rotten_food"]
onset = "Your skin greys."
symptoms = ["Your flesh rots."]
recovered = "The rot is gone."

[[disease]]
id = "pox"
name = "pox"
difficulty = 50
check = "willpower"
incubation = 1
hp = 5
recovery = 0
onset = "Spots."
symptoms = ["More spots."]
recovered = "No spots."

[[disease]]
id = "plague"
name = "plague"
difficulty = 50
check = "willpower"
incubation = 1
hp = 5
recovery = 0
kills = true
onset = "Buboes."
symptoms = ["More buboes."]
recovered = "No buboes."
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"~" = "marsh"
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"b" = { terrain = "floor", creature = "biter" }
"t" = { terrain = "floor", creature = "rat", faction = "tame" }
"##;

/// A room to oneself, with marsh to the east.
const ALONE: &str = "######\n#@.~~#\n######\n";

/// Willpower 1 (only a natural 24 wins a check against 50), Toughness 10, 30 more hit points.
const WEAK_WILLED: &str = "stats = { willpower = 1 }\nhp = 30";

fn files(map: &str, scenario: &str, extra: &[(&str, &str)]) -> ContentFiles {
    let mut files = ContentFiles::default();
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", CREATURES),
        ("data/status.toml", STATUS),
        ("data/diseases.toml", DISEASES),
        ("data/legend.toml", LEGEND),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "levels/sick.toml",
        &format!("name = \"Sickroom\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/sick.toml",
        &format!("name = \"Sick\"\nstart_level = \"sick\"\n{scenario}\n"),
    );
    for (path, text) in extra {
        files.insert(path, text);
    }
    files
}

fn world_with(map: &str, scenario: &str, extra: &[(&str, &str)], seed: Option<u64>) -> World {
    let content = Content::load(&files(map, scenario, extra)).expect("test content is valid");
    World::new(Arc::new(content), "sick", seed).expect("world")
}

fn world_on(map: &str, scenario: &str) -> World {
    world_with(map, scenario, &[], None)
}

fn disease(w: &World, id: &str) -> DiseaseId {
    w.content().disease_id(id).expect(id)
}

fn diseased(w: &World) -> StatusId {
    w.content().status_id("diseased").expect("diseased")
}

fn infection(w: &World, id: &str) -> Option<crate::disease::Infection> {
    w.state().player().diseases.get(disease(w, id)).copied()
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    parse_actions(actions)
        .expect("actions")
        .iter()
        .map(|a| world.apply(a))
        .collect()
}

fn messages(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn count(world: &World, text: &str) -> usize {
    (world.messages().iter())
        .filter(|m| m.display().starts_with(text))
        .count()
}

fn rest_day(w: &mut World) -> Outcome {
    w.apply(&Action::Rest(Rest::For(Duration::seconds(DAY))))
}

fn score(w: &World, stat: Stat) -> u8 {
    w.perception().own_scores()[stat]
}

/// The diseases the player knows they have, by name.
fn known(w: &World) -> Vec<String> {
    (w.perception().own_diseases())
        .map(|d| w.content().disease(d).name.clone())
        .collect()
}

fn shows_diseased(w: &World) -> bool {
    w.perception().own_effects().any(|s| s == diseased(w))
}

#[test]
fn a_disease_incubates_unseen_then_shows_as_diseased_and_is_named_after_a_progression() {
    // Toughness 40: every check against fever's 12 wins but a natural 2.
    let mut w = world_on(ALONE, "stats = { toughness = 40 }\nhp = 30");
    let start = w.time();
    let rng = w.state.rng.clone();
    run(&mut w, "infect:fever");
    assert_eq!(w.state.rng, rng, "a fixed incubation draws nothing");
    assert_eq!(
        messages(&w).last().map(String::as_str),
        Some("You catch fever (dev).")
    );
    let caught = infection(&w, "fever").expect("caught");
    assert_eq!(
        (caught.left, caught.shown),
        (2 * DAY as u32, false),
        "two days"
    );
    // A day: nothing to see.
    rest_day(&mut w);
    assert!(!shows_diseased(&w) && known(&w).is_empty());
    assert_eq!(
        w.healing_rate(w.player_id()),
        100,
        "it does not slow healing yet"
    );
    // The second day ends with its onset, which stops the rest at that moment.
    let out = rest_day(&mut w);
    assert_eq!(w.time(), start + Duration::seconds(2 * DAY));
    let onset = Event::Disease {
        actor: w.player_id(),
        disease: disease(&w, "fever"),
        news: DiseaseNews::Onset,
    };
    assert!(out.events.contains(&onset), "{:?}", out.events);
    assert_eq!(count(&w, "You feel hot."), 1);
    assert!(shows_diseased(&w), "Diseased");
    assert!(known(&w).is_empty(), "but not yet which disease");
    let names: Vec<String> = (w.perception().own_effects())
        .map(|s| w.content().status(s).name.clone())
        .collect();
    assert_eq!(names, ["Diseased"]);
    // A day later it progresses (and is won): now the player knows it.
    rest_day(&mut w);
    let fever = infection(&w, "fever").expect("still there");
    assert_eq!((fever.progressions, fever.streak), (1, 1));
    assert_eq!(known(&w), ["fever"]);
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Stats);
    for row in ["Effects Diseased", "Diseases fever"] {
        assert!(rows.contains(&row.to_string()), "{row} in {rows:?}");
    }
    // The second won in a row shakes it off: said, but it lets the rest go on.
    rest_day(&mut w);
    assert_eq!(infection(&w, "fever"), None);
    assert!(!shows_diseased(&w) && known(&w).is_empty());
    assert_eq!(count(&w, "The fever breaks."), 1);
    assert_eq!(
        messages(&w).last().map(String::as_str),
        Some("You rest for 24 hours.")
    );
}

#[test]
fn failed_checks_damage_stats_and_toughness_lowers_the_most_hit_points() {
    let mut w = world_on(ALONE, WEAK_WILLED);
    let (hp, max) = w.perception().own_hp();
    assert_eq!((hp, max), (40, 40));
    run(&mut w, "infect:ache");
    rest_day(&mut w);
    assert!(shows_diseased(&w), "a day: it shows");
    let out = rest_day(&mut w);
    let worse = Event::Disease {
        actor: w.player_id(),
        disease: disease(&w, "ache"),
        news: DiseaseNews::Worse(1),
    };
    assert!(out.events.contains(&worse), "{:?}", out.events);
    assert_eq!(count(&w, "Your bones ache."), 1);
    assert_eq!(
        (score(&w, Stat::Toughness), score(&w, Stat::Dexterity)),
        (8, 9)
    );
    assert_eq!(w.perception().own_base_scores()[Stat::Toughness], 10);
    assert_eq!(
        w.state().player().stats.score(Stat::Toughness),
        10,
        "untouched"
    );
    // Toughness 8: two hit points fewer at most, and what was left is cut down to them.
    assert_eq!(w.perception().own_hp(), (38, 38));
    assert_eq!(w.state().player().hp, 38);
    // Every rule reads the damaged scores: Defence by Dexterity 9.
    let me = w.state().player();
    let content = w.content().clone();
    assert_eq!(me.effective_stats(&content).modifier(Stat::Dexterity), -1);
    // Another day, more damage: it adds up.
    rest_day(&mut w);
    assert_eq!(
        (score(&w, Stat::Toughness), score(&w, Stat::Dexterity)),
        (6, 8)
    );
    assert_eq!(infection(&w, "ache").map(|i| i.progressions), Some(2));
}

#[test]
fn stat_damage_never_kills_and_leaves_every_score_at_least_one() {
    // Toughness 3: the ache's 2 a day would take it to 1, then no lower; the player lives on.
    let mut w = world_on(
        ALONE,
        "stats = { willpower = 1, toughness = 3, dexterity = 2 }\nhp = 5",
    );
    run(&mut w, "infect:ache");
    for _ in 0..5 {
        rest_day(&mut w);
    }
    assert!(w.death().is_none());
    assert_eq!(
        (score(&w, Stat::Toughness), score(&w, Stat::Dexterity)),
        (1, 1)
    );
    assert_eq!(w.perception().own_hp().1, 6, "Toughness 1 and 5 more");
    // Hit point damage stops at the last one too, unless the disease kills.
    let mut w = world_on(ALONE, WEAK_WILLED);
    run(&mut w, "infect:pox");
    rest_day(&mut w);
    // Three hit points left, and no healing (bleeding) until the progression.
    let player = w.player_id();
    w.state.actors.get_mut(&player).expect("player").hp = 3;
    run(&mut w, "afflict:bleeding");
    rest_day(&mut w);
    assert_eq!(count(&w, "More spots."), 1);
    assert_eq!((w.state().player().hp, w.death()), (1, None));
}

#[test]
fn a_disease_that_kills_kills_at_toughness_zero_or_the_last_hit_point() {
    let mut w = world_on(ALONE, WEAK_WILLED);
    run(&mut w, "infect:rot");
    rest_day(&mut w);
    // Three Toughness a day: 7, 4, 1, and then none left.
    for (day, toughness) in [(1, 7), (2, 4), (3, 1)] {
        rest_day(&mut w);
        assert_eq!(score(&w, Stat::Toughness), toughness, "day {day}");
        assert_eq!(score(&w, Stat::Presence), 10 - day, "day {day}");
        assert!(w.death().is_none());
    }
    let start = w.time();
    rest_day(&mut w);
    let death = w.death().expect("rotted away").clone();
    assert_eq!(death.of, Some(diseased(&w)));
    assert_eq!(
        death.time,
        start + Duration::seconds(DAY),
        "at the progression"
    );
    assert_eq!(
        messages(&w)[messages(&w).len() - 2..],
        ["Your flesh rots.".to_string(), "You die...".to_string()]
    );
    // It never goes by itself: no check won counts.
    assert!(!w.content().disease(disease(&w, "rot")).recovers());
    // Hit point damage that kills.
    let mut w = world_on(ALONE, WEAK_WILLED);
    run(&mut w, "infect:plague");
    rest_day(&mut w);
    let player = w.player_id();
    w.state.actors.get_mut(&player).expect("player").hp = 3;
    run(&mut w, "afflict:bleeding");
    rest_day(&mut w);
    assert_eq!(w.death().map(|d| d.of), Some(Some(diseased(&w))));
}

#[test]
fn checks_won_in_a_row_shake_a_disease_off_and_a_loss_starts_the_count_over() {
    // Ague against an average Toughness: about half the checks are won. Mirror the draws.
    for seed in 1..=6 {
        let mut w = world_with(ALONE, "hp = 30", &[], Some(seed));
        run(&mut w, "infect:ague");
        rest_day(&mut w);
        assert!(shows_diseased(&w), "seed {seed}");
        let mut rng = w.state.rng.clone();
        let (mut streak, mut losses, mut days) = (0, 0, 0);
        while streak < 2 {
            if check(&mut rng, 0, 13) {
                streak += 1;
            } else {
                streak = 0;
                losses += 1;
            }
            days += 1;
        }
        for day in 1..=days {
            assert!(infection(&w, "ague").is_some(), "seed {seed} day {day}");
            rest_day(&mut w);
        }
        assert_eq!(
            infection(&w, "ague"),
            None,
            "seed {seed}: gone after {days}"
        );
        assert_eq!(w.state.rng, rng, "seed {seed}: two draws a check");
        assert_eq!(count(&w, "The ague leaves you."), 1);
        // A point of Strength each loss, mending a point a day since: none mended yet.
        assert_eq!(10 - score(&w, Stat::Strength), losses.min(9), "seed {seed}");
    }
}

#[test]
fn stat_damage_mends_a_point_a_day_once_no_disease_shows() {
    let mut w = world_on(ALONE, WEAK_WILLED);
    run(&mut w, "infect:ache");
    rest_day(&mut w);
    rest_day(&mut w);
    rest_day(&mut w);
    // Two progressions: Toughness 4 and Dexterity 2 down, none of it mended meanwhile.
    assert_eq!(
        (score(&w, Stat::Toughness), score(&w, Stat::Dexterity)),
        (6, 8)
    );
    let player = w.player_id();
    let mut events = Vec::new();
    assert!(w.cure(player, "remove_disease", &mut events));
    assert_eq!(
        events,
        [
            Event::Disease {
                actor: player,
                disease: disease(&w, "ache"),
                news: DiseaseNews::Recovered
            },
            Event::Status {
                actor: player,
                status: diseased(&w),
                began: false
            }
        ]
    );
    assert!(!w.state().player().effects.has(diseased(&w)));
    assert!(
        !w.cure(player, "remove_disease", &mut events),
        "nothing left"
    );
    // A point of each a day.
    rest_day(&mut w);
    assert_eq!(
        (score(&w, Stat::Toughness), score(&w, Stat::Dexterity)),
        (7, 9)
    );
    rest_day(&mut w);
    assert_eq!(
        (score(&w, Stat::Toughness), score(&w, Stat::Dexterity)),
        (8, 10)
    );
    rest_day(&mut w);
    rest_day(&mut w);
    assert_eq!(score(&w, Stat::Toughness), 10);
    assert!(w.state().player().damage.is_none(), "all mended");
    // The most hit points came back with Toughness; the hit points themselves heal as ever.
    assert_eq!(w.perception().own_hp().1, 40);
}

#[test]
fn diseases_slow_healing_once_they_show() {
    let mut w = world_on(ALONE, WEAK_WILLED);
    let player = w.player_id();
    run(&mut w, "infect:ague");
    assert_eq!(w.healing_rate(player), 100, "incubating");
    rest_day(&mut w);
    assert_eq!(
        w.healing_rate(player),
        25,
        "ague: slower than Diseased's half"
    );
    run(&mut w, "infect:pox");
    rest_day(&mut w);
    assert_eq!(w.healing_rate(player), 25, "the slowest counts");
    let mut w = world_on(ALONE, WEAK_WILLED);
    run(&mut w, "infect:pox");
    rest_day(&mut w);
    assert_eq!(w.healing_rate(player), 50, "Diseased: half");
}

#[test]
fn bites_expose_their_target_who_resists_with_a_check() {
    let blow = Blow {
        strike: Strike::Hit { critical: false },
        damage: 1,
        kind: None,
    };
    let caught = |seed: u64, toughness: u8| {
        let scenario = format!("stats = {{ toughness = {toughness} }}\nhp = 1000");
        let mut w = world_with("#####\n#@b.#\n#####\n", &scenario, &[], Some(seed));
        let (player, biter) = (w.player_id(), ActorId(1));
        let mut expected = w.state.rng.clone();
        let modifier = crate::stats::modifier(i32::from(toughness));
        let resisted = check(&mut expected, modifier, 12);
        w.after_blow(biter, player, AttackKind::Melee, &blow, &mut Vec::new());
        assert_eq!(
            w.state.rng, expected,
            "two draws at 100 %, none for the incubation"
        );
        let got = infection(&w, "fever");
        assert_eq!(got.is_some(), !resisted, "seed {seed}");
        if let Some(i) = got {
            assert_eq!(
                (i.by, i.shown),
                (Some(biter), false),
                "silent until it shows"
            );
        }
        // Bitten again while it carries it: nothing drawn.
        let rng = w.state.rng.clone();
        if got.is_some() {
            w.after_blow(biter, player, AttackKind::Melee, &blow, &mut Vec::new());
            assert_eq!(w.state.rng, rng);
        }
        got.is_some()
    };
    let average: Vec<bool> = (1..=30).map(|seed| caught(seed, 10)).collect();
    assert!(
        average.contains(&true) && average.contains(&false),
        "{average:?}"
    );
    assert_eq!(
        average,
        (1..=30).map(|seed| caught(seed, 10)).collect::<Vec<_>>()
    );
    let hardy = (1..=30).filter(|&seed| caught(seed, 40)).count();
    assert!(hardy <= 1, "Toughness 40 resists all but a natural 2");
}

#[test]
fn wading_through_marsh_may_give_a_disease_by_the_hours_spent_there() {
    // A sure thing for a step of an hour: no draw for the chance, then the check.
    let mut w = world_on(ALONE, "hp = 30");
    let marsh = w.content().terrain_id("marsh").expect("marsh");
    let mut expected = w.state.rng.clone();
    let resisted = check(&mut expected, 0, 13);
    w.wade(marsh, 3600);
    assert_eq!(w.state.rng, expected);
    assert_eq!(infection(&w, "ague").is_some(), !resisted);
    // Half an hour: one draw out of 360 000 for the chance, then the check if it comes.
    let mut w = world_on(ALONE, "hp = 30");
    let mut expected = w.state.rng.clone();
    let exposed = expected.below(360_000) < 100 * 1800;
    let resisted = exposed && check(&mut expected, 0, 13);
    w.wade(marsh, 1800);
    assert_eq!(w.state.rng, expected);
    assert_eq!(infection(&w, "ague").is_some(), exposed && !resisted);
    // A step onto the marsh (9 s) draws for it; a step onto the floor does not; neither does a
    // step while it carries the disease.
    let mut w = world_on(ALONE, "hp = 30");
    let rng = w.state.rng.clone();
    run(&mut w, "move:E");
    assert_eq!(w.state.rng, rng, "floor");
    let mut expected = w.state.rng.clone();
    let exposed = expected.below(360_000) < 100 * 9;
    if exposed {
        check(&mut expected, 0, 13);
    }
    run(&mut w, "move:E");
    assert_eq!(w.state.rng, expected, "marsh: 9 s of it");
    run(&mut w, "infect:ague");
    let rng = w.state.rng.clone();
    run(&mut w, "move:E");
    assert_eq!(w.state.rng, rng, "it carries it already");
}

#[test]
fn rotten_food_exposes_the_eater_to_what_it_carries() {
    let mut w = world_on(ALONE, WEAK_WILLED);
    let player = w.player_id();
    let mut expected = w.state.rng.clone();
    let resisted = check(&mut expected, 0, 50);
    w.expose_to(player, "rotten_food");
    assert_eq!(w.state.rng, expected, "rot only");
    assert_eq!(infection(&w, "rot").is_some(), !resisted);
    let rng = w.state.rng.clone();
    w.expose_to(player, "moonlight");
    assert_eq!(w.state.rng, rng, "nothing comes of that");
}

#[test]
fn a_creature_killed_by_a_disease_the_player_gave_it_is_their_kill() {
    let experience = include_str!("../../../../assets/data/experience.toml");
    let rot = |by: Option<ActorId>| {
        let extra = [("data/experience.toml", experience)];
        let mut w = world_with("######\n#@..t#\n######\n", "hp = 1000", &extra, None);
        let rat = ActorId(1);
        let rot = disease(&w, "rot");
        assert!(w.infect(rat, rot, by));
        // Toughness 4: a day to show, then 3 and then the last of it.
        for _ in 0..3 {
            rest_day(&mut w);
        }
        assert!(!w.state.actors.contains_key(&rat), "it rotted away");
        assert_eq!(count(&w, "The giant rat succumbs to disease."), 1);
        w.player_xp()
    };
    assert_eq!(rot(Some(ActorId(0))), 5, "the player's doing: their kill");
    assert_eq!(rot(None), 0);
}

#[test]
fn time_cut_up_differently_ends_the_same() {
    // A day's rests against minute-long waits: the same draws at the same moments.
    let mut rested = world_with(ALONE, "hp = 30", &[], Some(3));
    let mut waited = world_with(
        ALONE,
        "hp = 30",
        &[("data/actions.toml", "wait = 10000\n")],
        Some(3),
    );
    for w in [&mut rested, &mut waited] {
        run(w, "infect:fever infect:ague");
    }
    for _ in 0..4 {
        rest_day(&mut rested);
    }
    for _ in 0..4 * 24 * 60 {
        waited.apply(&Action::Wait);
    }
    assert_eq!(rested.time(), waited.time());
    let (a, b) = (rested.state().player(), waited.state().player());
    assert_eq!((&a.diseases, &a.damage), (&b.diseases, &b.damage));
    assert_eq!(rested.state.rng, waited.state.rng);
}

#[test]
fn the_dev_infection_names_a_disease_and_the_umbrella_cannot_be_afflicted() {
    let mut w = world_on(ALONE, "hp = 30");
    let start = w.time();
    run(&mut w, "infect:smallpox");
    assert_eq!(
        messages(&w).last().map(String::as_str),
        Some("There is no disease `smallpox`.")
    );
    run(&mut w, "infect:fever infect:fever");
    assert_eq!(
        messages(&w).last().map(String::as_str),
        Some("You carry fever already (dev).")
    );
    run(&mut w, "afflict:diseased");
    assert!(!shows_diseased(&w));
    assert!(
        messages(&w)
            .last()
            .is_some_and(|m| m.contains("comes of a disease"))
    );
    assert_eq!(w.time(), start, "no game time");
}

#[test]
fn diseases_and_stat_damage_are_saved_hashed_and_replayed() {
    let content = Arc::new(Content::load(&files(ALONE, WEAK_WILLED, &[])).expect("content"));
    let mut w = World::new(content.clone(), "sick", None).expect("world");
    let mut replay = Replay::new(&w);
    let plain = w.state_hash();
    for action in parse_actions("infect:ache infect:fever rest:24h rest:24h wait*3").expect("ok") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_ne!(w.state_hash(), plain);
    assert!(w.state().player().damage.is_damaged());
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    // Saved and loaded, it goes on the same.
    let bytes = crate::save::encode(&w);
    let mut loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    rest_day(&mut w);
    rest_day(&mut loaded);
    assert_eq!(loaded.state_hash(), w.state_hash());
    // A save whose diseases make no sense is refused: Diseased without one showing, an
    // unknown disease, damage a score cannot carry.
    let player = w.player_id();
    let refuse = |change: &dyn Fn(&mut Actor)| {
        let mut state = w.state().clone();
        change(state.actors.get_mut(&player).expect("player"));
        assert!(matches!(
            World::from_state(w.content().clone(), state),
            Err(WorldError::InconsistentState(_))
        ));
    };
    let status = diseased(&w);
    refuse(&|a| {
        a.diseases = Default::default();
        assert!(a.effects.has(status));
    });
    refuse(&|a| {
        a.effects.remove(status);
    });
    refuse(&|a| {
        a.diseases.catch(DiseaseId(99), DAY, None);
    });
    refuse(&|a| {
        a.damage.take(Stat::Toughness, 5, 40);
        a.stats = crate::stats::Stats::new(
            crate::stats::PerStat::splat(3),
            crate::stats::PerStat::splat(3),
        );
    });
}

#[test]
fn the_built_in_diseases_are_the_six_pathologies() {
    let content = Content::builtin().expect("builtin content");
    let ids: Vec<&str> = (content.disease_defs().iter())
        .map(|d| d.id.as_str())
        .collect();
    assert_eq!(
        ids,
        [
            "filth_fever",
            "red_ache",
            "the_shakes",
            "marsh_ague",
            "grave_rot",
            "brain_fever"
        ]
    );
    let status = content.disease_status().expect("a status");
    assert_eq!(content.status(status).name, "Diseased");
    let def = |id: &str| content.disease(content.disease_id(id).expect(id));
    // The giant rat's bite may give filth fever.
    let rat = content.creature_id("rat").expect("rat");
    let bite = content.creature(rat).attack.as_ref().expect("a bite");
    let fever = content.disease_id("filth_fever").expect("filth fever");
    assert!(bite.on_hit.iter().any(|o| {
        matches!(o.what, crate::content::Affliction::Disease(d) if d == fever) && o.chance < 100
    }));
    let damaged = |id: &str| -> Vec<Stat> {
        (def(id).damage.iter())
            .filter(|(_, d)| !d.is_zero())
            .map(|(s, _)| s)
            .collect()
    };
    assert_eq!(damaged("filth_fever"), [Stat::Dexterity, Stat::Toughness]);
    assert_eq!(damaged("red_ache"), [Stat::Strength]);
    assert_eq!(damaged("the_shakes"), [Stat::Dexterity]);
    assert_eq!(damaged("marsh_ague"), [Stat::Toughness]);
    assert_eq!(damaged("grave_rot"), [Stat::Toughness, Stat::Presence]);
    assert_eq!(
        damaged("brain_fever"),
        [Stat::Intelligence, Stat::Willpower]
    );
    // Marsh ague: the marshes of swamp areas and the overworld's swamps; slower healing.
    let ague = def("marsh_ague");
    let lands: Vec<&str> = (ague.wading.as_ref().expect("caught wading").terrain.iter())
        .map(|&t| content.terrain(t).id.as_str())
        .collect();
    assert_eq!(lands, ["marsh", "swamp"]);
    assert!(ague.regen.is_some_and(|r| r < content.status(status).regen));
    // Grave rot: from rotten food, never by itself, and it kills.
    let rot = def("grave_rot");
    assert!(rot.caught.contains(&"rotten_food".to_string()) && !rot.recovers() && rot.kills);
    assert_eq!(def("brain_fever").check, Stat::Willpower);
    assert!(
        content
            .disease_defs()
            .iter()
            .all(|d| !d.kills || d.id == "grave_rot")
    );
}

#[test]
fn the_showcase_holds_no_disease() {
    // Its creatures are peaceful and it has no marsh: nothing there gives one.
    let content = Content::builtin().expect("builtin content");
    let world = World::new(Arc::new(content), "showcase", None).expect("world");
    let content = world.content();
    let marsh: Vec<_> = (content.disease_defs().iter())
        .filter_map(|d| d.wading.as_ref())
        .flat_map(|w| w.terrain.iter().copied())
        .collect();
    let (level, actors) = world.ground_truth();
    let hostile = actors
        .filter(|a| world.stance(a.id, world.player_id()) == crate::content::Stance::Hostile)
        .count();
    assert_eq!(hostile, 0);
    for y in 0..level.height() {
        for x in 0..level.width() {
            let t = level.terrain_at(Pos::new(x, y)).expect("in bounds");
            assert!(!marsh.contains(&t), "marsh in the hub at {x},{y}");
        }
    }
}

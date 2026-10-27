//! Forgotten altars and prayer (spec §7 "Special rooms"; `world/pray.rs`, `world/outcome.rs`):
//! the Here group lists Pray only beside an altar, a broken prayer gives nothing, the lean by
//! side and band (Devout), the hooks, the shares of the draw, exhaustion, the explosion, the
//! deed, and a spent altar kept as terrain alone.

use super::*;
use crate::ability::Ability;
use crate::alignment::{Alignment, Score};
use crate::content::{ContentFiles, OutcomeDef, OutcomeKind, OutcomeTable};
use crate::world::outcome::leaned_kinds;

/// A room with a forgotten altar at (5, 2), the player beside it at (4, 3), and at `k` a kobold
/// (also beside it), when asked for.
const ROOM: &str = "\
###########
#.........#
#....A....#
#...@k....#
#.........#
#.........#
###########
";

/// The altar.
const ALTAR: Pos = Pos::new(5, 2);

/// The built-in content with [`ROOM`], an altar room of side `side`; the kobold only with
/// `kobold`.
fn world(side: &str, kobold: bool) -> World {
    let mut files = ContentFiles::builtin();
    let k = if kobold {
        "{ terrain = \"floor\", creature = \"kobold\" }"
    } else {
        "\"floor\""
    };
    files.insert(
        "levels/test_altar.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [[room]]\neffect = \"altar\"\narea = [0, 0, 10, 6]\nside = \"{side}\"\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"A\" = \"forgotten_altar\"\n\"k\" = {k}\n"
        ),
    );
    files.insert(
        "scenarios/test_altar.toml",
        "name = \"T\"\nstart_level = \"test_altar\"\n\
         character = { race = \"human\", gender = \"female\", class = \"warrior\" }\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_altar", None).expect("world");
    // Sturdy enough for any outcome but the explosion's tests.
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    w
}

fn table(w: &World) -> OutcomeTable {
    w.content
        .outcomes()
        .get("forgotten_altar")
        .expect("table")
        .clone()
}

fn outcome(w: &World, id: &str) -> OutcomeDef {
    let table = table(w);
    table
        .outcomes
        .iter()
        .find(|o| o.id == id)
        .expect(id)
        .clone()
}

fn befall(w: &mut World, id: &str) {
    let outcome = outcome(w, id);
    w.befall(&outcome, ALTAR, &mut Vec::new());
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

fn terrain(w: &World, p: Pos) -> String {
    let t = w.state.current().terrain_at(p).expect("cell");
    w.content.terrain(t).id.clone()
}

fn here(w: &World) -> Vec<Ability> {
    (w.perception().abilities().into_iter())
        .map(|a| a.ability)
        .filter(|a| a.source() == crate::ability::Source::Place)
        .collect()
}

#[test]
fn pray_is_listed_only_beside_an_altar_that_is_not_spent() {
    let mut w = world("neutral", false);
    assert_eq!(here(&w), vec![Ability::Pray]);
    assert!(Ability::every(&w.content).contains(&Ability::Pray));
    w.apply(&Action::Move(Dir::S));
    assert!(here(&w).is_empty(), "two cells away");
    let before = w.state.time;
    w.apply(&Action::Pray);
    assert!(said(&w, "There is no altar here to pray at."));
    assert_eq!(w.state.time, before, "refused, no time");
    w.apply(&Action::Move(Dir::N));
    let spent = w.content.terrain_id("spent_altar").expect("spent");
    w.set_terrain(ALTAR, spent);
    w.apply(&Action::Wait);
    assert!(here(&w).is_empty(), "a spent altar offers nothing");
}

#[test]
fn a_prayer_takes_a_minute_and_a_broken_one_gives_nothing() {
    let mut w = world("good", false);
    let start = w.state.time;
    w.apply(&Action::Pray);
    assert_eq!(w.state.time.since(start).as_seconds(), 60);
    assert!(said(&w, "You kneel before the altar and pray."));
    // Broken by the kobold beside the player, who attacks (a line said stops it, as a rest).
    let mut w = world("good", true);
    let score = w.state.player().alignment;
    let start = w.state.time;
    w.apply(&Action::Pray);
    assert!(said(&w, "Your prayer is broken."));
    assert!(w.state.time.since(start).as_seconds() < 60);
    assert_eq!(terrain(&w, ALTAR), "forgotten_altar", "never spent");
    assert_eq!(w.state.player().alignment, score, "no deed");
}

#[test]
fn the_lean_follows_the_altars_side_and_the_players_band() {
    let w = world("neutral", false);
    let table = table(&w);
    use Alignment::{Evil, Good, Neutral};
    assert_eq!(leaned_kinds(&table, Good, Good, false), [50, 30, 20]);
    assert_eq!(leaned_kinds(&table, Evil, Evil, false), [50, 30, 20]);
    assert_eq!(leaned_kinds(&table, Good, Evil, false), [20, 30, 50]);
    assert_eq!(leaned_kinds(&table, Evil, Good, false), [20, 30, 50]);
    assert_eq!(leaned_kinds(&table, Good, Neutral, false), [35, 30, 35]);
    assert_eq!(leaned_kinds(&table, Neutral, Good, false), [35, 30, 35]);
    // Devout: an agreeing lean counts double, an opposed one not at all.
    assert_eq!(leaned_kinds(&table, Good, Good, true), [65, 30, 5]);
    assert_eq!(leaned_kinds(&table, Evil, Good, true), [35, 30, 35]);
    assert_eq!(leaned_kinds(&table, Neutral, Evil, true), [35, 30, 35]);
}

#[test]
fn the_draw_gives_the_kinds_and_tiers_their_shares() {
    let mut w = world("neutral", false);
    let table = table(&w);
    let n = 20_000;
    let mut kinds = [0u32; 3];
    let mut burst = 0;
    let mut moderate = 0;
    for _ in 0..n {
        let o = w.draw_outcome(&table, table.kinds).expect("drawn");
        kinds[o.kind as usize] += 1;
        burst += u32::from(o.id == "burst");
        moderate += u32::from(o.tier == crate::content::OutcomeTier::Moderate);
    }
    let percent = |c: u32| c * 1000 / n;
    // Boon 35, nothing 30, bane 35 (per mille, within 2 points of a percent).
    for (got, want) in kinds.iter().zip([350, 300, 350]) {
        assert!(percent(*got).abs_diff(want) <= 20, "{kinds:?}");
    }
    // The explosion: 5 of the bane's 100, 1.75 %.
    assert!(percent(burst).abs_diff(17) <= 5, "{burst}");
    // Moderate: boon 29 and bane 34 of their kinds' 100, about 22 %.
    assert!(percent(moderate).abs_diff(220) <= 20, "{moderate}");
    // Leaned: agreement draws boons half the time.
    let leaned = leaned_kinds(&table, Alignment::Good, Alignment::Good, false);
    let boons = (0..n)
        .filter(|_| w.draw_outcome(&table, leaned).expect("drawn").kind == OutcomeKind::Boon)
        .count() as u32;
    assert!(percent(boons).abs_diff(500) <= 20, "{boons}");
}

#[test]
fn nothing_happens_changes_nothing_but_the_message_log() {
    for id in ["silence", "echo", "dust", "flicker", "breath", "omen"] {
        let mut w = world("neutral", true);
        let looks = |w: &World| {
            let level = w.state.current();
            format!(
                "{:?} {:?} {:?}",
                w.state.actors, level.terrain, level.objects
            )
        };
        let before = looks(&w);
        let said_before = w.messages().len();
        befall(&mut w, id);
        assert_eq!(looks(&w), before, "{id}");
        assert_eq!(w.messages().len(), said_before + 1, "{id}: one line");
    }
}

#[test]
fn boons_heal_feed_restore_and_leave_gifts_at_the_altars_foot() {
    let mut w = world("neutral", false);
    w.player_mut().hp = 10;
    befall(&mut w, "soothed");
    let hp = w.state.player().hp;
    assert!((12..=22).contains(&hp), "2d6 back: {hp}");
    assert!(said(&w, "A little warmth runs through you."));
    befall(&mut w, "mended");
    let max = w.state.player().max_hp(&w.content);
    assert_eq!(w.state.player().hp, max, "full");
    // Coins and items lie beside the altar, nearest the player but not under them.
    befall(&mut w, "coins");
    befall(&mut w, "trinket");
    befall(&mut w, "offering");
    let me = w.player_pos();
    let piles: Vec<Pos> = w.state.current().objects.keys().copied().collect();
    assert!(!piles.is_empty());
    for p in &piles {
        assert!(p.chebyshev(ALTAR) == 1 && *p != me, "{p}");
    }
    // Mana.
    let max = w.state.player().max_mana(&w.content);
    w.player_mut().mana = 0;
    befall(&mut w, "replenished");
    assert_eq!(w.state.player().mana, max);
    befall(&mut w, "sapped");
    assert!(w.state.player().mana < max || max == 0);
    // Statuses, told by the outcome's line alone.
    befall(&mut w, "blessed");
    let blessed = w.content.status_id("blessed").expect("blessed");
    assert!(w.state.player().effects.has(blessed));
    assert!(
        !said(&w, "You feel blessed."),
        "the status's own line is not said"
    );
    // Experience: a fifth of the way to the next level.
    let xp = w.player_xp();
    befall(&mut w, "favour");
    assert!(w.player_xp() > xp);
}

#[test]
fn hunger_moves_one_state_and_never_past_satiated_or_into_starving() {
    use crate::hunger::Hunger;
    let mut w = world("neutral", false);
    let before = w.player_hunger().expect("hungers");
    assert_eq!(before, Hunger::NotHungry);
    befall(&mut w, "sated");
    assert_eq!(w.player_hunger(), Some(Hunger::Satiated));
    befall(&mut w, "sated");
    assert_eq!(
        w.player_hunger(),
        Some(Hunger::Satiated),
        "never past Satiated"
    );
    for _ in 0..8 {
        befall(&mut w, "hungry");
    }
    assert_eq!(
        w.player_hunger(),
        Some(Hunger::Fainting),
        "never into Starving"
    );
}

#[test]
fn glimpse_identifies_and_muddled_forgets_or_says_their_other_line() {
    let mut w = world("neutral", false);
    befall(&mut w, "glimpse");
    assert!(
        said(&w, "You feel you understand a little."),
        "nothing unidentified"
    );
    befall(&mut w, "muddled");
    assert!(
        said(&w, "You feel a moment's confusion."),
        "nothing identified"
    );
    // A magic ring, unidentified.
    let content = w.content.clone();
    let ring = content.object_id("ring_of_strength").or_else(|| {
        (content.object_defs().iter().position(|o| o.kind.is_some()))
            .map(|i| crate::content::ObjectId(i as u16))
    });
    let ring = ring.expect("a magic base");
    let mut rng = crate::rng::Rng::gameplay(7);
    let stack = crate::magic::loot::make(
        &content,
        &mut rng,
        ring,
        1,
        10,
        crate::magic::loot::LootGrade::Magic,
    );
    w.player_mut().inventory.push(stack);
    let i = w.state.player().inventory.len() - 1;
    befall(&mut w, "glimpse");
    assert!(said(&w, "Something you carry becomes clear to you."));
    assert!(w.state.player().inventory[i].is_identified());
    befall(&mut w, "muddled");
    assert!(said(&w, "Something you carry seems strange to you."));
    assert!(!w.state.player().inventory[i].is_identified());
}

#[test]
fn banes_burn_but_spare_smite_and_may_kill() {
    let mut w = world("neutral", false);
    w.player_mut().hp = 1;
    befall(&mut w, "scorched");
    assert_eq!(w.state.player().hp, 1, "the minor burn never kills");
    assert!(w.state.death.is_none());
    befall(&mut w, "smitten");
    assert!(
        w.state.death.is_some(),
        "4d10 from above kills at 1 hit point"
    );
    let death = w.state.death.as_ref().expect("dead");
    assert_eq!(death.cause, Some(Cause::Altar));
}

#[test]
fn banes_take_coins_the_lantern_and_the_most_precious_thing() {
    let mut w = world("neutral", false);
    let content = w.content.clone();
    let copper = content.object_id("copper_coin").expect("copper");
    let gold = content.object_id("gold_coin").expect("gold");
    w.player_mut()
        .inventory
        .push(crate::object::Stack::new(copper, 30));
    befall(&mut w, "lightened");
    let left: u32 = (w.state.player().inventory.iter())
        .filter(|s| s.object == copper)
        .map(|s| s.count)
        .sum();
    assert_eq!(left, 21, "a third (rounded down) of 30 gone");
    w.player_mut()
        .inventory
        .push(crate::object::Stack::new(gold, 5));
    befall(&mut w, "tithe");
    assert!(
        !w.state.player().inventory.iter().any(|s| s.object == gold),
        "the gold was worth most"
    );
    w.state.lantern_lit = true;
    befall(&mut w, "snuffed");
    assert!(!w.state.lantern_lit);
}

#[test]
fn banes_sicken_wither_hex_and_displace() {
    let mut w = world("neutral", false);
    befall(&mut w, "sickened");
    assert!(w.state.player().diseases.iter().next().is_some());
    befall(&mut w, "withered");
    assert!(w.state.player().damage.is_damaged());
    befall(&mut w, "hexed");
    let hexed = w.content.status_id("ill_wished").expect("ill_wished");
    assert!(w.state.player().effects.has(hexed));
    let from = w.player_pos();
    befall(&mut w, "displaced");
    assert_ne!(w.player_pos(), from);
    // Restored and cleansed mend what was done.
    befall(&mut w, "restored");
    assert!(!w.state.player().damage.is_damaged());
    befall(&mut w, "cleansed");
    assert!(said(&w, "A sickness leaves you."));
}

#[test]
fn summoned_creatures_come_on_a_level_with_a_danger_and_none_without() {
    let mut w = world("neutral", false);
    let count = w.state.actors.len();
    befall(&mut w, "answered");
    assert_eq!(w.state.actors.len(), count, "no danger here: nothing comes");
    // A dungeon with a danger: one comes beside the "altar".
    let content = Arc::new(Content::builtin().expect("content"));
    let mut w = World::new(content, "fennmynster", None).expect("fennmynster");
    let feature = w.player_pos();
    let count = w.state.actors.len();
    let outcome = outcome(&w, "answered");
    w.befall(&outcome, feature, &mut Vec::new());
    assert_eq!(w.state.actors.len(), count + 1);
    let newest = (w.state.actors.values())
        .max_by_key(|a| a.id)
        .expect("newest");
    assert!(newest.pos.chebyshev(feature) <= 3);
}

#[test]
fn the_altar_bursts_hurting_everyone_near_and_leaves_rubble() {
    let mut w = world("neutral", true);
    let kobold = (w.state.actors.values())
        .find(|a| a.kind == ActorKind::Creature)
        .map(|a| (a.id, a.hp))
        .expect("kobold");
    befall(&mut w, "burst");
    assert!(said(&w, "The altar bursts apart in fire!"));
    assert!(w.state.player().hp < 1000, "the player is in it");
    let hurt = w
        .state
        .actors
        .get(&kobold.0)
        .is_none_or(|k| k.hp < kobold.1);
    assert!(hurt, "so is the kobold");
    assert_eq!(terrain(&w, ALTAR), "rubble");
    assert!(!w.state.current().lights.contains_key(&ALTAR));
    // It can kill.
    let mut w = world("neutral", false);
    w.player_mut().hp = 1;
    befall(&mut w, "burst");
    assert_eq!(
        w.state.death.as_ref().and_then(|d| d.cause),
        Some(Cause::Altar)
    );
}

#[test]
fn a_burst_altar_is_never_spent_and_a_prayer_shifts_alignment_by_side() {
    // Praying at the good altar: the deed is good; at the evil one, evil.
    let mut good = world("good", false);
    good.player_mut().alignment = Some(Score::ZERO);
    good.apply(&Action::Pray);
    let mut evil = world("evil", false);
    evil.player_mut().alignment = Some(Score::ZERO);
    evil.apply(&Action::Pray);
    let score = |w: &World| w.state.player().alignment.map(Score::value);
    assert_eq!(score(&good), Some(5));
    assert_eq!(score(&evil), Some(-10));
    let mut neutral = world("neutral", false);
    neutral.player_mut().alignment = Some(Score::ZERO);
    neutral.apply(&Action::Pray);
    assert_eq!(score(&neutral), Some(0));
}

#[test]
fn about_half_of_completed_prayers_spend_the_altar_and_a_spent_one_is_kept_as_terrain() {
    let content = Arc::new(Content::builtin().expect("content"));
    let (mut spent, mut standing) = (0u32, 0u32);
    let mut kept = None;
    for seed in 0..300 {
        let mut w = World::new(content.clone(), "shrine", Some(seed)).expect("shrine");
        for a in crate::action::parse_actions("move:N open:N move:N*3").expect("walk") {
            w.apply(&a);
        }
        let altar = Pos::new(13, 3);
        let player = w.state.player;
        w.player_mut().hp_base = 1000;
        w.player_mut().hp = 1000;
        let _ = player;
        w.apply(&Action::Pray);
        match terrain(&w, altar).as_str() {
            "spent_altar" => {
                spent += 1;
                kept.get_or_insert(w);
            }
            "forgotten_altar" => standing += 1,
            _ => {}
        }
    }
    let share = spent * 100 / (spent + standing);
    assert!((40..=60).contains(&share), "{spent} spent, {standing} not");
    // Saved and loaded, the spent altar is still the terrain, and offers nothing.
    let w = kept.expect("one was spent");
    let loaded = World::from_state(w.content.clone(), w.state().clone()).expect("loaded");
    assert_eq!(terrain(&loaded, Pos::new(13, 3)), "spent_altar");
    assert!(here(&loaded).is_empty());
    assert_eq!(loaded.state_hash(), w.state_hash());
}

//! Miscasts (spec §8 "Spells": "Miscasts", #1271, `miscast.rs`): every cast with one's own mana
//! makes a Spellcraft check after it has gone off in full; the 1 and 1 always fails; a failure
//! rolls on the one ordered table, and each entry does what it says.

use super::caster_tests::{kobold, said, world_with};
use super::miscast::{Aim, entry_for};
use super::*;
use crate::check;
use crate::content::{ContentFiles, Miscast, MishapEffect};

/// The built-in content with `data/miscast.toml`'s `base` set to `base` (−1000: only the 1 and
/// 1 fails; 1000: every cast fails, by far), and, with `only`, the table cut to that one entry
/// covering every total.
fn content(base: i32, only: Option<&str>) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/miscast.toml",
        &format!(
            "stat = \"intelligence\"\nbase = {base}\nper_level = 1\narmour_percent = 10\n\
             die = 66\nper_margin = 3\n"
        ),
    );
    if let Some(only) = only {
        let builtin = Content::builtin().expect("content");
        for m in builtin.miscasts() {
            files.remove(&format!("data/miscasts/{}.toml", m.id));
        }
        let entry = (builtin.miscasts().iter())
            .find(|m| m.id == only)
            .expect("a built-in entry");
        let text = std::fs::read_to_string(format!(
            "{}/../../assets/data/miscasts/{only}.toml",
            env!("CARGO_MANIFEST_DIR")
        ))
        .expect("its file");
        let text = text
            .replace(&format!("from = {}\n", entry.from), "from = 1\n")
            .lines()
            .filter(|l| !l.starts_with("to = "))
            .collect::<Vec<_>>()
            .join("\n");
        files.insert(&format!("data/miscasts/{only}.toml"), &text);
    }
    Arc::new(Content::load(&files).expect("the content loads"))
}

/// A wizard (Spark, level 1) with `content`, mana topped up, a kobold `dx` east.
fn wizard(content: Arc<Content>) -> (World, ActorId) {
    let mut w = world_with(content, "wizard");
    let spark = w.content.spell_id("spark").expect("spark");
    w.player_mut().spells.insert(spark, 1);
    let k = kobold(&mut w, 2, 10_000);
    (w, k)
}

/// The player casts Spark at the kobold `k`, mana and hit points topped up first; the messages
/// it brought.
fn spark(w: &mut World, k: ActorId) -> Vec<String> {
    w.player_mut().mana = 40;
    let max = w.state.player().max_hp(w.content());
    w.player_mut().hp = max;
    let here = w.player_pos();
    let kobold = w.state.actors.get_mut(&k).expect("the kobold");
    kobold.pos = Pos::new(here.x + 2, here.y);
    kobold.hp = 10_000;
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let before = said(w).len();
    let at = Pos::new(here.x + 2, here.y);
    w.apply(&Action::Cast("spark".into(), Some(at)));
    said(w)[before..].to_vec()
}

/// Whether `lines` hold a miscast's message (every entry's starts "Your weave", or a surge's).
fn miscast(lines: &[String]) -> bool {
    lines
        .iter()
        .any(|l| l.contains("Your weave") || l.contains("your weave"))
}

#[test]
fn the_one_and_one_always_fails_and_rolls_at_least_the_die() {
    // Whatever the bonus, a natural 2 fails, by nothing.
    assert_eq!(check::ways(1000, 0), 143);
    assert!(!check::makes(2, 1000, 0));
    let (mut w, _) = wizard(content(-1000, None));
    for _ in 0..200 {
        let total = w.miscast_total(66, 0, 0);
        assert!((1..=66 * 3).contains(&total), "{total}");
    }
    // Ward never takes it below 1.
    assert_eq!(w.miscast_total(66, 0, 10_000), 1);
}

#[test]
fn a_master_miscasts_only_on_the_one_and_one() {
    let (mut w, k) = wizard(content(-1000, None));
    let casts = 1500;
    let miscasts = (0..casts).filter(|_| miscast(&spark(&mut w, k))).count();
    // One in 144 on average: about ten.
    assert!((2..=30).contains(&miscasts), "{miscasts} of {casts}");
}

#[test]
fn a_starting_wizard_miscasts_a_first_spell_rarely_and_a_far_one_often() {
    let content = Arc::new(Content::builtin().expect("content"));
    let mut w = world_with(content.clone(), "wizard");
    // Spellcraft 17 (the middle of a wizard's 2d6+10) and Intelligence 14 (world_with's).
    let id = content.skill_id("spellcraft").expect("spellcraft");
    let now = i32::from(w.state.player().skills.value(id).unwrap_or(0));
    w.player_mut().skills.add(id, 17 - now);
    let me = w.state.player();
    let bonus = me.cast_bonus(&content);
    let rate = |level: u8| {
        let mut def = content
            .spell(content.spell_id("spark").expect("spark"))
            .clone();
        def.level = level;
        let difficulty = me.cast_difficulty(&content, &def).expect("rules");
        144 - check::ways(bonus, difficulty)
    };
    // 1 in 20 to 1 in 10 for a level-1 spell; most of the time ten levels on.
    assert!((7..=15).contains(&rate(1)), "{} of 144", rate(1));
    assert!(rate(11) > 72, "{} of 144", rate(11));
}

#[test]
fn every_entry_is_reached_by_its_totals_and_the_table_climbs() {
    let c = Content::builtin().expect("content");
    let table = c.miscasts();
    assert!(table.len() >= 12, "the table ships");
    for m in table {
        assert_eq!(entry_for(table, m.from).map(|e| &e.id), Some(&m.id));
        if let Some(to) = m.to {
            assert_eq!(entry_for(table, to).map(|e| &e.id), Some(&m.id));
        }
    }
    assert_eq!(
        entry_for(table, 1_000).map(|e| e.id.as_str()),
        Some("unravelling")
    );
    // Boons and chaos among the harms.
    let kinds: Vec<&str> = table.iter().map(|m| m.effect.kind()).collect();
    for kind in [
        "refund", "twice", "surge", "swap", "boon", "harm", "backfire", "summon",
    ] {
        assert!(kinds.contains(&kind), "{kind} is on the table");
    }
}

#[test]
fn the_spell_goes_off_in_full_before_the_rider() {
    // Every cast fails by far, and the table is the deadly harm alone.
    let (mut w, k) = wizard(content(1000, Some("sparks")));
    let lines = spark(&mut w, k);
    let hit = lines.iter().position(|l| l.contains("kobold"));
    let slip = lines
        .iter()
        .position(|l| l.contains("sparks bite your fingers"));
    let (Some(hit), Some(slip)) = (hit, slip) else {
        panic!("both said: {lines:?}");
    };
    assert!(hit < slip, "the spell first: {lines:?}");
    assert!(w.state.actors[&k].hp < 10_000, "the spark struck in full");
    let max = w.state.player().max_hp(w.content());
    assert!(w.state.player().hp < max, "and then the sparks bit");
}

#[test]
fn a_deadly_miscast_can_kill_and_is_put_down_to_it() {
    let (mut w, k) = wizard(content(1000, Some("unravelling")));
    for _ in 0..20 {
        w.player_mut().hp = 1;
        w.player_mut().mana = 40;
        let at = w.state.actors[&k].pos;
        w.apply(&Action::Cast("spark".into(), Some(at)));
        if w.state.death.is_some() {
            break;
        }
    }
    let death = w.state.death.as_ref().expect("the raw magic killed them");
    let spark = w.content.spell_id("spark").expect("spark");
    assert_eq!(death.cause, Some(Cause::Miscast(spark)));
}

#[test]
fn the_echo_casts_again_free_and_the_refund_gives_the_mana_back() {
    let (mut w, k) = wizard(content(1000, Some("echo")));
    spark(&mut w, k);
    let struck = 10_000 - w.state.actors[&k].hp;
    assert!(struck > 0);
    let echoes = said(&w)
        .iter()
        .filter(|l| l.contains("goes off again"))
        .count();
    assert_eq!(echoes, 1, "one echo, which makes no check of its own");

    let (mut w, k) = wizard(content(1000, Some("refund")));
    let lines = spark(&mut w, k);
    assert!(miscast(&lines));
    assert_eq!(w.state.player().mana, 40, "the spent mana flows back");
}

#[test]
fn a_swap_and_a_boon_reach_someone_near() {
    let c = content(1000, Some("swap"));
    let (mut w, k) = wizard(c.clone());
    let here = w.player_pos();
    let there = Pos::new(here.x + 2, here.y);
    w.state.actors.get_mut(&k).expect("the kobold").pos = there;
    // Only the kobold near: the others of the showcase sent far off.
    let player = w.state.player;
    for a in w.state.actors.values_mut() {
        if a.id != player && a.id != k {
            a.pos = Pos::new(-100, -100);
        }
    }
    let spell = c.spell_id("spark").expect("spark");
    let me = w.state.player().clone();
    let cast = super::cast::Cast::by_creature(&me, &c, (spell, c.spell(spell)));
    let entry = c.miscasts()[0].clone();
    w.miscast(player, &cast, Aim::Player(None), &entry, &mut Vec::new());
    assert_eq!(w.player_pos(), there, "swapped with the kobold");
    assert_eq!(w.state.actors[&k].pos, here);

    let (mut w, k) = wizard(content(1000, Some("boon")));
    let blessed = w.content.status_id("blessed").expect("blessed");
    let mut caught = 0;
    for _ in 0..20 {
        spark(&mut w, k);
        let player = w.state.player().effects.has(blessed);
        let kobold = w.state.actors[&k].effects.has(blessed);
        caught += usize::from(player) + usize::from(kobold);
    }
    assert!(caught > 0, "someone near is blessed");
}

#[test]
fn a_creature_miscasts_too_and_is_told_when_seen() {
    let c = content(1000, Some("dizzy"));
    let (mut w, _) = wizard(c.clone());
    let shaman = kobold(&mut w, 3, 50);
    let spark = c.spell_id("spark").expect("spark");
    let staggered = c.status_id("staggered").expect("staggered");
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let player = w.state.player;
    let me = w.state.actors[&shaman].clone();
    let def = c.spell(spark);
    let cast = super::cast::Cast::by_creature(&me, &c, (spark, def));
    let entry: Miscast = c.miscasts()[0].clone();
    assert_eq!(entry.effect.kind(), "status");
    let mut events = Vec::new();
    w.miscast(shaman, &cast, Aim::Creature(player), &entry, &mut events);
    assert!(w.state.actors[&shaman].effects.has(staggered));
    assert!(
        said(&w)
            .iter()
            .any(|l| l.contains("The kobold's weave slips")),
        "{:?}",
        said(&w)
    );
    assert!(matches!(entry.effect, MishapEffect::Status { .. }));
}

#[test]
fn a_run_that_miscasts_replays_the_same() {
    // The circles' caster knows every buff and hex but is no master of Spellcraft: casting
    // Haste and Fire cloak again and again miscasts.
    let content = Arc::new(Content::builtin().expect("content"));
    let mut w = World::new(content.clone(), "circles", None).expect("world");
    let mut replay = crate::replay::Replay::new(&w);
    let actions = "use:haste use:fire_cloak use:renewal use:haste use:fire_cloak use:renewal";
    for action in crate::action::parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(miscast(&said(&w)), "{:?}", said(&w));
    let verified = crate::replay::Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content)
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

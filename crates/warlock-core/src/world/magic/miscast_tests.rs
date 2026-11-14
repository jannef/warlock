//! Miscasts (spec §8 "Spells": "Miscasts", #1271, `miscast.rs`): every cast with one's own mana
//! makes a Spellcraft check after it has gone off in full; the 1 and 1 always fails; a failure
//! rolls on the one ordered table, and each entry does what it says. The check and the total of
//! the miscasts design (#1288): the rank eases the check, the miss carries the total, and the
//! protected half cannot kill.

use super::caster_tests::{kobold, world_with};
use super::miscast::{Aim, entry_for};
use crate::check;
use crate::content::{ContentFiles, Miscast, MishapEffect, ZoneKind};
use crate::dice::Dice;
use crate::test_support::{CONTENT as BUILTIN, builtin, said};
use crate::world::*;

/// The built-in content with `data/miscast.toml`'s `base` set to `base` (−1000: only the 1 and
/// 1 fails; 1000: every cast fails, by far), and, with `only`, the table cut to that one entry
/// covering every total.
fn content(base: i32, only: Option<&str>) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert(
        "data/miscast.toml",
        &format!(
            "stat = \"intelligence\"\nbase = {base}\nper_level = 1\narmour_percent = 10\n\
             rank_ease = 1\ndie = \"2d12\"\nper_margin = 5\nsafe_below = 71\nnear = 6\n"
        ),
    );
    if let Some(only) = only {
        let builtin = builtin();
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

/// Whether `lines` hold a miscast's message.
fn miscast(lines: &[String]) -> bool {
    (BUILTIN.miscasts().iter()).any(|m| lines.contains(&m.message))
}

/// The built-in entry `id`.
fn entry(c: &Content, id: &str) -> Miscast {
    (c.miscasts().iter())
        .find(|m| m.id == id)
        .cloned()
        .expect("a built-in entry")
}

/// Sends every actor but the player and `keep` far off.
fn clear_round(w: &mut World, keep: &[ActorId]) {
    let player = w.state.player;
    for a in w.state.actors.values_mut() {
        if a.id != player && !keep.contains(&a.id) {
            a.pos = Pos::new(-100, -100);
        }
    }
}

#[test]
fn the_one_and_one_always_fails_and_rolls_at_least_the_die() {
    // Whatever the bonus, a natural 2 fails, by nothing.
    assert_eq!(check::ways(1000, 0), 143);
    assert!(!check::makes(2, 1000, 0));
    let (mut w, _) = wizard(content(-1000, None));
    let die: Dice = "2d12".parse().expect("dice");
    for _ in 0..200 {
        let total = w.miscast_total(die, 0, 0);
        assert!((2..=24).contains(&total), "{total}");
    }
    // Ward never takes it below 1.
    assert_eq!(w.miscast_total(die, 0, 10_000), 1);
}

#[test]
fn the_miss_carries_the_total() {
    let (mut w, _) = wizard(content(-1000, None));
    let rules = *w.content.miscast_rules().expect("rules");
    assert_eq!(rules.per_margin, 5);
    // A miss of 10 lands from 52 to 74: the edge of the protected half.
    for _ in 0..200 {
        let total = w.miscast_total(rules.die, rules.per_margin * 10, 0);
        assert!((52..=74).contains(&total), "{total}");
    }
    // A near miss stays in the first dozen rows, whatever the die.
    let table = w.content.miscasts().to_vec();
    for _ in 0..200 {
        let total = w.miscast_total(rules.die, 0, 0);
        let hit = entry_for(&table, total).expect("an entry");
        assert!(rules.protects(hit) && hit.to.is_some_and(|to| to <= 24));
    }
}

#[test]
fn each_rank_above_the_first_eases_the_check() {
    let content = builtin();
    let mut w = world_with(content.clone(), "wizard");
    let spark = content.spell_id("spark").expect("spark");
    let def = content.spell(spark);
    w.player_mut().spells.insert(spark, 1);
    let first = (w.state.player().cast_difficulty(&content, (spark, def))).expect("rules");
    w.player_mut().spells.insert(spark, 5);
    let fifth = (w.state.player().cast_difficulty(&content, (spark, def))).expect("rules");
    assert_eq!(first - fifth, 4, "four ranks above the first, four easier");
    // A creature's spells are rank 1.
    let k = kobold(&mut w, 2, 10);
    let kobold = &w.state.actors[&k];
    assert_eq!(kobold.cast_difficulty(&content, (spark, def)), Some(first));
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
    let content = builtin();
    let mut w = world_with(content.clone(), "wizard");
    // Spellcraft 17 (the middle of a wizard's 2d6+10) and Intelligence 14 (world_with's).
    let id = content.skill_id("spellcraft").expect("spellcraft");
    let now = i32::from(w.state.player().skills.value(id).unwrap_or(0));
    w.player_mut().skills.add(id, 17 - now);
    let me = w.state.player();
    let bonus = me.cast_bonus(&content);
    let spark = content.spell_id("spark").expect("spark");
    let rate = |level: u8| {
        let mut def = content.spell(spark).clone();
        def.level = level;
        let difficulty = me.cast_difficulty(&content, (spark, &def)).expect("rules");
        144 - check::ways(bonus, difficulty)
    };
    // 1 in 20 to 1 in 10 for a level-1 spell; most of the time ten levels on.
    assert!((7..=15).contains(&rate(1)), "{} of 144", rate(1));
    assert!(rate(11) > 72, "{} of 144", rate(11));
}

#[test]
fn every_entry_is_reached_by_its_totals_and_the_table_climbs() {
    let c = builtin();
    let table = c.miscasts();
    assert_eq!(table.len(), 65, "the table ships its 65 rows");
    for m in table {
        assert_eq!(entry_for(table, m.from).map(|e| &e.id), Some(&m.id));
        if let Some(to) = m.to {
            assert_eq!(entry_for(table, to).map(|e| &e.id), Some(&m.id));
        }
    }
    assert_eq!(
        entry_for(table, 1_000).map(|e| e.id.as_str()),
        Some("oblivion")
    );
    // Boons and chaos among the harms.
    let kinds: Vec<&str> = table.iter().map(|m| m.effect.kind()).collect();
    for kind in [
        "refund", "twice", "swap", "boon", "harm", "backfire", "noise", "zone", "drain",
    ] {
        assert!(kinds.contains(&kind), "{kind} is on the table");
    }
    assert!(!kinds.contains(&"surge"), "Surge is gone");
    // The protected half: 35 rows, the last ending at 70.
    let rules = c.miscast_rules().expect("rules");
    let protected: Vec<&Miscast> = table.iter().filter(|m| rules.protects(m)).collect();
    assert_eq!(protected.len(), 35);
    assert_eq!(protected.last().and_then(|m| m.to), Some(70));
}

#[test]
fn a_protected_harm_never_kills() {
    let c = builtin();
    let (mut w, _) = wizard(c.clone());
    let player = w.state.player;
    let spell = c.spell_id("spark").expect("spark");
    let me = w.state.player().clone();
    let cast = super::cast::Cast::by_creature(&me, &c, (spell, c.spell(spell)));
    let blast = entry(&c, "blast");
    assert!(c.miscast_rules().expect("rules").protects(&blast));
    // At 1 hit point it takes nothing; at 5 it is left at 1 at the least.
    for _ in 0..30 {
        w.player_mut().hp = 1;
        w.miscast(player, &cast, Aim::Player(None), &blast, &mut Vec::new());
        assert_eq!(w.state.player().hp, 1);
        w.player_mut().hp = 5;
        w.miscast(player, &cast, Aim::Player(None), &blast, &mut Vec::new());
        assert!((1..=5).contains(&w.state.player().hp));
        assert!(w.state.death.is_none());
    }
    // From 71 the harm may kill.
    let rend = entry(&c, "rend");
    assert!(!c.miscast_rules().expect("rules").protects(&rend));
    w.player_mut().hp = 1;
    w.miscast(player, &cast, Aim::Player(None), &rend, &mut Vec::new());
    assert!(w.state.death.is_some(), "rend kills at 1 hit point");
}

#[test]
fn the_loader_keeps_what_could_kill_off_the_protected_half() {
    let load = |effect: &str| {
        let mut files = ContentFiles::builtin();
        files.insert(
            "data/miscasts/glimmer.toml",
            &format!(
                "[[miscast]]\nid = \"glimmer\"\nfrom = 1\nto = 2\nmessage = \"x\"\n\
                 seen = \"{{who}}\"\neffect = {{ {effect} }}\n"
            ),
        );
        Content::load(&files)
    };
    for refused in [
        "kind = \"backfire\"",
        "kind = \"drain\", stat = \"random\", points = 1, difficulty = 16",
        "kind = \"summon\", creature = \"kobold\"",
        "kind = \"portal\", gate = \"rift\"",
        "kind = \"zone\", zone = \"fire\", radius = 1, turns = 5",
        "kind = \"status\", status = \"bleeding\"",
    ] {
        assert!(load(refused).is_err(), "{refused} is refused below 71");
    }
    for allowed in [
        "kind = \"zone\", zone = \"smoke\", radius = 1, turns = 5",
        "kind = \"noise\", loudness = 30",
        "kind = \"status\", status = \"staggered\"",
        "kind = \"harm\", dice = \"3d6\"",
    ] {
        assert!(load(allowed).is_ok(), "{allowed} is allowed below 71");
    }
}

#[test]
fn a_noise_is_heard_and_a_zone_is_laid() {
    let c = builtin();
    let (mut w, k) = wizard(c.clone());
    clear_round(&mut w, &[k]);
    let player = w.state.player;
    let spell = c.spell_id("spark").expect("spark");
    let me = w.state.player().clone();
    let cast = super::cast::Cast::by_creature(&me, &c, (spell, c.spell(spell)));
    w.state
        .actors
        .get_mut(&k)
        .expect("the kobold")
        .heard
        .clear();
    let thunder = entry(&c, "thunder");
    w.miscast(player, &cast, Aim::Player(None), &thunder, &mut Vec::new());
    assert!(!w.state.actors[&k].heard.is_empty(), "the kobold heard it");

    let fog = entry(&c, "fog");
    w.miscast(player, &cast, Aim::Player(None), &fog, &mut Vec::new());
    let here = w.player_pos();
    let zones = &w.state.current().zones;
    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].kind(), ZoneKind::Smoke);
    assert!(zones[0].cells().any(|p| p == here), "around the caster");
}

#[test]
fn a_creatures_swap_never_takes_the_player() {
    let c = builtin();
    let (mut w, k) = wizard(c.clone());
    clear_round(&mut w, &[k]);
    let spell = c.spell_id("spark").expect("spark");
    let me = w.state.actors[&k].clone();
    let cast = super::cast::Cast::by_creature(&me, &c, (spell, c.spell(spell)));
    let (here, there) = (w.player_pos(), w.state.actors[&k].pos);
    let player = w.state.player;
    let slip = entry(&c, "slip");
    w.miscast(k, &cast, Aim::Creature(player), &slip, &mut Vec::new());
    assert_eq!(w.player_pos(), here, "the player stays");
    assert_eq!(w.state.actors[&k].pos, there, "no one else near: nothing");
}

#[test]
fn the_spell_goes_off_in_full_before_the_rider() {
    // Every cast fails by far, and the table is the deadly harm alone.
    let (mut w, k) = wizard(content(1000, Some("sparks")));
    let lines = spark(&mut w, k);
    let hit = lines.iter().position(|l| l.contains("kobold"));
    let slip = lines
        .iter()
        .position(|l| l.contains("Sparks leap from your fingers"));
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
        .filter(|l| l.contains("sounds twice"))
        .count();
    assert_eq!(echoes, 1, "one echo, which makes no check of its own");

    let (mut w, k) = wizard(content(1000, Some("refund")));
    let lines = spark(&mut w, k);
    assert!(miscast(&lines));
    assert_eq!(w.state.player().mana, 40, "the spent mana flows back");
}

#[test]
fn a_swap_and_a_boon_reach_someone_near() {
    let c = content(1000, Some("slip"));
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

    let (mut w, k) = wizard(content(1000, Some("blessing")));
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
    let c = content(1000, Some("wobble"));
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
        said(&w).iter().any(|l| l.contains("The kobold reels")),
        "{:?}",
        said(&w)
    );
    assert!(matches!(entry.effect, MishapEffect::Status { .. }));
}

#[test]
fn a_seen_line_names_the_caster_in_lower_case_inside_it() {
    let c = content(1000, Some("sparks"));
    let (mut w, _) = wizard(c.clone());
    let shaman = kobold(&mut w, 3, 50);
    let spark = c.spell_id("spark").expect("spark");
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
    let player = w.state.player;
    let me = w.state.actors[&shaman].clone();
    let cast = super::cast::Cast::by_creature(&me, &c, (spark, c.spell(spark)));
    let entry: Miscast = c.miscasts()[0].clone();
    w.miscast(
        shaman,
        &cast,
        Aim::Creature(player),
        &entry,
        &mut Vec::new(),
    );
    assert!(
        said(&w)
            .iter()
            .any(|l| l == "Sparks bite the kobold's fingers."),
        "{:?}",
        said(&w)
    );
}

#[test]
fn a_run_that_miscasts_replays_the_same() {
    // The circles' caster knows every buff and hex but is no master of Spellcraft: casting
    // Haste and Fire cloak again and again miscasts.
    let content = builtin();
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

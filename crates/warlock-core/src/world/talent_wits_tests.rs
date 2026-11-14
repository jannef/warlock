//! The Scholar, Grit, and Delver talents (spec §8 "Talents"; #240): each changes the one rule it
//! names; Skill focus raises the chosen trained skill, past 100 too; Treasure hunter's
//! prerequisite is either of two.

use super::search::{Hidden, Look};
use super::*;
use crate::action::{Meal, parse_actions};
use crate::burden::Burden;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Need};
use crate::damage::Damage;
use crate::hunger::Hunger;
use crate::magic::{LootGrade, loot};
use crate::noise::{self, Act};
use crate::object::Stack;
use crate::rng::Rng;
use crate::stats::{PerStat, Stats};
use crate::talent::{self, Candidate};
use crate::test_support::CONTENT as BUILTIN;

/// A human warrior in the showcase, every score 14.
fn world() -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    let mut w =
        World::with_character(BUILTIN.clone(), "showcase", Some(7), Some(&spec)).expect("world");
    w.player_mut().stats = Stats::new(PerStat::splat(14), PerStat::splat(40));
    w
}

/// A lit hall of `map` (`@` the player, `D` a training dummy that never fights back), the
/// player carrying `pack` (a scenario's `carry`) with a thousand hit points to spare.
fn hall(map: &str, pack: &str) -> World {
    let mut files = ContentFiles::builtin();
    // Reads without the check (`decipher_tests.rs` has it): these are about what a read does.
    files.remove("data/decipher.toml");
    files.insert(
        "data/creatures/dummy.toml",
        "[[creature]]\nid = \"dummy\"\nname = \"training dummy\"\nglyph = \"D\"\n\
         fg = \"#806040\"\nweight = 30000\nfaction = \"peaceful\"\nbase_hp = 1000\n",
    );
    files.insert(
        "levels/wits_hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"D\" = {{ terrain = \"floor\", creature = \"dummy\" }}\n"
        ),
    );
    files.insert(
        "scenarios/wits_hall.toml",
        &format!(
            "name = \"Hall\"\nstart_level = \"wits_hall\"\nseed = 7\nhp = 1000\ncarry = {pack}\n"
        ),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "wits_hall", None).expect("world")
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| world.apply(a))
        .collect()
}

fn said(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

/// The player takes talent `id` (one rank more), without a level to pick it at.
fn give(world: &mut World, id: &str) {
    world.player_mut().talents.take(id, None);
}

#[test]
fn scroll_savant_reads_in_half_the_time_and_is_spared_its_burst() {
    let pack = "[\"scroll_of_fireball*3\"]";
    let map = "#######\n#@D...#\n#######\n";
    let mut w = hall(map, pack);
    let me = w.state.player;
    let hp = w.state.actors[&me].hp;
    let plain = run(&mut w, "read:a:2,1").remove(0);
    assert!(w.state.actors[&me].hp < hp, "the burst caught the reader");
    let mut w = hall(map, pack);
    give(&mut w, "scroll_savant");
    assert!(w.perception().bursts_spare_me());
    let quick = run(&mut w, "read:a:2,1").remove(0);
    assert_eq!(w.state.actors[&me].hp, hp, "spared");
    assert!(
        said(&w).iter().any(|m| m.contains("training dummy")),
        "the dummy is caught still: {:?}",
        said(&w)
    );
    assert_eq!(quick.elapsed.as_seconds() * 2, plain.elapsed.as_seconds());
}

#[test]
fn spell_recall_now_and_then_keeps_the_scroll() {
    let read = |talents: &[&str]| {
        let mut w = hall(
            "##########\n#@.......#\n##########\n",
            "[\"scroll_of_fireball*20\"]",
        );
        for talent in talents {
            give(&mut w, talent);
        }
        run(&mut w, "read:a:7,1*12");
        let left = w.state.player().inventory.first().map_or(0, |s| s.count);
        let kept = (said(&w).iter())
            .filter(|m| *m == "The writing does not fade from the scroll.")
            .count();
        (left, kept)
    };
    assert_eq!(read(&[]), (8, 0));
    assert_eq!(
        read(&["scroll_savant"]),
        (8, 0),
        "not with the savant alone"
    );
    let (left, kept) = read(&["scroll_savant", "spell_recall"]);
    assert!(kept > 0 && kept < 12, "{kept}");
    assert_eq!(left as usize, 8 + kept);
}

#[test]
fn treasure_hunter_finds_magic_more_often_from_the_same_draws() {
    let content = &BUILTIN;
    let count = |luck: i32, grade: LootGrade| {
        let mut rng = Rng::gameplay(11);
        let mut rarities = [0u32; 3];
        for _ in 0..3000 {
            let source = crate::variant::Source::Loot;
            let luck = loot::Luck {
                magic: luck,
                gems: 0,
            };
            if let Some(s) = loot::roll_with(content, &mut rng, 6, grade, source, luck) {
                rarities[s.rarity() as usize] += 1;
            }
        }
        rarities
    };
    let [normal, magic, rare] = count(0, LootGrade::Any);
    let [normal_l, magic_l, rare_l] = count(50, LootGrade::Any);
    assert!(normal_l < normal && magic_l > magic && rare_l > rare);
    // A find that is magic at least (a creature's drop): rare more often.
    let [_, _, rare] = count(0, LootGrade::Magic);
    let [_, _, rare_l] = count(50, LootGrade::Magic);
    assert!(rare_l > rare);
    // The map is the same: only what the loot rolls changes.
    let id = "depths:2";
    let plain = crate::mapgen::generate(content, ("showcase", 5), id, None).expect("level");
    let lucky = crate::mapgen::generate_with(
        content,
        ("showcase", 5),
        id,
        None,
        loot::Luck { magic: 50, gems: 0 },
    )
    .expect("level");
    assert_eq!(plain.terrain, lucky.terrain);
    assert_eq!(
        plain.objects.keys().collect::<Vec<_>>(),
        lucky.objects.keys().collect::<Vec<_>>()
    );
}

#[test]
fn treasure_hunter_asks_for_either_of_two() {
    let mut w = world();
    let content = w.content().clone();
    let book = content.talents();
    let def = book.get(book.id("treasure_hunter").expect("talent"));
    assert!(matches!(def.needs.iter().next(), Some(Need::Either(_))));
    let words: Vec<String> = def.needs.iter().map(|n| n.words(&content)).collect();
    assert_eq!(words, ["Intelligence 12 or Appraise 20"]);
    let may = |w: &World| {
        let who = Candidate::player(&content, w.state.player(), w.state.character.as_ref(), 1);
        talent::may_take(&content, &who, def, None).is_ok()
    };
    assert!(may(&w), "Intelligence 14");
    w.player_mut().stats = Stats::new(PerStat::splat(10), PerStat::splat(40));
    assert!(!may(&w));
    let appraise = content.skill_id("appraise").expect("appraise");
    w.player_mut().skills.add(appraise, 20);
    assert!(may(&w), "Appraise 20 will do");
}

#[test]
fn quick_learner_brings_a_roll_more_every_level() {
    let mut w = world();
    let rolls = w.level_rolls(w.player_level());
    give(&mut w, "quick_learner");
    assert_eq!(w.level_rolls(w.player_level()), rolls + 1);
}

#[test]
fn skill_focus_counts_the_chosen_trained_skill_higher_past_100_too() {
    use crate::skills::Effect;
    let content = BUILTIN.clone();
    let skill = content.skill_id("athletics").expect("athletics");
    // A warrior with Athletics at `value`, whose kick (a curve of 8 at 100, 12 at 200, rounded
    // down) reads the bonus; a talent due.
    let at = |value: i32| {
        let mut w = world();
        run(&mut w, "xp:4000");
        let due = &w.state.progress.due;
        assert!(due.iter().any(|d| d.talents > 0), "{due:?}");
        let had = i32::from(w.state.player().skills.value(skill).expect("trained"));
        w.player_mut().skills.add(skill, value - had);
        w
    };
    let kick = |w: &World| w.state.player().skill_effect(&content, Effect::Kick);

    let mut w = at(70);
    assert_eq!(kick(&w), 5);
    // A skill must be chosen, and one the player is trained in.
    run(&mut w, "talent:skill_focus");
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("Skill focus needs a choice: a skill you are trained in.")
    );
    let stealth = content.skill_id("stealth").expect("stealth");
    assert_eq!(w.state.player().skills.value(stealth), None);
    run(&mut w, "talent:skill_focus:stealth");
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("`stealth` is not a skill you are trained in.")
    );
    // The list holds the trained skills alone.
    let id = content.talents().id("skill_focus").expect("skill_focus");
    let def = content.talents().get(id);
    let ways = talent::openings(&content, &w.candidate(), def);
    assert!(ways.contains(&Some("athletics".to_string())), "{ways:?}");
    assert!(!ways.contains(&Some("stealth".to_string())), "{ways:?}");
    // The pick: 25 higher, the trained value as it was.
    run(&mut w, "talent:skill_focus:athletics");
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("You learn Skill focus (athletics).")
    );
    assert_eq!(kick(&w), 7, "read at 95");
    assert_eq!(w.state.player().skills.value(skill), Some(70));
    // One pick per skill.
    assert!(matches!(
        talent::may_take(&content, &w.candidate(), def, Some("athletics")),
        Err(talent::Refusal::Held)
    ));
    // It sums with Master of none (5 for every trained skill).
    give(&mut w, "master_of_none");
    assert_eq!(kick(&w), 8, "read at 100");

    // Past 100, the soft cap on growth and no ceiling.
    let mut w = at(100);
    assert_eq!(kick(&w), 8);
    run(&mut w, "talent:skill_focus:athletics");
    assert_eq!(kick(&w), 9, "read at 125");
}

#[test]
fn the_burden_line_eases_each_level_a_step_further() {
    use crate::stats::Stat;
    let mut w = world();
    let me = w.state.player;
    // Dexterity 30, so every burden's cap on it in Defence binds; nothing worn caps it, and no
    // Balance eases the slowdown.
    let mut scores = PerStat::splat(14);
    scores[Stat::Dexterity] = 30;
    w.player_mut().stats = Stats::new(scores, PerStat::splat(40));
    w.player_mut().equipment = Default::default();
    w.player_mut().inventory = Vec::new();
    w.player_mut().skills = Default::default();
    let content = w.content().clone();
    let dex = (w.state.player().stats).modifier(Stat::Dexterity);
    let free_dv = w.perception().own_defence();
    let wait = w.price_of(me, crate::content::ActionKind::Wait);
    let base = w.load_of(me).limit;
    let dagger = content.object_id("dagger").expect("dagger");
    let weight = Stack::one(dagger).weight(&content);
    // Carries `num`/`den` of the limit now: the burden, the penalty, a wait's price, Defence.
    let at = |w: &mut World, num: u64, den: u64| {
        let count = u32::try_from(w.load_of(me).limit * num / den / weight).expect("count");
        w.player_mut().inventory = vec![Stack::new(dagger, count)];
        let load = w.load_of(me);
        let wait = w.price_of(me, crate::content::ActionKind::Wait);
        (
            load.burden(),
            load.penalty(),
            wait,
            w.perception().own_defence(),
        )
    };
    let dv = |cap: i32| free_dv - dex + dex.min(cap);
    let (burdened, strained) = (Burden::Burdened, Burden::Strained);
    // Without the line: Burdened ×1.25 and Dexterity +3, Strained ×1.5 and +1.
    assert_eq!(
        at(&mut w, 1, 2),
        (burdened, burdened, burdened.slow(wait), dv(3))
    );
    assert_eq!(
        at(&mut w, 5, 6),
        (strained, strained, strained.slow(wait), dv(1))
    );
    // Pack mule: a tenth more, 60 % of the slowdown off, the caps 1 higher.
    give(&mut w, "pack_mule");
    assert_eq!(w.load_of(me).limit, base * 110 / 100 / 3 * 3);
    assert_eq!(
        at(&mut w, 1, 2),
        (burdened, burdened, burdened.slow_eased(wait, 60), dv(4))
    );
    assert_eq!(
        at(&mut w, 5, 6),
        (strained, strained, strained.slow_eased(wait, 60), dv(2))
    );
    // Porter: a tenth more again, Burdened costs nothing, the caps 1 higher again.
    give(&mut w, "porter");
    assert_eq!(w.load_of(me).limit, base * 120 / 100 / 3 * 3);
    assert_eq!(
        at(&mut w, 1, 2),
        (burdened, Burden::Unburdened, wait, free_dv)
    );
    assert_eq!(
        at(&mut w, 5, 6),
        (strained, strained, strained.slow_eased(wait, 60), dv(3))
    );
    // Beast of burden: a tenth more again, and Strained costs what Burdened costs others.
    give(&mut w, "beast_of_burden");
    assert_eq!(w.load_of(me).limit, base * 130 / 100 / 3 * 3);
    assert_eq!(
        at(&mut w, 5, 6),
        (strained, burdened, burdened.slow_eased(wait, 60), dv(5))
    );
    // Overloaded is the end of it, whatever the line.
    assert_eq!(at(&mut w, 11, 10).1, Burden::Overloaded);
}

#[test]
fn each_step_of_the_burden_line_needs_the_one_before() {
    let book = BUILTIN.talents();
    for (id, before) in [("porter", "pack_mule"), ("beast_of_burden", "porter")] {
        let def = book.get(book.id(id).expect(id));
        let before = book.id(before).expect("the step before");
        assert_eq!(def.needs.talents, [before], "{id}");
    }
}

#[test]
fn iron_will_trap_sense_and_keen_searcher_add_to_the_checks() {
    let mut w = world();
    let (active, passive) = (
        w.search_modifier(Look::Active, Hidden::Door(0)),
        w.search_modifier(Look::Passive, Hidden::Door(0)),
    );
    give(&mut w, "iron_will");
    assert_eq!(
        w.search_modifier(Look::Passive, Hidden::Door(0)),
        passive + 2
    );
    assert_eq!(w.search_modifier(Look::Active, Hidden::Door(0)), active);
    give(&mut w, "trap_sense");
    give(&mut w, "keen_searcher");
    assert_eq!(
        w.search_modifier(Look::Passive, Hidden::Door(0)),
        passive + 5
    );
    assert_eq!(w.search_modifier(Look::Active, Hidden::Door(0)), active + 7);
    assert_eq!(
        w.state
            .player()
            .talent_mods(w.content())
            .delving
            .search_reach,
        1
    );
}

#[test]
fn keen_searcher_reaches_a_cell_farther() {
    let map = "#####\n#@.S#\n#####\n";
    let search = |keen: bool| {
        let mut files = ContentFiles::builtin();
        files.remove("data/hunger.toml");
        files.insert(
            "levels/reach.toml",
            &format!(
                "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n[legend]\n\
                 \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
                 \"S\" = {{ terrain = \"door_closed\", secret = 1 }}\n"
            ),
        );
        files.insert(
            "scenarios/reach.toml",
            "name = \"Hall\"\nstart_level = \"reach\"\nseed = 7\ncharacter = { race = \
             \"human\", gender = \"male\", class = \"warrior\" }\n",
        );
        let content = Arc::new(Content::load(&files).expect("valid"));
        let mut w = World::new(content, "reach", None).expect("world");
        if keen {
            give(&mut w, "keen_searcher");
        }
        run(&mut w, "search:5");
        said(&w).contains(&"You find a secret door!".to_string())
    };
    assert!(!search(false), "two cells off, out of reach");
    assert!(search(true));
}

#[test]
fn trap_sense_halves_what_traps_do() {
    let hurt = |sense: bool| {
        let mut w = world();
        if sense {
            give(&mut w, "trap_sense");
        }
        let me = w.state.player;
        let trap = w.content().trap_id("arrow").expect("a trap");
        let kind = w.content().damage_type_id("piercing");
        let hp = w.state.actors[&me].hp;
        let at = w.state.actors[&me].pos;
        w.trap_hurts(trap, me, (at, false), Damage::new(9, kind), &mut Vec::new());
        hp - w.state.actors[&me].hp
    };
    assert_eq!(hurt(false), 9);
    assert_eq!(hurt(true), 4);
}

#[test]
fn lockbreaker_kicks_harder_and_quieter_and_tunneler_digs_faster() {
    let mut w = world();
    let me = w.state.player;
    let at = w.state.actors[&me].pos;
    let loud = |w: &World| {
        let level = w.state.current();
        let actor = w.state.player();
        noise::noise(w.content(), level, actor, Burden::Unburdened, Act::Kick(at))
            .map_or(0, |n| n.loudness)
    };
    let before = loud(&w);
    give(&mut w, "lockbreaker");
    give(&mut w, "tunneler");
    assert_eq!(loud(&w), before / 2);
    let mods = w.state.player().talent_mods(w.content());
    assert_eq!((mods.delving.door_kick, mods.delving.dig_percent), (6, 50));
}

#[test]
fn gourmand_stops_short_of_bloated() {
    let eat = |gourmand: bool| {
        let mut w = world();
        if gourmand {
            give(&mut w, "gourmand");
        }
        let ration = w.content().object_id("ration").expect("ration");
        w.player_mut().inventory.push(Stack::one(ration));
        let at = w.state.player().inventory.len() - 1;
        w.player_mut()
            .satiation
            .as_mut()
            .expect("hungers")
            .nutrition = 5000;
        w.apply(&Action::Eat(Meal::Pack(at)));
        let content = w.content().clone();
        (w.state.player().hunger(&content), said(&w))
    };
    let (hunger, _) = eat(false);
    assert_eq!(hunger, Some(Hunger::Bloated));
    let (hunger, said) = eat(true);
    assert_eq!(hunger, Some(Hunger::Satiated));
    assert!(said.contains(&"You eat your fill; the rest does you no good.".to_string()));
}

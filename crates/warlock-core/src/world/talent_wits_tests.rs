//! The Scholar, Grit, and Delver talents (spec §8 "Talents"; #240): each changes the one rule it
//! names; Apprentice trains its chosen skill as the rank is taken; Treasure hunter's
//! prerequisite is either of two.

use super::search::Look;
use super::*;
use crate::action::{Meal, parse_actions};
use crate::burden::Burden;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, Need};
use crate::damage::Damage;
use crate::hunger::Hunger;
use crate::magic::{LootGrade, Rarity, loot};
use crate::noise::{self, Act};
use crate::object::Stack;
use crate::rng::Rng;
use crate::stats::{PerStat, Stats};
use crate::talent::{self, Candidate};
use std::sync::LazyLock;

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("content")));

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

/// One unidentified magic item, rolled from `seed`.
fn magic_item(world: &World, seed: u64) -> Stack {
    let mut rng = Rng::gameplay(seed);
    loop {
        let stack = loot::roll(world.content(), &mut rng, 5, LootGrade::Magic).expect("loot");
        if stack.rarity() != Rarity::Normal && !stack.is_identified() {
            return stack;
        }
    }
}

#[test]
fn lore_identifies_what_has_been_carried_for_a_day() {
    let mut w = world();
    let item = magic_item(&w, 3);
    w.player_mut().inventory.push(item);
    let at = w.state.player().inventory.len() - 1;
    // Without the talent a day of carrying teaches nothing.
    run(&mut w, "rest:24h");
    assert!(!w.state.player().inventory[at].is_identified());
    give(&mut w, "lore");
    run(&mut w, "rest:1h");
    w.appraise_carried(22 * crate::time::HOUR);
    assert!(!w.state.player().inventory[at].is_identified(), "23 hours");
    w.appraise_carried(crate::time::HOUR);
    assert!(w.state.player().inventory[at].is_identified());
    assert!(
        said(&w).iter().any(|m| m.starts_with("This is ")),
        "{:?}",
        said(&w)
    );
}

#[test]
fn keen_appraiser_tells_what_things_are_worth() {
    let mut w = world();
    assert_eq!(w.perception().appraisal(), Default::default());
    give(&mut w, "keen_appraiser");
    let appraisal = w.perception().appraisal();
    assert!(appraisal.worth && appraisal.gems);
}

#[test]
fn scroll_savant_reads_in_half_the_time_and_careful_reader_is_spared_its_burst() {
    let pack = "[\"scroll_of_fireball*3\"]";
    let map = "#######\n#@D...#\n#######\n";
    let mut w = hall(map, pack);
    let me = w.state.player;
    let hp = w.state.actors[&me].hp;
    let plain = run(&mut w, "read:a:2,1").remove(0);
    assert!(w.state.actors[&me].hp < hp, "the burst caught the reader");
    let mut w = hall(map, pack);
    give(&mut w, "scroll_savant");
    give(&mut w, "careful_reader");
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
    let read = |recall: bool| {
        let mut w = hall(
            "##########\n#@.......#\n##########\n",
            "[\"scroll_of_fireball*20\"]",
        );
        if recall {
            give(&mut w, "spell_recall");
        }
        run(&mut w, "read:a:7,1*12");
        let left = w.state.player().inventory.first().map_or(0, |s| s.count);
        let kept = (said(&w).iter())
            .filter(|m| *m == "The writing does not fade from the scroll.")
            .count();
        (left, kept)
    };
    assert_eq!(read(false), (8, 0));
    let (left, kept) = read(true);
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
    let rolls = w.level_rolls();
    give(&mut w, "quick_learner");
    assert_eq!(w.level_rolls(), rolls + 1);
}

#[test]
fn apprentice_trains_the_chosen_skill_at_the_pick() {
    let mut w = world();
    run(&mut w, "xp:4000");
    let due = &w.state.progress.due;
    assert!(due.iter().any(|d| d.talents > 0), "{due:?}");
    // A skill must be chosen.
    run(&mut w, "talent:apprentice");
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("Apprentice needs a choice: a skill.")
    );
    let skill = w.content().skill_id("appraise").expect("appraise");
    let before = w.state.player().skills.value(skill).unwrap_or(0);
    run(&mut w, "talent:apprentice:appraise");
    assert_eq!(
        said(&w).last().map(String::as_str),
        Some("You learn Apprentice (appraise) (1).")
    );
    let after = w.state.player().skills.value(skill).expect("trained");
    assert!((5..=10).contains(&(after - before)), "{before} → {after}");
    assert_eq!(
        w.state
            .player()
            .talents
            .rank_of("apprentice", Some("appraise")),
        1
    );
    assert!(
        w.state.levels_rng.is_some(),
        "rolled from the levels stream"
    );
}

#[test]
fn porter_shrugs_off_burdened() {
    let mut w = world();
    let me = w.state.player;
    let limit = w.load_of(me).limit;
    let stone = w.content().object_id("dagger").expect("dagger");
    let weight = Stack::one(stone).weight(w.content());
    let count = u32::try_from(limit / 2 / weight).expect("count");
    w.player_mut().inventory = vec![Stack::new(stone, count)];
    assert_eq!(w.load_of(me).penalty(), Burden::Burdened);
    give(&mut w, "porter");
    assert_eq!(w.load_of(me).burden(), Burden::Burdened, "still burdened");
    assert_eq!(w.load_of(me).penalty(), Burden::Unburdened);
}

#[test]
fn iron_will_trap_sense_and_keen_searcher_add_to_the_checks() {
    let mut w = world();
    let (active, passive) = (
        w.search_modifier(Look::Active),
        w.search_modifier(Look::Passive),
    );
    give(&mut w, "iron_will");
    assert_eq!(w.search_modifier(Look::Passive), passive + 2);
    assert_eq!(w.search_modifier(Look::Active), active);
    give(&mut w, "trap_sense");
    give(&mut w, "keen_searcher");
    assert_eq!(w.search_modifier(Look::Passive), passive + 5);
    assert_eq!(w.search_modifier(Look::Active), active + 7);
    assert_eq!(w.state.player().talent_mods(w.content()).search_reach, 1);
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
    assert_eq!((mods.door_kick, mods.dig_percent), (6, 50));
}

#[test]
fn silver_tongue_and_haggler_move_prices_in_the_players_favour() {
    let mut w = world();
    let shops = w.content().shops().expect("shops").clone();
    let favour = |w: &World| super::shop::favour(w.content(), &shops, w.state.player());
    let plain = favour(&w);
    give(&mut w, "silver_tongue");
    assert_eq!(favour(&w), plain + 10);
    give(&mut w, "haggler");
    assert_eq!(favour(&w), plain + 25);
    // Neither lifts the cap.
    w.player_mut().stats = Stats::new(PerStat::splat(40), PerStat::splat(40));
    assert_eq!(favour(&w), crate::content::MAX_FAVOUR);
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

#[test]
fn sure_footed_halves_the_slowdown_on_steps_only() {
    let mut w = world();
    let me = w.state.player;
    w.player_mut()
        .satiation
        .as_mut()
        .expect("hungers")
        .nutrition = 6000;
    let content = w.content().clone();
    let floor = content.terrain(content.terrain_id("floor").expect("floor"));
    let plain = w.step_price(me, floor, false);
    let wait = w.price_of(me, crate::content::ActionKind::Wait);
    give(&mut w, "sure_footed");
    let step = w.step_price(me, floor, false);
    assert!(step < plain, "{step} < {plain}");
    // ×1.25 Bloated becomes ×1.125.
    assert_eq!(step, plain * 1125 / 1250);
    assert_eq!(w.price_of(me, crate::content::ActionKind::Wait), wait);
}

#[test]
fn the_trees_are_held_to_the_bar() {
    let book = BUILTIN.talents();
    for tree in ["scholar", "grit", "delver"] {
        let tree = book.tree(book.tree_id(tree).expect("tree"));
        assert!(!tree.pending, "{}", tree.id);
    }
    for system in [
        "searching",
        "digging",
        "shops",
        "magic_items",
        "scrolls",
        "skills",
        "experience",
    ] {
        let id = book.system_id(system).expect("system");
        assert!(!book.system(id).pending, "{system}");
    }
}

//! Rods (spec §8 "Magic items": "Rods"; #399): a rod in the missile weapon slot does its effect
//! for the user's mana (Channeler and Frugal caster cut it), reaching by the user's Willpower
//! (a ladder, as a spell's; no caster level read: the spell scaling design §7.5); sleep and
//! terror take creatures up to the rod's own level; no mana, no use; a rod type is unknown until used or identified; the effects dig, teleport,
//! and put to sleep or to flight; rod labels leave the scrolls' and wands' as they were.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::stats::{PerStat, Stat, Stats};
use crate::talent::TalentMods;

/// The pack: a, a rod of digging; b, a rod of sleep; c, a rod of teleport other; d, a rod of
/// terror; e, a scroll of identify; f, a crossbow.
const PACK: &str = "carry = [\"rod_of_digging\", \"rod_of_sleep\", \"rod_of_teleport_other\", \
     \"rod_of_terror\", \"scroll_of_identify\", \"crossbow\"]";

/// A lit hall: the player (`@`) with two stone walls east of them, granite (`G`) beyond, and
/// two more stone walls beyond that; a rat (`r`) and an ogre (`o`) in the south.
const HALL: &str = "###############\n#.............#\n#..@##G##.....#\n#.............#\n\
                    #.......r.....#\n#.......o.....#\n###############\n";

fn world() -> World {
    world_of("rat", "ogre", &[])
}

/// The hall with creatures `south` at `r` and `farther` at `o`, and more content `extra`.
fn world_of(south: &str, farther: &str, extra: &[(&str, String)]) -> World {
    let mut files = ContentFiles::builtin();
    for (path, text) in extra {
        files.insert(path, text);
    }
    files.insert(
        "levels/hall.toml",
        &format!(
            "name = \"Hall\"\nambient = [255, 255, 255, 0]\nmap = '''\n{HALL}'''\n[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"G\" = \"granite\"\n\
             \"r\" = {{ terrain = \"floor\", creature = \"{south}\" }}\n\
             \"o\" = {{ terrain = \"floor\", creature = \"{farther}\" }}\n"
        ),
    );
    files.insert(
        "scenarios/hall.toml",
        &format!("name = \"Hall\"\nstart_level = \"hall\"\nhp = 500\nmana = 40\n{PACK}\n"),
    );
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "hall", Some(3)).expect("world")
}

fn run(w: &mut World, actions: &str) -> Vec<Outcome> {
    (parse_actions(actions).expect("actions").iter())
        .map(|a| w.apply(a))
        .collect()
}

fn said(w: &World) -> String {
    let all: Vec<String> = w.messages().iter().map(|m| m.display()).collect();
    all.join(" ")
}

fn terrain_at(w: &World, x: i32, y: i32) -> String {
    let t = w.state.current().terrain.get(Pos::new(x, y)).copied();
    w.content.terrain(t.expect("in the level")).id.clone()
}

/// The creature of type `id`; its id.
fn creature(w: &World, id: &str) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.id != w.state.player && w.content.creature(a.creature).id == id)
        .map(|a| a.id)
        .expect(id)
}

/// Puts actor `id` at (x, y) with plenty of hit points.
fn place(w: &mut World, id: ActorId, x: i32, y: i32) {
    let actor = w.state.actors.get_mut(&id).expect("an actor");
    actor.pos = Pos::new(x, y);
    actor.hp = 1000;
    actor.hp_base = 1000;
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
}

#[test]
fn a_rod_of_digging_breaks_a_line_for_mana_and_names_itself() {
    let mut w = world();
    let first = w.lore().describe(&w.state.player().inventory[0]);
    assert!(first.ends_with(" rod"), "{first}");
    assert_eq!(w.caster_level(w.state.player), 1);
    run(&mut w, "equip:missile_weapon:a");
    let mana = w.state.player().mana;
    let out = run(&mut w, "fire:12,2").remove(0);
    assert!(out.took_turn);
    assert_eq!(w.state.player().mana, mana - 6, "the rod's cost, no charge");
    // Reach 3 at average Willpower: the two stone walls give way, the granite stops the line, and
    // the wall beyond it stands.
    assert_eq!(terrain_at(&w, 4, 2), "rubble");
    assert_eq!(terrain_at(&w, 5, 2), "rubble");
    assert_eq!(terrain_at(&w, 6, 2), "granite");
    assert_eq!(terrain_at(&w, 7, 2), "wall");
    let text = said(&w);
    let name = first.split_once(' ').expect("an article").1;
    assert!(text.contains(&format!("You zap the {name}.")), "{text}");
    assert!(text.contains("It is a rod of digging."), "{text}");
    assert!(
        text.contains("The stone crumbles away before you."),
        "{text}"
    );
    let stack = &w.state.player().inventory[0];
    assert_eq!(w.lore().describe(stack), "a rod of digging");
    assert!(w.lore().knows(stack));
    assert!(
        out.events
            .iter()
            .any(|e| matches!(e, Event::WallBroken { .. }))
    );
}

#[test]
fn without_the_mana_a_rod_does_nothing_and_takes_no_time() {
    let mut w = world();
    run(&mut w, "equip:missile_weapon:a");
    w.player_mut().mana = 5;
    let out = run(&mut w, "fire:12,2").remove(0);
    assert!(!out.took_turn);
    assert_eq!(w.state.player().mana, 5);
    assert_eq!(terrain_at(&w, 4, 2), "wall");
    assert!(said(&w).ends_with("You have not the mana to use it."));
    // Nor is the type learned by a use that never happened.
    assert!(!w.lore().is_identified(w.state.player().inventory[0].object));
}

/// Gives the player `willpower`, every other score 14.
fn set_willpower(w: &mut World, willpower: u8) {
    let mut scores = PerStat::splat(14);
    scores[Stat::Willpower] = willpower;
    w.player_mut().stats = Stats::new(scores, PerStat::splat(40));
}

#[test]
fn reach_climbs_the_willpower_ladder_and_reads_no_caster_level() {
    let mut w = world();
    run(&mut w, "equip:missile_weapon:a");
    assert_eq!(
        w.perception().own_rod(0),
        None,
        "unknown: neither cost nor reach"
    );
    run(&mut w, "identify:a");
    // Digging: `range = 3`, `willpower 14 4, 18 5, 22 6, 26 7`.
    let mut reach_at = |willpower: u8| {
        set_willpower(&mut w, willpower);
        w.perception().own_rod(0).map(|(_, reach)| reach)
    };
    let reaches = [10, 13, 14, 17, 18, 22, 25, 26, 40].map(&mut reach_at);
    assert_eq!(
        reaches,
        [3, 3, 4, 4, 5, 6, 6, 7, 7].map(Some),
        "the base below the first rung, the top rung's 7 past it"
    );
    // Two users of the same Willpower and different caster levels reach the same.
    set_willpower(&mut w, 18);
    let level_one = w.caster_level(w.state.player);
    let before = w.perception().own_rod(0);
    run(&mut w, "xp:100000");
    assert!(w.caster_level(w.state.player) > level_one + 5);
    set_willpower(&mut w, 18);
    assert_eq!(w.perception().own_rod(0), before);
    assert_eq!(before, Some((6, 5)));
    let digging = w
        .content
        .rod_effect(w.content.rod_effect_index("digging").expect("digging"));
    assert_eq!(w.state.player().rod_reach(&w.content, digging), 5);
}

/// A plain creature of `id` at `level`, as a content file.
fn leveled(id: &str, level: u8) -> (&'static str, String) {
    let path = if level == 20 {
        "data/creatures/zz_twenty.toml"
    } else {
        "data/creatures/zz_twenty_one.toml"
    };
    let text = format!(
        "[[creature]]\nid = \"{id}\"\nname = \"{id}\"\nglyph = \"b\"\nfg = \"#ffffff\"\n\
         level = {level}\nweight = 30000\nfaction = \"vermin\"\n"
    );
    (path, text)
}

#[test]
fn rods_of_sleep_and_terror_take_creatures_up_to_their_own_level() {
    let extra = [leveled("brute_twenty", 20), leveled("brute_twenty_one", 21)];
    let mut w = world_of("brute_twenty", "brute_twenty_one", &extra);
    let twenty = creature(&w, "brute_twenty");
    let twenty_one = creature(&w, "brute_twenty_one");
    place(&mut w, twenty, 6, 4);
    place(&mut w, twenty_one, 6, 5);
    assert_eq!(w.caster_level(w.state.player), 1, "no caster level read");
    run(&mut w, "equip:missile_weapon:b");
    let out = run(&mut w, "fire:6,4").remove(0);
    assert!(out.took_turn);
    assert!(w.asleep(twenty), "level 20: {}", said(&w));
    assert!(!w.asleep(twenty_one), "level 21");
    let text = said(&w);
    assert!(text.contains("The brute_twenty is asleep."), "{text}");
    assert!(
        text.contains("The brute_twenty_one shrugs it off."),
        "{text}"
    );
    // Terror the same: the level-20 one is frightened, the level-21 one is not.
    run(&mut w, "remove:missile_weapon equip:missile_weapon:d");
    run(&mut w, "fire:6,4");
    let frightened = w.content.status_id("frightened").expect("a status");
    let afraid = |w: &World, id: ActorId| {
        (w.state.actors[&id].effects.iter()).any(|e| e.status == frightened)
    };
    assert!(afraid(&w, twenty), "{}", said(&w));
    assert!(!afraid(&w, twenty_one), "{}", said(&w));
}

#[test]
fn a_sleeper_takes_no_turns_until_it_is_woken() {
    let mut w = world();
    let (rat, ogre) = (creature(&w, "rat"), creature(&w, "ogre"));
    w.state.actors.remove(&ogre);
    place(&mut w, rat, 6, 4);
    run(&mut w, "equip:missile_weapon:b");
    let out = run(&mut w, "fire:6,4").remove(0);
    assert!(out.took_turn);
    assert!(
        said(&w).contains("The giant rat is asleep."),
        "{}",
        said(&w)
    );
    assert!(w.asleep(rat));
    let at = w.state.actors[&rat].pos;
    run(&mut w, "wait*3");
    assert_eq!(w.state.actors[&rat].pos, at);
    assert!(w.asleep(rat));
}

#[test]
fn a_burst_out_of_reach_is_refused_and_costs_nothing() {
    let mut w = world();
    run(&mut w, "equip:missile_weapon:d");
    let mana = w.state.player().mana;
    let out = run(&mut w, "fire:13,5").remove(0);
    assert!(!out.took_turn);
    assert_eq!(w.state.player().mana, mana);
    assert!(said(&w).ends_with("That is out of range."));
}

#[test]
fn a_rod_of_terror_frightens_and_a_rod_of_teleport_other_sends_away() {
    let mut w = world();
    let rat = creature(&w, "rat");
    let ogre = creature(&w, "ogre");
    w.state.actors.remove(&ogre);
    place(&mut w, rat, 6, 4);
    run(&mut w, "equip:missile_weapon:d");
    run(&mut w, "fire:6,4");
    let frightened = w.content.status_id("frightened").expect("a status");
    assert!(
        w.state.actors[&rat]
            .effects
            .iter()
            .any(|e| e.status == frightened)
    );
    run(&mut w, "remove:missile_weapon equip:missile_weapon:c");
    place(&mut w, rat, 5, 3);
    let out = run(&mut w, "fire:5,3").remove(0);
    assert!(out.took_turn);
    assert_ne!(w.state.actors[&rat].pos, Pos::new(5, 3), "sent elsewhere");
    assert!(said(&w).contains("The giant rat vanishes!"), "{}", said(&w));
    // A bolt that meets no one still costs its mana and time.
    let mana = w.state.player().mana;
    let out = run(&mut w, "fire:3,1").remove(0);
    assert!(out.took_turn);
    assert_eq!(w.state.player().mana, mana - 12);
    assert!(said(&w).ends_with("The bolt strikes nothing."));
}

#[test]
fn channeler_cuts_a_rods_cost_before_frugal_caster() {
    let channeler = TalentMods {
        rod_cost_percent: -25,
        ..TalentMods::default()
    };
    assert_eq!(channeler.rod_cost(12), 9);
    assert_eq!(channeler.rod_cost(1), 1, "never below 1");
    let mut w = world();
    let content = w.content.clone();
    let rod = (content.object(w.state.player().inventory[2].object).rod).expect("a rod");
    assert_eq!(w.state.player().rod_cost(&content, &rod), 12);
    w.player_mut().talents.take("channeler", None);
    assert_eq!(w.state.player().rod_cost(&content, &rod), 9);
    w.player_mut().talents.take("frugal_caster", None);
    assert_eq!(w.state.player().rod_base(&content, &rod), 9);
    assert_eq!(
        w.state.player().rod_cost(&content, &rod),
        7,
        "9 less a fifth"
    );
    // The use spends that, through spend_mana.
    let rat = creature(&w, "rat");
    place(&mut w, rat, 5, 3);
    run(&mut w, "equip:missile_weapon:c");
    let mana = w.state.player().mana;
    run(&mut w, "fire:5,3");
    assert_eq!(w.state.player().mana, mana - 7);
}

#[test]
fn identifying_a_rod_tells_its_effect_and_cost() {
    let mut w = world();
    let unknown: Vec<String> = (crate::magic::describe::appraised(
        &w.state.player().inventory[1],
        w.lore(),
        Default::default(),
    ))
    .into_iter()
    .map(|l| l.text)
    .collect();
    let text = unknown.join(" ");
    assert!(!text.contains("Sleep") && !text.contains("mana"), "{text}");
    assert!(text.contains("what it does is unknown"), "{text}");
    let out = run(&mut w, "read:e:b").remove(0);
    assert!(out.took_turn);
    let stack = &w.state.player().inventory[1];
    assert_eq!(w.lore().describe(stack), "a rod of sleep");
    let lines: Vec<String> =
        (crate::magic::describe::appraised(stack, w.lore(), Default::default()))
            .into_iter()
            .map(|l| l.text)
            .collect();
    assert!(
        lines.iter().any(|l| l.starts_with("Holds Sleep: ")),
        "{lines:?}"
    );
    assert!(
        lines.iter().any(|l| l.starts_with("Costs 9 mana a use")),
        "{lines:?}"
    );
}

#[test]
fn a_rod_and_a_launcher_share_the_missile_weapon_slot_and_a_rod_saves_as_any_stack() {
    let mut w = world();
    run(&mut w, "equip:missile_weapon:a");
    assert_eq!(w.state.player().rod_item(&w.content), Some(0));
    assert!(w.perception().ready_rod().is_some());
    assert!(w.perception().ready_wand().is_none());
    let json = serde_json::to_string(&w.state.player().inventory[0]).expect("json");
    assert!(
        !json.contains("charges"),
        "nothing on a rod's stack: {json}"
    );
    run(&mut w, "fire:12,2");
    let loaded = crate::save::decode(&crate::save::encode(&w), w.content.clone()).expect("decodes");
    assert_eq!(loaded.state_hash(), w.state_hash());
    assert!(
        loaded
            .lore()
            .is_identified(loaded.state.player().inventory[0].object)
    );
    run(&mut w, "remove:missile_weapon equip:missile_weapon:f");
    assert_eq!(w.state.player().rod_item(&w.content), None);
    assert!(w.state.player().launcher(&w.content).is_some());
}

#[test]
fn rod_labels_leave_the_scrolls_and_wands_labels_as_they_were() {
    let with = Content::builtin().expect("content");
    let mut files = ContentFiles::builtin();
    files.remove("data/rod_labels.toml");
    let without = Content::load(&files).expect("content");
    let a = crate::scroll_label::Labels::draw(&with, 7);
    let b = crate::scroll_label::Labels::draw(&without, 7);
    let mut rods = Vec::new();
    for (i, o) in with.object_defs().iter().enumerate() {
        let id = crate::content::ObjectId(i as u16);
        if o.read.is_some() || o.wand.is_some() {
            assert_eq!(a.get(id), b.get(id), "{}", o.id);
        }
        if o.rod.is_some() {
            assert!(b.get(id).is_none());
            rods.push(a.get(id).expect("a label").to_string());
        }
    }
    let mut unique = rods.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), rods.len(), "{rods:?}");
    assert_eq!(rods.len(), 4);
}

#[test]
fn rod_data_is_checked() {
    let load = |rod: &str| {
        let mut files = ContentFiles::builtin();
        files.insert(
            "data/objects/test_rod.toml",
            &format!(
                "[[object]]\nid = \"test_rod\"\nname = \"test rod\"\nglyph = \"-\"\n\
                 fg = \"#ffffff\"\nclass = \"rods\"\nmaterials = [\"iron\"]\nweight = 10\n\
                 value = 1\nslot = \"missile_weapon\"\n[object.rod]\n{rod}\n"
            ),
        );
        Content::load(&files).map(|_| ()).map_err(|e| e.to_string())
    };
    assert_eq!(load("effect = \"digging\"\nmana = 3"), Ok(()));
    let refused = [
        "effect = \"flying\"\nmana = 3",
        "effect = \"digging\"\nmana = 0",
        "effect = \"digging\"\nmana = 101",
        "effect = \"digging\"",
    ];
    for rod in refused {
        assert!(load(rod).is_err(), "{rod}");
    }
}

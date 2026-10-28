//! The utility affixes' hooks (spec §8 "Magic items"; docs/design/affixes-systems.md §6): a
//! worn `door_kick` joins the kick's Strength, `dig_percent` the work a turn, `hunger` the pace
//! of hunger, `trap_damage` the cut of a trap's blow (with Trap sense at most 75 % off), and
//! `disease_check` every disease check. Each applies unidentified, stops when taken off, and
//! works on a creature's gear where the rules read a creature at all. The affixes here are
//! fixtures (in the shipped files' categories, with values of their own), beside the shipped
//! Hobnailed, Delving, of Sustenance, of Wariness, and of Health. The pick-axe is a base item
//! (`tool`): loot rolls it magic from item level 3, while the hand-placed ones stay normal.

use super::fixture_factions::peaceful_factions;
use super::*;
use crate::action::parse_actions;
use crate::character::CharacterSpec;
use crate::content::{ContentFiles, ItemKind};
use crate::damage::Damage;
use crate::magic::describe;
use crate::magic::{Affix, LootGrade, Magic, Numbers, Rarity, loot, may_roll};
use crate::rng::Rng;
use crate::stats::Stat;

// A lit cellar: the player beside packed earth (E) to dig, a kobold standing by.
const CELLAR: &str = r##"
name = "Cellar"
ambient = [255, 255, 255, 0]
map = '''
#######
#@E...#
#.....#
#k....#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"E" = "packed_earth"
"k" = { terrain = "floor", creature = "kobold" }
"##;

/// One fixture affix file: `id` with `effect` on `bases`, a single tier of `value`, in the
/// shipped affix's `category` (every affix of one effect names one).
fn affix(kind: &str, id: &str, effect: &str, category: &str, bases: &str, value: i32) -> String {
    format!(
        "[[{kind}]]\nid = \"{id}\"\nname = \"{id}\"\neffect = \"{effect}\"\n\
         category = \"{category}\"\nbases = [{bases}]\n\
         tiers = [{{ level = 1, min = {value}, max = {value} }}]\n"
    )
}

fn files() -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("data/factions.toml", &peaceful_factions());
    let fixtures = [
        (
            "prefix",
            "zz_hobnailed",
            "door_kick",
            "kick",
            "\"boots\"",
            4,
        ),
        (
            "prefix",
            "zz_delving",
            "dig_percent",
            "digging",
            "\"tool\"",
            50,
        ),
        (
            "suffix",
            "zz_sustenance",
            "hunger",
            "hunger",
            "\"ring\"",
            30,
        ),
        (
            "suffix",
            "zz_wariness",
            "trap_damage",
            "traps",
            "\"boots\"",
            40,
        ),
        (
            "suffix",
            "zz_health",
            "disease_check",
            "disease",
            "\"ring\"",
            3,
        ),
    ];
    for (kind, id, effect, category, bases, value) in fixtures {
        let text = affix(kind, id, effect, category, bases, value);
        files.insert(&format!("data/affixes/{id}.toml"), &text);
    }
    files.insert("levels/affix_cellar.toml", CELLAR);
    files.insert(
        "scenarios/affix_cellar.toml",
        "name = \"Cellar\"\nstart_level = \"affix_cellar\"\nhp = 100\n\
         stats = { strength = 10 }\n",
    );
    files
}

static CONTENT: std::sync::LazyLock<Arc<Content>> =
    std::sync::LazyLock::new(|| Arc::new(Content::load(&files()).expect("test content is valid")));

/// A human warrior in the cellar.
fn world() -> World {
    let spec: CharacterSpec = "human,male,warrior".parse().expect("a character");
    World::with_character(CONTENT.clone(), "affix_cellar", Some(7), Some(&spec)).expect("world")
}

fn run(w: &mut World, actions: &str) -> Outcome {
    let mut last = Outcome::default();
    for action in parse_actions(actions).expect("actions parse") {
        last = w.apply(&action);
    }
    last
}

/// One `object` of magic with affix `id` (tier 1, `value`), identified or not.
fn magic(object: &str, id: &str, value: i32, identified: bool) -> Stack {
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 1,
        affixes: vec![Affix {
            id: id.into(),
            tier: 1,
            value,
        }],
        name: None,
        identified,
        carried: 0,
        unique: None,
    };
    Stack::with_magic(CONTENT.object_id(object).expect("object"), 1, magic)
}

/// Actor `who` puts `stack` on in `slot` (as the dev tests do: no time).
fn wear(w: &mut World, who: ActorId, slot: &str, stack: Stack) {
    let content = w.content().clone();
    let actor = w.state.actors.get_mut(&who).expect("actor");
    actor.inventory.push(stack);
    let item = actor.inventory.len() - 1;
    let slot = content.slot_id(slot).expect("slot");
    let taken = crate::equipment::slots_taken(&content, actor.inventory[item].object, slot);
    for &s in &taken {
        if let Some(other) = actor.equipment.get(s) {
            actor.equipment.take_off(other);
        }
    }
    actor.equipment.put(&taken, item);
}

fn kobold(w: &World) -> ActorId {
    // The one actor here besides the player.
    let id = (w.state.actors.keys().copied()).find(|&id| id != w.state.player);
    id.expect("the kobold")
}

#[test]
fn hobnailed_boots_kick_as_if_stronger_and_stop_when_taken_off() {
    let mut w = world();
    let me = w.state.player;
    let bare = w.kick_force();
    wear(
        &mut w,
        me,
        "feet",
        magic("leather_boots", "zz_hobnailed", 4, false),
    );
    assert_eq!(
        w.kick_force(),
        bare + 4,
        "unidentified, it works all the same"
    );
    w.player_mut().talents.take("lockbreaker", None);
    assert_eq!(w.kick_force(), bare + 4 + 6, "with Lockbreaker's term");
    run(&mut w, "remove:feet");
    assert_eq!(w.kick_force(), bare + 6, "taken off, it stops");
}

#[test]
fn a_delving_pick_axe_digs_faster() {
    let dig = |pick: Stack| {
        let mut w = world();
        let me = w.state.player;
        wear(&mut w, me, "tool", pick);
        run(&mut w, "dig:E").elapsed.as_seconds()
    };
    let plain = dig(Stack::one(CONTENT.object_id("pick_axe").expect("pick")));
    let delving = dig(magic("pick_axe", "zz_delving", 50, false));
    assert!(plain > 0);
    assert_eq!(
        delving,
        plain * 2 / 3,
        "half again the work a turn: two thirds the time"
    );
}

#[test]
fn a_ring_of_sustenance_slows_hunger_and_keeps_the_pace_above_zero() {
    let mut w = world();
    let me = w.state.player;
    let before = w.hunger_percent();
    wear(
        &mut w,
        me,
        "left_ring",
        magic("gold_ring", "zz_sustenance", 30, false),
    );
    assert_eq!(w.hunger_percent(), before * 70 / 100);
    let mut w = world();
    wear(
        &mut w,
        me,
        "left_ring",
        magic("gold_ring", "zz_sustenance", 100, true),
    );
    wear(
        &mut w,
        me,
        "right_ring",
        magic("gold_ring", "zz_sustenance", 100, true),
    );
    assert_eq!(w.hunger_percent(), 1, "the pace stays at least 1");
}

/// What a trap's 20 piercing damage takes from `who`.
fn trapped(w: &mut World, who: ActorId) -> i32 {
    let trap = w.content().trap_id("arrow").expect("a trap");
    let kind = w.content().damage_type_id("piercing");
    let hp = w.state.actors[&who].hp;
    let at = w.state.actors[&who].pos;
    w.trap_hurts(
        trap,
        who,
        (at, false),
        Damage::new(20, kind),
        &mut Vec::new(),
    );
    hp - w.state.actors[&who].hp
}

#[test]
fn boots_of_wariness_soften_traps_capped_with_trap_sense() {
    let mut w = world();
    let me = w.state.player;
    assert_eq!(trapped(&mut w, me), 20);
    let mut w = world();
    wear(
        &mut w,
        me,
        "feet",
        magic("leather_boots", "zz_wariness", 40, false),
    );
    assert_eq!(trapped(&mut w, me), 12, "40 % less");
    let mut w = world();
    w.player_mut().talents.take("trap_sense", None);
    assert_eq!(trapped(&mut w, me), 10, "Trap sense alone: half");
    wear(
        &mut w,
        me,
        "feet",
        magic("leather_boots", "zz_wariness", 40, false),
    );
    assert_eq!(
        trapped(&mut w, me),
        5,
        "90 % off capped at 75: a trap always hurts"
    );
}

#[test]
fn a_kobold_in_boots_of_wariness_is_hurt_less_by_traps() {
    let mut w = world();
    let k = kobold(&w);
    w.state.actors.get_mut(&k).expect("kobold").hp = 100;
    assert_eq!(trapped(&mut w, k), 20);
    let mut w = world();
    w.state.actors.get_mut(&k).expect("kobold").hp = 100;
    wear(
        &mut w,
        k,
        "feet",
        magic("leather_boots", "zz_wariness", 40, true),
    );
    assert_eq!(trapped(&mut w, k), 12);
}

#[test]
fn a_ring_of_health_adds_to_every_disease_check() {
    let mut w = world();
    let me = w.state.player;
    let check = |w: &World, stat| w.disease_modifier(w.state.player(), stat);
    let (toughness, strength) = (check(&w, Stat::Toughness), check(&w, Stat::Strength));
    wear(
        &mut w,
        me,
        "left_ring",
        magic("gold_ring", "zz_health", 3, false),
    );
    assert_eq!(check(&w, Stat::Toughness), toughness + 3);
    assert_eq!(check(&w, Stat::Strength), strength + 3);
    let k = kobold(&w);
    let before = w.disease_modifier(&w.state.actors[&k], Stat::Toughness);
    wear(
        &mut w,
        k,
        "left_ring",
        magic("gold_ring", "zz_health", 3, true),
    );
    let after = w.disease_modifier(&w.state.actors[&k], Stat::Toughness);
    assert_eq!(after, before + 3, "a creature's ring counts too");
}

#[test]
fn the_pick_axe_is_a_tool_rolled_magic_from_item_level_3() {
    let pick = CONTENT.object_id("pick_axe").expect("pick");
    assert_eq!(CONTENT.object(pick).kind, Some(ItemKind::Tool));
    // A pick-axe that digs faster: one with a `dig_percent` affix (Delving or the fixture).
    let digs = |s: &Stack| {
        (s.magic().expect("magic").rolled(&CONTENT).iter())
            .any(|r| r.def.effect == crate::content::Effect::DigPercent)
    };
    let rolled = |level: u8| {
        (0..4000u64).find_map(|seed| {
            let mut rng = Rng::stream(seed, "affix_utility");
            loot::roll(&CONTENT, &mut rng, level, LootGrade::Magic)
                .filter(|s| s.object == pick && digs(s))
        })
    };
    assert!(rolled(2).is_none(), "never below its level");
    let found = rolled(3).expect("a Delving pick-axe at level 3");
    let magic = found.magic().expect("magic");
    let tool = CONTENT.object(pick);
    assert!((magic.rolled(&CONTENT).iter()).all(|r| may_roll(r.def, ItemKind::Tool, tool)));
    // It shows no numbers; its description gives its power.
    assert!(Numbers::of(&found, &CONTENT).is_empty());
    let lines: Vec<String> = (describe::lines(&found, &CONTENT).into_iter())
        .map(|l| l.text)
        .collect();
    assert!(lines.contains(&"Digs at 10".to_string()), "{lines:?}");
    // Worn and dug with: faster than a plain one.
    let dig = |pick: Stack| {
        let mut w = world();
        let me = w.state.player;
        wear(&mut w, me, "tool", pick);
        run(&mut w, "dig:E").elapsed.as_seconds()
    };
    assert!(dig(found) < dig(Stack::one(pick)));
}

#[test]
fn hand_placed_and_kit_pick_axes_stay_normal() {
    let content = Arc::new(Content::builtin().expect("content"));
    let w = World::new(content.clone(), "quarry", None).expect("the quarry");
    let pick = content.object_id("pick_axe").expect("pick");
    let (level, _) = w.ground_truth();
    let placed: Vec<&Stack> = (level.objects.values().flatten())
        .filter(|s| s.object == pick)
        .collect();
    assert!(!placed.is_empty(), "the quarry's pick-axe");
    assert!(placed.iter().all(|s| s.rarity() == Rarity::Normal));
    let spec: CharacterSpec = "dwarf,male,warrior".parse().expect("a character");
    let w = World::start_run(content, "quarry", Some(7), Some(&spec), None, true).expect("world");
    let kit: Vec<&Stack> = (w.state.player().inventory.iter())
        .filter(|s| s.object == pick)
        .collect();
    assert!(!kit.is_empty(), "a dwarf's kit holds one");
    assert!(kit.iter().all(|s| s.rarity() == Rarity::Normal));
}

/// The shipped tool affixes (Delving, Featherweight, of the Bear) make the pick-axe a magic
/// find too, as plain loot finds it plain (#635).
#[test]
fn loot_finds_the_pick_axe_plain_and_magic() {
    let builtin = Content::builtin().expect("content");
    let pick = builtin.object_id("pick_axe").expect("pick");
    let roll = |seed: u64, grade| {
        let mut rng = Rng::stream(seed, "affix_utility");
        loot::roll(&builtin, &mut rng, 3, grade)
    };
    let found: Vec<Stack> = (0..3000).filter_map(|s| roll(s, LootGrade::Any)).collect();
    let picks: Vec<&Stack> = found.iter().filter(|s| s.object == pick).collect();
    assert!(!picks.is_empty(), "a pick-axe turns up as loot");
    assert!(picks.iter().any(|s| s.rarity() == Rarity::Normal));
    let magic: Vec<Stack> = (0..3000)
        .filter_map(|s| roll(s, LootGrade::Magic))
        .collect();
    assert!(magic.iter().all(|s| s.magic().is_some()));
    assert!(
        magic.iter().any(|s| s.object == pick),
        "a magic pick-axe turns up"
    );
}

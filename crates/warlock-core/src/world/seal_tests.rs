//! Seals and the rare materials (docs/design/enchanting.md §9; `world/seal.rs`): a seal shuts its
//! family out of the roll and costs a crystal of the colour and half an hour; its refusals; the
//! wizard's seal steps narrowed by the crystal and the item; the heart-crystal on a rare's
//! Quicken; the soul-fetter on Recast with the affix kept; the dialog's words; and a replay.

use super::super::enchant::{EnchantBand, enchant_out_for_test};
use super::super::*;
use crate::action::parse_actions;
use crate::content::{ContentFiles, Rarity};
use crate::magic::{Affix, Magic, Quality};
use crate::object::Stack;
use crate::replay::Replay;
use crate::skills::Trained;
use crate::stats::{PerStat, Stats};
use std::sync::LazyLock;

const ROOM: &str = "\
#########
#.......#
#.@.....#
#.......#
#########
";

/// The built-in content with [`ROOM`]: a long sword and four red crystals lie east of the
/// player. A wizard is always trained in Enchanting, for the replay.
static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.edit_table("classes", |text| {
        text.replace(
            "enchanting = { chance = 30,",
            "enchanting = { chance = 100,",
        )
    });
    files.insert(
        "levels/test_seal.toml",
        &format!(
            "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             [[item]]\nat = [3, 2]\nobject = \"long_sword\"\n\
             [[item]]\nat = [3, 2]\nobject = \"crystal_red\"\ncount = 4\n"
        ),
    );
    files.insert(
        "scenarios/test_seal.toml",
        "name = \"T\"\nstart_level = \"test_seal\"\n\
         character = { race = \"human\", gender = \"female\", class = \"commoner\" }\n",
    );
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

/// The room; the player carries nothing, Enchanting 100, every score 10, `seed` their fate.
fn world(seed: u64) -> World {
    let mut w = World::new(FIXTURE.clone(), "test_seal", Some(seed)).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    me.stats = Stats::new(PerStat::splat(10), PerStat::splat(10));
    let id = w.content.skill_id("enchanting").expect("enchanting");
    let mut all: Vec<_> = (w.player_mut().skills.iter())
        .filter(|(s, _)| *s != id)
        .collect();
    all.push((
        id,
        Trained {
            value: 100,
            marks: 0,
        },
    ));
    w.player_mut().skills = all.into_iter().collect();
    w
}

/// `count` of `object`, at the end of the pack.
fn item(w: &mut World, object: &str, count: u32) {
    let id = w.content.object_id(object).expect(object);
    w.player_mut().inventory.push(Stack::new(id, count));
}

/// `count` crystals of `colour`, at the end of the pack.
fn crystals(w: &mut World, colour: &str, count: u32) {
    item(w, &format!("crystal_{colour}"), count);
}

/// A Masterwork long sword (it holds four) with `affixes`, magic or `rare`, identified, at the
/// end of the pack.
fn sword(w: &mut World, affixes: &[&str], rare: bool) {
    let content = w.content.clone();
    let affixes = (affixes.iter())
        .map(|id| Affix {
            id: (*id).to_string(),
            tier: 1,
            value: content.affixes().get(id).expect(id).tiers[0].min,
        })
        .collect();
    let magic = Magic {
        rarity: if rare { Rarity::Rare } else { Rarity::Magic },
        level: 1,
        affixes,
        name: rare.then(|| "Grim Bite".to_string()),
        identified: true,
        carried: 0,
        unique: None,
    };
    let id = content.object_id("long_sword").expect("sword");
    let mut stack = Stack::with_magic(id, 1, magic);
    stack.set_worked(
        Quality::Masterwork
            .worked(content.object(id))
            .expect("arms"),
    );
    w.player_mut().inventory.push(stack);
}

/// A plain Masterwork long sword at the end of the pack.
fn plain_sword(w: &mut World) {
    item(w, "long_sword", 1);
    let content = w.content.clone();
    let stack = w.player_mut().inventory.last_mut().expect("the sword");
    let worked = Quality::Masterwork.worked(content.object(stack.object));
    stack.set_worked(worked.expect("arms"));
}

/// The index of family `category` in `[[family]]` (a seal's pick).
fn family(w: &World, category: &str) -> usize {
    let table = w.content.crystals().expect("crystals");
    (table.families.iter())
        .position(|(c, _)| c == category)
        .expect(category)
}

fn category(w: &World, affix: &Affix) -> String {
    (w.content.affixes().get(&affix.id))
        .expect("affix")
        .category
        .clone()
}

fn last(w: &World) -> String {
    (w.messages().iter().last()).map_or_else(String::new, |m| m.text.clone())
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

fn apply(w: &mut World, text: &str) {
    for action in parse_actions(text).expect("parses") {
        w.apply(&action);
    }
}

fn recipe(w: &World, id: &str) -> crate::content::RecipeId {
    w.content.recipes().id(id).expect(id)
}

/// The magic of pack stack `at`, checked as a save would be.
fn checked(w: &World, at: usize) -> Magic {
    let stack = &w.state.player().inventory[at];
    let magic = stack.magic().expect("magic").clone();
    let def = w.content.object(stack.object);
    magic.check(def, &w.content).expect("Magic::check holds");
    magic
}

#[test]
fn a_seal_shuts_its_family_out_of_the_roll() {
    // Physical damage, fire, and the critical sealed: a red Kindle never rolls them.
    let shut = ["physical_damage", "fire_damage", "critical_chance"];
    let mut seen = Vec::new();
    for seed in 1..60 {
        let mut w = world(seed);
        plain_sword(&mut w); // a
        crystals(&mut w, "red", 4); // b
        let seals: Vec<Option<usize>> = shut.iter().map(|c| Some(family(&w, c))).collect();
        let mut picks = vec![Some(0), Some(1)];
        picks.extend(seals);
        enchant_out_for_test(&mut w, "kindle", &picks, EnchantBand::Made);
        let magic = checked(&w, 0);
        let rolled = category(&w, &magic.affixes[0]);
        assert!(!shut.contains(&rolled.as_str()), "{rolled} was sealed");
        seen.push(rolled);
        // The crystal and one for each seal: all four.
        assert_eq!(w.state.player().inventory.len(), 1);
    }
    seen.sort();
    seen.dedup();
    assert!(seen.len() > 1, "the roll still varies: {seen:?}");
}

#[test]
fn each_seal_costs_a_crystal_of_the_colour_and_half_an_hour() {
    let mut w = world(3);
    plain_sword(&mut w); // a
    crystals(&mut w, "red", 4); // b
    let (fire, crit) = (family(&w, "fire_damage"), family(&w, "critical_chance"));
    let plan = (w.perception())
        .craft_plan(
            recipe(&w, "kindle"),
            &[Some(0), Some(1), Some(fire), Some(crit)],
        )
        .expect("a plan");
    assert_eq!(
        plan.uses,
        "a red crystal and two more red crystals, sealing out fire and the critical"
    );
    assert_eq!(plan.time, "about 2 hours");
    let start = w.state.time;
    apply(&mut w, &format!("craft:kindle:a,b,{fire},{crit}"));
    assert_eq!(w.state.time.since(start).as_seconds(), 3600 + 2 * 1800);
    let left = &w.state.player().inventory;
    let red = left.iter().find(|s| s.object != left[0].object);
    assert_eq!(red.map(|s| s.count), Some(1), "three of the four spent");
}

#[test]
fn a_seal_is_refused_with_no_time_where_it_cannot_hold() {
    let mut w = world(1);
    plain_sword(&mut w); // a
    crystals(&mut w, "red", 2); // b
    sword(&mut w, &["sharp"], false); // c
    crystals(&mut w, "blue", 1); // d
    let fire = family(&w, "fire_damage");
    let mana = family(&w, "mana");
    let damage = family(&w, "physical_damage");
    let crit = family(&w, "critical_chance");
    let before = w.state.time;
    for (picks, line) in [
        (
            format!("craft:kindle:a,b,{mana}"),
            "The red crystal holds nothing of mana.".to_string(),
        ),
        (
            format!("craft:kindle:a,b,{fire},{fire}"),
            "You have sealed that already.".into(),
        ),
        (
            format!("craft:quicken:c,b,{damage}"),
            "There is nothing to seal there.".into(),
        ),
        (
            format!("craft:kindle:a,b,{fire},{crit}"),
            "You carry too few crystals for the seals.".into(),
        ),
        (
            "craft:kindle:a,b,999".into(),
            "There is no such seal.".into(),
        ),
        (
            "craft:kindle:a,b,-,-,-,0".into(),
            "That is more than the work takes.".into(),
        ),
    ] {
        apply(&mut w, &picks);
        assert_eq!(last(&w), line, "{picks}");
    }
    assert_eq!(w.state.time, before);
    // A blue crystal seals mana, but each seal takes one more from the stack picked.
    apply(&mut w, &format!("craft:kindle:a,d,{mana}"));
    assert_eq!(last(&w), "You carry too few crystals for the seals.");
    crystals(&mut w, "blue", 1); // e: two blue crystals, one stack each
    apply(&mut w, &format!("craft:kindle:a,d,{mana}"));
    assert_eq!(last(&w), "You carry too few crystals for the seals.");
}

#[test]
fn the_seal_steps_follow_the_crystal_and_the_item() {
    let mut w = world(1);
    sword(&mut w, &["sharp"], false); // a
    crystals(&mut w, "red", 2); // b
    crystals(&mut w, "blue", 1); // c
    let damage = word(&w.content, "physical_damage");
    let view = w.perception();
    let quicken = recipe(&w, "quicken");
    let words = |picks: &[Option<usize>], step: usize| -> Vec<String> {
        (view.craft_steps_given(quicken, picks)[step].options.iter())
            .map(|o| o.name.clone())
            .collect()
    };
    let steps = view.craft_steps_given(quicken, &[]);
    assert_eq!(
        (steps.iter().map(|s| s.label.as_str())).collect::<Vec<_>>(),
        ["item", "crystal", "seal", "second seal", "third seal"]
    );
    // Before a crystal is picked, every colour carried; then the red's alone.
    let all = words(&[], 2);
    assert!(all.iter().any(|w| w == "mana") && all.iter().any(|w| w == "fire"));
    let red = words(&[Some(0), Some(1)], 2);
    assert!(!red.iter().any(|w| w == "mana"), "{red:?}");
    assert!(red.iter().any(|w| w == "fire"), "{red:?}");
    // The sword has its physical damage: nothing to seal there.
    assert!(!red.contains(&damage), "{red:?}");
    // A family sealed already is not offered again.
    let fire = family(&w, "fire_damage");
    let second = words(&[Some(0), Some(1), Some(fire)], 3);
    assert!(!second.iter().any(|w| w == "fire"), "{second:?}");
    let mut sorted = red.clone();
    sorted.sort();
    assert_eq!(red, sorted, "in words, in order");
}

fn word(content: &Content, category: &str) -> String {
    let table = content.crystals().expect("crystals");
    table.family_word(category).expect(category).to_string()
}

#[test]
fn a_rares_quicken_takes_a_heart_crystal() {
    let mut w = world(1);
    sword(&mut w, &["sharp", "flaming", "of_the_bear"], true); // a: room for one more
    crystals(&mut w, "red", 1); // b
    let before = w.state.time;
    apply(&mut w, "craft:quicken:a,b");
    assert_eq!(
        last(&w),
        "A rare takes a heart-crystal beside the crystal, and you have none."
    );
    assert_eq!(w.state.time, before);
    item(&mut w, "heart_crystal", 1); // c
    let plan = (w.perception())
        .craft_plan(recipe(&w, "quicken"), &[Some(0), Some(1)])
        .expect("a plan");
    assert_eq!(plan.uses, "a red crystal and a heart-crystal");
    for seed in 1..20 {
        let mut w = world(seed);
        sword(&mut w, &["sharp", "flaming", "of_the_bear"], true);
        crystals(&mut w, "red", 1);
        item(&mut w, "heart_crystal", 1);
        enchant_out_for_test(&mut w, "quicken", &[Some(0), Some(1)], EnchantBand::Made);
        let magic = checked(&w, 0);
        assert_eq!(magic.affixes.len(), 4);
        assert_eq!(magic.rarity, Rarity::Rare);
        assert_eq!(magic.name.as_deref(), Some("Grim Bite"));
        assert_eq!(w.state.player().inventory.len(), 1, "both spent");
        assert!(said(
            &w,
            "Grim Bite takes the red crystal's glow and the heart-crystal's, and holds more."
        ));
    }
    // Failed, the heart-crystal goes too; the rare as it was.
    let mut w = world(1);
    sword(&mut w, &["sharp", "flaming", "of_the_bear"], true);
    crystals(&mut w, "red", 1);
    item(&mut w, "heart_crystal", 1);
    enchant_out_for_test(&mut w, "quicken", &[Some(0), Some(1)], EnchantBand::Failed);
    assert_eq!(checked(&w, 0).affixes.len(), 3);
    assert_eq!(w.state.player().inventory.len(), 1);
    assert!(said(&w, "The heart-crystal goes dark with it."));
    // At its capacity, a rare holds no more.
    let mut w = world(1);
    sword(
        &mut w,
        &["sharp", "flaming", "of_the_bear", "of_precision"],
        true,
    );
    crystals(&mut w, "red", 1);
    item(&mut w, "heart_crystal", 1);
    apply(&mut w, "craft:quicken:a,b");
    assert_eq!(last(&w), "It holds all it can.");
}

#[test]
fn a_soul_fetter_keeps_one_affix_through_a_recast() {
    for seed in 1..20 {
        let mut w = world(seed);
        sword(&mut w, &["sharp", "of_the_bear"], false); // a
        crystals(&mut w, "red", 1); // b
        item(&mut w, "soul_fetter", 1); // c
        let kept = w.state.player().inventory[0]
            .magic()
            .expect("magic")
            .affixes[1]
            .clone();
        // Recast's picks: the item, the crystal, three seals, the affix kept (the bear's: 1).
        let picks = [Some(0), Some(1), None, None, None, Some(1)];
        enchant_out_for_test(&mut w, "recast", &picks, EnchantBand::Made);
        let magic = checked(&w, 0);
        assert_eq!(magic.affixes.len(), 2);
        assert_eq!(magic.affixes[1], kept, "the suffix kept exactly");
        assert_eq!(
            w.state.player().inventory.len(),
            1,
            "crystal and fetter spent"
        );
        assert!(said(
            &w,
            "The soul-fetter holds one enchantment fast: of the Bear."
        ));
    }
    let mut w = world(1);
    sword(&mut w, &["sharp", "of_the_bear"], false); // a
    crystals(&mut w, "red", 1); // b
    let before = w.state.time;
    apply(&mut w, "craft:recast:a,b,-,-,-,a");
    assert_eq!(last(&w), "You have no soul-fetter to keep it with.");
    item(&mut w, "soul_fetter", 1); // c
    apply(&mut w, "craft:recast:a,b,-,-,-,c");
    assert_eq!(last(&w), "There is no such magic in it.");
    sword(&mut w, &["sharp"], false); // d
    apply(&mut w, "craft:recast:d,b,-,-,-,a");
    assert_eq!(last(&w), "Nothing would be left to recast.");
    assert_eq!(w.state.time, before);
    // The wizard asks for the affix kept only while a fetter is carried, last.
    let view = w.perception();
    let steps = view.craft_steps_given(recipe(&w, "recast"), &[Some(0)]);
    let affix = steps.last().expect("a step");
    assert_eq!(affix.label, "affix kept");
    let names: Vec<&str> = affix.options.iter().map(|o| o.short.as_str()).collect();
    assert_eq!(names, ["Sharp", "of the Bear"]);
    let plan = (view.craft_plan(
        recipe(&w, "recast"),
        &[Some(0), Some(1), None, None, None, Some(0)],
    ))
    .expect("a plan");
    assert_eq!(plan.uses, "a red crystal and a soul-fetter (keeping Sharp)");
    // A failure spends the fetter too.
    let mut w = world(1);
    sword(&mut w, &["sharp", "of_the_bear"], false);
    crystals(&mut w, "red", 1);
    item(&mut w, "soul_fetter", 1);
    let picks = [Some(0), Some(1), None, None, None, Some(0)];
    enchant_out_for_test(&mut w, "recast", &picks, EnchantBand::Failed);
    assert_eq!(w.state.player().inventory.len(), 1);
    assert!(said(&w, "The soul-fetter crumbles to nothing."));
}

#[test]
fn imbues_seals_are_paid_by_the_first_crystal_of_their_colour() {
    let mut w = world(1);
    plain_sword(&mut w); // a: holds four
    crystals(&mut w, "red", 1); // b
    crystals(&mut w, "blue", 3); // c
    let mana = family(&w, "mana");
    let fire = family(&w, "fire_damage");
    let plan = (w.perception())
        .craft_plan(
            recipe(&w, "imbue"),
            &[Some(0), Some(1), Some(2), Some(2), Some(mana)],
        )
        .expect("a plan");
    assert_eq!(
        plan.uses,
        "a red crystal, a blue crystal, a blue crystal, and one more blue crystal, sealing out mana"
    );
    // Fire is the red's alone, and the red stack has nothing to spare.
    let refused = (w.perception()).craft_plan(
        recipe(&w, "imbue"),
        &[Some(0), Some(1), Some(2), Some(2), Some(fire)],
    );
    assert_eq!(
        refused,
        Err("You carry too few crystals for the seals.".to_string())
    );
    for seed in 1..15 {
        let mut w = world(seed);
        plain_sword(&mut w);
        crystals(&mut w, "red", 1);
        crystals(&mut w, "blue", 3);
        let picks = [Some(0), Some(1), Some(2), Some(2), Some(mana)];
        enchant_out_for_test(&mut w, "imbue", &picks, EnchantBand::Made);
        let magic = checked(&w, 0);
        assert!(
            magic.affixes.iter().all(|a| category(&w, a) != "mana"),
            "mana sealed"
        );
        assert_eq!(w.state.player().inventory.len(), 1, "all four spent");
    }
}

#[test]
fn a_sealed_kindle_replays() {
    let wizard: crate::character::CharacterSpec = "human,female,wizard".parse().expect("spec");
    let mut w =
        World::with_character(FIXTURE.clone(), "test_seal", Some(5), Some(&wizard)).expect("world");
    let mut replay = Replay::new(&w);
    for action in parse_actions("move:E pickup").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    let pack = &w.state.player().inventory;
    let find = |name: &str| {
        (pack.iter())
            .position(|s| w.lore().title(s) == name)
            .expect(name)
    };
    let (sword, stone) = (find("long sword"), find("red crystal"));
    let text = format!(
        "craft:kindle:{sword},{stone},{},{} wait",
        family(&w, "physical_damage"),
        family(&w, "strength")
    );
    let start = w.state.time;
    for action in parse_actions(&text).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert!(
        w.state.time.since(start).as_seconds() >= 3600 + 2 * 1800,
        "{}",
        last(&w)
    );
    let (verified, divergence) = replay.play(FIXTURE.clone()).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}

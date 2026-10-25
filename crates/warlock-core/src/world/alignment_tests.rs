//! Alignment (spec §8 "Alignment", #254) in play: starting bands, each deed that moves the score
//! (kills by stance and alignment, shopkeepers, theft, paying, eating kin, quests), diminishing,
//! band crossings said as notes, what reads it (cold greetings, quest offers, a good keeper's
//! prices, abilities that ask for an alignment), saves, and replays.

use super::*;
use crate::action::{Meal, parse_actions};
use crate::alignment::{Alignment, Band, Requirement, Score};
use crate::content::{ContentFiles, Sign};
use crate::object::Stack;
use crate::replay::Replay;

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
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "folk"
name = "folk"

[[faction]]
id = "wild"
name = "wild beasts"

[[faction]]
id = "foes"
name = "foes"
hostile = ["player"]
"##;

// Everyone but the player is frail (one hit point), so a kick or two kills.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "priest"
name = "priest"
glyph = "@"
fg = "#ffffc0"
faction = "folk"
alignment = "good"
weight = 60000
stats = { toughness = 1, dexterity = 1 }

[[creature]]
id = "warlock"
name = "warlock"
glyph = "@"
fg = "#c040c0"
faction = "folk"
alignment = "evil"
weight = 60000
stats = { toughness = 1, dexterity = 1 }

[[creature]]
id = "deer"
name = "deer"
plural = "deer"
glyph = "d"
fg = "#a07040"
faction = "wild"
weight = 50000
stats = { toughness = 1, dexterity = 1 }

[[creature]]
id = "imp"
name = "imp"
glyph = "i"
fg = "#e04040"
faction = "foes"
alignment = "evil"
weight = 5000
stats = { toughness = 1, dexterity = 1, strength = 1 }

[[creature]]
id = "wolf"
name = "wolf"
glyph = "C"
fg = "#909090"
faction = "foes"
weight = 30000
stats = { toughness = 1, dexterity = 1, strength = 1 }

[[creature]]
id = "knight"
name = "knight"
glyph = "K"
fg = "#c0c0ff"
faction = "foes"
alignment = "good"
weight = 80000
stats = { toughness = 1, dexterity = 1, strength = 1 }
"##;

const MATERIALS: &str = r##"
[[material]]
id = "plant"
name = "plant"
"##;

const OBJECTS: &str = r##"
[[object]]
id = "apple"
name = "apple"
glyph = "%"
fg = "#d84838"
class = "food"
materials = ["plant"]
weight = 150
value = 0
stackable = true
"##;

const DIALOGUE: &str = r##"
[[dialogue]]
creature = "priest"
greet = "Bless you, child."
greet_evil = ["Begone, foul one.", "I will pray for you. From afar."]
lines = ["Peace be with you."]

[[dialogue]]
creature = "warlock"
greet = "Well met, friend."
greet_good = ["Your light offends me."]
lines = ["Heh."]
"##;

const QUESTS: &str = r##"
[[quest]]
id = "alms"
name = "Alms"
giver = "priest"
type = "bring"
target = "apple"
alignment = 40
offer = "An apple for the poor?"
accept = "Bless you."
reminder = "The apple?"
done = "The poor thank you."

[[quest]]
id = "chore"
name = "A chore"
giver = "priest"
type = "bring"
target = "apple"
offer = "An apple for my own table?"
accept = "Good."
reminder = "My apple?"
done = "Crunchy."

[[quest]]
id = "blessing"
name = "A blessing"
giver = "priest"
type = "bring"
target = "apple"
requires_alignment = { min = "kind" }
offer = "You have a kind heart. An apple for the altar?"
accept = "Good."
reminder = "The altar waits."
done = "Blessed be."

[[quest]]
id = "dark"
name = "A dark errand"
giver = "warlock"
type = "bring"
target = "apple"
alignment = -60
requires_alignment = { max = "callous" }
offer = "A poisoned apple, for the priest. Bring me one to poison."
accept = "Heh."
reminder = "The apple."
done = "Heh heh."
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
"@" = { terrain = "floor", spawn = "player" }
"P" = { terrain = "floor", creature = "priest" }
"V" = { terrain = "floor", creature = "warlock" }
"d" = { terrain = "floor", creature = "deer" }
"i" = { terrain = "floor", creature = "imp" }
"C" = { terrain = "floor", creature = "wolf" }
"K" = { terrain = "floor", creature = "knight" }
"k" = { terrain = "floor", creature = "imp", faction = "folk" }
"g" = { terrain = "floor", creature = "imp", alignment = "good" }
"##;

/// Fixture content with the built-in alignment rules, the yard on `map`.
fn files_with(map: &str) -> ContentFiles {
    let mut files = ContentFiles::default();
    files.insert(
        "data/item_classes.toml",
        crate::content::BUILTIN_ITEM_CLASSES,
    );
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", CREATURES),
        ("data/materials.toml", MATERIALS),
        ("data/objects.toml", OBJECTS),
        ("data/dialogue.toml", DIALOGUE),
        ("data/quests.toml", QUESTS),
        ("data/legend.toml", LEGEND),
        (
            "data/alignment.toml",
            include_str!("../../../../assets/data/alignment.toml"),
        ),
        (
            "data/vision.toml",
            "[[vision]]\nid = \"normal\"\nname = \"Normal\"\nthreshold = 40\n",
        ),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "levels/yard.toml",
        &format!("name = \"The Yard\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/yard.toml",
        "name = \"Yard\"\nstart_level = \"yard\"\nhp = 30\n\
         stats = { strength = 30, dexterity = 30 }\ncarry = [\"apple*5\"]\n",
    );
    files
}

fn yard(map: &str) -> World {
    let content = Content::load(&files_with(map)).expect("test content is valid");
    World::new(Arc::new(content), "yard", None).expect("world")
}

fn builtin() -> Arc<Content> {
    Arc::new(Content::builtin().expect("built-in content"))
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn score(w: &World) -> i32 {
    w.state
        .player()
        .alignment
        .expect("the player has a score")
        .value()
}

fn set_score(w: &mut World, value: i32) {
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").alignment = Some(Score::new(value));
}

/// What the player's birth animal added to their starting score (spec §8 "Alignment").
fn nudge(w: &World) -> i32 {
    let sign = (w.character().and_then(|c| c.sign.as_ref())).expect("a sign");
    w.content()
        .horoscope()
        .expect("a horoscope")
        .alignment(sign)
}

fn band(w: &World) -> Band {
    w.perception().own_alignment().expect("alignment rules")
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.text.contains(text))
}

fn last(w: &World) -> String {
    (w.messages().iter().last()).map_or(String::new(), |m| m.text.clone())
}

/// Kicks whoever stands in `dir` until they are dead (they have one hit point).
fn kick_to_death(w: &mut World, dir: Dir) {
    let at = w.player_pos().step(dir);
    for _ in 0..40 {
        if w.state.actor_at("yard", at).is_none() {
            return;
        }
        w.apply(&Action::Kick(dir));
    }
    panic!("nobody died at {at}");
}

#[test]
fn characters_start_in_their_race_class_and_sign_band() {
    let content = builtin();
    let rules = content.alignment().expect("alignment rules");
    let start = |spec: &str| {
        let spec: CharacterSpec = spec.parse().expect("spec");
        let w =
            World::with_character(content.clone(), "showcase", None, Some(&spec)).expect("world");
        (score(&w) - nudge(&w), score(&w), band(&w))
    };
    // The race's and the class's parts; a race leans a little, never alone out of Neutral.
    for (spec, theirs) in [
        ("human,male,paladin", 450),
        ("human,female,thief", -200),
        ("human,other,witch", -200),
        ("dwarf,female,warrior", 0),
        ("elf,female,wizard", 50),
        ("half_orc,male,paladin", 400),
        ("halfling,male,thief", -150),
    ] {
        let (before_the_stars, score, band) = start(spec);
        assert_eq!(before_the_stars, theirs, "{spec}");
        assert_eq!(band, rules.band(Score::new(score)), "{spec}");
    }
    // The showcase's human paladin is Basilisk-born: the stars take them down to Kind.
    assert_eq!(start("human,male,paladin"), (450, 350, Band::Kind));
    // The summary's start says the same, before the run.
    let spec: CharacterSpec = "human,male,paladin".parse().unwrap();
    let showcase = content.scenario("showcase").expect("showcase");
    let s = spec.start(&content, 1, showcase.start).expect("start");
    let w = World::with_character(content.clone(), "showcase", Some(1), Some(&spec)).expect("w");
    assert_eq!(s.alignment, Score::new(score(&w)));
}

#[test]
fn the_birth_animal_nudges_the_starting_score_and_the_element_not() {
    // The design's table (docs/design/horoscope.md §9.2), pinned against the data.
    let table = [
        ("salamander", 0),
        ("hydra", 0),
        ("manticore", -100),
        ("unicorn", 100),
        ("wyvern", 50),
        ("basilisk", -100),
        ("roc", 0),
        ("griffin", 50),
        ("sphinx", 0),
        ("phoenix", 50),
        ("kraken", -50),
        ("barghest", 0),
    ];
    let content = builtin();
    let horoscope = content.horoscope().expect("a horoscope");
    assert!(horoscope.elements.iter().all(|e| e.alignment == 0));
    let today = content.scenario("showcase").expect("showcase").start;
    // A human warrior's own part is 0, so the start is the nudge alone; a paladin's 450 plus it.
    let mut seen = std::collections::BTreeMap::new();
    for seed in 0..400 {
        for (spec, theirs) in [("human,male,warrior", 0), ("human,female,paladin", 450)] {
            let spec: CharacterSpec = spec.parse().unwrap();
            let s = spec.start(&content, seed, today).expect("start");
            let sign = s.character.sign.clone().expect("a sign");
            let (_, nudge) = *table
                .iter()
                .find(|(a, _)| *a == sign.animal)
                .expect("known");
            assert_eq!(s.alignment, Score::new(theirs + nudge), "{spec} {sign:?}");
            seen.insert(sign.animal.clone(), sign.element.clone());
        }
    }
    assert_eq!(seen.len(), table.len(), "every animal seen: {seen:?}");
    // The same animal under another element: the same nudge.
    let wood = Sign {
        animal: "unicorn".into(),
        element: "wood".into(),
    };
    let water = Sign {
        element: "water".into(),
        ..wood.clone()
    };
    assert_eq!(horoscope.alignment(&wood), 100);
    assert_eq!(horoscope.alignment(&water), 100);
    // Kept on the scale: a Unicorn-born at the far good end stays there.
    assert_eq!(Score::new(1000 + 100), Score::new(1000));
}

#[test]
fn a_run_without_a_character_starts_neutral_and_dumps_say_the_band_only() {
    let w = yard("#####\n#@..#\n#####\n");
    assert_eq!((score(&w), band(&w)), (0, Band::Neutral));
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Stats);
    assert!(rows.iter().any(|r| r == "Alignment Neutral"), "{rows:?}");
    let json = crate::dump::json(&w, &[], false);
    assert_eq!(json["alignment"], "neutral");
    // The dev layer has the number and every creature's nature.
    let actors = crate::dump::layer_rows(&w, crate::dump::Layer::Actors).join("\n");
    assert!(actors.contains("; alignment 0 (neutral)"), "{actors}");
    // Without the rules, nobody has a score and nothing is shown.
    let mut files = files_with("#####\n#@..#\n#####\n");
    files.remove("data/alignment.toml");
    let content = Arc::new(Content::load(&files).expect("content"));
    let w = World::new(content, "yard", None).expect("world");
    assert_eq!(w.state.player().alignment, None);
    assert_eq!(w.perception().own_alignment(), None);
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Stats);
    assert!(!rows.iter().any(|r| r.starts_with("Alignment")), "{rows:?}");
}

#[test]
fn killing_the_peaceful_is_evil_and_the_more_so_for_the_good() {
    // A good priest who talks: the worst.
    let mut w = yard("#####\n#P@.#\n#####\n");
    kick_to_death(&mut w, Dir::W);
    assert_eq!(score(&w), -150);
    // A peaceful evil creature (an imp the level made folk): evil still.
    let mut w = yard("#####\n#k@.#\n#####\n");
    assert_eq!(w.stance(w.player_id(), ActorId(1)), Stance::Neutral);
    kick_to_death(&mut w, Dir::W);
    assert_eq!(score(&w), -100);
    // A peaceful neutral creature that talks is a person: evil.
    let mut w = yard("#####\n#V@.#\n#####\n");
    let warlock = ActorId(1);
    w.state.actors.get_mut(&warlock).unwrap().alignment_override = Some(Alignment::Neutral);
    kick_to_death(&mut w, Dir::W);
    assert_eq!(score(&w), -100);
    // A deer (a beast: neutral, no words) may be hunted.
    let mut w = yard("#####\n#d@.#\n#####\n");
    kick_to_death(&mut w, Dir::W);
    assert_eq!(score(&w), 0);
}

#[test]
fn killing_hostile_evil_is_a_little_good_and_other_foes_nothing() {
    let mut w = yard("#####\n#i@.#\n#####\n");
    kick_to_death(&mut w, Dir::W);
    assert_eq!(score(&w), 10);
    // A hostile neutral beast and a hostile good knight: nothing.
    for map in ["#####\n#C@.#\n#####\n", "#####\n#K@.#\n#####\n"] {
        let mut w = yard(map);
        kick_to_death(&mut w, Dir::W);
        assert_eq!(score(&w), 0, "{map}");
    }
    // A level may make an evil imp good: then its death is no good deed.
    let mut w = yard("#####\n#g@.#\n#####\n");
    assert_eq!(
        w.state.actors[&ActorId(1)].nature(w.content()),
        Alignment::Good
    );
    kick_to_death(&mut w, Dir::W);
    assert_eq!(score(&w), 0);
}

#[test]
fn deeds_diminish_the_further_along_the_player_is() {
    let mut w = yard("#####\n#i@.#\n#####\n");
    set_score(&mut w, 500);
    kick_to_death(&mut w, Dir::W);
    assert_eq!(score(&w), 505, "half of 10 halfway to good");
    // Against the lean, the full weight.
    let mut w = yard("#####\n#P@.#\n#####\n");
    set_score(&mut w, 500);
    kick_to_death(&mut w, Dir::W);
    assert_eq!(score(&w), 350);
    let mut w = yard("#####\n#P@.#\n#####\n");
    set_score(&mut w, -500);
    kick_to_death(&mut w, Dir::W);
    assert_eq!(score(&w), -575, "half of -150 halfway to evil");
}

#[test]
fn crossing_into_a_band_is_said_as_a_note_after_the_deed() {
    let mut w = yard("#####\n#P@.#\n#####\n");
    set_score(&mut w, 155);
    assert_eq!(band(&w), Band::Kind);
    let before = w.said;
    kick_to_death(&mut w, Dir::W);
    assert_eq!(band(&w), Band::Neutral);
    assert_eq!(last(&w), "You feel your kindness cool.");
    let kill = w
        .messages()
        .iter()
        .position(|m| m.text == "The priest dies.");
    let note = w
        .messages()
        .iter()
        .position(|m| m.text == "You feel your kindness cool.");
    let all: Vec<String> = w.messages().iter().map(|m| m.text.clone()).collect();
    assert!(
        kill.is_some() && kill < note,
        "after the kill's own messages: {all:?}"
    );
    // The note did not count as something said (a rest would go on): only the kicks did.
    let kicks = w.said - before;
    let mut quiet = yard("#####\n#P@.#\n#####\n");
    set_score(&mut quiet, 400);
    let before = quiet.said;
    kick_to_death(&mut quiet, Dir::W);
    assert_eq!(band(&quiet), Band::Kind, "400 - 150");
    assert_eq!(
        last(&quiet),
        "You feel you have strayed from the path of good."
    );
    assert_eq!(quiet.said - before, kicks, "the same messages but the note");
    // Small moves within a band are silent.
    let mut w = yard("#####\n#i@.#\n#####\n");
    kick_to_death(&mut w, Dir::W);
    assert!(!said(&w, "You feel"), "{}", last(&w));
}

#[test]
fn good_folk_greet_the_evil_coldly_and_evil_folk_the_good() {
    let mut w = yard("#####\n#P@V#\n#####\n");
    run(&mut w, "chat:W");
    assert_eq!(last(&w), "The priest says: \"Bless you, child.\"");
    set_score(&mut w, -500);
    assert_eq!(band(&w), Band::Evil);
    run(&mut w, "chat:W");
    assert!(last(&w).starts_with("The priest says: \""));
    let cold = ["Begone, foul one.", "I will pray for you. From afar."];
    assert!(cold.iter().any(|c| last(&w).contains(c)), "{}", last(&w));
    // Callous is not evil enough to be scorned.
    set_score(&mut w, -300);
    run(&mut w, "chat:W chat:W");
    assert_eq!(last(&w), "The priest says: \"Peace be with you.\"");
    // The warlock is cold to the good, from the first word.
    set_score(&mut w, 450);
    run(&mut w, "chat:E");
    assert_eq!(last(&w), "The warlock says: \"Your light offends me.\"");
    set_score(&mut w, 0);
    run(&mut w, "chat:E");
    assert_eq!(last(&w), "The warlock says: \"Heh.\"");
}

/// The quest topics the creature in `dir` offers now.
fn quest_topics(w: &World, dir: Dir) -> Vec<String> {
    let at = w.player_pos().step(dir);
    let id = w.state.actor_at("yard", at).expect("someone there").id;
    (w.topics(id).into_iter())
        .map(|t| t.id)
        .filter(|t| t.starts_with("quest-"))
        .collect()
}

#[test]
fn quests_ask_for_an_alignment_and_the_good_offer_no_good_to_the_evil() {
    let mut w = yard("#####\n#P@V#\n#####\n");
    // Neutral: the priest's alms and chore, not the blessing (Kind or better); the warlock's dark
    // errand wants Callous or worse.
    assert_eq!(quest_topics(&w, Dir::W), ["quest-alms", "quest-chore"]);
    assert!(quest_topics(&w, Dir::E).is_empty());
    set_score(&mut w, 200);
    assert_eq!(
        quest_topics(&w, Dir::W),
        ["quest-alms", "quest-chore", "quest-blessing"]
    );
    // Evil: the priest scorns them and offers nothing that would do them good; a chore still.
    set_score(&mut w, -500);
    assert_eq!(quest_topics(&w, Dir::W), ["quest-chore"]);
    assert_eq!(quest_topics(&w, Dir::E), ["quest-dark"]);
    // Asking about a quest not offered is refused.
    run(&mut w, "ask:W:quest-alms");
    assert_eq!(last(&w), "The priest knows nothing about that.");
}

#[test]
fn a_quest_moves_the_score_as_it_is_completed() {
    let mut w = yard("#####\n#P@V#\n#####\n");
    run(&mut w, "ask:W:quest-alms ask:W:accept-alms");
    assert_eq!(score(&w), 0, "nothing until it is done");
    run(&mut w, "ask:W:quest-alms");
    assert!(said(&w, "Quest complete: Alms."));
    assert_eq!(score(&w), 40);
    // A dark errand, the other way (from Callous, where the warlock offers it).
    set_score(&mut w, -200);
    run(
        &mut w,
        "ask:E:quest-dark ask:E:accept-dark ask:E:quest-dark",
    );
    assert!(said(&w, "Quest complete: A dark errand."));
    assert_eq!(score(&w), -200 - 60 * 800 / 1000);
    // Accepted, a quest is done whatever the player became meanwhile.
    let mut w = yard("#####\n#P@V#\n#####\n");
    run(&mut w, "ask:W:quest-alms ask:W:accept-alms");
    set_score(&mut w, -800);
    run(&mut w, "ask:W:quest-alms");
    assert!(said(&w, "Quest complete: Alms."));
}

/// From the courtyard's start into Grimbold's General Store (as `shop_tests.rs` walks it).
const INTO_SHOP: &str = "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 \
                         move:S*5 move:E*5";

fn in_shop() -> World {
    let mut w = World::new(builtin(), "courtyard", None).expect("courtyard");
    run(&mut w, INTO_SHOP);
    assert_eq!(w.player_pos(), Pos::new(54, 14), "in the shop");
    // A human druid's 0, before the stars (the courtyard's sign nudges it to -100).
    set_score(&mut w, 0);
    w
}

#[test]
fn paying_is_a_little_good_and_theft_and_attacking_the_keeper_evil() {
    let mut w = in_shop();
    run(&mut w, "move:E move:NE pickup pay");
    assert!(said(&w, "You pay Grimbold"));
    assert_eq!(score(&w), 5);
    // Paying nothing is no deed.
    run(&mut w, "pay");
    assert_eq!(score(&w), 5);
    // Walking off with a dagger through a hole in the wall: theft.
    let mut w = in_shop();
    run(&mut w, "move:E move:NE pickup move:SW move:E*7");
    assert_eq!(w.player_pos(), Pos::new(62, 14));
    let hole = Pos::new(63, 14);
    let floor = w.content().terrain_id("plank_floor").expect("plank floor");
    w.set_terrain(hole, floor);
    run(&mut w, "move:E");
    assert!(said(&w, "\"Stop, thief!\""));
    assert_eq!(score(&w), -100);
    // Attacking a keeper who trades with you.
    let mut w = in_shop();
    run(&mut w, "move:E move:NE pickup pay throw:c:54,13");
    assert!(said(&w, "\"You will pay for that!\""));
    assert_eq!(score(&w), 5 - 100);
    // Attacking him again, outraged already, is no further deed.
    let keeper = w.state.current().shops[0].keeper;
    w.provoke(keeper);
    assert_eq!(score(&w), 5 - 100);
}

#[test]
fn a_good_keeper_charges_the_evil_more_and_the_saintly_less() {
    let mut w = in_shop();
    let dagger = Stack::one(w.content().object_id("dagger").expect("dagger"));
    let at = Pos::new(56, 13);
    let price = |w: &World| w.perception().price_tag(at, &dagger).expect("for sale");
    let neutral = price(&w);
    set_score(&mut w, 900);
    let saintly = price(&w);
    set_score(&mut w, -500);
    let evil = price(&w);
    set_score(&mut w, -900);
    let wicked = price(&w);
    assert!(
        saintly < neutral && neutral < evil && evil < wicked,
        "{saintly} {neutral} {evil} {wicked}"
    );
    // The rules ask the same price as look mode shows.
    assert_eq!(w.asking_price(at, &dagger), Some(wicked));
    // Kind or Callous: no difference.
    set_score(&mut w, 200);
    assert_eq!(price(&w), neutral);
    set_score(&mut w, -200);
    assert_eq!(price(&w), neutral);
}

#[test]
fn eating_kin_is_evil() {
    let content = builtin();
    let eat = |spec: &str, creature: &str, files: Option<ContentFiles>| {
        let content = match files {
            Some(f) => Arc::new(Content::load(&f).expect("content")),
            None => content.clone(),
        };
        let spec: CharacterSpec = spec.parse().expect("spec");
        // The quarry, where nobody comes to interrupt the meal.
        let mut w =
            World::with_character(content.clone(), "quarry", None, Some(&spec)).expect("world");
        let corpse = (content.creature_id(creature))
            .and_then(|c| content.corpse_of(c))
            .expect("a corpse");
        let here = w.player_pos();
        let fresh = Stack::corpse(corpse, w.time());
        w.state.current_mut().set_pile(here, vec![fresh]);
        // Neutral, whatever the stars gave (a deed counts in full from 0).
        set_score(&mut w, 0);
        let before = score(&w);
        w.apply(&Action::Eat(Meal::Floor(0)));
        assert!(said(&w, "You eat the"), "{}", last(&w));
        score(&w) - before
    };
    // A gardener: a peaceful creature who talks, and a human: kin to anyone.
    assert_eq!(eat("dwarf,male,warrior", "gardener", None), -200);
    assert_eq!(eat("human,male,warrior", "gardener", None), -200);
    // A rat: no kin of anyone's.
    assert_eq!(eat("human,male,warrior", "rat", None), 0);
    // A hostile kobold of the player's own race would be kin by race alone.
    let mut files = ContentFiles::builtin();
    files.edit_table("creatures", |text| {
        text.replace("id = \"kobold\"\n", "id = \"kobold\"\nrace = \"human\"\n")
    });
    assert_eq!(
        eat("human,male,warrior", "kobold", Some(files.clone())),
        -200
    );
    assert_eq!(eat("elf,male,warrior", "kobold", Some(files)), 0);
}

#[test]
fn a_paladin_who_kills_the_peaceful_strays_from_their_gifts() {
    // The paladin's abilities come with the class tables (#243); a test ability asks the same.
    let content = builtin();
    let spec: CharacterSpec = "human,male,paladin".parse().expect("spec");
    let mut w = World::with_character(content, "showcase", None, Some(&spec)).expect("world");
    // The paladin's own start, before the stars (the showcase's is Basilisk-born).
    set_score(&mut w, 450);
    let gift = Some(Requirement {
        min: Some(Band::Good),
        max: None,
    });
    let strayed = |w: &World| crate::ability::strayed(w.state.player(), w.content(), gift);
    assert_eq!(strayed(&w), None);
    // A peaceful good creature slain: Good falls to Kind.
    let hermit = (w.state.actors.values())
        .find(|a| w.content().creature(a.creature).id == "hermit")
        .map(|a| a.id)
        .expect("the hermit");
    w.judge_kill(hermit);
    w.say_news();
    assert_eq!(score(&w), 300);
    assert_eq!(band(&w), Band::Kind);
    assert_eq!(last(&w), "You feel you have strayed from the path of good.");
    assert_eq!(strayed(&w), Some(crate::ability::Unavailable::Strayed));
    // Climbing back makes them usable again.
    set_score(&mut w, 420);
    assert_eq!(strayed(&w), None);
}

#[test]
fn the_score_survives_a_save_and_a_replay_exactly() {
    let actions = "kick:W*6 chat:E";
    let mut w = yard("######\n#i@P.#\n######\n");
    let mut replay = Replay::new(&w);
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_eq!(score(&w), 10);
    let content = w.content().clone();
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content.clone())
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    assert_eq!(loaded.perception().own_alignment(), Some(Band::Neutral));
    // A score off the scale does not load.
    let mut json = serde_json::to_value(w.state()).expect("json");
    json["actors"]["0"]["alignment"] = serde_json::json!(1001);
    assert!(serde_json::from_value::<WorldState>(json).is_err());
}

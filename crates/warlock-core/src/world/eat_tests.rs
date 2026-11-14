//! A fresh corpse's gifts to the stats (spec §8 "Hunger"; `world/eat.rs::corpse_gifts`, design
//! `docs/design/stat-training.md` §8): sure training marks into the pile, a rare chance to widen
//! the potential, never a score; the tags' defaults under each creature's own `on_eat`; the
//! words; Gourmand's doubling; and nothing from a rotten or vomited corpse.

use super::*;
use crate::action::Meal;
use crate::content::ContentFiles;
use crate::stats::Stat;

/// Food underfoot, nothing else about.
const PANTRY: &str = r##"
name = "Pantry"
ambient = [200, 200, 200, 0]
map = '''
#######
#@....#
#######
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"##;

/// The built-in content with the pantry, each creature file's `from` replaced by `to`.
fn content_with(edits: &[(&str, &str)]) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.insert("levels/pantry.toml", PANTRY);
    files.insert(
        "scenarios/pantry.toml",
        "name = \"pantry\"\nstart_level = \"pantry\"\nhp = 1000\n",
    );
    for (from, to) in edits {
        let mut found = false;
        files.edit_table("creatures", |t| {
            found |= t.contains(from);
            t.replace(from, to)
        });
        assert!(found, "{from}");
    }
    Arc::new(Content::load(&files).expect("test content is valid"))
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.text == text)
}

fn marks(w: &World, stat: Stat) -> i32 {
    w.state.player().stat_marks[stat]
}

/// Lays a corpse of `creature` that died `ago` seconds back on the player's cell and eats it,
/// after a meal that leaves room for it.
fn eat(w: &mut World, creature: &str, ago: u64) {
    let corpse = (w.content().object_id(&format!("{creature}_corpse"))).expect("a corpse");
    let died = GameTime::from_seconds(w.time().seconds() - ago);
    let here = w.player_pos();
    w.state
        .current_mut()
        .set_pile(here, vec![Stack::corpse(corpse, died)]);
    let player = w.state.player;
    w.state.actors.get_mut(&player).unwrap().satiation = Some(crate::hunger::Satiation::new(1000));
    w.apply(&Action::Eat(Meal::Floor(0)));
}

fn pantry(content: &Arc<Content>) -> World {
    World::new(content.clone(), "pantry", Some(7)).expect("world")
}

#[test]
fn tags_give_corpses_their_gifts_and_a_creature_its_own_over_them() {
    let content = Content::builtin().expect("content loads");
    let gifts = |id: &str| {
        let creature = content.creature(content.creature_id(id).expect(id));
        let eat = creature.on_eat.clone().unwrap_or_default();
        (eat.train, eat.widen)
    };
    let one = |stat: Stat, n: u16| [(stat, n)].into_iter().collect();
    // A giant's heart: the giant tag's.
    let (train, widen) = gifts("hill_giant");
    assert_eq!(train, one(Stat::Strength, 60));
    assert_eq!(widen, [(Stat::Strength, 5)].into_iter().collect());
    // A troll: the troll tag's, its old hand-set chance gone under it.
    let (train, widen) = gifts("troll");
    assert_eq!(train, one(Stat::Toughness, 60));
    assert_eq!(widen, [(Stat::Toughness, 5)].into_iter().collect());
    // An ogre is a giant too, but its own `on_eat` sets the ogre row's Strength over the
    // giant's; the ogre tag's Toughness stays.
    let (train, widen) = gifts("ogre");
    assert_eq!(
        train,
        [(Stat::Strength, 40), (Stat::Toughness, 15)]
            .into_iter()
            .collect()
    );
    assert_eq!(widen, [(Stat::Strength, 2)].into_iter().collect());
    // Plain meat: an orc's 8 Toughness marks, the warlord's Strength of its own beside them.
    assert_eq!(gifts("orc_warrior").0, one(Stat::Toughness, 8));
    assert_eq!(
        gifts("orc_warlord").0,
        [(Stat::Strength, 18), (Stat::Toughness, 8)]
            .into_iter()
            .collect()
    );
    // The beetle, untagged, by hand.
    assert_eq!(gifts("beetle").0, one(Stat::Toughness, 60));
    // The undead switch their tags' gifts off.
    for creature in content.creature_defs() {
        if creature.tags.iter().any(|t| t == "undead")
            && let Some(eat) = &creature.on_eat
        {
            assert!(
                eat.train.is_empty() && eat.widen.is_empty(),
                "{}",
                creature.id
            );
        }
    }
}

#[test]
fn the_old_stat_is_an_unknown_field() {
    let mut files = ContentFiles::builtin();
    let beetle = "train = { toughness = 60 }";
    files.edit_table("creatures", |t| {
        t.replace(beetle, "stat = { toughness = 10 }")
    });
    let e = Content::load(&files).expect_err("refused").to_string();
    assert!(e.contains("unknown field `stat`"), "{e}");
}

#[test]
fn a_corpse_gives_marks_never_a_score_and_many_are_felt() {
    let content = content_with(&[]);
    let mut w = pantry(&content);
    let stats = w.state.player().stats;
    let before = marks(&w, Stat::Toughness);
    eat(&mut w, "beetle", 0);
    assert!(said(&w, "Crunchy."));
    assert_eq!(marks(&w, Stat::Toughness), before + 60);
    assert!(said(&w, "Something of its hardiness passes into you."));
    assert_eq!(w.state.player().stats, stats, "no score, no potential");
    assert!(!said(&w, "You feel tougher!"));
    // A rat's few marks are quiet.
    let mut w = pantry(&content);
    eat(&mut w, "rat", 0);
    assert_eq!(marks(&w, Stat::Toughness), before + 4);
    assert!(!(w.messages().iter()).any(|m| m.text.starts_with("Something of its")));
}

#[test]
fn a_rotten_or_vomited_corpse_gives_nothing() {
    let content = content_with(&[(
        "on_eat = { message = \"This kobold meat tastes terrible!\", vomit = 40 }",
        "on_eat = { message = \"This kobold meat tastes terrible!\", vomit = 100 }",
    )]);
    let mut w = pantry(&content);
    // (Other uses may train other stats meanwhile: the kobold's and the beetle's Toughness.)
    let before = marks(&w, Stat::Toughness);
    eat(&mut w, "kobold", 0);
    assert!(said(&w, "You vomit!"));
    // The corpse gives nothing, and vomiting is a deed: it takes Toughness marks (#724).
    let after_vomit = before - 10;
    assert_eq!(marks(&w, Stat::Toughness), after_vomit);
    // Rotten: 10 hours dead, past its 8.
    eat(&mut w, "beetle", 10 * 3600);
    assert!(said(&w, "Ugh! The cave beetle corpse was rotten."));
    // No gift (a rotten corpse may make one vomit again: another deed, never a gain).
    assert!(marks(&w, Stat::Toughness) <= after_vomit);
}

#[test]
fn a_corpse_may_widen_the_potential_and_gourmand_doubles_its_gifts() {
    let content = content_with(&[(
        "train = { toughness = 60 }",
        "train = { toughness = 60 }, widen = { toughness = 20 }",
    )]);
    let mut w = pantry(&content);
    let (score, potential) = {
        let s = w.state.player().stats;
        (s.score(Stat::Toughness), s.potential(Stat::Toughness))
    };
    let before = marks(&w, Stat::Toughness);
    for _ in 0..30 {
        eat(&mut w, "beetle", 0);
    }
    let widened = (w.messages().iter())
        .filter(|m| m.text == "You feel you could grow tougher yet.")
        .count();
    let stats = w.state.player().stats;
    assert_eq!(stats.score(Stat::Toughness), score);
    assert_eq!(
        usize::from(stats.potential(Stat::Toughness) - potential),
        widened
    );
    // One in five, 30 times: 6 on average.
    assert!((1..=15).contains(&widened), "{widened}");
    assert_eq!(marks(&w, Stat::Toughness), before + 30 * 60);
    // Gourmand: twice the marks, twice the chance.
    let mut w = pantry(&content);
    w.player_mut().talents.take("gourmand", None);
    let before = marks(&w, Stat::Toughness);
    let potential = w.state.player().stats.potential(Stat::Toughness);
    for _ in 0..30 {
        eat(&mut w, "beetle", 0);
    }
    assert_eq!(marks(&w, Stat::Toughness), before + 30 * 120);
    let more = w.state.player().stats.potential(Stat::Toughness) - potential;
    assert!(usize::from(more) > widened, "{more} against {widened}");
}

/// The player's Decipher script, `None` while untrained.
fn decipher(w: &World) -> Option<u16> {
    let id = w.content().skill_id("decipher_script").expect("the skill");
    w.state.player().skills.value(id)
}

/// Sets the player's Decipher script (`None`: untrained), their other skills as they were.
fn set_decipher(w: &mut World, value: Option<u16>) {
    let id = w.content().skill_id("decipher_script").expect("the skill");
    let mut skills = crate::skills::Skills::default();
    for (other, trained) in w.state.player().skills.iter().filter(|(s, _)| *s != id) {
        skills.add(other, i32::from(trained.value));
    }
    if let Some(v) = value {
        skills.add(id, i32::from(v));
    }
    w.player_mut().skills = skills;
}

const LEARN: &str = "Strange letters swim before your eyes, and settle into sense.";
const RISE: &str = "The letters come clearer.";

#[test]
fn a_casters_corpse_teaches_decipher_script_up_to_its_cap() {
    let content = content_with(&[]);
    // The necromancer's: 6, up to 45.
    let gift = |id: &str| {
        let creature = content.creature(content.creature_id(id).expect(id));
        (creature.on_eat.as_ref()).and_then(|e| e.skills.get("decipher_script").copied())
    };
    let necromancer = gift("necromancer").expect("a caster's gift");
    assert_eq!((necromancer.gives, necromancer.up_to), (6, 45));
    // Beasts and the undead teach nothing.
    assert_eq!(gift("beetle"), None);
    assert_eq!(gift("mummy_priest"), None);
    // Untrained: learned at its `gives`.
    let mut w = pantry(&content);
    set_decipher(&mut w, None);
    eat(&mut w, "necromancer", 0);
    assert_eq!(decipher(&w), Some(6));
    assert!(said(&w, LEARN));
    assert!(!said(&w, RISE));
    // Trained: it rises by it.
    let mut w = pantry(&content);
    set_decipher(&mut w, Some(10));
    eat(&mut w, "necromancer", 0);
    assert_eq!(decipher(&w), Some(16));
    assert!(said(&w, RISE) && !said(&w, LEARN));
    // Never past the cap, and nothing at it.
    let mut w = pantry(&content);
    set_decipher(&mut w, Some(42));
    eat(&mut w, "necromancer", 0);
    assert_eq!(decipher(&w), Some(45));
    let mut w = pantry(&content);
    set_decipher(&mut w, Some(45));
    eat(&mut w, "necromancer", 0);
    assert_eq!(decipher(&w), Some(45));
    assert!(!said(&w, RISE) && !said(&w, LEARN));
    // Above the cap (from other sources): nothing taken.
    let mut w = pantry(&content);
    set_decipher(&mut w, Some(80));
    eat(&mut w, "necromancer", 0);
    assert_eq!(decipher(&w), Some(80));
}

#[test]
fn a_rotten_casters_corpse_teaches_nothing() {
    let content = content_with(&[]);
    let mut w = pantry(&content);
    set_decipher(&mut w, None);
    // Ten hours dead, past its eight.
    eat(&mut w, "necromancer", 10 * 3600);
    assert!(said(&w, "Ugh! The necromancer corpse was rotten."));
    assert_eq!(decipher(&w), None);
    assert!(!said(&w, LEARN));
}

#[test]
fn a_uniques_corpse_is_named_by_its_proper_name() {
    // #1660: "Gnægan the Rat King's corpse", never "the Gnægan the Rat King corpse".
    let content = content_with(&[]);
    let mut w = pantry(&content);
    let named = |id: &str| {
        let object = content.object_id(id).expect("a corpse");
        Stack::corpse(object, w.time()).describe(&content)
    };
    assert_eq!(named("gnaegan_corpse"), "Gnægan the Rat King's corpse");
    assert_eq!(named("kobold_corpse"), "a kobold corpse");
    eat(&mut w, "gnaegan", 60);
    assert!(said(&w, "You eat Gnægan the Rat King's corpse."));
    eat(&mut w, "kobold", 60);
    assert!(said(&w, "You eat the kobold corpse."));
}

#[test]
fn skill_gifts_without_words_are_refused() {
    let necromancer = "on_eat = { skills = { decipher_script = { gives = 6, up_to = 45 } } }";
    let refused = |files: ContentFiles| Content::load(&files).expect_err("refused").to_string();
    let edited = |to: &str| {
        let mut files = ContentFiles::builtin();
        let mut found = false;
        files.edit_table("creatures", |t| {
            found |= t.contains(necromancer);
            t.replace(necromancer, to)
        });
        assert!(found);
        files
    };
    let e = refused(edited(
        "on_eat = { skills = { stealth = { gives = 6, up_to = 45 } } }",
    ));
    assert!(
        e.contains("`stealth` has no words in [corpses.skills]"),
        "{e}"
    );
    let mut files = ContentFiles::builtin();
    let text = include_str!("../../../../assets/data/hunger.toml");
    files.insert(
        "data/hunger.toml",
        &text.replace("\ndecipher_script = {", "\ndecypher_script = {"),
    );
    let e = refused(files);
    assert!(e.contains("unknown skill `decypher_script`"), "{e}");
}

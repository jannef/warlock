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
    // Potential points are rare everywhere.
    for creature in content.creature_defs() {
        if let Some(eat) = &creature.on_eat {
            assert!(eat.widen.values().all(|&c| c <= 20), "{}", creature.id);
            assert!(eat.widen.keys().all(|s| eat.train.contains_key(s)));
        }
    }
}

#[test]
fn the_old_stat_and_gifts_out_of_range_are_refused() {
    let beetle = "train = { toughness = 60 }";
    let refused = |files: ContentFiles| Content::load(&files).expect_err("refused").to_string();
    let edited = |to: &str| {
        let mut files = ContentFiles::builtin();
        files.edit_table("creatures", |t| t.replace(beetle, to));
        files
    };
    for (to, says) in [
        ("stat = { toughness = 10 }", "`train"),
        ("train = { toughness = 201 }", "0 to 200 marks"),
        (
            "train = { toughness = 60 }, widen = { toughness = 21 }",
            "0 to 20",
        ),
        ("widen = { strength = 5 }", "needs a `train`"),
    ] {
        let e = refused(edited(to));
        assert!(e.contains(says), "{to}: {e}");
    }
    let hunger = |from: &str, to: &str| {
        let mut files = ContentFiles::builtin();
        let text = include_str!("../../../../assets/data/hunger.toml");
        assert!(text.contains(from), "{from}");
        files.insert("data/hunger.toml", &text.replace(from, to));
        files
    };
    let giant = "giant = { train = { strength = 60 }, widen = { strength = 5 } }";
    for (to, says) in [
        (
            "giant = { train = { strength = 60 }, widen = { strength = 6 } }",
            "1 to 5 for a tag",
        ),
        ("giant = { widen = { strength = 5 } }", "needs a `train`"),
        ("giant = { train = { strength = 0 } }", "1 to 200"),
        (
            "gaint = { train = { strength = 60 } }",
            "`gaint` is no creature's tag",
        ),
    ] {
        let e = refused(hunger(giant, to));
        assert!(e.contains(says), "{to}: {e}");
    }
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
    assert_eq!(marks(&w, Stat::Toughness), before);
    // Rotten: 10 hours dead, past its 8.
    eat(&mut w, "beetle", 10 * 3600);
    assert!(said(&w, "Ugh! The cave beetle corpse was rotten."));
    assert_eq!(marks(&w, Stat::Toughness), before);
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

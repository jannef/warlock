//! Salvage (docs/design/professions.md §5; `world/salvage.rs`): the owner's ruby-tipped hardwood
//! quarterstaff with both, one, and neither check made; a gem's size and cut kept, chipped, or
//! shattered; the untrained still rolling; every refusal with no time; an interruption; the
//! marks; a replay; and a profession's craft read for its own skill alone.

use super::*;
use crate::ability::Ability;
use crate::action::parse_actions;
use crate::content::{ContentFiles, Cutting, ObjectId};
use crate::item_parts::ItemPart;
use crate::magic::{Affix, Magic, Rarity};
use crate::object::Stack;
use crate::replay::Replay;
use crate::skills::{Effect, SkillId, Trained};
use crate::stats::{PerStat, Stats};
use std::sync::LazyLock;

const ROOM: &str = "\
###########
#.........#
#.@s......#
#.........#
#.......k.#
###########
";

/// The built-in content with [`ROOM`] twice: with no one in it, and with a kobold.
static FIXTURE: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    for (name, k) in [
        ("test_alone", "\"floor\""),
        (
            "test_watched",
            "{ terrain = \"floor\", creature = \"kobold\" }",
        ),
    ] {
        files.insert(
            &format!("levels/{name}.toml"),
            &format!(
                "name = \"Test\"\nambient = [120, 120, 120, 0]\nmap = '''\n{ROOM}'''\n\
                 [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\"k\" = {k}\n\
                 \"s\" = {{ terrain = \"floor\", objects = [\"long_sword\", \"quarterstaff\"] }}\n"
            ),
        );
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!(
                "name = \"T\"\nstart_level = \"{name}\"\n\
                 character = {{ race = \"human\", gender = \"female\", class = \"commoner\" }}\n"
            ),
        );
    }
    Arc::new(Content::load(&files).expect("fixture content is valid"))
});

/// A hall with no one in it, or with a kobold in sight. The player is a human commoner carrying
/// nothing and trained in no profession, every score 10 (a modifier of 0), `seed` their fate.
fn world(kobold: bool, seed: u64) -> World {
    let scenario = if kobold { "test_watched" } else { "test_alone" };
    let mut w = World::new(FIXTURE.clone(), scenario, Some(seed)).expect("world");
    let me = w.player_mut();
    me.hp_base = 1000;
    me.hp = 1000;
    me.inventory.clear();
    me.equipment = Default::default();
    me.stats = Stats::new(PerStat::splat(10), PerStat::splat(10));
    for id in ["metalcraft", "woodcraft", "jewelcraft"] {
        train(&mut w, id, None);
    }
    w
}

fn skill(w: &World, id: &str) -> SkillId {
    w.content.skill_id(id).expect(id)
}

/// Sets the player's skill `id` to `value` with no marks, or leaves them without it.
fn train(w: &mut World, id: &str, value: Option<u16>) {
    let s = skill(w, id);
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(k, _)| *k != s).collect();
    if let Some(value) = value {
        all.push((s, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

fn marks(w: &World, id: &str) -> Option<u32> {
    let s = skill(w, id);
    w.state.player().skills.get(s).map(|t| t.marks)
}

fn object(w: &World, id: &str) -> ObjectId {
    w.content.object_id(id).expect(id)
}

fn cutting(w: &World, size: &str, cut: &str) -> Cutting {
    let gems = w.content.gems().expect("gems");
    Cutting {
        size: gems.size_id(size).expect(size),
        cut: gems.cut_id(cut).expect(cut),
    }
}

/// The owner's staff: a hardwood quarterstaff tipped with a medium fine ruby.
fn staff(w: &World) -> Stack {
    let c = &w.content;
    let tip = c.part_id("tip").expect("tip");
    let (hardwood, ruby) = (
        c.material_id("hardwood").expect("hardwood"),
        c.material_id("ruby").expect("ruby"),
    );
    let cut = cutting(w, "medium", "fine");
    Stack::one(object(w, "quarterstaff")).with_parts(vec![
        ItemPart::material(c, None, hardwood),
        ItemPart::gem(c, Some(tip), ruby, cut),
    ])
}

/// What `make` makes, in the pack's first place.
fn give(w: &mut World, make: impl FnOnce(&World) -> Stack) {
    let stack = make(w);
    w.player_mut().inventory.insert(0, stack);
}

fn texts(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.text.clone()).collect()
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

fn pack(w: &World) -> Vec<String> {
    (w.state.player().inventory.iter())
        .map(|s| s.describe(&w.content))
        .collect()
}

/// What came of salvaging the staff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Came {
    Both,
    WoodOnly,
    RubyOnly,
    Neither,
}

/// The staff salvaged with fate `seed`: what came of the two checks, and the world after.
fn salvage_staff(seed: u64) -> (Came, World) {
    let mut w = world(false, seed);
    give(&mut w, staff);
    w.apply(&Action::Salvage { item: 0 });
    let wood = !said(&w, "The hardwood cracks and is good for nothing.");
    let ruby = !said(&w, "The ruby chips.") && !said(&w, "The ruby shatters.");
    let came = match (wood, ruby) {
        (true, true) => Came::Both,
        (true, false) => Came::WoodOnly,
        (false, true) => Came::RubyOnly,
        (false, false) => Came::Neither,
    };
    (came, w)
}

#[test]
fn the_owners_staff_gives_hardwood_and_its_ruby_by_their_checks() {
    let mut seen = Vec::new();
    for seed in 0..300 {
        let (came, w) = salvage_staff(seed);
        let all = texts(&w);
        assert!(
            said(&w, "You take the ruby-tipped hardwood quarterstaff apart."),
            "{all:?}"
        );
        let stock = pack(&w);
        match came {
            Came::Both => assert_eq!(
                stock,
                ["5 hardwood billets", "a medium fine ruby"],
                "{all:?}"
            ),
            // A failed gem drops a cut (fine to fair, the plain cut names leave unsaid), or shatters
            // on a natural 2.
            Came::WoodOnly if said(&w, "The ruby shatters.") => {
                assert_eq!(stock, ["5 hardwood billets"])
            }
            Came::WoodOnly => assert_eq!(stock, ["5 hardwood billets", "a medium ruby"]),
            Came::RubyOnly => assert_eq!(stock, ["a medium fine ruby"]),
            Came::Neither if said(&w, "The ruby shatters.") => {
                assert!(stock.is_empty());
                assert!(said(&w, "Nothing is left worth keeping."));
            }
            Came::Neither => assert_eq!(stock, ["a medium ruby"]),
        }
        if !seen.contains(&came) {
            seen.push(came);
        }
    }
    for came in [Came::Both, Came::WoodOnly, Came::RubyOnly, Came::Neither] {
        assert!(seen.contains(&came), "{came:?} comes of some fate");
    }
}

#[test]
fn a_natural_two_shatters_the_gem() {
    let shattered = (0..600).any(|seed| {
        let (_, w) = salvage_staff(seed);
        said(&w, "The ruby shatters.")
    });
    assert!(shattered, "some fate rolls a natural 2 on the ruby");
}

#[test]
fn a_saved_gem_keeps_its_size_and_cut() {
    let w = (0..300)
        .map(salvage_staff)
        .find_map(|(came, w)| (came == Came::Both).then_some(w))
        .expect("some fate saves both");
    let gem = (w.state.player().inventory.iter())
        .find_map(|s| s.gem(&w.content))
        .expect("the ruby");
    assert_eq!(gem.1, cutting(&w, "medium", "fine"));
}

#[test]
fn a_failed_gem_drops_a_cut_then_a_size_then_is_lost() {
    let w = world(false, 0);
    let gems = w.content.gems().expect("gems");
    let c = |size, cut| cutting(&w, size, cut);
    assert_eq!(
        chipped(gems, c("medium", "fine")),
        Some(c("medium", "fair"))
    );
    assert_eq!(chipped(gems, c("large", "fair")), Some(c("large", "crude")));
    // Crude is the roughest true cut: a size goes instead.
    assert_eq!(
        chipped(gems, c("large", "crude")),
        Some(c("medium", "crude"))
    );
    // An uncut stone has no cut to lose.
    assert_eq!(chipped(gems, c("small", "uncut")), Some(c("tiny", "uncut")));
    // Nothing left to lose: it is gone.
    assert_eq!(chipped(gems, c("tiny", "crude")), None);
    assert_eq!(chipped(gems, c("tiny", "uncut")), None);
}

#[test]
fn ten_minutes_destroy_the_item_and_the_untrained_still_roll() {
    let mut w = world(false, 1);
    give(&mut w, staff);
    let start = w.state.time;
    w.apply(&Action::Salvage { item: 0 });
    assert_eq!(w.state.time.since(start).as_seconds(), 600);
    assert!(
        (w.state.player().inventory.iter()).all(|s| s.object != object(&w, "quarterstaff")),
        "the staff is gone"
    );
    // Untrained in every profession, both checks are still made (a saved material shows one).
    let saved = (0..300).any(|seed| salvage_staff(seed).0 == Came::Both);
    assert!(saved, "an untrained hand saves both now and then");
}

#[test]
fn one_of_a_stack_is_salvaged_and_the_rest_is_kept() {
    let mut w = world(false, 2);
    let mut daggers = Stack::one(object(&w, "dagger"));
    daggers.count = 3;
    give(&mut w, |_| daggers);
    w.apply(&Action::Salvage { item: 0 });
    let left = (w.state.player().inventory.iter())
        .find(|s| s.object == object(&w, "dagger"))
        .map(|s| s.count);
    assert_eq!(left, Some(2));
}

#[test]
fn every_refusal_takes_no_time() {
    // Nothing a profession works: a scroll.
    let mut w = world(false, 3);
    give(&mut w, |w| Stack::one(object(w, "scroll_of_blessing")));
    let before = w.state.time;
    w.apply(&Action::Salvage { item: 0 });
    assert!(said(&w, "There is nothing in it worth saving."));
    assert_eq!(w.state.time, before);
    // Coins are never melted down.
    let mut w = world(false, 3);
    let coin = w.content.shops().expect("shops").currency;
    give(&mut w, |_| Stack::new(coin, 40));
    w.apply(&Action::Salvage { item: 0 });
    assert!(said(&w, "There is nothing in it worth saving."));
    assert_eq!(w.state.time, before);
    // A spirit weapon is spirit-stuff.
    let mut w = world(false, 3);
    give(&mut w, |w| Stack::spirit(object(w, "long_sword")));
    w.apply(&Action::Salvage { item: 0 });
    assert!(said(&w, "There is nothing in it worth saving."));
    assert_eq!(w.state.time, before);
    // Worn or wielded.
    let mut w = world(false, 3);
    give(&mut w, |w| Stack::one(object(w, "dagger")));
    w.apply(&parse_actions("equip:main_hand:a").expect("action")[0]);
    let before = w.state.time;
    w.apply(&Action::Salvage { item: 0 });
    assert!(said(&w, "Take it off first."), "{:?}", texts(&w));
    assert_eq!(w.state.time, before);
    assert_eq!(w.state.player().inventory.len(), 1, "kept");
    // Protected: of Proof.
    let mut w = world(false, 3);
    let magic = Magic {
        rarity: Rarity::Magic,
        level: 10,
        affixes: vec![Affix {
            id: "of_proof".into(),
            tier: 1,
            value: 1,
        }],
        name: None,
        identified: false,
        carried: 0,
        unique: None,
    };
    give(&mut w, |w| {
        Stack::with_magic(object(w, "long_sword"), 1, magic)
    });
    let before = w.state.time;
    w.apply(&Action::Salvage { item: 0 });
    assert!(said(&w, "It will not come apart."));
    assert_eq!(w.state.time, before);
    // No such stack.
    w.apply(&Action::Salvage { item: 9 });
    assert!(said(&w, "You carry no such thing."));
    assert_eq!(w.state.time, before);
}

#[test]
fn unpaid_goods_are_refused() {
    let content = Arc::new(Content::builtin().expect("content"));
    let mut w = World::new(content, "courtyard", None).expect("courtyard");
    let walk = "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 move:S*5 \
                move:E*5 move:E move:NE pickup";
    for action in parse_actions(walk).expect("actions") {
        w.apply(&action);
    }
    let dagger = w.state.player().inventory.len() - 1;
    assert_eq!(
        w.state.player().inventory[dagger].object,
        object(&w, "dagger")
    );
    let before = w.state.time;
    w.apply(&Action::Salvage { item: dagger });
    assert!(
        said(&w, "You have not paid for a dagger."),
        "{:?}",
        texts(&w)
    );
    assert_eq!(w.state.time, before);
}

#[test]
fn an_interrupted_salvage_does_nothing_to_the_item() {
    let mut w = world(true, 4);
    give(&mut w, staff);
    let start = w.state.time;
    w.apply(&Action::Salvage { item: 0 });
    assert!(
        (w.messages().iter()).any(|m| m.text.ends_with("you leave off your work.")),
        "{:?}",
        texts(&w)
    );
    assert!(w.state.time.since(start).as_seconds() < 600);
    assert_eq!(pack(&w), ["a ruby-tipped hardwood quarterstaff"]);
}

#[test]
fn each_profession_rolled_is_marked_alone() {
    let mut w = world(false, 5);
    for id in ["metalcraft", "woodcraft", "jewelcraft"] {
        train(&mut w, id, Some(20));
    }
    give(&mut w, staff);
    w.apply(&Action::Salvage { item: 0 });
    assert_eq!(marks(&w, "woodcraft"), Some(1), "the hardwood's");
    assert_eq!(marks(&w, "jewelcraft"), Some(1), "the ruby's");
    assert_eq!(marks(&w, "metalcraft"), Some(0), "no metal in it");
}

#[test]
fn a_professions_craft_is_read_for_its_own_skill_alone() {
    let mut w = world(false, 6);
    train(&mut w, "metalcraft", Some(100));
    train(&mut w, "woodcraft", Some(100));
    let me = w.state.player();
    let c = &w.content;
    assert_eq!(
        me.skill_effect_of(c, skill(&w, "metalcraft"), Effect::Craft),
        8
    );
    assert_eq!(
        me.skill_effect_of(c, skill(&w, "woodcraft"), Effect::Craft),
        8
    );
    assert_eq!(
        me.skill_effect_of(c, skill(&w, "jewelcraft"), Effect::Craft),
        0
    );
    // Summed across skills, it would be twice that.
    assert_eq!(me.skill_effect(c, Effect::Craft), 16);
}

#[test]
fn everyone_has_salvage_among_the_skills() {
    let w = world(false, 7);
    let me = w.state.player();
    assert!(Ability::of(me, &w.content).contains(&Ability::Salvage));
    assert_eq!(Ability::Salvage.unavailable(me, &w.content), None);
    assert_eq!(
        Ability::Salvage.action_on(&w.content, 2),
        Some(Action::Salvage { item: 2 })
    );
}

#[test]
fn a_salvage_replays() {
    // As the scenario starts: the sword and the staff lie east of the player.
    let mut w = World::new(FIXTURE.clone(), "test_alone", Some(8)).expect("world");
    let mut replay = Replay::new(&w);
    let mut play = |w: &mut World, text: &str| {
        for action in parse_actions(text).expect("actions") {
            w.apply(&action);
            replay.push(&action, w);
        }
    };
    play(&mut w, "move:E pickup");
    let at = |w: &World, id: &str| {
        let i = (w.state.player().inventory.iter())
            .position(|s| s.object == object(w, id))
            .expect(id);
        crate::action::position_text(i)
    };
    let sword = at(&w, "long_sword");
    play(&mut w, &format!("salvage:{sword}"));
    let staff = at(&w, "quarterstaff");
    play(&mut w, &format!("salvage:{staff} wait"));
    assert!(
        said(&w, "You take the long sword apart."),
        "{:?}",
        texts(&w)
    );
    assert!(
        said(&w, "You take the quarterstaff apart."),
        "{:?}",
        texts(&w)
    );
    let (verified, divergence) = replay.play(FIXTURE.clone()).expect("replays");
    assert_eq!(divergence, None);
    assert_eq!(verified.state_hash(), w.state_hash());
}

/// Bread ahead of the sword rots away while it is taken apart (#1564): the sword comes apart
/// all the same, and the quarterstaff that moves up to the sword's place is untouched.
#[test]
fn a_stack_gone_ahead_of_the_work_never_turns_it_on_the_next() {
    let mut w = world(false, 3);
    let (sword, staff) = (object(&w, "long_sword"), object(&w, "quarterstaff"));
    let loaf = super::work_on::rotting_loaf(&w);
    let pack = &mut w.player_mut().inventory;
    pack.extend([loaf, Stack::one(sword), Stack::one(staff)]);
    w.apply(&Action::Salvage { item: 1 });
    assert!(
        said(&w, "Your loaf of bread has rotted away."),
        "{:?}",
        texts(&w)
    );
    assert!(
        said(&w, "You take the long sword apart."),
        "{:?}",
        texts(&w)
    );
    let pack = &w.state.player().inventory;
    assert!(pack.iter().all(|s| s.object != sword), "{:?}", texts(&w));
    assert_eq!(pack[0], Stack::one(staff), "untouched, first now");
}

//! Vitals (spec §8 "Surprise and sneak attacks"; `Content::has_vitals`, `ambush.rs`): what has
//! none is immune to criticals (no confirmation drawn) and to sneak attacks, but is surprised
//! like anyone; Anatomist finds them in bones and constructs; without the surprise rules
//! everything has them.

use super::*;
use crate::action::{AttackKind, Event};
use crate::combat::Strike;
use crate::content::ContentFiles;
use crate::skills::Trained;

/// A creature that says it has no vitals (`vitals = false`), none of the rules' kinds.
const STONE: &str = r##"
[[creature]]
id = "vt_stone"
name = "stone man"
glyph = "o"
fg = "#808080"
weight = 60000
faction = "vermin"
speed = 100
base_hp = 1000
vitals = false
"##;

/// A lit hall: the player, then a goblin, a skeleton, a ghost, a draugr, a mud golem, and a
/// stone man in a row, each a cell apart, nobody acting.
const HALL: &str = "\
################
#@.g.s.G.d.m.o.#
#..............#
################
";

/// The built-in content with [`HALL`], with the surprise rules or (`rules` false) without.
pub(super) fn world(rules: bool) -> World {
    let mut files = ContentFiles::builtin();
    if !rules {
        files.remove("data/surprise.toml");
    }
    files.insert("data/creatures/zz_vt_test.toml", STONE);
    files.insert(
        "levels/test_vitals.toml",
        &format!(
            "name = \"Test\"\nambient = [255, 255, 255, 0]\nmap = '''\n{HALL}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"g\" = {{ terrain = \"floor\", creature = \"goblin\" }}\n\
             \"s\" = {{ terrain = \"floor\", creature = \"skeleton\" }}\n\
             \"G\" = {{ terrain = \"floor\", creature = \"ghost\" }}\n\
             \"d\" = {{ terrain = \"floor\", creature = \"draugr\" }}\n\
             \"m\" = {{ terrain = \"floor\", creature = \"mud_golem\" }}\n\
             \"o\" = {{ terrain = \"floor\", creature = \"vt_stone\" }}\n"
        ),
    );
    files.insert(
        "scenarios/test_vitals.toml",
        "name = \"T\"\nstart_level = \"test_vitals\"\nhp = 1000\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    let mut w = World::new(content, "test_vitals", None).expect("world");
    let player = w.state.player;
    for actor in w.state.actors.values_mut() {
        if actor.id != player {
            actor.energy = -1_000_000_000;
            actor.hp = 10_000;
        }
    }
    w
}

/// The creature of type `id`.
pub(super) fn one(w: &World, id: &str) -> ActorId {
    let creature = w.content.creature_id(id).expect("a creature type");
    (w.state.actors.values())
        .find(|a| a.kind == ActorKind::Creature && a.creature == creature)
        .expect("one in the hall")
        .id
}

/// Puts `id` to sleep: it has seen no one.
pub(super) fn lull(w: &mut World, id: ActorId) {
    let asleep = w.content.sleep_status().expect("asleep");
    let def = w.content.status(asleep).clone();
    let actor = w.state.actors.get_mut(&id).expect("actor");
    actor.effects.add(asleep, &def, 1, None);
}

/// The player trained in `skill` to `value`.
pub(super) fn train(w: &mut World, skill: &str, value: u16) {
    let id = w.content().skill_id(skill).expect("skill");
    let skills = &mut w.player_mut().skills;
    let mut all: Vec<(crate::skills::SkillId, Trained)> =
        skills.iter().filter(|(s, _)| *s != id).collect();
    all.push((id, Trained { value, marks: 0 }));
    *skills = all.into_iter().collect();
}

/// The player's blow at `target`, as sure to hit as it can be and a critical whenever it hits
/// (`sure`) or threatening on every hit (not `sure`): how it struck.
fn blow(w: &mut World, target: ActorId, sure: bool) -> Strike {
    let player = w.state.player;
    let mut profile = w.state.player().melee(w.content());
    profile.to_hit += 40;
    profile.sure_critical = sure;
    profile.threat = 2;
    let mut events = Vec::new();
    w.attack_with(
        player,
        target,
        AttackKind::Melee,
        Some(profile),
        &mut events,
    );
    w.state.actors.get_mut(&target).expect("still there").hp = 10_000;
    (events.iter())
        .find_map(|e| match e {
            Event::Attacked { strike, .. } => *strike,
            _ => None,
        })
        .expect("rolled")
}

#[test]
fn what_has_no_vitals_by_rule_and_by_data() {
    let w = world(true);
    let content = w.content();
    for (id, vitals) in [
        ("goblin", true),
        ("draugr", true),
        ("skeleton", false),
        ("ghost", false),
        ("mud_golem", false),
        ("vt_stone", false),
        ("air_elemental", false),
    ] {
        let creature = content.creature_id(id).expect(id);
        assert_eq!(content.has_vitals(creature), vitals, "{id}");
    }
    // Animated objects have none.
    let animated = (content.creature_defs().iter())
        .find(|c| c.animated.is_some())
        .and_then(|c| content.creature_id(&c.id))
        .expect("an animated object");
    assert!(!content.has_vitals(animated));
    // The sides the blows strike say so.
    for (id, immune) in [("goblin", false), ("skeleton", true), ("draugr", false)] {
        assert_eq!(w.target_of(one(&w, id)).no_criticals, immune, "{id}");
    }
    // Without the surprise rules everything has vitals.
    let plain = world(false);
    for id in ["skeleton", "ghost", "mud_golem", "vt_stone"] {
        assert!(!plain.target_of(one(&plain, id)).no_criticals, "{id}");
    }
}

#[test]
fn no_critical_lands_on_a_thing_without_vitals() {
    let mut w = world(true);
    let (goblin, skeleton) = (one(&w, "goblin"), one(&w, "skeleton"));
    for sure in [true, false] {
        let mut criticals = 0;
        for _ in 0..40 {
            match blow(&mut w, skeleton, sure) {
                Strike::Hit { critical } => assert!(!critical, "a plain hit"),
                other => assert!(!other.is_hit()),
            }
            if blow(&mut w, goblin, sure) == (Strike::Hit { critical: true }) {
                criticals += 1;
            }
        }
        assert!(criticals > 20, "the goblin has vitals: {criticals}");
    }
}

#[test]
fn no_sneak_dice_land_on_a_thing_without_vitals_though_it_is_surprised() {
    let mut w = world(true);
    let player = w.state.player;
    train(&mut w, "sneak_attack", 100);
    for id in ["goblin", "skeleton", "ghost", "draugr", "mud_golem"] {
        let them = one(&w, id);
        lull(&mut w, them);
    }
    for (id, sneaks) in [
        ("goblin", true),
        ("draugr", true),
        ("skeleton", false),
        ("ghost", false),
        ("mud_golem", false),
    ] {
        assert_eq!(w.sneaking(player, one(&w, id), false), sneaks, "{id}");
    }
    // The skeleton is caught unawares all the same: a sleeper wakes Surprised.
    let skeleton = one(&w, "skeleton");
    let mut events = Vec::new();
    w.startle(skeleton, Some(player), &mut events);
    assert!(w.surprised(skeleton));
    assert!(!w.sneaking(player, skeleton, false));
}

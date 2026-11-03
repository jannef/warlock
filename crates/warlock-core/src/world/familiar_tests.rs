//! The witch's familiar (`familiar.rs`; spec §8 "Allies"): the starting beast by race and the
//! rares, the name and the arrival, the touch (her hex, her the giver, her other hexes kept
//! going), the harrying, the shared kept-beast rules (growth, always along, the words, the
//! dumps), a companion beside it, and saves.

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::character::CharacterSpec;
use crate::content::{Content, ContentFiles, Stance};
use crate::perception::Perception;
use crate::world::stance_between;
use std::sync::{Arc, LazyLock};

static BUILTIN: LazyLock<Arc<Content>> =
    LazyLock::new(|| Arc::new(Content::builtin().expect("builtin content")));

/// The builtin content with a lit hall of two kobolds, stairs down to a quiet floor.
static HALL: LazyLock<Arc<Content>> = LazyLock::new(|| {
    let mut files = ContentFiles::builtin();
    files.insert("levels/famil_hall.toml", HALL_LEVEL);
    files.insert("levels/famil_below.toml", BELOW_LEVEL);
    files.insert(
        "scenarios/famil_hall.toml",
        "name = \"Hall\"\nstart_level = \"famil_hall\"\nhp = 100\n",
    );
    Arc::new(Content::load(&files).expect("test content is valid"))
});

const HALL_LEVEL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
######################
#@..k...k...........>#
#....................#
#....................#
######################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
">" = { terrain = "stairs_down", to = "famil_below" }
"##;

const BELOW_LEVEL: &str = r##"
name = "Below"
ambient = [255, 255, 255, 0]
map = '''
##########
#<.......#
#........#
##########
'''
[legend]
"<" = { terrain = "stairs_up", to = "famil_hall" }
"##;

fn spec(text: &str) -> CharacterSpec {
    text.parse().expect("a character")
}

/// A kitted run of `spec` on the Ascmearc's start, as New run makes one.
fn kitted(spec: &str, seed: u64) -> World {
    let picks = vec!["keen_ears".into(), "lean".into(), "healthy".into()];
    let spec = self::spec(spec).with_talents(picks);
    World::start_run(
        BUILTIN.clone(),
        "ascmearc",
        Some(seed),
        Some(&spec),
        None,
        true,
    )
    .expect("world")
}

/// A human witch in the hall (no kit, no story).
fn hall(seed: u64) -> World {
    World::with_character(
        HALL.clone(),
        "famil_hall",
        Some(seed),
        Some(&spec("human,female,witch")),
    )
    .expect("world")
}

fn run(world: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        world.apply(&action);
    }
}

fn said(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.display()).collect()
}

fn familiar_of(w: &World) -> (&Actor, &CompanionData) {
    let id = w.familiar().expect("a familiar");
    let actor = &w.state.actors[&id];
    (actor, actor.familiar().expect("familiar data"))
}

/// The kobolds of the hall, nearest the player first.
fn kobolds(w: &World) -> Vec<ActorId> {
    let kobold = w.content().creature_id("kobold").unwrap();
    let mut out: Vec<&Actor> = (w.state.actors.values())
        .filter(|a| a.creature == kobold && a.level == w.state.current_level)
        .collect();
    out.sort_by_key(|a| a.pos.x);
    out.into_iter().map(|a| a.id).collect()
}

fn status(w: &World, id: &str) -> crate::content::StatusId {
    w.content().status_id(id).expect(id)
}

// ---- the draw ------------------------------------------------------------------------------

#[test]
fn every_race_draws_from_its_list_or_the_rares_at_the_rare_odds() {
    let content = &*BUILTIN;
    let rules = content.familiars().expect("familiars.toml");
    let mut draws = 0u32;
    let mut rares = 0u32;
    for race in &content.races().list {
        let list = rules.list_of(&race.id).expect("every race has a list");
        let spec = spec(&format!("{},female,witch", race.id));
        for seed in 0..400 {
            let beast = super::draw(content, seed, &spec).expect("a beast");
            assert!(rules.arrives(beast).is_some(), "{}", race.id);
            if rules.rare.contains(&beast) && !list.contains(&beast) {
                rares += 1;
            } else {
                assert!(list.contains(&beast), "{}", race.id);
            }
            draws += 1;
        }
    }
    let expected = f64::from(draws) * 0.05;
    assert!(
        (f64::from(rares) - expected).abs() < expected / 4.0,
        "{rares} rares of {draws}, about {expected} expected"
    );
    let witch = spec("human,female,witch");
    assert_eq!(
        super::draw(content, 7, &witch),
        super::draw(content, 7, &witch)
    );
    assert!(
        rules
            .names
            .contains(&super::name_for(content, 7, ActorId(1)))
    );
}

// ---- the start -----------------------------------------------------------------------------

/// The id of the line the companion chapter told.
fn companion_line(w: &World) -> String {
    let story = &w.state.character.as_ref().unwrap().story;
    (story.iter())
        .find(|t| t.chapter == "companion")
        .expect("the companion chapter")
        .line
        .clone()
}

#[test]
fn a_familiar_is_told_its_own_line_and_a_rare_the_plain_one() {
    let content = &*BUILTIN;
    let chapter = (content
        .background()
        .unwrap()
        .chapter("ascmearc", "companion"))
    .unwrap();
    let (mut own, mut plain) = (0, 0);
    for seed in 0..120 {
        let w = kitted("human,female,witch", seed);
        let (beast, _) = familiar_of(&w);
        let line = chapter.line(&companion_line(&w)).expect("a line");
        if content.familiars().unwrap().rare.contains(&beast.creature) {
            assert_eq!(line.id, "plain_companion");
            plain += 1;
        } else {
            assert!(
                line.only && line.when.familiar == [beast.creature],
                "{}",
                line.id
            );
            assert!(line.when.companion.is_empty());
            own += 1;
        }
    }
    assert!(own > 100 && plain > 0, "{own} {plain}");
}

#[test]
fn a_witch_begins_with_a_named_familiar_and_a_warrior_without() {
    let content = &*BUILTIN;
    let rules = content.familiars().unwrap();
    let w = kitted("human,female,witch", 7);
    let (beast, data) = familiar_of(&w);
    let kind = content.creature(beast.creature);
    assert!(rules.names.contains(&data.name), "{}", data.name);
    assert_eq!(data.grade, kind.level.max(1));
    assert_eq!(beast.hp, beast.max_hp(content), "whole");
    assert!(beast.ally.as_ref().unwrap().until.is_none(), "for good");
    assert_eq!(beast.faction(content), w.state.player().faction(content));
    assert!(w.companion().is_none(), "a familiar is no companion");
    let line = format!(
        "{}, your {}, {}.",
        data.name,
        kind.name,
        rules.arrives(beast.creature).unwrap()
    );
    assert!(said(&w).contains(&line), "{:?}", said(&w));
    // The companion chapter is told to a kept beast, with the familiar's own line.
    assert_eq!(companion_line(&w), format!("familiar_{}", kind.id));
    // The dumps.
    let layers = crate::dump::parse_layers("familiar").unwrap();
    let text = crate::dump::ascii(&w, &layers);
    let head = format!(
        "{}, {} {} (grade {})",
        data.name,
        crate::content::article(&kind.name),
        kind.name,
        data.grade
    );
    assert!(text.contains(&head), "{text}");
    let json = crate::dump::json(&w, &[], false);
    assert_eq!(json["familiar"]["name"], data.name.as_str());
    assert_eq!(json["familiar"]["creature"], kind.id.as_str());
    assert_eq!(json["companion"], serde_json::Value::Null);
    assert_eq!(json["allies"][0]["source"], "familiar");
    let actors = crate::dump::parse_layers("actors").unwrap();
    let text = crate::dump::ascii(&w, &actors);
    // On the overworld at the start: it rides in the retinue.
    assert!(
        text.contains(&format!("(familiar; follow; retinue; named {}", data.name)),
        "{text}"
    );
    // A warrior of the same seed: none.
    let w = kitted("human,female,warrior", 7);
    assert!(w.familiar().is_none());
    let text = crate::dump::ascii(&w, &layers);
    assert!(text.contains("no familiar"), "{text}");
    let story = &w.state.character.as_ref().unwrap().story;
    assert!(story.iter().all(|t| t.chapter != "companion"));
}

#[test]
fn a_restored_run_makes_no_second_familiar() {
    let w = hall(3);
    let again = World::from_state(HALL.clone(), w.state().clone()).expect("restored");
    assert_eq!(again.allies_of(again.state.player).len(), 1);
    assert_eq!(again.state_hash(), w.state_hash());
}

// ---- the touch -----------------------------------------------------------------------------

#[test]
fn the_touch_lays_her_evil_eye_with_her_as_the_giver_and_keeps_her_hexes_going() {
    let mut w = hall(3);
    let content = w.content().clone();
    let player = w.state.player;
    let familiar = w.familiar().unwrap();
    let kobold = kobolds(&w)[0];
    let hexed = status(&w, "ill_wished");
    let weakened = status(&w, "weakened");
    let evil_eye = content.spell(content.spell_id("evil_eye").unwrap());
    let degrees = w.state.player().spell_degrees(&content, evil_eye).unwrap();
    let mut events = Vec::new();
    // Her Hex of weakness, worn a while.
    w.afflict(kobold, weakened, 2, Some(player), &mut events);
    w.state
        .actors
        .get_mut(&kobold)
        .unwrap()
        .effects
        .get_mut(weakened)
        .unwrap()
        .wear = 30;
    w.familiar_touch(familiar, kobold, AttackKind::Melee, &mut events);
    let effects = &w.state.actors[&kobold].effects;
    let eye = effects.get(hexed).expect("hexed");
    assert_eq!(eye.by, Some(player), "her hex, not the cat's");
    assert_eq!(eye.severity, degrees.min(content.status(hexed).max));
    let weak = effects.get(weakened).expect("still weakened");
    assert_eq!(
        (weak.severity, weak.wear),
        (2, 0),
        "refreshed at its own degree"
    );
    // Cackle sees the familiar's hex as hers.
    assert!(
        (w.own_hexes_in_sight().iter()).any(|&(id, s, _)| id == kobold && s == hexed),
        "{:?}",
        w.own_hexes_in_sight()
    );
    // A missile, or a blow at the master, lays nothing.
    let other = kobolds(&w)[1];
    w.familiar_touch(familiar, other, AttackKind::Thrown, &mut events);
    w.familiar_touch(familiar, player, AttackKind::Melee, &mut events);
    assert!(!w.state.actors[&other].effects.has(hexed));
    assert!(!w.state.player().effects.has(hexed));
}

#[test]
fn a_summoned_cats_blow_hexes_no_one() {
    let mut w = hall(3);
    run(&mut w, "ally:cat");
    let cat = content_id(&w, "cat");
    let summon = (w.allies_of(w.state.player).into_iter())
        .find(|id| {
            let a = &w.state.actors[id];
            a.creature == cat && a.familiar().is_none()
        })
        .expect("the summoned cat");
    let kobold = kobolds(&w)[0];
    w.familiar_touch(summon, kobold, AttackKind::Melee, &mut Vec::new());
    assert!(w.state.actors[&kobold].effects.iter().next().is_none());
}

fn content_id(w: &World, id: &str) -> crate::content::CreatureId {
    w.content().creature_id(id).expect(id)
}

#[test]
fn the_familiars_scratches_hex_what_it_fights() {
    let mut w = hall(5);
    let hexed = status(&w, "ill_wished");
    let player = w.state.player;
    let mut seen = false;
    for _ in 0..60 {
        run(&mut w, "wait");
        seen |= (w.state.actors.values())
            .any(|a| a.effects.get(hexed).is_some_and(|e| e.by == Some(player)));
        if seen {
            break;
        }
    }
    assert!(seen, "{:?}", said(&w));
}

// ---- the harrying --------------------------------------------------------------------------

fn hostile(s: &WorldState, c: &Content, _: &Actor, a: &Actor) -> bool {
    stance_between(s, c, s.player(), a) == Stance::Hostile
}

#[test]
fn the_familiar_goes_for_what_she_hexed_before_what_is_nearer() {
    let mut w = hall(3);
    let player = w.state.player;
    let familiar = w.familiar().unwrap();
    let [near, far] = kobolds(&w)[..] else {
        panic!("two kobolds");
    };
    // Where whatever kind it is sees both.
    let at = crate::geom::Pos::new(3, 2);
    w.state.actors.get_mut(&familiar).unwrap().pos = at;
    assert_eq!(w.quarry(familiar, &hostile).map(|q| q.0), Some(near));
    let hexed = status(&w, "ill_wished");
    w.afflict(far, hexed, 1, Some(player), &mut Vec::new());
    assert_eq!(
        w.quarry(familiar, &hostile).map(|q| q.0),
        Some(far),
        "her hexed first"
    );
    // Another's hex is not hers.
    let mut w = hall(3);
    let familiar = w.familiar().unwrap();
    w.state.actors.get_mut(&familiar).unwrap().pos = at;
    w.afflict(far, hexed, 1, None, &mut Vec::new());
    assert_eq!(w.quarry(familiar, &hostile).map(|q| q.0), Some(near));
}

// ---- the kept beast's rules ----------------------------------------------------------------

#[test]
fn the_familiar_grows_and_always_comes_along() {
    let content = &*BUILTIN;
    let mut w = kitted("human,female,witch", 7);
    run(&mut w, "xp:2000 levelup:auto");
    assert_eq!(w.player_level(), 4);
    let (beast, data) = familiar_of(&w);
    assert_eq!(data.grade, 4.max(content.creature(beast.creature).level));
    assert!(said(&w).contains(&format!("{} grows stronger.", data.name)));
    // Held fast across the hall, it comes down the stairs all the same.
    let mut w = hall(3);
    let familiar = w.familiar().unwrap();
    let webbed = status(&w, "webbed");
    w.afflict(familiar, webbed, 2, None, &mut Vec::new());
    w.state.actors.get_mut(&familiar).unwrap().pos = crate::geom::Pos::new(10, 3);
    for k in kobolds(&w) {
        w.state.actors.remove(&k);
    }
    w.state.actors.get_mut(&w.state.player).unwrap().pos = crate::geom::Pos::new(19, 1);
    run(&mut w, "move:E descend");
    assert_eq!(w.state.current_level, "famil_below");
    let (beast, data) = familiar_of(&w);
    assert_eq!(beast.level, "famil_below");
    assert!(said(&w).contains(&format!("{} comes with you.", data.name)));
}

#[test]
fn a_witch_may_keep_a_familiar_and_a_companion_at_once() {
    let mut w = hall(3);
    run(&mut w, "ally:hound");
    let hound = content_id(&w, "hound");
    let id = (w.allies_of(w.state.player).into_iter())
        .find(|id| w.state.actors[id].creature == hound)
        .unwrap();
    w.state
        .actors
        .get_mut(&id)
        .unwrap()
        .ally
        .as_mut()
        .unwrap()
        .source = AllySource::Companion(CompanionData {
        name: "Snel".into(),
        grade: 1,
        scent: None,
    });
    assert!(w.companion().is_some() && w.familiar().is_some());
    *w.perception = Perception::compute(&w.state, &w.content, &mut w.lights);
    let view = w.perception();
    assert_eq!(view.kept().count(), 2);
    assert!(view.companion().is_some() && view.familiar().is_some());
    // Both beside the player: tending asks which way.
    let familiar = w.familiar().unwrap();
    let me = w.state.player().pos;
    w.state.actors.get_mut(&familiar).unwrap().pos = me.step(crate::geom::Dir::S);
    w.state.actors.get_mut(&id).unwrap().pos = me.step(crate::geom::Dir::SE);
    *w.perception = Perception::compute(&w.state, &w.content, &mut w.lights);
    assert_eq!(w.perception().kept_beside().len(), 2);
}

#[test]
fn a_save_round_trip_keeps_the_familiar() {
    let mut w = kitted("human,female,witch", 3);
    run(&mut w, "wait*3 xp:1000 levelup:auto wait");
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, BUILTIN.clone()).expect("loads");
    assert_eq!(loaded.state_hash(), w.state_hash());
    let (before, data) = familiar_of(&w);
    let (after, loaded_data) = familiar_of(&loaded);
    assert_eq!(before.id, after.id);
    assert_eq!(data, loaded_data);
}

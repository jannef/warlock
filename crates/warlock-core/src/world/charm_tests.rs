//! Charmed allies (`world/charm.rs`; the spell types design §7 entry 34): Beguile on the shipped
//! content, a kobold turned on its band, its end where it stands, its end as the player leaves,
//! the refusals, the kill credit, the dumps, saves, and a replay.

use super::*;
use crate::action::parse_actions;
use crate::content::Stance;

/// A lit hall: the player at the west end, a kobold a few cells east, its fellow farther on, a
/// skeleton in the corner, and stairs down beside the player's start.
const HALL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
####################
#@>..k.......K.....#
#..................#
#..................#
#.................s#
####################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"k" = { terrain = "floor", creature = "kobold" }
"K" = { terrain = "floor", creature = "kobold" }
"s" = { terrain = "floor", creature = "skeleton" }
">" = { terrain = "stairs_down", to = "below" }
"##;

const BELOW: &str = r##"
name = "Below"
ambient = [255, 255, 255, 0]
map = '''
##########
#<.......#
#........#
#........#
##########
'''
[legend]
"<" = { terrain = "stairs_up", to = "hall" }
"##;

fn content() -> Arc<Content> {
    let mut files = super::caster_tests::files();
    files.insert("levels/hall.toml", HALL);
    files.insert("levels/below.toml", BELOW);
    files.insert(
        "scenarios/hall.toml",
        "name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n",
    );
    Arc::new(Content::load(&files).expect("valid content"))
}

/// The witch in the hall, knowing Beguile, with mana to spare.
fn world() -> World {
    let spec = "human,female,witch".parse().expect("a character");
    let mut w = World::with_character(content(), "hall", Some(7), Some(&spec)).expect("world");
    let beguile = w.content().spell_id("beguile").expect("the spell");
    w.player_mut().spells.insert(beguile, 1);
    w.player_mut().mana_base = 400;
    w.player_mut().mana = 400;
    see(&mut w);
    w
}

fn see(w: &mut World) {
    *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn at(w: &World, x: i32, y: i32) -> Option<ActorId> {
    w.state.actor_at("hall", Pos::new(x, y)).map(|a| a.id)
}

fn last_said(w: &World) -> String {
    let line = (w.messages().iter().last()).map_or_else(String::new, |m| m.display());
    line.split(" (×").next().unwrap_or_default().to_string()
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.display().starts_with(text))
}

/// The near kobold, charmed.
fn charmed() -> (World, ActorId, ActorId) {
    let mut w = world();
    let near = at(&w, 5, 1).expect("the near kobold");
    let far = at(&w, 13, 1).expect("the far kobold");
    run(&mut w, "use:beguile:5,1");
    (w, near, far)
}

#[test]
fn a_charmed_kobold_is_the_players_ally_against_its_band() {
    let (w, near, far) = charmed();
    assert_eq!(last_said(&w), "The kobold takes you for a friend.");
    let kobold = &w.state.actors[&near];
    let ally = kobold.ally.as_ref().expect("an ally");
    assert_eq!(ally.master, w.state.player);
    assert_eq!(ally.source.id(), "charmed");
    let AllySource::Charmed { faction, .. } = ally.source else {
        panic!("charmed");
    };
    assert_eq!(faction, None, "it had no override to put back");
    // Five minutes at the witch's Presence, or more by its ladder, from the cast (a turn ago).
    let left = ally
        .until
        .expect("a clock")
        .since(w.state.time)
        .as_seconds();
    assert!(left >= 5 * 60 - 10, "{left}");
    assert_eq!(w.stance(w.state.player, near), Stance::Neutral);
    assert_eq!(w.stance(far, near), Stance::Hostile, "its band turns on it");
    assert!(w.state.player().mana < 400, "paid");
}

#[test]
fn it_fights_its_band_and_the_kill_is_the_players() {
    let (mut w, near, far) = charmed();
    // The witch's familiar goes home: the kill is the kobold's alone.
    for id in w.allies_of(w.state.player) {
        if id != near {
            w.state.actors.remove(&id);
        }
    }
    // The far one is weak and the near one strong: the near one's blow is the end of it.
    w.state.actors.get_mut(&far).expect("there").hp = 1;
    w.state.actors.get_mut(&near).expect("there").hp_base = 500;
    w.state.actors.get_mut(&near).expect("there").hp = 500;
    let before = w.state.experience;
    for _ in 0..80 {
        if !w.state.actors.contains_key(&far) {
            break;
        }
        run(&mut w, "wait");
    }
    assert!(
        !w.state.actors.contains_key(&far),
        "the kobold's own killed it"
    );
    assert!(w.state.actors.contains_key(&near));
    assert!(w.state.experience > before, "the kill paid the witch");
    assert_eq!(w.state.quests.killed("kobold"), 1);
}

#[test]
fn the_charm_ends_and_it_is_hostile_where_it_stands() {
    let (mut w, near, _) = charmed();
    let now = w.state.time;
    let ally = (w.state.actors.get_mut(&near)).and_then(|a| a.ally.as_mut());
    ally.expect("an ally").until = Some(now);
    let pos = w.state.actors[&near].pos;
    run(&mut w, "order:stay wait");
    let kobold = &w.state.actors[&near];
    assert!(kobold.ally.is_none());
    assert_eq!(
        kobold.faction_override, None,
        "its faction is its own again"
    );
    assert_eq!(kobold.pos, pos, "where it stood");
    assert_eq!(w.stance(w.state.player, near), Stance::Hostile);
    assert!(said(&w, "The kobold shakes off the charm."));
}

#[test]
fn what_it_summoned_while_charmed_goes_back_with_it() {
    let (mut w, near, far) = charmed();
    // The far kobold stands for one the charmed one called: on the player's side with it.
    let mine = w.state.player().faction(w.content());
    let called = w.state.actors.get_mut(&far).expect("there");
    called.summoner = Some(near);
    called.faction_override = mine;
    let now = w.state.time;
    let ally = (w.state.actors.get_mut(&near)).and_then(|a| a.ally.as_mut());
    ally.expect("an ally").until = Some(now);
    run(&mut w, "wait");
    assert_eq!(w.state.actors[&far].faction_override, None);
    assert_eq!(w.stance(w.state.player, far), Stance::Hostile);
}

#[test]
fn the_charm_ends_when_the_player_leaves_and_it_stays_behind() {
    let (mut w, near, _) = charmed();
    // The charmed kobold beside the player on the stairs: neither ally nor stair follower comes.
    w.state.actors.get_mut(&near).expect("there").pos = Pos::new(3, 1);
    w.state.actors.get_mut(&w.state.player).expect("there").pos = Pos::new(2, 1);
    see(&mut w);
    run(&mut w, "descend");
    assert_eq!(w.state.current_level, "below");
    let kobold = &w.state.actors[&near];
    assert_eq!(kobold.level, "hall", "left behind");
    assert_eq!(kobold.pos, Pos::new(3, 1));
    assert!(kobold.ally.is_none(), "the charm is over");
    assert_eq!(kobold.faction_override, None);
    // The witch's familiar came along; the kobold is no one's.
    assert!(!w.allies_of(w.state.player).contains(&near));
    assert!(!said(&w, "Your kobold is left behind."));
    assert!(!said(&w, "The kobold follows you"));
}

#[test]
fn a_charm_keeps_a_faction_its_level_gave() {
    let mut w = world();
    let near = at(&w, 5, 1).expect("the near kobold");
    let peaceful = w.content().factions().id("berserk").expect("a faction");
    w.state
        .actors
        .get_mut(&near)
        .expect("there")
        .faction_override = Some(peaceful);
    run(&mut w, "use:beguile:5,1");
    let ally = w.state.actors[&near].ally.clone().expect("an ally");
    assert_eq!(
        ally.source,
        AllySource::Charmed {
            spell: w.content().spell_id("beguile").expect("the spell"),
            faction: Some(peaceful),
        }
    );
    let now = w.state.time;
    let a = w.state.actors.get_mut(&near).expect("there");
    a.ally.as_mut().expect("an ally").until = Some(now);
    run(&mut w, "wait");
    assert_eq!(w.state.actors[&near].faction_override, Some(peaceful));
}

#[test]
fn it_is_refused_on_the_mindless_a_named_one_and_an_ally() {
    let mut w = world();
    let turn = w.state.turn;
    let mana = w.state.player().mana;
    // The skeleton: undead, no mind.
    let skeleton = at(&w, 18, 4).expect("the skeleton");
    w.state.actors.get_mut(&skeleton).expect("there").pos = Pos::new(6, 2);
    see(&mut w);
    run(&mut w, "use:beguile:6,2");
    assert_eq!(last_said(&w), "The skeleton has no mind to charm.");
    // A kobold with a name and a trade.
    let near = at(&w, 5, 1).expect("the near kobold");
    w.state.actors.get_mut(&near).expect("there").npc = Some("someone".into());
    run(&mut w, "use:beguile:5,1");
    assert!(
        last_said(&w).ends_with("is no one's fool."),
        "{}",
        last_said(&w)
    );
    w.state.actors.get_mut(&near).expect("there").npc = None;
    assert_eq!((w.state.turn, w.state.player().mana), (turn, mana));
    // One already at someone's side.
    run(&mut w, "use:beguile:5,1");
    let mana = w.state.player().mana;
    let turn = w.state.turn;
    run(&mut w, "use:beguile:5,1");
    assert_eq!(last_said(&w), "The kobold is already at someone's side.");
    assert_eq!((w.state.turn, w.state.player().mana), (turn, mana));
}

#[test]
fn the_dumps_say_it_is_charmed() {
    let (w, near, _) = charmed();
    let json = crate::dump::json(&w, &[], false);
    let allies = json["allies"].as_array().expect("allies");
    let kobold = (allies.iter()).find(|a| a["source"] == "charmed");
    assert_eq!(kobold.expect("the charmed one")["creature"], "kobold");
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Actors).join("\n");
    assert!(
        rows.contains(&format!("#{} ", near.0)) && rows.contains("(charmed, "),
        "{rows}"
    );
    // The friendly mark's source: a seen ally.
    let view = w.perception();
    let seen = (view.actors().iter()).find(|a| a.id == near);
    assert!(seen.expect("seen").ally);
}

#[test]
fn a_save_keeps_the_charm_and_a_replay_verifies() {
    let (w, near, _) = charmed();
    let state = w.state().clone();
    let text = serde_json::to_string(&state).expect("saves");
    let back: WorldState = serde_json::from_str(&text).expect("loads");
    assert_eq!(back.actors[&near].ally, state.actors[&near].ally);
    let actions = "use:beguile:5,1 wait*3 move:E wait*60 descend";
    let spec = "human,female,witch".parse().expect("a character");
    let mut a = World::with_character(content(), "hall", Some(7), Some(&spec)).expect("world");
    let beguile = a.content().spell_id("beguile").expect("the spell");
    a.player_mut().spells.insert(beguile, 1);
    a.player_mut().mana_base = 400;
    a.player_mut().mana = 400;
    let mut b = World::from_state(a.content().clone(), a.state().clone()).expect("restores");
    for action in parse_actions(actions).expect("actions") {
        a.apply(&action);
        b.apply(&action);
        assert_eq!(a.state_hash(), b.state_hash(), "{action:?}");
    }
}

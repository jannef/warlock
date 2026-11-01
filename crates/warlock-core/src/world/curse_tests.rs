//! Cursed rooms (spec §7 "Special rooms", design §5; `world/curse.rs`) and Consecrate
//! (`world/consecrate.rs`): the dead killed inside rise two minutes later as undead of their
//! level (the nearest level when none has theirs), unless their corpse is gone; Old delver;
//! a whole cursed level; frozen floors catching up; Consecrate lifting a room or a level.

use super::*;
use crate::ability::{Ability, Unavailable};
use crate::action::parse_actions;
use crate::alignment::Score;
use crate::content::{ClassActive, ContentFiles, CreatureId, CreatureSpawn};
use crate::dump::{self, Layer};

/// A hall, 30 wide, with a cursed room over x 10 to 19. Lit, so the player sees all of it.
const HALL: &str = "\
##############################
#@...........................#
#............................#
#............................#
##############################
";

/// A cell inside the room.
const INSIDE: Pos = Pos::new(11, 2);
/// A cell outside the room.
const OUTSIDE: Pos = Pos::new(3, 2);

/// The fixture content: the hall (cursed room, or the whole level cursed with `pervaded`) and a
/// plain level beside it.
fn content(pervaded: bool) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    let (room, header) = if pervaded {
        (String::new(), "effect = \"cursed\"\n")
    } else {
        (
            "[[room]]\neffect = \"cursed\"\narea = [10, 0, 19, 4]\n".to_string(),
            "",
        )
    };
    files.insert(
        "levels/test_curse.toml",
        &format!(
            "name = \"Test\"\n{header}ambient = [120, 120, 120, 0]\nmap = '''\n{HALL}'''\n\
             {room}[legend]\n\
             \"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n"
        ),
    );
    files.insert(
        "levels/test_curse_away.toml",
        "name = \"Away\"\nambient = [120, 120, 120, 0]\nmap = '''\n#####\n#@..#\n#####\n'''\n\
         [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n",
    );
    files.insert(
        "scenarios/test_curse.toml",
        "name = \"T\"\nstart_level = \"test_curse\"\n",
    );
    Arc::new(Content::load(&files).expect("fixture content is valid"))
}

fn world(pervaded: bool) -> World {
    World::new(content(pervaded), "test_curse", None).expect("world")
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn said(w: &World, line: &str) -> bool {
    w.messages().iter().any(|m| m.text == line)
}

/// The creature standing at `p`, if any.
fn at(w: &World, p: Pos) -> Option<ActorId> {
    let level = w.state.current_level.clone();
    w.state.actor_at(&level, p).map(|a| a.id)
}

/// A giant rat at `p`.
fn rat_at(w: &mut World, p: Pos) -> ActorId {
    let spawn = CreatureSpawn {
        creature: w.content().creature_id("rat").expect("rat"),
        faction: None,
        alignment: None,
        npc: None,
        pos: p,
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Default::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    };
    let (content, level) = (w.content.clone(), w.state.current_level.clone());
    travel::make_creature(&mut w.state, &content, &level, &spawn)
}

/// Kills whoever stands at `p` (a giant rat put there if nobody does), by `by` (the player, or
/// nobody).
fn kill(w: &mut World, p: Pos, by: Option<ActorId>) {
    let id = at(w, p).unwrap_or_else(|| rat_at(w, p));
    let now = w.state.time;
    let mut events = Vec::new();
    w.dies(id, by, None, now, &mut events);
}

/// The corpses of giant rats lying at `p`.
fn corpses(w: &World, p: Pos) -> u32 {
    let rat = w.content().creature_id("rat").expect("giant rat");
    let corpse = w.content().corpse_of(rat).expect("a rat's corpse");
    (w.state.current().objects_at(p).iter())
        .filter(|s| s.object == corpse)
        .map(|s| s.count)
        .sum()
}

/// The faction of the undead.
fn undead(w: &World) -> crate::content::FactionId {
    w.content().factions().id("undead").expect("undead")
}

#[test]
fn the_cursed_room_is_data() {
    let content = Content::builtin().expect("content");
    let effects = content.room_effects();
    let cursed = effects.get(effects.id("cursed").expect("cursed"));
    assert_eq!(
        (cursed.name.as_str(), cursed.mark, cursed.weight),
        ("cursed room", 'g', 6)
    );
    assert_eq!(
        (cursed.level_chance, cursed.needs_danger, cursed.unholy),
        (5, true, true)
    );
    let rise = cursed.rise.as_ref().expect("the dead rise");
    assert_eq!(rise.faction, "undead");
    assert_eq!(rise.after, crate::time::Duration::minutes(2));
    assert_eq!(
        cursed.near.as_deref(),
        Some("A cold draught smells of the grave.")
    );
    assert_eq!(cursed.enter, "The air is cold and the dead are near.");
    assert_eq!(
        cursed.pervade.as_deref(),
        Some("The dead do not rest on this level.")
    );
    // The draining room is unholy too; nothing else raises the dead.
    let draining = effects.get(effects.id("draining").expect("draining"));
    assert!(draining.unholy);
    for e in effects.defs().iter().filter(|e| e.id != "cursed") {
        assert!(e.rise.is_none(), "{}", e.id);
    }
}

#[test]
fn a_rise_is_refused_without_a_danger_or_with_a_bad_time_or_faction() {
    let builtin = include_str!("../../../../assets/data/room_effects.toml");
    let load = |text: String| {
        let mut files = ContentFiles::builtin();
        files.insert("data/room_effects.toml", &text);
        Content::load(&files)
    };
    assert!(load(builtin.to_string()).is_ok());
    let cursed = "needs_danger = true\nlevel_chance = 5";
    assert!(builtin.contains(cursed));
    // A rise needs a danger.
    assert!(load(builtin.replace(cursed, "level_chance = 5")).is_err());
    let rise = "rise = { faction = \"undead\", after = \"2m\" }";
    assert!(builtin.contains(rise));
    for bad in ["2h", "0s", "soon"] {
        let text = builtin.replace(rise, &rise.replace("2m", bad));
        assert!(load(text).is_err(), "{bad}");
    }
    // An unknown faction is refused once the factions load.
    assert!(load(builtin.replace(rise, &rise.replace("undead", "nobody"))).is_err());
}

#[test]
fn the_dead_rise_after_two_minutes_as_undead_of_their_level() {
    let mut w = world(false);
    let died = w.state.time;
    kill(&mut w, INSIDE, None);
    assert_eq!(corpses(&w, INSIDE), 1);
    let rising = w.risings()[0];
    assert_eq!(
        (rising.pos(), rising.at()),
        (INSIDE, died + crate::time::Duration::minutes(2))
    );
    // Said with the death's news.
    run(&mut w, "wait");
    assert!(
        said(&w, "The giant rat's corpse twitches."),
        "{:?}",
        w.messages()
    );
    let into = w.content().creature(rising.risen()).clone();
    let rat = w.content().creature(rising.dead());
    assert_eq!(into.faction, Some(undead(&w)));
    assert!(!into.unique);
    let candidates = rise_candidates(w.content(), undead(&w), "", rat.level);
    assert!(candidates.contains(&rising.risen()));
    assert!(
        candidates
            .iter()
            .all(|&c| w.content().creature(c).level == rat.level)
    );
    // Not yet after a minute...
    run(&mut w, "rest:1m");
    assert_eq!(corpses(&w, INSIDE), 1);
    assert_eq!(w.risings().len(), 1);
    // ...and it has risen after two: the corpse is gone, and an undead stands there.
    run(&mut w, "rest:2m");
    assert_eq!(corpses(&w, INSIDE), 0);
    assert!(w.risings().is_empty());
    let risen = at(&w, INSIDE).expect("the risen dead");
    assert_eq!(w.state.actors[&risen].creature, rising.risen());
    assert!(
        w.state.actors[&risen].inventory.is_empty(),
        "it carries nothing"
    );
    let line = format!(
        "The giant rat's corpse rises as {} {}!",
        crate::content::article(&into.name),
        into.name
    );
    assert!(said(&w, &line), "{:?}", w.messages());
    // The rest stopped for it.
    assert!(
        w.messages()
            .iter()
            .any(|m| m.text.contains("you stop resting"))
    );
}

#[test]
fn a_corpse_taken_or_eaten_does_not_rise() {
    let mut w = world(false);
    kill(&mut w, INSIDE, None);
    let level = w.state.current_mut();
    level.set_pile(INSIDE, Vec::new());
    run(&mut w, "rest:3m");
    assert!(w.risings().is_empty());
    assert!(at(&w, INSIDE).is_none(), "nothing rose");
}

#[test]
fn a_corpse_carried_out_and_dropped_is_not_cursed_again() {
    let mut w = world(false);
    kill(&mut w, INSIDE, None);
    let pile = w.state.current().objects_at(INSIDE).to_vec();
    let level = w.state.current_mut();
    level.set_pile(INSIDE, Vec::new());
    level.set_pile(Pos::new(4, 2), pile);
    run(&mut w, "rest:3m");
    assert!(at(&w, INSIDE).is_none() && at(&w, Pos::new(4, 2)).is_none());
}

#[test]
fn the_dead_outside_and_the_undead_never_rise() {
    let mut w = world(false);
    kill(&mut w, OUTSIDE, None);
    assert!(w.risings().is_empty());
    // A risen one killed in the room stays dead.
    kill(&mut w, INSIDE, None);
    run(&mut w, "rest:3m");
    kill(&mut w, INSIDE, None);
    assert!(w.risings().is_empty());
}

#[test]
fn the_nearest_level_stands_in_when_none_has_the_dead_s() {
    let content = Content::builtin().expect("content");
    let undead = content.factions().id("undead").expect("undead");
    let level_of = |c: CreatureId| content.creature(c).level;
    let highest = (content.creature_defs().iter())
        .filter(|d| d.faction == Some(undead) && !d.unique && d.campaign.is_none())
        .map(|d| d.level)
        .max()
        .expect("undead");
    let top = rise_candidates(&content, undead, "", u8::MAX);
    assert!(!top.is_empty() && top.iter().all(|&c| level_of(c) == highest));
    // Every level gets the nearest: none nearer than what is picked.
    for level in 0..=u8::MAX {
        let picked = rise_candidates(&content, undead, "", level);
        let d = level_of(picked[0]).abs_diff(level);
        assert!(picked.iter().all(|&c| level_of(c).abs_diff(level) == d));
        // On a tie the lower level.
        let lower = level.checked_sub(d).filter(|_| d > 0);
        if let Some(lower) = lower {
            let has_lower = !rise_candidates(&content, undead, "", lower).is_empty()
                && rise_candidates(&content, undead, "", lower)
                    .iter()
                    .all(|&c| level_of(c) == lower);
            if has_lower {
                assert!(
                    picked.iter().all(|&c| level_of(c) == lower),
                    "level {level}"
                );
            }
        }
    }
    // A campaign's own undead join only in that campaign.
    let own = (content.creature_defs().iter()).find(|d| {
        d.faction == Some(undead) && !d.unique && d.campaign.as_deref() == Some("ascmearc")
    });
    if let Some(own) = own {
        let id = content.creature_id(&own.id).expect("id");
        assert!(rise_candidates(&content, undead, "ascmearc", own.level).contains(&id));
        assert!(!rise_candidates(&content, undead, "showcase", own.level).contains(&id));
    }
}

#[test]
fn old_delver_keeps_the_player_s_kills_down() {
    let mut w = world(false);
    w.player_mut().talents.take("old_delver", None);
    let player = w.state.player;
    kill(&mut w, INSIDE, Some(player));
    assert!(w.risings().is_empty());
    // A death the player did not cause still rises.
    let mut w = world(false);
    w.player_mut().talents.take("old_delver", None);
    kill(&mut w, INSIDE, None);
    assert_eq!(w.risings().len(), 1);
}

#[test]
fn on_a_cursed_level_the_dead_rise_anywhere() {
    let mut w = world(true);
    assert!(said(&w, "The dead do not rest on this level."));
    kill(&mut w, OUTSIDE, None);
    kill(&mut w, INSIDE, None);
    assert_eq!(w.risings().len(), 2);
    run(&mut w, "rest:3m");
    assert!(at(&w, OUTSIDE).is_some() && at(&w, INSIDE).is_some());
}

#[test]
fn a_frozen_level_s_dead_rise_when_the_player_returns() {
    let mut w = world(false);
    kill(&mut w, INSIDE, None);
    w.travel("test_curse_away", None).expect("away");
    run(&mut w, "rest:10m");
    // Nothing rose while the player was away...
    let rising = w.state.levels["test_curse"].risings.clone();
    assert_eq!(rising.len(), 1);
    w.travel("test_curse", None).expect("back");
    assert!(at(&w, INSIDE).is_none());
    // ...and it rises on the first turn back.
    run(&mut w, "wait");
    assert!(w.risings().is_empty());
    assert_eq!(
        at(&w, INSIDE).map(|id| w.state.actors[&id].creature),
        Some(rising[0].risen())
    );
}

#[test]
fn risings_are_saved_and_dumped() {
    let mut w = world(false);
    kill(&mut w, INSIDE, None);
    let json = serde_json::to_string(&w.state).expect("saves");
    let back: WorldState = serde_json::from_str(&json).expect("loads");
    assert_eq!(back.levels["test_curse"].risings, w.state.current().risings);
    assert!(json.contains("risings"));
    let text = dump::ascii(&w, &[Layer::Actors]);
    assert!(text.contains("rising rat at (11, 2) as "), "{text}");
    // A level without one writes none.
    let plain = World::new(content(false), "test_curse", None).expect("world");
    let json = serde_json::to_string(&plain.state).expect("saves");
    assert!(!json.contains("risings"));
}

/// A world of the fixture whose player has Consecrate, is Good, and has mana to spare.
fn priest(pervaded: bool) -> World {
    let mut w = world(pervaded);
    let me = w.player_mut();
    me.talents.take("consecrate", None);
    me.alignment = Some(Score::new(500));
    me.mana_base = 40;
    me.mana = 40;
    w
}

#[test]
fn consecrate_lifts_a_cursed_room_for_good() {
    let mut w = priest(false);
    run(&mut w, "move:E*10");
    assert_eq!(w.state.player().pos, Pos::new(11, 1));
    let alignment = w.state.player().alignment;
    let mana = w.state.player().mana;
    let start = w.state.time;
    // A corpse that would rise.
    let rat = w.content().creature_id("rat").expect("rat");
    let corpse = w.content().corpse_of(rat).expect("corpse");
    w.state
        .current_mut()
        .set_pile(INSIDE, vec![crate::object::Stack::corpse(corpse, start)]);
    w.state.current_mut().risings.push(Rising {
        at: start + crate::time::Duration::minutes(2),
        pos: INSIDE,
        from: rat,
        into: rat,
    });
    run(&mut w, "use:consecrate");
    assert!(said(&w, "You kneel and pray to consecrate this place."));
    assert!(said(
        &w,
        "A stillness settles over the room: it is consecrated."
    ));
    assert!(w.state.time.since(start) >= crate::time::Duration::minutes(1));
    assert!(w.state.current().rooms.is_empty(), "a plain room");
    assert!(w.risings().is_empty(), "nothing rises there now");
    assert_eq!(w.state.player().mana, mana - 15);
    assert!(w.state.player().alignment > alignment, "a good deed");
    run(&mut w, "rest:3m");
    assert!(at(&w, INSIDE).is_none());
    // Now there is nothing to consecrate: refused, no time.
    let turn = w.state.turn;
    run(&mut w, "use:consecrate");
    assert!(said(&w, "There is nothing here to consecrate."));
    assert_eq!(w.state.turn, turn);
}

#[test]
fn consecrate_lifts_a_whole_cursed_level() {
    let mut w = priest(true);
    run(&mut w, "use:consecrate");
    assert!(said(
        &w,
        "A stillness settles over the whole level: it is consecrated."
    ));
    assert_eq!(w.state.current().pervading, None);
}

#[test]
fn consecrate_is_refused_elsewhere_with_no_time_and_no_mana() {
    let mut w = priest(false);
    let (turn, mana) = (w.state.turn, w.state.player().mana);
    run(&mut w, "use:consecrate");
    assert!(said(&w, "There is nothing here to consecrate."));
    assert_eq!((w.state.turn, w.state.player().mana), (turn, mana));
    assert_eq!(w.state.current().rooms.len(), 1);
}

#[test]
fn consecrate_asks_for_mana_and_answers_only_the_good_side() {
    let mut w = priest(false);
    let ability = Ability::Class(ClassActive::Consecrate);
    assert_eq!(ability.unavailable(w.state.player(), w.content()), None);
    // Neutral: usable, and unanswered: a cast's time, no mana, nothing lifted.
    w.player_mut().alignment = Some(Score::new(0));
    assert_eq!(ability.unavailable(w.state.player(), w.content()), None);
    run(&mut w, "move:E*10");
    let (turn, mana) = (w.state.turn, w.state.player().mana);
    run(&mut w, "use:consecrate");
    assert!(said(
        &w,
        "You kneel and pray, but no holy power answers you."
    ));
    assert_eq!((w.state.turn, w.state.player().mana), (turn + 1, mana));
    assert_eq!(w.state.current().rooms.len(), 1, "unanswered");
    run(&mut w, "move:W*10");
    w.player_mut().alignment = Some(Score::new(500));
    w.player_mut().mana = 10;
    assert_eq!(
        ability.unavailable(w.state.player(), w.content()),
        Some(Unavailable::NoMana)
    );
    run(&mut w, "move:E*10 use:consecrate");
    assert_eq!(w.state.current().rooms.len(), 1, "refused");
}

#[test]
fn an_interrupted_prayer_lifts_nothing() {
    let mut w = priest(false);
    run(&mut w, "move:E*10");
    let mana = w.state.player().mana;
    // The dead rise twenty seconds into the prayer, in view: it stops.
    let start = w.state.time;
    let rat = w.content().creature_id("rat").expect("rat");
    let corpse = w.content().corpse_of(rat).expect("corpse");
    w.state
        .current_mut()
        .set_pile(INSIDE, vec![crate::object::Stack::corpse(corpse, start)]);
    w.state.current_mut().risings.push(Rising {
        at: start + crate::time::Duration::seconds(20),
        pos: INSIDE,
        from: rat,
        into: rat,
    });
    run(&mut w, "use:consecrate");
    assert!(
        said(&w, "A giant rat comes into view; you stop your prayer."),
        "{:?}",
        w.messages()
    );
    assert_eq!(w.state.current().rooms.len(), 1, "still cursed");
    assert_eq!(w.state.player().mana, mana - 15, "the mana is spent");
    assert!(w.state.time.since(start) < crate::time::Duration::minutes(1));
}

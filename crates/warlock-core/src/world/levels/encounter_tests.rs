//! Overworld encounters (spec §7 "Overworld encounters", #804; `docs/dev/encounters.md`): the
//! roll and its draws, the four outcomes, the pending choice, evading, the positions by outcome,
//! the known party, saves, and replays (the design's §9.3 invariants). The fixture is the
//! built-in content with a table in the showcase campaign: kobolds (hostile), deer (wildlife),
//! and the hermit (peaceful).

use crate::action::{Action, parse_actions};
use crate::content::{ContentFiles, LevelKind, local_area_id};
use crate::replay::Replay;
use crate::test_support::draws;
use crate::test_support::{builtin, last_text as last, run as act};
use crate::world::*;

const TABLE: &str = r#"
[[table]]
id = "wilds"
[[table.entry]]
id = "kobolds"
name = "a band of kobolds"
weight = 1
group = [{ creature = "kobold", count = "3" }]
[[table.entry]]
id = "deer"
name = "a herd of deer"
weight = 1
stance = "wildlife"
group = [{ creature = "deer", count = "2" }]
[[table.entry]]
id = "hermit"
name = "a hermit"
weight = 1
stance = "peaceful"
group = [{ creature = "hermit", count = "1" }]
"#;

const RULES: &str = include_str!("../../../../../assets/data/encounters.toml");

/// The built-in content with the fixture table, under a clear sky and with nobody hungering;
/// `certain`: every step rolls an encounter (the rates at their highest, the table at 1000 %).
fn content(certain: bool) -> Arc<Content> {
    fixture(certain, 25)
}

/// [`content`], with a rest rolling `rest` percent of the rate when `certain` (#807).
fn fixture(certain: bool, rest: u32) -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.remove("data/weather.toml");
    files.remove("data/hunger.toml");
    let table = if certain {
        TABLE.replacen("id = \"wilds\"", "id = \"wilds\"\nrate_percent = 1000", 1)
    } else {
        TABLE.to_string()
    };
    files.insert("campaigns/showcase/data/encounters/test.toml", &table);
    if certain {
        let rules = (RULES.lines())
            .map(|l| match l.split_once(" = ") {
                Some(("rate_percent", _)) => format!("rate_percent = {rest}"),
                Some((k, _))
                    if [
                        "road",
                        "plains",
                        "grassland",
                        "forest",
                        "hills",
                        "dense_forest",
                        "swamp",
                    ]
                    .contains(&k) =>
                {
                    format!("{k} = 10000")
                }
                _ => l.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n");
        files.insert("data/encounters.toml", &rules);
    }
    Arc::new(Content::load(&files).expect("fixture content"))
}

fn world(certain: bool) -> World {
    World::new(content(certain), "world", None).expect("world")
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.text.contains(text))
}

fn party(w: &World) -> Vec<ActorId> {
    (w.state.actors.values())
        .filter(|a| a.id != w.state.player && w.party_of(a.id).is_some())
        .map(|a| a.id)
        .collect()
}

#[test]
fn no_draws_without_an_encounter() {
    // No tables: nothing is drawn on any step, from any stream.
    let mut plain = World::new(builtin(), "world", None).expect("world");
    let rng = draws(&plain);
    act(&mut plain, "move:E*3 move:S*2");
    assert!(plain.state.encounters.is_none(), "no stream is made");
    assert_eq!(draws(&plain), rng, "the gameplay stream is not drawn");
    // Pending: steps are refused and draw nothing.
    let mut w = world(true);
    act(&mut w, "meet:kobolds:drop");
    let (rng, encounters) = (draws(&w), w.state.encounters.clone());
    let at = w.player_pos();
    act(&mut w, "move:E wait");
    assert_eq!(w.player_pos(), at);
    assert_eq!(last(&w), PENDING_REFUSED);
    assert_eq!((draws(&w), w.state.encounters.clone()), (rng, encounters));
}

#[test]
fn a_forced_outcome_on_a_peaceful_party_draws_no_check() {
    let mut w = world(true);
    let rng = draws(&w);
    act(&mut w, "meet:hermit");
    assert_eq!(draws(&w), rng, "no lookouts for travellers");
    let view = w.perception().encounter().expect("pending");
    assert_eq!(view.title, "A hermit");
    let ids: Vec<_> = view.choices.iter().map(|(_, c)| c.id()).collect();
    assert_eq!(ids, ["hail", "pass"]);
    act(&mut w, "encounter:pass");
    assert!(w.state.encounter.is_none());
    assert_eq!(w.state.current().kind, LevelKind::Overworld);
}

#[test]
fn every_step_rolls_when_the_chance_is_certain_and_the_outcome_follows_the_lookouts() {
    let mut w = world(true);
    let mut met = 0;
    for _ in 0..30 {
        if w.state.encounter.is_some() {
            let outcome = w.state.encounter.as_ref().unwrap().outcome;
            let answer = match outcome {
                EncounterOutcome::Peaceful | EncounterOutcome::Wildlife => "encounter:pass",
                _ => "encounter:evade",
            };
            act(&mut w, answer);
            met += 1;
        }
        if w.state.current().kind != LevelKind::Overworld {
            break;
        }
        let at = w.player_pos();
        // Back and forth along the road.
        act(&mut w, if met % 2 == 0 { "move:E" } else { "move:W" });
        if w.player_pos() == at && w.state.encounter.is_none() {
            act(&mut w, "move:W");
        }
    }
    assert!(
        w.state.encounters.is_some(),
        "the stream is made on the first roll"
    );
    assert!(met > 0, "someone was met");
}

#[test]
fn the_drop_offers_three_choices_and_met_two() {
    let mut w = world(false);
    act(&mut w, "meet:kobolds:drop");
    assert!(said(&w, "You spot a band of kobolds"), "{}", last(&w));
    let v = w.perception().encounter().expect("pending");
    let labels: Vec<_> = v.choices.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(labels, ["Lie in wait", "Slip past", "Stand and fight"]);
    act(&mut w, "encounter:hunt");
    assert_eq!(last(&w), "That is not a choice now.");
    let mut w = world(false);
    act(&mut w, "meet:kobolds:met");
    let v = w.perception().encounter().expect("pending");
    let ids: Vec<_> = v.choices.iter().map(|(_, c)| c.id()).collect();
    assert_eq!(ids, ["evade", "fight"]);
    // Nothing pending: refused.
    let mut w = world(false);
    act(&mut w, "encounter:fight");
    assert_eq!(last(&w), "There is nothing to decide.");
}

#[test]
fn pass_by_generates_no_level_and_makes_no_actor() {
    let mut w = world(false);
    let (actors, levels) = (w.state.actors.len(), w.state.levels.len());
    act(&mut w, "meet:kobolds:pass");
    assert!(w.state.encounter.is_none());
    assert_eq!(
        (w.state.actors.len(), w.state.levels.len()),
        (actors, levels)
    );
}

/// The arrival edge cell's distance from the player along the way, and the area's middle.
fn area_of(w: &World) -> (String, Pos) {
    let level = w.state.current();
    (
        level.id.clone(),
        Pos::new(level.width() / 2, level.height() / 2),
    )
}

#[test]
fn a_chosen_fight_opens_the_area_with_everyone_aware_on_free_walkable_cells() {
    let mut w = world(false);
    let cell = w.player_pos();
    act(&mut w, "meet:kobolds:drop encounter:fight");
    let (id, middle) = area_of(&w);
    assert_eq!(id, local_area_id(cell));
    let members = party(&w);
    assert_eq!(members.len(), 3);
    let me = w.player_pos();
    let mut cells = vec![me];
    for m in &members {
        let a = &w.state.actors[m];
        assert_eq!(a.level, id);
        assert!(w.state.current().is_walkable(&w.content, a.pos));
        assert!(w.state.current().water_at(&w.content, a.pos).is_none());
        assert!(a.mind.last_seen.is_some(), "aware");
        cells.push(a.pos);
    }
    cells.sort();
    cells.dedup();
    assert_eq!(cells.len(), 4, "one a cell");
    // Between the edge and the middle.
    assert!(me.chebyshev(middle) > 0);
    assert_eq!(w.danger("overworld"), 0, "the overworld keeps danger 0");
}

#[test]
fn the_player_stands_nearer_the_edge_the_better_the_encounter() {
    let distance = |actions: &str| {
        let mut w = world(false);
        act(&mut w, actions);
        let (_, middle) = area_of(&w);
        w.player_pos().chebyshev(middle)
    };
    let ambush = distance("meet:kobolds:drop encounter:ambush");
    let fight = distance("meet:kobolds:drop encounter:fight");
    let ambushed = distance("meet:kobolds:ambushed");
    assert!(
        ambush > fight,
        "lying in wait nearer the edge: {ambush} {fight}"
    );
    assert!(
        fight > ambushed,
        "ambushed in the middle: {fight} {ambushed}"
    );
    assert!(ambushed <= 2, "ambushed in the middle: {ambushed}");
}

#[test]
fn an_ambushing_party_is_unaware_and_owes_an_action() {
    let mut w = world(false);
    act(&mut w, "meet:kobolds:drop encounter:ambush");
    assert!(said(&w, "You wait in cover."));
    let player = w.state.player;
    for m in party(&w) {
        assert!(w.unaware(m, player), "unaware of the player");
        assert!(w.state.actors[&m].energy < 0, "one action behind");
        assert!(w.state.actors[&m].patrol.is_some(), "walking the way");
    }
}

#[test]
fn an_ambushed_player_is_surprised_at_most_once() {
    let mut w = world(false);
    let turn = w.turn();
    let out = w.apply(&"meet:kobolds:ambushed".parse::<Action>().unwrap());
    assert!(
        said(&w, "You are set upon by a band of kobolds"),
        "{}",
        last(&w)
    );
    assert_eq!(w.state.current().kind, LevelKind::Outdoor);
    assert_eq!(
        w.turn(),
        turn + 1,
        "the party took the player's lost action"
    );
    let surprised = w.content.surprise().unwrap().status;
    let begun = (out.events.iter())
        .filter(|e| {
            matches!(e, crate::action::Event::Status { actor, status, began: true }
            if *actor == w.state.player && *status == surprised)
        })
        .count();
    assert!(begun <= 1);
    let player = w.state.player;
    for m in party(&w) {
        assert!(!w.unaware(player, m), "the player knows them all");
        assert!(w.state.actors[&m].pos.chebyshev(w.player_pos()) <= 4);
    }
}

#[test]
fn evading_takes_the_detour_and_a_failure_opens_the_fight_close() {
    let mut made = 0;
    for seed in 0..20 {
        let mut w = World::new(content(false), "world", Some(seed)).expect("world");
        act(&mut w, "meet:kobolds:met");
        let time = w.time();
        act(&mut w, "encounter:evade");
        assert!(w.time() > time, "the detour takes time");
        assert!(w.state.encounter.is_none());
        if w.state.current().kind == LevelKind::Overworld {
            made += 1;
            assert!(said(&w, "You lose them in"));
            assert!(party(&w).is_empty(), "nothing is made");
        } else {
            assert!(said(&w, "They run you down."));
            for m in party(&w) {
                let d = w.state.actors[&m].pos.chebyshev(w.player_pos());
                assert!(d <= 5, "close: {d}");
            }
        }
    }
    assert!(made > 0 && made < 20, "both ways: {made}");
}

#[test]
fn a_party_got_away_from_is_met_again_and_known() {
    let mut w = world(false);
    let cell = w.player_pos();
    act(&mut w, "meet:kobolds:drop encounter:fight");
    let first = party(&w);
    // Back to the overworld cell (nobody within reach of the edge rule: they are far in).
    let exit = w.state.current().exit.clone().expect("an edge exit");
    let player = w.state.player;
    w.state.actors.get_mut(&player).unwrap().pos = Pos::new(0, 0);
    let _ = w.travel(&exit.to, Some(exit.pos));
    assert_eq!(w.player_pos(), cell);
    act(&mut w, "meet:kobolds:met");
    let view = w.perception().encounter().expect("pending");
    assert!(view.text.ends_with(KNOWN_LINE), "{}", view.text);
    act(&mut w, "encounter:fight");
    assert_eq!(party(&w), first, "the same party, placed again");
}

#[test]
fn the_dev_meet_is_refused_off_the_overworld_and_for_an_unknown_entry() {
    let mut w = world(false);
    act(&mut w, "meet:nobody");
    assert!(last(&w).contains("no encounter `nobody`"));
    act(&mut w, "descend");
    assert_eq!(w.state.current().kind, LevelKind::Outdoor);
    act(&mut w, "meet:kobolds");
    assert!(last(&w).contains("no local area"), "{}", last(&w));
}

#[test]
fn a_pending_encounter_survives_a_save() {
    let mut w = world(false);
    act(&mut w, "meet:kobolds:drop");
    let json = serde_json::to_string(&w.state).expect("saves");
    let state: WorldState = serde_json::from_str(&json).expect("loads");
    let loaded = World::from_state(w.content.clone(), state).expect("restores");
    assert_eq!(loaded.perception().encounter(), w.perception().encounter());
    assert_eq!(loaded.state_hash(), w.state_hash());
}

#[test]
fn encounters_replay_exactly_through_each_outcome() {
    let content = content(true);
    for actions in [
        "meet:kobolds:drop encounter:ambush wait*3",
        "meet:kobolds:met encounter:fight wait*3",
        "meet:kobolds:ambushed wait*3",
        "meet:kobolds:met encounter:evade",
        "meet:deer encounter:hunt wait",
        "move:E encounter:pass encounter:evade encounter:fight move:W",
    ] {
        let mut w = World::new(content.clone(), "world", None).expect("world");
        let mut replay = Replay::new(&w);
        for action in parse_actions(actions).expect("actions") {
            w.apply(&action);
            replay.push(&action, &w);
        }
        let verified = Replay::from_json(&replay.to_json())
            .expect("json")
            .run(content.clone())
            .expect("the replay verifies");
        assert_eq!(verified.state_hash(), w.state_hash(), "{actions}");
    }
}

#[test]
fn the_dumps_say_what_is_pending_and_who_came_of_it() {
    use crate::dump::{self, Layer};
    let mut w = world(false);
    act(&mut w, "meet:kobolds:drop");
    let text = dump::ascii(&w, &[Layer::Truth]);
    let header = text.lines().next().unwrap();
    assert!(
        header.contains("encounter pending: a band of kobolds (ambush, evade, fight)"),
        "{header}"
    );
    assert!(text.contains("encounter rate here"), "{text}");
    let json = dump::json(&w, &[], false);
    assert_eq!(json["encounter"]["outcome"], "drop");
    act(&mut w, "encounter:fight");
    let text = dump::ascii(&w, &[Layer::Actors]);
    assert!(text.contains("of encounter kobolds"), "{text}");
    assert!(dump::json(&w, &[], false)["encounter"].is_null());
}

/// The certain fixture (a rest rolling `rest` percent of the rate) at `seed`.
fn camp(rest: u32, seed: u64) -> World {
    World::new(fixture(true, rest), "world", Some(seed)).expect("world")
}

#[test]
fn resting_without_tables_a_share_or_the_overworld_draws_nothing() {
    let mut plain = World::new(builtin(), "world", None).expect("world");
    act(&mut plain, "rest:8h");
    assert!(plain.state.encounters.is_none(), "no tables: no stream");
    // No share of the rate for a rest: none drawn either.
    let mut w = camp(0, 1);
    act(&mut w, "rest:8h");
    assert!(w.state.encounters.is_none(), "a chance of 0 draws nothing");
    assert!(said(&w, "You rest for 8 hours"), "{}", last(&w));
    // In the cell's area (no roll there) a rest rolls nothing.
    let mut w = camp(25, 1);
    act(&mut w, "descend rest:8h");
    assert_eq!(w.state.current().kind, LevelKind::Outdoor);
    assert!(w.state.encounters.is_none());
}

#[test]
fn an_encounter_stops_a_rest_and_a_sleeper_never_has_the_drop() {
    let (mut met, mut ambushed) = (0, 0);
    for seed in 0..24 {
        let mut w = camp(25, seed);
        let time = w.time();
        act(&mut w, "rest:24h");
        assert!(w.state.encounters.is_some(), "the rest rolled");
        let rested = w.time().since(time);
        assert!(rested < crate::time::Duration::hours(24), "{rested:?}");
        assert!(said(&w, "You stop resting after"), "{}", last(&w));
        match &w.state.encounter {
            Some(p) => {
                assert!(p.woken, "met in camp");
                assert_ne!(p.outcome, EncounterOutcome::Drop, "asleep: never the drop");
                let view = w.perception().encounter().expect("pending");
                assert!(view.text.starts_with("You wake"), "{}", view.text);
                if p.outcome == EncounterOutcome::Met {
                    met += 1;
                }
                // Nothing more until it is answered.
                act(&mut w, "rest:1h");
                assert_eq!(last(&w), PENDING_REFUSED);
            }
            None => {
                assert!(said(&w, "You wake with a band of kobolds upon you!"));
                assert_eq!(w.state.current().kind, LevelKind::Outdoor);
                assert!(!party(&w).is_empty());
                ambushed += 1;
            }
        }
    }
    assert!(met > 0 && ambushed > 0, "met {met}, ambushed {ambushed}");
}

/// The player trained in Survival, standing on the nearest overworld cell of wild land with a
/// local area.
fn forager(w: &mut World) {
    let content = w.content.clone();
    let survival = content.skill_id("survival").expect("Survival");
    let player = w.state.player;
    let me = w.state.actors.get_mut(&player).expect("player");
    let mut skills: Vec<_> = me.skills.iter().filter(|(s, _)| *s != survival).collect();
    let trained = crate::skills::Trained {
        value: 20,
        marks: 0,
    };
    skills.push((survival, trained));
    me.skills = skills.into_iter().collect();
    let campaign = w.state.campaign.clone();
    let level = w.state.current();
    let from = w.player_pos();
    let wild = (level.terrain.iter())
        .map(|(p, _)| p)
        .filter(|&p| crate::world::skills::forage::forage_ground(&content, level, p).is_some())
        .filter(|&p| content.local_area(&campaign, &local_area_id(p)).is_some())
        .min_by_key(|&p| (p.chebyshev(from), p))
        .expect("wild land near the start");
    w.state.actors.get_mut(&player).unwrap().pos = wild;
}

#[test]
fn a_forage_on_the_overworld_rolls_and_an_encounter_cuts_it_short() {
    let mut stopped = 0;
    for seed in 0..24 {
        let mut w = camp(25, seed);
        forager(&mut w);
        act(&mut w, "forage");
        assert!(w.state.encounters.is_some(), "the forage rolled");
        let met = w.state.encounter.is_some() || w.state.current().kind != LevelKind::Overworld;
        if met {
            stopped += 1;
            assert!(said(&w, "You stop foraging."), "{}", last(&w));
            assert!(!said(&w, "You gather"), "a forage cut short finds nothing");
            if let Some(p) = &w.state.encounter {
                assert!(!p.woken, "a forager is awake");
            }
        }
    }
    assert!(stopped > 0, "someone came upon a forager");
}

#[test]
fn camping_replays_exactly() {
    let content = fixture(true, 25);
    for actions in ["rest:24h encounter:pass rest:2h", "rest:dawn wait*3"] {
        for seed in [1, 2, 3] {
            let mut w = World::new(content.clone(), "world", Some(seed)).expect("world");
            let mut replay = Replay::new(&w);
            for action in parse_actions(actions).expect("actions") {
                w.apply(&action);
                replay.push(&action, &w);
            }
            let verified = Replay::from_json(&replay.to_json())
                .expect("json")
                .run(content.clone())
                .expect("the replay verifies");
            assert_eq!(verified.state_hash(), w.state_hash(), "{actions}");
        }
    }
}

// Talking past (#1017; `talk_past.rs`).

/// The fixture with a flagged entry of kobolds beside the rest (`talk_past = true`).
fn talking() -> Arc<Content> {
    let mut files = ContentFiles::builtin();
    files.remove("data/weather.toml");
    files.remove("data/hunger.toml");
    let table = format!(
        "{TABLE}[[table.entry]]\nid = \"talkers\"\nname = \"a band of talkers\"\nweight = 1\n\
         group = [{{ creature = \"kobold\", count = \"2\" }}]\ntalk_past = true\n"
    );
    files.insert("campaigns/showcase/data/encounters/test.toml", &table);
    Arc::new(Content::load(&files).expect("fixture content"))
}

/// A run of the fixture on `seed`, its player Presence `presence` (the rest 14) and Bluff
/// `bluff` (`None`: untrained).
fn talker(seed: u64, presence: u8, bluff: Option<u16>) -> World {
    let mut w = World::new(talking(), "world", Some(seed)).expect("world");
    let skill = w.content().skill_id("bluff").expect("bluff");
    let p = w.player_mut();
    let mut scores = crate::stats::PerStat::splat(14);
    scores[crate::stats::Stat::Presence] = presence;
    p.stats = crate::stats::Stats::new(scores, crate::stats::PerStat::splat(40));
    let mut all: Vec<_> = p.skills.iter().filter(|(s, _)| *s != skill).collect();
    if let Some(value) = bluff {
        all.push((skill, crate::skills::Trained { value, marks: 0 }));
    }
    p.skills = all.into_iter().collect();
    w
}

fn choice_ids(w: &World) -> Vec<&'static str> {
    let v = w.perception().encounter().expect("pending");
    v.choices.iter().map(|(_, c)| c.id()).collect()
}

#[test]
fn talking_past_is_offered_only_when_flagged_met_and_trained() {
    let mut w = talker(1, 14, Some(50));
    act(&mut w, "meet:talkers:met");
    assert_eq!(choice_ids(&w), ["evade", "bluff", "fight"]);
    let v = w.perception().encounter().unwrap();
    assert_eq!(v.choices[1].0, "Talk your way past");
    let json = crate::dump::json(&w, &[], false);
    assert_eq!(json["encounter"]["choices"][1]["id"], "bluff");
    // Not from the drop: the party has not seen them.
    let mut w = talker(1, 14, Some(50));
    act(&mut w, "meet:talkers:drop");
    assert_eq!(choice_ids(&w), ["ambush", "evade", "fight"]);
    act(&mut w, "encounter:bluff");
    assert_eq!(last(&w), "That is not a choice now.");
    // Not untrained.
    let mut w = talker(1, 14, None);
    act(&mut w, "meet:talkers:met");
    assert_eq!(choice_ids(&w), ["evade", "fight"]);
    let rng = draws(&w);
    act(&mut w, "encounter:bluff");
    assert_eq!(last(&w), "That is not a choice now.");
    assert_eq!(draws(&w), rng, "a refusal draws nothing");
    // Not for an entry without the flag.
    let mut w = talker(1, 14, Some(50));
    act(&mut w, "meet:kobolds:met");
    assert_eq!(choice_ids(&w), ["evade", "fight"]);
}

#[test]
fn talking_past_is_made_or_failed_by_chance_and_takes_time_either_way() {
    let (mut made, mut failed) = (0, 0);
    for seed in 0..40 {
        let mut w = talker(seed, 16, Some(50));
        act(&mut w, "meet:talkers:met");
        let time = w.time();
        act(&mut w, "encounter:bluff");
        assert!(w.time() > time, "the try takes time");
        assert!(w.state.encounter.is_none());
        if w.state.current().kind == LevelKind::Overworld {
            made += 1;
            assert!(said(&w, "You talk your way past."));
            assert!(party(&w).is_empty(), "they let the player go");
        } else {
            failed += 1;
            assert!(said(&w, "They are not fooled."));
            assert_eq!(party(&w).len(), 2, "the fight opens");
            for m in party(&w) {
                assert!(w.state.actors[&m].mind.last_seen.is_some(), "aware");
            }
        }
    }
    assert!(made > 0 && failed > 0, "both ways: {made} {failed}");
    // A silver tongue mostly gets by; a clumsy one mostly does not.
    let past = |presence, bluff| {
        (0..40)
            .filter(|&seed| {
                let mut w = talker(seed, presence, Some(bluff));
                act(&mut w, "meet:talkers:met encounter:bluff");
                w.state.current().kind == LevelKind::Overworld
            })
            .count()
    };
    assert!(past(30, 200) > past(3, 1), "Presence and Bluff help");
}

#[test]
fn every_try_at_talking_past_marks_bluff() {
    let mut w = talker(3, 14, Some(50));
    act(&mut w, "meet:talkers:met encounter:bluff");
    let skill = w.content().skill_id("bluff").expect("bluff");
    let marks = w.state.player().skills.get(skill).map_or(0, |t| t.marks);
    assert!(marks > 0, "a mark every try");
}

#[test]
fn talking_past_replays_exactly() {
    let content = talking();
    let spec: crate::character::CharacterSpec = "human,male,bard".parse().expect("a bard");
    for seed in 0..6 {
        let mut w =
            World::with_character(content.clone(), "world", Some(seed), Some(&spec)).expect("w");
        let mut replay = Replay::new(&w);
        let actions = "meet:talkers:met encounter:bluff wait*2";
        for action in parse_actions(actions).expect("actions") {
            w.apply(&action);
            replay.push(&action, &w);
        }
        assert!(said(&w, "past.") || said(&w, "fooled."), "seed {seed}");
        let verified = Replay::from_json(&replay.to_json())
            .expect("json")
            .run(content.clone())
            .expect("the replay verifies");
        assert_eq!(verified.state_hash(), w.state_hash(), "seed {seed}");
    }
}

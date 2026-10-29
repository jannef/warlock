//! Surprise (spec §8 "Surprise and sneak attacks"; `world/surprise.rs`): unaware both ways round,
//! the initiative check's two outcomes and its two draws, the Surprised status (flat-footed, no
//! free blows out of turn, the dear action and Cool head's ease), hidden traps and their
//! bystanders, sleepers and mimics without a check, the messages, saves and replays, no draw
//! while nobody is unaware, Hair trigger's blow, and Stalker's eye's word in the snapshot.

use super::*;
use crate::action::parse_actions;
use crate::combat::{Category, Defence, Target};
use crate::content::{ActionKind, ContentFiles};
use crate::replay::Replay;

/// A hall split by a wall: the player in the west part, a goblin at `k` beside them, another
/// at `K` behind the wall, a hidden alarm trap at `^` and a known one at `!`.
const HALL: &str = "\
############
#....#.....#
#@k^!#..K..#
#....#.....#
############
";

/// What `data/surprise.toml` says in a fixture.
#[derive(Clone, Copy)]
enum Rules {
    /// The shipped file.
    Shipped,
    /// The shipped file with this difficulty: 40 fails all but a natural 24, 1 makes all but a
    /// natural 2.
    Difficulty(i32),
    /// No file: nobody is surprised.
    Without,
}

/// The built-in content with [`HALL`] at `ambient` light, the surprise rules `rules`.
fn world(ambient: u8, rules: Rules) -> World {
    let mut files = ContentFiles::builtin();
    let path = "data/surprise.toml";
    match rules {
        Rules::Shipped => {}
        Rules::Difficulty(d) => {
            let text = include_str!("../../../../assets/data/surprise.toml");
            // Nobody slips past anyone here (`slip_tests.rs`): a creature in the dark sees.
            files.insert(
                path,
                &text
                    .replacen("difficulty = 13", &format!("difficulty = {d}"), 1)
                    .replacen("enabled = true", "enabled = false", 1),
            );
        }
        Rules::Without => {
            files.remove(path);
        }
    }
    files.insert(
        "levels/test_surprise.toml",
        &format!(
            "name = \"Test\"\nambient = [{ambient}, {ambient}, {ambient}, 0]\nmap = '''\n{HALL}'''\n\
             [legend]\n\"@\" = {{ terrain = \"floor\", spawn = \"player\" }}\n\
             \"k\" = {{ terrain = \"floor\", creature = \"goblin\" }}\n\
             \"K\" = {{ terrain = \"floor\", creature = \"goblin\" }}\n\
             \"^\" = {{ terrain = \"floor\", trap = \"alarm\" }}\n\
             \"!\" = {{ terrain = \"floor\", trap = \"alarm\", known = true }}\n"
        ),
    );
    files.insert(
        "scenarios/test_surprise.toml",
        "name = \"T\"\nstart_level = \"test_surprise\"\n",
    );
    let content = Arc::new(Content::load(&files).expect("fixture content is valid"));
    World::new(content, "test_surprise", None).expect("world")
}

/// The goblin standing in column `x`.
fn goblin(w: &World, x: i32) -> ActorId {
    (w.state.actors.values())
        .find(|a| a.kind == ActorKind::Creature && a.pos.x == x)
        .expect("a goblin")
        .id
}

/// Puts `id` to sleep: it takes no turn.
fn lull(w: &mut World, id: ActorId) {
    let asleep = w.content.sleep_status().expect("asleep");
    let def = w.content.status(asleep).clone();
    w.state
        .actors
        .get_mut(&id)
        .expect("actor")
        .effects
        .add(asleep, &def, 1, None);
}

fn surprised_status(w: &World) -> crate::content::StatusId {
    w.content.status_id("surprised").expect("surprised")
}

fn texts(w: &World) -> Vec<String> {
    w.messages().iter().map(|m| m.text.clone()).collect()
}

/// `w`'s generator after one natural roll more: the check's two draws.
fn after_a_check(w: &World) -> crate::rng::Rng {
    let mut rng = w.state.rng.clone();
    crate::combat::roll_natural(&mut rng);
    rng
}

#[test]
fn the_player_is_unaware_of_what_they_have_not_seen_lately() {
    let mut w = world(120, Rules::Shipped);
    let player = w.state.player;
    let far = goblin(&w, 8);
    for id in [goblin(&w, 2), far] {
        lull(&mut w, id);
    }
    assert!(w.unaware(player, far), "behind the wall, never seen");
    assert!(!w.unaware(player, goblin(&w, 2)), "in plain sight");
    // Brought into view, it is noticed this turn.
    w.state.actors.get_mut(&far).expect("goblin").pos = Pos::new(3, 1);
    w.apply(&Action::Wait);
    let turn = w.state.turn;
    assert_eq!(w.state.actors[&far].mind.noticed, Some(turn));
    // Out of sight again, it is remembered for `memory_turns` turns, and no longer.
    w.state.actors.get_mut(&far).expect("goblin").pos = Pos::new(8, 2);
    w.apply(&Action::Wait);
    assert_eq!(
        w.state.actors[&far].mind.noticed,
        Some(turn),
        "not seen again"
    );
    assert!(!w.unaware(player, far), "seen a turn ago");
    w.state.turn = turn + 20;
    assert!(!w.unaware(player, far), "twenty turns ago");
    w.state.turn = turn + 21;
    assert!(w.unaware(player, far), "forgotten");
    // Without the rules the player is never unaware, and notices nothing.
    let mut w = world(120, Rules::Without);
    let far = goblin(&w, 8);
    assert!(!w.unaware(player, far));
    w.apply(&Action::Wait);
    assert!(w.state.actors.values().all(|a| a.mind.noticed.is_none()));
}

#[test]
fn a_creature_is_unaware_of_what_its_eyes_miss() {
    let mut w = world(120, Rules::Shipped);
    let player = w.state.player;
    assert!(w.unaware(goblin(&w, 8), player), "the wall between");
    assert!(
        !w.unaware(goblin(&w, 2), player),
        "beside them in the light"
    );
    // Hunting someone, it is aware whatever it sees.
    let far = goblin(&w, 8);
    w.state.actors.get_mut(&far).expect("goblin").mind.last_seen = Some(Pos::new(1, 2));
    assert!(!w.unaware(far, player));
}

#[test]
fn a_failed_check_surprises_and_a_made_one_does_not() {
    let surprised = |rules| {
        let mut w = world(120, rules);
        let (player, id) = (w.state.player, goblin(&w, 2));
        let rng = after_a_check(&w);
        w.startle(id, Some(player), &mut Vec::new());
        assert_eq!(w.state.rng, rng, "two draws");
        (w.surprised(id), texts(&w))
    };
    let (caught, said) = surprised(Rules::Difficulty(40));
    assert!(caught);
    assert_eq!(
        said.last().map(String::as_str),
        Some("You catch the goblin unawares.")
    );
    let (caught, said) = surprised(Rules::Difficulty(1));
    assert!(!caught);
    assert_eq!(
        said.last().map(String::as_str),
        Some("The goblin whirls to face you.")
    );
}

#[test]
fn sleepers_and_mimics_are_surprised_with_no_check_and_no_word() {
    let mut w = world(120, Rules::Difficulty(1));
    let (player, id) = (w.state.player, goblin(&w, 2));
    lull(&mut w, id);
    let (rng, said) = (w.state.rng.clone(), texts(&w));
    w.startle(id, Some(player), &mut Vec::new());
    assert!(w.surprised(id), "no check to make");
    assert_eq!(w.state.rng, rng, "no draw");
    assert_eq!(
        texts(&w),
        said,
        "it was asleep: unawares goes without saying"
    );
    // A mimic in disguise likewise.
    let mut w = world(120, Rules::Difficulty(1));
    let id = goblin(&w, 2);
    let disguise = w.content.disguise_status().expect("disguised");
    let def = w.content.status(disguise).clone();
    w.state
        .actors
        .get_mut(&id)
        .expect("goblin")
        .effects
        .add(disguise, &def, 1, None);
    let rng = w.state.rng.clone();
    w.startle(id, Some(player), &mut Vec::new());
    assert!(w.surprised(id));
    assert_eq!(w.state.rng, rng);
}

#[test]
fn a_creature_out_of_the_dark_takes_the_player_by_surprise() {
    // In the dark the goblin sees the player (darkvision) and the player sees nothing.
    let mut w = world(0, Rules::Difficulty(40));
    w.apply(&Action::Wait);
    let said = texts(&w);
    let at = said.iter().position(|t| t == "You are taken by surprise!");
    let at = at.unwrap_or_else(|| panic!("taken by surprise: {said:?}"));
    assert!(at + 1 < said.len(), "before the attack's line: {said:?}");
    let player = w.state.player;
    assert!(w.surprised(player));
    // The dear action ends it, and the player is told (the goblin, asleep, strikes no more: in
    // the dark each of its blows would catch them unawares again).
    let id = goblin(&w, 2);
    lull(&mut w, id);
    w.apply(&Action::Wait);
    assert!(
        texts(&w).iter().any(|t| t == "You recover your wits."),
        "{:?} {}",
        texts(&w),
        w.surprised(player)
    );
    // Made, the player keeps their wits, said before the attack's line.
    let mut w = world(0, Rules::Difficulty(1));
    w.apply(&Action::Wait);
    let said = texts(&w);
    let at = said
        .iter()
        .position(|t| t == "You keep your wits about you.");
    assert!(at.expect("kept their wits") + 1 < said.len(), "{said:?}");
    assert!(!w.surprised(player));
}

#[test]
fn the_surprised_are_flat_footed() {
    let mut w = world(120, Rules::Shipped);
    let id = goblin(&w, 2);
    let defence = Defence::from_bonuses([
        (Category::Dexterity, 3),
        (Category::Dodge, 2),
        (Category::Armour, 4),
    ]);
    let target = Target::bare(defence);
    let flat = Target::bare(Defence::from_bonuses([(Category::Armour, 4)]));
    assert_eq!(w.flat_footed(id, false, target.clone()), target);
    assert_eq!(
        w.flat_footed(id, true, target.clone()),
        flat,
        "caught unawares"
    );
    let surprised = surprised_status(&w);
    w.afflict(id, surprised, 1, None, &mut Vec::new());
    assert_eq!(w.flat_footed(id, false, target.clone()), flat, "Surprised");
    // Without the rules nobody is caught flat-footed.
    let w = world(120, Rules::Without);
    assert_eq!(w.flat_footed(id, true, target.clone()), target);
}

#[test]
fn the_surprised_strike_nothing_out_of_their_turn() {
    let mut w = world(120, Rules::Shipped);
    let id = goblin(&w, 2);
    let all = FreeBlows {
        cleave: true,
        riposte: true,
        opportunist: true,
        gatekeeper: true,
        ..FreeBlows::default()
    };
    w.state.actors.get_mut(&id).expect("goblin").free_blows = all;
    assert_eq!(w.free_blows_of(&w.state.actors[&id]), all);
    let surprised = surprised_status(&w);
    w.afflict(id, surprised, 1, None, &mut Vec::new());
    let blows = w.free_blows_of(&w.state.actors[&id]);
    assert_eq!(
        blows,
        FreeBlows {
            cleave: true,
            ..FreeBlows::default()
        },
        "its own attack's cleave stays"
    );
}

#[test]
fn the_dear_action_costs_half_again_and_cool_head_spares_it() {
    let mut w = world(120, Rules::Shipped);
    let player = w.state.player;
    let wait = w.price_of(player, ActionKind::Wait);
    let surprised = surprised_status(&w);
    w.afflict(player, surprised, 1, None, &mut Vec::new());
    assert_eq!(w.price_of(player, ActionKind::Wait), wait * 3 / 2);
    w.player_mut().talents.take("cool_head", None);
    assert_eq!(w.price_of(player, ActionKind::Wait), wait);
    assert_eq!(w.status_cost(player, surprised), 100);
}

#[test]
fn wary_adds_to_the_check() {
    let w = world(120, Rules::Shipped);
    let mut wary = w.clone();
    wary.player_mut().talents.take("wary", None);
    let mods = wary.state.player().talent_mods(&wary.content);
    assert_eq!(mods.initiative_bonus, 4);
    let plain = w.state.player().talent_mods(&w.content);
    assert_eq!(plain.initiative_bonus, 0);
}

#[test]
fn a_hidden_trap_surprises_the_one_it_springs_on_and_nobody_beside() {
    let mut w = world(120, Rules::Difficulty(40));
    let (player, beside) = (w.state.player, goblin(&w, 2));
    lull(&mut w, beside);
    w.player_mut().pos = Pos::new(3, 1);
    // South onto the hidden alarm, the sleeping goblin beside it.
    w.apply(&Action::Move(Dir::S));
    assert_eq!(w.state.player().pos, Pos::new(3, 2));
    assert!(w.surprised(player), "the one it sprang on");
    let beside_surprised = w.surprised(beside);
    assert!(!beside_surprised, "not the one beside");
    // A known trap stepped onto surprises nobody.
    let mut w = world(120, Rules::Difficulty(40));
    let beside = goblin(&w, 2);
    lull(&mut w, beside);
    w.player_mut().pos = Pos::new(4, 1);
    w.apply(&Action::Move(Dir::S));
    assert!(!w.surprised(player));
}

#[test]
fn an_attack_on_the_unaware_is_rolled_flat_footed_and_surprises() {
    let mut w = world(120, Rules::Difficulty(40));
    let (player, far) = (w.state.player, goblin(&w, 8));
    // Behind the wall, it has not seen the player: struck (the reach aside), it is surprised
    // before the roll.
    let profile = w.state.player().melee(&w.content);
    let mut events = Vec::new();
    w.attack_with(
        player,
        far,
        crate::action::AttackKind::Melee,
        Some(profile),
        &mut events,
    );
    assert!(w.surprised(far));
    // An intent rolls nothing and surprises nobody.
    let mut w = world(120, Rules::Difficulty(40));
    let rng = w.state.rng.clone();
    w.attack_with(
        player,
        far,
        crate::action::AttackKind::Melee,
        None,
        &mut Vec::new(),
    );
    assert!(!w.surprised(far));
    assert_eq!(w.state.rng, rng);
}

#[test]
fn an_unseen_archer_surprises_once_until_the_memory_lapses() {
    // In the dark the player never sees the goblin (asleep: it takes no turn of its own); its
    // shots are struck at them straight.
    let mut w = world(0, Rules::Difficulty(40));
    let (player, archer) = (w.state.player, goblin(&w, 2));
    lull(&mut w, archer);
    let surprise = surprised_status(&w);
    let shoot = |w: &mut World| {
        let profile = w.state.actors[&archer].melee(&w.content);
        let kind = crate::action::AttackKind::Ranged;
        w.attack_with(archer, player, kind, Some(profile), &mut Vec::new());
        let surprised = w.surprised(player);
        // Healed after each shot: the goblin hits hard enough to end the test.
        let me = w.state.actors.get_mut(&player).expect("player");
        me.hp = me.hp.max(10);
        w.state
            .actors
            .get_mut(&player)
            .expect("player")
            .effects
            .remove(surprise);
        surprised
    };
    assert!(w.unaware(player, archer));
    assert!(shoot(&mut w), "the first shot out of the dark");
    let turn = w.state.turn;
    assert_eq!(
        w.state.actors[&archer].mind.noticed,
        Some(turn),
        "struck, they know"
    );
    assert!(!shoot(&mut w), "the next shot finds them aware");
    w.state.turn = turn + 20;
    assert!(!shoot(&mut w), "twenty turns on, still");
    w.state.turn = turn + 21;
    assert!(w.unaware(player, archer), "forgotten");
    assert!(shoot(&mut w), "surprised again");
}

#[test]
fn no_draws_while_nothing_is_unaware() {
    // A fight in plain sight, in the light: the same rolls with and without the rules.
    let actions = parse_actions("move:E*6 wait*4").expect("actions");
    let mut with = world(120, Rules::Shipped);
    let mut without = world(120, Rules::Without);
    for action in &actions {
        with.apply(action);
        without.apply(action);
        assert_eq!(with.state.rng, without.state.rng, "{action}");
        let hp = |w: &World| {
            w.state
                .actors
                .values()
                .map(|a| (a.id, a.hp))
                .collect::<Vec<_>>()
        };
        assert_eq!(hp(&with), hp(&without), "{action}");
    }
    // Only what the player noticed differs.
    let mut state = with.state.clone();
    for actor in state.actors.values_mut() {
        actor.mind.noticed = None;
    }
    assert_eq!(state.hash(), without.state.hash());
}

#[test]
fn what_the_player_noticed_survives_saves_and_replays() {
    let mut w = world(120, Rules::Shipped);
    let mut replay = Replay::new(&w);
    let actions = parse_actions("wait*3").expect("actions");
    for action in &actions {
        w.apply(action);
        replay.push(action, &w);
    }
    let loaded = crate::save::decode(&crate::save::encode(&w), w.content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    let replayed = replay.run(w.content.clone()).expect("the replay verifies");
    assert_eq!(replayed.state_hash(), w.state_hash());
}

/// The attackers of the `Attacked` events in `events`, in order (the player's own blows at
/// what they do not see are not among them).
fn attackers(events: &[Event]) -> Vec<ActorId> {
    (events.iter())
        .filter_map(|e| match e {
            Event::Attacked { attacker, .. } => Some(*attacker),
            _ => None,
        })
        .collect()
}

/// The player's blows at something unseen said so far in another's turn: held for the news.
fn answers(w: &World) -> usize {
    (w.news.iter())
        .filter(|t| t.starts_with("You ") && t.contains("something"))
        .count()
}

/// In the dark (the player sees nothing), the goblin beside them attacks once in melee, the
/// player holding Hair trigger if `hair_trigger`: the attackers of the events.
fn struck_in_the_dark(w: &mut World, hair_trigger: bool) -> Vec<ActorId> {
    let (player, id) = (w.state.player, goblin(w, 2));
    if hair_trigger {
        w.player_mut().talents.take("hair_trigger", None);
    }
    w.state.actors.get_mut(&id).expect("goblin").hp = 1000;
    assert!(w.unaware(player, id));
    let profile = w.state.actors[&id].melee(&w.content);
    let mut events = Vec::new();
    let kind = crate::action::AttackKind::Melee;
    w.attack_with(id, player, kind, Some(profile), &mut events);
    attackers(&events)
}

#[test]
fn hair_trigger_answers_a_made_check_with_a_blow() {
    let mut w = world(0, Rules::Difficulty(1));
    let (player, id) = (w.state.player, goblin(&w, 2));
    assert_eq!(struck_in_the_dark(&mut w, true), [id], "its attack goes on");
    assert!(!w.surprised(player), "the check was made");
    assert_eq!(answers(&w), 1, "{:?}", w.news);
    assert!(
        texts(&w)
            .iter()
            .any(|t| t == "You keep your wits about you."),
        "{:?}",
        texts(&w)
    );
    // One per surprise: struck, the player knows the goblin is there; its next attack finds
    // them aware, and nothing answers it.
    let profile = w.state.actors[&id].melee(&w.content);
    let mut events = Vec::new();
    let kind = crate::action::AttackKind::Melee;
    w.attack_with(id, player, kind, Some(profile), &mut events);
    assert_eq!(attackers(&events), [id]);
    assert_eq!(answers(&w), 1);
}

#[test]
fn no_blow_without_the_talent_a_made_check_or_an_attacker_beside() {
    // Without Hair trigger.
    let mut w = world(0, Rules::Difficulty(1));
    let id = goblin(&w, 2);
    assert_eq!(struck_in_the_dark(&mut w, false), [id]);
    assert_eq!(answers(&w), 0);
    // The check failed: the player is Surprised and strikes nothing.
    let mut w = world(0, Rules::Difficulty(40));
    assert_eq!(struck_in_the_dark(&mut w, true), [id]);
    assert!(w.surprised(w.state.player));
    assert_eq!(answers(&w), 0);
    // An archer behind the wall: not beside.
    let mut w = world(0, Rules::Difficulty(1));
    let (player, far) = (w.state.player, goblin(&w, 8));
    w.player_mut().talents.take("hair_trigger", None);
    let profile = w.state.actors[&far].melee(&w.content);
    let mut events = Vec::new();
    let kind = crate::action::AttackKind::Ranged;
    w.attack_with(far, player, kind, Some(profile), &mut events);
    assert_eq!(attackers(&events), [far]);
    assert_eq!(answers(&w), 0);
    // A hidden trap: nobody to strike, nothing drawn beyond the check.
    let mut w = world(120, Rules::Difficulty(1));
    w.player_mut().talents.take("hair_trigger", None);
    let player = w.state.player;
    let rng = after_a_check(&w);
    assert!(w.startle(player, None, &mut Vec::new()));
    assert_eq!(w.state.rng, rng);
}

#[test]
fn hair_trigger_may_cut_the_attacker_down_before_it_strikes() {
    // Some fate fells the goblin with the answering blow: its own attack is never rolled.
    let felled = (0..200).find_map(|seed| {
        let mut w = world(0, Rules::Difficulty(1));
        w.state.rng = crate::rng::Rng::gameplay(seed);
        let (player, id) = (w.state.player, goblin(&w, 2));
        w.player_mut().talents.take("hair_trigger", None);
        let profile = w.state.actors[&id].melee(&w.content);
        w.state.actors.get_mut(&id).expect("goblin").hp = 1;
        let mut events = Vec::new();
        let kind = crate::action::AttackKind::Melee;
        let blow = w.attack_with(id, player, kind, Some(profile), &mut events);
        (!w.state.actors.contains_key(&id)).then_some((blow, attackers(&events)))
    });
    let (blow, attackers) = felled.expect("some seed fells it");
    assert_eq!(blow, None, "nothing rolled");
    assert_eq!(attackers, [], "no attack of its");
}

#[test]
fn stalkers_eye_tells_who_is_unaware_and_nothing_leaks_without_it() {
    let unaware = |w: &mut World, id: ActorId| {
        *w.perception = crate::perception::Perception::compute(&w.state, &w.content, &mut w.lights);
        (w.perception().actors().iter())
            .find(|a| a.id == id)
            .expect("the goblin in view")
            .unaware
    };
    let mut w = world(120, Rules::Shipped);
    let id = goblin(&w, 2);
    // Beside the player in the light, it sees them.
    w.player_mut().talents.take("stalkers_eye", None);
    assert!(!unaware(&mut w, id));
    // Asleep, it has noticed no one.
    lull(&mut w, id);
    assert!(unaware(&mut w, id), "a sleeper is unaware");
    // Awake again and hunting the player, it is aware.
    let asleep = w.content.sleep_status().expect("asleep");
    let me = w.state.actors.get_mut(&id).expect("goblin");
    me.effects.remove(asleep);
    me.mind.last_seen = Some(Pos::new(1, 2));
    assert!(!unaware(&mut w, id));
    // Without the talent the snapshot never says.
    let mut plain = world(120, Rules::Shipped);
    lull(&mut plain, id);
    assert!(plain.unaware(id, plain.state.player));
    assert!(!unaware(&mut plain, id));
    assert!(plain.perception().actors().iter().all(|a| !a.unaware));
}

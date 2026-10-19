//! Skills in the world (spec §8 "Skills"): what trained skills give (tumble, stealth, listen,
//! balance, survival), that untrained ones give nothing and are never perceived, training marks
//! on use, saves, and replays.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::noise::{Noise, SoundKind};
use crate::object::Stack;
use crate::replay::Replay;
use crate::skills::{MAX_VALUE, SkillId, Trained};

/// A lit yard: the player in a corner, a peaceful rat standing in the far one.
const YARD: &str = r##"
name = "Yard"
ambient = [255, 255, 255, 0]
map = '''
################
#@.............#
#..............#
#..............#
#.............r#
################
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat", faction = "peaceful" }
"##;

/// A lit pit: the player beside a hostile rat.
const PIT: &str = r##"
name = "Pit"
ambient = [255, 255, 255, 0]
map = '''
#####
#@r.#
#####
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"##;

/// A dark room: a peaceful rat paces near the player, who cannot see it.
const DARK: &str = r##"
name = "Dark"
map = '''
##########
#@.......#
#........#
#...r....#
##########
'''
[legend]
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat", faction = "peaceful", route = "E2 W2" }
"##;

/// The built-in content plus the test levels, each with a scenario of its name whose player has
/// no character (average stats, untrained in everything); with `eager`, every mark rule's chance
/// is 100 and every mark is a point.
fn files(eager: bool) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    for (name, level) in [("yard", YARD), ("pit", PIT), ("dark", DARK)] {
        files.insert(&format!("levels/{name}.toml"), level);
        files.insert(
            &format!("scenarios/{name}.toml"),
            &format!("name = \"{name}\"\nstart_level = \"{name}\"\n"),
        );
    }
    if eager {
        let text = include_str!("../../../../assets/data/skills.toml");
        let eager: Vec<String> = text
            .lines()
            .map(|line| match line.split_once("chance = ") {
                Some((before, _)) => format!("{before}chance = 100 }}]"),
                None if line.starts_with("marks = ") => "marks = 1".into(),
                None if line.starts_with("soft_cap = ") => "soft_cap = 999".into(),
                None if line.starts_with("per = ") => "per = 999".into(),
                None => line.to_string(),
            })
            .collect();
        files.insert("data/skills.toml", &eager.join("\n"));
    }
    files
}

fn content(eager: bool) -> Arc<Content> {
    Arc::new(Content::load(&files(eager)).expect("test content is valid"))
}

fn world(scenario: &str) -> World {
    World::new(content(false), scenario, None).expect("world")
}

fn eager(scenario: &str) -> World {
    World::new(content(true), scenario, None).expect("world")
}

fn skill(w: &World, id: &str) -> SkillId {
    w.content().skill_id(id).expect("a skill")
}

/// The player trained in skill `id` at `value` (untrained with `None`).
fn set(w: &mut World, id: &str, value: Option<u16>) {
    let id = skill(w, id);
    let player = w.state.player;
    let skills = &mut w.state.actors.get_mut(&player).expect("player").skills;
    let mut all: Vec<(SkillId, Trained)> = skills.iter().filter(|(s, _)| *s != id).collect();
    if let Some(value) = value {
        all.push((id, Trained { value, marks: 0 }));
    }
    *skills = all.into_iter().collect();
}

fn value(w: &World, id: &str) -> Option<u16> {
    w.state.player().skills.value(skill(w, id))
}

fn said(w: &World, text: &str) -> bool {
    w.messages().iter().any(|m| m.text == text)
}

#[test]
fn the_player_perceives_only_their_own_trained_skills() {
    let mut w = world("yard");
    assert!(
        w.perception().own_skills().is_empty(),
        "untrained in everything"
    );
    set(&mut w, "stealth", Some(0));
    set(&mut w, "tumble", Some(42));
    let own: Vec<(String, u16)> = (w.perception().own_skills().iter())
        .map(|s| (w.content().skill(s.skill).id.clone(), s.value))
        .collect();
    // Trained at 0 is shown (it is not untrained); in skill order.
    assert_eq!(own, [("stealth".into(), 0), ("tumble".into(), 42)]);
    // A creature's skills are never perceived: the arena's shaman has some, the player none of
    // its.
    let arena = World::new(content(false), "arena", None).expect("world");
    let shaman = (arena.state.actors.values())
        .find(|a| arena.content().creature(a.creature).id == "kobold_shaman")
        .expect("a shaman");
    assert!(!shaman.skills.is_empty());
    let mine: Vec<SkillId> = (arena.perception().own_skills().iter())
        .map(|s| s.skill)
        .collect();
    let thief: Vec<SkillId> = arena
        .state
        .player()
        .skills
        .iter()
        .map(|(id, _)| id)
        .collect();
    assert_eq!(
        mine, thief,
        "the arena's halfling thief's own, nothing else"
    );
}

#[test]
fn tumble_adds_dodge_to_defence() {
    let mut w = world("yard");
    assert_eq!(
        w.perception().own_defence(),
        10,
        "average Dexterity, nothing worn"
    );
    for (tumble, defence) in [
        (None, 10),
        (Some(0), 10),
        (Some(19), 10),
        (Some(20), 11),
        (Some(60), 13),
        (Some(100), 15),
        (Some(125), 16),
        (Some(150), 17),
    ] {
        set(&mut w, "tumble", tumble);
        assert_eq!(w.perception().own_defence(), defence, "tumble {tumble:?}");
    }
    let defence = w.defence_of(w.state.player);
    assert_eq!(defence.bonus(crate::combat::Category::Dodge), 7);
}

#[test]
fn stealth_makes_steps_and_doors_quieter() {
    let mut w = world("yard");
    let loud = |w: &World, act| {
        let state = &w.state;
        noise::noise(
            &w.content,
            state.current(),
            state.player(),
            Burden::Unburdened,
            act,
        )
        .map_or(0, |n| n.loudness)
    };
    let step = Act::Step(Pos::new(2, 1));
    let door = Act::OpenDoor(Pos::new(0, 0));
    let fight = Act::Attack(crate::action::AttackKind::Melee);
    // The player's footsteps are 60 on stone floor; a wall makes no door noise, so a door's
    // 60 is checked through the closed door's own terrain below.
    for (stealth, steps) in [(None, 60), (Some(0), 60), (Some(50), 42), (Some(100), 24)] {
        set(&mut w, "stealth", stealth);
        assert_eq!(loud(&w, step), steps, "stealth {stealth:?}");
        assert_eq!(loud(&w, fight), 60, "attacks are as loud as ever");
        assert_eq!(loud(&w, door), 0);
    }
    set(&mut w, "stealth", Some(150));
    assert_eq!(loud(&w, step), 15, "75% quieter far past 100");
    // Without stealth the step is as loud as ever: the plain noise.
    let state = &w.state;
    let plain = noise::plain_noise(
        &w.content,
        state.current(),
        state.player(),
        Burden::Unburdened,
        step,
    );
    assert_eq!(plain.map(|n| n.loudness), Some(60));
    // A door: the hub's closed doors open with 60.
    let mut hub = World::new(content(false), "showcase", None).expect("world");
    let player = hub.state.player;
    hub.state.actors.get_mut(&player).expect("player").skills = Default::default();
    let closed = (0..hub.state.current().height())
        .flat_map(|y| (0..hub.state.current().width()).map(move |x| Pos::new(x, y)))
        .find(|&p| {
            let t = hub.state.current().terrain[p];
            hub.content.terrain(t).is_closed_door()
        })
        .expect("a closed door");
    assert_eq!(loud(&hub, Act::OpenDoor(closed)), 60);
    set(&mut hub, "stealth", Some(100));
    assert_eq!(loud(&hub, Act::OpenDoor(closed)), 24);
}

#[test]
fn listen_lowers_the_hearing_threshold() {
    let mut w = world("yard");
    let threshold = |w: &World| w.state.player().hearing(&w.content);
    for (listen, heard) in [
        (None, 10),
        (Some(9), 10),
        (Some(10), 9),
        (Some(50), 5),
        (Some(MAX_VALUE), 1),
    ] {
        set(&mut w, "listen", listen);
        assert_eq!(threshold(&w), Some(heard), "listen {listen:?}");
    }
    // A sound 6 loud where the player stands: unheard by plain ears, heard with listen 50.
    let rat = (w.state.actors.values())
        .find(|a| a.kind == ActorKind::Creature)
        .expect("rat")
        .id;
    let faint = Noise {
        kind: SoundKind::Footsteps,
        at: Pos::new(1, 1),
        loudness: 6,
        maker: rat,
    };
    set(&mut w, "listen", None);
    noise::make(&mut w.state, &w.content, faint);
    assert!(w.state.player().heard.is_empty());
    set(&mut w, "listen", Some(50));
    noise::make(&mut w.state, &w.content, faint);
    assert_eq!(w.state.player().heard.len(), 1);
    // The deer lists listen 30: it hears 3 fainter than its type's 5.
    let content = content(false);
    let deer = content.creature_id("deer").expect("deer");
    assert_eq!(content.creature(deer).hearing, Some(5));
    let mut actor = w.state.player().clone();
    actor.creature = deer;
    actor.skills = content.creature(deer).skills.clone();
    assert_eq!(actor.hearing(&content), Some(2));
}

#[test]
fn balance_eases_the_burdens_slowdown() {
    let mut w = world("yard");
    let player = w.state.player;
    let mail = w.content.object_id("chain_mail").expect("chain mail");
    // Two chain mail shirts: 22 kg on St 10 To 10, Burdened (over 15 kg).
    w.state.actors.get_mut(&player).expect("player").inventory = vec![
        Stack {
            object: mail,
            count: 1,
        };
        2
    ];
    assert_eq!(w.load_of(player).burden(), Burden::Burdened);
    for (balance, price) in [
        (None, 7500),
        (Some(0), 7500),
        (Some(50), 7050),
        (Some(100), 6600),
    ] {
        set(&mut w, "balance", balance);
        assert_eq!(
            w.price_of(player, ActionKind::Wait),
            price,
            "balance {balance:?}"
        );
    }
    // Strained (a third shirt and a fourth: 44 kg): ×1.5, and ×1.2 at balance 100.
    w.state.actors.get_mut(&player).expect("player").inventory = vec![
        Stack {
            object: mail,
            count: 1,
        };
        4
    ];
    assert_eq!(w.load_of(player).burden(), Burden::Strained);
    set(&mut w, "balance", None);
    assert_eq!(w.price_of(player, ActionKind::Wait), 9000);
    set(&mut w, "balance", Some(100));
    assert_eq!(w.price_of(player, ActionKind::Wait), 7200);
    // Unburdened, balance changes nothing.
    w.state
        .actors
        .get_mut(&player)
        .expect("player")
        .inventory
        .clear();
    assert_eq!(w.price_of(player, ActionKind::Wait), 6000);
}

#[test]
fn survival_shortens_journeys_on_the_overworld_only() {
    let mut w = World::new(content(false), "world", None).expect("world");
    assert!(
        value(&w, "survival").is_some(),
        "the world's ranger knows survival"
    );
    let player = w.state.player;
    let road = w.content.terrain_id("road").expect("road");
    let road = w.content.terrain(road).clone();
    for (survival, minutes) in [(None, 20 * 60), (Some(50), 18 * 60), (Some(100), 16 * 60)] {
        set(&mut w, "survival", survival);
        let price = w.step_price(player, &road);
        assert_eq!(price, i64::from(minutes) * 1000, "survival {survival:?}");
    }
    // Nothing off the overworld.
    let mut yard = world("yard");
    set(&mut yard, "survival", Some(100));
    let floor = yard.content.terrain_id("floor").expect("floor");
    let floor = yard.content.terrain(floor).clone();
    assert_eq!(yard.step_price(yard.state.player, &floor), 6000);
}

#[test]
fn sneaking_within_earshot_trains_stealth_and_untrained_skills_never_train() {
    let mut w = eager("yard");
    // Untrained: no marks, ever.
    w.state.actors.get_mut(&w.state.player).expect("player").pos = Pos::new(11, 4);
    w.apply(&Action::Move(Dir::E));
    assert_eq!(value(&w, "stealth"), None);
    // Trained, far from the rat (12 cells; a step carries 6): nothing to sneak past.
    let mut w = eager("yard");
    set(&mut w, "stealth", Some(0));
    w.apply(&Action::Move(Dir::E));
    assert_eq!(value(&w, "stealth"), Some(0));
    assert!(!said(&w, "Your stealth improves."));
    // Within earshot: a mark, and with every mark a point here, a point.
    w.state.actors.get_mut(&w.state.player).expect("player").pos = Pos::new(10, 4);
    w.apply(&Action::Move(Dir::E));
    assert_eq!(value(&w, "stealth"), Some(1));
    assert!(said(&w, "Your stealth improves."));
    // Waiting is no sneaking.
    w.apply(&Action::Wait);
    assert_eq!(value(&w, "stealth"), Some(1));
}

#[test]
fn hearing_something_new_trains_listen() {
    let mut w = eager("dark");
    set(&mut w, "listen", Some(5));
    let mut heard = 0;
    for _ in 0..6 {
        w.apply(&Action::Wait);
        heard += usize::from(!w.perception().sounds().is_empty());
    }
    assert!(heard > 0, "the pacing rat is heard");
    assert!(value(&w, "listen") > Some(5), "{:?}", value(&w, "listen"));
    assert!(said(&w, "Your listen improves."));
    // Sounds already known are no news and train nothing.
    let before = value(&w, "listen");
    w.apply(&Action::Wait);
    w.apply(&Action::Wait);
    assert_eq!(value(&w, "listen"), before);
}

#[test]
fn dodging_attacks_trains_tumble() {
    let mut w = eager("pit");
    // Defence 31 against a giant rat: it misses but for a natural 24, and the misses that would
    // have hit an unarmoured, motionless target are dodges.
    set(&mut w, "tumble", Some(500));
    for _ in 0..10 {
        w.apply(&Action::Wait);
    }
    assert!(value(&w, "tumble") > Some(500), "{:?}", value(&w, "tumble"));
    assert!(said(&w, "Your tumble improves."));
    // The dodges are said too.
    assert!(said(&w, "You dodge the giant rat's bite."));
}

#[test]
fn acting_under_a_load_trains_balance() {
    let mut w = eager("yard");
    set(&mut w, "balance", Some(0));
    let player = w.state.player;
    let mail = w.content.object_id("chain_mail").expect("chain mail");
    let pack = vec![
        Stack {
            object: mail,
            count: 1,
        };
        2
    ];
    w.state.actors.get_mut(&player).expect("player").inventory = pack.clone();
    w.apply(&Action::Wait);
    assert_eq!(value(&w, "balance"), Some(0), "waiting trains nothing");
    w.apply(&Action::Move(Dir::E));
    assert_eq!(value(&w, "balance"), Some(1));
    assert!(said(&w, "Your balance improves."));
    // Unburdened, steps train nothing.
    w.state
        .actors
        .get_mut(&player)
        .expect("player")
        .inventory
        .clear();
    w.apply(&Action::Move(Dir::E));
    assert_eq!(value(&w, "balance"), Some(1));
    // A dwarf's Burdened costs nothing, so it trains nothing either.
    let spec: CharacterSpec = "dwarf,male,monk".parse().expect("a character");
    let mut dwarf = World::with_character(content(true), "yard", None, Some(&spec)).expect("world");
    let start = value(&dwarf, "balance").expect("monks know balance");
    let player = dwarf.state.player;
    dwarf
        .state
        .actors
        .get_mut(&player)
        .expect("player")
        .inventory = pack;
    assert_eq!(dwarf.load_of(player).burden(), Burden::Burdened);
    dwarf.apply(&Action::Move(Dir::E));
    assert_eq!(value(&dwarf, "balance"), Some(start));
}

#[test]
fn journeys_train_survival() {
    let mut w = World::new(content(true), "world", None).expect("world");
    let start = value(&w, "survival").expect("the ranger knows survival");
    // Along the road and back.
    let mut moved = 0;
    for dir in [Dir::E, Dir::W, Dir::N, Dir::S] {
        let before = w.player_pos();
        w.apply(&Action::Move(dir));
        moved += usize::from(w.player_pos() != before);
    }
    assert!(moved > 0);
    assert_eq!(value(&w, "survival"), Some(start + moved as u16));
    assert!(said(&w, "Your survival improves."));
}

#[test]
fn training_draws_from_its_own_stream_and_marks_count_toward_the_curve() {
    // The built-in chances and curve: stealth gets a mark 15% of the time, 4 marks a point at 0.
    let mut w = world("yard");
    set(&mut w, "stealth", Some(0));
    let gameplay = w.state.rng.clone();
    let training = w.state.training.clone();
    for _ in 0..200 {
        w.train(crate::skills::Use::Sneak);
    }
    assert_eq!(w.state.rng, gameplay, "the gameplay stream is untouched");
    assert_ne!(w.state.training, training);
    // About 30 marks: some 7 points, with marks left over toward the next.
    let trained = w
        .state
        .player()
        .skills
        .get(skill(&w, "stealth"))
        .expect("trained");
    assert!((4..=11).contains(&trained.value), "{trained:?}");
    // Uses no skill trains on change nothing.
    let before = w.state.clone();
    w.train(crate::skills::Use::Travel);
    assert_eq!(w.state, before);
}

#[test]
fn skills_and_marks_survive_a_save_and_replays_reproduce_training() {
    let content = content(true);
    // The scenario's player knows nothing; a character knows its race's and class's skills.
    let plain = World::new(content.clone(), "yard", None).expect("world");
    assert!(plain.state.player().skills.is_empty());
    let spec: CharacterSpec = "halfling,female,thief".parse().expect("a character");
    let mut a = World::with_character(content.clone(), "yard", None, Some(&spec)).expect("world");
    let mut replay = Replay::new(&a);
    let actions = "move:E*9 move:S*3 move:E*2 wait move:W";
    for action in parse_actions(actions).expect("actions") {
        a.apply(&action);
        replay.push(&action, &a);
    }
    let stealth = value(&a, "stealth").expect("a thief knows stealth");
    assert!(said(&a, "Your stealth improves."), "{stealth}");
    let played = replay.run(content.clone()).expect("the replay verifies");
    assert_eq!(played.state(), a.state());
    let bytes = crate::save::encode(&a);
    let b = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(b.state(), a.state());
    assert_eq!(b.state_hash(), a.state_hash());
    assert_eq!(b.perception().own_skills(), a.perception().own_skills());
    // A state that names a skill the content does not have is refused, not trusted.
    let mut odd = a.state().clone();
    let player = odd.player;
    let unknown = SkillId(content.skills().len() as u8);
    (odd.actors.get_mut(&player).expect("player").skills).add(unknown, 5);
    let err = World::from_state(content, odd).unwrap_err();
    assert!(err.to_string().contains("knows skills"), "{err}");
}

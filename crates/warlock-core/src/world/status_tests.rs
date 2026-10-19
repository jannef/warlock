//! Status effects and healing in the world (spec §8 "Status effects", "Healing"): wounds bleed,
//! bites poison, effects tick and ease by game time however it is cut up, slow healing, lower
//! stats, kill, and are said, saved, and replayed.

use super::*;
use crate::action::{AttackKind, parse_actions};
use crate::combat::{Blow, Strike};
use crate::content::{ContentFiles, StatusId};
use crate::replay::Replay;
use crate::stats::Stat;
use crate::status::Effect;

const TERRAIN: &str = r##"
[[terrain]]
id = "floor"
name = "floor"
glyph = "."
fg = "#808080"

[[terrain]]
id = "wall"
name = "wall"
glyph = "#"
fg = "#a0a0a0"
walkable = false
transparent = false

[[terrain]]
id = "stairs_down"
name = "staircase down"
glyph = ">"
fg = "#ffffff"
stairs = "down"

[[terrain]]
id = "stairs_up"
name = "staircase up"
glyph = "<"
fg = "#ffffff"
stairs = "up"
"##;

const FACTIONS: &str = r##"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "vermin"
name = "vermin"
hostile = ["player"]

[[faction]]
id = "kobolds"
name = "kobolds"
hostile = ["player", "vermin"]
"##;

// The rat is frail; the brute hits hard (every blow a big hit on a small target); the biter's
// bite always poisons, the nipper's half the time.
const CREATURES: &str = r##"
[[creature]]
id = "player"
name = "you"
glyph = "@"
fg = "#ffffff"
weight = 70000

[[creature]]
id = "rat"
name = "giant rat"
glyph = "r"
fg = "#806040"
weight = 4000
faction = "vermin"
stats = { toughness = 4 }
attack = { damage = "1d2", name = "bite", verb = "bites" }
level = 1
xp = 5

[[creature]]
id = "brute"
name = "kobold brute"
glyph = "k"
fg = "#60a040"
weight = 30000
faction = "kobolds"
stats = { strength = 30, toughness = 20 }
base_hp = 20

[[creature]]
id = "biter"
name = "biter"
glyph = "b"
fg = "#60a040"
weight = 3000
faction = "vermin"
stats = { strength = 1, dexterity = 30 }
attack = { damage = "1d2", name = "bite", verb = "bites", on_hit = [{ status = "poisoned", chance = 100, severity = 4 }] }

[[creature]]
id = "nipper"
name = "nipper"
glyph = "n"
fg = "#60a040"
weight = 3000
faction = "vermin"
stats = { strength = 1, dexterity = 30 }
attack = { damage = "1d2", name = "bite", verb = "bites", on_hit = [{ status = "poisoned", chance = 50, severity = 1 }] }
"##;

// Round numbers: bleeding costs a hit point every 10 s per degree and eases a degree every 30 s;
// poison a hit point every 60 s per degree, easing every 60 s, and weakens from 4 degrees on.
// Weariness only halves healing, until cured.
const STATUS: &str = r##"
[healing]
resting = 200

[[status]]
id = "bleeding"
name = "Bleeding"
seen = "bleeding"
max = 5
tick = 10
damage = 1
eases = 30
regen = 0
wound = { big_hit = 1, critical = 2 }
cured_by = ["bandage"]
start = "You are bleeding."
end = "The bleeding stops."
dies = "bleeds to death"
death = "Bled to death"

[[status]]
id = "poisoned"
name = "Poisoned"
max = 10
tick = 60
damage = 1
eases = 60
regen = 50
penalty = { from = 4, strength = -1, dexterity = -1 }
start = "You feel sick."
end = "The poison wears off."

[[status]]
id = "weary"
name = "Weary"
max = 1
regen = 50
"##;

const LEGEND: &str = r##"
[legend]
"." = "floor"
"#" = "wall"
">" = { terrain = "stairs_down", to = "below" }
"@" = { terrain = "floor", spawn = "player" }
"r" = { terrain = "floor", creature = "rat" }
"k" = { terrain = "floor", creature = "brute" }
"b" = { terrain = "floor", creature = "biter" }
"n" = { terrain = "floor", creature = "nipper" }
"##;

/// The test content with a level "fight" on `map` (fully lit) and a scenario "fight" with
/// `scenario` lines; `extra` files added or replaced.
fn files(map: &str, scenario: &str, extra: &[(&str, &str)]) -> ContentFiles {
    let mut files = ContentFiles::default();
    for (path, text) in [
        ("data/terrain.toml", TERRAIN),
        ("data/factions.toml", FACTIONS),
        ("data/creatures.toml", CREATURES),
        ("data/status.toml", STATUS),
        ("data/legend.toml", LEGEND),
        (
            "levels/below.toml",
            "name = \"Below\"\nambient = [255, 255, 255, 0]\nmap = '''\n####\n#<.#\n####\n'''\n\
             [legend]\n\"<\" = { terrain = \"stairs_up\", to = \"fight\" }\n",
        ),
    ] {
        files.insert(path, text);
    }
    files.insert(
        "levels/fight.toml",
        &format!("name = \"The Pit\"\nambient = [255, 255, 255, 0]\nmap = '''\n{map}'''\n"),
    );
    files.insert(
        "scenarios/fight.toml",
        &format!("name = \"Fight\"\nstart_level = \"fight\"\n{scenario}\n"),
    );
    for (path, text) in extra {
        files.insert(path, text);
    }
    files
}

fn world_with(map: &str, scenario: &str, extra: &[(&str, &str)]) -> World {
    let content = Content::load(&files(map, scenario, extra)).expect("test content is valid");
    World::new(Arc::new(content), "fight", None).expect("world")
}

fn world_on(map: &str, scenario: &str) -> World {
    world_with(map, scenario, &[])
}

/// A room to oneself.
const ALONE: &str = "#####\n#@..#\n#####\n";

fn status(w: &World, id: &str) -> StatusId {
    w.content().status_id(id).expect(id)
}

fn effect(w: &World, actor: ActorId, id: &str) -> Option<Effect> {
    let status = status(w, id);
    (w.state.actors.get(&actor)).and_then(|a| a.effects.get(status).copied())
}

fn afflict(w: &mut World, actor: ActorId, id: &str, severity: u16, by: Option<ActorId>) {
    let status = status(w, id);
    w.afflict(actor, status, severity, by, &mut Vec::new());
}

fn hp(w: &World) -> i32 {
    w.state().player().hp
}

fn set_hp(w: &mut World, hp: i32) {
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").hp = hp;
}

fn run(world: &mut World, actions: &str) -> Vec<Outcome> {
    parse_actions(actions)
        .expect("actions")
        .iter()
        .map(|a| world.apply(a))
        .collect()
}

fn messages(world: &World) -> Vec<String> {
    world.messages().iter().map(|m| m.display()).collect()
}

fn count(world: &World, text: &str) -> usize {
    (world.messages().iter())
        .filter(|m| m.display().starts_with(text))
        .count()
}

#[test]
fn bleeding_costs_hit_points_by_game_time_and_eases_away() {
    // Toughness 10 and 30 more: 40 hit points.
    let mut w = world_on(ALONE, "hp = 30");
    let player = w.player_id();
    afflict(&mut w, player, "bleeding", 2, None);
    assert_eq!(effect(&w, player, "bleeding").map(|e| e.severity), Some(2));
    // Two degrees for 30 s (a hit point every 5 s: 6), then one for 30 s (every 10 s: 3).
    run(&mut w, "wait*5");
    assert_eq!(hp(&w), 34, "30 s at two degrees");
    assert_eq!(effect(&w, player, "bleeding").map(|e| e.severity), Some(1));
    run(&mut w, "wait*4");
    assert_eq!(hp(&w), 32);
    assert!(effect(&w, player, "bleeding").is_some());
    let outs = run(&mut w, "wait");
    assert_eq!(hp(&w), 31, "nine in all");
    assert_eq!(effect(&w, player, "bleeding"), None, "it stopped");
    assert_eq!(count(&w, "The bleeding stops."), 1);
    let ended = Event::Status {
        actor: player,
        status: status(&w, "bleeding"),
        began: false,
    };
    assert!(outs[0].events.contains(&ended));
    // No hit point came back while it bled; now they come back again from scratch.
    assert_eq!(w.state().player().regen, 0);
    run(&mut w, "wait*40");
    assert_eq!(hp(&w), 32, "240 s later, one back");
}

#[test]
fn time_cut_up_differently_ends_the_same() {
    // Waits of 6 s against waits of a minute (cost 10000): two minutes of bleeding and poison
    // in a hurt body, healing as the bleeding stops and the poison lets it.
    let mut quick = world_on(ALONE, "hp = 30");
    let mut slow = world_with(ALONE, "hp = 30", &[("data/actions.toml", "wait = 10000\n")]);
    for w in [&mut quick, &mut slow] {
        let player = w.player_id();
        set_hp(w, 30);
        afflict(w, player, "bleeding", 3, None);
        afflict(w, player, "poisoned", 2, None);
    }
    run(&mut quick, "wait*30");
    run(&mut slow, "wait*3");
    assert_eq!(quick.time(), slow.time(), "three minutes each");
    let (a, b) = (quick.state().player(), slow.state().player());
    assert_eq!((a.hp, a.regen, &a.effects), (b.hp, b.regen, &b.effects));
    assert!(a.hp < 30, "they cost hit points");
    // A long journey through it in one action too: an hour's rest at once against waits.
    let mut rested = world_on(ALONE, "hp = 30");
    let player = rested.player_id();
    afflict(&mut rested, player, "poisoned", 3, None);
    set_hp(&mut rested, 20);
    rested.apply(&Action::Rest(Rest::For(Duration::minutes(2))));
    let mut waited = world_on(ALONE, "hp = 30");
    afflict(&mut waited, player, "poisoned", 3, None);
    set_hp(&mut waited, 20);
    run(&mut waited, "wait*20");
    let (a, b) = (rested.state().player(), waited.state().player());
    assert_eq!(
        (a.effects.clone(), rested.time()),
        (b.effects.clone(), waited.time())
    );
    assert!(a.hp >= b.hp, "resting heals at least as fast");
}

#[test]
fn big_hits_and_criticals_make_their_target_bleed() {
    // 30 hit points: the brute's fists (1d3 + 10) take a third of them every time.
    let mut w = world_on("#####\n#@k.#\n#####\n", "hp = 20");
    let bleeding = status(&w, "bleeding");
    let player = w.player_id();
    let mut hits = 0;
    for _ in 0..30 {
        let out = w.apply(&Action::Wait);
        hits += (out.events.iter())
            .filter(|e| matches!(e, Event::Attacked { strike: Some(s), .. } if s.is_hit()))
            .count();
        if hits > 0 {
            assert!(
                out.events.contains(&Event::Status {
                    actor: player,
                    status: bleeding,
                    began: true
                }) || effect(&w, player, "bleeding").is_some()
            );
            break;
        }
    }
    assert_eq!(hits, 1, "the brute hit");
    let shown = messages(&w);
    let i = shown.iter().position(|m| m == "You are bleeding.");
    let i = i.expect("the player is told");
    assert!(
        shown[i - 1].starts_with("The kobold brute hits you"),
        "{shown:?}"
    );
    let e = effect(&w, player, "bleeding").expect("bleeding");
    assert!((1..=2).contains(&e.severity), "a big hit, or a critical");
    assert_eq!(e.by, Some(ActorId(1)), "the brute's doing");
    // Unit rules: a big hit takes at least a third; a critical bleeds more.
    assert!(crate::combat::big_hit(10, 30) && !crate::combat::big_hit(9, 30));
    assert!(crate::combat::big_hit(1, 1) && !crate::combat::big_hit(0, 1));
    let blow = |critical, damage| Blow {
        strike: Strike::Hit { critical },
        damage,
    };
    let mut w = world_on("#####\n#@r.#\n#####\n", "hp = 1000");
    let rat = ActorId(1);
    w.after_blow(
        player,
        rat,
        AttackKind::Melee,
        &blow(false, 1),
        &mut Vec::new(),
    );
    assert_eq!(effect(&w, rat, "bleeding"), None, "a scratch");
    w.after_blow(
        player,
        rat,
        AttackKind::Melee,
        &blow(false, 2),
        &mut Vec::new(),
    );
    assert_eq!(effect(&w, rat, "bleeding").map(|e| e.severity), Some(1));
    w.after_blow(
        player,
        rat,
        AttackKind::Thrown,
        &blow(true, 1),
        &mut Vec::new(),
    );
    let e = effect(&w, rat, "bleeding").expect("worse");
    assert_eq!((e.severity, e.by), (3, Some(player)), "a critical adds two");
    // Severity stacks to the most there is.
    for _ in 0..5 {
        w.after_blow(
            player,
            rat,
            AttackKind::Melee,
            &blow(true, 3),
            &mut Vec::new(),
        );
    }
    assert_eq!(effect(&w, rat, "bleeding").map(|e| e.severity), Some(5));
    // A creature's bleeding shows; the player sees it in look mode (after a turn: perception).
    w.apply(&Action::Wait);
    assert!(w.state.actors.contains_key(&rat), "still alive");
    let shows: Vec<StatusId> = w.perception().effects_of(rat).collect();
    assert_eq!(shows, [status(&w, "bleeding")]);
}

#[test]
fn bites_poison_by_their_chance_drawing_only_below_a_sure_thing() {
    let blow = Blow {
        strike: Strike::Hit { critical: false },
        damage: 1,
    };
    // The biter always poisons: no draw.
    let mut w = world_on("#####\n#@b.#\n#####\n", "hp = 1000");
    let (player, biter) = (w.player_id(), ActorId(1));
    let rng = w.state.rng.clone();
    w.after_blow(biter, player, AttackKind::Melee, &blow, &mut Vec::new());
    assert_eq!(w.state.rng, rng, "nothing drawn at 100 %");
    assert_eq!(effect(&w, player, "poisoned").map(|e| e.severity), Some(4));
    // Only its bite: a shot or a spell by it carries nothing.
    let mut w = world_on("#####\n#@b.#\n#####\n", "hp = 1000");
    w.after_blow(biter, player, AttackKind::Ranged, &blow, &mut Vec::new());
    assert_eq!(effect(&w, player, "poisoned"), None);
    // The nipper poisons half the time: one draw per hit, the same for the same seed.
    let nips = |seed: u64| {
        let files = files("#####\n#@n.#\n#####\n", "hp = 1000", &[]);
        let content = Arc::new(Content::load(&files).expect("content"));
        let mut w = World::new(content, "fight", Some(seed)).expect("world");
        (0..40)
            .map(|_| {
                let mut expected = w.state.rng.clone();
                let lands = expected.chance(50);
                let poisoned = status(&w, "poisoned");
                let me = w.state.actors.get_mut(&player).expect("player");
                me.effects.remove(poisoned);
                w.after_blow(
                    ActorId(1),
                    player,
                    AttackKind::Melee,
                    &blow,
                    &mut Vec::new(),
                );
                assert_eq!(w.state.rng, expected, "one draw");
                assert_eq!(effect(&w, player, "poisoned").is_some(), lands);
                lands
            })
            .collect::<Vec<bool>>()
    };
    let (a, b) = (nips(7), nips(7));
    assert_eq!(a, b, "deterministic");
    assert!(a.contains(&true) && a.contains(&false), "{a:?}");
    assert_ne!(a, nips(8));
}

#[test]
fn a_poisonous_bite_is_said_after_the_bite_and_weakens() {
    let mut w = world_on("#####\n#@b.#\n#####\n", "hp = 1000");
    let player = w.player_id();
    let before = w.perception().own_scores();
    let (dv, limit) = (
        w.perception().own_defence(),
        w.perception().own_load().limit,
    );
    for _ in 0..30 {
        w.apply(&Action::Wait);
        if effect(&w, player, "poisoned").is_some() {
            break;
        }
    }
    let shown = messages(&w);
    let i = shown.iter().position(|m| m == "You feel sick.");
    let i = i.expect("the player is told");
    assert!(shown[i - 1].starts_with("The biter bites you"), "{shown:?}");
    assert_eq!(count(&w, "You feel sick."), 1);
    // Four degrees: Strength and Dexterity are a point lower to every rule.
    let now = w.perception().own_scores();
    for stat in Stat::ALL {
        let lower = matches!(stat, Stat::Strength | Stat::Dexterity);
        assert_eq!(now[stat], before[stat] - u8::from(lower), "{stat:?}");
    }
    assert_eq!(w.perception().own_base_scores(), before);
    assert_eq!(w.perception().own_defence(), dv - 1, "Dexterity 9");
    assert!(
        w.perception().own_load().limit < limit,
        "Strength 9 carries less"
    );
    let me = w.state().player();
    assert_eq!(me.melee(w.content()).to_hit, -1, "Strength 9 to hit");
    assert_eq!(
        me.stats.score(Stat::Strength),
        10,
        "the score itself is untouched"
    );
}

#[test]
fn poison_wears_off_and_its_weakness_with_it() {
    let mut w = world_on(ALONE, "hp = 1000");
    let player = w.player_id();
    afflict(&mut w, player, "poisoned", 3, None);
    assert_eq!(
        w.perception().own_scores()[Stat::Strength],
        10,
        "weak poison"
    );
    afflict(&mut w, player, "poisoned", 2, None);
    assert_eq!(effect(&w, player, "poisoned").map(|e| e.severity), Some(5));
    assert_eq!(w.perception().own_scores()[Stat::Strength], 9);
    // A degree a minute: strong for two more minutes, gone after five.
    run(&mut w, "wait*20");
    assert_eq!(effect(&w, player, "poisoned").map(|e| e.severity), Some(3));
    assert_eq!(w.perception().own_scores()[Stat::Strength], 10);
    run(&mut w, "wait*30");
    assert_eq!(effect(&w, player, "poisoned"), None);
    assert_eq!(count(&w, "The poison wears off."), 1);
    // A hit point a minute per degree: 5 + 4 + 3 + 2 + 1.
    assert_eq!(hp(&w), w.perception().own_hp().1 - 15);
}

#[test]
fn healing_is_twice_as_fast_resting_and_slower_or_none_with_statuses() {
    // Toughness 10: a hit point every 240 s, 15 an hour.
    let hour = |rest: bool, status: Option<&str>| {
        let mut w = world_on(ALONE, "hp = 1000");
        let player = w.player_id();
        set_hp(&mut w, 100);
        if let Some(id) = status {
            afflict(&mut w, player, id, 1, None);
        }
        if rest {
            w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
        } else {
            run(&mut w, "wait*600");
        }
        hp(&w) - 100
    };
    assert_eq!(hour(false, None), 15);
    assert_eq!(hour(true, None), 30, "twice as fast resting");
    assert_eq!(hour(false, Some("weary")), 7, "half as fast");
    assert_eq!(hour(true, Some("weary")), 15, "the two together");
    // Bleeding stops it while it lasts (half a minute here), then it costs and heals as usual.
    let mut w = world_on(ALONE, "hp = 1000");
    let player = w.player_id();
    set_hp(&mut w, 100);
    afflict(&mut w, player, "bleeding", 1, None);
    assert_eq!(w.healing_rate(player), 0);
    run(&mut w, "wait*5");
    assert_eq!((hp(&w), w.state().player().regen), (97, 0));
    assert_eq!(w.healing_rate(player), 100);
}

#[test]
fn creatures_bleed_to_death_where_the_player_sees_it() {
    let mut w = world_on("#####\n#@.r#\n#####\n", "hp = 1000");
    let (player, rat) = (w.player_id(), ActorId(1));
    // Four hit points against five degrees (a hit point every 2 s): dead within 8 s.
    afflict(&mut w, rat, "bleeding", 5, Some(player));
    let mut died = None;
    for _ in 0..3 {
        let out = w.apply(&Action::Wait);
        if let Some(e) = (out.events.iter()).find(|e| matches!(e, Event::Died { .. })) {
            died = Some(e.clone());
            break;
        }
    }
    let bleeding = status(&w, "bleeding");
    assert!(
        matches!(died, Some(Event::Died { actor, of: Some(s), .. }) if actor == rat && s == bleeding),
        "{died:?}"
    );
    assert!(!w.state.actors.contains_key(&rat), "gone");
    assert_eq!(count(&w, "The giant rat bleeds to death."), 1);
}

#[test]
fn a_creature_bled_to_death_by_the_players_wound_is_their_kill() {
    let experience = include_str!("../../../../assets/data/experience.toml");
    let bleed_out = |by: Option<ActorId>| {
        let extra = [("data/experience.toml", experience)];
        let mut w = world_with("#####\n#@.r#\n#####\n", "hp = 1000", &extra);
        let rat = ActorId(1);
        afflict(&mut w, rat, "bleeding", 5, by);
        for _ in 0..5 {
            w.apply(&Action::Wait);
        }
        assert!(!w.state.actors.contains_key(&rat), "it bled to death");
        w.player_xp()
    };
    let player = Some(ActorId(0));
    assert_eq!(bleed_out(player), 5, "the player's wound: their kill");
    assert_eq!(
        bleed_out(Some(ActorId(7))),
        0,
        "another's wound: not theirs"
    );
    assert_eq!(bleed_out(None), 0);
}

#[test]
fn the_player_bleeds_to_death_at_the_moment_it_happens() {
    let mut w = world_on(ALONE, "hp = 1000");
    let player = w.player_id();
    set_hp(&mut w, 2);
    afflict(&mut w, player, "bleeding", 5, None);
    let start = w.time();
    let out = w.apply(&Action::Wait);
    let death = w.death().expect("bled out").clone();
    assert_eq!(death.of, Some(status(&w, "bleeding")));
    assert_eq!(death.by, None);
    assert_eq!(
        death.time,
        start + Duration::seconds(4),
        "two hit points, one every 2 s"
    );
    assert!(
        out.events
            .iter()
            .any(|e| matches!(e, Event::Died { actor, .. } if *actor == player))
    );
    assert_eq!(messages(&w).last().map(String::as_str), Some("You die..."));
    // Saved, it is still the same death.
    let bytes = crate::save::encode(&w);
    let loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.death(), Some(&death));
}

#[test]
fn a_rest_goes_on_when_a_status_ends_and_says_so_once() {
    let mut w = world_on(ALONE, "hp = 1000");
    let player = w.player_id();
    set_hp(&mut w, 100);
    afflict(&mut w, player, "bleeding", 1, None);
    w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    assert_eq!(count(&w, "The bleeding stops."), 1);
    assert_eq!(
        messages(&w).last().map(String::as_str),
        Some("You rest for 1 hour.")
    );
    // Three lost in its 30 s, then 3570 s of rest at twice the pace: 29 back (240 s each).
    assert_eq!(hp(&w), 100 - 3 + 29);
}

#[test]
fn the_dev_skip_and_other_levels_pass_no_status_time() {
    let mut w = world_on("#####\n#@>r#\n#####\n", "hp = 1000");
    let (player, rat) = (w.player_id(), ActorId(1));
    afflict(&mut w, player, "poisoned", 2, None);
    afflict(&mut w, rat, "poisoned", 2, None);
    let before = effect(&w, player, "poisoned");
    run(&mut w, "skip:1h");
    assert_eq!(
        effect(&w, player, "poisoned"),
        before,
        "nobody lives through a skip"
    );
    // Downstairs, the rat's poison waits for the player to come back.
    run(&mut w, "move:E descend");
    let rat_before = effect(&w, rat, "poisoned");
    run(&mut w, "wait*30");
    assert_eq!(effect(&w, rat, "poisoned"), rat_before);
    assert_eq!(
        effect(&w, player, "poisoned"),
        None,
        "the player's own wore off"
    );
}

#[test]
fn a_bandage_stops_the_bleeding_and_nothing_else() {
    let mut w = world_on(ALONE, "hp = 1000");
    let player = w.player_id();
    afflict(&mut w, player, "bleeding", 3, None);
    afflict(&mut w, player, "poisoned", 3, None);
    let mut events = Vec::new();
    assert!(w.cure(player, "bandage", &mut events));
    assert_eq!(effect(&w, player, "bleeding"), None);
    assert!(effect(&w, player, "poisoned").is_some());
    assert_eq!(
        events,
        [Event::Status {
            actor: player,
            status: status(&w, "bleeding"),
            began: false
        }]
    );
    assert!(
        !w.cure(player, "bandage", &mut events),
        "nothing left to bandage"
    );
    assert!(!w.cure(player, "antidote", &mut events));
}

#[test]
fn the_player_knows_their_own_statuses_but_sees_only_some_on_others() {
    let mut w = world_on("#####\n#@r.#\n#####\n", "hp = 1000");
    let (player, rat) = (w.player_id(), ActorId(1));
    afflict(&mut w, player, "poisoned", 1, None);
    afflict(&mut w, player, "bleeding", 1, None);
    afflict(&mut w, rat, "poisoned", 1, None);
    w.apply(&Action::Wait);
    let names = |w: &World| -> Vec<String> {
        (w.perception().own_effects())
            .map(|s| w.content().status(s).name.clone())
            .collect()
    };
    assert_eq!(
        names(&w),
        ["Bleeding", "Poisoned"],
        "the most important first"
    );
    assert_eq!(
        w.perception().effects_of(rat).count(),
        0,
        "poison does not show"
    );
    afflict(&mut w, rat, "bleeding", 1, None);
    w.apply(&Action::Wait);
    let shows: Vec<StatusId> = w.perception().effects_of(rat).collect();
    assert_eq!(shows, [status(&w, "bleeding")]);
    // The stats dump names them, never how bad they are.
    let rows = crate::dump::layer_rows(&w, crate::dump::Layer::Stats);
    assert!(
        rows.contains(&"Effects Bleeding, Poisoned".to_string()),
        "{rows:?}"
    );
}

#[test]
fn status_effects_are_saved_and_in_the_state_hash() {
    let mut w = world_on("#####\n#@r.#\n#####\n", "hp = 1000");
    let (player, rat) = (w.player_id(), ActorId(1));
    let plain = w.state_hash();
    afflict(&mut w, rat, "bleeding", 2, Some(player));
    afflict(&mut w, player, "poisoned", 6, None);
    assert_ne!(w.state_hash(), plain);
    run(&mut w, "wait*3");
    let bytes = crate::save::encode(&w);
    let mut loaded = crate::save::decode(&bytes, w.content().clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    run(&mut w, "wait*20");
    run(&mut loaded, "wait*20");
    assert_eq!(loaded.state_hash(), w.state_hash());
    // A save whose effects this content cannot have is refused.
    let mut state = w.state().clone();
    let me = state.actors.get_mut(&player).expect("player");
    me.effects = Default::default();
    me.effects
        .add(StatusId(0), w.content().status(StatusId(0)), 1, None);
    me.effects.iter_mut().for_each(|e| e.status = StatusId(9));
    assert!(matches!(
        World::from_state(w.content().clone(), state),
        Err(WorldError::InconsistentState(_))
    ));
}

#[test]
fn fights_with_wounds_and_poison_replay_exactly() {
    let map = "#######\n#@k.n.#\n#..b..#\n#######\n";
    let content = Arc::new(Content::load(&files(map, "hp = 1000", &[])).expect("content"));
    let mut w = World::new(content.clone(), "fight", None).expect("world");
    let mut replay = Replay::new(&w);
    let mut statuses = 0;
    for action in parse_actions("wait*30 rest:1h wait*5").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
        statuses += w.state().player().effects.iter().count();
    }
    assert!(statuses > 0, "something ailed the player");
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content)
        .expect("the replay verifies");
    assert_eq!(verified.state_hash(), w.state_hash());
}

#[test]
fn the_built_in_rat_may_poison_and_wounds_may_bleed() {
    let content = Content::builtin().expect("builtin content");
    let rat = content.creature_id("rat").expect("rat");
    let bite = content.creature(rat).attack.as_ref().expect("a bite");
    let poisoned = content.status_id("poisoned").expect("poisoned");
    assert!(
        bite.on_hit
            .iter()
            .any(|o| o.status == poisoned && o.chance < 100),
        "a small chance"
    );
    let bleeding = content.status_id("bleeding").expect("bleeding");
    let def = content.status(bleeding);
    assert!(def.wound.is_some() && def.seen.is_some() && def.regen == 0);
    assert!(def.cured_by.iter().any(|c| c == "bandage"));
    assert_eq!(content.healing().resting, 200);
    // Bleeding first: it is the more urgent.
    assert!(bleeding < poisoned);
}

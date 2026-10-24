//! Mana in the world (spec §8 "Mana"): the maximum from data and stats, regeneration exact
//! however time is cut and changed by resting, statuses, diseases, rooms, and starving (and by
//! nothing else), the maximum falling, spending, levels, saves, and replays.

use super::*;
use crate::action::parse_actions;
use crate::content::ContentFiles;
use crate::replay::Replay;

// Round numbers: resting brings mana back three times as fast; dazed halves the pace; a hex
// drains 2 mana a tick (every 30 s a degree) and stops it, easing a degree a minute; befuddled
// takes 4 Intelligence for a minute; Diseased halves it.
const STATUS: &str = r##"
[healing]
resting = 200
mana_resting = 300

[[status]]
id = "poisoned"
name = "Poisoned"
max = 10
tick = 60
damage = 1
eases = 60
regen = 50
start = "You feel sick."
end = "The poison wears off."

[[status]]
id = "dazed"
name = "Dazed"
max = 1
mana_regen = 50

[[status]]
id = "hexed"
name = "Hexed"
max = 5
tick = 30
mana_damage = 2
eases = 60
mana_regen = 0
start = "You are hexed."
end = "The hex lifts."

[[status]]
id = "befuddled"
name = "Befuddled"
max = 1
eases = 60
penalty = { from = 1, intelligence = -4 }

[[status]]
id = "diseased"
name = "Diseased"
max = 1
regen = 50
mana_regen = 50
"##;

// A fog of the mind: it shows after a day and slows mana to a tenth; a progression (every
// day, against the hardest check) takes Intelligence and Willpower, never by itself.
const FOG: &str = r##"
[[disease]]
id = "fog"
name = "the fog"
difficulty = 50
check = "willpower"
incubation = 1
damage = { intelligence = 4, willpower = 2 }
recovery = 0
mana_regen = 10
onset = "Fog fills your head."
symptoms = ["The fog thickens."]
recovered = "The fog lifts."
"##;

// From level 3 on, levels add no mana.
const EXPERIENCE: &str = "max_level = 50\nhp_level = 20\nmana_level = 3\nbonus_percent = 5\n\
                          bonus_levels = 4\n[kills]\nplateau = 213\nramp = 5\n\
                          slope_per_100 = 285\nguideline = { per_level = 5, base = 10 }\n\
                          [[zero_difference]]\nfrom = 1\nzd = 5\n";

// A lit hall; its east part (walls included) is a mana well.
const LEVEL: &str = r##"
name = "Hall"
ambient = [255, 255, 255, 0]
map = '''
#########
#@......#
#########
'''
[[room]]
effect = "well"
area = [5, 0, 8, 2]
[legend]
"@" = { terrain = "floor", spawn = "player" }
"##;

const WELL: &str = "\n[[effect]]\nid = \"well\"\nname = \"mana well\"\nmark = \"w\"\n\
                    enter = \"The air hums.\"\nmana_regen = 300\n";

/// The built-in content with the statuses, disease, experience, and room above, and a scenario
/// "hall" without a character with `scenario`'s lines; `extra` files added or replaced.
fn files(scenario: &str, extra: &[(&str, &str)]) -> ContentFiles {
    let mut files = ContentFiles::builtin();
    files.insert("data/status.toml", STATUS);
    // The talents name the built-in statuses, which STATUS replaces.
    files.remove("data/talents/");
    files.remove("data/talents.toml");
    let diseases = include_str!("../../../../assets/data/diseases.toml");
    files.insert("data/diseases.toml", &format!("{diseases}\n{FOG}"));
    files.insert("data/experience.toml", EXPERIENCE);
    let rooms = include_str!("../../../../assets/data/room_effects.toml");
    files.insert("data/room_effects.toml", &format!("{rooms}{WELL}"));
    files.insert("levels/hall.toml", LEVEL);
    files.insert(
        "scenarios/hall.toml",
        &format!("name = \"Hall\"\nstart_level = \"hall\"\nhp = 1000\n{scenario}\n"),
    );
    for (path, text) in extra {
        files.insert(path, text);
    }
    files
}

fn content_with(scenario: &str, extra: &[(&str, &str)]) -> Arc<Content> {
    Arc::new(Content::load(&files(scenario, extra)).expect("test content is valid"))
}

/// A world of `scenario` where nobody hungers, for rests of days.
fn unfed_world(scenario: &str) -> World {
    let mut files = files(scenario, &[]);
    files.remove("data/hunger.toml");
    let content = Arc::new(Content::load(&files).expect("test content is valid"));
    World::new(content, "hall", None).expect("world")
}

/// Intelligence 14 (+2), and 100 mana beyond it.
const MAGE: &str = "mana = 100\nstats = { intelligence = 14 }";

fn world_as(scenario: &str) -> World {
    World::new(content_with(scenario, &[]), "hall", None).expect("world")
}

fn world() -> World {
    world_as(MAGE)
}

fn run(w: &mut World, actions: &str) {
    for action in parse_actions(actions).expect("actions") {
        w.apply(&action);
    }
}

fn mana(w: &World) -> i32 {
    w.state().player().mana
}

fn set_mana(w: &mut World, mana: i32) {
    let player = w.state.player;
    w.state.actors.get_mut(&player).expect("player").mana = mana;
}

fn afflict(w: &mut World, id: &str, severity: u16) {
    let status = w.content().status_id(id).expect(id);
    let player = w.state.player;
    w.afflict(player, status, severity, None, &mut Vec::new());
}

/// Mana back over an hour, from 10, by waits or by a rest.
fn an_hour(w: &mut World, rest: bool) -> i32 {
    set_mana(w, 10);
    if rest {
        w.apply(&Action::Rest(Rest::For(Duration::hours(1))));
    } else {
        run(w, "wait*600");
    }
    mana(w) - 10
}

#[test]
fn the_maximum_is_the_base_and_the_mental_stats() {
    let w = world();
    // 100 and Intelligence 14's +2; a run starts with all of it.
    assert_eq!(w.perception().own_mana(), (102, 102));
    let dull = world_as("mana = 3\nstats = { intelligence = 6, willpower = 8, presence = 12 }");
    assert_eq!(
        dull.perception().own_mana(),
        (1, 1),
        "3, less 2 and 1, and 1 more"
    );
    let none = world_as("stats = { intelligence = 3 }");
    assert_eq!(none.perception().own_mana(), (0, 0), "never below 0");
    // A character's comes from their race and class: the built-in data's.
    let content = content_with("", &[]);
    let spec: CharacterSpec = "elf,female,wizard".parse().expect("spec");
    let w = World::with_character(content.clone(), "hall", None, Some(&spec)).expect("world");
    let base = spec.race(&content).expect("race").mana + spec.class(&content).expect("class").mana;
    let me = w.state().player();
    let mental = crate::mana::mental_modifier(&me.effective_stats(&content));
    assert_eq!(me.mana_base, base);
    assert_eq!(w.perception().own_mana().1, (base + mental).max(0));
}

#[test]
fn creature_types_may_have_mana_and_others_save_none() {
    let level = "name = \"Den\"\nambient = [255, 255, 255, 0]\nmap = '''\n#####\n#@cr#\n#####\n'''\n\
                 [legend]\n\"@\" = { terrain = \"floor\", spawn = \"player\" }\n\
                 \"c\" = { terrain = \"floor\", creature = \"adept\", faction = \"peaceful\" }\n\
                 \"r\" = { terrain = \"floor\", creature = \"rat\", faction = \"peaceful\" }\n";
    let adept = "id = \"adept\"\nname = \"adept\"\nglyph = \"p\"\nfg = \"#ffffff\"\n\
                 weight = 60000\nfaction = \"peaceful\"\nstats = { willpower = 16 }\nmana = 7\n";
    let content = content_with(
        "",
        &[
            ("levels/den.toml", level),
            (
                "data/creatures/adept.toml",
                &format!("[[creature]]\n{adept}"),
            ),
            (
                "scenarios/den.toml",
                "name = \"Den\"\nstart_level = \"den\"\n",
            ),
        ],
    );
    let w = World::new(content.clone(), "den", None).expect("world");
    let (_, actors) = w.ground_truth();
    let actors: Vec<&Actor> = actors.collect();
    let adept = actors
        .iter()
        .find(|a| content.creature(a.creature).id == "adept");
    let adept = adept.expect("the adept");
    assert_eq!(
        (adept.mana, adept.max_mana(&content)),
        (10, 10),
        "7 and Willpower 16's +3"
    );
    // An actor without mana saves (and hashes) as it did before mana: none of its fields show.
    let rat = actors
        .iter()
        .find(|a| content.creature(a.creature).id == "rat");
    let json = serde_json::to_string(rat.expect("the rat")).expect("json");
    assert!(!json.contains("mana"), "{json}");
}

#[test]
fn mana_comes_back_by_willpower_and_faster_resting() {
    // Willpower 10: a point every 240 s, 15 an hour; resting three times as fast.
    assert_eq!(an_hour(&mut world(), false), 15);
    assert_eq!(an_hour(&mut world(), true), 45);
    // Willpower 20: every 120 s.
    let mut strong = world_as("mana = 100\nstats = { willpower = 20 }");
    assert_eq!(an_hour(&mut strong, false), 30);
    // Never past the maximum, and the progress is dropped there.
    let mut w = world();
    set_mana(&mut w, 101);
    run(&mut w, "wait*100");
    let me = w.state().player();
    assert_eq!((me.mana, me.mana_regen), (102, 0));
}

#[test]
fn time_cut_up_differently_ends_the_same() {
    // Waits of 6 s against waits of a minute: three minutes of a hex draining and easing, in a
    // dazed head, mana coming back as the hex lets it.
    let mut quick = world();
    let mut slow = World::new(
        content_with(MAGE, &[("data/actions.toml", "wait = 10000\n")]),
        "hall",
        None,
    )
    .expect("world");
    for w in [&mut quick, &mut slow] {
        set_mana(w, 40);
        afflict(w, "hexed", 2);
        afflict(w, "dazed", 1);
    }
    run(&mut quick, "wait*30");
    run(&mut slow, "wait*3");
    assert_eq!(quick.time(), slow.time(), "three minutes each");
    let (a, b) = (quick.state().player(), slow.state().player());
    assert_eq!(
        (a.mana, a.mana_regen, &a.effects),
        (b.mana, b.mana_regen, &b.effects)
    );
    // 2 degrees: a tick every 15 s for a minute (4 ticks), then 1: every 30 s (2 more); the
    // last minute, dazed, is too short for a point back.
    assert_eq!(
        a.mana,
        40 - 2 * 6,
        "the hex drained it, and none came back meanwhile"
    );
    // An hour's rest in one action against waits.
    let mut rested = world();
    let mut waited = world();
    for w in [&mut rested, &mut waited] {
        set_mana(w, 20);
        afflict(w, "dazed", 1);
    }
    rested.apply(&Action::Rest(Rest::For(Duration::minutes(30))));
    run(&mut waited, "wait*300");
    assert_eq!(rested.time(), waited.time());
    assert_eq!(mana(&waited) - 20, 30 * 60 / 240 / 2, "half as fast dazed");
    assert_eq!(
        mana(&rested) - 20,
        30 * 60 * 3 / 240 / 2,
        "resting makes up for it"
    );
}

#[test]
fn a_hex_drains_mana_but_never_below_nothing() {
    let mut w = world();
    set_mana(&mut w, 3);
    afflict(&mut w, "hexed", 5);
    run(&mut w, "wait*5");
    assert_eq!(mana(&w), 0);
    assert!(w.state().player().hp > 0, "draining mana never kills");
    assert_eq!(w.mana_rate(w.player_id()), 0, "a hex stops it");
}

#[test]
fn statuses_diseases_and_starving_slow_or_stop_it() {
    let mut w = world();
    let player = w.player_id();
    assert_eq!(w.mana_rate(player), 100);
    afflict(&mut w, "poisoned", 1);
    assert_eq!(
        w.mana_rate(player),
        100,
        "poison slows hit points, not mana"
    );
    afflict(&mut w, "dazed", 1);
    assert_eq!(w.mana_rate(player), 50, "the slowest status counts");
    // A disease, once it shows: the slower of Diseased's pace and its own.
    let mut w = unfed_world(MAGE);
    let fog = w.content().disease_id("fog").expect("fog");
    assert!(w.infect(player, fog, None));
    assert_eq!(w.mana_rate(player), 100, "not while it incubates");
    w.apply(&Action::Rest(Rest::For(Duration::hours(24))));
    assert!(w.state().player().diseases.iter().any(|i| i.shown));
    assert_eq!(w.mana_rate(player), 10);
    // Starving: none at all.
    let mut w = world();
    let me = w.state.actors.get_mut(&player).expect("player");
    me.satiation = Some(crate::hunger::Satiation::new(-1));
    assert_eq!(
        w.perception().own_hunger(),
        Some(crate::hunger::Hunger::Starving)
    );
    assert_eq!(w.mana_rate(player), 0);
    assert_eq!(an_hour(&mut w, true), 0);
}

#[test]
fn a_mana_well_speeds_it_and_burden_and_stances_do_not() {
    let mut w = world();
    let player = w.player_id();
    run(&mut w, "move:E*3");
    assert_eq!(w.mana_rate(player), 100, "outside the well");
    run(&mut w, "move:E");
    assert_eq!(w.mana_rate(player), 300, "three times as fast inside");
    // Nothing else changes it: a stance, a load that slows the player.
    let mut w = world();
    w.apply(&Action::SetStance(crate::content::CombatStance::Aggressive));
    let rock = w.content().object_id("chain_mail").expect("object");
    let me = w.state.actors.get_mut(&player).expect("player");
    me.inventory
        .extend((0..5).map(|_| crate::object::Stack::one(rock)));
    assert_ne!(
        w.load_of(player).penalty(),
        crate::burden::Burden::Unburdened
    );
    assert_eq!(w.mana_rate(player), 100);
}

#[test]
fn a_status_penalty_passes_and_the_mana_kept_stays() {
    let mut w = world();
    afflict(&mut w, "befuddled", 1);
    // Intelligence 10 for a minute: the maximum is 100, the 102 stay, and none comes back.
    assert_eq!(w.state().player().max_mana(w.content()), 100);
    assert_eq!(
        w.perception().own_mana(),
        (100, 100),
        "shown within the maximum"
    );
    run(&mut w, "wait");
    assert_eq!(mana(&w), 102);
    run(&mut w, "wait*10");
    assert_eq!(w.perception().own_mana(), (102, 102), "back once it passes");
}

#[test]
fn damage_to_the_mental_stats_cuts_mana_down() {
    // Willpower 1: the fog's checks are all but lost, and each takes 4 Intelligence and 2
    // Willpower (at least 1 left), lowering the most mana by their modifiers.
    let mut w = unfed_world("mana = 100\nstats = { intelligence = 14, willpower = 1 }");
    let player = w.player_id();
    let fog = w.content().disease_id("fog").expect("fog");
    assert!(w.infect(player, fog, None));
    w.apply(&Action::Rest(Rest::For(Duration::hours(24))));
    assert_eq!(mana(&w), 100 + 2 - 5, "full at onset");
    w.apply(&Action::Rest(Rest::For(Duration::hours(24))));
    let me = w.state().player();
    assert!(me.damage.is_damaged(), "the check was lost");
    assert_eq!(me.max_mana(w.content()), 100 - 5);
    assert_eq!(mana(&w), 95, "cut down to the new maximum");
}

#[test]
fn spending_is_refused_when_short_and_the_dev_drain_spends() {
    let mut w = world();
    let player = w.player_id();
    assert!(w.spend_mana(player, 100));
    assert_eq!(mana(&w), 2);
    assert!(!w.spend_mana(player, 3), "too little");
    assert!(!w.spend_mana(player, -1));
    assert_eq!(mana(&w), 2, "nothing spent");
    assert!(w.spend_mana(player, 2) && w.spend_mana(player, 0));
    // The dev action spends the same way, in no time, and says so.
    let mut w = world();
    let outcome = w.apply(&Action::Drain(30));
    assert!(outcome.elapsed.is_zero() && !outcome.took_turn);
    assert_eq!(w.perception().own_mana(), (72, 102));
    let said = |w: &World, text: &str| w.messages().iter().any(|m| m.text == text);
    assert!(said(&w, "You spend 30 mana (dev)."));
    w.apply(&Action::Drain(100));
    assert_eq!(mana(&w), 72);
    assert!(said(&w, "You have only 72 mana (dev)."));
}

#[test]
fn levels_add_half_the_class_mana_and_the_better_mind_up_to_mana_level() {
    // Half of 100 and Intelligence 14's +2 (better than Willpower 10's 0): 52 a level, for
    // levels 2 and 3 only.
    let mut w = world();
    let to_five = w.content().experience().threshold(5);
    w.grant_xp(to_five);
    assert_eq!(w.player_level(), 5);
    let me = w.state().player();
    assert_eq!((me.mana_base, me.mana), (100 + 2 * 52, 102 + 2 * 52));
}

#[test]
fn mana_is_saved_in_the_state_hash_and_replays() {
    let content = content_with(MAGE, &[]);
    let mut w = World::new(content.clone(), "hall", None).expect("world");
    let full = w.state_hash();
    let mut replay = Replay::new(&w);
    for action in parse_actions("drain:40 wait*3 rest:30m drain:5").expect("actions") {
        w.apply(&action);
        replay.push(&action, &w);
    }
    assert_ne!(w.state_hash(), full);
    let bytes = crate::save::encode(&w);
    let mut loaded = crate::save::decode(&bytes, content.clone()).expect("loads");
    assert_eq!(loaded.state(), w.state());
    run(&mut w, "wait*20");
    run(&mut loaded, "wait*20");
    assert_eq!(loaded.state_hash(), w.state_hash());
    let verified = Replay::from_json(&replay.to_json())
        .expect("json")
        .run(content)
        .expect("the replay verifies");
    // 45 spent, and 22 back over the half-hour's rest.
    assert_eq!(verified.state().player().mana, 102 - 45 + 3 * 30 * 60 / 240);
}

/// One `object` with the affix `id` at `value`, identified, as loot would make it.
fn magic(w: &World, object: &str, id: &str, value: i32) -> Stack {
    let magic = crate::magic::Magic {
        rarity: crate::magic::Rarity::Magic,
        level: 1,
        affixes: vec![crate::magic::Affix {
            id: id.into(),
            tier: 1,
            value,
        }],
        name: None,
        identified: true,
    };
    let object = w.content().object_id(object).expect("object");
    Stack::with_magic(object, 1, magic)
}

#[test]
fn magic_items_raise_the_maximum_and_the_pace_and_taking_them_off_lowers_it() {
    let mut w = world();
    let player = w.player_id();
    let ring = magic(&w, "gold_ring", "mystic", 5);
    let amulet = magic(&w, "jade_amulet", "of_clarity", 50);
    let me = w.state.actors.get_mut(&player).expect("player");
    me.inventory.extend([ring, amulet]);
    run(&mut w, "equip:left_ring:a equip:neck:b");
    let me = w.state().player();
    assert_eq!(me.max_mana(w.content()), 107, "+5 to maximum mana");
    assert_eq!(me.mana, 102, "putting it on fills nothing");
    assert_eq!(w.mana_rate(player), 150, "half again as fast");
    // Full, then the ring comes off: down to the maximum without it.
    set_mana(&mut w, 107);
    run(&mut w, "remove:left_ring");
    assert_eq!(w.perception().own_mana(), (102, 102));
    run(&mut w, "remove:neck");
    assert_eq!(w.mana_rate(player), 100);
}

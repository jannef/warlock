//! The action fuzz test (#1569): random action text, every verb with arguments in and out of
//! range (stacks past the end, cells off the map, directions into walls, ids that name nothing),
//! applied to a few scenarios. An action is refused or done, never a panic; a refused or free
//! action passes no time; and the same actions from the same start end in the same state hash.
//!
//! `cargo test -p warlock-core --test action_fuzz` runs a small fixed set (the same every run,
//! for CI); `-- --ignored` runs many more cases from a random start.

#![cfg(feature = "showcase")]

use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};
use std::sync::Arc;
use warlock_core::action::Action;
use warlock_core::content::Content;
use warlock_core::world::World;

/// The scenarios fuzzed: the hub, a fight, a generated dungeon floor, and the overworld.
const SCENARIOS: [&str; 4] = ["showcase", "arena", "depths", "world"];

const DIRS: [&str; 9] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW", "X"];

/// Ids no table names, mixed in with the real ones.
const NOTHING: &str = "no_such_thing";

/// The ids the verbs draw from: the content's, with a few fixed lists beside them.
struct Vocab {
    spells: Vec<String>,
    creatures: Vec<String>,
    statuses: Vec<String>,
    talents: Vec<String>,
}

impl Vocab {
    fn of(content: &Content) -> Self {
        let mut talents: Vec<String> = content
            .talents()
            .iter()
            .map(|(_, t)| t.id.clone())
            .collect();
        talents.push("weapon_focus:long_blades".into());
        Self {
            spells: content.spells().iter().map(|s| s.id.clone()).collect(),
            creatures: content
                .creature_defs()
                .iter()
                .map(|c| c.id.clone())
                .collect(),
            statuses: content.status_defs().iter().map(|s| s.id.clone()).collect(),
            talents,
        }
    }
}

/// One of `list` by `n`, now and then an id that names nothing.
fn one<S: AsRef<str>>(list: &[S], n: u32) -> String {
    if n.is_multiple_of(13) || list.is_empty() {
        return NOTHING.into();
    }
    list[n as usize % list.len()].as_ref().to_string()
}

fn dir(n: u32) -> &'static str {
    DIRS[n as usize % DIRS.len()]
}

/// A stack position: mostly a letter, sometimes a number past any pack.
fn stack(n: u32) -> String {
    match n % 5 {
        0 => format!("{}", n % 300),
        _ => ((b'a' + (n % 26) as u8) as char).to_string(),
    }
}

/// A cell: mostly on or near a map, sometimes far off it, now and then past what action text
/// takes (`action::MOST_COORDINATE`: refused by the parser).
fn cell(a: u32, b: u32) -> String {
    let most = warlock_core::action::MOST_COORDINATE;
    match a % 13 {
        0 => format!("{},{}", -(b as i32 % 1000), b % 2000),
        1 => format!("{},{}", most - (b % 3) as i32, -most + (b % 3) as i32),
        2 => format!("{},{}", -most, most),
        3 => format!(
            "{},{}",
            i32::MAX - (b % 3) as i32,
            i32::MIN + (b % 3) as i32
        ),
        _ => format!("{},{}", a % 130, b % 70),
    }
}

const ABILITIES: [&str; 26] = [
    "rage",
    "second_wind",
    "blood_magic",
    "veterans_wind",
    "lay_on_hands",
    "smite",
    "holy_radiance",
    "quivering_palm",
    "exploit",
    "inspire",
    "lullaby",
    "cackle",
    "knock",
    "field_forge",
    "orison",
    "sanctuary",
    "call_familiar",
    "volley",
    "sic",
    "charge",
    "make_do_and_mend",
    "thrown_exploit",
    "spirit_weapon",
    "lunge",
    "shield_wall",
    "piercing_shot",
];

const RECIPES: [&str; 12] = [
    "long_sword",
    "dagger",
    "arrows",
    "bolts",
    "arrowheads",
    "smelt",
    "sling_stones",
    "potion_of_healing",
    "cut",
    "set",
    "kindle",
    "unbind",
];

const SLOTS: [&str; 8] = [
    "head",
    "body",
    "main_hand",
    "off_hand",
    "missile",
    "tool",
    "neck",
    "finger",
];

const SKILLS: [&str; 8] = [
    "stealth",
    "mining",
    "heal",
    "long_blades",
    "bows",
    "decipher_script",
    "metalcraft",
    "duelist",
];

/// The action text of verb `v` with the numbers `a`, `b`, `c` for its arguments.
fn text(vocab: &Vocab, (v, a, b, c): (usize, u32, u32, u32)) -> String {
    let d = dir(a);
    let s = stack(b);
    let at = cell(b, c);
    match v {
        0 => format!("move:{d}"),
        1 => format!("run:{d}"),
        2 => "wait".into(),
        3 => format!("open:{d}"),
        4 => format!("close:{d}"),
        5 => format!("kick:{d}"),
        6 => format!("pick:{d}"),
        7 => "ascend".into(),
        8 => "descend".into(),
        9 => "use_tool".into(),
        10 => format!("pickup:{s}"),
        11 => format!("drop:{s}"),
        12 => format!("sell:{s}"),
        13 => format!("pay:{d}"),
        14 => format!("equip:{}:{s}", one(&SLOTS, c)),
        15 => format!("remove:{}", one(&SLOTS, c)),
        16 => format!("bash:{d}"),
        17 => format!("fire:{at}"),
        18 => format!("throw:{s}:{at}"),
        19 => format!("eat:{s}"),
        20 => format!("eat:floor:{s}"),
        21 => format!("craft:{}:{s},{}", one(&RECIPES, a), stack(c)),
        22 => format!("give:{s}:{d}"),
        23 => match c % 3 {
            0 => format!("read:{s}"),
            1 => format!("read:{s}:{at}"),
            _ => format!("read:{s}:{}", stack(a)),
        },
        24 => format!("scribe:{}", one(&vocab.spells, c)),
        25 => format!("dig:{d}"),
        26 => format!("search:{}", 1 + c % 10),
        27 => format!("disarm:{d}"),
        28 => format!("listen:{d}"),
        29 => format!("gather:{d}"),
        30 => format!("wedge:{d}"),
        31 => format!("heal:{}:{d}", one(&vocab.statuses, c)),
        32 => format!("feint:{d}"),
        33 => format!("intimidate:{at}"),
        34 => "pray".into(),
        35 => format!("temper:{s}"),
        36 => format!("whittle:{s}"),
        37 => format!("salvage:{s}"),
        38 => format!("rig:{}:{d}", one(&["tripwire", "stakes", "pitfall"], c)),
        39 => format!("decipher:{s}"),
        40 => "drink".into(),
        41 => match c % 4 {
            0 => format!("use:{}", one(&ABILITIES, a)),
            1 => format!("use:{}:{d}", one(&ABILITIES, b)),
            2 => format!("use:{}:{at}", one(&vocab.spells, a)),
            _ => format!("use:{}:{s}", one(&vocab.spells, a)),
        },
        42 => format!("chat:{d}"),
        43 => format!(
            "ask:{d}:{}",
            one(&["rooms", "cure", "wares", "quest-rats"], c)
        ),
        44 => format!("steal:{d}"),
        45 => format!("tame:{d}"),
        46 => format!("throw_voice:{at}"),
        47 => match c % 3 {
            0 => "order:follow".into(),
            1 => "order:stay".into(),
            _ => format!("order:hunt:{at}"),
        },
        48 => format!("rest:{}", one(&["30s", "10m", "1m"], c)),
        49 => format!(
            "encounter:{}",
            one(
                &["ambush", "bluff", "evade", "fight", "hail", "hunt", "pass"],
                c
            )
        ),
        50 => format!(
            "stance:{}",
            one(
                &["aggressive", "neutral", "defensive", "coward", "frenzy"],
                c
            )
        ),
        51 => format!("talent:{}", one(&vocab.talents, c)),
        52 => format!("advance:{}", one(&SKILLS, c)),
        53 => "levelup:auto".into(),
        54 => format!("xp:{}", c % 5000),
        55 => format!("call:{}", one(&["hunter", "duelist", "archmage"], c)),
        56 => format!("rank:{}:{}", one(&SKILLS, a), c % 12),
        57 => format!("skip:{}", one(&["30s", "10m"], c)),
        58 => format!(
            "weather:{}",
            one(&["clear", "overcast", "rain", "heavy"], c)
        ),
        59 => "spawn".into(),
        60 => format!("ally:{}:{}", one(&vocab.creatures, c), 1 + a % 3),
        61 => format!("effect:{}@{at}", one(&["fireball", "lightning_bolt"], a)),
        62 => format!(
            "zone:{}@{at},{}",
            one(&["fire", "fouled", "slick", "dark", "smoke"], a),
            c % 25
        ),
        63 => format!("identify:{s}"),
        64 => format!("afflict:{}", one(&vocab.statuses, c)),
        65 => format!("drain:{}", 1 + c % 20),
        66 => format!("resource:{}", one(&["rage", "openings", "stamina"], c)),
        67 => format!("infect:{}", one(&["the_shakes", "filth_fever"], c)),
        68 => format!("freeze:{at},{}", c % 25),
        69 => format!("thaw:{at},{}", c % 25),
        70 => format!("search:{}", c % 3),
        71 => format!("steal:{d}"),
        72 => "forage".into(),
        73 => format!("vision:{}", one(&["normal", "darkvision", "uv_sight"], c)),
        74 => format!(
            "meet:{}:{}",
            NOTHING,
            one(&["drop", "met", "ambushed", "pass"], c)
        ),
        75 => format!("quest_roll:{NOTHING}:{}", one(&vocab.creatures, c)),
        76 => format!("reveal:{NOTHING}"),
        77 => format!("lure:{NOTHING}"),
        _ => format!("move:{d}"),
    }
}

/// How many verbs [`text`] knows.
const VERBS: usize = 78;

/// Nearly every word [`text`] makes parses (the rest are refused by the parser, `move:X`), so
/// the fuzz applies actions, not parse errors.
#[test]
fn the_words_parse() {
    let content = Content::builtin().expect("the built-in content loads");
    let vocab = Vocab::of(&content);
    let mut words = 0;
    let mut parsed = 0;
    for v in 0..VERBS {
        for n in 0..50u32 {
            let text = text(&vocab, (v, n * 7 + 1, n * 11 + 2, n * 13 + 3));
            words += 1;
            parsed += usize::from(text.parse::<Action>().is_ok());
        }
    }
    assert!(parsed * 10 >= words * 8, "{parsed} of {words} parse");
}

/// The state hashes after each of `texts` applied to a new run of `scenario`.
fn play(
    content: &Arc<Content>,
    scenario: &str,
    texts: &[String],
) -> Result<Vec<u64>, TestCaseError> {
    let mut w = World::new(content.clone(), scenario, None).expect("the scenario starts");
    let mut hashes = Vec::with_capacity(texts.len());
    for (i, text) in texts.iter().enumerate() {
        // Text that does not parse is refused by the parser: nothing to apply.
        let Ok(action) = text.parse::<Action>() else {
            continue;
        };
        let before = w.time();
        let outcome = w.apply(&action);
        if !outcome.took_turn && outcome.elapsed.as_seconds() == 0 {
            prop_assert_eq!(w.time(), before, "`{}` took no time but time passed", text);
        }
        prop_assert!(w.time() >= before, "`{}` turned time back", text);
        if i % 8 == 7 {
            let _ = w.perception();
        }
        hashes.push(w.state_hash());
    }
    let _ = w.perception();
    Ok(hashes)
}

fn fuzz(scenario: &str, runner: &mut TestRunner, steps: usize) {
    let content = Arc::new(Content::builtin().expect("the built-in content loads"));
    let vocab = Vocab::of(&content);
    // A third more draws are steps, past the verbs: the player gets about, not only refused.
    let word = (
        0..VERBS + VERBS / 3,
        any::<u32>(),
        any::<u32>(),
        any::<u32>(),
    );
    let strategy = proptest::collection::vec(word, 1..=steps);
    let result = runner.run(&strategy, |words| {
        let texts: Vec<String> = words.iter().map(|&w| text(&vocab, w)).collect();
        let first = play(&content, scenario, &texts)?;
        let second = play(&content, scenario, &texts)?;
        prop_assert_eq!(
            first,
            second,
            "the same actions, a different state: {:?}",
            texts
        );
        Ok(())
    });
    if let Err(e) = result {
        panic!("{scenario}: {e}");
    }
}

/// The CI run: a few cases per scenario, from a fixed start, so a failure is the same each time.
fn small(scenario: &str) {
    let config = Config {
        cases: 4,
        failure_persistence: None,
        ..Config::default()
    };
    let rng = TestRng::deterministic_rng(RngAlgorithm::ChaCha);
    fuzz(scenario, &mut TestRunner::new_with_rng(config, rng), 60);
}

#[test]
fn fuzz_showcase() {
    small(SCENARIOS[0]);
}

#[test]
fn fuzz_arena() {
    small(SCENARIOS[1]);
}

#[test]
fn fuzz_depths() {
    small(SCENARIOS[2]);
}

#[test]
fn fuzz_world() {
    small(SCENARIOS[3]);
}

/// The long run: many cases of many steps from a random start, every scenario.
#[test]
#[ignore = "slow: run by hand"]
fn fuzz_long() {
    for scenario in SCENARIOS {
        let config = Config {
            cases: 64,
            failure_persistence: None,
            ..Config::default()
        };
        fuzz(scenario, &mut TestRunner::new(config), 300);
    }
}

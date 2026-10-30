//! The balance probe (#569, `docs/balance.md`; the combat progression design §2 and §15 D1): an
//! `#[ignore]` test that measures the fight between the player and the monsters of their level
//! through the game's own code and prints the tables `docs/balance.md` records. Run it with
//!
//! ```text
//! cargo test --release -p warlock-core balance_probe -- --ignored --nocapture
//! ```
//!
//! It changes nothing and asserts nothing: it is a measuring instrument, kept so the first-guess
//! numbers in the data can be judged again after a tuning PR. Everything is deterministic (fixed
//! seeds and streams), so the same content prints the same tables.
//!
//! - **Monsters**: every creature type with a level (shared and campaigns'), made by
//!   `travel::make_creature`, once bare and once per seed with the gear its type rolls
//!   (`gear::roll` at its own level). Its side of a fight is `World::target_of` (Defence,
//!   Protection, resistances), its blow `Actor::melee`, its blows a round its speed over its
//!   `World::attack_price`.
//! - **Players**: a human male warrior, thief, wizard, and priest per seed, made by
//!   `World::start_run` with the kit, levelled one level at a time with `xp:` and
//!   `levelup:auto`. **Trained** stats (#727, `training`): before each level, its game days
//!   (the stat training design's §3.1: one to two, three, four; 189 to level 60), each with its
//!   share of the level's kills (`Experience::kills_for`) fought blow by blow against the
//!   monsters of the player's level through the game's training hooks, the day's other uses
//!   (`training::COMMON` and the build's `extras`), and the daily check at 06:00
//!   (`World::daily_stat_check`); then the level's own check: the game's rules on the game's
//!   streams. **Untrained**: no marks. Weapon
//!   skill ranks are set by level (`rank`), as ranks come from use, not levels. Gear: the kit,
//!   plain level-appropriate pieces, or the same pieces magic (rare from level 30, with an
//!   amulet and two rings from 15), equipped with `equip:`.
//! - **A fight**: one round is a normal turn. Each side's expected damage a round is its blows
//!   a round times the mean of `combat::strike` against the other's target over `STRIKES`
//!   draws (the player's off hand adds its blow); the hit chance is `combat::hit_ways` exactly.
//!   Rounds to kill are the monster's hit points over the player's damage a round, rounds to
//!   die the player's hit points over the monster's. Each cell is the median over the monsters
//!   within two levels of the player's (`+3`: one to five levels above).
//! - **Spells**: the wizard's and the priest's own casts of every damage spell, at their
//!   caster level, with their trained stats' lift (`Actor::spell_lift`) and without, the dice
//!   as `SpellDice::lifted` and the talents make them, nothing holding the roll; attack spells
//!   strike the monster's Defence against missiles, areas always hit.
//!
//! - **Sneak attacks** (#688): the thief's first blow on a foe unaware of them against its
//!   plain blow. Its Sneak attack skill is set by level (`sneak_value`), as it rises by use and
//!   `levelup:auto` spends the rolls elsewhere; the dice are `World::sneak_dice` (the skill and
//!   the talents), the foe flat-footed (`World::flat_footed`), Master thief's threat
//!   (`TalentMods::ambush_threat`), and no dice on a type without vitals (`Content::has_vitals`).
//!
//! Not modelled: triggered and active weapon abilities, stances, talents chosen for a build
//! (`levelup:auto` takes the first it may), the initiative check's Surprised after the first
//! blow, potions, and resistances against spells.

mod tables;
mod training;

use super::{ActorId, World};
use crate::action::{AttackKind, parse_actions};
use crate::character::{CharacterSpec, Gender};
use crate::combat::{self, Attack, Defence, Profile, Target};
use crate::content::{
    ActionKind, BASE_TURN, Content, CreatureId, CreatureSpawn, Factor, NORMAL_SPEED, Resolution,
    SpellId, StatUse, price,
};
use crate::damage::Kinds;
use crate::dice::Dice;
use crate::magic::{LootGrade, loot};
use crate::object::Stack;
use crate::rng::Rng;
use crate::stats::Stat;
use std::sync::Arc;

/// The player levels measured.
const LEVELS: [u8; 9] = [1, 5, 10, 15, 20, 30, 40, 50, 60];
/// The players per class and level (and the monsters' gear rolls): seeds 1 to `SEEDS`.
const SEEDS: u64 = 6;
/// Strikes drawn per pairing and direction.
const STRIKES: u32 = 300;

/// The gear a player is measured in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Gear {
    Kit,
    Plain,
    Magic,
}

/// The Sneak attack value assumed at a character level: the thief's creation 2d6+10 and about
/// three and a half a level from use and the level-up rolls (the design's "about 100 by level 25
/// to 30", 200 around 50; `docs/design/surprise-and-sneak-attacks.md`, "Numbers").
fn sneak_value(level: u8) -> u16 {
    12 + u16::from(level) * 7 / 2
}

/// The weapon skill rank assumed at a character level (the 2026-10-27 report's).
fn rank(level: u8) -> u8 {
    match level {
        0..=2 => 2,
        3..=4 => 3,
        5..=7 => 4,
        8..=9 => 5,
        10..=14 => 6,
        15..=19 => 7,
        20..=29 => 8,
        30..=39 => 9,
        _ => 10,
    }
}

/// One measured class.
struct Build {
    class: &'static str,
    /// Weapon skills and styles, each with how many ranks below `rank` it stays.
    skills: &'static [(&'static str, u8)],
    /// What the class does in a day beyond [`training::COMMON`]: a use and how many times.
    extras: &'static [(StatUse, f64)],
    /// Whether it spends mana on an ability (a heal, a blessing) once a fight.
    casts_a_kill: bool,
    gear: fn(u8) -> Vec<(&'static str, &'static str)>,
}

const BUILDS: [Build; 4] = [
    Build {
        class: "warrior",
        skills: &[
            ("long_blades", 0),
            ("sword_and_board", 0),
            ("shield_block", 0),
        ],
        extras: &[(StatUse::Exerted, 5.0)],
        casts_a_kill: false,
        gear: warrior,
    },
    Build {
        class: "thief",
        skills: &[("daggers", 0), ("twin_blades", 0)],
        extras: &[
            (StatUse::Sneak, 70.0),
            (StatUse::Picked, 1.0),
            (StatUse::Disarmed, 0.5),
            (StatUse::Stole, 0.5),
            (StatUse::Exerted, 3.0),
        ],
        casts_a_kill: false,
        gear: thief,
    },
    Build {
        class: "wizard",
        skills: &[("staves", 2), ("spell_attack", 0)],
        extras: &[(StatUse::Read, 1.0), (StatUse::Identified, 0.3)],
        casts_a_kill: false,
        gear: wizard,
    },
    Build {
        class: "priest",
        skills: &[
            ("maces", 0),
            ("sword_and_board", 0),
            ("shield_block", 0),
            ("spell_attack", 2),
        ],
        extras: &[(StatUse::Prayed, 0.2), (StatUse::Exerted, 2.0)],
        casts_a_kill: true,
        gear: priest,
    },
];

fn warrior(l: u8) -> Vec<(&'static str, &'static str)> {
    let mut g = vec![("main_hand", "long_sword")];
    if l >= 5 {
        g.push((
            "off_hand",
            if l >= 15 {
                "large_shield"
            } else {
                "small_shield"
            },
        ));
        g.push((
            "body",
            match l {
                0..=14 => "chain_mail",
                15..=29 => "banded_mail",
                _ => "plate_armour",
            },
        ));
        g.push(("head", if l >= 20 { "great_helm" } else { "iron_helm" }));
        g.push((
            "hands",
            if l >= 10 {
                "gauntlets"
            } else {
                "leather_gloves"
            },
        ));
        g.push((
            "feet",
            if l >= 15 {
                "iron_shod_boots"
            } else {
                "leather_boots"
            },
        ));
        g.push(("cloak", "wool_cloak"));
    }
    g
}

fn thief(l: u8) -> Vec<(&'static str, &'static str)> {
    let mut g = vec![("main_hand", "dagger")];
    if l >= 5 {
        g.push((
            "body",
            if l >= 10 {
                "studded_leather"
            } else {
                "leather_jerkin"
            },
        ));
        g.push(("head", "leather_cap"));
        g.push(("hands", "leather_gloves"));
        g.push(("feet", "leather_boots"));
        g.push(("cloak", "wool_cloak"));
    }
    g
}

fn wizard(l: u8) -> Vec<(&'static str, &'static str)> {
    let mut g = vec![("main_hand", "quarterstaff")];
    if l >= 5 {
        g.push(("body", "padded_coat"));
        g.push(("head", "leather_cap"));
        g.push(("feet", "leather_boots"));
        g.push(("cloak", "wool_cloak"));
    }
    g
}

fn priest(l: u8) -> Vec<(&'static str, &'static str)> {
    let mut g = vec![("main_hand", "mace")];
    if l >= 5 {
        g.push((
            "off_hand",
            if l >= 15 {
                "large_shield"
            } else {
                "small_shield"
            },
        ));
        g.push((
            "body",
            match l {
                0..=9 => "studded_leather",
                10..=19 => "chain_mail",
                20..=29 => "banded_mail",
                _ => "plate_armour",
            },
        ));
        g.push(("head", "iron_helm"));
        g.push(("hands", "leather_gloves"));
        g.push(("feet", "leather_boots"));
        g.push(("cloak", "wool_cloak"));
    }
    g
}

/// The mean of `dice` (a sum of dice is symmetric about the middle of its range).
fn mean(dice: Dice) -> f64 {
    f64::from(dice.min() + dice.max()) / 2.0
}

/// The mean damage of a hit of `p` before soak and criticals.
fn hit_mean(p: &Profile) -> f64 {
    let base = f64::from(p.damage.mean_x2()) / 2.0 + f64::from(p.bonus) + p.extra.map_or(0.0, mean);
    base.max(1.0) + p.riders.iter().map(|r| mean(r.dice)).sum::<f64>()
}

/// The mean damage of `n` strikes of `p` against `t`, misses included.
fn strike_mean(p: &Profile, t: &Target, rng: &mut Rng) -> f64 {
    let total: i64 = (0..STRIKES)
        .map(|_| i64::from(combat::strike(p, t, rng).damage))
        .sum();
    total as f64 / f64::from(STRIKES)
}

/// The chance in percent that a blow at `to_hit` lands on `defence`.
fn hit_pct(to_hit: i32, defence: &Defence) -> f64 {
    f64::from(combat::hit_ways(to_hit, defence.total())) * 100.0 / 144.0
}

/// One side of a fight, as the rules strike it.
#[derive(Clone)]
struct Side {
    blow: Profile,
    off: Option<Profile>,
    target: Target,
    /// Its side caught unawares: flat-footed, nothing of its Dexterity or Dodge.
    flat: Target,
    /// Its Defence against a missile (a spell's bolt or ray).
    missile: Defence,
    /// Blows a round (a normal turn).
    blows: f64,
    hp: i32,
}

/// Actor `id`'s side in `w`.
fn side_of(w: &World, id: ActorId) -> Side {
    let content = &w.content;
    let a = &w.state.actors[&id];
    let target = w.target_of(id);
    let with = a.weapon_item(content);
    let object = with.map(|i| a.inventory[i].object);
    let skills = a.attack_skills(content, AttackKind::Melee, object);
    let cost = w.attack_price(id, ActionKind::Melee, with, skills);
    let turn = price(NORMAL_SPEED, BASE_TURN) as f64;
    let blows = f64::from(a.speed(content)) / f64::from(NORMAL_SPEED) * turn / cost as f64;
    Side {
        blow: a.melee(content),
        off: a.off_hand_blow(content).map(|(_, p)| p),
        flat: w.flat_footed(id, true, target.clone()),
        target,
        missile: w.target_against(id, true).defence,
        blows,
        hp: a.max_hp(content),
    }
}

/// A creature type, bare and with each seed's gear.
struct Monster {
    creature: CreatureId,
    level: u8,
    bare: Side,
    geared: Vec<Side>,
}

/// Every creature type with a level, made in one world.
fn monsters(content: &Arc<Content>) -> Vec<Monster> {
    let mut w = World::new(content.clone(), "ascmearc", Some(1)).expect("world");
    let level = w.state.current_level.clone();
    let pos = w.state.player().pos;
    let mut out = Vec::new();
    for i in 0..content.creature_defs().len() {
        let cid = CreatureId(u16::try_from(i).expect("creature id"));
        let def = content.creature(cid);
        if def.level == 0 {
            continue;
        }
        let il = def.level.clamp(1, 99);
        let mut sides = Vec::new();
        for seed in 0..=SEEDS {
            let (carry, equipment) = if seed == 0 {
                (Vec::new(), Default::default())
            } else {
                let p = crate::gear::roll(content, &mut Rng::gear(seed, "balance"), cid, il);
                (p.items, p.equipment)
            };
            let spawn = CreatureSpawn {
                creature: cid,
                faction: None,
                alignment: None,
                npc: None,
                pos,
                route: Vec::new(),
                carry,
                equipment,
                rolls_gear: false,
                asleep: false,
            };
            let id = super::travel::make_creature(&mut w.state, content, &level, &spawn);
            sides.push(side_of(&w, id));
            w.state.actors.remove(&id);
        }
        let bare = sides.remove(0);
        out.push(Monster {
            creature: cid,
            level: def.level,
            bare,
            geared: sides,
        });
    }
    out
}

/// A spell as one caster casts it.
struct Cast {
    spell: SpellId,
    name: String,
    /// Its spell level.
    level: u8,
    resolution: Resolution,
    lift: i32,
    /// The dice with the lift, and without.
    dice: (Dice, Dice),
    /// The attack roll's profile (to-hit and the spell skill's benefit), for attack and touch.
    aim: Profile,
    cost: i32,
}

/// The damage spells `w`'s player would cast at their caster level: those they know, and those
/// of the schools their class has an affinity with.
fn casts(w: &World) -> Vec<Cast> {
    let content = &w.content;
    if content.spell_rules().is_none() {
        return Vec::new();
    }
    let me = w.state.player();
    let class = (w.state.character.as_ref()).and_then(|c| c.spec.class(content).ok());
    let level = w.caster_level(w.state.player);
    let mods = me.talent_mods(content);
    let mut out = Vec::new();
    for spell in content.spells_by_level() {
        let def = content.spell(spell);
        let Some(damage) = def.damage else {
            continue;
        };
        let affinity = class.and_then(|c| c.affinity.get(&def.school).copied());
        if !me.knows_spell(content, spell) && affinity.unwrap_or(0) <= 0 {
            continue;
        }
        let lift = me.spell_lift(content, def);
        let dice = |l: i32| mods.spell_damage(damage.lifted(level, def.grows_from(), l));
        let skills = me.spell_skills(content, def.shape == crate::content::Shape::Touch);
        let benefit = me
            .benefit_of(content, skills)
            .at_distance(3, content.weapon_skills().spell_attacks());
        let attack = Attack {
            damage: Dice::of(1, 2).into(),
            to_hit: me.spell_to_hit(content, def),
            threat: combat::DEFAULT_THREAT,
            crit: combat::DEFAULT_CRIT,
            strength: None,
            kinds: Kinds::NONE,
        };
        out.push(Cast {
            spell,
            name: def.name.clone(),
            level: def.level,
            resolution: def.resolution,
            lift,
            dice: (dice(lift), dice(0)),
            aim: Profile::bare(&attack).with(&benefit),
            cost: me.spell_cost(content, def),
        });
    }
    out
}

/// One measured player.
struct Snap {
    class: &'static str,
    level: u8,
    gear: Gear,
    trained: bool,
    side: Side,
    scores: [u8; 6],
    potentials: [u8; 6],
    /// The game day reached, the day each stat first rose (trained only), and the marks a day
    /// earned in each stat over the level before this one.
    day: u32,
    first_rise: [Option<u32>; 6],
    marks_a_day: [f64; 6],
    mana: i32,
    caster_level: i32,
    casts: Vec<Cast>,
    /// Its sneak dice (`World::sneak_dice`), Master thief's lower threat on a sneak attack, and
    /// the tags Anatomist finds vitals in.
    sneak: Option<Dice>,
    ambush_threat: u8,
    vitals_in: Vec<String>,
}

/// Applies action text `text` to `w`.
fn act(w: &mut World, text: &str) {
    for a in parse_actions(text).expect("actions") {
        w.apply(&a);
    }
}

/// Puts `build`'s gear for `level` on `w`'s player: plain, or magic (rare from level 30).
fn dress(w: &mut World, build: &Build, level: u8, gear: Gear, seed: u64) {
    let content = w.content.clone();
    let mut pieces = (build.gear)(level);
    if gear == Gear::Magic && level >= 15 {
        pieces.push(("neck", "gold_amulet"));
        pieces.push(("left_ring", "gold_ring"));
        pieces.push(("right_ring", "silver_ring"));
    }
    for (slot, name) in pieces {
        let object = content.object_id(name).expect(name);
        let mut rng = Rng::stream(seed * 1000 + u64::from(level), name);
        let mut stack = match gear {
            Gear::Magic if level >= 30 => {
                loot::make(&content, &mut rng, object, 1, level, LootGrade::Rare)
            }
            Gear::Magic if level >= 3 => {
                loot::make(&content, &mut rng, object, 1, level, LootGrade::Magic)
            }
            _ => Stack::new(object, 1),
        };
        stack.identify();
        let at = w.player_mut().inventory.len();
        w.player_mut().inventory.push(stack);
        act(w, &format!("equip:{slot}:{at}"));
    }
}

/// One career of `build` from level 1 to the last of [`LEVELS`], measured at each.
fn career(
    content: &Arc<Content>,
    build: &Build,
    seed: u64,
    trained: bool,
    monsters: &[Monster],
) -> Vec<Snap> {
    let spec = CharacterSpec {
        race: "human".into(),
        gender: Gender::Male,
        class: build.class.into(),
        talents: Vec::new(),
    };
    let mut w = World::start_run(
        content.clone(),
        "ascmearc",
        Some(seed),
        Some(&spec),
        None,
        true,
    )
    .expect("run");
    let factor = Factor::of(content, Some(&spec));
    let player = w.state.player;
    let mut out = Vec::new();
    let mut log = training::Log::default();
    for level in 1..=*LEVELS.last().expect("levels") {
        if level > 1 {
            if trained {
                training::live_level(&mut w, build, level - 1, monsters, seed, &mut log);
            } else {
                log.day += training::days(level);
            }
            let need = content.experience().threshold_for(level, factor);
            let more = need.saturating_sub(w.player_xp());
            let before = training::scores(&w);
            act(&mut w, &format!("xp:{more}"));
            act(&mut w, "levelup:auto levelup:auto");
            log.rises(&w, before);
        }
        if !LEVELS.contains(&level) {
            continue;
        }
        let table = content.weapon_skills();
        for &(skill, less) in build.skills {
            let id = table.id(skill).expect(skill);
            let r = rank(level).saturating_sub(less).max(1);
            if r > w.state.player().weapon_skills.rank(id) {
                w.player_mut().weapon_skills.raise(table, id, r);
            }
        }
        if let Some(id) = content.skill_id("sneak_attack") {
            let have = w.state.player().skills.value(id);
            if let Some(have) = have.filter(|&v| v < sneak_value(level)) {
                let by = i32::from(sneak_value(level) - have);
                w.player_mut().skills.add(id, by);
            }
        }
        for gear in [Gear::Kit, Gear::Plain, Gear::Magic] {
            let mut w2 = w.clone();
            if gear != Gear::Kit {
                dress(&mut w2, build, level, gear, seed);
            }
            let me = w2.state.player();
            let scores = Stat::ALL.map(|s| me.stats.score(s));
            let potentials = Stat::ALL.map(|s| me.stats.potential(s));
            let days = f64::from(log.earned_days.max(1));
            let mods = me.talent_mods(content);
            out.push(Snap {
                class: build.class,
                level,
                gear,
                trained,
                side: side_of(&w2, player),
                scores,
                potentials,
                day: log.day,
                first_rise: log.first_rise,
                marks_a_day: log.earned.map(|m| m as f64 / days),
                mana: me.max_mana(content),
                caster_level: w2.caster_level(player),
                casts: if gear == Gear::Plain {
                    casts(&w2)
                } else {
                    Vec::new()
                },
                sneak: w2.sneak_dice(player),
                ambush_threat: mods.ambush_threat,
                vitals_in: mods.vitals_in.clone(),
            });
        }
    }
    out
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

fn avg(v: impl IntoIterator<Item = f64>) -> f64 {
    let (sum, n) = v.into_iter().fold((0.0, 0u32), |(s, n), x| (s + x, n + 1));
    if n == 0 { f64::NAN } else { sum / f64::from(n) }
}

/// A fight's medians: player hit %, monster hit %, damage a round each way, rounds to kill and
/// to die.
struct Fight {
    hit: f64,
    struck: f64,
    dealt: f64,
    taken: f64,
    kill: f64,
    die: f64,
}

/// The players `snaps` (one per seed) against `monsters`, the monster's Protection replaced by
/// `pv(level)` when given.
fn fight(
    snaps: &[&Snap],
    monsters: &[&Monster],
    pv: Option<fn(u8) -> i32>,
    rng: &mut Rng,
) -> Fight {
    let mut rows = Vec::new();
    for m in monsters {
        let mut per = Vec::new();
        for (s, snap) in snaps.iter().enumerate() {
            let me = &snap.side;
            let them = &m.geared[s % m.geared.len()];
            let mut target = them.target.clone();
            if let Some(pv) = pv {
                target = Target::protected(target.defence, pv(m.level));
            }
            let mut dealt = strike_mean(&me.blow, &target, rng);
            if let Some(off) = &me.off {
                dealt += strike_mean(off, &target, rng);
            }
            let dealt = dealt * me.blows;
            let taken = strike_mean(&them.blow, &me.target, rng) * them.blows;
            per.push((
                hit_pct(me.blow.to_hit, &target.defence),
                hit_pct(them.blow.to_hit, &me.target.defence),
                dealt,
                taken,
                f64::from(them.hp) / dealt.max(0.01),
                f64::from(me.hp) / taken.max(0.01),
            ));
        }
        rows.push(per);
    }
    let pick = |f: fn(&(f64, f64, f64, f64, f64, f64)) -> f64| {
        median(rows.iter().map(|per| avg(per.iter().map(f))).collect())
    };
    Fight {
        hit: pick(|r| r.0),
        struck: pick(|r| r.1),
        dealt: pick(|r| r.2),
        taken: pick(|r| r.3),
        kill: pick(|r| r.4),
        die: pick(|r| r.5),
    }
}

/// Rounds, short: one decimal under 10, whole above, `>999` past it.
fn r(x: f64) -> String {
    if !x.is_finite() || x > 999.0 {
        ">999".into()
    } else if x < 10.0 {
        format!("{x:.1}")
    } else {
        format!("{x:.0}")
    }
}

fn fight_cell(f: &Fight) -> String {
    format!(
        "{:.0}/{:.0} · {:.1}:{:.1} · {} / {}",
        f.hit,
        f.struck,
        f.dealt,
        f.taken,
        r(f.kill),
        r(f.die)
    )
}

/// §8.2's Protection by body at level `l`.
fn hide(l: u8) -> i32 {
    ((i32::from(l) - 10) / 8).max(0)
}
fn bony(l: u8) -> i32 {
    i32::from(l) / 6
}
fn armoured(l: u8) -> i32 {
    i32::from(l) / 4 + 1
}

#[test]
#[ignore = "the balance probe: run by hand, docs/balance.md"]
fn balance_probe() {
    let content = Arc::new(Content::builtin().expect("content"));
    let monsters = monsters(&content);
    let mut snaps = Vec::new();
    for build in &BUILDS {
        for seed in 1..=SEEDS {
            for trained in [true, false] {
                snaps.extend(career(&content, build, seed, trained, &monsters));
            }
        }
    }
    let sweep = training::SWEEP
        .iter()
        .map(|&(per_point, ramp)| {
            let text: String = (include_str!("../../../../assets/data/stats.toml").lines())
                .map(|l| match l {
                    l if l.starts_with("price_per_point = ") => {
                        format!("price_per_point = {per_point}\n")
                    }
                    l if l.starts_with("ramp_divisor = ") => format!("ramp_divisor = {ramp}\n"),
                    l => format!("{l}\n"),
                })
                .collect();
            let mut files = crate::content::ContentFiles::builtin();
            files.insert("data/stats.toml", &text);
            let content = Arc::new(Content::load(&files).expect("swept content"));
            let mut finals = Vec::new();
            for build in &BUILDS {
                for seed in 1..=SEEDS {
                    let last = career(&content, build, seed, true, &monsters);
                    let snap = last.into_iter().last().expect("level 60");
                    finals.push((build.class, snap.scores));
                }
            }
            training::Swept {
                per_point,
                ramp,
                finals,
            }
        })
        .collect();
    let p = Probe {
        content: content.clone(),
        monsters,
        snaps,
        sweep,
    };
    let mut rng = Rng::stream(569, "balance");
    let mut out = String::new();
    tables::players(&p, &mut out);
    tables::defence(&p, &mut out);
    tables::monsters(&p, &mut out);
    tables::fights(&p, &mut rng, &mut out);
    tables::depth(&p, &mut rng, &mut out);
    tables::bodies(&p, &mut rng, &mut out);
    tables::spells(&p, &mut rng, &mut out);
    tables::sneak_attacks(&p, &mut rng, &mut out);
    tables::stats(&p, &mut out);
    tables::stat_sweep(&p, &mut out);
    println!("{out}");
}

/// What the probe measured: every monster, and every player at every level, gear, and seed.
struct Probe {
    content: Arc<Content>,
    monsters: Vec<Monster>,
    snaps: Vec<Snap>,
    /// The trained careers' level-60 scores with other prices and ramps.
    sweep: Vec<training::Swept>,
}

impl Probe {
    /// The players of `class` at `level` in `gear`, trained or not: one per seed.
    fn of(&self, class: &str, level: u8, gear: Gear, trained: bool) -> Vec<&Snap> {
        (self.snaps.iter())
            .filter(|s| s.class == class && s.level == level && s.gear == gear)
            .filter(|s| s.trained == trained)
            .collect()
    }

    /// The monsters `lo..=hi` levels from `level`.
    fn window(&self, level: u8, lo: i32, hi: i32) -> Vec<&Monster> {
        let l = i32::from(level);
        (self.monsters.iter())
            .filter(|m| (l + lo..=l + hi).contains(&i32::from(m.level)))
            .collect()
    }
}

/// `dice` as the data writes it: `4d6`, `2d4+1`.
fn dice_text(dice: Dice) -> String {
    match dice.bonus() {
        0 => format!("{}d{}", dice.count(), dice.sides()),
        b => format!("{}d{}{b:+}", dice.count(), dice.sides()),
    }
}

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
//!   `levelup:auto`. **Trained** stats: before each level, two game days of marks (each
//!   build's `marks`, from the design's per-use arithmetic in §9.1: a fighter's 4 to 5
//!   Strength marks a day from blows and 2 to 3 from criticals and burden, 3 to 4 Toughness
//!   from being hit, 2 to 3 Dexterity from dodges and parries, a caster's 3 to 6 Willpower from
//!   casting; 1 a day in a stat the class does not use) and the daily check at each day's
//!   06:00 (`World::daily_stat_check`), then the level's own check: the game's rules on the
//!   game's streams. **Untrained**: no marks. Weapon
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
//!   caster level, with their trained stats' lift (`Actor::spell_cap`) and without
//!   (`SpellRules::cap_of`), the dice as `SpellDice::lifted` and the talents make them, held to
//!   the cap; attack spells strike the monster's Defence against missiles, areas always hit.
//!
//! Not modelled: triggered and active weapon abilities, stances, talents chosen for a build
//! (`levelup:auto` takes the first it may), sneak attacks, potions, and resistances against
//! spells.

mod tables;

use super::{ActorId, World};
use crate::action::{AttackKind, parse_actions};
use crate::character::{CharacterSpec, Gender};
use crate::combat::{self, Attack, Defence, Profile, Target};
use crate::content::{
    ActionKind, BASE_TURN, Content, CreatureId, CreatureSpawn, Factor, NORMAL_SPEED, Resolution,
    price,
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
/// Game days of marks before each level (the design's "a level every two days", §9.2).
const DAYS_PER_LEVEL: u32 = 2;

/// The gear a player is measured in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Gear {
    Kit,
    Plain,
    Magic,
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
    /// Marks a game day in each stat (`Stat::ALL` order: St, Dx, To, In, Wi, Pr; module docs).
    marks: [u16; 6],
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
        marks: [7, 3, 4, 1, 1, 1],
        gear: warrior,
    },
    Build {
        class: "thief",
        skills: &[("daggers", 0), ("twin_blades", 0)],
        marks: [4, 7, 4, 1, 1, 1],
        gear: thief,
    },
    Build {
        class: "wizard",
        skills: &[("staves", 2), ("spell_attack", 0)],
        marks: [1, 1, 2, 3, 5, 1],
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
        marks: [4, 1, 3, 1, 5, 1],
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
    let base = mean(p.damage) + f64::from(p.bonus) + p.extra.map_or(0.0, mean);
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
    let with = a.weapon_item(content);
    let object = with.map(|i| a.inventory[i].object);
    let skills = a.attack_skills(content, AttackKind::Melee, object);
    let cost = w.attack_price(id, ActionKind::Melee, with, skills);
    let turn = price(NORMAL_SPEED, BASE_TURN) as f64;
    let blows = f64::from(a.speed(content)) / f64::from(NORMAL_SPEED) * turn / cost as f64;
    Side {
        blow: a.melee(content),
        off: a.off_hand_blow(content).map(|(_, p)| p),
        target: w.target_of(id),
        missile: w.target_against(id, true).defence,
        blows,
        hp: a.max_hp(content),
    }
}

/// A creature type, bare and with each seed's gear.
struct Monster {
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
            level: def.level,
            bare,
            geared: sides,
        });
    }
    out
}

/// A spell as one caster casts it.
struct Cast {
    name: String,
    tier: u8,
    resolution: Resolution,
    lift: i32,
    cap: i32,
    lifted: i32,
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
    let Some(rules) = content.spell_rules() else {
        return Vec::new();
    };
    let me = w.state.player();
    let class = (w.state.character.as_ref()).and_then(|c| c.spec.class(content).ok());
    let level = w.caster_level(w.state.player);
    let mods = me.talent_mods(content);
    let mut out = Vec::new();
    for spell in content.spells_by_tier() {
        let def = content.spell(spell);
        let Some(damage) = def.damage else {
            continue;
        };
        let affinity = class.and_then(|c| c.affinity.get(&def.school).copied());
        if !me.knows_spell(content, spell) && affinity.unwrap_or(0) <= 0 {
            continue;
        }
        let comfortable = rules.comfortable[usize::from(def.tier)];
        let cap = rules.cap_of(def);
        let lifted = me.spell_cap(content, def);
        let lift = me.spell_lift(content, def);
        let dice = |c: (i32, i32)| mods.spell_damage(damage.lifted(level, comfortable, c));
        let skills = me.spell_skills(content, def.shape == crate::content::Shape::Touch);
        let benefit = me
            .benefit_of(content, skills)
            .at_distance(3, content.weapon_skills().spell_attacks());
        let attack = Attack {
            damage: Dice::of(1, 2),
            to_hit: me.spell_to_hit(content, def),
            threat: combat::DEFAULT_THREAT,
            crit: combat::DEFAULT_CRIT,
            strength: None,
            kinds: Kinds::NONE,
        };
        out.push(Cast {
            name: def.name.clone(),
            tier: def.tier,
            resolution: def.resolution,
            lift,
            cap,
            lifted,
            dice: (dice((cap, lifted)), dice((cap, cap))),
            aim: Profile::bare(&attack).with(&benefit),
            cost: me.spell_cost(content, def, level),
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
    mana: i32,
    caster_level: i32,
    casts: Vec<Cast>,
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
fn career(content: &Arc<Content>, build: &Build, seed: u64, trained: bool) -> Vec<Snap> {
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
    for level in 1..=*LEVELS.last().expect("levels") {
        if level > 1 {
            if trained {
                for _ in 0..DAYS_PER_LEVEL {
                    let me = w.player_mut();
                    for (stat, marks) in Stat::ALL.into_iter().zip(build.marks) {
                        me.stat_marks[stat] = me.stat_marks[stat].saturating_add(marks);
                    }
                    let now = w.state.time;
                    w.state.next_stat_check = Some(now);
                    w.daily_stat_check(player, now);
                }
            }
            let need = content.experience().threshold_for(level, factor);
            let more = need.saturating_sub(w.player_xp());
            act(&mut w, &format!("xp:{more}"));
            act(&mut w, "levelup:auto levelup:auto");
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
        for gear in [Gear::Kit, Gear::Plain, Gear::Magic] {
            let mut w2 = w.clone();
            if gear != Gear::Kit {
                dress(&mut w2, build, level, gear, seed);
            }
            let me = w2.state.player();
            let scores = Stat::ALL.map(|s| me.stats.score(s));
            out.push(Snap {
                class: build.class,
                level,
                gear,
                trained,
                side: side_of(&w2, player),
                scores,
                mana: me.max_mana(content),
                caster_level: w2.caster_level(player),
                casts: if gear == Gear::Plain {
                    casts(&w2)
                } else {
                    Vec::new()
                },
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
    let mut snaps = Vec::new();
    for build in &BUILDS {
        for seed in 1..=SEEDS {
            for trained in [true, false] {
                snaps.extend(career(&content, build, seed, trained));
            }
        }
    }
    let p = Probe {
        monsters: monsters(&content),
        snaps,
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
    println!("{out}");
}

/// What the probe measured: every monster, and every player at every level, gear, and seed.
struct Probe {
    monsters: Vec<Monster>,
    snaps: Vec<Snap>,
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

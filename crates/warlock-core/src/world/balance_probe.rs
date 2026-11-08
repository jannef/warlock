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
//!   streams. **Untrained**: no marks. Weapon skill ranks come from use (#1095): a trained
//!   career's fights train the main hand's skill and style through `World::practise`; an
//!   untrained career takes its trained twin's ranks; the skills no fight trains are set by
//!   level (`rank`, the trained warrior's pace). Gear: the kit,
//!   plain level-appropriate pieces, or the same pieces magic (rare from level 30, with an
//!   amulet and two rings from 15), equipped with `equip:`.
//! - **A fight**: one round is a normal turn. Each side's expected damage a round is its blows
//!   a round times the mean of `combat::strike` against the other's target over `STRIKES`
//!   draws (the player's off hand adds its blow); the hit chance is `combat::hit_ways` exactly.
//!   Rounds to kill are the monster's hit points over the player's damage a round, rounds to
//!   die the player's hit points over the monster's. Each cell is the median over the monsters
//!   within two levels of the player's (`+3`: one to five levels above).
//! - **Spells** (the spell scaling design §12.4, S7): the wizard's, the witch's, and the
//!   priest's own casts of every spell that rolls (harm or healing) of the schools they know or
//!   have an affinity with, as `Actor::spell_roll` makes it: the rung their trained stats reach,
//!   their caster style's rank, the talents' and items' hooks, nothing holding the roll; beside
//!   it the same roll at style rank 0. The ranks are not set by level: each level's days are
//!   cast through `World::practise`, a pool a day (`caster_pace`), so the pace
//!   is the real curve's. No casts to kill: the probe prints the roll and the pool. The
//!   priest's prayer (#886, `world/orison.rs`) has no roll: a line a level says its name and
//!   what one prayer answers (`World::prayer_state`).
//!
//! - **Sneak attacks** (#688): the thief's first blow on a foe unaware of them against its
//!   plain blow. Its Sneak attack skill is set by level (`sneak_value`), as it rises by use and
//!   `levelup:auto` spends the rolls elsewhere; the dice are `World::sneak_dice` (the skill and
//!   the talents), the foe flat-footed (`World::flat_footed`), Master thief's threat
//!   (`TalentMods::ambush_threat`), and no dice on a type without vitals (`Content::has_vitals`).
//!
//! - **Rage** (#842): the berserker, calm and raging (`rage_of`: the pool granted, Raging and
//!   the stance it forces), and what feeds the pool a round: the foe's blows that `is_hit()`,
//!   the berserker's criticals, and kills, at the content's and the talents' builds.
//!
//! - **Ki** (#846): the ascetic bare-handed (`ki_of`: the hands emptied), with one blow an
//!   attack and with the flurry's second (`World::flurry`'s profile), the pool's maximum
//!   (`Actor::resource_max`), and its pace a round (`World::resource_rate`).
//!
//! - **Zeal** (#910): the paladin with plain blows and with every blow a smite
//!   (`zeal_of`: `World::smite_profile`, and its unholy dice on the unholy among the monsters),
//!   with Banishing smite's doubled dice on the unholy from level 50 and Holy wrath's die on every
//!   blow it reaches from level 40 (`Side::wrath`, as `World::holy_wrathed` rolls it, #914), the
//!   pool's maximum, and what a round builds against a smite's cost, with Zealot too.
//!
//! - **Casters** (#1102, `casters.rs`): each caster type's first damage spell as it casts it
//!   (`Actor::spell_roll` at its stats), what its pool pays for, and its damage a round against
//!   the trained players of its level.
//!
//! Not modelled: triggered and active weapon abilities, stances (but the rage table's), talents chosen for a build
//! (`levelup:auto` takes the first it may), the initiative check's Surprised after the first
//! blow, potions, and resistances against spells.

mod caster_pace;
mod casters;
mod dragon;
mod familiar;
mod ranger;
mod spread;
mod starters;
mod summons;
mod tables;
mod training;

use super::{ActorId, CastBy, World};
use crate::action::{AttackKind, parse_actions};
use crate::character::{CharacterSpec, Gender};
use crate::combat::{self, Attack, Defence, Profile, Target};
use crate::content::{
    ActionKind, BASE_TURN, Content, CreatureId, CreatureSpawn, Factor, NORMAL_SPEED, RANKS,
    Resolution, SpellId, SpellPurpose, StatUse, WeaponSkillId, price,
};
use crate::damage::Kinds;
use crate::dice::{Dice, Roll};
use crate::magic::{LootGrade, loot};
use crate::object::Stack;
use crate::rng::Rng;
use crate::stats::Stat;
use std::collections::BTreeMap;
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

/// The weapon skill rank assumed at a character level for the skills the fights do not train
/// (a shield, a second weapon skill; an untrained career's come from its trained twin): the
/// trained warrior's long blades as `[marks]` and the melee curve bring them ("Weapon pace",
/// #1095).
fn rank(level: u8) -> u8 {
    match level {
        0..=2 => 2,
        3..=4 => 3,
        5..=6 => 4,
        7..=9 => 5,
        10..=14 => 6,
        15..=22 => 7,
        23..=31 => 8,
        32..=41 => 9,
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
    /// Whether it spends mana on an ability (a heal, a blessing) once a fight; with a `school`,
    /// its pace spell is a heal.
    casts_a_kill: bool,
    /// The school whose spells it casts (`caster_pace`: its caster style's rank comes of the
    /// pool a day), if it casts.
    school: Option<&'static str>,
    /// Whether it fights with its pace spell instead of blows (`training`).
    fights_with_spells: bool,
    gear: fn(u8) -> Vec<(&'static str, &'static str)>,
}

const BUILDS: [Build; 8] = [
    Build {
        class: "warrior",
        skills: &[("long_blades", 0), ("sword_and_board", 0)],
        extras: &[(StatUse::Exerted, 5.0)],
        casts_a_kill: false,
        school: None,
        fights_with_spells: false,
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
        school: None,
        fights_with_spells: false,
        gear: thief,
    },
    Build {
        class: "wizard",
        skills: &[("staves", 2)],
        extras: &[(StatUse::Read, 1.0), (StatUse::Identified, 0.3)],
        casts_a_kill: false,
        school: Some("evocation"),
        fights_with_spells: true,
        gear: wizard,
    },
    Build {
        class: "priest",
        skills: &[("maces", 0), ("sword_and_board", 0)],
        extras: &[(StatUse::Prayed, 0.2), (StatUse::Exerted, 2.0)],
        casts_a_kill: true,
        school: Some("restoration"),
        fights_with_spells: false,
        gear: priest,
    },
    Build {
        class: "witch",
        skills: &[("daggers", 2)],
        extras: &[(StatUse::Read, 0.5), (StatUse::Foraged, 1.0)],
        casts_a_kill: false,
        school: Some("necromancy"),
        fights_with_spells: true,
        gear: witch,
    },
    // Measured for the rage table alone (#842).
    Build {
        class: "berserker",
        skills: &[("axes", 0), ("great_blades", 0), ("great_weapon", 0)],
        extras: &[(StatUse::Exerted, 5.0)],
        casts_a_kill: false,
        school: None,
        fights_with_spells: false,
        gear: berserker,
    },
    // Measured for the ki table alone (#846).
    Build {
        class: "ascetic",
        skills: &[("unarmed", 0), ("brawler", 0)],
        extras: &[(StatUse::Exerted, 3.0)],
        casts_a_kill: false,
        school: None,
        fights_with_spells: false,
        gear: ascetic,
    },
    // Measured for the zeal table alone (#910).
    Build {
        class: "paladin",
        skills: &[("long_blades", 0), ("sword_and_board", 0)],
        extras: &[(StatUse::Exerted, 4.0)],
        casts_a_kill: false,
        school: None,
        fights_with_spells: false,
        gear: warrior,
    },
];

/// The builds the stat training's sweep and the tables before "Spells" measure (the witch is
/// measured for necromancy alone, the berserker for its rage, the ascetic for its ki).
const MELEE_TABLES: usize = 4;

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

/// The berserker in armour (no unarmoured fantasy, #842): a great axe, the warrior's mail and plate.
fn berserker(l: u8) -> Vec<(&'static str, &'static str)> {
    let mut g = vec![("main_hand", "great_axe")];
    if l >= 5 {
        g.extend(
            warrior(l)
                .into_iter()
                .filter(|(slot, _)| !matches!(*slot, "main_hand" | "off_hand" | "cloak")),
        );
        g.push(("cloak", "fur_cloak"));
    }
    g
}

/// The ascetic bare-handed and unarmoured (#846): gloves, boots, and a cloak from level 5. A kit
/// that rolled the staff has it taken off where the ki is measured (`ki_of`).
fn ascetic(l: u8) -> Vec<(&'static str, &'static str)> {
    if l < 5 {
        return Vec::new();
    }
    vec![
        ("hands", "leather_gloves"),
        ("feet", "leather_boots"),
        ("cloak", "wool_cloak"),
    ]
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

fn witch(l: u8) -> Vec<(&'static str, &'static str)> {
    let mut g = vec![("main_hand", "dagger")];
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

/// The mean of `roll` with every die showing `reroll` or less drawn once more (`Roll::roll`):
/// a die of `S` sides averages `(r (S + 1) / 2 + (r + 1) + ... + S) / S` for `r` below `S`.
fn roll_mean(roll: Roll, reroll: u32) -> f64 {
    let die = |sides: u32| {
        let s = f64::from(sides.max(1));
        let r = f64::from(reroll.min(sides));
        let high = (s * (s + 1.0) - r * (r + 1.0)) / 2.0;
        (r * (s + 1.0) / 2.0 + high) / s
    };
    let dice: f64 = (roll.groups().iter())
        .map(|d| f64::from(d.count()) * die(d.sides()) + f64::from(d.bonus()))
        .sum();
    dice + f64::from(roll.bonus())
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

/// The mean damage of one melee attack of `side` against `t`: its blow and each of its natural
/// attacks after the first, misses included.
fn melee_mean(side: &Side, t: &Target, rng: &mut Rng) -> f64 {
    let first = strike_mean(&side.blow, t, rng);
    first
        + side
            .more
            .iter()
            .map(|p| strike_mean(p, t, rng))
            .sum::<f64>()
}

/// [`strike_mean`] with `wrath`, Holy wrath's die, on every hit as `World::holy_wrathed` adds
/// it: of the blow's kind, resisted, never soaked, rolled once whatever a critical does.
fn wrathed_mean(p: &Profile, wrath: Option<Dice>, t: &Target, rng: &mut Rng) -> f64 {
    let Some(dice) = wrath else {
        return strike_mean(p, t, rng);
    };
    let total: i64 = (0..STRIKES)
        .map(|_| {
            let blow = combat::strike(p, t, rng);
            let more = if blow.strike.is_hit() {
                crate::damage::resist(dice.roll(rng).max(0), t.resistances.get(blow.kind))
            } else {
                0
            };
            i64::from(blow.damage + more)
        })
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
    /// Its natural attacks after the first, each struck in the same attack (a dragon's claws
    /// and bite; `Actor::more_natural_blows`).
    more: Vec<Profile>,
    target: Target,
    /// Its side caught unawares: flat-footed, nothing of its Dexterity or Dodge.
    flat: Target,
    /// Its Defence against a missile (a spell's bolt or ray).
    missile: Defence,
    /// Blows a round (a normal turn).
    blows: f64,
    hp: i32,
    /// Holy wrath's die on its hits, and whether on every foe (Avatar) or the unholy alone.
    wrath: Option<(Dice, bool)>,
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
    let mods = a.talent_mods(content);
    Side {
        blow: a.melee(content),
        off: a.off_hand_blow(content).map(|(_, p)| p),
        more: (a.more_natural_blows(content).into_iter())
            .map(|(_, p)| p)
            .collect(),
        flat: w.flat_footed(id, true, target.clone()),
        target,
        missile: w.target_against(id, true).defence,
        blows,
        hp: a.max_hp(content),
        wrath: mods.holy_wrath.map(|dice| (dice, mods.avatar.is_some())),
    }
}

/// A creature type, bare and with each seed's gear.
struct Monster {
    creature: CreatureId,
    level: u8,
    /// Its type is one holy wrath falls on (`Content::unholy`).
    unholy: bool,
    bare: Side,
    geared: Vec<Side>,
    /// A caster's spells as it casts them, bare, and its full pool (`casters.rs`).
    spells: Vec<casters::Spell>,
    mana: i32,
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
        let mut spells = (Vec::new(), 0);
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
                status: Vec::new(),
            };
            let id = super::travel::make_creature(&mut w.state, content, &level, &spawn);
            sides.push(side_of(&w, id));
            if seed == 0 {
                spells = casters::spells_of(&w, id);
            }
            w.state.actors.remove(&id);
        }
        let bare = sides.remove(0);
        out.push(Monster {
            creature: cid,
            level: def.level,
            unholy: content.unholy(cid),
            bare,
            geared: sides,
            spells: spells.0,
            mana: spells.1,
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
    /// A heal (else harm).
    heals: bool,
    /// The rung of its ladder the caster's stats and talents reach.
    rung: usize,
    /// The caster style it is cast in and its rank.
    style: Option<(String, u8)>,
    /// The roll (`Actor::spell_roll`) and the face it draws again at or under (the hooks'
    /// `reroll`), and the same at style rank 0.
    dice: Roll,
    reroll: u32,
    bare: Roll,
    bare_reroll: u32,
    /// The attack roll's profile (to-hit and the caster style's benefit), for attack and touch.
    aim: Profile,
    /// What a cast costs (`Actor::spell_cost`), and its cost table's price with no talent.
    cost: i32,
    base_cost: i32,
    /// What its harm does more to a target under one hex (Malison's bite, `status_damage` on a
    /// hex status; `World::lore_blow`), 0 for none.
    hexed: i32,
}

/// The spells that roll (harm or healing) `w`'s player would cast: those they know, and those of
/// the schools their class has an affinity with, but those aimed at a corpse.
fn casts(w: &World) -> Vec<Cast> {
    let content = &w.content;
    if content.spell_rules().is_none() {
        return Vec::new();
    }
    let me = w.state.player();
    let class = (w.state.character.as_ref()).and_then(|c| c.spec.class(content).ok());
    let mut out = Vec::new();
    let table = content.weapon_skills();
    let rules = content.spell_rules().expect("spell rules");
    let bite = (me.talent_mods(content).status_damage.iter())
        .filter(|(status, _)| content.hex_statuses().contains(status))
        .map(|(_, v)| *v)
        .max()
        .unwrap_or(0);
    for spell in content.spells_by_level() {
        let def = content.spell(spell);
        // A spell aimed at a corpse wants one the probe never lays (Charnel burst, Grave feast).
        if def.aims_corpse() {
            continue;
        }
        let Some(rolled) = me.spell_roll(content, def, CastBy::Own) else {
            continue;
        };
        let affinity = class.and_then(|c| c.affinity.get(&def.school).copied());
        if !me.knows_spell(content, spell) && affinity.unwrap_or(0) <= 0 {
            continue;
        }
        let mods = me.spell_mods_in_style(content, def);
        let rung = me.spell_rung(content, def, CastBy::Own, &mods);
        let skills = me.spell_skills(content, def);
        let mut bare = me.clone();
        for id in [skills.skill, skills.style].into_iter().flatten() {
            bare.weapon_skills.raise(id, 0);
        }
        let bare_reroll = bare.spell_mods_in_style(content, def).reroll;
        let bare = (bare.spell_roll(content, def, CastBy::Own)).unwrap_or(rolled);
        let style = skills
            .style
            .map(|id| (table.skill(id).name.clone(), me.weapon_skills.rank(id)));
        let benefit = me
            .benefit_of(content, skills)
            .at_distance(3, content.weapon_skills().spell_attacks());
        let attack = Attack {
            damage: Dice::of(1, 2).into(),
            to_hit: me.spell_to_hit(content, def),
            threat: combat::DEFAULT_THREAT,
            crit: combat::DEFAULT_CRIT,
            strength: None,
            finesse: false,
            kinds: Kinds::NONE,
        };
        out.push(Cast {
            spell,
            name: def.name.clone(),
            level: def.level,
            resolution: def.resolution,
            heals: def.kind == SpellPurpose::Heal,
            rung,
            style,
            dice: rolled,
            reroll: mods.reroll,
            bare,
            bare_reroll,
            aim: Profile::bare(&attack).with(&benefit),
            cost: me.spell_cost(content, def),
            base_cost: rules.mana(def, 0),
            hexed: if def.kind == SpellPurpose::Damage {
                bite
            } else {
                0
            },
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
    /// Its Decipher script (0 untrained) and its learning (0 when it cannot learn; #1270).
    decipher: i32,
    learning: i32,
    casts: Vec<Cast>,
    /// The level before this one's casting (`caster_pace`) and its spell casts in the fights
    /// (`training`; a healer's one a kill), a day.
    paced: caster_pace::Paced,
    fought_a_day: f64,
    /// Its weapon skills and styles, and the rank of the caster style it was paced in.
    ranks: crate::weapon_skills::WeaponSkills,
    style_rank: Option<u8>,
    /// The weapon skills and styles its fights trained (trained careers) and the level each
    /// first stood at each rank (#1095, "Weapon pace").
    reached: BTreeMap<WeaponSkillId, [Option<u8>; RANKS]>,
    /// Its sneak dice (`World::sneak_dice`) and Master thief's lower threat on a sneak attack.
    sneak: Option<Dice>,
    ambush_threat: u8,
    /// Raging, in plain gear (a class with the Rage ability).
    rage: Option<Rage>,
    /// Its ki, bare-handed in plain gear (a class whose Flurry spends ki).
    ki: Option<Ki>,
    /// Its zeal and smites, in plain gear (a class with Smite).
    zeal: Option<Zeal>,
    /// Its prayer's name in the menu and its state, in plain gear (a class that prays): no roll,
    /// so the spell table prints what it answers.
    prayer: Option<(String, super::orison::PrayerState)>,
}

/// A bare-handed player's ki (#846): their side with one blow an attack and with the flurry's
/// second, the pool's maximum, and the ki that comes back a round.
#[derive(Clone)]
struct Ki {
    single: Side,
    flurry: Side,
    max: u16,
    /// Ki back a round (a normal turn), at its pace.
    back: f64,
}

/// Player `id`'s ki in `w`, if their Flurry spends it: their hands emptied, the second blow as
/// `World::flurry` makes it.
fn ki_of(w: &World, id: ActorId) -> Option<Ki> {
    let content = w.content.clone();
    let cost = (w.state.actors[&id].talent_mods(&content)).flurry_cost?;
    let ki = cost.resource;
    let mut w = w.clone();
    if w.state.actors[&id].weapon_item(&content).is_some() {
        act(&mut w, "remove:main_hand");
    }
    let me = &w.state.actors[&id];
    assert!(me.weapon_item(&content).is_none(), "bare-handed");
    let single = side_of(&w, id);
    let mut second = me.melee(&content);
    second.to_hit += super::free_blow::FLURRY_TO_HIT;
    let every = match content.resource(ki).over_time {
        Some(crate::content::OverTime::Regenerates { every }) => f64::from(every),
        _ => f64::INFINITY,
    };
    let turn = BASE_TURN.as_seconds() as f64;
    Some(Ki {
        flurry: Side {
            off: Some(second),
            ..single.clone()
        },
        single,
        max: me.resource_max(&content, ki, w.holder_level(id)),
        back: turn / every * f64::from(w.resource_rate(id, ki)) / 100.0,
    })
}

/// A smiting player (#910): their side with every blow a smite, on a foe and on an unholy one,
/// and a Banishing smite on an unholy one if they have it, the zeal pool, what comes back a round,
/// what a smite costs, and what a blow that hits and a blow taken build, and a blow taken with
/// Zealot.
#[derive(Clone)]
struct Zeal {
    smite: Side,
    unholy: Side,
    banishing: Option<Side>,
    max: u16,
    /// Zeal back a round (a normal turn), at its pace.
    back: f64,
    cost: u16,
    /// Zeal a blow of theirs that hits (`hit`) and a blow taken (`struck`) build.
    builds: [f64; 2],
    /// Zeal a blow taken builds with Zealot.
    zealot: f64,
}

/// Player `id`'s zeal in `w`, if they have Smite: the smite as `World::smite_profile` makes it
/// (the player's own type is not unholy), again with the unholy dice, and with those twice over as
/// `World::mode_profile` makes Banishing smite's on an unholy foe not summoned.
fn zeal_of(w: &World, id: ActorId) -> Option<Zeal> {
    use crate::content::{Build as Event, ClassActive};
    let content = w.content.clone();
    let me = &w.state.actors[&id];
    let mods = me.talent_mods(&content);
    let smite = mods.smite?;
    let cost = content.class_active(ClassActive::Smite)?.resource?;
    let zeal = cost.resource;
    let def = content.resource(zeal);
    let side = side_of(w, id);
    let blow = w.smite_profile(id)?;
    let dice = smite.dice_at(w.holder_level(id), true);
    let unholy = Profile {
        extra: Some(dice),
        ..blow.clone()
    };
    let banishing = mods
        .smite_mode(ClassActive::BanishingSmite)
        .map(|_| Profile {
            extra: Some(dice.and(dice)),
            ..blow.clone()
        })
        .map(|blow| Side {
            blow,
            ..side.clone()
        });
    let every = match def.over_time {
        Some(crate::content::OverTime::Regenerates { every }) => f64::from(every),
        _ => f64::INFINITY,
    };
    let built = |mods: &crate::talent::TalentMods, on: Event| {
        let more: u16 = (mods.resource_builds.iter())
            .filter(|(r, b, _)| *r == zeal && *b == on)
            .map(|(_, _, n)| *n)
            .sum();
        f64::from(def.built_by(on) + more)
    };
    let mut twin = me.clone();
    twin.talents.take("zealot", None);
    let turn = BASE_TURN.as_seconds() as f64;
    Some(Zeal {
        smite: Side {
            blow,
            ..side.clone()
        },
        unholy: Side {
            blow: unholy,
            ..side
        },
        banishing,
        max: me.resource_max(&content, zeal, w.holder_level(id)),
        back: turn / every * f64::from(w.resource_rate(id, zeal)) / 100.0,
        cost: cost.amount,
        builds: [built(&mods, Event::Hit), built(&mods, Event::Struck)],
        zealot: built(&twin.talent_mods(&content), Event::Struck),
    })
}

/// A raging player (#842): their side with the frenzy stance and Raging on, the pool as the
/// Rage ability grants it, and what its builds and decay are worth.
#[derive(Clone)]
struct Rage {
    side: Side,
    start: u16,
    max: u16,
    /// Rage a hit taken (`struck`), a critical of theirs, and a kill build: the data's and the
    /// talents'.
    builds: [f64; 3],
    /// Rage lost a round (a normal turn), at its decay's rate.
    decay: f64,
}

/// Player `id`'s rage in `w`, if their talents give them the Rage ability: granted at once (the
/// dev grant, with its status and the stance it brings).
fn rage_of(w: &World, id: ActorId) -> Option<Rage> {
    use crate::content::Build as Event;
    let content = w.content.clone();
    let rage = (w.state.actors[&id].talent_mods(&content)).rage?;
    let mut w = w.clone();
    act(&mut w, &format!("resource:{}", content.resource(rage).id));
    let me = &w.state.actors[&id];
    let mods = me.talent_mods(&content);
    let def = content.resource(rage);
    let built = |on: Event| {
        let more: u16 = (mods.resource_builds.iter())
            .filter(|(r, b, _)| *r == rage && *b == on)
            .map(|(_, _, n)| *n)
            .sum();
        f64::from(def.built_by(on) + more)
    };
    let every = match def.over_time {
        Some(crate::content::OverTime::Decays { every }) => f64::from(every),
        _ => f64::INFINITY,
    };
    let turn = BASE_TURN.as_seconds() as f64;
    Some(Rage {
        side: side_of(&w, id),
        start: me.resources.get(rage).map_or(0, |p| p.value),
        max: me.resource_max(&content, rage, w.holder_level(id)),
        builds: [
            built(Event::Struck),
            built(Event::Critical),
            built(Event::Kill),
        ],
        decay: turn / every * f64::from(w.resource_rate(id, rage)) / 100.0,
    })
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

/// One career of `build` from level 1 to the last of [`LEVELS`], measured at each. An
/// untrained career takes the weapon ranks of its trained `twin` (the same build and seed).
fn career(
    content: &Arc<Content>,
    build: &Build,
    seed: u64,
    (trained, twin): (bool, Option<&[Snap]>),
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
    let mut paced = caster_pace::first_spell(&w, build, seed);
    for level in 1..=*LEVELS.last().expect("levels") {
        if level > 1 {
            if trained {
                training::live_level(&mut w, build, level - 1, monsters, seed, &mut log);
            } else {
                log.day += training::days(level);
            }
            paced = caster_pace::cast_level(&mut w, build, level - 1, seed);
            let need = content.experience().threshold_for(level, factor);
            let more = need.saturating_sub(w.player_xp());
            let before = training::scores(&w);
            act(&mut w, &format!("xp:{more}"));
            act(&mut w, "levelup:auto levelup:auto");
            log.rises(&w, before);
            log.note_ranks(&w, level);
        }
        if !LEVELS.contains(&level) {
            continue;
        }
        let table = content.weapon_skills();
        for &(skill, less) in build.skills {
            let id = table.id(skill).expect(skill);
            // Ranks trained by the fights' blows stay as they came (#1095).
            if trained && log.practised.contains(&id) {
                continue;
            }
            let twin = twin.and_then(|t| t.iter().find(|s| s.level == level));
            let r = match twin {
                Some(t) => t.ranks.rank(id),
                None => rank(level).saturating_sub(less).max(1),
            };
            if r > w.state.player().weapon_skills.rank(id) {
                w.player_mut().weapon_skills.raise(id, r);
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
                decipher: (content.skill_id("decipher_script"))
                    .and_then(|id| me.skills.value(id))
                    .map_or(0, i32::from),
                learning: me.learning(content).unwrap_or(0),
                casts: if gear == Gear::Plain {
                    casts(&w2)
                } else {
                    Vec::new()
                },
                ranks: me.weapon_skills.clone(),
                style_rank: paced.style_id.map(|id| me.weapon_skills.rank(id)),
                reached: log.reached.clone(),
                paced: paced.clone(),
                fought_a_day: f64::from(log.fought_casts) / days,
                sneak: w2.sneak_dice(player),
                ambush_threat: mods.ambush_threat,
                rage: (gear == Gear::Plain)
                    .then(|| rage_of(&w2, player))
                    .flatten(),
                ki: (gear == Gear::Plain).then(|| ki_of(&w2, player)).flatten(),
                zeal: (gear == Gear::Plain)
                    .then(|| zeal_of(&w2, player))
                    .flatten(),
                prayer: (gear == Gear::Plain)
                    .then(|| {
                        let title = me.prayer_title(content)?.0.to_string();
                        Some((title, w2.prayer_state()?))
                    })
                    .flatten(),
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
    let sides: Vec<&Side> = snaps.iter().map(|s| &s.side).collect();
    fight_sides(&sides, monsters, pv, rng)
}

/// [`fight`] of the players' `sides` (one per seed; a raging side, #842).
fn fight_sides(
    sides: &[&Side],
    monsters: &[&Monster],
    pv: Option<fn(u8) -> i32>,
    rng: &mut Rng,
) -> Fight {
    let rows = per_monster(sides, monsters, pv, rng);
    let pick = |f: fn(&Row) -> f64| median(rows.iter().map(f).collect());
    Fight {
        hit: pick(|r| r.0),
        struck: pick(|r| r.1),
        dealt: pick(|r| r.2),
        taken: pick(|r| r.3),
        kill: pick(|r| r.4),
        die: pick(|r| r.5),
    }
}

/// One monster's fight, the mean over the players' seeds: player hit %, monster hit %, damage a
/// round each way, rounds to kill and to die.
type Row = (f64, f64, f64, f64, f64, f64);

/// The players' `sides` against each of `monsters`, a [`Row`] each in their order.
fn per_monster(
    sides: &[&Side],
    monsters: &[&Monster],
    pv: Option<fn(u8) -> i32>,
    rng: &mut Rng,
) -> Vec<Row> {
    let mut rows = Vec::new();
    for m in monsters {
        let mut per = Vec::new();
        for (s, me) in sides.iter().enumerate() {
            let them = &m.geared[s % m.geared.len()];
            let mut target = them.target.clone();
            if let Some(pv) = pv {
                target = Target::protected(target.defence, pv(m.level));
            }
            let wrath = (me.wrath)
                .filter(|&(_, every_foe)| every_foe || m.unholy)
                .map(|(dice, _)| dice);
            let mut dealt = wrathed_mean(&me.blow, wrath, &target, rng);
            if let Some(off) = &me.off {
                dealt += wrathed_mean(off, wrath, &target, rng);
            }
            let dealt = dealt * me.blows;
            let taken = melee_mean(them, &me.target, rng) * them.blows;
            per.push((
                hit_pct(me.blow.to_hit, &target.defence),
                hit_pct(them.blow.to_hit, &me.target.defence),
                dealt,
                taken,
                f64::from(them.hp) / dealt.max(0.01),
                f64::from(me.hp) / taken.max(0.01),
            ));
        }
        let mean = |f: fn(&Row) -> f64| avg(per.iter().map(f));
        rows.push((
            mean(|r| r.0),
            mean(|r| r.1),
            mean(|r| r.2),
            mean(|r| r.3),
            mean(|r| r.4),
            mean(|r| r.5),
        ));
    }
    rows
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
            let trained = career(&content, build, seed, (true, None), &monsters);
            let untrained = career(&content, build, seed, (false, Some(&trained)), &monsters);
            snaps.extend(trained);
            snaps.extend(untrained);
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
            for build in &BUILDS[..MELEE_TABLES] {
                for seed in 1..=SEEDS {
                    let last = career(&content, build, seed, (true, None), &monsters);
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
    spread::spread(&p, &mut Rng::stream(1134, "spread"), &mut out);
    starters::starters(&p, &mut Rng::stream(1200, "starters"), &mut out);
    dragon::dragons(&p, &mut Rng::stream(1099, "dragons"), &mut out);
    casters::casters(&p, &mut Rng::stream(1102, "casters"), &mut out);
    tables::bodies(&p, &mut rng, &mut out);
    tables::spells(&p, &mut out);
    tables::caster_pace(&p, &mut out);
    tables::weapon_pace(&p, &mut out);
    tables::sneak_attacks(&p, &mut rng, &mut out);
    tables::rage(&p, &mut rng, &mut out);
    tables::ki(&p, &mut rng, &mut out);
    tables::zeal(&p, &mut rng, &mut out);
    ranger::ranger_and_hound(&p, &mut rng, &mut out);
    summons::summons(&p, &mut rng, &mut out);
    familiar::familiar(&p, &mut out);
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

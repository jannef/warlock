//! Weapon abilities (spec §8 "Weapon skills", "Fighting styles"; the combat progression design
//! §4): what a rank in a weapon skill or a fighting style teaches. One `[[weapon_ability]]` a
//! file in `data/weapon_abilities/` (a campaign's may add more), read in path order:
//! [`WeaponAbilityId`] is an index in that order, in the content fingerprint, and never saved.
//!
//! ```toml
//! [[weapon_ability]]
//! id = "rend"
//! name = "Rend"
//! description = "When both your blades bite in one attack, ..."
//! skills = ["twin_blades"]   # one or more weapon skills, or one or more styles
//! rank = 4
//! kind = "triggered"
//! trigger = "both_hands_hit" # or a list of triggers
//! systems = ["weapon_abilities", "styles"]
//! [weapon_ability.effect]
//! kind = "extra_damage"
//! dice = "main_hand"
//! ```
//!
//! An ability names its skills; a skill does not list its abilities. A **triggered** ability
//! fires by itself on a combat event ([`Trigger`]), instantly and for free, so it has no
//! `stamina`, `energy`, or `cooldown`; an effect that is a switch ([`WeaponDoes::FreeBlow`],
//! [`WeaponDoes::ParryMissiles`]) has no `trigger` either: the switch is read where its event
//! already is. An **active** ability (#564, W3) is used from the abilities menu, aimed as its
//! `aim` says, and paid in two things after its own refusals ([`Active`]): its `stamina` points
//! and its `energy`, a percent (50 to 300) of the price of the action it is, so none is instant.
//!
//! ```toml
//! [[weapon_ability]]
//! id = "lunge"
//! ...
//! kind = "active"
//! aim = "direction"            # nothing | direction | cell
//! stamina = 3
//! energy = { action = "melee", percent = 100 }
//! [weapon_ability.effect]
//! kind = "reach_blow"
//! reach = 2
//! ```

use super::{ActionKind, Content, ContentError, StatusId, WeaponSkillId, duplicate_id};
use crate::combat::Category;
use crate::dice::Dice;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The resource (`data/resources.toml`) active abilities are paid from.
pub const STAMINA: &str = "stamina";

/// The highest rank an ability may ask for.
pub const MAX_ABILITY_RANK: u8 = 10;

/// A weapon ability: an index in [`Content::weapon_abilities`], in path order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WeaponAbilityId(pub u16);

/// The combat events triggered abilities fire on (the design §4.3), each raised at one line that
/// already marks it (`world/weapon_ability.rs`). Closed: [`Trigger::ALL`] in the wiki's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Trigger {
    /// A blow, shot, or throw of the actor's hits (soaked or resisted too: it landed).
    Hit,
    /// A critical of the actor's is confirmed.
    Critical,
    /// A blow, shot, or throw of the actor's is a big hit (a third of the target's hit points)
    /// and leaves it alive.
    BigHit,
    /// A blow, shot, or throw of the actor's kills.
    Kill,
    /// A sneak attack of the actor's hits: its sneak dice land (`world/ambush.rs`).
    SneakAttack,
    /// The actor is hit.
    Struck,
    /// A melee attack on the actor missed by its Parry.
    Parried,
    /// A melee attack on the actor missed by its Shield.
    Blocked,
    /// A melee attack on the actor missed by its Dexterity or Dodge.
    Dodged,
    /// A hostile's spell that rolls an attack against the actor's Defence (a bolt, a ray, a
    /// touch; a wand's too) missed by its Shield.
    SpellBlocked,
    /// Both blows of the actor's two-weapon attack hit the same target, which lives.
    BothHandsHit,
    /// A hostile steps up beside the actor.
    Approaches,
}

impl Trigger {
    /// Every trigger, in the wiki's order.
    pub const ALL: [Trigger; 12] = [
        Trigger::Hit,
        Trigger::Critical,
        Trigger::BigHit,
        Trigger::Kill,
        Trigger::SneakAttack,
        Trigger::Struck,
        Trigger::Parried,
        Trigger::Blocked,
        Trigger::Dodged,
        Trigger::SpellBlocked,
        Trigger::BothHandsHit,
        Trigger::Approaches,
    ];

    /// Its id, as data writes it.
    pub const fn id(self) -> &'static str {
        match self {
            Trigger::Hit => "hit",
            Trigger::Critical => "critical",
            Trigger::BigHit => "big_hit",
            Trigger::Kill => "kill",
            Trigger::SneakAttack => "sneak_attack",
            Trigger::Struck => "struck",
            Trigger::Parried => "parried",
            Trigger::Blocked => "blocked",
            Trigger::Dodged => "dodged",
            Trigger::SpellBlocked => "spell_blocked",
            Trigger::BothHandsHit => "both_hands_hit",
            Trigger::Approaches => "approaches",
        }
    }

    /// When it fires, in words (the wiki).
    pub const fn describe(self) -> &'static str {
        match self {
            Trigger::Hit => "a blow, shot, throw, or spell attack of yours hits",
            Trigger::Critical => "a critical of yours is confirmed",
            Trigger::BigHit => "a hit of yours takes a third of the target's hit points",
            Trigger::Kill => "a blow, shot, or throw of yours kills",
            Trigger::SneakAttack => "a sneak attack of yours hits",
            Trigger::Struck => "you are hit",
            Trigger::Parried => "a melee attack on you misses by your Parry",
            Trigger::Blocked => "a melee attack on you misses by your Shield",
            Trigger::Dodged => "a melee attack on you misses by your Dexterity or Dodge",
            Trigger::SpellBlocked => "a spell's attack on you misses by your Shield",
            Trigger::BothHandsHit => "both blows of your two-weapon attack hit, and the foe lives",
            Trigger::Approaches => "a hostile steps up beside you",
        }
    }

    /// Whether the actor it fires for is the one attacked (its skills are what it holds), not
    /// the attacker (its skills are the attack's).
    pub const fn defending(self) -> bool {
        matches!(
            self,
            Trigger::Struck
                | Trigger::Parried
                | Trigger::Blocked
                | Trigger::Dodged
                | Trigger::SpellBlocked
                | Trigger::Approaches
        )
    }

    fn parse(id: &str) -> Result<Self, String> {
        (Trigger::ALL.into_iter().find(|t| t.id() == id)).ok_or_else(|| {
            let ids: Vec<&str> = Trigger::ALL.iter().map(|t| t.id()).collect();
            format!("unknown trigger `{id}` ({})", ids.join(", "))
        })
    }
}

/// The free blow a `free_blow` effect switches on (`FreeBlows` in `world/free_blow.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreeBlowKind {
    Riposte,
    Cleave,
    GreatCleave,
    Opportunist,
    Gatekeeper,
}

impl FreeBlowKind {
    const ALL: [FreeBlowKind; 5] = [
        FreeBlowKind::Riposte,
        FreeBlowKind::Cleave,
        FreeBlowKind::GreatCleave,
        FreeBlowKind::Opportunist,
        FreeBlowKind::Gatekeeper,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            FreeBlowKind::Riposte => "riposte",
            FreeBlowKind::Cleave => "cleave",
            FreeBlowKind::GreatCleave => "great_cleave",
            FreeBlowKind::Opportunist => "opportunist",
            FreeBlowKind::Gatekeeper => "gatekeeper",
        }
    }
}

/// Whose a `status` effect's status is: the other end of the event (the one struck, or the
/// attacker of a defending trigger), or the actor's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum On {
    Target,
    Own,
}

/// The dice an `extra_damage` effect rolls once more.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtraDice {
    /// The main hand's weapon's damage dice.
    MainHand,
    /// The off hand's weapon's.
    OffHand,
    /// These.
    Fixed(Dice),
}

/// A set of Defence categories: what an `ignore_defence` effect rolls without.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Categories(u16);

impl Categories {
    /// These and `category`.
    pub const fn with(self, category: Category) -> Self {
        Self(self.0 | 1 << category as u16)
    }

    pub const fn contains(self, category: Category) -> bool {
        self.0 & 1 << category as u16 != 0
    }

    /// Each, in category order.
    pub fn iter(self) -> impl Iterator<Item = Category> {
        Category::ALL.into_iter().filter(move |&c| self.contains(c))
    }
}

/// What an active ability asks for before it is used (the menu's aim).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveAim {
    Nothing,
    Direction,
    Cell,
}

/// How an active ability is used and paid for (the design §4.5): its aim, its stamina, and its
/// energy, a percent of the price of the action it is (a blow's `attack_price`, a shot's
/// `shot_price`, a wait's for a stance-like buff).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Active {
    pub aim: ActiveAim,
    /// Points of the stamina pool it costs.
    pub stamina: u16,
    /// The action it is priced as.
    pub action: ActionKind,
    /// The percent of that action's price it takes (50 to 300).
    pub percent: u32,
    /// A status with `eases` and no tick given as it is used, while which it is unavailable:
    /// the optional per-ability limiter (none in the catalogue).
    pub cooldown: Option<StatusId>,
}

/// The lowest and highest energy percent of an active.
pub const ENERGY_PERCENT: std::ops::RangeInclusive<u32> = 50..=300;

/// What a weapon ability does (the design §4.4): a closed list, each on one existing mechanism.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeaponDoes {
    /// Switches a free blow on for the holder (read by `World::free_blows_of`).
    FreeBlow(FreeBlowKind),
    /// `status` on the other end or on the actor: `severity` degrees, or degrees drawn from
    /// `turns` (one draw, as a hold's turns are).
    Status {
        status: StatusId,
        severity: u16,
        turns: Option<Dice>,
        on: On,
    },
    /// One more roll of `dice` (no bonus, no riders) on the target, after the blow's own damage,
    /// of the blow's kind of damage (or `kind`), resisted but never soaked.
    ExtraDamage {
        dice: ExtraDice,
        kind: Option<super::DamageTypeId>,
    },
    /// A shot (or, with nothing to shoot, a throw) at the one that stepped up beside the actor,
    /// free, with what it has ready; nothing without.
    FreeShot,
    /// Switches on Parry against shots and throws for the holder.
    ParryMissiles,
    /// The blocked spell is cast back at its caster from the actor's cell, as the caster cast
    /// it, at most `per_turn` times a turn of the actor's; a reflected spell is never reflected
    /// again (`world/reflect.rs`).
    ReflectSpell { per_turn: u8 },
    /// (Active) a blow at the first hostile along the direction within `reach` cells, the cells
    /// between empty of actors and passable: Lunge.
    ReachBlow { reach: u8 },
    /// (Active) a blow that is a critical if it hits, with no confirmation: Vital strike.
    SureCritical,
    /// (Active) a blow rolled against the target's Defence without `categories`: Feint.
    IgnoreDefence { categories: Categories },
    /// (Active) a blow, or a shot, that the target's Protection does not soak: Sunder, Gap shot.
    IgnoreProtection,
    /// (Active) one blow at each hostile beside the actor, clockwise from the direction aimed
    /// (or the north), each at `to_hit`; with a weapon in each hand the blades take turns: Whirl,
    /// Staff sweep.
    Sweep { to_hit: i32 },
    /// (Active) a shot that flies on past the first it hits, to the next on its line: Piercing
    /// shot.
    PierceShot,
    /// (Active) a blow (a shield bash with `bash`) that drives a foe it hits `cells` back, if
    /// the cell is free, and leaves it suffering `status` (one degree): Shield rush, Hip throw.
    Knockback {
        cells: u8,
        bash: bool,
        status: Option<StatusId>,
    },
    /// (Active) `status`, `severity` degrees, on the actor: Shield wall.
    Buff { status: StatusId, severity: u16 },
    /// (Active) a blow whose damage dice are rolled twice (its bonus and riders once; no
    /// critical), at `to_hit`: Mighty swing.
    DoubleDice { to_hit: i32 },
}

impl WeaponDoes {
    /// Its kind's id, as data writes it.
    pub const fn kind(&self) -> &'static str {
        match self {
            WeaponDoes::FreeBlow(_) => "free_blow",
            WeaponDoes::Status { .. } => "status",
            WeaponDoes::ExtraDamage { .. } => "extra_damage",
            WeaponDoes::FreeShot => "free_shot",
            WeaponDoes::ParryMissiles => "parry_missiles",
            WeaponDoes::ReflectSpell { .. } => "reflect_spell",
            WeaponDoes::ReachBlow { .. } => "reach_blow",
            WeaponDoes::SureCritical => "sure_critical",
            WeaponDoes::IgnoreDefence { .. } => "ignore_defence",
            WeaponDoes::IgnoreProtection => "ignore_protection",
            WeaponDoes::Sweep { .. } => "sweep",
            WeaponDoes::PierceShot => "pierce_shot",
            WeaponDoes::Knockback { .. } => "knockback",
            WeaponDoes::Buff { .. } => "buff",
            WeaponDoes::DoubleDice { .. } => "double_dice",
        }
    }

    /// Whether it is an active's effect (used, not fired).
    pub const fn is_active(&self) -> bool {
        matches!(
            self,
            WeaponDoes::ReachBlow { .. }
                | WeaponDoes::SureCritical
                | WeaponDoes::IgnoreDefence { .. }
                | WeaponDoes::IgnoreProtection
                | WeaponDoes::Sweep { .. }
                | WeaponDoes::PierceShot
                | WeaponDoes::Knockback { .. }
                | WeaponDoes::Buff { .. }
                | WeaponDoes::DoubleDice { .. }
        )
    }

    /// Whether it is a switch, read where its own event already is: no trigger of its own.
    pub const fn is_switch(&self) -> bool {
        matches!(self, WeaponDoes::FreeBlow(_) | WeaponDoes::ParryMissiles)
    }
}

/// A weapon ability (`data/weapon_abilities/`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeaponAbility {
    pub id: String,
    pub name: String,
    /// What it does, in the game's words (the abilities menu's line): no number.
    pub description: String,
    /// The weapon skills, or the fighting styles, whose rank teaches it, in skill order.
    pub skills: Vec<WeaponSkillId>,
    /// The rank in one of them that teaches it (1 to 10).
    pub rank: u8,
    /// What it fires on; empty for a switch.
    pub triggers: Vec<Trigger>,
    /// The systems it touches (the wiki's): ids of the talent table's systems (the shipped
    /// abilities' are checked against it by a test; a test's own talent table may have others).
    pub systems: Vec<String>,
    pub effect: WeaponDoes,
    /// How it is used and paid for, for an active (`kind = "active"`); `None`: triggered.
    pub active: Option<Active>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WeaponAbilityFile {
    #[serde(default)]
    weapon_ability: Vec<WeaponAbilitySpec>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WeaponAbilitySpec {
    id: String,
    name: String,
    description: String,
    skills: Vec<String>,
    rank: u8,
    kind: String,
    trigger: Option<OneOrMany>,
    // An active's: refused on a triggered ability.
    aim: Option<String>,
    stamina: Option<u16>,
    energy: Option<EnergySpec>,
    cooldown: Option<String>,
    #[serde(default)]
    systems: Vec<String>,
    effect: EffectSpec,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EnergySpec {
    action: String,
    percent: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectSpec {
    kind: String,
    reach: Option<u8>,
    categories: Option<Vec<String>>,
    to_hit: Option<i32>,
    cells: Option<u8>,
    bash: Option<bool>,
    trigger: Option<String>,
    status: Option<String>,
    severity: Option<u16>,
    turns: Option<Dice>,
    on: Option<String>,
    dice: Option<String>,
    #[serde(rename = "damage")]
    damage_kind: Option<String>,
    per_turn: Option<u8>,
}

/// Parses the weapon ability files, in path order; weapon skills, statuses, damage types, and
/// the talent table's systems must be loaded.
pub(super) fn parse<'a>(
    files: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Vec<WeaponAbility>, ContentError> {
    let mut all = Vec::new();
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    for (path, text) in files {
        let file: WeaponAbilityFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        for spec in file.weapon_ability {
            if let Some(first) = seen.get(&spec.id) {
                return Err(duplicate_id("weapon ability", &spec.id, path, first));
            }
            seen.insert(spec.id.clone(), path.to_string());
            let id = spec.id.clone();
            let ability = resolve(spec, content)
                .map_err(|m| ContentError::new(path, format!("weapon ability `{id}`: {m}")))?;
            all.push(ability);
        }
    }
    check_talents(&all, content)?;
    if all.iter().any(|a| a.active.is_some()) && content.resource_id(STAMINA).is_none() {
        return Err(ContentError::new(
            "data/resources.toml",
            format!("active weapon abilities are paid in `{STAMINA}`, which is no resource"),
        ));
    }
    Ok(all)
}

/// Every weapon ability a talent names (`weapon_stamina`'s `abilities`) is one of `all`, and an
/// active, while there are any (a content without weapon abilities leaves the names unread).
fn check_talents(all: &[WeaponAbility], content: &Content) -> Result<(), ContentError> {
    if all.is_empty() {
        return Ok(());
    }
    for (_, def) in content.talents().iter() {
        for effect in def.effects_at(def.rank_count()) {
            let super::TalentEffect::WeaponStamina { abilities, .. } = effect else {
                continue;
            };
            for name in abilities {
                if !all.iter().any(|a| a.id == name && a.active.is_some()) {
                    return Err(ContentError::new(
                        "data/talents",
                        format!("talent `{}`: `{name}` is no active weapon ability", def.id),
                    ));
                }
            }
        }
    }
    Ok(())
}

fn resolve(spec: WeaponAbilitySpec, content: &Content) -> Result<WeaponAbility, String> {
    if spec.name.trim().is_empty() || spec.description.trim().is_empty() {
        return Err("`name` and `description` are not empty".into());
    }
    let table = content.weapon_skills();
    if spec.skills.is_empty() {
        return Err("`skills` names one or more weapon skills or styles".into());
    }
    let mut skills = Vec::new();
    for name in &spec.skills {
        let id = table
            .id(name)
            .ok_or_else(|| format!("unknown weapon skill `{name}`"))?;
        if skills.contains(&id) {
            return Err(format!("`{name}` twice in `skills`"));
        }
        skills.push(id);
    }
    let styles = skills
        .iter()
        .filter(|&&s| table.skill(s).is_style())
        .count();
    if styles != 0 && styles != skills.len() {
        return Err("`skills` names weapon skills or styles, not both".into());
    }
    skills.sort();
    if !(1..=MAX_ABILITY_RANK).contains(&spec.rank) {
        return Err(format!("`rank` is 1 to {MAX_ABILITY_RANK}"));
    }
    let costs = [
        ("aim", spec.aim.is_some()),
        ("stamina", spec.stamina.is_some()),
        ("energy", spec.energy.is_some()),
        ("cooldown", spec.cooldown.is_some()),
    ];
    let effect = effect(spec.effect, content)?;
    let active = match spec.kind.as_str() {
        "triggered" => {
            if let Some((name, _)) = costs.iter().find(|(_, set)| *set) {
                return Err(format!(
                    "a triggered ability is free and instant: it has no `{name}`"
                ));
            }
            if effect.is_active() {
                return Err(format!(
                    "a `{}` effect is an active's: `kind = \"active\"`",
                    effect.kind()
                ));
            }
            None
        }
        "active" => {
            if spec.trigger.is_some() {
                return Err("an active ability has no `trigger`: it is used".into());
            }
            let cooldown = spec.cooldown.as_deref();
            Some(active(
                (spec.aim.as_deref(), spec.stamina, spec.energy, cooldown),
                &effect,
                content,
            )?)
        }
        other => return Err(format!("unknown kind `{other}` (triggered, active)")),
    };
    let triggers = match spec.trigger {
        None => Vec::new(),
        Some(OneOrMany::One(t)) => vec![Trigger::parse(&t)?],
        Some(OneOrMany::Many(ts)) => {
            (ts.iter().map(|t| Trigger::parse(t))).collect::<Result<_, _>>()?
        }
    };
    match (effect.is_switch(), triggers.is_empty()) {
        (true, false) => {
            return Err(format!(
                "a `{}` effect is a switch: no `trigger`",
                effect.kind()
            ));
        }
        (false, true) if active.is_none() => {
            return Err("a triggered ability needs a `trigger`".into());
        }
        _ => {}
    }
    if effect == WeaponDoes::FreeShot && triggers != [Trigger::Approaches] {
        return Err("a `free_shot` fires on `approaches`".into());
    }
    if matches!(effect, WeaponDoes::ReflectSpell { .. }) && triggers != [Trigger::SpellBlocked] {
        return Err("a `reflect_spell` fires on `spell_blocked`".into());
    }
    // `use:ID` names an active: never a spell's, a class active's, or a birth sign gift's id,
    // which the action text reads first.
    if active.is_some()
        && (content.spell_id(&spec.id).is_some()
            || super::ClassActive::from_id(&spec.id).is_some()
            || super::SignActive::from_id(&spec.id).is_some())
    {
        return Err("an active's id is no spell's, class active's, or birth sign gift's".into());
    }
    if spec.systems.iter().any(|s| s.trim().is_empty()) {
        return Err("`systems` names systems, not blanks".into());
    }
    Ok(WeaponAbility {
        id: spec.id,
        name: spec.name,
        description: spec.description,
        skills,
        rank: spec.rank,
        triggers,
        systems: spec.systems,
        effect,
        active,
    })
}

/// Resolves an active's aim, stamina, energy, and cooldown, for `effect`.
fn active(
    (aim, stamina, energy, cooldown): (Option<&str>, Option<u16>, Option<EnergySpec>, Option<&str>),
    effect: &WeaponDoes,
    content: &Content,
) -> Result<Active, String> {
    if !effect.is_active() {
        return Err(format!(
            "a `{}` effect is a triggered ability's: `kind = \"triggered\"`",
            effect.kind()
        ));
    }
    let aim = match aim {
        Some("nothing") => ActiveAim::Nothing,
        Some("direction") => ActiveAim::Direction,
        Some("cell") => ActiveAim::Cell,
        Some(other) => {
            return Err(format!(
                "unknown `aim` `{other}` (nothing, direction, cell)"
            ));
        }
        None => return Err("an active needs an `aim` (nothing, direction, cell)".into()),
    };
    let stamina = stamina.ok_or("an active needs `stamina`")?;
    if !(1..=50).contains(&stamina) {
        return Err("`stamina` is 1 to 50".into());
    }
    let energy = energy.ok_or("an active needs `energy`: no active is free of time")?;
    let action = (ActionKind::ALL.iter().copied())
        .find(|k| k.id() == energy.action)
        .ok_or_else(|| format!("unknown action `{}` in `energy`", energy.action))?;
    if !ENERGY_PERCENT.contains(&energy.percent) {
        return Err(format!(
            "`energy`'s `percent` is {} to {}",
            ENERGY_PERCENT.start(),
            ENERGY_PERCENT.end()
        ));
    }
    // What each effect is: a blow aimed in a direction, a sweep or a buff aimed at nothing, a
    // shot aimed at a cell.
    let (aims, actions): (&[ActiveAim], &[ActionKind]) = match effect {
        WeaponDoes::ReachBlow { .. }
        | WeaponDoes::SureCritical
        | WeaponDoes::IgnoreDefence { .. }
        | WeaponDoes::Knockback { .. }
        | WeaponDoes::DoubleDice { .. } => (&[ActiveAim::Direction], &[ActionKind::Melee]),
        WeaponDoes::IgnoreProtection => (
            &[ActiveAim::Direction, ActiveAim::Cell],
            &[ActionKind::Melee, ActionKind::Shoot],
        ),
        WeaponDoes::Sweep { .. } => (&[ActiveAim::Nothing], &[ActionKind::Melee]),
        WeaponDoes::PierceShot => (&[ActiveAim::Cell], &[ActionKind::Shoot]),
        WeaponDoes::Buff { .. } => (&[ActiveAim::Nothing], &[ActionKind::Wait]),
        _ => (&[], &[]),
    };
    if !aims.contains(&aim) || !actions.contains(&action) {
        let aims: Vec<String> = aims
            .iter()
            .map(|a| format!("{a:?}").to_lowercase())
            .collect();
        let actions: Vec<&str> = actions.iter().map(|a| a.id()).collect();
        return Err(format!(
            "a `{}` effect aims at {} and is priced as {}",
            effect.kind(),
            aims.join(" or "),
            actions.join(" or ")
        ));
    }
    // A shot aims at a cell, a blow in a direction.
    if (action == ActionKind::Shoot) != (aim == ActiveAim::Cell) {
        return Err("a shot aims at a `cell`, and only a shot does".into());
    }
    let cooldown = match cooldown {
        None => None,
        Some(name) => {
            let id = (content.status_id(name)).ok_or(format!("unknown status `{name}`"))?;
            let def = content.status(id);
            if def.eases.is_none() || def.tick.is_some() {
                return Err(format!(
                    "a `cooldown` is a status that eases and does not tick: `{name}` is not"
                ));
            }
            Some(id)
        }
    };
    Ok(Active {
        aim,
        stamina,
        action,
        percent: energy.percent,
        cooldown,
    })
}

/// Resolves an effect: its kind, with only its own fields.
fn effect(spec: EffectSpec, content: &Content) -> Result<WeaponDoes, String> {
    let fields = [
        ("reach", spec.reach.is_some()),
        ("categories", spec.categories.is_some()),
        ("to_hit", spec.to_hit.is_some()),
        ("cells", spec.cells.is_some()),
        ("bash", spec.bash.is_some()),
        ("trigger", spec.trigger.is_some()),
        ("status", spec.status.is_some()),
        ("severity", spec.severity.is_some()),
        ("turns", spec.turns.is_some()),
        ("on", spec.on.is_some()),
        ("dice", spec.dice.is_some()),
        ("damage", spec.damage_kind.is_some()),
        ("per_turn", spec.per_turn.is_some()),
    ];
    let reads: &[&str] = match spec.kind.as_str() {
        "free_blow" => &["trigger"],
        "status" => &["status", "severity", "turns", "on"],
        "extra_damage" => &["dice", "damage"],
        "reflect_spell" => &["per_turn"],
        "free_shot" | "parry_missiles" | "sure_critical" | "ignore_protection" | "pierce_shot" => {
            &[]
        }
        "reach_blow" => &["reach"],
        "ignore_defence" => &["categories"],
        "sweep" | "double_dice" => &["to_hit"],
        "knockback" => &["cells", "bash", "status"],
        "buff" => &["status", "severity"],
        other => {
            return Err(format!(
                "unknown effect kind `{other}` (free_blow, status, extra_damage, free_shot, \
                 parry_missiles, reflect_spell, reach_blow, sure_critical, ignore_defence, \
                 ignore_protection, sweep, pierce_shot, knockback, buff, double_dice)"
            ));
        }
    };
    if let Some((name, _)) = fields.iter().find(|(n, set)| *set && !reads.contains(n)) {
        return Err(format!("a `{}` effect has no `{name}`", spec.kind));
    }
    Ok(match spec.kind.as_str() {
        "free_blow" => {
            let name = spec
                .trigger
                .ok_or("a `free_blow` effect needs a `trigger`")?;
            let kind = (FreeBlowKind::ALL.into_iter())
                .find(|k| k.id() == name)
                .ok_or_else(|| {
                    format!(
                        "unknown free blow `{name}` (riposte, cleave, great_cleave, \
                         opportunist, gatekeeper)"
                    )
                })?;
            WeaponDoes::FreeBlow(kind)
        }
        "status" => {
            let name = spec.status.ok_or("a `status` effect needs a `status`")?;
            let status = (content.status_id(&name)).ok_or(format!("unknown status `{name}`"))?;
            let max = content.status(status).max;
            if spec.severity.is_some() && spec.turns.is_some() {
                return Err("a `status` effect has a `severity` or `turns`, not both".into());
            }
            let severity = spec.severity.unwrap_or(1);
            if severity < 1 || severity > max {
                return Err(format!("`severity` is 1 to the status's max, {max}"));
            }
            if let Some(turns) = spec.turns
                && (turns.min() < 1 || turns.max() > i32::from(max))
            {
                return Err(format!("`turns` roll 1 to the status's max, {max}"));
            }
            let on = match spec.on.as_deref() {
                None | Some("target") => On::Target,
                Some("self") => On::Own,
                Some(other) => return Err(format!("unknown `on` `{other}` (target, self)")),
            };
            WeaponDoes::Status {
                status,
                severity,
                turns: spec.turns,
                on,
            }
        }
        "extra_damage" => {
            let dice = match spec.dice.as_deref() {
                Some("main_hand") => ExtraDice::MainHand,
                Some("off_hand") => ExtraDice::OffHand,
                Some(dice) => ExtraDice::Fixed(dice.parse().map_err(|e| {
                    format!("`dice` is main_hand, off_hand, or dice (\"1d6\"): {e}")
                })?),
                None => return Err("an `extra_damage` effect needs `dice`".into()),
            };
            let kind = match spec.damage_kind {
                None => None,
                Some(name) => Some(
                    content
                        .damage_type_id(&name)
                        .ok_or(format!("unknown damage type `{name}`"))?,
                ),
            };
            WeaponDoes::ExtraDamage { dice, kind }
        }
        "free_shot" => WeaponDoes::FreeShot,
        "reflect_spell" => match spec.per_turn {
            Some(per_turn @ 1..) => WeaponDoes::ReflectSpell { per_turn },
            _ => return Err("a `reflect_spell` effect needs `per_turn`, 1 or more".into()),
        },
        "parry_missiles" => WeaponDoes::ParryMissiles,
        "sure_critical" => WeaponDoes::SureCritical,
        "ignore_protection" => WeaponDoes::IgnoreProtection,
        "pierce_shot" => WeaponDoes::PierceShot,
        "reach_blow" => {
            let reach = spec.reach.ok_or("a `reach_blow` effect needs a `reach`")?;
            if !(2..=4).contains(&reach) {
                return Err("`reach` is 2 to 4".into());
            }
            WeaponDoes::ReachBlow { reach }
        }
        "ignore_defence" => {
            let names = (spec.categories)
                .filter(|c| !c.is_empty())
                .ok_or("an `ignore_defence` effect names its `categories`")?;
            let mut categories = Categories::default();
            for name in &names {
                let category = (Category::ALL.into_iter())
                    .find(|c| c.id() == name)
                    .ok_or_else(|| format!("unknown Defence category `{name}`"))?;
                categories = categories.with(category);
            }
            WeaponDoes::IgnoreDefence { categories }
        }
        "sweep" | "double_dice" => {
            let to_hit = spec.to_hit.unwrap_or(0);
            if !(-20..=20).contains(&to_hit) {
                return Err("`to_hit` is -20 to 20".into());
            }
            if spec.kind == "sweep" {
                WeaponDoes::Sweep { to_hit }
            } else {
                WeaponDoes::DoubleDice { to_hit }
            }
        }
        "knockback" => {
            let cells = spec.cells.unwrap_or(1);
            if cells != 1 {
                return Err("`cells` is 1".into());
            }
            let status = match spec.status {
                None => None,
                Some(name) => {
                    Some((content.status_id(&name)).ok_or(format!("unknown status `{name}`"))?)
                }
            };
            WeaponDoes::Knockback {
                cells,
                bash: spec.bash.unwrap_or(false),
                status,
            }
        }
        _ => {
            let name = spec.status.ok_or("a `buff` effect needs a `status`")?;
            let status = (content.status_id(&name)).ok_or(format!("unknown status `{name}`"))?;
            let max = content.status(status).max;
            let severity = spec.severity.unwrap_or(1);
            if severity < 1 || severity > max {
                return Err(format!("`severity` is 1 to the status's max, {max}"));
            }
            WeaponDoes::Buff { status, severity }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentFiles;

    fn load_with(file: &str) -> Result<Content, ContentError> {
        let mut files = ContentFiles::builtin();
        files.insert("data/weapon_abilities/zz_test.toml", file);
        Content::load(&files)
    }

    #[test]
    fn the_shipped_abilities_load() {
        let content = Content::load(&ContentFiles::builtin()).expect("content");
        let rend = (content.weapon_abilities().iter())
            .find(|a| a.id == "rend")
            .expect("rend");
        let twin = content.weapon_skills().id("twin_blades").expect("twin");
        assert_eq!(rend.skills, [twin]);
        assert_eq!(
            (rend.rank, rend.triggers.as_slice()),
            (4, &[Trigger::BothHandsHit][..])
        );
        assert_eq!(
            rend.effect,
            WeaponDoes::ExtraDamage {
                dice: ExtraDice::MainHand,
                kind: None
            }
        );
        let block = (content.weapon_abilities().iter())
            .find(|a| a.id == "staggering_block")
            .expect("staggering block");
        assert_eq!(block.triggers, [Trigger::Parried, Trigger::Blocked]);
        let reflection = (content.weapon_abilities().iter())
            .find(|a| a.id == "spell_reflection")
            .expect("spell reflection");
        let board = content
            .weapon_skills()
            .id("sword_and_board")
            .expect("style");
        assert_eq!(
            (reflection.skills.as_slice(), reflection.rank),
            (&[board][..], 10)
        );
        assert_eq!(reflection.triggers, [Trigger::SpellBlocked]);
        assert_eq!(reflection.effect, WeaponDoes::ReflectSpell { per_turn: 1 });
        // The sneak attack's two (the surprise design §6.4).
        let daggers = content.weapon_skills().id("daggers").expect("daggers");
        let marksman = content.weapon_skills().id("marksman").expect("marksman");
        let status = |id| content.status_id(id).expect("status");
        let knife = (content.weapon_abilities().iter())
            .find(|a| a.id == "knife_in_the_dark")
            .expect("knife in the dark");
        assert_eq!(
            (
                knife.skills.as_slice(),
                knife.rank,
                knife.triggers.as_slice()
            ),
            (&[daggers][..], 5, &[Trigger::SneakAttack][..])
        );
        assert_eq!(
            knife.effect,
            WeaponDoes::Status {
                status: status("bleeding"),
                severity: 1,
                turns: None,
                on: On::Target,
            }
        );
        let hunter = (content.weapon_abilities().iter())
            .find(|a| a.id == "hunters_shot")
            .expect("hunter's shot");
        assert_eq!(
            (
                hunter.skills.as_slice(),
                hunter.rank,
                hunter.triggers.as_slice()
            ),
            (&[marksman][..], 6, &[Trigger::SneakAttack][..])
        );
        assert!(matches!(
            hunter.effect,
            WeaponDoes::Status { status: s, turns: Some(_), on: On::Target, .. } if s == status("pinned")
        ));
        assert_eq!(Trigger::SneakAttack.id(), "sneak_attack");
        // Every trigger has an id the parser reads back.
        for t in Trigger::ALL {
            assert_eq!(Trigger::parse(t.id()), Ok(t));
        }
        // Every system named is one of the talent table's.
        let book = content.talents();
        for a in content.weapon_abilities() {
            for s in &a.systems {
                assert!(book.system_id(s).is_some(), "{}: system `{s}`", a.id);
            }
        }
        // The actives: Lunge as the issue writes it, and the catalogue's twelve.
        let lunge = (content.weapon_abilities().iter())
            .find(|a| a.id == "lunge")
            .expect("lunge");
        assert_eq!(
            lunge.active,
            Some(Active {
                aim: ActiveAim::Direction,
                stamina: 3,
                action: ActionKind::Melee,
                percent: 100,
                cooldown: None,
            })
        );
        assert_eq!(lunge.effect, WeaponDoes::ReachBlow { reach: 2 });
        let actives = (content.weapon_abilities().iter()).filter(|a| a.active.is_some());
        assert_eq!(actives.count(), 12);
        // No ability is called an aimed shot (the owner's rule): every shot is aimed.
        assert!(
            (content.weapon_abilities().iter()).all(|a| !a.name.to_lowercase().contains("aimed"))
        );
    }

    #[test]
    fn malformed_abilities_are_refused() {
        let base = "[[weapon_ability]]\nid = \"x\"\nname = \"X\"\ndescription = \"x\"\n";
        let cases = [
            (
                "skills = [\"nunchaku\"]\nrank = 4\nkind = \"triggered\"\ntrigger = \"hit\"\n\
                 [weapon_ability.effect]\nkind = \"parry_missiles\"",
                "unknown weapon skill",
            ),
            (
                "skills = [\"duelist\", \"long_blades\"]\nrank = 4\nkind = \"triggered\"\n\
                 [weapon_ability.effect]\nkind = \"parry_missiles\"",
                "not both",
            ),
            (
                "skills = [\"duelist\"]\nrank = 11\nkind = \"triggered\"\n\
                 [weapon_ability.effect]\nkind = \"parry_missiles\"",
                "`rank` is 1 to 10",
            ),
            (
                "skills = [\"spears\"]\nrank = 9\nkind = \"active\"\ntrigger = \"hit\"\n\
                 aim = \"direction\"\n[weapon_ability.effect]\nkind = \"parry_missiles\"",
                "an active ability has no `trigger`",
            ),
            (
                "skills = [\"spears\"]\nrank = 9\nkind = \"active\"\naim = \"direction\"\n\
                 stamina = 3\nenergy = { action = \"melee\", percent = 100 }\n\
                 [weapon_ability.effect]\nkind = \"parry_missiles\"",
                "a triggered ability's",
            ),
            (
                "skills = [\"spears\"]\nrank = 9\nkind = \"active\"\naim = \"direction\"\n\
                 stamina = 3\n[weapon_ability.effect]\nkind = \"reach_blow\"\nreach = 2",
                "an active needs `energy`",
            ),
            (
                "skills = [\"spears\"]\nrank = 9\nkind = \"active\"\naim = \"direction\"\n\
                 stamina = 3\nenergy = { action = \"melee\", percent = 40 }\n\
                 [weapon_ability.effect]\nkind = \"reach_blow\"\nreach = 2",
                "`percent` is 50 to 300",
            ),
            (
                "skills = [\"spears\"]\nrank = 9\nkind = \"active\"\naim = \"direction\"\n\
                 stamina = 3\nenergy = { action = \"melee\", percent = 301 }\n\
                 [weapon_ability.effect]\nkind = \"reach_blow\"\nreach = 2",
                "`percent` is 50 to 300",
            ),
            (
                "skills = [\"spears\"]\nrank = 9\nkind = \"active\"\naim = \"direction\"\n\
                 stamina = 3\nenergy = { action = \"melee\", percent = 100 }\n\
                 cooldown = \"bleeding\"\n[weapon_ability.effect]\nkind = \"reach_blow\"\n\
                 reach = 2",
                "a `cooldown` is a status that eases and does not tick",
            ),
            (
                "skills = [\"spears\"]\nrank = 9\nkind = \"active\"\naim = \"cell\"\n\
                 stamina = 3\nenergy = { action = \"melee\", percent = 100 }\n\
                 [weapon_ability.effect]\nkind = \"reach_blow\"\nreach = 2",
                "aims at direction",
            ),
            (
                "skills = [\"spears\"]\nrank = 9\nkind = \"triggered\"\ntrigger = \"hit\"\n\
                 [weapon_ability.effect]\nkind = \"sure_critical\"",
                "an active's",
            ),
            (
                "skills = [\"spears\"]\nrank = 9\nkind = \"active\"\naim = \"nothing\"\n\
                 stamina = 3\nenergy = { action = \"wait\", percent = 100 }\n\
                 [weapon_ability.effect]\nkind = \"buff\"\nstatus = \"blessed\"\nreach = 2",
                "a `buff` effect has no `reach`",
            ),
            (
                "skills = [\"duelist\"]\nrank = 4\nkind = \"triggered\"\ntrigger = \"hit\"\n\
                 stamina = 3\n[weapon_ability.effect]\nkind = \"status\"\nstatus = \"staggered\"",
                "has no `stamina`",
            ),
            (
                "skills = [\"duelist\"]\nrank = 4\nkind = \"triggered\"\ntrigger = \"hit\"\n\
                 energy = { action = \"melee\", percent = 100 }\n[weapon_ability.effect]\n\
                 kind = \"status\"\nstatus = \"staggered\"",
                "has no `energy`",
            ),
            (
                "skills = [\"duelist\"]\nrank = 4\nkind = \"triggered\"\ntrigger = \"hit\"\n\
                 [weapon_ability.effect]\nkind = \"status\"\nstatus = \"staggered\"\n\
                 dice = \"1d6\"",
                "a `status` effect has no `dice`",
            ),
            (
                "skills = [\"duelist\"]\nrank = 4\nkind = \"triggered\"\ntrigger = \"hit\"\n\
                 [weapon_ability.effect]\nkind = \"free_blow\"\ntrigger = \"riposte\"",
                "is a switch",
            ),
            (
                "skills = [\"duelist\"]\nrank = 4\nkind = \"triggered\"\n\
                 [weapon_ability.effect]\nkind = \"status\"\nstatus = \"staggered\"",
                "needs a `trigger`",
            ),
            (
                "skills = [\"duelist\"]\nrank = 4\nkind = \"triggered\"\ntrigger = \"wink\"\n\
                 [weapon_ability.effect]\nkind = \"status\"\nstatus = \"staggered\"",
                "unknown trigger",
            ),
            (
                "skills = [\"duelist\"]\nrank = 4\nkind = \"triggered\"\ntrigger = \"hit\"\n\
                 [weapon_ability.effect]\nkind = \"status\"\nstatus = \"bleeding\"\n\
                 severity = 9",
                "`severity` is 1",
            ),
            (
                "skills = [\"bows\"]\nrank = 4\nkind = \"triggered\"\ntrigger = \"hit\"\n\
                 [weapon_ability.effect]\nkind = \"free_shot\"",
                "fires on `approaches`",
            ),
            (
                "skills = [\"sword_and_board\"]\nrank = 10\nkind = \"triggered\"\n\
                 trigger = \"blocked\"\n[weapon_ability.effect]\nkind = \"reflect_spell\"\n\
                 per_turn = 1",
                "fires on `spell_blocked`",
            ),
            (
                "skills = [\"sword_and_board\"]\nrank = 10\nkind = \"triggered\"\n\
                 trigger = \"spell_blocked\"\n[weapon_ability.effect]\nkind = \"reflect_spell\"\n\
                 per_turn = 0",
                "needs `per_turn`",
            ),
            (
                "skills = [\"sword_and_board\"]\nrank = 10\nkind = \"triggered\"\n\
                 trigger = \"spell_blocked\"\n[weapon_ability.effect]\nkind = \"status\"\n\
                 status = \"staggered\"\nper_turn = 1",
                "a `status` effect has no `per_turn`",
            ),
            (
                "skills = [\"duelist\"]\nrank = 4\nkind = \"triggered\"\ntrigger = \"hit\"\n\
                 systems = [\" \"]\n[weapon_ability.effect]\nkind = \"status\"\n\
                 status = \"staggered\"",
                "not blanks",
            ),
        ];
        for (body, want) in cases {
            let err = load_with(&format!("{base}{body}\n"))
                .expect_err(body)
                .to_string();
            assert!(err.contains(want), "{body}: {err}");
        }
        // `use:ID` reads a spell first: an active never takes a spell's id.
        let err = load_with(
            "[[weapon_ability]]\nid = \"blessing\"\nname = \"X\"\ndescription = \"x\"\n\
             skills = [\"spears\"]\nrank = 9\nkind = \"active\"\naim = \"nothing\"\n\
             stamina = 3\nenergy = { action = \"wait\", percent = 100 }\n\
             [weapon_ability.effect]\nkind = \"buff\"\nstatus = \"blessed\"\n",
        )
        .expect_err("a spell's id")
        .to_string();
        assert!(err.contains("no spell's"), "{err}");
    }
}

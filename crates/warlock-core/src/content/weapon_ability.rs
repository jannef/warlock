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
//! already is. **Active** abilities (used from the abilities menu, paid in stamina and time)
//! come with W3: until then `kind = "active"` is refused.

use super::{Content, ContentError, StatusId, WeaponSkillId, duplicate_id};
use crate::dice::Dice;
use serde::Deserialize;
use std::collections::BTreeMap;

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
    /// The actor is hit.
    Struck,
    /// A melee attack on the actor missed by its Parry.
    Parried,
    /// A melee attack on the actor missed by its Shield.
    Blocked,
    /// A melee attack on the actor missed by its Dexterity or Dodge.
    Dodged,
    /// Both blows of the actor's two-weapon attack hit the same target, which lives.
    BothHandsHit,
    /// A hostile steps up beside the actor.
    Approaches,
}

impl Trigger {
    /// Every trigger, in the wiki's order.
    pub const ALL: [Trigger; 10] = [
        Trigger::Hit,
        Trigger::Critical,
        Trigger::BigHit,
        Trigger::Kill,
        Trigger::Struck,
        Trigger::Parried,
        Trigger::Blocked,
        Trigger::Dodged,
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
            Trigger::Struck => "struck",
            Trigger::Parried => "parried",
            Trigger::Blocked => "blocked",
            Trigger::Dodged => "dodged",
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
            Trigger::Struck => "you are hit",
            Trigger::Parried => "a melee attack on you misses by your Parry",
            Trigger::Blocked => "a melee attack on you misses by your Shield",
            Trigger::Dodged => "a melee attack on you misses by your Dexterity or Dodge",
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
        }
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
    // An active's (W3): refused on a triggered ability.
    aim: Option<String>,
    stamina: Option<u16>,
    energy: Option<toml::Value>,
    cooldown: Option<String>,
    #[serde(default)]
    systems: Vec<String>,
    effect: EffectSpec,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectSpec {
    kind: String,
    trigger: Option<String>,
    status: Option<String>,
    severity: Option<u16>,
    turns: Option<Dice>,
    on: Option<String>,
    dice: Option<String>,
    #[serde(rename = "damage")]
    damage_kind: Option<String>,
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
    Ok(all)
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
    match spec.kind.as_str() {
        "triggered" => {
            if let Some((name, _)) = costs.iter().find(|(_, set)| *set) {
                return Err(format!(
                    "a triggered ability is free and instant: it has no `{name}`"
                ));
            }
        }
        "active" => {
            if spec.trigger.is_some() {
                return Err("an active ability has no `trigger`: it is used".into());
            }
            return Err("`kind = \"active\"` is not in the game yet (W3)".into());
        }
        other => return Err(format!("unknown kind `{other}` (triggered, active)")),
    }
    let effect = effect(spec.effect, content)?;
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
        (false, true) => return Err("a triggered ability needs a `trigger`".into()),
        _ => {}
    }
    if effect == WeaponDoes::FreeShot && triggers != [Trigger::Approaches] {
        return Err("a `free_shot` fires on `approaches`".into());
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
    })
}

/// Resolves an effect: its kind, with only its own fields.
fn effect(spec: EffectSpec, content: &Content) -> Result<WeaponDoes, String> {
    let fields = [
        ("trigger", spec.trigger.is_some()),
        ("status", spec.status.is_some()),
        ("severity", spec.severity.is_some()),
        ("turns", spec.turns.is_some()),
        ("on", spec.on.is_some()),
        ("dice", spec.dice.is_some()),
        ("damage", spec.damage_kind.is_some()),
    ];
    let reads: &[&str] = match spec.kind.as_str() {
        "free_blow" => &["trigger"],
        "status" => &["status", "severity", "turns", "on"],
        "extra_damage" => &["dice", "damage"],
        "free_shot" | "parry_missiles" => &[],
        other => {
            return Err(format!(
                "unknown effect kind `{other}` (free_blow, status, extra_damage, free_shot, \
                 parry_missiles)"
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
        _ => WeaponDoes::ParryMissiles,
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
                 [weapon_ability.effect]\nkind = \"parry_missiles\"",
                "not in the game yet",
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
    }
}

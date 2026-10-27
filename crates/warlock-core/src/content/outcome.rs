//! Outcome tables (`data/outcomes/<id>.toml`, one table a file; spec §7 "Special rooms"): what
//! using a place's ability draws: a forgotten altar's prayer, a healing spring's drink. The rules
//! are `world/outcome.rs` (applying an outcome), `world/pray.rs` (the prayer), and
//! `world/drink.rs` (the drink).
//!
//! ```toml
//! # data/outcomes/forgotten_altar.toml
//! seconds = 60                                  # how long the use takes (turns of `pray`)
//! kinds = { boon = 35, nothing = 30, bane = 35 } # the kinds' weights, before the lean
//! lean = 15                                     # moved between boon and bane by the side
//! exhaust = 50                                  # percent: spent after a completed use
//! spent = "spent_altar"                         # what the feature becomes, spent
//!
//! [[outcome]]
//! kind = "boon"            # boon, nothing, or bane
//! tier = "minor"           # minor, moderate, or great
//! id = "soothed"
//! weight = 13              # its share of its kind's draw
//! effect = { heal = "2d6", cure = ["bleeding"] }
//! message = "A little warmth runs through you."   # or `messages`, one drawn
//! otherwise = "..."        # said instead when the effect found nothing to do (optional)
//! ```
//!
//! An `effect` is a set of hooks, each one field (none: a "nothing happens" line); the rules
//! apply them in the order [`OutcomeEffect`] lists them.

use super::spell::{Burst, SpellFields, check_effect, one_kind};
use super::trap::TrapNoise;
use super::{
    Content, ContentError, DamageTypeId, DiseaseId, EffectShape, ObjectId, StatusId, TerrainId,
};
use crate::dice::Dice;
use crate::magic::loot::LootGrade;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The heaviest an outcome's or a kind's weight may be.
pub const MAX_WEIGHT: u32 = 1000;
/// The longest a use may take, in seconds.
pub const MAX_SECONDS: u32 = 3600;
/// The seconds of one turn of a use (the `pray` action's time at normal speed).
pub const TURN_SECONDS: u32 = 10;
/// The farthest the `know_traps` hook reaches, in cells.
pub const MAX_REACH: i32 = 30;
/// The most experience levels an outcome gives or takes.
pub const MAX_LEVELS: i8 = 5;

/// What kind of outcome it is: the kinds' weights are drawn first, leaned by the side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeKind {
    Boon,
    Nothing,
    Bane,
}

impl OutcomeKind {
    /// Every kind, the best first.
    pub const ALL: [OutcomeKind; 3] = [OutcomeKind::Boon, OutcomeKind::Nothing, OutcomeKind::Bane];

    /// Its id in data and the wiki: `boon`.
    pub const fn id(self) -> &'static str {
        match self {
            OutcomeKind::Boon => "boon",
            OutcomeKind::Nothing => "nothing",
            OutcomeKind::Bane => "bane",
        }
    }
}

/// How much an outcome matters: most are minor, a few moderate, the great ones rare.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeTier {
    Minor,
    Moderate,
    Great,
}

impl OutcomeTier {
    /// Every tier, the least first.
    pub const ALL: [OutcomeTier; 3] = [
        OutcomeTier::Minor,
        OutcomeTier::Moderate,
        OutcomeTier::Great,
    ];

    /// Its id in data and the wiki: `minor`.
    pub const fn id(self) -> &'static str {
        match self {
            OutcomeTier::Minor => "minor",
            OutcomeTier::Moderate => "moderate",
            OutcomeTier::Great => "great",
        }
    }
}

/// How much a heal or a mana gift gives: dice, half the maximum, or everything up to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Amount {
    Dice(Dice),
    /// Half the maximum (rounded down), never past it.
    Half,
    Full,
}

/// Harm an outcome does: `damage` of `kind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Harmful {
    pub damage: Dice,
    pub kind: Option<DamageTypeId>,
}

/// Where summoned creatures come.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Near {
    /// At the room's edge, out of the player's view if a cell there is.
    Edge,
    /// Beside the feature (the altar).
    Feature,
}

/// Hostile creatures an outcome brings: `count` of them, of the floor's danger plus `danger`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutcomeSummon {
    pub danger: u8,
    pub near: Near,
    pub count: Dice,
}

/// The feature blows apart: a burst on its cell, with its noise.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Explosion {
    pub burst: Burst,
    pub noise: Option<TrapNoise>,
}

/// What an outcome does: each field one hook (`world/outcome.rs`), applied in this order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OutcomeEffect {
    /// Hit points back.
    pub heal: Option<Amount>,
    /// These statuses end.
    pub cure: Vec<StatusId>,
    /// Everything this means cures ends (statuses and diseases: `prayer`).
    pub cure_by: Option<String>,
    /// All stat damage mends (the temple's mending).
    pub mend: bool,
    /// Mana back.
    pub mana: Option<Amount>,
    /// Mana lost (never below 0).
    pub drain: Option<Dice>,
    /// A status, at this many degrees.
    pub status: Option<(StatusId, u16)>,
    /// Nutrition this many states better (or worse, negative), never past Satiated nor into
    /// Starving.
    pub hunger: i8,
    /// One of these is left at the feature's foot: an object, how many, and its weight.
    pub objects: Vec<(ObjectId, Dice, u32)>,
    /// An item of loot at the floor's item level plus this, at the feature's foot.
    pub loot: Option<u8>,
    /// Every trap within this many cells becomes known.
    pub know_traps: Option<i32>,
    /// One unidentified item carried is identified.
    pub identify: bool,
    /// One identified magic item carried is unidentified again.
    pub forget: bool,
    /// Damage that never takes the last hit point (resistance counts).
    pub burn: Option<Harmful>,
    /// Damage from above that may kill (resistance counts).
    pub smite: Option<Harmful>,
    /// Percent of the coins carried that vanish.
    pub lose_coins: u8,
    /// The player's lantern goes out.
    pub snuff: bool,
    /// Hostile creatures come.
    pub summon: Option<OutcomeSummon>,
    /// A random disease is caught.
    pub disease: bool,
    /// This disease is caught.
    pub catch: Option<DiseaseId>,
    /// Stat damage to a random stat.
    pub wither: Option<Dice>,
    /// Percent of the experience to the next level.
    pub xp: u8,
    /// The most valuable stack carried vanishes.
    pub tithe: bool,
    /// The player is moved to a free cell of the level.
    pub teleport: bool,
    /// The feature blows apart.
    pub explode: Option<Explosion>,
    /// What the feature becomes.
    pub becomes: Option<TerrainId>,
    /// The least rarity `loot` makes (`rarity = "rare"`: the top a base allows; a relic).
    pub rarity: LootGrade,
    /// A random stat's score and potential this much higher, for good (Gift of the forgotten).
    pub gift: Option<Dice>,
    /// A random stat's potential and score this much lower, for good, never below
    /// [`crate::stats::DRAIN_FLOOR`] (Wrath of the forgotten).
    pub wrath: Option<Dice>,
    /// Experience levels gained (positive: to the next level's threshold, Exalted) or lost
    /// (negative: to the start of the level below, `World::lose_level`, Unmade).
    pub levels: i8,
    /// A talent the player may take, given free and kept for life (Chosen; `Actor::boons`).
    pub talent: bool,
    /// A flaw talent the player lacks, kept for life (Forsaken; `Actor::boons`).
    pub flaw: bool,
}

/// One outcome of a table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutcomeDef {
    pub id: String,
    pub kind: OutcomeKind,
    pub tier: OutcomeTier,
    /// Its share of its kind's draw.
    pub weight: u32,
    /// Said as it happens: one of them, drawn.
    pub messages: Vec<String>,
    /// Said instead when its effect found nothing to do (no unidentified item to make clear).
    pub otherwise: Option<String>,
    pub effect: OutcomeEffect,
}

/// An outcome table (`data/outcomes/<id>.toml`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutcomeTable {
    pub id: String,
    /// How long a use takes, in seconds.
    pub seconds: u32,
    /// The kinds' weights before the lean, by [`OutcomeKind::ALL`].
    pub kinds: [u32; 3],
    /// Points moved between the best kind and the worst by the side's agreement.
    pub lean: u32,
    /// Percent: the place is spent after a completed use.
    pub exhaust: u8,
    /// What the feature becomes, spent.
    pub spent: Option<TerrainId>,
    pub outcomes: Vec<OutcomeDef>,
}

impl OutcomeTable {
    /// How many turns a use takes.
    pub fn turns(&self) -> u32 {
        self.seconds.div_ceil(TURN_SECONDS).max(1)
    }

    /// The weight of `kind` before the lean.
    pub fn weight(&self, kind: OutcomeKind) -> u32 {
        self.kinds[kind as usize]
    }

    /// The kinds' weights with the best `lean` points moved toward the boon (negative: toward
    /// the bane), each held to what it has.
    pub fn leaned(&self, lean: i32) -> [u32; 3] {
        let [boon, nothing, bane] = self.kinds;
        let shift = if lean >= 0 {
            lean.unsigned_abs().min(bane)
        } else {
            lean.unsigned_abs().min(boon)
        };
        if lean >= 0 {
            [boon + shift, nothing, bane - shift]
        } else {
            [boon - shift, nothing, bane + shift]
        }
    }

    /// The outcomes of `kind`, in file order.
    pub fn of(&self, kind: OutcomeKind) -> impl Iterator<Item = &OutcomeDef> {
        self.outcomes.iter().filter(move |o| o.kind == kind)
    }

    /// The sum of the weights of `kind`'s outcomes.
    pub fn total(&self, kind: OutcomeKind) -> u32 {
        self.of(kind).map(|o| o.weight).sum()
    }
}

fn fifty() -> u8 {
    50
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TableSpec {
    seconds: u32,
    kinds: KindsSpec,
    #[serde(default)]
    lean: u32,
    #[serde(default = "fifty")]
    exhaust: u8,
    spent: Option<String>,
    #[serde(default)]
    outcome: Vec<OutcomeSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KindsSpec {
    #[serde(default)]
    boon: u32,
    #[serde(default)]
    nothing: u32,
    #[serde(default)]
    bane: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OutcomeSpec {
    kind: OutcomeKind,
    tier: OutcomeTier,
    id: String,
    weight: u32,
    #[serde(default)]
    effect: EffectSpec,
    message: Option<String>,
    #[serde(default)]
    messages: Vec<String>,
    otherwise: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct EffectSpec {
    heal: Option<String>,
    #[serde(default)]
    cure: Vec<String>,
    cure_by: Option<String>,
    #[serde(default)]
    mend: bool,
    mana: Option<String>,
    drain: Option<Dice>,
    status: Option<StatusSpec>,
    #[serde(default)]
    hunger: i8,
    #[serde(default)]
    objects: Vec<ObjectSpec>,
    loot: Option<u8>,
    know_traps: Option<i32>,
    #[serde(default)]
    identify: bool,
    #[serde(default)]
    forget: bool,
    burn: Option<HarmSpec>,
    smite: Option<HarmSpec>,
    #[serde(default)]
    lose_coins: u8,
    #[serde(default)]
    snuff: bool,
    summon: Option<SummonSpec>,
    #[serde(default)]
    disease: bool,
    catch: Option<String>,
    wither: Option<Dice>,
    #[serde(default)]
    xp: u8,
    #[serde(default)]
    tithe: bool,
    #[serde(default)]
    teleport: bool,
    explode: Option<ExplodeSpec>,
    becomes: Option<String>,
    rarity: Option<String>,
    gift: Option<Dice>,
    wrath: Option<Dice>,
    #[serde(default)]
    levels: i8,
    #[serde(default)]
    talent: bool,
    #[serde(default)]
    flaw: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusSpec {
    id: String,
    #[serde(default = "one")]
    degrees: u16,
}

fn one() -> u16 {
    1
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectSpec {
    object: String,
    count: Dice,
    #[serde(default = "one_u32")]
    weight: u32,
}

fn one_u32() -> u32 {
    1
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HarmSpec {
    damage: Dice,
    damage_type: Option<super::damage_type::TypeNames>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SummonSpec {
    #[serde(default)]
    danger: u8,
    near: Near,
    count: Option<Dice>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExplodeSpec {
    spell: String,
    radius: i32,
    damage: Dice,
    damage_type: Option<super::damage_type::TypeNames>,
    #[serde(default)]
    heat: bool,
    noise: Option<TrapNoise>,
}

/// The outcome tables, by id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OutcomeTables(BTreeMap<String, OutcomeTable>);

impl OutcomeTables {
    /// The table `id` names.
    pub fn get(&self, id: &str) -> Option<&OutcomeTable> {
        self.0.get(id)
    }

    /// Every table, by id.
    pub fn iter(&self) -> impl Iterator<Item = &OutcomeTable> {
        self.0.values()
    }

    /// Parses the outcome files (`data/outcomes/<id>.toml`; the id is the file's name), in path
    /// order; terrain, statuses, objects, damage types, and spell effects must be loaded.
    pub(super) fn parse<'a>(
        files: impl Iterator<Item = (&'a str, &'a str)>,
        content: &Content,
    ) -> Result<Self, ContentError> {
        let mut tables = BTreeMap::new();
        for (path, text) in files {
            let id = (path.rsplit('/').next())
                .and_then(|name| name.strip_suffix(".toml"))
                .unwrap_or(path)
                .to_string();
            let err = |m: String| ContentError::new(path, m);
            let spec: TableSpec = toml::from_str(text).map_err(|e| err(e.to_string()))?;
            if tables.contains_key(&id) {
                return Err(err(format!("duplicate outcome table `{id}`")));
            }
            let table = resolve(&id, spec, content).map_err(err)?;
            tables.insert(id, table);
        }
        Ok(Self(tables))
    }
}

fn resolve(id: &str, spec: TableSpec, content: &Content) -> Result<OutcomeTable, String> {
    if !(1..=MAX_SECONDS).contains(&spec.seconds) {
        return Err(format!("`seconds` is 1 to {MAX_SECONDS}"));
    }
    let kinds = [spec.kinds.boon, spec.kinds.nothing, spec.kinds.bane];
    if kinds.iter().any(|&w| w > MAX_WEIGHT) || kinds.iter().sum::<u32>() == 0 {
        return Err(format!(
            "`kinds` weights are 0 to {MAX_WEIGHT}, and not all 0"
        ));
    }
    if spec.lean > spec.kinds.boon.min(spec.kinds.bane) {
        return Err("`lean` moves at most what the boon and the bane each have".into());
    }
    if spec.exhaust > 100 {
        return Err("`exhaust` is a percent: 0 to 100".into());
    }
    let terrain = |name: &str| {
        content
            .terrain_id(name)
            .ok_or_else(|| format!("unknown terrain `{name}`"))
    };
    let spent = spec.spent.as_deref().map(terrain).transpose()?;
    if spec.exhaust > 0 && spent.is_none() {
        return Err("a table that may be spent (`exhaust` above 0) needs `spent`".into());
    }
    let mut outcomes: Vec<OutcomeDef> = Vec::new();
    for o in spec.outcome {
        let bad = |m: String| format!("outcome `{}`: {m}", o.id);
        if outcomes.iter().any(|d| d.id == o.id) {
            return Err(format!("duplicate outcome `{}`", o.id));
        }
        if !(1..=MAX_WEIGHT).contains(&o.weight) {
            return Err(bad(format!("`weight` is 1 to {MAX_WEIGHT}")));
        }
        let mut messages = o.messages;
        messages.extend(o.message);
        if messages.is_empty() || messages.iter().any(|m| m.trim().is_empty()) {
            return Err(bad("needs a `message` or `messages`, none empty".into()));
        }
        let effect = resolve_effect(o.effect, content).map_err(bad)?;
        outcomes.push(OutcomeDef {
            id: o.id,
            kind: o.kind,
            tier: o.tier,
            weight: o.weight,
            messages,
            otherwise: o.otherwise,
            effect,
        });
    }
    let table = OutcomeTable {
        id: id.to_string(),
        seconds: spec.seconds,
        kinds,
        lean: spec.lean,
        exhaust: spec.exhaust,
        spent,
        outcomes,
    };
    for kind in OutcomeKind::ALL {
        if table.weight(kind) > 0 && table.total(kind) == 0 {
            return Err(format!("kind `{}` has a weight but no outcome", kind.id()));
        }
    }
    Ok(table)
}

fn amount(text: &str) -> Result<Amount, String> {
    match text {
        "full" => return Ok(Amount::Full),
        "half" => return Ok(Amount::Half),
        _ => {}
    }
    let dice: Dice = text.parse()?;
    if dice.min() < 0 || dice.max() < 1 {
        return Err(format!("`{text}` must give at least 1"));
    }
    Ok(Amount::Dice(dice))
}

fn harmful(spec: Option<HarmSpec>, content: &Content) -> Result<Option<Harmful>, String> {
    let Some(spec) = spec else {
        return Ok(None);
    };
    if spec.damage.min() < 0 || spec.damage.max() < 1 {
        return Err("`damage` dice must be able to do harm".into());
    }
    let kind = one_kind(content, spec.damage_type.as_ref())?;
    Ok(Some(Harmful {
        damage: spec.damage,
        kind,
    }))
}

fn resolve_effect(e: EffectSpec, content: &Content) -> Result<OutcomeEffect, String> {
    let status = |name: &str| {
        content
            .status_id(name)
            .ok_or_else(|| format!("unknown status `{name}`"))
    };
    let cure = e.cure.iter().map(|s| status(s)).collect::<Result<_, _>>()?;
    let effect_status = match e.status {
        Some(s) => {
            let id = status(&s.id)?;
            let max = content.status(id).max;
            if s.degrees == 0 || s.degrees > max {
                return Err(format!("status `{}`: `degrees` is 1 to {max}", s.id));
            }
            Some((id, s.degrees))
        }
        None => None,
    };
    if !(-6..=6).contains(&e.hunger) {
        return Err("`hunger` is -6 to 6 states".into());
    }
    let objects = (e.objects.iter())
        .map(|o| {
            let id = (content.object_id(&o.object))
                .ok_or_else(|| format!("unknown object `{}`", o.object))?;
            if o.count.min() < 1 || o.weight == 0 {
                return Err(format!(
                    "object `{}`: `count` gives at least 1 and `weight` is at least 1",
                    o.object
                ));
            }
            Ok((id, o.count, o.weight))
        })
        .collect::<Result<_, String>>()?;
    if e.know_traps.is_some_and(|r| !(1..=MAX_REACH).contains(&r)) {
        return Err(format!("`know_traps` is 1 to {MAX_REACH} cells"));
    }
    if e.lose_coins > 100 || e.xp > 100 {
        return Err("`lose_coins` and `xp` are percents: 0 to 100".into());
    }
    let summon = match e.summon {
        Some(s) => {
            let count = s.count.unwrap_or_else(|| "1".parse().expect("one"));
            if count.min() < 1 || count.max() > 12 {
                return Err("summon: `count` gives 1 to 12".into());
            }
            Some(OutcomeSummon {
                danger: s.danger,
                near: s.near,
                count,
            })
        }
        None => None,
    };
    let explode = match e.explode {
        Some(x) => {
            let fields = SpellFields {
                spell: Some(x.spell.clone()),
                damage: Some(x.damage),
                kind: one_kind(content, x.damage_type.as_ref())?,
                radius: Some(x.radius),
                heat: x.heat,
                ..SpellFields::default()
            };
            let burst = Burst::new(&fields).map_err(|m| format!("explode: {m}"))?;
            check_effect(content, &x.spell, EffectShape::Burst)?;
            if let Some(n) = x.noise
                && !(1..=crate::noise::MAX_LOUDNESS).contains(&n.loudness)
            {
                return Err(format!(
                    "explode: `noise.loudness` is 1 to {}",
                    crate::noise::MAX_LOUDNESS
                ));
            }
            Some(Explosion {
                burst,
                noise: x.noise,
            })
        }
        None => None,
    };
    let catch = match &e.catch {
        Some(name) => Some(
            content
                .disease_id(name)
                .ok_or_else(|| format!("unknown disease `{name}`"))?,
        ),
        None => None,
    };
    let becomes = match &e.becomes {
        Some(name) => Some(
            content
                .terrain_id(name)
                .ok_or_else(|| format!("unknown terrain `{name}`"))?,
        ),
        None => None,
    };
    let rarity = match e.rarity.as_deref() {
        None => LootGrade::Any,
        Some(_) if e.loot.is_none() => return Err("`rarity` is for `loot`".into()),
        Some("magic") => LootGrade::Magic,
        Some("rare") => LootGrade::Rare,
        Some(other) => return Err(format!("`rarity` is `magic` or `rare`, not `{other}`")),
    };
    for (name, dice) in [("gift", e.gift), ("wrath", e.wrath)] {
        if dice.is_some_and(|d| d.min() < 1) {
            return Err(format!("`{name}` dice give at least 1"));
        }
    }
    if !(-MAX_LEVELS..=MAX_LEVELS).contains(&e.levels) {
        return Err(format!("`levels` is -{MAX_LEVELS} to {MAX_LEVELS}"));
    }
    Ok(OutcomeEffect {
        heal: e.heal.as_deref().map(amount).transpose()?,
        cure,
        cure_by: e.cure_by,
        mend: e.mend,
        mana: e.mana.as_deref().map(amount).transpose()?,
        drain: e.drain,
        status: effect_status,
        hunger: e.hunger,
        objects,
        loot: e.loot,
        know_traps: e.know_traps,
        identify: e.identify,
        forget: e.forget,
        burn: harmful(e.burn, content)?,
        smite: harmful(e.smite, content)?,
        lose_coins: e.lose_coins,
        snuff: e.snuff,
        summon,
        disease: e.disease,
        catch,
        wither: e.wither,
        xp: e.xp,
        tithe: e.tithe,
        teleport: e.teleport,
        explode,
        becomes,
        rarity,
        gift: e.gift,
        wrath: e.wrath,
        levels: e.levels,
        talent: e.talent,
        flaw: e.flaw,
    })
}

#[cfg(test)]
#[path = "outcome_tests.rs"]
mod tests;

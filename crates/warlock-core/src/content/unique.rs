//! Unique items (spec §8 "Magic items": "Unique items"; `data/uniques/` and each campaign's
//! `data/uniques/`, one file per item named by its id): a rare item with a set name, set affixes
//! and parts, an item level, a power ([`Power`]: a rule only a unique carries), and a line of
//! lore. Content places one as `unique:ID` wherever it names objects (a pile, a legend's
//! `objects` and `carry`, and `wield`/`wear` by the same name, a gear line, a kit, a shop's or a
//! scenario's `carry`, a quest reward's `items`); [`Uniques::place`] makes it, and refuses a
//! second placement anywhere in the content (a load error), so with pinned vaults and unique
//! creatures a run holds it at most once with no save state. Loot never rolls one.
//!
//! The item is [`Magic`] of rarity rare with its name, plus [`Magic::unique`] (its id, saved),
//! by which its power and lore are found again ([`Content::unique`]).

use super::level::{ItemPartSpec, item_part, magic_of};
use super::status::{OnHit, OnHitSpec};
use super::{Content, ContentError, ObjectId, StatusId};
use crate::item_parts;
use crate::magic::Magic;
use crate::object::Stack;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};

/// How a placement names a unique item: `unique:widowmaker`.
pub const UNIQUE: &str = "unique:";

/// What a unique item does besides its affixes, while it is worn or wielded (spec §8 "Magic
/// items"): a rule, read in one place each. A new power is a variant here, its one read, its
/// phrase (the unique's `phrase`), and a wiki word (`xtask/src/wiki/uniques.rs`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Power {
    /// Its hits cause a status, by a chance (`{ inflict = "poisoned", chance = 100, severity =
    /// 1 }`): read where a natural attack's `on_hit` is applied (`World::after_blow`), for the
    /// weapon that hit (a launcher's for its shots), through the attack's
    /// [`crate::combat::Profile`]. The target's resistance applies as to any `on_hit`.
    Inflict(OnHit),
    /// Its wearer sees with this vision profile (an index into
    /// [`Content::vision_profiles`]) where it is better than their own (`{ vision =
    /// "darkvision" }`): read where an actor's eyes are chosen.
    Vision(usize),
    /// Its wearer never suffers this status (`{ ward = "frightened", message = "..." }`): read
    /// in `World::afflict`, which refuses it, telling the player `message`.
    Ward { status: StatusId, message: String },
}

/// A unique item as data.
#[derive(Debug)]
pub struct UniqueDef {
    pub id: String,
    /// Its name, which the item shows once identified ("Widowmaker").
    pub name: String,
    pub object: ObjectId,
    /// The item as placed: one, rare, unidentified, with its affixes, parts, and id.
    pub stack: Stack,
    pub power: Option<Power>,
    /// The power in words, for the item description ("Its every cut poisons.").
    pub phrase: Option<String>,
    /// The item description's last line.
    pub lore: Option<String>,
    /// The campaign whose table it is in (`None`: shared, `data/uniques/`).
    pub campaign: Option<String>,
}

impl UniqueDef {
    /// What it is made with, as it truly is (its data: the wiki, tests): rare, its name, its
    /// affixes, its item level, and its id.
    pub fn magic(&self) -> &Magic {
        self.stack.magic().expect("a unique item is magic")
    }
}

/// The unique items, in table order, and which of them content has placed.
#[derive(Debug, Default)]
pub struct Uniques {
    defs: Vec<UniqueDef>,
    ids: BTreeMap<String, usize>,
    /// Whether each has been placed while content loads (the once-a-run check).
    placed: Vec<AtomicBool>,
}

impl Uniques {
    pub fn iter(&self) -> impl Iterator<Item = &UniqueDef> {
        self.defs.iter()
    }

    pub fn get(&self, id: &str) -> Option<&UniqueDef> {
        self.ids.get(id).map(|&i| &self.defs[i])
    }

    /// The unique item `id` as a placement makes it; an error for an unknown id or one already
    /// placed (each unique is named by one placement in all content).
    pub(super) fn place(&self, id: &str) -> Result<Stack, String> {
        let &i = (self.ids.get(id)).ok_or_else(|| format!("unknown unique item `{id}`"))?;
        if self.placed[i].swap(true, Ordering::Relaxed) {
            return Err(format!(
                "`{UNIQUE}{id}` is placed twice: a unique item has one placement in all content"
            ));
        }
        Ok(self.defs[i].stack.clone())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UniqueFile {
    #[serde(default)]
    unique: Vec<UniqueSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UniqueSpec {
    id: String,
    name: String,
    object: String,
    affixes: Vec<String>,
    #[serde(default)]
    parts: Vec<ItemPartSpec>,
    level: Option<u8>,
    power: Option<PowerSpec>,
    lore: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PowerSpec {
    inflict: Option<String>,
    chance: Option<u8>,
    severity: Option<u16>,
    vision: Option<String>,
    ward: Option<String>,
    message: Option<String>,
    phrase: String,
}

/// The longest name a unique item may have, in characters.
const MAX_NAME: usize = 30;

/// Parses the uniques table (`table`: its files in order), against `content`'s objects,
/// affixes, materials, statuses, and vision profiles.
pub(super) fn parse<'a>(
    table: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Uniques, ContentError> {
    let mut uniques = Uniques::default();
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for (path, text) in table {
        let parsed: UniqueFile =
            toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
        for spec in parsed.unique {
            if let Some(first) = seen.insert(spec.id.clone(), path) {
                return Err(super::duplicate_id("unique item", &spec.id, path, first));
            }
            let id = spec.id.clone();
            let mut def = resolve(spec, content)
                .map_err(|m| ContentError::new(path, format!("unique `{id}`: {m}")))?;
            def.campaign = super::campaign::of_path(path).map(str::to_string);
            uniques.ids.insert(def.id.clone(), uniques.defs.len());
            uniques.defs.push(def);
            uniques.placed.push(AtomicBool::new(false));
        }
    }
    Ok(uniques)
}

fn resolve(spec: UniqueSpec, content: &Content) -> Result<UniqueDef, String> {
    let ok_id = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_';
    if spec.id.is_empty() || !spec.id.chars().all(ok_id) {
        return Err("ids are lower-case letters, digits, and underscores".into());
    }
    let name = spec.name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME {
        return Err(format!("`name` is 1 to {MAX_NAME} characters"));
    }
    if spec.affixes.is_empty() {
        return Err("a unique item has `affixes`".into());
    }
    let object = (content.object_id(&spec.object))
        .ok_or_else(|| format!("unknown object `{}`", spec.object))?;
    let def = content.object(object);
    if def.stackable {
        return Err(format!("`{}` stacks: a unique item is one", def.id));
    }
    let mut magic: Magic = magic_of(def, &spec.affixes, Some(name.into()), spec.level, content)?;
    magic.unique = Some(spec.id.clone());
    let mut stack = Stack::with_magic(object, 1, magic);
    if !spec.parts.is_empty() {
        let own = (spec.parts.iter())
            .map(|p| item_part(p, content))
            .collect::<Result<Vec<_>, _>>()?;
        item_parts::check(def, &own, content)?;
        stack = stack.with_parts(own);
    }
    let (power, phrase) = match spec.power {
        Some(p) => {
            let phrase = p.phrase.trim().to_string();
            if phrase.is_empty() {
                return Err("power: `phrase` says what it does".into());
            }
            (Some(power(&p, content)?), Some(phrase))
        }
        None => (None, None),
    };
    let lore = (spec.lore.map(|l| l.trim().to_string())).filter(|l| !l.is_empty());
    Ok(UniqueDef {
        id: spec.id,
        name: name.into(),
        object,
        stack,
        power,
        phrase,
        lore,
        campaign: None,
    })
}

/// The power a `power` table names: exactly one of `inflict` (with `chance` and `severity`),
/// `vision`, and `ward` (with a `message`).
fn power(p: &PowerSpec, content: &Content) -> Result<Power, String> {
    let named = [p.inflict.is_some(), p.vision.is_some(), p.ward.is_some()];
    if named.iter().filter(|&&n| n).count() != 1 {
        return Err("power: name one of `inflict`, `vision`, and `ward`".into());
    }
    if p.inflict.is_none() && (p.chance.is_some() || p.severity.is_some()) {
        return Err("power: `chance` and `severity` belong to `inflict`".into());
    }
    if p.ward.is_none() && p.message.is_some() {
        return Err("power: `message` belongs to `ward`".into());
    }
    if let Some(status) = &p.inflict {
        let spec = OnHitSpec {
            status: Some(status.clone()),
            chance: p
                .chance
                .ok_or("power: `inflict` needs a `chance` (1 to 100)")?,
            severity: Some(p.severity.ok_or("power: `inflict` needs a `severity`")?),
            ..OnHitSpec::default()
        };
        // The status diseases show as comes of a disease, never of a blow.
        let known = |id: &str| {
            let found = (content.status_id(id)).filter(|&s| Some(s) != content.disease_status)?;
            Some((found, content.status(found).max))
        };
        let hit = spec
            .resolve(known, |_| None)
            .map_err(|m| format!("power: {m}"))?;
        return Ok(Power::Inflict(hit));
    }
    if let Some(vision) = &p.vision {
        let index = (content.vision_index(vision))
            .ok_or_else(|| format!("power: unknown vision profile `{vision}`"))?;
        return Ok(Power::Vision(index));
    }
    let ward = p.ward.as_deref().unwrap_or_default();
    let status =
        (content.status_id(ward)).ok_or_else(|| format!("power: unknown status `{ward}`"))?;
    let message = (p.message.as_deref().map(str::trim))
        .filter(|m| !m.is_empty())
        .ok_or("power: `ward` needs a `message` (what the player is told)")?;
    Ok(Power::Ward {
        status,
        message: message.into(),
    })
}

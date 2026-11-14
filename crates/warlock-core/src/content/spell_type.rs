//! What every spell of a damage type does beyond its damage (`[types]` in `data/spells.toml`, the
//! spell types design `docs/design/spell-types-and-new-spells.md` §3, §4): the table and its
//! checks. A type without an entry does nothing beyond damage; weapon blows never read it.
//! `world/magic/spell_type.rs` holds the rules.

use super::spell::BurstIce;
use super::{Content, ContentError, DamageTypeId, LightId, StatusId};
use crate::ice;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct TypesFile {
    #[serde(default)]
    types: BTreeMap<String, TypeSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TypeSpec {
    words: Option<String>,
    rider: Option<RiderSpec>,
    #[serde(default)]
    ends: Vec<String>,
    #[serde(default)]
    heat: bool,
    frost: Option<u16>,
    #[serde(default)]
    douses: bool,
    fire: Option<FireSpec>,
    arc: Option<ArcSpec>,
    #[serde(default)]
    conducts: bool,
    #[serde(default)]
    shove: bool,
    #[serde(default)]
    shatter: bool,
    #[serde(default)]
    doors: bool,
    bite: Option<BiteSpec>,
    #[serde(default)]
    immune_tags: Vec<String>,
    #[serde(default)]
    corrodes: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArcSpec {
    chance: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BiteSpec {
    below_percent: u8,
    percent: u16,
}

/// A rider as written: `{ status, severity, chance }` (chance in percent, default 100); a
/// creature's burst writes its riders the same way (`burst.rs`).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RiderSpec {
    status: String,
    severity: u16,
    #[serde(default = "certain")]
    chance: u8,
}

/// Resolves a rider: its status known.
pub(super) fn rider(r: &RiderSpec, content: &Content) -> Result<TypeRider, String> {
    let id =
        (content.status_id(&r.status)).ok_or_else(|| format!("unknown status `{}`", r.status))?;
    Ok(TypeRider {
        status: id,
        severity: r.severity,
        chance: r.chance,
    })
}

fn certain() -> u8 {
    100
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FireSpec {
    light: String,
    burns: u32,
}

/// A status laid on every creature a spell of the type harms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeRider {
    pub status: StatusId,
    pub severity: u16,
    /// In percent, one draw a creature; at 100 no draw.
    pub chance: u8,
}

/// Harm made more against a target low on hit points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeBite {
    /// It bites a target below this share of its maximum hit points, in percent.
    pub below_percent: u8,
    /// The harm against one, in percent, rounded down.
    pub percent: u16,
}

/// What a spell of a damage type does beyond its damage.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpellType {
    /// The rule as the player reads it after the roll (`burns one in four`).
    pub words: Option<String>,
    pub rider: Option<TypeRider>,
    /// The statuses it ends on every creature it harms.
    pub ends: Vec<StatusId>,
    /// What it does to the ice and water of the cells it reaches.
    pub ice: Option<BurstIce>,
    /// Whether it puts out the fires on those cells.
    pub douses: bool,
    /// The fire a burst leaves on its centre cell: its light, and the seconds it burns.
    pub fire: Option<(LightId, u32)>,
    /// The percent chance a single-target or line strike jumps to a creature beside the one it
    /// harmed (`world/magic/spell_arc.rs`).
    pub arc: Option<u8>,
    /// Whether it strikes everyone standing in the body of water of a creature it harmed.
    pub conducts: bool,
    /// Whether a big hit or a critical knocks the target a cell away.
    pub shove: bool,
    /// Whether a burst's blast shatters the thin ice it reaches.
    pub shatter: bool,
    /// Whether a burst's blast breaks the shut doors it reaches.
    pub doors: bool,
    /// More harm against a target low on hit points.
    pub bite: Option<TypeBite>,
    /// The creature tags immune to the type (`Content::load` lays it on their `resist`).
    pub immune_tags: Vec<String>,
    /// Whether it eats the gear of those it harms: a piece the player wears takes acid's item
    /// damage (`world/combat/corrode.rs`).
    pub corrodes: bool,
}

/// Reads and checks `[types]` of `data/spells.toml`: every key a damage type, every status and
/// light one content has.
pub(super) fn parse(
    path: &str,
    text: &str,
    content: &Content,
) -> Result<BTreeMap<DamageTypeId, SpellType>, ContentError> {
    let file: TypesFile =
        toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
    let mut types = BTreeMap::new();
    for (name, spec) in file.types {
        let kind = content.damage_type_id(&name).ok_or_else(|| {
            ContentError::new(path, format!("types: unknown damage type `{name}`"))
        })?;
        let rule = resolve(spec, content)
            .map_err(|m| ContentError::new(path, format!("types: `{name}`: {m}")))?;
        types.insert(kind, rule);
    }
    Ok(types)
}

/// Every creature type of a tag a type's `immune_tags` names is immune to that type: among its
/// `immune_kinds`, so all that reads them knows (the blow, the status's dose, the Knowledge skill,
/// the description, the wiki) and no creature file need say it. Once the creature types are in.
pub(super) fn lay_immunities(content: &mut Content) {
    let rules: Vec<(DamageTypeId, Vec<String>)> = (content.spell_rules.iter())
        .flat_map(|rules| &rules.types)
        .filter(|(_, rule)| !rule.immune_tags.is_empty())
        .map(|(&kind, rule)| (kind, rule.immune_tags.clone()))
        .collect();
    for (kind, tags) in rules {
        for def in &mut content.creatures {
            if tags.iter().any(|tag| def.tags.contains(tag)) && !def.immune_kinds.contains(&kind) {
                def.immune_kinds.push(kind);
                def.immune_kinds.sort();
            }
        }
    }
}

fn resolve(spec: TypeSpec, content: &Content) -> Result<SpellType, String> {
    let status =
        |name: &str| (content.status_id(name)).ok_or_else(|| format!("unknown status `{name}`"));
    let rider = (spec.rider.as_ref())
        .map(|r| rider(r, content))
        .transpose()?;
    let ends = (spec.ends.iter())
        .map(|name| status(name))
        .collect::<Result<_, _>>()?;
    let ice = match (spec.heat, spec.frost) {
        (true, Some(_)) => return Err("`heat` or `frost`, not both".into()),
        (true, None) => Some(BurstIce::Melts),
        // The ice the simulation keeps is never thicker (`crate::ice::MAX`).
        (false, Some(mm)) if mm <= ice::MAX_MM => Some(BurstIce::Freezes(u32::from(mm) * 1000)),
        (false, Some(_)) => {
            return Err(format!("`frost` must be at most {} mm", ice::MAX_MM));
        }
        (false, None) => None,
    };
    let fire = match spec.fire {
        Some(f) => {
            let light = (content.light_id(&f.light))
                .ok_or_else(|| format!("fire: unknown light `{}`", f.light))?;
            Some((light, f.burns))
        }
        None => None,
    };
    let bite = spec.bite.map(|b| TypeBite {
        below_percent: b.below_percent,
        percent: b.percent,
    });
    for tag in &spec.immune_tags {
        super::tag::check(tag).map_err(|m| format!("`immune_tags`: {m}"))?;
    }
    Ok(SpellType {
        words: spec.words,
        rider,
        ends,
        ice,
        douses: spec.douses,
        fire,
        arc: spec.arc.map(|a| a.chance),
        conducts: spec.conducts,
        shove: spec.shove,
        shatter: spec.shatter,
        doors: spec.doors,
        bite,
        immune_tags: spec.immune_tags,
        corrodes: spec.corrodes,
    })
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    /// The shipped content with `line` in place of the cold entry of `[types]`: what loading it
    /// says.
    fn loads(line: &str) -> Result<Content, String> {
        let mut files = ContentFiles::builtin();
        let text = files.get_optional("data/spells.toml").expect("the table");
        let text: String = (text.lines())
            .map(|l| if l.starts_with("cold = ") { line } else { l })
            .collect::<Vec<_>>()
            .join("\n");
        files.insert("data/spells.toml", &text);
        Content::load(&files).map_err(|e| e.message)
    }

    #[test]
    fn the_shipped_table_loads_and_a_type_without_an_entry_has_no_rule() {
        let content = crate::test_support::builtin();
        let of = |id: &str| content.spell_type(content.damage_type_id(id));
        let fire = of("fire").expect("fire");
        assert_eq!(fire.rider.expect("a rider").chance, 25);
        assert!(fire.fire.is_some());
        let cold = of("cold").expect("cold");
        assert_eq!(cold.rider.expect("a rider").chance, 100);
        assert!(cold.douses);
        let force = of("force").expect("force");
        assert!(force.shove && force.shatter && force.doors);
        let poison = of("poison").expect("poison");
        assert_eq!(poison.rider.expect("a rider").chance, 100);
        let necrotic = of("necrotic").expect("necrotic");
        assert_eq!(necrotic.immune_tags, ["undead"]);
        assert_eq!(necrotic.bite.expect("a bite").percent, 150);
        assert!(of("acid").expect("acid").rider.is_some());
        assert!(content.spell_type(None).is_none());
        assert!(of("slashing").is_none(), "a blow's type has no entry");
        let electricity = of("electricity").expect("electricity");
        assert_eq!(electricity.arc, Some(25));
        assert!(electricity.conducts);
    }

    #[test]
    fn every_status_and_light_is_checked_at_load() {
        for (line, says) in [
            (
                "cold = { rider = { status = \"nope\", severity = 1 } }",
                "unknown status `nope`",
            ),
            ("cold = { ends = [\"nope\"] }", "unknown status `nope`"),
            (
                "cold = { fire = { light = \"nope\", burns = 60 } }",
                "unknown light `nope`",
            ),
            ("cold = { heat = true, frost = 100 }", "not both"),
            ("cold = { frost = 1001 }", "`frost`"),
            ("cold = { immune_tags = [\"Not A Tag\"] }", "`immune_tags`"),
            ("cold = { nonsense = true }", "nonsense"),
            ("nether = { douses = true }", "unknown damage type `nether`"),
        ] {
            let e = loads(line).err().unwrap_or_else(|| panic!("{line} loads"));
            assert!(e.contains(says), "{line}: {e}");
        }
    }
}

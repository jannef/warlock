//! Material drops (spec §7 "Material drops"; the design: docs/design/recipes.md §9, §10; #1341):
//! what a creature's body leaves of crafting stock where it dies, by creature tag. One file per
//! tag, `data/drops/<tag>.toml` (its file name is the tag), with `bodiless` and `[[drop]]` rows;
//! a creature's own `[[creature.drops]]` rows add to its tags', and `drops_from_tags = false`
//! switches its tags' rows off. The loader resolves every creature type's rows once
//! ([`resolve`]): every tag file it carries in path order, then its own; a tag row only for a type
//! that leaves a corpse unless its file is `bodiless`; each chance scaled by its size's `drops`
//! (`sizes.toml`: a small one rolls at half). The roll is `world/material_drops.rs`.

use super::{Content, ContentError, CreatureId, MaterialId, ObjectId, PER_MILLE};
use crate::dice::Dice;
use serde::Deserialize;

/// One drop row as written, in a tag file or a creature's own `[[creature.drops]]`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DropSpec {
    /// Stock of this material (a piece object, `piece.rs`): `count` pieces.
    #[serde(default)]
    material: Option<String>,
    /// Or this object, `count` of it.
    #[serde(default)]
    object: Option<String>,
    /// How many, as dice (`"1d2"`, `"1"`).
    count: Dice,
    /// The chance in percent, 1 to 100.
    chance: u8,
}

/// A tag file as written.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TagFileSpec {
    #[serde(default)]
    bodiless: bool,
    #[serde(default)]
    drop: Vec<DropSpec>,
}

/// What a row gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropGoods {
    /// Pieces of a material (`piece::make`).
    Material(MaterialId),
    /// An object.
    Object(ObjectId),
}

/// Where a creature's resolved row comes from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DropFrom {
    /// A tag file's row.
    Tag(String),
    /// The creature's own `[[creature.drops]]`.
    Own,
}

/// One drop row, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drop {
    pub goods: DropGoods,
    /// How many of it, as dice; a roll below 1 gives nothing.
    pub count: Dice,
    /// The chance in per mille ([`PER_MILLE`]): as written × 10 in a tag file, and for a
    /// creature's resolved row scaled by its size.
    pub chance: u16,
    pub from: DropFrom,
}

/// A tag file, resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagDrops {
    /// The tag (the file's name).
    pub tag: String,
    /// Whether its rows apply to a creature that leaves no corpse too (the skeleton's bone).
    pub bodiless: bool,
    /// Its rows, chances as written (× 10, per mille).
    pub rows: Vec<Drop>,
}

/// A creature's own rows as written, and whether its tags' rows apply.
#[derive(Clone, Debug, Default)]
pub(super) struct OwnDrops {
    pub rows: Vec<DropSpec>,
    pub from_tags: bool,
}

/// The tag files of `data/drops/` (and the campaigns'), in path order.
pub(super) fn parse<'a>(
    files: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Vec<TagDrops>, ContentError> {
    let tags = content.tags();
    let mut out: Vec<TagDrops> = Vec::new();
    for (path, text) in files {
        let name = path.rsplit('/').next().unwrap_or(path);
        if name == "README.toml" {
            continue;
        }
        let tag = name.trim_end_matches(".toml").to_string();
        let err = |m: String| ContentError::new(path, m);
        if !tags.contains(&tag.as_str()) {
            return Err(err(format!(
                "no creature carries the tag `{tag}` (the file's name)"
            )));
        }
        if out.iter().any(|t| t.tag == tag) {
            return Err(err(format!("the tag `{tag}` has a drops file already")));
        }
        let spec: TagFileSpec = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let rows = (spec.drop.iter())
            .map(|r| resolve_row(r, content, DropFrom::Tag(tag.clone())))
            .collect::<Result<Vec<_>, _>>()
            .map_err(err)?;
        out.push(TagDrops {
            tag,
            bodiless: spec.bodiless,
            rows,
        });
    }
    Ok(out)
}

/// `spec` resolved, from `from`: its chance per mille as written.
fn resolve_row(spec: &DropSpec, content: &Content, from: DropFrom) -> Result<Drop, String> {
    if !(1..=100).contains(&spec.chance) {
        return Err(format!("drop: chance {} is not 1 to 100", spec.chance));
    }
    let goods = match (&spec.material, &spec.object) {
        (Some(m), None) => {
            let id = (content.material_id(m)).ok_or_else(|| format!("drop: no material `{m}`"))?;
            let def = content.material(id);
            if def.gem.is_some() {
                return Err(format!(
                    "drop: `{m}` is a gem: gems come by the gems' chance, never as a drop"
                ));
            }
            if def.group.is_none() {
                return Err(format!(
                    "drop: `{m}` has no group, so no stock is made of it"
                ));
            }
            DropGoods::Material(id)
        }
        (None, Some(o)) => DropGoods::Object(
            content
                .object_id(o)
                .ok_or_else(|| format!("drop: no object `{o}`"))?,
        ),
        _ => return Err("drop: give `material` or `object`, one of them".into()),
    };
    Ok(Drop {
        goods,
        count: spec.count,
        chance: u16::from(spec.chance) * 10,
        from,
    })
}

/// Every creature type's resolved rows, by [`CreatureId`] (the module docs): its tags' files'
/// rows in path order (a corpse needed unless the file is bodiless; none for an animated
/// thing, which breaks into its pieces already, or with `drops_from_tags = false`), then its
/// own; each chance scaled by its size's `drops`.
pub(super) fn resolve(
    content: &Content,
    tags: &[TagDrops],
    own: &[OwnDrops],
) -> Result<Vec<Vec<Drop>>, ContentError> {
    let mut all = Vec::with_capacity(own.len());
    for (i, own) in own.iter().enumerate() {
        let def = content.creature(CreatureId(i as u16));
        let mut rows: Vec<Drop> = Vec::new();
        if own.from_tags && def.animated.is_none() {
            for file in tags {
                if def.tags.contains(&file.tag) && (def.corpse || file.bodiless) {
                    rows.extend(file.rows.iter().cloned());
                }
            }
        }
        for spec in &own.rows {
            let row = resolve_row(spec, content, DropFrom::Own)
                .map_err(|m| ContentError::new("data/creatures", format!("{}: {m}", def.id)))?;
            rows.push(row);
        }
        let percent = u32::from(content.size(def.size).drops);
        for row in &mut rows {
            let scaled = u32::from(row.chance) * percent / 100;
            row.chance = u16::try_from(scaled.min(PER_MILLE)).unwrap_or(1000);
        }
        rows.retain(|r| r.chance > 0);
        all.push(rows);
    }
    Ok(all)
}

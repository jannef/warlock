//! What an item is made of where it differs from its base item (spec §7 "Objects and materials",
//! "Gems"; #191): its own parts, carried on its stack beside its magic
//! ([`crate::object::Stack::parts`]), in saves and replays.
//!
//! An item's own parts are a short list of [`ItemPart`]s, each a part (`None`: the main part),
//! the material it is made of, and for a gem its size and cut. Each takes the place of its base
//! item's part of that name, keeping its share of the weight, or comes after the base item's
//! parts if it has none of that name (a gem the item was set with: it weighs nothing beside the
//! rest). A loose gem is the gems' object ([`crate::content::Gems::object`]) whose main part is
//! its gem; an ornament is a gem part of any other item, never its main part.
//!
//! Everything is saved by id (materials, parts, sizes, cuts), as affixes are, so adding content
//! keeps saves; an item whose ids content does not know fails [`check`] when a save loads.
//! Metal variants (#193) will add their own main-part materials the same way.

use crate::content::{Content, Cutting, MAX_OBJECT_PARTS, MaterialId, ObjectDef, Part, PartId};
use serde::{Deserialize, Serialize};

/// A gem's size and cut, by their ids in `data/gems.toml`, as a stack saves them.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GemCut {
    pub size: String,
    pub cut: String,
}

/// One part of an item that differs from its base item's.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemPart {
    /// What part it is: a part id in `data/parts.toml` (`pommel`); `None` for the main part.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    /// What it is made of: a material id.
    pub of: String,
    /// A gem's size and cut (a part of a gem material has one, any other none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gem: Option<GemCut>,
}

impl ItemPart {
    /// `gem` cut as `cutting`, as the part `part` (`None`: the main part, a loose gem's).
    pub fn gem(content: &Content, part: Option<PartId>, gem: MaterialId, cutting: Cutting) -> Self {
        let gems = content
            .gems()
            .expect("gems are cut only with data/gems.toml");
        Self {
            part: part.map(|p| content.part_defs()[usize::from(p.0)].id.clone()),
            of: content.material(gem).id.clone(),
            gem: Some(GemCut {
                size: gems.size(cutting.size).id.clone(),
                cut: gems.cut(cutting.cut).id.clone(),
            }),
        }
    }
}

/// Where among `base` a part `part` goes: the main part's place for `None` (or the main part's
/// own name), the place of the base part of that name, or `None`: after them all.
fn place(base: &[Part], part: Option<PartId>) -> Option<usize> {
    match part {
        None => Some(0),
        Some(p) => base.iter().position(|b| b.part == Some(p)),
    }
}

/// One own part with its ids resolved: its part, material, and gem cutting; `None` if content
/// does not know an id (a checked item has none).
fn resolve_one(
    own: &ItemPart,
    content: &Content,
) -> Option<(Option<PartId>, MaterialId, Option<Cutting>)> {
    let part = match &own.part {
        Some(id) => Some(content.part_id(id)?),
        None => None,
    };
    let material = content.material_id(&own.of)?;
    let gem = match &own.gem {
        Some(g) => {
            let gems = content.gems()?;
            Some(Cutting {
                size: gems.size_id(&g.size)?,
                cut: gems.cut_id(&g.cut)?,
            })
        }
        None => None,
    };
    Some((part, material, gem))
}

/// The parts of an item of base parts `base` made with `own`: each own part in place of the
/// base part it names, keeping its share, or after them all with no share.
pub fn resolve(base: &[Part], own: &[ItemPart], content: &Content) -> Vec<Part> {
    let mut parts = base.to_vec();
    for o in own {
        let Some((part, material, gem)) = resolve_one(o, content) else {
            continue;
        };
        match place(base, part) {
            Some(i) => {
                parts[i].material = material;
                parts[i].gem = gem;
            }
            None => parts.push(Part {
                part,
                material,
                share: 0,
                gem,
            }),
        }
    }
    parts
}

/// Whether `own` is a valid make of an item of `def` for `content` (a guard for saves and level
/// files): known ids, a size and cut exactly on the parts of a gem material, no part named twice,
/// at most [`MAX_OBJECT_PARTS`] parts in all; the gems' object is made of one gem, its main part,
/// and no other item has a gem as its main part.
pub fn check(def: &ObjectDef, own: &[ItemPart], content: &Content) -> Result<(), String> {
    if own.is_empty() {
        return Err("an item's own parts list one part at least".into());
    }
    let loose = content
        .gems()
        .is_some_and(|g| content.object(g.object).id == def.id);
    let mut places = Vec::new();
    let mut added = 0;
    for o in own {
        let what = o.part.as_deref().unwrap_or("main part");
        let (part, material, gem) = resolve_one(o, content)
            .ok_or_else(|| format!("{what} of `{}`: an id content does not know", o.of))?;
        let is_gem = content.material(material).gem.is_some();
        if is_gem != gem.is_some() {
            return Err(format!(
                "{what} of `{}`: a part of a gem has a size and cut, and only it",
                o.of
            ));
        }
        let at = place(&def.parts, part);
        if at.is_none() {
            added += 1;
        }
        // A base part is named once, however it is named (the main part as `None` or by its
        // own name); a part the base does not have, once by its name.
        let twice = match at {
            Some(_) => places.iter().any(|(a, _)| *a == at),
            None => places.contains(&(at, part)),
        };
        if twice {
            return Err(format!("{what}: named twice"));
        }
        places.push((at, part));
        match (loose, at == Some(0)) {
            (true, false) => return Err(format!("`{}` is made of one gem, its main part", def.id)),
            (true, true) if !is_gem => return Err(format!("`{}` is made of a gem", def.id)),
            (false, true) if is_gem => {
                return Err(format!(
                    "`{}`: a gem is set in an item, never its main part",
                    def.id
                ));
            }
            _ => {}
        }
    }
    if def.parts.len() + added > MAX_OBJECT_PARTS {
        return Err(format!("at most {MAX_OBJECT_PARTS} parts in all"));
    }
    Ok(())
}

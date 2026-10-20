//! Gems (spec §7 "Gems"): the ten gems, their sizes and cuts, how loot finds them, and where items
//! are set with them.

use super::combat::percent;
use super::order;
use super::text::capital;
use std::fmt::Write as _;
use warlock_core::content::{Content, GemDef, ItemClassId, PartId};
use warlock_core::item_parts::ItemPart;
use warlock_core::object::Stack;

/// The deepest item level the wiki shows loot chances at: the Depths' bottom.
const DEEP: u8 = 8;

/// The gems by worth (dearest first, then name): their colour, what a small one of fair cut is
/// worth, and how often loot finds each among the gems.
pub(super) fn gems(content: &Content) -> String {
    let mut out =
        String::from("| Gem | Colour | Worth (plain) | Found among gems |\n|---|---|---:|---:|\n");
    let mut all: Vec<(&str, &GemDef)> = (order::materials(content).into_iter())
        .filter_map(|(_, m)| Some((m.name.as_str(), m.gem.as_ref()?)))
        .collect();
    all.sort_by(|a, b| b.1.value.cmp(&a.1.value).then(a.0.cmp(b.0)));
    let total: u32 = all.iter().map(|(_, g)| g.rarity).sum();
    for (name, gem) in all {
        let found = match gem.rarity {
            0 => "never".to_string(),
            r => percent(r, total.max(1)),
        };
        let _ = writeln!(
            out,
            "| {} | `{}` | {} gold | {found} |",
            capital(name),
            gem.color,
            gem.value
        );
    }
    out
}

/// A multiplier in percent as a factor: ×0.3, ×1, ×15.
fn times(percent: u32) -> String {
    let whole = percent / 100;
    match percent % 100 {
        0 => format!("×{whole}"),
        r if r % 10 == 0 => format!("×{whole}.{}", r / 10),
        r => format!("×{whole}.{r:02}"),
    }
}

/// The gem sizes, smallest first: their worth, weight, and how often loot finds each, at the
/// first item level and deep down.
pub(super) fn sizes(content: &Content) -> String {
    let mut out = format!(
        "| Size | Worth | Weight | Found at depth 1 | Found at depth {DEEP} |\n\
         |---|---:|---:|---:|---:|\n"
    );
    let Some(gems) = content.gems() else {
        return out;
    };
    let (shallow, deep) = (gems.size_weights(1), gems.size_weights(DEEP));
    let sum = |w: &[(_, u32)]| w.iter().map(|(_, n)| *n).sum::<u32>().max(1);
    for (i, size) in gems.sizes().iter().enumerate() {
        let plain = if i == usize::from(gems.plain().size.0) {
            " (plain)"
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "| {}{plain} | {} | {} g | {} | {} |",
            capital(&size.name),
            times(size.value),
            size.weight,
            percent(shallow[i].1, sum(&shallow)),
            percent(deep[i].1, sum(&deep)),
        );
    }
    out
}

/// The cuts, roughest first: their worth and how often loot finds each, at the first item level
/// and deep down.
pub(super) fn cuts(content: &Content) -> String {
    let mut out = format!(
        "| Cut | Worth | Found at depth 1 | Found at depth {DEEP} |\n|---|---:|---:|---:|\n"
    );
    let Some(gems) = content.gems() else {
        return out;
    };
    let (shallow, deep) = (gems.cut_weights(1, false), gems.cut_weights(DEEP, false));
    let sum = |w: &[(_, u32)]| w.iter().map(|(_, n)| *n).sum::<u32>().max(1);
    for (i, cut) in gems.cuts().iter().enumerate() {
        let note = if i == usize::from(gems.plain().cut.0) {
            " (plain)"
        } else if cut.rough {
            " (never set in an item)"
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "| {}{note} | {} | {} | {} |",
            capital(&cut.name),
            times(cut.value),
            percent(shallow[i].1, sum(&shallow)),
            percent(deep[i].1, sum(&deep)),
        );
    }
    out
}

/// Where loot sets items with gems: by item class (in class order), the part the gem becomes, how
/// often, and an example name; then the objects set with a gem as another part.
pub(super) fn ornaments(content: &Content) -> String {
    let mut out =
        String::from("| Items | Set with a gem | How often | For example |\n|---|---|---:|---|\n");
    let Some(gems) = content.gems() else {
        return out;
    };
    let mut placed = gems.ornaments().to_vec();
    placed.sort_by_key(|o| o.class);
    for o in placed {
        let part = &content.part_defs()[usize::from(o.part.0)];
        let example = example(content, o.class, o.part);
        let colour = if o.colour {
            ", in its gem's colour"
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "| {} | as its {}{colour} | {}% | {example} |",
            capital(&content.item_class(o.class).name.to_lowercase()),
            part.name,
            o.chance,
        );
    }
    let mut own: Vec<(String, String)> = (order::objects(content).into_iter())
        .filter_map(|(_, o)| {
            let part = &content.part_defs()[usize::from(o.ornament?.0)];
            Some((o.plural.clone(), part.name.clone()))
        })
        .collect();
    own.sort();
    let mut by_part: Vec<(String, Vec<String>)> = Vec::new();
    for (object, part) in own {
        match by_part.iter_mut().find(|(p, _)| *p == part) {
            Some((_, list)) => list.push(object),
            None => by_part.push((part, vec![object])),
        }
    }
    for (part, objects) in by_part {
        let _ = writeln!(
            out,
            "\nThese are set with a gem as their {part} instead: {}.",
            objects.join(", ")
        );
    }
    out
}

/// What the dearest object of `class` is called set with an emerald of the plain size and cut
/// as its `part`.
fn example(content: &Content, class: ItemClassId, part: PartId) -> String {
    let Some((id, _)) = (order::objects(content).into_iter())
        .rfind(|(_, o)| o.class == class && o.ornament.is_none() && o.corpse.is_none())
    else {
        return "-".into();
    };
    let (Some(gems), Some(emerald)) = (content.gems(), content.material_id("emerald")) else {
        return "-".into();
    };
    let own = ItemPart::gem(content, Some(part), emerald, gems.plain());
    Stack::one(id).with_parts(vec![own]).describe(content)
}

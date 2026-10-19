//! Objects: the objects, shops, equipment slots, materials, and the pack.

use super::actions::at_normal_pace;
use super::text::{capital, glyph, yes};
use std::fmt::Write as _;
use warlock_core::content::{BASE_TURN, Content, ObjectClass, ObjectDef, ObjectId, price};
use warlock_core::object::{INVENTORY_SLOTS, format_weight};

/// The equipment slots in the order the screen lists them, with their letters, what goes in
/// each, and how long putting a thing on there or taking it off takes.
pub(super) fn equipment(content: &Content) -> String {
    let mut out = String::from(
        "| Letter | Slot | What goes there | On or off takes |\n|:---:|---|---|---:|\n",
    );
    for (i, (slot, def)) in content.slot_ids().enumerate() {
        let fits: Vec<String> = (content.object_defs().iter().enumerate())
            .filter(|&(o, _)| content.fits_properly(ObjectId(o as u16), slot))
            .map(|(_, o)| {
                let mut name = o.name.clone();
                if o.two_handed {
                    name.push_str(" (both hands)");
                }
                if let Some(kind) = &o.launches {
                    let ammo = (content.object_defs().iter())
                        .find(|a| a.ammo.as_ref() == Some(kind))
                        .map_or(kind.clone(), |a| a.plural.clone());
                    let _ = write!(name, " (shoots {ammo})");
                }
                name
            })
            .collect();
        let mut fits = if fits.is_empty() {
            "-".to_string()
        } else {
            capital(&fits.join(", "))
        };
        if def.takes_weapons {
            fits.push_str(", any other weapon, and anything made for throwing");
        }
        let cost = content.action_costs().cost(def.action);
        let _ = writeln!(
            out,
            "| `{}` | {} | {fits} | {} |",
            char::from(b'a' + i as u8),
            def.name,
            at_normal_pace(price(cost, BASE_TURN))
        );
    }
    out
}

/// Where an object goes, if it can be worn or wielded: the slots that fit it.
pub(super) fn goes_on(content: &Content, object: usize) -> String {
    let id = ObjectId(object as u16);
    // Where it belongs; else where it may be readied (rocks: the missile weapon slot).
    let names = |proper: bool| -> Vec<String> {
        (content.slot_ids())
            .filter(|&(s, _)| {
                if proper {
                    content.fits_properly(id, s)
                } else {
                    content.fits(id, s)
                }
            })
            .map(|(_, s)| s.name.to_lowercase())
            .collect()
    };
    let slots = Some(names(true))
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| names(false));
    if slots.is_empty() {
        return "-".into();
    }
    let both = if content.object(id).two_handed {
        " (both hands)"
    } else {
        ""
    };
    format!("{}{both}", capital(&slots.join(" or ")))
}

/// What a glyph means for objects (ADOM-like classes); `None` for a glyph no class uses.
pub(super) fn object_class(glyph: char) -> Option<&'static str> {
    Some(match glyph {
        ')' => "weapon",
        '}' => "missile weapon",
        '/' => "missile",
        '[' => "armor",
        '(' => "tool",
        '!' => "potion",
        '?' => "scroll",
        '%' => "food",
        '*' => "stone or gem",
        '$' => "coin",
        '=' => "ring",
        '"' => "amulet",
        _ => return None,
    })
}

/// Every object but corpses (Food and corpses has those).
pub(super) fn objects(content: &Content) -> String {
    let mut out = String::from(
        "| Object | Glyph | Kind | Material | Weight (each) | Worth (each) | Stacks | Worn or wielded on |\n\
         |---|:---:|---|---|---:|---:|:---:|---|\n",
    );
    for (i, o) in content.object_defs().iter().enumerate() {
        if o.corpse.is_some() {
            continue;
        }
        let worth = match o.value {
            0 => "nothing".to_string(),
            v => format!("{v} gold"),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {worth} | {} | {} |",
            capital(&o.name),
            glyph(o.glyph),
            object_class(o.glyph).unwrap_or("-"),
            content.material(o.material).name,
            format_weight(u64::from(o.weight)),
            yes(o.stackable),
            goes_on(content, i)
        );
    }
    out
}

/// The kinds of shop (spec §7 "Shops"): what each sells and buys, by class, and its prices in
/// words of the value (an object's worth, in the objects table).
pub(super) fn shops(content: &Content) -> String {
    let mut out =
        String::from("| Shop | Sign | Sells | Buys | Asks | Pays |\n|---|---|---|---|---:|---:|\n");
    let Some(shops) = content.shops() else {
        return out;
    };
    let classes = |list: &[ObjectClass]| {
        let names: Vec<&str> = list.iter().map(|c| c.name()).collect();
        match names.as_slice() {
            [] => "nothing".to_string(),
            [one] => (*one).to_string(),
            [a, b] => format!("{a} and {b}"),
            [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
        }
    };
    for def in shops.defs() {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {}% of the worth | {}% of the worth |",
            capital(&def.name),
            def.sign("Grimbold"),
            classes(&def.stock),
            classes(&def.buys),
            def.markup,
            def.pays
        );
    }
    out
}

pub(super) fn materials(content: &Content) -> String {
    let mut out = String::from("| Material | Objects made of it |\n|---|---|\n");
    for (i, m) in content.materials().iter().enumerate() {
        let of_it = |o: &&ObjectDef| usize::from(o.material.0) == i;
        let objects = content.object_defs().iter().filter(of_it);
        let mut made: Vec<&str> = (objects.clone())
            .filter(|o| o.corpse.is_none())
            .map(|o| o.plural.as_str())
            .collect();
        if objects.clone().any(|o| o.corpse.is_some()) {
            made.push("corpses");
        }
        let made = if made.is_empty() {
            "-".to_string()
        } else {
            made.join(", ")
        };
        let _ = writeln!(out, "| {} | {made} |", capital(&m.name));
    }
    out
}

/// The pack's size and the player's own weight.
pub(super) fn pack(content: &Content) -> String {
    let player = content
        .creature_id("player")
        .map_or(0, |id| content.creature(id).weight);
    let last = char::from(b'a' + (INVENTORY_SLOTS - 1) as u8);
    format!(
        "| Fact | Value |\n|---|---|\n\
         | Stacks your pack holds | {INVENTORY_SLOTS}, lettered `a` to `{last}` |\n\
         | Your own weight | {} |\n",
        format_weight(u64::from(player))
    )
}

//! Objects: the objects, their item classes, shops, equipment slots, materials, and the pack.

use super::actions::at_normal_pace;
use super::order;
use super::text::{Term, anchored, capital, glyph, yes};
use std::fmt::Write as _;
use warlock_core::content::{BASE_TURN, Content, ItemClassId, ObjectDef, ObjectId, price};
use warlock_core::object::format_weight;

/// The equipment slots in the order the screen lists them, with their letters, what goes in
/// each, and how long putting a thing on there or taking it off takes.
pub(super) fn equipment(content: &Content) -> String {
    let mut out = String::from(
        "| Letter | Slot | What goes there | On or off takes |\n|:---:|---|---|---:|\n",
    );
    for (i, (slot, def)) in content.slot_ids().enumerate() {
        let fits: Vec<String> = (order::objects(content).into_iter())
            .filter(|&(id, _)| content.fits_properly(id, slot))
            .map(|(_, o)| {
                let mut name = o.name.clone();
                if o.two_handed {
                    name.push_str(" (both hands)");
                }
                if let Some(kind) = &o.launches {
                    let ammo =
                        (content.ammunition(kind)).map_or(kind.clone(), |a| a.plural.clone());
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
pub(super) fn goes_on(content: &Content, id: ObjectId) -> String {
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

/// The item classes in the order item lists group by, each with the objects of it (by name, in
/// the plural; corpses as one entry).
pub(super) fn item_classes(content: &Content) -> String {
    let mut out = String::from("| Class | Objects of it |\n|---|---|\n");
    for (i, class) in content.item_classes().iter().enumerate() {
        let of_it = |o: &&ObjectDef| usize::from(o.class.0) == i;
        let objects = content.object_defs().iter().filter(of_it);
        let mut named: Vec<&ObjectDef> = objects.clone().filter(|o| o.corpse.is_none()).collect();
        named.sort_by_cached_key(|o| o.name.to_lowercase());
        let mut names: Vec<String> = named.iter().map(|o| o.plural.clone()).collect();
        if objects.clone().any(|o| o.corpse.is_some()) {
            names.push("corpses".into());
        }
        let names = if names.is_empty() {
            "-".to_string()
        } else {
            capital(&names.join(", "))
        };
        let _ = writeln!(
            out,
            "| {} | {names} |",
            anchored(Term::ItemClass, &class.id, &class.name)
        );
    }
    out
}

/// Every object but corpses (Food and corpses has those), by class, then name.
pub(super) fn objects(content: &Content) -> String {
    let mut out = String::from(
        "| Object | Glyph | Class | Made of | Weight (each) | Worth (each) | Stacks | Worn or wielded on |\n\
         |---|:---:|---|---|---:|---:|:---:|---|\n",
    );
    let loose = content.gems().map(|g| g.object);
    for (id, o) in order::objects(content) {
        if o.corpse.is_some() {
            continue;
        }
        let worth = match o.value {
            0 => "nothing".to_string(),
            v => format!("{v} gold"),
        };
        // A loose gem is made of its gem, which weighs and prices it (Gems has the tables).
        let (made, weight, worth) = match (loose == Some(id), content.gems()) {
            (true, Some(gems)) => {
                let grams = |s: Option<&warlock_core::content::GemSize>| s.map_or(0, |s| s.weight);
                let (least, most) = (grams(gems.sizes().first()), grams(gems.sizes().last()));
                (
                    "its gem".to_string(),
                    format!("{least}–{most} g"),
                    "its gem's".to_string(),
                )
            }
            _ => (
                content.describe_parts(&o.parts),
                format_weight(u64::from(o.weight)),
                worth,
            ),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {made} | {weight} | {worth} | {} | {} |",
            capital(&o.name),
            glyph(o.glyph),
            content.item_class(o.class).name.to_lowercase(),
            yes(o.stackable),
            goes_on(content, id)
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
    let classes = |list: &[ItemClassId]| {
        let names: Vec<String> = (list.iter())
            .map(|&c| content.item_class(c).name.to_lowercase())
            .collect();
        match names.as_slice() {
            [] => "nothing".to_string(),
            [one] => one.clone(),
            [a, b] => format!("{a} and {b}"),
            [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
        }
    };
    for def in shops.defs() {
        // What a generated one always has, first: "scrolls of identify always".
        let always: Vec<String> = (def.always.iter())
            .map(|&o| content.object(o).plural.clone())
            .collect();
        let stock = match always.as_slice() {
            [] => classes(&def.stock),
            _ => format!("{} ({} always)", classes(&def.stock), always.join(", ")),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {}% of the worth | {}% of the worth |",
            capital(&def.name),
            def.sign("Grimbold"),
            stock,
            classes(&def.buys),
            def.markup,
            def.pays
        );
    }
    out
}

/// The player's own weight (the pack holds any number of stacks: weight is its only limit).
pub(super) fn pack(content: &Content) -> String {
    let player = content
        .creature_id("player")
        .map_or(0, |id| content.creature(id).weight);
    format!(
        "| Fact | Value |\n|---|---|\n| Your own weight | {} |\n",
        format_weight(u64::from(player))
    )
}

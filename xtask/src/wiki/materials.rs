//! Materials (spec §7 "Materials", #193, #194): their properties, how often loot and creatures'
//! gear are made of each, what each item may be made of, and what a few items are like in each
//! metal.

use super::order;
use super::util::{capital, percent, signed, times, yes};
use std::fmt::Write as _;
use warlock_core::content::{
    Content, Fit, MaterialDef, MaterialId, ObjectDef, PartRole, VERY_RARE,
};
use warlock_core::item_parts::ItemPart;
use warlock_core::object::{Stack, format_weight};
use warlock_core::variant;

/// Every material, by group (metals first, then by group name, then those of no group) and
/// then by name.
fn by_group(content: &Content) -> Vec<(MaterialId, &MaterialDef)> {
    let mut all = order::materials(content);
    all.sort_by_cached_key(|(_, m)| {
        let group = match m.group.as_deref() {
            Some("metal") => (0, String::new()),
            Some(g) => (1, g.to_string()),
            None => (2, String::new()),
        };
        (group, m.name.to_lowercase())
    });
    all
}

/// How often a weight is, in words (materials/README.toml): very common 800, common 300,
/// uncommon 100, rare 40, very rare 10.
fn how_often(weight: u32) -> &'static str {
    match weight {
        0 => "never",
        1..=VERY_RARE => "very rare",
        11..=40 => "rare",
        41..=100 => "uncommon",
        101..=300 => "common",
        _ => "very common",
    }
}

/// Every material by group, then name: its group, colour, weight, weapon modifiers (and the dice
/// more a weapon of it rolls), the Protection an armour of it keeps, the Defence a shield of it
/// gives, and worth, all on steel's scale; whether it is precious
/// (shown in names); its chance to withstand each kind of item damage (acid, rust, fire), or
/// "never damaged" for a `proof` one; its
/// `work` (added to the difficulty of working it); its `binding` (what it adds to the
/// enchantments an item of it can bear); and the objects made of it, in part or whole.
pub(super) fn materials(content: &Content) -> String {
    let mut out = String::from(
        "| Material | Group | Colour | Weight | Weapon (to hit / damage) | Armour | Shield | Worth \
         | Precious (shown in names) | Withstands acid | Withstands rust | Withstands fire | \
         Work | Binding | Objects made of it, in part or whole |\n\
         |---|---|---|---:|:---:|---:|---:|---:|:---:|---:|---:|---:|---:|---:|---|\n",
    );
    // A loose gem is made of its gem, never of what its object would be without one.
    let loose = content.gems().map(|g| g.object);
    let all: Vec<_> = (order::objects(content).into_iter())
        .filter(|(id, _)| Some(*id) != loose)
        .collect();
    for (id, m) in by_group(content) {
        let of_it = |o: &&ObjectDef| o.parts.iter().any(|p| p.material == id);
        let objects = all.iter().map(|(_, o)| *o).filter(of_it);
        let mut made: Vec<&str> = (objects.clone())
            .filter(|o| o.corpse.is_none())
            .map(|o| o.plural.as_str())
            .collect();
        if objects.clone().any(|o| o.corpse.is_some()) {
            made.push("corpses");
        }
        if m.gem.is_some() && loose.is_some() {
            made.push("loose gems, and items set with one");
        }
        let made = if made.is_empty() {
            "-".to_string()
        } else {
            made.join(", ")
        };
        let group = m.group.as_deref().unwrap_or("-");
        let colour = (m.color.or(m.gem.as_ref().map(|g| g.color)))
            .map_or("-".to_string(), |c| format!("`{c}`"));
        let weapon = (m.weapon()).map_or("-".to_string(), |(hit, damage)| match m.dice {
            0 => format!("{} / {}", signed(hit), signed(damage)),
            1 => format!("{} / {}, a die more", signed(hit), signed(damage)),
            n => format!("{} / {}, {n} dice more", signed(hit), signed(damage)),
        });
        let armour = (m.protection).map_or("-".to_string(), |p| format!("{p}%"));
        // What cannot be armour is no shield either.
        let shield = (m.protection).map_or("-".to_string(), |_| format!("{}%", m.defence));
        let resists = m.resists();
        let withstands = |chance: u8| match m.proof {
            true => "never damaged".to_string(),
            false => format!("{chance}%"),
        };
        let _ = writeln!(
            out,
            "| {} | {group} | {colour} | {}% | {weapon} | {armour} | {shield} | {} | {} | {} | {} \
             | {} | {} | {} | {made} |",
            capital(&m.name),
            m.weight,
            times(m.value),
            yes(m.precious),
            withstands(resists.acid),
            withstands(resists.rust),
            withstands(resists.fire),
            signed(m.work),
            signed(i32::from(m.binding)),
        );
    }
    out
}

/// How often loot and creatures' gear are made of each material: the weight at level 1 (in
/// words too), how it changes with each level deeper, the least level, and how well it suits
/// each use of item (#1191).
pub(super) fn material_finds(content: &Content) -> String {
    let mut out = String::from(
        "| Material | Loot at depth 1 | Each depth deeper | From depth | Gear at creature level 1 \
         | Each level higher | Weapons | Armour | Jewellery | Other items |\n\
         |---|---|---:|---:|---|---:|---|---|---|---|\n",
    );
    let level = |found: Option<warlock_core::content::Found>| match found {
        Some(f) => (
            format!("{} ({})", f.weight, how_often(f.weight)),
            signed(f.deeper),
            f.level.to_string(),
        ),
        None => ("never".to_string(), "-".to_string(), "-".to_string()),
    };
    for (_, m) in by_group(content) {
        if m.rarity.is_none() && m.gear.is_none() {
            continue;
        }
        let (loot, deeper, from) = level(m.rarity);
        let (gear, higher, _) = level(m.gear);
        let f = m.fit;
        let _ = writeln!(
            out,
            "| {} | {loot} | {deeper} | {from} | {gear} | {higher} | {} | {} | {} | {} |",
            capital(&m.name),
            f.weapon.id(),
            f.armour.id(),
            f.jewellery.id(),
            f.other.id()
        );
    }
    out
}

/// A share in parts per million as a percentage: 90%, 1.2%, 0.05%.
fn per_million(ppm: u32) -> String {
    percent(f64::from(ppm) / 10_000.0)
}

/// The items [`material_odds`] shows: a few of each use.
const SAMPLES: [&str; 8] = [
    "long_sword",
    "spear",
    "dagger",
    "chain_mail",
    "leather_jerkin",
    "wool_cloak",
    "gold_ring",
    "bone_amulet",
];

/// How often a few items are found made of their own material, of another that suits them well,
/// of a poor fit, and of an odd one, in loot at depths 1, 10, and 21 (#1191), from the game's
/// own weights; the three likeliest other materials by name.
pub(super) fn material_odds(content: &Content) -> String {
    let mut out = String::from(
        "| Item | Depth | Its own material | Another that suits it | A poor fit | An odd fit | \
         Likeliest others |\n|---|---:|---:|---:|---:|---:|---|\n",
    );
    for id in SAMPLES {
        let Some(object) = content.object_id(id) else {
            continue;
        };
        let def = content.object(object);
        for depth in [1, 10, 21] {
            let weights = variant::weights(content, def, variant::Source::Loot, depth);
            if weights.is_empty() {
                continue;
            }
            let mut by_fit = [0u32; 4];
            let mut others: Vec<(u32, &str)> = Vec::new();
            for &(m, w) in &weights {
                if m == def.material() {
                    by_fit[0] += w;
                    continue;
                }
                let at = match variant::fit(def, content.material(m)) {
                    Fit::Good => 1,
                    Fit::Poor => 2,
                    Fit::Odd => 3,
                };
                by_fit[at] += w;
                if w > 0 {
                    others.push((w, &content.material(m).name));
                }
            }
            others.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
            let likeliest: Vec<String> = (others.iter().take(3))
                .map(|(w, name)| format!("{name} {}", per_million(*w)))
                .collect();
            let _ = writeln!(
                out,
                "| {} | {depth} | {} | {} | {} | {} | {} |",
                capital(&def.name),
                per_million(by_fit[0]),
                per_million(by_fit[1]),
                per_million(by_fit[2]),
                per_million(by_fit[3]),
                likeliest.join(", ")
            );
        }
    }
    out
}

/// The materials each faction's creatures favour for what they carry and drop (#1191), by
/// faction name: each with its share of the draws for an item that may be made of it.
pub(super) fn material_tribes(content: &Content) -> String {
    let mut out = String::from("| Creatures | Favoured materials |\n|---|---|\n");
    let mut rows: Vec<(String, String)> = (content.factions().defs().iter())
        .filter(|f| !f.gear.is_empty())
        .map(|f| {
            let mut gear: Vec<(u8, &str)> = (f.gear.iter())
                .map(|&(m, p)| (p, content.material(m).name.as_str()))
                .collect();
            gear.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
            let words: Vec<String> = gear.iter().map(|(p, m)| format!("{m} {p}%")).collect();
            (capital(&f.name), words.join(", "))
        })
        .collect();
    rows.sort();
    for (name, gear) in rows {
        let _ = writeln!(out, "| {name} | {gear} |");
    }
    out
}

/// What each item may be made of besides its own material (its construction list), by item
/// class: groups where the list holds every material of a group that the item's main part can
/// be, else the materials one by one; "any material" for every material of a group.
pub(super) fn construction(content: &Content) -> String {
    let mut out = String::from("| Item | Made of | May be made of |\n|---|---|---|\n");
    for (_, o) in order::objects(content) {
        let list = o.made_of(0);
        if list.len() < 2 {
            continue;
        }
        let made = &content.material(o.material()).name;
        let _ = writeln!(
            out,
            "| {} | {made} | {} |",
            capital(&o.name),
            list_words(content, o, list)
        );
    }
    out
}

/// `list` in words for `object`: "any material", or its groups and materials.
fn list_words(content: &Content, object: &ObjectDef, list: &[MaterialId]) -> String {
    let role = object.role();
    let fits = |m: &MaterialDef| m.group.is_some() && role.allows(m);
    let all: Vec<MaterialId> = (0..content.materials().len())
        .map(|i| MaterialId(i as u16))
        .filter(|&id| fits(content.material(id)))
        .collect();
    if role == PartRole::Any && all.iter().all(|m| list.contains(m)) {
        return "any material".into();
    }
    let mut groups: Vec<&str> = (content.materials().iter())
        .filter_map(|m| m.group.as_deref())
        .collect();
    groups.sort_unstable();
    groups.dedup();
    let mut words = Vec::new();
    let mut said = Vec::new();
    for group in groups {
        let of: Vec<MaterialId> = (all.iter().copied())
            .filter(|&id| content.material(id).group.as_deref() == Some(group))
            .collect();
        if of.len() > 1 && of.iter().all(|m| list.contains(m)) {
            words.push(format!("any {group}"));
            said.extend(of);
        }
    }
    let mut single: Vec<&str> = (list.iter())
        .filter(|m| !said.contains(m))
        .map(|&m| content.material(m).name.as_str())
        .collect();
    single.sort_unstable();
    words.extend(single.into_iter().map(str::to_string));
    words.join(", ")
}

/// A long sword and a chain mail shirt in each metal they may be made of: weight, what the metal
/// does in a fight (the sword's to hit and damage modifiers and its dice), and worth.
pub(super) fn variants(content: &Content) -> String {
    let mut out = String::from(
        "| Metal | Long sword | To hit, damage | Dice | Worth | Chain mail shirt | Protection \
         | Worth |\n\
         |---|---:|:---:|---:|---:|---:|---:|---:|\n",
    );
    let (Some(sword), Some(mail)) = (
        content.object_id("long_sword"),
        content.object_id("chain_mail"),
    ) else {
        return out;
    };
    let made = |object, material: MaterialId| {
        let def = content.object(object);
        if material == def.material() {
            return Stack::one(object);
        }
        Stack::one(object).with_parts(vec![ItemPart::material(content, None, material)])
    };
    let metals =
        (by_group(content).into_iter()).filter(|(_, m)| m.group.as_deref() == Some("metal"));
    for (id, m) in metals {
        let (s, c) = (made(sword, id), made(mail, id));
        let (s_def, c_def) = (content.object(sword), content.object(mail));
        if !s_def.made_of(0).contains(&id) || !c_def.made_of(0).contains(&id) {
            continue;
        }
        let (hit, damage) = variant::weapon(s_def, &s.parts(content), content);
        let dice = s
            .attack(content)
            .map_or("-".to_string(), |a| a.damage.to_string());
        let _ = writeln!(
            out,
            "| {} | {} | {} / {} | {dice} | {} gold | {} | {} | {} gold |",
            capital(&m.name),
            format_weight(s.weight(content)),
            signed(hit),
            signed(damage),
            s.value(content),
            format_weight(c.weight(content)),
            c.base_protection(content),
            c.value(content),
        );
    }
    out
}

/// The common weapons, armour, helmets, and shields as made and in each material that rolls
/// more dice (`dice`; adamantium): a weapon's dice and damage modifier, an armour's or helmet's
/// Protection, a shield's Defence.
pub(super) fn fighting_metals(content: &Content) -> String {
    const PIECES: [&str; 14] = [
        "dagger",
        "short_sword",
        "long_sword",
        "great_sword",
        "battle_axe",
        "mace",
        "chain_mail",
        "scale_mail",
        "banded_mail",
        "plate_armour",
        "iron_helm",
        "great_helm",
        "small_shield",
        "tower_shield",
    ];
    let metals: Vec<_> = (by_group(content).into_iter())
        .filter(|(_, m)| m.dice > 0)
        .collect();
    let mut out = String::from("| Item | As made |");
    for (_, m) in &metals {
        let _ = write!(out, " In {} |", m.name);
    }
    out.push_str("\n|---|---:|");
    out.push_str(&"---:|".repeat(metals.len()));
    out.push('\n');
    // What one of `stack` gives in a fight: its dice and damage, Protection, or Defence.
    let numbers = |stack: &Stack| {
        let def = content.object(stack.object);
        let parts = stack.parts(content);
        if let Some(attack) = stack.attack(content) {
            let (_, damage) = variant::weapon(def, &parts, content);
            return attack.damage.plus(damage).to_string();
        }
        if def.shield > 0 {
            let shield = def.shield + variant::shield(def, &parts, content);
            return format!("Defence {shield}");
        }
        format!("Protection {}", stack.base_protection(content))
    };
    for id in PIECES {
        let Some(object) = content.object_id(id) else {
            continue;
        };
        let def = content.object(object);
        let plain = Stack::one(object);
        let _ = write!(out, "| {} | {} |", capital(&def.name), numbers(&plain));
        for (m, _) in &metals {
            let cell = match def.made_of(0).contains(m) {
                true => {
                    let made = plain.clone();
                    numbers(&made.with_parts(vec![ItemPart::material(content, None, *m)]))
                }
                false => "-".to_string(),
            };
            let _ = write!(out, " {cell} |");
        }
        out.push('\n');
    }
    out
}

/// What a piece of each material with a group is called, how many pieces' worth it is (its
/// group's `unit`), what it weighs and is worth, and the skill and stat that work it
/// (`objects/piece.toml`, `stock.toml`: what a broken item falls to, the material stock), metals
/// first, then by group and name.
pub(super) fn pieces(content: &Content) -> String {
    let mut out = String::from(
        "| Material | Group | A piece | Unit | Weight | Worth | Worked by |\n\
         |---|---|---|---:|---:|---:|---|\n",
    );
    for (id, m) in by_group(content) {
        let (Some(group), Some(piece)) = (&m.group, warlock_core::piece::make(content, id, 1))
        else {
            continue;
        };
        let worth = match piece.value(content) {
            0 => "nothing".to_string(),
            n => format!("{n} gold"),
        };
        let worked = (content.stock().profession_of(content, id)).map_or("-".to_string(), |p| {
            format!("{} ({})", content.skill(p.skill).name, p.stat.name())
        });
        let _ = writeln!(
            out,
            "| {} | {group} | {} | {} | {} | {worth} | {worked} |",
            capital(&m.name),
            piece.names(content).0,
            content.stock().unit_of(content, id),
            format_weight(piece.weight(content)),
        );
    }
    out
}

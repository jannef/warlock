//! Spells (spec §8 "Spells"): the shapes' and resolutions' cost multipliers, the classes'
//! casting percents, the spell list by spell level, the classes' guaranteed spells, the armour
//! that makes casting dearer, and the wands.

use super::text::{Term, anchored};
use std::fmt::Write as _;
use warlock_core::content::{
    Content, Resolution, RodDoes, RodShape, Shape, SpellData, SpellDice, SpellPurpose,
    TalentEffect, Weights,
};

/// What each shape multiplies the base cost by; a burst by radius.
pub(super) fn shapes(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut out = String::from("| Shape | Area | Cost |\n|---|:---:|:---:|\n");
    for &shape in Shape::ALL {
        let area = if shape.is_area() { "yes" } else { "no" };
        if shape == Shape::Burst {
            for radius in 1..=rules.max_radius() {
                let cost = rules.shape_cost(shape, radius);
                let _ = writeln!(
                    out,
                    "| burst, radius {radius} | {area} | ×{} |",
                    times(cost)
                );
            }
        } else {
            let cost = rules.shape_cost(shape, 0);
            let _ = writeln!(out, "| {} | {area} | ×{} |", shape.id(), times(cost));
        }
    }
    out
}

/// Each resolution's cost.
pub(super) fn resolutions(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut out = String::from("| Resolution | Cost |\n|---|:---:|\n");
    for &resolution in Resolution::ALL {
        let ratio = rules.ratio(resolution);
        let cost = if resolution == Resolution::Area {
            "the shape's".to_string()
        } else {
            format!("×{}", times(ratio.cost))
        };
        let _ = writeln!(out, "| {} | {cost} |", resolution.id());
    }
    out
}

/// Each class's casting percent, most first, then by name.
pub(super) fn casting(content: &Content) -> String {
    let mut classes: Vec<(&str, u8)> = (content.classes().iter())
        .map(|c| (c.name.as_str(), c.casting))
        .collect();
    classes.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let mut out = String::from("| Class | Casting |\n|---|:---:|\n");
    for (name, casting) in classes {
        let _ = writeln!(out, "| {name} | {casting} % |");
    }
    out
}

/// Every spell, by spell level and then name: its school, shape and resolution, reach (as cast,
/// and how far its stats take it), what it does, what it grows with, its mana (no armour, no
/// talents; whoever casts it), and its time in casts.
pub(super) fn list(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut spells: Vec<&SpellData> = content.spells().iter().collect();
    spells.sort_by(|a, b| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
    let mut out = String::from(
        "| Spell | Level | School | Shape | Resolves | Reach | What it does | Grows with | Mana | Casts \
         |\n|---|:---:|---|---|---|---|---|---|:---:|:---:|\n",
    );
    for spell in spells {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            anchored(Term::Spell, &spell.id, &spell.name),
            spell.level,
            spell.school.id(),
            spell.shape.id(),
            spell.resolution.id(),
            reach(spell),
            does(content, spell),
            grows_with(spell),
            rules.mana(spell, 0),
            spell.casts,
        );
    }
    out
}

/// Each class's guaranteed spells (its level table's spell abilities), by class name: each
/// spell with its spell level and the level it comes at, lowest first; every other spell is learned
/// from scrolls.
pub(super) fn class_spells(content: &Content) -> String {
    let book = content.talents();
    let mut classes: Vec<_> = content.classes().iter().collect();
    classes.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::from(
        "| Class | Casting | Guaranteed spells (spell level, from level) |\n|---|:---:|---|\n",
    );
    for class in classes {
        let mut given: Vec<(u8, &str, u8)> = Vec::new();
        for (level, talent) in &class.abilities {
            let Some(id) = book.id(talent) else {
                continue;
            };
            for effect in book.get(id).effects_at(1) {
                let TalentEffect::Spell { spell } = effect else {
                    continue;
                };
                if let Some(s) = content.spell_id(&spell).map(|s| content.spell(s)) {
                    given.push((*level, s.name.as_str(), s.level));
                }
            }
        }
        let spells = if given.is_empty() {
            "none".to_string()
        } else {
            (given.iter())
                .map(|(from, name, level)| format!("{name} (level {level}, from {from})"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = writeln!(out, "| {} | {} % | {spells} |", class.name, class.casting);
    }
    out
}

/// What `spell` grows with, from its `[spell.scales]`: each dimension the stats change, its
/// stats with their weights in percent of the modifier ("dice: Intelligence 10 %, Willpower 5 %
/// of its most dice a point"), the dimensions it cannot have left out.
fn grows_with(spell: &SpellData) -> String {
    let named = |w: &Weights, unit: &str| {
        if w.is_empty() {
            return None;
        }
        let stats: Vec<String> = (w.iter())
            .map(|(stat, weight)| format!("{} {weight} %", stat.name()))
            .collect();
        Some(format!("{} {unit}", stats.join(", ")))
    };
    let sc = &spell.scales;
    let rolls = spell.damage.is_some() || spell.effect.heal.is_some();
    let attack = spell.kind == SpellPurpose::Damage
        && matches!(spell.resolution, Resolution::Attack | Resolution::Touch);
    let reaches = !matches!(spell.shape, Shape::Touch | Shape::Myself);
    let lasts = matches!(spell.kind, SpellPurpose::Buff | SpellPurpose::Hex);
    let parts = [
        (rolls, "dice", named(&sc.damage, "of its most dice a point")),
        (attack, "to-hit", named(&sc.to_hit, "of the modifier")),
        (reaches, "reach", named(&sc.reach, "of a cell a point")),
        (lasts, "degrees", named(&sc.duration, "of a degree a point")),
    ];
    let text: Vec<String> = (parts.into_iter())
        .filter_map(|(has, what, by)| {
            has.then_some(by)
                .flatten()
                .map(|by| format!("{what}: {by}"))
        })
        .collect();
    if text.is_empty() {
        "—".into()
    } else {
        text.join("; ")
    }
}

/// How far `spell` reaches, and how far its stats may take it: "6 (to 8)", a burst's "range 10,
/// radius 2", "—" on oneself.
fn reach(spell: &SpellData) -> String {
    let grows = |base: Option<u8>, cap: Option<u8>| match (base, cap) {
        (Some(b), Some(c)) if c > b => format!("{b} (to {c})"),
        (Some(b), _) => b.to_string(),
        _ => String::new(),
    };
    match spell.shape {
        Shape::Myself => "—".into(),
        Shape::Touch => "next to you".into(),
        Shape::Burst => format!(
            "range {}, radius {}",
            grows(spell.range, spell.max_range),
            grows(spell.radius, spell.max_radius)
        ),
        Shape::Cone | Shape::Line => grows(spell.length, spell.max_length),
        Shape::Ray | Shape::Bolt => grows(spell.range, spell.max_range),
    }
}

/// What `spell` does, as its data says: its dice at the caster level of its spell level (and
/// what more caster levels bring), the status it leaves, what it does to ice and walls.
pub(super) fn does(content: &Content, spell: &SpellData) -> String {
    let status = |s| content.status(s).name.clone();
    let at = spell.grows_from();
    let dice = |d: SpellDice| {
        let now = d.at(i32::from(at), at);
        if d.max_dice > d.count && d.per_levels > 0 {
            format!(
                "{now}, a die more per {} caster levels above {at}, to {}d{}",
                d.per_levels, d.max_dice, d.sides
            )
        } else {
            now.to_string()
        }
    };
    let mut text = match spell.kind {
        SpellPurpose::Damage => {
            let rolled = spell.damage.map_or_else(String::new, dice);
            let kind = (spell.damage_type)
                .map_or_else(String::new, |k| format!(" {}", content.damage_type(k).name));
            format!("{rolled}{kind} damage")
        }
        SpellPurpose::Heal => {
            let healed = spell.effect.heal.map_or_else(String::new, dice);
            format!("{healed} hit points back")
        }
        SpellPurpose::Buff | SpellPurpose::Hex => match spell.effect.status {
            Some((s, n)) => {
                let who = if spell.kind == SpellPurpose::Buff {
                    "you are"
                } else if spell.shape == Shape::Burst {
                    "everyone it reaches but you, friend or foe, suffers"
                } else {
                    "a hostile you see suffers"
                };
                format!("{who} {} ({n})", status(s))
            }
            None => String::new(),
        },
        SpellPurpose::Cleanse => {
            let mut ends: Vec<String> = spell.effect.cures.iter().map(|&s| status(s)).collect();
            if spell.effect.diseases {
                ends.push("every disease".into());
            }
            format!("ends {}", ends.join(" and "))
        }
        SpellPurpose::Utility => spell.description.clone(),
    };
    if spell.kind == SpellPurpose::Damage
        && let Some((s, n)) = spell.effect.status
    {
        let _ = write!(text, "; everyone it harms suffers {} ({n})", status(s));
    }
    text.push_str(&super::traps::ice_text(spell.look.ice));
    if spell.look.blast > 0 {
        text.push_str("; it breaks walls");
    }
    if spell.open_sky {
        text.push_str("; only under the open sky");
    }
    text
}

/// The armour and shields that make spells dearer while worn, most first, then by name.
pub(super) fn armour(content: &Content) -> String {
    let mut worn: Vec<(&str, u16)> = (content.object_defs().iter())
        .filter(|o| o.spell_cost_percent > 0)
        .map(|o| (o.name.as_str(), o.spell_cost_percent))
        .collect();
    worn.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let mut out = String::from("| Worn | Spells cost more |\n|---|:---:|\n");
    for (name, percent) in worn {
        let _ = writeln!(out, "| {name} | {percent} % |");
    }
    out
}

/// The wands (spec §8 "Magic items": "Wands"): each one's spell, its spell level, charges, the
/// caster level it casts at, and where loot finds it; by spell level, then name.
pub(super) fn wands(content: &Content) -> String {
    let mut wands: Vec<_> = (content.object_defs().iter())
        .filter_map(|o| Some((o, o.wand?)))
        .collect();
    wands.sort_by(|(a, x), (b, y)| {
        let level = |w: &warlock_core::content::Wand| content.spell(w.spell).level;
        level(x).cmp(&level(y)).then(a.name.cmp(&b.name))
    });
    let mut out = String::from(
        "| Wand | Spell | Spell level | Charges | Caster level | Item level | Frequency | Value |\n\
         |---|---|:---:|:---:|:---:|:---:|:---:|:---:|\n",
    );
    for (object, wand) in wands {
        let spell = content.spell(wand.spell);
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} gold |",
            object.name,
            spell.name,
            spell.level,
            wand.charges,
            wand.caster_level,
            object.level,
            object.frequency,
            object.value
        );
    }
    out
}

/// The rods (spec §8 "Magic items": "Rods"): each one's effect, its shape and reach by the
/// user's caster level, what it does exactly, its mana cost, and where loot finds it; by item
/// level, then name.
pub(super) fn rods(content: &Content) -> String {
    let mut rods: Vec<_> = (content.object_defs().iter())
        .filter_map(|o| Some((o, o.rod?)))
        .collect();
    rods.sort_by(|(a, _), (b, _)| a.level.cmp(&b.level).then(a.name.cmp(&b.name)));
    let mut out = String::from(
        "| Rod | Effect | Shape | Reach | What it does | Mana | Item level | Frequency | Value |\n\
         |---|---|---|---|---|:---:|:---:|:---:|:---:|\n",
    );
    for (object, rod) in rods {
        let effect = content.rod_effect(rod.effect);
        let shape = match effect.shape {
            RodShape::Burst => format!("burst, radius {}", effect.radius),
            other => other.id().to_string(),
        };
        let reach = match effect.per_levels {
            0 => format!("{}", effect.range),
            per => format!(
                "{} + 1 per {per} caster levels, at most {}",
                effect.range, effect.max_range
            ),
        };
        let does = match effect.does {
            RodDoes::Dig { power } => {
                format!("breaks every wall on its line of hardness below {power}")
            }
            RodDoes::Teleport => {
                "moves the first creature it hits to a random free cell of the level".to_string()
            }
            RodDoes::Status {
                status,
                severity,
                up_to_caster_level,
            } => {
                let who = if up_to_caster_level {
                    "every creature of a level up to the user's caster level"
                } else {
                    "everyone"
                };
                format!("{} {severity} on {who} in it", content.status(status).name)
            }
        };
        let _ = writeln!(
            out,
            "| {} | {} | {shape} | {reach} | {does} | {} | {} | {} | {} gold |",
            object.name, effect.name, rod.mana, object.level, object.frequency, object.value
        );
    }
    out
}

/// A percent as a multiplier: `160` is `1.6`, `100` is `1`.
fn times(percent: u16) -> String {
    let whole = percent / 100;
    match percent % 100 {
        0 => whole.to_string(),
        n if n % 10 == 0 => format!("{whole}.{}", n / 10),
        n => format!("{whole}.{n:02}"),
    }
}

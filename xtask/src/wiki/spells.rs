//! Spells (spec §8 "Spells"): the tier table (comfortable caster level, base cost, damage
//! ceiling), the shapes' and resolutions' multipliers, the classes' casting percents, the spell
//! list, the classes' guaranteed spells, the armour that makes casting dearer, and the wands.

use std::fmt::Write as _;
use warlock_core::content::{
    Content, Resolution, Shape, SpellData, SpellDice, SpellPurpose, TIERS, TalentEffect,
};

/// Each tier's comfortable caster level, base mana cost, and damage ceiling, tier 0 first.
pub(super) fn tiers(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut out = String::from(
        "| Tier | Comfortable caster level | Base mana cost | Damage ceiling |\n\
         |:---:|:---:|:---:|:---:|\n",
    );
    for tier in 0..TIERS {
        let name = if tier == 0 {
            "0 (cantrip)".to_string()
        } else {
            tier.to_string()
        };
        let _ = writeln!(
            out,
            "| {name} | {} | {} | {} |",
            rules.comfortable[tier], rules.cost[tier], rules.ceiling[tier]
        );
    }
    let _ = writeln!(
        out,
        "\nEach tier of overreach adds {} % to a spell's cost.",
        rules.overreach_cost
    );
    out
}

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

/// Each resolution's damage cap (of the tier's ceiling) and cost.
pub(super) fn resolutions(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut out =
        String::from("| Resolution | Most damage, of the ceiling | Cost |\n|---|:---:|:---:|\n");
    for &resolution in Resolution::ALL {
        let ratio = rules.ratio(resolution);
        let cost = if resolution == Resolution::Area {
            "the shape's".to_string()
        } else {
            format!("×{}", times(ratio.cost))
        };
        let _ = writeln!(out, "| {} | {} % | {cost} |", resolution.id(), ratio.damage);
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

/// Every spell, by tier and then name: its school, shape and resolution, reach (as cast, and
/// how far Willpower takes it), what it does, its mana at its tier's comfortable caster level,
/// and its time in casts.
pub(super) fn list(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut spells: Vec<&SpellData> = content.spells().iter().collect();
    spells.sort_by(|a, b| a.tier.cmp(&b.tier).then(a.name.cmp(&b.name)));
    let mut out = String::from(
        "| Spell | Tier | School | Shape | Resolves | Reach | What it does | Mana | Casts |\n\
         |---|:---:|---|---|---|---|---|:---:|:---:|\n",
    );
    for spell in spells {
        let at = i32::from(rules.comfortable[usize::from(spell.tier)]);
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            spell.name,
            spell.tier,
            spell.school.id(),
            spell.shape.id(),
            spell.resolution.id(),
            reach(spell),
            does(content, spell),
            rules.mana(spell, at, 0),
            spell.casts,
        );
    }
    out
}

/// Each class's guaranteed spells (its level table's spell abilities), by class name: each
/// spell with its tier and the level it comes at, lowest first; every other spell is learned
/// from scrolls.
pub(super) fn class_spells(content: &Content) -> String {
    let book = content.talents();
    let mut classes: Vec<_> = content.classes().iter().collect();
    classes.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::from(
        "| Class | Casting | Guaranteed spells (tier, from level) |\n|---|:---:|---|\n",
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
                    given.push((*level, s.name.as_str(), s.tier));
                }
            }
        }
        let spells = if given.is_empty() {
            "none".to_string()
        } else {
            (given.iter())
                .map(|(level, name, tier)| format!("{name} ({tier}, {level})"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = writeln!(out, "| {} | {} % | {spells} |", class.name, class.casting);
    }
    out
}

/// How far `spell` reaches, and how far Willpower may take it: "6 (to 8)", a burst's "range 10,
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

/// What `spell` does, as its data says: its dice at its tier's comfortable caster level (and
/// what more caster levels bring), the status it leaves, what it does to ice and walls, its
/// cap.
pub(super) fn does(content: &Content, spell: &SpellData) -> String {
    let status = |s| content.status(s).name.clone();
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let at = rules.comfortable[usize::from(spell.tier)];
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
                format!(
                    "{who} {} ({n}, a degree more per {} of Presence's modifier)",
                    status(s),
                    rules.presence_step
                )
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
    if matches!(spell.kind, SpellPurpose::Damage | SpellPurpose::Heal) {
        let _ = write!(text, " (never more than {})", rules.cap_of(spell));
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

/// The wands (spec §8 "Magic items": "Wands"): each one's spell, tier, charges, the caster level
/// it casts at, and where loot finds it; by tier, then name.
pub(super) fn wands(content: &Content) -> String {
    let mut wands: Vec<_> = (content.object_defs().iter())
        .filter_map(|o| Some((o, o.wand?)))
        .collect();
    wands.sort_by(|(a, x), (b, y)| {
        let tier = |w: &warlock_core::content::Wand| content.spell(w.spell).tier;
        tier(x).cmp(&tier(y)).then(a.name.cmp(&b.name))
    });
    let mut out = String::from(
        "| Wand | Spell | Tier | Charges | Caster level | Item level | Frequency | Value |\n\
         |---|---|:---:|:---:|:---:|:---:|:---:|:---:|\n",
    );
    for (object, wand) in wands {
        let spell = content.spell(wand.spell);
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} gold |",
            object.name,
            spell.name,
            spell.tier,
            wand.charges,
            wand.caster_level,
            object.level,
            object.frequency,
            object.value
        );
    }
    out
}

/// The charges a wand is made with by its spell's tier, unless it says otherwise.
pub(super) fn wand_charges(content: &Content) -> String {
    let Some(rules) = content.spell_rules() else {
        return String::new();
    };
    let mut out = String::from("| Tier | Charges |\n|:---:|:---:|\n");
    for tier in 0..TIERS {
        let _ = writeln!(out, "| {tier} | {} |", rules.wand_charges[tier]);
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

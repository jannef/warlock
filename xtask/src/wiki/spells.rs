//! Spells (spec §8 "Spells"): the tier table (comfortable caster level, base cost, damage
//! ceiling), the shapes' and resolutions' multipliers, and the classes' casting percents.

use std::fmt::Write as _;
use warlock_core::content::{Content, Resolution, Shape, TIERS};

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

/// A percent as a multiplier: `160` is `1.6`, `100` is `1`.
fn times(percent: u16) -> String {
    let whole = percent / 100;
    match percent % 100 {
        0 => whole.to_string(),
        n if n % 10 == 0 => format!("{whole}.{}", n / 10),
        n => format!("{whole}.{n:02}"),
    }
}

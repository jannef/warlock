//! Vision profiles and light presets, named as the player sees them.

use super::order;
use super::text::{article, capital, yes};
use std::fmt::Write as _;
use warlock_core::content::{AffixKind, Content, LightId};

pub(super) fn vision(content: &Content) -> String {
    let mut out = String::from(
        "| Vision | Needs a cell lit to at least | Sees in the dark within | Sees UV light |\n\
         |---|---:|---:|:---:|\n",
    );
    for v in content.vision_profiles() {
        // Full light is 255 per channel.
        let percent = super::combat::percent(u32::from(v.threshold), 255);
        let dark = if v.darkvision > 0 {
            format!("{} cells", v.darkvision)
        } else {
            "-".into()
        };
        let _ = writeln!(
            out,
            "| {} | {} of 255 ({percent} of full light) | {dark} | {} |",
            v.name,
            v.threshold,
            yes(v.sees_uv)
        );
    }
    out
}

pub(super) fn lights(content: &Content) -> String {
    let mut out = String::from("| Light | Reaches | Kind |\n|---|---:|---|\n");
    for (i, l) in content.light_presets().iter().enumerate() {
        let kind = if l.color.rgb == [0, 0, 0] {
            "UV only: invisible without UV sight"
        } else if l.color.uv > 0 {
            "visible and UV"
        } else {
            "visible"
        };
        let _ = writeln!(
            out,
            "| {} | {} cells | {kind} |",
            light_name(content, LightId(i as u16)),
            l.radius
        );
    }
    out
}

/// What a player calls a light: the terrain that gives it ("Wall torch"), what carries it ("Torch
/// carried by a kobold"), their own lantern, a crystal by its color ("Light crystal (red)" for a
/// `crystal_red` the levels place on crystal pedestals), else its id in words ("Lamp").
fn light_name(content: &Content, id: LightId) -> String {
    let preset = &content.light_presets()[usize::from(id.0)];
    let words = |id: &str| id.replace('_', " ").replace("uv", "UV");
    let terrain: Vec<&str> = (content.terrain_defs().iter())
        .filter(|t| t.light == Some(id))
        .map(|t| t.name.as_str())
        .collect();
    if !terrain.is_empty() {
        return capital(&terrain.join(", "));
    }
    let carriers: Vec<String> = (order::creatures(content).into_iter())
        .map(|(_, c)| c)
        .filter(|c| c.light == Some(id))
        .map(|c| format!("{} {}", article(&c.name), c.name))
        .collect();
    if !carriers.is_empty() {
        return format!(
            "{} carried by {}",
            capital(&words(&preset.id)),
            carriers.join(" or ")
        );
    }
    if content
        .scenarios()
        .any(|s| s.lantern.as_deref() == Some(preset.id.as_str()))
    {
        return "Your lantern".into();
    }
    let affixes: Vec<&str> = (order::affixes(content, AffixKind::Prefix).into_iter())
        .chain(order::affixes(content, AffixKind::Suffix))
        .filter(|a| a.tiers.iter().any(|t| t.light == Some(id)))
        .map(|a| a.name.as_str())
        .collect();
    if !affixes.is_empty() {
        return format!(
            "{} magic item ({})",
            capital(glow_word(preset.radius)),
            affixes.join(", ")
        );
    }
    let pedestal = (content.terrain_defs().iter())
        .filter(|t| !t.walkable && t.light.is_none())
        .find_map(|t| Some((t, preset.id.strip_prefix(&format!("{}_", t.id))?)));
    match pedestal {
        Some((t, kind)) => format!("{} ({})", capital(&t.name), words(kind)),
        None => capital(&words(&preset.id)),
    }
}

/// How a light of `radius` glows, in words: `faintly glowing`, `glowing`, `brightly glowing`.
pub(super) fn glow_word(radius: i32) -> &'static str {
    match radius {
        ..=2 => "faintly glowing",
        3 => "glowing",
        _ => "brightly glowing",
    }
}

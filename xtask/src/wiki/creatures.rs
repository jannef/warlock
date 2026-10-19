//! Creatures and factions (spec §7 "Factions"): never their hidden numbers.

use super::order;
use super::text::{capital, glyph, yes};
use std::fmt::Write as _;
use warlock_core::content::{Combat, Content, FactionId, Stance};

/// Speed, eyes, and sight radius are deliberately left out: they are hidden from the player (spec
/// §8 "Speed"); how a creature stands toward the player and how it attacks are seen in play. By
/// name.
pub(super) fn creatures(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Glyph | Faction | Toward you | Attacks | Opens doors | Carries a light | \
         Drops magic |\n|---|:---:|---|---|---|:---:|:---:|:---:|\n",
    );
    let player = content.creature_id("player");
    for (id, c) in order::creatures(content) {
        if c.id == "player" {
            continue;
        }
        let faction = c.faction.map_or_else(
            || "none".into(),
            |f| capital(&content.factions().def(f).name),
        );
        let stance = match player.map(|p| content.stance(p, id)) {
            Some(Stance::Hostile) => "hostile",
            _ => "neutral",
        };
        let fights = c.faction.is_some_and(|f| content.factions().has_enemies(f));
        let mut attacks = match c.combat {
            _ if !fights => "never",
            Combat::Melee => "up close",
            Combat::Ranged => "shoots from afar",
            Combat::Caster => "casts spells from afar",
        }
        .to_string();
        if let (true, Some(cost)) = (fights, c.attack_cost) {
            let _ = write!(attacks, " (cost {cost})");
        }
        // How often it drops a magic or rare item where it dies, in words.
        let drops = match c.loot {
            0 => "-",
            1..=29 => "now and then",
            30..=49 => "sometimes",
            _ => "often",
        };
        let _ = writeln!(
            out,
            "| {} | {} | {faction} | {stance} | {attacks} | {} | {} | {drops} |",
            capital(&c.name),
            glyph(c.glyph),
            yes(c.opens_doors),
            yes(c.light.is_some())
        );
    }
    out
}

/// Who fights whom: each faction and the factions hostile to it (relations are symmetric).
pub(super) fn factions(content: &Content) -> String {
    let factions = content.factions();
    let mut out = String::from("| Faction | Hostile to |\n|---|---|\n");
    for (i, f) in factions.defs().iter().enumerate() {
        let id = FactionId(i as u16);
        let enemies: Vec<String> = (factions.defs().iter().enumerate())
            .filter(|&(j, _)| factions.stance(id, FactionId(j as u16)) == Stance::Hostile)
            .map(|(_, e)| e.name.clone())
            .collect();
        let enemies = if enemies.is_empty() {
            "nobody".to_string()
        } else {
            enemies.join(", ")
        };
        let _ = writeln!(out, "| {} | {enemies} |", capital(&f.name));
    }
    out
}

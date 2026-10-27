//! Animated objects (spec §8 "Animated objects"): the tiers, the numbers by level, the household
//! bodies, and what a body's material gives, stated exactly from `data/animated.toml` and
//! `data/animated/`, read through the generated creature types themselves.

use super::creatures::on_hit;
use super::text::{capital, glyph, signed};
use std::fmt::Write as _;
use warlock_core::content::{Content, CreatureDef, Numbers};
use warlock_core::stats::Stat;

/// The tiers: their levels, what they add to the base stats, their hit point factor and dice
/// steps, size and footsteps; then what every animated type is, and the drop rule.
pub(super) fn animated_tiers(content: &Content) -> String {
    let animated = content.animated();
    let mut out = String::from(
        "| Tier | Levels | Stats | Hit points (× 2.5 × level) | Dice steps | Size | Footsteps | \
         Death line |\n|---|---|---|---|---|---|---|---|\n",
    );
    for tier in &animated.tiers {
        let stats: Vec<String> = (Stat::ALL.iter())
            .filter_map(|s| {
                tier.stats
                    .get(s)
                    .map(|v| format!("{} {}", s.abbrev(), signed(*v)))
            })
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} to {} | {} | {} | {} | {} | {} | \"{}\" |",
            tier.name,
            tier.levels[0],
            tier.levels[1],
            if stats.is_empty() {
                "-".into()
            } else {
                stats.join(", ")
            },
            tier.hp_factor,
            signed(tier.dice_steps),
            capital(&tier.size),
            tier.footsteps,
            tier.dies
        );
    }
    let common = &animated.creature;
    let immune: Vec<String> = (common.immune.iter())
        .map(|id| (content.status_id(id)).map_or(id.clone(), |s| content.status(s).name.clone()))
        .collect();
    let _ = write!(
        out,
        "\nEvery animated type: faction **{}**, tags {}, {} eyes (sight {} cells), hearing {}, \
         immune to {}; no corpse and no blood. What it is made of falls whole with **{} %**, \
         else nothing of it.\n",
        content
            .factions()
            .id(&common.faction)
            .map_or(common.faction.clone(), |f| capital(
                &content.factions().def(f).name
            )),
        common
            .tags
            .iter()
            .map(|t| format!("`{t}`"))
            .collect::<Vec<_>>()
            .join(", "),
        content
            .vision(&common.vision)
            .map_or(common.vision.clone(), |v| v.name.to_lowercase()),
        common.sight,
        common.hearing,
        immune.join(", ").to_lowercase(),
        animated.drops.intact,
    );
    out
}

/// The numbers of every tier at each of its levels: stats, hit points beyond Toughness and in
/// all, damage dice (a family that wields: none of its own), a family's speed and natural
/// Defence (a household thing's speed is its body's), and experience.
pub(super) fn animated_numbers(content: &Content) -> String {
    let mut out = String::from(
        "| Tier | Level | St | Dx | To | In | Wi | Pr | Hit points | Max | Dice | Speed | Defence \
         | XP |\n|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---:|---:|---:|\n",
    );
    let guideline = content.experience().guideline();
    for tier in &content.animated().tiers {
        for level in tier.levels[0]..=tier.levels[1] {
            let n = Numbers::at(level, tier);
            let score = |s: Stat| n.stats.get(&s).copied().unwrap_or(0);
            let toughness = u16::from(score(Stat::Toughness));
            let member = (tier.is_family())
                .then(|| content.creature_id(&format!("{}_{level}", tier.family)))
                .flatten()
                .map(|c| content.creature(c));
            let dice = match member {
                Some(def) if def.attack.is_none() => "wields".to_string(),
                _ => n.dice.to_string(),
            };
            let speed = member.map_or("body's".to_string(), |d| d.speed.to_string());
            let defence = member
                .and_then(|d| d.natural.as_ref())
                .map_or("-".to_string(), |h| h.defence.to_string());
            let _ = writeln!(
                out,
                "| {} | {level} | {} | {} | {} | {} | {} | {} | {} | {} | {dice} | {speed} | \
                 {defence} | {} |",
                tier.name,
                score(Stat::Strength),
                score(Stat::Dexterity),
                score(Stat::Toughness),
                score(Stat::Intelligence),
                score(Stat::Willpower),
                score(Stat::Presence),
                n.base_hp,
                toughness + n.base_hp,
                guideline.xp(level)
            );
        }
    }
    out
}

/// The families (blades): what each is made of, slot by slot, how it shows, what it resists,
/// and by level the chances that a piece is magic or rare and its rank in its weapon's skill.
pub(super) fn animated_families(content: &Content) -> String {
    let mut out = String::from(
        "| Family | Levels | Rarity | Made of | Shows as | Resists | Footsteps | Dies |\n\
         |---|---|---|---|---|---|---:|---|\n",
    );
    let animated = content.animated();
    let families: Vec<(usize, &warlock_core::content::animated::Tier)> = (animated.tiers.iter())
        .enumerate()
        .filter(|(_, t)| t.is_family())
        .collect();
    for &(t, tier) in &families {
        let made: Vec<String> = (animated.lines[t].iter())
            .map(|line| {
                let classes: Vec<String> = (line.classes.iter())
                    .map(|&c| content.item_class(c).name.to_lowercase())
                    .collect();
                format!(
                    "{}: {} ({} %)",
                    content.slot(line.slot).name,
                    classes.join(" or "),
                    line.chance
                )
            })
            .collect();
        let shows = match (&tier.shown_by, tier.item_glyph) {
            (Some(slot), true) => {
                format!("its {} piece's glyph and colour", slot.replace('_', " "))
            }
            (Some(slot), false) => format!(
                "{} in its {} piece's colour",
                glyph(tier.glyph.unwrap_or('?')),
                slot.replace('_', " ")
            ),
            (None, _) => glyph(tier.glyph.unwrap_or('?')),
        };
        let resists: Vec<String> = (tier.resist.iter())
            .map(|(k, v)| format!("{k} {}", signed(*v)))
            .collect();
        let rarity = tier.rarity.as_deref().unwrap_or("common");
        let _ = writeln!(
            out,
            "| {} | {} to {} | {} | {} | {shows} | {} | {} | \"{}\" |",
            capital(tier.creature.as_deref().unwrap_or_default()),
            tier.levels[0],
            tier.levels[1],
            capital(&rarity.replace('_', " ")),
            made.join("; "),
            if resists.is_empty() {
                "-".into()
            } else {
                resists.join(", ")
            },
            tier.footsteps,
            tier.dies
        );
    }
    out.push_str(
        "\n| Family | Level | At least magic | Rare | Rank in its weapon's skill |\n\
         |---|---:|---:|---:|---:|\n",
    );
    for (_, tier) in families {
        let at = |ramp: Option<warlock_core::content::Ramp>, level| ramp.map_or(0, |r| r.at(level));
        for level in tier.levels[0]..=tier.levels[1] {
            let _ = writeln!(
                out,
                "| {} | {level} | {} % | {} % | {} |",
                capital(tier.creature.as_deref().unwrap_or_default()),
                at(tier.magic, level).max(at(tier.rare, level)),
                at(tier.rare, level),
                at(tier.weapon_rank, level)
            );
        }
    }
    out
}

/// The household bodies, by level then name: what each is made of, how it moves and strikes,
/// and what its base material gives it.
pub(super) fn animated_bodies(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Glyph | Level | Rarity | Made of | Speed | Attack | Protection | \
         Resists | Dies |\n|---|:---:|---:|---|---|---:|---|---:|---|---|\n",
    );
    let mut bodies: Vec<(&CreatureDef, &warlock_core::content::Body)> = (content.creature_defs())
        .iter()
        .filter_map(|d| {
            content
                .animated()
                .body_of(d.animated.as_ref())
                .map(|b| (d, b))
        })
        .collect();
    bodies.sort_by(|a, b| (a.0.level, &a.0.name).cmp(&(b.0.level, &b.0.name)));
    for (def, body) in bodies {
        let object = content.object(body.object);
        let materials: Vec<String> = (object.made_of(0).iter())
            .map(|&m| content.material(m).name.clone())
            .collect();
        let mut made = materials.join(", ");
        if body.count.1 > 1 {
            let _ = write!(made, " ({} to {} pieces)", body.count.0, body.count.1);
        }
        let attack = def.attack.as_ref().map_or("-".to_string(), |a| {
            let kinds: Vec<String> = (a.attack.kinds.iter())
                .map(|k| content.damage_type(k).name.clone())
                .collect();
            let mut text = format!("{} {}, \"{}\"", a.attack.damage, kinds.join(" or "), a.verb);
            for hit in on_hit(content, a) {
                let _ = write!(text, "; {hit}");
            }
            text
        });
        let resists: Vec<String> = (def.resist.iter())
            .map(|&(k, v)| format!("{} {}", content.damage_type(k).name, signed(v)))
            .collect();
        let protection = def.natural.as_ref().map_or(0, |h| h.protection);
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {attack} | {protection} | {} | \"{}\" |",
            capital(&def.name),
            glyph(def.glyph),
            def.level,
            content.creature_rarity(def.rarity).name,
            made,
            def.speed,
            if resists.is_empty() {
                "-".into()
            } else {
                resists.join(", ")
            },
            def.death_verb()
        );
    }
    out
}

/// What a household body's base material gives its type, by the material's group.
pub(super) fn animated_groups(content: &Content) -> String {
    let mut out =
        String::from("| Material group | Resists | Protection | Dies |\n|---|---|---:|---|\n");
    let mut groups: Vec<_> = content.animated().groups.iter().collect();
    groups.sort_by(|a, b| a.id.cmp(&b.id));
    for group in groups {
        let resists: Vec<String> = (group.resist.iter())
            .map(|(k, v)| format!("{k} {}", signed(*v)))
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | \"{}\" |",
            capital(&group.id),
            if resists.is_empty() {
                "-".into()
            } else {
                resists.join(", ")
            },
            group.protection,
            group.dies
        );
    }
    out
}

//! Creatures and factions (spec §7 "Factions"), and the numbers the game keeps to itself about
//! them (spec §8 "Creature behavior", "Combat", "Speed", "Stats"): the wiki states them exactly,
//! read through the rules' own functions.

use super::actions::at_speed;
use super::combat::threat_range;
use super::order;
use super::text::{capital, damage_text, glyph, signed, yes};
use std::fmt::Write as _;
use warlock_core::action::AttackKind;
use warlock_core::burden::{self, Burden};
use warlock_core::combat::Category;
use warlock_core::content::{
    ActionKind, Affliction, BASE_TURN, Benefit, Combat, Content, CreatureDef, CreatureId,
    FactionId, NORMAL_COST, NaturalAttack, Stance, article, price, spoken,
};
use warlock_core::geom::Pos;
use warlock_core::object::format_weight;
use warlock_core::stats::Stat;
use warlock_core::world::{Actor, ActorId, ActorKind};

/// How a creature stands toward the player and how it attacks, as seen in play, how often it
/// drops a magic item, and the gear it may spawn with. By name: the shared types, then each
/// campaign's own (spec §3) under its name.
pub(super) fn creatures(content: &Content) -> String {
    let mut out = creature_table(content, None);
    for campaign in order::campaigns(content) {
        let own = creature_table(content, Some(&campaign.id));
        if own.lines().count() > 2 {
            let _ = write!(
                out,
                "\n**{}** has creatures of its own:\n\n{own}",
                campaign.name
            );
        }
    }
    out
}

/// [`creatures`]' table of the types of campaign `campaign` (`None`: the shared types).
fn creature_table(content: &Content, campaign: Option<&str>) -> String {
    let mut out = String::from(
        "| Creature | Glyph | Faction | Toward you | Attacks | Opens doors | Carries a light | \
         Drops magic | Gear |\n|---|:---:|---|---|---|:---:|:---:|:---:|---|\n",
    );
    let player = content.creature_id("player");
    let theirs = (listed(content).into_iter()).filter(|(_, c)| c.campaign.as_deref() == campaign);
    for (id, c) in theirs {
        let (faction, stance) = match player {
            // A type of several factions: each creature joins one by these odds (the fey courts).
            Some(p) if !c.factions.is_empty() => courts(content, p, c),
            _ => {
                let faction = c.faction.map_or_else(
                    || "none".into(),
                    |f| capital(&content.factions().def(f).name),
                );
                let stance = match player.map(|p| content.stance(p, id)) {
                    Some(Stance::Hostile) => "hostile",
                    _ => "neutral",
                };
                (faction, stance.to_string())
            }
        };
        let mut attacks = match c.combat {
            _ if !fights(content, c) => "never",
            Combat::Melee => "up close",
            Combat::Ranged => "shoots from afar",
            Combat::Caster => "casts spells from afar",
        }
        .to_string();
        if let (true, Some(cost)) = (fights(content, c), c.attack_cost) {
            let _ = write!(attacks, " (cost {cost})");
        }
        if let (true, Some(throw)) = (fights(content, c), &c.throw) {
            let noun = &throw.attack.name;
            let _ = write!(attacks, ", throws {} {noun} from afar", article(noun));
        }
        // The chance it drops a magic or rare item where it dies.
        let drops = match c.loot {
            0 => "-".to_string(),
            n => format!("{n}%"),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {faction} | {stance} | {attacks} | {} | {} | {drops} | {} |",
            capital(&c.name),
            glyph(c.glyph),
            yes(c.opens_doors),
            yes(c.light.is_some()),
            gear(content, c)
        );
    }
    out
}

/// The factions creatures of type `c` join, each with its odds ("Seelie court 90%, Unseelie
/// court 10%"), and how each stands toward the player (`player`, the player's type), in the
/// same order ("neutral or hostile").
fn courts(content: &Content, player: CreatureId, c: &CreatureDef) -> (String, String) {
    let theirs = content.creature(player).faction;
    let total: u32 = c.factions.iter().map(|&(_, w)| w).sum();
    let (names, stances): (Vec<String>, Vec<&str>) = (c.factions.iter())
        .map(|&(f, w)| {
            let name = capital(&content.factions().def(f).name);
            let stance = match content.stance_of(theirs, Some(f)) {
                Stance::Hostile => "hostile",
                Stance::Neutral => "neutral",
            };
            (format!("{name} {}%", w * 100 / total), stance)
        })
        .unzip();
    (names.join(", "), stances.join(" or "))
}

/// A creature type's gear table in words (spec §8 "Creature gear"): each line in the order it is
/// tried, its alternatives, its chance, and the chance it is magic; the racial weapon marked.
fn gear(content: &Content, c: &CreatureDef) -> String {
    let racial = c.gear.racial_weapon;
    let lines: Vec<String> = (c.gear.lines.iter())
        .map(|line| {
            let alternatives: Vec<String> = (line.one_of.iter())
                .map(|(stacks, _)| {
                    let names: Vec<String> = stacks.iter().map(|s| s.describe(content)).collect();
                    names.join(" and ")
                })
                .collect();
            let mut text = format!("{} {}%", alternatives.join(" or "), line.chance);
            let only = |o| line.one_of.len() == 1 && line.one_of[0].0.iter().any(|s| s.object == o);
            if racial.is_some_and(only) {
                text.push_str(" (racial weapon)");
            }
            if line.magic > 0 {
                let _ = write!(text, " (magic {}%)", line.magic);
            }
            text
        })
        .collect();
    if lines.is_empty() {
        "-".into()
    } else {
        lines.join("; ")
    }
}

/// Who fights whom: each faction and the factions hostile to it (relations are symmetric).
pub(super) fn factions(content: &Content) -> String {
    let factions = content.factions();
    let mut out = String::from(
        "| Faction | Hostile to | Animals | Attacked by you, joins |\n|---|---|---|---|\n",
    );
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
        let provoked = f
            .provoked
            .map_or_else(|| "-".to_string(), |p| capital(&factions.def(p).name));
        let _ = writeln!(
            out,
            "| {} | {enemies} | {} | {provoked} |",
            capital(&f.name),
            yes(f.animal)
        );
    }
    out
}

/// Every creature tag (spec §8 "Quests"), by tag: the word a kill quest names (`tag:goblin`),
/// its plural as quests say it, and the creature types that carry it, by name (a tag carried
/// by one type is listed too).
pub(super) fn creature_tags(content: &Content) -> String {
    let mut out = String::from("| Tag | Said | Creatures |\n|---|---|---|\n");
    for tag in content.tags() {
        let mut names: Vec<&str> = (content.tagged(tag))
            .filter(|&c| content.family(c) == c)
            .map(|c| content.creature(c))
            .filter(|c| order::described(content, c.campaign.as_deref()))
            .map(|c| c.name.as_str())
            .collect();
        if names.is_empty() {
            continue;
        }
        names.sort_by_cached_key(|n| n.to_lowercase());
        let said = content.plural(&spoken(tag));
        let _ = writeln!(out, "| `{tag}` | {said} | {} |", names.join(", "));
    }
    out
}

/// Every creature type but the player's, by name.
fn listed(content: &Content) -> Vec<(CreatureId, &CreatureDef)> {
    (order::creatures(content).into_iter())
        .filter(|(_, c)| c.id != "player")
        .collect()
}

/// Whether creatures of type `c` fight at all: their faction has enemies.
fn fights(content: &Content, c: &CreatureDef) -> bool {
    c.faction.is_some_and(|f| content.factions().has_enemies(f))
}

/// A creature of type `id` as a level spawns it, but bare: carrying, wearing, and wielding
/// nothing, with no route, nowhere in particular (`world/travel.rs` makes the real ones). The
/// tables read its numbers through the rules' own [`Actor`] methods, so they cannot drift from
/// what a fight uses.
///
/// It is read from the actor's saved form with only the fields a save requires, so every other
/// field takes its default, as it does for a fresh creature (a field an actor gains later
/// needs nothing here); then its type's skills.
fn bare(content: &Content, id: CreatureId) -> Actor {
    let def = content.creature(id);
    let saved = serde_json::json!({
        "id": ActorId(0),
        "kind": ActorKind::Creature,
        "creature": id,
        "level": "",
        "pos": Pos::new(0, 0),
        "patrol": null,
        "energy": 0,
        "stats": def.stats,
        "hp": 0,
        "hp_base": def.base_hp,
    });
    let mut actor: Actor = serde_json::from_value(saved)
        .unwrap_or_else(|e| panic!("a bare {} does not load: {e}", def.id));
    actor.skills = def.skills.clone();
    actor.weapon_skills = def.weapon_skills.clone();
    actor.hp = actor.max_hp(content);
    actor
}

/// The energy price of a blow by `actor` with nothing in its hands, as the rules price it for one
/// unburdened in the neutral stance (`World::attack_price`): its type's cost of a melee attack,
/// less what its ranks take off, over the 6-second base.
fn blow_price(content: &Content, actor: &Actor) -> i64 {
    let saving = actor.attack_benefit(content, AttackKind::Melee, None).speed;
    let cost = content.action_cost(actor.creature, ActionKind::Melee);
    price(Benefit::saving(cost, saving), BASE_TURN)
}

/// A score with its modifier: `13 (+1)`.
fn score(score: u8) -> String {
    let modifier = warlock_core::stats::modifier(i32::from(score));
    format!("{score} ({})", signed(modifier))
}

/// Each creature's level, the experience a kill is worth at that level, its six scores with
/// their modifiers, the potentials above them, and its trained skills and weapon skills.
pub(super) fn creature_stats(content: &Content) -> String {
    let stats: String = Stat::ALL
        .iter()
        .map(|s| format!(" {} |", s.abbrev()))
        .collect();
    let mut out = format!(
        "| Creature | Level | XP | Rarity |{stats} Potentials | Skills | Weapon skills |\n\
         |---|:---:|---:|---|{}---|---|---|\n",
        ":---:|".repeat(Stat::COUNT)
    );
    for (_, c) in listed(content) {
        let scores: String = (Stat::ALL.iter())
            .map(|&s| format!(" {} |", score(c.stats.score(s))))
            .collect();
        let higher: Vec<String> = (Stat::ALL.iter())
            .filter(|&&s| c.stats.potential(s) > c.stats.score(s))
            .map(|&s| format!("{} {}", s.abbrev(), c.stats.potential(s)))
            .collect();
        let potentials = if higher.is_empty() {
            "as its scores".to_string()
        } else {
            higher.join(", ")
        };
        let skills: Vec<String> = (c.skills.iter())
            .map(|(id, t)| format!("{} {}", content.skill(id).name, t.value))
            .collect();
        let table = content.weapon_skills();
        let weapon_skills: Vec<String> = (c.weapon_skills.trained())
            .filter(|(_, t)| t.rank > 0)
            .map(|(id, t)| format!("{} {}", table.skill(id).name, t.rank))
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |{scores} {potentials} | {} | {} |",
            capital(&c.name),
            c.level,
            c.xp,
            content.creature_rarity(c.rarity).name,
            dash(&skills),
            dash(&weapon_skills)
        );
    }
    out
}

/// A list for a table cell, `-` when empty.
fn dash(items: &[String]) -> String {
    if items.is_empty() {
        "-".into()
    } else {
        items.join(", ")
    }
}

/// Each creature's hit points, its Defence with no gear (and its parts), its natural armour's
/// Protection, and what it resists.
pub(super) fn creature_defences(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Hit points | Defence | Protection | Resists |\n|---|:---:|---|---|---|\n",
    );
    for (id, c) in listed(content) {
        let actor = bare(content, id);
        let defence = actor.defence(content, Burden::Unburdened, None);
        let parts: Vec<String> = (Category::ALL.iter())
            .filter(|&&k| defence.bonus(k) != 0)
            .map(|&k| format!("{} {}", k.name(), signed(defence.bonus(k))))
            .collect();
        let mut defence = if parts.is_empty() {
            defence.total().to_string()
        } else {
            format!("{} ({})", defence.total(), parts.join(", "))
        };
        // An incorporeal creature inside a wall (spec §8 "Incorporeal creatures").
        if let Some(i) = c.incorporeal.filter(|i| i.wall_defence != 0) {
            defence.push_str(&format!("; Wall {} inside walls", signed(i.wall_defence)));
        }
        let protection = match &c.natural {
            Some(hide) if hide.protection > 0 => format!("{} ({})", hide.protection, hide.name),
            _ => "-".into(),
        };
        let resists: Vec<String> = (c.resist.iter())
            .map(|&(kind, percent)| {
                let name = &content.damage_type(kind).name;
                match percent {
                    100 => format!("immune to {name}"),
                    p if p < 0 => format!("{name} {}% (takes more)", signed(p)),
                    p => format!("{name} {p}%"),
                }
            })
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {defence} | {protection} | {} |",
            capital(&c.name),
            actor.max_hp(content),
            dash(&resists)
        );
    }
    out
}

/// What a hit with `attack` may cause: each status with its degrees and each disease, with its
/// chance.
pub(super) fn on_hit(content: &Content, attack: &NaturalAttack) -> Vec<String> {
    (attack.on_hit.iter())
        .map(|o| match o.what {
            Affliction::Status { status, severity } => {
                let degrees = if severity == 1 { "degree" } else { "degrees" };
                format!(
                    "{}% {} ({severity} {degrees})",
                    o.chance,
                    content.status(status).name.to_lowercase()
                )
            }
            Affliction::Hold { status, difficulty } => {
                let def = content.status(status);
                let grip = difficulty.or(def.hold.map(|h| h.difficulty)).unwrap_or(0);
                format!("{}% {} (grip {grip})", o.chance, def.name.to_lowercase())
            }
            Affliction::Disease(d) => format!(
                "{}% {} (unless resisted)",
                o.chance,
                content.disease(d).name
            ),
            Affliction::Drain(drain) => format!(
                "{}% drains {} for good (unless resisted)",
                o.chance,
                super::stats::drained_stat(drain.stat)
            ),
        })
        .collect()
}

/// How each creature fights and its blow with nothing in its hands: its natural attack or its
/// fists, the damage, the to-hit, the critical, how long a blow takes it, and what a hit may
/// cause.
pub(super) fn creature_attacks(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Fights | Attack | Damage | To hit | Critical | A blow takes | On a hit |\n\
         |---|---|---|---|:---:|:---:|---:|---|\n",
    );
    for (id, c) in listed(content) {
        let fight = match c.combat {
            _ if !fights(content, c) => "never".to_string(),
            Combat::Melee => "up close".to_string(),
            Combat::Ranged => format!("shoots, up to {} cells", c.range),
            Combat::Caster => format!("casts spells, up to {} cells", c.range),
        };
        let actor = bare(content, id);
        // A family that wields what it is made of (animated blades): its weapon's blow.
        if c.attack.is_none() && content.family_levels(id).is_some() {
            let _ = writeln!(
                out,
                "| {} | {fight} | its weapon | its weapon's, with its Strength | its weapon's | \
                 its weapon's | {} | its weapon's |",
                capital(&c.name),
                at_speed(blow_price(content, &actor), c.speed),
            );
            continue;
        }
        let profile = actor.melee(content);
        let kinds: Vec<String> = (profile.kinds.iter())
            .map(|k| content.damage_type(k).name.clone())
            .collect();
        let damage = format!(
            "{} {}",
            damage_text(profile.damage, profile.bonus),
            kinds.join(" or ")
        );
        let attack = c.attack.as_ref().map_or("fists", |a| a.name.as_str());
        let caused = c
            .attack
            .as_ref()
            .map_or_else(Vec::new, |a| on_hit(content, a));
        let _ = writeln!(
            out,
            "| {} | {fight} | {attack} | {} | {} | {} ×{} | {} | {} |",
            capital(&c.name),
            damage.trim_end(),
            signed(profile.to_hit),
            threat_range(profile.threat),
            profile.crit,
            at_speed(blow_price(content, &actor), c.speed),
            dash(&caused)
        );
    }
    out
}

/// What each creature that throws (spec §8 "Ranged combat": "Creatures' throws") throws: what,
/// the damage (its Strength modifier and its ranks counted), the to-hit (its Dexterity's), the
/// critical, how far, how long a throw takes it, and what a hit may cause.
pub(super) fn creature_throws(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Throws | Damage | To hit | Critical | Range | A throw takes | On a hit |\n\
         |---|---|---|:---:|:---:|---:|---:|---|\n",
    );
    for (id, c) in listed(content) {
        let Some(throw) = &c.throw else {
            continue;
        };
        let actor = bare(content, id);
        let Some(profile) = actor.hurled(content) else {
            continue;
        };
        let kinds: Vec<String> = (profile.kinds.iter())
            .map(|k| content.damage_type(k).name.clone())
            .collect();
        let damage = format!(
            "{} {}",
            damage_text(profile.damage, profile.bonus),
            kinds.join(" or ")
        );
        let saving = actor
            .attack_benefit(content, AttackKind::Thrown, None)
            .speed;
        let cost = content.action_cost(id, ActionKind::Throw);
        let takes = price(Benefit::saving(cost, saving), BASE_TURN);
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} ×{} | {} cells | {} | {} |",
            capital(&c.name),
            throw.attack.name,
            damage.trim_end(),
            signed(profile.to_hit),
            threat_range(profile.threat),
            profile.crit,
            throw.range,
            at_speed(takes, c.speed),
            dash(&on_hit(content, &throw.attack))
        );
    }
    out
}

/// Each creature's speed and how long a normal action takes it, how far and with which eyes it
/// sees, how faint a sound it hears (its listen skill included), how loud its steps are, what it
/// weighs, its size, and its carrying limit (spec §8 "Burden").
pub(super) fn creature_senses(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Speed | A normal action takes | Sight | Vision | Hearing | Footsteps | Weight \
         | Size | Carrying limit |\n|---|---:|---:|---:|---|:---:|:---:|---:|---|---:|\n",
    );
    for (id, c) in listed(content) {
        let actor = bare(content, id);
        let hearing = actor
            .hearing(content)
            .map_or_else(|| "deaf".to_string(), |h| h.to_string());
        let footsteps = match c.footsteps {
            0 => "silent".to_string(),
            n => n.to_string(),
        };
        let size = content.size(c.size);
        let limit = burden::limit(
            c.stats.score(Stat::Strength),
            c.stats.score(Stat::Toughness),
            size.carry,
        );
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} cells | {} | {hearing} | {footsteps} | {} | {} | {} |",
            capital(&c.name),
            c.speed,
            at_speed(price(NORMAL_COST, BASE_TURN), c.speed),
            c.sight,
            content.vision_profiles()[c.vision].name,
            format_weight(u64::from(c.weight)),
            size.name,
            format_weight(limit)
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tags_name_their_creatures() {
        let content = Content::builtin().expect("content");
        let table = creature_tags(&content);
        assert!(
            table.contains(
                "| `goblin` | goblins | goblin, goblin archer, goblin chief, hobgoblin, \
                 hobgoblin archer, hobgoblin captain, hobgoblin legionnaire, hobgoblin sergeant, \
                 hobgoblin warcaster, hobgoblin warlord |"
            ),
            "{table}"
        );
        assert!(
            table.contains("| `marsh-thing` | marsh things |"),
            "{table}"
        );
        assert!(table.contains("| `undead` | undead |"), "{table}");
    }
}

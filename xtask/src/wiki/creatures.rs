//! Creatures and factions (spec §7 "Factions"), and the numbers the game keeps to itself about
//! them (spec §8 "Creature behavior", "Combat", "Speed", "Stats"): the wiki states them exactly,
//! read through the rules' own functions.

use super::combat::threat_range;
use super::order;
use super::util::{
    Term, anchored, at_speed, capital, damage_link, damage_text, glyph, link, signed, yes,
};
use std::fmt::Write as _;
use warlock_core::action::AttackKind;
use warlock_core::burden::{self, Burden};
use warlock_core::combat::Category;
use warlock_core::content::{
    ActionKind, Affliction, BASE_TURN, Benefit, BurstWhen, Combat, Content, CorrodesWhat,
    CreatureDef, CreatureId, FactionId, Invisible, Leaving, NORMAL_COST, NaturalAttack, OnHit,
    Stance, Takes, price, spoken,
};
use warlock_core::geom::Pos;
use warlock_core::object::format_weight;
use warlock_core::skills::Effect;
use warlock_core::stats::Stat;
use warlock_core::text::article;
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
        "| Creature | Glyph | Faction | Toward you | Attacks |\n|---|:---:|---|---|---|\n",
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
        let vortex = (c.burst.as_ref()).is_some_and(|b| b.when == BurstWhen::Touches);
        let mut attacks = match c.combat {
            _ if !fights(content, c) || strikes_nothing(c) => "never",
            _ if vortex => "bursts beside you",
            Combat::Melee => "up close",
            Combat::Ranged => "shoots from afar",
            Combat::Caster => "casts spells from afar",
            Combat::Dragon => "breathes, and up close",
        }
        .to_string();
        if let (true, Some(cost)) = (fights(content, c), c.attack_cost) {
            let _ = write!(attacks, " (cost {cost})");
        }
        if let (true, Some(throw)) = (fights(content, c), &c.throw) {
            let noun = &throw.attack.name;
            let _ = write!(attacks, ", throws {} {noun} from afar", article(noun));
        }
        if (c.burst.as_ref()).is_some_and(|b| b.when == BurstWhen::Dies) {
            attacks.push_str(", bursts when it dies");
        }
        let _ = writeln!(
            out,
            "| {} | {} | {faction} | {stance} | {attacks} |",
            anchored(Term::Creature, &c.id, &capital(&c.name)),
            glyph(c.glyph),
        );
    }
    out
}

/// What each creature does with doors and light, how often it drops a magic item, the colour of
/// crystal its nature is, and the gear it may spawn with (#1668: apart from [`creatures`]). By
/// name: the shared types, then each campaign's own under its name.
pub(super) fn creature_carries(content: &Content) -> String {
    let mut out = carries_table(content, None);
    for campaign in order::campaigns(content) {
        let own = carries_table(content, Some(&campaign.id));
        if own.lines().count() > 2 {
            let _ = write!(out, "\n**{}**:\n\n{own}", campaign.name);
        }
    }
    out
}

/// [`creature_carries`]' table of the types of campaign `campaign` (`None`: the shared types).
fn carries_table(content: &Content, campaign: Option<&str>) -> String {
    let mut out = String::from(
        "| Creature | Doors and light | Drops magic | Crystal | Gear |\n\
         |---|---|:---:|:---:|---|\n",
    );
    let theirs = (listed(content).into_iter()).filter(|(_, c)| c.campaign.as_deref() == campaign);
    for (id, c) in theirs {
        let mut habits = Vec::new();
        if c.opens_doors {
            habits.push("opens doors");
        }
        if c.light.is_some() {
            habits.push("carries a light");
        }
        let habits = match habits.is_empty() {
            true => "-".to_string(),
            false => habits.join(", "),
        };
        // The chance it drops a magic or rare item where it dies.
        let drops = match c.loot {
            0 => "-".to_string(),
            n => format!("{n}%"),
        };
        // The colour of crystal its nature is (data/crystals.toml): the first whose tags it
        // carries.
        let crystal = (content.crystals())
            .and_then(|k| k.colour_of(content, id))
            .map_or_else(|| "-".into(), |k| link(Term::Crystal, &k.id, &k.name));
        let _ = writeln!(
            out,
            "| {} | {habits} | {drops} | {crystal} | {} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
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
                    // A unique item by its name, then what it looks like: "Widowmaker (a
                    // stiletto (rare))".
                    let names: Vec<String> = (stacks.iter())
                        .map(|s| match content.uniques().find(|u| u.stack == *s) {
                            Some(u) => format!("{} ({})", u.name, s.describe(content)),
                            None => s.describe(content),
                        })
                        .collect();
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
            // Everyone is at war with the berserk and the loosed: said on their own rows, not on
            // every row.
            .filter(|&(_, e)| !e.hates_everyone())
            .filter(|&(j, _)| factions.stance(id, FactionId(j as u16)) == Stance::Hostile)
            .map(|(_, e)| e.name.clone())
            .collect();
        // The factions at peace with everyone, the berserk and the loosed too (the gates).
        let spared: Vec<&str> = (factions.defs().iter())
            .filter(|e| e.friendly_to_all)
            .map(|e| e.name.as_str())
            .collect();
        let and_spared: String = spared.iter().map(|s| format!(" and {s}")).collect();
        let enemies = if f.hostile_to_all {
            match spared.is_empty() {
                true => "everyone, its own too".to_string(),
                false => format!("everyone, its own too, but {}", spared.join(" and ")),
            }
        } else if f.hostile_to_others {
            format!("everyone but its own{and_spared}")
        } else if enemies.is_empty() {
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

/// The families the numbers' tables are split by, a page each (#1550): its title, its anchor
/// (the page is `creature-numbers-ANCHOR.md`), and its tags. A creature is in the first family
/// whose tags it carries, else in the last.
pub(super) const FAMILIES: &[(&str, &str, &[&str])] = &[
    (
        "The dead and spirits",
        "dead",
        &["undead", "skeleton", "spirit"],
    ),
    (
        "Fiends",
        "fiends",
        &["fiend", "demon", "devil", "yugoloth", "gate"],
    ),
    ("The fey", "fey", &["fey"]),
    (
        "Elementals",
        "elementals",
        &[
            "elemental",
            "air",
            "earth",
            "fire",
            "ice",
            "lightning",
            "water",
        ],
    ),
    (
        "Animated things and mimics",
        "animated",
        &["construct", "animated", "mimic"],
    ),
    (
        "Giants, trolls, hags, and dragons",
        "giants",
        &["giant", "ogre", "troll", "hag", "dragon"],
    ),
    (
        "Kobolds, goblins, and orcs",
        "goblins",
        &["kobold", "goblin", "hobgoblin", "orc"],
    ),
    (
        "Drow, duergar, and deep gnomes",
        "deep-folk",
        &["drow", "duergar", "deep-gnome"],
    ),
    (
        "Beasts and vermin",
        "beasts",
        &["beast", "vermin", "marsh-thing"],
    ),
    ("People and other creatures", "others", &[]),
];

/// The family (an index into [`FAMILIES`]) creature type `id` is in.
fn family(content: &Content, id: CreatureId) -> usize {
    (FAMILIES.iter())
        .position(|(_, _, tags)| tags.iter().any(|t| content.has_tag(id, t)))
        .unwrap_or(FAMILIES.len() - 1)
}

/// A creature table (its header, then rows that each start with a creature's link) split by
/// family: each family's rows under the header again, between mdBook anchors named by the
/// family, so its page includes its own (`{{#include generated/creature-stats.md:fey}}`).
fn by_family(content: &Content, table: &str) -> String {
    let rows = table
        .find("\n|---")
        .and_then(|i| table[i + 1..].find('\n').map(|j| i + j + 2));
    let rows = rows.expect("a table with a header");
    let (header, rows) = table.split_at(rows);
    let families: std::collections::BTreeMap<String, usize> = (listed(content).into_iter())
        .map(|(id, c)| {
            (
                link(Term::Creature, &c.id, &capital(&c.name)),
                family(content, id),
            )
        })
        .collect();
    let mut out = String::new();
    for (i, (_, anchor, _)) in FAMILIES.iter().enumerate() {
        let _ = writeln!(out, "<!-- ANCHOR: {anchor} -->\n\n{header}");
        out.pop();
        for row in rows.lines() {
            let creature = row
                .trim_start_matches("| ")
                .split(" | ")
                .next()
                .unwrap_or("");
            if families.get(creature) == Some(&i) {
                let _ = writeln!(out, "{row}");
            }
        }
        let _ = writeln!(out, "\n<!-- ANCHOR_END: {anchor} -->\n");
    }
    out
}

/// Whether creatures of type `c` fight at all: their faction has enemies.
fn fights(content: &Content, c: &CreatureDef) -> bool {
    c.faction.is_some_and(|f| content.factions().has_enemies(f))
}

/// Whether a creature of type `c` strikes no blow: rooted, with no natural attack, and holding
/// nothing (a miscast's gate; `World::rooted_plan`), so it only summons or waits.
fn strikes_nothing(c: &CreatureDef) -> bool {
    c.rooted && c.attack.is_none() && c.gear.is_empty()
}

/// A creature of type `id` as a level spawns it, but bare: carrying, wearing, and wielding
/// nothing, with no route, nowhere in particular (`world/levels/travel.rs` makes the real ones). The
/// tables read its numbers through the rules' own [`Actor`] methods, so they cannot drift from
/// what a fight uses.
///
/// It is read from the actor's saved form with only the fields a save requires, so every other
/// field takes its default, as it does for a fresh creature (a field an actor gains later
/// needs nothing here); then its type's skills.
pub(super) fn bare(content: &Content, id: CreatureId) -> Actor {
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

/// Each creature's six scores with their modifiers, and the potential where it is above the
/// score (`13 (+1), potential 15`): a grid along the stats (#1668); its level and skills are
/// [`creature_levels`].
pub(super) fn creature_stats(content: &Content) -> String {
    let stats: String = Stat::ALL
        .iter()
        .map(|s| format!(" {} |", s.abbrev()))
        .collect();
    let mut out = format!(
        "{}| Creature |{stats}\n|---|{}\n",
        super::util::stat_key(),
        ":---:|".repeat(Stat::COUNT)
    );
    for (_, c) in listed(content) {
        let scores: String = (Stat::ALL.iter())
            .map(|&s| {
                let potential = c.stats.potential(s);
                match potential > c.stats.score(s) {
                    true => format!(" {}, potential {potential} |", score(c.stats.score(s))),
                    false => format!(" {} |", score(c.stats.score(s))),
                }
            })
            .collect();
        let _ = writeln!(
            out,
            "| {} |{scores}",
            link(Term::Creature, &c.id, &capital(&c.name)),
        );
    }
    by_family(content, &out)
}

/// Each creature's level, the experience a kill is worth at that level, its rarity, and its
/// trained skills and weapon skills.
pub(super) fn creature_levels(content: &Content) -> String {
    let mut out =
        String::from("| Creature | Level | XP | Rarity | Skills |\n|---|:---:|---:|---|---|\n");
    for (_, c) in listed(content) {
        let mut skills: Vec<String> = (c.skills.iter())
            .map(|(id, t)| format!("{} {}", content.skill(id).name, t.value))
            .collect();
        let table = content.weapon_skills();
        let weapon_skills: Vec<String> = (c.weapon_skills.trained())
            .filter(|(_, t)| t.rank > 0)
            .map(|(id, t)| format!("{} {}", table.skill(id).name, t.rank))
            .collect();
        if !weapon_skills.is_empty() {
            skills.push(format!("weapon skills: {}", weapon_skills.join(", ")));
        }
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            c.level,
            xp_cell(c),
            rarity_cell(content, c),
            dash(&skills),
        );
    }
    by_family(content, &out)
}

/// A creature's rarity for a table cell, and whether it is dread (docs/design/monster-dread.md
/// §8): `Rare, [dread](dungeons.md#dread)`.
fn rarity_cell(content: &Content, c: &CreatureDef) -> String {
    let rarity = &content.creature_rarity(c.rarity).name;
    match c.dread {
        true => format!("{rarity}, [dread](dungeons.md#dread)"),
        false => rarity.clone(),
    }
}

/// A creature's XP for a table cell: what a kill pays at its own level, and for one with a
/// hand-set `xp_percent` how that comes of its `xp`, `45 (30 × 150 %)`.
pub(super) fn xp_cell(c: &CreatureDef) -> String {
    if c.xp_percent == 100 {
        c.xp.to_string()
    } else {
        format!("{} ({} × {} %)", c.kill_xp(), c.xp, c.xp_percent)
    }
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
/// Protection, and what it resists (a bloodless one never bleeds).
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
        // Immunities first (#1620): no resistance, nothing of the kind hurts it.
        let immune = (c.immune_kinds.iter())
            .map(|&kind| format!("immune to {}", damage_link(content, kind)));
        let mut resists: Vec<String> = immune
            .chain(c.resist.iter().map(|&(kind, percent)| {
                let name = damage_link(content, kind);
                match percent {
                    p if p < 0 => format!("{name} {}% (takes more)", signed(p)),
                    p => format!("{name} {p}%"),
                }
            }))
            .collect();
        // Bloodless: no wound's status takes on it (Bleeding; `World::afflict_one`).
        if c.blood.is_none() {
            resists.push("never [bleeds](status-effects.md#bleeding)".into());
        }
        // Acid blood (`world/combat/corrode.rs`): what strikes it may come away damaged.
        if let Some(acid) = c.corrodes.filter(|a| a.what.eats(CorrodesWhat::Weapon)) {
            resists.push(format!(
                "{}% to [damage](materials.md#item-damage) the weapon that strikes it ({})",
                acid.chance,
                acid.kind.id()
            ));
        }
        // Regeneration (`world/health/regen.rs`): its own healing pace, stopped by fire and acid.
        let hp = match c.regen {
            100 => actor.max_hp(content).to_string(),
            pace => format!("{}, heals {} as fast", actor.max_hp(content), times(pace)),
        };
        let _ = writeln!(
            out,
            "| {} | {hp} | {defence} | {protection} | {} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            dash(&resists)
        );
    }
    by_family(content, &out)
}

/// What a hit with `attack` may cause: each status with its degrees and each disease, with its
/// chance.
pub(super) fn on_hit(content: &Content, attack: &NaturalAttack) -> Vec<String> {
    riders(content, &attack.on_hit)
}

/// What `riders` cause (an attack's `on_hit`, a gaze's riders), each with its chance.
fn riders(content: &Content, riders: &[OnHit]) -> Vec<String> {
    (riders.iter())
        .map(|o| match o.what {
            Affliction::Status {
                status,
                severity,
                check,
            } => {
                let degrees = if severity == 1 { "degree" } else { "degrees" };
                let resisted = match check {
                    Some((stat, difficulty)) => {
                        format!(", unless a {} check against {difficulty}", stat.name())
                    }
                    None => String::new(),
                };
                format!(
                    "{}% {} ({severity} {degrees}{resisted})",
                    o.chance,
                    content.status(status).name.to_lowercase()
                )
            }
            Affliction::Hold {
                status,
                difficulty,
                damage,
            } => {
                let def = content.status(status);
                let grip = difficulty.or(def.hold.map(|h| h.difficulty)).unwrap_or(0);
                let harm = match damage {
                    Some(1) if def.tick.is_some() => ", 1 hit point a tick".to_string(),
                    Some(d) if def.tick.is_some() => format!(", {d} hit points a tick"),
                    _ => String::new(),
                };
                format!(
                    "{}% {} (grip {grip}{harm})",
                    o.chance,
                    def.name.to_lowercase()
                )
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
            _ if !fights(content, c) || strikes_nothing(c) => "never".to_string(),
            Combat::Melee => "up close".to_string(),
            Combat::Ranged => format!("shoots, up to {} cells", c.range),
            Combat::Caster => format!("casts spells, up to {} cells", c.range),
            Combat::Dragon => "breathes, and up close".to_string(),
        };
        let actor = bare(content, id);
        // A vortex has no blow: its burst (the bursts' table).
        if c.burst
            .as_ref()
            .is_some_and(|b| b.when == BurstWhen::Touches)
        {
            let _ = writeln!(
                out,
                "| {} | {fight} | its burst | the bursts' table | - | - | {} | - |",
                link(Term::Creature, &c.id, &capital(&c.name)),
                at_speed(blow_price(content, &actor), c.speed),
            );
            continue;
        }
        // A gate has no blow at all: it only lets others through (the summons' table).
        if strikes_nothing(c) {
            let _ = writeln!(
                out,
                "| {} | {fight} | none | - | - | - | - | - |",
                link(Term::Creature, &c.id, &capital(&c.name)),
            );
            continue;
        }
        // A family that wields what it is made of (animated blades): its weapon's blow.
        if c.attack.is_none() && content.family_levels(id).is_some() {
            let _ = writeln!(
                out,
                "| {} | {fight} | its weapon | its weapon's, with its Strength | its weapon's | \
                 its weapon's | {} | its weapon's |",
                link(Term::Creature, &c.id, &capital(&c.name)),
                at_speed(blow_price(content, &actor), c.speed),
            );
            continue;
        }
        // Its natural attacks in the order it strikes them (a dragon's two claws and a bite):
        // the first takes the blow's time, the rest follow it in the same attack.
        let mut blows = vec![(None, actor.melee(content))];
        blows.extend((actor.more_natural_blows(content).into_iter()).map(|(i, p)| (Some(i), p)));
        for (index, profile) in blows {
            let natural = c.natural_attacks().nth(index.map_or(0, usize::from));
            let kinds: Vec<String> = (profile.kinds.iter())
                .map(|k| content.damage_type(k).name.clone())
                .collect();
            let damage = format!(
                "{} {}",
                damage_text(profile.damage, profile.bonus),
                kinds.join(" or ")
            );
            let attack = natural.map_or("fists", |a| a.name.as_str());
            let (attack, takes) = match index {
                None => (
                    attack.to_string(),
                    at_speed(blow_price(content, &actor), c.speed),
                ),
                Some(_) => (format!("then {attack}"), "with the first".to_string()),
            };
            let mut caused = natural.map_or_else(Vec::new, |a| on_hit(content, a));
            // Its filth (`world/combat/corrode.rs`): a worn piece damaged.
            if let Some(acid) = c.corrodes.filter(|a| a.what.eats(CorrodesWhat::Armour)) {
                caused.push(format!(
                    "{}% [damages](materials.md#item-damage) a piece worn ({})",
                    acid.chance,
                    acid.kind.id()
                ));
            }
            let _ = writeln!(
                out,
                "| {} | {fight} | {attack} | {} | {} | {} ×{} | {takes} | {} |",
                link(Term::Creature, &c.id, &capital(&c.name)),
                damage.trim_end(),
                signed(profile.to_hit),
                threat_range(profile.threat),
                profile.crit,
                dash(&caused)
            );
        }
    }
    by_family(content, &out)
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
        let saving = actor
            .attack_benefit(content, AttackKind::Thrown, None)
            .speed;
        let cost = content.action_cost(id, ActionKind::Throw);
        let takes = price(Benefit::saving(cost, saving), BASE_TURN);
        let creature = link(Term::Creature, &c.id, &capital(&c.name));
        let range = throw.range;
        let takes = at_speed(takes, c.speed);
        // A bomber's flask has no blow of its own: its splash, the flask's spell, is the attack.
        if let Some(flask) = &content.object(throw.object).flask {
            let spell = content.spell(flask.spell);
            let _ = writeln!(
                out,
                "| {creature} | {} | its splash: {} (rung {}) | - | - | {range} cells | {takes} | - |",
                link(
                    Term::Potion,
                    &content.object(throw.object).id,
                    &throw.attack.name
                ),
                link(Term::Spell, &spell.id, &spell.name),
                flask.rungs,
            );
            continue;
        }
        let kinds: Vec<String> = (profile.kinds.iter())
            .map(|k| content.damage_type(k).name.clone())
            .collect();
        let damage = format!(
            "{} {}",
            damage_text(profile.damage, profile.bonus),
            kinds.join(" or ")
        );
        let _ = writeln!(
            out,
            "| {creature} | {} | {} | {} | {} ×{} | {range} cells | {takes} | {} |",
            throw.attack.name,
            damage.trim_end(),
            signed(profile.to_hit),
            threat_range(profile.threat),
            profile.crit,
            dash(&on_hit(content, &throw.attack))
        );
    }
    out
}

/// Each summoner's summoning (spec §8 "Summoning"): what it calls forth, how many at once, how
/// often and at what mana, from how far, and how many of its summoned may live at once.
pub(super) fn creature_summons(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Calls forth | At once | Every | Mana | Within | Appear within | At most |\n\
         |---|---|:---:|---:|---:|---:|---:|---:|\n",
    );
    for (_, c) in listed(content) {
        let Some(summon) = &c.summon else {
            continue;
        };
        // The pool by name, each with its share when some are likelier than others.
        let mut names: Vec<(String, usize)> = Vec::new();
        for &p in &summon.pool {
            let name = content.creature(p).name.clone();
            match names.iter_mut().find(|(n, _)| *n == name) {
                Some((_, k)) => *k += 1,
                None => names.push((name, 1)),
            }
        }
        names.sort();
        let total = summon.pool.len();
        let even = names.iter().all(|(_, k)| *k == names[0].1);
        let pool: Vec<String> = (names.iter())
            .map(|(n, k)| match even {
                true => n.clone(),
                false => format!("{n} ({}%)", k * 100 / total),
            })
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} s | {} | {} cells | {} cells | {} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            pool.join(", "),
            summon.count,
            summon.cooldown,
            summon.mana,
            summon.range,
            summon.spread,
            summon.most
        );
    }
    out
}

/// What cuts each creature in two, and what crawls out of it when it dies
/// (docs/design/monster-dread.md §5.5): the kinds of damage that split it and the fewest hit
/// points a half keeps, then what its corpse lets out, how many, and how far from it.
pub(super) fn creature_splits(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Splits under | A half keeps at least | Dies into | How many | Within |\n\
         |---|---|---:|---|:---:|---:|\n",
    );
    for (_, c) in listed(content) {
        if c.splits.is_none() && c.dies_into.is_none() {
            continue;
        }
        let (on, least) = match &c.splits {
            Some(s) => (
                (s.on.iter())
                    .map(|&k| damage_link(content, k))
                    .collect::<Vec<_>>()
                    .join(", "),
                format!("{} hit points", s.least),
            ),
            None => ("—".into(), "—".into()),
        };
        let (into, count, spread) = match &c.dies_into {
            Some(d) => {
                let mut names: Vec<String> = (d.pool.iter())
                    .map(|&p| {
                        let other = content.creature(p);
                        link(Term::Creature, &other.id, &other.plural)
                    })
                    .collect();
                names.sort();
                names.dedup();
                (
                    names.join(", "),
                    d.count.to_string(),
                    format!("{} cell{}", d.spread, if d.spread == 1 { "" } else { "s" }),
                )
            }
            None => ("—".into(), "—".into(), "—".into()),
        };
        let _ = writeln!(
            out,
            "| {} | {on} | {least} | {into} | {count} | {spread} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
        );
    }
    out
}

/// What each dragon (spec §8 "Creature behavior": "Dragons") breathes: its element, the damage
/// rolled once for everyone in the cone, how long the cone is, what else it causes on those it
/// harmed (the gorgon's stiffening), and how long before it breathes again.
pub(super) fn creature_breaths(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Breathes | Damage | Cone | Then | Again after |\n|---|---|---|---:|---|---:|\n",
    );
    for (_, c) in listed(content) {
        let Some(breath) = &c.breath else {
            continue;
        };
        let then = match riders(content, &breath.riders) {
            r if r.is_empty() => "—".to_string(),
            r => r.join(", "),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} {} | {} cells | {then} | {} s |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            breath.noun,
            damage_text(breath.damage.into(), 0),
            damage_link(content, breath.kind),
            breath.length,
            breath.cooldown
        );
    }
    out
}

/// Each gazer's gaze (spec §8 "Creature behavior": "Gazes"): how far it reaches, the chance that
/// eyes that meet it are caught (and a glance's against averted ones), what it then causes, and
/// how long before it looks again.
pub(super) fn creature_gazes(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Reach | Catches the eye | Averted eyes | Then | Again after |\n\
         |---|---:|---:|---|---|---:|\n",
    );
    for (_, c) in listed(content) {
        let Some(gaze) = &c.gaze else {
            continue;
        };
        let averted = match gaze.averted {
            true => format!("forced; {}%", gaze.chance_against(true)),
            false => "—".to_string(),
        };
        let _ = writeln!(
            out,
            "| {} | {} cells | {}% | {averted} | {} | {} s |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            gaze.range,
            gaze.chance,
            riders(content, &gaze.riders).join(", "),
            gaze.cooldown
        );
    }
    out
}

/// Each thief's theft (docs/design/monster-dread.md §5.6): what it takes, its chance on a hit,
/// and how long before it steals again.
pub(super) fn creature_thieves(content: &Content) -> String {
    let mut out =
        String::from("| Creature | Takes | Chance on a hit | Again after |\n|---|---|---:|---:|\n");
    for (_, c) in listed(content) {
        let Some(steals) = &c.steals else {
            continue;
        };
        let takes = match steals.what {
            Takes::Small => "coins, gems, potions, rings, scrolls",
            Takes::Any => "anything",
        };
        let _ = writeln!(
            out,
            "| {} | {takes} | {}% | {} s |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            steals.chance,
            steals.cooldown
        );
    }
    out
}

/// What each creature that bursts (the dread design §5.3) bursts in: when (beside its target
/// instead of a blow, or where it dies), the damage rolled once for everyone around it, how far
/// it reaches, and what else it does to those it harms and the cells it reaches.
pub(super) fn creature_bursts(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Bursts | Damage | Radius | On those harmed | Also |\n\
         |---|---|---|---:|---|---|\n",
    );
    for (_, c) in listed(content) {
        let Some(burst) = &c.burst else {
            continue;
        };
        let when = match burst.when {
            BurstWhen::Touches => "beside its target, and is gone",
            BurstWhen::Dies => "where it dies",
        };
        let riders: Vec<String> = (burst.riders.iter())
            .map(|r| {
                let status = content.status(r.status);
                format!(
                    "{}% {}",
                    r.chance,
                    link(Term::Status, &status.id, &status.name.to_lowercase())
                )
            })
            .collect();
        let mut also = Vec::new();
        if burst.douses {
            also.push("puts out the burning and the fires");
        }
        if burst.gust {
            also.push("blows what lies loose a cell outward");
        }
        let _ = writeln!(
            out,
            "| {} | {when} | {} {} | {} | {} | {} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            damage_text(burst.damage.into(), 0),
            damage_link(content, burst.kind),
            burst.radius,
            dash(&riders),
            dash(&also.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        );
    }
    out
}

/// Each blinker's blink (docs/design/monster-dread.md §5.6): how far, and how long before it
/// blinks again.
pub(super) fn creature_blinks(content: &Content) -> String {
    let mut out = String::from("| Creature | Reach | Again after |\n|---|---:|---:|\n");
    for (_, c) in listed(content) {
        let Some(blinks) = &c.blinks else {
            continue;
        };
        let _ = writeln!(
            out,
            "| {} | {} cells | {} s |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            blinks.range,
            blinks.cooldown
        );
    }
    out
}

/// Each duellist's disarming blow (#1166): its chance on a hit, and the difficulty of the
/// Dexterity check that keeps the weapon in hand.
pub(super) fn creature_disarmers(content: &Content) -> String {
    let mut out =
        String::from("| Creature | Chance on a hit | Difficulty to hold on |\n|---|---:|---:|\n");
    for (_, c) in listed(content) {
        let Some(blow) = c.disarms else {
            continue;
        };
        let _ = writeln!(
            out,
            "| {} | {}% | {} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            blow.chance,
            blow.difficulty
        );
    }
    out
}

/// Warning signs (docs/design/monster-dread.md §9): every creature that leaves something about
/// it where the generator places it, or has a line said the first time it is seen.
pub(super) fn creature_signs(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Level | Rarity | Leaves | First seen |\n|---|---:|---|---|---|\n",
    );
    for (_, c) in listed(content) {
        if c.leavings.is_empty() && c.first_sight.is_none() {
            continue;
        }
        let leaves: Vec<String> = (c.leavings.iter())
            .map(|l| match l {
                Leaving::Statue => "statues",
                Leaving::Web => "old webs",
                Leaving::AcidPit => "acid-pitted floor",
                Leaving::TornCloth => "torn cloth",
            })
            .map(String::from)
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            c.level,
            rarity_cell(content, c),
            dash(&leaves),
            c.first_sight.as_deref().unwrap_or("-"),
        );
    }
    out
}

/// The unseen (docs/dev/creatures.md "The unseen"): every creature that is shadowy, invisible by
/// its kind, or fades from sight on a cooldown, with how long its Invisible status lasts and how
/// soon it fades again, and whether it hunts by scent.
pub(super) fn creature_unseen(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Level | Rarity | Unseen | Lasts | Again after | Hunts by scent |\n\
         |---|---:|---|---|---:|---:|:---:|\n",
    );
    for (_, c) in listed(content) {
        let (how, lasts, again) = match c.invisible {
            Some(Invisible::Innate) => ("always".to_string(), "-".into(), "-".into()),
            Some(Invisible::Fades {
                status, cooldown, ..
            }) => {
                let def = content.status(status);
                let lasts = def.eases.map_or("-".into(), |s| format!("{s} s"));
                let how = link(Term::Status, &def.id, "fades from sight");
                (how, lasts, format!("{cooldown} s"))
            }
            None if c.shadowy => ("in the dim".to_string(), "-".into(), "-".into()),
            None => continue,
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {how} | {lasts} | {again} | {} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            c.level,
            rarity_cell(content, c),
            yes(c.hunts),
        );
    }
    out
}

/// What each caster (spec §8 "Creature behavior": "Casters") casts, in the order it prefers
/// them, with what each costs it, and its pool of mana; how rare it is beside it.
pub(super) fn creature_spells(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Level | Rarity | Mana | Spells (mana each), first preferred |\n|---|---:|---|---:|---|\n",
    );
    for (id, c) in listed(content) {
        if c.spells.is_empty() {
            continue;
        }
        let mut actor = bare(content, id);
        actor.mana_base = c.base_mana;
        let spells: Vec<String> = (c.spells.iter())
            .map(|&s| {
                let spell = content.spell(s);
                let cost = actor.spell_cost(content, spell);
                format!("{} ({cost})", link(Term::Spell, &spell.id, &spell.name))
            })
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            c.level,
            content.creature_rarity(c.rarity).name,
            actor.max_mana(content),
            spells.join(", ")
        );
    }
    out
}

/// Each creature's presence (`world/creatures/aura.rs`): its aura (the status, its degrees, how far, by
/// sight or by hearing and how loud, the check that keeps it off, how often), the statuses its
/// kind always has, and its cry (what it sounds like, how loud, how often on average).
pub(super) fn creature_auras(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Aura | Reach | Kept off by | Every | Always has | Cry |\n\
         |---|---|---|---|---:|---|---|\n",
    );
    for (_, c) in listed(content) {
        if c.aura.is_none() && c.innate.is_empty() && c.cry.is_none() {
            continue;
        }
        let status = |s| {
            let def = content.status(s);
            link(Term::Status, &def.id, &def.name.to_lowercase())
        };
        let (aura, reach, check, every) = match &c.aura {
            None => ("-".into(), "-".into(), "-".into(), "-".into()),
            Some(a) => {
                let mut aura = format!("{} ({})", status(a.status), a.severity);
                if let Some(kind) = a.kind {
                    let _ = write!(aura, ", resisted as {}", damage_link(content, kind));
                }
                let reach = match a.heard {
                    Some(loud) => format!("{} cells, heard (loudness {loud})", a.radius),
                    None => format!("{} cells, in sight", a.radius),
                };
                let check = match a.check {
                    Some((stat, d)) => format!("{} {d}", stat.name()),
                    None => "nothing".into(),
                };
                (aura, reach, check, format!("{} s", a.every))
            }
        };
        let innate: Vec<String> = c.innate.iter().map(|&s| status(s)).collect();
        let cry = c.cry.map_or("-".to_string(), |cry| {
            format!(
                "{} (loudness {}, every {} s)",
                cry.kind.heard_as(),
                cry.loudness,
                cry.every
            )
        });
        let _ = writeln!(
            out,
            "| {} | {aura} | {reach} | {check} | {every} | {} | {cry} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            dash(&innate)
        );
    }
    out
}

/// Grades (spec §8 "Grades"): how often a creature comes greater or lesser than its kind, and
/// what that changes, from `warlock_core::grade`'s own numbers.
pub(super) fn creature_grades(_: &Content) -> String {
    use warlock_core::grade::{DAMAGE, HP_PERCENT, ONE_IN, SPEED_PERCENT, TO_HIT};
    let mut out = String::from(
        "| Grade | Chance | Maximum hit points | To hit | Damage | Speed |\n\
         |---|:---:|:---:|:---:|:---:|:---:|\n",
    );
    let percent = |n: i32| format!("{}%", signed(n));
    for (name, g) in [("Greater", 1), ("Lesser", -1)] {
        let _ = writeln!(
            out,
            "| {name} | 1 in {ONE_IN} | {} | {} | {} | {} |",
            percent(HP_PERCENT * g),
            signed(TO_HIT * g),
            signed(DAMAGE * g),
            percent(SPEED_PERCENT * g),
        );
    }
    out
}

/// The words look mode gives a lesser and a greater creature, each pair with the kinds that use
/// it, and the kinds that are never graded (spec §8 "Grades").
pub(super) fn creature_grade_words(content: &Content) -> String {
    let mut pairs: Vec<(&[String; 2], Vec<String>)> = Vec::new();
    let mut none = Vec::new();
    for (_, c) in listed(content) {
        let name = link(Term::Creature, &c.id, &c.name);
        if !c.grades {
            none.push(name);
            continue;
        }
        match pairs.iter_mut().find(|(w, _)| *w == &c.grade_words) {
            Some((_, names)) => names.push(name),
            None => pairs.push((&c.grade_words, vec![name])),
        }
    }
    pairs.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = String::from("| Lesser | Greater | Creatures |\n|---|---|---|\n");
    for (words, names) in pairs {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            words[0],
            words[1],
            names.join(", ")
        );
    }
    let _ = writeln!(out, "| - | - | never graded: {} |", none.join(", "));
    out
}

/// Each creature's speed and how long a normal action takes it, how far and with which eyes it
/// sees, how faint a sound it hears (its listen skill included), and how loud its steps are; its
/// body is [`creature_size`] (#1668).
pub(super) fn creature_senses(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Speed | Sight | Hearing | Footsteps |\n|---|---:|---|:---:|:---:|\n",
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
        let _ = writeln!(
            out,
            "| {} | {} ({}) | {} cells, {} | {hearing} | {footsteps} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            c.speed,
            at_speed(price(NORMAL_COST, BASE_TURN), c.speed),
            c.sight,
            content.vision_profiles()[c.vision].name,
        );
    }
    by_family(content, &out)
}

/// Each creature's weight, its size, its carrying limit (spec §8 "Burden"), and whether it may
/// be tamed, with its nose as a companion (spec §8 "Allies").
pub(super) fn creature_size(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Weight | Size | Carrying limit | Tameable |\n|---|---:|---|---:|---|\n",
    );
    for (_, c) in listed(content) {
        let size = content.size(c.size);
        let limit = burden::limit(
            c.stats.score(Stat::Strength),
            c.stats.score(Stat::Toughness),
            size.carry,
        );
        let tameable = match c.nose {
            Some(n) => format!("{}, nose {n} cells", yes(c.tameable)),
            None => yes(c.tameable).to_string(),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {tameable} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            format_weight(u64::from(c.weight)),
            size.name,
            format_weight(limit)
        );
    }
    by_family(content, &out)
}

/// The creatures with a craft for surprise (spec §8 "Surprise and sneak attacks"), by name:
/// their Sneak attack (its dice and the points it adds to the initiative check's difficulty),
/// their Initiative (the points it adds to their own check and the share of the dear action's
/// extra time it spares), their Spot against slipping past, and whether they lurk; each read
/// through the rules' own `Actor::skill_effect`.
pub(super) fn creature_surprise(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Sneak attack | Dice | Harder to keep wits | Initiative | To keep wits | Extra \
         time spared | Spot | Against slipping past | Lurks |\n\
         |---|---:|---:|---:|---:|---:|---:|---:|---:|:---:|\n",
    );
    let value = |c: &CreatureDef, skill: &str| {
        (content.skill_id(skill)).map_or(0, |s| c.skills.value(s).unwrap_or(0))
    };
    let points = |n: i32| match n {
        0 => "-".to_string(),
        n => signed(n),
    };
    for (id, c) in listed(content) {
        let (sneak, initiative, spot) = (
            value(c, "sneak_attack"),
            value(c, "initiative"),
            value(c, "spot"),
        );
        if sneak == 0 && initiative == 0 && spot == 0 && !c.lurks {
            continue;
        }
        let actor = bare(content, id);
        let effect = |e| actor.skill_effect(content, e);
        let dice = match effect(Effect::SneakDice) {
            0 => "-".to_string(),
            n => format!("{n}d6"),
        };
        let spared = match effect(Effect::SurpriseEase) {
            0 => "-".to_string(),
            n => format!("{n}%"),
        };
        let shown = |v: u16| match v {
            0 => "-".to_string(),
            v => v.to_string(),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {dice} | {} | {} | {} | {spared} | {} | {} | {} |",
            link(Term::Creature, &c.id, &capital(&c.name)),
            shown(sneak),
            points(effect(Effect::Surprise)),
            shown(initiative),
            points(effect(Effect::Initiative)),
            shown(spot),
            points(effect(Effect::Spot)),
            yes(c.lurks),
        );
    }
    out
}

/// A pace in percent of the normal as a factor: "four times" for 400, else "×1.5".
fn times(percent: u16) -> String {
    match percent {
        200 => "twice".into(),
        p if p % 100 == 0 => format!("{} times", p / 100),
        p => format!("×{}.{:02}", p / 100, p % 100),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tags_name_their_creatures() {
        let content = warlock_assets::content().expect("content");
        let table = creature_tags(&content);
        assert!(
            table.contains(
                "| `goblin` | goblins | goblin, goblin archer, goblin chief, goblin firebomber, goblin wizard, \
                 hobgoblin, hobgoblin archer, hobgoblin captain, hobgoblin legionnaire, hobgoblin sergeant, \
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

    /// Every family of the numbers' pages has creatures, and every creature is in one (#1550).
    #[test]
    fn every_family_has_its_creatures() {
        let content = warlock_assets::content().expect("content");
        let stats = creature_stats(&content);
        for (i, (title, _, _)) in FAMILIES.iter().enumerate() {
            let n = (listed(&content).into_iter()).filter(|&(id, _)| family(&content, id) == i);
            assert!(n.count() > 0, "{title} is empty");
        }
        let rows = stats.lines().filter(|l| l.starts_with("| [")).count();
        assert_eq!(rows, listed(&content).len());
    }
}

//! `cargo xtask wiki [--check | --build]`: the player wiki's generated tables.
//!
//! The wiki (`wiki/`, built with mdBook) is written by hand except for facts that live in data
//! (month names, moons, weather, terrain, vision profiles, light presets, creatures, factions,
//! objects and materials, equipment slots, scenarios, action costs, races and classes, their sizes
//! and carrying limits, the horoscope, skills: what each does, how it trains, what it gives by
//! value, who trains it, and the increase curve, special rooms; the walls that break and how
//! long digging takes; weapon skills: `wiki/weapon_skills.rs`) and a few constants of the rules
//! (sight radii, the pack's size, the stats' names, what each gender raises). Those are written
//! to `wiki/src/generated/*.md` from the built-in content, and the pages pull them in with
//! `{{#include generated/NAME.md}}`, so they cannot drift. Output is deterministic. `--check`
//! fails when a file is stale; `--build` also runs `mdbook build wiki`.

mod weapon_skills;

use crate::workspace_root;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::Command;
use warlock_core::burden::{self, Burden, Load};
use warlock_core::character::Gender;
use warlock_core::combat;
use warlock_core::content::{
    ActionKind, BASE_TURN, Combat, Content, CreatureId, FactionId, LevelKind, LightId,
    NORMAL_SPEED, ObjectId, Stance, TerrainDef, TrapEffect, price,
};
use warlock_core::daylight::{
    MOONLIGHT_PER_CELL, SIGHT_DAY, SIGHT_MOONLIT, SIGHT_NIGHT, SIGHT_TWILIGHT, overworld_sight,
};
use warlock_core::dice::Dice;
use warlock_core::light::{Light, Moon, Sky, Sun};
use warlock_core::object::{INVENTORY_SLOTS, format_weight};
use warlock_core::perception::VIEW_RADIUS;
use warlock_core::skills::{Effect, SkillId};
use warlock_core::stats::{AVERAGE, PerStat, Stat};
use warlock_core::time::{DAY, Duration, GameTime};
use warlock_core::weather::{Clouds, Conditions, Weather};
use warlock_core::world::digs_to_break;

/// What to do with the generated files.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Write,
    Check,
    Build,
}

pub fn parse_mode(args: &[String]) -> Result<Mode, String> {
    match args {
        [] => Ok(Mode::Write),
        [a] if a == "--check" => Ok(Mode::Check),
        [a] if a == "--build" => Ok(Mode::Build),
        _ => Err("usage: cargo xtask wiki [--check | --build]".into()),
    }
}

fn generated_dir() -> PathBuf {
    workspace_root().join("wiki/src/generated")
}

pub fn run(mode: Mode) -> Result<(), String> {
    let content = Content::builtin().map_err(|e| format!("cannot load content: {e}"))?;
    let files = generate(&content);
    let dir = generated_dir();
    match mode {
        Mode::Check => {
            let stale: Vec<&str> = files
                .iter()
                .filter(|(name, text)| {
                    std::fs::read_to_string(dir.join(name)).ok().as_deref() != Some(text.as_str())
                })
                .map(|(name, _)| name.as_str())
                .collect();
            if stale.is_empty() {
                println!("wiki: generated tables are up to date");
                Ok(())
            } else {
                Err(format!(
                    "wiki/src/generated is stale ({}); run `cargo xtask wiki` and commit",
                    stale.join(", ")
                ))
            }
        }
        Mode::Write | Mode::Build => {
            std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            for (name, text) in &files {
                let path = dir.join(name);
                std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
            }
            println!("wiki: wrote {} files to {}", files.len(), dir.display());
            if mode == Mode::Build {
                build_book()?;
            }
            Ok(())
        }
    }
}

fn build_book() -> Result<(), String> {
    let status = Command::new("mdbook")
        .current_dir(workspace_root())
        .args(["build", "wiki"])
        .status()
        .map_err(|e| {
            format!("cannot run mdbook ({e}); install it with `cargo install mdbook --locked`")
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("mdbook exited with {status}"))
    }
}

/// The generated files: name and text, in a fixed order.
fn generate(content: &Content) -> Vec<(String, String)> {
    let mut files = vec![
        ("calendar.md".to_string(), months(content)),
        ("actions.md".to_string(), action_costs(content)),
        ("terrain-local.md".to_string(), local_terrain(content)),
        (
            "terrain-overworld.md".to_string(),
            overworld_terrain(content),
        ),
        (
            "overworld-sight.md".to_string(),
            overworld_sight_table(content),
        ),
        ("moons.md".to_string(), moons(content)),
        ("weather.md".to_string(), weather_chances(content)),
        ("weather-effects.md".to_string(), weather_effects(content)),
        ("vision.md".to_string(), vision(content)),
        ("lights.md".to_string(), lights(content)),
        ("creatures.md".to_string(), creatures(content)),
        ("factions.md".to_string(), factions(content)),
        ("footsteps.md".to_string(), footsteps(content)),
        ("objects.md".to_string(), objects(content)),
        ("equipment.md".to_string(), equipment(content)),
        ("materials.md".to_string(), materials(content)),
        ("pack.md".to_string(), pack(content)),
        ("scenarios.md".to_string(), scenarios(content)),
        ("stats.md".to_string(), stats()),
        ("genders.md".to_string(), genders()),
        ("races.md".to_string(), races(content)),
        ("classes.md".to_string(), classes(content)),
        ("horoscope.md".to_string(), horoscope(content)),
        ("skills.md".to_string(), skills(content)),
        ("skill-benefits.md".to_string(), skill_benefits(content)),
        ("skill-grants.md".to_string(), skill_grants(content)),
        ("skill-increase.md".to_string(), skill_increase(content)),
        ("weapons.md".to_string(), weapons(content)),
        ("launchers.md".to_string(), launchers(content)),
        ("ammunition.md".to_string(), ammunition(content)),
        ("armour.md".to_string(), armour(content)),
        ("stances.md".to_string(), stances(content)),
        ("hit-chances.md".to_string(), hit_chances()),
        ("threats.md".to_string(), threats()),
        ("hit-points.md".to_string(), hit_points(content)),
        ("regeneration.md".to_string(), regeneration()),
        (
            "experience-levels.md".to_string(),
            experience_levels(content),
        ),
        (
            "experience-zd.md".to_string(),
            experience_zero_difference(content),
        ),
        (
            "experience-scaling.md".to_string(),
            experience_scaling(content),
        ),
        (
            "experience-example.md".to_string(),
            experience_example(content),
        ),
        (
            "experience-creatures.md".to_string(),
            experience_creatures(content),
        ),
        ("burden.md".to_string(), burden_levels()),
        ("burden-races.md".to_string(), burden_races(content)),
        ("burden-examples.md".to_string(), burden_examples(content)),
        ("digging.md".to_string(), digging(content)),
        ("digging-strength.md".to_string(), digging_strength(content)),
        ("special-rooms.md".to_string(), special_rooms(content)),
        (
            "weapon-skills.md".to_string(),
            weapon_skills::skills(content),
        ),
        (
            "weapon-defaults.md".to_string(),
            weapon_skills::defaults(content),
        ),
        ("weapon-marks.md".to_string(), weapon_skills::marks(content)),
        (
            "weapon-benefits.md".to_string(),
            weapon_skills::benefits(content),
        ),
        ("traps.md".to_string(), traps(content)),
    ];
    for (_, text) in &mut files {
        *text = format!(
            "<!-- Generated by `cargo xtask wiki` from the game's data. Do not edit. -->\n\n{text}"
        );
    }
    files
}

fn months(content: &Content) -> String {
    let mut out = String::from("| Month | Name |\n|---:|---|\n");
    for month in 1..=12 {
        let time = GameTime::from_calendar(1, month, 1, 0, 0).expect("a valid date");
        let _ = writeln!(out, "| {month} | {} |", content.calendar().month_name(time));
    }
    out
}

fn glyph(c: char) -> String {
    if c == ' ' {
        "(blank)".into()
    } else {
        format!("`{c}`")
    }
}

fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

/// A name for a player-facing table, first letter capitalized.
fn capital(name: &str) -> String {
    let mut chars = name.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}

/// The ids of the terrain the overworld map is drawn with (it has its own table).
fn overworld_ids(content: &Content) -> BTreeSet<String> {
    content
        .levels()
        .filter(|l| l.kind == LevelKind::Overworld)
        .flat_map(|l| l.terrain.cells().iter())
        .map(|t| content.terrain(*t).id.clone())
        .collect()
}

fn is_overworld_terrain(t: &TerrainDef, ids: &BTreeSet<String>) -> bool {
    t.travel_minutes.is_some() || t.landscape.is_some() || ids.contains(&t.id)
}

/// How long an action of energy price `price` takes at normal speed (the player's), in words:
/// `6 seconds`, `5.4 seconds`, `45 minutes`. Speed itself stays hidden.
fn at_normal_pace(price: i64) -> String {
    let speed = i64::from(NORMAL_SPEED);
    let tenths = (price * 10 + speed / 2) / speed;
    if tenths % 10 != 0 && tenths < 600 {
        return format!("{}.{} seconds", tenths / 10, tenths % 10);
    }
    let seconds = u64::try_from(price.div_euclid(speed)).unwrap_or(0);
    Duration::seconds(seconds).spoken()
}

/// What each action costs and how long it takes at a normal pace (spec §8 "Time"), and the
/// creature types whose attacks cost something else.
fn action_costs(content: &Content) -> String {
    let mut out = String::from("| Action | Cost | Takes at a normal pace |\n|---|---:|---:|\n");
    for kind in ActionKind::ALL {
        let what = match kind {
            ActionKind::Step => "A step onto normal ground (other ground: see Terrain)",
            ActionKind::Wait => "Waiting",
            ActionKind::OpenDoor => "Opening a door",
            ActionKind::CloseDoor => "Closing a door",
            ActionKind::Lantern => "Lighting or dousing the lantern",
            ActionKind::Stairs => {
                "Taking stairs, entering a place from the overworld, or walking off an area's edge"
            }
            ActionKind::PickUp => "Picking up (however many stacks)",
            ActionKind::Drop => "Dropping (however many stacks)",
            ActionKind::Melee => "Attacking up close",
            ActionKind::Shoot => "Shooting with a missile weapon",
            ActionKind::Throw => "Throwing",
            ActionKind::Reload => "Reloading a crossbow (after every shot, on top of shooting)",
            ActionKind::Cast => "Casting a spell (creatures)",
            ActionKind::Wield => "Wielding something, or putting it away",
            ActionKind::Wear | ActionKind::BodyArmor => "Putting something on, or taking it off",
            ActionKind::Ready => "Readying something, or putting it away",
            ActionKind::Dig => "A turn of digging (digging through a wall takes many)",
        };
        // Equipment actions say which slots take them (data/slots.toml); a kind no slot uses
        // is left out.
        let what = if kind.is_equipment() {
            let slots = slots_of(content, kind);
            if slots.is_empty() {
                continue;
            }
            format!("{what}: {slots}")
        } else {
            what.to_string()
        };
        let cost = content.action_costs().cost(kind);
        let _ = writeln!(
            out,
            "| {what} | {cost} | {} |",
            at_normal_pace(price(cost, BASE_TURN))
        );
    }
    for c in content.creature_defs() {
        if let Some(cost) = c.attack_cost {
            let _ = writeln!(out, "\nThe {}'s attacks cost {cost} each.", c.name);
        }
    }
    out
}

/// The names of the slots whose action is `kind`, in screen order, lower case after the first:
/// "Head, neck, cloak".
fn slots_of(content: &Content, kind: ActionKind) -> String {
    let names: Vec<String> = (content.slots().iter())
        .filter(|s| s.action == kind)
        .map(|s| s.name.to_lowercase())
        .collect();
    capital(&names.join(", "))
}

/// The equipment slots in the order the screen lists them, with their letters, what goes in
/// each, and how long putting a thing on there or taking it off takes.
fn equipment(content: &Content) -> String {
    let mut out = String::from(
        "| Letter | Slot | What goes there | On or off takes |\n|:---:|---|---|---:|\n",
    );
    for (i, (slot, def)) in content.slot_ids().enumerate() {
        let fits: Vec<String> = (content.object_defs().iter().enumerate())
            .filter(|&(o, _)| content.fits_properly(ObjectId(o as u16), slot))
            .map(|(_, o)| {
                let mut name = o.name.clone();
                if o.two_handed {
                    name.push_str(" (both hands)");
                }
                if let Some(kind) = &o.launches {
                    let ammo = (content.object_defs().iter())
                        .find(|a| a.ammo.as_ref() == Some(kind))
                        .map_or(kind.clone(), |a| a.plural.clone());
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
fn goes_on(content: &Content, object: usize) -> String {
    let id = ObjectId(object as u16);
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

fn local_terrain(content: &Content) -> String {
    let ids = overworld_ids(content);
    let step = content.action_costs().cost(ActionKind::Step);
    let mut out = String::from(
        "| Terrain | Glyph | Walk on it | Blocks sight | Step takes | Footsteps | Height (cells) \
         | Notes |\n|---|:---:|:---:|:---:|---:|:---:|:---:|---|\n",
    );
    for t in content
        .terrain_defs()
        .iter()
        .filter(|t| !is_overworld_terrain(t, &ids))
    {
        let step_time = if t.walkable {
            at_normal_pace(t.step_price(step, LevelKind::Dungeon))
        } else if t.is_closed_door() {
            let open = content.action_costs().cost(ActionKind::OpenDoor);
            format!("{} (opens it)", at_normal_pace(price(open, BASE_TURN)))
        } else {
            "-".into()
        };
        let mut notes = Vec::new();
        if t.is_closed_door() {
            notes.push("door: walking into it opens it".to_string());
        }
        if t.is_open_door() {
            notes.push("door: can be closed".to_string());
        }
        match t.stairs {
            Some(warlock_core::content::StairsDir::Down) => notes.push("stairs down".into()),
            Some(warlock_core::content::StairsDir::Up) => notes.push("stairs up".into()),
            None => {}
        }
        if t.light.is_some() {
            notes.push("gives light".into());
        }
        if t.breaks.is_some() {
            notes.push("can be dug through or blasted open".into());
        }
        if t.roofed {
            notes.push("roofed: no sun or sky light".into());
        }
        match t.muffle {
            None => {}
            Some(0) if t.walkable || t.transparent => {}
            Some(_) => notes.push("muffles sound".into()),
        }
        let footsteps = if t.walkable {
            footsteps_word(t.step_noise)
        } else {
            "-"
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            capital(&t.name),
            glyph(t.glyph),
            yes(t.walkable || t.is_closed_door()),
            yes(!t.transparent),
            step_time,
            footsteps,
            t.height,
            notes.join("; ")
        );
    }
    out
}

/// How loud steps onto terrain are, in words: the exact numbers are rules detail.
fn footsteps_word(step_noise: u16) -> &'static str {
    match step_noise {
        0..=90 => "quiet",
        91..=109 => "normal",
        110..=199 => "loud",
        _ => "very loud",
    }
}

/// Walkable dungeon and outdoor terrain on which steps are louder or quieter than normal,
/// grouped (spec §5 "Hearing"); the overworld is silent.
fn footsteps(content: &Content) -> String {
    let ids = overworld_ids(content);
    let mut out = String::from("| Footsteps | Terrain |\n|---|---|\n");
    for word in ["very loud", "loud", "quiet"] {
        let names: Vec<String> = content
            .terrain_defs()
            .iter()
            .filter(|t| t.walkable && !is_overworld_terrain(t, &ids))
            .filter(|t| footsteps_word(t.step_noise) == word)
            .map(|t| t.name.clone())
            .collect();
        if !names.is_empty() {
            let _ = writeln!(
                out,
                "| {} | {} |",
                capital(word),
                capital(&names.join(", "))
            );
        }
    }
    out
}

fn overworld_terrain(content: &Content) -> String {
    let ids = overworld_ids(content);
    let step = content.action_costs().cost(ActionKind::Step);
    let mut out = String::from(
        "| Terrain | Glyph | Travel per step | Blocks sight | Local area (`>`) |\n\
         |---|:---:|---:|:---:|:---:|\n",
    );
    for t in content
        .terrain_defs()
        .iter()
        .filter(|t| is_overworld_terrain(t, &ids))
    {
        let travel = match t.travel_minutes {
            Some(_) => at_normal_pace(t.step_price(step, LevelKind::Overworld)),
            None => "impassable".to_string(),
        };
        let local = if t.stairs.is_some() {
            "location: `>` enters it"
        } else {
            yes(t.local.is_some())
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            capital(&t.name),
            glyph(t.glyph),
            travel,
            yes(!t.transparent),
            local
        );
    }
    out
}

/// How long digging through a wall of `hardness` takes with a tool of `power` at Strength
/// `strength`, at a normal pace: to the second under ten minutes, else to the nearest minute.
fn dig_time(content: &Content, hardness: u16, power: u16, strength: u8) -> String {
    let turn = price(content.action_costs().cost(ActionKind::Dig), BASE_TURN);
    let turns = i64::from(digs_to_break(hardness, power, strength));
    let seconds = u64::try_from(turn * turns / i64::from(NORMAL_SPEED)).unwrap_or(0);
    let (minutes, rest) = (seconds / 60, seconds % 60);
    match (minutes, rest) {
        (0, _) | (_, 0) => Duration::seconds(seconds).spoken(),
        (1..10, _) => format!(
            "{} {}",
            Duration::minutes(minutes).spoken(),
            Duration::seconds(rest).spoken()
        ),
        _ => Duration::minutes((seconds + 30) / 60).spoken(),
    }
}

/// The walls that break (spec §8 "Digging"): how hard each is, how long each digging tool takes
/// through it at average Strength, and what it becomes.
fn digging(content: &Content) -> String {
    let tools: Vec<(&str, u16)> = (content.object_defs().iter())
        .filter_map(|o| o.dig.map(|power| (o.name.as_str(), power)))
        .collect();
    let blasts: Vec<(&str, u16)> = (content.trap_defs().iter())
        .filter_map(|t| match t.effect {
            TrapEffect::Burst { blast, .. } if blast > 0 => Some((t.noun.as_str(), blast)),
            _ => None,
        })
        .collect();
    let mut out = String::from("| Wall | Glyph | Hardness |");
    for (name, _) in &tools {
        let _ = write!(out, " With {} {name} |", article(name));
    }
    for (name, _) in &blasts {
        let _ = write!(out, " {} {name} breaks it |", capital(article(name)));
    }
    out.push_str(" Becomes |\n|---|:---:|---:|");
    out.push_str(&"---:|".repeat(tools.len()));
    out.push_str(&":---:|".repeat(blasts.len()));
    out.push_str("---|\n");
    for t in content.terrain_defs() {
        let Some(breaks) = t.breaks else { continue };
        let _ = write!(
            out,
            "| {} | {} | {} |",
            capital(&t.name),
            glyph(t.glyph),
            breaks.hardness
        );
        for (_, power) in &tools {
            let _ = write!(
                out,
                " {} |",
                dig_time(content, breaks.hardness, *power, AVERAGE)
            );
        }
        for (_, blast) in &blasts {
            let _ = write!(out, " {} |", yes(breaks.hardness < *blast));
        }
        let _ = writeln!(out, " {} |", content.terrain(breaks.becomes).name);
    }
    out
}

/// How Strength changes digging: the first digging tool through each wall that breaks, at a
/// few Strength scores.
fn digging_strength(content: &Content) -> String {
    let Some((tool, power)) =
        (content.object_defs().iter()).find_map(|o| o.dig.map(|power| (o.name.as_str(), power)))
    else {
        return String::new();
    };
    let walls: Vec<&TerrainDef> = (content.terrain_defs().iter())
        .filter(|t| t.breaks.is_some())
        .collect();
    let mut out = String::from("| Strength |");
    for t in &walls {
        let _ = write!(out, " {} |", capital(&t.name));
    }
    out.push_str("\n|---:|");
    out.push_str(&"---:|".repeat(walls.len()));
    out.push('\n');
    for strength in [3, 6, 10, 14, 18, 25] {
        let _ = write!(out, "| {strength} |");
        for t in &walls {
            let hardness = t.breaks.map_or(0, |b| b.hardness);
            let _ = write!(out, " {} |", dig_time(content, hardness, power, strength));
        }
        out.push('\n');
    }
    let _ = writeln!(out, "\nWith {} {tool}, at a normal pace.", article(tool));
    out
}

/// A share of full light (255) in whole percent, rounded.
fn percent_of_full(luma: u16) -> u32 {
    (u32::from(luma) * 100 + 127) / 255
}

/// The light of the brightest moon when full and high, if there are moons.
fn brightest_moon(content: &Content) -> Option<Light> {
    content
        .moons()
        .iter()
        .map(Moon::brightest)
        .max_by_key(|l| l.luma())
}

fn overworld_sight_table(content: &Content) -> String {
    let mut out = String::from("| When | Sight on the overworld |\n|---|---:|\n");
    let _ = writeln!(out, "| Day (06:30 to 17:30) | {SIGHT_DAY} cells |");
    let _ = writeln!(
        out,
        "| Twilight (half an hour either side of 06:00 and 18:00) | {SIGHT_TWILIGHT} cells |"
    );
    let _ = writeln!(out, "| Night, no moonlight | {SIGHT_NIGHT} cells |");
    let midnight = GameTime::from_calendar(1, 1, 1, 0, 0).expect("a valid date");
    if let Some(full) = brightest_moon(content) {
        let half = Light::new(full.rgb[0] / 2, full.rgb[1] / 2, full.rgb[2] / 2, 0);
        for (name, light) in [("half", half), ("full", full)] {
            let sight = overworld_sight(midnight, light, 0);
            let _ = writeln!(
                out,
                "| Night, a {name} moon high in the sky | {sight} cells |"
            );
        }
    }
    // Clouds dim the moon, so they shorten moonlit sight even when they take no cells away.
    if !content.moons().is_empty() {
        let full_moon = GameTime::from_calendar(1, 1, 16, 0, 0).expect("a valid date");
        for weather in [Weather::Overcast, Weather::Rain, Weather::HeavyRain] {
            let def = content.climate().def(weather);
            let conditions = Conditions {
                weather,
                clouds: Clouds::of(def),
            };
            let moonlight = Sky::at(full_moon, content.moons(), conditions).moonlight();
            let sight = overworld_sight(full_moon, moonlight, def.sight);
            let under = match weather {
                Weather::Overcast => "overcast".to_string(),
                w => format!("in {}", w.name().to_lowercase()),
            };
            let _ = writeln!(
                out,
                "| Night, a full moon high in the sky, {under} | {sight} cells |"
            );
        }
    }
    let noon = GameTime::from_calendar(1, 1, 1, 12, 0).expect("a valid date");
    for weather in [Weather::Rain, Weather::HeavyRain] {
        let lost = content.climate().def(weather).sight;
        if lost > 0 {
            let sight = overworld_sight(noon, Light::DARK, lost);
            let name = weather.name().to_lowercase();
            let _ = writeln!(out, "| Day, in {name} | {sight} cells |");
        }
    }
    let _ = writeln!(
        out,
        "\nAt night moonlight (all moons together) adds a cell of sight for every {}% of full light \
         it gives, but by moonlight you never see farther than {SIGHT_MOONLIT} cells. Clouds dim \
         the moon, and rain takes cells away at any hour (see [the weather](weather.md)), but you \
         always see at least {SIGHT_NIGHT}. The farthest you can ever see is {VIEW_RADIUS} cells.",
        percent_of_full(MOONLIGHT_PER_CELL)
    );
    out
}

/// The chance of each weather in each month.
fn weather_chances(content: &Content) -> String {
    let mut out = String::from("| Month |");
    for w in Weather::ALL {
        let _ = write!(out, " {} |", w.name());
    }
    out.push_str("\n|---|---:|---:|---:|---:|\n");
    for month in 1..=12 {
        let time = GameTime::from_calendar(1, month, 1, 0, 0).expect("a valid date");
        let _ = write!(out, "| {} |", content.calendar().month_name(time));
        for chance in content.climate().chances(month as usize - 1) {
            let _ = write!(out, " {chance}% |");
        }
        out.push('\n');
    }
    out
}

/// A share of `of` in whole percent, rounded.
fn share(part: u16, of: u16) -> u32 {
    (u32::from(part) * 100 + u32::from(of) / 2) / u32::from(of.max(1))
}

/// What each weather does to the light and to overworld sight.
fn weather_effects(content: &Content) -> String {
    let mut out = String::from(
        "| Weather | Light at noon, in the open | In the shade | A full moon's light | \
         Overworld sight lost |\n|---|---:|---:|---:|---:|\n",
    );
    let noon = GameTime::from_calendar(1, 1, 1, 12, 0).expect("a valid date");
    let full_moon = GameTime::from_calendar(1, 1, 16, 0, 0).expect("a valid date");
    let clear = Sun::at(noon);
    let open = |sun: &Sun| sun.light.saturating_add(sun.sky).luma();
    let moonlight = |w: Weather| {
        let conditions = Conditions {
            weather: w,
            clouds: Clouds::of(content.climate().def(w)),
        };
        Sky::at(full_moon, content.moons(), conditions)
            .moonlight()
            .luma()
    };
    for w in Weather::ALL {
        let def = content.climate().def(w);
        let clouds = Clouds::of(def);
        let sun = Sun {
            light: clouds.direct(clear.light),
            sky: clouds.sky(clear.sky, clear.light),
            ..clear
        };
        let moon = if content.moons().is_empty() {
            "no moon".to_string()
        } else {
            format!("{}%", share(moonlight(w), moonlight(Weather::Clear)))
        };
        let sight = match def.sight {
            0 => "none".to_string(),
            1 => "1 cell".to_string(),
            n => format!("{n} cells"),
        };
        let _ = writeln!(
            out,
            "| {} | {}% | {}% | {moon} | {sight} |",
            w.name(),
            share(open(&sun), open(&clear)),
            share(sun.sky.luma(), clear.sky.luma()),
        );
    }
    let _ = writeln!(
        out,
        "\nPercentages are of the light of a clear sky at the same moment. The shade is where the \
         sun does not reach but the sky does; the brighter it is next to the open ground, the \
         fainter the shadows. The last column is what the weather itself takes from your sight \
         on the overworld; at night its clouds also dim the moon, and with it how far you see \
         by moonlight (see [the overworld](overworld.md))."
    );
    out
}

fn moons(content: &Content) -> String {
    if content.moons().is_empty() {
        return "There are no moons: nights are dark.\n".into();
    }
    let mut out = String::from(
        "| Moon | From new moon to new moon | Size | Light when full and high | First new moon |\n\
         |---|---:|---|---:|---|\n",
    );
    for moon in content.moons() {
        let size = if moon.size == 100 {
            "standard".to_string()
        } else {
            format!("{}% of the standard", moon.size)
        };
        let first = (moon.period_days - moon.phase_offset_days) % moon.period_days;
        let first = GameTime::from_seconds(u64::from(first) * DAY);
        let _ = writeln!(
            out,
            "| {} | {} days | {size} | about {}% of full light | {} |",
            moon.name,
            moon.period_days,
            percent_of_full(Moon::brightest(moon).luma()),
            content.calendar().full_date(first)
        );
    }
    out
}

fn vision(content: &Content) -> String {
    let mut out = String::from(
        "| Vision | Needs a cell lit to at least | Sees in the dark within | Sees UV light |\n\
         |---|---:|---:|:---:|\n",
    );
    for v in content.vision_profiles() {
        // Full light is 255 per channel.
        let percent = (u32::from(v.threshold) * 100).div_ceil(255);
        let dark = if v.darkvision > 0 {
            format!("{} cells", v.darkvision)
        } else {
            "-".into()
        };
        let _ = writeln!(
            out,
            "| {} | about {percent}% of full light | {dark} | {} |",
            v.name,
            yes(v.sees_uv)
        );
    }
    out
}

fn lights(content: &Content) -> String {
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
    let carriers: Vec<String> = (content.creature_defs().iter())
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
    let pedestal = (content.terrain_defs().iter())
        .filter(|t| !t.walkable && t.light.is_none())
        .find_map(|t| Some((t, preset.id.strip_prefix(&format!("{}_", t.id))?)));
    match pedestal {
        Some((t, kind)) => format!("{} ({})", capital(&t.name), words(kind)),
        None => capital(&words(&preset.id)),
    }
}

/// `a` or `an` for a noun.
fn article(noun: &str) -> &'static str {
    match noun.chars().next() {
        Some(c) if "aeiouAEIOU".contains(c) => "an",
        _ => "a",
    }
}

/// The scenarios a new run can start, as the New run screen lists them (the showcase first), with
/// the moment each starts.
fn scenarios(content: &Content) -> String {
    let mut list: Vec<_> = content.scenarios().collect();
    list.sort_by_key(|s| s.id != "showcase");
    let mut out = String::from("| Scenario | Starts at | What it is |\n|---|---|---|\n");
    for s in list {
        let _ = writeln!(
            out,
            "| {} | {:02}:{:02}, {} | {} |",
            s.name,
            s.start.hour(),
            s.start.minute(),
            content.calendar().full_date(s.start),
            table_text(&s.description)
        );
    }
    out
}

/// Free text for a table cell: command-line flags (`--seed`) as code, so they keep both hyphens,
/// and `|` escaped.
fn table_text(text: &str) -> String {
    text.split(' ')
        .map(|word| match word.strip_prefix("--") {
            Some(flag) if !flag.is_empty() => format!("`--{flag}`"),
            _ => word.replace('|', "\\|"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Speed, eyes, and sight radius are deliberately left out: they are hidden from the player (spec
/// §8 "Speed"); how a creature stands toward the player and how it attacks are seen in play.
fn creatures(content: &Content) -> String {
    let mut out = String::from(
        "| Creature | Glyph | Faction | Toward you | Attacks | Opens doors | Carries a light |\n\
         |---|:---:|---|---|---|:---:|:---:|\n",
    );
    let player = content.creature_id("player");
    for (i, c) in content.creature_defs().iter().enumerate() {
        if c.id == "player" {
            continue;
        }
        let id = CreatureId(i as u16);
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
        let _ = writeln!(
            out,
            "| {} | {} | {faction} | {stance} | {attacks} | {} | {} |",
            capital(&c.name),
            glyph(c.glyph),
            yes(c.opens_doors),
            yes(c.light.is_some())
        );
    }
    out
}

/// What a glyph means for objects (ADOM-like classes); `None` for a glyph no class uses.
fn object_class(glyph: char) -> Option<&'static str> {
    Some(match glyph {
        ')' => "weapon",
        '}' => "missile weapon",
        '/' => "missile",
        '[' => "armor",
        '(' => "tool",
        '!' => "potion",
        '?' => "scroll",
        '%' => "food",
        '*' => "stone or gem",
        '$' => "coin",
        '=' => "ring",
        '"' => "amulet",
        _ => return None,
    })
}

fn objects(content: &Content) -> String {
    let mut out = String::from(
        "| Object | Glyph | Kind | Material | Weight (each) | Stacks | Worn or wielded on |\n\
         |---|:---:|---|---|---:|:---:|---|\n",
    );
    for (i, o) in content.object_defs().iter().enumerate() {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} |",
            capital(&o.name),
            glyph(o.glyph),
            object_class(o.glyph).unwrap_or("-"),
            content.material(o.material).name,
            format_weight(u64::from(o.weight)),
            yes(o.stackable),
            goes_on(content, i)
        );
    }
    out
}

fn materials(content: &Content) -> String {
    let mut out = String::from("| Material | Objects made of it |\n|---|---|\n");
    for (i, m) in content.materials().iter().enumerate() {
        let made: Vec<&str> = content
            .object_defs()
            .iter()
            .filter(|o| usize::from(o.material.0) == i)
            .map(|o| o.plural.as_str())
            .collect();
        let made = if made.is_empty() {
            "-".to_string()
        } else {
            made.join(", ")
        };
        let _ = writeln!(out, "| {} | {made} |", capital(&m.name));
    }
    out
}

/// Who fights whom: each faction and the factions hostile to it (relations are symmetric).
fn factions(content: &Content) -> String {
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

/// The pack's size and the player's own weight.
fn pack(content: &Content) -> String {
    let player = content
        .creature_id("player")
        .map_or(0, |id| content.creature(id).weight);
    let last = char::from(b'a' + (INVENTORY_SLOTS - 1) as u8);
    format!(
        "| Fact | Value |\n|---|---|\n\
         | Stacks your pack holds | {INVENTORY_SLOTS}, lettered `a` to `{last}` |\n\
         | Your own weight | {} |\n",
        format_weight(u64::from(player))
    )
}

/// The stats in the order the character screen lists them, with their abbreviations. Only the
/// names: never a modifier or a potential, nor anyone's scores.
fn stats() -> String {
    let mut out = String::from("| Stat | Short |\n|---|:---:|\n");
    for stat in Stat::ALL {
        let _ = writeln!(out, "| {} | `{}` |", stat.name(), stat.abbrev());
    }
    let _ = writeln!(out, "\nA score of {AVERAGE} is average.");
    out
}

/// What each gender raises (spec §8 "Character creation").
fn genders() -> String {
    let mut out = String::from("| Gender | Starting stats |\n|---|---|\n");
    for gender in Gender::ALL {
        let raises = match gender {
            Gender::Male => format!("{} +1", Stat::Strength.name()),
            Gender::Female => format!("{} +1", Stat::Dexterity.name()),
            Gender::Other => "+1 to one stat, picked by the run's seed".to_string(),
        };
        let _ = writeln!(out, "| {} | {raises} |", gender.name());
    }
    out
}

/// A stat adjustment in a table, as written in data: `+2`, `0`, `1d3+1`, and with a minus sign
/// (not a hyphen) `−1`, `−1d2`. Rules, not modifiers.
fn adjustment(v: Dice) -> String {
    v.to_string().replace('-', "\u{2212}")
}

/// The stat columns' header: `| St | Dx | …` and its alignment row.
fn stat_columns() -> (String, String) {
    let names: Vec<String> = Stat::ALL
        .iter()
        .map(|s| format!(" {} |", s.abbrev()))
        .collect();
    (names.concat(), ":---:|".repeat(Stat::COUNT))
}

/// One adjustment per stat, as table cells.
fn stat_cells(by: &PerStat<Dice>) -> String {
    by.iter()
        .map(|(_, v)| format!(" {} |", adjustment(v)))
        .collect()
}

/// The races in the order New run offers them: their stat adjustments, vision, and starting
/// age, then what they are. Never their potentials.
fn races(content: &Content) -> String {
    let (names, align) = stat_columns();
    let mut out = format!(
        "| Race |{names} Vision | Starting age | Hit points | Weapon skills |\n\
         |---|{align}---|---|:---:|---|\n"
    );
    for r in &content.races().list {
        let vision = content
            .vision(&r.vision)
            .map_or(r.vision.as_str(), |v| &v.name);
        let _ = writeln!(
            out,
            "| {} |{} {vision} | {}–{} | {} | {} |",
            r.name,
            stat_cells(&r.stats),
            r.age.start(),
            r.age.end(),
            r.hp,
            weapon_skills::ranks_text(content, &r.weapon_skills),
        );
    }
    out.push('\n');
    for r in &content.races().list {
        let _ = writeln!(out, "- **{}**: {}", r.name, r.description);
    }
    out
}

/// The classes in the order New run offers them: their stat adjustments, then what they are.
fn classes(content: &Content) -> String {
    let (names, align) = stat_columns();
    let mut out =
        format!("| Class |{names} Hit points | Weapon skills |\n|---|{align}:---:|---|\n");
    for c in content.classes() {
        let _ = writeln!(
            out,
            "| {} |{} {} | {} |",
            c.name,
            stat_cells(&c.stats),
            c.hp,
            weapon_skills::class_ranks(content, c)
        );
    }
    out.push('\n');
    for c in content.classes() {
        let _ = writeln!(out, "- **{}**: {}", c.name, c.description);
    }
    out
}

/// Every skill in the order the Skills screen lists them: what kind it is, what it is about,
/// what it gives (the benefits table has the numbers), and when it earns a training mark (with
/// the chance).
fn skills(content: &Content) -> String {
    let mut out = String::from(
        "| Skill | Kind | What it is about | Gives | Trains on |\n|---|---|---|---|---|\n",
    );
    for skill in content.skills() {
        let gives: Vec<&str> = skill
            .benefits
            .iter()
            .map(|b| effect_name(b.effect))
            .collect();
        let gives = if gives.is_empty() {
            "nothing yet".to_string()
        } else {
            gives.join("; ")
        };
        let trains: Vec<String> = (skill.marks.iter())
            .map(|m| format!("{} ({}%)", m.on.describe(), m.chance))
            .collect();
        let trains = if trains.is_empty() {
            "nothing yet".to_string()
        } else {
            trains.join("; ")
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {gives} | {trains} |",
            skill.name,
            skill.kind.name(),
            skill.description
        );
    }
    out
}

/// What an effect does, in words.
fn effect_name(effect: Effect) -> &'static str {
    match effect {
        Effect::Dodge => "a Dodge bonus to Defence",
        Effect::Quiet => "quieter steps and doors",
        Effect::Hearing => "keener hearing",
        Effect::Burden => "less slowdown from your load",
        Effect::Travel => "quicker journeys on the overworld",
    }
}

/// The values the benefits table shows.
const BENEFIT_VALUES: [u16; 14] = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 125, 150, 200];

/// What each skill's benefits give at a range of values, one column per benefit, from the data.
fn skill_benefits(content: &Content) -> String {
    let columns: Vec<(&str, &warlock_core::skills::Benefit)> = (content.skills().iter())
        .flat_map(|s| s.benefits.iter().map(move |b| (s.name.as_str(), b)))
        .collect();
    let mut out = String::from("| Value |");
    for (name, b) in &columns {
        let what = match b.effect {
            Effect::Dodge => "Dodge",
            Effect::Quiet => "noise",
            Effect::Hearing => "hears fainter by",
            Effect::Burden => "Burdened / Strained",
            Effect::Travel => "journeys",
        };
        let _ = write!(out, " {name}: {what} |");
    }
    out.push_str("\n|---:|");
    out.push_str(&":---:|".repeat(columns.len()));
    out.push('\n');
    for value in BENEFIT_VALUES {
        let _ = write!(out, "| {value} |");
        for (_, b) in &columns {
            let n = b.at(value);
            let cell = match b.effect {
                _ if n == 0 => "—".to_string(),
                Effect::Dodge => format!("+{n}"),
                Effect::Quiet | Effect::Travel => format!("\u{2212}{n}%"),
                Effect::Hearing => n.to_string(),
                Effect::Burden => {
                    let slowed = |b: Burden| factor(b.slow_eased(100, n));
                    format!(
                        "×{} / ×{}",
                        slowed(Burden::Burdened),
                        slowed(Burden::Strained)
                    )
                }
            };
            let _ = write!(out, " {cell} |");
        }
        out.push('\n');
    }
    out
}

/// A skill's name, for the grants tables.
fn skill_name(content: &Content, id: SkillId) -> &str {
    &content.skill(id).name
}

/// What each race and each class trains, as dice (`1d6+4`), in the order New run offers them.
fn skill_grants(content: &Content) -> String {
    let list = |grants: &[(SkillId, Dice)]| -> String {
        let parts: Vec<String> = (grants.iter())
            .map(|&(id, dice)| format!("{} `{dice}`", skill_name(content, id)))
            .collect();
        parts.join(", ")
    };
    let mut out = String::from("| Race | Trains |\n|---|---|\n");
    for r in &content.races().list {
        let mut trains = list(&r.skills);
        if !r.class_skill.is_zero() {
            if !trains.is_empty() {
                trains.push_str("; ");
            }
            let _ = write!(
                trains,
                "one of the class's skills `+{}`, picked at random",
                r.class_skill
            );
        }
        if trains.is_empty() {
            trains = "-".into();
        }
        let _ = writeln!(out, "| {} | {trains} |", r.name);
    }
    out.push_str("\n| Class | Trains |\n|---|---|\n");
    for c in content.classes() {
        let _ = writeln!(out, "| {} | {} |", c.name, list(&c.skills));
    }
    out
}

/// **[provisional]** The increase curve: the marks a point needs at a range of values, and all
/// the marks from 0.
fn skill_increase(content: &Content) -> String {
    let inc = content.increase();
    let mut out =
        String::from("| Value | Marks for the next point | Marks from 0 |\n|---:|---:|---:|\n");
    for value in [0, 10, 25, 50, 75, 99, 100, 101, 110, 120, 150, 200] {
        let _ = writeln!(
            out,
            "| {value} | {} | {} |",
            inc.need(value),
            inc.total(0, value)
        );
    }
    out
}

/// The horoscope: the animals in the order of the Great Race and the elements, each with its
/// stat adjustments and what it means; then the great cycle, sign by sign, with its years in the
/// great cycle of the showcase's start and the two before it.
fn horoscope(content: &Content) -> String {
    let Some(h) = content.horoscope() else {
        return "There is no horoscope.\n".into();
    };
    let (names, align) = stat_columns();
    let mut out =
        format!("## Animals\n\n| | Animal |{names} Those born under it |\n|---:|---|{align}---|\n");
    for (i, a) in h.animals.iter().enumerate() {
        let _ = writeln!(
            out,
            "| {} | {} |{} {} |",
            i + 1,
            a.name,
            stat_cells(&a.stats),
            table_text(&a.description)
        );
    }
    let _ = write!(
        out,
        "\n## Elements\n\n| Element |{names} Those born under it |\n|---|{align}---|\n"
    );
    for e in &h.elements {
        let _ = writeln!(
            out,
            "| {} |{} {} |",
            e.name,
            stat_cells(&e.stats),
            table_text(&e.description)
        );
    }
    let cycle = h.great_cycle() as i64;
    let showcase = content
        .scenario("showcase")
        .map_or(1, |s| s.start.year() as i64);
    // The first year of the great cycle the showcase starts in.
    let current = showcase - (showcase - h.anchor_year).rem_euclid(cycle);
    let starts = [current - 2 * cycle, current - cycle, current];
    let _ = write!(
        out,
        "\n## Every sign\n\nEvery sign of the great cycle in order, with its years in the cycles \
         that began in {}, {}, and {}. Add or take away {cycle} years for any other cycle.\n\n\
         | | Sign | Years |\n|---:|---|---|\n",
        starts[0], starts[1], starts[2]
    );
    for i in 0..cycle {
        let sign = h.sign_of_year(current + i);
        let years: Vec<String> = starts.iter().map(|s| (s + i).to_string()).collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            i + 1,
            h.name(&sign),
            years.join(", ")
        );
    }
    out
}

/// The natural rolls of a threat range: `22–24`, or `24` alone.
fn threat_range(threat: u8) -> String {
    if i32::from(threat) >= combat::SURE_HIT {
        combat::SURE_HIT.to_string()
    } else {
        format!("{threat}–{}", combat::SURE_HIT)
    }
}

/// `part` of `of` as a percentage, one decimal below 10 %, whole above.
fn percent(part: u32, of: u32) -> String {
    let p = f64::from(part) * 100.0 / f64::from(of);
    if p < 10.0 && p.fract() != 0.0 {
        format!("{p:.1}%")
    } else {
        format!("{p:.0}%")
    }
}

/// The weapons and missile weapons: their damage dice, critical (threat range and multiplier),
/// parry, the Strength a bow needs, and how they are held. Rules from data, nothing about who
/// wields them.
fn weapons(content: &Content) -> String {
    let mut out = String::from(
        "| Weapon | Damage | Critical on | Critical damage | Parry | Needs Strength | Held | \
         Thrown | Weapon skill |\n|---|:---:|:---:|:---:|:---:|:---:|---|:---:|---|\n",
    );
    for (i, o) in content.object_defs().iter().enumerate() {
        let Some(attack) = &o.attack else { continue };
        let parry = if o.parry > 0 {
            format!("+{}", o.parry)
        } else {
            "-".into()
        };
        let strength = attack.strength.map_or("-".to_string(), |s| s.to_string());
        let held = if o.launches.is_some() {
            format!("{}, shoots", goes_on(content, i).to_lowercase())
        } else {
            goes_on(content, i).to_lowercase()
        };
        let thrown = match o.range {
            Some(range) if o.is_thrown() => format!("{range} cells"),
            _ => "-".into(),
        };
        let skill = (o.skill).map_or("-".to_string(), |id| {
            content.weapon_skills().skill(id).name.clone()
        });
        let _ = writeln!(
            out,
            "| {} | {} | {} | ×{} | {parry} | {strength} | {held} | {thrown} | {skill} |",
            capital(&o.name),
            attack.damage,
            threat_range(attack.threat),
            attack.crit,
        );
    }
    let _ = writeln!(
        out,
        "\nBare hands: {} (plus your Strength, as every blow), critical on {}, ×{}.",
        combat::Attack::unarmed().damage,
        threat_range(combat::DEFAULT_THREAT),
        combat::DEFAULT_CRIT
    );
    out
}

/// What a launcher kind of ammunition is called: every object of that kind, in the plural, in
/// data order ("arrows, flaming arrows").
fn ammunition_of(content: &Content, kind: &str) -> String {
    let names: Vec<&str> = (content.object_defs().iter())
        .filter(|o| o.ammo.as_deref() == Some(kind))
        .map(|o| o.plural.as_str())
        .collect();
    names.join(", ")
}

/// The missile weapons (ranged combat): their damage dice, critical, the Strength a bow needs,
/// how far they shoot, what they shoot, and how long a shot takes.
fn launchers(content: &Content) -> String {
    let mut out = String::from(
        "| Missile weapon | Damage | Critical on | Critical damage | Needs Strength | Range | \
         Shoots | A shot takes |\n|---|:---:|:---:|:---:|:---:|:---:|---|---:|\n",
    );
    let costs = content.action_costs();
    for o in content.object_defs() {
        let (Some(attack), Some(kind)) = (&o.attack, &o.launches) else {
            continue;
        };
        let strength = attack.strength.map_or("-".to_string(), |s| s.to_string());
        let mut cost = costs.cost(ActionKind::Shoot);
        if o.reloads {
            cost += costs.cost(ActionKind::Reload);
        }
        let takes = at_normal_pace(price(cost, BASE_TURN));
        let takes = if o.reloads {
            format!("{takes} (reloading)")
        } else {
            takes
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | ×{} | {strength} | {} cells | {} | {takes} |",
            capital(&o.name),
            attack.damage,
            threat_range(attack.threat),
            attack.crit,
            o.range.unwrap_or(0),
            capital(&ammunition_of(content, kind)),
        );
    }
    out
}

/// The ammunition (what shoots it, its chance to break on a hit, whether it burns where it
/// lands) and the throwing weapons (their damage, critical, and how far they are thrown).
fn ammunition(content: &Content) -> String {
    let mut out = String::from(
        "| Ammunition | Shot from | Breaks on a hit | Where it lands |\n|---|---|:---:|---|\n",
    );
    for o in content.object_defs() {
        let Some(kind) = &o.ammo else { continue };
        let from: Vec<&str> = (content.object_defs().iter())
            .filter(|l| l.launches.as_ref() == Some(kind))
            .map(|l| l.name.as_str())
            .collect();
        let breaks = if o.breaks > 0 {
            format!("{}%", o.breaks)
        } else {
            "never".into()
        };
        let lands = match o.lit.map(|l| content.object(l)) {
            Some(lit) => match lit.burning {
                Some(burning) => format!(
                    "burns as {} for {}, lighting {} cells around",
                    lit.describe(1),
                    burning.lasts.spoken(),
                    content.light(burning.light).radius
                ),
                None => "-".into(),
            },
            None => "lies there, to be picked up".into(),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {breaks} | {lands} |",
            capital(&o.name),
            capital(&from.join(", ")),
        );
    }
    out.push_str(
        "\n| Throwing weapon | Damage | Critical on | Critical damage | Range | Breaks on a hit \
         |\n|---|:---:|:---:|:---:|:---:|:---:|\n",
    );
    for o in content.object_defs() {
        let (Some(attack), Some(range), true) = (&o.attack, o.range, o.is_thrown()) else {
            continue;
        };
        let breaks = if o.breaks > 0 {
            format!("{}%", o.breaks)
        } else {
            "never".into()
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | ×{} | {range} cells | {breaks} |",
            capital(&o.name),
            attack.damage,
            threat_range(attack.threat),
            attack.crit,
        );
    }
    out
}

/// Armour and shields: what each adds to Defence, in which category, and the most Dexterity
/// may add while it is worn.
fn armour(content: &Content) -> String {
    let mut out = String::from(
        "| Armour or shield | Worn on | Defence | Category | Most Dexterity adds | Bash |\n\
         |---|---|:---:|---|:---:|:---:|\n",
    );
    for (i, o) in content.object_defs().iter().enumerate() {
        let (bonus, category) = match (o.armour, o.shield) {
            (0, 0) => continue,
            (a, s) if a >= s => (a, "Armour"),
            (_, s) => (s, "Shield"),
        };
        let max_dex = o
            .max_dex
            .map_or("no limit".to_string(), |m| format!("+{m}"));
        let bash = (o.bash).map_or("-".to_string(), |a| a.damage.to_string());
        let _ = writeln!(
            out,
            "| {} | {} | +{bonus} | {category} | {max_dex} | {bash} |",
            capital(&o.name),
            goes_on(content, i).to_lowercase(),
        );
    }
    out
}

/// The combat stances: what each does to the player's attacks, Defence, and speed (percent for
/// costs and noise; "-" for no change).
fn stances(content: &Content) -> String {
    let bonus = |n: i32| match n {
        0 => "-".to_string(),
        n if n > 0 => format!("+{n}"),
        n => format!("\u{2212}{}", -n),
    };
    let percent = |n: u32| {
        if n == 100 {
            "-".to_string()
        } else {
            format!("{n}%")
        }
    };
    let mut out = String::from(
        "| Stance | To hit | Damage | Dodge | Parry | Attacks cost | Fleeing steps cost | \
         Fleeing steps' noise | Weapon marks count |\n\
         |---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|\n",
    );
    for s in content.stances().all() {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            s.name,
            bonus(s.to_hit),
            bonus(s.damage),
            bonus(s.dodge),
            bonus(s.parry),
            percent(s.attack_cost),
            percent(s.flee_cost),
            percent(s.flee_noise),
            percent(s.weapon_mark),
        );
    }
    out
}

/// The traps (spec §8 "Traps"): what each does, its damage dice, how often it goes off, and what
/// it sounds like. Never a to-hit, which the player does not know.
fn traps(content: &Content) -> String {
    let mut out = String::from(
        "| Trap | What it does | Damage | Goes off | Sound |\n|---|---|:---:|---|---|\n",
    );
    for t in content.trap_defs() {
        let what = match &t.effect {
            TrapEffect::Shot { object, .. } => format!(
                "{} shoots out of a hidden slit in the nearest wall at whoever steps on it: an \
                 attack against their Defence; it lands where it falls",
                capital(&article_noun(&content.object(*object).name))
            ),
            TrapEffect::Strike { .. } => format!(
                "{}s spring from the floor: an attack against the Defence of whoever steps on it",
                capital(&t.noun)
            ),
            TrapEffect::Burst { radius, .. } => format!(
                "A blast {radius} cells around it (walls stop it): everyone in it takes the \
                 damage, half at the edge; no roll to hit"
            ),
            TrapEffect::Bolt { length, .. } => format!(
                "A bolt from it along a line until a wall, up to {length} cells: everyone on it \
                 takes the damage; no roll to hit"
            ),
            TrapEffect::Pit { .. } => {
                "Whoever steps on it falls in and loses their next action climbing out".into()
            }
            TrapEffect::Alarm => "Only its noise: creatures that hear it come to look".into(),
            TrapEffect::Teleport => {
                "Whoever steps on it is moved to a free place on the level".into()
            }
        };
        let damage = t.effect.damage().map_or("—".to_string(), |d| d.to_string());
        let goes_off = match t.charges {
            None => "every time".to_string(),
            Some(1) => "once".to_string(),
            Some(2) => "twice".to_string(),
            Some(n) => format!("{n} times"),
        };
        let sound = t.noise.map_or("Silent".to_string(), |n| {
            let loud = match n.loudness {
                0..50 => "quiet",
                50..150 => "loud",
                150..250 => "very loud",
                _ => "deafening",
            };
            format!("{}, {loud}", capital(&n.sound.name().replace('_', " ")))
        });
        let _ = writeln!(
            out,
            "| {} `{}` | {what} | {damage} | {goes_off} | {sound} |",
            capital(&t.name),
            t.glyph
        );
    }
    out
}

/// A noun with its article: "an arrow".
fn article_noun(noun: &str) -> String {
    format!("{} {noun}", article(noun))
}

/// How often an attack hits by what its roll needs (Defence minus to-hit): the 2d12 bell.
fn hit_chances() -> String {
    let mut out = String::from("| The roll needs | Hits |\n|:---:|:---:|\n");
    for needed in [4, 6, 8, 10, 11, 12, 13, 14, 15, 16, 18, 20, 22, 24] {
        let _ = writeln!(
            out,
            "| {needed} | {} |",
            percent(combat::hit_ways(0, needed), 144)
        );
    }
    out
}

/// How often an attack roll threatens a critical, by threat range, and how often one confirms
/// against Defence 10 at +0.
fn threats() -> String {
    let mut out = String::from(
        "| Critical on | A roll threatens | Confirmed, with no bonus against Defence 10 | Like D&D's |\n\
         |:---:|:---:|:---:|:---:|\n",
    );
    for (threat, dnd) in [(22, "20"), (20, "19–20"), (19, "18–20")] {
        let threatens: u32 = (threat..=combat::SURE_HIT).map(combat::ways).sum();
        let hit = combat::hit_ways(0, combat::BASE_DEFENCE);
        let confirmed: u32 = (threat..=combat::SURE_HIT)
            .filter(|&n| combat::hits(n, 0, combat::BASE_DEFENCE))
            .map(combat::ways)
            .sum::<u32>()
            * hit;
        let _ = writeln!(
            out,
            "| {} | {} | {} | {dnd} |",
            threat_range(threat as u8),
            percent(threatens, 144),
            percent(confirmed, 144 * 144),
        );
    }
    out
}

/// What each race and each class adds to the hit points.
fn hit_points(content: &Content) -> String {
    let mut out = String::from("| Race | Hit points |\n|---|:---:|\n");
    for r in &content.races().list {
        let _ = writeln!(out, "| {} | {} |", r.name, r.hp);
    }
    out.push_str("\n| Class | Hit points |\n|---|:---:|\n");
    for c in content.classes() {
        let _ = writeln!(out, "| {} | {} |", c.name, c.hp);
    }
    out
}

/// The XP needed for each level (spec §8 "Experience"), every level to 20 and then every fifth.
fn experience_levels(content: &Content) -> String {
    let rules = content.experience();
    let mut out = String::from("| Level | XP needed | Hit points gained |\n|:---:|---:|:---:|\n");
    for level in (2..=rules.max_level()).filter(|l| *l <= 20 || l % 5 == 0) {
        let hp = if level <= rules.hp_level() {
            "yes"
        } else {
            "no"
        };
        let _ = writeln!(out, "| {level} | {} | {hp} |", rules.threshold(level));
    }
    out
}

/// The "zero difference" by the player's level.
fn experience_zero_difference(content: &Content) -> String {
    let rules = content.experience();
    let mut out =
        String::from("| Your level | Grey at this many levels below you |\n|---|:---:|\n");
    let bands = rules.bands();
    for (i, &(from, zd)) in bands.iter().enumerate() {
        let to = bands
            .get(i + 1)
            .map_or(rules.max_level(), |next| next.0 - 1);
        let levels = if from >= to {
            from.to_string()
        } else {
            format!("{from} to {to}")
        };
        let _ = writeln!(out, "| {levels} | {zd} |");
    }
    out
}

/// What a kill is worth by the monster's level against yours, as a share of its XP, for a player
/// of level 7 (a zero difference of 5).
fn experience_scaling(content: &Content) -> String {
    let rules = content.experience();
    let player = 7u8;
    let mut out = String::from(
        "| Monster's level | Compared with you | Share of its XP |\n|:---:|---|:---:|\n",
    );
    let (bonus_levels, _) = rules.bonus();
    for diff in (-6i32..=i32::from(bonus_levels) + 1).rev() {
        let monster = (i32::from(player) + diff) as u8;
        let share = rules.kill_xp(100, monster, player);
        let against = match diff {
            0 => "the same level".to_string(),
            d if d > 0 => format!("{d} above"),
            d => format!("{} below", -d),
        };
        let note = if share == 0 {
            "nothing (grey)".to_string()
        } else {
            format!("{share}%")
        };
        let _ = writeln!(out, "| {monster} | {against} | {note} |");
    }
    out
}

/// One creature's XP at several player levels: the kobold shaman, if there is one.
fn experience_example(content: &Content) -> String {
    let rules = content.experience();
    let Some(id) = content.creature_id("kobold_shaman") else {
        return String::new();
    };
    let shaman = content.creature(id);
    let mut out = format!(
        "| Your level | XP for a {} (level {}, worth {}) |\n|:---:|---:|\n",
        shaman.name, shaman.level, shaman.xp
    );
    for level in 1..=10 {
        let xp = rules.kill_xp(shaman.xp, shaman.level, level);
        let _ = writeln!(out, "| {level} | {xp} |");
    }
    out
}

/// The creatures' levels and XP.
fn experience_creatures(content: &Content) -> String {
    let mut out = String::from("| Creature | Level | XP |\n|---|:---:|---:|\n");
    let mut list: Vec<_> = (content.creature_defs().iter())
        .filter(|c| c.id != "player")
        .collect();
    list.sort_by_key(|c| (c.level, c.xp));
    for c in list {
        let _ = writeln!(out, "| {} | {} | {} |", capital(&c.name), c.level, c.xp);
    }
    out
}

/// How long a hit point takes to come back, by Toughness.
fn regeneration() -> String {
    let mut out = String::from("| Toughness | A hit point back every |\n|:---:|---|\n");
    for toughness in [3, 6, 8, 10, 12, 14, 16, 18, 20, 22, 25] {
        let seconds = combat::regen_seconds(toughness);
        let (m, s) = (seconds / 60, seconds % 60);
        let minutes = if m == 1 {
            "1 minute".to_string()
        } else {
            format!("{m} minutes")
        };
        let text = match s {
            0 => minutes,
            s => format!("{minutes} and {s} seconds"),
        };
        let label = if toughness == 25 {
            "25 or more".to_string()
        } else {
            toughness.to_string()
        };
        let _ = writeln!(out, "| {label} | {text} |");
    }
    out
}

/// How many times as much `percent` is, in words: "twice", "two fifths", "30% of".
fn times(percent: u16) -> String {
    match percent {
        200 => "twice".into(),
        300 => "three times".into(),
        50 => "half".into(),
        40 => "two fifths".into(),
        25 => "a quarter".into(),
        p => format!("{p}% of"),
    }
}

/// The special rooms (spec §7 "Special rooms"): what each is called, what it says the first time
/// you enter it, and what it does inside, in words.
fn special_rooms(content: &Content) -> String {
    let mut out = String::from("| Room | The first time you enter | Inside |\n|---|---|---|\n");
    for e in content.room_effects().defs() {
        let mut does: Vec<String> = Vec::new();
        if e.light < 100 {
            does.push(format!(
                "All light is drained to {}% of what it would be.",
                e.light
            ));
        }
        if !e.glow.is_dark() {
            let faint = if e.glow.luma() < 80 { "faint " } else { "" };
            let warm = if e.glow.rgb[0] > e.glow.rgb[2] {
                "warm "
            } else {
                ""
            };
            does.push(format!("A {faint}{warm}glow lights it."));
        }
        if e.noise != 100 {
            let farther = if e.noise > 100 {
                ", and so carries farther"
            } else {
                ""
            };
            does.push(format!(
                "Every noise made in it is {} as loud{farther}.",
                times(e.noise)
            ));
        }
        if e.regen != 100 {
            does.push(format!("Wounds heal {} as fast.", times(e.regen)));
        }
        if e.shunned {
            does.push("Creatures hostile to you do not step in.".into());
        }
        if let Some(floor) = e.floor {
            does.push(format!(
                "Its floor is {}.",
                content.terrain(floor).name.to_lowercase()
            ));
        }
        let _ = writeln!(
            out,
            "| {} | \"{}\" | {} |",
            capital(&e.name),
            e.enter,
            does.join(" ")
        );
    }
    out
}

/// What each level of burden costs (spec §8 "Burden"), from the rules' own numbers.
fn burden_levels() -> String {
    let mut out = String::from(
        "| Burden | You carry | Every action takes | Dexterity adds to Defence | Your steps |\n\
         |---|---|---|---|---|\n",
    );
    for b in Burden::ALL {
        let carried = match b {
            Burden::Unburdened => "up to a third of your limit",
            Burden::Burdened => "up to two thirds",
            Burden::Strained => "up to your limit",
            Burden::Overloaded => "more than your limit",
        };
        // How long a normal action takes, from its price: 6 s at a normal pace.
        let normal = price(warlock_core::content::NORMAL_COST, BASE_TURN);
        let takes = match (b.can_move(), at_normal_pace(b.slow(normal))) {
            (true, t) if b == Burden::Unburdened => format!("{t} (as usual)"),
            (true, t) => format!("{t} (×{})", factor(b.slow(100))),
            (false, t) => format!("{t} (×{}); you cannot move", factor(b.slow(100))),
        };
        let dex = b
            .max_dex()
            .map_or("all of it".to_string(), |cap| match cap {
                0 => "nothing".to_string(),
                cap => format!("at most +{cap}"),
            });
        let steps = match b.footsteps() {
            100 => "as loud as ever".to_string(),
            _ if !b.can_move() => "-".to_string(),
            percent => format!("×{} as loud", factor(i64::from(percent))),
        };
        let _ = writeln!(
            out,
            "| {} | {carried} | {takes} | {dex} | {steps} |",
            b.name()
        );
    }
    out
}

/// `percent` as a factor: `1`, `0.75`, `1.25`, `1.5`.
fn factor(percent: i64) -> String {
    let text = format!("{}.{:02}", percent / 100, percent % 100);
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Each race's size and what it carries next to a medium race, and whether it shrugs off
/// Burdened.
fn burden_races(content: &Content) -> String {
    let mut out = String::from("| Race | Size | Carries | When burdened |\n|---|---|:---:|---|\n");
    for r in &content.races().list {
        let size = content.size(r.size);
        let burdened = if r.ignores_burdened {
            "no penalty at all"
        } else {
            "slowed, louder, less nimble"
        };
        let _ = writeln!(
            out,
            "| {} | {} | ×{} | {burdened} |",
            r.name,
            size.name,
            factor(i64::from(size.carry))
        );
    }
    out
}

/// Carrying limits of a few characters: the race's size with the given Strength and Toughness.
fn burden_examples(content: &Content) -> String {
    let mut out = String::from(
        "| Character | Limit | Unburdened up to | Burdened up to | Strained up to |\n\
         |---|---:|---:|---:|---:|\n",
    );
    for (race, strength, toughness) in [
        ("human", 10, 10),
        ("half_orc", 16, 14),
        ("dwarf", 14, 16),
        ("halfling", 8, 12),
        ("elf", 6, 8),
    ] {
        let Some(r) = content.race(race) else {
            continue;
        };
        let load = Load {
            carried: 0,
            limit: burden::limit(strength, toughness, content.size(r.size).carry),
            ignores_burdened: r.ignores_burdened,
        };
        let [burdened, strained, overloaded] = load.thresholds().map(|(_, g)| format_weight(g));
        let _ = writeln!(
            out,
            "| {}, St {strength} To {toughness} | {overloaded} | {burdened} | {strained} | \
             {overloaded} |",
            r.name
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_are_deterministic_and_list_the_data() {
        let content = Content::builtin().expect("content loads");
        let files = generate(&content);
        assert_eq!(files, generate(&content));
        let text = |name: &str| &files.iter().find(|(n, _)| n == name).expect("file").1;
        assert!(text("calendar.md").contains("| 4 | Rainmoon |"));
        let traps = text("traps.md");
        assert!(
            traps.contains("| Fireball trap `^` | A blast 2 cells around it"),
            "{traps}"
        );
        assert!(
            traps.contains("| 3d6 | once | Explosion, very loud |"),
            "{traps}"
        );
        assert!(traps.contains("| Pit `^` |") && !traps.contains("to_hit"));
        assert!(
            text("moons.md").contains(
                "| Moon | 30 days | standard | about 17% of full light | 1 Frostmere, Year 1 |"
            ),
            "{}",
            text("moons.md")
        );
        assert!(
            text("overworld-sight.md").contains("| Night, a full moon high in the sky | 4 cells |")
        );
        assert!(
            text("overworld-sight.md").contains("| Night, a half moon high in the sky | 3 cells |")
        );
        assert!(text("vision.md").contains("| Darkvision | about 16% of full light | 5 cells |"));
        assert!(text("terrain-overworld.md").contains("| Road | `·` | 20 minutes |"));
        assert!(text("terrain-local.md").contains("| Still water | `~` | yes | no | 12 seconds |"));
        assert!(text("terrain-local.md").contains("| 12 seconds | very loud |"));
        assert!(text("terrain-local.md").contains("muffles sound"));
        assert!(
            text("footsteps.md")
                .contains("| Very loud | Still water, running water, shallow water |")
        );
        let rooms = text("special-rooms.md");
        for row in [
            "| Shadowed room | \"The light seems to drain away here.\" | All light is drained to \
             30% of what it would be. |",
            "| Echoing hall | \"Your footsteps echo loudly.\" | Every noise made in it is twice as \
             loud, and so carries farther. |",
            "| Hushed chamber | \"A deep hush fills this room.\" | Every noise made in it is two \
             fifths as loud. |",
            "| Sanctuary | \"You feel at peace here.\" | A faint warm glow lights it. Wounds heal \
             three times as fast. Creatures hostile to you do not step in. |",
            "| Flooded room | \"The floor here is flooded.\" | Its floor is shallow water. |",
        ] {
            assert!(rooms.contains(row), "{row}\n{rooms}");
        }
        assert!(text("footsteps.md").contains("grass"));
        assert!(
            text("objects.md")
                .contains("| Dagger | `)` | weapon | steel | 450 g | no | Main hand |")
        );
        assert!(
            text("objects.md").contains("| Arrow | `/` | missile | wood | 30 g | yes | Missiles |")
        );
        assert!(text("objects.md").contains("| Left ring or right ring |"));
        assert!(text("objects.md").contains("| Main hand (both hands) |"));
        assert!(
            text("objects.md").contains("| Apple | `%` | food | plant matter | 150 g | yes | - |")
        );
        let equipment = text("equipment.md");
        assert!(
            equipment.contains("| `a` | Head | Iron helm | 6 seconds |"),
            "{equipment}"
        );
        assert!(
            equipment.contains("| `c` | Body | Leather jerkin, chain mail shirt | 18 seconds |")
        );
        assert!(equipment.contains(", quarterstaff (both hands) |"));
        assert!(equipment.contains(
            "| `k` | Missile weapon | Short bow (shoots arrows), long war bow (shoots arrows), \
             crossbow (shoots crossbow bolts), sling (shoots sling stones), any other weapon, and anything made for throwing |"
        ));
        assert!(equipment.contains(
            "| `l` | Missiles | Arrow, flaming arrow, crossbow bolt, sling stone | 6 seconds |"
        ));
        assert!(equipment.contains("| `m` | Tool | Pick-axe, mattock | 6 seconds |"));
        let digging = text("digging.md");
        assert!(
            digging.contains(
                "| Wall | Glyph | Hardness | With a pick-axe | With a mattock | A fireball breaks \
                 it | Becomes |"
            ),
            "{digging}"
        );
        assert!(
            digging.contains("| Stone wall | `#` | 600 | 6 minutes | 4 minutes | yes | rubble |")
        );
        assert!(
            digging.contains("| Granite | `#` | 3000 | 30 minutes | 20 minutes | no | rubble |")
        );
        let strength = text("digging-strength.md");
        assert!(strength.contains("| 18 |"), "{strength}");
        assert!(strength.contains("With a pick-axe, at a normal pace."));
        let launchers = text("launchers.md");
        assert!(
            launchers.contains(
                "| Long war bow | 1d8+2 | 22–24 | ×3 | 14 | 15 cells | Arrows, flaming arrows | 6 \
                 seconds |"
            ),
            "{launchers}"
        );
        assert!(
            launchers.contains("| 12 seconds (reloading) |"),
            "{launchers}"
        );
        let ammunition = text("ammunition.md");
        assert!(
            ammunition.contains(
                "| Flaming arrow | Short bow, long war bow | never | burns as a burning arrow for \
                 1 minute, lighting 5 cells around |"
            ),
            "{ammunition}"
        );
        assert!(ammunition.contains("| Arrow | Short bow, long war bow | 30% | lies there"));
        assert!(ammunition.contains("| Dagger | 1d4 | 20–24 | ×2 | 5 cells | never |"));
        assert!(ammunition.contains("| Rock | 1d3 |"));
        let actions = text("actions.md");
        assert!(
            actions.contains(
                "| Wielding something, or putting it away: Main hand, off hand | 1000 | 6 seconds |"
            ),
            "{actions}"
        );
        assert!(
            actions
                .contains("| Putting something on, or taking it off: Body | 3000 | 18 seconds |")
        );
        assert!(text("materials.md").contains("| Steel | daggers, short swords |"));
        assert!(text("pack.md").contains("| Your own weight | 70 kg |"));
        let burden = text("burden.md");
        for row in [
            "| Unburdened | up to a third of your limit | 6 seconds (as usual) | all of it | as \
             loud as ever |",
            "| Burdened | up to two thirds | 7.5 seconds (×1.25) | at most +3 | ×1.25 as loud |",
            "| Strained | up to your limit | 9 seconds (×1.5) | at most +1 | ×1.5 as loud |",
            "| Overloaded | more than your limit | 9 seconds (×1.5); you cannot move | nothing | - |",
        ] {
            assert!(burden.contains(row), "{row} in {burden}");
        }
        let races = text("burden-races.md");
        assert!(races.contains("| Halfling | Small | ×0.75 | slowed, louder, less nimble |"));
        assert!(races.contains("| Dwarf | Medium | ×1 | no penalty at all |"));
        let examples = text("burden-examples.md");
        assert!(examples.contains("| Human, St 10 To 10 | 45 kg | 15 kg | 30 kg | 45 kg |"));
        assert!(
            examples.contains("| Halfling, St 8 To 12 | 31.5 kg | 10.5 kg | 21 kg | 31.5 kg |")
        );
        assert!(examples.contains("| Half-orc, St 16 To 14 | 69 kg | 23 kg | 46 kg | 69 kg |"));
        assert!(text("stats.md").contains("| Strength | `St` |\n| Dexterity | `Dx` |"));
        assert!(text("stats.md").contains("| Presence | `Pr` |"));
        assert!(text("genders.md").contains("| Female | Dexterity +1 |"));
        // Skills: what each does and trains on, what the passive ones give by value, who trains
        // them as dice, and the provisional increase curve.
        let skills = text("skills.md");
        assert!(
            skills.contains(
                "| Tumble | passive | Rolling, diving, and twisting out of harm's way. | a Dodge \
                 bonus to Defence | dodging an attack (40%) |"
            ),
            "{skills}"
        );
        assert!(skills.contains("| Swim | read by rules |"));
        assert_eq!(skills.matches("\n| ").count(), 1 + 26, "{skills}");
        let benefits = text("skill-benefits.md");
        assert!(
            benefits.contains(
                "| Value | Balance: Burdened / Strained | Listen: hears fainter by | Stealth: \
                 noise | Survival: journeys | Tumble: Dodge |"
            ),
            "{benefits}"
        );
        assert!(benefits.contains("| 100 | ×1.1 / ×1.2 | 10 | \u{2212}60% | \u{2212}20% | +5 |"));
        assert!(benefits.contains("| 150 | ×1.08 / ×1.15 | 15 | \u{2212}75% | \u{2212}20% | +7 |"));
        let grants = text("skill-grants.md");
        assert!(grants.contains("| Halfling | Listen `1d6+4`, Stealth `1d6+4` |"));
        assert!(
            grants.contains("| Human | one of the class's skills `+2d4+4`, picked at random |")
        );
        assert!(grants.contains("| Commoner | Athletics `2d4+4`, Handle animal `2d4+4` |"));
        let increase = text("skill-increase.md");
        assert!(
            increase.contains("| 99 | 13 | 837 |\n| 100 | 42 | 850 |"),
            "{increase}"
        );
        assert!(increase.contains("| 120 | 48 | 1720 |"));
        assert!(
            text("races.md")
                .contains("| Race | St | Dx | To | In | Wi | Pr | Vision | Starting age |"),
            "{}",
            text("races.md")
        );
        // Adjustments as written in data, dice included.
        assert!(
            text("races.md").contains(
                "| Dwarf | +1 | \u{2212}1 | 1d2+2 | 0 | +1 | \u{2212}2 | Darkvision | 40–120 |"
            ),
            "{}",
            text("races.md")
        );
        assert!(text("races.md").contains("| Human | 0 | 0 | 0 | 0 | 0 | 0 | Normal | 16–35 |"));
        assert!(text("races.md").contains("| Elf | \u{2212}1 | 1d2+1 |"));
        assert!(text("races.md").contains("- **Halfling**: Small, quick"));
        assert!(
            text("classes.md")
                .contains("| Wizard | \u{2212}2 | 0 | \u{2212}1d2 | 2d2+1 | +1 | 0 |"),
            "{}",
            text("classes.md")
        );
        assert!(text("classes.md").contains("| Warrior | 1d3+1 | +1 | 1d3 | \u{2212}1 | 0 | 0 |"));
        assert!(text("classes.md").contains("- **Merchant**: A trader"));
        // Combat: race and class hit points, weapons, armour, the rules' odds.
        assert!(text("races.md").contains("| Normal | 16–35 | 4 |"));
        assert!(
            text("classes.md").contains("| Barbarian |") && text("classes.md").contains(" 8 |")
        );
        assert!(text("hit-points.md").contains("| Dwarf | 6 |"));
        assert!(text("hit-points.md").contains("| Wizard | 0 |"));
        let weapons = text("weapons.md");
        assert!(
            weapons.contains("| Dagger | 1d4 | 20–24 | ×2 | - | - | main hand | 5 cells |"),
            "{weapons}"
        );
        assert!(weapons.contains("| Spear | 1d8 | 22–24 | ×3 |"));
        assert!(
            weapons
                .contains("| Quarterstaff | 1d6 | 22–24 | ×2 | +1 | - | main hand (both hands) |")
        );
        assert!(
            weapons.contains("| Short bow | 1d6 | 22–24 | ×3 | - | 10 | missile weapon, shoots |")
        );
        assert!(weapons.contains("Bare hands: 1d3"));
        let armour = text("armour.md");
        assert!(
            armour.contains("| Chain mail shirt | body | +4 | Armour | +4 |"),
            "{armour}"
        );
        assert!(armour.contains("| Small shield | off hand | +1 | Shield | no limit |"));
        let stances = text("stances.md");
        assert!(
            stances.contains("| Aggressive | +1 | +2 | \u{2212}3 | - | - | - | - | 125% |"),
            "{stances}"
        );
        assert!(
            stances.contains("| Coward | \u{2212}4 | \u{2212}3 | +4 | - | 125% | 85% | 50% | - |")
        );
        assert!(text("hit-chances.md").contains("| 10 | 75% |"));
        assert!(text("hit-chances.md").contains("| 24 | 0.7% |"));
        assert!(
            text("threats.md").contains("| 22–24 | 4.2% | 3.1% | 20 |"),
            "{}",
            text("threats.md")
        );
        assert!(text("threats.md").contains("| 19–24 | 15% | 11% | 18–20 |"));
        assert!(text("regeneration.md").contains("| 10 | 4 minutes |"));
        assert!(text("regeneration.md").contains("| 3 | 5 minutes and 24 seconds |"));
        assert!(text("regeneration.md").contains("| 25 or more | 1 minute |"));
        // Potentials stay hidden: the human's rolled potentials are nowhere.
        assert!(!text("races.md").contains("otential"));
        let horoscope = text("horoscope.md");
        assert!(
            horoscope.contains("| 3 | Manticore | +1 | 0 | +1 | 0 | 0 | \u{2212}1 |"),
            "{horoscope}"
        );
        assert!(
            horoscope.contains("| Fire | 0 | 0 | 0 | 0 | 0 | +1 |"),
            "{horoscope}"
        );
        assert!(
            horoscope.contains("began in 1321, 1381, and 1441"),
            "{horoscope}"
        );
        assert!(
            horoscope.contains("| 1 | Wood Salamander | 1321, 1381, 1441 |"),
            "{horoscope}"
        );
        assert!(
            horoscope.contains("| 4 | Metal Unicorn | 1324, 1384, 1444 |"),
            "{horoscope}"
        );
        assert!(
            horoscope.contains("| 60 | Water Barghest | 1380, 1440, 1500 |"),
            "{horoscope}"
        );
        assert_eq!(horoscope.matches(" Salamander | 1").count(), 5);
        // Every object glyph belongs to a class the objects page explains.
        for o in content.object_defs() {
            assert!(object_class(o.glyph).is_some(), "{}", o.id);
        }
        assert!(
            text("creatures.md")
                .contains("| Giant rat | `r` | Vermin | hostile | up close | no | no |")
        );
        assert!(text("creatures.md").contains("| Deer | `d` | Wildlife | neutral | never |"));
        assert!(text("creatures.md").contains("shoots from afar"));
        assert!(text("factions.md").contains("| Kobolds | you, vermin |"));
        assert!(text("factions.md").contains("| Wildlife | nobody |"));
        assert!(
            text("weather.md").contains("| Rainmoon | 20% | 30% | 35% | 15% |"),
            "{}",
            text("weather.md")
        );
        assert!(text("weather-effects.md").contains("| Clear | 100% | 100% | 100% | none |"));
        assert!(text("weather-effects.md").contains("| 2 cells |"));
        assert!(text("overworld-sight.md").contains("| Day, in heavy rain | 5 cells |"));
        assert!(
            text("overworld-sight.md")
                .contains("| Night, a full moon high in the sky, overcast | 3 cells |")
        );
        // Lights go by what the player sees: the terrain, the carrier, the lantern, the crystal.
        for row in [
            "| Wall torch | 7 cells |",
            "| Torch carried by a kobold | 6 cells |",
            "| Your lantern | 5 cells |",
            "| Light crystal (red) | 7 cells |",
            "| Faint rune | 4 cells | UV only",
        ] {
            assert!(text("lights.md").contains(row), "{row}");
        }
        let scenarios = text("scenarios.md");
        assert!(scenarios.contains("| Showcase | 08:00, 11 Darkening, Year 1444 |"));
        assert!(scenarios.contains("| World | 08:00, 11 Darkening, Year 1444 |"));
        assert!(scenarios.find("| Showcase").unwrap() < scenarios.find("| Arena").unwrap());
    }
}

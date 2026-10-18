//! `cargo xtask wiki [--check | --build]`: the player wiki's generated tables.
//!
//! The wiki (`wiki/`, built with mdBook) is written by hand except for facts that live in data
//! (month names, moons, weather, terrain, vision profiles, light presets, creatures, factions,
//! objects and materials, equipment slots, scenarios, action costs, races and classes, the
//! horoscope) and a few constants of the rules (sight radii, the pack's size, the stats' names,
//! what each gender raises). Those are
//! written to `wiki/src/generated/*.md` from the built-in content, and the pages pull them in with
//! `{{#include generated/NAME.md}}`, so they cannot drift. Output is deterministic. `--check` fails
//! when a file is stale; `--build` also runs `mdbook build wiki`.

use crate::workspace_root;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::Command;
use warlock_core::character::Gender;
use warlock_core::content::{
    ActionKind, BASE_TURN, Combat, Content, CreatureId, FactionId, LevelKind, LightId,
    NORMAL_SPEED, ObjectId, Stance, TerrainDef, price,
};
use warlock_core::daylight::{
    MOONLIGHT_PER_CELL, SIGHT_DAY, SIGHT_MOONLIT, SIGHT_NIGHT, SIGHT_TWILIGHT, overworld_sight,
};
use warlock_core::dice::Dice;
use warlock_core::light::{Light, Moon, Sky, Sun};
use warlock_core::object::{INVENTORY_SLOTS, format_weight};
use warlock_core::perception::VIEW_RADIUS;
use warlock_core::stats::{AVERAGE, PerStat, Stat};
use warlock_core::time::{DAY, Duration, GameTime};
use warlock_core::weather::{Clouds, Conditions, Weather};

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
            ActionKind::Shoot => "Shooting (creatures)",
            ActionKind::Cast => "Casting a spell (creatures)",
            ActionKind::Wield => "Wielding something, or putting it away",
            ActionKind::Wear | ActionKind::BodyArmor => "Putting something on, or taking it off",
            ActionKind::Ready => "Readying something, or putting it away",
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
            .filter(|&(o, _)| content.fits(ObjectId(o as u16), slot))
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
        let fits = if fits.is_empty() {
            "-".to_string()
        } else {
            capital(&fits.join(", "))
        };
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
    let slots: Vec<String> = (content.slot_ids())
        .filter(|&(s, _)| content.fits(id, s))
        .map(|(_, s)| s.name.to_lowercase())
        .collect();
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
    let mut out = format!("| Race |{names} Vision | Starting age |\n|---|{align}---|---|\n");
    for r in &content.races().list {
        let vision = content
            .vision(&r.vision)
            .map_or(r.vision.as_str(), |v| &v.name);
        let _ = writeln!(
            out,
            "| {} |{} {vision} | {}–{} |",
            r.name,
            stat_cells(&r.stats),
            r.age.start(),
            r.age.end()
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
    let mut out = format!("| Class |{names}\n|---|{align}\n");
    for c in content.classes() {
        let _ = writeln!(out, "| {} |{}", c.name, stat_cells(&c.stats));
    }
    out.push('\n');
    for c in content.classes() {
        let _ = writeln!(out, "- **{}**: {}", c.name, c.description);
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
        assert!(text("footsteps.md").contains("| Very loud | Still water, running water |"));
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
        assert!(equipment.contains("| `k` | Missile weapon | Short bow (shoots arrows) |"));
        assert!(equipment.contains("| `l` | Missiles | Arrow | 6 seconds |"));
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
        assert!(text("stats.md").contains("| Strength | `St` |\n| Dexterity | `Dx` |"));
        assert!(text("stats.md").contains("| Presence | `Pr` |"));
        assert!(text("genders.md").contains("| Female | Dexterity +1 |"));
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

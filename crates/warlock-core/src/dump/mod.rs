//! Text and JSON dumps of the simulation for developers and agents (spec §9).
//!
//! Every layer except [`Layer::Truth`], [`Layer::Actors`], and [`Layer::Rooms`] is built from
//! [`World::perception`] only, so dumps cannot leak information the player does not have.
//! `Truth`, `Actors`, and `Rooms` read ground truth and are dev-only.

use crate::ability::Aim;
use crate::action::{letter, position_text};
use crate::blood::{self, Stain};
use crate::character::Gender;
use crate::content::{Content, GiftKind, GiftTier, SpecialRoom, WeaponSkillId};
use crate::equipment;
use crate::geom::Pos;
use crate::hunger::Hunger;
use crate::ice::Ice;
use crate::noise::{PerceivedSound, SoundKind, Volume};
use crate::object::{Stack, format_weight};
use crate::perception::{CellView, PerceivedLight, PerceptionView};
use crate::stats::Stat;
use crate::text::article;
use crate::weather::Weather;
use crate::world::{CallingStatus, Cause, LevelState, World};
use serde_json::{Value, json};
use std::fmt::Write;
use std::str::FromStr;

mod abilities;
mod actors;
mod character;
mod items;
mod json;
mod kept;
mod known;
mod layer;
mod map;
mod progress;
mod quests;
mod spells;
mod story;
mod truth;
mod weapons;

pub use json::json;
pub use layer::{Layer, parse_layers};
pub use map::{blood_digit, ice_char, light_digit, sound_char};

use abilities::*;
use actors::*;
use character::*;
use items::*;
use kept::*;
use known::*;
use layer::*;
use map::*;
use progress::*;
use quests::*;
use spells::*;
use story::*;
use truth::*;
use weapons::*;

/// Renders one layer as rows of text with trailing spaces trimmed.
pub fn layer_rows(world: &World, layer: Layer) -> Vec<String> {
    match layer {
        Layer::Truth => [
            spawn_header(world),
            encounter_header(world),
            truth_rows(world),
            chest_rows(world),
            crate::world::rigged_rows(world),
            statue_rows(world),
            grave_rows(world),
            leaving_rows(world),
            hallow_rows(world),
            zone_rows(world),
            pit_rows(world),
        ]
        .concat(),
        Layer::Actors => [spawn_header(world), actor_rows(world), rising_rows(world)].concat(),
        Layer::Progress => progress_rows(world),
        Layer::Rooms => room_rows(world),
        Layer::Ice => ice_rows(world),
        Layer::Inventory => inventory_rows(&world.perception()),
        Layer::Equipment => equipment_rows(&world.perception()),
        Layer::Stats => stat_rows(&world.perception()),
        Layer::Skills => skill_rows(&world.perception()),
        Layer::WeaponSkills => weapon_skill_rows(&world.perception()),
        Layer::Attacks => attack_rows(&world.perception()),
        Layer::Talents => talent_rows(&world.perception()),
        Layer::Abilities => ability_rows(&world.perception()),
        Layer::Recipes => recipe_rows(&world.perception()),
        Layer::Spells => spell_rows(&world.perception()),
        Layer::Traps => trap_rows(&world.perception()),
        Layer::Locations => location_rows(&world.perception()),
        Layer::Quests => quest_rows(&world.perception()),
        Layer::Story => story_rows(world),
        Layer::Companion => kept_rows(&world.perception(), Kept::Companion),
        Layer::Familiar => kept_rows(&world.perception(), Kept::Familiar),
        _ => {
            let view = world.perception();
            rows(view.width(), view.height(), |p| {
                perceived_glyph(&view, layer, p)
            })
        }
    }
}

/// One-line summary used as the dump header; under the open sky it names the weather (and the
/// weather coming, `then rain`, for a Weather-wise player), and it has the temperature where the
/// player is (and `dazzled` while strong light dazzles them, `averted` while they avert their
/// eyes from a gaze).
fn summary(world: &World) -> String {
    let view = world.perception();
    // Spelled as in the JSON dump (`heavy_rain`).
    let spelled = |w: Weather| serde_json::to_value(w).ok()?.as_str().map(str::to_string);
    let weather = (view.weather())
        .and_then(spelled)
        .map(|w| format!(" weather {w}"))
        .unwrap_or_default();
    let temperature = view.temperature();
    let (hp, max) = view.own_hp();
    let (mp, max_mp) = view.own_mana();
    // The resources the player holds, as the status bar shows them (spec §8 "Resources").
    let resources: String = (view.own_resources().iter())
        .map(|r| {
            let id = &view.content().resource(r.resource).id;
            format!(" {id} {}/{}", r.value, r.max)
        })
        .collect();
    let dead = if world.death().is_some() { " dead" } else { "" };
    // As the status bar shows it: only when it is not Not hungry.
    let hunger = (view.own_hunger())
        .filter(|h| *h != Hunger::NotHungry)
        .map_or(String::new(), |h| format!(" hunger {}", h.id()));
    let dazzled = if view.dazzled() { " dazzled" } else { "" };
    // Averted eyes (`world/avert.rs`), as the status bar shows them.
    let averted = if view.averted() { " averted" } else { "" };
    // The allies with the player: on the level or in the retinue (spec §8 "Allies").
    let allies = match view.allies_with() {
        0 => String::new(),
        n => format!(" allies {n}"),
    };
    // An overworld encounter to answer (spec §7 "Overworld encounters").
    let encounter = view.encounter().map_or(String::new(), |e| {
        let choices: Vec<&str> = e.choices.iter().map(|(_, c)| c.id()).collect();
        format!(" encounter pending: {} ({})", e.name, choices.join(", "))
    });
    format!(
        "turn {} time {} level {} viewer {} hp {hp}/{max} mp {mp}/{max_mp}{resources}{dead}{hunger}{dazzled}{averted}{allies}{encounter}{weather} \
         temperature {temperature} hash {:016x}",
        world.turn(),
        world.time(),
        view.level_id(),
        view.viewer(),
        world.state_hash()
    )
}

/// Plain-text dump: a header line, then each requested layer under a `== name ==` heading.
/// Dev-only layers are headed like `== truth (DEV ONLY: ground truth) ==`.
pub fn ascii(world: &World, layers: &[Layer]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", summary(world));
    for &layer in layers {
        let note = if layer.is_dev_only() {
            " (DEV ONLY: ground truth)"
        } else {
            ""
        };
        let _ = writeln!(out, "== {}{note} ==", layer.name());
        for row in layer_rows(world, layer) {
            let _ = writeln!(out, "{row}");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, parse_actions};
    use crate::content::Content;
    use std::sync::Arc;

    /// Walks north out of the start room and through the first door, then along the corridor.
    const SHOWCASE_RUN: &str = "move:N move:N move:N move:N move:N move:N move:E move:E";

    fn showcase() -> World {
        let content = Arc::new(Content::builtin().expect("builtin content"));
        World::new(content, "showcase", None).expect("world")
    }

    fn run(world: &mut World, actions: &str) {
        for action in parse_actions(actions).expect("actions") {
            world.apply(&action);
        }
    }

    /// A dump with its state hash redacted, for the insta snapshots: the hash covers every
    /// field of the state and changes with any content or rules change, so leaving it in would
    /// churn the snapshots on every feature (determinism is `replay.rs`'s tests' job).
    fn snapshot(world: &World, layers: &[Layer]) -> String {
        let text = ascii(world, layers);
        let (header, rest) = text.split_once('\n').expect("a header line");
        let (before, hash) = header.rsplit_once(" hash ").expect("a hash in the header");
        assert_eq!(hash.len(), 16, "{header}");
        format!("{before} hash [redacted]\n{rest}")
    }

    #[test]
    fn parses_layer_lists() {
        assert_eq!(
            parse_layers("view, fov,truth"),
            Ok(vec![Layer::View, Layer::Fov, Layer::Truth])
        );
        assert_eq!(parse_layers("light"), Ok(vec![Layer::Light]));
        assert!(parse_layers("view,lights").is_err());
    }

    #[test]
    fn view_hides_unseen_actors_that_truth_shows() {
        let world = showcase();
        let view = layer_rows(&world, Layer::View).join("\n");
        let truth = layer_rows(&world, Layer::Truth).join("\n");
        assert!(truth.contains('r') || truth.contains('k'));
        assert_eq!(view.matches('@').count(), 1);
        assert!(view.chars().filter(|c| *c != '\n').all(|c| c != 'k'));
    }

    #[test]
    fn json_has_documented_fields() {
        let mut world = showcase();
        world.apply(&Action::Wait);
        let value = json(&world, &[Layer::View, Layer::Fov], true);
        assert_eq!(value["turn"], 1);
        assert_eq!(value["time"]["text"], "1444-11-11 08:00:06");
        assert_eq!(value["time"]["month_name"], "Darkening");
        assert_eq!(value["level"], "hub");
        assert_eq!(value["layers"]["fov"].as_array().map(Vec::len), Some(21));
        assert!(value["layers"].get("memory").is_none());
        assert_eq!(value["state"]["turn"], 1);
        assert!(value["room"].is_null(), "not in a special room");
        assert!(value["shop"].is_null(), "not in a shop");
        assert!(value["seen_actors"][0]["kind"] == "player");
        // The player's character, and their own scores by stat id; nothing else about stats.
        assert_eq!(
            value["character"],
            json!({ "name": "Tamsin", "race": "human", "gender": "female", "class": "thief",
                    "raised": "dexterity", "age": 34,
                    "born": { "year": 1410, "month": 5, "day": 29 },
                    "sign": { "animal": "basilisk", "element": "water" },
                    "classes": [{ "class": "thief", "level": 1 }] })
        );
        assert_eq!(
            value["stats"],
            json!({ "strength": 9, "dexterity": 13, "toughness": 10, "intelligence": 12,
                    "willpower": 11, "presence": 9 })
        );
        // Their own trained skills by skill id, with their values: no marks, nothing untrained.
        assert_eq!(
            value["skills"],
            json!({ "appraise": 11, "detect_traps": 14, "disable_device": 14, "escape_artist": 10, "initiative": 9, "listen": 6, "negotiation": 10, "open_lock": 17, "search": 10, "sneak_attack": 25, "spot": 9, "steal": 12, "stealth": 16, "tumble": 15 })
        );
        assert!(json(&world, &[], false).get("state").is_none());
    }

    #[test]
    fn skills_show_the_players_trained_skills_only() {
        let world = showcase();
        assert_eq!(
            layer_rows(&world, Layer::Skills),
            [
                "Appraise 11",
                "Detect traps 14",
                "Disable device 14",
                "Escape artist 10",
                "Initiative 9",
                "Listen 6",
                "Negotiation 10",
                "Open lock 17",
                "Search 10",
                "Sneak attack 25",
                "Spot 9",
                "Steal 12",
                "Stealth 16",
                "Tumble 15"
            ]
        );
        assert_eq!(
            parse_layers("skills,stats"),
            Ok(vec![Layer::Skills, Layer::Stats])
        );
        assert!(!Layer::Skills.is_dev_only());
        // A creature's skills show only in the dev layer: the arena's shaman's.
        let content = Arc::new(Content::builtin().expect("content"));
        let arena = World::new(content, "arena", None).expect("world");
        let actors = layer_rows(&arena, Layer::Actors);
        let shaman = actors.iter().find(|a| a.contains(" kobold_shaman at "));
        assert!(
            shaman.is_some_and(|s| s.ends_with(
                "; skills concentration 20 (0 +1), spellcraft 25 (0 +1); unaware of player; \
                 resists fire 25"
            )),
            "{actors:#?}"
        );
        let text = ascii(&arena, &[Layer::Skills, Layer::View]);
        assert!(
            !text.contains("spellcraft") && !text.contains("Spellcraft"),
            "{text}"
        );
    }

    #[test]
    fn showcase_snapshots() {
        let mut world = showcase();
        let layers = [Layer::View, Layer::Fov];
        insta::assert_snapshot!("showcase_start", snapshot(&world, &layers));
        run(&mut world, &SHOWCASE_RUN[..SHOWCASE_RUN.len() / 2]);
        insta::assert_snapshot!("showcase_mid", snapshot(&world, &layers));
        let mut world = showcase();
        run(&mut world, SHOWCASE_RUN);
        insta::assert_snapshot!(
            "showcase_end",
            snapshot(&world, &[Layer::View, Layer::Memory])
        );
    }

    #[test]
    fn stats_show_the_players_scores_and_actors_show_everyones() {
        let world = showcase();
        assert_eq!(
            layer_rows(&world, Layer::Stats),
            [
                "Name Tamsin",
                "Race Human",
                "Gender Female",
                "Class Thief",
                "Age 34",
                "Born 29 Greenleaf 1410",
                "Sign Water Basilisk",
                "Strength 9",
                "Dexterity 13",
                "Toughness 10",
                "Intelligence 12",
                "Willpower 11",
                "Presence 9",
                "Alignment Callous"
            ]
        );
        // Other names the stat it raised.
        let content = Arc::new(Content::builtin().expect("content"));
        let other = "gnome,other,bard".parse().expect("a character");
        let gnome =
            World::with_character(content, "showcase", Some(3), Some(&other)).expect("world");
        let raised = crate::character::other_stat(3).name();
        assert!(
            layer_rows(&gnome, Layer::Stats).contains(&format!("Gender Other ({raised} +1)")),
            "{:?}",
            layer_rows(&gnome, Layer::Stats)
        );
        let actors = layer_rows(&world, Layer::Actors);
        // Every actor's stats, then its trained skills with their marks and the ladder's die.
        assert!(
            actors[0].contains(
                "stats St 9/16 Dx 13/16 To 10/16 In 12/16 Wi 11/16 Pr 9/16; skills appraise 11 (0 +1), detect_traps 14 (0 +1), disable_device 14 (0 +1), escape_artist 10 (0 +1), initiative 9 (0 +1), listen 6 (0 +1), negotiation 10 (0 +1), open_lock 17 (0 +1), search 10 (0 +1), sneak_attack 25 (0 +1), spot 9 (0 +1), steal 12 (0 +1), stealth 16 (0 +1), tumble 15 (0 +1); \
                 nutrition 3600 (not_hungry)"
            ),
            "{}",
            actors[0]
        );
        let kobold = actors.iter().find(|a| a.contains(" kobold at "));
        assert!(
            kobold
                .is_some_and(|k| k
                    .ends_with("St 9/9 Dx 13/13 To 6/6 In 10/10 Wi 9/9 Pr 8/8; unaware of player")),
            "{actors:#?}"
        );
        assert!(Layer::Actors.is_dev_only() && !Layer::Stats.is_dev_only());
    }

    #[test]
    fn weapon_skills_show_the_players_ranks_and_progress_and_actors_show_everyones() {
        // The showcase thief: daggers 2 and thrown 1 from the class, and (the seed's pick)
        // Duelist 1. With nothing in their hands they fight as a brawler.
        let mut world = showcase();
        let rows = layer_rows(&world, Layer::WeaponSkills);
        assert_eq!(rows.len(), 26);
        assert_eq!(rows[0], "Daggers and knives 2 (0%)");
        assert_eq!(rows[15], "Duelist 1 (0%)");
        assert_eq!(rows[21], "Brawler 0 (0%), in use");
        assert!(
            rows.contains(&"Thrown weapons 1 (0%)".to_string()),
            "{rows:#?}"
        );
        assert!(rows.contains(&"Long blades 0 (0%)".to_string()));
        let actors = layer_rows(&world, Layer::Actors);
        assert!(
            actors[0].contains(
                "; weapon skills daggers 2 (0/190), thrown 1 (0/30), duelist 1 (0/45); stats"
            ),
            "{}",
            actors[0]
        );
        let kobold = actors.iter().find(|a| a.contains(" kobold at "));
        assert!(kobold.is_some_and(|k| k.contains("weapon skills spears 1 (0/45)")));
        // Marks show as a share of the next rank.
        let content = world.content().clone();
        let table = content.weapon_skills();
        let daggers = table.id("daggers").expect("daggers");
        let mut state = world.state().clone();
        let me = state.actors.get_mut(&world.player_id()).expect("player");
        me.weapon_skills.train(table, daggers, 76);
        world = World::from_state(content.clone(), state).expect("valid");
        assert_eq!(
            layer_rows(&world, Layer::WeaponSkills)[0],
            "Daggers and knives 2 (40%)"
        );
        let value = json(&world, &[], false);
        assert_eq!(value["weapon_skills"]["daggers"]["rank"], 2);
        assert_eq!(value["weapon_skills"]["daggers"]["progress"], 40);
        assert_eq!(value["weapon_skills"]["bows"]["rank"], 0);
        assert_eq!(value["weapon_skills"]["duelist"]["rank"], 1);
        assert_eq!(value["styles_in_use"], json!(["brawler"]));
        assert!(!Layer::WeaponSkills.is_dev_only());
    }

    #[test]
    fn abilities_show_what_the_player_can_do_and_why_not() {
        // The showcase thief: the class's skills used as actions (Disable device's, disarming a
        // trap, Open lock's, picking a lock, and Steal's, stealing, each in a direction, greyed
        // with nothing beside the player to use them on, #1327; Listen's left out away from a
        // closed door), the shield bash, which needs a shield in hand;
        // first their birth sign's stare (a Basilisk).
        let mut world = showcase();
        assert_eq!(
            layer_rows(&world, Layer::Abilities),
            [
                "Transfixing stare (sign): a cell",
                "Exploit (class): you have no opening",
                "Disarm a trap (skill): no known trap beside you",
                "Pick lock (skill): no lock beside you",
                "Steal (skill): no one beside you to rob",
                "Salvage (skill): an item you carry",
                "Shield bash (weapon skill): you hold no shield",
            ]
        );
        let value = json(&world, &[], false);
        assert_eq!(
            value["abilities"][5],
            json!({"id": "salvage", "source": "skill", "aim": "an item you carry",
                   "unavailable": null})
        );
        assert_eq!(
            value["abilities"][6],
            json!({"id": "shield_bash", "source": "weapon_skill", "aim": "a direction",
                   "unavailable": "no_shield"})
        );
        assert_eq!(
            value["abilities"][2],
            json!({"id": "disarm", "source": "skill", "aim": "a direction",
                   "unavailable": "no_target:trap"})
        );
        assert_eq!(
            value["abilities"][3],
            json!({"id": "pick_lock", "source": "skill", "aim": "a direction",
                   "unavailable": "no_target:lock"})
        );
        assert_eq!(value["abilities"][0]["aim"], "a cell");
        assert_eq!(
            value["abilities"][1],
            json!({"id": "exploit", "source": "class", "aim": "a direction",
                   "unavailable": "short:openings"})
        );
        assert_eq!(
            value["abilities"][4],
            json!({"id": "steal", "source": "skill", "aim": "a direction",
                   "unavailable": "no_target:mark"})
        );
        // Below the hall's door: Listen is listed, and Pick lock may be used, in a direction.
        for action in crate::action::parse_actions("move:N*3").expect("actions") {
            world.apply(&action);
        }
        let rows = layer_rows(&world, Layer::Abilities);
        assert_eq!(rows[3], "Listen at a door (skill): a direction");
        assert_eq!(rows[4], "Pick lock (skill): a direction");
        // The storeroom's small shield in hand: with no foe beside, there is nobody to bash.
        let walk = "open:N move:N*2 move:E*16 open:S move:S*4 open:W move:W*2 move:N \
                    pickup:a equip:off_hand:b";
        for action in crate::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        let rows = layer_rows(&world, Layer::Abilities);
        assert_eq!(rows[6], "Shield bash (weapon skill): no foe within reach");
        assert_eq!(
            json(&world, &[], false)["abilities"][6]["unavailable"],
            "no_target:foe"
        );
        assert!(!Layer::Abilities.is_dev_only());
    }

    #[test]
    fn light_digits_are_tenths_of_full_light() {
        let light = |rgb: [u16; 3], uv: u16| PerceivedLight {
            rgb,
            uv,
            darkvision: false,
            dist2: 0,
        };
        assert_eq!(light_digit(light([0, 0, 0], 0)), '0');
        assert_eq!(light_digit(light([25, 25, 25], 0)), '0');
        assert_eq!(light_digit(light([26, 26, 26], 0)), '1');
        assert_eq!(light_digit(light([128, 128, 128], 0)), '5');
        assert_eq!(light_digit(light([255, 255, 255], 0)), '9');
        assert_eq!(light_digit(light([900, 900, 900], 0)), '9');
        assert_eq!(
            light_digit(light([0, 0, 0], 128)),
            '5',
            "perceived UV counts"
        );
    }

    #[test]
    fn light_layer_shows_visible_cells_only() {
        let world = showcase();
        insta::assert_snapshot!("showcase_light", snapshot(&world, &[Layer::Light]));
        let rows = layer_rows(&world, Layer::Light);
        let fov = layer_rows(&world, Layer::Fov);
        for (light, fov) in rows.iter().zip(&fov) {
            assert_eq!(light.len(), fov.len());
            for (l, f) in light.chars().zip(fov.chars()) {
                assert_eq!(l.is_ascii_digit(), f == '*');
            }
        }
    }
}

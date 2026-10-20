//! `cargo xtask wiki [--build]`: the player wiki's generated tables.
//!
//! The wiki (`wiki/`, built with mdBook) is written by hand except for facts that live in data
//! or in a few constants of the rules. Those are written to `wiki/src/generated/*.md` from the
//! built-in content, and the pages pull them in with `{{#include generated/NAME.md}}`, so they
//! cannot drift. Output is deterministic. The generated files are not committed (CI and the
//! release workflow run this before `mdbook build wiki`); `--build` also runs `mdbook build wiki`.
//!
//! Layout: one module per wiki topic (`calendar`, `terrain`, `combat`, ...), each a set of
//! `pub(super) fn table(content: &Content) -> String`; `text` holds the helpers two or more of
//! them share. [`generate`] is the one registration list: a line per generated file, in order.

mod abilities;
mod actions;
mod burden;
mod calendar;
mod character;
mod combat;
mod creatures;
mod dialogue;
mod digging;
mod doors;
mod experience;
mod healing;
mod hunger;
mod magic;
mod moons;
mod objects;
mod order;
mod rooms;
mod scenarios;
mod scrolls;
mod searching;
mod sight;
mod skills;
mod temperature;
mod terrain;
mod text;
mod traps;
mod vision;
mod weapon_skills;
mod weather;

use crate::workspace_root;
use std::path::PathBuf;
use std::process::Command;
use warlock_core::content::Content;

/// What to do with the generated files.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Write,
    Build,
}

pub fn parse_mode(args: &[String]) -> Result<Mode, String> {
    match args {
        [] => Ok(Mode::Write),
        [a] if a == "--build" => Ok(Mode::Build),
        _ => Err("usage: cargo xtask wiki [--build]".into()),
    }
}

fn generated_dir() -> PathBuf {
    workspace_root().join("wiki/src/generated")
}

pub fn run(mode: Mode) -> Result<(), String> {
    let content = Content::builtin().map_err(|e| format!("cannot load content: {e}"))?;
    let files = generate(&content);
    let dir = generated_dir();
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

/// One generated table: its text from the content.
type Table = fn(&Content) -> String;

/// The generated files: name and text, in a fixed order.
///
/// This is the one registration list. A new table is its own function in the module of its
/// topic (a new module for a new topic) plus one line here, next to its topic: never at the end.
fn generate(content: &Content) -> Vec<(String, String)> {
    let tables: &[(&str, Table)] = &[
        ("calendar.md", calendar::months),
        ("actions.md", actions::action_costs),
        ("terrain-local.md", terrain::local_terrain),
        ("terrain-overworld.md", terrain::overworld_terrain),
        ("overworld-sight.md", sight::overworld_sight_table),
        ("moons.md", moons::moons),
        ("weather.md", weather::weather_chances),
        ("weather-effects.md", weather::weather_effects),
        ("temperature.md", temperature::temperatures),
        ("temperature-weather.md", temperature::temperature_weather),
        ("freezing.md", temperature::freezing),
        ("ice.md", temperature::ice_thickness),
        ("ice-growth.md", temperature::ice_growth),
        ("vision.md", vision::vision),
        ("lights.md", vision::lights),
        ("creatures.md", creatures::creatures),
        ("factions.md", creatures::factions),
        ("dialogue.md", dialogue::dialogue),
        ("quests.md", dialogue::quests),
        ("footsteps.md", terrain::footsteps),
        ("objects.md", objects::objects),
        ("item-classes.md", objects::item_classes),
        ("shops.md", objects::shops),
        ("equipment.md", objects::equipment),
        ("materials.md", objects::materials),
        ("pack.md", objects::pack),
        ("scenarios.md", scenarios::scenarios),
        ("stats.md", character::stats),
        ("genders.md", character::genders),
        ("races.md", character::races),
        ("classes.md", character::classes),
        ("horoscope.md", character::horoscope),
        ("skills.md", skills::skills),
        ("skill-benefits.md", skills::skill_benefits),
        ("skill-grants.md", skills::skill_grants),
        ("skill-increase.md", skills::skill_increase),
        ("abilities.md", abilities::abilities),
        ("weapons.md", combat::weapons),
        ("light-weapons.md", combat::light_weapons),
        ("launchers.md", combat::launchers),
        ("ammunition.md", combat::ammunition),
        ("armour.md", combat::armour),
        ("piercing.md", combat::piercing),
        ("stances.md", combat::stances),
        ("hit-chances.md", combat::hit_chances),
        ("threats.md", combat::threats),
        ("hit-points.md", combat::hit_points),
        ("regeneration.md", combat::regeneration),
        ("healing.md", healing::healing),
        ("status-effects.md", healing::status_effects),
        ("diseases.md", healing::diseases),
        ("experience-levels.md", experience::experience_levels),
        ("experience-zd.md", experience::experience_zero_difference),
        ("experience-scaling.md", experience::experience_scaling),
        ("experience-example.md", experience::experience_example),
        ("experience-creatures.md", experience::experience_creatures),
        ("burden.md", burden::burden_levels),
        ("burden-races.md", burden::burden_races),
        ("burden-examples.md", burden::burden_examples),
        ("doors.md", doors::doors),
        ("keys.md", doors::keys),
        ("hunger-states.md", hunger::hunger_states),
        ("hunger-facts.md", hunger::hunger_facts),
        ("hunger-races.md", hunger::hunger_races),
        ("food.md", hunger::food),
        ("corpses.md", hunger::corpses),
        ("digging.md", digging::digging),
        ("digging-strength.md", digging::digging_strength),
        ("special-rooms.md", rooms::special_rooms),
        ("magic-rarity.md", magic::magic_rarity),
        ("magic-bases.md", magic::magic_bases),
        ("magic-prefixes.md", magic::prefixes),
        ("magic-suffixes.md", magic::suffixes),
        ("magic-names.md", magic::rare_names),
        ("scrolls.md", scrolls::scrolls),
        ("weapon-skills.md", weapon_skills::skills),
        ("weapon-defaults.md", weapon_skills::defaults),
        ("weapon-marks.md", weapon_skills::marks),
        ("weapon-benefits.md", weapon_skills::benefits),
        ("styles.md", weapon_skills::styles),
        ("style-defaults.md", weapon_skills::style_defaults),
        ("style-benefits.md", weapon_skills::style_benefits),
        ("search-features.md", searching::search_features),
        ("search-depths.md", searching::search_depths),
        ("search-races.md", searching::search_races),
        ("traps.md", traps::traps),
    ];
    let mut files: Vec<(String, String)> = tables
        .iter()
        .map(|(name, table)| (name.to_string(), table(content)))
        .collect();
    for (_, text) in &mut files {
        *text = format!(
            "<!-- Generated by `cargo xtask wiki` from the game's data. Do not edit. -->\n\n{text}"
        );
    }
    files
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
        let scrolls = text("scrolls.md");
        assert!(
            scrolls.contains("| Scroll of fireball `?` | Flies at the cell you aim at"),
            "{scrolls}"
        );
        assert!(
            scrolls.contains("| 3d6 | 10 cells | Explosion, very loud | item level 3 | 150 gold |"),
            "{scrolls}"
        );
        assert!(scrolls.contains("| Scroll of identify `?` | Identifies one unidentified item"));
        assert!(
            !scrolls.contains("Scroll of light `"),
            "nothing to read on it yet"
        );
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
        assert!(text("terrain-local.md").contains("breaks into a broken door (sturdiness 30)"));
        assert!(
            text("doors.md").contains("| Closed door | 30 | 10% | 20% | 30% | 33% |"),
            "{}",
            text("doors.md")
        );
        assert!(text("keys.md").contains("| Skeleton key | `(` | every lock |"));
        assert!(text("keys.md").contains("| Iron key | `(` | locks named `pantry` |"));
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
        assert!(text("objects.md").contains(
            "| Dagger | `)` | melee weapons | steel | 450 g | 30 gold | no | Main hand or off hand |"
        ));
        // Objects of several parts list them all, the main one first; names show the gems and
        // precious metals.
        let objects = text("objects.md");
        assert!(
            objects.contains("| Long sword | `)` | melee weapons | steel blade, leather grip |")
        );
        assert!(
            objects
                .contains("| Gold and ruby amulet | `\"` | amulets | gold chain, ruby setting |")
        );
        assert!(text("materials.md").contains("| Ruby | yes | gold and ruby amulets, rubies |"));
        // Fighting styles: their configurations, defaults, ranks, and the classes' starting ones.
        let styles = text("styles.md");
        for row in [
            "| Duelist | A one-handed weapon, the other hand empty | any | melee |",
            "| Great Weapon | A two-handed weapon | any but staves | melee |",
            "| Staff Fighter | A two-handed weapon | Staves | melee |",
            "| Marksman | Shooting a missile weapon | Bows, Crossbows, Slings | missile |",
        ] {
            assert!(styles.contains(row), "{row}\n{styles}");
        }
        assert!(
            text("style-defaults.md").contains("| Duelist | Twin Blades −3, Sword and Board −3 |")
        );
        let benefits = text("style-benefits.md");
        assert!(benefits.contains("main hand strikes at **−4** to hit and the off hand at **−8**"));
        assert!(benefits.contains("| 1 | −3 | −7 |") && benefits.contains("| 10 | none | none |"));
        assert!(
            benefits.contains("| 8 | +4 | +1 roll | 10% less |"),
            "{benefits}"
        );
        assert!(text("classes.md").contains("| Twin Blades 1 or Duelist 1 |"));
        assert!(
            !text("weapon-skills.md").contains("Duelist"),
            "styles have their own tables"
        );
        assert!(
            text("objects.md")
                .contains("| Arrow | `/` | missiles | wood | 30 g | 1 gold | yes | Missiles |")
        );
        assert!(text("objects.md").contains("| Left ring or right ring |"));
        assert!(text("objects.md").contains("| Main hand (both hands) |"));
        assert!(
            text("objects.md")
                .contains("| Apple | `%` | food | plant matter | 150 g | 1 gold | yes | - |")
        );
        assert!(
            text("objects.md").contains("| Rock | `*` | miscellaneous | stone | 800 g | nothing |")
        );
        let shops = text("shops.md");
        assert!(
            shops.contains(
                "| Armourer | Grimbold's Armoury | shields, body armour, helmets, cloaks, gloves, and \
                 boots | shields, body armour, helmets, cloaks, gloves, and boots | 130% of the \
                 worth | 40% of the worth |"
            ),
            "{shops}"
        );
        let equipment = text("equipment.md");
        assert!(
            equipment.contains("| `a` | Head | Leather cap, iron helm, great helm | 6 seconds |"),
            "{equipment}"
        );
        assert!(equipment.contains(
            "| `c` | Body | Padded coat, leather jerkin, studded leather jerkin, chain mail shirt, \
             suit of scale mail, suit of banded mail, suit of plate armour | 18 seconds |"
        ));
        // Weapons by skill, then value, then name (the staves last among the melee skills).
        assert!(equipment.contains(", quarterstaff (both hands), ironwood staff (both hands) |"));
        assert!(
            equipment
                .contains("| `f` | Off hand | Stiletto, dagger, short sword, hand axe, buckler, ")
        );
        assert!(equipment.contains(
            "| `k` | Missile weapon | Short bow (shoots arrows), long war bow (shoots arrows), \
             composite bow (shoots arrows), crossbow (shoots crossbow bolts), heavy crossbow \
             (shoots crossbow bolts), sling (shoots sling stones), any other weapon, and anything \
             made for throwing |"
        ));
        assert!(equipment.contains(
            "| `l` | Missiles | Sling stone, arrow, crossbow bolt, flaming arrow | 6 seconds |"
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
                "| Flaming arrow | Short bow, long war bow, composite bow | never | burns as a \
                 burning arrow for 1 minute, lighting 5 cells around |"
            ),
            "{ammunition}"
        );
        assert!(ammunition.contains("| Arrow | Short bow, long war bow, composite bow | 30% |"));
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
        let abilities = text("abilities.md");
        assert!(
            abilities.contains(
                "| Shield bash | Weapon skills | everyone (usable with a shield in hand) | a \
                 direction | Strike someone next to you with the shield you hold. | 6 seconds |"
            ),
            "{abilities}"
        );
        assert!(abilities.contains("| Open lock | Skills | everyone trained in the skill | — |"));
        assert!(
            text("materials.md")
                .contains("| Steel | no | stilettos, daggers, short swords, scimitars,")
        );
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
                "| Value | Appraise: prices | Athletics: kicking doors in | Balance: Burdened / \
                 Strained | Listen: hears fainter by | Search: searching | Search: search reaches \
                 | Spot: noticing | Stealth: noise | Survival: journeys | Tumble: Dodge |"
            ),
            "{benefits}"
        );
        assert!(benefits.contains(
            "| 100 | 20% better | +8 Strength | ×1.1 / ×1.2 | 10 | +8 | +1 cell | +8 | \u{2212}60% \
                 | \u{2212}20% | +5 |"
        ));
        assert!(benefits.contains(
            "| 150 | 25% better | +10 Strength | ×1.08 / ×1.15 | 15 | +10 | +1 cell | +10 | \
             \u{2212}75% | \u{2212}20% | +7 |"
        ));
        // Searching: the hidden things' chances for an average character, deeper down, and who
        // notices in passing.
        let features = text("search-features.md");
        assert!(features.contains("| Secret door | A seam in the wall | 46% | over 99% | 19% |"));
        assert!(
            features.contains("| Alarm trap | A tripwire |"),
            "{features}"
        );
        let depths = text("search-depths.md");
        assert!(
            depths.contains("| The Depths 3 | none (caves have no doors) |"),
            "{depths}"
        );
        assert!(
            depths.contains("| The Depths 8 | 1 to 2 | 6.9% | 51% | 0.7% |"),
            "{depths}"
        );
        let races = text("search-races.md");
        assert!(
            races.contains("| Dwarf | What is hidden in stone | — |"),
            "{races}"
        );
        assert!(
            races.contains("| Elf | Everything | more easily (+2) |"),
            "{races}"
        );
        assert!(
            races.contains("trained in Spot, who notice everything in passing: Ranger, Thief.")
        );
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
            weapons.contains(
                "| Dagger | 1d4 | 20–24 | ×2 | - | - | main hand or off hand | 5 cells | Daggers and \
                 knives | 6 seconds |"
            ),
            "{weapons}"
        );
        assert!(weapons.contains("| Spear | 1d8 | 22–24 | ×3 |"));
        assert!(
            weapons
                .contains("| Quarterstaff | 1d6 | 22–24 | ×2 | +1 | - | main hand (both hands) |")
        );
        assert!(weapons.contains(
            "| Short bow | 1d6 | 22–24 | ×3 | - | 10 | missile weapon, shoots | - | Bows | - |"
        ));
        assert!(weapons.contains("| Great sword | 2d6 |") && weapons.contains("| Great blades |"));
        assert!(weapons.contains("| 7.2 seconds |"));
        assert!(
            weapons.contains("| Rock | 1d3 |")
                && weapons.contains("| 6 cells | Thrown weapons | - |")
        );
        // Magic items: rarity, bases, affixes with their tiers, and the words of rare names.
        assert!(text("magic-rarity.md").contains("| Rare | 6% |"));
        assert!(text("magic-bases.md").contains("| Long sword | weapons |"));
        let prefixes = text("magic-prefixes.md");
        assert!(prefixes.contains(
            "| Sharp | damage, added to every roll | weapons, ammunition | 1: +1 · 3: +1–2"
        ));
        assert!(
            prefixes.contains("| Fleet | you move faster | boots | 1: a little faster to faster")
        );
        // Speed stays in words: no row about it has a number of percent.
        let suffixes = text("magic-suffixes.md");
        for line in prefixes.lines().chain(suffixes.lines()) {
            if line.contains("| you move faster |") || line.contains("| attacks faster |") {
                assert!(!line.contains('%'), "{line}");
            }
        }
        assert!(text("magic-suffixes.md").contains("| of the Bear | Strength |"));
        assert!(text("magic-names.md").contains("| Weapons | Bite,"));
        assert!(
            text("lights.md")
                .contains("| Faintly glowing magic item (Radiant, of Light) | 2 cells |")
        );
        assert!(weapons.contains("Bare hands: 1d3"));
        let armour = text("armour.md");
        assert!(
            armour.contains("| Chain mail shirt | body | +2 | Armour | 3 | +4 |"),
            "{armour}"
        );
        assert!(armour.contains("| Small shield | off hand | +1 | Shield | - | no limit |"));
        assert!(armour.contains("| Great helm | head | - | - | 2 | no limit | - |"));
        assert!(
            text("piercing.md")
                .ends_with("The crossbow bolt, the stiletto, and the war hammer pierce armour.\n")
        );
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
        let healing = text("healing.md");
        for row in [
            "| Resting (`R`) | twice as fast |",
            "| Bleeding | not at all |",
            "| Poisoned | half as fast |",
            "| Starving | not at all |",
        ] {
            assert!(healing.contains(row), "{row} in {healing}");
        }
        let statuses = text("status-effects.md");
        for row in [
            "| Bleeding | yes, \"bleeding\" | Big hits and confirmed criticals | A hit point every \
             40 seconds, faster the worse it is | A degree a minute | Not at all | — | Bandage |",
            "| A degree every 2 minutes | Half as fast |",
            "| Poisoned | no | Some creatures' attacks; eating rotten food |",
            "Strength \u{2212}1, Dexterity \u{2212}1 when strong",
        ] {
            assert!(statuses.contains(row), "{row} in {statuses}");
        }
        assert!(
            !statuses.contains("iant rat"),
            "which creatures poison stays hidden"
        );
        assert!(statuses.contains("| Diseased | no | Any disease, once it shows | Nothing |"));
        let diseases = text("diseases.md");
        for row in [
            "| Filth fever | Some creatures' bites | Toughness, 54% | 1d3 days | Every day | \
             1d3 Dexterity, 1d3 Toughness | Two checks won in a row | no | Half as fast |",
            "| Marsh ague | Crossing marsh or swamp |",
            "| Grave rot | Eating rotten food |",
            "| Never by itself | yes |",
            "| Brain fever | — | Willpower, 54% |",
        ] {
            assert!(diseases.contains(row), "{row} in {diseases}");
        }
        assert!(
            !diseases.contains("iant rat"),
            "which creatures stays hidden"
        );
        assert!(healing.contains("| Diseased with marsh ague | a quarter as fast |"));
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
        // Every class is listed, in order, with its objects.
        let classes = text("item-classes.md");
        assert!(
            classes.contains("| Melee weapons | Bastard swords, battle axes, "),
            "{classes}"
        );
        assert!(
            classes.contains(
                "| Food | Angelica roots, apples, food rations, loaves of bread, strips of dried \
                 meat, corpses |"
            ),
            "{classes}"
        );
        let at = |row: &str| classes.find(row).unwrap_or_else(|| panic!("{row}"));
        assert!(at("| Melee weapons |") < at("| Gems | Rubies |"));
        assert!(at("| Gems | Rubies |") < at("| Miscellaneous | Rocks |"));
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
        // Hunger: the states by nutrition, each race's need, food, and corpses (their effects
        // when eaten stay unsaid: the player finds them out).
        let states = text("hunger-states.md");
        assert!(
            states.contains("| Hungry | 720 to 1439 | 36 hours |"),
            "{states}"
        );
        assert!(states.contains("| Not hungry | 1440 to 3999 | (you start here) |"));
        assert!(text("hunger-races.md").contains("| Halfling | ×1.25 | 1800 |"));
        assert!(text("food.md").contains("| Food ration | 1500 | 25 hours | 4 | 500 g |"));
        // How long food keeps, roughly: never a timer.
        assert!(text("food.md").contains("| about a month | about two months |"));
        assert!(text("food.md").contains("| about five days | about ten days |"));
        assert!(text("corpses.md").contains("| Kobold corpse | Small | 25 kg | 400 |"));
        assert!(text("corpses.md").contains("| a few hours | a day or two |"));
        assert!(!text("objects.md").contains("corpse"));
        assert!(!text("corpses.md").contains("terrible"));
    }
}

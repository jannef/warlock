//! `cargo xtask wiki [--build | --check-links]`: the player wiki's generated tables.
//!
//! The wiki (`wiki/`, built with mdBook) is written by hand except for facts that live in data
//! or in a few constants of the rules. Those are written to `wiki/src/generated/*.md` from the
//! built-in content, and the pages pull them in with `{{#include generated/NAME.md}}`, so they
//! cannot drift. Output is deterministic. The generated files are not committed (CI and the
//! release workflow run this before `mdbook build wiki`); `--build` also runs `mdbook build wiki`,
//! and `--check-links` builds and then checks every link and anchor in the book (`links`).
//!
//! Layout: one module per wiki topic (`calendar`, `terrain`, `combat`, ...), each a set of
//! `pub(super) fn table(content: &Content) -> String`; `text` holds the helpers two or more of
//! them share. [`generate`] is the one registration list: a line per generated file, in order.

mod abilities;
mod actions;
mod alignment;
mod animated;
mod background;
mod burden;
mod calendar;
mod character;
mod chests;
mod combat;
mod creatures;
mod damage;
mod dialogue;
mod digging;
mod doors;
mod dungeons;
mod experience;
mod gems;
mod guild;
mod healing;
mod hearing;
mod horoscope;
mod hunger;
mod kits;
mod learning;
mod links;
mod magic;
mod mana;
mod materials;
mod mimics;
mod moons;
mod objects;
mod order;
mod outcomes;
mod overworld;
mod potions;
mod resources;
mod rooms;
mod scenarios;
mod schools;
mod scrolls;
mod searching;
mod services;
mod sight;
mod skills;
mod spells;
mod stats;
mod surprise;
mod talents;
mod temperature;
mod terrain;
mod text;
mod traps;
mod uniques;
mod vision;
mod weapon_abilities;
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
    /// Build, then check every link in the built book.
    CheckLinks,
}

pub fn parse_mode(args: &[String]) -> Result<Mode, String> {
    match args {
        [] => Ok(Mode::Write),
        [a] if a == "--build" => Ok(Mode::Build),
        [a] if a == "--check-links" => Ok(Mode::CheckLinks),
        _ => Err("usage: cargo xtask wiki [--build | --check-links]".into()),
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
    if mode != Mode::Write {
        build_book()?;
    }
    if mode == Mode::CheckLinks {
        let wiki = workspace_root().join("wiki");
        links::check(&wiki.join("src"), &wiki.join("book"))?;
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
        ("speeds.md", actions::speeds),
        ("terrain-local.md", terrain::local_terrain),
        ("terrain-overworld.md", terrain::overworld_terrain),
        ("overworld-sight.md", sight::overworld_sight_table),
        ("overworld-places.md", overworld::locations),
        ("overworld-map.md", overworld::map),
        ("overworld-regions.md", overworld::regions),
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
        ("creature-tags.md", creatures::creature_tags),
        ("creature-stats.md", creatures::creature_stats),
        ("creature-defences.md", creatures::creature_defences),
        ("creature-attacks.md", creatures::creature_attacks),
        ("creature-throws.md", creatures::creature_throws),
        ("creature-summons.md", creatures::creature_summons),
        ("creature-surprise.md", creatures::creature_surprise),
        ("creature-senses.md", creatures::creature_senses),
        ("animated-tiers.md", animated::animated_tiers),
        ("animated-numbers.md", animated::animated_numbers),
        ("animated-bodies.md", animated::animated_bodies),
        ("animated-families.md", animated::animated_families),
        ("animated-groups.md", animated::animated_groups),
        ("animated-loot.md", animated::animated_loot),
        ("animated-parts.md", animated::animated_parts),
        ("mimic-bands.md", mimics::mimic_bands),
        ("dungeons.md", dungeons::dungeons),
        ("dungeon-floors.md", dungeons::dungeon_floors),
        ("danger-weights.md", dungeons::danger_weights),
        ("creature-rarities.md", dungeons::creature_rarities),
        ("draw-odds.md", dungeons::draw_odds),
        ("dialogue.md", dialogue::dialogue),
        ("rumoured-ways.md", dialogue::rumoured_ways),
        ("quests.md", dialogue::quests),
        ("guild-ranks.md", guild::guild_ranks),
        ("guild-draws.md", guild::guild_draws),
        ("services.md", services::services),
        ("footsteps.md", terrain::footsteps),
        ("noises.md", hearing::noises),
        ("sound-travel.md", hearing::sound_travel),
        ("muffling.md", hearing::muffling),
        ("objects.md", objects::objects),
        ("item-classes.md", objects::item_classes),
        ("shops.md", objects::shops),
        ("equipment.md", objects::equipment),
        ("materials.md", materials::materials),
        ("material-finds.md", materials::material_finds),
        ("construction.md", materials::construction),
        ("variants.md", materials::variants),
        ("pieces.md", materials::pieces),
        ("gems.md", gems::gems),
        ("gem-sizes.md", gems::sizes),
        ("gem-cuts.md", gems::cuts),
        ("ornaments.md", gems::ornaments),
        ("pack.md", objects::pack),
        ("scenarios.md", scenarios::scenarios),
        ("stats.md", character::stats),
        ("stat-modifiers.md", character::stat_modifiers),
        ("stat-uses.md", character::stat_uses),
        ("potentials.md", character::potentials),
        ("stat-training.md", stats::stat_training),
        ("stat-losses.md", stats::stat_losses),
        ("stat-prices.md", stats::stat_prices),
        ("stat-odds.md", stats::stat_odds),
        ("stat-pace.md", stats::stat_pace),
        ("drain-attacks.md", stats::drain_attacks),
        ("genders.md", character::genders),
        ("races.md", character::races),
        ("race-traits.md", character::race_traits),
        ("classes.md", character::classes),
        ("horoscope.md", character::horoscope),
        ("horoscope-animals.md", horoscope::animals),
        ("horoscope-elements.md", horoscope::elements),
        ("horoscope-flavour.md", horoscope::flavour),
        ("horoscope-lucky.md", horoscope::lucky),
        ("horoscope-talents.md", horoscope::talents),
        ("background-tiers.md", background::tiers),
        ("background-points.md", background::points),
        ("background-talents.md", background::talents),
        ("background-parents.md", background::parents),
        ("background-birth.md", background::birth),
        ("background-childhood.md", background::childhood),
        ("background-youth.md", background::youth),
        ("background-trial.md", background::trial),
        ("background-fortune.md", background::fortune),
        ("background-leaving.md", background::leaving),
        ("background-road.md", background::road),
        ("class-kits.md", kits::class_kits),
        ("race-kits.md", kits::race_kits),
        ("alignment-bands.md", alignment::alignment_bands),
        ("alignment-start.md", alignment::alignment_start),
        ("alignment-deeds.md", alignment::alignment_deeds),
        ("alignment-quests.md", alignment::alignment_quests),
        ("alignment-gifts.md", alignment::alignment_gifts),
        ("alignment-shops.md", alignment::alignment_shops),
        ("talents-creation.md", talents::creation),
        ("talents.md", talents::talents),
        ("talents-racial.md", talents::racial),
        ("talents-heirlooms.md", talents::heirlooms),
        ("talents-by-system.md", talents::by_system),
        ("class-levels.md", talents::class_levels),
        ("foraging.md", skills::foraging),
        ("skills.md", skills::skills),
        ("skill-benefits.md", skills::skill_benefits),
        ("skill-resist.md", skills::skill_resist),
        ("skill-grants.md", skills::skill_grants),
        ("skill-ladder.md", skills::skill_ladder),
        ("skill-rolls.md", skills::skill_rolls),
        ("slipping-past.md", skills::slipping_past),
        ("stealing.md", skills::stealing),
        ("heal-targets.md", skills::heal_targets),
        ("abilities.md", abilities::abilities),
        ("weapons.md", combat::weapons),
        ("light-weapons.md", combat::light_weapons),
        ("launchers.md", combat::launchers),
        ("ammunition.md", combat::ammunition),
        ("armour.md", combat::armour),
        ("piercing.md", combat::piercing),
        ("protection.md", combat::protection),
        ("damage-types.md", damage::damage_types),
        ("damage-weapons.md", damage::weapon_kinds),
        ("damage-armour.md", damage::armour_kinds),
        ("damage-elements.md", damage::elements),
        ("stances.md", combat::stances),
        ("to-hit.md", combat::to_hit),
        ("hit-chances.md", combat::hit_chances),
        ("threats.md", combat::threats),
        ("surprise-rules.md", surprise::surprise_rules),
        ("surprise-ladders.md", surprise::surprise_ladders),
        ("surprise-check.md", surprise::initiative_odds),
        ("surprise-vitals.md", surprise::no_vitals),
        ("hit-points.md", combat::hit_points),
        ("health.md", combat::health),
        ("regeneration.md", combat::regeneration),
        ("healing.md", healing::healing),
        ("status-effects.md", healing::status_effects),
        ("diseases.md", healing::diseases),
        ("disease-checks.md", healing::disease_checks),
        ("mana.md", mana::races_and_classes),
        ("mana-regeneration.md", mana::regeneration),
        ("mana-paces.md", mana::paces),
        ("spell-shapes.md", spells::shapes),
        ("spell-resolutions.md", spells::resolutions),
        ("spell-casting.md", spells::casting),
        ("spell-list.md", spells::list),
        ("class-spells.md", spells::class_spells),
        ("spell-armour.md", spells::armour),
        ("wands.md", spells::wands),
        ("rods.md", spells::rods),
        ("spell-runes.md", spells::runes),
        ("school-skills.md", schools::skills),
        ("school-benefits.md", schools::benefits),
        ("school-abilities.md", schools::abilities),
        ("caster-style-benefits.md", schools::caster_style_benefits),
        (
            "caster-style-abilities.md",
            weapon_abilities::caster_style_abilities,
        ),
        ("study-chances.md", learning::chances),
        ("study-times.md", learning::times),
        ("study-affinities.md", learning::affinities),
        ("mishaps.md", learning::mishaps),
        ("resources.md", resources::resources),
        ("experience-levels.md", experience::experience_levels),
        ("experience-factors.md", experience::experience_factors),
        ("experience-discovery.md", experience::experience_discovery),
        ("experience-zd.md", experience::experience_zero_difference),
        ("experience-scaling.md", experience::experience_scaling),
        ("experience-example.md", experience::experience_example),
        ("experience-creatures.md", experience::experience_creatures),
        ("burden.md", burden::burden_levels),
        ("burden-races.md", burden::burden_races),
        ("burden-examples.md", burden::burden_examples),
        ("doors.md", doors::doors),
        ("keys.md", doors::keys),
        ("lock-picking.md", doors::lock_picking),
        ("chest-bands.md", chests::chest_bands),
        ("chest-descriptions.md", chests::chest_descriptions),
        ("chest-breaking.md", chests::chest_breaking),
        ("chest-loot.md", chests::chest_loot),
        ("chest-odds.md", chests::chest_odds),
        ("chest-places.md", chests::chest_places),
        ("hunger-states.md", hunger::hunger_states),
        ("hunger-facts.md", hunger::hunger_facts),
        ("hunger-races.md", hunger::hunger_races),
        ("food.md", hunger::food),
        ("corpses.md", hunger::corpses),
        ("corpse-effects.md", hunger::corpse_effects),
        ("digging.md", digging::digging),
        ("digging-strength.md", digging::digging_strength),
        ("special-rooms.md", rooms::special_rooms),
        ("branch-odds.md", rooms::branch_odds),
        ("teeming-dens.md", rooms::teeming_dens),
        ("altar-outcomes.md", outcomes::altar_outcomes),
        ("altar-shares.md", outcomes::altar_shares),
        ("forge-outcomes.md", outcomes::forge_outcomes),
        ("forge-shares.md", outcomes::forge_shares),
        ("spring-outcomes.md", outcomes::spring_outcomes),
        ("risings.md", rooms::risings),
        ("magic-rarity.md", magic::magic_rarity),
        ("magic-bases.md", magic::magic_bases),
        ("magic-prefixes.md", magic::prefixes),
        ("magic-suffixes.md", magic::suffixes),
        ("magic-names.md", magic::rare_names),
        ("magic-systems.md", magic::systems),
        ("uniques.md", uniques::uniques),
        ("scrolls.md", scrolls::scrolls),
        ("potions.md", potions::potions),
        ("weapon-skills.md", weapon_skills::skills),
        ("weapon-defaults.md", weapon_skills::defaults),
        ("weapon-marks.md", weapon_skills::marks),
        ("weapon-benefits.md", weapon_skills::benefits),
        ("weapon-abilities.md", weapon_abilities::weapon_abilities),
        ("styles.md", weapon_skills::styles),
        ("style-defaults.md", weapon_skills::style_defaults),
        ("style-benefits.md", weapon_skills::style_benefits),
        ("style-abilities.md", weapon_abilities::style_abilities),
        ("search-features.md", searching::search_features),
        ("search-depths.md", searching::search_depths),
        ("search-races.md", searching::search_races),
        ("traps.md", traps::traps),
        ("trap-avoid.md", traps::avoiding),
        ("trap-disarm.md", traps::disarming),
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

    /// The generated files without their rows' anchors (`<a id="trap-pit"></a>Pit` is `Pit`) and
    /// with links to wiki pages as their text (`[Spears](weapon-skills.md#weapon-skill-spears)` is
    /// `Spears`), so the assertions read the tables as a reader does; `rows_have_anchors` checks
    /// the anchors and links.
    fn plain(content: &Content) -> Vec<(String, String)> {
        (generate(content).into_iter())
            .map(|(name, text)| (name, unlink(&strip_anchors(&text))))
            .collect()
    }

    fn strip_anchors(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(at) = rest.find("<a id=\"") {
            out.push_str(&rest[..at]);
            let end = rest[at..].find("</a>").map_or(rest.len(), |e| at + e + 4);
            rest = &rest[end..];
        }
        out.push_str(rest);
        out
    }

    fn unlink(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(open) = rest.find('[') {
            out.push_str(&rest[..open]);
            let after = &rest[open + 1..];
            let link = after.find("](").and_then(|mid| {
                let target_end = after[mid + 2..].find(')')? + mid + 2;
                let (name, target) = (&after[..mid], &after[mid + 2..target_end]);
                let wiki = target.contains(".md") && !target.contains(' ') && !name.contains('[');
                wiki.then_some((name, target_end + 1))
            });
            match link {
                Some((name, len)) => {
                    out.push_str(name);
                    rest = &after[len..];
                }
                None => {
                    out.push('[');
                    rest = after;
                }
            }
        }
        out.push_str(rest);
        out
    }

    /// Rows that define a term carry the anchor prose links to (`wiki/TERMS.md`).
    #[test]
    fn rows_have_anchors() {
        let content = Content::builtin().expect("content loads");
        let files = generate(&content);
        let text = |name: &str| &files.iter().find(|(n, _)| n == name).expect("file").1;
        assert!(text("traps.md").contains("| <a id=\"trap-pit\"></a>Pit `^` |"));
        assert!(text("status-effects.md").contains("| <a id=\"status-bleeding\"></a>Bleeding |"));
        assert!(text("potions.md").contains("<a id=\"potion-of-healing\"></a>Potion of healing"));
        assert!(text("stats.md").contains("| <a id=\"stat-strength\"></a>Strength |"));
        assert!(
            text("weapon-abilities.md").contains("[Spears](weapon-skills.md#weapon-skill-spears)")
        );
    }

    /// The line of `page` that starts with `start` must contain every fragment: a row that lists
    /// creatures is checked by the parts that stay, so a new creature edits no assertion (#473).
    fn row_has(page: &str, start: &str, fragments: &[&str]) {
        let line = page
            .lines()
            .find(|line| line.starts_with(start))
            .unwrap_or_else(|| panic!("no row {start:?} in {page}"));
        for fragment in fragments {
            assert!(line.contains(fragment), "{fragment:?} in {line}");
        }
    }

    /// The system effects (docs/design/affixes-systems.md) have their words and tier forms in
    /// the affix tables: the shipped files (#635), every tier on its ladder.
    #[test]
    fn system_affixes_have_words_in_the_tables() {
        let content = Content::builtin().expect("content loads");
        let prefixes = magic::prefixes(&content);
        let suffixes = magic::suffixes(&content);
        for row in [
            "| Tireless | maximum stamina | stamina | body armour, gloves, boots, rings, amulets | \
             1: +1 · 3: +1–2 · 5: +2 · 7: +2–3 · 9: +3 · 11: +3–4 · 12: +4–5 | 60 |",
            "| Delving | digs faster | digging | body armour, gloves, tools | 2: 10–15% less time · \
             4: 17–22% less time · 6: 24–29% less time · 8: 31–36% less time · \
             10: 38–43% less time · 12: 45–50% less time | 60 |",
            "| Mage-wrought | this armour's spell cost surcharge is lower, in points | \
             spell armour | body armour, helmets, shields | 2: 4 points less · 4: 7 points less · \
             6: 10 points less · 8: 13 points less · 10: 16 points less · 12: 20 points less | 50 |",
        ] {
            assert!(prefixes.contains(row), "{row}\nin\n{prefixes}");
        }
        for row in [
            "| of Thrift | every mana cost is lower | mana cost | helmets, cloaks, rings, amulets | \
             2: 4–5% less · 4: 6–7% less · 6: 8–9% less · 8: 10–11% less · 10: 12–13% less · \
             12: 14–16% less | 60 |",
            "| of Ease | weapon feats cost less stamina | stamina cost | weapons, missile weapons, \
             shields, gloves | 3: 1 less · 7: 1–2 less · 10: 2 less | 40 |",
            "| of Sustenance | hunger comes slower | hunger | body armour, cloaks, rings, amulets | \
             2: 8–10% slower · 4: 12–14% slower · 6: 16–19% slower · 8: 21–24% slower · \
             10: 25–29% slower · 12: 30–35% slower | 60 |",
        ] {
            assert!(suffixes.contains(row), "{row}\nin\n{suffixes}");
        }
    }

    /// What systems have affixes (#637): every affix a row, with its system and the talents read
    /// beside it, named as the game names them.
    #[test]
    fn the_systems_table_has_every_affix() {
        let content = Content::builtin().expect("content loads");
        let table = magic::systems(&content);
        let rows = table.lines().filter(|l| l.starts_with("| ")).count() - 1;
        assert_eq!(rows, content.affixes().all().len(), "{table}");
        row_has(
            &table,
            "| of Thrift |",
            &["suffix", "Mana", "Frugal caster"],
        );
        row_has(
            &table,
            "| Tireless |",
            &["prefix", "Stamina", "Deep lungs, Bellows"],
        );
        row_has(&table, "| of Fury |", &["Resources", "maximum rage"]);
        row_has(
            &table,
            "| of the Adept |",
            &["Spells", "Arcane training, Attuned"],
        );
        row_has(&table, "| Delving |", &["Digging", "Tunneler"]);
        row_has(&table, "| of the Eagle |", &["Skills", "Spot (a skill)"]);
    }

    /// The numbers the game keeps to itself are stated exactly (#264): creatures, stats, speed,
    /// hearing, combat, diseases, and corpses.
    #[test]
    fn the_story_tables_list_every_line() {
        // Every line of every chapter of a story in the Ascmearc in its chapter's table, its
        // effects exact; the tiers' odds as the data has them.
        let content = Content::builtin().expect("content loads");
        let files = generate(&content);
        let text = |name: &str| {
            let file = format!("background-{name}.md");
            (files.iter().find(|(n, _)| *n == file))
                .unwrap_or_else(|| panic!("{file}"))
                .1
                .clone()
        };
        let b = content.background().expect("stories");
        for chapter in b.chapters_for("ascmearc") {
            let table = text(&chapter.id);
            for line in &chapter.lines {
                assert!(table.contains(&format!("| `{}` |", line.id)), "{}", line.id);
            }
        }
        assert!(
            text("tiers").contains("| Uncommon | 12 % | 24 % | 64 % | 0.96 | +1 to +3 | yes |")
        );
        assert!(text("childhood").contains("[Strength](stats.md#stat-strength) +1 (worth +2)"));
        assert!(
            text("parents").contains("| with [Family wealth](talents.md#talent-family-wealth) |")
        );
        assert!(!text("road").contains("the Hall"), "never the test world's");
    }

    #[test]
    fn hidden_numbers_are_exact() {
        let content = Content::builtin().expect("content loads");
        let files = plain(&content);
        let text = |name: &str| &files.iter().find(|(n, _)| n == name).expect("file").1;
        let has = |name: &str, row: &str| {
            assert!(
                text(name).contains(row),
                "{row}\nin {name}:\n{}",
                text(name)
            );
        };
        // Creatures: stats with modifiers, Defence with its parts, attacks read through the
        // rules (the rat's Dexterity, the beetle's shell), speed, senses (the deer's listen
        // skill lowers its hearing from 5), and carrying limits.
        has(
            "creature-stats.md",
            "| Giant rat | 1 | 5 | Common | 10 (+0) | 17 (+3) | 12 (+1) | 1 (\u{2212}5) | 12 (+1) | \
             4 (\u{2212}3) | as its scores | - | - |",
        );
        has("creature-stats.md", "| Deer | 1 | 2 |");
        // Rarity (#373): each type's tier, and the tiers' factors.
        has("creature-stats.md", "| Kobold shaman | 4 | 40 | Rare |");
        row_has(
            text("creature-rarities.md"),
            "| Very rare | 10% | ",
            &["bog hag", "cinder elemental"],
        );
        has("creature-stats.md", "| Listen 30 | - |");
        has(
            "creature-defences.md",
            "| Giant rat | 12 | 13 (Dexterity +3) | - | - |",
        );
        has(
            "creature-defences.md",
            "| Cave beetle | 11 | 11 (Natural +1) | 3 (shell) | acid 50%, poison 25% |",
        );
        has(
            "creature-attacks.md",
            "| Giant rat | up close | bite | 1d4 piercing | +0 | 22–24 ×2 | 5 seconds | 10% \
             poisoned (2 degrees), 10% filth fever (unless resisted) |",
        );
        has(
            "creature-attacks.md",
            "| Kobold | up close | fists | 1d3\u{2212}1 bludgeoning | \u{2212}1 |",
        );
        has(
            "creature-senses.md",
            "| Cave bat | 2000 | 3 seconds | 8 cells | Darkvision | 3 | silent | 500 g | Small |",
        );
        has(
            "creature-senses.md",
            "| Deer | 1300 | 4.6 seconds | 12 cells | Normal | 2 | 50 |",
        );
        assert!(!text("creature-senses.md").contains("| You"));
        // Speed: yours first, then the quickest.
        assert!(
            text("speeds.md").contains(
                "|---|---:|---:|\n| You, whatever your race | 1000 | 6 seconds |\n| Cave bat | \
                 2000 | 3 seconds |"
            ),
            "{}",
            text("speeds.md")
        );
        // Stats: every score's modifier, the races' potentials, and what reads them.
        for row in [
            "| 1 | \u{2212}5 |",
            "| 10–11 | +0 |",
            "| 18–19 | +4 |",
            "| 40 | +15 |",
        ] {
            has("stat-modifiers.md", row);
        }
        has("potentials.md", "| Human | 16–17 | 16–17 |");
        has("potentials.md", "| Dwarf | 17 | 13 | 19 |");
        has(
            "stat-uses.md",
            "| Presence | shop prices: 3% in your favour for each point",
        );
        has("stat-uses.md", "checks against brain fever");
        // Stat training: the price by score and past the potential, the odds, the pace.
        has(
            "stat-prices.md",
            "| 14 | 160 | 320 | 640 | 1120 | 1760 | 2560 | 3520 | 75 |",
        );
        has("stat-odds.md", "| the price | 50 % |");
        has("stat-odds.md", "| under a quarter | 0 % |");
        has("stat-pace.md", "| 1 | 49 | 62 | 80 | 198 |");
        // The deeds that cost marks, and the drop by score.
        has(
            "stat-losses.md",
            "| Toughness | each hit point lost to starvation | a mark, 50 % of the time |",
        );
        has(
            "stat-losses.md",
            "| Presence | being caught stealing | 20 marks |",
        );
        has("stat-losses.md", "| 14 | 75 | −19 |");
        // Hearing: loudness where made and how far you hear it, how sound fades, what muffles.
        has(
            "noises.md",
            "| Your step onto normal ground | footsteps | 60 | 5 cells (3 diagonally) |",
        );
        has(
            "noises.md",
            "| An alarm trap going off | a bell ring | 300 | 29 cells (20 diagonally) |",
        );
        has("noises.md", "| Kicking a door | banging | 100 |");
        has("sound-travel.md", "loses 10 loudness for every cell");
        has("sound-travel.md", "farther than 30 cells");
        has("muffling.md", "| Closed door | 30 | 3 more cells |");
        // Combat: the soak, the parts of the to-hit, how hurt creatures look, regeneration.
        has("protection.md", "| 3 | 2–3 | 2.5 |");
        has("protection.md", "| 20 | 10–20 | 15 |");
        has(
            "to-hit.md",
            "main hand \u{2212}4, off hand \u{2212}8 to hit",
        );
        has(
            "to-hit.md",
            "| A weapon not made for throwing | - | - | \u{2212}4 to hit |",
        );
        has("health.md", "| Lightly wounded | 41–59 | 9–11 |");
        has("health.md", "| Nearly dead | 1–10 | 1–2 |");
        has("regeneration.md", "max(60, 360 − 12 × Toughness)");
        // Diseases: each check's difficulty and odds by score (an average 10 as in the
        // diseases table), and what fresh corpses do.
        has(
            "disease-checks.md",
            "| Filth fever | Toughness | 13 | 31% | 38% | 46% | 54% |",
        );
        has(
            "corpse-effects.md",
            "| Cave beetle corpse | 60 Toughness marks | \"Crunchy.\" |",
        );
        has(
            "corpse-effects.md",
            "| Deer corpse | +1000 nutrition; heals 1d6 hit points |",
        );
    }

    #[test]
    fn tables_are_deterministic_and_list_the_data() {
        let content = Content::builtin().expect("content loads");
        let files = plain(&content);
        assert_eq!(generate(&content), generate(&content));
        let text = |name: &str| &files.iter().find(|(n, _)| n == name).expect("file").1;
        assert!(text("calendar.md").contains("| 4 | Rainmoon |"));
        let traps = text("traps.md");
        assert!(
            traps.contains("| Fireball trap `^` | A blast 2 cells around it"),
            "{traps}"
        );
        assert!(
            traps.contains("| 3d6 fire | once | Explosion, very loud (150) |"),
            "{traps}"
        );
        assert!(traps.contains("| Pit `^` |") && !traps.contains("to_hit"));
        let scrolls = text("scrolls.md");
        assert!(
            scrolls.contains("| Scroll of fireball `?` | Casts Fireball a rung up its ladder"),
            "{scrolls}"
        );
        assert!(
            scrolls.contains(
                "| 3d6 fire | 10 cells | Explosion, very loud (150) | item level 6, frequency 20 | 200 gold |"
            ),
            "{scrolls}"
        );
        assert!(
            scrolls
                .contains("| Scroll of lightning `?` | Casts Lightning bolt 2 rungs up its ladder"),
            "{scrolls}"
        );
        assert!(
            scrolls.contains("| 4d6 electricity | 8 cells |"),
            "{scrolls}"
        );
        assert!(
            scrolls.contains("| Scroll of identify `?` | Casts Identify at its base"),
            "{scrolls}"
        );
        assert!(scrolls.contains("| an item you carry |"), "{scrolls}");
        assert!(
            scrolls.contains("| Scroll of light `?` | Casts Light at its base"),
            "{scrolls}"
        );
        let potions = text("potions.md");
        assert!(
            potions.contains(
                "| Potion of healing `!` | heal | 2d8 hit points back at once, never past your \
                 maximum | red (`#c83c3c`) | 15 Toughness | item level 1, frequency 60 | 40 gold |"
            ),
            "{potions}"
        );
        assert!(
            potions.contains("| Murky draught `!` | harm | 2 degrees of Poisoned"),
            "{potions}"
        );
        assert!(text("lights.md").contains("| You, while Lit | 6 cells | visible |"));
        assert!(text("status-effects.md").contains("You shed light 6 cells around you"));
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
        assert!(
            text("vision.md").contains("| Darkvision | 40 of 255 (16% of full light) | 5 cells |")
        );
        assert!(text("terrain-overworld.md").contains("| Road | `·` | 20 minutes |"));
        assert!(text("terrain-local.md").contains("| Still water | `~` | yes | no | 12 seconds |"));
        assert!(text("terrain-local.md").contains("| 12 seconds | very loud, 200% |"));
        assert!(text("terrain-local.md").contains("muffles sound by 30"));
        assert!(text("terrain-local.md").contains("breaks into a broken door (sturdiness 30)"));
        assert!(
            text("doors.md").contains("| Closed door | 30 | 10% | 20% | 30% | 33% |"),
            "{}",
            text("doors.md")
        );
        assert!(text("keys.md").contains("| Skeleton key | `(` | every lock |"));
        assert!(text("keys.md").contains("| Iron key | `(` | locks named `pantry` |"));
        let picking = text("lock-picking.md");
        assert!(
            picking.contains("| A lock of the usual difficulty | 14 | 46% | 62% |"),
            "{picking}"
        );
        assert!(
            picking.contains("| Usual, on The Depths 8 | 21 |"),
            "{picking}"
        );
        assert!(
            text("footsteps.md")
                .contains("| Very loud, 200% | Still water, running water, shallow water |")
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
             three times as fast. Mana comes back three times as fast. Creatures hostile to you \
             do not step in. |",
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
        assert!(text("materials.md").contains(
            "| Ruby | - | `#d3213b` | 100% | - | - | ×1 | yes | gold and ruby amulets, loose gems, \
             and items set with one |"
        ));
        // Materials (#193, #194): their properties on steel's scale, how often they are found,
        // what each item may be made of, and a long sword and chain mail in each metal.
        assert!(text("materials.md").contains(
            "| Bronze | metal | `#cd7f32` | 112% | +0 / −1 | 80% | ×0.8 | no | bronze keys, minster reliquaries |"
        ));
        assert!(
            text("material-finds.md")
                .contains("| Steel | 300 (common) | +60 | 1 | 20 (rare) | +15 |")
        );
        let construction = text("construction.md");
        assert!(construction.contains("| Long sword | steel | any metal, dragonbone |"));
        assert!(construction.contains("| Gold ring | gold | any material |"));
        assert!(!construction.contains("| Rock |"));
        assert!(text("variants.md").contains("| Bronze | 2 kg | +0 / −1 | 65 gold |"));
        // Gems: their worth and how often they are found, the sizes and cuts, and ornaments.
        assert!(text("gems.md").contains("| Diamond | `#e8f4ff` | 400 gold | 1.9% |"));
        assert!(text("gem-sizes.md").contains("| Small (plain) | ×1 | 1 g | 33% |"));
        assert!(text("gem-cuts.md").contains("| Uncut (never set in an item) | ×0.25 | 45% |"));
        let ornaments = text("ornaments.md");
        assert!(ornaments.contains("| Rings | as its setting, in its gem's colour | 35% |"));
        assert!(ornaments.contains("as their tip instead: ironwood staves, quarterstaves."));
        assert!(text("objects.md").contains("| Gem | `*` | gems | its gem | 1–4 g | its gem's |"));
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
        assert!(
            benefits.contains("| 1 | - | −3 | −7 |")
                && benefits.contains("| 9 | +2 | none | none |")
        );
        assert!(
            benefits.contains("| 8 | +6 | +1 roll | 15% less |"),
            "{benefits}"
        );
        assert!(text("classes.md").contains("| Twin Blades 1 or Duelist 1 |"));
        // Weapon abilities: the weapon skills' and the styles' apart, exactly.
        let weapon = text("weapon-abilities.md");
        assert!(
            weapon.contains(
                "| Deep cut | Daggers and knives, Short blades, Long blades, Great blades | 5 |"
            ),
            "{weapon}"
        );
        assert!(!weapon.contains("| Rend |"));
        let style = text("style-abilities.md");
        assert!(
            style.contains("| Rend | Twin Blades | 4 | triggered |"),
            "{style}"
        );
        assert!(style.contains("| Pinned on the one you struck, 1d3 degrees (one roll) |"));
        assert!(style.contains("Staggered on your attacker, 1 degree"));
        // The caster styles are the schools page's (where a spell is cast from).
        assert!(!style.contains("Pinning ray") && !styles.contains("Sniper"));
        let casters = text("caster-style-benefits.md");
        assert!(
            casters.contains("| 5 | +2 | +2 | 1 wider | +1 cell |")
                && casters.contains("| 4 | +2 | - | - | +1 |"),
            "{casters}"
        );
        let caster_abilities = text("caster-style-abilities.md");
        assert!(
            caster_abilities.contains("| Pinning ray | Sniper | 4 | triggered |"),
            "{caster_abilities}"
        );
        assert!(
            !text("weapon-skills.md").contains("Duelist"),
            "styles have their own tables"
        );
        assert!(text("objects.md").contains(
            "| Arrow | `/` | missiles | iron head, wood shaft | 30 g | 1 gold | yes | Missiles |"
        ));
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
        // Shields fit either hand (two-shield fighting, #611), after the weapons.
        assert!(equipment.contains(
            ", quarterstaff (both hands), ironwood staff (both hands), buckler, small shield, \
             large shield, tower shield |"
        ));
        assert!(
            equipment
                .contains("| `f` | Off hand | Stiletto, dagger, short sword, hand axe, buckler, ")
        );
        assert!(equipment.contains(
            "| `k` | Missile weapon | Short bow (shoots arrows), long war bow (shoots arrows), \
             composite bow (shoots arrows), crossbow (shoots crossbow bolts), heavy crossbow \
             (shoots crossbow bolts), sling (shoots sling stones), wand of burning hands, wand of \
             magic missile, wand of scorching ray, wand of sleep, wand of web, wand of fireball, \
             wand of lightning bolt, rod of terror, rod of digging, rod of sleep, rod of teleport \
             other, any other weapon, and anything made for throwing |"
        ));
        assert!(equipment.contains(
            "| `l` | Missiles | Sling stone, arrow, crossbow bolt, flaming arrow | 6 seconds |"
        ));
        assert!(equipment.contains("| `m` | Tool | Lantern, pick-axe, mattock | 6 seconds |"));
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
        assert!(
            abilities.contains(
                "| Steal | Skills | everyone trained in Steal | a direction | Lift a purse or a \
                 trinket from someone next to you who is not hostile; caught, they turn on you. | \
                 6 seconds |"
            ),
            "{abilities}"
        );
        assert!(
            abilities.contains(
                "| Pick lock | Skills | everyone trained in Open lock | a direction | Open a \
                 locked door next to you without its key. | 12 seconds |"
            ),
            "{abilities}"
        );
        assert!(
            abilities.contains(
                "| Disarm a trap | Skills | everyone trained in Disable device | a direction | Take \
                 apart a trap you know of next to you; a clumsy try sets it off. | 12 seconds |"
            ),
            "{abilities}"
        );
        assert!(!abilities.contains("| Disable device |"), "{abilities}");
        // Stepping past a known trap: 55% at dodge 0, 10% a point, from 15% to 90%.
        let avoid = text("trap-avoid.md");
        assert!(
            avoid.contains("| \u{2212}4 or less | 15% | Dexterity 3 or less |"),
            "{avoid}"
        );
        assert!(
            avoid.contains("| +0 | 55% | Dexterity 10 or 11 |"),
            "{avoid}"
        );
        assert!(
            avoid.contains("| +2 | 75% | Dexterity 14 or 15; Dexterity 10 with Tumble 40 |"),
            "{avoid}"
        );
        assert!(
            avoid.contains(
                "| +4 or more | 90% | Dexterity 18 or more; Dexterity 10 with Tumble 80 |"
            ),
            "{avoid}"
        );
        // Disarming: an arrow trap (14) at Dexterity 10 is 66 of 144 rolls, set off by 36 (a
        // natural 9 or less); at Dexterity 14 (+2) 89 and 21; with Disable device 100 (+8 more)
        // all but a natural 2 or 3, and never set off.
        let disarm = text("trap-disarm.md");
        assert!(
            disarm.contains(
                "| Arrow trap | 14 | 46% disarmed, 25% set off | 62% disarmed, 15% set off | 98% \
                 disarmed, 0% set off |"
            ),
            "{disarm}"
        );
        assert!(text("materials.md").contains(
            "| Steel | metal | `#a8b0b8` | 100% | +0 / +0 | 100% | ×1 | no | stilettos, daggers, \
             short swords, scimitars,"
        ));
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
                 bonus to Defence | dodging an attack or a known trap (40%) |"
            ),
            "{skills}"
        );
        assert!(skills.contains("| Swim | read by rules |"));
        assert_eq!(skills.matches("\n| ").count(), 1 + 29, "{skills}");
        let benefits = text("skill-benefits.md");
        assert!(
            benefits.contains(
                "| Value | Appraise: identifying, each hour | Athletics: kicking doors in | Athletics: speed | \
                 Balance: Burdened / Strained | Concentration: mana back | Concentration: mana pool | \
                 Decipher script: study check | Detect traps: finding traps | Disable device: disarming traps | \
                 Heal: treating | Initiative: initiative check | Initiative: surprise's extra cost | \
                 Listen: hears fainter by | Metalcraft: tempering | Negotiation: prices | Open lock: picking locks | Search: searching | \
                 Search: search reaches | Sneak attack: sneak attack dice | Sneak attack: their check's difficulty | \
                 Spellcraft: study check | Spot: noticing | Steal: \
                 stealing | Stealth: noise | Stealth: slipping past | Survival: journeys | Survival: foraging | Tumble: \
                 Dodge |"
            ),
            "{benefits}"
        );
        let foraging = text("foraging.md");
        assert!(
            foraging.contains("A forage takes 10 minutes."),
            "{foraging}"
        );
        assert!(
            foraging.contains(
                "| forest | 40% | `1d3` | handful of berries 27%, handful of hazelnuts 27%, \
                 mushroom 27%, angelica root 9.1%, apple 9.1% |"
            ),
            "{foraging}"
        );
        assert!(benefits.contains(
            "| 100 | 12% | +8 Strength | +5% | ×1.1 / ×1.2 | +40% | +5 mana | +10 | +8 | +8 | +8 | +8 | \u{2212}50% \
             | 10 | +8 | 20% better | +8 | +8 | +1 cell | 5d6 | +8 | +10 | +8 | +8 | \u{2212}60% | +8 | \u{2212}20% | +15% | +5 |"
        ));
        assert!(benefits.contains(
            "| 150 | 16% | +10 Strength | +7% | ×1.08 / ×1.15 | +50% | +6 mana | +15 | +10 | +10 | +10 | +10 \
             | \u{2212}62% | 15 | +10 | 25% better | +10 | +10 | +1 cell | 6d6 | +10 | +15 | +10 | +10 | \u{2212}75% | +10 | \u{2212}20% | +22% | +7 |"
        ));
        // Treating oneself: the difficulty and odds of each ailment.
        let heal = text("heal-targets.md");
        assert!(
            heal.contains("| Bleeding (1 degree) | 8 | 85% | 99% | it stops |"),
            "{heal}"
        );
        assert!(heal.contains("| Grave rot | 15 |"), "{heal}");
        // Stealing: the odds by the thief's edge, a contest of even dice.
        let slipping = text("slipping-past.md");
        assert!(slipping.contains("for 60 turns at most"), "{slipping}");
        assert!(slipping.contains("| +0 | 53% | 47% |"), "{slipping}");
        let stealing = text("stealing.md");
        assert!(stealing.contains("| +0 | 53% | 47% |"), "{stealing}");
        assert!(stealing.contains("| \u{2212}6 | 22% | 78% |"), "{stealing}");
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
        assert!(races.contains(
            "trained in Spot, who notice everything but traps in passing: Ranger, Thief."
        ));
        assert!(
            races.contains("trained in Detect traps, who notice hidden traps in passing: Thief.")
        );
        assert!(
            races.contains("| Kobold | Hidden traps | more easily (+2) |"),
            "{races}"
        );
        let grants = text("skill-grants.md");
        assert!(grants.contains("| Halfling | Listen `1d6+4`, Stealth `1d6+4` | - |"));
        assert!(
            grants.contains("| Human | one of the class's skills `+2d4+4`, picked at random | - |")
        );
        assert!(
            grants.contains("| Drow | Spot `1d6+4`, Stealth `1d4+2` | Sneak attack `2d4` (40 %) |")
        );
        assert!(grants.contains(
            "| Commoner | Athletics `2d4+4`, Handle animal `2d4+4` | Heal `2d4` (15 %), \
             Metalcraft `2d4` (10 %), Survival `2d4` (30 %), Swim `2d4` (20 %), \
             Woodcraft `2d4` (20 %) |"
        ));
        let ladder = text("skill-ladder.md");
        assert!(
            ladder.contains("| 0 | `+1` | 1.0 | `1d3` | 2.0 |"),
            "{ladder}"
        );
        assert!(
            ladder.contains("| 800 | `4d5` | 12.0 | `4d5` | 12.0 |"),
            "{ladder}"
        );
        let rolls = text("skill-rolls.md");
        assert!(rolls.contains("| Bard | 4 | Bluff, Music |"), "{rolls}");
        assert!(rolls.contains("| Commoner | 2 | - |"), "{rolls}");
        assert!(rolls.contains("| Human | 1 |"), "{rolls}");
        assert!(
            rolls.contains("at 14 and one more for every 4 points"),
            "{rolls}"
        );
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
        // The races' gifts and flaws beyond their stats, by name; a race with none left out.
        let traits = text("race-traits.md");
        assert!(
            traits.contains("| Lizardfolk | Protection 1 (scales) | \u{2014} |"),
            "{traits}"
        );
        assert!(traits.contains("dazzled from brightness 250: sees 10 cells, to-hit \u{2212}1"));
        assert!(traits.contains("| +4 | eaten unharmed |"), "{traits}");
        assert!(!traits.contains("| Human |"), "{traits}");
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
        assert!(text("magic-rarity.md").contains("| Rare | 2% |"));
        assert!(text("magic-rarity.md").contains("| a prefix and a suffix | 20% |"));
        assert!(text("magic-bases.md").contains("| Long sword | weapons |"));
        let prefixes = text("magic-prefixes.md");
        assert!(prefixes.contains(
            "| Sharp | damage, added to every roll | physical damage | weapons, ammunition | 1: +1 · 3: +1–2"
        ));
        // Speed is exact: the share of the time it takes off.
        assert!(prefixes.contains(
            "| Fleet | you move faster | movement speed | boots | 1: 5–8% less time · 2: 5–9%"
        ));
        assert!(text("magic-suffixes.md").contains("| of the Bear | Strength | strength |"));
        assert!(text("magic-names.md").contains("| Weapons | Bite,"));
        assert!(text("magic-names.md").contains("| Tools | Tooth, Fang, Bite, Jaw, Maw |"));
        // The tier window and a drop's item level (docs/design/affixes-systems.md §3, §8).
        assert!(text("magic-rarity.md").contains("at least 3 below it (an item of level 12"));
        assert!(text("magic-rarity.md").contains("has the creature's level as its item level"));
        // Unique items: the campaigns', exactly, never the test world's.
        let uniques = text("uniques.md");
        assert!(uniques.contains("| Widowmaker | Stiletto | 12 | Venomous (4: adds 1d4 poison"));
        assert!(uniques.contains("every hit that leaves its target alive: 1 degree of Poisoned"));
        assert!(uniques.contains("| its wearer is never Frightened |"));
        assert!(
            uniques.contains("| Mother Sallow |"),
            "who carries Widowmaker"
        );
        assert!(!uniques.contains("Stonebiter"));
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
        assert!(
            armour
                .contains("| Small shield | main hand or off hand | +1 | Shield | - | no limit |")
        );
        assert!(armour.contains("| Great helm | head | - | - | 2 | no limit | - |"));
        assert!(text("piercing.md").ends_with(
            "The crossbow bolt (2), the stiletto (2), and the war hammer (2) pierce armour, each \
             ignoring that much of the Protection of whatever it hits.\n"
        ));
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
        assert!(text("mana-regeneration.md").contains("| 10 | 4 minutes |"));
        assert!(text("mana.md").contains("| Wizard | 12 |"));
        assert!(text("mana-paces.md").contains("| In a sanctuary | three times as fast |"));
        assert!(text("spell-shapes.md").contains("| cone | yes | ×1.6 |"));
        assert!(text("spell-shapes.md").contains("| burst, radius 3 | yes | ×3 |"));
        assert!(
            text("spell-resolutions.md")
                .contains("next to you; may critical, the damage rolled again | ×0.7 |")
        );
        let list = text("spell-list.md");
        assert!(
            list.contains(
                "| burst, area | range 10, radius 2 | 2d6 fire damage; it melts the ice it reaches, and whoever stands on it falls in; it breaks walls | Intelligence 13: 3d6, 16: 4d6, 19: 5d6, 25: 5d6+2; range Willpower 14: 11, 18: 12 | 26 | 1 |"
            ),
            "{list}"
        );
        assert!(text("wands.md").contains("| wand of fireball | Fireball | 11 | 7 | 2d6 to 4d6 |"));
        assert!(text("spell-casting.md").contains("| Wizard | 100 % |"));
        let resources = text("resources.md");
        assert!(
            resources.contains(
                "| Rage | 20 | 12 | a point lost every 6 seconds | - | ends, and its bar goes, and \
                 Raging with it |"
            ),
            "{resources}"
        );
        assert!(resources.contains(
            "| Focus | 10 | full | a point back every 6 seconds, not while resting | - | stays, empty |"
        ));
        assert!(resources.contains(
            "| Stamina | 10 + the Toughness modifier + 1 per 10 levels (at least 5) | full | a point \
             back every 12 seconds | - | stays, empty |"
        ));
        assert!(resources.contains("| +1 for a kill |"));
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
        row_has(
            statuses,
            "| Bleeding | yes, \"bleeding\" | ",
            &[
                "Big hits and confirmed criticals; ",
                "the elder troll's claws (20%, 2 degrees)",
                "the forest troll's claws (20%, 1 degree)",
                "the giant leech's sucker (40%, 1 degree)",
                "the two-headed troll's bites (20%, 1 degree)",
                "| A hit point every 40 seconds, faster the worse it is | A degree a minute | Not at \
                 all | — | Bandage |",
            ],
        );
        for row in [
            "| A degree every 2 minutes | Half as fast |",
            "; eating rotten food |",
            "Strength \u{2212}1, Dexterity \u{2212}1 when strong",
        ] {
            assert!(statuses.contains(row), "{row} in {statuses}");
        }
        row_has(
            statuses,
            "| Poisoned | ",
            &[
                "the giant rat's bite (10%, 2 degrees)",
                "the grave worm's bite (20%, 1 degree)",
            ],
        );
        assert!(statuses.contains("| Diseased | no | Any disease, once it shows | Nothing |"));
        // Buffs and cooldowns: a raise, what it keeps you from, and what follows it.
        for row in [
            "| Raging | yes, \"raging\" | Holding Rage | Nothing | When rage ends (the Resources \
             page) |",
            "| Strength +2, Toughness +2, Intelligence \u{2212}2 | — | You cannot read, study, or \
             cast spells; Exhausted follows it |",
            "| Exhausted | no | The end of Raging | Nothing |",
        ] {
            assert!(statuses.contains(row), "{row} in {statuses}");
        }
        let diseases = text("diseases.md");
        row_has(
            diseases,
            "| Filth fever | ",
            &[
                "the giant rat's bite (10%)",
                "| Toughness, 54% | 1d3 days | Every day | 1d3 Dexterity, 1d3 Toughness | Two checks \
                 won in a row | no | Half as fast |",
            ],
        );
        row_has(
            diseases,
            "| Marsh ague | ",
            &["crossing marsh or swamp (4% an hour)"],
        );
        row_has(
            diseases,
            "| Grave rot | ",
            &["the zombie's fist (15%)", "eating rotten food |"],
        );
        row_has(
            diseases,
            "| Mummy rot | ",
            &[
                "mummy's fist (20%)",
                "| Toughness, 31% | 1d3 days | Every 3 days | 1d4 Toughness | Never by itself | no |",
            ],
        );
        row_has(
            diseases,
            "| Burrowing worms | ",
            &[
                "wormhost's fist (25%)",
                "| Toughness, 25% | 1 day | Every day | 1d4 Toughness, 1d3 Intelligence, 2d6 hit \
                 points | Two checks won in a row | yes |",
            ],
        );
        assert!(diseases.contains("| Never by itself | yes |"));
        assert!(diseases.contains("| Brain fever | — | Willpower, 54% |"));
        assert!(healing.contains("| Diseased with marsh ague | a quarter as fast |"));
        // The races' potentials have their own table.
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
        // The signs' gifts with their exact rules, and the lucky days with their odds by race.
        let animals = text("horoscope-animals.md");
        assert!(
            animals.contains("| 1 | Salamander | **Fleet-footed**: speed +5 %<br>**Slip away**"),
            "{animals}"
        );
        assert!(animals.contains("**Slight** (flaw)"), "{animals}");
        // The sign-limited picks, by sign in the Great Race's order, the elements after.
        let picks = text("horoscope-talents.md");
        assert!(
            picks.contains(
                "| Salamander | Hare-footed | level 12, to be born in the year of the \
                            Salamander | speed +5 % |"
            ),
            "{picks}"
        );
        assert!(
            picks.find("| Barghest |") < picks.find("| Wood-born |"),
            "{picks}"
        );
        let lucky = text("horoscope-lucky.md");
        assert!(
            lucky.contains("| Metal Manticore | 3 | the Iron Scourge |"),
            "{lucky}"
        );
        assert!(
            lucky.contains(
                "| Human | 16 to 35 | the Deep One (1415); the Gate Warden (1428); \
                            the Iron Scourge (1419) | 212 |"
            ),
            "{lucky}"
        );
        // Starting kits, items as named, with how they are kept and when they come.
        let kits = text("class-kits.md");
        assert!(
            kits.contains(
                "| Commoner | a club (wielded); 2 turnips; a loaf of bread; 12 copper coins |"
            ),
            "{kits}"
        );
        assert!(
            kits.contains("a dagger (wielded, with Twin Blades); a stiletto (with Duelist)"),
            "{kits}"
        );
        assert!(
            kits.contains("a gold ring (on left hand, magic or rare)"),
            "{kits}"
        );
        assert!(
            text("race-kits.md")
                .contains("| Elf | a hardwood long war bow (readied); 20 arrows (quivered); ")
        );
        // Every class is listed, in order, with its objects.
        let classes = text("item-classes.md");
        assert!(
            classes.contains("| Melee weapons | Battle axes, "),
            "{classes}"
        );
        assert!(
            classes.contains(
                "| Food | Angelica roots, apples, food rations, handfuls of berries, handfuls of \
                 hazelnuts, loaves of bread, mushrooms, strips of dried meat, turnips, wafers of \
                 waybread, corpses |"
            ),
            "{classes}"
        );
        let at = |row: &str| classes.find(row).unwrap_or_else(|| panic!("{row}"));
        assert!(at("| Melee weapons |") < at("| Gems | Gems |"));
        assert!(at("| Gems | Gems |") < at("| Miscellaneous | Boulders, long-stemmed pipes"));
        assert!(
            text("creatures.md")
                .contains("| Giant rat | `r` | Vermin | hostile | up close | no | no |")
        );
        assert!(text("creatures.md").contains("| Deer | `d` | Wildlife | neutral | never |"));
        assert!(text("creatures.md").contains("shoots from afar"));
        // Monsters fight the player alone but for the story feuds and the arena's rats (#612).
        row_has(text("factions.md"), "| Kobolds | ", &["you, warren rats |"]);
        row_has(text("factions.md"), "| Demons | ", &["you, devils |"]);
        assert!(text("factions.md").contains("| Wildlife | nobody |"));
        assert!(text("factions.md").contains("| Cultists | you | no |"));
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
        row_has(
            text("lights.md"),
            "| Torch carried by ",
            &[
                "a balor",
                "a burning skull",
                "a kobold",
                "a nightmare",
                " | 6 cells |",
            ],
        );
        for row in [
            "| Wall torch | 7 cells |",
            "| Your lantern | 5 cells |",
            "| Light crystal (red) | 7 cells |",
            "| Faint rune | 4 cells | UV only",
        ] {
            assert!(text("lights.md").contains(row), "{row}");
        }
        let scenarios = text("scenarios.md");
        // The playable campaign only: the test world leaves the player wiki.
        assert!(scenarios.contains("| The Ascmearc | 08:00, 11 Darkening, Year 1444 |"));
        assert!(!scenarios.contains("| Showcase"), "{scenarios}");
        // Only the start a release's New run starts: the dev scenarios are left out.
        assert!(!scenarios.contains("| Cēaptūn |"), "{scenarios}");
        let places = text("overworld-places.md");
        for row in [
            "| Beorggeat | (12, 28) | Grægweall | Mountain pass | — | — |",
            "| Fennmynster | (70, 48) | Micelfenn | Dungeon entrance | Fennmynster 1 | until a quest reveals it |",
        ] {
            assert!(places.contains(row), "{row}\n{places}");
        }
        // The map: 60 rows of 120 cells, the hidden ways in drawn (the wiki tells the truth),
        // and its key; the regions in the order they match.
        let map = text("overworld-map.md");
        assert!(map.contains("**The Ascmearc**, 120 by 60 cells"), "{map}");
        let rows: Vec<&str> = (map
            .split("```text\n")
            .nth(1)
            .and_then(|m| m.split("```").next()))
        .expect("a code block")
        .lines()
        .collect();
        assert_eq!(rows.len(), 60);
        assert!(rows.iter().all(|r| r.chars().count() == 120));
        assert!(map.contains("| Mountain pass |"), "{map}");
        let regions = text("overworld-regions.md");
        assert!(regions.contains("| Cēapdenu |"), "{regions}");
        assert!(!regions.contains("Wildlands"), "{regions}");
        // Hunger: the states by nutrition, each race's need, food, and corpses (what eating them
        // does is a table of its own).
        let states = text("hunger-states.md");
        assert!(
            states.contains("| Hungry | 720 to 1439 | 36 hours |"),
            "{states}"
        );
        assert!(states.contains("| Not hungry | 1440 to 3999 | (you start here) |"));
        assert!(text("hunger-races.md").contains("| Halfling | ×1.25 | 1800 |"));
        assert!(text("food.md").contains("| Food ration | 1500 | 25 hours | 4 | 500 g |"));
        // How long food keeps, exactly.
        assert!(text("food.md").contains("| 30 days | 60 days |"));
        assert!(text("food.md").contains("| 5 days | 10 days |"));
        assert!(text("corpses.md").contains("| Kobold corpse | Small | 25 kg | 400 |"));
        assert!(text("corpses.md").contains("| 8 hours | 36 hours |"));
        assert!(!text("objects.md").contains("corpse"));
        assert!(!text("corpses.md").contains("terrible"));
    }
}

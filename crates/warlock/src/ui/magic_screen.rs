//! The Magic page (spec §6; page 6 of the info pages; the spell scaling design §11): what the
//! player has trained of magic and what they cast with it, read, never used. One column that
//! scrolls (↑↓) on every console, in order: their mana as the status bar shows it; the seven
//! schools of magic (spec §8 "Weapon skills"), each with its rank and the faint bar of its marks
//! toward the next, as the Weapons page draws a skill; the caster styles (Sniper, Toucher, Battle
//! caster), the one cast in now marked; and the spells known, by level then name (the abilities
//! menu's order), each with the roll it makes now and its mana as the menu's row says them
//! ("4d6 fire, 26 mana, aimed"). Never what a rank gives: perception has none of it, and the wiki
//! documents the rules.
//!
//! A character with no mana and no spells sees the schools at rank 0, no styles, and "You know no
//! spells." No key opens it in play: it is reached by its number or Tab from any info page (the
//! Weapons page's hint line names it). Esc or Enter closes it. It is read from the perception
//! snapshot when it opens; no game time passes while it is open, so it cannot go stale.

use super::abilities_screen::{needs, spell_head};
use super::menu::{Hints, Nav, Panel, Press};
use super::screens::ScreenAction;
use super::weapon_skills_screen::{BAR, IN_USE, draw_bar};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use std::sync::atomic::{AtomicUsize, Ordering};
use warlock_core::ability::Ability;
use warlock_core::content::Role;
use warlock_core::perception::PerceptionView;

/// Width of the name column (a skill's, a spell's).
const NAME_W: i32 = 24;
/// Width of a line: indent, name, and a spell's roll and mana; legends end at its right edge.
const LINE_W: i32 = 2 + NAME_W + 36;
/// Width of the panel's own: the line in the frame and its margins.
const WIDTH: i32 = LINE_W + 6;
/// What the caster styles' heading says while one is marked.
const LEGEND: &str = "the style you cast in now";
/// What the spells' section says when there are none.
const NO_SPELLS: &str = "You know no spells.";

/// A row of the page.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Row {
    /// The player's mana: `Mana 47/106`.
    Mana(String),
    /// A section's heading (`Schools`), with the legend of the mark while one is marked.
    Section { name: &'static str, legend: bool },
    /// A school or a caster style: its name, its rank, how many of the bar's cells its marks
    /// fill (`None` at the last rank), and whether it is the style cast in now.
    Skill {
        name: String,
        rank: u8,
        filled: Option<u32>,
        in_use: bool,
    },
    /// A spell known: its name and what the abilities menu says of it.
    Spell { name: String, detail: String },
    /// A line of its own, dim ("You know no spells.").
    Note(&'static str),
    /// A gap between sections.
    Gap,
}

#[derive(Debug)]
pub struct MagicScreen {
    rows: Vec<Row>,
    /// The first row shown, scrolled with ↑↓ when the rows do not fit.
    top: usize,
    /// How far the list could scroll when it was last drawn (0: it all fits), so ↓ stops at the
    /// end. Drawing knows the console; key presses do not.
    most: AtomicUsize,
}

impl MagicScreen {
    /// The page of what the player perceives of their magic (`None` outside a run: nothing).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        Self {
            rows: view.map_or(Vec::new(), rows),
            top: 0,
            most: AtomicUsize::new(0),
        }
    }

    /// Applies a key press: ↑↓ scroll a page too long for the console; Esc or Enter closes it.
    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        match press.nav {
            Nav::Back | Nav::Enter => return ScreenAction::Close,
            Nav::Up => self.top = self.top.saturating_sub(1),
            Nav::Down => self.top = (self.top + 1).min(self.most.load(Ordering::Relaxed)),
            _ => {}
        }
        ScreenAction::Stay
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Rows: frame, margin, the rows, a gap, the footer, frame. In a console too short for
        // them all, the gaps between sections, the margin, and the gap above the footer give
        // way; then the rows scroll.
        let compact = self.rows.len() as i32 + 5 > console.rows() as i32;
        let around = if compact { 3 } else { 5 };
        let rows: Vec<&Row> = (self.rows.iter())
            .filter(|r| !(compact && **r == Row::Gap))
            .collect();
        let body = rows.len().max(1) as i32;
        let width = super::info_tabs::panel_width(WIDTH);
        let panel = Panel::new(console, " Magic ", width, body + around);
        super::info_tabs::draw_tabs(console, &panel, bindings, super::ScreenName::Magic);
        let room = (panel.h - around).max(0);
        let most = (body - room).max(0) as usize;
        self.most.store(most, Ordering::Relaxed);
        let first = self.top.min(most);
        let top = panel.y + if compact { 1 } else { 2 };
        let x = panel.inner_x();
        for (y, row) in (top..).zip(rows.iter().skip(first).take(room as usize)) {
            draw_row(console, x, y, row);
        }
        // Arrows on the frame where rows are hidden above or below.
        let edge = panel.x + panel.w - 1;
        if first > 0 && room > 0 {
            console.print(edge, top, "↑", TEXT_DIM);
        }
        if first < most && room > 0 {
            console.print(edge, top + room - 1, "↓", TEXT_DIM);
        }
        let footer = if most > 0 {
            format!("{} scroll   Esc close", Hints::of(bindings).up_down())
        } else {
            "Esc close".to_string()
        };
        panel.footer(console, &footer);
    }
}

/// The page's rows for what `view` shows of the player.
fn rows(view: &PerceptionView<'_>) -> Vec<Row> {
    let content = view.content();
    let table = content.weapon_skills();
    let own = view.own_weapon_skills();
    let in_use = view.own_styles_in_use();
    let skill = |id| Row::Skill {
        name: table.skill(id).name.clone(),
        rank: own.rank(id),
        filled: (own.progress(table, id)).map(|(marks, need)| marks * BAR / need.max(1)),
        in_use: in_use.contains(&id),
    };
    let spells: Vec<Row> = (view.abilities().into_iter())
        .filter_map(|own| match own.ability {
            a @ Ability::Spell(spell) => {
                let head = spell_head(view, spell);
                let detail = if needs(a.aim(content)).is_empty() {
                    head
                } else {
                    format!("{head}, aimed")
                };
                Some(Row::Spell {
                    name: a.name(content).to_string(),
                    detail,
                })
            }
            _ => None,
        })
        .collect();
    let mut rows = Vec::new();
    let (mana, max) = view.own_mana();
    if max > 0 {
        rows.extend([Row::Mana(format!("Mana {mana}/{max}")), Row::Gap]);
    }
    rows.push(Row::Section {
        name: "Schools",
        legend: false,
    });
    rows.extend(
        (table.ids())
            .filter(|&(id, def)| def.role == Role::School && own.shows(table, id))
            .map(|(id, _)| skill(id)),
    );
    // The caster styles, for a caster: one who knows a spell, or has marks or a rank in one of
    // them (everyone has a little mana; a warrior is no caster).
    let casters: Vec<_> = (table.styles())
        .filter(|&(id, _, style)| style.grip.casts() && own.shows(table, id))
        .map(|(id, _, _)| id)
        .collect();
    let trained = (casters.iter())
        .any(|&id| own.rank(id) > 0 || own.progress(table, id).is_some_and(|(marks, _)| marks > 0));
    let styles: Vec<Row> = casters.into_iter().map(skill).collect();
    if !spells.is_empty() || trained {
        let legend = (styles.iter()).any(|r| matches!(r, Row::Skill { in_use: true, .. }));
        rows.extend([
            Row::Gap,
            Row::Section {
                name: "Caster styles",
                legend,
            },
        ]);
        rows.extend(styles);
    }
    rows.extend([
        Row::Gap,
        Row::Section {
            name: "Spells",
            legend: false,
        },
    ]);
    if spells.is_empty() {
        rows.push(Row::Note(NO_SPELLS));
    }
    rows.extend(spells);
    rows
}

/// One row at (`x`, `y`).
fn draw_row(console: &mut Console, x: i32, y: i32, row: &Row) {
    match row {
        Row::Mana(text) => {
            console.print(x, y, text, TEXT);
        }
        Row::Section { name, legend } => {
            console.print(x, y, name, TEXT_DIM);
            if *legend {
                let at = x + LINE_W - LEGEND.chars().count() as i32 - 2;
                console.print(at, y, &IN_USE.to_string(), ACCENT);
                console.print(at + 2, y, LEGEND, TEXT_DIM);
            }
        }
        Row::Skill {
            name,
            rank,
            filled,
            in_use,
        } => {
            // Untrained ones read dimmer, so what the player knows stands out.
            let fg = if *rank == 0 { TEXT_DIM } else { TEXT };
            if *in_use {
                console.print(x, y, &IN_USE.to_string(), ACCENT);
            }
            console.print(x + 2, y, name, fg);
            console.print(x + 2 + NAME_W, y, &format!("{rank:>2}"), fg);
            if let Some(filled) = filled {
                draw_bar(console, x + NAME_W + 6, y, *filled);
            }
        }
        Row::Spell { name, detail } => {
            console.print(x + 2, y, name, TEXT);
            console.print(x + 2 + NAME_W, y, detail, TEXT_DIM);
        }
        Row::Note(text) => {
            console.print(x + 2, y, text, TEXT_DIM);
        }
        Row::Gap => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use warlock_core::World;
    use warlock_core::action::parse_actions;
    use warlock_core::character::CharacterSpec;
    use warlock_core::content::Content;

    /// A wizard of level 18: four spells.
    const LEVELLED: &str = "xp:200000 levelup:auto";

    fn world(scenario: &str, character: &str, actions: &str) -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        let spec: CharacterSpec = character.parse().expect("character");
        let mut world =
            World::start_run(content, scenario, None, Some(&spec), None, true).expect("world");
        for action in parse_actions(actions).expect("actions") {
            world.apply(&action);
        }
        world
    }

    fn drawn(screen: &MagicScreen, cols: u32, rows: u32) -> Vec<String> {
        let mut console = Console::default();
        console.resize(cols, rows);
        screen.draw(&mut console, &Bindings::default());
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect()
            })
            .collect()
    }

    fn find<'a>(text: &'a [String], needle: &str) -> &'a str {
        (text.iter().find(|l| l.contains(needle)))
            .unwrap_or_else(|| panic!("{needle} in {text:#?}"))
    }

    #[test]
    fn a_wizard_sees_mana_schools_styles_and_spells_in_order() {
        let world = world("showcase", "human,female,wizard", LEVELLED);
        let screen = MagicScreen::new(Some(&world.perception()));
        let text = drawn(&screen, 120, 50);
        let at = |needle: &str| text.iter().position(|l| l.contains(needle));
        assert!(find(&text, " Magic ").contains('─'));
        assert!(find(&text, "Mana ").contains('/'));
        assert!(at("Mana ") < at("Schools") && at("Schools") < at("Caster styles"));
        assert!(at("Caster styles") < at("Spells") && at("Spells") < at("Magic missile"));
        for school in ["Evocation", "Necromancy", "Divination"] {
            find(&text, school);
        }
        assert!(find(&text, "Sniper").contains("Sniper                   1"));
        // A spell's row is the abilities menu's: its roll, its mana, aimed.
        let missile = find(&text, "Magic missile");
        assert!(missile.contains(" mana, aimed"), "{missile}");
        assert!(find(&text, "Esc close").contains("Esc close"));
        assert!(at(NO_SPELLS).is_none());
    }

    #[test]
    fn armour_that_hinders_casting_marks_battle_caster() {
        // The paladin's kit hinders nothing: no mark, no legend.
        let leathers = world("forge", "human,male,paladin", "");
        let text = drawn(&MagicScreen::new(Some(&leathers.perception())), 120, 50);
        find(&text, "Battle caster");
        assert!(!text.join("\n").contains(IN_USE));
        // In the forge's chain mail: Battle caster marked, and the heading's legend.
        let mail = world(
            "forge",
            "human,male,paladin",
            "move:SW move:S pickup equip:body:l",
        );
        let text = drawn(&MagicScreen::new(Some(&mail.perception())), 120, 50);
        let marked = format!("{IN_USE} Battle caster");
        assert!(find(&text, "Battle caster").contains(&marked), "{text:#?}");
        assert!(find(&text, "Caster styles").contains(&format!("{IN_USE} {LEGEND}")));
        assert_eq!(text.iter().filter(|l| l.contains(IN_USE)).count(), 2);
    }

    #[test]
    fn a_warrior_sees_the_schools_unranked_and_no_spells() {
        let world = world("showcase", "human,male,warrior", "");
        let screen = MagicScreen::new(Some(&world.perception()));
        let text = drawn(&screen, 120, 50);
        find(&text, NO_SPELLS);
        find(&text, "Evocation");
        let at = |needle: &str| text.iter().position(|l| l.contains(needle));
        assert!(at("Caster styles").is_none() && at("Sniper").is_none());
        for hidden in ["+", "%", "to-hit", "marks"] {
            assert!(!text.join("\n").contains(hidden), "{hidden:?}");
        }
    }

    #[test]
    fn a_page_too_long_scrolls_and_closes_on_esc_or_enter() {
        let world = world("showcase", "human,female,wizard", LEVELLED);
        let mut screen = MagicScreen::new(Some(&world.perception()));
        let text = drawn(&screen, 80, 16);
        let has = |text: &[String], needle: &str| text.iter().any(|l| l.contains(needle));
        assert!(has(&text, "Schools") && !has(&text, "Magic missile"));
        assert!(has(&text, "↑↓ scroll"));
        for _ in 0..60 {
            assert_eq!(screen.handle(&Press::nav(Nav::Down)), ScreenAction::Stay);
        }
        let text = drawn(&screen, 80, 16);
        assert!(
            has(&text, "Magic missile") && !has(&text, "Schools"),
            "{text:#?}"
        );
        assert_eq!(screen.handle(&Press::nav(Nav::Back)), ScreenAction::Close);
        assert_eq!(screen.handle(&Press::nav(Nav::Enter)), ScreenAction::Close);
        // Outside a run, and in tiny consoles, it draws without a panic.
        let empty = MagicScreen::new(None);
        for (cols, rows) in [(30, 8), (5, 3), (0, 0)] {
            assert_eq!(drawn(&empty, cols, rows).len(), rows as usize);
            assert_eq!(drawn(&screen, cols, rows).len(), rows as usize);
        }
    }
}

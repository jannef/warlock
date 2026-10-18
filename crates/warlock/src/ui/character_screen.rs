//! The character screen (spec §6; `@`): who the player is, as far as they know it. Their name,
//! their race, gender, and class, their age, birthday, and birth sign (spec §8 "Character
//! creation"; when the run has a character), their score in each stat (spec §8 "Stats") with its
//! abbreviation and full name, their hit points and Defence (spec §8 "Combat"; the total only),
//! and what they weigh. Never a modifier, a potential, or a to-hit: perception has none of them.
//!
//! Read-only: Esc, Enter, or its own key closes it. It is read from the perception snapshot when
//! it opens; no game time passes while it is open, so it cannot go stale.

use super::menu::{Nav, Panel, Press};
use super::screens::ScreenAction;
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use warlock_core::character::{Character, Gender};
use warlock_core::content::Content;
use warlock_core::object::{Weight, format_weight};
use warlock_core::perception::PerceptionView;
use warlock_core::stats::{PerStat, Stat};
use warlock_core::time::GameTime;

/// Width of the stat name column: the longest name and a space.
const NAME_W: i32 = 14;

/// Width of the panel.
const WIDTH: i32 = 48;

#[derive(Debug)]
pub struct CharacterScreen {
    name: String,
    /// Race, gender, and class, by name; empty for a run without a character.
    identity: Vec<(&'static str, String)>,
    /// Age, birthday, and birth sign ([`birth_parts`]); empty without a character.
    birth: Vec<String>,
    scores: PerStat<u8>,
    weight: Weight,
    /// Hit points left and the most.
    hp: (i32, i32),
    /// Defence: the total only.
    defence: i32,
}

impl CharacterScreen {
    /// The screen for what the player perceives of themselves (`None` outside a run: average
    /// scores and no weight).
    pub fn new(view: Option<&PerceptionView<'_>>) -> Self {
        let Some(view) = view else {
            return Self {
                name: "You".into(),
                identity: Vec::new(),
                birth: Vec::new(),
                scores: PerStat::splat(warlock_core::stats::AVERAGE),
                weight: Weight::default(),
                hp: (0, 0),
                defence: warlock_core::combat::BASE_DEFENCE,
            };
        };
        let content = view.content();
        let name = content
            .creature_id("player")
            .map_or("you", |id| content.creature(id).name.as_str());
        let identity = view.character().map_or(Vec::new(), |c| {
            let spec = &c.spec;
            vec![
                (
                    "Race",
                    (spec.race(content)).map_or(spec.race.clone(), |r| r.name.clone()),
                ),
                ("Gender", gender_text(c)),
                (
                    "Class",
                    (spec.class(content)).map_or(spec.class.clone(), |k| k.name.clone()),
                ),
            ]
        });
        let birth = (view.character()).map_or(Vec::new(), |c| birth_parts(c, content, view.time()));
        Self {
            name: capitalized(name),
            identity,
            birth,
            scores: view.own_scores(),
            weight: view.own_weight(),
            hp: view.own_hp(),
            defence: view.own_defence(),
        }
    }

    /// Applies a key press: Esc, Enter, or the screen's own key closes it.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        let own_key = press
            .binding
            .as_ref()
            .is_some_and(|key| bindings.owner(key) == Some(KeyCommand::Character));
        if own_key || matches!(press.nav, Nav::Back | Nav::Enter) {
            ScreenAction::Close
        } else {
            ScreenAction::Stay
        }
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Rows: margin, the name, a gap, race, gender, class, the birth, and a gap (with a
        // character), the stats, a gap, hit points and Defence, a gap, the weight, a gap, the
        // footer, frame.
        let birth = fit_parts(&self.birth, WIDTH.min(console.cols() as i32) - 4);
        let identity_rows = match self.identity.len() {
            0 => 0,
            n => (n + birth.len()) as i32 + 1,
        };
        let h = Stat::COUNT as i32 + 12 + identity_rows;
        let panel = Panel::new(console, " Character ", WIDTH, h);
        let x = panel.inner_x();
        // Rows from the footer's down would draw over it, or the frame, of a panel cut short.
        let fits = |y: i32| y < panel.y + panel.h - 2;
        let mut y = panel.y + 2;
        if fits(y) {
            console.print(x, y, &self.name, ACCENT);
        }
        y += 2;
        for (label, value) in &self.identity {
            // `Race    Dwarf`: the value lines up with the stat names.
            if fits(y) {
                console.print(x, y, label, TEXT_DIM);
                console.print(x + 8, y, value, TEXT);
            }
            y += 1;
        }
        for line in &birth {
            if fits(y) {
                console.print(x, y, line, TEXT);
            }
            y += 1;
        }
        y += i32::from(!self.identity.is_empty());
        for (stat, score) in self.scores.iter() {
            // `St  Strength      12`: the abbreviation, the name, the score right-aligned.
            if fits(y) {
                console.print(x, y, stat.abbrev(), TEXT_DIM);
                console.print(x + 4, y, stat.name(), TEXT);
                console.print(x + 4 + NAME_W, y, &format!("{score:>2}"), TEXT);
            }
            y += 1;
        }
        y += 1;
        // `HP  Hit points  18/24` and `Dv  Defence  15`, in the stats' columns.
        let (hp, max) = self.hp;
        for (abbrev, name, value) in [
            ("HP", "Hit points", format!("{hp}/{max}")),
            ("Dv", "Defence", format!("{:>2}", self.defence)),
        ] {
            if fits(y) {
                console.print(x, y, abbrev, TEXT_DIM);
                console.print(x + 4, y, name, TEXT);
                console.print(x + 4 + NAME_W, y, &value, TEXT);
            }
            y += 1;
        }
        y += 1;
        if fits(y) {
            let w = &self.weight;
            let weight = match w.carried {
                0 => format!("Weight {}, carrying nothing", format_weight(w.total())),
                carried => format!(
                    "Weight {} (you {}, carrying {})",
                    format_weight(w.total()),
                    format_weight(w.base),
                    format_weight(carried),
                ),
            };
            console.print(x, y, &weight, TEXT);
        }
        let footer = match bindings.keys_of(KeyCommand::Character).first() {
            Some(key) => format!("{} or Esc close", key.display()),
            None => "Esc close".to_string(),
        };
        panel.footer(console, &footer);
    }
}

/// The gender as shown: its name, and for "other" the stat it raised (`Other (Willpower +1)`).
pub fn gender_text(character: &Character) -> String {
    match character.spec.gender {
        Gender::Other => format!("Other ({} +1)", character.raised.name()),
        gender => gender.name().to_string(),
    }
}

/// Age on `now`, birthday, and birth sign, as shown: `Age 27`, `born 3 Seedtide 1417`, `year of
/// the Fire Manticore` (no sign without a horoscope).
pub fn birth_parts(character: &Character, content: &Content, now: GameTime) -> Vec<String> {
    let mut parts = vec![
        format!("Age {}", character.age_on(now)),
        format!("born {}", character.born.text(content.calendar())),
    ];
    if let Some(sign) = character.sign_name(content) {
        parts.push(format!("year of the {sign}"));
    }
    parts
}

/// `parts` joined with ` · ` into lines of at most `width` characters, a new line where the next
/// part would not fit: `Age 27 · born 3 Seedtide 1417 · year of the Fire Manticore`.
pub fn fit_parts(parts: &[String], width: i32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for part in parts {
        match lines.last_mut() {
            Some(line) if (line.chars().count() + 3 + part.chars().count()) as i32 <= width => {
                line.push_str(" · ");
                line.push_str(part);
            }
            _ => lines.push(part.clone()),
        }
    }
    lines
}

/// `name` with its first letter in upper case.
fn capitalized(name: &str) -> String {
    let mut chars = name.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeyBinding;
    use std::sync::Arc;
    use warlock_core::content::Content;
    use warlock_core::{Action, Dir, Letters, World};

    fn showcase() -> World {
        let content = Arc::new(Content::builtin().expect("content"));
        World::new(content, "showcase", None).expect("world")
    }

    fn text(console: &Console) -> Vec<String> {
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect()
            })
            .collect()
    }

    fn drawn(world: &World, cols: u32, rows: u32) -> Vec<String> {
        let screen = CharacterScreen::new(Some(&world.perception()));
        let mut console = Console::default();
        console.resize(cols, rows);
        screen.draw(&mut console, &Bindings::default());
        text(&console)
    }

    #[test]
    fn shows_the_name_scores_and_weight() {
        let mut world = showcase();
        // Pick up the start room's jerkin, for something carried.
        world.apply(&Action::Move(Dir::SW));
        world.apply(&Action::Move(Dir::SW));
        world.apply(&Action::PickUp(Letters::of([0])));
        let text = drawn(&world, 80, 30);
        let row = |needle: &str| {
            text.iter()
                .find(|l| l.contains(needle))
                .unwrap_or_else(|| panic!("{needle} in {text:#?}"))
        };
        assert!(row(" Character ").contains('┌') || row(" Character ").contains('─'));
        assert!(
            row("You")
                .trim_matches(|c| c == ' ' || c == '│')
                .starts_with("You")
        );
        for expected in [
            "Race    Human",
            "Gender  Female",
            "Class   Thief",
            "Age 34 · born 29 Greenleaf 1410",
            "year of the Water Basilisk",
            "St  Strength       9",
            "Dx  Dexterity     13",
            "To  Toughness     10",
            "In  Intelligence  12",
            "Wi  Willpower     11",
            "Pr  Presence       9",
            // Toughness 10 and a human's 4 and a thief's 3; Defence 10 and Dexterity's share.
            "HP  Hit points    17/17",
            "Dv  Defence       11",
        ] {
            row(expected);
        }
        row("Weight 73.5 kg (you 70 kg, carrying 3.5 kg)");
        row("@ or Esc close");
        let empty_handed = drawn(&showcase(), 80, 30);
        assert!(
            (empty_handed.iter()).any(|l| l.contains("Weight 70 kg, carrying nothing")),
            "{empty_handed:#?}"
        );
    }

    #[test]
    fn never_shows_potentials_or_modifiers() {
        let text = drawn(&showcase(), 80, 30).join("\n");
        // The showcase player's potentials (16 each), and their modifiers (+2, -1), appear
        // nowhere; neither do signs or the words.
        for hidden in ["16", "+", "-", "otential", "odifier"] {
            assert!(!text.contains(hidden), "{hidden:?} in\n{text}");
        }
    }

    #[test]
    fn shows_the_chosen_character_and_nothing_without_one() {
        let content = Arc::new(Content::builtin().expect("content"));
        let spec = "half_orc,other,witch".parse().expect("a character");
        let world =
            World::with_character(content.clone(), "showcase", None, Some(&spec)).expect("world");
        let text = drawn(&world, 80, 30).join("\n");
        let raised = warlock_core::character::other_stat(1).name();
        for expected in [
            "Race    Half-orc",
            &format!("Gender  Other ({raised} +1)"),
            "Class   Witch",
            "year of the",
        ] {
            assert!(text.contains(expected), "{expected:?} in\n{text}");
        }
        let mut files = warlock_core::content::ContentFiles::builtin();
        files.insert(
            "scenarios/plain.toml",
            "name = \"P\"\nstart_level = \"hub\"\n",
        );
        let content = Arc::new(Content::load(&files).expect("valid"));
        let plain = World::new(content, "plain", None).expect("world");
        let text = drawn(&plain, 80, 30).join("\n");
        assert!(!text.contains("Race") && !text.contains("Class"), "{text}");
        assert!(!text.contains("Age") && !text.contains("born"), "{text}");
        assert!(text.contains("St  Strength      10"), "{text}");
    }

    #[test]
    fn the_age_counts_birthdays_and_the_parts_fit_the_width() {
        let world = showcase();
        let c = world.character().expect("a character").clone();
        let content = world.content().clone();
        let parts = birth_parts(&c, &content, world.time());
        assert_eq!(
            parts,
            [
                "Age 34",
                "born 29 Greenleaf 1410",
                "year of the Water Basilisk"
            ]
        );
        assert_eq!(
            fit_parts(&parts, 60),
            ["Age 34 · born 29 Greenleaf 1410 · year of the Water Basilisk"]
        );
        assert_eq!(
            fit_parts(&parts, 44),
            [
                "Age 34 · born 29 Greenleaf 1410",
                "year of the Water Basilisk"
            ]
        );
        assert_eq!(fit_parts(&parts, 5).len(), 3);
        // Past the birthday (29 Greenleaf), a year older.
        let later = warlock_core::time::GameTime::from_calendar(1445, 5, 29, 0, 0).expect("a date");
        assert_eq!(birth_parts(&c, &content, later)[0], "Age 35");
    }

    #[test]
    fn closes_on_its_key_esc_or_enter() {
        let b = Bindings::default();
        let mut screen = CharacterScreen::new(None);
        assert_eq!(screen.handle(&Press::typed('a'), &b), ScreenAction::Stay);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Down), &b),
            ScreenAction::Stay
        );
        let at = Press {
            binding: KeyBinding::from_text("@"),
            ..Press::typed('@')
        };
        assert_eq!(screen.handle(&at, &b), ScreenAction::Close);
        assert_eq!(
            screen.handle(&Press::nav(Nav::Back), &b),
            ScreenAction::Close
        );
        assert_eq!(
            screen.handle(&Press::nav(Nav::Enter), &b),
            ScreenAction::Close
        );
    }

    #[test]
    fn draws_in_small_consoles() {
        let world = showcase();
        for (cols, rows) in [(48, 15), (30, 8), (5, 3), (0, 0)] {
            let text = drawn(&world, cols, rows);
            assert_eq!(text.len(), rows as usize);
        }
        let screen = CharacterScreen::new(None);
        let mut console = Console::default();
        console.resize(40, 12);
        screen.draw(&mut console, &Bindings::default());
    }
}

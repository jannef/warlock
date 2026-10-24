//! The New run screen (spec §6): the scenarios from content and, in developer builds, an
//! optional seed. Enter goes on to character creation ([`super::chargen_screen`]), or with
//! content that has no races or classes, starts the run as the scenario's. A release build's run
//! gets a seed no one chooses ([`fresh_seed`], drawn when the screen opens), and the screen never
//! mentions it (#256).

use super::chargen_screen::ScenarioPick;
use super::menu::{Hints, Nav, Panel, Press, wrap};
use super::screens::{MenuChoice, ScreenAction, ScreenName};
use super::{TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use warlock_core::character::CharacterSpec;
use warlock_core::content::Content;
use warlock_core::time::GameTime;

/// A scenario as listed on the screen.
#[derive(Clone, Debug)]
struct ScenarioInfo {
    id: String,
    name: String,
    description: String,
    seed: u64,
    character: Option<CharacterSpec>,
    start: GameTime,
}

/// Longest seed text we accept.
const MAX_SEED_LEN: usize = 24;

/// Where the new run's seed comes from.
#[derive(Debug)]
enum Seed {
    /// Developer builds: the seed field's text; empty means the scenario's own seed.
    Field(String),
    /// Release builds: drawn when the screen opened, and never shown on it.
    Fresh(u64),
}

/// The New run screen: a scenario list and, in developer builds, a seed field.
#[derive(Debug)]
pub struct NewRunScreen {
    scenarios: Vec<ScenarioInfo>,
    /// The content has races and classes to choose a character from.
    creates_characters: bool,
    selected: usize,
    seed: Seed,
}

impl NewRunScreen {
    /// The screen for `content`'s scenarios. `fresh` is the seed of a release build's run, which
    /// has no seed field; `None` in developer builds, which show the field. A new run is a new
    /// character with a save of its own: it touches no other character's.
    pub fn new(content: &Content, fresh: Option<u64>) -> Self {
        let mut scenarios: Vec<ScenarioInfo> = content
            .scenarios()
            .map(|s| ScenarioInfo {
                id: s.id.clone(),
                name: s.name.clone(),
                description: s.description.clone(),
                seed: s.seed,
                character: s.character.clone(),
                start: s.start,
            })
            .collect();
        // The showcase (the command-line default) comes first and is preselected.
        scenarios.sort_by_key(|s| s.id != "showcase");
        Self {
            scenarios,
            creates_characters: !content.races().list.is_empty() && !content.classes().is_empty(),
            selected: 0,
            seed: match fresh {
                Some(seed) => Seed::Fresh(seed),
                None => Seed::Field(String::new()),
            },
        }
    }

    /// The selected scenario and the seed, for character creation.
    pub fn pick(&self) -> Option<ScenarioPick> {
        let info = self.scenarios.get(self.selected)?;
        Some(ScenarioPick {
            id: info.id.clone(),
            name: info.name.clone(),
            seed: self.seed(info),
            shows_seed: matches!(self.seed, Seed::Field(_)),
            character: info.character.clone(),
            start: info.start,
        })
    }

    /// The seed a run of scenario `info` gets: the fresh one, else the typed one (a number as-is,
    /// a word hashed: `warlock_core::rng::parse_seed`), else the scenario's.
    fn seed(&self, info: &ScenarioInfo) -> u64 {
        match &self.seed {
            Seed::Fresh(seed) => *seed,
            Seed::Field(text) => warlock_core::rng::parse_seed(text).unwrap_or(info.seed),
        }
    }

    /// Characters the seed field accepts. Letters double as hjkl navigation elsewhere, so with a
    /// seed field the arrow keys navigate and typed characters go to the seed.
    fn accepts(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '-' || ch == '_'
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        let count = self.scenarios.len();
        if let Seed::Field(seed) = &mut self.seed
            && let Some(ch) = press.text.filter(|ch| Self::accepts(*ch))
            && !press.repeat
        {
            if seed.chars().count() < MAX_SEED_LEN {
                seed.push(ch);
            }
            return ScreenAction::Stay;
        }
        match press.nav {
            Nav::Back => return ScreenAction::Close,
            Nav::Up if count > 0 => self.selected = (self.selected + count - 1) % count,
            Nav::Down if count > 0 => self.selected = (self.selected + 1) % count,
            Nav::Erase => {
                if let Seed::Field(seed) = &mut self.seed {
                    seed.pop();
                }
            }
            Nav::Enter if self.creates_characters && !self.scenarios.is_empty() => {
                return ScreenAction::Open(ScreenName::Race);
            }
            Nav::Enter => {
                if let Some(info) = self.scenarios.get(self.selected) {
                    return ScreenAction::Choose(MenuChoice::NewRun {
                        scenario: info.id.clone(),
                        seed: Some(self.seed(info)),
                        character: None,
                        name: None,
                    });
                }
            }
            _ => {}
        }
        ScreenAction::Stay
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        let field = match &self.seed {
            Seed::Field(text) => Some(text),
            Seed::Fresh(_) => None,
        };
        // The seed field and its hint take three rows, with the gap above them.
        let h = 11 + 3 * i32::from(field.is_some()) + self.scenarios.len() as i32;
        let panel = Panel::new(console, " New run ", 60, h);
        let x = panel.inner_x();
        let mut y = panel.y + 2;
        if self.scenarios.is_empty() {
            console.print(x, y, "No scenarios are installed.", TEXT);
        }
        for (i, info) in self.scenarios.iter().enumerate() {
            panel.row(console, y, i == self.selected, &info.name, "", 0);
            y += 1;
        }
        y += 1;
        if let Some(info) = self.scenarios.get(self.selected) {
            for line in wrap(&info.description, panel.inner_w() - 2) {
                console.print(x + 2, y, &line, TEXT_DIM);
                y += 1;
            }
        }
        if let (Some(info), Some(text)) = (self.scenarios.get(self.selected), field) {
            y += 1;
            let (seed, color) = if text.is_empty() {
                (format!("(default: {})", info.seed), TEXT_DIM)
            } else {
                (text.clone(), TEXT)
            };
            console.print(x + 2, y, "Seed: [", TEXT);
            let printed = console.print(x + 9, y, &seed, color);
            console.print(x + 9 + printed, y, "_]", TEXT);
            console.print(
                x + 2,
                y + 1,
                "Type a number or a word to set a seed.",
                TEXT_DIM,
            );
        }
        let keys = Hints::of(bindings);
        let next = if self.creates_characters {
            "next"
        } else {
            "start"
        };
        let footer = format!(
            "{} scenario   {} {next}   Esc back",
            keys.up_down(),
            keys.confirm
        );
        panel.footer(console, &footer);
    }
}

/// A seed for a release build's new run, which no one chooses (#256): the clock (in
/// nanoseconds; on the web in milliseconds, with `Math.random`) and, natively, the process's
/// random hash keys, mixed, kept to 32 bits (ten digits at most, so the status bar's `Seed`
/// segment stays short). Once drawn it is the run's like any other seed: the run is as
/// deterministic, and its replay records it.
pub fn fresh_seed() -> u64 {
    #[cfg(target_arch = "wasm32")]
    let noise = {
        use web_sys::js_sys::{Date, Math};
        let random = (Math::random() * (1u64 << 53) as f64) as u64;
        (Date::now() as u64) ^ (random << 11)
    };
    #[cfg(not(target_arch = "wasm32"))]
    let noise = {
        use std::hash::BuildHasher;
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);
        nanos ^ std::collections::hash_map::RandomState::new().hash_one(nanos)
    };
    mix(noise) >> 32
}

/// SplitMix64's finalizer: every bit of `z` moves every bit of the result.
fn mix(z: u64) -> u64 {
    let z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The developer builds' screen, with the seed field.
    fn screen() -> NewRunScreen {
        NewRunScreen::new(&Content::builtin().expect("content"), None)
    }

    fn drawn(screen: &NewRunScreen) -> String {
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console, &Bindings::default());
        (0..console.rows() as i32)
            .map(|y| {
                (0..console.cols() as i32)
                    .map(|x| console.get(x, y).map_or(' ', |c| c.ch))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn goes_on_to_the_character_with_the_chosen_scenario_and_typed_seed() {
        let mut s = screen();
        for d in ['4', '2'] {
            s.handle(&Press::typed(d));
        }
        s.handle(&Press::nav(Nav::Erase));
        s.handle(&Press::typed('7'));
        assert_eq!(
            s.handle(&Press::nav(Nav::Enter)),
            ScreenAction::Open(ScreenName::Race)
        );
        let pick = s.pick().expect("a scenario");
        assert_eq!(pick.id, "showcase");
        assert_eq!((pick.seed, pick.shows_seed), (47, true));
        assert_eq!(
            pick.character.map(|c| c.to_string()).as_deref(),
            Some("human,female,thief")
        );
        assert_eq!(s.handle(&Press::nav(Nav::Back)), ScreenAction::Close);
    }

    #[test]
    fn word_seeds_are_hashed_and_hjkl_type_letters() {
        let mut s = screen();
        for ch in "jolly".chars() {
            s.handle(&Press::typed(ch));
        }
        let seed = warlock_core::rng::parse_seed("jolly").expect("a seed");
        assert_eq!(s.pick().map(|p| p.seed), Some(seed));
    }

    #[test]
    fn empty_seed_means_the_scenario_default() {
        let s = screen();
        assert_eq!(s.pick().map(|p| p.seed), Some(1));
        let text = drawn(&s);
        assert!(text.contains("Seed: [(default: 1)_]"), "{text}");
        assert!(text.contains("Type a number or a word"), "{text}");
    }

    #[test]
    fn release_builds_have_a_fresh_seed_and_never_mention_it() {
        let content = Content::builtin().expect("content");
        let mut s = NewRunScreen::new(&content, Some(9_876_543_210));
        // Typed characters set no seed: letters are left to navigation.
        for ch in "42".chars() {
            s.handle(&Press::typed(ch));
        }
        s.handle(&Press::nav(Nav::Erase));
        let pick = s.pick().expect("a scenario");
        assert_eq!((pick.seed, pick.shows_seed), (9_876_543_210, false));
        let text = drawn(&s);
        assert!(text.contains("> Showcase"), "{text}");
        assert!(!text.to_lowercase().contains("seed"), "{text}");
    }

    #[test]
    fn fresh_seeds_differ_and_stay_short() {
        let seeds: std::collections::BTreeSet<u64> = (0..8).map(|_| fresh_seed()).collect();
        assert!(seeds.len() > 1, "{seeds:?}");
        assert!(seeds.iter().all(|&s| s <= u64::from(u32::MAX)), "{seeds:?}");
    }

    #[test]
    fn without_races_or_classes_enter_starts_the_scenario() {
        use warlock_core::content::ContentFiles;
        let mut files = ContentFiles::builtin();
        files.remove("data/races.toml");
        files.remove("data/names.toml");
        files.remove("data/classes/");
        for id in [
            "arena",
            "courtyard",
            "depths",
            "forest",
            "quarry",
            "showcase",
            "traps",
            "vast",
            "world",
        ] {
            files.remove(&format!("scenarios/{id}.toml"));
        }
        files.insert(
            "scenarios/plain.toml",
            "name = \"Plain\"\nstart_level = \"hub\"\n",
        );
        let content = Content::load(&files).expect("valid");
        let default = content.scenario("plain").expect("plain").seed;
        for (fresh, seed) in [(None, default), (Some(77), 77)] {
            let mut s = NewRunScreen::new(&content, fresh);
            assert_eq!(
                s.handle(&Press::nav(Nav::Enter)),
                ScreenAction::Choose(MenuChoice::NewRun {
                    scenario: "plain".into(),
                    seed: Some(seed),
                    character: None,
                    name: None,
                })
            );
        }
    }
}

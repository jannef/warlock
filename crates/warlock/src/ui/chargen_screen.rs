//! Character creation (spec §6 "New run", §8 "Character creation"): after the scenario and the
//! seed, New run asks for a race, reveals the birth sign (`sign_screen`), asks for a gender and a
//! class, one screen each ([`ChoiceScreen`]), then a name (`name_screen`), then shows the
//! character ([`SummaryScreen`]): the name, age, birthday,
//! and birth sign, the final scores (the dice already rolled), the starting trained skills with
//! their values, the starting kit in words, the alignment band they start in (spec §8
//! "Alignment"), and a way back to any step.
//!
//! The steps are screens on the stack, in order (New run, race, the sign, gender, class, name,
//! summary), so
//! Esc steps back one and the summary's "change" rows return to a step as it was left. What has
//! been chosen so far is read off the screens on the stack ([`Draft`]); choices made before going
//! back are remembered ([`Picks`]) and preselected when a step opens again, else the scenario's
//! own character, else the first option (the name: an empty field). Each choice step has a
//! Random row, rolled from the run seed and how often Random was pressed there, so it is
//! reproducible; the name's Random draws the race's names from the seed. No text says so: to the
//! player it is chance, and "other" never names the stat it raises (#256).
//!
//! Like every screen these are plain state machines, testable without a window.

use super::character_screen::{birth_parts, fit_joined, fit_parts};
use super::menu::{Hints, Nav, Panel, Press, wrap};
use super::screens::{MenuChoice, ScreenAction, ScreenName};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::Bindings;
use crate::render::console::Console;
use warlock_core::character::{CharacterSpec, Gender, NAMELESS, Start, check_name, random_name};
use warlock_core::content::{Content, pace_words};
use warlock_core::dice::Dice;
use warlock_core::kit;
use warlock_core::rng::Rng;
use warlock_core::stats::{PerStat, Stat};
use warlock_core::time::GameTime;

/// Width of the panels: wide enough for the summary's age, birthday, and sign on one line.
pub(super) const WIDTH: i32 = 72;
/// Width of the summary's panel (where the console has room): wider, for the starting kit.
const SUMMARY_WIDTH: i32 = 84;

/// A step of character creation, in order: three choices ([`ChoiceScreen`]), then the name
/// (`name_screen::NameScreen`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Race,
    Gender,
    Class,
    Name,
}

impl Step {
    /// The screen of this step.
    pub fn screen(self) -> ScreenName {
        match self {
            Step::Race => ScreenName::Race,
            Step::Gender => ScreenName::Gender,
            Step::Class => ScreenName::Class,
            Step::Name => ScreenName::Name,
        }
    }

    /// The screen after this step (after the race, the birth sign's reveal).
    fn next(self) -> ScreenName {
        match self {
            Step::Race => ScreenName::Sign,
            Step::Gender => ScreenName::Class,
            Step::Class => ScreenName::ChargenTalents,
            Step::Name => ScreenName::Summary,
        }
    }

    /// `race`, also the label of its random rolls.
    fn id(self) -> &'static str {
        match self {
            Step::Race => "race",
            Step::Gender => "gender",
            Step::Class => "class",
            Step::Name => "name",
        }
    }

    pub(super) fn title(self) -> &'static str {
        match self {
            Step::Race => " Choose your race (1 of 5) ",
            Step::Gender => " Choose your gender (2 of 5) ",
            Step::Class => " Choose your class (3 of 5) ",
            Step::Name => " Choose your name (5 of 5) ",
        }
    }
}

/// The scenario and seed chosen on the New run screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioPick {
    pub id: String,
    pub name: String,
    /// The seed the run will have: in developer builds the typed one, else the scenario's; in
    /// release builds one no one chose.
    pub seed: u64,
    /// The seed is shown (developer builds): the summary names it.
    pub shows_seed: bool,
    /// The scenario's own character, preselected.
    pub character: Option<CharacterSpec>,
    /// When the run starts: the character's birth counts back from it.
    pub start: GameTime,
}

/// Choices made in steps since left (going back), by step id; preselected when a step opens
/// again. Forgotten when the stack is cleared. The name is what was typed (possibly nothing).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Picks {
    pub race: Option<String>,
    pub gender: Option<String>,
    pub class: Option<String>,
    pub name: Option<String>,
    /// The talents marked on the Talents step, in order.
    pub talents: Option<Vec<String>>,
}

impl Picks {
    fn get(&self, step: Step) -> Option<&str> {
        match step {
            Step::Race => self.race.as_deref(),
            Step::Gender => self.gender.as_deref(),
            Step::Class => self.class.as_deref(),
            Step::Name => self.name.as_deref(),
        }
    }

    pub fn set(&mut self, step: Step, id: &str) {
        let slot = match step {
            Step::Race => &mut self.race,
            Step::Gender => &mut self.gender,
            Step::Class => &mut self.class,
            Step::Name => &mut self.name,
        };
        *slot = Some(id.to_string());
    }
}

/// What New run has chosen so far: read off the screens on the stack, bottom to top.
#[derive(Clone, Debug, Default)]
pub struct Draft {
    pub scenario: Option<ScenarioPick>,
    /// The steps on the stack, with their selections.
    pub chosen: Picks,
    /// Choices remembered from steps since left.
    pub remembered: Picks,
    /// The talents marked on the Talents step, if it is on the stack (else none: the run takes
    /// the Random picks).
    pub talents: Option<Vec<String>>,
}

impl Draft {
    /// The seed of the run being set up.
    pub(super) fn seed(&self) -> u64 {
        self.scenario.as_ref().map_or(0, |s| s.seed)
    }

    /// When the run being set up starts.
    pub(super) fn start(&self) -> GameTime {
        self.scenario.as_ref().map_or(GameTime::EPOCH, |s| s.start)
    }

    /// What a step preselects when it opens: the choice remembered for it, else the scenario's
    /// character's (no name: the Name step opens empty).
    fn preselect(&self, step: Step) -> Option<String> {
        self.remembered.get(step).map(str::to_string).or_else(|| {
            let character = self.scenario.as_ref()?.character.as_ref()?;
            match step {
                Step::Race => Some(character.race.clone()),
                Step::Gender => Some(character.gender.id().to_string()),
                Step::Class => Some(character.class.clone()),
                Step::Name => None,
            }
        })
    }

    /// The name typed before going back from the Name step, if any.
    pub(super) fn remembered_name(&self) -> Option<String> {
        self.remembered.name.clone()
    }

    /// The character's name as chosen so far: what is typed on the Name step (or was before
    /// going back), else, for an empty field, the first of the race's random names, else
    /// [`NAMELESS`].
    pub fn name(&self, content: &Content) -> String {
        let typed = (self.chosen.get(Step::Name))
            .or_else(|| self.remembered.get(Step::Name))
            .and_then(|text| check_name(text).ok());
        match typed {
            Some(name) => name.to_string(),
            None => (self.spec(content))
                .and_then(|spec| random_name(content, self.seed(), &spec, 0))
                .unwrap_or_else(|| NAMELESS.into()),
        }
    }

    /// The character as chosen so far; a step not on the stack takes its preselection, else the
    /// first option. `None` when the content has no races or classes.
    pub fn spec(&self, content: &Content) -> Option<CharacterSpec> {
        let pick = |step: Step, first: Option<&String>| {
            (self.chosen.get(step).map(str::to_string))
                .or_else(|| self.preselect(step))
                .or_else(|| first.cloned())
        };
        let race = pick(Step::Race, content.races().list.first().map(|r| &r.id))?;
        let class = pick(Step::Class, content.classes().first().map(|c| &c.id))?;
        let gender = (self.chosen.get(Step::Gender).map(str::to_string))
            .or_else(|| self.preselect(Step::Gender))
            .and_then(|id| Gender::from_id(&id))
            .unwrap_or(Gender::Male);
        let talents = self.talents.clone().unwrap_or_default();
        Some(CharacterSpec::new(race, gender, class).with_talents(talents))
    }
}

/// One option of a step.
#[derive(Clone, Debug)]
struct Choice {
    id: String,
    name: String,
    description: String,
    /// Lines under the description: the stat adjustments, the race's eyes, how fast levels come.
    details: Vec<String>,
}

/// The stat adjustments that are not zero, in stat order, as written in data: `St 1d3+1  Dx −1`.
pub(super) fn adjustments(by: &PerStat<Dice>) -> String {
    let parts: Vec<String> = (by.iter())
        .filter(|(_, v)| !v.is_zero())
        .map(|(stat, v)| format!("{} {}", stat.abbrev(), dice_text(v)))
        .collect();
    if parts.is_empty() {
        "No stat changes".into()
    } else {
        parts.join("  ")
    }
}

/// An adjustment as written: `+2`, `1d3+1`, or with a minus sign, not a hyphen: `−1`, `−1d2`.
fn dice_text(dice: Dice) -> String {
    dice.to_string().replace('-', "\u{2212}")
}

/// A step: its options and a Random row.
#[derive(Debug)]
pub struct ChoiceScreen {
    step: Step,
    choices: Vec<Choice>,
    /// The selected row; `choices.len()` is Random.
    selected: usize,
    /// The run seed, for Random.
    seed: u64,
    /// How often Random was pressed here.
    rolls: u32,
}

impl ChoiceScreen {
    pub fn new(step: Step, content: &Content, draft: &Draft) -> Self {
        let seed = draft.seed();
        let choices: Vec<Choice> = match step {
            Step::Race => (content.races().list.iter())
                .map(|r| {
                    let eyes = content
                        .vision(&r.vision)
                        .map_or(r.vision.as_str(), |v| &v.name);
                    let mut details = vec![adjustments(&r.stats), format!("Vision: {eyes}")];
                    details.extend(pace_words(r.experience).map(String::from));
                    Choice {
                        id: r.id.clone(),
                        name: r.name.clone(),
                        description: r.description.clone(),
                        details,
                    }
                })
                .collect(),
            Step::Gender => Gender::ALL
                .into_iter()
                .map(|g| Choice {
                    id: g.id().to_string(),
                    name: g.name().to_string(),
                    description: gender_description(g).to_string(),
                    details: Vec::new(),
                })
                .collect(),
            Step::Class => (content.classes().iter())
                .map(|c| Choice {
                    id: c.id.clone(),
                    name: c.name.clone(),
                    description: c.description.clone(),
                    details: [adjustments(&c.stats)]
                        .into_iter()
                        .chain(pace_words(c.experience).map(String::from))
                        .collect(),
                })
                .collect(),
            // The name is typed, on a screen of its own.
            Step::Name => Vec::new(),
        };
        let selected = draft
            .preselect(step)
            .and_then(|id| choices.iter().position(|c| c.id == id))
            .unwrap_or(0);
        Self {
            step,
            choices,
            selected,
            seed,
            rolls: 0,
        }
    }

    pub fn step(&self) -> Step {
        self.step
    }

    /// The id of the selected option; `None` on the Random row.
    pub fn chosen(&self) -> Option<&str> {
        self.choices.get(self.selected).map(|c| c.id.as_str())
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        let rows = self.choices.len() + 1;
        match press.nav {
            Nav::Back => return ScreenAction::Close,
            Nav::Up => self.selected = (self.selected + rows - 1) % rows,
            Nav::Down => self.selected = (self.selected + 1) % rows,
            Nav::Enter if self.choices.is_empty() => {}
            Nav::Enter => {
                if self.selected == self.choices.len() {
                    self.selected = self.roll();
                }
                return ScreenAction::Open(self.step.next());
            }
            _ => {}
        }
        ScreenAction::Stay
    }

    /// A random option, from the run seed and the number of rolls so far.
    fn roll(&mut self) -> usize {
        let label = format!("new-run/{}/{}", self.step.id(), self.rolls);
        self.rolls += 1;
        Rng::stream(self.seed, &label).index(self.choices.len())
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Rows: margin, the list (options and Random), a gap, the description (up to three
        // lines), the details (up to three), a gap, the footer, the frame.
        let rows = self.choices.len() as i32 + 1;
        let panel = Panel::new(console, self.step.title(), WIDTH, rows + 12);
        let x = panel.inner_x();
        let visible = (panel.h - 12).clamp(1, rows);
        let first = (self.selected as i32 - visible + 1).clamp(0, rows - visible);
        for row in first..first + visible {
            let y = panel.y + 2 + row - first;
            let i = row as usize;
            let name = self.choices.get(i).map_or("Random", |c| c.name.as_str());
            panel.row(console, y, i == self.selected, name, "", 0);
        }
        if first > 0 {
            console.print(panel.x + panel.w - 3, panel.y + 1, "↑", TEXT_DIM);
        }
        if first + visible < rows {
            console.print(panel.x + panel.w - 3, panel.y + 2 + visible, "↓", TEXT_DIM);
        }
        let footer_y = panel.y + panel.h - 2;
        let (description, details) = match self.choices.get(self.selected) {
            Some(c) => (c.description.clone(), c.details.clone()),
            None => (
                format!("Let chance pick your {}.", self.step.id()),
                Vec::new(),
            ),
        };
        let lines = wrap(&description, panel.inner_w() - 2).into_iter().take(3);
        let text = (lines.map(|l| (l, TEXT_DIM))).chain(details.into_iter().map(|l| (l, TEXT)));
        for (y, (line, color)) in (panel.y + 3 + visible..).zip(text) {
            if y < footer_y - 1 {
                console.print(x + 2, y, &line, color);
            }
        }
        let keys = Hints::of(bindings);
        let footer = format!(
            "{} choose   {} next   Esc back",
            keys.up_down(),
            keys.confirm
        );
        panel.footer(console, &footer);
    }
}

/// What a gender does. `other` never names the stat it raises (#256): fate picks it.
fn gender_description(gender: Gender) -> &'static str {
    match gender {
        Gender::Male => "Strength +1.",
        Gender::Female => "Dexterity +1.",
        Gender::Other => "+1 to one stat, chosen by fate.",
    }
}

/// What a summary row does: start the run, go back to a step, or read the sign's reveal again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    Start,
    Step(Step),
    Sign,
    Talents,
}

/// The summary's rows: start, go back to a step, or to the birth sign's reveal.
const SUMMARY_ROWS: &[(&str, Row)] = &[
    ("Start the run", Row::Start),
    ("Name", Row::Step(Step::Name)),
    ("Race", Row::Step(Step::Race)),
    ("Sign", Row::Sign),
    ("Gender", Row::Step(Step::Gender)),
    ("Class", Row::Step(Step::Class)),
    ("Talents", Row::Talents),
];

/// Where the starting skills' columns begin, beside the scores (from the panel's inner edge).
const SKILLS_X: i32 = 26;
/// The room between two columns of skills.
const SKILL_GAP: i32 = 3;
/// Where the kit's lines begin, after its label (from the panel's inner edge), as the eyes' and
/// the scenario's values do.
const KIT_X: i32 = 12;

/// What the summary says a character starts with (spec §8 "Character creation"): every stack
/// of the starting pack as named, their kit (a run started here always has it) and the
/// scenario's own pack on top; "nothing" for nothing.
fn kit_words(content: &Content, draft: &Draft, start: &Result<Start, String>) -> Vec<String> {
    let Ok(start) = start else {
        return Vec::new();
    };
    let scenario = (draft.scenario.as_ref()).and_then(|s| content.scenario(&s.id));
    let pack = match scenario {
        Some(scenario) => kit::starting_pack(content, scenario, Some(&start.kit)),
        None => start.kit.clone(),
    };
    match pack.names(content) {
        names if names.is_empty() => vec!["nothing".into()],
        names => names,
    }
}

/// The summary's Talents line: the creation picks by name (`Healthy, Weapon focus (long
/// blades)`), or, with none marked, that chance picks them.
fn talent_names(content: &Content, spec: Option<&CharacterSpec>) -> String {
    let book = content.talents();
    let names: Vec<String> = (spec.iter().flat_map(|s| &s.talents))
        .filter_map(|pick| {
            let (id, choice) = match pick.split_once(':') {
                Some((id, choice)) => (id, Some(choice)),
                None => (pick.as_str(), None),
            };
            book.id(id).map(|t| book.get(t).name_for(content, choice))
        })
        .collect();
    match names.is_empty() {
        true => "chosen by chance".into(),
        false => names.join(", "),
    }
}

/// The character before the run starts: name, race, gender, and class (each a way back to its
/// step), age, birthday, and birth sign, the final scores, the starting trained skills, the
/// starting kit, the alignment band, the eyes, and the scenario (and in developer builds the
/// seed).
#[derive(Debug)]
pub struct SummaryScreen {
    scenario: Option<ScenarioPick>,
    spec: Option<CharacterSpec>,
    /// The name the character starts with.
    name: String,
    /// What the character starts with, or why it cannot start.
    start: Result<Start, String>,
    /// Race, gender, class, and name as shown, in step order.
    names: [String; 4],
    /// The birth sign as shown, with a lucky day's title: `Metal Manticore, the Iron Scourge`;
    /// empty without one.
    sign: String,
    /// Age, birthday, and birth sign, as shown.
    birth: Vec<String>,
    /// The starting trained skills, by name, with their values, in skill order.
    skills: Vec<(String, u16)>,
    /// What the character starts with, each stack as named (`a dagger (wielded)`), the
    /// scenario's own pack included.
    kit: Vec<String>,
    eyes: String,
    /// The band the character starts in (spec §8 "Alignment"); empty without alignment rules.
    alignment: String,
    /// The talents picked, by name (`Healthy, Lean`); "chosen by chance" for none marked.
    talents: String,
    /// The line under the kit that a background story will be told (spec §8 "Background
    /// story"), with content that tells them; empty without.
    fate: &'static str,
    selected: usize,
}

/// What the summary says of the background story to come: it changes what the run starts with,
/// and nothing of it is known yet.
pub const FATE: &str = "Fate will have its say when you set out.";

impl SummaryScreen {
    pub fn new(content: &Content, draft: &Draft) -> Self {
        let spec = draft.spec(content);
        let start = match &spec {
            Some(spec) => spec.start(content, draft.seed(), draft.start()),
            None => Err("There are no races or classes to choose from.".into()),
        };
        let name = draft.name(content);
        // The gender by name alone: "other" never names the stat it raised here (#256).
        let names = match &spec {
            Some(spec) => [
                spec.race(content)
                    .map_or(spec.race.clone(), |r| r.name.clone()),
                spec.gender.name().to_string(),
                spec.class(content)
                    .map_or(spec.class.clone(), |c| c.name.clone()),
                name.clone(),
            ],
            None => Default::default(),
        };
        let birth = (start.as_ref()).map_or(Vec::new(), |s| {
            birth_parts(&s.character, content, draft.start())
        });
        let sign = (start.as_ref().ok())
            .and_then(|s| s.character.sign_title(content))
            .unwrap_or_default();
        let eyes = start.as_ref().map_or(String::new(), |s| {
            content
                .vision(&s.vision)
                .map_or(s.vision.clone(), |v| v.name.clone())
        });
        let alignment = (start.as_ref().ok())
            .zip(content.alignment())
            .map_or(String::new(), |(s, rules)| {
                rules.band(s.alignment).name().to_string()
            });
        let skills = start.as_ref().map_or(Vec::new(), |s| {
            (s.skills.iter())
                .map(|(id, t)| (content.skill(id).name.clone(), t.value))
                .collect()
        });
        let kit = kit_words(content, draft, &start);
        let talents = talent_names(content, spec.as_ref());
        Self {
            scenario: draft.scenario.clone(),
            spec,
            name,
            start,
            names,
            sign,
            birth,
            skills,
            kit,
            eyes,
            alignment,
            talents,
            fate: if content.background().is_some() {
                FATE
            } else {
                ""
            },
            selected: 0,
        }
    }

    /// The starting kit's lines in a panel `inner` cells wide: the stacks with a comma after each
    /// but the last, wrapped beside the label.
    fn kit_lines(&self, inner: i32) -> Vec<String> {
        let n = self.kit.len();
        let parts: Vec<String> = (self.kit.iter().enumerate())
            .map(|(i, name)| {
                if i + 1 < n {
                    format!("{name},")
                } else {
                    name.clone()
                }
            })
            .collect();
        fit_joined(&parts, " ", inner - KIT_X)
    }

    /// The width of a column of the starting skills: the longest name, a space, and a value.
    fn skill_width(&self) -> i32 {
        let longest = self.skills.iter().map(|(name, _)| name.chars().count());
        longest.max().unwrap_or(0) as i32 + 4
    }

    /// How the starting skills are laid out in a panel `inner` cells wide: in two columns beside
    /// the scores (more than there are scores), or one, or (too narrow for that) as lines of
    /// `Name value` parts below them. Returns the columns' rows (0 when below) and the lines
    /// below.
    fn skill_layout(&self, inner: i32) -> (usize, Vec<String>) {
        let n = self.skills.len();
        let w = self.skill_width();
        if n == 0 {
            return (0, Vec::new());
        }
        if inner >= SKILLS_X + 2 * w + SKILL_GAP && n > Stat::COUNT {
            (n.div_ceil(2), Vec::new())
        } else if inner >= SKILLS_X + w {
            (n, Vec::new())
        } else {
            let parts: Vec<String> = (self.skills.iter())
                .map(|(name, value)| format!("{name} {value}"))
                .collect();
            (0, fit_parts(&parts, inner - 2))
        }
    }

    pub fn handle(&mut self, press: &Press) -> ScreenAction {
        let rows = SUMMARY_ROWS.len();
        match press.nav {
            Nav::Back => return ScreenAction::Close,
            Nav::Up => self.selected = (self.selected + rows - 1) % rows,
            Nav::Down => self.selected = (self.selected + 1) % rows,
            Nav::Enter => match SUMMARY_ROWS[self.selected].1 {
                Row::Step(step) => return ScreenAction::Return(step.screen()),
                Row::Sign => return ScreenAction::Return(ScreenName::Sign),
                Row::Talents => return ScreenAction::Return(ScreenName::ChargenTalents),
                Row::Start => {
                    if let (Some(scenario), Some(spec), Ok(_)) =
                        (&self.scenario, &self.spec, &self.start)
                    {
                        return ScreenAction::Choose(MenuChoice::NewRun {
                            scenario: scenario.id.clone(),
                            seed: Some(scenario.seed),
                            character: Some(spec.clone()),
                            name: Some(self.name.clone()),
                        });
                    }
                }
            },
            _ => {}
        }
        ScreenAction::Stay
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Rows: margin, start and the four steps, a gap, the birth and a gap (if any), the
        // stats with the skills beside them (or the skills below them, and a gap), a gap, the
        // kit, alignment, eyes, and scenario, a gap, the footer, the frame. Too tall for the
        // console, the gaps before the kit, after the birth, and after the steps go, in that
        // order.
        let width = SUMMARY_WIDTH.min(console.cols() as i32);
        let birth = fit_parts(&self.birth, width - 6);
        let inner = width - 4;
        let (skill_rows, below) = self.skill_layout(inner);
        let block = (Stat::COUNT.max(skill_rows)) as i32;
        let below_rows = match below.len() {
            0 => 0,
            n => n as i32 + 1,
        };
        let kit = self.kit_lines(inner);
        let birth_rows = birth.len() as i32 + i32::from(!birth.is_empty());
        let alignment_rows = i32::from(!self.alignment.is_empty());
        let full = SUMMARY_ROWS.len() as i32
            + block
            + 9
            + birth_rows
            + below_rows
            + alignment_rows
            + kit.len() as i32
            + i32::from(!self.fate.is_empty());
        let squeeze = (full - console.rows() as i32).clamp(0, 3);
        let gap = |n: i32| i32::from(squeeze < n);
        let h = full - squeeze;
        let panel = Panel::new(console, " Your character ", width, h);
        let x = panel.inner_x();
        let fits = |y: i32| y < panel.y + panel.h - 2;
        let mut y = panel.y + 2;
        for (i, (label, step)) in SUMMARY_ROWS.iter().enumerate() {
            if fits(y) {
                let value = match step {
                    Row::Start => "",
                    Row::Step(s) => self.names[*s as usize].as_str(),
                    Row::Sign => self.sign.as_str(),
                    Row::Talents => self.talents.as_str(),
                };
                // The value starts 12 cells in and is padded by two spaces on each side; a
                // longer one is cut to stay inside the frame.
                let value = clipped(value, inner - 14);
                panel.row(console, y, i == self.selected, label, &value, 6);
            }
            y += 1;
        }
        y += gap(3);
        for line in &birth {
            if fits(y) {
                console.print(x + 2, y, line, TEXT);
            }
            y += 1;
        }
        y += i32::from(!birth.is_empty()) * gap(2);
        match &self.start {
            Ok(start) => {
                for (i, (stat, score)) in start.stats.scores().iter().enumerate() {
                    let row = y + i as i32;
                    if fits(row) {
                        console.print(x + 2, row, stat.abbrev(), TEXT_DIM);
                        console.print(x + 6, row, stat.name(), TEXT);
                        console.print(x + 20, row, &format!("{score:>2}"), TEXT);
                    }
                }
                // The skills beside the scores, down the first column, then the second.
                let beside = if skill_rows > 0 { self.skills.len() } else { 0 };
                let w = self.skill_width();
                for (i, (name, value)) in self.skills.iter().enumerate().take(beside) {
                    let row = y + (i % skill_rows) as i32;
                    let sx = x + SKILLS_X + (i / skill_rows) as i32 * (w + SKILL_GAP);
                    if fits(row) {
                        console.print(sx, row, name, TEXT);
                        console.print(sx + w - 3, row, &format!("{value:>3}"), TEXT);
                    }
                }
                y += block;
                if !below.is_empty() {
                    y += 1;
                    for line in &below {
                        if fits(y) {
                            console.print(x + 2, y, line, TEXT);
                        }
                        y += 1;
                    }
                }
            }
            Err(why) => {
                if fits(y) {
                    console.print(x + 2, y, why, ACCENT);
                }
                y += block;
            }
        }
        y += gap(1);
        for (i, line) in kit.iter().enumerate() {
            if fits(y) {
                if i == 0 {
                    console.print(x + 2, y, "Kit", TEXT_DIM);
                }
                console.print(x + KIT_X, y, line, TEXT);
            }
            y += 1;
        }
        if !self.fate.is_empty() {
            if fits(y) {
                console.print(x + KIT_X, y, self.fate, TEXT_DIM);
            }
            y += 1;
        }
        // The seed only in developer builds (#256).
        let scenario = self.scenario.as_ref().map_or(String::new(), |s| {
            if s.shows_seed {
                format!("{}, seed {}", s.name, s.seed)
            } else {
                s.name.clone()
            }
        });
        let rows = [
            ("Alignment", &self.alignment),
            ("Vision", &self.eyes),
            ("Scenario", &scenario),
        ];
        for (label, value) in rows {
            if label == "Alignment" && value.is_empty() {
                continue;
            }
            if fits(y) {
                console.print(x + 2, y, label, TEXT_DIM);
                console.print(x + 12, y, value, TEXT);
            }
            y += 1;
        }
        let keys = Hints::of(bindings);
        let (up_down, confirm) = (keys.up_down(), keys.confirm);
        let footer = format!("{up_down} choose   {confirm} start or change   Esc back");
        panel.footer(console, &footer);
    }
}

/// `text` cut to `width` cells, an ellipsis marking the cut: `Weapon focus (daggers and k…`.
fn clipped(text: &str, width: i32) -> String {
    let width = width.max(1) as usize;
    if text.chars().count() <= width {
        return text.to_string();
    }
    let head: String = text.chars().take(width - 1).collect();
    format!("{head}…")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::LazyLock;

    static CONTENT: LazyLock<Content> =
        LazyLock::new(|| Content::builtin().expect("builtin content"));

    fn draft(seed: u64) -> Draft {
        let showcase = CONTENT.scenario("showcase").expect("showcase");
        Draft {
            scenario: Some(ScenarioPick {
                id: "showcase".into(),
                name: showcase.name.clone(),
                seed,
                shows_seed: true,
                character: showcase.character.clone(),
                start: showcase.start,
            }),
            ..Draft::default()
        }
    }

    fn press(screen: &mut ChoiceScreen, nav: Nav) -> ScreenAction {
        screen.handle(&Press::nav(nav))
    }

    fn text(console: &Console) -> String {
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
    fn steps_list_every_option_and_preselect_the_scenarios_character() {
        let d = draft(1);
        let race = ChoiceScreen::new(Step::Race, &CONTENT, &d);
        assert_eq!(race.choices.len(), 13);
        assert_eq!(race.chosen(), Some("human"), "the showcase's human");
        let gender = ChoiceScreen::new(Step::Gender, &CONTENT, &d);
        assert_eq!(gender.chosen(), Some("female"));
        let class = ChoiceScreen::new(Step::Class, &CONTENT, &d);
        assert_eq!(class.choices.len(), 15);
        assert_eq!(class.chosen(), Some("thief"));
        // A remembered choice wins; without either, the first option.
        let mut remembered = d.clone();
        remembered.remembered.set(Step::Race, "gnome");
        let race = ChoiceScreen::new(Step::Race, &CONTENT, &remembered);
        assert_eq!(race.chosen(), Some("gnome"));
        let race = ChoiceScreen::new(Step::Race, &CONTENT, &Draft::default());
        assert_eq!(race.chosen(), Some("elf"));
    }

    #[test]
    fn enter_moves_on_esc_goes_back_and_the_list_wraps() {
        let mut race = ChoiceScreen::new(Step::Race, &CONTENT, &draft(1));
        assert_eq!(press(&mut race, Nav::Down), ScreenAction::Stay);
        assert_eq!(race.chosen(), Some("dwarf"));
        assert_eq!(
            press(&mut race, Nav::Enter),
            ScreenAction::Open(ScreenName::Sign),
            "the stars are revealed after the race"
        );
        assert_eq!(press(&mut race, Nav::Back), ScreenAction::Close);
        // Up from the first row is Random, the last.
        let mut gender = ChoiceScreen::new(Step::Gender, &CONTENT, &Draft::default());
        assert_eq!(gender.chosen(), Some("male"));
        press(&mut gender, Nav::Up);
        assert_eq!(gender.chosen(), None, "on Random");
        press(&mut gender, Nav::Down);
        assert_eq!(gender.chosen(), Some("male"));
        let mut class = ChoiceScreen::new(Step::Class, &CONTENT, &draft(1));
        assert_eq!(
            press(&mut class, Nav::Enter),
            ScreenAction::Open(ScreenName::ChargenTalents)
        );
    }

    #[test]
    fn random_picks_from_the_seed_and_rolls_again() {
        let pick = |seed: u64| {
            let mut s = ChoiceScreen::new(Step::Class, &CONTENT, &draft(seed));
            s.selected = s.choices.len();
            let mut picks = Vec::new();
            for _ in 0..4 {
                s.selected = s.choices.len();
                assert_eq!(
                    press(&mut s, Nav::Enter),
                    ScreenAction::Open(ScreenName::ChargenTalents)
                );
                picks.push(s.chosen().expect("a class").to_string());
            }
            picks
        };
        // The same seed rolls the same classes; rolling again picks anew.
        assert_eq!(pick(5), pick(5));
        let many: std::collections::BTreeSet<String> = (0..20).flat_map(pick).collect();
        assert!(many.len() > 5, "{many:?}");
    }

    #[test]
    fn other_never_names_the_stat_it_raises_nor_random_the_seed() {
        for seed in [1, 2, 3] {
            let mut gender = ChoiceScreen::new(Step::Gender, &CONTENT, &draft(seed));
            let other = &gender.choices[2];
            assert_eq!(other.id, "other");
            assert_eq!(other.description, "+1 to one stat, chosen by fate.");
            // Other selected, then Random: neither names a stat or the seed.
            for _ in 0..2 {
                gender.selected = (gender.selected + 1) % (gender.choices.len() + 1);
                let mut console = Console::default();
                console.resize(80, 32);
                gender.draw(&mut console, &Bindings::default());
                let text = text(&console);
                assert!(!text.to_lowercase().contains("seed"), "{text}");
                for stat in Stat::ALL {
                    assert!(!text.contains(stat.name()), "{stat:?} in\n{text}");
                }
            }
        }
        let mut race = ChoiceScreen::new(Step::Race, &CONTENT, &draft(1));
        race.selected = race.choices.len();
        let mut console = Console::default();
        console.resize(80, 32);
        race.draw(&mut console, &Bindings::default());
        assert!(text(&console).contains("Let chance pick your race."));
    }

    #[test]
    fn steps_show_the_selected_options_description_and_adjustments() {
        let mut race = ChoiceScreen::new(Step::Race, &CONTENT, &draft(1));
        press(&mut race, Nav::Down);
        let mut console = Console::default();
        console.resize(80, 32);
        race.draw(&mut console, &Bindings::default());
        let text = text(&console);
        assert!(text.contains("Choose your race (1 of 5)"), "{text}");
        assert!(text.contains("> Dwarf"), "{text}");
        assert!(text.contains("Stout, stubborn folk"), "{text}");
        assert!(
            text.contains("St +1  Dx \u{2212}1  To 1d2+2  Wi +1  Pr \u{2212}2"),
            "{text}"
        );
        assert!(text.contains("Vision: Darkvision"), "{text}");
        assert!(text.contains("Levels come slowly."), "{text}");
        assert!(text.contains("Random"), "{text}");
        assert!(text.contains("Enter next"), "{text}");
        // Nothing hidden: no potentials (the dwarf's toughness 19), no modifiers.
        assert!(!text.contains("19") && !text.contains("otential"), "{text}");
    }

    #[test]
    fn race_and_class_steps_say_how_fast_levels_come() {
        // In words only, and only where the experience factor is far from 100 %.
        let words = |step: Step, id: &str| {
            let mut draft = draft(1);
            draft.remembered.set(step, id);
            let screen = ChoiceScreen::new(step, &CONTENT, &draft);
            let mut console = Console::default();
            console.resize(80, 40);
            screen.draw(&mut console, &Bindings::default());
            let text = text(&console);
            assert!(!text.contains('%'), "{text}");
            ["Levels come slowly.", "Levels come quickly."]
                .into_iter()
                .find(|w| text.contains(w))
        };
        assert_eq!(words(Step::Race, "elf"), Some("Levels come slowly."));
        assert_eq!(words(Step::Race, "human"), None);
        assert_eq!(words(Step::Race, "half_orc"), Some("Levels come quickly."));
        assert_eq!(words(Step::Class, "wizard"), Some("Levels come slowly."));
        assert_eq!(words(Step::Class, "warrior"), None);
        assert_eq!(words(Step::Class, "commoner"), Some("Levels come quickly."));
    }

    #[test]
    fn long_lists_scroll_in_small_consoles() {
        let mut class = ChoiceScreen::new(Step::Class, &CONTENT, &Draft::default());
        for (cols, rows) in [(80, 32), (64, 20), (40, 14), (5, 3), (0, 0)] {
            for _ in 0..16 {
                press(&mut class, Nav::Down);
                let mut console = Console::default();
                console.resize(cols, rows);
                class.draw(&mut console, &Bindings::default());
                if rows >= 20 {
                    // The selected row is always in view.
                    let name = class.chosen().map_or("Random".to_string(), |id| {
                        CONTENT.class(id).expect("a class").name.clone()
                    });
                    assert!(text(&console).contains(&format!("> {name}")), "{name}");
                }
            }
        }
    }

    fn summary(spec: &str, seed: u64) -> SummaryScreen {
        let spec: CharacterSpec = spec.parse().expect("a character");
        let mut d = draft(seed);
        d.chosen.set(Step::Race, &spec.race);
        d.chosen.set(Step::Gender, spec.gender.id());
        d.chosen.set(Step::Class, &spec.class);
        SummaryScreen::new(&CONTENT, &d)
    }

    #[test]
    fn the_summary_lists_the_starting_kit() {
        // A run started here has the kit: the world's ranger's is listed as it is named and
        // worn.
        let world = CONTENT.scenario("world").expect("world");
        let mut d = draft(1);
        d.scenario = Some(ScenarioPick {
            id: "world".into(),
            name: world.name.clone(),
            shows_seed: true,
            seed: 1,
            character: world.character.clone(),
            start: world.start,
        });
        let text = drawn(&SummaryScreen::new(&CONTENT, &d));
        for expected in [
            "Kit       a short bow (readied), 24 arrows (quivered),",
            "a short sword (wielded)",
            "a pair of leather boots (being worn)",
            "3 strips of dried meat",
            "2 loaves of bread",
        ] {
            assert!(text.contains(expected), "{expected:?} in\n{text}");
        }
        // So does a run of the showcase, whose own pack is empty.
        let showcase = drawn(&summary("human,female,thief", 1));
        assert!(
            showcase.contains("Kit       a dagger (wielded)"),
            "{showcase}"
        );
        // A short console squeezes out the gaps before the kit's lines go.
        let mut console = Console::default();
        console.resize(90, 27);
        let mut d = d.clone();
        d.chosen.set(Step::Race, "halfling");
        d.chosen.set(Step::Class, "merchant");
        SummaryScreen::new(&CONTENT, &d).draw(&mut console, &Bindings::default());
        let short = super::tests::text(&console);
        for expected in [
            "Scenario  World, seed 1",
            "scroll of identify",
            "2 wool cloaks",
        ] {
            assert!(short.contains(expected), "{expected:?} in\n{short}");
        }
    }

    fn drawn(screen: &SummaryScreen) -> String {
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console, &Bindings::default());
        text(&console)
    }

    #[test]
    fn the_summary_shows_the_character_and_its_final_scores() {
        let text = drawn(&summary("dwarf,female,warrior", 1));
        let spec: CharacterSpec = "dwarf,female,warrior".parse().expect("a character");
        let name = random_name(&CONTENT, 1, &spec, 0).expect("a name");
        assert!(text.contains(&format!("Name      {name}")), "{text}");
        for expected in [
            "Your character",
            "> Start the run",
            "Race      Dwarf",
            "Gender    Female",
            "Class     Warrior",
            "Age 101 · born 23 Rainmoon 1343 · year of the Earth Kraken",
            "St  Strength      15",
            "Dx  Dexterity     10",
            "To  Toughness     16",
            "In  Intelligence   9",
            "Wi  Willpower     12",
            "Pr  Presence       8",
            "Alignment Neutral",
            "Vision    Darkvision",
            "Scenario  Showcase, seed 1",
            // The starting skills beside the scores: the dwarf's and the warrior's.
            "St  Strength      15    Athletics   17",
            "To  Toughness     16    Initiative  12   Swim         9",
        ] {
            assert!(text.contains(expected), "{expected:?} in\n{text}");
        }
        // A paladin starts Good: the band, never the score.
        let paladin = drawn(&summary("human,male,paladin", 3));
        assert!(paladin.contains("Alignment Good"), "{paladin}");
        assert!(!paladin.contains("550"), "{paladin}");
        // Born in a Basilisk year the stars nudge them down (#272), still Good (#317).
        let basilisk = drawn(&summary("human,male,paladin", 1));
        assert!(basilisk.contains("Basilisk"), "{basilisk}");
        assert!(basilisk.contains("Alignment Good"), "{basilisk}");
        assert!(!basilisk.contains("450"), "{basilisk}");
        for skill in ["Balance", "Intimidate", "Metalcraft", "Search"] {
            assert!(text.contains(skill), "{skill:?} in\n{text}");
        }
        // More than six skills take two columns; a narrow panel lists them below the scores.
        let thief = drawn(&summary("halfling,male,thief", 2));
        for expected in ["Appraise", "Tumble", "Listen"] {
            assert!(thief.contains(expected), "{expected:?} in\n{thief}");
        }
        let mut console = Console::default();
        console.resize(44, 40);
        summary("halfling,male,thief", 2).draw(&mut console, &Bindings::default());
        let narrow = super::tests::text(&console);
        assert!(narrow.contains("Appraise "), "{narrow}");
        assert!(narrow.contains(" · "), "{narrow}");
        // Final scores only: no dice, no breakdown, never a potential or a modifier.
        for hidden in ["1d", "d2", "d3", "otential", "odifier", "+"] {
            assert!(!text.contains(hidden), "{hidden:?} in\n{text}");
        }
        // The scores shown are the ones the run starts with.
        let spec: CharacterSpec = "dwarf,female,warrior".parse().expect("a character");
        let content = std::sync::Arc::new(Content::builtin().expect("content"));
        let world = warlock_core::World::with_character(content, "showcase", Some(1), Some(&spec))
            .expect("world");
        let view = world.perception();
        for (stat, score) in view.own_scores().iter() {
            let row = format!("{}  {:<14}{score:>2}", stat.abbrev(), stat.name());
            assert!(text.contains(&row), "{row:?} in\n{text}");
        }
        // In a narrow console the sign goes on the next line.
        let mut console = Console::default();
        console.resize(60, 30);
        summary("dwarf,female,warrior", 1).draw(&mut console, &Bindings::default());
        let narrow = super::tests::text(&console);
        assert!(
            narrow.contains("Age 101 · born 23 Rainmoon 1343  "),
            "{narrow}"
        );
        assert!(narrow.contains("year of the Earth Kraken"), "{narrow}");
        // Other never names the stat it raised.
        let other = drawn(&summary("elf,other,wizard", 3));
        assert!(other.contains("Gender    Other  "), "{other}");
        assert!(!other.contains(" +1"), "{other}");
    }

    #[test]
    fn the_summary_names_the_seed_only_in_developer_builds() {
        let spec: CharacterSpec = "dwarf,female,warrior".parse().expect("a character");
        let mut d = draft(1);
        d.chosen.set(Step::Race, &spec.race);
        let dev = drawn(&SummaryScreen::new(&CONTENT, &d));
        assert!(dev.contains("Scenario  Showcase, seed 1"), "{dev}");
        if let Some(pick) = d.scenario.as_mut() {
            pick.shows_seed = false;
        }
        let mut release = SummaryScreen::new(&CONTENT, &d);
        let text = drawn(&release);
        assert!(text.contains("Scenario  Showcase  "), "{text}");
        assert!(!text.to_lowercase().contains("seed "), "{text}");
        // The run still starts with the seed.
        let ScreenAction::Choose(MenuChoice::NewRun { seed, .. }) =
            release.handle(&Press::nav(Nav::Enter))
        else {
            panic!("a new run");
        };
        assert_eq!(seed, Some(1));
    }

    #[test]
    fn the_summary_starts_the_run_or_returns_to_a_step() {
        let spec: CharacterSpec = "gnome,other,merchant".parse().expect("a character");
        let mut s = summary("gnome,other,merchant", 7);
        assert_eq!(
            s.handle(&Press::nav(Nav::Enter)),
            ScreenAction::Choose(MenuChoice::NewRun {
                scenario: "showcase".into(),
                seed: Some(7),
                character: Some(spec.clone()),
                name: random_name(&CONTENT, 7, &spec, 0),
            })
        );
        for screen in [
            ScreenName::Name,
            ScreenName::Race,
            ScreenName::Sign,
            ScreenName::Gender,
            ScreenName::Class,
            ScreenName::ChargenTalents,
        ] {
            s.handle(&Press::nav(Nav::Down));
            assert_eq!(
                s.handle(&Press::nav(Nav::Enter)),
                ScreenAction::Return(screen)
            );
        }
        s.handle(&Press::nav(Nav::Down));
        assert_eq!(s.selected, 0, "the rows wrap");
        assert_eq!(s.handle(&Press::nav(Nav::Back)), ScreenAction::Close);
    }

    #[test]
    fn a_draft_fills_missing_steps_from_the_scenario_then_the_first_option() {
        let d = draft(1);
        assert_eq!(
            d.spec(&CONTENT).map(|s| s.to_string()).as_deref(),
            Some("human,female,thief")
        );
        assert_eq!(
            Draft::default()
                .spec(&CONTENT)
                .map(|s| s.to_string())
                .as_deref(),
            Some("elf,male,barbarian")
        );
        // Without a scenario the summary cannot start a run.
        let mut s = SummaryScreen::new(&CONTENT, &Draft::default());
        assert_eq!(s.handle(&Press::nav(Nav::Enter)), ScreenAction::Stay);
    }

    #[test]
    fn the_name_is_what_is_typed_else_the_first_random_name() {
        let mut d = draft(5);
        let spec = d.spec(&CONTENT).expect("a character");
        let first = random_name(&CONTENT, 5, &spec, 0).expect("a name");
        assert_eq!(d.name(&CONTENT), first);
        // Remembered from before going back, then what the Name step on the stack holds (an
        // empty field, the random one; spaces around a name go).
        d.remembered.set(Step::Name, "Brakka");
        assert_eq!(d.name(&CONTENT), "Brakka");
        d.chosen.set(Step::Name, "");
        assert_eq!(d.name(&CONTENT), first);
        d.chosen.set(Step::Name, " Hilde ");
        assert_eq!(d.name(&CONTENT), "Hilde");
        // The summary starts the run with it.
        let mut s = SummaryScreen::new(&CONTENT, &d);
        let ScreenAction::Choose(MenuChoice::NewRun { name, .. }) =
            s.handle(&Press::nav(Nav::Enter))
        else {
            panic!("a new run");
        };
        assert_eq!(name.as_deref(), Some("Hilde"));
    }
}

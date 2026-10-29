//! The character screen (spec §6; `@`): who the player is, as far as they know it. Their name,
//! their race, gender, and class, their age, birthday, and birth sign (spec §8 "Character
//! creation"; when the run has a character), their score in each stat (spec §8 "Stats") with its
//! abbreviation and full name (as the rules read it: a score a status effect or stat damage
//! lowers is shown lowered, in a warning color), their hit points, Defence, and Protection
//! (spec §8 "Combat"; the totals only), their alignment band (spec §8 "Alignment": `Kind`, never
//! the score), the status effects they suffer, by name (spec §8 "Status
//! effects"), the diseases they know they have, by name (spec §8 "Diseases": `Diseased  Filth
//! fever`), what they weigh and carry, their burden, and the weights at which it changes (spec §8 "Burden": they follow from
//! scores the player knows). Beside the stats, the attacks they have ready (main hand, off hand,
//! missile weapon), each with its to-hit, damage, and range as a strike would roll them now
//! (`PerceptionView::own_attacks`): the owner's one exception to hidden numbers. Never a
//! modifier, a potential, a creature's to-hit, or how severe an effect is.
//!
//! Read-only: Esc, Enter, or its own key closes it; the Skills key opens the Skills screen over
//! it (spec §8 "Skills"; Esc there comes back here). It is read from the perception snapshot when
//! it opens; no game time passes while it is open, so it cannot go stale.

use super::menu::{Nav, Panel, Press};
use super::screens::{ScreenAction, ScreenName};
use super::{ACCENT, TEXT, TEXT_DIM};
use crate::input::{Bindings, KeyCommand};
use crate::render::console::Console;
use warlock_core::burden::{Burden, Load};
use warlock_core::character::{Character, Gender};
use warlock_core::color::Rgb;
use warlock_core::content::Content;
use warlock_core::object::{Weight, format_weight};
use warlock_core::perception::{PerceptionView, Progress};
use warlock_core::stats::{PerStat, Stat};
use warlock_core::time::GameTime;

/// Width of the stat name column: the longest name and a space.
const NAME_W: i32 = 14;

/// The player's resistances as rows (spec §8 "Damage types"): one per word, in the order
/// immune, resistant, vulnerable, each listing its kinds in damage type order; none at 0.
fn resistance_rows(view: &PerceptionView<'_>) -> Vec<(&'static str, String)> {
    use warlock_core::damage::ResistanceWord;
    let content = view.content();
    let own = view.own_resistances();
    [
        ResistanceWord::Immune,
        ResistanceWord::Resistant,
        ResistanceWord::Vulnerable,
    ]
    .into_iter()
    .filter_map(|word| {
        let kinds: Vec<&str> = (own.iter())
            .filter(|(_, w)| *w == word)
            .map(|(k, _)| content.damage_type(*k).name.as_str())
            .collect();
        (!kinds.is_empty()).then(|| (word.label(), kinds.join(", ")))
    })
    .collect()
}

/// Where a value's neighbour on its row begins (Mana beside Hit points, Protection beside
/// Defence), from the value's column: room for the value and a gap.
const BESIDE: i32 = 8;

/// Width of a neighbour's name column: the longest name ("Protection") and a space.
const BESIDE_NAME_W: i32 = 11;

/// Width of the panel: the birth and the burden's thresholds fit on one line each, and an
/// attack's numbers beside the stats.
const WIDTH: i32 = super::info_tabs::panel_width(80);

/// Where the attacks begin, from the stats' column: past the score and a gap.
const ATTACKS_X: i32 = 23;

/// Width of an attack's label column ("Main hand") and a space; its numbers go under the name.
const ATTACK_LABEL_W: i32 = 11;

/// A score lowered by a status effect or stat damage, and the status effects and diseases
/// themselves: a muted red.
const LOWERED: Rgb = Rgb::new(0xd0, 0x78, 0x68);

#[derive(Debug)]
pub struct CharacterScreen {
    name: String,
    /// Race, gender, and class, by name; empty for a run without a character.
    identity: Vec<(&'static str, String)>,
    /// Age, birthday, and birth sign ([`birth_parts`]); empty without a character.
    birth: Vec<String>,
    /// The birth sign's gifts and flaw by name, in grant order (spec §8 "Birth sign"): the
    /// dimmed Stars line; empty without any.
    stars: Vec<String>,
    scores: PerStat<u8>,
    /// The scores without status effects: a score below its own is lowered.
    base: PerStat<u8>,
    /// The status effects they suffer, by name, the most important first.
    effects: Vec<String>,
    /// The diseases they know they have, by name (one that has not progressed yet is only
    /// "Diseased").
    diseases: Vec<String>,
    /// What they resist, in words only (spec §8 "Damage types"): a row per word ("Resistant
    /// to", "Vulnerable to", "Immune to") with the kinds it holds for, in that order.
    resistances: Vec<(&'static str, String)>,
    weight: Weight,
    /// What they carry against what they can; `None` outside a run.
    load: Option<Load>,
    /// Character level and experience (spec §8 "Experience").
    progress: Progress,
    /// Hit points left and the most.
    hp: (i32, i32),
    /// Mana left and the most.
    mana: (i32, i32),
    /// Defence: the total only.
    defence: i32,
    /// Protection: the total only.
    protection: i32,
    /// The combat stance's name.
    stance: String,
    /// The alternative resources they hold (spec §8 "Resources"), in resource order: the name,
    /// and what it holds of its maximum (`7/20`).
    resources: Vec<(String, String)>,
    /// How many talents they have (spec §8 "Talents"): picked, and the sign's; not the class's
    /// abilities.
    talents: usize,
    /// Their alignment band's name (spec §8 "Alignment"), never the score; `None` without
    /// alignment rules.
    alignment: Option<&'static str>,
    /// The guilds they belong to (spec §8 "Quests": "Guilds"): the rank's title, in its metal's
    /// colour, and the guild's name in a sentence ("the Weyfarers' Gilde").
    guilds: Vec<(String, Rgb, String)>,
    /// The attacks they have ready: where each comes from, what it is made with, and its
    /// numbers (`+4 to hit, 1d8+2 slashing`), beside the stats.
    attacks: Vec<(&'static str, String, String)>,
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
                stars: Vec::new(),
                scores: PerStat::splat(warlock_core::stats::AVERAGE),
                base: PerStat::splat(warlock_core::stats::AVERAGE),
                effects: Vec::new(),
                diseases: Vec::new(),
                resistances: Vec::new(),
                weight: Weight::default(),
                load: None,
                progress: Progress {
                    level: 1,
                    xp: 0,
                    next: None,
                },
                hp: (0, 0),
                mana: (0, 0),
                defence: warlock_core::combat::BASE_DEFENCE,
                protection: 0,
                stance: "Neutral".into(),
                resources: Vec::new(),
                talents: 0,
                alignment: None,
                guilds: Vec::new(),
                attacks: Vec::new(),
            };
        };
        let content = view.content();
        // The character's name, as typed, with a lucky day's title; without a character, what
        // the player is called.
        let name = match view.character() {
            Some(c) => c.titled_name(content),
            None => capitalized(
                content
                    .creature_id("player")
                    .map_or("you", |id| content.creature(id).name.as_str()),
            ),
        };
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
        let stars = (view.own_talents().into_iter())
            .filter(|t| t.granted && !t.race && !t.boon)
            .map(|t| match t.spent {
                true => format!("{} (spent)", t.name),
                false => t.name,
            })
            .collect();
        Self {
            name,
            identity,
            birth,
            stars,
            scores: view.own_scores(),
            base: view.own_base_scores(),
            effects: (view.own_effects())
                .map(|s| content.status(s).name.clone())
                .collect(),
            diseases: (view.own_diseases())
                .map(|d| content.disease(d).name.clone())
                .collect(),
            resistances: resistance_rows(view),
            weight: view.own_weight(),
            load: Some(view.own_load()),
            progress: view.own_experience(),
            hp: view.own_hp(),
            mana: view.own_mana(),
            defence: view.own_defence(),
            protection: view.own_protection(),
            stance: content.stance_def(view.own_stance()).name.clone(),
            resources: (view.own_resources().iter())
                .map(|r| {
                    let name = content.resource(r.resource).name.clone();
                    (name, format!("{}/{}", r.value, r.max))
                })
                .collect(),
            talents: view.own_talents().iter().filter(|t| !t.class).count(),
            alignment: view.own_alignment().map(|b| b.name()),
            guilds: (view.own_guilds().iter())
                .map(|g| {
                    let color = g.guild.color(content, g.rank, TEXT);
                    (g.title().to_string(), color, g.guild.name.clone())
                })
                .collect(),
            attacks: (view.own_attacks().iter())
                .map(|a| (a.source.label(), a.name.clone(), a.summary(content)))
                .collect(),
        }
    }

    /// Applies a key press: Esc, Enter, or the screen's own key closes it; the Skills key opens
    /// the Skills screen.
    pub fn handle(&mut self, press: &Press, bindings: &Bindings) -> ScreenAction {
        let owner = (press.binding.as_ref()).and_then(|key| bindings.owner(key));
        match owner {
            Some(KeyCommand::Character) => ScreenAction::Close,
            Some(KeyCommand::Skills) => ScreenAction::Open(ScreenName::Skills),
            _ if matches!(press.nav, Nav::Back | Nav::Enter) => ScreenAction::Close,
            _ => ScreenAction::Stay,
        }
    }

    /// The attacks beside the stats, from (`x`, `y`) and at most `width` wide: two rows each,
    /// `Main hand  long sword` and its numbers under the name; three attacks fill the stats'
    /// six rows.
    fn draw_attacks(
        &self,
        console: &mut Console,
        x: i32,
        mut y: i32,
        width: i32,
        fits: impl Fn(i32) -> bool,
    ) {
        let clip =
            |text: &str, w: i32| -> String { text.chars().take(w.max(0) as usize).collect() };
        for (label, name, numbers) in &self.attacks {
            let value_w = width - ATTACK_LABEL_W;
            if fits(y) {
                console.print(x, y, label, TEXT_DIM);
                console.print(x + ATTACK_LABEL_W, y, &clip(name, value_w), TEXT);
            }
            if fits(y + 1) {
                console.print(x + ATTACK_LABEL_W, y + 1, &clip(numbers, value_w), TEXT);
            }
            y += 2;
        }
    }

    pub fn draw(&self, console: &mut Console, bindings: &Bindings) {
        // Rows: margin, the name, a gap, race, gender, class, the birth, and a gap (with a
        // character), the stats, a gap, level, experience, talents, hit points and mana, Defence and
        // Protection, and stance (and the resources held, the status effects and known diseases, if any), a gap, the weight, the
        // burden's thresholds, a gap, the footer, frame.
        let width = WIDTH.min(console.cols() as i32) - 4;
        let birth = fit_parts(&self.birth, width);
        let stars = stars_lines(&self.stars, width);
        let identity_rows = match self.identity.len() {
            0 => 0,
            n => (n + birth.len() + stars.len()) as i32 + 1,
        };
        let thresholds = (self.load.as_ref()).map_or(Vec::new(), |l| thresholds(l, width));
        let status_rows = self.resources.len() as i32
            + i32::from(self.alignment.is_some())
            + self.guilds.len() as i32
            + i32::from(!self.effects.is_empty())
            + i32::from(!self.diseases.is_empty())
            + self.resistances.len() as i32;
        let h = Stat::COUNT as i32 + 16 + identity_rows + thresholds.len() as i32 + status_rows;
        let panel = Panel::new(console, " Character ", WIDTH, h);
        super::info_tabs::draw_tabs(console, &panel, bindings, super::ScreenName::Character);
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
        for (i, line) in stars.iter().enumerate() {
            // `Stars   Fleet, Slight, Supple`: dimmed, the names lined up with the values.
            if fits(y) {
                if i == 0 {
                    console.print(x, y, "Stars", TEXT_DIM);
                }
                console.print(x + 8, y, line, TEXT_DIM);
            }
            y += 1;
        }
        y += i32::from(!self.identity.is_empty());
        self.draw_attacks(console, x + ATTACKS_X, y, width - ATTACKS_X, fits);
        for (stat, score) in self.scores.iter() {
            // `St  Strength      12`: the abbreviation, the name, the score right-aligned.
            if fits(y) {
                let color = if score < self.base[stat] {
                    LOWERED
                } else {
                    TEXT
                };
                console.print(x, y, stat.abbrev(), TEXT_DIM);
                console.print(x + 4, y, stat.name(), TEXT);
                console.print(x + 4 + NAME_W, y, &format!("{score:>2}"), color);
            }
            y += 1;
        }
        y += 1;
        // `Lv  Level  3`, `XP  Experience  85 / 120` (the total alone at the highest level),
        // `HP  Hit points  18/24`, and `Dv  Defence  15` in the stats' columns, with
        // `MP  Mana  5/12` beside the hit points and `Pv  Protection  3` beside the Defence.
        let (hp, max) = self.hp;
        let (mana, max_mana) = self.mana;
        let mana = ("MP", "Mana", format!("{mana}/{max_mana}"));
        let protection = ("Pv", "Protection", format!("{:>2}", self.protection));
        for (abbrev, name, value, beside) in [
            ("Lv", "Level", self.progress.level.to_string(), None),
            ("XP", "Experience", experience_text(&self.progress), None),
            ("", "Talents", self.talents.to_string(), None),
            ("HP", "Hit points", format!("{hp}/{max}"), Some(mana)),
            (
                "Dv",
                "Defence",
                format!("{:>2}", self.defence),
                Some(protection),
            ),
            ("", "Stance", self.stance.clone(), None),
        ] {
            if fits(y) {
                console.print(x, y, abbrev, TEXT_DIM);
                console.print(x + 4, y, name, TEXT);
                console.print(x + 4 + NAME_W, y, &value, TEXT);
                if let Some((abbrev, name, value)) = beside {
                    let x = x + 4 + NAME_W + BESIDE;
                    console.print(x, y, abbrev, TEXT_DIM);
                    console.print(x + 4, y, name, TEXT);
                    console.print(x + 4 + BESIDE_NAME_W, y, &value, TEXT);
                }
            }
            y += 1;
        }
        // `    Rage          7/20`: each resource held, under the stance.
        for (name, value) in &self.resources {
            if fits(y) {
                console.print(x + 4, y, name, TEXT);
                console.print(x + 4 + NAME_W, y, value, TEXT);
            }
            y += 1;
        }
        // `    Alignment     Kind`: the band, never the score.
        if let Some(band) = self.alignment {
            if fits(y) {
                console.print(x + 4, y, "Alignment", TEXT);
                console.print(x + 4 + NAME_W, y, band, TEXT);
            }
            y += 1;
        }
        // `    Guild         Tin of the Weyfarers' Gilde`: the title in its metal's colour.
        for (title, color, guild) in &self.guilds {
            if fits(y) {
                let at = x + 4 + NAME_W;
                console.print(x + 4, y, "Guild", TEXT);
                console.print(at, y, title, *color);
                let rest = format!(" of {guild}");
                console.print(at + title.chars().count() as i32, y, &rest, TEXT);
            }
            y += 1;
        }
        // `    Status        Bleeding, Poisoned`: names only, never how bad.
        if !self.effects.is_empty() {
            if fits(y) {
                console.print(x + 4, y, "Status", TEXT);
                console.print(x + 4 + NAME_W, y, &self.effects.join(", "), LOWERED);
            }
            y += 1;
        }
        // `    Diseased      Filth fever`: the diseases they know they have.
        if !self.diseases.is_empty() {
            if fits(y) {
                let names = capitalized(&self.diseases.join(", "));
                console.print(x + 4, y, "Diseased", TEXT);
                console.print(x + 4 + NAME_W, y, &names, LOWERED);
            }
            y += 1;
        }
        // `    Resistant to  fire, cold`, `    Vulnerable to acid`: words, never a percentage.
        for (label, kinds) in &self.resistances {
            if fits(y) {
                let color = if *label == "Vulnerable to" {
                    LOWERED
                } else {
                    TEXT
                };
                console.print(x + 4, y, label, TEXT);
                console.print(x + 4 + NAME_W, y, kinds, color);
            }
            y += 1;
        }
        y += 1;
        if fits(y) {
            let w = &self.weight;
            // `carrying 28.4 kg, burdened`: the burden is named when there is one.
            let burden = (self.load.map(|l| l.burden()))
                .filter(|b| *b != Burden::Unburdened)
                .map_or(String::new(), |b| format!(", {}", b.id()));
            let weight = match w.carried {
                0 => format!("Weight {}, carrying nothing", format_weight(w.total())),
                carried => format!(
                    "Weight {} (you {}, carrying {}{burden})",
                    format_weight(w.total()),
                    format_weight(w.base),
                    format_weight(carried),
                ),
            };
            console.print(x, y, &weight, TEXT);
        }
        for line in &thresholds {
            y += 1;
            if fits(y) {
                console.print(x, y, line, TEXT_DIM);
            }
        }
        let close = match bindings.keys_of(KeyCommand::Character).first() {
            Some(key) => format!("{} or Esc close", key.display()),
            None => "Esc close".to_string(),
        };
        let footer = match bindings.keys_of(KeyCommand::Skills).first() {
            Some(key) => format!("{close}   {} skills", key.display()),
            None => close,
        };
        panel.footer(console, &footer);
    }
}

/// Experience as shown: `85 / 120`, the total and the total at which the next level comes; just
/// the total at the highest level.
pub fn experience_text(progress: &Progress) -> String {
    match progress.next {
        Some(next) => format!("{} / {next}", progress.xp),
        None => progress.xp.to_string(),
    }
}

/// The gender as the character screen shows it: its name, and for "other" the stat it raised
/// (`Other (Willpower +1)`), named once the run has begun as the rest of the character is;
/// character creation never names it (#256).
fn gender_text(character: &Character) -> String {
    match character.spec.gender {
        Gender::Other => format!("Other ({} +1)", character.raised.name()),
        gender => gender.name().to_string(),
    }
}

/// The Stars line's names (spec §8 "Birth sign"), fitted to `width` cells after the label: one
/// line, two at most (the second ending in "…" if even that is too little).
fn stars_lines(names: &[String], width: i32) -> Vec<String> {
    let mut lines = fit_joined(names, ", ", width - 8);
    if lines.len() > 2 {
        lines.truncate(2);
        lines[1].push('…');
    }
    lines
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
    fit_joined(parts, " · ", width)
}

/// `parts` joined with `sep` into lines of at most `width` characters, a new line where the next
/// part would not fit.
pub fn fit_joined(parts: &[String], sep: &str, width: i32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let sep_len = sep.chars().count();
    for part in parts {
        match lines.last_mut() {
            Some(line)
                if (line.chars().count() + sep_len + part.chars().count()) as i32 <= width =>
            {
                line.push_str(sep);
                line.push_str(part);
            }
            _ => lines.push(part.clone()),
        }
    }
    lines
}

/// The weights at which the player's burden changes (spec §8 "Burden"), in lines of at most
/// `width` characters: `Burdened over 15 kg, strained over 30 kg, overloaded over 45 kg`.
pub fn thresholds(load: &Load, width: i32) -> Vec<String> {
    let mut parts = load.over().to_vec();
    parts[0] = capitalized(&parts[0]);
    let last = parts.len() - 1;
    for part in &mut parts[..last] {
        part.push(',');
    }
    fit_joined(&parts, " ", width)
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
    use warlock_core::content::{CombatStance, Content};
    use warlock_core::{Action, Dir, Positions, World};

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
        world.apply(&Action::PickUp(Positions::of([0])));
        let text = drawn(&world, 80, 30);
        let row = |needle: &str| {
            text.iter()
                .find(|l| l.contains(needle))
                .unwrap_or_else(|| panic!("{needle} in {text:#?}"))
        };
        assert!(row(" Character ").contains('┌') || row(" Character ").contains('─'));
        // The character's name heads it.
        assert!(
            row("Tamsin")
                .trim_matches(|c| c == ' ' || c == '│')
                .starts_with("Tamsin")
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
            // Toughness 10 and a human's 4 and a thief's 4; Defence 10 and Dexterity's share,
            // and no Protection beside it.
            // Beside them a human's 1 mana and a thief's 1, and the mental stats' +1, 0, and -1.
            "HP  Hit points    18/18   MP  Mana       2/2",
            "Dv  Defence       11      Pv  Protection  0",
            "Lv  Level         1",
            // A human thief's level 2: 192 × 95 %.
            "XP  Experience    0 / 182",
            // A thief starts Callous: the band, never the score.
            "    Alignment     Callous",
        ] {
            row(expected);
        }
        assert!(row("Stance").contains("Neutral"));
        world.apply(&Action::SetStance(CombatStance::Aggressive));
        // A row taller: the thief holds openings from level 1.
        let text = drawn(&world, 80, 31);
        assert!(
            (text.iter()).any(|l| l.contains("Stance") && l.contains("Aggressive")),
            "{text:#?}"
        );
        row("Weight 73.9 kg (you 70 kg, carrying 3.9 kg)");
        // St 9 To 10: a limit of 42 kg, by thirds; the line right below the weight.
        let at = |needle: &str| text.iter().position(|l| l.contains(needle));
        let thresholds = "Burdened over 14 kg, strained over 28 kg, overloaded over 42 kg";
        assert_eq!(at(thresholds), at("Weight 73.9 kg").map(|y| y + 1));
        row("@ or Esc close   A skills");
        let empty_handed = drawn(&showcase(), 80, 30);
        assert!(
            (empty_handed.iter()).any(|l| l.contains("Weight 70.4 kg (you 70 kg, carrying 400 g)")),
            "{empty_handed:#?}"
        );
    }

    #[test]
    fn names_the_guild_rank_in_its_metals_colour() {
        let content = Arc::new(Content::builtin().expect("content"));
        let mut world = World::new(content.clone(), "ceaptun", None).expect("Cēaptūn");
        assert!(!drawn(&world, 80, 30).join("\n").contains("Guild"));
        // To Wat One-Eye in the Gildhalle, and the first link of his chain.
        let walk = "move:E*26 move:S*4 move:E*9 open:S move:S*5 ask:S:quest-guild_quarry";
        for action in warlock_core::action::parse_actions(walk).expect("actions") {
            world.apply(&action);
        }
        let screen = CharacterScreen::new(Some(&world.perception()));
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console, &Bindings::default());
        let text = text(&console);
        let y = (text.iter())
            .position(|l| l.contains("    Guild         Tin of the Weyfarers' Gilde"))
            .unwrap_or_else(|| panic!("{text:#?}"));
        let x = text[y].find("Tin").expect("the title") as i32;
        let x = text[y][..x as usize].chars().count() as i32;
        let tin = content.material(content.material_id("tin").expect("tin"));
        assert_eq!(console.get(x, y as i32).map(|c| c.fg), tin.color);
        assert_eq!(console.get(x + 4, y as i32).map(|c| c.fg), Some(TEXT));
    }

    #[test]
    fn names_the_burden_and_wraps_the_thresholds_to_the_width() {
        let mut world = showcase();
        let load = world.perception().own_load();
        assert_eq!(
            thresholds(&load, 68),
            ["Burdened over 14 kg, strained over 28 kg, overloaded over 42 kg"]
        );
        assert_eq!(
            thresholds(&load, 44),
            [
                "Burdened over 14 kg, strained over 28 kg,",
                "overloaded over 42 kg"
            ]
        );
        // Two chain mails (22 kg) for average scores (St 10 To 10: burdened over 15 kg).
        world = crate::ui::hoard();
        world.apply(&Action::PickUp(Positions::default()));
        let text = drawn(&world, 80, 30).join("\n");
        for expected in [
            "Weight 92 kg (you 70 kg, carrying 22 kg, burdened)",
            "Burdened over 15 kg, strained over 30 kg, overloaded over 45 kg",
        ] {
            assert!(text.contains(expected), "{expected:?} in\n{text}");
        }
    }

    #[test]
    fn lists_status_effects_and_marks_the_scores_they_lower() {
        let mut world = showcase();
        assert!(!drawn(&world, 80, 30).join("\n").contains("Status"));
        // Strong poison (four degrees) and a wound.
        for status in ["poisoned", "poisoned", "poisoned", "poisoned", "bleeding"] {
            world.apply(&Action::Afflict(status.into()));
        }
        let text = drawn(&world, 80, 30);
        let joined = text.join("\n");
        for expected in [
            "St  Strength       8",
            "Dx  Dexterity     12",
            "To  Toughness     10",
            "Status        Bleeding, Poisoned",
        ] {
            assert!(joined.contains(expected), "{expected:?} in\n{joined}");
        }
        // The lowered scores in their own color.
        let screen = CharacterScreen::new(Some(&world.perception()));
        let mut console = Console::default();
        console.resize(80, 30);
        screen.draw(&mut console, &Bindings::default());
        let y = text
            .iter()
            .position(|l| l.contains("St  Strength"))
            .expect("St");
        let score = (0..80)
            .filter_map(|x| console.get(x, y as i32))
            .find(|c| c.ch == '8')
            .expect("the score");
        assert_eq!(score.fg, LOWERED);
    }

    #[test]
    fn names_a_disease_once_known_and_shows_its_stat_damage() {
        use warlock_core::Rest;
        use warlock_core::time::Duration;
        let mut world = crate::ui::hoard();
        world.apply(&Action::Infect("the_shakes".into()));
        let day = Action::Rest(Rest::For(Duration::hours(24)));
        world.apply(&day);
        // It shows: Diseased, but not yet which disease.
        let text = drawn(&world, 80, 30).join("\n");
        assert!(text.contains("Status        Diseased"), "{text}");
        assert!(!text.contains("shakes"), "{text}");
        // A day later it has progressed once: named. (A check lost shows as Dexterity lowered.)
        // Hunger may cut a rest short ("You are getting hungry."): rest on until it has.
        for _ in 0..4 {
            if world.perception().own_diseases().next().is_some() {
                break;
            }
            world.apply(&day);
        }
        let text = drawn(&world, 80, 30).join("\n");
        for expected in ["Status        Diseased", "Diseased      The shakes"] {
            assert!(text.contains(expected), "{expected:?} in\n{text}");
        }
        let dexterity = world.perception().own_scores()[Stat::Dexterity];
        assert!(
            text.contains(&format!("Dx  Dexterity     {dexterity:>2}")),
            "{text}"
        );
        assert_eq!(world.perception().own_base_scores()[Stat::Dexterity], 10);
    }

    #[test]
    fn never_shows_potentials_or_modifiers() {
        let world = showcase();
        let mut text = drawn(&world, 80, 30).join("\n");
        // The attacks' numbers are the one exception (the owner's): take them out first.
        let view = world.perception();
        for attack in view.own_attacks() {
            let numbers = attack.summary(view.content());
            assert!(text.contains(&numbers), "{numbers:?} in\n{text}");
            text = text.replace(&numbers, "");
        }
        // The showcase player's potentials (16 each), and their modifiers (+2, -1), appear
        // nowhere else; neither do signs or the words.
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
        let s = Press {
            binding: KeyBinding::from_text("A"),
            ..Press::typed('A')
        };
        assert_eq!(
            screen.handle(&s, &b),
            ScreenAction::Open(ScreenName::Skills)
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

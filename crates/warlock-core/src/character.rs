//! Character creation (spec §8 "Character creation"): who the player is, made when a run
//! starts. A [`CharacterSpec`] is the choice — a race (`data/races.toml`), a gender, and a class
//! (`data/classes/`) — written `dwarf,female,warrior` on the command line. A [`Character`]
//! is what came of it in a run: the name, the choice, the stat the gender raised, the birth
//! date, and the birth sign (`data/horoscope.toml`).
//!
//! - **Name**: chosen with the character, or one of the race's names for the gender
//!   (`data/names.toml`) picked from the run seed ([`random_name`]: the stream
//!   `character/name/<race>,<gender>/<draw>`, so no other roll moves), at most [`MAX_NAME`]
//!   characters ([`check_name`]). A run told no name gets [`default_name`]: the scenario's own
//!   name for its own character, else the race's first random name. The rules never read it.
//! - **Age and birthday**: the age is uniform in the race's `age` range and the birthday
//!   (month, then day) uniform over the year, from the random stream `character/birth/<race>`.
//!   The birth date counts back from the run's start date, so the character is exactly that old
//!   on it ([`BirthDate::counted_back`]).
//! - **Birth sign**: the horoscope's sign of the birth year
//!   ([`Horoscope::sign_of_year`](crate::content::Horoscope::sign_of_year)); never chosen.
//! - **Starting scores**: [`AVERAGE`] plus the race's adjustment, plus the gender's (male:
//!   Strength +1; female: Dexterity +1; other: +1 to a stat picked from the run seed, see
//!   [`Gender::raises`]), plus the class's, plus the sign's animal's and element's, kept within
//!   [`SCORES`](crate::stats::SCORES).
//! - **Potentials**: the races file's `base_potential` plus the race's potential adjustment,
//!   kept within the same range and never below the starting score; like every potential they
//!   are hidden from the player.
//! - **Dice**: adjustments may be dice ([`Dice`](crate::dice::Dice)), rolled from the stream
//!   `character/rolls/<race>,<gender>,<class>` in this order: the race's scores, the class's,
//!   the animal's, the element's (each in stat order), then the race's potentials, then the
//!   skills (below). Fixed numbers draw nothing.
//! - **Skills** (spec §8 "Skills"): the race's and the class's skills, each by its dice, rolled
//!   after everything above (so they never change a score) in this order: the race's skills, the
//!   class's (each in skill order), then the race's extra class skill (`class_skill`, humans):
//!   one of the class's skills picked uniformly, then its dice. A skill both train sums the two.
//!   Every other skill is untrained.
//! - **Eyes**: the player's vision profile is the race's.
//! - **Weapon skills** (spec §8 "Weapon skills"): the race's starting ranks plus the class's,
//!   and one set of the class's `weapon_skill_choice` if it has one, at most 10 each; each
//!   then raises the skills it defaults to. Dice among them are rolled from their own stream,
//!   `character/weapon-skills/<race>,<gender>,<class>`, in this order: the race's (in skill
//!   order), the class's, the pick of the choice (one draw), then the chosen set's; so the
//!   stat rolls above never move.
//! - **Fighting styles** (spec §8 "Fighting styles"): the class's starting ranks in styles, and
//!   one set of its `style_choice` if it has one, from yet another stream,
//!   `character/styles/<race>,<gender>,<class>`: the class's (in skill order), the pick of the
//!   choice, then the chosen set's; so neither the stats nor the weapon skills move. Styles
//!   raise the styles they default to, like weapon skills.
//! - **Starting kit** (spec §8 "Character creation"): what the class gives, then the race, then
//!   the birth sign, from yet another stream, `character/kit/<race>,<gender>,<class>`
//!   ([`crate::kit`]); a line may go only with a weapon skill or style the class trained (the
//!   picked choice included), so the kit follows those rolls without moving them.
//!
//! Everything follows from the content, the spec, the run seed, and the start date, so a run
//! (and its replay) is reproduced by its scenario, seed, and character.

use crate::alignment::Score;
use crate::content::{Calendar, ClassDef, Content, RaceDef, ScenarioDef, Sign, WeaponSkillId};
use crate::kit::{self, Pack};
use crate::rng::Rng;
use crate::skills::{self, Skills};
use crate::stats::{self, AVERAGE, PerStat, Stat, Stats, adjusted};
use crate::time::{DAYS_PER_MONTH, GameTime, MONTHS_PER_YEAR};
use crate::weapon_skills::{self, WeaponSkills};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The player's gender. It raises one stat by one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
    Other,
}

impl Gender {
    /// Every gender, in the order offered.
    pub const ALL: [Gender; 3] = [Gender::Male, Gender::Female, Gender::Other];

    /// The id on the command line and in saves: `female`.
    pub const fn id(self) -> &'static str {
        match self {
            Gender::Male => "male",
            Gender::Female => "female",
            Gender::Other => "other",
        }
    }

    /// The name: `Female`.
    pub const fn name(self) -> &'static str {
        match self {
            Gender::Male => "Male",
            Gender::Female => "Female",
            Gender::Other => "Other",
        }
    }

    pub fn from_id(id: &str) -> Option<Gender> {
        Gender::ALL.into_iter().find(|g| g.id() == id)
    }

    /// The stat this gender raises by one in a run with `seed`: Strength for male, Dexterity
    /// for female, and for other a stat picked from the seed ([`other_stat`]).
    pub fn raises(self, seed: u64) -> Stat {
        match self {
            Gender::Male => Stat::Strength,
            Gender::Female => Stat::Dexterity,
            Gender::Other => other_stat(seed),
        }
    }
}

/// The stat the gender "other" raises in a run with `seed`: uniform over the stats, from the
/// random stream `character/gender` (`rng.rs`), so it never depends on the rest of the
/// character or on gameplay.
pub fn other_stat(seed: u64) -> Stat {
    let mut rng = Rng::stream(seed, "character/gender");
    Stat::ALL[rng.index(Stat::COUNT)]
}

/// What the player chooses: a race, a gender, and a class, by id. Written
/// `race,gender,class` (`dwarf,female,warrior`) on the command line; an object with those
/// three fields in scenarios, replays, and saves.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterSpec {
    pub race: String,
    pub gender: Gender,
    pub class: String,
}

/// A date on the calendar that may lie before its year 1: a birth date. `month` is 1 to 12,
/// `day` 1 to 30.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BirthDate {
    pub year: i64,
    pub month: u32,
    pub day: u32,
}

impl BirthDate {
    /// The birth date of someone who is `age` years old on `today` and has their birthday on
    /// `month`/`day`: their last birthday on or before `today` was their `age`th.
    pub fn counted_back(today: GameTime, age: u32, month: u32, day: u32) -> Self {
        let not_yet = (month, day) > (today.month(), today.day());
        Self {
            year: today.year() as i64 - i64::from(age) - i64::from(not_yet),
            month,
            day,
        }
    }

    /// Age in whole years on `date`: one more on every birthday; 0 before the first.
    pub fn age_on(&self, date: GameTime) -> u32 {
        let before = (date.month(), date.day()) < (self.month, self.day);
        let age = date.year() as i64 - self.year - i64::from(before);
        u32::try_from(age).unwrap_or(0)
    }

    /// Day, month, and year: `3 Seedtide 1417`.
    pub fn text(&self, calendar: &Calendar) -> String {
        format!("{} {} {}", self.day, calendar.month(self.month), self.year)
    }
}

/// Who the player is in a run (spec §8 "Character creation"): the choice and what came of it.
/// Saved with the run; the player knows all of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Character {
    /// The character's name ([`check_name`]): shown on the character screen, the death screen,
    /// and the list of saved characters; the rules never read it.
    pub name: String,
    /// What was chosen.
    pub spec: CharacterSpec,
    /// The stat the gender raised by one.
    pub raised: Stat,
    pub born: BirthDate,
    /// The birth sign; `None` for content without a horoscope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sign: Option<Sign>,
}

impl Character {
    /// Age in whole years on `date`.
    pub fn age_on(&self, date: GameTime) -> u32 {
        self.born.age_on(date)
    }

    /// Whether the character was born in a year of `element` (an id: "is this character
    /// Fire-born?" is `born_of_element("fire")`).
    pub fn born_of_element(&self, element: &str) -> bool {
        self.sign.as_ref().is_some_and(|s| s.is_element(element))
    }

    /// Whether the character was born in a year of `animal` (an id: `manticore`).
    pub fn born_in_year_of(&self, animal: &str) -> bool {
        self.sign.as_ref().is_some_and(|s| s.is_animal(animal))
    }

    /// The birth sign as players say it (`Fire Manticore`); `None` without one.
    pub fn sign_name(&self, content: &Content) -> Option<String> {
        let sign = self.sign.as_ref()?;
        Some(content.horoscope().map_or_else(
            || format!("{} {}", sign.element, sign.animal),
            |h| h.name(sign),
        ))
    }

    /// The lucky day the character was born on (spec §8 "Birth sign"), if they were.
    pub fn lucky<'c>(&self, content: &'c Content) -> Option<&'c crate::content::Lucky> {
        content.lucky(self.sign.as_ref()?, &self.born)
    }

    /// The title a lucky day gives, after the name ("the Iron Scourge"); `None` for most.
    pub fn title<'c>(&self, content: &'c Content) -> Option<&'c str> {
        self.lucky(content).map(|l| l.title.as_str())
    }

    /// The name with a lucky day's title, as the character and death screens say it: `Brakka,
    /// the Iron Scourge`; the name alone for most.
    pub fn titled_name(&self, content: &Content) -> String {
        match self.title(content) {
            Some(title) => format!("{}, {title}", self.name),
            None => self.name.clone(),
        }
    }

    /// The sign as players say it with a lucky day's title: `Metal Manticore, the Iron
    /// Scourge`; `None` without a sign.
    pub fn sign_title(&self, content: &Content) -> Option<String> {
        let name = self.sign_name(content)?;
        Some(match self.title(content) {
            Some(title) => format!("{name}, {title}"),
            None => name,
        })
    }

    /// The talents the character's sign gives them (spec §8 "Birth sign"), by id: the animal's,
    /// the element's, then a lucky day's. None without a sign or a horoscope.
    pub fn grants<'c>(&self, content: &'c Content) -> Vec<&'c str> {
        match (&self.sign, content.horoscope()) {
            (Some(sign), Some(h)) => h.grants(sign, self.born.day),
            _ => Vec::new(),
        }
    }

    /// Whether `content` has everything this character names: race, class, and sign.
    pub fn check(&self, content: &Content) -> Result<(), String> {
        self.spec.check(content)?;
        match (&self.sign, content.horoscope()) {
            (None, _) => Ok(()),
            (Some(sign), Some(h)) if h.parts(sign).is_some() => Ok(()),
            (Some(sign), _) => Err(format!(
                "unknown birth sign `{} {}`",
                sign.element, sign.animal
            )),
        }
    }
}

/// The longest name a character may have, in characters.
pub const MAX_NAME: usize = 20;

/// The name of a character there is no name for: no names for their race and gender, and no
/// scenario name.
pub const NAMELESS: &str = "Wanderer";

/// `name` as a character's name: without the spaces around it. An error when that is empty or
/// longer than [`MAX_NAME`] characters, or when it holds a control character.
pub fn check_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("a character's name cannot be empty".into());
    }
    if name.chars().count() > MAX_NAME {
        return Err(format!(
            "the name `{name}` is longer than {MAX_NAME} characters"
        ));
    }
    if name.chars().any(char::is_control) {
        return Err(format!(
            "the name `{}` holds a control character",
            name.escape_debug()
        ));
    }
    Ok(name)
}

/// The `draw`th random name (0 first) for a character of `spec` in a run with `seed`: one of the
/// race's names for the gender (`data/names.toml`; `other` picks from both lists), from the
/// stream `character/name/<race>,<gender>/<draw>`. `None` when there are none to pick from.
pub fn random_name(
    content: &Content,
    seed: u64,
    spec: &CharacterSpec,
    draw: u32,
) -> Option<String> {
    let names = content.names().offered(&spec.race, spec.gender);
    if names.is_empty() {
        return None;
    }
    let label = format!("character/name/{},{}/{draw}", spec.race, spec.gender.id());
    let mut rng = Rng::stream(seed, &label);
    Some(names[rng.index(names.len())].to_string())
}

/// The name a run of `scenario` with `seed` gives a character of `spec` when it is told none (a
/// replay or a command line without one): the scenario's own name when `spec` is its own
/// character, else the race's first random name ([`random_name`]), else [`NAMELESS`].
pub fn default_name(
    content: &Content,
    scenario: &ScenarioDef,
    seed: u64,
    spec: &CharacterSpec,
) -> String {
    (scenario.character.as_ref())
        .filter(|own| *own == spec)
        .and(scenario.character_name.clone())
        .or_else(|| random_name(content, seed, spec, 0))
        .unwrap_or_else(|| NAMELESS.into())
}

/// What a character starts a run with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Start {
    /// The starting scores and (hidden) potentials.
    pub stats: Stats,
    /// The player's vision profile: the race's.
    pub vision: String,
    /// What the race and the class add to the hit points (spec §8 "Combat").
    pub hp: i32,
    /// What the race and the class add to the mana (spec §8 "Mana").
    pub mana: i32,
    /// The trained skills, with their starting values (spec §8 "Skills").
    pub skills: Skills,
    /// Starting ranks in weapon skills, defaults included (spec §8 "Weapon skills").
    pub weapon_skills: WeaponSkills,
    /// The starting alignment score (spec §8 "Alignment"): the race's plus the class's.
    pub alignment: Score,
    pub character: Character,
    /// The starting kit: what the class, the race, and the sign give (a scenario may add to it
    /// or leave it out, [`kit::starting_pack`]).
    pub kit: Pack,
}

impl CharacterSpec {
    /// The race, or why there is none.
    pub fn race<'c>(&self, content: &'c Content) -> Result<&'c RaceDef, String> {
        content
            .race(&self.race)
            .ok_or_else(|| format!("unknown race `{}`", self.race))
    }

    /// The class, or why there is none.
    pub fn class<'c>(&self, content: &'c Content) -> Result<&'c ClassDef, String> {
        content
            .class(&self.class)
            .ok_or_else(|| format!("unknown class `{}`", self.class))
    }

    /// Whether `content` has the race and the class.
    pub fn check(&self, content: &Content) -> Result<(), String> {
        self.race(content).and(self.class(content)).map(|_| ())
    }

    /// What this character starts a run with `seed` on `today` (the run's start) with: see the
    /// module docs. Its name is the first random name ([`random_name`]) until the caller names
    /// it.
    pub fn start(&self, content: &Content, seed: u64, today: GameTime) -> Result<Start, String> {
        let race = self.race(content)?;
        let class = self.class(content)?;
        let raised = self.gender.raises(seed);
        let born = birth(race, seed, today);
        let horoscope = content.horoscope();
        let sign = horoscope.map(|h| h.sign_of_year(born.year));
        let mut rng = Rng::stream(seed, &format!("character/rolls/{self}"));
        let mut by = stats::roll(&race.stats, &mut rng);
        by[raised] += 1;
        let mut add = |more: PerStat<i32>| {
            for stat in Stat::ALL {
                by[stat] += more[stat];
            }
        };
        add(stats::roll(&class.stats, &mut rng));
        if let Some((animal, element)) = horoscope.zip(sign.as_ref()).and_then(|(h, s)| h.parts(s))
        {
            add(stats::roll(&animal.stats, &mut rng));
            add(stats::roll(&element.stats, &mut rng));
        }
        let potential = stats::roll(&race.potential, &mut rng);
        let skills = starting_skills(race, class, &mut rng);
        let (weapon_skills, trained) = self.weapon_skills(content, seed);
        let character = Character {
            name: random_name(content, seed, self, 0).unwrap_or_else(|| NAMELESS.into()),
            spec: self.clone(),
            raised,
            born,
            sign,
        };
        Ok(Start {
            stats: starting_stats(by, potential, content.races().base_potential),
            vision: race.vision.clone(),
            hp: race.hp + class.hp,
            mana: race.mana + class.mana,
            skills,
            weapon_skills,
            alignment: Score::new(race.alignment + class.alignment),
            kit: kit::roll(content, &character, &trained, seed),
            character,
        })
    }
}

impl CharacterSpec {
    /// The starting ranks in weapon skills of this character in a run with `seed` (see the
    /// module docs), and the weapon skills and styles the class trained: its fixed ranks and the
    /// sets of its choices that were picked (what a kit's `trained` asks). Content without
    /// races or classes gives none.
    fn weapon_skills(&self, content: &Content, seed: u64) -> (WeaponSkills, Vec<WeaponSkillId>) {
        let (Ok(race), Ok(class)) = (self.race(content), self.class(content)) else {
            return (WeaponSkills::default(), Vec::new());
        };
        let mut rng = Rng::stream(seed, &format!("character/weapon-skills/{self}"));
        let mut ranks = Vec::new();
        weapon_skills::roll_ranks(&race.weapon_skills, &mut rng, &mut ranks);
        let mut trained: Vec<WeaponSkillId> = Vec::new();
        let mut train = |set: &[(WeaponSkillId, crate::dice::Dice)]| {
            trained.extend(set.iter().map(|&(id, _)| id));
        };
        train(&class.weapon_skills);
        weapon_skills::roll_ranks(&class.weapon_skills, &mut rng, &mut ranks);
        let mut choose = |rng: &mut Rng, choice: &[weapon_skills::RankDice], ranks: &mut Vec<_>| {
            if !choice.is_empty() {
                let picked = &choice[rng.index(choice.len())];
                train(picked);
                weapon_skills::roll_ranks(picked, rng, ranks);
            }
        };
        choose(&mut rng, &class.weapon_skill_choice, &mut ranks);
        let mut rng = Rng::stream(seed, &format!("character/styles/{self}"));
        weapon_skills::roll_ranks(&class.styles, &mut rng, &mut ranks);
        choose(&mut rng, &class.style_choice, &mut ranks);
        trained.extend(class.styles.iter().map(|&(id, _)| id));
        let ranks = ranks
            .into_iter()
            .map(|(id, rank)| (id, rank.clamp(0, i32::from(crate::content::MAX_RANK)) as u8));
        let skills = WeaponSkills::from_ranks(content.weapon_skills(), ranks);
        (skills, trained)
    }
}

/// The birth date of a new character of `race` in a run with `seed` that starts `today`.
fn birth(race: &RaceDef, seed: u64, today: GameTime) -> BirthDate {
    let mut rng = Rng::stream(seed, &format!("character/birth/{}", race.id));
    let (young, old) = (*race.age.start() as i32, *race.age.end() as i32);
    let age = rng.range(young, old) as u32;
    let month = rng.range(1, MONTHS_PER_YEAR as i32) as u32;
    let day = rng.range(1, DAYS_PER_MONTH as i32) as u32;
    BirthDate::counted_back(today, age, month, day)
}

/// Starting skills: the race's, then the class's, then the race's extra class skill on one of
/// the class's skills picked uniformly (drawing nothing when there is none to pick from).
fn starting_skills(race: &RaceDef, class: &ClassDef, rng: &mut Rng) -> Skills {
    let mut out = Skills::default();
    skills::roll_grants(&race.skills, &mut out, rng);
    skills::roll_grants(&class.skills, &mut out, rng);
    if !race.class_skill.is_zero() && !class.skills.is_empty() {
        let (skill, _) = class.skills[rng.index(class.skills.len())];
        out.add(skill, race.class_skill.roll(rng));
    }
    out
}

/// Starting stats from the adjustments: scores 10 plus `by`, potentials `base_potential` plus
/// `potential`, all within 1 to 40 and no potential below its score.
fn starting_stats(by: PerStat<i32>, potential: PerStat<i32>, base_potential: u8) -> Stats {
    let mut scores = PerStat::splat(AVERAGE);
    let mut potentials = PerStat::splat(AVERAGE);
    for stat in Stat::ALL {
        scores[stat] = adjusted(AVERAGE, by[stat]);
        potentials[stat] = adjusted(base_potential, potential[stat]);
    }
    Stats::new(scores, potentials)
}

impl fmt::Display for CharacterSpec {
    /// `race,gender,class`, as on the command line.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.race, self.gender.id(), self.class)
    }
}

impl FromStr for CharacterSpec {
    type Err = String;

    /// `race,gender,class` (`dwarf,female,warrior`). Only the gender is checked here; the race
    /// and class are checked against content when a run starts ([`CharacterSpec::check`]).
    fn from_str(text: &str) -> Result<Self, String> {
        let parts: Vec<&str> = text.split(',').map(str::trim).collect();
        let [race, gender, class] = parts[..] else {
            return Err(format!(
                "expected RACE,GENDER,CLASS (e.g. dwarf,female,warrior), got `{text}`"
            ));
        };
        let gender = Gender::from_id(gender).ok_or_else(|| {
            format!("unknown gender `{gender}` (expected male, female, or other)")
        })?;
        if race.is_empty() || class.is_empty() {
            return Err(format!(
                "the race and the class must be named, got `{text}`"
            ));
        }
        Ok(Self {
            race: race.to_string(),
            gender,
            class: class.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentFiles;
    use crate::dice::Dice;
    use crate::skills::SkillId;
    use std::collections::{BTreeMap, BTreeSet};

    fn content() -> Content {
        Content::builtin().expect("builtin content")
    }

    fn spec(text: &str) -> CharacterSpec {
        text.parse().expect("a character")
    }

    /// 08:00 on `day` `month` `year`.
    fn date(year: u64, month: u32, day: u32) -> GameTime {
        GameTime::from_calendar(year, month, day, 8, 0).expect("a date")
    }

    /// The default start: 11 Darkening 1444.
    fn start_date() -> GameTime {
        date(1444, 11, 11)
    }

    fn start(content: &Content, text: &str, seed: u64) -> Start {
        spec(text)
            .start(content, seed, start_date())
            .expect("starts")
    }

    fn scores(stats: &Stats) -> Vec<u8> {
        Stat::ALL.map(|s| stats.score(s)).to_vec()
    }

    fn potentials(stats: &Stats) -> Vec<u8> {
        Stat::ALL.map(|s| stats.potential(s)).to_vec()
    }

    /// The builtin content plus a race `giant` and a class `brute` with fixed adjustments, and a
    /// horoscope of one animal and one element; `horoscope: false` leaves the horoscope out.
    fn fixed_content(horoscope: bool) -> Content {
        let mut files = ContentFiles::builtin();
        let races = include_str!("../../../assets/data/races.toml");
        files.insert(
            "data/races.toml",
            &format!(
                "{races}\n[[race]]\nid = \"giant\"\nname = \"Giant\"\ndescription = \"\"\n\
                 vision = \"normal\"\nage = [20, 20]\n\
                 stats = {{ strength = 4, dexterity = -2 }}\npotential = {{ strength = 3 }}\n"
            ),
        );
        files.insert(
            "data/classes/99-brute.toml",
            "[[class]]\nid = \"brute\"\nname = \"Brute\"\ndescription = \"\"\n\
             stats = { strength = 2, intelligence = -3 }\n",
        );
        if horoscope {
            files.insert(
                "data/horoscope.toml",
                "anchor_year = 1\n\
                 [[animal]]\nid = \"ox\"\nname = \"Ox\"\ndescription = \"\"\n\
                 stats = { toughness = 2, strength = 1 }\n\
                 [[element]]\nid = \"stone\"\nname = \"Stone\"\ndescription = \"\"\n\
                 stats = { willpower = 1 }\n",
            );
        } else {
            files.remove("data/horoscope.toml");
        }
        Content::load(&files).expect("valid content")
    }

    #[test]
    fn specs_parse_and_print_as_race_gender_class() {
        let dwarf = spec("dwarf,female,warrior");
        assert_eq!(dwarf.race, "dwarf");
        assert_eq!(dwarf.gender, Gender::Female);
        assert_eq!(dwarf.class, "warrior");
        assert_eq!(dwarf.to_string(), "dwarf,female,warrior");
        assert_eq!(spec(" elf , other , wizard "), spec("elf,other,wizard"));
        for (bad, why) in [
            ("dwarf,female", "expected RACE,GENDER,CLASS"),
            ("dwarf,female,warrior,extra", "expected RACE,GENDER,CLASS"),
            ("dwarf,woman,warrior", "unknown gender `woman`"),
            (",male,warrior", "must be named"),
        ] {
            let err = bad.parse::<CharacterSpec>().unwrap_err();
            assert!(err.contains(why), "{bad}: {err}");
        }
        let json = serde_json::to_string(&dwarf).expect("serializes");
        assert_eq!(
            json,
            r#"{"race":"dwarf","gender":"female","class":"warrior"}"#
        );
        assert_eq!(
            serde_json::from_str::<CharacterSpec>(&json).ok(),
            Some(dwarf)
        );
    }

    #[test]
    fn scores_add_race_gender_class_animal_and_element_to_ten() {
        let content = fixed_content(true);
        // Giant: St +4 Dx -2; male: St +1; brute: St +2 In -3; ox: St +1 To +2; stone: Wi +1.
        let giant = start(&content, "giant,male,brute", 1);
        assert_eq!(scores(&giant.stats), [18, 8, 12, 7, 11, 10]);
        // Potentials: 15 plus the race's (St +3), never below the score.
        assert_eq!(potentials(&giant.stats), [18, 15, 15, 15, 15, 15]);
        assert_eq!(giant.vision, "normal");
        assert_eq!(giant.character.raised, Stat::Strength);
        let sign = giant.character.sign.as_ref().expect("a sign");
        assert!(giant.character.born_of_element("stone"));
        assert!(giant.character.born_in_year_of("ox"));
        assert!(!giant.character.born_of_element("fire"));
        assert_eq!(
            sign,
            &Sign {
                animal: "ox".into(),
                element: "stone".into()
            }
        );
        assert_eq!(
            giant.character.sign_name(&content).as_deref(),
            Some("Stone Ox")
        );
        // Female raises Dexterity instead; other the stat its seed picks.
        let female = start(&content, "giant,female,brute", 1);
        assert_eq!(scores(&female.stats), [17, 9, 12, 7, 11, 10]);
        for seed in 0..12 {
            let other = start(&content, "giant,other,brute", seed);
            let raised = other_stat(seed);
            assert_eq!(other.character.raised, raised);
            let mut expected = [17, 8, 12, 7, 11, 10];
            expected[Stat::ALL.iter().position(|&s| s == raised).expect("a stat")] += 1;
            assert_eq!(scores(&other.stats), expected, "seed {seed}");
        }
        // Without a horoscope there is no sign, and nothing from it.
        let content = fixed_content(false);
        let signless = start(&content, "giant,male,brute", 1);
        assert_eq!(signless.character.sign, None);
        assert_eq!(scores(&signless.stats), [17, 8, 10, 7, 10, 10]);
        assert!(!signless.character.born_of_element("stone"));
        assert_eq!(signless.character.sign_name(&content), None);
    }

    #[test]
    fn scores_and_potentials_are_clamped_to_the_valid_range() {
        let by = |pairs: &[(Stat, i32)]| {
            let mut out = PerStat::splat(0);
            for &(stat, v) in pairs {
                out[stat] = v;
            }
            out
        };
        // St 10 + 31 caps at 40 and its potential 35 + 10 too; Dx 10 - 30 stops at 1, its
        // potential 35 - 10 = 25.
        let stats = starting_stats(
            by(&[(Stat::Strength, 31), (Stat::Dexterity, -30)]),
            by(&[(Stat::Strength, 10), (Stat::Dexterity, -10)]),
            35,
        );
        assert_eq!(scores(&stats), [40, 1, 10, 10, 10, 10]);
        assert_eq!(potentials(&stats), [40, 25, 35, 35, 35, 35]);
        // A potential below the score is raised to it; a potential of 1 - 10 stops at 1.
        let low = starting_stats(
            by(&[(Stat::Strength, 20)]),
            by(&[(Stat::Dexterity, -10)]),
            1,
        );
        assert_eq!(scores(&low)[..2], [30, 10]);
        assert_eq!(potentials(&low)[..3], [30, 10, 10]);
        let floor = starting_stats(by(&[]), by(&[(Stat::Dexterity, -10)]), 5);
        assert_eq!(potentials(&floor)[..2], [10, 10]);
    }

    #[test]
    fn rolls_are_the_same_for_the_same_seed_and_choices() {
        let content = content();
        let warrior = start(&content, "dwarf,female,warrior", 1);
        assert_eq!(warrior, start(&content, "dwarf,female,warrior", 1));
        // The warrior's St 1d3+1 and To 1d3 (with the dwarf's To 1d2+2) vary from seed to seed,
        // within what the dice allow; other choices roll their own.
        let mut strength = BTreeSet::new();
        let mut differ = 0;
        for seed in 0..100 {
            let a = start(&content, "dwarf,female,warrior", seed);
            let b = start(&content, "dwarf,female,barbarian", seed);
            strength.insert(a.stats.score(Stat::Strength));
            differ += usize::from(scores(&a.stats) != scores(&b.stats));
        }
        assert!(strength.len() >= 3, "{strength:?}");
        assert!(differ > 50, "{differ}");
        // The birth depends on the seed and the race only: the class does not move a birthday.
        let a = start(&content, "elf,male,wizard", 9);
        let b = start(&content, "elf,female,merchant", 9);
        assert_eq!(a.character.born, b.character.born);
        assert_ne!(
            a.character.born,
            start(&content, "elf,male,wizard", 10).character.born
        );
    }

    #[test]
    fn rolled_scores_stay_within_what_the_dice_allow() {
        let content = content();
        let h = content.horoscope().expect("a horoscope");
        for seed in 0..40 {
            let s = start(&content, "half_orc,other,barbarian", seed);
            let race = content.race("half_orc").expect("race");
            let class = content.class("barbarian").expect("class");
            let (animal, element) = h
                .parts(s.character.sign.as_ref().expect("a sign"))
                .expect("parts");
            for stat in Stat::ALL {
                let dice: [Dice; 4] = [
                    race.stats[stat],
                    class.stats[stat],
                    animal.stats[stat],
                    element.stats[stat],
                ];
                let gender = i32::from(stat == s.character.raised);
                let lo = 10 + gender + dice.iter().map(Dice::min).sum::<i32>();
                let hi = 10 + gender + dice.iter().map(Dice::max).sum::<i32>();
                let score = i32::from(s.stats.score(stat));
                assert!(
                    (lo..=hi).contains(&score),
                    "seed {seed} {stat:?}: {score} not in {lo}..={hi}"
                );
            }
        }
    }

    #[test]
    fn ages_and_birthdays_follow_the_race_and_count_back_from_the_start() {
        let content = content();
        let today = start_date();
        for race in &content.races().list {
            let mut ages = BTreeSet::new();
            let mut months = BTreeSet::new();
            for seed in 0..300 {
                let c = spec(&format!("{},male,commoner", race.id))
                    .start(&content, seed, today)
                    .expect("starts")
                    .character;
                let age = c.age_on(today);
                assert!(race.age.contains(&age), "{} {age}", race.id);
                assert!((1..=12).contains(&c.born.month) && (1..=30).contains(&c.born.day));
                // A birthday still to come this year was last year's age.
                let not_yet = (c.born.month, c.born.day) > (11, 11);
                assert_eq!(c.born.year, 1444 - i64::from(age) - i64::from(not_yet));
                ages.insert(age);
                months.insert(c.born.month);
            }
            assert_eq!(months.len(), 12, "{}", race.id);
            assert!(
                ages.len() > 10.min(race.age.clone().count()),
                "{} {ages:?}",
                race.id
            );
        }
        // Humans start 16 to 35, elves 90 to 300.
        assert_eq!(content.race("human").map(|r| r.age.clone()), Some(16..=35));
        assert_eq!(content.race("elf").map(|r| r.age.clone()), Some(90..=300));
    }

    #[test]
    fn ages_count_birthdays() {
        let born = BirthDate {
            year: 1417,
            month: 3,
            day: 3,
        };
        assert_eq!(born.age_on(date(1444, 3, 2)), 26);
        assert_eq!(born.age_on(date(1444, 3, 3)), 27, "on the birthday");
        assert_eq!(born.age_on(date(1444, 11, 11)), 27);
        assert_eq!(born.age_on(date(1445, 1, 1)), 27);
        assert_eq!(born.age_on(date(1417, 3, 3)), 0);
        assert_eq!(born.age_on(date(1416, 1, 1)), 0, "before birth");
        assert_eq!(BirthDate::counted_back(date(1444, 11, 11), 27, 3, 3), born);
        assert_eq!(BirthDate::counted_back(date(1444, 3, 3), 27, 3, 3), born);
        assert_eq!(BirthDate::counted_back(date(1444, 3, 2), 26, 3, 3), born);
        // Before the calendar's year 1, years go on down through 0.
        let early = BirthDate::counted_back(date(1, 6, 1), 30, 12, 30);
        assert_eq!(early.year, -30);
        assert_eq!(early.age_on(date(1, 6, 1)), 30);
        assert_eq!(born.text(content().calendar()), "3 Seedtide 1417");
    }

    #[test]
    fn the_sign_follows_the_birth_year_and_changes_on_the_first_of_frostmere() {
        let content = content();
        let h = content.horoscope().expect("a horoscope");
        assert_eq!(h.anchor_year, 1);
        assert_eq!(h.great_cycle(), 60);
        let name = |year| h.name(&h.sign_of_year(year));
        // Year 1, the year of the Great Race, starts a great cycle, and so do 1381 and 1441.
        for year in [1, 61, 1381, 1441] {
            assert_eq!(name(year), "Wood Salamander", "{year}");
        }
        assert_eq!(name(1444), "Metal Unicorn");
        assert_eq!(name(1417), "Fire Salamander");
        assert_eq!(name(1416), "Wood Barghest");
        assert_eq!(name(1440), "Water Barghest", "the last of a great cycle");
        // On 1 Frostmere 1444 someone aged 27 with a birthday on 30 Deepwinter was born in 1416,
        // one with a birthday that day in 1417: across the year boundary, another sign.
        let today = date(1444, 1, 1);
        let late = BirthDate::counted_back(today, 27, 12, 30);
        let early = BirthDate::counted_back(today, 27, 1, 1);
        assert_eq!((late.year, early.year), (1416, 1417));
        assert_ne!(h.sign_of_year(late.year), h.sign_of_year(early.year));
        // A new character's sign is their birth year's.
        for seed in 0..50 {
            let c = start(&content, "gnome,female,bard", seed).character;
            assert_eq!(c.sign, Some(h.sign_of_year(c.born.year)));
        }
    }

    #[test]
    fn names_are_checked() {
        assert_eq!(check_name("  Brakka "), Ok("Brakka"));
        assert_eq!(check_name("Ånne-Marie O'Hara"), Ok("Ånne-Marie O'Hara"));
        assert_eq!(
            check_name(&"x".repeat(MAX_NAME)).map(str::len),
            Ok(MAX_NAME)
        );
        assert!(check_name(&"x".repeat(MAX_NAME + 1)).is_err());
        assert!(check_name("").is_err());
        assert!(check_name("   ").is_err());
        assert!(check_name("Bra\nkka").is_err());
    }

    #[test]
    fn random_names_come_from_the_race_and_gender_and_the_seed() {
        let content = content();
        let dwarf = spec("dwarf,female,warrior");
        let females = content.names().offered("dwarf", Gender::Female);
        let mut seen = BTreeSet::new();
        for draw in 0..40 {
            let name = random_name(&content, 7, &dwarf, draw).expect("a name");
            assert!(females.contains(&name.as_str()), "{name}");
            assert_eq!(random_name(&content, 7, &dwarf, draw), Some(name.clone()));
            seen.insert(name);
        }
        assert!(seen.len() > 5, "draws vary: {seen:?}");
        // The class plays no part; the seed does.
        let monk = spec("dwarf,female,monk");
        assert_eq!(
            random_name(&content, 7, &monk, 0),
            random_name(&content, 7, &dwarf, 0)
        );
        let other = spec("dwarf,other,warrior");
        let both = content.names().offered("dwarf", Gender::Other);
        assert!(both.contains(&random_name(&content, 7, &other, 0).unwrap().as_str()));
        // A new character starts with the first draw.
        let c = start(&content, "dwarf,female,warrior", 7).character;
        assert_eq!(Some(c.name), random_name(&content, 7, &dwarf, 0));
        // Without names to pick from there is none, and the character is nameless.
        let mut files = ContentFiles::builtin();
        files.remove("data/names.toml");
        let bare = Content::load(&files).expect("valid");
        assert_eq!(random_name(&bare, 7, &dwarf, 0), None);
        assert_eq!(
            start(&bare, "dwarf,female,warrior", 7).character.name,
            NAMELESS
        );
    }

    #[test]
    fn names_never_move_a_roll() {
        let content = content();
        let mut files = ContentFiles::builtin();
        files.remove("data/names.toml");
        let bare = Content::load(&files).expect("valid");
        for text in [
            "dwarf,female,warrior",
            "human,other,thief",
            "gnome,male,bard",
        ] {
            let (named, unnamed) = (start(&content, text, 11), start(&bare, text, 11));
            assert_eq!(named.stats, unnamed.stats, "{text}");
            assert_eq!(named.skills, unnamed.skills, "{text}");
            assert_eq!(named.weapon_skills, unnamed.weapon_skills, "{text}");
            let (mut a, b) = (named.character, unnamed.character);
            a.name = b.name.clone();
            assert_eq!(a, b, "{text}");
        }
    }

    #[test]
    fn the_default_name_is_the_scenarios_for_its_own_character() {
        let content = content();
        let showcase = content.scenario("showcase").expect("showcase");
        let own = showcase.character.clone().expect("a character");
        assert_eq!(default_name(&content, showcase, 1, &own), "Tamsin");
        assert_eq!(default_name(&content, showcase, 99, &own), "Tamsin");
        let dwarf = spec("dwarf,female,warrior");
        assert_eq!(
            Some(default_name(&content, showcase, 1, &dwarf)),
            random_name(&content, 1, &dwarf, 0)
        );
    }

    #[test]
    fn characters_round_trip_and_check_against_content() {
        let content = content();
        let c = start(&content, "half_elf,other,ranger", 4).character;
        let json = serde_json::to_string(&c).expect("serializes");
        assert!(json.contains(&format!(r#""name":"{}""#, c.name)), "{json}");
        assert!(
            json.contains(r#""spec":{"race":"half_elf","gender":"other","class":"ranger"}"#),
            "{json}"
        );
        assert!(json.contains(r#""born":{"year":"#), "{json}");
        assert_eq!(
            serde_json::from_str::<Character>(&json).ok().as_ref(),
            Some(&c)
        );
        assert!(c.check(&content).is_ok());
        let mut odd = c.clone();
        odd.sign = Some(Sign {
            animal: "dragon".into(),
            element: "fire".into(),
        });
        assert_eq!(
            odd.check(&content).unwrap_err(),
            "unknown birth sign `fire dragon`"
        );
        let mut no_race = c;
        no_race.spec.race = "orc".into();
        assert_eq!(no_race.check(&content).unwrap_err(), "unknown race `orc`");
    }

    #[test]
    fn unknown_races_and_classes_are_errors() {
        let content = content();
        assert_eq!(
            spec("orc,male,warrior")
                .start(&content, 1, start_date())
                .unwrap_err(),
            "unknown race `orc`"
        );
        assert_eq!(
            spec("human,male,necromancer").check(&content).unwrap_err(),
            "unknown class `necromancer`"
        );
        assert!(spec("half_orc,other,witch").check(&content).is_ok());
    }

    /// The built-in content without skills: no skills file, and no race or class training any.
    fn skill_free_content() -> Content {
        let mut files = ContentFiles::builtin();
        files.remove("data/skills.toml");
        let untrained = |text: &str| -> String {
            let kept: Vec<&str> = (text.lines())
                .filter(|l| {
                    !l.starts_with("skills = ")
                        && !l.starts_with("class_skill = ")
                        && !l.starts_with("known_for = ")
                })
                .collect();
            kept.join("\n")
        };
        files.edit_table("races", untrained);
        files.edit_table("classes", untrained);
        files.edit_table("creatures", untrained);
        Content::load(&files).expect("valid content")
    }

    #[test]
    fn skills_are_rolled_after_everything_else_and_never_move_a_score() {
        let content = content();
        let plain = skill_free_content();
        assert!(plain.skills().is_empty());
        for race in &content.races().list {
            for class in content.classes() {
                for gender in Gender::ALL {
                    let spec = CharacterSpec {
                        race: race.id.clone(),
                        gender,
                        class: class.id.clone(),
                    };
                    for seed in 0..3 {
                        let with = spec.start(&content, seed, start_date()).expect("starts");
                        let without = spec.start(&plain, seed, start_date()).expect("starts");
                        // Scores, potentials, eyes, hit points, birth: all as before skills.
                        assert_eq!(with.stats, without.stats, "{spec} seed {seed}");
                        assert_eq!(with.character, without.character, "{spec} seed {seed}");
                        assert_eq!((with.vision, with.hp), (without.vision, without.hp));
                        assert!(without.skills.is_empty());
                        // Trained in exactly what the race and the class train.
                        let granted: BTreeSet<SkillId> = (race.skills.iter().chain(&class.skills))
                            .map(|&(id, _)| id)
                            .collect();
                        let trained: BTreeSet<SkillId> =
                            with.skills.iter().map(|(id, _)| id).collect();
                        assert_eq!(trained, granted, "{spec}");
                        // Each value within what the dice allow, both summed, plus a human's
                        // extra class skill on one of them.
                        let mut extra = 0;
                        for (id, t) in with.skills.iter() {
                            let dice: Vec<Dice> = (race.skills.iter().chain(&class.skills))
                                .filter(|&&(s, _)| s == id)
                                .map(|&(_, d)| d)
                                .collect();
                            let lo: i32 = dice.iter().map(Dice::min).sum();
                            let hi: i32 = dice.iter().map(Dice::max).sum();
                            let v = i32::from(t.value);
                            if v > hi {
                                extra += 1;
                                assert!(v <= hi + race.class_skill.max(), "{spec} {id}");
                            }
                            assert!(v >= lo, "{spec} {id}: {v} < {lo}");
                            assert_eq!(t.marks, 0);
                        }
                        assert!(extra <= 1, "{spec}");
                    }
                }
            }
        }
        // The same seed and choices, the same skills; another seed rolls others.
        let thief = start(&content, "human,female,thief", 1);
        assert_eq!(
            thief.skills,
            start(&content, "human,female,thief", 1).skills
        );
        let others =
            (2..12).filter(|&s| start(&content, "human,female,thief", s).skills != thief.skills);
        assert!(others.count() > 5);
    }

    #[test]
    fn humans_add_their_extra_class_skill_to_one_of_the_classs() {
        let content = content();
        let human = content.race("human").expect("human");
        assert_eq!(human.class_skill.to_string(), "2d4+4");
        assert!(human.skills.is_empty());
        let class = content.class("merchant").expect("merchant");
        // Over many seeds, every one of the merchant's skills gets the extra now and then.
        let mut boosted = BTreeSet::new();
        for seed in 0..60 {
            let s = start(&content, "human,male,merchant", seed);
            for &(id, dice) in &class.skills {
                if i32::from(s.skills.value(id).expect("trained")) > dice.max() {
                    boosted.insert(id);
                }
            }
        }
        assert_eq!(boosted.len(), class.skills.len(), "{boosted:?}");
    }

    #[test]
    fn every_builtin_combination_starts_within_range() {
        let content = content();
        let mut lowest = BTreeMap::new();
        let mut highest = BTreeMap::new();
        for race in &content.races().list {
            for gender in Gender::ALL {
                for class in content.classes() {
                    let spec = CharacterSpec {
                        race: race.id.clone(),
                        gender,
                        class: class.id.clone(),
                    };
                    for seed in 0..4 {
                        let start = spec.start(&content, seed, start_date()).expect("starts");
                        for stat in Stat::ALL {
                            let (s, p) = (start.stats.score(stat), start.stats.potential(stat));
                            assert!((4..=22).contains(&s), "{spec} {stat:?} {s}");
                            assert!(p >= s && p <= 22, "{spec} {stat:?} {p}");
                            let low = lowest.entry(stat).or_insert(s);
                            *low = (*low).min(s);
                            let high = highest.entry(stat).or_insert(s);
                            *high = (*high).max(s);
                        }
                        assert!(content.vision(&start.vision).is_some());
                        assert!(start.character.check(&content).is_ok());
                    }
                }
            }
        }
        // Some characters are poor on purpose, some strong.
        assert!(lowest.values().all(|&s| s <= 8), "{lowest:?}");
        assert!(lowest.values().any(|&s| s <= 5), "{lowest:?}");
        assert!(highest.values().all(|&s| s >= 13), "{highest:?}");
        assert!(highest.values().any(|&s| s >= 18), "{highest:?}");
    }
}

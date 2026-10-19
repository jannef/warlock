//! Weapon skills (`assets/data/weapon_skills.toml`, spec §8 "Weapon skills"): the categories of
//! weapons an actor trains with, ADOM-like, and how they relate, after *The Riddle of Steel*.
//!
//! - **Skills** come in file order, each with an `id`, a `name` for the weapon skills screen, a
//!   `phrase` for the message of a rank gained ("You feel more skilled *with long blades*."), a
//!   [`Role`] (what uses it), the need `curve` it trains along, and its benefits per rank, 0 to
//!   [`MAX_RANK`] ([`Benefit`]). Rank 0 is untrained: no benefits.
//! - **Marks** ([`MarkRules`]): every attempt earns some, a success more, scaled by how the
//!   opponent's level compares with the rank. A rank is gained when its marks reach the need of
//!   the next rank, which grows along an exponential **curve** (`curves.melee`: 15, 30, 60, ...).
//! - **Defaults** (The Riddle of Steel): symmetric penalties between related skills. When a rank
//!   is gained in A, every B with `rank(A) − penalty(A, B) > rank(B)` rises to that rank
//!   (`crate::weapon_skills`).
//!
//! The numbers are rules, never shown in the game: the player sees ranks, and a marks bar. A
//! skill is general enough for what comes next (fighting styles): another role with the same
//! ranks, marks, curve, and benefits.

use super::ContentError;
use crate::content::object::MAX_BONUS;
use crate::dice::Dice;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The highest rank; rank 0 is untrained.
pub const MAX_RANK: u8 = 10;
/// How many ranks there are, 0 to [`MAX_RANK`].
pub const RANKS: usize = MAX_RANK as usize + 1;
/// The most a curve may ask for one rank.
pub const MAX_NEED: u32 = 1_000_000;
/// The most a rank may widen a threat range by, in natural rolls.
pub const MAX_THREAT_WIDENING: u8 = 10;
/// The most a rank may take off an action's cost, in percent.
pub const MAX_SAVING: u8 = 50;
/// The most marks one use may earn before scaling, and the most a percent may be.
const MAX_MARKS: u32 = 1000;

/// Index of a skill in [`WeaponSkillTable::skills`], in file order.
///
/// Ids are assigned in file order when content loads; saves record the content fingerprint to
/// catch a reordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WeaponSkillId(pub u8);

/// What uses a skill: a weapon that names it (`melee`, `launcher`, `thrown`), or an attack or a
/// defence that implies it (`unarmed`, `thrown`, `shield_block`, `shield_bash`; at most one
/// skill each).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Weapons wielded in a hand name it (`category`): their blows train it, and its Parry
    /// bonus defends while one is wielded.
    Melee,
    /// Missile weapons name it: their shots train it.
    Launcher,
    /// Every throw trains it, whatever is thrown; things that are only thrown (a rock) name it.
    Thrown,
    /// Bare hands and natural attacks.
    Unarmed,
    /// Defending with a held shield: a blow the shield blocked trains it (spec §8 "Combat": a
    /// miss credited to the Shield category), and it adds to the Shield category.
    ShieldBlock,
    /// Bashing with a held shield (the bash attack).
    ShieldBash,
}

impl Role {
    pub const ALL: [Role; 6] = [
        Role::Melee,
        Role::Launcher,
        Role::Thrown,
        Role::Unarmed,
        Role::ShieldBlock,
        Role::ShieldBash,
    ];

    /// Its id in the data file: `shield_block`.
    pub const fn id(self) -> &'static str {
        match self {
            Role::Melee => "melee",
            Role::Launcher => "launcher",
            Role::Thrown => "thrown",
            Role::Unarmed => "unarmed",
            Role::ShieldBlock => "shield_block",
            Role::ShieldBash => "shield_bash",
        }
    }

    /// Whether no weapon names it: it is used by an attack or a defence of its own, so there is
    /// at most one skill of it.
    pub const fn is_implicit(self) -> bool {
        !matches!(self, Role::Melee | Role::Launcher)
    }

    /// Whether it is used to attack (to-hit, damage, threat, and speed apply).
    pub const fn attacks(self) -> bool {
        !matches!(self, Role::ShieldBlock)
    }

    /// The section of the weapon skills screen it is listed in.
    pub const fn section(self) -> Section {
        match self {
            Role::Melee | Role::Unarmed => Section::Melee,
            Role::Launcher | Role::Thrown => Section::Missile,
            Role::ShieldBlock | Role::ShieldBash => Section::Shields,
        }
    }
}

/// The sections of the weapon skills screen, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section {
    Melee,
    Missile,
    Shields,
}

impl Section {
    pub const ALL: [Section; 3] = [Section::Melee, Section::Missile, Section::Shields];

    /// `Missile and thrown`.
    pub const fn name(self) -> &'static str {
        match self {
            Section::Melee => "Melee",
            Section::Missile => "Missile and thrown",
            Section::Shields => "Shields",
        }
    }
}

/// What a rank in a skill gives (rules only, never shown). Benefits of several skills add up
/// ([`Benefit::plus`]), so later skills (fighting styles) can stack on a weapon's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Benefit {
    /// Added to the to-hit of attacks with it.
    pub to_hit: i32,
    /// Added to every roll of their damage.
    pub damage: i32,
    /// Added to the wielded weapon's Parry bonus (the Parry category of Defence).
    pub parry: i32,
    /// Added to a held shield's Shield bonus (the Shield category).
    pub shield: i32,
    /// How many natural rolls lower a critical threatens from.
    pub threat: u8,
    /// Percent off the attack's cost (`melee`, `shoot`, `throw`).
    pub speed: u8,
    /// Percent off the cost of reloading (`reload`).
    pub reload: u8,
}

impl Benefit {
    /// Both benefits together.
    pub fn plus(self, other: Benefit) -> Benefit {
        Benefit {
            to_hit: self.to_hit + other.to_hit,
            damage: self.damage + other.damage,
            parry: self.parry + other.parry,
            shield: self.shield + other.shield,
            threat: self.threat.saturating_add(other.threat),
            speed: self.speed.saturating_add(other.speed),
            reload: self.reload.saturating_add(other.reload),
        }
    }

    /// `cost` with `percent` taken off, rounded down (never below 1).
    pub fn saving(cost: u32, percent: u8) -> u32 {
        let percent = u32::from(percent.min(100));
        (cost * (100 - percent) / 100).max(1)
    }
}

/// How marks are earned (ADOM-like): `attempt` for every use, `hit` more for a success (a hit, a
/// blow blocked), scaled by `100 + per_level × (opponent's level − rank)` percent, kept within 0
/// and `most`, rounded to the nearest mark (halves up): fewer against weak opponents and as the
/// rank rises, more against strong ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkRules {
    pub attempt: u32,
    pub hit: u32,
    pub per_level: u32,
    pub most: u32,
}

impl MarkRules {
    /// The percent of the base marks a use earns against an opponent of `level` at `rank`.
    pub fn percent(&self, level: u32, rank: u8) -> u32 {
        let difference = i64::from(level) - i64::from(rank);
        let percent = 100 + i64::from(self.per_level) * difference;
        percent.clamp(0, i64::from(self.most)) as u32
    }

    /// Marks for one use at `rank` against an opponent of `level`: a `success` (a hit, a blow
    /// blocked) or not.
    pub fn earned(&self, success: bool, level: u32, rank: u8) -> u32 {
        let base = self.attempt + if success { self.hit } else { 0 };
        (base * self.percent(level, rank) + 50) / 100
    }
}

/// A skill.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeaponSkillDef {
    pub id: String,
    /// As the weapon skills screen lists it: `Long blades`.
    pub name: String,
    /// What "You feel more skilled ..." says of it: `with long blades`.
    pub phrase: String,
    pub role: Role,
    /// The curve it trains along (a key of the file's `curves`).
    pub curve: String,
    /// Marks to go from each rank to the next: `need[r]` takes rank `r` to `r + 1`.
    pub need: [u32; MAX_RANK as usize],
    /// What each rank gives, 0 (nothing) to [`MAX_RANK`].
    pub benefits: [Benefit; RANKS],
}

/// Every skill, the defaults between them, and how marks are earned.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WeaponSkillTable {
    skills: Vec<WeaponSkillDef>,
    /// The penalty between two skills, keyed by the pair in id order.
    defaults: BTreeMap<(WeaponSkillId, WeaponSkillId), u8>,
    marks: Option<MarkRules>,
    /// The curves by name, as the file gives them.
    curves: BTreeMap<String, [u32; MAX_RANK as usize]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillFile {
    marks: MarkRules,
    curves: BTreeMap<String, Vec<u32>>,
    skill: Vec<SkillSpec>,
    #[serde(default)]
    default: Vec<DefaultSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillSpec {
    id: String,
    name: String,
    phrase: String,
    role: Role,
    curve: String,
    to_hit: Option<Vec<i32>>,
    damage: Option<Vec<i32>>,
    parry: Option<Vec<i32>>,
    shield: Option<Vec<i32>>,
    threat: Option<Vec<i32>>,
    speed: Option<Vec<i32>>,
    reload: Option<Vec<i32>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DefaultSpec {
    between: [String; 2],
    penalty: u8,
}

/// One benefit's ranks as the file gives them: [`RANKS`] numbers, 0 at rank 0, never falling,
/// each within `0..=max`.
fn ranks(field: &str, given: &Option<Vec<i32>>, max: i32) -> Result<[i32; RANKS], String> {
    let Some(values) = given else {
        return Ok([0; RANKS]);
    };
    let values: [i32; RANKS] = values.clone().try_into().map_err(|v: Vec<i32>| {
        format!(
            "`{field}` needs {RANKS} numbers, one per rank 0 to {MAX_RANK}, not {}",
            v.len()
        )
    })?;
    if values[0] != 0 {
        return Err(format!("`{field}`: rank 0 is untrained and gives nothing"));
    }
    if values.iter().any(|v| !(0..=max).contains(v)) {
        return Err(format!("`{field}`: each rank's value must be 0 to {max}"));
    }
    if values.windows(2).any(|w| w[1] < w[0]) {
        return Err(format!("`{field}`: a higher rank never gives less"));
    }
    Ok(values)
}

impl SkillSpec {
    /// The benefits per rank; a benefit the role does not use is refused.
    fn benefits(&self) -> Result<[Benefit; RANKS], String> {
        let bonus = MAX_BONUS;
        let to_hit = ranks("to_hit", &self.to_hit, bonus)?;
        let damage = ranks("damage", &self.damage, bonus)?;
        let parry = ranks("parry", &self.parry, bonus)?;
        let shield = ranks("shield", &self.shield, bonus)?;
        let threat = ranks("threat", &self.threat, i32::from(MAX_THREAT_WIDENING))?;
        let speed = ranks("speed", &self.speed, i32::from(MAX_SAVING))?;
        let reload = ranks("reload", &self.reload, i32::from(MAX_SAVING))?;
        let role = self.role;
        let unused = [
            ("to_hit", &self.to_hit, role.attacks()),
            ("damage", &self.damage, role.attacks()),
            ("threat", &self.threat, role.attacks()),
            ("speed", &self.speed, role.attacks()),
            ("parry", &self.parry, role == Role::Melee),
            ("shield", &self.shield, role == Role::ShieldBlock),
            ("reload", &self.reload, role == Role::Launcher),
        ];
        if let Some((field, ..)) = unused
            .iter()
            .find(|(_, given, used)| given.is_some() && !used)
        {
            return Err(format!(
                "`{field}` does nothing for a skill of role `{}`",
                role.id()
            ));
        }
        // The values are within u8 where they are percents or rolls (checked above).
        let small = |v: i32| v as u8;
        Ok(std::array::from_fn(|r| Benefit {
            to_hit: to_hit[r],
            damage: damage[r],
            parry: parry[r],
            shield: shield[r],
            threat: small(threat[r]),
            speed: small(speed[r]),
            reload: small(reload[r]),
        }))
    }
}

impl WeaponSkillTable {
    /// Parses `data/weapon_skills.toml`.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: SkillFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let marks = parsed.marks;
        if marks.attempt > MAX_MARKS || marks.hit > MAX_MARKS || marks.per_level > MAX_MARKS {
            return Err(err(format!(
                "marks: `attempt`, `hit`, and `per_level` must be at most {MAX_MARKS}"
            )));
        }
        if !(100..=MAX_MARKS).contains(&marks.most) {
            return Err(err(format!(
                "marks: `most` is a percent, 100 to {MAX_MARKS}"
            )));
        }
        let mut curves = BTreeMap::new();
        for (name, need) in parsed.curves {
            let need: [u32; MAX_RANK as usize] = need.try_into().map_err(|v: Vec<u32>| {
                err(format!(
                    "curve `{name}` needs {MAX_RANK} numbers, the marks for ranks 1 to \
                     {MAX_RANK}, not {}",
                    v.len()
                ))
            })?;
            if need[0] == 0 || need.windows(2).any(|w| w[1] <= w[0]) || need[9] > MAX_NEED {
                return Err(err(format!(
                    "curve `{name}`: each rank needs more marks than the last, from 1 up to \
                     {MAX_NEED}"
                )));
            }
            curves.insert(name, need);
        }
        if parsed.skill.is_empty() {
            return Err(err("at least one skill is required".into()));
        }
        if parsed.skill.len() > usize::from(u8::MAX) {
            return Err(err("too many skills".into()));
        }
        let mut skills: Vec<WeaponSkillDef> = Vec::new();
        for spec in parsed.skill {
            let id = &spec.id;
            let skill_err = |m: String| err(format!("skill `{id}`: {m}"));
            if skills.iter().any(|s| &s.id == id) {
                return Err(err(format!("duplicate skill id `{id}`")));
            }
            if spec.name.trim().is_empty() || spec.phrase.trim().is_empty() {
                return Err(skill_err("`name` and `phrase` must not be empty".into()));
            }
            if spec.role.is_implicit() && skills.iter().any(|s| s.role == spec.role) {
                return Err(skill_err(format!(
                    "there is already a skill of role `{}`, and it can only have one",
                    spec.role.id()
                )));
            }
            let need = *curves
                .get(&spec.curve)
                .ok_or_else(|| skill_err(format!("unknown curve `{}`", spec.curve)))?;
            let benefits = spec.benefits().map_err(skill_err)?;
            skills.push(WeaponSkillDef {
                id: spec.id,
                name: spec.name,
                phrase: spec.phrase,
                role: spec.role,
                curve: spec.curve,
                need,
                benefits,
            });
        }
        let mut table = Self {
            skills,
            defaults: BTreeMap::new(),
            marks: Some(marks),
            curves,
        };
        for spec in parsed.default {
            let [a, b] = &spec.between;
            let (Some(x), Some(y)) = (table.id(a), table.id(b)) else {
                return Err(err(format!(
                    "default between `{a}` and `{b}`: unknown skill"
                )));
            };
            if x == y {
                return Err(err(format!("default between `{a}` and itself")));
            }
            if !(1..=MAX_RANK).contains(&spec.penalty) {
                return Err(err(format!(
                    "default between `{a}` and `{b}`: `penalty` must be 1 to {MAX_RANK}"
                )));
            }
            if table
                .defaults
                .insert((x.min(y), x.max(y)), spec.penalty)
                .is_some()
            {
                return Err(err(format!(
                    "default between `{a}` and `{b}` is given twice"
                )));
            }
        }
        Ok(table)
    }

    /// Whether there are no skills (content without the file).
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Every skill, in file order.
    pub fn skills(&self) -> &[WeaponSkillDef] {
        &self.skills
    }

    /// Every skill with its id, in file order.
    pub fn ids(&self) -> impl Iterator<Item = (WeaponSkillId, &WeaponSkillDef)> {
        (self.skills.iter().enumerate()).map(|(i, s)| (WeaponSkillId(i as u8), s))
    }

    pub fn skill(&self, id: WeaponSkillId) -> &WeaponSkillDef {
        &self.skills[usize::from(id.0)]
    }

    /// The skill with data id `name`.
    pub fn id(&self, name: &str) -> Option<WeaponSkillId> {
        self.ids().find(|(_, s)| s.id == name).map(|(id, _)| id)
    }

    /// The skill of an implicit role ([`Role::is_implicit`]): bare hands, throws, shields.
    pub fn of_role(&self, role: Role) -> Option<WeaponSkillId> {
        self.ids().find(|(_, s)| s.role == role).map(|(id, _)| id)
    }

    /// How marks are earned; `None` without skills.
    pub fn marks(&self) -> Option<&MarkRules> {
        self.marks.as_ref()
    }

    /// The curves by name.
    pub fn curves(&self) -> &BTreeMap<String, [u32; MAX_RANK as usize]> {
        &self.curves
    }

    /// The penalty between `a` and `b`, if one defaults to the other (the same both ways).
    pub fn penalty(&self, a: WeaponSkillId, b: WeaponSkillId) -> Option<u8> {
        self.defaults.get(&(a.min(b), a.max(b))).copied()
    }

    /// The skills `a` defaults to, with their penalties, in id order.
    pub fn defaults_of(&self, a: WeaponSkillId) -> Vec<(WeaponSkillId, u8)> {
        let mut out: Vec<(WeaponSkillId, u8)> = (self.defaults.iter())
            .filter_map(|(&(x, y), &p)| match (x == a, y == a) {
                (true, _) => Some((y, p)),
                (_, true) => Some((x, p)),
                _ => None,
            })
            .collect();
        out.sort_unstable();
        out
    }

    /// What rank `rank` (capped at [`MAX_RANK`]) of skill `id` gives.
    pub fn benefit(&self, id: WeaponSkillId, rank: u8) -> Benefit {
        self.skills
            .get(usize::from(id.0))
            .map_or_else(Benefit::default, |s| {
                s.benefits[usize::from(rank.min(MAX_RANK))]
            })
    }

    /// The marks skill `id` needs to go from `rank` to the next; `None` at [`MAX_RANK`].
    pub fn need(&self, id: WeaponSkillId, rank: u8) -> Option<u32> {
        let skill = self.skills.get(usize::from(id.0))?;
        skill.need.get(usize::from(rank)).copied()
    }

    /// Starting ranks as a race, a class, or a creature gives them (`{ long_blades = 2,
    /// shield_block = "1d2" }`), in skill order: every skill known, every result a roll can
    /// have within 0 to [`MAX_RANK`].
    pub(super) fn starting_ranks(
        &self,
        data: &BTreeMap<String, Dice>,
    ) -> Result<Vec<(WeaponSkillId, Dice)>, String> {
        let mut ranks = Vec::new();
        for (name, &dice) in data {
            let id = self.id(name).ok_or_else(|| {
                format!("unknown weapon skill `{name}` (data/weapon_skills.toml)")
            })?;
            if dice.min() < 0 || dice.max() > i32::from(MAX_RANK) {
                return Err(format!(
                    "weapon skill `{name}`: {dice} must give a rank from 0 to {MAX_RANK}"
                ));
            }
            ranks.push((id, dice));
        }
        ranks.sort_by_key(|&(id, _)| id);
        Ok(ranks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MARKS: &str = "[marks]\nattempt = 1\nhit = 2\nper_level = 20\nmost = 200\n\
                         [curves]\nmelee = [15, 30, 60, 120, 240, 480, 960, 1920, 3840, 7680]\n";

    fn skill(id: &str, role: &str, extra: &str) -> String {
        format!(
            "[[skill]]\nid = \"{id}\"\nname = \"{id}\"\nphrase = \"with {id}\"\nrole = \
             \"{role}\"\ncurve = \"melee\"\n{extra}\n"
        )
    }

    fn parse(text: &str) -> Result<WeaponSkillTable, String> {
        WeaponSkillTable::parse("w", text).map_err(|e| e.message)
    }

    #[test]
    fn the_builtin_table_loads() {
        let text = include_str!("../../../../assets/data/weapon_skills.toml");
        let table = parse(text).expect("valid");
        assert_eq!(table.skills().len(), 16);
        let id = |name| table.id(name).expect(name);
        // The owner's example: long blades 8 → short blades 6.
        assert_eq!(
            table.penalty(id("long_blades"), id("short_blades")),
            Some(2)
        );
        assert_eq!(
            table.penalty(id("short_blades"), id("long_blades")),
            Some(2)
        );
        assert_eq!(table.penalty(id("bows"), id("axes")), None);
        for role in Role::ALL.into_iter().filter(|r| r.is_implicit()) {
            assert!(table.of_role(role).is_some(), "{role:?}");
        }
        assert_eq!(table.need(id("long_blades"), 0), Some(15));
        assert_eq!(table.need(id("long_blades"), 9), Some(7680));
        assert_eq!(table.need(id("long_blades"), 10), None);
        assert!(table.need(id("bows"), 0) < table.need(id("long_blades"), 0));
        for (skill, _) in table.ids() {
            assert_eq!(table.benefit(skill, 0), Benefit::default());
        }
    }

    #[test]
    fn marks_scale_with_the_opponent_against_the_rank() {
        let rules = MarkRules {
            attempt: 1,
            hit: 2,
            per_level: 20,
            most: 200,
        };
        assert_eq!(rules.percent(3, 3), 100);
        assert_eq!(rules.percent(5, 3), 140);
        assert_eq!(rules.percent(50, 0), 200, "at most `most`");
        assert_eq!(rules.percent(1, 6), 0, "nothing to learn from the weak");
        assert_eq!(rules.percent(1, 10), 0);
        // A miss earns `attempt`, a hit `attempt + hit`, scaled and rounded halves up.
        assert_eq!(rules.earned(false, 1, 1), 1);
        assert_eq!(rules.earned(true, 1, 1), 3);
        assert_eq!(rules.earned(true, 1, 0), 4, "3 × 120 % = 3.6");
        assert_eq!(rules.earned(true, 1, 3), 2, "3 × 60 % = 1.8");
        assert_eq!(rules.earned(false, 1, 3), 1, "1 × 60 % = 0.6");
        assert_eq!(rules.earned(false, 1, 4), 0, "1 × 40 % = 0.4");
        assert_eq!(rules.earned(true, 1, 6), 0);
        assert_eq!(rules.earned(true, 9, 2), 6, "3 × 200 %, the most");
    }

    #[test]
    fn benefits_add_up_and_savings_round_down() {
        let a = Benefit {
            to_hit: 1,
            threat: 1,
            speed: 10,
            ..Benefit::default()
        };
        let b = Benefit {
            to_hit: 2,
            damage: 1,
            speed: 5,
            ..Benefit::default()
        };
        assert_eq!(
            a.plus(b),
            Benefit {
                to_hit: 3,
                damage: 1,
                threat: 1,
                speed: 15,
                ..Benefit::default()
            }
        );
        assert_eq!(Benefit::saving(1000, 10), 900);
        assert_eq!(Benefit::saving(1000, 0), 1000);
        assert_eq!(Benefit::saving(1001, 15), 850);
        assert_eq!(Benefit::saving(1, 50), 1, "never free");
    }

    #[test]
    fn defaults_are_symmetric_and_listed_both_ways() {
        let text = format!(
            "{MARKS}{}{}{}[[default]]\nbetween = [\"a\", \"c\"]\npenalty = 3\n\
             [[default]]\nbetween = [\"b\", \"a\"]\npenalty = 2\n",
            skill("a", "melee", ""),
            skill("b", "melee", ""),
            skill("c", "melee", "")
        );
        let table = parse(&text).expect("valid");
        let (a, b, c) = (WeaponSkillId(0), WeaponSkillId(1), WeaponSkillId(2));
        assert_eq!(table.defaults_of(a), [(b, 2), (c, 3)]);
        assert_eq!(table.defaults_of(b), [(a, 2)]);
        assert_eq!(table.penalty(c, a), Some(3));
        assert_eq!(table.penalty(b, c), None);
    }

    #[test]
    fn bad_tables_are_refused() {
        let ok = skill("a", "melee", "");
        let eleven = "[0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5]";
        for (text, why) in [
            (MARKS.to_string(), "missing field `skill`"),
            (
                format!("{MARKS}{}", skill("a", "melee", "to_hit = [0, 1]")),
                "`to_hit` needs 11 numbers",
            ),
            (
                format!(
                    "{MARKS}{}",
                    skill("a", "melee", "to_hit = [1, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5]")
                ),
                "rank 0 is untrained",
            ),
            (
                format!(
                    "{MARKS}{}",
                    skill("a", "melee", "damage = [0, 2, 1, 2, 2, 3, 3, 4, 4, 5, 5]")
                ),
                "never gives less",
            ),
            (
                format!(
                    "{MARKS}{}",
                    skill("a", "melee", &format!("shield = {eleven}"))
                ),
                "`shield` does nothing for a skill of role `melee`",
            ),
            (
                format!(
                    "{MARKS}{}",
                    skill("a", "shield_block", &format!("to_hit = {eleven}"))
                ),
                "`to_hit` does nothing",
            ),
            (
                format!(
                    "{MARKS}{}{}",
                    skill("a", "unarmed", ""),
                    skill("b", "unarmed", "")
                ),
                "already a skill of role `unarmed`",
            ),
            (format!("{MARKS}{ok}{ok}"), "duplicate skill id `a`"),
            (
                format!(
                    "{MARKS}{}",
                    ok.replace("curve = \"melee\"", "curve = \"great\"")
                ),
                "unknown curve `great`",
            ),
            (
                format!("{MARKS}{ok}[[default]]\nbetween = [\"a\", \"z\"]\npenalty = 2\n"),
                "unknown skill",
            ),
            (
                format!("{MARKS}{ok}[[default]]\nbetween = [\"a\", \"a\"]\npenalty = 2\n"),
                "and itself",
            ),
            (
                format!(
                    "{MARKS}{ok}{}[[default]]\nbetween = [\"a\", \"b\"]\npenalty = 2\n\
                     [[default]]\nbetween = [\"b\", \"a\"]\npenalty = 3\n",
                    skill("b", "melee", "")
                ),
                "given twice",
            ),
            (
                format!(
                    "{MARKS}{ok}{}[[default]]\nbetween = [\"a\", \"b\"]\npenalty = 0\n",
                    skill("b", "melee", "")
                ),
                "`penalty` must be 1 to 10",
            ),
            (
                MARKS.replace("15, 30", "30, 15") + &ok,
                "each rank needs more marks",
            ),
            (MARKS.replace("most = 200", "most = 50") + &ok, "`most`"),
        ] {
            let e = parse(&text).unwrap_err();
            assert!(e.contains(why), "{why}: {e}");
        }
    }
}

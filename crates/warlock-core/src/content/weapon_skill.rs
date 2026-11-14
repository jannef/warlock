//! Weapon skills (`assets/data/weapon_skills.toml`, spec §8 "Weapon skills"): the categories of
//! weapons an actor trains with, ADOM-like;
//! and the fighting styles (spec §8 "Fighting styles"), after AD&D 2e's *Complete Fighter's
//! Handbook*: skills of how weapons are held and used.
//!
//! - **Skills** come in file order, each with an `id`, a `name` for the weapon skills screen, a
//!   `phrase` for the message of a rank gained ("You feel more skilled *with long blades*."), a
//!   [`Role`] (what uses it), the need `curve` it trains along, and its benefits per rank, 0 to
//!   [`MAX_RANK`] ([`Benefit`]). Rank 0 is untrained: no benefits.
//! - **Fighting styles** are skills of role [`Role::Style`]: the same ranks, marks, curve, and
//!   benefits, and a [`Style`]: the [`Grip`] they are fought in (what the hands hold for a blow,
//!   or the kind of attack) and the weapon skills they are fought with. Fighting with two
//!   weapons has its penalties here too ([`TwoWeapons`]), and so has fighting with two shields
//!   ([`TwoShields`]).
//! - **Marks** ([`MarkRules`]): every attempt earns some, a success more, scaled by how the
//!   opponent's level compares with the rank. A rank is gained when its marks reach the need of
//!   the next rank, which grows along an exponential **curve** (`curves.melee`: 15, 30, 60, ...).
//!
//! The numbers are rules, never shown in the game: the player sees ranks, and a marks bar.

use super::{ContentError, SpellMods};
use crate::dice::Dice;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The highest rank; rank 0 is untrained.
pub const MAX_RANK: u8 = 10;
/// How many ranks there are, 0 to [`MAX_RANK`].
pub const RANKS: usize = MAX_RANK as usize + 1;

/// Index of a skill in [`WeaponSkillTable::skills`], in file order.
///
/// Ids are assigned in file order when content loads; saves record the content fingerprint to
/// catch a reordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WeaponSkillId(pub u8);

/// What uses a skill: a weapon that names it (`melee`, `launcher`, `thrown`), an attack or a
/// a defence that implies it (`unarmed`, `thrown`, `shield_bash`; at most one skill each), or a
/// way of fighting (`style`).
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
    /// Bashing with a held shield (the bash attack).
    ShieldBash,
    /// A fighting style (spec §8 "Fighting styles"): every attack made in its configuration
    /// ([`Style`]) trains it, and its benefits add to the weapon skill's.
    Style,
}

impl Role {
    pub const ALL: [Role; 6] = [
        Role::Melee,
        Role::Launcher,
        Role::Thrown,
        Role::Unarmed,
        Role::ShieldBash,
        Role::Style,
    ];

    /// Its id in the data file: `shield_bash`.
    pub const fn id(self) -> &'static str {
        match self {
            Role::Melee => "melee",
            Role::Launcher => "launcher",
            Role::Thrown => "thrown",
            Role::Unarmed => "unarmed",
            Role::ShieldBash => "shield_bash",
            Role::Style => "style",
        }
    }

    /// The role of id `id` in the data file.
    fn from_id(id: &str) -> Result<Role, String> {
        (Role::ALL.into_iter().find(|r| r.id() == id)).ok_or_else(|| format!("unknown role `{id}`"))
    }

    /// Whether no weapon names it and it is used by an attack or a defence of its own, so there
    /// is at most one skill of it.
    pub const fn is_implicit(self) -> bool {
        !matches!(self, Role::Melee | Role::Launcher | Role::Style)
    }

    /// The section of the weapon skills screen it is listed in.
    pub const fn section(self) -> Section {
        match self {
            Role::Melee | Role::Unarmed => Section::Melee,
            Role::Launcher | Role::Thrown => Section::Missile,
            Role::ShieldBash => Section::Shields,
            Role::Style => Section::Styles,
        }
    }
}

/// The sections of the weapon skills screen, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section {
    Melee,
    Missile,
    Shields,
    Styles,
}

impl Section {
    pub const ALL: [Section; 4] = [
        Section::Melee,
        Section::Missile,
        Section::Shields,
        Section::Styles,
    ];

    /// `Missile and thrown`.
    pub const fn name(self) -> &'static str {
        match self {
            Section::Melee => "Melee",
            Section::Missile => "Missile and thrown",
            Section::Shields => "Shields",
            Section::Styles => "Fighting styles",
        }
    }
}

/// How a fighting style is fought (spec §8 "Fighting styles"): for blows, what the hands hold
/// (which holds for Defence too); else the kind of attack.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grip {
    /// One one-handed weapon in a hand, every other hand empty (a duelist).
    OneWeapon,
    /// A weapon in each of two hands: both strike (two-weapon fighting).
    TwoWeapons,
    /// A two-handed weapon.
    TwoHanded,
    /// A one-handed weapon and a shield.
    WeaponAndShield,
    /// A shield in each of two hands: every blow is a shield bash (two-shield fighting, #611).
    TwoShields,
    /// Nothing in the hands.
    Unarmed,
    /// Shooting a missile weapon.
    Launcher,
    /// Throwing.
    Thrown,
    /// A spell that rolls to hit at range: a ray or a bolt (spec §8 "Spells").
    RangedSpell,
    /// A touch spell that rolls to hit.
    TouchSpell,
    /// A spell that rolls to hit, ray, bolt, or touch, cast in armour that hinders casting
    /// (anything worn with a `spell_cost_percent`): it comes before the other two.
    ArmouredSpell,
}

impl Grip {
    pub const ALL: &[Grip] = &[
        Grip::OneWeapon,
        Grip::TwoWeapons,
        Grip::TwoHanded,
        Grip::WeaponAndShield,
        Grip::TwoShields,
        Grip::Unarmed,
        Grip::Launcher,
        Grip::Thrown,
        Grip::RangedSpell,
        Grip::TouchSpell,
        Grip::ArmouredSpell,
    ];

    /// Its id in the data file: `weapon_and_shield`.
    pub const fn id(self) -> &'static str {
        match self {
            Grip::OneWeapon => "one_weapon",
            Grip::TwoWeapons => "two_weapons",
            Grip::TwoHanded => "two_handed",
            Grip::WeaponAndShield => "weapon_and_shield",
            Grip::TwoShields => "two_shields",
            Grip::Unarmed => "unarmed",
            Grip::Launcher => "launcher",
            Grip::Thrown => "thrown",
            Grip::RangedSpell => "ranged_spell",
            Grip::TouchSpell => "touch_spell",
            Grip::ArmouredSpell => "armoured_spell",
        }
    }

    /// Whether it is where a spell is cast from (a caster's style; the spell scaling design
    /// §10.2): from afar, up close, or in armour that hinders casting.
    pub const fn casts(self) -> bool {
        matches!(
            self,
            Grip::RangedSpell | Grip::TouchSpell | Grip::ArmouredSpell
        )
    }

    /// Whether a weapon is wielded in it (a Parry benefit adds to the wielded weapon's).
    pub const fn wields(self) -> bool {
        matches!(
            self,
            Grip::OneWeapon | Grip::TwoWeapons | Grip::TwoHanded | Grip::WeaponAndShield
        )
    }

    /// The roles of the weapon skills a style of it may be fought with (`weapons`): none for
    /// bare hands.
    const fn weapon_roles(self) -> &'static [Role] {
        match self {
            Grip::Unarmed
            | Grip::TwoShields
            | Grip::RangedSpell
            | Grip::TouchSpell
            | Grip::ArmouredSpell => &[],
            Grip::Launcher => &[Role::Launcher],
            Grip::Thrown => &[Role::Melee, Role::Thrown],
            _ => &[Role::Melee],
        }
    }
}

/// A fighting style's configuration: the grip it is fought in, and the weapon skills it is
/// fought with. Empty `weapons`: any weapon of its grip that no other style of the grip names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Style {
    pub grip: Grip,
    pub weapons: Vec<WeaponSkillId>,
}

/// Fighting with a weapon in each hand (spec §8 "Fighting styles"): both hands strike each
/// melee attack, the main hand's blow with `main` to hit and the off hand's with `off`. A style
/// eases them (`ease_main`, `ease_off`), never past 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TwoWeapons {
    pub main: i32,
    pub off: i32,
}

impl TwoWeapons {
    /// D&D's: −4 for the main hand, −8 for a light weapon in the off hand.
    pub const DEFAULT: TwoWeapons = TwoWeapons { main: -4, off: -8 };

    /// `penalty` (`main` or `off`) eased by `ease`, never past 0.
    pub fn eased(penalty: i32, ease: u8) -> i32 {
        (penalty + i32::from(ease)).min(0)
    }
}

impl Default for TwoWeapons {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Fighting with a shield in each hand (spec §8 "Fighting styles", #611): every blow is a
/// shield bash, at `to_hit` (a penalty), each attack `slow` percent dearer, and a spell cast
/// fails by a chance of `spell_failure` percent. The style eases each (`ease_bash`,
/// `ease_slow`, `ease_spell`), never past nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TwoShields {
    pub to_hit: i32,
    pub slow: u8,
    pub spell_failure: u8,
}

impl TwoShields {
    pub const DEFAULT: TwoShields = TwoShields {
        to_hit: -6,
        slow: 50,
        spell_failure: 75,
    };

    /// A percent bane (`slow`, `spell_failure`) eased by `ease` points, never below 0.
    pub fn eased(bane: u8, ease: u32) -> u32 {
        u32::from(bane).saturating_sub(ease)
    }
}

impl Default for TwoShields {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Spells that roll to hit (spec §8 "Fighting styles": Sniper): how far is far.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellAttacks {
    /// From this many cells away (the larger of the two distances) a ranged spell's far
    /// benefits (`far_to_hit`, `far_threat`) apply.
    pub far: u8,
}

impl SpellAttacks {
    pub const DEFAULT: SpellAttacks = SpellAttacks { far: 5 };
}

impl Default for SpellAttacks {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// What a rank in a skill gives (rules only, never shown). Benefits of several skills add up
/// ([`Benefit::plus`]): a fighting style's stack on a weapon's.
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
    /// Added to a confirmed critical's damage multiplier (never past ×5).
    pub crit: u8,
    /// Percent off the attack's cost (`melee`, `shoot`, `throw`).
    pub speed: u8,
    /// Percent off the cost of reloading (`reload`).
    pub reload: u8,
    /// Points off the main hand's penalty for fighting with two weapons ([`TwoWeapons`]).
    pub ease_main: u8,
    /// Points off the off hand's penalty for fighting with two weapons.
    pub ease_off: u8,
    /// The chance in percent of a shield bash after a blow that hits (weapon and shield; with
    /// two shields, the other shield's).
    pub bash: u8,
    /// Points off the to-hit penalty of fighting with two shields ([`TwoShields`]).
    pub ease_bash: u8,
    /// Percent points off the slowness of fighting with two shields.
    pub ease_slow: u8,
    /// Percent points off the chance a spell fails with two shields held.
    pub ease_spell: u8,
    /// Added to the to-hit of a ranged spell at a target [`SpellAttacks::far`] or more away.
    pub far_to_hit: i32,
    /// Added to the threat widening of a ranged spell at such a target.
    pub far_threat: u8,
    /// Percent off what worn armour adds to every spell's mana cost (`spell_cost_percent`).
    pub armour: u8,
    /// A Toucher's spells (a touch that rolls to hit): added to every roll (the dice hook).
    pub bonus: i32,
    /// A Sniper's spells (a ray or a bolt that rolls to hit): cells farther (the reach hook).
    pub cells: i32,
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
            crit: self.crit.saturating_add(other.crit),
            speed: self.speed.saturating_add(other.speed),
            reload: self.reload.saturating_add(other.reload),
            ease_main: self.ease_main.saturating_add(other.ease_main),
            ease_off: self.ease_off.saturating_add(other.ease_off),
            bash: self.bash.saturating_add(other.bash).min(100),
            ease_bash: self.ease_bash.saturating_add(other.ease_bash),
            ease_slow: self.ease_slow.saturating_add(other.ease_slow),
            ease_spell: self.ease_spell.saturating_add(other.ease_spell),
            far_to_hit: self.far_to_hit + other.far_to_hit,
            far_threat: self.far_threat.saturating_add(other.far_threat),
            armour: self.armour.saturating_add(other.armour).min(100),
            bonus: self.bonus + other.bonus,
            cells: self.cells + other.cells,
        }
    }

    /// What it does to a spell through the spell hooks (`content/spell_mod.rs`; a caster
    /// style's rank): `bonus` and `cells` one to one. Its `to_hit` and `threat` are the attack
    /// roll's.
    pub fn spell_mods(&self) -> SpellMods {
        SpellMods {
            bonus: self.bonus,
            cells: self.cells,
            ..SpellMods::default()
        }
    }

    /// This benefit for a spell at a target `distance` cells away: the far benefits added at
    /// [`SpellAttacks::far`] or more.
    pub fn at_distance(self, distance: i32, spells: SpellAttacks) -> Benefit {
        if distance < i32::from(spells.far) {
            return self;
        }
        Benefit {
            to_hit: self.to_hit + self.far_to_hit,
            threat: self.threat.saturating_add(self.far_threat),
            ..self
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
/// and 100 ([`MOST_PERCENT`]), rounded to the nearest mark (halves up): fewer against weak
/// opponents and as the rank rises, never more than the full marks against strong ones (#1095).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkRules {
    pub attempt: u32,
    pub hit: u32,
    pub per_level: u32,
}

/// The most percent of its base marks a use earns, however strong the opponent (#1095).
pub const MOST_PERCENT: u32 = 100;

impl MarkRules {
    /// The percent of the base marks a use earns against an opponent of `level` at `rank`.
    pub fn percent(&self, level: u32, rank: u8) -> u32 {
        let difference = i64::from(level) - i64::from(rank);
        let percent = 100 + i64::from(self.per_level) * difference;
        percent.clamp(0, i64::from(MOST_PERCENT)) as u32
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
    /// A fighting style's configuration (`Some` exactly for [`Role::Style`]).
    pub style: Option<Style>,
    /// The curve it trains along (a key of the file's `curves`).
    pub curve: String,
    /// Marks to go from each rank to the next: `need[r]` takes rank `r` to `r + 1`.
    pub need: [u32; MAX_RANK as usize],
    /// What each rank gives, 0 (nothing) to [`MAX_RANK`].
    pub benefits: [Benefit; RANKS],
    /// Listed (the weapon skills screen, the dumps, the talents that ask for it) only once its
    /// owner has marks or a rank in it (`hidden = true`: Two Shields, #611).
    pub hidden: bool,
}

impl WeaponSkillDef {
    /// Whether it is a fighting style.
    pub fn is_style(&self) -> bool {
        self.role == Role::Style
    }
}

/// Which skills starting ranks may name: weapon skills (races, classes), fighting styles
/// (classes' `styles`), or either (creatures).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Ranked {
    Weapons,
    Styles,
    Any,
}

/// Every skill, and how marks are earned.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WeaponSkillTable {
    skills: Vec<WeaponSkillDef>,
    marks: Option<MarkRules>,
    /// The curves by name, as the file gives them.
    curves: BTreeMap<String, [u32; MAX_RANK as usize]>,
    two_weapons: TwoWeapons,
    two_shields: TwoShields,
    spell_attacks: SpellAttacks,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillFile {
    marks: MarkRules,
    curves: BTreeMap<String, Vec<u32>>,
    #[serde(default)]
    two_weapons: TwoWeapons,
    #[serde(default)]
    two_shields: TwoShields,
    #[serde(default)]
    spell_attacks: SpellAttacks,
    skill: Vec<SkillSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillSpec {
    id: String,
    name: String,
    phrase: String,
    role: String,
    grip: Option<Grip>,
    weapons: Option<Vec<String>>,
    curve: String,
    to_hit: Option<Vec<i32>>,
    damage: Option<Vec<i32>>,
    parry: Option<Vec<i32>>,
    shield: Option<Vec<i32>>,
    threat: Option<Vec<i32>>,
    crit: Option<Vec<i32>>,
    speed: Option<Vec<i32>>,
    reload: Option<Vec<i32>>,
    ease_main: Option<Vec<i32>>,
    ease_off: Option<Vec<i32>>,
    bash: Option<Vec<i32>>,
    ease_bash: Option<Vec<i32>>,
    ease_slow: Option<Vec<i32>>,
    ease_spell: Option<Vec<i32>>,
    far_to_hit: Option<Vec<i32>>,
    far_threat: Option<Vec<i32>>,
    armour: Option<Vec<i32>>,
    bonus: Option<Vec<i32>>,
    cells: Option<Vec<i32>>,
    #[serde(default)]
    hidden: bool,
}

/// One benefit's ranks as the file gives them: [`RANKS`] numbers, one per rank 0 to
/// [`MAX_RANK`].
fn ranks(field: &str, given: &Option<Vec<i32>>) -> Result<[i32; RANKS], String> {
    let Some(values) = given else {
        return Ok([0; RANKS]);
    };
    values.clone().try_into().map_err(|v: Vec<i32>| {
        format!(
            "`{field}` needs {RANKS} numbers, one per rank 0 to {MAX_RANK}, not {}",
            v.len()
        )
    })
}

/// [`ranks`] of a benefit held in a `u8` (a percent, a roll, points): each 0 to 255.
fn small_ranks(field: &str, given: &Option<Vec<i32>>) -> Result<[u8; RANKS], String> {
    let values = ranks(field, given)?;
    let mut small = [0; RANKS];
    for (to, &v) in small.iter_mut().zip(&values) {
        *to = u8::try_from(v).map_err(|_| format!("`{field}`: each rank's value is 0 to 255"))?;
    }
    Ok(small)
}

impl SkillSpec {
    /// The benefits per rank of a skill of `role`; a benefit the role (a style: its grip) does
    /// not use is refused.
    fn benefits(&self, role: Role) -> Result<[Benefit; RANKS], String> {
        let to_hit = ranks("to_hit", &self.to_hit)?;
        let damage = ranks("damage", &self.damage)?;
        let parry = ranks("parry", &self.parry)?;
        let shield = ranks("shield", &self.shield)?;
        let threat = small_ranks("threat", &self.threat)?;
        let crit = small_ranks("crit", &self.crit)?;
        let speed = small_ranks("speed", &self.speed)?;
        let reload = small_ranks("reload", &self.reload)?;
        let ease_main = small_ranks("ease_main", &self.ease_main)?;
        let ease_off = small_ranks("ease_off", &self.ease_off)?;
        let bash = small_ranks("bash", &self.bash)?;
        let ease_bash = small_ranks("ease_bash", &self.ease_bash)?;
        let ease_slow = small_ranks("ease_slow", &self.ease_slow)?;
        let ease_spell = small_ranks("ease_spell", &self.ease_spell)?;
        let far_to_hit = ranks("far_to_hit", &self.far_to_hit)?;
        let far_threat = small_ranks("far_threat", &self.far_threat)?;
        let armour = small_ranks("armour", &self.armour)?;
        let spell_bonus = ranks("bonus", &self.bonus)?;
        let cells = ranks("cells", &self.cells)?;
        let grip = self.grip;
        let is = |g: Grip| grip == Some(g);
        // A caster style never quickens a cast (a caster grip).
        let casts = grip.is_some_and(Grip::casts);
        let unused = [
            ("speed", &self.speed, !casts),
            (
                "parry",
                &self.parry,
                role == Role::Melee || grip.is_some_and(Grip::wields),
            ),
            (
                "shield",
                &self.shield,
                is(Grip::WeaponAndShield) || is(Grip::TwoShields),
            ),
            (
                "reload",
                &self.reload,
                role == Role::Launcher || is(Grip::Launcher),
            ),
            ("ease_main", &self.ease_main, is(Grip::TwoWeapons)),
            ("ease_off", &self.ease_off, is(Grip::TwoWeapons)),
            (
                "bash",
                &self.bash,
                is(Grip::WeaponAndShield) || is(Grip::TwoShields),
            ),
            ("ease_bash", &self.ease_bash, is(Grip::TwoShields)),
            ("ease_slow", &self.ease_slow, is(Grip::TwoShields)),
            ("ease_spell", &self.ease_spell, is(Grip::TwoShields)),
            ("far_to_hit", &self.far_to_hit, is(Grip::RangedSpell)),
            ("far_threat", &self.far_threat, is(Grip::RangedSpell)),
            ("armour", &self.armour, is(Grip::ArmouredSpell)),
            // The caster styles: a ray's or a bolt's reach (Sniper), a touch's roll (Toucher).
            ("bonus", &self.bonus, is(Grip::TouchSpell)),
            ("cells", &self.cells, is(Grip::RangedSpell)),
        ];
        if let Some((field, ..)) = unused
            .iter()
            .find(|(_, given, used)| given.is_some() && !used)
        {
            return Err(match grip {
                Some(grip) => format!("`{field}` does nothing for a style of grip `{}`", grip.id()),
                None => format!("`{field}` does nothing for a skill of role `{}`", role.id()),
            });
        }
        Ok(std::array::from_fn(|r| Benefit {
            to_hit: to_hit[r],
            damage: damage[r],
            parry: parry[r],
            shield: shield[r],
            threat: threat[r],
            crit: crit[r],
            speed: speed[r],
            reload: reload[r],
            ease_main: ease_main[r],
            ease_off: ease_off[r],
            bash: bash[r],
            ease_bash: ease_bash[r],
            ease_slow: ease_slow[r],
            ease_spell: ease_spell[r],
            far_to_hit: far_to_hit[r],
            far_threat: far_threat[r],
            armour: armour[r],
            bonus: spell_bonus[r],
            cells: cells[r],
        }))
    }
}

impl WeaponSkillTable {
    /// Parses `data/weapon_skills.toml`.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: SkillFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let marks = parsed.marks;
        // `MarkRules::earned` multiplies a use's marks by a percent (at most 100) in a `u32`.
        let most = (u32::MAX - 50) / MOST_PERCENT;
        if marks
            .attempt
            .checked_add(marks.hit)
            .is_none_or(|m| m > most)
        {
            return Err(err(format!(
                "marks: `attempt` and `hit` together must be at most {most}"
            )));
        }
        let two = parsed.two_weapons;
        let shields = parsed.two_shields;
        let mut curves = BTreeMap::new();
        for (name, need) in parsed.curves {
            let need: [u32; MAX_RANK as usize] = need.try_into().map_err(|v: Vec<u32>| {
                err(format!(
                    "curve `{name}` needs {MAX_RANK} numbers, the marks for ranks 1 to \
                     {MAX_RANK}, not {}",
                    v.len()
                ))
            })?;
            // `WeaponSkills::train` keeps the marks left over below the next rank's need.
            if need.contains(&0) {
                return Err(err(format!(
                    "curve `{name}`: each rank needs 1 mark or more"
                )));
            }
            curves.insert(name, need);
        }
        if parsed.skill.len() > usize::from(u8::MAX) {
            return Err(err("too many skills".into()));
        }
        let mut skills: Vec<WeaponSkillDef> = Vec::new();
        // Each style's weapons by name, resolved once every skill is known.
        let mut weapons: Vec<Vec<String>> = Vec::new();
        for spec in parsed.skill {
            let id = &spec.id;
            let skill_err = |m: String| err(format!("skill `{id}`: {m}"));
            if skills.iter().any(|s| &s.id == id) {
                return Err(err(format!("duplicate skill id `{id}`")));
            }
            let role = Role::from_id(&spec.role).map_err(skill_err)?;
            if role.is_implicit() && skills.iter().any(|s| s.role == role) {
                return Err(skill_err(format!(
                    "there is already a skill of role `{}`, and it can only have one",
                    role.id()
                )));
            }
            let style = match (role, spec.grip) {
                (Role::Style, Some(grip)) => Some(Style {
                    grip,
                    weapons: Vec::new(),
                }),
                (Role::Style, None) => {
                    return Err(skill_err("a fighting style needs its `grip`".into()));
                }
                (_, Some(_)) => {
                    return Err(skill_err(
                        "`grip` is for fighting styles (`role = \"style\"`)".into(),
                    ));
                }
                (_, None) if spec.weapons.is_some() => {
                    return Err(skill_err(
                        "`weapons` is for fighting styles (`role = \"style\"`)".into(),
                    ));
                }
                (_, None) => None,
            };
            let need = *curves
                .get(&spec.curve)
                .ok_or_else(|| skill_err(format!("unknown curve `{}`", spec.curve)))?;
            let benefits = spec.benefits(role).map_err(skill_err)?;
            let hidden = spec.hidden;
            weapons.push(spec.weapons.unwrap_or_default());
            skills.push(WeaponSkillDef {
                id: spec.id,
                name: spec.name,
                phrase: spec.phrase,
                role,
                style,
                curve: spec.curve,
                need,
                benefits,
                hidden,
            });
        }
        let mut table = Self {
            skills,
            marks: Some(marks),
            curves,
            two_weapons: two,
            two_shields: shields,
            spell_attacks: parsed.spell_attacks,
        };
        table.resolve_styles(&weapons).map_err(err)?;
        Ok(table)
    }

    /// Resolves each style's `weapons` (by name, one list per skill in file order): weapon
    /// skills of the roles its grip fights with, each once. Within a grip, at most one style
    /// names none (it takes the rest), and no weapon skill is named by two.
    fn resolve_styles(&mut self, weapons: &[Vec<String>]) -> Result<(), String> {
        for (i, names) in weapons.iter().enumerate() {
            let Some(grip) = self.skills[i].style.as_ref().map(|s| s.grip) else {
                continue;
            };
            let style = self.skills[i].id.clone();
            let mut ids = Vec::new();
            for name in names {
                let id = (self.id(name)).ok_or_else(|| {
                    format!("style `{style}`: `weapons`: unknown weapon skill `{name}`")
                })?;
                if !grip.weapon_roles().contains(&self.skill(id).role) {
                    return Err(format!(
                        "style `{style}`: `weapons`: `{name}` is no weapon skill a style of \
                         grip `{}` is fought with",
                        grip.id()
                    ));
                }
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
            for other in &self.skills[..i] {
                let Some(theirs) = other.style.as_ref().filter(|s| s.grip == grip) else {
                    continue;
                };
                let clash = if ids.is_empty() {
                    theirs.weapons.is_empty()
                } else {
                    ids.iter().any(|id| theirs.weapons.contains(id))
                };
                if clash {
                    return Err(format!(
                        "styles `{}` and `{style}` are both fought with the same weapons in \
                         grip `{}`: one configuration, one style",
                        other.id,
                        grip.id()
                    ));
                }
            }
            if let Some(s) = self.skills[i].style.as_mut() {
                s.weapons = ids;
            }
        }
        Ok(())
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

    /// Every fighting style with its id and configuration, in file order.
    pub fn styles(&self) -> impl Iterator<Item = (WeaponSkillId, &WeaponSkillDef, &Style)> {
        self.ids()
            .filter_map(|(id, s)| s.style.as_ref().map(|style| (id, s, style)))
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

    /// The fighting style of `grip` fought with weapons of skills `weapons` (those in the hands,
    /// the launcher's, what is thrown): the style of the grip that names every one of them,
    /// else the one that names none.
    pub fn style_for(&self, grip: Grip, weapons: &[WeaponSkillId]) -> Option<WeaponSkillId> {
        let of_grip = || self.styles().filter(move |(_, _, s)| s.grip == grip);
        let named = of_grip().find(|(_, _, s)| {
            !weapons.is_empty()
                && !s.weapons.is_empty()
                && weapons.iter().all(|w| s.weapons.contains(w))
        });
        (named.or_else(|| of_grip().find(|(_, _, s)| s.weapons.is_empty()))).map(|(id, ..)| id)
    }

    /// The penalties for fighting with two weapons.
    pub fn two_weapons(&self) -> TwoWeapons {
        self.two_weapons
    }

    /// The banes of fighting with two shields (`[two_shields]`, else [`TwoShields::DEFAULT`]).
    pub fn two_shields(&self) -> TwoShields {
        self.two_shields
    }

    /// How far is far for a ranged spell's far benefits.
    pub fn spell_attacks(&self) -> SpellAttacks {
        self.spell_attacks
    }

    /// How marks are earned; `None` without skills.
    pub fn marks(&self) -> Option<&MarkRules> {
        self.marks.as_ref()
    }

    /// The curves by name.
    pub fn curves(&self) -> &BTreeMap<String, [u32; MAX_RANK as usize]> {
        &self.curves
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
    /// shield_bash = "1d2" }`), in skill order: every skill known and of the kind `ranked`
    /// allows (a roll is kept within 0 to [`MAX_RANK`] where it is used).
    pub(super) fn starting_ranks(
        &self,
        data: &BTreeMap<String, Dice>,
        ranked: Ranked,
    ) -> Result<Vec<(WeaponSkillId, Dice)>, String> {
        let mut ranks = Vec::new();
        for (name, &dice) in data {
            let id = self.id(name).ok_or_else(|| {
                format!("unknown weapon skill `{name}` (data/weapon_skills.toml)")
            })?;
            match (ranked, self.skill(id).is_style()) {
                (Ranked::Weapons, true) => {
                    return Err(format!(
                        "`{name}` is a fighting style: classes give styles in `styles`"
                    ));
                }
                (Ranked::Styles, false) => {
                    return Err(format!(
                        "`{name}` is a weapon skill, not a fighting style: give it in \
                         `weapon_skills`"
                    ));
                }
                _ => {}
            }
            ranks.push((id, dice));
        }
        ranks.sort_by_key(|&(id, _)| id);
        Ok(ranks)
    }
}

impl super::RulesFile for WeaponSkillTable {
    const PATH: &'static str = "data/weapon_skills.toml";

    fn parse(path: &str, text: &str, _content: &super::Content) -> Result<Self, ContentError> {
        Self::parse(path, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::CRITS;

    const MARKS: &str = "[marks]\nattempt = 1\nhit = 2\nper_level = 20\n\
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
        // Fifteen weapon skills and twelve fighting styles.
        assert_eq!(table.skills().len(), 27);
        assert_eq!(table.styles().count(), 12);
        let id = |name: &str| table.id(name).expect(name);
        for role in Role::ALL.into_iter().filter(|r| r.is_implicit()) {
            assert!(table.of_role(role).is_some(), "{role:?}");
        }
        assert_eq!(table.need(id("long_blades"), 0), Some(15));
        assert_eq!(table.need(id("long_blades"), 9), Some(88000));
        assert_eq!(table.need(id("long_blades"), 10), None);
        assert!(table.need(id("bows"), 0) < table.need(id("long_blades"), 0));
        for (skill, _) in table.ids() {
            assert_eq!(table.benefit(skill, 0), Benefit::default());
        }
        // Every configuration is one style: the staff its own, any other two-handed weapon a
        // great weapon, every launcher a marksman's.
        let style = |grip, weapons: &[&str]| {
            let weapons: Vec<WeaponSkillId> = weapons.iter().map(|w| id(w)).collect();
            table
                .style_for(grip, &weapons)
                .map(|s| table.skill(s).id.as_str())
        };
        assert_eq!(style(Grip::OneWeapon, &["daggers"]), Some("duelist"));
        assert_eq!(
            style(Grip::TwoWeapons, &["spears", "daggers"]),
            Some("twin_blades")
        );
        assert_eq!(style(Grip::TwoHanded, &["staves"]), Some("staff_fighter"));
        assert_eq!(
            style(Grip::TwoHanded, &["great_blades"]),
            Some("great_weapon")
        );
        assert_eq!(
            style(Grip::WeaponAndShield, &["maces"]),
            Some("sword_and_board")
        );
        assert_eq!(style(Grip::Unarmed, &[]), Some("brawler"));
        for launcher in ["bows", "crossbows", "slings"] {
            assert_eq!(style(Grip::Launcher, &[launcher]), Some("marksman"));
        }
        assert_eq!(style(Grip::Thrown, &["thrown"]), Some("skirmisher"));
        assert_eq!(style(Grip::Thrown, &["daggers"]), Some("skirmisher"));
        assert_eq!(table.two_weapons(), TwoWeapons { main: -4, off: -8 });
        // Twin Blades 10 eases both penalties away: a full attack with each hand.
        let twin = table.benefit(id("twin_blades"), MAX_RANK);
        assert_eq!(TwoWeapons::eased(-4, twin.ease_main), 0);
        assert_eq!(TwoWeapons::eased(-8, twin.ease_off), 0);
    }

    /// Rank 10 of every weapon skill holds its family's values (docs/design/combat-progression.md
    /// §3: light blades, precise, heavy, middling, launchers and thrown), every
    /// style's §5's, and every benefit of every skill never falls as the rank rises.
    #[test]
    fn rank_ten_holds_the_family_tables() {
        let text = include_str!("../../../../assets/data/weapon_skills.toml");
        let table = parse(text).expect("valid");
        // (id, to_hit, damage, threat, speed, parry, shield, reload)
        #[allow(clippy::type_complexity)]
        #[rustfmt::skip]
        let skills: [(&str, i32, i32, u8, u8, i32, i32, u8); 15] = [
            // Light blades
            ("daggers", 7, 5, 3, 30, 4, 0, 0),
            ("short_blades", 7, 5, 3, 25, 5, 0, 0),
            // Precise
            ("long_blades", 7, 8, 2, 0, 6, 0, 0),
            ("spears", 7, 8, 2, 0, 4, 0, 0),
            ("unarmed", 7, 8, 2, 30, 0, 0, 0),
            // Heavy
            ("great_blades", 5, 12, 2, 0, 4, 0, 0),
            ("axes", 5, 12, 2, 0, 0, 0, 0),
            ("polearms", 5, 12, 2, 0, 0, 0, 0),
            // Middling
            ("maces", 6, 8, 1, 0, 0, 0, 0),
            ("staves", 6, 5, 1, 0, 6, 0, 0),
            ("shield_bash", 6, 5, 1, 0, 0, 0, 0),
            // Launchers and thrown
            ("bows", 7, 5, 2, 25, 0, 0, 0),
            ("crossbows", 6, 5, 1, 0, 0, 0, 60),
            ("slings", 7, 5, 1, 25, 0, 0, 0),
            ("thrown", 7, 5, 2, 30, 0, 0, 0),
        ];
        for (name, to_hit, damage, threat, speed, parry, shield, reload) in skills {
            let id = table.id(name).expect(name);
            let b = table.benefit(id, MAX_RANK);
            assert_eq!(
                (
                    b.to_hit, b.damage, b.threat, b.speed, b.parry, b.shield, b.reload
                ),
                (to_hit, damage, threat, speed, parry, shield, reload),
                "{name}"
            );
        }
        let style = |name: &str| table.benefit(table.id(name).expect(name), MAX_RANK);
        assert_eq!((style("duelist").to_hit, style("duelist").parry), (5, 7));
        let twin = style("twin_blades");
        assert_eq!((twin.to_hit, twin.ease_main, twin.ease_off), (2, 4, 8));
        // Twin Blades is penalty-free a rank early, at 9.
        let twin9 = table.benefit(table.id("twin_blades").unwrap(), MAX_RANK - 1);
        assert_eq!(TwoWeapons::eased(-4, twin9.ease_main), 0);
        assert_eq!(TwoWeapons::eased(-8, twin9.ease_off), 0);
        let great = style("great_weapon");
        assert_eq!((great.damage, great.speed), (10, 20));
        // ×4 criticals at Great Weapon 10: the least multiplier and two more.
        assert_eq!(*CRITS.start() + great.crit, 4);
        let board = style("sword_and_board");
        assert_eq!((board.shield, board.bash), (6, 35));
        // Two Shields (#611): its banes are gone at rank 9, a rank before mastery.
        let shields = style("two_shields");
        assert_eq!((shields.shield, shields.damage, shields.bash), (7, 5, 40));
        let nine = table.benefit(table.id("two_shields").unwrap(), MAX_RANK - 1);
        let banes = table.two_shields();
        assert_eq!(banes.to_hit + i32::from(nine.ease_bash), 0);
        assert_eq!(TwoShields::eased(banes.slow, nine.ease_slow.into()), 0);
        assert_eq!(
            TwoShields::eased(banes.spell_failure, nine.ease_spell.into()),
            0
        );
        let eight = table.benefit(table.id("two_shields").unwrap(), MAX_RANK - 2);
        assert!(TwoShields::eased(banes.spell_failure, eight.ease_spell.into()) > 0);
        let marksman = style("marksman");
        assert_eq!(
            (marksman.to_hit, marksman.speed, marksman.reload),
            (6, 25, 40)
        );
        let skirmisher = style("skirmisher");
        assert_eq!(
            (skirmisher.to_hit, skirmisher.damage, skirmisher.speed),
            (4, 5, 30)
        );
        assert_eq!((style("brawler").to_hit, style("brawler").damage), (5, 10));
        let staff = style("staff_fighter");
        assert_eq!((staff.to_hit, staff.parry), (5, 7));
    }

    #[test]
    fn styles_are_checked() {
        let style = |id: &str, grip: &str, extra: &str| {
            format!(
                "[[skill]]\nid = \"{id}\"\nname = \"{id}\"\nphrase = \"at {id}\"\nrole = \
                 \"style\"\ngrip = \"{grip}\"\ncurve = \"melee\"\n{extra}\n"
            )
        };
        let eleven = "[0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5]";
        let staves = skill("staves", "melee", "");
        let bows = skill("bows", "launcher", "");
        // A style of a grip that names no weapons takes every weapon the others do not.
        let text = format!(
            "{MARKS}{staves}{}{}{}",
            skill("great_blades", "melee", ""),
            style("great", "two_handed", ""),
            style("staff", "two_handed", "weapons = [\"staves\"]")
        );
        let table = parse(&text).expect("valid");
        let (staves_id, great_id) = (WeaponSkillId(0), WeaponSkillId(1));
        assert_eq!(
            table.style_for(Grip::TwoHanded, &[staves_id]),
            Some(WeaponSkillId(3))
        );
        assert_eq!(
            table.style_for(Grip::TwoHanded, &[great_id]),
            Some(WeaponSkillId(2))
        );
        assert_eq!(table.style_for(Grip::OneWeapon, &[great_id]), None);
        assert_eq!(table.two_weapons(), TwoWeapons::DEFAULT, "the default");
        for (text, why) in [
            (
                format!(
                    "{MARKS}{}",
                    style("a", "one_weapon", "").replace("grip = \"one_weapon\"\n", "")
                ),
                "a fighting style needs its `grip`",
            ),
            (
                format!("{MARKS}{}", skill("a", "melee", "grip = \"one_weapon\"")),
                "`grip` is for fighting styles",
            ),
            (
                format!("{MARKS}{}", skill("a", "melee", "weapons = []")),
                "`weapons` is for fighting styles",
            ),
            (
                format!(
                    "{MARKS}{}",
                    style("a", "one_weapon", "weapons = [\"spears\"]")
                ),
                "unknown weapon skill `spears`",
            ),
            (
                format!(
                    "{MARKS}{bows}{}",
                    style("a", "one_weapon", "weapons = [\"bows\"]")
                ),
                "`bows` is no weapon skill a style of grip `one_weapon` is fought with",
            ),
            (
                format!(
                    "{MARKS}{staves}{}",
                    style("a", "unarmed", "weapons = [\"staves\"]")
                ),
                "is no weapon skill a style of grip `unarmed`",
            ),
            (
                format!(
                    "{MARKS}{}{}",
                    style("a", "one_weapon", ""),
                    style("b", "one_weapon", "")
                ),
                "styles `a` and `b` are both fought with the same weapons",
            ),
            (
                format!(
                    "{MARKS}{staves}{}{}",
                    style("a", "two_handed", "weapons = [\"staves\"]"),
                    style("b", "two_handed", "weapons = [\"staves\"]")
                ),
                "one configuration, one style",
            ),
            (
                format!(
                    "{MARKS}{}",
                    style("a", "unarmed", &format!("parry = {eleven}"))
                ),
                "`parry` does nothing for a style of grip `unarmed`",
            ),
            (
                format!(
                    "{MARKS}{}",
                    style("a", "one_weapon", &format!("shield = {eleven}"))
                ),
                "`shield` does nothing for a style of grip `one_weapon`",
            ),
            (
                format!(
                    "{MARKS}{}",
                    style("a", "one_weapon", &format!("ease_off = {eleven}"))
                ),
                "`ease_off` does nothing",
            ),
            (
                format!(
                    "{MARKS}{}",
                    style("a", "two_handed", &format!("bash = {eleven}"))
                ),
                "`bash` does nothing",
            ),
            (
                format!(
                    "{MARKS}{}",
                    style("a", "thrown", &format!("reload = {eleven}"))
                ),
                "`reload` does nothing",
            ),
            (
                format!(
                    "{MARKS}{}",
                    skill("a", "melee", &format!("ease_main = {eleven}"))
                ),
                "`ease_main` does nothing for a skill of role `melee`",
            ),
            (
                format!(
                    "{MARKS}{}",
                    style("a", "weapon_and_shield", &format!("ease_spell = {eleven}"))
                ),
                "`ease_spell` does nothing for a style of grip `weapon_and_shield`",
            ),
        ] {
            let e = parse(&text).unwrap_err();
            assert!(e.contains(why), "{why}: {e}");
        }
    }

    #[test]
    fn marks_scale_with_the_opponent_against_the_rank() {
        let rules = MarkRules {
            attempt: 1,
            hit: 2,
            per_level: 20,
        };
        assert_eq!(rules.percent(3, 3), 100);
        assert_eq!(rules.percent(2, 3), 80);
        // A stronger opponent never speeds training (#1095): the full marks at most.
        assert_eq!(rules.percent(5, 3), 100);
        assert_eq!(rules.percent(50, 0), 100);
        assert_eq!(rules.percent(70, 10), 100);
        assert_eq!(rules.percent(1, 6), 0, "nothing to learn from the weak");
        assert_eq!(rules.percent(1, 10), 0);
        // A miss earns `attempt`, a hit `attempt + hit`, scaled and rounded halves up.
        assert_eq!(rules.earned(false, 1, 1), 1);
        assert_eq!(rules.earned(true, 1, 1), 3);
        assert_eq!(rules.earned(true, 1, 0), 3, "3 × 100 %, not 120 %");
        assert_eq!(rules.earned(true, 1, 3), 2, "3 × 60 % = 1.8");
        assert_eq!(rules.earned(false, 1, 3), 1, "1 × 60 % = 0.6");
        assert_eq!(rules.earned(false, 1, 4), 0, "1 × 40 % = 0.4");
        assert_eq!(rules.earned(true, 1, 6), 0);
        assert_eq!(rules.earned(true, 9, 2), 3, "3 × 100 %, the most");
        // Never above the base marks, whatever the levels.
        for level in 0..=80 {
            for rank in 0..=MAX_RANK {
                assert!(rules.percent(level, rank) <= MOST_PERCENT);
                assert!(rules.earned(true, level, rank) <= rules.attempt + rules.hit);
            }
        }
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
                    skill("a", "melee", &format!("shield = {eleven}"))
                ),
                "`shield` does nothing for a skill of role `melee`",
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
                format!("{MARKS}{ok}[[default]]\nbetween = [\"a\", \"b\"]\npenalty = 2\n"),
                "unknown field `default`",
            ),
            (
                MARKS.replace("15, 30", "0, 30") + &ok,
                "each rank needs 1 mark or more",
            ),
            (
                format!(
                    "{MARKS}{}",
                    skill(
                        "a",
                        "launcher",
                        "reload = [0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 256]"
                    )
                ),
                "`reload`: each rank's value is 0 to 255",
            ),
            (
                MARKS.replace("attempt = 1", "attempt = 4294967295") + &ok,
                "`attempt` and `hit` together must be at most",
            ),
            (
                MARKS.replace("per_level = 20", "per_level = 20\nmost = 200") + &ok,
                "unknown field `most`",
            ),
        ] {
            let e = parse(&text).unwrap_err();
            assert!(e.contains(why), "{why}: {e}");
        }
    }

    #[test]
    fn the_caster_ladder_loads_and_is_checked() {
        let text = include_str!("../../../../assets/data/weapon_skills.toml");
        let table = parse(text).expect("valid");
        let id = |name: &str| table.id(name).expect(name);
        // One configuration, one caster style.
        let style = |grip| {
            table
                .style_for(grip, &[])
                .map(|s| table.skill(s).id.as_str())
        };
        assert_eq!(style(Grip::RangedSpell), Some("sniper"));
        assert_eq!(style(Grip::TouchSpell), Some("toucher"));
        assert_eq!(style(Grip::ArmouredSpell), Some("battle_caster"));
        // Sniper's far benefits join at `far` cells and more.
        let sniper = table.benefit(id("sniper"), MAX_RANK);
        let far = table.spell_attacks();
        assert_eq!(far.far, 5);
        assert_eq!(sniper.at_distance(4, far), sniper);
        let away = sniper.at_distance(5, far);
        assert_eq!((away.to_hit, away.threat), (7, 2));
        assert_eq!(table.benefit(id("battle_caster"), MAX_RANK).armour, 50);
        let caster = |id: &str, grip: &str, extra: &str| {
            format!(
                "[[skill]]\nid = \"{id}\"\nname = \"{id}\"\nphrase = \"at {id}\"\nrole = \
                 \"style\"\ngrip = \"{grip}\"\ncurve = \"melee\"\n{extra}\n"
            )
        };
        let eleven = "[0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5]";
        let three = "[0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 3]";
        // A caster style's spell fields are its grip's; a weapon skill takes none of them.
        for (text, why) in [
            (
                format!(
                    "{MARKS}{}",
                    skill("a", "melee", &format!("bonus = {three}"))
                ),
                "`bonus` does nothing for a skill of role `melee`",
            ),
            (
                format!(
                    "{MARKS}{}",
                    caster("s", "ranged_spell", &format!("mana = {three}"))
                ),
                "unknown field `mana`",
            ),
            (
                format!(
                    "{MARKS}{}",
                    caster("s", "touch_spell", &format!("speed = {eleven}"))
                ),
                "`speed` does nothing",
            ),
            (
                format!(
                    "{MARKS}{}",
                    caster("s", "touch_spell", &format!("far_to_hit = {eleven}"))
                ),
                "`far_to_hit` does nothing for a style of grip `touch_spell`",
            ),
            (
                format!(
                    "{MARKS}{}",
                    caster("s", "ranged_spell", &format!("armour = {eleven}"))
                ),
                "`armour` does nothing",
            ),
            // A style's reach and roll (the spell scaling design §10.2): from afar and up close
            // only, never another grip's.
            (
                format!(
                    "{MARKS}{}",
                    caster("s", "touch_spell", &format!("cells = {three}"))
                ),
                "`cells` does nothing for a style of grip `touch_spell`",
            ),
            (
                format!(
                    "{MARKS}{}",
                    caster("s", "ranged_spell", &format!("bonus = {three}"))
                ),
                "`bonus` does nothing for a style of grip `ranged_spell`",
            ),
            (
                format!(
                    "{MARKS}{}",
                    caster("s", "armoured_spell", &format!("cells = {three}"))
                ),
                "`cells` does nothing for a style of grip `armoured_spell`",
            ),
            (
                format!(
                    "{MARKS}{}",
                    caster("s", "one_weapon", &format!("bonus = {three}"))
                ),
                "`bonus` does nothing for a style of grip `one_weapon`",
            ),
            (
                format!(
                    "{MARKS}{}",
                    caster("s", "launcher", &format!("cells = {three}"))
                ),
                "`cells` does nothing for a style of grip `launcher`",
            ),
            (
                format!(
                    "{MARKS}{}",
                    caster("s", "touch_spell", &format!("parry = {eleven}"))
                ),
                "`parry` does nothing",
            ),
            (
                format!(
                    "{MARKS}{}{}",
                    skill("a", "melee", ""),
                    caster("s", "ranged_spell", "weapons = [\"a\"]")
                ),
                "is no weapon skill a style of grip `ranged_spell` is fought with",
            ),
        ] {
            let e = parse(&text).unwrap_err();
            assert!(e.contains(why), "{why}: {e}");
        }
    }
}

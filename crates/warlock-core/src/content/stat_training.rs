//! Stat training (`assets/data/stats.toml`, spec §8 "Stats"; the design is
//! `docs/design/stat-training.md`): uses of the player's stats earn training marks into a signed
//! pile per stat, and at every level gained and every day at 06:00 each stat gets one check,
//! its chance set by its pile against the price of its next point.
//!
//! A use ([`StatUse`]) is reported where it happens (`World::train` for the uses skills share,
//! `World::train_stat` for the rest); each `[[training.use]]` rule for it gives its stat a mark
//! with the rule's chance. A point costs [`StatTraining::price`] marks: `price_base +
//! price_per_point × score`, ramped quadratically past the potential (`ramp_divisor`); a drop
//! pays back [`StatTraining::drop_price`], half (`drop_share`) the base of the point lost. At a
//! check a pile `m` moves its stat with [`StatTraining::odds`]: nothing while `quarter × |m|` is
//! below the price, else `|m|^p / (|m|^p + price^p)` (`p` = `odds_power`).
//!
//! Without the file nothing trains and no level checks a stat ([`StatTraining::default`]).

use super::ContentError;
use crate::stats::Stat;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// What `price_base` and `price_per_point` may be.
pub const PRICES: RangeInclusive<u16> = 0..=1000;
/// What `ramp_divisor`, `quarter`, and `drop_share` may be.
pub const DIVISORS: RangeInclusive<u16> = 1..=20;
/// What `odds_power` may be (the odds' integer arithmetic holds up to 5).
pub const POWERS: RangeInclusive<u8> = 1..=5;
/// The scale of [`StatTraining::odds`]: a chance of `odds / ODDS_SCALE`, drawn as one
/// `below(ODDS_SCALE)`.
pub const ODDS_SCALE: u32 = 65_536;
/// The pile is counted up to this many times the price (99.8 % at the cube); above it the
/// chance is the same.
const PILE_CAP: u64 = 8;
/// The least a stack picked up weighs to train [`StatUse::Hauled`], in grams: 20 kg.
pub const HAUL_GRAMS: u64 = 20_000;

/// A use of the player's stats that may earn a training mark (spec §8 "Stats"), reported at
/// the one place the thing happens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatUse {
    /// An action other than waiting while the load slows the player (Balance's use).
    Burdened,
    /// A melee blow of the player's that hits (a kick, a bash too).
    Blow,
    /// A confirmed critical of the player's in melee.
    Critical,
    /// A blow on the player that their shield blocked.
    Block,
    /// A turn of digging.
    Dig,
    /// A kick at a closed door or a chest (Athletics' use).
    Kick,
    /// A tempering done at an anvil to its end, whatever came of it (Metalcraft's use).
    Forged,
    /// A struggle against a hold, won or not (Strength's and Willpower's both).
    Struggled,
    /// A step through deep water (not over its ice).
    Waded,
    /// A pick-up that lifts a stack of [`HAUL_GRAMS`] or more (a corpse, a load of ore).
    Hauled,
    /// An attack on the player that missed because of their Dexterity or Dodge (Tumble's use).
    Dodge,
    /// A blow on the player that their Parry turned.
    Parry,
    /// A shot or a throw of the player's that hits.
    Shot,
    /// A step within earshot of a creature that could hear it (Stealth's use).
    Sneak,
    /// A sneak attack of the player's that hits (Sneak attack's use).
    SneakAttack,
    /// Climbing out of the water onto ice, or out of a pit.
    Climb,
    /// A try at picking a lock, whatever came of it (Open lock's use).
    Picked,
    /// A try at disarming a trap, whatever came of it (Disable device's use; Dexterity's and
    /// Intelligence's both).
    Disarmed,
    /// A theft attempted, whatever came of it (Pick pocket's use).
    Stole,
    /// An initiative check of the player's made: not surprised.
    KeptWits,
    /// An attack spell of the player's that hits (a ray, a bolt, a touch).
    Aimed,
    /// A blow that costs the player hit points.
    Hurt,
    /// A big hit or a confirmed critical on the player.
    BigHit,
    /// A kill of the player's in melee of a creature of their level or above.
    Killed,
    /// A check against a disease won, catching it or as it progresses (not a Willpower one).
    Resisted,
    /// Falling through the ice.
    Plunge,
    /// A step on the overworld (Survival's use).
    Marched,
    /// An action other than waiting while Hungry (not yet Weak).
    HungryStep,
    /// A status that harms as it ticks (bleeding, poison, ebbing mana) wearing off by time.
    ShookOff,
    /// Stamina spent on a weapon ability.
    Exerted,
    /// A daily stat check made while the player stands below freezing (0 °C).
    Chilled,
    /// A search that finds something (a secret door, a trap).
    Found,
    /// A scroll read.
    Read,
    /// An item identified by reading.
    Identified,
    /// A spell studied from a scroll, whatever comes of it (Spellcraft's use too).
    Study,
    /// The first kill of a creature type.
    Discovered,
    /// A spell the player knows cast (not a scroll's, not a class or sign ability): trains the
    /// stats its `[spell.scales]` name (damage's, else Intelligence; reach's when it has a
    /// reach; duration's when it has a status), the rules of the stats among them.
    Spell,
    /// The first arrival on a level that pays arrival experience.
    Explored,
    /// A carried thing told for what it is by looking it over (Appraise's use).
    Appraised,
    /// A turn of searching that found nothing.
    Searched,
    /// A forage that ran its time, whatever it found (Survival's use).
    Foraged,
    /// Noticing something hidden in passing (Spot's use; a trap noticed so too).
    Spot,
    /// A long action of more than one turn done to its end: a rest, a wall dug through, a
    /// search for a while, a meal.
    Endured,
    /// A Willpower check against a disease won (brain fever).
    ResistedWill,
    /// A blow of the player's that lands while they are Hungry or worse.
    HungryFight,
    /// Mana spent on an ability (`World::spend_mana`).
    Cast,
    /// A prayer done at a forgotten altar.
    Prayed,
    /// A check against a drain of the player's stats made.
    HeldOut,
    /// Recovering one's wits after a surprise: Surprised ending with the player's next action.
    Steadied,
    /// A chat (asking about a topic takes no time, so it trains nothing).
    Chat,
    /// A purchase paid for or a sale made.
    Trade,
    /// A quest completed.
    Quest,
    /// A creature making way for the player.
    Swapped,
    /// Inspire or Throw your voice: a class active that works on others' minds (or one's own).
    Inspired,
    /// A guild rank won.
    Promoted,
    /// A deal made in a shop while Negotiation bettered the price.
    Bargained,
    /// The first chat with someone.
    Greeted,
    /// A potion drunk: names its stat (the potion's), and gives the potion's marks at once.
    Drank,
    /// An elixir drunk: names its stat, and gives the elixir's marks at once.
    Elixir,
}

impl StatUse {
    /// The use a skill use is for stats too, if any: the uses the two share (one report,
    /// `World::train`).
    pub const fn of_skill(used: crate::skills::Use) -> Option<StatUse> {
        use crate::skills::Use;
        match used {
            Use::Burdened => Some(StatUse::Burdened),
            Use::Kick => Some(StatUse::Kick),
            Use::Temper => Some(StatUse::Forged),
            Use::Pick => Some(StatUse::Picked),
            Use::Disarm => Some(StatUse::Disarmed),
            Use::Steal => Some(StatUse::Stole),
            Use::Travel => Some(StatUse::Marched),
            Use::Appraise => Some(StatUse::Appraised),
            Use::Forage => Some(StatUse::Foraged),
            Use::Dodge => Some(StatUse::Dodge),
            Use::Sneak => Some(StatUse::Sneak),
            Use::SneakAttack => Some(StatUse::SneakAttack),
            Use::Spot => Some(StatUse::Spot),
            Use::Study => Some(StatUse::Study),
            _ => None,
        }
    }

    /// Whether the use names the stat it trains where it happens (a potion's), so its rules
    /// have no `stat` of their own; every other use's rules have one.
    pub const fn names_stat(self) -> bool {
        matches!(self, StatUse::Drank | StatUse::Elixir)
    }

    /// How the wiki says it.
    pub const fn describe(self) -> &'static str {
        match self {
            StatUse::Burdened => "acting (not waiting) while your load slows you",
            StatUse::Blow => "a blow of yours that hits up close (a kick or a shield bash too)",
            StatUse::Critical => "a critical of yours up close",
            StatUse::Block => "a blow on you that your shield blocked",
            StatUse::Dig => "a turn of digging",
            StatUse::Kick => "kicking at a closed door or a chest",
            StatUse::Forged => "tempering a weapon or armour at an anvil, whatever comes of it",
            StatUse::Struggled => "struggling against a hold, won or not",
            StatUse::Waded => "a step through deep water",
            StatUse::Hauled => "picking up something of 20 kg or more (a corpse, a heavy load)",
            StatUse::Dodge => {
                "an attack on you that your Dexterity or Dodge made miss, or a known trap you \
                 stepped past"
            }
            StatUse::Parry => "a blow on you that your Parry turned",
            StatUse::Shot => "a shot or a throw of yours that hits",
            StatUse::Sneak => "a step within earshot of a creature that could hear it",
            StatUse::SneakAttack => "a sneak attack of yours that hits",
            StatUse::Climb => "climbing out of the water onto ice, or out of a pit",
            StatUse::Picked => "a try at picking a lock",
            StatUse::Disarmed => "a try at disarming a trap",
            StatUse::Stole => "a theft attempted",
            StatUse::KeptWits => "keeping your wits when something tries to take you by surprise",
            StatUse::Aimed => "an attack spell of yours that hits (a ray, a bolt, a touch)",
            StatUse::Hurt => "a blow that costs you hit points",
            StatUse::BigHit => "a big hit or a critical on you",
            StatUse::Killed => "a kill of yours up close, of a foe of your level or above",
            StatUse::Resisted => "fighting off a disease (a check won, other than a Willpower one)",
            StatUse::Plunge => "falling through the ice",
            StatUse::Marched => "a step on the overworld",
            StatUse::HungryStep => "acting (not waiting) while Hungry, not yet Weak",
            StatUse::ShookOff => {
                "shaking off bleeding, poison, or anything else that harms as it lasts (it wears \
                 off)"
            }
            StatUse::Exerted => "spending stamina on a weapon ability",
            StatUse::Chilled => "dawn (06:00) finding you below freezing",
            StatUse::Found => "finding something hidden by searching",
            StatUse::Read => "reading a scroll",
            StatUse::Identified => "identifying an item",
            StatUse::Study => "studying a spell from a scroll",
            StatUse::Discovered => "your first kill of a kind of creature",
            StatUse::Spell => {
                "casting a spell you know whose damage, reach, or duration grows with this stat \
                 (Intelligence too for one whose damage grows with none)"
            }
            StatUse::Explored => "your first arrival on a level that pays experience",
            StatUse::Appraised => "telling a thing you carry for what it is by looking it over",
            StatUse::Searched => "a turn of searching that finds nothing",
            StatUse::Foraged => "foraging, whatever you find",
            StatUse::Spot => "noticing something hidden in passing",
            StatUse::Endured => {
                "a long action of more than one turn done to its end (a rest, a wall dug \
                 through, a search for a while, a meal)"
            }
            StatUse::ResistedWill => "fighting off a disease with Willpower (brain fever)",
            StatUse::HungryFight => "a blow of yours that lands while you are Hungry or worse",
            StatUse::Cast => "spending mana on an ability",
            StatUse::Prayed => "praying at a forgotten altar",
            StatUse::HeldOut => "holding out against something that drains your stats",
            StatUse::Steadied => "recovering your wits after a surprise",
            StatUse::Chat => "chatting with someone",
            StatUse::Trade => "paying for goods, or selling",
            StatUse::Quest => "completing a quest",
            StatUse::Swapped => "a creature making way for you",
            StatUse::Inspired => "inspiring with song, or throwing your voice",
            StatUse::Promoted => "winning a guild rank",
            StatUse::Bargained => "a deal in a shop where your Negotiation bettered the price",
            StatUse::Greeted => "your first chat with someone",
            StatUse::Drank => "a potion drunk: each of its marks, in the stat it names",
            StatUse::Elixir => "an elixir drunk: each of its marks, in the stat it names",
        }
    }
}

/// When a stat earns a training mark: on each `on`, with `chance` percent (1 to 100). `stat` is
/// `None` exactly for a use that names its own ([`StatUse::names_stat`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatRule {
    #[serde(default)]
    pub stat: Option<Stat>,
    pub on: StatUse,
    pub chance: u8,
}

/// The rules of stat training.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatTraining {
    /// What a point's price starts at: 20.
    pub price_base: u16,
    /// What each point of the score adds to its price: 10.
    pub price_per_point: u16,
    /// The ramp past the potential: the `over`-th point past it costs `(ramp_divisor + over ×
    /// (over + 1)) / ramp_divisor` times the base: 4.
    pub ramp_divisor: u16,
    /// The power of the odds' S-curve: 3.
    pub odds_power: u8,
    /// A pile under `1 / quarter` of the price is out of play: 4.
    pub quarter: u16,
    /// A drop pays back the base price of the point lost divided by this: 2.
    pub drop_share: u16,
    /// The mark rules, in file order (the order the training stream is drawn in).
    #[serde(default, rename = "use")]
    pub uses: Vec<StatRule>,
}

impl Default for StatTraining {
    /// No rules: nothing trains, and no level checks a stat.
    fn default() -> Self {
        Self {
            price_base: 20,
            price_per_point: 10,
            ramp_divisor: 4,
            odds_power: 3,
            quarter: 4,
            drop_share: 2,
            uses: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    training: StatTraining,
}

impl StatTraining {
    /// Parses `data/stats.toml`.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let File { training } = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for (name, value, range) in [
            ("price_base", training.price_base, PRICES),
            ("price_per_point", training.price_per_point, PRICES),
            ("ramp_divisor", training.ramp_divisor, DIVISORS),
            ("quarter", training.quarter, DIVISORS),
            ("drop_share", training.drop_share, DIVISORS),
            (
                "odds_power",
                u16::from(training.odds_power),
                u16::from(*POWERS.start())..=u16::from(*POWERS.end()),
            ),
        ] {
            if !range.contains(&value) {
                return Err(err(format!(
                    "{name} {value} is out of range ({} to {})",
                    range.start(),
                    range.end()
                )));
            }
        }
        if training.price_base == 0 && training.price_per_point == 0 {
            return Err(err(
                "price_base and price_per_point may not both be 0".into()
            ));
        }
        let id = |rule: &StatRule| rule.stat.map_or("any stat", Stat::id);
        for rule in &training.uses {
            if !(1..=100).contains(&rule.chance) {
                return Err(err(format!(
                    "{} on {:?}: chance {} is out of range (1 to 100)",
                    id(rule),
                    rule.on,
                    rule.chance
                )));
            }
            if rule.stat.is_some() == rule.on.names_stat() {
                let want = match rule.on.names_stat() {
                    true => "names its own stat: leave out `stat`",
                    false => "needs a `stat`",
                };
                return Err(err(format!("{:?} {want}", rule.on)));
            }
        }
        for (i, rule) in training.uses.iter().enumerate() {
            if (training.uses[..i].iter()).any(|r| r.stat == rule.stat && r.on == rule.on) {
                return Err(err(format!(
                    "{} on {:?} is listed twice",
                    id(rule),
                    rule.on
                )));
            }
        }
        Ok(training)
    }

    /// Whether anything trains at all: without rules no level checks a stat either.
    pub fn is_active(&self) -> bool {
        !self.uses.is_empty()
    }

    /// The rules for `used`, in file order.
    pub fn rules(&self, used: StatUse) -> impl Iterator<Item = &StatRule> {
        self.uses.iter().filter(move |r| r.on == used)
    }

    /// The plain price of the point after `score`: `price_base + price_per_point × score`
    /// (at least 1).
    pub fn base_price(&self, score: u8) -> u32 {
        (u32::from(self.price_base) + u32::from(self.price_per_point) * u32::from(score)).max(1)
    }

    /// The marks a stat at `score` with `potential` pays for its next point: the base price,
    /// ramped past the potential. With `over = score + 1 − potential` (1 for the first point
    /// past it), `base × (ramp_divisor + over × (over + 1)) / ramp_divisor`; the point reaching
    /// the potential costs the base.
    pub fn price(&self, score: u8, potential: u8) -> u32 {
        let base = self.base_price(score);
        let over = (u32::from(score) + 1).saturating_sub(u32::from(potential));
        if over == 0 {
            return base;
        }
        let ramp = u32::from(self.ramp_divisor);
        (u64::from(base) * u64::from(ramp + over * (over + 1)) / u64::from(ramp)) as u32
    }

    /// The marks a drop from `score` pays back: the base price of the point lost divided by
    /// `drop_share`, never ramped (at least 1).
    pub fn drop_price(&self, score: u8) -> u32 {
        (self.base_price(score.saturating_sub(1)) / u32::from(self.drop_share)).max(1)
    }

    /// The price a pile of `marks` is checked against: the rise's for a positive pile, the
    /// drop's for a negative one, none for zero.
    pub fn price_for(&self, marks: i32, score: u8, potential: u8) -> Option<u32> {
        match marks {
            0 => None,
            1.. => Some(self.price(score, potential)),
            _ => Some(self.drop_price(score)),
        }
    }

    /// The chance a pile of `marks` (its size; the sign is the caller's) moves its stat at a
    /// check against `price`, out of [`ODDS_SCALE`]: 0 while `quarter × marks < price`, else
    /// `marks^p / (marks^p + price^p)` with the pile counted up to eight times the price.
    /// Integers only: both are scaled to the price first (`marks × 65 536 / price` against
    /// 65 536), which keeps every power of [`POWERS`] within `u128`.
    pub fn odds(&self, marks: u32, price: u32) -> u32 {
        let price = u64::from(price.max(1));
        let marks = u64::from(marks);
        if marks * u64::from(self.quarter) < price {
            return 0;
        }
        let scale = u64::from(ODDS_SCALE);
        let m = u128::from(marks.min(PILE_CAP * price) * scale / price);
        let power = u32::from(self.odds_power);
        let num = m.pow(power);
        let den = num + u128::from(scale).pow(power);
        (num * u128::from(scale) / den) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<StatTraining, ContentError> {
        StatTraining::parse("data/stats.toml", text)
    }

    const HEAD: &str = "[training]\nprice_base = 20\nprice_per_point = 10\nramp_divisor = \
                        4\nodds_power = 3\nquarter = 4\ndrop_share = 2\n";

    /// A chance out of [`ODDS_SCALE`] in whole percent, rounded.
    fn percent(odds: u32) -> u32 {
        (odds * 100 + ODDS_SCALE / 2) / ODDS_SCALE
    }

    #[test]
    fn the_price_is_linear_to_the_potential_and_ramps_past_it() {
        let t = parse(HEAD).unwrap();
        assert!(!t.is_active());
        // The design's table (§4.2): the base, then the 1st to 6th point past the potential.
        assert_eq!(t.price(14, 16), 160);
        assert_eq!(
            t.price(13, 14),
            150,
            "the point reaching the potential costs the base"
        );
        let past: Vec<u32> = (1..=6).map(|over| t.price(14, 15 - over)).collect();
        assert_eq!(past, [240, 400, 640, 960, 1360, 1840]);
        let past: Vec<u32> = (1..=6).map(|over| t.price(35, 36 - over)).collect();
        assert_eq!(past, [555, 925, 1480, 2220, 3145, 4255]);
        assert_eq!(t.price(6, 1), 920);
        // The drop: half the base of the point lost, never ramped.
        assert_eq!(t.drop_price(14), 75);
        assert_eq!(t.drop_price(6), 35);
        assert_eq!(t.drop_price(35), 180);
        assert_eq!(t.price_for(0, 14, 16), None);
        assert_eq!(t.price_for(5, 14, 14), Some(240));
        assert_eq!(t.price_for(-5, 14, 14), Some(75));
    }

    #[test]
    fn the_odds_are_the_cube_with_the_quarter_rule() {
        let t = parse(HEAD).unwrap();
        // §4.3's column, at a price of 160.
        let at = |share: u32| percent(t.odds(160 * share / 100, 160));
        assert_eq!(at(24), 0, "under a quarter: out of play");
        assert_eq!(at(25), 2);
        assert_eq!(at(50), 11);
        assert_eq!(at(75), 30);
        assert_eq!(at(100), 50);
        assert_eq!(at(125), 66);
        assert_eq!(at(150), 77);
        assert_eq!(at(200), 89);
        assert_eq!(at(300), 96);
        // Exact at the price, capped at eight times it.
        assert_eq!(t.odds(160, 160), ODDS_SCALE / 2);
        assert_eq!(t.odds(8 * 160, 160), t.odds(100 * 160, 160));
        assert!(t.odds(u32::MAX, 160) < ODDS_SCALE);
        // The worst price the ranges allow still computes.
        let mut worst = t.clone();
        (worst.price_base, worst.price_per_point, worst.ramp_divisor) = (1000, 1000, 1);
        worst.odds_power = *POWERS.end();
        let price = worst.price(40, 1);
        assert!(worst.odds(price, price).abs_diff(ODDS_SCALE / 2) <= 1);
        assert!(worst.odds(u32::MAX, price) < ODDS_SCALE);
    }

    #[test]
    fn rules_are_validated() {
        let rule = |chance: u32| {
            format!(
                "{HEAD}[[training.use]]\nstat = \"strength\"\non = \"blow\"\nchance = {chance}\n"
            )
        };
        let t = parse(&rule(5)).unwrap();
        assert!(t.is_active());
        assert_eq!(t.rules(StatUse::Blow).count(), 1);
        assert_eq!(t.rules(StatUse::Dig).count(), 0);
        assert!(
            parse(&rule(0))
                .unwrap_err()
                .to_string()
                .contains("chance 0")
        );
        assert!(parse(&rule(101)).is_err());
        let twice = format!("{}{}", rule(5), &rule(5)[HEAD.len()..]);
        assert!(parse(&twice).unwrap_err().to_string().contains("twice"));
        let bad = HEAD.replace("divisor = 4", "divisor = 0");
        assert!(
            parse(&bad)
                .unwrap_err()
                .to_string()
                .contains("ramp_divisor")
        );
        let bad = HEAD.replace("odds_power = 3", "odds_power = 6");
        assert!(parse(&bad).unwrap_err().to_string().contains("odds_power"));
        let old = "[training]\nneed_base = 20\nneed_per_point = 3\n";
        assert!(parse(old).is_err(), "the old keys are refused");
        let unknown =
            format!("{HEAD}[[training.use]]\nstat = \"luck\"\non = \"blow\"\nchance = 5\n");
        assert!(parse(&unknown).is_err());
        let statless = format!("{HEAD}[[training.use]]\non = \"blow\"\nchance = 5\n");
        assert!(
            parse(&statless)
                .unwrap_err()
                .to_string()
                .contains("needs a `stat`")
        );
    }

    #[test]
    fn a_potion_use_names_its_own_stat() {
        let drank = format!("{HEAD}[[training.use]]\non = \"drank\"\nchance = 100\n");
        let t = parse(&drank).unwrap();
        assert_eq!(t.rules(StatUse::Drank).next().map(|r| r.stat), Some(None));
        let named = drank.replace("on = ", "stat = \"strength\"\non = ");
        assert!(parse(&named).unwrap_err().to_string().contains("leave out"));
    }
}

//! The tuning numbers of rules that have no data file of their own
//! (`assets/data/rules/tuning.toml`,
//! #1541): check difficulties, ranges, reaches, to-hit penalties, chances, and floors a designer
//! would tune, one table per rule. The rules themselves live in `world/`, each reading its table
//! here; a rule with a data file of its own keeps its numbers there (`data/rules/swim.toml`,
//! `data/traps.toml`, ...).
//!
//! The file is optional: without it (a test's bare content) every rule reads
//! [`Tuning::default`], the numbers the file held when they moved into it. The file is what the
//! game plays by: tune it, not the default.

use super::ContentError;
use serde::Deserialize;

/// The tuning numbers, one field per table of the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tuning {
    /// Wriggling past a creature in the way (`world/levels/wriggle.rs`).
    pub wriggle: Difficulty,
    /// A step onto slick ground (`world/magic/zone.rs`).
    pub slick: Difficulty,
    /// Keeping afloat after a plunge through the ice (`world/levels/ice.rs`).
    pub plunge: Difficulty,
    /// Throwing one's voice (`world/turn/voice.rs`).
    pub throw_voice: Range,
    /// The on-kill affixes (`world/items/siphon.rs`).
    pub siphon: Range,
    /// A Pit's mouth (`world/levels/pit.rs`).
    pub pit: PitRules,
    /// The quick leave (`world/levels/quick_leave.rs`).
    pub quick_leave: QuickLeave,
    /// What stops a long action (`world/turn/rest.rs`).
    pub long_action: LongAction,
    /// Where a creature that comes may appear (`world/creatures/spawn.rs`).
    pub spawn: SpawnRules,
    /// The unseen (`world/creatures/unseen.rs`).
    pub unseen: UnseenRules,
    /// Averted eyes (`world/creatures/avert.rs`).
    pub averted: ToHit,
    /// A flurry's second blow (`world/combat/free_blow.rs`).
    pub flurry: ToHit,
    /// Holding on to a weapon a blow would knock away (`world/combat/knock_away.rs`).
    pub knock_away: KnockAway,
    /// A read's price under talents (`world/magic/read.rs`).
    pub read: Floor,
    /// A weapon active's time under talents (`world/combat/weapon_active.rs`).
    pub weapon_active: Floor,
    /// A listen at a door (`world/skills/listen.rs`).
    pub listen: ListenRules,
    /// The chance a potion drunk is kept (`world/health/potion.rs`).
    pub potion_kept: PotionKept,
}

/// A check's difficulty.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Difficulty {
    /// What the check is against (a creature's in the way adds its Willpower modifier).
    pub difficulty: i32,
}

/// How far an aimed thing reaches, in cells (a disc, as ranges are: Euclidean).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Range {
    pub range: i32,
}

/// A Pit's mouth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PitRules {
    /// How far from the caster the mouth may open, in cells (Chebyshev).
    pub mouth_reach: i32,
}

/// The quick leave.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuickLeave {
    /// How near (Chebyshev) an unseen hostile creature stops it.
    pub reach: i32,
}

/// What stops a long action (a rest, a run, a dig).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LongAction {
    /// How near (Chebyshev) a hostile creature in view must come.
    pub stop_within: i32,
}

/// Where a creature that comes may appear.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnRules {
    /// The fewest cells (Chebyshev) between it and the player.
    pub away: i32,
}

/// The unseen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnseenRules {
    /// What a blow at a creature the player does not see adds to its to-hit.
    pub to_hit: i32,
    /// How near (Chebyshev) an unseen shadowy creature comes before the player feels a draught.
    pub draught: i32,
}

/// A to-hit penalty.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToHit {
    pub to_hit: i32,
}

/// What a grip adds to holding on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnockAway {
    /// Both hands on the hilt.
    pub two_handed: i32,
    /// A shield on the other arm to turn the blow.
    pub shield: i32,
}

/// The least a price may be cut to, in percent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Floor {
    pub least_percent: i32,
}

/// A listen at a door.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListenRules {
    /// What it adds to the hearing threshold's fall, besides the skill's.
    pub keener: u16,
}

/// The chance a potion drunk is kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PotionKept {
    /// The most the worn items raise it to with a talent's, in percent (a talent's own chance
    /// above it stands).
    pub most: u8,
}

/// The numbers for content without the file (bare test content): what
/// `assets/data/rules/tuning.toml` held when they moved into it (#1541).
impl Default for Tuning {
    fn default() -> Self {
        Self {
            wriggle: Difficulty { difficulty: 11 },
            slick: Difficulty { difficulty: 12 },
            plunge: Difficulty { difficulty: 11 },
            throw_voice: Range { range: 6 },
            siphon: Range { range: 3 },
            pit: PitRules { mouth_reach: 3 },
            quick_leave: QuickLeave { reach: 10 },
            long_action: LongAction { stop_within: 5 },
            spawn: SpawnRules { away: 8 },
            unseen: UnseenRules {
                to_hit: -4,
                draught: 2,
            },
            averted: ToHit { to_hit: -4 },
            flurry: ToHit { to_hit: -2 },
            knock_away: KnockAway {
                two_handed: 2,
                shield: 1,
            },
            read: Floor { least_percent: 25 },
            weapon_active: Floor { least_percent: 50 },
            listen: ListenRules { keener: 2 },
            potion_kept: PotionKept { most: 50 },
        }
    }
}

/// The widest a range, reach, or distance of the file may be, in cells.
const MOST_CELLS: i32 = 40;

impl Tuning {
    /// Parses `data/rules/tuning.toml`, each number within its bounds.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let t: Tuning = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let within = |name: &str, v: i32, lo: i32, hi: i32| {
            if (lo..=hi).contains(&v) {
                Ok(())
            } else {
                Err(err(format!("`{name}` must be {lo} to {hi}, not {v}")))
            }
        };
        for (name, d) in [
            ("wriggle.difficulty", t.wriggle),
            ("slick.difficulty", t.slick),
            ("plunge.difficulty", t.plunge),
        ] {
            within(name, d.difficulty, 0, 40)?;
        }
        within("throw_voice.range", t.throw_voice.range, 1, MOST_CELLS)?;
        within("siphon.range", t.siphon.range, 1, MOST_CELLS)?;
        within("pit.mouth_reach", t.pit.mouth_reach, 1, MOST_CELLS)?;
        within("quick_leave.reach", t.quick_leave.reach, 0, MOST_CELLS)?;
        within(
            "long_action.stop_within",
            t.long_action.stop_within,
            0,
            MOST_CELLS,
        )?;
        within("spawn.away", t.spawn.away, 0, MOST_CELLS)?;
        within("unseen.draught", t.unseen.draught, 0, MOST_CELLS)?;
        for (name, v) in [
            ("unseen.to_hit", t.unseen.to_hit),
            ("averted.to_hit", t.averted.to_hit),
            ("flurry.to_hit", t.flurry.to_hit),
        ] {
            within(name, v, -20, 0)?;
        }
        within("knock_away.two_handed", t.knock_away.two_handed, 0, 20)?;
        within("knock_away.shield", t.knock_away.shield, 0, 20)?;
        within("read.least_percent", t.read.least_percent, 1, 100)?;
        let floor = t.weapon_active.least_percent;
        within("weapon_active.least_percent", floor, 1, 100)?;
        within("listen.keener", i32::from(t.listen.keener), 0, 20)?;
        within("potion_kept.most", i32::from(t.potion_kept.most), 0, 100)?;
        Ok(t)
    }
}

impl super::RulesFile for Tuning {
    const PATH: &'static str = "data/rules/tuning.toml";

    fn parse(path: &str, text: &str, _content: &super::Content) -> Result<Self, ContentError> {
        Self::parse(path, text)
    }
}

#[cfg(test)]
mod tests {
    use super::Tuning;
    use crate::content::{Content, ContentFiles};

    const PATH: &str = "data/rules/tuning.toml";

    #[test]
    fn a_number_out_of_bounds_is_refused_and_none_is_the_default() {
        let files = ContentFiles::builtin();
        let text = files.get(PATH).expect("the file").to_string();
        for (from, to, says) in [
            ("range = 6", "range = 0", "throw_voice.range"),
            ("to_hit = -2", "to_hit = 3", "flurry.to_hit"),
            (
                "least_percent = 25",
                "least_percent = 0",
                "read.least_percent",
            ),
        ] {
            let mut files = ContentFiles::builtin();
            files.insert(PATH, &text.replacen(from, to, 1));
            let err = Content::load(&files).expect_err(to);
            assert!(err.message.contains(says), "{}", err.message);
        }
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("the file is optional");
        assert_eq!(*content.tuning(), Tuning::default());
    }
}

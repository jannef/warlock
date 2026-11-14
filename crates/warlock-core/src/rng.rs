//! Deterministic random streams (spec §8 Determinism).
//!
//! Every stream is a ChaCha8 generator (`rand_chacha`; its output is specified, so it is identical
//! on every platform) seeded from a 64-bit key: FNV-1a over the run seed and a stream label
//! ([`stream_key`]). Nothing else makes a key: no stream is drawn from another, so a stream
//! depends only on the run seed and its own label, never on the order streams are made or
//! first used, and adding one moves no other.
//!
//! # Keys (#1662)
//!
//! There is no shared gameplay stream: every rule that rolls in play draws from a stream of its
//! own, keyed by what it decides, so a draw added to or taken from one rule never moves another's
//! rolls (replays, insta snapshots, and golden walks of unrelated features stay as they were).
//! A key is **unique** (no two decisions share one) and **stable** (it names *what* is decided,
//! never how many things came before it). Three forms:
//!
//! - **A moment's draw**, [`crate::draw!`]: `draw!(self, "<rule>/<decision>/<actor key>")` in a
//!   `World` method. Its label names the rule and whom it concerns; the key is the label, the
//!   second of game time, and how many draws of that label the second has seen already
//!   (`<label>@<second>#<n>`, counted by [`Draws`], kept in the state). So the key is unique
//!   (the counter) and stable (the counter is scoped to the one label: another rule's draw in the
//!   same second is another label). Several related dice of one decision (a check and the
//!   degrees it inflicts) come from the one stream; distinct decisions take distinct labels.
//! - **A one-shot stream of an entity**, `Rng::stream(seed, "drop/<actor key>")`: for a decision
//!   an entity meets once (what it drops as it dies, its grade as it is made), keyed by the
//!   entity alone. The entity's key must be unique by construction.
//! - **A kept stream of a subsystem** (`training`, `levels`, `ruin`, `encounters`, a level's
//!   `spawn/<level id>`): sequential within one subsystem, kept in the state. Only that
//!   subsystem's own draws move it.
//!
//! **Actor keys** ([`crate::actor::Actor::key`]): `player`, or for a creature the level it was
//! made on, its cell there, the second, and a counter of the creatures made on that cell in that
//! second (`depths:1/12,5@3600#0`), given as it is made (`world/levels/travel.rs::make_creature`).
//! A generated level's creature is keyed by where its population stream put it, a spawned one by
//! where and when it came: never by `ActorId`, the global creation counter, which renumbers
//! every later creature when one more or fewer is made earlier.
//!
//! Generation, per level (none kept; derived on demand, so a level depends only on the run seed
//! and its id, never on the order levels are visited or on gameplay):
//!
//! | Stream | Label |
//! |---|---|
//! | Map | `map/<level id>` (retry *k*: `map/<level id>/<k>`) |
//! | An endless branch's floor seed | key `endless/<floors made>` (a seed, not a stream) |
//! | Rooms floor or caves on an endless floor | `floor_kind/<level id>` |
//! | A vault's phases | `vault_phases/<vault id>` |
//! | Deep water in a cave | `deep_water/<level id>` |
//! | Ore veins | `veins/<level id>` |
//! | Hidden traps | `traps/<level id>` |
//! | Secret doors | `secrets/<level id>` |
//! | Loot | `loot/<level id>` |
//! | A wand's rung in a level's, a fixed kit's, or a pack's objects | `objects/<level id>`, `objects/actor/<actor key>`, `objects/pack` |
//! | Vault monsters | `vault_monsters/<level id>` |
//! | Initial population (and its leavings) | `population/<level id>` |
//! | A teeming den's sleepers and hoards | `swarm/<level id>` |
//! | Vault chests | `vault_chests/<level id>` |
//! | Chests | `chests/<level id>` |
//! | Treasure rooms | `treasure/<level id>` |
//! | Creatures' gear | `gear/<level id>` |
//! | Animated creatures' bodies | `animated/<level id>` |
//! | Gravestones (and which fallen they name) | `graves/<level id>`, `graves/<level id>/fallen` |
//! | Which petrified dead name its statues | `statue-names/<level id>` |
//! | A Pit's seed | `pit/<integer>` (a seed, not a stream) |
//!
//! Per actor or object (derived on demand, never kept; `<actor key>` as above):
//!
//! | Stream | Label |
//! |---|---|
//! | What a creature drops when it dies | `drop/<actor key>` |
//! | The materials a dead creature leaves | `materials/<actor key>` |
//! | A creature's gear, placed in play | `gear/<actor key>` |
//! | An animated creature's body, placed in play | `animated/<actor key>` |
//! | A creature's court (a fey's faction) | `court/<actor key>` |
//! | A creature's grade | `grade/<actor key>` |
//! | Gear's condition | `condition/<key>` (the gear stream's key) |
//! | A beast's name | `companion/name/<actor key>` |
//! | A trader's wares | `wares/<actor key>` |
//!
//! Kept in [`crate::WorldState`] (saves continue them):
//!
//! | Stream | Label | Kept |
//! |---|---|---|
//! | Training marks' chances (skills) | `training` | always |
//! | What level-ups roll (skill rolls, stat checks) | `levels` | from the first draw |
//! | What an area spell ruins | `ruin` | from the first draw |
//! | Overworld encounters | `encounters` | from the first draw |
//! | What comes to a level with a danger (spec §7 "Danger levels") | `spawn/<level id>` | in the level's state, from its making |
//! | The moment's draws made this second ([`Draws`]) | (counts, not a stream) | always |
//!
//! In play, keyed by the moment (made where used, never kept):
//!
//! | Stream | Label |
//! |---|---|
//! | Every rule's roll in play | `<rule>/<decision>/<actor key>@<second>#<n>` ([`crate::draw!`]; the labels are listed at their sites, `docs/dev/merging.md` "Random streams") |
//! | A rolled quest's draw | `quest/<quest id>` |
//! | Appraising | `appraise/<hour>` |
//! | Whether food is told stale | `freshness/<object>/<died>/<hour>` |
//! | An investment's week | `invest/<week>` |
//! | An aristocrat's stipend | `stipend/<level>` |
//! | A patron's gift | `patron/<taken>` |
//! | Scroll labels | `identification` |
//! | Weather, one-liners' order, mutters | keyed draws (`weather::draw`): `weather`, `weather/start`, `line…`, `mutter…` |
//!
//! A new character (derived on demand, never kept; only the run seed and the choices):
//!
//! | Stream | Label |
//! |---|---|
//! | The stat gender "other" raises | `character/gender` |
//! | Age and birthday | `character/birth/<race>` |
//! | Dice | `character/rolls/<race>,<gender>,<class>` |
//! | Name | `character/name/<race>,<gender>/<draw>` |
//! | Weapon skills, styles, optional skills, a skill choice | `character/weapon-skills/<spec>`, `character/styles/<spec>`, `character/optional-skills/<spec>`, `character/skill-choice/<spec>` |
//! | Random talents | `character/talents/<spec>` |
//! | Starting kit, family wealth, father's sword, trade | `character/kit/<spec>`, `character/family-wealth/<spec>`, `character/fathers-sword/<spec>`, `character/trade/<spec>/<skill>` |
//! | Starting companion, familiar | `companion/<spec>`, `familiar/<spec>` |
//! | Background story, per chapter | `background/<race>,<gender>,<class>/<chapter>` |
//!
//! Visual randomness in the app uses its own generator and never touches these streams. All
//! helpers are integer-only: no floating point decides anything.

use crate::world::{FNV_OFFSET, fnv1a, fnv1a_fold};
use rand_chacha::ChaCha8Rng;
use rand_core::{Rng as _, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{self, Write as _};

/// A deterministic random stream. Serializes as ChaCha's abstract state (seed, stream, position).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rng(ChaCha8Rng);

impl Rng {
    /// The stream `label` of the run with `run_seed`.
    pub fn stream(run_seed: u64, label: &str) -> Self {
        Self::from_key(stream_key(run_seed, label))
    }

    /// The stream of `key`, a [`stream_key`].
    fn from_key(key: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(key))
    }

    /// The training stream of a run: the chances of skills' training marks (spec §8 "Skills"),
    /// kept apart from every rule's draws so training never changes a fight's rolls.
    pub fn training(run_seed: u64) -> Self {
        Self::stream(run_seed, "training")
    }

    /// The levels stream of a run (spec §8 "Talents"): what level-ups roll (skill rolls, stat
    /// checks), apart from the gameplay stream so a level moves no fight, and apart from
    /// training so marks and level-ups never shift each other. The world state keeps it from
    /// the first draw on.
    pub fn levels(run_seed: u64) -> Self {
        Self::stream(run_seed, "levels")
    }

    /// The ruin stream of a run (spec §8 "Spells": "Ruin"): which items lying in an area spell
    /// are lost, apart from the gameplay stream so a pile in a Fireball moves no fight's roll.
    /// The world state keeps it from the first draw on.
    pub fn ruin(run_seed: u64) -> Self {
        Self::stream(run_seed, "ruin")
    }

    /// The map-generation stream for `level`; `attempt` > 0 derives the stream for a retry after
    /// a generated map failed validation.
    pub fn map(run_seed: u64, level: &str, attempt: u32) -> Self {
        if attempt == 0 {
            Self::stream(run_seed, &format!("map/{level}"))
        } else {
            Self::stream(run_seed, &format!("map/{level}/{attempt}"))
        }
    }

    /// The deep water stream of generated level `level` (`mapgen/deep.rs`): whether a cave floor
    /// has a deep pool (one in six) and where, apart from its map's so no other draw moves.
    pub fn deep_water(run_seed: u64, level: &str) -> Self {
        Self::stream(run_seed, &format!("deep_water/{level}"))
    }

    /// The ore vein stream of generated level `level` (`mapgen/vein.rs`, #1390): whether the floor
    /// has veins, of what, and where, apart from every other stream so no other draw moves.
    pub fn veins(run_seed: u64, level: &str) -> Self {
        Self::stream(run_seed, &format!("veins/{level}"))
    }

    /// The stream that hides traps on generated level `level` (spec §8 "Traps"), apart from its
    /// map's, so a level is the same with or without traps.
    pub fn traps(run_seed: u64, level: &str) -> Self {
        Self::stream(run_seed, &format!("traps/{level}"))
    }

    /// The stream that hides doors on generated level `level` (secret doors, spec §8
    /// "Searching"), apart from its map's and its traps', so a level is the same with or without
    /// them.
    pub fn secrets(run_seed: u64, level: &str) -> Self {
        Self::stream(run_seed, &format!("secrets/{level}"))
    }

    /// The loot stream of generated level `level` (spec §8 "Magic items"): what lies on its floor
    /// and in its vaults, rolled once the map is built, so the map is the same without it.
    pub fn loot(run_seed: u64, level: &str) -> Self {
        Self::stream(run_seed, &format!("loot/{level}"))
    }

    /// The objects stream of `key` (the spell scaling design §7.4): a level's id, for the wands
    /// its own piles and chests hold as it is first made (a level file's, a vault's), or
    /// `actor/<id>` for those a creature placed with a fixed kit carries, or `pack` for a
    /// scenario's pack. It draws a wand's rung only, so nothing else moves with it.
    pub fn objects(run_seed: u64, key: &str) -> Self {
        Self::stream(run_seed, &format!("objects/{key}"))
    }

    /// The population stream of generated level `level` (spec §7 "Danger levels"): its initial
    /// creatures, their cells and routes, drawn once its loot is (so the map, traps, secrets, and
    /// loot are the same with any population).
    pub fn population(run_seed: u64, level: &str) -> Self {
        Self::stream(run_seed, &format!("population/{level}"))
    }

    /// The vault monsters stream of generated level `level` (spec §7 "Level file format"): who
    /// its vaults' monster pools bring, their cells and routes, drawn just before its population
    /// (so the map, traps, secrets, and loot are the same with any of them).
    pub fn vault_monsters(run_seed: u64, level: &str) -> Self {
        Self::stream(run_seed, &format!("vault_monsters/{level}"))
    }

    /// The swarm stream of generated level `level` (spec §7 "Special rooms"): its teeming dens'
    /// sleepers, their cells, and their hoards, drawn once its population is (so the rest of the
    /// level is the same with or without a den).
    pub fn swarm(run_seed: u64, level: &str) -> Self {
        Self::stream(run_seed, &format!("swarm/{level}"))
    }

    /// The chest stream of generated level `level` (spec §8 "Chests"): whether its floor gets
    /// chests, their cells, locks, traps, and contents, drawn once its rooms' loot is and before
    /// its gear (so the rest of the level is the same with or without chests).
    pub fn chests(run_seed: u64, level: &str) -> Self {
        Self::stream(run_seed, &format!("chests/{level}"))
    }

    /// The vault chest stream of generated level `level` (spec §8 "Chests"): what the chests its
    /// vaults stamped with nothing in them hold, drawn before its own chests (so the rest of the
    /// level is the same with or without a vault's chests).
    pub fn vault_chests(run_seed: u64, level: &str) -> Self {
        Self::stream(run_seed, &format!("vault_chests/{level}"))
    }

    /// The treasure stream of generated level `level` (spec §7 "Special rooms"): its treasure
    /// rooms' chests, heaps of gold, and gems, drawn once its chests are (so the rest of the level,
    /// its other chests too, is the same with or without a treasure room).
    pub fn treasure(run_seed: u64, level: &str) -> Self {
        Self::stream(run_seed, &format!("treasure/{level}"))
    }

    /// The spawn stream of level `level` (spec §7 "Danger levels"): what comes to it over time,
    /// apart from `gameplay` and its generation streams, so a fight rolls the same with or
    /// without a spawn between two blows. The level's state keeps it.
    pub fn spawn(run_seed: u64, level: &str) -> Self {
        Self::stream(run_seed, &format!("spawn/{level}"))
    }

    /// The gear stream of `key` (spec §8 "Creature gear"): a generated level's id, rolled once
    /// its loot is (so the map, traps, secrets, and loot are the same without it), or an actor's
    /// id for a creature placed in play without a fixed kit.
    pub fn gear(run_seed: u64, key: &str) -> Self {
        Self::stream(run_seed, &format!("gear/{key}"))
    }

    /// The stream of quest `id`'s roll (spec §8 "Quests", #422): used once, as the quest is
    /// given, never kept; only what it drew is saved.
    pub fn quest(run_seed: u64, id: &str) -> Self {
        Self::stream(run_seed, &format!("quest/{id}"))
    }

    /// The stream of chapter `chapter` of the background story of a character of `spec` (spec §8
    /// "Background story"): the tier, the line, and its dice, apart from every other stream, so
    /// no roll of the character moves with or without a story. The spec's `Display` names it
    /// (race, gender, class; never the picks).
    pub fn background(
        run_seed: u64,
        spec: &crate::character::CharacterSpec,
        chapter: &str,
    ) -> Self {
        Self::stream(run_seed, &format!("background/{spec}/{chapter}"))
    }

    /// The court stream of actor `key` (spec §7 "Factions"): the faction a creature of a type
    /// with several draws when it is made, apart from every other stream, so a level is the
    /// same whichever court its fey joined.
    pub fn court(run_seed: u64, key: &str) -> Self {
        Self::stream(run_seed, &format!("court/{key}"))
    }

    /// The grade stream of actor `key` (spec §8 "Grades"; [`crate::grade`]): the
    /// one draw that makes a creature greater or lesser as it is made, apart from every other
    /// stream, so nothing else moves with it.
    pub fn grade(run_seed: u64, key: &str) -> Self {
        Self::stream(run_seed, &format!("grade/{key}"))
    }

    pub fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    /// Uniform in `0..n` without modulo bias (Lemire's multiply-and-reject); 0 when `n` is 0.
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        let threshold = n.wrapping_neg() % n;
        loop {
            let m = u64::from(self.next_u32()) * u64::from(n);
            if (m as u32) >= threshold {
                return (m >> 32) as u32;
            }
        }
    }

    /// Uniform in `lo..=hi`; `lo` when the range is empty.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (i64::from(hi) - i64::from(lo) + 1).min(i64::from(u32::MAX)) as u32;
        (i64::from(lo) + i64::from(self.below(span))) as i32
    }

    /// True with probability `percent` / 100.
    pub fn chance(&mut self, percent: u32) -> bool {
        self.below(100) < percent
    }

    /// A uniformly chosen index into a slice of length `len` (0 when `len` is 0).
    pub fn index(&mut self, len: usize) -> usize {
        self.below(u32::try_from(len).unwrap_or(u32::MAX)) as usize
    }

    /// A uniformly chosen element, or `None` for an empty slice.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        (!items.is_empty()).then(|| &items[self.index(items.len())])
    }

    /// One of `items` drawn by `weight`, the one weighted draw of the core: a single
    /// `below(total)` of the summed weights, then a walk to the item it falls in. `None`, with no
    /// draw made, when they weigh nothing in all or their sum overflows a `u32` (the loaders
    /// refuse such tables: [`weight_sum`]).
    pub fn weighted<'a, T>(&mut self, items: &'a [T], weight: impl Fn(&T) -> u32) -> Option<&'a T> {
        let total = weight_sum(items.iter().map(&weight))?;
        if total == 0 {
            return None;
        }
        let mut left = self.below(total);
        items.iter().find(|item| {
            let w = weight(item);
            let here = left < w;
            left = left.saturating_sub(w);
            here
        })
    }

    /// Fisher–Yates shuffle.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.index(i + 1);
            items.swap(i, j);
        }
    }
}

/// The sum of `weights`, or `None` when it overflows a `u32`: what [`Rng::weighted`] draws
/// below, and what a loader checks a table of weights with (an overflow is a content error).
pub fn weight_sum(weights: impl IntoIterator<Item = u32>) -> Option<u32> {
    weights.into_iter().try_fold(0u32, u32::checked_add)
}

/// A loader's words for a table whose weights overflow ([`weight_sum`]).
pub const WEIGHTS_OVERFLOW: &str = "the weights add up to more than 4294967295";

/// The 64-bit key of stream `label` in the run with `run_seed`: FNV-1a over the seed's
/// little-endian bytes, a `0xff` separator, and the label's UTF-8 bytes.
pub fn stream_key(run_seed: u64, label: &str) -> u64 {
    let mut key = Key::new(run_seed);
    key.bytes(label.as_bytes());
    key.0
}

/// A stream key as it is written (the module docs, "Keys"): FNV-1a over the run seed, the
/// separator, then whatever is written to it, so a label formatted into it is never allocated.
/// The same text written in any pieces gives [`stream_key`]'s value.
struct Key(u64);

impl Key {
    fn new(run_seed: u64) -> Self {
        let mut key = Self(FNV_OFFSET);
        key.bytes(&run_seed.to_le_bytes());
        key.bytes(&[0xff]);
        key
    }

    fn bytes(&mut self, bytes: &[u8]) {
        self.0 = fnv1a_fold(self.0, bytes);
    }
}

impl fmt::Write for Key {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.bytes(s.as_bytes());
        Ok(())
    }
}

/// The moment's draws of a run (the module docs, "Keys"): how many times each label has been
/// drawn in the current second of game time, so the n-th draw of a label in a second takes the
/// key `<label>@<second>#<n>` and no two draws share one. Kept in the world state (saves
/// continue it); it forgets a second as the next begins.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Draws {
    /// The second the counts are of.
    second: u64,
    /// By the label's [`stream_key`], how many draws of it the second has seen.
    counts: BTreeMap<u64, u32>,
    /// How many draws the run has made in all (tests read it to see that a rule drew nothing).
    total: u64,
}

impl Draws {
    /// Whether nothing was drawn and no actor keyed yet (a save leaves it out then).
    pub fn is_empty(&self) -> bool {
        self.total == 0 && self.counts.is_empty()
    }

    /// How many draws the run has made in all.
    pub fn total(&self) -> u64 {
        self.total
    }

    /// The stream the next draw of `label` at `second` would take ([`Draws::keyed`]), without
    /// counting it: what a test rigs a seed by.
    pub fn peek(&self, run_seed: u64, second: u64, label: &str) -> Rng {
        let mut key = Key::new(run_seed);
        key.bytes(label.as_bytes());
        let n = match self.second == second {
            true => self.counts.get(&key.0).copied().unwrap_or(0),
            false => 0,
        };
        write!(key, "@{second}#{n}").expect("hashing never fails");
        Rng::from_key(key.0)
    }

    /// Counts a draw of the label whose key is `label` at `second`: how many came before it in
    /// that second.
    fn count(&mut self, second: u64, label: u64) -> u32 {
        self.total += 1;
        self.tick(second, label)
    }

    /// How many times the key `label` was ticked in `second` before this one (a draw's count, an
    /// actor key's).
    fn tick(&mut self, second: u64, label: u64) -> u32 {
        if self.second != second {
            self.second = second;
            self.counts.clear();
        }
        let n = self.counts.entry(label).or_insert(0);
        let before = *n;
        *n += 1;
        before
    }

    /// The stream of the moment's draw `label` at `second` of the run with `run_seed`: the
    /// stream `<label>@<second>#<n>`, `n` the draws of that label the second saw before this one.
    /// [`crate::rng::draw!`] calls it with the world's seed, clock, and counts.
    pub fn keyed(&mut self, run_seed: u64, second: u64, label: fmt::Arguments<'_>) -> Rng {
        let mut key = Key::new(run_seed);
        key.write_fmt(label).expect("hashing never fails");
        let n = self.count(second, key.0);
        write!(key, "@{second}#{n}").expect("hashing never fails");
        Rng::from_key(key.0)
    }

    /// The key of the actor made on `level` at `pos` in `second`: the level, the cell, the
    /// second, and how many were made on that cell in that second before it (the module docs,
    /// "Actor keys"), `depths:1/12,5@3600#0`.
    pub fn actor_key(
        &mut self,
        run_seed: u64,
        second: u64,
        level: &str,
        pos: crate::Pos,
    ) -> String {
        let mut key = Key::new(run_seed);
        write!(key, "actor/{level}/{},{}", pos.x, pos.y).expect("hashing never fails");
        let n = self.tick(second, key.0);
        format!("{level}/{},{}@{second}#{n}", pos.x, pos.y)
    }
}

/// A moment's draw in a `World` method (the module docs, "Keys"):
/// `draw!(self, "<rule>/<decision>/{}", key)` is the stream `<label>@<second>#<n>` of the run,
/// `n` counted by the state's [`Draws`]. The label's arguments may read the world
/// (`self.state.actors[&id].key`): only the seed, the clock, and the counts are touched.
macro_rules! draw {
    ($world:expr, $($label:tt)*) => {
        $world.state.draws.keyed(
            $world.state.seed,
            $world.state.time.seconds(),
            ::core::format_args!($($label)*),
        )
    };
}
pub(crate) use draw;

/// Parses a `--seed` value: a decimal number is used as is; any other text (a word or phrase) is
/// hashed with FNV-1a 64 over its UTF-8 bytes, so `--seed warlock` always means the same run.
/// Surrounding whitespace is ignored; case matters.
pub fn parse_seed(text: &str) -> Result<u64, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("the seed is empty".into());
    }
    if text.bytes().all(|b| b.is_ascii_digit()) {
        return text
            .parse()
            .map_err(|_| format!("seed `{text}` is larger than {}", u64::MAX));
    }
    Ok(fnv1a(text.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::Rng;
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn streams_are_reproducible_and_distinct() {
        let a: Vec<u32> = {
            let mut r = Rng::map(7, "depths:1", 0);
            (0..8).map(|_| r.next_u32()).collect()
        };
        let again: Vec<u32> = {
            let mut r = Rng::map(7, "depths:1", 0);
            (0..8).map(|_| r.next_u32()).collect()
        };
        assert_eq!(a, again);
        let mut other_level = Rng::map(7, "depths:2", 0);
        let mut retry = Rng::map(7, "depths:1", 1);
        let mut other_seed = Rng::map(8, "depths:1", 0);
        assert_ne!(a[0], other_level.next_u32());
        assert_ne!(a[0], retry.next_u32());
        assert_ne!(a[0], other_seed.next_u32());
        assert_ne!(Rng::stream(7, "gameplay"), Rng::map(7, "gameplay", 0));
    }

    /// The weighted draw is one `below(total)` and a walk, as the hand-written draws it replaced
    /// were (#1594): the same pick from the same stream; nothing weighing anything, or a sum past
    /// a `u32`, draws nothing.
    #[test]
    fn weighted_draws_once_by_weight() {
        let items = [("a", 3), ("b", 0), ("c", 5), ("d", 2)];
        for seed in 0..200 {
            let mut walk = Rng::stream(seed, "gameplay");
            let mut left = walk.below(10);
            let want = items.iter().find(|&&(_, w)| {
                let here = left < w;
                left = left.saturating_sub(w);
                here
            });
            let mut rng = Rng::stream(seed, "gameplay");
            assert_eq!(rng.weighted(&items, |&(_, w)| w), want);
            assert_eq!(rng, walk, "one draw, as the walk's");
        }
        let untouched = Rng::stream(1, "gameplay");
        let mut rng = untouched.clone();
        assert_eq!(rng.weighted(&[("a", 0), ("b", 0)], |&(_, w)| w), None);
        assert_eq!(
            rng.weighted(&[("a", u32::MAX), ("b", 1)], |&(_, w)| w),
            None
        );
        assert_eq!(rng.weighted::<(&str, u32)>(&[], |&(_, w)| w), None);
        assert_eq!(rng, untouched);
        assert_eq!(weight_sum([u32::MAX, 0]), Some(u32::MAX));
        assert_eq!(weight_sum([u32::MAX, 1]), None);
    }

    /// Pins the algorithm: if this changes, every seed produces a different run.
    #[test]
    fn output_is_pinned() {
        assert_eq!(stream_key(1, "gameplay"), 0x6a1b_986d_d3b2_e6a1);
        let mut r = Rng::stream(1, "gameplay");
        let first: Vec<u32> = (0..3).map(|_| r.next_u32()).collect();
        assert_eq!(first, [2_610_789_080, 3_370_076_174, 2_717_488_749]);
    }

    #[test]
    fn seeds_parse_from_numbers_and_words() {
        assert_eq!(parse_seed("42"), Ok(42));
        assert_eq!(parse_seed(" 7 "), Ok(7));
        assert_eq!(parse_seed("warlock"), Ok(fnv1a(b"warlock")));
        assert_ne!(parse_seed("Warlock"), parse_seed("warlock"));
        assert_eq!(parse_seed("-1"), Ok(fnv1a(b"-1")));
        assert!(parse_seed("").is_err());
        assert!(parse_seed("99999999999999999999").is_err());
    }

    #[test]
    fn serde_round_trip_continues_the_stream() {
        let mut r = Rng::stream(3, "gameplay");
        for _ in 0..5 {
            r.next_u32();
        }
        let json = serde_json::to_string(&r).unwrap();
        let mut back: Rng = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
        assert_eq!(back.next_u32(), r.next_u32());
        let via_value: Rng = serde_json::from_value(serde_json::to_value(&r).unwrap()).unwrap();
        assert_eq!(via_value, r);
    }

    /// A moment's draws: the same label in one second counts up and gives distinct streams, the
    /// next second starts over, a peek is the next draw, and the counts survive a save.
    #[test]
    fn keyed_draws_count_within_a_second_and_round_trip() {
        let mut draws = Draws::default();
        assert!(draws.is_empty());
        let first = draws.keyed(7, 100, format_args!("fight/strike/player/{}", "k"));
        let second = draws.keyed(7, 100, format_args!("fight/strike/player/{}", "k"));
        assert_ne!(first, second, "the same label twice in a second: two keys");
        assert_eq!(first, Rng::stream(7, "fight/strike/player/k@100#0"));
        assert_eq!(second, Rng::stream(7, "fight/strike/player/k@100#1"));
        let other = draws.keyed(7, 100, format_args!("fight/bash/player"));
        assert_eq!(
            other,
            Rng::stream(7, "fight/bash/player@100#0"),
            "its own count"
        );
        assert_eq!(
            draws.peek(7, 100, "fight/strike/player/k"),
            Rng::stream(7, "fight/strike/player/k@100#2")
        );
        assert_eq!(
            draws.peek(7, 101, "fight/strike/player/k"),
            Rng::stream(7, "fight/strike/player/k@101#0")
        );
        let later = draws.keyed(7, 101, format_args!("fight/strike/player/k"));
        assert_eq!(later, Rng::stream(7, "fight/strike/player/k@101#0"));
        assert_eq!(draws.total(), 4);
        let key = draws.actor_key(7, 101, "depths:1", crate::Pos::new(12, 5));
        assert_eq!(key, "depths:1/12,5@101#0");
        assert_eq!(
            draws.actor_key(7, 101, "depths:1", crate::Pos::new(12, 5)),
            "depths:1/12,5@101#1"
        );
        assert_eq!(draws.total(), 4, "an actor's key is no draw");
        let json = serde_json::to_string(&draws).unwrap();
        let back: Draws = serde_json::from_str(&json).unwrap();
        assert_eq!(back, draws);
        let via_value: Draws =
            serde_json::from_value(serde_json::to_value(&draws).unwrap()).unwrap();
        assert_eq!(via_value, draws);
    }

    proptest! {
        #[test]
        fn helpers_stay_in_range(seed: u64, n in 1u32..1000, lo in -50i32..50, span in 0i32..50) {
            let mut r = Rng::stream(seed, "test");
            prop_assert!(r.below(n) < n);
            let x = r.range(lo, lo + span);
            prop_assert!((lo..=lo + span).contains(&x));
            let mut items: Vec<u32> = (0..span as u32).collect();
            r.shuffle(&mut items);
            items.sort_unstable();
            prop_assert_eq!(items, (0..span as u32).collect::<Vec<_>>());
        }
    }
}

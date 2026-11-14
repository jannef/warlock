//! The senses talents (#1495): what a talent does to the holder's senses: hearing, sight,
//! darkvision, scent, and the lights it sheds. [`SensesEffect`] is the data (a talent's
//! `[[effect]]` of one of [`SensesEffect::KINDS`]), [`SensesMods`] the sum every rule reads
//! (`TalentMods::senses`). A new effect of the area is a variant here, its field and its arm in
//! [`SensesMods::add`] here, its wiki text in `xtask/src/wiki/talents/senses.rs`, and its one read.

use crate::content::{Content, LightId, StatusId};
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SensesEffect {
    /// The actor hears sounds this much fainter (its hearing threshold, lower).
    Hearing { value: i32 },
    /// It places the footsteps it hears exactly, at the cell they came from, never roughly
    /// (Watchful).
    ExactFootsteps,
    /// Its eyes need this percent more light to see by (negative: less; Night eyes), on the
    /// vision profile's threshold ([`crate::world::WorldState::eyes`]).
    ThresholdPercent { value: i32 },
    /// Its darkvision reaches this many cells farther (from none: this far).
    Darkvision { value: i32 },
    /// It sees creatures suffering `status` (an id in `data/status/`) in its line of sight
    /// within `radius` cells however dark it is (Blood scent).
    Scent { status: String, radius: i32 },
    /// Its overworld sight at night reaches this many cells farther.
    NightSight { value: i32 },
    /// Its overworld sight by day reaches this many cells farther (Eyes of the heights).
    DaySight { value: i32 },
    /// The lit light it has ready in the Tool slot (a lantern) burns as the light preset `light`
    /// (an id in `data/lights.toml`; Lamplighter), not while hooded.
    LanternLight { light: String },
    /// It sheds light `light` (an id in `data/lights.toml`) wherever it is, always (Inner
    /// light): it sees by it, and so does everything else.
    InnerLight { light: String },
    /// It places every sound it hears where it was made (Tracker).
    ExactSounds,
    /// Its eyes are vision profile `profile` (an id in `data/vision.toml`), not its race's
    /// (Deep-eyed; [`crate::world::WorldState::eyes`]); the first such talent's.
    Vision { profile: String },
}

impl SensesEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`SensesEffect`]).
    pub const KINDS: [&'static str; 11] = [
        "darkvision",
        "day_sight",
        "exact_footsteps",
        "exact_sounds",
        "hearing",
        "inner_light",
        "lantern_light",
        "night_sight",
        "scent",
        "threshold_percent",
        "vision",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SensesMods {
    /// How much fainter it hears (its hearing threshold, lower).
    pub hearing: i32,
    /// It places the footsteps it hears exactly.
    pub exact_footsteps: bool,
    /// Percent more light its eyes need (negative: less).
    pub threshold_percent: i32,
    /// Cells farther its darkvision reaches.
    pub darkvision: i32,
    /// Statuses whose sufferers it sees in the dark, each within a radius: `(status, radius)`.
    pub scent: Vec<(StatusId, i32)>,
    /// Cells farther it sees on the overworld at night.
    pub night_sight: i32,
    /// Cells farther it sees on the overworld by day.
    pub day_sight: i32,
    /// The light preset the lit light it has ready (a lantern) burns as, if not its own
    /// (`WorldState::carried_light`; not while hooded).
    pub lantern_light: Option<LightId>,
    /// The light it sheds wherever it is, if any.
    pub inner_light: Option<LightId>,
    /// It places every sound it hears exactly (Tracker).
    pub exact_sounds: bool,
    /// The vision profile its eyes are, if not its race's (an index into
    /// `Content::vision_profiles`).
    pub vision: Option<usize>,
}

impl SensesMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: SensesEffect) {
        match effect {
            SensesEffect::Hearing { value } => self.hearing += value,
            SensesEffect::ExactFootsteps => self.exact_footsteps = true,
            SensesEffect::ThresholdPercent { value } => self.threshold_percent += value,
            SensesEffect::Darkvision { value } => self.darkvision += value,
            SensesEffect::Scent { status, radius } => {
                if let Some(status) = content.status_id(&status) {
                    self.scent.push((status, radius));
                }
            }
            SensesEffect::NightSight { value } => self.night_sight += value,
            SensesEffect::DaySight { value } => self.day_sight += value,
            SensesEffect::LanternLight { light } => {
                self.lantern_light = content.light_id(&light).or(self.lantern_light);
            }
            SensesEffect::InnerLight { light } => {
                self.inner_light = content.light_id(&light).or(self.inner_light);
            }
            SensesEffect::ExactSounds => self.exact_sounds = true,
            SensesEffect::Vision { profile } => {
                self.vision = self.vision.or(content.vision_index(&profile));
            }
        }
    }
}

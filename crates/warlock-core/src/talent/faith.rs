//! The faith talents (#1495): what a talent does for the holy and the unholy: altars, temples, the
//! paladin's healing and smites, holy radiance, the priest's rites and prayer, and retribution.
//! [`FaithEffect`] is the data (a talent's `[[effect]]` of one of [`FaithEffect::KINDS`]),
//! [`FaithMods`] the sum every rule reads (`TalentMods::faith`). A new effect of the area is a
//! variant here, its field and its arm in [`FaithMods::add`] here, its wiki text in
//! `xtask/src/wiki/talents/faith.rs`, and its one read.

use crate::content::{Content, LightId, ResourceCost, SmiteRiderKind, StatusId};
use crate::dice::Dice;
use crate::stats::Stat;
use serde::Deserialize;

/// What a talent of the area does, one typed effect, read by one rule each.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FaithEffect {
    /// Temple services cost it half, and a cure also mends a point of each damaged stat
    /// (Pious; `world/service.rs`).
    Pilgrim,
    /// At a forgotten altar of the actor's own side the side's lean counts double, and an
    /// opposed one leans as a neutral one (Pious; the prayer draw, `World::lean`).
    Devout,
    /// The class active Lay on hands: `per_level` hit points per character level back at once
    /// (what it costs is the talent's `resource`: Zeal).
    LayOnHands { per_level: u8 },
    /// Lay on hands heals fully and cures every status that may be cured and every disease, for
    /// `hands_cost_percent` of its cost; and Holy wrath's die lands on every foe (Avatar).
    Avatar { hands_cost_percent: u8 },
    /// The class active Smite: a melee blow in a direction with one die of `sides` more damage
    /// per `per_levels` character levels (at least one), `unholy` dice more against an unholy
    /// foe (`data/smite.toml`), and the smiter's modifier in `aim` added to hit (what it costs
    /// is the talent's `resource`: Zeal; `world/smite.rs`).
    Smite {
        per_levels: u8,
        sides: u8,
        #[serde(default)]
        aim: Option<Stat>,
        #[serde(default)]
        unholy: u8,
    },
    /// A mode of smiting (the paladin's; `world/smite.rs`, docs/design/paladin-smites.md §6):
    /// the class active of its `mode` (Branding smite, Wrathful smite), the Smite talent's blow
    /// with a rider on a hit that did damage (what it costs is the talent's `resource`: Zeal).
    /// `brand`: the foe suffers `severity` degrees of `status`. `fear`: the foe makes a `check`
    /// of that stat against `difficulty` plus the smiter's Smite `aim` modifier, and failing
    /// suffers `status` (one that flees) at `turns` degrees, caused by the smiter. `banish`
    /// takes nothing more: a summoned foe is unmade, and an unholy one takes the smite's dice
    /// twice (the rule's, `world/smite.rs`).
    SmiteMode {
        mode: SmiteRiderKind,
        #[serde(default)]
        status: String,
        #[serde(default)]
        severity: u16,
        #[serde(default)]
        turns: Option<Dice>,
        #[serde(default)]
        check: Option<Stat>,
        #[serde(default)]
        difficulty: i32,
    },
    /// Every blow, shot, or throw that hits an unholy target (`Content::unholy`; with Avatar any
    /// target) rolls `dice` more, after the blow's own damage, of its kind, resisted but never
    /// soaked (the paladin's Holy wrath, `world/smite.rs`).
    HolyWrath { dice: Dice },
    /// The class active Holy radiance: a toggle; while it suffers `status` it sheds light
    /// `light` (an id in `data/lights.toml`).
    Radiance { status: String, light: String },
    /// A hostile's melee attack beside it that lands a blow on it is answered by its smite at
    /// once, free, spending `cost` of `resource`; with less, nothing (Retribution, the
    /// paladin's at 25, on zeal; `world/smite.rs`).
    Retribution { resource: String, cost: u16 },
    /// The class active Consecrate: `time` (a duration) of prayer, for `mana` mana, lifts an
    /// unholy special room (or a whole level's unholy effect) for good (`world/consecrate.rs`).
    Consecrate { mana: i32, time: String },
    /// Communion with the dark (the priest design §6.3, `world/dark_face.rs`): standing in an
    /// unholy place, its prayers add no strain and the place's status never touches it. A
    /// band-gated passive: outside its talent's `alignment` it does nothing.
    Communion,
    /// The class active Sanctuary: `time` (a duration) of a kneeling rite, for `mana` mana,
    /// hallows the ground within `radius` of the player for `lasts`: no hostile sets foot in it,
    /// and those in it heal and regain mana `regen` percent as fast (`world/sanctuary.rs`).
    Sanctuary {
        mana: i32,
        time: String,
        radius: u8,
        lasts: String,
        regen: u16,
    },
    /// The priest's prayer (the class active Orison, `world/orison.rs`; `data/prayer.toml`):
    /// a prayer talent of `tier` (1 to 5, its `title` for each face) or a use added between
    /// them; `uses` it may answer, `answers` problems a prayer (the largest held), `all` every
    /// problem and every pool at once. The tier-1 talent carries the ability.
    Prayer {
        #[serde(default)]
        tier: Option<u8>,
        #[serde(default)]
        title: Option<crate::content::Faced>,
        #[serde(default)]
        uses: Vec<crate::content::PrayerUse>,
        #[serde(default)]
        answers: Option<u8>,
        #[serde(default)]
        all: bool,
    },
}

impl FaithEffect {
    /// The `kind`s of the data that are effects of the area, one a variant (`TalentEffect`'s
    /// deserializer hands these to [`FaithEffect`]).
    pub const KINDS: [&'static str; 13] = [
        "avatar",
        "communion",
        "consecrate",
        "devout",
        "holy_wrath",
        "lay_on_hands",
        "pilgrim",
        "prayer",
        "radiance",
        "retribution",
        "sanctuary",
        "smite",
        "smite_mode",
    ];
}

/// What an actor's talents of the area do, summed: one field per effect kind, read by one
/// rule each.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FaithMods {
    /// Temple services cost it half, and a cure also mends a point of each damaged stat
    /// (Pious).
    pub services: bool,
    /// A forgotten altar's lean counts double when it agrees, and not at all when it opposes
    /// (Pious).
    pub devout: bool,
    /// The active Lay on hands, if it has it.
    pub lay_on_hands: Option<LayOnHands>,
    /// Lay on hands heals fully and cures, for this percent of its cost (Avatar); `None`
    /// without Avatar.
    pub avatar: Option<u8>,
    /// The active Smite, if it has it.
    pub smite: Option<Smite>,
    /// The modes of smiting it has (Branding smite, Wrathful smite; `world/smite.rs`): each
    /// the Smite blow with its rider.
    pub smite_modes: Vec<SmiteRider>,
    /// Holy wrath's dice (`world/smite.rs`), if it has it.
    pub holy_wrath: Option<Dice>,
    /// The active Holy radiance: its status and the light it sheds while it lasts.
    pub radiance: Option<(StatusId, LightId)>,
    /// What each smite Retribution answers a blow with costs (the paladin's, on zeal); `None`
    /// without it.
    pub retribution: Option<ResourceCost>,
    /// The active Consecrate, if it has it.
    pub consecrate: Option<Consecrate>,
    /// Communion with the dark: in an unholy place its prayers add no strain and the place's
    /// status never touches it, within its talent's bands (`World::communes`).
    pub communion: bool,
    /// The active Sanctuary, if it has it.
    pub sanctuary: Option<Sanctuary>,
    /// The priest's prayer, its talents summed (`world/orison.rs`), if it has any.
    pub prayer: Option<Prayer>,
}

impl FaithMods {
    /// Adds what `effect` does.
    pub(crate) fn add(&mut self, content: &Content, effect: FaithEffect) {
        match effect {
            FaithEffect::Pilgrim => self.services = true,
            FaithEffect::Devout => self.devout = true,
            FaithEffect::LayOnHands { per_level } => {
                self.lay_on_hands = Some(LayOnHands { per_level });
            }
            FaithEffect::Avatar { hands_cost_percent } => {
                self.avatar = Some(hands_cost_percent);
            }
            FaithEffect::Smite {
                per_levels,
                sides,
                aim,
                unholy,
            } => {
                self.smite = Some(Smite {
                    per_levels,
                    sides,
                    aim,
                    unholy,
                });
            }
            FaithEffect::SmiteMode {
                mode,
                status,
                severity,
                turns,
                check,
                difficulty,
            } => {
                let rider = match (mode, content.status_id(&status), turns, check) {
                    (SmiteRiderKind::Banish, ..) => Some(SmiteRider::Banish),
                    (SmiteRiderKind::Brand, Some(status), ..) => {
                        Some(SmiteRider::Brand { status, severity })
                    }
                    (SmiteRiderKind::Fear, Some(status), Some(turns), Some(check)) => {
                        Some(SmiteRider::Fear {
                            status,
                            turns,
                            check,
                            difficulty,
                        })
                    }
                    _ => None,
                };
                if let Some(rider) = rider.filter(|r| self.smite_mode(r.active()).is_none()) {
                    self.smite_modes.push(rider);
                }
            }
            FaithEffect::HolyWrath { dice } => self.holy_wrath = Some(dice),
            FaithEffect::Radiance { status, light } => {
                if let (Some(status), Some(light)) =
                    (content.status_id(&status), content.light_id(&light))
                {
                    self.radiance = Some((status, light));
                }
            }
            FaithEffect::Retribution { resource, cost } => {
                if let Some(resource) = content.resource_id(&resource) {
                    self.retribution = Some(ResourceCost {
                        resource,
                        amount: cost,
                    });
                }
            }
            FaithEffect::Consecrate { mana, time } => {
                if let Ok(time) = time.parse() {
                    self.consecrate = Some(Consecrate { mana, time });
                }
            }
            FaithEffect::Sanctuary {
                mana,
                time,
                radius,
                lasts,
                regen,
            } => {
                if let (Ok(time), Ok(lasts)) = (time.parse(), lasts.parse()) {
                    self.sanctuary = Some(Sanctuary {
                        mana,
                        time,
                        radius,
                        lasts,
                        regen,
                    });
                }
            }
            FaithEffect::Communion => self.communion = true,
            FaithEffect::Prayer {
                tier,
                title,
                uses,
                answers,
                all,
            } => {
                let prayer = self.prayer.get_or_insert_with(|| Prayer {
                    answers: 1,
                    ..Prayer::default()
                });
                prayer.uses.extend(uses);
                prayer.answers = prayer.answers.max(answers.unwrap_or(1));
                prayer.all |= all;
                if let Some(tier) = tier.filter(|&t| t > prayer.tier) {
                    prayer.tier = tier;
                    prayer.title = title;
                }
            }
        }
    }

    /// The rider of mode of smiting `active` (Branding smite, Wrathful smite), if it has it.
    pub fn smite_mode(&self, active: crate::content::ClassActive) -> Option<SmiteRider> {
        self.smite_modes
            .iter()
            .copied()
            .find(|r| r.active() == active)
    }
}

/// The priest's prayer (spec §8 "Class abilities"; `world/orison.rs`): what its prayer talents
/// sum to: the uses (their union), the answers a prayer gives (the largest; at least 1), every
/// problem at once (any), and the highest tier with its titles.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Prayer {
    pub uses: std::collections::BTreeSet<crate::content::PrayerUse>,
    pub answers: u8,
    pub all: bool,
    pub tier: u8,
    pub title: Option<crate::content::Faced>,
}

impl Prayer {
    /// Whether it may answer `id`: a use held, or every use with `all`.
    pub fn answers_to(&self, id: crate::content::PrayerUse) -> bool {
        self.all || self.uses.contains(&id)
    }

    /// Its title as `face` gives it (`Orison`, `Whisper`), if a tier is held.
    pub fn title(&self, face: crate::content::Face) -> Option<&str> {
        self.title.as_ref().map(|t| t.of(face))
    }
}

/// The active Consecrate (spec §8 "Class abilities"): its mana and how long the prayer takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Consecrate {
    pub mana: i32,
    pub time: crate::time::Duration,
}

/// The active Sanctuary (spec §8 "Class abilities", §7 "Special rooms"; `world/sanctuary.rs`):
/// its mana, how long the rite takes, and the hallow it lays: its radius, how long it lasts,
/// and the healing and mana pace inside it, in percent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sanctuary {
    pub mana: i32,
    pub time: crate::time::Duration,
    pub radius: u8,
    pub lasts: crate::time::Duration,
    pub regen: u16,
}

/// The active Lay on hands (spec §8 "Class abilities"): hit points a character level; paid for
/// by its talent's `resource` (Zeal).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayOnHands {
    pub per_level: u8,
}

/// The active Smite (spec §8 "Class abilities"; `world/smite.rs`): paid for by its talent's
/// `resource` (Zeal).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Smite {
    pub per_levels: u8,
    pub sides: u8,
    /// The stat whose modifier is added to the smite's to-hit (Presence).
    pub aim: Option<Stat>,
    /// Dice more against an unholy foe (`Content::unholy`).
    pub unholy: u8,
}

/// A mode of smiting's rider (`FaithEffect::SmiteMode`; `world/smite.rs`), on a hit that did
/// damage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmiteRider {
    /// Branding smite's: `severity` degrees of `status` (Branded) on the foe.
    Brand { status: StatusId, severity: u16 },
    /// Wrathful smite's: a `check` of the foe's against `difficulty` plus the smiter's Smite
    /// `aim` modifier; failed, `status` (Frightened) at `turns` degrees.
    Fear {
        status: StatusId,
        turns: Dice,
        check: Stat,
        difficulty: i32,
    },
    /// Banishing smite's: a summoned foe is unmade (an unholy one took the dice twice).
    Banish,
}

impl SmiteRider {
    /// The class active whose rider it is.
    pub const fn active(self) -> crate::content::ClassActive {
        match self {
            SmiteRider::Brand { .. } => crate::content::ClassActive::BrandingSmite,
            SmiteRider::Fear { .. } => crate::content::ClassActive::WrathfulSmite,
            SmiteRider::Banish => crate::content::ClassActive::BanishingSmite,
        }
    }
}

impl Smite {
    /// The extra dice at character `level`: one die per `per_levels` levels, at least one, and
    /// `unholy` more against an unholy foe.
    pub fn dice_at(self, level: u8, unholy: bool) -> crate::dice::Dice {
        let count = (u32::from(level) / u32::from(self.per_levels.max(1))).max(1);
        let more = if unholy { u32::from(self.unholy) } else { 0 };
        crate::dice::Dice::of(count + more, u32::from(self.sides))
    }
}

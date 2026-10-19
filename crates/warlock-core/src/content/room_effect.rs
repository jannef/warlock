//! Special rooms (`assets/data/room_effects.toml`, spec §7 "Special rooms"), ADOM-like: a room of
//! a generated level, or an area a hand-made level marks, can carry an effect that changes the
//! rules while one is inside it.
//!
//! An effect ([`RoomEffectDef`]) may keep only part of the light ([`RoomEffectDef::light`]: a
//! shadowed room), add a glow ([`RoomEffectDef::glow`]: a sanctuary's), make the noises made
//! inside louder or quieter ([`RoomEffectDef::noise`]: an echoing hall, a hushed chamber), speed
//! up regeneration ([`RoomEffectDef::regen`]), keep creatures hostile to the player out
//! ([`RoomEffectDef::shunned`]), and have generators lay a floor of its own
//! ([`RoomEffectDef::floor`]: a flooded room's shallow water). The rules ask one question,
//! [`crate::World::room_effect_at`] (inside the core, the level state's `room_effect_at`).
//!
//! Where a special room lies is a [`SpecialRoom`]: a rectangle (a room of the rooms generator, an
//! area of a level file) or a blob of cells (a cave's), kept in the level's state with whether
//! the player has been inside yet.
//!
//! Without the file there are no special rooms.

use super::{ContentError, TerrainDef, TerrainId};
use crate::geom::Pos;
use crate::light::Light;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

/// Index of an effect in [`RoomEffects::defs`]; assigned in file order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RoomEffectId(pub u8);

/// The percent of a noise's loudness an effect may leave (spec §5 "Hearing").
pub const NOISE: RangeInclusive<u16> = 1..=1000;
/// The percent of the usual pace of regeneration an effect may give (spec §8 "Combat").
pub const REGEN: RangeInclusive<u16> = 1..=1000;
/// The most effects there can be.
pub const MAX_EFFECTS: usize = 32;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomEffectFile {
    /// Percent per room (or cave chamber) of a generated level.
    #[serde(default)]
    chance: u8,
    /// At most this many special rooms a generated level.
    #[serde(default = "one")]
    per_level: u8,
    #[serde(default)]
    effect: Vec<RoomEffectSpec>,
}

fn one() -> u8 {
    1
}

fn hundred() -> u16 {
    100
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomEffectSpec {
    id: String,
    name: String,
    mark: char,
    enter: String,
    #[serde(default = "hundred")]
    light: u16,
    #[serde(default)]
    glow: Light,
    #[serde(default = "hundred")]
    noise: u16,
    #[serde(default = "hundred")]
    regen: u16,
    #[serde(default)]
    shunned: bool,
    floor: Option<String>,
}

/// What a special room does while one is inside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomEffectDef {
    pub id: String,
    /// What it is called, without an article: "echoing hall" (look mode: "in an echoing hall").
    pub name: String,
    /// Marks its cells in the `rooms` dev dump layer.
    pub mark: char,
    /// Said the first time the player enters such a room.
    pub enter: String,
    /// Percent of all light kept inside (spec §5 "Special rooms"): 100 leaves it as it is.
    pub light: u16,
    /// Light added over the room, like ambient light (a sanctuary's warm glow).
    pub glow: Light,
    /// Percent of the loudness of every noise made inside (spec §5 "Hearing").
    pub noise: u16,
    /// Percent of the usual pace at which hit points come back inside (spec §8 "Combat").
    pub regen: u16,
    /// Creatures hostile to the player do not step into it (spec §8 "Creature behavior").
    pub shunned: bool,
    /// The floor generators lay over the room's floor (a flooded room's shallow water).
    pub floor: Option<TerrainId>,
}

impl RoomEffectDef {
    /// Its name with an article: "an echoing hall", "a sanctuary".
    pub fn with_article(&self) -> String {
        format!("{} {}", super::article(&self.name), self.name)
    }

    /// `light` as this room leaves it: all of it but in a room that keeps only part of it.
    pub fn shade(&self, light: Light) -> Light {
        light.percent(self.light)
    }
}

/// The effects of `data/room_effects.toml` and how often generated levels get them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoomEffects {
    /// Percent per room of a generated level (each of a cave's chambers) to be special.
    pub chance: u8,
    /// At most this many special rooms a generated level.
    pub per_level: u8,
    defs: Vec<RoomEffectDef>,
}

impl RoomEffects {
    /// Every effect, in file order (by [`RoomEffectId`]).
    pub fn defs(&self) -> &[RoomEffectDef] {
        &self.defs
    }

    pub fn get(&self, id: RoomEffectId) -> &RoomEffectDef {
        &self.defs[usize::from(id.0)]
    }

    pub fn id(&self, name: &str) -> Option<RoomEffectId> {
        let i = self.defs.iter().position(|d| d.id == name)?;
        Some(RoomEffectId(u8::try_from(i).ok()?))
    }

    /// Whether generated levels get special rooms at all.
    pub fn generated(&self) -> bool {
        self.chance > 0 && self.per_level > 0 && !self.defs.is_empty()
    }

    /// Parses `data/room_effects.toml`; `terrain` resolves a floor's terrain id and definition.
    pub(super) fn parse<'t>(
        file: &str,
        text: &str,
        terrain: impl Fn(&str) -> Option<(TerrainId, &'t TerrainDef)>,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: RoomEffectFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if parsed.chance > 100 {
            return Err(err(format!(
                "chance {} is a percent: 0 to 100",
                parsed.chance
            )));
        }
        if parsed.effect.len() > MAX_EFFECTS {
            return Err(err(format!("at most {MAX_EFFECTS} effects")));
        }
        let mut defs: Vec<RoomEffectDef> = Vec::new();
        for spec in parsed.effect {
            let bad = |m: String| err(format!("effect `{}`: {m}", spec.id));
            if defs.iter().any(|d| d.id == spec.id) {
                return Err(err(format!("duplicate effect id `{}`", spec.id)));
            }
            if spec.name.trim().is_empty() || spec.enter.trim().is_empty() {
                return Err(bad("`name` and `enter` must not be empty".into()));
            }
            if spec.mark.is_whitespace() || defs.iter().any(|d| d.mark == spec.mark) {
                return Err(bad(format!(
                    "mark `{}` must be visible and not another effect's",
                    spec.mark
                )));
            }
            if spec.light > 100 {
                return Err(bad(format!(
                    "light {} is the percent kept: 0 to 100",
                    spec.light
                )));
            }
            for (what, value, range) in [("noise", spec.noise, NOISE), ("regen", spec.regen, REGEN)]
            {
                if !range.contains(&value) {
                    return Err(bad(format!(
                        "{what} {value} is out of range ({} to {})",
                        range.start(),
                        range.end()
                    )));
                }
            }
            let floor = match &spec.floor {
                Some(name) => {
                    let (id, def) =
                        terrain(name).ok_or_else(|| bad(format!("unknown terrain `{name}`")))?;
                    let plain = def.walkable
                        && def.transparent
                        && def.stairs.is_none()
                        && !def.is_closed_door()
                        && !def.is_open_door()
                        && !def.flows();
                    if !plain {
                        return Err(bad(format!(
                            "floor `{name}` must be walkable and transparent, and not stairs, a \
                             door, or running water"
                        )));
                    }
                    Some(id)
                }
                None => None,
            };
            defs.push(RoomEffectDef {
                id: spec.id,
                name: spec.name,
                mark: spec.mark,
                enter: spec.enter,
                light: spec.light,
                glow: spec.glow,
                noise: spec.noise,
                regen: spec.regen,
                shunned: spec.shunned,
                floor,
            });
        }
        Ok(Self {
            chance: parsed.chance,
            per_level: parsed.per_level,
            defs,
        })
    }
}

/// Where a special room lies on a level, and whether the player has been inside it: a rectangle
/// (a room of the rooms generator with its walls and doors, an area of a level file), or a blob of
/// cells (a cave chamber with the rock around it). Kept in the level's state (saved).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecialRoom {
    pub effect: RoomEffectId,
    /// Corners of the box around it, inclusive.
    pub min: Pos,
    pub max: Pos,
    /// Its cells, sorted; empty when it is the whole box.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    cells: Vec<Pos>,
    /// Whether the player has been inside it: they were told what it is, and look mode names it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub entered: bool,
}

impl SpecialRoom {
    /// The rectangle from `min` to `max`, inclusive.
    pub fn rect(effect: RoomEffectId, min: Pos, max: Pos) -> Self {
        Self {
            effect,
            min,
            max,
            cells: Vec::new(),
            entered: false,
        }
    }

    /// The cells `cells` (at least one).
    pub fn blob(effect: RoomEffectId, mut cells: Vec<Pos>) -> Self {
        cells.sort();
        cells.dedup();
        let (min, max) = cells.iter().fold(
            (Pos::new(i32::MAX, i32::MAX), Pos::new(i32::MIN, i32::MIN)),
            |(lo, hi), p| {
                (
                    Pos::new(lo.x.min(p.x), lo.y.min(p.y)),
                    Pos::new(hi.x.max(p.x), hi.y.max(p.y)),
                )
            },
        );
        Self {
            effect,
            min,
            max,
            cells,
            entered: false,
        }
    }

    pub fn contains(&self, p: Pos) -> bool {
        let boxed =
            (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y);
        boxed && (self.cells.is_empty() || self.cells.binary_search(&p).is_ok())
    }

    /// Every cell of it, in position order.
    pub fn cells(&self) -> Vec<Pos> {
        if !self.cells.is_empty() {
            return self.cells.clone();
        }
        (self.min.x..=self.max.x)
            .flat_map(|x| (self.min.y..=self.max.y).map(move |y| Pos::new(x, y)))
            .collect()
    }

    /// Whether it is well formed within a `width` × `height` level: a box inside it, blob cells
    /// sorted, distinct, and inside the box (a guard for loaded saves).
    pub(crate) fn is_valid_within(&self, width: i32, height: i32) -> bool {
        let inside = |p: Pos| (0..width).contains(&p.x) && (0..height).contains(&p.y);
        let boxed = |p: &Pos| {
            (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y)
        };
        inside(self.min)
            && inside(self.max)
            && self.min.x <= self.max.x
            && self.min.y <= self.max.y
            && self.cells.windows(2).all(|w| w[0] < w[1])
            && self.cells.iter().all(boxed)
    }

    /// Whether it shares a cell with `other`.
    pub fn overlaps(&self, other: &SpecialRoom) -> bool {
        let boxes = self.min.x <= other.max.x
            && other.min.x <= self.max.x
            && self.min.y <= other.max.y
            && other.min.y <= self.max.y;
        boxes && self.cells().into_iter().any(|p| other.contains(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floor() -> TerrainDef {
        crate::content::Content::builtin()
            .expect("content")
            .terrain_defs()[1]
            .clone()
    }

    const SHADOWED: &str = "[[effect]]\nid = \"shadowed\"\nname = \"shadowed room\"\nmark = \"s\"\n\
                            enter = \"Dark.\"\nlight = 30\n";

    #[test]
    fn effects_parse_with_their_defaults() {
        let def = floor();
        let text = format!("chance = 7\n{SHADOWED}");
        let effects =
            RoomEffects::parse("e.toml", &text, |_| Some((TerrainId(1), &def))).expect("valid");
        assert_eq!((effects.chance, effects.per_level), (7, 1));
        let e = effects.get(RoomEffectId(0));
        assert_eq!((e.light, e.noise, e.regen), (30, 100, 100));
        assert_eq!((e.glow, e.shunned, e.floor), (Light::DARK, false, None));
        assert_eq!(effects.id("shadowed"), Some(RoomEffectId(0)));
        assert_eq!(e.with_article(), "a shadowed room");
        assert_eq!(
            e.shade(Light::new(100, 200, 10, 50)),
            Light::new(30, 60, 3, 15)
        );
        assert!(effects.generated());
        let none = RoomEffects::parse("e.toml", SHADOWED, |_| None).expect("valid");
        assert!(!none.generated(), "no chance, no generated rooms");
    }

    #[test]
    fn bad_effects_are_refused() {
        let def = floor();
        let wall = {
            let mut w = floor();
            w.walkable = false;
            w
        };
        for (text, why) in [
            (format!("chance = 101\n{SHADOWED}"), "chance 101"),
            (format!("{SHADOWED}{SHADOWED}"), "duplicate effect id"),
            (SHADOWED.replace("30", "101"), "light 101"),
            (format!("{SHADOWED}noise = 0\n"), "noise 0 is out of range"),
            (
                format!("{SHADOWED}regen = 1001\n"),
                "regen 1001 is out of range",
            ),
            (SHADOWED.replace("Dark.", " "), "must not be empty"),
            (SHADOWED.replace("\"s\"", "\" \""), "mark ` `"),
            (
                format!("{SHADOWED}floor = \"lava\"\n"),
                "unknown terrain `lava`",
            ),
            (
                format!("{SHADOWED}floor = \"wall\"\n"),
                "floor `wall` must be walkable",
            ),
            (format!("{SHADOWED}color = 3\n"), "unknown field `color`"),
        ] {
            let terrain = |name: &str| match name {
                "floor" => Some((TerrainId(1), &def)),
                "wall" => Some((TerrainId(2), &wall)),
                _ => None,
            };
            let e = RoomEffects::parse("e.toml", &text, terrain).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
        let twin = format!("{SHADOWED}{}", SHADOWED.replace("\"shadowed\"", "\"dark\""));
        let e = RoomEffects::parse("e.toml", &twin, |_| None).unwrap_err();
        assert!(e.message.contains("not another effect's"), "{}", e.message);
    }

    #[test]
    fn rooms_are_boxes_or_blobs() {
        let id = RoomEffectId(0);
        let rect = SpecialRoom::rect(id, Pos::new(2, 1), Pos::new(4, 3));
        assert!(rect.contains(Pos::new(2, 1)) && rect.contains(Pos::new(4, 3)));
        assert!(!rect.contains(Pos::new(5, 3)) && !rect.contains(Pos::new(1, 2)));
        assert_eq!(rect.cells().len(), 9);
        let blob = SpecialRoom::blob(id, vec![Pos::new(6, 2), Pos::new(5, 3), Pos::new(6, 2)]);
        assert_eq!((blob.min, blob.max), (Pos::new(5, 2), Pos::new(6, 3)));
        assert!(blob.contains(Pos::new(6, 2)) && !blob.contains(Pos::new(6, 3)));
        assert_eq!(blob.cells(), vec![Pos::new(5, 3), Pos::new(6, 2)]);
        assert!(rect.is_valid_within(10, 10) && blob.is_valid_within(10, 10));
        assert!(!rect.is_valid_within(4, 10));
        assert!(!rect.overlaps(&blob));
        let touching = SpecialRoom::rect(id, Pos::new(4, 3), Pos::new(5, 3));
        assert!(rect.overlaps(&touching) && blob.overlaps(&touching));
        // Saved compactly: a box has no cell list, a room not entered no flag.
        let json = serde_json::to_string(&rect).expect("serializes");
        assert_eq!(
            json,
            r#"{"effect":0,"min":{"x":2,"y":1},"max":{"x":4,"y":3}}"#
        );
        let back: SpecialRoom = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(back, rect);
    }
}

//! Blood (spec §8 "Blood"): the stains big hits and kills leave on floors and walls.
//!
//! Stains are presentation data the rules keep, like the flow of water: they live per level
//! ([`crate::world::LevelState`], sparse, by cell), are saved and hashed, and **no rule reads
//! them**. Where blood lands comes from a hash of the blow ([`seed`]: the turn, the moment, the
//! two actors) and the cells, never from the gameplay random stream, so blood never shifts a
//! later roll.
//!
//! - **When** ([`Wound::volume`]): a blow that kills leaves a pool on the victim's cell and
//!   spray around it; a big hit (damage of at least a third of the target's maximum hit points,
//!   or any confirmed critical) a little spatter. Lesser blows leave nothing.
//! - **How much**: [`KILL`] or [`BIG_HIT`] scaled by the victim's size (`blood` in
//!   `data/sizes.toml`, percent of a medium one's), then overkill (damage past 0 hit points, up
//!   to twice as much for a whole second life's worth), a confirmed critical (half again), and a
//!   missile (half: it pierces rather than cleaves).
//! - **Where** ([`Wound::spatter`]): a share on the victim's cell; the rest sprays away from
//!   whoever struck, in a fan along the blow (±45° in melee, ±30° along a missile's flight), up
//!   to [`Wound::reach`] cells: always on the cells right behind the victim, sparser and fainter
//!   farther out. A kill also flecks the cells around the victim. Spray stops at opaque cells: a
//!   wall next to the victim takes it on its face (the side facing the victim,
//!   [`Stain::faces`]), twice as much as a floor would; farther walls and what lies behind them
//!   take none.
//! - **Accumulation** ([`Stain::add`]): stains add up to [`CAP`] per cell; their colours mix by
//!   amount, and their age (for drying, drawn by the app) is the amount-weighted mean of when
//!   each was spilled.

use crate::color::Rgb;
use crate::geom::{Dir, Pos};
use crate::time::GameTime;
use serde::{Deserialize, Serialize};

/// The most blood a cell holds: soaked.
pub const CAP: u8 = 100;
/// Blood a kill spills (before size, overkill, a critical, and a missile), in stain amounts.
pub const KILL: u32 = 30;
/// Blood a big hit spills.
pub const BIG_HIT: u32 = 10;
/// The colour of blood unless a creature type says otherwise: dark red.
pub const RED: Rgb = Rgb::new(0x6e, 0x0e, 0x0c);
/// What a blow must take of its target's maximum hit points, at least, to be a big hit: a
/// third.
pub const BIG_SHARE: i32 = 3;

/// Blood on one cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stain {
    /// How much: 1 to [`CAP`].
    pub amount: u8,
    /// Its colour when fresh: the bloods spilled there, mixed by amount.
    pub color: Rgb,
    /// When it was spilled: the mean of the moments of each spill weighted by its amount. Blood
    /// dries with age (drawn darker and browner).
    pub spilled: GameTime,
    /// On an opaque cell, the faces that are stained: one bit per direction from the cell toward
    /// where the blood came from, in [`Dir::ALL`] order. 0 on floors.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub faces: u8,
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

/// The bit of face `dir` in [`Stain::faces`].
pub fn face_bit(dir: Dir) -> u8 {
    let index = Dir::ALL.iter().position(|d| *d == dir).unwrap_or(0);
    1 << index
}

impl Stain {
    /// A fresh stain of `amount` of blood of `color` spilled at `now` (on face `face` of a
    /// wall).
    pub fn new(amount: u8, color: Rgb, now: GameTime, face: Option<Dir>) -> Self {
        Self {
            amount: amount.clamp(1, CAP),
            color,
            spilled: now,
            faces: face.map_or(0, face_bit),
        }
    }

    /// More blood lands here: `amount` of `color` at `now` (on face `face`). The amount adds up
    /// to [`CAP`]; colour and age mix by amount (what lands, before the cap).
    pub fn add(&mut self, amount: u8, color: Rgb, now: GameTime, face: Option<Dir>) {
        let (old, new) = (u32::from(self.amount), u32::from(amount));
        let total = old + new;
        if total == 0 {
            return;
        }
        let blend = |a: u8, b: u8| ((u32::from(a) * old + u32::from(b) * new) / total) as u8;
        self.color = Rgb::new(
            blend(self.color.r, color.r),
            blend(self.color.g, color.g),
            blend(self.color.b, color.b),
        );
        let (t0, t1) = (
            u128::from(self.spilled.seconds()),
            u128::from(now.seconds()),
        );
        let mean = (t0 * u128::from(old) + t1 * u128::from(new)) / u128::from(total);
        self.spilled = GameTime::from_seconds(u64::try_from(mean).unwrap_or(u64::MAX));
        self.amount = total.min(u32::from(CAP)) as u8;
        self.faces |= face.map_or(0, face_bit);
    }

    /// Whether face `dir` of a wall is stained.
    pub fn on_face(&self, dir: Dir) -> bool {
        self.faces & face_bit(dir) != 0
    }

    /// How it reads in look mode, before the terrain's name: "blood-spattered",
    /// "blood-stained", "blood-soaked" by amount; a `wall` is spattered or smeared.
    pub fn word(&self, wall: bool) -> &'static str {
        match (wall, self.amount) {
            (false, ..25) | (true, ..40) => "blood-spattered",
            (false, ..70) => "blood-stained",
            (false, _) => "blood-soaked",
            (true, _) => "blood-smeared",
        }
    }
}

/// The seed of a blow's splatter: the turn and the moment it was struck and who struck whom.
/// Never the gameplay random stream.
pub fn seed(turn: u64, time: GameTime, attacker: u32, victim: u32) -> u64 {
    let who = (u64::from(attacker) << 32) | u64::from(victim);
    mix(mix(turn) ^ mix(time.seconds().rotate_left(17)) ^ who)
}

/// SplitMix64's finalizer: a well-mixed 64-bit hash of `z`.
fn mix(z: u64) -> u64 {
    let z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// One blow as blood sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wound {
    /// Where the blow came from: the attacker's cell (the shooter's for a missile).
    pub from: Pos,
    /// The victim's cell.
    pub at: Pos,
    /// A shot or a throw rather than a melee blow.
    pub missile: bool,
    pub damage: i32,
    /// The victim's hit points before the blow, and its maximum.
    pub hp: i32,
    pub max_hp: i32,
    /// A confirmed critical.
    pub critical: bool,
    /// The victim's size's `blood`, in percent of a medium one's.
    pub size: u16,
    /// [`seed`].
    pub seed: u64,
}

/// Blood landing on one cell: `amount`, on face `face` of a wall.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spatter {
    pub at: Pos,
    pub amount: u8,
    pub face: Option<Dir>,
}

impl Wound {
    /// Whether the blow kills.
    pub fn kills(&self) -> bool {
        self.damage >= self.hp
    }

    /// Whether the blow is a big hit: a critical, or at least a third of the maximum.
    pub fn is_big(&self) -> bool {
        self.critical || self.damage * BIG_SHARE >= self.max_hp
    }

    /// How much blood the blow spills (module docs); 0 for a lesser blow.
    pub fn volume(&self) -> u32 {
        let base = if self.kills() {
            KILL
        } else if self.is_big() {
            BIG_HIT
        } else {
            return 0;
        };
        let mut volume = base * u32::from(self.size) / 100;
        if self.kills() {
            let max = self.max_hp.max(1);
            let over = (self.damage - self.hp.max(0)).clamp(0, max);
            volume = volume * (100 + (100 * over / max) as u32) / 100;
        }
        if self.critical {
            volume = volume * 3 / 2;
        }
        if self.missile {
            volume /= 2;
        }
        volume
    }

    /// How far the spray flies, in cells: farther the more blood there is, at most 3 (2 for a
    /// missile).
    pub fn reach(&self) -> i32 {
        let reach = match self.volume() {
            ..20 => 1,
            20..45 => 2,
            _ => 3,
        };
        if self.missile { reach.min(2) } else { reach }
    }

    /// Where the blood lands (module docs), the victim's cell first, then the cells around it
    /// row by row; `is_opaque` tells walls (and the level's edge) from open cells. Whether a cell
    /// takes a stain (water and lava do not) is the caller's to decide.
    pub fn spatter(&self, is_opaque: impl Fn(Pos) -> bool) -> Vec<Spatter> {
        let volume = self.volume();
        if volume == 0 {
            return Vec::new();
        }
        let kills = self.kills();
        let pool = if kills { volume / 2 } else { volume / 3 };
        let mut out = vec![Spatter {
            at: self.at,
            amount: clamp_amount(pool.max(1)),
            face: None,
        }];
        let (bx, by) = (
            i64::from(self.at.x - self.from.x),
            i64::from(self.at.y - self.from.y),
        );
        let b2 = bx * bx + by * by;
        let reach = self.reach();
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let d2 = dx * dx + dy * dy;
                if d2 == 0 || d2 > reach * reach + reach {
                    continue;
                }
                let p = Pos::new(self.at.x + dx, self.at.y + dy);
                let (ox, oy) = (i64::from(dx), i64::from(dy));
                let dot = ox * bx + oy * by;
                let o2 = ox * ox + oy * oy;
                // Within the fan: cos² of the angle to the blow at least ½ (45°) in melee, ¾
                // (30°) along a missile's flight.
                let in_fan = b2 > 0
                    && dot > 0
                    && if self.missile {
                        4 * dot * dot >= 3 * o2 * b2
                    } else {
                        2 * dot * dot >= o2 * b2
                    };
                let ring = p.chebyshev(self.at);
                let (chance, amount) = if in_fan {
                    // Right behind the victim it always lands; farther out it thins.
                    let near = [100, 60, 40][(ring - 1).clamp(0, 2) as usize];
                    (near, volume * 4 / (15 * ring as u32))
                } else if kills && ring == 1 {
                    (35, volume * 2 / 15)
                } else {
                    continue;
                };
                let roll = mix(self.seed ^ mix(cell_key(p)));
                if roll % 100 >= chance {
                    continue;
                }
                // Spray stops at opaque cells: one next to the victim takes it on its face.
                if (is_opaque(p) && ring > 1) || self.at.line_between(p).any(&is_opaque) {
                    continue;
                }
                // Each drop lands with 60 to 140 % of its share; a wall catches twice as much
                // (it runs down the face rather than spreading over the floor).
                let jitter = 60 + ((roll >> 8) % 81) as u32;
                let face = is_opaque(p).then(|| Dir::toward(p, self.at)).flatten();
                let wall = if face.is_some() { 2 } else { 1 };
                out.push(Spatter {
                    at: p,
                    amount: clamp_amount((amount * jitter * wall / 100).max(1)),
                    face,
                });
            }
        }
        out
    }
}

fn clamp_amount(amount: u32) -> u8 {
    amount.min(u32::from(CAP)) as u8
}

fn cell_key(p: Pos) -> u64 {
    (u64::from(p.x as u32) << 32) | u64::from(p.y as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// A blow from the west on (10, 10) of `damage` on a victim with `hp` of `max`.
    fn wound(damage: i32, hp: i32, max: i32) -> Wound {
        Wound {
            from: Pos::new(9, 10),
            at: Pos::new(10, 10),
            missile: false,
            damage,
            hp,
            max_hp: max,
            critical: false,
            size: 100,
            seed: seed(7, GameTime::from_seconds(1000), 1, 2),
        }
    }

    fn open(_: Pos) -> bool {
        false
    }

    #[test]
    fn kills_and_big_hits_bleed_lesser_blows_do_not() {
        // 10 hit points: a blow of 3 is less than a third, 4 is a big hit, 10 kills.
        assert_eq!(wound(3, 10, 12).volume(), 0);
        assert!(wound(3, 10, 12).spatter(open).is_empty());
        assert!(wound(4, 10, 12).is_big() && !wound(4, 10, 12).kills());
        assert_eq!(wound(4, 10, 12).volume(), BIG_HIT);
        assert_eq!(wound(10, 10, 12).volume(), KILL);
        // Any confirmed critical is a big hit, however little it does.
        let crit = Wound {
            critical: true,
            ..wound(1, 10, 12)
        };
        assert_eq!(crit.volume(), BIG_HIT * 3 / 2);
    }

    #[test]
    fn size_overkill_criticals_and_missiles_scale_the_blood() {
        let kill = wound(10, 10, 10);
        let small = Wound { size: 60, ..kill };
        assert_eq!(small.volume(), KILL * 60 / 100);
        // Overkill: half a life past 0 is half again; a whole one (or more) twice.
        assert_eq!(wound(15, 10, 10).volume(), KILL * 3 / 2);
        assert_eq!(wound(20, 10, 10).volume(), KILL * 2);
        assert_eq!(wound(90, 10, 10).volume(), KILL * 2);
        let crit = Wound {
            critical: true,
            ..kill
        };
        assert_eq!(crit.volume(), KILL * 3 / 2);
        let arrow = Wound {
            missile: true,
            ..kill
        };
        assert_eq!(arrow.volume(), KILL / 2);
        // A rat killed by an arrow leaves a few drops; a critical spear thrust through a kobold
        // a pool and a fan of spatter behind it.
        let rat = Wound {
            size: 60,
            missile: true,
            ..wound(5, 5, 12)
        };
        let thrust = Wound {
            size: 60,
            critical: true,
            ..wound(18, 4, 10)
        };
        assert!(rat.volume() < 10 && rat.reach() == 1);
        assert!(thrust.volume() >= 50 && thrust.reach() == 3);
        let spilled =
            |w: Wound| -> u32 { w.spatter(open).iter().map(|s| u32::from(s.amount)).sum() };
        assert!(spilled(rat) * 4 < spilled(thrust));
    }

    #[test]
    fn melee_sprays_away_from_the_attacker() {
        // Struck from the west: everything but the flecks right around the victim lies east.
        for s in 0..40 {
            let w = Wound {
                seed: seed(s, GameTime::from_seconds(5), 1, 2),
                critical: true,
                ..wound(25, 10, 10)
            };
            let cells = w.spatter(open);
            assert_eq!(cells[0].at, w.at, "the pool first");
            for c in &cells[1..] {
                assert!(
                    c.at.x > w.at.x || c.at.chebyshev(w.at) == 1,
                    "{:?} behind the blow",
                    c.at
                );
            }
            assert!(
                cells.iter().any(|c| c.at.x >= w.at.x + 2),
                "a fan reaches out"
            );
        }
        // From the north-east, the fan lies south-west.
        let w = Wound {
            from: Pos::new(11, 9),
            critical: true,
            ..wound(25, 10, 10)
        };
        let far: Vec<Pos> = (w.spatter(open).iter())
            .map(|c| c.at)
            .filter(|p| p.chebyshev(w.at) > 1)
            .collect();
        assert!(!far.is_empty());
        assert!(
            far.iter().all(|p| p.x <= w.at.x && p.y >= w.at.y),
            "{far:?}"
        );
    }

    #[test]
    fn missiles_spray_along_their_flight_and_less() {
        // Shot from far to the west: a narrow fan east.
        let shot = Wound {
            from: Pos::new(2, 10),
            missile: true,
            critical: true,
            ..wound(30, 10, 10)
        };
        let blow = Wound {
            missile: false,
            ..shot
        };
        let cells = shot.spatter(open);
        assert!(
            cells[1..]
                .iter()
                .all(|c| c.at.x > shot.at.x || c.at.chebyshev(shot.at) == 1)
        );
        assert!(cells[1..].iter().all(|c| (c.at.y - shot.at.y).abs() <= 1));
        let total = |cells: &[Spatter]| cells.iter().map(|s| u32::from(s.amount)).sum::<u32>();
        assert!(total(&cells) < total(&blow.spatter(open)));
    }

    #[test]
    fn walls_take_blood_only_next_to_the_victim_on_the_face_toward_it() {
        // A wall all along x = 11 (east of the victim) and x = 13.
        let walls = |p: Pos| p.x == 11 || p.x == 13;
        let mut stained = BTreeMap::new();
        for s in 0..20 {
            let w = Wound {
                seed: seed(s, GameTime::from_seconds(9), 3, 4),
                ..wound(20, 10, 10)
            };
            for c in w.spatter(walls) {
                assert!(c.at.x <= 11, "nothing gets past the wall: {:?}", c.at);
                if walls(c.at) {
                    assert_eq!(c.at.chebyshev(w.at), 1, "only walls next to the victim");
                    let face = c.face.expect("a wall's face");
                    assert_eq!(c.at.step(face).x, 10, "the face toward the victim");
                }
                *stained.entry(c.at).or_insert(0) += 1;
            }
        }
        assert_eq!(
            stained.get(&Pos::new(11, 10)),
            Some(&20),
            "the wall it was driven into"
        );
    }

    #[test]
    fn the_pattern_is_a_pure_function_of_the_blow() {
        let w = wound(20, 10, 10);
        assert_eq!(w.spatter(open), w.spatter(open));
        let other = Wound {
            seed: seed(8, GameTime::from_seconds(1000), 1, 2),
            ..w
        };
        assert_ne!(
            w.spatter(open),
            other.spatter(open),
            "another turn, another pattern"
        );
    }

    #[test]
    fn stains_add_up_to_the_cap_mixing_colour_and_age() {
        let t = |s| GameTime::from_seconds(s);
        let mut stain = Stain::new(30, RED, t(100), None);
        stain.add(30, RED, t(400), None);
        assert_eq!((stain.amount, stain.color), (60, RED));
        assert_eq!(stain.spilled, t(250), "the mean moment, weighted by amount");
        let ichor = Rgb::new(0x3c, 0x4a, 0x20);
        stain.add(60, ichor, t(400), Some(Dir::E));
        assert_eq!(stain.amount, CAP);
        let half = |a: u8, b: u8| ((u32::from(a) + u32::from(b)) / 2) as u8;
        assert_eq!(
            stain.color,
            Rgb::new(
                half(RED.r, ichor.r),
                half(RED.g, ichor.g),
                half(RED.b, ichor.b)
            )
        );
        assert!(stain.on_face(Dir::E) && !stain.on_face(Dir::W));
        for _ in 0..10 {
            stain.add(CAP, RED, t(500), None);
        }
        assert_eq!(stain.amount, CAP);
        assert_eq!(Stain::new(0, RED, t(1), None).amount, 1);
    }

    #[test]
    fn stains_read_by_amount() {
        let at = |amount| Stain::new(amount, RED, GameTime::from_seconds(0), None);
        assert_eq!(at(10).word(false), "blood-spattered");
        assert_eq!(at(40).word(false), "blood-stained");
        assert_eq!(at(90).word(false), "blood-soaked");
        assert_eq!(at(10).word(true), "blood-spattered");
        assert_eq!(at(60).word(true), "blood-smeared");
    }
}

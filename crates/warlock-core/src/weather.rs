//! Weather (spec §5 "Weather"): one weather over every level under the open sky, changing every
//! few hours, a pure function of the run seed and the time.
//!
//! **Blocks**: time is cut into blocks of about [`BLOCK`] (three hours). Block `k` starts at
//! `k · BLOCK` plus a shift of up to [`SHIFT`] drawn for it, so the weather does not always
//! change on the hour. Each block has one weather.
//!
//! **Which weather**: every block draws a number `r_k`, uniform in `[0, 1)`, from a hash of the
//! seed and `k`. The block's value is the sum of its own and the previous [`WINDOW`] − 1 blocks'
//! numbers, turned back into a uniform value by the sum's distribution (Irwin–Hall), so
//! neighboring blocks share most of their value: weather lasts, and mostly changes to a
//! neighboring kind (clear, overcast, rain, heavy rain, in that order). The value picks the
//! weather by the chances of the block's month (`data/weather.toml`): each weather comes up
//! exactly as often as its chance says.
//!
//! **Clouds**: the weather dims the light of the sky ([`Clouds`], applied in
//! [`crate::light::Sky::at`]). Where the weather changes, the clouds change over [`CHANGE`]
//! (an hour) centred on the moment the weather changes, so the light dims and brightens
//! smoothly.
//!
//! No state: skipping time or resting needs no history. Integer math only; the hash is FNV-1a
//! over the seed, a label, and the block number, mixed by SplitMix64's finalizer, so the weather
//! is the same on every platform. A dev override ([`Override`], the `weather:` action) sets the
//! weather for [`OVERRIDE`] from the moment it is given.

use crate::content::{Climate, WeatherDef};
use crate::light::Light;
use crate::time::{Duration, GameTime, HOUR, MINUTE};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The nominal length of a weather block, in seconds.
pub const BLOCK: u64 = 3 * HOUR;
/// How far past `k · BLOCK` a block may start, in seconds. Less than `BLOCK − CHANGE`, so every
/// block is longer than a change of clouds.
pub const SHIFT: u64 = 90 * MINUTE;
/// How long the clouds take to change from one weather to the next, in seconds, centred on the
/// moment the weather changes.
pub const CHANGE: u64 = HOUR;
/// How long a dev override lasts.
pub const OVERRIDE: Duration = Duration::hours(3);
/// How many blocks' draws a block's weather sums: the more, the longer weather lasts.
pub const WINDOW: usize = 4;

const _: () = assert!(SHIFT + CHANGE < BLOCK);

/// The weather, from the clearest to the wettest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Weather {
    Clear,
    Overcast,
    Rain,
    HeavyRain,
}

impl Weather {
    pub const ALL: [Weather; 4] = [
        Weather::Clear,
        Weather::Overcast,
        Weather::Rain,
        Weather::HeavyRain,
    ];

    /// Its place in [`Weather::ALL`].
    pub const fn index(self) -> usize {
        self as usize
    }

    /// How the status bar names it.
    pub const fn name(self) -> &'static str {
        match self {
            Weather::Clear => "Clear",
            Weather::Overcast => "Overcast",
            Weather::Rain => "Rain",
            Weather::HeavyRain => "Heavy rain",
        }
    }

    /// Its id in action text (`weather:heavy`).
    pub const fn id(self) -> &'static str {
        match self {
            Weather::Clear => "clear",
            Weather::Overcast => "overcast",
            Weather::Rain => "rain",
            Weather::HeavyRain => "heavy",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|w| w.id() == id)
    }

    /// Whether it rains.
    pub const fn is_rain(self) -> bool {
        matches!(self, Weather::Rain | Weather::HeavyRain)
    }
}

impl fmt::Display for Weather {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// How much of the sky's light the clouds let through, in per mille: `direct` of the direct
/// light of the sun and the moons, `sky` of the sky's own light, and `scatter` of the direct
/// light spread over the whole sky as sky light.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Clouds {
    pub direct: u16,
    pub sky: u16,
    pub scatter: u16,
}

impl Clouds {
    /// No clouds: the sky as it is.
    pub const CLEAR: Clouds = Clouds {
        direct: 1000,
        sky: 1000,
        scatter: 0,
    };

    /// The clouds of a weather.
    pub fn of(def: &WeatherDef) -> Self {
        Self {
            direct: u16::from(def.direct) * 10,
            sky: u16::from(def.sky) * 10,
            scatter: u16::from(def.scatter) * 10,
        }
    }

    /// Between `self` and `to`, `w` per mille of the way.
    fn mix(self, to: Clouds, w: u64) -> Self {
        let w = w.min(1000);
        let mix = |a: u16, b: u16| ((u64::from(a) * (1000 - w) + u64::from(b) * w) / 1000) as u16;
        Self {
            direct: mix(self.direct, to.direct),
            sky: mix(self.sky, to.sky),
            scatter: mix(self.scatter, to.scatter),
        }
    }

    /// Direct light (the sun's or a moon's) under these clouds.
    pub fn direct(&self, light: Light) -> Light {
        part(light, self.direct)
    }

    /// Sky light under these clouds: `sky`, the sky's own light, and the share of `direct`, the
    /// direct light above them, they spread.
    pub fn sky(&self, sky: Light, direct: Light) -> Light {
        part(sky, self.sky).saturating_add(part(direct, self.scatter))
    }
}

/// `per_mille` of `light`, channel by channel, rounded up: a light that shines at all still
/// shines (so the sun is still up behind the clouds and a moon still shines).
fn part(light: Light, per_mille: u16) -> Light {
    let c = |c: u16| {
        let scaled = (u32::from(c) * u32::from(per_mille)).div_ceil(1000);
        u16::try_from(scaled).unwrap_or(u16::MAX)
    };
    let [r, g, b, uv]: [u16; 4] = light.into();
    Light::new(c(r), c(g), c(b), c(uv))
}

/// The weather at one moment and the clouds it brings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Conditions {
    pub weather: Weather,
    pub clouds: Clouds,
}

impl Conditions {
    /// A clear sky.
    pub const CLEAR: Conditions = Conditions {
        weather: Weather::Clear,
        clouds: Clouds::CLEAR,
    };
}

/// Dev: the weather set by hand (the `weather:` action) until `until`; then the weather of the
/// day returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Override {
    pub weather: Weather,
    pub until: GameTime,
}

impl Override {
    /// `weather` for [`OVERRIDE`] from `now`.
    pub fn new(weather: Weather, now: GameTime) -> Self {
        Self {
            weather,
            until: now + OVERRIDE,
        }
    }

    /// The weather it sets at `time`: from when it was given (`until` less [`OVERRIDE`]) until
    /// `until`. (Ice catching up with the past asks about moments before it was given.)
    fn at(over: Option<&Override>, time: GameTime) -> Option<Weather> {
        let given = |o: &Override| {
            GameTime::from_seconds(o.until.seconds().saturating_sub(OVERRIDE.as_seconds()))
        };
        over.filter(|o| given(o) <= time && time < o.until)
            .map(|o| o.weather)
    }
}

/// The weather at `time` in the run with `seed`, with the dev override `over` if any.
pub fn weather_at(
    seed: u64,
    time: GameTime,
    climate: &Climate,
    over: Option<&Override>,
) -> Weather {
    if let Some(w) = Override::at(over, time) {
        return w;
    }
    let block = Block::at(seed, time);
    block.weather(seed, climate)
}

/// The weather at `time` and its clouds; see [`weather_at`].
pub fn conditions(
    seed: u64,
    time: GameTime,
    climate: &Climate,
    over: Option<&Override>,
) -> Conditions {
    let clouds = |w: Weather| Clouds::of(climate.def(w));
    let blend = blend(seed, time, climate, over);
    Conditions {
        weather: blend.weather,
        clouds: clouds(blend.from).mix(clouds(blend.to), blend.w),
    }
}

/// The weather at one moment, and the sky on its way between two weathers: `w` per mille of the
/// way from `from` to `to` (the clouds, and the temperature, change over [`CHANGE`] centred on
/// the moment the weather changes; away from a change both are the weather of the moment).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Blend {
    pub weather: Weather,
    pub from: Weather,
    pub to: Weather,
    pub w: u64,
}

impl Blend {
    /// `of(from)` and `of(to)` mixed `w` per mille of the way, rounded down.
    pub fn mix(&self, of: impl Fn(Weather) -> i64) -> i64 {
        let w = self.w.min(1000) as i64;
        (of(self.from) * (1000 - w) + of(self.to) * w).div_euclid(1000)
    }
}

/// The weather at `time` and how the sky is changing then; see [`Blend`]. A dev override sets
/// its weather at once, with nothing on its way.
pub fn blend(seed: u64, time: GameTime, climate: &Climate, over: Option<&Override>) -> Blend {
    let steady = |weather| Blend {
        weather,
        from: weather,
        to: weather,
        w: 0,
    };
    if let Some(weather) = Override::at(over, time) {
        return steady(weather);
    }
    let t = time.seconds() as i64;
    let half = (CHANGE / 2) as i64;
    let block = Block::at(seed, time);
    let weather = block.weather(seed, climate);
    // Within half a change of either end, the sky is on its way from or to the neighbor's.
    if t < block.start + half {
        let before = Block::nth(seed, block.k - 1).weather(seed, climate);
        let w = (t - (block.start - half)) as u64 * 1000 / CHANGE;
        Blend {
            weather,
            from: before,
            to: weather,
            w,
        }
    } else if t >= block.end - half {
        let after = Block::nth(seed, block.k + 1).weather(seed, climate);
        let w = (t - (block.end - half)) as u64 * 1000 / CHANGE;
        Blend {
            weather,
            from: weather,
            to: after,
            w,
        }
    } else {
        steady(weather)
    }
}

/// The weather block `k`: from `start` (inclusive) to `end` (exclusive), in seconds since the
/// epoch (the block before the epoch's first starts before it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Block {
    k: i64,
    start: i64,
    end: i64,
}

impl Block {
    fn nth(seed: u64, k: i64) -> Self {
        Self {
            k,
            start: start(seed, k),
            end: start(seed, k + 1),
        }
    }

    /// The block `time` falls in.
    fn at(seed: u64, time: GameTime) -> Self {
        let t = time.seconds() as i64;
        let k = t.div_euclid(BLOCK as i64);
        // Block `k` starts within `SHIFT` after `k · BLOCK`; before that, it is still `k − 1`.
        let k = if t < start(seed, k) { k - 1 } else { k };
        Self::nth(seed, k)
    }

    /// The block's weather: its spread value against the chances of the month it starts in.
    fn weather(&self, seed: u64, climate: &Climate) -> Weather {
        let month = GameTime::from_seconds(self.start.max(0) as u64).month_index();
        let value = spread((0..WINDOW as i64).map(|i| draw(seed, b"weather", self.k - i) >> 48));
        pick(value, climate.chances(month))
    }
}

/// When block `k` starts, in seconds since the epoch.
fn start(seed: u64, k: i64) -> i64 {
    let shift = draw(seed, b"weather/start", k) % (SHIFT + 1);
    k * BLOCK as i64 + shift as i64
}

/// The weather a value (in 1/65 536, uniform) picks by `chances` (percent, in
/// [`Weather::ALL`] order): the lowest values the clearest.
fn pick(value: u32, chances: [u8; 4]) -> Weather {
    let mut below = 0u64;
    for (w, chance) in Weather::ALL.into_iter().zip(chances) {
        below += u64::from(chance);
        if u64::from(value) * 100 < below << 16 {
            return w;
        }
    }
    Weather::HeavyRain
}

/// One unit of a draw (16 bits).
const UNIT: i128 = 1 << 16;

/// The sum of [`WINDOW`] draws (each in 1/[`UNIT`], uniform) mapped through its own distribution
/// (the Irwin–Hall CDF, `Σ_j (−1)^j C(n, j) (x − j)^n / n!` for `x` in units), so the result is
/// uniform again, in 1/65 536.
fn spread(draws: impl Iterator<Item = u64>) -> u32 {
    let x: i128 = draws.map(i128::from).sum();
    let n = WINDOW as u32;
    let mut sum: i128 = 0;
    let mut choose: i128 = 1; // C(n, j)
    for j in 0..=WINDOW as i128 {
        let d = x - j * UNIT;
        if d <= 0 {
            break;
        }
        let term = choose * d.pow(n);
        sum += if j % 2 == 0 { term } else { -term };
        choose = choose * (WINDOW as i128 - j) / (j + 1);
    }
    let factorial: i128 = (1..=WINDOW as i128).product();
    // In 1/65 536: sum · 65 536 / (n! · UNIT^n).
    let value = sum / (factorial * UNIT.pow(n - 1));
    value.clamp(0, 65_535) as u32
}

/// A 64-bit hash of the seed, a label, and a block number: FNV-1a over the seed's little-endian
/// bytes, a `0xff` separator, the label, and `k`'s little-endian bytes, then SplitMix64's
/// finalizer so every bit depends on every input bit.
pub(crate) fn draw(seed: u64, label: &[u8], k: i64) -> u64 {
    let bytes = (seed.to_le_bytes().into_iter())
        .chain([0xff])
        .chain(label.iter().copied())
        .chain(k.to_le_bytes());
    let mut z = bytes.fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentFiles;
    use crate::time::{DAY, MONTH};

    fn climate() -> Climate {
        crate::content::Content::load(&ContentFiles::builtin())
            .expect("content")
            .climate()
            .clone()
    }

    fn at(seconds: u64) -> GameTime {
        GameTime::from_seconds(seconds)
    }

    #[test]
    fn the_spread_is_uniform_again() {
        // Its ends and middle, exactly.
        assert_eq!(spread([0u64; WINDOW].into_iter()), 0);
        let half = UNIT as u64 / 2;
        assert_eq!(spread([half; WINDOW].into_iter()), 32_768);
        assert_eq!(spread([UNIT as u64 - 1; WINDOW].into_iter()), 65_535);
        // Over many draws, every quarter of the range comes up a quarter of the time.
        let mut quarters = [0u32; 4];
        for k in 0..40_000 {
            let value = spread((0..WINDOW as i64).map(|i| draw(9, b"test", k - i) >> 48));
            quarters[(value >> 14) as usize] += 1;
        }
        for q in quarters {
            assert!((9_600..10_400).contains(&q), "{quarters:?}");
        }
    }

    #[test]
    fn blocks_tile_time_and_start_near_every_third_hour() {
        let mut last: Option<Block> = None;
        for t in (0..20 * DAY).step_by(600) {
            let b = Block::at(5, at(t));
            assert!(b.start <= t as i64 && (t as i64) < b.end, "{t}: {b:?}");
            let len = b.end - b.start;
            assert!((BLOCK - SHIFT) as i64 <= len && len <= (BLOCK + SHIFT) as i64);
            assert!(b.start >= b.k * BLOCK as i64);
            if let Some(prev) = last
                && prev != b
            {
                assert_eq!((prev.k + 1, prev.end), (b.k, b.start), "no gaps");
            }
            last = Some(b);
        }
        // The starts are spread over the shift, not all on the hour.
        let shifts: std::collections::BTreeSet<i64> =
            (0..50).map(|k| start(5, k) - k * BLOCK as i64).collect();
        assert!(shifts.len() > 40);
    }

    #[test]
    fn the_weather_is_a_pure_function_of_seed_and_time() {
        let climate = climate();
        for t in (0..60 * DAY).step_by(4_567) {
            let a = conditions(3, at(t), &climate, None);
            assert_eq!(a, conditions(3, at(t), &climate, None));
            assert_eq!(a.weather, weather_at(3, at(t), &climate, None));
        }
        let differs = (0..200u64)
            .filter(|b| {
                let t = at(b * BLOCK + BLOCK / 2);
                weather_at(3, t, &climate, None) != weather_at(4, t, &climate, None)
            })
            .count();
        assert!(differs > 50, "other seeds, other weather ({differs})");
    }

    /// The share of each weather over many blocks of `month` (1 to 12) across seeds.
    fn shares(climate: &Climate, month: u64) -> [u32; 4] {
        let mut counts = [0u32; 4];
        for seed in 0..40 {
            for day in 0..30 {
                for b in 0..8 {
                    let t = (month - 1) * MONTH + day * DAY + b * BLOCK + BLOCK - 1;
                    counts[weather_at(seed, at(t), climate, None).index()] += 1;
                }
            }
        }
        let total: u32 = counts.iter().sum();
        counts.map(|c| c * 1000 / total)
    }

    #[test]
    fn each_month_has_its_chances() {
        let climate = climate();
        for month in 1..=12 {
            let shares = shares(&climate, month);
            let chances = climate.chances(month as usize - 1);
            for (share, chance) in shares.iter().zip(chances) {
                let want = u32::from(chance) * 10;
                assert!(
                    share.abs_diff(want) <= 40,
                    "month {month}: {shares:?} per mille, chances {chances:?}"
                );
            }
        }
        // Rainmoon is wet, Highsun dry.
        let wet = |m| {
            let s = shares(&climate, m);
            s[2] + s[3]
        };
        assert!(
            wet(4) > 2 * wet(6),
            "Rainmoon {} Highsun {}",
            wet(4),
            wet(6)
        );
    }

    #[test]
    fn weather_lasts_and_mostly_changes_by_one_step() {
        let climate = climate();
        let (mut changes, mut jumps, mut blocks) = (0, 0, 0);
        for seed in 0..20 {
            let mut last = None;
            for k in 0..2_000 {
                let w = Block::nth(seed, k).weather(seed, &climate);
                if let Some(prev) = last
                    && prev != w
                {
                    changes += 1;
                    jumps += u32::from(w.index().abs_diff(Weather::index(prev)) > 1);
                }
                last = Some(w);
                blocks += 1;
            }
        }
        // Weather changes in fewer than half of the blocks, and seldom skips a step.
        assert!(changes * 2 < blocks, "{changes} changes in {blocks} blocks");
        assert!(jumps * 6 < changes, "{jumps} jumps in {changes} changes");
    }

    #[test]
    fn clouds_change_smoothly_around_a_change_of_weather() {
        let climate = climate();
        let seed = 11;
        // The first change of weather after a few days.
        let block = (20..)
            .map(|k| Block::nth(seed, k))
            .find(|b| {
                let before = Block::nth(seed, b.k - 1).weather(seed, &climate);
                before != b.weather(seed, &climate)
            })
            .expect("the weather changes");
        let now = |t: i64| conditions(seed, at(t as u64), &climate, None);
        let half = (CHANGE / 2) as i64;
        let (from, to) = (now(block.start - 1), now(block.start));
        assert_ne!(from.weather, to.weather, "the weather changes at the start");
        let full = |t: i64| Clouds::of(climate.def(now(t).weather));
        assert_eq!(
            now(block.start - half - 1).clouds,
            full(block.start - half - 1)
        );
        assert_eq!(now(block.start + half).clouds, full(block.start + half));
        // In between, minute by minute, the clouds move a little at a time.
        let mut prev = now(block.start - half).clouds;
        for t in (block.start - half..block.start + half).step_by(60) {
            let c = now(t).clouds;
            assert!(
                c.direct.abs_diff(prev.direct) <= 20,
                "{t}: {prev:?} -> {c:?}"
            );
            prev = c;
        }
    }

    #[test]
    fn an_override_sets_the_weather_for_three_hours() {
        let climate = climate();
        let now = at(10 * DAY + 5 * HOUR);
        let over = Override::new(Weather::HeavyRain, now);
        let heavy = conditions(1, now, &climate, Some(&over));
        assert_eq!(heavy.weather, Weather::HeavyRain);
        assert_eq!(heavy.clouds, Clouds::of(climate.def(Weather::HeavyRain)));
        let later = now + Duration::hours(3);
        assert_eq!(
            conditions(1, later, &climate, Some(&over)),
            conditions(1, later, &climate, None),
            "then the weather of the day returns"
        );
    }

    #[test]
    fn clouds_dim_without_putting_out() {
        let sun = Light::new(325, 312, 280, 0);
        let sky = Light::new(58, 72, 110, 0);
        assert_eq!(Clouds::CLEAR.direct(sun), sun);
        assert_eq!(Clouds::CLEAR.sky(sky, sun), sky);
        let heavy = Clouds {
            direct: 100,
            sky: 500,
            scatter: 150,
        };
        assert_eq!(heavy.direct(sun), Light::new(33, 32, 28, 0));
        assert_eq!(
            heavy.sky(sky, sun),
            Light::new(29 + 49, 36 + 47, 55 + 42, 0)
        );
        // A glimmer stays a glimmer.
        let faint = Light::new(1, 0, 2, 0);
        assert_eq!(heavy.direct(faint), Light::new(1, 0, 1, 0));
        assert!(heavy.direct(Light::DARK).is_dark());
    }

    /// Dev: prints the weather hour by hour, a line per day (`.` clear, `o` overcast, `r` rain,
    /// `R` heavy rain), for `WARLOCK_SEED` (default 1) from day `WARLOCK_FROM` (`month,day`,
    /// default `1,1`) for `WARLOCK_DAYS` days (default 30):
    /// `cargo test -p warlock-core --lib print_weather -- --ignored --nocapture`.
    #[test]
    #[ignore = "prints the weather for tuning by eye"]
    fn print_weather() {
        let var = |name: &str| std::env::var(name).ok();
        let seed = var("WARLOCK_SEED")
            .and_then(|s| crate::rng::parse_seed(&s).ok())
            .unwrap_or(1);
        let (month, day) = var("WARLOCK_FROM")
            .and_then(|s| {
                let (m, d) = s.split_once(',')?;
                Some((m.parse().ok()?, d.parse().ok()?))
            })
            .unwrap_or((1, 1));
        let days: u64 = var("WARLOCK_DAYS")
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        let content = crate::content::Content::builtin().expect("content");
        let from = GameTime::from_calendar(1, month, day, 0, 0).expect("a valid date");
        println!("seed {seed}; hours 0 to 23");
        for d in 0..days {
            let day = from + Duration::seconds(d * DAY);
            let line: String = (0..24)
                .map(|h| {
                    let t = day + Duration::hours(h);
                    match weather_at(seed, t, content.climate(), None) {
                        Weather::Clear => '.',
                        Weather::Overcast => 'o',
                        Weather::Rain => 'r',
                        Weather::HeavyRain => 'R',
                    }
                })
                .collect();
            println!("{:>16}  {line}", content.calendar().date(day));
        }
    }

    #[test]
    fn weather_text_round_trips() {
        for w in Weather::ALL {
            assert_eq!(Weather::from_id(w.id()), Some(w));
        }
        assert_eq!(Weather::from_id("hail"), None);
        assert!(Weather::Rain.is_rain() && !Weather::Overcast.is_rain());
        assert!(Weather::Clear < Weather::HeavyRain);
    }
}

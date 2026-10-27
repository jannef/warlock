//! Alignment (`assets/data/alignment.toml`, spec §8 "Alignment", #254): the edges of the seven
//! bands and what crossing into each says, how far each kind of deed moves the score, and how a
//! good shopkeeper's prices follow the band. Optional: without it nobody has a score, everyone
//! counts as Neutral, and nothing moves (`crate::alignment` holds the score, the bands, and the
//! diminishing).

use super::{ContentError, MAX_FAVOUR};
use crate::alignment::{Alignment, Band, MAX, Score};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AlignmentFile {
    band: Vec<BandSpec>,
    shift: Shifts,
    #[serde(default)]
    shops: ShopSpec,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BandSpec {
    id: Band,
    from: i32,
    description: String,
    rise: Option<String>,
    fall: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShopSpec {
    #[serde(default)]
    favour: BTreeMap<Band, i32>,
}

/// A deed's worth by the alignment of whoever it was done to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ByAlignment {
    #[serde(default)]
    pub good: i32,
    #[serde(default)]
    pub neutral: i32,
    #[serde(default)]
    pub evil: i32,
}

impl ByAlignment {
    /// Its worth against someone of `alignment`.
    pub const fn of(&self, alignment: Alignment) -> i32 {
        match alignment {
            Alignment::Good => self.good,
            Alignment::Neutral => self.neutral,
            Alignment::Evil => self.evil,
        }
    }
}

/// How far each kind of deed moves the score before it diminishes ([`Score::shifted`]):
/// positive is good, negative evil.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shifts {
    /// Killing someone not hostile to you, by their alignment.
    pub kill_peaceful: ByAlignment,
    /// Killing a beast (a neutral creature that does not talk) not hostile to you: hunting.
    pub kill_beast: i32,
    /// Killing someone hostile to you, by their alignment.
    pub kill_hostile: ByAlignment,
    /// Attacking a shopkeeper who trades with you.
    pub attack_shopkeeper: i32,
    /// Leaving a shop with goods you have not paid for.
    pub theft: i32,
    /// Paying a shopkeeper what you owe.
    pub pay_debt: i32,
    /// Eating the corpse of one of your own race, or of a peaceful creature who talks.
    pub eat_kin: i32,
    /// Consecrating an unholy place (Consecrate).
    #[serde(default)]
    pub consecrate: i32,
}

/// One band as the file gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BandDef {
    pub band: Band,
    /// The least score in the band.
    pub from: i32,
    /// What it means, in a line (the wiki).
    pub description: String,
    /// Said when the score rises into it (none for the worst band).
    pub rise: Option<String>,
    /// Said when the score falls into it (none for the best).
    pub fall: Option<String>,
}

/// The alignment rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlignmentRules {
    /// The bands, best first ([`Band::ALL`]'s order), each from a lower score than the one
    /// before; the last from −[`MAX`].
    bands: Vec<BandDef>,
    pub shift: Shifts,
    /// What a good shopkeeper moves its prices in the player's favour, in percent, by band (a
    /// missing band 0).
    favour: BTreeMap<Band, i32>,
}

impl AlignmentRules {
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let p: AlignmentFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let ids: Vec<Band> = p.band.iter().map(|b| b.id).collect();
        if ids != Band::ALL {
            return Err(err(
                "the bands are saintly, good, kind, neutral, callous, evil, wicked, in that order"
                    .into(),
            ));
        }
        let mut above = MAX + 1;
        for spec in &p.band {
            let id = spec.id.id();
            if spec.from >= above || spec.from < -MAX {
                return Err(err(format!(
                    "band `{id}`: `from` is -{MAX} to {MAX} and below the band before's"
                )));
            }
            above = spec.from;
            let (best, worst) = (spec.id == Band::Saintly, spec.id == Band::Wicked);
            if spec.rise.is_some() == worst || spec.fall.is_some() == best {
                return Err(err(format!(
                    "band `{id}`: every band but the worst says something as the score rises \
                     into it (`rise`), and every band but the best as it falls into it (`fall`)"
                )));
            }
        }
        if above != -MAX {
            return Err(err(format!("band `wicked` is `from` -{MAX}")));
        }
        let s = &p.shift;
        let amounts = [
            s.kill_peaceful.good,
            s.kill_peaceful.neutral,
            s.kill_peaceful.evil,
            s.kill_beast,
            s.kill_hostile.good,
            s.kill_hostile.neutral,
            s.kill_hostile.evil,
            s.attack_shopkeeper,
            s.theft,
            s.pay_debt,
            s.eat_kin,
            s.consecrate,
        ];
        if amounts.iter().any(|a| a.abs() > MAX) {
            return Err(err(format!("every [shift] is -{MAX} to {MAX}")));
        }
        if p.shops.favour.values().any(|f| f.abs() > MAX_FAVOUR) {
            return Err(err(format!(
                "[shops] favour is -{MAX_FAVOUR} to {MAX_FAVOUR} percent"
            )));
        }
        let bands = (p.band.into_iter())
            .map(|b| BandDef {
                band: b.id,
                from: b.from,
                description: b.description,
                rise: b.rise,
                fall: b.fall,
            })
            .collect();
        Ok(Self {
            bands,
            shift: p.shift,
            favour: p.shops.favour,
        })
    }

    /// The bands, best first.
    pub fn bands(&self) -> &[BandDef] {
        &self.bands
    }

    /// The band `score` falls in.
    pub fn band(&self, score: Score) -> Band {
        (self.bands.iter())
            .find(|b| score.value() >= b.from)
            .map_or(Band::Wicked, |b| b.band)
    }

    /// The definition of `band`.
    pub fn def(&self, band: Band) -> &BandDef {
        (self.bands.iter())
            .find(|b| b.band == band)
            .expect("every band is defined")
    }

    /// What crossing from band `from` into band `to` says: `to`'s `rise` or `fall`; nothing
    /// within a band.
    pub fn news(&self, from: Band, to: Band) -> Option<&str> {
        let def = self.def(to);
        match to.cmp(&from) {
            std::cmp::Ordering::Greater => def.rise.as_deref(),
            std::cmp::Ordering::Less => def.fall.as_deref(),
            std::cmp::Ordering::Equal => None,
        }
    }

    /// How far a good shopkeeper moves its prices in the favour of a customer of `band`, in
    /// percent (spec §7 "Shops").
    pub fn favour(&self, band: Band) -> i32 {
        self.favour.get(&band).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = include_str!("../../../../assets/data/alignment.toml");

    fn rules() -> AlignmentRules {
        AlignmentRules::parse("data/alignment.toml", FILE).expect("parses")
    }

    #[test]
    fn the_builtin_bands_cover_the_scale() {
        let r = rules();
        assert_eq!(r.band(Score::new(-MAX)), Band::Wicked);
        assert_eq!(r.band(Score::new(MAX)), Band::Saintly);
        assert_eq!(r.band(Score::ZERO), Band::Neutral);
        // Every band holds some score, and they follow each other.
        let mut last = Band::Wicked;
        for v in -MAX..=MAX {
            let b = r.band(Score::new(v));
            assert!(b >= last, "{v}: {b:?} after {last:?}");
            last = b;
        }
        for def in r.bands() {
            assert_eq!(r.band(Score::new(def.from)), def.band);
        }
    }

    #[test]
    fn crossing_a_band_says_the_band_entered() {
        let r = rules();
        assert_eq!(r.news(Band::Good, Band::Good), None);
        assert_eq!(
            r.news(Band::Good, Band::Kind),
            r.def(Band::Kind).fall.as_deref()
        );
        assert_eq!(
            r.news(Band::Kind, Band::Good),
            r.def(Band::Good).rise.as_deref()
        );
        assert!(r.news(Band::Wicked, Band::Saintly).is_some());
    }

    #[test]
    fn bad_files_are_refused() {
        let good = FILE;
        let cases = [
            (
                good.replace("from = 750", "from = 1001"),
                "below the band before",
            ),
            (good.replace("from = -1000", "from = -900"), "`from` -1000"),
            (
                good.replacen("id = \"kind\"", "id = \"good\"", 1),
                "in that order",
            ),
            (good.replace("theft = -100", "theft = -1001"), "[shift]"),
            (good.replace("saintly = 10", "saintly = 51"), "favour"),
        ];
        for (text, expected) in cases {
            assert_ne!(text, good, "the case changes the file: {expected}");
            let e = AlignmentRules::parse("data/alignment.toml", &text).unwrap_err();
            assert!(e.message.contains(expected), "{e} lacks {expected}");
        }
    }
}

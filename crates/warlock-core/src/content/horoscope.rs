//! The horoscope (`assets/data/horoscope.toml`, spec §8 "Character creation"): the sign a
//! character is born under, from their birth year.
//!
//! Every year has an **animal** and an **element**. Both advance one step a year, in file order,
//! on the 1st of the first month (Frostmere), so with 12 animals and 5 elements every pair comes
//! exactly once in a **great cycle** of 60 years. The file's `anchor_year` starts a great cycle:
//! it is the year of the first animal and the first element. A sign adjusts the starting scores
//! (the animal's and the element's adjustments both apply), and it is a flag later rules may ask
//! about ([`Sign::is_element`], [`Sign::is_animal`]).

use super::ContentError;
use super::kit::{Kit, KitLineSpec};
use crate::dice::Dice;
use crate::stats::{PerStat, Stat, adjustments};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HoroscopeFile {
    anchor_year: i64,
    #[serde(default)]
    animal: Vec<SignSpec>,
    #[serde(default)]
    element: Vec<SignSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SignSpec {
    id: String,
    name: String,
    description: String,
    #[serde(default)]
    stats: BTreeMap<Stat, Dice>,
    #[serde(default)]
    kit: Vec<KitLineSpec>,
}

/// An animal or an element of the horoscope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignPart {
    pub id: String,
    pub name: String,
    /// One line for the wiki: what those born under it are like.
    pub description: String,
    /// Adjustments to the starting scores of those born under it.
    pub stats: PerStat<Dice>,
    /// The starting kit as written; resolved by the loader into `kit`.
    pub(super) kit_spec: Vec<KitLineSpec>,
    /// What those born under it add to their starting kit, after the class's and the race's
    /// (spec §8 "Character creation"; [`super::kit`]); filled in by the loader.
    pub kit: Kit,
}

/// The animals and elements, in the order the years go through them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Horoscope {
    /// A year that starts a great cycle: the first animal's and the first element's.
    pub anchor_year: i64,
    pub animals: Vec<SignPart>,
    pub elements: Vec<SignPart>,
}

/// A birth sign: the animal and the element of a year, by id. A flag for the rules: "is this
/// character Fire-born?" is `sign.is_element("fire")`, "born in the year of the Manticore?"
/// `sign.is_animal("manticore")`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sign {
    pub animal: String,
    pub element: String,
}

impl Sign {
    /// Whether this sign's element is `element` (an id: `fire`).
    pub fn is_element(&self, element: &str) -> bool {
        self.element == element
    }

    /// Whether this sign's animal is `animal` (an id: `manticore`).
    pub fn is_animal(&self, animal: &str) -> bool {
        self.animal == animal
    }
}

impl Horoscope {
    /// Parses `data/horoscope.toml`.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: HoroscopeFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let parts = |kind: &str, specs: Vec<SignSpec>| -> Result<Vec<SignPart>, ContentError> {
            if specs.is_empty() {
                return Err(err(format!("at least one {kind} is required")));
            }
            let mut list: Vec<SignPart> = Vec::new();
            for spec in specs {
                let id = &spec.id;
                if list.iter().any(|p| &p.id == id) {
                    return Err(err(format!("duplicate {kind} id `{id}`")));
                }
                let stats = adjustments(&spec.stats)
                    .map_err(|m| err(format!("{kind} `{id}`: stats: {m}")))?;
                list.push(SignPart {
                    id: spec.id,
                    name: spec.name,
                    description: spec.description,
                    stats,
                    kit_spec: spec.kit,
                    kit: Kit::default(),
                });
            }
            Ok(list)
        };
        let animals = parts("animal", parsed.animal)?;
        let elements = parts("element", parsed.element)?;
        if gcd(animals.len(), elements.len()) != 1 {
            return Err(err(format!(
                "{} animals and {} elements would not give every pair once a great cycle: \
                 their numbers must have no common divisor",
                animals.len(),
                elements.len()
            )));
        }
        Ok(Self {
            anchor_year: parsed.anchor_year,
            animals,
            elements,
        })
    }

    /// The length of a great cycle in years: every animal–element pair once (60 for 12 animals
    /// and 5 elements).
    pub fn great_cycle(&self) -> usize {
        self.animals.len() * self.elements.len()
    }

    /// The animal and the element of `year` (any year, also before the calendar's first).
    pub fn parts_of_year(&self, year: i64) -> (&SignPart, &SignPart) {
        let since = year - self.anchor_year;
        let at = |n: usize| since.rem_euclid(n as i64) as usize;
        (
            &self.animals[at(self.animals.len())],
            &self.elements[at(self.elements.len())],
        )
    }

    /// The sign of `year`: of those born in it.
    pub fn sign_of_year(&self, year: i64) -> Sign {
        let (animal, element) = self.parts_of_year(year);
        Sign {
            animal: animal.id.clone(),
            element: element.id.clone(),
        }
    }

    /// The animal and the element of `sign`; `None` if this horoscope lacks either.
    pub fn parts(&self, sign: &Sign) -> Option<(&SignPart, &SignPart)> {
        let animal = self.animals.iter().find(|a| a.id == sign.animal)?;
        let element = self.elements.iter().find(|e| e.id == sign.element)?;
        Some((animal, element))
    }

    /// The sign as players say it: element, then animal (`Fire Manticore`).
    pub fn name(&self, sign: &Sign) -> String {
        match self.parts(sign) {
            Some((animal, element)) => format!("{} {}", element.name, animal.name),
            None => format!("{} {}", sign.element, sign.animal),
        }
    }
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn part(kind: &str, id: &str) -> String {
        format!("[[{kind}]]\nid = \"{id}\"\nname = \"{id}\"\ndescription = \"\"\n")
    }

    /// A horoscope with `animals` animals `a0`… and `elements` elements `e0`….
    fn text(anchor: i64, animals: usize, elements: usize) -> String {
        let mut text = format!("anchor_year = {anchor}\n");
        for i in 0..animals {
            text += &part("animal", &format!("a{i}"));
        }
        for i in 0..elements {
            text += &part("element", &format!("e{i}"));
        }
        text
    }

    fn parse(text: &str) -> Result<Horoscope, ContentError> {
        Horoscope::parse("horoscope.toml", text)
    }

    #[test]
    fn every_pair_comes_once_a_great_cycle_from_the_anchor() {
        let h = parse(&text(1441, 12, 5)).expect("valid");
        assert_eq!(h.great_cycle(), 60);
        // The anchor year has the first animal and the first element; each advances a year.
        assert_eq!(
            h.sign_of_year(1441),
            Sign {
                animal: "a0".into(),
                element: "e0".into()
            }
        );
        assert_eq!(
            h.sign_of_year(1442),
            Sign {
                animal: "a1".into(),
                element: "e1".into()
            }
        );
        assert_eq!(
            h.sign_of_year(1446),
            Sign {
                animal: "a5".into(),
                element: "e0".into()
            }
        );
        assert_eq!(
            h.sign_of_year(1453),
            Sign {
                animal: "a0".into(),
                element: "e2".into()
            }
        );
        // Sixty years later (or earlier) it all repeats, also before year 1.
        for year in [-200, -1, 0, 1, 2, 1381, 1440, 1441, 1444, 1500, 1501, 9999] {
            assert_eq!(h.sign_of_year(year), h.sign_of_year(year + 60), "{year}");
            assert_eq!(h.sign_of_year(year), h.sign_of_year(year - 60), "{year}");
        }
        let cycle: BTreeSet<Sign> = (1441..1501).map(|y| h.sign_of_year(y)).collect();
        assert_eq!(cycle.len(), 60, "every pair once in sixty years");
        let shifted: BTreeSet<Sign> = (-17..43).map(|y| h.sign_of_year(y)).collect();
        assert_eq!(shifted, cycle, "any sixty years in a row");
        // The anchor is any first year of a great cycle: 1 and 1441 are the same cycle.
        let one = parse(&text(1, 12, 5)).expect("valid");
        for year in -100..1600 {
            assert_eq!(one.sign_of_year(year), h.sign_of_year(year));
        }
    }

    #[test]
    fn signs_answer_whether_they_are_an_element_or_an_animal() {
        let sign = Sign {
            animal: "manticore".into(),
            element: "fire".into(),
        };
        assert!(sign.is_element("fire") && !sign.is_element("water"));
        assert!(sign.is_animal("manticore") && !sign.is_animal("roc"));
        let h = parse(&format!(
            "anchor_year = 1\n{}name = \"Manticore\"\n{}",
            "[[animal]]\nid = \"manticore\"\ndescription = \"\"\nstats = { strength = 1 }\n",
            "[[element]]\nid = \"fire\"\nname = \"Fire\"\ndescription = \"\"\n"
        ))
        .expect("valid");
        assert_eq!(h.name(&sign), "Fire Manticore");
        assert_eq!(h.animals[0].stats[Stat::Strength], Dice::fixed(1));
        assert_eq!(h.sign_of_year(1444), sign);
        let unknown = Sign {
            animal: "roc".into(),
            element: "fire".into(),
        };
        assert!(h.parts(&unknown).is_none());
        let json = serde_json::to_string(&sign).expect("json");
        assert_eq!(json, r#"{"animal":"manticore","element":"fire"}"#);
    }

    #[test]
    fn bad_horoscopes_are_refused() {
        for (text, why) in [
            (text(1, 12, 4), "12 animals and 4 elements"),
            (text(1, 6, 3), "no common divisor"),
            (text(1, 0, 5), "at least one animal"),
            (text(1, 12, 0), "at least one element"),
            (
                format!("{}{}", text(1, 1, 1), part("animal", "a0")),
                "duplicate animal id `a0`",
            ),
            (
                format!("{}stats = {{ luck = 1 }}\n", text(1, 1, 1)),
                "unknown stat `luck`",
            ),
            (
                format!("{}stats = {{ presence = 11 }}\n", text(1, 1, 1)),
                "element `e0`: stats: presence +11 is out of range",
            ),
            (
                text(1, 1, 1).replace("anchor_year = 1\n", ""),
                "missing field `anchor_year`",
            ),
        ] {
            let e = parse(&text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
        // One animal and one element is a (dull) horoscope.
        assert_eq!(parse(&text(1, 1, 1)).expect("valid").great_cycle(), 1);
    }
}

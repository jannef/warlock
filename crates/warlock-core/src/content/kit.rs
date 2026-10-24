//! Starting kits (spec §8 "Character creation"): what a new character carries, wields, and wears
//! at the start of a run. A class's `kit` (`data/classes.toml`) comes first, then the race's
//! (`data/races.toml`), then their birth sign's animal's and element's (`data/horoscope.toml`;
//! a hook, empty so far). [`crate::kit`] rolls one for a character.
//!
//! A kit is a list of lines, each one item (or a few of one, `"ration*2"`) as level files place
//! objects. A line is a string, an item carried, or a table:
//!
//! - `carry`, `wield`, or `wear` (exactly one): the item, carried in the pack, or wielded or
//!   readied (weapons, shields, missile weapons and missiles, tools) or worn (armour, cloaks,
//!   rings, amulets), each in the first free slot it fits;
//! - alternatives `"club|quarterstaff"`: one of them, rolled when the character is made;
//! - `trained`: only for a character whose class trained this weapon skill or fighting style (its
//!   fixed ranks, or the set of its choice that was picked): the barbarian's great axe goes with
//!   axes, the thief's second dagger with Twin Blades;
//! - `gender`: only for that gender (the hook for gender's items);
//! - `magic`: a base item made as loot makes it, its rarity rolled (`any`: perhaps magic,
//!   `magic`: magic or rare, `rare`), unidentified like any other.

use super::{Content, WeaponSkillId, object};
use crate::character::Gender;
use crate::content::ActionKind;
use crate::equipment::{self, Equipment};
use crate::magic::LootGrade;
use crate::object::Stack;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

/// How a kit item is kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keep {
    /// In the pack.
    Carry,
    /// Wielded or readied: in a hand, the missile weapon or missiles slot, or the Tool slot.
    Wield,
    /// Worn.
    Wear,
}

impl Keep {
    /// The field it is written with: `carry`, `wield`, or `wear`.
    pub const fn id(self) -> &'static str {
        match self {
            Keep::Carry => "carry",
            Keep::Wield => "wield",
            Keep::Wear => "wear",
        }
    }
}

/// One line of a kit as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KitLineSpec {
    keep: Keep,
    /// Placements, `|` between alternatives.
    items: String,
    trained: Option<String>,
    gender: Option<Gender>,
    magic: Option<LootGrade>,
}

/// A line written as a table.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KitTable {
    carry: Option<String>,
    wield: Option<String>,
    wear: Option<String>,
    trained: Option<String>,
    gender: Option<Gender>,
    magic: Option<LootGrade>,
}

impl<'de> Deserialize<'de> for KitLineSpec {
    /// A string (an item carried) or a table.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Line;
        impl<'de> Visitor<'de> for Line {
            type Value = KitLineSpec;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(
                    "an item carried (`\"ration*2\"`) or a table (`{ wield = \"dagger\" }`)",
                )
            }

            fn visit_str<E: de::Error>(self, items: &str) -> Result<KitLineSpec, E> {
                Ok(KitLineSpec {
                    keep: Keep::Carry,
                    items: items.to_string(),
                    trained: None,
                    gender: None,
                    magic: None,
                })
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<KitLineSpec, A::Error> {
                let t = KitTable::deserialize(de::value::MapAccessDeserializer::new(map))?;
                let kept = [
                    (Keep::Carry, t.carry),
                    (Keep::Wield, t.wield),
                    (Keep::Wear, t.wear),
                ];
                let mut given = kept.into_iter().filter_map(|(k, i)| Some((k, i?)));
                let (Some((keep, items)), None) = (given.next(), given.next()) else {
                    return Err(de::Error::custom(
                        "a kit line names its item once, with `carry`, `wield`, or `wear`",
                    ));
                };
                Ok(KitLineSpec {
                    keep,
                    items,
                    trained: t.trained,
                    gender: t.gender,
                    magic: t.magic,
                })
            }
        }
        deserializer.deserialize_any(Line)
    }
}

/// One line of a kit: one of its alternatives, kept so, if its conditions hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KitItem {
    pub keep: Keep,
    /// The alternatives, one rolled when there are several; each the stacks its placement makes
    /// (one, but for several of a thing that does not stack, which are only carried).
    pub one_of: Vec<Vec<Stack>>,
    /// Only for a character whose class trained this weapon skill or fighting style.
    pub trained: Option<WeaponSkillId>,
    /// Only for this gender.
    pub gender: Option<Gender>,
    /// A base item made magic as loot is, at least this rare.
    pub magic: Option<LootGrade>,
}

impl KitItem {
    /// Whether it comes to a character of `gender` whose class trained `trained`.
    pub fn applies(&self, gender: Gender, trained: &[WeaponSkillId]) -> bool {
        self.gender.is_none_or(|g| g == gender) && self.trained.is_none_or(|t| trained.contains(&t))
    }
}

/// A starting kit: its lines in order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Kit(pub Vec<KitItem>);

impl Kit {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn items(&self) -> &[KitItem] {
        &self.0
    }
}

/// Resolves the lines `specs` against `content` (its objects and weapon skills). `trainable` is
/// what a class's kit's `trained` may name (what the class trains, or may by its choices); `None`
/// for a race's or a sign's, which may name any weapon skill or style.
pub(super) fn resolve(
    specs: &[KitLineSpec],
    content: &Content,
    trainable: Option<&[WeaponSkillId]>,
) -> Result<Kit, String> {
    let mut kit = Vec::new();
    for spec in specs {
        let at = |m: String| format!("kit `{}`: {m}", spec.items);
        let one_of = (spec.items.split('|'))
            .map(|placed| object::parse_stacks(&[placed.trim().to_string()], content))
            .collect::<Result<Vec<_>, _>>()
            .map_err(at)?;
        for stacks in &one_of {
            check_kept(spec, stacks, content).map_err(at)?;
        }
        let trained = match &spec.trained {
            None => None,
            Some(name) => {
                let id = (content.weapon_skills().id(name)).ok_or_else(|| {
                    at(format!("`trained`: unknown weapon skill or style `{name}`"))
                })?;
                if trainable.is_some_and(|t| !t.contains(&id)) {
                    return Err(at(format!(
                        "`trained`: the class never trains `{name}` (its weapon skills, styles, \
                         and their choices)"
                    )));
                }
                Some(id)
            }
        };
        kit.push(KitItem {
            keep: spec.keep,
            one_of,
            trained,
            gender: spec.gender,
            magic: spec.magic,
        });
    }
    Ok(Kit(kit))
}

/// Whether one alternative of `spec`, `stacks`, may be kept as it says: what is wielded or worn
/// is one stack that goes in a slot of that kind, and what is made magic is a base item.
fn check_kept(spec: &KitLineSpec, stacks: &[Stack], content: &Content) -> Result<(), String> {
    for stack in stacks {
        let def = content.object(stack.object);
        if spec.magic.is_some() && def.kind.is_none() {
            return Err(format!(
                "`magic`: `{}` is never magic (it has no `kind`)",
                def.id
            ));
        }
        if spec.keep == Keep::Carry {
            continue;
        }
        if stacks.len() > 1 {
            return Err(format!(
                "`{}`: wield or wear one at a time (it does not stack)",
                spec.keep.id()
            ));
        }
        let slot = equipment::free_slot(content, &Equipment::default(), stack.object)
            .ok_or_else(|| format!("`{}` cannot be worn or wielded", def.id))?;
        let held = matches!(
            content.slot(slot).action,
            ActionKind::Wield | ActionKind::Ready
        );
        if held != (spec.keep == Keep::Wield) {
            let (is, other) = if held {
                ("wielded, not worn", "wield")
            } else {
                ("worn, not wielded", "wear")
            };
            return Err(format!("`{}` is {is}: use `{other}`", def.id));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::LazyLock;

    static CONTENT: LazyLock<Content> =
        LazyLock::new(|| Content::builtin().expect("built-in content"));

    #[derive(Deserialize)]
    struct Lines {
        kit: Vec<KitLineSpec>,
    }

    /// The kit written in `text` (`kit = [...]`), resolved; `trainable` as for a class.
    fn kit(text: &str, trainable: Option<&[&str]>) -> Result<Kit, String> {
        let lines: Lines = toml::from_str(text).map_err(|e| e.to_string())?;
        let ids: Option<Vec<WeaponSkillId>> = trainable.map(|names| {
            (names.iter())
                .map(|n| CONTENT.weapon_skills().id(n).expect("a weapon skill"))
                .collect()
        });
        resolve(&lines.kit, &CONTENT, ids.as_deref())
    }

    #[test]
    fn lines_are_items_carried_or_tables() {
        let kit = kit(
            r#"kit = [
                "ration*2",
                { wield = "dagger", trained = "twin_blades" },
                { wear = "gold_ring", magic = "magic", gender = "female" },
                "club|quarterstaff",
                { carry = "dagger*2" },
            ]"#,
            None,
        )
        .expect("valid");
        let [ration, dagger, ring, stick, daggers] = kit.items() else {
            panic!("{kit:?}");
        };
        assert_eq!((ration.keep, ration.one_of.len()), (Keep::Carry, 1));
        assert_eq!(ration.one_of[0][0].count, 2);
        assert_eq!(dagger.keep, Keep::Wield);
        assert_eq!(dagger.trained, CONTENT.weapon_skills().id("twin_blades"));
        assert_eq!((ring.keep, ring.gender), (Keep::Wear, Some(Gender::Female)));
        assert_eq!(ring.magic, Some(LootGrade::Magic));
        assert_eq!(stick.one_of.len(), 2);
        // Two of what does not stack: two stacks, carried.
        assert_eq!(daggers.one_of[0].len(), 2);
        let twin = CONTENT.weapon_skills().id("twin_blades").unwrap();
        assert!(dagger.applies(Gender::Male, &[twin]) && !dagger.applies(Gender::Male, &[]));
        assert!(ring.applies(Gender::Female, &[]) && !ring.applies(Gender::Other, &[]));
    }

    #[test]
    fn bad_lines_are_refused() {
        for (line, why) in [
            (
                r#"{ wield = "dagger", wear = "dagger" }"#,
                "names its item once",
            ),
            (r#"{ trained = "axes" }"#, "names its item once"),
            (r#"{ wield = "dagger", colour = "red" }"#, "unknown field"),
            (r#"{ wield = "dagger", gender = "any" }"#, "unknown variant"),
            (r#""spork""#, "unknown object `spork`"),
            (r#""dagger|spork""#, "unknown object `spork`"),
            (r#""kobold_corpse""#, "corpse"),
            (
                r#"{ wield = "ration" }"#,
                "`ration` cannot be worn or wielded",
            ),
            (r#"{ wear = "dagger" }"#, "wielded, not worn: use `wield`"),
            (
                r#"{ wield = "leather_jerkin" }"#,
                "worn, not wielded: use `wear`",
            ),
            (r#"{ wield = "dagger*2" }"#, "one at a time"),
            (
                r#"{ carry = "ration", magic = "any" }"#,
                "`ration` is never magic",
            ),
            (
                r#"{ carry = "dagger", magic = "legendary" }"#,
                "unknown variant",
            ),
            (
                r#"{ carry = "dagger", trained = "lances" }"#,
                "unknown weapon skill",
            ),
            (r#"12"#, "an item carried"),
        ] {
            let err = kit(&format!("kit = [{line}]"), None).expect_err(line);
            assert!(err.contains(why), "{line}: {err}");
        }
        // A class's kit may only follow what the class trains.
        let line = r#"kit = [{ wield = "short_bow", trained = "bows" }]"#;
        let err = kit(line, Some(&["daggers", "twin_blades"])).expect_err("not trained");
        assert!(err.contains("the class never trains `bows`"), "{err}");
        assert!(kit(line, Some(&["bows"])).is_ok() && kit(line, None).is_ok());
    }

    #[test]
    fn every_class_and_race_has_a_kit() {
        assert!(CONTENT.classes().iter().all(|c| !c.kit.is_empty()));
        assert!(CONTENT.races().list.iter().all(|r| !r.kit.is_empty()));
        // The birth signs give nothing yet: the hook is there for the horoscope.
        let h = CONTENT.horoscope().expect("a horoscope");
        assert!(
            h.animals
                .iter()
                .chain(&h.elements)
                .all(|p| p.kit.is_empty())
        );
    }
}

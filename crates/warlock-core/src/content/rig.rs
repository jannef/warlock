//! Rigs (`assets/data/rigs.toml`, spec §8 "Traps": "Rigging traps"; the craftsperson design
//! §7.6): the traps the craftsperson's Trapwright rigs on a cell beside them, what each takes,
//! how long it takes, and how often the trap it places goes off.
//!
//! The rule itself lives in `world/rig.rs`. The file is optional: without it Trapwright is "not
//! usable yet" (`Unavailable::NotYet`).

use super::{Content, ContentError, TrapId};
use crate::skills::SkillId;
use serde::Deserialize;

/// A rig's place in [`RigRules::rigs`] (file order).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RigId(pub u8);

/// The rules of rigging that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RigRules {
    /// What the check must reach.
    pub difficulty: i32,
    /// The skill whose `craft` points add to the check (Woodcraft): a trained wright rigs
    /// better, but anyone with the talent rigs.
    pub skill: SkillId,
    /// The rigs, in file order.
    pub rigs: Vec<RigDef>,
}

/// One rig of the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RigDef {
    pub id: String,
    /// What the list calls it ("Tripwire").
    pub name: String,
    /// The trap it places.
    pub trap: TrapId,
    /// How long the work takes, in seconds of game time.
    pub seconds: u32,
    /// What it takes.
    pub takes: RigTakes,
    /// Whether a knife must be carried (kept).
    pub knife: bool,
    /// How near a wall must stand, in cells along one of the eight directions; `None`: no wall
    /// is asked for.
    pub wall: Option<u8>,
    /// Said when it is rigged.
    pub made: String,
    /// Said when a natural 2 springs it on the rigger.
    pub snaps: String,
}

/// What a rig takes, and so how often its trap goes off.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RigTakes {
    /// Ammunition of `kind` (objects' `ammo`): up to `most` of one stack, spent, a charge each.
    Ammo { kind: String, most: u16 },
    /// One piece of stock of material group `group`, spent; the trap has `charges`.
    Stock { group: String, charges: u16 },
    /// A digging tool ready in the Tool slot, kept; the trap has `charges`.
    DigTool { charges: u16 },
}

impl RigRules {
    /// The rig of id `id`.
    pub fn id(&self, id: &str) -> Option<RigId> {
        let i = self.rigs.iter().position(|r| r.id == id)?;
        u8::try_from(i).ok().map(RigId)
    }

    /// The rig `id` is.
    pub fn rig(&self, id: RigId) -> &RigDef {
        &self.rigs[usize::from(id.0)]
    }

    /// Every rig with its id, in file order.
    pub fn iter(&self) -> impl Iterator<Item = (RigId, &RigDef)> {
        (self.rigs.iter().enumerate()).map(|(i, r)| (RigId(i as u8), r))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RigsFile {
    difficulty: i32,
    skill: String,
    rig: Vec<RigSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RigSpec {
    id: String,
    name: String,
    trap: String,
    seconds: u32,
    #[serde(default)]
    ammo: Option<String>,
    #[serde(default)]
    most: Option<u16>,
    #[serde(default)]
    stock: Option<String>,
    #[serde(default)]
    dig_tool: bool,
    #[serde(default)]
    charges: Option<u16>,
    #[serde(default)]
    knife: bool,
    #[serde(default)]
    wall: Option<u8>,
    made: String,
    snaps: String,
}

impl RigRules {
    /// Parses `data/rigs.toml` against `content`'s traps, objects, and stock.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: RigsFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if parsed.rig.is_empty() {
            return Err(err("there must be at least one `[[rig]]`".into()));
        }
        if parsed.rig.len() > usize::from(u8::MAX) {
            return Err(err("too many rigs".into()));
        }
        let mut rigs: Vec<RigDef> = Vec::with_capacity(parsed.rig.len());
        for spec in parsed.rig {
            let rig = spec.resolve(content).map_err(err)?;
            if rigs.iter().any(|r| r.id == rig.id) {
                return Err(err(format!("duplicate rig id `{}`", rig.id)));
            }
            rigs.push(rig);
        }
        let skill = (content.skill_id(&parsed.skill))
            .ok_or_else(|| err(format!("no skill `{}`", parsed.skill)))?;
        Ok(Self {
            difficulty: parsed.difficulty,
            skill,
            rigs,
        })
    }
}

impl RigSpec {
    fn resolve(self, content: &Content) -> Result<RigDef, String> {
        let id = self.id;
        let trap = (content.trap_id(&self.trap))
            .ok_or_else(|| format!("rig `{id}`: no trap `{}`", self.trap))?;
        if self.seconds == 0 {
            return Err(format!("rig `{id}`: `seconds` must be at least 1"));
        }
        let count = |n: Option<u16>, field: &str| match n {
            Some(n @ 1..=100) => Ok(n),
            Some(_) => Err(format!("rig `{id}`: `{field}` must be 1 to 100")),
            None => Err(format!("rig `{id}` needs `{field}`")),
        };
        let takes = match (self.ammo, self.stock, self.dig_tool) {
            (Some(kind), None, false) => {
                if self.charges.is_some() {
                    return Err(format!(
                        "rig `{id}`: an `ammo` rig has a charge for each spent, no `charges`"
                    ));
                }
                if !(content.object_defs().iter()).any(|o| o.ammo.as_deref() == Some(&kind)) {
                    return Err(format!("rig `{id}`: no object is ammunition `{kind}`"));
                }
                RigTakes::Ammo {
                    kind,
                    most: count(self.most, "most")?,
                }
            }
            (None, Some(group), false) => {
                if content.stock().group(&group).is_none() {
                    return Err(format!("rig `{id}`: no stock group `{group}`"));
                }
                RigTakes::Stock {
                    group,
                    charges: count(self.charges, "charges")?,
                }
            }
            (None, None, true) => RigTakes::DigTool {
                charges: count(self.charges, "charges")?,
            },
            _ => {
                return Err(format!(
                    "rig `{id}` takes exactly one of `ammo`, `stock`, and `dig_tool`"
                ));
            }
        };
        if self.most.is_some() && !matches!(takes, RigTakes::Ammo { .. }) {
            return Err(format!("rig `{id}`: `most` is for an `ammo` rig"));
        }
        if self.wall.is_some_and(|w| !(1..=10).contains(&w)) {
            return Err(format!("rig `{id}`: `wall` must be 1 to 10"));
        }
        Ok(RigDef {
            id,
            name: self.name,
            trap,
            seconds: self.seconds,
            takes,
            knife: self.knife,
            wall: self.wall,
            made: self.made,
            snaps: self.snaps,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentFiles;

    #[test]
    fn the_builtin_rigs_load() {
        let content = Content::builtin().expect("content");
        let rules = content.rigs().expect("rigs.toml");
        let ids: Vec<&str> = rules.rigs.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["tripwire", "stakes", "pitfall"]);
        let tripwire = rules.rig(rules.id("tripwire").expect("tripwire"));
        assert_eq!(content.trap(tripwire.trap).id, "rigged_arrows");
        assert_eq!(
            tripwire.takes,
            RigTakes::Ammo {
                kind: "arrow".into(),
                most: 3
            }
        );
        assert_eq!(tripwire.wall, Some(2));
        let stakes = rules.rig(rules.id("stakes").expect("stakes"));
        assert!(stakes.knife);
        assert_eq!(
            stakes.takes,
            RigTakes::Stock {
                group: "wood".into(),
                charges: 2
            }
        );
        let pitfall = rules.rig(rules.id("pitfall").expect("pitfall"));
        assert_eq!(pitfall.takes, RigTakes::DigTool { charges: 3 });
        assert_eq!(rules.difficulty, 10);
    }

    #[test]
    fn faults_are_refused() {
        let refused = |rig: &str, words: &str| {
            let mut files = ContentFiles::builtin();
            let text = format!(
                "difficulty = 10\nskill = \"woodcraft\"\n[[rig]]\nid = \"odd\"\nname = \"Odd\"\nmade = \"m\"\n\
                 snaps = \"s\"\n{rig}"
            );
            files.insert("data/rigs.toml", &text);
            let e = Content::load(&files).expect_err(rig).to_string();
            assert!(e.contains(words), "{rig}: {e}");
        };
        let pit = "trap = \"rigged_pit\"\nseconds = 60\n";
        refused(
            "trap = \"nothing\"\nseconds = 60\ndig_tool = true\ncharges = 1\n",
            "no trap",
        );
        refused(pit, "exactly one");
        refused(&format!("{pit}dig_tool = true\n"), "needs `charges`");
        refused(&format!("{pit}dig_tool = true\ncharges = 0\n"), "1 to 100");
        refused(
            &format!("{pit}dig_tool = true\nstock = \"wood\"\n"),
            "exactly one",
        );
        refused(&format!("{pit}ammo = \"pebble\"\nmost = 2\n"), "no object");
        refused(
            &format!("{pit}ammo = \"arrow\"\nmost = 2\ncharges = 2\n"),
            "no `charges`",
        );
        refused(
            &format!("{pit}stock = \"nothing\"\ncharges = 2\n"),
            "no stock group",
        );
        refused(
            &format!("{pit}dig_tool = true\ncharges = 1\nmost = 2\n"),
            "`most`",
        );
        refused(
            &format!("{pit}dig_tool = true\ncharges = 1\nwall = 0\n"),
            "`wall`",
        );
        refused(
            "trap = \"rigged_pit\"\nseconds = 0\ndig_tool = true\ncharges = 1\n",
            "`seconds`",
        );
    }

    #[test]
    fn without_the_file_there_are_no_rigs() {
        let mut files = ContentFiles::builtin();
        files.remove("data/rigs.toml");
        let content = Content::load(&files).expect("content");
        assert!(content.rigs().is_none());
    }
}

//! Alternative resources (`assets/data/resources.toml`, spec §8 "Resources", #225): integer
//! pools beside hit points and mana that a character holds once something grants them (a class
//! ability, a talent), each with its own rules: it may regenerate toward its maximum or decay
//! toward 0 over time, build up from events (a kill, a hit, a critical, being hit, a sneak
//! attack), be spent by abilities, and end at zero, taking its bar with it until something
//! grants it again.
//!
//! The list is ordered: a [`ResourceId`] is an index into it, in the content fingerprint (the
//! pools an actor holds are saved by it), so a new resource is a new `[[resource]]` entry, never
//! a reordering. Without the file there are none.

use super::ContentError;
use serde::{Deserialize, Serialize};

/// Index of a resource in [`super::Content::resources`], in file order (in the content
/// fingerprint).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ResourceId(pub u16);

/// The largest maximum a resource may have, and the most an event may build.
pub const MAX_RESOURCE: u16 = 1000;

/// The longest a point may take to come back or wear off, in seconds: a day.
pub const MAX_EVERY: u32 = 86_400;

/// The longest a status bar label may be.
pub const MAX_SHORT: usize = 12;

/// What builds a resource up (`builds = [{ on = "kill", amount = 5 }]`): something its holder
/// did or suffered, said where it happens (`World::gain_resource`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Build {
    /// The holder killed a creature (credited the kill: a wound it dealt bleeding it to death
    /// counts).
    Kill,
    /// A blow of the holder's hit (any attack: melee, a shot, a throw, a bash, a kick).
    Hit,
    /// A blow of the holder's was a critical.
    Critical,
    /// The holder was hit by a blow.
    Struck,
    /// A sneak attack of the holder's hit: a blow at a creature unaware of it (spec §8 "Class
    /// abilities": the thief's sneak attack).
    SneakAttack,
}

impl Build {
    /// Every event, in the order the wiki lists them.
    pub const ALL: [Build; 5] = [
        Build::Kill,
        Build::Hit,
        Build::Critical,
        Build::Struck,
        Build::SneakAttack,
    ];

    /// Its id in data: `sneak_attack`.
    pub const fn id(self) -> &'static str {
        match self {
            Build::Kill => "kill",
            Build::Hit => "hit",
            Build::Critical => "critical",
            Build::Struck => "struck",
            Build::SneakAttack => "sneak_attack",
        }
    }

    /// In words, for the wiki: "a kill".
    pub const fn words(self) -> &'static str {
        match self {
            Build::Kill => "a kill",
            Build::Hit => "a hit",
            Build::Critical => "a critical",
            Build::Struck => "being hit",
            Build::SneakAttack => "a sneak attack that hits",
        }
    }
}

/// How a resource changes by itself as time passes: one point every `every` seconds of game
/// time at the normal pace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverTime {
    /// Toward its maximum (a ranger's focus).
    Regenerates { every: u32 },
    /// Toward 0 (a barbarian's rage).
    Decays { every: u32 },
}

impl OverTime {
    /// Seconds per point at the normal pace.
    pub const fn every(self) -> u32 {
        match self {
            OverTime::Regenerates { every } | OverTime::Decays { every } => every,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceFile {
    #[serde(default)]
    resource: Vec<ResourceSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceSpec {
    id: String,
    name: String,
    short: String,
    max: u16,
    start: Option<u16>,
    #[serde(default = "yes")]
    show_max: bool,
    regenerates: Option<u32>,
    decays: Option<u32>,
    #[serde(default = "yes")]
    resting: bool,
    #[serde(default)]
    ends_at_zero: bool,
    ended: Option<String>,
    #[serde(default)]
    builds: Vec<BuildSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildSpec {
    on: Build,
    amount: u16,
}

fn yes() -> bool {
    true
}

/// A resource, in file order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceDef {
    pub id: String,
    /// As the player reads it: "Rage" (the character screen).
    pub name: String,
    /// Its status bar label: "Rage" (`Rage 7`).
    pub short: String,
    /// The most it holds (talents may raise it later: `World::resource_max`).
    pub max: u16,
    /// What it starts at when granted.
    pub start: u16,
    /// Whether the status bar shows the maximum too (`Rage 7/20`) or the value alone (`Rage 7`).
    pub show_max: bool,
    /// How it changes by itself as time passes, if it does.
    pub over_time: Option<OverTime>,
    /// Whether it changes over time while the player rests.
    pub resting: bool,
    /// Whether it ends at zero: the holder loses it (and its bar) until it is granted again.
    pub ends_at_zero: bool,
    /// What the player is told when it ends ("Your rage subsides."), if anything.
    pub ended: Option<String>,
    /// What builds it up, and by how much, in file order (one entry per event).
    pub builds: Vec<(Build, u16)>,
}

impl ResourceDef {
    /// How much event `on` builds it up (0: it does not).
    pub fn built_by(&self, on: Build) -> u16 {
        (self.builds.iter())
            .find(|(b, _)| *b == on)
            .map_or(0, |(_, n)| *n)
    }
}

/// Parses `data/resources.toml`: resources with unique ids, a maximum of 1 to
/// [`MAX_RESOURCE`], a start no higher than it (at least 1 for one that ends at zero), at most
/// one way of changing over time, and each building event once.
pub(super) fn parse(file: &str, text: &str) -> Result<Vec<ResourceDef>, ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: ResourceFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    let mut list: Vec<ResourceDef> = Vec::new();
    for spec in parsed.resource {
        let id = spec.id.clone();
        let err = |m: &str| err(format!("resource `{id}`: {m}"));
        if id.trim().is_empty() {
            return Err(err("`id` must not be empty"));
        }
        if list.iter().any(|r| r.id == id) {
            return Err(err("duplicate id"));
        }
        if spec.name.trim().is_empty() || spec.short.trim().is_empty() {
            return Err(err("`name` and `short` must not be empty"));
        }
        if spec.short.chars().count() > MAX_SHORT {
            return Err(err(&format!("`short` is at most {MAX_SHORT} characters")));
        }
        if !(1..=MAX_RESOURCE).contains(&spec.max) {
            return Err(err(&format!("`max` must be 1 to {MAX_RESOURCE}")));
        }
        let start = spec.start.unwrap_or(spec.max);
        if start > spec.max {
            return Err(err("`start` must not be above `max`"));
        }
        if spec.ends_at_zero && start == 0 {
            return Err(err("one that `ends_at_zero` must `start` above 0"));
        }
        let over_time = match (spec.regenerates, spec.decays) {
            (Some(_), Some(_)) => {
                return Err(err("it either `regenerates` or `decays`, not both"));
            }
            (Some(every), None) => Some(OverTime::Regenerates { every }),
            (None, Some(every)) => Some(OverTime::Decays { every }),
            (None, None) => None,
        };
        if over_time.is_some_and(|o| !(1..=MAX_EVERY).contains(&o.every())) {
            return Err(err(&format!(
                "`regenerates` and `decays` are 1 to {MAX_EVERY} seconds a point"
            )));
        }
        if spec.ended.is_some() && !spec.ends_at_zero {
            return Err(err("`ended` is said when it ends: it needs `ends_at_zero`"));
        }
        if spec.ended.as_ref().is_some_and(|t| t.trim().is_empty()) {
            return Err(err("`ended` must not be empty"));
        }
        let mut builds: Vec<(Build, u16)> = Vec::new();
        for b in &spec.builds {
            if builds.iter().any(|(on, _)| *on == b.on) {
                return Err(err(&format!("builds on `{}` twice", b.on.id())));
            }
            if !(1..=MAX_RESOURCE).contains(&b.amount) {
                return Err(err(&format!("a build's `amount` is 1 to {MAX_RESOURCE}")));
            }
            builds.push((b.on, b.amount));
        }
        list.push(ResourceDef {
            id: spec.id,
            name: spec.name,
            short: spec.short,
            max: spec.max,
            start,
            show_max: spec.show_max,
            over_time,
            resting: spec.resting,
            ends_at_zero: spec.ends_at_zero,
            ended: spec.ended,
            builds,
        });
    }
    if list.len() > usize::from(u16::MAX) {
        return Err(err("too many resources".into()));
    }
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAGE: &str = "[[resource]]\nid = \"rage\"\nname = \"Rage\"\nshort = \"Rage\"\n\
                        max = 20\nstart = 12\nshow_max = false\ndecays = 6\nends_at_zero = true\n\
                        ended = \"Your rage subsides.\"\nbuilds = [{ on = \"kill\", amount = 5 }]\n";

    #[test]
    fn resources_parse_in_order_with_their_rules() {
        let text = format!(
            "{RAGE}[[resource]]\nid = \"focus\"\nname = \"Focus\"\nshort = \"Focus\"\n\
             max = 10\nregenerates = 6\nresting = false\n"
        );
        let list = parse("r.toml", &text).expect("parses");
        assert_eq!(list.len(), 2);
        let rage = &list[0];
        assert_eq!((rage.max, rage.start, rage.show_max), (20, 12, false));
        assert_eq!(rage.over_time, Some(OverTime::Decays { every: 6 }));
        assert!(rage.ends_at_zero && rage.resting);
        assert_eq!(rage.built_by(Build::Kill), 5);
        assert_eq!(rage.built_by(Build::Hit), 0);
        let focus = &list[1];
        assert_eq!(focus.start, 10, "it starts full without a `start`");
        assert!(focus.show_max && !focus.resting && !focus.ends_at_zero);
        assert_eq!(focus.over_time, Some(OverTime::Regenerates { every: 6 }));
    }

    #[test]
    fn bad_resource_files_are_refused() {
        let head = "[[resource]]\nid = \"x\"\nname = \"X\"\nshort = \"X\"\n";
        let bad = |rest: &str| {
            let text = format!("{head}{rest}");
            parse("r.toml", &text).expect_err(rest).to_string()
        };
        assert!(bad("max = 0").contains("`max`"));
        assert!(bad("max = 1001").contains("`max`"));
        assert!(bad("max = 5\nstart = 6").contains("above `max`"));
        assert!(bad("max = 5\nstart = 0\nends_at_zero = true").contains("above 0"));
        assert!(bad("max = 5\nregenerates = 6\ndecays = 6").contains("not both"));
        assert!(bad("max = 5\ndecays = 0").contains("seconds a point"));
        assert!(bad("max = 5\nended = \"Gone.\"").contains("needs `ends_at_zero`"));
        let twice =
            "max = 5\nbuilds = [{ on = \"hit\", amount = 1 }, { on = \"hit\", amount = 2 }]";
        assert!(bad(twice).contains("twice"));
        assert!(bad("max = 5\nbuilds = [{ on = \"hit\", amount = 0 }]").contains("amount"));
        assert!(bad("max = 5\nbuilds = [{ on = \"sneeze\", amount = 1 }]").contains("sneeze"));
        let long = "[[resource]]\nid = \"x\"\nname = \"X\"\nshort = \"Thirteen long\"\nmax = 5\n";
        assert!(
            parse("r.toml", long)
                .expect_err("long")
                .to_string()
                .contains("`short`")
        );
        let dup = format!("{RAGE}{RAGE}");
        assert!(
            parse("r.toml", &dup)
                .expect_err("dup")
                .to_string()
                .contains("duplicate")
        );
    }
}

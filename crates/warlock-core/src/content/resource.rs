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

use super::{ContentError, StatusId};
use crate::stats::Stat;
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
    /// A sneak attack of the holder's killed (after its hit built `SneakAttack`).
    SneakKill,
}

impl Build {
    /// Every event, in the order the wiki lists them.
    pub const ALL: [Build; 6] = [
        Build::Kill,
        Build::Hit,
        Build::Critical,
        Build::Struck,
        Build::SneakAttack,
        Build::SneakKill,
    ];

    /// Its id in data: `sneak_attack`.
    pub const fn id(self) -> &'static str {
        match self {
            Build::Kill => "kill",
            Build::Hit => "hit",
            Build::Critical => "critical",
            Build::Struck => "struck",
            Build::SneakAttack => "sneak_attack",
            Build::SneakKill => "sneak_kill",
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
            Build::SneakKill => "a sneak attack that kills",
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
    status: Option<String>,
    #[serde(default)]
    builds: Vec<BuildSpec>,
    max_from: Option<MaxFromSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MaxFromSpec {
    stat: String,
    per_levels: u8,
    #[serde(default)]
    least: u16,
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

/// What else a resource's maximum grows with (`max_from = { stat = "toughness", per_levels =
/// 10, least = 5 }`): the holder's modifier in `stat` and a point per `per_levels` character
/// levels, added to its `max`, the sum at least `least` (before what talents add). Read in one
/// place, `Actor::resource_max`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaxFrom {
    pub stat: Stat,
    /// A point more per this many character levels (1 or more).
    pub per_levels: u8,
    /// The least the maximum may come to by the stat and the level.
    pub least: u16,
}

impl MaxFrom {
    /// The maximum `max` comes to for a holder with modifier `modifier` in the stat at character
    /// `level`: at least [`MaxFrom::least`], at most [`MAX_RESOURCE`].
    pub fn max(self, max: u16, modifier: i32, level: u8) -> u16 {
        let more = modifier + i32::from(level / self.per_levels);
        let sum = (i32::from(max) + more).max(i32::from(self.least));
        u16::try_from(sum.clamp(0, i32::from(MAX_RESOURCE))).unwrap_or(MAX_RESOURCE)
    }
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
    /// Whether it starts full when granted (no `start` in data): at its maximum by data, what
    /// `max_from` adds included (not its talents'; [`ResourceDef::start`] is then `max`).
    pub starts_full: bool,
    /// What else its maximum grows with, if anything (stamina: Toughness and level).
    pub max_from: Option<MaxFrom>,
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
    /// The status its holder suffers while they hold it (Raging, for rage): it begins as the
    /// resource is granted and ends with it, never easing by itself.
    pub status: Option<StatusId>,
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
/// one way of changing over time, each building event once, and a `status` that `status` knows
/// (`(id, whether it eases by itself)`) and that does not ease by itself.
pub(super) fn parse(
    file: &str,
    text: &str,
    status: impl Fn(&str) -> Option<(StatusId, bool)>,
) -> Result<Vec<ResourceDef>, ContentError> {
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
        let tied = match &spec.status {
            None => None,
            Some(name) => match status(name) {
                None => return Err(err(&format!("names an unknown status `{name}`"))),
                Some((_, true)) => {
                    return Err(err(&format!(
                        "its status `{name}` lasts while it is held: it must not `eases`"
                    )));
                }
                Some((id, false)) => Some(id),
            },
        };
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
        let max_from = match spec.max_from {
            None => None,
            Some(m) => {
                let Some(stat) = Stat::from_id(&m.stat) else {
                    return Err(err(&format!(
                        "`max_from` names an unknown stat `{}`",
                        m.stat
                    )));
                };
                if m.per_levels == 0 {
                    return Err(err("`max_from`'s `per_levels` must be 1 or more"));
                }
                if m.least > spec.max {
                    return Err(err("`max_from`'s `least` must not be above `max`"));
                }
                Some(MaxFrom {
                    stat,
                    per_levels: m.per_levels,
                    least: m.least,
                })
            }
        };
        list.push(ResourceDef {
            id: spec.id,
            name: spec.name,
            short: spec.short,
            max: spec.max,
            start,
            starts_full: spec.start.is_none(),
            max_from,
            show_max: spec.show_max,
            over_time,
            resting: spec.resting,
            ends_at_zero: spec.ends_at_zero,
            ended: spec.ended,
            status: tied,
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

    fn none(_: &str) -> Option<(StatusId, bool)> {
        None
    }

    #[test]
    fn a_resource_may_bring_a_status_that_lasts_while_it_is_held() {
        let statuses = |name: &str| match name {
            "raging" => Some((StatusId(3), false)),
            "winded" => Some((StatusId(4), true)),
            _ => None,
        };
        let with = |status: &str| format!("{RAGE}status = \"{status}\"\n");
        let list = parse("r.toml", &with("raging"), statuses).expect("parses");
        assert_eq!(list[0].status, Some(StatusId(3)));
        let bad = |status: &str| {
            parse("r.toml", &with(status), statuses)
                .expect_err(status)
                .to_string()
        };
        assert!(bad("winded").contains("must not `eases`"));
        assert!(bad("sulking").contains("unknown status `sulking`"));
        assert_eq!(parse("r.toml", RAGE, none).expect("parses")[0].status, None);
    }

    #[test]
    fn a_status_that_is_followed_must_end_by_time_a_cure_or_its_resource() {
        use crate::content::{Content, ContentFiles};
        let mut files = ContentFiles::builtin();
        let (_, text) = files
            .table("resources")
            .next()
            .expect("data/resources.toml");
        let text = text.replace("status = \"raging\"\n", "");
        files.insert("data/resources.toml", &text);
        let e = Content::load(&files).expect_err("Raging would never end");
        assert!(
            e.to_string()
                .contains("status `raging`: follows: it never ends"),
            "{e}"
        );
    }

    #[test]
    fn resources_parse_in_order_with_their_rules() {
        let text = format!(
            "{RAGE}[[resource]]\nid = \"focus\"\nname = \"Focus\"\nshort = \"Focus\"\n\
             max = 10\nregenerates = 6\nresting = false\n"
        );
        let list = parse("r.toml", &text, none).expect("parses");
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
    fn a_maximum_may_grow_with_a_stat_and_the_level() {
        let text = "[[resource]]\nid = \"stamina\"\nname = \"Stamina\"\nshort = \"Sta\"\n\
                    max = 10\nmax_from = { stat = \"toughness\", per_levels = 10, least = 5 }\n";
        let list = parse("r.toml", text, none).expect("parses");
        let from = list[0].max_from.expect("max_from");
        assert_eq!(
            (from.stat, from.per_levels, from.least),
            (Stat::Toughness, 10, 5)
        );
        assert!(list[0].starts_full);
        assert_eq!(from.max(10, 2, 1), 12, "Toughness 14 at level 1");
        assert_eq!(from.max(10, 4, 60), 20, "Toughness 18 at level 60");
        assert_eq!(from.max(10, -5, 9), 5, "never below `least`");
        let head = "[[resource]]\nid = \"x\"\nname = \"X\"\nshort = \"X\"\nmax = 10\n";
        let bad = |rest: &str| {
            let text = format!("{head}{rest}");
            parse("r.toml", &text, none).expect_err(rest).to_string()
        };
        assert!(bad("max_from = { stat = \"luck\", per_levels = 10 }").contains("unknown stat"));
        assert!(bad("max_from = { stat = \"toughness\", per_levels = 0 }").contains("per_levels"));
        let least = "max_from = { stat = \"toughness\", per_levels = 10, least = 11 }";
        assert!(bad(least).contains("least"));
        assert!(bad("max_from = { stat = \"toughness\" }").contains("per_levels"));
    }

    #[test]
    fn bad_resource_files_are_refused() {
        let head = "[[resource]]\nid = \"x\"\nname = \"X\"\nshort = \"X\"\n";
        let bad = |rest: &str| {
            let text = format!("{head}{rest}");
            parse("r.toml", &text, none).expect_err(rest).to_string()
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
            parse("r.toml", long, none)
                .expect_err("long")
                .to_string()
                .contains("`short`")
        );
        let dup = format!("{RAGE}{RAGE}");
        assert!(
            parse("r.toml", &dup, none)
                .expect_err("dup")
                .to_string()
                .contains("duplicate")
        );
    }
}

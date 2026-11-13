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
    /// The holder was hit by a blow: any that hit, even one their armour soaked or they
    /// resisted to nothing (`Strike::is_hit`).
    Struck,
    /// A sneak attack of the holder's hit: a blow at a creature unaware of it (spec §8 "Class
    /// abilities": the thief's sneak attack).
    SneakAttack,
    /// A feint of the holder's fooled its foe (`world/feint.rs`: it falls for it). A chosen
    /// attack under the mark rule (`world/mark.rs`), made before the check.
    Feint,
    /// The holder killed its mark, the creature its openings are in (`world/mark.rs`; a wound
    /// it dealt bleeding the mark to death counts). Raised beside `Kill`, before the mark is
    /// cleared.
    MarkKill,
}

impl Build {
    /// Every event, in the order the wiki lists them.
    pub const ALL: [Build; 7] = [
        Build::Kill,
        Build::Hit,
        Build::Critical,
        Build::Struck,
        Build::SneakAttack,
        Build::Feint,
        Build::MarkKill,
    ];

    /// Its id in data: `sneak_attack`.
    pub const fn id(self) -> &'static str {
        match self {
            Build::Kill => "kill",
            Build::Hit => "hit",
            Build::Critical => "critical",
            Build::Struck => "struck",
            Build::SneakAttack => "sneak_attack",
            Build::Feint => "feint",
            Build::MarkKill => "mark_kill",
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
            Build::Feint => "a feint the foe falls for",
            Build::MarkKill => "a kill of the creature the openings are in",
        }
    }
}

/// How a resource changes by itself as time passes: one point every `every` seconds of game
/// time at the normal pace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverTime {
    /// Toward its maximum (a ranger's focus).
    Regenerates { every: u32 },
    /// Toward 0 (a berserker's rage).
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
    #[serde(default)]
    marked: bool,
    ended: Option<String>,
    lacking: Option<String>,
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
    /// Whether its points are in one creature's guard (Openings): its holder's mark
    /// (`Actor::mark`, `world/mark.rs`) is kept while it holds it, a chosen attack at another
    /// living creature empties it, and it builds on nobody but the mark.
    pub marked: bool,
    /// What the player is told when it ends ("Your rage subsides."), if anything.
    pub ended: Option<String>,
    /// How the abilities menu says an ability it pays for is out of reach while too little is
    /// held ("you are short of breath"); none: "not enough stamina".
    pub lacking: Option<String>,
    /// The status its holder suffers while they hold it (Raging, for rage): it begins as the
    /// resource is granted and ends with it, never easing by itself.
    pub status: Option<StatusId>,
    /// What builds it up, and by how much, in file order (one entry per event).
    pub builds: Vec<(Build, u16)>,
}

impl ResourceDef {
    /// The most it may hold by data at the highest level for a holder of average stats, before
    /// talents and affixes: `max`, with what `max_from` adds at [`super::MAX_LEVEL`] (ki: 4 and
    /// a point per 6 levels). What a talent's cost is checked against at load.
    pub fn most(&self) -> u16 {
        (self.max_from).map_or(self.max, |from| from.max(self.max, 0, super::MAX_LEVEL))
    }

    /// How much event `on` builds it up (0: it does not).
    pub fn built_by(&self, on: Build) -> u16 {
        (self.builds.iter())
            .find(|(b, _)| *b == on)
            .map_or(0, |(_, n)| *n)
    }
}

/// Parses `data/resources.toml`: resources with unique ids, at most one way of changing over
/// time (a point at least a second), `ended` only on one that ends at zero, and a `status` that
/// `status` knows (`(id, whether it eases by itself)`).
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
        let start = spec.start.unwrap_or(spec.max);
        let over_time = match (spec.regenerates, spec.decays) {
            (Some(_), Some(_)) => {
                return Err(err("it either `regenerates` or `decays`, not both"));
            }
            (Some(every), None) => Some(OverTime::Regenerates { every }),
            (None, Some(every)) => Some(OverTime::Decays { every }),
            (None, None) => None,
        };
        if over_time.is_some_and(|o| o.every() == 0) {
            return Err(err(
                "`regenerates` and `decays` are 1 or more seconds a point",
            ));
        }
        if spec.ended.is_some() && !spec.ends_at_zero {
            return Err(err("`ended` is said when it ends: it needs `ends_at_zero`"));
        }
        let tied = match &spec.status {
            None => None,
            Some(name) => match status(name) {
                None => return Err(err(&format!("names an unknown status `{name}`"))),
                Some((id, _)) => Some(id),
            },
        };
        let builds: Vec<(Build, u16)> = spec.builds.iter().map(|b| (b.on, b.amount)).collect();
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
            marked: spec.marked,
            ended: spec.ended,
            lacking: spec.lacking,
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
        assert!(rage.ends_at_zero && rage.resting && !rage.marked);
        assert_eq!(rage.built_by(Build::Kill), 5);
        assert_eq!(rage.built_by(Build::Hit), 0);
        let focus = &list[1];
        assert_eq!(focus.start, 10, "it starts full without a `start`");
        assert!(focus.show_max && !focus.resting && !focus.ends_at_zero);
        assert_eq!(focus.over_time, Some(OverTime::Regenerates { every: 6 }));
    }

    #[test]
    fn a_pool_may_be_marked_and_built_by_the_marks_kill() {
        let text = "[[resource]]\nid = \"openings\"\nname = \"Openings\"\nshort = \"Openings\"\n\
                    max = 5\nstart = 0\nmarked = true\n\
                    builds = [{ on = \"mark_kill\", amount = 1 }]\n";
        let list = parse("r.toml", text, none).expect("parses");
        assert!(list[0].marked);
        assert_eq!(list[0].built_by(Build::MarkKill), 1);
        assert_eq!(Build::MarkKill.id(), "mark_kill");
        let old = text.replace("mark_kill", "sneak_kill");
        assert!(parse("r.toml", &old, none).is_err(), "sneak_kill is gone");
    }

    #[test]
    fn a_pool_may_be_built_by_a_feint() {
        let text = "[[resource]]\nid = \"openings\"\nname = \"Openings\"\nshort = \"Openings\"\n\
                    max = 5\nstart = 0\nmarked = true\n\
                    builds = [{ on = \"feint\", amount = 1 }]\n";
        let list = parse("r.toml", text, none).expect("parses");
        assert_eq!(list[0].built_by(Build::Feint), 1);
        assert_eq!(list[0].built_by(Build::SneakAttack), 0);
        assert_eq!(Build::Feint.id(), "feint");
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
        assert!(bad("max_from = { stat = \"toughness\" }").contains("per_levels"));
    }

    #[test]
    fn bad_resource_files_are_refused() {
        let head = "[[resource]]\nid = \"x\"\nname = \"X\"\nshort = \"X\"\n";
        let bad = |rest: &str| {
            let text = format!("{head}{rest}");
            parse("r.toml", &text, none).expect_err(rest).to_string()
        };
        assert!(bad("max = 5\nregenerates = 6\ndecays = 6").contains("not both"));
        assert!(bad("max = 5\ndecays = 0").contains("seconds a point"));
        assert!(bad("max = 5\nended = \"Gone.\"").contains("needs `ends_at_zero`"));
        assert!(bad("max = 5\nbuilds = [{ on = \"sneeze\", amount = 1 }]").contains("sneeze"));
        let dup = format!("{RAGE}{RAGE}");
        assert!(
            parse("r.toml", &dup, none)
                .expect_err("dup")
                .to_string()
                .contains("duplicate")
        );
    }
}

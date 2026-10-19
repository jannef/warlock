//! Status effects (`assets/data/status.toml`, spec §8 "Status effects"): what can ail an actor,
//! such as bleeding or poison, and the passive healing rules (`[healing]`).
//!
//! A status is data: its name, how strong it can get (`max` degrees of severity), how a new dose
//! combines with one the actor already has ([`Stacking`]), its timers in game time (a hit point
//! lost every `tick` seconds at one degree, `n` times as often at `n` degrees; a degree eased
//! every `eases` seconds), how fast hit points come back while it lasts (`regen`), what it does to
//! stats from some severity on ([`Penalty`]), what causes it (big hits and criticals,
//! [`Wound`]; a creature's attack, [`OnHit`]), what cures it (`cured_by`), and its messages.
//! The rules that apply them live in `crate::status` and `world/status.rs`.
//!
//! Without the file there are no status effects, and resting heals at the default rate
//! ([`Healing::default`]).

use super::ContentError;
use super::disease::DiseaseId;
use crate::stats::{ADJUSTMENTS, PerStat, Stat};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The most degrees of severity a status may have.
pub const MAX_SEVERITY: u16 = 100;
/// The fastest and slowest healing a status (or resting) may give, in percent of normal.
pub const MAX_REGEN: u16 = 1000;
/// The most hit points one tick may cost.
pub const MAX_TICK_DAMAGE: u16 = 1000;

/// Index of a status definition in [`super::Content::status_defs`], in file order (which is
/// also how important it is: the status bar shows the first first).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StatusId(pub u16);

/// How a new dose of a status combines with the one an actor already has.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stacking {
    /// The degrees add up, to the status's `max`; its timers run on (bleeding wounds, poison).
    #[default]
    Add,
    /// The stronger of the two, and its easing starts over.
    Refresh,
    /// The stronger of the two; its timers run on.
    Highest,
}

impl Stacking {
    /// Its id in data: `add`.
    pub const fn id(self) -> &'static str {
        match self {
            Stacking::Add => "add",
            Stacking::Refresh => "refresh",
            Stacking::Highest => "highest",
        }
    }
}

/// What a status does to stats from a severity on: `penalty = { from = 4, strength = -1 }`.
/// Every rule that reads a stat reads it lowered (raised) by this ([`crate::world::Actor::
/// effective_stats`]); penalties of several statuses add up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Penalty {
    /// The lowest severity it applies at.
    pub from: u16,
    /// What it adds to each score (negative: a penalty).
    pub by: PerStat<i8>,
}

/// Which blows cause a status, and how many degrees of it (spec §8 "Status effects"): a big hit
/// (damage of at least a third of the target's most hit points, [`crate::combat::big_hit`]) or
/// a confirmed critical. A blow that is both causes the more of the two.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wound {
    #[serde(default)]
    pub big_hit: u16,
    #[serde(default)]
    pub critical: u16,
}

impl Wound {
    /// The degrees a hit causes: `critical` for a confirmed critical, `big_hit` for a big hit,
    /// the more of the two for both.
    pub fn degrees(&self, critical: bool, big: bool) -> u16 {
        let crit = if critical { self.critical } else { 0 };
        let big = if big { self.big_hit } else { 0 };
        crit.max(big)
    }
}

/// What an attack may cause when it hits and the target lives (`on_hit` on a creature's natural
/// attack): a status, `{ status = "poisoned", chance = 10, severity = 2 }`, or exposure to a
/// disease, `{ disease = "filth_fever", chance = 10 }` (spec §8 "Diseases").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OnHit {
    pub what: Affliction,
    /// The chance in percent (1 to 100) that a hit causes it: one draw of the gameplay stream,
    /// made only when it is below 100.
    pub chance: u8,
}

/// What an `on_hit` entry causes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Affliction {
    /// So many degrees of a status.
    Status { status: StatusId, severity: u16 },
    /// Exposure to a disease: the target resists it with a check, or catches it.
    Disease(DiseaseId),
}

/// An `on_hit` entry as written in content files, before its status or disease is resolved.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OnHitSpec {
    pub status: Option<String>,
    pub disease: Option<String>,
    pub chance: u8,
    pub severity: Option<u16>,
}

impl OnHitSpec {
    /// Resolves its status by id (to the status and its `max`; `None`: unknown, or one only a
    /// disease causes) or its disease, and checks the chance and the severity.
    pub(super) fn resolve(
        &self,
        status: impl Fn(&str) -> Option<(StatusId, u16)>,
        disease: impl Fn(&str) -> Option<DiseaseId>,
    ) -> Result<OnHit, String> {
        let name = self.status.as_ref().or(self.disease.as_ref());
        let name = name.map_or("", String::as_str);
        if !(1..=100).contains(&self.chance) {
            return Err(format!(
                "on_hit `{name}`: chance must be 1 to 100 (percent)"
            ));
        }
        let what = match (&self.status, &self.disease, self.severity) {
            (Some(id), None, Some(severity)) => {
                let (found, max) =
                    status(id).ok_or_else(|| format!("on_hit: unknown status `{id}`"))?;
                if !(1..=max).contains(&severity) {
                    return Err(format!(
                        "on_hit `{id}`: severity must be 1 to {max} (the status's max)"
                    ));
                }
                Affliction::Status {
                    status: found,
                    severity,
                }
            }
            (Some(id), None, None) => {
                return Err(format!("on_hit `{id}`: a status needs a `severity`"));
            }
            (None, Some(id), None) => Affliction::Disease(
                disease(id).ok_or_else(|| format!("on_hit: unknown disease `{id}`"))?,
            ),
            (None, Some(id), Some(_)) => {
                return Err(format!(
                    "on_hit `{id}`: a disease has no `severity` (it is caught or resisted)"
                ));
            }
            _ => {
                return Err(
                    "on_hit: name either a `status` (with a `severity`) or a `disease`".into(),
                );
            }
        };
        Ok(OnHit {
            what,
            chance: self.chance,
        })
    }
}

/// The passive healing rules (spec §8 "Healing"): `[healing]` in the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Healing {
    /// How fast hit points come back while the player rests, in percent of normal.
    pub resting: u16,
}

impl Default for Healing {
    /// Resting heals twice as fast.
    fn default() -> Self {
        Self { resting: 200 }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusFile {
    #[serde(default)]
    healing: Healing,
    #[serde(default)]
    status: Vec<StatusSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusSpec {
    id: String,
    name: String,
    seen: Option<String>,
    #[serde(default)]
    stacking: Stacking,
    max: u16,
    tick: Option<u32>,
    #[serde(default)]
    damage: u16,
    eases: Option<u32>,
    #[serde(default = "normal_regen")]
    regen: u16,
    penalty: Option<PenaltySpec>,
    wound: Option<Wound>,
    #[serde(default)]
    cured_by: Vec<String>,
    start: Option<String>,
    end: Option<String>,
    dies: Option<String>,
    death: Option<String>,
}

fn normal_regen() -> u16 {
    100
}

/// `{ from = 4, strength = -1 }`: every key but `from` is a stat.
#[derive(Deserialize)]
struct PenaltySpec {
    from: u16,
    #[serde(flatten)]
    stats: BTreeMap<Stat, i8>,
}

/// A status effect, in file order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusDef {
    pub id: String,
    /// As the status bar and the character screen show it: `Bleeding`.
    pub name: String,
    /// How look mode names it on a creature the player sees (`bleeding`); `None`: it does not
    /// show on others.
    pub seen: Option<String>,
    pub stacking: Stacking,
    /// The most degrees of severity it reaches.
    pub max: u16,
    /// Seconds of game time per tick at one degree of severity; at `n` degrees it ticks `n`
    /// times as often. `None`: it does not tick.
    pub tick: Option<u32>,
    /// Hit points each tick costs.
    pub damage: u16,
    /// Seconds of game time for it to ease by one degree; at none it is gone. `None`: it lasts
    /// until cured.
    pub eases: Option<u32>,
    /// How fast hit points come back while it lasts, in percent of normal: 0 not at all, 100 as
    /// usual. With several statuses the slowest counts.
    pub regen: u16,
    pub penalty: Option<Penalty>,
    /// The blows that cause it, if any.
    pub wound: Option<Wound>,
    /// The means that cure it (`bandage`): [`crate::World`]'s cure takes one of these.
    pub cured_by: Vec<String>,
    /// Said when the player comes to suffer it: "You are bleeding."
    pub start: Option<String>,
    /// Said when it leaves the player: "The bleeding stops."
    pub end: Option<String>,
    /// What a creature the player sees does when it kills it: "bleeds to death" ("The giant rat
    /// bleeds to death."); `None`: it dies.
    pub dies: Option<String>,
    /// The death screen's line when it kills the player: "Bled to death".
    pub death: Option<String>,
}

impl StatusDef {
    /// What it does to each score at `severity` (nothing below its penalty's `from`).
    pub fn penalty_at(&self, severity: u16) -> Option<&PerStat<i8>> {
        self.penalty
            .as_ref()
            .filter(|p| severity >= p.from)
            .map(|p| &p.by)
    }
}

/// Parses `data/status.toml`: the healing rules and the statuses in file order.
pub(super) fn parse(file: &str, text: &str) -> Result<(Healing, Vec<StatusDef>), ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: StatusFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    if parsed.healing.resting > MAX_REGEN {
        return Err(err(format!(
            "healing: resting must be 0 to {MAX_REGEN} (percent)"
        )));
    }
    let mut list: Vec<StatusDef> = Vec::new();
    for spec in parsed.status {
        let id = spec.id.clone();
        if list.iter().any(|s| s.id == id) {
            return Err(err(format!("duplicate status id `{id}`")));
        }
        let def = resolve(spec).map_err(|m| err(format!("status `{id}`: {m}")))?;
        list.push(def);
    }
    Ok((parsed.healing, list))
}

fn resolve(spec: StatusSpec) -> Result<StatusDef, String> {
    if spec.id.trim().is_empty() || spec.name.trim().is_empty() {
        return Err("`id` and `name` must not be empty".into());
    }
    if spec.seen.as_ref().is_some_and(|s| s.trim().is_empty()) {
        return Err("`seen` must not be empty".into());
    }
    if !(1..=MAX_SEVERITY).contains(&spec.max) {
        return Err(format!("max must be 1 to {MAX_SEVERITY}"));
    }
    if spec.tick == Some(0) || spec.eases == Some(0) {
        return Err("`tick` and `eases` are seconds and must be at least 1".into());
    }
    if spec.damage > MAX_TICK_DAMAGE {
        return Err(format!("damage must be at most {MAX_TICK_DAMAGE}"));
    }
    if spec.damage > 0 && spec.tick.is_none() {
        return Err("`damage` needs a `tick`".into());
    }
    if spec.regen > MAX_REGEN {
        return Err(format!("regen must be 0 to {MAX_REGEN} (percent)"));
    }
    let penalty = spec.penalty.map(|p| penalty(p, spec.max)).transpose()?;
    if let Some(w) = spec.wound
        && (w.big_hit > spec.max || w.critical > spec.max || w.big_hit + w.critical == 0)
    {
        return Err(format!(
            "wound: big_hit and critical must be 0 to {} (max), not both 0",
            spec.max
        ));
    }
    if spec.cured_by.iter().any(|c| c.trim().is_empty()) {
        return Err("cured_by: a cure must have a name".into());
    }
    Ok(StatusDef {
        id: spec.id,
        name: spec.name,
        seen: spec.seen,
        stacking: spec.stacking,
        max: spec.max,
        tick: spec.tick,
        damage: spec.damage,
        eases: spec.eases,
        regen: spec.regen,
        penalty,
        wound: spec.wound,
        cured_by: spec.cured_by,
        start: spec.start,
        end: spec.end,
        dies: spec.dies,
        death: spec.death,
    })
}

fn penalty(spec: PenaltySpec, max: u16) -> Result<Penalty, String> {
    if !(1..=max).contains(&spec.from) {
        return Err(format!("penalty: from must be 1 to {max} (max)"));
    }
    let mut by = PerStat::splat(0i8);
    for (stat, value) in spec.stats {
        if !ADJUSTMENTS.contains(&value) {
            return Err(format!(
                "penalty: {} {value} is out of range ({} to {})",
                stat.id(),
                ADJUSTMENTS.start(),
                ADJUSTMENTS.end()
            ));
        }
        by[stat] = value;
    }
    if by.iter().all(|(_, v)| v == 0) {
        return Err("penalty: it changes no stat".into());
    }
    Ok(Penalty {
        from: spec.from,
        by,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLEEDING: &str = "[[status]]\nid = \"bleeding\"\nname = \"Bleeding\"\nmax = 5\n\
                            tick = 40\ndamage = 1\neases = 60\nregen = 0\n";

    #[test]
    fn statuses_parse_with_defaults() {
        let text = format!(
            "[healing]\nresting = 300\n{BLEEDING}wound = {{ big_hit = 1, critical = 2 }}\n\
             cured_by = [\"bandage\"]\n\
             [[status]]\nid = \"poisoned\"\nname = \"Poisoned\"\nmax = 10\nstacking = \"refresh\"\n\
             penalty = {{ from = 4, strength = -1, dexterity = -1 }}\n"
        );
        let (healing, list) = parse("status.toml", &text).expect("valid");
        assert_eq!(healing.resting, 300);
        assert_eq!(list.len(), 2);
        let (bleeding, poison) = (&list[0], &list[1]);
        assert_eq!(bleeding.stacking, Stacking::Add, "the default");
        assert_eq!(
            (bleeding.tick, bleeding.eases, bleeding.regen),
            (Some(40), Some(60), 0)
        );
        assert_eq!(bleeding.wound.map(|w| w.degrees(true, true)), Some(2));
        assert_eq!(bleeding.wound.map(|w| w.degrees(false, true)), Some(1));
        assert_eq!(bleeding.wound.map(|w| w.degrees(false, false)), Some(0));
        assert_eq!(bleeding.cured_by, ["bandage"]);
        assert_eq!(poison.stacking, Stacking::Refresh);
        assert_eq!((poison.regen, poison.tick, poison.damage), (100, None, 0));
        assert_eq!(poison.penalty_at(3), None);
        let at_4 = poison.penalty_at(4).expect("from 4 on");
        assert_eq!(
            (
                at_4[Stat::Strength],
                at_4[Stat::Dexterity],
                at_4[Stat::Toughness]
            ),
            (-1, -1, 0)
        );
        // Without statuses: none, and resting heals twice as fast.
        let (healing, list) = parse("status.toml", "").expect("empty is valid");
        assert_eq!((healing, list.len()), (Healing::default(), 0));
        assert_eq!(Healing::default().resting, 200);
    }

    #[test]
    fn bad_statuses_are_refused() {
        for (text, why) in [
            (
                format!("{BLEEDING}{BLEEDING}"),
                "duplicate status id `bleeding`",
            ),
            (
                BLEEDING.replace("max = 5", "max = 0"),
                "max must be 1 to 100",
            ),
            (
                BLEEDING.replace("tick = 40", "tick = 0"),
                "must be at least 1",
            ),
            (
                BLEEDING.replace("tick = 40\n", ""),
                "`damage` needs a `tick`",
            ),
            (
                BLEEDING.replace("regen = 0", "regen = 1001"),
                "regen must be 0 to 1000",
            ),
            (
                format!("{BLEEDING}penalty = {{ from = 6, strength = -1 }}\n"),
                "from must be 1 to 5",
            ),
            (
                format!("{BLEEDING}penalty = {{ from = 1 }}\n"),
                "it changes no stat",
            ),
            (
                format!("{BLEEDING}penalty = {{ from = 1, luck = -1 }}\n"),
                "unknown stat `luck`",
            ),
            (
                format!("{BLEEDING}penalty = {{ from = 1, strength = -11 }}\n"),
                "out of range",
            ),
            (
                format!("{BLEEDING}wound = {{ big_hit = 6 }}\n"),
                "wound: big_hit and critical",
            ),
            (format!("{BLEEDING}wound = {{}}\n"), "not both 0"),
            (
                format!("{BLEEDING}cured_by = [\"\"]\n"),
                "a cure must have a name",
            ),
            (
                format!("{BLEEDING}seen = \" \"\n"),
                "`seen` must not be empty",
            ),
            (
                format!("{BLEEDING}stacking = \"stack\"\n"),
                "unknown variant `stack`",
            ),
            (format!("{BLEEDING}colour = 1\n"), "unknown field `colour`"),
            (
                "[healing]\nresting = 2000\n".into(),
                "resting must be 0 to 1000",
            ),
        ] {
            let e = parse("status.toml", &text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }

    #[test]
    fn on_hit_entries_name_a_status_within_its_max_or_a_disease() {
        let known = |id: &str| (id == "poisoned").then_some((StatusId(1), 10));
        let disease = |id: &str| (id == "fever").then_some(DiseaseId(0));
        let spec = |status: Option<&str>, disease: Option<&str>, chance, severity| OnHitSpec {
            status: status.map(String::from),
            disease: disease.map(String::from),
            chance,
            severity,
        };
        assert_eq!(
            spec(Some("poisoned"), None, 10, Some(2)).resolve(known, disease),
            Ok(OnHit {
                what: Affliction::Status {
                    status: StatusId(1),
                    severity: 2
                },
                chance: 10,
            })
        );
        assert_eq!(
            spec(None, Some("fever"), 25, None).resolve(known, disease),
            Ok(OnHit {
                what: Affliction::Disease(DiseaseId(0)),
                chance: 25,
            })
        );
        for (bad, why) in [
            (
                spec(Some("rotten"), None, 10, Some(2)),
                "unknown status `rotten`",
            ),
            (
                spec(Some("poisoned"), None, 0, Some(2)),
                "chance must be 1 to 100",
            ),
            (
                spec(Some("poisoned"), None, 101, Some(2)),
                "chance must be 1 to 100",
            ),
            (
                spec(Some("poisoned"), None, 10, Some(11)),
                "severity must be 1 to 10",
            ),
            (spec(Some("poisoned"), None, 10, None), "needs a `severity`"),
            (spec(None, Some("pox"), 10, None), "unknown disease `pox`"),
            (spec(None, Some("fever"), 10, Some(1)), "has no `severity`"),
            (spec(None, None, 10, None), "name either"),
            (
                spec(Some("poisoned"), Some("fever"), 10, Some(1)),
                "name either",
            ),
        ] {
            let e = bad.resolve(known, disease).unwrap_err();
            assert!(e.contains(why), "{why}: {e}");
        }
    }
}

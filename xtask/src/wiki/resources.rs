//! Alternative resources (spec §8 "Resources"): each resource's maximum (with what a stat and
//! the level add), its start, how it changes over time, what builds it, and whether it ends at
//! zero.

use super::text::{Term, anchored, link, signed};
use std::fmt::Write as _;
use warlock_core::content::{Build, Content, OverTime, ResourceDef, TalentEffect};
use warlock_core::dice::Dice;

/// Every resource, by name.
pub(super) fn resources(content: &Content) -> String {
    let mut list: Vec<&ResourceDef> = content.resources().iter().collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::from(
        "| Resource | Most | Starts at | Over time | Built by | At zero |\n\
         |---|:---:|:---:|---|---|---|\n",
    );
    for def in list {
        let resting = if def.resting {
            ""
        } else {
            ", not while resting"
        };
        let over = match def.over_time {
            Some(OverTime::Regenerates { every }) => {
                format!("a point back every {}{resting}", seconds(every))
            }
            Some(OverTime::Decays { every }) => {
                format!("a point lost every {}{resting}", seconds(every))
            }
            None => "-".to_string(),
        };
        let built: Vec<String> = (Build::ALL.iter())
            .filter(|&&on| def.built_by(on) > 0)
            .map(|&on| format!("+{} for {}", def.built_by(on), on.words()))
            .collect();
        let built = if built.is_empty() {
            "-".to_string()
        } else {
            built.join(", ")
        };
        let brings = def.status.map_or_else(String::new, |s| {
            format!(", and {} with it", content.status(s).name)
        });
        let zero = if def.ends_at_zero {
            format!("ends, and its bar goes{brings}")
        } else {
            "stays, empty".to_string()
        };
        let most = def.max_from.map_or_else(
            || def.max.to_string(),
            |from| {
                format!(
                    "{} + the {} modifier + 1 per {} levels (at least {})",
                    def.max,
                    from.stat.name(),
                    from.per_levels,
                    from.least
                )
            },
        );
        let start = if def.starts_full {
            "full".to_string()
        } else {
            def.start.to_string()
        };
        let _ = writeln!(
            out,
            "| {} | {most} | {start} | {over} | {built} | {zero} |",
            anchored(Term::Resource, &def.id, &def.name)
        );
    }
    out
}

/// Exploit's ladder (spec §8 "Class abilities"; the openings design §6): every class talent on a
/// class's level table that adds to an exploit, by level, what it adds per opening spent, and
/// the ladder from that level on.
pub(super) fn exploit_ladder(content: &Content) -> String {
    let book = content.talents();
    let mut out = String::from(
        "| Level | Talent | Adds per opening spent | An exploit from then on, per opening |\n\
         |:---:|---|---|---|\n",
    );
    for class in content.classes() {
        let mut rows: Vec<(u8, &str)> = (class.abilities.iter())
            .map(|(level, talent)| (*level, talent.as_str()))
            .collect();
        rows.sort_unstable();
        let mut ladder = Ladder::default();
        for (level, id) in rows {
            let Some(t) = book.id(id).map(|t| book.get(t)) else {
                continue;
            };
            let adds: Vec<String> = (t.effects.iter())
                .filter_map(|e| ladder.add(content, e))
                .collect();
            if adds.is_empty() {
                continue;
            }
            let _ = writeln!(
                out,
                "| {level} | {} | {} | {} |",
                link(Term::Talent, &t.id, &t.name),
                adds.join("; "),
                ladder.words()
            );
        }
    }
    out
}

/// Exploit's ladder summed so far, as [`exploit_ladder`] walks a class's table.
#[derive(Default)]
struct Ladder {
    dice: Option<Dice>,
    to_hit: i32,
    status: Option<String>,
    threat: u32,
    sure_at: Option<u16>,
}

impl Ladder {
    /// Adds `effect`, if it is one of the ladder's; what it adds in words.
    fn add(&mut self, content: &Content, effect: &TalentEffect) -> Option<String> {
        Some(match effect {
            TalentEffect::ExploitDice { dice } => {
                self.dice = Some(self.dice.map_or(*dice, |had| had.and(*dice)));
                format!("+{dice} damage")
            }
            TalentEffect::ExploitToHit { value } => {
                self.to_hit += value;
                format!("{} to hit", signed(*value))
            }
            TalentEffect::ExploitStatus { status } => {
                let name = (content.status_id(status))
                    .map_or(status.clone(), |s| content.status(s).name.to_lowercase());
                let named = link(Term::Status, status, &name);
                self.status = Some(named.clone());
                format!("a hit leaves the foe {named} (1 degree)")
            }
            TalentEffect::ExploitThreat { value, sure_at } => {
                self.threat += u32::from(*value);
                self.sure_at = Some(self.sure_at.map_or(*sure_at, |had| had.min(*sure_at)));
                format!(
                    "a [critical](combat.md#criticals) threat {value} lower (never below 2); with \
                     {sure_at} or more spent, a critical whenever it hits, with no confirmation \
                     roll"
                )
            }
            _ => return None,
        })
    }

    /// The ladder in words: "+2d6 damage, +1 to hit, stunned, threat 4 lower, sure critical
    /// from 5".
    fn words(&self) -> String {
        let mut parts = vec![
            self.dice
                .map_or_else(|| "-".into(), |d| format!("+{d} damage")),
        ];
        if self.to_hit != 0 {
            parts.push(format!("{} to hit", signed(self.to_hit)));
        }
        parts.extend(self.status.clone());
        if self.threat != 0 {
            parts.push(format!("threat {} lower", self.threat));
        }
        if let Some(at) = self.sure_at {
            parts.push(format!("a sure critical from {at} spent"));
        }
        parts.join(", ")
    }
}

/// `6 seconds` (a base turn), `2 minutes`, `1 minute and 30 seconds`.
fn seconds(every: u32) -> String {
    let (m, s) = (every / 60, every % 60);
    let unit = |n: u32, one: &str| match n {
        1 => format!("1 {one}"),
        n => format!("{n} {one}s"),
    };
    match (m, s) {
        (0, s) => unit(s, "second"),
        (m, 0) => unit(m, "minute"),
        (m, s) => format!("{} and {}", unit(m, "minute"), unit(s, "second")),
    }
}

//! Healing (spec §8 "Healing"), status effects (spec §8 "Status effects"), and diseases (spec §8
//! "Diseases").

use super::combat::percent;
use super::order;
use super::text::{Term, anchored, capital, signed, stat_link, yes};
use std::fmt::Write as _;
use warlock_core::check;
use warlock_core::content::{Affliction, Content, DiseaseDef, DiseaseId, StatusDef};
use warlock_core::dice::Dice;
use warlock_core::hunger::Hunger;
use warlock_core::stats;
use warlock_core::time::Duration;

/// A pace of healing in percent of normal, in words: `twice as fast`, `half as fast`, `not at
/// all`.
pub(super) fn pace(percent: u32) -> String {
    match percent {
        0 => "not at all".into(),
        100 => "as usual".into(),
        200 => "twice as fast".into(),
        300 => "three times as fast".into(),
        50 => "half as fast".into(),
        25 => "a quarter as fast".into(),
        p if p % 100 == 0 => format!("{} times as fast", p / 100),
        p => format!("at {p}% of the usual pace"),
    }
}

/// How fast hit points come back (spec §8 "Healing"): resting, each status that changes it,
/// and each disease that slows it more than Diseased does.
pub(super) fn healing(content: &Content) -> String {
    let mut out = String::from("| While | Hit points come back |\n|---|---|\n");
    let resting = u32::from(content.healing().resting);
    let _ = writeln!(out, "| Resting (`R`) | {} |", pace(resting));
    for def in content.status_defs().iter().filter(|s| s.regen != 100) {
        let _ = writeln!(out, "| {} | {} |", def.name, pace(u32::from(def.regen)));
    }
    for (_, def) in content.disease_ids() {
        if def.regen.is_some_and(|r| r < diseased_regen(content)) {
            let _ = writeln!(
                out,
                "| Diseased with {} | {} |",
                def.name,
                pace(u32::from(disease_regen(content, def)))
            );
        }
    }
    if content.hunger().is_some() {
        let _ = writeln!(out, "| {} | {} |", Hunger::Starving.name(), pace(0));
    }
    out
}

/// How fast hit points come back while Diseased (its status's `regen`), in percent.
fn diseased_regen(content: &Content) -> u16 {
    content
        .disease_status()
        .map_or(100, |s| content.status(s).regen)
}

/// How fast hit points come back while disease `def` shows: the slower of Diseased's pace and
/// its own.
fn disease_regen(content: &Content, def: &DiseaseDef) -> u16 {
    def.regen
        .map_or(diseased_regen(content), |r| r.min(diseased_regen(content)))
}

/// How often, in words: `a minute`, `every 40 seconds`, `every 2 minutes`.
fn every(seconds: u32) -> String {
    match seconds {
        60 => "a minute".into(),
        s => format!("every {}", Duration::seconds(u64::from(s)).spoken()),
    }
}

/// The creatures' natural attacks and throws a hit with which may cause `what` (an `on_hit`
/// entry it matches; `what` returns what to add after the chance), by creature name: "the giant
/// rat's bite (10%, 2 degrees)", "the drow crossbowman's poisoned bolt (20%, 1 degree)".
fn attacks_causing(content: &Content, what: impl Fn(&Affliction) -> Option<String>) -> Vec<String> {
    let mut found = Vec::new();
    for (_, c) in order::creatures(content) {
        let thrown = c.throw.as_ref().map(|t| &t.attack);
        for attack in c.attack.iter().chain(thrown) {
            for o in &attack.on_hit {
                if let Some(detail) = what(&o.what) {
                    found.push(format!(
                        "the {}'s {} ({}%{detail})",
                        c.name, attack.name, o.chance
                    ));
                }
            }
        }
    }
    found
}

/// What causes a status, in words: wounds, diseases, the creatures' attacks whose `on_hit` names
/// it (with the chance a hit has and the degrees it gives), and rotten corpses when it is the
/// sickness of food (spec §8 "Hunger").
fn causes(content: &Content, status: usize, def: &StatusDef) -> String {
    let attacks = attacks_causing(content, |what| match *what {
        Affliction::Status {
            status: s,
            severity,
        } if usize::from(s.0) == status => {
            let degrees = if severity == 1 { "degree" } else { "degrees" };
            Some(format!(", {severity} {degrees}"))
        }
        Affliction::Hold {
            status: s,
            difficulty,
        } if usize::from(s.0) == status => {
            let grip = difficulty.or(def.hold.map(|h| h.difficulty)).unwrap_or(0);
            Some(format!(", grip {grip}"))
        }
        _ => None,
    });
    let mut causes = Vec::new();
    if content
        .disease_status()
        .is_some_and(|s| usize::from(s.0) == status)
    {
        causes.push("any disease, once it shows");
    }
    if let Some(w) = &def.wound {
        match (w.big_hit > 0, w.critical > 0) {
            (true, true) => causes.push("big hits and confirmed criticals"),
            (true, false) => causes.push("big hits"),
            (false, _) => causes.push("confirmed criticals"),
        }
    }
    causes.extend(attacks.iter().map(String::as_str));
    let brought = brought_by(content, status).map(|r| format!("holding {r}"));
    let sickness = (content.hunger()).and_then(|h| content.status_id(h.sickness.as_deref()?));
    if sickness.is_some_and(|s| usize::from(s.0) == status) {
        causes.push("eating rotten food");
    }
    let mut causes: Vec<String> = causes.into_iter().map(String::from).collect();
    causes.extend(brought);
    // The special rooms that give it to everyone inside (the Special rooms page).
    causes.extend(
        (content.room_effects().defs().iter())
            .filter(|e| e.status.is_some_and(|s| usize::from(s.0) == status))
            .map(|e| format!("standing in {}", e.with_article())),
    );
    // The weapon abilities that leave it (the Weapon skills and Fighting styles pages).
    causes.extend(
        (content.weapon_abilities().iter())
            .filter(|a| {
                matches!(a.effect, warlock_core::content::WeaponDoes::Status { status: s, .. }
                    if usize::from(s.0) == status)
            })
            .map(|a| format!("{} (a weapon ability)", a.name)),
    );
    // The potions that lend it (the Potions page).
    causes.extend(
        (content.object_defs().iter())
            .filter(|o| {
                (o.drink.as_ref().and_then(|d| d.does.lends()))
                    .is_some_and(|(s, _)| usize::from(s.0) == status)
            })
            .map(|o| format!("drinking a {}", o.name)),
    );
    // Being caught unawares (the surprise rules, spec §8 "Surprise and sneak attacks").
    if let Some(rules) = content
        .surprise()
        .filter(|r| usize::from(r.status.0) == status)
    {
        causes.push(format!(
            "being caught unawares (an attack by someone you have not seen, or a hidden trap \
             springing under you) and failing the initiative check: 2d12 + your Dexterity \
             modifier against {}; a sleeper or a mimic in disguise always",
            rules.difficulty
        ));
    }
    let ending = (content.status_defs().iter())
        .filter(|s| s.follows.is_some_and(|f| usize::from(f.0) == status))
        .map(|s| format!("the end of {}", s.name));
    causes.extend(ending);
    if causes.is_empty() {
        "—".into()
    } else {
        capital(&causes.join("; "))
    }
}

/// The resource that brings status `status` while it is held (Rage brings Raging), by name.
fn brought_by(content: &Content, status: usize) -> Option<String> {
    (content.resources().iter())
        .find(|r| r.status.is_some_and(|s| usize::from(s.0) == status))
        .map(|r| r.name.clone())
}

/// What else a status does while it lasts and as it ends, in words: its fighting bonuses (as a
/// stance's), how much slower it makes every action, what it keeps you from doing, and what
/// follows it.
fn also(content: &Content, def: &StatusDef) -> String {
    let mut parts = Vec::new();
    let c = def.combat;
    let fighting: Vec<String> = [
        ("to hit", c.to_hit),
        ("damage", c.damage),
        ("Dodge", c.dodge),
        ("Parry", c.parry),
        ("Shield (on top of a held shield's)", c.shield),
    ]
    .into_iter()
    .filter(|&(_, n)| n != 0)
    .map(|(what, n)| format!("{what} {}", signed(n)))
    .collect();
    if !fighting.is_empty() {
        parts.push(fighting.join(", "));
    }
    let surprise = content
        .surprise()
        .map(|r| content.status(r.status).id.as_str());
    if surprise == Some(def.id.as_str()) {
        parts.push(
            "flat-footed: Dexterity and Dodge count nothing toward your Defence; no free blow \
             out of your turn (riposte, opportunist, gatekeeper, a point-blank shot)"
                .to_string(),
        );
    }
    if def.cost != 100 {
        parts.push(format!("every action takes {}% of the time", def.cost));
    }
    if def.attack_cost != 100 {
        parts.push(format!(
            "every attack takes {}% of the time",
            def.attack_cost
        ));
    }
    if !def.blocks.is_empty() {
        let doing: Vec<&str> = def.blocks.iter().map(|k| k.doing()).collect();
        let list = match doing.as_slice() {
            [init @ .., one, last] if !init.is_empty() => {
                format!("{}, {one}, or {last}", init.join(", "))
            }
            _ => doing.join(" or "),
        };
        parts.push(format!("you cannot {list}"));
    }
    if let Some(next) = def.follows {
        parts.push(format!("{} follows it", content.status(next).name));
    }
    if let Some(hold) = def.hold {
        parts.push(format!(
            "a step is a struggle to break free: a {} check against the holder's grip ({} \
             unless its attack says otherwise); made, you step, failed, the turn is lost",
            hold.check.name(),
            hold.difficulty
        ));
    }
    if let Some(light) = def.light {
        let radius = content.light(light).radius;
        parts.push(format!("you shed light {radius} cells around you"));
    }
    if let Some(vision) = def.vision {
        let eyes = &content.vision_profiles()[vision];
        parts.push(format!(
            "you see with {} eyes where they are better than yours",
            eyes.name
        ));
    }
    if !def.resist.is_empty() {
        let kinds: Vec<String> = (def.resist.iter())
            .map(|&(kind, r)| format!("{r}% {}", content.damage_type(kind).name))
            .collect();
        parts.push(format!("you resist {}, as armour does", kinds.join(", ")));
    }
    if parts.is_empty() {
        "—".into()
    } else {
        capital(&parts.join("; "))
    }
}

/// The status effects (spec §8 "Status effects"): how they show, what causes them, what they
/// cost, how they wear off, and what they do to healing and stats.
pub(super) fn status_effects(content: &Content) -> String {
    let mut out = String::from(
        "| Status | Seen on others | Caused by | Costs | Wears off | Healing | Stats | Cured by | \
         Also |\n|---|---|---|---|---|---|---|---|---|\n",
    );
    for (i, def) in content.status_defs().iter().enumerate() {
        let seen = def
            .seen
            .as_ref()
            .map_or_else(|| "no".to_string(), |s| format!("yes, \"{s}\""));
        let costs = match def.tick {
            Some(tick) if def.damage > 0 => {
                let hp = if def.damage == 1 {
                    "a hit point".to_string()
                } else {
                    format!("{} hit points", def.damage)
                };
                let faster = if def.max > 1 {
                    ", faster the worse it is"
                } else {
                    ""
                };
                format!("{hp} {}{faster}", every(tick))
            }
            Some(tick) if def.mana_damage > 0 => {
                let none = if def.mana_regen == 0 {
                    ", and none comes back"
                } else {
                    ""
                };
                format!("{} mana {}{none}", def.mana_damage, every(tick))
            }
            _ => "nothing".into(),
        };
        let diseased = content
            .disease_status()
            .is_some_and(|s| usize::from(s.0) == i);
        let eases = match def.eases {
            Some(e) if def.hold.is_some() => {
                let turns = def.hold.map(|h| h.turns.to_string()).unwrap_or_default();
                format!(
                    "after {turns} turns ({} each) at most; at once when the holder dies, leaves \
                     your side, or is held or slowed",
                    Duration::seconds(u64::from(e)).spoken()
                )
            }
            Some(e) if def.max == 1 => {
                format!("after {}", Duration::seconds(u64::from(e)).spoken())
            }
            Some(e) => format!("a degree {}", every(e)),
            None if def.per_action && def.max == 1 => {
                "after your next action (one given during an action outlasts it)".to_string()
            }
            None if def.per_action => {
                "a degree with each action you take (one given during an action outlasts it)"
                    .to_string()
            }
            None if diseased => "when the diseases are gone".to_string(),
            None if brought_by(content, i).is_some() => {
                let r = brought_by(content, i).unwrap_or_default();
                format!("when {} ends (the Resources page)", r.to_lowercase())
            }
            None => "only when cured".to_string(),
        };
        let stats = def.penalty.as_ref().map_or_else(
            || "—".to_string(),
            |p| {
                let lowered: Vec<String> = (p.by.iter())
                    .filter(|(_, by)| *by != 0)
                    .map(|(stat, by)| format!("{} {}", stat_link(stat), signed(i32::from(by))))
                    .collect();
                let when = if p.from > 1 { " when strong" } else { "" };
                let per = if p.per_degree { " a degree" } else { "" };
                format!("{}{per}{when}", lowered.join(", "))
            },
        );
        let cured = if def.cured_by.is_empty() {
            "—".to_string()
        } else {
            capital(&def.cured_by.join(", "))
        };
        let _ = writeln!(
            out,
            "| {} | {seen} | {} | {} | {} | {} | {stats} | {cured} | {} |",
            anchored(Term::Status, &def.id, &def.name),
            causes(content, i, def),
            capital(&costs),
            capital(&eases),
            capital(&pace(u32::from(def.regen))),
            also(content, def),
        );
    }
    out
}

/// The scores [`disease_checks`] gives the odds for.
const CHECK_SCORES: [u8; 8] = [4, 6, 8, 10, 12, 14, 16, 18];

/// Each disease's check (spec §8 "Checks": 2d12 plus the stat's modifier against the
/// difficulty, [`check::ways`]): the stat, the difficulty, and how often it is won by score.
pub(super) fn disease_checks(content: &Content) -> String {
    let scores: String = CHECK_SCORES.iter().map(|s| format!(" {s} |")).collect();
    let mut out = format!(
        "| Disease | Stat | Difficulty |{scores}\n|---|---|:---:|{}\n",
        ":---:|".repeat(CHECK_SCORES.len())
    );
    for (_, def) in content.disease_ids() {
        let odds: String = (CHECK_SCORES.iter())
            .map(|&s| {
                let modifier = stats::modifier(i32::from(s));
                format!(" {} |", percent(check::ways(modifier, def.difficulty), 144))
            })
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} |{odds}",
            capital(&def.name),
            def.check.name(),
            def.difficulty
        );
    }
    out
}

/// Days, as dice or a number: `1d3 days`, `1 day`.
fn days(dice: Dice) -> String {
    if dice.is_fixed() && dice.min() == 1 {
        "1 day".into()
    } else {
        format!("{dice} days")
    }
}

/// How a disease is caught, in words: the creatures' attacks that may pass it on (with the chance
/// a hit has), the terrain that does (with the chance an hour), and the other sources.
fn caught_by(content: &Content, disease: DiseaseId, def: &DiseaseDef) -> String {
    let mut ways: Vec<String> = attacks_causing(content, |what| {
        matches!(*what, Affliction::Disease(d) if d == disease).then(String::new)
    });
    if let Some(wading) = &def.wading {
        let lands: Vec<String> = (wading.terrain.iter())
            .map(|&t| content.terrain(t).name.clone())
            .collect();
        ways.push(format!(
            "crossing {} ({}% an hour)",
            lands.join(" or "),
            wading.per_hour
        ));
    }
    for source in &def.caught {
        ways.push(match source.as_str() {
            "rotten_food" => "eating rotten food".into(),
            other => other.replace('_', " "),
        });
    }
    if ways.is_empty() {
        "—".into()
    } else {
        capital(&ways.join("; "))
    }
}

/// The diseases (spec §8 "Diseases"): how they are caught and resisted (the stat, and how often
/// an average one makes the check), how long they incubate, how often they progress, what a
/// failed check does, how they are shaken off, whether they kill, and healing meanwhile.
pub(super) fn diseases(content: &Content) -> String {
    let mut out = String::from(
        "| Disease | Caught by | Resisted with (an average score) | Incubation | Progresses | \
         A failed check costs | Shaken off | Can kill | Healing |\n|---|---|---|---|---|---|---|:---:|---|\n",
    );
    for (disease, def) in content.disease_ids() {
        let resisted = format!(
            "{}, {}",
            stat_link(def.check),
            percent(check::ways(0, def.difficulty), 144)
        );
        let progresses = match def.interval {
            1 => "Every day".to_string(),
            n => format!("Every {n} days"),
        };
        let mut costs: Vec<String> = (def.damage.iter())
            .filter(|(_, d)| !d.is_zero())
            .map(|(stat, d)| format!("{d} {}", stat_link(stat)))
            .collect();
        if let Some(hp) = def.hp {
            costs.push(format!("{hp} hit points"));
        }
        let shaken = match def.recovery {
            0 => "Never by itself".to_string(),
            1 => "A check won".to_string(),
            2 => "Two checks won in a row".to_string(),
            n => format!("{n} checks won in a row"),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {resisted} | {} | {progresses} | {} | {shaken} | {} | {} |",
            anchored(Term::Disease, &def.id, &capital(&def.name)),
            caught_by(content, disease, def),
            capital(&days(def.incubation)),
            costs.join(", "),
            yes(def.kills),
            capital(&pace(u32::from(disease_regen(content, def)))),
        );
    }
    out
}

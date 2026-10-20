//! Combat (spec §8 "Combat"): weapons, missile weapons, ammunition, armour, stances, the 2d12
//! odds, hit points, and regeneration.

use super::actions::at_normal_pace;
use super::objects::goes_on;
use super::order;
use super::text::capital;
use std::fmt::Write as _;
use warlock_core::combat;
use warlock_core::content::{ActionKind, BASE_TURN, Content, price};

/// The natural rolls of a threat range: `22–24`, or `24` alone.
fn threat_range(threat: u8) -> String {
    if i32::from(threat) >= combat::SURE_HIT {
        combat::SURE_HIT.to_string()
    } else {
        format!("{threat}–{}", combat::SURE_HIT)
    }
}

/// `part` of `of` as a percentage, one decimal below 10 %, whole above.
pub(super) fn percent(part: u32, of: u32) -> String {
    let p = f64::from(part) * 100.0 / f64::from(of);
    if p < 10.0 && p.fract() != 0.0 {
        format!("{p:.1}%")
    } else {
        format!("{p:.0}%")
    }
}

/// The weapons and missile weapons: their damage dice, critical (threat range and multiplier),
/// parry, the Strength a bow needs, and how they are held. Rules from data, nothing about who
/// wields them.
pub(super) fn weapons(content: &Content) -> String {
    let mut out = String::from(
        "| Weapon | Damage | Critical on | Critical damage | Parry | Needs Strength | Held | \
         Thrown | Weapon skill | A blow takes |\n|---|:---:|:---:|:---:|:---:|:---:|---|:---:|---|---:|\n",
    );
    let melee = content.action_costs().cost(ActionKind::Melee);
    for (id, o) in order::objects(content) {
        let Some(attack) = &o.attack else { continue };
        let in_hand = (content.slots().iter()).any(|s| s.hand && o.slot.as_ref() == Some(&s.fits));
        let blow = if in_hand {
            at_normal_pace(price(o.attack_cost.unwrap_or(melee), BASE_TURN))
        } else {
            "-".to_string()
        };
        let parry = if o.parry > 0 {
            format!("+{}", o.parry)
        } else {
            "-".into()
        };
        let strength = attack.strength.map_or("-".to_string(), |s| s.to_string());
        let held = if o.launches.is_some() {
            format!("{}, shoots", goes_on(content, id).to_lowercase())
        } else {
            goes_on(content, id).to_lowercase()
        };
        let thrown = match o.range {
            Some(range) if o.is_thrown() => format!("{range} cells"),
            _ => "-".into(),
        };
        let skill = (o.skill).map_or("-".to_string(), |id| {
            content.weapon_skills().skill(id).name.clone()
        });
        let _ = writeln!(
            out,
            "| {} | {} | {} | ×{} | {parry} | {strength} | {held} | {thrown} | {skill} | {blow} |",
            capital(&o.name),
            attack.damage,
            threat_range(attack.threat),
            attack.crit,
        );
    }
    let _ = writeln!(
        out,
        "\nBare hands: {} (plus your Strength, as every blow), critical on {}, ×{}.",
        combat::Attack::unarmed().damage,
        threat_range(combat::DEFAULT_THREAT),
        combat::DEFAULT_CRIT
    );
    out
}

/// The light weapons, in name order, as a sentence: "The light weapons are the dagger, ... and
/// the stiletto." (spec §7: `light_weapon`).
pub(super) fn light_weapons(content: &Content) -> String {
    let mut names: Vec<&str> = (order::objects(content).into_iter())
        .filter(|(_, o)| o.light_weapon)
        .map(|(_, o)| o.name.as_str())
        .collect();
    names.sort_unstable();
    let the: Vec<String> = names.iter().map(|n| format!("the {n}")).collect();
    match the.as_slice() {
        [] => "No weapon is light.\n".into(),
        [one] => format!("The one light weapon is {one}.\n"),
        [rest @ .., last] => format!("The light weapons are {}, and {last}.\n", rest.join(", ")),
    }
}

/// What a launcher kind of ammunition is called: every object of that kind, in the plural, in
/// name order ("arrows, flaming arrows").
fn ammunition_of(content: &Content, kind: &str) -> String {
    let names: Vec<&str> = (order::objects(content).into_iter())
        .filter(|(_, o)| o.ammo.as_deref() == Some(kind))
        .map(|(_, o)| o.plural.as_str())
        .collect();
    names.join(", ")
}

/// The missile weapons (ranged combat): their damage dice, critical, the Strength a bow needs,
/// how far they shoot, what they shoot, and how long a shot takes.
pub(super) fn launchers(content: &Content) -> String {
    let mut out = String::from(
        "| Missile weapon | Damage | Critical on | Critical damage | Needs Strength | Range | \
         Shoots | A shot takes |\n|---|:---:|:---:|:---:|:---:|:---:|---|---:|\n",
    );
    let costs = content.action_costs();
    for (_, o) in order::objects(content) {
        let (Some(attack), Some(kind)) = (&o.attack, &o.launches) else {
            continue;
        };
        let strength = attack.strength.map_or("-".to_string(), |s| s.to_string());
        let mut cost = costs.cost(ActionKind::Shoot);
        if o.reloads {
            cost += costs.cost(ActionKind::Reload);
        }
        let takes = at_normal_pace(price(cost, BASE_TURN));
        let takes = if o.reloads {
            format!("{takes} (reloading)")
        } else {
            takes
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | ×{} | {strength} | {} cells | {} | {takes} |",
            capital(&o.name),
            attack.damage,
            threat_range(attack.threat),
            attack.crit,
            o.range.unwrap_or(0),
            capital(&ammunition_of(content, kind)),
        );
    }
    out
}

/// The ammunition (what shoots it, its chance to break on a hit, whether it burns where it
/// lands) and the throwing weapons (their damage, critical, and how far they are thrown).
pub(super) fn ammunition(content: &Content) -> String {
    let mut out = String::from(
        "| Ammunition | Shot from | Breaks on a hit | Where it lands |\n|---|---|:---:|---|\n",
    );
    let all = order::objects(content);
    for (_, o) in &all {
        let Some(kind) = &o.ammo else { continue };
        let from: Vec<&str> = (all.iter())
            .filter(|(_, l)| l.launches.as_ref() == Some(kind))
            .map(|(_, l)| l.name.as_str())
            .collect();
        let breaks = if o.breaks > 0 {
            format!("{}%", o.breaks)
        } else {
            "never".into()
        };
        let lands = match o.lit.map(|l| content.object(l)) {
            Some(lit) => match lit.burning {
                Some(burning) => format!(
                    "burns as {} for {}, lighting {} cells around",
                    lit.describe(1),
                    burning.lasts.spoken(),
                    content.light(burning.light).radius
                ),
                None => "-".into(),
            },
            None => "lies there, to be picked up".into(),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {breaks} | {lands} |",
            capital(&o.name),
            capital(&from.join(", ")),
        );
    }
    out.push_str(
        "\n| Throwing weapon | Damage | Critical on | Critical damage | Range | Breaks on a hit \
         |\n|---|:---:|:---:|:---:|:---:|:---:|\n",
    );
    for (_, o) in &all {
        let (Some(attack), Some(range), true) = (&o.attack, o.range, o.is_thrown()) else {
            continue;
        };
        let breaks = if o.breaks > 0 {
            format!("{}%", o.breaks)
        } else {
            "never".into()
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | ×{} | {range} cells | {breaks} |",
            capital(&o.name),
            attack.damage,
            threat_range(attack.threat),
            attack.crit,
        );
    }
    out
}

/// Armour and shields: what each adds to Defence, in which category, its Protection, and the
/// most Dexterity may add while it is worn. Body armour before shields before the other armour
/// (helmets, cloaks, gloves, boots: Protection only), the least Protection first, then the
/// least Defence, then by name.
pub(super) fn armour(content: &Content) -> String {
    let mut out = String::from(
        "| Armour or shield | Worn on | Defence | Category | Protection | Most Dexterity adds | \
         Bash |\n|---|---|:---:|---|:---:|:---:|:---:|\n",
    );
    let mut rows: Vec<_> = (order::objects(content).into_iter())
        .filter_map(|(id, o)| match (o.armour, o.shield, o.protection) {
            (0, 0, 0) => None,
            (0, 0, _) => Some((id, o, (2, 0), None)),
            (a, s, _) if a >= s => Some((id, o, (0, a), Some("Armour"))),
            (_, s, _) => Some((id, o, (1, s), Some("Shield"))),
        })
        .collect();
    rows.sort_by_key(|&(_, o, (group, defence), _)| {
        (group, o.protection, defence, o.name.to_lowercase())
    });
    let bonus = |n: i32| {
        if n > 0 { format!("+{n}") } else { "-".into() }
    };
    let number = |n: i32| {
        if n > 0 { n.to_string() } else { "-".into() }
    };
    for (id, o, (_, defence), category) in rows {
        let defence = if category.is_some() {
            bonus(defence)
        } else {
            "-".into()
        };
        let max_dex = o
            .max_dex
            .map_or("no limit".to_string(), |m| format!("+{m}"));
        let bash = (o.bash).map_or("-".to_string(), |a| a.damage.to_string());
        let _ = writeln!(
            out,
            "| {} | {} | {defence} | {} | {} | {max_dex} | {bash} |",
            capital(&o.name),
            goes_on(content, id).to_lowercase(),
            category.unwrap_or("-"),
            number(o.protection),
        );
    }
    out
}

/// The weapons and missiles that pierce armour (`penetration`: how much stays hidden), in name
/// order, as a sentence.
pub(super) fn piercing(content: &Content) -> String {
    let mut names: Vec<&str> = (order::objects(content).into_iter())
        .filter(|(_, o)| o.penetration > 0)
        .map(|(_, o)| o.name.as_str())
        .collect();
    names.sort_unstable();
    let the: Vec<String> = names.iter().map(|n| format!("the {n}")).collect();
    match the.as_slice() {
        [] => "Nothing pierces armour.\n".into(),
        [one] => format!("Only {one} pierces armour.\n"),
        [one, two] => format!("{} and {two} pierce armour.\n", capital(one)),
        [rest @ .., last] => format!("{}, and {last} pierce armour.\n", capital(&rest.join(", "))),
    }
}

/// The combat stances: what each does to the player's attacks, Defence, and speed (percent for
/// costs and noise; "-" for no change).
pub(super) fn stances(content: &Content) -> String {
    let bonus = |n: i32| match n {
        0 => "-".to_string(),
        n if n > 0 => format!("+{n}"),
        n => format!("\u{2212}{}", -n),
    };
    let percent = |n: u32| {
        if n == 100 {
            "-".to_string()
        } else {
            format!("{n}%")
        }
    };
    let mut out = String::from(
        "| Stance | To hit | Damage | Dodge | Parry | Attacks cost | Fleeing steps cost | \
         Fleeing steps' noise | Weapon marks count |\n\
         |---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|\n",
    );
    for s in content.stances().all() {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            s.name,
            bonus(s.to_hit),
            bonus(s.damage),
            bonus(s.dodge),
            bonus(s.parry),
            percent(s.attack_cost),
            percent(s.flee_cost),
            percent(s.flee_noise),
            percent(s.weapon_mark),
        );
    }
    out
}

/// How often an attack hits by what its roll needs (Defence minus to-hit): the 2d12 bell.
pub(super) fn hit_chances(_content: &Content) -> String {
    let mut out = String::from("| The roll needs | Hits |\n|:---:|:---:|\n");
    for needed in [4, 6, 8, 10, 11, 12, 13, 14, 15, 16, 18, 20, 22, 24] {
        let _ = writeln!(
            out,
            "| {needed} | {} |",
            percent(combat::hit_ways(0, needed), 144)
        );
    }
    out
}

/// How often an attack roll threatens a critical, by threat range, and how often one confirms
/// against Defence 10 at +0.
pub(super) fn threats(_content: &Content) -> String {
    let mut out = String::from(
        "| Critical on | A roll threatens | Confirmed, with no bonus against Defence 10 | Like D&D's |\n\
         |:---:|:---:|:---:|:---:|\n",
    );
    for (threat, dnd) in [(22, "20"), (20, "19–20"), (19, "18–20")] {
        let threatens: u32 = (threat..=combat::SURE_HIT).map(combat::ways).sum();
        let hit = combat::hit_ways(0, combat::BASE_DEFENCE);
        let confirmed: u32 = (threat..=combat::SURE_HIT)
            .filter(|&n| combat::hits(n, 0, combat::BASE_DEFENCE))
            .map(combat::ways)
            .sum::<u32>()
            * hit;
        let _ = writeln!(
            out,
            "| {} | {} | {} | {dnd} |",
            threat_range(threat as u8),
            percent(threatens, 144),
            percent(confirmed, 144 * 144),
        );
    }
    out
}

/// What each race and each class adds to the hit points.
pub(super) fn hit_points(content: &Content) -> String {
    let mut out = String::from("| Race | Hit points |\n|---|:---:|\n");
    for r in &content.races().list {
        let _ = writeln!(out, "| {} | {} |", r.name, r.hp);
    }
    out.push_str("\n| Class | Hit points |\n|---|:---:|\n");
    for c in content.classes() {
        let _ = writeln!(out, "| {} | {} |", c.name, c.hp);
    }
    out
}

/// How long a hit point takes to come back, by Toughness.
pub(super) fn regeneration(_content: &Content) -> String {
    let mut out = String::from("| Toughness | A hit point back every |\n|:---:|---|\n");
    for toughness in [3, 6, 8, 10, 12, 14, 16, 18, 20, 22, 25] {
        let seconds = combat::regen_seconds(toughness);
        let (m, s) = (seconds / 60, seconds % 60);
        let minutes = if m == 1 {
            "1 minute".to_string()
        } else {
            format!("{m} minutes")
        };
        let text = match s {
            0 => minutes,
            s => format!("{minutes} and {s} seconds"),
        };
        let label = if toughness == 25 {
            "25 or more".to_string()
        } else {
            toughness.to_string()
        };
        let _ = writeln!(out, "| {label} | {text} |");
    }
    out
}

//! What an item is, in words, as the player knows it (spec §6 "Item description", §8 "Magic
//! items"): its base stats, and its affixes once it is identified. The item description screen
//! shows these lines; they read an item only through [`Stack::known_magic`], so an unidentified
//! item's affixes cannot leak.
//!
//! Numbers that are item properties are shown, as in Path of Exile (dice, criticals, Defence
//! bonuses, Protection, an affix's rolled value); speed is said in words ("attacks faster") and
//! penetration only as a fact ("Pierces armour"), never as a number, and nothing about who wears
//! it (no to-hit total, no modifier).

use super::{AffixKind, Effect, Rolled};
use crate::combat::SURE_HIT;
use crate::content::{ActionKind, Content, ItemKind, ObjectDef, Rarity, weapon_noun};
use crate::object::{Stack, format_weight};

/// What a line of a description is, for how it is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    /// What it is: rarity, base, what it is made of, weight.
    Base,
    /// A base stat: damage, Defence, range.
    Stat,
    /// The item level, or that it is unidentified.
    Note,
    /// An affix: its kind ("Prefix"), name, and what it does.
    Affix(AffixKind),
}

/// One line of a description.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub kind: LineKind,
    pub text: String,
}

fn line(kind: LineKind, text: impl Into<String>) -> Line {
    Line {
        kind,
        text: text.into(),
    }
}

/// What a description shows of worth (spec §8 "Class abilities": Keen appraisal, Appraiser):
/// nothing unless the player has an eye for it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Appraisal {
    /// What the thing is worth, in gold (its value: never what its unknown magic adds, which
    /// the value leaves out).
    pub worth: bool,
    /// What each gem in it is worth.
    pub gems: bool,
}

/// [`lines`] for a player with an eye for worth: a line of what it is worth after what it is
/// (`Worth 12 gold`; `Worth 3 gold each, 9 gold in all` for a stack), and each gem's worth after
/// its line (`Pommel: small emerald, fine cut (worth 40 gold)`), as `appraisal` shows.
pub fn appraised(stack: &Stack, content: &Content, appraisal: Appraisal) -> Vec<Line> {
    let mut out = lines(stack, content);
    // The first line says what it is; a line for each gem in it follows.
    let gems_at = 1;
    let parts = stack.parts(content);
    if appraisal.gems
        && let Some(gems) = content.gems()
    {
        let worths = (parts.iter()).filter_map(|p| {
            let gem = content.material(p.material).gem.as_ref();
            p.gem
                .map(|cutting| gem.map_or(0, |g| gems.value(g, cutting)))
        });
        for (line, worth) in out[gems_at..].iter_mut().zip(worths) {
            line.text += &format!(" (worth {worth} gold)");
        }
    }
    if appraisal.worth {
        let total = stack.value(content);
        let text = match stack.count {
            1 => format!("Worth {total} gold"),
            n => format!(
                "Worth {} gold each, {total} gold in all",
                total / u64::from(n)
            ),
        };
        let at = 1 + parts.iter().filter(|p| p.gem.is_some()).count();
        out.insert(at.min(out.len()), line(LineKind::Base, text));
    }
    out
}

/// The description of `stack`, as the player knows it: what it is (`Magic long sword · steel
/// blade, leather grip · 1.8 kg`: every part, as materials show identified or not), each gem in
/// it with its size and cut (`Pommel: small emerald, fine cut`; a loose gem's `Gem: tiny
/// garnet, uncut`), its base stats, and its affixes if identified (`Sharp: +2 damage`) or a note
/// that its magic is unknown.
pub fn lines(stack: &Stack, content: &Content) -> Vec<Line> {
    let def = content.object(stack.object);
    let mut out = Vec::new();
    let (name, plural) = stack.names(content);
    let base = if stack.count == 1 {
        name
    } else {
        format!("{} {plural}", stack.count)
    };
    let what = match stack.rarity() {
        Rarity::Normal => capitalized(&base),
        rarity => format!("{} {base}", capitalized(rarity.word())),
    };
    let parts = stack.parts(content);
    let made = content.describe_parts(&parts);
    let weight = format_weight(stack.weight(content));
    out.push(line(LineKind::Base, format!("{what} · {made} · {weight}")));
    for part in parts.iter() {
        let Some(cutting) = part.gem else { continue };
        let kind = (part.part).map_or("gem", |p| {
            content.part_defs()[usize::from(p.0)].name.as_str()
        });
        let gem = crate::gem::phrase(content, part.material, cutting);
        out.push(line(
            LineKind::Base,
            format!("{}: {gem}", capitalized(kind)),
        ));
    }
    // Its own stats as it is made: another material's damage and Protection (#193).
    let (_, damage) = crate::variant::weapon(def, &parts, content);
    out.extend(
        stats(def, content, damage, stack.base_protection(content))
            .into_iter()
            .map(|t| line(LineKind::Stat, t)),
    );
    if stack.rarity() == Rarity::Normal {
        return out;
    }
    match stack.known_magic() {
        None => out.push(line(
            LineKind::Note,
            "Unidentified: what its magic does is unknown, though it works all the same.",
        )),
        Some(magic) => {
            out.push(line(LineKind::Note, format!("Item level {}", magic.level)));
            let kind = def.kind.unwrap_or(ItemKind::Ring);
            for rolled in magic.rolled(content) {
                let text = format!("{}: {}", rolled.def.name, phrase(&rolled, kind, content));
                out.push(line(LineKind::Affix(rolled.def.kind), text));
            }
        }
    }
    out
}

/// What `rolled`, on an item of `kind`, does, in plain language: "+2 damage", "adds 1d4
/// damage", "+1 to Strength", "attacks faster".
pub fn phrase(rolled: &Rolled<'_>, kind: ItemKind, content: &Content) -> String {
    let effect = rolled.def.effect;
    let v = rolled.value;
    let local = effect.is_local_on(kind);
    let every = |text: String| {
        if local {
            text
        } else {
            format!("{text} with every attack")
        }
    };
    match effect {
        Effect::ToHit => every(format!("+{v} to hit")),
        Effect::Damage => every(format!("+{v} damage")),
        Effect::ExtraDamage => format!("adds 1d{v} damage"),
        Effect::Elemental => format!("adds 1d{v} {} damage", kind_name(rolled, content)),
        Effect::Resistance => format!("+{v}% {} resistance", kind_name(rolled, content)),
        Effect::Threat => every(format!("criticals threaten on {v} more {}", rolls(v))),
        Effect::Crit => every(format!("+{v} to the critical damage multiplier")),
        Effect::AttackSpeed if local => format!("attacks {}", faster(v)),
        Effect::AttackSpeed => format!("all your attacks are {}", faster(v)),
        Effect::Range => format!("+{v} {} of range", cells(v)),
        Effect::Draw => format!("needs {v} less Strength to draw"),
        Effect::Durability => format!("breaks {v}% less often"),
        Effect::Parry => format!("+{v} to Parry"),
        Effect::Armour => format!("+{v} to Armour"),
        Effect::Protection => format!("+{v} to Protection"),
        Effect::Shield => format!("+{v} to Shield"),
        Effect::MaxDex => format!("lets Dexterity add {v} more to Defence"),
        Effect::Weight => format!("{v}% lighter"),
        Effect::Dodge => format!("+{v} to Dodge"),
        Effect::Deflection => format!("+{v} to Deflection"),
        Effect::Natural => format!("+{v} to natural armour"),
        Effect::MaxHp => format!("+{v} to maximum hit points"),
        Effect::Regeneration => format!("hit points come back {v}% faster"),
        Effect::Stat(stat) => format!("+{v} to {}", stat.name()),
        Effect::MoveSpeed => format!("you move {}", faster(v)),
        Effect::Carry => format!("+{v} kg carrying capacity"),
        Effect::Stealth => format!("your footsteps are {v}% quieter"),
        Effect::Light => {
            let tier = rolled.def.tier(rolled.tier);
            let radius = (tier.and_then(|t| t.light)).map_or(0, |id| content.light(id).radius);
            match radius {
                ..=2 => "glows faintly".into(),
                3 => "glows".into(),
                _ => "glows brightly".into(),
            }
        }
        Effect::Skill => match rolled.def.skill {
            Some(id) => format!("+{v} to {}", content.skill(id).name),
            None => format!("+{v} to a skill"),
        },
    }
}

/// The damage type a typed affix names ("fire"); "some kind of" for one that names none.
fn kind_name(rolled: &Rolled<'_>, content: &Content) -> String {
    (rolled.def.damage_type).map_or_else(
        || "some kind of".to_string(),
        |k| content.damage_type(k).name.clone(),
    )
}

/// How much faster, in words: speed numbers stay hidden.
fn faster(percent: i32) -> &'static str {
    match percent {
        ..=7 => "a little faster",
        8..=12 => "faster",
        _ => "much faster",
    }
}

fn rolls(n: i32) -> &'static str {
    if n == 1 { "roll" } else { "rolls" }
}

fn cells(n: i32) -> &'static str {
    if n == 1 { "cell" } else { "cells" }
}

/// The base stats of `def` in words: how it is held or worn, its damage and criticals, how far it
/// shoots or is thrown, what it needs, how quick it is, its Defence, and whether it breaks.
pub fn base_stats(def: &ObjectDef, content: &Content) -> Vec<String> {
    stats(def, content, 0, def.protection)
}

/// [`base_stats`] of one made of another material: `damage` more to its damage dice and
/// `protection` its Protection.
fn stats(def: &ObjectDef, content: &Content, damage: i32, protection: i32) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(held) = how_held(def, content) {
        out.push(held);
    }
    if let Some(attack) = &def.attack {
        let critical = if i32::from(attack.threat) >= SURE_HIT {
            format!("{SURE_HIT}")
        } else {
            format!("{}–{SURE_HIT}", attack.threat)
        };
        let shot = if def.launches.is_some() {
            " a shot"
        } else {
            ""
        };
        out.push(format!(
            "Damage {}{shot}; criticals on {critical}, ×{} damage",
            attack.damage.plus(damage),
            attack.crit
        ));
        // Its kinds of damage (spec §8 "Damage types"): "Slashing or piercing damage".
        let kinds: Vec<&str> = (attack.kinds.iter())
            .map(|k| content.damage_type(k).name.as_str())
            .collect();
        if !kinds.is_empty() {
            out.push(format!("{} damage", capitalized(&kinds.join(" or "))));
        }
        if let Some(strength) = attack.strength {
            out.push(format!("Needs Strength {strength} to draw"));
        }
        if def.attack_cost.is_some() || def.launches.is_some() {
            let action = if def.launches.is_some() {
                ActionKind::Shoot
            } else {
                ActionKind::Melee
            };
            let normal = content.action_costs().cost(action);
            let cost = def.attack_cost.unwrap_or(normal);
            if cost < normal {
                out.push("Quick to strike".into());
            } else if cost > normal {
                out.push("Slow to strike".into());
            }
        }
    }
    match (def.range, def.launches.is_some()) {
        (Some(range), true) => out.push(format!("Shoots up to {range} cells")),
        (Some(range), false) => out.push(format!("Thrown up to {range} cells")),
        _ => {}
    }
    if def.reloads {
        out.push("Reloaded after every shot".into());
    }
    if let Some(added) = &def.added {
        let kind = &content.damage_type(added.kind).name;
        out.push(format!("Adds {} {kind} damage", added.dice));
    }
    // How much armour it ignores stays hidden, as the soak does: only that it does.
    if def.penetration > 0 {
        out.push("Pierces armour".into());
    }
    if def.parry > 0 {
        out.push(format!("Parry +{}", def.parry));
    }
    if def.armour > 0 {
        out.push(format!("Armour +{}", def.armour));
    }
    if protection > 0 {
        let against: Vec<String> = (def.protects.iter())
            .map(|&(k, more)| {
                let sign = if more < 0 { "\u{2212}" } else { "+" };
                format!(
                    "{sign}{} against {}",
                    more.abs(),
                    content.damage_type(k).name
                )
            })
            .collect();
        if against.is_empty() {
            out.push(format!("Protection {protection}"));
        } else {
            out.push(format!("Protection {protection} ({})", against.join(", ")));
        }
    }
    let resists: Vec<String> = (def.resist.iter())
        .filter(|&&(_, p)| p > 0)
        .map(|&(k, p)| format!("{} {p}%", content.damage_type(k).name))
        .collect();
    if !resists.is_empty() {
        out.push(format!("Resists {}", resists.join(", ")));
    }
    let weak: Vec<String> = (def.resist.iter())
        .filter(|&&(_, p)| p < 0)
        .map(|&(k, p)| format!("{} {}%", content.damage_type(k).name, -p))
        .collect();
    if !weak.is_empty() {
        out.push(format!("Vulnerable to {}", weak.join(", ")));
    }
    if def.shield > 0 {
        out.push(format!("Shield +{}", def.shield));
    }
    if let Some(max) = def.max_dex {
        out.push(format!(
            "Dexterity adds at most +{max} to Defence while worn"
        ));
    }
    if def.breaks > 0 {
        out.push(format!("Breaks on a hit {}% of the time", def.breaks));
    }
    out
}

/// How it is held or worn: "Long blade, for the main hand", "Bow: readied as a missile weapon,
/// shoots arrows", "Worn on the head".
fn how_held(def: &ObjectDef, content: &Content) -> Option<String> {
    let slot = def.slot.as_ref()?;
    let slot = content.slots().iter().find(|s| &s.fits == slot)?;
    let category = weapon_noun(def, content).map(|c| capitalized(&c));
    let place = slot.name.to_lowercase();
    Some(match (category, slot.action) {
        (Some(c), ActionKind::Wield) if def.two_handed => format!("{c}, for both hands"),
        (Some(c), ActionKind::Wield) => format!("{c}, for the {place}"),
        (Some(c), _) if def.launches.is_some() => {
            let ammo = (def.launches.as_deref())
                .and_then(|kind| content.ammunition(kind))
                .map_or("ammunition".to_string(), |o| o.plural.clone());
            format!("{c}, readied as a missile weapon; shoots {ammo}")
        }
        (_, _) if def.ammo.is_some() => {
            let from: Vec<String> = (content.object_defs().iter())
                .filter(|o| o.launches.is_some() && o.launches == def.ammo)
                .filter_map(|o| weapon_noun(o, content))
                .map(|c| format!("{c}s"))
                .fold(Vec::new(), |mut all, c| {
                    if !all.contains(&c) {
                        all.push(c);
                    }
                    all
                });
            format!("Ammunition, quivered; shot from {}", from.join(" and "))
        }
        (_, ActionKind::Wield) => format!("Held in the {place}"),
        (_, ActionKind::Ready) => format!("Readied: {place}"),
        _ if slot.fits == "ring" => "Worn on a finger".into(),
        _ => format!("Worn: {place}"),
    })
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}

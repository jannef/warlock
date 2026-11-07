//! Armed monsters' to-hit (#1169): a type that fights with a weapon in hand has no flat to-hit;
//! its weapon skill's ranks (and its fighting style's, where its grip has one that aims) bring
//! the weapon to about `L/4`, as the variance pass (#1134) set the natural attackers. A caster
//! (a type with spells) aims at the casters' `L/4 − L/8`. No rank passes what a trained warrior
//! of the type's level holds ([`cap`]), so a heavy weapon (great blades, axes, polearms, whose
//! ranks give damage before aim) may stay below the mark: then it holds the most its ranks at
//! the cap give.

use super::{Content, CreatureDef, Grip, Keep, WeaponSkillId};
use crate::weapon_skills::WeaponSkills;

/// One melee weapon a type may wield: its id, its own to-hit, its skill, and the grip it is
/// fought in at worst (beside a shield whenever the type may carry one).
struct Weapon {
    id: String,
    to_hit: i32,
    skill: Option<WeaponSkillId>,
    grip: Grip,
}

/// The melee weapons the type's gear may put in its hand (none: it is not armed).
fn weapons(content: &Content, def: &CreatureDef) -> Vec<Weapon> {
    let lines = || def.gear.lines.iter().flat_map(|l| &l.one_of);
    let shield =
        (lines().flat_map(|(stacks, _)| stacks)).any(|s| content.object(s.object).shield > 0);
    let mut out = Vec::new();
    for (stacks, keep) in lines() {
        if *keep != Keep::Wield {
            continue;
        }
        for s in stacks {
            let o = content.object(s.object);
            let Some(attack) = &o.attack else {
                continue;
            };
            if o.launches.is_some() || o.ammo.is_some() {
                continue;
            }
            let grip = if o.two_handed {
                Grip::TwoHanded
            } else if shield {
                Grip::WeaponAndShield
            } else {
                Grip::OneWeapon
            };
            out.push(Weapon {
                id: o.id.clone(),
                to_hit: attack.to_hit,
                skill: o.skill,
                grip,
            });
        }
    }
    out
}

/// The weapon's to-hit with `ranks`: its own, its skill's, and its grip's style's (no Strength:
/// a natural attack's `to_hit` has none either).
fn to_hit(content: &Content, ranks: &WeaponSkills, w: &Weapon) -> i32 {
    let table = content.weapon_skills();
    let style = table.style_for(w.grip, &w.skill.into_iter().collect::<Vec<_>>());
    let of =
        |id: Option<WeaponSkillId>| id.map_or(0, |id| table.benefit(id, ranks.rank(id)).to_hit);
    w.to_hit + of(w.skill) + of(style)
}

/// The mark: `L/4`, a caster's `L/4 − L/8`.
fn mark(def: &CreatureDef) -> i32 {
    let caster = !def.spells.is_empty();
    i32::from(def.level / 4) - if caster { i32::from(def.level / 8) } else { 0 }
}

/// The rank a trained warrior holds at `level` (the balance probe's `rank`, "Weapon pace",
/// `docs/balance.md`): an armed type's raised ranks stop there.
pub(super) fn cap(level: u8) -> u8 {
    match level {
        0..=2 => 2,
        3..=4 => 3,
        5..=6 => 4,
        7..=9 => 5,
        10..=14 => 6,
        15..=22 => 7,
        23..=31 => 8,
        32..=41 => 9,
        _ => 10,
    }
}

/// The most the weapon's to-hit reaches with its skill and style at the cap (or the type's own
/// ranks, where higher).
fn reach(content: &Content, def: &CreatureDef, w: &Weapon) -> i32 {
    let table = content.weapon_skills();
    let style = table.style_for(w.grip, &w.skill.into_iter().collect::<Vec<_>>());
    let cap = cap(def.level);
    let ranks = [w.skill, style]
        .into_iter()
        .flatten()
        .map(|id| (id, def.weapon_skills.rank(id).max(cap)));
    to_hit(content, &WeaponSkills::from_ranks(ranks), w)
}

#[test]
fn armed_types_aim_near_a_quarter_of_their_level_by_their_ranks() {
    let content = Content::builtin().expect("content");
    let mut armed = 0;
    for def in content.creature_defs() {
        for w in weapons(&content, def) {
            armed += 1;
            let (id, level) = (&def.id, def.level);
            let hit = to_hit(&content, &def.weapon_skills, &w);
            let mark = mark(def);
            let low = mark.min(reach(&content, def, &w));
            assert!(
                (low..=mark + 1).contains(&hit),
                "{id} (level {level}) with a {}: to-hit {hit}, not {low} to {}",
                w.id,
                mark + 1
            );
        }
    }
    assert!(armed > 50, "the armed types are found ({armed})");
}

#[test]
fn an_animated_weapons_rank_brings_it_near_a_quarter_of_its_level() {
    // The blades and the panoplies wield a weapon of any melee skill, drawn as they spawn, at
    // their tier's `weapon_rank` (no style: a panoply's Sword and board gives no aim).
    let content = Content::builtin().expect("content");
    let table = content.weapon_skills();
    let melee: Vec<WeaponSkillId> = (table.ids())
        .filter(|(_, s)| s.role == super::Role::Melee)
        .map(|(id, _)| id)
        .collect();
    let mut seen = 0;
    for def in content.creature_defs() {
        let Some(tier) = content.animated().family_tier(def.animated.as_ref()) else {
            continue;
        };
        let Some(ramp) = tier.weapon_rank else {
            continue;
        };
        seen += 1;
        let rank = u8::try_from(ramp.at(def.level)).expect("a rank");
        let mark = mark(def);
        for &skill in &melee {
            let hit = table.benefit(skill, rank).to_hit;
            let most = table.benefit(skill, cap(def.level)).to_hit;
            let low = mark.min(most) - 1;
            assert!(
                (low..=mark + 1).contains(&hit),
                "{} (level {}) in {}: to-hit {hit} at rank {rank}, not {low} to {}",
                def.id,
                def.level,
                table.skill(skill).id,
                mark + 1
            );
        }
    }
    assert!(seen > 0, "the animated weapons are found");
}

//! The balance probe's school pace (the spell scaling design §8.4, §12.4; S7): the ranks a
//! caster's school and caster style really reach by level, measured on the real curve. Each game
//! day of a level ([`super::training::days`]) the caster spends one pool (the night's rest
//! refills it) on its pace spell ([`pace_spell`]): the casts are its maximum mana over the
//! spell's cost (the rank's mana cut included, as in play), a healer's never more than the day's
//! kills (one heal a fight, as the probe's priest casts). Each cast lands on a monster of the
//! caster's level and goes through `World::practise`, the hook `practise_cast` calls: once for
//! the spell's school, once for the caster style it is cast in from where the caster stands in
//! plain gear (`Actor::standing_aim`: armoured in hindering armour), so `[marks]`, the school's
//! curve, and the talents' marks (Weapon drill) are the game's own.

use super::{Build, Cast, Gear, casts, dress, roll_mean};
use crate::content::WeaponSkillId;
use crate::world::World;

/// What one level's casting was, for the snapshot taken after it.
#[derive(Clone, Default)]
pub(super) struct Paced {
    /// The pace spell's name and what a cast of it cost.
    pub spell: Option<(String, i32)>,
    /// The caster style it was cast in, by name and id.
    pub style: Option<String>,
    pub style_id: Option<WeaponSkillId>,
    /// Casts a day from the pool.
    pub casts_a_day: f64,
}

/// The spell `w`'s player of `build` casts most, if the build casts: of its school's heals for a
/// healer, else of its school's damage spells, the one of the most mean roll a cast among those
/// costing at most a fifth of the pool (else the cheapest). Never one cast only under the open
/// sky (Call lightning): the fights are in a dungeon.
pub(super) fn pace_spell(w: &World, build: &Build) -> Option<Cast> {
    let content = &w.content;
    let table = content.weapon_skills();
    let school = table.id(build.school?);
    let pool = w.state.player().max_mana(content);
    let (cheap, dear): (Vec<_>, Vec<_>) = (casts(w).into_iter())
        .filter(|c| c.cost > 0 && c.heals == build.casts_a_kill)
        .filter(|c| {
            let def = content.spell(c.spell);
            !def.open_sky && table.of_school(def.school) == school
        })
        .partition(|c| c.cost * 5 <= pool);
    let mean = |c: &Cast| roll_mean(c.dice, c.reroll);
    match cheap.is_empty() {
        false => cheap.into_iter().max_by(|a, b| mean(a).total_cmp(&mean(b))),
        true => dear.into_iter().min_by_key(|c| c.cost),
    }
}

/// Casts the days that bring `w`'s player (of `build`, at level `level`) to the next level, each
/// day's pool on the pace spell, practising its school and caster style.
pub(super) fn cast_level(w: &mut World, build: &Build, level: u8, seed: u64) -> Paced {
    if build.school.is_none() {
        return Paced::default();
    }
    let content = w.content.clone();
    let mut dressed = w.clone();
    dress(&mut dressed, build, level, Gear::Plain, seed);
    let Some(spell) = pace_spell(&dressed, build) else {
        return Paced::default();
    };
    let def = content.spell(spell.spell);
    let me = dressed.state.player();
    let skills = me.spell_skills(&content, def, me.standing_aim(&content, def));
    let pool = me.max_mana(&content);
    let days = super::training::days(level + 1);
    let kills = content.experience().kills_for(level + 1);
    let mut cast = 0u32;
    for d in 0..days {
        let mut today = u32::try_from(pool / spell.cost.max(1)).unwrap_or(0);
        if build.casts_a_kill {
            today = today.min(kills * (d + 1) / days - kills * d / days);
        }
        for _ in 0..today {
            for skill in [skills.skill, skills.style].into_iter().flatten() {
                w.practise(skill, true, u32::from(level));
            }
        }
        cast += today;
    }
    Paced {
        spell: Some((spell.name.clone(), spell.cost)),
        style: skills
            .style
            .map(|id| content.weapon_skills().skill(id).name.clone()),
        style_id: skills.style,
        casts_a_day: f64::from(cast) / f64::from(days),
    }
}

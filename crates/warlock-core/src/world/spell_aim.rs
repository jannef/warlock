//! Where a spell is cast from (spec §8 "Fighting styles"; the spell scaling design §10.2): the
//! caster style a cast is made in, decided once for the whole cast before its kind runs.
//!
//! A spell aimed at a creature **beside** its caster is a Toucher's; one aimed **farther** (a
//! creature or a cell two or more away, or an empty cell, a ray or a burst alike) a Sniper's; any
//! spell cast in **armour that hinders casting** (`Actor::worn_spell_cost` above 0) a Battle
//! caster's, wherever it is aimed; a spell on oneself out of such armour is cast in **no style**.
//! The style's benefit joins the cast's hooks after the school's ([`Actor::spell_mods_from`]:
//! Sniper's `cells`, Toucher's `bonus`), its `to_hit` and `threat` a rolling spell's attack, and
//! every spell paid in mana trains it (`World::practise_cast`). Nothing is saved.

use super::{Actor, AttackSkills, World};
use crate::action::ReadTarget;
use crate::content::{Content, Grip, Shape, SpellData, SpellMods};

/// Where a spell is cast from (module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpellAim {
    /// In armour that hinders casting: Battle caster, wherever it is aimed.
    Armoured,
    /// At a creature beside the caster: Toucher.
    Beside,
    /// Farther than that, at a creature or a cell: Sniper.
    Afar,
    /// On oneself (or a stack one carries), out of such armour: no style.
    Myself,
}

impl SpellAim {
    /// The grip of the caster style it is cast in; `None` for none.
    pub const fn grip(self) -> Option<Grip> {
        match self {
            SpellAim::Armoured => Some(Grip::ArmouredSpell),
            SpellAim::Beside => Some(Grip::TouchSpell),
            SpellAim::Afar => Some(Grip::RangedSpell),
            SpellAim::Myself => None,
        }
    }
}

impl Actor {
    /// Where it would cast `spell` from as it stands, with no target yet (what the abilities
    /// menu, the dumps, and the aiming mode show): armoured in hindering armour; else a touch
    /// beside it, any other aimed spell afar, a spell on itself or a stack its own.
    pub fn standing_aim(&self, content: &Content, spell: &SpellData) -> SpellAim {
        if self.worn_spell_cost(content) > 0 {
            SpellAim::Armoured
        } else if spell.shape == Shape::Touch {
            SpellAim::Beside
        } else if spell.aimed() {
            SpellAim::Afar
        } else {
            SpellAim::Myself
        }
    }

    /// The skill and the caster style a cast of `spell` made from `aim` draws on and trains
    /// (spec §8 "Weapon skills": "Schools of magic", "Fighting styles"): its school's skill, and
    /// the style of where it is cast from (module docs), none on oneself out of armour.
    pub fn spell_skills(
        &self,
        content: &Content,
        spell: &SpellData,
        aim: SpellAim,
    ) -> AttackSkills {
        let table = content.weapon_skills();
        AttackSkills {
            skill: table.of_school(spell.school),
            style: aim.grip().and_then(|grip| table.style_for(grip, &[])),
        }
    }

    /// What its talents, worn items, school rank, and the caster style of `aim` do to `spell`
    /// ([`Actor::spell_mods`]'s sum, then the style's benefit: a style adds to the school as a
    /// weapon style adds to the weapon skill).
    pub fn spell_mods_from(
        &self,
        content: &Content,
        spell: &SpellData,
        aim: SpellAim,
    ) -> SpellMods {
        let mut mods = self.spell_mods(content, spell);
        let style = self.spell_skills(content, spell, aim).style;
        if style.is_some() {
            mods.add(&self.benefit(content, style).spell_mods());
        }
        mods
    }
}

impl World {
    /// Where the player casts a spell from at `target` (module docs): armoured in hindering
    /// armour; else beside at a cell next to them with a creature on it, afar at any other cell
    /// but their own, and on themselves for their own cell, a stack, or no target.
    pub(super) fn spell_aim(&self, target: Option<ReadTarget>) -> SpellAim {
        let me = self.state.player();
        if me.worn_spell_cost(&self.content) > 0 {
            return SpellAim::Armoured;
        }
        let here = self.player_pos();
        match target {
            Some(ReadTarget::Cell(at)) if at == here => SpellAim::Myself,
            Some(ReadTarget::Cell(at)) => {
                let level = &self.state.current().id;
                if here.chebyshev(at) == 1 && self.state.actor_at(level, at).is_some() {
                    SpellAim::Beside
                } else {
                    SpellAim::Afar
                }
            }
            _ => SpellAim::Myself,
        }
    }
}

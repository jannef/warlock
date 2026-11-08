//! The caster style a spell is cast in (spec §8 "Fighting styles"; #571, #1251): decided by the
//! spell and the caster's armour, never by where it is aimed.
//!
//! Any spell cast in **armour that hinders casting** (`Actor::worn_spell_cost` above 0) is a
//! Battle caster's. Out of such armour, a spell that rolls to hit ([`SpellData::rolls_to_hit`])
//! is a Toucher's when its shape is `touch` and a Sniper's otherwise (a ray, a bolt); every other
//! spell (an area, a bolt that never misses, a heal, a buff, a hex) is cast in **no style**. The
//! style's benefit joins the cast's hooks after the talents' and items' ([`Actor::spell_mods_in_style`]:
//! Sniper's `cells`, Toucher's `bonus`), its `to_hit` and `threat` a rolling spell's attack, and
//! every spell paid in mana trains it (`World::practise_cast`). The player's casts and the
//! creatures' are decided alike. Nothing is saved.

use super::{Actor, AttackSkills, CastBy};
use crate::content::{Content, Grip, Shape, SpellData, SpellMods};

impl Actor {
    /// The grip of the caster style its cast of `spell` is made in (module docs); `None` for
    /// none.
    pub fn spell_grip(&self, content: &Content, spell: &SpellData) -> Option<Grip> {
        if self.worn_spell_cost(content) > 0 {
            Some(Grip::ArmouredSpell)
        } else if !spell.rolls_to_hit() {
            None
        } else if spell.shape == Shape::Touch {
            Some(Grip::TouchSpell)
        } else {
            Some(Grip::RangedSpell)
        }
    }

    /// The skills a cast of `spell` draws on and trains (spec §8 "Fighting styles"): no weapon
    /// skill, only the caster style it is cast in (module docs).
    pub fn spell_skills(&self, content: &Content, spell: &SpellData) -> AttackSkills {
        let table = content.weapon_skills();
        AttackSkills {
            skill: None,
            style: (self.spell_grip(content, spell)).and_then(|grip| table.style_for(grip, &[])),
        }
    }

    /// What its talents, worn items, and the caster style it casts `spell` in do to it
    /// ([`Actor::spell_mods`]'s sum, then the style's benefit). What a cast settles with, and
    /// what the menu, the dumps, and the aiming mode show.
    pub fn spell_mods_in_style(&self, content: &Content, spell: &SpellData) -> SpellMods {
        self.spell_mods_in_style_by(content, spell, CastBy::Own)
    }

    /// [`Actor::spell_mods_in_style`] for a cast as `by` ([`Actor::spell_mods_by`]: a scroll's
    /// or a wand's cast has no rank).
    pub fn spell_mods_in_style_by(
        &self,
        content: &Content,
        spell: &SpellData,
        by: CastBy,
    ) -> SpellMods {
        let mut mods = self.spell_mods_by(content, spell, by);
        let style = self.spell_skills(content, spell).style;
        if style.is_some() {
            mods.add(&self.benefit(content, style).spell_mods());
        }
        mods
    }
}

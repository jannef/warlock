//! What a caster's talents and worn items do to one of its spells (spec §8 "Spells": "Hooks";
//! the spell scaling design §7.1): the six hooks' [`SpellMods`] (`content/spell_mod.rs`), summed
//! once a cast from two sources, each filtered by the spell's tags:
//!
//! - its **talents**' `spell_mod` effects ([`crate::talent::TalentMods::spell_mods`]: Empower,
//!   Archmage, Keen mind, School focus, Scroll adept, Wand mastery, Sure hand, Brimming,
//!   Numbing cold);
//! - its **worn items**' spell effects, unidentified too (#629): `spell_rungs`, `spell_bonus`
//!   (Potent), `spell_sides`, `spell_dice`, `spell_reroll`, `wand_rungs` (of Wands), and
//!   `spell_reach` (of Farcasting) as cells and `spell_duration` (of Lingering) as degrees.
//!   Items carry no tag filter yet: each acts on every spell.
//!
//! Derived at cast time; nothing is saved. A third source, a school's rank, joins here in a later
//! slice (S8). Nothing is clamped (decision 16).

use super::Actor;
use crate::content::{Content, Effect as ItemEffect, SpellData, SpellMods};

impl Actor {
    /// What its talents and worn items do to `spell` (module docs), summed.
    pub fn spell_mods(&self, content: &Content, spell: &SpellData) -> SpellMods {
        let tags = spell.tags(content);
        let mut mods = self.talent_mods(content).spell_mods(&tags);
        mods.add(&self.worn_spell_mods(content));
        mods
    }

    /// What its worn items do to every spell of its, as [`SpellMods`].
    fn worn_spell_mods(&self, content: &Content) -> SpellMods {
        let worn = self.mods(content);
        let count = |e: ItemEffect| u32::try_from(worn.get(e)).unwrap_or(0);
        SpellMods {
            rungs: worn.get(ItemEffect::SpellRungs).max(0),
            wand_rungs: worn.get(ItemEffect::WandRungs).max(0),
            sides: count(ItemEffect::SpellSides),
            dice: count(ItemEffect::SpellDice),
            bonus: worn.get(ItemEffect::SpellBonus),
            reroll: count(ItemEffect::SpellReroll),
            cells: worn.get(ItemEffect::SpellReach).max(0),
            degrees: worn.get(ItemEffect::SpellDuration).max(0),
            ..SpellMods::default()
        }
    }

    /// Where the rungs from outside its stats come from for `spell`'s roll, for the dev
    /// `progress` layer: `(name, rungs)` per talent held (by its name) and `worn` for its items,
    /// each above 0.
    pub fn spell_rung_sources(&self, content: &Content, spell: &SpellData) -> Vec<(String, i32)> {
        let tags = spell.tags(content);
        let book = content.talents();
        let mut sources: Vec<(String, i32)> = Vec::new();
        for taken in self.talents.iter() {
            let Some(id) = book.id(&taken.id) else {
                continue;
            };
            let def = book.get(id);
            let mut rungs = 0;
            for rank in 1..=taken.rank.min(def.rank_count()) {
                for effect in def.effects_at(rank) {
                    if let crate::content::TalentEffect::SpellMod(spec) = effect {
                        let one = spec.resolve(content, taken.choice.as_deref());
                        if one.fits(&tags) {
                            rungs += one.mods.rungs;
                        }
                    }
                }
            }
            if rungs > 0 {
                sources.push((def.name.clone(), rungs));
            }
        }
        let worn = self.worn_spell_mods(content).rungs;
        if worn > 0 {
            sources.push(("worn".into(), worn));
        }
        sources
    }
}

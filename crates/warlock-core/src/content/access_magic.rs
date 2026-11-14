//! Content's accessors for magic: spells and their rules, learning and miscasts, crystals and
//! enchanting, rods, and spell effects.

use super::{
    CircleStatuses, Content, Crystals, DamageTypeId, DecipherRules, EffectDef, EnchantingRules,
    Miscast, MiscastRules, Mishap, RodEffect, SpellData, SpellId, SpellRules, SpellType, StatusId,
    StudyRules,
};

impl Content {
    /// The spells' cost table (`data/spells.toml`, spec §8 "Spells"): none without the file.
    pub fn spell_rules(&self) -> Option<&SpellRules> {
        self.spell_rules.as_ref()
    }

    /// What every spell of damage type `kind` does beyond its damage (`[types]` in
    /// `data/spells.toml`): none for a type without an entry.
    pub fn spell_type(&self, kind: Option<DamageTypeId>) -> Option<&SpellType> {
        self.spell_rules.as_ref()?.types.get(&kind?)
    }

    /// The study check and its time (`data/study.toml`, spec §8 "Spells"): none without it,
    /// and then nothing can be studied.
    pub fn study_rules(&self) -> Option<&StudyRules> {
        self.study.as_ref()
    }

    /// The check of reading a scroll (`data/decipher.toml`, spec §8 "Scrolls"): none without
    /// it, and then every scroll simply reads.
    pub fn decipher(&self) -> Option<&DecipherRules> {
        self.decipher.as_ref()
    }

    /// The magic crystals' colours (`data/crystals.toml`): none without it, and then no crystal
    /// is caught or consumed.
    pub fn crystals(&self) -> Option<&Crystals> {
        self.crystals.as_ref()
    }

    /// Enchanting's numbers (`data/enchanting.toml`): none without it, and then salvage makes no
    /// affix checks.
    pub fn enchanting(&self) -> Option<&EnchantingRules> {
        self.enchanting.as_ref()
    }

    /// What a failed study may bring (`data/mishaps/`), in path order.
    pub fn mishaps(&self) -> &[Mishap] {
        &self.mishaps
    }

    /// The Spellcraft check every cast makes (`data/miscast.toml`): none without it, and then no
    /// cast miscasts.
    pub fn miscast_rules(&self) -> Option<&MiscastRules> {
        self.miscast.as_ref()
    }

    /// The miscast table (`data/miscasts/`), ordered by its totals.
    pub fn miscasts(&self) -> &[Miscast] {
        &self.miscasts
    }

    /// What rods do (`data/rod_effects/`), in path order: a [`Rod`]'s `effect` is an index.
    pub fn rod_effects(&self) -> &[RodEffect] {
        &self.rod_effects
    }

    /// Rod effect `index` (a [`Rod`]'s `effect`).
    pub fn rod_effect(&self, index: usize) -> &RodEffect {
        &self.rod_effects[index]
    }

    /// The index of rod effect `id`, if there is one.
    pub fn rod_effect_index(&self, id: &str) -> Option<usize> {
        self.rod_effects.iter().position(|e| e.id == id)
    }

    /// The spells (`data/spells/`, spec §8 "Spells"), in path order: a [`SpellId`] is an index.
    pub fn spells(&self) -> &[SpellData] {
        &self.spells
    }

    pub fn spell(&self, id: SpellId) -> &SpellData {
        &self.spells[usize::from(id.0)]
    }

    /// The statuses a hex lays (spec §8 "Status effects"): those of every `kind = "hex"` spell,
    /// in status order. An actor with one of them is "under a hex" (the witch's Cackle reads it,
    /// `world/witch.rs`).
    pub fn hex_statuses(&self) -> &[StatusId] {
        &self.hex_statuses
    }

    /// Every spell, lowest spell level first, each level in path order (the abilities menu's
    /// order).
    pub fn spells_by_level(&self) -> Vec<SpellId> {
        let mut ids: Vec<SpellId> = (0..self.spells.len())
            .filter_map(|i| u16::try_from(i).ok().map(SpellId))
            .collect();
        ids.sort_by_key(|&id| (self.spell(id).level, id));
        ids
    }

    pub fn spell_id(&self, name: &str) -> Option<SpellId> {
        let i = self.spells.iter().position(|s| s.id == name)?;
        Some(SpellId(u16::try_from(i).ok()?))
    }

    /// The spell effects of `data/effects.toml`, in file order (none without the file).
    /// Presentation only: no rule reads them.
    pub fn effects(&self) -> &[EffectDef] {
        &self.effects
    }

    /// The spell circles' statuses of `data/circles.toml` (none without the file). The renderer draws
    /// their circles; the one rule they make, a circled status's light, is in its `StatusDef::light`.
    pub fn circles(&self) -> &CircleStatuses {
        &self.circles
    }

    /// The spell effect with id `id`.
    pub fn effect(&self, id: &str) -> Option<&EffectDef> {
        self.effects.iter().find(|e| e.id == id)
    }
}

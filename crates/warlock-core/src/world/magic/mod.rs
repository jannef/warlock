//! Magic: mana, spells, scrolls, wands (docs/dev/mana.md, spells.md, caster-styles.md, scrolls.md).

mod backfire;
pub(in crate::world) mod cast;
mod cast_style;
mod caster;
mod concentration;
mod corpse_spell;
pub(in crate::world) mod decipher;
mod forget;
pub(in crate::world) mod learn;
mod malison;
pub(in crate::world) mod mana;
pub(in crate::world) mod miscast;
mod read;
mod restore;
mod rod;
mod ruin;
pub(in crate::world) mod scribe;
pub(in crate::world) mod spell;
mod spell_arc;
mod spell_mods;
mod spell_rank;
mod spell_rule;
pub(in crate::world) mod spell_rung;
mod spell_type;
pub(in crate::world) mod spell_words;
mod summon_spell;
mod zap;
pub(in crate::world) mod zone;
cfg_if! {
    if #[cfg(test)] {
        mod backfire_tests;
        pub(in crate::world) mod cast_tests;
        mod caster_style_tests;
        pub(in crate::world) mod caster_tests;
        mod circle_tests;
        mod concentration_tests;
        mod corpse_spell_tests;
        mod decipher_tests;
        mod forget_tests;
        mod identify_spell_tests;
        mod learn_tests;
        mod magic_tests;
        mod malison_tests;
        mod mana_tests;
        mod miscast_stat_tests;
        mod miscast_tests;
        mod necromancy_rank_tests;
        mod read_tests;
        mod rod_tests;
        mod ruin_tests;
        mod scribe_tests;
        mod scroll_label_tests;
        mod spell_arc_tests;
        mod spell_data_tests;
        mod spell_list_tests;
        mod spell_rank_tests;
        mod spell_rule_tests;
        mod spell_type_force_tests;
        pub(in crate::world) mod spell_type_tests;
        mod summon_spell_tests;
        mod transmutation_rank_tests;
        mod zap_tests;
        mod zone_tests;
    }
}

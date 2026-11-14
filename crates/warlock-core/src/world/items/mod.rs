//! Objects, equipment, and magic items (docs/dev/objects.md, equipment.md, magic-items.md).

mod body_slots;
mod carry;
mod equip;
pub(crate) mod gear_condition;
mod lamp;
mod loot;
mod material_drops;
mod siphon;
mod stock;
cfg_if! {
    if #[cfg(test)] {
        mod affix_draught_tests;
        mod affix_spell_tests;
        mod affix_stamina_tests;
        mod affix_utility_tests;
        mod body_slots_tests;
        mod equip_tests;
        mod gear_condition_tests;
        mod gear_tests;
        mod gem_tests;
        mod lamp_tests;
        mod material_draw_tests;
        mod material_drops_tests;
        mod object_tests;
        mod parts_tests;
        mod siphon_tests;
    }
}

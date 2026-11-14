//! Crafting, enchanting, and digging (docs/dev/crafting.md, enchanting.md, digging.md).

pub(in crate::world) mod craft;
pub(in crate::world) mod craft_make;
mod crystal;
pub(in crate::world) mod cut;
pub(in crate::world) mod dig;
pub(in crate::world) mod enchant;
pub(in crate::world) mod field_forge;
mod fletch;
pub(in crate::world) mod make_do;
mod mastery;
mod mine;
pub(crate) mod patron;
pub(in crate::world) mod rig;
pub(in crate::world) mod salvage;
pub(in crate::world) mod seal;
pub(in crate::world) mod set;
pub(in crate::world) mod temper;
mod trade;
pub(in crate::world) mod work;
mod work_on;
cfg_if! {
    if #[cfg(test)] {
        mod brew_tests;
        mod craft_armour_tests;
        mod craft_bow_tests;
        mod craft_hand_tests;
        mod craft_hide_tests;
        mod craft_make_tests;
        mod craft_tests;
        mod craftsperson_tests;
        mod cut_tests;
        mod dig_tests;
        mod enchant_tests;
        mod field_forge_tests;
        mod fletch_tests;
        mod make_do_tests;
        mod makers_mark_tests;
        mod mastery_tests;
        mod mine_tests;
        mod patron_tests;
        mod rig_tests;
        mod salvage_tests;
        mod set_tests;
        mod temper_tests;
        mod trade_tests;
        mod whittle_tests;
    }
}

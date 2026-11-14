//! Travel, levels, and the overworld (docs/dev/overworld.md, encounters.md, mapgen.md,
//! temperature.md).

mod edge_leave;
pub(in crate::world) mod encounter;
mod encounter_area;
pub(in crate::world) mod endless;
pub(in crate::world) mod enter;
mod footing;
pub(in crate::world) mod ice;
mod layer_boss;
pub(in crate::world) mod pit;
mod quick_leave;
mod reveal;
mod stair_followers;
pub(in crate::world) mod swim;
pub(in crate::world) mod talk_past;
pub(in crate::world) mod travel;
mod wriggle;
cfg_if! {
    if #[cfg(test)] {
        mod beside_tests;
        mod branch_tests;
        mod edge_leave_tests;
        mod encounter_tests;
        mod endless_tests;
        mod escape_tests;
        mod everlasting_tests;
        mod footing_tests;
        mod gate_tests;
        mod ice_tests;
        mod layer_boss_tests;
        mod overworld_tests;
        mod pit_tests;
        mod quick_leave_tests;
        mod reveal_tests;
        mod room_temperature_tests;
        mod stair_followers_tests;
        mod swim_deep_tests;
        mod swim_tests;
        mod way_out_tests;
        mod wriggle_tests;
    }
}

//! Doors, chests, traps, and special rooms (docs/dev/doors.md, chests.md, traps.md,
//! special-rooms.md).

mod boons;
mod chest;
mod collapse;
pub(in crate::world) mod curse;
mod door;
pub(in crate::world) mod drink;
pub(in crate::world) mod feature;
pub(in crate::world) mod graves;
pub(in crate::world) mod graveyard;
mod haunt;
mod hoard;
mod knock;
pub(crate) mod mirrors;
pub(in crate::world) mod outcome;
pub(in crate::world) mod pray;
pub(in crate::world) mod rooms;
pub(in crate::world) mod shrine;
mod trap;
pub(in crate::world) mod treasury;
mod warp;
cfg_if! {
    if #[cfg(test)] {
        mod chest_hall_tests;
        mod chest_tests;
        mod chest_trap_tests;
        mod collapse_tests;
        mod curse_tests;
        mod door_tests;
        mod door_trap_tests;
        mod feature_tests;
        mod graveyard_tests;
        mod haunt_tests;
        mod heavy_hall_tests;
        mod hoard_tests;
        mod mirrors_tests;
        mod outcome_tests;
        mod room_tests;
        mod shrine_tests;
        mod spring_tests;
        mod trap_rung_tests;
        mod trap_tests;
        mod treasury_tests;
        mod unstable_room_tests;
    }
}

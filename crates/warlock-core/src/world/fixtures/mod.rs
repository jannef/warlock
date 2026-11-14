//! Doors, chests, traps, and special rooms (docs/dev/doors.md, chests.md, traps.md,
//! special-rooms.md).

mod boons;
mod chest;
mod collapse;
mod curse;
mod door;
mod drink;
mod feature;
mod graves;
mod graveyard;
mod haunt;
mod hoard;
mod knock;
pub(crate) mod mirrors;
mod outcome;
mod pray;
mod rooms;
mod shrine;
mod trap;
mod treasury;
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

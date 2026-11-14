//! The run, the turn, and time (docs/dev/time.md, saves.md, rendering.md's perception).

pub(in crate::world) mod actor;
pub(in crate::world) mod actors;
mod dev;
mod eyes;
mod glare;
pub(in crate::world) mod hash;
mod hearing;
pub(in crate::world) mod level;
mod perceive;
pub(in crate::world) mod rest;
mod run;
mod schedule;
pub(in crate::world) mod stride;
mod voice;
cfg_if! {
    if #[cfg(test)] {
        mod cost_tests;
        mod hearing_tests;
        pub(in crate::world) mod light_tests;
        mod moon_tests;
        mod numbers_tests;
        mod rest_close_tests;
        mod run_tests;
        mod speed_tests;
        mod stamina_tests;
        mod stride_tests;
        mod sun_tests;
        mod tests;
        mod time_tests;
        mod voice_tests;
        mod weather_tests;
    }
}

//! The run, the turn, and time (docs/dev/time.md, saves.md, rendering.md's perception).

mod actor;
mod actors;
mod dev;
mod eyes;
mod glare;
mod hash;
mod hearing;
mod level;
mod perceive;
mod rest;
mod run;
mod schedule;
mod stride;
mod voice;
cfg_if! {
    if #[cfg(test)] {
        mod cost_tests;
        mod hearing_tests;
        mod light_tests;
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

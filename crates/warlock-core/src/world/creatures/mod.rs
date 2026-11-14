//! Creatures (docs/dev/creatures.md, summoning.md, animated.md).

pub(in crate::world) mod ai;
mod animated;
pub(in crate::world) mod aura;
mod avert;
mod blink;
mod breath;
pub(in crate::world) mod court;
mod creature_burst;
mod creature_cast;
mod den;
mod first_sight;
mod gaze;
mod incorporeal;
pub(in crate::world) mod mimic;
mod path;
mod rooted;
mod snatch;
pub(in crate::world) mod spawn;
pub(in crate::world) mod split;
pub(in crate::world) mod statue;
mod summon;
pub(in crate::world) mod unseen;
cfg_if! {
    if #[cfg(test)] {
        mod ai_tests;
        mod animated_blade_tests;
        mod animated_dread_tests;
        mod animated_panoply_tests;
        mod animated_tests;
        mod aura_tests;
        mod beast_den_tests;
        mod bestiary_tests;
        mod breath_tests;
        mod court_tests;
        mod creature_burst_tests;
        mod creature_cast_tests;
        mod creature_flask_tests;
        mod creature_magic_tests;
        mod den_tests;
        mod dwell_tests;
        mod first_sight_tests;
        pub(in crate::world) mod fixture_factions;
        mod gaze_tests;
        mod incorporeal_tests;
        mod mimic_tests;
        mod retribution_tests;
        mod rooted_tests;
        mod spawn_tests;
        mod split_tests;
        mod statue_tests;
        mod summon_tests;
        mod tag_tests;
        mod unique_tests;
        mod unseen_tests;
        mod variant_tests;
    }
}

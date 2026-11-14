//! Classes and their abilities (docs/dev/progression.md, openings.md, prayer.md).

mod ascetic;
pub(in crate::world) mod calling;
mod charge;
pub(in crate::world) mod class;
mod class_ability;
mod consecrate;
mod dark_face;
pub(in crate::world) mod focus;
pub(in crate::world) mod invest;
mod jack;
pub(in crate::world) mod mark;
mod merchants_eye;
mod openings;
pub(in crate::world) mod orison;
mod rage;
pub(in crate::world) mod sanctuary;
mod smite;
mod song;
pub(in crate::world) mod spirit_weapon;
pub(in crate::world) mod stipend;
mod thief;
mod volley;
mod war_cry;
mod witch;
cfg_if! {
    if #[cfg(test)] {
        mod ascetic_tests;
        mod calling_tests;
        mod charge_tests;
        pub(in crate::world) mod class_active_tests;
        mod class_rogue_tests;
        mod class_tests;
        mod dark_face_tests;
        mod focus_tests;
        mod invest_tests;
        mod ki_tests;
        mod lullaby_tests;
        mod mark_tests;
        mod merchants_eye_tests;
        mod openings_tests;
        mod orison_tests;
        mod sanctuary_tests;
        mod smite_mode_tests;
        mod smite_tests;
        mod smite_wrath_tests;
        mod song_tests;
        mod spirit_weapon_tests;
        mod stipend_tests;
        pub(in crate::world) mod war_cry_tests;
        mod witch_tests;
    }
}

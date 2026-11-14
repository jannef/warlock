//! Classes and their abilities (docs/dev/progression.md, openings.md, prayer.md).

mod ascetic;
mod calling;
mod charge;
mod class;
mod class_ability;
mod consecrate;
mod dark_face;
mod focus;
mod invest;
mod jack;
mod mark;
mod merchants_eye;
mod openings;
mod orison;
mod rage;
mod sanctuary;
mod smite;
mod song;
mod spirit_weapon;
mod stipend;
mod thief;
mod volley;
mod war_cry;
mod witch;
cfg_if! {
    if #[cfg(test)] {
        mod ascetic_tests;
        mod calling_tests;
        mod charge_tests;
        mod class_active_tests;
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
        mod war_cry_tests;
        mod witch_tests;
    }
}

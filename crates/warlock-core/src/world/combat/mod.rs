//! Combat (docs/dev/combat.md, ranged-combat.md, weapon-skills.md, weapon-abilities.md, blood.md,
//! surprise.md).

mod ambush;
pub(in crate::world) mod attacks;
mod bleed;
mod corrode;
mod dummy;
pub(in crate::world) mod fight;
pub(in crate::world) mod free_blow;
mod frenzy;
pub(in crate::world) mod hurl;
pub(in crate::world) mod item_damage;
mod knock_away;
mod lurk;
mod natural;
pub(in crate::world) mod reflect;
mod sear;
pub(in crate::world) mod shoot;
pub(in crate::world) mod slip;
pub(crate) mod surprise;
mod two_shields;
pub(in crate::world) mod weapon_ability;
pub(in crate::world) mod weapon_active;
cfg_if! {
    if #[cfg(test)] {
        mod ambush_tests;
        mod attacks_tests;
        mod balance_probe;
        mod blood_tests;
        mod corrode_tests;
        mod damage_tests;
        mod dummy_tests;
        mod fight_tests;
        mod free_blow_tests;
        mod frenzy_tests;
        mod hurl_tests;
        mod item_damage_tests;
        mod knock_away_tests;
        mod lurk_tests;
        mod protection_tests;
        mod sear_tests;
        mod shoot_tests;
        mod shot_range_tests;
        mod silent_tests;
        mod slip_tests;
        mod stunning_strike_tests;
        mod style_tests;
        mod surprise_tests;
        mod surprise_words_tests;
        mod thrown_exploit_tests;
        mod two_shield_tests;
        pub(in crate::world) mod vitals_tests;
        mod weapon_ability_tests;
        mod weapon_active_tests;
        mod weapon_skill_tests;
    }
}

//! Combat (docs/dev/combat.md, ranged-combat.md, weapon-skills.md, weapon-abilities.md, blood.md,
//! surprise.md).

mod ambush;
mod attacks;
mod bleed;
mod corrode;
mod dummy;
mod fight;
mod free_blow;
mod frenzy;
mod hurl;
mod item_damage;
mod knock_away;
mod lurk;
mod natural;
mod reflect;
mod sear;
mod shoot;
mod slip;
pub(crate) mod surprise;
mod two_shields;
mod weapon_ability;
mod weapon_active;
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
        mod vitals_tests;
        mod weapon_ability_tests;
        mod weapon_active_tests;
        mod weapon_skill_tests;
    }
}

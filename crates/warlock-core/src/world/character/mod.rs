//! The character: creation, progression, stats (docs/dev/character-creation.md, background.md,
//! progression.md, experience.md, stats.md, skills.md, alignment.md, resources.md).

pub(in crate::world) mod alignment;
mod background;
pub(in crate::world) mod creation;
mod element_gifts;
pub(in crate::world) mod experience;
mod flavour;
pub(in crate::world) mod gifts;
mod heirloom;
mod lucky;
mod profane;
mod racial;
pub(in crate::world) mod resource;
mod second_wind;
mod sign_picks;
pub(in crate::world) mod skill_roll;
pub(in crate::world) mod stars;
pub(in crate::world) mod stat_deeds;
pub(in crate::world) mod stat_training;
pub(in crate::world) mod talent;
mod train;
cfg_if! {
    if #[cfg(test)] {
        mod alignment_tests;
        mod body_talent_tests;
        mod character_tests;
        mod creation_tests;
        mod damage_talent_tests;
        mod element_gift_tests;
        mod experience_tests;
        mod flavour_tests;
        mod gifts_tests;
        mod heirloom_tests;
        mod land_talent_tests;
        mod lucky_tests;
        mod mana_talent_tests;
        mod race_tests;
        mod racial_tests;
        mod resource_talent_tests;
        mod resource_tests;
        mod sign_pick_tests;
        mod skill_roll_tests;
        mod skills_tests;
        mod spell_talent_tests;
        mod stars_roc_tests;
        mod stars_tests;
        mod stat_deeds_tests;
        mod stat_paths_tests;
        mod stat_training_tests;
        mod stats_tests;
        mod talent_arms_tests;
        mod talent_tests;
        mod talent_wits_tests;
        mod waste_not_tests;
    }
}

//! The character: creation, progression, stats (docs/dev/character-creation.md, background.md,
//! progression.md, experience.md, stats.md, skills.md, alignment.md, resources.md).

mod alignment;
mod background;
mod creation;
mod element_gifts;
mod experience;
mod flavour;
mod gifts;
mod heirloom;
mod lucky;
mod profane;
mod racial;
mod resource;
mod second_wind;
mod sign_picks;
mod skill_roll;
mod stars;
mod stat_deeds;
mod stat_training;
mod talent;
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

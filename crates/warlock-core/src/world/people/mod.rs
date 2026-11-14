//! People: talk, quests, shops, guilds (docs/dev/npcs.md, quests.md, shops.md, guild.md).

pub(in crate::world) mod escort;
pub(in crate::world) mod escort_waits;
mod goodwill;
pub(in crate::world) mod guild;
mod lure;
pub(in crate::world) mod quest;
mod quest_roll;
pub(in crate::world) mod rumour;
mod service;
pub(in crate::world) mod shop;
pub(in crate::world) mod talk;
pub(in crate::world) mod wares;
cfg_if! {
    if #[cfg(test)] {
        mod escort_tests;
        mod goodwill_tests;
        mod guild_tests;
        mod guild_wanted_tests;
        mod identify_service_tests;
        mod outlaw_tests;
        mod quest_roll_tests;
        mod quest_tests;
        mod rumour_tests;
        mod service_haggle_tests;
        mod service_tests;
        mod shop_tests;
        mod talk_tests;
        mod train_service_tests;
        mod wares_tests;
    }
}

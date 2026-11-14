//! Statuses, healing, hunger, and potions (docs/dev/status-effects.md, hunger.md, potions.md,
//! burden.md).

mod buff;
mod disease;
mod drain;
mod eat;
mod elixir;
mod flask;
pub(crate) mod freshness;
mod heal;
mod heal_harm;
mod hold;
mod hunger;
mod potion;
mod regen;
mod rot;
mod status;
cfg_if! {
    if #[cfg(test)] {
        mod buff_field_tests;
        mod buff_tests;
        mod burden_tests;
        mod disease_tests;
        mod drain_tests;
        mod eat_tests;
        mod elixir_tests;
        mod flask_content_tests;
        mod flask_fire_tests;
        mod flask_tests;
        mod freshness_tests;
        mod heal_harm_tests;
        mod heal_tests;
        mod hold_tests;
        mod hunger_tests;
        mod potion_gamble_tests;
        mod potion_status_tests;
        mod potion_tests;
        mod potion_utility_tests;
        mod rot_tests;
        mod smoke_tests;
        mod status_tests;
    }
}

//! Skill uses (docs/dev/skills.md, searching.md, stealing.md, hearing.md).

mod appraise;
mod disarm;
mod feint;
pub(in crate::world) mod forage;
pub(in crate::world) mod gather;
mod intimidate;
mod knowledge;
mod listen;
mod lockpick;
pub(in crate::world) mod search;
mod sidestep;
mod steal;
mod wedge;
cfg_if! {
    if #[cfg(test)] {
        mod appraise_tests;
        mod disarm_tests;
        mod feint_tests;
        mod forage_tests;
        mod gather_tests;
        pub(in crate::world) mod intimidate_tests;
        mod knowledge_tests;
        mod lockpick_tests;
        mod search_tests;
        mod sidestep_tests;
        mod steal_tests;
        mod theft_tests;
        mod wedge_tests;
    }
}

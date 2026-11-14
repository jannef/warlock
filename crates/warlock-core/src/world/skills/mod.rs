//! Skill uses (docs/dev/skills.md, searching.md, stealing.md, hearing.md).

mod appraise;
mod disarm;
mod feint;
mod forage;
mod gather;
mod intimidate;
mod knowledge;
mod listen;
mod lockpick;
mod search;
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
        mod intimidate_tests;
        mod knowledge_tests;
        mod lockpick_tests;
        mod search_tests;
        mod sidestep_tests;
        mod steal_tests;
        mod theft_tests;
        mod wedge_tests;
    }
}

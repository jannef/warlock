//! Allies (docs/dev/allies.md, companion.md, familiar.md).

mod ally;
mod beast_friend;
mod bond;
mod charm;
mod collar;
mod companion;
mod familiar;
mod pack;
mod sic;
mod tame;
cfg_if! {
    if #[cfg(test)] {
        mod ally_tests;
        mod bond_tests;
        mod charm_tests;
        mod collar_tests;
        mod pack_tests;
        mod sic_tests;
        mod tame_tests;
    }
}

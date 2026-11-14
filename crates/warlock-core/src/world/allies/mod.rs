//! Allies (docs/dev/allies.md, companion.md, familiar.md).

pub(in crate::world) mod ally;
pub(in crate::world) mod beast_friend;
mod bond;
mod charm;
pub(in crate::world) mod collar;
pub(in crate::world) mod companion;
pub(in crate::world) mod familiar;
mod pack;
mod sic;
pub(in crate::world) mod tame;
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

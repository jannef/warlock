//! Platform directories for the game's files (saves, logs). One place, so they always agree.

/// The game's platform directories, or `None` where there is no filesystem (the web build) or
/// no home directory.
#[cfg(not(target_arch = "wasm32"))]
pub fn project_dirs() -> Option<directories::ProjectDirs> {
    directories::ProjectDirs::from("", "", "warlock")
}

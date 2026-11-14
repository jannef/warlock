//! The content under `assets/` for the binaries (#1587): `build.rs` embeds it here, and
//! warlock-core takes it as input, [`ContentFiles`], so a data edit rebuilds this crate and the
//! binaries above it, never the core. The game, the CLI, and xtask load the built-in copy with
//! [`files`] (or [`content`]) and a content directory (`--content-dir`) with [`from_dir`]; a set
//! of extra files (mods, later: #196) would be laid over either with [`ContentFiles::insert`].

use std::path::Path;
use warlock_core::content::{Content, ContentError, ContentFiles};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/builtin_assets.rs"));
}

/// Whether this build holds the test world, `campaigns/showcase/` (the `showcase` feature: never
/// in a release; docs/dev/campaign.md).
pub const SHOWCASE: bool = cfg!(feature = "showcase");

/// The content compiled into the binary from `assets/`.
pub fn files() -> ContentFiles {
    let mut files = ContentFiles::default();
    for (path, text) in embedded::FILES {
        files.insert(path, text);
    }
    files
}

/// Loads the built-in content.
pub fn content() -> Result<Content, ContentError> {
    Content::load(&files())
}

/// Reads the content under `root` (`--content-dir`): the same set, under the same paths, as
/// [`files`] embeds from `assets/`, the test world left out without the `showcase` feature (so
/// `--content-dir assets` on a release build has none either).
pub fn from_dir(root: &Path) -> std::io::Result<ContentFiles> {
    ContentFiles::from_dir(root, SHOWCASE)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The embedded set is exactly what is on disk under `assets/` (with the `showcase` feature
    /// or without): the same paths, each with the same text.
    #[test]
    fn the_embedded_copy_is_the_assets_directory() {
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let on_disk = from_dir(&assets).expect("read assets/");
        let embedded: Vec<&str> = embedded::FILES.iter().map(|(p, _)| *p).collect();
        assert_eq!(on_disk.paths().collect::<Vec<_>>(), embedded);
        for (path, text) in embedded::FILES {
            let disk = std::fs::read_to_string(assets.join(path)).expect("read");
            assert_eq!(disk, *text, "{path}");
        }
    }

    #[test]
    fn the_built_in_content_loads() {
        content().expect("the built-in content loads");
    }
}

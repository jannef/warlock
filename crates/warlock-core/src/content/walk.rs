//! The one walk of a content directory (#1560): `build.rs` embeds what it finds under `assets/`
//! (it includes this file with `#[path]`, so it uses `std` alone), and
//! [`super::ContentFiles::from_dir`] reads the same set from any directory at run time.

use std::path::{Path, PathBuf};

/// The directories under a content root that hold content.
pub const CONTENT_DIRS: [&str; 3] = ["campaigns", "data", "vaults"];

/// The test world's directory, as content paths start (docs/dev/campaign.md, "The test world in
/// release builds").
pub const SHOWCASE: &str = "campaigns/showcase/";

/// Every `.toml` file under `root`'s [`CONTENT_DIRS`], subdirectories included, as (path relative
/// to `root` with `/` separators, path on disk), sorted by the relative path. Without `showcase`
/// the test world's files are left out. `visit` is told of every directory walked (the root's
/// content directories and all below them); a content directory that does not exist is skipped.
pub fn walk(
    root: &Path,
    showcase: bool,
    visit: &mut dyn FnMut(&Path),
) -> std::io::Result<Vec<(String, PathBuf)>> {
    let mut files = Vec::new();
    for dir in CONTENT_DIRS {
        let path = root.join(dir);
        if path.is_dir() {
            collect(&path, dir, visit, &mut files)?;
        }
    }
    if !showcase {
        files.retain(|(rel, _)| !rel.starts_with(SHOWCASE));
    }
    files.sort();
    Ok(files)
}

fn collect(
    dir: &Path,
    rel: &str,
    visit: &mut dyn FnMut(&Path),
    files: &mut Vec<(String, PathBuf)>,
) -> std::io::Result<()> {
    visit(dir);
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let rel = format!("{rel}/{name}");
        if path.is_dir() {
            collect(&path, &rel, visit, files)?;
        } else if path.extension().is_some_and(|e| e == "toml") {
            files.push((rel, path));
        }
    }
    Ok(())
}

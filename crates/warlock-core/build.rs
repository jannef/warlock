//! Embeds the TOML content under `assets/{campaigns,data,levels,scenarios,vaults}` (subdirectories
//! included: `data/creatures/kobold.toml`, `campaigns/showcase/levels/hub.toml`) into the binary, so every build (including the web build) is
//! self-contained. The list is in sorted path order, the order `ContentFiles::from_dir` reads
//! a content directory in.
//!
//! The test world, `campaigns/showcase/`, is embedded only with the `showcase` feature (on by
//! default, off in release builds and the web build): without it a build embeds no file of
//! it (docs/dev/campaign.md, "The test world in release builds").

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let assets = manifest.join("../../assets");
    let mut files = Vec::new();
    // A path that does not exist is always stale to cargo (every build reruns this script and
    // rebuilds the crate), so only existing directories are named; the assets directory itself
    // is watched to notice one appearing.
    println!("cargo:rerun-if-changed={}", assets.display());
    let showcase = std::env::var_os("CARGO_FEATURE_SHOWCASE").is_some();
    for dir in ["campaigns", "data", "levels", "scenarios", "vaults"] {
        let path = assets.join(dir);
        if !path.exists() {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());
        collect(&path, dir, &mut files);
    }
    if !showcase {
        files.retain(|(rel, _)| !rel.starts_with(SHOWCASE));
    }
    files.sort();

    let mut out = String::from("pub(super) const FILES: &[(&str, &str)] = &[\n");
    for (rel, abs) in &files {
        println!("cargo:rerun-if-changed={}", abs.display());
        writeln!(
            out,
            "    ({rel:?}, include_str!({:?})),",
            abs.display().to_string()
        )
        .expect("writing to a String cannot fail");
    }
    out.push_str("];\n");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("set by cargo"));
    std::fs::write(out_dir.join("builtin_assets.rs"), out).expect("write builtin_assets.rs");
}

/// The test world's directory, as embedded paths start.
const SHOWCASE: &str = "campaigns/showcase/";

/// Every `.toml` file under `dir`, subdirectories included, as (`rel/name`, absolute path).
fn collect(dir: &Path, rel: &str, files: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if path.is_dir() {
            println!("cargo:rerun-if-changed={}", path.display());
            collect(&path, &format!("{rel}/{name}"), files);
        } else if path.extension().is_some_and(|e| e == "toml") {
            let abs = path.canonicalize().unwrap_or(path.clone());
            files.push((format!("{rel}/{name}"), abs));
        }
    }
}

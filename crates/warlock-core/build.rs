//! Embeds the TOML content under `assets/{campaigns,data,vaults}` (subdirectories included:
//! `data/creatures/kobold.toml`, `campaigns/showcase/levels/hub.toml`) into the binary, so every
//! build (including the web build) is self-contained. The walk is `src/content/walk.rs`, the one
//! `ContentFiles::from_dir` takes too, so the list is in sorted path order, the order it reads a
//! content directory in.
//!
//! The test world, `campaigns/showcase/`, is embedded only with the `showcase` feature (on by
//! default, off in release builds and the web build): without it a build embeds no file of
//! it (docs/dev/campaign.md, "The test world in release builds").

#[path = "src/content/walk.rs"]
#[allow(dead_code)]
mod walk;

use std::fmt::Write as _;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let assets = manifest.join("../../assets");
    // A path that does not exist is always stale to cargo (every build reruns this script and
    // rebuilds the crate), so only existing directories are named; the assets directory itself
    // is watched to notice one appearing.
    println!("cargo:rerun-if-changed={}", assets.display());
    let showcase = std::env::var_os("CARGO_FEATURE_SHOWCASE").is_some();
    let mut watch = |dir: &std::path::Path| println!("cargo:rerun-if-changed={}", dir.display());
    let files = walk::walk(&assets, showcase, &mut watch).expect("read assets/");

    let mut out = String::from("pub(super) const FILES: &[(&str, &str)] = &[\n");
    for (rel, path) in &files {
        let abs = path.canonicalize().unwrap_or(path.clone());
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

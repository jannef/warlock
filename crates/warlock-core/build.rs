//! Embeds the TOML content under `assets/{data,levels,scenarios,vaults}` into the binary, so every
//! build (including the web build) is self-contained.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let assets = manifest.join("../../assets");
    let mut files = Vec::new();
    for dir in ["data", "levels", "scenarios", "vaults"] {
        let path = assets.join(dir);
        println!("cargo:rerun-if-changed={}", path.display());
        collect(&path, dir, &mut files);
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

fn collect(dir: &Path, rel: &str, files: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "toml") {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let abs = path.canonicalize().unwrap_or(path.clone());
            files.push((format!("{rel}/{name}"), abs));
        }
    }
}

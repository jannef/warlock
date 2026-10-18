//! `cargo xtask licenses [--check]`: regenerates `THIRD_PARTY_LICENSES.html` with `cargo-about`
//! (config `about.toml`, template `about.hbs`), or checks that the committed file is current.

use std::path::PathBuf;
use std::process::Command;

use crate::workspace_root;

/// The committed notices file, at the repository root.
pub(crate) const FILE: &str = "THIRD_PARTY_LICENSES.html";

pub(crate) fn run(check: bool) -> Result<(), String> {
    let root = workspace_root();
    let probe = Command::new(env!("CARGO"))
        .args(["about", "--version"])
        .output();
    if !probe.is_ok_and(|o| o.status.success()) {
        return Err("cargo-about is not installed. Install it with:\n  \
                    cargo install cargo-about --locked --features cli"
            .into());
    }

    let tmp: PathBuf = root.join("target/xtask/licenses.html");
    std::fs::create_dir_all(tmp.parent().ok_or("no parent directory")?)
        .map_err(|e| format!("cannot create target/xtask: {e}"))?;
    // `--workspace` scans every member, including xtask; its dependencies (image, serde, toml,
    // warlock-core) are a subset of the game's, so the shipped set is unchanged. If xtask ever
    // gains a dependency the game lacks, it would be listed here too: keep that in mind.
    let status = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args(["about", "generate", "--workspace", "--locked", "--fail"])
        .arg("--config")
        .arg("about.toml")
        .arg("--output-file")
        .arg(&tmp)
        .arg("about.hbs")
        .status()
        .map_err(|e| format!("failed to run cargo about: {e}"))?;
    if !status.success() {
        return Err(format!("cargo about failed: {status}"));
    }
    // Some license files use CRLF; normalize so the output does not depend on the source.
    let fresh = std::fs::read_to_string(&tmp)
        .map_err(|e| format!("cannot read {}: {e}", tmp.display()))?
        .replace("\r\n", "\n");

    let target = root.join(FILE);
    if check {
        let committed = std::fs::read_to_string(&target).unwrap_or_default();
        if committed != fresh {
            return Err(format!(
                "{FILE} is stale; run `cargo xtask licenses` and commit the result"
            ));
        }
        println!("{FILE} is up to date");
    } else {
        std::fs::write(&target, &fresh)
            .map_err(|e| format!("cannot write {}: {e}", target.display()))?;
        println!("wrote {FILE} ({} bytes)", fresh.len());
    }
    Ok(())
}

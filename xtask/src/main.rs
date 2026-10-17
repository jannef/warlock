//! `cargo xtask <command>` — developer automation.
//!
//! Commands:
//! - `smoke`: run the game with `--capture` and check that something was rendered.
//! - `web`: build the WebGPU site into `target/web` (needs `wasm-bindgen-cli`, see CLAUDE.md).
//! - `gen-vast [--check]`: regenerate `assets/levels/vast.toml` (`--check` fails if it is stale).
//! - `bench-turns`: release-mode turn-time table for the hub and the vast level.

mod bench;
mod vast;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("smoke") => smoke(),
        Some("web") => web(),
        Some("gen-vast") => gen_vast(args.get(1).is_some_and(|a| a == "--check")),
        Some("bench-turns") => bench_turns(),
        _ => {
            eprintln!("usage: cargo xtask <smoke | web | gen-vast [--check] | bench-turns>");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("xtask: {err}");
            ExitCode::FAILURE
        }
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace root")
        .to_path_buf()
}

/// Runs the game binary with the given arguments (debug profile).
fn run_game(extra: &[&str]) -> Result<(), String> {
    let status = Command::new(env!("CARGO"))
        .current_dir(workspace_root())
        .args(["run", "--quiet", "--package", "warlock", "--"])
        .args(extra)
        .status()
        .map_err(|e| format!("failed to run cargo: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("game exited with {status}"))
    }
}

fn smoke() -> Result<(), String> {
    let out = workspace_root().join("target/xtask/smoke.png");
    let _ = std::fs::remove_file(&out);
    let out_str = out.to_str().ok_or("non-UTF-8 path")?;
    run_game(&["--window", "640x360", "--scale", "1", "--capture", out_str])?;
    let img = image::open(&out)
        .map_err(|e| format!("cannot open {}: {e}", out.display()))?
        .to_rgb8();
    let first = *img.get_pixel(0, 0);
    if img.pixels().all(|p| *p == first) {
        return Err("smoke capture is a single flat color; nothing was rendered".into());
    }
    println!(
        "smoke ok: {} ({}x{})",
        out.display(),
        img.width(),
        img.height()
    );
    Ok(())
}

/// The `wasm-bindgen` crate version pinned in `Cargo.lock`; the CLI must match it exactly.
fn locked_wasm_bindgen_version() -> Result<String, String> {
    let lock = std::fs::read_to_string(workspace_root().join("Cargo.lock"))
        .map_err(|e| format!("cannot read Cargo.lock: {e}"))?;
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line == "name = \"wasm-bindgen\"" {
            let version = lines.next().unwrap_or_default();
            return version
                .strip_prefix("version = \"")
                .and_then(|v| v.strip_suffix('"'))
                .map(str::to_owned)
                .ok_or_else(|| "malformed wasm-bindgen entry in Cargo.lock".into());
        }
    }
    Err("wasm-bindgen is not in Cargo.lock".into())
}

/// Builds the browser version into `target/web` (serve that directory over HTTP).
fn web() -> Result<(), String> {
    let root = workspace_root();
    let version = locked_wasm_bindgen_version()?;
    let installed = Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    if installed.split_whitespace().nth(1) != Some(version.as_str()) {
        return Err(format!(
            "wasm-bindgen-cli {version} is required (found: {}). Install it with:\n  \
             cargo install wasm-bindgen-cli --version {version} --locked",
            if installed.is_empty() {
                "not installed"
            } else {
                installed.trim()
            },
        ));
    }

    let status = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args([
            "build",
            "--release",
            "--locked",
            "--package",
            "warlock",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .status()
        .map_err(|e| format!("failed to run cargo: {e}"))?;
    if !status.success() {
        return Err(format!("wasm build failed: {status}"));
    }

    let out = root.join("target/web");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| format!("cannot create {}: {e}", out.display()))?;
    let status = Command::new("wasm-bindgen")
        .current_dir(&root)
        .args([
            "--target",
            "web",
            "--no-typescript",
            "--remove-name-section",
            "--remove-producers-section",
            "--out-name",
            "warlock",
            "--out-dir",
        ])
        .arg(&out)
        .arg("target/wasm32-unknown-unknown/release/warlock.wasm")
        .status()
        .map_err(|e| format!("failed to run wasm-bindgen: {e}"))?;
    if !status.success() {
        return Err(format!("wasm-bindgen failed: {status}"));
    }
    std::fs::copy(root.join("web/index.html"), out.join("index.html"))
        .map_err(|e| format!("cannot copy web/index.html: {e}"))?;
    println!(
        "web ok: {}\nserve it with: python3 -m http.server --directory {} 8080",
        out.display(),
        out.display()
    );
    Ok(())
}

fn gen_vast(check: bool) -> Result<(), String> {
    let path = workspace_root().join("assets/levels/vast.toml");
    let text = vast::generate()?;
    if check {
        let current = std::fs::read_to_string(&path).unwrap_or_default();
        if current != text {
            return Err(format!(
                "{} is stale; run `cargo xtask gen-vast` and commit the result",
                path.display()
            ));
        }
        println!("gen-vast ok: {} is up to date", path.display());
    } else {
        std::fs::write(&path, &text)
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        println!(
            "wrote {} ({}x{})",
            path.display(),
            vast::WIDTH,
            vast::HEIGHT
        );
    }
    Ok(())
}

/// Turn timing is only meaningful with optimizations, so a debug xtask re-runs itself in release.
fn bench_turns() -> Result<(), String> {
    if !cfg!(debug_assertions) {
        return bench::run();
    }
    let status = Command::new(env!("CARGO"))
        .current_dir(workspace_root())
        .args([
            "run",
            "--quiet",
            "--release",
            "--package",
            "xtask",
            "--",
            "bench-turns",
        ])
        .status()
        .map_err(|e| format!("failed to run cargo: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("bench exited with {status}"))
    }
}

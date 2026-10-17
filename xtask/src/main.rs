//! `cargo xtask <command>` — developer automation.
//!
//! Commands:
//! - `smoke`: run the game with `--capture` and check that something was rendered.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("smoke") => smoke(),
        _ => {
            eprintln!("usage: cargo xtask smoke");
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

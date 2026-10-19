//! `cargo xtask <command>` — developer automation.
//!
//! Commands:
//! - `smoke`: run the game with `--capture` and check that something was rendered.
//! - `web`: build the WebGPU site into `target/web` (needs `wasm-bindgen-cli`, see CLAUDE.md).
//! - `web-smoke [CHROME]`: load `target/web` in headless Chromium, start a run, screenshot it.
//! - `gen-vast [--check]`: regenerate `assets/levels/vast.toml` (`--check` fails if it is stale).
//! - `licenses [--check]`: regenerate `THIRD_PARTY_LICENSES.html` (needs `cargo-about`).
//! - `golden [--update | --local]`: compare headless captures with `tests/golden/*.png`.
//! - `wiki [--build]`: write the player wiki's generated tables (`--build`: run mdBook).
//! - `bench-frames`: release-mode frame-time measurement of offscreen captures.
//! - `bench-turns`: release-mode turn-time table for the hub and the vast level.
//!
//! `target/` above is Cargo's target directory, wherever it is ([`target_dir`]): build output
//! is looked for there and xtask's own output (`xtask/`, `web/`) goes there too.

mod bench;
mod golden;
mod licenses;
mod vast;
mod wasm_check;
mod wiki;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::OnceLock;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("smoke") => smoke(),
        Some("web") => web(),
        Some("web-smoke") => web_smoke(&args[1..]),
        Some("gen-vast") => gen_vast(args.get(1).is_some_and(|a| a == "--check")),
        Some("licenses") => licenses::run(args.get(1).is_some_and(|a| a == "--check")),
        Some("golden") => golden::parse_mode(&args[1..]).and_then(golden::run),
        Some("wiki") => wiki::parse_mode(&args[1..]).and_then(wiki::run),
        Some("bench-turns") => bench_turns(),
        Some("bench-frames") => bench::frames(args.get(1).map_or("2560x1440", String::as_str)),
        _ => {
            eprintln!(
                "usage: cargo xtask <smoke | web | web-smoke [CHROME] | gen-vast [--check] | licenses [--check] | golden [--update | --local] | wiki [--build] | bench-turns | bench-frames [WxH]>"
            );
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

pub(crate) fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace root")
        .to_path_buf()
}

/// Cargo's target directory: `CARGO_TARGET_DIR` when it is set (a relative one taken from the
/// workspace root, where every cargo command xtask runs starts), else `cargo metadata`'s
/// `target_directory` (`target/`, or a cargo config's `build.target-dir`). The one place xtask
/// learns where build output is.
pub(crate) fn target_dir() -> Result<PathBuf, String> {
    static DIR: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    DIR.get_or_init(|| match std::env::var_os("CARGO_TARGET_DIR") {
        Some(dir) if !dir.is_empty() => Ok(workspace_root().join(dir)),
        _ => metadata_target_dir(),
    })
    .clone()
}

fn metadata_target_dir() -> Result<PathBuf, String> {
    let output = Command::new(env!("CARGO"))
        .current_dir(workspace_root())
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|e| format!("failed to run cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(format!("cargo metadata failed: {}", output.status));
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("cannot read cargo metadata: {e}"))?;
    metadata["target_directory"]
        .as_str()
        .map(PathBuf::from)
        .ok_or_else(|| "cargo metadata has no target_directory".into())
}

/// Runs the game binary with the given arguments (debug profile).
pub(crate) fn run_game(extra: &[&str]) -> Result<(), String> {
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
    let out = target_dir()?.join("xtask/smoke.png");
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
            "--profile",
            "web",
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

    let target = target_dir()?;
    let out = target.join("web");
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
        .arg(target.join("wasm32-unknown-unknown/web/warlock.wasm"))
        .status()
        .map_err(|e| format!("failed to run wasm-bindgen: {e}"))?;
    if !status.success() {
        return Err(format!("wasm-bindgen failed: {status}"));
    }
    optimize_wasm(&out.join("warlock_bg.wasm"))?;
    wasm_check::check(&out.join("warlock_bg.wasm"))?;
    // Stamp the workspace version into the page (like `release/Info.plist.in`).
    let page = std::fs::read_to_string(root.join("web/index.html"))
        .map_err(|e| format!("cannot read web/index.html: {e}"))?;
    std::fs::write(
        out.join("index.html"),
        page.replace("@VERSION@", env!("CARGO_PKG_VERSION")),
    )
    .map_err(|e| format!("cannot write index.html: {e}"))?;
    std::fs::copy(root.join(licenses::FILE), out.join(licenses::FILE))
        .map_err(|e| format!("cannot copy {}: {e}", licenses::FILE))?;
    println!(
        "web ok: {}\nserve it with: python3 -m http.server --directory {} 8080",
        out.display(),
        out.display()
    );
    Ok(())
}

/// Shrinks the wasm further with binaryen's `wasm-opt -Oz`. Optional locally (a hint is printed
/// when it is missing); CI installs it so the published build is always optimized.
fn optimize_wasm(wasm: &Path) -> Result<(), String> {
    let before = std::fs::metadata(wasm).map_or(0, |m| m.len());
    let tmp = wasm.with_extension("opt.wasm");
    let status = Command::new("wasm-opt")
        // Enable exactly the features rustc emits; `-all` would let newer binaryen use
        // proposals (compact imports) that browsers do not ship yet.
        .args([
            "-Oz",
            "--enable-bulk-memory",
            "--enable-nontrapping-float-to-int",
            "--enable-sign-ext",
            "--enable-mutable-globals",
            "--enable-multivalue",
            "--enable-reference-types",
            "--strip-debug",
            "--strip-producers",
            "-o",
        ])
        .arg(&tmp)
        .arg(wasm)
        .status();
    match status {
        Ok(s) if s.success() => {
            std::fs::rename(&tmp, wasm).map_err(|e| format!("cannot replace wasm: {e}"))?;
            let after = std::fs::metadata(wasm).map_or(0, |m| m.len());
            println!("wasm-opt: {before} -> {after} bytes");
            Ok(())
        }
        Ok(s) => Err(format!("wasm-opt failed: {s}")),
        Err(_) if std::env::var_os("CI").is_some() => Err(
            "wasm-opt (binaryen) is required on CI; install the pinned release (see ci.yml)".into(),
        ),
        Err(_) => {
            eprintln!(
                "warning: wasm-opt (binaryen) not found; the build works but is larger.\n  \
                 Install it: `brew install binaryen` or `sudo apt-get install binaryen`."
            );
            Ok(())
        }
    }
}

/// Loads `target/web` in headless Chromium (WebGPU), starts a run and screenshots it.
fn web_smoke(args: &[String]) -> Result<(), String> {
    let root = workspace_root();
    let target = target_dir()?;
    let web = target.join("web");
    if !web.join("warlock_bg.wasm").exists() {
        return Err(format!(
            "{} is missing; run `cargo xtask web` first",
            web.display()
        ));
    }
    let chrome = match args.first() {
        Some(c) => c.clone(),
        None => std::env::var("CHROME").map_err(|_| {
            "pass a Chromium/Chrome binary: `cargo xtask web-smoke <chrome>` (or set CHROME)"
                .to_string()
        })?,
    };
    let port = "8765";
    let mut server = Command::new("python3")
        .args(["-m", "http.server", "--directory"])
        .arg(&web)
        .arg(port)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("cannot start python3 http.server: {e}"))?;
    let out = target.join("xtask/web-smoke.png");
    let profile = target.join("xtask/web-smoke-profile");
    let _ = std::fs::create_dir_all(target.join("xtask"));
    let _ = std::fs::remove_dir_all(&profile);
    let result = Command::new("node")
        .current_dir(&root)
        .arg("xtask/web-smoke.mjs")
        .arg(&chrome)
        .arg(format!("http://127.0.0.1:{port}/"))
        .arg(&out)
        .arg(&profile)
        .status()
        .map_err(|e| format!("cannot run node: {e}"));
    let _ = server.kill();
    let _ = server.wait();
    if !result?.success() {
        return Err("web smoke run failed (see output above)".into());
    }
    println!("web-smoke ok: look at {}", out.display());
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

# Releases, the web build, and logs

Developer notes for cutting a release, the web build and GitHub Pages, and the desktop logs and
crash reports (moved from CLAUDE.md, #1505). Keep it current with `.github/workflows/`,
`xtask/src/`, and `crates/warlock/src/logging.rs`, as CLAUDE.md says.


**Cut a release.** Make sure `main` is green and README's install command names the new tag (`--tag vX.Y.Z`; the wiki introduction's is generated from the workspace version, `xtask/src/wiki/install.rs`, so the workspace version must be the tag's), then tag and push the tag (for example
`v0.1.0`); the `Release` workflow does the rest. It builds Linux x86_64 (`.tar.gz`), Windows
x86_64 (`.zip`), macOS arm64 (`.zip` with an unsigned `Warlock.app` plus the bare binary; see
`release/`) and the web build (`.zip` to serve from any static host) and attaches them, with
generated notes, to a GitHub Release; `v0.x` tags are published as pre-releases. Run the workflow
by hand (Actions > Release > Run workflow) to get the archives as workflow artifacts without
publishing. Releases are the only place macOS and Windows runners are used. Every release
artifact is built with `--no-default-features`: the test world (the `showcase` feature) is never
in a release, and the workflow checks it (docs/dev/campaign.md, "The test world in release
builds").

**Web build** (wasm32 + WebGPU; all content is embedded, nothing on the startup path may touch
`std::fs`): one-time setup is `rustup target add wasm32-unknown-unknown` and
`cargo install wasm-bindgen-cli --version <X> --locked`, where `<X>` is the `wasm-bindgen` version
in `Cargo.lock` (the CLI must match exactly; `cargo xtask web` checks it and prints the command).
Then `cargo xtask web` builds `target/web/` without the test world (`--no-default-features`;
it fails if the wasm embeds it) with the size-tuned `[profile.web]` (opt-level `s`, fat LTO,
1 codegen unit, `panic = "abort"`) and then runs binaryen's `wasm-opt -Oz` (`brew install binaryen`;
locally optional with a warning, required on CI, which installs a pinned binaryen 133 release — not Ubuntu's older apt package, whose `wasm-opt` exported the wrong table as `__wbindgen_externrefs` so the page could not start; `cargo xtask web` now checks that table, `xtask/src/wasm_check.rs`). `multi_threaded` is a
native-only Bevy feature. Sizes (`warlock_bg.wasm`): 45.9 MB raw / 10.1 MB gzip / 5.9 MB brotli
before, 20.8 MB / 6.8 MB / 4.6 MB after. Do not give wasm-opt `-all`: it emits compact imports
that Chromium rejects. After changing the web build run `cargo xtask web-smoke` and read the
screenshot (the Playwright Chromium under `~/Library/Caches/ms-playwright` works with
`--use-angle=metal`). The output is `target/web/` (`index.html` from `web/`, `warlock.js`,
`warlock_bg.wasm`). Serve it with `python3 -m http.server --directory target/web 8080` and open
http://localhost:8080 in a WebGPU browser. CI builds it on pushes to `main` (artifact
`warlock-web`). There is no command line on the web, so `cli.rs` uses the defaults.

**GitHub Pages** (https://jannef.github.io/warlock/): the `web` job of the `Release` workflow also
uploads `target/web` plus `LICENSE` as the Pages artifact, and the `pages` job (`environment:
github-pages`) deploys it on every `v*` tag; a manual run deploys only with the `publish_pages`
input ticked. The page is `web/index.html` (inline, no external requests, relative asset paths so
it works under `/warlock/`), and states the version it is for (header, tab title, loading message, footer); every `v*` tag redeploys it, so it always names the latest release: top bar with links, the canvas in `#stage` (Bevy's
`fit_canvas_to_parent` sizes it to that container), fullscreen on `#stage`, a hint footer.
`cargo xtask web` replaces `@VERSION@` in the page with the workspace version (as
`release/Info.plist.in` does in the macOS bundle). Keep the canvas id `#warlock-canvas` (the window
plugin selects it).

**Logs and crash reports** (desktop only): each run writes `warlock-<UTC>.log` (last 5 kept) in
`<data dir>/warlock/logs` (macOS `~/Library/Application Support/warlock/logs`, Linux
`~/.local/share/warlock/logs`, Windows `%LOCALAPPDATA%\warlock\data\logs`). A panic writes
`crash-<UTC>.txt` (version, OS, backtrace, last log lines; last 10 kept) next to them and prints
its path. `WARLOCK_CRASH_TEST=1 cargo run -p warlock` triggers a deliberate panic to test this.
Code: `crates/warlock/src/logging.rs`.

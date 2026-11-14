# Golden screenshots: the manifest and the update procedure

Developer notes for the golden-image check (moved from CLAUDE.md, #1505). When a golden may be
added at all, and the cheaper tools to use first, is CLAUDE.md "Golden screenshots"; read that
first. Keep it current with `xtask/src/golden.rs`, as CLAUDE.md says.

`tests/golden/manifest/` is the manifest: `defaults.toml` holds `[defaults]`, and the other
`*.toml` files (one per feature, read in file-name order; entry names are unique across files)
list the entries: a scenario and text `actions` (`move:N*3 open:E`; `*N` repeats) or a `replay` file,
and optional `character` (`human,male,paladin`: the run starts as it, not the scenario's), `kit`
(`true`: the run starts with the character's starting kit), `window`, `scale`, `freeze_time`, `screen` (a `--screen` value: over the run, e.g.
`pause`; or over the main menu, the replay then unused: `main`), `args` (more game flags, e.g.
`["--fire"]`, `["--keys", "h"]`), `tolerance` (per-channel, default 2) and `max_diff_ratio`
(default 0.05% of pixels). Walks that fight or wait for creatures belong on hand-made levels
(the showcase, the arena, the trap hall): a walk through a generated level breaks whenever the
generator changes.
`cargo xtask golden` builds a replay per entry with `warlock-core`, runs the game with
`--replay ... --capture ...`, and compares with `tests/golden/<name>.png`. On a mismatch it writes
`target/xtask/golden/<name>.{actual,diff}.png` (diff: red pixels are over tolerance). Every entry
is tried: a blocked walk (an action that takes no turn and is not free), a failed capture, a
missing reference, or a mismatch is reported and the run goes on; it fails at the end, listing
them all (and any reference that no entry names).

References are produced with **Mesa lavapipe only**, the renderer of the Linux CI job; other GPUs
and drivers rasterize differently, so on macOS `cargo xtask golden` is advisory and `--local` is
the way to look at the captures. The cloud agents' sandbox (`ops/agents/`) has the same lavapipe,
and its captures have matched CI's every time so far, so references are made there. CI compares
and never updates.

Deliberate update procedure (after an intended visual change, e.g. lighting or effects landing),
done by the PR's cloud finisher in the same pass as its merge of `main`:
1. `WGPU_BACKEND=vulkan cargo xtask golden` lists the entries that differ; look at each
   `target/xtask/golden/<name>.{actual,diff}.png` and check the change explains it.
2. `WGPU_BACKEND=vulkan cargo xtask golden --update`, then keep only the references step 1
   flagged and the change explains (`git checkout -- tests/golden/<name>.png` the rest: rewriting
   an unchanged screen may give sub-tolerance noise), and commit them with the merge.
3. Adding an entry (rare; see the warning in CLAUDE.md "Golden screenshots", and never for a screen:
   screens are text snapshots): add it to the feature's manifest file (or a new file); its
   reference is made the same way, in the same PR (CI fails on an entry without a
   reference). Never update references just to make a failing check pass: an unexplained
   difference is a bug to fix.

Fallback, when no cloud finisher is used or CI disagrees with the sandbox: push the branch, run
`gh workflow run ci.yml --ref <branch> -f update_golden=true`, download its artifact with
`gh run download <run-id> -n golden-references -D <scratch dir>`, review the images the same way,
and copy in the explained ones.

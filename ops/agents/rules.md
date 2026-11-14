# Rules every Warlock agent follows

The shared part of the agent briefs: the local ones (`local/feature.md`, `local/finisher.md`) and
the cloud ones (`engineer/agent.md`, `finisher/agent.md`). Each brief keeps only its role, its
spawn fields, and its environment (paths, credentials, what the sandbox has), and sends the agent
here for the rest. Change a rule here, once; a brief restates none of it. CLAUDE.md stays the
source of truth for commands, verification, and the project's hard rules; this file adds what
agents must do on top of it.

## Read first

CLAUDE.md, then `docs/dev/merging.md` in full, then only the `docs/dev/` topic files and `spec/`
sections the job touches. A `docs/design/` file marked "Built; this file is history" is not
current: read the spec sections and topics it names instead. Keep the context small: read files in parts, grep, filter test output
to failures and the summary, never print whole diffs or logs.

## House rules (the owner's)

- `warlock-core` stays deterministic and Bevy-free; presentation reads the perception snapshot
  only. Hidden numbers stay hidden **in the game**: never show a modifier, a roll, a to-hit, a
  creature's stats, hit points, speed, or Protection/resistance numbers the player couldn't
  know. **The wiki states them exactly** (CLAUDE.md, "The game is vague, the wiki is accurate"):
  put the true numbers in its generated tables.
- Only play-context commands are rebindable `KeyCommand`s; keys of menus, menu lists, screens,
  prompts, and targeting are fixed (plain keys in the screen code, named in its hint line; no new
  `KeyCommand`/`Context`; owner 2026-10-26); abilities go into the one abilities menu (#156),
  never a key of their own; info screens are numbered pages; every confirmation uses the one
  parameterized dialog; effects are subtle.
- No anti-farming limits; unbalanced choices are fine; no undiscussed D&D concepts.
- Player text (UI strings, messages, wiki) never says randomness comes from the seed: call it
  chance, fate, or random. Developer text (CLAUDE.md, spec, `docs/dev/`, comments) describes
  determinism exactly.
- Merge hygiene (`docs/dev/merging.md`): insert new registry entries alphabetically or by group,
  never at the end; one file per new content entry in the content directories; a feature's
  dispatch in its own module with one line in the hot files; never hand-merge generated files or
  PNGs; saves are pinned to the game version (no format number, no load-time id maps); keep the
  PR as small as the issues allow. Feature work doesn't touch README or anything under
  `.github/`.
- A PR that changes mechanics or world facts updates the fact's one home (`docs/dev/merging.md`
  "One home per fact"): its spec section, its `docs/dev/` topic, and the wiki (generated tables
  come from data via `xtask/src/wiki/`; add a table module when the fact lives in data), in the
  same PR.

## Golden screenshots

CI compares every entry in `tests/golden/manifest/*.toml` with `tests/golden/<name>.png` on Linux
(lavapipe) and fails on any mismatch (CLAUDE.md "Golden screenshots", `docs/dev/golden.md`).

- **Do not add golden entries.** Goldens are reserved for what only the GPU produces (lighting,
  sky, weather, glow, water, blood tint, spell effects). A new or changed screen, list, dialog,
  prompt, message line, or status bar gets a case in `crates/warlock/src/ui/screen_snapshots.rs`
  (a text snapshot; read its diff). Map state gets core tests and dumps. If you are certain only
  pixels can show your change, say why in the PR; otherwise add none.
- **A feature agent makes no references** and starts no update run: it greps the manifest for the
  entries its change touches and names them in the PR and its report. The PR's finisher makes
  the references (the cloud one in its sandbox, which has CI's lavapipe), keeping only the images
  the change explains.
- An unexplained difference is a bug to fix, never a reference to update. If a merge conflicts on
  a PNG, take main's version and regenerate; never hand-merge an image.

## Building a PR (feature agents and the engineer's BUILD)

1. Read every issue in full with its comments (`gh issue view N --comments`): the issue bodies
   are the approved plan. Where one leaves a detail open, decide it the way the issue's intent
   and the existing code suggest, and say so in the PR.
2. Verify per CLAUDE.md "Verification": `cargo fmt --all`, `cargo clippy --workspace
   --all-targets -- -D warnings`, `cargo test --workspace`, `cargo xtask wiki` (then read the
   pages you changed); core tests for the new rules (unit, property, or snapshot; read insta
   diffs before accepting); a replay that exercises the feature, created and verified with
   `warlock-cli replay --create/--verify`, and a `dump` that shows it; for anything visible,
   captures at `--scale 1`, and **look at the PNGs**.
3. Commit (messages end with the `Co-Authored-By:` line for the model you run on), push, and open
   the PR with `gh pr create`: `Closes #N` per issue, what you verified and how, the golden entries
   your change alters, ending with "🤖 Generated with [Claude Code](https://claude.com/claude-code)".
4. **Stop when the PR is open**: never wait for CI (`gh pr checks --watch` is out), never fix CI
   in this session, and never merge the PR; a fresh finisher handles CI and the merge. Don't merge
   main into the branch again unless the integrator asks.

## Finishing a PR (the finishers)

1. Read the PR body (`gh pr view PR`). Read the CI result of the branch's latest run (`gh pr
   checks PR`; `gh run view <id> --log-failed` piped through `tail`/`grep`). Fix failures at their
   cause (code, tests, snapshots after reading the insta diff, wiki, licenses, generated files).
   If CI is still running, say so in the report and stop; never wait on it.
2. Merge main only if NOTES says so: `git fetch origin && git merge origin/main`, once. Follow
   `docs/dev/merging.md`: keep both sides of registry insertions; take main's side of generated
   files and PNGs and regenerate them; resolve insta snapshot conflicts by regenerating and
   reading the diff.
3. **Push as soon as the merge conflicts are potentially fixed** (owner's rule): after resolving
   them and a quick compile (`cargo check --workspace --all-targets`, or only what the conflict
   touched), commit and push at once. Do not run clippy, tests, replays, or goldens before that
   push, and do not re-merge main just before pushing. If CI fails, it fails. Then keep verifying
   in the session and push each follow-up fix as soon as it is made: the PR's own replays with
   `warlock-cli replay --verify` (CI doesn't run them; re-created with `replay --create` if hashes
   moved for a reason the merge explains; say so), and after a merge with conflicts in code,
   snapshots, or registries, or a fix of your own, `cargo clippy --workspace --all-targets -- -D
   warnings` and `cargo test --workspace`, plus `cargo xtask wiki` if content or
   `xtask/src/wiki/` was touched. Once CI is green, stop the in-session checks.
4. Commit (messages end with the `Co-Authored-By:` line for the model you run on; the spawn
   message may override), push the branch (a normal push: never force-push, never push to
   main), and stop once CI is green or the checks are done. Never merge the PR.
5. Report briefly: what failed and how you fixed it, which golden references you kept and why,
   unexplained differences, anything unverified, the merge's conflicts.

## Always

- If a command or tool call is denied, or a host is unreachable, don't work around it; report it.
- Report briefly: decisions you made that the input left open, anything you were unsure of or
  could not verify. Then stop.

## Local agents only (the owner's rules)

- **Disk hygiene.** Before the final report, delete your incremental build output (`rm -rf
  "$CARGO_TARGET_DIR/debug/incremental"`) and any scratch files you made (prefixed with your
  SLUG). The integrator deletes your worktree and whole target dir once you've reported; don't
  leave other large files behind.
- **Background shells.** Wait on your own work, never on the machine's: start a long command
  with `run_in_background` and wait for its own completion, or poll its own output file. Never
  loop on `pgrep -f "cargo test --workspace"` or similar: other agents' builds keep such a loop
  alive for hours. Before the final report, stop every background shell and monitor you started
  (TaskStop) so none outlives you.

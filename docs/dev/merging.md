# Merging playbook

How feature branches land on `main` without eating their budget in conflicts (#185: merging cost
a fifth of the October wave's tokens, nearly all of it mechanical). Read it before opening a PR
and again when the integrator says your PR is next.

## When to merge main, and what to run after

- **Don't chase main.** A PR merges (or rebases onto) `origin/main` once, when the integrator says
  it is next in line, not after every merge into `main`. Until then, keep working on the branch.
  `main`'s protection does not require branches to be up to date, so a PR behind `main` still
  merges. The finisher does not re-merge `main` just before pushing.
- **Push early.** The owner's rule (2026-10-26): a finisher pushes a fix as soon as the merge
  conflicts are potentially fixed, after a quick compile and with no clippy, tests, replays, or
  goldens first. If CI fails, it fails; CI that is already ahead (red) is ahead of the game. The
  finisher keeps verifying in its session after the push (the PR's own replays, which CI does not
  run; clippy and `cargo test --workspace` after conflicts in code, snapshots, or registries or a
  fix of its own; the golden update in the cloud sandbox if anything visible changed, CLAUDE.md
  "Golden screenshots"), pushes each follow-up fix at once, and stops the checks when CI is green.
- **The full check list** (CLAUDE.md "Verification") runs once, before the PR opens.
- **Keep PRs small**: one feature per PR, rules first (core, tests, dumps), then the UI, then the
  content that uses it, when they can stand alone. Small PRs merge in minutes and conflict less.
- The integrator **schedules features by overlap**: at most about three PRs at once that add
  actions or save state; PRs that touch the same registries wait for each other.

## The false-conflict trick

`main` takes squash merges, so a branch that already contains another PR's commits (a stack, or
a branch that pulled from a sibling) sees that PR's whole diff again as a conflict although its
tree already matches. When main's squash commit `S` has the same tree as a commit `C` on your
branch (`git diff C S --stat` is empty, or nearly), record a merge without taking anything:

```sh
git merge -s ours S            # "we already have this": a merge commit with our tree
git merge origin/main          # the real merge, now only the newer commits
```

Check `git diff origin/main --stat` afterwards: only your own changes may remain.

## Generated files: never hand-merge

Take one side and regenerate; never resolve hunks in these by hand:

| File | Regenerate with |
|---|---|
| `wiki/src/generated/*.md` | not committed any more (`.gitignore`); `cargo xtask wiki` writes them |
| insta snapshots (`crates/warlock-core/src/snapshots/`, `src/mapgen/snapshots/`, `src/mapgen/local/snapshots/`) | take `main`'s (`git checkout --theirs -- <path>` while merging main), then `INSTA_UPDATE=always cargo test -p warlock-core`, then read the diff (below) |
| `assets/campaigns/showcase/levels/vast.toml` | take either side, `cargo xtask gen-vast` (`--check` must pass) |
| `THIRD_PARTY_LICENSES.html` | take either side, `cargo xtask licenses` after `Cargo.lock` is settled |
| `Cargo.lock` | take `main`'s, then `cargo build --workspace` re-resolves your additions |
| `tests/golden/*.png` | take `main`'s, then `WGPU_BACKEND=vulkan cargo xtask golden --update` in the cloud sandbox (lavapipe, as CI; CLAUDE.md "Golden screenshots"), never on macOS |
| `assets/keymaps/default.toml` | take either side, `WARLOCK_UPDATE_KEYMAPS=1 cargo test -p warlock keymap` (only a changed default key makes it stale) |

**Reading an insta diff.** `INSTA_UPDATE=always cargo test -p warlock-core` rewrites the `.snap`
files in place (`cargo insta review` walks them one by one); then `git diff` them. The showcase
dump snapshots have their state hash redacted, so a change there is a real change in what the
player sees: a glyph, a message, a light value. A mapgen snapshot is a map: a different room,
corridor, or vault placement means the generator drew differently (a new random draw before an
old one, a changed pick by index); accept it only when you can say why, and say it in the PR. A
snapshot you cannot explain is a bug until proven otherwise. Never accept a snapshot to make a
test pass.

## Hot registries: insert by group, never append

Every list a feature extends is edited by every feature. Two PRs that both append at the end
conflict every time; two that insert at different places merge cleanly. So:

- **New entries go where they belong** (alphabetically, or by the group the list is in), never at
  the end: `Action` and its replay text syntax (`action.rs`: the enum, `parse`, `Display`,
  `FromStr`), `ActionKind::ALL` and `data/actions.toml`, `KeyCommand` and `REBINDABLE`
  (`settings/keys.rs`: the controls screen shows that order, so a new command goes next to its
  group), the dump layers (`dump.rs`), the settings rows (`ui/settings_screen.rs`), the
  `Content` fields and `Content::load` steps, the `mod` lists (`world/mod.rs` is alphabetical),
  the wiki's `generate()` list (`xtask/src/wiki/mod.rs`) and `wiki/src/SUMMARY.md` (pages by
  topic), `tests/golden/manifest/*.toml` (the feature's own file), the content fingerprint
  groups, and single-file data tables (`data/slots.toml`, `stats.rs`, `data/weapon_skills.toml`,
  `data/skills.toml`, `data/factions.toml`, `data/actions.toml`, ...).
- **Fixed-size arrays become slices** when a feature would grow them: `pub const ALL: &[Kind] =
  &[...]`, never `[Kind; 23]`, so two PRs adding an entry do not both edit the length.
- **Never reorder what exists** in a list whose order is observable: the content fingerprint's
  id lists (terrain, creatures, levels, lights, dungeon branches, objects, equipment slots,
  skills, room effects, weapon skills, statuses, diseases, traps, shops: ids are indices in
  saves), the stats (`stats!`: display order), the slots (screen order), the weapon skills and
  styles (saved by index), the controls screen's binding order (`REBINDABLE`), the wiki's page
  and table order. Moving an existing entry is a deliberate change: say so in the PR.
- **Content that churns is a directory**, one file per entity: `assets/data/creatures/`,
  `objects/`, `materials/`, `affixes/` (`docs/dev/content.md`). A new creature is a new file; a
  renamed id is a renamed file; never put two unrelated entities in one file; the affix settings
  (`[rarity]`, `[magic]`, `[rare]`, `[names]`) live only in `data/affixes.toml`. A table's ids are
  indices in path order, so two PRs adding files both change the ids after theirs: that is the
  per-release save bump's business, not the PR's.
- **A feature's dispatch lives in its own module**: one match arm in `world/mod.rs::player_acts`
  calling `self.feature(..)` in `world/feature.rs`, one line each in the other hot files
  (`action.rs`, `content/mod.rs`, `dump.rs`, `input.rs`, `settings/keys.rs`), and its own tests
  file (`world/feature_tests.rs`).
- **Docs follow the same rule**: a feature edits its own `docs/dev/<topic>.md` and the `spec/`
  files of the sections it touches (bullets next to the related ones); CLAUDE.md only for a new
  command or flag; the wiki gets its page and tables; `README.md` is not touched by feature work.

## Roster and content PRs and hot docs

Creature roster PRs edit only their own `docs/dev/creatures/<tribe>.md`; never
`docs/dev/creatures.md` (the stable index) or the spec's creature and faction lists
(`spec/07b-data.md` points at `assets/data/creatures/` and `assets/data/factions.toml`; the
tag list is the one in `assets/data/creatures/README.toml`). Other hot spots seen in the
roster wave, and what to do in each:
- `assets/data/factions.toml`: insert by group, never at the end.
- test fixtures and the factions list: `world/{equip,magic,object,read}_tests.rs` take their
  at-peace factions from `world/fixture_factions.rs::peaceful_factions()` (the shipped
  `factions.toml` without its grudges), so a new faction edits none of them. `world/mana_tests.rs`
  keeps the shipped `status.toml` and replaces only the few statuses it needs
  (`status_text()`), so a creature's rider status needs no fixture edit.
- the creatures README tag list: add a tag's line by group, only when something uses the tag.
- `assets/data/plurals.toml`: insert alphabetically.
- `xtask/src/wiki/mod.rs` test rows that list creatures use `row_has(page, start, fragments)`:
  assert the stable parts of a row, never the whole list, so a creature edits no assertion.

## The save format: one bump per release

`FORMAT_VERSION` (`warlock-core/src/save.rs`) is bumped by the integrator once per release, never
in a feature PR, and its history is one line per release. Feature PRs that change `WorldState` or
the content fingerprint (any new id in an indexed table) say "saves change" in the PR and leave the
number alone: before 1.0, saves are not promised across development builds, and a save a build
cannot read is renamed `run.sav.bad`, never lost. Do not write "format version N" in the spec,
CLAUDE.md, the topic files, or the wiki.

## Design documents

A design that needs the owner's approval before anyone builds it (a feature too large for an
issue body, or one with many choices to review) is a **draft PR that adds
`docs/design/<topic>.md`** and changes nothing else.

- The owner reviews it in the PR: line comments, suggested changes, a review with questions.
  Answer by revising the document; each revision is a commit, so the owner can see what changed
  since their last read.
- **Merging the PR approves the design.** Then split it into implementation issues, each linking
  to the section it implements. The spec stays the source of truth for what is built: when a
  feature lands, its spec section and `docs/dev/` topic describe it, and the design document
  stays as the record of why.
- CI's `changes` job sees a PR that touches only `docs/design/` and skips the build. The
  required checks report as skipped, which counts as passing. Touching any other file builds as
  usual, so keep design PRs to their document.
- A document can be any length (issue and Discussion bodies stop at 65,536 characters). Prefer
  tables to prose.

## Agents, finishers, and context

The owner's token-saving rules (2026-10-24). The local subagents' briefs live in the repo, in
`ops/agents/local/` (`feature.md` for a feature agent, `finisher.md` for a finisher; the spawn
message adds the fields each one lists), so they survive a wiped `/tmp`:

1. **A feature agent stops when its PR is open.** It pushes, opens the PR, triggers CI (pushing
   does it), names in its report the golden entries its change alters or adds (it makes no
   references and starts no update run), reports briefly, and stops. It never waits for CI
   (`gh pr checks --watch` is out), never fixes CI in the same context, and is never resumed for
   the merge.
2. **A finisher agent** (fresh, small context, Sonnet; in the cloud sandbox, for its lavapipe)
   later handles one PR in one pass: it reads the CI result and fixes failures; when the
   integrator says the PR is next it merges `origin/main` (both sides of registry insertions;
   main's side of generated files and PNGs, then regenerate); it updates the golden references
   (CLAUDE.md "Golden screenshots": `cargo xtask golden`, look at every difference, `--update`,
   keep only the images the change explains; an unexplained difference is a bug to fix, not a
   reference to update); it pushes as soon as the conflicts are potentially fixed (a quick compile,
   no re-merge of `origin/main` before pushing), then keeps verifying in the session: the PR's
   replays (`replay --verify`, re-created if hashes moved), and clippy `-D warnings` and
   `cargo test --workspace` only after conflicts or a fix of its own, pushing follow-up fixes and
   stopping the checks once CI is green. There is no CI update run and no second CI
   round. It does not merge the PR: the integrator's merge watcher squash-merges a PR once it is
   non-draft and CLEAN. A local finisher (macOS, no lavapipe) falls back to the CI update run.
3. **Agents keep contexts small**: read files in parts, grep, filter test output, never print
   whole diffs or logs.
4. **Local or cloud, never both.** Every agent job above (feature, finisher, an escalation fix, a
   design document, filing issues) can run as a local subagent (subscription quota) or as a
   Claude Managed Agent in a cloud sandbox (API credits) with the same brief and the same
   stopping point; `ops/agents/README.md` has the launchers and when to pick which. The
   integrator chooses one place per job, by quota; a cloud agent never doubles a local one. A
   cloud-built PR is verified on Linux (the sandbox has CI's lavapipe, so `cargo xtask golden`
   compares there) instead of on macOS. A PR that changes golden captures gets a cloud finisher:
   only there can references be made.

The integrator schedules finishers: one per PR, when that PR is next in line.

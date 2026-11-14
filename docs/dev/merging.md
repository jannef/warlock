# Merging playbook

How feature branches land on `main` without eating their budget in conflicts (#185: merging cost
a fifth of the October wave's tokens, nearly all of it mechanical). Read it before opening a PR
and again when the integrator says your PR is next.

## When to merge main, and what to run after

- **Merge when green.** A PR is squash-merged as soon as its CI is green, without merging `main`
  first: `main`'s protection does not require branches to be up to date. After each merge,
  `main`'s CI is checked, and a red `main` is fixed before anything else.
- **Conflicts**: resolve, push at once, and let CI check it.
- **The full check list** (CLAUDE.md "Verification") runs once, before the PR opens.
- **Keep PRs small**: one feature per PR. Small PRs merge in minutes and conflict less.

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
| `tests/golden/*.png` | take `main`'s, then `WGPU_BACKEND=vulkan cargo xtask golden --update` in the cloud sandbox (lavapipe, as CI; `docs/dev/golden.md`), never on macOS |
| `assets/keymaps/default.toml` | take either side, `WARLOCK_UPDATE_KEYMAPS=1 cargo test -p warlock keymap` (only a changed default key makes it stale) |

**Reading an insta diff.** `INSTA_UPDATE=always cargo test -p warlock-core` rewrites the `.snap`
files in place (`cargo insta review` walks them one by one); then `git diff` them. The showcase
dump snapshots have their state hash redacted, so a change there is a real change in what the
player sees: a glyph, a message, a light value. A mapgen snapshot is a map: a different room,
corridor, or vault placement means the generator drew differently (a new random draw before an
old one, a changed pick by index); accept it only when you can say why, and say it in the PR. A
snapshot you cannot explain is a bug until proven otherwise. Never accept a snapshot to make a
test pass.

**Orphan snapshots.** A renamed or removed snapshot test leaves its `.snap` file behind. CI fails
on one (#1537): its test run records every snapshot file a test reads
(`INSTA_SNAPSHOT_REFERENCES_FILE`), and `.github/scripts/orphan-snapshots.sh` rejects a committed
`.snap` not among them. Delete the orphan (or `cargo insta test --unreferenced=delete`).

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
  files of the sections it touches (bullets next to the related ones); `docs/dev/cli.md` for a new
  flag, `--screen` name, scenario, dump layer, or action (by its group); CLAUDE.md only for a new
  command; the wiki gets its page and tables; `README.md` is not touched by feature work.

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

## The save format: pinned to the version

A save is pinned to the game version that wrote it (#840, `docs/dev/saves.md`): another
version's save is listed, never loaded. There is no format number: the workspace version, which
the release bumps, is the pin, and nothing carries a save across it. So a feature PR that
renames, removes, or merges an id (a talent, a story line, an affix tier) adds **no** load-time
mapping, `merged` table, or lenient read for old saves; `#[serde(default)]` on a new field is
fine. Feature PRs that change `WorldState` or the content fingerprint (any new id in an indexed
table) say "saves change" in the PR: development builds share a version, so their saves are not
promised across builds, and a save a build cannot read is renamed `<id>.sav.bad`, never lost.
The release that bumps the workspace version also sets `version` in the fixture summaries under
`tests/golden/saved-characters/saves/` (all but Osric's, another version's on purpose).

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

## Agents and context

The local subagents' briefs live in `ops/agents/local/` (`feature.md`, `finisher.md`), so they
survive a wiped `/tmp`. A feature agent stops when its PR is open: it never waits for CI and is
never resumed. Agents keep contexts small: read files in parts, grep, filter test output.

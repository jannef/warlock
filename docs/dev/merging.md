# Merging playbook

How feature branches land on `main` without eating their budget in conflicts (#185: merging cost
a fifth of the October wave's tokens, nearly all of it mechanical). Read it before opening a PR
and again when the integrator says your PR is next.

## When to merge main, and what to run after

- **Don't chase main.** A PR merges (or rebases onto) `origin/main` once, when the integrator says
  it is next in line, not after every merge into `main`. Until then, keep working on the branch.
- **After that one merge** run `cargo fmt --all`, `cargo clippy --workspace --all-targets --
  -D warnings`, `cargo test --workspace`, `cargo xtask wiki`, and the PR's own captures or
  replays (what it changed visibly). CI covers the rest (wasm32 check, screenshot smoke test,
  `mdbook build wiki`, `cargo deny`, the third-party licenses).
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
| `assets/levels/vast.toml` | take either side, `cargo xtask gen-vast` (`--check` must pass) |
| `THIRD_PARTY_LICENSES.html` | take either side, `cargo xtask licenses` after `Cargo.lock` is settled |
| `Cargo.lock` | take `main`'s, then `cargo build --workspace` re-resolves your additions |
| `tests/golden/*.png` | take `main`'s; references come only from the Linux CI workflow |
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

## The save format: one bump per release

`FORMAT_VERSION` (`warlock-core/src/save.rs`) is bumped by the integrator once per release, never
in a feature PR, and its history is one line per release. Feature PRs that change `WorldState` or
the content fingerprint (any new id in an indexed table) say "saves change" in the PR and leave the
number alone: before 1.0, saves are not promised across development builds, and a save a build
cannot read is renamed `run.sav.bad`, never lost. Do not write "format version N" in the spec,
CLAUDE.md, the topic files, or the wiki.

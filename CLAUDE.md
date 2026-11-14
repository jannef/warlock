# CLAUDE.md — working on Warlock

Warlock is a GPU-rendered ASCII roguelike in Rust + Bevy. **`specification.md` is the source of
truth** for what to build (an index over `spec/`, one file per section); this file is about *how*
to work on it, and `docs/dev/` holds the notes per subsystem (the index is under "Architecture").
Each kind of fact has one home and the others link to it (`docs/dev/merging.md` "One home per
fact"): a change edits the homes of the facts it changes.

## Toolchain

- Rust is pinned in `rust-toolchain.toml` (rustup installs it automatically).
- Cargo lives in `~/.cargo/bin`. In a fresh shell: `source ~/.cargo/env` (bash/zsh) or
  `source ~/.cargo/env.fish` (fish).

## Commands

| Task | Command |
|---|---|
| Build everything | `cargo build --workspace` |
| Run the game | `cargo run -p warlock` |
| Run the headless CLI | `cargo run -p warlock-cli -- --help` |
| Tests | `cargo test --workspace` |
| Lint (CI-equivalent) | `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings` |
| WASM check | `cargo check -p warlock -p warlock-core --target wasm32-unknown-unknown` |
| Dependency audit | `cargo deny check` |
| Screenshot smoke test | `cargo xtask smoke` (writes `target/xtask/smoke.png`) |
| Web build | `cargo xtask web` (writes `target/web/`, without the test world; see "Releases, web build, logs") |
| Release configuration | `cargo build --no-default-features -p warlock -p warlock-cli` (the test world left out, as in a release; docs/dev/campaign.md) |
| Web smoke test | `cargo xtask web-smoke <chrome>` (drives headless Chromium through a new run and a reload; not run in CI) |
| Regenerate the vast level | `cargo xtask gen-vast` (writes `assets/campaigns/showcase/levels/vast.toml`; deterministic, never hand-edit) |
| Check the vast level is fresh | `cargo xtask gen-vast --check` (run it after touching `xtask/src/vast.rs`) |
| Check the docs' code paths | `cargo xtask docs-check` (every source file, and item of a file, type, or module, named in CLAUDE.md, `spec/`, `docs/dev/` exists; CI runs it; docs/dev/cli.md) |
| Third-party notices | `cargo xtask licenses` (regenerates `THIRD_PARTY_LICENSES.html`; `--check` in CI). **Run it after changing dependencies**; keep `about.toml`'s `accepted` in sync with `deny.toml`'s `allow` |
| Player wiki tables | `cargo xtask wiki` (writes `wiki/src/generated/*.md`), `--build` (mdBook into `wiki/book/`), `--check-links` (CI runs it) |
| Golden screenshots | `cargo xtask golden` (compare; authoritative on Linux only), `--local`, `--update` (see "Golden screenshots") |
| Frame-time benchmark | `cargo xtask bench-frames [WxH]` (release, offscreen; `docs/perf.md`) |
| Turn-time benchmark | `cargo xtask bench-turns [ROW...]` (re-runs itself in release; rows named by label run alone, `bench-turns l-1 long`; scenes and budgets in `xtask/src/bench.rs` and `docs/perf.md`) |
| Light stress benchmark | `cargo test --release -p warlock-core turn_cost -- --ignored --nocapture` |
| Balance probe | `cargo test --release -p warlock-core balance_probe -- --ignored --nocapture` (run after tuning combat data; update `docs/balance.md`) |

Each row in detail: `docs/dev/cli.md` "`cargo xtask` commands in detail".

`target/` in the xtask rows is Cargo's target directory: xtask follows `CARGO_TARGET_DIR` (else
`cargo metadata`'s `target_directory`) for the build output it reads and the files it writes
(`xtask/src/main.rs::target_dir`, the one place).

**The CLI reference is `docs/dev/cli.md`**: every game flag (`--scenario` names, `--screen`
names, the capture and dev flags), `warlock-cli`'s commands, the replay format, the dump layers,
and the action text syntax. Read it before scripting a run; add to it when you add a flag, a
screen, a scenario, a dump layer, or an action. In brief:

- `cargo run -p warlock -- --scenario NAME [--seed S] [--character RACE,GENDER,CLASS] [--name
  NAME]` starts a run (without these the game opens on the main menu); `--content-dir assets`
  reads content from disk; `--help` lists every flag and the values it takes.
- Captures: `--capture out.png` renders offscreen and exits; `--window WxH`, `--scale 1`,
  `--freeze-time SECONDS`, and `--reduced-effects` make them reproducible; `--replay FILE` sets up
  any state first, `--screen NAME` opens a screen, `--keys "KEYS"` presses keys.
- `warlock-cli dump --scenario showcase --actions "move:N wait" --layers view,fov,memory` prints
  the simulation as text (`--format json`, `--state`); `replay --create ... -o x.json` and
  `replay --verify x.json` check determinism; `gen --level depths:3` previews a generated level.
- Action text is whitespace-separated `verb:args` words (`move:N open:E pickup:a equip:body:b
  fire:12,5 rest:dawn`, `wait*10` repeats); dev actions (`xp:2500 levelup:auto afflict:bleeding
  effect:fireball@30,14 ...`) mostly take no game time.

Typical render check: `cargo run -p warlock -- --scale 1 --freeze-time 1 --capture target/shot.png`,
then read the PNG.

## Architecture (spec §8)

```
crates/warlock-core   plain Rust, no Bevy, deterministic simulation; takes its content as input
crates/warlock-assets the content under assets/ embedded for the binaries (#1587)
crates/warlock-cli    headless dev tool on top of core (no Bevy)
crates/warlock        Bevy app: rendering, input, UI, effects, platform
crates/workspace-hack no code: pins features so every member shares one warlock-core build (docs/dev/merging.md)
xtask                 screenshot capture, golden-image comparison, vast level generator, benchmarks
assets                fonts, data definitions, vaults, campaigns (each a world: levels, scenarios, ...)
```

Each subsystem has a topic file under `docs/dev/` with its modules, data, invariants, and
tests. **Read the ones your task touches before changing code, and update them with it**
(a feature PR adds or edits its own topic file; nothing about one subsystem goes in this
file). Topics:

- `docs/dev/cli.md` — the CLI reference: game flags, `--scenario` and `--screen` names, `warlock-cli`, the `cargo xtask` rows in detail, replay files, dump layers, action text syntax
- `docs/dev/content.md` — TOML content embedded by `warlock-assets` and handed to core, level files and legends (faction overrides, arrival lines, water flow)
- `docs/dev/campaign.md` — campaigns: `assets/campaigns/<id>/`, `campaign.toml`, what is a campaign's and what is shared, the cross-campaign checks, a run's campaign, the fingerprint per campaign, New run's order and its skip in a release, the Characters screen's campaign
- `docs/dev/saves.md` — storage backends (native files, browser `localStorage`), one save per character and the list of them, autosave, permadeath, where files live (`paths.rs`, portable mode)
- `docs/dev/graves.md` — burial grounds: the run's dead laid in Līctūn's plots and crypts, the `[burial]` table and the legend marks (places, fixed graves, the way down, statue places) a crypt level needs, the chain, out of the state hash
- `docs/dev/releases.md` — cutting a release, the web build and its size tuning, GitHub Pages, logs and crash reports
- `docs/dev/settings.md` — `settings.toml`, effect switches, clock, palettes, rebindable key commands (`settings/keys.rs`), the `Game` resource
- `docs/dev/screens.md` — the screen stack (open, input, draw, the dialog, info pages, the text field), prompts as input modes, the screen modules, key commands and the abilities menu
- `docs/dev/time.md` — `GameTime`, the calendar, action costs and prices, the energy scheduler, rest, creature speed
- `docs/dev/hearing.md` — noise per action, propagation, what creatures and the player hear, sound markers
- `docs/dev/objects.md` — object and material data, parts and instance parts, gems, stacks, piles, inventories, weight, pick up and drop
- `docs/dev/equipment.md` — slots as data, wielding and wearing, what combat reads from equipment
- `docs/dev/combat.md` — the 2d12 rules, Defence categories, strikes, hit points and death, messages
- `docs/dev/ranged-combat.md` — firing and throwing, missiles in flight, fires, the aiming mode
- `docs/dev/weapon-skills.md` — ranks, marks, benefits, training hooks, the styles' grips and rows
- `docs/dev/weapon-abilities.md` — what ranks teach: the data, who has one, the switches, `Trigger` and `weapon_trigger`, `WeaponDoes`, statuses that ease by actions, the menu rows
- `docs/dev/stats.md` — the six stats, modifiers, scores and potentials, what may be shown, stat training (marks by use, the level-up checks)
- `docs/dev/burden.md` — load, limits, sizes, every penalty's one hook
- `docs/dev/hunger.md` — satiation, eating, corpses and rot, starving
- `docs/dev/profane-gifts.md` — the profane gifts: `data/profane/`, `ProfaneTable` and its gifts as talents of the `profane` tree, the loader's checks, `World::bestow_gift` and `rescind` with the relations, the dumps
- `docs/dev/character-creation.md` — `CharacterSpec`, races, classes, genders, the horoscope, starting rolls, names, starting kits
- `docs/dev/background.md` — background stories: `data/background/` (and a campaign's chapter), the loader's checks, `background::tell` (streams, tiers, `only`, slots; no caps), `World::tell_background` and its effects, `Character::story`, the `story` dump
- `docs/dev/alignment.md` — the good–evil score and its bands, the deeds that move it (one hook each), what reads it
- `docs/dev/skills.md` — trained skills, effects and uses, training marks, the skills screen
- `docs/dev/crafting.md` — professions and recipes: `data/recipes/<skill>/`, the loader's checks, `Ability::Profession`, `PerceptionView::recipes` and its reasons, the profession screen, `craft:`, making a recipe (the forge's check and grades), the forging wizard; who works which material (`Effect::Craft` read per skill), and salvage: the ability, the checks per material, gems chipped or shattered, the marks
- `docs/dev/experience.md` — the level curve, kill XP, level-ups
- `docs/dev/progression.md` — talents (data, `TalentMods`, the hooks), the picks a level brings and their refusal, the `levels` stream
- `docs/dev/status-effects.md` — statuses and severities, healing paces, checks, diseases and stat damage
- `docs/dev/mana.md` — the second pool: its maximum, regeneration in the status loop, every modifier's one hook, spending, the dev drain
- `docs/dev/caster-styles.md` — the caster styles (Sniper, Toucher, Battle caster): the grip by the spell, their benefit in the hooks, `practise_cast` and the `Outcome`, the caster curve, the Magic page (the schools of magic are only spell categories since #1263)
- `docs/dev/spells.md` — the spell data: spell levels (0 to 70), the cost table, the `[[spell]]` files and their checks (none on strength), tags, learning
- `docs/dev/resources.md` — alternative resources: the data, `Actor::resources`, grant, gain, spend, and end, time in the status loop, ability costs, the dev grant
- `docs/dev/openings.md` — the thief's openings: the mark (`Actor::mark`, `world/classes/mark.rs`), chosen attacks and free blows, the builds (`mark_kill`), the clears, the refusal, what the player sees
- `docs/dev/blood.md` — wounds and stains, bleeding, how blood is drawn, blood by damage type, fire's char
- `docs/dev/creatures.md` — factions and stance, the AI, paths, creature sight, spawning, gear (a stable index: roster PRs edit only their own `docs/dev/creatures/<tribe>.md`, e.g. `undead.md`, never this file)
- `docs/dev/summoning.md` — creatures' summoning (enemy-only): the `summon` table, the cooldown status, who summoned whom, the AI hook, the words
- `docs/dev/allies.md` — allies: `Actor::ally`, the side, the ally arm of the AI, the three orders, following across levels and the retinue, the clock, an ally's kill, Beast-bond, the dev `ally:`
- `docs/dev/companion.md` — the animal companion: `AllySource::Companion(CompanionData)`, the starting beast by race and the rares (`data/companions.toml`), taming (`world/allies/tame.rs`), the name, growth with the master's level (the hooks in `actor.rs`), always along, the nose, `CreatureDef::{tameable, nose}`, the story's chapter, the `companion` dump
- `docs/dev/familiar.md` — the witch's familiar: `AllySource::Familiar(CompanionData)` and `kept()`, the starting beast by race (`data/familiars.toml`), the touch, the harrying, the direction asked with two kept beasts beside, the `familiar` dump
- `docs/dev/animated.md` — animated objects: creature types generated at load, their bodies and streams, drops, immunities, the look
- `docs/dev/npcs.md` — unique characters, chat and ask, rotation lines and mutters, displacement
- `docs/dev/quests.md` — quest data and stages, hooks in kills, travel, and talk, the quest log
- `docs/dev/guild.md` — guilds and their ranks: the data, the derived rank, the promotion hook, the Weyfarers' Gilde
- `docs/dev/shops.md` — shop data, keepers, prices, buying, selling, theft, the shop dialogs
- `docs/dev/stealing.md` — the Steal ability and action, the contest, what is taken, a caught thief, the alignment hook
- `docs/dev/surprise.md` — unaware both ways round, `Mind::noticed`, the initiative check, the Surprised status and its hooks
- `docs/dev/doors.md` — locks and keys, kicking, breaking, what creatures open
- `docs/dev/chests.md` — chests as cell fixtures: the bands, opening, keys, Pick lock and Knock, smashing, blasts, what perception shows
- `docs/dev/digging.md` — digging tools, wall hardness, the dig loop, blasts, mining (ore, stone, and gems from a wall dug through; `data/mining.toml`, a branch's `ores`)
- `docs/dev/traps.md` — trap data, triggering, known traps, the dev trap actions
- `docs/dev/searching.md` — secret doors, the active and passive checks, who notices, the secrets stream
- `docs/dev/magic-items.md` — affixes, rarities, base items, loot, identification, the item description
- `docs/dev/enchanting.md` — affix capacity (`magic/capacity.rs`, materials' `binding`), the trim as the last step of making any item (`loot::finished`, gear's after its condition), `quality` on hand-made items, the capacity line for the Appraise-trained, the recipes (Kindle, Quicken, Recast, Imbue, Ennoble, Unbind: `world/crafting/enchant.rs`, `roll_affix_within`, the reach)
- `docs/dev/potions.md` — potions: the drink kinds, drinking through Consume, words and colours, sources, their talents
- `docs/dev/prayer.md` — the priest's prayer: `data/prayer.toml`, the prayer talents and `FaithMods::prayer`, the face, the problems in priority order, the strain and its lock, the fixes, the dumps
- `docs/dev/scrolls.md` — reading, spell scrolls on the traps' shared spell code, the scroll of identify, the read and identify lists
- `docs/dev/overworld.md` — the overworld level, locations, sight at map scale, generated local areas
- `docs/dev/encounters.md` — overworld encounters: the rules file and the campaigns' tables, the roll, the lookouts, the pending choice, evading, where everyone stands by outcome, known parties
- `docs/dev/mapgen.md` — RNG streams, dungeon branches, generators, vaults, snapshots
- `docs/dev/special-rooms.md` — room effects as data, the draw (weights, favoured and banned, eligibility), whole-level effects, `effects_at`, what the player is told
- `docs/dev/rendering.md` — the three layers, the console shader, visual lighting and creature shadows, light stability, particles, glow, post, rain, spell effects, the `ui/` modules
- `docs/dev/golden.md` — the golden screenshots' manifest, the compare, lavapipe references, the update procedure
- `docs/dev/sky.md` — the sun's path, the moons, the sky, weather blocks and clouds
- `docs/dev/temperature.md` — air temperature, freezing water, ice thickness, falling in

`docs/dev/merging.md` is the merging playbook: generated files, hot registries, insta
diffs, the save format policy, and the false-conflict trick.

Hard rules:
- `warlock-core` never depends on Bevy, never reads wall-clock time, and never does platform I/O
  inside the rules. Iteration order in core must be deterministic (no `HashMap` iteration).
- Rendering, UI, and dumps read the per-turn **perception snapshot** only — never ground truth —
  so hidden information cannot leak. The only exception is an explicit dev "reveal" mode.
- Gameplay outcomes never depend on frame rate, animation, or visual randomness.
- Only play-context (gameplay) commands are rebindable: a `KeyCommand` in `input::Context::Play`,
  rebindable in Controls and listed by `?`. The keys of menus, menu lists (rows named by letters
  and digits, paging, confirm and cancel inside lists), screens, info pages, prompts, dialogs,
  text fields, and the targeting mode are FIXED, never rebindable (owner, 2026-10-26: "do not
  fucking make menus rebindable. that just gets too messy"; "at least menu lists like that. it
  just won't work"). New ones are plain fixed keys in their screen or input-mode code, named in
  the screen's hint line; add no `KeyCommand` or `Context` entry for them.
  The back-style ones (`Back`, `TextBack`, `MoreSkip`, `PrevPage`,
  `PrevTarget`, unbound) stay exactly as they are (owner, #644).
- Abilities (racial, class, skill, weapon skill and style, item; #156) register in
  `warlock_core::ability`, the one registry, and are used from the one abilities menu (`a`):
  never a default key of their own (an ability's own `KeyCommand` stays unbound by default,
  `settings/keys.rs::UNBOUND`, and Controls lists it last, `ABILITY_KEYS`). Basic commands keep their keys (`docs/dev/screens.md`).
- Keep modules small with clear responsibilities. No speculative abstractions, no premature
  optimization.

## Wiki

`wiki/` is the player wiki: Markdown pages for players (rules and world facts, not developer
notes, and no locations or level walkthroughs yet), built with [mdBook](https://rust-lang.github.io/mdBook/)
(`wiki/book.toml`, table of contents in `wiki/src/SUMMARY.md`, default theme `coal`). It is
published at https://jannef.github.io/warlock/wiki/ by the release workflow's web job, which
builds it into `pages/wiki/` beside the game. Facts that live in data (months, moons, weather,
temperatures by month and weather, water that freezes, ice by thickness and what it holds, how
long water takes to freeze, terrain, footstep loudness in words, vision profiles, light presets (named as the player sees
them), creatures, factions, objects, materials, equipment slots and what fits them, scenarios,
action costs, sight radii, item classes and
what is in each, the stats' names, races (stat adjustments as written, dice included, vision, starting
ages, hit points), classes (stat adjustments, hit points, level tables), prestige classes (their tables, and each
world's callings with the words that make them), the bard's songs (what Music adds, the odds of shaking one off),
what each gender raises, the horoscope
(animals, elements, and the great cycle's signs and years), background stories (the kinds' odds and
worth, the points, every chapter's lines with their conditions and effects), combat (weapons, armour and shields,
the 2d12 hit chances and threat ranges, regeneration by Toughness), status effects (what causes
them, what they cost, how they wear off, healing and stats) and healing paces, diseases (how
they are caught, the stat and the chance an average score resists, incubation, progression,
damage, recovery, whether they kill, healing), burden (its levels
and what each costs, the races' sizes, sample carrying limits), and the exact numbers the game
itself keeps vague (speed, hearing thresholds, creature sight, creature weights, stat modifiers
and potentials, to-hit, creatures' stats, hit points, natural attacks and armour, resistances))
are tables generated by
`cargo xtask wiki`
(`xtask/src/wiki/`: one module per topic, `generate()` in `wiki/mod.rs` the one registration
list) into `wiki/src/generated/` (ignored by git: a build product) and pulled into pages with
`{{#include generated/NAME.md}}`; CI runs `cargo xtask wiki --check-links` (generate, build, check
links) on the Linux job, and the release workflow does the same before publishing (mdBook pinned in both). To
preview: `cargo xtask wiki --build`, then open `wiki/book/index.html`. Tables sort by explicit
keys (class, level, name), never by content file order. New pages go into
`wiki/src/SUMMARY.md` by topic, never at the end, in the part where a player would look (the parts
run in the order a player meets things: Getting started first, How the world works, the reference,
last; #1522); a new table is its own function in its
topic's module plus one line in `generate()` next to its topic. A page that grows past about
250 lines is split into sub-pages by topic (flat files in `wiki/src/`, nested under the parent in
`SUMMARY.md`; the parent keeps a short overview linking them), its moved anchors fixed everywhere
they are linked (pages, `xtask/src/wiki/`, `wiki/TERMS.md`).

**Wiki prose is written for players** (#1172): plain words, short sentences and paragraphs, the
question a player would ask answered first; no developer terms (modules, fields, ids the player
never sees) and no padded register (stacked hedges, empty summaries, a heading that restates the
page). The numbers stay exact.

**Tables a human can read** (#1586): a big generated table follows two or three plain lines on
what it shows and how to read it, never the page's first line; long tables go at the end of the
page. A table has at most 5 columns where practical: split a wide one by topic (the stats apart
from the rest), turn one round (a row per skill effect, its values as columns), and put rare
fields in a detail table or a per-row cell. A grid of short numbers along one axis (odds by bonus,
a value by level) may be wider. Stat abbreviations (`St`, `Dx`, ...) are spelled out above the
table that uses them (`wiki/util.rs::stat_key`), and no cell holds code or action text (`use:...`).

**The game is vague, the wiki is accurate** (the owner, 2026-10-20): in the game, hidden numbers
stay hidden and effects are told in words (the perception rules above are unchanged), but the
wiki states the truth exactly. The game is open source, so the truth is one look at the sources
away, and the wiki is also where facts are inspected during development.

**Wiki prose links its terms once per section** (the owner, #1609): new or edited wiki prose
links each mechanic or keyword at its first mention in a section to the section that explains it
(`wiki/TERMS.md`, the term map, says where; generated tables anchor every row,
`status-list.md#status-staggered`), and not again in that section; CI checks the links
(`cargo xtask wiki --check-links`).

**Worked examples are tested**: the hand-written `> **Example: ...**` boxes (Combat, Stats, Burden,
Skills, Mana, Enchanting) are checked by `xtask/src/wiki/examples.rs` (`cargo test -p xtask`), which
rebuilds each example's character through `warlock-core` and fails unless the box states the
numbers the game computes. A tuning change that moves one fails there: fix the box. A new box gets
a test there too.

**A PR that changes gameplay mechanics or world facts updates the wiki in the same PR**, like the
spec and this file: edit the page, and generate a table in `xtask/src/wiki/` if the fact lives
in data. Run `cargo xtask wiki` after changing content.

## Bevy 0.20 — pinned references

Bevy 0.19 (2026-06) and 0.20 (2026-10) are newer than most model training data. **Do not write
Bevy code from memory; check the pinned sources.**

- API docs: https://docs.rs/bevy/0.20.0/bevy/
- Examples at the pinned tag: https://github.com/bevyengine/bevy/tree/v0.20.0/examples
- Release notes: https://bevy.org/news/bevy-0-20/ and https://bevy.org/news/bevy-0-19/
- Migration guides: https://bevy.org/learn/migration-guides/
- Fastest: a local shallow clone to grep —
  `git clone --depth 1 --branch v0.20.0 https://github.com/bevyengine/bevy /tmp/bevy-0.20`
  (relevant: `examples/shader/shader_material_2d.rs`, `examples/2d/`, `examples/window/screenshot.rs`,
  `crates/bevy_sprite_render/src/mesh2d/`).

Known differences from older Bevy:
- Shaders are **WESL**: `import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;`,
  conditional code via `@if(FLAG)`, material bind group via `@group(constants::MATERIAL_BIND_GROUP)`.
  The old `#import`/`#ifdef` dialect is gone. Embed shaders with `embedded_asset!` and reference
  them with `AssetPath::from_path_buf(embedded_path!("x.wesl")).with_source("embedded")`.
- `Material2d`, `Mesh2d`, `MeshMaterial2d`, `AlphaMode2d` live in `bevy::sprite_render`.
- Buffered events are **messages**: `MessageWriter<AppExit>`, `MessageReader<T>`; observers take
  `On<E>`.
- Bevy features are trimmed in the workspace `Cargo.toml` (`default-features = false`, leaf
  features only: no `bevy_text`, `bevy_state`, `bevy_animation`, or gizmos, #1590). Adding a
  Bevy feature is a deliberate change. `tonemapping_luts` is deliberately off (it pulls `ruzstd`
  0.8, RUSTSEC-2026-0344); cameras use `Tonemapping::None`, which also keeps palette colors exact.

## Verification

A successful compile does not verify anything visual. Before opening a PR:
1. `cargo fmt`, clippy with `-D warnings`, `cargo test --workspace`.
2. Simulation changes: add or update unit, property, or snapshot tests in `warlock-core` (its
   shared helpers, `crate::test_support`: the built-in content, `world`, `run`, the message readers)
   (`insta` snapshots live in `crates/warlock-core/src/snapshots/`, `src/dump/snapshots/`, `src/mapgen/snapshots/`, and
   `src/mapgen/local/snapshots/`, the screens' in `crates/warlock/src/ui/snapshots/`; review with `cargo insta review` or `INSTA_UPDATE=always`, and read the diff). Look at the
   result with `warlock-cli dump` (or `gen` for a generated level), and check determinism with
   `replay --create` then `replay --verify`.
3. Rendering changes: capture a screenshot with `--capture` and **look at it** (read the PNG).
   UI changes (screens, prompts, the message line, the status bar): add or update a case in
   `crates/warlock/src/ui/screen_snapshots.rs` and read the text diff.
   Golden images are updated deliberately, never to make a failing check pass.
4. Note in the PR what you verified and how.

## Workflow

- Work is tracked in GitHub issues under milestones (and Housekeeping); use the `gh` CLI.
- One branch and PR per issue (or a tight group of related issues); reference issues with
  `Closes #N`. CI must be green before a squash merge into `main`.
- CI runs on Linux only (`CI` workflow: fmt, clippy, tests, headless screenshots in a job beside them, wasm32 check,
  cargo-deny, the release configuration without the test world; the web build on pushes to
  `main`). macOS is not run in Actions: development happens
  on macOS, so run the full local check there before every PR (a PR built by a cloud agent,
  `ops/agents/`, runs it on Linux instead). `main` is branch-protected: PRs
  only, the Linux, wasm32, and cargo-deny checks must pass, linear history.
- **Merging** (`docs/dev/merging.md`): a PR is squash-merged as soon as its CI is green;
  branches need not be up to date with `main`, so `main`'s CI is checked after each merge and a
  red `main` is fixed first. Agents stop when their PR is open.
- **Keep PRs small**: one feature per PR.
- **Hot files**: new entries in registries go where they belong (by group or alphabetically),
  never at the end; generated files are never hand-merged (take one side, regenerate);
  saves are pinned to the game version (no format number, no load-time id maps); a
  feature's dispatch lives in its own module with one line in each hot file; churning content
  tables are directories, one file per entity. All in the playbook.
- **Design documents** pending the owner's approval are draft PRs that add
  `docs/design/<topic>.md` and touch nothing else: the owner reviews them line by line in the PR,
  each revision is a commit, and merging the PR approves the design. CI skips the build for them.
  Implementation issues link to the merged document. Details in the playbook.

## Golden screenshots

**Do not add a golden screenshot unless you are certain nothing else can check the change.**
Goldens are reserved for what only the GPU produces: lighting regimes, sun, moon and weather,
glow and heat haze, animated water, blood tint, spell effects, a missile in flight, and one
overlay over a run (#56's guard). In #301 they were cut from 103 to 16. Nearly every difference
they had reported was an intended change that cost a lavapipe update pass and an image review,
and none had caught an unintended regression. Check the cheaper tools first:
- a screen, list, dialog, prompt, message line, or status bar → a **text snapshot**: a case in
  `crates/warlock/src/ui/screen_snapshots.rs` (the UI console as text, `insta`; docs/dev/screens.md);
- what is on the map, who is where, what happened → a core test, an `insta` snapshot, or a
  `warlock-cli dump`;
- a one-off visual check → `--capture`, looked at, and noted in the PR (not kept).

A PR that adds a golden says in its description why none of these can see the change; reviewers
and the integrator refuse one that doesn't. When in doubt, don't add one.

The manifest (`tests/golden/manifest/`), what `cargo xtask golden` does, why references are
made with Mesa lavapipe only (on macOS the check is advisory: use `--local`), and the deliberate
update procedure (done by the PR's cloud finisher; never to make a failing check pass: an
unexplained difference is a bug to fix) are in `docs/dev/golden.md`.

## Releases, web build, logs

Cutting a release (tag `v*` on a green `main`; the `Release` workflow builds every platform and
the web build, always `--no-default-features`, without the test world), the web build's setup and
size tuning, GitHub Pages, and the desktop logs and crash reports are in `docs/dev/releases.md`.
Nothing on the web build's startup path may touch `std::fs`; after changing the web build run
`cargo xtask web-smoke` and read the screenshot.

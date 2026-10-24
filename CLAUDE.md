# CLAUDE.md — working on Warlock

Warlock is a GPU-rendered ASCII roguelike in Rust + Bevy. **`specification.md` is the source of
truth** for what to build (an index over `spec/`, one file per section); this file is about *how*
to work on it, and `docs/dev/` holds the notes per subsystem (the index is under "Architecture").
Update them together whenever behavior, commands, or architecture change.

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
| Web build | `cargo xtask web` (writes `target/web/`; see "Releases, web build, logs") |
| Web smoke test | `cargo xtask web-smoke <chrome>` (serves `target/web`, drives headless Chromium through menu → New run → race, gender, class (preselected) → name (empty: a random one) → summary → hub, checks the new character's save `warlock/saves/<id>.sav`, waits five turns, reloads, Continue, and checks the run resumed at turn 5; writes `target/xtask/web-smoke.png`; fails on logged errors; not run in CI) |
| Regenerate the vast level | `cargo xtask gen-vast` (writes `assets/levels/vast.toml`; deterministic, never hand-edit) |
| Check the vast level is fresh | `cargo xtask gen-vast --check` (fails if the file differs from the generator output; run it after touching `xtask/src/vast.rs`) |
| Third-party notices | `cargo xtask licenses` (regenerates `THIRD_PARTY_LICENSES.html` with `cargo-about`, config `about.toml`/`about.hbs`; install: `cargo install cargo-about --locked --features cli`); `--check` fails when it is stale (CI runs it). **Run it after changing dependencies** (`Cargo.toml`/`Cargo.lock`); keep `about.toml`'s `accepted` list in sync with `deny.toml`'s `allow` |
| Player wiki tables | `cargo xtask wiki` (writes `wiki/src/generated/*.md` from the game data; not committed: CI and the release workflow run it before `mdbook build wiki`), `--build` (also runs `mdbook build wiki` into `wiki/book/`; needs `cargo install mdbook --locked`) |
| Golden screenshots | `cargo xtask golden` (compare; authoritative on Linux only), `--local` (capture to `target/xtask/golden/*.local.png`, no compare), `--update` (rewrite references; see "Golden screenshots") |
| Frame-time benchmark | `cargo xtask bench-frames [WxH]` (release, offscreen; method and results in `docs/perf.md`) |
| Turn-time benchmark | `cargo xtask bench-turns` (re-runs itself in release; budgets: hub 5 ms, vast 16 ms, generated depths level 5 ms, sunlit courtyard at dawn 5 ms, overworld 5 ms, generated forest area at dawn 5 ms, courtyard and forest area at full-moon midnight 5 ms, arena 5 ms per turn; a dusk-to-dawn rest 500 ms, also beside the arena's fight going on unseen until one side is dead (a walk that gets the player killed starts over, untimed), and rested on to dawn when rain wakes the player) |
| Light stress benchmark | `cargo test --release -p warlock-core turn_cost -- --ignored --nocapture` (400×300, 825 lights; again outdoors at 06:15) |

`target/` in the xtask rows is Cargo's target directory: xtask follows `CARGO_TARGET_DIR` (else
`cargo metadata`'s `target_directory`) for the build output it reads and the files it writes
(`xtask/src/main.rs::target_dir`, the one place).

Game flags (`cargo run -p warlock -- --help`):
- `--scenario NAME` (default `showcase`; `depths` starts in the generated dungeon, `courtyard` in
  the sunlit courtyard at 06:30, `world` on the overworld beside the Hall, `forest` in the glade
  of a generated forest area of the overworld at 06:30, `arena` in the dark corner of the arena
  where kobolds and giant rats fight, `quarry` in the quarry beside a pick-axe, `traps` in the
  trap hall), `--seed S`
  (default: the scenario's seed; a number, or a word hashed with FNV-1a, e.g. `--seed warlock`),
  `--character RACE,GENDER,CLASS` (e.g. `dwarf,female,warrior`; default: the scenario's
  character; like `--seed` it shapes the run other flags start and alone does not skip the menu),
  `--name NAME` (the character's name, up to 20 characters; default: the scenario's for its own
  character, else one of the race's names picked from the seed; like `--character`). A run
  started with `--character` or `--name` (or from New run) has a chosen character and starts with
  its starting kit; `--scenario` alone starts with the scenario's pack (docs/dev/character-creation.md)
- `--content-dir assets` — read content from disk instead of the copy built into the binary
- `--window WxH` (physical pixels), `--scale F` (display scale; `1` for reproducible captures)
- `--capture out.png` — render **offscreen** (no window), save a screenshot, exit.
  `--capture-after-frames N` (default 10), `--freeze-time SECONDS` pins animation time (glyph
  animations show that moment; slides and camera moves complete instantly).
- `--reduced-effects` — force reduced effects on (no glyph animation, light flicker, slides,
  camera easing, particles, bloom/glow, heat haze, fades, screen shake, or falling rain; a spell
  effect is a still flash); always wins over the settings
  (`Settings::effective_effects(force)`). With `--freeze-time`, particle bursts from the last turn's events (a `--replay`'s last step)
  show a fixed moment of their life; fades complete instantly.
- Without `--scenario`, `--new`, `--replay`, `--record`, or `--capture` the game opens on the main
  menu. Dev: `--screen NAME` opens a screen at startup, even with `--capture`: `main`,
  `new-run`, and `characters` (the saved characters; a capture reads them, never writes, from
  `--save-dir`, e.g. `tests/golden/saved-characters`) on the main menu (e.g. `--scale 1 --screen
  new-run --capture target/menu.png`), and character creation's `race`, `gender`, `class`,
  `name`, and `summary` over New run and the steps before them (each on its preselection: the
  showcase's character, an empty name);
  `pause`, `help`, `inventory`, `equipment`, `character`, `weapon-skills`, `quests`, `messages` (the message history), `pick-up` (the pile on the player's cell), `drop`, `eat`, `read`, `identify` (for the first stack's scroll), and `abilities` (the abilities menu) over a run
  (e.g. after a `--replay` that walks onto a pile), `dialogue` over a run whose last action was
  talk with topics (a `--replay` ending in `chat:SE` or `ask:SE:rooms` next to the hub's hermit),
  `equip-item` (the choice for the first slot, Head) over the equipment screen; `settings`,
  `controls`, and `dialog` (the quit dialog) over the main menu, or over the pause menu when
  `--scenario`, `--new`, `--replay`, or `--record` names a run (`--capture` alone does not).
  `--config-dir DIR` keeps `settings.toml` in DIR; captures and replays otherwise use default
  settings and never write the file.

- `--fire` (dev) — start the run aiming the readied missile weapon, as `f` (unbound by default) would (captures of the
  targeting line; golden entries pass it with `args = ["--fire"]`). `--look X,Y` (dev) — start
  in look mode with the cursor on level cell (X, Y) (captures of what look mode says, e.g. a
  shop's price). `--keys "KEYS"` (dev) — press keys at startup, after any `--replay`, as if typed
  (single characters, or `Enter`, `Esc`, `Tab`, `Up`, `Down`, `Left`, `Right`), so captures show
  what they open: `--keys h` at a shop's door owing (the pay dialog), `--keys "d b Enter"`
  (a sell offer).
- `--replay FILE` — apply a replay at startup (its scenario/seed/character win; a hash divergence
  is logged as a warning), so `--capture` can screenshot any state. `--record FILE` — write every applied
  action (with state hashes) to a replay, rewritten after each action.

Headless dev loop (`warlock-cli`, no window; `--content-dir` works here too):

| Task | Command |
|---|---|
| ASCII dump after actions | `cargo run -p warlock-cli -- dump --scenario showcase --actions "move:N move:N" --layers view,fov,memory` (`--character dwarf,female,warrior` starts as that character and `--name Brakka` names them, here and in `replay --create`) |
| Dump a replay's end state | `... dump --replay runs/x.json [--actions "wait"] --format ascii\|json [--state]` |
| Create a replay | `... replay --create --scenario showcase --actions "move:N wait" -o runs/x.json` |
| Verify a replay | `... replay --verify runs/x.json` (exit 1 and first diverging step on mismatch) |
| Preview a generated level | `... gen --level depths:3 --seed warlock` (ground truth; `*` = light placed on floor, `&` = locked door, `S` = secret door; `^` = a trap, hidden ones too; a special room's floor by its effect's letter; also local areas, `--level local:49,23`) |

Dump layers beyond the map ones: `inventory`, `equipment`, `stats`, `skills` (the player's
trained skills, `Stealth 17`), `abilities` (the abilities menu's list), and `quests` (the quest
log, active first) from perception;
`actors` (dev-only: every actor's hit points, load, stats, and skills with their marks).

Action text syntax: whitespace-separated `move:DIR wait open:DIR close:DIR kick:DIR pick:DIR ascend descend lantern
pickup[:LETTERS] drop[:LETTERS] sell[:LETTERS] pay[:DIR] equip:SLOT:LETTER remove:SLOT bash:DIR fire:X,Y throw:LETTER:X,Y
eat:LETTER eat:floor:LETTER read:LETTER[:X,Y|:LETTER] dig:DIR search search:N disarm:DIR second_wind chat:DIR ask:DIR:TOPIC rest:GOAL stance:NAME vision:PROFILE
talent:ID[:CHOICE] advance:SKILL levelup:auto xp:N
skip:DURATION weather:KIND effect:ID@X,Y[>X,Y] identify[:LETTERS] afflict:STATUS infect:DISEASE`
(`DIR` = N
NE E SE S SW W NW; `NAME` = aggressive, neutral, defensive, or coward (a stance, no time; `assets/data/stances.toml`); `LETTERS` = stacks by position, as letters
for the first 26 (`a` = 0, the first) or numbers for any (`0,2,30`; mixed works too, `ac,30`),
e.g. `pickup:ac` (the pile on the player's cell, `a` its top) or `drop:b` (the inventory), none
= everything; `LETTER` = one position, `b` or `27` (`equip:body:27`, `eat:floor:30`); positions
are where stacks are stored, never the letters a screen's page shows; `sell:b` sells to
the shop the player stands in and `pay` pays it (spec §7 "Shops"); `SLOT` = a slot id in
`assets/data/slots.toml`, e.g. `equip:body:b` (put inventory stack `b` on), `remove:body`;
`pick:DIR` picks the lock of the door that way (the Pick lock ability: Open lock trained;
`docs/dev/doors.md`), `bash:DIR` bashes with the held shield, `fire:X,Y` shoots the readied missile weapon at level
cell (X, Y), `throw:c:X,Y` throws one of
inventory stack `c` there (any weapon is thrown whole, badly: `t` in the game sends `fire` for a launcher, else `throw` for what the Missile weapon slot holds); `eat:b` eats one of inventory stack `b`, `eat:floor:a` one of the top
stack underfoot; `read:b:12,5` reads scroll stack `b` at level cell (12, 5) (a spell scroll:
fireball, lightning, frost), `read:b:c` reads it on inventory stack `c` (the scroll of
identify); `dig:E` digs into the wall east with the tool in the Tool slot, turn
after turn until it gives way or something interrupts; `search` searches the cells around the
player for secret doors and hidden traps, `search:N` for up to N turns (1 to 100), stopping when
something is found; `second_wind` uses the Second wind talent (hit points back at once, then Winded); `disarm:E` tries to disarm the trap the player knows of east of them (the
Disarm ability, for those trained in Disable device; refused, with no time, without the skill
or a known trap there); `chat:E` talks to the creature east of
the player, `ask:E:rooms` asks it about topic `rooms` (an id in `assets/data/dialogue.toml`;
quest topics are `quest-ID`, `accept-ID`, `decline-ID`, ids in `assets/data/quests.toml`);
`talent:hardy` takes a talent (an id in `assets/data/talents/`; `talent:weapon_focus:long_blades`
with its choice) when a level's pick is due (every third level; until it is made every other
action is refused), `advance:stealth` spends a level's skill roll (none yet, #230), `levelup:auto`
is the dev action that spends every choice due with the first the player may take, and `xp:2500`
the dev experience grant (no time; `xp:2431` brings a 100 % character to level 3);
`PROFILE` = an id in
`assets/data/vision.toml`; `GOAL` = `dawn`, `dusk`, or a duration up to `24h`; durations are
`30s`, `10m`, `2h`; `skip` is the dev hour skip; `weather` is the dev weather override, `KIND` =
`clear`, `overcast`, `rain`, `heavy`, for three hours of game time; `effect` is the dev spell
effect, `ID` an id in `assets/data/effects.toml`: `effect:fireball@30,14` (a burst there),
`effect:fireball@20,10>30,14` (flying there first), `effect:lightning_bolt@21,17>58,17` (a bolt
toward the second cell), no time and no rules effect, for captures; `identify` is the dev
identify (magic items, no time; without letters everything carried); `afflict` is the dev status
effect: one degree more of `STATUS`, an id in `assets/data/status.toml`, e.g.
`afflict:poisoned*4 afflict:bleeding`, no game time (not `diseased`: that comes of a disease);
`infect` is the dev disease: the player catches `DISEASE`, an id in `assets/data/diseases.toml`,
as if they had lost the check (it incubates, then shows), e.g. `infect:the_shakes rest:24h*3`,
no game time; `freeze:X,Y,R` and `thaw:X,Y,R` are the dev frost and heat bursts (radius 0 to 20
around level cell (X, Y), in its line of sight; no time); `ACTION*N` repeats one, e.g.
`wait*10`). Dump layers: `view` (what the player sees), `fov` (`*` visible),
`memory` (seen terrain, remembered objects, and known traps), `light` (perceived brightness of
visible cells as a digit, `0` dark to `9` fully lit), `sound` (heard sounds at their rough places
over the view: `f` footsteps, `o`/`c` a door opening/closing, `x` fighting, `b` a bowstring, `m`
chanting, `k` banging (a door kicked), `d` digging, `v` a voice, `a` an alarm bell, `e` an
explosion, `t` a thunderclap, upper case when loud), `blood` (blood the
player sees or remembers as a digit over the view, `1` a few drops to `9` soaked), `inventory`
(what the player
carries, grouped by item class under `Class:` header lines as the screens group it, each stack by
its position as action text names it, and weighs, worn and wielded things marked, their burden
and its thresholds),
`equipment` (the player's slots), `stats`
(the player's name, race, gender, and class, then their own scores as the rules read them, then
their alignment band, `Alignment Kind`, then
`Effects Bleeding, Diseased` when they suffer any, then `Diseases filth fever`, the diseases they
know they have, then what they resist in words, `Resistant to cold`, `Vulnerable to fire`,
`Immune to poison`), `weapon_skills` (the player's
ranks and progress toward the next, `Long blades 2 (40%)`, fighting styles too, those in use
marked `Duelist 1 (0%), in use`), `talents` (the player's talents in the order taken, `Hardy
2/3`, `Weapon focus (long blades)`), `abilities` (the player's abilities in the abilities menu's
order, each with its source and what it asks for or why it cannot be used now: `Shield bash
(weapon skill): a direction`, `Open lock (skill): not usable yet`), `traps` (the traps the player knows,
one a line), `quests` (the quest log, active
first: name, stage, giver, objective, and status), `truth`
(ground truth, hidden traps and secret doors (`S`) too, **dev-only**), `actors` (every actor on the level with its hit points and
Defence and Protection, its alignment (`alignment evil`; the player's score and band), its weight, what it carries and of that wears and wields (magic items as they truly
are, "(unidentified)" marked), what its magic items do to it (`magic strength +1`), its burden
and carrying limit, its weapon skills with marks, its stats as score/potential, its status
effects with severities, its diseases (progressions, checks won in a row, time to their next
turn) and stat damage, the player's nutrition, its npc id and talk counts, if any, and what it resists in percent (`resists fire 25`); then
all food on the level and in the packs of those on it with when it was made (a corpse: when it
died) and its stage, `fresh`, `old`, `rotten`, or `gone`, which the player is never shown,
**dev-only**), `progress` (the level-up choices due, `due level 3: 1 talent`, the talents the
player may take now by id with their choices, the first kills that paid, and the stat training
marks, `stat marks St 3, To 1`, **dev-only**),
`rooms`
(the level's special rooms: their plain ground as the effect's `mark`, then a line per room,
**dev-only**), `ice` (the level's temperature and the thickness of the ice on its water in cm,
`0`-`9`, `a`-`z` for 10-35, `#` beyond, **dev-only**). The header line has the player's `hp
17/17` (and `dead` once the run has ended), `hunger hungry` unless Not hungry, and the
`temperature`; JSON dumps add `hp`, `defence`, `protection`, `resistances` (by damage type id, the
word: `{"cold": "resistant"}`), `burden` (`level`, `limit`), `alignment` (the band's id), `hunger`, `effects`
(status ids), `diseases` (the known ones' ids), `death` (with `of`, the status that killed,
`cause`, starving, choking, or a trap, and `trap`, the id of the trap that killed),
`known_traps`, `temperature` (°C to a tenth), `weapon_skills` (by skill id:
`rank`, `progress`), `styles_in_use` (style ids), `talents` (in the order taken: `id`, `name`,
`rank`, `ranks`, `choice`), `due` (the level-up choices due: `level`, `talents`, or null), `abilities` (in menu order: `id`, `source`,
`aim`, and `unavailable`, the reason's id or null), `quests` (the log: `id`, `name`, `kind`, `active`, `completed`, `giver`,
`objective`, `status`), each seen actor's `health` and `effects` (those that show), `room` (the
special room the player stands in, if they know it), and per inventory stack (in position
order) its `letter` (the position as action text), `class` (item class id), `name`, `rarity`,
`identified`, and `magic` (affixes only once identified). Replay
files are JSON
`{game_version, scenario, seed, character, name, kit, steps:[{action, hash}]}` (`character` =
`{race, gender, class}`, left out for a run without one; a replay without it starts as the
scenario's, so old replays load; `name` the character's name, a replay without it gives the run's
default name; `kit` true when the run started with the character's starting kit, left out
otherwise); `hash` is the FNV-1a 64 state hash (`World::state_hash`) after
the step, as 16 hex digits.

- `--portable [DIR]` keep everything in `warlock-data` next to the game (or DIR); a `warlock-data`
  folder there does it without the flag. `--paths` prints the locations and why, then exits.
  `--move-data native|portable [DIR]` (`--dry-run` to preview) moves files between portable and
  the platform folders.
- `--reset-keys` puts every key back to its default (saved at once), then starts as usual.
  `--export-keys FILE` writes the keymap (every command's keys by context, the format of
  Controls' Export keys, `warlock-keys.toml`) to FILE; `--import-keys FILE` reads one into the
  settings (the commands it lists only) and says what changed; both then exit.
- `--new` start a fresh run (a new character, with a save of its own) instead of resuming the
  character played last; `--dev` enables the dev keys in release
  builds (always on in debug: F2 reveal map (ground truth), F3 next vision profile, F4 light-value
  overlay, F5/F9 snapshots, F6 skip an hour (recorded as `skip:1h`), F7 next weather (recorded as
  `weather:KIND`), F8 identify everything carried (recorded as `identify`); all listed by `?`),
  and New run's seed field (without it New run draws a random seed and never mentions it, #256);
  `--save-dir DIR` keeps saves in DIR instead of the platform data dir (`DIR/saves/<id>.sav`, one
  per character; with `--capture` or `--replay` only read).

Typical render check: `cargo run -p warlock -- --scale 1 --freeze-time 1 --capture target/shot.png`,
then read the PNG.

## Architecture (spec §8)

```
crates/warlock-core   plain Rust, no Bevy, deterministic simulation
crates/warlock-cli    headless dev tool on top of core (no Bevy)
crates/warlock        Bevy app: rendering, input, UI, effects, platform
xtask                 screenshot capture, golden-image comparison, vast level generator, benchmarks
assets                fonts, data definitions, levels, scenarios, vaults
```

Each subsystem has a topic file under `docs/dev/` with its modules, data, invariants, and
tests. **Read the ones your task touches before changing code, and update them with it**
(a feature PR adds or edits its own topic file; nothing about one subsystem goes in this
file). Topics:

- `docs/dev/content.md` — TOML content embedded by `build.rs`, level files and legends (faction overrides, arrival lines, water flow)
- `docs/dev/saves.md` — storage backends (native files, browser `localStorage`), one save per character and the list of them, autosave, permadeath, where files live (`paths.rs`, portable mode)
- `docs/dev/settings.md` — `settings.toml`, effect switches, clock, palettes, rebindable key commands (`settings/keys.rs`), the `Game` resource
- `docs/dev/screens.md` — the screen stack (open, input, draw, the dialog, info pages, the text field), prompts as input modes, the screen modules, key commands and the abilities menu
- `docs/dev/time.md` — `GameTime`, the calendar, action costs and prices, the energy scheduler, rest, creature speed
- `docs/dev/hearing.md` — noise per action, propagation, what creatures and the player hear, sound markers
- `docs/dev/objects.md` — object and material data, parts and instance parts, gems, stacks, piles, inventories, weight, pick up and drop
- `docs/dev/equipment.md` — slots as data, wielding and wearing, what combat reads from equipment
- `docs/dev/combat.md` — the 2d12 rules, Defence categories, strikes, hit points and death, messages
- `docs/dev/ranged-combat.md` — firing and throwing, missiles in flight, fires, the aiming mode
- `docs/dev/weapon-skills.md` — ranks, marks, benefits, training hooks, the styles' grips and rows
- `docs/dev/stats.md` — the six stats, modifiers, scores and potentials, what may be shown, stat training (marks by use, the level-up checks)
- `docs/dev/burden.md` — load, limits, sizes, every penalty's one hook
- `docs/dev/hunger.md` — satiation, eating, corpses and rot, starving
- `docs/dev/character-creation.md` — `CharacterSpec`, races, classes, genders, the horoscope, starting rolls, names, starting kits
- `docs/dev/alignment.md` — the good–evil score and its bands, the deeds that move it (one hook each), what reads it
- `docs/dev/skills.md` — trained skills, effects and uses, training marks, the skills screen
- `docs/dev/experience.md` — the level curve, kill XP, level-ups
- `docs/dev/progression.md` — talents (data, `TalentMods`, the hooks), the picks a level brings and their refusal, the `levels` stream, the coverage policy
- `docs/dev/status-effects.md` — statuses and severities, healing paces, checks, diseases and stat damage
- `docs/dev/blood.md` — wounds and stains, bleeding, how blood is drawn
- `docs/dev/creatures.md` — factions and stance, the AI, paths, creature sight
- `docs/dev/npcs.md` — unique characters, chat and ask, rotation lines and mutters, displacement
- `docs/dev/quests.md` — quest data and stages, hooks in kills, travel, and talk, the quest log
- `docs/dev/shops.md` — shop data, keepers, prices, buying, selling, theft, the shop dialogs
- `docs/dev/doors.md` — locks and keys, kicking, breaking, what creatures open
- `docs/dev/digging.md` — digging tools, wall hardness, the dig loop, blasts
- `docs/dev/traps.md` — trap data, triggering, known traps, the dev trap actions
- `docs/dev/searching.md` — secret doors, the active and passive checks, who notices, the secrets stream
- `docs/dev/magic-items.md` — affixes, rarities, base items, loot, identification, the item description
- `docs/dev/scrolls.md` — reading, spell scrolls on the traps' shared spell code, the scroll of identify, the read and identify lists
- `docs/dev/overworld.md` — the overworld level, locations, sight at map scale, generated local areas
- `docs/dev/mapgen.md` — RNG streams, dungeon branches, generators, vaults, snapshots, special rooms
- `docs/dev/rendering.md` — the three layers, the console shader, visual lighting and creature shadows, light stability, particles, glow, post, rain, spell effects, the `ui/` modules
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
- Every key the game reads goes through the binding system: a `KeyCommand` in its
  `input::Context`, rebindable in Controls and listed by `?`. Never match a `KeyCode` or a typed
  character in a screen or an input mode (only Esc and the letters and digits naming list rows
  are fixed); hints name keys as bound (`docs/dev/settings.md`).
- Abilities (racial, class, skill, weapon skill and style, item; #156) register in
  `warlock_core::ability`, the one registry, and are used from the one abilities menu (`a`):
  never a default key of their own (an ability's own `KeyCommand` stays unbound by default,
  `settings/keys.rs::UNBOUND`). Basic commands keep their keys (`docs/dev/screens.md`).
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
ages, hit points), classes (stat adjustments, hit points), what each gender raises, the horoscope
(animals, elements, and the great cycle's signs and years), combat (weapons, armour and shields,
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
`{{#include generated/NAME.md}}`; CI runs `cargo xtask wiki` and then `mdbook build wiki` on the
Linux job, and the release workflow does the same before publishing (mdBook pinned in both). To
preview: `cargo xtask wiki --build`, then open `wiki/book/index.html`. Tables sort by explicit
keys (class, level, name), never by content file order. New pages go into
`wiki/src/SUMMARY.md` by topic, never at the end; a new table is its own function in its
topic's module plus one line in `generate()` next to its topic.

**The game is vague, the wiki is accurate** (the owner, 2026-10-20): in the game, hidden numbers
stay hidden and effects are told in words (the perception rules above are unchanged), but the
wiki states the truth exactly. The game is open source, so the truth is one look at the sources
away, and the wiki is also where facts are inspected during development.

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
- Bevy features are trimmed in the workspace `Cargo.toml` (`default-features = false`). Adding a
  Bevy feature is a deliberate change. `tonemapping_luts` is deliberately off (it pulls `ruzstd`
  0.8, RUSTSEC-2026-0344); cameras use `Tonemapping::None`, which also keeps palette colors exact.

## Verification

A successful compile does not verify anything visual. Before opening a PR:
1. `cargo fmt`, clippy with `-D warnings`, `cargo test --workspace`.
2. Simulation changes: add or update unit, property, or snapshot tests in `warlock-core`
   (`insta` snapshots live in `crates/warlock-core/src/snapshots/`, `src/mapgen/snapshots/`, and
   `src/mapgen/local/snapshots/`; review with `cargo insta review` or `INSTA_UPDATE=always`, and read the diff). Look at the
   result with `warlock-cli dump` (or `gen` for a generated level), and check determinism with
   `replay --create` then `replay --verify`.
3. Rendering changes: capture a screenshot with `--capture` and **look at it** (read the PNG).
   Golden images are updated deliberately, never to make a failing check pass.
4. Note in the PR what you verified and how.

## Workflow

- Work is tracked in GitHub issues under milestones (M0–M12, Housekeeping); use the `gh` CLI.
- One branch and PR per issue (or a tight group of related issues); reference issues with
  `Closes #N`. CI must be green before a squash merge into `main`.
- CI runs on Linux only (`CI` workflow: fmt, clippy, tests, headless screenshots, wasm32 check,
  cargo-deny; the web build on pushes to `main`). macOS is not run in Actions: development happens
  on macOS, so run the full local check there before every PR (a PR built by a cloud agent,
  `ops/agents/`, runs it on Linux instead). `main` is branch-protected: PRs
  only, the Linux, wasm32, and cargo-deny checks must pass, linear history.
- **Merging** (`docs/dev/merging.md` has the details): agents don't chase `main`. A PR merges
  or rebases onto `origin/main` once, when the integrator says it is next in line; after that
  merge run `cargo fmt --all`, clippy with `-D warnings`, `cargo test --workspace`,
  `cargo xtask wiki`, and the PR's own captures or replays (CI covers the wasm32 check, the
  smoke test, `mdbook build wiki`, cargo-deny, and the licenses). The full check list
  ("Verification") runs once, before the PR opens.
  Agents stop when their PR is open; a finisher agent handles CI and the merge (`docs/dev/merging.md`,
  "Agents, finishers, and context").
- **Keep PRs small**: one feature per PR, rules first (core, tests, dumps), then the UI, then
  the content that uses it, when they can stand alone. The integrator schedules features by
  overlap: at most about three PRs at once that add actions or save state.
- **Hot files**: new entries in registries go where they belong (by group or alphabetically),
  never at the end; generated files are never hand-merged (take one side, regenerate);
  `FORMAT_VERSION` is bumped by the integrator once per release, never in a feature PR; a
  feature's dispatch lives in its own module with one line in each hot file; churning content
  tables are directories, one file per entity. All in the playbook.
- **Design documents** pending the owner's approval are draft PRs that add
  `docs/design/<topic>.md` and touch nothing else: the owner reviews them line by line in the PR,
  each revision is a commit, and merging the PR approves the design. CI skips the build for them.
  Implementation issues link to the merged document. Details in the playbook.

## Golden screenshots

`tests/golden/manifest/` is the manifest: `defaults.toml` holds `[defaults]`, and the other
`*.toml` files (one per showcase room or feature, read in file-name order; entry names are
unique across files) list one entry per showcase room plus the vast level (its start and
its river): a scenario and text `actions` (`move:N*3 open:E`; `*N` repeats) or a `replay` file,
and optional `character` (`human,male,paladin`: the run starts as it, not the scenario's), `kit`
(`true`: the run starts with the character's starting kit), `window`, `scale`, `freeze_time`, `screen` (a `--screen` value: over the run, e.g.
`pause`, `help`, `settings`, `controls`, `inventory`, `equipment`, `equip-item`, `describe`,
`character`, `skills`, `weapon-skills`, `quests`, `messages`, `pick-up`, `drop`, `eat`, `abilities`,
`dialogue`; or over
the main menu, the replay then unused: `main`, `characters`, `race`, `gender`, `name`, `summary`), `args` (more game flags, e.g.
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

References are produced on **Linux CI (Mesa lavapipe) only**; other GPUs and drivers rasterize
differently, so on macOS `cargo xtask golden` is advisory and `--local` is the way to look at the
captures. CI compares and never updates.

Deliberate update procedure (after an intended visual change, e.g. lighting or effects landing):
1. Push the branch, then Actions > CI > Run workflow on that branch with "update_golden" ticked
   (`gh workflow run ci.yml --ref <branch> -f update_golden=true`, then `gh run watch`).
2. `gh run download <run-id> -n golden-references -D tests/golden`, review the images (look at them,
   and check the diff against the old ones makes sense), and commit them with the change.
3. Adding a room, scenario, or screen: add an entry to the feature's manifest file (or a new
   file) and generate its reference the same way, in the same PR (CI fails on an entry without a
   reference). Never update references just to make a failing check pass.

## Releases, web build, logs

**Cut a release.** Make sure `main` is green, then tag and push the tag (for example
`v0.1.0`); the `Release` workflow does the rest. It builds Linux x86_64 (`.tar.gz`), Windows
x86_64 (`.zip`), macOS arm64 (`.zip` with an unsigned `Warlock.app` plus the bare binary; see
`release/`) and the web build (`.zip` to serve from any static host) and attaches them, with
generated notes, to a GitHub Release; `v0.x` tags are published as pre-releases. Run the workflow
by hand (Actions > Release > Run workflow) to get the archives as workflow artifacts without
publishing. Releases are the only place macOS and Windows runners are used.

**Web build** (wasm32 + WebGPU; all content is embedded, nothing on the startup path may touch
`std::fs`): one-time setup is `rustup target add wasm32-unknown-unknown` and
`cargo install wasm-bindgen-cli --version <X> --locked`, where `<X>` is the `wasm-bindgen` version
in `Cargo.lock` (the CLI must match exactly; `cargo xtask web` checks it and prints the command).
Then `cargo xtask web` builds `target/web/` with the size-tuned `[profile.web]` (opt-level `s`, fat LTO,
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
it works under `/warlock/`): top bar with links, the canvas in `#stage` (Bevy's
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

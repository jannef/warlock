# CLAUDE.md — working on Warlock

Warlock is a GPU-rendered ASCII roguelike in Rust + Bevy. **`specification.md` is the source of
truth** for what to build; this file is about *how* to work on it. Update both together whenever
behavior, commands, or architecture change.

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
| Web smoke test | `cargo xtask web-smoke <chrome>` (serves `target/web`, drives headless Chromium through menu → New run → race, gender, class (preselected) → summary → hub, waits five turns, reloads, Continue, and checks the run resumed at turn 5; writes `target/xtask/web-smoke.png`; fails on logged errors; not run in CI) |
| Regenerate the vast level | `cargo xtask gen-vast` (writes `assets/levels/vast.toml`; deterministic, never hand-edit) |
| Check the vast level is fresh | `cargo xtask gen-vast --check` (fails if the file differs from the generator output; run it after touching `xtask/src/vast.rs`) |
| Third-party notices | `cargo xtask licenses` (regenerates `THIRD_PARTY_LICENSES.html` with `cargo-about`, config `about.toml`/`about.hbs`; install: `cargo install cargo-about --locked --features cli`); `--check` fails when it is stale (CI runs it). **Run it after changing dependencies** (`Cargo.toml`/`Cargo.lock`); keep `about.toml`'s `accepted` list in sync with `deny.toml`'s `allow` |
| Player wiki tables | `cargo xtask wiki` (writes `wiki/src/generated/*.md` from the game data), `--check` (fails when stale), `--build` (also runs `mdbook build wiki` into `wiki/book/`; needs `cargo install mdbook --locked`) |
| Golden screenshots | `cargo xtask golden` (compare; authoritative on Linux only), `--local` (capture to `target/xtask/golden/*.local.png`, no compare), `--update` (rewrite references; see "Golden screenshots") |
| Frame-time benchmark | `cargo xtask bench-frames [WxH]` (release, offscreen; method and results in `docs/perf.md`) |
| Turn-time benchmark | `cargo xtask bench-turns` (re-runs itself in release; budgets: hub 5 ms, vast 16 ms, generated depths level 5 ms, sunlit courtyard at dawn 5 ms, overworld 5 ms, generated forest area at dawn 5 ms, courtyard and forest area at full-moon midnight 5 ms, arena 5 ms per turn; a dusk-to-dawn rest 500 ms, also beside the arena's fight going on unseen until one side is dead (a walk that gets the player killed starts over, untimed), and rested on to dawn when rain wakes the player) |
| Light stress benchmark | `cargo test --release -p warlock-core turn_cost -- --ignored --nocapture` (400×300, 825 lights; again outdoors at 06:15) |

Game flags (`cargo run -p warlock -- --help`):
- `--scenario NAME` (default `showcase`; `depths` starts in the generated dungeon, `courtyard` in
  the sunlit courtyard at 06:30, `world` on the overworld beside the Hall, `forest` in the glade
  of a generated forest area of the overworld at 06:30, `arena` in the dark corner of the arena
  where kobolds and giant rats fight, `quarry` in the quarry beside a pick-axe), `--seed S`
  (default: the scenario's seed; a number, or a word hashed with FNV-1a, e.g. `--seed warlock`),
  `--character RACE,GENDER,CLASS` (e.g. `dwarf,female,warrior`; default: the scenario's
  character; like `--seed` it shapes the run other flags start and alone does not skip the menu)
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
  menu. Dev: `--screen NAME` opens a screen at startup, even with `--capture`: `main` and
  `new-run` on the main menu (e.g. `--scale 1 --screen new-run --capture target/menu.png`), and
  character creation's `race`, `gender`, `class`, and `summary` over New run and the steps
  before them (each on its preselection: the showcase's character);
  `pause`, `help`, `inventory`, `equipment`, `character`, `weapon-skills`, `pick-up` (the pile on the player's cell), and `drop` over a run
  (e.g. after a `--replay` that walks onto a pile), `equip-item` (the choice for the first slot,
  Head) over the equipment screen; `settings` and `controls` over the main menu, or over the pause
  menu when `--scenario`, `--new`, `--replay`, or `--record` names a run (`--capture` alone does
  not).
  `--config-dir DIR` keeps `settings.toml` in DIR; captures and replays otherwise use default
  settings and never write the file.

- `--fire` (dev) — start the run aiming the readied missile weapon, as `f` (unbound by default) would (captures of the
  targeting line; golden entries pass it with `args = ["--fire"]`).
- `--replay FILE` — apply a replay at startup (its scenario/seed/character win; a hash divergence
  is logged as a warning), so `--capture` can screenshot any state. `--record FILE` — write every applied
  action (with state hashes) to a replay, rewritten after each action.

Headless dev loop (`warlock-cli`, no window; `--content-dir` works here too):

| Task | Command |
|---|---|
| ASCII dump after actions | `cargo run -p warlock-cli -- dump --scenario showcase --actions "move:N move:N" --layers view,fov,memory` (`--character dwarf,female,warrior` starts as that character, here and in `replay --create`) |
| Dump a replay's end state | `... dump --replay runs/x.json [--actions "wait"] --format ascii\|json [--state]` |
| Create a replay | `... replay --create --scenario showcase --actions "move:N wait" -o runs/x.json` |
| Verify a replay | `... replay --verify runs/x.json` (exit 1 and first diverging step on mismatch) |
| Preview a generated level | `... gen --level depths:3 --seed warlock` (ground truth; `*` = light placed on floor, `&` = locked door; a special room's floor by its effect's letter; also local areas, `--level local:49,23`) |

Dump layers beyond the map ones: `inventory`, `equipment`, `stats`, and `skills` (the player's
trained skills, `Stealth 17`) from perception; `actors` (dev-only: every actor's hit points,
load, stats, and skills with their marks).

Action text syntax: whitespace-separated `move:DIR wait open:DIR close:DIR kick:DIR ascend descend lantern
pickup[:LETTERS] drop[:LETTERS] equip:SLOT:LETTER remove:SLOT bash:DIR fire:X,Y throw:LETTER:X,Y
dig:DIR rest:GOAL stance:NAME vision:PROFILE skip:DURATION weather:KIND effect:ID@X,Y[>X,Y]
afflict:STATUS` (`DIR` = N
NE E SE S SW W NW; `NAME` = aggressive, neutral, defensive, or coward (a stance, no time; `assets/data/stances.toml`); `LETTERS` = stacks by letter, `a` first, e.g. `pickup:ac` (the pile on the
player's cell, `a` its top) or `drop:b` (the inventory), none = everything; `SLOT` = a slot id in
`assets/data/slots.toml`, e.g. `equip:body:b` (put inventory stack `b` on), `remove:body`;
`bash:DIR` bashes with the held shield, `fire:X,Y` shoots the readied missile weapon at level
cell (X, Y), `throw:c:X,Y` throws one of
inventory stack `c` there (any weapon is thrown whole, badly: `t` in the game sends `fire` for a launcher, else `throw` for what the Missile weapon slot holds); `dig:E` digs into the wall east with the tool in the Tool slot, turn
after turn until it gives way or something interrupts; `PROFILE` = an id in
`assets/data/vision.toml`; `GOAL` = `dawn`, `dusk`, or a duration up to `24h`; durations are
`30s`, `10m`, `2h`; `skip` is the dev hour skip; `weather` is the dev weather override, `KIND` =
`clear`, `overcast`, `rain`, `heavy`, for three hours of game time; `effect` is the dev spell
effect, `ID` an id in `assets/data/effects.toml`: `effect:fireball@30,14` (a burst there),
`effect:fireball@20,10>30,14` (flying there first), `effect:lightning_bolt@21,17>58,17` (a bolt
toward the second cell), no time and no rules effect, for captures; `afflict` is the dev status
effect: one degree more of `STATUS`, an id in `assets/data/status.toml`, e.g.
`afflict:poisoned*4 afflict:bleeding`, no game time; `ACTION*N` repeats one, e.g.
`wait*10`). Dump layers: `view` (what the player sees), `fov` (`*` visible),
`memory` (seen terrain and remembered objects), `light` (perceived brightness of visible cells as a
digit, `0` dark to `9` fully lit), `sound` (heard sounds at their rough places over the view: `f`
footsteps, `o`/`c` a door opening/closing, `x` fighting, `b` a bowstring, `m` chanting, `k`
banging (a door kicked), `d` digging, upper case when loud), `blood` (blood the player sees or
remembers as a digit over the view, `1` a few drops to `9` soaked), `inventory` (what the player
carries and weighs, worn and wielded things marked, their burden and its thresholds),
`equipment` (the player's slots), `stats`
(the player's race, gender, and class, then their own scores as the rules read them, then
`Effects Bleeding, Poisoned` when they suffer any), `weapon_skills` (the player's
ranks and progress toward the next, `Long blades 2 (40%)`), `truth`
(ground truth, **dev-only**), `actors` (every actor on the level with its hit points and
Defence, its weight, what it carries and of that wears and wields, its burden and carrying
limit, its weapon skills with marks, its stats as score/potential, and its status effects with
severities, **dev-only**), `rooms` (the level's special rooms: their plain ground as the
effect's `mark`, then a line per room, **dev-only**). The header line has the player's `hp
17/17` (and `dead` once the run has ended); JSON dumps add `hp`, `defence`, `burden` (`level`,
`limit`), `effects` (status ids), `death` (with `of`, the status that killed), `weapon_skills`
(by skill id: `rank`, `progress`), each seen actor's `health` and `effects` (those that show),
and `room` (the special room the player stands in, if they know it). Replay
files are JSON
`{game_version, scenario, seed, character, steps:[{action, hash}]}` (`character` =
`{race, gender, class}`, left out for a run without one; a replay without it starts as the
scenario's, so old replays load); `hash` is the FNV-1a 64 state hash (`World::state_hash`) after
the step, as 16 hex digits.

- `--portable [DIR]` keep everything in `warlock-data` next to the game (or DIR); a `warlock-data`
  folder there does it without the flag. `--paths` prints the locations and why, then exits.
  `--move-data native|portable [DIR]` (`--dry-run` to preview) moves files between portable and
  the platform folders.
- `--new` start a fresh run instead of resuming the save; `--dev` enables the dev keys in release
  builds (always on in debug: F2 reveal map (ground truth), F3 next vision profile, F4 light-value
  overlay, F5/F9 snapshots, F6 skip an hour (recorded as `skip:1h`), F7 next weather (recorded as
  `weather:KIND`); all listed by `?`);
  `--save-dir DIR` keeps saves in DIR instead of the platform data dir.

Storage backends (`crates/warlock/src/storage.rs`): `Backend::Dir` (native files, atomic writes) or
`Backend::Browser` (wasm `localStorage`, keys `warlock/<name>`, e.g. `warlock/run.sav`,
`warlock/run.sav.bad`, `warlock/snapshots/quick.sav`, `warlock/settings.toml`; bytes base64).
`SaveStore` and `SettingsStore` hold an `Option<Backend>`; new persistent files go through it, never
`std::fs` directly. On the web the page-hide save cannot ask the world for a save (a hidden tab runs
no frames), so `save.rs::web_hooks` keeps a clone of the state after every turn (encoding is
milliseconds on the vast level) and the `visibilitychange`/`pagehide` listeners encode and write it. Unavailable storage means no saving plus a log
warning; failed writes (quota) are messages, never panics. Only the `web-sys` features in
`crates/warlock/Cargo.toml` are enabled (size).

Saves: Quit in the pause menu saves and quits (closing the window saves too; the `Q` key is unbound by default, rebindable); the game autosaves every 50 turns and on
level change to `run.sav` in the data dir (see "File locations" below). On startup an existing save is resumed unless `--new`, `--capture`,
`--replay`, or `--record` (replays and recordings always start a fresh run). F5 / F9 write / load `snapshots/quick.sav`. An unreadable save is renamed `run.sav.bad`,
never deleted. Format and `Content::fingerprint` live in `warlock-core/src/save.rs`: bump
`FORMAT_VERSION` on any incompatible `WorldState` change; adding or reordering terrain, creatures,
lights, levels, objects, equipment slots, skills, weapon skills, statuses, or dungeon branches invalidates
existing saves.
**Permadeath**: when the player dies, `screens::on_death` deletes `run.sav`
(`SaveStore::delete_run`) and opens the death screen; `SaveStore::save_run` never writes a dead
run (it deletes instead), so autosave, quitting, and closing the window cannot bring it back, and the
web build parks no state for the page being hidden.

**File locations** (`crates/warlock/src/paths.rs`, the one place that decides; never call
`ProjectDirs` elsewhere): `paths::init(&args, check_only)` runs first in `main`, before logging,
and `paths::locations()` is what stores, logging, and the Files screen ask (it carries each
folder's `Reason`, shown by `--paths`). Precedence per folder: `--config-dir` / `--save-dir`, then
`--portable [DIR]`, then a `warlock-data` folder next to the game (`paths::game_folder`: the
executable's folder; on macOS next to the `.app`, never inside it), then the platform dirs
(`directories`, app name "warlock"; macOS `~/Library/Application Support/warlock`). Portable
layout `warlock-data/{config,data,logs}`; `Locations::prepare` creates it and warns (never
fails) when it is unwritable. `resolve` is pure over `Inputs` and tested without a filesystem; new
files go in the folder `Locations` gives (`keymaps_dir()` for key map files). `warlock --paths`
prints the report and exits; `warlock --move-data native|portable [DIR] [--dry-run]`
(`move_data.rs`: copy, compare, rename, remove; `*.bak-<time>` for conflicts; refuses while a game
holds the shared lock on `logs/.in-use`) moves files between the two layouts. Settings > Files
(`ui/files_screen.rs`, `ScreenName::Files`, `--screen files`) shows the report with Open folder.

Settings (`crates/warlock/src/settings/`): `settings.toml` in the config dir, schema in the `settings/mod.rs` module docs. Changes
apply live and are written at once; a bad file means defaults plus a warning, per value. Effects:
`reduced_effects` is a preset over the `[effects]` switches (`EffectSwitches`: glyph_animation,
flicker, motion, glow, distortion, particles, screen_fades, screen_shake, weather; all on by default), all but
`blood` (on by default too), a content preference the preset leaves as set
(`EffectSwitches::effective`; its settings row is not indented and never says "(preset)"). Each consumer reads
its own switch from the `render::Effects` resource (the effective values, synced every frame from
`Settings` + `--reduced-effects`), never the preset; a new effect needs a switch in
`EffectSwitches`, the file, and a row in `ui/settings_screen.rs`. `clock` (`settings/clock.rs`, `"24h"`/`"12h"`) is the one way a time of day is written
in the UI (`Clock::format`: status bar, death screen); dumps stay 24-hour. Palettes
(`settings/palette.rs`) are applied only when cells are uploaded to the GPU (`render::upload`) and
in `ui/glyphs.rs`, never to content colors. Rebindable commands have stable ids in
`settings/keys.rs` (`KeyCommand::id`): **a new `KeyCommand` variant must be added there**
(`REBINDABLE` or the non-rebindable arm). `Game` exists only during a run (the menu runs without
it): gate systems that read it with `run_if(resource_exists::<Game>)`; content is the
`GameContent` resource.

Modal screens (spec §6) — main menu, new run and character creation (race, gender, class,
summary), pause, settings, controls, the `?` help, the inventory (`i`), the equipment screen
(`e`) and its choice of what goes in a slot, the character screen (`@`), the weapon skills
screen (`W`), choosing what to
pick up (`,`/`g` on several stacks) or drop (`d`), and the death screen (opened by
`screens::on_death` when `World::death` is set; Enter/Esc is `MenuChoice::LeaveRun`, back to the
main menu without saving; no `--screen`: capture a dead replay instead) — are one
screen stack (`ui/screens.rs`, the `ScreenStack` resource); add a screen there, never a second way
to open, read keys for, or draw one:
- **Open**: `Screens::open(ScreenName)` (a `SystemParam` over the stack, content, save store, and
  the run, if any) is the one way a screen opens: `?`, `i`, `e`, `@`, `W`, `,`, `d`, and Esc in play,
  sub-screens, the startup main menu and `--screen`, and the return to the main menu after a run.
  Screens that show the run read the perception snapshot when they open (no time passes while
  they are open but for an action chosen on one). Back/Esc pops the top screen, so a sub-screen
  returns to the screen it was opened from as it was left; `ScreenAction::Return(name)` pops
  until `name` is on top (the summary going back to a step). Sub-screens read what was chosen
  below them off the stack: character creation's steps (`ScreenStack::draft`: New run's scenario
  and seed, each step's selection) preselect what was chosen before going back
  (`ScreenStack::picks`), else the scenario's character, and the summary's Start is
  `MenuChoice::NewRun { scenario, seed, character }`; `EquipItem` takes the slot selected on the
  equipment screen beneath (`ScreenStack::selected_slot`).
- **Input**: `input::route_keys` is the one reader of `KeyboardInput`. Stack not empty: the press
  goes to the top screen (`ScreenStack::handle` → `ScreenAction`: stay, close, open, a
  `MenuChoice`, or `Act(Action)`: close and apply a gameplay action, e.g. `pickup:ac`,
  `equip:body:b`; the screens beneath stay, and `screens::refresh`, after `SimSet`, shows the
  equipment screen the result, as in ADOM); otherwise
  it is a gameplay command in the current `InputMode`. Opens and closes apply at once, so the next
  press in the same frame goes to the new top; menu choices (start, resume, save-and-quit to menu,
  quit) are messages applied by `screens::apply_choice`, and the rest of that frame's presses are
  dropped. Held-key repeats never open, close, choose, or act.
- **Draw**: `ui::draw` is the one system that writes the UI console: when anything it shows
  changed, it clears the console, draws the gameplay UI if a run exists, then the top screen.
- **Confirm** (`ui/confirm_screen.rs`): a reusable Yes/No dialog, No selected. A screen (or gameplay)
  answers `ScreenAction::Confirm(question, MenuChoice)`; the dialog pops itself on Yes and the
  choice is applied as usual, `n`/Esc/Enter-on-No close it. Every quit path (pause Quit and Save and
  quit to menu, main menu Quit, a bound `KeyCommand::Quit`) goes through it. `y`/`n` are fixed (they
  are movement keys in play), like the rest prompt's letters. `--screen confirm` shows it. On
  wasm, quit saves then reloads the page (`storage::web::reload`; `AppExit` also reloads), and the
  main menu has no Quit.
- **Info pages** (`ui/info_tabs.rs`): the screens about the player (Character, Inventory,
  Equipment, Skills) are an ordered group, `PAGES`. While one is the top screen, the page keys replace
  it (`ScreenAction::Replace` -> `Screens::replace`: the stack depth is unchanged, so Esc still
  returns to where the first was opened) and each draws its tab line on the bottom border with
  `info_tabs::draw_tabs`. **A new info screen joins the group**: one entry in `PAGES` (its
  position is its number) and one `draw_tabs` call in its `draw`; never a separate way to switch.
  Sub-steps of a page (`EquipItem`, pick-up/drop/throw lists) are not pages and keep their
  number keys. The keys are the `Context::Pages` commands `Page(1..9)`, `NextPage` (Shift
  reverses), `PrevPage`: rebindable like all commands (`KeyCommand::context`: keys are unique
  per context, so Tab is `NextTarget` in play and `NextPage` here; `Bindings::owner_in` and
  `lookup` read one context). `Press::shift` carries Shift from the router.
- Prompts in the message line (open/close/bash direction, rest, look mode) are `InputMode`s,
  not screens.
- Screens are plain state machines tested without a window: `ui/menu.rs` (`Press`, `Panel`, main
  and pause menu), `ui/new_run_screen.rs`, `ui/settings_screen.rs`, `ui/controls_screen.rs` (also
  the `?` help: the same binding list, read-only), `ui/object_screen.rs` (inventory, pick up,
  drop: stacks by letter; in the choices letters always name stacks, even `h j k l`),
  `ui/equipment_screen.rs` (`EquipmentScreen`: slots by letter, which always name slots, so its
  own key `e` closes it only when it names none; `ItemScreen`: the stacks that fit one slot, by
  their inventory letters), `ui/character_screen.rs` (name, race, gender, class, age, birthday,
  and sign (`birth_parts`/`fit_parts`, shared with the summary), the player's scores, hit points
  and Defence total, weight and burden thresholds; 72 wide like the summary),
  `ui/weapon_skills_screen.rs` (sections, ranks, and a faint ten-cell marks bar; never a
  benefit), `ui/chargen_screen.rs` (character creation: `ChoiceScreen` per
  step with a Random row rolled from the seed, `SummaryScreen`, `Draft`), `ui/death_screen.rs`
  (who killed the player, where, when, after how many turns).

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

Content (`assets/data`, `assets/levels`, `assets/scenarios`, `assets/vaults`) is TOML, embedded into
`warlock-core` at build time by `build.rs`. Level maps are ASCII with a legend; see
`assets/levels/hub.toml` and spec §7. A legend entry with a `creature` may set `faction = "peaceful"` (any id in
`data/factions.toml`) to override its type's faction for that spawn (`Actor::faction_override`; read it with
`Actor::faction`, never the creature type's, when computing stance). The showcase (hub, vast, courtyard) is safe
this way: `xtask/src/vast.rs` emits it for its rats and kobolds; fighting is the arena's and the Depths'. A level
file's optional `arrival = "..."` line is said when the player arrives by stairs. Water: `~` is still water; running water (terrain with a
`flow` anim) needs a flow on every cell from its legend entry (`flow = "S"`, optional
`flow_speed` percent), and the shared legend draws it as arrows (`→` normal, `⇒` double speed).
Flow is presentation data carried by `LevelTemplate::flow` / `LevelState::flow` and read by the
renderer through `PerceptionView::flow` (visible cells only); no rule reads it.

Experience (spec §8 "Experience", M21): `content/experience.rs` holds the rules from
`data/experience.toml` (level curve, `hp_level`, the WoW-style `kill_xp` scaling with its zero
difference bands), `world/experience.rs` applies them (`earn_kill` from `fight.rs::dies` when the
player lands the killing blow, or caused the status effect that killed: `Effect::by`; `grant_xp`,
level-up messages queued in `World::news` and said after the attack's own, or at the end of
`perceive` for a creature that bled to death). Creatures carry `level` and `xp` in `creatures.toml`. The saved state is
only `WorldState::experience` (the level follows from it); the UI reads
`PerceptionView::own_experience()`. Other kills (traps, creatures) give nothing: keep it that way
by awarding only where the attacker is the player.

Time (spec §8 "Time") lives in `warlock-core`: `time.rs` (`GameTime` = seconds since the
calendar epoch with calendar fields, minute of day, and 16-bit day fraction; `Duration`; 12×30-day
calendar math; `DAWN`/`DUSK`), `content/calendar.rs` (month names from `data/calendar.toml`),
`content/action.rs` (action costs, below), and the turn pipeline in `world/mod.rs`.
**Action costs** (spec §8 "Time", M13): every action that takes time has a cost in energy
(`ActionKind`, costs in `data/actions.toml`: 1000 = a normal action; a creature type may give its
attacks their own `attack_cost`; `Content::action_cost`) and a base time, `BASE_TURN` (6 s) for
everything but an overworld step, whose base is the terrain's `travel_minutes`. Its energy price
is `content::price(cost, base)` = cost × base seconds; an actor gains its speed in energy per
second, so it takes `cost / 1000 × 1000 / speed × base` (1400 at speed 1000: 8.4 s; 1000 at 1111:
about 5.4 s). `TerrainDef::step_price` (asked through `LevelState::step_price`) is the one place
movement time is decided: `move_cost` percent on local levels, `travel_minutes` on the overworld.
`player_acts` returns `Spent::Turn(price)` (`World::price_of(actor, kind)`, `World::step_price`),
`run_turn` (the energy scheduler, `world/schedule.rs`, spec §8 "Speed") charges the player for it
and runs the other actors on the level in time order until the player may act again, advancing
the clock (whole seconds; leftover energy carries, so a run of actions takes its exact total time,
up to the second the actor is ready, with no drift) to each action, and `rest` loops waits until
the goal or an interruption, its last turn cut short with `price_until` so it ends exactly at the
goal at any player speed. Long actions (`rest`, `dig`) go through `World::keep_at(events, more,
act)`: turn after turn of `act` while `more` holds, stopping on a creature coming into view, a
message (`say`; `note` does not stop it), or death (`Ended`), perceiving only after turns in which
a creature could have come into view (`out_of_view_for`, which accounts for speed and the cheapest
step) or in which the weather the player would see changed (`weather_turned`). A new action that
takes time needs an `ActionKind` (with its id and a line in `data/actions.toml`) and must return
its price there,
asked through `price_of`/`step_price`/`price_for`: those apply the actor's burden (below,
`World::burdened`), so a price taken anywhere else would escape it (`price_until`, the rest's
last turn, deliberately does). Price an action that changes the load before it does (pick up,
drop).
`creature_turn` is one creature's action and returns its price: the one place creature behavior
lives (AI decisions, the noise and events an action makes; the AI is `world/ai.rs`, below).
Speed (`speed` in `data/creatures.toml`, default 1000 = the player's; any player speed works)
is rules-only: never put it in perception, the UI, or messages; energy is per actor in
`WorldState` (saved, hashed); creatures on other levels and the dev skip gain none.
Levels carry `kind` (no turn length of their own); terrain carries `move_cost` (percent); scenarios
carry `start_date`/`start_time`. Rest (`R`) is a message-line input mode (`InputMode::Rest`,
`input::REST_CHOICES`), not a menu screen.

Hearing (spec §5 "Hearing") lives in `warlock-core/src/noise.rs`. `noise::noise(content, level,
actor, burden, Act)` is the one place an action's loudness is decided (creature `footsteps` ×
terrain `step_noise` percent × the burden's `footsteps()` percent for `Act::Step`, the door's
`toggle_noise` for `Act::OpenDoor`/`CloseDoor`; `make_noise` passes the actor's
`Load::penalty`):
a new noisy action gets an `Act` variant there, and its call site in `world/mod.rs` is one
`self.make_noise(actor, Act::…)` where the action happens. `noise::make` delivers it at once to
every other actor on the level whose `hearing` threshold it reaches (`propagate`: integer Dijkstra,
10 a step, 14 diagonally, plus terrain `muffle`; solid terrain blocks unless it has a muffle),
pushing a `Heard { kind, at, loudness }` onto `Actor::heard`. A creature's list is what it heard
since it last acted and is cleared as it acts in `World::creature_turn` (the AI reads it there
before clearing); the player's is cleared in `World::run_turn` once the player has acted, and
becomes `PerceptionView::sounds()` (sources not visible only; kind, `Volume`, and `rough_place`,
never the actor; a walker's several steps within one turn are one sound). `World::hear` logs news
(`noise::news` against `WorldState::recent_sounds`) as messages, which stop a rest. The app draws
`PerceptionView::sounds()` in `ui/sounds.rs` (a still `◌` marker, always: it is information; the
ripple only with `glyph_animation`). Sounds carry at most `noise::MAX_REACH` (30) cells, under
`VIEW_RADIUS`, which keeps rest batching (`out_of_view_for`) exact (a const assert guards it).
Objects (spec §5, §6, §7; M11): every object in `data/objects.toml` defines its glyph, color,
material (`data/materials.toml`), and weight in grams (`content/object.rs`: `ObjectDef`,
`ObjectId`, and the `"id*N"` placement strings; `content/material.rs`); every creature has a base
`weight`. `object.rs` holds the runtime side: `Stack` (object + count; stackable objects join,
others are stacks of one), piles (`add_to_pile`: dropped stacks go on top, index 0 is the top the
map shows), inventories (`add_to_inventory`, at most `INVENTORY_SLOTS` = 26, lettered by index),
`Weight` (base + carried), `format_weight`, and `phrase` for messages. Piles live in
`LevelState::objects` (by position, never empty) with `object_memory` beside them (piles as last
seen, updated by `Perception::remember` on cells where creatures would be seen); inventories in
`Actor::inventory` (spawned from a level's `carry`). The rules know every actor's weight
(`Actor::weight`, `World::actor_weight`); presentation reads only `PerceptionView::objects`
(current pile in sight, else remembered), `inventory`, and `own_weight`: never what creatures
carry. `Action::PickUp(Letters)` / `Action::Drop(Letters)` (`Letters` = a set of `a`–`z`, empty
= everything) are actions of their own cost (`pick_up`, `drop`); a refused one takes no time. The map grid draws the top object
in place of the terrain glyph (no terrain animation or glow there); look mode names piles.

Equipment (spec §6 "Equipment", §7; M16): slots are data, `data/slots.toml`
(`content/slot.rs`: `SlotDef` with `id`, `name`, `fits`, `action`, `mark`, `hand`; `SlotId` is an
index in screen order, in the content fingerprint), so **a new slot is one entry there** (and a
`FORMAT_VERSION` bump if saves change). Objects say where they go (`slot` = a slot's `fits`,
`two_handed`, `ammo`, `launches`; `Content::fits`, `Content::fires`, `Content::hand_slots`).
`equipment.rs` holds the runtime side: `Equipment` (slot → inventory position, on
`Actor::equipment`, saved and hashed; equipped stacks stay in the inventory and keep weighing),
`slots_taken` (every hand for a two-handed object), `free_slot` (spawn data), `mark` ("being
worn"), `verbs` (the message verbs per action). `world/equip.rs` is the rules: `Action::Equip {
slot, item }` / `Action::Remove(slot)` take off what is in the way (a swap) and cost the slot
action (`wield`, `wear`, `body_armor`, `ready` in `data/actions.toml`) of everything put on or
taken off (`World::price_for`); dropping takes off first (`take_off_before_dropping`) and
`Equipment::after_removal` follows the letters. Creatures wield and wear from their spawn
(`wield`/`wear` in a level legend, resolved in `content/level.rs::equip`). Presentation reads
`PerceptionView::equipment` (the player's own) and `PerceptionView::gear(actor)` (what a seen
creature wears and wields, all of what it carries that is seen; look mode, `ui/look.rs::gear`).
Digging reads it too: `Actor::dig_tool` (the first equipped thing with `dig` power: the Tool slot's
pick-axe). Combat reads equipment (below): `Actor::weapon` (the first hand holding a weapon),
`launcher` and `ammunition` (the `ready` slots), and `defence(content, burden)` (armour, shield,
parry, Dexterity capped by worn `max_dex` and the burden's `max_dex`, the lower winning; the rules
ask `World::defence_of(id)`, which passes the actor's `Load::penalty`).

Combat (spec §8 "Combat"; M16, #116): the pure rules are `warlock-core/src/combat.rs` (2d12
`roll_natural`, `hits`, `hit_ways`, `ways`; `Category` (Dexterity, Armour, Shield, Parry,
Natural, Dodge, Deflection: one entry each in its `categories!` list) and `Defence` (highest per
category, Dodge stacks; `why_missed` weighs a miss by the positive bonuses); `Attack` (what data
gives: dice, to-hit, threat, crit, a bow's Strength) and `Profile` (an attack with the attacker's
modifiers: `melee`, `ranged`, `thrown`); `strike` resolves one with a fixed order of draws from
the gameplay stream; `max_hp`, `regen_seconds`, `Health` words). `world/fight.rs` applies them:
`World::attack` (picks the profile: melee weapon / natural attack / bare hands, the readied
launcher for a shot, none for a spell) → `attack_with(attacker, target, kind, Option<Profile>)`
(**the entry point for ranged combat, #117**) → `Event::Attacked { .., strike }`, noise, damage
(`hurt`), `Event::Died`, then for a target left alive `World::after_blow` (status effects:
wounds and `on_hit`, `world/status.rs`); `dies` is the one way an actor dies (a blow or a status,
`Event::Died { of }`): `creature_dies` removes the actor and piles its inventory on its cell,
the player's death sets `WorldState::death` (`Death`: turn, time, place, `by` the creature type
if seen, `of` the status that killed) and `run_turn` stops; `World::apply` refuses everything
after it. Healing and status ticks run as the scheduler moves the clock (`World::pass_time`,
below). Messages: the player's own attacks are said at once
(`player_attacks`, by what they see as they strike); a creature's in `World::perceive_events`
(`fight::attack_message`, `death_message`; actors that died this action are in the unsaved
`World::fallen`, so their names and words still resolve). Data: objects `damage`, `threat`,
`crit`, `to_hit`, `strength`, `parry`, `armour`, `max_dex`, `shield` (`content/object.rs`);
creatures `base_hp`, `attack` (`NaturalAttack`), `natural` (`Hide`); races and classes `hp`; a
scenario without a character `hp`. Hit points live on `Actor` (`hp`, `hp_base`, `regen`;
`max_hp(content)`, `health(content)`: the effective Toughness), saved and hashed. Presentation reads only
`PerceptionView::own_hp`, `own_defence` (the total: its Dexterity part would give a modifier
away), `SeenActor::health`, and `death`: **never a creature's hit points, a to-hit, a roll, or a
modifier**. Spells stay intents (`strike: None`). Tests: `combat.rs` (exhaustive 2d12
enumeration for hit chances, criticals, miss attribution), `world/fight_tests.rs`.

Weapon skills (spec §8 "Weapon skills"; M22, #134): data is `data/weapon_skills.toml`
(`content/weapon_skill.rs`: `WeaponSkillTable` with `MarkRules`, named need curves, skills
(`WeaponSkillDef`: `id`, `name`, `phrase`, `Role`, curve, `[Benefit; 11]` per rank) and the
symmetric defaults; `SkillId` is an index in file order, in the content fingerprint). Weapons
name theirs (`category` → `ObjectDef::skill`, its role checked against what the weapon is),
shields bash (`bash` → `ObjectDef::bash`), races and classes give starting ranks
(`weapon_skills`, dice; classes `weapon_skill_choice`), creatures fixed ranks. The runtime side
is `weapon_skills.rs`: `WeaponSkills` (a `Training { rank, marks }` per skill on
`Actor::weapon_skills`, saved and hashed, `FORMAT_VERSION` 16), `train` (marks → a rank, then
`raise`: the defaults rise, their marks reset; no chaining), `from_ranks` (starting ranks,
order-free), `progress`. The rules hook in at one place each: `Actor::attack_skill(kind, with)`
picks the skill of an attack (a blow: the wielded weapon's, else unarmed; `Melee` `with` a
shield: shield bashing; a shot: the launcher's; a throw: thrown; a kick, spec §8 "Doors": unarmed
whatever is wielded, `World::player_kicks`), and `fight.rs::resolve` takes the skill its caller
picked; `Profile::with(&Benefit)` and
`Actor::melee`/`ranged`/`thrown`/`bash`/`kick` apply to-hit, damage, and threat; `Actor::defence` adds
parry and shield blocking; `World::attack_price` takes `speed`/`reload` off the cost;
`World::resolve` (`fight.rs`) earns the player's marks (`practise`; a Shield-credited miss on
the player trains shield blocking; `World::level_of` weighs the opponent by its creature type's
`level`, spec §8 "Experience") and pushes "You feel more skilled ..." into the unsaved
`World::news`, said after the attack's own messages (`say_news`, also at the end of
`perceive` for a block in a creature's turn). A later kind of skill (fighting styles, #135) is another `Role` with the
same ranks, marks, curve, and `Benefit`s (`Benefit::plus` sums them). Starting ranks roll from
their own stream (`character/weapon-skills/...`), so stat rolls never move. Presentation reads
`PerceptionView::own_weapon_skills` only: ranks and a bar, **never a benefit**. Tests:
`content/weapon_skill.rs`, `weapon_skills.rs`, `world/weapon_skill_tests.rs`; the wiki's tables
are `xtask/src/wiki/weapon_skills.rs`.

Key commands `,`/`g` (pick up: one stack at once, else the `PickUp` screen), `d` (`Drop` screen),
`i` (`Inventory` screen), `e` (`Equipment` screen), `@` (`Character` screen), `W` (`WeaponSkills`
screen), and bash (`KeyCommand::Bash`: a direction prompt, `InputMode::Direction`) are in
`input.rs` and `settings/keys.rs`. **Abilities** (racial, class, skill, style; the owner's rule,
#156) get no key of their own: they go into one abilities menu (to come). Each is a core
`Action` with replay text, an entry in `warlock_core::ability` (`Ability`, listed by
`PerceptionView::abilities` when usable), and a rebindable `KeyCommand` left unbound by default
(`settings/keys.rs::UNBOUND`). Shield bash is the first.

Stats (spec §8 "Stats"; M14) live in `warlock-core/src/stats.rs`: `Stat` (Strength, Dexterity,
Toughness, Intelligence, Willpower, Presence; id, abbreviation, name) is generated by the
`stats!` list there, so **a new stat is one entry in that list** (and a `FORMAT_VERSION` bump);
`PerStat<T>` holds one value per stat (serialized as a map by stat id), `Stats` an actor's scores
and potentials (`Actor::stats`, saved and hashed), and `modifier(score)` /
`Stats::modifier(stat)` the D&D 3.5 modifier (`floor((score − 10) / 2)`) for the rules. Content:
`stats` and `potential` tables (by stat id; 1 to 40; missing score 10, missing potential = the
score, never below it; `Stats::from_data` validates) on creature types in `data/creatures.toml`;
the player's come from their character (below), or from a scenario without one
(`ScenarioDef::player_stats`; the `player` creature may not set any). **Never show a modifier, a potential, or a creature's stats**: presentation reads only
`PerceptionView::own_scores` (the player's scores as the rules read them, lowered by status
effects; `own_base_scores` without them, so the character screen can mark the lowered ones) — the
character screen and the `stats` dump layer; only the dev `actors` layer and `--state` dumps show
the rest. Only combat (Strength, Dexterity, Toughness) and burden (Strength and Toughness scores)
use stats, **always through `Actor::effective_stats(content)`** (status penalties applied; never
`actor.stats` directly in a rule, but for the hit points a level adds, `World::level_hp`, so a
passing penalty never costs them for good), and no other D&D concept (checks, saves) comes with them. The status bar has no stats segment: it is already
full at common window sizes (it shows `HP 17/17` right after the level name, dropping only
before it, and the burden right after that when there is one).

Burden (spec §8 "Burden"; M16, #115) lives in `warlock-core/src/burden.rs`: `Load` (`carried`,
`limit`, `ignores_burdened`; `burden()` the level by thirds, `penalty()` what the actor suffers:
Unburdened for a Burdened dwarf, `thresholds()`/`over()`), `Burden` (Unburdened, Burdened,
Strained, Overloaded; its effects: `slow(price)` ×1.25/×1.5 rounded up, `max_dex()` +3/+1/0,
`footsteps()` 125/150 %, `can_move()`, `news()`), `limit(st, to, carry)` = 1500 g × (2 St + To)
× carry % (always a multiple of 3, so thirds are exact), and `load(actor, race, content)`.
Sizes are data, `data/sizes.toml` (`content/size.rs`, `SizeDef { id, name, carry }`;
`Content::sizes`/`size`/`size_index`; `medium` required, the default; without the file only
medium); races have `size` and `ignores_burdened` (dwarves), creature types `size` (the
kobolds, the rat, the bat, and the beetle are small). The player's size is their race's when the
run has a character (`WorldState::load(content, actor)` picks it; `World::actor_load(id)` for
dev tools). Nothing is stored (no `FORMAT_VERSION` bump): the burden follows from the inventory,
stats, and content. Every penalty goes by `Load::penalty`: prices in `schedule.rs`
(`World::burdened`, inside `price_of` and `step_price`: the one place, so every action and
overworld journey is slowed), `World::defence_of`, `make_noise`, and moving (`World::can_move`
refuses steps, edge exits, stairs, and entering a location or local area: "You carry too much to
move."; `creature_steps` makes an overloaded creature wait). `World::apply` says the change
(`burden_news`) right after the player's action. Presentation reads `PerceptionView::own_load`:
the status bar (`Burdened`/`Strained`/`Overloaded` after the HP, priority 1), the inventory
(72 wide; `Carrying 28.4 kg (burdened)` and the thresholds line, `character_screen::thresholds`)
and the character screen.

Character creation (spec §8 "Character creation"; M15) lives in `warlock-core/src/character.rs`:
`CharacterSpec` (race id, `Gender`, class id; `FromStr`/`Display` as `dwarf,female,warrior`;
serialized as `{race, gender, class}`) and `CharacterSpec::start(content, seed, start_date)` →
`Start` (starting `Stats`, the race's vision, and a `Character`: the spec, `raised` (the stat the
gender raised: male Strength, female Dexterity, other `other_stat(seed)` from the
`character/gender` stream), `born` (`BirthDate { year: i64, month, day }`: age uniform in the
race's `age` range and birthday from `character/birth/<race>`, counted back from the start date
with `BirthDate::counted_back`; `age_on(date)`), and `sign` (`content::Sign { animal, element }`,
the birth year's)). Scores: 10 + race + gender + class + animal + element, clamped to 1..=40;
potentials `base_potential` + the race's, never below the score. Flags for rules:
`Character::born_of_element("fire")`, `born_in_year_of("manticore")`. Content:
`data/races.toml` (`content/race.rs`: `Races` with `base_potential`, `RaceDef` with `vision`,
`age`, `stats`, `potential`, `hp`), `data/classes.toml` (`content/class.rs`: `ClassDef` with
`stats`, `hp`; `Start::hp` is their sum),
and `data/horoscope.toml` (`content/horoscope.rs`: `Horoscope` with `anchor_year`, `animals`,
`elements`, `sign_of_year`, `great_cycle` (60), `name` ("Fire Manticore")); all optional.
Adjustments are `PerStat<Dice>` (`stats::adjustments` checks every result is within -10..=10;
`stats::roll` rolls them in stat order) and are rolled from `character/rolls/<spec>` in a fixed
order (race, class, animal, element, race potentials): **adding a die to data changes later
rolls** of every character with that race or class. `Dice` (`warlock-core/src/dice.rs`) is the
one dice type (parse `2d3+1`/`-1d2`/`+2`, `Display`, `min`/`max`, `roll(&mut Rng)`); combat
uses it for damage. Scenarios name their character (`character = { race, gender, class }`,
`ScenarioDef::character`); with one, `vision`, `stats`, `potential`, and `hp` are refused.
`World::new` starts as the scenario's character, `World::with_character(content, scenario, seed,
Some(&spec))` as another; `WorldState::character` (the `Character`, saved, hashed;
`FORMAT_VERSION` 9) and `PerceptionView::character` (the player knows it; `PerceptionView::time`
for their age). Replays record only the spec. The summary and the character screen show final
scores only: never dice, a breakdown, a modifier, or a potential (the step screens and the wiki
show adjustments as written in data). Balance is not a goal; classes only adjust starting
scores and give hit points.

Factions and creature behavior (spec §7 "Factions", §5 "Creature sight", §8 "Creature behavior"):
`data/factions.toml` (`content/faction.rs`: `Factions`, symmetric hostility, `Stance`; optional,
without it everyone is neutral, but then no creature may name a faction) and per creature type
`faction`, `vision`, `sight`, `combat` (`melee`/`ranged`/`caster`), `range`, `opens_doors`
(`content/creature.rs`, checked in `content/mod.rs`). Stance comes from types:
`Content::stance(creature, creature)`, `World::stance(actor, actor)`, and for presentation only
`PerceptionView::stance(seen actor)` (look mode's "(hostile)"). The AI is `world/ai.rs`
(look → fight → search the last seen spot → look into the newest unseen noise → back to the post →
patrol), paths `world/path.rs` (A*, fixed tie-breaks), and per-creature memory `Mind` on `Actor`
(saved, hashed). Creature sight is `sight.rs`: the player's rules computed per creature, only for
cells where hostiles stand (nearest first, line of sight only out to that distance, darkvision
before light, bounce light only when needed); it must stay equal to the player's perception —
`light_tests::check_creature_sight` compares `sight::light` with the perceived light on every
visible cell in the light property tests (and under each weather in `weather_tests`). Both take
the sky from `WorldState::sky` (the weather's clouds included) and overworld sight from
`daylight::sight_under` (rain included). A creature's attack is `World::attack` (Combat above):
`Event::Attacked` (kept only when the player sees both ends), a noise (`noise::Act::Attack`), and
messages built in `World::perceive_events` (unseen ends are "something"). The app shows a lunge
(`ui/motion.rs`, `motion` switch), a shot glyph, and a creature's `on = "attack"` particle burst
(`ui/effects.rs`, `particles` switch); `Event::Died` has no effect of its own. No creature
behavior may use randomness (combat rolls are the rules', from the gameplay stream); tie-breaks
are by distance, direction order, and actor id.

The overworld (spec §3, §5, §7) is `assets/levels/overworld.toml`, the one level of kind
`overworld` (`LevelKind::Overworld`: a step's base time is the terrain's `travel_minutes`; every
other action, waiting and `>` included, the usual 6 s).
Its locations are stairs-down terrain (`hall`, `cave_mouth`, `dungeon_entrance`) whose legend `to`
names the level; the level's `<` with `to = "overworld"` leads back to the location (`World::travel`
arrives on the stairs leading back). `>` on any other cell enters the local area `local:x,y`
(`content/overworld.rs`: `local_area_id`, `parse_local_area`, `Content::local_area`, valid where
the terrain has a `local` ground and a land or road `landscape`), generated on the first visit by
the outdoor generator (`mapgen/local/`, below); the edge exit (`exit` in `LevelState`) makes
walking off any edge return to the cell. `>` arrives on the side the player walked onto the cell
from: `WorldState::overworld_step` (the last overworld step, cleared by any level change) picks
the side and `mapgen::local::arrival` the edge cell (road first, then dry, open ground near the
middle of the side); without a step, at the level's start (the middle). Sight on the overworld is line
of sight within `daylight::overworld_sight` (day 9, twilight 5, night 2 plus one per
`MOONLIGHT_PER_CELL` of `Sky::moonlight`, at most `SIGHT_MOONLIT` 4), ignoring light. The
overworld is sky-exposed (sun, moons, and sky, like outdoor levels); `daylight::overworld_night`
tops its ambient light up to the faint `OVERWORLD_NIGHT` where the sun's sky falls short
(moonlight comes on top), so cells in sight stand apart from memory at night (local areas get
none). Regions (`[[region]]` in level
files, `LevelTemplate::region_at`, `PerceptionView::region`) show in the status bar. Unlike
`vast.toml`, the overworld map is hand-authored: edit the file directly (keep every walkable cell
inside a region; a test checks).

Procedural generation (spec §7, §8) lives in `warlock-core`: `rng.rs` (ChaCha8 streams derived
from the run seed: `map/<level id>` per generated level, `gameplay` kept in `WorldState`;
`parse_seed` for `--seed`), `content/dungeon.rs` (branches in `data/dungeons.toml`; level ids
`depths:N`), `content/vault.rs` (vault rules), and `mapgen/` (`rooms.rs`, `caves.rs`, `vault.rs`
stamping, `connect.rs` flood fill / corridor digging / validation). A branch level is generated by
`World::travel` on its first visit and then persists in `LevelState` like a hand-made level (level
states carry their own ambient and placed lights). Generation is integer-only; a level that fails
validation is regenerated from the next derived stream. The insta snapshots in
`src/mapgen/snapshots/` and `src/mapgen/local/snapshots/` flag any change in generator output:
review the maps in the diff, then accept them deliberately.

Special rooms (spec §7 "Special rooms", M25) are data: `data/room_effects.toml`
(`content/room_effect.rs`: effects, the per-room `chance`, `per_level`). Generators make them last
(`mapgen/special.rs`, from the level's map stream, so levels without one are unchanged; rooms
levels: a room's rectangle, walls included; caves: a floor blob away from the arrival), and level
files mark them (`[[room]] effect area`; the hub's stairs nook is a sanctuary). They live in
`LevelState::rooms` (saved, with whether the player has been in each), and the rules ask one
question, `LevelState::room_effect_at` (public: `World::room_effect_at`): perception light and
creature sight (`room_shade` on every part of a cell's light, `room_glow` in the base light —
change `perception.rs` and `sight.rs` together; `special_rooms_keep_the_light_invariants` checks
they agree), noise (`noise::noise`), regeneration (`World::regen_rate`, the one pace hook), and
hostile creatures shunning a sanctuary (`World::shuns` in `ai.rs`). A flooded room's slower,
louder steps come from its floor terrain (`shallow_water`). `enter_room` (world/mod.rs) says the
first-entry message; `PerceptionView::room` gives look mode the rooms the player knows;
`PerceptionView::light_kept` lets the renderer dim its own transient light in a shadowed room.

Local areas (spec §3, §7) are generated in `mapgen/local/`: `mod.rs` (`Setting`: the cell's
landscape and its neighbors' roads, rivers, lakes, and land, read from the overworld; per-landscape
feature chances; the attempt loop, glade, ruins, deer), `kit.rs` (the terrain and creature ids it
builds with, checked when content with an overworld loads), `noise.rs` (integer value noise and
quantiles: "this share of the area"), `land.rs` (the canvas with per-cell `Claim`s: `Fixed` water,
roads, ruins; `Clear` glade, banks, roadsides, no obstacles; `Free`), `water.rs` (river and lake
bands, ponds, brooks, winding courses), `road.rs`, `cover.rs` (one function per landscape; wood
share blends towards the neighbors), and `reach.rs` (trails, joining pockets, ways out,
validation, `arrival`). Overworld terrain picks its generator with `landscape` in
`data/terrain.toml`; outdoor vaults are vault files with `kind = "outdoor"`. Tune by eye:
`WARLOCK_AREAS="25,20 60,44" cargo test -p warlock-core --lib print_areas -- --ignored
--nocapture` prints areas with one character per terrain (`WARLOCK_SEED` for the seed), and the
`forest` scenario or `--content-dir` scenarios with `start_level = "local:x,y"` capture them.

Digging (spec §8 "Digging"; M19, #131) lives in `warlock-core/src/world/dig.rs`. Terrain that
breaks has `hardness` and `becomes` (`TerrainDef::breaks: Option<Breakable>`, validated: only
solid terrain breaks, into walkable, transparent, plain ground); `void` (solid rock) has a `face`
(`wall`) it turns into when a cell next to it opens, as mapgen walls open ground. Cells on a
level's edge never break (`LevelState::on_edge`, `breaks_at`). `Action::Dig(Dir)` (`dig:DIR`,
key `U` = `KeyCommand::UseTool`, ADOM's use-item key, a direction prompt; `a` is left for the
abilities menu, #156) needs `Actor::dig_tool`; `start_dig` refuses (no time) or returns
`Spent::Dig(at)`, and `dig` runs `keep_at` with a `dig` action per turn
(priced by `price_of`, so burden slows it): `dig_turn` makes `Act::Dig` noise (`DIG_NOISE` 80,
`SoundKind::Digging`) and adds `dig_work(power, strength)` (power × (Strength score + 10), in
`WORK_SCALE` = 20ths of a hardness point) to `LevelState::dug` (saved, by position); at hardness ×
20 `break_wall` sets the `becomes` terrain through `set_terrain` (light masks), clears the
progress, bares `face`s around, and pushes `Event::WallBroken { pos, cause: Breach::Dug }`.
Explosions (none yet; the fireball trap, #128, is to be the first) call
`World::blast(center, radius, blast, events)`: every breakable wall in the burst
(line of sight from the center, computed before anything breaks) with hardness below the blast
breaks (`Breach::Blast`; `perceive_events` says "The wall collapses!" for those the player sees).
Perception sees and remembers the rubble like any terrain; `PerceptionView::ready_to_dig` and
`is_dug` (a visible wall with progress: look mode's "partly dug") are all presentation gets. The
app bursts the new terrain's `toggle` particles (rubble's dust) on `WallBroken`, like a door's.
Creatures do not dig. Times are in the wiki's generated `digging.md` tables.

Ranged combat (spec §8 "Ranged combat"; keys: `t` shoots or throws the Missile weapon slot's contents, `T` is the throw chooser, `f` unbound but rebindable; the slot `takes_weapons`) lives in `warlock-core/src/world/shoot.rs`: `fire`,
`throw`, and `creature_shoots` loose one missile (`loose`): it leaves its stack, makes its noise
once, flies along `Pos::ray` (the line the AI's `clear_shot` checks) over transparent cells as far
as the weapon's `range`, attacks whoever stands in its way (`World::missile_hits`, no noise) until
one is hit, and comes down (`come_down`) on top of the pile, unless it broke (`breaks`, one
gameplay draw on a hit) or the cell is not walkable. What is `lit` flies as its burning object and
starts a fire (`LevelState::fires`, saved): a light source in `LightCache::sources` while the clock
is before its end, cleaned up by `World::burn_out` at the start of `perceive`. Prices go through
`price_of`/`price_for` (shoot, plus `reload` for a launcher that `reloads`; throw), so burden
slows them too. `Event::Missile` carries the path; `perceive_events` cuts it to what the player
saw (a burning missile: everything in their line of sight). In the app, `ui/target.rs` is the
aiming input mode (`InputMode::Aim`: cursor start, Tab cycling, the line of fire as the player
knows it, `LastTarget`), `ui/missiles.rs` flies missiles on the free-glyph layer (glyphs turned
with `FreeGlyph::turn`, trail, embers via `particles::trail`, impact puff) and writes this frame's
`render::lighting::Transients` (the moving light of a burning missile, and landed lights held back
until it arrives; `draw_map`, object glow, and object particles hold back what is `arriving` too).

Skills (spec §8 "Skills", #132) live in `warlock-core`: `skills.rs` (`Skills` per actor: trained
skills only, a missing one is untrained and gives nothing; `Effect` and `Use`, the typed benefit
and training-mark hooks; `Benefit` growth by `steps` or a `curve`; the **provisional**
`Increase` curve), `content/skill.rs` (`data/skills.toml`; adding a skill is one entry), and
`world/train.rs` (marks for the player only, drawn from the `training` stream in `WorldState`,
never the gameplay stream). Race and class grants are dice rolled in `character.rs` after every
other roll, so they never move a score. Each effect is read in one place: Dodge in
`Actor::defence`, quiet in `noise::noise`, hearing in `Actor::hearing`, burden in
`World::burdened` (`Burden::slow_eased`), travel in `World::step_price`; each use is reported
where it happens (`make_noise`, `hear`, `fight.rs::resolve`, `apply`, `player_moves`). A new
effect or use is a variant there plus its one hook. The player sees values only
(`PerceptionView::own_skills`, the Skills screen `ui/skills_screen.rs`, `S`); never marks or
benefit numbers. Skills get no keys of their own: active uses will be chosen from the one
abilities menu (#156), which reads `PerceptionView::active_skills` (`OwnSkill::active`: the
skill's kind is `action`; `available`: usable now, false until the uses exist). Pending hooks: athletics (kicking doors, #130; climbing out of pits, #128) and
escape artist (pits, #128).

Doors (spec §8 "Doors"; M19, #130): the rules are `warlock-core/src/door.rs` (`Lock`, `Key`,
`kick_breaks`/`break_chance`: `1d(sturdiness) ≤ Strength`, one gameplay draw), applied in
`world/door.rs` (`open_door` unlocks with `Actor::key_for` and costs `unlock` + `open_door`,
`swing_open`, `close_door`, `kick`/`kick_door`; kicking a creature is `World::player_kicks`, an
unarmed blow whatever is wielded: `Actor::kick` (bare hands with unarmed fighting's benefits and
the stance), training unarmed fighting, priced `kick` by `attack_price`, said with `Words::kick`). Locks are per-cell state, never terrain: `locks` on
`LevelTemplate` and `LevelState` (by position, saved, hashed, `FORMAT_VERSION` 22; ground truth only, so perception,
memory, look mode, and dumps show a locked door as any closed door); `World::set_terrain` drops a
lock whenever the cell stops being a closed door. Level legends lock with `lock = "id"` or
`locked = true`; objects are keys with `key = "id"` (`"*"`: skeleton); closed doors break with
`breaks_to` + `sturdiness` (`TerrainDef::kicked_in`; digging and blasts break only terrain with a
`hardness`, `TerrainDef::breaks`, never a door). The AI (`world/ai.rs`) paths through a locked door, and opens it, only
with a key (`Stepped::Unlocked`); `go_look` stops next to a spot no one can stand on (a door kicked
at). Generated levels: `Needs::key` (the branch's `key` in `dungeons.toml`) makes `rooms.rs`'s
`lock_rooms` lock up to two single-door rooms and drop the key in an unlocked room;
`connect::validate` checks locks sit on closed doors and that the stairs and a key to each lock
(`LevelTemplate::has_key_for`, lying or carried) are reachable without unlocking anything; vaults
must hold their own keys (`content/vault.rs`, `check_keys`). The app binds `K` (`KeyCommand::Kick`,
a direction prompt like open/close, bash, and dig) and bursts the door terrain's `toggle` particles on
`Event::DoorKicked`/`DoorBroken`.

Blood (spec §8 "Blood", §4.8; M17): the pure pattern is `warlock-core/src/blood.rs` (`Wound`:
`volume` by kill or big hit (a third of the maximum, or a critical), size `blood` percent,
overkill, critical, missile; `spatter`: pool, fan away from the attacker (±45° melee, ±30°
missiles), kill flecks, stopped by opaque cells, walls next to the victim stained on their face;
`Stain`: amount to `CAP` 100, colour and spill time mixed by amount, `faces` bits; `seed` hashes
turn, time, and actor ids with SplitMix: **never the gameplay stream**). `world/bleed.rs`
(`World::bleed`, called from `fight.rs::resolve` before `hurt`) writes `LevelState::blood` via
`LevelState::stain` (terrain `stains`, data; never on the overworld). No rule reads blood;
`blood_tests.rs` checks a fight is identical with and without it. Perception: `PerceptionView::blood`
(open cells in sight; walls only from a stained face; else `blood_memory` as last seen, updated
in `Perception::remember`). Data: creature `blood` (`"#rrggbb"` or `"none"`, default
`blood::RED`), size `blood`, terrain `stains`. App: `ui/blood.rs::stained` shifts a cell's fg/bg
toward the blood (dried by age) in `draw_map` before the shader lights it, so darkvision, memory
dimming, and palettes apply as for terrain; look mode prefixes the terrain with `Stain::word`.
The `blood` switch hides both. Tune by capturing the arena at `wait*8` (first kills) and `wait*40`
(after the brawl) and the `blood-wall` golden's walk.

Status effects and healing (spec §8 "Status effects", "Healing"; M24, #138; **diseases, #139,
build on this**): statuses are data, `data/status.toml` (`content/status.rs`: `StatusDef` with
`seen`, `max`, `stacking`, `tick`/`damage`, `eases`, `regen`, `penalty`, `wound`, `cured_by`,
and the words `start`/`end`/`dies`/`death`; `Healing { resting }`; `OnHit` on a creature's
natural attack, `NaturalAttack::on_hit`; `Content::status`/`status_id`/`status_ids`/`healing`;
optional file; `StatusId` is an index, in the fingerprint). `crate::status` holds one actor's
effects (`Actor::effects: Effects`, at most one `Effect { status, severity, tick, wear, by }` per
status in status order; `Effects::add` applies the stacking rule, `penalty`, `regen`,
`is_valid_for`; `Effect::until_tick`/`until_eases`/`advance` are the timer math: ticks bank
`seconds × severity` degree-seconds). `world/status.rs` applies them: `World::afflict(actor,
status, severity, by, events)` (**the one way to give an actor a status**: hunger and rotten
corpses (#137), traps, diseases call it), `World::cure(actor, means, events)` (the Heal skill's
bandage, #132), `World::after_blow` (wounds by `combat::big_hit` and criticals, then `on_hit`
with one gameplay draw per chance below 100), `World::healing_rate` (min of the statuses'
`regen`, × resting for the player while `World::resting`; hook hunger's Weak here or as a status),
and `World::pass_time` → `live`, called by `run_turn` for every scheduled actor as the clock
moves: an event loop over the next tick, easing, or hit point back, so splitting time never
changes the result, deaths at their exact moment (`dies(.., Some(status), when)`), the blood
drip hook `drip_blood` (#127's stains go there). The player's statuses beginning and ending are
`Event::Status` (said by `perceive_events`: begin = `say`, stops a rest; end = `note`, does not;
the rest loop keeps them from the turns it does not perceive). Presentation reads
`PerceptionView::own_effects` (status bar after HP, character screen `Status` row) and
`effects_of(actor)` (look mode; only statuses with `seen`), never severities. Saved and hashed
(`FORMAT_VERSION` 20; `Actor::regen` counts hundredths of a second at the normal pace). The dev
action `afflict:ID` (`Action::Afflict`) gives the player a degree, for tests and captures. Tests:
`status.rs`, `content/status.rs`, `world/status_tests.rs`.

Rendering (spec §4.2) is three layers, bottom to top, all positioned in framebuffer pixels:
1. **Map grid** (`render/grid.rs`): a `Console` (`render/console.rs`) drawn by one quad with the
   console `Material2d` (`render/console.wesl`). Each fragment finds its cell from its framebuffer
   position and reads glyph coverage from the 1:1 font atlas (`render/atlas.rs`) with integer
   texel addressing, so still glyphs are pixel-exact. The grid is one cell larger than the map
   viewport and drawn behind a clip rect with a whole-pixel sub-cell offset (smooth camera). A
   per-cell `Rgba8Uint` texture (table entry, phase byte) drives in-cell animation (`wave`,
   `flow`, `shimmer`, `cycle`) evaluated in the shader from a uniform table (`render/anim.rs`).
   Water (`render/water.rs`): running water's flow (direction and speed, `water::flow_byte`)
   rides in the glyph texture's spare third byte, and the shader scrolls the glyph pattern
   downstream (cells with the same flow line up; a direction or speed change along a stream shows
   a seam). Glints are hashed from the level position (`ConsoleParams::level_origin`), so they
   stay put while the camera scrolls.
2. **Free-glyph layer** (`render/free_glyph.rs` + `.wesl`): producers push `FreeGlyph`s
   (pixel position, color/alpha, scale) each frame; they become one dynamic mesh, one draw call.
   Glyphs at rest read the 1:1 atlas; moving, scaled, slanted, or turned ones sample a
   supersampled (2×, linear) atlas whose slots are exactly 2× the 1:1 slots. Creatures,
   particles, missiles, the aiming line's dots, and `pulse` terrain live here.
3. **UI console**: the same console material over the whole window; unwritten cells are
   transparent.

**Visual lighting** (spec §4.4, `render/lighting.rs` + `lighting.wesl`, imported by
`console.wesl`): every frame `lighting::update` composites the perceived light for the map grid's
cells plus a one-cell margin, from `PerceptionView::base_light` (ambient plus gameplay bounce light,
steady, never shadowed by creatures) and `light_sources()` contributions, less creature shadows,
and folds in UV (violet tint) and the darkvision fade. Gameplay light lives in
`warlock-core/src/light/` (`mod.rs`: falloff and the wall-face rule; `cache.rs`: per-source shadow
masks; `bounce.rs`: one weak bounce; `sun.rs`: sunlight; `moon.rs`: moonlight and `Sky`) and is
combined in `perception.rs`;
Σ contributions + base light == `light(p)` is a tested invariant. **Creature shadows**
(`render/shadows.rs`, visual only): each visible creature shades every perceived source that
lights its cell, except one on its own cell (its carried light). The source is shadowcast
(`fov::compute_fov`) over the perceived terrain with and without the creature's cell as an extra
occluder; contribution cells only the first reaches keep `shadows::keep` of that source's light
(`KEEP` 0.4 right behind the creature, back to all of it at `REACH` 3.5 cells), so colored shadows
come from per-source attenuation. The sun and the moons are directional: `shadows::sky_shade`
walks `Emitter::shadow_steps(CREATURE_HEIGHT)` from the creature instead. `shadows::update` recomputes the `Shadows` resource only when
`Game` changes (once per turn), not per frame; darkvision still measures the unshaded light (else
its glow would brighten shadows). **Light stability** (`render/modulation.rs`; presets'
`stability.{intensity,hue,dips}` in `lights.toml`, absent = steady): each source's `Modulation`
is a pure function of its key, preset, and `VisualTime` (steady with the `flicker` effect off).
Its hue `tint` multiplies the source's contributions on the CPU (exact; drawn light =
Σ tinted contributions · shadow keep + base light, while darkvision still measures the perceived
light); its intensity `gain` becomes the per-cell brightness gain (drawn-light-weighted mean,
base light = 1) applied after shading. Tune with captures at several `--freeze-time` values;
the crystals, fungus, runes, and altar must stay constant. Two textures on the map material:
`light` (`Rgba32Float`: rgb light, 1.0 = fully lit, and saturation) and `light_flags`
(`Rgba8Uint`: lit/opaque flags, intensity gain byte). The shader
interpolates light per pixel between cell centers, never across opaque, unseen, or remembered
cells (walls are lit flat by their wall-face light); then exposure (readability floor, soft
overbright knee plus a faint glow in the light's hue), desaturation, gain. Unlit cells (remembered
terrain, the UI console's 1×1 textures) are drawn as they are. Free glyphs are lit on the CPU by
`VisualLight::sample` + `lighting::shade`, which mirror the shader — change both together.

**Sunlight** (spec §5 "Sunlight", `light/sun.rs`): a level gets the sun and sky iff
`LevelKind::is_sky_exposed()` (outdoor and the overworld; add new sky kinds there, nowhere
else); terrain `roofed` never does. `Sun::at(time)` reads a per-minute table (`SUN_PATH`) built by
`const fn`s with integer math only (fixed-point sine), so no runtime trig; a test pins a hash of
the whole table — after a deliberate change to the sun's path or colors, update the hash.
`Sun::shadow_steps(height)` is the one shadow line (core terrain shadows and renderer creature
shadows both use it). `Sunlight::compute` marches each cell of the bounce area toward the sun
(terrain `height`, `Content::tallest_terrain` bounds the reach); it is not cached. The sun is a
`SeenLight` with key `SUN_KEY` and `emitter: Emitter::Sun(sun)` (point lights:
`Emitter::Point { pos, preset }`; use `SeenLight::pos()`/`preset()`), last in `light_sources()`;
sky light goes into base light. `PerceptionView::sun()` gives the sun and sky of the moment on a
sky-exposed level (`is_up`, `is_night`, `elevation`), e.g. for day/night rules. The showcase
courtyard (`levels/courtyard.toml`, scenario `courtyard`) is up the hub stairs nook's `<`.

**Moonlight** (spec §5 "Moonlight", `light/moon.rs`): the moons are `data/sky.toml`
(`content/sky.rs`, `MoonDef`, `Content::moons()`; optional, none without the file, so test
fixtures have dark nights unless they add one). `MoonDef::phase_at(time)` is its age over its
period (16-bit fraction, 0 new, half full). `Moon::at` walks the sun's path (`Sun::at_minute`)
trailing the sun by its phase of a day, so a moon's elevation and shadow line are the sun's at that
minute; its `color` and `sky` scale by lit fraction, elevation (`RISE`), daylight (none while the
sun is up, all from `DUSK_DEPTH` 6° below), and size², integer math only; a test pins a hash of a
whole cycle — update it after a deliberate change. **`Sky::at(time, content.moons())` is the one
place sky light is assembled** (sun, moons, `Sky::glow` for base light, `Sky::moonlight` for
overworld sight): anything that dims the whole sky (weather's clouds) applies there. A moon that
`shines()` is a `SeenLight` with key `moon_key(index)` and `Emitter::Moon(moon)` after the sun in
`light_sources()`, lit by `Sunlight::compute(shadow, light, ...)` like the sun;
`Emitter::shadow_steps` gives the shadow line of either. `PerceptionView::sky()` has the moons.
The status bar shows the phase (`MoonPhase`, glyph in `ui/mod.rs`) and the full date
(`Calendar::full_date`); its segments give way by priority, the moon cut to its glyph first
(`ui::fit_segments`). Tune moonlight with captures of the `courtyard` and `forest` scenarios after
`skip:353h skip:30m` (full moon at midnight) and `--content-dir assets`, which reads `sky.toml`
from disk without a rebuild. The wiki's moons and overworld-sight tables come from `sky.toml`
(`Moon::brightest`); run `cargo xtask wiki` after changing it.

**Weather** (spec §5 "Weather", `weather.rs`; data `data/weather.toml`, `content/weather.rs`,
`Climate`, `WeatherDef`, `Content::climate()`; optional, always clear without the file, so test
fixtures without it have clear skies — `ContentFiles::remove("data/weather.toml")` gives built-in
content a clear sky, as the overworld tests do): stateless, `weather::conditions(seed, time,
climate, override)` (weather and `Clouds`) and `weather_at` (just the weather, cheap). Blocks of
~3 h with a hashed start shift; a block's weather is the Irwin–Hall-spread sum of `WINDOW` block
draws against the month's chances; clouds ramp over `CHANGE` around a change. Never keep weather
state: only the dev override (`Action::SetWeather`, `weather:rain`, `WorldState::weather_override`,
3 hours) lives in the world state. The clouds apply in `Sky::at(time, moons, conditions)`, and
nowhere else (rounded up, so `is_up`/`shines` never change); `Sky::clear` is the sky without them;
`WorldState::sky` is the sky of the moment for perception and creature sight alike.
Perception knows the weather only on sky-exposed levels (`Sky::conditions`,
`PerceptionView::weather`); `World::perceive` compares it with the previous snapshot's and says the
change (`weather_news`: rain starting or growing heavier `say`s, which stops a rest; the rest
`note`s, which does not). Overworld sight loses the weather's `sight` (`daylight::overworld_sight`).
Look at the weather of a seed: `WARLOCK_SEED=1 WARLOCK_FROM=4,1 WARLOCK_DAYS=10 cargo test -p
warlock-core --lib print_weather -- --ignored --nocapture` (a line per day, `.` clear, `o`
overcast, `r` rain, `R` heavy). Tune the clouds and the rain with captures of the `courtyard` and
`forest` scenarios after `skip:330m weather:rain` (noon) and `skip:353h skip:30m weather:heavy`
(full-moon midnight) with `--content-dir assets`; the wiki's weather tables come from the file.

Effects (spec §4.8), each off with its own switch (`particles`, `glow`, `distortion`,
`screen_fades`, `screen_shake`, `weather`) and all off with reduced effects:
- **Particles** (`render/particles.rs`): stateless, a pure function of visual time and a seed via
  a visual-only SplitMix RNG (ambient emitters: one particle per `1/rate` slot; bursts: `count`
  particles per event). `ui/effects.rs` emits them from perception (visible cells, seen creatures,
  `Outcome.events`) onto the free-glyph layer, below creatures; non-glowing ones are lit like
  free glyphs (`VisualLight::sample` + `lighting::shade`). `obstacle` emitters (foam) run like
  ambient ones on visible running water whose flow runs into a known cell that is not running
  water, turned so their `y` points downstream (`particles::Source::down`).
- **Glow and heat haze** run in `console.wesl` from two spare bytes of the per-cell animation
  texture (`CellFx`, written by `ui/effects.rs::draw_cell_fx`): haze moves where glyph coverage is
  sampled (backgrounds and light stay put); after lighting, colors are clamped to 1 (overbright
  light alone never blooms) and glowing glyphs are multiplied past 1. Free glyphs do the same
  through their HDR vertex color (`post::in_range`, `post::emissive`).
- **Post** (`render/post.rs` + `tint.wesl`): the camera gets `Hdr` + `Bloom` (threshold 1,
  additive, `Tonemapping::None`), so only glowing glyphs bloom (with `glow` off the camera stays plain LDR,
  pixel-identical to no effects). `ScreenTints` draws a fading color over the map viewport:
  the level-change fade, and `flash()` for hit-style events.
- **Rain** (`ui/rain.rs`, the `weather` switch): the weather's `drops`/`splash`/`ripples` emitters
  (`data/weather.toml`) run as ambient emitters on every visible unroofed cell of the grid (plus
  `BELOW` rows), keyed by level position; a particle is drawn only over a cell it rains on, at most
  `MAX_PARTICLES` a frame. Drops are `│` sheared along their velocity (`FreeGlyph::slant`, pixels
  across per pixel down, applied in `free_glyph::glyph_mesh`) and faint (`ParticleEmitter::opacity`).
- **Spell effects** (spec §4.8 "Spell effects"): `Event::Spell { effect, origin, shape:
  SpellShape::{Burst { center, radius }, Bolt { path }}, cells }` in `warlock-core/src/action.rs`
  (traps emit it; `perceive_events` cuts it with `world/spell.rs::perceive_spell` to the cells the
  player sees or has in line of sight). Data: `data/effects.toml` (`content/effect.rs`,
  `EffectDef`, `Content::effects()`/`effect(id)`; presentation only, not in the fingerprint; a
  test plays the same run with changed looks and checks the hashes). The dev action
  `effect:ID@X,Y[>X,Y]` (`World::show_effect`, `Spent::NoTime`) makes one for captures; because it
  takes no turn, `ui/spells.rs` reads each outcome once by `Game::applied`, not by the turn. The
  model is pure in `render/spells.rs` (`Timeline` travel/impact/linger, `strength`, `burst_heat`
  with `burst_distance` measured on screen, `bolt_strokes` (turned `━` strokes about a cell wide,
  a slow wander plus jitter, forks only into `open` cells), `darken_curve`, `shake_offset`);
  `ui/spells.rs` (`Spells`) starts casts (one after another; frozen time: all at visual time 0, so
  `--freeze-time T` shows T s after), writes `render::lighting::SpellLights` (lights, `dim`, and
  the cells seen, which spell light passes even when unknown), and draws glyphs and particles last
  (on top, unlit, undimmed). **Darkening** happens where the light is made: `VisualLight` scales
  everything but spell light by `SpellLights::keep()` (`(1 − dim)^2.2`, so `darken.amount` is how
  much darker it looks), free glyphs scale their floor (`glyphs::lit`) and emissive particles and
  burning missiles their color (`glyphs::darkened`) by `VisualLight::keep()`, and the map shader
  gets `ConsoleParams::dim` for its floor and unlit (remembered) cells. **Shake**:
  `Spells::shake` (whole physical pixels) is added to the camera in `draw_map_layer` before
  `camera_split`, so the grid, free glyphs, and light move together and the viewport edges never
  gap. `motion` off: a still flash (`Timeline::still`); `screen_fades` off: no darkening;
  `screen_shake` off: no shake. Tune with `--content-dir` and captures across freeze times (e.g.
  `effect:fireball@30,14` and `effect:lightning_bolt@21,17>58,17` in the arena, 0.03 to 0.6 s).

`ui/` decides what is drawn from the perception snapshot: `ui/mod.rs` (UI console, map grid in
base colors, the status bar: `status_segments`), `ui/glyphs.rs` (free glyphs), `ui/motion.rs`
(slides from `Outcome.events`, one per creature along all the steps it took that turn; camera
easing; new input completes running motion), `ui/effects.rs` (particles, bursts, glow and haze,
fades), `ui/rain.rs` (rain), `ui/spells.rs` (spell effects), `ui/sounds.rs` (heard-sound markers), `ui/look.rs` (look mode), `ui/dev.rs` (dev overlays: reveal map, the one reader of
ground truth, and light values).
Animation curves live in `render/anim.rs` (water: `render/water.rs`, test-only mirrors of the
shader) and are mirrored in `console.wesl` — change both together. Everything animated is a function of `VisualTime`. Effects are deliberately subtle (the
bar: sparse, short-lived particles, gentle bloom, barely perceptible distortion, short fades;
the one exception is spell effects, dramatic but short); judge
changes by capturing several `--freeze-time` values, at `--scale 1` and `--window 2560x1440
--scale 2`.

Hard rules:
- `warlock-core` never depends on Bevy, never reads wall-clock time, and never does platform I/O
  inside the rules. Iteration order in core must be deterministic (no `HashMap` iteration).
- Rendering, UI, and dumps read the per-turn **perception snapshot** only — never ground truth —
  so hidden information cannot leak. The only exception is an explicit dev "reveal" mode.
- Gameplay outcomes never depend on frame rate, animation, or visual randomness.
- Keep modules small with clear responsibilities. No speculative abstractions, no premature
  optimization.

## Wiki

`wiki/` is the player wiki: Markdown pages for players (rules and world facts, not developer
notes, and no locations or level walkthroughs yet), built with [mdBook](https://rust-lang.github.io/mdBook/)
(`wiki/book.toml`, table of contents in `wiki/src/SUMMARY.md`, default theme `coal`). It is
published at https://jannef.github.io/warlock/wiki/ by the release workflow's web job, which
builds it into `pages/wiki/` beside the game. Facts that live in data (months, moons, weather,
terrain, footstep loudness in words, vision profiles, light presets (named as the player sees
them), creatures, factions, objects, materials, equipment slots and what fits them, scenarios,
action costs, sight radii, the pack's
size, the stats' names, races (stat adjustments as written, dice included, vision, starting
ages, hit points), classes (stat adjustments, hit points), what each gender raises, the horoscope
(animals, elements, and the great cycle's signs and years), combat (weapons, armour and shields,
the 2d12 hit chances and threat ranges, regeneration by Toughness), status effects (what causes
them, what they cost, how they wear off, healing and stats) and healing paces, burden (its levels
and what each costs, the races' sizes, sample carrying limits); never hidden rules such as
speed, hearing thresholds, creature sight, creature weights, stat modifiers and potentials, a
to-hit, or creatures' stats, hit points, and natural attacks and armour) are tables
generated by
`cargo xtask wiki`
(`xtask/src/wiki.rs`) into `wiki/src/generated/` (committed) and pulled into pages with
`{{#include generated/NAME.md}}`; CI runs `cargo xtask wiki --check` and `mdbook build wiki` on the
Linux job (mdBook pinned there and in `release.yml`). To preview: `cargo xtask wiki --build`, then
open `wiki/book/index.html`.

**A PR that changes gameplay mechanics or world facts updates the wiki in the same PR**, like the
spec and this file: edit the page, and generate a table in `xtask/src/wiki.rs` if the fact lives
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
  on macOS, so run the full local check there before every PR. `main` is branch-protected: PRs
  only, the Linux, wasm32, and cargo-deny checks must pass, linear history.

## Golden screenshots

> **Temporarily advisory (#166).** While the current wave of visual changes lands, CI's golden
> comparison doesn't fail the build, and feature PRs don't regenerate or commit reference images
> (adding manifest entries is fine). #166 regenerates and reviews everything, then makes it
> blocking again; until then, ignore the procedure below.

`tests/golden/manifest.toml` lists one entry per showcase room plus the vast level (its start and
its river): a scenario and text `actions` (`move:N*3 open:E`; `*N` repeats) or a `replay` file,
and optional `window`, `scale`, `freeze_time`, `screen` (`--screen` over the run: `pause`,
`help`, `settings`, `controls`, `inventory`, `equipment`, `equip-item`, `character`, `pick-up`,
`drop`; or over the main menu, the replay then unused: `race`, `summary`), `args` (more game
flags, e.g. `["--fire"]`), `tolerance` (per-channel, default 2) and `max_diff_ratio`
(default 0.05% of pixels).
`cargo xtask golden` builds a replay per entry with `warlock-core`, runs the game with
`--replay ... --capture ...`, and compares with `tests/golden/<name>.png`. On a mismatch it writes
`target/xtask/golden/<name>.{actual,diff}.png` (diff: red pixels are over tolerance) and fails.

References are produced on **Linux CI (Mesa lavapipe) only**; other GPUs and drivers rasterize
differently, so on macOS `cargo xtask golden` is advisory and `--local` is the way to look at the
captures. CI compares and never updates.

Deliberate update procedure (after an intended visual change, e.g. lighting or effects landing):
1. Push the branch, then Actions > CI > Run workflow on that branch with "update_golden" ticked
   (`gh workflow run ci.yml --ref <branch> -f update_golden=true`, then `gh run watch`).
2. `gh run download <run-id> -n golden-references -D tests/golden`, review the images (look at them,
   and check the diff against the old ones makes sense), and commit them with the change.
3. Adding a room or scenario: add a manifest entry and generate its reference the same way. Never
   update references just to make a failing check pass.

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

# Command-line reference

Developer notes for the development interface (moved from CLAUDE.md, #1505): every flag of the
game (`warlock`), the headless dev tool (`warlock-cli`), the `cargo xtask` commands in detail,
the replay format, the dump layers, and the action text syntax. What the interface must do is
spec §9 (`spec/09-development-interface.md`); CLAUDE.md keeps the one-line command table and
points here. Keep it current with the code, as CLAUDE.md says: a new flag, `--screen` name,
scenario, dump layer, or action is added here in the same PR, by its group, never at the end
(an action's rules belong in its own topic file; here its syntax and a short gloss).

Where the lists come from: `--scenario` names are the files of `assets/campaigns/*/scenarios/`,
`--screen` names are `ui/screens.rs::ScreenName` (clap lists them: `cargo run -p warlock --
--help`), dump layers are `dump/layer.rs::Layer`, and action verbs are `action/text.rs`'s `VERBS` table
(read by `Action::from_str`).

## `cargo xtask` commands in detail

The one-line table is in CLAUDE.md ("Commands"); this is what each row does.

- **Screenshot smoke test**, `cargo xtask smoke`: writes `target/xtask/smoke.png`; captures
  after 60 frames, retried once after 600 if the capture is a single flat color, which still
  fails: #669.
- **Web build**, `cargo xtask web`: writes `target/web/`, without the test world (CLAUDE.md
  "Releases, web build, logs").
- **Release configuration**, `cargo build --no-default-features -p warlock -p warlock-cli`: no
  `showcase` feature, the test world left out, as in a release; `cargo test
  --no-default-features -p warlock-core --test release_content` checks it; CI's
  `release-config` job; docs/dev/campaign.md.
- **Web smoke test**, `cargo xtask web-smoke <chrome>`: serves `target/web`, drives headless
  Chromium through menu → New run (skipped: the web build lists one scenario, the Ascmearc's
  start, so it opens on the race) → race, the stars, gender, class (preselected) → talents
  (Random) → name (empty: a random one) → summary → the story (opened once as the run begins;
  Enter) → the run on Beorggeat, checks the new character's save `warlock/saves/<id>.sav`, waits
  five turns, reloads, Continue, and checks the run resumed at turn 5; writes
  `target/xtask/web-smoke.png`; fails on logged errors; not run in CI.
- **Regenerate the vast level**, `cargo xtask gen-vast`: writes
  `assets/campaigns/showcase/levels/vast.toml`; deterministic, never hand-edit. `--check` fails
  if the file differs from the generator output; run it after touching `xtask/src/vast.rs` (CI runs it).
- **Third-party notices**, `cargo xtask licenses`: regenerates `THIRD_PARTY_LICENSES.html` with
  `cargo-about`, config `about.toml`/`about.hbs`; install: `cargo install cargo-about --locked
  --features cli`; `--check` fails when it is stale (CI runs the same `cargo about` call in shell,
  without building xtask: keep `.github/workflows/ci.yml`'s `deny` job in step with
  `xtask/src/licenses.rs`). **Run it after changing
  dependencies** (`Cargo.toml`/`Cargo.lock`); keep `about.toml`'s `accepted` list in sync with
  `deny.toml`'s `allow`.
- **Player wiki tables**, `cargo xtask wiki`: writes `wiki/src/generated/*.md` from the game data
  (not committed: CI and the release workflow run it before `mdbook build wiki`); `--build` also
  runs `mdbook build wiki` into `wiki/book/` (needs `cargo install mdbook --locked`);
  `--check-links` builds, then fails on any link to a missing page or anchor, or an anchor
  defined twice (CI runs it).
- **Golden screenshots**, `cargo xtask golden`: compare, authoritative on Linux only; `--local`
  captures to `target/xtask/golden/*.local.png`, no compare; `--update` rewrites references
  (CLAUDE.md "Golden screenshots").
- **Frame-time benchmark**, `cargo xtask bench-frames [WxH]`: release, offscreen; the hub, again
  with the Flicker switch off (`steady`), the vast level, and fifty spell circles with the switch
  on and off, `xtask/src/bench_circles.rs`; method and results in `docs/perf.md`.
- **Turn-time benchmark**, `cargo xtask bench-turns [ROW...]`: re-runs itself in release (rows
  named by label run alone, `bench-turns l-1 long`). Budgets: hub
  5 ms, vast 16 ms, generated depths level 5 ms, sunlit courtyard at dawn 5 ms, overworld 5 ms,
  generated forest area at dawn 5 ms, courtyard and forest area at full-moon midnight 5 ms, arena
  5 ms, a generated 120×60 cave floor at full population (a fixture branch of danger 8 with a
  teeming den, `delf`), the same floor with all but its kobolds shedding a spell circle's light
  (`d-hex`; its rest is shown, not judged: `bench.rs::UNJUDGED_REST`), and the same floor of
  animated panoplies at danger 40 (`panop`) 16 ms per turn, and that floor with a high-level
  ranger and their companion beside them (`d-pet`, 16 ms), and the deepest floor of a 40-floor
  cave branch, fresh (`l-1`) and after visiting the other 39 (`long`: a long run, #1507), 16 ms
  (their rests shown, not judged); a dusk-to-dawn rest 500 ms (on `delf`
  too: far creatures take coarse turns while the player rests, `docs/dev/creatures.md`), also
  beside the arena's fight going on unseen until one side is dead (a walk that gets the player
  killed starts over, untimed), and rested on to dawn when rain wakes the player.
- **Light stress benchmark**, `cargo test --release -p warlock-core turn_cost -- --ignored
  --nocapture`: 400×300, 825 lights; again outdoors at 06:15.
- **Balance probe**, `cargo test --release -p warlock-core balance_probe -- --ignored
  --nocapture`: prints the tables `docs/balance.md` records: the player against the monsters of
  their level, through the game's own code; run it after tuning combat data and update the
  tables.

## The game: `warlock`

`cargo run -p warlock -- --help` lists the flags and the values clap accepts.

### Scenarios: `--scenario NAME`

Default `showcase`; without the test world, the first campaign's start. The names are the
scenario files of `assets/campaigns/<id>/scenarios/`.

The Ascmearc (`assets/campaigns/ascmearc/`):

- `ascmearc`: on Beorggeat, the pass into the Ascmearc, the first campaign.
- `ceaptun`: on the market square of its town, Cēaptūn, with 60 gold.
- `holtwic`: on the lane into Holtwīc, the village at the north road's end, with its wise-woman.
- `aelfham`: on the first floor of Ælfhām, the hidden fey hall under the mound near it.
- `cealdhlaw`: on the first floor of Cealdhlāw, the Ascmearc's first dungeon.
- `deopgang`: on the first floor of Dēopgang, the deep way down from the south-west corner.
- `gimmwic`: in Gimmwīc, the deep gnomes' town at its bottom.
- `fennmynster`: on the first floor of Fennmynster, the drowned minster's cellars.
- `fyrdelf`: on the first floor of Fyrdelf, its deepest.
- `grundleas`: on the first floor of Grundlēas, the endless branch that forgets each floor left.
- `rat_king`: on its tenth floor, kept by its first layer boss, Gnægan the Rat King, whose death
  opens the way down (a level-1 warrior: give levels with `xp:` and `levelup:auto`).
- `utgang`: on the top floor of Ūtgang, the way up from Fyrdelf's bottom (its `<` climbs out and
  reveals its hidden way in).
- `drake`: in a lava hall eleven cells from the fire drake, a dragon (a level-1 warrior: give
  levels with `xp:` and `levelup:auto`).

The test world (`assets/campaigns/showcase/`, the `showcase` feature):

- `showcase`: the hub.
- `depths`: in the generated dungeon.
- `courtyard`: in the sunlit courtyard at 06:30.
- `world`: on the overworld beside the Hall.
- `forest`: in the glade of a generated forest area of the overworld at 06:30.
- `arena`: in the dark corner of the arena where kobolds and giant rats fight.
- `quarry`: in the quarry beside a pick-axe (its pit holds a pool of deep water to swim in, with
  `--character lizardfolk,female,warrior`).
- `traps`: in the trap hall.
- `summoners`: in the summoning hall's south hall (three summoners behind doors).
- `thieves`: in the thieves' hall (a pixie, a nymph, and an imp that rob you as they fight,
  behind doors; a blink dog past statues).
- `chests`: in the chest hall (every kind of chest, a mimic, and both treasure rooms).
- `hellegeat`: on the first floor of Pit 1 of Hellegeat, the fiends' branch a `portal` miscast
  opens (its record made at the start: its `<` climbs out to the hub).
- `shrine`: before three forgotten altars, good, neutral, and evil, to pray at.
- `forge`: beside a forge's anvil with arms and armour to temper.
- `spring`: below a cave room with a healing spring to drink at.
- `circles`: in a hall of peaceful kobolds already buffed and hexed, knowing every buff and hex
  spell with 300 mana, for the spell circles.
- `vast`: the 420×300 cavern level (scrolling camera, river, rooms, peaceful patrols, many
  lights), for captures and perf checks (`cargo xtask gen-vast` makes it).

### The run and the character

- `--seed S`: default the scenario's seed; a number, or a word hashed with FNV-1a, e.g. `--seed
  warlock`.
- `--character RACE,GENDER,CLASS` (e.g. `dwarf,female,warrior`; default: the scenario's
  character): like `--seed` it shapes the run other flags start (`--scenario`, `--new`, `--record`,
  `--capture`, and `warlock-cli dump` and `replay --create`) and alone does not skip the menu;
  a class closed to the race and gender, `drow,male,priest`, is refused, while a class hidden
  from New run (none is now) is taken; `assets/data/classes/README.toml`.
- `--name NAME`: the character's name, up to 20 characters; default: the scenario's for its own
  character, else one of the race's names picked from the seed; like `--character`. A run
  started with `--character` or `--name` (or from New run) has a chosen character and starts with
  its starting kit; `--scenario` alone starts with the scenario's pack
  (docs/dev/character-creation.md).
- `--talents IDS` (the creation talents, `hardy,weapon_focus:long_blades`): like `--character`,
  and of the scenario's own character without one; a chosen character takes these, else its
  Random picks (docs/dev/progression.md "Creation picks").
- `--new`: start a fresh run (a new character, with a save of its own) instead of resuming the
  character played last.
- Without `--scenario`, `--new`, `--replay`, `--record`, or `--capture` the game opens on the
  main menu.

### Content, window, and captures

- `--content-dir assets`: read content from disk instead of the copy built into the binary
  (`warlock-cli` takes it too).
- `--window WxH` (physical pixels), `--scale F` (display scale; `1` for reproducible captures).
- `--capture out.png`: render **offscreen** (no window), save a screenshot, exit.
  `--capture-after-frames N` (default 10), `--freeze-time SECONDS` pins animation time (glyph
  animations show that moment; slides and camera moves complete instantly).
- `--reduced-effects`: force reduced effects on (no glyph animation, light flicker, slides,
  camera easing, particles, bloom/glow, heat haze, fades, screen shake, or falling rain; a spell
  effect is a still flash); always wins over the settings
  (`Settings::effective_effects(force)`). With `--freeze-time`, particle bursts from the last
  turn's events (a `--replay`'s last step) show a fixed moment of their life; fades complete
  instantly.
- `--config-dir DIR` keeps `settings.toml` in DIR; captures and replays otherwise use default
  settings and never write the file.

### Screens: `--screen NAME` (dev)

Opens a screen at startup, even with `--capture` (e.g. `--scale 1 --screen new-run --capture
target/menu.png`). The names are `ui/screens.rs::ScreenName`'s; those that open by themselves
(`death`, `level-up`), the dialog under the forging wizard, and a profession or recipe by id are
no `--screen`.

On the main menu:

- `main`, `new-run` (a release build, which skips New run, shows the race), `characters` (the
  saved characters; a capture reads them, never writes, from `--save-dir`, e.g.
  `tests/golden/saved-characters`), and `graveyard` (the fallen, most experienced first; a
  capture reads `graveyard.toml` from `--config-dir`).
- Character creation over New run and the steps before them, each on its preselection (the
  first scenario's character, the Ascmearc's, an empty name): `race`, `sign` (the birth sign's
  reveal), `gender`, `class`, `chargen-talents` (the creation picks), `name`, and `summary`.

Over a run (e.g. after a `--replay` that walks onto a pile):

- `pause`, `help`, `inventory`, `equipment`, `character`, `skills` (the trained skills),
  `weapon-skills`, `magic` (the Magic page, page 6), `quests`, `talents` (the Talents page),
  `story` (the background story as it opens when a run begins; over a `--character` run or a
  `--replay` with `kit`; page 9 is reached with `--keys 9` from another page), `messages` (the
  message history).
- `describe` (what one carried stack is: the first, over the inventory).
- `equip-item` (the choice for the first slot, Head) over the equipment screen.
- `pick-up` (the pile on the player's cell), `drop` (the inventory in its drop mode, as `d` opens
  it), `throw` (what to throw, as `t` asks with no throwing weapon readied), `thrown-exploit`
  (what to throw in the thief's Thrown exploit), `eat`, `read` (the scrolls and grimoires
  carried), `identify` (for the first stack's scroll), `decipher` (the scrolls of a type not yet
  identified, as the Decipher ability asks).
- `temper` (the metal arms and armour carried that can be made finer, as the Temper ability
  asks), `whittle` (the wooden weapons carried, as the Whittle ability asks), `salvage` (what can
  be taken apart, as the Salvage ability asks), `make-do` (the damaged things carried, as Make do
  and mend asks), `cut` (the loose gems, as Jewelcraft's Cut asks), `give` (what fits a neck, as
  the Give ability asks).
- `shape` (the Spirit weapon's shape list), `scribe` (the Scribe ability's spell list), `rig`
  (Trapwright's rig list), `profession` (the first profession's recipes, Metalcraft's, as its row
  of the abilities menu opens them), `craft-pick` (the forging wizard of the first recipe with a
  choice, the dagger; its steps are reached from `profession` with `--keys`), and `abilities`
  (the abilities menu).
- `dialogue` over a run whose last action was talk with topics (a `--replay` ending in `chat:SE`
  or `ask:SE:rooms` next to the hub's hermit).

Over the main menu, or over the pause menu when `--scenario`, `--new`, `--replay`, or `--record`
names a run (`--capture` alone does not): `settings`, `files` (where the settings, saves, and
logs are, over Settings), `controls`, and `dialog` (the quit dialog).

### Dev flags for captures and replays

- `--fire` (dev): start the run aiming the readied missile weapon, as the Fire command (unbound
  by default; `t` covers it) would (captures of the targeting line; golden entries pass it with
  `args = ["--fire"]`).
- `--look X,Y` (dev): start in look mode with the cursor on level cell (X, Y) (captures of what
  look mode says, e.g. a shop's price).
- `--keys "KEYS"` (dev): press keys at startup, after any `--replay`, as if typed (single
  characters, or `Enter`, `Esc`, `Tab`, `Up`, `Down`, `Left`, `Right`), so captures show what
  they open: `--keys Left` at a shop's door owing (the pay dialog), `--keys "d b Enter"` (a sell
  offer).
- `--replay FILE`: apply a replay at startup (its scenario, seed, character, and name win; a hash
  divergence is logged as a warning), so `--capture` can screenshot any state.
- `--record FILE`: write every applied action (with state hashes) to a replay, rewritten after
  each action (replayed steps included).
- `--dev`: enables the dev keys in release builds (always on in debug: F2 reveal map (ground
  truth), F3 next vision profile, F4 light-value overlay, F5/F9 snapshots, F6 skip an hour
  (recorded as `skip:1h`), F7 next weather (recorded as `weather:KIND`), F8 identify everything
  carried (recorded as `identify`); all listed by `?`), and New run's seed field (without it New
  run draws a random seed and never mentions it, #256).

### Data, keys, and saves

- `--portable [DIR]`: keep everything in `warlock-data` next to the game (or DIR); a
  `warlock-data` folder there does it without the flag. `--paths` prints the locations and why,
  then exits. `--move-data native|portable [DIR]` (`--dry-run` to preview) moves files between
  portable and the platform folders.
- `--reset-keys` puts every key back to its default (saved at once), then starts as usual.
  `--export-keys FILE` writes the keymap (every command's keys by context, the format of
  Controls' Export keys, `warlock-keys.toml`) to FILE; `--import-keys FILE` reads one into the
  settings (the commands it lists only) and says what changed; both then exit.
- `--save-dir DIR` keeps saves in DIR instead of the platform data dir (`DIR/saves/<id>.sav`, one
  per character; with `--capture` or `--replay` only read).

## The headless dev loop: `warlock-cli`

No window, no Bevy; `--content-dir` works here too.

| Task | Command |
|---|---|
| ASCII dump after actions | `cargo run -p warlock-cli -- dump --scenario showcase --actions "move:N move:N" --layers view,fov,memory` |
| Dump a replay's end state | `... dump --replay runs/x.json [--actions "wait"] --format ascii\|json [--state]` |
| Create a replay | `... replay --create --scenario showcase --actions "move:N wait" -o runs/x.json` |
| Verify a replay | `... replay --verify runs/x.json` (exit 1 and first diverging step on mismatch) |
| Preview a generated level | `... gen --level depths:3 --seed warlock` |

- `dump` and `replay --create` take `--character dwarf,female,warrior` (starts as that
  character), `--name Brakka` (names them), `--talents healthy,lean` (picks their creation
  talents), and `--fallen Brakka:dwarf:stone,Tamsin:elf` (gives the run the graveyard's dead,
  `:stone` the petrified, to name statues (the petrified) and gravestones (all) from; out of the
  state hash; docs/dev/saves.md "The graveyard").
- `gen` prints ground truth: `*` = light placed on floor, `&` = locked door, `S` = secret door;
  `^` = a trap, hidden ones too; a layer boss's lair prints its `hidden stairs` line; `⊟` = a
  chest, locked or trapped or not; `V` = an ore vein, hidden in the rock too; a special room's
  floor by its effect's letter. It previews local areas of a campaign's overworld too, `--level
  local:49,23`, `--campaign showcase` the default.

## Replay files

JSON `{game_version, scenario, seed, character, name, kit, steps:[{action, hash}]}`:

- `character` = `{race, gender, class, talents}`, `talents` the creation picks (left out while
  none), left out for a run without one; a replay without it starts as the scenario's, so old
  replays load.
- `name` the character's name; a replay without it gives the run's default name.
- `kit` true when the run started with the character's starting kit, left out otherwise.
- `hash` is the FNV-1a 64 state hash (`World::state_hash`, of the canonical JSON encoding of
  `WorldState`) after the step, as 16 hex digits. Verification reports the first diverging step.

## Dump layers

`--layers` takes a comma-separated list. The layers built from the perception snapshot show
only what the player knows; `truth`, `actors`, `progress`, `rooms`, and `ice` show ground truth
and are **dev-only**.

### Map layers

- `view`: what the player sees.
- `fov`: `*` visible.
- `memory`: seen terrain, remembered objects (the top object of each known pile), and known
  traps over them.
- `light`: perceived brightness of visible cells as a digit, `0` dark to `9` fully lit or
  brighter.
- `sound`: heard sounds at their rough places over the view: `f` footsteps, `o`/`c` a door
  opening/closing, `x` fighting, `b` a bowstring, `m` chanting, `k` banging (a door kicked), `d`
  digging, `v` a voice, `h` a howl, `r` a roar (a dragon's breath), a creature's cries `z`
  buzzing, `g` giggling, `s` a hiss, `l` a rattle, `y` a scream, `w` a wail, `j` a crash of
  glass (a flask breaking), `q` a heartbeat, `a` an alarm bell, `e` an explosion, `t` a
  thunderclap, upper case when loud.
- `blood`: blood the player sees or remembers as a digit over the view, `1` a few drops to `9`
  soaked.
- `scorch`: fire's char, the same way, `1` a little soot to `9` charred through.

### The player's own (from perception)

- `inventory`: what the player carries, grouped by item class under `Class:` header lines as the
  screens group it (`Melee weapons:`), each stack by its position as action text names it with
  its numbers as the lists show them, `a long sword (+0 hit, 1d8)`, a spirit weapon marked
  `(spirit)`, a wand whose type is known with its rung, `(rung 2)`, a flask whose type is known
  with its throw's range, `(throw 6)`, a base item the affixes it truly holds against its
  capacity, `(holds 1/2)` (docs/dev/enchanting.md), and weighs, worn and wielded things marked,
  `b - a leather jerkin (being worn) (3.5 kg)`, and in a shop what is unpaid, `c - a dagger
  (unpaid, 45 gold) (450 g)`; then what they carry, their burden and its thresholds, and what
  they weigh, `carried 4.2 kg (unburdened: burdened over 14 kg, strained over 28 kg, overloaded
  over 42 kg), total weight 74.2 kg`.
- `equipment`: the player's slots by letter, `c Body: a leather jerkin (b)`, `-` when empty.
- `stats`: the player's name, race, gender, and class (`Name Brakka`, `Race Dwarf`, `Class
  Bard`; `Class Bard 30` at the class's cap; `Class Bard 30, Hunter 4` once called into a
  prestige class), age, birth date, and sign with a lucky day's title (`Age 27`, `Born 3
  Seedtide 1417`, `Sign Wood Salamander, the Racer`), then their own scores as the rules read
  them (`Strength 12`), then their alignment band, `Alignment Kind` (never the score), then their
  combat stance when it is not neutral, `Stance Frenzy`, then each guild they belong to with
  their rank, `Guild Weyfarers' Gilde, Tin`, then `Effects Bleeding, Diseased` when they suffer
  any, then `Mark the kobold`, the creature their openings are in (docs/dev/openings.md), then
  `Diseases filth fever`, the diseases they know they have, then what they resist in words,
  `Resistant to cold`, `Vulnerable to fire`, `Immune to poison`.
- `skills`: the player's trained skills in skill order, `Stealth 17`.
- `weapon_skills`: the player's weapon skills and fighting styles in file order with their rank
  and progress toward the next, `Long blades 2 (40%)` (at rank 10 the rank alone), the caster
  styles among them, `Sniper 2 (10%)` (JSON `caster_styles` has those alone), those in use
  marked `Duelist 1 (0%), in use`.
- `attacks`: the player's ready attacks as the character screen shows them: `Main hand long
  sword: +4 to hit, 1d9+2 slashing`, `Off hand dagger: ...`, `Missile short bow: +3 to hit, 1d6
  piercing, range 10`, a spirit weapon's ending `, pierces armour`; unidentified magic left out.
- `talents`: the player's talents in the order taken, `Hardy`, `Skill focus (stealth)`, `Weapon
  focus (long blades)`, a birth sign's gifts first, `Fleet-footed (sign)`, `Slight (sign,
  flaw)`, those their class gives marked `Haggler (class)`; never what a rank gives.
- `spells`: the spells the player knows with spell level, rank, and cost and what each does now,
  its roll and reach as they would cast it, `Fireball (level 11, rank 1, 26 mana): 4d6 fire,
  burns one in four, range 11, radius 2`.
- `recipes`: every profession's recipes as its screen lists them for the player, a
  `Metalcraft:` line, then `  Temper: a metal weapon or armour, an anvil, about 10 minutes (no
  anvil beside you)`.
- `abilities`: the player's abilities in the abilities menu's order, each with its source and
  what it asks for or why it cannot be used now: `Shield bash (weapon skill): a direction`,
  `Open lock (skill): not usable yet`.
- `traps`: the traps the player knows, one a line, `(12, 5) fireball trap`, `(remembered)` when
  out of sight; `no known traps`.
- `locations`: the overworld locations the player knows, one a line, `(45, 23) The Hall`, a
  hidden one only once revealed; `no known locations`, also on any other level.
- `quests`: the quest log, active first: name, stage, giver, objective, and status, `Rats in the
  arena (active, from the old hermit): Kill 3 giant rats in the Arena. 0 / 3 killed.`
- `story`: the player's background story, a paragraph a line as the story screen tells it, `no
  story` without one.
- `companion`: the player's animal companion as they know it, `Snel, a hound (grade 12) at (12,
  5): unhurt, order follow`, the cell only where they see it, `Snel, a hound (grade 12): with
  you` in the retinue, `no companion` without one (docs/dev/companion.md).
- `familiar`: the witch's familiar, in the same words; `no familiar` without one, `no familiar;
  Tibb, a cat, answers a calling` while the one remembered is dead (docs/dev/familiar.md).

### Ground truth (dev-only)

On a level with a danger, `truth` and `actors` start with `danger 3 population 2/6 spawn next
08:15:00`.

- `truth`: terrain, objects, and creatures as they are, hidden traps and secret doors (`S`) too,
  a sealed room's standing breach (`H`); after the map a line per chest, `chest 7 (hutch) at (12,
  5): shut, locked, holds 3 stacks`, a line per gravestone with its inscription, `grave at (7,
  3): Here lies …`, a line per zone, `zone fire by #0 until 08:01:00, harm 4: (30, 14)`, and
  hidden locations by their entrance's glyph.
- `actors`: every actor on the level with its hit points, mana, its resources with their
  progress (`resources rage 11/20 (0)`), and Defence and Protection (`hp 8/12 mp 0/0 dv 13`), its
  alignment (`alignment evil`; the player's score and band, `alignment 450 (good)`), its court
  (`court seelie`, a fey's), the player's mark (`mark #12`, docs/dev/openings.md), whether it is
  unseen (`invisible`, by its kind or a status now, or `shadowy`), whether it is `rooted`, its
  weight, base plus carried, what it carries and of that wears and wields (`equipped main_hand a
  spear`; magic items as they truly are, `an iron ring of the Bear (unidentified)`, a wand's rung
  `(rung 2)`, an anvil's work `(attempts 2: die to_hit)`), what its magic items do to it by
  effect id (`magic strength +1 light item_glow`), its burden and carrying limit (`unburdened
  (limit 31.5 kg)`), its weapon skills with rank and marks (`weapon skills spears 1 (0/30)`), its
  stats as score/potential (`St 12/15`), its trained skills with their marks toward the next
  point (`skills stealth 17 (2/5)`), its status effects with severities (`effects bleeding 2`),
  its diseases (progressions, checks won in a row, or `incubating`, and time to their next turn,
  `diseases filth_fever 2 (1 won) 0d17h`), stat damage (`damage Dx 2 To 1`), and drained points
  (`drained St 2`), the player's nutrition and hunger (`nutrition 3540 (not_hungry)`), its npc id
  and talk counts, if any (`npc elder; talk said 3, muttered 1, met`), whose ally it is with what
  made it one, its minutes left, and its order (`; ally of #0 (summoned, 4 min; follow)`,
  `(summoned; stay at (12, 5))`, `; retinue` while travelling with the player; `; risen, faction
  berserk, 9 min` for one that rose berserk, docs/dev/allies.md "The risen"), and what it resists
  in percent (`resists fire 25`); then all food on the level and in the packs of those on it with
  when it was made (a corpse: when it died) and its stage, `fresh`, `old`, `rotten`, or `gone`,
  which the player is never shown (`corpse rat_corpse at (2, 1): died 1444-11-11 08:01:12, 2
  hours ago, fresh`).
- `progress`: the level-up choices due, `due level 3: 1 talent`, `due level 30: 1 talent, 1
  bonus talent`; the talents the player may take now by id with their choices, `takeable
  healthy, weapon_focus:{bows,slings}`; the first kills that paid; each rolled quest's draw and
  the size of its pool now, `roll slug_charm scroll_of_lightning x1 of 4`; each calling of the
  run's campaign and where the player stands with it, `calling hunter: open`, `calling hunter:
  level 30 not met`, `calling hunter: not for a thief`, `called hunter at 30`; the named monsters
  dead, `slain fire_drake`; what an aristocrat's stipend holds, `stipend 412 gold: a fizzy pink
  potion`; a merchant's stake with their partner in hundredths, its last week, and the weekly
  bonus in basis points, `invest 98.24 gold, week 74260, bonus 0 bp`; the stat training marks
  with the price in force (the drop's for a negative pile), `stat marks St 120/160, To -30/75`;
  each known spell's rung of its roll's ladder with the score and the outside rungs, `fireball
  rung 2/4 (In 16; +1 Empower)`; the learning, `learning 36 (Decipher script 108)` (the skill's
  trained value) or `cannot learn: not trained in Decipher script`.
- `rooms`: the level's special rooms: the terrain with their plain ground drawn as the effect's
  `mark`, then a line per room, `s shadowed room at (34, 8)-(49, 13), entered`.
- `ice`: a line with the level's temperature and when its ice was last brought up to date, then
  the map with the thickness of the ice on its water in whole centimetres, `0`-`9`, `a`-`z` for
  10-35, `#` beyond, over the terrain.

### The ASCII header line

The turn, the time (`0001-01-01 08:00:00`), level, viewer, the player's `hp 17/17` and `mp 3/3`,
each resource they hold (`rage 11/20`), `dead` once the run has ended, `hunger hungry` unless Not
hungry, `dazzled` while strong light dazzles light-blind eyes, `allies 2` while the player has
allies on the level or in their retinue, `encounter pending: a band of kobolds (ambush, evade,
fight)` while an overworld encounter waits for an answer, the weather under the open sky
(`weather rain`), the `temperature` (`temperature -3.5 °C`), and the hash.

### JSON dumps

`--format json` carries: turn, game time (seconds and calendar fields), level, viewer, recent
messages, heard sounds, and, with `--state`, the full world state; and:

- `room`: the special room the player stands in, if they know it (its effect id; else null).
- `shop`: the open shop they stand in, `{sign, keeper, owed}`, else null (spec §7 "Shops").
- `weather`: under the open sky, else null; `temperature`: where the player is, °C to a tenth.
- Seen actors, each with its `health` (`unhurt`, `lightly_wounded`, `wounded`, `badly_wounded`,
  `nearly_dead`) and `effects` (those that show, by id), `known` (what the player's Knowledge
  tells of its kind, by damage type id, the word; left out while it tells nothing), `ally` (true
  for an ally of the player's), `marked` (true for the player's mark), and a composite animated
  thing's `bands` (its colours, top first).
- `hp` (`{current, max}`), `mana` (`current`, `max`), `resources` (the ones held, in resource
  order: `id`, `current`, `max`, and `on`, the mark a marked pool's points are in, else null),
  `defence` (the total), `protection`, `resistances` (by damage type id, the word: `{"cold":
  "resistant"}`), `weight`, `burden` (`level`, `limit`, the limit in grams), `alignment` (the
  band's id, or null without alignment rules), `guilds` (`id`, `name`, `rank`; left out before
  joining one), `hunger` (`bloated` … `starving`, or null when nobody hungers), `stance` (the
  combat stance's id), `dazzled`.
- `encounter`: the overworld encounter pending: `entry`, `name`, `outcome`, `title`, `text`,
  `choices` (`id`, `label`), or null; with `--state` also the dev `encounter_tables`, those
  matching the player's cell, and the dev `callings`, each of the campaign's with its `class`,
  `level`, `from`, and `status`: `open`, `level_not_met`, `not_from`, `called`.
- `effects` (status ids), `diseases` (the known ones' ids).
- `death`: `{turn, time, place, by, of, cause, trap}` or null; `of` the status that killed,
  `cause` when neither a creature nor a status effect killed them ("Starved to death", choking,
  "Killed by a fireball trap", "Fell through a collapsed floor", and the like), `trap` the id of
  the trap that killed, else null.
- `spells`: the spells known: `id`, `level`, `rank` (1 to 10), `cost`, `roll` (the menu's text,
  `4d6 fire`, or null), `damage_type` (`healed` for a heal, `called` for a summon's count, or
  null), `reach` (`range`, `length`, `radius`, `jumps`, each null where none).
- `known_traps` (`[{pos, trap, seen}]`), `known_hollows` (the hollow walls found: positions),
  `known_locations` (`[{pos, name}]`), `chests` (seen or remembered: `pos`, `band`, `state`,
  `locked` with Chest-wise, else null).
- `stats` (scores by stat id), `skills` (trained skills by skill id with their values),
  `weapon_skills` (by skill id: `rank`, `progress`, the progress a percent or null at rank 10),
  `caster_styles` (the caster styles alone), `styles_in_use` (style ids), `attacks` (`source`,
  `kind`, `name`, `to_hit`, `damage` (the text), `range`).
- `talents` (in the order taken: `id`, `name`, `choice`, `class`, `granted`, `flaw`).
- `character`: `name`, `race`, `gender`, `class`, `raised`, `age`, `born` (`{year, month,
  day}`), `sign` (`{animal, element}`), and `lucky`, a lucky day's title, only for one born on it,
  `prestige`, the prestige class once called, and `classes`, every class with its own level,
  `[{ class, level }]`; or null.
- `due`: the level-up choices due: `level`, `talents`, `rolls`, `bonus`, or null.
- `abilities` (in menu order: `id`, `source`, `aim`, and `unavailable`, the reason's id or null).
- `quests`: the log: `id`, `name`, `kind`, `active`, `completed`, `giver`, `target` (what a
  rolled quest drew, else null), `objective`, `status`.
- `story`: the background story's lines: `chapter`, `line`, `text`; `character.story` the ids,
  `parents/plain_folk`.
- `allies`: the player's allies wherever they are: `id`, `creature`, `name`, `health`, `order`,
  `at`, `source`, `lasts` (minutes, or null), `retinue`, `here`.
- `companion`: the player's companion: `id`, `name`, `creature`, `grade`, `health`, `order`,
  `at`, `pos` (where seen, else null), `retinue`; null without one.
- `inventory`, per stack in position order: its `letter` (the position as action text), object,
  count, weight, `class` (item class id), `name` (as the player knows it), `rarity`,
  `identified`, `magic` (`{level, name, affixes: [{id, tier, value}]}` once identified, else
  null), `numbers` (what it gives as the player knows it: `to_hit`, `damage`, `range`, `dv`,
  `pv`, each null where it has none; null for nothing), a wand's `wand_rung` once its type is
  known, a flask's `throw` (`range`) once its type is known, and an unpaid stack's price,
  `unpaid`.
- `equipment`: by slot id: letter, object, count.

## Action text syntax

Actions are whitespace-separated words; `ACTION*N` repeats one, e.g. `wait*10` (N at most 100,000, `action::MOST_REPEATS`; more is a parse error). The verbs:

`move:DIR run:DIR wait open:DIR close:DIR kick:DIR pick:DIR ascend descend use_tool
pickup[:LETTERS] drop[:LETTERS] sell[:LETTERS] pay[:DIR] equip:SLOT:LETTER remove:SLOT bash:DIR
fire:X,Y throw:LETTER:X,Y eat:LETTER eat:floor:LETTER craft:ID[:POS,...] give:LETTER[:DIR]
read:LETTER[:X,Y|:LETTER] scribe:SPELL dig:DIR search search:N disarm:DIR listen:DIR gather:DIR
wedge:DIR heal[:ID[:DIR]] feint:DIR intimidate:X,Y pray forage temper:LETTER whittle:LETTER
salvage:LETTER rig:ID:DIR decipher:LETTER drink use:ID[:DIR|:X,Y] chat:DIR ask:DIR:TOPIC
steal:DIR tame:DIR throw_voice:X,Y order:follow|stay|hunt:X,Y rest:GOAL encounter:CHOICE
stance:NAME vision:PROFILE talent:ID[:CHOICE] advance:SKILL levelup:auto xp:N call:ID
rank:SKILL:N skip:DURATION weather:KIND spawn ally:CREATURE[:N[:DURATION]] meet:ENTRY[:OUTCOME]
effect:ID@X,Y[>X,Y] zone:KIND@X,Y,R identify[:LETTERS] afflict:STATUS drain:N resource:ID
infect:DISEASE reveal:ID lure:ID quest_roll:ID:TARGET freeze:X,Y,R thaw:X,Y,R`

### Arguments

- `DIR` = N NE E SE S SW W NW.
- `X,Y` = a level cell.
- `LETTERS` = stacks by position, as letters for the first 26 (`a` = 0, the first, `z` the 26th)
  or numbers for any (`0,2,30`; mixed works too, `ac,30`), as positions past `z` must, e.g.
  `pickup:ac` (the pile on the player's cell, `a` its top) or `drop:b` (the inventory); none =
  everything (`pickup`, `drop`, `sell`, and the dev `identify`).
- `LETTER` = one position, `b` or `27` (`equip:body:27`, `eat:floor:30`, `throw:27:12,5`);
  positions are where stacks are stored, never the letters a screen's page shows.
- `SLOT` = a slot id in `assets/data/slots.toml`, e.g. `equip:body:b` (put inventory stack `b`
  on), `remove:body` (take off what is there).
- `NAME` (a stance) = aggressive, neutral, defensive, or coward (`assets/data/stances.toml`).
- `PROFILE` = an id in `assets/data/vision.toml`.
- `GOAL` = `dawn`, `dusk`, or a duration up to `24h`.
- Durations are a whole number with `s`, `m`, or `h`: `30s`, `10m`, `2h`.

### Moving, doors, and the level

- `move:DIR` steps that way.
- `run:E` runs east, step after step as one long action (in the open to whatever blocks the way,
  along a corridor through its bends to a branch, a side door, a dead end, or a room; it stops
  for whatever stops a rest; a first step a run cannot take is a plain `move`; docs/dev/time.md
  "Running").
- `search` searches the cells around the player for secret doors and hidden traps for a turn,
  and `wait` is another spelling of it (waiting is searching, one action: #1326; the game's
  Wait/search, Num5 or `s`); `search:N` for up to N turns (1 to 100), stopping when something is
  found (Alt+Num5 or Alt+s).
- `rest:GOAL` rests until dawn, dusk, or for a duration (`rest:1h`, `rest:dawn`).
- `open:DIR`, `close:DIR` a door; `kick:DIR` kicks that way (docs/dev/doors.md).
- `pick:DIR` picks the lock of the door that way (the Pick lock ability: Open lock trained;
  `docs/dev/doors.md`).
- `wedge:E` wedges the closed door east of the player shut for a turn (the Wedge ability, for
  those trained in Woodcraft: a creature that opens doors cannot open it and must kick it in,
  against the door's sturdiness plus Woodcraft's `wedge` points; the player opens it as usual; a
  door flag in the save; refused, with no time, without the skill, a closed door that way, or a
  door already wedged; `docs/dev/doors.md`).
- `listen:E` listens at the closed door east of the player for a turn (the Listen ability, for
  those trained in Listen: what comes through that door is heard at a threshold lower by
  Listen's `hearing` and 2; every sound of the turn is told, or that none was; marks Listen;
  refused, with no time, without the skill or a closed door that way; `docs/dev/hearing.md`).
- `ascend`, `descend` take the stairs.
- `dig:E` digs into the wall east with the tool in the Tool slot, turn after turn until it gives
  way or something interrupts (refused, with no time, untrained in Mining: a teacher sells it,
  `train_skill`, docs/dev/digging.md).
- `use_tool` uses the light ready in the Tool slot (a lantern: unlit, lit, hooded, unlit again;
  `u` in the game, which asks for a direction to dig instead without one).
- `vision:PROFILE` switches the vision profile (`vision:darkvision`).
- `stance:NAME` takes a combat stance, no time; `frenzy` is refused: only rage gives it.

### Items

- `pickup[:LETTERS]`, `drop[:LETTERS]` (docs/dev/objects.md).
- `sell:b` sells to the shop the player stands in and `pay` pays it, `pay:DIR` pays and steps
  out that way (spec §7 "Shops").
- `equip:SLOT:LETTER`, `remove:SLOT` (docs/dev/equipment.md).
- `eat:b` eats one of inventory stack `b`, `eat:floor:a` one of the top stack underfoot (a
  potion is drunk, and a magic crystal consumed, at once: a turn, `[consume]` of
  `data/crystals.toml`, mana to the maximum, Willpower marks, a step toward evil;
  docs/dev/enchanting.md).
- `read:b:12,5` reads scroll stack `b` at level cell (12, 5) (a spell scroll: fireball,
  lightning, frost, or any scroll of an aimed spell), `read:b` a scroll of a spell on oneself
  (haste) or the scroll of restoration, `read:b:c` reads it on inventory stack `c` (the scroll of
  identify), and `read:c` on a grimoire is a study of the spell it teaches, the grimoire kept,
  or, for a spell known below rank 10, a deepening to its next rank (rank times the learning
  time, always a check; docs/dev/spells.md "Ranks") (refused, with no time, for a spell known at
  rank 10 or a character not trained in Decipher script; docs/dev/spells.md "Learning").
- `decipher:b` makes out the scroll in inventory stack `b`, of a type not yet identified,
  without reading it (the Decipher ability, Decipher script's use: ten minutes, then the reading
  check of `world/decipher.rs`; made, the type is learned and the scroll kept, failed, the time
  is gone and it may be tried again; refused, with no time, for a known scroll or a stack that is
  no scroll; docs/dev/scrolls.md).
- `scribe:fireball` writes a scroll of a spell the player knows on the first blank scroll they
  carry (the wizard's Scribe: three times its mana and its grimoire's reading time, spent at the
  end; refused, no time, without a blank or the mana; docs/dev/scrolls.md).
- `give:b` fastens one of inventory stack `b`, something for a neck, on the player's kept beast
  (a companion or a familiar) beside them, and what it wore falls on its cell (the Give ability,
  the Allies group: a wear's time; `give:b:E` names the way to the beast; refused, with no time,
  with none beside them or a stack that fits no neck; docs/dev/companion.md "The collar").

### Fighting

- `bash:DIR` bashes with the held shield.
- `fire:X,Y` shoots the readied missile weapon at level cell (X, Y); `throw:c:X,Y` throws one of
  inventory stack `c` there (any weapon is thrown whole, badly: `t` in the game sends `fire` for
  a launcher, else `throw` for what the Missile weapon slot holds; docs/dev/ranged-combat.md).
- `feint:E` feints at the hostile creature east of the player (the Feint ability, Bluff's use,
  for those trained in it: a turn, the foe's Willpower check against `data/feint.toml`'s
  difficulty + their Presence modifier + Bluff; failed, it is Surprised, and the player's next
  blow finds it flat-footed; refused, with no time, without the skill, a foe there, or one with
  no mind (`Content::has_vitals`); `use:false_lead:E` is the Duelist's weapon active, False lead;
  docs/dev/skills.md).
- `intimidate:X,Y` threatens the hostile creature on level cell (X, Y), one the player sees
  within 6 (the Intimidate ability, for those trained in it: War cry's check at one foe,
  Frightened for more degrees the better their Intimidate; refused, with no time, without the
  skill or a hostile with vitals there; `data/intimidate.toml`, docs/dev/skills.md).

### Skills and places

- `disarm:E` tries to disarm the trap the player knows of east of them (the Disarm ability, for
  those trained in Disable device; refused, with no time, without the skill or a known trap
  there; docs/dev/traps.md).
- `pray` prays at the forgotten altar beside the player (the Pray ability, the abilities menu's
  Here group: a minute, then an outcome from `data/outcomes/forgotten_altar.toml`, and perhaps
  the altar spent; refused, with no time, with no altar beside them;
  docs/dev/special-rooms.md).
- `drink` drinks at the healing spring beside the player (the Drink ability, Here too: ten
  seconds, then an outcome from `data/outcomes/spring.toml`; a spring never runs dry; refused,
  with no time, with no spring beside them).
- `gather:NE` gathers herbs from the glowing fungus northeast of the player (the Gather ability,
  the Here group, for those trained in Herbalism: five minutes, then `1d2` + Herbalism ÷ 50
  herbs, each drawn by the level's depth from `data/herbs.toml`; cut short, nothing; the fungus
  never runs out; refused, with no time, with no fungus that way or untrained in Herbalism; spec
  §8 "Gathering herbs").
- `forage` forages for food in the wild (the Forage ability, Survival's use: turns of the
  `forage` action for `data/forage.toml`'s time, then a chance of food by the landscape and
  Survival; interrupted, it finds nothing; refused, with no time, untrained, or anywhere but on
  land of the overworld or a local area of it; docs/dev/skills.md "Foraging").
- `heal:bleeding` treats the player's own bleeding with the Heal skill (the Heal ability: a
  status id with a `treat` table, `heal:poisoned`, or a showing disease's id,
  `heal:filth_fever`; `heal:bleeding:E` the player's ally east of them; bare `heal` the player's
  first there is; refused, with no time, untrained, with no ally that way, with nothing to treat,
  or for a poison or disease already tried; docs/dev/status-effects.md).
- `steal:DIR` steals from the creature that way (the Steal ability; docs/dev/stealing.md).
- `tame:N` tames the beast north of the player (the Tame ability, for those trained in Handle
  animal: ten turns beside it, then a Willpower check; refused, with no time, with a companion
  already or no beast there that may be tamed now; docs/dev/companion.md "Taming").

### Crafting

All in docs/dev/crafting.md unless named.

- `craft:long_sword:c,f` makes a recipe (an id in a campaign's or the shared
  `data/recipes/<skill>/`) with the picks of its steps as positions, `-` a skipped optional
  (`craft:dagger:a,-`). Refused, with no time, untrained in the recipe's skill ("You are not
  trained in Metalcraft."), for a built-in one (Temper, Whittle, Cut, Set run their own actions),
  one beyond the player's Metalcraft, away from the anvil, or with picks that do not hold
  ("Making a recipe").
  - A forge recipe (the seven swords of `data/recipes/metalcraft/`, at an anvil) takes the metal
    pieces for its blade and an optional cut gem for its pommel, works its time, then a Strength
    check plus Metalcraft against its difficulty and the metal's `work`: Fine, Marvellous, or a
    Masterwork by the margin (`data/recipes.toml`), plain, or failed (half the stock back, the
    gem kept).
  - A counted recipe makes a batch instead, scaled by the margin: `craft:arrows:c,d` arrows for
    the bow ready (Woodcraft's, with a knife: the head, an arrowhead stack, a bone shard, a stone
    block, or `-` for none, brittle arrows, then a wood billet, no step for it in a forest, dense
    forest, or swamp: `craft:arrows:-` there), `craft:bolts:` the twin for a crossbow,
    `craft:arrowheads:c` twenty arrowheads from an ingot at an anvil, `craft:smelt:c` an ingot of
    its metal from lump of ore `c` at an anvil (Metalcraft's Smelt; ore from Mining,
    docs/dev/digging.md "Mining"), `craft:sling_stones:c` ten from a block of stone,
    `craft:flaming_arrows:c,d` ten from arrows and a flask of oil (the old `fletch` action is
    gone, #1336; "Counted recipes", "Arrows and bolts").
  - A brew, `craft:potion_of_healing` (Alchemy's herb and essence recipes,
    `data/recipes/alchemy/`; `craft:flask_of_fire` takes an essence of fire and a flask of oil),
    takes no picks: its herbs and essences come from the pack, twenty minutes, then an
    Intelligence check plus Alchemy; made, the potion, its type learned; failed, the herbs spent
    for a murky draught, a natural 2 a brewer's blunder; refused, with no time, beyond the
    player's Alchemy or short of an input ("Brewing").
  - `craft:cut:k` cuts loose gem `k` (Jewelcraft's Cut, the trained's: half an hour, then a
    Dexterity check plus Jewelcraft against 12 plus the gem's `work`; the margin by `cut_step`
    sets crude to flawless, an uncut gem losing a size on a failure or a crude cut, a re-cut
    never worse and always a size smaller, a natural 2 shattering it; refused, with no time, for
    a tiny cut gem, a flawless one, or no gem; "Cutting a gem").
  - `craft:set:a,k` sets cut loose gem `k` in item `a` (Jewelcraft's Set, from 10: twenty
    minutes, then Cut's check; made, the gem is the item's ornament part ("a ruby-pommelled long
    sword"); failed, the item untouched and the gem a cut rougher or a size smaller; a natural 2
    shatters it; refused, with no time, for an item with no ornament part, one set already, one
    worn or wielded, or an uncut gem; "Setting a gem").
  - Enchanting's (anywhere; docs/dev/enchanting.md "The recipes"): `craft:kindle:a,k` kindles
    plain item `a` with magic crystal `k` (an hour, then a Willpower check plus Enchanting against
    11 plus the roll's item level ÷ 3; made, one affix of the crystal's colour at the greater of
    the item's level and the enchanter's reach, identified; failed, the crystal spent; a natural 2
    warps the item); `craft:quicken:a,k` adds the affix magic item `a` of one affix lacks (from
    Enchanting 10, against 13; a rare with room takes a heart-crystal carried too and gains an
    affix of a kind with room); `craft:recast:a,k` rolls magic item `a`'s affixes anew, as many
    and of the same kinds (from 20, against 12; a rare refused;
    `craft:recast:a,k,-,-,-,0` keeps its first affix with a soul-fetter carried);
    `craft:imbue:a,k,l,m` makes plain item `a` of capacity 3 or more rare with three crystals,
    the first three affixes one of each crystal's colour in order, the rest of any of them, never
    more than its capacity, and a name (two hours, from 30, against 15); `craft:ennoble:a,k`
    makes magic item `a` of capacity 3 or more rare with one affix more and a name (an hour and a
    half, from 40, against 14); `craft:unbind:a` takes every affix off `a`, nothing back (half an
    hour, against 10; no crystal). After the crystals up to three seals, each an index in
    `data/crystals.toml`'s `[[family]]` list (`craft:kindle:a,k,4,17`), shut their families out
    of the roll, each a crystal more of the colour from those picked and half an hour
    (docs/dev/enchanting.md "Seals and the rare materials"); a whole stack of ammunition at once;
    refused, with no time, untrained, for an item worn or wielded, unidentified, a unique, with
    no room ("It holds all it can."), or nothing of the colour left to roll.
- `temper:c` works inventory stack `c`, metal arms or armour, at the anvil beside the player (the
  Temper recipe, Metalcraft's, on its profession screen, which walking into the anvil opens: ten
  minutes, then a Strength check plus Metalcraft, harder for every attempt ever made on the item:
  made, an improvement, up to three, or a damaged item repaired; failed, the item damaged, or a
  damaged one destroyed; `data/temper.toml`; refused, with no time, untrained in Metalcraft, with
  no anvil beside them, a stack not of metal, or one with three improvements).
- `whittle:c` whittles inventory stack `c`, a weapon of wood (a bow, a staff, a club), with a
  knife carried, anywhere (the Whittle recipe, Woodcraft's, beside the arrows on its profession
  screen: Temper's rules and `data/temper.toml` for wood, a Dexterity check plus Woodcraft's
  `whittle` points; refused, with no time, untrained in Woodcraft, without a knife, a stack not of
  wood, or one with three improvements; docs/dev/skills.md "Whittling").
- `salvage:c` takes one of inventory stack `c` apart for its materials, anywhere, with nothing
  (the Salvage ability, every profession's, in the Skills group: ten minutes, then a check per
  material a profession works, by its skill's stat plus that skill's `craft` points against
  `data/salvage.toml`'s difficulty plus the material's `work`; made, half its weight back as
  stock, a gem whole; failed, lost, a gem a cut or a size less and shattered only on a natural 2;
  the item gone either way; refused, with no time, for a stack worn or wielded, protected,
  unpaid, a spirit weapon, or with nothing a profession works).
- `rig:tripwire:E` rigs a trap on the cell east of the player (the craftsperson's Trapwright,
  from level 25: a rig of `data/rigs.toml`, its `seconds` of `craft` turns, then a Dexterity
  check plus Woodcraft's `craft` points; made, the trap placed, known, and rigged, its kills the
  player's, the inputs spent (a tripwire up to 3 arrows, stakes a wood billet with a knife
  carried, a pitfall a digging tool ready, kept); a natural 2 springs it on them; refused, with
  no time, without the ability, a direction, a cell that may hold it, a wall within 2 for a
  tripwire, or what it takes; docs/dev/traps.md "Rigging").

### Abilities and spells: `use:ID`

- `use:second_wind` uses the Second wind talent (hit points back at once, then Winded) and
  `use:blood_magic` the Blood magic talent (hit points for mana, never the last), a talent's own
  active.
- `use:rage` uses a class ability (spec §8 "Class abilities"; `use:smite:E` one aimed in a
  direction): rage, veterans_wind, lay_on_hands, smite, holy_radiance, quivering_palm, exploit
  (`use:exploit:E`, the thief's: every opening held spent on one sneak attack,
  `docs/dev/openings.md`), inspire, lullaby (the bard's song over every hostile that hears it,
  docs/dev/hearing.md), cackle (the witch's: her hexes in sight a degree longer), knock
  (`use:knock:E`, a lock), field_forge (`use:field_forge:E`, the craftsperson's: an ingot carried
  set on open ground there in an hour, a makeshift anvil for good, `docs/dev/crafting.md`),
  orison (the priest's prayer at every tier, `docs/dev/prayer.md`), sanctuary (the priest's
  hallowed ground, `docs/dev/special-rooms.md`), call_familiar (the witch's hour's sitting,
  `docs/dev/familiar.md`).
- One aimed at a cell: `use:volley:12,5`, `use:sic:12,5` (Sic 'em: a hostile the player sees,
  their companion set on it, docs/dev/companion.md); `use:charge:12,5` the warrior's Charge: a
  rush along the line to a hostile the player sees and a blow, for 1 stamina and 1 a cell, no
  free blows provoked, docs/dev/progression.md.
- `use:make_do_and_mend:c` one on inventory stack `c` (the craftsperson's Make do and mend: a
  damaged thing mended anywhere, ten minutes, then a check that destroys nothing; refused, no
  time, for one worn or wielded or of a material no profession works; docs/dev/crafting.md "Make
  do and mend").
- `use:thrown_exploit:c:12,5` one that throws one of inventory stack `c` at a cell (the thief's
  Thrown exploit, from 18: Exploit's blow as a throw at the creature there, within the throw's
  range and the sneak dice's reach, `docs/dev/openings.md`).
- A birth sign's active the same way, `use:horns_grace`, `use:blaze`, `use:hounds_bay`, or aimed
  at a cell, `use:transfixing_stare:12,5`, `use:fire_breath:12,5`.
- A spell the player knows too, by its id in `assets/data/spells/`, paid in mana, `use:haste`,
  or aimed at a cell, `use:magic_missile:12,5`, or cast on inventory stack `c`,
  `use:identify:c` (any `use:ID` that is no class or birth sign ability is a spell).
- A class active in a weapon's shape by the object's id, `use:spirit_weapon:long_sword` (the
  Hunter's Spirit weapon: a weapon of spirit-stuff in the main hand, for a wield; it fades when
  it leaves the hand, `docs/dev/equipment.md`).
- A weapon active after those, for its stamina and time, `use:lunge:E` aimed in a direction (in
  upper case: a lower-case letter is a stack), `use:shield_wall`, or aimed at a cell,
  `use:piercing_shot:12,5` (docs/dev/weapon-abilities.md).

### Talk and allies

- `chat:E` talks to the creature east of the player, `ask:E:rooms` asks it about topic `rooms`
  (an id in a campaign's `data/dialogue/`; quest topics are `quest-ID`, ids in a campaign's
  `data/quests/`: asking gives the quest, reminds, or hands it in, there is no accepting or
  declining (#422); a service is asked for by its id the same way, `ask:E:cure`, paid in gold and
  taking a chat's time (`ask:E:train_mining` Alberich's, a skill taught, offered only while
  untrained in it), and a trader's one offer too, `ask:E:wares` buying it (the pedlar's;
  docs/dev/npcs.md "A trader's offer"), and a calling's, `ask:E:life_before_death`, offered only
  while the calling is open, calling the player and taking a chat's time (the strange spirit's;
  docs/dev/npcs.md "Callings by topic"); docs/dev/npcs.md).
- `throw_voice:X,Y` throws the player's voice to level cell (X, Y) (one they see, within 6; the
  bard's, refused without the ability).
- `order:follow`, `order:stay`, and `order:hunt:X,Y` give every ally of the player's on the level
  an order (Come to heel, Hold here, Set on a cell the player sees: the Allies group of the
  abilities menu; no time, a voice's noise; refused with no ally on the level;
  docs/dev/allies.md).
- `encounter:fight` answers the overworld encounter pending (`CHOICE` = `ambush`, `bluff`,
  `evade`, `fight`, `hail`, `hunt`, `pass`, as its outcome offers; `bluff` talks past a party
  flagged `talk_past` when met, for one trained in Bluff; while one pends every other action
  that takes time is refused; docs/dev/encounters.md).

### Level-ups

- `talent:hardy` takes a talent (an id in `assets/data/talents/`; `talent:weapon_focus:long_blades`
  with its choice) when a level's pick is due (every third level; until it is made every other
  action is refused).
- `advance:stealth` spends one of a level's skill rolls (on a trained skill;
  `world/skill_roll.rs`).

### Dev actions

None takes game time unless said.

- `levelup:auto` spends every choice due with the first the player may take.
- `xp:2500` grants experience with the levels it brings (`xp:461` brings a 100 % character to
  level 3).
- `rank:long_blades:5`: a weapon skill or style at that rank, 0 to 10, and saying what it
  teaches, as a rank risen would (docs/dev/weapon-abilities.md).
- `call:hunter`: the player is called into prestige class `hunter`, an id in
  `assets/data/prestige/`, whatever their level, through the calling of the run's campaign that
  opens it to their class: the class is theirs and exactly the experience the first prestige
  level costs with it, so the level-up comes at once; refused with no such class, one held
  already, or no calling open to their class (docs/dev/experience.md "Prestige classes").
- `skip:DURATION` skips hours of game time (the dev hour skip; F6 records `skip:1h`).
- `weather:KIND` is the weather override, `KIND` = `clear`, `overcast`, `rain`, `heavy`, for
  three hours of game time.
- `spawn`: one attempt to bring a creature to the current level now, whatever its cap and chance
  (refused at danger 0; spec §7 "Danger levels").
- `ally:wolf`, `ally:wolf:2`, `ally:wolf:2:10m`: N (1 to 12) allies of the type beside the
  player, following, on their side, their stay ending after the duration if given (on the
  overworld they join the retinue at once); refused without `data/allies.toml`, at the cap
  (`[allies] most`), or with no free cell (docs/dev/allies.md).
- `meet:kobolds:ambushed`: entry `ENTRY` of the run's campaign's encounter tables met now on the
  player's overworld cell, by the lookouts or with `OUTCOME` (`drop`, `met`, `ambushed`, `pass`)
  forced; refused off the overworld or a cell with a local area.
- `effect:ID@X,Y[>X,Y]` shows spell effect `ID` (an id in `assets/data/effects.toml`) as the
  player perceives it, for captures: `effect:fireball@30,14` (a burst there),
  `effect:fireball@20,10>30,14` (flying there first), `effect:lightning_bolt@21,17>58,17` (a bolt
  toward the second cell, cut short by the first opaque cell); it hurts no one, has no rules
  effect, and only says so in the message log.
- `identify[:LETTERS]` identifies magic items and labelled scrolls' types; without letters
  everything carried (F8 records it).
- `afflict:STATUS` gives one degree more of `STATUS`, an id in `assets/data/status.toml`, e.g.
  `afflict:poisoned*4 afflict:bleeding` (not `diseased`: that comes of a disease; nor `raging`:
  that comes of `resource:rage`).
- `drain:N` the player spends N mana (1 or more) through `World::spend_mana` (as an ability
  would: Frugal caster's cut, a Willpower training chance), refused when they have less, for
  captures (`docs/dev/mana.md`).
- `resource:ID` the player holds `ID`, an id in `assets/data/resources.toml`, at its start,
  through `World::grant_resource_with_status` (with the status it brings: Raging), e.g.
  `resource:rage wait*3` (`docs/dev/resources.md`).
- `infect:DISEASE` the player catches `DISEASE`, an id in `assets/data/diseases.toml`, as if they
  had lost the check (it incubates, then shows), e.g. `infect:the_shakes rest:24h*3`.
- `reveal:ID` hidden location `ID` of the campaign (a legend's `hidden`, on the overworld or a
  hand-made level, `docs/dev/overworld.md`) is revealed and marked on the map as a quest would.
- `lure:ID` active lure quest `ID`'s creature comes beside the player, aware of them (made if it
  is nowhere), e.g. `ask:E:quest-crystal_guardian` on Ysella then `descend
  lure:crystal_guardian ascend` (`docs/dev/quests.md` "Lure quests").
- `quest_roll:ID:TARGET` active rolled quest `ID`, with no progress yet, draws `TARGET` instead (a
  creature, `tag:` a tag, or an object id its pool can hold, wherever the player can go, or a
  variant id; its kills count from then, a variant's reveal is made; refused otherwise;
  `docs/dev/quests.md`).
- `freeze:X,Y,R` and `thaw:X,Y,R` are the frost and heat bursts (radius 0 to 20 around level
  cell (X, Y), in its line of sight): the frost freezes the water there to 10 cm of ice, the heat
  melts the ice and whoever stood on it falls in (docs/dev/temperature.md).
- `zone:KIND@X,Y,R` (`KIND` = `fire`, `fouled`, `slick`, `dark`, `smoke`; by the player, for ten
  turns, a fire's harm 4, within radius 0 to 20 in its line of sight; refused on the overworld;
  `docs/dev/spells.md` "Zones").

# Content files and levels

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/07a-levels.md and spec/07b-data.md. Keep it current with the code, as CLAUDE.md says.

Content (`assets/campaigns`, `assets/data`, `assets/vaults`; test fixtures may also insert
`levels/` and `scenarios/` at the top) is TOML. `warlock-core` embeds none of it: it takes the
files as input, a `ContentFiles`, and `Content::load` builds the rules' tables from them (#1587).
The small crate `crates/warlock-assets` embeds `assets/` at build time with its `build.rs` (the
test world, `campaigns/showcase/`, only with its `showcase` feature: never in a release;
`docs/dev/campaign.md`), and the binaries (the game, the CLI, xtask) load it with
`warlock_assets::files()` (or `content()`), or a directory with `warlock_assets::from_dir`
(`--content-dir`). So a data edit rebuilds that crate and the binaries above it, never the core;
warlock-core's own tests read `assets/` from disk instead (`ContentFiles::builtin` and
`Content::builtin`, test-only, once per test binary; the tests share one parsed copy,
`test_support::CONTENT`, #1509), and its integration tests do the same with
`ContentFiles::from_dir`. Extra files laid over a set (mods, later: #196) go in with
`ContentFiles::insert` before the load, which checks the whole set ("The loader's whole-set
checks" below). A world's levels, scenarios, branches, quests, dialogue, and own creature types live
in its campaign, `assets/campaigns/<id>/` (`docs/dev/campaign.md`). Level maps are ASCII with a
legend; see `assets/campaigns/showcase/levels/hub.toml` and spec §7. A legend entry with a `creature` may set `faction = "peaceful"` (any id in
`data/factions/`) to override its type's faction for that spawn (`Actor::faction_override`; read it with
`Actor::faction`, never the creature type's, when computing stance), and `alignment = "good"`
to override its type's alignment the same way (`Actor::alignment_override`, read with
`Actor::nature`; `docs/dev/alignment.md`), and `status = ["blessed", "weakened*2"]` to place it
with statuses (`CreatureSpawn::status`, `level.rs::parse_statuses`: a status id, `*N` degrees,
default one; unknown or over its `max` refused; added in `travel.rs::make_creature`
beside a den's `asleep`; the test world's `circles` hall, spec §4.8 "Spell circles"). The showcase (hub, vast, courtyard) is safe
this way: `xtask/src/vast.rs` emits it for its rats and kobolds; fighting is the arena's and the Depths'. A level
file's optional `arrival = "..."` line is said when the player arrives by stairs (a dungeon
branch's on its first level, `Content::arrival`; a scenario's `arrival` once, as a run of it
starts). A legend entry's `name` (and `note`) names the place on
its cell (`docs/dev/overworld.md`, "Named places"); a vault's are stamped with it
(`docs/dev/mapgen.md`). A vault's legend entry may roll `loot` when the level is generated: a
grade (`loot = "magic"`: `any`, `magic`, or `rare`, anything of at least that) or a pool (`loot = {
pool = ["armour", "rings"], grade = "magic" }`, `grade` optional, default `any`; `pool` one name or
a list). A pool's names are item class ids or the `group`s of `data/item_classes.toml` (`weapons`,
`armour`, `jewellery`, `money`); `LootPool::named` refuses at load a name that is neither; a pool
that can hold nothing of its grade leaves its cell bare (`magic/pool_tests.rs`). Use one
cell per find; prefer a pool to a static `[[pile]]` in a generic vault. A vault's legend entry may
draw `monsters` too (`content/monster_pool.rs`): `monsters = { pool = "level", count = 20 }`
(`"level"` the floor's population, or tags: `pool = "goblin"`, `pool = ["orc", "troll"]`; `count`
over the entry's marked cells, default one on each, never more than they hold; refused at load:
`MonsterPoolSpec::resolve` (`"level"` beside tags, a `count` of 0), `parse_level`'s checks; a tag
no drawable type carries loads and brings no one; docs/dev/creatures.md for the draw and the cap). Beside `monsters` the entry may give `status`, `carry`, and `route`
(`level.rs::pool_cell` parses them into `VaultMonsters::{status, carry, route}` with the placed
creature's parsers; the route is checked from every marked cell, `validate_walk`, and turns with
the vault, `vault_turn.rs`; a pool's `carry` key counts in `LevelTemplate::has_key_for` when every
marked cell is within reach); a `faction`, `alignment`, `npc`, `animated`, `wield`, or `wear`
there is refused. A vault that dangerous should be avoidable and pay better loot. A vault
file's `turns = false` keeps it as drawn when one is drawn at random (default: any of eight ways,
`docs/dev/mapgen.md`): for text that depends on direction or a layout that must not turn; refused on
a level. A vault file's `danger = { from = 8 }`, `{ to = 6 }`, or both (`content::DangerRange`,
each end included; absent, every danger) keeps a vault drawn at random to floors of that danger
(`Vault::suits`, docs/dev/mapgen.md): give a deep vault, one whose pool's shallowest type is
past danger 1, its `from`, and a shallow one its `to`. The level parser reads a top-level
`danger` as a level's rule first (`level::DangerField`), then as this range, which
`vault::parse_danger` reads again; refused at load: on a level, on an outdoor vault, on a vault
a branch level names, a level's own rule (`danger = 3`) on a vault (the ends are the
author's, #1205), and a table that is no range (`{ form = 8 }`, `{ from = 300 }`: a vault's range is read
before the level parser, so the error names the field, #1588) (`mapgen/vault_danger_tests.rs`). `secret =
true` (or a difficulty) on a closed door makes it a secret door: the cell shows the wall around
it until found (`docs/dev/searching.md`). Water: `~` is still water; running water (terrain with a
`flow` anim) needs a flow on every cell from its legend entry (`flow = "S"`, optional
`flow_speed` percent), and the shared legend draws it as arrows (`→` normal, `⇒` double speed).
Flow is presentation data carried by `LevelTemplate::flow` / `LevelState::flow` and read by the
renderer through `PerceptionView::flow` (visible cells only); no rule reads it. Deep water
(`terrain = "deep_water"`, no shared legend glyph: a level names it in its own legend) is `swim =
true` with `walkable = false`; the loader refuses `swim` on anything walkable, solid, freezing, or
a door, stairs, chest, breaking wall, light, or overworld terrain (`docs/dev/skills.md`).
`data/rules/swim.toml` holds a swim's numbers.

## The loader's modules (`content/`)

`content/mod.rs` holds the module list and re-exports, `ContentError`, the `Content` struct
(`#[derive(Default)]`: an empty set, but for sizes and creature rarities, which `Content::load`
seeds), and `Content::load`, which reads every file in its order (a table before what names it).
The rest is split by topic (#1519): `files.rs` (`ContentFiles`), the per-table loaders in
`load_world.rs`, `load_creatures.rs`, `load_items.rs`, and `load_status.rs`, the cross-checks in
`validate.rs`, `fingerprint_for` in `fingerprint.rs`, the accessors in `access_world.rs`,
`access_creatures.rs`, `access_items.rs`, `access_character.rs`, `access_magic.rs`,
`access_status.rs`, and `access_rules.rs`, and the loader's tests in `tests.rs`.

A **small rules file** (one optional file read whole into one value: `data/rules/steal.toml`,
`data/rules/swim.toml`, ...) implements the `RulesFile` trait beside its own `parse`, in its own
module: its `PATH` and a `parse(path, text, &Content)` that passes on what its `parse` needs. Adding one
takes its type's impl, a `Content` field, an accessor, and one line in `Content::load`:
`content.x = content.load_optional(files)?` (an `Option`, `None` without the file) or
`content.load_or_default(files)?` (the type's default without it).

The small rules files of one system each live in **`assets/data/rules/`**, one file per system,
named by it (`steal`, `feint`, `patron`, `stipend`, `decipher`, `salvage`, `search`, `smite`,
`enchanting`, `intimidate`, `invest`, `swim`, `temper`, `tuning`; the owner, #1541): one folder
keeps them together while two PRs adding a system still touch different files, and a mod may
override one file. A new one goes there; a rules file that is a table (`data/status/`) or that
several systems read (`data/actions.toml`, `data/traps.toml`) stays where it is.

**Tuning numbers live in data** (the owner, #1541): a number a designer would tune (a check's
difficulty, a range or reach, a chance, a duration, a to-hit penalty, a multiplier, a floor or
cap that is balance and not an invariant) is never a Rust `const`. It goes in the data file of
its rule when there is one (a table or key of `data/traps.toml`, `data/hunger.toml`,
`data/miscast.toml`, a talent effect's field, ...), else in a table of `data/rules/tuning.toml`
(`content/tuning.rs`, `Content::tuning`: one table per rule, every key required, each checked
against its bounds at load), and the rule reads it from `Content`. The wiki's tables read the same
value, so they cannot drift. What stays in Rust is structural: units and scales (`time::HOUR`,
`dig.rs::WORK_SCALE`), array sizes and id ranges, hash constants, search bounds of an algorithm
(`spawn.rs::TRIES`, `swim.rs::SHORE_REACH`, `trap.rs::OVERSHOOT`), path costs of the AI
(`ai.rs`), the engine's coarse-turn stride (`stride.rs::STRIDE`), sentinels
(`weapon_active.rs::ALL_PROTECTION`), caps that are invariants of a format or a rule
(`seal.rs::MOST_SEALS`, `enchant.rs::RARE_LEAST`, a rare's three affixes),
the dev actions' own numbers, and the balance probe's sample sizes. A rules file's `Default`
(for bare test content without the file) repeats the numbers the file held when it was made;
the file is what the game plays by.

Creature types made in code (the animated objects, the mimics) are `CreatureSpec` values built
with `..CreatureSpec::default()` (#1552), whose `Default` is serde's field defaults (a test in
`content/creature.rs` holds them together), and go through `load_creatures` like a file's.

## Content directories

The tables that grow entry by entry are directories, so two PRs adding an entity never touch
the same file: `assets/data/creatures/`, `objects/` (base items included), `materials/`,
`affixes/`, `talents/`, `classes/`, `uniques/`, `weapon_abilities/`, `skills/`, `status/`,
`terrain/`, `factions/`, `room_effects/`, and the talent trees and systems in `talents/trees/` and
`talents/systems/` (each campaign's too), **one file per entity, named by its id** (classes as
`NN-id.toml`: their order is the order New run offers them, so the number keeps it; the
background story's chapters in `background/` the same, `NN-<chapter>.toml`, docs/dev/background.md;
statuses, terrain, factions, room effects, and talent trees and systems as `NNN-id.toml` in steps
of 10 (#1525): their ids are indices in their order, which saves, state hashes, the status bar,
and the special rooms' draw read, so the number keeps the order the single files had and a new
entry takes a free number where it belongs; skills were already by id, so they are `id.toml`)
(`creatures/kobold.toml`,
`affixes/of_the_bear.toml`). A family file holds several entries only for a tight set of
variants always edited together, named after the family (`objects/arrows.toml`: the arrow, the
flaming arrow, and the burning arrow it flies as). Each directory's `README.toml` is comments
only (the field documentation and notes for authors; a file of comments defines nothing, the
loaders take a missing array as empty). `warlock-assets`' `build.rs` embeds and
`ContentFiles::from_dir` reads `{campaigns,data,vaults}` recursively through one walk,
core's `content/walk.rs` (the build script includes it with `#[path]`), both in sorted path order
and both leaving `campaigns/showcase/` out without the `showcase` feature (`from_dir` takes it as
an argument, `warlock_assets::from_dir` passes the build's, so `--content-dir assets` on a release
build has no test world either; #1560; `the_embedded_copy_is_the_assets_directory` checks the
embedded set against the disk), and a table loader reads `ContentFiles::table(name)`: `data/<name>.toml` first, if there is one (a
whole table in one file is still accepted, so test fixtures that insert it keep working), then
`data/<name>/**/*.toml` in path order, then each campaign's `data/<name>/**/*.toml`
(`ContentFiles::campaign_order`: playable campaigns, then test ones). `quests`, `dialogue`, and
`dungeons` are tables too: their single files hold the field reference and the shared entries
(the shopkeeper's dialogue), the campaigns their own. `data/plurals.toml` (the plural dictionary, `content/plural.rs`) is
no table: the shared file, then each campaign's `data/plurals.toml`, merged into one, read
first so creature, object, and gem names without a `plural` take theirs from it. Ids (`CreatureId`, `ObjectId`, `MaterialId`, an affix's
index) are indices in that order and feed the content fingerprint, so moving entries between
files changes ids and invalidates saves (the release bump covers it). Besides ids, one group of
pairs feeds it: `data/circles.toml`'s rows as status and school (a school decides the light a
circled status sheds, `circles::shed_lights`; docs/dev/status-effects.md). Each file is parsed on its
own, so an error names the file the bad entry is in; **a duplicate id across files is a
`ContentError` naming both** (`content::duplicate_id`). For affixes the scalar tables (`[rarity]`,
`[magic]`, `[rare]`, `[names]`) stay in the settings file `data/affixes.toml`, each set in
exactly one file of the table (a second file setting one is an error naming both), and the
entry files hold `[[prefix]]`/`[[suffix]]` only. `ContentFiles::remove("data/creatures/")` drops
a directory; `ContentFiles::edit_table(name, edit)` rewrites every file of a table (tests that
change one built-in line; the test-only `replace_in_table`, `insert_builtin_table`, `table_text`,
and `replace_table` do the same by a line, copy a shipped table into a fixture, or swap a whole
table for one file). The tables with settings keep them in their single file, each set in at
most one file of the table (a second is an error naming both): `[healing]` in
`data/status.toml`, `[ladder]` and `[rolls]` in `data/skills.toml`, `chance` and `per_level` in
`data/room_effects.toml`. Every other `data/*.toml` file is a single file whose order means
something (slots, stats, weapon skills, actions, ...): never split those.

**Nothing visible follows the load order of these four tables.** Wiki tables sort by an
explicit key (`xtask/src/wiki/order.rs`: objects by class, weapon skill, value, name; creatures,
materials, and affixes by name), the shop stock a generated shop draws from
(`mapgen/shop.rs`) and the bases and affixes loot rolls from (`magic/loot.rs`) are sorted by id
before the draw, and "shoots arrows" names `Content::ammunition(kind)`, the first ammunition of
the kind by id. A new list over one of these tables gets a sort key too.

## The loader's whole-set checks

`Content::load` checks the set of files it was given, whichever it is (the built-in copy,
`--content-dir`, or files a test or, later, a mod lays over it), so none of these depends on the
embedded copy:

- **Every file is read** (#1530). `ContentFiles` records each path a loader looks up
  (`get`, `get_optional`, `table`, `in_dir`); after the load, a file nobody read is an error
  ("no loader reads this file"), `README.toml` files excepted. A misnamed file
  (`data/feints.toml`) or directory (`data/town/`) is a load error, not a feature quietly off.
  A new loader reads its files through those accessors; `ContentFiles::edit_table` reads
  without counting.
- **The ids the engine names** (#1546) are listed once in `content/required.rs` (the creature
  `player`; the statuses `staggered`, `bleeding`, `poisoned`, `webbed`; the effect
  `frost_burst`; the terrain `stairs_down`, `rubble`, `makeshift_anvil`; the object `bones`;
  the slot `neck`) and resolved at load into `Content::ids()`. Each is required while its table
  or the feature using it is loaded (`staggered` while a room effect collapses, the other
  statuses while there is a prayer, the creature while there are creatures, the effect while
  there are effects, `bones` while a room is a teeming den, the Pit's terrain while a branch
  has a `portal`, the anvil while a talent gives Field forge, the neck while companions
  or familiars are), else `None`: rules code reads `content.ids().staggered`, never the string.
  A new id the engine names goes into that list.
- **`data/gear_condition.toml`'s keys** (#1556): each `[kinds]` key is some material's `group`,
  and each `[[tribe]]`'s `tag` some creature's.

## Words

The small text helpers live once, in `warlock_core::text`, and core, the app, and the wiki
generator all call them (#1543, #1548): `capitalized` (a first letter in capitals), `article`
(`a` or `an`, lower case), `and_list` and `or_list` (a list with the Oxford comma, "a, b, and c",
the owner's choice for all game text), and `number_word` ("no", "one" to "twelve", digits
beyond). The app keeps its wall-clock helpers in `crates/warlock/src/utc.rs` (`since_epoch`,
`utc_parts`; #1571) and its cut-to-width helpers beside `wrap` in `ui/menu.rs` (`truncated`,
`truncated_head`). The wiki generator writes every percentage through `xtask/src/wiki/util.rs`
(`percent`, `percent_of`: `12.5%`, no space, a tenth from 1% up; #1577).

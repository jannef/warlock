# Warlock — Campaign: the game world, its overworld, the first town, the nearest dungeons, danger levels

Design document for the owner's review (2026-10-25). Written against `origin/main` at 953fd30b. Nothing here is implemented; §13 cuts it into issues.

## Contents

1. Summary
2. Research, tied to the decisions
3. Campaigns: the seam between worlds
4. Keeping the test world out of release builds
5. The overworld
6. The first town
7. The dungeons nearest the town
8. Hidden locations revealed by quests
9. Danger levels and creatures that keep coming
10. The creatures the first dungeons need
11. What the player sees (messages, the map, the quest log, dumps, the wiki)
12. Data and code sketch (files, state, hooks, determinism, tests)
13. Implementation plan
14. Open questions for the owner

## 1. Summary

**What the owner asked for**: start the game world. Everything built so far (the hub, the arena, the courtyard, the quarry, the trap hall, the Frost Grotto, the vast level, the Depths, the Wildlands overworld, and the nine scenarios; `spec/03-showcase-world.md`) stays, for tests: it is one test world with several start locations. The game gets a **second world with no connection to it**, and **test scenarios do not appear in release builds**, only in dev and CI builds. In the distant future there may be several full campaigns that share no world; this document designs the seam and builds no more of the machinery than the second world needs. To build now: an **overworld** entered ADOM-style through a **mountain pass**, with a **market town** the closest location to the entry, a **volcano, mountain ranges, swamps, rivers, and deep forests**, and room for **about 40 locations** without crowding or sprawl; the **first town**; the **dungeons closest to it**, procedurally generated, **two of them hidden until a quest reveals their location** (at least one visible from the start); and on every dungeon level a **hidden danger level** (ADOM's) from which **monsters of that level keep spawning over time**.

**The proposal in one table:**

| Piece | Decision |
|---|---|
| Campaigns | a directory per world under `assets/campaigns/<id>/` holding what is the world's own (its overworld and levels, scenarios, dungeon branches, quests, dialogue, and any vaults of its own) with a `campaign.toml` (name, description, the scenario New run starts, `kind = "test"` for the showcase); everything else (`assets/data`, `assets/vaults`) stays shared engine data. Ids stay global and unique across campaigns (a duplicate is a content error naming both files). A run records its campaign (`WorldState::campaign`), local areas and the content fingerprint resolve through it. §3 |
| The showcase in release builds | a cargo feature `showcase` on `warlock-core`, **on by default** and forwarded by `warlock` and `warlock-cli`; `build.rs` embeds `assets/campaigns/showcase/` only with it, so a release build holds not one byte of the test world. The release workflow and `cargo xtask web` build with `--no-default-features`; every dev command, test, golden, and benchmark is unchanged. A CI job checks the release configuration. §4 |
| The overworld | **120×60** (the Wildlands is 100×50): one third mountains and sea, ~5 000 walkable cells, 40 locations at least 6 cells apart claim under a third of them; three to four days across by road, a day and a half to the farthest of the first three dungeons. The pass is on the west edge, the town five road cells east of it. The volcano stands in the eastern range, the big river runs from it to the sea through the marsh, a second river from the western wall joins it; two deep forests. §5 |
| The first town | **Bramblecross**, a hand-made outdoor level (about 90×32) at the crossroads below the pass: four shops of the existing kinds, a temple whose nave is a sanctuary, an inn with an innkeeper who trades rumours, and three quest-givers (the reeve, the temple's sister, the mine-factor), each with a short chain on the existing kill/bring/reach/deliver quests. §6 |
| The dungeons | three generated branches at rising difficulty, each with an overworld entrance: the **Cairn Barrows** (6 levels, danger 1–6, visible from the start, most of a day's walk from town), the **Drowned Cellars** (8 levels, danger 4–11, hidden, revealed by the sister's second quest), the **Ember Mines** (12 levels, danger 8–19, hidden, revealed by the mine-factor's second quest). §7 |
| Hidden locations | an overworld legend entry may hide a location: the cell is its plain terrain in ground truth until revealed, so no view, memory, look, or dump can leak it (the secret-door model, `docs/dev/searching.md`); a quest's `reveals = { location, when }` makes the reveal at a stage (offered, accepted, done); the reveal sets the terrain, adds the stairs link, marks the cell on the player's map (an open question), and says so. Saved as `LevelState::hidden` and `WorldState::revealed`. §8 |
| Danger levels | one per area, always computable, never stored: a branch floor's is **`base + (depth − 1) × per_depth`**, a hand-made level's its `danger` (default 0: nothing spawns); an area whose data gives a `player` table scales with the player's level (ADOM's puppy cave), by default nothing does. Hidden in the game, exact in the wiki. §9.1 |
| Initial density | high: a generated floor starts with **35 creatures per 1 000 walkable cells** (25 to 80: about 40 on an 80×21 rooms floor, 70 on a 100×40 cave, 80 on a 120×60 one), drawn by the danger from the branch's `population` through the stream `population/<level id>` so the map stays the same; three full clears take a baseline character to level 14. §9.3–9.4 |
| Spawning | over game time while the player is on the level: every `every` (15 min of game time) a `chance` (60 %) of one creature from the population whose level is at most the danger (5 % out of depth by up to 3), weighted toward the danger, placed out of the player's view by the perception rules and at least 8 cells away, while the level holds fewer hostiles than its population (so a cleared floor refills, about 2.4 an hour). Its own stream per level, `spawn/<level id>`, in the level's state. Floors the player is not on **stay frozen** (proposed; the catch-up alternative is flagged in §14). §9.5 |
| Talents | the system `danger` with **Quiet delver** (creatures come half as often where you are) and **Sixth sense** (a note when something new stirs on your level; the danger in words on arrival); no class ability (`min_class = 0`). §9.7 |
| Creatures | 32 new creature types in three themed waves (barrow, marsh, mine) so each dungeon has three to five types per danger step; the six existing hostiles start the Barrows. §10 |
| Victory | not designed here: the campaign's end is story-driven (the owner's standing decision), and the three chains point at the volcano; §14 asks what the arc is. |

## 2. Research, tied to the decisions

| Finding | Source | Used in |
|---|---|---|
| ADOM: every dungeon level has a danger level (its branch's base plus its depth); monsters are generated over game time from those at or below it, with a small chance of one out of depth; monsters keep coming while you stay | ADOM Guidebook, "Danger levels", "Monster generation" | §9: the formula, the out-of-depth chance, the stream per level |
| ADOM: most dungeons have a fixed danger level, a few (the puppy cave) scale with the character's level | ADOM Guidebook | §9.1: a constant by default, a `player` table where an area's data names it |
| Angband: monsters have a native level and generate over time with a per-turn chance; "out of depth" monsters are the memorable ones; the level feeling tells the danger in words | Angband manual, "The dungeon", "Level feelings" | §9.2 weights, §9.7 Sixth sense's words |
| Brogue: new monsters arrive out of sight (some by the stairs), never in view | Brogue wiki, "Monster spawning" | §9.5: never in view, never within 8 cells |
| DCSS: spawning slows the longer you stay on a level (camping pays nothing) | crawl.develz.org manual, "Monster generation" | not adopted: the owner's "no anti-farming limits" rule (time is the penalty) |
| ADOM: the Drakalor Chain is entered through a pass in the mountains and the first village is a few steps away; wilderness steps take hours by terrain | ADOM Guidebook, "The wilderness" | §5: the pass on the edge, the town five road cells away, the travel budget |
| Diablo II, ToME 4: locations appear on the map as quests send you to them; the quest log names the region | the games | §8: a quest reveals a location at a stage; the objective text names the region |
| Elona, Dwarf Fortress adventure mode: a town of a few named services (smith, general, inn, temple) and three or four quest-givers is enough to feel inhabited | the games | §6: four shops, a temple, an inn, three givers |

## 3. Campaigns: the seam between worlds

### 3.1 What a campaign is

A campaign is a **world**: an overworld, the levels and dungeon branches that hang off it, the scenarios that start in it, its people (dialogue, quests), and nothing of the rules. Two campaigns never link (a stairs `to`, a scenario `start_level`, or a quest target naming another campaign's level is a content error). The engine is everything else, shared by all campaigns.

| Shared engine data (`assets/data/`, `assets/vaults/`; today's files, unmoved) | Campaign-owned (`assets/campaigns/<id>/`) |
|---|---|
| terrain, materials, objects, item classes, parts, gems, affixes, slots, stats, skills, weapon skills, stances, races, classes, talents, horoscope, names, calendar, sky, weather, hunger, status effects, diseases, damage types, resources, traps, room effects, search, steal, actions, experience, alignment, vision, lights, effects, legend, sizes, factions | `campaign.toml`; `levels/` (the overworld and every hand-made level); `scenarios/`; `data/dungeons/` (the branches); `data/quests/`; `data/dialogue/`; `vaults/` (optional: pieces only this world stamps) |
| **creatures** (types; `data/creatures/`): shared, because the generators and the local-area kit need them (`mapgen/local/kit.rs` asks for `deer`), ids are indices and feed the content fingerprint, and a type unused by a campaign costs nothing | people as **unique characters**: an `npc = "..."` on a legend entry of the campaign's level (`docs/dev/npcs.md`), their dialogue in the campaign's `data/dialogue/` |
| **shops** (`data/shops.toml`: the kinds, the keeper template, the names) | which level has which shop (`[[shop]]` in its level file), as today |
| **dialogue by creature type** (the shopkeeper's lines): `data/dialogue.toml` stays, holding the shared entries | `data/dialogue/*.toml`: the campaign's npcs and its own types' lines |

The showcase's quests (`data/quests.toml`) and the hermit's, gardener's, and bard's dialogue move to `campaigns/showcase/data/{quests,dialogue}/`; the creature types `hermit`, `gardener`, `bard` stay in `data/creatures/` (placed only by showcase levels; harmless elsewhere; moving them would renumber creature ids for nothing).

### 3.2 `campaign.toml`

```toml
id = "embermarch"            # must equal the directory name (a content error otherwise)
name = "The Embermarch"
description = "Over the Needle's Pass into the Embermarch, where the ash falls."
start = "embermarch"         # the scenario New run (and a release) starts
# kind = "test"              # the showcase: its scenarios are listed on New run in dev builds
                             # only, the wiki leaves it out, and release builds do not hold it
```

A campaign has at most one level of kind `overworld` (today's "at most one" check, `content/mod.rs::check`, becomes one per campaign). A campaign without one is allowed (the showcase would work without the Wildlands; a future dungeon-only campaign too).

### 3.3 How the content loads: the smallest change

`build.rs` embeds `assets/{data,levels,scenarios,vaults}` recursively, and `ContentFiles::from_dir` reads the same; a table loader reads `ContentFiles::table(name)` = `data/<name>.toml` then `data/<name>/**/*.toml` in path order (`docs/dev/content.md`). The change:

| Today | Proposed |
|---|---|
| `build.rs` collects four directories | and `campaigns/` (recursively; the `showcase` directory only with the feature, §4) |
| `ContentFiles::table(name)` reads `data/<name>.toml` and `data/<name>/` | and `campaigns/*/data/<name>/` (campaigns in path order after the shared files, so shared ids come first) |
| `in_dir("levels")`, `in_dir("scenarios")`, `in_dir("vaults")` read the top-level directories | and `campaigns/*/levels/` etc.; each entry remembers its campaign (`LevelTemplate::campaign`, `ScenarioDef::campaign`, `DungeonDef::campaign`, `QuestDef::campaign`) |
| `data/quests.toml`, `data/dialogue.toml`, `data/dungeons.toml` are single files read with `files.get` | they become **tables** (`quests`, `dialogue`, `dungeons`): the single file still works, directories add to it. None of the three has an order that means anything (dungeons are a `BTreeMap`; quests validate `requires` by id; dialogue is looked up by type or npc), so the "never split" rule of `docs/dev/content.md` does not apply to them |
| ids: level ids are file stems, unique by the file system | still file stems; **unique across campaigns**, else a `ContentError` naming both files (`content::duplicate_id`) |
| `Content::overworld()` is the one overworld; `local:x,y` is parsed against it | `Content::overworld_of(campaign)`; `Content::local_area(campaign, id)`; the two callers (`content/overworld.rs`, `world/travel.rs`) pass `state.campaign` |
| `Content::fingerprint()` hashes every level and dungeon id | `Content::fingerprint_for(campaign)`: the shared groups plus that campaign's levels and dungeons, so a release save (no showcase) loads in a dev build of the same content, and a bug report's save can be opened |

Levels, scenarios, dungeons, quests, and dialogue keep their ids as they are: no replay, golden manifest entry, test, or doc changes its id. The showcase scenarios keep `scenario = "showcase"` and friends.

### 3.4 A run knows its campaign

`WorldState::campaign: String`, set from the scenario at `World::new` (`ScenarioDef::campaign`), saved and hashed (a save is rejected if its campaign is not loaded, like any missing content), and `#[serde(default)]` to the scenario's campaign for the one format bump. Replays need nothing: they record the scenario (`docs/dev/saves.md`, `replay.rs`), and the campaign follows from it. The save summary (`saves/<id>.toml`) adds `campaign` (its name): the Characters screen shows it only when more than one campaign is loaded (dev builds).

### 3.5 New run

| Today | Proposed |
|---|---|
| New run lists every scenario (`ui/new_run_screen.rs`, from `Content::scenarios()`), the showcase first | lists the **start scenario of every campaign of kind `campaign`** first, then, in dev builds (debug or `--dev`) only, every scenario of every campaign (test ones included), grouped under their campaign's name; a release build has one row, so the step is **skipped** and New run goes straight to character creation (race first, `spec/06-input-and-ui.md`) |
| `--scenario NAME` starts any scenario | unchanged (an unknown one is refused as today; in a release build the showcase ones are unknown) |
| a scenario of a campaign that starts in its town or a dungeon (dev conveniences, like `depths` today) | `dev = true` in the scenario file: not listed by New run outside dev builds; still in the binary (a scenario is twenty lines; the content it starts in is the campaign's own) |

The main menu, Continue, and Characters are unchanged.

## 4. Keeping the test world out of release builds

### 4.1 The options

| Option | In the release binary? | Tests, goldens, benchmarks | The web build | Verdict |
|---|---|---|---|---|
| A. runtime hide: New run hides test scenarios and `--scenario` refuses them unless `--dev` (`cli.rs`, `input.rs::dev` reads `cfg!(debug_assertions) \|\| args.dev` today) | **yes**: the content is embedded whatever the flags; the wasm carries it too | unchanged | carries it | rejected: the owner asked that it not be in release builds, and a binary that holds a flag-locked world is one `--dev` away from showing it |
| B. `#[cfg(debug_assertions)]` around the embed | no in `--release` | **breaks `cargo xtask bench-turns`** (it re-runs itself in release and walks the hub, the vast level, the courtyard, the arena; `xtask/src/main.rs::bench_turns`) and `bench-frames` (release, the vast level), and every `cargo run --release -- --scenario ...` dev check | ok (the web profile inherits release) | rejected |
| **C. a cargo feature `showcase`, on by default** (`warlock-core`: `[features] default = ["showcase"]`; `warlock` and `warlock-cli`: `default = ["showcase"]`, `showcase = ["warlock-core/showcase"]`, with `warlock-core = { ..., default-features = false }` in the workspace table); `build.rs` reads `CARGO_FEATURE_SHOWCASE` and skips `assets/campaigns/showcase/` without it | **no** when built `--no-default-features`: `Content::builtin()` has no showcase files at all | unchanged: every documented command runs with default features | `cargo xtask web` adds `--no-default-features` (it is a release artifact: GitHub Pages and the release zip) | **proposed** |
| C′. the same feature, **off** by default | no in a plain `cargo build --release` | every test, golden, benchmark, capture, dump, and replay command gains `--features showcase` (about fifteen documented commands in CLAUDE.md, every `cargo test` invocation, xtask's own cargo calls) | ok | the honest alternative if the owner wants "a plain release build is a release"; open question 2 |

### 4.2 Every consumer of the test world, with C

| Consumer | Effect |
|---|---|
| `Content::builtin()` and the ~200 tests that start `"showcase"` (`World::new(content, "showcase", None)` across `warlock-core`, `warlock`, and the CLI) | unchanged: `cargo test --workspace` builds with default features |
| `cargo xtask smoke`, `golden`, `bench-frames`, `bench-turns`, `web-smoke` | unchanged, except `web-smoke`, which drives New run (§3.5: in the release web build the scenario step is skipped) and then checks the save and the resume at turn 5 as today; its walk changes from the hub to the campaign's start |
| `cargo xtask wiki` (CI, release workflow; default features, so the showcase is loaded) | `wiki/scenarios.rs` (the "Scenarios" table) lists only campaigns of kind `campaign` and their listed scenarios: the player wiki never mentions the test world; the dungeons table (§11.4) is by campaign likewise |
| `--content-dir assets` | reads everything on disk (`from_dir`), the showcase included: a dev tool |
| replays and the golden manifest (`scenario = "showcase"` and friends) | verified in dev builds, as today; a release binary refuses them as unknown scenarios |
| the release workflow (`cargo build --release --locked --package warlock --package warlock-cli`) | `--no-default-features`; so the web job (`cargo xtask web`, which passes it itself) |
| CI | a new job: `cargo build --no-default-features -p warlock -p warlock-cli`, `cargo check --no-default-features --target wasm32-unknown-unknown`, and `cargo test -p warlock-core --no-default-features release_content` (a test compiled only without the feature: `Content::builtin()` loads, has exactly the campaigns of kind `campaign`, no scenario of the showcase, and New run's list is one row) |
| `warlock-cli` | unchanged; `--scenario showcase` in a `--no-default-features` build says "unknown scenario" |
| saves | the fingerprint per campaign (§3.3) keeps a release save loadable in a dev build |

## 5. The overworld

### 5.1 Entry and the first hours

The player enters the map through **the Needle's Pass**, a cut in the western range (the Greywall), on the map's west edge: the run starts on the pass cell, a location of its own (terrain `mountain_pass`, `travel_minutes = 30`, no stairs yet: the way back over the mountains is closed, which is the frame of the campaign and an open question for the ending). The road runs east, and **Bramblecross**, the market town, is five road cells away: an hour and forty minutes at normal speed, within sight by day (sight 9). Nothing on the road hurts a level-1 character: the overworld spawns nothing (§9.1), and the town is safe.

### 5.2 Size

| | The Wildlands (`levels/overworld.toml`) | Proposed |
|---|---|---|
| Size | 100×50 (5 000 cells) | **120×60** (7 200 cells) |
| Impassable (mountains, deep water) | about 35 % | about 32 %: ~4 900 walkable cells |
| Locations | 3 | 40: the pass, the town, 3 dungeons, 35 reserved sites (§5.5) |
| Walkable cells per location | 1 080 | ~120, at least 6 cells between any two: 40 patches of 6×6 claim 29 % of the walkable cells |
| Across by road, 12 h of daylight a day (`travel_minutes`: road 20, plains 45, grassland 50, forest 90, hills 120, swamp and dense forest 150, `data/terrain.toml`) | 2¾ days | **3⅓ days** (a day's march: 36 road cells, 16 plains, 14 grassland, 8 forest, 6 hills, 5 swamp) |
| Town to the farthest of the first three dungeons | — | a day and a half (§5.6) |

Why not 100×50: with 40 locations it is 81 walkable cells each, and the 6-cell spacing claims 43 % of the land: every third patch is a site, which reads as crowded, and the mountains and the sea leave the coast nowhere to breathe. Why not larger: 150×75 puts the far dungeon two and a half days out and the ends of the map a week apart; the owner asked for "not too vast", and a 120×60 map is already a place of journeys (a trip to the Cellars and back is three days with a night in the marsh). The travel minutes are shared engine data; they stay. The local areas (80×40 per cell) are unchanged.

### 5.3 The sketch (1:2; each character is 2×2 cells of the 120×60 map)

```
MMMMMMMMMMMMMMM.............................MMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMM.......f...................MMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMfffffffffffffff...........MMMMMMMMMMMMMMMMMMM
MMMMMMMMM....fffffFFFFFFFFFfffff........MMMMMMMMMMMMMMMMMMMM
MMMMMMMMM..fffffFFFFoFFFFFFFFfffff.....MMMMMMMMMMMMMMMMMMMMM
MMMMMMMMM.fffffFFFFFFFFFFFFFFFfffof.....MMMMMMMMMM^MMMMMMMMM
MMMMM.....ffffFFFFFFFFFFFFFFFFFffff......MMMMMMMMMMMMMMMMMMM
MMMMM....ffffffFFFFFFFFFFFFFFFffffff......MMMMMMMMMMMMMMMMMM
MMMMM.....ff=offFFFFFFFFFFFFFffffff.....o...hMMMMMMMMM......
MMMMM.....==ffffffFFFFFFFFFfffoffff...hhhhhhhhhhh~h.........
MMMMM....=.ffffffhffffffffffffffff".hhhhhhhhhh3h~hhhh.......
MMMMM..""="""hhhhhhhhhffffffffff"""hhhhhhhhhhhh~hhhhhh......
MMMMM"o""=""hhhhhhhhhhhfffffof""""""hhhhhhhhhh~hhhhhh"""....
MMMMM""""="hhhhhh1hhhhhh""""""""""""""hhhhhhh~hhhhh"o""""...
MMMMP====T===============================b====o"""""""""""..
MMMMM""""="""hhhhhhhhh""""""""""""""""""..~"""""""""""""""".
MMMMM"""h="""""""h"""."""o"""""""""""""..~."""""""""""""""..
MMMMMhhhh=hhh"""""""".""""""""""""""o""..~.."""""""""""""...
MMMMMhhhh=hhhh""""""..."""""""""""""""..~...f""""""""wwwwwww
MMMMMhhhh=hhh"""""o....."""""""""""""...~fffffff.."..wwwwwww
MMMMM~~~~bffffff~~~~~~~~~~~~~~~~"""sssss~sssssfff....wwwwwww
MMMMMffff~~~~~~~ff............".~~~~~~~~~ssssssfof...wwwwwww
MMMMMfFFFF====================...ss2ssss~sssssssf.wwwwwwwwww
MMMMMFoFFFFFFFfffff.........".=osssssssss~sssssss.wwwwwwwwww
MMMMMFFFFFFFFFFfffff."o""""""""""sssssssss~sssss..wwwwwwwwww
MMMMMFFFFFFFFFfffff"""""""""""""""sssssssss~sss...wwwwwwwwww
MMMMMMMMMFFFFofffff"""""""""""""""osssosssss~~wwwwwwwwwwwwww
MMMMMMMMMFffffffff."""""o"""""""""""""..s....w~wwwwwwwwwwwww
MMMMMMMMMfffffff.....""""""""""""""".........wwwwwwwwwwwwwww
MMMMMMMMM.f................."................wwwwwwwwwwwwwww
```

`M` mountains, `^` the volcano, `h` hills, `f` forest, `F` deep forest, `"` grassland, `.` plains, `s` swamp, `~` rivers, `w` the sea, `=` roads, `b` bridges, `P` the pass (the start), `T` the town, `1` the Cairn Barrows (visible from the start), `2` the Drowned Cellars and `3` the Ember Mines (hidden until revealed; drawn here for the owner, as `truth` would draw them), `o` reserved sites (24 of the 35; the rest fit in the plains and along the coast). Rivers flow from the mountains to the sea: the Ash from the volcano's range south through the marsh, the Lind from the western wall east into the Ash. Roads: the pass to the town; the town east through the barrow hills to the ford over the Ash and a village beyond; south from the town over the Lind and along its south bank to the marsh village; north from the town to the wood's edge. No road leads to a hidden dungeon.

### 5.4 Regions (status bar names, `[[region]]` rectangles at full scale; every walkable cell in one, as the overworld test requires)

| Region | Area (x0, y0, x1, y1) | Land | What is there |
|---|---|---|---|
| The Greywall | 0, 0, 9, 59 and 0, 0, 29, 5 | mountains | the Needle's Pass (the start) |
| Bramblecross Vale | 10, 20, 50, 40 | grassland, plains, the two roads | Bramblecross; the bridge over the Lind; farms (reserved) |
| The Cairn Hills | 20, 18, 46, 30 | hills | the Cairn Barrows (dungeon 1); a shepherd's hut, a standing stone (reserved) |
| Hollin Wood | 18, 2, 72, 22 | forest around a deep heart (the Deep Hollin) | a hunters' camp, a hermitage, a ruined tower (reserved) |
| The Ash Fells | 70, 14, 108, 28 | hills, the Ash's upper course | the Ember Mines (dungeon 3, hidden); the ford village; a watchtower (reserved) |
| Ashenfell | 76, 0, 119, 16 | mountains and the volcano | the crater (a late location, reserved) |
| The Eastmarch | 94, 16, 119, 44 | grassland, plains | a trading post, a port on the Reach (reserved) |
| The Lind Vale | 10, 40, 64, 48 | the Lind, grassland, the river road | the marsh village; a mill, a ferry (reserved) |
| Lindwood | 10, 42, 40, 59 | forest, deep in the west (the Darkling) | a woodcutters' camp, a witch's hut (reserved) |
| The Drownmarsh | 60, 38, 96, 56 | swamp, the Ash's lower course | the Drowned Cellars (dungeon 2, hidden); a drowned chapel, a hag's island (reserved) |
| The Grey Reach | 90, 36, 119, 59 | the sea | the port's lighthouse on the shore (reserved) |

Names are proposals (open question 6).

### 5.5 Locations: 40 without crowding

| Built now | Reserved in the layout (named by later designs; not built) |
|---|---|
| the pass, the town, the three dungeons (5) | 2 villages, 4 farms and hamlets, a hunters' camp, a woodcutters' camp, a shepherd's hut, a mill, a ferry, a ford village (built as a cell of road: its level later), a hermitage, a monastery, a drowned chapel, a witch's hut, 2 watchtowers, a ruined tower, a fort, a trading post, a port and its lighthouse, 3 caves, 2 barrow fields, a standing stone, a hag's island, a giants' stair, a dragon's lair, the crater (35) |

A reserved site is a plain cell today (its local area generates as its terrain says); the map keeps its spacing so a later level drops in without moving anything.

### 5.6 The travel budget (at normal speed, by day; nights double it since the player rests)

| Journey | Cells | Time | Days (12 h of travel) |
|---|---|---|---|
| The pass → Bramblecross | 5 road | 1 h 40 | the first hour |
| Bramblecross → the Cairn Barrows | 14 road, 2 hills | 8 h 40 | within a day; back by dusk is not possible after a delve: camp, or sleep in the barrows' first level |
| Bramblecross → the marsh village → the Drowned Cellars | 48 road, 8 swamp | 16 h + 20 h | a day and a half; a night on the road and a night in the marsh (the marsh's disease checks, `docs/dev/status-effects.md`) |
| Bramblecross → the ford → the Ember Mines | 64 road, 10 hills | 21 h + 20 h | a day and three quarters |
| The town to the volcano's foot | 64 road, 24 hills | 21 h + 48 h | three days: the far end of the arc |

Food: a human burns a ration a day (`data/hunger.toml`); the trips are why the provisioner is in the town (§6).

### 5.7 New terrain

| Terrain | Glyph | Why |
|---|---|---|
| `mountain_pass` (location, `travel_minutes` 30, no stairs) | `)` in the mountains' colours | the start cell; walkable, in the mountain wall |
| `volcano` (impassable, glow, a particle plume) | `▲` red | the landmark; presentation only (its glow is the sky's at night) |
| `ford` (road over water, `travel_minutes` 30) | `≈` road colour | the Ash is forded, not bridged, east of the barrows (the Wildlands has `bridge`) |
| `town` (location, stairs down) | `#` warm | the town's cell; `hall` and `dungeon_entrance` exist (`data/terrain.toml` "Locations") |
| `barrow` (location, stairs down) | `∩` pale | the Barrows' entrance (a mound), distinct from a cave mouth |
| `mine` (location, stairs down) | `>` ochre | the Mines' entrance, once revealed |

## 6. The first town

**Bramblecross**: a market town at the crossroads below the pass, the trading post where the mountain road meets the river roads. A hand-made outdoor level (about 90×32; `kind = "outdoor"`: sky, sun, weather, the pond freezes; `item_level = 1`; `danger = 0`), its `<` on the square leading back to the town's overworld cell. Walled on the pass side, open to the river. Everyone in it is `peaceful` by type or by legend (`docs/dev/content.md`), so it is safe unless the player steals.

| Place | Reuses | Who | What for |
|---|---|---|---|
| The market square | the level, piles | townsfolk (a `villager` type: one-liners, mutters) | the start on arrival; the `<` |
| Harl's General Store | `general_store` (`data/shops.toml`) | a keeper (generated name) | a bit of everything; buys gems |
| The Forge (weaponsmith) and the Armourer's | `weaponsmith`, `armourer` | keepers | arms and armour on the floor at the shop's markup |
| The Provisioner's | `provisioner` | a keeper | rations, bread, dried meat for the road (§5.6) |
| The temple of the Hearth | a `sanctuary` special room (`data/room_effects.toml`: regen, hostiles shun it) | **Sister Ael** (npc; quest-giver 2) | rest and heal; her chain reveals the Drowned Cellars |
| The Crossed Keys (inn) | a room, a table of food | **Wenna the innkeeper** (npc; topics: the barrows, the marsh, the mines, the volcano, the pass) | rumours: the same facts the quest-givers tell, in other words; never a reveal (open question 9) |
| The reeve's house | — | **Reeve Cadoc** (npc; quest-giver 1) | the Barrows chain |
| The mine-factor's office | — | **Factor Ysolde** (npc; quest-giver 3) | the Mines chain |
| The well, the pond, the granary, a shrine stone, a gallows hill outside the wall | the courtyard's pieces | — | texture; the local areas around the town are its fields |

The healer: paying the temple to cure a disease or mend stat damage is a **service** no system has yet (shops sell objects). Angelica root (the remedy, `docs/dev/status-effects.md`) lies in Harl's store instead; a temple service is open question 8 and, if wanted, its own issue.

### 6.1 The quest chains

Every quest is one of today's four kinds (`data/quests.toml`: kill, bring, reach, deliver; `docs/dev/quests.md`); no new kind, no abandoning, no cap. Rewards on today's `Reward` (gold, items, XP); the XP figures follow `docs/design/progression.md` §3.4 (a quest is worth a few kills of its level).

| Giver | # | Kind | Target | Reveals | Teaches |
|---|---|---|---|---|---|
| Reeve Cadoc | C1 "Rats in the barrows" | kill 5 `rat` on `cairn` (any level) | — | the road east, the first delve, kill tallies |
| | C2 "The cairn's heart" (requires C1) | reach `cairn:4` | — | depth, the danger rising |
| | C3 "The wight of the barrow" (requires C2) | kill 1 `barrow_wight` on `cairn` | — | the first named foe (a level-6 creature at danger 6, §10) |
| Sister Ael | A1 "Grave goods" | bring 1 `silver_chalice` (a vault item in the Barrows: the barrow chamber vault, §7.2) | — | vaults, carrying things home |
| | A2 "The drowned abbey" (requires A1) | reach `cellars:3` | **the Drowned Cellars, on accept** | the river road, the marsh, a hidden place found |
| | A3 "What the water keeps" (requires A2) | bring 1 `abbey_reliquary` (a vault item deep in the Cellars) | — | the Cellars' bottom |
| Factor Ysolde | Y1 "The foreman's ledger" | deliver `ledger` to Wenna the innkeeper ("she sends it on with the next cart"): a delivery inside the town, since a recipient needs a hand-made level to stand on and the ford village has none yet (local areas place no npcs) | — | deliver quests |
| | Y2 "The lost shaft" (requires Y1, C2) | reach `mines:2` | **the Ember Mines, on accept** | the far road |
| | Y3 "Fire in the deep" (requires Y2) | kill 1 `fire_drake` on `mines` | — | the Mines' bottom and the arc's end for now |

The reveal of the Mines waits for C2 (a reach of `cairn:4`): the Mines start at danger 8, and a character who has not seen danger 4 is not sent there. Topics fit the dialogue screen's budget of nine (three quests × three topics each is the limit: a giver holds at most three quests at once, which is why each chain is three long).

## 7. The dungeons nearest the town

### 7.1 The three branches (`campaigns/embermarch/data/dungeons/*.toml`, one file per branch; the fields of `data/dungeons.toml` plus §9's)

| | The Cairn Barrows (`cairn`) | The Drowned Cellars (`cellars`) | The Ember Mines (`mines`) |
|---|---|---|---|
| Where | the Cairn Hills, 16 cells east of the town | the Drownmarsh, under a ruined abbey | the Ash Fells, below the volcano |
| Visible | **from the start** (`barrow` on the overworld) | hidden; revealed by A2 | hidden; revealed by Y2 |
| Depth | 6 | 8 | 12 |
| Generator | `rooms` 1–2 (the barrow's passages), `caves` 3–6 (the hill's hollows), 80×21 and 100×40 | `rooms` throughout (cellars and crypts), 80×21, every level with a flooded room's chance raised (`room_effects` per branch: open question 10) | `rooms` 1–4 (galleries), `caves` 5–12 (the deep workings), 120×60 at the bottom |
| Danger (§9.1; none scales with the player) | 1 to 6 (`danger = 1`) | 4 to 11 (`danger = 4`) | 8 to 19 (`danger = 8`) |
| Population at the start (§9.3) | 40 on the rooms floors, 70 in the caves | 40 a floor | 40 on the galleries, 80 in the deep workings |
| Population (§9.2) | rats, bats, beetles, kobolds, spiders, grave worms, skeletons, zombies, ghouls, goblins, the wight | leeches, frogs, drowned, eels, crocodiles, wisps, cultists, ghosts, marsh trolls, the hag, mud golems | ember beetles, duergar, salamanders, trolls, ogres, magma crawlers, basilisks, ash wraiths, cinder elementals, the fire drake |
| Traps | arrow, spear, pit, alarm | pit, frost, alarm, teleport | arrow, spear, fireball, lightning, alarm |
| Secret doors, `hidden_per_depth`, `locks_per_depth` | [1, 2], 1, 1 (as the Depths) | [1, 3], 1, 1 | [0, 2], 1, 2 |
| Loot (`loot`, item level = depth) | [2, 4] | [2, 4] | [3, 5] |
| Key | `rusty_key` | `iron_key` | `brass_key` |
| Temperature | 8 °C | 6 °C, water that freezes nowhere (the marsh is warm: `temperature = 6`) | 10 on top, 24 at the bottom (`[[dungeon.levels]]` overrides: the volcano's heat) |
| Shops | the Depths' one-in-six chance | none | one in four (duergar traders: a `general_store` with the duergar keeper's name list; open question 10) |

Each entrance is an overworld location whose `to` is `<branch>:1`; the first level's `<` leads back to the level it was first entered from (the overworld), as today (`spec/07c-generation.md`).

### 7.2 Vaults of their own (`campaigns/embermarch/vaults/`)

| Vault | Where it stamps | Holds |
|---|---|---|
| the barrow chamber | `cairn` rooms levels | the silver chalice (A1) on a plinth behind a locked door, its key on the chamber's guardian skeleton |
| the sunken crypt | `cellars` | the abbey reliquary (A3) at the far end of a flooded room (shallow water: slow, loud steps) |
| the foreman's cabin | `mines` 1–4 | a pick-axe, a lamp, the branch's key |
| the drake's hoard | `mines` 12 only (`[[dungeon.levels]]` may name vaults per depth: a small addition) | the fire drake (Y3) and rare loot (`loot = "rare"`) |

The shared vaults (the guard post, the shrine, the flooded cell, the ruins) stamp here too.

## 8. Hidden locations revealed by quests

The owner's addition: two of the three dungeons are not on the overworld until a quest (or one of its stages) reveals them; at least one is visible from the start, so a new character has somewhere to go (the Cairn Barrows).

### 8.1 The model: the secret-door precedent

A secret door is, in ground truth, the wall it looks like; `LevelState::secrets` holds what it becomes when found, and finding it sets the terrain (`docs/dev/searching.md`). Because the terrain grid holds the wall, the view, the memory, look mode, and every perception-built dump show a wall without any filter: nothing can leak but through the dev `truth` layer. Hidden locations work the same way.

| | Hidden location |
|---|---|
| In the overworld level file | a legend entry on the plain terrain the cell shows, with `hidden = { id, terrain, to, name }`: `"2" = { terrain = "swamp", hidden = { id = "drowned_cellars", terrain = "dungeon_entrance", to = "cellars:1", name = "the Drowned Cellars" } }`. The cell is a swamp cell in every way: terrain `swamp`, travel 150 min, a local area of marsh (generated as any swamp cell's if the player wanders there), no stairs link |
| In the template | `LevelTemplate::hidden: BTreeMap<Pos, HiddenLocation { id, terrain, to, name }>`; validated like a stairs link (`to` exists in the campaign, `terrain` has `stairs = "down"`, the id is unique across the campaign's overworld; a `hidden` on a cell that already has stairs, or on an impassable cell, is a content error) |
| In the state | `LevelState::hidden: BTreeMap<Pos, HiddenLocation>` (ground truth; saved; hashed; an entry leaves it when revealed) and `WorldState::revealed: BTreeSet<String>` (location ids revealed so far; saved; hashed; left out of the JSON while empty, so runs without a reveal hash as before) |
| The reveal, `World::reveal_location(id) -> bool` | finds the campaign's overworld: if it is instantiated, takes the entry out of `hidden`, `set_terrain(pos, entry.terrain)`, pushes `StairsLink { pos, dir: Down, to }` onto `stairs`, and (open question 7) writes `memory[pos] = Some(entry.terrain)`; if it is not yet instantiated (a scenario that starts in the town before the player has stepped outside), the id goes into `revealed` and `instantiate_level` applies every revealed entry when the overworld is made. Returns whether anything changed (a second reveal of the same id is nothing) |
| The hook in quests | `QuestDef::reveals: Option<Reveal { location: String, when: RevealWhen }>`, `when` one of `offered` (the giver tells you where it is as they ask: for a quest that is a tip), `accepted` (the default: "she marks the drowned abbey on your map"), `done`. `world/quest.rs` calls `reveal_location` in `ask_quest` at the matching transition (`Stage::Offered`, `Stage::Active`, `Stage::Done`, the three places the stage is set), before the quest's own line, so the message order is the giver's words, then "Sister Ael marks the drowned abbey on your map." (`Spent::NoTime`, as talk is) |
| Content checks | a quest's `reveals.location` names a hidden location of the quest's campaign's overworld; the same location may be revealed by several quests (the first to happen wins; the rest change nothing); a reach quest whose target lies behind a hidden location is fine (the player has to find the way) |

### 8.2 What the player sees and remembers

| Moment | The map (view, memory) | Look mode | The quest log | Messages |
|---|---|---|---|---|
| Before the reveal | the cell is swamp; walking onto it and pressing `>` explores the marsh's local area ("You explore the swamp.") | "a swamp" | A2 not yet offered: nothing | — |
| The reveal (accepting A2 in the temple) | the overworld is not in view; with the map marked, its memory now holds the entrance at the cell (drawn as remembered terrain when the player next looks at the overworld); without marking, nothing yet | — | "The drowned abbey (active, from Sister Ael): Reach the Drowned Cellars 3. The abbey lies in the Drownmarsh, east of the marsh village." (the objective adds the quest's `where` text: a `where` field on the quest, free prose, since the rules never tell the cell) | "Sister Ael marks the drowned abbey on your map." / "Sister Ael tells you where the drowned abbey lies: in the Drownmarsh, east of the marsh village." |
| Walking there after | the entrance in view when within sight; in memory as last seen | "the Drowned Cellars" (the location's `name`; `hidden.name` becomes the stairs' name the way `place_name` names levels) | as above, then "Done" | "You enter the Drowned Cellars." on `>` |
| Dev reveal (F2; `truth`) | hidden entrances drawn with their entrance glyph, as `S` marks secret doors; marked dev-only on screen, as F2 is | — | — | — |

A hidden location the player never hears of is never found: there is no search check on the overworld, and the cell is honestly a swamp. That is the owner's intent (the quest is the key); open question 9 asks whether rumours or exploration should be a second key.

### 8.3 The perception-snapshot rule

Nothing new is needed to keep the rule: `PerceptionView` reads `terrain` and `memory` (and `stairs` through them), which hold the plain terrain until the reveal. `LevelState::hidden` is read by exactly two readers: `reveal_location` and the dev `truth` dump (`dump.rs::truth_rows`, which draws `S` for secrets today and would draw the hidden entrance's glyph), both already on the dev-only side of the line (`Layer::is_dev_only`). A core test asserts that before the reveal the `view`, `memory`, and JSON dumps of a run standing beside the cell contain no entrance glyph, no `cellars` level id, and no location name, and that after `ask:N:accept-drowned_abbey` the memory dump shows the marked cell (if marked) and `>` on it enters `cellars:1`. The renderer draws from the snapshot only, as today.

### 8.4 Saves, replays, determinism

Saved: `hidden` (per level) and `revealed` (per run), both hashed; a save naming a location this content does not have is inconsistent (like a quest id). A replay's `ask:N:accept-drowned_abbey` step reveals deterministically: no stream is drawn, and the reveal is part of that step's state hash. The overworld's region test still covers hidden cells (they are plain cells). The content fingerprint is unchanged by hidden locations (they are not ids in a numbered table).

## 9. Danger levels and creatures that keep coming

The owner's decisions (2026-10-25), taken as given here: every area has **one danger level that can always be computed**: usually a constant (a branch's base plus the depth for a floor, a set value for a location), never rising with the player's level in general, but a **per-area rule may scale it** with the player where the area's data says so (ADOM's puppy cave); the game hides it and the wiki states it. Creatures **keep spawning on the level the player is on, over game time during play**, out of the player's sight by the perception rules, deterministically (a stream of their own, a pace by game time, a population cap). Generated floors **start well populated**, so a character can expect to level up in reasonable time or die trying.

### 9.1 Danger: a formula per area, a constant by default

`World::danger(level_id) -> u8` computes it when asked (an attempt, an arrival note, a dump); nothing stores it.

| Area | Data | Danger |
|---|---|---|
| a branch floor at depth *d* | `DungeonDef::danger = { base = 1, per_depth = 1 }` (`base` 0 to 99, `per_depth` 0 to 5, default 1); a `[[dungeon.levels]]` override may give a range its own `danger` table (a sudden deep floor) | `base + (d − 1) × per_depth` |
| a hand-made level | `danger = 3` in the level file (0 to 99; default 0) | the number; **0 means nothing spawns** (the town, every showcase level, the arena, the trap hall: unchanged) |
| the overworld and local areas | none | 0: no wandering encounters now (a line on #196) |
| **a player-scaled area** | either table with `player = { percent = 100, from = 1, max = 20 }`: `percent` 1 to 200, `from` the player level it starts counting at, `max` a ceiling on the sum | the constant above `+ max(0, player level − from) × percent / 100`, at most `max`; the player level is `World::player_level` (`docs/dev/experience.md`), read at the moment of the computation, so a scaled floor's draw follows the character as they grow, as the puppy cave's does |

The constant is the rule; a scaled area is the exception its data names (none of the three dungeons of §7 scales; the open question of §14 asks whether a training cave near the town should). The shorthand `danger = 4` in a branch file means `{ base = 4, per_depth = 1 }`.

### 9.2 Who may come

| Field (`DungeonDef`) | Meaning |
|---|---|
| `population = ["rat", "bat", ...]` (creature ids; default: the `creatures` list) | the types that live in the branch. Every type has a `level` (`data/creatures/README.toml`), which is its danger |
| `creatures` (today's field) | the patrols the generator places by itself, as today (the showcase's Depths keep theirs; a branch may give both) |

The weight of a type at danger *D* with level *L*, for every draw (the initial population and the spawns alike):

| | Weight |
|---|---|
| `L > D` | 0, except an **out-of-depth** draw: with `out_of_depth` % (default 5) the draw is made at `D + 1 … D + 3` instead (ADOM's few percent) |
| `L = D` or `L = D − 1` | 4 |
| `L = D − 2` or `L = D − 3` | 2 |
| `L ≤ D − 4` | 1, down to a floor: types below `D − 8` are not drawn (a danger-20 floor has no rats) |

Among the types of one weight the draw is uniform; the candidates are sorted by creature id before the draw, as the shop stock and loot are (`docs/dev/content.md`: nothing visible follows file order). A type with `unique = true` (the bog hag, the fire drake) is drawn at most once per run (`WorldState::progress` remembers the types spawned once; a small addition).

### 9.3 The initial population: high

A generated floor is laid down full. Its **population** is a count from its walkable cells (`mapgen` knows them after validation): `population = { per_1000 = 35, least = 25, most = 80 }` on the branch (defaults; a hand-made level with `danger > 0` gives `population = N` outright). The creatures are drawn by §9.2's weights from the stream `population/<level id>` after the loot (so a map, its traps, secrets, and loot are the same with any population), each placed on plain floor away from the arrival staircase (never within 8 cells of it: the player arrives there) and given a short patrol route where one fits (`Canvas::add_patrol`, validated as the generator's own patrols are), else a post. They carry their type's gear (`carry`, `wield`, `wear` on a creature type: a small addition, so a drawn kobold has its spear as a level-placed one does).

| Floor | Walkable cells (about) | Population | Of which on patrol |
|---|---|---|---|
| rooms, 80×21 (the Barrows 1–2, the Cellars, the Mines 1–4) | 1 000–1 200 | **35–40** | most |
| caves, 100×40 (the Barrows 3–6) | 1 800–2 000 | **65–70** | about half (caves give fewer loops) |
| caves, 120×60 (the Mines 5–12) | 3 000–3 600 | **80** (the ceiling) | about half |
| the showcase's Depths (80×21 and 120×60) | | unchanged: `population` not given, the two or three patrols of today |

The ceiling is the turn budget's: `cargo xtask bench-turns` gets a new entry (a Mines cave floor at full population, 16 ms a turn like the vast level) and the ceiling comes down if it is missed; creature sight is computed only for hostiles that could see the player (`docs/dev/creatures.md`), which is what keeps 80 affordable.

### 9.4 Expected XP per floor against the curve

The curve (`data/experience.toml`, `docs/dev/experience.md`; `docs/design/progression.md` §3.2): XP totals of 767 for level 2, 2 431 for 3, 5 084 for 4, 8 797 for 5, 13 628 for 6, 26 840 for 8, 45 064 for 10, 68 601 for 12, 114 464 for 15; a creature of level *L* is worth `5 × L + 10`. With the weights of §9.2 the average level on a floor of danger *D* is about `D − 1` (`D` on the first floor), so a floor's initial population is worth about `population × (5 × (D − 1) + 10)` to a character it does not out-level, and the spawns of a day's delve (§9.5: about 2.4 an hour) add a quarter again.

| Dungeon | Floors × population | Initial XP | With a day's spawns | Running total | Level at the bottom |
|---|---|---|---|---|---|
| The Cairn Barrows (danger 1–6; 40, 40, 70, 70, 70, 70) | 600 + 700 + 1 470 + 1 750 + 2 100 + 2 450 | 9 100 | 11 400 | 11 400 | **5**, nearly 6 |
| The Drowned Cellars (danger 4–11; 40 × 8) | 1 000 … 2 400 | 13 600 | 17 000 | 28 400 | **8**, going on 9 |
| The Ember Mines (danger 8–19; 40 × 4, 80 × 8) | 1 800 … 2 400, 5 200 … 8 000 | 55 200 | 69 000 | 97 400 | **14**, going on 15 |

Three full clears bring a baseline character to level 14 of 60, with the race and class factors (72 % to 175 %) moving it a level either way, the above-level bonus (5 % a level, up to 4) helping the brave, and grey kills (the zero-difference bands) taking from those who linger in the Barrows at level 10. In kills: the Barrows are about 430 kills (the curve asks 38 for level 2, 106 for level 5); a careless character dies to them long before (the owner's "or die trying").

### 9.5 Spawning over game time, on the player's level

| Field (`DungeonDef`, or a hand-made level's `[spawn]` table) | Default | Meaning |
|---|---|---|
| `every` | `15m` | game time between attempts on the level (150 normal turns): `LevelState::spawn_at: GameTime`, the next attempt's time (saved) |
| `chance` | 60 | percent that an attempt spawns something |
| `cap` | the floor's population count (§9.3) | an attempt is skipped while the level holds that many creatures hostile to the player (by `World::stance`; peaceful folk, wildlife, and shopkeepers do not count): a floor **refills toward its population** as it is cleared, about 2.4 an hour, 24 over a night's rest |

The hook: at the end of `World::run_turn` (`world/schedule.rs`), after the clock advanced, `spawn::tick(self)` makes every attempt whose time has come on the **current level** (several, if a rest passed hours: a rest to dawn lets the dungeon refill, which is ADOM's feel and the cost of resting below; `rest` stops when a creature comes into view, as today). **Where**: a cell drawn from the stream, walkable, unoccupied, not stairs, not a door, not a shop's floor, not in a sanctuary (`room_effect_at`), **not in the player's view by the perception rules** (the cells `perceive.rs` would mark visible now: line of sight and light, computed by the same functions, so a lit far cell counts as seen and a dark near one does not), and **at least 8 cells away** (Chebyshev). Up to 32 draws; none free, the attempt is skipped. The creature gets a short route where one fits, else a post (it waits until it sees or hears something, `world/ai.rs`), its type's gear, and is an ordinary actor from then on: `next_actor_id`, saved, hashed.

**Floors the player is not on stay frozen** (the proposal; flagged in §14): no clock runs for them, as no creature there gains energy today (`docs/dev/time.md`), and `spawn_at` is moved to the arrival time when the player returns, so nothing happened while they were away. The alternative is a catch-up on re-entry (the attempts the elapsed time allows, at most `catch_up`, placed out of sight of the arrival cell, from the level's own stream, so it is as deterministic); it is a day's work on top, and it makes a cleared floor refill behind the player's back, which cuts the other way: a cleared floor as a safe camp is a thing players value, and the current floor keeps spawning around the camp anyway. Frozen is proposed; `catch_up` can be added as a field later without touching the rest.

### 9.6 Determinism and streams

| Stream | Label | Kept | Draws |
|---|---|---|---|
| the level's spawns | `spawn/<level id>` | `LevelState::spawn_rng: Rng` (saved; made with the level; left out of the JSON with `spawn_at` while the level's danger is 0, so every existing level hashes as before) | the chance roll, the type, the cell, the route |
| the initial population | `population/<level id>` | derived at generation, never kept | the initial creatures, their cells and routes |

Neither touches `gameplay`, `map/…`, `traps/…`, `secrets/…`, or `loot/…`: a level generates the same map with any population, a fight rolls the same with or without a spawn between two blows, and a replay reproduces every spawn because attempt times follow the clock and the draws the level's stream. No `HashMap`; candidates sorted by id; cells scanned in row order. A property test (`spawn_tests.rs`): over many seeds, nothing spawns in view or within 8 cells, nothing above the danger but out-of-depth draws within 3, the cap holds, a unique type comes once, the same actions give the same spawns, and the population of a generated floor lands in the table of §9.3.

### 9.7 Talents (the rule: every new system ships with talents)

The system tag `danger` ("Danger levels and creatures that come") in `data/talents.toml` beside `awareness`, with two talents (`min_talents = 2`) and no class ability (`min_class = 0`):

| Talent | Tree | Ranks | Prerequisite | Effect (one hook each) |
|---|---|---|---|---|
| **Quiet delver** | Delver | 1 | none | while you are on a level, its attempts come half as often (`every` × 2 in `spawn::tick`) |
| **Sixth sense** | Delver | 2 | Alert | rank 1: on arrival, the level's danger against your level in words ("This place feels quiet / uneasy / dangerous / deadly" at danger ≤ level − 4 / ≤ level / ≤ level + 3 / beyond; a note, like a special room's line); rank 2: "Something stirs in the dark." when an attempt spawns on your level (a note: never stops a rest) |

Two more fit the Wayfarer and Shadow trees later (a lure for hunters, a ward that keeps spawns farther off); the ranger's Favoured prey already reads the kill tally that spawns feed.

### 9.8 Hidden in the game, exact in the wiki

The danger number is never shown: no screen, no look mode, no message (Sixth sense says words, relative to the player). The dev `actors` and `truth` dump headers gain `danger N`, `population N/M`, and `spawn next <time>`. The wiki's dungeons page (§11.4) states every floor's danger (and a scaled area's formula), the population with levels and weights, the population count by floor size, `every`, `chance`, the cap rule, and the out-of-depth chance, generated from the data.

## 10. The creatures the first dungeons need

Today's bestiary: nine types besides the player, six of them hostile (rat 1, bat 1, beetle 2, kobold 2, kobold archer 3, kobold shaman 4; `assets/data/creatures/`). Three to five types per danger step keeps a floor varied without a cast of a hundred; the three branches span danger 1 to 19 with overlaps, so **32 new types** in three waves, one file each on the creature XP guideline (`5 × level + 10`, `data/experience.toml`), all on existing fields (natural attacks with `on_hit` statuses and diseases, `natural` armour, `resist`, `combat`, `vision`, `speed`, `corpse`/`on_eat`, `loot`). No new combat mechanic is needed for any of them; the casters announce until the spell system lands, as the kobold shaman does.

| Wave | Type | Level | Kind | Notes |
|---|---|---|---|---|
| Barrows | grave worm | 1 | melee | slow, bites, poison 1 |
| | barrow spider | 2 | melee | quick, poison 2, darkvision |
| | goblin | 2 | melee | wields a club, opens doors, loud |
| | goblin archer | 3 | ranged | a short bow; drops arrows |
| | skeleton | 3 | melee | no corpse, resists piercing 50, vulnerable to bludgeoning |
| | zombie | 3 | melee | slow, lots of hit points, grave rot on hit |
| | giant centipede | 3 | melee | poison 2, darkvision |
| | barrow ghoul | 4 | melee | bleeding on a critical; quick |
| | goblin chief | 5 | melee | wears a jerkin, carries coins, loot 50 |
| | barrow wight | 6 | caster (announced) | C3's target; cold touch (cold damage), resists cold, darkvision, no corpse |
| Cellars | giant leech | 4 | melee | bleeding on hit |
| | giant frog | 4 | melee | quick, swims (shallow water at no cost: a `wades` flag is a small addition, else none) |
| | drowned | 5 | melee | undead, slow, filth fever on hit |
| | marsh eel | 5 | melee | only in water cells (placed there by the spawner when the type says `habitat = "water"`: a small addition, or dropped from the list) |
| | crocodile | 6 | melee | strong bite, natural armour 2/3 |
| | cultist | 6 | melee | wields a dagger, carries a scroll now and then |
| | will-o'-wisp | 7 | caster | light `glow`, flies, evasive (Defence high) |
| | ghost | 8 | melee | resists physical 50, no corpse, darkvision |
| | mud golem | 8 | melee | slow, protection 5, no corpse |
| | cult priest | 9 | caster | announces curses; loot 50 |
| | marsh troll | 9 | melee | regenerates quickly, big hit points |
| | the bog hag | 11 | caster | one per run (the spawner never draws a type with `unique = true` while one lives or has died: a small addition; else a normal type) |
| Mines | ember beetle | 8 | melee | fire bite, resists fire 50, glow |
| | duergar miner | 9 | melee | wields a pick-axe (as a weapon; creatures do not dig), darkvision |
| | duergar warrior | 11 | melee | mail, an axe, a shield |
| | ogre | 12 | melee | huge hits, slow |
| | fire salamander | 12 | melee | fire, resists fire 100, lights the way |
| | troll | 13 | melee | regenerates; the marsh troll's big brother |
| | duergar smith | 13 | melee | a war hammer; loot 75 |
| | magma crawler | 14 | melee | fire, vulnerable to cold, burrows (flavour) |
| | ash wraith | 15 | caster | announced; cold, no corpse |
| | basilisk | 16 | melee | its bite: a slowing status (Chilled exists; else poison 4) |
| | cinder elemental | 17 | melee | fire, immune to fire, no corpse, bright |
| | fire drake | 19 | caster/ranged (its breath: the `fire_breath` effect exists) | Y3's target; the drake's hoard vault; one per run |

Factions: `undead` (hostile to the player and to `kobolds`, `goblins`), `goblins` (hostile to the player, `vermin`, `kobolds`), `marsh` (the Cellars' living things), `cult` (the cultists; hostile to the player and `marsh`), `duergar` (hostile to the player; neutral to `fire` things), `fire` (the elementals, the salamanders, the drake). Wars inside a dungeon are the arena's lesson applied: a floor at danger 9 where cultists and marsh things fight gives a careful player kills to watch.

## 11. What the player sees

### 11.1 Messages

| Moment | Text |
|---|---|
| the run starts (the start scenario's `arrival` line) | "The pass opens on the Embermarch. Far to the east, Ashenfell smokes." |
| the town's arrival | "Market day in Bramblecross: the square is loud." |
| a reveal (marked) | "Sister Ael marks the drowned abbey on your map." |
| a reveal (told) | "Sister Ael tells you where the drowned abbey lies: in the Drownmarsh, east of the marsh village." |
| Sixth sense 1 | "This place feels uneasy." |
| Sixth sense 2 | "Something stirs in the dark." (a note) |
| a branch's first level, by `arrival` on the branch (a new optional field, said on `<branch>:1`) | "Cold air breathes up from the mound." / "The water is up to the third step." / "The galleries smell of sulphur." |

### 11.2 Screens

| Screen | Change |
|---|---|
| New run | §3.5: one row in a release (skipped), campaigns then scenarios in dev builds; a text snapshot case in `ui/screen_snapshots.rs` for each |
| the Characters screen | the campaign's name beside the place when more than one campaign is loaded; a snapshot case |
| the status bar | the region names of §5.4; nothing new |
| the quest log | the objective with the quest's `where` text (§8.2) |
| look mode | a revealed location by its name; a hidden one by its plain terrain |
| the death screen | "in the Cairn Barrows 3" as today's place names |

No golden screenshot: nothing here is something only the GPU produces (the overworld's sun, moon, and weather are covered by the Wildlands entries that exist).

### 11.3 The CLI and dumps

| Addition | Where |
|---|---|
| `--scenario embermarch` (the start), `bramblecross`, `cairn`, `cellars`, `mines` (`dev = true` scenarios starting in the town and on each branch's first level, the character chosen like `depths`) | `campaigns/embermarch/scenarios/` |
| `warlock-cli gen --level cairn:3 --seed warlock` | as today for any branch |
| the `locations` layer: the overworld locations the player knows, from the memory grid (perception): `(18, 28) Bramblecross`, `(34, 26) the Cairn Barrows`; `no known locations` | `dump.rs` |
| the `truth` and `actors` layers: hidden locations by their entrance glyph; their header `danger N population N/M spawn next 08:15:00` | dev-only |
| JSON: `campaign`, `known_locations: [{pos, name}]`, `due`… unchanged; `danger` only under `--state` | |
| a dev action `reveal:ID` (reveal a hidden location, no time, for captures and tests), listed with the dev actions | `world/dev.rs` |
| a dev action `spawn` (make one attempt on the current level now, no time) | `world/dev.rs` |

### 11.4 The wiki (the game is vague, the wiki is accurate)

| Page | Generated table |
|---|---|
| The world (new section: regions, the pass, the town; the overworld map as a code block at 1:1, generated from the level file with hidden locations drawn: the wiki states the truth, with a spoiler note, open question 12) | `overworld.md` |
| Dungeons (new page) | per branch: depth, generator, size, danger per floor, the population with each type's level and its weight at each floor, `every`, `chance`, `cap`, out-of-depth; "revealed by: The drowned abbey (Sister Ael)" |
| Creatures | the 32 new rows come by themselves |
| Quests (new page: name, giver, kind, target, reward, requires, reveals) | `quests.md` |
| Scenarios → Campaigns | lists campaigns of kind `campaign` and their listed scenarios; the showcase leaves the player wiki |
| Talents by system | the `danger` row by itself |

## 12. Data and code sketch

### 12.1 Files

| Path | Content |
|---|---|
| `assets/campaigns/showcase/campaign.toml` | `id = "showcase"`, `kind = "test"`, `start = "showcase"` |
| `assets/campaigns/showcase/{levels,scenarios}/` | today's `assets/levels/*.toml` and `assets/scenarios/*.toml`, moved (a `git mv`; the vast generator writes to the new path: `xtask/src/vast.rs`, `gen-vast --check`) |
| `assets/campaigns/showcase/data/{dungeons,quests,dialogue}/showcase.toml` | the Depths; the four quests; the hermit, gardener, and bard dialogues |
| `assets/data/dialogue.toml` | the shopkeeper's entry stays |
| `assets/campaigns/embermarch/campaign.toml`, `levels/{overworld,bramblecross}.toml`, `scenarios/{embermarch,bramblecross,cairn,cellars,mines}.toml`, `data/dungeons/{cairn,cellars,mines}.toml`, `data/quests/{cadoc,ael,ysolde}.toml`, `data/dialogue/{bramblecross}.toml`, `vaults/{barrow_chamber,sunken_crypt,foremans_cabin,drakes_hoard}.toml` | the campaign |
| `assets/data/creatures/<32 files>` | §10 |
| `assets/data/terrain.toml` | §5.7 (appended by group, under "Locations" and "The overworld") |
| `assets/data/factions.toml` | §10's six factions |
| `assets/data/talents/{quiet_delver,sixth_sense}.toml`, `talents.toml` `[[system]] id = "danger"` | §9.7 |
| `crates/warlock-core/build.rs`, `Cargo.toml` ×3, `.github/workflows/{ci,release}.yml`, `xtask/src/web.rs` | §4 |
| `content/campaign.rs` (new), `content/mod.rs` (tables, `in_dir` over campaigns, checks per campaign, `fingerprint_for`), `content/overworld.rs`, `content/dungeon.rs` (§9 fields, `population`), `content/quest.rs` (`reveals`, `where`), `content/level.rs` (`hidden`, `danger`, `population`, `[spawn]`) | §3, §8, §9 |
| `world/spawn.rs` (new: `tick`, `attempt`, the cell and route picks; `World::danger`), `world/schedule.rs` (one call), `world/travel.rs` (one line resetting `spawn_at` on arrival; `instantiate_level` applies `revealed`), `mapgen/population.rs` (new: the initial population), `world/reveal.rs` (new: `reveal_location`), `world/quest.rs` (three calls), `world/dev.rs` (`reveal`, `spawn`), `rng.rs` (`spawn`, `population`), `dump.rs` | |
| `crates/warlock/src/ui/new_run_screen.rs`, `save/characters.rs` (the summary's campaign), `cli.rs` (nothing: `--scenario` is any id) | §3.5 |
| `xtask/src/wiki/{scenarios,dungeons,quests,overworld}.rs`, `wiki/src/SUMMARY.md` | §11.4 |
| `docs/dev/campaign.md` (new topic: the seam, the gating, the checklist of §4.2), `docs/dev/overworld.md`, `mapgen.md`, `quests.md`, `creatures.md` (spawning), `content.md`, `saves.md`; `spec/03` (the showcase as the test campaign), a new `spec/03b-campaign.md`, `spec/07c`, `07f`, `08b`; CLAUDE.md's commands | the repo's rule: together with the code |

### 12.2 State

| Where | Field | Saved | Hashed |
|---|---|---|---|
| `WorldState` | `campaign: String` | yes (`default` for the bump) | yes |
| `WorldState` | `revealed: BTreeSet<String>` | yes (left out while empty) | yes |
| `LevelState` | `hidden: BTreeMap<Pos, HiddenLocation>` | yes (left out while empty) | yes |
| `LevelState` | `spawn_at: GameTime`, `spawn_rng: Rng` (left out while the level's danger is 0, so every existing level's JSON is unchanged; the danger itself is computed, never stored) | yes | yes |
| `WorldState::progress` | the unique types spawned once | yes | yes |
| the save envelope | the fingerprint of `fingerprint_for(campaign)` | yes | — |
| the save summary | `campaign` | yes | — |

The format bump is the integrator's, once (`docs/dev/merging.md`); every PR here says "invalidates saves" and leaves the number alone.

### 12.3 Hooks (one each)

| Rule | Hook |
|---|---|
| spawning on the current level | `run_turn` → `spawn::tick` |
| a frozen floor | `travel` moves `spawn_at` to the arrival time (one line) |
| the initial population | `mapgen::generate_with` after `place_loot`, from `population/<id>` (`mapgen/population.rs`) |
| the danger | `World::danger(level_id)`: the branch's or level's table, the depth, and `player_level` for a scaled area |
| the reveal | `quest.rs` at the three stage transitions → `reveal::reveal_location`; `instantiate_level` → `reveal::apply_revealed` |
| the campaign's overworld | `content.local_area(&state.campaign, id)` in `enter_local_area` and `parse_local_area` |
| Quiet delver | `spawn::tick` reads `talent_mods(content).spawn_every_percent` |
| Sixth sense | `travel` (the arrival note) and `spawn::tick` (the stir note), both through `World::note` |

### 12.4 Tests each PR carries

Unit tests in `warlock-core` for the rules; a content test that every campaign's links stay inside it and that ids are unique; `release_content` under `cfg(not(feature = "showcase"))`; the perception leak test of §8.3; the spawn property test of §9.6; insta snapshots of the three branches' first levels (`mapgen/snapshots/`), of the town's dumps, and of the New run and Characters screens (`ui/screen_snapshots.rs`); a replay per campaign scenario, created and verified; `cargo xtask wiki` in CI; no golden entry (the manifest's Wildlands entries keep their references, since the Wildlands only moves directory).

## 13. Implementation plan

Issue-ready items; each one issue, one branch, one PR, with the spec, `docs/dev/`, CLAUDE.md, and wiki in the same PR. Size S (a day), M (two to three). Items in a wave touch no common hot file; A owns `content/mod.rs`, C owns `world/schedule.rs` and `travel.rs`'s arrival, D owns `world/quest.rs` and `travel.rs`'s instantiation (one line each, by the playbook's rule). The critical path to a playable release is **A → B → E → F → K** (five PRs); the dungeons and the danger system widen it.

| # | Issue title | Implements | Depends on | Size | Parallel with |
|---|---|---|---|---|---|
| **A** | Campaigns: `assets/campaigns/<id>/`, `campaign.toml`, quests/dialogue/dungeons as tables, global ids, one overworld per campaign, `WorldState::campaign`, the fingerprint per campaign, New run's order, the wiki's scenarios by campaign; the showcase moved into `campaigns/showcase/` | §3 | — | M | C, D |
| **B** | The `showcase` feature: `build.rs` gating, the three `Cargo.toml`s, `--no-default-features` in the release workflow and `cargo xtask web`, the CI release-configuration job, `release_content`, `web-smoke` on the campaign start | §4 | A | S | E, G |
| **C** | Danger levels and spawning: `World::danger` with the `player` scaling table, the branch and level fields, `LevelState` spawn state, `spawn::tick`, the initial population from `population/<id>` with its counts by floor size and the `bench-turns` entry, unique types, the `spawn` dev action, dumps, the property test, the system tag with Quiet delver and Sixth sense, the wiki's dungeons table | §9, §11.4 | — | M–L (split the initial population and the bench entry into **C2** if it runs over) | A, D |
| **D** | Hidden locations and quest reveals: the legend's `hidden`, `LevelState::hidden`, `WorldState::revealed`, `reveal_location`, `QuestDef::reveals` and `where`, the messages, the `locations` layer, the `reveal` dev action, the leak test | §8 | — (uses the Wildlands and the hermit's quests for its tests) | S–M | A, C |
| **E** | The Embermarch overworld: `campaign.toml`, the 120×60 map with its regions, the new terrain, the pass, the start scenario, the Barrows' entrance (visible) and the two hidden entrances | §5 | A, D (for the hidden entries; else they land in H) | M | B, G |
| **F** | Bramblecross: the town level, its four shops, the temple's sanctuary, the inn, the npcs with one-liners and topics (no quests yet), the `bramblecross` dev scenario | §6 (without §6.1) | E | M | G, I, J |
| **G** | The Cairn Barrows: the branch, the barrow chamber vault, the barrow wave of creatures (10 types) and their factions, the `cairn` dev scenario | §7, §10 | C, E | M | B, F |
| **H** | The town's quests: the three chains, the two reveals, the silver chalice and the reliquary and the ledger objects, the branch `arrival` lines | §6.1 | D, F, G, I, J (the targets; a chain may land before its deep targets with the last quest held back) | S–M | K |
| **I** | The Drowned Cellars: the branch, the sunken crypt vault, the marsh wave (12 types; `unique`, `habitat` if kept), the `cellars` dev scenario | §7, §10 | C, E | M | F, J |
| **J** | The Ember Mines: the branch, the cabin and hoard vaults, per-depth vaults, the mine wave (12 types), the `mines` dev scenario | §7, §10 | C, E | M | F, I |
| **K** | Release start: New run skips the scenario step in a release, the Characters screen's campaign, `spec/03b-campaign.md`, the wiki's world page, CLAUDE.md's commands | §3.5, §11 | B, E, F | S | H |

**Waves**: **1** A, C, D · **2** B, E, G (G needs C and E: it starts once both land, else it joins wave 3) · **3** F, I, J · **4** H, K. Four waves of at most three, under the integrator's "about three PRs at once that add actions or save state" (A, C, D each add state: that is wave 1's three).

## 14. Open questions for the owner

1. **Campaign-owned vs shared**: creatures stay shared engine data (§3.1); people are unique characters on the campaign's levels. Agreed? The alternative (creatures per campaign) renumbers ids and makes the generators' kits campaign-aware.
2. **The feature's default**: `showcase` on by default, with the release workflow and `cargo xtask web` turning it off (§4.1 C), or off by default so a plain `cargo build --release` is a release and every dev command says `--features showcase` (C′)?
3. **Ids**: global and unique across campaigns (no namespacing; nothing existing is renamed), with a content error on a clash. Or namespaced (`showcase/hub`), which renames every replay, manifest entry, and doc?
4. **The overworld's size**: 120×60 as proposed (§5.2), or 100×60 (a day less across, 40 locations at 100 walkable cells each)?
5. **The pass**: a closed way back (the frame), a way out later (an ending), or an exit from the start (as ADOM's)?
6. **Names and tone**: the Embermarch, the Needle's Pass, Bramblecross, Ashenfell, the Ash and the Lind, Hollin Wood, the Drownmarsh, the Cairn Barrows, the Drowned Cellars, the Ember Mines; the reeve Cadoc, Sister Ael, Factor Ysolde, Wenna. The tone: a frontier march newly opened beyond the mountains, a trading town living off the pass, and a volcano the old things under the hills answer to. Keep, change, or hand me a different register?
7. **A reveal marks the map** (the memory grid shows the entrance at once; ADOM-like certainty) **or only tells** (the objective says the region; the player finds the cell by walking within sight of it)? The proposal is to mark; both are one flag on the quest (`marks = true`).
8. **The temple as a service** (pay to cure a disease or mend stat damage): a new mechanic, its own design line later, or not at all (angelica root at the general store does it)?
9. **A second key to a hidden dungeon**: rumours at the inn (a topic that reveals, at a price) or none (quests only, as proposed)?
10. **Per-branch room effects and shop kinds** (§7.1: more flooded rooms in the Cellars, duergar traders in the Mines): a small addition to `DungeonDef` (`room_effects`, `shops`) in I and J, or leave the defaults?
11. **Frozen or catch-up** for floors the player is not on (§9.5): frozen is proposed (nothing happens there while the player is away; `spawn_at` resets on arrival); the catch-up alternative is deterministic too and a day's work more, and it refills cleared floors behind the player's back.
12. **Hidden locations in the wiki**: the wiki states the truth (the map with every location, the dungeons page saying what reveals each) under a spoiler heading, or leaves the hidden two unnamed until the reader has the quest?
13. **A player-scaled area near the town**: none of the three dungeons scales (§9.1). Should one of the reserved caves of §5.5 be a training cave whose danger follows the character (`player = { percent = 100, from = 1, max = 12 }`), so there is always somewhere to fight at your own level, or is that against the grain of a world whose places have their own danger?
14. **The creature list** (§10): 32 types in three waves, each a small file; cut it to ~20 (two or three per danger step) for the first release and grow later, or take it whole?
15. **The campaign's arc and ending**: the three chains end at the fire drake; the story-driven victory is not designed here. Is the volcano the arc (a fourth dungeon, the crater, and an ending there), and should its design be the next document?
16. **Spawning on hand-made levels of the showcase** (the arena, the trap hall): none, as proposed (`danger = 0`), so every golden walk and replay there stays what it is.

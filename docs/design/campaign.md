# Warlock — Campaign: the game world, its overworld, the first town, the nearest dungeons, danger levels

Design document for the owner's review (2026-10-25). Written against `origin/main` at 953fd30b. Nothing here is implemented; §13 cuts it into issues. Revision 2 (2026-10-25) records the owner's answers to the first revision's sixteen open questions as decisions, each in the section it belongs to, marked **Decision (2026-10-25)**; §14 lists where each went. Nothing is open.

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
10. The creatures the first dungeons need (and the gear humanoids carry)
11. What the player sees (messages, the map, the quest log, dumps, the wiki)
12. Data and code sketch (files, state, hooks, determinism, tests)
13. Implementation plan
14. The owner's decisions (2026-10-25): where each answer went

## 1. Summary

**What the owner asked for**: start the game world. Everything built so far (the hub, the arena, the courtyard, the quarry, the trap hall, the Frost Grotto, the vast level, the Depths, the Wildlands overworld, and the nine scenarios; `spec/03-showcase-world.md`) stays, for tests: it is one test world with several start locations. The game gets a **second world with no connection to it**, and **test scenarios do not appear in release builds**, only in dev and CI builds. In the distant future there may be several full campaigns that share no world; this document designs the seam and builds no more of the machinery than the second world needs. To build now: an **overworld** entered ADOM-style through a **mountain pass**, with a **market town** the closest location to the entry, a **volcano, mountain ranges, swamps, rivers, and deep forests**, and room for **about 40 locations** without crowding or sprawl; the **first town**; the **dungeons closest to it**, procedurally generated, **two of them hidden until a quest reveals their location** (at least one visible from the start); and on every dungeon level a **hidden danger level** (ADOM's) from which **monsters of that level keep spawning over time**.

**The proposal in one table:**

| Piece | Decision |
|---|---|
| Campaigns | a directory per world under `assets/campaigns/<id>/` holding what is the world's own (its overworld and levels, scenarios, dungeon branches, quests, dialogue, its own **semi-unique creatures** (people, bosses), and any vaults of its own) with a `campaign.toml` (name, description, the scenario New run starts, `kind = "test"` for the showcase); everything else (`assets/data`, `assets/vaults`) stays shared engine data. **Ids are global string slugs, unique across campaigns** (a duplicate is a content error naming both files; a prefix in the slug is the only namespacing there will be). A run records its campaign (`WorldState::campaign`), local areas and the content fingerprint resolve through it. §3 |
| The showcase in release builds | a cargo feature `showcase` on `warlock-core`, **on by default** and forwarded by `warlock` and `warlock-cli`; `build.rs` embeds `assets/campaigns/showcase/` only with it, so a release build holds not one byte of the test world. The release workflow and `cargo xtask web` build with `--no-default-features`; every dev command, test, golden, and benchmark is unchanged. A CI job checks the release configuration, and a test that a release build's content holds no campaign of kind `test`. §4 |
| The overworld | **120×60** (the Wildlands is 100×50): one third mountains and sea, ~5 000 walkable cells, 40 locations at least 6 cells apart claim under a third of them; three to four days across by road, a day and a half to the farthest of the first three dungeons. The pass is on the west edge, **a closed way back** (the player came through it and cannot leave by it), the town five road cells east of it. The volcano stands in the eastern range, the big river runs from it to the sea through the marsh, a second river from the western wall joins it; two deep forests. §5 |
| Names | plain English words for what a place is, put into **Old English** ("market town" → *cēap* + *tūn* → **Cēaptūn**), in spellings the game's font draws; the campaign is **the Ascmearc** ("the ash march"), id and directory `ascmearc`. The table of every name, its source words, and its meaning is §5.8. |
| The first town | **Cēaptūn**, a hand-made outdoor level (about 90×32) at the crossroads below the pass: four shops of the existing kinds, a temple whose nave is a sanctuary and whose sister **sells services** (a cure of every disease, a mending of stat damage), an inn with an innkeeper who trades rumours, and three quest-givers (the reeve, the temple's sister, the mine-factor), each with a short chain on the existing kill/bring/reach/deliver quests. §6 |
| The dungeons | three generated branches at rising difficulty, each with an overworld entrance: **Cealdhlāw** (6 levels, danger 1–6, visible from the start, most of a day's walk from town), **Fennmynster** (8 levels, danger 4–11, hidden, revealed by the sister's second quest), **Fyrdelf** (12 levels, danger 8–19, hidden, revealed by the mine-factor's second quest). A branch may set its own special-room and shop odds (`room_effects`, `shops` on `DungeonDef`). §7 |
| Hidden locations | an overworld legend entry may hide a location: the cell is its plain terrain in ground truth until revealed, so no view, memory, look, or dump can leak it (the secret-door model, `docs/dev/searching.md`); a quest's `reveals = { location, when }` makes the reveal at a stage (offered, accepted, done); the reveal sets the terrain, adds the stairs link, **marks the cell on the player's map**, and says so. Quests are the only key (no rumour or exploration reveals one). Saved as `LevelState::hidden` and `WorldState::revealed`. §8 |
| Danger levels | one per area, always computable, never stored: a branch floor's is **`base + (depth − 1) × per_depth`**, a hand-made level's its `danger` (default 0: nothing spawns); an area whose data gives a `player` table scales with the player's level (ADOM's puppy cave); **nothing in the start area does** (the mechanism stays in the data model, unused). Hidden in the game, exact in the wiki. §9.1 |
| Initial density | high: a generated floor starts with **35 creatures per 1 000 walkable cells** (25 to 80: about 40 on an 80×21 rooms floor, 70 on a 100×40 cave, 80 on a 120×60 one), drawn by the danger from the branch's `population` through the stream `population/<level id>` so the map stays the same; three full clears take a baseline character to level 14. §9.3–9.4 |
| Spawning | over game time while the player is on the level: every `every` (15 min of game time) a `chance` (60 %) of one creature from the population whose level is at most the danger (5 % out of depth by up to 3), weighted toward the danger, placed out of the player's view by the perception rules and at least 8 cells away, while the level holds fewer hostiles than its population (so a cleared floor refills, about 2.4 an hour). Its own stream per level, `spawn/<level id>`, in the level's state. Floors the player is not on **stay frozen**; nothing spawns on the showcase's hand-made levels (`danger = 0`). §9.5 |
| Talents | the system `danger` with **Quiet delver** (creatures come half as often where you are) and **Sixth sense** (a note when something new stirs on your level; the danger in words on arrival); no class ability (`min_class = 0`). §9.7 |
| Creatures | 32 new creature types in three themed waves (barrow, fen, delf), **all scheduled**, so each dungeon has three to five types per danger step; the six existing hostiles start Cealdhlāw. The common types are shared engine data; the bosses and the townsfolk are the campaign's own files. §10 |
| Gear on humanoids | a humanoid type carries a **gear table**: each line a chance of an object it wields or wears (combat reads it as it reads the player's), with a **racial weapon** it has ranks in and a high chance to carry; drawn from the stream `gear/<actor id>` when the creature is made, some of it magic at the floor's item level through today's `loot::make`; it falls on the floor at death as everything carried does. §10.3 |
| Victory | deferred: no ending or arc is designed here (the campaign's end is story-driven, the owner's standing decision). |

## 2. Research, tied to the decisions

| Finding | Source | Used in |
|---|---|---|
| ADOM: every dungeon level has a danger level (its branch's base plus its depth); monsters are generated over game time from those at or below it, with a small chance of one out of depth; monsters keep coming while you stay | ADOM Guidebook, "Danger levels", "Monster generation" | §9: the formula, the out-of-depth chance, the stream per level |
| ADOM: most dungeons have a fixed danger level, a few (the puppy cave) scale with the character's level | ADOM Guidebook | §9.1: a constant by default, a `player` table where an area's data names it |
| Angband: monsters have a native level and generate over time with a per-turn chance; "out of depth" monsters are the memorable ones; the level feeling tells the danger in words | Angband manual, "The dungeon", "Level feelings" | §9.2 weights, §9.7 Sixth sense's words |
| Brogue: new monsters arrive out of sight (some by the stairs), never in view | Brogue wiki, "Monster spawning" | §9.5: never in view, never within 8 cells |
| DCSS: spawning slows the longer you stay on a level (camping pays nothing) | crawl.develz.org manual, "Monster generation" | not adopted: the owner's "no anti-farming limits" rule (time is the penalty) |
| ADOM: the player arrives in the Drakalor Chain through the pass at its edge and cannot leave by it; the first village is a few steps away; wilderness steps take hours by terrain | ADOM Guidebook, "The wilderness" | §5: the pass on the edge, a closed way back, the town five road cells away, the travel budget |
| Tolkien's Rohan: place names that are plain words for the place in Old English (Edoras "the courts", Meduseld "the mead hall") read as old and real without a made-up tongue | *The Lord of the Rings*, appendix F | §5.8: the naming register |
| ADOM, Angband, DCSS: humanoid monsters carry weapons and armour they use, and drop them; orcs favour orcish weapons, kobolds spears | the games' monster lists | §10.3: gear tables with a racial weapon |
| Diablo II, ToME 4: locations appear on the map as quests send you to them; the quest log names the region | the games | §8: a quest reveals a location at a stage; the objective text names the region |
| Elona, Dwarf Fortress adventure mode: a town of a few named services (smith, general, inn, temple) and three or four quest-givers is enough to feel inhabited | the games | §6: four shops, a temple, an inn, three givers |

## 3. Campaigns: the seam between worlds

### 3.1 What a campaign is

A campaign is a **world**: an overworld, the levels and dungeon branches that hang off it, the scenarios that start in it, its people (dialogue, quests), and nothing of the rules. Two campaigns never link (a stairs `to`, a scenario `start_level`, or a quest target naming another campaign's level is a content error). The engine is everything else, shared by all campaigns.

| Shared engine data (`assets/data/`, `assets/vaults/`; today's files, unmoved) | Campaign-owned (`assets/campaigns/<id>/`) |
|---|---|
| terrain, materials, objects, item classes, parts, gems, affixes, slots, stats, skills, weapon skills, stances, races, classes, talents, horoscope, names, calendar, sky, weather, hunger, status effects, diseases, damage types, resources, traps, room effects, search, steal, actions, experience, alignment, vision, lights, effects, legend, sizes, factions | `campaign.toml`; `levels/` (the overworld and every hand-made level); `scenarios/`; `data/dungeons/` (the branches); `data/quests/`; `data/dialogue/`; `data/creatures/` (its semi-unique types, §3.6); `vaults/` (optional: pieces only this world stamps) |
| **creatures** (the common types; `data/creatures/`): the generators' patrols and the local-area kit need them (`mapgen/local/kit.rs` asks for `deer`), and a type unused by a campaign costs nothing | the campaign's **own creature files** layered over the shared ones (§3.6): its people (a `villager`), its bosses (the barrow wight, the bog hag, the fire drake); and people as **unique characters**: an `npc = "..."` on a legend entry of the campaign's level (`docs/dev/npcs.md`), their dialogue in the campaign's `data/dialogue/` |
| **shops** (`data/shops.toml`: the kinds, the keeper template, the names) | which level has which shop (`[[shop]]` in its level file), as today; a branch's own shop odds and kinds (§7.3) |
| **dialogue by creature type** (the shopkeeper's lines): `data/dialogue.toml` stays, holding the shared entries | `data/dialogue/*.toml`: the campaign's npcs and its own types' lines |

**Decision (2026-10-25, question 1)**: creatures are shared engine data, and a campaign may add its own on top. The showcase's quests (`data/quests.toml`) and the hermit's, gardener's, and bard's dialogue move to `campaigns/showcase/data/{quests,dialogue}/`, and so do the creature types `hermit`, `gardener`, and `bard` (to `campaigns/showcase/data/creatures/`): they are the test world's people, and §3.6's numbering keeps the move from touching any other campaign's ids (it renumbers the shared block once, which the release bump covers, like the rest of A). `shopkeeper`, `deer`, `player`, and the six hostiles stay shared.

### 3.2 `campaign.toml`

```toml
id = "ascmearc"              # must equal the directory name (a content error otherwise)
name = "The Ascmearc"        # "the ash march" (§5.8)
description = "Through Beorggeat into the Ascmearc, where the ash falls and no road leads home."
start = "ascmearc"           # the scenario New run (and a release) starts
# kind = "test"              # the showcase: its scenarios are listed on New run in dev builds
                             # only, the wiki leaves it out, and release builds do not hold it
```

A campaign has at most one level of kind `overworld` (today's "at most one" check, `content/mod.rs::check`, becomes one per campaign). A campaign without one is allowed (the showcase would work without the Wildlands; a future dungeon-only campaign too).

### 3.3 How the content loads: the smallest change

`build.rs` embeds `assets/{data,levels,scenarios,vaults}` recursively, and `ContentFiles::from_dir` reads the same; a table loader reads `ContentFiles::table(name)` = `data/<name>.toml` then `data/<name>/**/*.toml` in path order (`docs/dev/content.md`). The change:

| Today | Proposed |
|---|---|
| `build.rs` collects four directories | and `campaigns/` (recursively; the `showcase` directory only with the feature, §4) |
| `ContentFiles::table(name)` reads `data/<name>.toml` and `data/<name>/` | and `campaigns/*/data/<name>/` (after the shared files, so shared ids come first: campaigns of kind `campaign` in path order, then the test ones, Â§3.6) |
| `in_dir("levels")`, `in_dir("scenarios")`, `in_dir("vaults")` read the top-level directories | and `campaigns/*/levels/` etc.; each entry remembers its campaign (`LevelTemplate::campaign`, `ScenarioDef::campaign`, `DungeonDef::campaign`, `QuestDef::campaign`) |
| `data/quests.toml`, `data/dialogue.toml`, `data/dungeons.toml` are single files read with `files.get` | they become **tables** (`quests`, `dialogue`, `dungeons`): the single file still works, directories add to it. None of the three has an order that means anything (dungeons are a `BTreeMap`; quests validate `requires` by id; dialogue is looked up by type or npc), so the "never split" rule of `docs/dev/content.md` does not apply to them |
| ids: level ids are file stems, unique by the file system | still file stems; **unique across campaigns**, else a `ContentError` naming both files (`content::duplicate_id`); the same rule for every id a campaign file declares (a dungeon, a quest, an npc, a creature type, a scenario) |
| `data/creatures/` is the one creature table | and `campaigns/*/data/creatures/`, numbered after the shared block (§3.6) |
| `Content::overworld()` is the one overworld; `local:x,y` is parsed against it | `Content::overworld_of(campaign)`; `Content::local_area(campaign, id)`; the two callers (`content/overworld.rs`, `world/travel.rs`) pass `state.campaign` |
| `Content::fingerprint()` hashes every level and dungeon id | `Content::fingerprint_for(campaign)`: the shared groups plus that campaign's levels and dungeons, so a release save (no showcase) loads in a dev build of the same content, and a bug report's save can be opened |

Levels, scenarios, dungeons, quests, and dialogue keep their ids as they are: no replay, golden manifest entry, test, or doc changes its id. The showcase scenarios keep `scenario = "showcase"` and friends.

**Decision (2026-10-25, question 3): ids are global string slugs, unique across campaigns; a clash is a content error.** If namespacing is ever wanted it comes from a naming convention, a prefix in the slug (`ascmearc_villager`, `foo_kobold_chief`), never from a mechanism. The owner's words: "ids can then be global; namespacing can be faked in later with prefixes." The comparison that was weighed, kept as the rationale:

| | What ids are today | Global slugs (**decided**) | Namespaced slugs (`showcase/hub`) | Numbers or GUIDs |
|---|---|---|---|---|
| In the data | string slugs in TOML (`id = "kobold"`, a level's file stem, `to = "depths:1"`); at load, four tables (creatures, objects, materials, affixes) also get an index in path order (`CreatureId(u16)`, `content/creature.rs:53`; `load_creatures`, `content/mod.rs:966`) | unchanged | every reference (`to`, `creature =`, `population`, `start_level`, `requires`) grows a prefix; a resolver strips it | a second id beside the slug, assigned by hand (a GUID in every file) or by a registry file that every PR edits (a hot file, the playbook's worst case) |
| In saves | `LevelState` keyed by the level id string; actors hold `CreatureId` indices and stacks `ObjectId` indices; affixes by slug; the envelope's fingerprint is the FNV-1a of the ordered id lists (`Content::fingerprint`, `mod.rs:1778`), so a reorder invalidates a save (`docs/dev/saves.md`) | unchanged | every saved level id, quest id, npc id, and stage changes shape: one format bump and a one-time rewrite of the fixtures under `tests/` and `crates/warlock/src/ui/snapshots/` | ids would survive renames and reorders without a bump (the one real gain), at the cost of a mapping table in content and in every save |
| In replays and the golden manifest | `scenario = "showcase"`, action text with slugs (`use:rage`, `talent:hardy`, `ask:E:rooms`, `effect:fireball@30,14`), `replay` files under `runs/` and `tests/golden/` | unchanged: no replay or entry is touched | every scenario name and the `ask`/quest topics in every replay and manifest entry change (the first revision counted them: all of them) | the text syntax would still use slugs, so nothing here is simpler |
| Modding and merges | a second campaign adds files with new slugs; a clash is an error naming both files | a modder picks a prefix for their own; two PRs adding the same slug clash at load, which the content test in CI catches at once | a mod cannot clash with a campaign, at the price above | two mods cannot clash by accident, but a GUID in TOML is unreadable in diffs and in `gen`/`dump` output |
| Cost now | — | zero: this is what the code does | a rename of every id-bearing reference in the repo and a format bump | a new id scheme in every table, every reference, every save, and the dumps |

### 3.4 A run knows its campaign

`WorldState::campaign: String`, set from the scenario at `World::new` (`ScenarioDef::campaign`), saved and hashed (a save is rejected if its campaign is not loaded, like any missing content), and `#[serde(default)]` to the scenario's campaign for the one format bump. Replays need nothing: they record the scenario (`docs/dev/saves.md`, `replay.rs`), and the campaign follows from it. The save summary (`saves/<id>.toml`) adds `campaign` (its name): the Characters screen shows it only when more than one campaign is loaded (dev builds).

### 3.5 New run

| Today | Proposed |
|---|---|
| New run lists every scenario (`ui/new_run_screen.rs`, from `Content::scenarios()`), the showcase first | lists the **start scenario of every campaign of kind `campaign`** first, then, in dev builds (debug or `--dev`) only, every scenario of every campaign (test ones included), grouped under their campaign's name; a release build has one row, so the step is **skipped** and New run goes straight to character creation (race first, `spec/06-input-and-ui.md`) |
| `--scenario NAME` starts any scenario | unchanged (an unknown one is refused as today; in a release build the showcase ones are unknown) |
| a scenario of a campaign that starts in its town or a dungeon (dev conveniences, like `depths` today) | `dev = true` in the scenario file: not listed by New run outside dev builds; still in the binary (a scenario is twenty lines; the content it starts in is the campaign's own) |

The main menu, Continue, and Characters are unchanged.

### 3.6 A campaign's own creatures (decision 2026-10-25, question 1)

The common types (rats, goblins, trolls, the duergar) are shared engine data, because every generator, population list, and kit may draw on them and a type unused by a campaign costs nothing. A campaign adds its **semi-unique** types on top: its people (the `villager` of §6, a type with its own one-liners), its bosses (the barrow wight, the bog hag, the fire drake, the cult priest, the duergar smith), anything that would be out of place in another world. The files are ordinary creature files; nothing in a creature's definition says which world it is from.

| | Shared | Campaign-owned |
|---|---|---|
| Files | `assets/data/creatures/<id>.toml`, one per type, `README.toml` the field reference (`docs/dev/content.md`) | `assets/campaigns/<campaign>/data/creatures/<id>.toml`, the same format, the same README |
| Loading | `ContentFiles::table("creatures")` reads `data/creatures.toml` then `data/creatures/**` in path order (`load_creatures`, `content/mod.rs:966`) | the table reads on, per campaign, after the shared block (§3.3): first every campaign of kind `campaign` in path order, then the test campaigns. `CreatureDef::campaign: Option<String>` remembers the owner (`None` for a shared type) |
| Ids and indices | `CreatureId(u16)` is the index in that order; `Content::creature_id(name)` resolves a slug across the whole table | the same: a campaign's types take the indices after the shared block, so **a release build (no test campaign) numbers a real campaign's types exactly as a dev build does**, because the test block comes last. A second real campaign's block would follow the first's, so adding types to the first renumbers the second: the release bump covers it, as adding a shared type does today (`docs/dev/saves.md`). If that ever bites, the fallback is saving a creature's type by slug, as affixes are saved, and the index becomes a load-time detail; not needed now |
| Slug clashes | `content::duplicate_id` | the same error, naming both files, across the shared block and every campaign (§3.3) |
| Who may name a type | anyone | **only its campaign**: a level legend's `creature = "..."`, a branch's `creatures` and `population` (§9.2), a quest's `kill` target, a dialogue's `creature = "..."`, a vault's spawn, and `[keeper]` are checked at load: a campaign's file may name shared types and its own, never another campaign's (`content::check` per campaign: "`cealdhlaw.toml` names `hermit`, a creature of campaign `showcase`"). A shared file (`data/shops.toml`, `assets/vaults/`) names shared types only |
| Generators | `Canvas::add_patrol` draws a type from the branch's `creatures` by `CreatureId` (`mapgen/rooms.rs:487`, `caves.rs:296`); the local-area kit resolves `deer` once (`mapgen/local/kit.rs`, shared by definition) | nothing changes: the branch's lists are resolved to ids at load, so a campaign type in `population` reaches the initial population and the spawner (§9.3, §9.5) like any other |
| The fingerprint | the ordered id list of the whole table | `fingerprint_for(campaign)` (§3.3) hashes the shared block and the run's campaign's block, so a save of the Ascmearc loads in a dev build (whose table holds the showcase's block after it) |
| The corpse objects | each type with `corpse = true` appends an `ObjectId` (`content/mod.rs:1546`) | appended in the same order as the types, so the objects list keeps the same property |
| The wiki | the creatures table, by name | a section per campaign of kind `campaign` ("Creatures of the Ascmearc"); a test campaign's types are left out, as its scenarios are (§4.2) |
| Dumps and tests | `actors` names types by slug | unchanged; the content test of §12.4 adds the cross-campaign check and a fixture with two campaigns that both add a type |

A campaign-owned type may carry a gear table and a racial weapon (§10.3), be `unique` (§9.2), and have `npc`-less dialogue of its own (`campaigns/<id>/data/dialogue/`): the fields are the shared ones.

## 4. Keeping the test world out of release builds

### 4.1 The options

| Option | In the release binary? | Tests, goldens, benchmarks | The web build | Verdict |
|---|---|---|---|---|
| A. runtime hide: New run hides test scenarios and `--scenario` refuses them unless `--dev` (`cli.rs`, `input.rs::dev` reads `cfg!(debug_assertions) \|\| args.dev` today) | **yes**: the content is embedded whatever the flags; the wasm carries it too | unchanged | carries it | rejected: the owner asked that it not be in release builds, and a binary that holds a flag-locked world is one `--dev` away from showing it |
| B. `#[cfg(debug_assertions)]` around the embed | no in `--release` | **breaks `cargo xtask bench-turns`** (it re-runs itself in release and walks the hub, the vast level, the courtyard, the arena; `xtask/src/main.rs::bench_turns`) and `bench-frames` (release, the vast level), and every `cargo run --release -- --scenario ...` dev check | ok (the web profile inherits release) | rejected |
| **C. a cargo feature `showcase`, on by default** (`warlock-core`: `[features] default = ["showcase"]`; `warlock` and `warlock-cli`: `default = ["showcase"]`, `showcase = ["warlock-core/showcase"]`, with `warlock-core = { ..., default-features = false }` in the workspace table); `build.rs` reads `CARGO_FEATURE_SHOWCASE` and skips `assets/campaigns/showcase/` without it | **no** when built `--no-default-features`: `Content::builtin()` has no showcase files at all | unchanged: every documented command runs with default features | `cargo xtask web` adds `--no-default-features` (it is a release artifact: GitHub Pages and the release zip) | **proposed** |
| C′. the same feature, **off** by default | no in a plain `cargo build --release` | every test, golden, benchmark, capture, dump, and replay command gains `--features showcase` (about fifteen documented commands in CLAUDE.md, every `cargo test` invocation, xtask's own cargo calls) | ok | the honest alternative if "a plain release build is a release" mattered more than the dev commands; not taken |

**Decision (2026-10-25, question 2)**: option C, with the owner's one condition, **test content never appears in a release**, made checkable in three places: the release workflow and `cargo xtask web` pass `--no-default-features`; a CI job builds the release configuration (§4.2); and the test `release_content`, compiled only without the feature, asserts that `Content::builtin()` holds **no campaign of kind `test`**, no scenario, level, dungeon, quest, dialogue, or creature type of one, and that New run's list is one row. A fourth guard costs nothing: `cargo xtask web` and the release workflow run `warlock-cli --scenario showcase` on the built binary and expect "unknown scenario".

### 4.2 Every consumer of the test world, with C

| Consumer | Effect |
|---|---|
| `Content::builtin()` and the ~200 tests that start `"showcase"` (`World::new(content, "showcase", None)` across `warlock-core`, `warlock`, and the CLI) | unchanged: `cargo test --workspace` builds with default features |
| `cargo xtask smoke`, `golden`, `bench-frames`, `bench-turns`, `web-smoke` | unchanged, except `web-smoke`, which drives New run (§3.5: in the release web build the scenario step is skipped) and then checks the save and the resume at turn 5 as today; its walk changes from the hub to the campaign's start |
| `cargo xtask wiki` (CI, release workflow; default features, so the showcase is loaded) | `wiki/scenarios.rs` (the "Scenarios" table) lists only campaigns of kind `campaign` and their listed scenarios: the player wiki never mentions the test world; the dungeons table (§11.4) is by campaign likewise |
| `--content-dir assets` | reads everything on disk (`from_dir`), the showcase included: a dev tool |
| replays and the golden manifest (`scenario = "showcase"` and friends) | verified in dev builds, as today; a release binary refuses them as unknown scenarios |
| the release workflow (`cargo build --release --locked --package warlock --package warlock-cli`) | `--no-default-features`; so the web job (`cargo xtask web`, which passes it itself) |
| CI | a new job: `cargo build --no-default-features -p warlock -p warlock-cli`, `cargo check --no-default-features --target wasm32-unknown-unknown`, and `cargo test -p warlock-core --no-default-features release_content` (a test compiled only without the feature: `Content::builtin()` loads, has exactly the campaigns of kind `campaign` and none of kind `test`, no scenario, level, dungeon, quest, or creature type of the showcase, and New run's list is one row) |
| `warlock-cli` | unchanged; `--scenario showcase` in a `--no-default-features` build says "unknown scenario" |
| saves | the fingerprint per campaign (§3.3) keeps a release save loadable in a dev build |

## 5. The overworld

### 5.1 Entry and the first hours

The player enters the map through **Beorggeat** ("the mountain gate"), a cut in the western range (Grægweall), on the map's west edge: the run starts on the pass cell, a location of its own (terrain `mountain_pass`, `travel_minutes = 30`, no stairs). The road runs east, and **Cēaptūn**, the market town, is five road cells away: an hour and forty minutes at normal speed, within sight by day (sight 9). Nothing on the road hurts a level-1 character: the overworld spawns nothing (§9.1), and the town is safe.

**Decision (2026-10-25, question 5): the pass is a closed way back, "ADOM style for now."** As in ADOM, where the player arrives through the pass at the edge of the Drakalor Chain and cannot leave by it, the player came through Beorggeat and cannot go back: the cell has no stairs and no link, walking west off it is the mountain wall, and look mode says "Beorggeat: the way you came; snow has closed it behind you." Whether it ever opens is the ending's business, which is deferred (§1). Nothing in the data needs a flag for it: a location without `to` is already a cell one stands on.

### 5.2 Size

| | The Wildlands (`levels/overworld.toml`) | Proposed |
|---|---|---|
| Size | 100×50 (5 000 cells) | **120×60** (7 200 cells) |
| Impassable (mountains, deep water) | about 35 % | about 32 %: ~4 900 walkable cells |
| Locations | 3 | 40: the pass, the town, 3 dungeons, 35 reserved sites (§5.5) |
| Walkable cells per location | 1 080 | ~120, at least 6 cells between any two: 40 patches of 6×6 claim 29 % of the walkable cells |
| Across by road, 12 h of daylight a day (`travel_minutes`: road 20, plains 45, grassland 50, forest 90, hills 120, swamp and dense forest 150, `data/terrain.toml`) | 2¾ days | **3⅓ days** (a day's march: 36 road cells, 16 plains, 14 grassland, 8 forest, 6 hills, 5 swamp) |
| Town to the farthest of the first three dungeons | — | a day and a half (§5.6) |

Why not 100×50: with 40 locations it is 81 walkable cells each, and the 6-cell spacing claims 43 % of the land: every third patch is a site, which reads as crowded, and the mountains and the sea leave the coast nowhere to breathe. Why not larger: 150×75 puts the far dungeon two and a half days out and the ends of the map a week apart; the owner asked for "not too vast", and a 120×60 map is already a place of journeys (a trip to Fennmynster and back is three days with a night in the marsh). The travel minutes are shared engine data; they stay. The local areas (80×40 per cell) are unchanged.

**Decision (2026-10-25, question 4): 120×60**, the recommendation (100×60 was the alternative).

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

`M` mountains, `^` the volcano, `h` hills, `f` forest, `F` deep forest, `"` grassland, `.` plains, `s` swamp, `~` rivers, `w` the sea, `=` roads, `b` bridges, `P` the pass (the start), `T` the town, `1` Cealdhlāw (visible from the start), `2` Fennmynster and `3` Fyrdelf (hidden until revealed; drawn here for the owner, as `truth` would draw them), `o` reserved sites (24 of the 35; the rest fit in the plains and along the coast). Rivers flow from the mountains to the sea: Blæcwæter from the volcano's range south through the marsh, Beorhtburna from the western wall east into Blæcwæter. Roads: the pass to the town; the town east through the barrow hills to the ford over Blæcwæter and a village beyond; south from the town over Beorhtburna and along its south bank to Fennhām; north from the town to the wood's edge. No road leads to a hidden dungeon.

### 5.4 Regions (status bar names, `[[region]]` rectangles at full scale; every walkable cell in one, as the overworld test requires)

| Region | Area (x0, y0, x1, y1) | Land | What is there |
|---|---|---|---|
| Grægweall | 0, 0, 9, 59 and 0, 0, 29, 5 | mountains | Beorggeat (the start) |
| Cēapdenu | 10, 20, 50, 40 | grassland, plains, the two roads | Cēaptūn; the bridge over Beorhtburna; farms (reserved) |
| Hlāwdūn | 20, 18, 46, 30 | hills | Cealdhlāw (dungeon 1); a shepherd's hut, a standing stone (reserved) |
| Holegnwudu | 18, 2, 72, 22 | forest around a deep heart (Dēopholt) | a hunters' camp, a hermitage, a ruined tower (reserved) |
| Ascdūn | 70, 14, 108, 28 | hills, Blæcwæter's upper course | Fyrdelf (dungeon 3, hidden); Fordwīc; a watchtower (reserved) |
| Rēcbeorgas | 76, 0, 119, 16 | mountains and the volcano | the crater (a late location, reserved) |
| Ēastmearc | 94, 16, 119, 44 | grassland, plains | a trading post, a port on Grægsæ (reserved) |
| Burnadenu | 10, 40, 64, 48 | Beorhtburna, grassland, the river road | Fennhām; a mill, a ferry (reserved) |
| Lindwudu | 10, 42, 40, 59 | forest, deep in the west (Deorcholt) | a woodcutters' camp, a witch's hut (reserved) |
| Micelfenn | 60, 38, 96, 56 | swamp, Blæcwæter's lower course | Fennmynster (dungeon 2, hidden); a drowned chapel, a hag's island (reserved) |
| Grægsæ | 90, 36, 119, 59 | the sea | the port's lighthouse on the shore (reserved) |

Every name is in §5.8 with its source words and meaning.

### 5.5 Locations: 40 without crowding

| Built now | Reserved in the layout (named by later designs; not built) |
|---|---|
| the pass, the town, the three dungeons (5) | 2 villages, 4 farms and hamlets, a hunters' camp, a woodcutters' camp, a shepherd's hut, a mill, a ferry, a Fordwīc (built as a cell of road: its level later), a hermitage, a monastery, a drowned chapel, a witch's hut, 2 watchtowers, a ruined tower, a fort, a trading post, a port and its lighthouse, 3 caves, 2 barrow fields, a standing stone, a hag's island, a giants' stair, a dragon's lair, the crater (35) |

A reserved site is a plain cell today (its local area generates as its terrain says); the map keeps its spacing so a later level drops in without moving anything.

### 5.6 The travel budget (at normal speed, by day; nights double it since the player rests)

| Journey | Cells | Time | Days (12 h of travel) |
|---|---|---|---|
| The pass → Cēaptūn | 5 road | 1 h 40 | the first hour |
| Cēaptūn → Cealdhlāw | 14 road, 2 hills | 8 h 40 | within a day; back by dusk is not possible after a delve: camp, or sleep in the barrows' first level |
| Cēaptūn → Fennhām → Fennmynster | 48 road, 8 swamp | 16 h + 20 h | a day and a half; a night on the road and a night in the marsh (the marsh's disease checks, `docs/dev/status-effects.md`) |
| Cēaptūn → the ford → Fyrdelf | 64 road, 10 hills | 21 h + 20 h | a day and three quarters |
| The town to the volcano's foot | 64 road, 24 hills | 21 h + 48 h | three days: the far end of the map, for later designs |

Food: a human burns a ration a day (`data/hunger.toml`); the trips are why the provisioner is in the town (§6).

### 5.7 New terrain

| Terrain | Glyph | Why |
|---|---|---|
| `mountain_pass` (location, `travel_minutes` 30, no stairs) | `)` in the mountains' colours | the start cell; walkable, in the mountain wall |
| `volcano` (impassable, glow, a particle plume) | `▲` red | the landmark; presentation only (its glow is the sky's at night) |
| `ford` (road over water, `travel_minutes` 30) | `≈` road colour | Blæcwæter is forded, not bridged, east of the barrows (the Wildlands has `bridge`) |
| `town` (location, stairs down) | `#` warm | the town's cell; `hall` and `dungeon_entrance` exist (`data/terrain.toml` "Locations") |
| `barrow` (location, stairs down) | `∩` pale | Cealdhlāw's entrance (a mound), distinct from a cave mouth |
| `mine` (location, stairs down) | `>` ochre | Fyrdelf's entrance, once revealed |

### 5.8 Names and tone (decision 2026-10-25, question 6)

**The rule**: a place is named by a plain English description of what it is ("market town", "cold barrow", "black water"), **put into Old English**. The names read as old and real to an English reader without a glossary (Tolkien's Rohan does exactly this: Edoras "the courts", Meduseld "the mead hall"), they say what a place is before the player gets there (a fen minster stands in the fen), and they carry no lore of their own: no invented tongue, nothing to look up. People are named in the same register, names an English reader half-knows (Hild, Leofric, Godgifu). The tone the names set: an old, plain-spoken land of farmers and barrow-diggers beyond a gate that has closed behind you, a market town living off the pass, a cold barrow in the downs, a minster drowned in the fen, and a fire mountain smoking over it all, which the old things under the hills answer to.

**Spelling**: the font (`assets/fonts/JuliaMono-Medium.ttf`) has every Old English letter, but the glyph atlas draws Latin-1 and Latin Extended-A only (`GLYPH_RANGES`, `crates/warlock/src/render/atlas.rs:23-31`; anything outside it draws as `?`, `atlas.rs:109`): **æ, þ, ð and the macron vowels ā ē ī ō ū draw; ǣ, ȳ, ƿ (wynn), and ȝ (yogh) do not.** So the names write *ǣ* as **æ** and *ȳ* as **y** (Græg-, Fyr-), and use no wynn or yogh; macrons are kept where the atlas draws them, since they are the difference between *ceap* and *cēap*. (Widening the atlas's first range to U+024F is one line and about 200 slots; the names do not need it, and the name screen's `atlas::draws` check would follow it by itself.) **Ids are ASCII** slugs without the marks (`ceaptun`, `cealdhlaw`, `fennmynster`, `fyrdelf`); the `name` fields hold the spelling. The campaign's id and directory follow the campaign's name: `ascmearc`.

| Name | Old English | Plain English | What it is |
|---|---|---|---|
| **the Ascmearc** | *asce* "ash, cinders" + *mearc* "march, borderland" | the ash march | the campaign and its land (`assets/campaigns/ascmearc/`) |
| **Beorggeat** | *beorg* "mountain" + *geat* "gate, gap" | the mountain gate | the pass, the start; a closed way back |
| **Grægweall** | *grǣg* "grey" (written *græg*) + *weall* "wall" | the grey wall | the western range |
| **Cēaptūn** | *cēap* "trade, market" + *tūn* "enclosure, town" | the market town | the first town |
| **Cēapdenu** | *cēap* + *denu* "valley" | the market vale | the town's region |
| **Hlāwdūn** | *hlāw* "burial mound, barrow" + *dūn* "hill, down" | the barrow downs | the hills east of the town |
| **Cealdhlāw** | *ceald* "cold" + *hlāw* | the cold barrow | dungeon 1 |
| **Holegnwudu** | *holegn* "holly" + *wudu* "wood" | the holly wood | the northern forest (the first revision's "Hollin" was holly already) |
| **Dēopholt** | *dēop* "deep" + *holt* "wood, thicket" | the deep holt | its heart |
| **Lindwudu** | *lind* "lime tree" + *wudu* | the lime wood | the south-western forest |
| **Deorcholt** | *deorc* "dark" + *holt* | the dark holt | its heart |
| **Beorhtburna** | *beorht* "bright" + *burna* "stream" | the bright stream | the western river |
| **Burnadenu** | *burna* + *denu* | the stream vale | its valley and the river road |
| **Blæcwæter** | *blæc* "black" + *wæter* "water" | the black water | the big river, grey with ash |
| **Micelfenn** | *micel* "great" + *fenn* "fen, marsh" | the great fen | the marsh |
| **Fennmynster** | *fenn* + *mynster* "minster, abbey" | the fen minster | dungeon 2: the cellars of the drowned minster |
| **Fennhām** | *fenn* + *hām* "home, village" | the fen village | the marsh village (reserved) |
| **Ascdūn** | *asce* + *dūn* | the ash downs | the hills under the volcano |
| **Fyrdelf** | *fȳr* "fire" (written *fyr*) + *gedelf* "digging, quarry, mine" | the fire diggings | dungeon 3 |
| **Fyrbeorg** | *fȳr* + *beorg* | the fire mountain | the volcano |
| **Rēcbeorgas** | *rēc* "smoke, reek" + *beorgas* "mountains" | the smoking mountains | the volcano's range |
| **Fordwīc** | *ford* + *wīc* "dwelling, trading place" | the ford village | the village at the ford (reserved) |
| **Ēastmearc** | *ēast* + *mearc* | the east march | the eastern grassland |
| **Grægsæ** | *grǣg* + *sǣ* "sea" (written *græg*, *sæ*) | the grey sea | the sea |
| **the Mynster** | *mynster* | the minster | Cēaptūn's temple (the fen minster was its mother house) |
| **the Gæsthūs** | *gæst* "guest" + *hūs* "house" | the guest house | the inn |
| **Leofric** | *lēof* "dear" + *rīc* "power, ruler" | — | the reeve (quest-giver 1; "reeve" is *gerēfa*) |
| **Hild** | *hild* "battle" (St Hild, abbess of Whitby) | — | the sister of the Mynster (quest-giver 2) |
| **Wulfrun** | *wulf* "wolf" + *rūn* "secret, counsel" | — | the mine-factor (quest-giver 3) |
| **Godgifu** | *god* + *gifu* "gift" | — | the innkeeper |

The reserved sites of §5.5 are named the same way when their designs come. Creature names stay plain English (goblin, barrow wight, the drowned): they are the engine's words, shared across worlds.

## 6. The first town

**Cēaptūn** ("the market town"): a market town at the crossroads below the pass, the trading post where the mountain road meets the river roads. A hand-made outdoor level (about 90×32; `kind = "outdoor"`: sky, sun, weather, the pond freezes; `item_level = 1`; `danger = 0`), its `<` on the square leading back to the town's overworld cell. Walled on the pass side, open to the river. Everyone in it is `peaceful` by type or by legend (`docs/dev/content.md`), so it is safe unless the player steals.

| Place | Reuses | Who | What for |
|---|---|---|---|
| The market square | the level, piles | townsfolk (a `villager` type: one-liners, mutters) | the start on arrival; the `<` |
| The general store | `general_store` (`data/shops.toml`) | a keeper (generated name) | a bit of everything; buys gems |
| The Forge (weaponsmith) and the Armourer's | `weaponsmith`, `armourer` | keepers | arms and armour on the floor at the shop's markup |
| The Provisioner's | `provisioner` | a keeper | rations, bread, dried meat for the road (§5.6) |
| The Mynster (the temple) | a `sanctuary` special room (`data/room_effects.toml`: regen, hostiles shun it) | **Sister Hild** (npc; quest-giver 2) | rest and heal; **her services** (§6.2: a cure, a mending, for gold); her chain reveals Fennmynster |
| The Gæsthūs (the inn) | a room, a table of food | **Godgifu the innkeeper** (npc; topics: the barrows, the marsh, the mines, the volcano, the pass) | rumours: the same facts the quest-givers tell, in other words; **never a reveal** (decision 2026-10-25, question 9: quests are the only key to a hidden location; other keys come later, if ever) |
| The reeve's house | — | **Reeve Leofric** (npc; quest-giver 1) | Cealdhlāw chain |
| The mine-factor's office | — | **Factor Wulfrun** (npc; quest-giver 3) | Fyrdelf chain |
| The well, the pond, the granary, a shrine stone, a gallows hill outside the wall | the courtyard's pieces | — | texture; the local areas around the town are its fields |

Angelica root (the remedy that cures every disease when eaten, `docs/dev/status-effects.md`; 30 gold) lies in the general store and the provisioner's as today; the temple is the other way.

### 6.1 The temple's services (decision 2026-10-25, question 8)

**The temple sells services**: a thing no system has yet (shops sell objects, `docs/dev/shops.md`). A service is a **dialogue topic with a price and an effect**, on the npc's dialogue entry, so it is asked for on the dialogue screen like a quest (`World::topics` is the one source of what an npc can be asked, `docs/dev/npcs.md`) and needs no new screen, action, or key: `ask:N:cure` in action text.

| | Design |
|---|---|
| Data | `[[dialogue.service]]` on an npc's (or a type's) entry: `id`, `name` (the topic's row, with the price added by the screen: "A cure for what ails you (60 gold)"), `price` (gold), `does` (`cure` or `mend`), `reply` (said when it is done), `nothing` (said when there is nothing to do: "Nothing ails you, child.") |
| `cure` | `World::cure(player, "prayer")` (`world/status.rs`): every status and every disease whose `cured_by` lists `prayer` ends, incubating ones too; every disease in `data/diseases.toml` gets `prayer` in its `cured_by` (data), so the temple cures what the root cures and what it does not (a disease with no remedy still has the temple). What ended is said by the existing cure events |
| `mend` | `World::mend(player)`: every point of stat damage comes back at once (`Actor::damage: StatDamage` cleared, `docs/dev/stats.md`); refused while a disease shows (it would be damaged again: "Be cured first."), as the daily mending is held while one shows |
| Paying | the price in `gold_coin` (the shops' `currency`) from the player's inventory, through the same purse code `pay` uses in `world/shop.rs`; refused with "You cannot pay." (no time, nothing taken); refused with `nothing` when there is nothing to cure or mend (nothing taken). A service done costs the `chat` action's time. No stream is drawn |
| Price | the data's number, cut by the Pilgrim talent (below); never moved by Presence or Appraise (a temple does not haggle). Shown on the topic row and in look mode's line for the npc ("Sister Hild, who cures and mends for gold") |
| Alignment | none: paying for a cure is a trade, not a deed |
| Talents (the rule: every new system ships with talents) | `[[system]] id = "services"` ("Services: paid cures and mendings"), `min_talents = 1`, `min_class = 0`: **Pilgrim** (Wayfarer tree, 1 rank, no prerequisite): temple services cost half, and a cure also mends one point of each damaged stat (one hook: `service_price` and `cure` read `TalentMods::services`) |
| Wiki | a "Services" table on the town's page, generated from the dialogue data (who, what, price) |
| Dumps | the `quests`-style `topics` of an npc are not dumped today; `ask:N:cure` and the JSON `gold` after it are what tests read |
| Tests | a core test with a fixture npc: `infect:the_shakes rest:24h*3 ask:N:cure` leaves no disease and 60 gold less; `mend` after a disease's damage restores the score; refused without gold or without anything to cure; a replay reproduces it (no stream) |

Sister Hild sells both (`cure` 60 gold, `mend` 40 gold, level-1 prices: a few kills' loot, a day's trade). Topics on her screen: at most the one open quest's three and the two services, within the dialogue screen's nine.

### 6.2 The quest chains

Every quest is one of today's four kinds (`data/quests.toml`: kill, bring, reach, deliver; `docs/dev/quests.md`); no new kind, no abandoning, no cap. Rewards on today's `Reward` (gold, items, XP); the XP figures follow `docs/design/progression.md` §3.4 (a quest is worth a few kills of its level).

| Giver | Quest | Kind and target | Reveals | Teaches |
|---|---|---|---|---|
| Reeve Leofric | C1 "Rats in the barrows" | kill 5 `rat` on `cealdhlaw` (any level) | — | the road east, the first delve, kill tallies |
| | C2 "The barrow's heart" (requires C1) | reach `cealdhlaw:4` | — | depth, the danger rising |
| | C3 "The wight of the barrow" (requires C2) | kill 1 `barrow_wight` on `cealdhlaw` | — | the first named foe (a level-6 creature at danger 6, §10) |
| Sister Hild | A1 "Grave goods" | bring 1 `silver_chalice` (a vault item in Cealdhlāw: the barrow chamber vault, §7.2) | — | vaults, carrying things home |
| | A2 "The fen minster" (requires A1) | reach `fennmynster:3` | **Fennmynster, on accept** | the river road, the marsh, a hidden place found |
| | A3 "What the water keeps" (requires A2) | bring 1 `minster_reliquary` (a vault item deep in Fennmynster) | — | Fennmynster's bottom |
| Factor Wulfrun | Y1 "The foreman's ledger" | deliver `ledger` to Godgifu the innkeeper ("she sends it on with the next cart"): a delivery inside the town, since a recipient needs a hand-made level to stand on and Fordwīc has none yet (local areas place no npcs) | — | deliver quests |
| | Y2 "The lost shaft" (requires Y1, C2) | reach `fyrdelf:2` | **Fyrdelf, on accept** | the far road |
| | Y3 "Fire in the deep" (requires Y2) | kill 1 `fire_drake` on `fyrdelf` | — | Fyrdelf's bottom and the chains' end for now |

The reveal of Fyrdelf waits for C2 (a reach of `cealdhlaw:4`): Fyrdelf starts at danger 8, and a character who has not seen danger 4 is not sent there. Topics fit the dialogue screen's budget of nine (three quests × three topics each is the limit: a giver holds at most three quests at once, which is why each chain is three long).

## 7. The dungeons nearest the town

### 7.1 The three branches (`campaigns/ascmearc/data/dungeons/*.toml`, one file per branch; the fields of `data/dungeons.toml` plus §9's)

| | Cealdhlāw (`cealdhlaw`) | Fennmynster (`fennmynster`) | Fyrdelf (`fyrdelf`) |
|---|---|---|---|
| Where | Hlāwdūn, 16 cells east of the town | Micelfenn, under the ruined minster | Ascdūn, below the volcano |
| Visible | **from the start** (`barrow` on the overworld) | hidden; revealed by A2 | hidden; revealed by Y2 |
| Depth | 6 | 8 | 12 |
| Generator | `rooms` 1–2 (the barrow's passages), `caves` 3–6 (the hill's hollows), 80×21 and 100×40 | `rooms` throughout (cellars and crypts), 80×21, every level with a flooded room's chance raised (`room_effects` on the branch, §7.3: `chance = 25`, `per_level = 2`, `effects = ["flooded", "hushed", "shadowed"]`) | `rooms` 1–4 (galleries), `caves` 5–12 (the deep workings), 120×60 at the bottom |
| Danger (§9.1; none scales with the player) | 1 to 6 (`danger = 1`) | 4 to 11 (`danger = 4`) | 8 to 19 (`danger = 8`) |
| Population at the start (§9.3) | 40 on the rooms floors, 70 in the caves | 40 a floor | 40 on the galleries, 80 in the deep workings |
| Population (§9.2) | rats, bats, beetles, kobolds, spiders, grave worms, skeletons, zombies, ghouls, goblins, the wight | leeches, frogs, drowned, eels, crocodiles, wisps, cultists, ghosts, marsh trolls, the hag, mud golems | ember beetles, duergar, salamanders, trolls, ogres, magma crawlers, basilisks, ash wraiths, cinder elementals, the fire drake |
| Traps | arrow, spear, pit, alarm | pit, frost, alarm, teleport | arrow, spear, fireball, lightning, alarm |
| Secret doors, `hidden_per_depth`, `locks_per_depth` | [1, 2], 1, 1 (as the Depths) | [1, 3], 1, 1 | [0, 2], 1, 2 |
| Loot (`loot`, item level = depth) | [2, 4] | [2, 4] | [3, 5] |
| Key | `rusty_key` | `iron_key` | `brass_key` |
| Temperature | 8 °C | 6 °C, water that freezes nowhere (the marsh is warm: `temperature = 6`) | 10 on top, 24 at the bottom (`[[dungeon.levels]]` overrides: the volcano's heat) |
| Shops | the Depths' one-in-six chance (the shared `chance = 70`) | none (`shops = { chance = 0 }`) | one in four (`shops` on the branch, §7.3: `chance = 100`, `kinds = ["general_store"]`, the duergar keepers' names) |
| Population's gear (§10.3) | goblins with clubs and short bows, the chief in a jerkin | cultists with daggers and a scroll now and then | duergar with pick-axes, axes, mail, and shields; the smith's war hammer |

Each entrance is an overworld location whose `to` is `<branch>:1`; the first level's `<` leads back to the level it was first entered from (the overworld), as today (`spec/07c-generation.md`).

### 7.2 Vaults of their own (`campaigns/ascmearc/vaults/`)

| Vault | Where it stamps | Holds |
|---|---|---|
| the barrow chamber | `cealdhlaw` rooms levels | the silver chalice (A1) on a plinth behind a locked door, its key on the chamber's guardian skeleton |
| the sunken crypt | `fennmynster` | the minster reliquary (A3) at the far end of a flooded room (shallow water: slow, loud steps) |
| the foreman's cabin | `fyrdelf` 1–4 | a pick-axe, a lamp, the branch's key |
| the drake's hoard | `fyrdelf` 12 only (`[[dungeon.levels]]` may name vaults per depth: a small addition) | the fire drake (Y3) and rare loot (`loot = "rare"`) |

The shared vaults (the guard post, the shrine, the flooded cell, the ruins) stamp here too.

### 7.3 Per-branch special rooms and shops (decision 2026-10-25, question 10)

Two optional tables on `DungeonDef`, each overriding the shared settings for that branch; `[[dungeon.levels]]` may override either for a range of depths (a flooded bottom).

| Table | Fields | Default | Where it is read |
|---|---|---|---|
| `room_effects` | `chance` (percent a room is special), `per_level` (at most), `effects` (ids in `data/room_effects.toml`; the pick stays uniform among them) | the shared `chance = 7`, `per_level = 1`, every effect | `mapgen/special.rs::roll` reads the branch's table through the canvas (`Canvas::room_odds`, set in `generate_with` from the `DungeonDef`) instead of `content.room_effects()`; the same one chance roll and one uniform pick, so a branch without the table generates the same map as today (the Depths' snapshots are unchanged) |
| `shops` | `chance` (percent a rooms level with a fitting room has a shop), `kinds` (ids in `data/shops.toml`), `names` (the keepers' names), `keeper` (a creature type, the campaign's own allowed, §3.6) | the shared `chance = 70`, every kind, the shared names, the shared `[keeper]` | `mapgen/shop.rs::make` (`shops.chance`, `rng.index(shops.defs().len())`, `rng.pick(&shops.names)`, `keeper_spawn`) reads the branch's table the same way; the draw count is unchanged |

Content checks: every id named exists, `chance` 0 to 100, `per_level` 1 to 8, `kinds` not empty when `chance > 0`. A test fixture branch with `effects = ["flooded"]` only ever makes flooded rooms, and one with `shops = { chance = 0 }` never a shop. The wiki's dungeons page states each branch's odds and rooms.

## 8. Hidden locations revealed by quests

The owner's addition: two of the three dungeons are not on the overworld until a quest (or one of its stages) reveals them; at least one is visible from the start, so a new character has somewhere to go (Cealdhlāw).

### 8.1 The model: the secret-door precedent

A secret door is, in ground truth, the wall it looks like; `LevelState::secrets` holds what it becomes when found, and finding it sets the terrain (`docs/dev/searching.md`). Because the terrain grid holds the wall, the view, the memory, look mode, and every perception-built dump show a wall without any filter: nothing can leak but through the dev `truth` layer. Hidden locations work the same way.

| | Hidden location |
|---|---|
| In the overworld level file | a legend entry on the plain terrain the cell shows, with `hidden = { id, terrain, to, name }`: `"2" = { terrain = "swamp", hidden = { id = "fennmynster", terrain = "dungeon_entrance", to = "fennmynster:1", name = "Fennmynster" } }`. The cell is a swamp cell in every way: terrain `swamp`, travel 150 min, a local area of marsh (generated as any swamp cell's if the player wanders there), no stairs link |
| In the template | `LevelTemplate::hidden: BTreeMap<Pos, HiddenLocation { id, terrain, to, name }>`; validated like a stairs link (`to` exists in the campaign, `terrain` has `stairs = "down"`, the id is unique across the campaign's overworld; a `hidden` on a cell that already has stairs, or on an impassable cell, is a content error) |
| In the state | `LevelState::hidden: BTreeMap<Pos, HiddenLocation>` (ground truth; saved; hashed; an entry leaves it when revealed) and `WorldState::revealed: BTreeSet<String>` (location ids revealed so far; saved; hashed; left out of the JSON while empty, so runs without a reveal hash as before) |
| The reveal, `World::reveal_location(id) -> bool` | finds the campaign's overworld: if it is instantiated, takes the entry out of `hidden`, `set_terrain(pos, entry.terrain)`, pushes `StairsLink { pos, dir: Down, to }` onto `stairs`, and **marks the map**: writes `memory[pos] = Some(entry.terrain)` (decision 2026-10-25, question 7: every reveal marks; there is no "only told" variant and no flag for it); if it is not yet instantiated (a scenario that starts in the town before the player has stepped outside), the id goes into `revealed` and `instantiate_level` applies every revealed entry when the overworld is made. Returns whether anything changed (a second reveal of the same id is nothing) |
| The hook in quests | `QuestDef::reveals: Option<Reveal { location: String, when: RevealWhen }>`, `when` one of `offered` (the giver tells you where it is as they ask: for a quest that is a tip), `accepted` (the default: "she marks the fen minster on your map"), `done`. `world/quest.rs` calls `reveal_location` in `ask_quest` at the matching transition (`Stage::Offered`, `Stage::Active`, `Stage::Done`, the three places the stage is set), before the quest's own line, so the message order is the giver's words, then "Sister Hild marks the fen minster on your map." (`Spent::NoTime`, as talk is) |
| Content checks | a quest's `reveals.location` names a hidden location of the quest's campaign's overworld; the same location may be revealed by several quests (the first to happen wins; the rest change nothing); a reach quest whose target lies behind a hidden location is fine (the player has to find the way) |

### 8.2 What the player sees and remembers

| Moment | The map (view, memory) | Look mode | The quest log | Messages |
|---|---|---|---|---|
| Before the reveal | the cell is swamp; walking onto it and pressing `>` explores the marsh's local area ("You explore the swamp.") | "a swamp" | A2 not yet offered: nothing | — |
| The reveal (accepting A2 in the temple) | the overworld is not in view; its memory now holds the entrance at the cell (drawn as remembered terrain when the player next looks at the overworld) | — | "The fen minster (active, from Sister Hild): Reach Fennmynster 3. The minster lies in Micelfenn, east of Fennhām." (the objective adds the quest's `where` text: a `where` field on the quest, free prose, since the rules never tell the cell) | "Sister Hild marks the fen minster on your map." / "Sister Hild tells you where the fen minster lies: in Micelfenn, east of Fennhām." |
| Walking there after | the entrance in view when within sight; in memory as last seen | "Fennmynster" (the location's `name`; `hidden.name` becomes the stairs' name the way `place_name` names levels) | as above, then "Done" | "You enter Fennmynster." on `>` |
| Dev reveal (F2; `truth`) | hidden entrances drawn with their entrance glyph, as `S` marks secret doors; marked dev-only on screen, as F2 is | — | — | — |

A hidden location the player never hears of is never found: there is no search check on the overworld, and the cell is honestly a swamp. **Decision (2026-10-25, question 9): quests are the only key.** No rumour at the inn, no exploration, no second way reveals a location now; alternative unlocks come later, if ever, and the `reveals` hook is where they would plug in.

### 8.3 The perception-snapshot rule

Nothing new is needed to keep the rule: `PerceptionView` reads `terrain` and `memory` (and `stairs` through them), which hold the plain terrain until the reveal. `LevelState::hidden` is read by exactly two readers: `reveal_location` and the dev `truth` dump (`dump.rs::truth_rows`, which draws `S` for secrets today and would draw the hidden entrance's glyph), both already on the dev-only side of the line (`Layer::is_dev_only`). A core test asserts that before the reveal the `view`, `memory`, and JSON dumps of a run standing beside the cell contain no entrance glyph, no `fennmynster` level id, and no location name, and that after `ask:N:accept-fen_minster` the memory dump shows the marked cell (if marked) and `>` on it enters `fennmynster:1`. The renderer draws from the snapshot only, as today.

### 8.4 Saves, replays, determinism

Saved: `hidden` (per level) and `revealed` (per run), both hashed; a save naming a location this content does not have is inconsistent (like a quest id). A replay's `ask:N:accept-fen_minster` step reveals deterministically: no stream is drawn, and the reveal is part of that step's state hash. The overworld's region test still covers hidden cells (they are plain cells). The content fingerprint is unchanged by hidden locations (they are not ids in a numbered table).

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

The constant is the rule; a scaled area is the exception its data names. **Decision (2026-10-25, question 13): no player-scaled area in the start area.** None of the three dungeons scales and no reserved cave becomes a training cave; the `player` table stays in the data model, loaded, tested with a fixture, and unused by the Ascmearc. The shorthand `danger = 4` in a branch file means `{ base = 4, per_depth = 1 }`.

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

A generated floor is laid down full. Its **population** is a count from its walkable cells (`mapgen` knows them after validation): `population = { per_1000 = 35, least = 25, most = 80 }` on the branch (defaults; a hand-made level with `danger > 0` gives `population = N` outright). The creatures are drawn by §9.2's weights from the stream `population/<level id>` after the loot (so a map, its traps, secrets, and loot are the same with any population), each placed on plain floor away from the arrival staircase (never within 8 cells of it: the player arrives there) and given a short patrol route where one fits (`Canvas::add_patrol`, validated as the generator's own patrols are), else a post. They roll their type's gear table (Â§10.3: from `gear/<level id>`, after the population draw, so a drawn kobold has its spear as a level-placed one does).

| Floor | Walkable cells (about) | Population | Of which on patrol |
|---|---|---|---|
| rooms, 80×21 (Cealdhlāw 1–2, Fennmynster, Fyrdelf 1–4) | 1 000–1 200 | **35–40** | most |
| caves, 100×40 (Cealdhlāw 3–6) | 1 800–2 000 | **65–70** | about half (caves give fewer loops) |
| caves, 120×60 (Fyrdelf 5–12) | 3 000–3 600 | **80** (the ceiling) | about half |
| the showcase's Depths (80×21 and 120×60) | | unchanged: `population` not given, the two or three patrols of today |

The ceiling is the turn budget's: `cargo xtask bench-turns` gets a new entry (a Fyrdelf cave floor at full population, 16 ms a turn like the vast level) and the ceiling comes down if it is missed; creature sight is computed only for hostiles that could see the player (`docs/dev/creatures.md`), which is what keeps 80 affordable.

### 9.4 Expected XP per floor against the curve

The curve (`data/experience.toml`, `docs/dev/experience.md`; `docs/design/progression.md` §3.2): XP totals of 767 for level 2, 2 431 for 3, 5 084 for 4, 8 797 for 5, 13 628 for 6, 26 840 for 8, 45 064 for 10, 68 601 for 12, 114 464 for 15; a creature of level *L* is worth `5 × L + 10`. With the weights of §9.2 the average level on a floor of danger *D* is about `D − 1` (`D` on the first floor), so a floor's initial population is worth about `population × (5 × (D − 1) + 10)` to a character it does not out-level, and the spawns of a day's delve (§9.5: about 2.4 an hour) add a quarter again.

| Dungeon | Floors × population | Initial XP | With a day's spawns | Running total | Level at the bottom |
|---|---|---|---|---|---|
| Cealdhlāw (danger 1–6; 40, 40, 70, 70, 70, 70) | 600 + 700 + 1 470 + 1 750 + 2 100 + 2 450 | 9 100 | 11 400 | 11 400 | **5**, nearly 6 |
| Fennmynster (danger 4–11; 40 × 8) | 1 000 … 2 400 | 13 600 | 17 000 | 28 400 | **8**, going on 9 |
| Fyrdelf (danger 8–19; 40 × 4, 80 × 8) | 1 800 … 2 400, 5 200 … 8 000 | 55 200 | 69 000 | 97 400 | **14**, going on 15 |

Three full clears bring a baseline character to level 14 of 60, with the race and class factors (72 % to 175 %) moving it a level either way, the above-level bonus (5 % a level, up to 4) helping the brave, and grey kills (the zero-difference bands) taking from those who linger in Cealdhlāw at level 10. In kills: Cealdhlāw are about 430 kills (the curve asks 38 for level 2, 106 for level 5); a careless character dies to them long before (the owner's "or die trying").

### 9.5 Spawning over game time, on the player's level

| Field (`DungeonDef`, or a hand-made level's `[spawn]` table) | Default | Meaning |
|---|---|---|
| `every` | `15m` | game time between attempts on the level (150 normal turns): `LevelState::spawn_at: GameTime`, the next attempt's time (saved) |
| `chance` | 60 | percent that an attempt spawns something |
| `cap` | the floor's population count (§9.3) | an attempt is skipped while the level holds that many creatures hostile to the player (by `World::stance`; peaceful folk, wildlife, and shopkeepers do not count): a floor **refills toward its population** as it is cleared, about 2.4 an hour, 24 over a night's rest |

The hook: at the end of `World::run_turn` (`world/schedule.rs`), after the clock advanced, `spawn::tick(self)` makes every attempt whose time has come on the **current level** (several, if a rest passed hours: a rest to dawn lets the dungeon refill, which is ADOM's feel and the cost of resting below; `rest` stops when a creature comes into view, as today). **Where**: a cell drawn from the stream, walkable, unoccupied, not stairs, not a door, not a shop's floor, not in a sanctuary (`room_effect_at`), **not in the player's view by the perception rules** (the cells `perceive.rs` would mark visible now: line of sight and light, computed by the same functions, so a lit far cell counts as seen and a dark near one does not), and **at least 8 cells away** (Chebyshev). Up to 32 draws; none free, the attempt is skipped. The creature gets a short route where one fits, else a post (it waits until it sees or hears something, `world/ai.rs`), its gear (Â§10.3, from `gear/<actor id>`), and is an ordinary actor from then on: `next_actor_id`, saved, hashed.

**Floors the player is not on stay frozen** (decision 2026-10-25, question 11): no clock runs for them, as no creature there gains energy today (`docs/dev/time.md`), and `spawn_at` is moved to the arrival time when the player returns, so nothing happened while they were away. The alternative weighed was a catch-up on re-entry (the attempts the elapsed time allows, placed out of sight of the arrival cell, from the level's own stream); it was a day's work more and it refills a cleared floor behind the player's back, and a cleared floor as a safe camp is a thing players value. A `catch_up` field can be added later without touching the rest.

**Decision (2026-10-25, question 16): nothing spawns on the showcase's hand-made levels** (the arena, the trap hall, the hub, the Depths' hand-made neighbours): every one keeps `danger = 0`, so every golden walk and replay there stays what it is.

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

## 10. The creatures the first dungeons need (and the gear humanoids carry)

Today's bestiary: nine types besides the player, six of them hostile (rat 1, bat 1, beetle 2, kobold 2, kobold archer 3, kobold shaman 4; `assets/data/creatures/`). Three to five types per danger step keeps a floor varied without a cast of a hundred; the three branches span danger 1 to 19 with overlaps, so **32 new types** in three waves, one file each on the creature XP guideline (`5 × level + 10`, `data/experience.toml`), all on existing fields (natural attacks with `on_hit` statuses and diseases, `natural` armour, `resist`, `combat`, `vision`, `speed`, `corpse`/`on_eat`, `loot`) plus §10.3's gear table. No new combat mechanic is needed for any of them; the casters announce until the spell system lands, as the kobold shaman does.

**Decision (2026-10-25, question 14): the whole list is scheduled**, all three waves, each its own PR (§13: N1, N2, N3), none cut for the first release. **Where** says whose file a type is (§3.6): the common types are shared engine data in `data/creatures/`; the campaign's people and bosses are in `campaigns/ascmearc/data/creatures/`. **Gear** is the type's gear table (§10.3), the racial weapon first.

### 10.1 The waves

| Wave | Type | Level | Kind | Where | Gear (§10.3) | Notes |
|---|---|---|---|---|---|---|
| Town | villager | 1 | peaceful | campaign | — | the townsfolk of §6: one-liners and mutters, `budge = "always"` |
| Cealdhlāw | grave worm | 1 | melee | shared | — | slow, bites, poison 1 |
| | barrow spider | 2 | melee | shared | — | quick, poison 2, darkvision |
| | goblin | 2 | melee | shared | racial **club** 90 %; leather cap 30 % | opens doors, loud |
| | goblin archer | 3 | ranged | shared | racial **short bow** 90 % with arrows; club 50 % | drops its arrows |
| | skeleton | 3 | melee | shared | an old sword or spear 30 % | no corpse, resists piercing 50, vulnerable to bludgeoning |
| | zombie | 3 | melee | shared | — | slow, lots of hit points, grave rot on hit |
| | giant centipede | 3 | melee | shared | — | poison 2, darkvision |
| | barrow ghoul | 4 | melee | shared | — | bleeding on a critical; quick |
| | goblin chief | 5 | melee | shared | racial **club** 90 %; short sword 40 %; leather jerkin 70 %; coins | loot 50 |
| | barrow wight | 6 | caster (announced) | **campaign** | an ancient long sword 60 %, magic 25 % | C3's target; cold touch (cold damage), resists cold, darkvision, no corpse |
| Fennmynster | giant leech | 4 | melee | shared | — | bleeding on hit |
| | giant frog | 4 | melee | shared | — | quick, swims (shallow water at no cost: a `wades` flag is a small addition, else none) |
| | drowned | 5 | melee | shared | — | undead, slow, filth fever on hit |
| | marsh eel | 5 | melee | shared | — | only in water cells (placed there by the spawner when the type says `habitat = "water"`: a small addition, or dropped from the list) |
| | crocodile | 6 | melee | shared | — | strong bite, natural armour 2/3 |
| | cultist | 6 | melee | **campaign** | racial **dagger** 90 %; a scroll 15 % | the cult is this world's |
| | will-o'-wisp | 7 | caster | shared | — | light `glow`, flies, evasive (Defence high) |
| | ghost | 8 | melee | shared | — | resists physical 50, no corpse, darkvision |
| | mud golem | 8 | melee | shared | — | slow, protection 5, no corpse |
| | cult priest | 9 | caster | **campaign** | racial **dagger** 90 %, magic 20 %; a scroll 40 % | announces curses; loot 50 |
| | marsh troll | 9 | melee | shared | — | regenerates quickly, big hit points |
| | the bog hag | 11 | caster | **campaign** | a staff 80 %, magic 50 % | `unique = true`: once per run (§9.2) |
| Fyrdelf | ember beetle | 8 | melee | shared | — | fire bite, resists fire 50, glow |
| | duergar miner | 9 | melee | shared | racial **pick-axe** 90 % (a weapon in its hands; creatures do not dig); leather cap 40 % | darkvision |
| | duergar warrior | 11 | melee | shared | racial **hand axe** 90 %; small shield 70 %; chain mail 60 %, magic 5 % | |
| | ogre | 12 | melee | shared | racial **great club** 80 % | huge hits, slow |
| | fire salamander | 12 | melee | shared | — | fire, resists fire 100, lights the way |
| | troll | 13 | melee | shared | — | regenerates; the marsh troll's big brother |
| | duergar smith | 13 | melee | shared | racial **war hammer** 95 %, magic 15 %; chain mail 80 % | loot 75 |
| | magma crawler | 14 | melee | shared | — | fire, vulnerable to cold, burrows (flavour) |
| | ash wraith | 15 | caster | shared | — | announced; cold, no corpse |
| | basilisk | 16 | melee | shared | — | its bite: a slowing status (Chilled exists; else poison 4) |
| | cinder elemental | 17 | melee | shared | — | fire, immune to fire, no corpse, bright |
| | fire drake | 19 | caster/ranged (its breath: the `fire_breath` effect exists) | **campaign** | — (its hoard is the vault's) | Y3's target; the drake's hoard vault; `unique = true` |

The existing kobolds get tables too (kobold: racial **spear** 90 %; kobold archer: racial **short bow** 90 % with arrows; kobold shaman: bone amulet 60 %), so a Depths kobold placed by the generator (`Canvas::add_patrol` gives it nothing today, `mapgen/mod.rs:290`) fights as the arena's do.

### 10.2 Factions

`undead` (hostile to the player and to `kobolds`, `goblins`), `goblins` (hostile to the player, `vermin`, `kobolds`), `marsh` (Fennmynster's living things), `cult` (the cultists; hostile to the player and `marsh`), `duergar` (hostile to the player; neutral to `fire` things), `fire` (the elementals, the salamanders, the drake). Factions are shared data (`data/factions.toml`, a single file whose order means something, `docs/dev/content.md`): the six rows land with the first wave that needs each. Wars inside a dungeon are the arena's lesson applied: a floor at danger 9 where cultists and marsh things fight gives a careful player kills to watch.

### 10.3 Gear on humanoids (decision 2026-10-25, question 14)

**The requirement**: humanoid creatures have a chance to spawn with equipment they benefit from (they wield or wear it, and combat reads it) and drop when they die; each tribe or type has an affinity for its racial weapon and a high chance to carry it.

**What exists**: a level legend or a vault gives a placed creature fixed items with `carry`, `wield`, `wear` (`content/level.rs:262-268`; the arena's kobolds, `assets/levels/arena.toml:52-54`: `carry = ["spear"], wield = "spear"`; the shopkeeper's `[keeper]`, `data/shops.toml`), resolved by `equip_ids` (`level.rs:1151`: `wield` ids into Wield/Ready slots, `wear` ids into the rest, through `equipment::free_slot` and `Equipment::put`) into `CreatureSpawn { carry, equipment }`, copied onto the actor in `instantiate_level` (`world/travel.rs:454`). **Generated creatures get nothing** (`Canvas::add_patrol`, `mapgen/mod.rs:290`: `carry: Vec::new()`), so a Depths kobold fights bare-handed at 1d3. Combat reads a creature's equipment through the same `Actor` methods as the player's: `weapon` (the first hand slot with an `attack`; a wielded weapon wins over the type's `attack` in `Actor::melee`, `world/actor.rs:1033`), `defence`, `protection`, `resistances`, `mods` (the affixes of what is worn, `magic/mods.rs:173`), `launcher` and `ammunition` (the AI shoots only with both, `world/ai.rs:242`). On death `creature_dies` (`world/fight.rs:937`) drops the whole inventory, worn and wielded included, on the cell with `add_to_pile`, then the `loot_of` roll (`world/loot.rs:74`, stream `drop/<actor id>`, `Source::Gear(creature level)` for the material, `magic/loot.rs::roll_with`). The player's starting kit rolls lines with alternatives and a `magic` grade through `kit::roll` (`kit.rs:96`, stream `character/kit/<spec>`) and `loot::make` (`magic/loot.rs:154`: a chosen base object made magic at an item level), equipping with `Pack::equip` (a piece with no free slot stays carried, no error). Creatures have fixed weapon-skill ranks (`weapon_skills = { spears = 1 }`, `CreatureDef::weapon_skills`, copied at spawn, `travel.rs:470`; they never train).

**The design** reuses all of it: a gear table on the type, rolled by the kit's machinery into a `CreatureSpawn`'s `carry` and `equipment`, read by combat as today, dropped at death as today.

| | Design |
|---|---|
| Data on the type (`data/creatures/README.toml`) | `racial_weapon = "spear"` (an object with an `attack`; optional) and `[[gear]]` lines: `chance` (percent), `one_of` (objects, one drawn uniformly; `"arrow*20"` counts as the kit's lines do), `keep` (`wield`, `wear`, or `carry`; default by the object: a weapon, shield, launcher, or missile is wielded, armour and jewellery worn, else carried), `magic` (percent that the piece is rolled magic, default 0). The `[[gear]]` lines are tried in order |
| The racial weapon | sugar for the first line, `{ chance = 90, one_of = [racial_weapon], keep = "wield" }` (a type may write the line itself with another chance), **and a content check** that the type's `weapon_skills` holds at least rank 1 in the weapon's skill (`ObjectDef::skill`; "`goblin` has racial weapon `club` but no rank in `clubs`"): the affinity is ranks in the skill, which `Actor::attack_skill` and `benefit` already apply. The wiki states it |
| Rolling, `gear::roll(content, rng, creature, item_level) -> Pack` (`crates/warlock-core/src/gear.rs`, new; `Pack` from `kit.rs`) | per line: `rng.chance(chance)`, one `rng.index` among `one_of`, the material by `variant::draw` with `Source::Gear(creature level)` (as drops: primitive for a kobold, steel for a duergar), and with `magic` a roll of `loot::make(content, rng, object, count, item_level, LootGrade::Magic)` (rarity by `roll_rarity` with `Luck::default()`: the player's luck is the player's), then `Pack::put` and `Pack::equip` (a piece that would overload the creature, `burden` against its size's limit, is skipped: nothing carries what it cannot lift; a piece with no free slot stays carried). The `item_level` is the floor's (`Content::item_level`, `content/mod.rs:2432`: a hand-made level's `item_level`, else the branch depth), so a deep floor's duergar wears better mail |
| Who rolls it | every creature made from a type without fixed items: `Canvas::add_patrol` (the generators' patrols), `mapgen/population.rs` (§9.3), `spawn::tick` (§9.5), and a level legend or vault entry **without** `carry`/`wield`/`wear` (one with any of them keeps its fixed kit: the arena, the hub's kobold, the keeper). The local kit's deer have no table: nothing |
| Streams | at generation, `gear/<level id>`, consumed in canvas order **after** `place_loot` (so the map, traps, secrets, loot, and the initial population's types and cells are the same with or without gear; `rng.rs` gets the label); at play, `gear/<actor id>` for a spawn (the actor's id is fixed at `next_actor_id`, as `drop/<actor id>` keys the drop). Neither touches `gameplay` or `map/…` |
| Combat and AI | nothing new: the wielded weapon wins over the natural attack, worn armour counts in `defence` and `protection`, the archer's bow and arrows make it shoot, and ammunition it looses lands where it fell. The type's `attack` stays as what it fights with when the roll gave it nothing |
| Death | nothing new: `creature_dies` drops it all; a magic piece lands unidentified, as loot does (`docs/dev/magic-items.md`) |
| Perception | nothing new: "Others see what a creature wears and wields; never what else it carries" (`Actor::equipment`'s rule); look mode names it ("a goblin, wielding a club") |
| Balance | a type's `level`, `xp`, and hit points are set with its usual gear in mind (kobold 2 already is: the arena's spear). The `actors` dump shows the gear; `gen` shows nothing new (it draws the map) |
| Talents (every new system ships with talents) | `[[system]] id = "creature_gear"` ("The gear creatures carry and drop"), `min_talents = 1`, `min_class = 0`: **Scavenger** (Delver tree, 1 rank, no prerequisite): the gear a creature you kill drops is identified as it lands (one hook in `creature_dies`, through `World::identify` as the dev `identify` does). A second later: a thief's **Disarming blow** (a critical may knock a weapon from a hand), when a disarm action exists |
| Wiki | the creatures table gains "Gear" (the racial weapon and each line with its chance and magic percent), by the rule the wiki is exact |
| Tests | `gear_tests.rs`: with a fixture type, over many seeds the racial weapon is wielded whenever carried, weapons go to hand slots and armour is worn, `magic = 0` never makes a magic piece and `magic = 100` always does at the floor's item level, nothing over the carrying limit, no line draws when `chance = 0`, the same seed gives the same gear; a content test for the racial-weapon rank check; a spawn through `spawn::tick` and one through the population have gear from their own streams and the map hash is unchanged by the table. **What moves once**: the Depths' mapgen snapshots do not (the canvas's cells are the same; the spawns' `carry` is in the template, so their insta snapshot of the template does, reviewed once), and a replay that fights a generated kobold (none under `tests/` today; `runs/` are checked) is re-created by the playbook's rule. The arena's goldens and replays are unchanged: fixed kit |

## 11. What the player sees

### 11.1 Messages

| Moment | Text |
|---|---|
| the run starts (the start scenario's `arrival` line) | "The pass opens on the Ascmearc. Far to the east, Fyrbeorg smokes." |
| the town's arrival | "Market day in Cēaptūn: the square is loud." |
| a reveal | "Sister Hild marks the fen minster on your map." |
| a service (§6.1) | "Sister Hild lays her hands on you. The shakes leaves you." / "Nothing ails you, child." / "You cannot pay." |
| Beorggeat in look mode | "Beorggeat: the way you came; snow has closed it behind you." |
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
| look mode | a revealed location by its name; a hidden one by its plain terrain; a creature with what it wields and wears, as today |
| the dialogue screen | a service as a topic row with its price ("A cure for what ails you (60 gold)"); a snapshot case |
| the death screen | "in Cealdhlāw 3" as today's place names |

No golden screenshot: nothing here is something only the GPU produces (the overworld's sun, moon, and weather are covered by the Wildlands entries that exist).

### 11.3 The CLI and dumps

| Addition | Where |
|---|---|
| `--scenario ascmearc` (the start), `ceaptun`, `cealdhlaw`, `fennmynster`, `fyrdelf` (`dev = true` scenarios starting in the town and on each branch's first level, the character chosen like `depths`) | `campaigns/ascmearc/scenarios/` |
| `warlock-cli gen --level cealdhlaw:3 --seed warlock` | as today for any branch |
| the `locations` layer: the overworld locations the player knows, from the memory grid (perception): `(18, 28) Cēaptūn`, `(34, 26) Cealdhlāw`; `no known locations` | `dump.rs` |
| the `truth` and `actors` layers: hidden locations by their entrance glyph; their header `danger N population N/M spawn next 08:15:00` | dev-only |
| JSON: `campaign`, `known_locations: [{pos, name}]`, `due`… unchanged; `danger` only under `--state` | |
| a dev action `reveal:ID` (reveal a hidden location, no time, for captures and tests), listed with the dev actions | `world/dev.rs` |
| a dev action `spawn` (make one attempt on the current level now, no time) | `world/dev.rs` |
| `ask:N:cure`, `ask:N:mend` (a service, by its id, as a topic) | `world/talk.rs`, nothing new in the syntax |
| the `actors` layer shows every creature's gear, as it shows what it carries today; a magic piece "(unidentified)" | `dump.rs`, dev-only |

### 11.4 The wiki (the game is vague, the wiki is accurate)

| Page | Generated table |
|---|---|
| The world (new section: regions, the pass, the town, the names of §5.8 with their meanings; the overworld map as a code block at 1:1, generated from the level file **with the hidden locations drawn and named**: decision 2026-10-25, question 12, the wiki tells the truth, hidden locations included, with no spoiler hiding; the dungeons page says what reveals each. Spoilers may be handled differently when the ultra-ending material arrives, later) | `overworld.md` |
| Dungeons (new page) | per branch: depth, generator, size, danger per floor, the population with each type's level and its weight at each floor, `every`, `chance`, `cap`, out-of-depth, the special-room and shop odds (§7.3); "revealed by: The fen minster (Sister Hild)" |
| Creatures | the 32 new rows come by themselves, in a section per campaign for the campaign-owned ones (§3.6); a **Gear** column: the racial weapon and each line's chance and magic percent (§10.3) |
| Quests (new page: name, giver, kind, target, reward, requires, reveals) | `quests.md` |
| The town | a **Services** table: who, what, the price (§6.1) |
| Scenarios → Campaigns | lists campaigns of kind `campaign` and their listed scenarios; the showcase leaves the player wiki |
| Talents by system | the `danger`, `creature_gear`, and `services` rows by themselves |

## 12. Data and code sketch

### 12.1 Files

| Path | Content |
|---|---|
| `assets/campaigns/showcase/campaign.toml` | `id = "showcase"`, `kind = "test"`, `start = "showcase"` |
| `assets/campaigns/showcase/{levels,scenarios}/` | today's `assets/levels/*.toml` and `assets/scenarios/*.toml`, moved (a `git mv`; the vast generator writes to the new path: `xtask/src/vast.rs`, `gen-vast --check`) |
| `assets/campaigns/showcase/data/{dungeons,quests,dialogue}/showcase.toml`, `data/creatures/{hermit,gardener,bard}.toml` | the Depths; the four quests; the hermit, gardener, and bard dialogues and types |
| `assets/data/dialogue.toml` | the shopkeeper's entry stays |
| `assets/campaigns/ascmearc/campaign.toml`, `levels/{overworld,ceaptun}.toml`, `scenarios/{ascmearc,ceaptun,cealdhlaw,fennmynster,fyrdelf}.toml`, `data/dungeons/{cealdhlaw,fennmynster,fyrdelf}.toml`, `data/quests/{leofric,hild,wulfrun}.toml`, `data/dialogue/{ceaptun}.toml` (the npcs, the villager's lines, Sister Hild's services), `data/creatures/{villager,barrow_wight,cultist,cult_priest,bog_hag,fire_drake}.toml`, `vaults/{barrow_chamber,sunken_crypt,foremans_cabin,drakes_hoard}.toml` | the campaign |
| `assets/data/creatures/<26 files>`, and `[[gear]]` on `kobold`, `kobold_archer`, `kobold_shaman`; `README.toml` (`racial_weapon`, `[[gear]]`) | §10 |
| `assets/data/terrain.toml` | §5.7 (appended by group, under "Locations" and "The overworld") |
| `assets/data/factions.toml` | §10.2's six factions |
| `assets/data/diseases.toml` | `prayer` in every `cured_by` (§6.1) |
| `assets/data/talents/{quiet_delver,sixth_sense,scavenger,pilgrim}.toml`, `talents.toml` `[[system]]` `danger`, `creature_gear`, `services` | §9.7, §10.3, §6.1 |
| `crates/warlock-core/build.rs`, `Cargo.toml` ×3, `.github/workflows/{ci,release}.yml`, `xtask/src/web.rs` | §4 |
| `content/campaign.rs` (new), `content/mod.rs` (tables, `in_dir` over campaigns, the creature blocks, checks per campaign, `fingerprint_for`), `content/creature.rs` (`campaign`, `racial_weapon`, `gear`), `content/overworld.rs`, `content/dungeon.rs` (§9 fields, `population`; §7.3 `room_effects`, `shops`), `content/quest.rs` (`reveals`, `where`), `content/level.rs` (`hidden`, `danger`, `population`, `[spawn]`), `content/dialogue.rs` (`service`) | §3, §6.1, §7.3, §8, §9, §10.3 |
| `gear.rs` (new: `gear::roll`), `mapgen/mod.rs` (`add_patrol` rolls gear; `Canvas::room_odds`), `mapgen/special.rs`, `mapgen/shop.rs` (the branch's odds), `world/service.rs` (new: `cure`, `mend`, the purse), `world/talk.rs` (services in `topics` and `ask`), `world/spawn.rs` (new: `tick`, `attempt`, the cell and route picks; `World::danger`), `world/schedule.rs` (one call), `world/travel.rs` (one line resetting `spawn_at` on arrival; `instantiate_level` applies `revealed` and rolls legend gear), `mapgen/population.rs` (new: the initial population), `world/reveal.rs` (new: `reveal_location`), `world/quest.rs` (three calls), `world/dev.rs` (`reveal`, `spawn`), `rng.rs` (`spawn`, `population`, `gear`), `dump.rs` | |
| `crates/warlock/src/ui/new_run_screen.rs`, `save/characters.rs` (the summary's campaign), `cli.rs` (nothing: `--scenario` is any id) | §3.5 |
| `xtask/src/wiki/{scenarios,dungeons,quests,overworld,services}.rs`, `creatures.rs` (the campaign sections, the Gear column), `wiki/src/SUMMARY.md` | §11.4 |
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

Gear (§10.3) and services (§6.1) add no state: a creature's gear is its `inventory` and `equipment`, saved today; a cure or a mending changes what is already saved. Per-branch odds (§7.3) are content.

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
| a creature's gear | `gear::roll`, called where a creature is made from a type without fixed kit (`add_patrol`, `population`, `spawn::attempt`, `instantiate_level` for a legend entry without `carry`/`wield`/`wear`) |
| the racial weapon's affinity | the content check on `weapon_skills` at load; the ranks do the rest through `Actor::attack_skill` |
| Scavenger | `creature_dies` identifies what falls |
| a service | `talk.rs::ask` → `service::perform` (the price, then `World::cure` or `World::mend`) |
| Pilgrim | `service::price` and `service::perform` read `TalentMods::services` |
| a branch's special rooms and shops | `special::roll` and `shop::make` read `Canvas::room_odds` / `Canvas::shop_odds`, set once in `generate_with` from the `DungeonDef` |
| a campaign's own creatures | `load_creatures` numbers the blocks; `content::check` refuses a cross-campaign reference |

### 12.4 Tests each PR carries

Unit tests in `warlock-core` for the rules; a content test that every campaign's links stay inside it, that ids are unique, and that a campaign names no other campaign's creature (a fixture with two campaigns that both add a type); `release_content` under `cfg(not(feature = "showcase"))`; the perception leak test of §8.3; the spawn property test of §9.6; the gear property test and the racial-weapon rank check of §10.3; the service tests of §6.1; the per-branch odds fixtures of §7.3; insta snapshots of the three branches' first levels (`mapgen/snapshots/`), of the town's dumps, and of the New run and Characters screens (`ui/screen_snapshots.rs`); a replay per campaign scenario, created and verified; `cargo xtask wiki` in CI; no golden entry (the manifest's Wildlands entries keep their references, since the Wildlands only moves directory).

## 13. Implementation plan

Issue-ready items; each one issue, one branch, one PR, with the spec, `docs/dev/`, CLAUDE.md, and wiki in the same PR. Size S (a day), M (two to three). Items in a wave touch no common hot file; A owns `content/mod.rs`, C owns `world/schedule.rs` and `travel.rs`'s arrival, D owns `world/quest.rs` and `travel.rs`'s instantiation, M owns `mapgen/mod.rs`'s `add_patrol` and `content/creature.rs`, L owns `world/talk.rs`, P owns `mapgen/special.rs` and `mapgen/shop.rs` (one line each in the shared hot files, by the playbook's rule). The critical path to a playable release is **A → E → F → K** (four PRs, B beside them); the dungeons, the danger system, and the creatures widen it. Seventeen PRs in all.

| # | Issue title | Implements | Depends on | Size | Parallel with |
|---|---|---|---|---|---|
| **A** | Campaigns: `assets/campaigns/<id>/`, `campaign.toml`, quests/dialogue/dungeons as tables, **campaign creature files numbered after the shared block** and the cross-campaign check, global ids with the clash error, one overworld per campaign, `WorldState::campaign`, the fingerprint per campaign, New run's order, the wiki's scenarios and creatures by campaign; the showcase moved into `campaigns/showcase/` (its levels, scenarios, the Depths, the four quests, the hermit's, gardener's, and bard's dialogue **and types**) | §3 | — | M | C, D, M, L, P |
| **B** | The `showcase` feature: `build.rs` gating, the three `Cargo.toml`s, `--no-default-features` in the release workflow and `cargo xtask web`, the CI release-configuration job, `release_content` (no campaign of kind `test` in a release build's content), the "unknown scenario" guard, `web-smoke` on the campaign start | §4 | A | S | E, N1–N3 |
| **C** | Danger levels and spawning: `World::danger` with the `player` scaling table (loaded, fixture-tested, unused by the Ascmearc), the branch and level fields, `LevelState` spawn state, `spawn::tick` (frozen floors), the initial population from `population/<id>` with its counts by floor size and the `bench-turns` entry, unique types, the `spawn` dev action, dumps, the property test, the system tag with Quiet delver and Sixth sense, the wiki's dungeons table | §9, §11.4 | — (a creature's gear at spawn comes by M's `gear::roll`; until M lands, spawns carry nothing, as patrols do today) | M–L (split the initial population and the bench entry into **C2** if it runs over) | A, D, M, L, P |
| **D** | Hidden locations and quest reveals: the legend's `hidden`, `LevelState::hidden`, `WorldState::revealed`, `reveal_location` (marks the map), `QuestDef::reveals` and `where`, the message, the `locations` layer, the `reveal` dev action, the leak test | §8 | — (uses the Wildlands and the hermit's quests for its tests) | S–M | A, C, M, L, P |
| **M** | **Gear on creatures**: `racial_weapon` and `[[gear]]` on the type with the rank check, `gear.rs` (`gear::roll` on the kit's `Pack` and `loot::make`, the burden guard), the streams `gear/<level id>` and `gear/<actor id>`, the hooks in `add_patrol`, legend and vault spawns without fixed kit, and `instantiate_level`, the kobolds' tables, the `actors` dump, the system tag `creature_gear` with Scavenger, the wiki's Gear column, `gear_tests.rs`; the Depths' template snapshot reviewed once; adds no save state and no action | §10.3 | — (C and the population call it once both are in; whichever lands second adds the call) | M | A, C, D, L, P |
| **L** | **Temple services**: `[[dialogue.service]]`, `World::topics` listing them with the price, `ask` performing `cure` (`prayer` in every disease's `cured_by`) and `mend` (`World::mend`), the purse code shared with `pay`, the refusals, the system tag `services` with Pilgrim, the wiki's services table, the core tests with a fixture npc; adds no action and no save state | §6.1 | — | S | A, C, D, M, P |
| **P** | **Per-branch special rooms and shops**: `room_effects` and `shops` on `DungeonDef` and `[[dungeon.levels]]`, read in `special::roll` and `shop::make` with the draw count unchanged, the content checks, the fixture tests, the wiki's dungeons odds | §7.3 | — | S | A, C, D, M, L |
| **E** | The Ascmearc overworld: `campaign.toml`, the 120×60 map with its regions and names, the new terrain, Beorggeat as a closed way back, the start scenario and its arrival line, Cealdhlāw's entrance (visible) and the two hidden entrances | §5 | A, D (for the hidden entries; else they land in H) | M | B, N1–N3 |
| **N1** | The barrow wave: 10 types (the barrow wight in `campaigns/ascmearc/data/creatures/`), the `undead` and `goblins` factions, their gear tables, the wiki rows by themselves | §10.1–10.3 | A, M | S | B, E, N2, N3 |
| **N2** | The fen wave: 12 types (the cultist, the cult priest, and the bog hag campaign-owned; `unique`; `habitat` if kept), the `marsh` and `cult` factions, gear tables | §10.1–10.3 | A, M | S | B, E, N1, N3 |
| **N3** | The delf wave: 12 types (the fire drake campaign-owned), the `duergar` and `fire` factions, gear tables | §10.1–10.3 | A, M | S | B, E, N1, N2 |
| **F** | Cēaptūn: the town level, its four shops, the temple's sanctuary and Sister Hild's two services, the inn, the `villager` type, the npcs with one-liners and topics (no quests yet), the `ceaptun` dev scenario | §6 (without §6.2) | E, L (the services' entries; else they land in H) | M | G, I, J |
| **G** | Cealdhlāw: the branch (danger 1–6, its population from N1), the barrow chamber vault, the `cealdhlaw` dev scenario, its first level's snapshot | §7 | C, E, N1, P | S–M | F, I, J |
| **I** | Fennmynster: the branch (danger 4–11, the raised flooded-room odds, no shops), the sunken crypt vault, the `fennmynster` dev scenario | §7 | C, E, N2, P | S–M | F, G, J |
| **J** | Fyrdelf: the branch (danger 8–19, the duergar traders), the cabin and hoard vaults, per-depth vaults, the `fyrdelf` dev scenario | §7 | C, E, N3, P | S–M | F, G, I |
| **H** | The town's quests: the three chains, the two reveals, the silver chalice, the minster reliquary, and the ledger objects, the branch `arrival` lines | §6.2 | D, F, G, I, J (the targets; a chain may land before its deep targets with the last quest held back) | S–M | K |
| **K** | Release start: New run skips the scenario step in a release, the Characters screen's campaign, `spec/03b-campaign.md`, the wiki's world page with the names and the map, CLAUDE.md's commands | §3.5, §11 | B, E, F | S | H |

**Waves**: **1** A, C, D, and beside them M, L, P (A, C, D each add save state: that is the integrator's three; M, L, and P add no action and no save state, so they sit outside the limit and touch no file of the three but one line each) · **2** B, E, N1, N2, N3 (the waves are data files and tests) · **3** F, G, I, J · **4** H, K. The ending is deferred (§1) and no PR here depends on it.

## 14. The owner's decisions (2026-10-25): where each answer went

Every question of the first revision is answered; nothing stays open. The owner's answers are recorded as decisions in the sections they belong to:

| # | Question | Decision | Where |
|---|---|---|---|
| 1 | campaign-owned or shared creatures | shared, with a campaign's own semi-unique types layered on top | §3.1, **§3.6** |
| 2 | the `showcase` feature's default | option C (on by default, off in release and web); test content never in a release, checked by CI and `release_content` | §4.1 |
| 3 | ids | global string slugs, unique across campaigns; a clash is a content error; a prefix in the slug if namespacing is ever wanted | §3.3 |
| 4 | the overworld's size | 120×60 | §5.2 |
| 5 | the pass | a closed way back, ADOM style for now | §5.1, §2 |
| 6 | names and tone | plain English descriptions put into Old English, in spellings the font draws; the campaign is the Ascmearc | **§5.8**, every name in the document |
| 7 | a reveal marks or tells | marks the map | §8.1, §8.2, §11.1 |
| 8 | the temple as a service | yes: a cure and a mending, for gold, as dialogue topics | **§6.1**, §13 L |
| 9 | a second key to a hidden dungeon | none: quests only | §6, §8.2 |
| 10 | per-branch room effects and shops | yes: `room_effects` and `shops` on `DungeonDef` | **§7.3**, §13 P |
| 11 | frozen or catch-up floors | frozen | §9.5 |
| 12 | hidden locations in the wiki | the wiki tells the truth, hidden locations included; no spoiler hiding now | §11.4 |
| 13 | a player-scaled area near the town | none; the mechanism stays in the data model, unused | §9.1 |
| 14 | the creature list | all 32, every wave scheduled; and gear on humanoids, designed | **§10.1**, **§10.3**, §13 M, N1–N3 |
| 15 | the arc and ending | deferred: nothing designed | §1 |
| 16 | spawning on the showcase's hand-made levels | none (`danger = 0`) | §9.5 |

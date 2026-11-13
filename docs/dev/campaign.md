# Campaigns

Developer notes for campaigns, the seam between worlds (#325, design `docs/design/campaign.md`
§3). The rules are in spec/03b-campaign.md (the Ascmearc, New run, the Characters screen),
spec/03-showcase-world.md ("The showcase is a test campaign"), and
spec/07f-scenarios-and-saves.md. Keep this current with the code, as CLAUDE.md says.

## What is a campaign's

A campaign is a directory `assets/campaigns/<id>/` holding one world: `campaign.toml`,
`levels/` (its overworld and every hand-made level), `scenarios/`, optional `vaults/`, and the
tables `data/dungeons/`, `data/quests/`, `data/dialogue/`, `data/creatures/` (its own,
semi-unique creature types: its people and bosses), `data/uniques/` (its unique items,
`docs/dev/magic-items.md`; the wiki leaves out the test world's), and `data/callings/` (its
doors into the shared prestige classes of `data/prestige/`, `docs/dev/experience.md` "Prestige
classes": `CallingDef::campaign`; a calling outside every campaign is a fixture's and any run's,
`Content::calling(campaign, class)`). Everything else under `assets/` is shared
engine data, `assets/data/` (the common creature types, the shopkeeper's dialogue in
`data/dialogue.toml`, the field references `data/quests.toml` and `data/dungeons.toml`, which
hold no entries) and `assets/vaults/`. There are two campaigns: the Ascmearc (`ascmearc`, kind
`campaign`, #332), the first world to play, so far its overworld and its start scenario
(`docs/dev/overworld.md`) and its market town Cēaptūn (#336, below); and the showcase (`kind = "test"`): the test world, with all nine
scenarios, the hub and its levels, the Wildlands, the Depths, the four quests, and the hermit,
gardener, and bard (dialogue and types). With a campaign of kind `campaign` loaded, release
builds' New run lists only its start, and the player wiki describes only it.

`campaign.toml` (`content/campaign.rs::CampaignDef`): `id` (the directory's name, else a
content error), `name` (New run's heading), `description`, `start` (the scenario a new run
starts, one of the campaign's own), and `kind`, `campaign` (default) or `test`.

## Loading (`content/campaign.rs`, `content/mod.rs`)

- `build.rs` embeds and `ContentFiles::from_dir` reads `assets/campaigns/` with the rest.
- `ContentFiles::campaign_order`: the campaign directories, those of kind `campaign` in path
  order, then the test ones. Every table and directory reads in this order.
- `ContentFiles::table(name)`: `data/<name>.toml`, `data/<name>/**`, then each campaign's
  `data/<name>/**`. `quests`, `dialogue`, and `dungeons` are tables now (`quest::parse`,
  `dialogue::parse`, `Content::load_dungeons`); none has an order that means anything but
  quests' `requires`, which names a quest earlier in the table. `background` too: a campaign's
  chapters (`data/background/`, the Ascmearc's `road`) come after the shared ones in its runs'
  stories, never with a shared chapter's id (docs/dev/background.md). The Ascmearc's `road`
  (seven lines) is the only chapter that names the march's places (Old English names stay out
  of the shared chapters); the showcase's two-line `road` is the tests' (never in a release, and
  never in the wiki, whose tables list the Ascmearc's).
- `ContentFiles::in_dir("levels" | "scenarios" | "vaults")`: the top-level directory, then
  each campaign's.
- Each entry remembers its campaign (`campaign::of_path`): `CreatureDef::campaign`,
  `DungeonDef::campaign`, `QuestDef::campaign`, `Dialogue::campaign`, `Vault::campaign`,
  `ScenarioDef::campaign`, and for hand-made levels `Content::level_campaigns` (read through
  `Content::level_campaign`, which also answers for a branch and its levels).
  `LevelTemplate` carries none: it is also what the generators make.
- Ids are global slugs, unique across every campaign: a level, scenario, vault, dungeon,
  quest, dialogue speaker, or creature type defined twice is a `ContentError` naming both
  files (`content::duplicate_id`). Namespacing, if ever wanted, is a prefix in the slug.
- Creature types: the shared block first, then each campaign's in `campaign_order`, so a
  build without the test world (the `showcase` feature, #331) numbers a real campaign's types
  exactly as a build with it does. Corpse objects follow the creature order.

## The checks (`Content::check_campaigns`, `Content::validate`)

- A campaign's file names its own campaign's or shared levels and creature types, never
  another campaign's: stairs `to`, a scenario's `start_level`, a quest's `level`, `target`
  (reach), kill target (a creature type; a tag must be carried by a shared type or one of the
  campaign's, checked as the quest is read), giver and recipient, a level or vault legend's `creature`, a branch's
  `creatures` and `population` (and a level's `population`), a vault's `dungeons`, and a dialogue's `creature` ("`campaigns/ashland/levels/town.toml`: names level
  `hub`, a level of campaign `showcase`").
- Shared data names shared creature types only (`data/shops.toml`'s `[keeper]`, the shared
  `vaults/`). Any creature, shared or a campaign's, may carry any tag; a tag quest's text
  lists only the types a run of the quest's campaign can meet (`Content::tagged_in`).
- A campaign's `data/plurals.toml` adds words to the one plural dictionary
  (`content/plural.rs`); a word it gives another plural than the shared file is an error.
- Files outside `campaigns/` that campaigns would own (a top-level `levels/`, `scenarios/`, or
  a single `data/quests.toml`) belong to no campaign and are not checked: test fixtures insert
  them on top of the built-in content. A scenario there plays in its start level's campaign.
- At most one level of kind `overworld` a campaign (and one outside every campaign).
- `start` is a scenario of the campaign's own.

## A run's campaign

`WorldState::campaign` (saved and hashed; `#[serde(default)]`, filled from the scenario by
`World::from_state`, and refused there if it is not the scenario's) is the scenario's
campaign, `""` for a fixture outside every campaign. `World::campaign`, `World::fingerprint`.

- **Local areas** belong to the campaign's overworld: `Content::overworld_of(campaign)`,
  `Content::local_area(campaign, id)`, `Content::has_level(campaign, id)`, and
  `mapgen::generate(content, (campaign, seed), id, ..)` take it. `warlock-cli gen` has
  `--campaign` (default `showcase`).
- **The fingerprint** is `Content::fingerprint_for(campaign)`: the shared groups, the levels
  and branches of that campaign (and of none), and the creature types up to the last the run
  may meet (another campaign's types numbered after them are left out with their corpses, the
  objects after every other). A save of a real campaign therefore loads in a developer build
  that also holds the test world. `save::decode` reads the campaign from the saved state (or
  its scenario's) before comparing.

## New run (`crates/warlock/src/ui/new_run_screen.rs::rows_of`)

A release build lists the start scenario of every campaign of kind `campaign` (while there is
none, every campaign's start: the showcase's). A developer build (debug or `--dev`) lists every
scenario of every campaign under its campaign's name, campaigns in content order, the start
first and the rest by id; scenarios outside every campaign last, without a heading.

**One row skips the step** (#341, design §3.5): a release build lists one scenario, the
Ascmearc's start, so `NewRunScreen::skips` (one scenario, and races and classes to create a
character from) is true. `Screens::open_on` then opens the race step over it at once; the New
run screen stays on the stack unseen, so `ScreenStack::draft` reads its pick (the scenario and
the fresh seed) as before, and `ScreenStack::close` closes it too when the step above it closes,
so Esc on the race goes back to the main menu. `--screen race` and the later steps open over it
as usual (`open_startup_screens` skips a screen already on top); `--screen new-run` in a release
build shows the race step. A developer build lists the test world too and shows the screen.
Tests: `new_run_screen.rs` (`skips` in both builds), `screens.rs`
(`a_release_new_run_is_skipped_both_ways`), and `release_content` (one row).

## The Characters screen's campaign (#341, design §3.4)

The save summary (`saves/<id>.toml`, `save::Summary`) keeps `campaign`, the run's campaign's
name (`The Ascmearc`; left out when empty, `#[serde(default)]` for summaries written before).
The Characters screen names it beside the place, `In Cēaptūn (The Ascmearc).`, only when more
than one campaign is installed (a developer build, with the test world): a release build has
one. The fixture summaries under `tests/golden/saved-characters/` (the `characters-screen` text
snapshot) carry `The Showcase`, but Osric's, `The Ascmearc` (made by another version,
docs/dev/saves.md).

## The test world in release builds (#331, design §4)

Test content never appears in a release. The cargo feature `showcase` holds it:

- `warlock-core` has `[features] default = ["showcase"]`; `warlock` and `warlock-cli` have the
  same default and forward it (`showcase = ["warlock-core/showcase"]`); the workspace table
  takes `warlock-core` with `default-features = false`, so each member's own feature decides.
  `xtask` asks for it (`features = ["showcase"]`): the wiki, the goldens, and the benchmarks
  read the test world.
- `build.rs` leaves out every file under `campaigns/showcase/` without the feature
  (`CARGO_FEATURE_SHOWCASE`), so such a build embeds no file of the test world.
  `--content-dir assets` still reads everything on disk: a developer tool.
- Every documented command (`cargo build`, `test`, `run`, the xtask commands but `web`) has the
  feature on and is unchanged. Release artifacts turn it off: the `Release` workflow builds with
  `--no-default-features`, and `cargo xtask web` passes it itself.
- Without a scenario named (`--new`, a bare `--capture`), the game starts
  `Content::default_scenario`: the showcase where it is built in, else the first campaign's
  start. `warlock-cli` keeps its default `showcase` and, in a release build, refuses it as an
  unknown scenario.
- **Guards**: the test `release_content` (compiled only without the feature, in
  `content/campaign.rs` and `ui/new_run_screen.rs`) checks that `Content::builtin()` loads and
  holds no campaign of kind `test` and nothing of one (scenario, level, dungeon, quest,
  dialogue, creature type, vault), and that New run lists exactly the campaigns' starts, also
  with `--dev`. CI's `release-config` job builds the release configuration, checks it for
  wasm32, runs `cargo test --no-default-features -p warlock-core -p warlock release_content`,
  and expects `warlock-cli dump --scenario showcase` to fail with "unknown scenario". The
  `Release` workflow does the last on each platform's binary and checks the game binary holds
  no `campaigns/showcase/campaign.toml` (the key of the embedded file), and `cargo xtask web`
  checks `warlock_bg.wasm` the same way.
- A release build holds one campaign, the Ascmearc (#332): New run goes straight to the race
  ("New run" above). Without any campaign it would say "No scenarios are installed." 

## Cēaptūn, the Ascmearc's market town (#336, design §6)

`campaigns/ascmearc/levels/ceaptun.toml` (90×32, `kind = "outdoor"`, `item_level = 1`, no
danger), entered from the overworld's `T` (`to = "ceaptun"`), arriving on the square (its `@`), and,
with no stairs, left by its edge back to that cell (or `<` when no enemy is about;
docs/dev/overworld.md, the edge exit of a hand-made outdoor level and the quick leave). The map was drawn by a throwaway script and is edited by hand now. Its
`[[shop]]`s (general store, weaponsmith, armourer, provisioner; keepers named in Old English)
lay out `[[pile]]`s of goods each kind buys; the Forge also has an `anvil` (legend `A`, at (35, 5) beside the smith's brazier), the `forge` effect's feature, so Temper is offered beside it and walking into it asks (#975, #1023; free, not the shop's service); the Mynster is a `sanctuary` `[[room]]`. The people
are campaign creature types (`data/creatures/{villager,sister,innkeeper,reeve,mine_factor,guildmaster}.toml`)
and their words are `data/dialogue/ceaptun.toml`: the `villager` type's one-liners and mutters,
and the npcs `hild` (Sister Hild: two topics, the `cure` 60 and `mend` 40 services), `godgifu`
(five rumour topics, each with a `route` from the town, `docs/dev/npcs.md` "Rumoured ways"; never a reveal), `leofric`, `wulfrun`, and `wat` (Wat One-Eye, the
guildmaster, in the Gildhalle of the south-east corner, `[77, 22, 85, 28]`, its door north on the
lane and the bounty board, a named place on the wall beside it: `docs/dev/guild.md`), each with a proper `name`
(`Dialogue::name`: `World::called`, `Content::speaker_name`/`speaker_called`, look mode and the
dialogue screen through `SeenActor::dialogue`). Their quests are below ("Cēaptūn's quests"); each one
done makes their services 5% cheaper (`data/towns/ceaptun.toml`, docs/dev/npcs.md "Goodwill"). The dev
scenario `ceaptun` starts on the square (seed 2, a warrior, 60 gold). Tests:
`world/ceaptun_tests.rs` (the way in and out, the shops' goods, everyone at peace, names, the
sanctuary and the services, Godgifu's topics, the view snapshot `ceaptun_start`, a replay),
`ui/look.rs` (Sister Hild's look line), the `dialogue-services` and `dialogue-guild` text
snapshots.

## Cēaptūn's quests (#340; the design's §6.2)

`campaigns/ascmearc/data/quests/{hild,leofric,wulfrun}.toml`, one file a giver, three chains
(three quests each), each quest offered once the one before it is done (so a
giver shows one quest topic at a time): Reeve Leofric's Cealdhlāw chain (`barrow_rats`: kill 5 rats anywhere in Cealdhlāw, 5 skeletal rats for a Beast-friend, a `fallback` variant;
`barrow_heart`: reach `cealdhlaw:4`; `wight`: kill the barrow wight), Sister Hild's Fennmynster
chain (`grave_goods`: bring the `silver_chalice`; `fen_minster`: reach `fennmynster:3`, which
reveals `fennmynster` as it is given; `water_keeps`: bring the `minster_reliquary`), and
Factor Wulfrun's Fyrdelf chain (`lost_shaft`: reach `fyrdelf:2`, which `requires` `barrow_heart`
(Fyrdelf starts at danger 5, so nobody is sent there before danger 4) and has `min_level = 5`
(asked sooner, Wulfrun says her `early` line and gives nothing) and reveals `fyrdelf` as it is given;
`fire_deep`: kill the fire drake, a named monster: its death counts however it came; then
#1435's two deliveries, `gnome_letter`, the shared `sealed_letter` to Headman Peran of Gimmwīc
(`to = "peran"`, paid there with a cut sapphire), and `gnome_reply`, given by Peran once that is
done, the moot's answer carried back to Wulfrun; both in `wulfrun.toml`, since `requires` names
only quests loaded before, and neither reveals anything: Dēopgang is not hidden and no quest
gates it, the owner's rule). Each reach and bring quest has a `where` for the log. The
quest objects are shared objects (`data/objects/{silver_chalice,minster_reliquary}.toml`,
worth 0: never traded). The chalice lies on the barrow chamber's plinth, which Cealdhlāw names
for its second floor (`[[dungeon.levels]] vaults`: certain, once, and never drawn at random); the
reliquary on the sunken crypt's altar, Fennmynster 8; the drake in its hoard, Fyrdelf 12. The
three branches' first levels say their `arrival` lines (`DungeonDef::arrival`, read through
`Content::arrival` in `travel` with the hand-made levels'). Tests: `world/ceaptun_quest_tests.rs`
(one quest a giver at a time, the lost shaft's prerequisite, its level-5 refusal, and its reveal and log, the fen minster's reveal, kills on any Cealdhlāw level, the arrival lines, a
replay), `mapgen/cealdhlaw_tests.rs` (the chamber and its chalice on floor 2 for certain),
`mapgen/fennmynster_tests.rs` (the reliquary on the crypt's altar).

## Holtwīc, the village at the north road's end (#1140)

`campaigns/ascmearc/levels/holtwic.toml` (72×26, `kind = "outdoor"`, `item_level = 1`, no
danger, no shop), entered from the overworld's `V` (18, 17), the cell north of the north road's
last road cell (18, 18), arriving on the lane from the south (its `@`, (30, 24)), and, with no
stairs, left by its edge back to that cell like Cēaptūn. Cottages round a green with a well,
hedged fields, the wood (trees, pines, forest floor) on the east; a named place, the hoar-stone
(54, 4). The people are campaign creature types `cottar` (five, on `route`s) and `wise_woman`
(`data/creatures/{cottar,wise_woman}.toml`); their words are `data/dialogue/holtwic.toml`: the
`cottar` type's one-liners and mutters, and the npc `wynflaed` (Gammer Wynflæd, at (54, 13) in
her cottage at the wood's edge: topics `holtwic`, `wood`, `fair_folk`; no service, no quest, no
`route`, no reveal). The dev scenario `holtwic` starts on the lane (seed 3, a warrior).

**Ælfhām** (`aelfham`, `data/dungeons/aelfham.toml`, `docs/dev/mapgen.md`): the Unseelie court's
hall, three cave floors at danger 4 to 6, fey only, its way in the hidden `E` at (25, 13) (forest →
`barrow`, `to = "aelfham:1"`), seven cells from the village. Its third floor stamps the pinned
vault `moss_bower` (`campaigns/ascmearc/vaults/moss_bower.toml`), whose named place **"the moss
bed"** (the vault's `M`, `LevelTemplate::places` of `aelfham:3` once generated) is where the
missing villager stands: a vault cannot place a unique npc (`Vault::new`), so the quest (#1141)
places them there. Nothing reveals the mound yet but the dev `reveal:aelfham`. Its dev scenario
is `aelfham`.

For the quest (#1141): giver npc `wynflaed` (level `holtwic`, (54, 13)), hidden location id
`aelfham` (a quest's `reveals`), the bottom level `aelfham:3`, the place name `the moss bed`.
Tests: `world/holtwic_tests.rs` (the way in from Cēaptūn by road and out by the edge, everyone at
peace, the wise-woman's name and topics with no reveal, the hidden mound until `reveal:aelfham`
and the way down, the three floors and the moss bed on the third, the view snapshot
`holtwic_start`, a replay).

## Līctūn, Cēaptūn's churchyard (#1258)

`campaigns/ascmearc/levels/lictun.toml` (46×25, `kind = "outdoor"`, `item_level = 1`, no
danger, no shop), entered from the overworld's `g` (19, 32), beside the south road four cells
out of Cēaptūn (overworld terrain `graveyard`, `†`), arriving on the road outside the gate (its
`@`, (2, 12)), and, with no stairs, left by its edge back to that cell. A `drystone_wall` (new,
shared: stone, seen past, not walked through) with a `gate_open` lych-gate on the road; fifty
`gravestone`s (new, shared: `grave = true`, `∩` with a cross now and then by `variants`) in four
rows of twelve and two beside the chapel; yews (`tree`) in the corners; a little stone chapel
in the middle (a door between two sconces, stools for pews, the altar between two lamps). The
level's `[graves]` table gives the stones their words: given names and bynames of Cēaptūn's
folk (Old and Middle English), seven epitaphs with `{name}`, `{born}`, `{died}`, the `fallen`
words (`Here lies {name}, {death}.`), `died = [1, 150]`, `age = [1, 80]` (`world/graves.rs`,
spec §7 "Graves"). The keeper is the campaign creature type `sexton`
(`data/creatures/sexton.toml`), npc `dudda`, **Old Dudda**, on a `route` round the chapel
(`E30 S11 W30 N11`); his words are `data/dialogue/lictun.toml`: greetings, nine one-liners and
five mutters ("grumbles") in the commoner register, no topic, no service, no quest. No dev
scenario: tests and captures walk from the Ascmearc's start (`move:E*6 move:S*4 move:E
descend`). Tests: `world/lictun_tests.rs` (the way in and out, the sexton alone and at peace,
his line, fifty stones each cut, one in ten for the graveyard and never twice, a stranger's
stone the same under any graveyard, the hash, a graveyard given late, a save, the view
snapshot `lictun_arrival`, a replay verified without the graveyard), `ui/look.rs`
(`a_gravestone_reads_its_inscription`).

## The wiki (`xtask/src/wiki/order.rs::campaigns`)

The wiki describes the campaigns of kind `campaign` (while there is none, every campaign): the
scenarios table is by campaign, the creatures table lists the shared types and then each
campaign's own under its name, and every creature and dialogue table leaves out the types of
campaigns it does not describe (`order::described`).

## Tests

`content/campaign.rs` (a fixture second campaign, the Ashland, beside the showcase: the
numbering, a run of each, the fingerprint and a save across a build without the test world, the
cross-campaign refusals, the id clashes, one overworld a campaign), `new_run_screen.rs` (the
order in release and developer builds), the `new-run-screen` text snapshot, and
`release_content` in both (compiled only in the release configuration).

Vaults (#337): `Vault::stamps_on(def, depth)` keeps a campaign's vaults to its own branches, and
to the branches or branch levels its file's `dungeons` names (`vault::parse_dungeons`, checked by
`Vault::check_dungeons` once the branches are loaded: they must exist and be the vault's own
campaign's; a level file giving `dungeons` is refused). `generate_for` puts the level's list on
the `Canvas` (`Canvas::vaults`), which `vault::shuffled` shuffles, so a branch that no campaign
vault names draws exactly what it did. Tests: `mapgen/cealdhlaw_tests.rs`.

A branch level's `vaults` (#339, docs/dev/mapgen.md) keeps the vaults it names to itself
(`Vault::pinned`, left out of `vault::shuffled`): those may be shared or of the branch's campaign,
never another's (Fyrdelf's cabin and hoard, Fennmynster's sunken crypt, #338; the four lairs'
still-room, robbers' hoard, unhallowed chapel, and captain's hall, #426, `docs/dev/guild.md`).

Hellegeat (`hellegeat`, #1296, `docs/dev/mapgen.md` "The Pits") is the one shared branch:
`data/dungeons/hellegeat.toml`, outside every campaign (`DungeonDef::campaign` `None`), its
population shared fiends, so the cross-campaign checks have nothing to refuse and every
campaign's runs may open its Pits. No campaign's level or quest may name it or its floors. Its
population is in no other area's lists (`Content::fill_generic`), so a campaign's
`stray_pool = "populations"` branches do not gain its fiends.

Grundlēas (`grundleas`, #1010, `docs/dev/mapgen.md`, `world/endless.rs`) is the Ascmearc's
endless branch: an ordinary location (`9` at (52, 52) on the overworld, no `hidden`), 40 cells from
Beorggeat, so no reveal is needed and no one starts beside it. No quest names it: a reach quest
naming an endless branch's level is refused at load, and `Pick::Creatures` lists leave it out. Its
dev scenario is `grundleas`.

Ūtgang (`utgang`, #1117, `docs/dev/mapgen.md` "Ways up", `docs/dev/overworld.md`) is the
Ascmearc's way up: three rooms floors, `below = "fyrdelf:12"`, so its bottom's `>` leads to
Fyrdelf's bottom (which gains a `<` near the drake's hoard) and its top's `<` climbs out to its
hidden way in (`0` at (103, 17)), revealed when the player first climbs out. No quest reveals or
names it; it opens to rolled quests once revealed (`World::branch_open`). Its dev scenario is
`utgang`.

Dēopgang (`deopgang`, #1434, `docs/dev/mapgen.md` "A bottom", `docs/dev/creatures/deep-gnomes.md`)
is the Ascmearc's deepest tier: eleven floors from danger 14 to 24 (four rooms floors of the
gnomes' cut galleries with deep-gnome traders' shops, then seven of caves), ores silver and gold
only (so a miner's finds that are not stone are more often gems), more chests than the shared
odds, and `bottom = "gimmwic"`: its eleventh floor's `>` leads down to Gimmwīc, the deep gnomes'
town, a hand-made level with no danger. Its way in is an ordinary location (`D` at (18, 57), the
south-west corner against Grægweall, no `hidden`): the owner chose no quest gate. The town has
Headman Peran (`peran`, its leader, for the letter quest of #1435), the Glimmering (a sanctuary;
Mother Senara: cure and mend), Kerensa's Stones (`gem_dealer`; identify), Jory's Outfitters
(`outfitter`), and the cutting floor (three anvils; Old Tregear teaches Jewelcraft). Its dev
scenarios are `deopgang` (its first floor) and `gimmwic` (the town).

The dev scenario `drake` (#1099, `docs/dev/creatures.md` "Dragons") starts on `drakes_lair`, a
hand-made Ascmearc level with no stairs: the fire drake in a lava hall like its hoard's, for
replays and captures of a dragon's fight.

Not yet (the design's later items): `dev = true` scenarios (not needed while a release build lists
only the campaigns' starts: the dev scenarios `ceaptun`, `fyrdelf`, ... are never listed there).

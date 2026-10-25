# Campaigns

Developer notes for campaigns, the seam between worlds (#325, design `docs/design/campaign.md`
§3). The rules are in spec/03-showcase-world.md ("The showcase is a test campaign") and
spec/07f-scenarios-and-saves.md. Keep this current with the code, as CLAUDE.md says.

## What is a campaign's

A campaign is a directory `assets/campaigns/<id>/` holding one world: `campaign.toml`,
`levels/` (its overworld and every hand-made level), `scenarios/`, optional `vaults/`, and the
tables `data/dungeons/`, `data/quests/`, `data/dialogue/`, and `data/creatures/` (its own,
semi-unique creature types: its people and bosses). Everything else under `assets/` is shared
engine data, `assets/data/` (the common creature types, the shopkeeper's dialogue in
`data/dialogue.toml`, the field references `data/quests.toml` and `data/dungeons.toml`, which
hold no entries) and `assets/vaults/`. There are two campaigns: the Ascmearc (`ascmearc`, kind
`campaign`, #332), the first world to play, so far its overworld and its start scenario
(`docs/dev/overworld.md`); and the showcase (`kind = "test"`): the test world, with all nine
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
  quests' `requires`, which names a quest earlier in the table.
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
  (reach), kill target, giver and recipient, a level or vault legend's `creature`, a branch's
  `creatures`, and a dialogue's `creature` ("`campaigns/ashland/levels/town.toml`: names level
  `hub`, a level of campaign `showcase`").
- Shared data names shared creature types only (`data/shops.toml`'s `[keeper]`, the shared
  `vaults/`).
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
first and the rest by id; scenarios outside every campaign last, without a heading. Skipping the
step when there is one row is #341's.

## The wiki (`xtask/src/wiki/order.rs::campaigns`)

The wiki describes the campaigns of kind `campaign` (while there is none, every campaign): the
scenarios table is by campaign, the creatures table lists the shared types and then each
campaign's own under its name, and every creature and dialogue table leaves out the types of
campaigns it does not describe (`order::described`).

## Tests

`content/campaign.rs` (a fixture second campaign, the Ashland, beside the showcase: the
numbering, a run of each, the fingerprint and a save across a build without the test world, the
cross-campaign refusals, the id clashes, one overworld a campaign), `new_run_screen.rs` (the
order in release and developer builds), and the `new-run-screen` text snapshot.

Not yet (the design's later items): the `showcase` cargo feature and `release_content` (#331),
campaign vaults kept to their campaign's branches (the vault picker reads every vault), the
save summary's and the Characters screen's campaign (#341), and `dev = true` scenarios.

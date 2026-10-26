# Content files and levels

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/07a-levels.md and spec/07b-data.md. Keep it current with the code, as CLAUDE.md says.

Content (`assets/campaigns`, `assets/data`, `assets/vaults`, and for test fixtures
`levels/` and `scenarios/` at the top) is TOML, embedded into `warlock-core` at build time by
`build.rs` (the test world, `campaigns/showcase/`, only with the `showcase` feature: never in a
release; `docs/dev/campaign.md`). A world's levels, scenarios, branches, quests, dialogue, and own creature types live
in its campaign, `assets/campaigns/<id>/` (`docs/dev/campaign.md`). Level maps are ASCII with a
legend; see `assets/campaigns/showcase/levels/hub.toml` and spec §7. A legend entry with a `creature` may set `faction = "peaceful"` (any id in
`data/factions.toml`) to override its type's faction for that spawn (`Actor::faction_override`; read it with
`Actor::faction`, never the creature type's, when computing stance), and `alignment = "good"`
to override its type's alignment the same way (`Actor::alignment_override`, read with
`Actor::nature`; `docs/dev/alignment.md`). The showcase (hub, vast, courtyard) is safe
this way: `xtask/src/vast.rs` emits it for its rats and kobolds; fighting is the arena's and the Depths'. A level
file's optional `arrival = "..."` line is said when the player arrives by stairs (a dungeon
branch's on its first level, `Content::arrival`; a scenario's `arrival` once, as a run of it
starts). A legend entry's `name` (and `note`) names the place on
its cell (`docs/dev/overworld.md`, "Named places"). `secret =
true` (or a difficulty) on a closed door makes it a secret door: the cell shows the wall around
it until found (`docs/dev/searching.md`). Water: `~` is still water; running water (terrain with a
`flow` anim) needs a flow on every cell from its legend entry (`flow = "S"`, optional
`flow_speed` percent), and the shared legend draws it as arrows (`→` normal, `⇒` double speed).
Flow is presentation data carried by `LevelTemplate::flow` / `LevelState::flow` and read by the
renderer through `PerceptionView::flow` (visible cells only); no rule reads it.

## Content directories

The tables that grow entry by entry are directories, so two PRs adding an entity never touch
the same file: `assets/data/creatures/`, `objects/` (base items included), `materials/`,
`affixes/`, `talents/`, `classes/`, and `uniques/` (each campaign's too), **one file per entity, named by its id** (classes as
`NN-id.toml`: their order is the order New run offers them, so the number keeps it) (`creatures/kobold.toml`,
`affixes/of_the_bear.toml`). A family file holds several entries only for a tight set of
variants always edited together, named after the family (`objects/arrows.toml`: the arrow, the
flaming arrow, and the burning arrow it flies as). Each directory's `README.toml` is comments
only (the field documentation and notes for authors; a file of comments defines nothing, the
loaders take a missing array as empty). `build.rs` embeds and `ContentFiles::from_dir` reads
`assets/{campaigns,data,levels,scenarios,vaults}` recursively, both in sorted path order, and a
table loader reads `ContentFiles::table(name)`: `data/<name>.toml` first, if there is one (a
whole table in one file is still accepted, so test fixtures that insert it keep working), then
`data/<name>/**/*.toml` in path order, then each campaign's `data/<name>/**/*.toml`
(`ContentFiles::campaign_order`: playable campaigns, then test ones). `quests`, `dialogue`, and
`dungeons` are tables too: their single files hold the field reference and the shared entries
(the shopkeeper's dialogue), the campaigns their own. Ids (`CreatureId`, `ObjectId`, `MaterialId`, an affix's
index) are indices in that order and feed the content fingerprint, so moving entries between
files changes ids and invalidates saves (the release bump covers it). Each file is parsed on its
own, so an error names the file the bad entry is in; **a duplicate id across files is a
`ContentError` naming both** (`content::duplicate_id`). For affixes the scalar tables (`[rarity]`,
`[magic]`, `[rare]`, `[names]`) stay in the settings file `data/affixes.toml`, each set in
exactly one file of the table (a second file setting one is an error naming both), and the
entry files hold `[[prefix]]`/`[[suffix]]` only. `ContentFiles::remove("data/creatures/")` drops
a directory; `ContentFiles::edit_table(name, edit)` rewrites every file of a table (tests that
change one built-in line). Every other `data/*.toml` file is a single file whose order means
something (slots, stats, weapon skills, factions, actions, ...): never split those.

**Nothing visible follows the load order of these four tables.** Wiki tables sort by an
explicit key (`xtask/src/wiki/order.rs`: objects by class, weapon skill, value, name; creatures,
materials, and affixes by name), the shop stock a generated shop draws from
(`mapgen/shop.rs`) and the bases and affixes loot rolls from (`magic/loot.rs`) are sorted by id
before the draw, and "shoots arrows" names `Content::ammunition(kind)`, the first ammunition of
the kind by id. A new list over one of these tables gets a sort key too.

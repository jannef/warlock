# The overworld and local areas

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/07c-generation.md. Keep it current with the code, as CLAUDE.md says.

A campaign's overworld (spec §3, §5, §7; the showcase's is
`assets/campaigns/showcase/levels/overworld.toml`) is its one level of kind `overworld`
(`Content::overworld_of(campaign)`; a run's local areas are its campaign's, `docs/dev/campaign.md`) (`LevelKind::Overworld`: a step's base time is the terrain's `travel_minutes`; every
other action, waiting and `>` included, the usual 6 s).
Its locations are stairs-down terrain (`hall`, `cave_mouth`, `dungeon_entrance`) whose legend `to`
names the level; the level's `<` with `to = "overworld"` leads back to the location (`World::travel`
arrives on the stairs leading back). `>` on any other cell enters the local area `local:x,y`
(`content/overworld.rs`: `local_area_id`, `parse_local_area`, `Content::local_area(campaign, id)`, valid where
the terrain has a `local` ground and a land or road `landscape`), generated on the first visit by
the outdoor generator (`mapgen/local/`, below); the edge exit (`exit` in `LevelState`) makes
walking off any edge return to the cell. `>` arrives on the side the player walked onto the cell
from: `WorldState::overworld_step` (the last overworld step, cleared by any level change) picks
the side and `mapgen::local::arrival` the edge cell (road first, then dry, open ground near the
middle of the side); without a step, at the level's start (the middle).
A hand-made `outdoor` level an overworld location leads into (stairs down or a hidden location's
`to`, first in the overworld's order) gets the location as its `exit` when content loads
(`content/overworld.rs::link_edge_exits`, called once after the levels are parsed; no level-file
field), so it is left by its edge like a local area (Cēaptūn has no stairs). The quick leave
(#474, spec §7 "Leaving an outdoor area"; `world/quick_leave.rs`): `Action::Ascend` with no
stairs up underfoot on an `Outdoor` level with an `exit` goes through `leave_by_edge` (priced as
`Stairs` by the dispatch) unless `enemies_about`: an actor on the level with
`Stance::Hostile` toward the player that `Perception::sees_actors_at`, or any within
`QUICK_LEAVE_REACH` (10, Chebyshev) unseen; ground truth, read as a rule. Refused, it says
`QUICK_LEAVE_REFUSED` and spends nothing. `take_stairs` calls it in its "no way" branch, so stairs
underfoot win and dungeons and the overworld are untouched. Tests: `world/quick_leave_tests.rs`.
Sight on the overworld is line
of sight within `daylight::overworld_sight` (day 9, twilight 5, night 2 plus one per
`MOONLIGHT_PER_CELL` of `Sky::moonlight`, at most `SIGHT_MOONLIT` 4), ignoring light. The
overworld is sky-exposed (sun, moons, and sky, like outdoor levels); `daylight::overworld_night`
tops its ambient light up to the faint `OVERWORLD_NIGHT` where the sun's sky falls short
(moonlight comes on top), so cells in sight stand apart from memory at night (local areas get
none). Regions (`[[region]]` in level
files, `LevelTemplate::region_at`, `PerceptionView::region`) show in the status bar. Unlike
`vast.toml`, the overworld map is hand-authored: edit the file directly (keep every walkable cell
inside a region; a test checks).

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

Hidden locations (#327; spec §7 "Hidden locations"; the campaign design §8): a legend's `hidden =
{ id, terrain, to, name }` on a plain overworld cell (`content/level.rs::parse_hidden`; `to` is
checked with the stairs' targets in `Content::validate`) goes into `LevelTemplate::hidden` and
`LevelState::hidden` (by cell; ground truth). It works like a secret door: the terrain grid holds
the plain terrain, so the view, memory, look mode, local areas (`Content::local_area` reads the
template), and every perception-built dump show it with no filter; only `world/reveal.rs` and the
dev views (`dump.rs::truth_rows`, the F2 overlay in `ui/dev.rs`) read `hidden`.
`World::reveal_location(id)` (the one reveal, called by `world/quest.rs` at a quest's
`reveals.when` (`given` or `done`; a rolled quest's variant reveals its own, #422), by the dev
`quest_roll` choosing a variant, and by the dev action `reveal:ID`) records the id in
`WorldState::revealed` (which also opens the branch behind it to rolled quests' picks,
`World::branch_open`); if the
overworld is instantiated, `reveal_on` takes the entry out of `hidden`, sets the terrain and the
player's memory of the cell (the map is marked wherever they are), and pushes a `StairsLink`
with the location's `name` (`StairsLink::name`, preferred by `PerceptionView::leads_to` and the
"The way into … is here." and "You enter …" messages); otherwise `instantiate_level` applies
every revealed id as the overworld is made (`reveal::apply_revealed`). No stream is drawn.
`PerceptionView::known_locations` (the `locations` dump layer, JSON `known_locations`) lists the
overworld's ways in that the player sees or remembers. Tests: `world/reveal_tests.rs` (a fixture
hiding a barrow in the Wildlands, revealed by the hermit's `depths` quest; the built-in Wildlands
hide nothing).
A hidden location's `to` may be left out: it leads nowhere yet, like stairs without one.

Named places (#332): a legend's `name` (and `note`, a sentence for look mode) on any cell of a
hand-made level goes into `LevelTemplate::places` (`content/level.rs::parse_place`; refused
beside a `hidden`, which would give it away). A way in on that cell carries the name as
`StairsLink::name`, so look mode, "The way into … is here." (`world/travel.rs`), and the
`locations` layer name it even while it leads nowhere yet; `PerceptionView::place` gives the
place on a cell the player sees or remembers, and `ui/look.rs` names it after the terrain and
adds the note.

The Ascmearc (#332; the campaign design §5): `assets/campaigns/ascmearc/levels/ascmearc.toml`,
the first real campaign's overworld, 120×60, drawn from the design's 1:2 sketch by a throwaway
script and then edited by hand (no generator: edit the file). Its legend's `P` is Beorggeat, the
start (`mountain_pass`: no stairs, no local area, mountains on three sides, so the road east is
the only way on); `T` Cēaptūn (`town`) leads into the town's level `ceaptun` (#336, `docs/dev/campaign.md`
"Cēaptūn"), and `1` Cealdhlāw (`barrow`) leads to `cealdhlaw:1` (#337, `docs/dev/mapgen.md`); `2` Fennmynster and `3` Fyrdelf are hidden. The rivers are two
cells wide (watertight to diagonal steps), crossed only at the bridge `b` and the ford `%`.
Regions are rectangles in first-match order (the forests' hearts first). Its scenario `ascmearc`
starts on the pass and says its `arrival` line. Tests: `world/ascmearc_tests.rs` (regions, the
closed way back, the town's distance, names and spacing, the rivers, the travel budget, no leak
of the hidden two, a replay, the legend's `name` and `note` checks). Fyrdelf's hidden entry
leads to `fyrdelf:1` (#339; `mapgen/fyrdelf_tests.rs` reveals it with the dev `reveal:fyrdelf`,
walks there, goes down and back up as a replay). Fennmynster's leads to `fennmynster:1` (#338;
`mapgen/fennmynster_tests.rs` does the same from Micelfenn's cell, (70, 48)). The four lairs of
the wanted (#426, `docs/dev/guild.md`) are hidden on reserved sites, legend entries `4` to `7`:
Wyrtholm (88, 46, swamp → `cave_mouth`), Wulfeshol (18, 49, dense forest → `cave_mouth`),
Bānfeld (42, 31, hills → `barrow`), Brocenburg (66, 12, forest → `dungeon_entrance`), each at
least six cells from every other location and with no road beside it (`ascmearc_tests`); each
is revealed by its own variant of `guild_wanted` (`mapgen/lairs_tests.rs` walks to each and goes
down once `reveal:ID` marks it).

Rumoured ways (spec §8 "Dialogue"; `docs/dev/npcs.md`): a dialogue topic's `route` marks the
overworld's memory as day sight along it would (`world/rumour.rs`), reading only the terrain
grid, so hidden locations stay their plain terrain and are never marked or named; a visible
location (Cealdhlāw) is remembered and then listed by the `locations` layer as when seen.

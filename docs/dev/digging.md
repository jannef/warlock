# Digging

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08d-traps-doors-digging.md. Keep it current with the code, as CLAUDE.md says.

Digging (spec §8 "Digging"; M19, #131) lives in `warlock-core/src/world/dig.rs`. Terrain that
breaks has `hardness` and `becomes` (`TerrainDef::breaks: Option<Breakable>`; the data breaks only
solid terrain, into walkable, transparent, plain ground); `void` (solid rock) has a `face`
(`wall`) it turns into when a cell next to it opens, as mapgen walls open ground. Cells on a
level's edge never break (`LevelState::on_edge`, `breaks_at`). `Action::Dig(Dir)` (`dig:DIR`,
key `u` = `KeyCommand::UseTool`, ADOM's use-tool key, a direction prompt; `a` is the
abilities menu, #156; with a carried light ready in the Tool slot, a lantern, `u` turns it
instead, `Action::UseTool`, `world/lamp.rs`, #643) needs `Actor::dig_tool`; `start_dig` refuses (no time) or returns
`Spent::Dig(at)`, and `dig` runs `keep_at` with a `dig` action per turn
(priced by `price_of`, so burden slows it): `dig_turn` makes `Act::Dig` noise (`DIG_NOISE` 80,
`SoundKind::Digging`) and adds `dig_work(power, strength)` (power × (Strength score + 10), in
`WORK_SCALE` = 20ths of a hardness point; the talents' `dig_percent` (Tunneler, Breaker) plus the
worn `dig_percent` (a Delving affix) plus Mining's `dig` benefit (#1343) more) to `LevelState::dug` (saved, by position); at hardness ×
20 `break_wall` sets the `becomes` terrain through `set_terrain` (light masks), clears the
progress, bares `face`s around, and pushes `Event::WallBroken { pos, cause: Breach::Dug }`.
Explosions call `World::blast(center, radius, blast, events)` (a burst trap with a data `blast`,
after its damage: the fireball's 1000): every breakable wall in the burst
(line of sight from the center, computed before anything breaks) with hardness below the blast
breaks (`Breach::Blast`; `perceive_events` says "The wall collapses!" for those the player sees).
Perception sees and remembers the rubble like any terrain; `PerceptionView::ready_to_dig` and
`is_dug` (a visible wall with progress: look mode's "partly dug") are all presentation gets. The
app bursts the new terrain's `toggle` particles (rubble's dust) on `WallBroken`, like a door's.
Creatures do not dig. Times are in the wiki's generated `digging.md` tables.
A blast also blows apart a shut chest whose band's hardness is below it (`World::blast`;
`docs/dev/chests.md`); digging at a chest is refused, a chest having no `hardness` as terrain.
A statue of the stoned (`petrified`, #1126, `world/statue.rs`) is a breaking wall of hardness 1500
(a quarter hour with a pick-axe; no fireball's blast of 1000 breaks it); whatever changes its
terrain, `set_terrain` calls `World::statue_gone`, which spills what it held onto the rubble.

## Mining

`world/mine.rs` (#1389; tests `world/mine_tests.rs`), the data `data/mining.toml`
(`content/mining.rs`: `MiningRules`, `MiningBand`, `MiningFind`, `Mined`; `Content::mining()`,
loaded after the branches, so their `ores` are checked against its ore rows). `World::dig` calls
`World::mine(at, wall)` once, after its "crumbles" line, when the wall it dug at no longer stands
(the one line in `dig.rs`). `mine` gives nothing off a branch floor (`Content::branch_level`),
for a statue (`TerrainDef::statue`), or to one untrained in the file's `skill` (Mining; no draws
then). Draws, all `state.rng`, in this order: the band's `chance` (`MiningRules::band`: the
hardest band not harder than the wall's `hardness`), the check (`combat::roll_natural`: stat
modifier + `skill_effect_of(Mining, Effect::Craft)` (Mining's new `craft` curve, +8 at 100, +12 at
200) + `TalentMods::check(stat)` against the band's `difficulty`; natural 2 fails, 24 makes), the
row (`MiningRules::at(Content::floor_danger(level, player level), allow-list)`, `pick_weighted`;
a row's weight is `rarity + deeper × (danger − its danger)`, 0 below its danger), and for the gem
row `gem::uncut` (gem by rarity, size by the item level; the roughest cut). Count for ore and
stone: `1 + margin / more_every`, at most `most`. The find is laid on the rubble's pile
(`object::add_to_pile`) and said ("You break out 2 lumps of copper ore."). No mark of its own:
Mining's `dug` mark is the wall's. The ore is `objects/ore.toml` (class `materials`, made of any
metal, its main part the metal: `mine::ore`; named by the variant rule, "lump of iron ore" →
"lump of copper ore"); `recipes/metalcraft/smelt.toml` (`main = { input = "ore" }`, result the
piece) makes the ingot (`docs/dev/crafting.md`). A branch's allow-list is `DungeonDef::ores`
(metal ids; `[]` none), stone and gems kept. The wiki's `mining.md` tables (`xtask/src/wiki/mining.rs`)
give the bands, the rows with their weights by danger, and each branch's ores.

**Veins** (#1390): `[veins]` (`Veins`: `chance`, `count`, `wall`) and an ore row's `vein = { fg,
size }` (`MiningFind::vein`, `Vein`). The loader makes one terrain per vein,
`add_vein_terrain`: a clone of `wall`'s `TerrainDef` (hardness, glyph, ground, `stone`) with id
`vein_<metal>`, name "vein of <metal> ore", a description, and the row's `fg`, appended to the
terrain list after `terrain.toml`'s (so no hand-made level names one). `MiningRules::vein_of`
maps a terrain back to its metal, `vein_table` is `at` with only the vein rows. In `mine`, a vein
wall skips the band's `chance` draw and the table draw: the check is rolled as for any wall and
gives `count` made, 1 failed (`lay_mined` lays and says it). The trained rule, the branch-floor
rule, and blasts giving nothing are unchanged; `dig.rs` is untouched. Placement is
`mapgen/vein.rs` (`docs/dev/mapgen.md`); the wiki's `mining-veins.md` table lists them, and
`mining-walls.md` leaves veins out.

A greater treasure room (#618) is reached only by breaking a wall: its breach, a found one a
hollow wall that `set_terrain` forgets once broken (`docs/dev/special-rooms.md`).

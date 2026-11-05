# Digging

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08d-traps-doors-digging.md. Keep it current with the code, as CLAUDE.md says.

Digging (spec §8 "Digging"; M19, #131) lives in `warlock-core/src/world/dig.rs`. Terrain that
breaks has `hardness` and `becomes` (`TerrainDef::breaks: Option<Breakable>`, validated: only
solid terrain breaks, into walkable, transparent, plain ground); `void` (solid rock) has a `face`
(`wall`) it turns into when a cell next to it opens, as mapgen walls open ground. Cells on a
level's edge never break (`LevelState::on_edge`, `breaks_at`). `Action::Dig(Dir)` (`dig:DIR`,
key `u` = `KeyCommand::UseTool`, ADOM's use-tool key, a direction prompt; `a` is the
abilities menu, #156; with a carried light ready in the Tool slot, a lantern, `u` turns it
instead, `Action::UseTool`, `world/lamp.rs`, #643) needs `Actor::dig_tool`; `start_dig` refuses (no time) or returns
`Spent::Dig(at)`, and `dig` runs `keep_at` with a `dig` action per turn
(priced by `price_of`, so burden slows it): `dig_turn` makes `Act::Dig` noise (`DIG_NOISE` 80,
`SoundKind::Digging`) and adds `dig_work(power, strength)` (power × (Strength score + 10), in
`WORK_SCALE` = 20ths of a hardness point; the talents' `dig_percent` (Tunneler, Breaker) plus the
worn `dig_percent` (a Delving affix) more) to `LevelState::dug` (saved, by position); at hardness ×
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
A greater treasure room (#618) is reached only by breaking a wall: its breach, a found one a
hollow wall that `set_terrain` forgets once broken (`docs/dev/special-rooms.md`).

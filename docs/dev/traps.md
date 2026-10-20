# Traps

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08d-traps-doors-digging.md. Keep it current with the code, as CLAUDE.md says.

Traps (spec §8 "Traps") are data (`data/traps.toml`, `content/trap.rs`: `TrapDef`, `TrapEffect`
shot / strike / burst / bolt / pit / alarm / teleport, and the instance `Trap` with `known`,
`charges`, `dir`) placed by level legends (`trap = "id"`, `known = true`, `dir`), dungeon vaults,
and branches (`traps`, `traps_per_level` in `data/dungeons.toml`); the showcase has none, the
trap hall (`levels/trap_hall.toml`, scenario `traps`) has one of each. They live in
`LevelState::traps` (and `trap_memory`, what the player remembers). `World::spring_trap(at,
victim)` in `world/trap.rs` is the one way a trap goes off: stepping onto a cell calls it through
`World::step_onto_trap` (the player in `player_moves`, creatures in `ai.rs::creature_steps`, one
who makes way for the player onto their old cell in `talk.rs::swap_places`, its pit's climb taken
off its energy), and returns the extra energy a pit costs. **Trapped doors** (spec §8 "Trapped
doors", #174): a trap may lie on a door, closed or open (`TerrainDef::is_door`,
`LevelState::is_door`), of any kind but a pit (`TrapEffect::fits_door`, checked by
`content/level.rs::parse_trap`, which also refuses a trapped secret door; `mapgen::hide_secrets`
never hides a trapped door). `step_onto_trap` skips door cells, so the step trigger never sets one
off; `world/door.rs` does: `swing_open` (every opening, the player's with or without a key and a
creature's in `creature_steps`) and `kick_door` (as the kick lands, before the door breaks) call
`spring_trap` with the door's cell and the opener or kicker beside it. `set_terrain` drops a trap
whose door stops being a door (kicked in). Aimed from the door: a shot and a strike go at the
victim where it stands, a bolt without a `dir` runs from the door through the victim (`spring_trap`
picks `Dir::toward` when the victim stands next to the trap), and a burst's own cell never stops it
(`trap_burst`), so a shut door's blast leaves both sides. The app's step prompt
(`input::trap_dialog`) skips doors. A trap's blow is dealt like any other
(`World::trap_deals`: blood by `World::spill`, `hurt` with `Harm::Trap`, then the wound's
statuses by `World::wounded`); its kills earn nothing, and the player's death records
`Cause::Trap` (`Death::cause`: "Killed by a fireball trap"). Traps never lie on water that
freezes (terrain with `freeze`: ice would cover them), and a teleport never lands on ice.
Knowledge is decided as the turn is perceived
(`World::learn_traps`: set off by the player, or on a cell they see), or as the player finds
one (searching or in passing, `docs/dev/searching.md`): both go through `World::know_trap`. A
trap's `difficulty` and `sign` are for finding it; messages come from
`perceive_events` (`Event::TrapSprung`, `TrapStruck`). Bursts and bolts emit the full
`Event::Spell { effect, origin, shape: SpellShape, cells }` (`spell::burst_cells`, so what is drawn
is what is hit; a bolt's path starts after the trap's cell; the trap's `spell` must be an effect of
its shape), which `spell::perceive_spell` cuts and `ui/spells.rs` draws; a burst with a `blast`
then calls `World::blast` (walls, `docs/dev/digging.md`), and one with `ice`
(`content::BurstIce`, from `heat = true` or `frost = MM` in data) `World::shock` with
`Shock::Heat` (the fireball melts the ice, and whoever stood on it falls in) or `World::frost`
(the frost trap freezes the water), `docs/dev/temperature.md`. A teleport's destination is an
FNV-1a hash, never the gameplay stream. A rest (and every long action through `World::keep_at`)
perceives every turn in which a trap went off (`World::sprung`), besides those in which the player
heard something.

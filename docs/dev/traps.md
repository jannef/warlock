# Traps

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08d-traps-doors-digging.md. Keep it current with the code, as CLAUDE.md says.

Traps (spec §8 "Traps") are data (`data/traps.toml`, `content/trap.rs`: `TrapDef`, `TrapEffect`
shot / strike / burst / bolt / pit / alarm / teleport, and the instance `Trap` with `known`,
`charges`, `dir`) placed by level legends (`trap = "id"`, `known = true`, `dir`), dungeon vaults,
and branches (`traps`, `traps_per_level` in `data/dungeons.toml`); the showcase has none, the
trap hall (`levels/trap_hall.toml`, scenario `traps`) has one of each. They live in
`LevelState::traps` (and `trap_memory`, what the player remembers). `World::spring_trap(at,
victim)` in `world/trap.rs` is the one way a trap goes off: stepping onto a cell calls it (the
player in `player_moves`, creatures in `ai.rs::creature_steps`; a trapped door will call it too)
and returns the extra energy a pit costs. A trap's blow is dealt like any other
(`World::trap_deals`: blood by `World::spill`, `hurt` with `Harm::Trap`, then the wound's
statuses by `World::wounded`); its kills earn nothing, and the player's death records
`Cause::Trap` (`Death::cause`: "Killed by a fireball trap"). Traps never lie on water that
freezes (terrain with `freeze`: ice would cover them), and a teleport never lands on ice.
Knowledge is decided as the turn is perceived
(`World::learn_traps`: set off by the player, or on a cell they see); messages come from
`perceive_events` (`Event::TrapSprung`, `TrapStruck`). Bursts and bolts emit the full
`Event::Spell { effect, origin, shape: SpellShape, cells }` (`spell::burst_cells`, so what is drawn
is what is hit; a bolt's path starts after the trap's cell; the trap's `spell` must be an effect of
its shape), which `spell::perceive_spell` cuts and `ui/spells.rs` draws; a burst with a `blast`
then calls `World::blast` (walls, `docs/dev/digging.md`). A teleport's destination is an
FNV-1a hash, never the gameplay stream. A rest (and every long action through `World::keep_at`)
perceives every turn in which a trap went off (`World::sprung`), besides those in which the player
heard something.

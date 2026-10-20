# Experience

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08c-combat.md. Keep it current with the code, as CLAUDE.md says.

Experience (spec §8 "Experience", M21): `content/experience.rs` holds the rules from
`data/experience.toml` (level curve, `hp_level`, `mana_level` (mana per level:
`World::level_up_mana`, `docs/dev/mana.md`), the WoW-style `kill_xp` scaling with its zero
difference bands), `world/experience.rs` applies them (`earn_kill` from `fight.rs::dies` when the
player lands the killing blow, or caused the status effect that killed: `Effect::by`; `grant_xp`,
level-up messages queued in `World::news` and said after the attack's own, or at the end of
`perceive` for a creature that bled to death). Creatures carry `level` and `xp` in `creatures.toml`. The saved state is
only `WorldState::experience` (the level follows from it); the UI reads
`PerceptionView::own_experience()`. Other kills (traps, creatures) give nothing: keep it that way
by awarding only where the attacker is the player.

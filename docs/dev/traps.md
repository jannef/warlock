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
off its energy), and returns the extra energy a pit costs. **Sidestepping** (spec §8 "Traps",
#476) is `world/sidestep.rs`: `player_moves` calls `World::step_willingly` instead (not when
`turned_back` changed the cell), which first gives a **known** trap off a door the chance
`Content::trap_avoid()` (`content::TrapAvoid`, `[avoid]` in `data/traps.toml`: `base` +
`per_point` × dodge within `min`..`max`) at `World::trap_dodge` (the Dexterity plus Dodge
categories of `defence_of`, so armour, burden, talents, magic, stance, and statuses all count),
one `rng.chance` draw of the gameplay stream; avoided, the trap is left untouched, the player told,
and `Use::Dodge` reported for marks; else `step_onto_trap` as before (with the Light step talent's
own check, `steps_lightly`). Tests: `world/sidestep_tests.rs`; the wiki's `trap-avoid.md` table
(`xtask/src/wiki/traps.rs::avoiding`) reads `TrapAvoid::chance`. **Trapped doors** (spec §8 "Trapped
doors", #174): a trap may lie on a door, closed or open (`TerrainDef::is_door`,
`LevelState::is_door`), of any kind but a pit (`TrapEffect::fits_fixture`, checked by
`content/level.rs::parse_trap`, which also refuses a trapped secret door; `mapgen::hide_secrets`
never hides a trapped door). `step_onto_trap` skips door cells, so the step trigger never sets one
off; `world/door.rs` does: `swing_open` (every opening, the player's with or without a key and a
creature's in `creature_steps`, each priced before the door swings, since the trap may kill the
opener) and `kick_door` (as the kick lands, before the door breaks) call
`spring_trap` with the door's cell and the opener or kicker beside it. `set_terrain` drops a trap
whose door stops being a door (kicked in). **Trapped chests** (C2) share all of this: the
same list, the shut chest's cell skipped by `step_onto_trap`, a shot out of the chest itself
(`slit`), and a burst blowing the chest apart (`TrapEffect::blows_fixture`); docs/dev/chests.md. Aimed from the door: a shot and a strike go at the
victim where it stands, a bolt without a `dir` runs from the door through the victim (`spring_trap`
picks `Dir::toward` when the victim stands next to the trap), and a burst's own cell never stops it
(`World::cast_burst`), so a shut door's blast leaves both sides. The app's step prompt
(`input::trap_dialog`) skips doors. A trap's blow is dealt like any other
(`World::trap_deals`: the target's Trap sense percent less its worn `trap_damage` (of Wariness),
at most `TRAP_DAMAGE_CUT` = 75 % off, rounded down, at least 1; blood by `World::spill`, `hurt` with `Harm::Trap`, then the wound's
statuses by `World::wounded`), of the trap's `damage_type` (`TrapEffect::damage_kind`;
docs/dev/combat.md "Damage types": a shot's and a strike's `Attack::kinds` go through
`combat::strike`, a burst's, a bolt's, and a pit's `Damage` through `World::trap_hurts` →
`World::resisted`); its kills earn nothing, and the player's death records
`Cause::Trap` (`Death::cause`: "Killed by a fireball trap"). Traps never lie on water that
freezes (terrain with `freeze`: ice would cover them), and a teleport never lands on ice.
Knowledge is decided as the turn is perceived
(`World::learn_traps`: set off by the player, or on a cell they see), or as the player finds
one (searching or in passing, `docs/dev/searching.md`, with the Detect traps skill in place of
Search's and Spot's, #663): both go through `World::know_trap`. A
trap's `difficulty` and `sign` are for finding it; messages come from
`perceive_events` (`Event::TrapSprung`, `TrapStruck`). Bursts and bolts (`TrapEffect::Burst(Burst)`, `Bolt(Bolt)`, the data types scrolls share,
`content/spell.rs`) go through the one shared spell code, `World::cast_burst` and
`World::cast_bolt` in `world/spell.rs`, with `Caster::Trap` (scrolls cast with `Caster::Actor`,
`docs/dev/scrolls.md`); the noise is `World::sound_at`. They emit the full
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

**Disarming** (spec §8 "Disarming traps", #232) is `world/disarm.rs`: `Action::Disarm(dir)`
(`disarm:DIR`, `ActionKind::Disarm`, `disarm = 2000` in `data/actions.toml`; `buff.rs::kind_of`
maps it for statuses that block) → `World::disarm`, one arm in `player_acts`. It refuses (no
time, no draw) a player without the ability (`Ability::Disarm.had_by`: trained in the skill with
`skills::Effect::Disarm`, Disable device) and a cell without a **known** trap (hidden or none:
the same message, so a try leaks nothing). Otherwise it prices the action first (the trap may
kill), rolls `check::roll` (`Checked::Made` / `Failed { by }`, two gameplay draws) with
`disarm_modifier` (effective Dexterity modifier plus `Actor::skill_effect(Effect::Disarm)`)
against `World::difficulty_of(Hidden::Trap(kind))` (search's: the trap's `difficulty` plus the
branch's depth), reports `Use::Disarm` for marks, and then removes the trap and its
`trap_memory` (made), says so (failed by less than `SPRINGS_SHORT_BY`, 5, re-exported from
`world`), or calls `spring_trap(at, player)` and adds the pit's climb to the price (failed by
more). A door's trap is the same cell's trap: nothing about doors is special. The app's
`Ability::Disarm` asks for a direction (`InputMode::Ability`, "Disarm in which direction?") from
the abilities menu or `KeyCommand::Disarm` (unbound). The wiki's `trap-disarm.md` table
(`xtask/src/wiki/traps.rs::disarming`) uses `check::ways` and `check::ways_short_by`. Tests:
`world/disarm_tests.rs` (rigged streams for each outcome, the pit's climb, a door's trap,
refusals, the modifier, marks, the menu, saves and replays), `check.rs`, `ability.rs`.

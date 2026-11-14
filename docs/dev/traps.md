# Traps

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08d-traps-doors-digging.md. Keep it current with the code, as CLAUDE.md says.

Traps (spec §8 "Traps") are data (`data/traps.toml`, `content/trap.rs`: `TrapDef`, `TrapEffect`
shot / strike / burst / bolt / pit / alarm / teleport, and the instance `Trap` with `known`,
`charges`, `dir`) placed by level legends (`trap = "id"`, `known = true`, `dir`), dungeon vaults,
and branches (`traps`, `traps_per_level` in `data/dungeons.toml`); the showcase has none, the
trap hall (`levels/trap_hall.toml`, scenario `traps`) has one of each. They live in
`LevelState::traps` (and `trap_memory`, what the player remembers). `World::spring_trap(at,
victim)` in `world/fixtures/trap.rs` is the one way a trap goes off: stepping onto a cell calls it through
`World::step_onto_trap` (the player in `player_moves`, creatures in `ai.rs::creature_steps`, one
who makes way for the player onto their old cell in `talk.rs::swap_places`, its pit's climb taken
off its energy), and returns the extra energy a pit costs; a hidden trap (any for a creature, an
unknown one for the player) that sprang then `startle`s its victim alone (docs/dev/surprise.md). **Sidestepping** (spec §8 "Traps",
#476) is `world/skills/sidestep.rs`: `player_moves` calls `World::step_willingly` instead (not when
`turned_back` changed the cell), which first gives a **known** trap off a door the chance
`Content::trap_avoid()` (`content::TrapAvoid`, `[avoid]` in `data/traps.toml`: `base` +
`per_point` × dodge within `min`..`max`) at `World::trap_dodge` (the Dexterity plus Dodge
categories of `defence_of`, so armour, burden, talents, magic, stance, and statuses all count),
one `rng.chance` draw of the gameplay stream; avoided, the trap is left untouched, the player told,
and `Use::Dodge` reported for marks; else `step_onto_trap` as before (with the Light step talent's
own check, `steps_lightly`). Tests: `world/skills/sidestep_tests.rs`; the wiki's `trap-avoid.md` table
(`xtask/src/wiki/traps.rs::avoiding`) reads `TrapAvoid::chance`. **Trapped doors** (spec §8 "Trapped
doors", #174): a trap may lie on a door, closed or open (`TerrainDef::is_door`,
`LevelState::is_door`), of any kind but a pit (`TrapEffect::fits_fixture`, checked by
`content/level.rs::parse_trap`, which also refuses a trapped secret door; `mapgen::hide_secrets`
never hides a trapped door). `step_onto_trap` skips door cells, so the step trigger never sets one
off; `world/fixtures/door.rs` does: `swing_open` (every opening, the player's with or without a key and a
creature's in `creature_steps`, each priced before the door swings, since the trap may kill the
opener) and `kick_door` (as the kick lands, before the door breaks) call
`spring_trap` with the door's cell and the opener or kicker beside it. `set_terrain` drops a trap
whose door stops being a door (kicked in). **Trapped chests** (C2) share all of this: the
same list, the shut chest's cell skipped by `step_onto_trap`, a shot out of the chest itself
(`slit`), and a burst blowing the chest apart (`TrapEffect::blows_fixture`); docs/dev/chests.md. Aimed from the door: a shot and a strike go at the
victim where it stands, a bolt without a `dir` runs from the door through the victim (`spring_trap`
picks `Dir::toward` when the victim stands next to the trap), and a burst's own cell never stops it
(`World::cast_burst`), so a shut door's blast leaves both sides. The app's step prompt
(`ui::step_dialogs::trap_dialog`) skips doors. **Damage by danger** (#1165): `TrapDef::grows`
(`content::TrapGrows`, parsed from `[trap.grows]`'s `damage = "danger 3 1d4, ..."` by
`spell_ladder::parse_rungs`, the rung reader `[spell.grows]` shares; refused: no `danger` prefix,
rungs not rising, out of 1 to `MAX_DANGER`, a rung that can roll below 0, or rungs on a trap
without damage). `spring_trap` reads
`TrapDef::effect_at(World::danger(level))`, a clone of the effect with `damage_at`'s roll put
in by `TrapEffect::with_damage`; `TrapEffect::damage()` stays the base (rung 0), which is what
`def.effect` holds. A pit's damage is a `Roll` (rolled with reroll 0, the same draws as its
old `Dice`). Tests: `world/fixtures/trap_rung_tests.rs` (every builtin trap's rung per band; a sprung
trap at danger 0, 2, 3, 10, 30; a door's trap; every race and class at level 1 survives every
damaging trap at danger 0, and no base's worst, a critical doubling a shot's or a strike's,
reaches the fewest hit points among them; malformed rungs refused). The wiki's
`trap-damage.md` (`xtask/src/wiki/traps.rs::by_danger`) is the trap × band table. A trap's blow
is dealt like any other
(`World::trap_deals`: the target's Trap sense percent less its worn `trap_damage` (of Wariness),
at most `[harm] most_cut` = 75 % (`data/traps.toml`) off, rounded down, at least 1; blood by `World::spill`, `hurt` with `Harm::Trap`, then the wound's
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
`World::cast_bolt` in `world/magic/spell.rs`, with `Caster::Trap` (scrolls cast with `Caster::Actor`,
`docs/dev/scrolls.md`); the noise is `World::sound_at`. They emit the full
`Event::Spell { effect, origin, shape: SpellShape, cells }` (`spell::burst_cells`, so what is drawn
is what is hit; a bolt's path starts after the trap's cell; the trap's `spell` must be an effect of
its shape), which `spell::perceive_spell` cuts and `ui/spells.rs` draws; a burst with a `blast`
then calls `World::blast` (walls, `docs/dev/digging.md`), and one with `ice`
(`content::BurstIce`, from `heat = true` or `frost = MM` in data) `World::shock` with
`Shock::Heat` (the fireball melts the ice, and whoever stood on it falls in) or `World::frost`
(the frost trap freezes the water), `docs/dev/temperature.md`. A teleport's destination is an
FNV-1a hash, never the gameplay stream. Where a teleport may land is `World::lands_on` (walkable, no
trap, no ice) with no one there (`World::may_land`), shared with the blink dog's blink
(`world/creatures/blink.rs`, #1130), whose cell beside its target is a gameplay draw. A rest (and every long action through `World::keep_at`)
perceives every turn in which a trap went off (`World::sprung`), besides those in which the player
heard something.

**Disarming** (spec §8 "Disarming traps", #232) is `world/skills/disarm.rs`: `Action::Disarm(dir)`
(`disarm:DIR`, `ActionKind::Disarm`, `disarm = 2000` in `data/actions.toml`; `buff.rs::kind_of`
maps it for statuses that block) → `World::disarm`, one arm in `player_acts`. It refuses (no
time, no draw) a player without the ability (`Ability::Disarm.had_by`: trained in the skill with
`skills::Effect::Disarm`, Disable device) and a cell without a **known** trap (hidden or none:
the same message, so a try leaks nothing). Otherwise it prices the action first (the trap may
kill), rolls `check::roll` (`Checked::Made` / `Failed { by }`, two gameplay draws) with
`disarm_modifier` (effective Dexterity modifier plus `Actor::skill_effect(Effect::Disarm)`)
against `World::difficulty_of(Hidden::Trap(kind))` (search's: the trap's `difficulty` plus the
branch's depth), reports `Use::Disarm` for marks, and then removes the trap and its
`trap_memory` (made), says so (failed by less than `[disarm] springs_short_by`, 5, of
`data/traps.toml`), or calls `spring_trap(at, player)` and adds the pit's climb to the price (failed by
more). A door's trap is the same cell's trap: nothing about doors is special. The app's
`Ability::Disarm` asks for a direction (`InputMode::Ability`, "Disarm in which direction?") from
the abilities menu or `KeyCommand::Disarm` (unbound). The wiki's `trap-disarm.md` table
(`xtask/src/wiki/traps.rs::disarming`) uses `check::ways` and `check::ways_short_by`. Tests:
`world/skills/disarm_tests.rs` (rigged streams for each outcome, the pit's climb, a door's trap,
refusals, the modifier, marks, the menu, saves and replays), `check.rs`, `ability.rs`.

**Rigging** (spec §8 "Traps": "Rigging traps", #1423; the craftsperson design §7.6) is
`world/crafting/rig.rs`: the craftsperson's Trapwright (`data/talents/trapwright.toml`, `effects = [{
kind = "rigs" }]` → `CraftMods::rigs` → `ClassActive::Rig`, id `trapwright`, `Aim::Rig`).
The rigs are `data/rigs.toml` (`content/rig.rs`: `RigRules { difficulty, skill, rigs }`,
`RigDef { trap, seconds, takes: RigTakes::{Ammo, Stock, DigTool}, knife, wall, made, snaps }`,
`RigId` its index; optional, and without it `Ability::unavailable` says `NotYet`; the loader
refuses a trap that is none, not exactly one of `ammo`/`stock`/`dig_tool`, `most` or `charges`
outside 1 to 100 or on the wrong kind, ammunition no object is, a stock group `stock.toml`
lacks, `wall` outside 1 to 10, `seconds` 0). The three traps it places (`rigged_arrows`,
`rigged_stakes`, `rigged_pit`) are the last rows of `data/traps.toml`, so no older trap's
`TrapId` moves; they have no `charges` (the rig gives them) and no `grows`. The app: the
abilities menu opens `ui/rig_screen.rs` (`ScreenName::Rig`, `--screen rig`; rows from
`PerceptionView::rigs`, `RigRow` with what is lacking, a lacking row stays), which acts with
`Action::Rig { rig, dir: None }`; `input.rs::act` turns that into `InputMode::RigWay(RigId)`,
the direction prompt, whose answer sends `rig:ID:DIR`. The rules: `World::start_rig` (refusals,
no time: the ability, the rig, a direction, the overworld, afloat, `World::rig_refusal` on the
cell as the player knows it, the wall in a straight line within `wall` cells, the inputs by
`rig::lack`) → `Spent::Rig(Rigging)` → `World::rig` (`keep_at` in `craft` turns, the last cut
to the rig's time) → `World::rigged` (a hidden trap there is found, someone there blocks, the
inputs found again, `combat::roll_natural` + Dexterity + `skill_effect_of(skill, Craft)` +
`VigourMods::check(Dexterity)` against `difficulty` by `check::makes`; made or a natural 2: the
inputs spent, `Trap { known, charges, rigged: true }` placed and in `trap_memory`; a natural 2
then `spring_trap(at, player)`, a pit's climb as a `run_turn`; then `perceive`). **The kill**:
`Trap::rigged` (saved, skipped while false) makes `spring_trap` set `World::rigger` (not saved)
around `set_off`; `trap_deals` then hurts with `Harm::Actor(player)` and wounds `by` them, so
`dies` credits the player as for their own blow, unless the victim is the player (then
`Harm::Trap`, `Cause::Trap`). Dumps: `truth` adds `rig::rigged_rows`; JSON `known_traps[]`
carries `rigged` (`PerceptionView::rigged`). The wiki's `rigs.md` is
`xtask/src/wiki/traps.rs::rigs`, and `traps.md` says a rigged trap goes off "as rigged". Tests:
`world/crafting/rig_tests.rs`, `content/rig.rs`, the rig list's screen snapshot.

# Doors

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08d-traps-doors-digging.md. Keep it current with the code, as CLAUDE.md says.

Doors (spec §8 "Doors"; M19, #130): the rules are `warlock-core/src/door.rs` (`Lock`, `Key`,
`kick_breaks`/`break_chance`: `1d(sturdiness) ≤ Strength`, one gameplay draw), applied in
`world/door.rs` (`open_door` unlocks with `Actor::key_for` and costs `unlock` + `open_door`,
`swing_open`, `close_door`, `kick`/`kick_door`, the kick's Strength `World::kick_force`
(effective Strength + athletics + talents' `door_kick` + worn `door_kick`, Hobnailed; a chest's
kick reads it too); kicking a creature is `World::player_kicks`, an
unarmed blow whatever is wielded: `Actor::kick` (bare hands with unarmed fighting's benefits and
the stance), training unarmed fighting, priced `kick` by `attack_price` (so the stance and
attack speed change its time), said with `Words::kick`). A kick at a door is no attack
(`price_of`: the plain `kick`); the player's athletics (`skills::Effect::Kick`) adds to the
Strength it is rolled against, and the kick earns athletics marks (`Use::Kick`). Locks are per-cell state, never terrain: `locks` on
`LevelTemplate` and `LevelState` (by position, saved, hashed; ground truth only, so perception,
memory, look mode, and dumps show a locked door as any closed door); `World::set_terrain` drops a
lock whenever the cell stops being a closed door. Secret doors (`docs/dev/searching.md`) are
per-cell state too (`secrets`, beside the wall they look like until found); a secret door is
never locked, nor trapped. Trapped doors (`docs/dev/traps.md`): `swing_open` and `kick_door` set
off a trap on the door at whoever opened or kicked it (`World::spring_trap`); `set_terrain` drops
the trap with a door kicked in. **Picking locks** (#233) is `world/lockpick.rs`
(`World::pick_lock`, dispatched from `player_acts` for `Action::PickLock(dir)`, replay text
`pick:DIR`, cost `pick_lock`): only for an actor trained in a skill with the `pick` effect
(`Skills::trains_effect`: Open lock) or holding a talent with the `pick_lock` effect (none since
#684 took the thief's Pick lock: the thief trains Open lock; `Ability::had_by`), a `crate::check` of the Dexterity modifier plus
`skill_effect(Effect::Pick)` against `World::lock_difficulty` (the lock's own
`Lock::difficulty`, `door::LOCK_DIFFICULTY` 14 unless a legend's `lock_difficulty` gives one,
plus the branch's `DungeonDef::locks_at`, from `locks_per_depth` in `dungeons.toml`); both prices
are taken before the check, `Use::Pick` trains Open lock, and a picked lock is removed and the
door opened by `swing_open` (so a trapped door's trap goes off at the picker); a failure changes
nothing, so trying again is free but for time. The ability is `Ability::PickLock`
(`warlock_core::ability`, Open lock's use; `KeyCommand::PickLock`, unbound). `Lock::difficulty`
is saved only when a level gives one (`skip_serializing_if`), so locks without one hash as
before. Level legends lock with `lock = "id"` or
`locked = true` (and `lock_difficulty = N`, 1 to 50); objects are keys with `key = "id"` (`"*"`: skeleton); closed doors break with
`breaks_to` + `sturdiness` (`TerrainDef::kicked_in`; digging and blasts break only terrain with a
`hardness`, `TerrainDef::breaks`, never a door). The AI (`world/ai.rs`) paths through a locked door, and opens it, only
with a key (`Stepped::Unlocked`); `go_look` stops next to a spot no one can stand on (a door kicked
at). Generated levels: `Needs::key` (the branch's `key` in `dungeons.toml`) makes `rooms.rs`'s
`lock_rooms` lock up to two single-door rooms and drop the key in an unlocked room;
`connect::validate` checks locks sit on closed doors and that the stairs and a key to each lock
(`LevelTemplate::has_key_for`, lying or carried) are reachable without unlocking anything; vaults
must hold their own keys (`content/vault.rs`, `check_keys`). The app binds `k` (`KeyCommand::Kick`,
a direction prompt like open/close, bash, and dig) and bursts the door terrain's `toggle` particles on
`Event::DoorKicked`/`DoorBroken`.
Chests (`docs/dev/chests.md`) are opened, unlocked, picked, kicked, and knocked through the
same functions, one arm each for a shut chest's terrain.

# Chests

Developer notes for chests (slice C1 of docs/design/chests-and-treasure.md, #613); the rules
are in spec/08d-traps-doors-digging.md "Chests" and the data in spec/07b-data.md "Chests". Keep
it current with the code, as CLAUDE.md says. Trapped chests (C2), generated chests and their
loot (C3), mimics (C4), and treasure rooms (C5, C6) come later; each extends this file.

## Modules

| Module | What |
|---|---|
| `warlock-core/src/chest.rs` | `Chest { level, holds }` (the per-cell state), `ChestState` (shut, open, smashed: a terrain's `chest` field), `ChestLook` (what perception shows: band index, state, `locked` with Chest-wise; `name`), `Breaking` and `destroyed` (the per-stack draw, coins and gems spared on a smash) |
| `content/chest.rs` | `ChestRules` (`data/chests.toml`: `bands`, `destroyed`), `ChestBand`, `ChestLoot`, `ChestGold`; `band_of(level)` / `band_index` (the last band whose `from` the level reaches, the first below them all), `band(id)`; the checks |
| `content/terrain.rs`, `content/mod.rs` | `TerrainDef::chest`, `is_shut_chest`; `check_chests` (one terrain per state, shut not walkable, the others walkable, none a door, stairs, or breakable); a shut chest may have a `toggle_noise`; `Content::chests()`, `chest_terrain(state)` |
| `content/level.rs` | the legend's `chest` (`ChestSpec`: a band id or a level), locks on shut chests, a shut chest's `objects` into its `holds`; a `Chest` on every chest terrain cell (`LevelTemplate::chests`), of the legend's level else the level's `item_level`. `content/vault.rs` refuses chests (vault stamping does not carry them yet: C7) |
| `world/level.rs` | `LevelState::chests` (saved, hashed, `skip_serializing_if` empty: levels without chests hash as before); `chests()` for dev tools; `ids_valid_for` checks what chests hold |
| `world/chest.rs` | `chest_name`, `open_chest` (key or not), `lift_lid` (the creak, `Act::OpenDoor`, spill, `chest_open`, "Inside you find ..."), `kick_chest`, `break_chest` (smashed or blown up: `destroyed`, Deft breaker's cut on a smash, spill, `chest_smashed`, `Event::ChestBroken`, the lines) |
| `world/door.rs`, `world/travel.rs`, `world/lockpick.rs`, `world/dig.rs` | one arm each: `open_door` and the bump hand a shut chest to `open_chest`, `kick` to `kick_chest`, `pick_lock` picks a chest's lock and calls `lift_lid`, `blast` blows apart shut chests whose band's `hardness` is below it; `set_terrain` keeps a lock on a closed door or a shut chest |
| `world/knock.rs` | Knock (`ClassActive::Knock`, `TalentEffect::Knock { mana }`, `TalentMods::knock`): springs the lock in a direction; refused, no time or mana, where nothing locked stands. `ClassActive::strikes` keeps the "nobody hostile there" refusal to the aimed blows |
| `world/travel.rs` | `instantiate_level` makes the food chests hold now (`rot::made_at`) and cuts their gems for Jeweller, as a level's piles |
| `perception.rs` | `PerceptionView::chest_at(p)` (cells seen or remembered; `locked` only with Chest-wise on a shut chest in sight), `known_chests()` |
| `dump.rs` | `truth`'s chest lines (`chest_rows`), JSON `chests` |
| `warlock/src/ui/mod.rs`, `ui/look.rs`, `ui/effects.rs` | the band's colour on the map; look mode's band name and description; `ChestBroken` bursts the smashed chest's splinters |

## Invariants

- A chest's band is never saved: only its level. Re-cutting or renaming bands breaks no save.
- What a shut chest holds is ground truth: nothing in perception reads `holds`; opened, the
  contents are a pile like any (piles are what perception shows).
- An open or smashed chest keeps its (empty) entry, so it is named by its band; the entry's
  level never changes, so a remembered chest's band leaks nothing.
- Draws: the kick's break (one) and the destruction (one a stack rolled, in the order held) are
  the gameplay stream; a blast blowing a chest apart compares numbers and draws only the
  destruction. Nothing else about chests draws.
- No new action, key, or `KeyCommand`: `open`, `move`, `pick`, `kick`, and `use:knock:DIR`.

## Talents

System `chests` (`data/talents.toml`, after `doors`): Deft breaker (Might, Strength 12,
`smash_destroy_percent = -50`, read in `break_chest` on a smash only), Chest-wise (Delver,
Intelligence 11 or Search 15, `chest_wise`, read in `chest_at`), Lockbreaker (its `systems`
gained `chests`: `door_kick` is read by `kick_chest`, `kick_noise_percent` by the kick's noise),
and the wizard's Knock (level 4, 5 mana).

## Tests

`content/chest.rs` (the shipped bands parse and rise, `band_of`, refusals), `world/chest_tests.rs`
(legends, a bump and `open`, keys, Pick lock, Knock and its refusals, the wizard's table, kicks
that hold and that smash, coins and gems spared, the smash's half and Deft breaker's quarter, a
blast sparing nothing and not reaching a kist, digging refused, the snapshot, Chest-wise, a
save and replays, the dumps), `warlock/src/ui/look.rs` (`a_chest_goes_by_its_band`), the
`abilities-menu-knock` screen snapshot, and `render/atlas.rs` (the font has `⊟ ⊔ ⊠`).

# Chests

Developer notes for chests (slices C1, #613, and C2, #614, of docs/design/chests-and-treasure.md);
the rules are in spec/08d-traps-doors-digging.md "Chests" and the data in spec/07b-data.md
"Chests". Keep it current with the code, as CLAUDE.md says. Generated chests and their loot (C3),
and treasure rooms (C5, C6) come later; each extends this file. Mimics (C4, #616) have their
own topic, `docs/dev/creatures/mimics.md`: a chest may be a mimic, which the perception snapshot
shows as a shut chest (`chest_at`, `known_chests`), and touching what the player takes for a
chest (`open_door`, the bump, `kick`, `pick_lock`, Knock) finds it out first.

## Modules

| Module | What |
|---|---|
| `warlock-core/src/chest.rs` | `Chest { level, holds }` (the per-cell state), `ChestState` (shut, open, smashed: a terrain's `chest` field), `ChestLook` (what perception shows: band index, state, `locked` with Chest-wise; `name`), `Breaking` and `destroyed` (the per-stack draw, coins and gems spared on a smash) |
| `content/chest.rs` | `ChestRules` (`data/chests.toml`: `bands`, `destroyed`), `ChestBand`, `ChestLoot`, `ChestGold`; `band_of(level)` / `band_index` (the last band whose `from` the level reaches, the first below them all), `band(id)`; the checks |
| `content/terrain.rs`, `content/mod.rs` | `TerrainDef::chest`, `is_shut_chest`; `check_chests` (one terrain per state, shut not walkable, the others walkable, none a door, stairs, or breakable); a shut chest may have a `toggle_noise`; `Content::chests()`, `chest_terrain(state)` |
| `content/level.rs` | the legend's `chest` (`ChestSpec`: a band id or a level), locks on shut chests, a trap on a shut chest (`parse_trap`: `TrapEffect::fits_fixture`, no pit; none on an open or smashed chest), a shut chest's `objects` into its `holds`; a `Chest` on every chest terrain cell (`LevelTemplate::chests`), of the legend's level else the level's `item_level`. `content/vault.rs` refuses chests (vault stamping does not carry them yet: C7) |
| `world/level.rs` | `LevelState::chests` (saved, hashed, `skip_serializing_if` empty: levels without chests hash as before); `chests()` for dev tools; `ids_valid_for` checks what chests hold |
| `world/chest.rs` | `chest_name`, `open_chest` (key or not), `open_lid` (the trap first, then `lift_lid` unless it blew the chest up), `spring_chest_trap` (`spring_trap`, then a burst on a shut chest blows it up: `TrapEffect::blows_fixture`, unless the burst's blast already did), `lift_lid` (the creak, `Act::OpenDoor`, spill, `chest_open`, "Inside you find ..."), `kick_chest` (broken: the trap, then smashed unless blown up), `break_chest` (smashed or blown up: `destroyed`, Deft breaker's cut on a smash, spill, `chest_smashed`, `Event::ChestBroken`, the lines), `chest_wise_look` (Chest-wise: hidden traps of shut chests in sight known, remembered, and said; called in `perceive` after `learn_traps`) |
| `world/door.rs`, `world/travel.rs`, `world/lockpick.rs`, `world/dig.rs` | one arm each: `open_door` and the bump hand a shut chest to `open_chest`, `kick` to `kick_chest`, `pick_lock` picks a chest's lock and calls `open_lid`, `blast` blows apart shut chests whose band's `hardness` is below it (their traps dropped, sprung at no one); `set_terrain` keeps a lock on a closed door or a shut chest, and drops a chest's trap and its memory when the chest stops being shut |
| `world/trap.rs`, `world/disarm.rs`, `world/search.rs` | trapped chests (C2): `step_onto_trap` skips a shut chest's cell; `slit` is the chest's own cell (a shot out of the box); a disarm failed badly springs through `spring_chest_trap` (a burst blows the chest up); `reveal` says "that the coffer is trapped" |
| `world/knock.rs` | Knock (`ClassActive::Knock`, `TalentEffect::Knock { mana }`, `TalentMods::knock`): springs the lock in a direction; refused, no time or mana, where nothing locked stands. `ClassActive::strikes` keeps the "nobody hostile there" refusal to the aimed blows |
| `world/travel.rs` | `instantiate_level` makes the food chests hold now (`rot::made_at`) and cuts their gems for Jeweller, as a level's piles |
| `perception.rs` | `PerceptionView::chest_at(p)` (cells seen or remembered; `locked` only with Chest-wise on a shut chest in sight), `known_chests()` |
| `dump.rs` | `truth`'s chest lines (`chest_rows`: `trapped (arrow)`), JSON `chests`; `view` and `memory` keep a chest's glyph over its known trap |
| `warlock/src/ui/mod.rs`, `ui/look.rs`, `ui/effects.rs` | the band's colour on the map, the chest's glyph over its known trap; look mode's band name, its known trap after it ("A coffer, trapped: an arrow trap."), and the description; `ChestBroken` bursts the smashed chest's splinters |

## Invariants

- A chest's band is never saved: only its level. Re-cutting or renaming bands breaks no save.
- What a shut chest holds is ground truth: nothing in perception reads `holds`; opened, the
  contents are a pile like any (piles are what perception shows).
- An open or smashed chest keeps its (empty) entry, so it is named by its band; the entry's
  level never changes, so a remembered chest's band leaks nothing.
- Draws: the kick's break (one) and the destruction (one a stack rolled, in the order held) are
  the gameplay stream, as are a sprung trap's own rolls; a blast blowing a chest apart compares
  numbers and draws only the destruction. Chest-wise draws nothing. Nothing else about chests
  draws.
- A chest's trap lies in `LevelState::traps` on the shut chest's cell and nowhere else: it is
  sprung only by `open_lid`, `kick_chest` (broken), and a failed `disarm`, always through
  `spring_chest_trap`; it goes with the lid (`set_terrain`). The order on a kick: the break draw,
  then the trap at the kicker, then the smash (or the blow-up). A kick that holds springs nothing.
- No new action, key, or `KeyCommand`: `open`, `move`, `pick`, `kick`, and `use:knock:DIR`.

## Talents

System `chests` (`data/talents.toml`, after `doors`): Deft breaker (Might, Strength 12,
`smash_destroy_percent = -50`, read in `break_chest` on a smash only), Chest-wise (Delver,
Intelligence 11 or Search 15, `chest_wise`, read in `chest_at` for the lock and in
`chest_wise_look` for the trap), Trap sense (its `trap_damage_percent` reaches a chest's trap
through `trap_deals`, no hook of its own), Lockbreaker (its `systems`
gained `chests`: `door_kick` is read by `kick_chest`, `kick_noise_percent` by the kick's noise),
and the wizard's Knock (level 4, 5 mana).

## Tests

`content/chest.rs` (the shipped bands parse and rise, `band_of`, refusals), `world/chest_tests.rs`
(legends, a bump and `open`, keys, Pick lock, Knock and its refusals, the wizard's table, kicks
that hold and that smash, coins and gems spared, the smash's half and Deft breaker's quarter, a
blast sparing nothing and not reaching a kist, digging refused, the snapshot, Chest-wise, a
save and replays, the dumps), `world/chest_trap_tests.rs` (C2: legends, an arrow out of the box
on a bump, a fireball blowing a coffer apart and a frost trap a kist, a kick that holds springing
nothing and one that breaks springing the trap, a fireball kicked, Pick lock, Disarm made and
failed badly, searching with the chest's sign and the `view` glyph, Chest-wise, Trap sense's half,
a blast from afar springing nothing, the `truth` line, replays), `warlock/src/ui/look.rs`
(`a_chest_goes_by_its_band`, with a known trap), the
`abilities-menu-knock` screen snapshot, and `render/atlas.rs` (the font has `⊟ ⊔ ⊠`).

# Chests

Developer notes for chests (slices C1, #613, C2, #614, C3, #615, and C7, #619, of
docs/design/chests-and-treasure.md); the rules are in spec/08d-traps-doors-digging.md "Chests",
the loot in spec/08g-magic-items.md "Chest loot", and the data in spec/07b-data.md "Chests".
Keep it current with the code, as CLAUDE.md says. Treasure rooms (C5, #617) and greater
treasure rooms (C6, #618) lay their chests with `mapgen/chest.rs::lay` from their own stream
(`docs/dev/special-rooms.md` "Treasure rooms"). Mimics (C4, #616) have their own topic,
`docs/dev/creatures/mimics.md`: a generated chest may be a mimic instead
(`mapgen/chest.rs::lay`), which the perception snapshot shows as a shut chest (`chest_at`,
`known_chests`), and touching what the player takes for a chest (`open_door`, the bump, `kick`,
`pick_lock`, Knock) finds it out first.

## Modules

| Module | What |
|---|---|
| `warlock-core/src/chest.rs` | `Chest { level, holds }` (the per-cell state), `ChestState` (shut, open, smashed: a terrain's `chest` field), `ChestLook` (what perception shows: band index, state, `locked` with Chest-wise; `name`), `Breaking` and `destroyed` (the per-stack draw, coins and gems spared on a smash) |
| `content/chest.rs` | `ChestRules` (`data/chests.toml`: `bands`, `destroyed`), `ChestBand`, `ChestLoot`, `ChestGold`; `band_of(level)` / `band_index` (the last band whose `from` the level reaches, the first below them all), `band(id)`; the checks |
| `content/terrain.rs`, `content/mod.rs` | `TerrainDef::chest`, `is_shut_chest`; `check_chests` (one terrain per state, shut not walkable, the others walkable, none a door, stairs, or breakable); a shut chest may have a `toggle_noise`; `Content::chests()`, `chest_terrain(state)` |
| `content/level.rs` | the legend's `chest` (`ChestSpec`: a band id or a level), locks on shut chests, a trap on a shut chest (`parse_trap`: `TrapEffect::fits_fixture`, no pit; none on an open or smashed chest), a shut chest's `objects` into its `holds`; a `Chest` on every chest terrain cell (`LevelTemplate::chests`), of the legend's level else the level's `item_level` |
| `content/vault.rs`, `mapgen/vault.rs` | vaults hold chests (C7, #619): `Vault::new` refuses a chest with a level of its own (`chest = N` or a band: its level is the floor's), a mimic, and chests in an outdoor vault; `check_keys` asks no key for a shut chest's lock (as `connect::validate`); `stamp` copies the chests onto `Canvas::chests`, and `generate_for` gives them the floor's item level |
| `mapgen/chest.rs` (`fill_vaults`) | what the vault chests stamped with no `objects` hold: `roll_contents` at the floor's level, in position order, from `vault_chests/<level id>` (`Rng::vault_chests`); run before `fill`. Their lock and trap are the legend's; nothing else is drawn, and none becomes a mimic |
| `world/level.rs` | `LevelState::chests` (saved, hashed, `skip_serializing_if` empty: levels without chests hash as before); `chests()` for dev tools; `ids_valid_for` checks what chests hold |
| `world/chest.rs` | `chest_name`, `open_chest` (key or not), `open_lid` (the trap first, then `lift_lid` unless it blew the chest up), `spring_chest_trap` (`spring_trap`, then a burst on a shut chest blows it up: `TrapEffect::blows_fixture`, unless the burst's blast already did), `lift_lid` (the creak, `Act::OpenDoor`, spill, `chest_open`, "Inside you find ..."), `kick_chest` (broken: the trap, then smashed unless blown up), `break_chest` (smashed or blown up: `destroyed`, Deft breaker's cut on a smash, spill, `chest_smashed`, `Event::ChestBroken`, the lines), `chest_wise_look` (Chest-wise: hidden traps of shut chests in sight known, remembered, and said; called in `perceive` after `learn_traps`) |
| `world/door.rs`, `world/travel.rs`, `world/lockpick.rs`, `world/dig.rs` | one arm each: `open_door` and the bump hand a shut chest to `open_chest`, `kick` to `kick_chest`, `pick_lock` picks a chest's lock and calls `open_lid`, `blast` blows apart shut chests whose band's `hardness` is below it (their traps dropped, sprung at no one); `set_terrain` keeps a lock on a closed door or a shut chest, and drops a chest's trap and its memory when the chest stops being shut |
| `world/trap.rs`, `world/disarm.rs`, `world/search.rs` | trapped chests (C2): `step_onto_trap` skips a shut chest's cell; `slit` is the chest's own cell (a shot out of the box); a disarm failed badly springs through `spring_chest_trap` (a burst blows the chest up); `reveal` says "that the coffer is trapped" |
| `chest.rs` (`roll_contents`) | what a generated chest holds (C3): the band's `loot` table rolled at the floor's level + `loot_level` with the finder's luck: the items (a grade draw each, then `loot::roll_with`), the purse (the shops' currency), the gems (`gem::roll`); like stacks joined |
| `content/dungeon.rs` | `ChestOdds` (`chests = { chance, count }` per branch; default 20 %, one) on `DungeonDef::chests` |
| `mapgen/chest.rs` | `fill` (C3): the chance and count, the cells (`cells`: plain floor with eight walkable non-door neighbours, `AWAY` 6 from the stairs, no vault, shop, kept special room, object, light, trap, lock, or patrol; two chests never touch), and per chest `lay`: lock, trap (of the branch's that fit a fixture), contents, the mimic draw, the shut terrain and the `Chest`. Run in `generate_for` after `room_loot::fill`, before `gear::arm` |
| `mapgen/connect.rs` | `validate` accepts a lock on a shut chest and asks no key for it |
| `world/knock.rs` | Knock (`ClassActive::Knock`, `TalentEffect::Knock { mana }`, `TalentMods::knock`): springs the lock in a direction; refused, no time or mana, where nothing locked stands. `ClassActive::strikes` keeps the "nobody hostile there" refusal to the aimed blows |
| `world/travel.rs` | `instantiate_level` makes the food chests hold now (`rot::made_at`), as a level's piles |
| `perception.rs` | `PerceptionView::chest_at(p)` (cells seen or remembered; `locked` only with Chest-wise on a shut chest in sight), `known_chests()` |
| `dump.rs` | `truth`'s chest lines (`chest_rows`: `trapped (arrow)`), JSON `chests`; `view` and `memory` keep a chest's glyph over its known trap |
| `warlock/src/ui/mod.rs`, `ui/look.rs`, `ui/effects.rs` | the band's colour on the map, the chest's glyph over its known trap; look mode's band name, its known trap after it ("A coffer, trapped: an arrow trap."), and the description; `ChestBroken` bursts the smashed chest's splinters |

## Content

- The Ascmearc's vaults (C7): the drake's hoard (two locked strongboxes, one with a spear
  trap), the robbers' hoard (a locked chest with an arrow trap), the captain's hall (the fort's
  locked pay chest), all filled by their band's table, and the still-room (Mother Sallow's
  chest, unlocked, holding cures and coin as its legend says).
- Every branch keeps the default `chests = { chance = 20, count = [1, 1] }` (the design names no
  other).
- The showcase's `chests` scenario (level `treasure_hall`, a thief): a coffer, a locked chest, a
  trapped one (a hidden arrow trap), and a mimic in a row; a treasure room behind a secret door;
  a greater treasure room walled up, its breach beside a pick-axe. For captures and the tests in
  `world/chest_hall_tests.rs`. Test world only (docs/dev/campaign.md).

## Invariants

- A chest's band is never saved: only its level. Re-cutting or renaming bands breaks no save.
- What a shut chest holds is ground truth: nothing in perception reads `holds`; opened, the
  contents are a pile like any (piles are what perception shows).
- An open or smashed chest keeps its (empty) entry, so it is named by its band; the entry's
  level never changes, so a remembered chest's band leaks nothing.
- Draws: the kick's break (one) and the destruction (one a stack rolled, in the order held) are
  the gameplay stream, as are a sprung trap's own rolls; a blast blowing a chest apart compares
  numbers and draws only the destruction. Chest-wise draws nothing. Nothing else about chests
  in play draws (generation: below).
- A chest's trap lies in `LevelState::traps` on the shut chest's cell and nowhere else: it is
  sprung only by `open_lid`, `kick_chest` (broken), and a failed `disarm`, always through
  `spring_chest_trap`; it goes with the lid (`set_terrain`). The order on a kick: the break draw,
  then the trap at the kicker, then the smash (or the blow-up). A kick that holds springs nothing.
- Generated chests (C3) draw only from `chests/<level id>` (`Rng::chests`), after everything but
  the gear: the map, traps, secrets, loot, population, dens, hoards, and scriptoria are the same
  with or without them (the chest's cell becomes the shut terrain, its lock and trap join the
  level's maps). Per chest, in order: the cell, `chance(locked)`, `chance(trapped)` and, if
  trapped, one `index` among the fitting traps (made with none: a branch without traps draws
  the same), `roll_contents`, `chance(mimic)` (made: no chest, the lock and trap drawn for it
  dropped, and a `CreatureSpawn` of `Content::mimic_at(level)` carrying the contents pushed on
  the template's creatures, nothing more drawn; `docs/dev/creatures/mimics.md`; never in a
  `shunned` room, a sanctuary, where the chest stays a chest, the draw made all the same, #661). Cells avoid the population, so a fixture comparing a floor with and
  without a danger sets `chests = { chance = 0 }` (`world/spawn_tests.rs`'s barrow).
- A vault's chests draw only from `vault_chests/<level id>`, and only for those with nothing in
  them. A vault chest is a cell that is not walkable, so stamping one moves the population's
  cells as any wall would (the lairs' bottom-floor snapshots changed with C7).
- No new action, key, or `KeyCommand`: `open`, `move`, `pick`, `kick`, and `use:knock:DIR`.

## Talents

System `chests` (`data/talents.toml`, after `doors`): Deft breaker (Might, Strength 12,
`smash_destroy_percent = -50`, read in `break_chest` on a smash only), Chest-wise (Delver,
Intelligence 11 or Detect traps 15, `chest_wise`, read in `chest_at` for the lock and in
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
`abilities-menu-knock` screen snapshot, `render/atlas.rs` (the font has `⊟ ⊔ ⊠`), and `mapgen/chest_tests.rs` (C3: one floor in five,
cells clear, the level the same without chests, locks and traps by the band, a branch without
traps drawing the same, banded chests at depth 8, the loot's means and magic share; C7: the
Ascmearc's four vaults' chests on their floors, of the floor's level, locked and trapped as
their legends say, the empty ones filled; a vault's refusals), `world/chest_hall_tests.rs` (C7:
the showcase's `chests` scenario, below);
`content/dungeon.rs` (`chests` parsed and refused); `xtask/src/wiki/chests.rs` (the dice means).

# Searching

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08d-traps-doors-digging.md ("Searching"). Keep it current with the code, as CLAUDE.md says.

Searching (spec §8 "Searching", #140) finds **hidden features**, secret doors and hidden traps,
with one check for all of them. Sealed rooms' breaches are hidden features too
(`Hidden::Hollow`, #618: a greater treasure room's thin wall, found as a hollow wall,
`LevelState::hollows`; `docs/dev/special-rooms.md` "Greater treasure rooms").

**Which skill finds what** (#663: Detect traps split from Spot and Search). Each kind of
`Hidden` reads one skill per look; the trap's is Detect traps either way:

| Hidden feature | Searching (`Look::Active`, Intelligence) | In passing (`Look::Passive`, Willpower) | Noticed in passing at all by | Mark on finding |
|---|---|---|---|---|
| `Hidden::Trap` (floor, door, shut chest) | Detect traps (`Effect::DetectTraps`) | Detect traps | Detect traps trained, a `notice` talent, elves (`All`), kobolds (`Traps`), dwarves and Stonewise on stone | `Use::FindTrap` both ways (in passing also `StatUse::Spot`) |
| `Hidden::Door` (secret door) | Search (`Effect::Search`) | Spot (`Effect::Spot`) | Spot trained, a `notice` talent, elves, dwarves and Stonewise on stone | `Use::Spot` in passing |
| `Hidden::Hollow` (a greater treasure room's breach) | Search | Spot | as a secret door | `Use::Spot` in passing |
| `Hidden::Mimic` (a disguised mimic) | Search | Spot | Spot trained, a `notice` talent, elves (never by stone) | `Use::Spot` in passing |

A turn of searching marks `Use::Search` whatever it finds; the reach (`Effect::SearchReach`,
Keen searcher) is Search's for every kind. Creatures and items in plain sight need no check.
The thief is the one class that starts trained in Detect traps (its `2d6+10`, known for since
#684 took the Trapfinding class ability, beside Spot's and Search's), kobolds the one race (`1d4+2`, in place of their Search); others may get it only
as #662's non-guaranteed class skills do, since level-up rolls advance trained skills only.

**Data**: `data/search.toml` (`content/search.rs`: `SearchRules { passive_penalty, turns,
secret_door }`, `Content::search()`; optional, `SearchRules::default` without it), the Search,
Spot, and Detect traps skills' benefits in `data/skills.toml` (`skills::Effect::Search`,
`SearchReach`, `Spot`, `DetectTraps`; marks on `Use::Search`, `Use::Spot`, `Use::FindTrap`), races' `passive_search` (`PassiveSearch::{All,
Stone, Traps}`) and `search` bonus (`RaceDef`), terrain `stone`, traps' `difficulty` and `sign`
(`TrapDef`), branches' `secret_doors` and `hidden_per_depth` (`DungeonDef::hidden_at`).

**Secret doors** are per-cell ground truth beside the wall they look like, like locks:
`content::Secret { door, difficulty }` in `LevelTemplate::secrets` and `LevelState::secrets`
(saved, hashed, left out of the JSON while empty, so levels without them hash as before). The
terrain grid holds the wall (`content::secret_look`, one rule for legends and the generator), so
everything that reads terrain (sight, light masks, memory, look mode, dumps, movement, the AI,
noise, digging, blasts) treats a secret door as that wall with no code of its own; perception
never carries secrets. `World::set_terrain` drops the secret whenever its cell changes (found,
dug, blasted): keep it the one way terrain changes. Legends: `secret = true` / `secret = N` on
an unlocked closed door with a wall beside it (`content/level.rs::hide_secret_doors`); vaults
stamp theirs (`mapgen/vault.rs`; outdoor vaults refuse them); the rooms generator hides
`secret_doors` of a level's doors last (`mapgen::hide_secrets`, stream `secrets/<level id>`,
after the traps; never a locked door, one on a patrol, a shop's, or a vault's), keeping each
only if `connect::validate` still passes: it treats secret cells as doors for reaching every
cell and as walls for reaching the stairs and the keys. The insta snapshots show them as `S`
where the map had `+`.

**The rules** are `world/search.rs`: `hidden_near(reach)` (the hidden features within reach,
never the player's own cell, in position order; beyond the adjacent cells only those in line of
sight), `difficulty_of` (own difficulty + `hidden_at(depth)`), `search_modifier(Look, Hidden)` (the
effective Intelligence or Willpower modifier, the skill effect through `Actor::skill_effect`,
Detect traps' for a trap, the race's `search`, the passive penalty), `notices(at, hidden)` (Spot
trained via `Skills::trains_effect(Effect::Spot)`, for a trap Detect traps, else the race's `passive_search`, `stone` read from the
cell's terrain, `traps` only for a `Hidden::Trap`; a `Hidden::Mimic`, a disguised mimic within
reach, is noticed in passing only by those who notice everything: `docs/dev/creatures/mimics.md`). `Action::Search(turns)` → `Spent::Search` → `World::search` runs `keep_at`
with `search_turn` (the `search` action's price first, then the active checks, then a Search
mark), stopping when `World::found` (an unsaved counter) moves; `notice_hidden` is the passive
check, called once per player turn at the start of `run_turn` (`schedule.rs`), so steps, waits,
rests, digs, meals, and searches all get it. `reveal` turns a door into its door
(`set_terrain`) or knows a trap (`World::know_trap`, shared with `learn_traps`), writes memory
at once (`memory`, `trap_memory`), and `say`s, which stops long actions. A trap on a door (spec
§8 "Trapped doors") is a hidden trap like any other (`hidden_near` reads `LevelState::traps`
whatever the terrain); found, it is said without the trap's `sign`, which is the floor's ("You
notice that the door is trapped: an arrow trap."); so is a shut chest's ("You notice that the
coffer is trapped: an arrow trap.", docs/dev/chests.md). Doors and chests are no `stone`, so
dwarves do not notice their traps in passing.

**Invariants**: checks draw from the gameplay stream (`crate::check`, two draws each) only for
hidden features in reach, so runs without any nearby keep their hashes; no memory of failed
checks and no cooldown (the owner's rule: no anti-farming limits); presentation never sees a
difficulty, a modifier, or a roll. The app binds `s` (`KeyCommand::Search`, `search`) and `S`
(`KeyCommand::SearchLong`, `search:N` with `SearchRules::turns`); basic commands, not
abilities. `dump`'s `truth` layer and `warlock-cli gen` mark secret doors `S`; `truth` marks a
standing breach `H`.

**Tests**: `world/search_tests.rs` (looks, finding, draws, who notices, traps and the skill each
look reads (`traps_are_detect_traps_to_find_and_the_rest_spot_and_search`), long searches,
rests, reach, `set_terrain`, depth, saves and replays, legend errors), `content/search.rs`,
`mapgen/tests.rs` (`rooms_levels_hide_a_few_doors_of_their_own`, the secret checks in
`check`). The wiki page is `wiki/src/searching.md`, its tables `xtask/src/wiki/searching.rs`.

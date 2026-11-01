# Mimics

General conventions: `docs/dev/creatures.md`. The rules are spec/08b-creatures.md "Mimics", the
data spec/07b-data.md "Mimics", the design docs/design/chests-and-treasure.md §5 (slice C4,
#616). Chests themselves: `docs/dev/chests.md`.

## The family (`content/mimic.rs`, `data/mimics.toml`)

A generated family, as the animated blades are (`docs/dev/animated.md` "Families"), but its own
file and code: `Mimics::parse` reads `[family]`, `[creature]`, and the `[[band]]`s after the
chest bands (`data/chests.toml` now loads before the creatures: a mimic band must match a chest
band's `from`, one to one, and its slam's hold chance is the chest band's `hold`).
`Mimics::specs` builds one `CreatureSpec` per level of `levels` (`mimic_1` … `mimic_60`) as a
TOML table, so a member is checked exactly as a written type: the `[creature]` table, then the
member's own `id`, `name` and `fg` (its band's), `weight`, `stats`, `base_hp`, `level`, `xp` (the
guideline), `natural` (the body rule's hide, from level 18), and the attack's `damage` with an
`on_hit` hold (`chance` the chest band's `hold`, `difficulty` `Mimics::grip`, 12 + L ÷ 5). The
numbers are `Numbers::of` (`content/animated.rs`, the roster's formulas, shared with the
animated families) with the brute's adjustments from `[family]`.

- **Order**: `load_creatures` splices the members after the animated types and before the first
  campaign's (in level order). Adding a level or a band shifts every later id (saves change: the
  integrator's format bump), as adding a creature file does. Generation draws nothing.
- **`MimicDef`** on `CreatureDef::mimic` (`#[serde(skip)]` on the spec): the family's first type
  and the member's place in it, filled in at numbering.
- **The family reads**: `Content::family`, `family_key` (`mimic`), `family_levels`, and
  `name_families` (the creature id `mimic` names `mimic_1`) know mimics beside the animated
  families, so first kills, kill quests (`mimic` or `tag:mimic`), and the wiki's creature
  tables treat the family as one. `Content::mimic_at(level)` is the member of a level, clamped
  to the family's; `Content::is_mimic`.
- **Not a stray**: `stray = false` and no population names it (`mimic_roster_tests`); the
  generic list test leaves mimics out as it does animated things.

## In play (`world/mimic.rs`)

A generated chest is a mimic by its band's `mimic` chance (`mapgen/chest.rs::lay`, after the
chest's lock, trap, and contents are drawn: the lock and trap dropped, the contents carried, no
draw more), on any floor, of the floor's item level (`mapgen/chest_tests.rs`'s
`a_mimic_stands_in_for_a_chest_carrying_its_loot`), but never in a `shunned` room (a
sanctuary): there the chest stays a chest, the draw made all the same (#661,
`mapgen/tests.rs`'s `a_mimic_never_stands_in_a_sanctuary`).

Every mimic is placed **disguised** (`travel::make_creature` adds the status that `hides`,
`Content::disguise_status`, to any type with `mimic`): a legend's, a swapped chest's, the dev
`spawn`'s. No `CreatureSpawn` field: the type decides.

| Read | Where |
|---|---|
| No turn | `den.rs::slumbers` waits for a sleeper or a disguised actor |
| Unaware (sneak attacks land) | `ambush.rs::unaware` |
| No creature targets it | `ai.rs::nearest_hostile_in_sight` leaves out an actor whose effects `hides` |
| The snapshot | `Perception::compute` lists the level's disguised actors (`Disguise { pos, terrain, level }`) but those Chest-wise sees within `TalentMods::sees_mimics`; they are not in `actors`, `gear`, or `effects`; `sees_actors_at` is false on their cells; `PerceptionView::cell` shows the shut chest terrain there; `remember` writes it into memory; `chest_at` gives the band of the disguise's level (a remembered shut chest with no chest entry and no disguise: the floor's item level); `known_chests` lists them |
| Touch | `World::touch(p)` from `player_moves` (the bump), `open_door`, `kick`, and `pick_lock`: a disguised mimic there that the player does not see as one (`Perception::shows`) is revealed (`Reveal::Touch`: the lurch line and `World::attack` by the mimic at the player, its hold included) and the action is `Spent::NoTime` |
| Knock | `knock.rs`: the mana spent, `Reveal::Knocked` (the lurch line, no blow), a cast's price |
| A blow or harm | `fight.rs::resolve` (after `sneaking` is decided) and the top of `hurt`: `Reveal::Struck` |
| Searching | `search.rs`: `Hidden::Mimic(id)` within reach, difficulty `Mimics::grip` + `hidden_at(depth)`; in passing only for those who notice everything (Spot, `passive_search = "all"`, Alert): never by Stone sense or a kobold's traps; `Reveal::Found` |
| Stealing | `steal.rs`: "It has no pockets." (a disguised one is not seen: "You see no one there") |
| Chest-wise's glue | `status.rs::afflict`: a status with a `hold` given by an actor with the `mimic` tag to one with `TalentMods::slick` is refused (`World::slips`) |

`World::reveal_mimic` removes the status directly (not through `ended`: nothing is said of a
status the player never saw), says the line when the cell is in sight (`Perception::in_sight`;
`Found` always), and on a touch strikes. A revealed mimic never hides again.

Draws: none of its own; the touch's blow is the gameplay stream's as any attack, the search
checks too. A run that never meets a mimic is unchanged.

## Talents

System `chests` (the `mimics` tag folded into it, #773): **Chest-wise** (delver, Intelligence 11
or Detect traps 15) carries `sees_mimics` (value 2) and `slick` beside `chest_wise` (#775 merged
Mimic-wise and Slick into it).

## Dumps

`truth` shows the mimic's `m`; `actors` lists `disguised 1` among its effects; `view`, `memory`,
JSON `seen_actors` and `chests` show the chest the player sees.

## Tests

`content/mimic_roster_tests.rs` (one type a level on the brute formulas, the bands and names, no
population), `world/mimic_tests.rs` (the legend, the snapshot equal to a real chest's, touch,
Knock, a blow, searching and Spot, Chest-wise seeing it and slipping its grip, no turn and no foe, the drop, unaware, a
save and a replay, the dumps), `content/creature_body_tests.rs` (woody hide), and
`warlock/src/ui/look.rs` (`a_disguised_mimic_looks_like_a_chest`).

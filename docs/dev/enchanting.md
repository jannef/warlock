# Enchanting

Developer notes for working on this part of Warlock; the approved design is
`docs/design/enchanting.md` (#1414), built in phases E1 to E6 (#1427 to #1432). Keep it current
with the code, as CLAUDE.md says. Only E1 is built: affix capacity and the trim. Crystals, the
Soulglass spell, and the recipes are later phases.

## Affix capacity

`warlock-core/src/magic/capacity.rs`: `capacity(stack, content) -> u8`, the one read, is how many
affixes, prefixes and suffixes together, an item can hold:

| Term | Value | Read from |
|---|---|---|
| The base | 2 (`capacity::BASE`; the owner, 2026-11-13) | every item |
| Jewellery | +1 | `ObjectDef::is_jewellery()` (ring and amulet kinds) |
| Quality | Fine +0, Marvellous +1, Masterwork +2 (#1444; first +1/+2/+3) | `Worked::improvements()` − 1, never below 0 (none while damaged) |
| Material | −2 to +2 | `MaterialDef::binding` of the main part's material (`Stack::parts(content)[0]`, a variant's own) |
| The set gem | +1, the finest cut (flawless, the last `[[cut]]` of `gems.toml`) +2 | the part of `gem::ornament_part` whose material is a gem; a base item's own gem part (the gold amulet's ruby setting) has no cut and counts +1 |
| Clamp | 1 to `MOST` (6) | `MAX_PER_KIND × 2` |

`binding` is a new optional material field (`data/materials/*.toml`, default 0, steel's; the
loader refuses one outside −2..=2): silver −2; tin, pewter, zinc, cadmium, bendalloy, lead, stone,
flint, obsidian −1; gold, electrum, ironwood, ivory, jade, amber, silk, adamantium +1 (#1444:
a fighting metal, its gifts a die more, twice steel's Protection, and no item damage); mithril,
dragonbone, dragonscale +2. The three-per-kind caps (`Rarity::most_per_kind`) stand beside the
capacity. `Magic::check` never reads it: a save or level file over capacity still loads.

Only a damaged Fine, Marvellous, or Masterwork piece is ever over its capacity (its improvements
are gone while damaged, and a repair does not bring them back). Nothing strips it; the recipes of
later phases refuse to add to it.

## The trim

`capacity::trim(magic, capacity, rng, content)`: while the affixes outnumber the capacity, drop
one of the kind (prefix or suffix) the item has more of, one of that kind by one `rng.index`
draw, a suffix on a tie. Never below one affix. The rarity and a rare's name stay: a trimmed rare
is a rare still. `capacity::trimmed(content, rng, stack)` trims a stack to its own capacity.

`loot::finished(content, rng, stack)` is the trim as the very last step of making an item,
after its material, ornament, wand rung, and (for creature gear) its condition:

- `loot::make`, `make_as`, `roll`/`roll_from`/`roll_with`, `roll_pool` (through `drawn`),
  `heirloom`, and `favoured` (through `heirloom`) end with it, from the stream the item rolled
  from. Every caller (floor loot, chests, hoards, dens, room loot, animated things, kits, the
  background's gifts, the stipend, the pedlar's offer, the overworld's outcomes) gets a finished
  item.
- Creature gear makes its magic pieces with `loot::make_as_unfinished`; `gear::finish` trims the
  whole pack from the gear stream (`gear/<level id>` or `gear/<actor id>`) after
  `gear_condition::apply` (its own `condition/<key>` stream), in `gear::arm`,
  `world/travel.rs::placed_gear`, and the balance probe. So a Fine piece keeps a second affix and
  a damaged one is trimmed to its plain capacity.
- An item that fits draws nothing, so its stream is as before; a trimmed one shifts what follows
  on that stream (the next find of a hoard, the next piece of a creature's gear).
- Hand-made items are trimmed too, at load (`content/level.rs::hand_made`, for a level's
  `[[item]]` and a unique item), from the stream `trim/<key>` of seed 0 (`<key>` the unique's id,
  or `<object>@x,y` for an `[[item]]`). The shipped ones are given the make their affixes need, so
  it never fires on them: `magic/capacity_tests.rs::hand_made_items_are_never_trimmed` reads the
  affixes written in every asset file's `[[unique]]` and `[[item]]` and fails with the item and
  its shortfall, and checks the heirloom (every race's weapon at the effect's `max`).

## Deeper finds are better made

#1436 (the owner, 2026-11-13): deeper finds come in better make and materials, so the trim leaves
them more affixes; the capacity and the trim are unchanged, and everything still ends in
`loot::finished`.

- **The make**: `data/gear_condition.toml`'s `[[depth]]` rows (`content::Depth`, by rising item
  level; `GearConditions::depth_at`) give per mille of Fine, Marvellous, and Masterwork. Loot
  (every find `make_from` makes with `Source::Loot`: floor loot, chests, hoards, dens, room loot,
  vault loot) draws its make with `world/gear_condition.rs::make` after the wand's rung and
  before the trim, from the loot stream: one draw, then one per improvement's aspect; none where
  the row gives nothing (item levels 1 and 2 have no row) or for what takes no make (not single
  arms or armour with an aspect). Creature gear adds the row of its level's item level to its
  tribe's odds (`Odds::deeper`: damaged gives way to what the improvements leave), the same one
  condition draw as before. The loader keeps every row's improvements with any table's within
  1000. Kits, the background's gifts, the stipend, the pedlar, and animated things (`loot::make`,
  `make_as` with `Source::Gear`) take none.
- **Gems**: the row's `ornament` adds percent to loot's `[[ornament]]` chance (`gem::ornament`'s
  `more`), the same one draw.
- **Materials**: in the material files' `rarity` (and `gear`): mithril from item level 8,
  `deeper = 8`; dragonbone and dragonscale from 6, `deeper = 5`; adamantium from 16, `deeper = 6`;
  gold `deeper = 3`. The draw itself (`variant::draw`, one draw) is unchanged.

Shallow levels draw exactly as before: no row below item level 3, and the material weights at
item level 1 are unchanged. Deeper, the make's draws come after the old ones on the loot stream,
so a level's later finds differ from before.

## Quality on hand-made items

`magic::Quality` (`fine`, `marvellous`, `masterwork`) is a data key on a unique
(`data/uniques/`), a level's `[[item]]`, and the `heirloom` talent effect. `Quality::worked(def)`
gives the item that many improvements in its aspects in turn (the first first, round again), each
one attempt, with no draw; refused at load for an item with no aspect (a ring). The shipped
changes (design §4.1): the Brosinga Mene (a mithril chain, a flawless ruby setting), the
Eorcanstān (a mithril band, a flawless sapphire setting), Grendles Grāpe (Masterwork mithril),
High Toby (Fine, an amethyst inlay), the Turncoat (Masterwork), the Unhalwed Bedes (a flawless
smokestone setting), Widowmaker (Marvellous); in the test world Rune Song (Masterwork, a topaz tip), Ash Coat (Masterwork), the hub's Dread
Visage (Masterwork) and Storm Song (Marvellous). Father's sword's heirloom is Masterwork
(`quality = "masterwork"` in `data/talents/fathers_sword.toml`: capacity 4 on every race's
weapon since #1444, its 2 to 4 affixes never trimmed). The design chose these against a base of 1
and quality +1/+2/+3; with the base at 2 and quality +0/+1/+2 (#1444) every one still fits, and
the two that no longer needed a make (Stonebiter and the trap hall's dagger, two affixes each)
are plain again.

## What the player sees

The item description (`magic/describe.rs::appraised_for`, through `Appraisal::capacity`) adds one
line after the affixes, before a unique's power and lore, only for a holder of Eye for the make
(the craftsperson's talent, `TalentMods::eye_for_make`) or a character trained in Appraise (any
value above 0); `PerceptionView::appraisal` is the one read: `It can bear two enchantments.` for a plain item
or one whose magic is unidentified (the count of its affixes stays hidden), `It bears one
enchantment and could bear one more.`, `It bears all it can.`, `It bears more than its make should
hold.` Everyone else is never told (`world/appraise_tests.rs`,
`world/craftsperson_tests.rs::eye_for_the_make_tells_the_capacity_without_appraise`).

The dumps show the truth: the `inventory` layer adds `(holds 1/2)` (affixes held / capacity) to
every base item, and the `actors` layer's items (`Stack::describe_truth`) the same.

## Tests

`magic/capacity_tests.rs`: the design's examples (the silver ring's clamp among them), a damaged
and repaired Masterwork, `binding` from a variant's part, the trim's sequence on 2+2, 3+3, 3+1,
and 1+3 at capacities 1 to 5 with the rarity and name kept, the trim's draws only when it trims,
every loot path at item levels 1 to 20 and every creature's gear after its condition within
capacity, and `hand_made_items_are_never_trimmed`. `magic/shares_tests.rs` prints the share of
rares the trim leaves at each affix count, and holds deep rares to keeping more (#1436); the
`[[depth]]` rows' loader checks and `Odds::deeper` are tested in `content/gear_condition.rs`, loot's
make in `world/gear_condition_tests.rs`; `magic/describe` tests the capacity line.

## The measured shares

`magic/shares_tests.rs` at item levels 1, 4, 8, 12, 16, and 20 (4 000 rares and 4 000
magic-grade finds each, loot's bases of every class), with the base at 2, deeper finds better
made (#1436), and the quality's term scaled down (Fine +0, Marvellous +1, Masterwork +2) and
adamantium's binding to +1 (#1444). Before #1436 every level looked like level 1: about 87 % of
bases held 2 and 10 % to 12 % of rares kept three or more. With #1436 and the old quality term
(+1/+2/+3) rares kept three or more 11 %, 15 %, 28 %, 49 %, 57 %, and 64 % of the time at the six
levels.

| Item level | Rares: capacity 2 | Rares keep 3+ | Rares untrimmed | Magic-grade untrimmed |
|---:|---:|---:|---:|---:|
| 1 | 85.5 % | 11.3 % | 8.4 % | 87.5 % |
| 4 | 84.2 % | 13.9 % | 8.8 % | 88.0 % |
| 8 | 81.0 % | 17.4 % | 10.7 % | 88.5 % |
| 12 | 72.8 % | 26.1 % | 16.8 % | 87.8 % |
| 16 | 64.9 % | 33.7 % | 21.4 % | 89.8 % |
| 20 | 57.5 % | 41.0 % | 26.1 % | 89.8 % |

The test fails if rares at item levels 1 and 4 keep three or more a fifth of the time or more, at
16 and 20 a quarter of the time or less (half before #1444), or if the share falls from one level
to the next. Magic-grade finds (one or two affixes) are untrimmed about 88 % to 90 % of the time at
every level.

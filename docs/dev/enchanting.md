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
| Quality | Fine +1, Marvellous +2, Masterwork +3 | `Worked::improvements()` (none while damaged) |
| Material | −2 to +2 | `MaterialDef::binding` of the main part's material (`Stack::parts(content)[0]`, a variant's own) |
| The set gem | +1, the finest cut (flawless, the last `[[cut]]` of `gems.toml`) +2 | the part of `gem::ornament_part` whose material is a gem; a base item's own gem part (the gold amulet's ruby setting) has no cut and counts +1 |
| Clamp | 1 to `MOST` (6) | `MAX_PER_KIND × 2` |

`binding` is a new optional material field (`data/materials/*.toml`, default 0, steel's; the
loader refuses one outside −2..=2): silver −2; tin, pewter, zinc, cadmium, bendalloy, lead, stone,
flint, obsidian −1; gold, electrum, ironwood, ivory, jade, amber, silk +1; mithril, adamantium,
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
(`quality = "masterwork"` in `data/talents/fathers_sword.toml`: capacity 5 on every race's
weapon, its 2 to 4 affixes never trimmed; Marvellous, 4, would now hold them too). The design
chose these against a base of 1; with the base at 2 every one still fits with room to spare, and
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
rares the trim leaves at each affix count; `magic/describe` tests the capacity line.

## The measured shares

`magic/shares_tests.rs` at item levels 1, 4, 8, and 12, with the base at 2 (the owner accepted
them, 2026-11-13; design §12 expected more trimming headroom for rares): about 87 % of loot's
bases have capacity 2, 2 % to 3 % capacity 1 (silver, the soft metals and knapped stone), and
about 10 % 3 or more (jewellery, gems, Fine gear, the precious materials). Rares roll three to six
affixes, so only 7 % to 8 % are left untrimmed: about 87 % keep two, 10 % to 12 % keep three or
more. Finds of the magic grade are untrimmed about 87 % of the time; about 70 % hold one affix and 28 %
two.

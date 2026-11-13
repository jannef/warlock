# Enchanting

Developer notes for working on this part of Warlock; the approved design is
`docs/design/enchanting.md` (#1414), built in phases E1 to E6 (#1427 to #1432). Keep it current
with the code, as CLAUDE.md says. E1, E2, and E4 are built: affix capacity and the trim; the
crystals, consuming one, and salvage's affix checks; the profession with Kindle, Quicken, and
Unbind. The Soulglass spell, Recast, Imbue, Ennoble, and the seals are later phases.

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

## Crystals

`data/crystals.toml`, loaded by `content/crystal.rs` (`Content::crystals`, optional: without it
no crystal is caught or consumed, and the Consume list offers none), after the creatures, affixes,
and objects:

- `[[colour]]` rows, a `Vec` in file order (black, blue, white, red, green, gold; the owner's
  order): `id`, `name`, `object` (`objects/crystal_<id>.toml`: class `materials`, stackable,
  20 g, value 25, `*` in the colour, made of stone, no `level` or `frequency`), `nature` (the
  wiki's words), `tags`, `families` (affix categories). `Crystals::colour_of(content, creature)`
  is the first colour one of whose tags the creature type carries (`None`: none);
  `colour_of_category` / `colour_of_affix(content, affix_id)` the first colour whose families
  hold the category; `colour_of_object` the colour a crystal object is (the rare materials are
  none). The design's `crystal = "white"` override on a creature is not built (no creature needs
  one yet).
- `[[family]]`: every affix category once with the words a seal will name it by (E6).
- `[yield] levels = [10, 20]`: `yield_at(level)`, Soulglass's count (E3): one, plus one per level
  reached.
- `[rare]`: `materials` (`heart_crystal`, `soul_fetter`: class `materials`, 30 g, value 300, not
  crystals), Soulglass's `level`, `chance`, `dragon` (E3), and salvage's `salvage_level = 10`,
  `salvage_chance = 3`.
- `[consume] mana = 6`, `marks = 12` (a tenth of a point's price at score 10: 20 + 10 × 10 =
  120 marks; a minor stat potion gives 15, so a crystal is worth drinking for Willpower alone
  without outdoing a potion made for it).

The loader refuses: no colour; a colour twice; an unknown, non-stacking, or shared object; a
malformed tag (that some creature carries each tag is a test, `every_colours_tag_is_carried`, not
a loader check: tests load content without a campaign, and `outlaw` is the Ascmearc's; so is that
every family is an affix's category, `every_family_is_an_affix_category`: tests strip affixes); **an affix category in no colour** (it could never be enchanted: add the
category to a colour when adding an affix); a category with no `[[family]]` word, or one twice; a
rare material that is a crystal; a chance above 100; `[consume] mana` or `marks` below 1.
Crystals and rare materials are in the `never` lists of the shops that stock `materials`
(`data/shops.toml`). The game says what a colour binds only in words, through Morwenna, a deep
gnome on Gimmwīc's cutting floor (`npc = "morwenna"`, `campaigns/ascmearc/data/dialogue/
gimmwic.toml`; `docs/dev/creatures/deep-gnomes.md`): her lines are hand-written from the
families, so change them with the colours.

`data/enchanting.toml` (`EnchantingRules`, `Content::enchanting`): `salvage_difficulty = 12`,
`salvage_tier_step = 3` (`salvage_against(affix)` = 12 + tier ÷ 3, a tier-12 affix +4),
`level_step = 3` (the recipes' check, below) and `seal_seconds = 1800` (E6).

## Consuming a crystal

A crystal is consumed as food is eaten and a potion drunk: it is in the Consume list (`e`,
`ObjectList::Eat`, whose filter takes `PerceptionView::is_crystal` beside food and drinks), and
the action is `Action::Eat` (`eat:b`, `eat:floor:a`): `world/eat.rs::eat` hands a crystal to
`world/crystal.rs::consume_crystal` before it looks for a drink or food. No ability, no screen of
its own, no hunger needed. One crystal of the stack (from the pack or the pile underfoot), a
turn of `ActionKind::Drink`, no check, no draw: `[consume] mana` (to the maximum, as a potion's
mana; never refused for a full pool), `train_stat_times(StatUse::Consumed, None, [consume]
marks)` (`data/stats.toml`: Willpower, 100 %, so twelve marks), and
`player_did(Deed::ConsumedCrystal)` (`[shift] consume_crystal = -20` in `data/alignment.toml`,
diminishing as every deed does). "You crush the red crystal and drink what is in it. Something
of it stays." In a shop it is bought as a drunk potion is (`drank_goods`; no shop stocks
crystals). The rare materials are no crystals: the list leaves them out, and `eat` refuses them
("You cannot consume a heart-crystal.").

## Salvage's affix checks

`world/crystal.rs::catch_magic`, called by `world/salvage.rs::salvaged` before its material
checks (so its draws come first; nothing changes until all are drawn). A magic or rare item,
**identified or not** (it is gone at once, so what its crystals' colours tell matters to no one;
the owner, 2026-11-13): one check per affix in the item's order, `combat::roll_natural` (two gameplay draws) +
Willpower's modifier + `TalentMods::check(Willpower)` + `skill_effect_of(enchanting, Craft)`
(0 until E4 gives Enchanting its `craft` curve) against `salvage_against(affix)`, `check::makes`.
Made: one crystal of `colour_of_affix` ("You catch its Flaming magic in a red crystal.", "You
catch its magic of the Bear in a red crystal."); failed: "Its Flaming magic goes out unread.".
An unidentified item's lines name no affix: "You catch some of its magic in a red crystal.",
"Some of its magic goes out unread.".
Then a rare of item level `salvage_level` or more draws `chance(salvage_chance)`, and a made draw
`index` among `materials` ("Where its magic was, a heart-crystal is left."). The crystals join
the saved list after the materials ("You save 2 steel ingots and a red crystal."). Marks:
`Use::Salvage` on Enchanting (its new `mark = [{ on = "salvage", chance = 100 }]`) when any affix
check was made; untrained, none.

## The recipes

E4 (#1430; design §7, §8). Enchanting is a profession: `data/recipes/enchanting/` holds
`kindle.toml`, `quicken.toml`, and `unbind.toml`, so its row "Enchanting…" is in the abilities
menu for the trained, and `data/recipes.toml`'s `[[profession]]` gives it "Your work as an
enchanter.", `stat = "willpower"`, `verb = "Enchant"`. `data/skills.toml` gives Enchanting
the professions' `craft` curve (0, 8 at 100, 12 at 200; salvage's affix checks read it too), its
reach `enchant_level` (`Effect::EnchantLevel`, `[[0, 1], [110, 12], [200, 12]]`, at most
`MAX_ENCHANT_LEVEL` 12), and a `craft` mark at 100 %.

- **The data** (`content/recipe.rs`): `use = "enchant"` with `enchant = "kindle" | "quicken" |
  "unbind"` (`RecipeUse::Enchant(EnchantKind)`) and `crystals` (`RecipeDef::crystals`, 0 to
  `MOST_CRYSTALS` 3); `difficulty` and `seconds` required as Cut's and Set's (`checked()`). The
  loader refuses an `enchant` kind on another `use` or missing on `enchant`, a kind twice,
  `crystals` above 3 or on another `use`, a kind that rolls (Kindle, Quicken) with none, and
  Unbind with any.
- **The rules** (`world/enchant.rs`; `Spent::Enchant(Enchanting)`, dispatched by one line each
  from `World::start_craft`, `craft::lack`, `craft::takes`, `craft_make::craft_steps`, and
  `craft_plan`): `craft:kindle:ITEM,CRYSTAL`, `craft:quicken:ITEM,CRYSTAL`, `craft:unbind:ITEM`.
  The wizard's steps are the item (`CraftStepKind::Item`: the stacks `refusal` passes) and a
  crystal for each the recipe takes (`CraftStepKind::Input`, "crystal": every crystal carried,
  any colour). The plan's verb is the recipe's name ("Kindle"), its name "the long sword", its
  uses the crystal ("a red crystal"; nothing for Unbind, whose dialog says only its time).
- **Refused, no time**, in order: untrained (`World::untrained`), below `from`, no item pick, more
  picks than the work takes, no crystal pick, then `refusal`: no such stack, not a base item or a
  spirit weapon ("No magic can be bound in that."; ammunition is a base item, the whole stack
  worked at once at the single cost), magic unidentified ("You must know what is bound in it
  first."), worn or wielded ("Take it off first."), a unique ("It will not take your will.");
  Kindle on magic ("There is magic in it already."), Quicken or Unbind on a plain item ("There is
  no magic in it."), Quicken on a rare ("It is rare already: this work cannot add to it.": a
  rare's Quicken takes the heart-crystal, E6), Quicken on two affixes or at capacity ("It holds
  all it can."); then the crystal pick that is no crystal ("That is no crystal."), nothing of
  its colour's families left to roll in the kinds the work may add ("No more of that nature will
  take."), a quest's object ("It will not take your will."), and unpaid goods (`may_loose`).
- **The rows** (`enchant::lack`, after the skill): `RecipeLack::Nothing` ("nothing to enchant":
  no stack the work may take), `Room` ("it holds all it can": Quicken's, every refused stack
  short only of room), `Crystal` ("no crystal to hand"). `takes`: "a plain item, a crystal, about
  1 hour", "a magic item, a crystal, about 1 hour", "a magic item, about 30 minutes".
- **The roll's item level**: the reach is `skill_effect_of(enchanting, EnchantLevel)` clamped
  to 1..12; Kindle rolls at the reach, Quicken at the greater of the item's `Magic.level` and the
  reach, Unbind's check reads the item's own. The made item's `Magic.level` is the roll's.
- **The check** (`World::enchanted`, after `keep_at` ran the recipe's `seconds`; an interruption
  spends nothing but the time): `combat::roll_natural` (two gameplay draws) + Willpower's
  modifier + `TalentMods::check(Willpower)` + `skill_effect_of(enchanting, Craft)` against
  `difficulty` + level ÷ `level_step`; `enchant_band`: a natural 2 `Warped`, a natural 24 or a
  margin of 0 or more `Made`, else `Failed`.
- **Made** (`World::enchant_out`, `worked`): Kindle draws the kind by `[magic]`'s prefix and suffix
  weights among the kinds with a candidate (`magic::loot::pick_weighted`, one draw), then
  `roll_affix_within` (affix, tier, value: three draws) with the crystal's `families`; the item
  gets a `Magic` of rarity magic, the roll's level, identified. Quicken rolls the kind the item
  lacks (a prefix goes first among the affixes, a suffix last), the level the roll's. Unbind
  draws nothing and clears the `Magic` (`Stack::set_magic(None)`): its `Worked`, parts, rung,
  and charges stay. "The long sword takes the red crystal's glow. It is now a Sharp long sword.",
  "The magic goes out of Grim Bite. It is now a long sword." The item is changed in place,
  a whole stack at once.
- **Failed**: the crystal is spent ("The crystal goes dark in your hands."), the item as it was;
  Unbind's "The Sharp long sword clings to its magic." **Warped** (a natural 2): the crystal is
  spent and `World::harm_item(item, ItemHarm::Enchant)` ("warped", withstood by no material; an
  item of Proof is spared as protected): "The long sword warps under your hands.", or, warped
  already, destroyed ("... warps and falls apart.").
- **`roll_affix_within`** (`magic/loot.rs`): `roll_affix` is its call with every family; the
  candidates (`candidates`, sorted by affix id) are narrowed to the categories `families` holds.
  `rolls_within` says whether there is any, drawing nothing.
- **Marks**: `Use::Craft` on Enchanting, whatever came of it. No noise, no alignment.
- **The wiki**: `wiki/src/enchanting.md` and the generated `recipes-enchanting.md`
  (`xtask/src/wiki/enchanting.rs`: each recipe's crystals, time, listed from, difficulty, and the
  odds of a made check at three skill values, from `enchant_band`).
- **Snapshots**: `profession-enchanting`, `enchant-pick-item`, `enchant-dialog`
  (`crates/warlock/src/ui/screen_snapshots.rs`).

## Soulglass

The necromancy ray that leaves crystals (#1429; the design's §6): `data/spells/soulglass.toml`
(level 11, a ray with an attack roll, necrotic `3d6`, range 6, Leech's look and ranks' shape),
the witch's at level 16 (`talents/soulglass.toml`, `classes/10-witch.toml`), anyone's from its
grimoire (`objects/scroll_of_soulglass.toml`, level 6, frequency 20, as the other level-11
spells' scrolls; its grimoire one time in ten, `Content::grimoire_or`). A scroll or a wand of it
crystallises too: the effect is the spell's.

- **The key**: `[spell.effect] crystal = true` (`SpellEffect::crystal`), refused by `spell_def`
  off a `damage` spell or off a single-target shape (ray, bolt, touch), as `leech` is: the hook
  is in `cast.rs::spell_attack`, the one place a single target's roll is made, because
  `spell_strikes` does not know the spell.
- **The mark**: `spell_attack` sets `World::glassing` (not saved) to the target around its
  `spell_strikes`, whoever casts it (a reflected one marks the caster it turns on). A kill of that
  blow reaches `fight.rs::creature_dies`, which asks `World::crystallise` (`world/crystal.rs`)
  before anything else of the body: a type with a corpse (`Content::corpse_of`), no animated
  thing, and a colour (`Crystals::colour_of`) leaves `yield_at(level)` crystals of its colour (no
  draw), no corpse, and no `drops_of` stock (as a statue leaves none); a creature tagged `dragon`
  also `[rare] dragon` (no draw), any other of `[rare] level` or more one `chance(chance)` draw
  and, made, an `index` among `materials`, all from the gameplay stream. Its gear falls as any
  dead creature's; a summoned or risen one leaves nothing, a statue's death is the statue's
  (stone first), and no cursed room raises glass. Anything else dies as it would.
- **Words**, where the player sees the cell: "Its body turns to glass: where it fell lies a green
  crystal." / "… lie 3 red crystals and a heart-crystal." (`object::phrase`), after the death
  line.
- **Not built**: the design's creature `crystal = "…"` override (no creature needs one).

## Tests

`world/enchant_tests.rs` (the band; the three recipes' data; Kindle of every colour on many
fates holding `Magic::check` and the colour rule; a failure and a warp; Quicken's lacking kind;
Unbind made and failed; a stack of arrows at once; the reach and a deeper item's level; the
steps, the plan, and the rows' lacks; every refusal with no time; the hour and the mark; a
replay). `content/crystal_tests.rs` (the owner's order and two rules, the yield, every affix's colour and
word, every creature's colour by its first tags, the loader's refusals), `world/crystal_tests.rs`
(consuming through `eat`: mana to the maximum, the marks, the step and its diminishing, from the
floor, the rare materials refused with no time, a replay; salvage: a crystal per affix by its
check, unidentified magic's the same by the same fate with nothing named, Enchanting's mark, the rare materials' chance at item level 10 and none
below), `world/soulglass_tests.rs` (a kobold's green crystal and nothing else, Leech's corpse,
the yield by level and the gorgon's heart-crystal, a creature of no colour dying as ever, the rare
material's chance from level 15 and none below, the witch's grant at 16).

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

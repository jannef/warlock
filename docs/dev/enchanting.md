# Enchanting

Developer notes for working on this part of Warlock; the approved design is
`docs/design/enchanting.md` (#1414), built in phases E1 to E6 (#1427 to #1432). Keep it current
with the code, as CLAUDE.md says. All six are built: affix capacity and the trim; the
crystals, consuming one, and salvage's affix checks; the Soulglass spell; the profession with
its six recipes; and the seals and the rare materials (E6).

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
are gone while damaged, and a repair does not bring them back). Nothing strips it; the recipes refuse
to add to it ("It holds all it can.").

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
  `gear_condition::apply` (its own `condition/<key>` stream), all three in `gear::equip`, the one
  place gear's last steps are written (#1494), which `gear::arm` and
  `world/travel.rs::placed_gear` call; the balance probe writes them out itself. So a Fine piece keeps a second affix and
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
  vault loot) draws its make with `magic/make.rs::make` after the wand's rung and
  before the trim, from the loot stream: one draw, then one per improvement's aspect; none where
  the row gives nothing (item levels 1 and 2 have no row) or for what takes no make (not single
  arms or armour with an aspect). Creature gear adds the row of its level's item level to its
  tribe's odds (`Odds::deeper`: damaged gives way to what the improvements leave), in the same one
  condition draw. The loader keeps every row's improvements with any table's within
  1000. Kits, the background's gifts, the stipend, the pedlar, and animated things (`loot::make`,
  `make_as` with `Source::Gear`) take none.
- **Gems**: the row's `ornament` adds percent to loot's `[[ornament]]` chance (`gem::ornament`'s
  `more`), the same one draw.
- **Materials**: in the material files' `rarity` (and `gear`): mithril from item level 8,
  `deeper = 8`; dragonbone and dragonscale from 6, `deeper = 5`; adamantium from 16, `deeper = 6`;
  gold `deeper = 3`. The draw itself is `variant::draw`'s one draw.

Shallow levels have no row below item level 3, and the material weights at item level 1 are
the base ones. Deeper, the make's draws come after the item's other draws on the loot stream.

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
weapon, #1444, its 2 to 4 affixes never trimmed). With the base at 2 and quality +0/+1/+2
(#1444) every one fits; Stonebiter and the trap hall's dagger (two affixes each) need no make
and are plain.

## What the player sees

The item description (`magic/describe.rs::appraised_for`, through `Appraisal::capacity`) adds one
line after the affixes, before a unique's power and lore, only for a holder of Eye for the make
(the craftsperson's talent, `CraftMods::eye_for_make`) or a character trained in Appraise (any
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
- `[[family]]`: every affix category once with the words a seal names it by
  (`Crystals::families`, `family_word`); a seal's pick is its index here.
- `[yield] levels = [10, 20]`: `yield_at(level)`, Soulglass's count (E3): one, plus one per level
  reached.
- `[rare]`: `materials` (`heart_crystal`, `soul_fetter`: class `materials`, 30 g, value 300, not
  crystals), Soulglass's `level`, `chance`, `tag` (`"dragon"`) and `dragon` (the material a creature
  of that tag always gives, E3), salvage's `salvage_level = 10`,
  `salvage_chance = 3`, and which the recipes take: `heart` (a rare's Quicken) and `fetter`
  (Recast's affix kept), each one of `materials` (`RareMaterials::{heart, fetter}`; the loader
  refuses otherwise).
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

`data/rules/enchanting.toml` (`EnchantingRules`, `Content::enchanting`): `skill = "enchanting"` (the
skill of every enchanting check's `craft` points, the reach, and salvage's affix mark; the loader
refuses an unknown one), `stat = "willpower"` (every enchanting check's stat; #1493),
`salvage_difficulty = 12`,
`salvage_tier_step = 3` (`salvage_against(affix)` = 12 + tier ÷ 3, a tier-12 affix +4),
`level_step = 3` (the recipes' check, below) and `seal_seconds = 1800` (a seal's time, below).

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
`stat`'s modifier (Willpower) + `VigourMods::check(stat)` + `skill_effect_of(skill, Craft)`
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

E4 (#1430) and E5 (#1431; design §7, §8). Enchanting is a profession: `data/recipes/enchanting/`
holds `kindle.toml`, `quicken.toml`, `recast.toml`, `imbue.toml`, `ennoble.toml`, and
`unbind.toml` (rows in file-name order), so its row "Enchanting…" is in the abilities
menu for the trained, and `data/recipes.toml`'s `[[profession]]` gives it "Your work as an
enchanter.", `stat = "willpower"`, `verb = "Enchant"`. `data/skills/` gives Enchanting
the professions' `craft` curve (0, 8 at 100, 12 at 200; salvage's affix checks read it too), its
reach `enchant_level` (`Effect::EnchantLevel`, `[[0, 1], [110, 12], [200, 12]]`, at most
`MAX_ENCHANT_LEVEL` 12), and a `craft` mark at 100 %.

- **The data** (`content/recipe.rs`): `use = "enchant"` with `enchant = "kindle" | "quicken" |
  "recast" | "imbue" | "ennoble" | "unbind"` (`RecipeUse::Enchant(EnchantKind)`) and `crystals` (`RecipeDef::crystals`, 0 to
  `MOST_CRYSTALS` 3); `difficulty` and `seconds` required as Cut's and Set's (`checked()`). The
  loader refuses an `enchant` kind on another `use` or missing on `enchant`, a kind twice,
  `crystals` above 3 or on another `use`, a kind that rolls (all but Unbind) with none, and
  Unbind with any. Imbue's file gives 3.
- **The rules** (`world/enchant.rs`; `Spent::Enchant(Enchanting)`, dispatched by one line each
  from `World::start_craft`, `craft::lack`, `craft::takes`, `craft_make::craft_steps`, and
  `craft_plan`): `craft:kindle:ITEM,CRYSTAL`, `craft:quicken:ITEM,CRYSTAL`, `craft:recast:ITEM,CRYSTAL`,
  `craft:imbue:ITEM,C1,C2,C3`, `craft:ennoble:ITEM,CRYSTAL`, `craft:unbind:ITEM`.
  The wizard's steps are the item (`CraftStepKind::Item`: the stacks `refusal` passes) and a
  crystal for each the recipe takes (`CraftStepKind::Input`, "crystal", Imbue's "first crystal",
  "second crystal", "third crystal": every crystal carried, any colour; one stack may be picked
  for several, `resolve` refusing more picks of it than it holds: "You do not carry so many
  crystals."). The plan's verb is the recipe's name ("Kindle"), its name "the long sword", its
  uses the crystals in pick order ("a red crystal"; "a red crystal, a red crystal, and a blue
  crystal"; nothing for Unbind, whose dialog says only its time).
- **Refused, no time**, in order: untrained (`World::untrained`), below `from`, no item pick, more
  picks than the work takes, no crystal pick, then `refusal`: no such stack, not a base item or a
  spirit weapon ("No magic can be bound in that."; ammunition is a base item, the whole stack
  worked at once at the single cost), magic unidentified ("You must know what is bound in it
  first."), worn or wielded ("Take it off first."), a unique ("It will not take your will.");
  Kindle on magic ("There is magic in it already."), Quicken or Unbind on a plain item ("There is
  no magic in it."), Quicken on a rare ("It is rare already: this work cannot add to it.": a
  rare's Quicken takes the heart-crystal: below), Quicken on two affixes or at capacity ("It holds
  all it can."); Recast on a plain item ("There is no magic in it.") or a rare ("It is rare:
  unbind it, then imbue it anew."); Imbue on magic ("There is magic in it already.") or an item
  of capacity below 3 ("It would not hold so much."); Ennoble on a plain item, a rare (Quicken's
  words), capacity below 3 ("It would not hold so much."), or at capacity ("It holds all it
  can."); then the crystal pick that is no crystal ("That is no crystal."), nothing of
  its colour's families left to roll in the kinds the work may add ("No more of that nature will
  take."; Recast: for each kind the item has, on an empty item; Imbue: each crystal's families
  for either kind, on the plain item), a quest's object ("It will not take your will."), and unpaid goods (`may_loose`).
- **The rows** (`enchant::lack`, after the skill): `RecipeLack::Nothing` ("nothing to enchant":
  no stack the work may take), `Room` ("it holds all it can": Quicken's, every refused stack
  short only of room or capacity: Quicken's, Imbue's, Ennoble's), `Crystal` ("no crystal to hand":
  fewer crystals carried, all stacks counted, than the recipe takes). `takes`: "a plain item, a crystal, about
  1 hour", "a magic item, a crystal, about 1 hour", "a magic item, about 30 minutes".
- **The roll's item level**: the reach is `skill_effect_of(enchanting, EnchantLevel)` clamped
  to 1..12; Kindle and Imbue roll at the reach, Quicken, Recast, and Ennoble at the greater of the item's `Magic.level` and the
  reach, Unbind's check reads the item's own. The made item's `Magic.level` is the roll's.
- **The check** (`World::enchanted`, after the work driver ran the recipe's `seconds`, Quick hands' share
  of them, `mastery::craft_seconds`, as the plan and the row say; an interruption
  spends nothing but the time): `combat::roll_natural` (two gameplay draws) + `stat`'s modifier
  (`data/rules/enchanting.toml`, Willpower) + `VigourMods::check(stat)` + `skill_effect_of(recipe's
  skill, Craft)` against
  `difficulty` + level ÷ `level_step`; `enchant_band`: a natural 2 `Warped`, a natural 24 or a
  margin of 0 or more `Made`, else `Failed`.
- **Made** (`World::enchant_out`, `worked`): Kindle draws the kind by `[magic]`'s prefix and suffix
  weights among the kinds with a candidate (`magic::loot::pick_weighted`, one draw), then
  `roll_affix_within` (affix, tier, value: three draws) with the crystal's `families`; the item
  gets a `Magic` of rarity magic, the roll's level, identified. Quicken rolls the kind the item
  lacks (a prefix goes after the last prefix, a suffix last: `place`), the level the roll's.
  Recast rolls one affix for each the item had, of the same kinds in order, with the crystal's
  families, the old ones gone first (none can crowd another out: a magic item holds one of each
  kind); rarity and the lack of a name kept. Imbue (`imbued`) draws the count from `[rare]
  affixes` (`pick_weighted`), held to the capacity, then the split (`loot::rare_split`), then
  each affix: the n-th of the n-th crystal's families, past the crystals of all their families
  together; a prefix while the split has one left and the families can roll one, else a suffix
  (a slot neither can fill is skipped); then the name (`loot::rare_name`); a fresh rare `Magic`.
  Ennoble draws the kind evenly (`Rng::index`) among the kinds below `MAX_PER_KIND` that can
  roll, the affix, then the name; the rarity becomes rare. Unbind
  draws nothing and clears the `Magic` (`Stack::set_magic(None)`): its `Worked`, parts, rung,
  and charges stay. "The long sword takes the red crystal's glow. It is now a Sharp long sword.",
  "The magic goes out of Grim Bite. It is now a long sword.", "The long sword drinks the three
  crystals and wakes as Grim Bite.", "The Sharp long sword takes the red crystal's glow and wakes
  as Storm Hunger." (Ennoble). `Magic::check` holds on every result (the tests). The item is changed in place,
  a whole stack at once.
- **Failed**: the crystals are spent ("The crystal goes dark in your hands.", "The crystals go
  dark in your hands."; `World::spend_crystals`: one from each pick, a stack picked twice gives
  two), the item as it was;
  Unbind's "The Sharp long sword clings to its magic." **Warped** (a natural 2): the crystal is
  spent and `World::harm_item(item, ItemHarm::Enchant)` on one of the item set apart
  (`set_apart`: of a stack of arrows, one alone warps, #1487; "warped", withstood by no material;
  an item of Proof is spared as protected: "The work goes wrong, but no harm is done."): "The
  long sword warps under your hands.", or, warped already, destroyed ("... warps and falls
  apart.").
- **`roll_affix_within`** (`magic/loot.rs`): `roll_affix` is its call with every family; the
  candidates (`candidates`, sorted by affix id) are narrowed to the categories `families` holds.
  `rolls_within` says whether there is any, drawing nothing.
- **Marks**: `Use::Craft` on Enchanting, whatever came of it. No noise, no alignment.
- **The wiki**: `wiki/src/enchanting.md` and the generated `recipes-enchanting.md`
  (`xtask/src/wiki/enchanting.rs`: each recipe's crystals, time, listed from, difficulty, and the
  odds of a made check at three skill values, from `enchant_band`).
- **Snapshots**: `profession-enchanting`, `enchant-pick-item`, `enchant-dialog`
  (`crates/warlock/src/ui/screen_snapshots.rs`).

## Seals and the rare materials

E6 (#1432; design §9; `world/seal.rs`, its tests `world/seal_tests.rs`).

- **The picks** (`seal::slots`): the item, a crystal for each the recipe takes, then three seal
  slots (none for Unbind), then Recast's affix kept. A seal's pick is a family's index in
  `[[family]]`, the affix kept's its index among the item's affixes (`magic.affixes`, prefixes
  first); trailing picks may be left off, `-` skips one: `craft:kindle:a,b,4,17`,
  `craft:recast:a,b,-,-,-,0`. More picks than the slots: "That is more than the work takes."
- **A seal** (`seal::check`, in `resolve` after the crystals): its family must be of the colour
  of a crystal picked (Imbue: of any; "The red crystal holds nothing of mana.", "The crystals
  hold nothing of …"), not sealed twice ("You have sealed that already."), one the item could
  take at the roll's level of either kind and does not keep (`seal::sealable`; Recast keeps only
  its kept affix; "There is nothing to seal there."), and paid: one crystal more from the first
  crystal picked whose colour holds the family with one to spare after the picks and the seals
  before it ("You carry too few crystals for the seals."); an index past the list: "There is no
  such seal." The sealed categories are taken out of every crystal's families before the kinds
  and "No more of that nature will take." are decided, so the roll never draws them. Each seal
  adds `seal_seconds` (`enchant::work_seconds`, then Quick hands' share). No state is left on the
  item.
- **The heart-crystal** (`[rare] heart`): Quicken on a rare (`refusal` lets it through below its
  capacity and with a kind below `MAX_PER_KIND`) takes the first heart-crystal carried ("A rare
  takes a heart-crystal beside the crystal, and you have none."); it draws its kind evenly among
  the kinds with room (`Rng::index`, as Ennoble) and keeps the rarity and the name. "Grim Bite
  takes the red crystal's glow and the heart-crystal's, and holds more." A magic item's Quicken
  takes none. Ennoble on a rare: "It is rare already: only a heart-crystal's Quicken adds to it."
- **The soul-fetter** (`[rare] fetter`): Recast with an affix kept takes the first soul-fetter
  carried ("You have no soul-fetter to keep it with."; an index past the item's affixes: "There
  is no such magic in it."; keeping the only one: "Nothing would be left to recast."); the kept
  affix stays exactly (tier and value), the others roll anew of their kinds with the kept one's
  category out, each placed by kind (`place`). "The soul-fetter holds one enchantment fast: of the
  Bear."
- **Spent**: the seals' crystals, the heart-crystal, and the fetter are spent with the crystals,
  made, failed, or warped (`Resolved::all_spent`; "The heart-crystal goes dark with it.", "The
  soul-fetter crumbles to nothing." when the work does not take).
- **The wizard** (`enchant::steps` with the picks so far; `PerceptionView::craft_steps_given`):
  after the crystals, three optional seal steps (`CraftStepKind::Family`: "seal", "second seal",
  "third seal"; an option's `at` the family's index, its name the family's word, sorted by word),
  listing the families of the crystals picked (of every crystal carried before one is), less
  those sealed in the steps before and, with the item picked, those it could not take; then, for
  Recast while a soul-fetter is carried, the optional "affix kept" (`CraftStepKind::Affix`, the
  item's affixes, "Sharp: +2 damage"). The screen (`ui/craft_pick_screen.rs`) reads the steps again
  after each pick (`CraftPickScreen::reread`, called from `input::route_keys` through
  `ScreenStack::reread` while `is_stale`), lists a seal step's families under one "Families"
  header with "a crystal more", and a seal's None skips the seals after it (Esc skips back over
  them).
- **The dialog** (`enchant::uses_words`): "Kindle the long sword? It uses a red crystal and two
  more red crystals, sealing out fire and the critical, and takes about 2 hours."; "… a red crystal
  and a heart-crystal …"; "… a red crystal and a soul-fetter (keeping Sharp) …".
- **The wiki**: `wiki/src/enchanting.md` "Seals" and "The rare materials", with
  `generated/enchanting-seals.md` (`xtask/src/wiki/enchanting.rs`: the seal's cost and time, the
  most seals, the materials' sources and chances from `data/crystals.toml`).

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
  before anything else of the body: a type with a corpse (`Content::corpse_of`), not
  `everlasting` (a layer boss always leaves its corpse, #1624), no animated thing, and a colour (`Crystals::colour_of`) leaves `yield_at(level)` crystals of its colour (no
  draw), no corpse, and no `drops_of` stock (as a statue leaves none); a creature tagged `[rare] tag`
  (`dragon`) also `[rare] dragon` (no draw), any other of `[rare] level` or more one `chance(chance)` draw
  and, made, an `index` among `materials`, all from the gameplay stream. Its gear falls as any
  dead creature's; a summoned or risen one leaves nothing, a statue's death is the statue's
  (stone first), and no cursed room raises glass. Anything else dies as it would.
- **Words**, where the player sees the cell: "Its body turns to glass: where it fell lies a green
  crystal." / "… lie 3 red crystals and a heart-crystal." (`object::phrase`), after the death
  line.
- **Not built**: the design's creature `crystal = "…"` override (no creature needs one).

## Tests

`world/seal_tests.rs` (a seal shuts its family on many fates; each costs a crystal of the colour
and half an hour; every seal refusal with no time; the seal steps narrowed by the crystal, the item,
and the seals before; a rare's Quicken with and without a heart-crystal, made, failed, and at
capacity; Recast keeping an affix with a soul-fetter, its refusals, the affix step and the plan's
words, a failure spending the fetter; Imbue's seals paid by the first crystal of their colour; a
replay). `world/enchant_tests.rs` (the band; the three recipes' data; Kindle of every colour on many
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
adamantium's binding to +1 (#1444).

| Item level | Rares: capacity 2 | Rares keep 3+ | Rares untrimmed | Magic-grade untrimmed |
|---:|---:|---:|---:|---:|
| 1 | 85.5 % | 11.3 % | 8.4 % | 87.5 % |
| 4 | 84.2 % | 13.9 % | 8.8 % | 88.0 % |
| 8 | 81.0 % | 17.4 % | 10.7 % | 88.5 % |
| 12 | 72.8 % | 26.1 % | 16.8 % | 87.8 % |
| 16 | 64.9 % | 33.7 % | 21.4 % | 89.8 % |
| 20 | 57.5 % | 41.0 % | 26.1 % | 89.8 % |

The test fails if rares at item levels 1 and 4 keep three or more a fifth of the time or more, at
16 and 20 a quarter of the time or less, or if the share falls from one level
to the next. Magic-grade finds (one or two affixes) are untrimmed about 88 % to 90 % of the time at
every level.

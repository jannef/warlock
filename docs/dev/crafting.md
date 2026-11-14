# Crafting: professions, recipes, salvage

Spec §8 "Skills" (professions); design `docs/design/professions.md` (§6 the recipe format, §8 the
menus) and `docs/design/recipes.md` (§4 the grades, the format's second half, P4b); epic #1337.
This topic covers what is built: the recipe data, the profession rows of the abilities menu, the
profession screen, `Action::Craft`, who works which material, making a generic recipe (the
forge, P5), the forging wizard, and salvage. Material stock (the piece) is in
`docs/dev/objects.md`.

## Who works what: professions and stock

- **The skills** (`data/skills/`): Metalcraft, Woodcraft, and Jewelcraft are the professions
  that work materials today; Alchemy is a profession that works no material (it brews from
  herbs, "Brewing" below), Enchanting a profession that binds magic into what is carried
  (Willpower, its recipes built in: docs/dev/enchanting.md "The recipes"), and Herbalism and
  Mining are skills with no craft yet (#1331). Each of the three declares `{ effect = "craft", curve = [[0, 0], [100, 8], [200, 12]] }`
  (Temper's curve, read by salvage and by a recipe made) and a `salvage` mark rule at 100 %
  (Metalcraft, Woodcraft, and Jewelcraft also a `craft` mark, for a recipe made or a gem cut). Temper's and Whittle's own
  effects (`Effect::Temper`, `Whittle`) are unchanged; Fletch's (`Effect::Fletch`, `Use::Fletch`)
  went with the action in #1336: the arrows read Woodcraft's `craft`, the same curve.
- **Who works what** is data, never code: `data/stock.toml`'s `[[group]]` rows name the skill,
  stat, and verb per material group, and `gems = { ... }` the gems' (no group). Today metal is
  Metalcraft (Strength); wood, bone, hide, and sinew are Woodcraft (Dexterity); stone, resin,
  and the gems are Jewelcraft (Dexterity), but `[[material]]` rows give flint, stone, and
  obsidian (knapped) to Woodcraft; cloth and glass have none. `World::profession_of(material)`
  (`world/stock.rs`) is the one read. The stock itself (the `piece` object, by weight; the form
  words; `piece::count_for`; `MaterialDef::work`) is in `docs/dev/objects.md` "Stock".
- **`Effect::Craft` is read per skill**: `Actor::skill_effect_of(content, skill, Effect::Craft)`
  (`world/actor.rs`) reads the one named skill's curve at its value plus what affixes and talents
  add to that skill (`Actor::skill_bonus`, as `skill_effect` counts them); it never sums another
  skill's craft, which `skill_effect` would (Metalcraft's and Woodcraft's `craft` together on one
  check). The magic tests' exhaustive `skill_reader` names `world/salvage.rs` as its reader.

## Recipes as data

`assets/data/recipes/<skill>/<id>.toml`, one `[[recipe]]` a file; the directory is the
profession (a skill with a recipe directory is a profession: Alchemy, Enchanting, Jewelcraft,
Metalcraft, and Woodcraft). `content/recipe.rs`: `RecipeDef`, `RecipeId` (an index
in path order, never saved: actions name recipes by id), `RecipeUse`, `PartSource`, `RecipePart`,
`RecipeInput`, `RecipeResult`, and `Recipes` (`Content::recipes()`: the list, `professions()` the
skills with a directory in skill order, `of(skill)`, `of_use(uses)` (the recipe running a
built-in: the Cut list's), `id(name)`, `description(skill)`). Loaded
after `data/stock.toml` (a stock part's materials must be the profession's) through
`ContentFiles::table("recipes")`, so campaigns' `data/recipes/<skill>/` join in.

`data/recipes.toml` is the settings file the professions share: each profession's line in the
abilities menu (`[[profession]] skill, description`, "Your recipes as a smith."; a profession
without a row shows its skill's description), with the optional `stat` and `verb` (the check's
stat and the dialog's verb when no chosen stock names them: Alchemy's Intelligence and "Brew")
and `failed` and `fumbled` (the lines said when a `failure` or `fumble` object is made; refused
when empty) on `ProfessionText` (`Recipes::profession`), and the bands (`RecipeSettings`,
`Recipes::settings`, `None` only for content without the file), all required, struct
`deny_unknown_fields`:

| Setting | Value | Meaning (read from P5 on) |
|---|---|---|
| `fine`, `marvellous`, `masterwork` | 6, 12, 20 | the check's margin from which a made item is Fine, Marvellous, a Masterwork (recipes.md §4) |
| `cut_step` | 5 | a gem's cut rises a grade every step of margin (§5) |
| `half_within`, `one_within` | 3, 6 | how far below the difficulty a counted recipe still makes half its count, one ("Counted recipes" below) |
| `yield` | 50 | the percent of the stock and object inputs a failure gives back (`yield_percent`) |
| `free_wood` | forest, dense forest, swamp | `Landscape`s where wood is free (an arrow's shaft takes no billet) |

| Field | Meaning |
|---|---|
| `id` | the file name; `craft:ID` names it |
| `name` | the row's name (default: the result object's, capitalized, or the use's) |
| `use` | a **built-in** recipe: `temper`, `whittle`, `cut`, `set` (`RecipeUse`); its row runs that existing action with its own list and rules. Cut and Set (Jewelcraft's, `RecipeUse::checked`) give their check's `difficulty` and `seconds` in their file; Cut is "Cutting a gem" below, Set "Setting a gem" |
| `verb` | the dialog's verb where it is not the profession's: `"Make"` on Jewelcraft's jewellery, whose profession says "Cut". A recipe's `verb` comes before its profession row's in `data/recipes.toml` (Alchemy's "Brew"), which comes before the stock's (`stock.toml`), else "Make" (`craft_make::profession_verb`); Set's dialog says its name, "Set" |
| `result` | `{ object, count }`: a **generic** recipe's result; `count` dice (`RecipeCount::Dice`) or `"fletch"` (`RecipeCount::Fletch`, the arrows' count: "Arrows and bolts" below); above 1 or `"fletch"` makes it **counted** |
| `[recipe.parts]` | `main` is the first part; sorted into the result's part order. `blade = "stock"` (`PartSource::Stock`: the profession's stock), a **table of sources** `head = { input = "arrowhead", stock = ["bone", "stone"], optional = true }` (`PartSource::Sources`: the wizard's one step offers the input's stacks, then stock of the listed groups or materials of any profession, then None if `optional`; `stock` holds the materials of the part's `made_of` the list names, in `made_of` order; a name both a group and a material, `stone`, is the group), or `setting = { gem = true }` (`PartSource::Gem`: a cut loose gem, required) |
| `inputs` | `[{ object, count, per_made }]`, or fixed stock: `{ material = "sinew", count }`, `{ group = "metal", count }` (`InputKind`; `RecipeInput::object()` for an object input) |
| `failure`, `fumble` | `{ object }`: what a failed check, a natural 2, makes instead of nothing |
| `brittle` | the result when an `optional` part is None |
| `ornament` | an optional gem for the result's ornament part |
| `difficulty`, `seconds` | a generic recipe's (and Cut's and Set's) check base and time |
| `station` | a terrain id that must stand beside the maker |
| `knife`, `launcher` | a knife carried; a launcher ready that shoots the result (its `ammo`) |
| `from` | the trained value it is listed from (default 0) |

Checked at load (`recipe::parse`; refusals tested in `content/recipe_tests.rs`): a file in a
skill's directory (the directory a known skill), one `[[recipe]]` a file (a file with none, a
README, is skipped), the id its file name and unique across professions; a `use` among the five
and with no result, parts, inputs, or ornament, and a `difficulty` and `seconds` of 1 or more for
Cut and Set, none for the others; a generic recipe with a
result object, a count that makes at least one and above 1 only on a stackable object, a
`difficulty`, and `seconds` of 1 or more; every part on the result, a stock part whose `made_of`
holds a material of the profession (`Stock::profession_of`), an input part naming an input, a
part source of a known shape; inputs that are objects, counts above 0; an ornament only on a
result with an ornament part (`ObjectDef::ornament` or its class's `[[ornament]]`); a station
that is terrain. The settings file: known skills, each once, non-empty descriptions.

The format's second half (#1350, `docs/design/recipes.md` §10) adds these checks, each with a
refusal test:

| Field | Refused when |
|---|---|
| a part's table | it names no source; `gem` with another source or `optional`; an `input` none of the recipe's object inputs, or one whose own material (`ObjectDef::material`) the part's `made_of` does not hold; a `stock` list empty, naming neither a group some material has nor a material, a material not on the part's `made_of`, or a group none of whose materials is (a recipe narrows, never widens) |
| `gem = true` | the part's `made_of` holds no gem material |
| `inputs` | not exactly one of `object`, `material`, `group`; an unknown object or material; a group no material has, or a gem's id (gems have no group); a count of 0 |
| `count` | neither dice nor `"fletch"`; above 1 (or `fletch`) on an object that does not stack |
| `failure`, `fumble` | an unknown object, or one that does not stack |
| `brittle` | an unknown object; a recipe with no `optional` part. An `optional` part with no `brittle` is refused too: no object part can be absent today, so "a part the result can lack" has no case yet |
| `use` | not one of the five; with a result, parts, inputs, ornament, difficulty, seconds, failure, fumble, or brittle |
| settings | `fine` not above 0, `marvellous` not above `fine`, `masterwork` not above `marvellous`, `cut_step` not above 0, `half_within` below 0, `one_within` not above `half_within`, `yield` above 100, a `free_wood` landscape twice or unknown, a band missing |

A table of `stock` alone (no `input`, not `optional`) is used from P5b (#1351: the spears'
hafts, the hand recipes' narrowed lists; `craft::stock_only`, "Routes and the recipe's
profession" below). The rest is used by the phase that makes the first recipe needing it, which
also builds its wizard step and its row's words and lack: the head step (a table of sources
with an input), `optional`, `brittle`, `per_made`, and counted results with P6 (#1336, the
arrows: "Arrows and bolts" below), stock inputs with P7 (sinew, iron; built, "Stock inputs" below), `gem = true` with P11,
`failure` and `fumble` with P13 (brewing, "Brewing" below). The bands and `yield` are read from
P5 (the forge, "Making a recipe" below). Until then the rows and the wizard read `"stock"`
parts, parts with sources, object inputs, and the ornament. `data/stock.toml`'s `unit` and
`[[material]]` overrides (P2b) are checked by `content/stock.rs`.

Every recipe is its skill's (#1387, the owner): the untrained see none and make none. The
built-ins are `recipes/metalcraft/temper.toml` (`station = "anvil"`) and
`recipes/woodcraft/whittle.toml` (`knife`), Jewelcraft's `recipes/jewelcraft/cut.toml`
(`difficulty = 12`, `seconds = 1800`, `from = 0`) the third, and `recipes/jewelcraft/set.toml` (`difficulty = 12`,
`seconds = 1200`, `from = 10`) the fourth.

The generic recipes (P5, #1335; `docs/design/recipes.md` §6 "Melee weapons", the metal route):
the seven swords under `recipes/metalcraft/`, each `station = "anvil"`, its main part (the long
sword's `blade`) `"stock"`, an `ornament` (the pommel) on all but the knife. The stock is priced
by the result's weight (`piece::count_for`), never written in the file.

| Recipe | `difficulty` | `seconds` | `from` | Steel ingots |
|---|---|---|---|---|
| knife | 10 | 1200 | 0 | 1 |
| dagger | 11 | 1800 | 0 | 1 |
| short_sword | 12 | 3600 | 10 | 2 |
| scimitar | 13 | 3600 | 20 | 3 |
| long_sword | 13 | 3600 | 20 | 3 (its leather grip free) |
| rapier | 14 | 3600 | 30 | 2 |
| great_sword | 15 | 5400 | 40 | 7 |

## Routes and the recipe's profession

P5b (#1351; `docs/design/recipes.md` §6 "Melee weapons", §2 rows 1 and 4) adds the rest of the
melee weapons, one file each, the columns the design's table (the stock is again priced by
weight, never written):

| Recipe | Profession | Parts | `difficulty` | `seconds` | `from` | Stock |
|---|---|---|---|---|---|---|
| stiletto | Metalcraft | main `"stock"` | 11 | 1800 | 0 | 1 steel |
| hand_axe | Metalcraft | main `"stock"` | 11 | 1800 | 0 | 3 iron |
| battle_axe | Metalcraft | main `"stock"` | 13 | 3600 | 20 | 5 iron |
| great_axe | Metalcraft | main `"stock"` | 14 | 5400 | 30 | 10 iron |
| mace | Metalcraft | main `"stock"` | 11 | 3600 | 0 | 5 iron |
| morning_star | Metalcraft | main `"stock"` | 12 | 3600 | 10 | 6 iron |
| war_hammer | Metalcraft | main `"stock"` | 12 | 3600 | 10 | 5 iron |
| maul | Metalcraft | main `"stock"` | 13 | 5400 | 20 | 12 iron |
| spear | Metalcraft | head `"stock"`, haft `{ stock = ["wood"] }`; an inlay | 11 | 1800 | 0 | 1 iron, 7 wood |
| javelin | Metalcraft | head `"stock"`, haft `{ stock = ["wood"] }` | 10 | 1200 | 0 | 1 iron, 4 wood |
| trident | Metalcraft | main `"stock"` | 13 | 3600 | 20 | 4 iron |
| glaive | Metalcraft | main `"stock"`; an inlay | 14 | 5400 | 30 | 10 iron |
| halberd | Metalcraft | main `"stock"`; an inlay | 15 | 5400 | 40 | 11 iron |
| knife_by_hand | Woodcraft | main `{ stock = ["stone", "bone"] }` | 9 | 1200 | 0 | 1 |
| dagger_by_hand | Woodcraft | main `{ stock = ["stone", "bone"] }` | 10 | 1800 | 0 | 1 |
| hand_axe_by_hand | Woodcraft | main `{ stock = ["stone"] }` | 10 | 1800 | 0 | 3 |
| club | Woodcraft | main `{ stock = ["wood", "bone"] }`; an inlay | 8 | 1200 | 0 | 7 |
| quarterstaff | Woodcraft | main `{ stock = ["wood"] }`; a tip | 9 | 1800 | 0 | 10 |
| ironwood_staff | Woodcraft | main `{ stock = ["ironwood"] }`; a tip | 12 (+3 work) | 2700 | 30 | 7 ironwood |
| spear_by_hand | Woodcraft | head `{ stock = ["stone", "bone"] }`, haft `{ stock = ["wood"] }` | 10 | 1800 | 0 | 1, 7 wood |
| javelin_by_hand | Woodcraft | the same | 9 | 1200 | 0 | 1, 4 wood |

The smith's are at an anvil (`station = "anvil"`); the hand-made ones need a knife carried
(`knife = true`) and no station. Ids are unique across
professions, so a base with two routes has the smith's at its own id and the hand route at
`<base>_by_hand`; both rows are named by the result ("Knife"), each on its own profession's
screen. Never made, so with no file: a stiletto by hand, a battle axe or great axe of stone, a
sword of dragonbone. The `stone` and `bone` lists are groups (a name that is both is the group),
so a hand knife may also be of ivory or dragonbone (both `bone`); jade is no weapon material.

- **A table of stock** (`PartSource::Sources` with no `input`, not `optional`;
  `craft::stock_only`) is a step like `"stock"`: `craft::worked_materials` returns its resolved
  `stock` list (any profession's: the smith's spear takes Woodcraft's billets for its haft)
  instead of the profession's materials on the part's `made_of`. The rows (`lack`, `takes`) and
  the steps read it the same way. A table with an input or `optional` still has no steps (P6).
- **The recipe's profession is its directory's** (`craft_make::profession`, through
  `Stock::profession(skill)`: the first `stock.toml` row worked by the skill): the check's stat
  and the dialog's verb ("Make" for Woodcraft, "Forge" for Metalcraft), the `craft` points, and
  the mark (`Use::Craft` on `def.skill`). A flint knife is Woodcraft's with Dexterity, a steel
  knife Metalcraft's with Strength. A profession row in `data/recipes.toml` that names a `stat`
  or `verb` overrides the stock's (`craft_make::profession_stat`, `profession_verb`: Alchemy's
  Intelligence and "Brew", herbs being no stock; "Brewing" below); without one, a skill that
  works no stock falls back to the main material's profession, then Strength and "Make".
  `work` stays the main chosen part's material's (a spear's head, not its haft).
- **The row's words** (`craft::stock_words`) price a part once per material group it may be of,
  the base material's group first, each other group by its cheapest material (`value`: a bow
  "or 4 horn shards", never dragonbone): "7 wood billets or 7 bone shards", "a bone shard or a block of
  flint". The greyed reason names the first material's group ("no bone to hand"; the haft's "no
  wood to hand").
- **The wizard's label** for an unnamed main part is its stock's group when all of it is one
  group ("metal", "wood"), else "material".
- **A failure** not of metal says "The work goes wrong, and you save what you can of it." (the
  metal's line stays).
- Tests: `world/craft_hand_tests.rs` (the design's counts, the spear's steps, the haft refused
  and greyed, the knife, skill, and station greyings, the profession's stat, verb, and mark, a
  bone javelin made and still thrown).

## Hide gear

P8 (#1353; `docs/design/recipes.md` §6 "Shields", "Body armour", "Helmets", "Cloaks, gloves,
boots", "Clothing", the Woodcraft rows; the Metalcraft rows are P9's, #1354) adds Woodcraft's
hand-made armour and clothing, one file each under `recipes/woodcraft/`, every one `knife = true`,
no station, trained only, its main part a table of `stock`. Hide is Woodcraft's (`stock.toml`'s
`hide` row, `unit = 8`: a leather hide is 1.4 kg, eight pieces' worth), so a hide count comes out
of `piece::count_for` by weight like any other and is never written in a file. A base with a
smith's route too has its hand route at `<base>_by_hand` (the smith's file keeps `<base>` for
P9).

| Recipe | Main `stock` | `difficulty` | `seconds` | `from` | Stock |
|---|---|---|---|---|---|
| buckler_by_hand | wood, hide, chitin, dragonscale | 10 | 3600 | 0 | 3 billets, 1 hide, 3 shards |
| small_shield_by_hand | the same; an inlay | 10 | 3600 | 0 | 13 billets, 2 hides, 12 shards |
| large_shield_by_hand | the same; an inlay | 12 | 5400 | 20 | 30 billets, 4 hides |
| tower_shield_by_hand | the same | 14 | 7200 | 30 | 24 billets, 3 hides |
| leather_jerkin | hide | 11 | 3600 | 0 | 3 hides |
| studded_leather | hide; `inputs` 1 ingot of `metal` | 13 | 5400 | 20 | 5 hides, 1 ingot |
| scale_mail_by_hand | bone, dragonscale | 14 | 7200 | 30 | 28 shards, 4 hides |
| leather_cap_by_hand | hide, bone | 9 | 1800 | 0 | 1 hide, 3 shards |
| iron_helm_by_hand | hide, bone | 11 | 3600 | 10 | 1 hide, 3 shards |
| great_helm_by_hand | hide, bone | 13 | 5400 | 30 | 1 hide, 6 shards |
| fur_cloak, travelling_cloak | fur, leather | 10 | 3600 | 0 | 2 pelts; 1 hide |
| leather_gloves, leather_boots | hide | 9 | 1200, 1800 | 0 | 1 hide |
| hide_tunic, leather_apron | hide | 8 | 1800 | 0 | 1 hide |
| sandals, shoes, turnshoes, soft_boots | hide | 7 | 1200 | 0 | 1 hide |
| hobnailed_boots | hide; `inputs` 1 ingot of `metal` | 7 | 1200 | 0 | 1 hide, 1 ingot |
| clogs | wood | 7 | 1200 | 0 | 5 billets |

- **Counts by weight**: chitin is lighter than wood, so a small shield takes 12 shards where the
  design's table wrote 13; the rule (by weight) wins over the table.
- **Names**: the pairs and the suit are named without "pair of" ("Shoes", "Scale mail") by the
  file's `name`; the rest by the result.
- **Grades**: shields, armour, helmets, cloaks, gloves, and boots have an aspect and come out
  Fine to Masterwork by the bands; clothing has none and is plain at any margin (the existing
  rule, "An item with no aspect is plain").
- **The ingot inputs** (studded leather's studs, the hobnails) are a stock input
  (`{ group = "metal", count = 1 }`), asked for by P7's input step (below) after the parts, any
  metal's ingot fitting (`craft::input_fits`), and used up.
- Hide parts on metal items (a long sword's grip, iron-shod boots' leather) stay free
  (recipes.md §12 item 13). Never yet: the padded coat, the wool cloak, every cloth garment
  (tailoring), the reed kilt.
- Tests: `world/craft_hide_tests.rs` (each file's fields, the counts, a jerkin made from three
  hides, cloaks of pelts and of a hide, graded gear and plain clothing, the knife and skill
  greyings, studded leather and hobnailed boots made with an iron ingot).

## Stock inputs: bows, crossbows, the sling, the lute, the pipe

P7 (#1352; `docs/design/recipes.md` §6 "Missile weapons" and the lute and pipe of "Gems, coins,
miscellaneous", §3, §10) adds Woodcraft's missile weapons and two instruments, one file each under
`recipes/woodcraft/`, all with a knife carried and no station, trained only. No Metalcraft route:
nothing that shoots is a smith's.

| Recipe | Parts | Inputs | `difficulty` | `seconds` | `from` | Stock |
|---|---|---|---|---|---|---|
| short_bow | main `{ stock = ["wood", "horn", "dragonbone"] }` | 1 sinew | 11 | 3600 | 0 | 4 |
| long_bow | the same | 1 sinew | 13 | 3600 | 20 | 7 |
| composite_bow | main `{ stock = ["horn", "wood", "dragonbone"] }` | 2 sinew | 15 | 5400 | 40 | 5 |
| crossbow | main `{ stock = ["wood"] }` | `{ group = "metal", count = 1 }` | 14 | 5400 | 30 | 16 |
| heavy_crossbow | the same | the same | 16 | 7200 | 60 | 23 |
| sling | main `{ stock = ["hide"] }` | — | 8 | 600 | 0 | 1 leather hide |
| lute | main `{ stock = ["wood"] }` | 1 sinew | 14 | 7200 | 40 | 8 |
| pipe | main `{ stock = ["wood"] }` | — | 7 | 600 | 0 | 1 |

- **A stock input** (`InputKind::Material`, `InputKind::Group`; `craft::input_fits`, the one
  test of a stack against any input; an object input through `craft::known_object`, below) is a wizard step like an object input (`CraftStepKind::Input`,
  after the parts): it lists the pieces of the material ("sinew") or of any material of the group
  ("metal": every metal ingot carried, so the wizard asks which), each with the count, greyed "too
  few" on a stack with fewer. It is spent at the end like any stock, is never a part of what is
  made (a bow has no sinew part; `made_of` holds no sinew), and a failure gives back
  `count × yield / 100` of it (none of one length). It never sets the main material (`work`).
- **The row** says it in pieces: "a length of sinew", "2 lengths of sinew", "an ingot of any
  metal" (the group's form word from its first material, `stock.toml`). Its lack is
  `RecipeLack::Stock` of the material, or of the group's first material, when no one stack
  holds the count: "no sinew to hand", "no metal to hand" (a composite bow with one length too).
- **The composite bow** keeps its base material, wood, so its stock counts are the design's (5
  horn shards or 5 billets); the horn one is "a horn composite bow" (better: horn's `damage` is
  -1 to wood's -2) and the wooden one plain "a composite bow". The lute and the pipe have no
  `Aspect`, so they come out plain whatever the margin; the bows, crossbows, and sling take the
  bands.
- Tests: `world/craft_bow_tests.rs` (the design's counts, the rows' words, the sinew and metal
  steps, the greyings and refusals with no time, a long bow made with its sinew spent and no part
  of it, the ingot chosen spent, the composite bow's names, the plain lute).
## Armour and tools

P9 (#1354; `docs/design/recipes.md` §6 "Shields", "Body armour", "Helmets", "Cloaks, gloves,
boots", "Light sources and tools", the metal route, and §12 item 15, mail by weight) adds one file
each, every smith's `station = "anvil"`, main `"stock"`, the stock priced by the weight (never
written: chain mail is 11 kg, so 22 ingots; salvage gives 11 back). The smith's take the base id;
P8's hide routes (#1353) and the stone tools take `<base>_by_hand`.

| Recipe | Profession | `name` | Ornament | `difficulty` | `seconds` | `from` | Stock |
|---|---|---|---|---|---|---|---|
| chain_mail | Metalcraft | Chain mail | — | 14 | 7200 | 30 | 22 iron |
| scale_mail | Metalcraft | Scale mail | — | 14 | 7200 | 30 | 28 iron |
| banded_mail | Metalcraft | Banded mail | — | 15 | 10800 | 40 | 34 steel |
| plate_armour | Metalcraft | Plate armour | — | 17 | 14400 | 60 | 44 steel |
| buckler | Metalcraft | | inlay | 11 | 3600 | 0 | 3 iron |
| small_shield | Metalcraft | | — | 12 | 3600 | 10 | 13 (wooden base: steel) |
| large_shield | Metalcraft | | — | 13 | 5400 | 20 | 30 (wooden base) |
| tower_shield | Metalcraft | | inlay | 14 | 7200 | 30 | 24 iron |
| leather_cap | Metalcraft | | — | 11 | 1800 | 0 | 3 (leather base: "a steel cap") |
| iron_helm | Metalcraft | | studs | 12 | 3600 | 10 | 3 iron |
| great_helm | Metalcraft | | studs | 14 | 5400 | 30 | 6 steel |
| gauntlets | Metalcraft | Gauntlets | — | 12 | 3600 | 10 | 2 iron |
| iron_shod_boots | Metalcraft | Iron-shod boots | — | 12 | 3600 | 10 | 4 iron |
| lantern | Metalcraft | | — | 10 | 1800 | 0 | 1 iron or brass (its `made_of`) |
| pick_axe | Metalcraft | | — | 12 | 3600 | 10 | 6 iron |
| mattock | Metalcraft | | — | 13 | 3600 | 20 | 9 iron |
| pick_axe_by_hand | Woodcraft | | — | 11 | 3600 | 10 | 6, `{ stock = ["stone"] }`, a knife |
| mattock_by_hand | Woodcraft | | — | 12 | 3600 | 20 | 9, the same |

- **No new code but the words**: the generic recipe already handles them. Grades follow the
  base's aspects (`magic::aspects`): armour and shields improve in Defence and Protection; the
  lantern, pick-axe, and mattock (no attack, no Defence) come out plain.
- **A base of a material the recipe cannot take** (a wooden small shield forged in metal, a
  leather cap in metal) is priced in the piece object's own material (steel), never the first of
  the group in file order (adamantium): `craft::stock_words` and the wiki's
  `xtask/src/wiki/recipes.rs` both fall back base material, then the piece's, then the first.
  The count is the same whatever the metal (a piece weighs by its material).
- **Never made, so no file**: candles (wax has no profession), rope (cloth), the keys, household
  goods (salvage only).
- Tests: `world/craft_armour_tests.rs` (the design's counts, mithril and bronze too, the rows'
  words, the lantern of iron or brass only, the anvil, the stone pick's knife and no anvil, plate
  beyond skill below 60, a graded suit against a plain lantern and pick-axe, a helm forged and
  marked, salvage's half).

## Smelt

`recipes/metalcraft/smelt.toml` (#1389, `docs/design/professions.md` §9): Metalcraft's, the
trained's (no `anyone`), from 0, at an anvil, 600 s, difficulty 8 (+ the metal's `work`). Its
`result` is the `piece`, its one input a lump of `ore`, and its `main = { input = "ore" }` gives
the ingot the lump's metal (the input's material, as an arrow's head takes the arrowhead's): the
wizard's one step lists the ore stacks carried ("lumps of copper ore"). No new code: a graded
recipe whose result has no aspect, so plain; a failure gives back `1 × yield / 100`, none, and
says "The work goes wrong, and you save what you can of it." The ore comes from Mining
(`docs/dev/digging.md` "Mining"). Tests: `world/mine_tests.rs` (copper ore to a copper ingot, one
lump spent; not listed for the untrained).

## Stations and the makeshift anvil: `works_as`, Field forge

A recipe's `station` is a terrain beside the maker. A terrain may serve as another's station:
`TerrainDef::works_as` (`works_as = "anvil"` in `data/terrain/`, resolved at load; a terrain
that works as one that works as another is refused). It is read in one place,
`Content::works_as(terrain)` (its `works_as`, else itself), which both station tests ask:
`world/craft.rs::station_beside` (every recipe's `lack`, so `start_making` and the screen; the
"You set to work on the dagger at the makeshift anvil." line names the cell's own terrain) and
`Content::offer` (what a feature offers, so Temper's `anvil_beside`, `anvil_needs`, and the
walk-in, `ui/step_dialogs.rs::step_screen`, need no change). Mining finds nothing in a terrain that works as
a station (`world/mine.rs`).

The one such terrain is `makeshift_anvil`, which the craftsperson's **Field forge** (level 32,
`ClassActive::FieldForge`, `data/talents/field_forge.toml`'s `field_forge { seconds = 3600 }`,
`TalentMods::field_forge`) makes, in `world/field_forge.rs`: `use:field_forge:E`, aimed in a
direction (`Aim::Direction`; the abilities menu's way is `beside::Target::Ground`, open ground the
player knows: `world::open_ground` on the known terrain, nobody seen, no known trap, nothing
lying there). The rules: the overworld refused (the menu: `Unavailable::NotHere`); the first
stack of ingots carried (a `piece` of a `metal`-group material, not unpaid goods) else "You
have no ingot to set."; the cell open ground (`open_ground`: walkable, no door, chest, stairs,
grave, freezing water, ice, swim water, glowing fungus, or shop floor), nobody on it (ground
truth), no known trap, nothing lying there; each refused with its line and no time. Then
the work driver's `Task::FieldForge(Forging { at })`: `seconds` in `craft` turns, as a
gather; interrupted, nothing ("...; you leave the anvil unfinished."); done, the ingot and the
cell are asked again, one ingot is spent, and `set_terrain(at, makeshift_anvil)`. No check, no
draw, no mark. The terrain is saved with the level; a pick clears it (`hardness` 300, becoming
floor). Tests: `world/field_forge_tests.rs`.

## The rows: `Ability::Profession`

`ability.rs`: `Ability::Profession(SkillId)`, `Source::Skill`, named "Metalcraft…" (the skill's
name and an ellipsis), described by `data/recipes.toml`'s line, `Aim::Recipes`, no cost, never
unavailable. `Ability::every` lists it in its skill's place (skill order) for every skill with
a recipe directory, before `Ability::use_of` and `Ability::Skill`, so a profession has no "not
usable yet" row (Jewelcraft's went with Cut). `had_by`: trained in the skill (#1387: no recipe is
open to the untrained). The anvil keeps
`Offer::Temper`: it is the Temper recipe's station, and walking into it
(`ui/step_dialogs.rs::step_screen`, after `step_dialog` asks nothing) opens Metalcraft's profession screen,
`ScreenName::Profession(skill)` of Temper's recipe's skill, at once, with no dialog and no time,
for one who has that row (the owner, #1387). Untrained, the step goes to the rules:
`travel.rs::player_moves` refuses it, taking no time, with `World::anvil_needs`' "You need
Metalcraft to work at the anvil." in place of the plain "The anvil blocks your way."
(`Ability::offered` is `None` for it, so the Here group lists only Pray and Drink). Wedge stays
its own row (a door, not a recipe). `input.rs::use_ability` opens
`ScreenName::Profession(skill)` for `Aim::Recipes`.

## What the screen reads: `PerceptionView::recipes`

`world/craft.rs::recipe_rows` (through `PerceptionView::recipes(skill)`): the profession's
recipes in path order, every one for a player trained in the skill and none otherwise, each a `RecipeRow { id, name, takes, unavailable }`. Computed in core from the pack,
the cells beside the player, and the skills: presentation never decides availability.

- `takes`: what a built-in works on ("a metal weapon or armour", "a wooden weapon"), each stock
  part priced in pieces (`piece::count_for` of the part's share of the result's weight, named by
  `piece::names`: "3 steel ingots"), inputs ("20 arrowheads", "an arrowhead for each made"), the
  station ("an anvil"), "a knife", "a bow ready" (`data/fletch.toml`'s word for what shoots the
  result), then the time in `Duration::spoken` ("about 10 minutes": Temper's and Whittle's
  `data/temper.toml`, a generic recipe's `seconds`, the arrows' cut by Fletcher). A part with
  sources is its choices joined by "or" ("an arrowhead for each made or a bone shard or a block of
  flint"); a counted recipe's stock is priced for the batch at its most; wood free here is not
  said. Never a difficulty or odds.
- `unavailable`: the first `RecipeLack` that applies, in this order: `Skill` ("beyond your
  skill": the trained value below `from`), `Station` ("no anvil beside you"), `Knife` ("no
  knife"), for Set `set::lack`'s `Setting` ("nothing to set a gem in") and `Gem` ("no gem to
  hand"), for Enchanting's `enchant::lack`'s `Nothing` ("nothing to enchant"), `Room` ("it holds
  all it can"), and `Crystal` ("no crystal to hand"), `Launcher(result)` ("no bow ready", "no crossbow ready": `fletch::shoots`), `Stock`
  ("no metal to hand", "no wood to hand": no piece whose material fits a stock part, wood free on
  wooded ground for the arrows; a required part with sources wants its input or its stock),
  `Input` ("not enough arrowheads", "not enough sprigs of moonwort": too few carried; an input a
  part's sources name is that part's; a stack of a type the player has not identified is not
  counted, below), `Gem` for a recipe with a gem part and no cut gem
  carried (the gold amulet).
- `takes` names a gem part "a cut gem" ("a gold ingot, a cut gem, about 1 hour and 30
  minutes").

## The profession screen

`crates/warlock/src/ui/profession_screen.rs`, `ScreenName::Profession(SkillId)` (skipped by
`--screen`; `--screen profession` is `ScreenName::FirstProfession`, the first profession with a
recipe the player sees: Metalcraft for most, Alchemy for one trained in it). The name column
widens to the longest name ("Greater potion of presence"). A `PagedList` grouped by the result's item class, the built-in recipes first under
"Work" (group `None`). A row: the letter and name, then what it takes; a row that cannot be made
is greyed (`TEXT_DIM`) with its reason in place of what it takes, and the line under the list
gives the selected row's `takes` in full. A letter or Enter on the `>` row makes an available
recipe: Temper opens `ScreenName::Temper` over the screen, Whittle `ScreenName::Whittle`, a
generic recipe the wizard (`ui/craft_pick_screen.rs`, below); a greyed one does nothing (its
reason is on the row). The list keys page; Esc closes. Every key is fixed and named in the footer
("a-b or Enter make   Esc back"): no `KeyCommand`, no `Context` entry.

## The action: `Action::Craft`

`Action::Craft { recipe: String, picks: Vec<Option<usize>> }`, replay text `craft:ID[:POS,...]`
(positions as `position_text`, `-` a skipped optional: `craft:long_sword:c,f`,
`craft:dagger:a,-`), dispatched by one arm in `World::player_acts` to `World::start_craft`
(`world/craft.rs`). An unknown recipe is refused ("You know no such recipe."), and so is a
built-in one ("Temper is done from its own list.": its row runs `temper:` or `whittle:`), both
taking no time; a generic one goes to `World::start_making` ("Making a recipe"
below). Its turns are `ActionKind::Craft` (`data/actions.toml` `craft = 1000`;
`World::kind_of`).

## Making a recipe

`world/craft_make.rs` (P5, #1335; `docs/design/recipes.md` §4, `professions.md` §7), tests in
`world/craft_make_tests.rs`:

- **Steps** (`craft_steps`, `PerceptionView::craft_steps(recipe)`): one `CraftStep` per choice in
  the wizard's order: each chosen part in part order, a stock part (`"stock"`, or a table of
  `stock` alone, P5b; `CraftStepKind::Stock`, labelled by the part's name, "blade", or for an
  unnamed main part by its stock's group, "metal"; none for wood free here, "Arrows and bolts"),
  a part with a table of sources with an input or `optional` (`Sources`: the input's stacks
  whose material the part's `made_of` holds, then the pieces of its `stock` materials as
  carried, `optional` when None may be chosen), or a gem part (`Gem`, P11: "Jewellery" below),
  each object input no part's sources name
  (`Input`), then the ornament (`Ornament`, `optional`; the part is `ObjectDef::ornament` or the
  class's `[[ornament]]`). Each lists the stacks carried that fit, `CraftOption { at, name (as
  the inventory lists it), short (the material: "steel", "ruby"), takes, each (for every one
  made: "uses 1 each"), enough }`: pieces whose
  material the part's `made_of` holds and the profession works (`worked_materials`), stacks of
  the input's object or pieces of its material or group (`input_fits`), cut loose gems (never an uncut one, `Cut::rough`). A part takes its share
  of the result's weight scaled by the material's weight over the base part's (as the made item
  weighs it), in pieces (`piece::count_for`): a mithril blade takes as many mithril ingots as a
  steel one takes steel. A recipe with no part and no ornament (a brew) has no steps ("Brewing"
  below). A recipe with more than `MAX_PICKS` (8) steps has no
  steps either, and `craft:` refuses it ("You cannot make that yet.") until the phase that
  builds its step.
- **The plan** (`craft_plan`, `PerceptionView::craft_plan(recipe, picks)`; `resolve`): every
  required step picked ("You must choose the metal."), the stack there ("You carry no such
  thing."), fitting ("That will not do for the blade.", "Only a cut gem can be set."), holding
  what it takes, summed over steps that pick it twice ("You have too few of those."); more picks
  than steps are refused ("That is more than the work takes."). The made item is the result's
  object with own parts (`ItemPart::material` for a chosen material other than the base part's,
  `ItemPart::gem` for the ornament; `item_parts::check`); None on an `optional` part makes the
  recipe's `brittle` object instead, each of its parts of the first chosen material its
  `made_of` holds (the brittle arrow of the shaft's wood). `CraftPlan { verb, name, uses,
  station, time }`: the verb the recipe's profession's (`craft_make::profession_verb`: its
  `data/recipes.toml` row's where it names one, else `stock.toml`'s for the directory's skill, so
  a steel-headed arrow is Woodcraft's "Make"), the name `Lore::describe` of the made item ("a
  ruby-pommelled mithril long sword"; a counted recipe's plural, "flint arrows"), `uses` the
  pieces counted, "an arrowhead for each made", and "the ruby".
- **Start** (`World::start_making`): refused with no time by the row's `lack` (beyond skill: "That
  is beyond your skill."; no station: "There is no anvil here."; no knife; no launcher: "You
  have no bow ready."), then by the plan's reason. Then "You set to work on the long sword at the anvil." and the work driver's
  `Task::Craft(Crafting)` (`Copy`: the picks in a `[Option<usize>; MAX_PICKS]`).
- **The work** (`world/work.rs`, "The work driver" below): `keep_at` for the recipe's `seconds` (`fletch::fletcher_seconds`:
  Fletcher's share for the arrows) in `ActionKind::Craft` turns, watched on the overworld as a
  forage is (`Watch::Forage`); stopped early, "...; you put down your work." and nothing is spent. Done (`crafted`),
  the picks are found again and the plan is resolved again from the pack ("You have nothing
  left to work with."; "Long work's stacks found again" below), then the
  check: two gameplay draws (`combat::roll_natural`), `craft_modifier` = the profession's stat
  modifier (the recipe's profession's `stat`, above: Strength for Metalcraft, Dexterity for
  Woodcraft's, a row's own where it names one) +
  `Actor::skill_effect_of(skill, Effect::Craft)` (that skill's benefit alone, the items' skill
  bonus counted; Metalcraft's `craft` curve is Temper's, +8 at 100, +12 at 200) +
  `TalentMods::check(stat)` (+ Fletcher's `check` for a `count = "fletch"` recipe) +
  `World::binding_boon(skill)` (a bound lure's boon: Jewelcraft's rings and amulets in Gimmwīc,
  #1490), against
  `difficulty` + the main material's `work`. A graded recipe (a count of 1) goes on to its grade;
  a counted one to "Counted recipes" below.
- **The grade** (`grade(settings, natural, margin)`, `Grade`): below 0 `Failed`, from `fine`
  `Fine`, from `marvellous` `Marvellous`, from `masterwork` `Masterwork`, else `Plain`; a natural
  24 at least `Fine`, a natural 2 `Failed`. `make_out` spends the picks, highest position first.
  Failed: each stock pick gives back `piece::count_for` of `yield` % of its grams (at most what
  it took), an input `count × yield / 100`, the gem all of it; "The blade comes out brittle and
  you break it up for the metal." Made: the gem is set, and an item whose base has an `Aspect`
  takes `Grade::improvements()` (1, 2, 3; at most `temper.toml`'s `most`) improvements, each an
  aspect drawn evenly (one gameplay draw), with `attempts` equal to them: a crafted Fine item is a
  once-tempered one (Temper's difficulty `13 + 2`), and a crafted Masterwork is `finished`, so
  Temper refuses it ("It can be made no finer."). An item with no aspect (a ring) is plain
  whatever the margin. The item is identified, joins the pack (`add_to_inventory`, over the limit
  as anything picked up), and is said: "You have made a Masterwork short sword!", "This is a
  Marvellous short sword!", "This is a Fine short sword.", "You have made a short sword.".
- **Marks**: `Use::Craft` on the recipe's skill alone (`World::train_skill`; Metalcraft's
  `mark`, 100 %), whatever came of it; no stat use. Metalcraft, Woodcraft, and Jewelcraft all
  have the `craft` benefit (salvage reads it too), and Metalcraft, Woodcraft, Jewelcraft (for
  Cut), and Alchemy the `craft` mark (Woodcraft's from P5b); `skill_effect_of` keeps one profession's benefit
  from counting for another's recipe.

## The work driver

Every craft's long work, and Decipher's, goes through one driver (#1492; `world/work.rs`): its
`start_*` checks what it checks and returns `Spent::Work(Work)`, a `Task` (one variant per kind:
`Crafting`, `GemWork`, `Deciphering`, `Enchanting`, `Forging`, `Mending`, `Rigging`,
`Salvaging`, `Setting`, `Tempering`) and its seconds (`World::craft_work(task, base)` takes
Quick hands' share, `World::craft_seconds`; Decipher's `Work::spent` none, and a recipe's making
passes `fletch::fletcher_seconds`, which took it already). `World::work`, the one dispatch line
in `world/mod.rs`, then runs `keep_at` for those seconds in turns of the task's action, the last
turn cut short, watched on the overworld where the task says; interrupted, it says the task's
words ("...; you put down your work.", "You leave off your work."), the time spent; done, it
finds the task's stacks again (below) and calls the task's end. Each kind implements `Job`
(`kind`, `watch`, `on`, `found`, `leave`, `nothing_left`, `done`; all but `done` have the common
default), so a rule for every craft goes in `work.rs` once.

## Long work's stacks found again

Every long work on stacks of the pack (`craft_make.rs` `make`, Temper and Whittle, Cut, Set,
salvage, Make do and mend, Decipher, the enchanting recipes) names them by position as it
begins, and a stack gone while it works (food rotting away, which does not stop it) shifts every
later one (#1564). So the driver keeps a copy of its stacks as its time begins (`World::work_on`,
`world/work_on.rs`; only the enchanting picks that are positions, `Enchanting::in_pack`) and finds
those very stacks when it ends (`World::worked_on`: at the old position if still there, else the
first stack equal to it); any gone or changed ends the work on nothing ("You have nothing left to
work on.", Decipher's "...to make out.", a recipe's "...to work with."). Tests: the
`a_stack_gone_ahead_of_the_work_never_turns_it_on_the_next` cases in `salvage_tests.rs` and
`enchant_tests.rs` (a loaf rotting away ahead of the item).

## Brewing

P13 (#1357; `docs/design/recipes.md` §6 "Potions (Alchemy)", §4, §12 items 5, 6, 11): Alchemy's
29 herb recipes, `data/recipes/alchemy/<potion>.toml`, each `result` the potion, `inputs` of
herbs only (`docs/dev/objects.md`'s herb objects), `seconds = 1200`, no station or knife, `failure
= { object = "murky_draught" }`, `fumble = { object = "brewers_blunder" }`, and the table's
`difficulty` and `from`. The elixirs, gambles, deeper draughts, holy water, oil, tar, and the
hexing flasks are never brewed.

P14 (#1358; `recipes.md` §6's essence rows, §9) adds nine files of the same shape whose `inputs`
take an **essence** (`essence_of_<element>`, plain `[[object]]`s of class `materials`,
`docs/dev/objects.md`): `potion_of_frost`, `potion_of_earthing`, `potion_of_sea_legs`,
`flask_of_fire`, `flask_of_frost`, `flask_of_thunder`, `flask_of_smoke`, `pitch_fire`, and
`blasting_flask` (38 brews in all). No code: an object input matches by object id as a herb does,
and so do the bought `flask_of_oil` and `flask_of_tar` (inputs of the flask of fire and pitch
fire).

**Known inputs** (#1383): an object input counts only once the player knows its type.
`craft::known_object(lore, stack, object)` is the one test (the stack is of the object and
`Lore::is_identified(object)`: a labelled potion, flask, scroll, wand, or rod type not yet
identified does not count; an unidentified magic item's base object is plain to see and does), and
every object input is matched through it: `input_fits`, a part's sources' input in `lack`, the
wizard's steps (`craft_steps`), `resolve` (a picked stack), and `taken` (a brew's). So `lack`,
`recipe_rows`, and `resolve` take the world's `Lore`. An unknown flask of oil leaves the flask of
fire's row greyed "not enough flasks of oil" and `craft:flask_of_fire` refused with no time ("You
have too few flasks of oil."): otherwise the row would tell the player what the clear potion
is. The flaming arrows' oil the same. Each essence's source is a tag's material drop
(`data/drops/{fire,ice,lightning,air,water,earth}.toml`, `bodiless`; `fire-thing.toml`'s essence
of fire at 25 %; `docs/dev/creatures.md`); `brew_tests.rs` checks every essence a brew takes drops
from some creature and every flask input is stocked by a generated shop. Alchemy in
`data/skills/` gains the `craft` curve (Metalcraft's) and the `craft` mark (100 %);
`data/recipes.toml`'s Alchemy row names `stat = "intelligence"`, `verb = "Brew"`, and its
`failed` and `fumbled` lines. In `world/craft_make.rs`:

- **Nothing chosen** (`chooses_nothing`: no part, no ornament): `steps` is empty (not `None`), and
  `resolve` takes each object input from the pack (`taken`: the first stacks of the object,
  across stacks, until its count; "You have too few sprigs of moonwort." when short; any pick is
  "That is more than the work takes."). `craft_plan` tells stacks of one thing as one count
  ("2 sprigs of moonwort") and names the result truly though its type is unknown (`made_name`:
  `Stack::describe` for a labelled type, the lore's otherwise), as does "You set to work on the
  potion of healing.". The check's stat and the verb are the profession row's
  (`profession_stat`, `profession_verb`): a row's `stat` or `verb` overrides the stock's
  profession ("Routes and the recipe's profession"), which decides where the row names none.
- **The outcome** (`make_out`): failed with a `failure` object (or a natural 2 with a `fumble`,
  else the failure's), the inputs are spent whole (`Spend::back` 0 for a recipe with a
  `failure`), the profession's `failed` or `fumbled` line is said, and `brew_spoilt` puts the
  object, identified and its type learned, in the pack ("You are left with a murky draught.").
  Made: `learn_scroll` learns the result's type with `identify` (the maker knows their work: a
  brewed potion of a type never met is known from then on; the decision for #1357: the recipe
  names it, so making it teaches it), and a potion, with no `Aspect`, is plain at any margin
  ("You have made a potion of healing."). A natural 24 always makes it.
- **The screen**: a brew's row (`Entry::confirm`: generic, no steps, a plan that holds) opens
  `ScreenName::CraftConfirm` straight from the profession screen; `ScreenStack::craft_chosen`
  reads `ProfessionScreen::chosen` (the `>` row, no picks) when no wizard is open, so the one
  dialog asks "Brew a potion of healing? It uses 2 sprigs of moonwort, and takes about 20
  minutes." A row short of a herb is greyed by `RecipeLack::Input`, "not enough sprigs of
  moonwort".
- **Tests**: `world/brew_tests.rs` (the table's rows and the never-brewed, nothing chosen and the
  plan's words, the potion known and the mark, no grade at margin 20, the murky draught and the
  blunder with the herbs spent, the refusals with no time, Intelligence the stat, a replay of
  gathers and a brew; the essence rows: each made from its inputs, a flask of fire spending one
  bought oil, the refusals "not enough essences of fire" with no time, a failed blasting flask,
  the essences' sources, and a replay that picks up an essence and oil, levels Alchemy past 50
  with `xp:` and `advance:alchemy`, and brews a flask of fire), `content/recipe_tests.rs` (Alchemy's row, a missing herb, an empty line),
  the text snapshots `profession-alchemy-herbs` and `brew-dialog`. The wiki's brewing page,
  `wiki/src/recipes-alchemy.md`, includes `xtask/src/wiki/recipes.rs::brews` (the odds at Alchemy
  20, 50, 100 with an average Intelligence); the forge's tables (`recipes_of`) list each
  profession's own.

## Counted recipes

A result whose `count` is above 1, or `"fletch"`, is **counted** (`craft_make::is_counted`;
`docs/design/professions.md` §7, `recipes.md` §4): the arrowheads, the arrows and bolts, the sling
stones, the flaming arrows. No grades:

- **The batch** (`resolve`): its most (`batch_most`: the dice's most, or `fletch::fletch_most`),
  at most what each stack chosen covers: a `per_made` input's count over its `count`, a stock
  pick's pieces by `covers` (the most whose weight its pieces hold, `piece::count_for`). A stock
  part is priced for that batch (`count_for` of the part's grams times the batch: one billet of
  wood covers up to 11 arrows, one bone shard a whole batch), and the plan asks for that; the
  wizard's row says what the batch at its most takes of the stack, and is pickable with one
  made's worth.
- **The count** (`make_batch`, after the check): the dice rolled (a gameplay draw a die), or
  `fletch::fletch_count` (`fletch.toml`'s dice, at least 1, plus the recipe skill's trained
  value over `value_per_count`); then `batch_made(settings, natural, margin, count)`: all of it
  from a margin of 0, half rounded up within `half_within`, one within `one_within`, else none; a
  natural 2 none, a natural 24 all. Then Fletcher's `count_times` (a `count = "fletch"` recipe),
  then at most the batch.
- **What is spent** (`Spend::gone`): a stock pick the pieces of what was made (at least one), a
  `per_made` input its `count` for each made, another input all of it. None made is a failure:
  "You spoil the whole batch.", the stock and the whole inputs back at `yield` (a single piece
  always comes back), the `per_made` ones untouched.
- **Made**: the stack identified into the pack, "You make 20 steel arrowheads.", joining a stack
  like it.

The shipped ones (`docs/design/recipes.md` §6 "Missiles"):

| Recipe | Profession | Chosen | Inputs | Count | Station, tools | Difficulty | Seconds | Listed |
|---|---|---|---|---|---|---|---|---|
| `arrowheads` | Metalcraft | `main` stock (metal) | — | 20 | anvil | 10 | 600 | from 0 |
| `arrows` | Woodcraft | `head` sources (arrowheads; bone, stone; None: `brittle_arrow`), `shaft` stock (wood) | an arrowhead `per_made` | `fletch` | knife, a bow ready | 12 | 600 | 0 |
| `bolts` | Woodcraft | the same; None: `brittle_bolt` | an arrowhead `per_made` | `fletch` | knife, a crossbow ready | 12 | 600 | 0 |
| `sling_stones` | Woodcraft | `main` stock (stone) | — | 10 | — (no knife) | 6 | 600 | 0 |
| `flaming_arrows` | Woodcraft | — | an arrow `per_made`, a flask of oil | 10 | — | 8 | 600 | from 0 |

## Arrows and bolts

Fletching (spec §8 "Fletching", #1021) is these two recipes (#1336), with no action, ability,
or built-in recipe of its own. Fletching in a replay is `craft:arrows:HEAD,SHAFT` (or
`craft:bolts:...`), `-` for no head: `craft:arrows:-,c`
for brittle arrows from billet `c`, `craft:arrows:-` where wood is free. `world/fletch.rs` holds
what only they read:

- **The head** (`head = { input = "arrowhead", stock = ["bone", "stone"], optional = true }`): the
  wizard's one step offers the arrowheads carried (any metal, one an arrow, `per_made`; the
  arrow's head takes the arrowhead's metal), then the bone shards and stone blocks (bone, horn,
  chitin, ivory, dragonbone; stone, flint, obsidian: the listed groups within the arrow's
  `made_of`; a shard or block heads the whole batch), then None (the `brittle` object,
  `breaks = 100`, of the shaft's wood). The arrow's parts are the head's material and the shaft's
  wood: their own goods, stacking only alike; a flint-headed arrow strikes as flint does (its
  main part's `to_hit` and `damage`, `variant::weapon`). Names follow the variant naming
  (`content/part.rs::names`): "flint arrows", "steel arrows"; the shaft's wood shows in the
  description, not the name.
- **The shaft** (`shaft = "stock"`, the part's `made_of = ["wood"]`): a billet of any wood, priced
  by the batch's weight. **Wood where trees are** (`fletch::part_free`): for a `count = "fletch"`
  recipe, a stock part all of whose worked materials are wood has no step and no lack on ground
  whose landscape `data/recipes.toml`'s `free_wood` lists (`fletch::wood_free`, through
  `forage::landscape_at`: the overworld's cell, or a local area's own); its part is the base wood.
- **The launcher** (`fletch::shoots`): a missile weapon ready whose `launches` is the result's
  `ammo`; `fletch.toml`'s `[[makes]]` gives its word ("no bow ready", "a crossbow ready").
- **The count**: `fletch.toml`'s `count` (`1d4`) + Woodcraft's trained value ÷ `value_per_count`
  (25), Fletcher ×2, half the time, +4 on the check (`TalentMods::fletcher`; headless arrows are
  brittle whatever the skill). `fletch.toml` holds `count`,
  `value_per_count`, `knives`, `[[makes]] { launches, launcher }` (`content/fletch.rs`); the time,
  difficulty, bands, and brittle objects are the recipes' and `data/recipes.toml`'s.

Tests: `world/fletch_tests.rs` (the head step's order and None; an arrow from an arrowhead with
both parts; a shard or block heading a batch, spent once; flint and bone arrows apart and flint's
blows; brittle with no head, even for a master or a ranger; bolts for a crossbow; the knife, the
launcher, and the wood asked for with no time; the rows' words; the bands; arrowheads at the
anvil; sling stones with no knife; flaming arrows by the arrow; Fletcher's half time, double
count, and better check; wood free in the forest area; the mark and an interruption; a replay),
`content/recipe_tests.rs` (the shipped missiles; a head source off the arrow's `made_of`
refused), `content/fletch.rs` (the trimmed file's refusals).

## The forging wizard

`crates/warlock/src/ui/craft_pick_screen.rs`, `ScreenName::CraftPick(RecipeId)` (skipped by
`--screen`; `--screen craft-pick` is `ScreenName::FirstCraftPick`, the first recipe with a
choice: the amber amulet). The profession screen opens it for a generic recipe with steps
(`Entry::wizard`; one without steps is applied at once, the rules saying why). One `PagedList`
per step, grouped by the stacks' item class, read from `craft_steps` when it opens: a row is
`letter - name` with "uses 3" ("uses 1 each" for a `per_made` input), greyed "too few" and unpickable when the stack holds too few; an
optional step draws "- - None" above the list, picked by `-` (an empty one says "None leaves it
plain." for the ornament, "None makes them brittle." for the arrows' head). The header is the recipe's name and
the picks so far ("Short sword — metal: mithril"), the title the recipe and the step's label. A
letter or Enter picks and goes on; Esc goes back a step, its pick kept (the cursor on it), and
closes from the first. The last pick returns `ScreenAction::Open(ScreenName::CraftConfirm)`:
`Screen::new` builds the one `Dialog` (`craft_dialog`) from `Beneath::craft` (the wizard's recipe
and picks, `ScreenStack::craft_chosen`) and the view's `craft_plan`: `Dialog::yes_no(verb, "Forge
a ruby-pommelled mithril short sword? It uses 2 mithril ingots and the ruby, at the anvil, and
takes about 1 hour.", verb, Action::Craft, "Not now")`; declined, it closes back to the last
step; a plan the rules refuse is a dialog with the reason and Back. Accepted, the action applies
and the wizard stays open beneath (as object lists do, #591): its `refresh` reads the steps again
and, when they changed, starts over on its first step; the profession screen beneath refreshes
its rows too. Every key is fixed and named in the footer ("a-c or Enter choose   - none   Esc
back"): no `KeyCommand`, no `Context` entry. Tests: the module's own (picking, `-` only where
optional, Esc back keeping the pick, the dialog's words) and the text snapshots `craft-pick-*`
(each step, the dialog, `--screen craft-pick`) and `profession-metalcraft-stock`.

## Cutting a gem

`world/cut.rs` (P10, #1355; `docs/design/recipes.md` §5), tests in `world/cut_tests.rs`. Cut is
Jewelcraft's built-in recipe, run by `Action::Craft` itself (`craft:cut:LETTER`, one pick: the
gem), dispatched from `World::start_craft` by one arm to `World::start_cut`; its turns are
`ActionKind::Craft`, its work `Spent::Cut(cut::GemWork)`.

- **The list**: the profession screen's Cut row opens `ScreenName::Cut` (`--screen cut`;
  `ObjectList::Cut`, `world::cuttable`): every uncut loose gem, and every cut one above the
  smallest size (tiny) and below the finest cut (flawless). Its pick applies
  `Action::Craft { recipe, picks: [gem] }`, the recipe id read by `Recipes::of_use(Cut)`.
- **Refused, no time**, in order: no pick ("You must choose the gem."), more than one ("That is
  more than the work takes."), below `from` ("That is beyond your skill."), no such stack, not a
  loose gem ("That is no loose gem."), a flawless one ("It could not be finer."), a tiny cut one
  ("There is nothing left to cut."), protected (`item_protected`: "You dare not cut it."), unpaid
  shop goods (`may_loose`). An uncut tiny gem may be cut.
- **Time**: the recipe's `seconds` (30 minutes) by `keep_at`; stopped early, "...; you put down
  your work." and the gem is untouched.
- **The check**: two gameplay draws (`combat::roll_natural`), plus the gem's profession's stat
  modifier (`stock.toml`: Dexterity), `TalentMods::check(stat)`, and
  `skill_effect_of(recipe skill, Effect::Craft)`, against `difficulty` + the gem's `work`.
  `cut_band(gems, cut_step, natural, margin)` (`CutBand`): a natural 2 `Shattered`; below 0
  `Failed`; else the roughest true cut (crude) plus `margin / cut_step`, capped at the finest; a
  natural 24 at least the plain cut (fair, `Gems::plain`). With `cut_step = 5`: 0-4 crude, 5-9
  fair, 10-14 fine, 15-19 exquisite, 20+ flawless. Data-driven: the bands follow `gems.toml`'s
  cut list.
- **The outcome** (`cut_outcome(gems, from, band)`): an uncut gem failed stays uncut a size
  smaller ("The stone splits badly; what is left is smaller."), crude is a size smaller ("You
  rough it out, and lose more than you meant to."), fair and better keep the size ("You cut the
  stone."); a re-cut takes the better of the old cut and the band's and always a size ("You cut
  the stone again." when better, else "You take off more stone and make it no finer."). A size
  lost below tiny loses the gem ("The stone crumbles under your tools; nothing worth keeping is
  left.", in place of the failure's or crude cut's line); a natural 2 "The ruby
  shatters.". One gem is set apart (`set_apart`), removed, and the new one (`ItemPart::gem`, no
  other state: no third ladder) joins the pack, said as names do: "It is now a medium crude
  ruby." Never the margin; Fine, Marvellous, and Masterwork are never said of a gem.
- **Marks**: `Use::Craft` on the recipe's skill (Jewelcraft's `craft` mark, 100 %), whatever came
  of it.
- **The wiki**: `gem-cutting.md` (generated `recipes::gem_cutting`: the bands, a large uncut
  ruby's fates with their worth, the odds by bonus on a ruby and a smokestone, all through
  `cut_band` and `cut_outcome`).

## Setting a gem

`world/set.rs` (P11, #1356; `docs/design/recipes.md` §5), tests in `world/set_tests.rs`. Set is
Jewelcraft's built-in recipe, run by `Action::Craft` itself (`craft:set:ITEM,GEM`, two picks),
dispatched from `World::start_craft` by one arm to `World::start_set`; its turns are
`ActionKind::Craft`, its work `Spent::Set(set::Setting)`.

- **The wizard**: the profession screen's Set row opens `ScreenName::CraftPick(set)`, the forging
  wizard. `craft_steps` and `craft_plan` hand a Set recipe to `set::steps` and `set::plan` (one
  line each): two steps, `CraftStepKind::Item` ("item": `settable` stacks not worn or wielded)
  and `CraftStepKind::Gem` ("gem": the cut loose gems, `craft_make::cut_gem`), neither optional;
  the plan's verb is the recipe's name, "Set", its name the item as it would be ("a
  sapphire-pommelled hand axe"), its uses "the" and the gem. Text snapshots `set-pick-item`,
  `set-pick-gem`, `set-pick-dialog`.
- **What can be set** (`settable`, `ornament_part`): an item whose object names its `ornament`
  part, else whose class has an `[[ornament]]` row in `data/gems.toml`, with no part of a gem's
  material (`MaterialDef::gem`) in that place: a gold amulet's base ruby setting counts as set. A
  loose gem, a piece, and a spirit weapon are not.
- **Refused, no time**, in order: below `from` ("That is beyond your skill."), no item pick ("You
  must choose the item."), no gem pick ("You must choose the gem."), more than two ("That is more
  than the work takes."), no such stack ("You carry no such thing."), an item with no ornament
  part ("No gem can be set in that."), one set already ("It is set already."), worn or wielded
  ("Take it off first."), the gem not a cut loose gem ("Only a cut gem can be set."), either
  protected ("You dare not work on it."), unpaid shop goods (`may_loose`).
- **Time**: the recipe's `seconds` (20 minutes) by `keep_at`; stopped early, "...; you put down
  your work." and nothing changes.
- **The check**: Cut's, against `difficulty` + the gem's `work`; `set_band(natural, margin)`
  (`SetBand`): a natural 2 `Shattered`, a natural 24 or a margin of 0 or more `Set`, else
  `Chipped`.
- **The outcome** (`World::set_out`): set, one item is set apart and replaced by itself with the
  gem as its ornament part (`ItemPart::gem` with the part's id, replacing an own part of that
  place; `item_parts::check`), said "You set the ruby. It is now a ruby-pommelled long sword."
  The item's identification is not changed. Either way one gem is set apart and removed; chipped,
  `salvage::chipped` gives it back a cut rougher or a size smaller ("The ruby slips and chips."
  then "It is now a medium ruby."), or, with neither to lose, it is gone ("The ruby
  shatters."), as on a natural 2.
- **Marks**: `Use::Craft` on the recipe's skill, whatever came of it.
- **The wiki**: `gem-cutting.md#setting-a-gem` (generated `recipes::gem_setting`: the odds by
  bonus on a ruby and a smokestone, through `set_band`).

## Jewellery

P11 (#1356; `docs/design/recipes.md` §6 "Rings and amulets"), generic recipes:

| Recipe | Profession | Parts | `difficulty` | `seconds` | `from` |
|---|---|---|---|---|---|
| iron_ring | Jewelcraft | main `{ stock = ["iron"] }`; an ornament | 11 | 3600 | 0 |
| silver_ring | Jewelcraft | main `{ stock = ["silver"] }`; an ornament | 13 | 3600 | 20 |
| gold_ring | Jewelcraft | main `{ stock = ["gold"] }`; an ornament | 13 | 3600 | 20 |
| jade_amulet | Jewelcraft | main `{ stock = ["jade"] }` | 12 | 3600 | 10 |
| amber_amulet | Jewelcraft | pendant `{ stock = ["amber"] }` (the cord free) | 12 | 3600 | 10 |
| gold_amulet | Jewelcraft | chain `{ stock = ["gold"] }`, setting `{ gem = true }` | 14 | 5400 | 30 |
| bone_amulet | Woodcraft, a knife | main `{ stock = ["bone"] }` (the group: bone, horn, ivory, dragonbone) | 10 | 1800 | 0 |

- Each ring is its own metal (a recipe names its base object); a 10 g ring takes a whole ingot
  (`count_for`'s minimum of one, recipes.md §12 item 12). Jewelcraft's name `verb = "Make"`.
- **A gem part** (`PartSource::Gem`): `craft_make`'s `Step::Gem`, a step of the cut loose gems
  that cannot be skipped (`CraftStepKind::Gem`, labelled by the part, "setting"); resolved as the
  ornament is (`ItemPart::gem` with the part's id, given back whole on a failure). The row is
  greyed `RecipeLack::Gem` ("no gem to hand") with no cut gem carried, and `craft:` refuses
  "You have no gem to hand." with no time.
- No aspect, so never graded; a failure of metal on an item with no aspect says "The work goes
  wrong, and you save what you can of it." (the brittle blade's line is for arms and armour).
- The wiki's table: generated `recipes-jewelcraft.md` (`recipes::recipes_jewelcraft`), under
  `crafting.md`'s "The recipes"; the prose in `crafting.md#making-jewellery`. The first generic
  recipe with steps, `--screen craft-pick`'s, is the amber amulet.

## Salvage

`world/salvage.rs` (one module; one line each in `world/mod.rs`'s `Spent` and dispatch,
`action/text.rs`, `ability.rs`'s matches, `buff.rs`), numbers in `data/salvage.toml`
(`content/salvage.rs`, `Content::salvage`, optional: without it nobody salvages and the ability
is not listed): `seconds = 600`, `difficulty = 12`, `yield = 50`.

- **The ability**: `Ability::Salvage`, Skills group, listed after the skills' own uses (after
  Wedge, so after the profession rows) while the rules exist; had by everyone, never unavailable;
  `Aim::Item` opens `ScreenName::Salvage` (`ObjectList::Salvage`: `world::salvageable` stacks
  not worn or wielded). `Action::Salvage { item }`, text `salvage:LETTER`; `ActionKind::Salvage`
  (`salvage = 1000`).
- **Refused, no time** (in this order): no such stack; nothing a profession works, or a spirit
  weapon, or the currency (coins are never melted down) ("There is nothing in it worth saving."); worn or wielded ("Take it off first.");
  protected (`World::item_protected`: a quest's object, a unique, of Proof; "It will not come
  apart."); unpaid shop goods (`may_loose`: "You have not paid for X.").
- **Time**: `seconds` by `keep_at`, as Temper; interrupted, nothing happens and the time is spent
  ("...; you leave off your work.").
- **The checks**, at the end, re-checking the stack first: one per distinct material among
  `Stack::parts(content)` (resolved, gems included) that `profession_of` maps, in first-seen
  order; each two gameplay draws (`combat::roll_natural`), all drawn before anything changes:
  natural + the stat's modifier + `TalentMods::check(stat)` + `skill_effect_of(skill, Craft)`
  against `difficulty + work`, `check::makes` (a natural 2 fails, a 24 succeeds). Untrained, the
  check is made the same.
- **The outcome**: one item is set apart (`World::set_apart`, Temper's) and removed
  (`outcome::remove_stacks`); `piece::break_up(item, yield)` gives its falls, each kept by its
  material's check. A made material: its pieces (`yield` % of its grams, at least one), or the
  loose gem with its size and cut. A failed material is lost ("The blade cracks and is good for
  nothing.": the part's name, else the material's); a failed gem is `salvage::chipped` (the
  integrator's answer 4 on #1337): one cut rougher while the cut below is a true cut (above
  crude), else one size smaller while above tiny ("The ruby chips."); with neither to lose, or on
  a natural 2, it shatters ("The ruby shatters."). A material no profession works gives nothing
  and is not mentioned (a long sword's leather grip is Woodcraft's now: hide). Affixes,
  improvements, rung, and charges go with the item; its name is the lore's title, so an
  unidentified item is never named. What is saved joins the pack ("You save X and Y.", or
  "Nothing is left worth keeping.").
- **Magic** (docs/dev/enchanting.md "Salvage's affix checks"): before the material checks,
  `World::catch_magic` (`world/crystal.rs`) makes one Enchanting check per affix of a magic or
  rare item, identified or not (an unidentified one's lines name no affix), each made a crystal
  of the affix's colour, and a deep rare's chance of a rare material; the crystals join the saved
  list after the materials.
- **Marks**: `Use::Salvage` on each distinct profession rolled, that skill alone
  (`World::train_skill`, a filter on `World::train`'s loop), and on Enchanting when an affix
  check was made; untrained skills get none. No noise, no alignment.
- **Tests**: `world/salvage_tests.rs` (the owner's ruby-tipped hardwood quarterstaff over many
  fates: both, wood only, ruby only, neither, the shattering; the cut kept; `chipped`'s ladder;
  ten minutes; one of a stack; every refusal with no time, unpaid goods in Grimbold's shop; an
  interruption; the marks; `skill_effect_of` not summing; the ability; a replay),
  `content/salvage.rs` (the loader), the `Salvage` screen snapshot (`screen_snapshots.rs`).

## Waste not (the craftsperson's)

The craftsperson's class ability at level 6 (#1420; the craftsperson design §7.3;
`data/talents/waste_not.toml`): "nothing lost but time", two effects and three reads, each
where the rule already reads its number:

- **`waste_not { salvage, failed }`** (`TalentMods::salvage_percent`, `failed_percent`, the most
  held of each). `salvage`: `world::salvage_percent(content, actor)` is `data/salvage.toml`'s
  `yield` or the holder's, whichever is more, read by `salvaged` for `piece::break_up` and by
  Eye for the make's line (`PerceptionView::describe_item`), so the description counts what the
  work would give. 75: a quarterstaff gives 8 billets for 5, chain mail 17 iron ingots for 11. A
  gem's fate (chipped, shattered) is unchanged. `failed`: `craft_make.rs::failed_percent`, the
  settings' `yield` or the holder's, at most 100, read by `resolve` and `taken` as they set each
  `Spend::back` (stock by `piece::count_for`, never more than was taken; object inputs `count ×
  percent / 100`). 100: a failed recipe, a natural 2 too, gives back all its stock and object
  inputs, and a counted recipe that made none its stock and whole inputs (`per_made` ones were
  never spent). A brew with a `failure` object still spends its inputs (`back` 0 before the
  percent is asked). The metal-arms line ends ", every ounce."; another failure says "The work
  goes wrong, but you save every scrap of it."
- **`temper_safe`** (`TalentMods::temper_safe`): `temper.rs::work_out`'s failure arm, before item
  damage is asked: the item keeps its `Worked` as it was, the attempt counted (`attempts` + 1, so
  the next is harder), and nothing is drawn; a damaged item is never destroyed. Temper and Whittle
  alike, a natural 2 too ("The work goes wrong, but you know when to stop before the steel
  spoils." / "... the wood spoils.").

Tests: `world/waste_not_tests.rs` (level 6, each read with and without the talent).

## Make do and mend

`world/make_do.rs` (#1422; the craftsperson design §7.5): the craftsperson's class active at
level 18, the talent `make_do_and_mend` with `{ kind = "make_do", seconds = 600, difficulty = 10
}` (`TalentEffect::MakeDo`, `TalentMods::make_do`). One line each in `world/mod.rs`'s `Spent`
(`Spent::MakeDo`) and dispatch, `world/merchants_eye.rs`'s `use_class_on` (the class actives on a
stack), and `world/class_ability.rs`'s bare `use` ("Use it on what?").

- **The ability**: `ClassActive::MakeDo` (id `make_do_and_mend`, `aims_item`, `ActionKind::Craft`),
  the abilities menu's Class group; `Aim::Item` opens `ScreenName::MakeDo` (`--screen make-do`,
  `ObjectList::MakeDo`: `world::mendable` stacks, damaged ones, not worn or wielded). Greyed
  "nothing to mend" (`Unavailable::NothingToMend`, from `world::mends_any` in
  `PerceptionView::unavailable`) with none. `Action::UseOn(MakeDo, item)`, text
  `use:make_do_and_mend:LETTER`.
- **Refused, no time** (in this order): no such stack ("You carry no such thing."); not damaged
  ("There is nothing wrong with it."); worn or wielded ("Take it off first."); a main material no
  profession works (`World::profession_of`: cloth, glass; "You know no craft that would mend
  it."); protected (`World::item_protected`: "You dare not work on it."); unpaid goods
  (`may_loose`).
- **Time**: `seconds` by `keep_at` in `craft` turns; interrupted, nothing happens and the time is
  spent ("...; you put down your work.").
- **The check**, at the end, the stack checked again: natural (two gameplay draws) + the main
  material's profession's stat modifier + `TalentMods::check(stat)` + `skill_effect_of(skill,
  Craft)` against `difficulty + work` (`check::makes`); untrained, the stat alone, as Salvage's.
- **Made**: one item set apart (`World::set_apart`) and `Worked::repair`: `damaged` and `by`
  cleared, the improvements lost stay lost, `attempts` untouched; nothing re-identified. "You make
  good the damage: it is a long sword again."
- **Failed**: nothing changes ("You cannot make it good here; it will need an anvil."); a natural
  2 the same. Nothing is ever destroyed; trying again costs only the time.
- **Marks**: `Use::Craft` on that profession alone (`World::train_skill`), if trained.
- **Eye for the make** says the damaged line with ", or your own hands" to a holder
  (`magic/describe.rs::make_lines`'s `mends`).
- **Tests**: `world/make_do_tests.rs` (the level, the action text, every refusal with no time, the
  row greyed and the wielded left out, made with the attempts kept, failed with nothing lost, Eye
  for the make's line, a replay), the `make-do-list` screen snapshot.

## Quick hands and Master's hand (the craftsperson's)

`world/mastery.rs` (#1425; the craftsperson design §7.8, §7.9): the craftsperson's passives at
levels 40 and 50, each one read.

- **Quick hands** (`data/talents/quick_hands.toml`, `{ kind = "craft_time_percent", value = 50
  }`, `TalentEffect::CraftTimePercent`, `TalentMods::craft_time_percent`, the least held):
  `mastery::craft_seconds(content, actor, base)` (and `World::craft_seconds(base)` for the
  player) is the time of every craft, at least a second. Every craft's `start_*` asks it for the
  seconds the work driver runs (`World::craft_work`, "The work driver" above): `fletch::fletcher_seconds` (every recipe's time, read there
  alone: forged, counted after Fletcher's `time_percent` so the two multiply, brewed, Smelt,
  and any recipe a later profession adds, enchanting's too), `temper.rs` (Temper and Whittle),
  `cut.rs`, `set.rs`, `salvage.rs`, `make_do.rs`, `field_forge.rs`, and `rig.rs`. The lists that
  tell a craft's time ask the same (`craft.rs::takes`, the forging wizard's and Set's `CraftPlan`,
  the rig list), so
  the time shown is the time spent. Not crafts, unchanged: Gather, Forage, Scribe, Decipher. A
  new craft starts with `craft_work` and its data's seconds.
- **Master's hand** (`data/talents/masters_hand.toml`, `{ kind = "grade_ease", value = 4 }`,
  `TalentEffect::GradeEase`, `TalentMods::grade_ease`, summed): `mastery::graded_margin(content,
  actor, margin)` adds it to a made check's margin (0 or more), never a failure's, so whether the
  work is made never changes. Two reads: `craft_make.rs::make_out` before `grade` (Fine from 2,
  Marvellous from 8, Masterwork from 16) and `cut.rs`'s check before `cut_band` (each cut a step
  of 4 sooner). A natural 2 still fails or shatters; a natural 24 still makes Fine or the plain
  cut at least. Set, a counted batch, and a brew have no grade and are unchanged.
- **Wiki**: `xtask/src/wiki/recipes.rs` reads the talent's `grade_ease` (`grade_ease`): the
  grades' and the cuts' margin tables gain a "With Master's hand" column, and the odds tables a
  column of the graded outcomes with it.
- **Tests**: `world/mastery_tests.rs` (the levels, the halved times of a recipe, Fletcher's share
  multiplied, Temper and Salvage run through, the eased margin and a failure kept, each grade and a
  cut four sooner).

## Maker's mark

The craftsperson's passive at level 60 (#1426; the craftsperson design §7.10): the talent
`makers_mark` with `{ kind = "makers_mark", worth_percent = 50 }` (`TalentEffect::MakersMark`,
`TalentMods::makers_mark`, `worth_percent` 0 refused at load).

- **The one read**: `world/craft_make.rs::make_out`, after the grade's improvements are drawn: a
  holder's made item whose object has a `kind` (arms, armour, a tool, jewellery) is built with
  `Worked::marked` (a plain one gets a `Worked` with nothing else on it), whatever the grade. A
  counted batch (`make_batch`), a brew's spoilt draught, and a kindless result (an ingot, a
  lantern, a potion) are never marked. No draw: the mark changes no stream.
- **Worth**: `Worked::value(worth, mark_percent)`, after the grade's multiplier or the damage's
  halving, `worth_percent` more, rounded down. The percent is `TalentBook::mark_percent` (the
  most any `makers_mark` effect says, cached at load), so a marked item keeps its worth whoever
  carries it.
- **Kept**: `Worked::damage` and `repair`, Temper, and Make do and mend never touch the flag.
- **Read elsewhere**: the patron's pay through the worth alone (`world/patron.rs`,
  `PatronRules::paid`; no bonus of its own on top, #1489), the capacity
  (+1, `magic/capacity.rs`), the description's line (`magic/describe.rs::mark_line`: "It bears
  {name}'s mark." with the character's name from `PerceptionView::describe_item`, "It bears its
  maker's mark." from the nameless `lines`), the dev dump's ` (marked)`.
- **Saves**: `Worked::marked` is new saved state, left out while false (saves pinned to the
  version).
- **Tests**: `world/makers_mark_tests.rs` (the level, a made sword marked and worth half again, the
  description, the capacity, a brew and an ingot unmarked, the patron's pay, no mark without the
  talent).

## Dumps

The `recipes` layer: a `Metalcraft:` line per profession, then each row the player sees,
`  Temper: a metal weapon or armour, an anvil, about 10 minutes (no anvil beside you)`, `  none`
under a profession with none. The `abilities` layer lists `Metalcraft… (skill): a recipe`.

## Tests

`content/recipe_tests.rs` (the built-ins, a generic fixture of every field, the second half's
fixtures of every new field and the old shape unchanged, each check's refusal, the settings
file's bands and refusals), `world/craft_tests.rs` (rows by training and place, each reason,
`craft:` refused with no time, the abilities menu's rows, Jewelcraft's rows), `world/cut_tests.rs` (Cutting a gem), `ability.rs`'s registry test (the
profession rows in their skills' places, no Temper, Whittle, or Fletch row; `world/fletch_tests.rs`
for the counted recipes), `ui/step_dialogs.rs`'s
walk-in test (the anvil still asks, opening the Temper list), `ui/profession_screen.rs` (a
built-in row's action, a greyed row), `world/set_tests.rs` (Setting a gem, Jewellery), and the
text snapshots `profession-*` and `set-pick-*` in
`ui/screen_snapshots.rs`. The forge's tests are under "Making a recipe" and "The forging wizard".

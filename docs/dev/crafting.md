# Crafting: professions, recipes, salvage

Spec §8 "Skills" (professions); design `docs/design/professions.md` (§6 the recipe format, §8 the
menus) and `docs/design/recipes.md` (§4 the grades, the format's second half, P4b); epic #1337.
This topic covers what is built: the recipe data, the profession rows of the abilities menu, the
profession screen, `Action::Craft`, who works which material, making a generic recipe (the
forge, P5), the forging wizard, and salvage. Material stock (the piece) is in
`docs/dev/objects.md`.

## Who works what: professions and stock

- **The skills** (`data/skills.toml`): Metalcraft, Woodcraft, and Jewelcraft are the professions
  that work materials today (Alchemy, Enchanting, Herbalism, and Mining are skills with no craft
  yet, #1331). Each declares `{ effect = "craft", curve = [[0, 0], [100, 8], [200, 12]] }`
  (Temper's curve, read by salvage and by a recipe made) and a `salvage` mark rule at 100 %
  (Metalcraft and Woodcraft also a `craft` mark, for a recipe made). Temper's, Whittle's, and Fletch's own
  effects (`Effect::Temper`, `Whittle`, `Fletch`) are unchanged.
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
profession (a skill with a recipe directory is a profession: Metalcraft and Woodcraft today,
Jewelcraft the day its first file lands). `content/recipe.rs`: `RecipeDef`, `RecipeId` (an index
in path order, never saved: actions name recipes by id), `RecipeUse`, `PartSource`, `RecipePart`,
`RecipeInput`, `RecipeResult`, and `Recipes` (`Content::recipes()`: the list, `professions()` the
skills with a directory in skill order, `of(skill)`, `id(name)`, `description(skill)`). Loaded
after `data/stock.toml` (a stock part's materials must be the profession's) through
`ContentFiles::table("recipes")`, so campaigns' `data/recipes/<skill>/` join in.

`data/recipes.toml` is the settings file the professions share: each profession's line in the
abilities menu (`[[profession]] skill, description`, "Your recipes as a smith."; a profession
without a row shows its skill's description), and the bands (`RecipeSettings`,
`Recipes::settings`, `None` only for content without the file), all required, struct
`deny_unknown_fields`:

| Setting | Value | Meaning (read from P5 on) |
|---|---|---|
| `fine`, `marvellous`, `masterwork` | 6, 12, 20 | the check's margin from which a made item is Fine, Marvellous, a Masterwork (recipes.md §4) |
| `cut_step` | 5 | a gem's cut rises a grade every step of margin (§5) |
| `half_within`, `one_within` | 3, 6 | how far below the difficulty a counted recipe still makes half its count, one (copies of `fletch.toml`'s, which P6 retires) |
| `yield` | 50 | the percent of the stock and object inputs a failure gives back (`yield_percent`) |
| `free_wood` | forest, dense forest, swamp | `Landscape`s where wood is free (an arrow's shaft takes no billet) |

| Field | Meaning |
|---|---|
| `id` | the file name; `craft:ID` names it |
| `name` | the row's name (default: the result object's, capitalized, or the use's) |
| `use` | a **built-in** recipe: `temper`, `whittle`, `fletch`, `cut`, `set` (`RecipeUse`); its row runs that existing action with its own list and rules. Cut and Set (Jewelcraft's) have no action yet (`RecipeUse::usable` false): their rows are greyed "not usable yet" (`RecipeLack::NotYet`), `craft:cut` says "You cannot do that yet.", and the profession screen does nothing on them, until P10 and P11 (which add `recipes/jewelcraft/cut.toml` and `set.toml`: the loader needs no file now) |
| `result` | `{ object, count }`: a **generic** recipe's result; `count` dice (`RecipeCount::Dice`) or `"fletch"` (`RecipeCount::Fletch`, Fletch's count) |
| `[recipe.parts]` | `main` is the first part; sorted into the result's part order. `blade = "stock"` (`PartSource::Stock`: the profession's stock), a **table of sources** `head = { input = "arrowhead", stock = ["bone", "stone"], optional = true }` (`PartSource::Sources`: the wizard's one step offers the input's stacks, then stock of the listed groups or materials of any profession, then None if `optional`; `stock` holds the materials of the part's `made_of` the list names, in `made_of` order; a name both a group and a material, `stone`, is the group), or `setting = { gem = true }` (`PartSource::Gem`: a cut loose gem, required) |
| `inputs` | `[{ object, count, per_made }]`, or fixed stock: `{ material = "sinew", count }`, `{ group = "metal", count }` (`InputKind`; `RecipeInput::object()` for an object input) |
| `failure`, `fumble` | `{ object }`: what a failed check, a natural 2, makes instead of nothing |
| `brittle` | the result when an `optional` part is None |
| `ornament` | an optional gem for the result's ornament part |
| `difficulty`, `seconds` | a generic recipe's check base and time |
| `station` | a terrain id that must stand beside the maker |
| `knife`, `launcher` | a knife carried; a launcher ready that fletching makes ammunition for |
| `from` | the trained value it is listed from (default 0) |
| `anyone` | listed for the untrained too |

Checked at load (`recipe::parse`; refusals tested in `content/recipe_tests.rs`): a file in a
skill's directory (the directory a known skill), one `[[recipe]]` a file (a file with none, a
README, is skipped), the id its file name and unique across professions; a `use` among the three
and with no result, parts, inputs, ornament, difficulty, or seconds; a generic recipe with a
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
also builds its wizard step and its row's words and lack: the input step and `brittle` with P6
(arrows), stock inputs with P7 (sinew, iron), `gem = true` with P11, `failure` and `fumble` with
P13 (brewing). The bands and `yield` are read from P5 (the forge, "Making a recipe" below).
Until then the rows and the wizard read only stock parts, object inputs, and the ornament. `data/stock.toml`'s `unit` and `[[material]]` overrides (P2b) are checked by
`content/stock.rs`.

The three built-ins: `recipes/metalcraft/temper.toml` (`anyone`, `station = "anvil"`),
`recipes/woodcraft/whittle.toml` (`anyone`, `knife`), `recipes/woodcraft/fletch.toml` (`anyone`,
`knife`, `launcher`). Temper, Whittle, and Fletch stay open to everyone (#1337 Q3).

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
(`knife = true`) and no station, and are trained only (no `anyone`). Ids are unique across
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
  knife Metalcraft's with Strength. `work` stays the main chosen part's material's (a spear's
  head, not its haft).
- **The row's words** (`craft::stock_words`) price a part once per material group it may be of,
  the base material's group first: "7 wood billets or 7 bone shards", "a bone shard or a block of
  flint". The greyed reason names the first material's group ("no bone to hand"; the haft's "no
  wood to hand").
- **The wizard's label** for an unnamed main part is its stock's group when all of it is one
  group ("metal", "wood"), else "material".
- **A failure** not of metal says "The work goes wrong, and you save what you can of it." (the
  metal's line stays).
- Tests: `world/craft_hand_tests.rs` (the design's counts, the spear's steps, the haft refused
  and greyed, the knife, skill, and station greyings, the profession's stat, verb, and mark, a
  bone javelin made and still thrown).

## The rows: `Ability::Profession`

`ability.rs`: `Ability::Profession(SkillId)`, `Source::Skill`, named "Metalcraft…" (the skill's
name and an ellipsis), described by `data/recipes.toml`'s line, `Aim::Recipes`, no cost, never
unavailable. `Ability::every` lists it in its skill's place (skill order) for every skill with
a recipe directory, before `Ability::use_of` and `Ability::Skill`, so a profession has no "not
usable yet" row (Jewelcraft keeps its until it has a directory). `had_by`: trained in the skill,
or any of its recipes `anyone` (so everyone has both rows today). The former `Ability::Temper`,
`Ability::Whittle`, and `Ability::Fletch` are gone; `Offer::Temper` stays: the anvil is the
Temper recipe's station and the walk-in dialog (`input.rs::offer_dialog`) still asks "Would you
like to temper something at the anvil?", opening `ScreenName::Temper` directly
(`Ability::offered` is `None` for it, so the Here group lists only Pray and Drink). Wedge stays
its own row (a door, not a recipe). `input.rs::use_ability` opens
`ScreenName::Profession(skill)` for `Aim::Recipes`.

## What the screen reads: `PerceptionView::recipes`

`world/craft.rs::recipe_rows` (through `PerceptionView::recipes(skill)`): the profession's
recipes in path order, every one for a player trained in the skill and only the `anyone` ones
otherwise, each a `RecipeRow { id, name, takes, unavailable }`. Computed in core from the pack,
the cells beside the player, and the skills: presentation never decides availability.

- `takes`: what a built-in works on ("a metal weapon or armour", "a wooden weapon"), each stock
  part priced in pieces (`piece::count_for` of the part's share of the result's weight, named by
  `piece::names`: "3 steel ingots"), inputs ("20 arrowheads", "an arrowhead for each made"), the
  station ("an anvil"), "a knife", "a bow or crossbow ready", then the time in
  `Duration::spoken` ("about 10 minutes": Temper's and Whittle's `data/temper.toml`, Fletch's
  `data/fletch.toml` cut by Fletcher, a generic recipe's `seconds`). Never a difficulty or odds.
- `unavailable`: the first `RecipeLack` that applies, in this order: `NotYet` ("not usable
  yet": a built-in without its action, Cut and Set), `Skill` ("beyond your
  skill": the trained value below `from`), `Station` ("no anvil beside you"), `Knife` ("no
  knife"), `Launcher` ("no bow or crossbow ready": `fletch::fletch_makes`), `Stock` ("no metal to
  hand": no piece whose material fits a stock part), `Input` ("no arrowheads").

## The profession screen

`crates/warlock/src/ui/profession_screen.rs`, `ScreenName::Profession(SkillId)` (skipped by
`--screen`; `--screen profession` is `ScreenName::FirstProfession`, the first profession's,
Metalcraft). A `PagedList` grouped by the result's item class, the built-in recipes first under
"Work" (group `None`). A row: the letter and name, then what it takes; a row that cannot be made
is greyed (`TEXT_DIM`) with its reason in place of what it takes, and the line under the list
gives the selected row's `takes` in full. A letter or Enter on the `>` row makes an available
recipe: Temper opens `ScreenName::Temper` over the screen, Whittle `ScreenName::Whittle`, Fletch
applies `Action::Fletch`; a greyed one does nothing (its reason is on the row). The list keys
page; Esc closes. Every key is fixed and named in the footer ("a-b or Enter make   Esc back"):
no `KeyCommand`, no `Context` entry. A generic recipe's row applies `Action::Craft` with no
picks for now; P5's wizard (`ui/craft_pick_screen.rs`) takes its place.

## The action: `Action::Craft`

`Action::Craft { recipe: String, picks: Vec<Option<usize>> }`, replay text `craft:ID[:POS,...]`
(positions as `position_text`, `-` a skipped optional: `craft:long_sword:c,f`,
`craft:dagger:a,-`), dispatched by one arm in `World::player_acts` to `World::start_craft`
(`world/craft.rs`). An unknown recipe is refused ("You know no such recipe."), and so is a
built-in one ("Temper is done from its own list.": its row runs `temper:`, `whittle:`, or
`fletch`), both taking no time; a generic one goes to `World::start_making` ("Making a recipe"
below). Its turns are `ActionKind::Craft` (`data/actions.toml` `craft = 1000`;
`World::kind_of`).

## Making a recipe

`world/craft_make.rs` (P5, #1335; `docs/design/recipes.md` §4, `professions.md` §7), tests in
`world/craft_make_tests.rs`:

- **Steps** (`craft_steps`, `PerceptionView::craft_steps(recipe)`): one `CraftStep` per choice in
  the wizard's order: each stock part (`"stock"`, or a table of `stock` alone, P5b) (`CraftStepKind::Stock`, labelled by the part's name,
  "blade", or for an unnamed main part by its stock's group, "metal"), each object input
  (`Input`), then the ornament (`Ornament`, `optional`; the part is `ObjectDef::ornament` or the
  class's `[[ornament]]`). Each lists the stacks carried that fit, `CraftOption { at, name (as
  the inventory lists it), short (the material: "steel", "ruby"), takes, enough }`: pieces whose
  material the part's `made_of` holds and the profession works (`worked_materials`), stacks of
  the input's object, cut loose gems (never an uncut one, `Cut::rough`). A part takes its share
  of the result's weight scaled by the material's weight over the base part's (as the made item
  weighs it), in pieces (`piece::count_for`): a mithril blade takes as many mithril ingots as a
  steel one takes steel. A recipe with another part source, a stock input, a count above 1,
  `failure`, `fumble`, or `brittle`, or more than `MAX_PICKS` (8) steps has no steps, and
  `craft:` refuses it ("You cannot make that yet.") until the phase that builds its step.
- **The plan** (`craft_plan`, `PerceptionView::craft_plan(recipe, picks)`; `resolve`): every
  required step picked ("You must choose the metal."), the stack there ("You carry no such
  thing."), fitting ("That will not do for the blade.", "Only a cut gem can be set."), holding
  what it takes, summed over steps that pick it twice ("You have too few of those."); more picks
  than steps are refused ("That is more than the work takes."). The made item is the result's
  object with own parts (`ItemPart::material` for a chosen material other than the base part's,
  `ItemPart::gem` for the ornament; `item_parts::check`). `CraftPlan { verb, name, uses,
  station, time }`: the verb `stock.toml`'s for the recipe's profession ("Forge"), the
  name `Lore::describe` of the made item ("a ruby-pommelled mithril long sword"), `uses` the
  pieces counted and "the ruby".
- **Start** (`World::start_making`): refused with no time by the row's `lack` (beyond skill: "That
  is beyond your skill."; no station: "There is no anvil here."; no knife), then by the plan's
  reason. Then "You set to work on the long sword at the anvil." and `Spent::Craft(Crafting)`
  (`Copy`: the picks in a `[Option<usize>; MAX_PICKS]`).
- **The work** (`World::make`): `keep_at` for the recipe's `seconds` in `ActionKind::Craft`
  turns; stopped early, "...; you put down your work." and nothing is spent. Done (`crafted`),
  the plan is resolved again from the pack ("You have nothing left to work with."), then the
  check: two gameplay draws (`combat::roll_natural`), `craft_modifier` = the profession's stat
  modifier (`stock.toml`'s `stat` for the recipe's profession: Strength for Metalcraft) +
  `Actor::skill_effect_of(skill, Effect::Craft)` (that skill's benefit alone, the items' skill
  bonus counted; Metalcraft's `craft` curve is Temper's, +8 at 100, +12 at 200) +
  `TalentMods::check(stat)`, against `difficulty` + the main material's `work`.
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
  have the `craft` benefit (salvage reads it too), and Metalcraft and Woodcraft the `craft`
  mark (Woodcraft's from P5b); `skill_effect_of` keeps one profession's benefit from counting for
  another's recipe.

## The forging wizard

`crates/warlock/src/ui/craft_pick_screen.rs`, `ScreenName::CraftPick(RecipeId)` (skipped by
`--screen`; `--screen craft-pick` is `ScreenName::FirstCraftPick`, the first recipe with a
choice: the dagger). The profession screen opens it for a generic recipe with steps
(`Entry::wizard`; one without steps is applied at once, the rules saying why). One `PagedList`
per step, grouped by the stacks' item class, read from `craft_steps` when it opens: a row is
`letter - name` with "uses 3", greyed "too few" and unpickable when the stack holds too few; an
optional step draws "- - None" above the list, picked by `-`. The header is the recipe's name and
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

## Salvage

`world/salvage.rs` (one module; one line each in `world/mod.rs`'s `Spent` and dispatch,
`action.rs`, `ability.rs`'s matches, `buff.rs`), numbers in `data/salvage.toml`
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
- **Marks**: `Use::Salvage` on each distinct profession rolled, that skill alone
  (`World::train_skill`, a filter on `World::train`'s loop); untrained skills get none. No noise,
  no alignment.
- **Tests**: `world/salvage_tests.rs` (the owner's ruby-tipped hardwood quarterstaff over many
  fates: both, wood only, ruby only, neither, the shattering; the cut kept; `chipped`'s ladder;
  ten minutes; one of a stack; every refusal with no time, unpaid goods in Grimbold's shop; an
  interruption; the marks; `skill_effect_of` not summing; the ability; a replay),
  `content/salvage.rs` (the loader), the `Salvage` screen snapshot (`screen_snapshots.rs`).

## Dumps

The `recipes` layer: a `Metalcraft:` line per profession, then each row the player sees,
`  Temper: a metal weapon or armour, an anvil, about 10 minutes (no anvil beside you)`, `  none`
under a profession with none. The `abilities` layer lists `Metalcraft… (skill): a recipe`.

## Tests

`content/recipe_tests.rs` (the built-ins, a generic fixture of every field, the second half's
fixtures of every new field and the old shape unchanged, each check's refusal, the settings
file's bands and refusals), `world/craft_tests.rs` (rows by training and place, each reason,
`craft:` refused with no time, the abilities menu's rows, Cut and Set not usable yet), `ability.rs`'s registry test (the
profession rows in their skills' places, no Temper, Whittle, or Fletch row), `input.rs`'s
walk-in test (the anvil still asks, opening the Temper list), `ui/profession_screen.rs` (a
built-in row's action, a greyed row), and the text snapshots `profession-*` in
`ui/screen_snapshots.rs`. The forge's tests are under "Making a recipe" and "The forging wizard".

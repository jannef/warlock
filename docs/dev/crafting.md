# Crafting: professions and recipes

Spec §8 "Skills" (professions); design `docs/design/professions.md` (§6 the recipe format, §8 the
menus) and `docs/design/recipes.md` (the format's second half, P4b); epic #1337. This topic
covers what is built: the recipe data, the profession rows of the abilities menu, the profession
screen, and `Action::Craft`. Material stock (the piece) is in `docs/dev/objects.md`.

## Recipes as data

`assets/data/recipes/<skill>/<id>.toml`, one `[[recipe]]` a file; the directory is the
profession (a skill with a recipe directory is a profession: Metalcraft and Woodcraft today,
Jewelcraft the day its first file lands). `content/recipe.rs`: `RecipeDef`, `RecipeId` (an index
in path order, never saved: actions name recipes by id), `RecipeUse`, `PartSource`, `RecipePart`,
`RecipeInput`, `RecipeResult`, and `Recipes` (`Content::recipes()`: the list, `professions()` the
skills with a directory in skill order, `of(skill)`, `id(name)`, `description(skill)`). Loaded
after `data/stock.toml` (a stock part's materials must be the profession's) through
`ContentFiles::table("recipes")`, so campaigns' `data/recipes/<skill>/` join in.

`data/recipes.toml` is the settings file the professions share; today each profession's line in
the abilities menu (`[[profession]] skill, description`, "Your recipes as a smith."; a
profession without a row shows its skill's description). Its struct is `deny_unknown_fields`:
the forge's bands, the counted bands, and a failure's yield join it with P4b and P5.

| Field | Meaning |
|---|---|
| `id` | the file name; `craft:ID` names it |
| `name` | the row's name (default: the result object's, capitalized, or the use's) |
| `use` | a **built-in** recipe: `temper`, `whittle`, `fletch` (`RecipeUse`); its row runs that existing action with its own list and rules |
| `result` | `{ object, count }`: a **generic** recipe's result, `count` dice |
| `[recipe.parts]` | `blade = "stock"` (`PartSource::Stock`) or `head = { input = "arrowhead" }` (`PartSource::Input`); `main` is the first part; sorted into the result's part order |
| `inputs` | `[{ object, count, per_made }]` |
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

The three built-ins: `recipes/metalcraft/temper.toml` (`anyone`, `station = "anvil"`),
`recipes/woodcraft/whittle.toml` (`anyone`, `knife`), `recipes/woodcraft/fletch.toml` (`anyone`,
`knife`, `launcher`). Temper, Whittle, and Fletch stay open to everyone (#1337 Q3). No generic
recipe exists yet (P5 adds the forge's).

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
- `unavailable`: the first `RecipeLack` that applies, in this order: `Skill` ("beyond your
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
(`world/craft.rs`). It is refused, taking no time, for every recipe now: an unknown one ("You
know no such recipe."), a built-in one ("Temper is done from its own list.": its row runs
`temper:`, `whittle:`, or `fletch`), and a generic one ("You cannot make that yet.")
until P5. `World::kind_of` gives it no kind yet.

## Dumps

The `recipes` layer: a `Metalcraft:` line per profession, then each row the player sees,
`  Temper: a metal weapon or armour, an anvil, about 10 minutes (no anvil beside you)`, `  none`
under a profession with none. The `abilities` layer lists `Metalcraft… (skill): a recipe`.

## Tests

`content/recipe_tests.rs` (the built-ins, a generic fixture of every field, each check's
refusal, the settings file's), `world/craft_tests.rs` (rows by training and place, each reason,
`craft:` refused with no time, the abilities menu's rows), `ability.rs`'s registry test (the
profession rows in their skills' places, no Temper, Whittle, or Fletch row), `input.rs`'s
walk-in test (the anvil still asks, opening the Temper list), `ui/profession_screen.rs` (a
built-in row's action, a greyed row), and the text snapshots `profession-*` in
`ui/screen_snapshots.rs`.

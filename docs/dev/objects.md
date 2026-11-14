# Objects, piles, and inventories

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/07b-data.md and spec/06-input-and-ui.md. Keep it current with the code, as CLAUDE.md says.

Objects (spec §5, §6, §7; M11): every object in `data/objects/` (one file per object, named by
its id; a family file only for variants always edited together, `arrows.toml`) defines its glyph,
color, item class (`class`, below), what it is made of (`materials`, its parts, below; materials
in `data/materials/`, one file per material), and weight in grams (a scroll also what reading it
does, `read`: `docs/dev/scrolls.md`; a grimoire, generated for each spell scroll by
`content/grimoire.rs` from `data/grimoires.toml`, the spell it teaches, `ObjectDef::grimoire`:
`docs/dev/spells.md` "Learning")
(`content/object.rs`: `ObjectDef`, `ObjectId`, and the `"id*N"` placement strings;
`content/material.rs`); the tables are directories read in path order, see "Content
directories" in `docs/dev/content.md`. Every creature has a base `weight`. `object.rs` holds the runtime side: `Stack` (object + count; stackable objects join,
others are stacks of one (a stack worked at an anvil, `Stack::worked`, `magic/worked.rs`, is like no other and never joins one; docs/dev/special-rooms.md; `Worked::marked`, saved, default false, is a craftsperson's Maker's mark: a marked plain item carries a `Worked` with nothing else on it; `Worked::value` adds `TalentBook::mark_percent` to its worth, `magic/capacity.rs` one enchantment, `world/crafting/patron.rs` through the worth, `magic/describe.rs::mark_line` "It bears Brakka's mark.", the dev `actors` dump ` (marked)`; docs/dev/crafting.md "Maker's mark"); food knows when it was made, `Stack::died`, and joins whatever its age,
keeping the oldest: `docs/dev/hunger.md`), piles (`add_to_pile`: dropped stacks go on top, index 0 is the top the
map shows), inventories (`add_to_inventory`: any number of stacks, weight is the only limit; new
stacks go last),
`Weight` (base + carried), `format_weight`, and `phrase` for messages. An area spell may destroy
items lying in piles (`world/magic/ruin.rs`, `data/ruin.toml`; docs/dev/spells.md "Ruin"). Piles live in
`LevelState::objects` (by position, never empty) with `object_memory` beside them (piles as last
seen, updated by `Perception::remember` on cells where creatures would be seen); inventories in
`Actor::inventory` (spawned from a level's `carry`). The rules know every actor's weight
(`Actor::weight`, `World::actor_weight`); presentation reads only `PerceptionView::objects`
(current pile in sight, else remembered), `inventory`, and `own_weight`: never what creatures
carry. Actions name stacks by **position**, never by letter (#188): `Action::PickUp(Positions)` /
`Action::Drop(Positions)` (`action.rs::Positions`, a set of positions, empty = everything; action
text writes `a`–`z` for positions 0–25, so old replays keep working, and numbers for any position:
`drop:0,2,30`, `equip:body:26`; `position_text` is the one way a position is written) are actions
of their own cost (`pick_up`, `drop`); a refused one takes no time, and no message names a
letter (a screen's letters change with its page). The map grid draws the top object
in place of the terrain glyph (no terrain animation or glow there); look mode names piles.

Item classes (spec §7 "Objects and materials", #189): `data/item_classes.toml`
(`content/item_class.rs`: `ItemClassDef` with `id`, `name` (the header item lists show), `kinds`,
`corpses`, `group` (the word a vault's loot pool names it by with its fellows, `Content::pool_classes`,
`docs/dev/content.md`; never a class's id); `ItemClassId` an index in list order, not saved, so not in the fingerprint) is the
ordered list, ADOM-like: melee weapons, missile weapons, missiles, shields, body armour, helmets,
cloaks, gloves, boots, rings, amulets, light sources, tools, food, potions, scrolls, gems, coins,
miscellaneous. **Every object names its class** (`class`, required; `ObjectDef::class`); a class
lists the magic item kinds (`ItemKind`) of its objects and each kind is of one class only, so an
object's `kind` and `class` cannot disagree (checked on load); corpses take the class with
`corpses = true` (`load_hunger`). The `tools` class lists the `tool` kind: the pick-axe is a base
item (`kind = "tool"`, loot from level 3 at `frequency` 10, so generated general stores stock it
too; hand-placed and kit pick-axes stay plain; docs/dev/magic-items.md "The utility hooks"). Test fixtures that bring their own objects insert
`crate::content::BUILTIN_ITEM_CLASSES` and give each object a `class`. Shops trade by class
(`docs/dev/shops.md`). Presentation reads the class through perception:
`PerceptionView::item_class(stack)` (the base item's, which the player knows of anything they
see) and `by_class(stacks)` (positions grouped by class in class order, each class in the stacks'
own order: the order every item list and the `inventory` dump layer show, `Melee weapons:` header
lines, positions as action text). Storage order never changes: positions stay stable, so replays,
equipment, and action text keep working. The UI's one paged list is `ui/paged_list.rs`
(`docs/dev/screens.md`).

Parts (spec §7 "Objects and materials", #199): an object is made of parts, each of one material
(`content/part.rs`). `materials` in an object file is the ordered list (`PartEntry`: a plain
material id is one unnamed part, the shorthand `materials = ["steel"]`; a table is `{ part, of,
share }`), resolved by `part::resolve` into `ObjectDef::parts` (`Part { part: Option<PartId>,
material, share }`, shares summing to 100, the first the main part, which takes what the others
leave). Part names are
`data/parts.toml` (`PartDef` with `id`, `name`, `adjective`; `PartId` an index in file order,
not saved, so not in the fingerprint; `Content::part_defs`/`part_id`; optional, without it only
unnamed parts). **The main part is what a single material was**: `ObjectDef::material()` returns
it, and every reader of an object's material reads that (later rules may read every part: rust,
fire, silver, #151). Materials gain `precious` (gems and precious metals, shown in names) and
`value` (percent of steel's, default 100; `MaterialDef::check`). Value by shares:
`ObjectDef::value_with(parts, content)` = `value` × `part::worth(parts without a gem)` /
`worth(own parts)`, rounded, plus each gem part's `Gems::value` (below); made of its own parts,
`value`. Presentation and shops go through the stack: `Stack::parts(content)` (a `Cow`: the base
item's parts, or with the item's own in their place, below) and `Stack::value(content)` (what
`Shops::price`/`offer` read). Names:
`part::names(name, plural, parts, ..)` adds the notable parts the written name does not mention
(an `adjective` part as "ruby-pommelled ...", any other joining the main material the name
begins with, "gold and ruby amulet"); `Content::load_objects` composes `ObjectDef::name` and
`plural` with it as each object loads, so every message, list, dump, and screen says the full
name without asking the parts; `ObjectDef::written_name`/`written_plural` keep the names as
written, which `Stack::names` composes again for an item with parts of its own. Descriptions
list every part
(`Content::describe_parts`: "steel blade, leather grip"; `magic/describe.rs`'s first line).
Materials are not hidden: nothing about them waits for identification. Construction lists per
part (`made_of`, #194) are resolved by `part::construction` (below, "Materials and variants").
Tests: `content/part.rs` (the
vocabulary, the forms, shares, naming, worth), `world/items/parts_tests.rs` (built-in objects, names
and descriptions, unidentified items, value by shares, generated loot and creature kits, saves
and replays).

**Instance parts** (#191; `item_parts.rs`): a stack may carry its own parts beside its magic,
`Stack::parts` (field, `Option<Box<[ItemPart]>>`, saved and hashed, left out of the JSON when
none, so other stacks hash as before; read with `Stack::own_parts`, built with
`Stack::with_parts`). An `ItemPart` is a part (`None`: the main part), a material, and a gem's
size and cut (`GemCut`), **all by id** (like affixes: adding content keeps saves). Each takes the
place of the base part of its name (the main part for `None` or the main part's own name),
keeping its share, or comes after the base parts with share 0 (`item_parts::resolve`, which
`Stack::parts` calls). `item_parts::check` is the guard for saves (`Stack::is_valid_for`), level
files, and the shop bill: known ids, a size and cut exactly on parts of a gem material, no part
twice, at most `MAX_OBJECT_PARTS` in all; the gems' object is made of one gem as its main part
(and must carry it), and no other item has a gem as its main part. Stacks join only with the
same parts (`Stack::same_goods`), and a shop's bill counts goods, object and parts
(`docs/dev/shops.md`). Variants (below) are own parts of a non-gem material in place of a base
part: `ItemPart::material`; `check` keeps them on their part's construction list.

**Fletching's objects** (#1021, #1336; `docs/dev/crafting.md` "Arrows and bolts"): `knife` is a
plain weapon of the `daggers` skill, which is what a knife is to `Content::is_knife` (a dagger and a
stiletto count; `short_blades` do not). `brittle_arrow` and `brittle_bolt` break on every hit
(`breaks = 100`), are made of any wood (`made_of = ["wood"]`: the shaft's), and, with no `kind`
or `frequency`, are never in loot or shops; `Content::ammunition`'s lowest id keeps `arrow` and
`bolt` the names of their kinds. The arrow's and the bolt's shaft may be any wood
(`made_of = ["wood"]` in its part's table: only crafting reads it, loot varies the main part
alone). `arrowhead` (class `materials`, 5 g, worth its metal, never loot) is the smith's
intermediate: one object for arrows and bolts, its main part its metal.

Whittle (#1028) works the same `Stack::worked` state as Temper, on a weapon, launcher, shield, or armour whose main part is of a material of `group = "wood"` (`world/crafting/temper.rs`).

**Materials and variants** (spec §7 "Materials", #193, #194). `content/material.rs`:
`MaterialDef` gains `group`, `color`, `weight` (percent of steel's), `to_hit` and `damage`
(`MaterialDef::weapon`: `None` when neither is given, it cannot be a weapon's working part),
`protection` (`None`: cannot be armour), and `rarity` and `gear` (`Found { weight, deeper, level
}`, `Found::at(level)`, capped at `MAX_FOUND`), all optional; `MaterialDef::check` keeps
what the code needs: a `weight` above 0 (`variant::weight` divides by the base material's), a
`rarity` or `gear` only with a group (only grouped materials are drawn), and a gem with no group
(gems are set, never a part's material). `VERY_RARE` (10) is the cap `variant.rs` puts on a
worse-but-dearer material, whatever its data says; `ANY` the group word for every group. Construction lists: `made_of` in a part's table
(`PartTable::made_of`) or the object's own `made_of` for the main part (`ObjectSpec::made_of`),
resolved by `part::construction` in `Content::load_objects` after `ObjectSpec::resolve` (the
object's `ObjectDef::role()`, a `PartRole`: `Weapon` for what has an `attack` or is ammunition,
`Armour` for armour kinds and shields, else `Any`; `PartRole::allows` filters a group's
materials; a material named alone is taken as it is) into `ObjectDef::construction`
(one list per part, sorted by material id, empty when no part has one); read it with
`ObjectDef::made_of(part)` (the list, else only the part's own material).
`ObjectDef::is_jewellery()` (ring and amulet kinds) changes only worth and colour. The math is
`variant.rs`: `weight` (each base part's share scaled by its material's weight over the base
part's, integer, rounded), `weapon` (the main parts' modifier difference), `protection` (ratio,
rounded to the nearest), `color`; `Stack::weight`, `Stack::base_protection` (the description's
and `Stack::protection`'s base), `Stack::fg` (after a gem's colour), and `Mods::local` (to hit
and damage, so every attack builder: melee, off hand, launcher and ammunition, thrown) read
them; value needs nothing new (`value_with`). Names: `part::names` takes `written_for` (the base
item's main material): a main part of another material replaces that material's whole word in
the written name (`with_material`: spaces, hyphens, ends) or goes before it, then the precious
parts are added as before, so the order is [ornament] [material] base. The description
(`magic::describe::lines`) shows damage dice with the material's bonus (`Dice::plus`) and the
scaled Protection; `base_stats(def)` (the wiki's) is the base item's. Generation (`variant::
weights`, `found`, `fit`, `variant_percent`, `draw`, `worse_but_dearer`, `Source::{Loot, Gear {
level, faction }}`, `Source::gear(level)` for none; #1191): the main part's list, weighed in
parts per million (`PER_MILLION`, every material that weighs anything at least 1). The base
material takes all but `variant_percent(level)` (10 % at level 1, half a point a level, 20 % from
21; all of it where no other weighs anything). The others share that by `found`: each material's
`rarity` (loot, item level) or `gear` (creature level) weight, capped at `VERY_RARE` when worse in
a fight than the base material (to hit or damage lower on what strikes, protection lower on
armour with Protection) and dearer, never for jewellery, then scaled by its `fit` for the item's
use (`MaterialDef::fit`, a `Fits` of `Fit::{Good, Poor, Odd}` per use: `weapon` by
`ObjectDef::role` Weapon, `armour` by Armour, `jewellery` by `is_jewellery` first, `other`;
`Fit::scale`: poor ÷ `POOR` (10), odd ÷ `ODD` (100), rounded up). Why per-material fits by use:
the owner's wrong combinations are a material against a kind of item (tin blades, wool rings), the
four uses are what the code already tells apart (`PartRole`, `is_jewellery`), and the default
`good` keeps the data to one line on the twenty-odd materials that need it. A gear source's
faction (`FactionDef::gear`, `data/factions/` `gear = { bone = 55 }`, resolved by
`Factions::resolve_gear` once materials load: known grouped materials, 1 to 100 each, at most
100 in all) takes its percent first for each favoured material on the item's list, the usual draw
the rest; it never adds a material. `gear.rs` reads the creature type's faction (not a legend's
override), `World::loot_of` the actor's (`Actor::faction`); animated bodies use `Source::gear`.
One draw, none for a single material; `magic/loot.rs` step 8 and `World::loot_of`
(`docs/dev/magic-items.md`). Placements
name a material, `"great_sword:bronze"` (`parse_stacks` / `placement`, checked by
`item_parts::check`), and `[[item]]` `parts = [{ of = "bronze" }]` (no size or cut); a unique item
is placed by its id, `"unique:widowmaker"`, once in all content (`docs/dev/magic-items.md`, "Unique
items"). Tests:
`world/creatures/variant_tests.rs` (material checks, construction lists and their refusals, the math,
names and colours, the description, placements and refusals, combat, saves and replays, the
weights and the cap, loot determinism, the built-in tables, a property test over thousands of
loot items and drops across seeds and depths); `world/items/material_draw_tests.rs` (#1191: the base
share and the odd share of every object's weights at every level, sampled loot by use against
the targets (own 85–93 %, poor under 3 %, odd under 0.5 % at levels 1 and 5), the fits of tin
swords and wool rings, the factions' favourites within the list, and rolled skeleton gear).

**Item damage** (#1131, #1168; the dread design §5.8): a material's file sets `resists =
{ acid, rust, fire }` (`content/material.rs::Resists`, percent, every kind given;
`deny_unknown_fields`), its chance to withstand that kind of
`ItemHarm` (`Acid`, `Rust`, `Fire`, and `Work`, a failed temper or whittle, which no material
withstands). A material without it withstands nothing. `world/combat/item_damage.rs` is the one path: `World::withstands(at, harm)`
(the one check every source makes; 0 and 100 draw nothing, else one gameplay draw; a protected
item withstands everything first, drawing nothing) and `World::harm_item(at, harm) -> Harmed::{Withstood, Damaged,
Destroyed}`: withstood, nothing; else an undamaged item is damaged (`Worked::damage(harm)`, its
`by` kept for the word: "corroded", "rusted", "scorched", "damaged"), one damaged already
destroyed (`remove_stacks`, off the equipment too). The sources: a creature's `corrodes`
(`world/combat/corrode.rs`, its `kind` acid, rust, or fire; `docs/dev/creatures.md`'s gimmicks) and a
failed attempt (`world/crafting/temper.rs`, kind `Work`). Ruin (`world/magic/ruin.rs`) is not item damage: it
destroys floor items outright by its own table, which already goes by material and group, but
it honours the same protection.

**Protection** (#1170): `World::item_protected(stack)` (`world/combat/item_damage.rs`, over the free
`protected(content, quest_objects, stack)` that ruin calls with the set gathered once a cast) is
the one rule every source of item damage reads, ruin, corrosion, rust, heat, and a failed temper
or whittle alike: a unique (`Magic::unique`), an item of the `proof` affix effect ("of Proof",
`Effect::Proof`, an item effect read through `Mods::local`, so it works before it is identified),
and `World::quest_objects` (every fixed quest's `hand_in` object, bring or deliver, until the
quest is completed and not repeatable, and an active rolled quest's drawn one). A quest's reward
is protected only when another quest's `hand_in` names it: there is no reward flag
(`item_damage_tests.rs::every_campaigns_quest_objects_are_protected_and_rewards_only_when_needed`
runs over every campaign's quests).
Mithril and adamantium (`data/materials/`) are very rare from item level 20 and 30 (gear from
creature level 30 and 40); the wiki's materials table has the three chances.

**Adamantium, the fighting metal** (#1444): three material fields, read in one place each.
`dice` (`variant::dice`, read in `Stack::attack`, the one place dice are built): a weapon of it
rolls that many more dice of its own size (`Roll::more`), so a melee, thrown, or missile weapon
(not ammunition, which has no dice of its own) of adamantium rolls 2d9 for 1d9. `defence`
(`variant::shield`, added as `Effect::Shield` in `Mods::local`, so Defence, the numbers, and the
item description's "Shield +N" all read it): a shield's Defence × it over the base material's.
`proof` (`magic::worked::proof`): `World::withstands` withstands every `ItemHarm` (`Work` too) with
no draw, `ruin.rs` spares it, and `gear::condition::apply` leaves a piece drawn damaged as made
(its draws unchanged). It is not `protected`: salvage and Set still take it. Armour of it keeps
200 % (`protection`), twice a steel piece's Protection.

**Gems** (spec §7 "Gems", #191): a gem is a material with a `gem` table (`content/gem.rs`,
`GemDef`: `color`, `value` in gold for the plain size and cut, `rarity`, `plural`; a gem is
precious, checked), one file each in `data/materials/` (diamond, ruby, emerald, sapphire, topaz,
heliodor, amethyst, zircon, garnet, smokestone; amber is no gem). `data/gems.toml` (`Gems`,
`Content::gems()`, optional: without it nothing is found or set) names the loose gem's `object`
(`objects/gem.toml`: `*`, class gems, value 0, made of stone, which no gem stack ever is), the
find `chance`, the ordered sizes (`GemSize`: `value` %, `weight` g, loot `find` + `deeper` per
item level) and cuts (`Cut`: `value`, `find`, `deeper`, `rough`), one `plain` of each (small, fair:
`Gems::plain`), and the `[[ornament]]`s per item class (`Ornament`: `part`, `chance`, `colour`);
objects may override the part with `ornament = "tip"` (`ObjectDef::ornament`; checked: a class
must have an ornament, and the part is never the object's main part). The runtime is `gem.rs`:
`names` ("[uncut] [size] [cut] gem", plain ones unsaid), `phrase` (the description's "small
emerald, fine cut"), `color` (`content::dull` for a rough cut, `ROUGH_SATURATION`), and the loot
draws `find` (one chance draw, then `roll`), `ornament` (one chance draw, then `roll` without
rough cuts), `roll` (gem by `rarity` sorted by material id, size and cut by their weights at the
item level). `Stack::gem` says whether a stack is a loose gem; `Stack::names`, `Stack::fg` (the
glyph colour every screen and the map read: a loose gem's, or a ring's gem's by the ornament's
`colour`), `Stack::weight` (a loose gem's size), and `Stack::value` follow from it.
`magic::describe::lines` adds a line per gem part. Level files place loose gems as
`"gem:ruby*3"` (`parse_stacks`, plain size and cut) and any gem with `[[item]]` `parts`
(`dagger:ruby` is refused: only a loose gem names its gem).
Loot, drops, and the item description are in `docs/dev/magic-items.md`. Tests:
`world/items/gem_tests.rs` (data, value, names, naming order, colour, stacking, placements, loot
frequency across seeds, drops, saves and a replay, the general store).

**Carried lights** (spec §5 "Light sources", #643; `world/items/lamp.rs`): an object with a `light`
and a `slot` but no `burns` is a carried light (`ObjectDef::carried_light`; the lantern,
`objects/lantern.toml`, a tool). Its mode is the stack's `light` (`object::LightMode`: unlit,
lit, hooded; saved, left out while unlit, and named after the item, "a lantern (lit)"), kept
wherever the stack goes. `Actor::ready_light` finds the one in the equipment (the Tool slot);
`World::use_tool` (`Action::UseTool`, `use_tool`, `u`) turns it to its next mode, and
`WorldState::carried_light` is the one place its light is read (a talent's `lantern_light`
while lit, the preset's `hooded` while hooded): the light cache's player source. In the pack or
on the ground it gives no light. Tests: `world/items/lamp_tests.rs` (`with_lantern` gives fixture
content one), the light itself in `light_tests.rs`.

**Pieces and breaking** (spec §7 "Pieces and breaking", #484; `piece.rs`): `objects/piece.toml`
(`piece::ID`) is the one object for raw material, `made_of = ["any"]`, its own material steel. A
piece of another material is a stack with that material as its main part (`piece::make`, `None`
for a material without a group); a piece of steel has no own parts. `Stack::names` names any
piece by `piece::names` ("piece of tin"), checked before the plain-object shortcut since a steel
piece has no parts; colour, weight, and value come from the existing variant code and
`value_with`, and same-material pieces join as any stacks made the same way. `object::break_up`
(re-exported from `piece::break_up`; `(stack, content, percent)`, no draws) walks
`Stack::parts`: a gem part gives a loose gem (`Gems::object`, the part's cutting or the plain
one, `count` of them), a grouped material its grams (the base part's share of `def.weight`,
scaled by material weight as `variant::weight` scales it, times the count) summed per material
in first-seen order, then `max(1, round(grams × percent / 100 / one piece's grams))` pieces; a
material without a group and a non-gem part beyond the base parts give nothing. No magic
survives. Tests: `piece.rs` (names, weight, value, colour, stacking, the dagger with a ruby, the
long sword, a stack, paper, the one-piece floor).

**Stock** (#1332; the design: docs/design/professions.md §3, §4): the piece is the professions'
material stock, by weight; no other stock object exists. One piece is 500 g of steel
(`weight = 500`; another material 500 × its `weight` %), item class `materials` (before `gems`
in `item_classes.toml`), and never loot (no `kind`, so no `frequency`: a shop lays it out at the
no-loot weight, 100). `data/stock.toml` (`content/stock.rs`, `Content::stock`, optional) has a
`[[group]]` row per material group: the form word (`name`, `plural`; `of = true` puts the
material after it), which `piece::names` reads ("steel ingot", "lump of amber"; a group without a
row keeps "piece of X"), and optionally the profession that works the group, `skill`, `stat`,
and `verb` together (`Profession`); `gems = { skill, stat, verb }` is the gems' (no group). The
loader refuses a group twice or one no material has, empty words, a partial profession, unknown
skills, and a skill named with two stats. `Stock::profession_of(content, material)` (a gem the
gems' row, else its group's) and `World::profession_of(material) -> Option<(SkillId, Stat)>`
(`world/items/stock.rs`) are the one read: nothing in code names a group or a skill. Jewelcraft
(Dexterity, verb Cut) works stone, resin, and the gems.

**Stock extensions** (#1349; docs/design/recipes.md §3): a `[[group]]` row's `unit` (default 1,
the loader refuses 0) is how many pieces' worth one piece of the group is: `piece::unit(content,
def, stack)` (1 for anything but a piece; `Stock::unit_of`) multiplies `Stack::weight` and
`Stack::value`, so one piece's grams are 500 × `weight` % × `unit` and `count_for` (which weighs
one piece) reads it with no code of its own. Hide is 8 (a leather hide 1.4 kg: a jerkin three
hides, not twenty), cloth 4. `[[material]]` rows (`StockMaterial`) override one material's
`skill` (its stat and verb are those another row gives the skill: flint, stone, and obsidian are
knapped, Woodcraft's, while jade stays Jewelcraft's) or its form word (`name` and `plural`
together: fur's *pelt*); `Stock::profession_of` and `Stock::form_of` (which `piece::names`
reads) take the material's row first. The loader refuses an unknown or groupless material, a
material twice, a row naming nothing, half a form word, and a skill no group or the gems names.
**Sinew** (`materials/sinew.toml`) is a material and one-member group, "a length of sinew"
(Woodcraft): an input (a bow's string), never a part, so no `rarity` or `gear`. **Herbs and
essences are objects, not materials** (class `materials`, stackable, no `level` or `frequency`):
nine herbs (`glowcap`, `moonwort`, `comfrey`, `feverfew`, `henbane`, `eyebright`, `bog_myrtle`,
`vervain`, `mandrake`; `"`, 20 g, `materials = ["plant"]`) and six `essence_of_<element>`
(fire, ice, lightning, air, water, earth; `*` in the element's colour, 50 g, 30 gold, made of
stone as the loose gem is). Not a `herb` material group, because `made_of = ["any"]` on
jewellery is every grouped material and would admit a moonwort ring, and a recipe's fixed input
is plainly an object. The alchemist `always` lays out glowcap and moonwort and buys `materials`;
the general store `never` lays out a herb or an essence (it stocks the class for pieces).
**Magic crystals** are objects of the same kind: six `crystal_<colour>` (black, blue, white, red,
green, gold; `*` in the colour, 20 g, 25 gold, made of stone) and the two rare materials
`heart_crystal` and `soul_fetter` (30 g, 300 gold), caught in salvage and never sold (every shop
that stocks `materials` has them in `never`; docs/dev/enchanting.md).
Gathering is Herbalism's Gather (#1340, `world/skills/gather.rs`, `data/herbs.toml`; docs/dev/skills.md),
and a herbalist's forage finds the surface ones; drops are below (P15).

`piece::count_for(content, grams, material)` =
`max(1, round(grams / one piece's grams))`, the rounding `break_up` shares (`count_of_share`).
`MaterialDef::work` (optional, signed, default 0) is the material's modifier on every salvage
and recipe check (salvage reads it, docs/dev/crafting.md). Shops (`shops.toml`): the smithy and the general
stores stock `materials`; they, the weaponsmith, the armourer, and the fletcher buy them;
Cēaptūn lays out steel and iron ingots in its smithy (a wood billet is worth nothing yet, so no shop trades it). Tests:
`piece.rs` (names with and without a row, the §3 counts, units, sinew and pelts),
`content/stock.rs` (professions, overrides, the loader's refusals), `world/items/stock.rs`
(`profession_of`, Cēaptūn's piles through a save, the shops, herbs and essences, the alchemist).

**Material drops** (#1341, P15; the design: docs/design/recipes.md §9, §10; spec §7 "Material
drops"): a dead creature's body leaves stock on its cell. `data/drops/<tag>.toml`, one file per
creature tag (the file name is the tag; `README.toml` skipped), holds `bodiless` (default false)
and `[[drop]]` rows `{ material | object, count (dice), chance (percent) }`; a creature's own
`[[creature.drops]]` rows (the same shape) add to its tags', and `drops_from_tags = false`
switches its tags' off. `content/drops.rs` parses the files after every creature type is known
(`Content::load_drops`, after `load_splits`) and resolves each type's rows once into
`CreatureDef::drops` (`Drop { goods, count, chance, from }`, the chance **per mille**): every
tag file it carries in path order (a union, not the first tag), then its own; a tag row only for
a type with `corpse` unless the file is `bodiless`; none for an animated type (it carries
`construct` but breaks into its own pieces); each chance × its size's `drops` % (`sizes.toml`,
small 50: a halved 25 % stays exact as 125 ‰). `Content::tag_drops` keeps the files for the
wiki. Loader refusals: neither or both of `material` and `object`, an unknown one, a gem (gems
come by the gems' chance) or a material with no group, bad dice, a chance outside 1 to 100, a
tag no creature carries (a typo would drop nothing), a tag's second file. The roll,
`World::drops_of` (`world/items/material_drops.rs`), runs in `fight.rs::creature_dies` after the
carried gear and `loot_of`, not for a body turned to stone (no corpse) nor a summoned or risen
ally (which leave nothing): each row a draw below 1000 then its dice (a roll below 1 gives
nothing), from the stream `materials/<actor id>`, so the loot (`drop/`) and gem draws and every
generated map stand; pieces by `piece::make`, an object by `Stack::new`, same goods joined in
one stack. Whoever made the kill. The untagged beasts were tagged for it (warg, dire warg,
deer, bat: `beast`; the gorgon `dragon`, as every `combat = "dragon"` type is), which also
makes them count for `tag:beast` and `tag:dragon` kill quests. Wiki: `creature-drops.md`
(`xtask/src/wiki/drops.rs`: each creature's resolved rows, and each tag's as written). Tests:
`world/items/material_drops_tests.rs` (the union and its order, bodiless rows, a spirit, half for
small, the own stream with loot unchanged, one stack, the refusals, every file, the new tags,
the arena's litter).

# Objects, piles, and inventories

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/07b-data.md and spec/06-input-and-ui.md. Keep it current with the code, as CLAUDE.md says.

Objects (spec §5, §6, §7; M11): every object in `data/objects/` (one file per object, named by
its id; a family file only for variants always edited together, `arrows.toml`) defines its glyph,
color, item class (`class`, below), what it is made of (`materials`, its parts, below; materials
in `data/materials/`, one file per material), and weight in grams (a scroll also what reading it
does, `read`: `docs/dev/scrolls.md`)
(`content/object.rs`: `ObjectDef`, `ObjectId`, and the `"id*N"` placement strings;
`content/material.rs`); the tables are directories read in path order, see "Content
directories" in `docs/dev/content.md`. Every creature has a base `weight`. `object.rs` holds the runtime side: `Stack` (object + count; stackable objects join,
others are stacks of one; food knows when it was made, `Stack::died`, and joins whatever its age,
keeping the oldest: `docs/dev/hunger.md`), piles (`add_to_pile`: dropped stacks go on top, index 0 is the top the
map shows), inventories (`add_to_inventory`: any number of stacks, weight is the only limit; new
stacks go last),
`Weight` (base + carried), `format_weight`, and `phrase` for messages. Piles live in
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
`corpses`; `ItemClassId` an index in list order, not saved, so not in the fingerprint) is the
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
leave). The old `material` key is refused with a message naming `materials`. Part names are
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
vocabulary, the forms, shares, naming, worth), `world/parts_tests.rs` (built-in objects, names
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

**Materials and variants** (spec §7 "Materials", #193, #194). `content/material.rs`:
`MaterialDef` gains `group`, `color`, `weight` (percent of steel's), `to_hit` and `damage`
(`MaterialDef::weapon`: `None` when neither is given, it cannot be a weapon's working part),
`protection` (`None`: cannot be armour), and `rarity` and `gear` (`Found { weight, deeper, level
}`, `Found::at(level)`), all optional and checked in `MaterialDef::check` (a gem has no group;
only a material of a group is found). `VERY_RARE` (10) is the cap for a worse-but-dearer
material, `ANY` the reserved group word. Construction lists: `made_of` in a part's table
(`PartTable::made_of`) or the object's own `made_of` for the main part (`ObjectSpec::made_of`),
resolved by `part::construction` in `Content::load_objects` after `ObjectSpec::resolve` (the
object's `ObjectDef::role()`, a `PartRole`: `Weapon` for what has an `attack` or is ammunition,
`Armour` for armour kinds and shields, else `Any`; `PartRole::allows` filters a group's
materials, a material named alone that it refuses is an error) into `ObjectDef::construction`
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
weights`, `draw`, `worse_but_dearer`, `Source::{Loot, Gear(level)}`): the main part's list, each
material's `rarity` (loot, item level) or `gear` (creature level) weight, capped at `VERY_RARE`
when worse in a fight than the base material (to hit or damage lower on what strikes, protection
lower on armour with Protection) and dearer, never for jewellery; one draw, none for a single
material; `magic/loot.rs` step 8 and `World::loot_of` (`docs/dev/magic-items.md`). Placements
name a material, `"great_sword:bronze"` (`parse_stacks` / `placement`, checked by
`item_parts::check`), and `[[item]]` `parts = [{ of = "bronze" }]` (no size or cut); a unique item
is placed by its id, `"unique:widowmaker"`, once in all content (`docs/dev/magic-items.md`, "Unique
items"). Tests:
`world/variant_tests.rs` (material checks, construction lists and their refusals, the math,
names and colours, the description, placements and refusals, combat, saves and replays, the
weights and the cap, loot determinism, the built-in tables, a property test over thousands of
loot items and drops across seeds and depths, and the depth shift).

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
`world/gem_tests.rs` (data, value, names, naming order, colour, stacking, placements, loot
frequency across seeds, drops, saves and a replay, the general store).

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


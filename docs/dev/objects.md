# Objects, piles, and inventories

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/07b-data.md and spec/06-input-and-ui.md. Keep it current with the code, as CLAUDE.md says.

Objects (spec §5, §6, §7; M11): every object in `data/objects/` (one file per object, named by
its id; a family file only for variants always edited together, `arrows.toml`) defines its glyph,
color, item class (`class`, below), what it is made of (`materials`, its parts, below; materials
in `data/materials/`, one file per material), and weight in grams
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
`corpses = true` (`load_hunger`). Test fixtures that bring their own objects insert
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
`ObjectDef::value_with(parts, content)` = `value` × `part::worth(parts)` / `worth(own parts)`,
rounded; made of its own parts, `value`. Presentation and shops go through the stack:
`Stack::parts(content)` (the base item's parts: **the seam for item instances with their own
parts**, metal variants #193 and gems #191, which will store them on the stack like `Magic`, and
then saves change) and `Stack::value(content)` (what `Shops::price`/`offer` read). Names:
`part::names(name, plural, parts, ..)` adds the notable parts the written name does not mention
(an `adjective` part as "ruby-pommelled ...", any other joining the main material the name
begins with, "gold and ruby amulet"); `Content::load_objects` composes `ObjectDef::name` and
`plural` with it as each object loads, so every message, list, dump, and screen says the full
name without asking the parts (an instance with parts of its own will compose from the written
name again, so #193 keeps that beside it). Descriptions list every part
(`Content::describe_parts`: "steel blade, leather grip"; `magic/describe.rs`'s first line).
Materials are not hidden: nothing about them waits for identification. Construction lists per
part (`made_of`, #194) will be checked in `part::resolve`. Tests: `content/part.rs` (the
vocabulary, the forms, shares, naming, worth), `world/parts_tests.rs` (built-in objects, names
and descriptions, unidentified items, value by shares, generated loot and creature kits, saves
and replays).

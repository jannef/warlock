# Objects, piles, and inventories

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/07b-data.md and spec/06-input-and-ui.md. Keep it current with the code, as CLAUDE.md says.

Objects (spec §5, §6, §7; M11): every object in `data/objects/` (one file per object, named by
its id; a family file only for variants always edited together, `arrows.toml`) defines its glyph,
color, material (`data/materials/`, one file per material), and weight in grams
(`content/object.rs`: `ObjectDef`, `ObjectId`, and the `"id*N"` placement strings;
`content/material.rs`); the tables are directories read in path order, see "Content
directories" in `docs/dev/content.md`. Every creature has a base `weight`. `object.rs` holds the runtime side: `Stack` (object + count; stackable objects join,
others are stacks of one; food knows when it was made, `Stack::died`, and joins whatever its age,
keeping the oldest: `docs/dev/hunger.md`), piles (`add_to_pile`: dropped stacks go on top, index 0 is the top the
map shows), inventories (`add_to_inventory`, at most `INVENTORY_SLOTS` = 26, lettered by index),
`Weight` (base + carried), `format_weight`, and `phrase` for messages. Piles live in
`LevelState::objects` (by position, never empty) with `object_memory` beside them (piles as last
seen, updated by `Perception::remember` on cells where creatures would be seen); inventories in
`Actor::inventory` (spawned from a level's `carry`). The rules know every actor's weight
(`Actor::weight`, `World::actor_weight`); presentation reads only `PerceptionView::objects`
(current pile in sight, else remembered), `inventory`, and `own_weight`: never what creatures
carry. `Action::PickUp(Letters)` / `Action::Drop(Letters)` (`Letters` = a set of `a`–`z`, empty
= everything) are actions of their own cost (`pick_up`, `drop`); a refused one takes no time. The map grid draws the top object
in place of the terrain glyph (no terrain animation or glow there); look mode names piles.

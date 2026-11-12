# What creatures leave

When a creature dies, its body may leave crafting stock on its cell, besides the gear it carried
and any [loot](magic-items.md#finding-them): a wolf's hide and a shard of its bone,
a kobold's flint, a skeleton's bones, an elemental's essence. You pick these up like anything
else. They are [pieces](materials.md#pieces) of a material, or [herbs and
essences](materials.md#herbs-and-essences), listed under **Materials**.

- It does not matter who made the kill. A creature killed by another creature, or by a trap,
  leaves the same.
- What a creature leaves goes by its [sort](quests.md#sorts-of-creatures): every sort it belongs
  to adds its own chances (a hobgoblin is a goblin and a hobgoblin, so it rolls both). Some kinds
  add chances of their own: a boar's tusks give horn, a bear's pelt fur, a spider's shell chitin.
- Each chance is rolled on its own, so one body can leave several things, or nothing. Pieces of
  the same material join one stack.
- Only a creature that leaves a corpse leaves stock, unless its sort says otherwise: a
  skeleton's bones and an elemental's essence come without a body. A ghost leaves nothing.
- A **small** creature (a rat, a kobold) has half the chance of each.
- A creature turned to stone leaves none, and neither does a summoned creature, which leaves
  nothing at all.

## By creature

Each creature that can leave anything, with how many of each and the chance, as rolled (a small
creature's already halved). "From" is the sort (or the creature itself) each chance comes from.

{{#include generated/creature-drops.md}}

## By sort

The chances of each sort as written, before a small creature halves them. "Without a body too"
is yes for the sorts whose chances apply to a creature that leaves no corpse.

{{#include generated/drops-by-tag.md}}

# Objects and your pack

Weapons, armor, tools, potions, scrolls, food, stones and gems, coins, rings, and amulets lie about
the world. You can pick them up, carry them, and put them down again. They do nothing else yet:
you cannot wield, wear, drink, read, or eat them.

## What an object is

Every object has its own glyph and color, a material, and a weight. The glyph tells you what kind
of thing it is, as in ADOM:

| Glyph | Kind |
|:---:|---|
| `)` | weapon |
| `}` | missile weapon (a bow) |
| `/` | missile (arrows) |
| `[` | armor |
| `(` | tool |
| `!` | potion |
| `?` | scroll |
| `%` | food |
| `*` | stone or gem |
| `$` | coins |
| `=` | ring |
| `"` | amulet |

Some glyphs are shared with terrain. Wildflowers are `*` too, and glowing fungus and tall grass
are `"`. Color and [look mode](controls.md#look-mode) tell them apart.

Weights are in grams. They read `450 g` below a kilogram and `3.5 kg` above. Some objects
**stack**: coins, arrows, candles, potions, scrolls, food, and stones and gems lie together as
one stack, such as "12 gold coins" (the "Stacks" column below). Everything else is a stack of one
each: three daggers are three stacks.

## Piles

Objects lie in **piles**, any number to a cell. The map shows the top object of a pile in place of
the floor, lit by the light around it like the floor would be, and under any creature standing
there. The last thing dropped on a pile goes on top. Objects lie only where you can walk.

When you step onto a pile, the message line says what is there: `You see here a dagger.` for a
single stack, `There are several objects here.` for more. [Look mode](controls.md#look-mode)
(`x`) names what lies on any cell you see or remember: up to three stacks by name ("a dagger and
12 arrows"), more as a count ("17 objects").

You see objects where you would see a creature: on cells in your [line of sight](sight.md) that
are lit enough for your eyes, or within your darkvision. Your own cell is always visible, so you
always know what lies under you. A pile in the dark stays unseen until you light it or come close
enough.

## Remembering piles

The game remembers each pile as you last saw it, the same way it remembers terrain (see
[Sight and memory](sight.md#memory)). Remembered piles are drawn dim like the rest of what you
remember. If a pile changes while you are not looking, you still remember it as it was. Once you
see the cell again, your memory catches up. If the pile is gone, it is gone from memory too.

## Picking up

Stand on a pile and press `,` or `g`. A single stack is picked up at once. Where several lie, a
screen lists them:

- each stack has a letter, `a` for the top of the pile, with its glyph, name, and weight;
- press a letter to mark or unmark that stack (`+` marked, `-` not), or `,` to mark everything
  (again to unmark everything);
- the arrow keys move the `>` cursor, and Space marks the cursor's row;
- `Enter` picks up everything marked, or the cursor's row if nothing is marked;
- `Esc` cancels.

On this screen every letter names a stack, even `h`, `j`, `k`, and `l`.

Picking up takes one turn, however many stacks you take. With nothing here, you are told so and no
time passes.

## Dropping

Press `d` to choose what to drop. The screen works like the pick-up screen and lists what you
carry. Dropped objects go on top of the pile where you stand, and stackable ones join a stack of
the same object already there. Dropping takes one turn.

You cannot close a door while objects lie in its doorway.

## Your pack

Press `i` to see what you carry. Nothing changes and no time passes; `i` or `Esc` closes the
screen. Stacks are lettered in the order you picked them up. Picking up a stackable object you
already carry adds to that stack, and dropping a stack moves the later letters up.

{{#include generated/pack.md}}

When every letter is taken, you cannot pick up anything new, though more of a stackable object you
already carry still fits. The message says `You have no room for more.`

The inventory screen lists each stack's weight, then:

- **Carrying**: the weight of everything in your pack;
- **Your weight**: your own weight plus what you carry.

Weight has no effect yet: you can carry as much as fits in your pack, and it does not slow you.
Creatures have weights too, and some carry objects, but you cannot see what they carry or what
they weigh.

## The objects

{{#include generated/objects.md}}

## Materials

Every object is made of a material. Materials only have a name for now.

{{#include generated/materials.md}}

# Objects and your pack

Weapons, armor, tools, potions, scrolls, food, stones and gems, coins, rings, and amulets lie about
the world. You can pick them up, carry them, and put them down again, and wear and wield weapons,
armor, rings, and amulets (see [Equipment](equipment.md)). Weapons and armour fight and protect
(see [Combat](combat.md)), some of them are magic (see [Magic items](magic-items.md)), and
everything you carry weighs on you (see [Burden](burden.md)). Food can be eaten (see [Food and
corpses](food.md)), but you cannot drink or read anything yet.

## What an object is

Every object has its own glyph and color, a class, what it is made of (see
[Materials](#materials)), a weight, and a worth in gold (what [shops](shops.md) price it from).
The glyph hints at what kind of thing it is, as in ADOM:

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

### Item classes

Every object belongs to an **item class**, as in ADOM: what kind of thing it is. Every list of
objects (your pack, a pile, what to drop, eat, throw, or put on) groups them by class, in the
order below, each class under a heading with its name; within a class they stay in the order
you picked them up (or the order of the pile). [Shops](shops.md) sell and buy by class. You
know the class of anything you see, even of a magic item you have not identified.

{{#include generated/item-classes.md}}

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
screen lists them, grouped by class (see [Lists of objects](#lists-of-objects)):

- each stack has a letter, `a` for the first on the page, with its glyph, name, and weight;
- press a letter to mark or unmark that stack (`+` marked, `-` not), or `,` to mark everything
  (again to unmark everything);
- the arrow keys move the `>` cursor, and Space marks the cursor's row;
- `>` and `<` (or `PageDown` and `PageUp`) turn the pages of a long list; what you marked stays
  marked on every page;
- `Enter` picks up everything marked, or the cursor's row if nothing is marked;
- `Esc` cancels.

On this screen every letter names a stack, even `h`, `j`, `k`, and `l`.

Picking up is one action (6 seconds), however many stacks you take. With nothing here, you are told so and no
time passes. In a [shop](shops.md), what you pick up is not yours until you pay for it.

## Dropping

Press `d` to choose what to drop. The screen works like the pick-up screen and lists what you
carry. Dropped objects go on top of the pile where you stand, and stackable ones join a stack of
the same object already there. Dropping is one action too. Something you wear or wield is taken
off first, which takes its own time as well (see [Equipment](equipment.md)). In a
[shop](shops.md), dropping puts back what you have not paid for and offers to buy your own
things.

You cannot close a door while objects lie in its doorway.

## Your pack

Press `i` to see what you carry. Nothing changes and no time passes; `Esc` closes the screen, and
so does `i` unless it is the letter of something you carry. Press a stack's letter to look at it:
its description says what it is and does (see [Magic items](magic-items.md#looking-at-an-item)).
Your pack holds as many stacks as you can carry: only their weight limits you (see
[Burden](burden.md)). Picking up a stackable object you already carry adds to that stack. Magic
and rare items show their names in blue and yellow; see [Magic items](magic-items.md).

{{#include generated/pack.md}}

## Lists of objects

Your pack, a pile to pick up from, and every other list of objects you choose from (what to drop,
throw, or eat, what to put on in an equipment slot) work the same way:

- stacks are grouped by [item class](#item-classes), each class under a heading (headings have
  no letter);
- the list shows a page at a time, as many rows as fit your window and at most 26 stacks, and
  the frame says which page you are on (`Page 2/3`);
- the letters `a` to `z` name the stacks of the page you see and start again at `a` on every
  page; a class that goes on over the next page shows its heading there again;
- `>` or `PageDown` turns to the next page, `<` or `PageUp` to the one before (you can change
  these keys in Controls); the arrow keys move the `>` cursor from page to page.

The inventory screen lists each stack's weight, marks what you wear or wield ("(being worn)",
"(wielded)", and so on; see [Equipment](equipment.md)), then:

- **Carrying**: the weight of everything in your pack, worn and wielded things included, and your
  burden if you have one;
- **Your weight**: your own weight plus what you carry;
- the weights at which you become burdened, strained, and overloaded.

What you carry has a weight limit, from your Strength and Toughness: carry too much and
everything you do is slower, until, overloaded, you cannot move. See [Burden](burden.md).
Creatures have weights too, and some carry objects, but you cannot see what they carry or what
they weigh, only what they wear and wield.

## The objects

{{#include generated/objects.md}}

## Materials

Every object is made of one or more **parts**, each of one material: most are of one material
throughout (a steel dagger), some of several (a long sword's steel blade and leather grip, a gold
amulet's chain and the ruby in its setting). The first part is the main one: it is the material
the object is mostly made of, and the one the table above names first.

You can always see what something is made of, even a magic item you have not identified. An
item's name shows its **gems and precious metals** (the precious materials below) that it does
not mention already: a gold amulet set with a ruby is a *gold and ruby amulet*, and a long sword
with a ruby pommel is a *ruby-pommelled long sword*. Plain parts, like a leather grip, show
only when you look at the item (see [Magic items](magic-items.md#looking-at-an-item)), which lists
every part.

**Gems** are materials too, each with a size and a cut (see [Gems](gems.md)): a loose gem is made
of its gem alone, and an item found set with a gem has it as one of its parts, which adds the
gem's worth to the item's.

Every other material has its properties on one scale, steel's: its **weight** (how heavy an item
of it is against the same item of steel: metals by their real densities, the rest by what an
item of them weighs), what it does as a weapon's blade or head (**to hit** and **damage**),
how much of an armour's Protection an armour of it keeps (**armour**), and its **worth**. A
material with no weapon figures cannot be a weapon's working part (no leather sword), one with
no armour figure cannot be armour (no flint mail). Materials come in **groups**: metal, wood,
bone (horn, ivory, chitin, and dragonbone with it), stone, hide, cloth, glass, and resin (amber).

{{#include generated/materials.md}}

### Items of other materials

Most items can be found made of another material than their usual one: *a bronze long sword*,
*a duralumin chain mail shirt*, *a flint spear*, *a dragonbone quarterstaff*, *a silk cloak*.
What an item may be made of is its **construction list**, below, by group or by material; loot
and creatures never make anything else, so there are no leather swords or wooden chain mail.
Rings and amulets may be made of any material of a group (never paper or wax). Rocks, food,
potions, scrolls, coins, gems, and keys are only ever what they are.

Against its usual material, an item of another one:

- **weighs** more or less: each of its parts weighs its share of the item's weight times its
  material's weight over the usual one's (a bronze long sword's blade is 112% as heavy, its
  leather grip the same);
- **strikes** better or worse: to hit and damage change by the difference between the two
  materials' weapon figures (a bronze long sword: +0 to hit, −1 damage against steel's), for
  every blow, throw, and shot made with it, arrows and bolts by their heads;
- **protects** more or less: armour keeps its Protection times its material's armour figure
  over the usual one's, rounded to the nearest (an iron chain mail shirt's Protection 3 is 1
  in tin, 4 in chromium);
- is **worth** more or less, by its parts' shares and their materials' worth, as every item of
  other parts (a gold long sword is worth about nineteen steel ones);
- is drawn in its material's **colour** (a ring set with a gem keeps its gem's colour).

Rings and amulets change only in worth and colour. An item's name says its material when it is
not the usual one, after any gem and before the item: *a Sharp emerald-pommelled bronze long
sword of the Bear*; a material the name already says gives way (*an iron helm* of bone is *a
bone helm*, *a pair of iron-shod boots* of bronze *a pair of bronze-shod boots*). Like every
material, it shows at once, identified or not; its damage and Protection show when you look at
the item, its to-hit does not.

{{#include generated/construction.md}}

**How often.** Loot draws an item's material from its list by how often each material is
found at its depth: primitive metals (copper and bronze, then iron and tin) are the common ones
near the surface, and steel becomes the usual metal only deeper. What a creature carries, and
so drops, is drawn by the creature's level instead: kobolds' gear is mostly copper, bronze,
iron, bone, flint, and hide, and steel comes only with tougher creatures. The weights are
relative among the materials an item may be made of (its usual one too), and a material worse
in a fight than the item's usual one but dearer (gold, silver, aluminium on a sword; ivory on a
dagger) is found at most at weight 10, very rare, whatever its own weight says; rings and
amulets are worth only, so that never applies to them. Chromium, nicrosil, dragonbone, and
dragonscale appear only deep. Items placed by hand in the world's made places are what they are
placed as.

{{#include generated/material-finds.md}}

A long sword and a chain mail shirt in each metal:

{{#include generated/variants.md}}

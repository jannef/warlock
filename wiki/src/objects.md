# Objects and your pack

Weapons, armor, [tools](#class-tools), [potions](potions.md), [scrolls](scrolls.md),
[food](food.md), stones and [gems](gems.md), [coins](#class-coins), [rings](#class-rings), and
[amulets](#class-amulets) lie about the world. You can pick them up, carry them, and put them
down again. Weapons, armor, rings, and amulets can be [worn and
wielded](equipment.md#what-fits-where) (see [Equipment](equipment.md)). Food can be eaten (see
[Food and corpses](food.md)), potions drunk, and scrolls read.

Weapons and armour fight and protect (see [Combat](combat.md)). Some of them are magic, and so
are some tools, the pick-axe among them (see [Magic items](magic-items.md)). Everything you
carry weighs on you (see [Burden](burden.md)). What things are made of, and why it matters, is
on its own page: [Materials](materials.md).

## What an object is

Every object has its own glyph and color, a [class](#item-classes), the
[materials](materials.md#materials) it is made of, a weight, and a worth in
[gold](shops.md#money). [Shops](shops.md) set their prices from the worth. The glyph hints at
what kind of thing it is:

| Glyph | Kind |
|:---:|---|
| `)` | [weapon](#class-melee-weapons) |
| `}` | [missile weapon](#class-missile-weapons) (a bow) |
| `/` | [missile](#class-missiles) ([arrows](ammunition.md)) |
| `[` | armor |
| `(` | [tool](#class-tools) |
| `!` | [potion](potions.md) |
| `?` | [scroll](scrolls.md) |
| `%` | [food](food.md) |
| `*` | stone or [gem](gems.md) |
| `,` | a piece of some material |
| `$` | [coins](#class-coins) |
| `=` | [ring](#class-rings) |
| `"` | [amulet](#class-amulets) |

Some glyphs are shared with terrain. Wildflowers are `*` too, and glowing fungus and tall grass
are `"`. Color and [look mode](controls.md#look-mode) tell them apart.

### Item classes

Every object belongs to an **item class**: the kind of thing it is. Every list of objects (your
pack, a pile, what to drop, eat, throw, or put on) groups them by class, in the order below, with
each class under a heading. Within a class, things stay in the order you picked them up, or in
the order of the pile. [Shops](shops.md) sell and buy by class. You always know the class of
anything you see, even of a [magic item](magic-items.md) you have not
[identified](magic-items.md#unidentified-items).

{{#include generated/item-classes.md}}

Weights are in grams. They read `450 g` below a kilogram and `3.5 kg` above.

Some objects **stack**: coins, [arrows](ammunition.md), candles, [potions](potions.md),
[scrolls](scrolls.md), [food](food.md), and stones and [gems](gems.md) lie together as one
stack, such as "12 gold coins" (see the "Stacks" column below). Everything else is a stack of
one, so three daggers are three stacks.

## Piles

Objects lie in **piles**, any number to a cell, and only where you can walk. The map shows the
top object of a pile in place of the floor. It is lit by the [light](light.md#what-light-is)
around it as the floor would be, and drawn under any [creature](creatures.md) standing there.
The last thing dropped on a pile goes on top.

When you step onto a pile, the [message line](the-screen.md#messages) tells you what is there:
`You see here a dagger.` for a single stack, `There are several objects here.` for more.

If you arrive on a special feature of the ground, such as an
[altar](altars-forges-springs.md#forgotten-altars) or a faint rune, it is named first, once each
time you come to it: `There is an altar here.` A feature you cannot stand on, such as a [worn
statue](terrain.md#terrain-statue), is named when you come beside it, and not again while you
walk around it. Whatever is written on it is read out after.

[Look mode](controls.md#look-mode) (`l`) names what lies on any cell you see or remember: up to
three stacks by name ("a dagger and 12 arrows"), or a count for more ("17 objects").

You see objects where you would see a [creature](creatures.md): on cells in your [line of
sight](sight.md) that are [lit](light.md#what-light-is) enough for your eyes, or within your
[darkvision](vision.md#the-four-types). Your own cell is always visible, so you always know what
lies under you. A pile in the dark stays unseen until you [light](light.md#light-sources) it or
come close enough.

## Remembering piles

The game remembers each pile as you last saw it, the same way it remembers [terrain](terrain.md)
(see [Sight and memory](sight.md#memory)). Remembered piles are drawn dim, like the rest of what
you remember. If a pile changes while you are not looking, you still remember it as it was. Once
you see the cell again, your memory catches up. If the pile is gone, it is gone from memory too.

## Picking up

Stand on a pile and press `,` or `;`. A single stack is picked up at once. Where several lie, a
screen lists them, grouped by [class](#item-classes) (see [Lists of objects](#lists-of-objects)):

- Each stack has a letter (`a` for the first on the page), its glyph, its name, and its weight.
- Press a letter to mark or unmark that stack (`+` marked, `-` not). Press `,` to mark
  everything, and again to unmark everything.
- The arrow keys move the `>` cursor, and Space marks the cursor's row.
- `>` and `<` (or `PageDown` and `PageUp`) turn the pages of a long list. What you marked stays
  marked on every page.
- `Enter` picks up everything marked, or the cursor's row if nothing is marked.
- `Esc` cancels.

On this screen every letter names a stack, even `h`, `j`, `k`, and `l`.

Picking up is one [action](turns.md#what-actions-cost) (6 seconds), however many stacks you
take. If there is nothing here, you are told so and no time passes. In a [shop](shops.md), what
you pick up is not yours until you [pay for it](shops.md#buying).

## Dropping

Press `d` to choose what to drop. The screen works like the pick-up screen and lists what you
carry. Dropped objects go on top of the pile where you stand, and stackable ones join a stack of
the same object already there.

Dropping is one action too. Something you wear or wield is taken off first, which takes its own
time as well (see [Equipment](equipment.md)). In a [shop](shops.md), dropping puts back what you
have not paid for, and the keeper offers to buy your own things.

You cannot close a [door](doors.md) while objects lie in its doorway.

## Your pack

Press `I` to see what you carry. Nothing changes and no time passes. `Esc` closes the screen, and
so does `I`. Press a stack's letter to look at it: the description says what it is and what it
does (see [Magic items](magic-items.md#looking-at-an-item)).

Your pack holds as many stacks as you can carry. Only their weight limits you (see
[Burden](burden.md)). Picking up a stackable object you already carry adds it to that stack.
Magic items show their names in blue and [rare](magic-items.md#rarity) items in yellow (see
[Magic items](magic-items.md)).

{{#include generated/pack.md}}

## Lists of objects

Your pack, a pile to pick up from, and every other list you choose objects from (what to drop,
throw, or eat, what to put on in an equipment slot) all work the same way:

- Stacks are grouped by [item class](#item-classes), each class under a heading. Headings have
  no letter.
- Weapons and armour show what they give after their name: "a long sword (+0 hit, 1d9)", "a
  leather jerkin (Dv +1, Pv 1)" (see [Equipment](equipment.md#numbers)).
- The list shows one page at a time: as many rows as fit your window, and at most 26 stacks. The
  frame says which page you are on (`Page 2/3`).
- The letters `a` to `z` name the stacks on the page you see, starting again at `a` on every
  page. A class that runs over onto the next page shows its heading there again.
- `>` or `PageDown` turns to the next page, `<` or `PageUp` to the one before (you can change
  these keys in Controls). The arrow keys move the `>` cursor from page to page.

The inventory screen lists each stack's weight and marks what you wear or wield ("(being worn)",
"(wielded)", and so on; see [Equipment](equipment.md)). Below the list it shows:

- **Carrying**: the weight of everything in your pack, worn and wielded things included, and
  your [burden](burden.md#burden-1) if you have one;
- **Your weight**: your own weight plus what you carry;
- the weights at which you become [burdened, strained, and overloaded](burden.md#burden-1).

What you can carry is limited by your [Strength](stats.md#stat-strength) and
[Toughness](stats.md#stat-toughness). Carry too much and everything you do is slower, until you
are [overloaded](burden.md#burden-1) and cannot move. See [Burden](burden.md).
[Creatures](creatures.md) have weights too, and some carry objects, but you cannot see what they
carry or what they weigh, only what they wear and wield.

## The objects

{{#include generated/objects.md}}

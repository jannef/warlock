# Scrolls

A scroll (`?`) holds one spell or charm. Press `r` to **read** one: a list of the scrolls you
carry opens; choose one with its letter (or the arrows and `Enter`). Reading takes a turn and
uses the scroll up.

{{#include generated/scrolls.md}}

## Unknown scrolls

A scroll you have never read shows by a **label** instead of its name: "a scroll labelled
ZUNBAR", in the same plain colour as every other unknown scroll. Each kind of scroll has its own
label, made up anew for each character and kept for their whole life (two scrolls labelled
ZUNBAR are always the same kind). The label shows everywhere: in your pack, on the floor, in a
shop, in look mode, and in what you are told.

You learn what a kind of scroll is, for good, in one of four ways:

- **Read one.** "You read the scroll labelled ZUNBAR." "It is a scroll of fireball." From then
  on every scroll of fireball shows by its name. Reading an unknown scroll is a gamble: a spell
  scroll is aimed first, and a fireball aimed near you may catch you too.
- **The scroll of identify**, read on it (below): the scroll of identify is known once you read
  your first one.
- **An appraisal**: Factor Wulfrun in Cēaptūn identifies everything you carry for 100 gold per
  unidentified stack (see [Services](npcs.md#services)).
- **Your starting kit**: you know the scrolls you start with.

Knowing what a scroll is does not teach you its spell: a scroll you know by name is still only a
scroll, but once you know it you may study it to learn its spell for good (see
[Magic](magic.md#learning-spells-from-scrolls)). The scroll of light cannot be read, and is known
on sight. A shop's price for an unknown scroll is its true price.

## Spell scrolls

The scrolls of fireball, lightning, and frost cast the same spells as the
[fireball, lightning, and frost traps](traps.md), with the same rules and damage, but **you
aim them**. After choosing the scroll a cursor appears on the map, as for a shot (see
[Controls](controls.md#aiming)): it starts on your last target if it is still there, else on the
nearest hostile creature you see, else on you; `Tab` moves it from creature to creature, the
movement keys move it anywhere. `Enter` (or `r` again) reads the scroll at the cursor, `Esc`
puts it away unread. A cell out of reach is marked "Out of range.", and the scroll is not used.

- **Fireball** and **frost** fly along the dotted line, like a missile, and burst on the cell you
  aimed at, or on the first creature in the way, or just short of a wall. The burst catches
  everyone within its radius, you too if you stand that close: when you aim so near that it may,
  a window asks first (`The fireball may catch you too. Read the scroll of fireball anyway?`),
  with **Read it** (`y`), **Aim again** (`n`), and **Don't read it** (`Esc`, chosen at first);
  with the [talent](talents.md) Careful reader your own bursts never catch you, and nothing asks.
  A
  fireball blows apart the softer walls it reaches and melts ice (whoever stands on it falls in);
  frost freezes the water it reaches into ice that holds a laden person (see
  [Temperature and ice](temperature.md)).
- **Lightning** runs from you along the line through the cursor and on, striking everyone on
  it, until a wall or the end of its reach. It never strikes you.

No spell rolls to hit: it hurts everyone it reaches. Unlike a trap's, a spell you read is
**yours**: what it kills earns you [experience](experience.md) and counts for your
[quests](quests.md), and a shopkeeper it hurts turns on you. A scroll that kills you is named on
the death screen ("Killed by their own scroll of fireball"). The explosion and the thunder are
heard like any noise (see [Hearing](hearing.md)). Spells cannot be read on the overworld.

Reading a scroll uses it up. Scholars do better (see [Talents](talents.md)): Scroll savant reads
in half the time, and with Spell recall one read in four leaves the scroll as it was ("The
writing does not fade from the scroll.").

## The scroll of identify

Choose the scroll of identify and a second list opens: the unidentified items you carry (see
[Magic items](magic-items.md)) and the scrolls you carry that show only their label. Choose one,
and you know what it is from then on: "This is a Sharp dagger of the Lynx!", "This is 2 scrolls
of lightning!" (and every scroll of lightning shows by its name). With nothing unidentified, the
list says so, and the scroll is kept.

The scroll of identify is the commonest scroll: it turns up more often than any other in loot,
and every general store in the dungeons has a stack of them.

## Where scrolls are

- **Loot**: scrolls lie about as loot on the floors of generated dungeon levels (see [Dungeons and danger](dungeons.md)), from the item
  level in the table above on. They are never magic, so vaults and creatures that leave only
  magic things never leave one.
- **Shops**: a general store sells and buys scrolls (see [Shops](shops.md)); one in the
  dungeons always has scrolls of identify among its goods, and so does Ælfric's in Cēaptūn.


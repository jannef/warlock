# Scrolls

A scroll (`?`) holds one [spell](magic.md#the-spells) or charm. Press `r` to **read** one: a list of the scrolls you
carry opens; choose one with its letter (or the arrows and `Enter`). Reading takes a [turn](time.md#what-takes-time) and
uses the scroll up.

{{#include generated/scrolls.md}}

## Unknown scrolls

A scroll you have never read shows by a **label** instead of its name: "a scroll labelled
ZUNBAR", in the same plain colour as every other unknown scroll. Each kind of scroll has its own
label, made up anew for each character and kept for their whole life (two scrolls labelled
ZUNBAR are always the same kind). The label shows everywhere: in your [pack](objects.md#your-pack), on the floor, in a
[shop](shops.md), in [look mode](controls.md#look-mode), and in what you are told.

You learn what a kind of scroll is, for good, in one of four ways:

- **Read one.** "You read the scroll labelled ZUNBAR." "It is a scroll of fireball." From then
  on every scroll of fireball shows by its name. Reading an unknown scroll is a gamble: a spell
  scroll is aimed first, and a [fireball](magic.md#spell-fireball) aimed near you may catch you too.
- **The [scroll of identify](#the-scroll-of-identify)**, read on it (below): the scroll of identify is known once you read
  your first one.
- **An appraisal**: Factor Wulfrun in Cēaptūn identifies everything you carry for 100 [gold](shops.md#money) per
  [unidentified](magic-items.md#unidentified-items) [stack](objects.md#piles) (see [Services](npcs.md#services)).
- **Your [starting kit](starting-kit.md)**: you know the scrolls you start with.

Knowing what a scroll is does not teach you its spell: a scroll you know by name is still only a
scroll, but once you know it you may study it to learn its spell for good (see
[Magic](magic.md#learning-spells-from-scrolls)). A scroll that cannot be read (a sealed letter) has
no label and is known on sight. A [shop](shops.md#prices)'s price for an unknown scroll is its true price.

## Spell scrolls

Every [spell](magic.md#the-spells) comes on a scroll of its own: a scroll of [magic missile](magic.md#spell-magic-missile), of
[haste](magic.md#spell-haste), of [web](magic.md#spell-web). Reading one casts its spell once, as if you cast it yourself (its [shape](magic.md#shapes), its
resolution, its [damage](combat.md#damage)), as written in the [spell's row](magic.md#the-spells), at the rung the scroll's row says
(the table's "Casts at"; the base for most): your stats do not grow a scroll's damage or
healing (its reach and how long it lasts are yours, as for your own casts);
[Scroll adept](talents.md#talent-scroll-adept) takes it a rung up. It is **free of [mana](mana.md#spending-it)**: anyone can read one, a [warrior](classes.md#warrior) too. A spell on yourself ([Haste](magic.md#spell-haste), [Barkskin](magic.md#spell-barkskin),
[Blessing](magic.md#spell-blessing)) is read at once; any other is aimed first, as below; [Mend](magic.md#spell-mend) and [Cure wounds](magic.md#spell-cure-wounds) are aimed at
the creature beside you to heal, or at yourself. Scrolls of lower [spell levels](magic.md#spell-levels) are found sooner and
more often: cantrips from the first floors, spells of level 11 and 12 from [item level](magic-items.md#base-items) 6 (the table
above), and [general stores](shops.md) lay out low levels more often than high ones.

The scrolls of [fireball](magic.md#spell-fireball) (a rung up: 3d6), [lightning](magic.md#spell-lightning-bolt) (two rungs up: 4d6), and
[frost](magic.md#spell-frost) (its base: 2d6) cast their spells as any spell scroll does, reaching as far as you cast the
spell. An aimed scroll is aimed: after choosing the scroll a cursor appears on the map, as for a [shot](ranged.md#shooting-and-throwing) (see
[Controls](controls.md#aiming)): it starts on your last target if it is still there, else on the
nearest [hostile](creatures.md#factions) [creature](creatures.md) you see, else on you; `Tab` moves it from creature to creature, the
movement keys move it anywhere. `Enter` (or `r` again) reads the scroll at the cursor, `Esc`
puts it away unread. A cell out of reach is marked "Out of range.", and the scroll is not used.

- **Fireball** and **frost** fly along the dotted line, like a [missile](ranged.md#where-a-missile-goes), and burst on the cell you
  aimed at, or on the first creature in the way, or just short of a wall. The burst catches
  everyone within its radius, you too if you stand that close: when you aim so near that it may,
  a window asks first (`The fireball may catch you too. Read the scroll of fireball anyway?`),
  with **Read it** (`y`), **Aim again** (`n`), and **Don't read it** (`Esc`, chosen at first);
  with the [talent](talents.md) [Scroll savant](talents.md#talent-scroll-savant) your own bursts never catch you, and nothing asks.
  A
  fireball blows apart the softer [walls](terrain.md#walls) it reaches and [melts ice](temperature.md#walking-on-ice) (whoever stands on it [falls in](temperature.md#falling-in));
  frost freezes the [water](terrain.md#water) it reaches into ice that holds a laden person (see
  [Temperature and ice](temperature.md)).
- **Lightning** runs from you along the line through the cursor and on, striking everyone on
  it, until a wall or the end of its reach. It never strikes you.

These three never roll [to hit](combat.md#hitting): they hurt everyone they reach (a spell scroll of a ray, such
as [ray of frost](magic.md#spell-ray-of-frost), rolls to hit as the spell does, with your Dexterity). Unlike a trap's, a spell you read is
**yours**: what it kills earns you [experience](experience.md) and counts for your
[quests](quests.md), and a [shopkeeper](shops.md#theft) it hurts turns on you. A scroll that kills you is named on
the [death](combat.md#death) screen ("Killed by their own scroll of fireball"). The explosion and the thunder are
heard like any noise (see [Hearing](hearing.md)). Spells cannot be read on the [overworld](overworld.md#travel).

Reading a scroll uses it up. Scholars do better (see [Talents](talents.md)): [Scroll savant](talents.md#talent-scroll-savant) reads
in half the time (with the [wizard](classes.md#wizard)'s Scroll adept as well, in a quarter: no less), and at its
second rank one read in four leaves the scroll as it was ("The writing does not fade from the
scroll."). A scroll you study is always used up, Scroll savant or not.

## The scroll of identify

Choose the scroll of identify and a second list opens: the unidentified items you carry (see
[Magic items](magic-items.md)), the scrolls you carry that show only their label, and the wands
and rods you do not know (a wand's charges too; see [Magic](magic.md#wands)). Choose one, and you
know what it is from then on: "This is a Sharp dagger of the Lynx!", "This is 2 scrolls of
lightning!" (and every scroll of lightning shows by its name). With nothing unidentified, the
list says so, and the scroll is kept.

Reading it casts the spell **[Identify](magic.md#spell-identify)** for you, free of [mana](mana.md#spending-it), and studying it may teach you the
spell for good (see [Magic](magic.md#learning-spells-from-scrolls)); cast for [mana](mana.md#how-much), Identify is
dear (60 mana), so only an experienced caster can afford it (see
[Magic](magic.md#what-a-spell-costs)).

The scroll of identify is the commonest scroll: it turns up more often than any other in [loot](magic-items.md#finding-them),
and every [general store](shops.md) in the [dungeons](dungeons.md) has a [stack](objects.md#piles) of them.

## Where scrolls are

- **Loot**: scrolls lie about as [loot](magic-items.md#finding-them) on the floors of generated dungeon levels (see [Dungeons and danger](dungeons.md)), from the item
  level in the table above on. They are never [magic](magic-items.md), so vaults and [creatures](creatures.md) that leave only
  magic things never leave one.
- **Shops**: a general store sells and buys scrolls (see [Shops](shops.md)); one in the
  dungeons always has scrolls of identify among its goods, and so does Ælfric's in Cēaptūn.


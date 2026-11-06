# Scrolls

A scroll (`?`) holds one [spell](magic.md#the-spells) or charm. Press `r` to **read** one: a list of the scrolls you
carry opens; choose one with its letter (or the arrows and `Enter`). Reading takes a [turn](time.md#what-takes-time) and
uses the scroll up, if you can [make it out](#making-out-a-scroll).

{{#include generated/scrolls.md}}

## Making out a scroll

Reading a scroll asks one of three things, once nothing else stands in its way (no target, nothing to
identify, nothing to restore: those are refused, and take no time).

- **A spell you know**: if the scroll is of a spell you already know (one you wrote yourself too), you simply read it, and it still counts as a use of [Decipher script](skills.md#skill-decipher-script) if you are trained in it.
- **Enough [Spellcraft](skills.md#skill-spellcraft)**: if you are trained in it and your
  [learning](magic.md#learning) is at least the scroll's [spell level](magic.md#spell-levels), you simply read it. So
  what you could learn for certain, you can read for certain. A scroll with no spell behind it
  ([restoration](#the-scroll-of-restoration)) counts as level 0, so any Spellcraft will do.
- **Everyone else makes a check**: 2d12 plus your [Intelligence](stats.md#stat-intelligence) [modifier](stats.md#modifiers), what
  [talents](talents.md) add to its checks, and the points of [Decipher script](skills.md#skill-decipher-script), against 10 + the
  spell level ÷ 5, rounded down. Nobody is refused: without the skill it is your Intelligence and talents alone, and nothing that raises the skill helps.
  A natural 24 always succeeds and a natural 2 always fails.

Make it, and the scroll reads as below. Fail, and "The writing swims before your eyes.": the
[turn](time.md#what-takes-time) is spent, you keep the scroll, a [label](#unknown-scrolls) stays unknown, and nothing else
happens. You may try again at once. Every check earns those trained in Decipher script a
[training mark](skills.md#getting-better) in it (two in a [scriptorium](special-rooms.md)), made or failed; reading a scroll trains no Spellcraft.

{{#include generated/decipher-chances.md}}

### Deciphering a scroll unread

Those trained in [Decipher script](skills.md#skill-decipher-script) can use the **Decipher** [ability](abilities.md) on a scroll they carry of a kind they have not [identified](#unknown-scrolls), without reading it. It takes ten minutes (a creature coming into view, or a blow, stops it and nothing comes of it) and then makes the same check as reading it, with the same rules above: a scroll of a spell you know, or within your Spellcraft's learning, is made out at once. Make it, and you learn the kind: every scroll of it shows by its name from now on, and the scroll is kept, unread. Fail, and the ten minutes are gone and the scroll stays a label; you may try again. Every check earns a [training mark](skills.md#getting-better) in Decipher script, as a reading's does.

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
- **A [merchant](classes.md#merchant)'s [Merchant's eye](talents.md#talent-merchants-eye)** (from level 12), used on it from
  the abilities menu: it tells the scroll for what it is, as the scroll of identify would.
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
[Scroll adept](talents.md#talent-scroll-adept) takes it a rung up. It is **free of [mana](mana.md#spending-it)**: anyone can try one, a [warrior](classes.md#warrior) too, by [the check](#making-out-a-scroll). A spell on yourself ([Haste](magic.md#spell-haste), [Barkskin](magic.md#spell-barkskin),
[Blessing](magic.md#spell-blessing)) is read at once; any other is aimed first, as below; [Mend](magic.md#spell-mend) and [Cure wounds](magic.md#spell-cure-wounds) are aimed at
the creature beside you to heal, or at yourself. Scrolls of lower [spell levels](magic.md#spell-levels) are found sooner and
more often: cantrips from the first floors, spells of level 11 and 12 from [item level](magic-items.md#base-items) 6 (the table
above), and [general stores](shops.md) lay out low levels more often than high ones.

The scrolls of [fireball](magic.md#spell-fireball) (a rung up: 6d6), [lightning](magic.md#spell-lightning-bolt) (two rungs up: 8d6), and
[frost](magic.md#spell-frost) (its base: 4d6) cast their spells as any spell scroll does, reaching as far as you cast the
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
in half the time (with the [wizard](classes.md#wizard)'s Scroll adept as well, in a quarter: no less), and with
[Spell recall](talents.md#talent-spell-recall) after it one read in four leaves the scroll as it was ("The writing does not fade from the
scroll."). A scroll you study is always used up, Scroll savant or not.

## The scroll of identify

Choose the scroll of identify and a second list opens: the unidentified items you carry (see
[Magic items](magic-items.md)), the scrolls you carry that show only their label, and the wands
and rods you do not know (a wand's charges too; see [Magic](magic.md#wands)). Choose one, and you
know what it is from then on: "This is a Sharp dagger of the Lynx!", "This is 2 scrolls of
lightning!" (and every scroll of lightning shows by its name). With nothing unidentified, the
list says so, and the scroll is kept. It is [made out](#making-out-a-scroll) as any other scroll, a spell of level 6:
if the writing swims, you keep the scroll and learn nothing of what you chose.

Reading it casts the spell **[Identify](magic.md#spell-identify)** for you, free of [mana](mana.md#spending-it), and studying it may teach you the
spell for good (see [Magic](magic.md#learning-spells-from-scrolls)); cast for [mana](mana.md#how-much), Identify is
dear (60 mana), so only an experienced caster can afford it (see
[Magic](magic.md#what-a-spell-costs)).

The scroll of identify is the commonest scroll: it turns up more often than any other in [loot](magic-items.md#finding-them),
and every [general store](shops.md) in the [dungeons](dungeons.md) has a [stack](objects.md#piles) of them.

## The scroll of restoration

The scroll of restoration casts no spell. Read it (it is [made out](#making-out-a-scroll) as a spell of level 0), and
it works at once, on you: every point of
[stat damage](diseases.md#stat-damage) you carry mends, and every point [drains](stats.md#drained-stats) took comes back ("You feel your
strength return.", one line for each stat), as Sister Hild's mending in Cēaptūn does (see
[Services](npcs.md#services)), but wherever you are. It works with a [disease](diseases.md) still on you, though the
disease may damage the stat again. With nothing to restore it is not read ("There is nothing to
restore."), and you keep it. As no spell is behind it, there is nothing in it to
[study](magic.md#learning-spells-from-scrolls). It is rare: from [item level](magic-items.md#base-items) 8, and dear (the table above).

## Writing scrolls

A **blank scroll** is paper with nothing on it: never labelled, known on sight, 50 g, worth 15
gold. Read it and nothing happens ("There is nothing written on it."), and you keep it; there is
nothing in it to [study](magic.md#learning-spells-from-scrolls).

The [wizard](classes.md#wizard)'s **[Scribe](abilities.md#ability-scribe)** (from level 12) writes on one. Choose it in the
abilities menu, and a list opens of the spells you know, by school, each with what writing it
costs you in [mana](mana.md#spending-it) and how long it takes; choose one, and you write its scroll on the first
blank scroll you carry:

- **The price** is three times the spell's mana: what casting it would cost you before
  [Frugal caster](talents.md#talent-frugal-caster) (your armour, School focus, Empower, and Archmage counted), then cut
  by Frugal caster as a whole. Fireball's 26 is 78, Magic missile's 3 is 9, Identify's 60 is
  180.
- **The time** is a study's of the spell: 100 turns and 20 more for each spell level (10 minutes
  and 2 a level at a normal pace: Magic missile 16 minutes, Fireball 32).
  [Quick study](talents.md#talent-quick-study) does not shorten it.
- **Interrupted** (a creature coming into view, anything said, a blow), you stop writing: the time
  is lost, and the blank and the mana are kept. The mana is spent and the blank used up only
  when the scroll is done: "You write a scroll of fireball."
- **The scroll** is the ordinary scroll of that spell, at its own [rung](scrolls.md#spell-scrolls) (as one found would
  be), and you know what it is. It reads, studies, and sells as any other (a shop buys it at
  its usual offer); [Scroll adept](talents.md#talent-scroll-adept) adds its rung when you read it.
  Writing it asks no [check](#making-out-a-scroll), but reading it does as for any scroll: knowing a spell
  you learned beyond your [learning](magic.md#learning) does not make its scroll plain to you.
- **Refused**, with no time or mana spent: for a spell you do not know, with no blank scroll ("You
  have nothing to write on."), while raging, or with too little mana ("You lack the mana to
  write it."). Light and place do not matter.

Each scroll written trains [Intelligence](stats.md#training) (always, as a study does) besides what spending
the mana trains. There is no roll: a scroll begun and finished is a scroll made.

## Where scrolls are

- **Loot**: scrolls lie about as [loot](magic-items.md#finding-them) on the floors of generated dungeon levels (see [Dungeons and danger](dungeons.md)), from the item
  level in the table above on. They are never [magic](magic-items.md), so vaults and [creatures](creatures.md) that leave only
  magic things never leave one.
- **Shops**: a general store sells and buys scrolls (see [Shops](shops.md)); one in the
  dungeons always has scrolls of identify among its goods, and so does Ælfric's in Cēaptūn.
- **Blank scrolls** are loot and general-store stock as the scrolls are, from item level 1
  (frequency 30).


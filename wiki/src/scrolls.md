# Scrolls

A scroll (`?`) holds one [spell](magic.md#the-spells) or charm. Press `r` to **read** one. A list
of the scrolls you carry opens, and you choose one by its letter (or with the arrows and
`Enter`). Reading takes a [turn](time.md#what-takes-time) and uses the scroll up, as long as you
can [make it out](#making-out-a-scroll).

{{#include generated/scrolls.md}}

## Making out a scroll

Some reads are refused before anything else happens: a scroll with no target, nothing to
identify, or nothing to restore. Those take no time. Otherwise, whether you can make out the
writing depends on one of three things:

- **A spell you know.** If the scroll holds a spell you already know (including one you wrote
  yourself), you simply read it. It still counts as a use of [Decipher
  script](skills.md#skill-decipher-script) if you are trained in it.
- **Enough [Spellcraft](skills.md#skill-spellcraft).** If you are trained in it and your
  [learning](magic.md#learning) is at least the scroll's [spell level](magic.md#spell-levels),
  you simply read it. What you could learn for certain, you can read for certain. A scroll with
  no spell behind it ([restoration](#the-scroll-of-restoration)) counts as level 0, so any
  Spellcraft will do.
- **Everyone else makes a check.** Roll 2d12 and add your
  [Intelligence](stats.md#stat-intelligence) [modifier](stats.md#modifiers), whatever your
  [talents](talents.md) add to its checks, and your points of [Decipher
  script](skills.md#skill-decipher-script). You need 10 + the spell level ÷ 5, rounded down.
  Nobody is refused outright: without the skill it is your Intelligence and talents alone, and
  nothing that raises the skill helps. A natural 24 always succeeds and a natural 2 always fails.

If you make it, the scroll works as described below. If you fail, you are told "The writing
swims before your eyes.". The [turn](time.md#what-takes-time) is spent, you keep the scroll, a
[label](#unknown-scrolls) stays unknown, and nothing else happens. You may try again at once.

Every check, made or failed, earns those trained in Decipher script a [training
mark](skills.md#getting-better) in it, or two in a [scriptorium](special-rooms.md). Reading a
scroll trains no Spellcraft.

{{#include generated/decipher-chances.md}}

### Deciphering a scroll unread

If you are trained in [Decipher script](skills.md#skill-decipher-script), the **Decipher**
[ability](abilities.md) lets you work out a scroll you carry without reading it, as long as you
have not [identified](#unknown-scrolls) its kind yet. It takes ten minutes. A creature coming
into view, or a blow, stops it, and then nothing comes of it.

After the ten minutes you make the same check as for reading, with the same rules: a scroll of a
spell you know, or one within your Spellcraft's learning, is made out at once. If you make it,
you learn the kind: every scroll of that kind shows by its name from now on, and you keep this
one, unread. If you fail, the ten minutes are gone and the scroll stays a label. You may try
again. Every check earns a [training mark](skills.md#getting-better) in Decipher script, just as
reading does.

## Unknown scrolls

A scroll you have never read shows a **label** instead of its name: "a scroll labelled ZUNBAR",
in the same plain colour as every other unknown scroll. Each kind of scroll has its own label.
The labels are made up anew for each character and stay the same for their whole life, so two
scrolls labelled ZUNBAR are always the same kind. The label shows everywhere: in your
[pack](objects.md#your-pack), on the floor, in a [shop](shops.md), in [look
mode](controls.md#look-mode), and in what you are told.

You learn what a kind of scroll is, for good, in any of these ways:

- **Read one.** "You read the scroll labelled ZUNBAR." "It is a scroll of fireball." From then
  on every scroll of fireball shows by its name. Reading an unknown scroll is a gamble: a spell
  scroll is aimed first, and a [fireball](magic.md#spell-fireball) aimed near you may catch you
  too.
- **Read the [scroll of identify](#the-scroll-of-identify) on it** (below). You know the scroll
  of identify itself once you have read your first one.
- **A [merchant](classes.md#merchant)'s [Merchant's eye](talents.md#talent-merchants-eye)**
  (from level 12), used on it from the abilities menu, tells the scroll for what it is, as the
  scroll of identify would.
- **An appraisal.** Factor Wulfrun in Cēaptūn identifies everything you carry for 100
  [gold](shops.md#money) per [unidentified](magic-items.md#unidentified-items)
  [stack](objects.md#piles) (see [Services](npcs.md#services)).
- **Your [starting kit](starting-kit.md).** You know the scrolls you start with, including a
  caster's random ones.

Knowing what a scroll is does not teach you its spell. A scroll you know by name is still only a
scroll, but once you know it you may study it to learn its spell for good (see
[Magic](magic.md#learning-spells-from-scrolls)). A scroll that cannot be read, such as a sealed
letter, has no label and is known on sight. A [shop](shops.md#prices) charges its true price for
an unknown scroll.

## Spell scrolls

Every [spell](magic.md#the-spells) comes on a scroll of its own: a scroll of [magic
missile](magic.md#spell-magic-missile), of [haste](magic.md#spell-haste), of
[web](magic.md#spell-web). Reading one casts its spell once, as if you cast it yourself, with
its [shape](spellcasting.md#shapes), its resolution, and its [damage](combat.md#damage) as given
in the [spell's row](magic.md#the-spells).

It casts at the rung the scroll's row gives (the table's "Casts at"), which for most scrolls is
the base. Your stats do not grow a scroll's damage or healing, but its reach and how long it
lasts are yours, as for your own casts. [Scroll adept](talents.md#talent-scroll-adept) takes it
a rung up.

A scroll is **free of [mana](mana.md#spending-it)**. Anyone can try one, even a
[warrior](classes.md#warrior), by making [the check](#making-out-a-scroll). A spell on yourself
([Haste](magic.md#spell-haste), [Barkskin](magic.md#spell-barkskin),
[Blessing](magic.md#spell-blessing)) is read at once. Any other spell is aimed first, as below.
[Mend](magic.md#spell-mend) and [Cure wounds](magic.md#spell-cure-wounds) are aimed at the
creature beside you that you want to heal, or at yourself.

Scrolls of lower [spell levels](magic.md#spell-levels) are found sooner and more often: cantrips
from the first floors, and spells of level 11 and 12 from [item level](affixes.md#base-items)
6 (see the table above). [General stores](shops.md) also lay out low levels more often than high
ones.

### Fireball, frost, and lightning

The scrolls of [fireball](magic.md#spell-fireball) (a rung up: 6d6),
[lightning](magic.md#spell-lightning-bolt) (two rungs up: 8d6), and [frost](magic.md#spell-frost)
(its base: 4d6) cast their spells as any spell scroll does. They reach as far as the spell would
when you cast it.

An aimed scroll is aimed like a [shot](ranged.md#shooting-and-throwing) (see
[Controls](controls.md#aiming)). After you choose the scroll, a cursor appears on the map. It
starts on your last target if it is still there, otherwise on the nearest
[hostile](creatures.md#factions) [creature](creatures.md) you see, otherwise on you. `Tab` moves
it from creature to creature, and the movement keys move it anywhere. `Enter` (or `r` again)
reads the scroll at the cursor, and `Esc` puts it away unread. A cell out of reach is marked "Out
of range.", and the scroll is not used.

- **Fireball** and **frost** fly along the dotted line, like a
  [missile](ranged.md#where-a-missile-goes). They burst on the cell you aimed at, on the first
  creature in the way, or just short of a wall. The burst catches everyone within its radius,
  including you if you stand that close. When you aim so near that it might, a window asks
  first: `The fireball may catch you too. Read the scroll of fireball anyway?`, with **Read it**
  (`y`), **Aim again** (`n`), and **Don't read it** (`Esc`, chosen at first). With the
  [talent](talents.md) [Scroll savant](talents.md#talent-scroll-savant) your own bursts never
  catch you, and nothing asks.

  A fireball blows apart the softer [walls](terrain.md#walls) it reaches and [melts
  ice](temperature.md#walking-on-ice), and whoever stands on the ice [falls
  in](temperature.md#falling-in). Frost freezes the [water](terrain.md#water) it reaches into ice
  that holds a laden person (see [Temperature and ice](temperature.md)).
- **Lightning** runs from you along the line through the cursor and on beyond it, striking
  everyone on it, until it hits a wall or the end of its reach. It never strikes you.

These three never roll [to hit](combat.md#hitting): they hurt everyone they reach. A spell
scroll of a ray, such as [ray of frost](magic.md#spell-ray-of-frost), does roll to hit as the
spell does, with your Dexterity.

Unlike a trap's spell, a spell you read is **yours**. What it kills earns you
[experience](experience.md) and counts for your [quests](quests.md), and a
[shopkeeper](shops.md#theft) it hurts turns on you. A scroll that kills you is named on the
[death](hit-points.md#death) screen ("Killed by their own scroll of fireball"). The explosion and
the thunder are heard like any noise (see [Hearing](hearing.md)). Spells cannot be read on the
[overworld](overworld.md#travel).

### Reading faster, and keeping the scroll

Reading a scroll uses it up, but scholars do better (see [Talents](talents.md)). [Scroll
savant](talents.md#talent-scroll-savant) reads in half the time, and together with the
[wizard](classes.md#wizard)'s Scroll adept in a quarter, but no less. With [Spell
recall](talents.md#talent-spell-recall) on top, one read in four leaves the scroll as it was:
"The writing does not fade from the scroll." A scroll you study is always used up, Scroll savant
or not.

## The scroll of identify

When you read the scroll of identify, a second list opens. It holds the unidentified items you
carry (see [Magic items](magic-items.md)), the scrolls you carry that show only their label, and
the wands and rods you do not know, a wand's charges included (see
[Wands](wands-and-rods.md#wands)). Choose one, and from then on you know what it is: "This is a
Sharp dagger of the Lynx!", "This is 2 scrolls of lightning!" (and every scroll of lightning
shows by its name). If nothing is unidentified, the list says so and you keep the scroll.

It is [made out](#making-out-a-scroll) like any other scroll, as a spell of level 6. If the
writing swims, you keep the scroll and learn nothing about what you chose.

Reading it casts the spell **[Identify](magic.md#spell-identify)** for you, free of
[mana](mana.md#spending-it), and studying it may teach you the spell for good (see
[Magic](magic.md#learning-spells-from-scrolls)). Cast for [mana](mana.md#how-much), Identify is
dear at 60 mana, so only an experienced caster can afford it (see [Magic](magic.md#what-a-spell-costs)).

The scroll of identify is the most common scroll. It turns up more often than any other in
[loot](magic-items.md#finding-them), and every [general store](shops.md) in the
[dungeons](dungeons.md) has a [stack](objects.md#piles) of them.

## The scroll of restoration

The scroll of restoration casts no spell. Read it (it is [made out](#making-out-a-scroll) as a
spell of level 0) and it works on you at once. Every point of [stat
damage](diseases.md#stat-damage) you carry mends, and every point that [drains](stats.md#drained-stats)
took comes back ("You feel your strength return.", one line for each stat). It does what Sister
Hild's mending in Cēaptūn does (see [Services](npcs.md#services)), but wherever you are.

It works even with a [disease](diseases.md) still on you, though the disease may damage the stat
again. With nothing to restore, it is not read ("There is nothing to restore."), and you keep it.
As no spell is behind it, there is nothing in it to [study](magic.md#learning-spells-from-scrolls).
It is rare, found from [item level](affixes.md#base-items) 8, and dear (see the table above).

## Writing scrolls

A **blank scroll** is paper with nothing on it. It is never labelled, it is known on sight, and
it weighs 50 g and is worth 15 gold. Read it and nothing happens ("There is nothing written on
it."), and you keep it. There is nothing in it to [study](magic.md#learning-spells-from-scrolls).

The [wizard](classes.md#wizard)'s **[Scribe](abilities.md#ability-scribe)** (from level 12)
writes on one. Choose it in the abilities menu and a list of the spells you know opens, sorted by
school, each with what writing it costs you in [mana](mana.md#spending-it) and how long it takes.
Choose one, and you write its scroll on the first blank scroll you carry.

- **The price** is three times the spell's mana. That is what casting it would cost you before
  [Frugal caster](talents.md#talent-frugal-caster), with your armour, School focus, Empower, and
  Archmage counted, and then the whole is cut by Frugal caster. Fireball's 26 becomes 78, Magic
  missile's 3 becomes 9, and Identify's 60 becomes 180.
- **The time** is the time a study of the spell takes: 100 turns plus 20 for each spell level,
  or 10 minutes plus 2 a level at a normal pace. Magic missile takes 16 minutes and Fireball 32.
  [Quick study](talents.md#talent-quick-study) does not shorten it.
- **If you are interrupted** (a creature coming into view, anything said, a blow), you stop
  writing. The time is lost, but you keep the blank and the mana. The mana is spent and the
  blank used up only when the scroll is done: "You write a scroll of fireball."
- **The scroll** is the ordinary scroll of that spell, at its own [rung](#spell-scrolls), as one
  you found would be, and you know what it is. It reads, studies, and sells like any other (a
  shop buys it at its usual offer). [Scroll adept](talents.md#talent-scroll-adept) adds its rung
  when you read it. Writing it asks no [check](#making-out-a-scroll), but reading it does, as for
  any scroll: knowing a spell you learned beyond your [learning](magic.md#learning) does not make
  its scroll plain to you.
- **Scribe is refused**, with no time or mana spent, for a spell you do not know, with no blank
  scroll ("You have nothing to write on."), while raging, or with too little mana ("You lack the
  mana to write it."). Light and place do not matter.

Each scroll written always trains [Intelligence](stats.md#training), as a study does, besides
what spending the mana trains. There is no roll: a scroll begun and finished is a scroll made.

## Where scrolls are

- **Loot**: scrolls lie about as [loot](magic-items.md#finding-them) on the floors of generated
  dungeon levels (see [Dungeons and danger](dungeons.md)), from the item level given in the
  table above. They are never [magic](magic-items.md), so vaults and [creatures](creatures.md)
  that leave only magic things never leave one.
- **Shops**: a general store sells and buys scrolls (see [Shops](shops.md)). One in the dungeons
  always has scrolls of identify among its goods, and so does Ælfric's in Cēaptūn.
- **Blank scrolls** turn up as loot and as general-store stock just as other scrolls do, from
  item level 1 (frequency 30).

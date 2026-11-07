# Talents

A **talent** is a knack your character picks up as they grow: a little more of something the rules
already measure, chosen by you. Talents come in **trees** (Vigour, Might, Finesse, and so on), and
most ask something of you first.

## When you pick

A new character picks **two talents at creation**, on the Talents step after the
[class](classes.md). The **[bard](classes.md#bard)**, the lucky class, picks one more, and so does a
**[human](races.md#race-human)** (their racial [Versatile](talents.md#talent-versatile), below). A
human bard picks four.

The list shows the talents your character so far may take. Its scores, [skills](skills.md),
[class abilities](talents.md#class-abilities), and [birth sign](horoscope.md#what-a-sign-does) all
count. Marking one may open another that needs it ([Healthy](talents.md#talent-healthy), then
[Fast healer](talents.md#talent-fast-healer)). The **Random** row lets chance pick them. A
character started without picks (from the command line) gets those same random picks. The
[character](character.md) screen counts your talents.

{{#include generated/talents-creation.md}}

Every third **[level](experience.md#levels)** brings a talent: levels 3, 6, 9, and so on, twenty by
level 60 (see [Experience](experience.md)). You make the pick **the moment the level comes**.
Nothing is banked for later, and until you have chosen you can do nothing else ("You must choose a
talent first."). A [human](races.md#race-human)'s [Versatile](talents.md#talent-versatile) brings a
second talent at level 30. A kill that brings two [levels](experience.md#levels) at once brings both
picks, lowest level first. You cannot change your mind afterwards: a talent, once learned, is yours
for good.

Some [classes](classes.md#levels) also give a **bonus talent** at a few levels (the everyday folk at
10, 20, 30, 45, and 55), from the trees the class names: "You must choose a talent of Weaponcraft
or Delver." When a level brings both kinds, a talent from the class's trees fills the bonus pick
first. The [commoner](classes.md#commoner)'s bonus talents come from any tree.

If there is nothing you can take (a run with no talents, or every talent you qualify for already
learned), the pick is simply lost. A level never waits for the impossible.

The **[level-up screen](experience.md#what-a-level-gives)** opens by itself when a level comes. It
walks through the level's choices in order:

1. the [skill rolls](skills.md), each [skill](skills.md) shown with the dice a roll on it adds;
2. a bonus talent;
3. the level's talent, listing only the talents you may take now, by tree.

A talent with a choice ([Weapon focus](talents.md#talent-weapon-focus),
[Skill focus](talents.md#talent-skill-focus)) asks for it next. The screen can't be closed until
everything is chosen. Two levels at once are one sequence, the lower level's choices first.

The **Talents page** (`G`, or page 8 of the [information pages](the-screen.md#info-pages)) lists
your talents, then every other talent you could still pick, by tree, with what it needs. The parts
you already meet stand out from those you don't, so you can plan a build there.

**Class abilities** are talents too, in a tree of their own. Your [class](classes.md) gives them as
its levels come (see [Classes](classes.md#levels)); you never pick them. If your class gives one you
already learned, it is not given again, and that level gives you a talent pick in its place. Some are
talents anyone may pick (the [aristocrat](classes.md#aristocrat)'s
[Riposte](talents.md#talent-riposte), the [commoner](classes.md#commoner)'s
[Second wind](talents.md#talent-second-wind)).

## Choices and prerequisites

- A talent is picked once.
- A talent with a **choice** is picked for that choice: a [weapon skill](weapon-skills.md), a
  [fighting style](fighting-styles.md), a [skill](skills.md), or an element. The same talent may be
  picked again for another choice. [Weapon focus](talents.md#talent-weapon-focus) with
  [long blades](weapon-skills.md#weapon-skill-long-blades) and Weapon focus with
  [bows](weapon-skills.md#weapon-skill-bows) are two talents. A weapon skill you choose must be one
  you attack with, a fighting style is chosen as a style, and a skill
  ([Skill focus](talents.md#talent-skill-focus)) must be one you are trained in.
- **Prerequisites** must all hold when you pick, and are never checked again. A
  [Strength](stats.md#stat-strength) lowered by [disease](diseases.md#how-a-disease-goes) later does
  not take a talent away.

A prerequisite may be:

- another talent;
- a [stat](stats.md) score: the score itself, as the [character screen](character.md) shows it,
  never what items or a sickness do to it;
- a [skill](skills.md) at a value;
- a [weapon skill](weapon-skills.md) or style at a rank, or any of a kind: "a missile weapon skill
  3" is [bows](weapon-skills.md#weapon-skill-bows), [crossbows](weapon-skills.md#weapon-skill-crossbows),
  or [slings](weapon-skills.md#weapon-skill-slings);
- a character level;
- your [race](races.md), your [class](classes.md), or your eyes (your race's [vision](vision.md));
- your [birth sign](horoscope.md#what-a-sign-does)'s animal or element, or a lucky day (see
  [Horoscope](horoscope.md));
- your [alignment](alignment.md) band.

A few ask for either of two ("Intelligence 12 or Appraise 20"). The game names what you lack: "You
cannot take Dodge: you need [Dexterity](stats.md#stat-dexterity) 13."

## Gifts of the stars

The first tree, **Stars**, holds what your [birth sign](horoscope.md#what-a-sign-does) gives you:
the gifts of your animal and your element, your animal's flaw, and a lucky day's gift (see
[Horoscope](horoscope.md)). They are yours from birth without a pick, and nobody can pick them. No
talent ever asks for a flaw.

## Heritage: only at character creation

The **Heritage** tree holds what your family gave you. Its talents are offered **only at
[character creation](character.md)** (the Talents step, or a random pick there), never at a level.
The Talents page shows them with "to be a new character", which no later level meets. What they
give comes at once, on top of your [starting kit](starting-kit.md).

- **[Family wealth](talents.md#talent-family-wealth)** gives you three times the
  [gold coins](shops.md#money) you start with more: your [class](classes.md)'s purse, and any a
  start of the world adds. That is at least 60 and at most 150 more. It also gives two or three
  loose [gems](gems.md), as found at [item level](magic-items.md#finding-them) 2. Coins weigh 10 g
  each, so the weakest of the small peoples ([halflings](races.md#race-halfling),
  [gnomes](races.md#race-gnome), [goblins](races.md#race-goblin), [kobolds](races.md#race-kobold),
  [ratfolk](races.md#race-ratfolk)) may set out [Burdened](burden.md#burden-1).
- **[Father's sword](talents.md#talent-fathers-sword)** (needs Family wealth) is your people's
  heirloom weapon: not always a sword, but always a weapon. It is [magic](magic-items.md), with 2 to
  4 [affixes](affixes.md#prefixes) rolled at item level 6. Two make it magic; more make it
  [rare](magic-items.md#rarity) and give it a name. You
  [know what it does](magic-items.md#unidentified-items) from the start. It is readied in its own
  [slot](equipment.md#what-fits-where) if your kit left that free: the main hand, or the
  [missile weapon](ranged.md#getting-ready) slot for a sling or a bow. Otherwise it waits in your
  pack beside the kit's own weapon.

{{#include generated/talents-heirlooms.md}}

## Racial talents

The **Racial** tree holds what each people is good at. Every talent in it asks for its
[race](races.md), and some for a score, a level, or a [weapon skill](weapon-skills.md) besides. So
only an [elf](races.md#race-elf) may take [Elven accuracy](talents.md#talent-elven-accuracy), and
only a [dwarf](races.md#race-dwarf) [Stone sense](talents.md#talent-stone-sense). They are picked
like any other talent.

[Humans](races.md#race-human) have no gift of their own, so they are given
**[Versatile](talents.md#talent-versatile)** from the start instead: one more talent at creation and
one more at level 30. Like the gifts of the stars it is never on a pick list, and the
[character](character.md) screen's Talents page marks it as your race's.

## What a talent does

In the game a talent is described in words; the tables below state the rule exactly. Most talents
simply change a rule for good. Where a talent scales ("one hit point per four levels"), what it gave
earlier grows as you level: a talent taken at level 3 still matters at level 50.

An **active** talent ([Second wind](talents.md#talent-second-wind)) is something you do. It appears
in the [abilities menu](abilities.md) under Talents. After use it leaves a status
([Winded](status-effects.md#status-winded)) during which it cannot be used again.
[Blood magic](talents.md#talent-blood-magic) is active too, with no waiting time: it costs
[hit points](hit-points.md#hit-points) instead (see [Mana](mana.md#spending-it)).

Some talents need a word more:

- Five talents work on a [class](classes.md)'s [resources](resources.md#talents-on-resources), and
  only that class can take them: [Battle trance](talents.md#talent-battle-trance),
  [Inner calm](talents.md#talent-inner-calm), [Steady hand](talents.md#talent-steady-hand),
  [Patient blade](talents.md#talent-patient-blade), and [Zealot](talents.md#talent-zealot). One that
  needs the [lantern's hood](lantern.md#the-hood) (Hooded light) is still to come.
- [Skill focus](talents.md#talent-skill-focus) adds 25 to the [skill](skills.md) you choose for
  [what it gives](skills.md#what-skills-give), past 100 too. The value you trained stays as it was.
- [Treasure hunter](talents.md#talent-treasure-hunter) changes loot as it is made. A level you have
  already been to keeps what lies there, and the levels themselves never change.
- [Breaker](talents.md#talent-breaker)'s kick drives a foe one cell straight back when it hits, if
  that cell is free. A wall, a creature, a [trap](traps.md), or [ice](temperature.md) behind the foe
  holds it where it is.

{{#include generated/talents.md}}

## Coverage

Every part of the game has talents that make you better at it. Each talent names the parts it
touches, from a fixed list, and every part on that list has at least one talent. Once
[classes](classes.md) have their level tables, every part also has at least one
[class ability](talents.md#class-abilities). Parts marked pending are still waiting for theirs.

The parts are broad: fighting, stealth, magic, light, hunger, and so on. A single kind of room,
chest, or creature belongs to its part, not a part of its own. A talent is only made when it changes
how a character plays, never for a number shown or a small gain in a rare place.

This holds for the future too: **every new part of the game arrives with a talent**. That includes
new [resources](resources.md). [Mana](mana.md#how-much) came with talents for a deeper pool, a
quicker refill, and cheaper casting. [Spells](magic.md#the-spells) came with the Spellcraft tree
([Quick study](talents.md#talent-quick-study), [Overreach](talents.md#talent-overreach),
[School focus](talents.md#talent-school-focus)), and with talents for
[wands](wands-and-rods.md#wands) and [rods](wands-and-rods.md#rods)
([Wand mastery](talents.md#talent-wand-mastery), [Channeler](talents.md#talent-channeler)). Each
[class](classes.md)'s own pool gets the same, such as a [berserker](classes.md#berserker)'s
[rage](resources.md#rage) and a [ranger](classes.md#ranger)'s [focus](resources.md#resource-focus).

[Abilities](abilities.md) that cost such a resource have no waiting time on top: the resource coming
back with time is the limit.

{{#include generated/talents-by-system.md}}

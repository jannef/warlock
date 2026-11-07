# Races

Your race is the first choice of a new run (see [Your character](character.md)). It does several
things:

- It changes your starting [stats](stats.md).
- It gives you your eyes: the [vision type](vision.md#the-four-types) you start with.
- It sets how old you can be when you start.
- It adds to your [hit points](hit-points.md#hit-points).
- It trains you in a [skill](skills.md) or two. [Humans](races.md#race-human) instead get a little
  more in one of their class's skills (see [Skills](skills.md#who-trains-what)).
- It may give you a rank in a [weapon skill](weapon-skills.md) or two, on top of your class's.
- It adds a few things to your [starting kit](starting-kit.md): an [elf](races.md#race-elf)'s
  waybread, a [dwarf](races.md#race-dwarf)'s pick-axe, a [halfling](races.md#race-halfling)'s pipe. It also gives the everyday clothes and shoes of
  your folk where your class gives none ([halflings](races.md#race-halfling) go barefoot).

{{#include generated/races.md}}

The changes are added to the average score of 10, together with your gender's, your
[class](classes.md)'s, and your [birth sign's](horoscope.md#what-a-sign-does). Most are plain
numbers. A few are dice, rolled when you start: `1d2+1` is one or two, plus one (see
[How your stats add up](character.md#how-your-stats-add-up)). Your starting age is picked at random
within the race's range.

Races are not balanced against each other. [Humans](races.md#race-human) start with no gifts and no
flaws. The others trade one strength for one weakness, or for two.

[Halflings](races.md#race-halfling), [gnomes](races.md#race-gnome), [goblins](races.md#race-goblin),
[kobolds](races.md#race-kobold), and [ratfolk](races.md#race-ratfolk) are small, and carry three
quarters of what the others do. [Dwarves](races.md#race-dwarf) and [duergar](races.md#race-duergar)
are not slowed or hindered by a load until they strain under it. See [Burden](burden.md).

Races need different amounts of [food](food.md). [Elves](races.md#race-elf),
[drow](races.md#race-drow), [goblins](races.md#race-goblin), and [kobolds](races.md#race-kobold)
need the least, [halflings](races.md#race-halfling) (second breakfast) the most. See
[Hunger](hunger.md).

## The evil races

Six races start on the evil side of [alignment](alignment.md).
[Goblins](races.md#race-goblin), [kobolds](races.md#race-kobold),
[lizardfolk](races.md#race-lizardfolk), and [ratfolk](races.md#race-ratfolk) start Callous;
[drow](races.md#race-drow) and [duergar](races.md#race-duergar) start Evil. Some are poor choices
on purpose, and their [levels](experience.md#levels) come quicker for it (see
[Experience](experience.md)).

- **[Goblins](races.md#race-goblin)** are the quick, crappy choice. They are weak and frail but
  quick-handed and sneaky ([Stealth](skills.md#skill-stealth)), short-lived, and light eaters. Only
  the kobolds' [levels](experience.md#levels) come faster.
- **[Drow](races.md#race-drow)** are the [elves](races.md#race-elf) of the deep dark: graceful and
  sharp, but frail. They are a little [resistant](damage-types.md#armour-and-resistance) to
  [poison](damage-types.md#damage-poison), and like elves they
  [notice every hidden thing in passing](searching.md#noticing-in-passing). Strong light dazzles
  them (below). They are long-lived, so their [levels](experience.md#levels) come slowly.
- **[Kobolds](races.md#race-kobold)** are the weakest of all, but good with
  [traps](traps.md#finding-traps). They [notice hidden traps in passing](searching.md#noticing-in-passing)
  (never a [secret door](doors.md#secret-doors)), find hidden things more easily, and start trained
  in [Disable device](skills.md#skill-disable-device). With their huge families, their
  [levels](experience.md#levels) come cheapest of all.
- **[Lizardfolk](races.md#race-lizardfolk)** are strong and tough, dull and alien. Their scales soak
  a point of every physical blow, as natural armour does for a [creature](creatures.md)'s hide (see
  [Combat](combat.md#protection)). They are cold-blooded, and will be slower in the
  [cold](temperature.md) once the cold slows anyone.
- **[Duergar](races.md#race-duergar)**, the [dwarves](races.md#race-dwarf)' grey cousins, are tough
  and dour. Like dwarves, they shrug off a [load](burden.md) until they strain under it, and they
  [notice what is hidden](searching.md#noticing-in-passing) in stone.
- **[Ratfolk](races.md#race-ratfolk)** are nimble scavengers, weak and shunned. They are hard to
  sicken, with a bonus on every [check](diseases.md#checks) against a [disease](diseases.md), and
  they eat rotten food without harm (see [Food](food.md)).

No [creature](creatures.md) takes your side because of your race.
[Hostile](creatures.md#factions) [kobolds](races.md#race-kobold) attack a kobold too. Good folk
greet an Evil character coldly, but that comes from your [alignment](alignment.md), not your race.

**Strong [light](light.md#what-light-is)** dazzles [drow](races.md#race-drow). Light is strong
enough when the [sun](sun.md#sky-light) is well up under a clear sky, or a bright fire is close
by; your own [lantern](lantern.md) alone is not enough. While the light on your own cell is that
bright, you see nothing farther than ten cells and fight a little worse. The game tells you when it
begins and ends, and the [status bar](the-screen.md#the-screen) shows `Dazzled` while it lasts.

Each race's gifts and flaws beyond its stats, exactly:

{{#include generated/race-traits.md}}

## Racial talents

Each race except the humans has [talents](talents.md#racial-talents) that only it may pick, in the
Racial tree. Humans get Versatile instead: one more talent at creation and at level 30.

{{#include generated/talents-racial.md}}

# Stats

You have six **stats**, each a number that says how strong, quick, hardy, clever, steadfast, or
striking you are. They are inspired by the abilities of Dungeons & Dragons, and three of them go
by other names here: [Toughness](stats.md#stat-toughness) is D&D's Constitution, [Willpower](stats.md#stat-willpower) its Wisdom, and [Presence](stats.md#stat-presence) its
Charisma.

{{#include generated/stats.md}}

Your stats are set when you create your character at the start of a run: your
[race](races.md), your gender, and your [class](classes.md) each add to or take from the average
of 10 (see [Your character](character.md)). After that they grow by use: see
[Training](#training) below.

## Seeing your stats

Press `@` to open the **character screen**. It shows your name, your [race](races.md), gender, and [class](classes.md),
your age, birthday, and [birth sign](horoscope.md), your stats in the order above (with their
short names; a stat that a [status effect](status-effects.md#how-they-work) or a [disease](diseases.md#how-a-disease-goes) lowers is shown lowered, in red), your [hit
points](combat.md#hit-points), [mana](mana.md#how-much) (see [Mana](mana.md)), and [Defence](combat.md#defence) (see [Combat](combat.md)), your weight: your own, plus what you carry (see [Objects and your pack](objects.md)), and how much you can carry (see
[Burden](burden.md)). Looking takes no time; press `@` or `Esc` to close it.

[Creatures](creatures.md) have stats too, but you cannot see theirs in the game: they are on
[Creatures in numbers](creature-numbers.md).

## What they do

In a fight (see [Combat](combat.md)), under a load (see [Burden](burden.md)), at a door (see
[Doors](doors.md)), and in your [mana](mana.md#how-much) (see [Mana](mana.md)):

- **[Strength](stats.md#stat-strength)** helps you [hit](combat.md#hitting) up close and adds to the [damage](combat.md#damage) of your blows, and draws a bow. It
  counts double toward how much you can [carry](burden.md#how-much-you-can-carry), and the stronger you are, the sooner you kick a
  [door](doors.md#kicking) in and [dig](digging.md#digging) through a wall.
- **[Dexterity](stats.md#stat-dexterity)** helps you aim a shot or a throw and dodge blows: it adds to your [Defence](combat.md#defence) (less
  under a heavy load or heavy armour).
- **[Toughness](stats.md#stat-toughness)** gives you [hit points](combat.md#hit-points), and makes them come back sooner. It counts toward how much
  you can [carry](burden.md#how-much-you-can-carry), and fights off most [diseases](diseases.md).
- **[Intelligence](stats.md#stat-intelligence)** helps you find hidden things when you search ([Searching](searching.md)), and
  adds to your [mana](mana.md).
- **[Willpower](stats.md#stat-willpower)** helps you [notice hidden things in passing](searching.md#noticing-in-passing), adds to your [mana](mana.md) and makes
  it come back sooner, and fights off a few [diseases](diseases.md#how-a-disease-goes) (brain fever).
- **[Presence](stats.md#stat-presence)** moves [shop](shops.md) prices in your favour, or against you if it is low, and adds
  to your [mana](mana.md#how-much).

### Modifiers

Most rules read a stat's **modifier** rather than its score, as in D&D: the score less 10,
halved and rounded down. The game never shows a modifier; here they are:

{{#include generated/stat-modifiers.md}}

What reads the modifier and what reads the score, exactly:

{{#include generated/stat-uses.md}}

### Potentials

Every stat also has a **potential**, which the game never shows: how far it can grow for good.
[Magic items](magic-items.md) raise a stat only while you wear them, potential or not;
training grows dearer past it but goes on, and never moves the potential itself (see
[Training](#training)). Only elixirs and, rarely, a fresh [corpse](food.md#corpses) widen it. Yours are set at the start of a run: the potential your [race](races.md)
gives, rolled once, and never below your starting score (a score above it raises the potential
to the score):

{{#include generated/potentials.md}}

Training never raises a potential; an **elixir** does: it raises its stat by 1 and the
potential by 1 or 2 more (see [Potions](potions.md#elixirs)).

### What changes them

Stats do not change how far you see or how quietly you walk. Strong [poison](status-effects.md#poisoned) lowers your [Strength](stats.md#stat-strength)
and [Dexterity](stats.md#stat-dexterity) by one while it lasts, for everything they do (see
[Status effects](status-effects.md)), and [hunger](hunger.md#how-hungry-you-are) saps them for a while ([Weak](hunger.md#how-hungry-you-are) lowers your
Strength and Dexterity; see [Hunger](hunger.md)). [Diseases](diseases.md#how-a-disease-goes) take points off your stats (**stat
damage**), which come back a point a day once no disease shows (see [Diseases](diseases.md)).
Eating the [corpse](food.md#corpses) of some [creatures](creatures.md) may raise one for good, as far as it can grow (see
[Food and corpses](food.md)), and so may [training](#training) at a new level. The touch of some
undead lowers one for good ([Drained stats](#drained-stats)), and so may a check after
[deeds that cost marks](#losing-marks). Nothing else raises or lowers them for good.

## Training

You never pick a stat to raise. Instead your stats grow with what you do: fighting up close,
landing [criticals](combat.md#criticals), and blocking with a shield train [Strength](stats.md#stat-strength), dodging, parrying, and shooting
[Dexterity](stats.md#stat-dexterity), being hurt and felling worthy foes [Toughness](stats.md#stat-toughness), reading and [searching](searching.md#searching-1) [Intelligence](stats.md#stat-intelligence),
enduring, noticing, and casting [Willpower](stats.md#stat-willpower), talking and trading [Presence](stats.md#stat-presence). Each such use may
earn the stat a **training mark**. A stat's marks are one running sum, its pile: marks never
fade, have no limit, and count the same however many come in a day.

What earns a mark, and how likely:

{{#include generated/stat-training.md}}

Asking someone about a topic takes no time, so it trains nothing; a long action counts only
when it lasts more than one turn and is not interrupted.

Every stat has paths outside its obvious one. A fighter who never opens a scroll still sharpens
Intelligence by exploring, searching, foraging, and disarming traps; a wizard grows stronger by
wading, hauling, and struggling free of a grip. Casting a spell you know trains the stats the
spell grows with ([its damage, reach, and duration](magic.md#the-lifted-cap)): Intelligence for most, Willpower for
those that reach far, Presence for hexes and blessings that last. Some uses train two stats at
once (disarming a trap, struggling against a hold).

### The price of a point

Each stat's next point has a **price** in marks: **20 + 10 × the stat's score** up to its
[potential](stats.md#potentials). Past the potential the price climbs quickly: the first point
beyond it costs half again, the second 2½ ×, the third 4 ×, then 6 ×, 8½ ×, 11½ × (with `n` the
point past the potential, (4 + n × (n + 1)) / 4 times the plain price). Training alone never
raises the potential itself, so a stat you live by ends a long career some ten points past its
nature and never reaches the top by use alone; an [elixir](potions.md#elixirs) or a rare
[corpse](food.md#corpses) that raises the potential makes every later point cheaper.

{{#include generated/stat-prices.md}}

(A drop, the last column, is the price a stat's pile pays back if it ever falls by a point:
half the plain price of the point lost, never more past the potential.)

### The checks

Every time you gain an [experience level](experience.md), and once a day at **06:00** (the hour
a [rest](resting.md#how-resting-works) until dawn ends), each stat in turn gets one **check**,
the same rule at both. A pile under a quarter of the price is out of play: nothing happens. From
a quarter up the stat rises by a point with chance **marks³ / (marks³ + price³)**: even odds at
the price, about one in nine at half of it, near certain at twice it.

{{#include generated/stat-odds.md}}

A rise **pays the price** out of the pile; whatever is left over waits for the next check, so
nothing is ever wasted. A lucky rise from a short pile spends what there was and no more. A
stat rises at most one point a check, so a great pile becomes a point a day and one a level. The
game says only "You feel stronger." (quicker, tougher, sharper, more resolute, more confident);
the new score shows on the character screen. A rise in [Toughness](stats.md#stat-toughness) brings its
[hit point](combat.md#hit-points) at once, and a rise at 06:00 stops a [rest](resting.md#how-resting-works). The check comes whether
you are awake or resting, once a day, at that hour and no other; nothing decays and nothing is
capped, so the only limit is the clock.

How many days the next point takes, by the marks a stat earns a day (the daily check alone; a
level's check comes on top):

{{#include generated/stat-pace.md}}

The **[Hard training](talents.md#talent-hard-training)** [talent](talents.md) gives every use half
again its chance of a mark (a sure one stays sure), so every stat trains half again as fast.

### Losing marks

A few deeds take marks away instead, one for each stat, so that you can lower a stat on purpose
if you want to: starve, faint, carry far too much, walk onto traps you know are there, or get
caught stealing. Bad luck alone never costs a mark: a hidden trap, a blow, a disease, or a drain
takes none. Each deed takes its marks at once, or a mark with the chance shown:

{{#include generated/stat-losses.md}}

The pile then goes below zero, and the checks read it the same way against the **drop's price**:
half the plain price of the point lost, never more past the potential. Nothing happens while
the pile is less than a quarter of that below zero; the first time a deed takes it that far, the
game warns you in words ("You feel your strength wasting.", "Your hands feel less sure.", "You
feel your health failing.", "Your mind feels dull.", "Your will feels brittle.", "You feel small
and unregarded."), so a warning always comes before a drop. From there a check lowers the stat
by a point with the same odds as a rise, "You feel weaker." (clumsier, frailer, duller; "Your
resolve wavers.", "You feel less sure of yourself."), and the drop pays its price back toward
zero, never past it. A drop never takes a stat below 3, nor below the
[stat damage](diseases.md#how-a-disease-goes) on it plus one, and never lowers the potential: a
point lost below the potential is won back at its plain price. Your most
[hit points](combat.md#hit-points) and [mana](mana.md) fall with it, but a drop never kills.

At 14, losing a point takes 75 marks of deeds (five dawns greeted Weak) and winning it back 150
marks of use, so there is nothing to gain by going up and down.

## Drained stats

Some of the undead drink the life out of the living. When such a [creature](creatures.md)'s touch hits you and
you live, you make a **check** of a stat ([Willpower](stats.md#stat-willpower), unless the creature says otherwise): 2d12
plus the stat's [modifier](stats.md#modifiers) against the drain's difficulty, as a [disease](diseases.md#how-a-disease-goes)'s check. If you hold, "A
deathly chill passes through you." and nothing is lost. If not, points come off one of your stats
**for good** ("You feel your strength drain away!"): your score is simply lower from then on,
for everything it does. A drain of [Toughness](stats.md#stat-toughness) lowers your most [hit points](combat.md#hit-points), one of [Intelligence](stats.md#stat-intelligence),
[Willpower](stats.md#stat-willpower), or [Presence](stats.md#stat-presence) your most [mana](mana.md#how-much); it never kills you, and no drain takes a score below 3
(nor below what the [stat damage](diseases.md#stat-damage) on it needs). [Resting](resting.md#how-resting-works) never brings drained points back.

Your **[potential](stats.md#potentials)** stays where it was, so a drained stat may grow back by
[training](#training) and corpses like any other. A temple's **mending** gives every drained
point back at once, with any [stat damage](diseases.md#stat-damage) (see [Services](npcs.md#services)). [Talents](talents.md) help you
hold: **[Unbroken soul](talents.md#talent-unbroken-soul)** (+4 to the check), **[Iron will](talents.md#talent-iron-will)** (+2 to every [Willpower](stats.md#stat-willpower) check), and the
[paladin](classes.md#paladin)'s **[Divine grace](talents.md#talent-divine-grace)** (+2 to the check); see [Talents](talents.md).

The [creatures](creatures.md) that drain:

{{#include generated/drain-attacks.md}}

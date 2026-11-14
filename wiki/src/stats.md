# Stats

You have six **stats**. Each is a number that says how strong, quick, hardy, clever, steadfast, or
striking you are.

{{#include generated/stats.md}}

Your stats are set when you create your character at the start of a run. Your [race](races.md), your
gender, and your [class](classes.md) each add to or take from the average of 10 (see
[Your character](character.md)). After that, they grow by use (see [Raising your stats](#raising-your-stats) below).

## Seeing your stats

Press `@` to open the **character screen**. It shows:

- your name, [race](races.md), gender, and [class](classes.md);
- your age, birthday, and [birth sign](horoscope.md);
- your stats in the order above, with their short names. A stat that a [status effect](status-effects.md#how-they-work) or a
  [disease](diseases.md#how-a-disease-goes) lowers is shown lowered, in red;
- your [hit points](hit-points.md#hit-points), [mana](mana.md#how-much-mana-you-have) (see [Mana](mana.md)), and [Defence](combat.md#defence) (see [Combat](combat.md));
- your weight: your own, plus what you carry (see [Objects and your pack](objects.md)), and how much you can carry (see
  [Burden](burden.md)).

Looking takes no time. Press `@` or `Esc` to close it.

[Creatures](creatures.md) have stats too, but you cannot see theirs in the game. They are on
[Creatures in numbers](creature-numbers.md).

## What they do

Stats matter in a fight (see [Combat](combat.md)), under a load (see [Burden](burden.md)), at a door (see
[Doors](doors.md)), and for your [mana](mana.md#how-much-mana-you-have) (see [Mana](mana.md)):

- **[Strength](stats.md#stat-strength)** helps you [hit](combat.md#hitting) up close, adds to the [damage](combat.md#damage) of your blows, and draws a bow. It counts
  double toward how much you can [carry](burden.md#how-much-you-can-carry). The stronger you are, the sooner you kick a [door](doors.md#kicking-a-door-open) in
  and [dig](digging.md#digging) through a wall.
- **[Dexterity](stats.md#stat-dexterity)** helps you aim a shot or a throw and dodge blows. It adds to your [Defence](combat.md#defence), but less
  under a heavy load or in heavy armour.
- **[Toughness](stats.md#stat-toughness)** gives you [hit points](hit-points.md#hit-points) and makes them come back sooner. It counts toward how much you
  can carry, and fights off most [diseases](diseases.md).
- **[Intelligence](stats.md#stat-intelligence)** helps you find hidden things when you search ([Searching](searching.md)) and adds to your
  mana. It also strengthens the spells whose [ladders](magic.md#the-spells) name it, which covers most spells' damage.
- **[Willpower](stats.md#stat-willpower)** helps you [notice hidden things in passing](searching.md#noticing-in-passing). It adds to your mana and makes it come
  back sooner, and it fights off a few [diseases](diseases.md#how-a-disease-goes) (brain fever). It strengthens the spells whose
  ladders name it: most spells' reach, and some spells' damage or healing.
- **[Presence](stats.md#stat-presence)** moves [shop](shops.md) prices in your favour, or against you if it is low. It adds to your
  mana, and strengthens the spells whose ladders name it: how long a blessing or a hex lasts.

### Modifiers

Most rules use a stat's **modifier** rather than its score: the score minus 10, halved and rounded
down. The game never shows a modifier. Here they are:

{{#include generated/stat-modifiers.md}}

Exactly which rules use the modifier and which use the score:

{{#include generated/stat-uses.md}}

> **Example: a dwarf warrior's stats.** Thora, a level 1 [dwarf](races.md#race-dwarf)
> [warrior](class-warrior.md), starts with these scores and modifiers:
>
> | Strength | Dexterity | Toughness | Intelligence | Willpower | Presence |
> |:---:|:---:|:---:|:---:|:---:|:---:|
> | 15 (+2) | 10 (+0) | 17 (+3) | 9 (−1) | 14 (+2) | 7 (−2) |
>
> - Strength +2: +2 to hit and +2 damage with her long sword ([Combat](combat.md#hitting)).
> - Dexterity +0: her Defence gets nothing from her feet, only from her armour and shield.
> - Toughness 17: 17 + 6 (dwarf) + 7 (warrior) = 30 [hit points](hit-points.md#hit-points), and
>   one back every 360 − 12 × 17 = 156 seconds. With her Strength it sets her
>   [carrying limit](burden.md#how-much-you-can-carry): 1.5 kg × (2 × 15 + 17) = 70.5 kg.
> - Presence −2: [shop](shops.md) prices are 6% against her.

### Potentials

Every stat also has a **potential**: how far it can grow for good. The game never shows it.

- [Magic items](magic-items.md) raise a stat only while you wear them, whatever the potential.
- Training gets more expensive past the potential but carries on. It never moves the potential
  itself (see [Raising your stats](#raising-your-stats)).
- Only elixirs, and rarely a fresh [corpse](food.md#corpses), widen the potential.

Yours are set at the start of a run. Each is the potential your [race](races.md) gives, rolled once. It is
never below your starting score: a score above it raises the potential to match.

{{#include generated/potentials.md}}

An **elixir** raises its stat by 1 and the potential by 1 or 2 more (see [Potions](potions.md#elixirs)).

### What changes them

Stats do not change how far you see or how quietly you walk.

For a while:

- Strong [poison](status-effects.md#poisoned) lowers your [Strength](stats.md#stat-strength) and [Dexterity](stats.md#stat-dexterity) by one while it lasts, for everything they do
  (see [Status effects](status-effects.md)).
- [Hunger](hunger.md#how-hungry-you-are) saps them: Weak lowers your Strength and Dexterity (see [Hunger](hunger.md)).
- [Diseases](diseases.md#how-a-disease-goes) take points off your stats (**stat damage**). These come back a point a day once no
  disease shows (see [Diseases](diseases.md)).

For good:

- Eating the [corpse](food.md#corpses) of some [creatures](creatures.md) may raise a stat, as far as it can grow (see
  [Food and corpses](food.md)).
- [Training](#raising-your-stats) may raise a stat at a new level or at dawn.
- The touch of some undead lowers a stat ([Drained stats](#drained-stats)), and so may a check after
  [deeds that cost marks](#losing-marks).

Nothing else raises or lowers them for good.

## Raising your stats

You never pick a stat to raise. Instead, your stats grow with what you do:

- **[Strength](stats.md#stat-strength)**: fighting up close, landing [criticals](combat.md#criticals), blocking with a shield.
- **[Dexterity](stats.md#stat-dexterity)**: dodging, parrying, shooting.
- **[Toughness](stats.md#stat-toughness)**: being hurt, felling worthy foes.
- **[Intelligence](stats.md#stat-intelligence)**: reading, [searching](searching.md#how-to-search).
- **[Willpower](stats.md#stat-willpower)**: enduring, noticing, casting.
- **[Presence](stats.md#stat-presence)**: talking, trading.

Each such use may earn the stat a **training mark**. A stat's marks are kept as one running total,
its pile. Marks never fade, have no limit, and count the same however many come in a day.

What earns a mark, and how likely:

{{#include generated/stat-training.md}}

Asking someone about a topic takes no time, so it trains nothing. A long action counts only when
it lasts more than one turn and is not interrupted.

Every stat can be trained in more than one way. A fighter who never opens a scroll still sharpens
Intelligence by exploring, searching, foraging, and disarming traps. A wizard grows stronger by
wading, hauling, and struggling free of a grip. Casting a spell you know trains the stats the spell
grows with ([its ladders](spellcasting.md#how-a-spell-grows)): Intelligence for most, Willpower for those that reach far, Presence for
hexes and blessings that last. Some uses train two stats at once, such as disarming a trap or
struggling against a hold.

### The price of a point

Each stat's next point has a **price** in marks: **20 + 10 × the stat's score**, up to its
[potential](stats.md#potentials). Past the potential the price climbs quickly. The first point beyond it costs twice the
plain price, the second 4 ×, the third 7 ×, then 11 ×, 16 ×, 22 ×. (With `n` the point past the
potential, that is (2 + n × (n + 1)) / 2 times the plain price.)

Training alone never raises the potential. So a stat you live by ends a long career of sixty
levels some twelve or thirteen points past its potential, at about 30. It never reaches the top by
use alone. An [elixir](potions.md#elixirs) or a rare [corpse](food.md#corpses) that raises the potential makes every later point cheaper.

{{#include generated/stat-prices.md}}

The last column, the drop, is what a stat's pile pays back if the stat ever falls by a point: half
the plain price of the point lost, never more past the potential.

### The checks

Every time you gain an [experience level](experience.md), and once a day at **06:00** (the hour a
[rest](resting.md#how-resting-works) until dawn ends), each stat in turn gets one **check**. The rule is the same for both.

- A pile under a quarter of the price is out of play: nothing happens.
- From a quarter up, the stat rises by a point with chance **marks³ / (marks³ + price³)**. That is
  even odds at the price, about one in nine at half of it, and near certain at twice it.

{{#include generated/stat-odds.md}}

A rise **pays the price** out of the pile. Whatever is left over waits for the next check, so
nothing is wasted. A lucky rise from a short pile spends what there was and no more. A stat rises
at most one point per check, so a great pile becomes a point a day and one a level.

The game says only "You feel stronger." (or quicker, tougher, sharper, more resolute, more
confident). The new score shows on the character screen. A rise in [Toughness](stats.md#stat-toughness) brings its [hit point](hit-points.md#hit-points)
at once, and a rise at 06:00 stops a rest.

The daily check comes whether you are awake or resting, once a day at that hour and no other.
Nothing decays and nothing is capped, so the only limit is the clock.

How many days the next point takes, by the marks a stat earns a day (the daily check alone; a
level's check comes on top):

{{#include generated/stat-pace.md}}

The **[Gourmand](talents-body.md#talent-gourmand)** [talent](talents.md) doubles the marks a fresh corpse gives.

### Losing marks

A few deeds take marks away instead, each from one stat, so you can lower a stat on purpose if you
want to. They are: starving, fainting, carrying far too much, walking onto traps you know are
there, and getting caught stealing. Bad luck alone never costs a mark: a hidden trap, a blow, a
disease, or a drain takes none. Each deed takes its marks at once, or a mark with the chance shown:

{{#include generated/stat-losses.md}}

The pile then goes below zero. The checks read it the same way, against the **drop's price**: half
the plain price of the point lost, never more past the potential.

Nothing happens while the pile is less than a quarter of that below zero. The first time a deed
takes it that far, the game warns you in words: "You feel your strength wasting.", "Your hands
feel less sure.", "You feel your health failing.", "Your mind feels dull.", "Your will feels
brittle.", "You feel small and unregarded." So a warning always comes before a drop.

From there, a check lowers the stat by a point with the same odds as a rise: "You feel weaker."
(or clumsier, frailer, duller; "Your resolve wavers.", "You feel less sure of yourself."). The drop
pays its price back toward zero, never past it.

A drop never takes a stat below 3, nor below the [stat damage](diseases.md#how-a-disease-goes) on it plus one. It never lowers the
potential, so a point lost below the potential is won back at its plain price. Your maximum
[hit points](hit-points.md#hit-points) and [mana](mana.md) fall with it, but a drop never kills.

At 14, losing a point takes 75 marks of deeds (five dawns greeted Weak), and winning it back takes
150 marks of use. So there is nothing to gain by going up and down.

## Drained stats

Some of the undead drink the life out of the living. When such a [creature](creatures.md)'s touch hits you and you
live, you make a **check** of a stat, [Willpower](stats.md#stat-willpower) unless the creature says otherwise. You roll 2d12 plus the
stat's [modifier](stats.md#modifiers) against the drain's difficulty, as for a [disease](diseases.md#how-a-disease-goes)'s check.

- If you hold: "A deathly chill passes through you." Nothing is lost.
- If not, points come off one of your stats **for good** ("You feel your strength drain away!").
  Your score is simply lower from then on, for everything it does.

A drain of [Toughness](stats.md#stat-toughness) lowers your maximum [hit points](hit-points.md#hit-points). A drain of [Intelligence](stats.md#stat-intelligence), Willpower, or [Presence](stats.md#stat-presence) lowers
your maximum [mana](mana.md#how-much-mana-you-have). A drain never kills you, and never takes a score below 3 (nor below what
the [stat damage](diseases.md#stat-damage) on it needs). [Resting](resting.md#how-resting-works) never brings drained points back.

Your **[potential](stats.md#potentials)** stays where it was, so a drained stat can grow back by [training](#raising-your-stats) and corpses
like any other. A temple's **mending** gives every drained point back at once, along with any
stat damage (see [Services](npcs.md#services)).

[Talents](talents.md) help you hold: **[Unbroken soul](talents-body.md#talent-unbroken-soul)** (+4 to the check), **[Iron will](talents-body.md#talent-iron-will)** (+2 to every Willpower check),
and the [paladin](class-paladin.md)'s **[Divine grace](class-paladin.md#talent-divine-grace)** (+2 to the check).

The creatures that drain:

{{#include generated/drain-attacks.md}}

See also: [Combat](combat.md), [Skills](skills.md), and [Your character](character.md).

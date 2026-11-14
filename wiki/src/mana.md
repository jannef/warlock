# Mana

Mana is your second pool, beside your [hit points](hit-points.md#hit-points), and it works the
same way. You have a maximum, you spend it, and it comes back by itself over time.

[Spells](magic.md#the-spells) spend it: those your class gives (see
[Classes](classes.md#levels)) and those you learn from [scrolls](scrolls.md#spell-scrolls).
[Rods](wands-and-rods.md#rods) spend it too. Each costs its mana, and with too little you cannot
cast it until enough has come back.

The [status bar](the-screen.md#the-screen) shows your mana right after your hit points, as
`MP 5/12`: what you have left out of your maximum. The [character screen](character.md) (`@`)
shows it beside your hit points.

Creatures may have mana too, but you never see theirs. [Magic items](magic-items.md) work for a
creature exactly as they do for you. One that wears or wields a "+5 to maximum mana" ring has 5
more mana (and starts with all of it). One whose amulet makes "mana come back 50% faster" regains
it half again as fast. A summoner pays for its summons from that larger pool.

## How much mana you have

Your maximum mana is what your [race](races.md) and your [class](classes.md) give, plus what your
[Intelligence](stats.md#stat-intelligence), [Willpower](stats.md#stat-willpower), and
[Presence](stats.md#stat-presence) add. High scores add to it and low ones take from it, but it
never goes below zero.

[Wizards](class-wizard.md) and [witches](class-witch.md) have the most, and
[aristocrats](class-aristocrat.md) a surprising amount. Every class gives at least 1, and
[warriors](class-warrior.md), [berserkers](class-berserker.md), [thieves](class-thief.md),
[commoners](class-commoner.md), and [craftspeople](class-craftsperson.md) give no more than
that. Only [elves](races.md#race-elf), [gnomes](races.md#race-gnome),
[half-elves](races.md#race-half-elf), [halflings](races.md#race-halfling), and
[humans](races.md#race-human) get any mana from their race.

{{#include generated/mana.md}}

Each [level](experience.md#levels) up to 60 raises your maximum, and the mana you have now, by
half your class's mana plus what the better of your Intelligence and Willpower adds (see
[Experience](experience.md#levelling-up)). So a wizard keeps growing their mana after
[hit points](hit-points.md#hit-points) stop at level 50, while a warrior
with ordinary wits gains none.

[Magic items](magic-items.md) that add to it ("+5 to maximum mana") raise your maximum, not what
you have now. The talent *[Deep well](talents.md#talent-deep-well)* raises both: by 1 for every 4
character levels, at least 2, when you take it and again as each later
level comes (see [Talents](talents.md)).

The skill *[Concentration](skills.md#skill-concentration)*, once you are trained in it, deepens
your pool a little: +1 mana at 25, +5 at 100, +8 at 200. It never adds more than a fifth of what
your pool would be without it, so a pool under 5 gains nothing. Untrained, it gives nothing, even
when an item raises it (see [Skills](skills.md) for the table by value).

> **Example: an elf wizard's mana.** Tathren, a level 1 elf
> wizard, has Intelligence 15 (+2), Willpower 12 (+1), and Presence 10 (+0)
> ([modifiers](stats.md#modifiers)).
>
> - **Her maximum:** 3 (elf) + 12 (wizard) + 2 + 1 + 0 = **18 mana**: three casts of
>   [Burning hands](magic.md#spell-burning-hands) at 6 mana each.
> - **Each level:** half the wizard's 12, plus the better of her Intelligence and Willpower
>   modifiers: 6 + 2 = **8 more**.
> - **Coming back:** with Willpower 12, a point every 360 − 12 × 12 = 216 seconds (3 minutes and
>   36 seconds) at the usual pace, so an empty pool fills in about an hour.

## How fast it comes back

At the usual pace, a point of mana comes back every so often. The stronger your
[Willpower](stats.md#stat-willpower), the faster it comes:

{{#include generated/mana-regeneration.md}}

[Resting](resting.md#how-resting-works), some [status effects](status-effects.md#how-they-work)
and [diseases](diseases.md), [starving](hunger.md#how-hungry-you-are), and
[special rooms](special-rooms.md) change the pace:

{{#include generated/mana-paces.md}}

When several things change the pace at once:

- If several status effects slow it, only the slowest counts.
- A disease that slows it more than being [Diseased](status-list.md#status-diseased) does counts
  too.
- Resting makes up for some of the slowing, as it does for healing, and so does a
  [sanctuary](special-rooms.md#room-sanctuary).
- These speed it up on top of all that, their percents added together:
  - [magic items](magic-items.md) that speed it ("mana comes back 50% faster");
  - being born under [Water](horoscope.md#elements), whose gift
    *[Tidal](horoscope.md#talent-tidal)* makes mana come back 25 % faster (see [Horoscope](horoscope.md));
  - the talent *[Clear mind](talents.md#talent-clear-mind)*, 25 % faster;
  - trained *[Concentration](skills.md#skill-concentration)*: 5 % faster at 20, 40 % at 100, 60 %
    at 200 (see [Skills](skills.md)).
- The talent *[Meditation](talents.md#talent-meditation)* makes resting bring mana back half again
  as fast (resting's pace times 1.5).

Nothing else changes it: not your [burden](burden.md), not the [cold](temperature.md), and not
your [combat stance](stances.md#stances). Spending mana now and then trains Concentration.

## Spending it

Anything that costs mana is refused when you have too little, and then spends nothing. Spending
any mana trains your [Willpower](stats.md#stat-willpower): each time is a chance at a training mark
(see [Stats](stats.md#raising-your-stats)).

The talent *[Frugal caster](talents.md#talent-frugal-caster)* makes abilities cost a fifth less
mana, rounded to the nearest point and never below 1: 5 becomes 4, 6 becomes 5, 10 becomes 8.
[Magic items](magic-items.md) *of Thrift* make abilities cheaper in the same way, by their percent.

The talent and everything you wear are added up first, then rounded once. Together they never
take off more than half. For example, Frugal caster (20 %) and a 10 % ring take 30 % off, so 7
becomes 5. Three 20 % items and the talent still take off only 50 %, so 10 becomes 5. An item of
Thrift works even before you have identified it, and stops when you take it off.

This cut comes last, after everything else that changes the cost of a spell or a rod (see
[What a spell costs](magic.md#what-a-spell-costs)). It applies to every ability that costs mana:
spells, [rods](wands-and-rods.md#rods), and summoning.

The talent *[Blood magic](talents.md#talent-blood-magic)* gives you an ability (in the
[abilities menu](abilities.md#the-abilities-menu)) that turns 5 of your hit points into 3 mana at
once. It takes as long as a wait. It never takes your last hit point, so with 5 or fewer it refuses.
It never brings your mana above your maximum, and does nothing while your mana is full. It is not
a wound: nothing soaks it, and it trains nothing.

## Mana on a kill

[Magic items](magic-items.md) *of Siphoning* give you some mana when you kill something close by,
never above your maximum. Only your own kills count, by your blow, your shot or throw, or your
spell, and the one you killed must stand **within 3 cells** of you.

Distance is measured round, as for every radius: 3 cells in a straight line, 3 across and 1 aside,
or 2 diagonally, but not 3 across and 2 aside.

You get nothing for a creature that dies of [poison](status-list.md#status-poisoned) or
[bleeding](status-list.md#status-bleeding) you caused, one a [trap](traps.md) kills, or one
your summoned creature kills, nor for a kill farther off. Two such items add up, identified or not.
A creature wearing one refills its own mana the same way when it kills.

A [witch](class-witch.md)'s [Malison](class-witch.md#talent-malison) (from level 18) pays her mana
for every death under one of her [hexes](status-list.md#status-ill-wished), however it died,
whoever killed it, and at any distance. She gets half the creature's level (rounded down, at least
1), never above her maximum. This adds to what an item of Siphoning gives.

## When the most falls

- Taking off a [magic item](magic-items.md) that raised your maximum lowers the mana you have to
  the new maximum, if need be.
- A [disease](diseases.md) that damages your Intelligence, Willpower, or Presence lowers your
  maximum, and the mana you have with it.
- A [status effect](status-effects.md#how-they-work) that lowers one of those stats lowers your
  maximum only while it lasts. You keep the mana you had, but none comes back above the lower
  maximum until the effect passes.

## The fine print

- Mana comes back on game time, like [hit points](hit-points.md#hit-points), whatever you are doing:
  waiting, walking, fighting, or making a long journey. Only the total time counts, not how it is
  split up.
- Progress toward the next point carries over from turn to turn. It is lost once you are full.
- You never see how fast it comes back as a number, only your mana rising.

See also [Healing](healing.md) and [Stats](stats.md).

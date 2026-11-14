# Fighting styles

A sword is one thing; how you hold it is another. A **fighting style** is the way you fight: with
one blade and a free hand, a weapon in each hand, a great two-handed weapon, behind a shield, with
a bow, by throwing, with your fists, or with a staff.

Each style is a skill like a [weapon skill](weapon-skills.md). It has a **rank** from 0 (untrained) to 10, **marks**
toward the next rank, and benefits per rank that add to your weapon skill's.

## The styles

{{#include generated/styles.md}}

You train a style by fighting in it. Every attack you make while you hold your weapons its way
earns marks in the style as well as in the weapon's own skill. The rules are the same: more for a
hit, more against stronger opponents (see [Raising a weapon skill](weapon-skills.md#raising-a-weapon-skill)). A [dagger](weapon-skills.md#weapon-skill-daggers) alone in your hand
trains [Duelist](fighting-styles.md#style-duelist). The same dagger beside a sword trains [Twin Blades](fighting-styles.md#style-twin-blades). A shield alone is no style at
all, and neither is a [shield bash](abilities.md#ability-shield-bash) with an empty other hand.

- **[Duelist](fighting-styles.md#style-duelist)**: one one-handed weapon and nothing in the other hand. You [parry](combat.md#defence) better and hit
  more often.
- **[Twin Blades](fighting-styles.md#style-twin-blades)**: a weapon in each hand. See below.
- **[Great Weapon](fighting-styles.md#style-great-weapon)**: any two-handed weapon except a staff, such as the great sword or the great
  axe. You hit harder, your [criticals](combat.md#criticals) even more so, and you swing faster.
- **[Sword and Board](fighting-styles.md#style-sword-and-board)**: a one-handed weapon and a [shield](objects.md#class-shields). The shield protects better, and a blow that
  hits may be followed by a free [shield bash](abilities.md#ability-shield-bash) (which trains [shield bashing](weapon-skills.md#the-skills)).
- **Two Shields**: a shield in each hand. See [Two shields](#two-shields) below.
- **[Marksman](fighting-styles.md#style-marksman)**: shooting a [bow](weapon-skills.md#weapon-skill-bows), a [crossbow](weapon-skills.md#weapon-skill-crossbows), or a [sling](weapon-skills.md#weapon-skill-slings), whatever your hands hold. Shots hit more often
  and take less time, and a crossbow reloads faster.
- **[Skirmisher](fighting-styles.md#style-skirmisher)**: throwing anything at all. Throws hit more often and harder, and take less time.
  The style does not care what armour you wear or what you hold.
- **[Brawler](fighting-styles.md#style-brawler)**: nothing in your hands at all. Fists hit more often and harder.
- **[Staff Fighter](fighting-styles.md#style-staff-fighter)**: a staff in both hands. You [parry](combat.md#defence) better and hit more often.

Shooting and throwing are styles of their own. So a marksman with a sword at their side trains both
[Marksman](fighting-styles.md#style-marksman) (with shots) and [Duelist](fighting-styles.md#style-duelist) (with blows). A kick is no style: you kick whatever your hands
hold, so it trains only [unarmed fighting](weapon-skills.md#the-skills), and [Brawler](fighting-styles.md#style-brawler) does not help it.

## Caster styles

Spells have styles too, decided by the spell and your armour: Sniper, Toucher, and Battle caster. They
are on the [Schools and caster styles](schools.md#caster-styles) page, with their tables.

## Two weapons

Your off hand takes a shield or a **light** weapon (listed below; see the [weapons](combat.md#damage) table:
"main hand or off hand"). With a weapon in each hand, every attack you make up close is two blows.
The main hand strikes first, then the off hand, each at a penalty to hit.

The off hand's blow is made with its own weapon and trains that weapon's skill. A magic weapon's
own powers (see [Magic items](magic-items.md)) go with its own blow, while what your other gear gives helps both.
Each blow can draw blood and open a [bleeding](status-effects.md#bleeding) wound on its own. Both blows are one attack, in the
time of one. If the first blow kills, there is no second.

{{#include generated/light-weapons.md}}

[Twin Blades](fighting-styles.md#style-twin-blades) eases both penalties. From rank 7 it adds to hit as well, and from rank 9 neither
hand has any penalty: a full attack with each. Creatures that hold two weapons fight with both too.

## Two shields

A shield fits either hand. With one on each arm you fight in **Two Shields**. Nobody starts with
this style. It is not listed on the weapon skills screen, and neither are the talents that need it,
until you earn your first marks in it.

Every attack you make up close is a **[shield bash](abilities.md#ability-shield-bash)** with the first shield. It happens by itself; you need
not choose Shield bash from the abilities menu. It draws on and trains both [shield bashing](weapon-skills.md#the-skills) and
Two Shields. After a bash that hits, the other shield may follow with a bash of its own, more often
as the style rises.

The style protects well (its Shield adds to your shields'), and it hits harder. From rank 4, a blow
your shields turn [staggers](status-list.md#status-staggered) its striker (*[Staggering block](fighting-styles.md#style-ability-staggering-block)*). But two shields are clumsy:

- every bash is at a penalty to hit;
- every attack takes longer;
- a spell you cast with your own [mana](mana.md#how-much-mana-you-have) may fail, because your hands are full of shields. The
  mana and the time are spent all the same.

The style eases all three, and from rank 9 none is left. A [scroll](scrolls.md) you read or a [wand](wands-and-rods.md#wands) you use never
fails this way.

## What a rank gives

As with weapon skills, the game never tells you in numbers what a rank gives. These are the exact
rules. A style's benefits add to those of the weapon skill you fight with:

- **[To hit](combat.md#hitting)**, **[Damage](combat.md#damage)**, **Time**, **[Parry](combat.md#defence)**, **[Shield](combat.md#defence)**, and **Reload time** work as for
  [weapon skills](weapon-skills.md#what-a-rank-gives).
- **Critical damage**: a confirmed critical rolls its damage that many more times (×3 instead of
  ×2, ×4 at Great Weapon 10; see [Criticals](combat.md#criticals)).
- **Main hand** and **Off hand**: the penalty to hit of each hand's blow with two weapons.
- **Shield bash after a hit**: the chance that a blow that hits is followed by a free bash with your
  shield. With two shields, it is the chance that a bash that hits is followed by one with the
  other.
- **Bash penalty**, **Slower attacks**, and **Spell failure**: with two shields, the penalty to hit
  of every bash, how much longer every attack takes, and the chance that a spell you cast with your
  own [mana](mana.md#how-much-mana-you-have) fails.

{{#include generated/style-benefits.md}}

Rank 0 gives nothing. The first five ranks grow slowly, and ranks 6 to 10 grow much faster. Rank
10 is mastery.

## What a rank teaches

A style's rank teaches abilities the same way a weapon skill's does (see
[What a rank teaches](weapon-skills.md#what-a-rank-teaches)). You are told with the rank, and the [abilities menu](abilities.md) lists them after the
style's name. Most fire by themselves, free and instant, but only while you fight in the style. A
duelist's [riposte](fighting-styles.md#style-ability-riposte) needs one weapon and an empty hand. [Staggering block](fighting-styles.md#style-ability-staggering-block) needs a weapon and a shield.

The *active* ones are used from the menu for [stamina](stances.md#stamina) and time, like a weapon skill's (see
[What a rank teaches](weapon-skills.md#what-a-rank-teaches)), and only in the style's grip:

- **[False lead](fighting-styles.md#style-ability-false-lead)** is one blow rolled as if the foe had no Parry and no Shield.
- **[Whirl](fighting-styles.md#style-ability-whirl)** and **[Staff sweep](fighting-styles.md#style-ability-staff-sweep)** strike every foe beside you, clockwise from the north. Each blow is at
  −2 to hit and brings the [free blows](stances.md#free-blows) the end of an attack brings. With a blade in each hand,
  the blades take turns.
- **[Mighty swing](fighting-styles.md#style-ability-mighty-swing)** rolls the blow's damage dice twice, at −2 to hit. Its bonus and extra effects
  count once, and it is not a [critical](combat.md#criticals).
- **[Shield rush](fighting-styles.md#style-ability-shield-rush)** (a [shield bash](abilities.md#ability-shield-bash)) and **[Hip throw](fighting-styles.md#style-ability-hip-throw)** (an unarmed blow) drive a foe they hit one cell
  back, if that cell is free, and [stagger](status-list.md#status-staggered) it. Against a wall or another creature, the foe stays
  where it is and is not staggered, but the blow lands all the same.

{{#include generated/style-abilities.md}}

Two abilities that do the same thing never add up:

- A [riposte](fighting-styles.md#style-ability-riposte) from [Duelist](fighting-styles.md#style-duelist) and one from a [talent](talents.md#talent-riposte) are one riposte.
- [Staggered](status-list.md#status-staggered) is never more than one slowed action, however many blows staggered the foe.
- A new [pin](status-list.md#status-pinned) keeps the longer of the two and never adds.
- [Rend](fighting-styles.md#style-ability-rend) comes once per attack, when both blades hit and the foe lives. A [shield bash](abilities.md#ability-shield-bash) is not a second
  blade.
- [Light-footed](status-list.md#status-light-footed) lasts for your next action only.
- [Spell reflection](fighting-styles.md#style-ability-spell-reflection) turns back at most one spell per turn of yours, and a spell turned back is never
  turned back again. So two shield-bearers cannot bat a bolt between them.

## Where your styles come from

Your [class](classes.md) starts you with a rank in a style or two. A [warrior](class-warrior.md) fights with
[sword and board](fighting-styles.md#style-sword-and-board), a [berserker](class-berserker.md) with a [great weapon](fighting-styles.md#style-great-weapon), and a [ranger](class-ranger.md) is a [marksman](fighting-styles.md#style-marksman). A [thief](class-thief.md) has learned
either [Twin Blades](fighting-styles.md#style-twin-blades) or [Duelist](fighting-styles.md#style-duelist), and an [ascetic](class-ascetic.md) either [Brawler](fighting-styles.md#style-brawler) or [Staff Fighter](fighting-styles.md#style-staff-fighter); the dice decide. The
[weapon skills screen](weapon-skills.md#seeing-your-skills) (`W`) lists your styles below your weapon skills, and marks the ones you
fight in now with a `•`.

See also: [Combat](combat.md), [Schools and caster styles](schools.md), and
[Stances and stamina](stances.md).

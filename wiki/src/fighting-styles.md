# Fighting styles

A sword is one thing; how you hold it is another. As in the old *Complete Fighter's Handbook*
of AD&D, Warlock has **fighting styles**: the way you fight, with one blade and a free hand,
with a weapon in each hand, with a great two-handed weapon, behind a shield, with a bow, by
throwing, with your fists, or with a staff. Each style is a skill like a
[weapon skill](weapon-skills.md): a **rank** from 0 (untrained) to 10, **marks** toward the next
rank, and benefits per rank that add to your weapon skill's.

## The styles

{{#include generated/styles.md}}

A style is trained by fighting in its configuration: every attack you make while you hold your
weapons its way earns marks in the style as well as in the weapon's own skill, by the same rules
(more for a hit, more against stronger opponents; see [Marks and ranks](weapon-skills.md#marks-and-ranks)).
A [dagger](weapon-skills.md#weapon-skill-daggers) alone in your hand trains [Duelist](fighting-styles.md#style-duelist); the same dagger beside a sword trains [Twin Blades](fighting-styles.md#style-twin-blades).
A shield alone is no style at all, and neither is a [shield bash](abilities.md#ability-shield-bash) with an empty other hand.

- **[Duelist](fighting-styles.md#style-duelist)**: one one-handed weapon and nothing in the other hand. It [parries](combat.md#defence) better and hits
  more often.
- **[Twin Blades](fighting-styles.md#style-twin-blades)**: a weapon in each hand. See below.
- **[Great Weapon](fighting-styles.md#style-great-weapon)**: any two-handed weapon but a staff, such as the great sword or the great axe.
  It hits harder, its [criticals](combat.md#criticals) more so, and it swings faster.
- **[Sword and Board](fighting-styles.md#style-sword-and-board)**: a one-handed weapon and a [shield](objects.md#class-shields). The shield protects better, and a blow
  that hits may be followed by a [shield bash](abilities.md#ability-shield-bash) for free (it trains
  [shield bashing](weapon-skills.md#the-skills)).
- **Two Shields**: a shield in each hand. See [Two shields](#two-shields) below.
- **[Marksman](fighting-styles.md#style-marksman)**: shooting a [bow](weapon-skills.md#weapon-skill-bows), a [crossbow](weapon-skills.md#weapon-skill-crossbows), or a [sling](weapon-skills.md#weapon-skill-slings), whatever your hands hold. Shots hit more
  often and take less time, and a crossbow reloads faster.
- **[Skirmisher](fighting-styles.md#style-skirmisher)**: throwing, anything at all. Throws hit more often and harder, and take less
  time. (A skirmisher is the one who throws; the style does not care what armour you wear or what
  you hold.)
- **[Brawler](fighting-styles.md#style-brawler)**: nothing in your hands at all. Fists hit more often and harder.
- **[Staff Fighter](fighting-styles.md#style-staff-fighter)**: a staff in both hands. It [parries](combat.md#defence) better and hits more often.

Shooting and throwing are styles of their own, so a marksman with a sword at their side trains
both [Marksman](fighting-styles.md#style-marksman) (shots) and [Duelist](fighting-styles.md#style-duelist) (blows). A kick is no style: you kick whatever your hands hold,
so it trains only [unarmed fighting](weapon-skills.md#the-skills), and [Brawler](fighting-styles.md#style-brawler) does not help it.

## Caster styles

[Spells that roll to hit](magic.md#how-a-spell-finds-its-target) have styles too: how you cast them. Each trains, as the
[Spell attacks](weapon-skills.md#the-skills) skill does, on every such spell you cast with your
own [mana](mana.md#how-much) that reaches a creature, and adds to it.

- **[Battle caster](fighting-styles.md#style-battle-caster)**: a spell that rolls to hit, cast while you wear armour that makes spells
  dearer (mail, plate, a great helm, a large shield). It comes first: in such armour every ray,
  bolt, and touch is a battle caster's. It hits more often, and it makes your armour's extra
  mana cost smaller for **every** spell you cast, whatever it is. It also trains a little on
  every other spell you cast in such armour. From rank 4 a spell that lands wards you: a point
  of [Defence](combat.md#defence) until your next action (*[Spell guard](fighting-styles.md#style-ability-spell-guard)*). A *Mage-wrought* piece eased so far that it
  makes spells no dearer is not such armour: in it you fight as a [Sniper](fighting-styles.md#style-sniper) or a [Toucher](fighting-styles.md#style-toucher).
- **[Sniper](fighting-styles.md#style-sniper)**: a ray or a bolt, out of armour. It hits more often, and better still against a
  target five cells away or more, where its [critical range](combat.md#criticals) is wider too. From rank 4 its best
  shots pin the target where it stands (*[Pinning ray](fighting-styles.md#style-ability-pinning-ray)*).
- **[Toucher](fighting-styles.md#style-toucher)**: a touch, out of armour. It hits more often, its [critical range](combat.md#criticals) is wider, and its
  criticals roll the damage up to twice more. From rank 4 its criticals [stagger](status-effects.md#status-staggered) (*[Staggering
  touch](fighting-styles.md#style-ability-staggering-touch)*); at rank 10 every touch that lands [weakens](status-effects.md#status-weakened) the foe (*[Withering touch](fighting-styles.md#style-ability-withering-touch)*).

The [wizard](classes.md#wizard) and the [druid](classes.md#druid) start in [Sniper](fighting-styles.md#style-sniper), the [witch](classes.md#witch) and the [priest](classes.md#priest) in [Toucher](fighting-styles.md#style-toucher), the [paladin](classes.md#paladin) in
[Battle caster](fighting-styles.md#style-battle-caster); any caster may advance in any of them by casting.

## Two weapons

Your off hand takes a shield, or a **light** weapon (listed below; see the
[weapons](combat.md#damage) table: "main hand or off hand"). With a weapon in each hand, every
attack you make up close is two blows: the main hand's first, then the off hand's, each at a
penalty to hit. The off hand's blow is made with its own weapon and trains that weapon's skill;
a magic weapon's own powers (see [Magic items](magic-items.md)) go with its own blow, while what
your other gear gives helps both. Each blow can draw blood and open a [bleeding](status-effects.md#bleeding) wound on its own.
Both blows are one attack, in the time of one. If the first blow kills, there is no second.

{{#include generated/light-weapons.md}}

[Twin Blades](fighting-styles.md#style-twin-blades) eases both penalties, and from rank 9 neither hand has any: a full attack with each.
From rank 7 it adds to hit as well.
Creatures that hold two weapons fight with both too.

## Two shields

A shield fits either hand. With one on each arm you fight in **Two Shields**, a style nobody
starts with: it is not listed on the weapon skills screen, nor are the talents that ask for it,
until your first marks in it. Every attack you make up close is a **[shield bash](abilities.md#ability-shield-bash)** with the first
shield, by itself (you need not choose Shield bash from the abilities menu), drawing on and
training [shield bashing](weapon-skills.md#the-skills) and Two Shields both. After a bash that
hits, the other shield may follow it with a bash of its own, more often as the style rises.

The style protects well (its Shield adds to your shields'), hits harder, and from rank 4 a blow
your shields turn [staggers](status-effects.md#status-staggered) its striker (*[Staggering block](fighting-styles.md#style-ability-staggering-block)*). But two shields are clumsy: every
bash is at a penalty to hit, every attack takes longer, and a spell you cast with your own [mana](mana.md#how-much)
may fail, your hands full of shields: the mana and the time are spent all the same. The style
eases all three, and from rank 9 none is left. A [scroll](scrolls.md) read or a [wand](magic.md#wands) used never fails so.

Talents for those deep in the style (see [Talents](talents.md)): *[Loose straps](talents.md#talent-loose-straps)* (Two Shields 5)
makes a spell fail far less often, *[Anvil and hammer](talents.md#talent-anvil-and-hammer)* (Two Shields 6) brings the second shield's
bash more often, and *[Craven's wall](talents.md#talent-cravens-wall)* (Two Shields 8) makes both shields guard you at once while
you fight in the coward's [stance](combat.md#stances).

## What a rank gives

As with weapon skills, the game never tells you what a rank gives in numbers. These are the
rules, exactly. A style's benefits add to those of the weapon skill you fight with:

- **[To hit](combat.md#hitting)**, **[Damage](combat.md#damage)**, **Time**, **[Parry](combat.md#defence)**, **[Shield](combat.md#defence)**, and **Reload time** as for
  [weapon skills](weapon-skills.md#what-a-rank-gives).
- **Critical damage**: a confirmed critical rolls its damage that many more times (×3 instead
  of ×2, ×4 at Great Weapon 10; see [Criticals](combat.md#criticals)).
- **Main hand** and **Off hand**: the penalty to hit of each hand's blow with two weapons.
- **Shield bash after a hit**: the chance that a blow that hits is followed by a free bash with
  your shield (with two shields, a bash that hits by one with the other).
- **Bash penalty**, **Slower attacks**, and **Spell failure**: with two shields, the penalty to
  hit of every bash, how much longer every attack takes, and the chance a spell you cast with
  your own [mana](mana.md#how-much) fails.

{{#include generated/style-benefits.md}}

The first five ranks grow slowly; ranks 6 to 10 grow much faster.

Rank 0 gives nothing. Rank 10 is mastery.

## What a rank teaches

A style's rank teaches abilities as a weapon skill's does (see
[What a rank teaches](weapon-skills.md#what-a-rank-teaches)): told with the rank, listed in the
[abilities menu](abilities.md) after the style's name, and firing by themselves, free and
instant, but only while you fight in the style: a duelist's [riposte](fighting-styles.md#style-ability-riposte) needs one weapon and an empty
hand, [Staggering block](fighting-styles.md#style-ability-staggering-block) a weapon and a shield. The *active* ones are used from the menu for
[stamina](combat.md#stamina) and time, as a weapon skill's are (see
[What a rank teaches](weapon-skills.md#what-a-rank-teaches)), and only in the style's grip:

- **[Feint](fighting-styles.md#style-ability-feint)** is one blow rolled as if the foe had no Parry and no Shield.
- **[Whirl](fighting-styles.md#style-ability-whirl)** and **[Staff sweep](fighting-styles.md#style-ability-staff-sweep)** strike every foe beside you, clockwise from the north, each at
  −2 to hit and each with the [free blows](combat.md#free-blows) the end of an attack brings; with a blade in each hand,
  the blades take turns.
- **[Mighty swing](fighting-styles.md#style-ability-mighty-swing)** rolls the blow's damage dice twice (its bonus and riders once; it is no
  [critical](combat.md#criticals)), at −2 to hit.
- **[Shield rush](fighting-styles.md#style-ability-shield-rush)** (a [shield bash](abilities.md#ability-shield-bash)) and **[Hip throw](fighting-styles.md#style-ability-hip-throw)** (an unarmed blow) drive a foe they hit a cell
  back, if that cell is free, and [stagger](status-effects.md#status-staggered) it; against a wall or another creature it stays where
  it is, unstaggered, but the blow lands all the same.

{{#include generated/style-abilities.md}}

Two that do the same never add up: a [riposte](fighting-styles.md#style-ability-riposte) from [Duelist](fighting-styles.md#style-duelist) and one from a [talent](talents.md#talent-riposte) are one riposte.
[Staggered](status-effects.md#status-staggered) is never more than one slowed action, however many blows staggered the foe. A new [pin](status-effects.md#status-pinned)
keeps the longer of the two and never adds. [Rend](fighting-styles.md#style-ability-rend) comes once an attack, when both blades hit and
the foe lives; a [shield bash](abilities.md#ability-shield-bash) is no second blade. [Light-footed](status-effects.md#status-light-footed) lasts for your next action only.
[Spell reflection](fighting-styles.md#style-ability-spell-reflection) turns at most one spell a turn of yours, and a spell turned back is never
turned back again, so two shield-bearers cannot bat a bolt between them.

## Related styles

Styles default to each other as weapon skills do (see
[Related skills](weapon-skills.md#related-skills)), but never to a weapon skill: when you gain a
rank in one, each related style rises to your new rank less the penalty, if that is more than you
have in it.

{{#include generated/style-defaults.md}}

## Where your styles come from

Your [class](classes.md) starts you with a rank in a style or two: a [warrior](classes.md#warrior) fights with [sword
and board](fighting-styles.md#style-sword-and-board), a [barbarian](classes.md#barbarian) with a [great weapon](fighting-styles.md#style-great-weapon), a [ranger](classes.md#ranger) is a [marksman](fighting-styles.md#style-marksman), and a [thief](classes.md#thief) has learned
either [Twin Blades](fighting-styles.md#style-twin-blades) or [Duelist](fighting-styles.md#style-duelist) (the dice decide, as do a [monk's](classes.md#monk) [Brawler](fighting-styles.md#style-brawler) or [Staff Fighter](fighting-styles.md#style-staff-fighter)). The
[weapon skills screen](weapon-skills.md#seeing-your-skills) (`W`) lists your styles below your
weapon skills, and marks the ones you fight in now with a `•`.

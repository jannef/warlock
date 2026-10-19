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
A dagger alone in your hand trains Duelist; the same dagger beside a sword trains Twin Blades.
A shield alone is no style at all, and neither is a shield bash without a weapon in the other
hand.

- **Duelist**: one one-handed weapon and nothing in the other hand. It parries better and hits
  more often.
- **Twin Blades**: a weapon in each hand. See below.
- **Great Weapon**: any two-handed weapon but a staff, such as the great sword or the great axe.
  It hits harder, its criticals more so, and it swings faster.
- **Sword and Board**: a one-handed weapon and a shield. The shield protects better, and a blow
  that hits may be followed by a shield bash for free (it trains
  [shield bashing](weapon-skills.md#the-skills)).
- **Marksman**: shooting a bow, a crossbow, or a sling, whatever your hands hold. Shots hit more
  often and take less time, and a crossbow reloads faster.
- **Skirmisher**: throwing, anything at all. Throws hit more often and harder, and take less
  time. (A skirmisher is the one who throws; the style does not care what armour you wear or what
  you hold.)
- **Brawler**: nothing in your hands at all. Fists hit more often and harder.
- **Staff Fighter**: a staff in both hands. It parries better and hits more often.

Shooting and throwing are styles of their own, so a marksman with a sword at their side trains
both Marksman (shots) and Duelist (blows). A kick is no style: you kick whatever your hands hold,
so it trains only [unarmed fighting](weapon-skills.md#the-skills), and Brawler does not help it.

## Two weapons

Your off hand takes a shield, or a **light** weapon: a dagger, a stiletto, or a short sword (see the
[weapons](combat.md#damage) table; "main hand or off hand"). With a weapon in each hand, every
attack you make up close is two blows: the main hand's first, then the off hand's, each at a
penalty to hit. The off hand's blow is made with its own weapon and trains that weapon's skill;
a magic weapon's own powers (see [Magic items](magic-items.md)) go with its own blow, while what
your other gear gives helps both. Each blow can draw blood and open a bleeding wound on its own.
Both blows are one attack, in the time of one. If the first blow kills, there is no second.

Twin Blades eases both penalties, and at rank 10 neither hand has any: a full attack with each.
Creatures that hold two weapons fight with both too.

## What a rank gives

As with weapon skills, the game never tells you what a rank gives in numbers. These are the
rules. A style's benefits add to those of the weapon skill you fight with:

- **To hit**, **Damage**, **Time**, **Parry**, **Shield**, and **Reload time** as for
  [weapon skills](weapon-skills.md#what-a-rank-gives).
- **Critical damage**: a confirmed critical rolls its damage that many more times (×3 instead
  of ×2; see [Criticals](combat.md#criticals)).
- **Main hand** and **Off hand**: the penalty to hit of each hand's blow with two weapons.
- **Shield bash after a hit**: the chance that a blow that hits is followed by a free bash with
  your shield.

{{#include generated/style-benefits.md}}

Rank 0 gives nothing. Rank 10 is mastery: later, masters of a style will learn feats of their
own (a duelist's riposte, a staff fighter's trip), which will be chosen from the one abilities
menu like every other ability.

## Related styles

Styles default to each other as weapon skills do (see
[Related skills](weapon-skills.md#related-skills)), but never to a weapon skill: when you gain a
rank in one, each related style rises to your new rank less the penalty, if that is more than you
have in it.

{{#include generated/style-defaults.md}}

## Where your styles come from

Your [class](classes.md) starts you with a rank in a style or two: a warrior fights with sword
and board, a barbarian with a great weapon, a ranger is a marksman, and a thief has learned
either Twin Blades or Duelist (the dice decide, as do a monk's Brawler or Staff Fighter). The
[weapon skills screen](weapon-skills.md#seeing-your-skills) (`W`) lists your styles below your
weapon skills, and marks the ones you fight in now with a `•`.

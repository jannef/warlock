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

## Caster styles

Spells that roll to hit have styles too: how you cast them. Each trains, as the
[Spell attacks](weapon-skills.md#the-skills) skill does, on every such spell you cast with your
own mana that reaches a creature, and adds to it.

- **Battle caster**: a spell that rolls to hit, cast while you wear armour that makes spells
  dearer (mail, plate, a great helm, a large shield). It comes first: in such armour every ray,
  bolt, and touch is a battle caster's. It hits more often, and it makes your armour's extra
  mana cost smaller for **every** spell you cast, whatever it is. It also trains a little on
  every other spell you cast in such armour. From rank 4 a spell that lands wards you: a point
  of Defence until your next action (*Spell guard*).
- **Sniper**: a ray or a bolt, out of armour. It hits more often, and better still against a
  target five cells away or more, where its critical range is wider too. From rank 4 its best
  shots pin the target where it stands (*Pinning ray*).
- **Toucher**: a touch, out of armour. It hits more often, its critical range is wider, and its
  criticals roll the damage up to twice more. From rank 4 its criticals stagger (*Staggering
  touch*); at rank 10 every touch that lands weakens the foe (*Withering touch*).

The wizard and the druid start in Sniper, the witch and the priest in Toucher, the paladin in
Battle caster; any caster may advance in any of them by casting.

## Two weapons

Your off hand takes a shield, or a **light** weapon (listed below; see the
[weapons](combat.md#damage) table: "main hand or off hand"). With a weapon in each hand, every
attack you make up close is two blows: the main hand's first, then the off hand's, each at a
penalty to hit. The off hand's blow is made with its own weapon and trains that weapon's skill;
a magic weapon's own powers (see [Magic items](magic-items.md)) go with its own blow, while what
your other gear gives helps both. Each blow can draw blood and open a bleeding wound on its own.
Both blows are one attack, in the time of one. If the first blow kills, there is no second.

{{#include generated/light-weapons.md}}

Twin Blades eases both penalties, and from rank 9 neither hand has any: a full attack with each.
From rank 7 it adds to hit as well.
Creatures that hold two weapons fight with both too.

## What a rank gives

As with weapon skills, the game never tells you what a rank gives in numbers. These are the
rules, exactly. A style's benefits add to those of the weapon skill you fight with:

- **To hit**, **Damage**, **Time**, **Parry**, **Shield**, and **Reload time** as for
  [weapon skills](weapon-skills.md#what-a-rank-gives).
- **Critical damage**: a confirmed critical rolls its damage that many more times (×3 instead
  of ×2, ×4 at Great Weapon 10; see [Criticals](combat.md#criticals)).
- **Main hand** and **Off hand**: the penalty to hit of each hand's blow with two weapons.
- **Shield bash after a hit**: the chance that a blow that hits is followed by a free bash with
  your shield.

{{#include generated/style-benefits.md}}

The first five ranks grow slowly; ranks 6 to 10 grow much faster.

Rank 0 gives nothing. Rank 10 is mastery.

## What a rank teaches

A style's rank teaches abilities as a weapon skill's does (see
[What a rank teaches](weapon-skills.md#what-a-rank-teaches)): told with the rank, listed in the
[abilities menu](abilities.md) after the style's name, and firing by themselves, free and
instant, but only while you fight in the style: a duelist's riposte needs one weapon and an empty
hand, Staggering block a weapon and a shield.

{{#include generated/style-abilities.md}}

Two that do the same never add up: a riposte from Duelist and one from a talent are one riposte.
Staggered is never more than one slowed action, however many blows staggered the foe. A new pin
keeps the longer of the two and never adds. Rend comes once an attack, when both blades hit and
the foe lives; a shield bash is no second blade. Light-footed lasts for your next action only.

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

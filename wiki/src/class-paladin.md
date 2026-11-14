# Paladin

{{#include generated/class-card.md:paladin}}

See also: [Zeal](zeal.md).

## How to play

You fight up close with sword and shield, like a [warrior](class-warrior.md), and you hold
[zeal](zeal.md): every blow of yours that hits, and every blow that hits you, builds a point.
Spend it from the [abilities menu](abilities.md#the-abilities-menu) (`a`) on
[Smite](#talent-smite), a harder blow, hardest against the undead and fiends, or on
[Lay on hands](#talent-lay-on-hands), hit points back at once. Your few spells come from level 6.
A deed that moves you toward evil empties your zeal (see [Alignment](alignment.md)).

## Levels

{{#include generated/class-levels.md:paladin}}

## Abilities

The paladin fights from one pool, **[zeal](zeal.md)**, for wrath and mercy
alike. It holds 10, plus your [Presence](stats.md#stat-presence)
[modifier](stats.md#modifiers), plus a point per 10 [levels](experience.md#levels). It comes with
Smite at level 1, holding 5.

You build a point of zeal with every blow, shot, or throw of yours that hits, and with every blow
that hits you. It also comes back a point a minute, resting too (see
[Zeal](zeal.md)). Your [mana](mana.md#how-much-mana-you-have) is for your spells; none of
this spends it.

**A deed that moves you toward evil** empties the pool at once ("Your zeal deserts you."). It fills
again, and bars nothing (see [Alignment](alignment.md)).

**The unholy** are the creatures holy wrath falls hardest on: every type tagged undead, fiend,
demon, devil, or yugoloth. What a creature has done, or which side it is on, plays no part. A
hostile bear takes your smite as well as a ghoul does; only the ghoul takes the unholy die.

**The smites** are each a row of the [abilities menu](abilities.md). Each is a melee blow with what
you wield at the [hostile](creatures.md#factions) in a direction:

- **Cost:** its zeal (the table below), and the time of the attack. A miss still spends its zeal.
- **Effect:** your [Presence](stats.md#stat-presence) modifier is added to hit, and 1d6 more
  [damage](combat.md) per 10 levels (at least 1d6), with one die more against the unholy.
- **Refused:** with nobody hostile there, for no time and no zeal.

| Level | Smite | Zeal | What it does |
|---:|---|---:|---|
| 1 | [Smite](abilities.md#ability-smite) | 3 | The blow, and nothing more |
| 12 | [Branding smite](abilities.md#ability-branding-smite) | 3 | The foe it wounds is [Branded](status-list.md#status-branded) for five minutes: it sheds a faint light wherever it goes, so you can see it in the dark and it lights its fellows |
| 32 | [Wrathful smite](abilities.md#ability-wrathful-smite) | 4 | The foe it wounds makes a [Willpower](stats.md#stat-willpower) [check](diseases.md#checks) against 13 plus your Presence modifier; failing, it is [Frightened](status-list.md#status-frightened) of you for 2d3 turns and flees |
| 50 | [Banishing smite](abilities.md#ability-banishing-smite) | 6 | A foe it wounds that was [summoned](creature-powers.md#summoning) is unmade, whatever its [hit points](hit-points.md#hit-points), your kill as any other; an unholy foe not summoned takes the smite's dice twice over; anyone else takes a plain smite |

A smite's extra effect still spends its zeal when it has nothing to do, and a blow that does no
damage carries no extra effect.

**[Lay on hands](abilities.md#ability-lay-on-hands)** (level 1) spends 6 zeal and a spell's time to
heal 2 hit points a level. **[Avatar](#talent-avatar)** (level 60) makes it heal fully,
cure every status that ticks and every [disease](diseases.md), and cost half: 3 zeal.

**[Retribution](#talent-retribution)** (level 25) works by itself. When a hostile creature
beside you lands a blow of its melee attack on you, your smite answers once the attack is over. It
costs 4 zeal and an [exertion](stances.md#stamina), and no time. The blow has already built its
point, so 3 zeal is enough. With less, or while you are off guard, nothing happens. An attack it
answers is not also [riposted](fighting-styles.md#style-ability-riposte); see
[Combat](stances.md#free-blows).

**[Holy wrath](#talent-holy-wrath)** (level 40) works by itself and costs nothing. Every
blow, shot, or throw of yours that hits the unholy, smites too, rolls 1d6 more of the blow's own
kind. It is [resisted](damage-types.md#armour-and-resistance) but never soaked by Protection. From
[Avatar](#talent-avatar) on, it lands on every creature you hit.

**[Zealot](talents.md#talent-zealot)** is the one talent pick on the pool, from level 6. Every blow
that hits you builds 2 zeal instead of 1, for a paladin who stands in the doorway and takes them.

**Which side you are on.** [Lay on hands](abilities.md#ability-lay-on-hands), the smites, and
[Holy radiance](abilities.md#ability-holy-radiance) work only while you are Kind or better;
[Consecrate](abilities.md#ability-consecrate) while you are Neutral or better (see
[Alignment](alignment.md)). Otherwise you can still use them, but nothing happens and no zeal is
spent. A smite says "You call for holy wrath, but none answers." The abilities that work by
themselves do not care, except Retribution, which answers no blow while you are outside them. Your
zeal builds whichever side you are on.

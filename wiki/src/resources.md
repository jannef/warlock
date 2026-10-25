# Resources

Beside your [hit points](combat.md#hit-points) and your [mana](mana.md), which everyone has, a
character may come to hold **resources** of their own: pools of points that something grants
them, such as a barbarian's rage or a ranger's focus. You may hold several at once, each with its
own rules:

- some **come back** by themselves with time, up to their most;
- some **wear off** with time, down to nothing;
- some **build up** from what you do: a kill, a blow that hits, a critical, being hit, a sneak
  attack;
- abilities may **spend** them: an ability that costs more than you have is greyed out in the
  [abilities menu](abilities.md) ("not enough focus");
- and some **end** when they run out: the resource is gone, and its bar with it, until something
  grants it again. Others stay with you, empty, and fill up again.

While you hold a resource, the status bar shows it after your mana, as `Rage 7` or `Focus 7/10`
(what you have, and for some your most), and the [character screen](character.md) (`@`) gives it
a row of its own. A resource that wears off while you [rest](resting.md) may run out before you
wake; one that does not come back while resting waits for you to get up. Creatures never hold
any.

Nothing converts between resources and mana: each is spent by what asks for it.

## The resources

The barbarian's rage runs on Rage (see [Rage](#rage) below). The ranger's **focus** is in play:
rapid shot (level 12) grants it, full, and you keep it for good. A shot you loose while you
hold 2 focus spends them and takes 10 % less time; a volley (level 50) costs 6. Focus comes
back while you act, but not while you rest.

The other classes' features that are to run on these pools (the thief's openings and finisher,
the monk's flurry and quivering palm, the bard's inspiration) still work as their
[class tables](classes.md) describe; they move onto the resources below in coming updates, and
until then nothing in play grants them. A base turn is 6 seconds.

{{#include generated/resources.md}}

## Rage

A barbarian's **Rage** ability (from the abilities menu, the time of a wait) gives them 12 rage,
and they are **Raging** (see [Status effects](status-effects.md)) for as long as they hold any:
Strength +2 and Toughness +2, Intelligence −2, and they can neither read nor spend mana. Rage
wears off a point a base turn, the turn of the rage itself first, resting too; at none the rage
ends, its bar with it, and the barbarian is Exhausted for ten minutes, with no rage until that
passes. Something that ends the Raging otherwise ends the rage too. The barbarian's later levels
add to it (see [Talents](talents.md)):

- **Blood frenzy** (level 40): every kill while raging builds 5 rage.
- **Mighty rage** (level 50): Raging gives Strength +4 and Toughness +4, rage holds up to 30, and
  the exhaustion lasts five minutes.
- **Unstoppable** (level 60): while raging, bleeding and poison cost no hit points, and a blow
  that would take your last hit point leaves it to you for 10 rage, as often as you have that
  much; spending the last of it ends the rage.

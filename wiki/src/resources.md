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

The classes' features that are to run on these pools (the barbarian's rage, the ranger's rapid
shot and volley, the monk's flurry and quivering palm, the bard's inspiration) still work as
their [class tables](classes.md) describe; they move onto the resources below in coming updates.
A base turn is 6 seconds.

The thief's **openings** are in: a thief of level 32 (Improved sneak attack) holds them, starting
at 0. Every sneak attack that hits gives one, up to 5, and one slips away every 20 base turns
(2 minutes); empty, they stay with you. At level 60 Master thief's **finisher** (an ability,
aimed in a direction) spends all 5 for a melee blow that is a critical whenever it hits: no
confirmation roll is made. A miss still spends them.

{{#include generated/resources.md}}

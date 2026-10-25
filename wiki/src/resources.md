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
shot and volley, the thief's openings and finisher, the monk's flurry and quivering palm, the
bard's inspiration) still work as their [class tables](classes.md) describe; they move onto the
resources below in coming updates, and until then nothing in play grants one. A base turn is 6
seconds.

{{#include generated/resources.md}}

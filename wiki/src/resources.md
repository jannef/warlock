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

The thief's **openings** are in too: a thief of level 32 (Improved sneak attack) holds them,
starting at 0. Every sneak attack that hits gives one, up to 5, and one slips away every 20
base turns (2 minutes); empty, they stay with you. At level 60 Master thief's **finisher** (an
ability, aimed in a direction) spends all 5 for a melee blow that is a critical whenever it
hits: no confirmation roll is made. A miss still spends them.

The monk's **ki** comes with flurry (level 18), full: each second blow of a flurry spends a
point, and with none the attack is one blow; quivering palm (level 50) spends all 5. Ki comes
back a point every 3 base turns (18 seconds), resting too.

The bard's **inspiration** comes with inspire (level 18), starting at 0. Every kill of your own
gives one, up to 5, and one fades every 30 base turns (3 minutes); empty, it stays with you.
Inspire spends 3 inspiration instead of its 3 mana whenever you hold 3, so no mana is spent;
with less it costs mana as before. A virtuoso's inspire spends neither.

A base turn is 6 seconds.

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

## Talents on resources

Four [talents](talents.md) change a resource, and only the class that has it may take them (a
pick from their trees, from the level given):

- **Battle trance** (Might; barbarian, level 6): your rage starts at 18 instead of 12 (never
  above its most), and wears off at two thirds of its pace: a point about every 9 seconds
  instead of every 6.
- **Inner calm** (Finesse; monk, level 18): ki holds up to 7 instead of 5, and comes back half
  again as fast: a point every 12 seconds instead of every 18.
- **Steady hand** (Weaponcraft; ranger, level 12): focus comes back half again as fast, a point
  every 4 seconds instead of every 6; still not while you rest.
- **Patient blade** (Shadow; thief, level 32): openings slip away half as fast, a point every 40
  base turns instead of every 20, and a sneak attack that kills gives two openings instead of
  one.

Inner calm changes nothing until something grants the monk ki.

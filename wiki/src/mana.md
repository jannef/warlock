# Mana

Mana is your second pool beside your [hit points](combat.md#hit-points), and it works the same way:
you have some, it has a most, and it comes back by itself with time. [Spells](magic.md) spend it
(those your class gives, see [Classes](classes.md#levels), and those you learn from scrolls), and
so do [rods](magic.md#rods): each costs its mana, and with too little you cannot cast it until
enough has come back.

The status bar shows it right after your hit points, as `MP 5/12`: what you have left of your
most. The [character screen](character.md) (`@`) shows it beside your hit points. Creatures may
have mana too, but you never see theirs. Magic items work for a creature exactly as for you: one
that wears or wields a "+5 to maximum mana" ring has 5 more mana (and starts with all of it), one
whose amulet makes "mana come back 50% faster" regains it half again as fast, and one whose gear
raises caster level has that higher caster level. A summoner pays for its summons from that larger
pool.

## How much

Your most mana is what your [race](races.md) and your [class](classes.md) give, plus what your
Intelligence, Willpower, and Presence add: high scores add to it, low ones take from it, and it
never goes below nothing. Wizards and witches have the most, and aristocrats oddly much;
every class gives at least 1 (warriors, barbarians, thieves, commoners, and craftspeople no
more), and only elves, gnomes, half-elves, halflings, and humans have any of their race's.

{{#include generated/mana.md}}

Each level up to 60 raises it (and the mana you have now) by half your class's mana plus what the
better of your Intelligence and Willpower adds (see
[Experience](experience.md#what-a-level-gives)): a wizard keeps growing it after hit points stop
at level 20; a warrior with ordinary wits gains none. [Magic items](magic-items.md) that add to it
("+5 to maximum mana") raise your most, not what you have. The talent *Deep well* raises both,
by 1 for every 4 character levels and at least 2 for each of its three ranks, as you take it and
as each level after comes (see [Talents](talents.md)).

## How fast it comes back

At the usual pace, a point of mana comes back every so often, faster the stronger your Willpower:

{{#include generated/mana-regeneration.md}}

Resting, some status effects and diseases, starving, and special rooms change the pace:

{{#include generated/mana-paces.md}}

When several status effects slow it, the slowest counts; a disease that slows it more than being
Diseased does counts too. Resting makes up for some of it, as it does for healing, and so does a
sanctuary (a [special room](special-rooms.md)). Magic items that speed it ("mana comes back 50%
faster") add on top of all that, and so does being born under Water (the gift *Tidal*: mana comes
back 25 % faster; see [Horoscope](horoscope.md)) and the talent *Clear mind* (25 % faster too),
adding their percents to theirs. The talent *Meditation* makes resting bring it back half again
as fast (resting's pace times 1.5). Nothing else changes it: not your burden, not the cold, not
your combat stance.

## Spending it

Whatever spends mana is refused when you have too little, and spends nothing then. Spending any
trains your Willpower (a chance at a training mark each time; see
[Stats](stats.md#training)). The talent *Frugal caster* makes abilities cost a fifth less mana,
rounded to the nearest point and never below 1: 5 becomes 4, 6 becomes 5, 10 becomes 8.
[Magic items](magic-items.md) *of Thrift* make abilities cheaper the same way, by their percent;
everything you wear and the talent are added up before the one rounding, and never take more
than half off together: *Frugal caster* (20 %) and a ring of 10 % take 30 % off, so 7 becomes 5;
three pieces of 20 % and the talent still take only 50 %, so 10 becomes 5. It works on a worn
item you have not identified, and stops when you take it off. This cut comes last, after
everything else that changes a spell's or a rod's cost (see
[What a spell costs](magic.md#what-a-spell-costs)), and it is the one for every ability that
costs mana: spells, rods, *Inspire*, a summoning.

The talent *Blood magic* gives you an ability (in the abilities menu) that turns 5 of your hit
points into 3 mana at once, for the time of a wait. It never takes your last hit point (with 5
or fewer it refuses), never brings your mana above your most, and does nothing while your mana
is full. It is not a wound: nothing soaks it, and it trains nothing.

## When the most falls

- Taking off a magic item that raised it lowers what you have down to your new most, if need be.
- A [disease](diseases.md) that damages your Intelligence, Willpower, or Presence lowers your most,
  and what you have with it.
- A status effect that lowers one of those stats lowers your most only while it lasts: you keep
  the mana you had, but none comes back above the lower most until it passes.

## The fine print

- Mana comes back on game time, whatever you do, like hit points: waiting, walking, fighting, or a
  long journey. Only the time counts, and it does not matter how it is cut up.
- Progress toward the next point carries over from turn to turn, and is lost once you are full.
- You never see how fast it comes back as a number, only your mana rising.

See also [Healing](healing.md) and [Stats](stats.md).

# Altars, forges, and springs

INTRO

## Forgotten altars

An altar room holds a **forgotten altar**: worn stone, its carvings rubbed smooth, its god's name
long lost, and yet something there still listens. It is not a temple and not a god you serve:
nothing to sacrifice, nothing to convert. It is a gamble. Walk into it and say yes when asked, or stand beside it and choose **[Pray](abilities.md#ability-pray)**
from the [abilities menu](abilities.md#the-abilities-menu) (it is listed under *Here* only while you stand next to such an altar).
A prayer takes a minute; anything that would stop a [rest](resting.md#how-resting-works) (a creature coming into view, a blow, a
noise) breaks it, and a broken prayer gives nothing, though the minute is gone.

At the end of a prayer one outcome is drawn. Most prayers do little: a few [coins](shops.md#money), a little
[healing](healing.md#how-fast), a pang of [hunger](hunger.md#how-hungry-you-are), a brief hex, a wandering creature, or nothing at all but a line of
flavour. A few do more, for good or ill, and now and then the altar itself bursts apart in [fire](damage-types.md#damage-fire),
hurting everyone near it, you included (it can kill), and leaving rubble.

Every altar has a hidden side, **[good, neutral, or evil](alignment.md)**. The game never says which, but its
glow does: gold, grey, or red. The side tilts the odds against your own [alignment](alignment.md) (see
[Alignment](alignment.md)): pray at an altar of your own side and the boons come more often,
at one of the other side the banes. A neutral altar, or a neutral you, leans neither way.
Praying also moves your [alignment](alignment.md#what-moves-you) a little: toward good at a good altar, toward evil (more) at an
evil one. The talent *[Pious](talents.md#talent-pious)* (Wayfarer) doubles the lean of an altar of your side and takes
away an opposed one's.

After each completed prayer, there is an even chance the altar is **[spent](terrain.md#terrain-spent-altar)**: its glow gutters and
dies, it stands cold and dark, and nothing answers there again. So an altar usually gives one or
two prayers. Seek out the next one: about one generated floor in twenty has one.

{{#include generated/altar-shares.md}}

Rarely, an altar marks you for good. The great boons raise a [stat](stats.md) for good, give a whole
[level](experience.md#levels) at once, grant a [talent](talents.md) for life (never one of your level's picks), or leave a relic or
an elixir (see [Potions](potions.md#elixirs)) at the altar's foot. The great banes lower a [stat](stats.md) for good, take a [level](experience.md#levels) away (what you chose at
that level you keep, but gaining it again gives no new choices), mark you with a flaw for life,
or call a horde around the altar. Talents an altar gave you are marked "altar" on the Talents
page.

The outcomes, by kind and tier ("Of its kind" is its share of its kind's draws):

{{#include generated/altar-outcomes.md}}

## Forges

A forge is a cold smithy: soot on the walls and an **anvil**, its hammer chained beside it. One more anvil stands in Cēaptūn, in Eadric's Forge beside the smith's fire (free to use; it is not the shop's service). Walk
into the anvil and say yes when asked, or stand beside it and choose **[Temper](abilities.md#ability-temper)** from the [abilities menu](abilities.md#the-abilities-menu) (listed under *Here* only there),
then pick something of metal you carry: a weapon, a missile weapon, armour, or a shield whose main
part is a metal, as its [description](objects.md#what-an-object-is) says (so no leather, wood, or bone, and never
[ammunition](ammunition.md), rings, or amulets). An attempt is ten minutes of work; anything that would stop a
[rest](resting.md#how-resting-works) stops it, and then the item is untouched and the attempt does not count, but the time
is gone. Anyone may temper; *[Metalcraft](skills.md#skill-metalcraft)* only makes it go better. Every finished attempt may train
[Metalcraft](skills.md#skill-metalcraft) and [Strength](stats.md#stat-strength), whatever came of it.

It is one action, and what it does depends on the item:

| The item | Made | Failed |
|---|---|---|
| Not damaged | It takes one **improvement** | It is **damaged**: all its improvements are gone |
| Damaged | The damage is gone: a plain item again | It is **destroyed**, even from your hand, with its [affixes](magic-items.md) and however rare it is |
| Three improvements | It can be made no finer, and the anvil will not take it | |

**Every attempt is harder than the last, made or failed, for the item's whole life**, repairs too,
so an item worked long enough will only ever break. Nothing tells you how many it has had.

{{#include generated/temper-odds.md}}

An improvement goes to something drawn at random, evenly, among what the item has, and the same may
come twice. The item's name says how far it has come, whatever you know of its magic:

{{#include generated/temper-grades.md}}

A damaged item is a point worse than a plain one until it is mended: its [Protection](combat.md#protection)
if it gives some, else its [Defence](combat.md#defence), else its [damage](combat.md#damage), so stopping there is a real choice.
[Acid](materials.md#corrosion) damages an item the same way. The anvil neither adds nor takes [affixes](magic-items.md); they stay through improvement, damage,
and repair. Whatever survives is [identified](magic-items.md#unidentified-items). A worked item is like no other and never stacks;
of a stack, one is worked at a time. The anvil never wears out.

## Healing springs

A spring room holds a **healing spring**: clear water welling up in an old stone basin. Walk
into it and say yes when asked, or stand beside it and choose **[Drink](abilities.md#ability-drink)** from the [abilities menu](abilities.md#the-abilities-menu) (listed under *Here* only while you stand
next to a spring). A drink takes ten seconds; anything that would stop a [rest](resting.md#how-resting-works) breaks it off, and
a broken drink gives nothing. Most drinks are good, some taste only of stone, and now and then
the water is foul and gives you [marsh ague](diseases.md#disease-marsh-ague), which a prayer (a temple's [cure](npcs.md#services)) or a remedy cures. A
spring never runs dry: drink as often as you like, and take the risk each time.

{{#include generated/spring-outcomes.md}}

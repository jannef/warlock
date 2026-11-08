# Altars, forges, and springs

Three kinds of [special room](special-rooms.md) hold something you can use. A forgotten altar
answers prayers, for good or ill. A forge's anvil lets you temper metal arms and armour. A healing
spring can be drunk from as often as you like. Each is used by walking into it, or from the *Here*
group of the [abilities menu](abilities.md#the-abilities-menu) while you stand beside it.

## Forgotten altars

An altar room holds a **forgotten altar**: worn stone, its carvings rubbed smooth, its god's name
long lost. Yet something there still listens. It is not a temple, and not a god you serve. There
is nothing to sacrifice and no one to convert to. It is a gamble.

To pray, walk into the altar and say yes when asked, or stand beside it and choose
[Pray](abilities.md#ability-pray) from the [abilities menu](abilities.md#the-abilities-menu). It is
listed under *Here* only while you stand next to an altar. A prayer takes a minute. Anything that
would stop a [rest](resting.md#how-resting-works), such as a creature coming into view, a blow, or a
noise, breaks it. A broken prayer gives nothing, and the minute is still gone.

At the end of a prayer one outcome is drawn. Most prayers do little: a few
[coins](shops.md#money), a little [healing](healing.md#how-fast), a pang of
[hunger](hunger.md#how-hungry-you-are), a brief hex, a wandering creature, or nothing at all but a
line of flavour. A few do more, for good or ill. Now and then the altar bursts apart in
[fire](damage-types.md#damage-fire) and leaves rubble. The blast hurts everyone near it, you
included, and it can kill.

Every altar has a hidden side: [good, neutral, or evil](alignment.md). The game never says which,
but its glow does: gold, grey, or red. The side tilts the odds against your own
[alignment](alignment.md). At an altar of your own side the boons come more often; at one of the
other side, the banes. A neutral altar, or a neutral you, leans neither way. Praying also moves
your [alignment](alignment.md#what-moves-you) a little: toward good at a good altar, and further
toward evil at an evil one. The [Pious](talents.md#talent-pious) talent (Wayfarer) doubles the lean
of an altar of your side and takes away an opposed one's.

After each completed prayer there is an even chance the altar is
**[spent](terrain.md#terrain-spent-altar)**. Its glow gutters and dies, it stands cold and dark,
and nothing answers there again. So an altar usually gives one or two prayers. Then seek out the
next one: about one generated floor in twenty has one.

{{#include generated/altar-shares.md}}

Rarely, an altar marks you for good. The great boons:

- raise a [stat](stats.md) for good;
- give a whole [level](experience.md#levels) at once;
- grant a [talent](talents.md) for life (never one of your level's picks);
- leave a relic or an elixir (see [Potions](potions.md#elixirs)) at the altar's foot.

The great banes:

- lower a [stat](stats.md) for good;
- take a [level](experience.md#levels) away. You keep what you chose at that level, but gaining it
  again gives no new choices;
- mark you with a flaw for life;
- call a horde around the altar.

Talents an altar gave you are marked "altar" on the Talents page.

The outcomes, by kind and tier ("Of its kind" is its share of its kind's draws):

{{#include generated/altar-outcomes.md}}

## Forges

A forge is a cold smithy: soot on the walls and an **anvil**, its hammer chained beside it. One
more anvil stands in Cēaptūn, in Eadric's Smithy beside the smith's fire. It is free to use; it is
not the shop's service.

To temper, walk into the anvil and say yes when asked, or stand beside it and choose
[Temper](abilities.md#ability-temper) from the [abilities menu](abilities.md#the-abilities-menu)
(listed under *Here* only there). Then pick something of metal you carry: a weapon, a missile
weapon, armour, or a shield whose main part is metal, as its
[description](objects.md#what-an-object-is) says. Leather, wood, and bone will not do, and neither
will [ammunition](ammunition.md), rings, or amulets.

An attempt is ten minutes of work. Anything that would stop a
[rest](resting.md#how-resting-works) stops it. The item is then untouched and the attempt does not
count, but the time is gone. Anyone may temper; [Metalcraft](skills.md#skill-metalcraft) only makes
it go better. Every finished attempt may train [Metalcraft](skills.md#skill-metalcraft) and
[Strength](stats.md#stat-strength), whatever came of it.

What an attempt does depends on the item:

| The item | Made | Failed |
|---|---|---|
| Not damaged | It takes one **improvement** | It is **damaged**: all its improvements are gone |
| Damaged | The damage is gone: a plain item again | It is **destroyed**, even from your hand, with its [affixes](magic-items.md) and however rare it is |
| Three improvements | It can be made no finer, and the anvil will not take it | |

**Every attempt is harder than the last, for the item's whole life**, whether it was made or
failed, and repairs count too. An item worked long enough will only ever break. Nothing tells you
how many attempts it has had.

{{#include generated/temper-odds.md}}

An improvement goes to one of the item's numbers, drawn evenly at random, and the same one may come
twice. The item's name says how far it has come, whatever you know of its magic:

{{#include generated/temper-grades.md}}

A damaged item is a point worse than a plain one until it is mended. It loses a point of
[Protection](combat.md#protection) if it gives some, else of [Defence](combat.md#defence), else of
[damage](combat.md#damage). So stopping while you are ahead is a real choice.
[Acid, rust, and heat](materials.md#item-damage) damage an item the same way (it is then called
*corroded*, *rusted*, or *scorched* instead of *damaged*), and the anvil mends it all the same.
Damaged again by anything, it is destroyed. No material withstands a failed attempt.

The anvil neither adds nor takes [affixes](magic-items.md); they stay through improvement, damage,
and repair. Whatever survives is [identified](magic-items.md#unidentified-items). A worked item is
like no other and never stacks; from a stack, one is worked at a time. The anvil never wears out.

## Healing springs

A spring room holds a **healing spring**: clear water welling up in an old stone basin. To drink,
walk into it and say yes when asked, or stand beside it and choose
[Drink](abilities.md#ability-drink) from the [abilities menu](abilities.md#the-abilities-menu)
(listed under *Here* only while you stand next to a spring).

A drink takes ten seconds. Anything that would stop a [rest](resting.md#how-resting-works) breaks it
off, and a broken drink gives nothing. Most drinks are good, and some taste only of stone. Now and
then the water is foul and gives you [marsh ague](diseases.md#disease-marsh-ague). A prayer (a
temple's [cure](npcs.md#services)) or a remedy cures it. A spring never runs dry: drink as often as
you like, and take the risk each time.

{{#include generated/spring-outcomes.md}}

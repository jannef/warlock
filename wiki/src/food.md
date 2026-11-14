# Food and corpses

This page has what you eat: eating, food that spoils, and the corpse tables. How hunger works
and what it does to you is on [Hunger](hunger.md).

Food (`%`) keeps [hunger](hunger.md#how-hungry-you-are) away. Press `e` (the Eat command) to
consume something. A list titled "Consume what?" shows what you can eat: first what lies where
you stand ("lying here"), then the food you carry (see [Lists of
objects](objects.md#lists-of-objects)). Press a letter, or `Enter` on the marked row, to eat it.
Your potions are in the same list and are drunk the same way (see [Potions](potions.md)), and so
are your [magic crystals](crystals.md#consuming-a-crystal).

## Eating

A meal takes a few turns, each a normal action (6 seconds at a normal pace; see [Turns and
movement](turns.md)). Like a [rest](resting.md#how-resting-works), it stops early when a
[hostile](creatures.md#factions) creature comes close in view or something happens that you are told about: "A
kobold comes into view; you stop eating." Then nothing is eaten and the food is left as it was.
The food is eaten, and feeds you, only when the last turn is done: "You eat the food ration.
You feel satiated."

{{#include generated/food.md}}

Eating when you are already full is risky. A meal that would fill you past what your stomach
can hold may come back up ("You vomit!"), leaving you [hungry](hunger.md#how-hungry-you-are) and
fed nothing. If it stays down, whatever does not fit is lost. See
[Hunger](hunger.md#how-hungry-you-are).

Ælfric's General Store in Cēaptūn ([The Ascmearc](ascmearc.md)) [sells](shops.md#buying) food
for the road. Your [starting kit](starting-kit.md) may include some food too.

## Food spoils

All food **rots**, whether it lies on the floor or in your [pack](objects.md#your-pack). It
keeps for a while (the table above says how long), and it is old once half of that time has
passed. Some time after it goes rotten, it rots away altogether. Food in your pack that rots
away is gone, and you are told: "Your loaf of bread has rotted away." Food on the floor is
simply gone when you next look.

Food does not show how old it is, unless you are trained in [Appraise](skills.md#skill-appraise)
or [Survival](skills.md#skill-survival). Then, now and then, the eat list and a look say
*fresh*, *old*, or *rotten*. Food of one kind always keeps together in one
[stack](objects.md#piles), whether you pick it up or put it down, and the stack is as old as
its oldest part. So remember where your food came from and when.

Food a little past its best is fine to eat. Food past its keeping is **rotten**, and eating it
is a bad idea: "Ugh! The loaf of bread was rotten." You may vomit. If it stays down, it makes
you sick ("You feel sick."). That is food poisoning, which costs [hit
points](hit-points.md#hit-points) for a while until it wears off (see [Status
effects](status-effects.md#poisoned)), though the food still feeds you. Rotten food may also
give you [grave rot](diseases.md#disease-grave-rot) (see [Diseases](diseases.md)).

[Ratfolk](races.md#race-ratfolk) eat rotten food without harm: "The loaf of bread was rotten,
but your stomach does not mind." It feeds them as fresh food would, and no more: a rotten
corpse has lost whatever else its flesh did. [Shopkeepers](shops.md) know rotten food when they
smell it. They will not buy it, and they do not sell it either (see [Shops](shops.md)).

## Corpses

A [creature](creatures.md) that dies leaves its **corpse** where it fell, on top of whatever it
dropped: a "kobold corpse", shown as `%` in the creature's color. A corpse weighs what the
creature did, so a [deer](creatures.md#creature-deer)'s is far too heavy to carry (see
[Burden](burden.md)). It feeds you according to the creature's size and takes a while to eat.

{{#include generated/corpses.md}}

Corpses rot like all food, only faster. A fresh corpse turns rotten some hours after the death
and rots away altogether in a day or two, on the floor and in your pack alike ("Your giant rat
corpse has rotted away."). A corpse does not show how old it is, so remember when it fell.
Eating a rotten one is as bad an idea as eating any rotten food: "Ugh! The kobold corpse was
rotten."

A corpse is good for more than a meal. [Necromancy](spellcasting.md#spell-types) has three uses
for a corpse of any age, and each one uses it up. [Charnel burst](magic.md#spell-charnel-burst)
blows it apart, [Grave feast](magic.md#spell-grave-feast) draws out what life is left, and
[Rouse the dead](allies.md#the-risen) stands it up again.

### What a corpse gives you

Eating a fresh corpse may do more than feed you. Some taste terrible and may come back up. Some
make you [sick](status-effects.md#poisoned). Some do you good, and many feed your body's
[training](stats.md#raising-your-stats): a corpse gives **marks** toward a stat, just as using the stat
does. A few corpses (giants', trolls', hags', ogres', and the fey's) have a small chance to
widen how far that stat can grow, its [potential](stats.md#potentials).

A corpse never raises a stat at once. The marks wait for the next check. Many marks make
themselves felt ("Something of its strength passes into you."), and a few are quiet. A widened
potential says "You feel you could grow stronger yet." What a corpse gives depends on what the
creature is (a giant, a troll, a caster, and so on), and some creatures give something of
their own.

The corpses of literate casters (the tribes' mages, shamans, priests, and hags) teach [Decipher
script](skills.md#skill-decipher-script). The first one teaches the skill if you lack it
("Strange letters swim before your eyes, and settle into sense."). Each one after that adds to
it ("The letters come clearer."), but never past a value set by the creature, and deeper
casters set it higher. Past that value, the corpse teaches nothing.

In the game you find all this out by eating. Here it is in the order it happens, each chance
rolled on its own. A rotten corpse has lost all of it.

{{#include generated/corpse-effects.md}}

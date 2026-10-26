# Food and corpses

Food (`%`) keeps [hunger](hunger.md) at bay. Press `e` to eat: a list shows what you can eat,
first what lies where you stand ("lying here"), then the food you carry (see [Lists of
objects](objects.md#lists-of-objects)). A letter, or `Enter` on the marked row, eats it.

## Eating

A meal takes a few turns, each a normal action (6 seconds at a normal pace; see
[Turns and movement](turns.md)), and like a rest it stops early when a creature comes into view
or something happens you are told of: "A kobold comes into view; you stop eating." Nothing is
eaten then, and the food is left as it was. Only when the last turn is done is it eaten and
does it feed you: "You eat the food ration. You feel satiated."

{{#include generated/food.md}}

Eating when you are already full is risky: a meal that would fill you past the most your stomach
holds may come back up ("You vomit!"), leaving you hungry and fed nothing; if it stays down,
what does not fit is lost. See [Hunger](hunger.md).

Cyneburg's Provisions in Cēaptūn ([The Ascmearc](ascmearc.md)) sells food for the road. Your
starting kit may include some food too.

## Food spoils

All food **rots**, whether it lies on the floor or in your pack: it keeps for a while (the
table above says how long; it is old past half of that), and some time after that it has rotted
away altogether. Food in your pack that rots away
is gone, and you are told ("Your loaf of bread has rotted away."); what lies on the floor is
simply gone when you next look. Food never shows how old it is, and picked up or put down, food
of a kind keeps together in one stack, as old as the oldest of it: remember where it came from
and when.

Food a little past its best is fine to eat. Food past its keeping is **rotten**, and eating it is
a bad idea: "Ugh! The loaf of bread was rotten." You may vomit, and if it stays down it makes
you sick ("You feel sick."): food poisoning, which costs hit points for a while until it wears
off (see [Status effects](status-effects.md#poisoned)), though it still feeds you. Rotten food
may also give you grave rot (see [Diseases](diseases.md)). Ratfolk eat it without harm: "The
loaf of bread was rotten, but your stomach does not mind." It feeds them as fresh food would, no
more (a rotten corpse has lost whatever else its flesh did). Shopkeepers know rotten food when they
smell it: they will not buy it, and they do not sell it either (see [Shops](shops.md)).

## Corpses

A creature that dies leaves its **corpse** where it fell, on top of what it dropped: a "kobold
corpse", `%` in the creature's color. A corpse weighs what the creature did, so a deer's is far
too heavy to carry (see [Burden](burden.md)); it feeds you by the creature's size and takes a
while to eat.

{{#include generated/corpses.md}}

Corpses rot like all food, only faster: a fresh corpse turns rotten some hours after the death,
and rots away altogether in a day or two, from the floor and from your pack alike ("Your giant
rat corpse has rotted away."). A corpse does not show how old it is: remember when it fell.
Eating a rotten one is as bad an idea as any rotten food: "Ugh! The kobold corpse was rotten."

Eating a fresh corpse may do more than feed you. Some taste terrible and may come back up, some
make you sick, some do you good, and a rare few may make you a little stronger in some way, as
far as you can grow ([potential](stats.md#potentials)). In the game you find out by eating it;
here it is, in the order it happens, each chance rolled on its own. A rotten corpse has lost all
of that.

{{#include generated/corpse-effects.md}}

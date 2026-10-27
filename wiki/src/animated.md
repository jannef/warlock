# Animated objects

Things a will set moving and left behind: a candlestick that rocks after you, a teapot that
scalds, a broom that scuttles, a rug that wraps around your legs. They set on the living who
come near and leave everything else alone. The household ones are almost harmless, and worth
killing for what they are made of: a silver candlestick is a prize at any level.

## What they are made of

An animated household thing *is* a real household object: an animated candlestick is a
candlestick, of brass, iron, pewter, or (rarely) silver. Its colour on the map is its
material's, and [look mode](controls.md#look-mode) names it: "an animated candlestick of
silver (hostile, unhurt), rocking". A clatter of cutlery is three to six pieces of cutlery at
once.

When one is destroyed, what it was made of falls **whole** half the time, as the object it was
(a silver candlestick you can sell); otherwise nothing of it is left. The **Salvager** talent
raises that share. They leave no corpse and never bleed.

## What they are

They are constructs: nothing lives in them to sicken, they have no blood or breath for poison,
and nothing in them bleeds, so they are never **diseased**, **poisoned**, or **bleeding**. They
can still be frightened (rarely: their will is not their own), held, staggered, or transfixed.
What they are made of decides what hurts them most: wood and cloth burn, metal takes lightning
badly, and glass, stone, and bone shatter under heavy blows. Nothing can be stolen from them:
what they carry is what they are.

The **Unbinder** talent strikes at the binding, not the thing: your blows against animated
things are surer and harder.

## Where they are found

The barrow of Cealdhlāw keeps candlesticks, cutlery, teapots, and cooking pots among its grave
goods; the drowned minster of Fennmynster its candlesticks, books, lanterns, and rugs. Now and
then one strays elsewhere.

## In numbers

Every animated type is generated from its tier and its body by the formulas below (rounded down
part by part): Strength 11 + 0.4 × level, Toughness 12 + 0.4 × level, Dexterity 10 + 0.25 ×
level, Willpower 16 + 0.35 × level, Intelligence and Presence 2 (they are mindless), then the
tier's adjustments; hit points beyond Toughness 2.5 × level × the tier's factor; damage dice the
nearest of 1d2, 1d3, … 5d12 to 2 + 0.55 × level, moved by the tier's steps.

{{#include generated/animated-tiers.md}}

### By level

{{#include generated/animated-numbers.md}}

### The household things

Speed 1000 is a normal pace. A thing's Protection, resistances, and death line come from the
material its object is first made of; one drawn of another material (a silver candlestick of a
brass kind) changes its colour and worth, never its numbers.

{{#include generated/animated-bodies.md}}

### By material

{{#include generated/animated-groups.md}}

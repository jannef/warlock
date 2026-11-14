# Status effects

Some harm lasts longer than a blow. A deep wound keeps bleeding, and a bite can leave poison
behind. These **status effects** cost [hit points](hit-points.md#hit-points) as time passes, slow
your [healing](healing.md), sometimes weaken you, and wear off on their own. They can kill.
What ends bleeding, poison, and disease sooner is gathered on
[Healing](healing.md#curing-bleeding-poison-and-disease).

**[Diseased](status-list.md#status-diseased)** is a status effect too. It shows while any [disease](diseases.md)
has broken out, and lasts as long as the diseases do.

Not every status effect harms. A [rage](rage.md) makes you stronger for a while, and some
effects leave you spent once they are over. What helpful effects, hexes, and cooldowns can do is
on [Buffs, hexes, and cooldowns](buffs.md).

You always know what ails you. The [status bar](the-screen.md#the-screen) names it right after your
hit points (`HP 12/17 · Bleeding · Poisoned`), the [character screen](character.md) lists it, and
the message line tells you when it starts and stops. What you never see is a number for how bad it
is or how long it will last: you just watch your hit points go.

Being grabbed, engulfed, or swallowed has a page of its own: [Held](held.md).

Every status effect is in the tables on [Every status effect](status-list.md), with its exact
paces and the creatures that cause it.

## How they work

- **Severity.** Every status effect has a severity, counted in degrees. The worse it is, the faster
  it costs hit points: a wound twice as bad bleeds twice as fast.
- **They add up.** A second wound while you are still [bleeding](#bleeding) makes the bleeding
  worse. A second dose of [poison](#poisoned) makes you sicker, and stronger poison lasts longer.
  Each status has a worst it can get.
- **They wear off** bit by bit, a degree at a time, until they are gone.
- **Time is time.** Status effects run on game time, whatever you do meanwhile. Waiting, walking,
  [resting](resting.md), and a long journey on the [overworld](overworld.md) all count the same.
  Only creatures on other levels are paused: their effects wait for you to come back.
- **They can kill.** If a status effect takes your last hit point, you die of it, and the death
  screen says so: "Bled to death".

## Stunned, frightened, drowsy, surprised

These four come up in most fights. None of them costs hit points; each costs you time or aim.

- **Stunned.** A heavy blow rings your head. Your next action takes half again as long, and until
  then you are [flat-footed](surprise.md#caught-unawares): your Dexterity and Dodge do not help
  your [Defence](combat.md#defence), you make no free blow, and a
  [sneak attack](surprise.md#sneak-attacks)'s extra dice land. A creature's blow or an ascetic's
  [Stunning strike](class-ascetic.md#talent-stunning-strike) can do it (see [Surprise](surprise.md#stunned)).
- **Frightened.** You hit less often (−3 [to hit](combat.md#hitting)). A frightened creature runs
  from you while it can, and fights only when cornered. Fear wears off a degree every 6 seconds.
- **Drowsy.** Everything you do takes three times as long. The bard's
  [lullaby](abilities.md#ability-lullaby) and some creatures cause it. It eases a degree every 12
  seconds.
- **Surprised.** You were caught unawares and failed the initiative
  check. You are flat-footed, and your next action takes half again as long.

**Getting rid of them:** they wear off by themselves, most within a few turns. Sooner:

- a [potion of clear head](potions.md#potion-of-clear-head) ends stunned, frightened, and drowsy
  at once (and staggered, transfixed, and sluggish);
- a [priest](class-priest.md)'s [prayer of release](prayer.md#prayer-release) ends frightened and
  drowsy;
- an [ascetic](class-ascetic.md)'s [Stillness](abilities.md#ability-stillness) ends all four.

## Bleeding

Two things open a wound ("You are bleeding."):

- a **[big hit](combat.md#damage)**, one that takes at least a third of your hit points at once;
- a **[confirmed critical](combat.md#criticals)** ("The kobold stabs you hard!"). A critical opens
  a worse wound than a big hit.

Only a blow that **cuts or stabs** does it. A club, a fall, or a [fireball](magic.md#spell-fireball)
does not (see [Damage types](damage-types.md)).

While you bleed, you lose a hit point every so often, faster the worse you bleed, and your hit
points do not come back at all. The bleeding eases by itself and stops after a while: "The
bleeding stops." If you are trained in **[Heal](skills.md#skill-heal)**, you can bind the wound and
stop it sooner, trying as often as it takes (see
[Treating yourself](skill-uses.md#treating-yourself)).

**Getting rid of it:** it stops by itself in time. Sooner:

- bind the wound with the Heal skill;
- drink a [potion of greater healing](potions.md#potion-of-greater-healing) or of
  [full healing](potions.md#potion-of-full-healing);
- a [priest](class-priest.md)'s prayer for [wounds](prayer.md#prayer-wounds);
- [Horn's grace](abilities.md#ability-horns-grace), if you were born in the year of the Unicorn;
- now and then, an answer at a [forgotten altar](altars-forges-springs.md#forgotten-altars).

Creatures bleed the same way, and you can see it: [look mode](controls-prompts.md#look-mode) says "A giant
rat (hostile, wounded, bleeding)". A creature can bleed to death after a fight: "The giant rat
bleeds to death."

A creature without blood never bleeds, whatever wounds it: neither a blow nor
[Flay](magic.md#spell-flay) opens it. A [skeleton](creatures.md#creature-skeleton), a ghost, or an
elemental has none; the [creature tables](creature-numbers.md#hit-points-defence-and-resistances)
say which.

## Burning

A [fire spell](spell-types.md) sometimes leaves what it harms burning: "You are
burning!" Burning costs a hit point every ten seconds, faster the more degrees of it you have, and
each degree burns out in half a minute: "The flames go out." What resists
[fire](damage-types.md#damage-fire) takes less harm, and what is immune never burns.

Anything burning gives off a small light, so it cannot hide in the dark, and look mode names it:
"A kobold (hostile, burning)".

**Getting rid of it:** it burns out in half a minute a degree. A step into
[water](terrain.md#water), or a fall through ice into it, puts the flames out at once. So does a
cold spell that harms you.

## Corroded

Every [acid spell](spell-types.md) that harms something leaves it Corroded: "The acid
keeps eating." A degree costs a hit point every six seconds and is spent in twelve, so it does two
more hit points per cast: "The acid is spent." What resists [acid](damage-types.md#damage-acid)
takes less. Look mode names it. It never eats armour.

**Getting rid of it:** nothing ends it sooner; it is spent in twelve seconds a degree.

## Poisoned

Some creatures' attacks carry poison. Now and then a
[barrow spider](creatures.md#creature-barrow-spider)'s bite leaves you sick: "You feel sick." So does bad food: [rotten food](food.md#food-spoils)
(a [corpse](food.md#corpses) too) that stays down, or the flesh of some creatures (see
[Food and corpses](food.md)).

Poison works slowly. It costs a hit point every couple of minutes, faster the stronger it is, and
your hit points come back only half as fast.

**Strong poison** (a few doses at once) also makes you weaker and clumsier. Your
[Strength](stats.md#stat-strength) and [Dexterity](stats.md#stat-dexterity) count one less in
everything they do, from [hitting](combat.md#hitting) and [Defence](combat.md#defence) to how much
you can [carry](burden.md#how-much-you-can-carry). The [character screen](the-screen.md#character)
shows them lowered.

**Getting rid of it:** poison wears off by itself: "The poison wears off." Sooner:

- drink a [potion of antidote](potions.md#potion-of-antidote) or of
  [full healing](potions.md#potion-of-full-healing);
- cast [Remove disease](magic.md#spell-remove-disease);
- a [priest](class-priest.md)'s prayer against [poison](prayer.md#prayer-poison);
- [Horn's grace](abilities.md#ability-horns-grace), if you were born in the year of the Unicorn;
- if you are trained in [Heal](skills.md#skill-heal), draw out some of a dose (see
  [Treating yourself](skill-uses.md#treating-yourself));
- now and then, an answer at a [forgotten altar](altars-forges-springs.md#forgotten-altars).

You cannot tell whether a creature is poisoned by looking at it.

## Ebbing

A [draining room](special-rooms.md#room-draining) drinks at magic. While you stand in one, and for
half a minute after you leave, you are **Ebbing** ("Your power ebbs away."). A point of
[mana](mana.md#how-much-mana-you-have) drains away every 20 seconds, and none comes back. It never takes you
below zero and does you no other harm. Once you are out of the room it passes: "Your power stops
ebbing."

Creatures with mana ebb in the room just as you do, and you can see it on them ("ebbing").
Creatures without mana are left alone. The talent *[Old delver](talents-exploring.md#talent-old-delver)*
keeps it from ever taking you.

## Asleep

The creatures of a [teeming den](special-rooms.md#room-teeming) lie **asleep** ("asleep" in look
mode). A sleeper does nothing at all and has seen no one, so a
[sneak attack](class-thief.md#talent-sneak-attack) on it always lands. Time never wakes it. A loud
enough noise does, as does any harm or a waking neighbour's cry.

## Silenced

A [silent vault](special-rooms.md#room-silent) swallows every sound. While you stand in one, and
for ten seconds after you leave, you are **Silenced** ("The silence closes around you."). You
cannot:

- cast a [spell](magic.md#the-spells), or spend [mana](mana.md#spending-it) on anything;
- read a [scroll](scrolls.md);
- sing a song ([inspire](abilities.md#ability-inspire), the [lullaby](abilities.md#ability-lullaby));
- [throw your voice](hearing.md#throwing-your-voice).

Trying takes no time. Out of the vault, sound comes back: "Sound returns."

Creatures inside are silenced just as you are, and you can see it on them ("silenced"). A caster
that cannot cast closes in to fight hand to hand.

## Unwelcome

A [shrine](special-rooms.md#room-shrine) of the other side from yours does not want you. While you
stand in one, and for half a minute after you leave, you are **Unwelcome** ("You feel
unwelcome."). Your [wounds heal](healing.md#how-fast-you-heal) at half the pace, and you are a little less
likely to hit (−1 [to hit](combat.md#hitting)). Once it passes: "You no longer feel unwelcome."

Creatures of the other side are unwelcome there too, and you can see it on them ("unwelcome").

## In the message line

"You are bleeding." and "You feel sick." stop a [rest](resting.md), like any news. "The bleeding
stops." and "The poison wears off." show in the message line but let you rest on.

See also: [Every status effect](status-list.md), [Buffs and hexes](buffs.md), and
[Healing](healing.md).

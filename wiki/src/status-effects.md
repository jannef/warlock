# Status effects

Some harm lasts longer than a blow. A deep wound keeps bleeding, and a bite can leave poison
behind. These **status effects** cost [hit points](hit-points.md#hit-points) as time passes, slow
your [healing](healing.md), sometimes weaken you, and wear off on their own. They can kill.
What ends bleeding, poison, and disease sooner is gathered on
[Healing](healing.md#curing-bleeding-poison-and-disease).

**[Diseased](#status-diseased)** is a status effect too. It shows while any [disease](diseases.md)
has broken out, and lasts as long as the diseases do.

Not every status effect harms. A [rage](resources.md#rage) makes you stronger for a while, and some
effects leave you spent once they are over (see [Buffs and cooldowns](#buffs-and-cooldowns)).

You always know what ails you. The [status bar](the-screen.md#the-screen) names it right after your
hit points (`HP 12/17 · Bleeding · Poisoned`), the [character screen](character.md) lists it, and
the message line tells you when it starts and stops. What you never see is a number for how bad it
is or how long it will last: you just watch your hit points go.

Being grabbed, engulfed, or swallowed has a page of its own: [Held](held.md).

Every status effect is in the [tables at the end of this page](#every-status-effect), with its
exact paces.

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
  [Stunning strike](classes.md#talent-stunning-strike) can do it (see [Surprise](surprise.md#stunned)).
- **Frightened.** You hit less often (−3 [to hit](combat.md#hitting)). A frightened creature runs
  from you while it can, and fights only when cornered. Fear wears off a degree every 6 seconds.
- **Drowsy.** Everything you do takes three times as long. The bard's
  [lullaby](abilities.md#ability-lullaby) and some creatures cause it. It eases a degree every 12
  seconds.
- **Surprised.** You were [caught unawares](surprise.md#caught-unawares) and failed the initiative
  check. You are flat-footed, and your next action takes half again as long.

A [potion of clear head](potions.md#potion-of-clear-head) cures stunned, frightened, and drowsy at
once.

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

Creatures bleed the same way, and you can see it: [look mode](controls.md#look-mode) says "A giant
rat (hostile, wounded, bleeding)". A creature can bleed to death after a fight: "The giant rat
bleeds to death."

A creature without blood never bleeds, whatever wounds it: neither a blow nor
[Flay](magic.md#spell-flay) opens it. A [skeleton](creatures.md#creature-skeleton), a ghost, or an
elemental has none; the [creature tables](creature-numbers.md#hit-points-defence-and-resistances)
say which.

## Burning

A [fire spell](spellcasting.md#spell-types) sometimes leaves what it harms burning: "You are
burning!" Burning costs a hit point every ten seconds, faster the more degrees of it you have, and
each degree burns out in half a minute: "The flames go out." What resists
[fire](damage-types.md#damage-fire) takes less harm, and what is immune never burns.

Anything burning gives off a small light, so it cannot hide in the dark, and look mode names it:
"A kobold (hostile, burning)".

A step into [water](terrain.md#water), or a fall through ice into it, puts the flames out at once.
So does a [cold spell](spellcasting.md#spell-types) that harms you.

## Corroded

Every [acid spell](spellcasting.md#spell-types) that harms something leaves it Corroded: "The acid
keeps eating." A degree costs a hit point every six seconds and is spent in twelve, so it does two
more hit points per cast: "The acid is spent." What resists [acid](damage-types.md#damage-acid)
takes less. Look mode names it. It never eats armour.

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

Poison wears off by itself: "The poison wears off." If you are trained in
**[Heal](skills.md#skill-heal)**, you may try to draw out some of a dose (see
[Treating yourself](skill-uses.md#treating-yourself)).

You cannot tell whether a creature is poisoned by looking at it.

## Ebbing

A [draining room](special-rooms.md#room-draining) drinks at magic. While you stand in one, and for
half a minute after you leave, you are **Ebbing** ("Your power ebbs away."). A point of
[mana](mana.md#how-much-mana-you-have) drains away every 20 seconds, and none comes back. It never takes you
below zero and does you no other harm. Once you are out of the room it passes: "Your power stops
ebbing."

Creatures with mana ebb in the room just as you do, and you can see it on them ("ebbing").
Creatures without mana are left alone. The talent *[Old delver](talents.md#talent-old-delver)*
keeps it from ever taking you.

## Asleep

The creatures of a [teeming den](special-rooms.md#room-teeming) lie **asleep** ("asleep" in look
mode). A sleeper does nothing at all and has seen no one, so a
[sneak attack](classes.md#talent-sneak-attack) on it always lands. Time never wakes it. A loud
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

## Buffs and cooldowns

A status effect can help or hinder in other ways while it lasts. The "What it does" column of
[the tables](#every-status-effect) says which of these each one does:

- **Stats.** It may raise a score as well as lower one. If a higher
  [Toughness](stats.md#stat-toughness) let you [heal](healing.md#how-fast-you-heal) to more hit points,
  those extra hit points go again when it ends.
- **Fighting.** It may add to your [to-hit](combat.md#hitting) and [damage](combat.md#damage), or
  to your [Dodge](combat.md#defence) and [Parry](combat.md#defence), as a
  [combat stance](stances.md#stances) does, on top of your stance. Creatures can suffer these too.
- **Blows.** It may add damage of some [kind](damage-types.md) to your melee blows with either
  hand, as a [Flaming](affixes.md#prefixes) sword does. [Brand](magic.md#spell-brand) adds 1d4
  [fire](damage-types.md#damage-fire) ([burning blade](#status-branded-blade)). A kick or a shield
  bash does not carry it. No [spell type's rule](spellcasting.md#spell-types) applies to a blow, so
  it never leaves anyone [burning](#burning).
- **Striking back.** It may hurt whoever lands a melee blow on you, as a spell of that kind would.
  [Fire cloak](magic.md#spell-fire-cloak) does 1d6 fire to the attacker
  ([fire-cloaked](#status-fire-cloaked)), and one time in four leaves them burning. What it kills
  counts as your kill.
- **Water.** It may bear you up. [Water walking](magic.md#spell-water-walking)
  ([water-borne](#status-water-borne)) makes [wading](terrain.md#water) cost no more than a step on
  dry land. [Ice](temperature.md#walking-on-ice) never gives way under you, however thin: a burst
  of heat or force leaves the ice you stand on, so you never [fall in](temperature.md#falling-in).
- **Slowness.** It may make everything you do take longer: at 150%, half again as long.
- **What you cannot do.** It may stop you doing something while it lasts. A
  [raging](#status-raging) fighter cannot read ("You cannot read while raging."). Trying takes no
  time, and an [ability](abilities.md) that would do it is greyed out: `you are raging`.
- **What follows.** As some end, another begins: when a [rage](resources.md#rage) subsides,
  [exhaustion](#status-exhausted) follows.
- **Eyes.** It may lend you other eyes. A [potion of owl's eyes](potions.md#potion-of-owls-eyes)
  lets you see in the dark as [darkvision](vision.md#the-four-types) does, in each respect where it
  is better than your own sight.
- **Resistance.** It may make you [resist](damage-types.md#armour-and-resistance) a kind of
  [damage](damage-types.md). This counts like armour's resistance: together with what you wear it
  never goes past 75%, and never gives immunity.

The [potions](potions.md) that last a while give:

- boosts: [Fortified](#status-fortified), [Nimble](#status-nimble), [Hale](#status-hale),
  [Keen](#status-keen), [Steadfast](#status-steadfast), and [Imposing](#status-imposing), each
  a [stat](stats.md) +2 per degree;
- vision: [Cat-eyed](#status-cat-eyed), [Owl-eyed](#status-owl-eyed), and
  [Moth-eyed](#status-moth-eyed);
- resistances: [Warmed](#status-warmed), [Chill-proof](#status-chill-proof),
  [Earthed](#status-earthed), and [Proof against poison](#status-poison-proof).

The bard's [songs](abilities.md#songs) give three more. [Inspired](#status-inspired) (and a
[virtuoso](classes.md#talent-virtuoso)'s greater inspired) lifts the singer.
[Drowsy](#status-drowsy), from the [lullaby](abilities.md#ability-lullaby), slows those who fail to
shake it off. Each goes up to three degrees, as many as the singer's [Music](skills.md#skill-music)
gives the song.

A **hex** is a status that a [hex spell](spellcasting.md#shapes) lays: [Hexed](#status-ill-wished),
[Weakened](#status-weakened), [Sluggish](#status-sluggish), [Drowsy](#status-drowsy), and
[Webbed](#status-webbed). A [witch](classes.md#witch) can feed her hexes:

- Her [cackle](abilities.md#ability-cackle) makes each hex she laid on a creature she sees grow a
  degree, up to its maximum, and starts its degree's time again.
- Her [familiar](familiars.md#the-touch)'s touch lays her Hexed on what it hits, and starts her
  other hexes on it afresh.
- Her [malison](classes.md#talent-malison) makes a creature take 3 more damage from her for each of
  her hexes on it, and a creature that dies under one of her hexes pays her mana.

A **cooldown** is a status effect that only waits to wear off. While you are, say,
[winded](#status-winded), the [ability](abilities.md) that winded you cannot be used again
(`you are winded`). Time is the only price: nothing counts how often you use an ability.

**[Raging](#status-raging)** comes with the berserker's [rage](resources.md#rage), one of the
[class abilities](classes.md#berserker). While the rage lasts you are stronger and tougher but
less clever, and cannot read or spend [mana](mana.md#spending-it). When it runs out you are
**[Exhausted](#status-exhausted)** for two and a half minutes: a little weaker, and with no rage
until it passes.

## In the message line

"You are bleeding." and "You feel sick." stop a [rest](resting.md), like any news. "The bleeding
stops." and "The poison wears off." show in the message line but let you rest on.

## Every status effect

The tables give every status effect in four groups, with the exact paces. **What it does** is
what it costs and changes while it lasts, and how you see it on others. **Caused by** is where it
comes from; the creatures whose attacks cause it are listed in [Who causes it](#who-causes-it).
**How it ends** is how it wears off, and what cures it at once.

### Harmful

These cost [hit points](hit-points.md#hit-points) or [mana](mana.md) as time passes, and can kill.

{{#include generated/status-effects.md:harmful}}

### Hindering

These cost nothing, but leave you weaker, slower, or unable to do something.

{{#include generated/status-effects.md:hindering}}

### Helpful

These help whoever has them. Some cost something besides, as a rage does.

{{#include generated/status-effects.md:helpful}}

### Cooldowns and marks

These do nothing by themselves. A cooldown only waits to wear off, and keeps the ability that
caused it from being used again. A mark matters to something else: a light, a smite, a
[spell circle](magic.md).

{{#include generated/status-effects.md:cooldowns}}

## Who causes it

Every creature whose attack, throw, or gaze can cause a status effect, with the chance a hit has
and how bad it makes it. A [Held](held.md) creature's grip is the difficulty of breaking free.

{{#include generated/status-causes.md}}

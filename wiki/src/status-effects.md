# Status effects

Some harm lasts longer than a blow. A deep wound keeps bleeding, and a bite can leave poison
behind. These **status effects** cost [hit points](combat.md#hit-points) as time passes, slow your
[healing](healing.md), sometimes weaken you, and wear off on their own. They can kill.
**[Diseased](status-effects.md#status-diseased)** is a status effect too, shown while any [disease](diseases.md) has broken out; it
lasts as long as the diseases do. Not every status effect harms: a [rage](resources.md#rage) makes you stronger for a
while, and some leave you spent once they are over (see [Buffs and cooldowns](#buffs-and-cooldowns)).

You always know what ails you: the [status bar](controls.md#the-screen) names it right after your
[hit points](combat.md#hit-points) (`HP 12/17 · Bleeding · Poisoned`), the [character screen](character.md) lists it, and
the message line tells you when it starts and stops. You never see how bad it is, or how long it
will last, as a number: you see your [hit points](combat.md#hit-points) go. The table gives the exact paces, and who
causes what, with the chance a hit has:

{{#include generated/status-effects.md}}

## How they work

- **Severity.** Every status effect has a severity, in degrees. The worse it is, the faster it
  costs [hit points](combat.md#hit-points): a wound twice as bad bleeds twice as fast.
- **They add up.** A second wound while you are still [bleeding](#bleeding) makes the bleeding worse; a
  second dose of [poison](#poisoned) makes you sicker, and stronger poison lasts longer. Each status has a
  worst it can get.
- **They wear off.** Bit by bit, a degree at a time, until they are gone.
- **Time is time.** Status effects run on game time, whatever you do meanwhile: waiting, walking,
  [resting](resting.md), or a long journey on the [overworld](overworld.md) all count the same.
  The game only skips them for creatures on other levels, which wait for you to come back.
- **They can kill.** If a status effect takes your last [hit point](combat.md#hit-points), you die of it, and the death
  screen says so: "Bled to death".

## Bleeding

A **[big hit](combat.md#damage)**, one that takes at least a third of your [hit points](combat.md#hit-points) at once, or a **[confirmed
critical](combat.md#criticals)** ("The kobold stabs you hard!") opens a wound: "You are bleeding." A critical opens a
worse one than a big hit. Only a blow that **cuts or stabs** does it: a club's, a fall's, or a
[fireball](magic.md#spell-fireball)'s does not (see [Damage types](damage-types.md)).

While you bleed, you lose a [hit point](combat.md#hit-points) every so often, faster the worse you bleed, and your [hit
points](combat.md#hit-points) do not come back at all. The bleeding eases by itself and stops after a while: "The
bleeding stops." If you are trained in **[Heal](skills.md#skill-heal)**, you can bind the wound and stop it sooner, trying
again as often as it takes (see [Treating yourself](skills.md#treating-yourself)).

Creatures bleed the same way, and you can see it: [look mode](controls.md#look-mode) says "A giant
rat (hostile, wounded, bleeding)". A creature can bleed to death after a fight: "The giant rat
bleeds to death."

## Poisoned

Some creatures' attacks carry poison: now and then a [giant rat](creatures.md#creature-rat)'s filthy bite leaves you sick.
"You feel sick." So does bad food: [rotten food](food.md#food-spoils) (a [corpse](food.md#corpses) too) that stays down, and the flesh of
some creatures (see [Food and corpses](food.md)).

Poison works slowly: a [hit point](combat.md#hit-points) every couple of minutes, faster the stronger it is, and your [hit
points](combat.md#hit-points) come back only half as fast. **Strong poison** (a few doses at once) also makes you
weaker and clumsier: your [Strength](stats.md#stat-strength) and [Dexterity](stats.md#stat-dexterity) count one less, in everything they do, from
[hitting](combat.md#hitting) and [Defence](combat.md#defence) to how much you can [carry](burden.md#how-much-you-can-carry). The [character screen](controls.md#character) shows them lowered. Poison
wears off by itself: "The poison wears off." Trained in **[Heal](skills.md#skill-heal)**, you may try once to draw out
some of a dose (see [Treating yourself](skills.md#treating-yourself)).

You cannot tell whether a creature is poisoned by looking at it.

## Held

Some creatures grab: a bite or a clutch that lands may hold you fast ("You are held fast."), and the
attack table on the [creature numbers](creature-numbers.md) page says which do, how often, and how
hard they grip. While **held** you cannot step away; you can still fight, and the one holding you is
the obvious target.

Each step you try is a **struggle** to break free: a check of your [Strength](stats.md#stat-strength) (2d12 plus its
modifier, as an [attack roll](combat.md#hitting); see [Stats](stats.md)) against the holder's **grip**, 13 unless the
creature's attack says otherwise. Made: "You break free of the ghast." and you take the step.
Failed: "You struggle, but the ghast holds you fast." and the turn is gone.

A hold also ends by itself, "You are free to move.":

- when the holder dies, is knocked away, or ends up anywhere but beside you;
- when the holder is held or slowed itself ([staggered](status-effects.md#status-staggered), [transfixed](status-effects.md#status-transfixed), [sluggish](status-effects.md#status-sluggish));
- after **3d4 turns** at most. Another hit draws the turns again and keeps the longer of the two.

A creature in your own [Drowning grip](talents.md#talent-drowning-grip) (a [lucky day](horoscope.md#lucky-days)'s gift) is held
the same way, and lets go the same way; it does not struggle, it fights you instead.

## Ebbing

A [draining room](special-rooms.md#room-draining) drinks at magic. While you stand in one, and half a minute
after you leave, you are **Ebbing** ("Your power ebbs away."): a point of [mana](mana.md#how-much) drains away every
20 seconds, and none comes back. It never takes you below nothing and never harms you otherwise.
Out of the room it passes: "Your power stops ebbing." Creatures with [mana](mana.md#how-much) ebb in the room just as
you do, and you can see it on them ("ebbing"); those without mana are left alone. The talent
*[Old delver](talents.md#talent-old-delver)* keeps it from ever taking you.

## Asleep

The creatures of a [teeming den](special-rooms.md#room-teeming) lie **asleep** ("asleep" in look
mode): a sleeper does nothing at all and has seen no one, so a [sneak attack](talents.md#talent-sneak-attack) always lands on it.
Time never wakes it; noise loud enough, any harm, or a waking neighbour's cry does.

## Silenced

A [silent vault](special-rooms.md#room-silent) swallows every sound. While you stand in one, and ten seconds
after you leave, you are **Silenced** ("The silence closes around you."): you cannot cast a
[spell](magic.md#the-spells) (nor spend [mana](mana.md#spending-it) on anything), read a [scroll](scrolls.md), or [throw your voice](hearing.md#throwing-your-voice); trying takes no time.
Out of the vault sound comes back: "Sound returns." Creatures inside are silenced just as you
are, and you can see it on them ("silenced"): a caster that cannot cast closes in to fight
hand to hand.

## Unwelcome

A [shrine](special-rooms.md#room-shrine) of the other side from yours does not want you. While you stand in
one, and half a minute after you leave, you are **Unwelcome** ("You feel unwelcome."): your
[wounds heal](healing.md#how-fast) at half the pace and you are a little less likely to hit (−1 [to hit](combat.md#hitting)). Creatures of
the other side are unwelcome in it too, and you can see it on them ("unwelcome"). "You no
longer feel unwelcome." once it passes.

## Buffs and cooldowns

A status effect can also help, or hinder, in other ways while it lasts (the table's last
column says which):

- **Stats.** It may raise a score as well as lower one. [Hit points](combat.md#hit-points) that a higher [Toughness](stats.md#stat-toughness) let
  you [heal](healing.md#how-fast) up to go again when it ends.
- **Fighting.** It may add to your [to-hit](combat.md#hitting) and [damage](combat.md#damage), or to your [Dodge](combat.md#defence) and [Parry](combat.md#defence), as a
  [combat stance](combat.md#stances) does, on top of your stance. Creatures can suffer these too.
- **Slowness.** It may make everything you do take longer: at 150%, half again as long.
- **What you cannot do.** It may keep you from doing something while it lasts: a [raging](status-effects.md#status-raging) fighter
  cannot read ("You cannot read while raging."). Trying takes no time. An
  [ability](abilities.md) that would do it is greyed out: `you are raging`.
- **What follows.** As some end, another begins: when a [rage](resources.md#rage) subsides, [exhaustion](status-effects.md#status-exhausted) follows.
- **Eyes.** It may lend you other eyes: a [potion of owl's eyes](potions.md#potion-of-owls-eyes) lets you see in the dark as
  [darkvision](vision.md#the-four-types) does, part by part where it is better than your own sight.
- **Resistance.** It may make you [resist](damage-types.md#armour-and-resistance) a kind of [damage](damage-types.md), counted as armour's resistance is:
  together with what you wear it never goes past 75%, never immunity.

The [potions](potions.md) that work for a while give the boosts ([Fortified](status-effects.md#status-fortified), [Nimble](status-effects.md#status-nimble), [Hale](status-effects.md#status-hale), [Keen](status-effects.md#status-keen),
[Steadfast](status-effects.md#status-steadfast), [Imposing](status-effects.md#status-imposing): a [stat](stats.md) +2 a degree), the visions ([Cat-eyed](status-effects.md#status-cat-eyed), [Owl-eyed](status-effects.md#status-owl-eyed), [Moth-eyed](status-effects.md#status-moth-eyed)), and the
resistances ([Warmed](status-effects.md#status-warmed), [Chill-proof](status-effects.md#status-chill-proof), [Earthed](status-effects.md#status-earthed), [Proof against poison](status-effects.md#status-poison-proof)).

A **cooldown** is a status effect that only waits to wear off: while you are, say, [winded](status-effects.md#status-winded), the
[ability](abilities.md) that winded you cannot be used again (`you are winded`). Time is the only price; nothing
counts how often you use an ability.

**[Raging](status-effects.md#status-raging)** (the barbarian's [rage](resources.md#rage), which comes with the [class abilities](talents.md#class-abilities)): while your
[rage](resources.md#rage) lasts you are stronger and tougher but less clever, and cannot read or spend
[mana](mana.md#spending-it). When the rage runs out you are **[Exhausted](status-effects.md#status-exhausted)** for ten minutes: a little weaker, and no rage
until it passes.

## In the message line

"You are bleeding." and "You feel sick." stop a [rest](resting.md), like any news. "The bleeding
stops." and "The poison wears off." are written to the message line but let you rest on.

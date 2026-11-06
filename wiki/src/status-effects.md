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
bleeds to death." A creature without blood (a [skeleton](creatures.md#creature-skeleton), a ghost, an elemental: the
[creature tables](creature-numbers.md#hit-points-defence-and-resistances) say which) never bleeds, whatever wounds it: neither a blow nor
[Flay](magic.md#spell-flay) opens it.

## Burning

A [fire spell](magic.md#spell-types) sometimes leaves what it harms burning. "You are burning!" It costs a [hit point](combat.md#hit-points) every
ten seconds, faster the more degrees of it you have, and each degree burns out in half a minute: "The flames go out." What
resists [fire](damage-types.md#damage-fire) takes it less, and what is immune never burns.

What burns gives off a small light, so it cannot hide in the dark, and look mode names it: "A kobold (hostile, burning)".
A step into [water](terrain.md#water), or a fall through ice into it, puts it out at once, and so does a [cold spell](magic.md#spell-types) that harms you.

## Corroded

Every [acid spell](magic.md#spell-types) that harms something leaves it Corroded: "The acid keeps eating." A degree costs a
[hit point](combat.md#hit-points) every six seconds and is spent in twelve, so two more hit points a cast: "The acid is spent." What
resists [acid](damage-types.md#damage-acid) takes it less. Look mode names it. It never eats armour.

## Poisoned

Some creatures' attacks carry poison: now and then a [giant rat](creatures.md#creature-rat)'s filthy bite leaves you sick.
"You feel sick." So does bad food: [rotten food](food.md#food-spoils) (a [corpse](food.md#corpses) too) that stays down, and the flesh of
some creatures (see [Food and corpses](food.md)).

Poison works slowly: a [hit point](combat.md#hit-points) every couple of minutes, faster the stronger it is, and your [hit
points](combat.md#hit-points) come back only half as fast. **Strong poison** (a few doses at once) also makes you
weaker and clumsier: your [Strength](stats.md#stat-strength) and [Dexterity](stats.md#stat-dexterity) count one less, in everything they do, from
[hitting](combat.md#hitting) and [Defence](combat.md#defence) to how much you can [carry](burden.md#how-much-you-can-carry). The [character screen](controls.md#character) shows them lowered. Poison
wears off by itself: "The poison wears off." Trained in **[Heal](skills.md#skill-heal)**, you may try to draw out
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

### Slipping bonds

If you are trained in [Escape artist](skills.md#skill-escape-artist), you slip bonds without being
asked: there is nothing to choose, no [ability](abilities.md) and no key. When
[Held](status-effects.md#status-held), [Webbed](status-effects.md#status-webbed), or
[Pinned](status-effects.md#status-pinned) stops what you try, the check happens as part of the
attempt: the better of your [Strength](stats.md#stat-strength) and
[Dexterity](stats.md#stat-dexterity) plus what [the skill](skills.md#what-skills-give) gives (and
[Knowledge](skills.md#skill-knowledge), against a holder), 2d12 as an
[attack roll](combat.md#hitting), against the grip, or 13 for a web or a pin. Made, the status ends
and the action goes ahead at its usual time. Failed, the action's time is gone: "You struggle,
but you are still caught in a web." It costs no more than the wait you would have had. If two
bonds stop you, the other asks on your next attempt, and this step is refused at no cost. A
[pin](status-effects.md#status-pinned) eases a degree with each action, a failed struggle too.

Without the skill a hold is still a struggle of Strength alone, and a web or a pin simply refuses
the step, at no cost, until it passes. Slowed minds and bodies
([Staggered](status-effects.md#status-staggered), [Stunned](status-effects.md#status-stunned),
[Surprised](status-effects.md#status-surprised), [Transfixed](status-effects.md#status-transfixed),
[Sluggish](status-effects.md#status-sluggish), [Drowsy](status-effects.md#status-drowsy)) are no
bonds, and the skill does nothing for them.

### When a hold ends

A hold also ends by itself, "You are free to move.":

- when the holder dies, is knocked away, or ends up anywhere but beside you;
- when the holder is held or slowed itself ([staggered](status-effects.md#status-staggered), [transfixed](status-effects.md#status-transfixed), [sluggish](status-effects.md#status-sluggish));
- after **3d4 turns** at most. Another hit draws the turns again and keeps the longer of the two.

A creature in your own [Drowning grip](talents.md#talent-drowning-grip) (a [lucky day](horoscope.md#lucky-days)'s gift) is held
the same way, and lets go the same way; it does not struggle, it fights you instead.

### Engulfed, swallowed, latched

Worse holds hurt while they last. The [charnel heap](creatures.md#creature-charnel-heap) and the
greater water elementals close over you ([Engulfed](status-effects.md#status-engulfed): "It closes
over you."), the [nightworm](creatures.md#creature-nightworm) swallows you whole
([Swallowed](status-effects.md#status-swallowed)), and a [giant leech](creatures.md#creature-giant-leech)
or a [canoloth](creatures.md#creature-canoloth)'s tongue fastens on
([Latched](status-effects.md#status-latched)). Each is a hold like any other: the struggle, the
grip (14 unless the attack says otherwise), and the endings above, but it lasts **2d4 turns** at
most, and each costs [hit points](combat.md#hit-points) as it ticks: Engulfed 2 every 6 seconds
for each turn it has left, Swallowed 4, Latched 1 every 12 seconds, so the harm is heaviest
as it begins. Engulfed and Swallowed also stop you shooting, throwing, and reading; your blows and
your spells still go, so you can hack your way out of what holds you.

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
[spell](magic.md#the-spells) (nor spend [mana](mana.md#spending-it) on anything), read a [scroll](scrolls.md), sing a song ([inspire](abilities.md#ability-inspire), the [lullaby](abilities.md#ability-lullaby)), or [throw your voice](hearing.md#throwing-your-voice); trying takes no time.
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
- **Blows.** It may add damage of a [kind](damage-types.md) to your melee blows with either hand, as a
  [Flaming](magic-items.md#prefixes) sword's: [Brand](magic.md#spell-brand) does 1d4 [fire](damage-types.md#damage-fire) more ([burning blade](status-effects.md#status-branded-blade)). A kick or a shield bash does not
  carry it. No [spell type's rule](magic.md#spell-types) reads a blow, so it never leaves anyone [burning](status-effects.md#burning).
- **Striking back.** It may hurt whoever lands a melee blow on you, as a spell of that kind would:
  [Fire cloak](magic.md#spell-fire-cloak) does 1d6 fire to the striker ([fire-cloaked](status-effects.md#status-fire-cloaked)), and one time in four leaves them burning. What it kills is your kill.
- **Water.** It may bear you up: [Water walking](magic.md#spell-water-walking) ([water-borne](status-effects.md#status-water-borne)) makes [wading](terrain.md#water) cost no more than a
  step on dry land, and [ice](temperature.md#walking-on-ice) never gives way under you, however thin: a burst of heat or force leaves the ice you stand on, so you never [fall in](temperature.md#falling-in).
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

The bard's [songs](abilities.md#songs) give three more: [Inspired](status-effects.md#status-inspired)
(and a [virtuoso](talents.md#talent-virtuoso)'s greater inspired) lifts the singer, and
[Drowsy](status-effects.md#status-drowsy), from the [lullaby](abilities.md#ability-lullaby), slows
those who fail to shake it off; each goes up to three degrees, as many as a song with
[Music](skills.md#skill-music) gives.

A **hex** is a status a [hex spell](magic.md#shapes) lays: [Hexed](status-effects.md#status-ill-wished),
[Weakened](status-effects.md#status-weakened), [Sluggish](status-effects.md#status-sluggish),
[Drowsy](status-effects.md#status-drowsy), and [Webbed](status-effects.md#status-webbed). A
[witch](classes.md#witch)'s [cackle](abilities.md#ability-cackle) keeps the hexes she laid on: each
on a creature she sees grows a degree, to its maximum, and its degree's time starts again. Her
[familiar](allies.md#the-touch)'s touch lays her Hexed on what it hits and starts her other hexes on it afresh. Her
[malison](talents.md#talent-malison) makes a creature under a hex take 3 more damage from her
for each hex, and one dying under a hex of hers pays her mana.

A **cooldown** is a status effect that only waits to wear off: while you are, say, [winded](status-effects.md#status-winded), the
[ability](abilities.md) that winded you cannot be used again (`you are winded`). Time is the only price; nothing
counts how often you use an ability.

**[Raging](status-effects.md#status-raging)** (the berserker's [rage](resources.md#rage), which comes with the [class abilities](talents.md#class-abilities)): while your
[rage](resources.md#rage) lasts you are stronger and tougher but less clever, and cannot read or spend
[mana](mana.md#spending-it). When the rage runs out you are **[Exhausted](status-effects.md#status-exhausted)** for two and a half minutes: a little weaker, and no rage
until it passes.

## In the message line

"You are bleeding." and "You feel sick." stop a [rest](resting.md), like any news. "The bleeding
stops." and "The poison wears off." are written to the message line but let you rest on.

# How creatures fight

INTRO

## Attacks

Blows and shots are rolled, hit or miss, hurt, and kill, by the rules of [Combat](combat.md):
"The kobold hits you.", "The giant rat bites the kobold hard!", "You block the kobold's spear
with your shield.", "The kobold archer shoots the giant rat.", "The giant rat dies." Casters
cast real [spells](magic.md) (see [Spells](creature-powers.md#spells)): "The kobold shaman casts
spark at you!", "The spark shocks you." You only hear of attacks you perceive:
ones where you see the attacker or the target, or are the target. An end you cannot see is
"something": "Something hits you.", "The kobold archer misses something." Attacks are [noisy](hearing.md#how-loud), so
you may also hear fighting, a bowstring, or chanting out of sight.

A creature that dies is gone, and what it carried lies where it fell. Creatures do not flee or
fight any differently when they are hurt, but [look mode](controls.md#look-mode) shows how hurt
they look, and whether they [bleed](status-effects.md#bleeding). [Big hits](combat.md#damage) and criticals make creatures [bleed](status-effects.md#bleeding) as they make you
bleed, and a badly wounded one may [bleed to death](status-effects.md#bleeding) after the fight; a [giant rat](creatures.md#creature-rat)'s bite may [poison](status-effects.md#status-poisoned)
whatever it bites (see [Status effects](status-effects.md)), or give it [filth fever](diseases.md#disease-filth-fever) (see
[Diseases](diseases.md)).

When you see both ends of an attack, you also see it happen: a creature fighting up close (you
too) lunges at its target, an arrow flies from the archer to its target, and a spell lands with a
faint violet sparkle (with the matching [settings](settings.md) on).

Moving into a hostile creature attacks it ("You hit the giant rat.", "You miss something." if
you cannot see it), which takes a turn. Moving into any other creature [swaps places](npcs.md#getting-past) with it, for
your step's time ("You swap places with the deer."); it walks back to where it stood afterwards
and carries on. A few will not make way ("The sentry doesn't budge."), which takes no time; see
[People and talk](npcs.md#getting-past).

The people of [Cēaptūn](ascmearc.md#the-places) (the [townsfolk](creatures.md#creature-villager), [Sister Hild](creatures.md#creature-sister), [Godgifu](creatures.md#creature-innkeeper), the [reeve](creatures.md#creature-reeve), the [mine-factor](creatures.md#creature-mine-factor)) are
peaceful folk by nature, and talk to you (see [People and talk](npcs.md)): the town is safe.
A creature hostile by nature may also be placed among the **[peaceful folk](creatures.md#factions)** by the place it lives
in: then it walks its rounds and never attacks you. Fighting happens in the dungeons (see
[Dungeons and danger](dungeons.md)).

**[Shopkeepers](creatures.md#creature-shopkeeper)** are **merchants**, who trade with everyone and fight no one, until you [rob or
attack](shops.md#theft) one: then it joins the outraged shopkeepers and comes after you (see [Shops](shops.md)).

Creatures of factions at war fight each other on sight, wherever they meet: a demon and a devil
fight to the death, and both turn on you when they see you. Light your [lantern](lantern.md) near them and
their archers will find you.

**Trolls** are a tribe of their own (the trolls faction) from about level 13 to 42: the [troll](creatures.md#creature-troll) and
the [hill troll](creatures.md#creature-hill-troll) of [Fyrdelf](dungeon-guide.md#fyrdelf), and the [forest](creatures.md#creature-forest-troll), [ice](creatures.md#creature-ice-troll), [stone](creatures.md#creature-stone-troll), [two-headed](creatures.md#creature-two-headed-troll), and [fire trolls](creatures.md#creature-fire-troll), the [troll
shaman](creatures.md#creature-troll-shaman), the [troll chieftain](creatures.md#creature-troll-chieftain), and the [elder troll](creatures.md#creature-elder-troll) further on. Every creature's [hit points](hit-points.md#hit-points) come
back by its [Toughness](stats.md#stat-toughness), and a troll's [Toughness](stats.md#stat-toughness) is the greatest of anything its size, so its
wounds close while you watch: hit it hard and keep hitting. [Fire](damage-types.md#damage-fire) hurts every troll more than
steel (half again as much; the stone troll a quarter more) but the [fire troll](creatures.md#creature-fire-troll), which shrugs it off and hates the [cold](damage-types.md#damage-cold) instead.
The [marsh troll](creatures.md#creature-marsh-troll) of the fen is a marsh thing and the [rotting troll](creatures.md#creature-rotting-troll) is undead, but a [quest](quests.md) to kill
trolls counts them too.

## Hides and bodies

What a creature is made of decides how much of each blow it shrugs off (its
[Protection](combat.md#protection)), and the older and stronger the kind, the more. Soft things
(skin, fur, feathers, dead flesh) give little until they grow huge; bony things (bones, scales,
chitin, shell) turn blades; the hard-skinned (stone, ice, iron-hard skin) turn most blows. Spirits,
flame, wind, and water have nothing to cut through, and none at all. A blow that does nothing
"glances off" the hide. People who fight in armour (kobolds, goblins, orcs, the drow, the duergar,
outlaws) have no hide of their own: what they wear is their [Protection](combat.md#protection). Hard bodies are what
penetration is for: the war hammer, the stiletto, [crossbow bolts](ammunition.md), and [criticals](combat.md#criticals), which are soaked
only once after they multiply the damage (the **[Armour-breaker](talents.md#talent-armour-breaker)** talent's criticals drive through
2 more).

[Protection](combat.md#protection) by body and level (rounded down; each kind's own is in
[Creatures in numbers](creature-numbers.md)):

| Body | Protection at level L | L 10 | L 20 | L 30 | L 40 | L 50 | L 60 | L 70 |
|---|---|---|---|---|---|---|---|---|
| None (spirits, flame, wind, water) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Hide (skin, fur, feathers, dead flesh, wrappings) | (L − 10) ÷ 8 | 0 | 1 | 2 | 3 | 5 | 6 | 7 |
| Bony (bones, scales, chitin, shell, plates) | L ÷ 6 | 1 | 3 | 5 | 6 | 8 | 10 | 11 |
| Armoured (stone, ice, iron-hard skin, slag, cinders) | L ÷ 4 + 1 | 3 | 6 | 8 | 11 | 13 | 16 | 18 |

The exceptions: the [cave beetle](creatures.md#creature-beetle)'s and the [ember beetle](creatures.md#creature-ember-beetle)'s shells (3), the [stone troll](creatures.md#creature-stone-troll)'s stony hide
(5), the [mud golem](creatures.md#creature-mud-golem) (5: a blow sinks into it), the [balor](creatures.md#creature-balor) and the [pit fiend](creatures.md#creature-pit-fiend) (10, held to the bony
line), the hags who cast (half a hide's), and every living [troll](creatures.md#creature-troll) (a hide of at least 2). The
[animated objects](animated.md) have their own table.

## Incorporeal creatures

Some restless dead have no body to stop them: **incorporeal** creatures (the [ghosts](creatures.md#creature-ghost), [shadows](creatures.md#creature-shadow),
[poltergeists](creatures.md#creature-poltergeist), [corpse candles](creatures.md#creature-corpse-candle), and wraiths of [Fennmynster](dungeon-guide.md#fennmynster), the [knockers](creatures.md#creature-knocker) and [spectres](creatures.md#creature-spectre) of [Fyrdelf](dungeon-guide.md#fyrdelf),
and, far deeper, [banshees](creatures.md#creature-banshee), [dread wraiths](creatures.md#creature-dread-wraith), [greater shadows](creatures.md#creature-greater-shadow), [white ladies](creatures.md#creature-white-lady), [wraith lords](creatures.md#creature-wraith-lord), the [sluagh](creatures.md#creature-sluagh), [washers at the ford](creatures.md#creature-washer-at-the-ford), [night mares](creatures.md#creature-night-mare), and [ghost lords](creatures.md#creature-ghost-lord)), and the [earth elementals](creatures.md#creature-earth-elemental), whose element is the stone, drift through walls, solid rock, trees, and closed [doors](doors.md) (which stay shut behind them), and may
stop inside the stone. They cannot leave the level that way, and [lava](terrain.md#terrain-lava) stops them as it stops
anyone. Nothing of them presses down: they set off no [traps](traps.md) and never break [ice](temperature.md#walking-on-ice).

One inside a wall is still there to fight. You see it when you see that piece of wall: [lit](light.md#what-light-is), or
close enough for [darkvision](vision.md#the-four-types). Move into the wall where you see it and you strike at it; it strikes
out of the wall at whoever stands beside it; an [arrow](ammunition.md) that reaches it is lost in the stone if it
hits, and falls before the wall if it misses. The stone shelters it, though: inside a wall it is
harder to [hit](combat.md#hitting) (its **Wall** [Defence](combat.md#defence), in the Defence table on
[Creatures in numbers](creature-numbers.md)), and some misses just strike stone ("Your sword
strikes stone where the stone haunt hides in the wall."). If you cannot see it there, the wall is
only a wall. What one carried falls out of the wall when it dies, onto the nearest floor.

## Elementals

**Elementals** are fire, earth, air, water, ice, and lightning given a shape and a will. Each
element comes in sizes and ages, from a **mote** (small, `e`) through the lesser and the plain
elemental to the greater and the elder (`E`), and the eldest of each has an old name: the
**[balefire](creatures.md#creature-balefire)**, the **[hoarstone](creatures.md#creature-hoarstone)**, the **[welkin wind](creatures.md#creature-welkin-wind)**, the **[brimwolf](creatures.md#creature-brimwolf)**, the **[rimethurse](creatures.md#creature-rimethurse)**, and
the **[thunderhead](creatures.md#creature-thunderhead)**. All of them are hostile to you and leave everyone else alone, [poison](damage-types.md#damage-poison) cannot
hurt them, and nothing is left of one when it is destroyed (the oldest leave something they
caught up now and then).

- **Fire** burns what it touches, cannot be hurt by [fire](damage-types.md#damage-fire), is hurt more by [cold](damage-types.md#damage-cold), and lights its
  way.
- **Earth** is slow and heavy and hard to cut or pierce; [electricity](damage-types.md#damage-electricity) cannot hurt it. Its blows
  may [stagger](status-effects.md#status-staggered) you (the eldest also [grip and hold](status-effects.md#status-held)), and from the earth elemental up it moves
  through stone as an [incorporeal](#incorporeal-creatures) creature does (above), though the stone shelters it less.
- **Air** is very [fast](turns.md#speed) and hard to hit, [arrows](ammunition.md) blow aside from it, and its blows may [stagger](status-effects.md#status-staggered) you.
- **Water** walks on the land; [fire](damage-types.md#damage-fire) and arrows do it less harm, [electricity](damage-types.md#damage-electricity) more. Its blows may
  leave you [sluggish](status-effects.md#status-sluggish), and the greater ones may [hold](status-effects.md#status-held) you.
- **Ice** is [slow](turns.md#speed), its cold blows may leave you [sluggish](status-effects.md#status-sluggish), [cold](damage-types.md#damage-cold) cannot hurt it, and [fire](damage-types.md#damage-fire) hurts it
  more.
- **Lightning** is [fast](turns.md#speed), lit blue, cannot be hurt by [electricity](damage-types.md#damage-electricity), and its shocks may [stagger](status-effects.md#status-staggered) you.

The exact numbers are on [Creatures in numbers](creature-numbers.md).

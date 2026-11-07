# How creatures fight

This page covers how creatures attack and what you are told about it, who fights whom, and how
much of a blow each kind of body shrugs off. It ends with two odd kinds: the incorporeal, who pass
through walls, and the elementals. The exact numbers for every creature are on
[Creatures in numbers](creature-numbers.md), and their special powers on
[Creature powers](creature-powers.md).

## Attacks

Creatures' blows and shots follow the rules of [Combat](combat.md), the same as yours. They hit or
miss, hurt, and kill: "The kobold hits you.", "The giant rat bites the kobold hard!", "You block
the kobold's spear with your shield.", "The kobold archer shoots the giant rat.", "The giant rat
dies." Casters cast real [spells](magic.md) (see [Spells](creature-tricks.md#spells)): "The kobold
shaman casts spark at you!", "The spark shocks you."

You are told only of attacks you perceive: those where you see the attacker or the target, or
where you are the target. An end you cannot see is "something": "Something hits you.", "The
kobold archer misses something." Attacks are [noisy](hearing.md#how-loud), so you may also hear
fighting, a bowstring, or chanting out of sight.

When you see both ends of an attack, you also see it happen. A creature fighting up close (you
too) lunges at its target, an arrow flies from the archer to its target, and a spell lands with a
faint violet sparkle. This needs the matching [settings](settings.md) on.

### Wounds and death

A creature that dies is gone, and what it carried lies where it fell. Creatures do not flee or
fight differently when they are hurt, but [look mode](controls.md#look-mode) shows how hurt they
look and whether they [bleed](status-effects.md#bleeding).

[Big hits](combat.md#damage) and criticals make creatures [bleed](status-effects.md#bleeding),
just as they make you bleed. A badly wounded creature may
[bleed to death](status-effects.md#bleeding) after the fight. Some attacks carry more: a
[barrow spider](creatures.md#creature-barrow-spider)'s bite may
[poison](status-effects.md#status-poisoned) whatever it bites (see
[Status effects](status-effects.md)), and once in a long while a
[giant rat](creatures.md#creature-rat)'s gives it [filth fever](diseases.md#disease-filth-fever)
(see [Diseases](diseases.md)).

### Bumping into creatures

Moving into a hostile creature attacks it, which takes a turn ("You hit the giant rat.", or "You
miss something." if you cannot see it).

Moving into any other creature [swaps places](npcs.md#getting-past) with it, for your step's time
("You swap places with the deer."). Afterwards it walks back to where it stood and carries on. A
few will not make way ("The sentry doesn't budge."), which takes no time; see
[People and talk](npcs.md#getting-past).

### Who fights

The people of [Cēaptūn](ascmearc.md#the-places) are peaceful folk by nature and talk to you (see
[People and talk](npcs.md)): the [townsfolk](creatures.md#creature-villager),
[Sister Hild](creatures.md#creature-sister), [Godgifu](creatures.md#creature-innkeeper), the
[reeve](creatures.md#creature-reeve), and the [mine-factor](creatures.md#creature-mine-factor). The
town is safe. The fighting happens in the dungeons (see [Dungeons and danger](dungeons.md)).

A creature that is hostile by nature may still be placed among the
**[peaceful folk](creatures.md#factions)** by the place it lives in. Then it walks its rounds and
never attacks you.

**[Shopkeepers](creatures.md#creature-shopkeeper)** are **merchants**. They trade with everyone
and fight no one, until you [rob or attack](shops.md#theft) one. Then it joins the outraged
shopkeepers and comes after you (see [Shops](shops.md)).

Creatures of factions at war fight each other on sight, wherever they meet. A demon and a devil
fight to the death, and both turn on you when they see you. Light your [lantern](lantern.md) near
them and their archers will find you.

### Trolls

**Trolls** are a tribe of their own (the trolls faction), from about level 13 to 42:

- the [troll](creatures.md#creature-troll) and the [hill troll](creatures.md#creature-hill-troll)
  of [Fyrdelf](dungeon-guide.md#fyrdelf);
- further on, the [forest](creatures.md#creature-forest-troll), [ice](creatures.md#creature-ice-troll),
  [stone](creatures.md#creature-stone-troll), [two-headed](creatures.md#creature-two-headed-troll),
  and [fire trolls](creatures.md#creature-fire-troll), the
  [troll shaman](creatures.md#creature-troll-shaman), the
  [troll chieftain](creatures.md#creature-troll-chieftain), and the
  [elder troll](creatures.md#creature-elder-troll).

Every creature's [hit points](hit-points.md#hit-points) come back at a pace set by its
[Toughness](stats.md#stat-toughness). A troll's Toughness is the greatest of anything its size, so
its wounds close while you watch. Hit it hard and keep hitting.

[Fire](damage-types.md#damage-fire) hurts every troll more than steel does: half again as much,
or a quarter more for the stone troll. The exception is the
[fire troll](creatures.md#creature-fire-troll), which shrugs fire off and hates the
[cold](damage-types.md#damage-cold) instead.

The [marsh troll](creatures.md#creature-marsh-troll) of the fen is a marsh thing, and the
[rotting troll](creatures.md#creature-rotting-troll) is undead, but a [quest](quests.md) to kill
trolls counts them too.

## Hides and bodies

What a creature is made of decides how much of each blow it shrugs off. This is its
[Protection](combat.md#protection), and the older and stronger the kind, the more it has.

- Soft bodies (skin, fur, feathers, dead flesh) give little until they grow huge.
- Bony bodies (bones, scales, chitin, shell) turn blades.
- Hard-skinned bodies (stone, ice, iron-hard skin) turn most blows.
- Spirits, flame, wind, and water have nothing to cut through, and so no Protection at all.

A blow that does nothing "glances off" the hide.

People who fight in armour have no hide of their own: kobolds, goblins, orcs, the drow, the
duergar, and outlaws. What they wear is their [Protection](combat.md#protection).

Hard bodies are what armour-piercing is for: the war hammer, the stiletto,
[crossbow bolts](ammunition.md), and [criticals](combat.md#criticals). A critical's damage is
multiplied first and the Protection taken off only once. The
**[Armour-breaker](talents.md#talent-armour-breaker)** talent's criticals drive through 2 more.

[Protection](combat.md#protection) by body and level, rounded down (each kind's own is in
[Creatures in numbers](creature-numbers.md)):

| Body | Protection at level L | L 10 | L 20 | L 30 | L 40 | L 50 | L 60 | L 70 |
|---|---|---|---|---|---|---|---|---|
| None (spirits, flame, wind, water) | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Hide (skin, fur, feathers, dead flesh, wrappings) | (L − 10) ÷ 8 | 0 | 1 | 2 | 3 | 5 | 6 | 7 |
| Bony (bones, scales, chitin, shell, plates) | L ÷ 6 | 1 | 3 | 5 | 6 | 8 | 10 | 11 |
| Armoured (stone, ice, iron-hard skin, slag, cinders) | L ÷ 4 + 1 | 3 | 6 | 8 | 11 | 13 | 16 | 18 |

The exceptions:

- the [cave beetle](creatures.md#creature-beetle)'s and the
  [ember beetle](creatures.md#creature-ember-beetle)'s shells: 3;
- the [stone troll](creatures.md#creature-stone-troll)'s stony hide: 5;
- the [mud golem](creatures.md#creature-mud-golem): 5 (a blow sinks into it);
- the [balor](creatures.md#creature-balor) and the [pit fiend](creatures.md#creature-pit-fiend):
  10, held to the bony line;
- the hags who cast: half a hide's;
- every living [troll](creatures.md#creature-troll): a hide of at least 2.

The [animated objects](animated.md) have their own table.

## Incorporeal creatures

Some creatures have no body to stop them. These **incorporeal** creatures drift through walls,
solid rock, trees, and closed [doors](doors.md), and may stop inside the stone. A door stays shut
behind them. They are:

- among the restless dead, the [ghosts](creatures.md#creature-ghost),
  [shadows](creatures.md#creature-shadow), [poltergeists](creatures.md#creature-poltergeist),
  [corpse candles](creatures.md#creature-corpse-candle), and wraiths of
  [Fennmynster](dungeon-guide.md#fennmynster), and the [knockers](creatures.md#creature-knocker)
  and [spectres](creatures.md#creature-spectre) of [Fyrdelf](dungeon-guide.md#fyrdelf);
- far deeper, [banshees](creatures.md#creature-banshee),
  [dread wraiths](creatures.md#creature-dread-wraith),
  [greater shadows](creatures.md#creature-greater-shadow),
  [white ladies](creatures.md#creature-white-lady), [wraith lords](creatures.md#creature-wraith-lord),
  the [sluagh](creatures.md#creature-sluagh),
  [washers at the ford](creatures.md#creature-washer-at-the-ford),
  [night mares](creatures.md#creature-night-mare), and [ghost lords](creatures.md#creature-ghost-lord);
- the [earth elementals](creatures.md#creature-earth-elemental), whose element is the stone.

They cannot leave the level through the stone, and [lava](terrain.md#terrain-lava) stops them as
it stops anyone. Nothing of them presses down, so they set off no [traps](traps.md) and never break
[ice](temperature.md#walking-on-ice).

One inside a wall is still there to fight. You see it when you see that piece of wall: when it is
[lit](light.md#what-light-is), or close enough for [darkvision](vision.md#the-four-types). If you
cannot see it there, the wall is only a wall.

- Move into the wall where you see it, and you strike at it.
- It strikes out of the wall at whoever stands beside it.
- An [arrow](ammunition.md) that hits it is lost in the stone. One that misses falls before the
  wall.

The stone shelters it, though. Inside a wall it is harder to [hit](combat.md#hitting): it uses
its **Wall** [Defence](combat.md#defence), in the Defence table on
[Creatures in numbers](creature-numbers.md). Some misses just strike stone ("Your sword strikes
stone where the stone haunt hides in the wall."). When one dies inside a wall, what it carried
falls out onto the nearest floor.

## Elementals

**Elementals** are fire, earth, air, water, ice, and lightning given a shape and a will. Each
element comes in sizes and ages: the **mote** (small, `e`), then the lesser and the plain
elemental, then the greater and the elder (`E`). The eldest of each has an old name: the
**[balefire](creatures.md#creature-balefire)**, the **[hoarstone](creatures.md#creature-hoarstone)**,
the **[welkin wind](creatures.md#creature-welkin-wind)**, the
**[brimwolf](creatures.md#creature-brimwolf)**, the **[rimethurse](creatures.md#creature-rimethurse)**,
and the **[thunderhead](creatures.md#creature-thunderhead)**.

All elementals are hostile to you and leave everyone else alone.
[Poison](damage-types.md#damage-poison) cannot hurt them. Nothing is left of one when it is
destroyed, though the oldest now and then leave something they had caught up.

- **Fire** burns what it touches and lights its way. [Fire](damage-types.md#damage-fire) cannot
  hurt it, and [cold](damage-types.md#damage-cold) hurts it more.
- **Earth** is slow, heavy, and hard to cut or pierce.
  [Electricity](damage-types.md#damage-electricity) cannot hurt it. Its blows may
  [stagger](status-effects.md#status-staggered) you, and the eldest also
  [grips and holds](status-effects.md#status-held). From the earth elemental up, it moves through
  stone as an [incorporeal](#incorporeal-creatures) creature does, though the stone shelters it
  less.
- **Air** is very [fast](turns.md#speed) and hard to hit. [Arrows](ammunition.md) blow aside from
  it, and its blows may [stagger](status-effects.md#status-staggered) you.
- **Water** walks on the land. [Fire](damage-types.md#damage-fire) and arrows do it less harm,
  [electricity](damage-types.md#damage-electricity) more. Its blows may leave you
  [sluggish](status-effects.md#status-sluggish), and the greater ones may
  [hold](status-effects.md#status-held) you.
- **Ice** is [slow](turns.md#speed). Its cold blows may leave you
  [sluggish](status-effects.md#status-sluggish). [Cold](damage-types.md#damage-cold) cannot hurt
  it, and [fire](damage-types.md#damage-fire) hurts it more.
- **Lightning** is [fast](turns.md#speed) and lit blue.
  [Electricity](damage-types.md#damage-electricity) cannot hurt it, and its shocks may
  [stagger](status-effects.md#status-staggered) you.

The exact numbers are on [Creatures in numbers](creature-numbers.md).

# Creatures

Creatures are the other glyphs that move about the world. They belong to [factions](creatures.md#factions), keep to their
routines, notice what they see and hear, and some of them fight, to the death (see
[Combat](combat.md)). This page will grow as the game does. Their [stats](stats.md), [hit points](combat.md#hit-points), attacks,
[speed](turns.md#speed), and senses, which the game never shows, are on [Creatures in numbers](creature-numbers.md).

## Factions

Every creature belongs to a **faction**, and so do you. Two factions are either **hostile** to
each other or **neutral**, both ways; a faction is neutral to itself. Every monster is your
enemy, but monsters do not fight each other, except for the fey courts' war and the demons' war
with the devils: a [kobold](creatures.md#creature-kobold) and a [giant rat](creatures.md#creature-rat) on the same floor leave each other alone, and so do an
orc band and the dead. Wildlife leaves everyone alone, and everyone leaves it alone.

{{#include generated/factions.md}}

**Animals** (the factions marked so above) leave a character with the **[Beast-friend](talents.md#talent-beast-friend)**
[talent](talents.md) alone, and are left alone by them, until that character attacks one: from
then on creatures of that type (all [giant rats](creatures.md#creature-rat), say) stand toward them as their faction does.
A fiend in a beast's shape, the [hell hound](creatures.md#creature-hell-hound) or the [nightmare](creatures.md#creature-nightmare), is no animal: a [Beast-friend](talents.md#talent-beast-friend) calms
neither.

**The fey courts.** Each fey creature ([sprites](creatures.md#creature-sprite), [pixies](creatures.md#creature-pixie), [redcaps](creatures.md#creature-redcap), [dryads](creatures.md#creature-dryad), and the rest) belongs to
one of two courts, chosen at random when it comes into the world, and keeps it: the **Seelie
court** leaves you alone, the **Unseelie court** hunts you, and the two courts fight each other
on sight. Some kinds lean one way: nearly every [redcap](creatures.md#creature-redcap) is Unseelie, nearly every [dryad](creatures.md#creature-dryad) Seelie.
The odds for each kind are in the "Faction" column of the table at the bottom of this page. A
Seelie fey you attack turns on you for good: it becomes one of the **wrathful fey**, hostile to
you and still at war with the Unseelie. Both courts leave every other faction alone.

You can tell how a creature stands toward you from what it is: look at it
([look mode](controls.md#look-mode)) and a hostile one is marked, as in "A giant rat (hostile,
unhurt), on stone floor." A fey shows its court the same way: an Unseelie one is marked hostile,
a Seelie one neutral.

## What they do

When nothing is about, each creature goes about its **routine**: it walks a fixed, looping patrol
route, one step at a time, or stays where it is.

A creature that **[sees](sight.md#line-of-sight)** something hostile to it (you, or a creature of a hostile faction) goes
for the nearest one:

- Creatures that fight **up close** come at it along the shortest way they can find, and attack
  once they stand next to it.
- Creatures that **[shoot](ranged.md#shooting-and-throwing)** or **[cast spells](magic.md#the-spells)** need to be close enough and to have a clear line:
  nothing solid, and nobody, in between. If they have one, they attack from where they are;
  otherwise they move to the nearest place that gives them one. An archer spends an [arrow](ammunition.md) a
  shot, and with none left (or no bow) it comes to fight up close ([Ranged combat](ranged.md)).
- When it loses sight of its target, it goes to where it last saw it (or next to that spot, if
  someone it cannot see stands there). If it finds no one there, or cannot get there, it gives up.
- When it **[hears](hearing.md#what-makes-noise)** something it cannot see, it goes to have a look, if it has enemies at all
  (wildlife does not bother). See [Hearing](hearing.md).
- With nothing left to chase, it walks back to where it left its routine and carries on.

Creatures behave the same way every time in the same situation: there is no chance in it, but
for whether you [slip past](skills.md#slipping-past) one in the dark. They
see by the same rules you do (see [Sight and memory](sight.md)), each with its own eyes and under
the same [light](light.md#what-light-is), clouds included: a creature without [darkvision](vision.md#the-four-types) does not see you standing in the
dark, and lighting your [lantern](lantern.md) gets you noticed. Some creatures open closed [doors](doors.md#creatures-and-doors)
in their way, which takes them a turn and makes [noise](hearing.md#what-makes-noise); the others cannot get past a closed door.
A [locked door](doors.md#locked-doors) stops even those, unless they carry its [key](doors.md#keys) (see [Doors](doors.md)).

Other creatures block the way: a creature walks a little way round someone in its way, or waits
behind them.

## Attacks

Blows and shots are rolled, hit or miss, hurt, and kill, by the rules of [Combat](combat.md):
"The kobold hits you.", "The giant rat bites the kobold hard!", "You block the kobold's spear
with your shield.", "The kobold archer shoots the giant rat.", "The giant rat dies." Spells are
only announced: "The kobold shaman casts a spell at you." You only hear of attacks you perceive:
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
the [hill troll](creatures.md#creature-hill-troll) of [Fyrdelf](dungeons.md#fyrdelf), and the [forest](creatures.md#creature-forest-troll), [ice](creatures.md#creature-ice-troll), [stone](creatures.md#creature-stone-troll), [two-headed](creatures.md#creature-two-headed-troll), and [fire trolls](creatures.md#creature-fire-troll), the [troll
shaman](creatures.md#creature-troll-shaman), the [troll chieftain](creatures.md#creature-troll-chieftain), and the [elder troll](creatures.md#creature-elder-troll) further on. Every creature's [hit points](combat.md#hit-points) come
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
[poltergeists](creatures.md#creature-poltergeist), [corpse candles](creatures.md#creature-corpse-candle), and wraiths of [Fennmynster](dungeons.md#fennmynster), the [knockers](creatures.md#creature-knocker) and [spectres](creatures.md#creature-spectre) of [Fyrdelf](dungeons.md#fyrdelf),
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
  through stone as an [incorporeal](creatures.md#incorporeal-creatures) creature does (above), though the stone shelters it less.
- **Air** is very [fast](turns.md#speed) and hard to hit, [arrows](ammunition.md) blow aside from it, and its blows may [stagger](status-effects.md#status-staggered) you.
- **Water** walks on the land; [fire](damage-types.md#damage-fire) and arrows do it less harm, [electricity](damage-types.md#damage-electricity) more. Its blows may
  leave you [sluggish](status-effects.md#status-sluggish), and the greater ones may [hold](status-effects.md#status-held) you.
- **Ice** is [slow](turns.md#speed), its cold blows may leave you [sluggish](status-effects.md#status-sluggish), [cold](damage-types.md#damage-cold) cannot hurt it, and [fire](damage-types.md#damage-fire) hurts it
  more.
- **Lightning** is [fast](turns.md#speed), lit blue, cannot be hurt by [electricity](damage-types.md#damage-electricity), and its shocks may [stagger](status-effects.md#status-staggered) you.

The exact numbers are on [Creatures in numbers](creature-numbers.md).

## Pace and levels

- Creatures move at their **[own pace](turns.md#speed)**. Some are quicker than you and may act more than once
  while you take one turn; some are slower and now and then let a turn of yours pass without
  acting. A creature of your own pace acts once for each normal action of yours. The game never
  tells you a creature's pace: you learn it by watching it move (or from its speed in
  [Creatures in numbers](creature-numbers.md#pace-senses-and-size)).
- Their actions have [costs](turns.md#what-actions-cost) like yours: a creature's step through
  water takes it twice as long as a step on open ground, and a creature type's attacks may cost
  it more or less than a normal action.
- How often a creature acts follows game time, not your turns: while you wade through [water](terrain.md#water) or
  [rest](resting.md), creatures get to act for as long as that took you.
- Creatures only act on the level you are on. On other levels they stand still until you come
  back, and then carry on where they left off: they do not catch up on the time you were away.
- You only see a creature while you see its cell, so creatures in the dark are hidden unless they
  carry a [light](light.md#light-sources). You never remember where one was.
- Creatures make [noise](hearing.md#what-makes-noise) as they walk ([bats](creatures.md#creature-bat) fly silently), so you may hear one you cannot see, and
  they [hear](hearing.md#what-makes-noise) your steps, doors, and attacks.
- Some creatures carry [objects](objects.md), and [wear or wield](equipment.md#what-fits-where) some of them: a [kobold](creatures.md#creature-kobold) may carry a
  spear, a [kobold archer](creatures.md#creature-kobold-archer) a bow and arrows. You see what a
  creature wears and wields ([look mode](controls.md#look-mode) says it), never what else it
  carries. They fight with it ([Equipment](equipment.md)), and drop all of it when they die.
- Creatures you meet in the dungeons (and those a place puts somewhere without a set kit) come
  with gear of their own, by their kind's table ("Gear" below): each line in turn by its
  chance, one of what it lists, and now and then a [magic](magic-items.md) piece (made as loot is, at the floor's
  item level, and [unidentified](magic-items.md#unidentified-items)). A kind's **racial weapon** is the first line, 90% unless the
  table says otherwise, and every kind with one is trained in its [weapon skill](weapon-skills.md). What a creature
  could not carry (over its [carrying limit](burden.md#how-much-you-can-carry)) it goes without, and what it gets is made of a
  material by the creature's level, as the drops are. The **[Scavenger](talents.md#talent-scavenger)** talent identifies what a
  creature you kill drops as it lands.

## The creatures

"Drops magic" is the chance that a creature drops a [magic or rare](magic-items.md#rarity) item where it dies, besides
what it carried (see [Magic items](magic-items.md)), at the creature's level as its [item level](magic-items.md#base-items),
or the floor's where that is higher. Their numbers are on
[Creatures in numbers](creature-numbers.md).

Some of the dead below are known to no dungeon yet, waiting in places deeper than any found so
far: [grave knights](creatures.md#creature-grave-knight) and [headless knights](creatures.md#creature-headless-knight) in their armour, [bodachs](creatures.md#creature-bodach), [banshees](creatures.md#creature-banshee), [wight lords](creatures.md#creature-wight-lord), [dread
wraiths](creatures.md#creature-dread-wraith), and [vampires](creatures.md#creature-vampire) that drink your life, the [charnel heaps](creatures.md#creature-charnel-heap) and [wormhosts](creatures.md#creature-wormhost), the [bones of drakes](creatures.md#creature-bone-drake)
and [drakes that rose](creatures.md#creature-rotting-drake) with their flesh still on, and the [lich](creatures.md#creature-lich), a sorcerer who would not die. Deeper still wait the lords of their kinds: [ghoul
lords](creatures.md#creature-ghoul-lord), [draugr lords](creatures.md#creature-draugr-lord), and [mummy lords](creatures.md#creature-mummy-lord), the plague-bearing [nosferatu](creatures.md#creature-nosferatu), [greater shadows](creatures.md#creature-greater-shadow) and [white
ladies](creatures.md#creature-white-lady), the black dogs of the churchyards and the shore lanes ([black shucks](creatures.md#creature-black-shuck) and [church grims](creatures.md#creature-church-grim)),
[screaming skulls](creatures.md#creature-screaming-skull), and the [bone colossus](creatures.md#creature-bone-colossus), a charnel house of bones stood up on its own. Deepest of all wait the kings of the dead: [barrow kings](creatures.md#creature-barrow-king), [ghoul kings](creatures.md#creature-ghoul-king), [vampire lords](creatures.md#creature-vampire-lord), [death knights](creatures.md#creature-death-knight) and [nightgangers](creatures.md#creature-nightganger), [wormlords](creatures.md#creature-wormlord), the [drowned king](creatures.md#creature-drowned-king), the [bone](creatures.md#creature-bone-dragon) and [rotting dragons](creatures.md#creature-rotting-dragon), the [archlich](creatures.md#creature-archlich), and the [hounds of the Wild Hunt](creatures.md#creature-hound-of-the-hunt).

At the very bottom of the roster wait the last of the dead, in places deeper than any found so
far: the [mummy king](creatures.md#creature-mummy-king) and the [wild huntsman](creatures.md#creature-wild-huntsman) of the old tales, [elder](creatures.md#creature-elder-vampire) and [eldest vampires](creatures.md#creature-eldest-vampire), the [dread
lord](creatures.md#creature-dread-lord) of the death knights, the [deathshade](creatures.md#creature-deathshade) (Old English *dēaþscūa*), the [ankou](creatures.md#creature-ankou) of Breton tales, and
the [ghost of a dragon](creatures.md#creature-ghost-dragon), the [nightwing](creatures.md#creature-nightwing) and the [nightworm](creatures.md#creature-nightworm), the [dread wyrm](creatures.md#creature-dread-wyrm), the [lich-drake](creatures.md#creature-lich-drake), and the
[charnel titan](creatures.md#creature-charnel-titan), the [deathless king](creatures.md#creature-deathless-king), and last the [demilich](creatures.md#creature-demilich), of which a world holds no more than one.
Their touch and bite take two points of a [stat](stats.md#drained-stats) for good (three for the eldest vampire and the
demilich) unless the check holds; a temple's [mending](npcs.md#services) gives them back.

**The outlaws** of the Ascmearc are people (`@`) hostile to you alone: no faction fights them,
so they hide among whatever lives in the old places they have gone to ground in. **[Footpads](creatures.md#creature-footpad)**
carry short swords, **[footpad archers](creatures.md#creature-footpad-archer)** short bows. Above them are four wanted criminals, each
one of a kind and met at most once: **[Mother Sallow](creatures.md#creature-sallow)**, the Fennhām Poisoner, quick and quiet,
whom [poison](damage-types.md#damage-poison) cannot touch, with the stiletto Widowmaker; **[Black Wystan](creatures.md#creature-wystan)**, the Gentleman of the
Road, an archer with the long bow High Toby, forty [arrows](ammunition.md), and a short sword; **[Brother
Osrick](creatures.md#creature-osrick)**, the Resurrection Man, with a mace, who [sees in the dark](vision.md#the-four-types) through the Unhalwed Bedes and
shrugs off half of any [cold](damage-types.md#damage-cold) or [poison](damage-types.md#damage-poison); and **[Sergeant Herrick](creatures.md#creature-herrick)**, the Deserter, with a great axe
(a [magic](magic-items.md) one half the time) and the Turncoat, the mail that keeps fear from him. Each always
carries their own [unique item](magic-items.md), and drops it where they die. Named monsters
go by their names in what the game tells you ("Black Wystan shoots you."), with no "the".
Each waits at the bottom of a lair of their own, which the [Weyfarers' Gilde](guild.md) sends you to (see
[The wanted](guild.md#the-wanted)); the footpads run with them there.

**Demons** come up from below: the lesser ones (`u`) from the [dretch](creatures.md#creature-dretch) to the [chasme](creatures.md#creature-chasme), the greater
(`U`) from the [hezrou](creatures.md#creature-hezrou) to the [balor](creatures.md#creature-balor). None of them leaves a body behind, [poison](damage-types.md#damage-poison) does nothing to
them, and [fire](damage-types.md#damage-fire) does them less harm than it should (the balor none at all). Only the [dretch](creatures.md#creature-dretch) and
the [quasit](creatures.md#creature-quasit) are met in the [delf](dungeons.md) so far; the rest wait in deeper places.

{{#include generated/creatures.md}}

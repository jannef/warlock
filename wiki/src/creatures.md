# Creatures

Creatures are the other glyphs that move about the world. They belong to [factions](creatures.md#factions), keep to their
routines, notice what they see and hear, and some of them fight, to the death (see
[Combat](combat.md)). This page will grow as the game does. Their [stats](stats.md), [hit points](hit-points.md#hit-points), attacks,
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
- **Dragons** breathe their element over a long cone whenever their breath is ready and you
  stand within its reach with a clear line, and between breaths come at you to fight up close
  with claws and bite (see [Creatures in numbers](creature-powers.md#breath)).
- A few creatures **lurk**: the [meenlock](creatures.md#creature-meenlock) and the
  [barrow spider](creatures.md#creature-barrow-spider). One that sees its target does not come at
  it: it waits where it stands, well aware of it, and attacks once the target comes beside it (or
  shoots or casts from where it stands, if it can). Once it is hurt, it hunts like any other.
  The table in [Creatures in numbers](creature-numbers.md#surprise) says which they are.
- When it loses sight of its target, it goes to where it last saw it (or next to that spot, if
  someone it cannot see stands there). If it finds no one there, or cannot get there, it gives up.
- When it **[hears](hearing.md#what-makes-noise)** something it cannot see, it goes to have a look, if it has enemies at all
  (wildlife does not bother). See [Hearing](hearing.md).
- With nothing left to chase, it walks back to where it left its routine and carries on.
- Beside you when you take [stairs](terrain.md#stairs), it comes along.

Creatures behave the same way every time in the same situation: there is no chance in it, but
for whether you [slip past](skill-uses.md#slipping-past) one in the dark. They
see by the same rules you do (see [Sight and memory](sight.md)), each with its own eyes and under
the same [light](light.md#what-light-is), clouds included: a creature without [darkvision](vision.md#the-four-types) does not see you standing in the
dark, and lighting your [lantern](lantern.md) gets you noticed. Some creatures open closed [doors](doors.md#creatures-and-doors)
in their way, which takes them a turn and makes [noise](hearing.md#what-makes-noise); the others cannot get past a closed door.
A [locked door](doors.md#locked-doors) stops even those, unless they carry its [key](doors.md#keys) (see [Doors](doors.md)).

Other creatures block the way: a creature walks a little way round someone in its way, or waits
behind them.

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
  material by the creature's level, as the drops are. What falls is
  [unidentified](magic-items.md#unidentified-items), as any find is.

## Grades

Two of the same creature need not be the same fight. As a creature comes into being, chance
makes one in six of them **greater** than its kind and one in six **lesser**; the rest are
ordinary. A greater one has more [hit points](hit-points.md#hit-points), hits more surely and harder up close,
and moves a little more [slowly](turns.md#speed); a lesser one is the reverse. [Look
mode](controls.md#look-mode) tells you which, with a word before its name: "a big
[kobold](creatures.md#creature-kobold)", "a scrawny kobold", "an ancient zombie"; messages call
it by its kind alone. Killing one is worth what its kind is worth in
[experience](experience.md), whatever its grade. Unique creatures, the peaceful and trading folk,
[animated objects](animated.md) but the blades ("a notched animated blade", "a keen" one), and
[mimics](mimics.md) are never graded, and neither is
your own [companion](companions.md#companions) or [familiar](familiars.md#familiars): they [grow](companions.md#growth) with you instead.

{{#include generated/creature-grades.md}}

The words for each:

{{#include generated/creature-grade-words.md}}

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

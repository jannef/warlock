# Creatures

Creatures are the other glyphs that move about the world. Each belongs to a
[faction](creatures.md#factions), keeps to a routine, and notices what it sees and hears. Some of
them fight, to the death (see [Combat](combat.md)).

More about creatures:

- [How creatures fight](creature-bodies.md): their attacks, hides and bodies, and the
  incorporeal and elemental kinds.
- [Creatures in numbers](creature-numbers.md): their [stats](stats.md),
  [hit points](hit-points.md#hit-points), attacks, [speed](turns.md#speed), and senses, which the
  game never shows.

## Factions

Every creature belongs to a **faction**, and so do you. Two factions are either **hostile** to
each other or **neutral**, both ways round. A faction is neutral to itself.

Every monster is your enemy, but monsters do not fight each other. A
[kobold](creatures.md#creature-kobold) and a [giant rat](creatures.md#creature-rat) on the same
floor leave each other alone, and so do an orc band and the dead. There are two exceptions: the
fey courts are at war, and so are the demons and the devils. Wildlife leaves everyone alone, and
everyone leaves it alone.

{{#include generated/factions.md}}

**Animals.** The factions marked as animals above leave a character with the
[Beast-friend](talents.md#talent-beast-friend) [talent](talents.md) alone, and that character
leaves them alone too. This lasts until the character attacks one. From then on, creatures of that
type (all [giant rats](creatures.md#creature-rat), say) treat them as their faction normally
would. A fiend in a beast's shape, such as the [hell hound](creatures.md#creature-hell-hound) or
the [nightmare](creatures.md#creature-nightmare), is not an animal, and
[Beast-friend](talents.md#talent-beast-friend) does not calm it.

**The fey courts.** Each fey creature ([sprites](creatures.md#creature-sprite),
[pixies](creatures.md#creature-pixie), [redcaps](creatures.md#creature-redcap),
[dryads](creatures.md#creature-dryad), and the rest) belongs to one of two courts. The court is
picked at random when the creature comes into the world, and it never changes.

- The **Seelie court** leaves you alone.
- The **Unseelie court** hunts you.

The two courts fight each other on sight, and both leave every other faction alone. Some kinds
lean one way: nearly every [redcap](creatures.md#creature-redcap) is Unseelie, and nearly every
[dryad](creatures.md#creature-dryad) is Seelie. The odds for each kind are in the "Faction" column
of the table at the bottom of this page.

If you attack a Seelie fey, it turns on you for good. It becomes one of the **wrathful fey**:
hostile to you, and still at war with the Unseelie.

To tell how a creature stands toward you, look at it ([look mode](controls.md#look-mode)). A
hostile one is marked, as in "A giant rat (hostile, unhurt), on stone floor." A fey shows its
court the same way: an Unseelie one is marked hostile, a Seelie one neutral.

## What they do

When nothing is about, a creature follows its **routine**. It either walks a fixed patrol route
in a loop, one step at a time, or stays where it is.

When a creature **[sees](sight.md#line-of-sight)** something hostile to it (you, or a creature of
a hostile faction), it goes for the nearest one:

- Creatures that fight **up close** come at it by the shortest way they can find, and attack once
  they stand next to it.
- Creatures that **[shoot](ranged.md#shooting-and-throwing)** or
  **[cast spells](magic.md#the-spells)** need to be close enough, with a clear line: nothing solid
  and nobody in between. If they have one, they attack from where they are. If not, they move to
  the nearest place that gives them one. An archer spends one [arrow](ammunition.md) a shot. With
  no arrows left, or no bow, it comes to fight up close ([Ranged combat](ranged.md)).
- **Dragons** breathe their element over a long cone whenever their breath is ready and you stand
  within reach with a clear line. Between breaths they come at you with claws and bite (see
  [Breath](creature-powers.md#breath)).
- A few creatures **lurk**: the [meenlock](creatures.md#creature-meenlock) and the
  [barrow spider](creatures.md#creature-barrow-spider). A lurker that sees its target does not
  come at it. It waits where it stands, well aware of the target, and attacks once the target
  comes beside it. If it can shoot or cast, it does so from where it stands. Once it is hurt, it
  hunts like any other creature. The table in
  [Creatures in numbers](creature-numbers.md#surprise) says which creatures lurk.
- A **rooted** creature never leaves the cell it stands on. It strikes whatever comes beside it,
  unless it has nothing to strike with, and shoots, casts, or
  [calls others](creature-powers.md#summoning) from where it is. Nothing
  moves it: it never [makes way](npcs.md#getting-past) for you, no blow or spell drives it back,
  no teleport takes it, and it never follows you on the [stairs](terrain.md#stairs).
- When it loses sight of its target, it goes to where it last saw it. If someone it cannot see
  stands on that spot, it goes next to it instead. If it finds no one there, or cannot get there,
  it gives up.
- When it **[hears](hearing.md#what-makes-noise)** something it cannot see, it goes to have a
  look, as long as it has enemies at all. Wildlife does not bother. See [Hearing](hearing.md).
- With nothing left to chase, it walks back to where it left its routine and carries on.
- If it is beside you when you take the [stairs](terrain.md#stairs), it comes along.

Creatures always behave the same way in the same situation. There is no chance in it, except
whether you [slip past](skill-uses.md#slipping-past) one in the dark.

They see by the same rules you do (see [Sight and memory](sight.md)). Each has its own eyes and
sees by the same [light](light.md#what-light-is), clouds included. A creature without
[darkvision](vision.md#the-four-types) does not see you standing in the dark, and lighting your
[lantern](lantern.md) gets you noticed.

Some creatures open closed [doors](doors.md#creatures-and-doors) in their way. That takes them a
turn and makes [noise](hearing.md#what-makes-noise). The others cannot get past a closed door. A
[locked door](doors.md#locked-doors) stops even the door-openers, unless they carry its
[key](doors.md#keys) (see [Doors](doors.md)).

Other creatures get in the way. A creature walks a little way round someone blocking it, or waits
behind them.

## Pace and levels

- Creatures move at their **[own pace](turns.md#speed)**. Some are quicker than you and may act
  more than once while you take one turn. Some are slower and now and then let one of your turns
  pass without acting. A creature of your own pace acts once for each normal action of yours. The
  game never tells you a creature's pace. You learn it by watching it move, or from its speed in
  [Creatures in numbers](creature-numbers.md#pace-senses-and-size).
- Their actions [cost](turns.md#what-actions-cost) time like yours. A step through water takes a
  creature twice as long as a step on open ground, and some creatures' attacks cost more or less
  than a normal action.
- How often a creature acts follows game time, not your turns. While you wade through
  [water](terrain.md#water) or [rest](resting.md), creatures act for as long as that took you.
- Creatures act only on the level you are on. On other levels they stand still until you come
  back, then carry on where they left off. They do not catch up on the time you were away.
- You see a creature only while you see its cell, so creatures in the dark are hidden unless they
  carry a [light](light.md#light-sources). You never remember where one was.
- Creatures make [noise](hearing.md#what-makes-noise) as they walk, so you may hear one you cannot
  see. [Bats](creatures.md#creature-bat) fly silently. Creatures
  [hear](hearing.md#what-makes-noise) your steps, doors, and attacks too.
- Some creatures carry [objects](objects.md) and [wear or wield](equipment.md#what-fits-where)
  some of them. A [kobold](creatures.md#creature-kobold) may carry a spear, a
  [kobold archer](creatures.md#creature-kobold-archer) a bow and arrows. You see what a creature
  wears and wields ([look mode](controls.md#look-mode) tells you), but never what else it carries.
  It fights with its gear ([Equipment](equipment.md)) and drops all of it when it dies.

### Gear

Creatures you meet in the dungeons come with gear of their own, and so do creatures that a place
puts somewhere without a set kit. The gear follows the kind's table ("Gear" below). Each line of
the table is rolled in turn by its chance, and gives one of the things it lists. Now and then a
creature also gets a [magic](magic-items.md) piece, made as loot is, at the floor's item level.

- A kind's **racial weapon** is its first line. It comes 90% of the time unless the table says
  otherwise, and every kind with one is trained in its [weapon skill](weapon-skills.md).
- An armed creature gets no flat bonus to hit. Its [ranks](weapon-skills.md#marks-and-ranks) in
  the weapon's skill (and in [Duelist](fighting-styles.md#style-duelist), for a one-handed weapon
  with no shield) are the fewest that bring the weapon to a quarter of its level to hit (a
  spellcaster's, to an eighth), as the creatures that fight with claws and teeth hit. No rank is
  higher than a trained warrior of the creature's level holds (2 at level 1, 7 at 15 to 22, 10
  from 42), so a heavy weapon (great blades, axes, polearms) may stay below and hit harder
  instead.
- Its ranks bring what ranks bring anyone: damage, speed, [Parry](combat.md#defence), and the
  abilities they teach that work by themselves
  ([Deep cut](weapon-skills.md#weapon-ability-deep-cut),
  [Crushing blow](weapon-skills.md#weapon-ability-crushing-blow),
  [Brace](weapon-skills.md#weapon-ability-brace),
  [Riposte](fighting-styles.md#style-ability-riposte)).
- What a creature could not carry (over its [carrying limit](burden.md#how-much-you-can-carry))
  it goes without.
- What it gets is made of a material by the creature's level, as drops are.
- What falls when it dies is [unidentified](magic-items.md#unidentified-items), as any find is.

#### Condition

Each piece of arms or armour a creature gets is often worn out and now and then finer than most.
It may be **damaged**, a point worse than a plain one, or **Fine**, **Marvellous**, or
**Masterwork**, with one, two, or three improvements in what an improvement can better, as if
worked at a [forge](altars-forges-springs.md#forges) (each counts as one attempt made on it). A
damaged piece is called by what wore it out: *rusted* iron and steel, *corroded* bronze, silver,
and other metals, and plainly *damaged* wood, leather, and cloth. The creature fights with it as
it is, and it falls as it is. A stack of several (thrown weapons, ammunition), rings and amulets,
a [unique](magic-items.md#unique-items) item, one [of Proof](materials.md#protected-items), anything a [quest](quests.md) may ask for, and a set
kit a place gives a creature are always as made. The chances, by the creature's
[tag](quests.md#sorts-of-creatures):

{{#include generated/gear-condition.md}}

## Grades

Two of the same creature need not be the same fight. When a creature comes into being, there is a
one in six chance it is **greater** than its kind and a one in six chance it is **lesser**. The
rest are ordinary.

A greater creature has more [hit points](hit-points.md#hit-points), hits more surely and harder up
close, and moves a little more [slowly](turns.md#speed). A lesser one is the reverse.

[Look mode](controls.md#look-mode) tells you the grade with a word before the name: "a big
[kobold](creatures.md#creature-kobold)", "a scrawny kobold", "an ancient zombie". Messages call it
by its kind alone. Killing one is worth what its kind is worth in [experience](experience.md),
whatever its grade.

Some creatures are never graded:

- unique creatures;
- the peaceful and trading folk;
- [animated objects](animated.md), except the blades ("a notched animated blade", "a keen" one);
- [mimics](mimics.md);
- your own [companion](companions.md#companions) or [familiar](familiars.md#familiars), which
  [grow](companions.md#growth) with you instead.

{{#include generated/creature-grades.md}}

The words for each:

{{#include generated/creature-grade-words.md}}

## The creatures

"Drops magic" in the table below is the chance that a creature drops a
[magic or rare](magic-items.md#rarity) item where it dies, besides what it carried (see
[Magic items](magic-items.md)). The item's [item level](affixes.md#base-items) is the
creature's level, or the floor's if that is higher. The creatures' numbers are on
[Creatures in numbers](creature-numbers.md).

**The dead still to come.** Some of the dead below live in no dungeon yet. They wait in places
deeper than any found so far:

- [grave knights](creatures.md#creature-grave-knight) and
  [headless knights](creatures.md#creature-headless-knight) in their armour,
  [bodachs](creatures.md#creature-bodach), [banshees](creatures.md#creature-banshee),
  [wight lords](creatures.md#creature-wight-lord), [dread wraiths](creatures.md#creature-dread-wraith),
  and [vampires](creatures.md#creature-vampire) that drink your life, the
  [charnel heaps](creatures.md#creature-charnel-heap) and [wormhosts](creatures.md#creature-wormhost),
  the [bones of drakes](creatures.md#creature-bone-drake) and
  [drakes that rose](creatures.md#creature-rotting-drake) with their flesh still on, and the
  [lich](creatures.md#creature-lich), a sorcerer who would not die.
- Deeper still, the lords of their kinds: [ghoul lords](creatures.md#creature-ghoul-lord),
  [draugr lords](creatures.md#creature-draugr-lord), and
  [mummy lords](creatures.md#creature-mummy-lord), the plague-bearing
  [nosferatu](creatures.md#creature-nosferatu), [greater shadows](creatures.md#creature-greater-shadow)
  and [white ladies](creatures.md#creature-white-lady), the black dogs of the churchyards and the
  shore lanes ([black shucks](creatures.md#creature-black-shuck) and
  [church grims](creatures.md#creature-church-grim)),
  [screaming skulls](creatures.md#creature-screaming-skull), and the
  [bone colossus](creatures.md#creature-bone-colossus), a charnel house of bones stood up on its
  own.
- Deepest of all, the kings of the dead: [barrow kings](creatures.md#creature-barrow-king),
  [ghoul kings](creatures.md#creature-ghoul-king), [vampire lords](creatures.md#creature-vampire-lord),
  [death knights](creatures.md#creature-death-knight) and
  [nightgangers](creatures.md#creature-nightganger), [wormlords](creatures.md#creature-wormlord),
  the [drowned king](creatures.md#creature-drowned-king), the [bone](creatures.md#creature-bone-dragon)
  and [rotting dragons](creatures.md#creature-rotting-dragon), the
  [archlich](creatures.md#creature-archlich), and the
  [hounds of the Wild Hunt](creatures.md#creature-hound-of-the-hunt).
- At the very bottom, the last of the dead: the [mummy king](creatures.md#creature-mummy-king)
  and the [wild huntsman](creatures.md#creature-wild-huntsman) of the old tales,
  [elder](creatures.md#creature-elder-vampire) and [eldest vampires](creatures.md#creature-eldest-vampire),
  the [dread lord](creatures.md#creature-dread-lord) of the death knights, the
  [deathshade](creatures.md#creature-deathshade) (Old English *dēaþscūa*), the
  [ankou](creatures.md#creature-ankou) of Breton tales, the
  [ghost of a dragon](creatures.md#creature-ghost-dragon), the
  [nightwing](creatures.md#creature-nightwing) and the [nightworm](creatures.md#creature-nightworm),
  the [dread wyrm](creatures.md#creature-dread-wyrm), the [lich-drake](creatures.md#creature-lich-drake),
  the [charnel titan](creatures.md#creature-charnel-titan), the
  [deathless king](creatures.md#creature-deathless-king), and last the
  [demilich](creatures.md#creature-demilich), of which a world holds no more than one.

The touch and bite of these last ones take two points of a [stat](stats.md#drained-stats) for
good unless the check holds. The eldest vampire and the demilich take three. A temple's
[mending](npcs.md#services) gives the points back.

**The outlaws** of the Ascmearc are people (`@`) hostile to you alone. No faction fights them, so
they hide among whatever lives in the old places where they have gone to ground.
**[Footpads](creatures.md#creature-footpad)** carry short swords, and
**[footpad archers](creatures.md#creature-footpad-archer)** short bows.

Above them are four wanted criminals. Each is one of a kind and met at most once:

- **[Mother Sallow](creatures.md#creature-sallow)**, the Fennhām Poisoner: quick and quiet, and
  [poison](damage-types.md#damage-poison) cannot touch her. She carries the stiletto Widowmaker.
- **[Black Wystan](creatures.md#creature-wystan)**, the Gentleman of the Road: an archer with the
  long bow High Toby, forty [arrows](ammunition.md), and a short sword.
- **[Brother Osrick](creatures.md#creature-osrick)**, the Resurrection Man: he fights with a mace,
  [sees in the dark](vision.md#the-four-types) through the Unhalwed Bedes, and shrugs off half of
  any [cold](damage-types.md#damage-cold) or [poison](damage-types.md#damage-poison).
- **[Sergeant Herrick](creatures.md#creature-herrick)**, the Deserter: he carries a great axe (a
  [magic](magic-items.md) one half the time) and wears the Turncoat, the mail that keeps fear from
  him.

Each always carries their own [unique item](magic-items.md) and drops it where they die. Named
monsters go by their names in what the game tells you, with no "the" ("Black Wystan shoots
you."). Each waits at the bottom of a lair of their own, where the
[Weyfarers' Gilde](guild.md) sends you (see [The wanted](guild.md#the-wanted)). The footpads run
with them there.

**Demons** come up from below. The lesser ones (`u`) run from the
[dretch](creatures.md#creature-dretch) to the [chasme](creatures.md#creature-chasme), the greater
ones (`U`) from the [hezrou](creatures.md#creature-hezrou) to the [balor](creatures.md#creature-balor).
None of them leaves a body behind. [Poison](damage-types.md#damage-poison) does nothing to them,
and [fire](damage-types.md#damage-fire) does them less harm than it should (the balor none at
all). So far only the [dretch](creatures.md#creature-dretch) and the
[quasit](creatures.md#creature-quasit) are met in the [delf](dungeons.md); the rest wait in deeper
places.

{{#include generated/creatures.md}}

# Encounters on the way

The [overworld](overworld.md#travel) is not empty between the places on the map. As you travel
you may meet someone on the way: a band of footpads on the road, goblin raiders in the hills, the
dead of the barrows walking at night, a pedlar with his pack, villagers going to market, or a herd
of [deer](creatures.md#creature-deer). Who you may meet depends on where you are, the time of
day, and what has happened in your story so far.

## The chance of meeting someone

Every step onto an overworld cell with a [local area](overworld.md#local-areas) may meet someone;
towns, dungeon entrances, and water are safe. The chance is a rate **per hour of travel**, set by
the land you step onto, times the hours the step takes. A longer step leaves you exposed for
longer.

Roads are the safest ground, open land less safe, and woods, hills, deep forest, and marsh the
least. In [twilight](time.md#days) the chance is half as much again, and at night double. Some
places are busier than others: while a story keeps people on the move there, a region's chance may
rise (see [The Ascmearc's encounters](#the-ascmearcs-encounters)). Travelling faster, with
[Survival](skills.md#skill-survival) and the talents that quicken your journeys, means fewer hours
on the way and so fewer encounters.

{{#include generated/encounter-rates.md}}

"A day's march" is how many encounters twelve hours of daylight on that land bring on average.
There is no limit to how many you meet; the chance is all there is.

## Who sees whom

When you meet someone, two checks decide who noticed whom first. Each is 2d12 plus an edge,
against the number it needs, as in [hitting](combat.md#hitting).

- **Your lookout.** Your edge is your [Willpower](stats.md#stat-willpower)
  [modifier](stats.md#modifiers), plus what [Spot](skills.md#skill-spot) and
  [Survival](skills.md#skill-survival) give the lookout, plus [Alert](talents-exploring.md#talent-alert) and
  the other talents below. You need the difficulty, plus their leader's stalk (what its
  [Stealth](skills.md#skill-stealth) gives), plus the land's cover, plus the
  [rain](weather.md#how-the-weather-changes), plus the dark at night or in twilight. The dark
  counts half as much if you have [darkvision](vision.md#the-four-types).
- **Their lookout.** Their edge is their leader's [Willpower](stats.md#stat-willpower)
  [modifier](stats.md#modifiers) plus what its [Spot](skills.md#skill-spot) gives. They need the
  difficulty, plus your stalk ([Stealth](skills.md#skill-stealth)), plus the cover, plus the rain,
  plus the dark unless the leader sees in the dark. A lit [lantern](lantern.md) at night or in
  twilight takes a good deal off what they need.

What comes of it:

| You saw them | They saw you | What happens |
|:---:|:---:|---|
| yes | no | **You have the drop on them.** "You spot a band of footpads on the road ahead. They have not seen you." You may lie in wait, slip past, or stand and fight. |
| yes | yes | **You meet.** "You and a band of footpads see each other at the same moment." You may give them the slip, or stand and fight. |
| no | yes | **You are ambushed.** "You are set upon by a band of footpads from the roadside!" There is no choice: the fight starts on their terms. |
| no | no | **You pass each other by.** Nothing happens, and you never know they were there. |

{{#include generated/encounter-checks.md}}

How often a check succeeds, by your edge and the number it needs:

{{#include generated/encounter-odds.md}}

What the skills give, by their value:

{{#include generated/encounter-ladders.md}}

Every lookout trains [Spot](skills.md#skill-spot) and [Survival](skills.md#skill-survival). A
party that misses you trains your [Stealth](skills.md#skill-stealth).

## Deciding what to do

When you meet someone, the game asks what you do, and you must answer. Until you choose, you can
do nothing else, except pick what a new level brings or change your
[stance](stances.md#stance-neutral).

- **Lie in wait** (when you have the drop). You take cover for ten minutes and let them walk into
  you. They come on unaware of you, like a creature you
  [slipped past](skill-uses.md#slipping-past), and a step behind, so your first blows are
  [sneak attacks](surprise.md#sneak-attacks).
- **Slip past** (when you have the drop) or **Give them the slip** (when you met). This is a check
  of your [Dexterity](stats.md#stat-dexterity) [modifier](stats.md#modifiers) plus what
  [Survival](skills.md#skill-survival) gives slipping away, plus your stalk if they have not seen
  you, against the difficulty. It is harder when they have seen you, and when their leader is
  quicker than you. The detour takes half the step's time again, whatever comes of it. Made, you
  are on your way ("You slip away unseen.", "You lose them in the woods."). Failed, they are onto
  you, and the fight starts close.
- <a id="talking-past"></a>**Talk your way past** (when you met, if you are trained in
  [Bluff](skills.md#skill-bluff)). Only a few parties would rather hear a story than fight: the
  footpads on the road, in the woods, and in the vale, the duergar prospectors at Ascdūn, and the
  cultists on the fen road. The tables below mark them "talked past".

  Their leader rolls 2d12 plus its [Willpower](stats.md#stat-willpower)
  [modifier](stats.md#modifiers). It must reach the difficulty a [feint](skill-uses.md#feinting)
  asks, plus your [Presence](stats.md#stat-presence) modifier, plus what your talents add to
  Presence checks, plus what your Bluff adds to a feint. If it falls short, "You talk your way
  past.": they let you go, and it took as long as a word with someone. Otherwise "They are not
  fooled.", and the fight starts as if you had stood to fight. Every try trains Bluff.
- **Stand and fight.** You go to meet them in the cell's [local area](overworld.md#local-areas).
- **Hail them** or **Pass by** (travellers who mean you no harm). If you hail them, you meet them
  in the local area, where you can [talk](npcs.md#chatting) to them as to anyone.
- **Hunt them** or **Let them be** (animals). If you hunt, you find them in the local area. A deer
  is good [food](food.md#corpses).

A party you got away from stays where you left it. Meet the same sort of party on that cell again
and it is the same one, and you are told so: "You remember them: you got away from them here
before."

## Where the fight happens

A fight, a hail, or a hunt opens the local area of the cell you stepped onto, on the side you came
from, as `>` would. How far in you stand depends on how the encounter went: the better it went for
you, the nearer you are to the edge you came from.

{{#include generated/encounter-positions.md}}

Ambushed, you are [caught unawares](surprise.md#caught-unawares) unless you keep your wits about
you. To get away, [leave the area](overworld.md#leaving-an-area) as you would any other, which
needs no enemy within three cells of you.

## Camping and foraging

[Resting](resting.md#how-resting-works) on the overworld is not safe either. Each hour of rest has
a quarter of the chance an hour's travel on that land has. Asleep, you keep no watch but your
ears. Your check is your [Willpower](stats.md#stat-willpower) [modifier](stats.md#modifiers) plus
your [hearing](hearing.md) ([Listen](skills.md#skill-listen) and
[Keen ears](talents-exploring.md#talent-keen-ears)) plus [Alert](talents-exploring.md#talent-alert). It needs the
difficulty plus their stalk plus the rain; neither cover nor the dark hides a sound.

Whoever comes always finds your camp. If you make the check, you wake in time to meet them ("You
wake to the sound of a band of footpads close by your camp."). If you fail, you wake with them upon
you. Travellers and animals wake you too. Either way your rest stops.

[Foraging](skill-uses.md#foraging) in the wild has the land's whole chance for the time it takes,
and you keep a traveller's lookout.

## The Ascmearc's encounters

Who you meet in the [Ascmearc](ascmearc.md), table by table. Where several tables apply, their
entries are drawn together by weight. A table "instead of" another leaves the other's entries out
while it applies. Where a row narrows its table (at night, once a place is known), you meet it only
then.

- **The roads** are the safest ground, but footpads walk them at every level, with their bowmen
  at night. The pedlar knows the ways east to Cealdhlāw and the ford. He tells them, and they are
  marked on your map as [rumoured ways](overworld.md#rumoured-ways) are. Ask him for the news of
  the roads too, and for [his one offer](npcs.md#the-pedlars-offer). Villagers of the vale greet
  you on their way to market.
- **Hlāwdūn**, the barrow downs: by day the vale's raiders and the barrows' spiders; at night and
  in twilight the barrows' dead, and whatever walks out of Cealdhlāw.
- **The deep woods** and **Ascdūn**: once you know where
  [Wulfeshol or Fyrdelf](overworld.md#hidden-locations) lies, what lives there comes out too.
- **Micelfenn**, the great fen: leeches and frogs, and the drowned at night. While Sister Hild's
  errand to the fen minster is under way, the cult walks the fen, and you meet more there.
- While the [Gilde](guild.md#the-wanted) wants Black Wystan, his archers watch the roads of the
  vale and the downs.
- A month into your stay, the orcs come down into Burnadenu.
- By day, on the roads, in the vale, on the downs, and in the woods, a [bard](class-bard.md) who
  has reached the class's last level may meet a [strange spirit](npcs.md#the-strange-spirit).
  Nobody else ever does (see [Prestige classes](prestige-classes.md#prestige-classes)).

{{#include generated/encounter-tables-ascmearc.md}}

"Drawn from the folk" of a [dungeon](dungeons.md#who-lives-there-and-how-many) means whoever you
would meet on its floors at that danger, strays included.

See also: [Surprise and sneak attacks](surprise.md) and [Using your skills](skill-uses.md).

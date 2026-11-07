# Encounters on the way

The [overworld](overworld.md#travel) is not empty between the places on the map. As you travel
you may meet someone on the way: a band of footpads on the road, goblin raiders in the hills, the
dead of the barrows walking at night, a pedlar with his pack, villagers going to market, or a herd
of [deer](creatures.md#creature-deer). Who you may meet depends on where you are, the time of
day, and what has happened in your story so far.

## The chance of meeting someone

Every step onto an overworld cell with a [local area](overworld.md#local-areas) (not a town, a
dungeon's entrance, or water) may meet someone. The chance is a rate **per hour of travel**, set
by the land you step onto, times the hours the step takes: a step that takes longer is longer
exposed. Roads are the safest ground, open land less safe, and woods, hills, deep forest, and
marsh the least. In [twilight](time.md#days) the chance is half as much again, and at night
double. Some places are busier than others: while a story keeps people on the move there, a
region's chance may rise (see [The Ascmearc's encounters](#the-ascmearcs-encounters)).
Travelling faster ([Survival](skills.md#skill-survival) and the talents that quicken your
journeys) means fewer hours on the way, and so fewer encounters.

{{#include generated/encounter-rates.md}}

A day's march is how many encounters twelve hours of daylight on that land bring on average.
There is no limit to how many you meet: the chance is the only one.

## Who sees whom

When someone is met, two checks decide who noticed whom first, each 2d12 plus an edge against the
number it needs, as [hitting](combat.md#hitting) is:

- **Your lookout**: your [Willpower](stats.md#stat-willpower) [modifier](stats.md#modifiers),
  plus what [Spot](skills.md#skill-spot) and [Survival](skills.md#skill-survival) give the
  lookout, plus [Alert](talents.md#talent-alert) and the other talents below. It needs the
  difficulty, plus their leader's stalk (what its [Stealth](skills.md#skill-stealth) gives), plus
  the land's cover, plus the dark at night or in twilight (half as much if you have
  [darkvision](vision.md#the-four-types)), plus the [rain](weather.md#how-the-weather-changes).
- **Their lookout**: their leader's [Willpower](stats.md#stat-willpower)
  [modifier](stats.md#modifiers) plus what its [Spot](skills.md#skill-spot) gives. It needs the
  difficulty, plus your stalk ([Stealth](skills.md#skill-stealth)), plus the cover, plus the dark
  (unless the leader sees in the dark), plus the rain, less a good deal if you carry a lit
  [lantern](lantern.md) at night or in twilight.

What comes of it:

| You saw them | They saw you | What happens |
|:---:|:---:|---|
| yes | no | **You have the drop on them.** "You spot a band of footpads on the road ahead. They have not seen you." You may lie in wait, slip past, or stand and fight. |
| yes | yes | **You meet.** "You and a band of footpads see each other at the same moment." You may give them the slip, or stand and fight. |
| no | yes | **You are ambushed.** "You are set upon by a band of footpads from the roadside!" There is no choice: the fight starts on their terms. |
| no | no | **You pass each other by.** Nothing happens, and you never know they were there. |

{{#include generated/encounter-checks.md}}

How often a check is made, by your edge and the number it needs:

{{#include generated/encounter-odds.md}}

What the skills give, by their value:

{{#include generated/encounter-ladders.md}}

Every lookout trains [Spot](skills.md#skill-spot) and [Survival](skills.md#skill-survival); a
party that misses you trains your [Stealth](skills.md#skill-stealth).

## Deciding what to do

When you meet someone, a question asks what you do, and it must be answered: until you choose,
you can do nothing else (but pick what a new level brings, or change your
[stance](combat.md#stance-neutral)).

- **Lie in wait** (when you have the drop): you take cover for ten minutes and let them walk into
  you. They come on unaware of you, as a creature you [slipped past](skill-uses.md#slipping-past)
  is, and a step behind: your first blows are [sneak attacks](surprise.md#sneak-attacks).
- **Slip past** (when you have the drop) or **Give them the slip** (when you met): a check of
  your [Dexterity](stats.md#stat-dexterity) [modifier](stats.md#modifiers) plus what
  [Survival](skills.md#skill-survival) gives slipping away (and your stalk too, if they have not
  seen you) against the difficulty, harder when they have seen you and when their leader is
  quicker than you. The detour takes half the step's time again, whatever comes of it. Made, you
  are on your way ("You slip away unseen.", "You lose them in the woods."); failed, they are
  onto you, and the fight starts close.
- <a id="talking-past"></a>**Talk your way past** (when you met, if you are trained in
  [Bluff](skills.md#skill-bluff), and only for the few parties who would rather hear a story
  than fight: footpads on the road, in the woods, and in the vale, the duergar prospectors at
  Ascdūn, and the cultists on the fen road; the tables below say "talked past"): their leader
  rolls 2d12 plus its [Willpower](stats.md#stat-willpower) [modifier](stats.md#modifiers) against
  the same difficulty a [feint](skill-uses.md#feinting) asks, plus your
  [Presence](stats.md#stat-presence) modifier, plus what your talents add to Presence checks,
  plus what your Bluff adds to a feint. Short of it, "You talk your way past.": they let you go,
  and it took as long as a word with someone. Otherwise, "They are not fooled.", and the fight
  starts as if you had stood to fight. Every try trains Bluff.
- **Stand and fight**: you go to meet them in the cell's [local area](overworld.md#local-areas).
- **Hail them** or **Pass by** (travellers who mean you no harm): hailed, you meet them in the
  local area, where you can [talk](npcs.md#chatting) to them as to anyone.
- **Hunt them** or **Let them be** (animals): hunted, you find them in the local area. A deer
  is good [food](food.md#corpses).

If you got away from a party, it stays where you left it. Meet the same sort of party on that
cell again and it is the same one, and you are told so: "You remember them: you got away from
them here before."

## Where the fight happens

A fight, a hail, or a hunt opens the local area of the cell you stepped onto, on the side you
came from, as `>` would. How far in you stand depends on how the encounter went: the better for
you, the nearer the edge you came from.

{{#include generated/encounter-positions.md}}

Ambushed, you are [caught unawares](surprise.md#caught-unawares) unless you keep your wits about
you. To get away, [leave the area](overworld.md#leaving-an-area) as you would any other: not with
an enemy within three cells of you.

## Camping and foraging

[Resting](resting.md#how-resting-works) on the overworld is not safe either. Each hour of rest has
a quarter of the land's chance of an hour's travel. Asleep, you keep no watch but your ears: your
check is your [Willpower](stats.md#stat-willpower) [modifier](stats.md#modifiers) plus your
[hearing](hearing.md) ([Listen](skills.md#skill-listen) and
[Keen ears](talents.md#talent-keen-ears)) plus [Alert](talents.md#talent-alert), against the
difficulty plus their stalk plus the rain (neither cover nor the dark hides a sound). Whoever comes
always finds your camp: made, you wake in time to meet them ("You wake to the sound of a band of
footpads close by your camp."); failed, you wake with them upon you. Travellers and animals wake
you too. Either way your rest stops.

[Foraging](skill-uses.md#foraging) in the wild has the land's whole chance for the time it takes, and
you keep a traveller's lookout.

## The Ascmearc's encounters

Who is met in the [Ascmearc](ascmearc.md), table by table. Where several tables hold, their
entries are drawn together by weight; a table "instead of" another leaves the other's entries
out while it holds. Where a row narrows its table (at night, once a place is known), it is met
only then.

- **The roads** are the safest ground, but footpads walk them at every level, with their bowmen
  at night. The pedlar knows the ways east to Cealdhlāw and the ford, and tells them, marking
  them on your map as [rumoured ways](overworld.md#rumoured-ways) are; ask him for the news of
  the roads too, and for [his one offer](npcs.md#the-pedlars-offer). Villagers of the vale greet you on their way to market.
- **Hlāwdūn**, the barrow downs: by day the vale's raiders and the barrows' spiders, at night and
  in twilight the barrows' dead, and whatever walks out of Cealdhlāw.
- **The deep woods** and **Ascdūn**: once you know where [Wulfeshol or Fyrdelf](overworld.md#hidden-locations)
  lies, what lives there comes out too.
- **Micelfenn**, the great fen: leeches and frogs, and the drowned at night. While Sister Hild's
  errand to the fen minster is under way, the cult walks the fen, and more is met there.
- While the [Gilde](guild.md#the-wanted) wants Black Wystan, his archers watch the roads of the
  vale and the downs.
- A month into your stay, the orcs come down into Burnadenu.
- By day, on the roads, in the vale, on the downs, and in the woods, a
  [bard](classes.md#bard) whose class is full may meet a
  [strange spirit](npcs.md#the-strange-spirit), and nobody else ever does (see
  [Prestige classes](prestige-classes.md#prestige-classes)).

{{#include generated/encounter-tables-ascmearc.md}}

"Drawn from the folk" of a [dungeon](dungeons.md#who-lives-there-and-how-many) is who you would
meet on its floors at that danger, strays included.

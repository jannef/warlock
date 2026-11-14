# Profane gifts

Eating the corpse of a [keeper of Grundlēas](dungeon-guide.md#the-keepers-of-the-layers) changes
your body for good. You get the keeper's own gift for certain, then one or more gifts drawn at
random from a table of good, bad, and mixed changes, and the meal pulls you hard toward evil.
Nothing takes a profane gift away again except another gift.

Every keeper's corpse gives profane gifts, from [Gnægan the Rat King](creatures.md#creature-gnaegan)
on the tenth floor to [Nīðdraca the Hate-Wyrm](creatures.md#creature-nithdraca) on the hundredth.
The deeper the keeper, the more gifts its meal draws, and the more of them come from the greater and dire
tiers.

## Eating a keeper's corpse

A keeper's corpse is eaten like any corpse, from the floor or your pack, and the game does not
ask you first. It never rots (it is **everlasting**), so the meal can wait as long as you like.
It feeds you as a medium creature's corpse does.

When the last turn of the meal is done, this happens, in this order:

1. The corpse's own words: "It is rats, many rats, and none of them fresh."
2. What any fresh corpse does ([what a corpse gives you](food.md#what-a-corpse-gives-you)). A
   keeper's corpse never comes back up.
3. The keeper's own gift, its **signature gift**: its line, then "You are Rat-king's court."
4. Each roll on the gift table, one after another: the gift's line, then "You are Hungering."
5. The pull toward evil (below), with a line if it moves you into another
   [band](alignment.md).
6. It feeds you.

A meal cut short (something comes into view) does nothing at all: the corpse stays where it was,
whole.

Gifts are [talents](talents.md) that nobody picks. They show on the Talents page under Profane, a
bad one marked as a flaw, as a birth sign's flaw is. The game never says which gift is good or
bad, nor any of their numbers; this page does.

## Each keeper's own gift

Each keeper's corpse gives its signature gift for certain, never drawn by anyone else. The rolls
column is how many gifts the meal draws; the last three columns are each roll's chance of each
tier of the table.

{{#include generated/profane-corpses.md}}

A gift's resistance adds to the rest of yours, with no cap on the total, but at most 75 % of it
ever counts (see [resistance](damage-types.md#armour-and-resistance)). So a signature's 100 % is
75 % with 25 to spare: [Rime-blooded](#talent-rime-blooded) keeps you at 75 % to cold even beside a 25 % weakness to it.
No gift makes you immune. Eat both wyrms and their gifts add up: 100 − 25 = 75 % to fire and to
cold alike, all of it counting.

[Hate-wyrm's heart](#talent-hate-wyrms-heart) lets you breathe fire, as the Wyvern's [Fire breath](horoscope.md#talent-fire-breath)
does but far stronger. It is used from the [abilities menu](abilities.md#the-abilities-menu), listed under Profane. If your sign
already gave you Fire breath you still have one breath, listed under your birth sign: the
stronger one, whichever came first.

## How a gift is drawn

Each roll works the same way:

1. **The tier.** The keeper's odds pick the lesser, greater, or dire tier.
2. **The gift.** One gift of that tier is picked by its weight, among the gifts you may take:
   ones you do not hold, that no gift you hold rules out, and that are not barred to your
   [race](races.md).
3. **An empty tier.** If you may take none of that tier, the roll moves to the next tier up, then
   down: lesser, then greater, then dire; dire, then greater, then lesser. If all three are empty,
   "Nothing more of it will take in you." and the roll gives nothing.

Each gift is given in full before the next roll, so a roll never draws what the one before it
ruled out. The odds in the table below are a tier's odds while none of its gifts is held or
barred. As you gather gifts the rest of the tier shares the odds, by the same weights.

## Gifts with other gifts

Some gifts change what happens to the others:

- **Replaces**: taking it takes the named gift away first. It is the greater form of the lesser
  gift (Wyrm's hunger over Hungering).
- **Undoes**: if you hold the named gift, the meal takes that gift away and gives you nothing
  else (Withered over Oak-hearted). If you do not hold it, you get this gift as usual.
- **Never with**: you cannot hold both. While you hold one, the other is never drawn.

There is no cure. No temple, potion, scroll, or prayer removes a profane gift. Only a later
gift that replaces or undoes it takes it away.

## The gift table

{{#include generated/profane-gifts.md}}

## The pull toward evil

Every keeper's corpse you eat is a deed worth −300 to your
[alignment](alignment.md#what-moves-you), the largest single sin in the game. Like every deed it
counts in full when you lean the other way, and for less the further toward evil you already
are. From Neutral at 0, one meal takes you to −300, which is Callous. A Saintly character at 1000
falls to 700, still Good, and an Evil one at −500 only to −650.

A [paladin](class-paladin.md)'s [zeal](zeal.md) empties at the meal, as it does at any sin. Once
you fall below Kind, a priest's holy gifts stop working (see
[What it changes](alignment.md#what-it-changes)).

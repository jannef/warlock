# Alignment

Every character stands somewhere between **good** and **evil**, and what you do moves you along
the way. Behind it is a score from −1000 (as evil as can be) to +1000 (as good), which the game
never shows: the character screen (`@`) and the summary at character creation name your
**band**, one of seven words.

{{#include generated/alignment-bands.md}}

## Where you start

Your class sets where you start, and your race leans you a little: paladins start Good, thieves
and witches Callous, and everyone else Neutral. Elves and halflings lean a little toward good and
half-orcs a little toward evil, never far enough to change a band on their own. The summary shows
the band you start in.

{{#include generated/alignment-start.md}}

## What moves you

Deeds move you. The further along a way you already are, the less each deed that way moves you:
a saint's next good deed counts for little, and small kindnesses alone never make anyone a
saint. A deed the other way counts in full, so a fall is quick and a climb back is slow.
Exactly: a deed in the direction your score already leans moves it by its worth × (1000 −
|score|) / 1000, rounded toward 0 (at +500 a good deed worth +10 moves you +5; at +950, not at
all); a deed the other way, or from 0, moves it by its whole worth. The score never passes −1000
or +1000.

{{#include generated/alignment-deeds.md}}

- **Killing**: it matters whether someone meant you harm. A creature is hostile to you if look
  mode says so; one that is not, such as the peaceful folk of the hall, is someone who means you
  no harm. Beasts that do not talk (deer, rats, bats) may be hunted without a stain. Only your
  own kills count, and those of a wound you dealt.
- **Shops**: attacking a shopkeeper who trades with you, and walking out with what you have not
  paid for, are evil; paying what you owe is a little good.
- **Corpses**: eating a corpse of your own race, or of someone who talks and meant you no harm,
  is very evil, fresh or rotten.
- **Quests** that help someone are a little good; a dark errand would be evil.

{{#include generated/alignment-quests.md}}

Nothing else moves you. When you cross into another band you feel it ("You feel righteous.",
"You feel you have strayed from the path of good."); smaller moves you do not notice.

## What it changes

- **People**: good folk turn cold to an Evil or Wicked character: they answer with a cold word
  instead of their greeting and their talk, and offer no quest that would do you good. Evil folk
  would do the same to a Good or Saintly one. Some quests are offered only to those of the right
  alignment.
- **Shops**: every shopkeeper is good, and their prices follow your band (the prices you see
  already include it):

{{#include generated/alignment-shops.md}}

- **Gifts**: some class abilities and talents ask for an alignment. A paladin's gifts need Good or
  better: a paladin who falls below keeps them, but cannot use them ("you have strayed") until
  they climb back.

What creatures are, good, neutral, or evil, you cannot see: you learn it from how they behave.
Magic (mana) does not care where you stand.

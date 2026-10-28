# Alignment

Every character stands somewhere between **good** and **evil**, and what you do moves you along
the way. Behind it is a score from −1000 (as evil as can be) to +1000 (as good), which the game
never shows: the [character screen](controls.md#character) (`@`) and the summary at character creation name your
**band**, one of seven words.

{{#include generated/alignment-bands.md}}

## Where you start

Your [class](classes.md) sets where you start, and your [race](races.md) leans you a little: [paladins](classes.md#paladin) start Good, [thieves](classes.md#thief)
and [witches](classes.md#witch) Callous, and everyone else Neutral. [Elves](races.md#race-elf) and [halflings](races.md#race-halfling) lean a little toward good and
[half-orcs](races.md#race-half-orc) a little toward evil, never far enough to change a band on their own; but the evil races
start on the evil side, [goblins](races.md#race-goblin), [kobolds](races.md#race-kobold), [lizardfolk](races.md#race-lizardfolk), and [ratfolk](races.md#race-ratfolk) Callous, [drow](races.md#race-drow) and [duergar](races.md#race-duergar) Evil
(see [Races](races.md#the-evil-races)), so a goblin [thief](classes.md#thief) starts Evil and a drow [paladin](classes.md#paladin) Neutral.
Then the [stars](horoscope.md) nudge you: the animal of your birth year adds to the score (the
[Horoscope](horoscope.md#what-a-sign-does) page's *Alignment* column): the Unicorn +100, the
Griffin, Phoenix, and Wyvern +50, the Kraken −50, the Basilisk and Manticore −100, the others
nothing. It is small, but near an edge it decides the band: an [elf](races.md#race-elf) [wizard](classes.md#wizard) born under the
Unicorn starts Kind (+150). A [paladin](classes.md#paladin) starts Good under any animal unless their race is one of
the evil ones: a [half-orc](races.md#race-half-orc) paladin born under the Manticore or the Basilisk starts at +400, just
Good.
The summary shows the band you start in. The table below is before the stars.

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

- **Killing**: it matters whether someone meant you harm. A [creature](creatures.md) is [hostile](creatures.md#factions) to you if [look
  mode](controls.md#look-mode) says so; one that is not, such as the peaceful folk of the hall, is someone who means you
  no harm. Beasts that do not talk ([deer](creatures.md#creature-deer), [rats](creatures.md#creature-rat), bats) may be hunted without a stain. Only your
  own kills count, and those of a wound you dealt.
- **[Shops](shops.md)**: attacking a shopkeeper who trades with you, and walking out with what you have not
  paid for, are evil; so is lifting something from anyone and getting away with it (see
  [Stealing](skills.md#stealing)); paying what you owe is a little good.
- **Corpses**: eating a [corpse](food.md#corpses) of your own race, or of someone who talks and meant you no harm,
  is very evil, fresh or rotten.
- **[Quests](quests.md)** that help someone are a little good; a dark errand would be evil.
- **Consecrating** a cursed, draining, or haunted place (the [priest](classes.md#priest)'s and the [paladin](classes.md#paladin)'s [Consecrate](abilities.md#ability-consecrate), see
  [Special rooms](special-rooms.md)) is good.

{{#include generated/alignment-quests.md}}

Nothing else moves you. When you cross into another band you feel it ("You feel righteous.",
"You feel your righteousness waver."); smaller moves you do not notice.

## What it changes

- **[People](npcs.md#chatting)**: good folk turn cold to an Evil or Wicked character: they answer with a cold word
  instead of their greeting and their talk, and offer no [quest](quests.md) that would do you good. Evil folk
  would do the same to a Good or Saintly one. Some quests are offered only to those of the right
  alignment.
- **[Shops](shops.md#prices)**: every shopkeeper is good, and their prices follow your band (the prices you see
  already include it):

{{#include generated/alignment-shops.md}}

- **Holy gifts**: no class, talent, or ability is barred by your alignment, and nothing you have
  is taken away when it sinks. But the holy gifts work only for the good side: used by anyone
  else they are still used, take the time of a cast, and do nothing (no healing, no blow, no
  light, no consecration, and no cooldown or [mana](mana.md#spending-it) spent). Putting your holy light out always
  works.

{{#include generated/alignment-gifts.md}}

What [creatures](creatures.md) are, good, neutral, or evil, you cannot see: you learn it from how they behave.
[Magic](magic.md) ([mana](mana.md#how-much)) does not care where you stand.

# Alignment

Every character stands somewhere between **good** and **evil**, and what you do moves you along
the way. Behind it is a score from −1000 (as evil as can be) to +1000 (as good). The game never
shows the score. The [character screen](the-screen.md#character) (`@`) and the summary at
character creation name your **band** instead, one of seven words.

{{#include generated/alignment-bands.md}}

## Where you start

Your [class](classes.md) sets where you start. [Paladins](class-paladin.md) start Good,
[thieves](class-thief.md) and [witches](class-witch.md) Callous, and everyone else Neutral.

Your [race](races.md) leans you a little. [Elves](races.md#race-elf) and
[halflings](races.md#race-halfling) lean a little toward good, and
[half-orcs](races.md#race-half-orc) a little toward evil, never far enough to change a band on
their own. The evil races, though, start on the evil side:
[goblins](races.md#race-goblin), [kobolds](races.md#race-kobold),
[lizardfolk](races.md#race-lizardfolk), and [ratfolk](races.md#race-ratfolk) Callous,
[drow](races.md#race-drow) and [duergar](races.md#race-duergar) Evil (see
[Races](races.md#the-evil-races)). So a goblin [thief](class-thief.md) starts Evil, and a drow
[paladin](class-paladin.md) Neutral.

Then the [stars](horoscope.md) nudge you. The animal of your birth year adds to the score (the
*Alignment* column on the [Horoscope](horoscope.md#what-a-sign-does) page):

| Animal | Adds |
|---|---:|
| Unicorn | +100 |
| Griffin, Phoenix, Wyvern | +50 |
| Kraken | −50 |
| Basilisk, Manticore | −100 |
| the others | 0 |

It is small, but near an edge it decides the band. An [elf](races.md#race-elf)
[wizard](class-wizard.md) born under the Unicorn starts Kind (+150). A
[paladin](class-paladin.md) starts Good under any animal unless their race is one of the evil
ones. A [half-orc](races.md#race-half-orc) paladin born under the Manticore or the Basilisk starts
at +400, just Good.

The summary shows the band you start in. The table below is before the stars.

{{#include generated/alignment-start.md}}

## What moves you

Deeds move you. The further along a way you already are, the less each deed that way moves you. A
saint's next good deed counts for little, and small kindnesses alone never make anyone a saint. A
deed the other way counts in full, so a fall is quick and a climb back is slow.

Exactly: a deed in the direction your score already leans moves it by its worth × (1000 − |score|)
/ 1000, rounded toward 0. At +500, a good deed worth +10 moves you +5; at +950, not at all. A deed
the other way, or from 0, moves it by its whole worth. The score never passes −1000 or +1000.

{{#include generated/alignment-deeds.md}}

- **Killing.** It matters whether someone meant you harm. A [creature](creatures.md) is
  [hostile](creatures.md#factions) to you if [look mode](controls-prompts.md#look-mode) says so. One that is
  not, such as the peaceful folk of the hall, means you no harm. Beasts that do not talk
  ([deer](creatures.md#creature-deer), [rats](creatures.md#creature-rat), bats) may be hunted
  without a stain. Only your own kills count, and those of a wound you dealt.
- **[Shops](shops.md).** Attacking a shopkeeper who trades with you is evil, and so is walking out
  with what you have not paid for. So is lifting something from anyone and getting away with it
  (see [Stealing](skill-uses.md#stealing)). Paying what you owe is a little good.
- **Corpses.** Eating a [corpse](food.md#corpses) of your own race, or of someone who talks and
  meant you no harm, is very evil, fresh or rotten.
- **[Quests](quests.md)** that help someone are a little good; a dark errand would be evil.
- **Consecrating** a cursed, draining, or haunted place is good. That is the
  [priest](class-priest.md)'s and the [paladin](class-paladin.md)'s
  [Consecrate](abilities.md#ability-consecrate); see [Special rooms](special-rooms.md).

{{#include generated/alignment-quests.md}}

Nothing else moves you. When you cross into another band you feel it ("You feel righteous.", "You
feel your righteousness waver."). Smaller moves you do not notice.

## What it changes

**[People](npcs.md#chatting).** Good folk turn cold to an Evil or Wicked character. They answer with
a cold word instead of their greeting and their talk, though they still offer you their
[quests](quests.md). Evil folk would do the same to a Good or Saintly one. Some quests are offered
only to those of the right alignment.

**[Shops](shops.md#prices).** Every shopkeeper is good, and their prices follow your band. The
prices you see already include it:

{{#include generated/alignment-shops.md}}

**Holy gifts.** No class, talent, or ability is barred by your alignment, and nothing you have is
taken away when it sinks. But the holy gifts work only for the good side. Used by anyone else, they
are still used and take the time of a cast, but do nothing: no healing, no blow, no light, no
consecration, and no cooldown, [zeal](resources.md#resource-zeal), or [mana](mana.md#spending-it)
spent. Putting your holy light out always works.

**Zeal.** Any deed that moves you toward evil empties a [paladin](class-paladin.md)'s
[zeal](resources.md#resource-zeal) at once ("Your zeal deserts you."). It bars nothing: the pool
fills again.

{{#include generated/alignment-gifts.md}}

You cannot see whether [creatures](creatures.md) are good, neutral, or evil; you learn it from how
they behave. [Magic](magic.md) ([mana](mana.md#how-much-mana-you-have)) does not care where you stand.

See also: [Races](races.md), [Prayer](prayer.md), and [Profane gifts](profane-gifts.md).

# Quests

Some of the [people you meet](npcs.md) have work for you. Quests are simple errands: kill
something, bring something, go somewhere, or carry something to someone else. Do it, come back,
and you are rewarded with [gold](shops.md#money), things, and [experience](experience.md).

## Getting a quest

Chat with someone (`C`). A quest they have for you is a topic on the
[dialogue screen](npcs.md#chatting), named after the quest and listed before their other topics.
Asking about it **gives** it to you. They say what they want, and the quest is yours at once, on
your [quest log](#the-quest-log).

There is no accepting and no declining. A quest you do not want simply waits in the log and costs
you nothing.

Some quests are offered only once you have done another one for the same person, and some only to
someone of the right [alignment](alignment.md). Good folk who greet an
[Evil or Wicked](alignment.md) character coldly still offer them their quests. Helping someone is
a little good.

Talking about quests takes no time.

## Kinds of quests

- **Kill**: kill a number of [creatures](creatures.md) of one kind, or of any kind of one
  [sort](#sorts-of-creatures), sometimes in a particular place ("Kill 5 giant rats in
  Cealdhlāw."). Only creatures you kill yourself count: with a weapon, a
  [shot](ranged.md#shooting-and-throwing), a [throw](ranged.md#shooting-and-throwing), or a wound
  you gave them that [bleeds them to death](status-effects.md#bleeding). One killed by someone
  else, or by a [trap](traps.md), does not count. Kills you made before you were given the quest
  count too, so the person is pleased to hear the job is done already. This is not so for a
  [drawn](#drawn-quests) quest, where only the kills after it was given count.
- **Named monsters**: a quest to kill a unique creature, one with a name of its own, is done by
  its death, however and whenever it died. It may die by your hand, by a trap, by another
  creature, or before you were even given the quest. Nothing can leave such a quest impossible to
  finish.
- **Bring**: bring some things in your [pack](objects.md#your-pack) ("Bring the silver chalice to
  Sister Hild."). They are taken from your pack when you hand the quest in. Anything you carry
  beyond what was asked stays yours.
- **Reach**: get to a place, such as a [level of a dungeon](dungeons.md) ("Reach level 4 of
  Cealdhlāw."). Having been there before counts.
- **Deliver**: the person gives you something as they give you the quest, to carry to someone
  else and hand in there. You need room for it in your [pack](objects.md#your-pack). Without it in
  your pack you cannot hand it in, so if you drop it, pick it up again.
- **Escort**: bring someone home ("Bring Eadgyth safely to the Hamlet."). See below.

### Escorts

Find the one you are to bring home and [chat](npcs.md#chatting) with them. They fall in behind you
as one of your [allies](allies.md), in the [coward stance](stances.md#stances), and follow you from
level to level.

They are no fighter. They strike only at an enemy right beside them, and they will not be
[set on](allies.md#orders) anything, though they come to heel and hold when told.
[Give](abilities.md#ability-give) them a weapon or armour and they put it on at once, dropping
what they held or wore there. What they cannot wear or wield, they refuse.

When they stand with you in the place they belong, the quest is done there and then. They say
their thanks, and they stay. The one who sent you thanks and pays you if they are there; if not,
the one you brought home pays.

If they die, whether they had joined you or not, the quest **fails**. The log says so, and it is
never offered again.

## Sorts of creatures

Some kill quests ask for a whole **sort** of creature: "Kill 100 goblins: goblins, goblin
archers, and goblin chiefs." Any creature of a kind that is of that sort counts, wherever you
kill it. Every level, every dungeon, and the overworld all add up, unless the quest names a place.

The quest says which kinds count. The table below lists every sort, by the tag the game's data
gives it, and the kinds of it.

- A kind may be of several sorts: a [goblin archer](creatures.md#creature-goblin-archer) is both a
  [goblin](creatures.md#creature-goblin) and an archer.
- Some who run together are not of one sort. An [ogre](creatures.md#creature-ogre) is no goblin, a
  [warg](creatures.md#creature-warg) no orc, and a [basilisk](creatures.md#creature-basilisk) no
  vermin.
- A creature's sort goes with its kind, whoever it sides with: a
  [kobold](creatures.md#creature-kobold) is always a kobold.

{{#include generated/creature-tags.md}}

Cēaptūn's adventurers' guild gives a chain of drawn quests, and ranks for doing them: see
[The Weyfarers' Gilde](guild.md).

## Drawn quests

Some quests do not say what they want until they are given. The person picks the target at random
as they give the quest, from a pool the quest names, and tells you ("There's a price on a goblin
chief."). Once drawn, the target stands until the quest is done. Asking again never draws anew.

The pool is one of:

- **a creature** of the kinds that live in some [dungeons](dungeons.md), never a named monster.
  Only the dungeons you can get into when you are given the quest count: a dungeon whose way in is
  still hidden from you is left out;
- **a sort of creature** from a list, one with a kind living in such a dungeon;
- **a thing** of one [class](objects.md#item-classes), such as the [scrolls](scrolls.md) you can
  find lying about;
- **one of a few named choices** the quest lists, each with its own words. Each choice may mark its
  own hidden place on your map, but only the one drawn is marked. A choice may be kept back for
  when none of the others may be drawn.

A [Beast-friend](talents.md#talent-beast-friend) (every [druid](classes.md#druid) is one) is never
sent to kill animals, the creatures of the [factions](creatures.md#factions) the talent calms: such a kind,
a sort of creature with an animal among its kinds, and a choice asking for either are left out of
the pool. Whether you are a Beast-friend counts as the quest is given, not after.

A kill quest drawn from a pool of creatures counts only the kills made after you were given it.

Some quests lead to a place nobody has found on their own. The person marks a
[hidden location](overworld.md#hidden-locations) on your map, and the quest log says where it
lies.

When something you do moves a quest along, the message line says so: "Quest updated: 2 / 5 giant
rats." When it is done: "Quest updated: 5 / 5 giant rats. Return to Reeve Leofric."

## Handing in

Go back to whoever the quest is handed in to and ask about the quest again. That is the person
who gave it, or for a delivery, the one you carry it to. Before it is done they remind you of what
they asked. Once it is done, they take what they asked for, thank you, and pay: "Quest complete:
Rats in the barrows.", "Reeve Leofric gives you 25 gold coins.", "You gain 30 experience." What
does not fit in your [pack](objects.md#your-pack) is put at your feet.

A few quests can be done again once they are done.

## The quest log

`q` opens the **quest log**: the quests you were given, then the ones you have done. Choose one
with the arrows to see:

- what it asks, in the giver's name ("Reeve Leofric wants you to kill 5 giant rats in
  Cealdhlāw.");
- how far you have got ("2 / 5 killed.", "Return to Reeve Leofric.");
- who gave it, in their words.

The log is page 7 of the [info screens](the-screen.md#info-pages). `1` to `9`, `Tab`, and
`Shift`+`Tab` switch between it and the [character screen](the-screen.md#character), the
[inventory](objects.md#your-pack), the [equipment](equipment.md), your [skills](skills.md), your
[weapon skills](weapon-skills.md), the [Magic page](schools.md#the-magic-page), your
[talents](talents.md), and [your story](background.md#when-it-is-told).

A quest stays in the log until you have done it. Quests cannot be given up, and there is no limit
on how many you may be on at once. Opening the log takes no time.

## Who gives quests

Three people in [Cēaptūn](ascmearc.md#the-places) have work for you. Each has a chain of quests,
offered one after the other.

- **[Reeve Leofric](creatures.md#creature-reeve)** sends you into
  [Cealdhlāw](dungeons.md#the-dungeons), the [barrow](dungeons.md#the-dungeons) east of town. He
  sends a [Beast-friend](talents.md#talent-beast-friend) after the barrow's skeletal rats instead
  of the living ones.
- **[Sister Hild](creatures.md#creature-sister)** at the Mynster wants back what the barrow and
  the drowned minster keep, and marks [Fennmynster](dungeon-guide.md#fennmynster) on your map.
- **[Factor Wulfrun](creatures.md#creature-mine-factor)** marks the old mine,
  [Fyrdelf](dungeon-guide.md#fyrdelf), once you have been deep in the barrow and reached level 5.
  Ask her sooner and she tells you that you are not ready, and gives you nothing.

In [Holtwīc](ascmearc.md#the-places), at the end of the north road,
**[Gammer Wynflæd](creatures.md#creature-wise-woman)** the wise-woman asks you to bring home a girl
the fair folk took. She marks [Ælfhām](dungeon-guide.md#ælfhām), their hall in the holly wood, on
your map. The girl waits at its very bottom.

The table lists every quest in its chain's order: what it asks, what it pays, what must be done
first, the level you must have reached for it to be given, and the
[hidden place](overworld.md#hidden-locations) it shows you.

{{#include generated/quests.md}}

# Quests

Some of the [people you meet](npcs.md) have work for you. Quests are simple errands, as in ADOM:
kill something, bring something, go somewhere, or carry something to someone else. Do it, come
back, and you are rewarded with gold, things, and [experience](experience.md).

## Getting a quest

Chat with someone (`C`). A quest they have for you is a topic on the dialogue screen, named after
the quest, listed before their other topics. Asking about it **gives** it to you: they say what
they want, and the quest is yours at once, on your [quest log](#the-quest-log). There is no
accepting and no declining (this is a roguelike: a quest you do not want simply waits in the log,
costing nothing). Some
quests are only offered once you have done another one for the same person, and some only to
someone of the right [alignment](alignment.md): good folk offer an Evil or Wicked character no
quest that would do them good. Helping someone is a little good.

Talking about quests takes no time.

## Kinds of quests

- **Kill**: kill a number of creatures of one kind, or of any kind of one [sort](#sorts-of-creatures),
  sometimes in a particular place ("Kill 5 giant rats in Cealdhlāw."). Only creatures you kill yourself count, with a weapon, a shot, or a
  throw, or by a wound you gave them that bleeds them to death; one killed by someone else, or by
  a trap, does not. Kills you made before you were given the
  quest count too, so the person is pleased to hear the job is done already (not for a
  [drawn](#drawn-quests) quest: there only the kills from the giving on count).
- **Named monsters**: a quest to kill a unique creature, one with a name of its own, is done by
  its death, however and whenever it died: by your hand, a trap, another creature, or before you
  were even given the quest. Nothing can leave such a quest undoable.
- **Bring**: bring some things in your pack ("Bring the silver chalice to Sister Hild."). They are taken
  from your pack when you hand the quest in; anything you carry beyond what was asked stays yours.
- **Reach**: get to a place, such as a level of a dungeon ("Reach level 4 of Cealdhlāw."). Having been
  there before counts.
- **Deliver**: the person gives you something as they give you the quest (you need room for it in your pack),
  to carry to someone else and hand in there. Without it in your pack you cannot hand it in: if
  you drop it, pick it up again.

## Sorts of creatures

Some kill quests ask for a whole **sort** of creature: "Kill 100 goblins: goblins, goblin
archers, and goblin chiefs." Any creature of a kind that is of that sort counts, wherever you
kill it: every level, every dungeon, the overworld, all add up (unless the quest names a
place). The quest says which kinds count; the table below lists every sort (its tag, as the
game's data calls it) and the kinds of it. A kind may be of several sorts (a goblin archer is a
goblin and an archer). Some who run together are not of one sort: an ogre is no goblin, a warg
no orc, and a basilisk no vermin. A creature's sort goes with its kind, whoever it sides with: a kobold is a
kobold.

{{#include generated/creature-tags.md}}

Cēaptūn's adventurers' guild gives a chain of drawn quests, and ranks for doing them: see
[The Weyfarers' Gilde](guild.md).

## Drawn quests

Some quests do not say what they want until they are given: the person picks it at random as
they give the quest, from a pool the quest names, and tells you ("There's a price on a goblin
chief."). Once drawn, it stands until the quest is done: asking again never draws anew. The pool
is one of:

- **a creature** of the kinds that live in some dungeons (never a named monster), of the dungeons
  you can get into when you are given the quest: a dungeon whose way in is still hidden from you
  is left out;
- **a sort of creature** from a list, one with a kind living in such a dungeon;
- **a thing** of one class, such as the scrolls you can find lying about;
- **one of a few named choices** the quest lists, each with its own words, and each perhaps
  marking its own hidden place on your map (only the one drawn is marked).

A drawn kill quest counts only the kills made after you were given it.

Some quests lead to a place nobody has found on their own: the person marks a
[hidden location](overworld.md#hidden-locations) on your map, and the quest log says where it
lies.

When something you do brings a quest along, the message line says so: "Quest updated: 2 / 5
giant rats.", and when it is done, "Quest updated: 5 / 5 giant rats. Return to Reeve Leofric."

## Handing in

Go back to whoever the quest is handed in to (the person who gave it, or for a delivery the one
you carry it to) and ask about the quest again. Before it is done they remind you of what they
asked; once it is, they take what they asked for, thank you, and pay: "Quest complete: Rats in
the barrows.", "Reeve Leofric gives you 25 gold coins.", "You gain 30 experience." What does not fit in
your pack is put at your feet. A few quests can be done again once they are done.

## The quest log

`q` opens the **quest log**: the quests you were given, then the ones you have done. Choose
one with the arrows to see what it asks, first in the giver's name ("Reeve Leofric wants you to
kill 5 giant rats in Cealdhlāw."), how far you have got ("2 / 5 killed.", "Return to
Reeve Leofric."), and who gave it, in their words. It is page 6 of the info screens: `1` to `6`,
`Tab`, and `Shift`+`Tab` switch between it and the character screen, the inventory, the
equipment, your skills, and your weapon skills.

A quest you are given stays in the log until you have done it: quests cannot be given up. There is
no limit on how many you may be on at once. Opening the log takes no time.

## Who gives quests

In Cēaptūn, three people have work for you, each a chain of three quests offered one after the
other: **Reeve Leofric** sends you into Cealdhlāw, the barrow east of town; **Sister Hild** at
the Mynster wants back what the barrow and the drowned minster keep, and marks Fennmynster on
your map; **Factor Wulfrun** starts you with an errand in town, and once you have been deep in
the barrow she marks the old mine, Fyrdelf. The table lists every quest in its chain's order,
what it asks, what it pays, what must be done first, and the hidden place it shows you.

{{#include generated/quests.md}}

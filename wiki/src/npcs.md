# People and talk

Not everyone you meet wants to fight. Some creatures are **people**: they go about their business,
talk when you talk to them, and now and then mutter to themselves. In the showcase you meet three:

- a **gardener**, pacing beside the flower bed by the ruined wall of the courtyard;
- a **wandering bard**, walking his round of the meadow east of the courtyard's gazebo;
- an **old hermit**, sitting in the lamplit corner of the hub's stairs nook. He knows the hall's
  rooms, and tells you about them if you ask.

They belong to the **peaceful folk** (see [Creatures](creatures.md#factions)): they never fight,
not even if you attack them.

## Chatting

Press `C` next to someone to chat. If exactly one creature you can see stands next to you, you
talk to that one; otherwise the message line asks `Talk in which direction? (Esc to cancel)`.
Chatting takes a few words' time (half a normal action, 3 seconds; see
[Turns and movement](turns.md#what-actions-cost)). You can only talk to someone you see.

- Most people answer with a **one-liner** on the message line: `The gardener says: "Mind the
  beds. Flowers bruise if you so much as look at them."` Chat again and you get another. Each
  person goes through all of their lines before saying one again, in an order of their own, and
  never says the same line twice in a row.
- Some greet you the first time you talk to them, with a line they say only then.
- Some have **topics** you can ask about. Chatting with them opens a small screen: who you are
  talking to, what they just said, and their topics, numbered. Press a topic's number (or move
  the `>` marker with the arrows and press `Enter`) to ask. Asking takes no time; the answer
  appears on the screen and on the message line. `Esc` (or `C`) leaves.
- Creatures with nothing to say do not answer ("The giant rat doesn't answer."), and anyone
  hostile to you is in no mood to talk. Trying still takes the time of a chat.

{{#include generated/dialogue.md}}

Some people have work for you: their quests are topics too, listed first. See
[Quests](quests.md).

## Muttering

Now and then, while they go about their business and you are within a few cells, people mutter
to themselves: the gardener grumbles about his beds, the bard sings snatches of old songs, the
hermit talks to nobody in particular. You hear it like any other sound (see
[Hearing](hearing.md)): if you see them, with their name (`The gardener grumbles: "Weeds. Always
the weeds."`); if you do not, as a voice from somewhere (`You hear a voice to the north: "…"`),
and if it was faint, without the words. Muttering never wakes you from a [rest](resting.md).

## Getting past

You never have to wait for someone to move out of your way. Walking into a creature that is not
hostile to you swaps places with it: you step onto its cell, it steps onto yours ("You swap
places with the deer."). That takes your step's time, and it does not cost the creature its own
turn. Afterwards it walks back to where it stood (waiting beside the spot while you are on it)
and carries on with whatever it was doing.

A few will not make way, and then the move takes no time and the message says why:

- some never budge, and some keep their post: a shopkeeper keeps to the door of their shop
  ("Grimbold doesn't budge."; shops are still to come);
- someone who cannot move at all, under too heavy a load, does not budge either;
- a creature two sizes or more larger than you is too big to squeeze past ("The giant is too big
  to squeeze past."). No creature in the game is that big yet.

If you cannot see what is in your way, you only learn that "Something is in the way.", or that
you swapped places with something.

Moving into a hostile creature attacks it instead; see [Combat](combat.md).

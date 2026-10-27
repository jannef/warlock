# People and talk

Not everyone you meet wants to fight. Some creatures are **people**: they go about their business,
talk when you talk to them, and now and then mutter to themselves. In Cēaptūn, the market town
of the Ascmearc, you meet:

- the **townsfolk**, going about the square and the lanes on market day;
- **Sister Hild**, in the Mynster, the town's temple: she cures and mends for gold (see
  "Services" below), and anyone may rest on the Mynster's floor, a sanctuary;
- **Godgifu**, who keeps the Gæsthūs, the inn: she hears every rumour in the march, and the
  bread on her table is free to travellers. Ask her about a place (the barrows, the marsh, the
  mines, the volcano, the pass) and she tells you the way: you mark it on your map ("You mark
  the way on your map."), as much of the country as you would have seen walking that way by day
  (see [The overworld](overworld.md#rumoured-ways));
- **Reeve Leofric**, who keeps the town's peace and its coin, at home south of the square;
- **Factor Wulfrun**, who bought and sold the ore of mines that have gone quiet, in her office.

The people of the town are known by their names: `Sister Hild says: "…"`. They belong to the **peaceful folk** (see [Creatures](creatures.md#factions)): they never fight,
not even if you attack them.

## Chatting

Press `C` next to someone to chat. If exactly one creature you can see stands next to you, you
talk to that one; otherwise the message line asks `Talk in which direction? (Esc to cancel)`.
Chatting takes a few words' time (half a normal action, 3 seconds; see
[Turns and movement](turns.md#what-actions-cost)). You can only talk to someone you see.

- Most people answer with a **one-liner** on the message line: `The villager says: "Snow on
  Beorggeat already. Early, this year."` Chat again and you get another. Each
  person goes through all of their lines before saying one again, in an order of their own, and
  never says the same line twice in a row.
- Some greet you the first time you talk to them, with a line they say only then.
- Good folk turn cold to an Evil or Wicked character (see [Alignment](alignment.md)): they
  answer with a cold word instead of their greeting and their talk.
- Some have **topics** you can ask about. Chatting with them opens a small screen: who you are
  talking to, what they just said, and their topics, numbered. Press a topic's number (or move
  the `>` marker with the arrows and press `Enter`) to ask. Asking takes no time; the answer
  appears on the screen and on the message line. `Esc` (or `C`) leaves.
- Creatures with nothing to say do not answer ("The giant rat doesn't answer."), and anyone
  hostile to you is in no mood to talk. Trying still takes the time of a chat.

{{#include generated/dialogue.md}}

Some people have work for you: their quests are topics too, listed first. See
[Quests](quests.md).

## Services

A few people sell a **service**: a cure, a mending, or an identification, for gold. Services are
listed on their screen after their topics, with the price: `A cure for what ails you (60 gold)`. Pick one as you
would a topic. If there is something to do and you carry the gold, you pay (the coins go into
their purse), they do it, and it takes the time of a chat; their price is fixed (no haggling:
neither Presence nor Appraise moves it).

- **A cure** ends every disease you have, even one that has not broken out yet (they can tell),
  and grave rot too. Damage a disease did to your stats stays and mends a point a day as usual,
  or all at once with a mending.
- **A mending** gives back every point of stat damage at once, and every point the undead
  drained ([Drained stats](stats.md#drained-stats)). It is refused while a disease
  shows ("Be cured first."): the disease would only take them again.
- **An identification** (Factor Wulfrun in Cēaptūn) identifies everything you carry that you
  do not know yet, all at once: the magic of every unidentified item, and the type of every
  scroll you have not identified. It costs **100 gold per unidentified stack** (a stack of
  several like things counts once, however many are in it), so three unknown things cost 300
  gold. The list shows the price per thing; before you pay, a dialog tells you the total. You
  pay for all or nothing: with too little gold for the whole lot, nothing is identified.
  Each thing's true name is told as it is learned.
- With nothing to cure, mend, or identify they tell you so, and you keep your gold. Without enough gold you
  are told "You cannot pay." Neither takes any time.
- The **Pilgrim** talent (Wayfarer) halves a temple's price, a cure's or a mending's (rounded
  up), and its cures also mend a point of each damaged stat (see [Talents](talents.md)). Nothing
  moves the price of an identification, Lore and Connoisseur included.

{{#include generated/services.md}}

## Muttering

Now and then, while they go about their business and you are within a few cells, people mutter
to themselves: the townsfolk grumble about prices and the ash, Sister Hild prays, Godgifu hums
over her work. You hear it like any other sound (see [Hearing](hearing.md)): if you see them,
with their name (`The villager mutters: "Ash in the well again."`); if you do not, as a voice from somewhere (`You hear a voice to the north: "…"`),
and if it was faint, without the words. Muttering never wakes you from a [rest](resting.md).

## Getting past

You never have to wait for someone to move out of your way. Walking into a creature that is not
hostile to you swaps places with it: you step onto its cell, it steps onto yours ("You swap
places with the deer."). That takes your step's time, and it does not cost the creature its own
turn. It lands on your cell as any step lands: a trap you stood on goes off at it, and ice that
held you may not hold something heavier. Afterwards it walks back to where it stood (waiting beside the spot while you are on it)
and carries on with whatever it was doing.

A few will not make way, and then the move takes no time and the message says why:

- some never budge, and some keep their post: a shopkeeper keeps to the door of their shop
  ("Eadric doesn't budge.");
- someone who cannot move at all, under too heavy a load, does not budge either;
- a creature two sizes or more larger than you is too big to squeeze past ("The giant is too big
  to squeeze past."). No creature in the game is that big yet.

If you cannot see what is in your way, you only learn that "Something is in the way.", or that
you swapped places with something.

Moving into a hostile creature attacks it instead; see [Combat](combat.md).

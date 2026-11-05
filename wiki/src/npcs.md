# People and talk

Not everyone you meet wants to fight. Some creatures are **people**: they go about their business,
talk when you talk to them, and now and then mutter to themselves. In [Cēaptūn](ascmearc.md#the-places), the market town
of the [Ascmearc](ascmearc.md), you meet:

- the **[townsfolk](creatures.md#creature-villager)**, going about the square and the lanes on market day;
- **[Sister Hild](creatures.md#creature-sister)**, in the Mynster, the town's temple: she cures and mends for [gold](shops.md#money) (see
  "[Services](#services)" below), and anyone may [rest](resting.md#how-resting-works) on the Mynster's floor, a sanctuary;
- **[Godgifu](creatures.md#creature-innkeeper)**, who keeps the Gæsthūs, the inn: she hears every rumour in the march, and the
  bread on her table is free to travellers. Ask her about a place (the barrows, the marsh, the
  mines, the volcano, the pass) and she tells you the way: you mark it on your map ("You mark
  the way on your map."), as much of the country as you would have seen walking that way by day
  (see [The overworld](overworld.md#rumoured-ways));
- **[Reeve Leofric](creatures.md#creature-reeve)**, who keeps the town's peace and its coin, at home south of the square;
- **[Factor Wulfrun](creatures.md#creature-mine-factor)**, who bought and sold the ore of mines that have gone quiet, in her office.

The people of the town are known by their names: `Sister Hild says: "…"`. They belong to the **peaceful folk** (see [Creatures](creatures.md#factions)): they never fight,
not even if you attack them.

On the roads of the march you may [meet](encounters.md) people too: villagers of the vale on their
way to market, with a word for the road, and a **[pedlar](creatures.md#creature-pedlar)** with
his pack, who tells you the news of the roads and the way east to Cealdhlāw and the ford (he
marks it on your map as Godgifu does). He is one of the merchants (see
[Creatures](creatures.md#factions)), not of the peaceful folk.

## Chatting

Press `C` next to someone to chat. If exactly one creature you can see stands next to you, you
talk to that one; otherwise the [message line](controls.md#messages) asks `Talk in which direction? (Esc to cancel)`.
Chatting takes a few words' time (half a normal action, 3 seconds; see
[Turns and movement](turns.md#what-actions-cost)). You can only talk to someone you see.

- Most people answer with a **one-liner** on the message line: `The villager says: "Snow on
  Beorggeat already. Early, this year."` Chat again and you get another. Each
  person goes through all of their lines before saying one again, in an order of their own, and
  never says the same line twice in a row.
- Some greet you the first time you talk to them, with a line they say only then.
- Good folk turn cold to an [Evil or Wicked](alignment.md) character (see [Alignment](alignment.md)): they
  answer with a cold word instead of their greeting and their talk.
- Some have **topics** you can ask about. Chatting with them opens a small screen: who you are
  talking to, what they just said, and their topics, numbered. Press a topic's number (or move
  the `>` marker with the arrows and press `Enter`) to ask. Asking takes no time; the answer
  appears on the screen and on the message line. `Esc` (or `C`) leaves.
- Creatures with nothing to say do not answer ("The giant rat doesn't answer."), and anyone
  [hostile](creatures.md#factions) to you is in no mood to talk. Trying still takes the time of a chat.

{{#include generated/dialogue.md}}

Some people have work for you: their [quests](quests.md) are topics too, listed first. See
[Quests](quests.md).

## Saying words

Some people listen for words: a password, the answer to a riddle, a name. Where the one you
talk to knows any, their talk screen has one more row under the topics, `s  Say something…`.
Press `s` (or choose the row and press `Enter`) and type what you say, up to 40 characters, then
`Enter` to say it (`Esc` goes back to the topics; an empty line says nothing).

- Only the words count: capitals, punctuation, and extra spaces are ignored, so *Life before
  death!* and *life, before death* are the same words.
- If they know the words they answer, a line at a time, and saying them may do something: call
  you into a new class (below, and [Prestige classes](classes.md#prestige-classes)), give you a [quest](quests.md), or send the speaker on their way.
- Words that would call you into a new class are heard only from someone that class is open to,
  at the level it asks, who has not been called before; anyone else is answered as if they had
  said the wrong words.
- If they do not, they say so in their own way, or say nothing.
- Saying something takes a chat's time ([3 seconds](turns.md#what-actions-cost)), heard or not.
  You may try as often as you like.
- Some speak only to people of a certain [alignment](alignment.md): anyone else gets a line
  that turns them away, and no topics to ask about and nothing to say.

{{#include generated/words.md}}

### The Strange Spirit

In the Ascmearc a **[strange spirit](creatures.md#creature-strange-spirit)** may be met by day on
the roads, in the vales, on the downs, and in the woods (see [Encounters](encounters.md)), but
only by a [bard](classes.md#bard) whose class is full and who has not yet been called into
another. It is [peaceful](encounters.md): hail it and it stands where it is and waits. It
speaks only to the good (Kind or better on the [alignment](alignment.md) scale); anyone else it
looks through. It has nothing to ask about: the `Say something…` row is the whole of its talk.
Say *life before death* to it and it echoes the whole oath, a line at a time, calls you to be a
Hunter, and fades. You are given the experience the next level costs if you have less, so the
first Hunter level comes at once. Wrong words cost a chat's time and nothing more, and it waits
until you leave. It has one hit point and never fights back: killed, it leaves no corpse and is
worth no experience, and killing it is killing a peaceful good soul. The road may bring it again
while the class is still open to you. The words are cut into a statue deep in Fyrdelf.

## Services

A few people sell a **service**: a cure, a mending, or an identification, for [gold](shops.md#money). Services are
listed on their screen after their topics, with the price: `A cure for what ails you (60 gold)`. Pick one as you
would a topic. If there is something to do and you carry the gold, you pay (the coins go into
their purse), they do it, and it takes the time of a chat. A temple's price is fixed (no haggling), but a healer's or a
sage's moves with your [Presence](stats.md#stat-presence) and [Negotiation](skills.md#skill-negotiation), as a
[shop's](shops.md) does (at most half off, at least one gold), and any service paid trains Negotiation now and then.

- **A cure** ends every [disease](diseases.md) you have, even one that has not broken out yet (they can tell),
  and [grave rot](diseases.md#grave-rot) too. Damage a [disease](diseases.md#how-a-disease-goes) did to your [stats](stats.md#drained-stats) stays and mends a point a day as usual,
  or all at once with a mending.
- **A mending** gives back every point of [stat damage](stats.md#drained-stats) at once, and every point the undead
  drained ([Drained stats](stats.md#drained-stats)). It is refused while a [disease](diseases.md)
  shows ("Be cured first."): the disease would only take them again.
- **An identification** ([Factor Wulfrun](creatures.md#creature-mine-factor) in Cēaptūn) identifies everything you carry that you
  do not know yet, all at once: the magic of every [unidentified](magic-items.md#unidentified-items) item, and the type of every
  [scroll](scrolls.md) you have not identified. It costs **100 [gold](shops.md#money) per unidentified [stack](objects.md#piles)** (a stack of
  several like things counts once, however many are in it), so three unknown things cost 300
  [gold](shops.md#money). The list shows the price per thing; before you pay, a dialog tells you the total. You
  pay for all or nothing: with too little [gold](shops.md#money) for the whole lot, nothing is identified.
  Each thing's true name is told as it is learned.
- With nothing to cure, mend, or identify they tell you so, and you keep your [gold](shops.md#money). Without enough [gold](shops.md#money) you
  are told "You cannot pay." Neither takes any time.
- The **[Pious](talents.md#talent-pious)** talent ([Wayfarer](talents.md#wayfarer)) halves a temple's price, a cure's or a mending's (rounded
  up), and its cures also mend a point of each damaged [stat](stats.md#drained-stats) (see [Talents](talents.md)). Nothing
  moves the price of an identification.

{{#include generated/services.md}}

## The pedlar's offer

The **[pedlar](creatures.md#creature-pedlar)** met on the roads ([Encounters](encounters.md#the-ascmearcs-encounters))
carries one thing worth more than pins and needles, and offers it for a fixed price: take it or
leave it. Ask him about **Your wares**, listed after his topics with the price; a dialog shows
the item, already [identified](magic-items.md#unidentified-items) (a pedlar knows what he sells),
its affixes in words, and the price, with **Buy it** and **Leave it**.

- **What it is**: a [rare](magic-items.md#rarity) item of any kind loot holds, but no ammunition,
  with **four affixes (half the time), five (35%), or six (15%)**, made at an
  [item level](magic-items.md#finding-them) of **your level plus four**, at least 4. Loot is
  rare one time in fifty, most often with three affixes, and at the depth's level, so his is
  much the better.
- **One of three uniques**: one offer in fifty (2%) is instead one of the three
  [unique items](magic-items.md#unique-items) only pedlars carry, the Brosinga Mene, the
  Eorcanstān, and Grendles Grāpe, each offered at most once a run. When all three have been
  offered, his offers are rare finds again.
- **The price** is fixed, never haggled ([Presence](stats.md#stat-presence) and
  [Negotiation](skills.md#skill-negotiation) do not move it): its value plus **15 [gold](shops.md#money)
  per affix per item level** (four affixes at item level 5: its value and 300 gold), and
  **1500 gold** for a unique.
- **Buying** pays him, puts the item in your pack, and takes the time of a chat; it is a deal,
  which trains Negotiation and [Presence](stats.md#stat-presence) as trade does. Without enough gold
  the dialog only lets you leave it, and you are told "You cannot pay." Leaving it costs nothing,
  and he keeps it for you: ask again while he is near.
- **One offer per pedlar**: once you have bought it he has nothing more ("That was the only one
  I 'ad"). The same pedlar met again on the road still has his; another pedlar brings another
  offer, made when you first talk to him.

## Muttering

Now and then, while they go about their business and you are within a few cells, people mutter
to themselves: the townsfolk grumble about prices and the ash, [Sister Hild](creatures.md#creature-sister) prays, [Godgifu](creatures.md#creature-innkeeper) hums
over her work. You hear it like any other sound (see [Hearing](hearing.md)): if you see them,
with their name (`The villager mutters: "Ash in the well again."`); if you do not, as a voice from somewhere (`You hear a voice to the north: "…"`),
and if it was faint, without the words. Muttering never wakes you from a [rest](resting.md).

## Getting past

You never have to wait for someone to move out of your way. Walking into a creature that is not
[hostile](creatures.md#factions) to you swaps places with it: you step onto its cell, it steps onto yours ("You swap
places with the deer."). That takes your step's time, and it does not cost the creature its own
turn. It lands on your cell as any step lands: a [trap](traps.md) you stood on goes off at it, and [ice](temperature.md#walking-on-ice) that
held you may not hold something heavier. Afterwards it walks back to where it stood (waiting beside the spot while you are on it)
and carries on with whatever it was doing.

A few will not make way, and then the move takes no time and the message says why:

- some never budge, and some keep their post: a [shopkeeper](creatures.md#creature-shopkeeper) keeps to the door of their shop
  ("Eadric doesn't budge.");
- someone who cannot move at all, under too heavy a [load](burden.md), does not budge either;
- a creature two sizes or more larger than you is too big to squeeze past ("The giant is too big
  to squeeze past."). No creature in the game is that big yet.

If you cannot see what is in your way, you only learn that "Something is in the way.", or that
you swapped places with something.

Moving into a hostile creature attacks it instead; see [Combat](combat.md).

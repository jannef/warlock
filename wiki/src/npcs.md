# People and talk

Not everyone you meet wants to fight. Some creatures are **people**: they go about their
business, talk when you talk to them, and now and then mutter to themselves. In
[Cēaptūn](ascmearc.md#the-places), the market town of the [Ascmearc](ascmearc.md), you meet:

- the **[townsfolk](creatures.md#creature-villager)**, going about the square and the lanes on
  market day;
- **[Sister Hild](creatures.md#creature-sister)**, in the Mynster, the town's temple. She cures and
  mends for [gold](shops.md#money) (see "[Services](#services)" below), and anyone may
  [rest](resting.md#how-resting-works) on the Mynster's floor, which is a sanctuary;
- **[Godgifu](creatures.md#creature-innkeeper)**, who keeps the Gæsthūs, the inn. She hears every
  rumour in the march, and the bread on her table is free to travellers. Ask her about a place
  (the barrows, the marsh, the mines, the volcano, the pass) and she tells you the way. You mark
  it on your map ("You mark the way on your map."), as much of the country as you would have seen
  walking that way by day (see [The overworld](overworld.md#rumoured-ways));
- **[Reeve Leofric](creatures.md#creature-reeve)**, who keeps the town's peace and its coin, at
  home south of the square;
- **[Factor Wulfrun](creatures.md#creature-mine-factor)**, in her office. She bought and sold the
  ore of mines that have now gone quiet.

The people of the town are known by their names: `Sister Hild says: "…"`. They belong to the
**peaceful folk** (see [Creatures](creatures.md#factions)) and never fight, not even if you attack
them.

On the roads of the march you may [meet](encounters.md) people too. Villagers of the vale on
their way to market have a word for the road. A **[pedlar](creatures.md#creature-pedlar)** with
his pack tells you the news of the roads and the way east to Cealdhlāw and the ford, and marks it
on your map as Godgifu does. He is one of the merchants (see [Creatures](creatures.md#factions)),
not of the peaceful folk.

## Chatting

Press `C` next to someone to chat. If exactly one creature you can see stands next to you, you
talk to that one. Otherwise the [message line](the-screen.md#messages) asks
`Talk in which direction? (Esc to cancel)`. You can only talk to someone you see. Chatting takes a
few words' time: half a normal action, 3 seconds (see
[Turns and movement](turns.md#what-actions-cost)).

- Most people answer with a **one-liner** on the message line: `The villager says: "Snow on
  Beorggeat already. Early, this year."` Chat again and you get another. Each person goes through
  all of their lines, in an order of their own, before saying one again, and never says the same
  line twice in a row.
- Some greet you the first time you talk to them, with a line they say only then.
- Good folk turn cold to an [Evil or Wicked](alignment.md) character (see
  [Alignment](alignment.md)). They answer with a cold word instead of their greeting and their
  talk.
- Some have **topics** you can ask about. Chatting with them opens a small screen showing who you
  are talking to, what they just said, and their topics, numbered. Press a topic's number (or
  move the `>` marker with the arrows and press `Enter`) to ask. Asking takes no time. The answer
  appears on the screen and on the message line. `Esc` (or `C`) leaves.
- Creatures with nothing to say do not answer ("The giant rat doesn't answer."), and anyone
  [hostile](creatures.md#factions) to you is in no mood to talk. Trying still takes the time of a
  chat.

{{#include generated/dialogue.md}}

Some people have work for you. Their [quests](quests.md) are topics too, listed first. See
[Quests](quests.md).

### The Strange Spirit

In the Ascmearc a **[strange spirit](creatures.md#creature-strange-spirit)** may be met by day on
the roads, in the vales, on the downs, and in the woods (see [Encounters](encounters.md)). Only a
[bard](classes.md#bard) who has reached level 30, the top of the class, and who has not yet been called into another, meets
it.

It is [peaceful](encounters.md): hail it and it stands where it is and waits. It speaks only to
the good (Kind or better on the [alignment](alignment.md) scale) and looks through anyone else.
Speak to it and it grants you the calling: you become a Hunter (see
[Prestige classes](prestige-classes.md#prestige-classes)), and it fades.

It has one hit point and never fights back. Killed, it leaves no corpse and is worth no
experience, and killing it is killing a peaceful good soul. The road may bring it again while the
class is still open to you.

### Morwenna

**Morwenna** reads [magic crystals](crystals.md). She is a deep gnome who stands by a brazier on
the cutting floor of [Gimmwīc](ascmearc.md#gimmwīc), near Old Tregear. She has no topics and
sells nothing, but each time you chat she tells you something about the crystals: what kinds of
magic each colour holds, what the [rare materials](crystals.md#rare-materials) do, and what
[consuming one](crystals.md#consuming-a-crystal) does to you. Chat with her a few times to hear
them all.

## Services

A few people sell a **service** for [gold](shops.md#money): a cure, a mending, an
identification, or training in a [skill](skills.md). Services are listed on their screen after their topics, with the price:
`A cure for what ails you (60 gold)`. Pick one as you would a topic. If there is something to do
and you carry the gold, you pay (the coins go into their purse) and they do it. It takes the time
of a chat.

A temple's price is fixed, with no haggling. A healer's or a sage's price moves with your
[Presence](stats.md#stat-presence) and [Negotiation](skills.md#skill-negotiation), as a
[shop's](shops.md) does: at most half off, and at least one gold. Any service you pay for trains
Negotiation now and then.

- **A cure** ends every [disease](diseases.md) you have, even one that has not broken out yet
  (they can tell), and [grave rot](diseases.md#grave-rot) too. Damage a
  [disease](diseases.md#how-a-disease-goes) did to your [stats](stats.md#drained-stats) stays. It
  mends a point a day as usual, or all at once with a mending.
- **A mending** gives back at once every point of [stat damage](stats.md#drained-stats) and every
  point the undead drained ([Drained stats](stats.md#drained-stats)). It is refused while a
  [disease](diseases.md) shows ("Be cured first."), since the disease would only take them again.
- **An identification** ([Factor Wulfrun](creatures.md#creature-mine-factor) in Cēaptūn)
  identifies, all at once, everything you carry that you do not know yet: the magic of every
  [unidentified](magic-items.md#unidentified-items) item, and the type of every
  [scroll](scrolls.md) you have not identified. It costs **100 [gold](shops.md#money) per
  unidentified [stack](objects.md#piles)**. A stack of several like things counts once, however
  many are in it, so three unknown things cost 300 gold. The list shows the price per thing, and
  before you pay a dialog tells you the total. You pay for all or nothing: with too little gold
  for the whole lot, nothing is identified. Each thing's true name is told as it is learned.
- **Training** teaches you a [skill](skills.md) you do not have yet. You start it at a low value
  (5), as if you had just picked it up, and it grows by use like any other. It is offered only
  while you are untrained in that skill: once you have it, it is gone from the list.
  [Alberich](#alberich), deep in Fyrdelf, teaches [Mining](skills.md#skill-mining), which you need
  to [dig](digging.md#digging) at all. [Eadric](#eadric-the-smith), the smith in Cēaptūn, teaches
  [Metalcraft](skills.md#skill-metalcraft), which every smithing
  [recipe](skill-uses.md#professions) needs. [Old Tregear](#old-tregear), in Gimmwīc, teaches
  [Jewelcraft](skills.md#skill-jewelcraft).
- With nothing to cure, mend, or identify, or a skill you already have, they tell you so, and you
  keep your gold. Without
  enough gold you are told "You cannot pay." Neither takes any time.
- The **[Pious](talents.md#talent-pious)** talent ([Wayfarer](talents.md#wayfarer)) halves a
  temple's price for a cure or a mending (rounded up), and its cures also mend a point of each
  damaged [stat](stats.md#drained-stats) (see [Talents](talents.md)). Nothing moves the price of
  an identification.

{{#include generated/services.md}}

### Alberich

On the fifth floor of Fyrdelf, the first of its deep caves, a dwarf keeps a little square chamber
with a heap of gold dust in it. He is **Alberich**, and he is not quite right in the head: he
counts his gold, talks to it, and watches your hands. He fights no one and never leaves his
hoard. Ask him, and he teaches you [Mining](skills.md#skill-mining) for 500 gold (a little less
or more by your [Presence](stats.md#stat-presence) and [Negotiation](skills.md#skill-negotiation),
as a healer's price is). Bring a [pick-axe](digging.md#digging).

### Eadric the smith

**Eadric** keeps the smithy on Cēaptūn's market square ([Shops](shops.md)), beside the smith's
fire and the [anvil](altars-forges-springs.md). Besides selling arms and armour, he teaches
[Metalcraft](skills.md#skill-metalcraft) for **2000 gold**, a little less or more by your
[Presence](stats.md#stat-presence) and [Negotiation](skills.md#skill-negotiation), and less again
with the town's [goodwill](#goodwill). You start it at 5. Without it you cannot temper or forge
anything at an anvil.

### Old Tregear

**Old Tregear** works the cutting floor of [Gimmwīc](ascmearc.md#gimmwīc), the deep gnomes' town.
He teaches [Jewelcraft](skills.md#skill-jewelcraft) for **1500 gold**, a little less or more by
your [Presence](stats.md#stat-presence) and [Negotiation](skills.md#skill-negotiation). You start
it at 5. Without it you cannot [cut](gem-cutting.md) a gem.

### Goodwill

A town remembers who helps it. Every [quest](quests.md) you complete for one of a town's people
earns it **goodwill**, and its people's services get cheaper: **5% off per quest** in Cēaptūn.
Its people are Sister Hild, Reeve Leofric, Factor Wulfrun, Godgifu, Wat One-Eye, and Eadric the
smith, so the
[Weyfarers' Gilde's](guild.md) work counts too. A quest you do again counts again. Quests from
elsewhere, such as Holtwīc's, do not count. You are told when it rises: "Cēaptūn's folk think
the better of you."

- The goodwill comes off **last**, after [Pious](talents.md#talent-pious) and any haggling, and
  the price is rounded up. One quest takes Sister Hild's 60-gold cure down to 57 gold.
- It stops at **100% off**: after twenty quests the town's services are **free**. They are listed
  as `(free)`, you pay nothing, and a free service does not train
  [Negotiation](skills.md#skill-negotiation).
- Only services get cheaper. [Shop](shops.md) goods keep their prices.

{{#include generated/towns.md}}

## The pedlar's offer

The **[pedlar](creatures.md#creature-pedlar)** met on the roads
([Encounters](encounters.md#the-ascmearcs-encounters)) carries one thing worth more than pins and
needles. He offers it for a fixed price: take it or leave it. Ask him about **Your wares**, listed
after his topics with the price. A dialog shows the item, its affixes in words, and the price,
with **Buy it** and **Leave it**. The item is already
[identified](magic-items.md#unidentified-items), since a pedlar knows what he sells.

- **What it is**: a [rare](magic-items.md#rarity) item of any kind loot holds, but no ammunition.
  It has **four affixes (half the time), five (35%), or six (15%)**, and is made at an
  [item level](magic-items.md#finding-them) of **your level plus four**, at least 4. Loot is rare
  one time in fifty, most often with three affixes, and at the depth's level, so his is much
  better.
- **One of three uniques**: one offer in fifty (2%) is instead one of the three
  [unique items](magic-items.md#unique-items) only pedlars carry: the Brosinga Mene, the
  Eorcanstān, and Grendles Grāpe. Each is offered at most once a run. Once all three have been
  offered, his offers are rare finds again.
- **The price** is fixed and never haggled ([Presence](stats.md#stat-presence) and
  [Negotiation](skills.md#skill-negotiation) do not move it). It is the item's value plus
  **15 [gold](shops.md#money) per affix per item level** (four affixes at item level 5: its value
  and 300 gold), or **1500 gold** for a unique.
- **Buying** pays him, puts the item in your pack, and takes the time of a chat. It is a deal, so
  it trains Negotiation and [Presence](stats.md#stat-presence) as trade does. Without enough gold
  the dialog only lets you leave it, and you are told "You cannot pay." Leaving it costs nothing,
  and he keeps it for you: ask again while he is near.
- **One offer per pedlar**: once you have bought it he has nothing more ("That was the only one
  I 'ad"). The same pedlar met again on the road still has his. Another pedlar brings another
  offer, made when you first talk to him.

## Muttering

Now and then, while they go about their business and you are within a few cells, people mutter to
themselves. The townsfolk grumble about prices and the ash,
[Sister Hild](creatures.md#creature-sister) prays, and [Godgifu](creatures.md#creature-innkeeper)
hums over her work.

You hear it like any other sound (see [Hearing](hearing.md)). If you see them, it comes with their
name (`The villager mutters: "Ash in the well again."`). If you do not, it is a voice from
somewhere (`You hear a voice to the north: "…"`), and if it was faint, you do not catch the words.
Muttering never wakes you from a [rest](resting.md).

## Getting past

You never have to wait for someone to move out of your way. Walking into a creature that is not
[hostile](creatures.md#factions) to you swaps places with it: you step onto its cell, and it steps
onto yours ("You swap places with the deer."). That takes your step's time, and it does not cost
the creature its own turn.

It lands on your cell as any step lands. A [trap](traps.md) you stood on goes off at it, and
[ice](temperature.md#walking-on-ice) that held you may not hold something heavier. Afterwards it
walks back to where it stood (waiting beside the spot while you are on it) and carries on with
whatever it was doing.

A few will not make way. Then the move takes no time, and the message says why:

- Some never budge, and some keep their post: a [shopkeeper](creatures.md#creature-shopkeeper)
  keeps to the door of their shop ("Eadric doesn't budge."). A character trained in
  [Escape artist](skills.md#skill-escape-artist) can [wriggle past](#getting-past) most of these,
  but never a shop's keeper.
- Someone who cannot move at all, under too heavy a [load](burden.md), does not budge either.
- A creature two sizes or more larger than you is too big to squeeze past ("The giant is too big
  to squeeze past."). No creature in the game is that big yet.

**Wriggling past.** If you are trained in [Escape artist](skills.md#skill-escape-artist), someone
who keeps their post or never budges does not stop you. The step becomes a check, with no action
of its own. You roll your [Dexterity](stats.md#stat-dexterity) modifier plus the skill's bonus
against 11 plus their [Willpower](stats.md#stat-willpower) modifier. The bonus is the same as for
slipping [bonds](held.md#slipping-bonds): +8 at 100, +12 at 200. If you make it, you swap
places ("You wriggle past Eadric."). If you fail, the step's time is spent and you stay where you
are. Each try may train the skill (half the time).

Wriggling does not help with a [shop's](shops.md) keeper, who never lets you by, with someone who
cannot move at all, with a creature too big to squeeze past, or while swimming.

If you cannot see what is in your way, you only learn that "Something is in the way.", or that you
swapped places with something.

Moving into a hostile creature attacks it instead; see [Combat](combat.md).

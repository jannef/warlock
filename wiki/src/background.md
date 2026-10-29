# Your story

Every [character](character.md) you make has a youth behind them. When the run begins, fate
tells it: a short story in four paragraphs, one sentence for each **chapter** of a life. Nothing
in it is chosen, and there is no way back from it: it is told after every choice you made in
character creation, and it may change what you start with.

## When it is told

The story is shown once, as the run begins: right after you start the run from the summary of
[character creation](character.md), it opens over the first turn, before you can move. Nothing
on it can be chosen; `Enter` (or `Esc`) goes on into the run, and the arrow keys scroll it on a
small window. Afterwards it is page 8 of the [info pages](controls.md#info-pages), "Story": open
any info page and press `8`. A character who has no story (the start of a scenario played as its
own character, or a character saved before stories were told) reads "Your story is not written."
there. A run you continue is never shown it again, and nothing on the death screen repeats it.

The chapters, in order:

| Chapter | What it tells |
|---|---|
| Parents | who raised you, and how they lived |
| Birth | how you came into the world (your [birth sign](horoscope.md) says when) |
| Childhood | what kind of child you were |
| Youth | a trade, a teacher, a levy |
| The trial | what it cost you, and what you kept |
| Fortune | what was given you |
| Leaving | why you left |
| The road | how you came to where the run begins: [the Ascmearc](ascmearc.md)'s own chapter |

Each sentence fits the character it is told of: a line may ask for a [race](races.md), a
[class](classes.md), a gender, a birth animal or element, a [lucky day](horoscope.md#lucky-days),
your starting [alignment](alignment.md), the [talents](talents.md) and [skills](skills.md) you
start with, your age, and what the story has already said (a foundling's leaving is not a
quarrel with their parents).

## What a story gives

A sentence may give or take:

- **stat points**: the score and its [potential](stats.md#potentials) move together, one or two
  points of a [stat](stats.md) a sentence; a score never falls below 1;
- **gold**: a purse, or (taken) never below none;
- **an item**: as the [starting kit](starting-kit.md) gives one, worn or wielded if a slot is
  free; a [magic item](magic-items.md) from a story is already [identified](magic-items.md#unidentified-items)
  (you grew up with it), and a [scroll](scrolls.md) from it is known;
- **a skill**: a start in a [skill](skills.md), or more of one you have;
- **marks** in a [weapon skill](weapon-skills.md#marks-and-ranks), never a rank;
- a nudge to your starting [alignment](alignment.md).

Nothing limits what a story gives or takes in all: a kind story may give a lot, and a grim one
may take a lot, and nothing protects you from either. The game tells you the story in words
only; the scores, the pack, and the skills show what it did.

The [heritage talents](talents.md#heritage-only-at-character-creation) are not paid twice: with
[Family wealth](talents.md#talent-family-wealth) your parents are always the old money the talent
gave you, and the story gives no more gold for it; with
[Father's sword](talents.md#talent-fathers-sword) the story tells of the blade you carry, and
gives no other heirloom.

## How rare

Each chapter is drawn in two steps: first how rare its sentence is (its **kind**), then a
sentence of that kind that fits you, by the weights below (if none fits, a sentence of the next
commoner kind; every chapter has a plain common sentence that fits anyone). The kinds:

- **Common**: the plain sentences, the small turns, the griefs with something kept;
- **Uncommon**: a gift, perhaps at a price;
- **Rare**: a thing people tell; always good news;
- **Fateful**: once in a life.

The odds of each kind, for every chapter as it is drawn (the expected counts are for a story in
[the Ascmearc](ascmearc.md), whose road is the eighth chapter), and what a sentence of each kind
is worth:

{{#include generated/background-tiers.md}}

What a sentence gives is weighed in points, and a sentence's worth (what it gives less what it
takes) must lie in its kind's range above. Nothing adds up a whole story's points: they only keep
each sentence true to its kind.

{{#include generated/background-points.md}}

Only a character you make is told a story; a run that is resumed keeps the story it had.

## Talents

Three [talents](talents.md) touch your story. Two are [heritage](talents.md#heritage-only-at-character-creation)
talents, offered only at character creation, since the story is told as the run begins and
nothing later changes it: [Favoured child](talents.md#talent-favoured-child) doubles the odds of
the luckier kinds (the "With Favoured child" column above), and [Hard-won](talents.md#talent-hard-won)
turns each [stat](stats.md) point a sentence takes into [stat training marks](stats.md#training).
The third, [Storyteller](talents.md#talent-storyteller), is picked at a level like any other: your
favour in [shop prices](shops.md#prices) rises, the more for each rare or fateful sentence in your story.

{{#include generated/background-talents.md}}

## Every sentence

Every sentence of every chapter, with what it asks for, what it gives, and what it leaves for
the chapters after it. A sentence's **weight** is its share among the sentences of its kind that
fit: a weight of 4 is told twice as often as a weight of 2. **Tags** are what the story
remembers: a later sentence may ask for one ("after `orphan`") or refuse one ("never after
`deep`"). Some sentences also choose words a later one uses: `{guardian}` (who raised you: "your
family" until a sentence says otherwise), `{home}` ("home"), and `{trade}` ("your trade").

Dice are written as the game rolls them: 2d4 is two four-sided dice added together. A skill's
dice start the [skill](skills.md) if you were not trained in it, or raise it if you were.
[Weapon marks](weapon-skills.md#marks-and-ranks) bring you toward your next rank, never past it.

### Parents

{{#include generated/background-parents.md}}

### Birth

{{#include generated/background-birth.md}}

### Childhood

{{#include generated/background-childhood.md}}

### Youth

{{#include generated/background-youth.md}}

### The trial

{{#include generated/background-trial.md}}

### Fortune

{{#include generated/background-fortune.md}}

### Leaving

{{#include generated/background-leaving.md}}

### The road

[The Ascmearc](ascmearc.md)'s own chapter: how you came over the pass.

{{#include generated/background-road.md}}

# Allies

An **ally** is a creature on your side: it follows you, fights what you are hostile to, and comes
along when you move on. Allies are **summons**, called by a spell for a while, such as the
[druid](classes.md#druid)'s wolves, and **[companions](#companions)**, beasts that are yours for
good, such as the [ranger](classes.md#ranger)'s hound.

An ally is an ordinary creature of its kind, with its own eyes, speed, attacks, and hit points. It
fights only what *you* are hostile to: with [Beast-friend](talents.md#talent-beast-friend) your
wolf leaves the rats alone as you do, and a shopkeeper or someone you are talking to is never its
idea of a foe. It gets in no one's way for long: you swap places with it as with anyone friendly.
The messages call it yours ("Your wolf bites the kobold.", "Your wolf dies."), and a faint mark
under its letter on the map tells it from the creatures around it.

## Orders

Three orders, in the Allies group of the [abilities menu](abilities.md), go to every ally of
yours on the level at once. None takes time, but each is your voice, and others hear it.

- **Come to heel**: follow you and fight what threatens you. This is what an ally does unless
  told otherwise; it heels again by itself when a fight would take it too far from you.
- **Hold here**: keep its place, fighting only what comes close.
- **Set on**: go to a place you can see and fight what is there, then come back to heel.

There is no order to send an ally away: a summon goes when its time runs out, or when it is
slain.

## Following you

Allies close to you when you take stairs or leave an area come along and appear beside you;
those farther away stay where they were, and wait (your [companion](#companions) comes along
wherever it stands). On the overworld they travel with you unseen and step out beside you at
the next place. A summon's time runs on wherever it is: one left behind, or on the road too long,
is gone when you next look for it.

## Experience

An ally's kill is yours: it counts for your [quests](quests.md) in full, and builds what a kill
of yours builds. It pays you **half** the experience your own blow would, but the classes that
live with beasts, the [ranger](classes.md#ranger) and the [druid](classes.md#druid), are paid in
full (see [The rules](#the-rules)). The share goes by your class, whatever brought the ally.
The ally itself gains nothing.

If you are trained in [Heal](skills.md#skill-heal), you can tend an ally beside you as you tend
yourself (see [Treating yourself](skills.md#treating-yourself)).

## Summons

A summon spell calls creatures to stand beside you for some minutes; a stronger
[Presence](stats.md#stat-presence) keeps them longer. Each spell keeps only so many of its own
at once: call again past that, and the oldest gives way to the new. You cannot lead more than a
few allies at once on one level, whatever brought them. A summon leaves nothing when it goes or
dies: no corpse, nothing it carried.

Your own spells that harm an area hurt your allies in it too.

{{#include generated/summon-spells.md}}

"Learning" is what the spell adds to the difficulty of [learning it from its
scroll](magic.md#learning-spells-from-scrolls): the calls are very hard to learn, even for a
caster well above their level.

### The druid's calls

The druid calls the wild: [Call of the wild](magic.md#spell-call-of-the-wild) at level 6,
[Call of the forest](magic.md#spell-call-of-the-forest) at 25, and [Call the
elements](magic.md#spell-call-the-elements) at 40, whose element the wild chooses. Each keeps one
of its own beside you until [Pack leader](talents.md#talent-pack-leader) (level 32): from then
every call brings one more, keeps two of its own, and lasts half again as long. [Lord of the
wild](talents.md#talent-lord-of-the-wild) (level 60) keeps three of each, and what you call
stays until it is slain. [Beast-bond](talents.md#talent-beast-bond), a Wayfarer talent for those
with some [Handle animal](skills.md#skill-handle-animal), makes every ally of yours strike surer
and take less harm.

## Companions

A **companion** is a beast that is yours for good: it stays until it is slain, and there is no
sending it away. You have at most one at a time. It is an ally like any other, with the same
[orders](#orders), the same side, and the same share of its kills, and the messages call it by
its **name**, drawn when it becomes yours ("Snel bites the kobold.", "Snel comes to heel.",
"Snel dies."). The [status bar](controls.md) says how it fares ("Snel: unhurt", or "Snel: with
you" while it travels with you on the overworld), [look mode](controls.md#look-mode) names it
("Snel, your hound"), and the Character screen lists it under your class.

A companion **always comes along**: wherever it stands when you take the stairs or leave an area,
held fast or holding its place or not, it arrives beside you ("Snel comes with you."), rides with
you unseen across the overworld, and steps out beside you at the next place ("Snel follows you
in."). Nothing but death keeps it from you. When it dies it leaves a corpse like any creature,
and nothing else happens to you: find another to tame.

Beasts that hunt by scent have a **nose**: within so many cells (see the Nose column of
[Creatures in numbers](creature-numbers.md#pace-senses-and-size)) your companion smells a foe of
yours however dark it is, and **through a closed door**, though never through a wall, and goes
for it when its eyes find nothing. When it scents something you cannot see, it growls, once
("Snel growls at the dark."). A door it cannot open keeps it from what it smells: open it.

### Starting companions

The [ranger](classes.md#ranger)'s **Animal companion** (level 1) means a beast at their side from
the first turn: one of their race's kinds, drawn by chance as the run begins, or rarely one of
the rares that any race may begin with. The [story](background.md#the-companion) of how it came
to be yours is told with the rest of your background.

{{#include generated/companion-starts.md}}

### Taming

Anyone trained in [Handle animal](skills.md#skill-handle-animal) may **tame** a beast, at any
level: choose **Tame** in the [abilities menu](abilities.md) and the direction of a beast next to
you. Only some kinds can be tamed (the Tameable column of
[Creatures in numbers](creature-numbers.md#pace-senses-and-size)), and only one that is not your
foe (wildlife, or an animal a [Beast-friend](talents.md#talent-beast-friend) has calmed) or that
has not seen you (asleep, say). You crouch beside it for a while, holding out food if you carry
any; if you are hurt, or it leaves your side, or someone comes into view, you stop, and nothing
is lost but the time. Then one check decides: made, it comes to your hand and is your companion,
named, at your level; failed, it snaps at you and backs away, and a foe among beasts has seen
you now. Try again as often as you like: time is the only price. While you have a companion you
cannot tame another. The ranger's Animal companion makes them a little better at it. The exact
check and the chances:

{{#include generated/companion-taming.md}}

### Growth

A companion's level is **yours**: whatever its kind's own level, it is raised to your level when
it becomes yours and every time you gain a level ("Snel grows stronger."), and never falls. Above
its kind's level it gains hit points every level, and now and then a surer bite, more damage and
another die of it, natural armour, tougher hide (its body's Protection at its level, as the
[hide rule](creatures.md#hides-and-bodies) gives every creature), and Strength, Dexterity, and
Toughness. Its speed, senses, size, and resistances stay its kind's. Summons do not grow: their
power is their spell's. The exact table, as the game computes it:

{{#include generated/companion-growth.md}}

### The companion's rules

{{#include generated/companion-rules.md}}

## The rules

{{#include generated/ally-rules.md}}

# Allies

An **ally** is a creature on your side: it follows you, fights what you are hostile to, and comes
along when you move on. Allies are **summons**, called by a spell for a while, such as the
[druid](classes.md#druid)'s wolves, and **[companions](companions.md#companions)**, beasts that are yours for
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
those farther away stay where they were, and wait (your [companion](companions.md#companions) comes along
wherever it stands). On the overworld they travel with you unseen and step out beside you at
the next place. A summon's time runs on wherever it is: one left behind, or on the road too long,
is gone when you next look for it. Your allies find their places first when [hostile creatures
follow you on the stairs](terrain.md#stairs).

## Experience

An ally's kill is yours: it counts for your [quests](quests.md) in full, and builds what a kill
of yours builds. It pays you **half** the experience your own blow would, but the classes that
live with beasts, the [ranger](classes.md#ranger) and the [druid](classes.md#druid), are paid in
full (see [The rules](#the-rules)). The share goes by your class, whatever brought the ally.
The ally itself gains nothing.

If you are trained in [Heal](skills.md#skill-heal), you can tend an ally beside you as you tend
yourself (see [Treating yourself](skill-uses.md#treating-yourself)).

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

### The risen

[Rouse the dead](magic.md#spell-rouse-the-dead) is aimed at a [corpse](food.md#corpses) you see, of
any age. The corpse is used up and **what it was** stands where it lay, or on the nearest free
cell: an ogre's corpse makes an ogre. It keeps everything of its kind, its
[stats](creature-numbers.md), its natural attacks and armour, what it resists, and any
[creature tag](creature-numbers.md) it had, but stands at a share of its hit points, with no gear
and no pack, and has no name and no trade: a shopkeeper's corpse rises a mute, empty-handed body.
A [mimic](mimics.md)'s corpse rises as a mimic, disguised as nothing. It serves as a summon does,
and when it falls again or its time is up it leaves nothing, so nothing rises twice. The corpse of
a unique creature will not rise for you. Whatever leaves no corpse never comes to this.

Now and then it rises **berserk**: "The dead thing stands, and it knows no one." It joins
[the berserk](creatures.md#factions), friendly to nothing: it attacks what it sees, you, your
allies, the peaceful, and another berserk alike. It takes no orders and counts toward neither its
spell's limit nor how many you may lead. Its time runs as a servant's would, however long your
[talents](talents.md) would keep a summon. It leaves nothing when it falls, and killing it pays
as killing any of its kind.

{{#include generated/rise-spells.md}}

### The charmed

[Beguile](magic.md#spell-beguile) is cast at a hostile creature you see: there is no check and no
limit by level, and it takes. "The ogre takes you for a friend." From then it is your ally, by
every rule above: it follows you, takes your [orders](#orders), pays you for its kills as any
ally does, and its own band turns on it. When the charm's minutes are up it is itself again,
hostile, wherever it stands ("The ogre shakes off the charm."). It never follows you to another
level: the moment you leave, the charm ends, and it stays behind as it was. It keeps its gear, and
when it dies it leaves its corpse and what it carried, as any of its kind.

It never takes on a unique creature or anyone with a name and a trade, on an
[animated thing](animated.md), or on what has no mind: the undead, constructs, and vermin, and
whatever has risen [berserk](#the-risen). Nor on one already at someone's side, and not past how
many allies you may lead.

{{#include generated/charm-spells.md}}

### Escorts

Someone you bring home for an [escort quest](quests.md#kinds-of-quests) is an ally too, from the
moment they agree to come: they follow you, come along on the stairs and travel with you over
the overworld, and come to heel or hold as told, but they are never set on anything and strike
only at what stands right beside them. [Give](abilities.md#ability-give) hands them any weapon
or armour they can use, which they put on at once, dropping what they had there. Home, they are
no one's ally any more and stay where they are.

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

## The rules

{{#include generated/ally-rules.md}}

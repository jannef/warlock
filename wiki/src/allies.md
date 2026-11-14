# Allies

An **ally** is a creature on your side. It follows you, fights what you are hostile to, and comes
along when you move on. Allies come in two kinds:

- **Summons** are called by a spell for a while, such as the [druid](classes.md#druid)'s wolves.
- **[Companions](companions.md#companions)** are beasts that are yours for good, such as the
  [ranger](classes.md#ranger)'s hound.

The witch's beast, her [familiar](familiars.md#familiars), is a kept beast of its own kind.
[Companions](companions.md) and [Familiars](familiars.md) have pages of their own. This page covers
what every ally shares, and the summons.

An ally is an ordinary creature of its kind, with its own eyes, speed, attacks, and hit points. It
fights only what *you* are hostile to. With [Beast-friend](talents.md#talent-beast-friend), your
wolf leaves the rats alone as you do, and a shopkeeper or someone you are talking to is never its
idea of a foe. It never blocks your way for long: you swap places with it as with anyone friendly.

The messages call it yours ("Your wolf bites the kobold.", "Your wolf dies."). A faint mark under
its letter on the map tells it apart from the creatures around it.

## Orders

Three orders, in the Allies group of the [abilities menu](abilities.md), go to every ally of yours
on the level at once. None takes time, but each is your voice, and others hear it.

- **Come to heel**: follow you and fight what threatens you. This is what an ally does unless told
  otherwise. It heels again by itself when a fight would take it too far from you.
- **Hold here**: keep its place, fighting only what comes close.
- **Set on**: go to a place you can see and fight what is there, then come back to heel.

There is no order to send an ally away. A summon goes when its time runs out, or when it is slain.

## Following you

When you take stairs or leave an area, allies close to you come along and appear beside you.
Those farther away stay where they were and wait. Your [companion](companions.md#companions) comes
along wherever it stands.

On the overworld your allies travel with you unseen, and step out beside you at the next place. A
summon's time runs on wherever it is: one left behind, or on the road too long, is gone when you
next look for it. When [hostile creatures follow you on the stairs](terrain.md#stairs), your allies
find their places first.

## Experience

An ally's kill is yours. It counts in full for your [quests](quests.md), and builds what a kill of
yours builds. It pays you **half** the experience your own blow would. The classes that live with
beasts, the [ranger](classes.md#ranger) and the [druid](classes.md#druid), are paid in full (see
[The rules](#the-rules)). The share goes by your class, whatever brought the ally. The ally itself
gains nothing.

If you are trained in [Heal](skills.md#skill-heal), you can tend an ally beside you as you tend
yourself (see [Treating yourself](skill-uses.md#treating-yourself)).

## Summons

A summon spell calls creatures to stand beside you for some minutes. A stronger
[Presence](stats.md#stat-presence) keeps them longer. Each spell keeps only so many of its own at
once; call again past that, and the oldest gives way to the new. You cannot lead more than a few
allies at once on one level, whatever brought them. A summon leaves nothing when it goes or dies:
no corpse, and nothing it carried.

Your own spells that harm an area hurt your allies in it too.

{{#include generated/summon-spells.md}}

"Learning" is what the spell adds to the difficulty of
[learning it from its grimoire](magic.md#learning-spells-from-grimoires). The calls are very hard to
learn, even for a caster well above their level.

### The risen

[Rouse the dead](magic.md#spell-rouse-the-dead) is aimed at a [corpse](food.md#corpses) you see,
of any age. The corpse is used up, and **what it was** stands where it lay, or on the nearest free
cell. An ogre's corpse makes an ogre.

It keeps everything of its kind: its [stats](creature-numbers.md), its natural attacks and armour,
what it resists, and any [creature tag](creature-numbers.md) it had. But it stands at a share of its
hit points, with no gear and no pack, and has no name and no trade. A shopkeeper's corpse rises as
a mute, empty-handed body. A [mimic](mimics.md)'s corpse rises as a mimic, disguised as nothing.

It serves as a summon does. When it falls again or its time is up it leaves nothing, so nothing
rises twice. The corpse of a unique creature will not rise for you, and whatever leaves no corpse
never comes to this.

Now and then it rises **berserk**: "The dead thing stands, and it knows no one." It joins
[the berserk](creatures.md#factions) and is friendly to nothing. It attacks what it sees: you, your
allies, the peaceful, and another berserk alike. It takes no orders, and counts neither toward its
spell's limit nor toward how many you may lead. Its time runs as a servant's would, however long
your [talents](talents.md) would keep a summon. It leaves nothing when it falls, and killing it pays
as killing any of its kind.

{{#include generated/rise-spells.md}}

### The charmed

[Beguile](magic.md#spell-beguile) is cast at a hostile creature you see. There is no check and no
limit by level: it always takes. "The ogre takes you for a friend." From then on it is your ally by
every rule above. It follows you, takes your [orders](#orders), and pays you for its kills as any
ally does, and its own band turns on it.

When the charm's minutes are up it is itself again, hostile, wherever it stands ("The ogre shakes
off the charm."). It never follows you to another level: the moment you leave, the charm ends and
it stays behind as it was. It keeps its gear, and when it dies it leaves its corpse and what it
carried, as any of its kind.

The charm never takes on:

- a unique creature, or anyone with a name and a trade;
- an [animated thing](animated.md);
- what has no mind: the undead, constructs, vermin, and whatever has risen
  [berserk](#the-risen);
- a creature already at someone's side;
- any creature past how many allies you may lead.

{{#include generated/charm-spells.md}}

### Escorts

Someone you bring home for an [escort quest](quests.md#kinds-of-quests) is an ally too, from the
moment they agree to come. They follow you, come along on the stairs, travel with you over the
overworld, and come to heel or hold as told. But they are never set on anything, and they strike
only at what stands right beside them.

[Give](abilities.md#ability-give) hands them any weapon or armour they can use. They put it on at
once and drop what they had there. Once home, they are no one's ally any more and stay where they
are.

### The druid's calls

The druid calls the wild:

- [Call of the wild](magic.md#spell-call-of-the-wild) at level 6;
- [Call of the forest](magic.md#spell-call-of-the-forest) at 25;
- [Call the elements](magic.md#spell-call-the-elements) at 40, whose element the wild chooses.

Each call keeps one of its own beside you. From [Pack leader](classes.md#talent-pack-leader) (level
32), every call brings one more, keeps two of its own, and lasts half again as long.
[Lord of the wild](classes.md#talent-lord-of-the-wild) (level 60) keeps three of each, and what you
call stays until it is slain.

[Beast-bond](talents.md#talent-beast-bond), a Wayfarer talent for those with some
[Handle animal](skills.md#skill-handle-animal), makes every ally of yours strike surer and take less
harm.

## The rules

{{#include generated/ally-rules.md}}

# Teeming dens

A teeming den is a [special room](special-rooms.md) full of sleeping creatures lying among their
hoard. Creep past and they sleep on. Wake one and the whole den may be up.

A den comes only on a level with a danger (see [Dungeons and danger](dungeons.md)), in a room of
16 floor cells or more. Its sleepers are the level's own kind of creatures. Each is drawn just as
the level's other creatures are: by the level's danger and how rare each kind is, with now and
then a weaker stray or one from deeper down. They are always [hostile](creatures.md#factions) to
you and never unique. Sometimes the whole den is of one kind (a *warren*, such as a nest of rats)
or of one [faction](creatures.md#factions) (a *tribe*, such as a goblin camp). Their hoard is a few
[items](objects.md) a little better than the level's own.

A *[beast den](special-rooms.md#room-beast-den)* holds only animals and fen things. Its sleepers
are drawn the same way, but only among the wildlife, the vermin, and the marsh things, whether
they are [hostile](creatures.md#factions) to you or not. A dungeon with none of these among its
creatures never has one. Its hoard is no treasure, only what the beasts dragged in: piles of bones
and fresh [corpses](food.md#corpses) of the level's creatures. These are good to
[eat](food.md#eating) when you first come to the level, and [rot](food.md#food-spoils) from then
on.

{{#include generated/teeming-dens.md}}

"Wakes at" is how loud a noise must be where a sleeper lies, in percent of the loudness it can
just hear (see [Hearing](hearing.md)). Sleepers hear a whisper but wake only to twice that. They
never wake by themselves, however long you wait or rest nearby. They wake when:

- a [noise](hearing.md#what-makes-noise) is loud enough where they lie: your footsteps, a door, a
  fight, a kick, [digging](digging.md#digging);
- something hurts them: a blow, a missile, a [spell](magic.md#the-spells), even
  [poison](status-effects.md#status-poisoned);
- a neighbour wakes. Every sleeper that wakes cries out at once ("Cry" is its loudness). The cry
  may wake those near it, and their cries the next, until the whole den is up.

[Light](light.md#what-light-is) does not wake them, so carrying your [lantern](lantern.md) in is
safe. Your plain footsteps (60) wake a kobold, which hears 15 and so wakes at 30, up to three
cells away. With some [Stealth](skills.md#skill-stealth) you can walk right past.

A sleeper has seen no one, so [sneak attacks](class-thief.md#talent-sneak-attack) land on it every
time. But every blow is noise, and a kill is a fight its neighbours may hear. A sleeper you see
shows as "asleep" in [look mode](controls-prompts.md#look-mode), and you are told when one wakes.

Sleepers count among the level's creatures. While the den is full, fewer newcomers find their way
to the level. Killing a sleeper is like killing any other foe.

## Homes of animated things

Two dens have creatures of their own, whatever the dungeon:

- An **[armoury](special-rooms.md#room-armoury)** sleeps [animated](animated.md) blades, hollow
  armour, and panoplies among weapon racks (`‡`) along its walls. They are drawn as if the level
  were four [dangers](dungeons.md#who-lives-there-and-how-many) deeper.
- A **[workshop](special-rooms.md#room-workshop)** sleeps the things of a house's work among
  workbenches (`π`): bellows, brooms, buckets, chairs, coat stands, cooking pots, kettles, ladles,
  stools, and warming pans.

You can see over racks and benches but not walk through them. They never stand beside a door or
block the way across the room. Each room comes only where its creatures can be met at the level's
[danger](dungeons.md#who-lives-there-and-how-many): an armoury from danger 1, a workshop no deeper
than danger 16. Animated things never stray, so outside the dungeons that keep them these rooms are
where you find them (see [Animated objects](animated.md#where-they-are-found)).

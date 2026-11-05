# Terrain

Every cell of the map has a terrain. It decides whether you can walk there, [how long a step takes](turns.md#what-actions-cost),
how [loud](hearing.md#how-loud) your steps are, whether it blocks [sight](sight.md#line-of-sight) and [sound](hearing.md), and how tall it stands for
[shadows](sun.md). The overworld has its own terrain, described on [its page](overworld.md).

## Doors

A closed door (`+`) blocks movement, [sight](sight.md#line-of-sight), and [light](light.md#what-light-is), and muffles [sound](hearing.md): you hear only what
happens close behind it. Walk into it to open it, which takes as long as opening it (6 seconds),
or press `o` and a direction. An open door (`'`) lets you, your light, and sound through. Press
`c` and a direction to close it again; you cannot close a door while something stands or lies in the doorway.
Opening and closing doors makes noise (see [Hearing](hearing.md)). Some creatures open doors too;
none closes them (see [Creatures](creatures.md)). Some doors are [locked](doors.md#locked-doors), and every closed door
can be [kicked](doors.md#kicking) in with `k`, leaving a [broken door](doors.md#kicking) (`,`) that stays open for good: see
[Doors](doors.md).

The game remembers a door as you last saw it: one you saw open stays drawn open until you see it
closed.

## Stairs

Stairs (`>` down, `<` up) join levels. Step onto them, then press `>` or `<` to use them. Using
stairs is a normal action (6 seconds). Each level you visit is kept as you left it, and [dungeon](dungeons.md) levels are generated
on your first visit. Levels you leave and return to are exactly as they were.

Stairs are no free escape. A [hostile](creatures.md#factions) creature standing right beside you
that knows you are there comes along: it arrives beside you and goes on hunting you, with the
[wounds](combat.md) and [status effects](status-effects.md#how-they-work) it had. It pays for the
climb as you do, so at the same [pace](creatures.md#pace-and-levels) you get to act first. It does
not come if it is:

- [asleep](status-effects.md#status-asleep), [held](status-effects.md#status-held),
  [webbed](status-effects.md#status-webbed), [pinned](status-effects.md#status-pinned),
  [stunned](status-effects.md#status-stunned), [surprised](status-effects.md#status-surprised), or
  [frightened](status-effects.md#status-frightened);
- [slipped past](skills.md#slipping-past) or otherwise unaware of you;
- a [mimic](mimics.md), a creature that [lurks](creatures.md#what-they-do) and has not been hurt,
  one rooted to its place, a [shopkeeper](shops.md), or someone you can [talk](npcs.md) to by name;
- out of room: only the cells right beside where you arrive will do, your
  [allies](allies.md#following-you) take theirs first, and it never stands in a
  [sanctuary](special-rooms.md#room-sanctuary). If there is no cell left, it stays where it was.

Only stairs do this, and only stairs between levels: nothing follows you out to the
[overworld](overworld.md), or when the floor gives way under you. A creature that
followed you counts for a [kill quest](quests.md) as if it died where it came from.

## Walls

Walls block movement, [sight](sight.md#line-of-sight), [light](light.md#what-light-is), and [sound](hearing.md). Most of them can be dug through with a [pick-axe](digging.md#digging),
slowly, and a big [explosion](digging.md#explosions) blows the softer ones apart; a broken wall becomes [rubble](#terrain-rubble). The edge of
a level never breaks. See [Digging and walls](digging.md).

A [worn statue](#terrain-statue) stands in the way as a wall does, but you see past it. Step up
beside one and you are told what it is, and what is cut into it, if anything: some carry words
in the old tongue worth reading (one deep in Fyrdelf keeps an oath; see
[Prestige classes](classes.md#prestige-classes)).

## Water

Still water (ponds and pools) and running water (streams) are both wadeable but slow: a step into
either takes twice as long as a step on dry ground, and [splashes loudly](hearing.md#how-loud). Running water flows in a
direction you can see in the ripples, and foams where it meets a bank or rock. The current does
not push you yet. Water glints with nearby [light](light.md#what-light-is). [Swim](skills.md#skill-swim) spares some of the extra time of
wading, here and in a marsh.

[Shallow water](#terrain-shallow-water), ankle-deep over a stone floor, covers the floor of a [flooded room](special-rooms.md#room-flooded) (see
[Special rooms](special-rooms.md)). It is paler than a pool. Wading through it is slower than
walking, though quicker than wading a pool, and it splashes just as loudly.

### Deep water

[Deep water](#terrain-deep-water) is too deep to wade. Only one trained in [Swim](skills.md#skill-swim) may go in, and
never [Strained](burden.md) or worse: "You cannot swim." and "You would sink." take no time. The game always
asks first, whenever you step into it from anything else, and no setting turns the question off;
staying back takes no time. Water you did not know was there (in the dark) stops you at its edge
and tells you so, also taking no time, so that the next step can ask.

Going in takes a step's time, twice a plain step's and as [loud](hearing.md#how-loud) as any water, and needs no check.
It puts out your ready [lantern](light.md#what-light-is), and no light can be lit while you swim. Spells' light and everything else goes on.

Every step that *starts* in the water, the step out onto the shore too, is a [check](skills.md#skill-swim):
2d12 plus your [Toughness](stats.md#stat-toughness) modifier, your talents' Toughness checks, and Swim's points (the
skills table's Swim check), against 11, and 2 more for each level of [burden](burden.md) above Unburdened. Made,
you take the step, at a price that wading's discounts apply to ([Swim](skills.md#skill-swim), the
[Swimmer](talents.md#talent-swimmer), [Strong swimmer](talents.md#talent-strong-swimmer), and [Undertow](talents.md#talent-undertow) talents). Failed, you spend the turn treading water, you do not move, and the
cold takes 1 hit point, resisted as any [cold](damage-types.md), but never your last: nobody drowns. At even Toughness and no Swim points, that is 69 % unburdened, 54 %
Burdened; at +4, 90 % and 81 %. Waiting asks no check.

In the water you cannot drop or pick up anything ("It would sink."), rest, search, or dig; all of that is refused and takes no
time. You can still fight, shoot, cast, read, and wait, and you lose nothing you carry. Your burden is all it asks.

No creature goes in. They walk around it, wait on the shore nearest you, shoot and cast from there, and strike you if
you swim beside them. Your allies wait ashore too, and none will swap places with you into the water.
It never freezes.

Deep water is rarer than the pools of shallow water. About one cave floor in six has a pool with a deep middle (the shore of it stays shallow), and the middle of a lake beside a wilderness area is deep. Ponds, rivers, and brooks are always wadeable, and no hand-made level, vault, or special room has deep water.

## Ice

Below [freezing](temperature.md), water grows ice: [thin ice](temperature.md#walking-on-ice), ice, and thick ice, by how thick it is. You walk on it
like dry ground, a little more carefully, but it holds only so much weight: thin ice gives way
under a person, and ice may under a heavy pack. [Falling through](temperature.md#falling-in) hurts a little, and climbing back
out takes a few turns. See [Temperature and ice](temperature.md).

## Plants and rock outdoors

[Trees](#terrain-tree), [pines](#terrain-pine), [dead trees](#terrain-dead-tree), [boulders](#terrain-boulder), crags (rock outcrops), and [standing stones](#terrain-standing-stone) block your way,
and all but dead trees block [sight](sight.md#line-of-sight) too. [Bushes](#terrain-bush) can be pushed through but slowly. [Reeds](#terrain-reeds), [marsh](#terrain-marsh), and
[rubble](#terrain-rubble) slow you a little. Their heights set how long their [shadows](sun.md) are.

Anything you can neither walk on nor see through blocks [sound](hearing.md), except a [closed door](doors.md), which only
muffles it: walls, solid rock, and the trees, boulders, crags, and stones above. Sound has to go
around them. [Water](#water), [bushes](#terrain-bush), [reeds](#terrain-reeds), [marsh](#terrain-marsh), [rubble](#terrain-rubble), [rocky ground](#terrain-rocky-ground), dry or tall grass, and creaking
boards make your steps louder; [grass](#terrain-grass), flowers, the [forest floor](#terrain-forest-floor), [dirt roads](#terrain-dirt-road), and [dusty floors](#terrain-dusty-floor)
soften them. See [Hearing](hearing.md).

## All terrain

Step times below are at a [normal pace](turns.md#what-actions-cost), which is yours: a normal step takes 6 seconds (see
[Turns and movement](turns.md)). "Walk on it" is yes for terrain you can enter, and for a [closed door](doors.md), which you open by walking into it.
"[Footsteps](hearing.md#what-makes-noise)" is how loud a step onto it is: quiet, normal, loud, or very loud.

{{#include generated/terrain-local.md}}

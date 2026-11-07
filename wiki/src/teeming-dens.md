# Teeming dens

A teeming den comes only on a level with a danger (see [Dungeons and danger](dungeons.md)), in a room
of 16 floor cells or more. Its sleepers are the level's own kind of creatures: each is drawn just
as the level's other creatures are (by the level's danger, how rare each kind is, now and then a
weaker stray or one from deeper down), only creatures [hostile](creatures.md#factions) to you and never a unique one.
Sometimes the whole den is of one kind (a *warren*: a nest of rats, say) or of one [faction](creatures.md#factions) (a
*tribe*: a goblin camp). They lie on the floor among their hoard, a few [items](objects.md) a little better
than the level's own.

A *[beast den](special-rooms.md#room-beast-den)* is a den of animals and fen things only: its sleepers are drawn the same way, but
only among the wildlife, the vermin, and the marsh things, whether they are [hostile](creatures.md#factions) to you or
not. A dungeon whose own creatures hold none of these never has one. Its hoard is no treasure
but what the beasts dragged in: piles of bones and fresh [corpses](food.md#corpses) of the level's creatures, good
to [eat](food.md#eating) when you first come to the level (they [rot](food.md#food-spoils) from then on).

{{#include generated/teeming-dens.md}}

"Wakes at" is how loud a noise must be where a sleeper lies, in percent of the loudness it can
just hear (see [Hearing](hearing.md)): sleepers hear a whisper but wake only to twice that.
Sleepers never wake by themselves, however long you wait or rest nearby. They wake when

- a [noise](hearing.md#what-makes-noise) is loud enough where they lie: your footsteps, a door, a fight, a kick, [digging](digging.md#digging);
- something hurts them, a blow, a missile, a [spell](magic.md#the-spells), even [poison](status-effects.md#status-poisoned); or
- a neighbour wakes: every sleeper that wakes cries out at once ("Cry" is its loudness), and the
  cry may wake those near it, and theirs the next, until the whole den is up.

[Light](light.md#what-light-is) does not wake them: carrying your [lantern](lantern.md) in is safe. Your plain footsteps (60) wake a
kobold (which hears 15, so wakes at 30) up to three cells away; with some [Stealth](skills.md#skill-stealth) you can walk
right past. A sleeper has seen no one, so *[sneak attacks](talents.md#talent-sneak-attack)* land on it every
time; but every blow is noise, and a kill is a fight its neighbours may hear. A sleeper you see
shows as "asleep" in [look mode](controls.md#look-mode), and you are told when one wakes.

Sleepers count among the level's creatures: while the den is full, fewer newcomers find their
way to the level. Killing a sleeper is like killing any other foe.

## Homes of animated things

Two dens have creatures of their own, whatever the dungeon's: an **[armoury](special-rooms.md#room-armoury)** sleeps
[animated](animated.md) blades, hollow armour, and panoplies, drawn as if the level were four
[dangers](dungeons.md#who-lives-there-and-how-many) deeper, among weapon racks (`‡`) along its walls; a **[workshop](special-rooms.md#room-workshop)** sleeps the household
things of a house's work (bellows, brooms, buckets, chairs, coat stands, cooking pots, kettles,
ladles, stools, and warming pans) among workbenches (`π`). Racks and benches are seen over but not
walked through, and never stand beside a door or bar the way across the room. Each comes only
where its creatures can be met at the level's [danger](dungeons.md#who-lives-there-and-how-many): an armoury from [danger](dungeons.md#who-lives-there-and-how-many) 1, a workshop no
deeper than danger 16. Animated things never stray, so these rooms are where they are found
outside the dungeons that keep them (see [Animated objects](animated.md#where-they-are-found)).

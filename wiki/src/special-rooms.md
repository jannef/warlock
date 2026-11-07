# Special rooms

Now and then a room of the dungeon is not like the others. As in ADOM, a **special room** changes
the rules while you are inside it: the [light](light.md#what-light-is), the [noise](hearing.md#what-makes-noise) you make, how fast you [heal](healing.md#how-fast), who dares to
follow you in, or the floor under your feet.

## Finding one

The first time you step into a special room, a message tells you what it is:

```
The light seems to drain away here.
```

From then on you know the room: [look mode](controls.md#look-mode) names it on any of its floor
you see or remember ("A stone floor in an echoing hall.", "You remember a stone floor here, in a
sanctuary."). Before you have been inside, it is just a room, though you may notice something
odd about it from the doorway: an unnatural darkness, a faint warm glow, [water](terrain.md#water) on the floor.
Some rooms give themselves away from outside: the first time you see any part of one, a line
hints at what lies there.

Special rooms are not terribly common, but not rare either: about three levels of a generated
[dungeon](dungeons.md) in five have one, and a level never has more than one (some [dungeons](dungeons.md) allow more). In the rooms-and-corridors levels it is a
whole room, walls and doors included, but never the room you arrive in; in the caverns it is a
patch of the cave, well away from where you arrive. Cēaptūn's Mynster is a
sanctuary.

## By dungeon

Each [dungeon](dungeons.md) may have odds of its own, and some of its levels other odds again: a dungeon may
be full of flooded rooms, or never have a [shop](shops.md). "A room is special" is the chance for each room
(each of eight tries in a cavern) until the level has as many as "At most a level"; the room
is then drawn: the rooms a dungeon favours keep their fixed share of the draws, a room it bans
never comes, and every other room comes by its weight (in the table of rooms below), so a dungeon
gets every kind of room it does not ban, the newest too. "A shop" is the chance a rooms-and-corridors level
has one, if one of its rooms suits (see [Shops](shops.md)).

{{#include generated/branch-odds.md}}

## Whole levels

Very rarely, a room's effect does not stay in a room: when a level draws a room that can, there
is a small chance (its "Whole level" chance below) that the effect covers the **whole level**
instead, and the room is plain. You are told each time you arrive on such a level, and the
effect holds everywhere on it. Where a room's effect and a whole level's meet, both apply: a
[shadowed room](special-rooms.md#room-shadowed) on a gloomy level is darker still. Today the [draining room](special-rooms.md#room-draining) and the [cursed room](special-rooms.md#room-cursed)
can do this.

## The rooms

"Weight" is how often a room is drawn against the others, and "Share" the part of all draws it
gets where a dungeon favours or bans nothing. "Only" says where it can come at all.

{{#include generated/special-rooms.md}}

- **Shadowed room.** Magical darkness: every [light](light.md#what-light-is) that reaches into the room keeps only a part of
  its strength there, your [lantern](lantern.md), wall [torches](light.md#light-sources), fires, and the [sun](sun.md#sky-light) or [moon](sun.md#moonlight) through an opening
  alike. With normal eyes you see little inside unless a light is close; creatures there are
  hidden just as you are, and they see you no better. [Darkvision](vision.md#the-four-types) works as anywhere else.
- **Echoing hall.** Everything you do in it is [loud](hearing.md#how-loud): your [footsteps](hearing.md#what-makes-noise), opening a [door](doors.md), fighting.
  Louder sounds carry farther, so creatures much farther away hear you (see
  [Hearing](hearing.md)). The same goes for creatures making noise in it.
- **Hushed chamber.** The opposite: [noise](hearing.md#what-makes-noise) made inside barely carries. A good place to sneak
  through, or to fight without drawing a crowd.
- **Sanctuary.** A place of peace with a faint warm glow, enough to see by. You [heal](healing.md#how-fast) much faster
  while you stand in it (see [Combat](hit-points.md#getting-better)), and creatures [hostile](creatures.md#factions) to you
  will not step in: they stay outside, waiting. They can still strike at you across its
  threshold if you stand in the doorway, and shoot at you from outside. Creatures that are not
  hostile to you come and go as they please.
- **Flooded room.** Ankle-deep water over the floor: wading through it is slower than walking,
  and every step splashes loudly (see [Terrain](terrain.md#water)).
- **Black chamber.** A dark that swallows every light, your [lantern](lantern.md) included: nothing is lit
  inside at all. Only [darkvision](vision.md#the-four-types) sees in it, so creatures inside are hidden from normal eyes, and
  you from theirs.
- **Draining room.** Something in it drinks at magic: everyone inside who has [mana](mana.md#how-much), you and
  creatures alike, is *[Ebbing](status-effects.md#status-ebbing)* (see [Status effects](status-effects.md)) for as long as they stay
  and half a minute after they leave. [Ebbing](status-effects.md#status-ebbing) drains a point of [mana](mana.md#how-much) every 20 seconds and none
  comes back meanwhile, so a fight between [casters](magic.md) in a draining room soon becomes a brawl. It
  glows a faint cold violet. One floor in twenty that draws a draining room is drained
  throughout instead.
- **Sweltering chamber.** Hot air: it is 25 °C warmer than the level around it, and glows a
  faint red. The heat does nothing to you yet, but [ice](temperature.md#walking-on-ice) melts in it (see
  [Temperature and ice](temperature.md)). The [status bar](the-screen.md#the-screen) shows the heat as you step in.
- **Frigid hall.** Bitter cold: 25 °C colder than the level around it, with a faint blue glow.
  Its [water](terrain.md#water) freezes even on a mild level, so a pool in it is usually solid [ice](temperature.md#walking-on-ice). The cold does
  nothing to you yet.
- **Silent vault.** A dead silence: every [noise](hearing.md#what-makes-noise) made inside is a hundredth as loud, so a fight
  there is barely heard even next door (see [Hearing](hearing.md)). Everyone inside, you and
  creatures alike, is *[Silenced](status-effects.md#status-silenced)* (see [Status effects](status-effects.md)) for as long as they
  stay and ten seconds after they leave: no [spell](magic.md#the-spells) can be cast, no [scroll](scrolls.md) read, and no [voice](hearing.md#throwing-your-voice)
  thrown. A [wizard](classes.md#wizard) and a [kobold shaman](creatures.md#creature-kobold-shaman) fight with sticks in it; a [thief](classes.md#thief) walks through unheard.
- **Heavy hall.** The stones pull at whatever you carry: inside, your [carrying limit](burden.md#how-much-you-can-carry) is half
  what it is anywhere else, so a load that was light makes you [burdened](burden.md#burden-1), and armour that only
  burdened you [strains](burden.md#burden-1) you (see [Burden](burden.md)). Creatures carry less inside too. If what
  you carry would overload you inside, you cannot go in ("You carry too much to go in there."),
  and if you overload yourself inside by picking something up, you can still step back out.
- **Teeming den.** A room full of sleeping [creatures](creatures.md) and the things they have gathered. You may
  smell it before you see it, and you hear it when you step in. See below.
- **Hall of mirrors.** Glass on every side: everyone in it, you too, is shown with two
  reflections that look exactly like them, standing elsewhere in the room and shifting every
  turn. Nothing on the screen tells a reflection from the real thing ([look mode](controls.md#look-mode) only knows your
  own: "Your reflection"). Swing at or walk into a reflection and your blow meets only glass,
  which costs you the blow's time; an [arrow](ranged.md#shooting-and-throwing) or a thrown dagger aimed at one flies on through it.
  An archer outside the hall shooting at you while you are inside aims at you or at one of your
  reflections, each as likely; creatures inside the hall know it and are never fooled. [Spells](magic.md#the-spells)
  that fly to the first creature in their way pass the reflections by. Only rooms of 80 floor
  cells or fewer can be one.
- **Unstable room.** A small room (60 floor cells at most) whose floor shimmers: every step
  you take inside has a one in ten chance to fling you somewhere else on the level, as a
  [teleport trap](traps.md#trap-teleport) does ("You are wrenched elsewhere."). Creatures stepping in are flung just the
  same, so fleeing through one is a gamble, and so is chasing someone into it.
- **Collapsed floor.** A room whose floor is cracked and sagging (shown as `:`, "cracked
  floor"), found only on a level with another below it. Only the middle of the room is
  cracked: the floor next to its walls and doors is always sound, so you can always walk round
  the cracks from one door to another. Every step you take onto the cracked
  stone has a 15 % chance to give way, 30 % while you are [burdened](burden.md#burden-1) or worse ("The floor gives way
  beneath you!"). You fall to the level below, land somewhere open on it (never behind a [locked
  door](doors.md#locked-doors) or in a [shop](shops.md)), take 2d6 [damage](combat.md#damage), which can kill you, and are [Staggered](status-effects.md#status-staggered). A
  quick way down, if you can take the bruises; creatures never fall through.
- **Treasure room.** Wealth nobody has touched in a long time: one or two chests of the
  level's depth (see [Chests](chests.md)), two to five heaps of [gold](shops.md#money) coins (three six-sided dice
  times two coins: 6 to 36 coins a heap), and one to three loose [gems](gems.md) lie inside. Every
  door of it is a **[secret door](doors.md#secret-doors)**, so you must [search](searching.md#searching-1) the walls to find the way in (or [dig](digging.md#digging) in);
  in a cavern it is a box cut into the rock with one hidden door. Nothing guards it and nothing
  seals: its only dangers are its chests' [traps](traps.md#finding-traps) and the time it takes to find. On every arrival
  on its level until you have been inside, *something tells you there is wealth hidden on this
  level*.
- **Greater treasure room.** A fortune walled up in the rock: two to four [chests](chests.md) of the level's
  depth, four to ten heaps of [gold](shops.md#money) coins (four six-sided dice times five coins: 20 to 120 coins a
  heap), and two to six loose [gems](gems.md). It has **no door and no corridor at all**: nothing leads
  there, and the only way in is through a wall. One of its walls, beside a passage you can reach,
  is guaranteed to be thin; search beside it (as for a [secret door](doors.md#secret-doors), at the same difficulty) and
  *the wall here sounds hollow*. Look at it and it is a hollow stone wall. Dig through it with a
  pick-axe or [blast](digging.md#explosions) it open (a [scroll of fireball](scrolls.md#scroll-of-fireball) does it); any other wall of it can be [dug](digging.md#digging) too,
  if you guess well. Without a digging tool or a blast you cannot get in: come back with one.
  Nothing the level needs is ever inside, no [trap](traps.md#finding-traps) is hidden in it, and no creature wanders in.
  On every arrival on its level until you have been inside, *something tells you a fortune lies
  walled up on this level*.
- **Tempting hoard.** Only on a level with a danger: three items lie on its
  floor, better than the level's own (three item levels above it), and no one guards them. The
  moment you pick anything up inside, the room seals: every door of it slams shut and locks
  ("The doors slam shut!"; a door with something in the doorway stays open), and two to four
  creatures [hostile](creatures.md#factions) to you, a step more dangerous than the level's own, come at the room's
  edges, where you cannot see them if there is such a place. The [locks](doors.md#locked-doors) are of the level's
  depth and no key but a [skeleton key](doors.md#keys) fits; [pick](doors.md#picking-locks) them, [kick](doors.md#kicking) the doors in, or fight. A cave
  chamber has no doors, so there only the guardians come. A room seals once.
- **Cursed room.** The dead do not rest in it: see below. Only on a level with a danger. One
  floor in twenty that draws a cursed room is cursed throughout instead.
- **Haunted room.** A fine camp spot that is not: [rest](resting.md#how-resting-works) in it and the dead come, once every
  hour (see Haunted rooms below). Only on a level with a danger.

- **Altar room.** An old altar stands in the middle of the room, its god long forgotten, and a
  faint glow rises from it: you notice it from the doorway. Stand beside it and you may [pray](abilities.md#ability-pray)
  (see [Forgotten altars](altars-forges-springs.md#forgotten-altars) below). Only rooms of 9 floor cells or more can be one.
- **Forge.** A cold forge with an old anvil in the middle of the room; you smell soot and cold
  iron from the doorway. Stand beside the anvil and you may [temper](abilities.md#ability-temper) metal arms or armour you carry
  (see [Forges](altars-forges-springs.md#forges) below). Only rooms of 9 floor cells or more can be one.
- **Shrine.** A squat stone shrine stands in the middle of the room. Like a forgotten altar it
  has a hidden side, here only **[good or evil](alignment.md)** (as likely either way), and its stone's light
  tells which: gold for good, red for evil. It cares only whether you are on its side (see
  [Alignment](alignment.md)): of its side, you [heal](healing.md#how-fast) twice as fast inside ("You feel welcome
  here."); of the other side, you are *[Unwelcome](status-effects.md#status-unwelcome)* (see [Status effects](status-effects.md)) for
  as long as you stay and half a minute after you leave: wounds [heal](healing.md#how-fast) at half the pace and your
  blows go a little astray ("Something here wants you gone."). A neutral character is ignored
  ("An old shrine. It ignores you."). Creatures are treated the same way by their own side, so
  a good shrine is a poor place for an orc to fight you. There is nothing to pray to. Only rooms
  of 9 floor cells or more can be one.
- **Scriptorium.** Shelves of rotting scrolls stand in the middle of the room (shown as `≡`),
  and you may smell the dust and old vellum before you see it. Two to four [scrolls](scrolls.md) lie on its
  floor, of the kinds the level would give anyway, unread. Inside, the chance of a
  [training mark](skills.md#getting-better) in [Decipher script](skills.md#skill-decipher-script) is higher: [making out a scroll](scrolls.md#making-out-a-scroll) earns a mark every time, and two here. Only rooms of 9 floor cells or more
  can be one.

The talent *[Old delver](talents.md#talent-old-delver)* (Delver; [Willpower](stats.md#stat-willpower) 11 or [Dexterity](stats.md#stat-dexterity) 12) makes you at home in strange rooms: a [collapsed floor](special-rooms.md#room-collapsed) never gives way under you,
the [Ebbing](status-effects.md#status-ebbing) of a [draining room](special-rooms.md#room-draining) never takes you, and nothing you kill ever rises in a
[cursed room](special-rooms.md#room-cursed). With *[Old hand](talents.md#talent-old-hand)* after it (level 10) a room that quickens healing or mana quickens it
half again more for you (a [sanctuary](special-rooms.md#room-sanctuary)'s three times as fast becomes four times), and you
can lift one thing from a [tempting hoard](special-rooms.md#room-hoard) before it seals; the second seals it.

Only what happens inside counts: a sound made in an [echoing hall](special-rooms.md#room-echoing) is [loud](hearing.md#how-loud) wherever it is heard,
while one made outside and heard inside is not changed. Light outside a [shadowed room](special-rooms.md#room-shadowed) is as
bright as ever, even right at its door.

## Cursed rooms

A creature that dies in a cursed room (or anywhere on a cursed level), whoever killed it,
rises from its [corpse](food.md#corpses) two minutes later as one of the undead ("The giant rat's corpse rises as
a skeleton!"). You see its corpse twitch as it dies, if you see it; a rising you do not see you
may hear, as a groan. The undead never rise again, and creatures that leave no [corpse](food.md#corpses) never
rise at all.

What it rises as is drawn at its death, each alike, among the undead of the dead creature's own
level, or, when there are none of that level, of the nearest level that has some (the lower one
on a tie). A risen creature carries nothing (what the dead carried still lies there) and is
worth its own [experience](experience.md) when you kill it.

{{#include generated/risings.md}}

The [corpse](food.md#corpses) is what rises: **take it, eat it, or destroy it** within the two minutes, and nothing
rises. A [corpse](food.md#corpses) carried out of the room and dropped elsewhere is not cursed again. A level you
leave stays as it was until you come back, and whatever was due to rise meanwhile rises as you
return.

**Consecrate.** A [priest](classes.md#priest) (from level 9) or [paladin](classes.md#paladin) (from level 12) of [Neutral or better](alignment.md) can lift the curse:
standing inside a [cursed](special-rooms.md#room-cursed), a [draining](special-rooms.md#room-draining), or a [haunted](special-rooms.md#room-haunted) room, they spend 15 [mana](mana.md#how-much) and pray for a minute (from the
[abilities menu](abilities.md#the-abilities-menu)). If nothing disturbs the prayer (anything that would stop a [rest](resting.md#how-resting-works) stops it, and
then the mana and the time are lost), the room becomes a plain room for good: the dead no longer
rise there, nor does mana drain, nor does anything come while you [rest](resting.md#how-resting-works). On a level a curse or a drain covers whole, it lifts the whole
level's instead. It is a good deed (see [Alignment](alignment.md)). Anywhere else it is refused
("There is nothing here to consecrate."), costing nothing.

## Haunted rooms

Whispers are all you notice from outside ("You hear a whisper, and then nothing."), and inside
it is too quiet ("It is very quiet here. Too quiet to sleep."). Nothing happens while you walk,
fight, or [search](searching.md#searching-1) in it. But [rest](resting.md#how-resting-works) in it, and each time the clock strikes a full hour while you
rest (at 1:00, 2:00, and so on, however long or short each of your rests is), one of the undead
comes into the room: drawn as a cursed room's dead rise, among the undead of the level's [danger](dungeons.md#who-lives-there-and-how-many)
(or of the nearest level that has some), each alike. It appears where you cannot see it if the
room has such a place, else anywhere free in the room, and it comes straight for you. Your rest
stops as soon as it comes into view, as it would for anyone. It carries nothing and is worth its
own [experience](experience.md). Rest outside the room, even just beside its door, and nothing comes. [Consecrate](abilities.md#ability-consecrate)
(above) lifts the haunting for good.

## Hallowed ground

Not a room but a place a [priest](classes.md#priest) makes: [Sanctuary](abilities.md#ability-sanctuary), the priest's ability at level 50, is a
minute's kneeling for 30 [mana](mana.md#spending-it) (anything that would stop a [rest](resting.md#how-resting-works) stops it, and then nothing
is hallowed, the mana spent). Done, every cell within 2 of where you knelt (a 5 by 5 square) is
hallowed ground for an hour. Creatures [hostile](creatures.md#factions) to you do not step onto it, as into a
sanctuary: one already standing on it when it was laid may move about on it and leave, but not
come back. Nothing new comes to the level there. Arrows and spells still cross it. Whoever stands
on it [heals](healing.md#how-fast) and gets [mana back](mana.md#how-fast-it-comes-back) three times as fast (in a sanctuary room, the faster of the two:
they do not add up). Strike from it (a blow, a shot, a throw, or a spell that hits someone) and
the peace is broken: the ground is ordinary again; so is it when a [dark prayer's twist](prayer.md#the-holy-and-the-dark) reaches a
creature. Being struck while on it breaks nothing.
Look at a cell of it and it reads "on hallowed ground"; a priest answered by the dark powers
(see [Prayer](prayer.md#the-holy-and-the-dark)) lays "dreaded ground" instead, which works the same. It gives no light, and
it cannot be laid on the overworld, in water, in a shop, or on ground already hallowed (no time
or mana spent).

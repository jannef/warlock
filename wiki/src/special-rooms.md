# Special rooms

Now and then a room of the dungeon is not like the others. As in ADOM, a **special room** changes
the rules while you are inside it: the light, the noise you make, how fast you heal, who dares to
follow you in, or the floor under your feet.

## Finding one

The first time you step into a special room, a message tells you what it is:

```
The light seems to drain away here.
```

From then on you know the room: [look mode](controls.md#look-mode) names it on any of its floor
you see or remember ("A stone floor in an echoing hall.", "You remember a stone floor here, in a
sanctuary."). Before you have been inside, it is just a room, though you may notice something
odd about it from the doorway: an unnatural darkness, a faint warm glow, water on the floor.
Some rooms give themselves away from outside: the first time you see any part of one, a line
hints at what lies there. The talent *Sense of place* (Wayfarer) lets you know a room's nature
as soon as you see it, without stepping in.

Special rooms are not terribly common, but not rare either: about three levels of a generated
dungeon in five have one, and a level never has more than one (some dungeons allow more). In the rooms-and-corridors levels it is a
whole room, walls and doors included, but never the room you arrive in; in the caverns it is a
patch of the cave, well away from where you arrive. Cēaptūn's Mynster is a
sanctuary.

## By dungeon

Each dungeon may have odds of its own, and some of its levels other odds again: a dungeon may
be full of flooded rooms, or never have a shop. "A room is special" is the chance for each room
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
shadowed room on a gloomy level is darker still. Today the draining room and the cursed room
can do this.

## The rooms

"Weight" is how often a room is drawn against the others, and "Share" the part of all draws it
gets where a dungeon favours or bans nothing. "Only" says where it can come at all.

{{#include generated/special-rooms.md}}

- **Shadowed room.** Magical darkness: every light that reaches into the room keeps only a part of
  its strength there, your lantern, wall torches, fires, and the sun or moon through an opening
  alike. With normal eyes you see little inside unless a light is close; creatures there are
  hidden just as you are, and they see you no better. Darkvision works as anywhere else.
- **Echoing hall.** Everything you do in it is loud: your footsteps, opening a door, fighting.
  Louder sounds carry farther, so creatures much farther away hear you (see
  [Hearing](hearing.md)). The same goes for creatures making noise in it.
- **Hushed chamber.** The opposite: noise made inside barely carries. A good place to sneak
  through, or to fight without drawing a crowd.
- **Sanctuary.** A place of peace with a faint warm glow, enough to see by. You heal much faster
  while you stand in it (see [Combat](combat.md#getting-better)), and creatures hostile to you
  will not step in: they stay outside, waiting. They can still strike at you across its
  threshold if you stand in the doorway, and shoot at you from outside. Creatures that are not
  hostile to you come and go as they please.
- **Flooded room.** Ankle-deep water over the floor: wading through it is slower than walking,
  and every step splashes loudly (see [Terrain](terrain.md#water)).
- **Black chamber.** A dark that swallows every light, your lantern included: nothing is lit
  inside at all. Only darkvision sees in it, so creatures inside are hidden from normal eyes, and
  you from theirs.
- **Draining room.** Something in it drinks at magic: everyone inside who has mana, you and
  creatures alike, is *Ebbing* (see [Status effects](status-effects.md)) for as long as they stay
  and half a minute after they leave. Ebbing drains a point of mana every 20 seconds and none
  comes back meanwhile, so a fight between casters in a draining room soon becomes a brawl. It
  glows a faint cold violet. One floor in twenty that draws a draining room is drained
  throughout instead.
- **Sweltering chamber.** Hot air: it is 25 °C warmer than the level around it, and glows a
  faint red. The heat does nothing to you yet, but ice melts in it (see
  [Temperature and ice](temperature.md)). The status bar shows the heat as you step in.
- **Frigid hall.** Bitter cold: 25 °C colder than the level around it, with a faint blue glow.
  Its water freezes even on a mild level, so a pool in it is usually solid ice. The cold does
  nothing to you yet.
- **Silent vault.** A dead silence: every noise made inside is a hundredth as loud, so a fight
  there is barely heard even next door (see [Hearing](hearing.md)). Everyone inside, you and
  creatures alike, is *Silenced* (see [Status effects](status-effects.md)) for as long as they
  stay and ten seconds after they leave: no spell can be cast, no scroll read, and no voice
  thrown. A wizard and a kobold shaman fight with sticks in it; a thief walks through unheard.
- **Heavy hall.** The stones pull at whatever you carry: inside, your carrying limit is half
  what it is anywhere else, so a load that was light makes you burdened, and armour that only
  burdened you strains you (see [Burden](burden.md)). Creatures carry less inside too. If what
  you carry would overload you inside, you cannot go in ("You carry too much to go in there."),
  and if you overload yourself inside by picking something up, you can still step back out.
- **Teeming den.** A room full of sleeping creatures and the things they have gathered. You may
  smell it before you see it, and you hear it when you step in. See below.
- **Hall of mirrors.** Glass on every side: everyone in it, you too, is shown with two
  reflections that look exactly like them, standing elsewhere in the room and shifting every
  turn. Nothing on the screen tells a reflection from the real thing (look mode only knows your
  own: "Your reflection"). Swing at or walk into a reflection and your blow meets only glass,
  which costs you the blow's time; an arrow or a thrown dagger aimed at one flies on through it.
  An archer outside the hall shooting at you while you are inside aims at you or at one of your
  reflections, each as likely; creatures inside the hall know it and are never fooled. Spells
  that fly to the first creature in their way pass the reflections by. Only rooms of 80 floor
  cells or fewer can be one.
- **Unstable room.** A small room (60 floor cells at most) whose floor shimmers: every step
  you take inside has a one in ten chance to fling you somewhere else on the level, as a
  teleport trap does ("You are wrenched elsewhere."). Creatures stepping in are flung just the
  same, so fleeing through one is a gamble, and so is chasing someone into it.
- **Collapsed floor.** A room whose floor is cracked and sagging (shown as `:`, "cracked
  floor"), found only on a level with another below it. Only the middle of the room is
  cracked: the floor next to its walls and doors is always sound, so you can always walk round
  the cracks from one door to another. Every step you take onto the cracked
  stone has a 15 % chance to give way, 30 % while you are burdened or worse ("The floor gives way
  beneath you!"). You fall to the level below, land somewhere open on it (never behind a locked
  door or in a shop), take 2d6 damage, which can kill you, and are Staggered. A
  quick way down, if you can take the bruises; creatures never fall through.
- **Cursed room.** The dead do not rest in it: see below. Only on a level with a danger. One
  floor in twenty that draws a cursed room is cursed throughout instead.

The talent *Hallowed rest* (Grit) makes a room that quickens healing or mana quicken it half
again more for you: a sanctuary's three times as fast becomes four times. The talent *Anchored
mind* (Scholar) keeps your power moored: Ebbing never takes you. The talent *Mirror-wise*
(Scholar) sees through reflections within 2 cells of you: they are not shown, and nothing stops
your step there. The talent *Light tread* (Delver) carries you over a collapsed floor as if it
were sound rock: it never gives way under you. The talent *Grave warden* (Guard) keeps what you
kill down: nothing you kill ever rises in a cursed room.

Only what happens inside counts: a sound made in an echoing hall is loud wherever it is heard,
while one made outside and heard inside is not changed. Light outside a shadowed room is as
bright as ever, even right at its door.

## Teeming dens

A teeming den comes only on a level with a danger (see [Dungeons and danger](dungeons.md)), in a room
of 16 floor cells or more. Its sleepers are the level's own kind of creatures: each is drawn just
as the level's other creatures are (by the level's danger, how rare each kind is, now and then a
weaker stray or one from deeper down), only creatures hostile to you and never a unique one.
Sometimes the whole den is of one kind (a *warren*: a nest of rats, say) or of one faction (a
*tribe*: a goblin camp). They lie on the floor among their hoard, a few items a little better
than the level's own.

A *beast den* is a den of animals and fen things only: its sleepers are drawn the same way, but
only among the wildlife, the vermin, and the marsh things, whether they are hostile to you or
not. A dungeon whose own creatures hold none of these never has one. Its hoard is no treasure
but what the beasts dragged in: piles of bones and fresh corpses of the level's creatures, good
to eat when you first come to the level (they rot from then on).

{{#include generated/teeming-dens.md}}

"Wakes at" is how loud a noise must be where a sleeper lies, in percent of the loudness it can
just hear (see [Hearing](hearing.md)): sleepers hear a whisper but wake only to twice that.
Sleepers never wake by themselves, however long you wait or rest nearby. They wake when

- a noise is loud enough where they lie: your footsteps, a door, a fight, a kick, digging;
- something hurts them, a blow, a missile, a spell, even poison; or
- a neighbour wakes: every sleeper that wakes cries out at once ("Cry" is its loudness), and the
  cry may wake those near it, and theirs the next, until the whole den is up.

Light does not wake them: carrying your lantern in is safe. Your plain footsteps (60) wake a
kobold (which hears 15, so wakes at 30) up to three cells away; with some Stealth, or the
talent *Hush-foot* (Shadow, Stealth 10: sleepers need twice the noise to wake from anything you
do), you can walk right past. A sleeper has seen no one, so *sneak attacks* land on it every
time; but every blow is noise, and a kill is a fight its neighbours may hear. A sleeper you see
shows as "asleep" in look mode, and you are told when one wakes. The talent *Sixth sense* tells
you on arrival when something sleeps on the level.

Sleepers count among the level's creatures: while the den is full, fewer newcomers find their
way to the level. Killing a sleeper is like killing any other foe.

## Cursed rooms

A creature that dies in a cursed room (or anywhere on a cursed level), whoever killed it,
rises from its corpse two minutes later as one of the undead ("The giant rat's corpse rises as
a skeleton!"). You see its corpse twitch as it dies, if you see it; a rising you do not see you
may hear, as a groan. The undead never rise again, and creatures that leave no corpse never
rise at all.

What it rises as is drawn at its death, each alike, among the undead of the dead creature's own
level, or, when there are none of that level, of the nearest level that has some (the lower one
on a tie). A risen creature carries nothing (what the dead carried still lies there) and is
worth its own experience when you kill it.

{{#include generated/risings.md}}

The corpse is what rises: **take it, eat it, or destroy it** within the two minutes, and nothing
rises. A corpse carried out of the room and dropped elsewhere is not cursed again. A level you
leave stays as it was until you come back, and whatever was due to rise meanwhile rises as you
return.

**Consecrate.** A Good priest (from level 9) or paladin (from level 12) can lift the curse:
standing inside a cursed or a draining room, they spend 15 mana and pray for a minute (from the
abilities menu). If nothing disturbs the prayer (anything that would stop a rest stops it, and
then the mana and the time are lost), the room becomes a plain room for good: the dead no longer
rise there, nor does mana drain. On a level a curse or a drain covers whole, it lifts the whole
level's instead. It is a good deed (see [Alignment](alignment.md)). Anywhere else it is refused
("There is nothing here to consecrate."), costing nothing.

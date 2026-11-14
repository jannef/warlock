# Spells with a rule of their own

Some [spells](magic.md#the-spells) do something no other spell does. None of them spares you or your
[allies](allies.md).

- **Thunderclap** and **Benison** go off around you. They need no aim, and you are at their centre.
  - Thunderclap strikes you with the full roll, as it does everyone at a burst's centre, and leaves
    you [Stunned](status-effects.md#status-stunned) along with the rest. The whole level hears it.
  - Benison's one roll heals you and every ally of yours within it who is hurt. It
    [sears every undead](#healing-harms-the-undead) within it, yours or not. It does nothing to
    anyone else, and it is not cast when no one there is hurt and no undead is there.
- **Leech** gives you back half the [hit points](hit-points.md#hit-points) it takes, rounded down,
  never past your maximum. The undead are immune to [necrotic](spell-types.md) harm, so they give
  nothing.
- **Rebuke** reaches only the undead, yours and others'. The living in it are not touched. It is
  fire, so it [burns](status-effects.md#burning) as fire does and leaves a fire at its centre.
- **Bramble** only catches those standing where things grow: grass, marsh, reeds, flowers, bushes,
  forest floor, fungus (the [terrain](terrain.md) table says which). On stone it does nothing.
- **Gale** drives everyone on its line one cell straight away from you, the nearest first. Anyone
  with someone behind them stays put, and so does anyone [held or webbed](status-effects.md#status-held).
  It harms no one. It puts out the fires on its line and tears away the [zones](#zones) of fire,
  foul cloud, darkness, and smoke on it; slick ground stays.
- **Flare** [stuns](status-effects.md#status-stunned) everyone it reaches who has eyes. The eyeless
  (worms, the black pudding, the canoloth) take nothing. A shut door stops it.
- **Bind** holds one hostile creature you see where it stands. It is
  [Pinned](status-effects.md#status-pinned) two degrees (more as your Presence grows), so it cannot
  step, though it may still strike. A pin eases a degree with each action it takes. Bind never
  misses. A [wand of bind](wands-and-rods.md#wands) casts it. (**Shatter**'s
  [wand](wands-and-rods.md#wands) is a [force](spell-types.md) burst, like the spell.)
- **Blink** takes you to a free cell within 5, each equally likely, through walls or not. It works
  while you are [held](status-effects.md#status-held), [webbed](status-effects.md#status-webbed), or
  [pinned](status-effects.md#status-pinned), and ends all three.
- **Sense the quick** lets you feel every living creature within 8 cells that you cannot see, as a
  heartbeat. You [hear](hearing.md) it within 2 cells of where the creature is, every turn while the
  spell lasts. The undead, constructs, and animated things have no heartbeat.
- **Cloudburst**, cast under the open sky, brings an hour of [rain](weather.md). It at once puts
  out every fire, and everyone [burning](status-effects.md#burning), under the open sky on the
  level.

## Healing harms the undead

Mend, Cure wounds, and Benison are healing spells. On the undead they work backwards: the roll
that would close wounds is [fire](spell-types.md) harm instead. "The light of mending sears the
skeleton." Grave feast is not a healing spell: it drains a corpse, so this does not apply to it.

- **A touch** (Mend, Cure wounds) on an undead beside you is an attack. It must hit, as
  [Cause wounds](magic.md#the-spells) must: your roll against its [Defence](combat.md#defence). A
  [critical](combat.md#criticals) rolls the harm again. It works on an undead that is not hurt.
- **Benison** sears every undead within it, yours or not, and heals the living allies there.
- The harm is the heal's whole roll, grown by your [scores](spellcasting.md#how-a-spell-grows) and the spell's
  rank. It is fire, so creatures that resist fire take less, and it may leave one
  [burning](status-effects.md#burning).
- Scrolls and wands of these spells do the same.
- The living are healed as before, friend or foe.
- Creatures that heal never heal an undead, and undead creatures cast no healing spells.

## Zones

A **zone** is an area that lies on the floor for a while. [Balefire](magic.md#the-spells) burns,
[Reek](magic.md#the-spells) poisons, [Mirk](magic.md#the-spells) darkens,
[Slick](magic.md#the-spells) trips, and [Fume](magic.md#the-spells) smokes. All five come only
from the [grimoires](learning-spells.md#learning-a-spell) and [scrolls](scrolls.md#spell-scrolls) you find (and a flask of smoke lays Fume's).
[Pitch fire](magic.md#spell-pitch-fire) burns too, in a pool where it bursts; it comes from a scroll or
a [flask](potions.md#flasks) of it.

- A zone's cells stay as they were laid, and it never spreads.
- It takes you and your [allies](allies.md) like anyone else. The caster has no protection from
  their own zone, though a kill it makes is theirs.
- A zone laid over another replaces it on the cells they share.
- It stays when you leave the level, and its time runs while you are away.
- No zone can be laid on the [overworld](overworld.md).

| Kind | Who it takes | What it does |
|---|---|---|
| Fire | whoever starts a turn on it or steps onto it | the spell's [fire](spell-types.md) harm, one number rolled when it is laid and the same for all its time, halved by [Evasion](talents.md#talent-evasion); then fire's one chance in four of [burning](status-effects.md#burning). Each cell sheds a fire's light, and the [ice](temperature.md#water-freezes) under it melts as it is laid. It lies only on walkable ground out of the water |
| Foul cloud | whoever starts a turn in it | [Poisoned](status-effects.md#poisoned) one degree more, no other harm; nothing to the [poison-immune](damage-types.md). It does not block sight |
| Slick ground | whoever steps onto it or off it | a [Dexterity](stats.md#stat-dexterity) check against 12. Failed: [Staggered](status-effects.md#status-staggered), the next action half again as slow. The step is still taken. Nothing incorporeal slips. It lies only on walkable ground out of the water |
| Smoke | no one directly | no eye sees into it or through it, yours and every creature's: your sight stops at its edge as at a wall, and whoever stands in it is seen only from a cell beside them. From inside, you see only the cells beside you. Light, missiles, and spells pass through it. Gale tears it away; rain does not |
| Darkness | no one directly | no light reaches its cells, the sun's included, and a light inside sheds none, a burning creature's too. Eyes that need light see nothing inside; [darkvision](vision.md#the-four-types) sees in as far as it reaches. Your own eyes follow the same rule: from inside, you see only the cell you stand on and what is lit beyond |

A [cold](spell-types.md) spell puts out the fire on each cell it reaches, and the rest of the zone
burns on. A zone shows as a still tint over its cells, and look mode names it.

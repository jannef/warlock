# Warlock specification — 3. Showcase world

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

## 3. Showcase world

From M1 on, the playable build is a hand-authored **showcase world** where `@` walks through one room
per engine capability. Each milestone adds or upgrades rooms. The showcase world is also the
regression scene for golden screenshots, replays, and snapshot tests.

**The showcase is a test campaign** (M32, `docs/design/campaign.md` §3). A campaign is a world of
its own, with nothing linking it to another: its overworld and levels, the scenarios that start
in it, its dungeon branches, quests, people (dialogue), and its own creature types, in
`assets/campaigns/<id>/` with a `campaign.toml` (name, description, the scenario a new run
starts, and `kind`). Everything else (terrain, objects, the common creature types, rules) is
engine data every campaign shares. The showcase is `assets/campaigns/showcase/` with
`kind = "test"`: its levels and nine scenarios, the Depths, the four quests, and the hermit,
gardener, and bard (their dialogue and their types). New run lists a test campaign's scenarios
in developer builds only (§6), and the player wiki leaves it out once a campaign to play exists.
**Test content never appears in a release** (#331): the showcase is built in only with the
cargo feature `showcase`, on for every developer command, test, and benchmark and off in the
release archives and the web build, which therefore hold no file of it and refuse its
scenarios as unknown.
A campaign's files name only its own and shared levels and creature types; ids are unique across
every campaign; each campaign has at most one overworld, and a run's local areas are its
campaign's overworld's; a run remembers its campaign, and a save carries that campaign's content
fingerprint (§7 "Saves"). The first campaign to play is the Ascmearc (`assets/campaigns/ascmearc/`,
#332, the campaign design §5): its 120×60 overworld, entered by the pass Beorggeat, a closed way
back, with the market town Cēaptūn five road cells east, the barrow Cealdhlāw in sight of the
road, and two dungeons hidden until quests reveal them (Fennmynster, Fyrdelf); its start
scenario is `ascmearc`.

**Level 1 — Hub (80×21)**

| Room | Demonstrates | Milestone |
|---|---|---|
| Start room | movement, message line, `?` help; gold and a small pile to pick up | M1, M11 |
| Door corridor | opening/closing doors, doors blocking sight; a storeroom of arms and armor (something for every equipment slot but the neck and the rings, a two-handed quarterstaff and great sword, a short bow and arrows, flaming arrows, a long war bow, a crossbow and bolts, a sling and stones, rocks to throw, a pick-axe for the Tool slot and the iron key to the pantry by its door; in its middle two piles of magic arms: a Sharp emerald-pommelled long sword of the Bear (its pommel a small emerald of fine cut, §7 "Gems") and Dread Visage, a rare iron helm that glows when worn, both identified, and a rare short bow and a magic silver ring, not) and a pantry of food (food rations, loaves of bread, dried meat, apples; eat with `E`) and other objects, whose west door is locked, the iron key to it lying in the storeroom (open it with the key, or kick it in) | M1–M2, M11, M16, M19, M23, M29 |
| Patrol gallery | creatures moving in and out of view, never shown in memory; a cave beetle and a quicker cave bat (wildlife, so they keep to their lanes) on parallel lanes, a slower torch-bearing kobold circling them (wielding a sword, carrying coins), a kobold by type but placed among the peaceful folk (§7 "Factions"), so it walks its round and never attacks the player; heard from the corridor behind its closed door, and in its dark middle (the bat flies silently), which a flaming arrow shot from its door lights where it lands | M2, M10, M11, M16 |
| Light hall | colored lights, falloff, pillar and creature shadows, steady magical light | M3, M8 |
| Darkness chamber | vision types, a UV-only rune, darkvision fade; an amulet seen only by those who see in the dark | M3, M11 |
| Water garden | still ponds and running rills (flow, glints, foam), pulsing altar with a ring before it | M4, M8, M11 |
| Effects room | glyph particles, glow, fades, distortion | M4 |
| Stairs | connection to Level 2, the arena, the quarry, the Frost Grotto, and the trap hall (down), the courtyard and the overworld (up); the nook is a sanctuary (§7 "Special rooms"); an old hermit sits in its corner beside the quarry stairs and tells of the hall's rooms and its seven stairs (topics, §8 "Dialogue"); he wants three of the arena's giant rats killed, then a look three levels down into the Depths, and takes the bard's letter (§8 "Quests") | M2, M9, M10, M18, M19, M25, M26, M28 |

**Courtyard (66×19, outdoor)**: up the stairs nook's `<`, a walled garden under the open sky —
trees, a ruined wall, a pond, standing stones, flower beds, and a roofed gazebo with a lamp — where
sunlight, sky light, and shadows turn and stretch with the time of day (§5 "Sunlight"; M9), and the
moon lights it faintly by night, waxing and waning with the month (§5 "Moonlight"; M12). The
`courtyard` scenario starts there at 06:30, while the shadows are long. Windfall apples lie under
a tree, and a candle and a scroll in the gazebo (M11). A gardener paces beside the flower bed by
the ruined wall (one-liners, grumbling now and then; he asks for three apples), and a wandering
bard walks a round of the meadow east of the gazebo, singing (one-liners and topics: the moon, the
year 1444, the Great Race, the Wildlands; he gives a sealed letter to carry down to the old hermit;
M28, §8 "Quests"). Its pond freezes with the season (§5 "Ice"; M26): on 11 Darkening, the default
start, the clear night has left thin ice on it at dawn, which the afternoon thins again; in
Deepwinter and Frostmere it freezes thick. A purse of 60 gold
coins lies in the gazebo, and east of it a flagstone path leads to a small roofed timber hut with
a door and a hanging lamp: Grimbold's General Store (§7 "Shops"; M27), Grimbold by the door and a
bit of everything on the plank floor, an angelica root among it (the remedy for diseases, §8
"Diseases"). A merchant fights no one, so the courtyard stays safe
unless the player steals.

**Frost Grotto (44×11, M26)**: down the stairs nook's third `>` (below the first), outside the
safe hub: a cave a little below freezing (−2 °C) with a frozen pool to walk on (§5 "Ice"). West
to east: thick ice (20 cm) that holds anything that walks, ice (5.5 cm) that holds a person but
may give way under a heavy pack, thin ice (2.5 cm) that gives way under anyone but a rat, and open
water where a spring of running water (which needs a harder frost) runs in from the east wall.
A silver ring lies out on the thin ice (it sinks to the bottom of the hole when the ice gives
way, where wading in fishes it out), and a heap of rocks by the stairs lets the player load up
and try the ice heavily laden. Arriving says "Your breath mists in the cold air."; lamps and a
faint cold ambient light the pool. The ice keeps and slowly thickens; a hole refreezes in a few
hours.

**Safety**: the hand-made showcase (the hub, Level 2, the courtyard) is safe. Its people (the hermit, the gardener, the bard) are peaceful folk by type. Creatures that would hunt the player by type (the hub's kobold, Level 2's kobolds and rats) are placed in the `peaceful` faction by their level (§7 "Factions"): they keep their types, patrols, speeds, footsteps, and looks, and attack no one. The showcase has no traps (§8 "Traps"). Fighting happens in the arena, and in the generated Depths (the real dungeon), which hide traps too. Arriving in the arena says "The clash of arms echoes off the stone." (a level's `arrival` line).

**Trap hall (72×21, M18)**: down the `>` below the stairs nook's `<`: the place to learn traps (§8
"Traps"), lit dimly throughout. The `traps` scenario starts there, in the middle of its south wall
(beside the `<` back up), where the whole hall is in view; arriving by stairs says "Scorch marks
and old arrows litter the flagstones." North, the proving row: seven bays, each with a known trap
to try (arrow, spear, pit, alarm, teleport, lightning, whose bolt runs south out of its bay
across the hall, and fireball). South, the crossing: creatures go their rounds across hidden
traps, so within fifteen turns the player watches a giant rat run onto spears, a cave beetle onto
a lightning trap whose bolt runs west along its lane (killing it), a torch-bearing kobold into a
fireball, and a second kobold shot from the south wall by an arrow trap, and from then on knows
those traps. Three pillars stand by that fireball, packed earth and stone north of it and granite
south: its blast breaks the two softer ones into rubble and the granite holds (§8 "Digging").
The rat and the kobolds are among the peaceful folk and the beetle is wildlife, so nobody attacks
the player; the traps attack everybody. North-west, three closets open onto the hall by trapped
doors (§8 "Trapped doors"): the west door shows its arrow trap, to try by opening it; the middle
one hides a fireball, to search for first; the east one stands open on a peaceful kobold's rounds
in and out of its closet and hides an alarm, so a player who shuts it watches the kobold ring it
opening the door again, and walk through the open doorway after without setting it off.

**Arena (72×21, M10)**: down the stairs nook's second `>`: the place to fight; factions at war. A kobold war band
(three torch-bearing kobolds wielding spears, an archer with a short bow and arrows ready, a slow
shaman wearing a bone amulet; look mode shows what each wields and wears, M16) and five giant
rats meet at a fire pit in
a pit lit by wall torches, with pillars for cover; two cave bats and a cave beetle (wildlife) go
their ways untouched. West lies the kobolds' camp behind two doors (kobolds open doors, rats do
not), east the rat warren. The player arrives in the dark south-west corner and can watch unseen;
each creature fights the nearest foe it sees, so coming close (lit, or within a kobold's
darkvision) turns them on the player. The brawl resolves (M16): melee pairs trade blows, the
archer shoots and the shaman casts (its spells announced only) over them, the wounded look it,
and the fallen drop their spears, bow, and amulet where they die (and now and then something
magic: kobolds drop loot at the arena's item level 2, §8 "Magic items"); the rats, quicker and harder
to hit, win within a few minutes of game time, leaving blood around the fire pit where the
fallen died (§8 "Blood"). Lighting the lantern there gets the
player shot from the dark. The archer spends an arrow a shot (they lie where they fell) and,
out of arrows, comes to fight with its fists. Beside the player's corner lie a short bow, arrows,
and flaming arrows, to shoot into the pit from the dark. The `arena` scenario starts there.

**Quarry (64×21, M19)**: down the stairs nook's third `>`: an old stone quarry, the place to dig
(§8 "Digging"), and nothing lives there. The arrival room has the stairs back up, a pick-axe beside
the player's start and a mattock below it, and a doorway into the pit: lit by wall torches,
walled by a granite face to the north and a bank of packed earth to the south, with a block of
stone and the rubble of old digging about and a cut into the granite. Behind the pit's east wall
lies a sealed chamber with a lamp, rubies, and gold, and no way in but through a wall: its light
spills into the pit once the wall gives way. The rock around (solid rock) shows as stone walls as
it is bared. The `quarry` scenario starts there, beside the pick-axe.

**Level 2 — Vast (≥ 400×300)**: scrolling camera, many light sources, performance budget check.
Introduced in M2, lit in M3. Beside its stairs up to the hub, stairs down lead into the depths and
stairs up lead out to the overworld (the cavern mouth). A river of running water winds through it
from north to south, foaming against a few boulders.

**The Depths (generated, M6)**: a dungeon branch of eight procedurally generated levels
(`depths:1` to `depths:8`; rooms and corridors at 80×21, caverns at 120×60 on depths 3–5) built
from the run seed, with vaults stamped in, a few items of loot on every floor, at the depth's
item level (§8 "Magic items"), and two to four hidden traps on each level (M18; the shrine
vault's offering lies on a hidden spear trap); about one level in two or three has a special
room (§7 "Special rooms"), and about one in six a shop (§7 "Shops"; with seed 2, a
provisioner's on the first level). On its rooms levels a few rooms are locked, their rusty key lying
where the player reaches it, and the guard post vault's store room is locked, its brass key
carried by the kobold on guard (M19). The `depths` scenario starts on its first level.

**The Overworld (M9)**: *The Wildlands*, a hand-authored 100×50 map of the land at large
(`campaigns/showcase/levels/overworld.toml`, level kind `overworld`), ADOM-like. Overworld terrain: plains,
grassland, forest, dense forest, hills, mountain ranges, swamp, rivers and deep water, roads, a
bridge, and locations. A step is a journey: it takes the destination terrain's `travel_minutes`
(road and bridge 20, plains 45, grassland 50, forest 90, hills 120, swamp and dense forest 150,
locations 30) at normal speed; every other action there (waiting, entering a location or a local
area) takes its usual time, 6 s for a normal action (§8 "Time"); mountains and open water are
impassable, and the river Wendle (drawn flowing, with legend arrows like running water; §4.3) can
only be crossed by the bridge. Named regions (Greenmarch, Thornwood, Barrowdowns, Hollowcrag, Silvermere, Sallow
Plains, Mirefen, Greyspine Mountains) show in the status bar. The Ascmearc's overworld (#332) adds
shared terrain: a `mountain_pass` (walkable, 30 minutes, nothing to explore, no way in or out),
a `volcano` (impassable, a faint glow and a thin plume of smoke), a `ford` (a road through a
river, 30 minutes), and the locations `town`, `barrow`, and `mine` (stairs down, 30 minutes).
- **Locations** are stairs down: the Hall (to the hub), the cavern mouth in the Hollowcrag (to the
  vast level), and the way into the Depths in the Barrowdowns (`depths:1`). `>` on a location
  enters its level, arriving on the stairs that lead back; those stairs (`<`) lead out to the
  overworld at the location. The hub's stairs nook has a `<` to the overworld.
- **Local areas**: `>` on any other cell whose terrain names a local ground enters the outdoor
  local area of that cell (level id `local:x,y`, 80×40), generated on the first visit from the
  run seed and the cell (§7 "Generators", *outdoor*) and then persistent like any level. The
  player arrives on the side (or corner) they walked onto the cell from — stepping east onto it,
  on the area's west edge, on the road if they came along one — or in a glade in the middle if
  they did not walk there (arriving by stairs or back from the area itself). Walking off any edge
  returns to the overworld cell. What an area looks like follows its overworld terrain's
  landscape and its neighbors:
  - *plains*: dry grass with greener hollows, tall grass, a lone tree or a copse, bushes,
    boulders; *grassland*: green meadows, tall grass, wildflowers and buttercups, copses fringed
    with bushes, lone trees, grazing deer;
  - *forest*: woods of leafy trees with stands of pines in clumps, undergrowth and bluebells,
    grassy clearings, a glade in the middle; *dense forest*: thick woods, mostly pines, small
    clearings, game trails from the middle out to every side;
  - *hills*: rocky ground with grassy slopes, crags (rock outcrops, 2 tall) with fallen boulders
    around them, a few pines, now and then a standing stone; *swamp*: marsh with pools of still
    water, reeds, grassy hummocks, willows, and dead trees;
  - a *road* cell runs a dirt road (2 wide, bridged where it crosses water) from near the middle
    out towards every neighboring road, bridge, or location, through the land most of its
    neighbors have; a *river* beside the cell runs along that side of the area (running water,
    one speed, flowing the way the overworld river does) and a *lake* lies along it (still water,
    reeds in the shallows);
  - ponds, a brook (running water across the area, quicker than a river), and ruins (outdoor
    vaults: a stone circle with an old altar that glows faintly at night, a broken watchtower, a
    roofless chapel) turn up now and then, and how wooded the land is blends towards the
    neighboring cells near each side, so areas side by side meet.

  Trees stand 2 tall, pines 3, standing stones 3, crags 2, bushes and reeds 1 (they cast sun
  shadows; trees, pines, crags, and boulders block sight). The `forest` scenario starts in a
  forest glade of the Thornwood at 06:30.
- **Day and night**: sight on the overworld depends on daylight and moonlight (§5), and the sun
  and the sky light the overworld and its local areas by day (§5 "Sunlight"), the moon by night
  (§5 "Moonlight"); at night the overworld is drawn by a faint cold light with the moonlight on
  top, while local areas are dark but for the moon.
- **Weather** (M12): clear, overcast, rain, or heavy rain over the overworld and its local areas
  (and the courtyard), changing every few hours by the month's chances; clouds dim the sun, the
  sky, and the moon, rain shortens sight on the overworld, and faint rain falls on what the player
  sees (§5 "Weather", §4.8).
- The `world` scenario starts on the road beside the Hall; the showcase scenario keeps starting in
  the hub.

# Warlock specification — 8. Architecture (continued): traps, doors, digging

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Traps** (`warlock-core/src/world/trap.rs`, data `data/traps.toml` §7; M18, #128):
  - **On a level**: a trap lies on a cell (`LevelState::traps`: its type, whether the player
    knows it, its charges left, a bolt's direction; saved), from the level's legend, a vault, or
    a generator (§7). It is **hidden** until the player sets it off, or sees it set off: the
    trap's cell is in their sight as the turn is perceived (a creature they see stepping onto
    it). Then it is **known** (`known`) from then on: drawn as its glyph and color over anything
    lying there (§4.2), remembered like terrain (`trap_memory`, as last seen; forgotten when seen
    gone), named by look mode, and in the snapshot (`PerceptionView::trap`, `known_traps`). There
    is no searching yet; a level may show a trap from the start (`known = true`).
  - **Setting one off**: whoever steps onto a trap sets it off, the player (any `move` onto it:
    the rules never ask; the interface does, §6 "Known traps") and creatures alike: creatures
    know nothing of traps, hidden or known, and walk onto them on their rounds or chasing someone,
    so luring them is a tactic; so does one who makes way for the player onto the cell they
    stood on (§8 "Displacement"), its pit's climb taken off its next action.
    `World::spring_trap(at, victim)` is the one way a trap goes off
    (a trapped door, later, calls it too): it spends a charge (the last leaves the trap spent and
    gone), makes the trap's noise on its cell (no actor made it, so everyone in earshot hears it,
    §5 "Hearing"; an alarm's bell draws every creature with enemies to look), and does what the
    trap does:
    - **shot** (the arrow trap): the missile shoots out of a hidden slit, the nearest opaque cell
      in the eight directions from the trap (outside the level counts; ties go to the direction
      a hash of the level and the cell picks first), along the line from it through the victim
      and on, at most 8 cells past them: it attacks whoever stands in its way in turn, rolled with
      the trap's to-hit and damage against their Defence (combat's rules, §8 "Combat": misses
      say why), until it hits, and comes down like any missile (§8 "Ranged combat": on whom it
      hit, else where it stopped; it breaks on a hit by its `breaks` chance), to be picked up.
      Its flight is an `Event::Missile` from the slit.
    - **strike** (the spear trap): an attack roll against the victim's Defence.
    - **burst** (the fireball): every actor on the transparent cells within the radius that a
      shadowcast from the trap reaches (walls stop it) takes one roll of the damage, half of it
      (at least 1) beyond radius − 1; no roll to hit, no saving throw. In ascending actor id
      order. Then, with a `blast`, the walls it reaches whose hardness is below it break (§8
      "Digging": the fireball's 1000 breaks earth and stone, not granite). Then a hot burst (the
      fireball) melts all the ice on the cells it reaches, and whoever stood on that ice falls
      in (§5 "Ice", `World::shock`); a cold one (the frost trap) freezes the water on them at once
      to at least its `frost` thick (`World::frost`: 10 cm, ice that holds a laden person).
    - **bolt** (lightning): a line from the trap's cell in its direction (the level's `dir`,
      else the longest clear line from it, ties to the earlier of N, NE, … NW) until an opaque
      cell, at most its `length` cells in all: every actor on it, from the trap outward, takes
      one roll of the damage.
    - **pit**: the victim takes the damage and loses its next action climbing out: its step
      costs a `wait` more (6 s at normal speed; a creature's too).
    - **alarm**: only its noise.
    - **teleport**: the victim is moved at once to a free cell of the level (walkable, no one
      there, no trap, no ice: §5 "Ice"), the cell at index `hash % count` of those in row order,
      the hash FNV-1a over the seed, the level id, the turn, the victim's id, and the trap's cell:
      deterministic, and never the gameplay random stream. A creature goes on from where it
      lands.
    Every roll (attack rolls, damage) comes from the gameplay random stream, so traps replay
    exactly. A trap's blow is dealt like any other: a big one spills the victim's blood (§8
    "Blood"; from the slit for an arrow, from the trap's cell for the rest), it takes hit points
    and kills (`Event::Died`; a creature drops what it carried), and a victim that lives gets the
    statuses its wound causes (bleeding from a big hit or a confirmed critical; §8 "Status
    effects"; caused by no one). A trap that kills the player is recorded as what killed them
    (`Death::cause`, the trap) and named on the death screen ("Killed by a fireball trap"). A
    trap's kill is no one's: it earns the player no experience (§8 "Experience") and counts for
    no quest (§8 "Quests"), even when they lured the creature there. Traps never lie on water
    that freezes (§5 "Ice": ice would cover them; a content error in a level file), so a fall into
    a pit and a fall through the ice never meet.
  - **Events**: `Event::TrapSprung { trap, at, victim }`, then what came of it: the shot's
    `Event::Missile`, a burst's or bolt's `Event::Spell { effect, origin, shape, cells }` (the
    spell effect a trap names in data, which must be an effect of `data/effects.toml` of the same
    shape when that file exists; `SpellShape::Burst { center, radius }` over the burst's cells,
    the very cells it hits, or `Bolt { path }` from the trap along the cells after it; drawn by
    the spell effects of §4.8 "Spell effects"), `Event::TrapStruck { trap, target, to, strike }`
    for each actor it reached (a hit for what is not rolled), and deaths. A trap going off is
    perceived (kept, and said) when the player set it off, sees its cell, or sees any of its
    blast (a spell is light itself: its cells are cut to those the player sees or has in line of
    sight, §4.8); what it did to someone is kept and said when that is the player or the player
    sees them.
  - **Messages**, only for what the player perceives, never a number: the trap's `message` ("An
    arrow shoots out of the wall!", "Spears spring from the floor!", "A fireball explodes!",
    "Lightning arcs across the floor!", "A loud bell rings!"), then what it did, by its `noun` and
    `verb` and combat's words ("The arrow hits you.", "You dodge the arrow.", "The spear glances
    off your armour.", "The fireball burns the kobold.", "The lightning bolt strikes the cave
    beetle."), a pit's "You fall into a pit." / "The kobold falls into a pit.", a teleport's "You
    are wrenched elsewhere." / "The kobold vanishes!", and deaths ("The cave beetle dies.", "You
    die..."). Like any message they stop a rest; a rest also perceives every turn in which a trap
    went off (a teleport moves a creature farther than its steps).
- **Doors** (`warlock-core/src/door.rs`, the rules; `world/door.rs`, applying them; M19, #130):
  locked doors and keys, and kicking doors in.
  - **Locks** are per-cell state of a level, not terrain of their own: a locked door is the
    closed door's terrain with a lock on its cell (`LevelState::locks`, saved and hashed). So it
    looks, blocks sight and sound, and is remembered exactly like any closed door, and the player
    cannot tell it is locked until they try it (perception never carries locks); every door
    property stays in one terrain definition; and a lock carries what a terrain id cannot: its
    optional **lock id** (§7 "Locks in level files"). A lock lies only on a closed door:
    unlocking it or breaking the door takes it away. Locking a door again is later work.
  - **Keys** are objects with a `key` (§7): the lock id they open, or `*` for a **skeleton key**,
    which opens every lock; a lock without an id opens only to a skeleton key. Opening a locked
    door (`o`, or walking into it) while carrying a key that fits unlocks it and opens it in one
    action, ADOM-like: `unlock` plus `open_door` (12 s at normal speed), "You unlock the door with
    your iron key and open it." The key made for the lock is used before a skeleton key; keys are
    kept. Without one: "The door is locked.", refused, no time.
  - **Kicking** (`K`, then a direction; `Action::Kick`, replay text `kick:DIR`; the `kick` action,
    6 s), ADOM-like:
    - **A closed door**, locked or not, breaks with a chance of the kicker's Strength **score**
      (never the modifier) in the door's `sturdiness` (§7): one draw of the gameplay stream,
      `1d(sturdiness) ≤ Strength`. The closed door's sturdiness is 30: Strength 9 breaks it 30 %
      of the time, 10 33 %, 15 50 %, 18 60 %, 30 and up always. A door without a sturdiness never
      breaks, and no die is rolled. The kicker's athletics (§8 "Skills") adds to the Strength:
      +8 at 100 (Strength 10 then breaks the closed door 60 % of the time), and every kick at a
      door may earn it a training mark. Kicking a door is no attack: it takes the plain `kick`,
      whatever the stance. A broken door becomes its `breaks_to`, the **broken door**
      (`,`, splinters in the doorway): walkable and see-through for good (it cannot be closed),
      its lock gone: "The door crashes open!" A door that holds: "WHAMM!" Either way the kick bangs
      on the door, a noise of 100 made on the door's cell (§5 "Hearing", heard as banging), and
      creatures that hear it and have enemies come to look (next to the door, §8 "Creature
      behavior").
    - **A creature** (seen or not, hostile or not) is an unarmed melee attack, whatever the
      player wields: bare hands (1d3) with Strength to hit and to damage (§8 "Combat"), what
      their unarmed fighting gives, and their combat stance (§8 "Weapon skills", "Stances"); it
      trains unarmed fighting, and its speed takes off the `kick`'s cost. Priced like every attack
      (§8 "Time"): the stance's `attack_cost` and the attack speed of what the player wears change
      its time too. Rolled, heard, and
      shown like any blow and said as a kick: "You kick the giant rat.", "You miss the giant
      rat.", "The giant rat dodges your kick."
    - **Anything else** is refused and takes no time: "You kick at empty space." where one could
      walk (an open or broken door too), else "Kicking the stone wall would only hurt your
      foot."
    - Creatures do not kick (yet).
  - **Creatures** that open doors (`opens_doors`) open a locked door only while they carry a key
    that fits: they unlock and open it in one action (`unlock` plus `open_door`). To every other
    creature a locked door is a wall (§8 "Creature behavior").
  - **Events**: a kick at a door the player sees is `Event::DoorKicked` (it held: dust shaken off
    it, the closed door's `toggle` particles) or `Event::DoorBroken` (splinters, the broken
    door's); unlocking a door is seen as its opening (`DoorOpened`).
  - **Where locks are**: hand-made levels lock doors in their legend (the hub's pantry, its
    iron key in the storeroom); vaults bring their locks and keys (the guard post's store room,
    its brass key carried by the kobold on guard, a skeleton key inside); a dungeon branch with a
    `key` gets a few locked rooms on its rooms levels, its key always within reach (§7
    "Generators", validated). The showcase's locked door is harmless.
- **Digging** (`warlock-core/src/world/dig.rs`; M19, #131): walls give way to digging and to
  explosions, ADOM-like.
  - **Walls that break**: terrain with a `hardness` and what it `becomes` (§7): stone walls 600,
    packed earth 100, granite 3000, each becoming rubble (walkable at 150 %, loud underfoot).
    Terrain without them never breaks (doors, torches, crystals, trees, boulders, mountains), and
    neither does any cell on a level's edge, whatever its terrain, so no level is dug out of. Solid
    rock (`void`) shows its `face`, a stone wall, once a cell next to it is broken open, as
    generated levels wall their open ground, so a tunnel is walled like a corridor.
  - **Digging** (`Action::Dig(dir)`, `dig:E`; key `U`): with a digging tool ready (the first thing
    worn or wielded with `dig` power: a pick-axe, 10, or a mattock, 15, in the Tool slot), into the
    wall next to the player in that direction. Refused, taking no time: without a tool ("You have
    no digging tool ready."), toward open ground ("There is nothing to dig there."), toward
    anything else that does not break ("You cannot dig through the closed door."), and on the
    overworld ("You cannot dig here."). It says "You dig into the stone wall." ("You go on digging
    at the stone wall." on a wall dug into before) and goes on turn after turn like a rest, through
    the same loop (perception batched while no creature can come into view): each turn is a `dig`
    action (1000: 6 s at normal speed, slowed by burden like every price), does work of the tool's
    power × (Strength + 10) / 20 (the score, never the modifier: a pick-axe at Strength 10 does 10,
    at 18 14), and makes the digging noise where the digger stands (80; §5 "Hearing"). The wall
    gives way, as the player acts, once the work done on it reaches its hardness: a pick-axe at
    Strength 10 digs through packed earth in 1 minute, a stone wall in 6, granite in 30; a mattock
    in 42 s, 4, and 20 minutes. Integer math: the work is kept in twentieths. It stops, like a rest,
    after a turn in which a creature came into view or a message that stops a rest was logged, when
    the player dies, and at the latest after a day; the work done stays on the wall
    (`LevelState::dug`, saved) for the next dig. Messages: "After 6 minutes, the stone wall
    crumbles.", "A kobold comes into view; you stop digging after 3 minutes.", "You stop digging
    after 3 minutes."
  - **Explosions** (`World::blast(center, radius, blast)`): an explosion breaks every wall in its
    burst whose hardness is below its blast, all at once and with no roll: the burst is the cells
    in line of sight of its centre within its radius, the walls bounding them included, taken
    before anything breaks, so a blast never reaches through a wall it breaks. Walls on the edge
    never break. "The wall collapses!" ("The walls collapse!") when the player sees one fall.
    A burst trap with a `blast` (§7) calls it after its damage, with its own radius: the fireball
    trap's 1000 breaks packed earth and stone walls, never granite.
  - **What follows**: a broken wall is its rubble at once and the light masks it could change are
    dropped (`World::set_terrain`, as for a door), so light, the player's sight, and creature
    sight pass the gap from the next perception, and memory keeps the rubble like any terrain.
    `Event::WallBroken { pos, cause }` (`Dug` or `Blast`) reaches presentation when the player
    sees the cell: dust rises there (the rubble's `toggle` particles, §4.8).
  - **What the player perceives**: whether they have a digging tool ready
    (`PerceptionView::ready_to_dig`) and which walls in sight they have dug into
    (`PerceptionView::is_dug`: look mode's "partly dug"); never the work done in numbers.
    Creatures do not dig, and no one digs down or up.

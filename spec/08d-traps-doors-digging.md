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
    gone), named by look mode, and in the snapshot (`PerceptionView::trap`, `known_traps`). The
    player may also find it (§8 "Searching"); a level may show a trap from the start (`known =
    true`).
  - **Setting one off**: whoever steps onto a trap sets it off, the player (any `move` onto it:
    the rules never ask; the interface does, §6 "Known traps") and creatures alike: creatures
    know nothing of traps, hidden or known, and walk onto them on their rounds or chasing someone,
    so luring them is a tactic; so does one who makes way for the player onto the cell they
    stood on (§8 "Displacement"), its pit's climb taken off its next action.
    `World::spring_trap(at, victim)` is the one way a trap goes off
    (opening or kicking a trapped door calls it too, "Trapped doors" below): it spends a charge (the last leaves the trap spent and
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
    - Every trap's damage is of a kind (`damage_type`, §8 "Damage types"): the arrow's and the
      spears' piercing (soaked by Protection like any blow), the fireball's fire, the frost's
      cold, the lightning's electricity, the pit's bludgeoning (a fall meets no armour, but the
      victim's resistance to the kind takes its share; a fall is no cut, so it opens no wound).
      A burst's or a bolt's damage a victim resists in full is `Strike::Resisted`: "The
      fireball does the salamander no harm."
    - **alarm**: only its noise.
    - **teleport**: the victim is moved at once to a free cell of the level (walkable, no one
      there, no trap, no ice: §5 "Ice"), the cell at index `hash % count` of those in row order,
      the hash FNV-1a over the seed, the level id, the turn, the victim's id, and the trap's cell:
      deterministic, and never the gameplay random stream. A creature goes on from where it
      lands.
    A burst or a bolt is the one shared spell code that scrolls cast too (§8 "Scrolls";
    `World::cast_burst`, `World::cast_bolt`, with the trap as the caster). Every roll (attack
    rolls, damage) comes from the gameplay random stream, so traps replay exactly. A trap's blow is dealt like any other: a big one spills the victim's blood (§8
    "Blood"; from the slit for an arrow, from the trap's cell for the rest), it takes hit points
    and kills (`Event::Died`; a creature drops what it carried), and a victim that lives gets the
    statuses its wound causes (bleeding from a big hit or a confirmed critical; §8 "Status
    effects"; caused by no one). A trap that kills the player is recorded as what killed them
    (`Death::cause`, the trap) and named on the death screen ("Killed by a fireball trap"). A
    trap's kill is no one's: it earns the player no experience (§8 "Experience") and counts for
    no quest (§8 "Quests"), even when they lured the creature there. Traps never lie on water
    that freezes (§5 "Ice": ice would cover them; a content error in a level file), so a fall into
    a pit and a fall through the ice never meet.
  - **Sidestepping a known trap** (`world/sidestep.rs`, #476): the player who steps onto a trap
    they know of, of their own will (a `move` onto it, after the interface's question), may avoid
    it. The chance in percent is `[avoid]` in `data/traps.toml`: `base` (55) plus `per_point`
    (10) for each point of their **dodge**, the Dexterity and Dodge categories of their Defence
    (§8 "Combat": the Dexterity modifier capped by the worn armour that allows the least and by
    the burden, plus every Dodge bonus: the Dodge skill, talents, magic items, the stance,
    statuses), kept within `min` (15) and `max` (90): never certain, never hopeless. One draw of
    the gameplay random stream, only when a known trap lies on the cell (not on a door: a door's
    trap goes off as the door opens, "Trapped doors" below). An avoided trap stays as it was (no
    charge spent, still known), nothing goes off, the step costs what it costs, the player is told
    in words ("You deftly avoid the arrow trap.", "You step lightly over the pit."), and it counts
    as a dodge for training (`Use::Dodge`: Tumble's marks, Dexterity's stat marks). Otherwise the
    trap goes off, and the Light step talent's check still applies (§8 "Talents"). A hidden trap
    gets no chance (it goes off; the Light step talent is its only escape), and neither does a
    step the player did not choose (turned back from a shop's door, §7 "Shops") nor a creature.
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
  - **Trapped doors** (#174): a level may put a trap on a door, closed or open (§7 "Traps in level
    files"), of any kind but a pit (`TrapEffect::fits_fixture`): a door's trap goes off at whoever
    opens the door from the next cell, and no one can fall into a pit there. It belongs to the
    door, not the floor: walking through the doorway never sets it off (`World::step_onto_trap`
    skips door cells, for the player, creatures, and whoever makes way; the interface never asks,
    §6 "Known traps"). Opening the door does (`o`, or walking into it, unlocking it with a key
    or picking its lock too), the player's and a creature's alike (`World::swing_open`), and so does every kick at it,
    as the kick lands and before the door breaks, whether it breaks or holds. It opens and shuts
    with the door: shut again, it is set again, and opening the door again sets it off again
    while it has charges. A door kicked in takes its trap with it (`World::set_terrain`, as with a
    lock). It is aimed from the door at whoever set it off: a shot from the slit beside the door at
    them, a strike at them, a burst around the door, a bolt along its `dir`, else from the door
    through them, an alarm's bell on the door, a teleport of them. The trap's own cell never stops
    its burst, so a door's blast goes out on both sides of it even while it is shut (the player
    sees their side of it). Hidden, it looks like the door; searching and noticing in passing find
    it like any hidden trap ("Searching" below), and once known it shows as the trap's glyph in
    place of the door's, like any known trap (§4.2), and look mode names both ("An arrow trap, on
    closed door."). A secret door is never trapped, as it is never locked (a content error in a
    level file; generators never hide a trapped door). The trap hall's closets show them (§3).
  - **Disarming traps** (`warlock-core/src/world/disarm.rs`; #232, the progression design's
    B7): the use of the Disable device skill, an ability (§6 "Abilities": `Ability::Disarm`,
    "Disarm a trap", in the Skills group; `KeyCommand::Disarm`, no key by default) and an action
    with replay text, `disarm:DIR` (`Action::Disarm`). Whoever is trained in Disable device (the
    skill with the `disarm` effect, §8 "Skills": the thief) may try to take apart a trap they
    know of on a cell next to them, on the floor or on a door (open or shut). The try is a check
    (§8 "Checks"; `check::roll`): 2d12 plus the Dexterity modifier (of the effective score)
    plus what Disable device gives (`disarm`: +8 at 100, +12 at 200, at most 20) against the
    trap's `difficulty`, the one finding it takes (with a branch's `hidden_per_depth`, §8
    "Searching"). Made: the trap is gone (from the level and from memory), "You disarm the
    arrow trap."; it earns nothing else. Failed: it stays, "You fail to disarm the arrow trap.";
    failed by 5 or more (`SPRINGS_SHORT_BY`; a natural 2 fails but falls short only by what the
    numbers say): "You set off the arrow trap!", and it goes off at the player as if they had
    set it off from where they stand (`World::spring_trap(at, player)`, as a trapped door is
    opened from the next cell: its arrow from the slit at them, its spears at them, its blast
    around it, its bolt through them, a pit's fall and the climb back out (its next action
    lost, the player still on their own cell), an alarm's bell, a teleport of them), spending
    a charge. Every try takes the time of `disarm` (2000: 12 s at a normal pace; priced before
    the trap may go off, plus a pit's climb) and may earn Disable device a training mark (`Use::
    Disarm`, 20 %, the training stream), whatever comes of it; the check is two draws of the
    gameplay stream. Nothing remembers a failed try and there is no limit on trying again (the
    owner's rule): time and the risk are the price. Refused, with no time and no draw: without
    the skill ("You do not know how to disarm traps."), and where the player knows of no trap
    ("You know of no trap there.": a hidden trap is refused as an empty cell is, so trying
    gives nothing away). Creatures do not disarm. The thief's class table (Trapfinding, level
    6) and the Tinker talent (the craftsperson's level 32 and the gnomes') will give it to
    those not trained in the skill when they land.
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
  - **Picking locks** (#233; the **Pick lock** ability, the Open lock skill's use, §6
    "Abilities", §8 "Skills"; `Action::PickLock(dir)`, replay text `pick:DIR`; the `pick_lock`
    action, 2000, 12 s), ADOM-like, no tool needed:
    - **Who**: an actor trained in a skill with the `pick` effect (Open lock; thieves are, from
      their class). Anyone else: "You do not know how to pick locks.", refused, no time.
      Creatures do not pick locks.
    - **A try** at the locked closed door in a direction is a check (§8 "Checks"): 2d12 plus the
      Dexterity modifier (of the effective score) plus what Open lock gives (+8 at 100, +12 at
      200, at most 20) against the lock's **difficulty**: its own (`lock_difficulty` in level
      and vault legends, 1 to 50; default 14, `door::LOCK_DIFFICULTY`), plus on a dungeon branch
      its `locks_per_depth` for each level below the first (the Depths: 1, so 14 on the first
      level and 21 on the eighth). Two draws of the gameplay stream; a natural 2 fails and a
      natural 24 succeeds, as every check. It takes `pick_lock` whatever comes of it, may earn
      Open lock a training mark (`pick`, 20 %), and makes no noise of its own.
    - **Picked**: the lock is gone and the door swings open in the same action, as with a key
      (its noise, `DoorOpened`, and a trap on the door goes off at the picker as it swings,
      "Trapped doors" below), on top of the try: `pick_lock` plus `open_door` (18 s), both
      priced before the check. "You pick the lock and open the door." The door stays unlocked.
    - **Failed**: "You fail to pick the lock." The lock holds, and nothing is remembered of the
      try: the next may succeed, as often as the player likes (no limit, the owner's rule: time
      is the price). A failed try sets off nothing.
    - **Refused**, no time: a closed door without a lock ("That door is not locked.", what `o`
      would find out at once), anything else ("There is no lock there."), or while a status
      `blocks` `pick_lock` ("You cannot pick locks while …", §8 "Status effects").
    - Lock difficulties are ground truth: never perceived, shown, or dumped but in the world
      state; the wiki gives the chances by difficulty.
  - **Kicking** (`k`, then a direction; `Action::Kick`, replay text `kick:DIR`; the `kick` action,
    6 s), ADOM-like:
    - **A closed door**, locked or not, breaks with a chance of the kicker's Strength **score**
      (never the modifier) in the door's `sturdiness` (§7): one draw of the gameplay stream,
      `1d(sturdiness) ≤ Strength`. The closed door's sturdiness is 30: Strength 9 breaks it 30 %
      of the time, 10 33 %, 15 50 %, 18 60 %, 30 and up always. A door without a sturdiness never
      breaks, and no die is rolled. The kicker's athletics (§8 "Skills") adds to the Strength:
      +8 at 100 (Strength 10 then breaks the closed door 60 % of the time), and every kick at a
      door may earn it a training mark; so do the talents' `door_kick` (Lockbreaker's 6) and the
      boots' (§8 "Magic items": the `door_kick` affix effect), `World::kick_force`. Kicking a door is no attack: it takes the plain `kick`,
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
- **Chests** (`warlock-core/src/chest.rs`, the rules; `world/chest.rs`, applying them; data
  `data/chests.toml` §7 "Chests"; docs/design/chests-and-treasure.md, slices C1 to C3): a chest is a
  **fixture of a cell**, as a door is, never a carried object.
  - **What it is**: one of three terrains (`chest = "shut"`, `"open"`, `"smashed"` in
    `data/terrain.toml`: `⊟`, `⊔`, `⊠`; a shut one is not walkable and blocks nothing else, an
    open or smashed one is walkable, a pile on it) plus a chest on the cell
    (`LevelState::chests`, saved and hashed, left out while a level has none): its **level** and,
    while it is shut, what it **holds** (ground truth). The level is the floor's (a branch floor's
    depth, a hand-made level's `item_level`) unless the legend gives one; the chest's **band**
    follows from it (the last band whose `from` the level reaches) and is never saved. The band
    names it ("coffer", "kist"), colours it, and gives its sturdiness and hardness. Its lock lies
    in the level's `locks`, as a door's (`World::set_terrain` keeps a lock on a closed door or a
    shut chest only). Opened, its contents lie in the pile on its cell and its entry stays,
    empty, so an open or smashed chest is still named by its band. Nothing closes it again.
  - **Opening** (`o` then a direction, or walking into it; `open:DIR`, `move:DIR`; the
    `open_chest` action, 1000, 6 s): "You open the coffer.", then "Inside you find a dagger and 12
    gold coins." (nothing identified) or "It is empty."; a creak (the shut chest's
    `toggle_noise`, 20, heard as a door opening). **Locked**: only a key that fits opens it
    (`unlock` + `open_chest`, "You unlock the coffer with your skeleton key and open it."); a
    lock without an id (`locked = true`) opens only to a skeleton key. Without one: "The coffer
    is locked.", refused, no time. Whether a chest is locked is never shown until tried (Chest-
    wise shows it, §8 "Talents").
  - **Pick lock** (`pick:DIR`) works on a locked chest as on a door (the same check against the
    same difficulty): picked, "You pick the lock and open the coffer.", `pick_lock` +
    `open_chest`. An unlocked chest: "The coffer is not locked.", refused, no time.
  - **Knock** (the wizard's class ability, §8 "Class abilities"; `use:knock:DIR`, 5 mana, the
    time of a spell): the lock of the locked door or shut chest that way springs open ("The lock
    springs open with a click."); nothing opens. Where nothing locked stands it is refused, with
    no time or mana spent: "There is no lock there to spring."
  - **Kicking** (`k`, `kick:DIR`; the plain `kick`, priced and heard as a door kick, and training
    athletics as one): one draw of the gameplay stream, `1d(sturdiness) ≤ Strength` + athletics +
    the talents' `door_kick` (Lockbreaker's 6) + the worn `door_kick` (`World::kick_force`), against the band's sturdiness (a coffer's 20).
    Held: "You kick the coffer. It holds.", nothing else changes (try again as often as you
    like). Broken, it is **smashed**: "The coffer bursts apart!", the lock gone, and each stack
    inside is destroyed with `[smash] destroyed` percent (50; one gameplay draw a stack, in the
    order held; Deft breaker halves it), **coins and loose gems spared** (never rolled); "The
    potion of healing and 2 scrolls are destroyed.", then what is left, spilled on the cell. The
    terrain becomes the smashed chest (splinters, `Event::ChestBroken`).
  - **Blasts** (`World::blast`, §8 "Digging": a fireball trap's or scroll's blast of 1000): a
    shut chest in the burst whose band's `hardness` is below the blast is **blown apart**:
    every stack rolled at the same percent, **nothing spared** (coins and gems too), "The coffer
    is blown apart!" if seen. A coffer (200) to a strongbox (900) blows apart in a fireball; a
    kist (1200) and the bands above outlast it.
  - **Digging** at a chest is refused ("You cannot dig through the chest.": it has no
    `hardness` of its own as terrain; the band's is read by blasts only). Creatures walk around
    a shut chest and never open or break one.
  - **Trapped chests** (slice C2): a level may put a trap on a shut chest, any kind a door may
    have (`TrapEffect::fits_fixture`: no pit; the same list, no chest-only kinds), never on an
    open or smashed one. It lies in the level's `traps` at the chest's cell, hidden unless
    `known`, and belongs to the chest: nothing steps onto it (`World::step_onto_trap` skips a
    shut chest's cell), and it goes with the lid, opened or broken (`World::set_terrain` drops it
    and its memory). It goes off (`World::spring_trap`) at whoever **opens** the chest (a bump,
    `open`, with a key or without, or after Pick lock picks it), **smashes it with a kick** (as
    the kick lands, before the smash; a kick that holds springs nothing), or **fails by 5 or more
    to disarm it**; Knock springs the lock only. It is aimed from the chest: a shot out of the box
    itself at the victim, a strike at them, a burst around the chest (its cell never stops it), a
    bolt along its `dir`, else from the chest through them, an alarm, a teleport of them. A
    **burst** goes off inside the box and **blows it apart**, however it was sprung: every stack
    rolled at `[smash] destroyed`, nothing spared (coins and gems too), "The coffer is blown
    apart!", whether the burst's own blast was enough for the band's hardness (a fireball in a
    coffer) or not (a frost trap, a fireball in a kist); a shot, a strike, a bolt, an alarm, or a
    teleport leaves the chest as it was (open if it was opened, smashed by the kick).
    `TrapEffect::blows_fixture` is true for a burst. A chest blown apart by a blast from outside
    springs its trap at no one (a trap needs a victim): the trap is dropped with the lid.
    Searching and noticing in passing find it like any hidden trap ("You notice that the coffer is
    trapped: an arrow trap."; a dwarf's stone sense does not help: a chest is no stone);
    **Chest-wise** knows the trap of every shut chest in sight at once, no check, saying the same
    line. **Disarm** works on a chest's known trap as on any (made: the trap is gone, the chest
    still shut). **Trap sense** halves its blow as any trap's (and the worn
    `trap_damage` takes its percent off too, together at most 75 %). Known, it shows in look mode after
    the chest's name ("A coffer, trapped: an arrow trap."); the map and the `view` dump keep the
    chest's glyph over it, the `traps` dump lists it.
  - **Perception** (`PerceptionView::chest_at`): on a cell seen or remembered, the chest's band
    and whether it is shut, open, or smashed (as the cell looks), and with Chest-wise, on a cell
    in sight, whether a shut one is locked. Never its level or what it holds. Look mode: "A
    coffer. A small wooden box with an iron hasp.", "an open coffer", "a smashed coffer"; with
    Chest-wise "a locked coffer", "an unlocked coffer".
  - **Generated chests** (`mapgen/chest.rs`, slice C3): with the branch's `chests.chance`
    percent (§7 "Generation": one floor in five by default) a generated floor gets between
    `count`'s least and most chests, from the level's own chest stream (`chests/<level id>`,
    after its rooms' loot and before its gear, §8 "Determinism"). Each stands on a cell of plain
    floor whose eight neighbours are all walkable and none a door (never in a doorway or a
    corridor, and never cutting a room apart), at least 6 cells from every staircase, never in a
    vault, a shop, or a special room with a feature, sleepers, or a hoard, where nothing lies,
    stands, shines, patrols, or is trapped; two never touch. It is of the floor's item level;
    locked with its band's `locked` percent (a lock only a skeleton key fits, as hard to pick as
    a door's lock on that floor; a level's validation asks no key for a chest's lock); trapped
    with its band's `trapped` percent, of a kind drawn from the branch's `traps` that fit a
    fixture (a branch without traps traps none, though it draws the same), hidden; and holding
    what its band's loot table rolls (§8 "Magic items" "Chest loot"). Draws per chest: its cell,
    the lock, the trap and its kind, the contents, and the mimic draw (`mimic` percent): a mimic
    of the floor's level stands there instead, disguised, carrying exactly the contents, its
    lock and trap dropped and nothing more drawn (§8 "Mimics").
  - **Level files** place chests on the `chest` terrain: `chest = "kist"` (that band's least
    level) or `chest = 7` (a level; default the level's `item_level`), `locked = true` or `lock =
    "id"` and `lock_difficulty` as on a door, and `objects` for what it holds (food in it is made
    as the level is), and `trap` (with `known`, `dir`) as on a door. Refused: a `chest` on other
    terrain, an unknown band, a pit on a chest, a trap on an open or smashed chest, and chests in
    vaults (C7).
  - **Dumps**: `view` shows the glyph; `truth` (dev) adds a line per chest, `chest 7 (hutch) at
    (12, 5): shut, locked, trapped (arrow), holds 3 stacks`; JSON `chests` lists those the player sees or
    remembers (`pos`, `band`, `state`, and `locked`: with Chest-wise for a shut one in sight,
    else null).
- **Searching** (`warlock-core/src/world/search.rs`; data `data/search.toml` §7; #140): one
  check finds every hidden feature, secret doors and hidden traps alike.
  - **Secret doors**: a closed door that looks and acts like the wall around it until it is
    found. It is per-cell ground truth beside that wall, like a lock (`LevelState::secrets`: the
    door it is and its difficulty; saved and hashed): the cell's terrain *is* the wall it looks
    like (`content::secret_look`: of its solid neighbors north, east, south, and west that are no
    door, rock counting as its face, the most common, ties to the first). So it blocks sight,
    sound, light, and every step, creatures' too, is drawn, remembered, and named as that wall,
    and breaks as that wall to digging and blasts; whatever becomes of the wall takes the secret
    with it (`World::set_terrain`). Perception never carries secrets.
  - **Difficulty**: each hidden feature has one, a trap type's (`difficulty`, §7) or a secret
    door's own (`secret = N` in a legend, else `data/search.toml`'s `secret_door`, 14), plus on
    a dungeon branch its `hidden_per_depth` for each level below the first (the Depths: 1, so a
    secret door on depth 8 is 21).
  - **The checks** (§8 "Checks": 2d12 plus a modifier against the difficulty; a natural 2 always
    fails, a natural 24 always succeeds), both of the effective scores (§8 "Stats"):
    - **active**, searching: the Intelligence modifier, plus the Search skill's benefit (+8 at
      100, +12 at 200), plus the race's `search` (elves 2);
    - **passive**, noticing in passing: the Willpower modifier, plus the Spot skill's benefit
      (+8 at 100, +12 at 200), plus the race's `search`, less `passive_penalty` (4).
    The player never sees a difficulty, a modifier, or a roll: an average searcher (Intelligence
    10, untrained) finds a secret door of 14 with 46 % of their turns of searching.
  - **Searching** (`s`: `Action::Search(1)`, replay text `search`; `S`, searching for a while:
    `search:N`, N up to 100, the key sending `data/search.toml`'s `turns`, 10): each turn is the
    `search` action (1000, 6 s; slowed by burden like every price) and gives every hidden feature
    within reach the active check: the eight cells around the player (never their own: a trap
    there has just gone off), and with the Search skill's reach (75 and up: one cell more) the
    cells that far in the player's line of sight. Searching for a while goes on turn after turn
    through the rest loop (`World::keep_at`), so it stops like a rest (a creature coming into
    view, a message, death) and at once when something is found. Every turn of it may earn the
    Search skill a training mark (10 %), whatever it finds. A single search that finds nothing
    says nothing; one for a while says how it ended: "You search for 1 minute and find
    nothing.", "You stop searching after 30 seconds.", "A kobold comes into view; you stop
    searching after 30 seconds."
  - **Noticing in passing**: after every turn the player takes (`run_turn`, before anyone else
    acts: a step, a wait, a turn of a rest, a dig, a meal, or a search), every hidden feature on
    the eight cells around them gets the passive check, if they notice such things at all:
    **elves** everything (the race's `passive_search = "all"`), **dwarves** what is on stone
    (`"stone"`: the cell's terrain is `stone`, stone walls and floors, most of a dungeon, but not
    timber, earth, or trees), **kobolds** hidden traps and never a secret door (`"traps"`),
    drow like elves and duergar like dwarves, and anyone **trained in Spot** everything (a
    thief, a ranger); everyone else nothing. Each thing noticed may earn Spot a mark (50 %).
  - **No limit on trying again** (the owner's rule): every check is rolled afresh, after every
    turn, with no memory of failed ones and no cooldown; time is the only price. Checks draw
    from the gameplay stream only when a hidden feature is in reach (two draws each), so a turn
    with nothing hidden around draws nothing extra.
  - **Found**: a secret door becomes its closed door ("You find a secret door!"); a trap becomes
    known as one seen going off does (`World::know_trap`, shared with `learn_traps`; "You notice
    a tripwire: an alarm trap.", the trap's `sign`, default "a pressure plate"; a door's trap
    "You notice that the door is trapped: an arrow trap.", since a sign is the floor's). Either is
    remembered at once, in the dark too, and the message stops a rest.
  - **Where secret doors are**: level legends (§7 "Secret doors in level files"), indoor vaults
    (stamped with them), and the rooms levels of a branch with `secret_doors` (the Depths: one
    or two a level), only where the level does without them (§7 "Generators"). The showcase hub
    has one, harmless: the hall below the storeroom and the pantry hides a closet with a little
    gold and two gems (§7 "Gems").
- **Digging** (`warlock-core/src/world/dig.rs`; M19, #131): walls give way to digging and to
  explosions, ADOM-like.
  - **Walls that break**: terrain with a `hardness` and what it `becomes` (§7): stone walls 600,
    packed earth 100, granite 3000, each becoming rubble (walkable at 150 %, loud underfoot).
    Terrain without them never breaks (doors, torches, crystals, trees, boulders, mountains), and
    neither does any cell on a level's edge, whatever its terrain, so no level is dug out of. Solid
    rock (`void`) shows its `face`, a stone wall, once a cell next to it is broken open, as
    generated levels wall their open ground, so a tunnel is walled like a corridor.
  - **Digging** (`Action::Dig(dir)`, `dig:E`; key `u`): with a digging tool ready (the first thing
    worn or wielded with `dig` power: a pick-axe, 10, or a mattock, 15, in the Tool slot), into the
    wall next to the player in that direction. Refused, taking no time: without a tool ("You have
    no digging tool ready."), toward open ground ("There is nothing to dig there."), toward
    anything else that does not break ("You cannot dig through the closed door."), and on the
    overworld ("You cannot dig here."). It says "You dig into the stone wall." ("You go on digging
    at the stone wall." on a wall dug into before) and goes on turn after turn like a rest, through
    the same loop (perception batched while no creature can come into view): each turn is a `dig`
    action (1000: 6 s at normal speed, slowed by burden like every price), does work of the tool's
    power × (Strength + 10) / 20 (the score, never the modifier: a pick-axe at Strength 10 does 10,
    at 18 14), that much more by the talents' `dig_percent` (Tunneler's 50) and what is worn
    (§8 "Magic items": a Delving pick-axe), and makes the digging noise where the digger stands (80; §5 "Hearing"). The wall
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

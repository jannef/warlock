# Warlock specification — 8. Architecture (continued): stats and burden

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Stats** (`warlock-core/src/stats.rs`; M14): six stats inspired by D&D, in display order
  Strength (St), Dexterity (Dx), Toughness (To; D&D's Constitution), Intelligence (In),
  Willpower (Wi; D&D's Wisdom), and Presence (Pr; D&D's Charisma). More will come (mostly
  magic): the list is one typed definition in core (id, abbreviation, name per stat), and every
  per-stat value is stored by stat, so a new stat is one entry there (plus a save format bump).
  - **Scores** are integers, 10 the average and the default. Every actor has one in each stat:
    a creature from its type, the player from their character (§8 "Character creation") or,
    without one, from the scenario (§7).
  - **Modifier**: as in D&D 3.5, `floor((score − 10) / 2)` (1 → −5, 8–9 → −1, 10–11 → 0,
    12–13 → +1, 18 → +4; rounded down for scores below 10 and below 0 alike). It is a core API
    for the rules (`stats::modifier`, `Stats::modifier`) and is **never shown**: not in the UI,
    messages, look mode, player-facing dumps, or the wiki.
  - **Potential** (ADOM-like): each stat's natural ceiling, the score it could grow towards.
    Stored and saved, **hidden from the player** like the modifiers. Eating a corpse raises a
    score (§8 "Hunger": by 1, never past the potential, `Stats::raise`), and so does training
    at a check (below; past the potential too, dearer, and never moving the potential). Hunger lowers the **effective** scores for a while (Weak: Strength and Dexterity;
    `Actor::effective_stats`, which combat and burden read; `Stats::adjusted`).
  - **What the player sees**: their own scores as they are now (effective, so lowered while
    Weak), on the character screen (§6) and in the `stats` dump layer; the perception snapshot carries those and nothing else of stats
    (`PerceptionView::own_scores`). A creature's stats are never shown; ground-truth dev dumps
    (`actors`, the world state) show every actor's scores and potentials.
  - **Training** (`data/stats.toml`, `content/stat_training.rs`, `world/stat_training.rs`;
    #231, #722, `docs/design/stat-training.md`): the player's uses of their stats earn
    **training marks** into a signed pile per stat (`Actor::stat_marks`, `PerStat<i32>`, saved
    and hashed, left out of the JSON while all zero; creatures earn none). The pile is the
    whole state: no daily counter, no decay, no cap. Each `[[training.use]]` rule (`stat`, `on`, `chance` 1 to 100) gives its stat
    a mark on its use, one draw of the **training** stream per rule below 100, never gameplay;
    marks never decay and have no cap (saturating at `i32::MAX`). A use may give several marks at
    once (`World::train_stat_times`, a potion's `marks`): one draw per mark below 100, none at
    100; Hard training's `train_percent` (+50) raises every chance below 100 (the draws stay
    as they are). A use that names its own stat (`drank`, `elixir`: the potion's) has rules without a
    `stat`; every other use's rules have one. The chances are plentiful (combat progression
    design §9.1: 8 to 15 marks a game day in a trained stat). The uses (`StatUse`), each
    reported where it happens: Strength burdened (an action but waiting under a slowing load),
    blow (a melee blow of the player's that hits, kicks and bashes too), critical (a confirmed
    critical of theirs in melee), block (a blow on them their shield blocked), dig (a turn),
    kick (a door); Dexterity dodge, parry (a blow on them their Parry turned), shot (a shot or
    throw that hits), sneak (a step within earshot), climb (onto ice, out of a pit); Toughness
    hurt (a blow that costs hit points), big_hit (a big hit or a critical on the player), killed
    (a kill of theirs in melee of a creature of their level or above), resisted (a disease check
    won, not a Willpower one), plunge (through the ice); Intelligence found (a search that finds
    something), read (a scroll), identified (by reading), discovered (a first kill of a type);
    Willpower spot, endured (a long action of more than one turn done to its end: a rest, a
    dig, a search for a while, a meal), resisted_will (a Willpower disease check won),
    hungry_fight (a blow landed while Hungry or worse), cast (mana spent, `World::spend_mana`);
    Presence chat (a chat; asking a topic takes no time and trains nothing), trade (a purchase
    paid or a sale), quest (completed), swapped (a creature making way); any stat drank and
    elixir (a potion or an elixir drunk, sure marks as many as it gives; defined, reached once
    potions land). The uses skills share (burdened, kick, dodge, sneak, spot) are one report,
    `World::train`; the rest `World::train_stat` (a trap noticed in passing reports `spot` there,
    beside Detect traps' `find_trap`).
    **A point's price** (`StatTraining::price`): `price_base (20) + price_per_point (10) ×
    score` (the stored score) up to the potential; past it, with `over = score + 1 −
    potential`, that times `(ramp_divisor (4) + over × (over + 1)) / ramp_divisor` (×1.5, ×2.5,
    ×4, ...); the point reaching the potential costs the base. **A drop's price**
    (`StatTraining::drop_price`): the base price of the point lost (at `score − 1`) divided by
    `drop_share` (2), never ramped. **The checks** (`World::settle`, one rule for both):
    **at every level gained** (`World::level_up`, level by level) on the **levels** stream, and
    **once a game day at 06:00** (#562) on the **training** stream, each stat in display order
    draws once of 65 536, always (six draws a check; none without the data). A pile `m` checked
    against its price `P` (the rise's for `m > 0`, the drop's for `m < 0`) is out of play while
    `quarter (4) × |m| < P`; else the stat moves with chance `|m|^p / (|m|^p + P^p)`, `p =
    odds_power` (3), `|m|` counted up to `8 × P`, in integers (`StatTraining::odds`). A rise
    (at most 40) pays `P`: `m ← max(0, m − P)`, the surplus kept for the next check, one move a
    stat a check; it never lifts the potential (`Stats::train`: the score only; elixirs,
    corpses, and gifts move potentials). A negative pile is not moved yet (the drop,
    `Stats::untrain` with its floors, `DRAIN_FLOOR` and the stat damage plus one, comes with
    the losses, #724's slice T3). A rise says "You feel stronger." (quicker, tougher, sharper,
    more resolute, more confident), after "Welcome to level N." at a level and at once at 06:00
    (it stops a rest); a Toughness rise brings its hit point at once. The daily check is a
    moment of the status loop, so however a rest is cut up the checks come at their hours in
    order; `WorldState::next_stat_check` is saved (left out without the data; scheduled on load
    for a save without it). The dev skip passes no check. No choice, and nothing numeric is
    shown: no marks, price, chance, or potential (the dev `progress` dump layer lists each pile
    with its price, `stat marks St 120/160, To -30/75`; the wiki states the rules exactly).
  - **Stat damage** (`StatDamage`, `Actor::damage`; saved, hashed): points a disease's failed
    checks take off the scores for a while; the rules read the scores less them, never below 1,
    and they mend a point a day each once no disease shows ("Diseases" below). The player sees
    their scores lowered, never the damage as a number apart.
  - **Drain** (#382; `world/drain.rs`, `Stats::drain`, `Actor::drained`; saved, hashed): the
    touch of some undead takes points off a stat **for good**. A natural attack's `on_hit` may
    name a drain (§7 "Data"); a hit that lands, leaves its target alive, and wins its chance
    makes the target check the drain's stat (Willpower by default; its modifier, what talents add
    to every check of that stat, and `drain_checks`) against the drain's difficulty. Made,
    nothing is lost; failed, the points come off the stat's score (a `random` drain draws the
    stat first, one more draw) while the potential stays, so lost points may be won back slowly
    (training, corpses). No drain takes a score below 3 (`DRAIN_FLOOR`) nor below what the
    stat damage on it needs; hit points and mana are cut down to the new maxima, never below 1
    hit point, so a drain never kills. `Actor::drained` counts the points by stat: a temple's
    mending (the `mend` service) gives them all back with the stat damage. The player is told
    in words ("You feel your strength drain away!", or "A deathly chill passes through you."
    when it holds), never the points; their score shows lowered. Life (level) drains, a scroll
    of restoration, and a sustaining affix are not in the game yet. The pale drinker is the
    first drainer (not yet in any dungeon's population). Talents: Unbroken soul, Iron will,
    and the paladin's Divine grace (the `drain` system).
  - **What uses them**: combat (Strength to hit and to damage in melee, Dexterity to hit with
    shots and in Defence, Toughness in hit points and regeneration; "Combat" above) and burden
    (Strength and Toughness scores, never modifiers, in the carrying limit; "Burden" below), and
    kicking doors in (the Strength score against the door's sturdiness; "Doors" above), each
    reading the effective score that status effects and stat damage may lower ("Status effects",
    "Diseases" above); and checks (Toughness, Willpower) resist diseases ("Checks" above). No
    other rule does yet, and no other D&D concept comes with them: no ability checks or saving
    throws beyond the disease checks. Stats and potentials are part of the world state (saved,
    covered by the state hash).
  - **Magic items** (§8 "Magic items") raise scores while worn or wielded: every rule reads the
    scores as they are now (`Actor::effective_stats(content)`: the stored scores plus what the
    actor wears adds, within 1 to 40, then less what its status effects, hunger, and stat damage
    take; potentials unchanged), and so does the player's character screen
    (`PerceptionView::own_scores`). The stored scores are never changed by an item. Hit points
    gained at a level-up read Toughness raised by items but not lowered by a penalty
    (`Actor::raised_stats`).
- **Burden** (`warlock-core/src/burden.rs`; M16, #115), ADOM-like, with 3.5's thirds:
  - **Limit**: 1.5 kg × (2 × Strength + Toughness) × the actor's size's `carry` percent (§7
    `data/sizes.toml`: small 75 %, medium 100 %), from the scores, never the modifiers: a St 10
    To 10 human carries 45 kg, a half-orc St 16 To 14 69 kg, a dwarf St 14 To 16 66 kg, a halfling
    St 8 To 12 31.5 kg, an elf St 6 To 8 30 kg. In grams it is always a multiple of 3, so the
    thirds are exact. The player's size is their race's when the run has a character, else the
    player creature type's; a creature's is its type's. Everything an actor carries counts, worn
    and wielded things included; its own body weight does not. Magic items (§8 "Magic items")
    count by the scores they raise, their carrying affixes (+N kg to the limit, which is then
    rounded down to a multiple of 3 g), and their weight affixes (that much lighter each).
    Inside a heavy hall (§7 "Special rooms", `burden`) the limit is half, again a multiple of
    3 g; a step onto a cell where the player would be Overloaded is refused ("You carry too
    much to go in there.").
  - **Levels**: Unburdened up to a third of the limit, Burdened up to two thirds, Strained up to
    the limit, Overloaded beyond it (`Burden`, `Load::burden`).
  - **Effects**: every action's price ×1.25 Burdened and ×1.5 Strained or Overloaded ("Costs" above;
    the balance skill takes a percent of the extra off, "Skills" below); the Dexterity bonus to
    Defence at most +3 Burdened, +1 Strained, 0 Overloaded, the lower of that and worn armour's
    `max_dex` winning ("Combat" above); footsteps ×1.25 and ×1.5 as loud (§5 "Hearing"). Overloaded,
    the actor cannot move: no step, no walking off a local area's edge, no stairs, no entering a
    location or local area ("You carry too much to move.", refused, no time); everything else works
    (waiting, attacking, opening doors, picking up, dropping, putting on and taking off), as slowly
    as Strained. Picking up is always allowed, even into Overloaded, as in ADOM. An overloaded
    creature opens doors but waits instead of stepping.
  - **Dwarves** (the race's `ignores_burdened`): Burdened costs them nothing (no slowdown, no
    Dexterity cap, no louder steps; `Load::penalty`); from Strained on they are treated like
    everyone else. Their burden is still Burdened in words.
  - **Creatures** follow the same rules from their stats and their type's size (none carries
    enough to be burdened yet; AI that picks things up will stop at Burdened, later).
  - **What the player sees** (`PerceptionView::own_load`: what they carry, their limit, and
    whether they ignore Burdened, all from what they know): their burden in words, in the status
    bar when it is not Unburdened (`Burdened`, `Strained`, `Overloaded`; it gives way late, after
    the time and before the hit points), in the inventory and on the character screen together
    with the weights at which it changes (`Burdened over 15 kg, strained over 30 kg, overloaded
    over 45 kg`; §6), and a message when it changes, right after the action that changed it:
    "You are burdened.", "You strain under your load.", "You carry too much to move.", "You are
    no longer burdened." Never a modifier, never another actor's load. Dev dumps: the
    `inventory` layer has the player's burden and thresholds, JSON dumps `burden` (`level`,
    `limit`), and the `actors` layer every actor's burden and limit.
  - Nothing is stored: the burden follows from the inventory, the stats, and the content, so
    saves and the state hash are unchanged.

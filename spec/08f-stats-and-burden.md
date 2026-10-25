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
    at a level (below; past the potential too, slowly). Hunger lowers the **effective** scores for a while (Weak: Strength and Dexterity;
    `Actor::effective_stats`, which combat and burden read; `Stats::adjusted`).
  - **What the player sees**: their own scores as they are now (effective, so lowered while
    Weak), on the character screen (§6) and in the `stats` dump layer; the perception snapshot carries those and nothing else of stats
    (`PerceptionView::own_scores`). A creature's stats are never shown; ground-truth dev dumps
    (`actors`, the world state) show every actor's scores and potentials.
  - **Training** (`data/stats.toml`, `content/stat_training.rs`, `world/stat_training.rs`;
    #231, the design's §6): the player's uses of their stats earn **training marks**
    (`Actor::stat_marks`, per stat, saved and hashed, left out of the JSON while none; creatures
    earn none). Each `[[training.use]]` rule (`stat`, `on`, `chance` 1 to 100) gives its stat
    a mark on its use, one draw of the **training** stream per rule below 100, never gameplay;
    marks never decay and have no cap. The uses (`StatUse`), each reported where it happens:
    Strength burdened (an action but waiting under a slowing load), blow (a melee blow of the
    player's that hits, kicks and bashes too), dig (a turn), kick (a door); Dexterity dodge, shot
    (a shot or throw that hits), sneak (a step within earshot), climb (onto ice, out of a pit);
    Toughness hurt (a blow that costs hit points), big_hit (a big hit or a critical on the
    player), resisted (a disease check won, not a Willpower one), plunge (through the ice);
    Intelligence found (a search that finds something), read (a scroll), identified (by reading),
    discovered (a first kill of a type); Willpower spot, endured (a long action of more than one
    turn done to its end: a rest, a dig, a search for a while, a meal), resisted_will (a
    Willpower disease check won), hungry_fight (a blow landed while Hungry or worse), cast (mana
    spent, `World::spend_mana`; 20 %); Presence
    chat (a chat; asking a topic takes no time and trains nothing), trade (a purchase paid or a
    sale), quest (completed), swapped (a creature making way). The uses skills share (burdened,
    kick, dodge, sneak, spot) are one report, `World::train`; the rest `World::train_stat`.
    **At every level gained** (`World::level_up`, level by level), each stat in display order
    gets one check on the **levels** stream (six draws a level, always; none without the
    data): it rises by 1 with chance `marks / (marks + need)`, `need = need_base (20) +
    need_per_point (3) × score` (the stored score), divided by `over_potential_divisor` (4) at
    or above the potential, where a rise lifts the potential with the score; at most 40. A rise
    uses up the stat's marks (a failed check keeps them) and says "You feel stronger."
    (quicker, tougher, sharper, more resolute, more confident) after "Welcome to level N."; a
    Toughness rise brings its hit point at once. No choice, and nothing numeric is shown: no
    marks, chance, or potential (the dev `progress` dump layer lists the marks; the wiki states
    the rules exactly).
  - **Stat damage** (`StatDamage`, `Actor::damage`; saved, hashed): points a disease's failed
    checks take off the scores for a while; the rules read the scores less them, never below 1,
    and they mend a point a day each once no disease shows ("Diseases" below). The player sees
    their scores lowered, never the damage as a number apart.
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

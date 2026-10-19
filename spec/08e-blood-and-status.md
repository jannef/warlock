# Warlock specification — 8. Architecture (continued): blood, status effects, healing, checks, diseases

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Blood** (`warlock-core/src/blood.rs`, the pattern; `world/bleed.rs`, applying it; M17):
  big hits and kills stain the floor and the walls around the victim. Presentation state the
  rules keep, like the flow of water: no rule reads it.
  - **When**: a blow that kills leaves a pool on the victim's cell and spray around it; a big hit
    (damage of at least a third of the target's maximum hit points, or any confirmed critical) a
    little spatter. Lesser blows leave nothing; spells, which do no harm, none.
  - **How much**: a kill spills 30 (a stain amount; a cell holds at most 100, soaked), a big hit
    10, scaled by the victim's size (`blood` in `data/sizes.toml`, percent of a medium one's:
    small 60), then by overkill (damage past 0 hit points: up to twice as much for a whole
    second life's worth), a confirmed critical (half again), and a missile (half: it pierces
    rather than cleaves). A rat killed by an arrow leaves a few drops; a critical spear thrust
    through a kobold, a pool and a fan of spatter behind it.
  - **Where**: half of a kill's blood (a third of a big hit's) lands on the victim's cell; the
    rest sprays away from whoever struck, in a fan along the blow (±45° in melee; ±30° along a
    missile's flight, from the shooter), 1 to 3 cells by how much there is (a missile's at most
    2): always on the cells right behind the victim, 60 % of the cells two away, 40 % three away,
    each with a share falling off with distance (60 to 140 % of it). A kill also flecks each
    other cell around the victim (35 %). Spray stops at opaque cells: a wall (or closed door)
    next to the victim takes it on its face, the side facing the victim (twice the share a floor
    cell there would get: the face catches the spray); walls farther away and
    whatever lies behind a wall take none. Terrain without `stains` (water, lava, fire) takes
    none, and nothing is spilled on the overworld (a map of the land).
  - **Accumulation**: stains add up per cell to the cap; their colours mix by amount, and when
    a stain was spilled is the mean of its spills weighted by amount. One kill is subtle; a room
    where a dozen foes died is soaked.
  - **Colour**: each creature type's `blood` (§7): dark red by default, greenish-black ichor for
    the cave beetle; `none` bleeds nothing.
  - **Deterministic, gameplay-neutral**: stains are level state (`LevelState::blood`, sparse by
    cell: amount, colour, when spilled, stained wall faces), saved and covered by the state hash.
    Where drops land is a hash of the turn, the moment, the two actors, and
    the cells, never the gameplay random stream: a fight goes exactly the same with or without
    blood.
  - **What the player perceives** (`PerceptionView::blood`): a stain on an open cell they see;
    one on a wall only when they see a stained face (a visible open cell next to the wall on
    that side), so blood on the far side of a wall stays hidden. Blood is remembered as last
    seen, like piles (`LevelState::blood_memory`). Look mode names it before the terrain ("A
    giant rat (hostile, unhurt), on blood-stained stone floor.", "A blood-spattered stone
    wall."): blood-spattered, blood-stained, or blood-soaked by amount, a wall spattered or
    smeared. The `blood` dump layer shows it as a digit, `1` a few drops to `9` soaked. Drawn
    subtly (§4.8 "Blood").
- **Status effects** (`warlock-core/src/status.rs`, the rules of one actor's effects;
  `world/status.rs`, applying them; data `data/status.toml`, §7; M24, #138): what ails an actor.
  Diseases ("Diseases" below) build on it: they show as one status, Diseased.
  - **Effects**: an actor carries at most one effect of each status (`Actor::effects`, in status
    order), each with a **severity** in degrees (1 to the status's `max`), its progress toward
    its next tick and toward easing, and who caused it (the last to add to it). A new dose adds
    by the status's `stacking`: `add` (degrees add up, to `max`), `highest`, or `refresh` (the
    stronger, its easing starting over). The player is told when one begins (its `start`
    message) and ends (`end`).
  - **Time**: as the scheduler moves the clock, every actor on the current level lives through
    it (`World::pass_time`, in schedule order): a status with a `tick` ticks once every `tick`
    seconds at one degree and `n` times as often at `n` degrees (each second banks its severity
    in degree-seconds; every `tick` of them is a tick), each tick costing `damage` hit points; a
    status with `eases` loses a degree every `eases` seconds, and at none it is gone. Hit points
    come back meanwhile ("Healing" below). Everything happens at its own moment, so it makes no
    difference how time is cut up (many waits or one long journey); at one moment a hit point
    back comes first, then the ticks, then the easing, then the player's hunger (§8 "Hunger": its
    burn, each change of state, and each hit point lost to starving are moments too), then a
    point of stat damage mending, then the diseases' turns ("Diseases" below). Actors on other
    levels are frozen, and the dev time skip passes no time for anyone.
  - **Stats**: a status's `penalty` lowers scores from a severity on (several add up, kept
    within 1 to 40); every rule reads the effective score (`Actor::effective_stats`): combat
    (to-hit, damage, Defence, hit points), burden (the limit), and healing. The player sees their
    scores lowered on the character screen.
  - **Deaths**: a tick that takes the last hit point kills by the status, at that moment
    (`Event::Died`'s `of`; the player's death records it, `Death::of`). A creature that dies of
    a status counts as killed by whoever caused it (`Effect::by`): experience (#133) credits the
    player for creatures bleeding to death from wounds they dealt.
  - **Cures**: `World::cure(actor, means)` ends every status that lists the means in its
    `cured_by` (a `bandage` stops the bleeding); the Heal skill (#132) will apply it.
  - **Causes**: `World::afflict(actor, status, severity, by)` is the one way to give an actor a
    status: wounds and `on_hit` (below), sickness from what the player eats (§8 "Hunger"), and
    later traps and diseases.
  - **Bleeding**: from big hits and confirmed criticals; it costs hit points, stops healing, and
    eases by itself; it shows on creatures ("bleeding"). Blood dripping where the bleeder goes
    (the stains of #127) hooks in where it ticks (`World::drip_blood`).
  - **Poisoned**: from a giant rat's bite (one hit in ten, 2 degrees) and bad food (§8
    "Hunger": the sickness of a rotten corpse, or of some creatures' flesh); it costs hit points
    slowly, halves healing, and from 4 degrees lowers Strength and Dexterity by 1; stronger
    poison lasts longer.
  - **Shown**: the player's own by name (status bar, character screen, the `stats` dump layer),
    those with a `seen` word on creatures the player sees (look mode); never a severity or a
    timer. Effects are part of the world state (saved, covered by the state hash).
  - **Dev**: the action `afflict:ID` gives the player one more degree of a status (no game
    time; recorded in replays), for trying statuses out and for captures.
- **Healing** (`world/status.rs`): a hurt actor's progress toward its next hit point back
  (`Actor::regen`, in hundredths of a second at the normal pace, saved) grows each second by its
  healing rate in percent: 100 normally, the slowest `regen` of its status effects (bleeding 0,
  poison 50, Diseased 50) and of the diseases that show (marsh ague 25), times `[healing]
  resting` (200) while the player rests (creatures never rest), times the `regen` of a special
  room it stands in (a sanctuary 300, §7 "Special rooms"); every `regen_seconds(Toughness)` ×
  100 of it is a hit point ("Combat" above; the effective Toughness). Progress carries over and
  is dropped at full health. A Starving player gets none back (§8 "Hunger"). The regeneration
  affixes of what the actor wears and wields multiply the rate by (100 + their percent) / 100 (§8
  "Magic items"). `World::healing_rate` is the one place that decides the pace.
- **Checks** (`warlock-core/src/check.rs`; M24, #139): a stat put to the test, rolled like an
  attack: 2d12 (the natural roll, `combat::roll_natural`, two draws of the gameplay stream) plus
  the modifier of the stat tested (the effective score's) succeeds at the **difficulty** or
  more; a natural 2 always fails and a natural 24 always succeeds (`check::check`,
  `check::ways` for the odds: against 13 at +0, 78 of 144). Diseases are resisted and fought off
  with them (Toughness, brain fever Willpower). The player never sees a difficulty, a modifier,
  or a roll; they see what comes of it. Nothing limits retrying: the time a disease takes is its
  cost.
- **Diseases** (`warlock-core/src/disease.rs`, the rules of one actor's infections;
  `world/disease.rs`, applying them; data `data/diseases.toml`, §7; M24, #139), modelled on D&D
  3.5's: the pathologies behind one umbrella status, **Diseased** (the file's `status`).
  - **Infections**: an actor carries at most one of each disease (`Actor::diseases`, in disease
    order), each with the seconds to its next turn (`Infection::left`), whether it shows, its
    progressions so far, its checks won in a row, and who gave it.
  - **Exposure** (`World::expose`): an `on_hit` entry naming a disease (its chance drawn first;
    the giant rat's bite, filth fever one hit in ten), terrain the player steps onto
    (`World::wade`: a disease's `wading` terrain, a chance of `per_hour` percent for each hour of
    the step's length (its price over the player's speed), one draw out of 360 000 unless it is
    sure; creatures do not catch diseases from the land), and named sources a rule exposes an
    actor to (`World::expose_to`: `rotten_food`, when the player eats a rotten corpse and keeps
    it down, after the sickness; §8 "Hunger").
    An actor that does not carry the disease makes a check against its difficulty; losing it, it
    catches it (`World::infect`), silently: it **incubates** for its `incubation`, rolled in days.
  - **Turns** (`World::disease_turn`, as the clock moves, in `World::live` like the status
    effects, so it makes no difference how time is cut up): when its incubation is over it
    **shows**: its `onset` is said, and the actor is Diseased (the status, `World::afflict`);
    then every `interval` days it **progresses**: a check. A loss costs its `damage` (stat
    damage, stat by stat in display order, then `hp`), says its next symptom, and starts the
    count of wins over; a win counts, and `recovery` wins in a row shake it off: its `recovered`
    is said, and with no disease left showing, Diseased ends (`World::end_disease`). A disease
    with `recovery = 0` never goes by itself (grave rot).
  - **Stat damage** (`stats::StatDamage`, `Actor::damage`): points off the scores, which every
    rule reads (`Actor::effective_stats`, with the status penalties); never below 1. Toughness
    damage lowers the most hit points through the hit point formula, as in 3.5, and what is left
    is cut down to them. Damage alone never kills (stats stop at 1, `hp` damage at the last hit
    point) unless the disease `kills`: then a progression whose Toughness damage reaches the
    score (Toughness 0) or whose `hp` takes the last hit point kills, by Diseased (`Event::Died`'s
    `of`, `Death::of`: "Died of disease"; a creature "succumbs to disease"), credited to whoever
    gave it. Once no disease shows, a point of each damaged stat mends every day (the seconds
    toward it banked in `StatDamage::mend`, dropped while a disease shows).
  - **Healing**: while a disease shows, hit points come back at the slowest of Diseased's `regen`
    and the showing diseases' own (marsh ague).
  - **Knowing it**: the player knows a disease by name after its first progression (whatever
    came of it; `Infection::is_known`); Heal or Knowledge (#132) could tell it sooner later.
    Until then it is only Diseased; while it incubates, nothing of it can be perceived.
  - **Shown**: Diseased in the status bar and on the character screen like any status, the known
    diseases by name on the character screen (`Diseased  Filth fever`) and in the `stats` dump
    layer (`Diseases filth fever`), the scores with the damage taken off (lowered, in red), and
    the words; never a difficulty, a timer, or a check. Diseased does not show on others.
  - **Messages** (`Event::Disease { actor, disease, news }`, the player's own: `Onset`,
    `Worse(n)`, `Recovered`): an onset and a symptom are news that stop a rest (a batched rest
    turn in which one comes is perceived at once), a recovery is only logged.
  - **Cures**: `World::cure(actor, means)` also ends every disease, showing or incubating, that
    lists the means in its `cured_by` (`remove_disease`); potions and priests will apply it.
  - **Dev**: the action `infect:ID` makes the player catch a disease as if they had lost the
    check (it incubates; no game time; recorded in replays; "You catch filth fever (dev)."), for
    tests and captures. `afflict:diseased` is refused: Diseased comes of a disease.
  - Infections and stat damage are part of the world state (saved, covered by the state hash).

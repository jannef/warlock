# Warlock specification — 8. Architecture (continued): blood, status effects, healing, mana, spells, resources, checks, diseases

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
    status with `eases` loses a degree every `eases` seconds, and at none it is gone; a status
    with `per_action` (Staggered, Pinned, Light-footed) eases by its sufferer's actions instead:
    a degree as each action is taken (its price paid, `World::spend_actions`), but not one given
    during that very action, so at one degree it is the next action's, priced with it. Hit points
    come back meanwhile ("Healing" below). Everything happens at its own moment, so it makes no
    difference how time is cut up (many waits or one long journey); at one moment a hit point
    back (and a point of mana back, "Mana" below) comes first, then the ticks, then the easing, then the player's hunger (§8 "Hunger": its
    burn, each change of state, and each hit point lost to starving are moments too), then a
    point of stat damage mending, then the diseases' turns ("Diseases" below). Actors on other
    levels are frozen, and the dev time skip passes no time for anyone.
  - **Stats**: a status's `penalty` lowers scores from a severity on (several add up, kept
    within 1 to 40); every rule reads the effective score (`Actor::effective_stats`): combat
    (to-hit, damage, Defence, hit points), burden (the limit), and healing. The player sees their
    scores lowered on the character screen. A positive number raises the score instead (Raging:
    Strength +2); `per_degree = true` counts it once a degree (a potion's boost: +2 a degree, so
    Fortified at two degrees is Strength +4; its value times `max` stays within −10 to 10), and the hit points a raised Toughness allows fall back as the status ends
    (`World::keep_hp_within_max`: above the lasting maximum, and above the maximum other statuses
    still raise, never).
  - **Buffs, hexes, and cooldowns** (`world/buff.rs`, #227; design `docs/design/progression.md`
    §4.1 B1): a status may also, while it lasts,
    - **fight** better or worse (`combat = { to_hit, damage, dodge, parry, shield }`, each −20 to
      20, as a combat stance's, §8 "Stances"): to-hit and damage join every attack profile with
      the stance's (`Actor::stanced`), Dodge and Parry the actor's Defence (Dodge stacks, of Parry
      the highest counts), creatures' as well as the player's (a hex is a status on a creature);
      `shield` adds on top of a held shield's Defence (Walled, Shield wall's, #564);
    - **slow attacks** (`attack_cost`, 50 to 1000 percent): every attack's price
      (`World::attack_price`) multiplied by it, on top of `cost` (Walled: 125);
    - **slow** (`cost`, 50 to 1000 percent): every action's price is multiplied by it, after the
      burden and Bloated (`World::burdened`, the one place every price passes); several multiply.
      Only a spell's buff on its caster may hasten (Haste's Hasted, 75; #397), the potion of
      quickness the same Hasted on the player who drinks it (§8 "Potions"), and Light-footed
      (50, the skirmisher's Hit and run, a weapon ability: one action), so a creature's quickest
      step stays what a rest's batching counts on;
    - **lend eyes** (`vision`, an id of `data/vision.toml`): `Actor::eyes` betters its sufferer's
      profile by it part by part, as a unique item's Vision power (the lower threshold, the
      farther darkvision, ultraviolet if any sees it), so what the player sees in the dark
      changes with it (a potion's Cat-eyed, Owl-eyed, Moth-eyed; #567);
    - **resist** (`resist = { fire = 50 }`, each −100 to 75 and not 0): counted with what its
      sufferer wears in `Actor::resistances`, gear-like and so never immunity (a potion's Warmed,
      Chill-proof, Earthed, Proof against poison; #567);
    - **block** actions (`blocks = ["read"]`, ids in `data/actions.toml`): the rules refuse the
      player's action of that kind before it is tried, with no time taken ("You cannot read while
      raging."; a step into a hostile creature is `melee`, into a closed door `open_door`, off a
      local area's edge `stairs`), and the abilities menu greys out an ability that costs one
      (`Unavailable::While(status)`: "you are raging"). A creature's plans consult it for
      steps only: one held fast (a status that blocks `step`: Held, §8 "Birth sign") does not
      step and attacks whoever holds it if they are beside it and hostile, else waits;
    - **hold** (`hold = { check, difficulty, turns }`, #431; `world/hold.rs`): whoever caused it
      (`Effect::by`, saved) holds the sufferer. A hold ends at once (as if worn off: "You are free
      to move.") when the holder is dead, on another level, not beside the held one, or itself
      suffers a status that blocks its steps or slows it (`cost` above 100); asked after the
      player's action, after every creature's turn, and as the turn ends. It lasts at most
      `turns` (3d4 for Held): a dose's degrees are drawn from those dice, and it eases a degree a
      turn (`eases = 6`; `stacking = "highest"`, so a new hit keeps the longer). The held
      player's step (not an attack) is a struggle: a check of `check` (Strength: its effective
      modifier and what talents add to Strength checks) against the holder's grip, its attack's
      `on_hit` entry's `difficulty` (`{ status = "held", chance = 10, difficulty = 15 }`), else
      the hold's own `difficulty` (13). Made, "You break free of the ghast." and the step goes
      ahead; failed, "You struggle, but the ghast holds you fast." and the turn is spent (a
      step's price). Creatures never struggle. A hold no one caused (`afflict:held`) ends only by
      its turns or a struggle. Drowning grip's Held is the same status;
    - **be followed** (`follows = "exhausted"`): as it ends, worn off or cured, the status it names
      begins at one degree, caused by whoever caused the one that ended (Raging, then Exhausted:
      the ending's message, then the beginning's). A chain never leads back to where it began, a
      status with `follows` must end (`eases` or `cured_by`), and nothing follows into or out of
      Diseased.
    A **cooldown** is a status with `eases` and no tick given as an ability is used: while it
    lasts, the ability is `Unavailable::While` it ("you are winded"), a visible limit that only
    time lifts; nothing counts uses. An ability that buffs the player calls `World::afflict(player,
    status, 1, player)`. The class abilities that give them come with the class tables; the
    built-in data has the barbarian's rage as the example: Raging (Strength and Toughness +2,
    Intelligence −2, no reading or casting, while the Rage resource lasts, "Resources" below),
    then Exhausted (10 minutes: Strength −1). A status that `blocks` casting also refuses every
    spending of mana (`World::spend_mana`) and greys out an ability that costs mana.
  - **Deaths**: a tick that takes the last hit point kills by the status, at that moment
    (`Event::Died`'s `of`; the player's death records it, `Death::of`). A creature that dies of
    a status counts as killed by whoever caused it (`Effect::by`): experience (#133) credits the
    player for creatures bleeding to death from wounds they dealt.
  - **Cures**: `World::cure(actor, means)` ends every status that lists the means in its
    `cured_by` (a `bandage` stops the bleeding). The Heal skill treats a status with a `treat`
    table instead (#549, §8 "Treating oneself"): bleeding stops on a made check and may be
    tried again; a poison eases a degree, one try per dose; a disease is eased, one try each.
  - **Causes**: `World::afflict(actor, status, severity, by)` is the one way to give an actor a
    status: wounds and `on_hit` (below), sickness from what the player eats (§8 "Hunger"), and
    later traps and diseases.
  - **Bleeding**: from big hits and confirmed criticals of slashing and piercing blows (the
    status's `wound.from`, §8 "Damage types": a club, a fall, or a fireball opens no wound; an
    untyped blow does); it costs hit points, stops healing, and
    eases by itself; it shows on creatures ("bleeding"). Blood dripping where the bleeder goes
    (the stains of #127) hooks in where it ticks (`World::drip_blood`). Its ticks are untyped:
    nothing resists them.
  - **Poisoned**: from a giant rat's bite (one hit in ten, 2 degrees) and bad food (§8
    "Hunger": the sickness of a rotten corpse, or of some creatures' flesh); it costs hit points
    slowly, halves healing, and from 4 degrees lowers Strength and Dexterity by 1; stronger
    poison lasts longer. It is damage of the kind poison (`damage_type`, §8 "Damage types"): a
    creature immune to poison is never poisoned, and a resistance takes its share off each dose
    as it is given (rounded to the nearest; 2 degrees at 50 % is 1), never off the ticks.
  - **Asleep** (#405; `sleeps = true`): a teeming den's sleepers are placed with it (§7 "Special
    rooms"). A creature suffering a status that `sleeps` takes no turn and is unaware of
    everyone; it has no easing (time never wakes it), and ends as noise loud enough, a blow, or
    a neighbour's cry wakes the sleeper. It shows on creatures ("asleep").
  - **Disguised** (#616; `hides = true`): a mimic is placed with it (§8 "Mimics"). A creature
    suffering a status that `hides` takes no turn, is unaware of everyone, is targeted by no
    creature, and is a shut chest to the player's perception; nothing but being found out ends
    it, and nothing is said of the status itself. It never shows (no `seen`).
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
- **Mana** (`warlock-core/src/mana.rs`, the pure rules; `world/mana.rs`, applying them; #223):
  a second pool beside hit points, built the same way. Spells spend it ("Spells" below: the
  casters' class spells and those learned from scrolls), as do rods (§8 "Rods"), the bard's
  inspire when short of inspiration, and a summoner's summons (§8 "Creatures"); Blood magic
  refills it. Every
  new system says in its issue how it interacts with mana, if at all.
  - **The maximum** (`Actor::max_mana`): `mana_base` (the race's and the class's `mana`, §7; a
    creature type's `mana`, default 0; a scenario without a character gives the player's) plus
    what magic items add (`max_mana` affixes, §8 "Magic items") plus the modifiers of the three
    mental stats, Intelligence, Willpower, and Presence, the effective scores', at least 0, then
    deepened by a trained **Concentration** (§8 "Skills"; `world/concentration.rs`): +1 at 25 to +5
    at 100 and +8 at 200 (`data/skills.toml`), never more than a fifth of the pool without it
    (rounded down, `mana::deeper`: a pool under 5 gains nothing), so it also follows the skill
    down when an item or a talent raising it comes off. Every actor starts with all of it. Each level gained up to the experience file's `mana_level` (60,
    past `hp_level`, so casters keep growing after hit points stop) adds `max(0, class mana ÷ 2 +
    the better of the Intelligence and Willpower modifiers)` (`docs/design/progression.md` §3.7;
    the race and class values are its table; the stats as worn magic raises
    them, not as a status lowers them) to the maximum and to the mana the player has; without a
    character the scenario's `mana` stands in for the class's (`World::level_up_mana`).
  - **When it falls**: taking off what raised it cuts the mana down to the maximum without its
    status effects (`Actor::lasting_max_mana`, `World::keep_mana_within_max`, after every player
    action, as hit points are kept by `lasting_max_hp`); damage to a mental stat from a disease
    cuts it down to the new maximum (as Toughness damage does hit points); a status effect's
    penalty passes, so the mana kept under it stays, none comes back above the lowered maximum,
    and the player is shown no more than it.
  - **Regeneration** (`World::mana_rate`, the one place that decides the pace, the twin of
    `World::healing_rate`): an actor below its maximum gets a point back per `max(60 s, 360 s −
    12 s × Willpower)` of game time at the normal pace (`mana::regen_seconds`; Wi 10: 4 minutes;
    Wi 25 and up: 1 minute), its progress banked in `Actor::mana_regen` (hundredths of a second
    at the normal pace, saved, dropped at the maximum). The pace in percent: the slowest
    `mana_regen` of its status effects (poison 50, Diseased 50) and of the diseases that show
    (brain fever 25), times `[healing] mana_resting` (200) while the player rests, times the
    `mana_regen` of a special room it stands in (a sanctuary 300), times (100 + the mana
    regeneration affixes of what it wears and wields, plus its talents' percent: Tidal and Clear
    mind +25, plus a trained Concentration's: +5 at 20, +40 at 100, +60 at 200) / 100, resting's 200 raised by Meditation's +50 % to 300;
    a Starving player gets none (§8 "Hunger"; Weak lowers only the stats it lowers). Burden, temperature, and combat stances do
    nothing to it. A point back is a moment of `World::live`'s loop ("Status effects" above: it
    comes with a hit point back, before the ticks), so it makes no difference how time is cut up;
    actors on other levels are frozen, and the dev time skip passes none.
  - **Draining**: a status with `mana_damage` drains that much mana a tick (§7), never below 0
    and never a death. **Ebbing** does (#403): a draining room's status (§7 "Special rooms"),
    held by everyone with mana inside the room, it drains a point every 20 seconds and stops
    mana coming back (`mana_regen` 0); one degree, renewed while one stays, eased half a minute
    after leaving ("Your power ebbs away.", "Your power stops ebbing."; on creatures "ebbing").
    Anchored mind (§8 "Talents") spares the one who has it.
  - **Silenced** (#409): a silent vault's status (§7 "Special rooms"), held by everyone inside,
    creatures too: one degree, renewed while one stays, eased ten seconds after leaving ("The
    silence closes around you.", "Sound returns."; on creatures "silenced"). It `blocks` `cast`,
    `read`, and `throw_voice`: the player's spells, scrolls, and thrown voice are refused with no
    time ("You cannot cast spells while silenced."), mana cannot be spent, and a caster creature
    fights in melee instead of casting (`ai.rs::fight`). Everyone on a level that gives a status
    takes it as the player arrives (and as a run starts), so one standing in the vault is silenced
    before its first action.
  - **Unwelcome** (#413): a shrine of a side's status (§7 "Special rooms") on everyone of the
    other side inside it, creatures too: one degree, renewed while one stays, eased half a minute
    after leaving ("You feel unwelcome.", "You no longer feel unwelcome."; on creatures
    "unwelcome"). Wounds heal at half the pace (`regen` 50) and blows land a little less often
    (`combat = { to_hit = -1 }`).
  - **Spending**: `World::spend_mana(actor, base)` is the one way to spend it, refused (nothing
    spent) when the actor has less than the cost: `base` as its talents make it
    (`Actor::mana_cost`, `TalentMods::mana_cost`: Frugal caster's −20 % and the `mana_cost`
    of the magic items worn (of Thrift, unidentified too), summed before the one rounding and
    never more than 50 % off together, rounded to the nearest, never below 1 for a cost above 0). The player's spending of any is the `cast` use of the
    stats (§8 "Stats": a Willpower mark, 20 %, from the training stream). The casters' spells call it (spec §8
    "Class abilities": after every check, so a refused cast spends nothing), as does the dev
    action `drain:N`, which stands in for an ability's cost ("You spend 5 mana (dev).",
    the cost as spent; refused, "You have only 3 mana (dev).", when short; no game time; recorded
    in replays).
  - **Talents** (#242, the Scholar tree, §8 "Talents"): Deep well (`max_mana_per_levels`, 3
    ranks: 1 mana per 4 character levels, at least 2, a rank; handed out into `mana_base` and
    the mana at the pick and as each later level comes, as Hardy's hit points, past `mana_level`
    too), Clear mind (`mana_regen_percent` +25), Meditation (`mana_resting_percent` +50, on
    resting's pace), Frugal caster (`mana_cost_percent` −20), and Blood magic (`blood_magic`, an
    active in the abilities menu, `use:blood_magic` in replays: 5 hit points for 3 mana, up to the
    maximum, for the time of a wait; refused, no time, with 5 hit points or fewer ("You have too
    little blood left to give.") or full mana ("Your mana is already full."); not a blow: no
    Protection, no pain, no training, never a death).
  - **Shown**: the player's own mana and maximum (`PerceptionView::own_mana`): the status bar's
    `MP 5/12` right after the hit points, giving way like them (§6), the character screen beside
    the hit points, the dump header's `mp`, and JSON dumps' `mana`. Never a creature's mana, and
    never the pace (hidden like `regen_seconds`); the dev `actors` layer shows every actor's
    (and, with any mana, its caster level, `cl N`).
  - **Creatures** (#525): a creature's maximum, pace, and caster level read what it wears and
    wields exactly as the player's do (`max_mana`, `mana_regeneration`, and `caster_level`
    affixes through `Mods::worn`); it is made with all of its larger pool, and a summoner pays
    for its summons from it. A creature without such things is as its type makes it.
  - **On a kill** (the affixes design §4, decision 7; #632): the `mana_on_kill` of what the
    slayer wears and wields (of Siphoning) raises its mana by their sum, never past its
    maximum, when its own blow, shot, throw, or spell kills, and the slain stood within
    `SIPHON_RANGE` (3) cells (Euclidean, as a radius: `dist2` within `radius2(3)`). A death by a
    status tick (poison, bleeding), a trap, or anything else that is not the slayer's own attack
    gives nothing, and a summoned creature's kill is its own. Creatures' kills count the same
    way. Read once, at the kill point of an attack (`World::siphon`, `world/siphon.rs`, called
    from `fight.rs::resolve` and `spell.rs::spell_strikes`); no draw. The player is told
    "close by", never the three cells.
    Mana is part of the world state (saved, covered by the state hash; an actor without any
    serializes as before).
- **Spells** (the magic design, `docs/design/magic.md` §3 to §8, as the spell scaling design,
  `docs/design/spell-scaling.md`, revised it: levels for tiers, ladders and hooks for the cap
  and the lift, caster level for learning only; `content/spell_def.rs`,
  `world/caster_level.rs`, `world/cast.rs`, `world/learn.rs`; #394, #395, #396, #397, #759): the
  spell data, the one cast path, learning, and the first spell list. A spell's growth is
  written in its own file as ladders on the caster's stats and printed in the wiki as written;
  talents and items act on it through the hooks. The casters' spells (§8 "Class
  abilities") are spells of `data/spells/` cast by it, and so are the spells learned from
  scrolls and the spells scrolls cast. `docs/dev/spells.md` has the details.
  - **The first list** (#397, design §7; names left for the world-language review, #378):
    by spell level (the spell scaling design §5.8): 0 Evil eye, Light (#589: the scroll of
    light's; Lit, its status, sheds a light about the caster, `light` on a status); 1 Mend, Ray
    of frost, Spark; 3 Barkskin, Blessing, Cure wounds, Curse of sloth, Hex of weakness, Magic
    missile; 4 Burning hands, Cause wounds; 6 Arcane bolt, Frost, Identify, Sleep; 7 Blight,
    Scorching ray, Web; 11 Call lightning, Fireball (the wizard's Fire burst folded into it),
    Haste, Lightning bolt, Remove disease; 12 Flame. A spell is cast by its
    data id (`use:magic_missile:12,5`, any `use:ID` that is no class or birth sign ability; one
    that is no spell either is a weapon active's, §8i "Weapon abilities");
    the abilities menu lists the spells known by spell level, then path order. Sleep and Web are
    **area hexes**: a burst whose status falls on everyone it reaches but the caster, friend or
    foe (`World::cast_area_hex`). Haste is the one status that hastens (`cost` 75; statuses may
    cost 50 to 1000 percent, and only a buff, on its caster, may hasten: a hex's or a damage
    spell's status that would is refused when the content loads).
  - **Identify** (#592): a divination `utility` spell on oneself that learns the truth of one
    stack the caster carries, as the scroll of identify does (§8 "Magic items"): cast as
    `use:identify:c` (an aim that is no cell names a stack by position, `Action::CastOn`), from
    the abilities menu through the identify list. Its cost is given outright, `mana = 60`, not
    by its level (a spell's `mana` replaces its level's price: no shape or resolution; armour
    and the talents still count), so a caster of the first levels cannot cast it at all; its
    level, 6, is how hard it is to learn. Refused, no time and no mana,
    without the mana, without a stack, or on one already known.
  - **Guaranteed spells** (design §8): a class's level table (`abilities`, §8 "Class
    abilities") gives its spells as class abilities with a `spell` effect, at the level each
    comes at, never above the class's caster level then (character level × `casting` ÷ 100, at
    least 1; the content test). Wizard: Ray of
    frost (1), Magic missile (3), Burning hands (12), Arcane bolt (18). Priest: Mend (1),
    Blessing (3), Cure wounds (3), Remove disease (12). Druid: Spark (1), Barkskin (3), Call
    lightning (12), Flame and frost (50). Witch: Evil eye (1), Hex of weakness (3), Cause wounds
    (12), Curse of sloth (32), Blight (40). Bard: Mend (1), Blessing (6). Paladin: Blessing
    (6). Ranger: Barkskin (6). Every other class none. Every class has some mana (the
    barbarian and the warrior 1); a race adds none unless its data says so.
  - **Spell scrolls** (§8 "Scrolls"): every spell comes on one scroll (`scroll_of_ID`, a
    `read` table of kind `spell`), which casts it through the cast path at the scroll's own
    `rungs` of the spell's damage or heal ladder (default 0, the base; at most the ladder's
    length) plus the reader's `scroll_rungs` (Scroll adept), the reader's other hooks applying
    and their reach and degrees, free of mana and never refused for what the reader knows
    (Spell recall may keep it; a death by it is the scroll's), and teaches it to whoever
    studies it. The scrolls of fireball (`rungs = 1`, 3d6), lightning (`rungs = 2`, 4d6), and
    frost (0, 2d6) are spell scrolls like the rest (the spell scaling design §7.4: their burst
    and bolt readings went; a `read` of kind `burst` or `bolt` is refused naming the
    replacement).
    Each scroll's item `level`, loot `frequency`, and value are its own; by convention (the
    objects' README) about half its spell's level, rounded up, at least 1 (spell levels 0–1:
    (1, 60, 40); 3–4: (2, 45, 80); 6–7: (4, 30, 130); 11–12: (6, 20, 200)); a shop's goods are
    drawn by loot frequency (100 for what is never loot), so a general store lays out low
    levels more often (§7 "Shops").
  - **Levels** (the spell scaling design §5, decisions 1, 9, 10, 16): a spell has a **level**,
    0 to 70 (`MAX_SPELL_LEVEL`), one number with two meanings: its **base mana** (the level, at
    least 1) and the **caster level at which a scroll of it teaches for certain**. A spell of
    level 0 or 1 is a *cantrip* (1 mana; anyone with mana learns it for certain). There is no
    table of levels, no ceiling, and no check on how much a spell of a level may do: its own
    data says it (decision 16). A `tier` key is refused, saying to use `level`.
  - **Cost table** (`data/spells.toml`): the **shapes** multiply the base mana (ray, bolt,
    touch, self ×1; cone ×1.6; line ×1.8; a burst ×1.8, ×2.4, ×3 by radius 1 to 3, its
    largest), and each **resolution** multiplies it (`attack`, `auto` ×1, `area` at its shape's
    cost, `touch` ×0.7). First guesses, in data.
  - **Spells** (`data/spells/`, one `[[spell]]` file each, numbered in path order: `SpellId` an
    index, in the content fingerprint): id, name, description, level, the **tags** school
    (evocation, necromancy, abjuration, restoration, transmutation, enchantment, divination),
    kind (damage, heal, buff, hex, cleanse, utility), shape, and damage type (a `damage` spell's,
    needed); a resolution that fits the shape (`area` exactly for cone, line, and burst, `touch`
    only for touch, `attack` for a ray or a bolt, `auto` for any single target); its **base**
    reach, each shape exactly its own (`range` 1 to 20 for a ray, bolt, or burst, `radius` 1
    to the burst table's length for a burst, `length` for a cone, 1 to 5, or a line, 1 to 20);
    `damage`, a damage spell's base roll (a `Roll`, `"2d6"`, `"3d6+1d10"`); `[spell.effect]`
    for the other kinds (a heal's base roll `heal = "2d6"`, a buff's or hex's `status` and base
    `severity`, a cleanse's `cures` and `diseases`, a damage spell's `status` suffered by
    everyone it harms); `aim` (an attack or touch damage spell's: the stat whose modifier aims
    it, default Dexterity); `casts` (its time in `cast` actions, 1 to 10); `open_sky`; `mana`, a
    cost given outright in place of its level's (Identify); `[spell.effect] identify` (a
    `utility` spell on `self` that asks for a stack); `[spell.cast]` (the `effect` drawn,
    `noun`, `verb`, `message`, `noise`, and a burst's `blast`, `heat`, `frost`); and
    `[spell.grows]`, its ladders (below). Retired and refused, each naming its replacement:
    `tier`, `[spell.damage]`'s table form (`dice`, `per_levels`, `max_dice`, `bonus`),
    `max_range`, `max_length`, `max_radius`, `[spell.scales]`, `[spell.scroll] level`.
  - **Ladders** (the spell scaling design §4, §6; `content/spell_ladder.rs`): `[spell.grows]`
    gives each dimension the spell has an optional ladder: `damage` and `heal` (rolls),
    `range`, `length`, `radius` (cells), `degrees` (of a buff's or a hex's status). A ladder is
    one string, the stat's id then rungs `AT VALUE` separated by commas (`"intelligence 13 3d6,
    16 4d6, 19 5d6, 25 5d6+2"`), at most 8, `AT` a score 1 to 40 strictly ascending. The
    loader refuses a ladder for a dimension the spell lacks, an unknown stat, a roll that
    weakens (each rung's lowest and highest at least the one before's, the base first, and not
    the same roll), cells or degrees not strictly ascending from the base or past the engine's
    limits (a range to 20, a cone to 5, a line to 20, a radius to the burst table's length,
    degrees to the status's `max`). **No check reads how much a rung does** (decision 16). The
    rung a caster reaches is how many rungs their **effective score** in the ladder's stat
    (statuses, stat damage, drains, worn items counted) is at or above; below the first, the
    base, so a low score never shrinks a spell. The cast rung is that plus the rungs from
    outside (the hooks), never past the last rung. The shipped ladders reproduce today's reach
    and duration exactly at the scores where the old rounding changed them (Willpower 14, 18;
    Presence 14, 18) and give every rolling spell a ladder (§12 of the design).
  - **Hooks** (the spell scaling design §7.1; `content/spell_mod.rs`, `world/spell_mods.rs`,
    `world/spell_rung.rs`): talents and items act on a spell through six named hooks, the
    fields of one `SpellMods`, summed once a cast from the caster's talents (`spell_mod`
    effects, filtered by the spell's tags: `when = ["cold"]`, or the choice of a talent chosen
    for a school or an element) and worn items (unidentified too): (1) **rung**, rungs higher
    on the roll's ladder for an own cast (`rungs`), a spell scroll read (`scroll_rungs`), a
    wand used (`wand_rungs`), or on the reach and degrees ladders; (2) **dice**, `sides`
    (every die bigger), `dice` (more of the first group's kind), `bonus`, applied to the rung's
    roll as `grow(sides).more(dice).plus(bonus)`; (3) **roll**, `reroll` (a die showing it or
    less drawn once more, that draw standing); (4) **rider**, statuses laid on every target the
    spell harms (merged with its own `effect.status`), none on a miss or a heal; (5) **reach**,
    `cells` on a range or length, `radius_cells` on a burst's radius; (6) **degrees**. Besides
    them a `cost` percent (School focus's) joins `spell_mana_percent`. The order: rung → dice
    → roll (with rerolls, again on a critical) → harm → riders; reach and degrees beside.
    Nothing holds them but the engine's own limits (`MAX_REACH` 20, a cone 5, the burst
    table's length, a status's `max`). The shipped users: Empower (a rung), Archmage (two),
    Keen mind (a die size), School focus (10 % off, a cell), Scroll adept (a scroll rung), Wand
    mastery (a wand rung), Sure hand (rerolls 1s), Brimming (a die more, +10 % mana), Numbing
    cold (cold spells' targets a degree Sluggish); the affixes `spell_rungs`, `spell_bonus`
    (Potent: +1 / +2 / +3), `spell_sides`, `spell_dice`, `spell_reroll`, `wand_rungs` (of
    Wands), `spell_reach` (of Farcasting, cells), `spell_duration` (of Lingering, degrees).
    Derived at cast time; nothing saved.
  - **No cap** (decision 16): nothing checks a spell's strength when the content loads, and
    nothing holds a roll when it is cast: the rungs, the hooks, and a critical add what they
    add.
  - **Casting** (`World::cast_spell`, `world/cast.rs`; `Action::Cast`, `use:arcane_bolt:12,5`):
    - **Cost** (the one hook, `Actor::spell_cost`): `SpellRules::mana`: the spell's level, at
      least 1 (no caster level anywhere in it: a known spell is cast at its level's mana, and
      the pool is the gate) × the shape's percent (a
      burst's at its own `radius`) × the resolution's `cost` percent × (100 + the summed
      `spell_cost_percent` of what is worn, `data/objects/`: chain mail 10, scale 20, banded 30,
      plate 40, great helm and large shield 10, tower shield 20; each piece's less its own
      Mage-wrought `spell_ease`, never below 0, a piece eased to 0 hindering nothing, so no
      longer armour for Battle caster's grip) %, rounded to the nearest once,
      at least 1; then the cost hook (School focus's −10 % for a spell of its tag) and
      Archmage's −50 %, Empower's −10 %, and Brimming's +10 % (`spell_mana_percent`), summed, at
      least 1 (`Actor::spell_base`, what
      `World::spend_mana` is given), and Frugal caster's and of Thrift's cut there. Spent once the cast is sure
      to go off; refused, with no time and no mana, without it ("You have too little mana.").
    - **Reach**: the base range or length climbed on its reach ladder (`reach_rungs` on top),
      then the reach hook's cells (School focus per focus on one of the spell's tags, the worn
      `spell_reach`, of Farcasting); a burst's radius on its own ladder plus `radius_cells`;
      held only by the engine's limits (`Actor::spell_reach`). A burst still costs by its base
      radius. A buff's or a hex's degrees: the base severity climbed on its degrees ladder,
      then the degrees hook (the worn `spell_duration`, of Lingering), at least 1; the status's
      `max` holds it (Blessed, Hexed, Weakened, and Sluggish go to 3, each degree its `eases`
      again; `Actor::spell_degrees`).
    - **Dice**: the roll at the rung reached (a scroll's or a wand's own rungs plus the reader's
      scroll or wand rungs in place of the stats') with the dice hook applied
      (`Actor::spell_roll`, what the abilities menu will show), drawn with the roll hook's
      rerolls; nothing holds the roll. The mishap backfire casts the spell's base, no hooks.
    - **Resolution**: `area` (burst, cone, line) strikes everyone in it with one roll, in
      ascending actor id order, no roll to hit, Evasion halving (`World::spell_hurts`),
      resistances taking their share, no armour; `auto` (ray, bolt) strikes the first creature
      on its line, never missing; `attack` rolls 2d12 + the modifier of the spell's `aim` stat
      (Dexterity unless it names another; plus the items worn's `spell_to_hit`, of the Magus; a
      scroll's or a wand's always Dexterity's alone) against
      the target's Defence (Deflect missiles counts), threat 22 (an own cast: less the items
      worn's `spell_threat`, Baleful, never below 2, `Actor::spell_threat`; a scroll's or a
      wand's 22) and critical ×2 as a blow's
      (`combat::strike`); `touch` the same against a creature next to the caster. Only attack
      and touch can critical. A hex falls on the hostile the caster sees at the cell aimed at.
    - **Shapes**: a burst flies and goes off as a scroll's (`burst_center`, `cast_burst`); a line
      runs through everyone as a scroll's bolt (`bolt_line`, `cast_bolt`); a **cone**
      (`cone_cells`) fans from the caster toward the aim, a quarter turn wide (the angle's
      cosine squared at least a half), as long as its length, in line of sight, drawn as a burst
      around the caster over its cells; a ray or a bolt flies to the first creature on its line.
    - **Determinism**: a cast draws only its own rolls (the attack roll, the dice, one draw a
      die in group order and one more a die the reroll hook sends back, an area's one roll for
      all); a spell without a roll, and a rider, draw nothing. Nothing new is saved.
  - **Caster level** (`World::caster_level`, `Actor::caster_level`; derived, never stored): the
    player's level times their class's `casting` percent (`data/classes/`, 1 to 100, default
    25: wizard, witch, priest, and druid 100, bard and aristocrat 75, paladin, ranger, and monk
    50, everyone else 25; 100 without a class), a creature's its type's level, rounded down; plus
    the `caster_level` talent effect (racial and birth-sign gifts are talents; flat, or `value`
    per `per_levels` character levels: the wizard's Arcane training, +1 per 5) and affix effect;
    never below 0, and at least 1 for anyone whose maximum mana is
    above 0. It sets learning only; never a spell's roll, reach, or mana (decisions 1 and 10). The player knows
    it: the dump `stats` layer's `Caster level N` (after the scores), JSON dumps' `caster_level`
    (`PerceptionView::own_caster_level`).
  - **Learning** (`world/learn.rs`, `content/learning.rs`; #396): a character knows a set of
    spells for good: those their class abilities give and those they learned
    (`Actor::spells`, saved by id, left out of the JSON while empty). Either kind is cast from
    the abilities menu's *Spells* group. A spell scroll **teaches** its own spell (fireball
    Fireball, lightning Lightning bolt, frost Frost). Every scroll is a real spell (#592): each
    teaches one, the scroll of identify Identify and the scroll of light Light.
    - **Study** (`Action::Study`, `study:b`; the `Study` key command, `L`, next to Read):
      refused with no time, the scroll kept, for no such stack, a thing that teaches nothing
      ("There is no spell in a dagger to learn."), a scroll still under its label ("You do not
      know what this says."), a spell known ("You know this spell already."), a character with
      no mana at all ("You have no gift for magic."), and what a status blocks (Raging blocks
      `study`). Identification and learning are separate: any identified scroll whose spell the
      player does not know may be studied, read or not. A scroll of a known spell still reads,
      sells, and gives as any other.
    - **Time**: `turns_base + turns_per_level × level` (100 and 20: ten minutes and two a spell
      level; Fireball, level 11, 32 minutes) turns of the `study` action (`data/actions.toml`,
      1000), passed like a meal (`World::keep_at`); Quick study takes
      half. It stops as a rest does (a creature that disturbs a rest comes into view, a message
      is said: a blow lands, a status begins); an interrupted study loses the time and keeps the
      scroll (design §14 q1). Once begun, the player cannot stop it themselves.
    - **The check**, as the last turn ends, on the scroll used up (Spell recall never keeps a
      studied scroll, design §14 q7): overreach is the spell's level less the caster level. At
      0 or less the spell is learned, no draw. Above, a 2d12 check with
      the Intelligence modifier + the skills' `study` benefit (Spellcraft and Decipher script,
      each value ÷ 10, at most 20) + the class's `affinity` with the spell's school
      (`data/classes/`, −10 to 10) + Quick study's bonus (2), against `base` (12) +
      `per_overreach` (1) × overreach′ + the sum of the levels of the spells known ÷
      `known_per_point` (20) (`data/study.toml`), overreach′ being the overreach with
      Overreach's third off, rounded down (`study { overreach_percent = -33 }`). Made: "…you
      have learned Frost."
    - **Failure**: "The words twist away from you, and the scroll crumbles to dust.", then a
      **mishap** (`data/mishaps/`, one file each): one draw by weight among those of the band
      (slight below `bad_from` (6) levels of overreach, bad from it, terrible from
      `terrible_from` (15)) and
      the lucky ones, its message, then its effect: nothing; a status (Sluggish, Weakened,
      Hexed; Blessed for a lucky one); the scroll's own spell going off on the reader (a damage
      spell's base roll as a burst around them; a death by it is the scroll's); a share of
      the mana lost; a teleport on the level (the teleport trap's); a stat training mark lost;
      a drain (`World::drain`: a Willpower check or points off a stat for good); a hostile
      summoned next to the reader (a dretch); or, lucky, a training mark of Intelligence or the
      spell learned anyway. A failure loses only that scroll: the spell can come again.
    - **Training**: the `study` use (Spellcraft's and Decipher script's marks; an Intelligence
      mark), whatever comes of it; in a scriptorium (§7 "Special rooms") Decipher script's
      mark is twice as likely.
    - **Determinism**: the check's two gameplay draws only when there is a check, the mishap's
      one only on a failure, then its effect's own.
    - **Shown**: the chance in words only (`StudyRules::words`: "certain", "easy" from 96 of 144
      ways, "a struggle" from 36, "far beyond you"); the study list
      (`PerceptionView::studies`) and its dialog; the dump `spells` layer (known spells with
      spell level and cost and their roll and reach now, `Fireball (level 11, 26 mana): 4d6 fire,
      range 11, radius 2`, then the scrolls one could study
      with the chance in words) and JSON's
      `spells` and `studies`; the exact odds only in the dev `progress` layer.
- **Resources** (`warlock-core/src/resource.rs`, the pools and time's rules;
  `content/resource.rs`, the data; `world/resource.rs`, applying them; #225): alternative
  resources, integer pools beside hit points and mana (which everyone has) that a character holds
  once something grants them. A character may hold any number at once; creatures hold none
  (their rage would never show). The class features of the progression design's §4.18 (the
  barbarian's Rage, the ranger's Focus, the thief's Openings, the monk's Ki, the bard's
  Inspiration) move onto them in follow-up changes (R1–R5); the barbarian's Rage has (R1,
  "Rage" below), and the ranger's Focus (R2), the thief's Openings (R3), and the monk's Ki and
  the bard's Inspiration (R4) have (below). Mana is separate: nothing converts between them;
  Inspiration is the one resource spent in place of mana.
  - **Data** (`data/resources.toml`, §7): each resource's maximum, start, whether the status bar
    shows its maximum, how it changes over time (it **regenerates** toward its maximum or
    **decays** toward 0, a point every so many seconds, while resting too or not), whether it
    **ends at zero**, what the player is told then, what **builds** it (`kill`, `hit`,
    `critical`, `struck`, `sneak_attack`, `sneak_kill`, each by an amount), and the **status** it brings, if
    any (Rage: Raging), which lasts exactly as long as the resource is held. Ids are in the
    content fingerprint.
  - **State**: `Actor::resources`, the pools it holds in resource order, each its value and its
    progress toward the next point over time (hundredths of a second at the normal pace, like
    `Actor::regen`); saved, covered by the state hash, and left out of the JSON while empty, so an
    actor without any serializes as before.
  - **The only ways a resource changes**: `World::grant_resource` (held from now on at its start,
    or what it holds if that is more), `World::gain_resource(actor, event)` (called where the
    event happens: a kill credited to the holder in `dies`, a blow of theirs that hits and one
    that is a critical, and their being hit, in `resolve`; a sneak attack that hits in
    `ambushed`, and one whose blow kills (`sneak_kill`, after the hit's) in `resolve`; each builds every resource held that builds on it, up to its maximum; one not
    held is not built), `World::spend_resource` (refused, nothing spent, when not held or short;
    a cost of 0 asks only that it is held), `World::end_resource` (the holder loses it and its
    bar; the player is told its `ended`, if it has one), and time. Talents add to them: a
    `resource_build` builds a held resource on an event beyond its own `builds`, a
    `resource_max` raises its maximum (`Actor::resource_max`), a `resource_start` what it holds
    when granted (`Actor::resource_start`, at most the maximum), and a `resource_rate` its pace
    over time in percent (`World::resource_rate`). What the holder wears does too, for the pool
    its affix names (§8 "Magic items"; the affixes design §5): a `resource_max` affix raises
    the maximum (twice over for stamina with **Bellows**, Vigour, Toughness 12) and a
    `resource_rate` affix speeds the pace, beside the talents', and an
    `active_stamina` affix takes stamina off every weapon active (`Actor::weapon_stamina`, at
    least 1), and a `resource_on_kill` affix (of the Hunt) adds its sum to the pool on a kill,
    under of Siphoning's rule ("Mana", "On a kill": the slayer's own blow, shot, or spell, the
    slain within three cells; never past the maximum). An affix never grants a pool: one who does not hold it (a wizard's stamina, any
    creature's) gets nothing of it. Taking off what raised a maximum lowers what is held to the
    new maximum at once (`World::keep_resources_within_max`, after each player action, as with
    mana).
  - **Its status**: a resource with a `status` brings it: granted by
    `World::grant_resource_with_status` (the Rage ability, the dev action), the status begins
    (said as it begins, "You fly into a rage!"); as the resource ends, spent, decayed, or
    otherwise, the status ends at that moment, with its end said ("Your rage subsides.") and what
    it `follows` into beginning; and should the status end some other way (a cure), the resource
    ends with it. Such a status must not ease by itself (refused at load), and the dev `afflict`
    refuses it ("`raging` comes of a resource: try `resource:rage`.").
  - **Time**: a regenerating resource gains a point every `regenerates` seconds up to its
    maximum (its progress dropped there); a decaying one loses a point every `decays` seconds
    down to 0, where one that ends at zero ends and one that does not sits, held, at 0. Each point
    is a moment of `World::live`'s loop (with a hit point and a point of mana back, before the
    ticks), so it makes no difference how time is cut up, and an end comes at its exact moment.
    The pace (`World::resource_rate`, the one place that decides it) is 100 % plus what the
    holder's talents add (`resource_rate`, never below 0), and none while the player rests for a
    resource with `resting = false`; the maximum (`Actor::resource_max(content, resource,
    level)`) is the data's, with what its `max_from` adds (the holder's modifier in that stat,
    of the scores as the rules read them, and a point per so many character levels, at least
    `least`), plus the talents' `resource_max`. Both are hooks the resource talents read (R5). Actors on other levels are
    frozen, and the dev time skip passes none.
  - **Ending at zero**: spent or decayed to 0, a resource that ends at zero is gone, and its bar
    with it, until something grants it again. A rest goes on through it (the end is only
    logged, as a status effect's end is).
  - **Abilities**: a class active may cost a resource in data (`resource = { id, cost }` on its
    talent, §7). While the player does not hold it or holds less, the ability is unavailable
    (`Unavailable::Short`, "not enough focus" in the abilities menu, `short:focus` in dumps; using
    it says "You have too little focus.", no time); used, the cost is spent first (Volley's,
    aimed at a cell, once its own checks pass; an aimed blow with nobody hostile that way is
    refused before it, spending nothing).
  - **Granted by talents**: a talent with the `grants_resource` effect (§7) holds its resource
    from the moment it is taken or given, at its start, and for good; a run restored gets any
    such resource its talents grant and it lacks (`world/focus.rs::hold`).
  - **Focus** (the ranger's; R2, #250): Rapid shot (level 12) grants it (10, full); it comes
    back a point a base turn but not while resting, and never ends. A shot loosed with `fire`
    while the ranger holds 2 spends them and takes 10 % less time (the whole price, reloading
    too; with less it is an ordinary shot, `World::rapid_shot`); Volley (level 50) costs 6 and
    has no cooldown.
  - **Openings** (the thief's; R3, #251, `world/openings.rs`): Sneak attack (level 1; #682)
    grants them at 0 (`grants_resource`); each sneak attack that hits builds one (to 5), one
    slips away every 120 s (20 base turns), and at 0 they stay, held. Master thief (level 60)
    spends 5 on its finisher (§8 "Class abilities").
  - **Ki** (the monk's; R4, #252): Flurry (level 18) grants it (5, full); it comes back a point
    every 18 s (3 base turns), resting too, and never ends. Each second blow of a flurry spends a
    point (`flurry_cost`, in `World::flurry`): with none held there is no second blow, the
    attack is one blow. Quivering palm (level 50) costs 5 and has no cooldown; with a weapon in
    hand it is refused before the ki is spent. The monk's mana stays theirs, separate.
  - **Inspiration** (the bard's; R4, #252): Inspire (level 18) grants it at 0; each kill credited
    to the bard builds one (to 5), one fades every 180 s (30 base turns), and at 0 it stays,
    held. Inspire spends 3 inspiration **in place of** its 3 mana while the bard holds 3 (then
    no mana is spent, and nothing blocks it as casting), else its mana as before; Virtuoso's
    song costs neither.
  - **Stamina** (#560, the combat progression design §4.5): the pool weapon actives pay
    from (§8 "Weapon abilities", #564). The granted talent **Trained wind**
    (`grants_resource`) is in the level-1 row of the warrior's, barbarian's, paladin's,
    ranger's, monk's, thief's, and bard's tables, so those hold it from creation, full; a saved
    character of those classes is granted it on load (`World::grant_class`). Its maximum is 10
    + the Toughness modifier + a point per 10 levels, at least 5 (`max_from`), so Toughness 14
    holds 12 at level 1; it comes back a point every 12 s (2 base turns), resting too, and sits
    at 0. A barbarian holds Rage and Stamina at once. Shown as `Sta 7/12`. Talents (the
    `stamina` system): **Deep lungs** (Vigour): +4 at most; **Bellows** (Vigour, Toughness 12):
    what magic items add to the maximum counts twice; **Steady breathing**
    (Weaponcraft, a weapon skill at 5): back at 150 %; **Second breath** (Might): a kill
    credited to the holder builds a point. Anyone else is granted it, full, on the first weapon
    rank that teaches an active (the third grant place, #564: `fight.rs::grant_stamina`, where
    every rank the player gains is said).
  - **Shown**: the player's own resources (`PerceptionView::own_resources`: each held, its value
    and maximum, in resource order): the status bar after `MP` (`Rage 11`, `Focus 7/10`), each
    with the hit points' priority after the mana, so they give way before it (§6); a row each on
    the character screen under the stance; the dump header (`rage 11/20`), JSON dumps'
    `resources`, and the dev `actors` layer (every actor's, with progress). Never a creature's,
    and never the pace. The dev action `resource:ID` grants one (§9; "You gain Rage: 12 (dev).").
  - **Rage** (the barbarian's, R1; `world/rage.rs`): the Rage ability (a wait's time) grants the
    Rage resource (20 at most, starting at 12, a point lost every 6 s, resting too; it ends at
    zero) and with it Raging; the action's own wait costs the first point. No rage while Raging
    or Exhausted lasts. Blood frenzy (level 40): each kill credited to the barbarian builds 5
    rage (a `resource_build`). Mighty rage (50): Raging's raises doubled, 10 more rage at most,
    Exhausted eases twice as fast. Unstoppable (60): while raging, bleeding and poison tick
    nothing, and a blow (`Harm::Actor`) that would take the last hit point leaves it for 10 rage
    (`World::spend_resource`; refused, so the blow falls, with less), as often as the rage
    holds; spent to nothing, the rage ends and Exhausted follows. While raging no mana is spent
    (Raging blocks `cast`).
  - **Talents** (R5, #253): the `resources` system is in the coverage list, held to its bar by
    the four resource talents, the only picks limited to a class (a resource belongs to the class
    that has it): **Battle trance** (Might; barbarian, level 6): rage starts 6 higher (18) and
    decays at 67 % of its pace; **Inner calm** (Finesse; monk, level 18): ki holds 2 more (7) and
    comes back at 150 %; **Steady hand** (Weaponcraft; ranger, level 12): focus comes back at
    150 % (still not while resting); **Patient blade** (Shadow; thief, level 32): openings decay
    at 50 %, and a sneak attack that kills builds one more (two in all). Rage, Blood frenzy,
    Mighty rage, Unstoppable, Rapid shot, Volley, Improved sneak attack, and Master thief carry
    it too. **Saves change** (a new id group in the fingerprint).
- **Checks** (`warlock-core/src/check.rs`; M24, #139): a stat put to the test, rolled like an
  attack: 2d12 (the natural roll, `combat::roll_natural`, two draws of the gameplay stream) plus
  the modifier of the stat tested (the effective score's) succeeds at the **difficulty** or
  more; a natural 2 always fails and a natural 24 always succeeds (`check::check`,
  `check::ways` for the odds: against 13 at +0, 78 of 144). Diseases are resisted and fought off
  with them (Toughness, brain fever Willpower). A **contest** puts one side's check against the
  other's: the other side's natural roll plus its modifier is the difficulty, four draws, the
  other side's roll first; a tie goes to the side making the check (`check::contest`,
  `check::contest_ways` for the odds, of 144 × 144: even sides win a little over half). Stealing
  is one (§8 "Stealing": Dexterity and Steal against Willpower). The player never sees a
  difficulty, a modifier, or a roll; they see what comes of it. Nothing limits retrying: the
  time a disease takes is its cost.
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
    The player's race's `disease_check` adds to this check and to every progression's
    (`World::disease_modifier`; ratfolk 4, #255), and so do the talents' `disease_checks` and
    the worn `disease_check` affix effect (§8 "Magic items"), a creature's gear too.
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
    lists the means in its `cured_by` (`remove_disease`). A **remedy** applies it: food whose
    `cures` names means (§7: objects), cured as it is eaten (`e`, §8 "Hunger"), what ended said at
    once ("The rot is gone from your flesh."; an incubating disease ends unseen, as it came).
    The built-in one is the **angelica root** (`%`, worth 30 gold; nutrition 20, one turn), which
    cures every disease, grave rot too, so a caught grave rot is no certain death: Grimbold's
    General Store in the courtyard has one (the purse in the gazebo pays for it), and general
    stores and provisioners stock it among their food. Potions and priests will apply cures too.
  - **Dev**: the action `infect:ID` makes the player catch a disease as if they had lost the
    check (it incubates; no game time; recorded in replays; "You catch filth fever (dev)."), for
    tests and captures. `afflict:diseased` is refused: Diseased comes of a disease; so is a
    status a resource brings (`afflict:raging`: Raging comes of `resource:rage`).
  - Infections and stat damage are part of the world state (saved, covered by the state hash).

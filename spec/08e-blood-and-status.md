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
  - **By damage type** (`blood` in `data/damage_types.toml`, default true): only a blow of a
    kind that draws blood spills any. Fire sears, cold and lightning numb and burn, poison and
    the withering of life (necrotic) leave no wound to bleed: those draw none. The physical
    kinds, acid, and force do. A blow of two kinds (a sword's slashing or piercing) draws blood
    if the kind that struck does; an untyped blow (no damage types) always may.
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
- **Char** (`warlock-core/src/scorch.rs`, the pattern; `world/sear.rs`, applying it; #1255):
  fire leaves scorch marks on the floor it burns and on the walls facing it, building up the more
  a cell burns. Blood's sibling, by its rules: presentation state no rule reads.
  - **Who chars**: a damage type with a `char` in `data/damage_types.toml` (`{ light, heavy }`
    colours): fire.
  - **Where and how much**: every open cell an area of it reaches takes a share, jittered 60 to
    140 %: a burst's 20 (half at its edge, beyond radius − 1), a cone's 20, a fire zone's 24 as
    it is laid (it burns there for turns). The walls beside those cells take 75 % of the cell's
    share on the face toward it (half again across a corner), the most any neighbour gives. A
    creature struck by a charring kind chars its own cell: 3, and a third of the blood the blow
    would have spilled had it bled (a big hit's or a kill's). Bolts char only the cells of those
    they strike. Terrain without `stains` takes none; nothing on the overworld.
  - **Accumulation**: amounts add up per cell to 100 (charred through), and the colour follows
    the amount from the type's `light` colour (brown soot) to its `heavy` one (near black):
    light soot, then blackened, then charred. One fireball scorches; a room fought through with
    fire is black.
  - **Deterministic, gameplay-neutral**: `LevelState::scorch` (sparse by cell, blood's `Stain`:
    amount, colour, when, faces), saved and covered by the state hash; how much lands where is a
    hash of the turn, the moment, and the cell the fire centres on, never the gameplay random
    stream: a fight goes exactly the same with or without char.
  - **What the player perceives** (`PerceptionView::scorch`): by blood's rules exactly (an open
    cell in sight, a wall's face seen; remembered as last seen, `LevelState::scorch_memory`).
    Look mode names it before the blood and the terrain: scorched, blackened, or charred by
    amount ("A charred blood-stained stone floor."). The `scorch` dump layer shows it as a digit,
    `1` a little soot to `9` charred through. Drawn subtly (§4.8 "Blood").
  - **With blood**: both are kept on a cell, neither erases the other; blood is drawn over the
    char.
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
    - **strike back or burn** (the spell types design §7, Fire cloak and Brand): `retaliates = {
      damage, damage_type }` does those dice to whoever lands a melee blow on its sufferer, as a
      spell of that type would (the type's rule on the striker, a kill the sufferer's; beside
      Barbed hide's barbs), and `blows = { damage, damage_type }` adds a rider to its sufferer's
      blows with either hand, as a worn `elemental` affix does, which reads no type rule. A kick
      or a shield bash is not such a blow;
    - **bear up on water** (Water walking): `wades_free` prices wading as a step on dry land,
      Undertow's effect, read with the talent's through `Actor::wades_free`; `ice_holds` (the
      owner's rule) means ice never gives way under the bearer, however thin, with no draw, and a
      burst that melts or shatters ice leaves the cell they stand on (`Actor::ice_holds`, in
      `World::tread` and `World::shock`): no plunge, no harm, no wet;
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
      (50, the skirmisher's Hit and run, a weapon ability: one action), and Savage on a companion
      (67, Sic 'em's, §8 "Allies"; an ally never disturbs a rest), so the quickest step of a
      creature that disturbs a rest stays what a rest's batching counts on;
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
      its turns or a struggle. A hold that ticks costs, a tick, the holder's `on_hit` entry's
      `damage` while it holds (1 to 20; the rug of smothering's engulfing, 1), else the status's
      own `damage`. Drowning grip's Held is the same status. A trained **Escape
      artist** (no action, no ability row, no key: the check is part of the attempt, #1015) takes
      the better of Strength and Dexterity, each with what talents add to its checks, plus the
      skill's `escape` (+8 at 100, +12 at 200, at most 20) in this check, and the same, without
      Knowledge, against other **bonds**: a status with `escape = DIFFICULTY` (Webbed, Pinned: 13;
      Staggered, Stunned, Surprised, Transfixed, Sluggish, and Drowsy are no bonds). The first
      status blocking the action's kind asks: made, it ends with its own `end` line and the
      action goes ahead at its price; failed, the action's price, "You struggle, but you are still
      caught in a web." (its `seen`). A second status blocking the same action asks on the next
      attempt, and `refuse_blocked` refuses this one. Untrained, Webbed and Pinned refuse the
      action, no time, as any blocking status does. A marked status that only slows (none in the
      data today) takes the same check as the action is tried: made, it ends and the action costs
      its plain price; failed, the action goes ahead slowed, at no extra cost. Two draws of the
      gameplay stream a check; each marks `escape` (100 %) and `struggled`;
    - **be followed** (`follows = "exhausted"`): as it ends, worn off or cured, the status it names
      begins at one degree, caused by whoever caused the one that ended (Raging, then Exhausted:
      the ending's message, then the beginning's). A chain never leads back to where it began, a
      status with `follows` must end (`eases` or `cured_by`), and nothing follows into or out of
      Diseased.
    A **cooldown** is a status with `eases` and no tick given as an ability is used: while it
    lasts, the ability is `Unavailable::While` it ("you are winded"), a visible limit that only
    time lifts; nothing counts uses. An ability that buffs the player calls `World::afflict(player,
    status, 1, player)`. The class abilities that give them come with the class tables; the
    built-in data has the berserker's rage as the example: Raging (Strength and Toughness +2,
    Intelligence −2, no reading or casting, while the Rage resource lasts, "Resources" below),
    then Exhausted (10 minutes: Strength −1). A status that `blocks` casting also refuses every
    spending of mana (`World::spend_mana`) and greys out an ability that costs mana.
  - **Deaths**: a tick that takes the last hit point kills by the status, at that moment
    (`Event::Died`'s `of`; the player's death records it, `Death::of`). A creature that dies of
    a status counts as killed by whoever caused it (`Effect::by`): experience (#133) credits the
    player for creatures bleeding to death from wounds they dealt.
    A status that `kills_at_max` kills as it reaches its max instead (`World::afflict`'s degree
    step, one hook: the status goes, then the bearer dies of it by whoever gave the last degree,
    as a tick's death is). **Stiffening** (#1126, a basilisk's gaze; a cockatrice's peck and a gorgon's breath, #1138) is one: four degrees
    adding up, two minutes a degree to ease, half again as slow and Dexterity −4 from the second;
    its `stages` tell each degree lived to ("Your limbs stiffen.", "Your skin greys and
    hardens.", "You can barely move."), and at the fourth the bearer is stone: "Turned to stone"
    on the death screen, "The kobold turns to stone." for a creature, which leaves a statue
    (`statue`, §8 "Statues"). The scroll of restoration (`restoration`) and a temple's cure
    (`prayer`) end it, the Heal skill eases it a degree a try, Still mind's Stillness ends it.
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
    nothing resists them. A bloodless creature (`blood = "none"`: the bone and spirit undead,
    the elementals) never bleeds: a status with a `wound` table takes only on a creature with
    blood (`World::afflict_one`, the spell types design Q7), whatever lays it (a blow, Flay).
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
    timer. **Every buff shows**, whatever its source (a spell, a potion, an altar, a talent, a
    class ability, a style): a status that lends something (a stat, a combat bonus, a quicker
    pace, eyes, a resistance, light) has a `seen` word ("quickened", "bark-skinned",
    "fortified", "cat-eyed", "warded"), as every hex has; a mimic's disguise and the thief's
    lurking are no buffs and never show. A buff's or hex's status carries a spell circle
    (§4.8), which a check at load keeps to statuses that show. Effects are part of the world
    state (saved, covered by the state hash).
  - **The circle's light**: a circled status (`data/circles.toml`) without a `light` of its own sheds
    its school's, `circle_<school>` (radius 1, steady, a strong UV share: just over normal eyes'
    threshold on the bearer's cell, under it a cell away, bright to UV eyes; Lit keeps the Light
    spell's brighter one). One light per actor, the widest status light, the first in status
    order on a tie; never scaled by severity or time left; it ends with the status. Creature
    sight reads it as the player's does (a buffed sneak lights their own cell, a hexed foe is
    marked in the dark), and it adds to a light-blind player's glare. A disguised mimic sheds
    no status light.
  - **Dev**: the action `afflict:ID` gives the player one more degree of a status (no game
    time; recorded in replays), for trying statuses out and for captures.
- **Healing** (`world/status.rs`): a hurt actor's progress toward its next hit point back
  (`Actor::regen`, in hundredths of a second at the normal pace, saved) grows each second by its
  healing rate in percent: 100 normally, the slowest `regen` of its status effects (bleeding 0,
  poison 50, Diseased 50) and of the diseases that show (marsh ague 25), or, when none of them
  slows it, the fastest (Renewal's `renewing`, 400), times `[healing]
  resting` (200) while the player rests (creatures never rest), times the `regen` of a special
  room it stands in (a sanctuary 300, §7 "Special rooms"); every `regen_seconds(Toughness)` ×
  100 of it is a hit point ("Combat" above; the effective Toughness). Progress carries over and
  is dropped at full health. A Starving player gets none back (§8 "Hunger"). The regeneration
  affixes of what the actor wears and wields multiply the rate by (100 + their percent) / 100 (§8
  "Magic items"). `World::healing_rate` is the one place that decides the pace.
- **Mana** (`warlock-core/src/mana.rs`, the pure rules; `world/mana.rs`, applying them; #223):
  a second pool beside hit points, built the same way. Spells spend it ("Spells" below: the
  casters' class spells and those learned from scrolls), as do rods (§8 "Rods") and a summoner's summons (§8 "Creatures"); Blood magic
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
    Old delver (§8 "Talents") spares the one who has it.
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
  - **Talents** (#242, the Scholar tree, §8 "Talents"): Deep well (`max_mana_per_levels`:
    1 mana per 4 character levels, at least 2; handed out into `mana_base` and
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
    never the pace (hidden like `regen_seconds`); the dev `actors` layer shows every actor's.
  - **Creatures** (#525): a creature's maximum and pace read what it wears and
    wields exactly as the player's do (`max_mana` and `mana_regeneration` affixes through
    `Mods::worn`); it is made with all of its larger pool, and a summoner pays
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
  and the lift, and learning by Spellcraft (Decipher script since #1270), `docs/design/no-caster-level.md`; `content/spell_def.rs`,
  `world/cast.rs`, `world/learn.rs`; #394, #395, #396, #397, #759): the
  spell data, the one cast path, learning, and the first spell list. A spell's growth is
  written in its own file as ladders on the caster's stats and printed in the wiki as written;
  talents and items act on it through the hooks. The casters' spells (§8 "Class
  abilities") are spells of `data/spells/` cast by it, and so are the spells learned from
  scrolls and the spells scrolls cast. `docs/dev/spells.md` has the details.
  - **Spells aimed at a corpse** (#999; the spell types design §7 entries 2, 29, 30): a spell with
    `[spell.effect] corpse` asks for a cell where the caster sees a corpse, and uses the pile's
    top-most corpse stack; `consumed` takes one. The corpse's stage is not read. Refused, no time
    and no mana, with none. **Charnel burst** goes off where it lies; **Grave feast**, a touch,
    heals and gives mana (`mana` on a heal), each held to its maximum; **Rouse the dead** raises
    its own creature type (§8 "Allies": "The risen").
  - **Beguile** (#1003; the spell types design §7 entry 34): a level-18 enchantment hex, a ray of
    range 6, with `[spell.effect] charm` in place of a status: the hostile creature it falls on
    is the caster's ally for its degrees in minutes, then itself again (§8 "Allies": "The
    charmed"). The witch's at level 44, and a scroll.
  - **The first list** (#397, design §7; names left for the world-language review, #378):
    by spell level (the spell scaling design §5.8): 0 Evil eye, Light (#589: the scroll of
    light's; Lit, its status, sheds a light about the caster, `light` on a status); 1 Burning
    hands, Grave breath, Mend, Ray of frost, Spark; 3 Barkskin, Blessing, Cure wounds, Curse of sloth, Hex of weakness, Magic
    missile; 4 Cause wounds, Nightsight; 5 Acid arrow, Call of the wild, Mesmerise, Water
    walking; 6 Arcane bolt, Frost, Identify, Sleep; 7 Blight, Brand, Scorching ray, Web; 8 Flay,
    Glamour, Unbinding; 9 Bind, Rime; 10 Fear, Renewal; 11 Call lightning, Fireball (the
    wizard's Fire burst folded into it), Haste, Lightning bolt, Remove disease, Silence; 12 Fire
    cloak, Flame; 14 Grave spear; 15 Slumber; 20 Call of the forest, Chain lightning; 30
    Sunfire; 35 Call the elements (the druid's calls, #839: summons, §8b "Allies"). #1001 adds ten of one
    rule each (the spell types design §7): 5 Sense the quick (Quick-sensed: the living within 8
    the player does not see, but the undead, constructs, and animated things, heard each turn as
    heartbeats at rough places); 6 Blink (to a free cell within 5, drawn; it works while held,
    webbed, or pinned and ends all three); 7 Thunderclap (a force burst about the caster,
    `range = 0`, no aim: everyone in it Stunned 1, the caster too); 8 Bramble (Webbed on those
    standing where things grow, terrain `plants`), Shatter (a force burst with a blast); 10 Leech
    (a necrotic ray: half the hit points it takes come back to the caster); 11 Gale (a line that
    drives everyone on it a cell on, nearest first, and puts its fires out; no harm); 12 Benison
    (one heal roll for the caster and their allies in a burst about them, two casts' time); 14
    Rebuke (fire on the undead in a burst alone, friend or foe, and Frightened 3); 20 Cloudburst
    (an hour of rain under the open sky, putting out what burns there).
    **Healing harms the undead** (#1278): a healing spell (a `heal` that feeds on no corpse:
    Mend, Cure wounds, Benison; never Grave feast) that lands on a creature tagged `undead` does
    its roll (its rung, rank, and hooks) as fire harm instead ("The light of mending sears the
    skeleton."), fire's resistances and rule standing (no holy type, as Rebuke). A touch on an
    undead beside the caster is an attack (the touch's roll against its Defence, a critical
    rolling again); Benison sears every undead in its burst, friend or foe, and is cast with only
    undead there. The living are healed as before, friend or foe. Scrolls, wands, and miscasts
    follow the cast path's rules. A creature healer heals no undead, and an undead caster casts no
    heal (§8b "Creature spells"). No player race is undead.
    The potions wave 2 (#1227, its design §9.3) adds 8 Pitch fire (a fire burst of radius 1 that
    leaves a pool of fire, a `fire` zone of 6 turns, on its cells: Balefire's zone in a burst;
    from its scroll and the pitch fire flask only). No spell protects its
    caster or their allies from it (the owner's Q14). A spell is cast by its
    data id (`use:magic_missile:12,5`, any `use:ID` that is no class or birth sign ability; one
    that is no spell either is a weapon active's, §8i "Weapon abilities");
    the abilities menu lists the spells known by spell level, then path order. Sleep and Web are
    **area hexes**: a burst whose status falls on everyone it reaches but the caster, friend or
    foe (`World::cast_area_hex`); so are Fear and Silence. Flay's Bleeding grows by a degrees
    ladder, as a hex's status does (a damage spell's status may have one). Haste is the one status that hastens (`cost` 75; statuses may
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
    comes at, whatever the character's Decipher script: a spell a class gives is its holder's at that
    level, and the level is the author's call. Wizard: Ray of
    frost (1), Burning hands (1), Magic missile (3), Arcane bolt (18), Chain lightning (24),
    Sunfire (36). Priest: Mend (1), Blessing (3), Cure wounds (3), Renewal (9), Silence (12)
    (their rite is the prayer, §8i). Druid: Spark (1), Barkskin (3), Call of the wild (6), Water
    walking (6), Call lightning (12), Rime (15), Call of the forest (25), Call the elements
    (40), Flame and frost (50). Witch: Evil eye (1), Grave breath (1), Hex of weakness (3), Flay
    (8), Cause wounds (12), Curse of sloth (32), Fear (36), Blight (40). Bard: Mend (1),
    Mesmerise (8), Bind (16). Paladin: Blessing (6), Brand (14), Unbinding (20). Ranger:
    Barkskin (6), Nightsight (9), Water walking (12). Every other class none. A class row is a
    way to meet a spell, not part of it (the spell types design Q13): Acid arrow, Grave spear,
    Slumber, and Glamour come only from scrolls. Every class has some mana (the
    berserker and the warrior 1); a race adds none unless its data says so.
  - **Spell scrolls** (§8 "Scrolls"): every spell comes on one scroll (`scroll_of_ID`, a
    `read` table of kind `spell`), which casts it through the cast path at the scroll's own
    `rungs` of the spell's damage or heal ladder (default 0, the base; at most the ladder's
    length) plus the reader's `scroll_rungs` (Scroll adept), the reader's other hooks applying
    and their reach and degrees, free of mana and never refused for what the reader knows
    (Scroll savant may keep it; a death by it is the scroll's); it teaches nothing (its grimoire
    does, below). The scrolls of fireball (`rungs = 1`, 3d6), lightning (`rungs = 2`, 4d6), and
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
    least the cost table's **floor**, `[mana] floor` = 4 (#1264): every spell of level 0 to 4
    is priced as a level-4 spell, so a starting caster's pool buys four to eight casts of a
    cantrip rather than twenty) and the **learning at which a grimoire of it teaches for
    certain** ("Learning" below). A spell of
    level 0 or 1 is a *cantrip* (4 base mana, times its shape: Ray of frost 4, Burning hands and
    Grave breath 6, Mend 3; a spell's own `mana` still stands in for its level's, Identify's;
    anyone trained in Decipher script learns it for certain). There is no
    table of levels, no ceiling, and no check on how much a spell of a level may do: its own
    data says it (decision 16).
  - **Ranks** (#1264; `content/spell_rank.rs`, `world/spell_rank.rs`): every spell a character
    knows has a **rank**, 1 to 10. A spell is known at rank 1 as it is learned, or as a class
    gives it. **Reading its grimoire again deepens it** to the next rank ("Learning" below:
    "Deepening"); casting trains no rank, and nothing else raises it. What a rank gives is the
    spell's own `[spell.ranks]` table (`data/spells/README.toml`): one entry a rank from 2 to
    10, each what reaching that rank **adds**, cumulative (a cast at rank N has everything the
    entries for ranks 2 to N add); a spell with no table gains nothing from its ranks. An entry
    may add, through the hooks: faces rolled again (`reroll`), bigger dice (`sides`), dice
    (`dice`), a bonus on the roll (`bonus`), cells of range or length (`cells`), a wider
    burst (`radius`), degrees or minutes (`degrees`), a percent off the mana (`mana_off`) or
    the casting time (`time_off`, never more than nine tenths off), and, on a spell that rolls
    to hit, `to_hit` and a wider threat range (`threat`); each only where the spell could read
    it (checked at load), and nothing bounds how much (decision 16). **Only a cast of one's
    own known spell reads its rank**: a scroll read or a wand used keeps its own rung rules at
    rank 1, and creatures' spells are at rank 1. Shown: the Magic page's spell list (each
    spell's rank), the `spells` dump and JSON (`rank`), the read list and its dialog ("rank 3
    of 10"), and a grimoire's description. The wiki's `spell-ranks.md` lists every spell's
    table. Two worked tables ship (Magic missile, Ray of frost); the rest are #1265 to #1269.
  - **Cost table** (`data/spells.toml`): the **shapes** multiply the base mana (ray, bolt,
    touch, self ×1; cone ×1.6; line ×1.8; a burst ×1.8, ×2.4, ×3 by radius 1 to 3, its
    largest), and each **resolution** multiplies it (`attack`, `auto` ×1, `area` at its shape's
    cost, `touch` ×0.7). First guesses, in data.
  - **Spells** (`data/spells/`, one `[[spell]]` file each, numbered in path order: `SpellId` an
    index, in the content fingerprint): id, name, description, level, the **tags** school
    (evocation, necromancy, abjuration, restoration, transmutation, enchantment, divination),
    kind (damage, heal, buff, hex, cleanse, utility, summon), shape, and damage type (a `damage`
    spell's); a resolution that fits the shape (`area` exactly for cone, line, and burst, `touch`
    only for touch, `attack` for a ray or a bolt, `auto` for any single target); its **base**
    reach, each shape exactly its own (`range` for a ray, bolt, or burst, `radius` for a burst,
    `length` for a cone or a line; casting holds a range or a line to 20, a cone to 5, a radius
    to the burst table's length);
    `damage`, a damage spell's base roll (a `Roll`, `"2d6"`, `"3d6+1d10"`); `[spell.effect]`
    for the other kinds (a heal's base roll `heal = "2d6"`, a buff's or hex's `status` and base
    `severity`, a cleanse's `cures` and `diseases`, a damage spell's `status` suffered by
    everyone it harms); `aim` (an attack or touch damage spell's: the stat whose modifier aims
    it, default Dexterity); `casts` (its time in `cast` actions, at least 1); `open_sky`; `mana`, a
    cost given outright in place of its level's (Identify); `[spell.effect] identify` (a
    `utility` spell on `self` that asks for a stack); `[spell.effect] summon` (a `summon`
    spell's call on `self` with `auto`: `creatures`, `count`, `most`, `minutes`, `spread`; spec
    §8 "Allies"); `study_difficulty`; `[spell.cast]` (the `effect` drawn,
    `noun`, `verb`, `message`, `noise`, and a burst's `blast`, `heat`, `frost`); and
    `[spell.grows]`, its ladders (below). The loader checks only that a spell is well formed
    (#1205): ids resolve, and no field is given where its kind or shape would never read it.
  - **Ladders** (the spell scaling design §4, §6; `content/spell_ladder.rs`): `[spell.grows]`
    gives each dimension the spell has an optional ladder: `damage` and `heal` (rolls),
    `range`, `length`, `radius` (cells), `degrees` (of a buff's or a hex's status). A ladder is
    one string, the stat's id then rungs `AT VALUE` separated by commas (`"intelligence 13 3d6,
    16 4d6, 19 5d6, 25 5d6+2"`), `AT` a score 1 to 40 strictly ascending. The loader refuses a
    ladder for a dimension the spell lacks and an unknown stat. **No check reads how much a
    rung does** (decision 16); casting holds reach to the engine's limits (a range to 20, a
    cone to 5, a line to 20, a radius to the burst table's length). The
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
    mastery (a wand rung), Sure hand (rerolls 1s), Brimming (a die more, +10 % mana) (the rider
    has no shipped talent: a type's own rider is below); the affixes `spell_rungs`, `spell_bonus`
    (Potent: +1 / +2 / +3), `spell_sides`, `spell_dice`, `spell_reroll`, `wand_rungs` (of
    Wands), `spell_reach` (of Farcasting, cells), `spell_duration` (of Lingering, degrees).
    Derived at cast time; nothing saved.
  - **Zones** (`docs/design/spell-types-and-new-spells.md` §7 "Zones"; `[spell.effect] zone`, five
    scroll-only spells, `world/zone.rs`): an area that lies on the floor for some of the player's
    turns, on the cells of a burst or a line, never spreading, a new one replacing an old on shared
    cells, kept when the player leaves the level, none on the overworld. It takes the caster and
    their allies like anyone; its kills are the caster's. `fire` (Balefire; Pitch fire's pool,
    #1227): a creature that starts
    a turn on it or steps onto it takes its harm, rolled once as it is laid, and the fire type's
    rule; each cell sheds a fire's light; a cold spell's `douses` clears it cell by cell. `fouled`
    (Reek): a creature that starts a turn in it takes the poison type's rule; it does not block
    sight. `slick` (Slick): a step onto it or off it is a Dexterity check against 12, failed
    Staggered 1, the step still taken. `dark` (Mirk): no light reaches its cells and a source on them
    sheds none, so eyes that need light see nothing there. `smoke` (Fume, the flask of smoke; the
    potions wave 2 design §9.2): no eye sees into it or through it, the player's and a
    creature's alike: a smoke cell stops a line of sight as a wall does, but what stands on it is
    seen only from beside it, and a viewer inside sees only the cells beside them; light,
    missiles, and spells pass; Gale clears it, rain does not. The player sees a zone as a still
    tint and in look mode's words, where they see or remember the cell; smoke also from outside
    it, on a known cell, where their sight stops at it.
  - **Ruin** (#1100; `data/ruin.toml`, `world/ruin.rs`): an area spell (a burst, a cone, or a
    line) with a damage type, cast by an actor (from mana, a scroll, or a wand; a dragon's breath
    too), may destroy the items lying on the floor cells it reaches: each item of each stack there
    is lost at the chance of the first row of the spell's type matching the stack's item class,
    the material of its main part, or that material's group (fire burns scrolls, paper, wax,
    cloth, food, and wood, and may boil a potion; cold shatters potions and glass; lightning
    burns scrolls; acid eats most things but glass, stone, gems, and coins; force smashes potions
    and glass; necrotic rots food). A stack may lose some or all. Never touched: what is carried,
    worn, or in a chest, and what is protected from all item damage (spec §7 "Materials": a unique
    item, an item of Proof, and an object a quest asks to be brought or delivered).
    Traps' and places' bursts ruin nothing. The draws are the run's own `ruin` stream. Seen
    cells say what went, a line per kind of item ("The scroll burns up.", "3 potions freeze and
    shatter."); unseen, nothing is said.
  - **Spell types** (`docs/design/spell-types-and-new-spells.md` §3, §4; `[types]` in
    `data/spells.toml`, read into `SpellRules::types`, every status and light checked at load):
    what every spell of a damage type does beyond its damage, always on, cast from mana, a
    scroll, or a wand; a type without an entry does nothing more. Magic's alone: a weapon's blow
    never reads it. It spares no one, the caster and their allies included. A `rider` (status,
    severity, `chance` in percent, default certain) on every creature the spell harmed (took hit
    points from: not a miss, nor one that resisted it all), then `ends`, the statuses it ends on
    them. `heat`, `frost` (mm), `douses`: on a burst's cells, a single target's cell struck (the
    cell aimed at when it finds no one), or a line's last cell; a cone's cells no. `fire`: a
    burst's centre cell, if walkable, gets a fire of that light for that long (light only; fixed
    cells, it never spreads). A spell's own `heat` or `frost` stands in for its type's. A trap's,
    a place's, and a birth sign's bursts and bolts are no spells and read nothing. **Fire**:
    Burning 1, one time in four per creature harmed; melts ice; `arrow_fire` for 60 s. **Cold**:
    Sluggish 1, certain; ends Burning; `frost = 100`; douses. **Burning** (`data/status.toml`):
    max 3, adding; 1 hit point of fire per 10 s at one degree, a degree easing per 30 s; sheds the
    `ember` light; a step or a fall into water ends it (`quenched`). **Force**: `shove`, after
    the hurt, on one who lives: a blow of at least ⌈max hp / 3⌉ taken or a confirmed critical knocks
    it a cell away from the cast's origin (a burst's centre, else the caster; the one at a burst's
    centre from the caster, and the caster there stays), if the cell is free; a status that blocks
    a step holds it. `shatter`: a burst's `blast` shatters ice thinner than 100 µm a point
    (`Shock::Blast`). `doors`: a burst's `blast` breaks the closed doors in its line of sight, as it
    stood before the blast, with a `breaks_to`, no check, lock and trap gone, sprung at no one; a
    chest is any blast's already. **Poison**: Poisoned 1, certain (Blight's own line is 1). **Necrotic**:
    `bite`, before resistance: 150 % of the harm, rounded down, against a target below 50 % of its
    maximum hit points; `immune_tags = ["undead"]`: laid on those creature types' `resist` at load,
    so an immunity to the type from whatever harm. **Acid**: Corroded 1, certain; `corrodes`
    (the potions wave 2 design §9.3): when it harms the player, the first piece they wear (the
    body armour first) takes acid's item damage after the rider, the material's chance to
    withstand it its only draw (`world/corrode.rs`); a creature's gear is never eaten.
    **Corroded** (`data/status.toml`): max 3, adding; 1 hit point of acid per 6 s, a degree easing
    per 12 s: two points a degree. **Electricity** spreads from
    every creature it harmed, at the cast's one roll, with no roll to hit and no Evasion, and
    spares no one, the caster included. `conducts`: everyone standing in that creature's body of
    water (open water connected over 8 neighbours, on terrain with a `freeze` table; ice is not
    water) not yet struck, in actor id order; a burst and a cone conduct too. Then `arc = { chance
    = 25 }`, a ray's, a bolt's, a touch's, or a line's only: the nearest creature beside it not
    yet struck, ties by actor id, one draw when there is one to jump to. A spell's `[spell.effect]
    arcs` (its `count` ladder grows them) takes that many jumps with no draw, and its
    jumps reach 2 cells, never through what stops sight (Chain lightning, 3 to 6). **Conductor**
    (Spellcraft, level 12): the player's electricity spells arc every time, and they resist
    electricity 25 %; the arcs still jump back and the water still conducts. A spell turned back
    or a mishap spreads nothing. The menu, the `spells` dump,
    and a wand's description say the rule after the roll (`4d6 fire, burns one in four`, the
    type's `words`). The aiming mode previews nothing. Determinism: one `below(100)` per living
    creature harmed, only for a `chance` below 100, from the gameplay stream, in the order they
    were struck; then the spread's draws, in the order it strikes.
  - **No cap** (decision 16): nothing checks a spell's strength when the content loads, and
    nothing holds a roll when it is cast: the rungs, the hooks, and a critical add what they
    add.
  - **Casting** (`World::cast_spell`, `world/cast.rs`; `Action::Cast`, `use:arcane_bolt:12,5`):
    - **Cost** (the one hook, `Actor::spell_cost`): `SpellRules::mana`: the spell's level, at
      least 1 (a known spell is cast at its level's mana, and the pool is the gate) × the shape's percent (a
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
      `max` holds it (Blessed, Hexed, and Weakened go to 3, Sluggish to 6 for its spell's ranks, each degree its `eases`
      again; `Actor::spell_degrees`).
    - **Dice**: the roll at the rung reached (a scroll's own rungs, or the rung a wand drew as
      it was made, §8 "Magic items": "Wands", plus the reader's scroll or wand rungs in place of
      the stats') with the dice hook applied
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
      all); a spell without a roll, and a rider, draw nothing; then the miscast's Spellcraft
      check (below). Nothing new is saved.
    - **Miscasts** (#1271, #1288; `world/miscast.rs`, `content/miscast.rs`; `data/miscast.toml`,
      `data/miscasts/`; `docs/design/miscasts-review.md`): every spell cast with the caster's own mana, the player's and a
      creature's alike (creatures carry Spellcraft values), makes a **Spellcraft check** once
      it has gone off in full; a scroll, a wand, a flask, and a miscast's echo make none.
      2d12 + the modifier of the rules' `stat` (Intelligence) + the talents' checks of it +
      Spellcraft's `cast` benefit against `base` (8) + `per_level` (1) × the spell's level + a
      point per `armour_percent` of the armour's spell cost (after Battle caster) − `rank_ease`
      (1) × the caster's rank in the spell above the first (a creature's spells are rank 1). A
      natural 2 (1 and 1) always fails, so a cast always may miscast. A failure never cancels or
      weakens the spell: it rolls on **one ordered table** of 65 `[[miscast]]` entries (a file
      each, two totals each, ranges `from`–`to` tiling the totals from 1 up, the last open at
      129): **total** = `die` (2d12) + `per_margin` (5) × the margin the check missed by −
      Spellcraft's `ward` benefit − the stance's `miscast` (Defensive 5, Aggressive −5, Frenzy
      −10), at least 1. The miss carries the total and the die is the small variance; higher
      totals are worse: nothing and boons to 16 (the mana refunded, the spell again for free, a
      blessing, haste), then small harms, mana, hindering statuses, noise, zones, a swap, a
      teleport, stat damage that mends a point a day (#1290), up to 70; from 71 backfires, drains
      for good (`times` over for the worst: each its own random stat and Willpower check), and
      harm climbing from 4d6 to
      20d10 force. **Protected entries**: one whose `to` is below `safe_below` (71) cannot kill
      its caster (its harm takes them to 1 hit point at the least, nothing at 1) and the loader
      refuses one that would backfire, drain, summon, lay a fire zone, or put on a status that
      ticks harm or mana, a campaign's too; only a spell missed by 10 or more reaches past it.
      The entries' effects are the mishaps' kinds (`MishapEffect`, `resolve_effect`) and the
      miscast's own (`harm`, `stat_damage` (`stat`, `points`: §8 "Stats"' stat damage, never a
      score below 1, mending as a disease's does once none shows), `refund`, `twice`, `swap`, `boon`, `noise` (`loudness`: a
      thunderclap's sound on the caster's cell), `zone` (`zone`, `radius`, `turns`: a zone
      around the caster, no harm), `backfire`'s `full`), applied to any caster; "near" is
      within 6 cells in a clear line, in actor-id order, and a creature's swap never takes the
      player. The rows that later phases of the design fill (rank loss, forgetting, stat damage,
      the drain's repeats, the gates, the Pits) stand in for now as existing kinds under their
      final ids. The player is told in words ("Sparks leap from your fingers and bite."); a
      creature's miscast is told by its entry's `seen` line when the player sees it. A death of the player by their own miscast
      is `Cause::Miscast` ("Killed by their own miscast fireball"). Determinism: the check's two
      draws, the die's, then the effect's own, all on the gameplay stream; none without
      `data/miscast.toml`.
  - **Learning, the number** (`Actor::learning`, `world/learn.rs`; derived, never stored;
    `docs/design/no-caster-level.md`, #987): what a grimoire teaches for certain comes from the
    Decipher script skill and nothing else (#1270; Spellcraft's before), its `learn` benefit (`data/skills.toml`: 0 to 10, 10 at 60,
    30 at the soft cap 100, 70, the last spell level, from 150) at the skill's value with what
    skill affixes and the talents' `skill` bonuses add (`Actor::skill_effect`), **at least 1** for a
    character trained in Decipher script. A character **not trained** in it (the skill not in their
    trained set: an item's or a talent's bonus to a skill they do not have trains nobody) has no
    learning and cannot learn from a grimoire at all, cantrips included: refused, no time, "You
    have no training in Decipher script; you cannot make sense of how to learn a spell from a
    grimoire." The
    spells a class grants need no Decipher script, nor does reading a scroll to cast it, though with
    no learning that reaches its level the reader must make it out by a check (§8 "Scrolls":
    "Making out a scroll"). Starting Decipher script by
    class (§7e): wizard, witch, priest, druid `2d6+10`; bard, paladin, ranger `2d4+4`; aristocrat
    35 % `2d4+4`, ascetic 20 % `2d4`, merchant 25 % `2d4` (optional skills); no other class gives
    any; background lines add to it, and a literate caster's corpse may teach it (§8 "Hunger"),
    and it grows by level-up rolls as any skill. **All reading trains it**: a check to make a
    scroll out (and a scroll of a spell one knows, read), 100 %; any scroll read and used up,
    50 %; a grimoire read to its end, 100 %. **Casting trains Spellcraft** (its one mark, `cast`,
    25 %); Spellcraft has no benefit of its own since #1270 and waits for a rule.
  - **Learning** (`world/learn.rs`, `content/learning.rs`, `content/grimoire.rs`; #396,
    grimoires #1260): a character knows a set of spells for good: those their class abilities
    give and those they learned (`Actor::spells`, saved by id, left out of the JSON while
    empty). Either kind is cast from the abilities menu's *Spells* group. Spells are learned
    from **grimoires**, never from scrolls: a scroll only casts its spell.
    - **Grimoires** (`data/grimoires.toml`): every scroll of a spell has a grimoire, generated
      as content loads (`grimoire_of_X` beside `scroll_of_X`, "a grimoire of fireball", the
      scroll's level, `value` (5) times its value, the rest the file's `[object]`: `?`, class
      Scrolls, paper and leather, 800 g, stackable), teaching the scroll's spell. Its title is
      plain words, so it is known on sight and never labelled. It is never loot by itself:
      wherever a spell scroll is made at random (loot, chests, hoards, dens, a vault's loot
      pool, a creature's drop, a special room's loot, a generated shop's stock, a creature's
      gear line, a starting kit, scroll by scroll), it is its grimoire instead with `chance`
      (10) in a hundred, one draw for a spell scroll only. A scroll a level, a shop's `always`,
      a scenario, or a background story names stays a scroll.
    - **Reading a grimoire** (`Action::Read` of one, `read:b`, with no target; the read list,
      `r`): a study. Refused with no time for a spell known at its last rank ("You know all
      there is of Frost."), a character with no mana at all ("You have no gift for magic."),
      no training in Decipher script (above), a target ("A grimoire is read, not aimed:
      `read:a`."), and what a status blocks (a grimoire's read is the `study` kind: Raging
      blocks it). The grimoire is **kept** whatever comes of it: read again after a failure (no
      limit but the time), or kept to deepen the spell ("Deepening" below), or sold.
    - **Time**: `turns_base + turns_per_level × level` (100 and 20: ten minutes and two a spell
      level; Fireball, level 11, 32 minutes) turns of the `study` action (`data/actions.toml`,
      1000), passed like a meal (`World::keep_at`); Quick study takes
      half. It stops as a rest does (a creature that disturbs a rest comes into view, a message
      is said: a blow lands, a status begins); an interrupted reading loses the time ("You stop
      reading."). Once begun, the player cannot stop it themselves.
    - **The check**, as the last turn ends: overreach is the spell's level less the learning. At
      0 or less the spell is learned, no draw, unless the spell is made hard to learn
      (`study_difficulty` above 0: a check at any learning, overreach counted as at least
      0). Above, a 2d12 check with
      the Intelligence modifier + Decipher script's `study` benefit (value ÷ 10, at most 20) + the
      class's `affinity` with the spell's school
      (`data/classes/`, −10 to 10) + Quick study's bonus (2), against `base` (12) +
      `per_overreach` (1) × overreach′ + the sum of the levels of the spells known ÷
      `known_per_point` (20) (`data/study.toml`) + the spell's `study_difficulty` (−20 to 40,
      default 0), overreach′ being the overreach with
      Overreach's third off, rounded down (`study { overreach_percent = -33 }`). Made: "The
      words settle in your mind: you have learned Frost."
    - **Failure**: "The words twist away from you.", then a
      **mishap** (`data/mishaps/`, one file each): one draw by weight among those of the band
      (slight below `bad_from` (6) levels of overreach, bad from it, terrible from
      `terrible_from` (15)) and
      the lucky ones, its message, then its effect: nothing; a status (Sluggish, Weakened,
      Hexed; Blessed for a lucky one); the grimoire's spell going off on the reader (a damage
      spell's base roll as a burst around them; a death by it is the grimoire's); a share of
      the mana lost; a teleport on the level (the teleport trap's); a stat training mark lost;
      a drain (`World::drain`: a Willpower check or points off a stat for good); a hostile
      summoned next to the reader (a dretch); or, lucky, a training mark of Intelligence or the
      spell learned anyway.
    - **Deepening** (#1264, "Ranks" above): the grimoire of a spell known below rank 10 is read
      for its **next rank**: the same reading, as many times as long as the rank sought (rank 2
      twice the learning time, rank 10 ten times), and always a check, whatever the learning
      (the overreach counted as at least 0), with `per_rank` (2, `data/study.toml`) added to the
      difficulty for each rank already held (+2 for rank 2, +18 for rank 10). Made: "The words
      settle deeper: you know Frost at rank 3 of 10.", the rank one higher; no experience.
      Failed: "The words add nothing new.": the time is gone and nothing else (no mishap), and
      the grimoire may be read again at once (no limit but the time, the owner's rule). A spell a
      class gives is deepened like any other. The read list marks such a grimoire "(rank 3 of
      10: a struggle)" and the dialog asks "Deepening Frost to rank 3 of 10 will be a struggle."
    - **Training**: the `study` use (Decipher script's mark, 100 %; an Intelligence mark),
      whatever comes of it. Spending mana on a spell is practice for Spellcraft (its `cast`
      mark, 25 %, as Concentration's); reading a scroll to cast it marks Decipher script (the
      `read` use, 50 %) and Intelligence.
    - **Determinism**: the check's two gameplay draws only when there is a check, the mishap's
      one only on a failure, then its effect's own.
    - **Shown**: the chance in words only (`StudyRules::words`: "certain", "easy" from 96 of 144
      ways, "a struggle" from 36, "far beyond you"): the read list marks each grimoire the
      player could learn from (`PerceptionView::studies`) "(learning: a struggle)" and asks in
      the one dialog before reading it ("Read the grimoire of frost? Learning Frost will be a
      struggle. Once you begin you cannot stop of your own accord.": Read it / Don't read it,
      the safe answer selected first); the dump `spells` layer (known spells with spell level,
      rank, and cost and their roll and reach now, `Fireball (level 11, rank 1, 26 mana): 4d6
      fire, range 11, radius 2`) and JSON's `spells` (each with `level`, `rank`, `cost`,
      `roll`, `damage_type`, and `reach`); the dev `progress` layer's learning.
- **Resources** (`warlock-core/src/resource.rs`, the pools and time's rules;
  `content/resource.rs`, the data; `world/resource.rs`, applying them; #225): alternative
  resources, integer pools beside hit points and mana (which everyone has) that a character holds
  once something grants them. A character may hold any number at once; creatures hold none
  (their rage would never show). The class features of the progression design's §4.18 (the
  berserker's Rage, the ranger's Focus, the thief's Openings, the ascetic's Ki, the bard's
  Inspiration) move onto them in follow-up changes (R1–R5); the berserker's Rage has (R1,
  "Rage" below), and the ranger's Focus (R2), the thief's Openings (R3), and the ascetic's Ki and
  the bard's Inspiration (R4) have (below). Mana is separate: nothing converts between them,
  and nothing is paid in one else the other (a song costs one thing, #870).
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
    the maximum and a
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
    too; with less it is an ordinary shot, `World::rapid_shot`); Sic 'em (level 40) costs 5 and
    Volley (level 50) 6, neither with a cooldown.
  - **Openings** (the thief's; R3, #251, `world/openings.rs`): Sneak attack (level 1; #682)
    grants them at 0 (`grants_resource`); each sneak attack that hits builds one (to 5), one
    slips away every 120 s (20 base turns), and at 0 they stay, held. Master thief (level 60)
    spends 5 on its finisher (§8 "Class abilities").
  - **Ki** (the ascetic's; R4, #252; the ki tricks design, #846): Martial arts (level 1) grants
    it, full. Its maximum is 4, plus the Willpower modifier, plus a point per 6 character levels,
    at least 3 (`max_from = { stat = "willpower", per_levels = 6, least = 3 }`: 5 at level 1 with
    Willpower 12, 10 at level 25 with 14, 17 at level 60 with 16); Inner calm adds 2 and the affix
    of Stillness its own. It comes back a point every 18 s (3 base turns), resting too (Inner
    calm +50 %, Perfect self at 60 +100 %), and never ends; the abilities menu greys what it pays
    for "your ki is spent". Each second blow of a flurry (Flurry, level 1) spends a point
    (`flurry_cost`, in `World::flurry`): with none held there is no second blow, the attack is
    one blow. Step of the wind (level 6) costs 1 and no time, refused before the spend while
    the ascetic is Light-footed already. Stunning strike (level 12, #848) costs 1 and Quivering palm (level 50) 5, neither
    with a cooldown; with a weapon in hand either is refused before the ki is spent. Patient defence (level 18) costs 1 and
    a wait's time (#849). Stillness (Still mind's, level 25, #850) costs 2 and no time, refused before the spend with
    nothing to end. The ascetic's mana stays theirs, separate.
  - **Inspiration** (the bard's; R4, #252): Inspire (level 9) grants it at 0; each kill credited
    to the bard builds one (to 5), one fades every 180 s (30 base turns), and at 0 it stays,
    held. Inspire, a song (§8 "Class abilities"), costs 3 inspiration and nothing else
    (#870: no mana fallback); a Virtuoso's song costs nothing.
  - **Zeal** (the paladin's; P1, #910, `docs/design/paladin-smites.md` §4): Smite (level 1)
    grants it at 5; its maximum is 10 + the Presence modifier + a point per 10 levels, at least
    6 (`max_from`); a blow, shot, or throw of the holder's that hits builds one, and so does
    every blow that hits them (`hit`, `struck`); it comes back a point every 60 s (10 base
    turns), resting too, and sits at 0, never ending. Smite costs 3, Branding smite 3,
    Wrathful smite 4 (#911), Banishing smite 6 (#913), and Lay on hands 6 (Avatar: half, 3), and Retribution (level 25, §8 "Free blows") 4 a blow it answers; a holy gift unanswered outside its bands spends none. A deed that moves
    the player toward evil empties it (§8 "Alignment"). Shown as `Zeal 7/12`. Zealot (the one
    pickable on it, #914, below) makes a blow taken build two.
  - **Stamina** (#560, the combat progression design §4.5): the pool weapon actives pay
    from (§8 "Weapon abilities", #564). The granted talent **Trained wind**
    (`grants_resource`) is in the level-1 row of the warrior's, berserker's, paladin's,
    ranger's, ascetic's, thief's, and bard's tables, so those hold it from creation, full; a saved
    character of those classes is granted it on load (`World::grant_class`). Its maximum is 10
    + the Toughness modifier + a point per 10 levels, at least 5 (`max_from`), so Toughness 14
    holds 12 at level 1; it comes back a point every 12 s (2 base turns), resting too, and sits
    at 0. A berserker holds Rage and Stamina at once. Shown as `Sta 7/12`. Talents (the
    `stamina` system): **Deep lungs** (Vigour): +4 at most; **Steady breathing**
    (Weaponcraft, a weapon skill at 5): back at 150 %. Anyone else is granted it, full, on the first weapon
    rank that teaches an active (the third grant place, #564: `fight.rs::grant_stamina`, where
    every rank the player gains is said).
  - **Shown**: the player's own resources (`PerceptionView::own_resources`: each held, its value
    and maximum, in resource order): the status bar after `MP` (`Rage 11`, `Focus 7/10`), each
    with the hit points' priority after the mana, so they give way before it (§6); a row each on
    the character screen under the stance; the dump header (`rage 11/20`), JSON dumps'
    `resources`, and the dev `actors` layer (every actor's, with progress). Never a creature's,
    and never the pace. The dev action `resource:ID` grants one (§9; "You gain Rage: 12 (dev).").
  - **Rage** (the berserker's, R1; `world/rage.rs`): the Rage ability (a wait's time) grants the
    Rage resource (20 at most, starting at 12, a point lost every 6 s, resting too; it ends at
    zero) and with it Raging; the action's own wait costs the first point. Fighting feeds it
    (#842, `builds` in data): every blow, shot, or throw that hits the holder builds 1 (`Struck`,
    raised on every strike that `is_hit()`: soaked and resisted to nothing too, the hit taken
    before mitigation), a critical of theirs that does damage 2, and a kill credited to them 3;
    a blow of theirs that lands builds nothing. Exhausted (150 s) follows its end; no rage while
    Raging or Exhausted lasts. While Raging, Battle-scarred (level 6) gives 2 Protection more
    (3 in all), Headlong (12) takes 20 % off every step, and Blood frenzy (40) 20 % off every
    melee attack (the talents' `while = "raging"`, §8 "Class abilities"). Blood frenzy: each
    kill credited to the berserker builds 5 rage more (a `resource_build`, 8 in all). Mighty rage (50): Raging's raises doubled, 10 more rage at most,
    Exhausted eases twice as fast. Unstoppable (60): while raging, bleeding and poison tick
    nothing, and a blow (`Harm::Actor`) that would take the last hit point leaves it for 10 rage
    (`World::spend_resource`; refused, so the blow falls, with less), as often as the rage
    holds; spent to nothing, the rage ends and Exhausted follows. While raging no mana is spent
    (Raging blocks `cast`).
  - **Talents** (R5, #253): the `resources` system is in the systems list, named by
    the five resource talents, the only picks limited to a class (a resource belongs to the class
    that has it; Zealot by its prerequisite, Smite): **Battle trance** (Might; berserker, level 6): rage starts 6 higher (18) and
    decays at 67 % of its pace; **Inner calm** (Finesse; ascetic, level 18): ki holds 2 more (7) and
    comes back at 150 %; **Steady hand** (Weaponcraft; ranger, level 12): focus comes back at
    150 % (still not while resting); **Patient blade** (Shadow; thief, level 32): openings decay
    at 50 %, and a sneak attack that kills builds one more (two in all); **Zealot** (Grit; who
    holds Smite, level 6; #914): a blow that hits them builds a second zeal. Rage, Blood frenzy,
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

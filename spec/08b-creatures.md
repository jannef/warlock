# Warlock specification — 8. Architecture (continued): creatures

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Creature behavior** (`warlock-core/src/world/ai.rs`, inside `World::creature_turn`; M10):
  quite deterministic, outplayable by design; no randomness but one contest. Each action, in
  order:
  1. **Look**: the hostile actors it sees (§5 "Creature sight"; hostile by faction, §7); the
     nearest is its target (squared distance; ties: the lower actor id). The player may slip
     past it here (§8 "Surprise and sneak attacks", `world/slip.rs`): one that would first see
     them by darkvision alone rolls a contest of their Stealth against its Spot, once, and won,
     does not see them by darkvision (nor their noise as one it sees) until the slip ends.
  2. **Fight**: a `melee` creature attacks a target next to it (diagonals too), else steps along
     a shortest path toward a cell next to it. A `ranged` creature or `caster` shoots or casts
     when the target is within its `range` (the rounded radius, like every radius; for a shot no
     farther than its missile weapon's range) with a clear line of fire: no opaque cell and no
     actor on the cells between (an integer Bresenham line, the one missiles fly); else it steps
     toward the nearest cell that gives it one, failing that toward the target. A `ranged`
     creature with nothing to shoot (no missile weapon ready, or no ammunition for it left) fights
     as a `melee` one. A creature with a throw (§8 "Ranged combat": "Creatures' throws", the
     giants' boulders) that is not next to its target throws first whenever the target is within
     the throw's range with a clear line of fire. A summoner ("Summoning" below) summons before
     all of this whenever it may, and a gazer ("Gazes" below) then looks it in the eye
     whenever its gaze is ready and the target within its range. A `dragon` ("Dragons" below) breathes whenever its breath is
     ready and the target stands in its cone with a clear line, and else fights as a `melee`
     one. A `caster` ("Casters" below) casts whatever spell of its list goes from where it
     stands, else closes to where one would, and with none it can cast (out of mana, Silenced)
     shoots if it can, else fights as a `melee` one. A **lurker** (a type with `lurks = true`: the meenlock, the
     barrow spider; `world/lurk.rs`) never steps toward the target it sees: it waits where it
     stands, aware of it, attacking once the target is beside it (or shooting or casting when it
     can from where it stands), until it has been harmed (hit points below its maximum), when it
     fights as any creature. No draws; with no hostile in sight it searches and listens as usual.
     Leaving its routine it remembers where it was (its post) and where it saw the target.
  3. **Search**: with no hostile in sight, it goes to where it last saw one (next to the spot if
     someone it cannot see stands there, or no one can stand there: a door shut or kicked at).
  4. **Listen**: failing that, to where it last heard a noise (§5 "Hearing"; the newest it heard
     since it last acted) whose source cell it does not see (next to it, as above, when someone
     stands there or no one can). Only creatures whose faction is hostile to someone go to look;
     wildlife does not care.
  5. **Return**: then back to its post (waiting beside it while someone else stands on it), and
     on along its patrol route from where it left it (or it waits there). A creature the player
     swapped places with has its old cell as its post ("Displacement" below), so it steps back
     and carries on.

  Paths are A* over king's moves within a bounded box (12 cells beyond the target, at most 64
  from the creature), each step costing 1: walkable cells, and closed doors for creatures that
  open them (1 more: it opens the door, a turn and a noise, then steps in); a locked door only
  for one carrying a key that fits it (1 more again: it unlocks the door as it opens it, §8
  "Doors"), to every other creature a wall. Another actor's cell
  costs 4 more, so a creature walks a short detour around someone in its way but waits behind
  them rather than take a long one. Ties go to cells nearer the target, then to the earlier
  direction (N, NE, E, … NW). A spot it cannot reach is given up. A creature hostile to the
  player never steps into a sanctuary (§7 "Special rooms") from outside it, on a path or on its
  patrol (it waits instead); one already inside may move about and leave. It may still strike
  across the threshold or shoot in.

  **Attacks** are resolved by the rules of combat ("Combat" below): melee blows and shots hurt
  and kill (a shot is a missile, "Ranged combat" below), and so do a caster's spells ("Casters"
  below, through the player's cast path, §8 "Spells"). An attack is an event (`Event::Attacked { attacker, target, kind: Melee | Ranged |
  Thrown | Spell, from, to, strike, with }`, `strike` what came of it, `None` for an intent,
  `with` the missile of a shot or throw), a noise (§5), and a message when the player perceives
  it (§6). The event reaches presentation only
  when the player sees both ends (or made the attack and saw the target). The player moving into
  a hostile creature is a melee attack; into any other one it swaps places with it
  ("Displacement" below). Creatures neither flee nor
  fight differently when hurt. What a creature has in mind (last seen, noise to look into, post)
  is part of the world state.
- **Stair followers** (`warlock-core/src/world/stair_followers.rs`; #985): stairs are no free
  escape. A creature beside the player (Chebyshev distance 1) as they take a staircase, hostile
  to them and aware of them, comes along and arrives beside them; several are told in one line
  when the player saw them as they left ("The kobold follows you down the stairs.", "Three
  kobolds follow you up the stairs."). Those the rules keep still or out of action do not come:
  one asleep or disguised, one whose steps a status blocks (Held, Webbed, Pinned), one
  flat-footed (Stunned, Surprised), one fleeing (Frightened), one unaware of the player. Nor one
  that keeps its place (a `budge` other than `always`, a mimic even found out, a lurker unhurt),
  an NPC, a shop's keeper, or an ally. **Where**: on free walkable cells beside the player's
  arrival, in id order, after the player's allies have stepped out, and never on ground a
  hostile shuns; with no such cell left it stays behind, frozen on the old level. **What it
  keeps**: hit points, statuses, gear, and awareness (it knows where the player arrived); its
  post, noises, and patrol route, which belonged to the old level, are gone. **Timing**: it pays
  the stairs' price from its own energy as the player does, so it gets no blow out of their
  climb: at equal speed the player acts first. It counts as a hostile of the new level toward
  the spawn cap. Its kill counts for a kill quest of the level it first left (`Actor::home`). No
  draws. **Only stairs**: a staircase between levels, not one out to the overworld (which has
  no creatures: it stays behind). Walking off an outdoor level's edge and the quick leave are
  refused with enemies near (§7 "Leaving an outdoor area"); a fall (a collapsing floor), entering a
  local area from the overworld, and an encounter's area bring no one. Stealth counts only as
  far as a creature slipped past is unaware; no talent reads it.
- **Incorporeal creatures** (`warlock-core/src/world/incorporeal.rs`; #381): a creature type
  with `incorporeal = { wall_defence = N }` (N 0 to 20) passes through **solid** terrain (neither
  walkable nor transparent: walls, rock, trees, closed doors, locked or not) and may stand inside
  it, never on a level's edge; it slips through a closed door without opening it. Transparent
  obstacles (lava, deep water it cannot walk) stop it as they stop anyone. Its paths
  (`ai.rs::route`) count a solid cell as a step plus one, so it keeps to open ground unless the
  stone is the shorter way, and a goal cell may be a solid one. Inside solid terrain its Defence
  gains its `wall_defence` in the **Wall** category (§8 "Combat"). It attacks out of the wall
  like any creature; the player moving into a solid cell strikes at a hostile one they **see**
  there (else the terrain is in the way as ever, so nothing unseen leaks), and never swaps places
  into a wall; a missile reaching that cell attacks it there, lost in the stone if it hits,
  falling before the wall if it misses (§8 "Ranged combat"). Sight has no rule of its own: the
  player sees it when they see its cell, as they see any wall (its face lit along an open line,
  or within darkvision); creatures see one inside a wall within their darkvision only, their
  rule for opaque cells. Nothing of it presses down: it sets off no trap and breaks no ice.
  Noise is unchanged (its `footsteps` by the terrain's `step_noise`). When it dies inside a
  wall, what it carried falls on the nearest walkable cell (Chebyshev rings, row order).
  Populations and spawns place creatures on walkable cells only, so one never comes inside a
  wall; a level legend may place one there. No new state: its position is a plain cell. It has
  no vitals (§8 "Surprise and sneak attacks"): immune to criticals and sneak attacks.
- **Vitals** (#682; `Content::has_vitals`): a creature type has vitals to find unless it is
  incorporeal, an animated object, carries a tag of `surprise.toml`'s `no_vitals` (elemental,
  skeleton, spirit, construct), or says `vitals = false` (default true). Without them it takes
  no critical and no sneak attack dice (§8 "Surprise and sneak attacks"); the fleshy undead have
  them. Rules only: the wiki says which.
  Not yet: immunity to weapons that are not magical (the hook is `World::resolve` in
  `fight.rs`); bolts and bursts stop at walls, so spells do not reach one inside. The example is
  the stone haunt (`stone_haunt`), which no level places yet.
- **Summoning** (`warlock-core/src/world/summon.rs`, the rules; `content/summon.rs`, the data;
  #520): an enemy-only creature ability; the player has none. A creature type's
  `[creature.summon]` names its pool (creature ids, each as likely, or a tag within a level
  window, uniques and itself left out; a shared type calls only shared types; a list may name
  the summoner's own type, a breeder's, docs/design/monster-dread.md §5.5 G5c), `count` (dice, 1
  to 12), `cooldown` (6 to 600 s), `mana` (its cost, 1 to 1000; the type must then have mana;
  left out, none: a breeder's, whose cooldown and `most` are its limits), and optionally
  `range` (default 8), `spread` (default 2), `most` (default 6), `verb` (default
  "summons"), and `faction` (default the summoner's; a miscast's gates let theirs through
  `loosed`, #1316). As step 2 begins, a creature with one summons when it sees a hostile within
  `range`, its casting is not blocked (a status that blocks `cast`: Silenced), its cooldown
  status `summoning` is off, it has the mana (`World::spend_mana`, its cut included), fewer than
  `most` of those it summoned live (on any level), and a free cell lies within `spread`; deciding
  draws nothing. It then pays the mana, takes `ceil(cooldown / eases)` degrees of `summoning`
  (the house rule: an ability's limit is a cooldown status), makes a spell's noise, and rolls
  `count` on the gameplay stream, capped by the room `most` leaves; each creature is one
  `below(pool.len())` draw, made like a spawn (gear rolled from `gear/<actor id>`) on the next
  free cell: within `spread` (Chebyshev) in its line of sight, walkable, no one there, not a
  sanctuary it would shun, nearest first (Chebyshev distance, squared distance, row order). Fewer
  cells, fewer come. The summoned take the summoning's `faction`, else the summoner's (a `faction_override` when it is
  not their type's) and remember who called them (`Actor::summoner`, saved, left out while
  `None`); they act from their next turn, and nothing else sets them apart: they stay when the
  summoner dies, and their kills pay full experience. The action costs a `cast`. Words: the
  player who sees the summoner is told what they see come ("The goblin wizard calls forth 2
  goblins and a warg!", "something" when they see none of it); an unseen summoner's creatures
  only as they appear in sight ("A skeleton appears!"); nothing seen, nothing said. Sight is the
  last turn's perception, so a summoning in the turn a door opens may go unseen.
- **Auras, innate statuses, and cries** (`warlock-core/src/world/aura.rs`, the rules;
  `content/aura.rs`, the data; #1128, design `docs/design/monster-dread.md` §5.4, §9). A type's
  `[creature.aura]` (`status`, `severity`, `radius` 1 to 6, optional `check` and `difficulty`,
  `every` 6 to 600 s (default 12), `heard` with `loudness`, `damage_type`, `touch`, `unseen`):
  as its turn starts, unless its cooldown status `aura` is on, every hostile within the radius
  that it reaches (line of sight, or for a heard aura the hearing rules for a voice that loud
  where it stands) makes the check or takes the status, caused by it, the damage type's
  resistance taking its share; anyone reached starts the cooldown, `ceil(every / 6)` degrees.
  The player taking it anew hears its `touch` first ("A chill comes off the wraith."), its
  `unseen` line when they do not see the bearer. No time, no noise. An ally of the player's has
  none. `innate` statuses: an actor is made with them and they come back as each of its turns
  starts; the look line never shows them. A `cry = { kind, loudness, every }`: after a turn
  that was not a fight, a chance of 6 in `every` (one gameplay draw on its turn) of
  `Act::Cry(kind)`, a noise heard by the hearing rules ("You hear a hiss.", the sound layer).
- **Splitting and dying into** (`warlock-core/src/world/split.rs`, the rules;
  `content/split.rs`, the data; docs/design/monster-dread.md §5.5 G5a and G5b, #1129): an
  enemy-only creature gimmick. `splits = { on = ["slashing", "piercing"], least = 8 }`: a blow
  (melee or missile, anyone's) of a listed damage type that leaves the creature alive with at
  least `2 × least` hit points halves them; it keeps the greater half, and a second creature of
  its type takes the other on the first of `summon_cells(id, 1)` (none free: no split), with no
  gear, the spawner's cap ignored, its side, summoner, and mind copied, and `Actor::split_from =
  Half`. No draw. `dies_into = { creatures = [...], count = "2d4", spread = 1 }` (count 1 to 20,
  spread 1 to 5, default 1; never the player, a unique, or itself): as it dies, on the
  player's level, `count` is rolled on the gameplay stream and each is one `below(pool.len())`
  draw, made on the next of `summon_cells(id, spread)` with the dead one's faction and no gear,
  `split_from = Corpse`. A kill of a half pays 25 % of its experience (after an ally's share), of
  a corpse's creature 10 %, and neither counts toward a quest's kills (`tally_kill`). Words: the
  first split of a type the player sees in a run, "The charnel heap comes apart, and both
  halves crawl." (`Progress::once` `split:<type>`), later "The charnel heap splits in two.";
  "3 grave worms crawl out of the wormhost!" with the death, for what the player sees; look
  mode adds ", writhing" after a `dies_into` type's parentheses. Bearers: the charnel heap and
  the black pudding (split), the wormhost and wormlord (die into grave worms; the wormlord breeds
  them too), the hydra (dies into its heads, `hydra_head`, met only so).
- **Dragons** (`warlock-core/src/world/breath.rs`, the rules; `content/breath.rs`, the data;
  #1099): an enemy-only creature ability. A `combat = "dragon"` type has a `[creature.breath]`
  (and only a dragon has one): `damage` (dice), `damage_type` (its element, one type),
  `length` (the cone's, 2 to 12 cells), `cooldown` (6 to 600 s), `effect` (a burst effect drawn
  over the cone), and optionally `noun` ("fire"; default "breath"), `verb` ("breathes fire";
  default "breathes"), and `riders` (statuses with a severity and drains with their check, as a
  gaze's: never a disease or a hold; #1138). In step 2, after a summoning and a throw, a dragon breathes when its
  cooldown status `breath` is off and its target stands in the cone it would breathe at it (the
  spells' shared cone, `spell::cone_cells`: within `length`, a quarter turn wide, in line of
  sight) with a clear line of fire (as a shot's); deciding draws nothing. Otherwise it fights as
  a `melee` creature, so it never stands off. Breathing, it takes `ceil(cooldown / eases)`
  degrees of `breath`, roars (`noise::Act::Roar`, heard as far as a howl), and the cone strikes
  through `World::cast_cone`, the one place a cone strikes (Burning hands too): one roll for
  everyone in it in ascending actor id order, no roll to hit, Evasion halving it, resistances
  taking their share; then its damage type's spell type falls on those it harmed
  (`World::spell_type_lands`: fire's burning, one in four), and each of its `riders` by its own
  chance on each of them still alive, as a blow's (`World::rider`; the gorgon's grey vapour, a
  degree of Stiffening unless a Toughness check against 17 is made). Its blows and kills are the
  dragon's. The action costs a `cast`. Words: "The fire drake breathes fire!" when the player
  sees it, then each blow seen as a spell's ("The fire burns you."); unseen, the roar is heard.
- **Gazes** (`warlock-core/src/world/gaze.rs`, the rules; `content/gaze.rs`, the data; #1125):
  an enemy-only creature ability, a rider at sight and not at touch. A type's `[creature.gaze]`
  has a `range` (1 to 8 cells), `riders` (statuses with a severity, drains with their check, each
  by its own chance, as a blow's `on_hit`; never a disease or a hold), and optionally `chance`
  (percent, default 100), `cooldown` (6 to 600 s, default 12: the `gaze` status), `averted`,
  and `glance` (percent, default 25). In step 2, after a summoning and before a throw, a gazer
  whose cooldown is off looks its target in the eye when it stands within `range` and has
  vitals (what has none has no eyes to meet); its target is one it sees, so never through a
  wall. It takes the cooldown and spends a `cast`. The eyes must meet both ways: the player
  sees the gazer (its cell in their sight, not disguised), a creature by its own eyes. Unmet,
  the stare is wasted: "You feel a cold stare on you." Met, one draw at `chance` (a `glance`'s
  share of it against averted eyes) catches them: "The vampire's eyes find yours.", then each
  rider falls as a hit's would (the status's own `start` line); missed, "You tear your eyes from
  the vampire's." (averted: "You keep your eyes from the basilisk's."). At a creature the player
  sees the gazer stare: "The vampire stares at the wolf." A gaze is silent. **Averted eyes,
  forced** (the owner): while the player sees a hostile whose gaze is `averted`, they avert their
  eyes by reflex: no toggle, no action, nothing saved, derived each turn from what they
  perceive as being dazzled is. Averted, they fight at −4 to hit, their Dodge halved, with no
  free blows, and the status bar says "Averted" ("You avert your eyes." / "You raise your eyes
  again."). Anything else that keeps them from seeing the gazer (darkness, a wall or a door, a
  disguise) means neither the penalty nor the gaze. The gazers: the vampires (transfixed; the
  bite keeps its Toughness drain), the sea hag (frightened and a Toughness drain), the spectre
  (a Willpower drain), the white lady, the night mare, and the ghost lord (transfixed; the touch
  keeps its drain), the hand of glory (transfixed, 3 cells), the ultroloth and the rakshasa
  (transfixed and frightened), the succubus and the incubus (transfixed; the kiss keeps its
  drain), the duergar mindbreaker (frightened and transfixed), and the basilisk, the one gaze
  `averted` (6 cells, `glance` 25: a degree of Stiffening unless a Toughness check against 15 is
  made; #1126). A status rider may carry a check as a drain does (`check` and `difficulty`): the
  target resists it with its effective modifier, what its talents add to that stat's checks
  (Iron will), and Knowledge, as a drain's check (`World::resists`).
- **Bursts** (`warlock-core/src/world/creature_burst.rs`, the rules; `content/burst.rs`, the
  data; the dread design §5.3, #1127): a type's `[creature.burst]`: `damage` (dice),
  `damage_type`, `radius` (0 to 3), `when`, `effect` (a burst effect), optionally `noun` (what
  strikes: "flame"; default "burst"), `verb` (default "bursts", or "comes apart" for death
  throes), `rider` (statuses with a severity and a chance), `douses` (a wave), `gust`. A
  **vortex** (`when = "touches"`; no `attack`, `combat` melee) bursts instead of attacking: every
  melee plan of its AI is a burst, after which it dies; killed, it never bursts. **Death throes**
  (`when = "dies"`) go off where the creature dies, however it dies, on the player's level. The
  burst is the spells' shared burst centred on its cell (`cast_burst_reaching`): one roll for
  everyone on its cells in ascending actor id order, in full (no halving at the edge), no roll to
  hit, Evasion halving it, resistances taking their share, creatures of its own type spared, the
  floor's ruin and the spell type's cells as a spell's burst; then a wave ends what water ends
  (Burning) on everyone reached and puts out the fires, a gust blows each pile on its cells but
  the centre a cell outward onto a walkable cell, and its riders land by their chance on those
  harmed (the spell types' riders do not). Its blows are the creature's, dead or not. Words: "The
  ember mote bursts!", "The fire elemental comes apart in flame!" when the player sees its cell
  (death throes after the death line), each blow seen as a spell's; unseen, an explosion
  (`SoundKind::Explosion`, 150). Who: the six motes (vortices, `2d4` to `2d6`, radius 1), every
  elemental line but earth (death throes of `(2 + L/8)d6`, half the doubled damage spells'
  `(4 + L/4)d6` of their level; radius and rider by rung, `docs/dev/creatures/elementals.md`), the
  burning skull, the magma crawler, the animated lantern, and the balor. The earth line throws stones instead
  (`[creature.throw]`).
- **Corroding gear** (`warlock-core/src/world/corrode.rs`; #1131, #1168, the dread design §5.8): a
  type's `corrodes = { chance, what, kind }` (chance 1 to 100; kind the item damage, `acid`,
  `rust`, or `fire`). `what = "armour"`: its melee hit that lands on the player rolls the
  chance; made, one piece the player wears (armour or a shield) is drawn and takes the harm
  (`world/item_damage.rs`, spec §7 "Materials"): its main part's material withstands it by its
  `resists` chance of the kind, else it is damaged (the anvil's state: improvements gone, a point
  worse until mended): "The acid eats at your chain mail shirt.", "Rust spreads over your chain
  mail shirt."; one damaged already is destroyed: "The acid eats your corroded chain mail shirt
  away." `what = "weapon"`: the player's melee hit on it rolls on what struck (the main hand's
  weapon, the off hand's, or the bashing shield): "Your long sword comes away pitted." `what =
  "both"`: each of the two at the chance (#1138). Any resistance to the damage type named like
  the kind (acid, fire) halves the chance; a spirit weapon and a stack of several are left be.
  Withstood: the first time in a life for each kind, "The acid runs off your mithril chain mail
  shirt.", "Your leather armour takes no rust." (`Progress::once`). Draws: one for the chance,
  then one for the piece and one for the material's chance unless it is 0 or 100. The ember beetle (weapon, 25; its acid blood, the innate `acid_blood`,
  also strikes back at a melee blow for `1d3` acid), the hezrou (armour, 25), the dretch (armour,
  10), and the black pudding (both, 25) are acid; the rust monster (both, 50) is rust.
- **Regeneration** (`world/regen.rs`; #1131, the dread design §5.9): a type's `regen` (percent of
  the normal healing pace, 0 to 1000, default 100) counts among its statuses' paces in
  `Effects::regen`, so a status that slows healing slows it (Burning and Corroded have `regen =
  0`: fire and acid stop every healing, the player's too). The player who sees a creature with
  `regen` above 100 get a tenth of its hit points back in one passing of time is told "The
  troll's wounds close.". Only a brute of its level or a dread type regenerates (the roster
  test): every troll brute at 400; never the rotting troll.
- **Theft** (`warlock-core/src/world/snatch.rs`, the rules; `content/theft.rs`, the data;
  docs/design/monster-dread.md §5.6, #1130): an enemy-only creature ability. `steals = { what,
  chance, cooldown }` on a type with hands (`what` = `small`, coins, gems, potions, rings, and
  scrolls, or `any`; `chance` 1 to 100; `cooldown` 6 to 600 s): when one of its melee blows hits
  the player and leaves them alive while its cooldown status `theft` is off, the chance is drawn;
  made, it takes one whole stack of what the player carries and neither wears nor wields, of the
  classes `what` allows, picked uniformly, into its own pack, and takes `ceil(cooldown / eases)`
  degrees of `theft`. Beside the player with its theft ready, a thief strikes in melee rather than
  cast (`World::snatches_at`). With nothing it may take, nothing happens. It never robs a creature, and it
  neither flees nor blinks away: it fights on, and what it took falls where it dies. Words:
  "The pixie snatches 40 gold coins from you!" ("Something" when unseen). The pixie, the imp,
  the succubus, and the incubus take what is small, the nymph anything.
- **The blink** (`warlock-core/src/world/blink.rs`; `content/theft.rs`; #1130): `blinks =
  { range, cooldown }` (2 to 12 cells, 6 to 600 s), the blink dog's alone. In step 2, a melee
  fighter whose way to its target is blocked (its path held by someone, or none: a door it
  cannot open, a gap no paw crosses) while its cooldown status `blink` is off moves at once to a
  free cell beside the target, within `range` of it and in its own line of sight, where a
  teleport could land (no trap, no ice), drawn among those cells; it takes `ceil(cooldown /
  eases)` degrees of `blink`, and the blink costs a step. It never blinks away. Words: "The
  blink dog is suddenly beside you." where the player sees it land; unseen, nothing.
- **The disarming blow** (`warlock-core/src/world/knock_away.rs`; #1166): an enemy-only creature
  ability, the duellists'. `disarms = { chance, difficulty }` on a type (`chance` 1 to 100,
  `difficulty` 2 to 40): when one of its melee blows hits the player and leaves them alive, with
  a weapon (a thing with an attack) in their main hand, the chance is drawn; made, and with an
  open cell beside the player (walkable, no deep water, no closed door, no one there), the player
  makes a Dexterity check against `difficulty`, adding their rank in the weapon's skill and
  their grip (+2 a two-handed weapon, +1 a shield in the other hand, their rank in the lone
  weapon's style, Duelist, with the other hand empty). Failed, the whole stack leaves the hands
  and the pack and lies on one of those cells, drawn among them; a spirit weapon fades instead.
  Without an open cell, nothing happens. Words: "The drow blademaster knocks your scimitar from
  your hand!" ("Something" when unseen), then "It lands beside you.". The drow blademaster and
  captain, the death knight, the ultroloth, and Wælcyrge the Chooser.
- **Casters** (`warlock-core/src/world/creature_cast.rs`, the rules;
  `content/creature_spells.rs`, the data; #1102): a `combat = "caster"` type has a `spells`
  list (ids in `data/spells/`, in the order it prefers them; only a caster has one), and only
  spells a creature can cast: harm (no zone, no corpse, no open sky), a hex laying a plain
  status (no zone, push, charm, or ground it needs), a heal by touch, a buff laying a status.
  It casts them through the player's cast path: the same reach, roll, and degrees ladders at
  its own stats (`Cast::by_creature`), the same bursts, lines, cones, and single blows
  (`cast_single`, so a shield that turns spells turns its spells back), the spell types,
  riders, and resistances; an `attack` spell's roll adds its level's quarter, as its natural
  attacks' do. Each cast costs the spell's mana (`Actor::spell_cost`) from its pool (its type's
  `mana` and its mental stats'), which comes back at the usual slow pace; a cast trains
  nothing and costs a `cast`. In step 2, after a summoning, a throw, and a breath, it casts the
  first spell of its list that it has the mana for, its casting not blocked (Silenced, a silent
  vault), off the overworld, and that goes from where it stands: a buff it lacks, on itself; a
  heal by touch on itself or a friend beside it below half its hit points (never an undead,
  whom a heal harms, and an undead caster heals no one: §8e "Spells"); a hex at its target
  if the target lacks the status and its type is not immune to it, in reach; harm in reach. In
  reach is the shape's: a ray or a bolt within its range with a clear line, a touch beside it,
  a burst within range with a clear line and the caster outside its cells (one about the caster:
  the target in its cells), a cone or a line whose cells hold the target with a clear line; an
  area is never cast over a friend of the caster's (anyone not hostile to it). Else it closes to
  the farthest reach of the aimed spells it could cast at the target (at most its type's
  `range`), and with none fights on ("Fight" above). Deciding draws nothing. Words: "The kobold
  shaman casts spark at you!" (or "at the goblin.", "casts haste.", "casts mend on the kobold.")
  when the player sees it, then each blow as a spell's ("The spark shocks you."); unseen, its
  chanting is heard. Up to level 10 a caster knows only level-1 harm and hexes, heals, and buffs
  of level 3 or less, and is very rare.
- **Several natural attacks** (`warlock-core/src/world/natural.rs`; #1099): a creature type may
  list its natural attacks (`[[creature.attack]]`, a dragon's two claws and a bite) instead of
  one. Wielding no weapon, its melee attack strikes the first as its blow and then each of the
  rest in turn, every one rolled on its own against the target's Defence (`World::attack_with`,
  noise and all) until the target or the attacker is gone; together one attack's price. Each
  blow's `Event::Attacked` names the natural attack after the first (`natural`, its index), so
  it is said in its own words ("The fire drake claws you."). A natural attack's `on_hit` rides
  on its own blow's profile (`Actor::melee`, `Actor::more_natural_blows`). Wielding a weapon,
  the weapon's blow stands in for the first and the rest are not struck.
- **Allies** (`warlock-core/src/world/ally.rs`, the rules; `content/ally.rs` and
  `data/allies.toml`, the numbers; #836, design `docs/design/druid-and-summons.md` §4 and §14):
  creatures on the player's side, a summon (the druid's calls, #839) or a companion (the
  ranger's, #916, below). An ally is an ordinary creature of its type (stats, speed, eyes, attacks,
  hit points, statuses, death) made with the player's faction, so every rule that reads stances
  (hostile creatures, the spawn cap, the quick leave, resting, swapping places) treats it as the
  player's. **Its mind**: it attacks the nearest actor it sees that its *master* is hostile to
  (a Beast-friend's wolf spares the rats as the druid does; a shopkeeper is never an ally's
  idea), else obeys its order: *follow* (heel within `heel` cells of the master; fighting farther
  than `leash` away, it breaks off and heels), *stay* (hold its cell, fighting only what comes
  within `stay.reach` of it), *hunt* (go to the cell it was set on and fight what is within
  `hunt.reach` of it; arrived with nothing to fight, it comes back to heel). It draws from no
  stream. **Orders**: three abilities of the Allies group of the abilities menu, no key of their
  own, given to every ally on the level at once: Come to heel, Hold here, Set on (a cell the
  player sees). An order takes **no time** and is a voice (`[orders] noise`) where the player
  stands; refused with no ally on the level. There is no sending away: a summon departs when its
  time runs out or it is slain. **Following**: allies within `comes_along` cells of the player
  as they take stairs or leave an area come along, placed beside them; the rest stay where they
  are, frozen. Onto the overworld, which has no creatures, they join the player's **retinue**
  (never drawn, never acting, never in the way) and step out beside them at the next level.
  **Time**: a summon's stay ends by the clock, wherever it is: one whose time has passed departs
  where the player is ("The wolf fades away.", nothing left behind), and is gone when they
  return to a level they left it on, or when the retinue steps out. **Experience**: an ally's
  kill pays its master's experience, the first-kill bonus too, at their class's `ally_share` (the
  pet classes, the ranger and the druid: 100) or else `[xp] ally_percent` (50; #915, the owner:
  "pet classes get 100% of xp share, but others get 50%"), whatever the ally's source; the quest
  log, Favoured prey, the named monsters slain, the alignment, and what a kill builds are
  credited in full ("Credit is still shared fully for quests"); the ally gets nothing. Handle animal earns its marks from an order given, an ally's kill, and a try at taming. At most
  `[allies] most` (8) allies on one level. **Beast-bond** (Wayfarer; Handle animal 20): your
  allies strike +1 to hit and take Protection +1, read through you. The messages never say a
  number. The dev `ally:CREATURE[:N[:DURATION]]` makes allies for tests and captures.
  **The risen** (`world/corpse_spell.rs`; #999): Rouse the dead stands a corpse's own creature type
  (`ObjectDef::corpse`) on its cell or the nearest free one, at half its maximum hit points rounded
  up, with no gear, pack, `npc` id, or mimic's disguise. It serves as a summon does (`most` 2, ten
  minutes), and a second death leaves nothing. A `unique` type's corpse is refused. One time in ten
  (one `below(100)` on the gameplay stream, after every refusal) it rises **berserk** instead: in
  the faction `berserk`, `hostile_to_all` (hostile to every faction and to itself), by the plain
  AI, no ally, under no orders, counted toward neither `most` nor the cap nor a level's population,
  on the same clock (`Actor::risen`). Its kill pays as any of its type.
  **The charmed** (`world/charm.rs`; #1003): Beguile, a hex with `[spell.effect] charm`, makes a
  hostile creature the player sees within its range their ally (`AllySource::Charmed { spell,
  faction }`, `faction` the `faction_override` it had) for its degrees in minutes, following and
  under orders, on the player's faction: its own band turns on it, and its kills pay as an ally's.
  No check and no level limit. Refused, costing nothing, on a `unique` type or an `npc`, an
  animated thing, a creature with one of the charm's `mindless` tags (undead, construct, vermin)
  or risen berserk, one already an ally, and at `[allies] most`. When its time is up it is itself
  again where it stands (its `faction_override` put back, no ally; "The ogre shakes off the
  charm."), and the charm ends at once when the player leaves its level: it never comes along nor
  joins the retinue (the owner's Q12), and it was an ally when hostile stair followers were
  chosen, so it never follows either. It keeps its gear and, slain, leaves its corpse and pack.

  **Summons** (`world/summon_spell.rs`; #837): a spell of `kind = "summon"`, cast on oneself,
  calls `count` creatures of its pool (`[spell.effect] summon`; the count a roll, grown by its
  `count` ladder and the dice hooks) beside the caster within `spread`, nearest first, on the
  player's side with no gear, for `minutes` (its `degrees` ladder, Lingering), each drawn as an
  enemy summoner draws (a summon bias, a talent effect, weighs the pool by tag; nothing gives
  one yet). A spell keeps at most `most` of its calls at once: past it the oldest departs, so a
  call always brings one. Refused, costing nothing, on the overworld, without a free cell beside
  the caster, and at `[allies] most` on the level (unless one of the spell's own gives way).
  `ally_mod` talents bring `count` more, make calls last `duration_percent` longer or, with
  `lasting`, until slain, and keep `most` more of each spell's. A slain summon leaves no corpse
  and drops nothing. "Two wolves come to your call." The druid's calls (#839, §8 "Class abilities"):
  Call of the wild (a wolf), Call of the forest (a bear), Call the elements (a greater
  elemental, the element drawn), one of each at once until Pack leader and Lord of the wild
  raise it. The beasts (`wolf`, `bear`) are wildlife, never strays. The messages call an ally
  the player sees "your": "Your wolf bites the kobold.", "Your wolf dies."; one whose time runs
  out goes in its type's words ("The wolf slips back into the dark."). **The animal companion**
  (`warlock-core/src/world/companion.rs`, `content/companion.rs` and `data/companions.toml`;
  #916, design `docs/design/ranger-companion.md`, `docs/dev/companion.md`): an ally that is the
  player's for good, `source = Companion(CompanionData { name, grade, .. })`, `until = None`; at most
  one at a time. The ranger's **Animal companion** (level 1) brings one as the run begins: a
  beast drawn from the character's race's list (`[start.by_race]`: a human's hound or boar, an
  elf's lynx or hound, a goblin's large spider, rat, or bat, …; one to three a race, of level at
  most `most_level` 2), or at `rare_per_mille` (50) one of the rares shared by every race (a
  wolf, a black hound, a great boar, a panther), on its own stream (two draws); named from
  `names` (Old English: Snel, Brūn, Græg, …; one draw on its own stream); "Snel, your hound, pads
  at your heel." A character with a starting beast is told a line of it in the story's
  **companion** chapter (§8 "Background story"). Only `tameable` creature types may be tamed
  (the shipped ones are plain beasts); a type's `nose` (in cells) is how far it smells. **Growth**: the companion's `grade` is its master's level
  (`[growth] grade_percent` 100 of it, never below its type's own), re-set at every level-up
  ("Snel grows stronger."); above its type's level each level adds `hp_per_level` (2) hit points,
  and every `to_hit_every` (4) +1 to hit, `damage_bonus_every` (6) +1 damage, `damage_dice_every`
  (12) one more die of its attack, `defence_every` (6) +1 natural Defence, `stat_every` (5) +1
  Strength, Dexterity, and Toughness; its natural Protection is its body's at its grade (the body
  rule above), never below its type's; speed, senses, size, and resistances stay the type's;
  summons have no grade. **Always along**: a companion comes with the player on every level
  change wherever it stands on the level, held fast or holding or not ("Snel comes with you."),
  rides in the retinue on the overworld and steps out at the next level ("Snel follows you
  in."); nothing but death keeps it from a level. **The nose**: a companion whose type has a
  `nose` smells a hostile of its master's within that many cells, however dark and **through a
  closed door** (never through a wall), and goes for it when its eyes find nothing; "Snel growls
  at the dark." once per thing scented unseen. The words say its name where they would say "your
  hound": "Snel bites the kobold.", "Snel comes to heel.", "Snel dies." (a corpse, as any
  creature's: a companion is no summon). The player sees it in the status bar ("Snel: unhurt",
  "Snel: with you"), in look mode ("Snel, your hound (unhurt), …"), on the Character screen
  ("With    Snel, a hound"), and in the `companion` dump; the wiki states the lists, the odds,
  and the growth table exactly. **Tame** (`world/tame.rs`; #917; the abilities menu, a skill
  ability of Handle animal, `tame:DIR`): anyone trained in Handle animal, at any level, without a
  companion ("You already have a companion."; a summon is no companion), may try a `tameable`
  beast they see next to them that is no one's ally and is not hostile to them (wildlife, or an
  animal a Beast-friend calmed) or has not seen them (`World::unaware`: asleep, say); else
  refused, taking no time ("The warg has seen you: it will not be tamed now."). `[tame] turns`
  (10) actions of `tame`'s price beside it, silent, stopped by a creature coming into view, a
  message, the player being hurt, or the beast leaving their side, with nothing lost but the time.
  Then the check, with no level gate: Willpower modifier (and what talents add to Willpower
  checks) + Handle animal's `tame` benefit (+10 at 100, +20 at 200) + `[tame] food` (2) if food
  is carried (one of the first food stack, eaten as the check is made) + `TalentMods::tame` (the
  ranger's 2), against `[tame] difficulty` (10) + `per_level` (2) × the beast's level. Made, the
  beast keeps its actor and its place and is the player's companion, its mind cleared, graded at
  their level, named: "The wolf comes to your hand. You call it Græg." Failed: "The wolf snaps at
  you and backs away."; a hostile one has seen the player now, so it may not be tried again until
  it loses them, one not hostile may be tried again at once. A sleeper wakes either way. A mark of
  Handle animal, made or failed.
  **Hunter's bond** (the ranger's level 25, #920; passive):
  *shared sight*: while the companion stands on the player's level (not in the retinue), the
  player also sees every cell it sees by its own line of sight and eyes (creature sight's rule,
  §5: its sight radius, its vision profile, the light as it reaches the cell; wall completion
  around them), and the cell of every hostile its nose finds (within its `nose`, however dark,
  through a closed door, never through a wall); these are seen cells like any other (memory,
  look mode, Set on, targeting, the dumps), shown in the light the player's eyes perceive; it
  finds no hidden trap or secret door, shows no disguised mimic, and a dazzled player keeps it.
  *Shared buffs*: a buff status that lands on the player lands on the companion too (on the
  level or in the retinue), and one on the companion on the player, at the same severity, each
  its own (it eases by its own clock), whatever laid it (a spell, a scroll, a potion, a boon):
  a buff is a buff spell's status or one a potion lends; a hex, a harm, a cooldown never
  crosses, and the shared one does not cross back. "Snel is bark-skinned too."
  **The collar** (`world/collar.rs`; #918): a kept beast wears one
  thing, on its neck. The **Give** ability (the Allies group, listed while the player keeps a beast,
  greyed out "not beside you" with none beside them; `give:LETTER[:DIR]`) asks for a stack the
  player carries that fits a neck; one of it leaves the pack, taken off first if it was on the
  player, and goes on the beast, a wear's time ("You fasten the jade amulet about Snel's neck."),
  and what it wore falls on its cell ("Snel's bone amulet falls to the floor."). With two kept
  beasts beside the player, it then asks which way ("Give it to which side?"); with one beside,
  nothing is asked. Refused, no time: with it not beside the player ("Snel is not beside you."),
  in the retinue too; a way with none ("No beast of yours stands that way."); a stack that fits no
  neck ("Snel has no use for that."); goods unpaid for. Its magic works on the beast as on any
  wearer, and wearing it on the beast teaches the player nothing. There is no taking one back but
  giving another or its death, when the collar is in its pile. Look mode says "Snel, your hound
  (unhurt), wearing a jade amulet, ...".
  **Sic 'em** (the ranger's level 40, #921; `use:sic:X,Y`,
  `world/sic.rs`, 5 Focus, the time of a wait, no cooldown): aimed at a creature the player sees
  within 10 cells that is hostile to them, it sets the companion on it ("Snel goes for the
  kobold's throat."; refused, no time, with no companion on the level or no such creature): a
  Hunt whose target is that creature before any nearer while the beast sees or smells it, its
  cell following it where the player sees that (a new order ends it), and **Savage** on the beast
  (3 degrees, half again as fast, a degree a minute): each blow that hits is a critical with no
  confirmation roll and spends a degree. It is neither a buff spell's status nor a potion's, so Hunter's bond's shared buffs never carry it. The ranger's companion design is complete (C1 to C6).
  **The familiar** (`warlock-core/src/world/familiar.rs`,
  `content/familiar.rs` and `data/familiars.toml`; #929, design `docs/design/witch-familiar.md`,
  `docs/dev/familiar.md`): the witch's small beast, an ally with `source =
  Familiar(CompanionData)`, `until = None`, at most one at a time; with a companion it is one of
  the player's **kept** beasts (`AllySource::kept`), and a witch may have both. A kept beast is
  named, grows, always comes along, wears a collar, leaves a corpse, and is shown by the rules above. Her
  **Familiar** (level 1) brings one as the run begins: drawn from her race's list
  (`[start.by_race]`: one to three beasts of level at most `most_level` 2: cat, toad, raven, hare,
  bat, and the small beasts the ranger's lists give a race where they fit a witch), or at
  `rare_per_mille` (50) one of the rares (a black hound, a panther), two draws on its own stream;
  named from `names` (the English witch trials' Tibb, Fancie, Pyewacket, …; one draw on its own
  stream); "Tibb, your cat, winds about your ankles." (`[arrives]`, a line a kind). **The touch**: a
  familiar's melee hit that leaves its target alive lays its master's hex (`touch`, Evil eye's
  Hexed) at the degrees she would cast it now, **with her as the giver**, so Cackle and Malison see
  it as hers, and starts every other hex of hers on the target afresh at its own degree (those that
  stack by refreshing). No mana, none of her turns, no marks, no draws, no message: a creature's
  statuses are silent. **The harrying**: its target is the nearest creature it sees under a hex of
  its master's, within its order's reach, before any other; else the nearest as any ally's. **Two
  kept beasts beside the player**, with something to treat on one: Heal asks a direction first (the
  wait key: the player); Give asks too, as the collar says; with one beside, nothing is asked. **Call familiar** (#930,
  `use:call_familiar`, a class active Familiar gives, the abilities menu's Class group, nothing to
  aim): `call_turns` (600: an hour) turns of the `call_familiar` action through `World::keep_at`,
  for nothing but the time: no mana, cooldown, count, or marks. Refused with no time with a familiar
  alive anywhere, the retinue too ("Your familiar lives yet."; the menu: "your familiar lives"), with
  a hostile the player sees ("Not with that watching."; "something is watching"), while a status
  blocks a cast, and without the data ("Nothing comes."). What would stop a rest breaks it off: the
  time is spent, nothing else lost. At the end the familiar she remembers comes back (#1470): the
  character keeps the kind and name of the first that came to her (`Character::familiar`, saved,
  hashed; set as it comes, kept after it dies), and a calling makes that kind with that name at the
  player's grade now, "Tibb, your cat, comes back to you out of the dark.", with no draw. Only a
  witch who never had one draws: two draws on the gameplay stream, the start's draw,
  with `bargain_kinds` (the imp) in the race's list from `bargain_level` (30) of the character's
  class level; "Something comes out of the dark and sits at your feet." and the kind's arrival, the
  name on the same stream as a start's, at the player's grade; that one is remembered from then on.
  A re-call has no chance of a rare kind or the imp. On the overworld in the retinue; with
  no free cell "Nothing comes." and the hour spent. **Familiar's sight** (the witch's level
  25, #932; passive): the shared sight of Hunter's bond with the familiar as its viewpoint, by the
  same rules, its nose reaching `nose` cells of `familiars.toml` (6) or its type's where that is
  farther (the type's alone still decides how the beast itself hunts); no buff is shared, and a
  companion she keeps lends her nothing. The companion chapter tells each beast of a race's list
  its own line (#931); a rare or a bargain's beast gets the chapter's plain line.
- **Animated objects** (`warlock-core/src/content/animated.rs`, the types; `animated.rs`, what
  one is made of; `world/animated.rs`, the drops and the talents; #483, design
  `docs/design/animated-objects.md`): things a will set moving, each an **ordinary creature
  type generated when content loads**. The household tier (levels 1 to 8) is one type per body
  in `data/animated/` (`animated_candlestick`, "animated candlestick"), at the body's level: its
  numbers are the undead roster's formulas (#383) with the tier's adjustments (small: two dice
  steps under the curve), its natural Protection, resistances, and death line ("The animated
  chair splinters.") by its object's base material's group, and the rest as
  `data/animated.toml`'s `[creature]` gives every animated type: faction `animated` (hostile to
  the player only), tags `animated` and `construct`, darkvision, no corpse, no blood, and
  **immune** to the diseased, poisoned, and bleeding statuses. A creature of it **is** its
  body: a real household object (class `household`) of a material drawn by the materials'
  `gear` weights at its level (a silver candlestick is rare), carried in no slot, rolled from
  `animated/<level id>` on a generated level (spawn order, after gear) and `animated/<actor id>`
  for every other spawn; a legend may give the material (`animated = { material = "silver" }`).
  It is seen by its body (the object's glyph in its material's colour; look mode adds "of
  silver" and how it moves). When it dies its body falls **whole** by `[drops] intact` (50 %),
  drawn from `drop/<actor id>`, else it **breaks into the parts it
  is made of** (§7 "Pieces and breaking": loose gems, pieces of its grouped materials by
  `[drops] pieces`, nothing of paper; its magic lost), with a line if the player sees it ("The
  silver candlestick breaks apart.", after the death line). Nothing can be
  stolen from one ("It is all of a piece.", no time). No talents of their own (Salvager,
  Unbinder, and Appraiser's eye were cut in #774). Three household bodies have a trick of
  their own (#1136, the dread design §13): the rug of smothering's blow may **engulf**
  (`{ status = "engulfed", chance = 15, difficulty = 12, damage = 1 }`: its grip 12, and a
  tick 1 hit point while it holds instead of the status's 2), the bellows' gust may
  **stagger** (20 %), and the animated lantern has **death throes** (a body's `burst`, as a
  creature's `[creature.burst]`: `1d6` fire, radius 1, "bursts in a spray of burning oil").
  The blades are **graded** (the tier's `grade_words`, "notched" and "keen"); no other
  animated type is.
  **Blades** (#485) are a **family**: a tier with `levels` and no bodies generates one type
  per level, `animated_blade_5` … `animated_blade_20`, all named "animated blade", numbered in a
  row after the household types; the family's id `animated_blade` names its first member. Its
  numbers are the quick role's: Dexterity +5, Toughness −2, hit points ×0.6, speed 1200 + 5L
  (at most 1600), natural Defence 2 + L/10 ("thin edge"), piercing 50, bludgeoning −25, no
  attack of its own, silent, uncommon, "falls in pieces". A blade **is** one melee weapon: its
  tier's slot line (`main_hand`, class `melee_weapons`, 100 %) draws a base item among the
  class's loot of level at most L by `frequency` (`loot::base_in`), then its grade, one draw:
  rare by the tier's `rare` (from L15, 1 % a level, at most 5 %), else at least magic by its
  `magic` (2 % a level, at most 40 %; a magic piece may come out rare by the rarity weights,
  as loot's "magic or better" does), else plain of a material by the `gear` weights; a magic
  or rare piece is made by `loot::make_as` at item level L. It is wielded in its slot (no
  burden check), from the same streams as a household body; its rank in the weapon's skill is
  `1 + 0.35 L` (at most 10; #1169: near `L/4` to hit), set as it is placed (`animated::weapon_skills`, no draws). It fights
  with the weapon as anyone does and is seen by it: the weapon's glyph in its material's colour
  (the tier's `shown_by` slot and `item_glyph`); look mode's gear line names it unidentified.
  A family is **one kind** wherever kinds are counted: one danger candidate (§7 "Danger
  levels"), one first kill ("You learn something of animated blades.", once), the kill tally
  and Favoured prey by the family's id, a kill quest naming `animated_blade` counts every
  member, a `tag:` objective and a rolled kill quest's pool name it once. Cealdhlāw and Fyrdelf
  list it.
  **Hollow armour and panoplies** (#486) are the composites, two more families:
  `animated_armour_12` … `_35` ("animated armour", the hollow role: Strength +5, Dexterity −2,
  Toughness +5, hit points ×1.2, speed 900 + 3L (at most 1100), piercing 25, uncommon) and
  `animated_panoply_20` … `_70` ("animated panoply", the standard role: Toughness +2, hit points
  ×1.2, speed 1000, +4 a level past 20 (at most 1200), rare). Both are medium, step loudly
  (footsteps 60), open doors, "collapse with a clang", and show as `A` in their body armour's
  material colour (the tier's `shown_by = "body"`; one colour until #463). A slot line may name
  `from` (the least level it is tried at) and `count` (how many times it is tried, each further
  piece going to a free slot of the same kind, `fits`: a ring for each hand); the lines are
  tried in order and a line whose slots are all taken is passed over with nothing drawn, so a
  panoply's shield never comes beside a two-handed weapon. Hollow armour wears body armour, a
  helm (70 %), a shield (40 %), gloves (30 %), and fights with its own blow (the tier's
  `attack`: its gauntlets slam, bludgeoning, the curve's dice at L); a panoply wears body armour
  and wields a melee weapon always, a shield (60 %) and a helm (75 %), from L30 gloves and boots
  (50 % each), from L40 a cloak (50 %), from L50 an amulet and two rings (30 % each), and holds
  `1 + 0.35 L` ranks (at most 10) in its weapon's skill and, beside a shield, `L/15` in the style of weapon and
  shield (Sword and Board, the tier's `shield_rank`). Pieces are at least magic by 10 + L % (at
  most 40, armour) and 20 + L % (at most 80, panoply), rare from L20. Everything they wear and
  wield works for them as for anyone (`Mods::worn`: stats, riders, resistances, light, ...),
  max mana and mana regeneration too (#525, §8 "Mana": a pool for a creature that casts or
  summons; a panoply does neither yet). Fyrdelf
  lists `animated_armour`; no branch population lists `animated_panoply` yet (none reaches
  danger 20): it stands in armouries (§7 "Special rooms", #487), whose sleepers are drawn four
  dangers deeper, so on Fyrdelf's bottom floor, and comes by a legend or the dev `spawn`.
  Their homes (#487) are two special rooms with populations of their own (§7 "Special rooms"):
  the **armoury** (blades, hollow armour, panoplies, among weapon racks) and the **workshop**
  (household things, among workbenches); the chromatic look of composites follows (#463).
- **Mimics** (`warlock-core/src/content/mimic.rs`, the types; `world/mimic.rs`, the rules;
  #616, `docs/design/chests-and-treasure.md` §5): a thing shaped as a chest. A **generated
  family** (`data/mimics.toml`, §7 "Data"), one creature type a level, `mimic_1` to `mimic_60`,
  numbered after the animated families and before the campaigns' types; a level's band (the last
  whose `from` it reaches) names it, ten bands matching the chest bands one to one: mimicling,
  young mimic, mimic, old mimic, greater mimic, elder mimic, hoary mimic, ancient mimic, primeval
  mimic, eldest mimic. Numbers by level from the roster's formulas for a brute (Toughness +5,
  `base_hp` 4 × L, the slam one dice step up), the hide's Protection (L − 10) ÷ 8 ("woody hide"),
  XP by the guideline; faction **vermin**, tag `mimic` (a kill quest's `mimic` or `tag:mimic`
  counts the family), speed 800, darkvision 6, `budge = "never"`, `stray = false` and in no
  population, immune to poison and sleep, a corpse that makes the eater sick. Its slam
  (bludgeoning) holds with its chest band's `hold` chance against a grip of 12 + L ÷ 5. It comes
  **only** through a chest: a generated chest is one with its band's `mimic` chance (§8
  "Chests"; the swap draws nothing more), of the floor's item level, carrying exactly what the
  chest would have held (it drops it whole when it dies); a legend's `mimic = true` beside
  `chest` (§7); the dev `spawn` as any type. Every mimic is placed **disguised**: the status
  `disguised` (`hides`), saved as any status. Disguised, it takes no turn (as a sleeper), is
  unaware of everyone (a sneak attack lands), no creature targets it, and the perception
  snapshot shows a shut chest of its band in its place and never it (not among the seen actors,
  `sees_actors_at` false there, its cell's terrain the shut chest's in view and in memory, look
  mode the chest's words); the spawn cap counts it. It is **found out** (`World::reveal_mimic`;
  the status removed, a line of its own when the cell is in sight, never again disguised) by a
  **touch** (a step into it, `open`, `kick`, or `pick` at it: the action refused with no time,
  "The coffer lurches: it is a young mimic!", and the mimic's free blow at the player at once,
  with its hold), by **Knock** at it (the same line, the mana spent, no blow), by **a blow or
  any harm** (an attack at it, after the sneak attack is decided; a missile stops at it; a burst
  or a trap: "The coffer shudders: ..."), and by **searching** (`Hidden::Mimic`, difficulty
  12 + L ÷ 5 plus the branch's `hidden_per_depth`; in passing only for those who notice
  everything: "You notice the coffer breathing: a young mimic!"). Stealing from one: "It has no
  pockets." Talents (system `chests`, which holds mimics since #773): **Chest-wise** (delver;
  Intelligence 11 or Detect traps 15; #775 merged Mimic-wise and Slick into it): within two
  cells a disguised mimic is shown as itself (the snapshot does not hide it; the rules still
  see it disguised), and a mimic's grip never holds them (`World::afflict`).
- **Natural Protection by body and level** (#563, `docs/design/combat-progression.md` §8): a
  creature type's `natural.protection` is set by its **body**, which the hide's `name` decides,
  and its level L (rounded down): none (insubstantial form, rushing wind, crackling form,
  flickering flame or form, flowing body, fey grace, mud, and no hide) 0; hide (hide and its
  adjectives, skin, fur, feathers, dead flesh, wrappings, rotting scales) (L − 10) ÷ 8, at least
  0; bony (bones, dry bones, bone, skull, scales, chitin, shell, carapace, plates) L ÷ 6;
  armoured (stone body, stony hide, iron-hard skin, frost-hard skin, ice, slag hide, cinder
  hide) L ÷ 4 + 1. A type of the peoples (tags kobold, goblin, hobgoblin, orc, drow, duergar,
  cultist, outlaw; the peaceful and merchants factions) or one whose gear may roll armour has
  none of its own (its race's: the armour supplies the rest; never both). Exceptions, kept in
  data: the cave beetle and ember beetle 3, the stone troll 5, the mud golem 5, the balor and the
  pit fiend 10 (bosses held to the bony line), a hag who casts half a hide's, every living troll
  at least 2 (the tribe's hide); the animated objects keep their own table. Data only, nothing
  saved; `content::creature_body_tests` checks every type. The player's answer is penetration
  (the war hammer, the stiletto, crossbow bolts) and criticals, soaked once after they multiply;
  **Armour-breaker** (Might, needs axes, maces, or polearms 7; the `protection` system): its
  confirmed criticals ignore 2 more Protection (`critical_penetration`), and Sunder and Gap shot
  cost it 2 stamina less (`weapon_stamina`, #564).
- **Grades** (`warlock-core/src/grade.rs`; #1135, `docs/design/monster-dread.md` §7.4): two of
  the same creature are two fights. As a creature is made (`travel::make_creature`, after its
  court) it draws its **grade** from its own stream `grade/<actor id>`: one in six is
  **greater** (+25 % maximum hit points, +1 to hit and +2 damage on its melee blows, 10 % less
  speed) and one in six **lesser** (the reverse: −25 %, −1, −2, 10 % more speed); the rest are
  neither. `Actor::grade` (−1, 0, +1) is saved and hashed, left out at 0. None for a type with
  `grades = false` (the people: the peaceful and merchants factions), a unique, a mimic, an
  animated type whose tier names no `grade_words` (all but the blades, "a notched animated
  blade", "a keen" one), or a unique character (a spawn with an `npc`); a kept
  beast (a companion or a familiar) reads as ungraded, its growth standing in. A kill pays its
  kind's experience. The player sees it in look mode as a word before the name, the type's
  `grade_words` (the kobolds' "a scrawny kobold", "a big kobold"), else its tags' (the undead
  "withered", "ancient"), else "small" and "big"; messages name the type alone. The `actors`
  dump says `; grade big`, the JSON a seen actor's `grade` (the word). No talent (a creature
  gimmick).
- **Variance by role** (#1134, `docs/design/monster-dread.md` §7.2; data only, the roster tests
  hold it, `docs/dev/creatures.md` "Variance by role"): two monsters of one level are not the
  same fight. A natural attack hits at `L/4` for the standard, `L/4 + L/8` for a striker (the
  quick and the assassins, at most +20), and `L/4 − L/8` for a brute, shambler, or caster (each
  part rounded down); an armed type keeps `L/4` on its natural attack, and its weapon aims by
  its weapon skill's ranks (and Duelist's, one-handed with no shield), never a flat bonus: the
  fewest ranks that bring the weapon to `L/4` (a caster's `L/4 − L/8`), none past what a
  trained warrior of its level holds, so a heavy weapon may stay below and hits harder instead
  (#1169). A striker's blow
  threatens from 19 (×2), a brute's crits ×3 on the natural 24 only. Hit points by role run 0.5
  (quick, small, caster) to 2.0 (brute) of the standard's, ±35 %. Speed is bimodal: the bulk
  between 800 and 1250 by the tribe's temperament (a people's rank-and-file at 950 to 1050, no
  tribe all at 1000), the dread at 600 or 700 or at 1400 and up. No talent (it extends the
  creatures).
- **The unseen** (`warlock-core/src/world/unseen.rs`; #1132, `docs/design/monster-dread.md`
  §5.10, §5.11): a **shadowy** type (`shadowy = true`: the shadow, the greater shadow, the
  deathshade, the darkling, the sluagh) on a cell **dim** to the player's eyes (perceived light
  under twice their profile's threshold, darkvision or not) is not in the player's perception
  unless it stands beside them; "You feel a cold draught." as one they do not see comes within
  2. An **invisible** one is not in it whatever the light: by its type (`invisible = true`: the
  **invisible stalker**, an air thing of level 22 that hunts the player by scent wherever they
  are on its level, `hunts = true`, and never idles), or by the **Invisible** status (two
  minutes, one degree, which nobody sees on anyone), which a type with `[creature.invisible]`
  (`cooldown = 300`: every duergar but the skeleton, the imp, the quasit, the will-o'-wisp) gives
  itself in the **self-buff step**: at the start of its turn, whenever it holds neither that
  nor the cooldown status (`invisibility`) and is not Silenced, free of time and mana, heard as
  chanting, "The duergar warrior fades from sight." where seen; then it goes on with its turn
  (patrolling, approaching, fighting). While invisible a creature sheds no light of its own (the
  wisp winks out). What shows them after all: **Seeing** (`seeing`, five minutes, the spell
  *See invisible* and the potion of seeing), Blood scent, the player's companion's nose, and a
  blow of the player's that hit (for that turn); their noises are heard where they are (the
  `sound` layer's `x` where an unseen blow came from, their footsteps, a heartbeat to
  Quick-sensed if they have one: elementals have none). Their blows are "something"'s; a move
  into an unseen creature's cell strikes at it at −4 ("You strike at something."). Creature
  sight is unchanged. No talent (a creature gimmick).
- **Creature gear** (`warlock-core/src/gear.rs`, the rolls; `content/gear.rs`, the data; #328,
  campaign design §10.3): a creature type may have a `racial_weapon` (a weapon it must have a
  rank in: the affinity is the rank, which combat applies) and `[[creature.gear]]` lines
  (`chance`, `one_of`, `keep`, `magic`; `data/creatures/README.toml`). The racial weapon is the
  first line, 90 % and wielded, unless a line names it alone. A creature made from the type
  without a fixed kit (a generated level's creatures; a level legend's or vault's entry with no
  `carry`, `wield`, or `wear`) rolls the lines in order: each by its chance, one of its
  alternatives, made magic by its `magic` chance (as loot is, at least magic, at the floor's
  item level, unidentified) or else plain, of a material by the materials' `gear` at the
  creature's level and its faction's favoured materials (§7 "Materials"); a piece that would
  overload the creature is left out, one with no free slot
  stays carried. It fights with what it wields and wears (combat reads it as it reads anyone's),
  a ranged creature shoots with its bow and arrows, and all of it falls where it dies with the
  rest of what it carried. Others see what it wears and wields, never what else it carries.
  Gear adds no state: it is the creature's inventory and equipment, and what falls is
  unidentified as any loot is. The wiki's creatures table states each type's gear exactly. A gear line may name a
  unique item (`unique:ID`, §8 "Magic items"): the Ascmearc's four wanted criminals (the
  `outlaws` faction, hostile to the player only and to no one else, with the footpads and
  footpad archers; #424, the guild design §8) each wield or wear their own this way.
  **Condition** (`world/gear_condition.rs`, `data/gear_condition.toml`; #1193): every rolled
  piece of arms or armour (not a stack of several, ammunition, rings, or a protected item: a
  unique, of Proof, or anything a quest may ask for, §7 "Materials") draws a condition by the
  creature's tribe (its first tag with a `[[tribe]]` row, else `[default]`; per mille): damaged
  (the smith's state, §7 "Special rooms": a point worse, its word by its material: iron and steel
  "rusted", other metals "corroded", wood, leather, and what withstands both plainly "damaged"),
  Fine, Marvellous, or Masterwork (improvements as the Temper ability makes them, each counted as
  an attempt), or as made. By default 30 % damaged, 4 % Fine, 0.8 % Marvellous, 0.2 % Masterwork;
  the undead's gear 95 % damaged; scavengers' (kobolds, goblins, orcs, trolls) about half;
  disciplined orders' (duergar, hobgoblins, drow, devils) 8 to 10 %, and Fine 10 to 15 %. From
  item level 3 on, the `[[depth]]` row of the level's item level adds its Fine, Marvellous, and
  Masterwork chances to these, damaged giving way (#1436, §8 "Magic items"). A fixed
  kit is never rolled and stays as made. The creature fights with the piece as it is, and it
  drops as it is, its condition in its name. No talent (a sub-feature of item condition).
- **Named monsters** (#424): a creature type whose `name` starts upper-case is a proper name
  ("Black Wystan"), said without an article wherever the game names a seen creature (combat,
  deaths, falls, waking, refusals, look mode: "Black Wystan (hostile, unhurt), ..."), as quest
  texts already do. Every other type's name is lower-case.
- **Dialogue** (`warlock-core/src/content/dialogue.rs`, the data; `world/talk.rs`, the rules;
  M28, #146): NPCs are creatures with a dialogue (§7), by their type or by the unique `npc` id
  their level gave them (which wins). A unique character with a proper name (the dialogue's
  `name`, #336) is called by it: `Sister Hild says: "…"`, and in look mode "Sister Hild (unhurt),
  who cures and mends for gold, on plank floor." (the services it sells, in words).
  - **Chatting** (`Action::Chat(dir)`, replay text `chat:E`; §6 "Chat") with a creature the player
    sees on that cell takes the time of `chat`. One without a dialogue does not answer ("The
    giant rat doesn't answer."), one hostile to the player is in no mood ("The brigand is in no
    mood to talk."); either still takes the time. Otherwise it answers (`The old hermit says:
    "…"`): its `greet` the first time the player chats with it (it is then *met*), else its next
    one-liner.
  - **Rotation**: the one-liners come in rounds, each line once a round, every round in an order
    of the speaker's own (a Fisher–Yates shuffle drawn from FNV-1a and SplitMix64's finalizer
    over the run seed, the actor, and the round), and a round never starts with the line the last
    one ended on, so the same line never comes twice in a row. Each actor counts the lines it has
    said (`Actor::talk`, saved and hashed once it has said something). **Lines after a quest**
    (`[[dialogue.after]]`, #1446): once the player has done quest `quest`, its `lines` join the
    speaker's one-liners (the rotation runs over both), so Kerensa of Gimmwīc speaks of the bound
    crystal guardian.
  - **Topics**: the answer is an event (`Event::Talked`: who, what they said, the topics the
    player may ask about, and the topic it answers); one with topics opens the dialogue screen
    (§6). Asking (`Action::Ask { dir, topic }`, replay text `ask:E:rooms`) takes no time and gets
    the topic's reply; an unknown topic is "The sage knows nothing about that." `World::topics`
    is the one place topics come from: quests ("Quests" below) add theirs there, first.
  - **Callings** (`world/calling.rs`, `World::ask_calling`; #1174): a topic may `calls` a
    prestige class's calling (§8 "Prestige classes", one of the dialogue's campaign). It is
    offered (`World::topics`) only while the calling is open to the player; asked, the topic's
    reply is said, the calling made, and, with `departs`, the speaker sent away
    (`World::departs`, as an ally whose time is up: no corpse, no pile, no experience, no kill;
    the reply is a message only, and the screen does not reopen). It takes the time of `chat`.
    Asked while not open (by replay text), it is unknown ("The sage knows nothing about
    that."), and takes none. The Ascmearc's **Strange Spirit** (`strange_spirit`, faction
    `spirits`, hostile to no one; good; one hit point, no corpse, no blood, no experience;
    rooted, `budge = never`) is met on the roads, vale, downs, and woods by day (`when = {
    calling = "hunter" }`, weight 2), speaks only to the Kind or better (`speaks_to`), and has
    one topic, *Life before death* (`life_before_death`, `calls = "hunter"`, `departs`): it
    echoes the oath, calls the bard into the Hunter, and departs ("The strange spirit fades,
    and is gone."). Killed, it is a peaceful good soul killed; its party is forgotten and it
    may be met again while the calling is open. Look mode says ", who waits" after one whose
    dialogue's only topics are callings', with no service or wares. A dialogue's `speaks_to`
    (an alignment range, §8 "Alignment") is the speaker's reaction to the player: outside it a
    chat or an ask gets only its `silent` line (a chat's time; an ask, none), with no topics.
  - **Rumoured ways** (`world/rumour.rs`; the owner's request of 2026-10-27): a topic with a
    `route` (§7 "Dialogue") tells the way to a place. Asking it, every time, marks on the
    player's overworld map what walking the route by day would show: each cell in line of sight
    within the day's sight radius (9) of a cell of the route goes into the overworld's memory as
    the terrain its cell holds, so a corridor of country and the land about the route's end,
    not a whole region. The overworld is made first if the run has not been on it yet. Only the
    terrain grid is read, and a hidden location's cell holds its plain terrain (§7 "Hidden
    locations"), so a rumour never marks, names, or reveals a hidden location (quests stay the
    only key); a visible location along the way (Cealdhlāw) is remembered as seeing it would
    show it. After the reply, "You mark the way on your map." if any cell was new to the map;
    asking again marks nothing more. No time beyond the ask's (none), no stream drawn, no state
    beyond the overworld's memory (saved as always). Built in: Godgifu's five topics in Cēaptūn
    (the barrows, the marsh, the mines, the volcano, the pass), each from the town's cell.
  - **Services** (`world/service.rs`; the campaign design §6.1, #329): a dialogue's
    `[[dialogue.service]]`s (`id`, `name`, `price` in gold, `does` = `cure`, `mend`, `identify`, or
    `train_skill`, `reply`, `nothing`, and a training's `train`) are sold for gold and asked for like topics (`ask:E:cure`): `World::topics` lists
    them after the topics with their price now (`TopicView::price`; the dialogue screen shows
    "A cure for what ails you (60 gold)"). Refused, taking no time and nothing taken, with the
    `nothing` line when there is nothing to do (a cure: no status or disease, incubating ones
    included, lists `prayer` in its `cured_by`; a mending: no stat damage), with "Be cured
    first." for a mending while a disease shows, and with "You cannot pay." without the gold.
    Otherwise the price moves from the player's pack into the speaker's in the shops' currency
    (`hand_over`, the purse code `pay` uses), `cure` ends what `prayer` cures (`World::cure`;
    every disease lists it) and `mend` clears every point of stat damage (`World::mend`), the
    speaker says the `reply`, and it takes the time of `chat`. The price is the data's, never
    moved by favour; the Pious talent halves it (rounded up) and has a cure also mend a point
    of each damaged stat (`TalentMods::services`). No alignment shift, no stream drawn, no new
    state.
  - **Identification** (`does = "identify"`, #432; Factor Wulfrun in Cēaptūn, nobody in the
    showcase): identifies, in one go, every inventory stack the player does not fully know
    (`World::unidentified`: magic not yet identified, or a scroll of a type not yet identified),
    through `World::identify`, each saying "This is …!". Its `price` is **per such stack**, not
    per piece (100 gold at Wulfrun's: three unknown stacks cost 300), the count in
    `TopicView::items`; the dialogue screen lists "(100 gold each)" and, with something to
    identify, asks first in the one dialog with the total (`Pay 300 gold` or `Not now`; without
    the gold only `Go back`). All or nothing: refused with the `nothing` line when everything is
    known and with "You cannot pay." short of the total, neither taking time. Pious does not
    halve it (a temple's talent).
  - **Training** (`does = "train_skill"`, #1391): `train = { skill = "mining", value = 5 }` names
    a skill (`data/skills.toml`, checked when content loads) and the value it is taught at: a
    fixed number, no roll (5, the low end of a dwarf's Mining). Offered only while the player is
    untrained in that skill: `World::topics` leaves it out once they are, and asked for anyway it
    is the `nothing` line, no time. Bought, the player is trained at that value and told "Alberich
    trains you in mining." Its price moves with favour as a healer's or a sage's does (not
    `temple`; Pious does not halve it), and with a town's goodwill. Built in: Alberich, the
    dwarven miser of Fyrdelf's fifth floor, teaches Mining for 500 gold (§7 "Dungeons"); digging
    needs it (§8 "Digging"). Eadric, Cēaptūn's smith and the keeper of its smithy (#1394; the
    level's `[[shop]]` names him `npc = "eadric"`, §7 "Shops"), teaches Metalcraft for 2000 gold,
    at 5, one of the town's people, so its goodwill takes its share off; every recipe needs
    Metalcraft (§8 "Professions").
  - **Goodwill** (`world/goodwill.rs`, `content/town.rs`; #1247): a campaign's `data/towns/`
    `[[town]]`s (`id`, `name`, `people`: npc or creature ids, `goodwill`: percent, `told`) name a
    town's people. Every quest one of them gives that the player completes (each completion of a
    repeatable one) adds one to the town's goodwill, derived from the quest records (nothing new
    saved), and the town's `told` line is said after "Quest complete" ("Cēaptūn's folk think the
    better of you."). Its people's services cost `goodwill × count` percent less, at most 100%,
    taken off the price after Pious and favour and rounded up, so only 100% off makes one free;
    a free service is listed "(free)", asks no pay dialog, moves no gold, says no "You pay", and
    trains nothing. Shop goods and offers keep their prices. Built in: Cēaptūn, 5% a quest for
    Sister Hild's, Reeve Leofric's, Factor Wulfrun's, and Wat One-Eye's (the Gilde's) quests:
    twenty make its services free (Eadric the smith's teaching among them, though he gives no
    quest).
  - **An offer** (`world/wares.rs`; #808, docs/design/overworld-encounters.md §15 Q7: the
    pedlar): a dialogue's `[dialogue.offer]` is one item at a fixed price, take it or leave it,
    asked for like a topic (`ask:E:wares`) and listed after the services. It is made the first
    time the player chats with or asks the trader, from the stream `wares/<actor id>`: by
    `uniques.chance` (basis points) one of the offer's uniques not yet offered in the run, else
    a **favoured** roll (`magic::loot::favoured`: a rare item of any base that can be magic but
    ammunition, with `affixes` of them by weight, at the player's level plus `level.bonus`, at
    least `level.least`), identified. Its price is fixed (no favour): `price.unique` for a
    unique, else its value plus `price.affix` gold per affix per item level. The dialogue screen
    asks in the one dialog with the item in words and the price ("Buy it" / "Leave it"; short of
    gold, "Leave it" alone); leaving it is nothing. Buying moves the gold to the trader, the item
    into the pack, counts as a deal (Negotiation's and Presence's marks), and takes the time of
    `chat`; refused, with no time, with "You cannot pay." Once sold the trader says `sold`: one
    offer per trader (the same actor met again keeps his). Saved: `WorldState::wares` and
    `uniques_offered`, left out while empty. The pedlar: four to six affixes (50/35/15), level
    + 4 (at least 4), 15 gold per affix per level, 2 % one of his three uniques at 1500 gold.
  - **Muttering**: after each of its actions, a creature whose dialogue has a `mutter`, that goes
    about its routine (its mind calm) within 5 cells of the player (as far as a voice carries), mutters
    with its `chance` (a hash of the run seed, the actor, and the moment: the gameplay stream is
    untouched) one of its muttered lines, in rounds like the one-liners. It is a noise, a voice
    (50; §5 "Hearing"), and the player gets the words only if they hear it: with the speaker's
    name if they see it (`The bard sings: "Over the Wendle and under the moon..."`), else from
    where they place the sound (`You hear a voice to the north: "…"`, or, faint, `You faintly
    hear a voice to the north.`, without the words). Mutters are logged without stopping a rest.
- **Quests** (`warlock-core/src/content/quest.rs`, the data; `world/quest.rs`, the rules; M28,
  #148): simple, ADOM-like errands people give through their dialogue topics. No timers or
  branching outcomes; escorts are the one quest that walks with the player (below).
  - **Topics** (first among a speaker's, in file order): a quest a giver may offer (what it
    requires done; new to the player, or done and `repeatable`; a rolled one while its pool holds
    something) is a topic named after it, `quest-ID` (replay text `ask:E:quest-rats`). **Asking
    gives it** (#422; the owner: a roguelike, not an MMO, so there is no accepting and no
    declining): the giver says the `offer`, then its `accept` line (if any) in the same reply, a
    rolled quest draws its target (below), and the quest is active at once, on the log with its
    first breadcrumb; a delivery's giver hands the item over ("The wandering bard gives you a
    sealed letter."; the pack has room for anything, §6 "Item lists"). The topics `accept-ID`
    and `decline-ID` of older replays are no one's and refused. A quest with a `min_level` (a
    character level, 1 to 100) is a topic before the player reaches it, but asked about then the
    giver says its `early` line (in words, never the number: "come and see me when you are
    rather more seasoned") and gives nothing, reveals nothing, and takes no time. An
    active quest stays a topic of its giver, who says the `reminder` until its goal is met; it is
    also a topic of whoever it is handed in to (the recipient of a delivery or of a bring with
    `to`) while its goal is met. Asking the one it is handed in to once the goal is met completes
    it: what it asks for leaves the pack ("You give the gardener 3 apples."; stacks not worn or
    wielded first), they say `done`, "Quest complete: Three apples.", the reward goes into the
    pack ("The gardener gives you 10 gold coins.", what has no room lies at the player's feet),
    and the XP is gained ("You gain 15 experience.", with any levels it earns, §8 "Experience").
    A reward may also teach skills (`skills`, by skill id; #1329): a skill the player is untrained
    in is trained at its `start` roll (drawn on the `levels` stream, never the gameplay one), a
    trained one gets `marks` training marks instead, each as a mark earned by use (§8 "Skills");
    told in words, never numbers ("Sister Hild shows you how to dress a wound.", "You learn more
    of healing."). Sister Hild's Grave goods teaches Heal: `1d4`, or 100 marks.
    Topics the speaker does not have now are refused ("… knows nothing about that.") and change
    nothing. Talk about quests takes no time.
  - **Goals**: *kill* counts the player's own kills (§8 "Experience": melee, shots, throws, and a
    wound of theirs that bleeds it to death; never by others or traps) of the creature type, or
    of any type carrying a **creature tag** (below), on its level or any level of its branch if it names
    one, else on every level summed. Kills are tallied per level and creature type, every type (saved;
    the ranger's Favoured prey reads the same tally, §8 "Class abilities"), so kills made before an
    authored quest was given count (a rolled quest's authored variant too), and a quest rolled
    by a `pick` or a repeat of a repeatable one counts only the kills since it was given. A kill of a **named monster** (a `unique` type; `count` 1) is
    met by its death, at any hand (the player's, a trap's, another creature's) and whenever it
    died, before the quest was given too (the guild design §8.4: no quest can be softlocked):
    every death passes `creature_dies`, which keeps the dead unique types (`Quests::slain`,
    saved). This is the one place the log may know what the player did not see. *Bring* counts what the player carries of
    the object. *Reach* is met once the player has been on the level, before taking the quest
    up too. *Deliver* is met while the player carries the item: dropping it (or losing it) blocks
    the quest until they pick it up again (it cannot be handed in without it).
  - **Escort quests** (`world/escort.rs`, #1139): `type = "escort"`, `target` a level, `escort`
    a unique character (an npc with a dialogue), `joins` their line. Chatted with while the quest
    is active, the escort says `joins` instead of a greeting and becomes the player's ally
    (`AllySource::Escort`: on the player's side, following, across levels and in the retinue as
    any ally) in the **coward** stance. It is no fighter: it strikes only at a hostile beside
    it, and Set on passes it by ("Eadgyth will not go looking for a fight."); Come to heel and
    Hold here reach it. The Give ability hands it anything that fits one of its slots (a free
    one first, else the first it fits; both hands for a two-handed thing): it puts it on at once
    ("Eadgyth wields the long sword.") and what the slot held falls on its cell ("Eadgyth drops
    the club."); what fits no slot is refused with no time. When the escort stands on the target
    level with the player (checked as they arrive, or as it joins there), the quest is done: the
    escort says `thanks` (optional), the giver, if on that level (else the escort), says `done`,
    the reward is paid as a hand-in's, and it is no one's ally (its own
    faction, the neutral stance, its post where it stands). Its death, joined or not, fails the
    quest ("Quest failed: Eadgyth."): the log keeps it among the finished ones, "Failed: Eadgyth
    is dead.", and it is never offered again; nothing else is locked. The log's status while
    active: "Eadgyth follows you." or "Eadgyth is not with you." An escort no level can place
    (a generated floor: a vault holds no npc) is placed by the quest, `waits = { creature,
    level, at }` (`world/escort_waits.rs`, #1141): a creature of that type, the escort, on the
    named place `at` of that level (or the nearest free cell), as the quest is given or as the
    level is first made after; once only. The Ascmearc's first: Gammer Wynflæd's "Taken under
    the hill", Ælfrun on the moss bed at the bottom of Ælfhām, home to Holtwīc.
  - **Lure quests** (`world/lure.rs`, #1446): `type = "lure"`, `target` a unique creature type,
    `level` a hand-made level, `waits = { level, at }` its place, and `[quest.binding]`. While the
    quest is active and no creature of the type is anywhere, one is made on its place as the
    quest is given (if that level exists) and whenever the player arrives on that level: killed
    before it is bound, it forms anew there ("Somewhere on this floor, crystal rings as it grows
    into a guardian again."), so the quest never fails and may be tried again. It is hostile,
    and comes along only by the stairs' rule (§8 "Stair followers": beside the player as they
    take a staircase), so the player leads it up floor by floor; left behind, it stays where it
    was. Once it stands on `level` with the player (they arrive by the stairs) and the giver is
    there too, the giver binds it: the binding's `words` are told, it joins the binding's
    `faction` (at peace), calms, takes the binding's `post` as its own (it walks there and
    stays), and carries the binding's `status` for good (one that never eases, shown by its
    spell circle, §4.8); the giver says `done` and pays. Its `boon` adds to the player's checks of
    one skill's recipes on its level while it stands there bound. The log's status while active:
    "The crystal guardian is not yet bound in Gimmwīc." The Ascmearc's one: Ysella the Binder's
    "The crystal guardian": asked, she opens the crystal portal beside her in Gimmwīc's garden (a
    hidden location of the town, §7 "Hidden locations"), into Meyn Bew, a pocket of the plane of
    earth, four floors of danger 22 to 25; the guardian (a slow, crystalline earth elemental of
    level 26) waits in the crystal hollow on the fourth; bound, it stands in the corner of
    Kerensa's Stones (the `bound` status, an enchantment circle), Kerensa and Ysella speak of it
    (§8 "Dialogue": lines after a quest), and Jewelcraft's Cut and Set checks made in Gimmwīc get
    +2. The reward: 1200 gold, 4000 experience, and an uncut sapphire.
  - **Creature tags** (`content/tag.rs`, #421, #441): a creature type carries plain singular
    words, chosen by hand as a card game gives creatures types (`tags = ["goblin", "archer"]`);
    there is one flat pool and no registry: a tag exists because a creature carries it. A kill
    quest's target may be a tag (`target = "tag:goblin"`): any type that carries it counts.
    Membership is by type: an actor's faction override (an outraged keeper) moves it neither in
    nor out. The noun is the tag's plural by the plural dictionary (§7 "Plurals"; hyphens said as
    spaces, `marsh-thing` "marsh things"), or the quest's `what`; the objective names the tagged
    types a run of the quest's campaign can meet: "Kill 100 goblins: goblins, goblin archers,
    and goblin chiefs."; progress says the same noun ("Quest updated: 37 / 100 goblins.").
    Group tags name a tribe or family (`goblin`, `undead`, ...), role and kind tags a role
    (`archer`, `caster`, `skeleton`, `spirit`, ...); the list and what each covers is the tag
    list in `assets/data/creatures/README.toml`, and the wiki lists the tags with their
    creatures. Every creature of the `undead` faction carries `undead` (a test sees to it), and
    every `combat = "caster"` type `caster`. Tags are never
    shown in the game; the wiki lists them.
  - **Guilds** (`content/guild.rs`, the data; `world/guild.rs`, the ranks; #425, the guild
    design §7.6): a guild (`data/guilds/`, a campaign's own) has a name, a master (who speaks
    for it), and ranks, lowest first, each a title named by a material and reached as a quest
    is *given* or *done*. The player's rank is the highest reached, **derived** from the quest
    records (never saved), and a title only: no rule reads it. As a quest is given or handed in
    and a rank rises, the master names the player to it, once ("Wat One-Eye names you a Tin of
    the Weyfarers' Gilde."). The character screen shows it (§6), and the `stats` dump (`Guild
    Weyfarers' Gilde, Tin`). The Ascmearc's is the Weyfarers' Gilde (§3b "Quests").
  - **Progress** the player's action made is said right after it: "Quest updated: 2 / 3 giant
    rats.", and once the goal is met "Quest updated: 3 / 3 giant rats. Return to the old hermit."
    (bring: "… apples. Return to the gardener." or "Take them to …"; reach: "Quest updated: you
    have reached the Depths 3. Return to the old hermit."; a delivery's item picked up again:
    "Quest updated: take the sealed letter to the old hermit."); a quest whose goal is met as it
    is given says so after the giver's words. Rewards and the XP gained are the only numbers said.
  - **Rolled quests** (`content/quest_roll.rs`, `world/quest_roll.rs`; #422, the guild design
    §5): a kill or bring quest may draw its target as it is given instead of naming it: a
    `pick` (`creatures` of branches' populations, no uniques, of the branches the player can
    enter; `tags`, one with a member in such a branch of the quest's campaign; `objects` of an
    item class, with `loot` only those loot holds) or authored `[[quest.variant]]`s (a target,
    a `weight`, and its own `reveals`, `where`, and texts). A branch can be entered unless a
    hidden location of the run's overworld leads into it unrevealed (§7 "Hidden locations"): a
    roll never names a place the player cannot go. The draw uses the keyed stream
    `quest/<id>` once, as the quest is given (never kept; inputs: the content and the locations
    revealed, so a replay draws the same), over the pool in content order, uniformly or by
    weight. What it drew, the **roll** (target id, variant id, count), is saved in the quest's
    record: **no rerolls**, it stands until the quest is done. The texts (all but `name`) say
    the roll through placeholders: `{target}`, `{targets}`, `{a_target}` (a named monster's
    proper name alone, "the fire drake" for a lower-case one), `{count}`, `{members}` (a tag's
    types), `{branch}` (the first branch, in the pick's order, it lives in), `{where}`, each
    also capitalised (`{A_target}`); every text, filled for every target the pool can hold, is
    checked at load (no unknown or unclosed placeholder). A variant's
    `reveals` (`when = "given"`) marks its own location only. **Beast-friend** (#1189): a
    player with the talent (§8 "Talents") is never asked to kill what it calms: the pool, as the
    quest is given, leaves out a creature type of an `animal` faction, a tag any of whose types
    of the quest's campaign is one, and a variant asking for either (read by the talent, not the
    class, at the giving; the saved roll keeps it). A variant with `fallback = true` is drawn
    only when no other variant is left: Reeve Leofric's "Rats in the barrows" asks for giant
    rats (`vermin`, an animal faction), and a Beast-friend draws its fallback, five skeletal
    rats (undead), with his own words. Dev: `quest_roll:ID:TARGET`
    (§9) replaces an active rolled quest's roll while it has no progress (a target its pool
    can hold wherever the player can go, or a variant id), its kills counted from then and a
    variant's reveal made; no time.
  - **Reveals** (#327; §7 "Hidden locations"): a quest with `reveals` marks its hidden location
    on the player's map as it is given (`when = "given"`, the default) or done, right after the
    speaker's words: "The old hermit marks the Old Barrow on your map." A location revealed
    before is not marked again, and said nothing of. A quest's `where` follows its objective in
    the log.
  - **The log's first breadcrumb**: the objective reads as the giver's wish, for every quest:
    "Sister Hild wants you to reach Fennmynster 3. The minster lies in Micelfenn, east of
    Fennhām.", "The gardener wants you to bring 3 apples." (a bring to the giver names no
    recipient), "Godgifu wants you to kill 100 kobolds: kobolds, kobold archers, and kobold
    shamans.", then the progress line, then "Return to …".
  - **No abandoning, no cap** (the owner's decision): a quest given stays in the log until it
    is done, and the player may have any number of them at once.
  - **State**: per quest the player has heard of, its stage (active, done, or failed; an `offered`
    record of an older save, an offer never answered, loads as active), how many times it was
    completed, the kills counted before a rolled quest or a repeat, whether a reach quest's level
    was reached, and a rolled quest's roll; the kill tally; and the unique types slain. Saved and
    hashed, each left out while empty, so runs without quests hash as before. Presentation reads
    `PerceptionView::quests()` (the log: name, giver, the offer, objective, the rolled target,
    status), never ground truth: the player knows their quests and what they were sent for.
- **Displacement** (`world/talk.rs`; M28, #147): the player moving into a creature that is not
  hostile to them (neutral or peaceful) swaps places with it: the player steps onto its cell (the
  step's time and noise) and it onto the player's (its footsteps), its own turn unaffected; "You
  swap places with the deer." ("…with something." if unseen). It lands there as any step lands:
  ice there may give way under its weight (§5 "Ice"), and a trap there goes off at it (§8
  "Traps"); what that costs it (climbing out of a pit or onto the ice) comes off its next action. It remembers the cell it stood on
  as its post (unless it already had one), walks back there once the player has moved off it, and
  carries on with its routine ("Creature behavior", Return). It refuses, and the move takes no
  time, when its type's `budge` is `never`, or `off_post` while its mind is calm (keeping its
  post: shopkeepers, #145); when it cannot move (overloaded): "The sentry doesn't budge."; or
  when it is two sizes or more larger than the player (sizes in `data/sizes.toml` order; none of
  the built-in sizes is): "The giant is too big to squeeze past." A refusal the player does not
  see the creature for is "Something is in the way."
  **Wriggling past** (`world/wriggle.rs`, #1032): a player trained in Escape artist who is refused
  because the creature keeps its post, never gives way, or is `off_post` (not for a creature that
  cannot move, one too big, or while swimming) is not refused: no action, no ability row, no key,
  the step itself is a check (as the check against bonds is, §8 "Status effects"). Their Dexterity
  modifier, what talents add to Dexterity's checks, and Escape artist's `escape` points, against
  11 plus the creature's Willpower modifier (and its talents'); two gameplay draws. Made, "You
  wriggle past the shopkeeper." and the step goes ahead as a swap; failed, "You try to wriggle
  past the shopkeeper, but it is in the way.", the step's price, the player stays. Every try marks
  Escape artist (`wriggle`, 50 %). A hostile creature is still struck, not wriggled past, and a shop's keeper is immune (the owner:
  the refusal stands, with no draw, mark, or time; §7 "Shops").

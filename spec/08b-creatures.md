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
     all of this whenever it may. A **lurker** (a type with `lurks = true`: the meenlock, the
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
  and kill (a shot is a missile, "Ranged combat" below); a creature's spells stay intents (the
  player's go through the one cast path, §8 "Spells"; creatures do not cast through it yet). An attack is an event (`Event::Attacked { attacker, target, kind: Melee | Ranged |
  Thrown | Spell, from, to, strike, with }`, `strike` what came of it, `None` for an intent,
  `with` the missile of a shot or throw), a noise (§5), and a message when the player perceives
  it (§6). The event reaches presentation only
  when the player sees both ends (or made the attack and saw the target). The player moving into
  a hostile creature is a melee attack; into any other one it swaps places with it
  ("Displacement" below). Creatures neither flee nor
  fight differently when hurt. What a creature has in mind (last seen, noise to look into, post)
  is part of the world state.
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
  window, uniques and itself left out; a shared type calls only shared types), `count` (dice, 1
  to 12), `cooldown` (6 to 600 s), `mana` (its cost; the type must have mana), and optionally
  `range` (default 8), `spread` (default 2), `most` (default 6), and `verb` (default
  "summons"). As step 2 begins, a creature with one summons when it sees a hostile within
  `range`, its casting is not blocked (a status that blocks `cast`: Silenced), its cooldown
  status `summoning` is off, it has the mana (`World::spend_mana`, its cut included), fewer than
  `most` of those it summoned live (on any level), and a free cell lies within `spread`; deciding
  draws nothing. It then pays the mana, takes `ceil(cooldown / eases)` degrees of `summoning`
  (the house rule: an ability's limit is a cooldown status), makes a spell's noise, and rolls
  `count` on the gameplay stream, capped by the room `most` leaves; each creature is one
  `below(pool.len())` draw, made like a spawn (gear rolled from `gear/<actor id>`) on the next
  free cell: within `spread` (Chebyshev) in its line of sight, walkable, no one there, not a
  sanctuary it would shun, nearest first (Chebyshev distance, squared distance, row order). Fewer
  cells, fewer come. The summoned take the summoner's faction (a `faction_override` when it is
  not their type's) and remember who called them (`Actor::summoner`, saved, left out while
  `None`); they act from their next turn, and nothing else sets them apart: they stay when the
  summoner dies, and their kills pay full experience. The action costs a `cast`. Words: the
  player who sees the summoner is told what they see come ("The goblin wizard calls forth 2
  goblins and a warg!", "something" when they see none of it); an unseen summoner's creatures
  only as they appear in sight ("A skeleton appears!"); nothing seen, nothing said. Sight is the
  last turn's perception, so a summoning in the turn a door opens may go unseen.
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
  credited in full ("Credit is still shared fully for quests"); the ally gets nothing. Handle animal earns its marks from an order given and an ally's kill. At most
  `[allies] most` (8) allies on one level. **Beast-bond** (Wayfarer; Handle animal 20): your
  allies strike +1 to hit and take Protection +1, read through you. The messages never say a
  number. The dev `ally:CREATURE[:N[:DURATION]]` makes allies for tests and captures.
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
  player's for good, `source = Companion(CompanionData { name, grade })`, `until = None`; at most
  one at a time. The ranger's **Animal companion** (level 1) brings one as the run begins: a
  beast drawn from the character's race's list (`[start.by_race]`: a human's hound or boar, an
  elf's lynx or hound, a goblin's large spider, rat, or bat, …; one to three a race, of level at
  most `most_level` 2), or at `rare_per_mille` (50) one of the rares shared by every race (a
  wolf, a black hound, a great boar, a panther), on its own stream (two draws); named from
  `names` (Old English: Snel, Brūn, Græg, …; one draw on its own stream); "Snel, your hound, pads
  at your heel." A character with a starting beast is told a line of it in the story's
  **companion** chapter (§8 "Background story"). Only `tameable` creature types may be
  companions (a beast: never a unique, an animated object, a summoner, one that opens doors, one
  of the peoples, one that fights from afar, or one that carries gear); a type's `nose` (1 to
  20 cells) is how far it smells. **Growth**: the companion's `grade` is its master's level
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
  and the growth table exactly. **Hunter's bond** (the ranger's level 25, #920; passive):
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
  crosses, and the shared one does not cross back. "Snel is bark-skinned too." Taming (C2) and the
  collar (C3) are the design's next slices.
  **The familiar** (`warlock-core/src/world/familiar.rs`,
  `content/familiar.rs` and `data/familiars.toml`; #929, design `docs/design/witch-familiar.md`,
  `docs/dev/familiar.md`): the witch's small beast, an ally with `source =
  Familiar(CompanionData)`, `until = None`, at most one at a time; with a companion it is one of
  the player's **kept** beasts (`AllySource::kept`), and a witch may have both. A kept beast is
  named, grows, always comes along, leaves a corpse, and is shown by the rules above. Her
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
  wait key: the player); with one beside, nothing is asked. **Call familiar** (#930,
  `use:call_familiar`, a class active Familiar gives, the abilities menu's Class group, nothing to
  aim): `call_turns` (600: an hour) turns of the `call_familiar` action through `World::keep_at`,
  for nothing but the time: no mana, cooldown, count, or marks. Refused with no time with a familiar
  alive anywhere, the retinue too ("Your familiar lives yet."; the menu: "your familiar lives"), with
  a hostile the player sees ("Not with that watching."; "something is watching"), while a status
  blocks a cast, and without the data ("Nothing comes."). What would stop a rest breaks it off: the
  time is spent, nothing else lost. At the end two draws on the gameplay stream, the start's draw,
  with `bargain_kinds` (the imp) in the race's list from `bargain_level` (30) of the character's
  class level; "Something comes out of the dark and sits at your feet." and the kind's arrival, the
  name on the same stream as a start's, at the player's grade; on the overworld in the retinue; with
  no free cell "Nothing comes." and the hour spent. The familiar's story lines (F3, #931; until then
  the companion chapter's plain line) are the design's next slice.
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
  Unbinder, and Appraiser's eye were cut in #774).
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
  `1 + L/10` (at most 8), set as it is placed (`animated::weapon_skills`, no draws). It fights
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
  `1 + L/10` ranks in its weapon's skill and, beside a shield, `L/15` in the style of weapon and
  shield (Sword and Board, the tier's `shield_rank`). Pieces are at least magic by 10 + L % (at
  most 40, armour) and 20 + L % (at most 80, panoply), rare from L20. Everything they wear and
  wield works for them as for anyone (`Mods::worn`: stats, riders, resistances, light, ...),
  max mana, mana regeneration, and caster level too (#525, §8 "Mana": a pool and a caster
  level for a creature that casts or summons; a panoply does neither yet). Fyrdelf
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
- **Creature gear** (`warlock-core/src/gear.rs`, the rolls; `content/gear.rs`, the data; #328,
  campaign design §10.3): a creature type may have a `racial_weapon` (a weapon it must have a
  rank in: the affinity is the rank, which combat applies) and `[[creature.gear]]` lines
  (`chance`, `one_of`, `keep`, `magic`; `data/creatures/README.toml`). The racial weapon is the
  first line, 90 % and wielded, unless a line names it alone. A creature made from the type
  without a fixed kit (a generated level's creatures; a level legend's or vault's entry with no
  `carry`, `wield`, or `wear`) rolls the lines in order: each by its chance, one of its
  alternatives, made magic by its `magic` chance (as loot is, at least magic, at the floor's
  item level, unidentified) or else plain, of a material by the materials' `gear` at the
  creature's level; a piece that would overload the creature is left out, one with no free slot
  stays carried. It fights with what it wields and wears (combat reads it as it reads anyone's),
  a ranged creature shoots with its bow and arrows, and all of it falls where it dies with the
  rest of what it carried. Others see what it wears and wields, never what else it carries.
  Gear adds no state: it is the creature's inventory and equipment, and what falls is
  unidentified as any loot is. The wiki's creatures table states each type's gear exactly. A gear line may name a
  unique item (`unique:ID`, §8 "Magic items"): the Ascmearc's four wanted criminals (the
  `outlaws` faction, hostile to the player only and to no one else, with the footpads and
  footpad archers; #424, the guild design §8) each wield or wear their own this way.
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
    said (`Actor::talk`, saved and hashed once it has said something).
  - **Topics**: the answer is an event (`Event::Talked`: who, what they said, the topics the
    player may ask about, and the topic it answers); one with topics opens the dialogue screen
    (§6). Asking (`Action::Ask { dir, topic }`, replay text `ask:E:rooms`) takes no time and gets
    the topic's reply; an unknown topic is "The sage knows nothing about that." `World::topics`
    is the one place topics come from: quests ("Quests" below) add theirs there, first.
  - **Saying words** (`warlock-core/src/words.rs`, `content/word.rs`, `world/words.rs`;
    docs/design/prestige-classes.md §6, #878): a dialogue may know words (`[[dialogue.word]]`,
    §7 "Dialogue"). Saying something (`Action::Say { dir, words }`, replay text
    `say:E:life_before_death`, the words normalised and joined by `_`) to the creature beside
    the player is refused as a chat is (no one seen there: no time; no dialogue or a hostile:
    the chat's words and its time). Else what was said is normalised (`words::normalise`:
    Unicode lower case, every character that is not a letter or digit a space, runs of spaces
    one, trimmed; so case and punctuation never matter) and matched against the dialogue's
    words in order: the first whose normalised `phrase` equals it and whose `does` can be done
    is heard. Its `reply` lines are said, each a message (`The spirit says: "…"`), the last the
    dialogue screen's line; then what it `does`: `quest` gives that quest as asking its topic
    would, if it could be asked for now (not active, its requirements and alignment met, the
    player not too green; else the reply is all); `departs` sends the speaker away after the
    reply (`World::departs`, as an ally whose time is up: no corpse, no pile, no experience, no
    kill; the screen does not reopen); `calls` (a prestige class's calling, §8 "Prestige
    classes", one of the dialogue's campaign) makes the calling after the reply, before
    `quest` and `departs`, and the word is heard only while the calling is open (anyone else
    gets `unheard`). The Ascmearc's **Strange Spirit** (`strange_spirit`, faction `spirits`,
    hostile to no one; good; one hit point, no corpse, no blood, no experience; rooted,
    `budge = never`) is met on the roads, vale, downs, and woods by day (`when = { calling =
    "hunter" }`, weight 2), speaks only to the Kind or better (`speaks_to`), has no topics, and
    knows one word, *life before death*: it echoes the oath's six lines, calls the bard into
    the Hunter, and departs ("The strange spirit fades, and is gone."). Killed, it is a
    peaceful good soul killed; its party is forgotten and it may be met again while the calling
    is open. Look mode says ", who waits" after one whose dialogue has words and nothing else
    to offer. No match: the
    dialogue's `unheard` line (without one, "The sage says nothing.", told plainly, the
    screen's line "…"). Either way it takes the time
    of `chat`: no count of tries, no lock-out. Nothing is drawn, trained, or saved; others hear
    a chat. A dialogue's `speaks_to` (an alignment range, §8 "Alignment") is the speaker's
    reaction to the player: outside it a chat or a say gets only its `silent` line (a chat's
    time; an ask, none), with no topics and no Say row.
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
    `[[dialogue.service]]`s (`id`, `name`, `price` in gold, `does` = `cure`, `mend`, or `identify`, `reply`,
    `nothing`) are sold for gold and asked for like topics (`ask:E:cure`): `World::topics` lists
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
  #148): simple, ADOM-like errands people give through their dialogue topics. No escorts, timers,
  or branching outcomes.
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
    Topics the speaker does not have now are refused ("… knows nothing about that.") and change
    nothing. Talk about quests takes no time.
  - **Goals**: *kill* counts the player's own kills (§8 "Experience": melee, shots, throws, and a
    wound of theirs that bleeds it to death; never by others or traps) of the creature type, or
    of any type carrying a **creature tag** (below), on its level or any level of its branch if it names
    one, else on every level summed. Kills are tallied per level and creature type, every type (saved;
    the ranger's Favoured prey reads the same tally, §8 "Class abilities"), so kills made before an
    authored quest was given count, and a rolled quest or a repeat of a repeatable one counts only
    the kills since it was given. A kill of a **named monster** (a `unique` type; `count` 1) is
    met by its death, at any hand (the player's, a trap's, another creature's) and whenever it
    died, before the quest was given too (the guild design §8.4: no quest can be softlocked):
    every death passes `creature_dies`, which keeps the dead unique types (`Quests::slain`,
    saved). This is the one place the log may know what the player did not see. *Bring* counts what the player carries of
    the object. *Reach* is met once the player has been on the level, before taking the quest
    up too. *Deliver* is met while the player carries the item: dropping it (or losing it) blocks
    the quest until they pick it up again (it cannot be handed in without it).
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
    checked at load (one line, at most 280 characters, no unknown placeholder). A variant's
    `reveals` (`when = "given"`) marks its own location only. Dev: `quest_roll:ID:TARGET`
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
  - **State**: per quest the player has heard of, its stage (active or done; an `offered`
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

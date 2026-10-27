# Warlock specification — 8. Architecture (continued): creatures

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Creature behavior** (`warlock-core/src/world/ai.rs`, inside `World::creature_turn`; M10):
  quite deterministic, outplayable by design; no randomness. Each action, in order:
  1. **Look**: the hostile actors it sees (§5 "Creature sight"; hostile by faction, §7); the
     nearest is its target (squared distance; ties: the lower actor id).
  2. **Fight**: a `melee` creature attacks a target next to it (diagonals too), else steps along
     a shortest path toward a cell next to it. A `ranged` creature or `caster` shoots or casts
     when the target is within its `range` (the rounded radius, like every radius; for a shot no
     farther than its missile weapon's range) with a clear line of fire: no opaque cell and no
     actor on the cells between (an integer Bresenham line, the one missiles fly); else it steps
     toward the nearest cell that gives it one, failing that toward the target. A `ranged`
     creature with nothing to shoot (no missile weapon ready, or no ammunition for it left) fights
     as a `melee` one. A creature with a throw (§8 "Ranged combat": "Creatures' throws", the
     giants' boulders) that is not next to its target throws first whenever the target is within
     the throw's range with a clear line of fire.
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
  and kill (a shot is a missile, "Ranged combat" below); spells stay intents until magic is
  designed. An attack is an event (`Event::Attacked { attacker, target, kind: Melee | Ranged |
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
  wall; a level legend may place one there. No new state: its position is a plain cell.
  Not yet: immunity to weapons that are not magical (the hook is `World::resolve` in
  `fight.rs`); bolts and bursts stop at walls, so spells do not reach one inside. The example is
  the stone haunt (`stone_haunt`), which no level places yet.
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
  silver" and how it moves). When it dies its body falls **whole** by `[drops] intact` (50 %,
  plus the player's Salvager), drawn from `drop/<actor id>`, else nothing of it. Nothing can be
  stolen from one ("It is all of a piece.", no time). Talents (system `animated`): Salvager,
  Unbinder (+1 to hit, +2 damage against a creature tagged `animated`). Blades, hollow armour,
  and panoplies follow (#484 to #487, #463).
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
  Gear adds no state: it is the creature's inventory and equipment. **Scavenger** (Delver
  talent, the `creature_gear` system): what a creature the player kills drops is identified as
  it lands. The wiki's creatures table states each type's gear exactly.
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
    moved by favour; the Pilgrim talent halves it (rounded up) and has a cure also mend a point
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
    known and with "You cannot pay." short of the total, neither taking time. Pilgrim does not
    halve it (a temple's talent), nor do Lore or Connoisseur.
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
    and `decline-ID` of older replays are no one's and refused. An
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

# Warlock specification — 8. Architecture (continued): combat

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Combat** (`warlock-core/src/combat.rs`, the rules; `world/fight.rs`, applying them; M16),
  after D&D 3rd edition with 2d12 and ADOM-like hit points. Every roll comes from the gameplay
  random stream.
  - **Hitting**: an attack hits when 2d12 (its *natural* roll, 2 to 24, average 13) plus its
    to-hit is at least the target's **Defence** ("Dv"). A natural 2 always misses (and is a plain
    miss), a natural 24 always hits. To-hit is the attack's own (`to_hit`, default 0) plus, in
    melee, the attacker's Strength modifier, and for a shot the Dexterity modifier. At +0 against
    Defence 10 an attack hits 75 % of the time; each point is worth about 7–8 points of chance
    near the middle and much less at the ends.
  - **Criticals** (3.5's threat range and confirmation): a hit whose natural roll is at least
    the attack's `threat` (default 22, "22–24", 3.5's natural 20 by probability; 20 for 3.5's
    19–20, 19 for 18–20) threatens; a second roll with the same to-hit against the same Defence
    confirms it (by the same rule: 2 never, 24 always), and then the damage is rolled `crit`
    times (default 2), modifiers each time. At +0 against Defence 10 a default weapon confirms
    on 3.1 % of attacks, a 19–24 one on 10.9 %.
  - **Damage**: the attack's dice (their fixed part included) plus its bonus, at least 1 a hit.
    Melee (and later thrown) weapons add the Strength modifier; a shot adds no modifier: the
    launcher's own bonus is in its dice (its draw weight), and each point of Strength below what
    a bow needs costs 1 to hit and 1 damage. Bare hands are 1d3 plus Strength; a creature with
    no weapon uses its natural attack (§7), else bare hands. Melee uses the weapon in the first
    hand slot that holds one, and with a weapon in each hand the other strikes too ("Fighting
    styles" below); a shot uses the missile weapon in a `ready` slot and the
    ammunition matched to it ("Ranged combat" below); a throw (`Profile::thrown`) its thrown
    weapon's own attack, Dexterity to hit and Strength to damage.
  - **Weapon skills** ("Weapon skills" below): the rank in the skill an attack draws on adds to
    its to-hit and damage and widens its threat range; the wielded weapon's skill adds to its
    Parry and shield blocking to a held shield's Shield; a high rank makes an attack cheaper.
    The fighting style the attack is fought in adds its own ("Fighting styles" below).
  - **Defence** = 10 + one bonus per category: **Dexterity** (the Dexterity modifier, capped
    by the lowest `max_dex` of what the actor wears and by its burden, +3 Burdened, +1 Strained,
    0 Overloaded ("Burden" below), the lowest cap winning; a penalty counts in full), **Armour** (worn
    `armour`), **Shield** (held `shield`), **Parry** (a wielded weapon's `parry`), **Natural** (the
    creature type's `natural` armour, or a magic item's), **Dodge** (the tumble skill's bonus,
    "Skills" below, and magic items), and **Deflection** (magic items). Within a category only
    the highest bonus counts, except Dodge, whose bonuses add up. A magic item's own Armour,
    Shield, Parry, and `max_dex` affixes add to that item's; its Dexterity affixes raise the
    score (§8 "Magic items").
  - **Protection** ("Pv", after ADOM's PV; #192): every hit that lands loses a **soak** of one
    draw, uniform from ⌈Pv/2⌉ to Pv (`combat::soak`, `soaks`), drawn only when the Protection
    left after the attack's penetration is above 0, so fights against unprotected targets keep
    their draws. The soak comes after the critical multiplied the damage, once, so criticals
    punch through armour; the at-least-1 of a hit is before it, so a hit can be **soaked to
    nothing** (`Strike::Soaked`): then nothing else comes of it (no blood, no bleeding, no big
    hit, no `on_hit`, no wound), but it landed: the player's weapon skill and style earn the
    marks of a hit, a shield bash may still follow it, and a missile may still break. Sources
    **add up**: each worn piece's `protection` (body armour, helmet, gloves, boots, cloak; never
    a shield, which is Defence only) with what its Protection affix adds (§8 "Magic items"), and
    the creature type's natural armour (`natural = { defence, protection, name }`; the player's
    race's instead when it has one, `Actor::hide`: lizardfolk's scales, Protection 1). Armour
    moves part of its value from Defence to Protection: light armour stays mostly Defence (a
    leather jerkin Dv +1 / Pv 1), heavy armour mostly Protection (chain mail Dv +2 / Pv 3, plate
    Dv +1 / Pv 6); the heavy helms, gloves, boots, and the fur cloak give Pv 1 or 2; the cave
    beetle's shell Dv +1 / Pv 3. **Penetration**: a weapon or a missile may have `penetration`,
    ignoring that much Protection (the war hammer, the stiletto, crossbow bolts; a shot's is its
    launcher's and its ammunition's together). Protection soaks every blow rolled against
    Defence: weapons, natural attacks, shots, throws, kicks, bashes, and a trap's arrows and
    spears; not a blast, a bolt, a fall, a status effect, or a disease. It soaks **physical
    damage only** ("Damage types" below): the Pv a blow meets is the target's against the kind
    the blow does (`damage::Protection`, `Actor::protection`): the sum of what is worn and the
    hide for a physical kind, each piece's `protects` adjustment added for that kind (chain
    mail `{ slashing = 1, piercing = -1 }`), nothing for an element (no soak, and no draw), and
    the full sum for untyped damage. Added damage ("riders", below) is never soaked.
  - **Damage types** (`warlock-core/src/damage.rs`, the model; `content/damage_type.rs`,
    `data/damage_types.toml`; #151): every amount of damage is of a kind (`Damage { amount,
    kind }`; `DamageTypeId` an index into the ordered file, in the content fingerprint) or
    untyped. Built in, in this order: the physical kinds slashing, piercing, bludgeoning, and the
    elements fire, cold, electricity, acid, poison; each with the verb its blows are said with
    ("slashes"/"slash", "stabs"/"stab", "strikes"/"strike", "burns", "chills", "shocks",
    "corrodes", "poisons") and `physical`; one type is `unarmed` (bludgeoning: bare hands, kicks,
    shield bashes). **Sources**: a weapon names its `damage_type`, one kind or two (a sword
    `["slashing", "piercing"]`, a morning star bludgeoning or piercing; daggers, spears, and
    arrows piercing; axes slashing with their own `verb` "cleaves"; maces and staves
    bludgeoning; a shield's bash the unarmed kind); a creature's natural attack names its own
    (bites piercing, the deer's hoof bludgeoning); traps and scrolls name theirs (the arrow and
    the spears piercing, the fireball fire, the frost cold, the lightning electricity, the pit
    bludgeoning); water that freezes names its plunge's (cold); the poisoned status is poison.
    A weapon with two kinds does **whichever is better against its target**, decided before the
    dice and without a draw (`Kinds::best`): the kind the target resists least, then the one its
    Protection stands least against, ties to the first named; the kind done is in the attack's
    event (`Event::Attacked { damage }`) and the blow is said with its verb ("The kobold stabs
    you.", "You slash the giant rat."; bare hands and throws still "hit", shots "shoot"). What
    names no kind (test content; the built-in data names one everywhere, a test pins it) is
    untyped: soaked and wounding as every blow was. **Added damage** (`Rider { kind, dice }`):
    a weapon, a missile weapon, or ammunition may add dice of another kind (`added = { type =
    "fire", damage = "1d4" }`: the flaming arrow is piercing plus 1d4 fire), and magic items add
    "1dN fire damage" (the `elemental` affix effect: Flaming, Freezing); riders are rolled after
    the attack's own dice and extra die, each time the damage is rolled (a critical doubles them
    too), in this order: the weapon's own, its affixes' (one die per kind, sides summed like the
    plain extra die), the ammunition's own and its affixes' for a shot, then the worn ones'; they
    are never soaked (added damage is always an element: a physical `added` is a content error).
    **Resistances** (`damage::Resistances`, `Actor::resistances`): a percentage per kind, what a
    creature is (`resist = { acid = 50 }` on its type, and the player's race's `resist`: drow
    poison 25) plus what it wears and wields (each piece's
    `resist`, and the `resistance` affix effect: of the Salamander, of Warmth), summed and capped
    at **75** (`MAX_RESISTANCE`) above and −100 below; **100 is immunity**, and only a creature's
    own 100 is immune: gear never makes anyone immune. After the soak, each kind of a blow is
    resisted on its own (`damage::resist`: the amount less the percentage, rounded to the nearest,
    half up, so a 1-point hit at 50 % still hurts and at 75 % does not; a vulnerability adds); a
    hit left with nothing is `Strike::Resisted` ("Your torch does the salamander no harm.", "The
    fireball does the salamander no harm."; unseen ends: "... but does no harm."), which, like a
    soaked hit, landed (it trains) and does nothing else; a hit whose main damage the soak took
    and that had no rider stays `Soaked`. The same resistance meets every other typed harm
    through `World::resisted`: a burst's and a bolt's damage (traps and scrolls alike), a pit's
    fall (bludgeoning), and the plunge through the ice (cold). **Untyped** harm, which nothing
    resists: starving and choking, a disease's hit points, a wound's bleeding ticks, and a status
    effect's ticks in general: the poisoned status is typed poison, and meets its resistance as a
    **dose** instead (`World::afflict`: an immune target is never poisoned, a resistance takes
    its share off each dose, rounded to the nearest, a dose shrunk to nothing is no dose), never
    per tick, where 1-point ticks would round away. **Wounds** (bleeding) come only of blows of
    the kinds the status's `wound.from` names (slashing, piercing): a club's big hit, a fall, and
    a fireball open no wound; an untyped blow wounds as before. **Heat and frost** stay what a
    burst's `heat`/`frost` say (§8 "Traps"): a fire burst melts ice and a cold one freezes water
    because its data says so, not by its kind. Bludgeoning does nothing special to ice or walls
    yet (no bludgeoning source reaches either: bursts are elemental and digging is by tool), and
    electricity does not arc through water. **Shown**: the player's own resistances in words
    only (`PerceptionView::own_resistances`, `ResistanceWord`: "Resistant to cold", "Vulnerable
    to fire", "Immune to poison", on the character screen and the `stats` dump layer, JSON
    `resistances` by type id); an item's kinds, added damage, per-kind Protection, and
    resistances in its description, and an affix's percentage once identified; never a
    creature's resistances, which the player learns by play ("does no harm"). The dev `actors`
    layer shows every actor's `resists`.
  - **Why it missed**: a miss whose natural roll plus to-hit is below 10 would have missed an
    unarmoured, motionless target: a plain miss. Otherwise one category with a positive bonus
    made the difference, picked by one draw weighted by its bonus (in category order); a
    negative bonus is never the reason. Against Defence 16 (Dexterity 2, Armour 3, Shield 1) an
    attacker at +0 misses 69 % of the time: 36 % of those plain, 32 % the armour, 21 % a dodge,
    11 % the shield. The reason is in the attack's `strike` and its message (§6): Dexterity and
    Dodge are dodges, Armour and Natural glance off, Shield blocks, Parry parries, Deflection
    turns aside. Later rules hook in there (dodges training Dexterity, shield and parry effects).
  - **Order of draws**: the two dice of the attack roll; on an attributable miss one draw for
    why; on a threat the two dice of the confirmation; then one draw per damage die each time the
    damage is rolled (the attack's own dice, then a magic weapon's extra die, "adds 1d4
    damage", then its riders in order, "Damage types" above; the kind a two-kind weapon does is
    picked before the dice, no draw); then, on a hit whose kind the target's Protection stands
    against (physical or untyped) above the attack's penetration, one draw for the soak (an
    element: none); the resistances come off without a draw; then, if the target lives and took
    damage, one draw for each `on_hit` effect of the attack whose
    chance is below 100 ("Status effects" below), and for a disease that lands on a target that
    does not carry it yet, the two dice of its check and, if it is caught, one draw per
    incubation die ("Diseases" below).
  - **Magic** (§8 "Magic items"): an attack is built from its weapon (or natural attack, or bare
    hands; a kick is bare hands) and the attacker's scores as they are now (raised by what it
    wears, lowered by its status effects, hunger, and stat damage: `Actor::effective_stats`), then
    its weapon skill's benefit, then the weapon's own attack affixes (its to-hit, damage, extra
    die, critical threat and multiplier; a shot's from the missile weapon and the ammunition, a
    bow's draw lowering the Strength it needs) and the global ones of what the attacker wears and
    wields, then its combat stance. A threat goes no lower than 2, a multiplier no higher than 5.
  - **Hit points** (ADOM-like, no hit dice, no levels): maximum = Toughness + race HP + class
    HP (§7 tables: e.g. a dwarf barbarian with To 16 has 6 + 8 + 16 = 30; a human warrior with
    To 12 has 23; an elf wizard with To 8 has 10), at least 1; a creature's = Toughness + its
    type's `base_hp` (a giant rat with To 12 and none has 12). Linear in Toughness: no runaway
    pools. Every actor starts at its maximum (`Actor::hp`, `hp_base`). Magic items add to the
    maximum (and Toughness affixes through the score; `Actor::max_hp(content)`), not to what it
    has; when the maximum falls (taking one off), hit points above it are lost at once (above
    the maximum without its status effects, `Actor::lasting_max_hp`: what a status takes off the
    maximum comes back, so the hit points kept under it stay; while it lasts the player is shown
    no more than the lowered maximum).
  - **Regeneration**: a hurt actor on the current level gets 1 hit point back per
    `max(60 s, 360 s − 12 s × Toughness)` of game time at the normal pace (To 10: 4 minutes; To
    25 and up: 1 minute); resting, status effects, diseases, a sanctuary (§7 "Special rooms"),
    starving (§8 "Hunger"), and the regeneration affixes of magic items (§8 "Magic items") change
    the pace ("Healing" below).
  - **Big hits and status effects**: a hit that takes at least a third of the target's maximum
    hit points is a **big hit** (`combat::big_hit`). A blow that hits and leaves its target alive
    may cause status effects: those whose `wound` it meets (bleeding: a big hit 1 degree, a
    confirmed critical 2; every blow alike until weapons have categories), then each `on_hit`
    effect of the attack (a creature's natural attack: the giant rat's bite poisons, or exposes to
    filth fever, by its chance) ("Status effects", "Diseases" below).
  - **Death**: at 0 hit points an actor dies (`Event::Died { actor, at, of }`, right after the
    attack, or at the moment a status effect takes the last hit point, `of` naming it). A
    creature leaves the world, and what it carried, worn and wielded things too, falls on its
    cell on top of the pile there, in inventory order, with its corpse on top of all (§8
    "Hunger"). The player's death ends the run: it is recorded (`World::death`: turn, time, the
    level's name, and the creature type that killed them if they saw it, or the status effect
    that did, or the `cause` when neither did: starving or choking, §8 "Hunger"), the turn stops
    there, every later action is refused, and presentation shows the death screen and deletes
    the save (§6, §7 "Saves").
  - **What the player perceives**: their own hit points, Defence total, and Protection total
    (`PerceptionView::own_hp`, `own_defence`, `own_protection`); an armour's Protection, and
    that a weapon pierces armour, in its description (§6); how hurt each seen creature looks, in words (`SeenActor::health`:
    unhurt, lightly wounded, wounded, badly wounded, nearly dead); and the messages. Never a
    creature's hit points, a to-hit, a roll, a modifier, the Defence breakdown (its Dexterity
    part would give a modifier away), a creature's Protection, a soak, or a penetration: a blow
    soaked to nothing glances off, and only the message hints. Ground-truth dev dumps (`actors`)
    show every actor's hit points, Defence, and Protection. The status effects the player suffers, by name
    (`PerceptionView::own_effects`), and those that show on seen creatures
    (`PerceptionView::effects_of`); never a severity or a timer.
- **Experience** (`warlock-core/src/content/experience.rs`, the rules from
  `assets/data/experience.toml`; `world/experience.rs`, applying them; M21, #133). Creature types
  have a `level` (1 to 99) and an `xp` value (§7). The player has an experience total
  (`WorldState::experience`, saved; zero is not serialized) and their level follows from it and
  their character (M30, #226; the design is `docs/design/progression.md` §3). **The curve is
  written in kills** of the player's own level (`[kills]`): with m = n − 1 a level n costs
  `k(n) = plateau × m ÷ (m + ramp) + slope_per_100 ÷ 100 × m` kills (213, 5, 285: 38 for level
  2, 163 for level 10, 223 for 20, 365 for 60), each worth the **creature XP guideline**
  `per_level × n + base` (5 and 10, which the creature data follows: a content test fails for a
  type more than three times off it either way, but for the listed outliers, the bat, the deer,
  and the shopkeeper); a level's XP is `k(n) × guideline(n)`, rounded once from the exact
  fraction, and the threshold of level n is the sum over levels 2 to n (767 for level 2, 45 052
  for 10, 221 097 for 20, 2 861 748 for the cap, level 60: about 3 000 such kills to reach level
  20 and 15 000 to reach 60; a test pins the table's hash; XP beyond the cap counts but buys
  nothing). **Race and class factors**: races and classes have `experience`, a percent of the
  XP a level needs (50 to 200, default 100: elf 140, gnome 120, dwarf 115, half-elf 105, human
  100, halfling 95, half-orc 90; wizard 125, monk 120, priest, druid, and witch 115, paladin 110,
  ranger 105, warrior and bard 100, thief 95, barbarian, craftsperson, aristocrat, and merchant
  90, commoner 80), and a character's thresholds are `(threshold × race × class + 5 000) ÷
  10 000` (`content::Factor`; a run without a character is at 100 %): an elf wizard needs 175 %,
  a half-orc commoner 72 %. The factor is on the thresholds only, so every source of XP is
  scaled by it once. A kill is worth `Experience::kill_xp(xp, monster level, player
  level)`, in integers (classic World of Warcraft): at or above the player's level the XP plus 5
  % for each level of difference up to 4 (+20 %), rounded; below it `xp × (ZD − d) ÷ ZD` rounded,
  with `d` the levels below and `ZD` the "zero difference" for the player's level (5 at levels 1 to
  7, 6 at 8–9, 7 at 10–11, 8 at 12–15, 9 at 16–19, 11 at 20–29, 12 at 30–39, 13 at 40–44, 14 at
  45–49, 15 at 50–54, 16 at 55–59, 17 at 60); a monster `d ≥ ZD` below is grey and gives 0, any other kill at least 1, and a creature worth
  0 XP gives 0. Only kills by the player count: `World::dies` awards the XP when the one who
  landed the killing blow is the player, whether by melee, a shot, or a throw (`resolve` and
  `missile_hits`), and when a creature dies of a status effect the player caused (a wound they
  dealt bleeds it to death: the effect's `by`, "Status effects" below); traps (#128) and kills
  by other creatures give nothing. **Discovery** (`[discovery]`, optional; no draw of any random
  stream): the player's first kill of each creature type (their own, as above) gives the type's
  XP once more, `first_kill` percent (100) of it unscaled by the levels, so even a grey one
  teaches ("You learn something of kobolds.", the type's `plural`, said before any level it
  earns; a type worth 0 XP teaches nothing and is not remembered); the types are remembered by id
  in `WorldState::progress` (`Progress::discovered`, left out of the JSON while empty). The
  player's first arrival on a level gives `arrival` (5) × the level's item level (§8 "Magic
  items": a branch level's depth, a hand-made level's `item_level`), "You gain 10 experience for
  finding your way here." as the turn is perceived; the level a run starts on, the overworld, and
  local areas give none, so walking the land is worth nothing. A level is new when this arrival
  made its state (`World::travel`), so arrivals need no memory. A level
  gained says "You feel more experienced." and "Welcome to level N." (once per kill, naming the
  last level; after the attack's own messages, or as the turn is perceived for a creature that
  bled to death), and each level gained up to `hp_level` (20)
  raises the player's maximum and current hit points (`Actor::hp_base`, `hp`) by `max(1, class
  hp ÷ 2 + the Toughness modifier)`; without a character the scenario's `hp` stands in for the
  class's. The player is shown their level, XP, and the XP of the next level
  (`PerceptionView::own_experience`), and nothing else about the rules.
- **Stances** (`content/stance.rs`, `data/stances.toml`; M22), after ADOM's combat tactics. The
  player is in one of four **combat stances**, `CombatStance`: aggressive, neutral (the
  default), defensive, coward. It is `Actor::stance` (saved, in the state hash; creatures stay
  neutral for now, and the AI may pick one later). `Action::SetStance(stance)`, text
  `stance:aggressive`, takes no game time (`Spent::NoTime`) and is recorded in replays; it says
  "You take an aggressive stance." (a neutral, a defensive, a cowardly). `+` and `-` step towards aggressive and towards coward (`CombatStance::more_aggressive`, `more_defensive`; they stop at the ends).
  The file has exactly one `[[stance]]` per stance (`id`, `name`) and, all optional: `to_hit` and
  `damage` (added to the stanced actor's melee, shot, and thrown attacks: `StanceDef::apply`,
  through `Actor::stanced`), `dodge` and `parry` (added to its Defence in those categories, so
  Dodge stacks and Parry only counts above the weapon's own; a stance may make Defence worse),
  `attack_cost` (percent of the price of every attack: a blow, a bash, a kick at a creature, a
  shot, a throw; `World::attack_price`; never a door kick or a reload),
  `flee_cost` and `flee_noise` (percent of the price and the loudness of a step that **flees**:
  the actor has a hostile next to it and the step ends out of that creature's reach,
  `World::flees`), and `weapon_mark` (percent: the multiplier for weapon skill marks, the hook
  for #134; nothing counts marks yet). Bonuses are -20 to 20, percentages 10 to 1000. Shipped
  numbers (first guesses, tuned by playtesting; none is ever shown, only the stance's name):
  aggressive +1 to hit, +2 damage, -3 Dodge, marks ×1.25; defensive -2 to hit, -1 damage, +2
  Dodge, +1 Parry; coward -4 to hit, -3 damage, +4 Dodge, attacks ×1.25, fleeing steps ×0.85
  and half as loud. The player perceives it (`PerceptionView::own_stance`): the status bar shows
  it when it is not neutral, the character screen always (§6); the wiki's Combat page has the
  generated table.
- **Free blows** (`world/free_blow.rs`; #228, design `docs/design/progression.md` §4.1 B3): blows
  beyond an attack's first, all decided by `World::second_blow` on a `Cue`, so there is one
  place for every extra blow. The second blow of an attack (the off hand's, Sword and Board's
  bash, §8 "Fighting styles") is one; the others are free blows, switched on per actor
  (`FreeBlows` on `Actor::free_blows`, saved and hashed but left out while none is on, read
  only through `World::free_blows_of`; talents and class abilities switch them on, none does
  yet):
  - **Cleave** (`cleave`): when the blows of an actor's melee attack (a blow and the second
    blow, a kick, a shield bash) kill its target, a blow with what it wields at another hostile
    beside it: the next one clockwise from where the target fell (decided: no draw, and a sweep
    reads naturally). One per attack; with **great cleave** (`great_cleave`, which cleaves as
    `cleave` does) each cleave that kills gives another, so at most once per hostile around.
  - **Riposte** (`riposte`): when a blow of a melee attack on the actor missed by its Parry
    (`Strike::Missed { by: Parry }`, `Defence::why_missed`), a blow back with what it wields
    once the attack is over (decided: one per attack however many of its blows were parried,
    after the attack's last blow, so it never cuts into it). Shots and throws are never
    riposted.
  - **Opportunist** (`opportunist`): a hostile stepping from beside the actor to a cell out of
    its reach (the geometry of fleeing, `World::leaves`, whatever the stepper's stance) is struck
    as it turns to go, before it leaves its cell (`World::step_away`, from the player's step and
    `creature_steps`): it may not live to take the step. Each opportunist beside it strikes, in
    actor order.
  - Each is a blow like any other: its own `Event::Attacked`, blood, wounds, kills (and their
    experience), and weapon marks; free (no time, no exertion); silent but the opportunist's,
    an attack of its own (the others follow a blow that made the noise of the fight). **A free
    blow brings none of its own** but a cleave after a cleave (decided: ripostes never answer
    ripostes, a riposte or an opportunist's blow that kills does not cleave), so they never
    chain past the hostiles around.
  - The player's are said as they strike (`player_blow`). What they strike in another's turn (a
    riposte, an opportunist's blow) is held (`World::say` while `World::held`) and joins the news
    said at the end of `perceive`, so it never comes before the report of what it answers ("You
    parry the kobold's spear." then "You hit the kobold.").
  - **Determinism**: deciding a free blow draws nothing (only the second blow's bash chance
    draws); a free blow draws only as the blow it is, so fights without free blows switched on,
    or whose triggers do not fire, keep their draws and hashes.
- **Ranged combat** (`warlock-core/src/world/shoot.rs`; M16): shots and throws.
  - **Loosing**: the player fires (`Action::Fire(cell)`, `fire:X,Y`) with the missile weapon in a
    `ready` slot and the ammunition it shoots in a `ready` slot, or throws (`Action::Throw { item,
    at }`, `throw:LETTER:X,Y`) one of an inventory stack that is a throwing weapon (wielded or
    not: the last one leaves the hand), or any other weapon that is no launcher or ammunition,
    whole, with `IMPROVISED_TO_HIT` (-4) to hit and `IMPROVISED_RANGE` cells (`ObjectDef::
    is_improvised`). The Missile weapon slot (`takes_weapons` in `slots.toml`) holds any weapon or
    throwing weapon besides launchers, so `t` throws what is readied there (a weapon readied
    there does not parry). Refused, taking no time: with nothing ready ("You have nothing ready
    to throw or shoot.", "You have no arrows ready."), at the player's own cell, beyond the
    range (the rounded radius of the weapon's `range`: "That is out of range."), something that
    cannot be thrown ("You cannot throw a coil of rope."), and on the overworld. A shot takes
    `shoot` (plus `reload` for a missile weapon that `reloads`), a throw `throw` (less what the
    weapon skill's rank takes off, "Weapon skills" below); creatures shoot
    by the same rules (`World::creature_shoots`; the AI saw to it that they can).
  - **One missile** leaves its stack (the last empties its slot: "You have no arrows left.") and
    makes its noise once, as it is loosed (a bowstring, 40; a throw, heard as fighting, 30).
    What is `lit` flies and lands as that object (a flaming arrow as a burning arrow).
  - **Flight**: along the integer line from the shooter through the cell aimed at and on beyond
    it (`Pos::ray`: up to that cell exactly the line of fire creatures check), cell by cell, as
    far as the weapon's range, over transparent cells only: it stops before the first opaque one
    (and the level's edge). Whoever stands on a cell it reaches is attacked in turn
    (`World::missile_hits`: rolled against their Defence, no noise of its own), until one is hit;
    on a miss it flies on. At the cell aimed at it comes down if nobody stands there; past
    someone it missed there it flies on.
  - **Coming down**: on the cell of whom it hit, else on the last cell it reached (before
    something solid, at the end of its range, or the cell aimed at), on top of the pile there, to
    be picked up again (stackable ammunition joins its stack). On a hit it breaks with its
    `breaks` chance (one draw of the gameplay stream, made only when the chance is not 0); a
    missile that comes down on a cell that is not walkable is lost.
  - **Fires**: a burning object (`light` and `burns`) that comes down starts a fire on its cell
    (`LevelState::fires`: the light preset and when it has burned out; saved), a light source
    keyed by the level and the cell (`LightCache`, only while the clock is before its end, so
    the player's perception and every creature's sight see by it exactly as by any other light,
    at any moment) that lights by every rule of light (falloff, shadows, bounce, wall faces).
    Another burning object on the same cell keeps the fire going until the last has burned out.
    When the time is up (checked as perception runs) the fire goes out and the burning objects
    there are gone (they leave nothing). A burning object cannot be picked up ("You cannot pick
    up a burning arrow while it burns."; the rest of the pile can). Fires cost nothing but
    themselves: only lit objects are sources.
  - **Messages**: the player's attacks are said as they are made, by what they see ("You shoot
    the kobold.", "You miss the kobold.", "You hit the kobold." for a throw); a missile that
    reaches nobody says itself ("You shoot a flaming arrow.", "You throw a dagger."). A creature's
    shots are said like its blows ("The kobold archer shoots you.", "You dodge the kobold
    archer's arrow.").
  - **What the player perceives**: `Event::Missile { object, from, path, end }`, the whole flight
    cut to the stretch between the first and the last cell of its path the player sees (a burning
    one: in their line of sight, light itself); `from` only if they saw it leave; `end` `Struck`,
    `Fell`, or `Unseen` when the last cell seen is not where it ended. Dropped if none of it was
    seen. The player's own readied missile weapon, its ammunition, and ranges are in the snapshot
    (`PerceptionView::ready_to_fire`, `ready_to_throw`, `throw_range`) for the targeting UI.

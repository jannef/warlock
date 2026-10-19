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
    damage"); then, if the target lives, one draw for each `on_hit` effect of the attack whose
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
  - **What the player perceives**: their own hit points and Defence total (`PerceptionView::
    own_hp`, `own_defence`); how hurt each seen creature looks, in words (`SeenActor::health`:
    unhurt, lightly wounded, wounded, badly wounded, nearly dead); and the messages. Never a
    creature's hit points, a to-hit, a roll, a modifier, or the Defence breakdown (its Dexterity
    part would give a modifier away). Ground-truth dev dumps (`actors`) show every actor's hit
    points and Defence. The status effects the player suffers, by name
    (`PerceptionView::own_effects`), and those that show on seen creatures
    (`PerceptionView::effects_of`); never a severity or a timer.
- **Experience** (`warlock-core/src/content/experience.rs`, the rules from
  `assets/data/experience.toml`; `world/experience.rs`, applying them; M21, #133). Creature types
  have a `level` (1 to 99) and an `xp` value (§7). The player has an experience total
  (`WorldState::experience`, saved; zero is not serialized) and their level follows from it:
  the XP needed for level n is `linear × (n − 1) + quadratic × (n − 1)²` (10 and 10: 20 for
  level 2, 60, 120, 200, ..., 900 for level 10, 24500 for the cap, level 50; XP beyond the cap
  counts but buys nothing). A kill is worth `Experience::kill_xp(xp, monster level, player
  level)`, in integers (classic World of Warcraft): at or above the player's level the XP plus 5
  % for each level of difference up to 4 (+20 %), rounded; below it `xp × (ZD − d) ÷ ZD` rounded,
  with `d` the levels below and `ZD` the "zero difference" for the player's level (5 at levels 1 to
  7, 6 at 8–9, 7 at 10–11, 8 at 12–15, 9 at 16–19, 11 at 20–29, 12 at 30–39, 13 at 40–44, 14 from
  45); a monster `d ≥ ZD` below is grey and gives 0, any other kill at least 1, and a creature worth
  0 XP gives 0. Only kills by the player count: `World::dies` awards the XP when the one who
  landed the killing blow is the player, whether by melee, a shot, or a throw (`resolve` and
  `missile_hits`), and when a creature dies of a status effect the player caused (a wound they
  dealt bleeds it to death: the effect's `by`, "Status effects" below); traps (#128) and kills
  by other creatures give nothing. A level
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

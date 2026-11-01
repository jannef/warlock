# Weapon abilities

Developer notes for working on this part of Warlock (#561, slice W2, and #564, slice W3 (the
actives), of the combat progression design, `docs/design/combat-progression.md` §4, §6, §11);
the rules it implements are in
spec/08i-character.md "Weapon abilities". Keep it current with the code, as CLAUDE.md says.
Ranks, marks, and benefits are `docs/dev/weapon-skills.md`; free blows `docs/dev/combat.md`.

## Data

`data/weapon_abilities/<id>.toml`, one `[[weapon_ability]]` a file (a directory: churning
content is one file per entity; `README.toml` has the format), read through
`ContentFiles::table("weapon_abilities")`, so a campaign's `data/weapon_abilities/` adds more.
`content/weapon_ability.rs` parses them after the talents (`Content::load`): `WeaponAbility`
(`id`, `name`, `description`, `skills`, `rank`, `triggers`, `systems`, `effect: WeaponDoes`,
`active: Option<Active>`; `Active { aim: ActiveAim, stamina, action: ActionKind, percent,
cooldown }`).
`WeaponAbilityId` is the index in path order, in the content fingerprint, **never saved**:
nothing in a save names an ability (free blows are bits on `FreeBlows`, statuses are statuses).
An ability names its skills; a skill does not list its abilities.

Loader checks: the skills exist and are weapon skills or styles, never both; `rank` 1 to 10;
`kind = "triggered"` has no `aim`, `stamina`, `energy`, or `cooldown`, and no active's effect;
`kind = "active"` has no `trigger`, needs `aim`, `stamina` (1 to 50), and `energy` (an
`actions.toml` id, `percent` 50 to 300: no active is free of time), an optional `cooldown` (a
status with `eases` and no `tick`), and an active's effect, each with the aims and actions it
fits (`active()`: a blow is `direction` and `melee`, a sweep `nothing` and `melee`, a buff
`nothing` and `wait`, a shot `cell` and `shoot`; `ignore_protection` either a blow or a shot);
an active's id is no spell's, class active's, or birth sign gift's (the action text reads those
first); with any active, the `stamina` resource exists (`STAMINA`); a talent's
`weapon_stamina { abilities }` names actives (`check_talents`, skipped for a content without
weapon abilities); a switch effect (`free_blow`, `parry_missiles`) has no `trigger`, every
other triggered effect needs one; `free_shot` fires on `approaches` only; an effect has only its kind's fields (as `RodDoes` is checked); a status
effect's `severity` or `turns` (dice, one draw) fit the status's `max`. `systems` are the
talent table's system ids (a test holds the shipped ones to the table; a test's own talent table
may have others, so the loader does not). The shipped catalogue is in the wiki's tables
(`xtask/src/wiki/weapon_abilities.rs`: `weapon-abilities.md` on the Weapon skills page,
`style-abilities.md` on the Fighting styles page).

Readers: `Content::weapon_abilities`, `weapon_ability(id)`, `weapon_ability_id(name)`, and
`weapon_abilities_for(skill, rank)` (a lookup, no draw).

## Who has one

`Actor::ability_rank(content, skill)` is the rank abilities read: the actor's own, raised by a
`weapon` talent's `ranks` (Stylist, Fencing; `TalentMods::weapon_benefit`, as benefits read it)
and by `ability_ranks` (Early mastery); an untrained skill stays 0. `Actor::weapon_abilities`
lists what the ranks teach, each once, with the skill it is listed under (the first of its
skills, in skill order, whose rank teaches it), in skill order, rank, path order: the menu's
order. Creatures have ranks from their types (`weapon_skills` in creature data, animated
objects' by their tier), so they have the abilities by the same rule.

What works now is what the actor fights with: `Actor::abilities_through(skills)` (private) reads
the abilities of an attack's `AttackSkills` (weapon skill and style). So a style's ability works
only in its grip, and a weapon skill's only with a weapon of the skill.

## Switches: free blows and Measure

`World::ability_switches(actor)` reads the abilities of what the actor holds for a blow
(`attack_skills(Melee, None)`) and turns on `FreeBlows` bits (`free_blow` effects: Brace's
gatekeeper, Riposte, Cleaving sweep, Great cleave) and `parry_missiles` (Measure). It is the one
reader: `World::free_blows_of` folds the bits in beside the actor's own and its talents'
(`either`, so a riposte from the style and the talent is one), and `World::target_against` asks
for Measure. **Parry counts against blows only** (`target_against(id, missile)` removes a
positive Parry for a shot, a throw, a trap's missile, a bolt) but for an actor with Measure.
`World::brace_of(id)`: Brace of steel's `(to_hit, status)` when the actor's gatekeeper comes from
a weapon ability (`gatekeeper` adds the to-hit, `step_up` the status on a hit).

## Triggers: one hook

`World::weapon_trigger(by, Trigger, Fired { other, kind, skills, blow }, events)` is **the one
place triggered abilities fire**: it skips at once when no ability has the trigger or the actor
has no ranks, reads the abilities of `skills` with the trigger, and applies each effect in path
order. `Trigger` is closed (`Trigger::ALL`, the wiki's order; `defending()` marks those that fire
for the one attacked). Where each is raised (a line that already marks the event):

| Trigger | Raised | Skills |
|---|---|---|
| `hit`, `critical`, `struck` | `fight.rs::resolve` → `World::resolve_triggers`, after the resources' `Build`s (a soaked or resisted hit is a hit) | the attack's; the target's held (`World::held_skills`) for `struck` |
| `blocked`, `dodged` | the same, on a melee miss by the target's Shield, or Dexterity or Dodge (decided: melee only, a striker beside) | the target's held |
| `big_hit` | `resolve`, before `after_blow` (the target alive), `World::big_hit_trigger` (`combat::big_hit`) | the attack's |
| `kill` | `resolve`, beside `train_kill` (the attack is known there, not in `dies`) | the attack's |
| `sneak_attack` | `ambush.rs::sneak_trigger`, from `resolve` after `resolve_triggers` and from `cast.rs::spell_attack` after `spell_practised` (#685): the attack was a sneak attack (`sneaks`: the attacker has sneak dice) and hit, so its dice landed in `ambushed`; before the damage, as `hit` (Knife in the dark, Hunter's shot) | the attack's |
| `parried` | `second_blow(Cue::Attacked { parried: true })`, before the cleave or riposte | the target's held |
| `both_hands_hit` | `World::both_hands`, from `World::attack` and `player_attacks` after the second blow: both `Strike::Hit`, the second made with a weapon (not a shield: a bash is no second hand; not a flurry's fist), the target alive; once an attack | the attack's |
| `approaches` | `World::approached`, from `World::step_up` (the creature's step in `ai.rs` and now the player's in `travel.rs`) | the approached's missile weapon (`Ranged`), else what it has ready to throw (`Thrown`) |
| `hit`, `critical` (spells) | `world/cast.rs::spell_practised` → `World::resolve_triggers` with `AttackKind::Spell`, before `spell_strikes` (a spell that rolls to hit, an own cast; #571) | the school and the caster style the cast was made in (`Cast::skills`, `Actor::spell_skills`; `world/spell_aim.rs`) |
| `spell_blocked` | `cast.rs::cast_single` → `World::spell_blocked` (`reflect.rs`), on an attack or touch spell's miss by the target's Shield (a bolt, a ray, a touch; a scroll's or a wand's too); never for a reflected flight | the target's held |

Effects (`WeaponDoes`, closed; a kind with no catalogue entry is not written): `Status` (on the
other end or the actor, `World::afflict`; `turns` dice drawn once, clamped to the status's
`max`; a defending trigger by the player says "The orc staggers from your shield."),
`ExtraDamage` (Rend: the main hand's dice without their bonus once more, with Second blade's
rend the off hand's too, rolled after the blows, of the blow's kind or the effect's,
`World::resisted` but never soaked, `hurt` with `Harm::Actor`; "Your blades rend the orc.";
"The orc's blades rend you."), `FreeShot` (Point-blank: `World::loose` with the missile weapon
and ready ammunition, or what is ready to throw; nothing without, on the overworld, or for the
player in a shop; the player's held as an out-of-turn free blow's messages are), `ReflectSpell`
(Spell reflection, #566: `World::reflect_ready` marks the reflector while it has made fewer
than `per_turn` reflections this turn of its own; the count lives in the transient
`World::reflection` and is cleared in `spend_actions`; the cast code casts the spell back,
`docs/dev/spells.md` "Spell reflection"), and the two switches (nothing to do here).

**Determinism**: deciding never draws (lookups, bits); an effect draws only as it lands
(Pinning shot's turns, Rend's dice, a free shot's flight); fights with no qualifying rank keep
their draws and hashes (`no_draws_while_no_rank_qualifies` runs a fight with and without the
abilities' data and compares every hash).

## Statuses that ease by actions

`per_action` statuses (`content/status.rs`, `StatusDef::per_action`; no `eases`, no `tick`)
ease a degree with each action their sufferer takes: Staggered (max 1: the next action), Pinned
(1 to 3 actions), Light-footed (the next action, at half price). `World::spend_actions(id)` is
called where each action's price is paid (`schedule.rs::run_turn`: the player's, then each
creature's after `creature_turn`). A dose given during the sufferer's own action is not eased by
it: `World::acting` (whose action is under way: the player's outside the schedule loop, the
creature's during its turn, nobody's while time passes) and `World::fresh` (doses given to the
acting actor; `World::afflict` records them) are transient, never saved. Staggered moved from
6 seconds to `per_action` here (#561): given in the sufferer's own turn (a blocked blow), a
time-based stagger eased away before its next action.

## Actives: used for stamina and time (#564)

`world/weapon_active.rs`. **The action**: `use:lunge:E` is `Action::Act(id, dir)` (the parser
reads the direction as typed, in upper case, since a lower-case letter is a stack for
`use:identify:e`; `ne`, `se`, `sw`, `nw`, no letters, in either case); `use:shield_wall` and
`use:piercing_shot:12,5` are `Action::Cast`, which `player_acts` sends to `World::use_weapon`
when the id is no spell's but a weapon ability's (and `buff.rs::kind_of` prices and blocks as
its `energy.action`, not `cast`: Raging blocks spells, not actives). `Ability::action` /
`action_at` make the same actions, so a replay of the menu's use reads back the same.

**Who may use one**: `Ability::Weapon`'s arm of `unavailable` (`ability.rs::weapon_unavailable`):
triggered, `FiresByItself`; not `Actor::holds_for` (the abilities a blow's, or for a shot the
missile weapon's, `attack_skills` teach, as `abilities_through` reads them; one shield blocking
teaches while a shield is in a hand and the rank holds), `NotSoHeld`; its cooldown, `While`;
short of `Actor::weapon_stamina` (its stamina less `TalentMods::weapon_stamina`, Drillmaster,
and `ability_stamina`, Armour-breaker's Sunder and Gap shot, and the worn `active_stamina` for
stamina, of Ease's, global: whatever piece holds it, B1 #630; at least 1; the early return is
"no talents and nothing worn"), `Short(stamina)`; a
status blocking its action, `While`. `use_weapon` says the reason capitalized ("You are short of
breath.") and refuses with no time.

**Costs, after the refusals**: each effect's function makes its own refusals first (no hostile
there, `reach_toward`'s "There is no room to lunge.", nothing to shoot, out of range, nobody
beside), prices the action before it is made (`attack_price` with what the blow is made with,
the shield for a rush; `shot_price`; `price_of(Wait)`), then `pay_active` (stamina through
`spend_resource`, the cooldown if any), then the blow or shot, then `exert`, and returns
`Spent::Turn(active_price(id, base))`: `percent_of(base, percent × (100 +
weapon_energy_percent) / 100)`, never below 50 (`LEAST_PERCENT`). Light-footed and Walled are
in `base` (statuses' `cost` in `burdened`, Walled's `attack_cost` in `attack_price`).

**Effects**: `active_blow` (a blow with `me.melee()`, its profile changed: `sure_critical`,
`penetration = ALL_PROTECTION` (1000, above any Pv: no soak drawn), `twice` the dice with the
to-hit; Feint's categories on `Technique::ignores`, which `fight.rs::resolve` takes through
`World::feinted` for the next attack the player makes; `after_attack` only for a foe beside;
`knock_back` after it), `rush` (a bash via `player_strikes` with the shield's skills), `sweep`
(`hostiles_beside` sorted clockwise from the direction given or north; odd targets take the off
hand's `off_hand_blow` when there is one), `active_shot` (`player_looses`; Piercing shot sets
`Technique::pierce`, which `shoot.rs::fly` takes through `World::pierces`: the first hit
`continue`s instead of stopping), `active_buff` (`afflict` on the player). `knock_back`: only on
a hit with the target alive; the cell along the blow must be walkable with no trap, no ice, and
nobody (a teleport's check); the move pushes `Event::ActorMoved`, says "The post is driven
back." if seen, then the effect's `status` (Staggered). `Technique` is transient (not saved,
cleared after each active), like `acting` and `fresh`.

**Walled**: `status.toml`'s last entry, `combat = { shield = 3 }` (`CombatBonus::shield`, added
by `Actor::defence` on top of the Shield category while it is above 0: a status's shield adds
to the shield's own instead of competing with it) and `attack_cost = 125`
(`StatusDef::attack_cost`, `Effects::slow_attack`, multiplied into `attack_price`).

**The pool**: `fight.rs::grant_stamina`, from `ranks_news`, holds stamina (`focus::hold`) for
the player without it when the abilities they now know include an active they did not before;
the line "You learn to pace your breath." follows that active's "You have learned the lunge."

## The menu, the news, the dev rank

`Ability::Weapon(skill, id)` (`ability.rs`), one row per (skill, ability) pair in `every`, had
only through the first skill that teaches it, so each ability is one row; its name is the skill's
and the ability's ("Long blades: Deep cut"); a triggered one is `Unavailable::FiresByItself`
("fires by itself"), no action, no aim; an active one has its data's aim and action (above), and
the menu row shows its stamina first ("3 stamina, in a direction"). The abilities screen widens
its panel to fit such rows.
`World::ranks_news(risen, before)` (`fight.rs`) says a rank risen ("You feel more skilled with
long blades.") with what it taught appended to its skill's line ("You have learned the deep
cut."); `practise` computes `before` only when a rank rose. The dev action `rank:SKILL:N`
(`Action::SetRank`, `World::set_rank_dev`) sets a rank as a rank risen would, said the same way.

## Talents

System `weapon_abilities` (`talents.toml`), with Early mastery (`ability_ranks`), Second blade's
rend (`rend_off_hand`), Brace of steel (`brace { to_hit, status }`), Practised technique
(Weaponcraft, a weapon skill at 7: `weapon_energy_percent` −25, read in `active_price`), and the
warrior's class ability Drillmaster (level 25: `weapon_stamina` 1, read in
`Actor::weapon_stamina`). Armour-breaker (P1's) gained `weapon_stamina { value = 2, abilities =
["sunder", "gap_shot"] }`.

## Tests

`content/weapon_ability.rs` (the shipped catalogue, its systems, no "aimed" names, every
refusal, the actives' too), `world/weapon_ability_tests.rs` (each triggered ability on a
hand-made fight, the abuse rules, Spell reflection's cap, mark, shapes, Parry, mana, lifted dice,
wand, and draws, creatures' Brace and Point-blank, no draws without a rank, the three talents,
the menu and the news), `world/weapon_active_tests.rs` (each active, its costs and refusals, the
greyed rows, Practised technique, Drillmaster, Armour-breaker, the pool granted, the action
text), `ability.rs` (the menu's order), the screen snapshots `abilities-menu-weapon-skills`,
`abilities-menu-weapon-actives`, `-short`, the wiki test (`xtask/src/wiki/mod.rs`).

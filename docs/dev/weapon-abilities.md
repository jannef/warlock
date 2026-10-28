# Weapon abilities

Developer notes for working on this part of Warlock (#561, slice W2 of the combat progression
design, `docs/design/combat-progression.md` §4, §6, §11); the rules it implements are in
spec/08i-character.md "Weapon abilities". Keep it current with the code, as CLAUDE.md says.
Ranks, marks, and benefits are `docs/dev/weapon-skills.md`; free blows `docs/dev/combat.md`.

## Data

`data/weapon_abilities/<id>.toml`, one `[[weapon_ability]]` a file (a directory: churning
content is one file per entity; `README.toml` has the format), read through
`ContentFiles::table("weapon_abilities")`, so a campaign's `data/weapon_abilities/` adds more.
`content/weapon_ability.rs` parses them after the talents (`Content::load`): `WeaponAbility`
(`id`, `name`, `description`, `skills`, `rank`, `triggers`, `systems`, `effect: WeaponDoes`).
`WeaponAbilityId` is the index in path order, in the content fingerprint, **never saved**:
nothing in a save names an ability (free blows are bits on `FreeBlows`, statuses are statuses).
An ability names its skills; a skill does not list its abilities.

Loader checks: the skills exist and are weapon skills or styles, never both; `rank` 1 to 10;
`kind = "triggered"` has no `aim`, `stamina`, `energy`, or `cooldown`; `kind = "active"` is
refused until W3 (an active with a `trigger` names that first); a switch effect (`free_blow`,
`parry_missiles`) has no `trigger`, every other effect needs one; `free_shot` fires on
`approaches` only; an effect has only its kind's fields (as `RodDoes` is checked); a status
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
| `parried` | `second_blow(Cue::Attacked { parried: true })`, before the cleave or riposte | the target's held |
| `both_hands_hit` | `World::both_hands`, from `World::attack` and `player_attacks` after the second blow: both `Strike::Hit`, the second made with a weapon (not a shield: a bash is no second hand; not a flurry's fist), the target alive; once an attack | the attack's |
| `approaches` | `World::approached`, from `World::step_up` (the creature's step in `ai.rs` and now the player's in `travel.rs`) | the approached's missile weapon (`Ranged`), else what it has ready to throw (`Thrown`) |
| `hit`, `critical` (spells) | `world/cast.rs::spell_practised` → `World::resolve_triggers` with `AttackKind::Spell`, before `spell_strikes` (a spell that rolls to hit, an own cast; #571) | the spell attack skill and the caster style (`Actor::spell_skills`) |

Effects (`WeaponDoes`, closed; a kind with no catalogue entry is not written): `Status` (on the
other end or the actor, `World::afflict`; `turns` dice drawn once, clamped to the status's
`max`; a defending trigger by the player says "The orc staggers from your shield."),
`ExtraDamage` (Rend: the main hand's dice without their bonus once more, with Second blade's
rend the off hand's too, rolled after the blows, of the blow's kind or the effect's,
`World::resisted` but never soaked, `hurt` with `Harm::Actor`; "Your blades rend the orc.";
"The orc's blades rend you."), `FreeShot` (Point-blank: `World::loose` with the missile weapon
and ready ammunition, or what is ready to throw; nothing without, on the overworld, or for the
player in a shop; the player's held as an out-of-turn free blow's messages are), and the two
switches (nothing to do here).

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

## The menu, the news, the dev rank

`Ability::Weapon(skill, id)` (`ability.rs`), one row per (skill, ability) pair in `every`, had
only through the first skill that teaches it, so each ability is one row; its name is the skill's
and the ability's ("Long blades: Deep cut"), `Unavailable::FiresByItself` ("fires by itself"),
no action, no aim. The abilities screen widens its panel to fit such rows.
`World::ranks_news(risen, before)` (`fight.rs`) says a rank risen ("You feel more skilled with
long blades.") with what it taught appended to its skill's line ("You have learned the deep
cut."); `practise` computes `before` only when a rank rose. The dev action `rank:SKILL:N`
(`Action::SetRank`, `World::set_rank_dev`) sets a rank as a rank risen would, said the same way.

## Talents

System `weapon_abilities` (`talents.toml`), with Early mastery (`ability_ranks`), Second blade's
rend (`rend_off_hand`), Brace of steel (`brace { to_hit, status }`). Drillmaster and Practised
technique ship with W3.

## Tests

`content/weapon_ability.rs` (the shipped catalogue, its systems, no "aimed" names, every
refusal), `world/weapon_ability_tests.rs` (each ability on a hand-made fight, the abuse rules,
creatures' Brace and Point-blank, no draws without a rank, the three talents, the menu and the
news), `ability.rs` (the menu's order), the screen snapshot `abilities-menu-weapon-skills`, the
wiki test (`xtask/src/wiki/mod.rs`).

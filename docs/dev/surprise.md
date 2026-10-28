# Surprise

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08c-combat.md ("Surprise and sneak attacks"), the design in
`docs/design/surprise-and-sneak-attacks.md` (its slices: #681 S1 this, then #682–#688). Keep it
current with the code, as CLAUDE.md says.

## Modules and data

- `assets/data/surprise.toml`, `content/surprise.rs`: `SurpriseRules` (`status`, `difficulty`,
  `missile_reach`, `memory_turns`, `no_vitals`), `Content::surprise()`. Optional: without it
  nobody is surprised, no Defence is stripped, and the player is never unaware (fixtures). Load
  checks: the status exists, is `per_action` with `max = 1`; the numbers in range; each
  `no_vitals` tag carried by some creature type. `missile_reach` and `no_vitals` are parsed and
  not read yet (the Sneak attack skill, #682).
- `assets/data/status.toml`: `surprised` (after Light-footed; `per_action`, `max = 1`, `refresh`,
  `cost = 150`, `seen = "surprised"`, its `start` and `end` lines).
- `world/surprise.rs`, every rule of it:
  - `World::unaware(id, other)` (moved from `ambush.rs` with `sees_now`): the one definition,
    both ways round. A creature: asleep or disguised, else `Mind::last_seen` unset and
    `sees_now` false. The player (`player_unaware`): the attacker is not among
    `perception().actors()` and its `Mind::noticed` is `None` or more than `memory_turns` turns
    before `WorldState::turn`; never without the rules.
  - `World::mark_noticed`: called from `perceive` right after `Perception::compute`, sets
    `noticed = Some(turn)` on each creature in view. Never from `from_state` (a loaded save's
    perception is recomputed without writing state).
  - `World::startle(target, by, events)`: the one place anyone is surprised. A sleeper or a
    disguised mimic: `afflict(surprised)` with no check and no word. Else the initiative check
    (`check::check`: Dexterity modifier + `talents.check(Dexterity)` + `initiative_bonus`
    against `difficulty`); failed, `afflict(surprised, 1, by)`. The messages (below).
  - `World::flat_footed(target, unaware, struck)`: `Target::without([Dexterity, Dodge])` when
    unaware (with the rules) or Surprised.
  - `World::surprised(id)`, and `World::status_cost(id, status)`: a status's `cost` percent as
    its sufferer pays it; Surprised's extra eased by `TalentMods::surprise_ease` (Cool head,
    capped at all of it), every other status's as its data says.
- `combat::Target::without(categories)`: the strip of positive Defence bonuses, shared with
  Feint (`weapon_active.rs::feinted`).

## The hooks (one line each)

| Where | What |
|---|---|
| `fight.rs::resolve` | `unaware` asked of every attack with a profile (no draw); `sneaks` (`ambush.rs`) decides the ambush dice on unaware **or Surprised**; `startle` before the mimic is found out and before the strike's draws; `flat_footed` after `target_against` |
| `cast.rs::spell_attack` | a spell that rolls to hit (an own cast, a scroll's, a wand's, a reflected one): `unaware`, `startle`, `flat_footed`, before its roll |
| `trap.rs::step_onto_trap` | a hidden trap (any trap for a creature, an unknown one for the player) that sprang (`World::sprung` moved) `startle`s its victim, after the trap's own draws; nobody beside |
| `free_blow.rs::free_blows_of` | Surprised: no riposte, opportunist, gatekeeper (its own attack's cleave, flurry, and second blow stay) |
| `weapon_ability.rs::approached` | a Surprised keeper fires no Point-blank shot |
| `schedule.rs::slowed` | `Effects::slow_with(price, status_cost)`: the dear action |
| `weapon_ability.rs::spend_actions` | a `per_action` status that ends pushes `Event::Status { began: false }` (its `end` line: "You recover your wits.", Staggered's and Pinned's too) |
| `perceive.rs::perceive` | `mark_noticed` |
| `dump.rs` | the `actors` layer: `unaware of player` / `aware`, `noticed turn N` |

## Messages

Told after the fact, never the roll: the player's attack, check failed: "You catch the kobold
unawares." (said at once, before the blow's line; only when the player sees the creature); made:
"The kobold whirls to face you."; a sleeper or a mimic: nothing beyond the waking or reveal line.
The player caught: "You are taken by surprise!" (the status's `start`, through `Event::Status`,
so after a trap's line and before a creature's attack line); made: "You keep your wits about
you." (`say` for an attack, before its line; `news` for a trap, after its line). Look mode shows
"surprised" (`seen`); the status bar shows Surprised.

## Invariants

- One definition of unaware, one place anyone is surprised, one place the dear action is priced.
- The check is two gameplay draws, only when an attack finds its target unaware or a hidden trap
  springs on someone awake; `unaware`, `flat_footed`, and the free-blow and cost hooks draw
  nothing, so a fight in plain sight draws as before (`no_draws_while_nothing_is_unaware`).
- `Mind::noticed` is the player's half of awareness on the creature: saved and hashed (left out
  while `None`; `Mind::is_empty` decides the actor's `mind` is left out, `Mind::is_calm` ignores
  it). A replay whose player sees a creature hashes differently from main from that step; one
  whose first attack finds a creature unaware also draws two more.
- The player stays unaware of an unseen attacker in the dark however often it strikes (each blow
  surprises again): `noticed` is what they saw, not what they felt.

## Tests

`world/surprise_tests.rs` (both ways round, memory, the check's outcomes and draws, sleepers and
mimics, the messages, flat-footed, free blows, the dear action and Cool head, Wary, traps and
bystanders, the hash test, saves and replays); `content/surprise.rs` (parse and refusals,
optional file); the screen snapshot `status-surprised`.

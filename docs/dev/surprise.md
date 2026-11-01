# Surprise

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08c-combat.md ("Surprise and sneak attacks"), the design in
`docs/design/surprise-and-sneak-attacks.md` (its slices: #681 S1, then #682–#688, all landed).
Keep it current with the code, as CLAUDE.md says.

## Modules and data

- `assets/data/surprise.toml`, `content/surprise.rs`: `SurpriseRules` (`status`, `difficulty`,
  `missile_reach`, `memory_turns`, `no_vitals`), `Content::surprise()`. Optional: without it
  nobody is surprised, no Defence is stripped, and the player is never unaware (fixtures). Load
  checks: the status exists, is `per_action` with `max = 1`; the numbers in range; each
  `no_vitals` tag carried by some creature type. `Content::has_vitals(creature)` (#682) is the
  one reader of what has no vitals: incorporeal, `vitals = false` (`CreatureDef::vitals`,
  default true), an animated object, or a `no_vitals` tag (Anatomist, which found vitals in
  some, was cut in #787); everything has vitals without the file.
- `assets/data/skills.toml`: `sneak_attack` (after Search; `sneak_dice`, `surprise`; marks on
  `sneak_attack`) and `initiative` (after Heal; `initiative`, `surprise_ease`; marks on
  `surprise`). The classes' lines and the drow's `optional_skills` (#682).
- `assets/data/status.toml`: `surprised` (after Light-footed; `per_action`, `max = 1`, `refresh`,
  `cost = 150`, `seen = "surprised"`, its `start` and `end` lines).
- `world/surprise.rs`, every rule of it:
  - `World::unaware(id, other)` (moved from `ambush.rs` with `sees_now`): the one definition,
    both ways round. A creature: asleep or disguised, else `Mind::last_seen` unset and
    `sees_now` false (`creature_unaware`, a free function of the state, content, and light
    masks, so `Perception::compute` asks it too: Stalker's eye, #687, the ranger's class
    ability at level 2 since #788). The player
    (`player_unaware`): the attacker is not among `perception().actors()` and its
    `Mind::noticed` is `None` or more than `memory_turns` turns before `WorldState::turn`;
    never without the rules.
  - `World::mark_noticed`: called from `perceive` right after `Perception::compute`, sets
    `noticed = Some(turn)` on each creature in view. Never from `from_state` (a loaded save's
    perception is recomputed without writing state).
  - `World::startle(target, by, events)`: the one place anyone is surprised. A sleeper or a
    disguised mimic: `afflict(surprised)` with no check and no word. Else the initiative check
    (`check::check`: Dexterity modifier + `talents.check(Dexterity)` + `initiative_bonus` + the
    Initiative skill's `initiative` against `difficulty` + `World::craft(by)`, the attacker's
    Sneak attack `surprise` and talents' `surprise_difficulty`); failed, `afflict(surprised, 1,
    by)`. The player's check marks `Use::Surprise`. Made against an attacker, it ends with
    Hair trigger (`World::hair_trigger`, `free_blow.rs`, #687): `Cue::Startled`'s blow if the
    target holds `startled_blow` and the attacker is a hostile beside it; `startle` returns
    whether the attacker still stands, and `resolve` and `spell_attack` stop when it does not. When the player is the one
    caught, the attacker's `noticed` becomes the turn (the second place it is set; never on
    load). The messages (below).
  - `World::flat_footed(target, unaware, struck)`: `Target::without([Dexterity, Dodge])` when
    unaware (with the rules) or Surprised.
  - `World::surprised(id)`, and `World::status_cost(id, status)`: a status's `cost` percent as
    its sufferer pays it; Surprised's extra eased by `TalentMods::surprise_ease` (Cool head) and
    the Initiative skill's `surprise_ease`, together capped at all of it, every other status's
    as its data says.
- `world/slip.rs`, slipping past (#683): `looks_at` (the look of `ai.rs::nearest_hostile_in_sight`,
  the one moment: the contest's four gameplay draws for a creature hunting no one that would
  see the player by darkvision alone, `sight::sees_by` and `sight::lit_for`; a lit look ends a
  slip), `World::slipped_past` (`Mind::missed` within `lasts_turns`), `end_slip` (clears it and,
  if the creature's eyes reach the player, sets `last_seen`: it sees them at once, no new roll),
  `slip_lapses`, `attacked_near`. `SurpriseRules::slip_turns` (`[slip]`; `None` when off) and
  Stealth's `slip` benefit (`Effect::Slip`, read here only); the creature's side is its Willpower
  and its skills' `spot`.
- `world/ambush.rs`, the sneak attack (#682): `World::sneak_dice(id)` (the Sneak attack skill's
  `sneak_dice` d6s `and` the talents' `ambush`), `World::sneaks(attacker, target, unaware,
  missile)` (dice, unaware or Surprised, vitals for the attacker, a missile within
  `missile_reach` + `sneak_reach`, Chebyshev; no draw), `sneak_threat` (Master thief),
  `ambushed` (the dice on a hit, gameplay stream; `Build::SneakAttack`; the player's
  `Use::SneakAttack`, training stream), `has_vitals` and `vitals` (the struck side's
  `no_criticals`), `no_vital_spot` (Vital strike's and the
  finisher's "It has no vital spot.", a `news` line after the blow's).
- `world/thief.rs`, the thief's actives and passive on surprise (#684): Lurk (`lurking` in
  `status.toml`, after Surprised: `per_action`, `max = 1`, `refresh`, "You wait in ambush.",
  no `end`), Low blow (`Technique::sneak`), Vanishing act (`sneak_kill_status`). The hooks are
  the rows marked #684 below.
- `world/lurk.rs`, lurking (#686): `World::lurker_waits` turns the step of `ai.rs::fight`'s plan
  into a wait for a type with `lurks = true` (`CreatureDef::lurks`) while its hit points are at
  their maximum; an attack (a blow beside it, a shot or cast from where it stands) and a summons
  stand. No draws. The creatures that sneak-attack are data: `skills` (`sneak_attack`,
  `initiative`, `spot`) on eight types, read through the same `Actor::skill_effect` as the
  player's (the roster notes: `docs/dev/creatures/drow.md`, `fey.md`, `undead.md`,
  `campaign-waves.md`).
- `combat::Target::no_criticals`: `strike_with` makes a threat a plain hit with no confirmation
  draw, and a sure critical a plain hit.
- `combat::Target::without(categories)`: the strip of positive Defence bonuses, shared with
  Feint (`weapon_active.rs::feinted`).

- `xtask/src/wiki/surprise.rs`, the player wiki's tables (#688; `wiki/src/surprise.md`, under
  Combat): `surprise-rules.md` (`difficulty`, `missile_reach` with Long reach, `memory_turns`,
  `slip_turns`, Surprised's `cost`, Lurking), `surprise-ladders.md` (Sneak attack's,
  Initiative's, and Stealth's slip benefits by value through `Benefit::at`, and what the
  Surprised action costs after the ease, as `World::status_cost` prices it),
  `surprise-check.md` (`check::ways` by edge and craft), `surprise-vitals.md` (every type
  `Content::has_vitals` refuses, and why). The page
  also includes `slipping-past.md` (`skills.rs`) and `creature-surprise.md` (`creatures.rs`).
- `world/balance_probe.rs`, `balance_probe/tables.rs::sneak_attacks` (#688; `docs/balance.md`
  "Sneak attacks"): the thief's first blow on an unaware monster with vitals against its plain
  blow, by level; Sneak attack set by `sneak_value(level)`, the dice `World::sneak_dice`, the
  side `Side::flat` (`World::flat_footed`), Master thief's `ambush_threat`.

## The hooks (one line each)

| Where | What |
|---|---|
| `fight.rs::resolve` | `unaware` asked of every attack with a profile (no draw); `sneaks` (`ambush.rs`) decides the sneak dice on unaware **or Surprised**, a missile within reach, a target with vitals; `startle` before the mimic is found out and before the strike's draws; `vitals` and `flat_footed` after `target_against` |
| `fight.rs::target_of`, `Actor::target` | `no_criticals` from `Content::has_vitals` |
| `cast.rs::spell_attack` | a spell that rolls to hit (an own cast, a scroll's, a wand's, a reflected one): `unaware`, `sneaks` (a missile unless a touch), `startle`, `vitals`, `flat_footed`, `sneak_threat` before its roll; `ambushed` after, its dice on top of the spell's cap |
| `fight.rs::resolve`, `cast.rs::spell_attack` | `sneak_trigger` (`ambush.rs`) after `resolve_triggers` (`spell_practised`): a sneak attack that hit (its dice landed) fires the `sneak_attack` weapon abilities of the attack's skills (#685; docs/dev/weapon-abilities.md) |
| `weapon_active.rs::active_blow`, `openings.rs::finisher` | `no_vital_spot` after a sure critical's blow |
| `trap.rs::step_onto_trap` | a hidden trap (any trap for a creature, an unknown one for the player) that sprang (`World::sprung` moved) `startle`s its victim, after the trap's own draws; nobody beside |
| `free_blow.rs::free_blows_of` | Surprised: no riposte, opportunist, gatekeeper (its own attack's cleave, flurry, and second blow stay) |
| `weapon_ability.rs::approached` | a Surprised keeper fires no Point-blank shot |
| `schedule.rs::slowed` | `Effects::slow_with(price, status_cost)`: the dear action |
| `weapon_ability.rs::spend_actions` | a `per_action` status that ends pushes `Event::Status { began: false }` (its `end` line: "You recover your wits.", Staggered's and Pinned's too) |
| `fight.rs::resolve` (#684) | `lurked` (a lurk's blow on a creature not hunting the player is unaware: `startle`, flat-footed, dice) and `low_blow_taken` (a low blow passes `sneaks` as unaware: dice only); `vanishing_act` beside `Build::SneakKill` |
| `free_blow.rs::free_blows_of`, `step_up` (#684) | Lurking switches the gatekeeper's blow on; `lie_in_wait` flags the player's blow from a lurk (`Technique::lurk`, cleared after) |
| `perceive.rs::perceive` | `mark_noticed` |
| `perception.rs::Perception::compute` (#687) | `SeenActor::unaware` from `creature_unaware` for each creature in view, only with `sees_unaware` (Stalker's eye); false for all otherwise |
| `ui/look.rs`, `ui/glyphs.rs` (#687) | "unaware of you" after the seen statuses; the unaware mark (`unaware_tint`, `unaware_mix`; docs/dev/rendering.md) |
| `free_blow.rs::startled` (#687) | Hair trigger's blow (`Cue::Startled`, `Free::Startled`: noisy, out of turn, so held into the news) |
| `dump.rs` | the `actors` layer: `unaware of player` / `aware`, `noticed turn N`; JSON seen actors' `unaware` (Stalker's eye only) |
| `ai.rs::decide` (#686) | `lurker_waits` (`lurk.rs`) on the plan `fight` returns for a hostile in sight |
| `dump.rs` (#686) | the `actors` layer: `lurks` after the awareness of a lurking type |

Slipping past's hooks (#683): `ai.rs::nearest_hostile_in_sight` calls `slip::looks_at` for each
hostile; `ai.rs::decide` calls `slip_lapses` first; `ai.rs::noise_to_look_into` takes a cell seen
by darkvision alone for unseen while slipped past; `surprise.rs::unaware` reads `slipped_past`;
`fight.rs::resolve` and `cast.rs::spell_attack` call `attacked_near` after `startle`; `fight.rs::hurt`
calls `end_slip`; `dump.rs` says `slipped past` and `missed turn N`.

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
- An unseen attacker surprises the player with its first blow only (integrator, #693): `startle`
  sets the attacker's `noticed` to the turn whatever the check gave, so its next blows find them
  aware until `memory_turns` pass. Only the player's side: a creature struck by someone it does
  not see stays unaware of them (design §3.1), so each of an unseen thief's shots surprises it.

- `SeenActor::unaware` is computed, never stored, and only for a player with Stalker's eye: the
  UI never asks the world (`stalkers_eye_tells_who_is_unaware_and_nothing_leaks_without_it`).
- Hair trigger draws only as the blow it is, after the check's two draws and before the
  attack's; one per surprise because the check notices the attacker.

- One moment anyone slips past: `looks_at`, four gameplay draws only there, only when the
  creature hunts no one, the player is in its darkvision and dark to its eyes, and no slip is
  set. `Mind::missed` is saved and hashed (left out while `None`; `Mind::is_calm` ignores it).

## Tests

`world/surprise_tests.rs` (both ways round, memory, the check's outcomes and draws, sleepers and
mimics, the messages, flat-footed, free blows, the dear action and Cool head, Wary, traps and
bystanders, the hash test, saves and replays, Hair trigger's blow, its one per surprise and the
attacker it fells, Stalker's eye's gate); `world/ambush_tests.rs` (the ladder, the sum with
the talents, the reach and Long reach, Surprised as the second door, the marks, the spell path,
the craft and Initiative in the check); `world/vitals_tests.rs` (what has none, no critical, no
dice, still surprised); `world/class_rogue_tests.rs`
(`a_ranger_gains_stalkers_eye_at_level_two`); `weapon_active_tests` (Vital strike's line);
`combat.rs` (no confirmation drawn); `content/surprise.rs` (parse and refusals, optional file,
`[slip]` off); `world/slip_tests.rs` (the moment, made and lost, no second roll, each ending, the
creature that came to look caught unawares, another hostile, the no-draw test, saves and replays,
the unaware mark on a creature slipped past); `ui/look.rs` (`stalkers_eye_tells_a_creature_unaware_of_you`),
`ui/glyphs.rs` (the tint at both ends of the breath and under reduced effects);
the screen snapshot `status-surprised`; `xtask/src/wiki/surprise.rs` (the tables read the data).

## Hashes that move (#683)

A run whose unlit player came within darkvision of a creature hunting no one, with no light of
its own or others' on their cell (a kobold's torch lights them: no roll), draws the contest's four
more from that creature turn, and, won, the creature does not see them.

## Hashes that move (#682)

A run with a thief, ranger, monk, bard, warrior, or barbarian (their new class skills move the
later skill rolls of creation) or a drow (its optional roll) starts with other skills; a thief
holds Openings from level 1; a thief's surprises are 2 harder to shrug off and a trained
character's dear action is eased; a threat on a thing without vitals draws one fewer (no
confirmation), and a sure critical on one is a plain hit. Replays of such runs hash differently
from the step it shows.

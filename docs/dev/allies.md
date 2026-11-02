# Allies

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08b-creatures.md ("Allies"), the design is `docs/design/druid-and-summons.md` (§4 the ally
core, §14 the owner's decisions, which override the rest). Slices: **A1** (#836, this file: the
core), A2 (summons: the spell kind, `Ally::until` set from the spell's degrees), A3 (the UI: the
Allies group's rows, Set on's aim, the friendly mark, look mode's "(your wolf)"), D1 (the druid's
calls), and the ranger's companion (#825) on `AllySource::Companion`. Keep it current with the
code, as CLAUDE.md says.

## What an ally is

An ally is an ordinary creature `Actor` with `Actor::ally: Option<Ally>` set (`world/ally.rs`;
`serde(default, skip_serializing_if = "Option::is_none")`, so every other actor saves and hashes
as before, old saves load, and `World::state_hash` covers it as every field):

```rust
pub struct Ally {
    pub master: ActorId,          // the player, today; the field makes no assumption
    pub source: AllySource,       // Summoned { spell: Option<SpellId> } | Companion
    pub order: Order,             // Follow (default) | Stay { at } | Hunt { at }
    pub retinue: bool,            // travelling with the master off any level
    pub until: Option<GameTime>,  // when its stay ends, on the clock; None: until slain
}
```

`AllySource::Summoned { spell: None }` is the dev `ally:` action's; A2 sets the spell. A2 adds
nothing to the struct: a summon's duration is `until`, absolute game time, set at the cast from
the spell's degrees. `Actor::in_retinue()`, `Actor::master()`, `World::allies_of(master)` (every
level and the retinue, id order), `World::allies_here(master)` (the current level, acting).

**The side**: `World::make_ally` makes it with `travel::make_creature` on a `CreatureSpawn` with
the player's faction as `faction` (so `faction_override` is the player's), `rolls_gear = false`
(a beast carries nothing), and bumps `World::spawned`. That one assignment does the work: enemy
AI (`nearest_hostile_in_sight`), the spawn cap (`spawn.rs::hostiles_here`), the quick leave, the
edge rule, coarse turns (`stride.rs`), and displacement (`talk.rs`) all read stances and treat it
as the player's with no branch of their own. Two reads needed a line: `talk.rs::disturbs_rest`
(an ally never disturbs a rest) and the three places that must not see the retinue (below).

## The data

`assets/data/allies.toml` → `content/ally.rs::AllyRules` (`Content::allies()`, `None` without
the file: then no ally can be made, and fixtures without it hash as before): `[follow] heel`
(2), `leash` (8), `comes_along` (3; the owner's Q5), `[stay] reach` (1), `[hunt] reach` (3),
`[orders] noise` (60), `[allies] most` (8), `[xp] ally_percent` (100; the owner's Q1). Checks:
distances 1 to 20, the leash at least the heel, the noise 1 to `MAX_LOUDNESS`, `most` 1 to 24,
the percent 0 to 100.

## Its mind

`World::ally_plan(id)` (`world/ally.rs`) is the one arm at the top of `ai.rs::decide`, taken
for any actor with `ally`, after `slip_lapses`; `creature_turn`'s `frightened` and `held_back`
still wrap it. In order:

1. **A target**: the nearest actor it sees (`ai.rs::nearest_in_sight`, the generalised
   `nearest_hostile_in_sight`: a predicate of the state, the content, the creature, and the
   other; a disguised mimic never; ties by lower id, line of sight computed lazily out to the
   nearest) with `stance_between(master, other) == Hostile` (`world/mod.rs::stance_between`, the
   borrowed-state form of `World::stance`: Beast-friend is the *master's*, so a druid's wolf
   leaves the rats alone as the druid does, and a shopkeeper or a quest-giver is never an
   ally's idea). With *Stay*, only one within `stay.reach` of its post; with *Hunt*, one within
   `hunt.reach` of the hunt cell, else the nearest anywhere in sight.
2. **Fight** it: `ai.rs::fight` as a creature would (melee closes in; a type that throws or
   shoots does; a summoner summons).
3. **Its order**: *Follow*: `route` toward the master to within `heel`, else wait; *Stay*: back
   to its post, else wait; *Hunt*: `route` to the cell; arrived (or no way there), the order falls
   back to *Follow* and, seen, "Your wolf looks back at you."
4. **The leash**: a *Follow* ally farther than `leash` from its master takes no target: it heels.

Nothing here draws from any stream. The ally's `Mind` is untouched (the arm returns before the
creature's look); `summons_at`, `hurls_at`, `shuns` (never: it is not hostile to the player),
doors (`opens_doors`), traps, ice, and footsteps are the creature's own rules, unchanged.
`ai.rs` exposes `Plan`, `Route`, `fight`, `route`, and `nearest_in_sight` as `pub(super)` for it.

## Orders

`Action::Order(Command)` (`action.rs`; `Command::{Heel, Hold, SetOn(Pos)}`; replay text
`order:follow`, `order:stay`, `order:hunt:X,Y`) → `World::order`: every ally of the player's on
the level (not the retinue) takes the order (*Follow*; *Stay* at its own cell; *Hunt* at the
cell), **no time** (`Spent::NoTime`), a voice's noise where the player stands
(`noise::Act::Order`: `SoundKind::Voice` at `[orders] noise`, never quietened by stealth, as a
mutter or a thrown voice is not), `Event::Ordered { at }` for presentation, the message ("Your
wolf comes to heel." / "Your wolves hold." / "You set your wolf on the kobold." or "... on that
spot."; `pack_words`: by type, "wolf", "wolves", "wolves and bear"), and a Handle animal mark
(`Use::Handle`, `skills.toml` `handle_animal`'s `mark`, 50 %: the skill's first; the owner's
Q10). Refused, no time and no noise, with no ally on the level ("You have no one to call.") and
Set on at a cell the player does not see (`Perception::is_visible`). `buff.rs::kind_of` prices
it as nothing (no `ActionKind`); `refuse_pending` refuses it while an encounter is pending, like
any action that is not free.

The three abilities (`ability.rs`): `Ability::Heel`, `Hold`, `SetOn` under `Source::Ally`
("Allies", after Here in `Source::ALL`); `Aim::Nothing`, `Nothing`, `Cell`; `cost()` `None` (no
time); `action` / `action_at` make the `Action::Order`. They are had by nobody (`had_by` false,
as a place's) and listed by `Ability::orders(any)`: `PerceptionView::abilities` chains them
while the player has an ally anywhere (`Perception::allies` is not empty), greyed
`Unavailable::NoAllies` ("no allies here") while none is on the level; `Ability::every` lists
them when the content has `allies.toml` (the wiki's table). No `KeyCommand`, no `Context`, no
key: the owner's rule. Send away is not an order (the owner's Q2): a summon departs when its
time runs out or it is slain.

## Following across levels, and the retinue

`World::follow_along(from, to, old, arrival)` is called by `World::travel` once the player
stands on `to` (after `arrive_spawner`, before the arrival lines), with the player's old cell:

- every ally of the player's on `from` within `comes_along` (Chebyshev) of `old` and not held
  fast (a status that `blocks` steps) comes along; the rest stay, frozen as another level's
  actors are, and those the last perception saw are said left behind ("Your wolf is left
  behind."). A follower's `order` becomes *Follow* (a post on another level means nothing), its
  `heard` and `Mind` are cleared, its energy kept;
- onto a level of `LevelKind::Overworld`, which has no creatures, they join the **retinue**:
  `retinue = true`, `level` the overworld's, `pos` the arrival (in bounds for `WorldState::check`,
  never read). Three places must not see a retinue actor, and are the only ones:
  `WorldState::actor_at` (it stands in no one's way), `schedule.rs::run_turn` (it takes no turn
  and lives through no time), and `Perception::compute`'s seen actors (it is never drawn). Every
  other reader filters by level and never meets one acting;
- onto any other level, the retinue steps out: each placed by `free_cell_near` around the
  arrival in id order ("Your wolves follow you in." from the overworld, "... come with you."
  otherwise);
- a timed ally whose `until` has passed on the way is removed silently and does not step out
  (the owner's Q4: its minutes run on in the retinue), and so is any left on `to` earlier whose
  time has passed (the owner's Q5).

`encounter_area.rs` recomputes the cells the party must avoid after `travel`, so the retinue
stepping out beside the player is never stood on. The dev `ally:` on the overworld makes its
allies in the retinue at once.

## Time

`Ally::until` is the clock. `World::allies_lapse` runs at the end of `run_turn` (after the
actors' energies are written back, before `spawn_tick`): every ally on the current level past
its time departs through `World::ally_departs` (removed from `actors`, nothing left behind, no
corpse or pile, `Event::Departed { actor, at }`, pushed to `fallen` so the turn's reports may
name it, "The wolf fades away." when the player sees its cell; A2 gives the type its own
`depart` line). Nothing ticks on another level or in the retinue: a summon left behind, or on
the road, is checked by the clock when it is next met (`follow_along`, above). Statuses on an
ally tick as any creature's while it acts.

## Experience, quests, resources

`fight.rs::dies`: the killer's master, if the killer is an ally of the player's, is credited as
the player would be: `World::earn_kill_at(creature, percent)` at `[xp] ally_percent` of what
the player's own blow pays (the first-kill bonus and the level scaling included; `earn_kill` is
the test-only 100 % form), `tally_kill` (the quest log and Favoured prey's record), `judge_kill`
(alignment), `gain_resource(master, Build::Kill)`, and a Handle animal mark. `creature_dies`
takes `credited` for its by-player rules (a cursed room's rising). A death of a status an ally
caused (`Effect::by`) credits the same way. `docs/dev/experience.md`'s "award only where the
attacker is the player" has this one exception. An ally's own death pays no one.

## Beast-bond

`TalentEffect::AllyMod { to_hit, protection }` (`content/talent.rs`: either may be left out, not
both; bonuses) → `TalentMods::ally: AllyMods { to_hit, protection }` (summed). Read through the
**master's** mods, `World::ally_mods(id)`: `World::ally_blow` in `fight.rs::resolve` (after
`favoured`) adds `to_hit` to an ally's attack profile; `World::ally_protection` in
`World::protection_of` adds `protection` through `Actor::protection_plus` (the master's hide is
untouched). `assets/data/talents/beast_bond.toml` (Wayfarer, needs Handle animal 20: the owner's
Q10; `[[system]] allies` in `data/talents.toml`, after `danger`). The wiki's talent tables say
it (`xtask/src/wiki/talents.rs::effect_text`).

## Dumps and perception

- The header line: `allies N` while the player has any on the level or in the retinue
  (`PerceptionView::allies_with`).
- `actors` (dev): `; ally of #0 (summoned, 4 min; follow)`, `(summoned; stay at (12, 5))`,
  `(summoned; follow; retinue)`; the minutes are rounded up.
- JSON: `allies` (`id`, `creature`, `name`, `health`, `order`, `at` (the order's cell or null),
  `source`, `lasts` (minutes, or null), `retinue`, `here`), from `PerceptionView::allies()`
  (`AllyView`, the snapshot's `allies`: the player knows their own); each seen actor's `ally:
  true` (`SeenActor::ally`, for A3's mark and look mode); the `abilities` layer's Allies rows.

## Tests

`world/ally_tests.rs` (a fixture of a wolf, a kobold, a deer, two levels, the shipped
`allies.toml` and `experience.toml`; the shipped content for the retinue, Beast-bond, and the
Handle animal marks): the dev action and its refusals (the cap), following within heel, the
master's stance (the kobold slain, the deer untouched), an ally's kill paying the master what
their own blow pays (and half at `ally_percent = 50`) and tallying the quest log, orders' time,
noise, words, and refusals, holding and Set on (there and back to heel; at an enemy), coming
along and being left behind, the clock here and on a level left, a rest undisturbed, a save
round trip, Beast-bond through the master (and nobody else), the Handle animal mark, the
retinue across the showcase's overworld (never drawn, not in the way, stepping out, the timed
one gone on the road, back into the retinue, saved), the dumps and the menu rows, and a replay
through all of it without divergence. `content/ally.rs` checks the file. The showcase's dump
snapshots are untouched: no scenario makes an ally.

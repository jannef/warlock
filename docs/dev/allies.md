# Allies

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08b-creatures.md ("Allies"), the design is `docs/design/druid-and-summons.md` (§4 the ally
core, §14 the owner's decisions, which override the rest). Slices: **A1** (#836, this file: the
core), A2 (#837, "Summons" below: the spell kind, `Ally::until` set from the spell's degrees), A3 (the UI: the
Allies group's rows, Set on's aim, the friendly mark, look mode's "(your wolf)"), D1 (#839, "The druid's
calls" below), and the ranger's companion (#916, `docs/dev/companion.md`) on
`AllySource::Companion(CompanionData)`, and the witch's familiar (#929, `docs/dev/familiar.md`) on
`AllySource::Familiar(CompanionData)`, both **kept** beasts (`AllySource::kept`). Keep it current with the code, as CLAUDE.md says.

## What an ally is

An ally is an ordinary creature `Actor` with `Actor::ally: Option<Ally>` set (`world/ally.rs`;
`serde(default, skip_serializing_if = "Option::is_none")`, so every other actor saves and hashes
as before, old saves load, and `World::state_hash` covers it as every field):

```rust
pub struct Ally {
    pub master: ActorId,          // the player, today; the field makes no assumption
    pub source: AllySource,       // Summoned { spell: Option<SpellId> } | Companion(CompanionData) | Familiar(CompanionData) | Charmed { spell, faction }
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
`[orders] noise` (60), `[allies] most` (8), `[xp] ally_percent` (50: what a master whose class
has no `ally_share` is paid; the ranger's and the druid's `ally_share = 100` in
`data/classes/`, `ClassDef::ally_share`; #915). Every field is needed; the numbers are the
author's (#1205).

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
   `hunt.reach` of the hunt cell, else the nearest anywhere in sight. Seeing none, a type with a
   `nose` takes the nearest it smells instead (`World::senses` → `companion.rs::smelled`). A familiar
   takes the nearest it sees under a hex of its master's before any of that (`familiar.rs::quarry`).
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
spot."; `pack_phrase`: "your wolf", "your wolves", "your wolves and bear" by type, a companion by
its name first, "Snel and your wolves"), and a Handle animal mark
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
  fast (a status that `blocks` steps) comes along, and the player's **companion** wherever it
  stands, and their familiar likewise (every kept beast), held fast or not (`companion.rs`); the rest stay, frozen as another level's actors are,
  and those the last perception saw are said left behind ("Your wolf is left behind."). A
  follower's `order` becomes *Follow* (a post on another level means nothing), its `heard` and
  `Mind` are cleared, its energy kept;
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

A crossing actor's move is `travel.rs::cross_to` (level, cell, what it heard), shared with
hostile stair followers (`stair_followers.rs`, `docs/dev/creatures.md`). Those are placed after
`travel` and only beside the arrival, so an ally is never dropped for one, whatever its order:
Hold here and Set on reset to Follow as it comes, and a kept beast comes from anywhere and takes
a cell first. In a tight spot the follower stays behind.

`encounter_area.rs` recomputes the cells the party must avoid after `travel`, so the retinue
stepping out beside the player is never stood on. The dev `ally:` on the overworld makes its
allies in the retinue at once.

## Time

`Ally::until` is the clock. `World::allies_lapse` runs at the end of `run_turn` (after the
actors' energies are written back, before `spawn_tick`): every ally on the current level past
its time departs through `World::ally_departs` (removed from `actors`, nothing left behind, no
corpse or pile, `Event::Departed { actor, at }`, pushed to `fallen` so the turn's reports may
name it, "The wolf fades away." when the player sees its cell, or the type's own `depart` words, "The wolf
slips back into the dark." (`CreatureDef::depart`, D1). Nothing ticks on another level or in the retinue: a summon left behind, or on
the road, is checked by the clock when it is next met (`follow_along`, above). Statuses on an
ally tick as any creature's while it acts.

## Summons (A2, #837)

A spell of `kind = "summon"` (`content/spell_def.rs`; `SpellPurpose::Summon`, its tag
`summon`) calls creatures to the player's side. It is cast on `self` with `auto` (refused
otherwise as it loads), priced as any self spell. Its call is `[spell.effect] summon = {
creatures, count, most, minutes, spread }` (`SpellEffect::summon: Option<SpellSummon>`): the
pool's names, resolved by `spell_def::resolve_summons` once every creature type is known (one
step after `Content::load_summons`; known, never `player`), `count` a
`Roll` (at least one comes), `most`, `minutes`,
`spread` (default 2). Ladders: `[spell.grows] count` (a roll ladder, a summon's
alone; `Grows::roll()` is damage, heal, or count, so `SpellData::roll()`,
the rung, Empower, and the dice hooks act on the count as on any roll) and `degrees` on its
minutes (`SpellData::base_degrees()`: a status's severity, else a summon's minutes; Lingering's
`degrees` lengthen it). The menu and the `spells` dump say the count as `2 called` (JSON
`damage_type` `"called"`).

`World::cast_summon` (`world/summon_spell.rs`; the `Summon` arm of `cast.rs::perform`, which
made `Cast`'s needed fields, `pay_cast`, and `spell_landed` `pub(super)`), after the cast path's
own refusals (unknown, silenced, too little mana):

1. Refused, no time and no mana: without `allies.toml` ("Nothing answers your call."); on the
   overworld ("Nothing answers your call out here."); no free cell in `summon_cells(player,
   spread)` (the enemy summoner's, shared: "There is no room for them beside you."); the
   player's allies on the level at `[allies] most` unless one of this spell's own would give way
   ("You cannot lead more than this.").
2. Paid, `[spell.cast] message` said, `[spell.cast] noise` at the caster.
3. This spell's calls alive (`calls_of`: on any level and in the retinue, by
   `AllySource::Summoned { spell: Some(id) }`) at its `most` + `ally_mod`'s `most` (the owner's
   Q8: per spell; the global cap is data alone): the **oldest** (lowest id) give way until one
   more fits, departing as a lapsed one does here (`ally_departs`), removed silently elsewhere.
4. The count: the cast's roll (its rung, the dice hooks, rerolls) + `ally_mod`'s `count`, at
   least 1, held to the spell's room, the global cap's, and the free cells (recomputed after the
   giving way).
5. Each: `World::summon_draw(pool, &TalentMods::summon_bias)` (`world/summon.rs`, shared with
   the enemy summoner, which passes no bias: one `below(pool.len())` as before), then
   `make_ally` on the next cell, `AllySource::Summoned { spell: Some(id) }`, `until` = now + the
   cast's degrees (minutes) × (100 + `ally_mod`'s `duration_percent`) % (rounded down, at least
   1), or `None` with `ally_mod`'s `lasting`. `Actor::summoner` stays unset (the master is the
   ally's own field).
6. Said: "Two wolves come to your call." (`few`, by type, `and_list`); landed on the player
   (`spell_landed`), a success as any cast; a self spell trains no style.

**The draw's bias** (the owner's Q9: an item that guarantees fire, someday):
`TalentEffect::SummonBias { tag, weight }` → `TalentMods::summon_bias` (in the order taken). An
entry whose type carries a bias's tag weighs that weight (several multiply), the rest 1; one
`below(total)`; a total of 0, or every weight 1, falls back to the plain draw. No talent or item
gives one yet; an item affix would feed the same list.

**A slain summon** (`AllySource::Summoned`, the dev `ally:`'s too) leaves no corpse and drops
nothing: `fight.rs::creature_dies` returns before `remains`, loot, the corpse, and a cursed
room's rising; blood is spilled as any.

`TalentEffect::AllyMod` gains `count`, `duration_percent`, `lasting`, `most` (`AllyMods`, summed;
`lasting` any), read only in `cast_summon`; the wiki's talent text says each.

Tests: `world/summon_spell_tests.rs` on the built-in content's spell-circle hall with fixture
spells (rats; a rat or a bat) and talents: the call beside the caster on their side with its
`until`, the words and the noise; the oldest giving way past `most`; the clock; the refusals
(the global cap, with and without one giving way; the overworld) costing nothing; `ally_mod`'s
four reads; the bias (never a rat); a slain call leaving nothing; `study_difficulty`; the load
checks; a save round trip and determinism.

## The druid's calls (D1, #839)

The first shipped summon spells, enchantment, each `count = "1"`, `most = 1` (the owner's Q8: one
of each at once), five minutes, `degrees = "presence 14 8, 18 12, 22 16"`, a scroll each with a
high `study_difficulty` (the owner's Q7: very hard to learn), and no count ladder (with a cap of
one a bigger roll would only be shown, never come; Pack leader's `count` is the more):

| Spell | Level | Pool | `study_difficulty` | Druid level |
|---|---:|---|---:|---:|
| `call_of_the_wild` | 5 | `wolf` | 8 | 6 |
| `call_of_the_forest` | 20 | `bear` | 10 | 25 |
| `call_the_elements` | 35 | `greater_earth_elemental`, `greater_water_elemental`, `greater_air_elemental`, `greater_fire_elemental` (one drawn) | 12 | 40 |

The druid's abilities raise them, class talents with `ally_mod`: **Pack leader** (32; `count =
1, most = 1, duration_percent = 50`: two of each, a call brings both, half again as long) and
**Lord of the wild** (60; `lasting = true, most = 1`: three of each, until slain). The global
cap (`[allies] most`, 8) is data alone. The beasts: `docs/dev/creatures/beasts.md`. The menu's
and the `spells` dump's count (`spell_words.rs`) adds `ally_mod`'s `count` as the cast does
(`2 called` with Pack leader); a fixed roll is said as its number (`1 called`, not `+1`).

**Fights**: `fight::End::Yours(name)` is an ally of the player's they see: "Your wolf bites the
kobold.", "The kobold claws your wolf.", "Your wolf dies." (`perceive_events`' ends; the
player's own blow at one, `fight.rs`; a throw's, `hurl.rs`); a companion is `End::Named`, "Snel
bites the kobold."; unseen it is "something" as ever.

Tests: `world/caster_tests.rs` (the table to 60 by `levelup:auto`, the calls one of each and
the raises, the fight words and the departure), `content/spell_def.rs`'s shipped-spell table, `world/spell_list_tests.rs` (the calls counted
apart from a full caster's four to six spells). The balance probe's "Summons" table
(`balance_probe/summons.rs`, `docs/balance.md`).

## Experience, quests, resources

`fight.rs::dies`: the killer's master, if the killer is an ally of the player's, is credited as
the player would be: `World::earn_kill_at(creature, percent)` at `World::ally_share()` (the
player's class's `ally_share`, else `[xp] ally_percent`, else 100 without the file; the rule is
the master's class, never the ally's source) of what the player's own blow pays (the first-kill bonus and the level scaling included; `earn_kill` is
the test-only 100 % form), `tally_kill` (the quest log and Favoured prey's record), `judge_kill`
(alignment), `gain_resource(master, Build::Kill)`, and a Handle animal mark, all in full.
**Tending**: the Heal ability treats an ally beside the player (`world/heal.rs::allies_beside`,
`heal::of_allies`, the snapshot's `ally_ailments`; `heal:ID:DIR`; `docs/dev/status-effects.md`). `creature_dies`
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
  `(summoned; follow; retinue)`, `(companion; follow; named Snel, grade 12)`; the minutes are
  rounded up.
- JSON: `allies` (`id`, `creature`, `name`, `health`, `order`, `at` (the order's cell or null),
  `source`, `lasts` (minutes, or null), `retinue`, `here`), from `PerceptionView::allies()`
  (`AllyView`, the snapshot's `allies`: the player knows their own; its `source` carries a
  companion's data, `PerceptionView::companion()`); each seen actor's `ally: true`
  (`SeenActor::ally`, for A3's mark and look mode; `companion` for the player's own); the
  `abilities` layer's Allies rows; the `companion` layer and JSON (`docs/dev/companion.md`).

## The app (A3, #838)

All read from the perception snapshot; nothing in core.

- **The menu**: `ui/abilities_screen.rs` lists the Allies group as any other (`Source::Ally`'s
  header, the rows' states: nothing for Come to heel and Hold here, "at a place you see" for Set
  on, "no allies here" greyed). Come to heel and Hold here are `Aim::Nothing`: chosen, the
  order is sent at once (`input.rs::use_ability`). Set on is `Aim::Cell`: the aiming mode
  (`ui/target.rs`, `AimKind::Ability(Ability::SetOn)`), with no range (`reach` is `None`: any
  cell the player sees, `sight_line`), the prompt "Set on: Tab or direction keys aim, Enter uses
  it, Esc cancels.", loosed as `Ability::action_at` → `order:hunt:X,Y`. No `KeyCommand`.
- **Targets**: `target.rs::targets` (what Tab cycles and where an aim starts) leaves allies
  out, so Set on, a shot, or a spell starts on the nearest foe, never on the player's own wolf
  (the direction keys still reach any cell).
- **Look mode**: `ui/look.rs` says "Your dire warg (unhurt), on stone floor." (no stance word);
  a proper-named one "(your ally, unhurt)".
- **The ally mark**: `ui/glyphs.rs::ally_mark`, a short faint sage underline beneath the glyph
  (`docs/dev/rendering.md`; spec §4.2).
- The words of orders, arrivals, departures, and fights are core's (`world/ally.rs`,
  `fight::attack_message`); the app shows them as any message.
- Tests: the screen snapshots `abilities-menu-allies`, `abilities-menu-allies-none` (an ally in
  the retinue on the overworld), `set-on-aim`, `come-to-heel`; `look.rs::an_ally_is_yours`;
  `target.rs::set_on_aims_past_the_allies_at_a_foe`; `glyphs.rs`'s mark test.

## Tests

`world/ally_tests.rs` (a fixture of a wolf, a kobold, a deer, two levels, the shipped
`allies.toml` and `experience.toml`; the shipped content for the retinue, Beast-bond, and the
Handle animal marks): the dev action and its refusals (the cap), following within heel, the
master's stance (the kobold slain, the deer untouched), an ally's kill paying the master what
their own blow pays at `ally_percent = 100` and half at the shipped 50, tallying the quest log
in full either way, the share by class on the shipped content (the druid's and the ranger's 100,
the commoner's and the wizard's 50, a wolf's kill paying the druid twice the commoner), orders' time,
noise, words, and refusals, holding and Set on (there and back to heel; at an enemy), coming
along and being left behind, the clock here and on a level left, a rest undisturbed, a save
round trip, Beast-bond through the master (and nobody else), the Handle animal mark, the
retinue across the showcase's overworld (never drawn, not in the way, stepping out, the timed
one gone on the road, back into the retinue, saved), the dumps and the menu rows, and a replay
through all of it without divergence. `content/ally.rs` parses the file. The showcase's dump
snapshots are untouched: no scenario makes an ally.

## The risen

Rouse the dead (`world/corpse_spell.rs`, `docs/dev/spells.md` "Spells aimed at a corpse"): a summon
spell with `effect.rise` in place of `effect.summon`. A serving one is made by `World::make_ally`
with `AllySource::Summoned { spell }`, so it follows, takes orders, pays kills, and leaves nothing,
as a call does; `calls_of` and `gives_way` (`summon_spell.rs`) are its `most`, `[allies] most` its
cap, `ally_mod`'s `most`, `duration_percent`, and `lasting` read as there (`count` not). One that
rises berserk is **not** an ally: `Actor::ally` is `None`, its `faction_override` is
`berserk_faction`, and `Actor::risen` holds its clock (`duration_percent` counted, `lasting` not:
it always ends), read by `allies_lapse` and at `fight.rs::creature_dies`'s early return. It counts
toward neither `most` nor the cap, nor toward `hostiles_here` (a level's population). The dev
`actors` layer says `; risen, faction berserk, 9 min`.

## Escorts

An escort quest's character (#1139, `world/escort.rs`, `docs/dev/quests.md`): `AllySource::Escort
{ quest, faction }` (dump id `escort`), set in place as a charm is (no `make_ally`, no cap):
`faction_override` the player's (the old one kept), `stance` `CombatStance::Coward`, order
*Follow*, `until` none. Everything else is the ally core's: it follows, crosses, rides in the
retinue, pays kills, is healed and displaced. Its differences, each one line in `ally.rs`:
`ally_plan` gives it only a target beside it (`near(pos, 1)`) whatever its order; `order`'s Set
on takes the escorts out (`World::escorts_refuse`: "Eadgyth will not go looking for a fight.",
no time when no one else is left); `pack_phrase` and `companion.rs::ally_subject` name it as
`called` does. **Give** (`collar.rs`): `collar_wearer` and the "any to give to" check take kept
beasts or escorts; an escort goes to `World::give_escort` (`escort_slot`: `free_slot`, else the
first slot it `fits_properly`, its own kind first; `slots_taken` for two hands; what those slots
held leaves its pack for its cell; priced by `hand_over_one` with the slot's action; refused, no
time, for what fits no slot or a spirit weapon). The snapshot's `kept_beside` counts escorts
(the way is asked with two beside), `escort_beside` widens the Give list to `fits_escort`, and
`Ability::orders`' `kept` is true with an escort. Home (`escorts_home`): `escort_released` puts
the faction back, the neutral stance, a fresh `Mind` with its post where it stands; then the
quest's `thanks` and the giver's `done` (`docs/dev/quests.md`). An escort a quest makes on a
level (`waits`, `world/escort_waits.rs`) is an ordinary creature of its type until it joins. Tests:
`world/escort_tests.rs`.

## Charmed

Beguile (#1003; the spell types design §7 entry 34; `world/charm.rs`): a `hex` ray or bolt with
`[spell.effect] charm = { minutes, mindless, message, ends }` (`SpellEffect::charm:
Option<SpellCharm>`; no `status`, `push`, or `zone` beside it; `minutes`,
the base its `degrees` ladder grows, `SpellData::base_degrees`).
`cast.rs::cast_hex` hands it to `World::cast_charm` (one arm):

1. Refused, no time and no mana: without `allies.toml`; at one already an ally of anyone's
   ("The kobold is already at someone's side."); `hex_target`'s refusals (no one seen there, out
   of reach, not hostile); a `unique` type or an actor with an `npc` ("... is no one's fool.");
   an animated thing, a type with one of `mindless`'s tags (the shipped `undead`, `construct`,
   `vermin`), or a risen berserk ("... has no mind to charm."); the player's allies on the level
   at `[allies] most`. No check and no level limit.
2. Paid; `Actor::ally = Ally { master: player, source: AllySource::Charmed { spell, faction },
   order: Follow, retinue: false, until: now + degrees minutes }`, `faction` the actor's
   `faction_override` as it was, and `faction_override` set to the player's faction. That one
   assignment is the side, as for `make_ally`: its band turns on it, the AI's ally arm drives it,
   orders reach it, its kills credit the player (`ally_share`), the friendly mark, look mode's
   "Your kobold", and the dumps (`(charmed, 5 min; follow)`, JSON `source` `"charmed"`) follow
   with no branch. Its `Mind`, gear, statuses, and hit points stay as they were. Said: "The kobold
   takes you for a friend." (`message`). `ally_mod`'s summon fields are not read.
3. **The end**, `World::charm_ends`: `faction_override` put back, `ally` cleared, where it
   stands. A charmed summoner's calls (the plain enemy summoning, `summon.rs`, from the ally
   arm's `fight`) took the player's faction from it: every actor it summoned still on that
   faction goes back to the summoner's restored one. Its clock: `allies_lapse` → `ally_departs`, which takes this arm instead of
   `departs` for a charmed one ("The kobold shakes off the charm." where the player sees it,
   `ends`). Leaving: `follow_along` ends the charm of every charmed ally on the level left,
   silently, before it picks who comes along: it never crosses and never joins the retinue (the
   owner's Q12). Stair followers (`stair_followers.rs`) are chosen in `take_stairs` before
   `travel`, while it is still an ally, so it never follows as a hostile either.

A slain charmed creature is the creature it is: corpse, gear, and pack fall (`creature_dies`
skips only `Summoned`). Saves: the variant (`docs/dev/saves.md`). Tests:
`world/charm_tests.rs` (the side and the band, the kill credit, the end where it stands, the end
on the stairs with neither ally nor follower crossing, the faction put back, the refusals, the
dumps, a save and a replay's hashes). Content: `assets/data/spells/beguile.toml`, its scroll,
the witch's level-44 row (`talents/beguile.toml`); the wiki's "The charmed"
(`generated/charm-spells.md`, `xtask/src/wiki/allies.rs::charm_spells`).

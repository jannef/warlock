# Status effects, healing, and diseases

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08e-blood-and-status.md. Keep it current with the code, as CLAUDE.md says.

Status effects and healing (spec §8 "Status effects", "Healing"; M24, #138; **diseases, #139,
build on this**): statuses are data, `data/status.toml` (`content/status.rs`: `StatusDef` with
`seen`, `max`, `stacking`, `tick`/`damage`, `eases`, `regen`, `penalty`, `wound`, `cured_by`,
and the words `start`/`end`/`dies`/`death`; `Healing { resting }`; `OnHit` on a creature's
natural attack, `NaturalAttack::on_hit`; `Content::status`/`status_id`/`status_ids`/`healing`;
optional file; `StatusId` is an index, in the fingerprint). `crate::status` holds one actor's
effects (`Actor::effects: Effects`, at most one `Effect { status, severity, tick, wear, by }` per
status in status order; `Effects::add` applies the stacking rule, `penalty`, `regen`,
`is_valid_for`; `Effect::until_tick`/`until_eases`/`advance` are the timer math: ticks bank
`seconds × severity` degree-seconds). `world/status.rs` applies them: `World::afflict(actor,
status, severity, by, events)` (**the one way to give an actor a status**: food poisoning
from what the player eats (#137), traps, diseases call it; a status with a `damage_type`
(poisoned: poison) meets the actor's resistance there as a dose, docs/dev/combat.md "Damage
types": immune refuses it, else `damage::resist` on the severity), `World::cure(actor, means, events)` (eating
a remedy, an object with `cures`, `world/eat.rs::remedy`, which says what ended at once; the Heal
skill's bandage, #132), `World::after_blow` (wounds by `combat::big_hit` and criticals, of the
kinds the status's `wound.from` names, `Wound::caused_by(blow.kind)`: bleeding from slashing
and piercing; then `on_hit`
with one gameplay draw per chance below 100), `World::healing_rate` (min of the statuses'
`regen`, × resting for the player while `World::resting`, none for a Starving player, × a
special room's `regen`, × (100 + the regeneration affixes of what the actor wears and wields) %),
and `World::pass_time` → `live`, called by `run_turn` for every scheduled actor as the clock
moves: an event loop over the next tick, easing, hit point or point of mana back (mana's pace and
drains: `docs/dev/mana.md`), or change in the player's hunger (`docs/dev/hunger.md`), so
splitting time never
changes the result, the ticks' damage and then starving's through `hurt` (`Harm::Status`,
`Harm::Starving`: whichever takes the last hit point kills), deaths at their exact moment
(`dies(.., Some(status), when)`), the blood
drip hook `drip_blood` (#127's stains go there). The player's statuses beginning and ending are
`Event::Status` (said by `perceive_events`: begin = `say`, stops a rest; end = `note`, does not;
the rest loop keeps them from the turns it does not perceive). Presentation reads
`PerceptionView::own_effects` (status bar after HP, character screen `Status` row) and
`effects_of(actor)` (look mode; only statuses with `seen`), never severities. Saved and hashed
(`Actor::regen` counts hundredths of a second at the normal pace). The dev
action `afflict:ID` (`Action::Afflict`) gives the player a degree, for tests and captures. Tests:
`status.rs`, `content/status.rs`, `world/status_tests.rs`.

Buffs, hexes, and cooldowns (spec §8 "Status effects"; #227, design
`docs/design/progression.md` §4.1 B1) are statuses too, with four more optional fields
(`StatusDef`): `combat` (`CombatBonus { to_hit, damage, dodge, parry }`, the stances' numbers
and ranges), `cost` (percent of every price, `COST_PERCENT` 100 to 1000: slows only, so
`World::out_of_view_for` stays exact), `blocks` (`Vec<ActionKind>`, ids from `data/actions.toml`),
and `follows` (`Option<StatusId>`, resolved after the whole file, `resolve_follows`: no chain back
to itself; a status with it must end; nothing follows into or out of the disease status,
checked in `Content::load_diseases`). `penalty` takes positive numbers too (a raise). Each field
has one reader: `Effects::sharpen` in `Actor::stanced` (every attack profile, after the stance),
`Effects::defence` in `Actor::defence`, `Effects::slow` in `World::burdened` (after the burden
and Bloated), and `Effects::blocking` in `World::refuse_blocked` (`world/buff.rs`, the first line
of `player_acts`: `kind_of` maps the player's action to its `ActionKind`, a step by what it would
do (`move_kind`); a new `Action` variant gets an arm there, `None` for what is no action in the
world) and in `Ability::unavailable` (`Unavailable::While(status)`, reason "you are <name>", id
`while:<status>`, after an ability's own reasons). Endings go through one place,
`World::ended(id, effects, events)` (`world/buff.rs`), from `live` (worn off: `drop_spent`
returns the effects) and `cure`: it pushes the player's end events, afflicts each `follows` at
one degree with the ended effect's `by`, and calls `keep_hp_within_max` (now the larger of
`lasting_max_hp` and `max_hp`, so the hit points a raised Toughness allowed fall back as the
raise ends, while a passing penalty still keeps them). A cooldown is a status with `eases` and no
tick; the class abilities (I1–I4) give theirs with `afflict(player, status, 1, player)` and add
`Unavailable::While(cooldown)` in `Ability::unavailable` while it lasts. Creature AI does not read
`blocks` yet (no creature status blocks anything). Built-in example: `raging` (penalty St +2, To
+2, In −2; `blocks = ["read"]`; `follows = "exhausted"`) and `exhausted` (St −1), reached only by
`afflict:raging` until the barbarian's table. Tests: `world/buff_tests.rs` (custom statuses on
built-in content: combat on the player and a creature, slows and time, refusals and the greyed
shield bash, the rage's whole course, cures followed, saves and replays), the parse tests in
`content/status.rs`, `Effects` in `status.rs`.

Diseases (spec §8 "Checks", "Diseases"; M24, #139) build on status effects: every disease shows
as one umbrella status, Diseased (`data/diseases.toml`'s `status`, `Content::disease_status`; it
may have no `tick`/`eases`/`wound`/`cured_by`, no `on_hit` names it, and `afflict:diseased` is
refused), present exactly while a disease shows (`from_state` checks it). **Checks** are
`crate::check` (`check(rng, modifier, difficulty)`: `combat::roll_natural` + the effective
stat's modifier, natural 2 fails, 24 succeeds; `ways` for the odds): use it for any future
stat check. Data: `content/disease.rs` (`DiseaseDef`: `difficulty`, `check` stat, `incubation`
dice in days, `interval`, `damage: PerStat<Dice>`, `hp`, `recovery` (0 = never), `kills`,
`regen`, `caught` (named `SOURCES`: `rotten_food`), `wading` (`Wading { terrain, per_hour }`),
`cured_by`, words `onset`/`symptoms`/`recovered`; `DiseaseId` an index, in the fingerprint);
`on_hit` entries are `OnHit { what: Affliction::Status { .. } | Affliction::Disease(id),
chance }`. `crate::disease` holds one actor's infections (`Actor::diseases: Diseases`, at most
one `Infection { disease, left, shown, progressions, streak, by }` per disease: `left` counts
down to its onset, then each progression; known by name once `progressions > 0`).
`crate::stats::StatDamage` (`Actor::damage`: points off scores, capped so a score stays ≥ 1,
`take` says when it would not have; `mend` seconds) is read by `Actor::effective_stats` with the
status penalties, so every rule (and max HP through Toughness) sees it. `world/disease.rs`:
`World::expose(actor, disease, by)` (**the one way to expose**: a check, then `infect`),
`World::infect` (rolls the incubation; the dev `infect:ID`), `World::expose_to(actor, source)`
(eating a rotten corpse calls `expose_to(player, "rotten_food")`, `world/eat.rs`),
`World::wade` (called from `player_moves` with the step's seconds: one draw out of 360 000
unless sure), `World::disease_turn` (onset: `afflict` Diseased; progression: a check, damage in
stat order then `hp` through `hurt` (`Harm::Spared` unless it `kills`), hp cut to the new
max, death by Diseased for a disease that `kills`),
`World::end_disease` (recovery and cures; Diseased ends with the last showing one). `live` steps
to `disease::until_next` (disease turns and mending a point a day while none shows) and runs
`disease::pass` then the due turns after the effects at each moment. `World::healing_rate` takes
the min with `Diseases::regen`; `World::cure` also ends diseases by their `cured_by`. Messages
are `Event::Disease { actor, disease, news: DiseaseNews::{Onset, Worse(n), Recovered} }` (onset
and symptoms `say`, recovery `note`s; a batched rest turn with an onset or symptom stops the
batch). Presentation: Diseased via `own_effects`, `PerceptionView::own_diseases` (known ones:
the character screen's `Diseased` row, the `stats` dump layer). Saved and hashed.
Tests: `check.rs`, `disease.rs`, `content/disease.rs`,
`world/disease_tests.rs` (custom content with extreme stats, mirrored draws).

# Status effects, healing, and diseases

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08e-blood-and-status.md. Keep it current with the code, as CLAUDE.md says.

**Three fields of buffs** (#1002, `world/health/buff_field_tests.rs`; the spell types design §7 entries
24 to 26). `retaliates = { damage, damage_type }` (Fire cloak's `fire_cloaked`): `World::retaliate`
(`world/character/gifts.rs`), one clause at the head of `barbed`, so a melee blow that took hit points; it
goes through `spell_strikes` and `say_spell`, words by the damage type, or for a physical type
by the status's name (an innate `barbed_hide`: "The barbed hide stabs you."; #1128: a creature
type's `innate` statuses, `world/creatures/aura.rs`, put back each turn and left out of the look line) ("The fire burns the
kobold."), then `spell_type_lands` on a striker harmed; a kill pays the wearer, unlike the barbs.
A draw for the dice, and one for the rule's chance; none without such a status. `blows = { damage,
damage_type }` (Brand's `branded_blade`): `Effects::blows`, docs/dev/equipment.md. `wades_free`
(Water walking's `water_borne`): `Actor::wades_free` asks the statuses and the talents, read in
`World::step_price`. `ice_holds` (`water_borne` too): `Actor::ice_holds` asks the statuses; in
`World::tread` the ice holds its bearer at once, with no draw, and `World::shock` skips the cell
it stands on, so no plunge (`fall_in`) ever comes of it.

Status effects and healing (spec §8 "Status effects", "Healing"; M24, #138; **diseases, #139,
build on this**): statuses are data, `data/status/` (`content/status.rs`: `StatusDef` with
`seen`, `max`, `stacking`, `tick`/`damage`, `eases`, `regen`, `penalty`, `wound`, `cured_by`,
and the words `start`/`end`/`dies`/`death`; `Healing { resting }`; `OnHit` on a creature's
natural attack, `NaturalAttack::on_hit`; `Content::status`/`status_id`/`status_ids`/`healing`;
optional file; `StatusId` is an index, in the fingerprint; `per_action` statuses, Staggered,
Pinned, and Light-footed, ease by their sufferer's actions instead of time,
`World::spend_actions`, docs/dev/weapon-abilities.md; Surprised too, docs/dev/surprise.md, whose
`cost` is read through `World::status_cost`, and Stunned, the ascetic's Stunning strike's (#848);
one that ends there pushes its `Event::Status` end). `flat_footed` (Surprised, Stunned) leaves
its sufferer flat-footed, read by the field through `Effects::flat_footed` and
`World::off_guard` (docs/dev/surprise.md). `crate::status` holds one actor's
effects (`Actor::effects: Effects`, at most one `Effect { status, severity, tick, wear, by }` per
status in status order; `Effects::add` applies the stacking rule, `penalty`, `regen`,
`is_valid_for`; `Effect::until_tick`/`until_eases`/`advance` are the timer math: ticks bank
`seconds × severity` degree-seconds). `world/health/status.rs` applies them: `World::afflict(actor,
status, severity, by, events)` (**the one way to give an actor a status**; Hunter's bond shares a
buff from it, `afflict_one` being the unshared door, `docs/dev/companion.md`: food poisoning
from what the player eats (#137), traps, diseases call it; a status with a `damage_type`
(poisoned: poison) meets the actor's resistance there as a dose, docs/dev/combat.md "Damage
types": immune refuses it, else `damage::resist` on the severity; a creature type's `immune`
list refuses its statuses first, nothing drawn: the animated objects' poisoned and bleeding,
`docs/dev/animated.md`; and beside it a status with a `wound` table, Bleeding, takes only on a
creature type with blood: `CreatureDef::blood` `None` (`blood = "none"`) refuses it, the one
rule that reads blood, the spell types design Q7, `world/magic/spell_data_tests.rs`), `World::cure(actor, means, events)` (eating
a remedy, an object with `cures`, `world/health/eat.rs::remedy`, which says what ended at once; the Heal
skill's bandage, #132), `World::after_blow` (wounds by `combat::big_hit` and criticals, of the
kinds the status's `wound.from` names, `Wound::caused_by(blow.kind)`: bleeding from slashing
and piercing; then `on_hit`
with one gameplay draw per chance below 100), `World::healing_rate` (the slowest of the statuses'
and the showing diseases' `regen` while one is below 100, else the fastest, `Effects::regen`,
which counts the creature type's own `regen` (a troll's 400, #1131) among the statuses' paces, so
Burning's and Corroded's `regen = 0` stop it: Renewal's `renewing` 400, × resting for the player while `World::resting`, none for a Starving player, × a
special room's `regen`, × (100 + the regeneration affixes of what the actor wears and wields) %),
and `World::pass_time` → `live`, called by `run_turn` for every scheduled actor as the clock
moves (a creature with nothing to live through, `untouched_by`: whole, no statuses, diseases,
stat damage, resources, hunger, birthday, or perishable food, only has its healing banks emptied,
as the loop would, without its paces being worked out): an event loop over the next tick, easing, hit point or point of mana back (mana's pace and
drains: `docs/dev/mana.md`), or change in the player's hunger (`docs/dev/hunger.md`), so
splitting time never
changes the result, the ticks' damage and then starving's through `hurt` (`Harm::Status`,
`Harm::Starving`: whichever takes the last hit point kills), deaths at their exact moment
(`dies(.., Some(status), when)`); a status that `kills_at_max` (Stiffening, #1126) kills in
`afflict_one`'s degree step instead (`killed_at_max`: the status removed, then `hurt` for the
hit points left with `Harm::Status`, by the last degree's giver), and its `stages` (a line a
degree, the first its `start`, loaded so) are said by `afflict_one` on `Added::Worse`; a rider
with `check` (`Affliction::Status`'s, the basilisk's Toughness against 15) is resisted by
`World::resists` (`drain.rs`, the drain's check shared); the blood
drip hook `drip_blood` (#127's stains go there). The player's statuses beginning and ending are
`Event::Status` (said by `perceive_events`: begin = `say`, stops a rest; end = `note`, does not;
the rest loop keeps them from the turns it does not perceive). Presentation reads
`PerceptionView::own_effects` (status bar after HP, character screen `Status` row) and
`effects_of(actor)` (look mode; only statuses with `seen`), never severities. **A buff status
carries `seen`, whatever its source** (owner, 2026-11-01; docs/design/buff-circles.md §9): a
status that lends something (a positive `penalty`, a `combat` bonus, a `cost` under 100, a
`vision`, a `resist`, or a `light`) shows on creatures: `hasted` "quickened", `barkskin`
"bark-skinned", the potions' boosts, visions, and resistances by their names, `spell_ward`
"warded", `light_footed` "light-footed", `poised` "poised". Not buffs, never shown: `disguised` (a mimic's disguise),
`lurking`, the cooldowns, and harms. `content/circles.rs`'s `every_buff_status_is_seen` test keeps
it. **Circled statuses** (spec §4.8 "Spell circles"): `data/circles.toml` (`content/circles.rs`,
`CircleStatuses`, `Content::circles()`; the rows' statuses and schools fingerprinted) gives each
buff and hex spell's status a school, whose colour its spell circle about the actor is drawn in:
`lit` (evocation), `hasted`, `barkskin`, `branded_blade`, `water_borne` (transmutation), `blessed`,
`fire_cloaked` (abjuration); `ill_wished`,
`drowsy` (enchantment), `weakened` (necromancy), `sluggish`, `webbed` (transmutation). A circled
status must have `seen` (a load-time check: the circle shows only what look mode may say) and be
laid by a buff or hex spell (which makes its circle a buff's or a hex's), or name its own `kind`
(`"buff"` or `"hex"`) when no spell lays it: the lure quest's `bound` (enchantment, a buff; #1446,
`world/people/lure.rs`), permanent (no `eases`), on the crystal guardian once bound; a buff or hex
spell's status without a row has no circle. **The circle's light** (docs/design/buff-circles.md §8):
`circles::shed_lights` fills `StatusDef::light` at load with the school's `circle_<school>` preset
(`data/lights.toml`: radius 1, steady, luma 40 to 52 and UV 90 at the source) for every circled
status without a light of its
own (`lit` keeps `spell_light`; `fire_cloaked` and `branded_blade` `ember`, the one light of these that flickers). Nothing new reads it: `light/cache.rs` already pushes one source
per actor for its status lights, the widest, the first in status order on a tie (key `CARRIED |
LIT | id`, so the mask stays cached while the actor stands); severity and time left never scale
it, and it ends with the status. An actor that hides (`disguised`) sheds no status light. So a
circled actor is seen by normal eyes on its own cell in the dark, by low-light eyes a cell away,
plainly by UV eyes; creatures see the player the same way (a buffed sneak is seen), and it adds
to a light-blind player's glare. Tests: `world/magic/circle_tests.rs` (the alcove archer of
`--scenario circles` seen by its glow alone, the eyes, the tie, a cast's light and its end, the
disguise, the drow at noon) and `content/circles.rs` (the presets, a missing one refused, the
fingerprint). Saved and hashed
(`Actor::regen` counts hundredths of a second at the normal pace). The dev
action `afflict:ID` (`Action::Afflict`) gives the player a degree, for tests and captures. Tests:
`status.rs`, `content/status.rs`, `world/health/status_tests.rs`.

Buffs, hexes, and cooldowns (spec §8 "Status effects"; #227, design
`docs/design/progression.md` §4.1 B1) are statuses too, with four more optional fields
(`StatusDef`): `combat` (`CombatBonus { to_hit, damage, dodge, parry }`, the stances' numbers),
`cost` (percent of every price, above 0, a price of 0 being no time at all: below 100 only a spell's buff on
its caster may be (and the potion of quickness's Hasted on the player, and Sic 'em's Savage on a companion), `spell_def` refusing a hex's or a damage spell's, so creatures that disturb a rest are never
hastened and `World::out_of_view_for` stays exact), `auto_critical` (hits are criticals with no confirmation, a degree each: Savage, `world/allies/sic.rs`), `blocks` (`Vec<ActionKind>`, ids from `data/actions.toml`),
and `follows` (`Option<StatusId>`, resolved after the whole file, `resolve_follows`: no chain back
to itself; a status with it must end; nothing follows into or out of the disease status,
checked in `Content::load_diseases`), and `light` (`Option<LightId>`, a light of
`data/lights.toml` its bearer sheds while it lasts, the brightest of several: Lit, the Light
spell's, #589, and Branded, Branding smite's on the foe it wounds, #911; read by `LightCache` sources in `light/cache.rs`), `vision` (`Option<usize>`, a
profile of `data/vision.toml` lent while it lasts, read in `Actor::eyes` beside `worn_vision`,
bettering part by part; a potion's Owl-eyed, #567), and `resist` (`Vec<(DamageTypeId, i32)>`,
−100 to 75 and not 0, counted as gear in `Actor::resistances`; a potion's Warmed, #567).
`penalty` takes positive numbers too (a raise), and `per_degree = true` counts it once a degree
(`StatusDef::penalty_times`, read in `Effects::penalty_for`; a potion's boost; its value times
`max` stays in −10 to 10). A talent's `status_lasts` lengthens a status's easing per degree
after `status_eases` shortens it (`VigourMods::eases_every`), so a saved `wear` may exceed the
status's own `eases`: `Effects::is_valid_for` takes the sufferer's talents and allows the
slower pace. Each field
has one reader: `Effects::sharpen` in `Actor::stanced` (every attack profile, after the stance),
`Effects::defence` in `Actor::defence`, `Effects::slow` in `World::burdened` (after the burden
and Bloated), and `Effects::blocking` in `World::refuse_blocked` (`world/health/buff.rs`, the first line
of `player_acts`: `kind_of` maps the player's action to its `ActionKind`, a step by what it would
do (`move_kind`); a new `Action` variant gets an arm there, `None` for what is no action in the
world) and in `Ability::unavailable` (`Unavailable::While(status)`, reason "you are <name>", id
`while:<status>`, after an ability's own reasons). Endings go through one place,
`World::ended(id, effects, events)` (`world/health/buff.rs`), from `live` (worn off: `drop_spent`
returns the effects) and `cure`: it pushes the player's end events, afflicts each `follows` at
one degree with the ended effect's `by`, and calls `keep_hp_within_max` (now the larger of
`lasting_max_hp` and `max_hp`, so the hit points a raised Toughness allowed fall back as the
raise ends, while a passing penalty still keeps them). A cooldown is a status with `eases` and no
tick; the class abilities (I1–I4) give theirs with `afflict(player, status, 1, player)` and add
`Unavailable::While(cooldown)` in `Ability::unavailable` while it lasts; a creature's gaze
(`world/creatures/gaze.rs`) takes the `gaze` status as a summoning takes `summoning`, and its riders fall
through `World::rider`, the one place an `on_hit` entry is drawn and applied (a blow's too).
Averted (`world/creatures/avert.rs`) is no status: it is derived from the perception. Creature AI reads `blocks`
for steps only: a creature a status of its blocks `step` (Held, Drowning grip's, #269) never
steps or opens a door (`creature_steps`) and attacks its holder instead (`World::held_back`,
`world/character/lucky.rs`). **Engulfing** (docs/design/monster-dread.md §5.7, #1129) is data on holds,
no code: `engulfed` (a hold with `tick = 6`, `damage = 2`, blocking `step`, `shoot`, `throw`,
`read`; the charnel heap, the greater and elder water elementals, the brimwolf), `swallowed`
(the same at `damage = 4`; the nightworm), and `latched` (`tick = 12`, `damage = 1`, blocking
`step` only; the giant leech, the canoloth's tongue); the hold's grip is the rider's
`difficulty`, else 14, its turns `2d4`; its harm a tick the rider's `damage` while that holder
holds (`World::hold_harms`, read by the status pass; the rug of smothering's 1), else the
status's `damage`. A tick at n degrees comes n times as often, so the harm
is heaviest as the hold begins and eases with it. A blow and a spell still go from inside.
`world/creatures/split_tests.rs` checks the data, the tick, and the holder's death letting go. **Holds** (#431, `world/health/hold.rs`): a status with `hold` (`content::Hold`:
`check`, `difficulty`, `turns`; parse checks only that `turns` never rolls below 1; the data's
holds ease, keep the highest dose, and block `step`) is held by its effect's `by`. `World::afflict` draws a hold's degrees
from `turns` (`World::hold_turns`; the passed severity is ignored, so Drowning grip's
`afflict(.., 1, ..)` draws too), so it eases away within a few turns. `World::loosen_holds` ends
(through `ended`) every hold on the current level whose holder is gone, elsewhere, not adjacent,
blocked from stepping, or slowed by a status (`cost` > 100); `run_turn` calls it after the
player's action, after each creature's turn, and after the loop. `World::struggle`, called in
`player_acts` just before `refuse_blocked`, turns the held player's step (`kind_of` is `Step`)
into a check against the grip: the holder's natural attack's `Affliction::Hold { difficulty }`,
else `Hold::difficulty`; failed it returns `Spent::Turn(price_of(Step))`, made it removes the
status silently (its own message instead) and lets the step go on. **Escape artist** (#1015; the
owner: "an automatic skill check when player action is prevented/slowed because of status it can
fix"): `World::struggle` takes the action's `kind_of` and the first status `blocking` it. Trained
(`trains_effect(Effect::Escape)`), a hold's check reads `World::slipping` (the better of its stat
and Dexterity, each with `talents.check`, plus `Effect::Escape`) with Knowledge's `resisting` on
top. A status with `StatusDef::escape` (Webbed, Pinned; parse checks no `hold`, and `blocks` or
`cost` above 100, without which it is never struggled against) is checked the same way without Knowledge, trained only: made,
it is removed and `ended` says its `end`, and `None` lets the action go on; failed,
`Spent::Turn(price_of(kind))` and "You struggle, but you are still {seen}.". Untrained or
unmarked, `struggle` returns `None` with no draw and `refuse_blocked` refuses. With nothing
blocking, a trained player's first marked status with `cost` above 100 (none in the data today)
is checked and `None` is returned either way. Each check marks `StatUse::Struggled` and
`Use::Escape` (`World::struggled`). A second blocking status is refused this attempt and asks on
the next. Tests: `world/levels/escape_tests.rs`. A creature's `on_hit` names a
hold as `{ status = "held", chance = 10, difficulty = 15 }` (no `severity`; `difficulty`
optional; `OnHitSpec::resolve` gives `Affliction::Hold`, and `difficulty` on a non-hold status is
refused). Draws: the turns' dice as a hold begins or is renewed, two for a struggle. Tests:
`world/health/hold_tests.rs`, the parse tests in `content/status.rs`, `world/character/lucky_tests.rs`. A status that `flees` (Frightened, Hound's bay's, #270; Frightened also costs `to_hit` 3, fear's teeth, #1124, the player too) makes a creature step
away from its `by` instead while a step takes it farther (`World::frightened`,
`world/character/sign_picks.rs`, before `held_back`). A status that blocks `cast` also refuses spending
mana (`World::spend_mana`) and greys out a class active that costs mana (`Ability::unavailable`).
Silenced also blocks `song` (the bard's songs, `world/classes/song.rs`, #870). A song's statuses
(`inspired`, `inspired_greatly`) are `max = 3`, `stacking = "refresh"`: Music adds degrees, and
degrees are time, the combat bonus stays. The Lullaby (#873) gives `drowsy` (`max = 3`,
`stacking = "refresh"`) to the hostile hearers who fail its check, caused by the singer.
**Under a hex**: an actor has an effect whose status a `kind = "hex"` spell lays;
A familiar's touch gives hexes too, with its master as the giver (`docs/dev/familiar.md`).
`Content::hex_statuses()` (computed at load from the spells, in status order; `ill_wished`,
`weakened`, `sluggish`, `drowsy`, `webbed`) is the one definition, "of hers" adds `Effect::by`
the player. The witch's Cackle (#904, `world/classes/witch.rs`) reads it: each such hex on a creature
the player sees is `afflict`ed again at its severity plus a degree (`refresh`: higher, clamped,
`wear` 0). Malison (#905) reads it twice: its bite is `status_damage` rows naming the five
(any giver's hex; `malison.toml` lists them, a test checks it against `hex_statuses()`), and its
tithe (`world/magic/malison.rs`, from `dies`) checks a dying creature for a hex of the player's.
Built-in example: `raging` (penalty St +2, To +2, In −2; `blocks = ["read", "cast"]`; `follows =
"exhausted"`; no `eases`: the Rage resource brings it and ends it, `docs/dev/resources.md`) and
`exhausted` (St −1). A followed status must end: it eases, is cured, or a resource brings it
(`Content::check_follows_end`, after the resources load). `afflict:raging` is refused, as
`afflict:diseased` is. Tests: `world/health/buff_tests.rs` (custom statuses on
built-in content: combat on the player and a creature, slows and time, refusals and the greyed
shield bash, the rage's whole course, cures followed, saves and replays), the parse tests in
`content/status.rs`, `Effects` in `status.rs`.

Diseases (spec §8 "Checks", "Diseases"; M24, #139) build on status effects: every disease shows
as one umbrella status, Diseased (`data/diseases.toml`'s `status`, `Content::disease_status`; it
may have no `tick`/`eases`/`wound`/`cured_by`, no `on_hit` names it, and `afflict:diseased` is
refused), present exactly while a disease shows (`from_state` checks it). **Checks** are
`crate::check` (`check(rng, modifier, difficulty)`: `combat::roll_natural` + the effective
stat's modifier, natural 2 fails, 24 succeeds; `ways` for the odds): use it for any future
stat check. A disease check's modifier is `World::disease_modifier` (the stat's, plus the
talents' `disease_checks`, the worn `disease_check` affix effect, and the player's race's
`disease_check`), for exposure and progression alike. Data: `content/disease.rs` (`DiseaseDef`: `difficulty`, `check` stat, `incubation`
dice in days, `interval`, `damage: PerStat<Dice>`, `hp`, `recovery` (0 = never), `kills`,
`regen`, `caught` (named `SOURCES`: `rotten_food`), `wading` (`Wading { terrain, per_hour }`),
`cured_by`, words `onset`/`symptoms`/`recovered`; `DiseaseId` an index, in the fingerprint);
`on_hit` entries are `OnHit { what: Affliction::Status { .. } | Affliction::Disease(id),
chance }`. `crate::disease` holds one actor's infections (`Actor::diseases: Diseases`, at most
one `Infection { disease, left, shown, progressions, streak, by }` per disease: `left` counts
down to its onset, then each progression; known by name once `progressions > 0`).
`crate::stats::StatDamage` (`Actor::damage`: points off scores, capped so a score stays ≥ 1,
`take` says when it would not have; `mend` seconds) is read by `Actor::effective_stats` with the
status penalties, so every rule (and max HP through Toughness) sees it. `world/health/disease.rs`:
`World::expose(actor, disease, by)` (**the one way to expose**: a check, then `infect`; never
for Purity of body nor for a type whose `immune` holds the diseases' status, nothing rolled),
`World::infect` (rolls the incubation; the dev `infect:ID`), `World::expose_to(actor, source)`
(eating a rotten corpse calls `expose_to(player, "rotten_food")`, `world/health/eat.rs`),
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
`world/health/disease_tests.rs` (custom content with extreme stats, mirrored draws).

Treating oneself (spec §8 "Treating oneself", #549): the Heal skill's use. `crate::heal` holds
what may be treated (`Ailment::{Status, Disease}`, `treatable` in status then disease order,
`may_treat`, `own` and `of_allies` for `PerceptionView::own_ailments`, `noun`: a disease's name
once known, else "sickness"); `world/health/heal.rs` the try (`World::heal`, dispatched from
`player_acts`; the patient the player or, with `heal:ID:DIR`, the ally beside them that way,
`allies_beside`, #915: the healer's check, the patient's effect; the snapshot's `ally_ailments`
are the allies' list, computed each turn; `kind_of`
maps it to `ActionKind::Heal`, so a status may block it). A status is treatable by its `treat`
table (`content::Treat`: `difficulty`, `per_degree`, `stops`, `made`, `failed`): `stops` (bleeding) ends it through `ended` on a made check and
lets a failure be retried (as does every ailment: no try is spent); without it (poison) a made check lowers the severity a degree (ending
it through `ended` at none); any failure may be retried at once (#1012: the one-try rule went). A shown
disease is treated against its own `difficulty`: made, `streak + 1` while that stays below
`world/health/disease.rs::recovery_of` (Fever-proof's adjusted recovery; the one helper
`disease_turn` uses too), else `left` raised to the disease's interval (within
`longest_wait`, so `is_valid_for` holds). The ability is
`Ability::Heal` (`Aim::Ailment`, `Unavailable::NothingToTreat`); the game's choice of what to
treat is the one dialog (`ui/step_dialogs.rs::treat_dialog`). Tests: `world/health/heal_tests.rs`, the parse
cases in `content/status.rs`.

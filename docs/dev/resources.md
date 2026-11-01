# Resources

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08e-blood-and-status.md ("Resources"). Keep it current with the code, as CLAUDE.md says.

Alternative resources (#225) are integer pools beside hit points and mana that a character holds
once something grants them, several at once. This is the foundation; the class features of
`docs/design/progression.md` §4.18 move onto it in R1–R5 (#249–#253): Rage (R1), Focus (R2),
Openings (R3), Ki and Inspiration (R4), and the four resource talents (R5). Rage (R1), Focus
(R2), Openings (R3), Ki, and Inspiration (R4), and the resource talents (R5) have moved (below).

- **Granted by talents**: the talent effect `grants_resource { resource }` collects into
  `TalentMods::grants`; `world/focus.rs::hold` gives the actor each it lacks at its start (one
  held keeps its value). Called from `class::hand_out` (a talent taken or given) and
  `World::grant_class` (so a restored run gets one its talents grant), the only two places so
  far. `hold` takes the character level (the maximum a pool starts at may depend on it). The
  third grant place (#564): `World::ranks_news` (`fight.rs::grant_stamina`, where every rank the
  player gains is said) holds stamina for a player without it when a rank first teaches an
  active weapon ability, said "You learn to pace your breath." after the ability's line
  (`weapon_active_tests::the_first_active_rank_grants_the_pool`).
- **Focus** (R2, #250; `world/focus.rs`): Rapid shot grants it. `TalentEffect::RapidShot {
  resource, cost, saving }` (`TalentMods::rapid_shot`) is read in one place,
  `World::rapid_shot`, which `shoot.rs::fire` calls on the shot's price: with `cost` held it
  spends it and takes `saving` percent off the whole price (`percent_of`, at least 1), else the
  price stands. Volley (`world/volley.rs`) costs `resource = { id = "focus", cost = 6 }` on its
  talent, spent after its own checks (so a refused volley spends nothing), and has no cooldown
  (`TalentEffect::Volley { shots }` only); its price is `shot_price`, never Rapid shot's.
  Creatures never shoot through `fire`, so never use either. Tests: `world/focus_tests.rs`.

- **Data**: `content/resource.rs` parses `data/resources.toml` into `ResourceDef`s (`id`,
  `name`, `short`, `max`, `start`, `starts_full` (no `start` in data: granted at its maximum by
  data, `max_from` included, talents not), `max_from: Option<MaxFrom>` (`{ stat, per_levels,
  least }`, checked: a known stat, `per_levels` at least 1, `least` at most `max`;
  `MaxFrom::max` is the sum), `show_max`, `over_time: Option<OverTime>` (`Regenerates {
  every }` or `Decays { every }`, seconds a point), `resting`, `ends_at_zero`, `ended`,
  `builds: Vec<(Build, u16)>`, `status: Option<StatusId>`, the status it brings: parsed after the
statuses, refused if it `eases`). `ResourceId` is an index, in `Content::fingerprint` (its own
  group, after the damage types). `Build` is the closed list of events (`kill`, `hit`,
  `critical`, `struck`, `sneak_attack`; `Build::ALL` in wiki order). Loaded before the talents,
  whose class actives may cost one (`TalentDef::resource: Option<ResourceCost>`, data `resource =
  { id, cost }`, checked: a known id, a talent with a class active, a cost no more than `max`).
- **State**: `Actor::resources: Pools` (`resource.rs`): `Pool { resource, value, progress }` in
  resource order (`Pools::set` keeps it sorted), saved and hashed, skipped in the JSON while
  empty. Creatures never get any; nothing stops one but nothing grants one.
- **Changes** (`world/resource.rs`, each the one place):
  - `World::grant_resource(id, resource)`: held at `start` (or more, if it already holds more),
    capped at the maximum. `World::grant_resource_with_status(id, resource, by, events)` also
    begins its `status` (the Rage ability and the dev action call this one).
  - `World::gain_resource(id, on)`: every pool held that builds on `on` gains its amount (its
    `builds`, plus the talents' `resource_build`s, `TalentMods::resource_builds`), up to the
    maximum. Called in `fight.rs::dies` (`Kill`, for `by`, creatures' deaths only),
    `fight.rs::resolve` (`Hit` and `Critical` for the attacker, `Struck` for the target, on a
    `Strike::Hit`, before the damage, so a killing blow counts), and `ambush.rs::ambushed`
    (`SneakAttack`, when the sneak attack's dice are added). A new event is a `Build` variant and
    one call where it happens.
  - `World::spend_resource(id, resource, amount, events) -> bool`: refused when not held or
    short; spending to 0 ends one that `ends_at_zero`. `world/class_ability.rs::use_class` spends
    a class active's cost after its checks.
  - `World::end_resource` / `resource::ended` (the free function `World::live` uses while it
    holds the actor): removes the pool, pushes `Event::ResourceEnded`, and takes its `status` off,
    returning that effect for `World::ended` (its end said, what it `follows` begun; `World::live`
    adds it to the step's spent statuses). The other way round, `World::ended` calls
    `end_resources_of`: a status ended otherwise (a cure) ends the resource that brought it.
    Perception says `ResourceEnded` in the resource's `ended` words, if any (Rage has none: its
    Raging's end says it) (logged, not stopping a rest: `perceive.rs`, `rest.rs` keeps
    the event across unperceived turns) and the effects layer ignores.
  - Time: `World::live` (`world/status.rs`) takes `World::resource_paces(id)` (each pool's
    `Pace { max, rate }`) every step, `resource::until_next` as a moment, and
    `resource::pass` for the step (in the fast path too, when no point is due within it); the
    resources `pass` returns as ended are ended at that moment. Progress is dropped when there
    is nowhere to go (full, or empty) and kept at a standstill (a rest that stops it).
- **Hooks for R1–R5**: `Actor::resource_max(content, resource, level)` (the data's maximum, with
  what `max_from` adds from the effective stats and the character level, plus the talents'
  `resource_max`: Mighty rage's 30, Inner calm's +2, Deep lungs' +4; `World::holder_level` gives
  the level, the player's own or a creature's type's), `Actor::resource_start(content, resource,
  level)` (the data's start, or the maximum by data for one that `starts_full`, plus the talents'
  `resource_start`, at most the maximum: Battle trance's 18 rage; read by
  `World::grant_resource` and `focus::hold`), and `World::resource_rate(id,
  resource)` (the pace in percent: 0 while resting for `resting = false`, else 100 plus the
  talents' `resource_rate`, never below 0: Battle trance's −33, Inner calm's and Steady hand's
  +50, Patient blade's −50). A talent's `resource_build { resource, on, amount }` is a build only
  its holder has (Blood frenzy's 5 rage a kill; Patient blade's opening on `sneak_kill`, which
  `World::resolve` raises after a sneak attack's blow kills, after `sneak_attack`'s).
- **Affixes** (B1, #630; docs/design/affixes-systems.md §5): the same two hooks read what the
  actor wears for the pool an affix names (`Mods::resource(effect, id)`, `Actor::mods`, so an
  unidentified piece counts): `resource_max` adds to `Actor::resource_max` (Tireless for stamina,
  of Fury for rage; never below 0), `resource_rate` to `World::resource_rate` beside the talents
  (of Breath; its early return is now "no talents and nothing worn for that pool"). An affix
  never grants a pool: one who does not hold it gets nothing (a wizard's Tireless ring, any
  creature's gear, since creatures hold none). `World::keep_resources_within_max` (beside
  `keep_mana_within_max`, one call in `World::apply`) clamps every held pool to its maximum
  after the player's action, so taking the piece off is exact at once (`resource::pass` also
  clamps as time moves); it leaves progress alone. Stamina's third, `active_stamina`, is read
  in `Actor::weapon_stamina` (docs/dev/weapon-abilities.md). Tests:
  `world/affix_stamina_tests.rs`, with fixture affix files; the shipped ones are Tireless, of
  Breath, of Ease, of the Hunt, and the pools' of Fury, of Steadiness, of Stillness, of Guile,
  and of the Muse (P1, #635). The
  fourth, `resource_on_kill` (of the Hunt; E1, #632), is read in `World::siphon`
  (`world/siphon.rs`) at the kill point of an attack, beside of Siphoning's mana: each pool held
  that an affix names gains its sum, up to its maximum, when the holder's own blow, shot, or
  spell kills within `SIPHON_RANGE` (3) cells. `gain_resource(by, Build::Kill)` stays in
  `dies` for the data's `builds` and the talents' (any kill). Tests: `world/siphon_tests.rs`.
- **Rage** (R1, #249; `world/rage.rs`): the talent effect `rage { resource = "rage" }`
  (`TalentMods::rage: Option<ResourceId>`, `rage_status` its resource's status) makes the Rage
  ability grant the pool with Raging (`World::rage`); the cooldown is Raging and what it `follows`
  (`ability.rs::class_cooldown`). Blood frenzy is a `resource_build`, Mighty rage a
  `resource_max` beside its `raise_percent` and `status_eases`, Unstoppable `unstoppable {
  statuses, cost }`: `World::unstoppable` spends `cost` rage for the last hit point
  (`spend_resource`, so the rage ends at 0), and `World::live` numbs the statuses while Raging
  lasts. Raging `blocks` `cast`, so `World::spend_mana` refuses while it lasts. While the
  player's own Raging lasts they are in the berserker stance and cannot leave it, and its end
  leaves them aggressive (`world/berserker.rs`, #680; spec §8 "Stances"): `berserker::settle`
  runs where a status begins or ends and on load; `world/berserker_tests.rs` covers it.
- **Ki** (R4, #252): Flurry grants it (`grants_resource`) and carries
  `TalentEffect::FlurryCost { resource, cost }` (`TalentMods::flurry_cost`), read in one place,
  `World::flurry` (`free_blow.rs`, from `second_blow` on `Cue::First`, which now takes `events`
  for the spend): the second blow is decided only once its cost is spent; short, none. Quivering
  palm costs `resource = { id = "ki", cost = 5 }` on its talent and lost its cooldown
  (`TalentEffect::QuiveringPalm { status, severity }`); `use_class` refuses it with a weapon in
  hand before the spend. Tests: `world/ki_tests.rs`.
- **Inspiration** (R4, #252): Inspire grants it and its effect names what it spends instead of
  mana, `inspire { status, mana, resource, cost }` (`talent::Inspire::instead`, a
  `ResourceCost`; Virtuoso's `inspiring()` clears it, so its song costs nothing).
  `World::inspire` spends the resource when held (`spend_resource`), else the mana;
  `Ability::unavailable` checks mana (and the casting block) only when the resource falls
  short. Tests: `world/ki_tests.rs`.
- **Openings** (R3, #251; `world/openings.rs`): the thief's Sneak attack (level 1, #682) grants the pool
  (`grants_resource`); its build is the data's `sneak_attack`; Master thief's `finisher` effect is
  the class active `ClassActive::MasterThief` (`use:master_thief:E`, aimed, the cost `resource = {
  id = "openings", cost = 5 }` on the talent): a melee blow with `Profile::sure_critical` (every
  hit a critical, no confirmation draw; `combat::strike_with`). `use_class` refuses an aimed
  active with nobody hostile that way before it spends anything. Tests:
  `world/openings_tests.rs`, and `combat.rs`'s sure-critical test.
- **Stamina** (SP1, #560; the combat progression design §4.5): the last entry of
  `resources.toml` (file order is save order), `short = "Sta"`, `max = 10`, `max_from = { stat =
  "toughness", per_levels = 10, least = 5 }`, `regenerates = 12`, resting too, starting full.
  The granted talent `trained_wind` (`grants_resource`) is in the level-1 row of the warrior's,
  barbarian's, paladin's, ranger's, monk's, thief's, and bard's tables (`data/classes/`), so
  `class::grant` holds it at creation and `World::grant_class` on load. The weapon actives
  spend it (#564, `world/weapon_active.rs::pay_active`, through `spend_resource`, after their
  refusals); its `lacking` words ("you are short of breath") are how `Unavailable::Short` reads
  for it. Its talents (`[[system]] id =
  "stamina"`): Trained wind, Deep lungs (`resource_max` +4), Steady breathing (`resource_rate`
  +50, a weapon skill at 5). Tests:
  `world/stamina_tests.rs`; the parse and `MaxFrom::max` in `content/resource.rs`.
- **Abilities**: `Unavailable::Short(ResourceId)` ("not enough focus", `short:focus`; a
  resource's `lacking` words instead when it has them), checked last in the Class arm of
  `Ability::unavailable` (`ability.rs::short_of`), and in the weapon actives'
  (`ability.rs::weapon_unavailable`, with their discounted stamina).
- **Presentation**: `PerceptionView::own_resources() -> Vec<OwnResource>` (value clamped to the
  maximum) is all the UI gets: `ui::status_segments` (after MP, priority 1, so they give way
  before it; `show_max` picks `Focus 7/10` or `Rage 7`), the character screen's rows under the
  stance. Dumps: the header's `rage 11/20`, JSON `resources: [{id, current, max}]`, the dev
  `actors` layer's `resources rage 11/20 (progress)`. Never a creature's, never a pace.
- **Talents**: `[[system]] id = "resources"` in `data/talents.toml`, held to the coverage bar
  since R5 (#253): Battle trance, Inner calm, Steady hand, and Patient blade (the only
  class-limited picks), and the class abilities Rage, Blood frenzy, Mighty rage, Unstoppable,
  Rapid shot, Volley, Improved sneak attack, and Master thief. Tests:
  `world/resource_talent_tests.rs`.
- **Wiki**: `xtask/src/wiki/resources.rs` (the table) for `wiki/src/resources.md`.
- **Tests**: `content/resource.rs` (parsing, refusals, a resource's status, a followed status
  that never ends), `world/class_active_tests.rs` (Rage's course on the pool, Mighty rage's
  maximum, Blood frenzy's builds, Unstoppable's cost and the end it brings, Raging ended
  otherwise, no mana while raging), `world/buff_tests.rs` (Raging's scores and end), `world/resource_tests.rs` (granting,
  decay to the end and the bar going, time cut differently and resting, regeneration, spending,
  several at once, builds from events and from a real fight, a class active's cost, content
  checks of costs, dumps, saves, the hash, a replay), `ui/mod.rs` (the status bar), and the
  `resources` screen snapshot (the character screen and the status bar).

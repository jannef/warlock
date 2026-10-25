# Resources

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08e-blood-and-status.md ("Resources"). Keep it current with the code, as CLAUDE.md says.

Alternative resources (#225) are integer pools beside hit points and mana that a character holds
once something grants them, several at once. This is the foundation; the class features of
`docs/design/progression.md` §4.18 move onto it in R1–R5 (#249–#253): Rage (R1), Focus (R2),
Openings (R3), Ki and Inspiration (R4), and the four resource talents (R5). Rage (R1), Focus
(R2), Openings (R3), Ki, and Inspiration (R4) have moved (below).

- **Granted by talents**: the talent effect `grants_resource { resource }` collects into
  `TalentMods::grants`; `world/focus.rs::hold` gives the actor each it lacks at its start (one
  held keeps its value). Called from `class::hand_out` (a talent taken or given) and
  `World::grant_class` (so a restored run gets one its talents grant), the only two places.
- **Focus** (R2, #250; `world/focus.rs`): Rapid shot grants it. `TalentEffect::RapidShot {
  resource, cost, saving }` (`TalentMods::rapid_shot`) is read in one place,
  `World::rapid_shot`, which `shoot.rs::fire` calls on the shot's price: with `cost` held it
  spends it and takes `saving` percent off the whole price (`percent_of`, at least 1), else the
  price stands. Volley (`world/volley.rs`) costs `resource = { id = "focus", cost = 6 }` on its
  talent, spent after its own checks (so a refused volley spends nothing), and has no cooldown
  (`TalentEffect::Volley { shots }` only); its price is `shot_price`, never Rapid shot's.
  Creatures never shoot through `fire`, so never use either. Tests: `world/focus_tests.rs`.

- **Data**: `content/resource.rs` parses `data/resources.toml` into `ResourceDef`s (`id`,
  `name`, `short`, `max`, `start`, `show_max`, `over_time: Option<OverTime>` (`Regenerates {
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
- **Hooks for R1–R5**: `Actor::resource_max(content, resource)` (the maximum plus the talents'
  `resource_max`: Mighty rage's 30, Inner calm's +2) and `World::resource_rate(id, resource)` (the
  pace in percent: 0 while resting for `resting = false`; Battle trance's slower decay, Steady
  hand's and Inner calm's faster regeneration). A talent's `resource_build { resource, on,
  amount }` is a build only its holder has (Blood frenzy's 5 rage a kill).
- **Rage** (R1, #249; `world/rage.rs`): the talent effect `rage { resource = "rage" }`
  (`TalentMods::rage: Option<ResourceId>`, `rage_status` its resource's status) makes the Rage
  ability grant the pool with Raging (`World::rage`); the cooldown is Raging and what it `follows`
  (`ability.rs::class_cooldown`). Blood frenzy is a `resource_build`, Mighty rage a
  `resource_max` beside its `raise_percent` and `status_eases`, Unstoppable `unstoppable {
  statuses, cost }`: `World::unstoppable` spends `cost` rage for the last hit point
  (`spend_resource`, so the rage ends at 0), and `World::live` numbs the statuses while Raging
  lasts. Raging `blocks` `cast`, so `World::spend_mana` refuses while it lasts.
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
- **Openings** (R3, #251; `world/openings.rs`): Improved sneak attack grants the pool
  (`grants_resource`); its build is the data's `sneak_attack`; Master thief's `finisher` effect is
  the class active `ClassActive::MasterThief` (`use:master_thief:E`, aimed, the cost `resource = {
  id = "openings", cost = 5 }` on the talent): a melee blow with `Profile::sure_critical` (every
  hit a critical, no confirmation draw; `combat::strike_with`). `use_class` refuses an aimed
  active with nobody hostile that way before it spends anything. Tests:
  `world/openings_tests.rs`, and `combat.rs`'s sure-critical test.
- **Abilities**: `Unavailable::Short(ResourceId)` ("not enough focus", `short:focus`), checked
  last in the Class arm of `Ability::unavailable` (`ability.rs::short_of`).
- **Presentation**: `PerceptionView::own_resources() -> Vec<OwnResource>` (value clamped to the
  maximum) is all the UI gets: `ui::status_segments` (after MP, priority 1, so they give way
  before it; `show_max` picks `Focus 7/10` or `Rage 7`), the character screen's rows under the
  stance. Dumps: the header's `rage 11/20`, JSON `resources: [{id, current, max}]`, the dev
  `actors` layer's `resources rage 11/20 (progress)`. Never a creature's, never a pace.
- **Talents**: `[[system]] id = "resources"` in `data/talents.toml`, `pending` until R5's
  talents (and R1–R4's class features) carry it; Rage, Blood frenzy, Mighty rage, and
  Unstoppable (class abilities) carry it now.
- **Wiki**: `xtask/src/wiki/resources.rs` (the table) for `wiki/src/resources.md`.
- **Tests**: `content/resource.rs` (parsing, refusals, a resource's status, a followed status
  that never ends), `world/class_active_tests.rs` (Rage's course on the pool, Mighty rage's
  maximum, Blood frenzy's builds, Unstoppable's cost and the end it brings, Raging ended
  otherwise, no mana while raging), `world/buff_tests.rs` (Raging's scores and end), `world/resource_tests.rs` (granting,
  decay to the end and the bar going, time cut differently and resting, regeneration, spending,
  several at once, builds from events and from a real fight, a class active's cost, content
  checks of costs, dumps, saves, the hash, a replay), `ui/mod.rs` (the status bar), and the
  `resources` screen snapshot (the character screen and the status bar).

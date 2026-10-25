# Resources

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08e-blood-and-status.md ("Resources"). Keep it current with the code, as CLAUDE.md says.

Alternative resources (#225) are integer pools beside hit points and mana that a character holds
once something grants them, several at once. This is the foundation; the class features of
`docs/design/progression.md` §4.18 move onto it in R1–R5 (#249–#253): Rage (R1), Focus (R2),
Openings (R3), Ki and Inspiration (R4), and the four resource talents (R5). Focus has moved
(below); the others are granted only by the dev action `resource:ID` until theirs do.

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
  `builds: Vec<(Build, u16)>`). `ResourceId` is an index, in `Content::fingerprint` (its own
  group, after the damage types). `Build` is the closed list of events (`kill`, `hit`,
  `critical`, `struck`, `sneak_attack`; `Build::ALL` in wiki order). Loaded before the talents,
  whose class actives may cost one (`TalentDef::resource: Option<ResourceCost>`, data `resource =
  { id, cost }`, checked: a known id, a talent with a class active, a cost no more than `max`).
- **State**: `Actor::resources: Pools` (`resource.rs`): `Pool { resource, value, progress }` in
  resource order (`Pools::set` keeps it sorted), saved and hashed, skipped in the JSON while
  empty. Creatures never get any; nothing stops one but nothing grants one.
- **Changes** (`world/resource.rs`, each the one place):
  - `World::grant_resource(id, resource)`: held at `start` (or more, if it already holds more),
    capped at the maximum.
  - `World::gain_resource(id, on)`: every pool held that builds on `on` gains its amount, up to
    the maximum. Called in `fight.rs::dies` (`Kill`, for `by`, creatures' deaths only),
    `fight.rs::resolve` (`Hit` and `Critical` for the attacker, `Struck` for the target, on a
    `Strike::Hit`, before the damage, so a killing blow counts), and `ambush.rs::ambushed`
    (`SneakAttack`, when the sneak attack's dice are added). A new event is a `Build` variant and
    one call where it happens.
  - `World::spend_resource(id, resource, amount, events) -> bool`: refused when not held or
    short; spending to 0 ends one that `ends_at_zero`. `world/class_ability.rs::use_class` spends
    a class active's cost after its checks.
  - `World::end_resource` / `resource::ended` (the free function `World::live` uses while it
    holds the actor): removes the pool and pushes `Event::ResourceEnded`, which perception says
    in the resource's `ended` words (logged, not stopping a rest: `perceive.rs`, `rest.rs` keeps
    the event across unperceived turns) and the effects layer ignores.
  - Time: `World::live` (`world/status.rs`) takes `World::resource_paces(id)` (each pool's
    `Pace { max, rate }`) every step, `resource::until_next` as a moment, and
    `resource::pass` for the step (in the fast path too, when no point is due within it); the
    resources `pass` returns as ended are ended at that moment. Progress is dropped when there
    is nowhere to go (full, or empty) and kept at a standstill (a rest that stops it).
- **Hooks for R1–R5**: `Actor::resource_max(content, resource)` (the maximum: Mighty rage's 30,
  Inner calm's +2) and `World::resource_rate(id, resource)` (the pace in percent: 0 while resting
  for `resting = false`; Battle trance's slower decay, Steady hand's and Inner calm's faster
  regeneration). Rage's kill build is Blood frenzy's (R1) and needs a condition `builds` does
  not have yet.
- **Abilities**: `Unavailable::Short(ResourceId)` ("not enough focus", `short:focus`), checked
  last in the Class arm of `Ability::unavailable` (`ability.rs::short_of`).
- **Presentation**: `PerceptionView::own_resources() -> Vec<OwnResource>` (value clamped to the
  maximum) is all the UI gets: `ui::status_segments` (after MP, priority 1, so they give way
  before it; `show_max` picks `Focus 7/10` or `Rage 7`), the character screen's rows under the
  stance. Dumps: the header's `rage 11/20`, JSON `resources: [{id, current, max}]`, the dev
  `actors` layer's `resources rage 11/20 (progress)`. Never a creature's, never a pace.
- **Talents**: `[[system]] id = "resources"` in `data/talents.toml`, `pending` until R5's
  talents (and R1–R4's class features) carry it.
- **Wiki**: `xtask/src/wiki/resources.rs` (the table) for `wiki/src/resources.md`.
- **Tests**: `content/resource.rs` (parsing, refusals), `world/resource_tests.rs` (granting,
  decay to the end and the bar going, time cut differently and resting, regeneration, spending,
  several at once, builds from events and from a real fight, a class active's cost, content
  checks of costs, dumps, saves, the hash, a replay), `ui/mod.rs` (the status bar), and the
  `resources` screen snapshot (the character screen and the status bar).

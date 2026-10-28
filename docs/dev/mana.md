# Mana

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08e-blood-and-status.md ("Mana"). Keep it current with the code, as CLAUDE.md says.

Mana (spec §8 "Mana", #223) is a second pool beside hit points, built the same way, for magic to
spend (the progression design, #220, gives classes mana values, per-level gains, and talents that
cost or raise it, #242). Spells (`world/cast.rs`, docs/dev/spells.md), rods, the bard's inspire,
and creatures' summons spend it, all through `World::spend_mana`. **Every new system says in its
issue how it interacts with mana**, and hooks in at the one place below.

- **Pure rules**: `warlock-core/src/mana.rs`: `max_mana(stats, base)` (base plus the mental
  stats' modifiers, `MENTAL` = Intelligence, Willpower, Presence, at least 0),
  `mental_modifier`, `regen_seconds(willpower)` (`max(60, 360 − 12 × Willpower)`, its own
  constants beside combat's), `MAX_BASE_MANA` (1000).
- **State**: `Actor::mana`, `mana_base`, `mana_regen` (progress in hundredths of a second at the
  normal pace, like `Actor::regen`), saved and hashed, each left out of the JSON while 0, so an
  actor without mana saves and hashes as before mana existed. `Actor::max_mana(content)` (the
  effective stats, plus `max_mana` affixes through `Mods::worn`) and `lasting_max_mana` (the
  raised stats: what a status takes off comes back), beside `max_hp` and `lasting_max_hp`.
- **Data**: `mana` on races, classes, creature types (`CreatureDef::base_mana`; the player
  creature may not set it), and scenarios without a character (`ScenarioDef::mana`, refused with
  one); `Start::mana` is race plus class. `mana_level` in `data/experience.toml` (default 0).
  `[healing] mana_resting` (default 200), statuses' `mana_regen` (default 100) and
  `mana_damage` (default 0, needs a `tick`), diseases' `mana_regen` (optional, like `regen`),
  room effects' `mana_regen` (default 100). Affix effects `ItemEffect::MaxMana` (`max_mana`,
  1 to 100) and `ItemEffect::ManaRegeneration` (`mana_regeneration`, percent 1 to 300); the
  built-in affixes Mystic and of Clarity.
- **Hooks** (`world/mana.rs`, each the one place):
  - `World::mana_rate(id)`: the pace in percent, the twin of `World::healing_rate`: min of
    `Effects::mana_regen` and `Diseases::mana_regen`, × `mana_resting` while the player rests, ×
    the room's `mana_regen`, × (100 + `ManaRegeneration` + `TalentMods::mana_regen_percent` +
    Concentration's `skills::Effect::ManaRegen`) / 100 (the talent effect `mana_regen_percent`:
    Tidal, the Water-born's gift, and Clear mind, +25 each; the skill's through `Actor::mind`,
    below); resting's pace is raised by `TalentMods::mana_resting_percent` first
    (Meditation, +50); 0 for a Starving player.
    Burden, temperature, and stances never enter it.
  - `World::live` (`world/status.rs`) calls `mana::regain` beside `heal` (in the fast path too)
    and takes `mana::until_regained` as a moment, so a point back is exact however time is cut;
    the ticks' `mana_damage` sums into `Moment::drained` and `mana::drain` takes it (never below
    0, never a death).
  - `World::spend_mana(id, base) -> bool`: the one way to spend; refused when short. The cost
    is `Actor::mana_cost(content, base)` (`TalentMods::mana_cost(base, worn_off)`:
    `mana_cost_percent`, Frugal caster's −20, and the worn `mana_cost` (of Thrift, #629)
    summed before the one rounding and held to `talent::MAX_MANA_CUT`, 50 off; rounded to the
    nearest, at least 1 for a cost above 0; a creature's gear counts for its summons); an ability that
    greys out without the mana compares that, not its base. The player's spend of any is
    `StatUse::Cast` (`train_stat`: a Willpower mark at 20 %, `data/stats.toml`). Spells (the one
    cast path, `world/cast.rs`, docs/dev/spells.md) call it after every check with
    `Actor::spell_base` (the tier table's price, `SpellRules::mana`: tier, overreach, shape,
    resolution, armour; then Archmage's `spell_mana_percent`; the talents' cut is
    `spend_mana`'s), and `Actor::spell_cost` (`spell_base` through `Actor::mana_cost`) is the one
    spell cost hook, what the menu's `Unavailable::NoMana` compares
    (`ability::spell_unavailable`); abilities call it (and say why they could not); the dev `drain:N` (`Action::Drain`,
    `World::drain_dev`, through the same cut and training, saying the cost spent) is another
    caller.
  - **Concentration** (#668, `world/concentration.rs`): `Actor::mind(content, effect)` is a
    mana skill effect (`skills::Effect::ManaRegen`, `MaxMana`) through `Actor::skill_effect`
    (items' and talents' skill bonuses count), but 0 unless the actor is trained in a skill
    giving it (`Skills::trains_effect`). `Actor::max_mana` and `lasting_max_mana` end in
    `Actor::deeper_pool` (`mana::deeper(pool, bonus)`: at most `pool / 5`), so
    `keep_mana_within_max` cuts mana when an item or talent raising the skill comes off. Its
    marks: `skills::Use::Cast` (`cast`, 25 % in `data/skills.toml`), reported by
    `spend_mana` beside the Willpower mark (the dev `drain:N` too). Levels and Deep well raise `mana_base` through
    `Actor::grow_mana_base`, which fills what the maximum grew by (Concentration's share of the
    larger pool too); a level-up roll on the skill raises the maximum alone, like an item. Tests:
    `world/concentration_tests.rs`, `mana.rs` (`deeper`).
  - Deep well's mana by level (`max_mana_per_levels`, `TalentMods::mana_at`) is handed out like
    Hardy's hit points: `class::hand_out` at the pick, `World::level_up` as each level comes
    (no `mana_level` cap), into `mana_base` and `mana`.
  - `World::blood_magic` (`Action::UseTalent(TalentActive::BloodMagic)`, `use:blood_magic`; `Ability::BloodMagic`, the
    talent's active): `TalentMods::blood_magic` (`hp`, `mana`) hit points straight off `hp`
    (not `hurt`: no Protection, pain, training, or death, since it refuses at `hp <= cost`) for
    mana up to the maximum, a wait's price; refused, no time, without it, low on blood, or full.
    The menu lists it as usable whenever held (the rules refuse).
  - `World::keep_mana_within_max` after every player action (in `World::apply`, beside
    `keep_hp_within_max`): down to the lasting maximum when something that raised it comes off.
    `World::disease_turn` cuts mana to the new maximum after mental stat damage, like hit points.
  - `World::level_up_mana(before, after)` from `grant_xp`: `max(0, class mana ÷ 2 + the better
    of the raised Intelligence and Willpower modifiers)` (`mana::level_modifier`) per level up to
    `mana_level` (60; `World::level_mana`; the scenario's `mana` without a character). The race
    and class values and this formula are the progression design's (`docs/design/progression.md`
    §3.7). An ability short of mana is `Unavailable::NoMana` (a spell's through
    `ability::spell_unavailable`, which knows the caster level).
- **Presentation**: `PerceptionView::own_mana()` (current clamped to the maximum, and the
  maximum) is all the UI gets: the status bar's `MP 5/12` (`ui::status_segments`, priority 1
  right after HP, so it gives way after everything else of that priority, before the HP), the
  character screen's `MP  Mana  5/12` beside the hit points (`BESIDE`, as Protection beside
  Defence), the dump header's `mp`, the JSON dump's `mana: {current, max}`. The dev `actors`
  layer shows every actor's `mp` (and, for one with mana, its caster level, `cl 8`). Never a
  creature's mana or any pace in the game.
- **Creatures** (#525): nothing above is the player's alone. A creature's maximum, pace, and
  caster level read what it wears and wields through the same hooks (`Actor::max_mana`,
  `World::mana_rate`, `World::caster_level`, each on `Mods::worn`), so a goblin wizard's Mystic
  bone amulet or an animated panoply's rings give it a larger pool, filled as it is made
  (`travel::make_creature` sets `mana` after its gear), a faster pace, and a higher caster
  level; a summoner spends from that pool (`World::summons_at`, `spend_mana`). Creature spells
  are still announced only, so its caster level is read by nothing in play yet; the dev
  `actors` layer shows it. Creature gear never changes after it is made (stealing takes no worn
  thing), so no creature's mana needs `keep_mana_within_max`.
- **On a kill** (E1, #632; the affixes design §4, decision 7): `World::siphon(by, (slain, at))`
  (`world/siphon.rs`) raises the slayer's mana by its worn `mana_on_kill` (of Siphoning), never
  past `max_mana` (never lowering it either), and adds its worn `resource_on_kill` to each pool
  it holds (docs/dev/resources.md). The limit: it is called only at the kill point of an attack,
  `fight.rs::resolve` (a blow, a shot, a throw, a creature's spell; beside `train_kill`) and
  `spell.rs::spell_strikes` (the player's spells, scrolls, a breath), and counts only when the
  slain stood within `SIPHON_RANGE` (3) of the slayer (`dist2 <= radius2(3)`, a radius as every
  other); a self-kill never. A death by a status tick or a trap goes through `dies` alone and
  never reaches it; a summon's kill is the summon's. Creatures count the same. No draw; nothing
  worn, nothing changed. Tests: `world/siphon_tests.rs` (a blow, a shot at 3 and 4, a spell at
  3 and 4, an area's, poison, a trap, the maximum, a creature's kill, no pool, nothing worn).
- **Wiki**: `xtask/src/wiki/mana.rs` (race and class mana, regeneration by Willpower, what
  changes the pace) for `wiki/src/mana.md`; the special rooms table says the room's pace.
- **Tests**: `mana.rs` (the formulas), `world/mana_talent_tests.rs` (the five talents, the
  `cast` training), `world/mana_tests.rs` (maximum from data and stats,
  creature mana and serialization, regeneration by Willpower and resting, time cut differently,
  drains, statuses, diseases, starving, rooms, burden and stances doing nothing, the penalty that
  passes, disease damage, spending and `drain:N`, levels, magic items, saves, replays),
  `world/creature_magic_tests.rs` (a creature's pool, pace, and caster level with mana affixes
  worn and without; a summoner one short of its cost summons with a Mystic ring), and the
  content parsers' own tests.

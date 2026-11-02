# The priest's prayer

The priest's one prayer (#883, the priest design `docs/design/priest.md` §5, §6.1; spec §8i
"Class abilities"): a class active given at level 1 that answers what ails the priest most, as
far as their prayer talents reach, under a hidden strain. The dark powers' twists (§6.2),
Communion with the dark (§6.3), and Sanctuary (§6.4) are later slices (#884 and after).

## Modules

| Module | What it holds |
|---|---|
| `content/prayer.rs` | `PrayerRules` (`Content::prayer()`), parsed from `data/prayer.toml`: `strain`, `decay`, `fizzle_per_100`, the faces' bands (`holy`, `dark`: `Requirement`s), the `release` statuses, each face's `pray`/`fizzle`/`nothing` lines, and the uses in priority order with their words; `PrayerUse` (the closed list: disease, poison, wounds, release, mana, hunger, restoration), `Face`, `Faced`. Optional: without the file every prayer is refused. |
| `content/talent.rs` | `TalentEffect::Prayer { tier, title, uses, answers, all }`; `active()` maps only `tier = 1` to `ClassActive::Orison`, so `Content::class_active` finds the tier-1 talent. |
| `talent.rs` | `TalentMods::prayer: Option<Prayer>`: `uses` (the union), `answers` (the largest, at least 1), `all` (any), `tier` and `title` (the highest tier's). `has_active(Orison)` asks for a tier. |
| `content/class.rs` | `ClassActive::Orison` (`orison`, a `Cast`: Raging and Silenced block it through `Ability::unavailable`). |
| `world/orison.rs` | `Strain` (saved in `Progress::prayer`), `Actor::prayer_face`, `Actor::prayer_title`, `World::orison` (the one arm in `class_ability.rs::use_class`), the problems (`ails`), the fixes (`answer_prayer`), `World::prayer_state` (dev). |
| `perception.rs` | `PerceptionView::ability_name` / `ability_description`: the prayer by its highest tier's title as the face gives it, and that talent's description (the abilities menu, the `abilities` dump). |

## Data

`assets/data/prayer.toml` (the numbers the wiki states; hidden in the game) and the talents of
the priest's table (`classes/03-priest.toml`): `orison` (1, tier 1 Orison/Whisper, wounds),
`daily_bread` (6, hunger, and `purify`), `prayer_of_cleansing` (12, tier 2 Prayer/Invocation,
disease and poison), `prayer_of_release` (18, release), `litany_of_restoring` (25, tier 3
Litany/Rite, restoration), `litany_of_the_mind` (32, mana), `benediction` (40, tier 4
Benediction/Pact, two answers), `miracle` (60, tier 5 Miracle, `all`). Load checks: the numbers
in range (`strain` 1–1000, `decay` 10 s–1 h, `fizzle_per_100` 1–100), no band left unanswered by
both faces, the release statuses known, none twice, none bleeding, poisoned, the diseases'
status, or a status a resource brings; every use once. A prayer effect: `tier` 1–5 with a
`title` (and only a tier), `answers` 1–7, no use twice, and something given.

## The rule (`World::orison`)

1. Refused, no time and no strain: no prayer talent; no `prayer.toml` ("You have nothing to pray
   for."); no problem among the uses held (the face's `nothing`).
2. The problems, in the file's order, each only if its use is held (or `all`): disease, a disease
   **showing** (`Infection::shown`; a brewing one alone is no problem, so praying tells nothing
   of one); poison, Poisoned; wounds, Bleeding or `hp < max_hp`; release, any status of the list;
   mana, `mana < max_mana`; hunger, `Hunger::Hungry` or worse; restoration, stat damage or a
   drained point.
3. The strain is lazy: `Strain::now = value − elapsed / decay` (never below 0). Locked (a fizzle
   locked it and `now > 0`) → a fizzle with no draw. Else the chance is `now × fizzle_per_100 /
   100` (at most 100); above 0 it is **one draw** of the gameplay stream (`rng.chance`), at 0 none.
   Either way the strain becomes `now + strain` at the current time, `locked` = fizzled.
4. Fizzled: the face's `fizzle` line (the same locked or unlucky), a cast's time
   (`price_of(Cast)`). Answered: the face's `pray` line, then the first `answers` problems (all
   of them with `all`), each fixed and said in the face's words: disease, `end_disease` for every
   infection, brewing ones too; poison and bleeding, the statuses removed and `ended`; wounds,
   `hp = max_hp`; release, every listed status ended (as Stillness ends its); mana, `mana =
   max_mana`; hunger, nutrition up to `states.satiated` (never down); restoration, `World::mend`
   (the temple's: `StatDamage::mend_all`, `restore_drained`). With `all`, every resource pool
   held fills to `World::resource_max`. The `Status`-ended and `Disease`-recovered events the
   fixes raise are dropped: the prayer's line covers them (a status that `follows` still begins).
5. Nothing trains (no stat mark, no school), and no alignment moves.

The face (`Actor::prayer_face`): the player's band within `holy` → Holy, else within `dark` →
Dark, else Holy; Holy without alignment rules or the file. P1 gives the dark face its words and
titles only; its answers are the holy ones until #884's twists.

## Saved state, determinism

`Progress::prayer: Option<Strain { value, at, locked }>`, left out until the first attempt (a run
that never prays saves and hashes as before). The strain is a function of `GameTime`, so however
time is cut it passes the same. Draws: one per attempt only while there is strain and the prayer
is unlocked. Replays: `use:orison` at every tier, no new syntax.

## Dumps

`abilities`: `Orison (class): nothing` (the face's title of the highest tier). `progress` (dev):
`prayer strain 180 (locked), face holy, answers 1, uses wounds, hunger`. JSON with `--state`:
`prayer: { strain, locked, face, uses, answers }` (`answers` null with `all`).

## Tests

`world/orison_tests.rs`: wounds at 1 for a cast's time and no draw; the free refusal; each use at
its level; the priority order; a brewing disease alone; two answers at 40, all and every pool at
60; the strain's decay however cut; the fizzle chance with mirrored draws; the lock and its end;
the faces' titles and words; a silenced priest; save and replay. `content/prayer.rs`: the file's
order and numbers, a bad file refused, none optional. `world/caster_tests.rs`: the priest's table.
`world/curse_tests.rs`: Consecrate answers Neutral (`min = "neutral"`, the design's Q3).

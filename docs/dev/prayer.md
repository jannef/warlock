# The priest's prayer

The priest's one prayer (#883, the priest design `docs/design/priest.md` §5, §6.1; spec §8i
"Class abilities"): a class active given at level 1 that answers what ails the priest most, as
far as their prayer talents reach, under a hidden strain. The dark powers' twists (§6.2) and
Communion with the dark (§6.3) came with #884; Sanctuary (§6.4, #885), the priest's 50, is
hallowed ground: `docs/dev/special-rooms.md` "Hallowed ground".

## Modules

| Module | What it holds |
|---|---|
| `content/prayer.rs` | `PrayerRules` (`Content::prayer()`), parsed from `data/prayer.toml`: `strain`, `decay`, `fizzle_per_100`, the faces' bands (`holy`, `dark`: `Requirement`s), the `release` statuses, each face's `pray`/`fizzle`/`nothing` lines, and the uses in priority order with their words; `PrayerUse` (the closed list: disease, poison, wounds, release, mana, hunger, restoration), `Face`, `Faced`. Optional: without the file every prayer is refused. |
| `content/talent.rs` | `TalentEffect::Prayer { tier, title, uses, answers, all }`; `active()` maps only `tier = 1` to `ClassActive::Orison`, so `Content::class_active` finds the tier-1 talent. |
| `talent.rs` | `TalentMods::prayer: Option<Prayer>`: `uses` (the union), `answers` (the largest, at least 1), `all` (any), `tier` and `title` (the highest tier's). `has_active(Orison)` asks for a tier. |
| `content/class.rs` | `ClassActive::Orison` (`orison`, a `Cast`: Raging and Silenced block it through `Ability::unavailable`). |
| `world/orison.rs` | `Strain` (saved in `Progress::prayer`), `Actor::prayer_face`, `Actor::prayer_title`, `World::orison` (the one arm in `class_ability.rs::use_class`), the problems (`ails`), the fixes (`answer_prayer`, which calls the dark face's twists), `World::prayer_state` (dev). |
| `world/dark_face.rs` | The dark face's twists: `nearest_seen`, `pass_on_diseases`, `pass_on_poison`, `turn_hexes_back`; Communion: `World::communes` (the talent, its band, the Dark face) and `communing_here` (and an unholy effect where the player stands). |
| `world/rooms.rs` | `World::dwell`, the room status's one place: an unholy effect's status skips an actor that `communes`. |
| `perception.rs` | `PerceptionView::ability_name` / `ability_description`: the prayer by its highest tier's title as the face gives it, and that talent's description (the abilities menu, the `abilities` dump). |

## Data

`assets/data/prayer.toml` (the numbers the wiki states; hidden in the game) and the talents of
the priest's table (`classes/03-priest.toml`): `orison` (1, tier 1 Orison/Whisper, wounds),
`daily_bread` (6, hunger, and `purify`), `prayer_of_cleansing` (12, tier 2 Prayer/Invocation,
disease and poison), `prayer_of_release` (18, release), `litany_of_restoring` (25, tier 3
Litany/Rite, restoration), `litany_of_the_mind` (32, mana), `benediction` (40, tier 4
Benediction/Pact, two answers), `miracle` (60, tier 5 Miracle, `all`), and `communion` (9,
Communion with the dark: `{ kind = "communion" }`, `alignment = { max = "callous" }`, beside
Consecrate). Load checks: the numbers
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
Dark, else Holy; Holy without alignment rules or the file.

## The dark face (`world/dark_face.rs`)

The dark powers answer the same problems in the same order, each fix with a twist after it (no
draw of its own but a passed-on disease's incubation, `World::infect`'s roll):

| Use | Twist |
|---|---|
| disease | every infection cured (brewing ones too) is caught by the **nearest creature the player sees** (`nearest_seen`: on the current level, `Perception::sees_actors_at`, the least `dist2`, the lowest `ActorId` on a tie) through `World::infect` (no check: it incubates), `by` the player; one immune to the diseases' status by its type takes none; none seen, nothing more |
| poison | the Poisoned effect cured is laid on that creature at its degrees through `World::afflict` (`by` the player: immunity and resistance apply; a death of it is the player's kill) |
| release | each effect ended whose `by` is a creature (not the player) the player sees on the level is laid on it at its degrees, `by` the player; a hold (`StatusDef::hold`) lays Webbed instead |
| hunger | fed to `states.bloated − 1` (one short of Bloated), not `states.satiated` |
| wounds, mana, restoration | the words alone |

The use lines of `prayer.toml` say the twists ("The sickness leaves you, and finds another.");
the creatures' statuses begin silently (only the player's are told). Nothing here is a blow: no
stance changes, no alignment moves (a kill is judged as any kill of the player's).

## Communion with the dark

`TalentEffect::Communion` → `TalentMods::communion`. `World::communes(id)`: the actor holds it,
its band is one the Communion talent's `alignment` allows (as `World::answered` reads a holy
gift's; no rules, allowed), and its prayer face is Dark. Two reads, each where an unholy effect
(`RoomEffectDef::unholy`: cursed, draining, haunted; a room's or the level's) lies at the
actor's cell:

- `World::orison`: `communing_here()` → the attempt adds no strain (the chance and the lock still
  read the strain there is; the strain's `at` still moves).
- `World::dwell`: an unholy effect's `status` (the draining room's Ebbing) is not laid on one that
  communes; a holy effect's status still is.

A priest the holy powers answer holds the talent and it does nothing. `TalentDef::alignment` is
accepted on a talent with a holy class active or a band-gated passive (`TalentEffect::band_gated`:
Communion).

## Saved state, determinism

`Progress::prayer: Option<Strain { value, at, locked }>`, left out until the first attempt (a run
that never prays saves and hashes as before). The strain is a function of `GameTime`, so however
time is cut it passes the same. Draws: one per attempt only while there is strain and the prayer
is unlocked. Replays: `use:orison` at every tier, no new syntax.

## Dumps

`abilities`: `Orison (class): nothing` (the face's title of the highest tier). `progress` (dev):
`prayer strain 180 (locked), face holy, answers 1, uses wounds, hunger`. JSON with `--state`:
`prayer: { strain, locked, face, uses, answers }` (`answers` null with `all`).

## Wiki, probe

`wiki/src/prayer.md` (#886) states the numbers exactly from `xtask/src/wiki/prayer.rs`:
`prayer-strain` (the file's strain, decay, and fizzle), `prayer-fizzle` (the chance the 2nd, 3rd,
5th, and 10th prayer fizzles prayed at a fixed gap, worked out from the same numbers; the gap
that lets a prayer's strain pass whole is the last row), `prayer-uses` (the file's order, each
use's problem and both faces' fixes in the rules' words, `use_text` the one `match` on
`PrayerUse`, the lines, and the level and talent of each class table that gives it),
`prayer-levels` (every class table's prayer talents: the uses, the answers, the titles), and
`prayer-rites` (the praying class's Consecrate, Communion, and Sanctuary with their bands and
numbers). A use's row anchors as `prayer-ID` (`text::Term::PrayerUse`). The balance probe's
spell table gives the priest a line a level for the prayer (`balance_probe/tables.rs::
prayer_rows`): no roll, its title and what it answers.

## Tests

`world/dark_face_tests.rs`: the plague passed to the nearest seen creature (brewing ones too),
the holy face passing nothing on, the poison at its degrees and an immune skeleton taking none, a
hex turned on its seen caster and not an unseen one, a holder webbed, the brim; Communion: no
strain in a cursed room (and the strain for the holy face), no Ebbing in a draining one.
`world/orison_tests.rs`: wounds at 1 for a cast's time and no draw; the free refusal; each use at
its level; the priority order; a brewing disease alone; two answers at 40, all and every pool at
60; the strain's decay however cut; the fizzle chance with mirrored draws; the lock and its end;
the faces' titles and words; a silenced priest; save and replay. `content/prayer.rs`: the file's
order and numbers, a bad file refused, none optional. `world/caster_tests.rs`: the priest's table.
`world/curse_tests.rs`: Consecrate answers Neutral (`min = "neutral"`, the design's Q3).

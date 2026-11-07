# The witch's familiar

Developer notes for the familiar, the witch's small beast (spec §8 "Allies": "The familiar"; the
design is `docs/design/witch-familiar.md`, its §16 the owner's round overrides §15's defaults).
Slices **F1** (#929) and **F2** Call familiar and the imp from level 30 (#930); the sight and nose shared at 25 (#932, below) are in; F3 the
story's lines (#931) and the probe's row (#933) build on them. It stands on the ally core (`docs/dev/allies.md`) and the companion
(`docs/dev/companion.md`). Keep it current with the code, as CLAUDE.md says.

## What a familiar is

An ally of the core with `source = AllySource::Familiar(CompanionData)` and `until = None`.
`AllySource::kept()` is true of a companion and a familiar; `AllySource::data()`/`Actor::kept()`
read their `CompanionData`.

| Reads `kept` (both) | Reads its own |
|---|---|
| the name in the words (`called`, `End::Named`, `pack_phrase`, the Heal rows), growth (`grade_above`, `growth_protection`, `grow_companion`), always along (`follow_along`), the scent's growl, the status bar, the Character screen's `With` rows, look mode, `kept_beside` | `Actor::companion()`, `World::companion()` (Pack tactics and the ranger's later abilities, Tame); `Actor::familiar()`, `World::familiar()` (the touch, the harrying, `bring_familiar`) |

A familiar and a companion may be had at once (the owner, §16 Q8): each is at most one of its
source. Nothing is on `WorldState`: `World::familiar()` is derived from the actors, on any level
or in the retinue. A familiar's kill pays as any ally's (`[xp] ally_percent`): the witch has no
`ally_share`.

## The data

`assets/data/familiars.toml` → `content/familiar.rs::FamiliarRules` (`Content::familiars()`,
`None` without the file: no familiar can be made, and fixtures without it hash as before):
`names`, `touch` (a `kind = "hex"` spell with a status), `call_turns` (600), `bargain_kinds`
(`["imp"]`) and `bargain_level` (30), `[start]` (`rare_per_mille` 50, `rare`, `most_level` 2,
`[start.by_race]` one to three beasts a race, as `companions.toml`'s), `[arrives]` (a line for every
beast a start or a calling may draw, `every_kind()`). The load checks: known creatures, never
the player (`content/familiar.rs`), a hex `touch` that lays a status, lists and names not empty
(the draws index them); the rest is the author's (#1205: what shape a beast listed has; a kind
without an `arrives` line arrives unsaid; `world/familiar_tests.rs` checks the shipped lists have
theirs).
Its growth is `companions.toml`'s `[growth]`, one table. The new creatures are `cat`, `toad`,
`raven`, `hare` (`docs/dev/creatures/beasts.md`), wildlife in no population: **not** `tameable`,
since a familiar is called, not tamed.

**Familiar** (`assets/data/talents/familiar.toml`: tree `class`, `granted`, effect `familiar`,
systems `allies`, `status`, `spells`) is the witch's third entry at level 1
(`data/classes/10-witch.toml`). `TalentEffect::Familiar` → `TalentMods::familiar`.

## The start

`World::bring_familiar()`, beside `bring_companion` in `World::begin`, before the story is told.
Nothing without the talent, a character, `allies.toml`, `familiars.toml`, a list for the race, or
with a familiar already; nothing on `from_state`. `familiar::draw` is two draws on
`familiar/<spec>`; `familiar::name_for` one on `companion/name/<id>` (the kept beasts' stream, of
the familiars' names); on the overworld it starts in the retinue. "Tibb, your cat, winds about your
ankles."

## Call familiar

`ClassActive::CallFamiliar`, `use:call_familiar`, `Aim::Nothing`, had by whoever holds `familiar`:
`TalentEffect::Familiar` is its active as well as the passive that brings the first, so
`Content::class_active` finds the Familiar talent, and `active_name` calls the menu's row "Call
familiar". `world/familiar.rs`:

- `World::start_calling`, from `use_class`, after its checks (the ability, a status blocking `cast`,
  as Scribe's in `Ability::unavailable`): "Nothing comes." without `familiars.toml` or `allies.toml`;
  then `PerceptionView::calling_barred`, the one read the rules and the menu share:
  `Unavailable::FamiliarLives` (`snapshot.allies` holds a `Familiar`, any level or the retinue: "Your
  familiar lives yet.") and `Unavailable::Watched` (a creature the player sees whose stance as the
  view knows it is hostile: "Not with that watching."). All with no time. Else `Spent::Calling(turns)`.
- `World::calling`: `call_turns` turns of `ActionKind::CallFamiliar` through `keep_at`, as a study.
  `Ended::Came`/`Said` break it off, the time spent and nothing else lost; there is nothing in
  flight between actions, nothing saved.
- `finish_calling`: `calling_draw` on the **gameplay** stream (two draws: the jackpot, then the pick;
  `bargain_kinds` join the race's list, never the rares, from `bargain_level` of the character's
  class level, the first of `Character::class_levels`). `familiar_cell` (the player's cell on the
  overworld, where `make_familiar` puts it in the retinue; else `free_cell_near`, `None` when that
  has only the player's own cell: "Nothing comes.", the hour spent; `bring_familiar` then makes none
  either). The last turn's perception did not know the new actor, so it perceives once more.

No `watch_roll`: an encounter on the overworld does not come upon a calling as it does a camp (nor a
study). A start never draws the bargain's: `draw` is unchanged.

## The story

The companion chapter (`data/background/08-companion.toml`, `docs/dev/background.md`) is told to a
witch because `Witness::kept` holds; her beast's tale is the `only` line whose
`when = { familiar = [kind] }` fits `Witness::familiar` (`World::tell_background` reads
`World::familiar()`), one for every beast a race's list may give (the author's to keep: a beast without one is told
the plain line); the rare black hound and panther are told the plain line. A companion and a
familiar at once: each `only` line fits by its own condition, the chapter drawing among those that fit.

## The touch

`World::familiar_touch`, from `fight.rs::resolve` when a blow hit: a melee hit of a familiar's that
leaves the target alive. `spell_degrees` of `touch` on the master (the one function a cast uses:
the ladder, the school, the talents), then `afflict(target, status, degrees, Some(master))`. Before
that every other effect on the target with `by == master`, a status in `hex_statuses()`, and
`Stacking::Refresh` is afflicted again at its own severity: "hexes that stick". No draws, no mana,
no marks, no message of its own. Not an `on_hit` row: its giver would be the creature and its
severity fixed. A dev `ally:cat` is `Summoned` and hexes no one.

## The harrying

`World::quarry` stands where `ally_plan` asked `senses`, under each order's own `wanted`: a
familiar first takes the nearest it **sees** under a hex of its master's, then whatever `senses`
finds. Every other ally goes straight to `senses`.

## Two kept beasts beside

`crate::world::allies_beside` filtered by `kept` is the snapshot's `kept_beside` (ways, in
direction order). `input.rs::tend_way`: with more than one way and an ailment on one of them, Heal
enters `InputMode::Ability(Ability::Heal)` and asks "Tend to which side?"; a direction goes to
`Gameplay::tend(Some(dir))` and the wait key to `tend(None)` (the player), each its one ailment
at once or the treat dialog of several, and a bare `heal::DIR` for nothing to treat, so the rules
say why. With one way the old rows hold.

The collar (C3, #918, `docs/dev/companion.md`) reads the same `kept_beside()`: Give works on a
familiar as on a companion, and with more than one beside the player its list's choice then asks
the way (`InputMode::GiveWay`). A witch trained in Handle animal has both once she tames a beast
(`collar_tests::a_witch_who_tames_a_beast_names_the_way_to_each_and_it_replays`, the screen case
`giving_with_two_kept_beasts_beside`).

## Familiar's sight (#932)

The witch's level 25, beside Second sight (design §16 Q3; `assets/data/talents/familiars_sight.toml`:
tree `class`, `granted`, systems `vision`, `allies`). Its one effect, `TalentEffect::FamiliarSight`
→ `TalentMods::familiar_sight`, opens Hunter's bond's second viewpoint on the familiar
(`perception.rs::bond_view`; the viewpoint's rules, `docs/dev/companion.md` "Hunter's bond", stand
unchanged: creature sight's rule by its own eyes, cells lit as the player's eyes perceive them, the
nose shows only the foe's cell, no hidden trap or secret door, a dazzled player keeps it, nothing
from the retinue). Each talent reads its own beast: a witch's tamed companion lends no eyes, and
no ranger's bond reads a familiar. No `shared_buffs`: no buff crosses.

**The nose.** Of the beasts a familiar may be only `rat`, `polecat`, and `black_hound` have a
`nose`, and `nose` is only for a `tameable` type, which cat, toad, raven, and hare are not. Giving
them one would also set them hunting by scent and growling from level 1. So the file has one number,
`nose` (`familiars.toml`, 1 to 20; 6, a hound's): what the viewpoint uses when its kind's reaches
less or it has none (`bond_view`, `world::smelt_within`). It is read here alone: `ally_plan` and
the growl still read the type's `nose` through `smelt`, so her cat hunts by eye all its life.

## What the player sees

As the companion's (`docs/dev/companion.md`), a row or a segment per kept beast in actor id order.
The story is told the companion chapter's plain line until F3 (`Witness::kept` is what the
chapter's `when` reads; `Witness::companion`, what its `only` lines read, stays a companion's).
Dumps: the `familiar` layer and JSON `familiar`, one function with the companion's
(`dump.rs::Kept`); the `actors` layer's `(familiar; follow; named Tibb, grade 12)`.

## Determinism, saves, replays

Streams above; the touch, the harrying, and growth draw nothing; a calling draws on the gameplay
stream at its end. A run without a witch hashes as before; no shipped scenario, replay, or golden
has a witch. Saved: the variant inside `Actor::ally`; a calling is one action, nothing in flight. The
new creatures and the new action change the content fingerprint: development saves are invalidated
(the owner, #834: no aliases, no shims).

## Tests

`world/familiar_tests.rs`: the draw's odds and lists, the start and its words, a restored run, the
touch (giver, degrees, the other hexes kept going), a summoned cat, the harrying, growth and always
along, both kept beasts at once, the save round trip. `world/familiar_call_tests.rs`: the refusals
and the menu's reasons, the hour, the interruption, no free cell, the overworld, the draw's odds with
and without the imp, an imp as a familiar, determinism. `world/bond_tests.rs`, Familiar's sight: the
talent at 25, its eyes, the nose through a door and not a wall, its reach by the rules or the kind,
the retinue, each talent its own beast, no buff. `content/familiar.rs`'s load checks.
`caster_tests`'s witch table. `input.rs::tend_way`. The screen snapshots `abilities-menu-witch` (the
status bar) and `character-screen-familiar`.

## The probe

`balance_probe/familiar.rs` (#933, `docs/balance.md` "Familiar"): a human witch at each probe level with each of the five kinds in turn, set in place of the run's draw at its grade; the table is the familiar's hit chance against the monsters within two levels of hers, the hexes it lays a round, and what the same Evil eye costs her.

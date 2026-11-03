# The witch's familiar

Developer notes for the familiar, the witch's small beast (spec §8 "Allies": "The familiar"; the
design is `docs/design/witch-familiar.md`, its §16 the owner's round overrides §15's defaults).
Slice **F1** (#929, this file); F2 Call familiar and the imp from level 30 (#930), F3 the story's
lines (#931), the sight and nose shared at 25 (design §16 Q3, with Hunter's bond), and the probe's
row (#933) build on it. It stands on the ally core (`docs/dev/allies.md`) and the companion
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
`names`, `touch` (a `kind = "hex"` spell with a status), `[start]` (`rare_per_mille` 50, `rare`,
`most_level` 2, `[start.by_race]` one to three beasts a race, as `companions.toml`'s), `[arrives]`
(a line for every beast a start may draw). The load checks: known creatures of the plain-beast
shape (`content/familiar.rs`), an `arrives` for each kind drawn, names of at most 20 characters.
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

**A note for the collar (C3, #918):** Give is to ask the same way: read `kept_beside()` and, with
more than one, enter the direction ask (the owner, §16 Q4). The reads go through `kept()`, so the
collar works on a familiar as on a companion without more. Tame is not built either (C2): until it
is, no character has two kept beasts in play, and only tests reach the ask.

## What the player sees

As the companion's (`docs/dev/companion.md`), a row or a segment per kept beast in actor id order.
The story is told the companion chapter's plain line until F3 (`Witness::kept` is what the
chapter's `when` reads; `Witness::companion`, what its `only` lines read, stays a companion's).
Dumps: the `familiar` layer and JSON `familiar`, one function with the companion's
(`dump.rs::Kept`); the `actors` layer's `(familiar; follow; named Tibb, grade 12)`.

## Determinism, saves, replays

Streams above; the touch, the harrying, and growth draw nothing. A run without a witch hashes as
before; no shipped scenario, replay, or golden has a witch. Saved: the variant inside `Actor::ally`.
The new creatures change the content fingerprint: development saves are invalidated (the owner,
#834: no aliases, no shims).

## Tests

`world/familiar_tests.rs`: the draw's odds and lists, the start and its words, a restored run, the
touch (giver, degrees, the other hexes kept going), a summoned cat, the harrying, growth and always
along, both kept beasts at once, the save round trip. `content/familiar.rs`'s load checks.
`caster_tests`'s witch table. `input.rs::tend_way`. The screen snapshots `abilities-menu-witch` (the
status bar) and `character-screen-familiar`.

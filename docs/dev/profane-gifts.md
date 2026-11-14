# Profane gifts

Developer notes for the profane gifts: the permanent changes to the body that eating a layer
boss's corpse draws (#1471). The design is `docs/design/profane-gifts.md` (approved; its §13 lays
out the phases). Keep this file current with the code, as CLAUDE.md says.

What is in so far: the everlasting corpses (P1, `docs/dev/hunger.md`) and the gift machinery
(P2, #1625): the data, the tree, bestowing and rescinding, the dumps, the Talents page. Nothing
gives a gift in play yet: no meal draws one until P3 (#1626), so neither the spec nor the wiki
says anything of them.

## Data: `data/profane/`

One file per gift, `[[gift]]` (`assets/data/profane/README.toml` lists the fields): `id`, `name`,
`kind` (`boon`, `mixed`, `bane`), `tier` (`lesser`, `greater`, `dire`), `weight` (1 to 100,
within its tier), `description` (the Talents page's words), `line` (said as it takes), `effects`
(talent effects, `TalentEffect`), and optionally `undone`, `instead`, `excludes`, `replaces`,
`undoes`, `not_for`.

`content/profane.rs` (`profane::parse`, called right after the talent table) reads them into
`ProfaneTable` (`Content::profane()`): a `GiftDef` per gift (`kind`, `tier`, `weight`, the lines,
the relations as `TalentId`s, `not_for` race ids), found by its talent with
`ProfaneTable::gift(TalentId)`. Each gift is also **a talent**: `TalentBook::add` registers a
`TalentDef` of the `profane` tree (`data/talents/trees/170-profane.toml`, listed last; it has no free talent, so
nothing ever offers it) with `granted = true`, `flaw = (kind == bane)`, no needs, no systems.
So `TalentMods` reads its effects at their hooks as it reads any talent's, `may_take` refuses it
as it refuses every granted talent, and the talent ids of `data/talents/` keep their numbers (the
gifts come after them).

The loader's checks (§6.5): the id is no talent's already and unique; the weight 1 to 100; every
effect passes the talent effects' own check and is none of the forbidden ones (§6.4:
`profane::forbidden`, a hand-out that could not be taken back cleanly or a class's or a pick's
business: `max_hp_per_levels`, `weapon_ranks`, `extra_pick`, `starting_gold`, a trade,
`companion`, `xp_percent`, ...; `stat` is the one hand-out allowed); `not_for` names known races;
every relation names another gift; `excludes` is made symmetric (written once, on either gift:
the phases write it on the gift that lands later, so a partial table loads); `undoes` asks for an
`instead` line and every gift a `replaces` or `undoes` names for an `undone` line; `replaces` and
`undoes` together are acyclic. Gifts with no talent table (a fixture's content) load nothing;
gifts with a talent table that lacks the `profane` tree are an error.

The design's `thin_skinned` is `paper_skinned` (Paper-skinned) here: `thin_skinned` is the
Unicorn sign's flaw already, and a gift's id must be no talent's. The later phases'
`excludes` name `paper_skinned`.

Not yet on: the signatures' `only` and the campaign check (P3/P4), the tier coverage check (P6).

## In the world: `world/profane.rs`

- `World::may_take_gift(gift)`: §7.2's pool, what a draw may give: not held, not `excludes`-related
  to a gift held, not `not_for` the player's race. `World::gift_pool(tier)` counts it (open, all).
- `World::bestow_gift(talent)` (§7.3): a gift held is refused. An `undoes` with a gift it names
  held rescinds that gift and is **not** taken (the undone gift's `undone` line, then this one's
  `instead`); else every held gift its `replaces` names is rescinded, then it is taken: pushed
  into `Actor::boons`, the granted talents put back (`stars::grant`, as a restore does), its
  `stat` effects handed out, its `line` said, then "You are <name>.". Excludes and races are the
  pool's business, not this (a signature is sure, §7.3).
- `World::rescind(talent)` (§7.4): out of `Actor::boons` and `Actor::talents`, each `stat` effect
  given back in the negative, the `undone` line said. The hook the deferred good use (#196) will
  call.
- Stats (`shift_stats`): a rise is `Stats::gift` (score and potential, a Toughness rise bringing
  its hit points); a fall is `Stats::lessen` never below `DRAIN_FLOOR`, as the altar's wrath, stat
  damage kept within the score, hit points and mana kept within their maxima, never below 1 hit
  point. So rescinding Oak-hearted gives back exactly what it gave; Withered stops at the floor.
- A forgotten altar's Forsaken never draws a profane bane (`bestow_flaw` skips the gifts).

Saves: a gift rides in `Actor::boons` with no new field; `stars::grant` puts it back on every
restore and the state hash covers both lists. A save naming a gift the content lacks fails to
load, as any unknown talent does.

## What is shown

- The Talents page (`crates/warlock/src/ui/talents_screen.rs`): the Profane group last among the
  player's own, each gift "(profane)", a bane "(profane, flaw)" (`OwnTalent::profane`, the kind;
  `OwnTalent::boon` stays the altar's). A screen snapshot: `talents-screen-profane`.
- `talents` dump: `Oak-hearted (profane)`, `Withered (profane, bane)`; JSON `talents[].profane`,
  `"boon" | "mixed" | "bane"`, left out otherwise.
- `progress` dump (dev): `profane 1 held; pool lesser 4/4, greater 0/1, dire 2/2` (each tier's
  gifts a draw may give now, of all it holds).
- Dev actions (no time; `docs/dev/cli.md`): `gift:ID` gives a gift as a draw would (refused for
  one held, excluded, or barred to the race), `rescind:ID` takes one away.

## Tests

`content/profane.rs` (the tree, the symmetric excludes, the loader's refusals) and
`world/profane_tests.rs` (bestow; rescind reversing `stat` exactly; Withered at the floor;
`replaces`; `undoes` with and without the boon; `excludes` both ways; `not_for`; a save; the
`progress` row).

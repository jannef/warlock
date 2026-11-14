# Profane gifts

Developer notes for the profane gifts: the permanent changes to the body that eating a layer
boss's corpse draws (#1471). The design is `docs/design/profane-gifts.md` (approved; its §13 lays
out the phases). Keep this file current with the code, as CLAUDE.md says.

What is in so far: the everlasting corpses (P1, `docs/dev/hunger.md`), the gift machinery
(P2, #1625): the data, the tree, bestowing and rescinding, the dumps, the Talents page; and
Gnægan's meal (P3, #1626): the signatures' `only`, a profane corpse's `gives` and `profane`, the
draw, and the deed. Gnægan is the one profane corpse so far; the other nine bosses' signatures
come with P4 (#1627). The spec (§8 "Hunger") and the wiki (`wiki/src/profane-gifts.md`) say what
eating one does.

## Data: `data/profane/`

One file per gift, `[[gift]]` (`assets/data/profane/README.toml` lists the fields): `id`, `name`,
`kind` (`boon`, `mixed`, `bane`), `tier` (`lesser`, `greater`, `dire`), `weight` (1 to 100,
within its tier), `description` (the Talents page's words), `line` (said as it takes), `effects`
(talent effects, `TalentEffect`), and optionally `undone`, `instead`, `excludes`, `replaces`,
`undoes`, `not_for`, and `only` (a signature).

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

**Signatures** (§5): a gift with `only = "<creature>"` weighs 0 (`GiftDef::drawn` is false: no
draw and no pool counts it) and is that creature's corpse's `on_eat.gives`, sure.
`profane::check_signatures` (after `load_hunger`, when creatures and their `on_eat` are in) ties
the two both ways, keeps a campaign's signature in its own campaign (`GiftDef::campaign`, from
the file's path: Rat-king's court is
`assets/campaigns/ascmearc/data/profane/rat_kings_court.toml`, since shared data names shared
creatures only), and refuses a `replaces` or `undoes` naming one.

Not yet on: the campaign check that every endless boss has `gives` and `profane` (P4), the tier
coverage check (P6). Rat-king's court lacks its `kin_to { tag = "vermin" }` until P5.

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
- `World::profane_meal(on_eat)` (§4): for an `on_eat` with `gives` and `profane`, from
  `eat.rs::eat_fresh` after the usual corpse steps and before it feeds: the signature
  (`bestow_gift`), then `profane.rolls` draws, each given before the next is drawn, then
  `player_did(Deed::EatProfane)` (`eat_profane`, −300 in `data/alignment.toml`; the band's note
  and a paladin's emptied zeal through `shift_alignment`).
- `World::draw_gift(rolls)` (§7.2), on `state.rng`: the tier by `rolls.tiers` (one weighted draw),
  then one of the tier's drawn gifts the player `may_take_gift` by `weight` (another); an empty
  tier gives way up, then down (lesser → greater → dire; dire → greater → lesser); all empty is
  `None`, and the meal says "Nothing more of it will take in you.".
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
`world/eat_tests.rs` (Gnægan's meal in §4's order, an interrupted meal, the fallbacks, the empty
table, the deed in full and diminished, a paladin's zeal, the loader's profane-corpse and
signature refusals), `world/profane_tests.rs` (bestow; rescind reversing `stat` exactly; Withered at the floor;
`replaces`; `undoes` with and without the boon; `excludes` both ways; `not_for`; a save; the
`progress` row).

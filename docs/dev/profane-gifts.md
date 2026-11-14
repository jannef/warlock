# Profane gifts

Developer notes for the profane gifts: the permanent changes to the body that eating a layer
boss's corpse draws (#1471). The design is `docs/design/profane-gifts.md` (approved; its §13 lays
out the phases). Keep this file current with the code, as CLAUDE.md says.

What is in so far: the everlasting corpses (P1, `docs/dev/hunger.md`), the gift machinery
(P2, #1625): the data, the tree, bestowing and rescinding, the dumps, the Talents page; and
Gnægan's meal (P3, #1626): the signatures' `only`, a profane corpse's `gives` and `profane`, the
draw, and the deed; and the other nine signatures (P4, #1627): every layer boss of Grundlēas
leaves a profane corpse; and the body (P5, #1628): the six effects of §6.4 and the ten gifts that
use them or relate to them (below); and the rest of the table (P6, #1629): all forty drawn gifts,
Twice-born's rising, and the tier coverage check. The design (#1471) is complete. The spec (§8 "Hunger") and the wiki (`wiki/src/profane-gifts.md`) say what
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
`undoes` together are acyclic; and the whole table covers each tier (`check_coverage`): every
tier holds at least one drawn boon, mixed gift, and bane, so a fixture that replaces
`data/profane/` must bring one of each (`content/profane.rs`'s test `load` adds them). Gifts with no talent table (a fixture's content) load nothing;
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

The ten signatures live in `assets/campaigns/ascmearc/data/profane/`, one file each, beside their
bosses' `on_eat` in `data/creatures/` (rolls and tier odds per boss: design §7.1, 1/1/1/2/2/2/2/3/3/3
from Gnægan to Nīðdraca). Where a boss's old corpse gave `train` marks its signature gives `stat`
points instead; the tags' default marks (`[corpses.by_tag]`: giant, hag, caster, vermin) still
come with the corpse, as for Gnægan. Līgetdrȳ's corpse teaches Decipher script (`skills`, up to
60) beside its gift. The campaign check (`content/layer_boss.rs::pin_lairs`): every keeper of an
endless branch leaves a corpse that is `everlasting` and profane (`gives` and `profane`); a
portal branch's warden is exempt.

The signatures' resistances are written against #1620: 100 in the total is 75 effective with 25
to spare (Rime-blooded's cold under Hate-wyrm's heart's −25 is still 75), the wyrms'
vulnerabilities stay (D8), and no gift makes the player immune.

**Sign actives from a gift.** Hate-wyrm's heart carries the Wyvern pick's `fire_breath` effect,
larger (12d8 over 10 cells, the `drained` cooldown at 1 degree). `SignMods::add` keeps one breath,
the one with the larger mean damage (`Roll::mean_x2`), whichever was taken first, and one stare,
the larger `severity`. The abilities menu lists such an active under `Source::Profane` ("profane")
when no non-gift talent of the player's gives it (`Ability::source_for`, `OwnAbility::source`),
else under the birth sign.

**The table** (P6, #1629; design §6.1–§6.3): forty drawn gifts, 14 lesser, 12 greater, 14 dire
(by weight 45/27/27 % boons, mixed, banes in the lesser tier, 45/28/26 in the greater, 37/25/38 in
the dire). Every effect is an existing talent effect; a gift may carry a negative `natural`
(Glass-boned), `skill` (Reeking, Stone-eyed), `checks` (Craven-hearted), `initiative_bonus`
(Palsied), or `resist`. The relations of P6, each written once: `scaled` excludes
`paper_skinned`, `glass_boned`, and `hollowed`; `glass_boned` excludes `thick_blooded`;
`wyrm_blooded` replaces `ember_veined`; `palsied` undoes `adder_quick` (its `instead`: "The palsy
takes your quick hands, and nothing more."). `hill_thewed` has no undoer. Ember-veined sheds the
`ember_glow` light (`data/lights.toml`: dim red, radius 2) through `inner_light`; a full moon
brightens it to the sign's `ember_full` as it brightens every inner light (the flavour's rule).
Wing-shouldered bars the cloak slot (its wings, "Your wings are in the way."). Stone-eyed carries the
Basilisk's `stare` with the sign's numbers (one stare with the sign's, the larger).

**Twice-born** (design §6.3, §11, D7): the Phoenix's `rise_from_ashes` with its numbers, under a
once of its own. `world/character/stars.rs::rise_from_ashes` asks `rising_sources` who gives the rising: a
talent that is no gift (the sign's) and a gift. The sign's rising takes `risen`, then
`risen_twice` with Second rising; then a gift's takes `profane_risen` (`PROFANE_RISEN`). So a
Phoenix who is Twice-born rises once by each (twice with Second rising, then once more), and
anyone else who is Twice-born once. The `talents` dump marks the gift `(profane, spent)` once its
once is gone (`OwnTalent::spent`, per talent: a gift's reads `profane_risen`, the sign's the other
two).

**The body** (P5, #1628; design §6.4). Six talent effects, any talent's to use, each read at one
hook (`docs/dev/progression.md`'s table has them all): `bars_slot` and `opens_slot` (the new
`slots` area, `talent/slots.rs`; read in `world/items/body_slots.rs`, `docs/dev/equipment.md`
"Barred and opened slots"), `unarmed_on_hit` (`offence`: bare hands' `on_hit`, `Actor::melee`),
`vision` (`senses`: `WorldState::base_vision`), `kin_to` (`beast`: Beast-friend's rule by tag,
`world/allies/beast_friend.rs::calms`), and `max_mana_percent` (`magic`: `Actor::mana_scaled`). The
gifts: lesser `soft_footed`, `barrow_eyed`, `horned`, `hoofed`, `mere_eyed`; greater `clawed`,
`many_fingered`, `dim_eyed`; dire `deep_eyed`, `fell_minded`; and Rat-king's court's
`kin_to { tag = "vermin" }`. Their relations are written on the later of each pair (`hoofed`
excludes `soft_footed`, `many_fingered` excludes `clawed`, `dim_eyed` excludes `barrow_eyed` and
`mere_eyed`, `deep_eyed` replaces those two and excludes `dim_eyed`). The design's `bars_slot`
shape gave a `mark` and `refused`; each gift's here: Horned's horns, Hoofed's hooves, Clawed's
claws. What falls, and its words, is the code's (`World::shed_unworn`): "Your iron helm no
longer fits, and falls to the floor." for a slot barred, "Your silver ring slips from a finger
that is no longer there." for a slot no longer the player's (only rings are gated to a class
today; an aristocrat's own extra ring stays when Many-fingered goes).

## In the world: `world/character/profane.rs`

- `World::may_take_gift(gift)`: §7.2's pool, what a draw may give: not held, not `excludes`-related
  to a gift held, not `not_for` the player's race. `World::gift_pool(tier)` counts it (open, all).
- `World::bestow_gift(talent)` (§7.3): a gift held is refused. An `undoes` with a gift it names
  held rescinds that gift and is **not** taken (the undone gift's `undone` line, then this one's
  `instead`); else every held gift its `replaces` names is rescinded, then it is taken: pushed
  into `Actor::boons`, the granted talents put back (`stars::grant`, as a restore does), its
  `stat` effects handed out, its `line` said, then "You are <name>.". Excludes and races are the
  pool's business, not this (a signature is sure, §7.3).
- `World::rescind(talent)` (§7.4): out of `Actor::boons` and `Actor::talents`, each `stat` effect
  given back in the negative, the `undone` line said, then what is worn in a slot no longer the
  player's falls (`shed_unworn`: an `opens_slot` gone; a `bars_slot` gone frees its slot, nothing
  put back). The hook the deferred good use (#196) will call.
- A gift given ends the same way: what is worn in a slot it bars falls (Horned's helm).
- Stats (`shift_stats`): a rise is `Stats::gift` (score and potential, a Toughness rise bringing
  its hit points); a fall is `Stats::lessen` never below `DRAIN_FLOOR`, as the altar's wrath, stat
  damage kept within the score, hit points and mana kept within their maxima, never below 1 hit
  point. So rescinding Oak-hearted gives back exactly what it gave; Withered stops at the floor.
- `World::profane_meal(on_eat)` (§4): for an `on_eat` with `gives` and `profane`, from
  `eat.rs::eat_fresh` after the usual corpse steps and before it feeds: the signature
  (`bestow_gift`), then `profane.rolls` draws, each given before the next is drawn, then
  `player_did(Deed::EatProfane)` (`eat_profane`, −300 in `data/alignment.toml`; the band's note
  and a paladin's emptied zeal through `shift_alignment`).
- `World::draw_gift(rolls)` (§7.2), on its keyed stream `profane/gift/player`: the tier by `rolls.tiers` (one weighted draw),
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
  `OwnTalent::boon` stays the altar's). A screen snapshot: `talents-screen-profane`. The
  character screen's Stars line lists the sign's gifts only, never a profane gift.
- `talents` dump: `Oak-hearted (profane)`, `Withered (profane, bane)`, `Twice-born (profane,
  spent)`; JSON `talents[].profane`,
  `"boon" | "mixed" | "bane"`, left out otherwise.
- `progress` dump (dev): `profane 1 held; pool lesser 14/14, greater 11/12, dire 14/14` (each tier's
  gifts a draw may give now, of all it holds).
- Dev actions (no time; `docs/dev/cli.md`): `gift:ID` gives a gift as a draw would (refused for
  one held, excluded, or barred to the race), `rescind:ID` takes one away.

## Tests

`content/profane.rs` (the tree, the symmetric excludes, the loader's refusals) and
`world/health/eat_tests.rs` (Gnægan's meal in §4's order, an interrupted meal, the fallbacks, the empty
table, the deed in full and diminished, a paladin's zeal, the loader's profane-corpse and
signature refusals), `world/character/signature_tests.rs` (each of the nine corpses: its gift, its stats, its resistance totals;
Līgetdrȳ's Decipher script; the two wyrms summed and capped; one breath, the larger; the
`Fire breath (profane)` row), `world/levels/layer_boss_tests.rs` (every keeper's profane corpse, the
campaign check), `world/items/body_slots_tests.rs` (P5: Horned's helm falls and the equip is refused
with no time; Many-fingered's ring, and its fall when the gift goes, an aristocrat's kept;
Clawed's bleeding; Rat-king's court's vermin neutral until struck; Deep-eyed's darkvision eyes
on a human, replacing Barrow-eyed; Fell-minded's quarter more mana), `content/profane.rs`'s
`the_loader_refuses_a_bad_body_effect`, the screen snapshot `equipment-horned`, `world/character/profane_tests.rs` (bestow; rescind reversing `stat` exactly; Withered at the floor;
`replaces`; `undoes` with and without the boon; `excludes` both ways; `not_for`; a save; the
`progress` row), `world/character/gift_table_tests.rs` (P6: the fixture run that eats all ten keepers, its
`progress` pools after each meal as the insta snapshot `profane_ten_meals`; a Phoenix who is
Twice-born rises twice, Twice-born alone once; Wyrm-blooded over Ember-veined and its glow;
Palsied undoing Adder-quick and nothing more; Scaled's excludes; Wing-shouldered's cloak), and
`content/profane.rs`'s `the_full_table_loads_with_every_tier_and_kind` and
`the_loader_refuses_a_tier_without_each_kind`.

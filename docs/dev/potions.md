# Potions

Spec §8 "Potions" (`spec/08g-magic-items.md`); the combat progression design §10
(`docs/design/combat-progression.md`); #565 (Q1: the kinds `heal`, `cure`, `mana`, `harm`);
#567 (Q2: `boost`, `vision`, `haste`, `resistance`, their thirteen statuses, Lasting draughts,
Cauldron-wise).

## Objects and data

- A potion is an object of class `potions` (`!`) with an `[object.drink]` table:
  `content/potion.rs` (`DrinkSpec` → `Drink` on `ObjectDef::drink`, resolved in
  `ObjectSpec::drink_of`, as `wand_of` and `rod_of` are). A potion is `stackable` and nothing
  else (no slot, attack, missile, food, `read`, wand, rod, or magic `kind`); `level` and
  `frequency` make it loot, never magic (`check_base`, the `loot` flag in `resolve`).
- `Drink { does: Draught, colour, marks }`. `Draught` is the kind with only its fields:
  `Heal { dice, cures }`, `Cure { cures, diseases }`, `Mana { dice }`,
  `Harm { dice, status }` (status required: hunger is loaded after objects),
  `Boost { status, severity }` (the status must raise a stat), `Vision { status }` (it must
  have `vision`), `Haste { status, severity }` (status always `hasted`, the Haste spell's; data
  writes only `severity`), `Resistance { status }` (it must have `resist`); a severity is 1 to
  the status's `max`. `Draught::lends` is the status and degrees of the four that lend one.
  `marks` are
  `(Stat, count)` in stat order, 1 to `MAX_POTION_MARKS` each.
- Content checks (`content/mod.rs`): a drink's `cures` must be named in some status's or
  disease's `cured_by` (with food's `cures`); potions of one `colour` share one `fg`;
  `data/potion_labels.toml` says `{colour}` and has at least as many labels as potion types
  (`check_potion_labels`). Poisoned's `cured_by` has `antidote` for the antidote.
- Files: one per potion in `assets/data/objects/` (`potion_of_healing`, `potion_of_greater_healing`,
  `potion_of_full_healing`, `potion_of_antidote`, `potion_of_cure_disease`, `potion_of_mana`,
  `murky_draught`; #567: `potion_of_might`, `_grace`, `_vigour`, `_wit`, `_will`, `_presence`
  and their `greater_potion_of_*`, `potion_of_cats_eyes`, `_owls_eyes`, `_moths_eyes`,
  `potion_of_quickness`, `potion_of_warmth`, `_frost`, `_earthing`, `_proofing`). The witch's old `red_potion` and `blue_potion` are now the potions of
  healing and mana (ids renamed; saves change).

## Drinking: `world/potion.rs`

- No action of its own: `Action::Eat` (`eat:b`, `eat:floor:a`) reaches `World::eat`, which
  hands a stack with a `drink` table to `World::drink_potion` before any hunger check
  (`world/eat.rs`). What is neither food nor a potion is refused with no time ("You cannot
  consume a dagger."). `world/drink.rs` is the healing spring's Drink ability, a different thing;
  the two share only the `drink` action price (`data/actions.toml`, one turn).
- Order: priced at the start; one taken (`take_draught`: pack or floor; Thrifty drinker's keep
  is one gameplay draw, only with the talent); "You drink the {title}." by the name the player
  knew; `learn_scroll` and "It is a potion of healing." for news; in a shop `drank_goods`
  (`world/shop.rs`, `ate_goods`'s twin, naming the potion as the player knows it); the effect;
  `train_stat_times(StatUse::Drank, Some(stat), count)` per mark (sure marks, no draw); the
  KEPT line if kept. Returns `Spent::Turn`.
- Effects: `Heal` rolls its dice on the gameplay stream, raises hp to at most the maximum, then
  `World::cure` per means; `Cure` cures per means and, with `diseases`, `end_disease` for every
  infection; `Mana` raises mana to at most the maximum; `Harm` rolls degrees and `afflict`s
  them. `Boost`, `Vision`, `Haste`, and `Resistance` go through `World::lend`: `afflict` the
  status at its degrees; its start line is the event's when it begins, said here when an
  existing one grows stronger, and "Nothing more comes of it." when nothing changes (boosts
  stack `highest`, the others `refresh`). The statuses (`data/status.toml`, the potions' group
  after Barkskin): six boosts (`penalty` with `per_degree`, `eases = 600`, `max = 2`), three
  visions (`vision`, `eases = 1800`), four resistances (`resist`, 50 %, `eases = 1800`); what
  they do is the statuses' (docs/dev/status-effects.md). Status and disease beginnings and ends are not said here: they are the action's events,
  which perception reports ("You feel sick.", "The poison wears off.").
- Creatures do not drink.

## Labels and colours

- `scroll_label.rs`: `Labels::draw` draws potion words **after** the rods' from the same
  `identification` stream, so scrolls', wands', and rods' labels (and runs' rolls) are unchanged.
  `data/potion_labels.toml` is the scroll labels' format with `name = "{label} {colour} potion"`.
- `Lore::labelled_names` puts the type's `colour` in `{colour}`; `Lore::fg` keeps a labelled
  potion's own `fg` (the type's colour, its family's hue) instead of the file's single `fg`.
- Known types live in `WorldState::known_scrolls` with scrolls, wands, and rods: learned by
  drinking, by `World::identify` (the scroll of identify, the appraisal service, dev
  `identify`), and from a chosen character's kit (`World::begin` adds kit potions as it adds
  kit scrolls).
- The item description (`magic/describe.rs::potion_lines`): a known potion's effect in words
  ("Heals wounds at once, and cures bleeding"), never dice; an unknown one "You do not know what
  this potion does."

## Sources

- Loot by `level` and `frequency` (`magic::loot`), as scrolls.
- Shops (`assets/data/shops.toml`): general stores and provisioners stock and buy the potions
  class (by loot frequency, so healing and antidote are the common ones); the `alchemist` kind
  stocks and buys potions only, a stack of potions of healing always first. Hand-made shops
  (Cēaptūn) are unchanged.
- Quest rewards and level piles name potions by id (the Watch's reward, the hub's pile, the
  still room vault).

## Talents (system `potions`)

- **Thrifty drinker** (`potion_kept { chance }`, `TalentMods::potion_kept`): read in
  `take_draught`.
- **Lasting draughts** (Vigour, Toughness 13; #567): `status_lasts` (50) on the six boosts and
  three visions, `TalentMods::status_lasts`, read in `TalentMods::eases_every` (each degree
  takes half again as long to ease). The design called it "Strong constitution", a name a
  diseases talent already has.
- **Cauldron-wise** (the witch's class ability at 12; granted): `potion_heals` (20,
  `TalentMods::potion_heals`, read in the `Heal` arm) and `status_lasts` (20) on the six boosts.
  The Cauldron talent (level 50) is about corpses and does not overlap.
- **Nose for draughts** (`potion_smell`, `TalentMods::potion_smell`): `World::smell_potions`,
  called by `pick_up` after "You pick up ...": one line per unknown potion type picked up, by
  its kind (`smell`), never its type.

## UI

- The Consume list (`ui/object_screen.rs`, `ObjectList::Eat`) lists food and potions, floor
  first, grouped by class (Food, then Potions). Text snapshots: `consume-list`,
  `potion-description`, `potion-description-unidentified` (`ui/screen_snapshots.rs`).

## Tests

- `world/potion_status_tests.rs` (#567): boosts raise and wear off, greater ones ease to
  common ones, `highest`, visions change `Actor::eyes` and what is seen in the dark,
  resistances add as gear and never reach immunity, haste, Lasting draughts, a save with a
  lengthened easing, Cauldron-wise, descriptions, drinks that lend the wrong status refused.
  `content/status.rs` tests `vision`, `resist`, and `per_degree`. Text snapshot
  `potion-boost` (the character screen and status bar with Fortified and Owl-eyed).
- `world/potion_tests.rs`: every kind through `eat:`, a floor potion, food still eaten,
  refusals with no time, words and colours, `known_scrolls`, identify, marks by count, Thrifty
  drinker (no draw without it), Nose for draughts, the witch's kit, earlier labels unchanged,
  content refusals.
- `xtask/src/wiki/potions.rs` makes the wiki's potions table (`wiki/src/potions.md`).

# Potions

Spec §8 "Potions" (`spec/08g-magic-items.md`); the combat progression design §10
(`docs/design/combat-progression.md`); #565 (Q1: the kinds `heal`, `cure`, `mana`, `harm`); #567
(Q2: `boost`, `vision`, `haste`, `resistance`, their thirteen statuses, Lasting draughts);
#568 (Q3: `elixir`); the potions wave 2 design (`docs/design/potions-wave-2.md`) S2, #1224:
`status`.

## Objects and data

- A potion is an object of class `potions` (`!`) with an `[object.drink]` table:
  `content/potion.rs` (`DrinkSpec` → `Drink` on `ObjectDef::drink`, resolved in
  `ObjectSpec::drink_of`, as `wand_of` and `rod_of` are). A potion is `stackable` and nothing
  else (no slot, attack, missile, food, `read`, wand, rod, or magic `kind`); `level` and
  `frequency` make it loot, never magic (`check_base`, the `loot` flag in `resolve`).
- `Drink { does: Draught, colour, marks }`. `Draught` is the kind with only its fields:
  `Heal { dice, cures }`, `Cure { cures, diseases }`, `Mana { dice }`,
  `Elixir { stat, points, potential }` (`points` 1 to `ELIXIR_POINTS`' end, default 1;
  `potential` dice, never negative; `stat`, `points`, `potential` refused on any other kind),
  `Harm { dice, status }` (status required: hunger is loaded after objects),
  `Boost { status, severity }` (the status must raise a stat), `Vision { status }` (it must
  have `vision`), `Haste { status, severity }` (status always `hasted`, the Haste spell's; data
  writes only `severity`), `Resistance { status }` (it must have `resist`),
  `Status { status, severity }` (#1224, the potions wave 2 design §4.1: any status; the data author
  is trusted to pick one that ends, #1205); a severity is 1 to the status's `max`. `Draught::lends` is the
  status and degrees of the five that lend one.
  `marks` are
  `(Stat, count)` in stat order, 1 to `MAX_POTION_MARKS` each.
- Content checks (`content/mod.rs`): a drink's `cures` must be named in some status's or
  disease's `cured_by` (with food's `cures`); potions of one `colour` share one `fg`;
  `data/potion_labels.toml` says `{colour}` and has at least as many labels as potion types
  (`check_potion_labels`). Poisoned's `cured_by` has `antidote` for the antidote.
- Files: one per potion in `assets/data/objects/` (`potion_of_healing`, `potion_of_greater_healing`,
  `potion_of_full_healing`, `potion_of_antidote`, `potion_of_cure_disease`, `potion_of_mana`,
  `murky_draught`; the six `elixir_of_*`: might, grace, vigour, wit, will, presence; #567:
  `potion_of_might`, `_grace`, `_vigour`, `_wit`, `_will`, `_presence` and their
  `greater_potion_of_*`, `potion_of_cats_eyes`, `_owls_eyes`, `_moths_eyes`,
  `potion_of_quickness`, `potion_of_warmth`, `_frost`, `_earthing`, `_proofing`; #1224, kind
  `status`: `potion_of_barkskin`, `_glamour`, `_renewal`, `_the_quick`, `_the_unseen`,
  `_sea_legs`, their colours the design's §4.5 hues: bronze, rose, opal, sky blue). The witch's old `red_potion` and `blue_potion` are now the potions of
  healing and mana (ids renamed; saves change).

## Drinking: `world/potion.rs`

- No action of its own: `Action::Eat` (`eat:b`, `eat:floor:a`) reaches `World::eat`, which
  hands a stack with a `drink` table to `World::drink_potion` before any hunger check
  (`world/eat.rs`). What is neither food nor a potion is refused with no time ("You cannot
  consume a dagger."). `world/drink.rs` is the healing spring's Drink ability, a different thing;
  the two share only the `drink` action price (`data/actions.toml`, one turn).
- Order: priced at the start; one taken (`take_draught`: pack or floor; the keep
  (`Actor::potion_kept`: Thrifty drinker, worn affixes) is one gameplay draw, only above 0); "You drink the {title}." by the name the player
  knew; `learn_scroll` and "It is a potion of healing." for news; in a shop `drank_goods`
  (`world/shop.rs`, `ate_goods`'s twin, naming the potion as the player knows it); the effect;
  `train_stat_times(StatUse::Drank, Some(stat), count)` per mark (sure marks, no draw); the
  KEPT line if kept. Returns `Spent::Turn`.
- Effects: `Heal` rolls its dice on the gameplay stream, raises hp to at most the maximum, then
  `World::cure` per means; `Cure` cures per means and, with `diseases`, `end_disease` for every
  infection; `Mana` raises mana to at most the maximum; `Elixir` is `World::drink_elixir`
  (`world/elixir.rs`): the potential's roll first (one gameplay draw), then `points` ×
  `Stats::train(stat)` (the score +1; a trained rise leaves the potential, so the elixir lifts
  it to the score where it was below with `Stats::widen`),
  then `Stats::widen(stat, roll)` (never past `SCORES`), the hit points a Toughness rise adds
  given at once (as `stat_checks` does), and "You feel stronger[, and that you could be stronger
  yet]." (the clause when the potential rose by 2 or more; "You feel no different." at the top
  of the scores); its marks go through `StatUse::Elixir`, every other kind's through
  `StatUse::Drank`; `Harm` rolls degrees and `afflict`s them. `Boost`, `Vision`, `Haste`,
  `Resistance`, and `Status` go through `World::lend`: `afflict` the
  status at its degrees; its start line is the event's when it begins, said here when an
  existing one grows stronger, and "Nothing more comes of it." when nothing changes (boosts
  stack `highest`, the others `refresh`). The statuses (`data/status.toml`, the potions' group
  after Barkskin): six boosts (`penalty` with `per_degree`, `eases = 600`, `max = 2`), three
  visions (`vision`, `eases = 1800`), four resistances (`resist`, 50 %, `eases = 1800`); what
  they do is the statuses' (docs/dev/status-effects.md). Status and disease beginnings and ends are not said here: they are the action's events,
  which perception reports ("You feel sick.", "The poison wears off.").
- Creatures do not drink.

## Flasks: `world/flask.rs`

The potions wave 2 design §3, §5 (`docs/design/potions-wave-2.md`), slice S1 (#1223). The path
every later flask (smoke, fire, frost, acid, the bombers) builds on.

- **Data**: `[object.throw]` (`ThrowSpec` → `Flask { spell, rungs, range, noise, words }` on
  `ObjectDef::flask`, resolved in `content/potion.rs`). The loader checks only that `spell`
  names a spell (the owner, #1205: well-formedness only; `range` is a `u8`, so nothing
  overflows). `Flask::splash(content)` gives the splash's numbers at its rung of each ladder
  and nobody's stats (`Splash { roll, radius, degrees }`): what the cast, the describe line,
  the aiming warning, and the wiki read. `ObjectDef::shatters()` (a drink or a flask) and
  `ObjectDef::throw_range()` (a flask's `range`, any other potion `IMPROVISED_RANGE`).
- **The throw**: `World::throw` hands a stack that `shatters()` to `World::throw_flask` (the one
  line in `shoot.rs`): `throw_range` (talents and items added), `may_loose`, `within_reach`,
  priced `attack_price(Throw, AttackSkills::default())`, one taken (no keep), "You throw the
  …", the throw's noise, then `shatter_flask`, `say_news`, `exert`.
- **`World::shatter_flask(by, object, (from, at, range), events)`**, general over the thrower (a
  creature's hurl, S8, calls it too): the centre by `burst_center`, an `Event::Missile` of the
  object along the line to it (`Struck` when someone stands there), the flask's noise there
  (`sound_at`), and when the player sees the centre (`perception.is_visible`): its `words`, and
  `learn_scroll` with "It was a sleeping draught.". Then `Cast::by_flask(by, content, flask)`
  (`Pay::Flask`, `Cast::by` the thrower, `reroll` 0, no riders or skills, reach and degrees at the
  rung) and `splash`: `SpellPurpose::Damage` → `cast_burst_at(Caster::Actor(by), (centre,
  centre), …)`, `spell_type_spreads`, `spell_lands`, `lay_spell_zone`; any other kind → the
  spell's effect over its cells (a burst's `burst_cells`, else the centre alone), then its zone
  (`lay_spell_zone`, by `cast.by`) or its status on everyone there but the thrower
  (`splash_hex`). A player killed by their own is `Cause::Flask(object)`.
- **Not `perform`**: the cast kinds in `cast.rs` aim from the player's cell and check the
  spell's own range; a flask's splash goes off where it broke, so `flask.rs` calls the shared
  pieces (as `creature_cast.rs` does) instead.
- **Noise**: `SoundKind::Shatter` ("a crash of glass", `j` in the `sound` layer),
  `noise::SHATTER_NOISE` 30.
- **UI**: the throw choice (`ObjectList::Throw`, `PerceptionView::is_throwing_weapon`) lists
  potions; the aiming mode's range is `throw_range`; a known flask of a harmful burst that
  would catch the thrower asks first (`Aim::catches_reader`). Text snapshots `throw-list-flasks`,
  `potion-description-flask`.
- **Content**: the sleeping draught (`sleeping_draught.toml`, Sleep at rung 0, range 6; drunk,
  Drowsy 3; pink).
- **Content on existing spells** (S4, #1226; each at rung 0, range 6; the design's §4.4 hues):
  `flask_of_oil` (Slick), `stinking_flask` (Reek), `flask_of_tar` (Web),
  `flask_of_enfeeblement` (Hex of weakness), `flask_of_sloth` (Curse of sloth),
  `flask_of_darkness` (Mirk), `vial_of_nightmares` (Fear), `vial_of_silence` (Silence),
  `flask_of_thunder` (Thunderclap, `noise` thunder 150), `blasting_flask` (Shatter, `noise`
  explosion 100), each drunk a `harm`; `holy_water` (Rebuke: only the undead, by the spell's
  `only_tags`), drunk a `cure` of `holy_water`, the means in Ill-wished's and Branded's
  `cured_by`. The flasks of thunder and blasting are in general stores' and provisioners'
  `never` lists. Tests: `world/flask_content_tests.rs` (each one's throw and drink once).
- **Tests**: `world/flask_tests.rs` (fixture flasks on Fireball, Hex of weakness, and Slick):
  the throw and the burst on the dummy, the first creature in the way, the thrower's kill and
  experience, learned only when seen and heard when not, a ray's hex, a zone by the thrower, a
  potion that spills, refusals, a death by one's own flask, saves and replays, the loader's
  one refusal.

## Labels and colours

- `scroll_label.rs`: `Labels::draw` draws potion words **after** the rods' from the same
  `identification` stream, so scrolls', wands', and rods' labels (and runs' rolls) are unchanged.
  `data/potion_labels.toml` is the scroll labels' format with `name = "{label} {colour} potion"`.
- `Lore::labelled_names` puts the type's `colour` in `{colour}`; `Lore::fg` keeps a labelled
  potion's own `fg` (the type's colour, its family's hue) instead of the file's single `fg`.
- Known types live in `WorldState::known_scrolls` with scrolls, wands, and rods: learned by
  drinking, by `World::identify` (the scroll of identify, the appraisal service, dev
  `identify`), and from a chosen character's kit (`World::begin` adds kit potions as it adds
  kit scrolls; every class's, #1104).
- The item description (`magic/describe.rs::potion_lines`): a known potion's effect in words
  ("Heals wounds at once, and cures bleeding"), never dice; an unknown one "You do not know what
  this potion does." A `status` draught's words come from what its status does (`lent`: unseen,
  wades free, senses, quicker regeneration, a defence or a blow bonus, a cheaper action), else
  its name: "Makes you harder to hit for a while".

## Sources

- Loot by `level` and `frequency` (`magic::loot`), as scrolls.
- Shops (`assets/data/shops.toml`): general stores and provisioners stock and buy the potions
  class (by loot frequency, so healing and antidote are the common ones); the `alchemist` kind
  stocks and buys potions only, a stack of potions of healing always first. Hand-made shops
  (Cēaptūn) are unchanged.
- Quest rewards and level piles name potions by id (the Watch's reward, the hub's pile, the
  still room vault).
- Elixirs: deep loot (level 8, frequency 2), the forgotten altar's great boon `elixir`
  (`data/outcomes/forgotten_altar.toml`, one of the six by weight), and the alchemist's draw (by
  frequency, so very rarely). A shop kind's `never` list (`ShopDef::never`, read in
  `Shops::stocks`) keeps them, and the costly flasks (thunder, blasting), out of general stores
  and provisioners.

## Talents (system `potions`)

- **Thrifty drinker** (`potion_kept { chance }`, `TalentMods::potion_kept`): read in
  `Actor::potion_kept` with the worn `potion_kept` affixes (#633; the sum held to `MAX_KEPT`,
  50, unless the talent alone is above it), which `take_draught` reads; one draw only above 0.
- **Lasting draughts** (Vigour, Toughness 13; #567): `status_lasts` (50) on the six boosts and
  three visions, `TalentMods::status_lasts`, read in `TalentMods::eases_every` (each degree
  takes half again as long to ease). The design called it "Strong constitution", a name a
  diseases talent already has.

#787 cut Quick study of the body, the witch's Cauldron-wise, and Nose for draughts, with their
effect kinds (`potion_marks_percent`, `potion_heals`, `potion_smell`) and `World::smell_potions`.

## UI

- The Consume list (`ui/object_screen.rs`, `ObjectList::Eat`) lists food and potions, floor
  first, grouped by class (Food, then Potions). Text snapshots: `consume-list`,
  `potion-description`, `potion-description-unidentified` (`ui/screen_snapshots.rs`).

## Tests

- `world/potion_status_tests.rs` (#567): boosts raise and wear off, greater ones ease to
  common ones, `highest`, visions change `Actor::eyes` and what is seen in the dark,
  resistances add as gear and never reach immunity, haste, Lasting draughts, a save with a
  lengthened easing, descriptions, drinks that lend the wrong status refused; #1224's six
  `status` draughts (each lends its degrees, marks, and wears off on time; the unseen and sea
  legs do what their statuses do; a second dose adds nothing; their words; a draught that lacks a
  severity refused, a `per_action` one allowed).
  `content/status.rs` tests `vision`, `resist`, and `per_degree`. Text snapshot
  `potion-boost` (the character screen and status bar with Fortified and Owl-eyed).
- `world/potion_tests.rs`: every kind through `eat:`, a floor potion, food still eaten,
  refusals with no time, words and colours, `known_scrolls`, identify, marks by count, Thrifty
  drinker (no draw without it), the witch's kit, earlier labels unchanged,
  content refusals.
- `world/elixir_tests.rs`: score +1 and potential +1 or +2 over seeds (both come up), the
  potential lifted first, hit points after a Toughness elixir and maximum mana after an
  Intelligence one, 60 marks and Quick study's 120, each elixir its own stat, the top of the
  scores, no general store or provisioner stocks one, the altar's outcome names all six,
  content refusals.
- `xtask/src/wiki/potions.rs` makes the wiki's potions table (`wiki/src/potions.md`).

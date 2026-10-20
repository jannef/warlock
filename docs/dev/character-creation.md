# Character creation

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08i-character.md. Keep it current with the code, as CLAUDE.md says.

Character creation (spec §8 "Character creation"; M15) lives in `warlock-core/src/character.rs`:
`CharacterSpec` (race id, `Gender`, class id; `FromStr`/`Display` as `dwarf,female,warrior`;
serialized as `{race, gender, class}`) and `CharacterSpec::start(content, seed, start_date)` →
`Start` (starting `Stats`, the race's vision, and a `Character`: the spec, `raised` (the stat the
gender raised: male Strength, female Dexterity, other `other_stat(seed)` from the
`character/gender` stream), `born` (`BirthDate { year: i64, month, day }`: age uniform in the
race's `age` range and birthday from `character/birth/<race>`, counted back from the start date
with `BirthDate::counted_back`; `age_on(date)`), and `sign` (`content::Sign { animal, element }`,
the birth year's)). Scores: 10 + race + gender + class + animal + element, clamped to 1..=40;
potentials `base_potential` + the race's, never below the score. Flags for rules:
`Character::born_of_element("fire")`, `born_in_year_of("manticore")`. Content:
`data/races.toml` (`content/race.rs`: `Races` with `base_potential`, `RaceDef` with `vision`,
`age`, `stats`, `potential`, `hp`), `data/classes.toml` (`content/class.rs`: `ClassDef` with
`stats`, `hp`; `Start::hp` is their sum),
and `data/horoscope.toml` (`content/horoscope.rs`: `Horoscope` with `anchor_year`, `animals`,
`elements`, `sign_of_year`, `great_cycle` (60), `name` ("Fire Manticore")); all optional.
Adjustments are `PerStat<Dice>` (`stats::adjustments` checks every result is within -10..=10;
`stats::roll` rolls them in stat order) and are rolled from `character/rolls/<spec>` in a fixed
order (race, class, animal, element, race potentials): **adding a die to data changes later
rolls** of every character with that race or class. `Dice` (`warlock-core/src/dice.rs`) is the
one dice type (parse `2d3+1`/`-1d2`/`+2`, `Display`, `min`/`max`, `roll(&mut Rng)`); combat
uses it for damage. Scenarios name their character (`character = { race, gender, class }`,
`ScenarioDef::character`); with one, `vision`, `stats`, `potential`, and `hp` are refused.
`World::new` starts as the scenario's character, `World::with_character(content, scenario, seed,
Some(&spec))` as another; `WorldState::character` (the `Character`, saved, hashed)
and `PerceptionView::character` (the player knows it; `PerceptionView::time`
for their age). Replays record only the spec. The summary and the character screen show final
scores only: never dice, a breakdown, a modifier, or a potential (the step screens and the wiki
show adjustments as written in data). Balance is not a goal; classes only adjust starting
scores, give hit points, train skills and weapon skills, and give a starting kit.

**Starting kits** (spec §8 "Character creation", #257): the data side is `content/kit.rs`
(`KitLineSpec`, parsed with the race, class, or sign as written, a string or a table;
`Content::load_kits` resolves them into `Kit` (`KitItem { keep: Keep, one_of, trained,
gender, magic }`) once objects, gems, slots, and weapon skills are known, checking each
alternative with `object::parse_stacks` and `equipment::free_slot`); `RaceDef::kit`,
`ClassDef::kit` (`ClassDef::trainable`: what a class's `trained` may name), and
`SignPart::kit` (the horoscope's hook, empty in data). The rules are `warlock-core/src/kit.rs`:
`kit::roll(content, character, trained, seed)` → `Pack { items, equipment }` from the stream
`character/kit/<spec>`, the kits in `kits_of` order (class, race, animal, element), a line
drawing only if `KitItem::applies` (one `rng.index` among alternatives, then
`magic::loot::make` at `kit::ITEM_LEVEL` for a `magic` line); `Pack::put` joins stackables,
`Pack::equip` uses `equipment::free_slot`. `CharacterSpec::start` returns it as `Start::kit`
(`weapon_skills` now also returns the class-trained set: its fixed ranks and the picked
choice sets). `kit::starting_pack(content, scenario, kit)` is the one place the starting pack
is put together (the kit unless `ScenarioDef::kit` is false, then `carry` on top): used by
`World::with_character` and the summary (`ui/chargen_screen.rs::kit_words`, names from
`Pack::names`). No new state: the pack is the player's inventory and equipment, and
`rot::made_at` dates the food as before. Tests: `content/kit.rs` (parsing, refusals),
`kit.rs` (Twin Blades and Duelist, the barbarian's pick, deterministic choices, no other roll
moves, every worn or wielded item finds its slot, nobody Strained and only the aristocrat and
the merchant Burdened, worst case analytically, magic unidentified, the scenario's pack, the
gender and sign hooks, a replay). The wiki's tables are `xtask/src/wiki/kits.rs`
(`class-kits.md`, `race-kits.md`) and the scenarios table's kit column.

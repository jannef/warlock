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
show adjustments as written in data; the wiki also gives the modifiers and the races' potentials
exactly). Balance is not a goal; classes only adjust starting scores and give hit points.

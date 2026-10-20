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
uses it for damage. Scenarios name their character (`character = { race, gender, class, name }`,
`ScenarioDef::character` and `character_name`; the `name` optional); with one, `vision`, `stats`,
`potential`, and `hp` are refused.
`World::new` starts as the scenario's character, `World::with_character(content, scenario, seed,
Some(&spec))` as another, `World::with_character_named(.., name)` named; `WorldState::character`
(the `Character`, saved, hashed) and `PerceptionView::character` (the player knows it;
`PerceptionView::time` for their age). Replays record the spec and the name (`Replay::name`). The
summary and the character screen show final scores only: never dice, a breakdown, a modifier, or a
potential (the step screens and the wiki show adjustments as written in data). Balance is not a
goal; classes only adjust starting scores and give hit points.

**Names** (#218): `Character::name` (saved, hashed: saves change). `character::check_name` is the
one check (trimmed, 1 to `MAX_NAME` = 20 characters, no control character; what the font draws is
the text field's business, `render::atlas::draws`). `data/names.toml` (`content/names.rs`:
`Names`, a `male` and a `female` list by race id, checked against the races; `Names::offered`
gives `other` both; not fingerprinted) feeds `character::random_name(content, seed, spec, draw)`,
drawn from `character/name/<race>,<gender>/<draw>`, its own stream, so no roll moves (the class
plays no part). `CharacterSpec::start` names the character with draw 0 (else `NAMELESS`,
"Wanderer"); a run told no name (`with_character`, a replay or command line without one) gets
`character::default_name`: the scenario's `character_name` for its own character, else draw 0. The
CLI and the game take `--name`; the `stats` dump layer starts `Name Brakka` and the JSON
`character` has `name`. Character creation's Name step (`ui/name_screen.rs`, after the class) is
the game's text field: typed text is literal, `Context::Text` edits it (`docs/dev/screens.md`); an
empty field gives draw 0 (`Draft::name`), and the summary starts the run with the name
(`MenuChoice::NewRun { name, .. }`). The name is shown on the character screen, the summary, the
death screen ("Here lies Brakka,"), and the list of saved characters (`docs/dev/saves.md`); no rule
or message reads it yet.

**Wording and the seed** (#256): everything here is deterministic in the seed, and no player-facing
text says so (UI, messages, scenario descriptions, the wiki): randomness is "chance", "fate", or
"random". The Random rows say `Let chance pick your race.`; "other" on the gender step says `+1 to
one stat, chosen by fate.` and the summary shows `Other` (`gender_description`; the stat is never
named before the run starts), while the character screen, once the run has begun, names it
(`character_screen::gender_text`: `Other (Willpower +1)`). A release build's new run gets a seed no
one chose (`new_run_screen::fresh_seed`; see `docs/dev/screens.md`); developer builds keep the
seed field, and only they show the seed on the summary. Code comments, data comments, the spec, and
these notes keep explaining the determinism exactly.

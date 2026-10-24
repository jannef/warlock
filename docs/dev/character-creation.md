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
`age`, `stats`, `potential`, `hp`, `alignment`), `data/classes.toml` (`content/class.rs`:
`ClassDef` with `stats`, `hp`, `alignment`; `Start::hp` is their sum, and `Start::alignment` the
sum of their `alignment`s, the starting score, `docs/dev/alignment.md`),
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
potential (the step screens and the wiki show adjustments as written in data; the wiki also gives
the modifiers and the races' potentials exactly). Balance is not a goal; classes only adjust
starting scores and give hit points.

## Birth signs' gifts and lucky days (#265)

Design: `docs/design/horoscope.md` (§3–§6, §8–§10, §12). `data/horoscope.toml`'s animals and
elements name their gifts in `grants` (talents of the `stars` tree, `granted = true`, a `flaw`
only among an animal's), and `[[lucky]]` lists the five lucky days (`animal`, `element`, `day`,
`title`, `description`, `grants`). `content/horoscope.rs`: `Horoscope::lucky(sign, day)`,
`Horoscope::grants(sign, day)` (the animal's, the element's, a lucky day's, in that order) and
`check_talents` (run after both load: grants exist, are granted `stars` talents, none twice for a
sign; talents' `animal`/`element` needs name real ones; content without the `stars` tree, a
fixture, grants nothing). `Content::lucky`, `Character::lucky`/`title`/`titled_name`/
`sign_title`/`grants`.

`world/stars.rs` puts the grants into `Actor::talents` (first, in grant order, rank 1;
`Talents::regrant` drops every granted talent first) in `World::with_character_named` and again
in `World::from_state`: they are **never a stored choice**, so a gift added to the data reaches a
saved character and a removed one leaves it. They change hashes of runs whose sign has gifts, as
any rule change does; the birth draws are untouched, so seeds keep their dates and signs. The
gifts' effects are `TalentEffect`s read at one hook each (`docs/dev/progression.md`); `stars.rs`
also says what is wading (`wading`: terrain that freezes, or a disease's wading ground).

Shown: the reveal (`ui/sign_screen.rs`, `--screen sign`, after the race), the summary's Sign row,
the character screen's dimmed Stars line (gift names, two lines at most) and the title after the
name (character and death screens), the `stats` dump's `Sign Wood Salamander, the Racer`, the
`talents` dump's `(sign)` / `(sign, flaw)`, JSON `character.lucky`, `talents[].granted`/`flaw`.
The wiki's `horoscope-animals.md`, `horoscope-elements.md`, `horoscope-lucky.md`
(`xtask/src/wiki/horoscope.rs`) state the rules exactly and the odds by race at the showcase's
start. Tests: `world/stars_tests.rs` (grants per sign and day, the lucky days pinned, restore,
every hook, the prerequisites, content checks, the lucky capture's seed 440), the screen tests in
`ui/sign_screen.rs`.

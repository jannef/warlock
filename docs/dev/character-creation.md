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
`age`, `stats`, `potential`, `hp`, `natural`, `resist`, `light_blind`, `disease_check`,
`eats_rotten`, `alignment`, `rolls`), `data/classes/` (`content/class.rs`:
`ClassDef` with `stats`, `hp`, `alignment`, `known_for`, `rolls`, and the level table's
`abilities` and `bonus_talents` (`docs/dev/progression.md` "Class tables"; level 1's are given
in `start_run`); one file per class, `NN-id.toml` in the order offered; `Start::hp` is their sum, and `Start::alignment` the
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
Some(&spec))` as another, `World::with_character_named(.., name)` named, and
`World::start_run(.., kit)` with the starting kit or without (below); `WorldState::character`
(the `Character`, saved, hashed) and `PerceptionView::character` (the player knows it;
`PerceptionView::time` for their age). Replays record the spec, the name (`Replay::name`), and
whether the run started with the kit (`Replay::kit`). The summary and the character screen show
final scores only: never dice, a breakdown, a modifier, or a potential (the step screens and the
wiki show adjustments as written in data; the wiki also gives the modifiers and the races'
potentials exactly). Balance is not a goal; classes only adjust starting scores, give hit
points, train skills and weapon skills, and give a starting kit.

**Races in the rules** (#255). `WorldState::race_of(content, actor)` is the one lookup of the
player's race (`None` for a creature, or a run without a character); `load`, `size_of`, and every
race trait go through it. The traits, each read in one place:
- `natural` (`content::Hide`, as a creature's, checked by `Hide::check`): `Actor::hide(content,
  race)` is the race's hide, else the creature type's; `Actor::defence`, `protection`,
  `soaked_by`, and `target` take the race for it, and so does `fight::Guard::of` (the hide's name
  in messages). `World::defence_of`/`protection_of` pass it (the dump reads those).
- `resist` (resolved by `Content::resolve_per_kind` in `Races::parse`): `Actor::resistances(content,
  race)` adds it to the creature type's; `World::resistances_of` passes it, so blows, blasts, and
  status doses all see it.
- `light_blind` (`content::LightBlind { glare, sight, to_hit }`): `LightBlind::dazzled_by(light)`
  is the rule. `Perception::compute` reads it on the viewer's own cell (after the light, before
  visibility) and caps the visible reach at `sight` (wall completion too), keeping `dazzled` for
  `PerceptionView::dazzled` (the status bar's `Dazzled`, the dump header's `dazzled`, JSON
  `dazzled`). `world/glare.rs`: `World::glared(attacker, profile)` (called once in
  `fight::resolve`, so melee, missiles, and free blows) takes `to_hit` off while
  `sight::light` on the attacker's cell dazzles it; `World::glare_news(before)` (in `perceive`)
  notes when it begins and ends. Creatures never have a race, so never pay for the light.
- `disease_check`: `World::disease_modifier(actor, stat)` (`world/disease.rs`), both checks.
- `eats_rotten`: the top of `World::eat_rotten` (`world/eat.rs`).
- `passive_search = "traps"` (`PassiveSearch::Traps`): `World::notices(at, hidden)`.
Tests: `world/race_tests.rs` (test races beside the built-in ones, one trait each, and the evil
races' starting bands); `content/race.rs` (parsing and refusals).

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
choice sets). **Who gets it**: `World::start_run(content, scenario, seed, character, name,
kit)` is the full constructor; `kit` is true for a run whose character the player chose:
`Game::new_run` (New run's character creation), the game and `warlock-cli` when `--character` or
`--name` is given, and `Replay::start` when the replay says `kit`. Everything else
(`World::new`, `with_character`, `with_character_named`, `--scenario` alone, tests, golden
walks, old replays) starts with the scenario's pack alone, as before kits. `WorldState::kit`
(saved, hashed, left out while false, so other runs hash as before) records it, and
`Replay::new` copies it into the replay. `kit::starting_pack(content, scenario, kit)` is the one
place the starting pack is put together (the kit, if any, then `carry` on top): used by
`World::start_run` and the summary (`ui/chargen_screen.rs::kit_words`, names from
`Pack::names`). `rot::made_at` dates the food as before. Tests: `content/kit.rs` (parsing,
refusals), `kit.rs` (Twin Blades and Duelist, the barbarian's pick, deterministic choices, no
other roll moves, every worn or wielded item finds its slot, nobody Strained and only the
aristocrat and the merchant Burdened, worst case analytically, magic unidentified, who gets the
kit and the scenario's pack, the gender and sign hooks, a replay with and without the flag).
The wiki's tables are `xtask/src/wiki/kits.rs` (`class-kits.md`, `race-kits.md`). The screen
snapshots' `kit: true` starts a case's run with the kit (`kit-equipment`, `kit-inventory`, and
`kit-summary` in `crates/warlock/src/ui/screen_snapshots.rs`); the golden manifest's `kit = true`
does the same for an entry.

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
`Talents::regrant` drops every granted talent of the `stars` tree first) in `World::with_character_named` and again
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

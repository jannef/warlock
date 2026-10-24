# Warlock specification — 8. Architecture (continued): the character

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Character creation** (`warlock-core/src/character.rs`; M15): a run starts as a character, a
  race (`data/races.toml`), a gender (male, female, or other), and a class
  (`data/classes/`), chosen on New run (§6) or given on the command line (`--character
  dwarf,female,warrior`, §9); otherwise the scenario's character (§7). It decides:
  - **Name** (#218): the character has a name, 1 to 20 characters, typed on New run's Name step
    (after the class, before the summary; §6) or given on the command line (`--name Brakka`).
    The step's Random, and an empty field, give one of the race's names for the gender
    (`data/names.toml`, a `male` and a `female` list per race; `other` draws from both), drawn
    from the stream `character/name/<race>,<gender>/<draw>`, so no other roll moves and the class
    plays no part. A run told no name (a replay or command line without one) gives the
    scenario's own name (§7) to its own character, else the race's first random name, else
    "Wanderer". The name is shown on the character screen, the summary, the death screen ("Here
    lies Brakka, …"), and the list of saved characters (§7 "Saves"); no rule or message reads it
    yet.
  - **Age and birthday**: the starting age is uniform within the race's `age` range, the
    birthday (month, then day) uniform over the year, both from the stream
    `character/birth/<race>` (so the class and gender never move a birthday). The birth date is
    counted back from the run's start date (the scenario's `start_date`): the character is
    exactly that old on it, so a birthday later in the year falls one year further back. Birth
    years before the calendar's year 1 count on down through 0. The age on the character screen
    is the age on the current date: one more on every birthday.
  - **Birth sign** (the horoscope, `data/horoscope.toml`): every year has an animal and an
    element; both advance one step a year, in data order, on the 1st of Frostmere (the new
    year), so with 12 animals and 5 elements every pair comes exactly once in a **great cycle**
    of 60 years. The anchor year 1 (the Great Race) is a Wood Salamander year, and so are 61,
    …, 1381, and 1441 (1444 is Metal Unicorn). The sign is the birth year's; the player does
    not choose it. A sign is said element first: "the Fire Manticore". Signs are **flags** for
    later rules: `Character::born_of_element("fire")` ("is this character Fire-born?"),
    `Character::born_in_year_of("manticore")`, and `Sign::is_element`/`is_animal`.
  - **What a sign gives** (#265; the design is `docs/design/horoscope.md`): each animal two
    **gifts** and a **flaw**, each element two gifts, as **granted talents** of the `stars` tree
    ("Talents" below): named by the animal's and the element's `grants`, given at birth, never on
    a pick list (`granted = true`), a flaw (`flaw = true`) shown with "but", dimmed, and never a
    prerequisite. They are computed from the sign and the birth day whenever a run starts or is
    restored (`world/stars.rs`), never stored as a choice, so a gift added to the data reaches a
    saved character; they come first among the talents held, in grant order (the animal's, the
    element's, a lucky day's), and work like any talent, through `TalentMods` at the one rule
    each names. The gifts whose rules exist so far: Fleet and Ponderous (speed ±5 %), Slight and
    Roc's strength (carrying −10 % / +25 %, Roc's strength also kicks doors as if Strength +4),
    Regrowth (healing +50 %, bleeding eases twice as fast), Strong swimmer (wading costs a plain
    step and never gives a disease), Pure blood (disease checks +4, poison eases twice as fast
    and never lowers the scores), Heedless and Always a way (−4 / +3 to every check to find
    hidden things; Always a way also searches a cell farther), Riddler's mind (XP +10 %), Little
    loved and Grim company (shop favour −10), Supple (Dodge +1, the burden's Dexterity cap +1),
    Evergreen (hunger −15 %, rotten food's sickness halved), Stone hide (Protection +1),
    Stonewise (notices on stone in passing, digging +25 %), Keen edge (threat +1 with every
    attack). The rest of the design's gifts land with their rules (S2–S4) and are listed in
    `grants` only then.
  - **Lucky days** (`[[lucky]]`): five animal–element pairs, each with a day of the month (any
    month), the day the animal's place in the Great Race: the Wood Salamander on the 1st (the
    Racer), the Metal Manticore on the 3rd (the Iron Scourge), the Fire Phoenix on the 10th (the
    Firebird), the Water Kraken on the 11th (the Deep One), and the Earth Barghest on the 30th
    (the Gate Warden). Born in that year **and** on that day, a character has the title after
    their name (the character and death screens: "Brakka, the Iron Scourge") and one gift more:
    the Racer's Winner's luck (a natural 2 on the attack roll is rolled again once, two more
    draws only then; speed +5 % more); the other four's gifts come with S5. About one character
    in 1 800 over an even spread of births; by race at a start date, as the wiki tabulates. The
    game never says lucky days exist until a character is born on one. The birth draws are
    unchanged, so existing seeds keep their dates and signs.
  - **Sign prerequisites**: a talent may ask `animal = "roc"`, `element = "fire"`, or `lucky =
    true` ("to be born in the year of the Roc", "to be Fire-born", "to be born on a lucky
    day"), read off `Character::sign` and the birth day.
  - **Starting scores**: 10 plus the race's adjustment, plus the gender's (male: Strength +1;
    female: Dexterity +1; other: +1 to a stat picked uniformly from the run seed's
    `character/gender` stream, so the same seed always raises the same stat; character creation
    never names it, the character screen does once the run has begun, §6), plus the class's,
    plus the birth sign's animal's and element's, kept within 1 to 40.
  - **Dice**: adjustments that are dice are rolled in the background, ADOM-like, from the stream
    `character/rolls/<race>,<gender>,<class>`, in this order: the race's scores, the class's, the
    animal's, the element's (each in stat order), then the race's potentials, then the skills
    ("Skills" below: the race's, the class's, each in skill order, then the race's extra class
    skill), so skills never move a score; fixed numbers draw nothing. The same seed, choices, and
    start date always make the same character. The player sees only the final scores; the step
    screens and the wiki show adjustments as written in data (`St 1d3+1`): they are rules, not
    modifiers.
  - **Potentials** (hidden): `base_potential` plus the race's potential adjustment (possibly
    rolled), within 1 to 40 and never below the starting score. Classes, genders, and signs do
    not change potentials.
  - **Eyes**: the player's vision profile is the race's (the dev vision switch still works).
  - **Hit points**: the race's and the class's `hp` add to Toughness for the maximum ("Combat"
    below).
  - **Alignment** ("Alignment" below): the starting score is the race's `alignment` plus the
    class's, kept on the scale: paladins start Good, thieves and witches Callous, everyone else
    Neutral; a race only leans (elves and halflings a little toward good, half-orcs a little
    toward evil), never alone out of Neutral. It draws nothing.
  - **Weapon skills** ("Weapon skills" below): the race's starting ranks plus the class's, plus
    one set of the class's `weapon_skill_choice` if it has one, at most 10 each; each then
    raises the skills it defaults to. Dice among them roll from their own stream,
    `character/weapon-skills/<race>,<gender>,<class>`: the race's (in skill order), the
    class's, the pick of the choice (one draw), then the chosen set's; so they never move a
    stat roll.
  - **Fighting styles** ("Fighting styles" below): the class's `styles` plus one set of its
    `style_choice` if it has one, from yet another stream,
    `character/styles/<race>,<gender>,<class>`: the class's (in skill order), the pick of the
    choice, then the chosen set's; so they never move a stat or a weapon skill roll. Each raises
    the styles it defaults to.
  - **Starting kit** (`warlock-core/src/kit.rs`, data `content/kit.rs`; #257): what the character
    carries, wields, and wears at the start. The class's `kit` (`data/classes/`) comes first,
    then the race's few racial items (`data/races.toml`), then the birth sign's animal's and
    element's (`data/horoscope.toml`: the hook for the horoscope; none give anything yet). A kit
    is a list of lines, each one item or a few of one, as level files place objects: a string is
    carried, a table says `carry`, `wield` (wielded or readied: hands, missile weapon, missiles,
    Tool slot), or `wear`, and may add `trained` (only if the class trained that weapon skill or
    fighting style: its fixed ranks, or the set of its choice that was picked; never the race's
    ranks or defaults), `gender` (only for that gender: the hook for gender's items), and
    `magic` (a base item made as loot makes it at item level 1, its rarity rolled at least
    `any`, `magic`, or `rare`); `"a|b|c"` is one of them. Each item goes into the pack in order
    (a stackable one joins a stack of the same), and a `wield` or `wear` item into the first free
    slot it fits (a second light weapon to the off hand); one that finds none is carried. Kits
    roll from a stream of their own, `character/kit/<race>,<gender>,<class>`, in that order: a
    line draws only if it applies, one draw to pick among alternatives, then for a `magic` line
    the loot draws (rarity, affixes, a rare name, an ornament); so no stat, skill, weapon skill,
    or style roll moves. Kit items are what they are: magic ones unidentified,
    food made when the run starts (§8 "Hunger"). Data is checked when content loads (every item
    exists, what is wielded or worn fits a slot of that kind, `magic` only on base items, a
    class's `trained` only what it trains). Unfair on purpose: the aristocrat sets out rich, with
    a perhaps magic rapier and an heirloom ring (magic or rare), the merchant with goods and gems,
    the commoner with a stick and a turnip; nobody starts Strained, and only the aristocrat and
    the merchant may start Burdened (a test checks the weakest character of every combination,
    every die at its least and the worst sign). **Who starts with it**: a run whose character
    the player chose, that is one started from New run's character creation, or by the game or
    `warlock-cli` with `--character` or `--name`, or a replay that records the kit; the
    scenario's `carry` comes on top (§7 "Scenarios"). A run on the scenario's own character that
    nobody chose (`--scenario` alone, tests, golden walks, replays from before kits) starts with
    the scenario's pack alone, as before kits. The world state and replays record which
    (`kit`). The summary of character creation (§6) lists what the run starts with, each stack
    as named ("a dagger (wielded)", "2 wool cloaks"); it never names the seed. The wiki lists
    every class's kit and race's additions.
  - Balance is deliberately not a goal: some combinations are poor (a halfling commoner), and
    classes do nothing but adjust starting scores, give hit points, train skills, give
    weapon skills and fighting styles, give a starting kit, and set where the alignment starts.
    No other D&D concept comes with them (spells; the alignment is good against evil only, not
    D&D's grid).
  - The character (`Character`: the name, the race, gender, and class ids, the stat the gender
    raised, the birth date, and the birth sign) is part of the world state (saved, hashed) and
    shown on the character screen; replays record the choice, the name, and whether the run
    started with the kit, and since everything else follows from the content, the choice, the
    seed, and the start date, a run is reproduced by its scenario, seed, character, name, and
    kit. `World::new` starts as the scenario's character, `World::with_character` as a given
    one, `World::with_character_named` named, `World::start_run` with or without the kit.
- **Alignment** (`warlock-core/src/alignment.rs`, `content/alignment.rs`, `world/alignment.rs`;
  #254): good against evil, as ADOM has law against chaos.
  - **The score**: the player has a hidden integer score from −1000 (as evil as can be) to 1000
    (as good), saved and hashed, never shown as a number. They know it as one of seven
    **bands**, by edges in `data/alignment.toml` (§7): Saintly (750 up), Good (400), Kind (150),
    Neutral (−149 to 149), Callous (to −150), Evil (to −400), Wicked (−750 down). The band is
    on the character screen, the character creation summary (the band the character starts
    in), and the `stats` dump; nothing else shows it.
  - **Creatures** have a fixed alignment instead (`alignment` in their file: good, neutral, or
    evil; neutral by default), which a level legend entry may change for one spawn, as it does
    a faction. The kobolds are evil, the beasts (rats, bats, beetles, deer) neutral, and the
    showcase's people (the gardener, the bard, the hermit, the shopkeepers) good. A creature's
    alignment is never shown (a "detect evil" ability may show it later).
  - **What moves it**: `World::shift_alignment(actor, amount, cause)` is the one place a score
    changes. Moves diminish, ADOM-style: a deed in the direction the score already leans (away
    from 0) moves it by its worth times the share of the way still left to that end, rounded
    toward 0 (halfway to good a good deed counts half, and small deeds never make a saint); a
    deed the other way counts in full. The deeds, each worth what `[shift]` in
    `data/alignment.toml` says: killing someone not hostile to the player (by their alignment:
    good −150, neutral and evil −100), except a beast (a neutral creature that does not talk),
    which may be hunted (0); killing someone hostile (evil +10, neutral and good 0); attacking a
    shopkeeper who trades with the player (−100, once: the keeper is outraged after it);
    leaving a shop with unpaid goods (theft, −100; Steal, #234, will be theft too); paying a
    shopkeeper what is owed (+5); eating a corpse of the player's own race (a creature type's
    `race`) or of a creature that talks and is not hostile to them by its type (−200); and a
    quest's own `alignment` as it is completed (helping quests good, dark ones evil). Nothing
    else moves it. Only the player has a score; deeds draw nothing from any random stream.
  - **Messages**: crossing into another band says so, after the deed's own messages, as a note
    that does not stop a rest (the band's `rise` or `fall` line: "You feel righteous.", "You
    feel you have strayed from the path of good."); smaller moves are silent.
  - **What reads it**: good creatures **scorn** an Evil or Wicked character, evil creatures a
    Good or Saintly one (`Band::scorned_by`): they answer a chat with a cold line
    (`greet_evil`, `greet_good` in `data/dialogue.toml`) instead of their greeting and
    one-liners, and offer no quest whose `alignment` is good (topics and quests they already
    gave stay). A quest may name the bands it is offered to (`requires_alignment`). A good
    shopkeeper's prices move with the band, by a percent in data added to the player's favour
    (§7 "Shops": Saintly 10 % better, Evil 10 % worse, Wicked 20 % worse); the prices shown
    include it. Talents and class abilities (#220, #243) may ask for an alignment
    (`requires_alignment`, `ability::strayed`): a paladin's gifts need Good or better, and a
    paladin below that keeps them but cannot use them (greyed out, "you have strayed") until
    they climb back. Mana reads nothing of it, for now. Gods and prayer, holy and unholy damage
    and items, and alignment-limited artifacts will read the same band.
  - **Talents** (proposed for the talents PR, on hooks that exist): *Righteous fury* (Good or
    better: +1 to hit and damage against evil creatures, per 10 levels; `Profile` in
    `attack_with`, reading the target's `Actor::nature`), *Hardened heart* (Callous or worse:
    killing peaceful creatures costs no further alignment; `shift_alignment` skipping
    `Deed::KilledPeaceful`), and *The middle way* (while Neutral: +1 Dodge and +2 on checks;
    `Actor::defence`, `check`).
- **Skills** (`warlock-core/src/skills.rs`, `content/skill.rs`, `world/train.rs`,
  `world/skill_roll.rs`; M20, #132; level-up rolls M30, #230), ADOM-like:
  - **Trained or untrained**: every actor has a set of trained skills, each with a **value**
    (`Actor::skills`). A skill not in it is **untrained**: it does not exist for that actor, is
    never shown, earns no marks, and gives nothing; rules that read a skill get "untrained",
    which is not a value of 0 (trained at 0 is shown and can grow). Magic items' skill affixes
    (§8 "Magic items") add to a skill's value for what it gives while worn
    (`Actor::skill_effect`, the one way rules read a benefit), an untrained skill's from 0; they
    never train it or change the value shown.
  - **From race and class** (§7): their `skills` are dice rolled when the character is made,
    after every other roll ("Character creation" above); a skill both train sums the two; the
    race's `class_skill` (humans, `2d4+4`) goes to one of the class's skills picked uniformly
    from the same stream. A creature type's skills are fixed values. A scenario without a
    character gives a player trained in nothing.
  - **Values** run 0 to 999 (`MAX_VALUE`), 100 a master; there is no hard cap, but points come
    slowly past the soft cap (100). A value grows **only by level-up rolls** (below).
  - **Training marks**: a meaningful use of a trained skill may earn it a mark, with its rule's
    chance (§7), one draw of the run's **training stream** (`training`, kept in the world state;
    never the gameplay stream, so training never changes a fight's rolls) per rule: a step of the
    player's that some creature on the level would hear at its threshold but for their stealth
    (`sneak`, through the same propagation as hearing; the overworld is silent), a perceived
    sound that is news (`hear`, one draw per turn with news), an attack on the player that missed
    because of their Dexterity or Dodge (`dodge`, the dodge messages), an action other than
    waiting (not a rest's turns) taken while the player's burden slows them, as it began
    (`burdened`; a dwarf's Burdened does not), a step on the overworld (`travel`), a kick at a
    closed door (`kick`), a turn spent searching, whatever it finds (`search`), noticing
    something hidden in passing (`spot`; §8 "Searching"). Only the player trains for now;
    creatures keep their type's skills. Marks are a **reservoir**: they raise nothing by
    themselves and nothing is said; they gather until a level-up roll on the skill spends them.
  - **Level-up rolls** (design `docs/design/progression.md` §5): every level gives the player
    **skill rolls**: their class's `rolls` (`data/classes/`: 2; the thief and the ranger 3, the
    bard 4), their race's (`races.toml`: humans 1), and one more for an Intelligence score of
    `intelligence_from` (14) and one more for every `intelligence_every` (4) points above it
    (`[rolls]` in `data/skills.toml`; the score without status effects, as the character screen
    shows it), kept in the level's `Due { rolls }` beside its talent pick (`World::level_up`;
    `World::level_rolls`). A player trained in nothing gets none. Each roll is spent with
    `Action::Advance` (`advance:stealth`; no time, recorded) on a **trained** skill of the
    player's choice, any number on one skill; while any is left every other action is refused
    ("You have skill rolls to spend."), as talents are. A roll throws the die of the skill's
    step on the **ladder**, set by its marks (`[ladder]`: `+1` from 0 marks, `1d3` from 5, `1d5`
    12, `2d4` 25, `3d3` 50, `3d4` 100, `3d5` 200, `4d4` 400, `4d5` 800; ADOM's), one step up
    (the top at most) for a skill the class is **known for** (`known_for`, its `2d6+10` skills),
    and adds **+1** for any other skill the class trains or one the race trains (both: a known-for
    racial skill is a step up and +1); it adds the result to the value, half of it rounded up
    from the soft cap (`soft_cap`, 100) on, and **spends** `spend` (50) percent of the skill's
    marks rounded up (50 → 25 → 12 → 6 → 3), so the next roll on it is smaller. Rolls draw from
    the run's **levels stream** (`WorldState::levels_rng`), never the gameplay or training
    stream. Said: "Your stealth improves." (never the die or the marks). The dev `levelup:auto`
    spends every roll on the first trained skill.
  - **Active uses**: skills get no keys of their own. Racial, class, skill, and style abilities
    are chosen from one abilities menu (§6 "Abilities", #156), unlike ADOM: each trained skill
    with an active use (`kind = "action"`) is an ability there (`warlock_core::ability`,
    `PerceptionView::abilities`), greyed out as "not usable yet": the uses come with their
    actions, and one whose use has come is that ability instead (Disable device's is "Disarm a
    trap", §8 "Disarming traps"; Open lock's "Pick lock", §8 "Doors"). Never a benefit's number,
    a mark, a
    chance, an untrained skill, or a creature's skills; ground-truth dev dumps (`actors`, the
    world state) show every actor's skills with their marks and the ladder's die for them. The
    Skills screen's meter shows a skill's ladder step in quarters (how practised it is). Skills
    and marks are part of the world state (saved, hashed).
- **Talents** (`warlock-core/src/talent.rs`, the runtime; `content/talent.rs`, the data;
  `world/talent.rs`, the picks; M30, #229; design `docs/design/progression.md` §7, §10): knacks
  a character picks as they level, each a typed effect read where the rules already read what
  magic items do.
  - **Data** (§7): `data/talents.toml` (the settings: `max_ranks`, `[coverage]`, the `[[tree]]`s
    in display order, the closed `[[system]]` list) and `data/talents/<id>.toml`, one `[[talent]]`
    a file (`id`, `name`, `tree`, `description` in the game's words, `ranks` 1 to `max_ranks`,
    `choice` (`weapon_skill`, `style`, `element`), `needs`, `effects` per rank held,
    `[[rank]]` with each rank's own `needs` and `effects`, `systems` from the list, `granted`:
    given, never picked; `flaw`: a birth sign's flaw, granted, never a prerequisite). The first
    tree, **Stars** (`no_free`), holds the birth signs' gifts (§8 "Birth sign"). Without the
    files there are no talents and no level asks for one.
  - **When**: every `talent_every`th character level (`[levels]` in `data/experience.toml`: 3,
    6, 9, …) brings one pick, a `Due { level, talents, rolls }` in `Progress::due` (lowest level
    first; several levels in one turn queue theirs), handed out by `World::level_up` from
    `grant_xp`. **Nothing is banked**: while a pick is due `World::apply` refuses every other
    action ("You must choose a talent first."), so the level-up screen (to come, #236), the
    CLI, and replays spend it alike with `Action::Talent { id, choice }` (`talent:hardy`,
    `talent:weapon_focus:long_blades`; no time, recorded) and the level's skill rolls with
    `Action::Advance` (`advance:stealth`, "Skills" above); the dev `levelup:auto` takes the
    first the player may take for every choice due, the dev `xp:N` grants experience. A pick
    nothing can be spent on (no talent the player may take: a run without talents, every
    talent they qualify for held, with no skill roll left that might open one) is dropped, so a
    level never waits for the impossible.
  - **Prerequisites** (`Needs`; all must hold at the pick and are never checked again: a score
    lowered later keeps the talent): other talents at a rank (a talent with a choice may be
    needed only by one of the same choice, taken for the same), stat **scores** (the stored
    score, never the effective one), skill values, weapon skill or style ranks, the chosen
    weapon skill's or style's rank (`chosen`), the character level, the race, the class, the
    eyes (the race's vision profile), the birth sign's animal and element and a lucky day
    (`animal`, `element`, `lucky`; §8 "Birth sign"), and the alignment band
    (`requires_alignment`, as quests and abilities ask it, §8 "Alignment": `{ min = "good" }`).
    Refused in words: "You cannot
    take Dodge: you need Dexterity 13." Ranks come one at a time ("You learn Hardy (2)."); a talent with a choice is
    one pick per choice, each with its own ranks.
  - **Effects** (`TalentEffect`, summed into `TalentMods` by `Actor::talent_mods`, one read
    each): `dodge` and `natural` in Defence (natural armour on top of the creature type's own
    or a magic item's, or alone), `melee_damage` on both hands' blows (after the affixes,
    before the stance), `max_hp_per_levels` (hit points handed into `hp_base` and `hp` at the
    pick and as each later level comes, as levels' own are; the floor `at_least` until
    `level / per_levels` passes it), `healing_percent` (with the regeneration affixes),
    `footsteps_percent` and `silent_doors` (`noise::noise`), `hearing`, `carry_percent`
    (`burden::load`), `travel_percent` (overworld steps), `hunger_percent` and `no_exertion`,
    `xp_percent` (every gain), `notice` (noticing in passing as if trained in Spot, with a
    bonus), `free_blow` (a switch of `World::free_blows_of`); with the birth signs (#265):
    `max_dex` (the burden's Dexterity cap in Defence), `protection` (added to the Protection
    sum), `threat` (every attack's threat range, in `Actor::stanced`), `reroll_fumble` (a natural
    2 rolled again once, `combat::strike_with`), `ease { of, times }` (bleeding or poison eases
    that many times as fast, `World::live`), `poison_spares_scores` (`Actor::penalty`),
    `disease_checks` (both disease checks), `swimmer` (`step_price` on wading terrain, `wade`),
    `speed_percent` (`Actor::speed`, every read of the scheduler), `rot_harm_percent`
    (`eat_rotten`), `stone_sense` and `search` (`search.rs::notices`, `search_modifier`),
    `search_reach`, `kick` (`kick_door`), `dig_percent` (`dig_turn`), `favour_percent`
    (`world::favour`). The first twelve: Healthy, Hardy, Dodge, Brawn, Silent feet, Keen ears,
    Pack mule, Long strider, Lean, Good learner, Alert, Thick skin.
  - **Coverage** (the design's §8): every talent names the systems it touches; at load a
    system not `pending` with fewer than `min_talents` talents (or `min_class` class abilities,
    0 until the class tables), or a tree not `pending` nor `no_free` without `free_per_tree`
    prerequisite-free talents, is a content error. A wave that ships a system's talents removes
    its `pending`; a new system of the game joins the list in the PR that ships its talents.
  - **Determinism and saves**: picks are replayed actions; nothing here draws from the gameplay
    stream; the `levels` stream (`WorldState::levels_rng`, for the skill rolls and stat checks
    to come) is made on its first draw. `Actor::talents` (by id, like affixes: adding talents
    keeps saves; on every actor, so creatures may get some from data later), `Progress::due`,
    and the stream are left out of the JSON while unused, so runs without talents hash as
    before. **Saves change.**
  - **What the player sees**: `PerceptionView::due`, `own_talents` (names for their choice,
    the rank of how many, the description), `talents` (every talent with the next rank's
    prerequisites and whether each holds, and the ways it may be taken now); the dumps'
    `talents` and dev `progress` layers; JSON `talents`, `due`. Never a rule's number: the
    wiki's Talents page states them.
- **Weapon skills** (`warlock-core/src/weapon_skills.rs`, training; `content/weapon_skill.rs`,
  the data; M22, #134), ADOM-like, with *The Riddle of Steel*'s defaults:
  - **Skills** (`data/weapon_skills.toml`, §7): the categories weapons belong to. A skill's
    role says what uses it: `melee` and `launcher` skills are named by weapons (`category`), the
    wielded weapon's for a blow, the readied missile weapon's for a shot; `thrown` is every
    throw, whatever is thrown; `unarmed` bare hands, natural attacks, and kicks, whatever is
    wielded (§8 "Doors"); `shield_block` and `shield_bash` a held shield's blocks and bashes
    (`Actor::attack_skill`).
  - **Ranks**: 0 (untrained: no benefits; marks still count) to 10, on every actor
    (`Actor::weapon_skills`, a rank and marks per skill): the player's from race and class, a
    creature's from its type (§7). Ranks are stored, not derived.
  - **Marks**: only the player trains. Each attack the player makes on someone (a blow, a bash,
    a kick, a shot or throw for each actor it reaches; hit or miss) earns `attempt` marks (1) in its
    skill, a hit `hit` more (2); a blow the player's shield blocks (a miss credited to the
    Shield category, "Combat" above) earns `attempt + hit` in shield blocking. The marks scale
    by `100 + per_level × (the opponent's level − the rank)` percent (20 a level), kept within 0
    and `most` (200), rounded to the nearest mark (halves up): more against stronger
    opponents, fewer as the rank rises. The opponent's level is its creature type's `level`
    (§8 "Experience"; the one attacked, or for a block the attacker): a level-1 giant rat
    teaches nothing past rank 5, a level-4 kobold shaman nothing past rank 8.
    A missile that reaches no one, and spells, earn nothing; nothing rolls for marks.
  - **Curve**: the marks to go from a rank to the next grow exponentially, by the skill's
    `curve`: melee 15, 30, 60, … 7680 (rank 10 at 15 345 in all), missile 10, 20, … 5120. Marks
    past the need count toward the next rank (one rank at a time); rank 10 earns no more.
  - **Defaults** (The Riddle of Steel): a symmetric table of penalties between related skills
    (§7; long blades 8 gives short blades 6). When skill A gains a rank, every skill B with
    `rank(A) − penalty(A, B) > rank(B)` rises to that rank and its marks start again from 0
    (`WeaponSkills::raise`); defaults do not chain within one gain. Starting ranks raise their
    defaults the same way, whatever their order (`WeaponSkills::from_ranks`).
  - **Benefits** (per skill and rank, data; never shown): to-hit and damage (to every roll of
    the dice) on attacks with the skill, a wider threat range (lower natural rolls threaten,
    never below 2), and a lower cost (`speed`, percent off `melee`, `shoot`, or `throw`;
    `reload` off reloading), all in the attack's `Profile` (`Profile::with`; `Actor::melee`,
    `ranged`, `thrown`, `bash`) and price (`World::attack_price`); Parry added to the wielded
    weapon's own while it is wielded, Shield (shield blocking) added to a held shield's own
    (`Actor::defence`). Benefits of several skills add up (`Benefit::plus`), so a later kind of
    skill (fighting styles, #135) can stack on a weapon's: another role, with the same ranks,
    marks, curve, and benefits.
  - **Shield bash** (`Action::Bash(dir)`, replay text `bash:DIR`; an ability, §6 "Abilities":
    chosen from the abilities menu, or a key once bound, then a direction): a melee blow at whoever stands that way with
    the held shield's `bash` dice (§7; the small shield 1d3) plus the Strength modifier to hit
    and to damage, drawing on and training shield
    bashing; it costs a blow with the shield: `melee` (faster by the shield's own attack speed
    and what the player wears, never the wielded weapon's `attack_cost` or affixes; less shield
    bashing's `speed`), and makes a blow's noise. Refused,
    taking no time: without a shield to bash with ("You have no shield to bash with."), with
    nobody there ("There is nothing there to bash."), or at someone not hostile ("You have no
    quarrel with the deer."; unseen: "Something is in the way."). Its event is an attack of
    kind `Melee` made `with` the shield; creatures do not bash.
  - **Messages**: each skill that rose, its own rank or by default, is said after the attack
    that earned it (and before the experience a kill earned), by its `phrase`: "You feel more
    skilled with long blades." No numbers.
  - **What the player perceives**: their own ranks and marks
    (`PerceptionView::own_weapon_skills`), shown as ranks and a bar on the weapon skills screen
    (§6) and in the `weapon_skills` dump layer; never a benefit or a creature's ranks (the
    `actors` dev layer shows every actor's). Ranks and marks are part of the world state
    (saved, hashed).
- **Fighting styles** (#135; `content/weapon_skill.rs`: `Role::Style`, `Grip`, `Style`,
  `TwoWeapons`; `world/actor.rs`: `Actor::grip`, `style`, `styles_in_use`, `attack_skills`
  (`AttackSkills`: an attack's weapon skill and style), `attack_benefit`, `off_hand`,
  `off_hand_blow`), after AD&D 2e's *Complete Fighter's Handbook*: how weapons are
  held and used, as skills with the weapon skills' ranks (0 to 10), marks, curves, defaults, and
  benefits ("Weapon skills" above), in `data/weapon_skills.toml` (§7).
  - **Configurations** (`grip`): for blows, what the hands hold (`Actor::grip`, which counts for
    Defence too): nothing (`unarmed`), one one-handed weapon and every other hand empty
    (`one_weapon`), a weapon in each of two hands (`two_weapons`), a two-handed weapon
    (`two_handed`), a one-handed weapon and a shield (`weapon_and_shield`); any other hold (a
    shield alone) is no style. A shot is `launcher` (whatever the hands hold), a throw `thrown`.
    A kick (§8 "Doors") is made whatever the hands hold: it draws on unarmed fighting alone and
    is fought in no style, so Brawler gives it nothing and its price is a kick's (decided: a
    style is how the hands are used).
    A style may name the weapon skills it is fought with; the style of a grip that names none
    takes every other weapon (`WeaponSkillTable::style_for`), so one configuration is always at
    most one style. A spell has none.
  - **The eight** (data): **Duelist** (`one_weapon`: to-hit, Parry), **Twin Blades**
    (`two_weapons`: eases both two-weapon penalties, none left at rank 10), **Great Weapon**
    (`two_handed`, any weapon but a staff: damage, from rank 8 a critical rolls its damage once
    more, cheaper blows), **Sword and Board** (`weapon_and_shield`: Shield, from rank 3 a chance
    of a shield bash after a blow that hits), **Marksman** (`launcher`, bows, crossbows, and
    slings: to-hit, cheaper shots and reloading), **Skirmisher** (`thrown`, anything thrown:
    to-hit, damage, cheaper throws; decided as throwing only: "a light weapon and light armour"
    would have overlapped Duelist and Twin Blades, and the issue's benefits are all throws'),
    **Brawler** (`unarmed`: to-hit and damage), **Staff Fighter** (`two_handed`, staves: to-hit,
    Parry). Defaults: duelist ↔ sword and board −3, duelist ↔ twin blades −3, great weapon ↔ staff
    fighter −3, marksman ↔ skirmisher −4; never to a weapon skill.
  - **Benefits** (never shown) add to the weapon skill's (`Benefit::plus`;
    `Actor::attack_benefit`): to-hit, damage, threat, and the critical multiplier (`crit`) in the
    attack's `Profile` (`Actor::melee`, `off_hand_blow`, `ranged`, `thrown`, `bash`), right after
    the weapon skill's: the effective stats (§8 "Magic items", "Status effects", "Hunger",
    "Diseases"), the weapon skill's benefit, the style's (with the two-weapon penalty it eases),
    the affixes, then the combat stance; `speed` and `reload` in its price (`World::attack_price`,
    which takes the attack's weapon skill and style: `World::attack_cost`, the weapon's
    `attack_cost` and magic attack speed, less the speed of both together, then the stance and
    the burden; priced as the attack begins, so a rank it earns quickens only the next, §8
    "Time"), Parry
    added to the wielded weapon's and Shield to a held shield's while the hands are in the
    style's grip (`Actor::defence`). A style's rank never helps another configuration.
  - **Training**: every attack the player makes (a blow, an off-hand blow, a bash, a shot or a
    throw for each actor it reaches) earns marks in its style as in its weapon skill, by the same
    rules (`World::resolve`: the weapon skill first, then the style, so their messages come in
    that order: "You feel more skilled at dueling."). Rank 0 trains like any other.
  - **Two weapons**: the off hand takes a shield or a light weapon (`light_weapon`, §7: the
    dagger, the stiletto, the short sword; `Content::fits`). With a weapon in each hand, every melee attack (the
    player's move into a hostile, a creature's) is two blows, the main hand's then the off hand's
    (`World::second_blow`), each at its penalty (`[two_weapons]`: main −4, off −8) less what
    Twin Blades eases (`TwoWeapons::eased`, never past 0). The off hand's blow is an attack made
    `with` its weapon (`Event::Attacked`), draws on and trains that weapon's skill, and is
    worded with it ("You parry the kobold's dagger."); it makes no noise of its own (the attack
    made one) and comes only if the target still stands. Each blow is resolved on its own: its
    own weapon's skill and affixes (§8 "Magic items"; the affixes of everything worn and wielded
    count for both), its own blood, and the bleeding of a big hit or a critical. Both are one
    action at the main hand's price, and hunger's exertion is counted once. Creatures that wield
    two weapons fight with both.
  - **Shield bash after a hit**: in the `weapon_and_shield` grip, a blow that hits a target that
    still stands is followed, by the style's `bash` chance (one gameplay draw, only when the
    chance is above 0), by a shield bash at it, free and silent, drawing on and training shield
    bashing and the style.
  - **Rank 10** is where a style's later active or triggered abilities will hang (a duelist's
    riposte, a staff fighter's trip); they will go into the one abilities menu (§6 "Abilities"),
    with no key of their own.
  - **What the player perceives**: their own ranks and marks (as weapon skills), and which
    styles they fight in now (`PerceptionView::own_styles_in_use`), on the weapon skills screen
    (§6) and in the `weapon_skills` dump layer (`Duelist 1 (0%), in use`; JSON `styles_in_use`).
    Never a benefit, a penalty, a chance, or a creature's ranks.

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
    each names. The gifts whose rules exist so far: Fleet-footed and Ponderous (speed ±5 %), Slight and
    Roc's strength (carrying −10 % / +25 %, Roc's strength also kicks doors as if Strength +4),
    Regrowth (healing +50 %, bleeding eases twice as fast), Strong swimmer (wading costs a plain
    step and never gives a disease), Pure blood (disease checks +4, poison eases twice as fast
    and never lowers the scores), Heedless and Always a way (−4 / +3 to every check to find
    hidden things; Always a way also searches a cell farther), Riddler's mind (XP +10 %), Little
    loved and Grim company (shop favour −10), Supple (Dodge +1, the burden's Dexterity cap +1),
    Evergreen (hunger −15 %, rotten food's sickness halved), Stone hide (Protection +1),
    Stonewise (notices on stone in passing, digging +25 %), Keen edge (threat +1 with every
    attack). From the Salamander to the Basilisk (#266): Slip away (a step out of reach of
    every hostile beside, −25 %, whatever the stance), Many minds (flaw: Willpower checks −2),
    Relentless (each melee blow at the creature the last was at, in the same turn or the next,
    +1 to hit per blow before it, to +3; `Actor::streak`), Barbed hide (a creature whose melee
    blow hurts the character takes 1 damage, no roll, and earns nothing if it dies of it),
    Horn's grace and the Transfixing stare (the actives below), Thin-skinned (flaw: a big hit
    that wounds wounds a degree more), Trailblazer (overworld journeys −20 %, forest, hills,
    and swamp as plains), Flame-hardened (fire resistance 25 %: damage types are in, so the
    resistance form at once, not the design's burst halving; a Fire Wyvern's Kindled adds its
    own 25 %), Heavy-footed (flaw: footsteps
    +25 %), Unblinking (notices in passing as if trained in Spot, +2), Trusted by none (flaw: no
    one makes way, Noblesse or not).
    The elements' last four (#268): Blaze (Fire's **active**, from the abilities menu's
    **Birth sign** group, `use:blaze`, the time of a `cast`, refused on the overworld: a burst
    of radius 1 around the player, one roll of 2d6 fire for every creature in it but them, whole
    to each, through the scrolls' cast code (Evasion and resistances apply, the kills are the
    player's); it melts the ice it reaches and leaves a burning missile's fire, `arrow_fire` for
    60 s, on every walkable cell of it but the player's; then the cooldown status Burnt out,
    twelve hours, during which the menu greys it "you are burnt out"), Kindled (the lantern burns
    as Lamplighter's brighter preset, a cell farther; with Lamplighter too, no farther; and fire
    resistance 25 %, #271), Forge-born (marks +50 % in every weapon skill and fighting style,
    added to Weapon drill's; in words "you learn weapons quickly"), Tidal (mana regeneration
    +25 %, added to the affixes' percent in `World::mana_rate`).
    The rest of the design's gifts land with their rules and are listed in `grants` only then.
  - **Birth sign actives** (#266): used from the abilities menu's **Birth sign** group
    (`Ability::Sign`, `Source::Sign`, between Racial and Class) as `Action::Gift` (no key of
    their own), each refused (no time) without it or while its cooldown status lasts. **Horn's
    grace** (Unicorn; `use:horns_grace`, the time of a wait): Bleeding and Poisoned end and the
    first disease that shows is shaken off, then **Graceless** for a day (three degrees of
    eight hours); refused with nothing to mend. **Transfixing stare** (Basilisk; `use:transfixing_stare:X,Y`, aimed at a cell in
    the targeting mode, within 6, the time of a wait, silent): a hostile the player sees there
    that sees them now (creature sight) is **Transfixed** (2 degrees, every action at 200 %, a
    degree easing every 12 s: about its next two actions), then **Spent-eyed** for two hours
    (two degrees of an hour);
    one that does not see them is not held ("The kobold does not meet your gaze.", the time
    taken, no cooldown); refused at the player's cell, out of range, where they see no one, and
    at anyone not hostile. Nothing is drawn. **Blaze** (Fire's, #268) is the third, described
    above; **Fire breath** and **Hound's bay**, two signs' picks, the fourth and fifth (below).
  - **The Roc to the Barghest** (#267): Eyes of the heights (overworld sight +2 cells by day, +1
    by night), Hoard sense (loot's better rarities +50 %, a find a loose gem +100 %: the same
    draws, so maps never change), Watchful (hears 3 fainter; footsteps it hears are placed at
    their cell, not roughly), Set in its ways (flaw: changing stance takes a wait's time), Old
    lore (a magic item carried 24 hours is identified; reading −50 %), **Rise from the ashes**
    (below), Inner light (an `ember` light, radius 1, always shed at the player's cell: they see
    the cells beside them in the dark, and so does everything else), Frail (flaw: maximum hit
    points −10 %, at least 1), Grasping arms (Twin Blades' two-weapon penalties eased by 3; Sword
    and Board's free bash +20 % where it has one), Of the deep (ice holds them as if they
    carried nothing, a plunge never hurts, climbing out onto ice −50 %, and cold resistance 25 %, #271), Ungainly ashore (flaw:
    overworld journeys +10 %), Hound's nose (bleeding creatures seen within 6 cells however dark;
    the eat list says whether food smells fresh, turning, or rotten: a word, never its age),
    Gatekeeper (a free blow at a hostile stepping up beside them, §8 "Free blows").
  - **Rise from the ashes** (the Phoenix's): **once in a life**, a hit, a trap, or a status that
    would bring the player to 0 hit points does not kill them (starving and choking still do):
    "You burst into flame and rise from your ashes!"; they are **Ashen** for a day (every score
    −2, hit points back half as fast), at a third of their (Ashen) maximum hit points, and flames
    burst from them (the `phoenix_fire` effect over the cells within 1): one roll of 1d6 fire for
    every creature there (no roll to hit; a kill is the player's), and the ice there melts. The
    rising is kept in the saved progression (`Progress::once`, `risen`), so it never comes again;
    the Stars line and the talents say "Rise from the ashes (spent)". With **Second rising** (a
    pick, below) it comes once more (`risen_twice`), and only then is it spent. A Toughness
    drained to 0 by
    a disease still kills (the rising would leave them dying of it). The flames draw from the
    gameplay stream only when they come.
  - **Sign-limited picks** (#270; design §9.1): the Stars tree also holds one talent per animal
    and per element that is **picked**, not granted: offered on the level-up screen and the
    creation Talents step like any talent whose prerequisites hold, the sign among them
    (`needs = { animal = "wyvern", level = 15 }`, said "to be born in the year of the Wyvern",
    "to be Wood-born"). The Talents page leaves out the picks of other signs (the stars never
    change); the wiki lists them all. Hare-footed (Salamander, level 12: speed +5 % more),
    Another head (Hydra, 9: hit points come back while Starving, resting heals +100 %), Relentless
    fury (Manticore, 9: the streak runs to +5, and every two blows before it +1 damage), Horn of
    plenty (Unicorn, 9: Horn's grace also gives back a hit point per level, never past the
    maximum, so a wound alone is cause enough, and leaves one degree of Graceless, eight hours),
    **Fire breath** (Wyvern, 15: an active, `use:fire_breath:X,Y`, aimed at a cell within 8, the
    time of a `cast`, refused on the overworld: a bolt along the line toward it and on, 4 cells
    or to the first wall, one roll of 2d6 fire for everyone on it through the scrolls' cast code,
    the `fire_breath` effect; then two degrees of Drained, ten minutes, shared with Lay on hands),
    Petrifying gaze (Basilisk, 12: the stare leaves 4 degrees of Transfixed and one of
    Spent-eyed, an hour), Roc's grip (Roc, Strength 14: carrying +25 % more, Burdened costs
    nothing), Dragon's hoard (Griffin, 12: Hoard sense's odds half again, loot +25 % and gems
    +50 %, shop favour +10 %), Riddle of the Sphinx (Sphinx, Intelligence 14: a skill roll more
    every level), Second rising (Phoenix, 18), Tentacles (Kraken, Twin Blades 3: off-hand blows
    +2 damage), **Hound's bay** (Barghest, 12: an active, `use:hounds_bay`, the time of a wait,
    refused on the overworld: a howl, `Act::Howl`, loudness 150, heard as "a howl" (`h` in the
    sound layer); every hostile that hears it by the hearing rules and whose creature level is
    below the player's is **Frightened**, 3 degrees easing a turn apiece, caused by the player;
    then **Hoarse** for an hour; no check, nothing drawn), Deep roots (Wood, 9: Dodge +1, rotten
    food eaten as by a race that eats it), Wildfire (Fire, 15: Blaze out to radius 2 for 3d6),
    Bedrock (Earth, 12: Protection +1, pits never catch them), Tempered (Metal, a weapon skill
    at 5: threat +1 with the weapon skill they hold at the highest rank, ties all counting; not
    styles or shield skills), Undertow (Water, 6: wading costs a plain step, a plunge through
    the ice never hurts; a disease may still be caught wading). A **frightened** creature (a
    status that `flees`) steps away from whoever frightened it, to the free cell beside it
    farthest from them (Chebyshev, then straight-line distance, the first in direction order on
    a tie), while a step takes it farther; cornered, it fights as it would have.
  - **Lucky days** (`[[lucky]]`): five animal–element pairs, each with a day of the month (any
    month), the day the animal's place in the Great Race: the Wood Salamander on the 1st (the
    Racer), the Metal Manticore on the 3rd (the Iron Scourge), the Fire Phoenix on the 10th (the
    Firebird), the Water Kraken on the 11th (the Deep One), and the Earth Barghest on the 30th
    (the Gate Warden). Born in that year **and** on that day, a character has the title after
    their name (the character and death screens: "Brakka, the Iron Scourge") and one gift more:
    the Racer's Winner's luck (a natural 2 on the attack roll is rolled again once, two more
    draws only then; speed +5 % more), the Iron Scourge's Scourge (a melee blow that kills
    strikes at once at another foe beside them: Cleave, granted), the Firebird's Reborn whole
    (rising from the ashes brings them back whole: every hit point, every status effect and
    disease burned away, nothing following them, and no Ashen), the Deep One's Drowning grip
    (a creature they hit in melee is **Held**, a status that `blocks` its steps: it cannot step,
    and beside its holder it attacks them instead, until they miss it, strike another, or move,
    going to another level too; no roll, no timer), and the Gate Warden's Warden of the gate (a
    foe stepping away from beside them takes a free blow: Opportunist, granted; Protection +1;
    what status effects take off their scores halved, rounded toward 0). About one character
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
  - **Light-blindness** (#255; a race's `light_blind = { glare, sight, to_hit }`, drow 250, 10,
    −1): while the light on the player's own cell, as their eyes perceive it (the light the
    perception finds there, the status bar's `Light`), is at least `glare` bright, they are
    **dazzled**: the perception shows nothing farther than `sight` cells (line of sight and
    light are unchanged within it; wall completion stops there too), and every attack they roll
    is `to_hit` worse (`World::glared`, in `resolve`, from the same light on their cell then).
    `LightBlind::dazzled_by` is the one rule; nothing else reads it. A clear sun well above the
    horizon dazzles a drow (on the courtyard's November morning from about 07:30), an overcast
    one does not, nor their own lantern alone (about 220); a lantern beside a brazier does. A note says when it begins ("The bright light dazzles
    you.") and ends ("Your eyes adjust to the dimmer light."), neither stopping a rest, and the
    status bar shows `Dazzled` while it lasts; never the numbers. Creatures are never
    light-blind.
  - **Gifts on other rules**: a race's `natural` armour adds to the player's Defence (Natural)
    and Protection, as a creature type's does ("Combat" below; lizardfolk); its `resist` to what
    they resist ("Damage types"; drow poison); its `disease_check` to every check against a
    disease ("Diseases"; ratfolk); `eats_rotten` lets rotten food feed them unharmed ("Hunger";
    ratfolk); and `passive_search = "traps"` lets them notice hidden traps in passing
    ("Searching"; kobolds). `WorldState::race_of` is the one lookup of the player's race;
    creatures go by their creature type.
  - **Race and creatures**: no creature takes a side by the player's race: hostile kobolds
    attack a kobold player as they would anyone, and good creatures greet a drow coldly only
    when their alignment is Evil or worse ("Alignment" below), never for their race. Tribal
    friendliness may come later.
  - **Hit points**: the race's and the class's `hp` add to Toughness for the maximum ("Combat"
    below).
  - **Alignment** ("Alignment" below): the starting score is the race's `alignment` plus the
    class's, kept on the scale: paladins start Good, thieves and witches Callous, everyone else
    Neutral; most races only lean (elves and halflings a little toward good, half-orcs a little
    toward evil), never alone out of Neutral, but the evil races (#255) start on the evil side:
    goblins, kobolds, lizardfolk, and ratfolk Callous (−200), drow and duergar Evil (−450), so a
    drow paladin starts Neutral and a goblin thief Evil. The birth animal **nudges** it (#272;
    `alignment` on an `[[animal]]` in `data/horoscope.toml`, elements none): Unicorn +100,
    Griffin, Phoenix, and Wyvern +50, Kraken −50, Basilisk and Manticore −100, the rest 0; added
    before the summary shows the band, and never shown as a number. Small against a band's width,
    it still decides the band near an edge: a Manticore- or Basilisk-born human paladin starts
    Kind (350), a Unicorn-born elf Kind (150). It draws nothing.
  - **Mana**: the race's and the class's `mana` add to the mental stats' modifiers for the
    maximum (§8 "Mana").
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
    classes adjust starting scores, give hit points, train skills, give weapon skills and
    fighting styles, give a starting kit, set where the alignment starts, and (those with a
    level table) give class abilities and bonus talent picks as their levels come ("Class
    abilities" below).
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
    leaving a shop with unpaid goods, or taking something with Steal unnoticed (theft, −100 each time); paying a
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
    something hidden in passing (`spot`; §8 "Searching"), an attempt to steal, whatever comes of
    it (`steal`; "Stealing" below). Only the player trains for now; creatures keep their type's
    skills. Marks are a **reservoir**: they raise nothing by
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
    trap", §8 "Disarming traps"; Open lock's "Pick lock", §8 "Doors"; Steal's "Steal",
    "Stealing" below). Never a benefit's number,
    a mark, a
    chance, an untrained skill, or a creature's skills; ground-truth dev dumps (`actors`, the
    world state) show every actor's skills with their marks and the ladder's die for them. The
    Skills screen's meter shows a skill's ladder step in quarters (how practised it is). Skills
    and marks are part of the world state (saved, hashed).
- **Stealing** (`world/steal.rs`, data `data/steal.toml`, §7; #234, the progression design's
  B7): the Steal skill's use, the abilities menu's **Steal** (`Ability::Steal`, in the skill's
  place among the Skills; a direction; `Action::Steal`, replay text `steal:DIR`; cost `steal`;
  a rebindable key, "Steal", unbound by default). Anyone trained in a skill with the `steal`
  benefit has it, while `data/steal.toml` exists (without it nobody steals and the skill stays
  "not usable yet"); the thief's class trains Steal.
  - **Whom**: a creature on the cell in that direction that the player sees and that is not
    hostile to them (`World::stance`). Refused, taking no time: untrained ("You do not know how
    to steal."), no one seen there ("You see no one there to steal from."), or someone hostile
    ("The kobold is hostile: it would never let you near enough.").
  - **The contest** ("Checks" in §8): 2d12 + the player's Dexterity modifier + what Steal gives,
    against 2d12 + the victim's Willpower modifier (both of the effective scores); a tie goes to
    the thief, and their natural 2 always fails, their natural 24 always works. Four draws of
    the gameplay stream (the victim's roll first). It takes the `steal` action's time whatever
    comes of it (priced before anything changes hands, so the burden applies) and may earn
    Steal a training mark.
  - **Unnoticed**: the player takes one stack, all of it, of what the victim carries and neither
    wears nor wields, picked uniformly (one more gameplay draw when there are two or more), "You
    steal 7 copper coins from the kobold."; with nothing like that on it, "You find nothing to
    take from the gardener.", unnoticed all the same. There is no choice of item: the player
    never sees a creature's pack (`PerceptionView::gear` is only what it wears and wields), and
    what is worn or wielded cannot be lifted unnoticed. `World::got_away_with` is the one place a
    theft that took something has consequences beyond the goods: it is a theft deed for alignment
    (§8 "Alignment": stealing is evil, as leaving a shop owing is).
  - **Caught**: the victim turns on the player. A keeper of an open shop (§7 "Shops") is outraged
    as a robbed one is: "Grimbold catches you stealing. \"Stop, thief!\"", its shop trades no
    more, and what the player owed it is stolen. Anyone else joins `data/steal.toml`'s `caught`
    faction (its `faction_override`, hostile to the player), so the AI and every stance follow:
    "The kobold catches you stealing and turns on you!".
  - **Shop goods** are not stolen this way: what lies in a shop stays the shop's rules' (picking
    it up is owing for it; leaving owing any way but the door is theft, §7 "Shops"). Stealing
    from a keeper takes from its pack: its purse.
  - No limit on trying again (the owner's rule): a victim that caught the player stays angry,
    and that is the price. Presentation shows only what comes of it, never a modifier or a roll.
- **Talents** (`warlock-core/src/talent.rs`, the runtime; `content/talent.rs`, the data;
  `world/talent.rs`, the picks; M30, #229; design `docs/design/progression.md` §7, §10): knacks
  a character picks as they level, each a typed effect read where the rules already read what
  magic items do.
  - **Data** (§7): `data/talents.toml` (the settings: `max_ranks`, `[coverage]`, the `[[tree]]`s
    in display order, the closed `[[system]]` list) and `data/talents/<id>.toml`, one `[[talent]]`
    a file (`id`, `name`, `tree`, `description` in the game's words, `ranks` 1 to `max_ranks`,
    `choice` (`weapon_skill`, `style`, `skill`, `element`), `needs`, `effects` per rank held,
    `[[rank]]` with each rank's own `needs` and `effects`, `systems` from the list, `granted`:
    given, never picked; `flaw`: a birth sign's flaw, granted, never a prerequisite). The first
    tree, **Stars** (`no_free`), holds the birth signs' gifts (§8 "Birth sign"). Without the
    files there are no talents and no level asks for one.
  - **When**: every `talent_every`th character level (`[levels]` in `data/experience.toml`: 3,
    6, 9, …) brings one pick, a `Due { level, talents, rolls }` in `Progress::due` (lowest level
    first; several levels in one turn queue theirs), handed out by `World::level_up` from
    `grant_xp`. **Nothing is banked**: while a pick is due `World::apply` refuses every other
    action ("You must choose a talent first."), so the level-up screen (§6 "Level-up"), the
    CLI, and replays spend it alike with `Action::Talent { id, choice }` (`talent:hardy`,
    `talent:weapon_focus:long_blades`; no time, recorded) and the level's skill rolls with
    `Action::Advance` (`advance:stealth`, "Skills" above); the dev `levelup:auto` takes the
    first the player may take for every choice due, the dev `xp:N` grants experience. A pick
    nothing can be spent on (no talent the player may take: a run without talents, every
    talent they qualify for held, with no skill roll left that might open one) is dropped, so a
    level never waits for the impossible.
  - **Creation picks** (`world/creation.rs`; #236): a character the player chose (a run with
    its starting kit: New run, `--character`, `--name`, `--talents`) picks `[levels]
    creation_talents` talents (2) at creation and its class's `creation_talents` more (the bard,
    the lucky class: 1). They are a choice due at level 1 while the run is made, spent at once
    by `World::start_run` on the picks `CharacterSpec::talents` names (`id` or `id:choice`, in
    order, each checked like a level's pick: an unknown, refused, missing, or extra pick fails
    the start), else on the **Random** picks: one by one, a talent the character may take then,
    uniformly, and a choice of it, from the stream `character/talents/<race>,<gender>,<class>`
    (a pure function of the seed and the character). What was taken is recorded in the
    character's `talents` (saves, replays: `character.talents`), and nothing is said. A run on
    the scenario's own character without the kit (a dev launch, tests, walks) picks none, and a
    replay without `talents` starts as it did before. Prerequisites see the run as it starts
    (scores, skills, the class's level 1 abilities, the sign's gifts), and picks may chain
    (Healthy, then Fast healer). Character creation's Talents step (§6 "New run") reads the run
    drafted (`World::draft`, `creation_offers`, `random_creation`). The humans' extra pick is
    their racial Versatile's (#241): a human picks 3, a human bard 4.
  - **Prerequisites** (`Needs`; all must hold at the pick and are never checked again: a score
    lowered later keeps the talent): other talents at a rank (a talent with a choice may be
    needed only by one of the same choice, taken for the same), stat **scores** (the stored
    score, never the effective one), skill values, weapon skill or style ranks, any one of
    several weapon skills at a rank (`any_weapon_skill = { roles = ["melee", "unarmed"], of =
    [...], rank = 3 }`: "a melee or unarmed weapon skill 3", "Maces and clubs or Unarmed 3"), the
    chosen weapon skill's or style's rank (`chosen`), the character level, the race, the class, the
    eyes (the race's vision profile), and the alignment band (`requires_alignment`, as quests
    and abilities ask it, §8 "Alignment": `{ min = "good" }`); and any one of several (`either
    = { stats = { intelligence = 12 }, skills = { appraise = 20 } }`: "Intelligence 12 or
    Appraise 20"). Refused in words: "You cannot
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
    bonus), `free_blow` (a switch of `World::free_blows_of`). The first twelve: Healthy, Hardy,
    Dodge, Brawn, Silent feet, Keen ears, Pack mule, Long strider, Lean, Good learner, Alert,
    Thick skin.
  - **Weaponcraft and Guard** (#238): `weapon` adds to the benefits of the chosen weapon skill
    or style (no `role`) or of every skill of a role, read in `Actor::benefit`, so every
    attack, Defence, and price that reads a skill's benefits reads it: Weapon focus (+1 to hit),
    Weapon specialization (+2 damage), Improved critical (threat one roll wider), Stylist (the
    style's benefits one rank higher, never past 10, and only for a rank held), Two-weapon
    training (both two-weapon penalties eased by 2), Shield expert (Shield +1; Sword and
    Board's free bash 10 more likely where it has one), Deadly aim (+2 damage with shots),
    Thrower (+1 damage and 10 % cheaper throws), Rapid reload (reloading 25 % cheaper);
    `launcher_range` and `throw_range` (Far shot: +2 and +1 cells); `max_dex` (Armour
    training: worn armour's Dexterity cap +1 a rank); `deflection` (Deflection +1);
    `protection` (Heavy armour mastery: +1 while the armour piece worn with the most
    Protection has 3 or more; Shield wall: +1 while holding a shield); `stance_to_hit` and
    `flee_percent` (Stalwart: the defensive stance's to-hit penalty gone; Unflinching: the
    coward's halved, rounded toward 0, and fleeing steps 10 % cheaper again); `pain` (Immune
    to pain: a hit, a blow, shot, throw, or trap, does 1 less after the soak, never below 1;
    a status's or starving's harm is no hit); `resist` (Elemental ward: 25 % a rank against
    the chosen element, any damage type that is no physical kind, summed and capped with the
    gear's). A weapon-skill choice is a skill that attacks: never shield blocking (it only
    defends) nor a style (chosen as a style). Steady hand waits for the ranger's focus
    (#225).
  - **Vigour, Might, Finesse** (#237; the design's §7.5 but for the two that wait for class
    resources, Battle trance and Inner calm): `resting_percent` (Fast healer: resting's pace
    half again, `World::healing_rate`); `second_wind` (Second wind, the first **active** talent:
    `Ability::SecondWind` under the abilities menu's new **Talents** group (`Source::Talent`,
    between Class and Skills), `Action::SecondWind` (`second_wind`): 1 hit point per character
    level back at once, at least 8, for the time of a `wait`, then the cooldown status Winded
    (15 minutes; "you are winded" in the menu; refused, no time, without the talent or while
    winded)); `iron_stomach` (rotten food and overeating never make you vomit, no draw for it;
    rotten food's sickness halved, `eat.rs`); `status_eases` and `no_penalty` (Poison
    resistance: Poisoned eases twice as fast and never lowers scores; Clotting's bleeding twice
    as fast: the sufferer's own easing pace in `World::live`); `clotting` (a big hit wounds no
    more, a confirmed critical a degree less, `World::wounded`); `penalty_percent` (Unbowed:
    each status's lowering of a score halved toward 0, raises untouched, `Effects::penalty_for`);
    `disease_checks` (Strong constitution, +2 on catching and on every progression) and
    `disease_recovery` (Fever-proof: a check fewer won in a row shakes a disease off, never
    below 1, never one that does not recover); `free_blow` (Cleave, Great cleave, Riposte,
    Opportunist); `kick_die` (Mighty kick: kicks at creatures roll 1d6) and `door_kick` (Mighty
    kick and Breaker, +4 each to a door kick's Strength); `dig_percent` (Breaker: a quarter more
    work a turn); `two_handed_price` (Iron grip: blows with a two-handed weapon 10 % cheaper,
    `World::attack_price`); `critical_status` (Staggering blow: a critical confirmed in melee
    leaves the target Staggered, `cost = 150` for 6 seconds: its next action half again as
    dear; `World::after_blow`, no draw of its own); `penetration` (Crusher: +2 on blows and kicks
    that can do bludgeoning damage); `burden_dex` (Nimble: the burden's Dexterity cap +1);
    `sure_step` and `pit_proof` (Light step: a trap stepped onto gets a Dexterity check against
    its difficulty first, two draws only with the talent; made, it does not go off and is found;
    at rank 2 pits never catch you; steps only, never a door's trap); `speed_percent` (Quick:
    +5 % a rank, every price divided by it in `World::burdened`, as a higher speed; hidden in the
    game); `missile_dodge` (Deflect missiles: Dodge +2 against shots, throws, and trap missiles,
    `World::target_against`); `spell_damage_percent` (Evasion: bursts and bolts, traps' and
    scrolls', do half, rounded down, before resistances). The statuses these name are checked
    at load; `data/status.toml` loads before the talents.
  - **Class abilities** (#246; the design's §4): a class's level table (`abilities` in
    `data/classes/`, `{ level, talent }` rows) names a talent for each of its levels 1, 6, 12,
    18, 25, 32, 40, 50, and 60: a talent of the **Class abilities** tree (`class = true` in
    `data/talents.toml`; `granted`, never on a pick list) or an ordinary talent it hands out
    (the aristocrat's Riposte at 18, the commoner's Second wind at 32). The player is given each
    at rank 1 as they reach its level ("You gain Fencing." with the level's news), level 1's at
    creation, and again whatever is missing when a run is restored, so a grown table reaches a
    save; one already held (picked earlier) is not given again, and nothing is taken away. What
    a rank hands out once comes with it: hit points by level, and **stat gifts** (`stat`: score
    and potential, Iron will's Presence +1; a Toughness gift raises the hit points with the
    maximum). Its `bonus_talents` (`levels`, `trees`) add a **bonus pick** at those levels
    (`Due::bonus`), of a talent from those trees (none named: any, the commoner); a talent of
    those trees spends a bonus pick before an ordinary one, any other is refused for it ("You
    must choose a talent of Weaponcraft or Delver."), and a bonus pick nothing fits is dropped
    like any other. The everyday folk have theirs (bonus picks at 10, 20, 30, 45, 55):
    aristocrat (Weaponcraft, Delver) Well dressed (favour +10 %, quest gold +25 %), Fencing
    (Duelist one rank higher), Noblesse (anyone not hostile budges, keepers at their posts too),
    Riposte, Connoisseur (descriptions show worth; a magic item carried 24 hours, counted from
    then, is identified), Iron will (+2 on Willpower checks, Presence +1), Commanding
    (Intimidate +20, credit 50 gold), Peerless (+2 on the confirmation roll), Legacy (XP +10 %,
    Presence +1); commoner (any tree) Hard work (60 % off the burden's slowdown on top of
    Balance), Hearty (hunger −10 %), Stubborn (+1 on Toughness checks, poison never lowers
    scores), Big hands (+1 damage with maces and unarmed), Thick skull (Protection +1), Second
    wind, Salt of the earth (healing and resting +25 %), Folk hero (quest XP and gold +50 %),
    Unlikely hero (every stat +1); craftsperson (Weaponcraft, Guard, Delver) Keen appraisal
    (descriptions show worth), Whetstone (+1 damage with melee weapons), Fletching (missiles
    break half as often), Sturdy work (Protection +1 in body armour), Mining (digging +50 %),
    Tinker (the Disarm ability untrained), Jeweller (loose gems found, on levels made from then
    on and in drops, a cut finer), Masterwork (melee threat +1), Artisan (Protection +1 per armour
    piece worn, +2 melee damage); merchant (Delver, Scholar) Haggler (favour +15 %), Deep
    pockets (carrying +10 kg, coins weightless), Eye for quality (what is bought or sold is
    identified), Trade routes (journeys −20 %), Credit (leave owing up to 100 gold: no
    dialog, no theft, the debt stays on the bill), Appraiser (descriptions show worth and each
    gem's), Insured (shops pay +50 %, never more than they ask), Magnate (quest gold +100 %, the
    favour cap +10), Golden tongue (Presence +2, favour +10 % and its cap +10). The favour cap
    is 50 % unless lifted, at most 90 %. Worth shown is the value, which never includes unknown
    magic. The adventurers' tables come with I1–I3. Dumps mark them `Well dressed (class)`;
    JSON `talents[].class`, `due.bonus`. **Saves change** (`Due::bonus`, `Magic::carried`).
  - **Scholar, Grit, Delver** (#240; the design's §7.5 but for the five mana talents, which
    came with #242: Deep well, Clear mind, Meditation, Frugal caster, Blood magic, §8 "Mana"). Their effects share what the class abilities use where the rules
    are the same (`identify_carried`: Lore, a magic item carried 24 hours; `item_worth` and
    `gem_worth`: Keen appraiser; `favour_percent`: Silver tongue +10 %, Haggler +15 %;
    `quest_gold_percent` and `quest_xp_percent`: Well liked, a quarter more of each; `checks`
    and `stat`: Iron will, +2 on Willpower checks and Presence +1). Iron will and Haggler are
    one talent each, a Grit and a Delver talent a class's table may also give (the aristocrat's
    Iron will, the merchant's Haggler). New: `read_percent` (Scroll savant: reading takes half
    the time); `spares_self` (Careful reader: the reader's own burst never catches them, and the
    game does not warn before reading one close); `scroll_kept` (Spell recall: one read in four
    keeps the scroll, one gameplay draw a read with the talent, "The writing does not fade from
    the scroll."); `loot_magic_percent` (Treasure hunter: the weights of the rarities better than
    a find's least half again, so anything found is magic and rare more often and a creature's
    drop, always magic, rare more often; from the same draws of the loot streams, so a map never
    changes, and a level made before the talent keeps its loot); `skill_rolls` (Quick learner: a
    roll more every level); `train_skill` (Apprentice, the `skill` choice, any skill, three ranks
    for each: the chosen skill trained at 1d6+4, or raised by it, rolled from the `levels` stream
    as the rank is taken); `ignore_burdened` (Porter: Burdened costs nothing, as a dwarf's);
    `corpse_stats_percent` and `no_bloating` (Gourmand: a fresh corpse's chance to raise each stat
    doubled, at most certain; a meal stops one short of Bloated, what does not fit is lost, and
    nothing overeats); `step_slowdown_percent` (Sure-footed: half the slowdown the burden and
    Bloated put on steps; other actions as before); `searching` (Keen searcher: +4
    searching) with the birth signs' `search_reach` (a cell farther); `search` and `trap_damage_percent` (Trap sense: +3 to finding hidden
    traps and secret doors, searching and in passing; a trap's damage halved, rounded down, at
    least 1); `door_kick` and `kick_noise_percent` (Lockbreaker: a door kick as if 6 stronger, half
    as loud); `dig_percent` (Tunneler: half again the work a turn). A prerequisite may be `either`
    of several (Treasure hunter: "Intelligence 12 or Appraise 20"). The Scholar and Delver trees
    and the searching, digging, shops, magic items, scrolls, skills, and experience systems are
    held to the bar now; talk and gems with the racial talents (#241).
  - **Birth sign gifts** (#265): the effects only the signs' talents use: `threat` (every
    attack's threat range, in `Actor::stanced`), `reroll_fumble` (a natural 2 rolled again once,
    `combat::strike_with`), `swimmer` (`step_price` on wading terrain, `wade`), `rot_harm_percent`
    (`eat_rotten`), `stone_sense` and `search` (`search.rs::notices`, `search_modifier`),
    `search_reach`, `favour_percent` (`world::favour`). The rest the signs share with the other
    trees' effects: `burden_dex` (Supple), `protection` (Stone hide), `door_kick` (Roc's
    strength), `dig_percent` (Stonewise), `status_eases` and `no_penalty` (Regrowth's bleeding,
    Pure blood's poison), `disease_checks`, `speed_percent`.
  - **Damage-type talents** (#248; the design's §7.5 `#151` entries): Elemental ward (Guard;
    2 ranks, To 12 / Lv 20; chosen for an element, any kind that is not physical: 25 % a rank,
    counted as gear in `Actor::resistances`, so it also shrinks the doses of a status that is
    damage, Poisoned for a ward against poison, and the character screen says "Resistant to
    fire" in words) and Crusher (Might; St 13, Maces and clubs or Unarmed 3: `penetration` +2
    on melee blows, off-hand blows, shield bashes, and kicks that can do bludgeoning damage;
    never shots or throws) make the `damage_types` system's two, so it is held to the coverage
    bar. The witch's Blight (#245) is poison-typed when it lands: its burst names
    `damage_type = "poison"`, and the Poisoned it gives meets poison resistance as every dose
    does (`World::afflict`), so no rule of its own.
  - **Shadow and Wayfarer** (#239; the design's §7.5 but for Hooded light, which waits for the
    lantern's hood (#235), and Patient blade, which waits for the thief's openings (#225)):
    **effective eyes** (the design's B4, `WorldState::eyes`, §5 "Effective eyes"):
    `threshold_percent` (Night eyes: −50 %, eyes normal), `darkvision` (Dark adaptation: +2,
    eyes darkvision; Owl eyes: 2 cells for low-light eyes, level 9), `sees_uv` (Rune sight,
    Intelligence 14, level 15); `scent` (Blood scent: a bleeding creature in line of sight
    within 9 cells, four beyond darkvision's five, is seen however dark, eyes darkvision);
    `night_sight` (Moon-eyed: overworld sight one cell longer at night, eyes low-light);
    `lantern_light` (Lamplighter: the lit lantern burns as `lantern_bright`, a cell farther,
    in the light cache, so creatures see it too); `ambush` (Ambusher: +1d6 after Protection on
    a hit, any attack, at a creature that has not seen the attacker: it hunts no one it has seen,
    `Mind::last_seen` unset, and its own eyes do not see the attacker's cell; the die is drawn
    only then, `world/ambush.rs`); `terrain_as` (a step onto these terrains costs one onto
    another, `World::step_price`: Pathfinder, forest, dense forest, hills, and swamp as plains
    on the overworld; Swimmer, still, running, and shallow water as floor; Marsh walker, marsh
    and reeds as floor); `plunge_proof` (Swimmer: a plunge through ice does no harm and draws
    nothing); `light_on_ice` and `climb_percent` (Winter-born: ice holds the body's weight
    alone, climbing out onto it half price); `wading_proof` (Marsh walker: wading never exposes
    to a disease, no draw); `beast_friend` (Beast-friend: creatures of `animal` factions are
    neutral to the player until they attack one, `Progress::empathy_broken` by creature type,
    saved; read by `World::stance`, the AI's choice of a hostile, and look mode); `forecast`
    (Weather-wise: the weather coming within 3 hours, §5 "Weather"); `sun_healing`
    (Sun-blessed: healing half again as fast while the sun is up over an unroofed cell of the
    current level, whatever the clouds, `World::healing_rate`). Silent feet also counts toward
    the hearing system. The light, awareness, and factions systems stay `pending` until Hooded
    light and the class tables bring their second talent.
    The Defence by attack kind of the design's B5 landed with Deflect missiles (#237,
    `World::target_against`).
  - **Racial talents** (#241; the design's §7.5 Racial): the `racial` tree (`no_free`), each
    talent asking for its race. Elf: Elven accuracy (bows 3: threat +1 with bows, `weapon`),
    Ageless (level 9: hunger 20 % slower, and `disease_interval_percent` 100, a disease's
    progressions twice as far apart, `World::disease_turn`). Half-elf: Dual heritage (a skill
    roll more every level), Diplomat (Presence 11: favour +10 %, quest gold +25 %). Dwarf: Stone
    sense (`stone_find` +4 to finding what is hidden on a `stone` cell, searching and in
    passing; digging +25 %), Dwarven resilience (Toughness 14: Poisoned eases twice as fast,
    disease checks +2), Stout (level 12: `stout`, Strained costs what Burdened costs others,
    `Load::penalty`). Halfling: Lucky (`reroll_fumble`), Slinger (slings 2: sling damage +1, and
    `launcher_range` with `skill = "slings"`, +2 cells for a sling only), Small target
    (Dexterity 13: `small_target`, Dodge +1 against an attacker of a larger size,
    `World::small_target` after `target_against`). Gnome: Tinkerer (Intelligence 12: the Disarm
    ability, as the craftsperson's Tinker, whose id it cannot share), Gem lore (gem worth shown,
    gems found twice as often). Half-orc: Ferocity (`ferocity` +2 damage on every attack while
    below a third of the hit points, `Actor::stanced`), Orcish hide (Toughness 13: Protection
    +1). Humans are **granted Versatile** (`talents = ["versatile"]` in `data/races.toml`; a
    race may grant only granted talents of the racial tree, checked at load): `extra_pick` at
    level 1 (a creation pick more, `creation_count`) and at level 30 (`World::level_up`). A
    race's grants join the birth sign's (`stars::grant`, at the start and on every restore),
    shown as the race's ("(race)" in the dump). The talk and gems systems are held to the bar.
  - **Coverage** (the design's §8): every talent names the systems it touches; at load a
    system not `pending` with fewer than `min_talents` talents (or `min_class` class abilities,
    the talents the class tables give, counted apart from the talents; 0 until every class has
    its table), or a tree not `pending`, `no_free`, nor a class tree without `free_per_tree`
    prerequisite-free talents, is a content error. A wave that ships a system's talents removes
    its `pending`; a new system of the game joins the list in the PR that ships its talents.
    **Policy for new systems** (design §8.2): every new system of the game ships with talents,
    at least `min_talents` (and `min_class` class abilities), tagged with its new `[[system]]`
    in the same PR, and its feature issue says which in the template's "Talents and class
    abilities" section (§13), or why none (presentation only, or an extension of a system
    already listed). Resources are systems too: mana and the pools of #225 join the list with
    talents on their maximum, refill, and costs, and abilities that spend them carry their tag;
    a cost in a resource stands instead of a cooldown status, never beside one. The bar is
    data and may rise as the list grows. Developer notes: `docs/dev/progression.md` "Coverage
    policy".
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
- **The adventurers' class tables, and the class actives** (`content/class.rs`: `ClassActive`;
  `world/class_ability.rs`; M30, #243; design `docs/design/progression.md` §4.2–4.5): the
  warrior, barbarian, paladin, and monk have level tables as the everyday folk do (the
  "Class abilities" bullet above): `abilities` rows naming talents of the class tree
  (`data/talents/`, `granted`) and `bonus_talents`.
  - **Who has them**: given as the level comes, "You gain Rage." (the shared rules of the bullet
    above); each is read with the talents through `Actor::talent_mods`. Their effects are the
    talents' kinds and the class's own (the effects README of `data/talents/`); an active's
    effect (`rage`, `veterans_wind`, `whirlwind`, `lay_on_hands`, `smite`, `radiance`,
    `quivering_palm`) makes the talent that has it the ability's name and description in the
    menu, and its `requires_alignment` need is what the active asks of the paladin. Monk's
    Deflect missiles is the ordinary talent, handed out. Shared entries (`divine_health` is
    the paladin's, `purity_of_body` the monk's) are a talent each.
  - **Bonus picks**: warrior (Weaponcraft or Guard) at 4, 10, 16, 22, 28, 34, 46, 58;
    barbarian (Might or Vigour), paladin (Guard or Vigour), monk (Finesse or Vigour) at 10, 20,
    30, 45, 55; as the bullet above says.
  - **Actives** (`Ability::Class`, the abilities menu's Class group; `Action::Use`, `use:rage`,
    `use:smite:E`; refused, no time, without the ability, while its cooldown lasts, or while
    the player has strayed from the band it asks): Rage (Raging: St +2, To +2, In −2, no
    reading, 5 minutes, then Exhausted 10 minutes; no rage while either lasts; a wait's time),
    Veteran's wind (1 hit point a level back, at least 10, then Winded; a wait), Whirlwind (a
    blow at every hostile beside, clockwise from the north, each with the free blows the end of
    an attack brings, for one melee attack's price and one exertion), Lay on hands (2 hit
    points a level, then two degrees of Drained, 10 minutes, standing in for its mana until
    the mana PR; a cast; Good or better), Smite (a blow in a direction with 1d6 more per 10
    levels, at least 1d6, a magic weapon's extra die folded into the bonus at its average; then
    Spent, 5 minutes; Good or better), Holy radiance (on and off: Radiant, with no end of its
    own, sheds the faint item glow, reaching 3 cells, from the player, a light source like a
    glowing item's; a cast; Good or better), Quivering palm (a bare-handed blow in a direction,
    refused with a weapon in hand; a hit that does damage leaves three degrees of Bleeding;
    then two degrees of Spent, 10 minutes).
  - **Passives**, each one read: Weapon drill (melee weapon skill marks +25 %, `practise`),
    Shield line and Shield of faith (Shield +1; Shield of faith also +10 to Sword and Board's
    bash chance), Cleaving strike (the cleave free blow; with the Cleave talent too, great
    cleave), Armour drill (worn armour's Dexterity cap +1, Protection +1 while wearing any),
    Weapon mastery (threat +1 with melee weapon skills at rank 8 or more), Iron stance (the
    aggressive stance's Dodge penalty and the defensive's to-hit penalty gone), Champion
    (critical multiplier +1 with melee weapons, at most ×5); Thick hide (natural armour +2 in
    no body armour: nothing in the Body slot), Fast movement (steps 10 % cheaper Unburdened in
    no body armour, `World::step_price`), Brutal critical (critical multiplier +1 with a
    two-handed weapon), Hardened (each level past `hp_level` adds the level's hit points again
    and 1 more), Mighty thews, Blood frenzy (a kill while raging sets the rage's easing back a
    minute, never past its full length), Mighty rage (Raging's raises doubled, Exhausted eases
    twice as fast), Unstoppable (while raging bleeding and poison tick nothing, and a blow that
    would take the last hit point leaves it and ends the rage, so once a rage); Divine grace
    (disease checks +2, poison eases twice as fast), Divine health and Purity of body (no
    disease is ever caught: `World::expose` refuses, nothing rolled), Steadfast (Burdened costs
    nothing, as a dwarf), Bulwark (Protection +2), Avatar (Lay on hands heals fully, cures every
    status that ticks and every disease, one degree of Drained); Martial arts (bare hands and
    kicks roll 1d6), Fleet (steps 10 % cheaper in no body armour and with no shield), Deflect
    missiles (Dodge +2 against missiles), Flurry (a melee attack with nothing in hand strikes
    twice, the second at −2), Still mind (+2 on checks made with Willpower: diseases', noticing
    in passing; no status lowers Intelligence or Willpower), Diamond soul (Deflection +2),
    Perfect self (bare hands 1d8; unarmed threat +1, critical multiplier +1). The paladin's
    passives do not ask for Good; only the gifts used from the menu do.
  - **Resources to come**: Rage, Blood frenzy, Mighty rage, Unstoppable, Flurry, and Quivering
    palm ship as statuses and passives and move onto the alternative resources (§8 "Resources",
    #225: the pools are in; the moves are the design's R1–R5).
  - **Coverage**: `min_class` stays 0 until every class has its table.
  - **What the player sees**: the news of each level ("You gain Rage."); the actives in the
    abilities menu with their cooldowns, and the abilities in the dumps' `abilities` layer.
    Never a number the game hides: the wiki's class tables state the rules. **Saves change**
    (new status ids).
- **The ranger's, thief's, and bard's tables** (M30, #244; design §4.6–4.8, §4.1 B6): as the
  adventurers' above. Shared entries are the ordinary talents, handed out: Evasion (Finesse;
  the ranger's at 32, the thief's at 18) and Lore (Scholar; the bard's at 1). One the player
  picked before its level comes is not given again.
  - **The kill tally** (the design's B6, `Quests::kills`): every kill of the player's is
    tallied by level and creature type, whatever the type (kill quests read the same tally);
    `Quests::prey` is the type killed most, the first to get there of those tied (it changes
    only when another is killed more often). Saved, hashed, left out while empty.
  - **Bonus picks**: ranger (Weaponcraft, Wayfarer) and bard (Scholar, Delver) at 10, 20, 30,
    45, 55; thief (Finesse, Shadow, Delver) at 4, 10, 16, 22, 28, 34, 46, 58.
  - **Actives**: Volley (the ranger's, 50: `Action::UseAt`, `use:volley:X,Y`, aimed at a cell
    as a shot is; the readied missile weapon's three shots there, one after another, each its
    own flight and attack, for the price of one shot and one exertion; it stops when the
    ammunition runs out; then one degree of Spent, 5 minutes; refused as a shot is), Inspire
    (the bard's, 18: Inspired, to hit +1 and Dodge +1 for 5 minutes, for 3 mana, the time of a
    chat; no cooldown, mana is the price: greyed out "you have too little mana",
    `Unavailable::NoMana`, while the player has less). Throw your voice (the bard's, 32) is no
    longer everyone's: the ability is had by a talent with the `throw_voice` effect (the
    action is refused, "You do not know how to throw your voice.", without it).
  - **Passives**, each one read: Favoured prey (+1 to hit, +2 damage against `Quests::prey`, any
    attack, in `World::resolve`), Tracker (hears 3 fainter; every sound it hears placed at its
    own cell, `noise::Exact::All`), Rapid shot (shots 10 % quicker, the launcher skills'
    `speed`), Woodland stride (forest floor, bushes, reeds, tall grass, and marsh priced as
    grass; their noise stays), Swift tracker (overworld journeys −20 %), Camouflage (footsteps
    under the open sky, the level sky-exposed and the cell stepped to unroofed, −50 %, summed
    with the other footstep percents), Master hunter (bows and crossbows threat +1; Favoured
    prey +1 and +2 more); Sneak attack (Ambusher's rule: +1d6 after Protection on a hit at a
    creature that has not seen the thief; the unaware check is made before the strike, without
    a draw, only for an attacker with ambush dice), Trapfinding (searching +4; hidden traps 2
    easier to find, searching and in passing, not to disarm; the Disarm ability untrained), Pick
    lock (the Pick lock ability untrained), Uncanny dodge (the burden's Dexterity cap +2),
    Improved sneak attack (+2d6: ambush dice of the same sides add up, so 3d6 in all), Light
    fingers (the Steal ability untrained), Hide in plain sight (no footsteps while Unburdened;
    attacks half as loud), Master thief (an attack on an unaware creature threatens a critical
    2 lower, never below 3); Wide learning (a skill roll more every level), Song of rest
    (resting heals half again as fast), Jack of all trades (every trained skill counts 5
    higher, as a skill bonus in `Actor::skill_effect`), Honeyed words (favour +15 %, quest gold
    +25 %), Legend (quest XP +50 %, Presence +1), Virtuoso (Inspire gives Inspired greatly, to
    hit +2 and Dodge +2 for 10 minutes, for no mana).
  - **Resources to come**: Rapid shot and Volley (Focus), Improved sneak attack and Master
    thief (Openings), Inspire (Inspiration) move onto the pools of #225.
  - **Saves change**: `Quests::prey`, the tally of every type, the statuses `inspired` and
    `inspired_greatly`.
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

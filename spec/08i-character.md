# Warlock specification — 8. Architecture (continued): the character

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Character creation** (`warlock-core/src/character.rs`; M15): a run starts as a character, a
  race (`data/races.toml`), a gender (male, female, or other), and a class
  (`data/classes/`), chosen on New run (§6) or given on the command line (`--character
  dwarf,female,warrior`, §9); otherwise the scenario's character (§7). It decides:
  - **Class limits** (#1279): a class may be closed to races (`races_not`: no goblin, drow,
    kobold, lizardfolk, duergar, or ratfolk paladin; no elven or dwarven witch) or open to a
    race only with some genders (`only`: a drow priest is female), and may be `hidden` (the
    commoner, craftsperson, and merchant, for now). New run's class step lists only the classes
    open to the race and gender chosen, never a hidden one, so its Random draws only those;
    `--character` refuses a closed pair with the reason in words (`a drow may be a priest only
    if female`) but takes a hidden class, and a scenario's own character is never checked
    (the test world's craftspeople at the forge and the quarry).
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
    one makes way).
    The elements' last four (#268): Blaze (Fire's **active**, from the abilities menu's
    **Birth sign** group, `use:blaze`, the time of a `cast`, refused on the overworld: a burst
    of radius 1 around the player, one roll of 2d6 fire for every creature in it but them, whole
    to each, through the scrolls' cast code (Evasion and resistances apply, the kills are the
    player's); it melts the ice it reaches and leaves a burning missile's fire, `arrow_fire` for
    60 s, on every walkable cell of it but the player's; then the cooldown status Burnt out,
    twelve hours, during which the menu greys it "you are burnt out"), Kindled (a lit lantern burns
    as Lamplighter's brighter preset, a cell farther; with Lamplighter too, no farther; and fire
    resistance 25 %, #271), Forge-born (marks +50 % in every weapon skill and fighting style,
    added to Weapon drill's; in words "you learn weapons quickly"), Tidal (mana regeneration
    +25 %, added to the affixes' percent in `World::mana_rate`).
    The rest of the design's gifts land with their rules and are listed in `grants` only then.
  - **Background story** (#702, B1 of `docs/design/background-story.md`; `background.rs`,
    `content/background.rs`, `world/background.rs`): a chosen character (a run that starts with
    the kit: New run, `--character`, `--name`, `--talents`, a replay with `kit`) is told a youth
    in **chapters** of one line each, Parents, Birth, Childhood, Youth, The trial, Fortune,
    Leaving, and The road (the campaign's own; a campaign without one tells seven), shown as
    four paragraphs. Each chapter draws from its own stream
    `background/<race>,<gender>,<class>/<chapter>`, so no other roll moves: first the **tier**
    (per mille, `data/background.toml`: uncommon 120, rare 30, fateful 5, common the rest), then
    a line of that tier whose conditions hold (race, class, gender, birth animal and element, a
    lucky day, the starting band, talents and skills held, the tags earlier lines set, the age,
    the campaign), falling a tier at a time to the chapter's plain line; a fitting `only` line
    forces the chapter (Family wealth's and Father's sword's, which tell their gift and give
    nothing twice); then the line's dice. Its effects: stat points (the score and its potential
    together, a score never below 1), gold (given, or taken down to none), items as kit lines (a
    magic one identified, a scroll or potion known), skill starts, weapon-skill marks (never a
    rank), the starting alignment, and (#707, B5) a talent or a birth sign's flaw,
    each with what it hands out and kept in `Actor::boons` as the forgotten altar's Chosen and
    Forsaken are (`World::bestow`), so every restore puts it back; never a given, class, or flaw
    talent as a `talent`, and never one held (the roll skips a line that would give one held or
    given by an earlier line; a class table that names it later gives a talent pick in its place,
    #790); the
    Talents page and the `talents` dump mark it "(story)" or "(story, flaw)". **No caps and no protection** (the owner, 2026-10-29):
    nothing is counted across chapters; only a line's own worth is checked against its tier at
    load (rare and fateful lines never cost anything). It is told in `World::start_run` after the
    creation picks, so it reads every choice and meets no pick's prerequisite.
    `Character::story` keeps the ids (saves change; left out while empty, so older saves load
    with no story), and the text is told again from the content. A scenario's own character
    (`--scenario` alone) has none. The game tells it in words only, never a number or a stat's
    name; the summary says under the kit "Fate will have its say when you set out." No talent
    touches the story's odds (#787 cut Favoured child, and the `background` system with it).
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
    Gatekeeper (sometimes a free blow at a hostile stepping up beside them, the likelier the
    bolder their stance: 1/4 neutral, §8 "Free blows").
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
    change); the wiki lists them all. Another head (Hydra, 9: hit points come back while Starving, resting heals +100 %), Relentless
    fury (Manticore, 9: the streak runs to +5, and every two blows before it +1 damage), Horn of
    plenty (Unicorn, 9: Horn's grace also gives back a hit point per level, never past the
    maximum, so a wound alone is cause enough, and leaves one degree of Graceless, eight hours),
    **Fire breath** (Wyvern, 15: an active, `use:fire_breath:X,Y`, aimed at a cell within 8, the
    time of a `cast`, refused on the overworld: a bolt along the line toward it and on, 4 cells
    or to the first wall, one roll of 2d6 fire for everyone on it through the scrolls' cast code,
    the `fire_breath` effect; then two degrees of Drained, ten minutes),
    Petrifying gaze (Basilisk, 12: the stare leaves 4 degrees of Transfixed and one of
    Spent-eyed, an hour), Dragon's hoard (Griffin, 12: Hoard sense's odds half again, loot +25 % and gems
    +50 %, shop favour +10 %), Second rising (Phoenix, 18), Tentacles (Kraken, Twin Blades 3: off-hand blows
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
    going to another level too; no roll; like every hold it also ends after 3d4 turns from the
    last hit, or as the held creature is knocked away or the holder is held or slowed, §8 "Status
    effects"), and the Gate Warden's Warden of the gate (a
    foe stepping away from beside them takes a free blow: Opportunist, granted; Protection +1;
    what status effects take off their scores halved, rounded toward 0). About one character
    in 1 800 over an even spread of births; by race at a start date, as the wiki tabulates. The
    game never says lucky days exist until a character is born on one. The birth draws are
    unchanged, so existing seeds keep their dates and signs.
  - **Calendar and moon flavour** (#273; design §7; `[flavour]` in `data/horoscope.toml`, and
    none of it without the table; `world/flavour.rs`): pure functions of the game time, the
    birth date, and the moons, drawing nothing. **The birthday** (the birth month and day):
    from midnight to midnight the player heals `birthday_healing` percent as fast (125), the
    last factor of `World::healing_rate_at`; both midnights are moments of `World::live`, so
    the healing is exact however time is split. At that day's dawn, when a perception's
    interval passes it, a note (it never stops a rest), "It is your birthday.", and the same
    line when a run begins on the birthday. **The birth month**: a sign active's cooldown
    status (Graceless, Spent-eyed, Burnt out, Drained after Fire breath, Hoarse; Ashen is not a
    cooldown and is untouched) lasts `birth_month_cooldown` percent as long (50) when it is
    used in the birth month: `World::gift_cooldown`, the one place a sign active applies its
    cooldown, leaves fewer degrees, the last partly worn (three degrees of eight hours become
    two, the second's first four hours worn). **The full moon**: while a moon is within a
    sixtieth of its cycle of full (`MoonDef::phase_at`; half a day either side for the built-in
    30-day moon) and above the horizon with the sun down, wherever the player is (under clouds
    or underground too), the gifts with a reach reach `full_moon_reach` cells farther (2;
    `Content::full_moon_reach`, the one read): Hound's nose's scent (6 to 8), the Transfixing
    stare (6 to 8; `Ability::reach_at`, which the targeting mode reads too), and Eyes of the
    heights at night (+1 to +3); only the gifts' (`TalentMods::gift_reach` marks the `scent`
    and `night_sight` of `stars` talents: Blood scent is not lengthened). Inner
    light burns as `full_moon_light` (`ember_full`, radius 2) instead of its `ember`. Nothing
    new is shown: the status bar shows the moon's phase already, the cooldown its status, and
    the wiki the numbers. The design's element seasons stay deferred.
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
    one does not, nor their own lit lantern alone (about 220); a lantern beside a brazier does. A note says when it begins ("The bright light dazzles
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
    it still decides the band near an edge: a Unicorn-born elf starts Kind (150). The paladin's
    550 is set so that no animal starts a paladin of a race that is not evil below the Good their
    gifts need (#317): a Manticore- or Basilisk-born half-orc paladin starts at 400, Good's edge.
    It draws nothing.
  - **Mana**: the race's and the class's `mana` add to the mental stats' modifiers for the
    maximum (§8 "Mana").
  - **Weapon skills** ("Weapon skills" below): the race's starting ranks plus the class's, plus
    one set of the class's `weapon_skill_choice` if it has one, at most 10 each. Dice among
    them roll from their own stream,
    `character/weapon-skills/<race>,<gender>,<class>`: the race's (in skill order), the
    class's, the pick of the choice (one draw), then the chosen set's; so they never move a
    stat roll.
  - **Fighting styles** ("Fighting styles" below): the class's `styles` plus one set of its
    `style_choice` if it has one, from yet another stream,
    `character/styles/<race>,<gender>,<class>`: the class's (in skill order), the pick of the
    choice, then the chosen set's; so they never move a stat or a weapon skill roll.
  - **Starting kit** (`warlock-core/src/kit.rs`, data `content/kit.rs`; #257): what the character
    carries, wields, and wears at the start. The class's `kit` (`data/classes/`) comes first,
    then the race's few racial items (`data/races.toml`: every race without darkvision, the elf,
    half-elf, human, halfling, gnome, lizardfolk, and ratfolk, has a lantern ready in the Tool
    slot, `{ wield = "lantern" }`, §5 "Light sources"), then the birth sign's animal's and
    element's (`data/horoscope.toml`: the hook for the horoscope; none give anything yet). A kit
    is a list of lines, each one item or a few of one, as level files place objects: a string is
    carried, a table says `carry`, `wield` (wielded or readied: hands, missile weapon, missiles,
    Tool slot), or `wear`, and may add `trained` (only if the class trained that weapon skill or
    fighting style: its fixed ranks, or the set of its choice that was picked; never the race's
    ranks), `gender` (only for that gender: the hook for gender's items), and
    `magic` (a base item made as loot makes it at item level 1, its rarity rolled at least
    `any`, `magic`, or `rare`) and `fill` (a `wield` or `wear` line given only into a slot still
    free once everything else is given, else left out; #1173); `"a|b|c"` is one of them. Each item goes into the pack in order
    (a stackable one joins a stack of the same), and a `wield` or `wear` item into the first free
    slot it fits (a second light weapon to the off hand); one that finds none is carried. Kits
    roll from a stream of their own, `character/kit/<race>,<gender>,<class>`, in that order: a
    line draws only if it applies, one draw to pick among alternatives, then for a `magic` line
    the loot draws (rarity, affixes, a rare name, an ornament); so no stat, skill, weapon skill,
    or style roll moves. Last come the class's random spell scrolls (#1104; `random_scrolls =
    { count = "1d3", below_level = 10 }`): the casters set out with a few more, the wizard and
    the witch 1d3, the priest, the druid, and the ranger 1d2, the bard 1d4, every other class
    none; each the scroll of a spell of spell level 0 to 9, drawn uniformly among those spells
    (duplicates stack), from the same stream after every line but the `fill` ones, which come
    last, the kits in the same order, each drawing its alternative and given only if a slot it
    fits is free. A race's `bare` slots (`bare = ["feet"]`: halflings go barefoot) take nothing
    of any kit: an item for one is left out after its draw. **Everyone is dressed** (#1173): every
    kit wears something on the body and, but for a bare race, on the feet, if only clothing
    (item class `clothing`: Dv 0 Pv 0, never loot or magic; tunics, kirtles, smocks, robes,
    habits, doublets, gowns, and the like, and shoes, turnshoes, clogs, sandals, slippers, soft,
    felt, and hobnailed boots, footwraps): a class's own garb or armour first, else the race's
    everyday clothes and shoes (its `fill` lines). Kit items are what they are: magic ones
    unidentified and food made when the run starts (§8 "Hunger"); and **everything a run starts
    with is known by type**, a rule of the run's start rather than of a kit: every scroll,
    potion, wand, and rod of the starting pack, the kit's and a scenario's own pack alike (§8
    "Scrolls", "Potions"). Data is checked when content loads (every item
    exists, what is wielded or worn fits a slot of that kind, `magic` only on base items, a
    class's `trained` only what it trains). Unfair on purpose: the aristocrat sets out rich, with
    a perhaps magic rapier and an heirloom ring (magic or rare), the merchant with goods and gems,
    the commoner with a stick, two turnips, and bread. The martial classes set out armoured
    (#695: the warrior in a hardened leather jerkin and an iron helm, the paladin the same with
    gauntlets), every elven race with a bow and its ammunition of the weapon skill its race gives
    a rank in (the elf a hardwood long bow, the half-elf a hardwood short bow, the drow a
    crossbow and steel bolts), the halfling with a sling and stones, and the dwarf in steel (pick,
    gauntlets, shod boots). Since #1173 the dwarf also carries a steel hand axe and, where the
    class gave none, wears a steel helm, a hardened leather jerkin, and readies a crossbow and
    steel bolts; the duergar the same in iron, with a war hammer and no crossbow; the elf and the
    drow carry a potion of healing, the elf wears a silk tunic, soft boots, and a travelling
    cloak where free, the drow dark silk, felt boots, and a silk cloak. An ordinary small or
    frail (a Toughness penalty: elves, drow) warrior or paladin may start Burdened under armour
    and shoes. Nobody starts Strained, not even the weakest character of a
    combination (every die at its least and the worst sign); an ordinary one (every die at its
    mean, no sign) starts Unburdened, but for the aristocrat's and the merchant's wealth and a
    small race's warrior's or paladin's armour (tests check both). **Who starts with it**: a run whose character
    the player chose, that is one started from New run's character creation, or by the game or
    `warlock-cli` with `--character` or `--name`, or a replay that records the kit; the
    scenario's `carry` comes on top (§7 "Scenarios"). A run on the scenario's own character that
    nobody chose (`--scenario` alone, tests, golden walks, replays from before kits) starts with
    the scenario's pack alone, as before kits, with what the scenario readies (its `wield`, the
    lantern) first. The world state and replays record which
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
    feel your righteousness waver."); smaller moves are silent.
  - **What reads it**: good creatures **scorn** an Evil or Wicked character, evil creatures a
    Good or Saintly one (`Band::scorned_by`): they answer a chat with a cold line
    (`greet_evil`, `greet_good` in `data/dialogue.toml`) instead of their greeting and
    one-liners, but offer their quests all the same (#692: a good giver's quests to an Evil
    character too). A quest may name the bands it is offered to (`requires_alignment`). A good
    shopkeeper's prices move with the band, by a percent in data added to the player's favour
    (§7 "Shops": Saintly 10 % better, Evil 10 % worse, Wicked 20 % worse); the prices shown
    include it. **Alignment bars nothing** (#642): no class, talent, or ability asks for a
    band, nothing is greyed out or refused by it, and nothing remembers a drop in band. A
    **holy gift** (Lay on hands, the smites, Holy radiance, Consecrate) may name the bands it works
    for (`alignment = { min = "kind" }` on its talent; Lay on hands, the smites, and Holy radiance
    are Kind or better, the good side, and Consecrate Neutral or better): outside them it is still listed and used, and does nothing ("You call for holy
    wrath, but none answers."), taking the time of a cast and spending no cooldown, mana, or
    resource; putting Holy radiance out always works. **A sin empties zeal** (#910): a deed
    that moves the player's score down empties the pool `data/smite.toml`'s `sin_empties` names
    (the paladin's zeal), if held, "Your zeal deserts you.", with the deed's own lines; it bars
    nothing, the pool fills again. Mana reads nothing of it, for now. Gods
    and prayer, holy and unholy damage and items, and artifacts that work only for some bands
    (as the holy gifts do) will read the same band.
  - **Talents** (proposed for the talents PR, on hooks that exist): *Righteous fury* (while Good or
    better: +1 to hit and damage against evil creatures, per 10 levels; `Profile` in
    `attack_with`, reading the target's `Actor::nature`), *Hardened heart* (while Callous or worse:
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
  - **Concentration** (#668): a trained Concentration makes mana come back faster and the pool a
    little deeper (§8 "Mana": the curves in `data/skills.toml`, the pool's at most a fifth);
    untrained, it gives nothing, even raised by an item or a talent. The Skills screen says so
    in its description, in words. The priest, druid, wizard, ascetic, paladin, and witch start
    trained in it; every class has some mana, so any trained in it gains.
  - **From race and class** (§7): their `skills` are dice rolled when the character is made,
    after every other roll ("Character creation" above); a skill both train sums the two; the
    race's `class_skill` (humans, `2d4+4`) goes to one of the class's skills picked uniformly
    from the same stream. The class's `optional_skills` (#662) each come with their chance, then
    their dice, from their own stream (`character/optional-skills/<race>,<gender>,<class>`), so
    no other roll moves; once, never re-rolled, and the chances are never shown (the wiki has
    them). They are not the class's skills for a level-up roll's +1, and the human's extra class
    skill never lands on one. A race may have `optional_skills` too (#682, the same table and
    parser, `content::OptionalSkill`): rolled after the class's from the same stream, so no
    class's draws move; the drow's Sneak attack (40 %, `2d4`), summed with a thief's own. A creature type's skills are fixed values. A scenario without a
    character gives a player trained in nothing.
  - **Values** run 0 to 999 (`MAX_VALUE`), 100 a master; there is no hard cap, but points come
    slowly past the soft cap (100). A value grows **only by skill rolls**: a level's and the
    automatic ones (below).
  - **Training marks**: a meaningful use of a trained skill may earn it a mark, with its rule's
    chance (§7), one draw of the run's **training stream** (`training`, kept in the world state;
    never the gameplay stream, so training never changes a fight's rolls) per rule: a step of the
    player's that some creature on the level would hear at its threshold but for their stealth
    (`sneak`, through the same propagation as hearing; the overworld is silent), a perceived
    sound that is news (`hear`, one draw per turn with news), an attack on the player that missed
    because of their Dexterity or Dodge (`dodge`, the dodge messages), an action other than
    waiting (not a rest's turns) taken while the player's burden slows them, as it began
    (`burdened`; a dwarf's Burdened does not), a step on the overworld (`travel`), a kick at a
    closed door (`kick`; Athletics is also marked by `burdened` and `travel`, 5 % each, #548, so
    its speed grows with plain play), a check against bonds, whatever came of it (`escape`, Escape
    artist's; §8 "Status effects"), a climb out of a pit (`climb`, Escape artist's and Athletics';
    §8 "Traps"), a turn spent searching, whatever it finds (`search`), noticing
    something hidden in passing, not a trap (`spot`; §8 "Searching"), finding a hidden trap,
    searching or in passing (`find_trap`, Detect traps' use), an attempt to steal, whatever comes of
    it (`steal`; "Stealing" below), a try at treating oneself, whatever comes of it (`heal`;
    "Treating oneself" below), a forage that ran its time, whatever it found (`forage`;
    "Foraging" below), a deal made in a shop: goods paid for or the player's own sold (`deal`;
    Negotiation), something carried told for what it is (`appraise`; "Appraise" below), mana
    spent, a spell or an ability paid in it (`cast`; Concentration, 25 %, #668), discovery
    experience gained (`discover`: a first kill of a kind, an arrival that pays; Experience §8), a
    check rolled to resist a creature's ability, whatever comes of it (`resist`; "Knowledge"
    below), a step wading (`wade`; Swim, below), a Swim check, whatever comes of it (`swim`), a step in deep water (`stroke`), a step onto rough ground (`rough_step`; Balance), an action
    taken Staggered (`staggered`; Balance). Only
    the player trains for now; creatures keep their type's
    skills. Marks are a **reservoir**: they raise nothing by
    themselves and nothing is said; they gather until a roll on the skill spends them.
  - **Automatic rolls** (#1328): when a trained skill's marks reach `auto_at` (`[rolls]` in
    `data/skills.toml`: 100, the ladder's `3d4` step) the skill rolls at once, with no choice:
    the same roll as a level-up roll (below: its die, the class and race affinities, the soft
    cap, the `spend` of the marks, the levels stream, "Your stealth improves."), so it falls back
    below the threshold. It is checked wherever marks are added (`World::mark_skill`), for every
    skill and way of training, and comes on top of the level's rolls, whatever level-up is due.
  - **Balance** (#1029): sure footing under a load, on bad ground, and after a stagger. Besides
    the burden's slowdown (`burden`), a step onto rough ground (off the overworld, `move_cost` over
    100 and not wading: bush, reeds, rubble, ice) spares a percent of what the terrain adds over
    a plain step (`footing`: 60 % at 100, all at 200; rounded up), and a Staggered actor's
    extra cost (`cost = 150`) is eased by a percent (`steady_ease`: half of the extra at 100, three
    quarters at 200, as Initiative's `surprise_ease` is for Surprised). Marks: `burdened` 5 %,
    `rough_step` 10 % per step onto such ground, `staggered` 20 % per action taken Staggered.
  - **Swim** (#1018): wading is quicker. A step onto wading terrain (`stars::wading`: water that
    freezes, ground a disease is caught wading in; `move_cost` above 100, off the overworld) has
    a percent of what it adds to a plain step spared: 60 at 100, all of it at 200
    (`data/skills.toml`, `wade`), under the talents that spare all of it already (Swimmer,
    Strong swimmer, Undertow). A player trained in it who plunges through the ice (§5 "Ice")
    makes a **Swim check**: Toughness's modifier, the talents' Toughness checks, and the `swim`
    benefit (+8 at 100, +12 at 200, at most 20) against 11; made, half the plunge's hit points,
    rounded down. Marks: `wade` 10 %, reported per wading step whoever wades; `swim` 100 %.
    **Deep water** (#1019; terrain `deep_water`, `swim = true`, `walkable = false`, so nothing else
    goes in: creatures path around it and wait on the nearest shore, shooting and casting from
    there, allies too, and a swimmer swaps with no one). A step into it from anything else is
    refused with no time for the untrained ("You cannot swim.") and for one Strained or worse
    ("You would sink."), and otherwise **always asks first**, in the one dialog, with no setting to
    turn it off (the owner; `input::deep_water_dialog`; "Dive in", "Stay back" selected first and
    Esc's, no time; core `move:DIR` needs no dialog). Water the player does not know of stops them
    at its edge with no time, and is then known, so the question comes before they are in. Going
    in takes the step and no check, and puts out the ready light. **A step that starts in it**,
    wherever it ends, is a Swim check against 11 + 2 per burden level above Unburdened
    (`data/swim.toml`): made, the step, priced by `step_price` (Wade, Swimmer, Strong swimmer,
    Undertow); failed, the turn is spent, no move, and 1 cold hit point through `hurt` with
    `Harm::Spared` (never the last: no drowning; one who grows Strained in it swims on, harder).
    Two gameplay draws a step; `stroke` 25 %. Afloat, dropping and picking up ("It would sink."),
    resting, searching, digging, and lighting a light ("The water would only put it out.") are
    refused with no time; fighting, firing, casting, reading, and waiting go on; a wait asks no
    check; nothing carried is lost. A light made ready in it goes out too. It does not freeze.
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
    stream. Said: "Your stealth improves." (never the number rolled). The level-up screen shows
    each skill's dice before the choice, with the marks and what set them, and where a level's
    rolls come from (the owner, 2026-10-27, #543; §6 "Level-up"); one function computes each for
    the rule and the screen (`world::skill_roll`, `world::roll_count`). The dev `levelup:auto`
    spends every roll on the first trained skill.
  - **Knowledge** (#667, the owner 2026-10-28: know your enemy): the player's check to resist a
    creature's ability (a drain, §8 "Stats"; exposure to a disease its attack carries, §8
    "Diseases"; struggling free of its hold, §8 "Status effects") adds what the `resist` benefit
    gives, read at how far the skill's value stands **above the creature's level** (its type's),
    not at the value: nothing at or below it, `curve = [[0, 0], [5, 2], [40, 6]]` (+2 at 5 above,
    +6 from 40 above), at most 6 (`MAX_RESIST_BONUS`), magic items' and talents' skill bonuses
    counted. Creatures resisting each other get nothing; a check no creature asked for (a scroll's
    backlash, rotten food, wading) neither. It is marked by `discover` (100 %) and `resist` (25 %).
    Nothing is said when it helps; the Skills screen's description says it in words; the wiki
    has the curve. One hook, `World::resisting` (`world/knowledge.rs`). Look mode (§6) says what
    a seen creature's kind resists, is immune to, and is vulnerable to, in the words of the
    character screen's resistances and never a number, once the skill's value stands above its
    level at all, with the same bonuses counted (#1030; `PerceptionView::known_resistances`): the
    kind's own, not what this one wears or has on it. Knowing is not practising: no marks.
  - **Sneak attack** and **Initiative** (#682; §8 "Surprise and sneak attacks"): Sneak attack
    (`kind = "rules"`, after Search) gives `sneak_dice` (`steps = [1, 25, 50, 75, 100], every =
    50`: 1d6 trained at all, 5d6 at 100, 7d6 at 200; read by `World::sneak_dice`) and `surprise`
    (+8 at 100, +12 at 200 on the difficulty of the initiative check of those its holder catches
    unawares; `World::startle`), marked by `sneak_attack` (50 %; a Dexterity mark too, 20 %).
    Initiative (`kind = "passive"`, after Heal) gives `initiative` (+8/+12 on the holder's own
    check) and `surprise_ease` (50 % of Surprised's extra cost at 100, 75 % at 200, summed with
    Cool head's, at most all; `World::status_cost`), marked by `surprise` (100 %: a check made,
    whatever came of it). The thief trains Sneak attack (`2d6+10`, known for) and Initiative
    (`2d4+4`), and Detect traps and Disable device at `2d6+10`, known for, and Search at
    `2d6+4` (#684: in place of the Trapfinding class ability); the ranger and the ascetic Initiative `2d6+10`; the bard, warrior, and berserker
    `2d4+4`; a drow may start with Sneak attack (an optional skill of the race).
  - **Intimidate** (#844; `kind = "action"`): gives `intimidate` (+8 at 100, +12 at 200, at most
    20) on the difficulty of the Willpower check the berserker's War cry asks of those who hear
    and see it ("Class abilities" below) and of the skill's own threat, marked by `intimidate`
    (100 %: a cry roared or a threat made, whatever came of it). The berserker trains it `2d4+4`,
    the half-orc `1d6+4`. Its use (#868) is the **Intimidate** ability (the abilities menu's
    Skills group, in the skill's place, while `data/intimidate.toml` exists; no key): aimed at a
    cell the player sees within the file's `range` (6), `intimidate:X,Y`, at a hostile creature
    with vitals there. It takes the `intimidate` action (a turn), is spoken aloud (a voice where
    the player stands, as loud as a mutter), and marks Intimidate, whatever comes of it. If the
    foe hears the voice (asked before it is made) and sees the player now, it rolls War cry's
    check (`world/war_cry.rs`, shared: two gameplay draws, its Willpower modifier against the
    file's `difficulty` 10 + the player's Strength modifier + their Intimidate); failed, it is
    Frightened, caused by the player, at the degrees the file's ladder gives for the points
    Intimidate adds (`degrees = [[0, 1], [3, 2], [6, 3]]`: one degree below 3 points, value 38;
    two from there; War cry's three from 6 points, value 75). A foe that does not hear or see
    the player pays no heed (no draw). Refused, with no time and no draw: untrained, at the
    player's own cell, out of range, at no creature seen, at one not hostile, or at one without
    vitals.
  - **Bluff** (#1016; `kind = "action"`): gives `feint` (+8 at 100, +12 at 200, at most 20) on the
    difficulty of a feint's Willpower check, marked by `feint` (50 %: a feint made, whatever came
    of it). Its use is the **Feint** ability (the abilities menu's Skills group, in the skill's
    place, while `data/feint.toml` exists; no key): in a direction, `feint:DIR`, at a hostile
    creature the player sees there, with a mind to fool. It takes the `feint` action (a turn). The
    foe rolls 2d12 + its Willpower modifier (+ its talents' Willpower checks; two gameplay draws)
    against the file's `difficulty` 10 + the player's Presence modifier + what Bluff adds. Failed,
    it is Surprised, caused by the player ("The kobold falls for it."): its first action in that
    turn does not ease it, so the player's next blow finds it flat-footed (no Dexterity or Dodge
    in its Defence, Sneak attack's dice); its action after that ends it. Made, the turn is spent
    ("The kobold is not fooled."); try again. Refused, with no time and no draw: untrained, at no
    creature seen, at one not hostile, or at one without vitals (`Content::has_vitals`). There
    is no Bluff check when caught: a caught thief stays caught (the owner). Its other use is
    **talking past** a party met face to face on the overworld (#1017; §7 "Overworld
    encounters"), marked by `talk_past` (100 %: every try).
  - **Active uses**: skills get no keys of their own. Racial, class, skill, and style abilities
    are chosen from one abilities menu (§6 "Abilities", #156), unlike ADOM: each trained skill
    with an active use (`kind = "action"`) is an ability there (`warlock_core::ability`,
    `PerceptionView::abilities`), greyed out as "not usable yet": the uses come with their
    actions, and one whose use has come is that ability instead (Disable device's is "Disarm a
    trap", §8 "Disarming traps"; Open lock's "Pick lock", §8 "Doors"; Steal's "Steal",
    "Stealing" below; Bluff's "Feint", above; Heal's "Heal", "Treating oneself" below); a passive skill with a use is listed as it too (Survival's "Forage",
    "Foraging" below). Never a benefit's number,
    a mark, a
    chance, an untrained skill, or a creature's skills; ground-truth dev dumps (`actors`, the
    world state) show every actor's skills with their marks and the ladder's die for them. The
    Skills screen's meter shows a skill's ladder step in quarters (how practised it is). Skills
    and marks are part of the world state (saved, hashed).
  - **Appraise and Negotiation** (#666, the owner 2026-10-28): the shops' prices are
    Negotiation's (`bargain`, 20 % at 100, 30 % at 200; §7 "Shops"), marked by deals (`deal`,
    25 %). Appraise (passive) tells things: on every whole game hour the clock passes
    (`world/appraise.rs`, called from the status loop's `live`, so a rest rolls each hour it
    passes, and from the dev `skip`), a player trained in a skill with the `identify` benefit
    who carries or wears something they do not fully know (what the scroll of identify and an
    appraisal would tell: unidentified magic, a scroll of an unknown type, a wand's unknown
    charges) has the benefit's chance in percent (Appraise: 2 % from value 0, 12 % at 100, 20 %
    at 200; with what magic items and talents add to the skill) of telling one of them, picked
    uniformly: "You realise your silver ring (magic) is a silver ring of the Mule." The draws
    come from the stream `appraise/<hour>` (the hour counted from the calendar's start), only
    when there is a check to make, and no state is kept, so a run without Appraise or without
    anything unknown hashes as before. Untrained, never (a magic item's bonus alone does not
    give it). Each thing told may mark Appraise (`appraise`, 100 %). The classes trained in
    Appraise (bard, thief, craftsperson, merchant) have Negotiation at the same dice; the
    merchant is known for both. Gnomes keep Appraise alone. Appraise and Survival also tell food's freshness
    (#1031, `freshness` benefit; §8 "Hunger"): in words (`fresh`, `old`, `rotten`) in the eat list and look mode, when
    a per-hour check passes, summed percent chance (Appraise 25 / 60 / 80 % at 0 / 100 / 200, Survival 15 / 35 / 50 %,
    rotten food 25 points easier, at most 95 %); eating such food marks Appraise (`judged`, 10 %). A healer's and a
    sage's service price follows the same favour as a shop's (a `temple` service stays flat), and a service paid
    marks Negotiation (`deal`).
  - **The professions' skills** (#1331, #1343; `docs/design/professions.md` §9): **Alchemy**
    (`identify_potion`) and **Enchanting** (`identify_magic`) look the same way each hour, after
    Appraise and from the same stream, each at its own things (5 / 25 / 40 % at 0 / 100 / 200):
    Alchemy at potions and flasks of an unknown kind ("You recognise your fizzy scarlet potion:
    a flask of fire."), Enchanting at unknown magic that is no potion or scroll (arms, armour,
    jewellery, wands, rods: "You read the magic bound in your long sword: …"); Enchanting does
    not mark until its craft comes, Alchemy marks by brewing (§8 "Professions"). **Mining** adds its `dig` percent to a dig turn's work (25 at 100,
    40 at 200; §8 "Digging"), marked by every wall dug through (`dug`). **Herbalism** gives a
    forage that found something a chance of a herb besides ("Foraging" below), and more herbs
    to a gather from the glowing fungus ("Gathering herbs" below). **Jewelcraft**
    cuts gems (its profession's Cut, "Professions" below). Who trains them: gnomes Jewelcraft, dwarves and duergar Mining (dwarves
    Jewelcraft now and then), kobolds Mining, halflings Herbalism; druids and witches Herbalism,
    witches Alchemy; optional for the craftsperson (Jewelcraft, Mining), merchant (Jewelcraft),
    commoner (Mining), ranger (Herbalism), wizard (Alchemy, Enchanting), druid (Alchemy), witch
    (Enchanting), and aristocrat (Enchanting); and a story line each.
- **Treating oneself** (`heal.rs`, `world/heal.rs`; #549, the owner 2026-10-27): the Heal
  skill's use, the abilities menu's **Heal** (`Ability::Heal`, in the skill's place among the
  Skills; `Action::Heal`, replay text `heal`, `heal:ID`, or `heal:ID:DIR`; cost `heal`, 2500: a
  quarter of a minute; no key of its own). Anyone trained in a skill with the `heal` effect has
  it; it is greyed out "nothing to treat" while there is nothing they may treat. The patient is
  the player, or **an ally of theirs beside them** (#915; design ranger-companion.md §9,
  "Tending": from level 1, for anyone with the skill, whatever the ally's source): its ailments
  are offered after the player's own, in direction order, as "Your wolf's bleeding" (the way
  after, "(east)", when two allies beside them go by one name), and `heal:ID:DIR` treats the
  ally that way; the healer's check, the patient's ailment and degrees, the same time and marks.
  The player knows their own allies' hurts, poison and sickness too; "You have no ally there."
  refuses a direction with none, with no time. What may be treated (`heal::treatable`, in status order, then disease
  order): a status with a `treat` table (`data/status.toml`: bleeding, poisoned), and a disease
  that shows (one still incubating is unknown to its sufferer). With more than one, the menu
  asks which in the dialog (§6), each by the name the player knows it by ("Sickness" for a
  disease not yet known by name); a bare `heal` treats the player's first. A try is a check (§8 "Checks"):
  2d12 plus the Intelligence modifier (effective score, and talents' Intelligence check bonus)
  plus Heal's `heal` benefit (+8 at 100, +12 at 200, at most 20) against the status's
  `treat.difficulty + treat.per_degree × severity` (bleeding 6 + 2 a degree, poisoned 8 + 1 a
  degree) or the disease's own `difficulty`. Every try takes its time, may earn Heal a mark
  (`heal`), and makes no noise. A status that `stops` (bleeding) ends when the check is made
  ("You bind the wound." then its end line); failed ("The blood still runs.") it may be tried
  again at once, any number of times, as may every try below. Any other status (poison) eases a degree when it is made
  (ending at none); a disease made is eased: it counts as a progression check won toward its
  `recovery`, unless that would shake it off or it never recovers by itself, when its next
  progression is put off to a full interval from the try instead, so a try never cures. A poison
  or a disease may be tried again and again, made or failed ("The poison will not come out."
  / "You can do no more for the filth fever."): time is the price. Refusals take no time and no
  draw: untrained ("You know nothing of healing."), or nothing to treat.
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
- **Fletching** (`world/fletch.rs`, `world/craft_make.rs`, data `data/fletch.toml` and the
  recipes `recipes/woodcraft/arrows.toml`, `bolts.toml`; #1021, #1336; the owner: "Do not require
  small blade in hand, just in inventory is fine"): **Arrows** and **Crossbow bolts**, Woodcraft
  recipes open to anyone on Woodcraft's profession screen, made as any generic recipe is
  (`Action::Craft`, `craft:arrows:HEAD,SHAFT`; the forging wizard asks for the head, then the
  shaft). There is no `fletch` action since #1336.
  - **Needs**: a knife carried, in the pack, worn or wielded or neither (`Content::is_knife`: an
    object of the `daggers` weapon skill: a dagger, a stiletto, a knife; a short blade is none);
    a launcher ready that shoots the result (a bow for arrows, a crossbow for bolts); a billet of
    wood for the shaft, except on ground whose landscape `data/recipes.toml`'s `free_wood` lists
    (forest, dense forest, swamp: the shaft is cut there, no step and no billet). Its row is greyed
    "no knife", "no bow ready", or "no wood to hand", and the rules refuse ("You have no knife.",
    "You have no bow ready."), taking no time.
  - **The head**: an arrowhead (one an arrow; the arrow takes its metal), a bone shard or a stone
    block of a material the arrow may be made of (bone, horn, chitin, ivory, dragonbone, flint,
    obsidian; one heads the whole batch), or None: `brittle_arrow` or `brittle_bolt`
    (`breaks = 100`, of the shaft's wood). Plain arrows need heads whatever the skill. The result's
    parts are the head's material and the shaft's wood: their own goods; a flint head strikes as
    flint does.
  - **Time**: the recipe's `seconds` (600: ten minutes), in `craft` turns, stopped like a rest
    ("you put down your work."): the time is spent and nothing else. On the overworld the
    encounter roll runs as for a forage.
  - **The check**, at the end: 2d12 + Dexterity's modifier + talents + Woodcraft's `craft` points
    (curve `[[0, 0], [100, 8], [200, 12]]`) against `difficulty` (12) plus the head's `work`. The
    count is `1d4` + Woodcraft's value ÷ 25, at most what the billets and arrowheads chosen cover
    (a billet covers up to 11 arrows, 4 bolts). Margin 0 or more makes the whole count; −1 to −3
    half, rounded up; −4 to −6 one; below −6 none (a natural 2 none, a natural 24 all). Gameplay
    stream. Woodcraft earns a mark on every batch that ran its time (`Use::Craft`). The stock is
    spent for what was made; none made, half of it comes back (a single piece whole).
  - **Fletcher** (`talents/fletcher.toml`, effect `fletcher`, `TalentMods::fletcher`): the ranger's at
    level 1, nobody else's; half the time, +4 to the check, twice the count after the margin. The
    ranger's kit has a knife, carried, not wielded.
  - **The other batches** (`docs/dev/crafting.md` "Counted recipes"): Metalcraft's `arrowheads`
    (20 from an ingot of any metal at an anvil, of that metal), Woodcraft's `sling_stones` (10 from
    a block of stone, no knife, anyone) and `flaming_arrows` (10, an arrow each and a flask of oil).
- **Whittling** (`world/temper.rs`, `Craft::Whittle`; #1028; design `skill-uses.md` W3): Temper's twin for wood on the same rules, so one item has one state (`magic/worked.rs`) and one set of numbers (`data/temper.toml`: ten minutes, `13 + 2 x attempts`, three improvements). **Whittle**, a recipe of Woodcraft's open to anyone (beside the arrows on Woodcraft's profession screen, #1334; anywhere; `Action::Whittle { item }`, replay text `whittle:LETTER`; asks for an item, the Whittle list, what `workable_wood` accepts) takes a weapon, missile weapon, shield, or armour whose main part is of a material of `group = "wood"` (bows, crossbows, staves, clubs), not a spirit weapon. Refused, taking no time, without a knife carried ("You have no knife."; its row greyed "no knife"), with a stack of another material ("Only a weapon of wood can be whittled."), or at three improvements. The check is the natural roll plus Dexterity's modifier, `TalentMods::check(Dexterity)`, and Woodcraft's `whittle` points (`Effect::Whittle`, curve 0, 8 at 100, 12 at 200), as Temper's is Strength with Metalcraft. Woodcraft earns a mark per finished attempt (`Use::Whittle`, 100 %).
- **Salvage** (`world/salvage.rs`, data `data/salvage.toml`; #1333; design `professions.md` §5; `docs/dev/crafting.md`): breaking an item into its materials, anywhere, with nothing. The abilities menu's **Salvage** (`Ability::Salvage`, Skills group, everyone's; `Action::Salvage { item }`, replay text `salvage:LETTER`; asks for an item) takes ten minutes (`seconds`, by `keep_at`; interrupted, nothing happens). Then one check per distinct material of the item (its resolved parts, gems included, in first-seen order) that a profession works (`data/stock.toml`: metal Metalcraft with Strength; wood, bone, hide, and sinew Woodcraft with Dexterity, and flint, stone, and obsidian by their own rows; jade, resin, and gems Jewelcraft with Dexterity; cloth and glass nothing): 2d12 + the stat's modifier + talents' checks of it + that skill's `craft` points (`Effect::Craft`, curve 0, 8 at 100, 12 at 200, read for that skill alone) against `difficulty` (12) + the material's `work`; a natural 2 fails, a natural 24 succeeds; untrained, the check is made the same. Made, the material comes back as stock, `yield` (50) % of its grams, at least one piece, or a gem whole with its size and cut; failed, it is lost, but a gem drops one cut (while above crude) or else one size (while above tiny), and shatters only on a natural 2 or with neither left to lose. The one item (one of a stack) is destroyed either way, its magic and improvements with it. Refused, with no time: worn or wielded, protected (quests, uniques, of Proof), unpaid, a spirit weapon, or nothing a profession works. Each profession rolled earns a mark (`Use::Salvage`, 100 %), that skill alone. No noise, no alignment.
- **Professions** (`content/recipe.rs`, `world/craft.rs`, `ui/profession_screen.rs`; #1334, design `docs/design/professions.md` §6, §8; `docs/dev/crafting.md`): a skill with a directory of **recipes** (`data/recipes/<skill>/<id>.toml`, one a file) is a profession: Alchemy, Jewelcraft, Metalcraft, and Woodcraft. Its row in the abilities menu ("Metalcraft…", in the skill's place) opens its screen of recipes, known by skill value (`from`, never learned). Whoever is trained in the skill sees every recipe of it, the untrained only those open to anyone (`anyone`): the built-in Temper (Metalcraft) and Whittle (Woodcraft), which run their own actions unchanged, and the arrows, bolts, and sling stones (#1337 Q3: they stay open to everyone). A recipe is greyed with the first reason it cannot be made now, decided by the rules: "beyond your skill", "no anvil beside you", "no knife", "no bow ready", "no crossbow ready", "no metal to hand", "not enough arrowheads". Recipes that make things (`result`, from stock and inputs at a `station`, in `seconds` against `difficulty`) are data checked at load, with the format's second half (#1350, `docs/design/recipes.md` §10): a part chosen from an input, a list of stock groups or materials of any profession, or None (`optional`, the result then the recipe's `brittle` object), a required cut gem for a part, inputs of a material or a group, `count = "fletch"`, a `failure` and a `fumble` object, and `data/recipes.toml`'s bands (Fine from a margin of 6, Marvellous 12, Masterwork 20, a cut's step 5, the counted bands 3 and 6, a failure's yield 50 %, the landscapes where wood is free), used by the phases that make such recipes. Jewelcraft's built-ins `use = "cut"` and `use = "set"` give their check's `difficulty` and `seconds` in their files; Set's row is greyed "not usable yet" until it comes. **Cut** (`world/cut.rs`, `data/recipes/jewelcraft/cut.toml`; #1355, design `docs/design/recipes.md` §5; for those trained in Jewelcraft): `craft:cut:LETTER` on a loose gem from the Cut list (any uncut gem; a cut one above tiny and below flawless), half an hour (an interruption spends only time), then one check: 2d12 plus the Dexterity modifier, Jewelcraft's `craft` points, and the talents' Dexterity checks, against 12 plus the gem's `work`. The margin sets the cut by `cut_step` (5): 0 crude, 5 fair, 10 fine, 15 exquisite, 20 flawless; below 0 failed; a natural 2 shatters the gem, a natural 24 cuts it at least fair. An uncut gem failed stays uncut and crude, both a size smaller ("The stone splits badly; what is left is smaller.", "You rough it out, and lose more than you meant to."); fair and better keep the size. A re-cut keeps the better of the old cut and the check's and always loses a size. A gem that would go below tiny is lost. Refused, taking no time: a tiny cut gem ("There is nothing left to cut."), a flawless one ("It could not be finer."), anything not a loose gem. The gem is said as names do ("It is now a medium crude ruby."), never the margin. Jewelcraft earns a mark per finished cut (`Use::Craft`, 100 %). `Action::Craft` (`craft:ID[:POS,...]`, `-` a skipped optional) makes a generic recipe (`world/craft_make.rs`; #1335, design `docs/design/recipes.md` §4, §6): the forge's seven swords (knife, dagger, short sword, scimitar, long sword, rapier, great sword; Metalcraft, at an anvil, from Metalcraft 0 to 40), each from the metal stock for its blade (as many pieces as the blade's share of the weight, whatever the metal) and an optional cut gem for its pommel. The **forging wizard** (`ui/craft_pick_screen.rs`) asks each choice in a paged list (the stacks that fit, a stack with too few greyed "too few", None on `-` for the pommel; Esc back a step, the pick kept), then the one dialog ("Forge a ruby-pommelled mithril short sword? It uses 2 mithril ingots and the ruby, at the anvil, and takes about 1 hour."). The work takes the recipe's `seconds` and an interruption spends only time; done, one check: 2d12 plus the profession's stat modifier (Strength), Metalcraft's `craft` points (`Effect::Craft`, Temper's curve), and the talents' checks of the stat, against the difficulty plus the main metal's `work`. The margin's band: 20 or more a **Masterwork**, 12 **Marvellous**, 6 **Fine** (three, two, one improvements, as Temper draws them, with `attempts` equal to them, so a crafted Fine item is a once-tempered one and Temper refuses a crafted Masterwork), 0 plain, below 0 failed (nothing made, half the stock back by `yield`, the gem kept; "The blade comes out brittle and you break it up for the metal."); a natural 24 makes at least Fine, a natural 2 fails. An item with no aspect is plain whatever the margin. The item is identified and joins the pack. Metalcraft earns a mark per finished work (`Use::Craft`, 100 %). Refused, taking no time, beyond the skill, away from the station, or with picks that do not hold. The rest of the melee weapons (#1351, `recipes.md` §6): the smith's stiletto, hand axe, battle axe, great axe, mace, morning star, war hammer, maul, spear, javelin, trident, glaive, and halberd at the anvil, and Woodcraft's **hand-made** ones, with a knife carried and no station (a knife, dagger, and hand axe of knapped stone or bone, a club of wood or bone, a quarterstaff, an ironwood staff, a spear and a javelin with a stone or bone head; ids `knife_by_hand`, `dagger_by_hand`, `hand_axe_by_hand`, `spear_by_hand`, `javelin_by_hand` beside the smith's `knife`, ...). **A recipe's profession is its directory's**, not its material's: a flint knife is Woodcraft's (Dexterity, Woodcraft's `craft` points and mark), a steel one Metalcraft's, though a profession's row in `data/recipes.toml` that names a `stat` or `verb` (Alchemy's) overrides the stock's; the `work` on the difficulty is the main chosen part's material's. A spear's or javelin's haft is chosen from wood billets on the smith's recipe too (`haft = { stock = ["wood"] }`, another profession's stock), greyed "no wood to hand" without them. Never made: a stiletto by hand, a battle axe or great axe of stone, a sword of dragonbone. A failure that is not of metal says "The work goes wrong, and you save what you can of it." **Brewing** (#1357, design `docs/design/recipes.md` §6 "Potions (Alchemy)"): Alchemy is a profession ("Alchemy…") with 29 herb recipes (`data/recipes/alchemy/`: healing, greater and full healing, mana, sustenance, antidote, cure disease, clear head, barkskin, sleeping draught, stinking flask, cat's, owl's, and moth's eyes, warmth, proofing, the six boosts and the six greater boosts, renewal), each 20 minutes, no station or knife, herbs only, the flask free, and 9 that take an elemental's **essence** (#1358, `recipes.md` §6, §9): the potions of frost (bog myrtle and an essence of ice), earthing (bog myrtle, earth), and sea legs (water, a glowcap), the flasks of fire (fire and a bought flask of oil), frost (ice, a glowcap), thunder (lightning, a glowcap), and smoke (air, henbane), pitch fire (two of fire and a bought flask of tar), and the blasting flask (fire and lightning), from Alchemy 25 to 100. Each essence is an elemental's drop (`data/drops/<element>.toml`; the fire-things' 25 %); oil and tar are bought, never brewed. A brew chooses nothing: its herbs are taken from the pack (the first stacks of each) and the one dialog confirms ("Brew a potion of healing? It uses 2 sprigs of moonwort, and takes about 20 minutes."); a row short of a herb is greyed "not enough sprigs of moonwort". The check is the forge's with the stat of the profession's row in `data/recipes.toml` (Intelligence: herbs are no stock) and Alchemy's `craft` points (+8 at 100, +12 at 200). Made, the potion is identified and its type learned: the maker knows their work (the dialog names it truly before). No grade. Failed, the herbs are spent and the recipe's `failure` object made, a **murky draught**; a natural 2 makes its `fumble`, a **brewer's blunder**, each said by the profession's line in `data/recipes.toml` and known for what it is. Alchemy earns a mark per finished brew (`Use::Craft`, 100 %).
- **Foraging** (`world/forage.rs`, data `data/forage.toml`, §7; #547; the owner, 2026-10-27:
  "survival can be used on wilderness map, some chance to gather food items"): Survival's use,
  the abilities menu's **Forage** (`Ability::Forage`, in the skill's place among the Skills;
  nothing to aim; `Action::Forage`, replay text `forage`; no key of its own). Anyone trained in a
  skill with the `forage` benefit has it, while `data/forage.toml` exists. Survival stays a
  passive skill (quicker journeys) besides.
  - **Where**: the wild only, `world::forage_ground`: on the overworld by the landscape of the
    terrain underfoot, in a local area by its overworld cell's, when the file has a row for that
    landscape (plains, grassland, forest, dense forest, hills, swamp; roads and water have none),
    and never standing in water or on ice. Towns, dungeons, and hand-made outdoor levels are not
    the wild. Elsewhere the menu greys it out ("not usable here") and the rule refuses, taking no
    time ("There is nothing to forage here."); untrained, "You do not know how to live off the
    land."
  - **Time**: the file's `seconds` (600: ten minutes), in turns of the `forage` action, stopped
    like a rest (something comes into view, a message, a blow: "You stop foraging."); a forage
    cut short finds nothing. It makes no noise of its own, as a rest makes none.
  - **The find**: the row's `chance` plus Survival's `forage` benefit (percent, at most 30), at
    most 100, one gameplay draw; found, one of the row's `finds` by weight (one draw) and its
    `count` dice. The food is fresh (made now) and goes into the pack: "You gather 2 handfuls of
    berries."; else "You find nothing worth eating.". Then Survival may earn a mark (`forage`).
  - **Herbs** (#1343): a forage that found something, by one trained in Herbalism, on ground
    with a `herbs` list, has Herbalism's `herbs` percent (25 / 60 / 80 at 0 / 100 / 200) of one
    herb besides, by weight, one of it: "Your eye for herbs finds an angelica root besides." Two
    more gameplay draws, a herbalist's only: anyone else's forage draws as before. The lists
    hold angelica root and the brewing herbs where they grow (#1340): moonwort and comfrey on
    grassland and in forests, feverfew and vervain in forests, henbane and bog myrtle in swamps,
    eyebright on hills; glowcap and mandrake never (the dungeon's). Herbalism marks on a
    forage, 25 %.
    Nothing is remembered: the same spot may be tried again at once (no anti-farming limit, the
    time is the price). Presentation shows the outcome only, never the chance.
  - **Food** found is ordinary food (§8 "Hunger"): berries, mushrooms, and hazelnuts (new), and
    apples and angelica roots.
- **Gathering herbs** (`world/gather.rs`, `data/herbs.toml`; #1340, design `recipes.md` §8): the
  dungeon's glowing fungus (terrain `fungus`, the file's `terrain`) is gathered for herbs, the
  inputs of Alchemy. The abilities menu's **Gather** (`Ability::Gather`, the *Here* group, as
  Pray and Drink: listed only beside fungus; `Action::Gather(Dir)`, replay text `gather:DIR`;
  no key of its own) is **everyone's**, while the file exists. With fungus on one cell beside
  the player the menu gathers there at once; with more it asks the way. A way with no fungus is
  refused, taking no time ("There is nothing here to gather.").
  - **Time**: the file's `seconds` (300: five minutes), in turns of the `gather` action (1000),
    stopped like a rest ("You stop gathering."); a gather cut short finds nothing, the time
    spent.
  - **The find**: trained in Herbalism, `count` (`1d2`, one gameplay draw per die) plus one for
    every `value_per_count` (50) of Herbalism's trained value; untrained, `untrained` (1), no
    draw. Each herb is one gameplay draw by weight from the `[[herb]]` rows at the level's item
    level (a dungeon level's depth; a hand-made level's `item_level`), read as `gems.toml`'s
    sizes are: `find` at level 1, `deeper` added per level past the first (negative for
    moonwort), never below 0, none below `level`. Nine herbs: glowcap 40/0/1, moonwort 30/−1/1,
    comfrey 20/0/1, feverfew and henbane 10/2/3, eyebright, bog myrtle, and vervain 6/2/4,
    mandrake 2/2/6. Into the pack: "You gather 2 sprigs of moonwort and a glowcap." Then
    Herbalism marks (`gather`, 100 %, the training stream).
  - **The fungus never depletes**: nothing changes on the level (no picked state, no light
    change), so it may be gathered again at once; time is the only cost. Fungus is placed by
    the caves generator's patches and hand-made levels; the rooms generator places none.
- **Talents** (`warlock-core/src/talent.rs`, the runtime; `content/talent.rs`, the data;
  `world/talent.rs`, the picks; M30, #229; design `docs/design/progression.md` §7, §10): knacks
  a character picks as they level, each a typed effect read where the rules already read what
  magic items do.
  - **Data** (§7): `data/talents.toml` (the settings: the `[[tree]]`s
    in display order, the `[[system]]` list) and `data/talents/<id>.toml`, one `[[talent]]`
    a file (`id`, `name`, `tree`, `description` in the game's words,
    `choice` (`weapon_skill`, `style`, `skill`, `element`, `stat`), `needs`, `effects`,
    `systems` from the list, `granted`:
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
  - **Heritage** (`world/heirloom.rs`; #697): talents offered only at creation (`needs = {
    creation = true }`, `Need::Creation`: "to be a new character", met at character level 1
    only, so never a level's pick), in the `heritage` tree (`no_free`). What they give is handed
    out once, as they are taken (`World::hand_down`, from `World::learn`), on top of the kit:
    **Family wealth** (`starting_gold`: 300 % of the gold coins carried more, at least 60 and at
    most 150; `starting_gems`: 2 to 3 loose gems at item level 2, from the stream
    `character/family-wealth/<race>,<gender>,<class>`) and **Father's sword** (needs Family
    wealth; `heirloom`: the race's weapon from its table, else the fallback, with 2 to 4 affixes
    at item level 6 (`loot::heirloom`: two a magic item's prefix and suffix, more a rare item's,
    named), identified, readied in its own slot when free, else carried, from the stream
    `character/fathers-sword/<race>,<gender>,<class>`). Each stream is its own, so no other roll
    moves. The weakest of a small race may start Burdened with them, never Strained. (Favoured
    child, which doubled the background story's luckier odds, was cut in #787 with its
    `background` system: no talent is left on the story's draw.)
  - **Prerequisites** (`Needs`; all must hold at the pick and are never checked again: a score
    lowered later keeps the talent): other talents (a talent with a choice may be
    needed only by one of the same choice, taken for the same), stat **scores** (the stored
    score, never the effective one), skill values, weapon skill or style ranks, any one of
    several weapon skills at a rank (`any_weapon_skill = { roles = ["melee", "unarmed"], of =
    [...], rank = 3 }`: "a melee or unarmed weapon skill 3", "Maces and clubs or Unarmed 3"), the
    chosen weapon skill's or style's rank (`chosen`), the character level, the race, the class, the
    eyes (the race's vision profile), and the birth sign's animal and element and a lucky day
    (`animal`, `element`, `lucky`; §8 "Birth sign"); and any one of several (`either = { stats =
    { intelligence = 12 }, skills = { appraise = 20 } }`: "Intelligence 12 or Appraise 20").
    Never an alignment band (§8 "Alignment": alignment bars nothing). Refused in words: "You cannot
    take Dodge: you need Dexterity 13." A talent is taken once ("You learn Hardy."); a talent with a choice is
    one pick per choice.
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
    Dodge, Brawn, Silent feet (since folded into Light step), Keen ears, Pack mule, Long strider (cut in #787), Lean, Good
    learner (cut in #774), Alert, Thick skin.
  - **Weaponcraft and Guard** (#238): `weapon` adds to the benefits of the chosen weapon skill
    or style (no `role`) or of every skill of a role, read in `Actor::benefit`, so every
    attack, Defence, and price that reads a skill's benefits reads it: Weapon focus (+1 to hit),
    Weapon specialization (after Weapon focus for the same skill, that skill at 5: its benefits
    read a rank higher, never past 10), Improved critical (threat one roll wider), Stylist (the
    style's benefits one rank higher, never past 10, and only for a rank held), Two-weapon
    training (both two-weapon penalties eased by 2), Shield expert (Sword and Board 2: Shield +1 in a shield style; Sword and
    Board's free bash 10 more likely where it has one), Deadly aim (+2 damage with shots),
    Thrower (+1 damage and 10 % cheaper throws), Rapid reload (reloading 25 % cheaper);
    `launcher_range` and `throw_range` (Far shot: +2 and +1 cells); `max_dex` (Armour
    training: worn armour's Dexterity cap +2); `deflection` (Deflection +1);
    `protection` (Heavy armour mastery: +1 while the armour piece worn with the most
    Protection has 3 or more; Shield wall: +1 while holding a shield); `stance_to_hit`
    (Stalwart: the defensive stance's to-hit penalty gone); `pain` (Immune
    to pain: a hit, a blow, shot, throw, or trap, does 1 less after the soak, never below 1;
    a status's or starving's harm is no hit); `resist` (Elemental ward: 25 % against
    the chosen element, any damage type that is no physical kind, summed and capped with the
    gear's). A weapon-skill choice is a skill that attacks: never a style (chosen as a
    style). Steady hand (the ranger's focus) came with the
    resource talents (R5, §8 "Resources").
  - **Vigour, Might, Finesse** (#237; the design's §7.5; Battle trance and Inner calm, on the
    class resources, came with R5, §8 "Resources"): `resting_percent` (Fast healer: resting's pace
    half again, `World::healing_rate`); `second_wind` (Second wind, the first **active** talent:
    `Ability::SecondWind` under the abilities menu's new **Talents** group (`Source::Talent`,
    between Class and Skills), `Action::UseTalent` (`use:second_wind`, the verb every ability use has; #644): 1 hit point per character
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
    below 1, never one that does not recover); `free_blow` (Cleave, and Great cleave after it;
    Riposte, Opportunist); `kick_die` (Mighty kick: kicks at creatures roll 1d6) and `door_kick`
    (Mighty kick, +4 to a door kick's Strength); `kick_back` and `dig_percent` (Breaker, after
    Mighty kick, Strength 14: a kick that hits drives the foe one cell along the kick when that
    cell is walkable, untrapped, unfrozen, and empty, `World::knock_back`, no draw; a quarter
    more work a turn of digging); `two_handed_price` (Iron grip: blows with a two-handed weapon 10 % cheaper,
    `World::attack_price`); `critical_status` (Staggering blow: a critical confirmed in melee
    leaves the target Staggered, `cost = 150` until its next action: that action half again
    as dear; `World::after_blow`, no draw of its own); `penetration` (Crusher: +2 on blows and kicks
    that can do bludgeoning damage); `burden_dex` (the burden line's Pack mule and Porter: the burden's Dexterity cap +1 each);
    `sure_step` and `pit_proof` (Light step: a trap stepped onto gets a Dexterity check against
    its difficulty first, two draws only with the talent; made, it does not go off and is found;
    pits never catch you; steps only, never a door's trap; the talent also has Silent feet's
    effects, folded into it: `footsteps_percent` -40 and `silent_doors`); `speed_percent` (Quick:
    +5 %, every price divided by it in `World::burdened`, as a higher speed; hidden in the
    game); `missile_dodge` (Deflect missiles: Dodge +2 against shots, throws, and trap missiles,
    `World::target_against`); `spell_damage_percent` (Evasion: bursts and bolts, traps' and
    scrolls', do half, rounded down, before resistances). The statuses these name are checked
    at load; `data/status.toml` loads before the talents.
  - **Class abilities** (#246; the design's §4): a class's level table (`abilities` in
    `data/classes/`, `{ level, talent }` rows) names talents for its levels (1, 6, 12, 18, 25,
    32, 40, 50, and 60 at first; the culls of #774 and #787 left some empty, and the ranger's
    Stalker's eye came at 2 with #788): a talent of the **Class abilities** tree (`class = true` in
    `data/talents.toml`; `granted`, never on a pick list) or an ordinary talent it hands out
    (the aristocrat's Riposte at 18, the commoner's Second wind at 32). The player is given each
    as they reach its level ("You gain Fencing." with the level's news), level 1's at
    creation, and again whatever is missing when a run is restored, so a grown table reaches a
    save; one already held (picked earlier, or a story's boon) is not given again: its level
    brings a talent pick in its place instead ("You know Second wind already: choose a talent in
    its place.", #790; not on a restore), and nothing is taken away. What
    a talent hands out once comes with it: hit points by level, and **stat gifts** (`stat`: score
    and potential, Iron will's Presence +1; a Toughness gift raises the hit points with the
    maximum). Its `bonus_talents` (`levels`, `trees`) add a **bonus pick** at those levels
    (`Due::bonus`), of a talent from those trees (none named: any, the commoner); a talent of
    those trees spends a bonus pick before an ordinary one, any other is refused for it ("You
    must choose a talent of Weaponcraft or Delver."), and a bonus pick nothing fits is dropped
    like any other. The everyday folk have theirs (bonus picks at 10, 20, 30, 45, 55):
    aristocrat (Weaponcraft, Delver) Fencing (Duelist one rank higher), Riposte, Iron will (+2
    on Willpower checks, Presence +1), Peerless (+2 on the confirmation roll); commoner (any
    tree) Pack mule (the burden line's first step, #1198), Stubborn (+1 on
    Toughness checks, poison never lowers scores), Big hands (+1 damage with maces and
    unarmed), Second wind, Salt of the earth (healing and resting +25 %), Unlikely hero (every
    stat +1); craftsperson (Weaponcraft, Guard, Delver) Sturdy work (Protection +1 in body
    armour), Masterwork (melee threat +1), Artisan (Protection +1 per armour piece worn, +2
    melee damage); merchant (Delver, Scholar) Haggler (favour +15 %: the one class ability on
    shop favour, #787), Deep pockets (coins weightless) and Pack mule (the burden line's first step, #1198), Merchant's eye (#816, at 12: an
    ability, on a thing carried, that identifies it as the scroll of identify would, for a
    reading's time and nothing else; refused, no time, for a thing already known); #789 cut its level 40 Insured (shops paid +50 %), a
    capstone that paid only in gold. The favour cap is 50 %. #774 cut the filler and the duplicates (Connoisseur, Hearty, Keen
    appraisal, Jeweller, Appraiser) and #787 the weak ones of the #776 review (Well dressed,
    Noblesse, Commanding, Legacy, Thick skull, Folk hero, Whetstone, Fletching, Mining, Tinker,
    Trade routes, Credit, Magnate, Golden tongue), and left their levels empty. The adventurers'
    tables come with I1–I3 (#243–#245). Dumps mark them `Haggler (class)`;
    JSON `talents[].class`, `due.bonus`. **Saves change** (`Due::bonus`, `Magic::carried`).
  - **Prestige classes** (#876, P1 of `docs/design/prestige-classes.md` §4; the owner, PR #835:
    "not going to be a general multiclass thing. More like prestige classes achievable by
    finding a 'hidden' mechanism"): a character has **one level**, read off the one curve from
    their XP as ever, but it stops at their class's **cap**, `ClassDef::max_level`
    (`data/classes/`, default `data/experience.toml` `[levels] class_max`, 60; the bard's
    30, #874, the only class below it; checked at load: at most the curve's top), plus a **prestige class's** `levels` once they are called
    (`Character::level_cap`). A prestige class (`data/prestige/`, `PrestigeDef`: `id`, `name`,
    `description`, `levels`, `hp`, `mana`, `rolls`, `known_for`, `abilities`,
    `bonus_talents` at its own levels; never a class's id) is a class in everything a level
    reads and nothing a creation reads. Levels up to the cap are the class's; each level past
    it is the prestige class's at its own level (`Character::class_of`: a bard capped at 30
    called becomes Hunter 1 at total 31, Hunter 30 at 60). **Every per-level read goes through
    the class of the level** (`content::Table`, the one view of either): hit points (half its
    `hp` plus Toughness, `World::level_hp(content, level)`), mana (`level_mana`), skill rolls
    (`roll_count(.., level)`), the bonus pick and its trees (`Table::bonus_at`,
    `World::bonus_trees_at(level)`: each `Due` keeps its level's trees), and the class abilities
    table by table (`class::grant` over `Character::class_levels`, at creation, as each level
    comes, and on a restore); a level-up roll steps up the ladder for a skill **either** class is
    known for; talent picks (every third level), stat checks, casting (the class's percent of
    the total level), and the experience factor are unchanged. **The calling** is the door:
    `CallingDef` (a campaign's `data/callings/`, `content/calling.rs`: `class` a prestige id,
    `from` the base classes, each named by one calling of a campaign, `level` (each one's cap in
    the shipped data), `message`; one outside every campaign is a fixture's and any run's). It is
    **open** (`World::calling_open`) to a character of a `from` class at that level with no
    prestige class yet; `World::call` (`world/calling.rs`, the one place a prestige class is
    given) sets `Character::prestige` (saved, hashed, left out while none; never cleared),
    says the `message`, and gives **exactly the XP the next level costs** if the bank is short
    (`add_xp`'s rule, never `grant_xp`'s scaling; a full bank is spent as it is and gives every
    level it buys), then the level-up (`World::rise_from`) with its hit points, rolls, picks,
    and the new class's abilities ("Welcome to level 31.", "You gain Spirit weapon."). Unmade's
    lost level lowers the total and so the prestige level, and gives back as it returns; the
    choices were made the first time. Who calls: the dev action `call:ID` (`Action::Call`, no
    time, whatever the level; refused with no such class, one held, or no calling open to the
    class), and a dialogue topic's `calls` (§8 "Dialogue", "Callings": the Ascmearc's
    Strange Spirit, §8 "Creatures"; the test world's hermit); an encounter entry's `when = {
    calling }` holds while a calling is open. Both worlds' Hunter callings ask for the bard's
    `max_level`, 30 (#874). The character screen's Class row:
    `Bard`, `Bard 30` at the cap, `Bard 30, Hunter 4` after; the Characters list's summary the
    same; the death screen "Here lies Brakka, / a bard and hunter,"; the XP row shows no next
    level at a cap. Dumps: `stats` `Class Bard 30, Hunter 4`, `progress` (dev) `calling hunter:
    open` / `level 30 not met` / `not for a thief` / `called hunter at 30`, JSON
    `character.prestige`, `character.classes`, and with `--state` `callings`.
    `data/prestige/README.toml` has the format; the tests' fixture is the Warden, called from a
    bard capped at 12 (`world/calling_tests.rs`). No new `[[system]]` and no talent: a prestige
    class is a class. **Saves change** (`Character::prestige`).
  - **The Hunter** (#881, H1 of the design, §9): `data/prestige/hunter.toml`, 30 levels, `hp = 7`
    (the warrior's), two rolls, known for Athletics and Survival, bonus picks at its own 5, 10,
    15, 23, and 28 from Weaponcraft and Guard, and one row: **Spirit weapon** at its level 1 (the
    owner: "regular martial progression with no abilities above level 1 for now"). Its door is a
    campaign's calling (the Strange Spirit's, C1). Spirit weapon (`talents/spirit_weapon.toml`,
    effect `spirit_weapon { to_hit = 3, threat = 2, ignores_protection = true, phrase }`;
    `world/spirit_weapon.rs`) is a class active asking for a **shape** (`Aim::Shape`, the shape
    list: every `melee_weapons` object with an attack a hand holds, 22, grouped by weapon skill,
    each as a plain one is listed; `use:spirit_weapon:long_sword`, `Action::Shape`; refused, no
    time, without it or for an id that is no shape's). It makes a `Stack::spirit` of that base
    (count 1, no magic, known; weighs, is made of, and is valued as its base; saved, left out
    while false), puts it in the pack, and wields it in the main hand through the equip path,
    what the hands held put away into the pack as a swap does and paid for as one: the call is a
    wield (`ClassActive::cost`), and nothing else is spent (no mana, cooldown, or limit). One
    held already fades first, so calling again reshapes. In a hand (the off hand too) and with
    the talent, its profile has +3 to hit, a threat 2 lower (never below 3), and every point of
    Protection ignored (`penetration` `ALL_PROTECTION`, no soak drawn). **The invariant**: a
    spirit stack exists only in a hand slot. Taken out of the hands and left in the pack (removed,
    swapped for steel, moved to the missile weapon slot) it fades at once ("The long sword fades
    from your hand."); a drop or a sale takes it off and lets it fade instead of letting it go
    (the shop never sees it, the floor never holds it); a throw is refused with no time ("It will
    not leave your hand."); quests never count it; a tithe takes spirit-stuff,
    which is called again. Its description adds the effect's `phrase` as a power line, its
    numbers and critical range fold the gifts in, and "Pierces armour"; item lists mark it
    `(spirit)`; the character screen's attack says `pierces armour`. Creatures never hold one.
    **Adds an action and saved state** (`Stack::spirit`).
  - **Scholar, Grit, Delver** (#240; the design's §7.5 but for the five mana talents, which
    came with #242: Deep well, Clear mind, Meditation, Frugal caster, Blood magic, §8 "Mana"). Their effects share what the class abilities use where the rules
    are the same (`checks` and `stat`: Iron will, +2 on Willpower checks and Presence +1). Iron will is a Grit talent the
    aristocrat's table also gives. #787 cut Keen appraiser, Silver tongue, and Well liked;
    Haggler is the merchant's class ability alone. New: Scroll savant (Scholar; Intelligence 13; #775 merged
    Careful reader into it): `read_percent`
    (reading takes half the time; with the wizard's Scroll adept a quarter, the least talents
    leave) and `spares_self` (the caster's own burst, a scroll's or a spell's, never catches
    them, and the game does not warn before reading one close); Spell recall (Scholar; after
    Scroll savant, Intelligence 15, level 12) `scroll_kept` (one read in four keeps the scroll,
    one gameplay draw a read with the talent, "The writing does not fade from the scroll."); `loot_magic_percent` (Treasure hunter: the weights of the rarities better than
    a find's least half again, so anything found is magic and rare more often and a creature's
    drop, always magic, rare more often; from the same draws of the loot streams, so a map never
    changes, and a level made before the talent keeps its loot); `skill_rolls` (Quick learner: a
    roll more every level); `skill` without a `skill` field (Skill focus, the `skill` choice,
    a skill the taker is trained in, one pick for each: the chosen skill counts 25 higher wherever
    a benefit reads it, past 100 too, as an affix's bonus does, and sums with Master of none and
    magic items; the value trained and shown does not move); `ignore_burdened` (Porter: Burdened costs nothing, as a dwarf's); the **burden line** (#1198): Pack mule (Strength 11: carrying +10 %, `burden_ease` 60, `burden_dex` 1), then Porter (Pack mule, Toughness 12: carrying +10 %, `burden_dex` 1, `ignore_burdened`), then Beast of burden (Porter, level 12: carrying +10 %, `stout`, Strained costs what Burdened costs others, `Load::penalty`), each a step further; it replaced Roc's grip, Stout, Hard work, Sure-footed, Nimble, and Uncanny dodge, and Deep pockets' 10 kg;
    `corpse_stats_percent` and `no_bloating` (Gourmand: a fresh corpse's stat training marks and
    its chances to widen a potential doubled, a chance at most certain; a meal stops one short of Bloated, what does not fit is lost, and
    nothing overeats); `searching` (Keen searcher: +4
    searching) with the birth signs' `search_reach` (a cell farther); `search` and `trap_damage_percent` (Trap sense: +3 to finding hidden
    traps and secret doors, searching and in passing; a trap's damage halved, rounded down, at
    least 1); `door_kick` and `kick_noise_percent` (Lockbreaker: a door or chest kick as if 6
    stronger, half as loud) and its `smash_destroy_percent` (what a chest kicked open holds
    breaks half as often, 25 % a stack; a blown-up chest is no kick; Deft breaker's, merged in
    #775), and `chest_wise` (Chest-wise, Delver, Intelligence 11 or Detect traps 15: look mode
    tells a locked shut chest in sight from an unlocked one, and the trap of a shut chest in
    sight is known at once, no check; with Mimic-wise's `sees_mimics` 2 and Slick's `slick`,
    merged in #775), the `chests` system's (§8 "Chests"), with Lockbreaker
    and the wizard's Knock; `dig_percent` (Tunneler: half again the work a turn). A prerequisite may be `either`
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
    To 12; chosen for an element, any kind that is not physical: 25 %,
    counted as gear in `Actor::resistances`, so it also shrinks the doses of a status that is
    damage, Poisoned for a ward against poison, and the character screen says "Resistant to
    fire" in words) and Crusher (Might; St 13, Maces and clubs or Unarmed 3: `penetration` +2
    on melee blows, off-hand blows, shield bashes, and kicks that can do bludgeoning damage;
    never shots or throws) make the `damage_types` system's two. The witch's Blight (#245) is poison-typed when it lands: its burst names
    `damage_type = "poison"`, and the Poisoned it gives meets poison resistance as every dose
    does (`World::afflict`), so no rule of its own.
  - **Shadow and Wayfarer** (#239; the design's §7.5 but for Hooded light, which waits for the
    lantern's hood to need one (#235; the hood is now a lantern's mode, #643); Patient blade, on the thief's openings, came with R5, §8
    "Resources"):
    **effective eyes** (the design's B4, `WorldState::eyes`, §5 "Effective eyes"):
    `threshold_percent` (Night eyes: −50 %, eyes normal), `darkvision` (Dark adaptation: +2,
    eyes darkvision or low-light, `eyes = ["darkvision", "low_light"]`; it took in Owl eyes,
    #817); `scent` (Blood scent: a bleeding creature in line of sight
    within 9 cells, four beyond darkvision's five, is seen however dark, eyes darkvision);
    `lantern_light` (Lamplighter: a lit lantern ready in the Tool slot burns as `lantern_bright`, a
    cell farther, not while hooded, in `WorldState::carried_light`, so creatures see it too); `ambush` (Ambusher: +1d6 after Protection on
    a hit, any attack, at a target that has not seen the attacker (`World::unaware`, §8
    "Surprise and sneak attacks") or is Surprised; the die is drawn only then,
    `world/ambush.rs`); `terrain_as` (a step onto these terrains costs one onto
    another, `World::step_price`: Swimmer, still, running, and shallow water as floor);
    `plunge_proof` (Swimmer: a plunge through ice does no harm and draws
    nothing); `beast_friend` (Beast-friend: creatures of `animal` factions are
    neutral to the player until they attack one, `Progress::empathy_broken` by creature type,
    saved; read by `World::stance`, the AI's choice of a hostile, and look mode). #787 cut
    Pathfinder, Winter-born, and Sun-blessed (`sun_healing` with it), and folded the `weather`
    system into `travel`, the outdoors; #791 cut Marsh walker (outdoors only; `wading_proof` is
    Nature's ward's now). Light step also counts toward
    the stealth and hearing systems. The light and factions systems are held to the bar since it is one
    talent (#773). The `awareness` system came off `pending` with
    surprise (#681, §8 "Surprise and sneak attacks"): Wary (Delver, Willpower 12,
    `initiative_bonus = 4`: added to the initiative check) and Cool head (Grit, Willpower 14,
    level 9, `surprise_ease = 100`: Surprised slows no action), beside Ambusher; #687 added
    Stalker's eye (`sees_unaware`: the perception snapshot tells which creatures in view have
    not noticed the player, `SeenActor::unaware`, read by look mode's "unaware of you" and the
    unaware mark, §4; also `sneak_attack`; a Shadow pick until #788 made it the ranger's class
    ability at level 2) and Hair trigger
    (Finesse, Dexterity 14, Initiative 20, level 12, `startled_blow`: an initiative check made
    against a hostile attacker beside the holder is answered at once by a free blow, before the
    attack is rolled, `Cue::Startled`; also `free_blows`). The
    `sneak_attack` system (#682, "Sneak attacks: dice, reach, vitals") came with Anatomist
    (cut in #787) and Long reach
    (Weaponcraft, any launcher or thrown skill at 4, `sneak_reach = 3`: shots, throws, and attack
    spells sneak-attack 3 cells farther), beside Ambusher and the thief's Sneak attack, Improved
    sneak attack, Master thief, and Patient blade. The `danger` system (#326, §7 "Danger
    levels") came with Quiet delver (`spawn_every_percent`, Delver; Sixth sense was cut in #774). The `special_rooms` system (#402, §7
    "Special rooms") came with Sense of place (cut in #787) and Old delver (Delver, #775, which merged Light tread,
    Anchored mind, Grave warden, Hallowed rest, and Hoard-wise into it and dropped Mirror-wise
    and Treasure sense; Willpower 11 or Dexterity 12): `collapse_proof` (a
    collapsed floor never gives way under them, `World::collapse_step`, which then draws
    nothing), `room_status_immune` `ebbing` (a draining room's or level's Ebbing never takes
    them, `World::dwell`), and `grave_warden` (nothing they kill rises in a cursed room,
    `World::curse_corpse`); Old hand (Delver; after Old delver, level 10): `room_pace_percent` 50 (where a room
    quickens healing or mana, the quickening is half again more, `World::room_pace`: a
    sanctuary's 300 % becomes 400 %) and `hoard_grace` 1 (a tempting hoard seals only after the
    second stack lifted inside it, `World::lift_in_hoard`); and Pious (Wayfarer, free,
    #788, which merged Devout and Pilgrim into it; also `talk`): `devout` (at a forgotten altar of the player's own side the side's lean counts
    twice, at one of the other side not at all, `leaned_kinds`) and `pilgrim` (temple services
    cost half, rounded up, and a cure also mends a point of each damaged stat,
    `world/service.rs`).
    Hush-foot (#405) was cut in #787.
    The `spells` system (the magic design §10, #400) came with the **Spellcraft** tree: Quick
    study (free, #396: study +2, study time −50 %), Overreach (level 6,
    `study { overreach_percent = -33 }`, the spell scaling design §7.2: a spell's levels over
    the learning count a third fewer, rounded down, in the study check;
    `TalentMods::study_overreach_percent`), and School focus (free, `choice = "school"`: a school or
    an element, the spell tags of §8 "Spells"; `spell_mod { cost = -10, cells = 1 }`: a spell
    carrying the tag costs a tenth less and reaches a cell farther, range or length); Keen mind
    (free, the same choice: `spell_mod { sides = 1 }`, its spells' dice a size bigger); Sure
    hand (level 12: `spell_mod { reroll = 1 }`, a spell die showing 1 rolled again, once);
    Brimming (level 25: `spell_mod {
    dice = 1 }` and `spell_mana_percent` +10, systems `spells` and `mana`) (the spell scaling
    design §7.2: every spell hook ships with a user); the casters' Empower, Archmage, Scroll
    adept, Frugal caster, the wizard's Arcane training, and every class spell name it too. Wand mastery comes with
    wands (#398) and Channeler with rods (#399), whose hooks they need.
    The `schools` system (Schooled, Devotee, and the wizard's Magister) was removed with the
    schools of magic as trained skills (#1263).
    The Defence by attack kind of the design's B5 landed with Deflect missiles (#237,
    `World::target_against`).
  - **Racial talents** (#241; the design's §7.5 Racial): the `racial` tree (`no_free`), each
    talent asking for its race. Elf: Elven accuracy (bows 3: threat +1 with bows, `weapon`),
    Ageless (level 9: hunger 20 % slower, and `disease_interval_percent` 100, a disease's
    progressions twice as far apart, `World::disease_turn`). Half-elf: Diplomat (Presence 11: favour +10 %, quest gold +25 %). Dwarf: Stone
    sense (`stone_find` +4 to finding what is hidden on a `stone` cell, searching and in
    passing; digging +25 %), Dwarven resilience (Toughness 14: Poisoned eases twice as fast,
    disease checks +2). Halfling: Lucky (`reroll_fumble`), Slinger (slings 2: sling damage +1, and
    `launcher_range` with `skill = "slings"`, +2 cells for a sling only), Small target
    (Dexterity 13: `small_target`, Dodge +1 against an attacker of a larger size,
    `World::small_target` after `target_against`). Gnome: Tinkerer (Intelligence 12: the Disarm
    ability untrained), Gem lore (gem worth shown,
    gems found twice as often). Half-orc: Ferocity (`ferocity` +2 damage on every attack while
    below a third of the hit points, `Actor::stanced`), Orcish hide (Toughness 13: Protection
    +1). Humans are **granted Versatile** (`talents = ["versatile"]` in `data/races.toml`; a
    race grants granted talents of the racial tree): `extra_pick` at
    level 1 (a creation pick more, `creation_count`) and at level 30 (`World::level_up`). A
    race's grants join the birth sign's (`stars::grant`, at the start and on every restore),
    shown as the race's ("(race)" in the dump).
  - **Systems**: a talent may name the systems it touches from the list in
    `data/talents.toml`; the wiki's talents-by-system table reads them. The load checks only
    that the talent table is well formed (#1205): it parses, every id it names (talents, trees,
    systems, statuses, resources, skills, weapon skills, races, classes, eyes, damage types,
    lights, spells, terrain) resolves, a choice-keyed effect has its choice, and a die's sides or
    a divisor is not 0. Whether a talent is worth taking, alike to another, or in range is the
    data author's call.
  - **Determinism and saves**: picks are replayed actions; nothing here draws from the gameplay
    stream; the `levels` stream (`WorldState::levels_rng`, for the skill rolls and stat checks
    to come) is made on its first draw. `Actor::talents` (by id, like affixes: adding talents
    keeps saves; on every actor, so creatures may get some from data later), `Progress::due`,
    and the stream are left out of the JSON while unused, so runs without talents hash as
    before. **Saves change.**
  - **What the player sees**: `PerceptionView::due`, `own_talents` (names for their choice,
    the description), `talents` (every talent with its
    prerequisites and whether each holds, and the ways it may be taken now); the dumps'
    `talents` and dev `progress` layers; JSON `talents`, `due`. Never a rule's number: the
    wiki's Talents page states them.
- **The adventurers' class tables, and the class actives** (`content/class.rs`: `ClassActive`;
  `world/class_ability.rs`; M30, #243; design `docs/design/progression.md` §4.2–4.5): the
  warrior, berserker, paladin, and ascetic have level tables as the everyday folk do (the
  "Class abilities" bullet above): `abilities` rows naming talents of the class tree
  (`data/talents/`, `granted`) and `bonus_talents`.
  - **Who has them**: given as the level comes, "You gain Rage." (the shared rules of the bullet
    above); each is read with the talents through `Actor::talent_mods`. Their effects are the
    talents' kinds and the class's own (the effects README of `data/talents/`); an active's
    effect (`rage`, `veterans_wind`, `whirlwind`, `lay_on_hands`, `smite`, `radiance`,
    `quivering_palm`) makes the talent that has it the ability's name and description in the
    menu; a holy gift's `alignment` names the bands it works for (§8 "Alignment"). Deflect
    missiles left the ascetic's table for Stunning strike at 12 (#848); it stays the ordinary
    Finesse talent, a bonus pick away. A shared entry is one talent the tables
    name alike (`purity_of_body`: the paladin's, the priest's, and the ascetic's; Divine health was
    its copy until #790).
  - **Bonus picks**: warrior (Weaponcraft or Guard) at 4, 10, 16, 22, 28, 34, 46, 58;
    berserker (Might or Vigour), paladin (Guard or Vigour), ascetic (Finesse or Vigour) at 10, 20,
    30, 45, 55; as the bullet above says.
  - **Actives** (`Ability::Class`, the abilities menu's Class group; `Action::Use`, `use:rage`,
    `use:smite:E`; refused, no time, without the ability or while its cooldown lasts; a holy
    gift outside its bands is used and does nothing, §8 "Alignment"): Rage (the Rage resource and with it Raging:
    St +2, To +2, In −2, no reading or casting, while the rage lasts, §8 "Resources"; then
    Exhausted 150 s; no rage while either lasts; a wait's time), War cry (the berserker's at
    25, #844, `world/war_cry.rs`: 4 rage, the talent's `resource`, and a wait's time; a roar as
    loud as a howl, `Act::Howl`, heard across the floor, waking dens; every hostile that hears
    it, sees the player now, and has vitals rolls its Willpower modifier against 10 + the
    player's Strength modifier + their Intimidate, two gameplay draws apiece in actor-id order,
    no level cut; failed, Frightened 3 degrees, caused by the player, it runs from them; "The
    kobold quails and flees from you." or "The kobold does not flinch." for those the player
    sees. Its one refusal is the rage: not held, or less than 4, "You have not the rage for
    it.", no time; a roar with nobody to hear it still spends the turn, the rage, and the noise;
    each cry marks Intimidate),
    Veteran's wind (1 hit point a level back, at least 10, then Winded; a wait), Whirlwind (a
    blow at every hostile beside, clockwise from the north, each with the free blows the end of
    an attack brings, for one melee attack's price and one exertion), Lay on hands (2 hit
    points a level, for 6 zeal, §8 "Resources"; a cast; works for Kind or better), Smite (the
    paladin's at level 1, #910, `world/smite.rs`: a blow in a direction with 1d6 more per 10
    levels, at least 1d6, and a die more against the unholy, a creature type carrying a tag of
    `data/smite.toml`'s `unholy` list: the undead and the fiends, never read by alignment; the
    paladin's Presence modifier added to hit; a magic weapon's extra die folded into the bonus at
    its average; for 3 zeal, granted by the talent, no cooldown; works for Kind or better,
    unanswered it spends no zeal), the **modes of smiting** (the paladin's, #911,
    `docs/design/paladin-smites.md` §6; each its own row and `use:ID:DIR`, Smite's blow, then a
    rider on a hit that did damage; a rider with nothing to do still spends its zeal; works for
    Kind or better): Branding smite (level 12, 3 zeal: the foe is Branded, five minutes of the
    faint item glow shed wherever it goes, so it is seen in the dark and lights its fellows),
    Wrathful smite (level 32, 4 zeal: the foe makes a Willpower check against 13 plus the
    paladin's Presence modifier, everyone alike, and failing is Frightened by the paladin for 2d3
    turns, fleeing them), and Banishing smite (level 50, 6 zeal, #913: a foe that was summoned is
    unmade, dying there whatever its hit points, the paladin's kill as any other; an unholy foe
    not summoned takes the smite's dice twice over, the rest of the blow once; anyone else, a
    plain smite), Holy radiance (on and off: Radiant, with no end of its
    own, sheds the faint item glow, reaching 3 cells, from the player, a light source like a
    glowing item's; a cast; works for Kind or better), Patient defence (the ascetic's at 18, #849: for a
    point of Ki and a wait's time, Poised until their next action: Dodge +4 from the status's own
    `combat`, and a melee blow on them missed by their Dexterity, Dodge, or Parry answered at once
    by the riposte, which the status switches on in `World::free_blows_of` (a blow missed by
    Dexterity or Dodge is answered only while Poised: `Cue::Attacked { dodged }`, raised only by
    a creature's attack, `World::attack`); refused, no time or ki, only with no ki; never for
    nobody near, and taken again while Poised it holds one action more; any grip), Step of the wind (the
    ascetic's at 6, #847: for a point of Ki and **no time**, a degree of Light-footed, so the next
    action, whatever it is, costs half its price and ends it; refused, no time or ki, while
    Light-footed already, "You are already light on your feet."; any grip), Stunning strike (the ascetic's at 12,
    #848, `world/ascetic.rs`: a bare-handed blow in a direction at a melee attack's price,
    refused with a weapon in hand or nobody hostile there before anything is spent; a hit that
    does damage leaves the foe **Stunned**, "You stun the kobold.": `per_action`, one degree,
    `refresh`, `cost = 150`, `flat_footed`, §8 "Surprise and sneak attacks"; for 1 Ki and no
    cooldown), Quivering palm (the same blow, `World::palm_blow`, one body for both; a hit that
    does damage leaves three degrees of Bleeding; for 5 Ki, §8 "Resources", and no cooldown),
    Stillness (Still mind's active, the ascetic's at 25, #850, `world/ascetic.rs`: for 2 Ki and
    **no time**, every status of its list the ascetic suffers ends at once, each with its own end
    words after "You still your mind.": Held, Pinned, Staggered, Stunned, Surprised, Transfixed,
    Frightened, Drowsy, Webbed, the list data on the effect, each id checked at load; refused, no
    time or ki, with none of them to end, "Your mind is already still." (greyed so in the menu);
    any grip), Consecrate (the priest's at
    9, the paladin's at 12: a minute's prayer, for 15 mana, that lifts an unholy special room's
    effect, or a whole level's, for good, §7 "Special rooms"; refused, no time, elsewhere; works
    for Kind or better), Knock (the wizard's at 4: in a direction, for 5 mana and a cast's time, the lock
    of the locked door or shut chest there springs open and nothing opens; refused, no time or
    mana, where nothing locked stands; §8 "Chests"), Merchant's eye (the merchant's at 12, #816:
    on a stack carried, `use:merchants_eye:c`, chosen from the identify list; it is identified as
    the scroll of identify would identify it, for the time of a `read` and nothing else, no mana,
    cooldown, check, or limit; refused, no time, for one already known), Scribe (the wizard's
    at 12, #906: `scribe:SPELL`, chosen from the scribe list, a scroll of a spell known written
    on a blank scroll for three times its mana and a study's time; §8 "Scrolls": "Writing
    scrolls").
  - **Passives**, each one read: Weapon drill (melee weapon skill marks +25 %, `practise`),
    Shield line (Shield +1 in a shield style, the styles' role since #1093), Cleaving strike (the cleave free blow; with the Cleave talent too, great
    cleave), Armour drill (worn armour's Dexterity cap +1, Protection +1 while wearing any),
    Weapon mastery (threat +1 with melee weapon skills at rank 8 or more), Iron stance (the
    aggressive stance's Dodge penalty and the defensive's to-hit penalty gone), Champion
    (critical multiplier +1 with melee weapons, at most ×5); Battle-scarred (Protection +1, and
    +2 more while raging: +3 in all, in any armour), Headlong (while raging, steps 20 % cheaper,
    in any armour and any burden, `World::step_price`), Brutal critical (critical multiplier +1
    with a two-handed weapon), Hardened (each level past `hp_level` adds the level's hit points
    again and 3 more), Mighty thews, Blood frenzy (while raging, melee attacks 20 % cheaper,
    `World::attack_price`; and a kill builds 5 rage), Mighty rage
    (Raging's raises doubled, 10 more rage at most, Exhausted eases twice as fast), Unstoppable
    (while raging bleeding and poison tick nothing, and a blow that would take the last hit point
    leaves it for 10 rage, while that much is held); Divine grace
    (disease checks +2, poison eases twice as fast), Purity of body (no
    disease is ever caught: `World::expose` refuses, nothing rolled), Retribution (the paladin's
    at 25, #912: a hostile beside them whose melee attack lands a blow takes their smite back at
    once, free, for 4 zeal; works for Kind or better, a band-gated passive; §8 "Free blows"), Holy wrath (the paladin's at 40, #913: every melee blow, shot, or throw of
    theirs that hits an unholy target rolls 1d6 more, after the blow's own damage, of its kind,
    resisted but never soaked, for no zeal; no bands), Bulwark (Protection +2), Avatar (Lay on
    hands heals fully, cures every status that ticks and every disease, for half its zeal: 3; and
    Holy wrath's die lands on every foe, not only the unholy); Martial arts (bare hands and
    kicks roll 1d6; grants Ki, §8 "Resources"), Flurry (level 1 beside it: a melee attack
    with nothing in hand strikes twice, the second at −2, for a point of ki; with none, once),
    Step of the wind's passive half (level 6: steps 10 % cheaper in no body armour and with no
    shield; it took Fleet's place, #847),
    Still mind (+2 on checks made with Willpower: diseases', noticing
    in passing; no status lowers Intelligence or Willpower; its active is Stillness, above), Diamond soul (Deflection +2; and, #851, a shot or throw, a
    trap's missile too, that would hit the ascetic while they hold no weapon and no shield
    spends a point of ki by itself and misses, the missile falling on their cell: "You pluck
    the arrow from the air.", said after the attack's own words; with no ki, the hit stands;
    spells and bursts are never caught; no action), Perfect self (bare hands 1d8; unarmed threat +1, critical multiplier +1; ki comes back
    twice as fast). The paladin's
    passives do not ask for Good; only the gifts used from the menu do. No berserker row reads
    what the berserker wears (#843).
  - **While a status lasts** (#843): any talent effect the talents' one sum reads may say
    `while = "STATUS"` (a status id, checked at load); it counts only while its holder suffers
    that status (`Talents::mods`, given the actor's statuses; every rule keeps its one read).
    Battle-scarred's second half, Headlong, and Blood frenzy's price say `while = "raging"`. An
    active, a spell given, or what is rolled or handed down when a talent is taken carries no
    condition (refused at load). The identical-effects check counts the condition as part of
    the effect: `protection 2 while raging` is not Bulwark's `protection 2`.
  - **Resources**: Rage, War cry, Blood frenzy, Mighty rage, and Unstoppable run on the Rage resource
    (§8 "Resources", R1); Martial arts grants the Ki resource, which Flurry, Step of the wind, Stunning strike,
    Patient defence, Stillness, Diamond soul's catch, and Quivering palm spend (R4, #252; #846, #847,
    #848, #849, #850, #851).
- **The casters' class tables, and their spells** (`content/spell_def.rs`: `SpellId`;
  `data/spells/`; `world/cast.rs`, `world/caster.rs`; M30, #245, #395; design §4.9–4.12, §3.7):
  the priest, druid, wizard,
  and witch have level tables as the others do (bonus picks at 10, 20, 30, 45, 55: priest Vigour
  or Grit, druid Wayfarer or Vigour, wizard Scholar, witch Scholar or Grit).
  - **Spells**: the casters' actives are **spells** (§8 "Spells"; the placeholders of #245,
    migrated by #395; the guaranteed lists of #397): a class ability's talent effect
    `{ kind = "spell", spell = "ID" }` gives the spell of that id in `data/spells/` (each
    given by one talent, which several classes may share: Blessing, Mend, Barkskin), cast by
    the one cast path: level, cost, reach, dice, and resolution are the spell's. A full caster
    has a cantrip at level 1 and about four spells in all; the bard, the paladin, and the
    ranger a cantrip or a boon (§8 "Spells", "Guaranteed spells").
    `Ability::Spell` in the menu's own **Spells** group (`Source::Spell`, after Class) with the
    roll it makes now, its damage type, and what it costs the player, as the character screen
    shows a weapon's ("4d6 fire, 26 mana, aimed", "3 mana" for one that rolls nothing; short of
    mana "4d6 fire, 26 mana: you have 14", `ability::spell_unavailable`; the owner's one
    exception to hidden numbers, the spell scaling design decision 11), its description line
    adding its reach now and a reroll ("Range 11, radius 2. Your 1s are rolled again."),
    `Action::Cast`
    (`use:blessing`, `use:arcane_bolt:12,5`, the replay text unchanged). They **cost mana
    instead of a cooldown** (spec §8 "Mana"); refused, no time and no mana, without the spell,
    without the cell, out of range, with no hostile seen there to hex, nothing to cure, not
    hurt, or (Call lightning) under a roof or below ground. Their kills are the caster's, and
    one that kills its caster is `Cause::Spell` ("Killed by their own fireball"). Too close a
    burst asks first, as a scroll's does ("Cast it anyway?").
  - **Priest** (the priest design, `docs/design/priest.md`; #883): Mend (level 1, at 1: a
    touch, 1d3 back; 3 mana), Blessing (level 3, at 3: Blessed 10 min a degree: +1 to hit,
    Willpower +1; 4 mana), Cure wounds (level 3, at 3: a touch, 2d6; 2 casts; 3 mana),
    Consecrate (9), Communion with the dark (9: Consecrate's dark twin, a passive working only
    for a Callous or worse priest: in an unholy place, a room's or a whole level's, their prayers
    add no strain and the place's status, Ebbing, never touches them), and **the prayer** (`use:orison` at every tier; `docs/dev/prayer.md`; the wiki's Prayer page states its numbers): one
    class active given at 1 that answers what ails the priest most, in a fixed priority order
    (disease, poison, wounds, release, mana, hunger, restoration; `data/prayer.toml`), as far as
    its uses reach: wounds at 1 (Orison: bleeding stops, hit points to full), hunger at 6 (Daily
    bread: Hungry or worse fed to Satiated, with Purify's rule: rotten food never sickens nor
    exposes), disease and poison at 12 (Prayer: every disease, the brewing ones with the shown,
    and poison), release at 18 (Prayer of release: hexes, holds, webs), restoration at 25
    (Litany: stat damage and drained points, the temple's mending), mana at 32 (Litany of the
    mind), two answers a prayer at 40 (Benediction), and at 60 everything at once with every
    resource pool filled (Miracle). It takes a cast's time and no mana; Raging and Silenced
    block it. A hidden **strain** limits it: +100 an attempt, answered or not, a point passing a
    minute; a prayer fizzles with 10 % per 100 strain, and a fizzle locks it until the strain
    has passed to 0 (the same words for both). With nothing to answer (a brewing disease alone is
    nothing) it is refused, with no time or strain. The band at the moment picks who answers:
    Neutral and better the holy powers (Orison, Prayer, Litany, Benediction, Miracle), Callous
    and worse the dark ones (Whisper, Invocation, Rite, Pact, Miracle), each with its own words.
    The dark ones answer with a twist (`docs/dev/prayer.md` "The dark face"): the diseases cured
    pass to the nearest creature the priest sees (incubating, unchecked), the poison cured is
    laid on it at its degrees, a released hex is laid on its caster if seen (a hold's holder is
    Webbed), and the hunger use feeds to one short of Bloated. **Sanctuary** (50; `use:sanctuary`, #885): the
    priest kneels a minute for 30 mana (interrupted as a rest is, nothing is laid) and the
    ground within two cells of where they knelt is hallowed for an hour: no creature hostile to
    them steps onto it from outside (one already on it may move about and leave; nothing spawns
    there; missiles and spells cross it), and whoever stands on it heals and regains mana three
    times as fast (or a special room's pace, the larger). A blow, shot, throw, or harmful spell
    of the priest's from on it breaks it ("The peace is broken."), and so does a dark twist that
    reaches a creature; being struck does not. The
    same rule for both faces, the words differing ("The ground about you is hallowed." / "A
    dread settles on the ground about you."); no light. Refused, with no time or mana, on the
    overworld, in water, in a shop, or on hallowed ground. Remove disease, Purify, Purity of body, Smite, the
    reprieve, and Saint left the table (the spell and its scroll stay).
  - **Druid**: Beast-friend (the Wayfarer talent, given; its copy Wild empathy until #790), Spark (level 1, at 1: a bolt, 1d4 electricity;
    4 mana), Barkskin (level 3, at 3: Dodge +2 a minute a degree; 4 mana), Nature's ward
    (wading never sickens; poison eases twice as fast), Call lightning (at 12; level 11: a line of
    8 cells, 2d6 electricity, its ladder Willpower 13: 3d6 to 25: 5d6+2, a thunderclap; open
    sky only; 20 mana), Flame and frost (two spells, Flame: level 12, the fireball's
    burst, 3d6 fire, radius 2, blast, 29 mana; Frost: level 6, 2d6 cold, freezes water, 14
    mana). #791 cut Woodland stride (6; the ranger's 18 too) and Winter's friend (25), which
    worked only outdoors; #839 fills 6, 25, 32, 40, and 60 with the calls (§8 "Allies"): Call of
    the wild (at 6; level 5: a wolf for 5 minutes, Presence 14: 8, 18: 12, 22: 16; 5 mana), Call
    of the forest (at 25; level 20: a bear; 20 mana), Pack leader (32: each call brings one more,
    keeps two of each, and lasts half again as long), Call the elements (at 40; level 35: a
    greater elemental of earth, water, air, or fire, one drawn; 35 mana), and Lord of the wild
    (60: calls last until slain, three of each). Each call keeps one of its own at once until
    Pack leader; its scroll is very hard to learn (`study_difficulty` 8, 10, 12).
  - **Wizard**: Scroll adept (reading −50 %; the spell scrolls read a rung higher, scrolls only:
    the spell scaling design, decision 12),
    Ray of frost (level 1, at 1: a ray that rolls to hit, 1d6 cold; 4 mana), Burning hands
    (level 1, at 1, #979: a cone of 3 cells, no hit roll, 1d4 fire, its ladder Intelligence 13:
    1d6 to 22: 2d4+2; 6 mana, the cone price at the floor, #1264), Arcane training
    (at 1: Decipher script +10 since #1270, a `skill` effect: it helps only a character trained in it), Magic missile
    (level 3, at 3: never misses, 1d4 force, its ladder Intelligence 13: 2d4 to 22: 3d4+1; 4
    mana), Arcane bolt (level 6, at
    18: a line of 6 cells, 2d6 force, its ladder to 3d6+4; 11 mana), Scribe (at 12: a scroll of
    a spell known written on a blank scroll, three times its mana; §8 "Scrolls"), Mage armour (Deflection +2 in no body armour),
    Empower (`spell_mod { rungs = 1 }`: its spells a rung higher on their ladders; spells' mana
    −10 %), Arcane mastery (mana +100 %), Archmage (two rungs; spells' mana −50 %, at least 1).
    The rungs never pass a ladder's last; nothing holds the roll after (the spell scaling
    design, decisions 5 and 16). The wizard's bonus talents
    come from the Scholar and Spellcraft trees.
  - **Witch**: Herb lore (rotten food −50 %, a corpse's `on_eat` harm and vomiting never, the
    angelica root heals 1d6), Evil eye (level 0, at 1: Hexed 1 min a degree: −2 to hit; 4 mana),
    Grave breath (level 1, at 1, #979: a cone of 3 cells, no hit roll, 1d4 necrotic, its ladder
    Willpower 13: 1d6 to 22: 2d4+2; 6 mana, the cone price at the floor, #1264),
    Hex of weakness (level 3, at 3: Weakened 2 min a degree: St −2, Dx −2; 4 mana), Cackle (at
    6, #904: `use:cackle`, nothing to aim, the time of a cast, no mana or cooldown; a voice 60
    loud, `Act::Cackle`; every effect on a creature the player's perception sees whose status a
    hex lays, `Content::hex_statuses()`, and whose giver is the player grows a degree, to its
    max, and starts its degree afresh, in actor-id order; refused with no time when no creature
    in sight is under a hex of hers, "No one you see is under your hex.", and while a status
    blocks a cast; "You cackle." "Your hexes hold."; no draws, no marks), Poison lore
    (poison eases twice as fast; +2 damage on hits and spells at a Poisoned creature,
    `World::lore_blow`, from `resolve` and `spell_strikes` before resistance),
    Cause wounds (level 4, at 12: a touch that rolls to hit, 1d6 necrotic, to 2d6; 3 mana),
    Malison (at 18, #905: +3 damage on hits and spells at a creature under each hex it is under,
    anyone's, `status_damage` on the five hex statuses; and the tithe, `hex_tithe`: a creature
    that dies under a hex of the player's, by any cause and anyone's hand, anywhere, gives the
    player its level / 2 in mana, at least 1, never past their maximum, `World::hex_tithe` from
    `dies`; "The hex comes home." when any came; never for the player's own death; no draws),
    Second sight (noticing
    as Spot +2, searching +2), Familiar's sight (#932: her familiar's sight and nose, §8 "Allies":
    "The familiar"; `TalentEffect::FamiliarSight` read in `Perception::compute`), Curse of sloth (level 3: Sluggish 1 min a degree: every action
    costs 150 %; 3 mana), Blight (level 7: a burst of 2d6 poison, radius 2, and two degrees of
    Poisoned on everyone it harms; 17 mana), Cauldron (a corpse's harm never; its stat marks and widening chances doubled; a fresh one heals 1d4),
    Hag's bargain.
  - **Reprieves** (`reprieve`, `World::reprieve` from `hurt`, after Rise from the ashes): once
    in a life (`Progress::once`), what would kill the player (not starving or choking) leaves
    them at 1 hit point, Worn thin a day (hit points and mana back half as fast), and for the
    Hag's bargain Presence −2 for good; "A miracle! You do not fall."
  - **What the player sees**: the news of each level ("You gain Rage."); the actives in the
    abilities menu with their cooldowns, and the abilities in the dumps' `abilities` layer.
    Never a number the game hides: the wiki's class tables state the rules. **Saves change**
    (new status ids).
- **The ranger's, thief's, and bard's tables** (M30, #244; design §4.6–4.8, §4.1 B6): as the
  adventurers' above. Shared entries are the ordinary talents, handed out: Evasion (Finesse;
  the ranger's at 32, the thief's at 18). One the player
  picked before its level comes is not given again (a pick comes in its place). The bard's Fast
  healer (Vigour) and the druid's Beast-friend (Wayfarer) are shared the same way (#790).
  - **The kill tally** (the design's B6, `Quests::kills`): every kill of the player's is
    tallied by level and creature type, whatever the type (kill quests read the same tally);
    `Quests::prey` is the type killed most, the first to get there of those tied (it changes
    only when another is killed more often). Saved, hashed, left out while empty.
  - **Bonus picks**: ranger (Weaponcraft, Wayfarer) at 10, 20, 30, 45, 55; bard (any tree) at
    5, 10, 15, 23, 28 (#874); thief (Finesse, Shadow, Delver) at 4, 10, 16, 22, 28, 34, 46, 58.
  - **The bard's 30 levels** (#874; the bard design §5): its table is the old one halved, every
    ability by 30 (Mend and Trained wind 1, Travelled hands 3, Fast healer 6, Inspire 9, Jack of
    all trades 12, Throw your voice 16, Lullaby 20, Master of none 25, Virtuoso 30), and the
    class ends there, `max_level = 30` ("Prestige classes" above): experience past it counts and
    buys nothing until the Hunter's calling, when Hunter 1 comes at total level 31.
  - **Actives**: Sic 'em (the ranger's, 40, #921: `use:sic:X,Y`, §8 "Allies": "Sic 'em"), Volley (the ranger's, 50: `Action::UseAt`, `use:volley:X,Y`, aimed at a cell
    as a shot is; the readied missile weapon's three shots there, one after another, each its
    own flight and attack, for the price of one shot and one exertion; it stops when the
    ammunition runs out; it costs 6 Focus (§8 "Resources"; no cooldown: greyed out "not enough
    focus" while short, "You have too little focus." if used), spent as it is loosed and priced
    as one ordinary shot (Rapid shot does not touch it); refused as a shot is, spending
    nothing), Inspire
    (the bard's, 9: grants Inspiration, §8 "Resources"; a **song**, #870: Inspired, to hit +1
    and Dodge +1 for 5 minutes a degree, for 3 inspiration and nothing else (no mana fallback:
    each song costs one thing), the time of a `song` (500, a chat's); no cooldown, the cost is the
    price: greyed out "not enough inspiration", `Unavailable::Short`, while short; "You have
    too little inspiration." if used).
    **Songs** (`world/song.rs::sing`, the one place a song is sung; the `song` talent effect
    `{ id, on, status, severity, resource, cost | mana, check, difficulty }`, an active that
    `ClassActive::sings`): paid in their one cost, heard (`Act::Sing`, a voice of `SONG_NOISE`
    80 where the singer stands: those who hear it come to look), then the status, `severity`
    degrees plus Music's `song_degrees` (one more at 50 and at 100, then one more every 100), at
    most the status's `max` (Inspired and Inspired greatly are `max = 3`: the degrees are time,
    the bonus stays), then a Presence mark (inspired) and a Music mark (`song`). Refused, no time,
    on the overworld and while Silenced (`silenced` blocks the `song` action). A song `on =
    "hearers"` is sung over every creature that hears it (the Sing noise's hearers on ground
    truth, asked before it is made: an unseen foe in earshot is checked like any other) and is
    hostile to the singer; friends, allies, and the peaceful only hear it. Each, in actor-id
    order, checks `check` (its modifier and its talents' checks of that stat) against
    `difficulty` plus Music's `song_craft` (+4 at 100, +8 at 200), two gameplay draws apiece;
    failed, it suffers the song's status at the song's degrees, caused by the singer. "You sing a
    lullaby.", then for each seen "The kobold's eyes grow heavy." or "The kobold shakes off the
    song."; nothing of the unseen. Sung to an empty room all the same (no refusal: who hears is
    the world's). The **Lullaby** (the bard's, 20, #873: `use:lullaby`, nothing to aim): Drowsy
    (everything costs 300 %, a degree easing every 12 s) to hostile hearers who fail Willpower
    against 13 plus the craft, for 6 mana and the time of a song, no cooldown; greyed out "you
    have too little mana" while short. Music is
    `kind = "rules"` (no "not usable yet" row). Throw your voice (the bard's, 16) is no
    longer everyone's: the ability is had by a talent with the `throw_voice` effect (the
    action is refused, "You do not know how to throw your voice.", without it). The thief's
    Lurk (6, #684: `use:lurk`, the time of a wait, no cost and no cooldown: the thief suffers
    Lurking, "You wait in ambush.", until their next action; a hostile that steps up beside them
    meanwhile takes their blow at once, the gatekeeper's free blow, and is caught unawares by it
    unless it was already hunting them, so the blow is resolved as on an unaware target: the
    initiative check, the sneak attack dice, flat-footed; refused with no time, "Too late to
    lurk.", with a hostile they see beside them) and Low blow (12, #684: `use:low_blow:E`, 3
    openings, the time of a melee attack: a blow at the hostile that way that is a sneak attack
    whatever it knows, its dice on a hit, the foe not Surprised and keeping its Defence; none on
    a thing without vitals, "It has no vital spot."; greyed out "not enough openings" while
    short; refused with no time, "There is no one there to blind.", without a hostile there).
    Trapfinding, Pick lock, and Light fingers, which only gave what the class's skills give,
    are gone from the table (#684); a save holding one loads without it (said in the log).
  - **Passives**, each one read: Familiar (the witch's level 1, #929: a small beast of her race's
    list, or rarely a rare one, at her heel from the first turn, whose touch lays her evil eye and
    which goes for what she has hexed, §8 "Allies": "The familiar"; `TalentEffect::Familiar` read
    by `World::bring_familiar`; with it the active Call familiar, #930, `use:call_familiar`: an
    hour's sitting brings another when hers is dead, the imp among the draw from her level 30), Animal companion (the ranger's level 1, #916: a beast of the
    race's kind, or rarely a rare one, at their side from the first turn, that grows with their
    level and always comes along, §8 "Allies": "The animal companion"; `TalentEffect::Companion`
    read by `World::bring_companion`; and `tame = 2`, `TalentMods::tame`, the ranger's edge at
    taming, read by the Tame skill ability, `world/tame.rs`), Pack tactics (the ranger's level
    18, #919: a creature beside both the ranger and their companion is flanked, the ranger's
    melee blows and the companion's at it +2 to hit, never a shot, a throw, or a spell; and the
    companion's attacks at the favoured prey carry the ranger's Favoured prey, Master hunter's
    half too; `TalentEffect::{PackTactics, SharesFavouredPrey}`, one hook in `World::resolve`,
    `world/pack.rs`), Favoured prey (+1 to hit, +2 damage against `Quests::prey`, any
    attack, in `World::resolve`), Stalker's eye (the ranger's at 2, #788: it sees which
    creatures in view have not noticed it, `sees_unaware`, above), Tracker (hears 3 fainter; every sound it hears placed at its
    own cell, `noise::Exact::All`), Rapid shot (grants Focus, §8 "Resources"; a shot loosed
    with `fire` while the ranger holds 2 focus spends them and takes 10 % less time, the whole
    shot's price; with less it is an ordinary shot; `world/focus.rs`), Hunter's bond (the
    ranger's at 25, #920: shared sight and shared buffs with the companion, §8 "Allies";
    `TalentEffect::SharedSight` read in `Perception::compute`, `TalentEffect::SharedBuffs` in
    `World::afflict`, `world/bond.rs`), Master hunter (bows and crossbows threat +1; Favoured
    prey +1 and +2 more); Sneak attack (#682: the class passive beside the Sneak attack skill,
    whose dice the thief trains, `2d6+10` and known for: it grants the Openings pool from level
    1, and those the thief catches unawares make their initiative check 2 harder,
    `surprise_difficulty`; §8 "Surprise and sneak attacks"), Pack mule and Porter (the
    burden line's first two steps, #1198), Improved sneak attack (+2d6 on top of the skill's: dice of the same sides
    add up), Vanishing act (40, #684: a sneak attack of the thief's that kills leaves them
    Light-footed, a degree, `sneak_kill_status`, read where `Build::SneakKill` is raised in
    `World::resolve`; `world/thief.rs`), Hide in plain sight (no footsteps while Unburdened;
    attacks half as loud), Master thief (an attack on an unaware creature threatens a critical
    2 lower, never below 3; and its finisher, an active aimed in a direction, `use:master_thief:E`,
    for 5 openings: a melee blow that is a critical whenever it hits, with no confirmation roll,
    `Profile::sure_critical`, for a melee attack's time; `world/openings.rs`); Travelled hands
    (the bard's at 3, in Blessing's place, #871: a hand-out once, `weapon_ranks { to = 1 }`:
    every weapon skill and fighting style the bard holds at rank 0, whatever rows the content
    has, but the hidden styles, rises to rank 1 by
    `WeaponSkills::raise_to`; "Your hands know every weapon a little.";
    the optional `skills` names the rows instead, for a later class's trade); Fast healer
    (the Vigour talent, given at 6; its copy Song of rest until #790: resting heals half again
    as fast), Jack of all trades (12, #872: a hand-out once,
    `take_up_skills { dice = "1d4+1", except = ["sneak_attack"] }`: every skill of the content
    the bard is not trained in, but Sneak attack, trained at 1d4+1 from the `levels` stream in
    file order, `world/jack.rs`, "You find you know a little of everything."), Master of none (25: `trained_skills`, every
    trained skill counts 5 higher, as a skill bonus in `Actor::skill_effect`), Virtuoso (30, #874: every song costs nothing and gives a
    degree more; Inspire gives Inspired greatly, to hit +2 and Dodge +2 for 10 minutes, for
    nothing: neither mana nor inspiration).
  - **Resources**: the ranger's Focus (R2, #250), the thief's Openings (R3, #251), and the
    bard's Inspiration (R4, #252) are in.
  - **Saves change**: `Quests::prey`, the tally of every type, the statuses `inspired` and
    `inspired_greatly`.
- **Weapon skills** (`warlock-core/src/weapon_skills.rs`, training; `content/weapon_skill.rs`,
  the data; M22, #134), ADOM-like:
  - **Skills** (`data/weapon_skills.toml`, §7): the categories weapons belong to. A skill's
    role says what uses it: `melee` and `launcher` skills are named by weapons (`category`), the
    wielded weapon's for a blow, the readied missile weapon's for a shot; `thrown` is every
    throw, whatever is thrown; `unarmed` bare hands, natural attacks, and kicks, whatever is
    wielded (§8 "Doors"); `shield_bash` a held shield's bashes (there is no skill of blocking with
    a shield: the shield's own Defence and the shield styles' `shield` are the Shield category,
    #1093). The schools of magic are no skills (#1263): a spell's school is its category (§8
    "Spells"), and a spell trains only the caster style it is cast in ("Fighting styles" below).
  - **Ranks**: 0 (untrained: no benefits; marks still count) to 10, on every actor
    (`Actor::weapon_skills`, a rank and marks per skill): the player's from race and class, a
    creature's from its type (§7). A skill's rank comes only from its own training, starting
    ranks, talents, and the dev `rank:` action: no skill raises another (#1188 removed *The
    Riddle of Steel*'s defaults).
  - **Marks**: only the player trains. Each attack the player makes on someone (a blow, a bash,
    a kick, a shot or throw for each actor it reaches; hit or miss) earns `attempt` marks (1) in its
    skill, a hit `hit` more (2); a blow the player's shield blocks trains no weapon skill. The
    marks scale
    by `100 + per_level × (the opponent's level − the rank)` percent (20 a level), kept within 0
    and 100, rounded to the nearest mark (halves up): fewer against weaker opponents and as the
    rank rises, never more than the full marks against stronger ones (#1095: monster levels run
    to 70, ranks to 10, so a cap above 100 % paid nearly double from the middle levels on). The opponent's level is its creature type's `level`
    (§8 "Experience"; the one attacked): a level-1 giant rat
    teaches nothing past rank 5, a level-4 kobold shaman nothing past rank 8.
    A missile that reaches no one earns nothing; nothing rolls for marks.
  - **Curve**: the marks to go from a rank to the next, by the skill's `curve` (#1095): melee
    15, 45, 190, 500, 1750, 4000, 17 500, 38 000, 68 000, 88 000 (rank 10 at 218 000 in all),
    steep from rank 7 so that ranks 7 to 10 take most of a career (a fighter of the monsters of
    their level: rank 6 about level 10, 8 about 25, 10 about 45; docs/balance.md "Weapon pace");
    missile two thirds of it, 10, 30, … 58 700; the caster styles, trained by far fewer casts
    than blows, `caster`: 30, 60, 120, … 1920 doubling, then 3600, 6000, 9000 (#1251: three
    times what it first asked; rank 1 ten casts that land on a foe of the rank's level or more,
    rank 4 150, rank 10 7470; docs/balance.md "Caster style pace"). Marks
    past the need count toward the next rank (one rank at a time); rank 10 earns no more.
  - **Target dummies** (#1199; `world/dummy.rs`): a creature type with a `training` table
    (`{ percent, melee, ranged, breaks }`, §7) is a dummy, the straw men in the Weyfarers' Gilde's
    yard in Cēaptūn (§3). It never acts and never dies (its hit points never drop), so it gives
    no experience, kill, loot, or quest credit; it is of the peaceful folk, but the player
    strikes it by moving into it as they would a hostile. A blow, bash, kick, shot, or throw at
    it is never a surprise or sneak attack, makes no noise of its own, angers and disturbs no
    one, and trains no stat: only the attack's weapon skill and style, at `percent` (50) of the
    marks a creature of the dummy's `level` (1) would give, rounded, and only below rank `melee`
    (2) for blows, bashes, kicks, and throws, or `ranged` (4) for shots. A skill or style that
    reaches that rank by a dummy's marks is told "You have learned all the dummy can teach
    you."; from it on a blow there trains it no further. No daily cap: time is the price.
    Missiles at a dummy: §8 "Ranged combat", Coming down.
  - **Benefits** (per skill and rank, data; never shown): to-hit and damage (to every roll of
    the dice) on attacks with the skill, a wider threat range (lower natural rolls threaten,
    never below 2), and a lower cost (`speed`, percent off `melee`, `shoot`, or `throw`;
    `reload` off reloading), all in the attack's `Profile` (`Profile::with`; `Actor::melee`,
    `ranged`, `thrown`, `bash`) and price (`World::attack_price`); Parry added to the wielded
    weapon's own while it is wielded, Shield (a shield style's, "Fighting styles") added to a held
    shield's own (`Actor::defence`). Benefits of several skills add up (`Benefit::plus`), so a later kind of
    skill (fighting styles, #135) can stack on a weapon's: another role, with the same ranks,
    marks, curve, and benefits. Ranks 1 to 5 grow slowly, ranks 6 to 10 much faster: bigger
    damage, a wider threat, more speed and Parry, and moderate to-hit (#554; a long-blade
    fighter's rank 10 is +7 to hit and +8 damage, a great blade's +5 and +12; the wiki's
    tables have every number). Styles grow the same way.
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
  - **Messages**: a rank gained is said after the attack
    that earned it (and before the experience a kill earned), by its `phrase`: "You feel more
    skilled with long blades." No numbers.
  - **What the player perceives**: their own ranks and marks
    (`PerceptionView::own_weapon_skills`), shown as ranks and a bar on the weapon skills screen
    (§6) and in the `weapon_skills` dump layer (the caster styles among them; JSON
    `weapon_skills` and `caster_styles`); never a benefit
    as such (the character
    screen's attacks show the to-hit and damage the ranks add up to, §6) or a creature's ranks (the
    `actors` dev layer shows every actor's). Ranks and marks are part of the world state
    (saved, hashed).
- **Fighting styles** (#135; `content/weapon_skill.rs`: `Role::Style`, `Grip`, `Style`,
  `TwoWeapons`; `world/actor.rs`: `Actor::grip`, `style`, `styles_in_use`, `attack_skills`
  (`AttackSkills`: an attack's weapon skill and style), `attack_benefit`, `off_hand`,
  `off_hand_blow`), after AD&D 2e's *Complete Fighter's Handbook*: how weapons are
  held and used, as skills with the weapon skills' ranks (0 to 10), marks, curves, and
  benefits ("Weapon skills" above), in `data/weapon_skills.toml` (§7).
  - **Configurations** (`grip`): for blows, what the hands hold (`Actor::grip`, which counts for
    Defence too): nothing (`unarmed`), one one-handed weapon and every other hand empty
    (`one_weapon`), a weapon in each of two hands (`two_weapons`), a two-handed weapon
    (`two_handed`), a one-handed weapon and a shield (`weapon_and_shield`), a shield in each hand
    (`two_shields`, #611); any other hold (a shield alone) is no style. A shot is `launcher` (whatever the hands hold), a throw `thrown`.
    A kick (§8 "Doors") is made whatever the hands hold: it draws on unarmed fighting alone and
    is fought in no style, so Brawler gives it nothing and its price is a kick's (decided: a
    style is how the hands are used).
    A style may name the weapon skills it is fought with; the style of a grip that names none
    takes every other weapon (`WeaponSkillTable::style_for`), so one configuration is always at
    most one style. **The spell decides the caster style** (#571, #1251; `world/cast_style.rs`):
    any spell cast in **armour that hinders casting** (anything worn with a
    `spell_cost_percent`, §8 "Spells") is cast as a Battle caster (`armoured_spell`); out of such
    armour, a spell that **rolls to hit** (a damage spell resolved by the attack roll) is cast as a
    Toucher (`touch_spell`) when its shape is `touch` and as a Sniper (`ranged_spell`) otherwise
    (a ray, a bolt), wherever it is aimed; every other spell (an area, a bolt that never misses, a
    heal, a buff, a hex, a touch heal) is cast in no style, and neither trains nor gains from
    Sniper or Toucher. The style is decided once a cast, by the player's and the creatures' alike,
    so the style is the cast's `AttackSkills` (no weapon skill) for the whole cast: its hooks (the
    style's benefit after the talents' and items', `Actor::spell_mods_in_style`), a rolling
    spell's to-hit and triggers, and its marks. Every spell paid in mana trains the style it was
    cast in, lands or not, as every blow trains its style (`World::practise_cast`, at the end of
    the cast, by the `Outcome` its kind filled through `World::spell_landed`: an attempt, and a
    success when it landed, a harm on someone, a hex that took, a heal of a point or more, a
    buff, a cure, or an identify done, weighed by the strongest creature it struck at, else the
    caster's own level; a scroll read or a wand used trains nothing). Caster styles name no
    weapons. What the menu and the aiming mode show is what is cast. A spell that rolls to hit
    aims with its stat, the caster style, talents, and items. A **touch** reaches the cell beside
    its caster and no farther: nothing gives it a cell (School focus, Sniper, an item).
  - **The twelve** (data): **Duelist** (`one_weapon`: to-hit, Parry), **Twin Blades**
    (`two_weapons`: eases both two-weapon penalties, none left from rank 9; from rank 7 to-hit too), **Great Weapon**
    (`two_handed`, any weapon but a staff: damage, from rank 8 a critical rolls its damage once
    more, at rank 10 twice more (×4), cheaper blows), **Sword and Board** (`weapon_and_shield`: Shield, from rank 3 a chance
    of a shield bash after a blow that hits), **Two Shields** (`two_shields`, hidden: see "Two
    shields" below), **Marksman** (`launcher`, bows, crossbows, and
    slings: to-hit, cheaper shots and reloading), **Skirmisher** (`thrown`, anything thrown:
    to-hit, damage, cheaper throws; decided as throwing only: "a light weapon and light armour"
    would have overlapped Duelist and Twin Blades, and the issue's benefits are all throws'),
    **Brawler** (`unarmed`: to-hit and damage), **Staff Fighter** (`two_handed`, staves: to-hit,
    Parry); and the caster styles (#571): **Sniper** (`ranged_spell`: to-hit, and against a
    target `far` (5) cells or more away more to-hit and a wider threat range,
    `[spell_attacks]`; `cells`, a cell farther from rank 5 and two from rank 9), **Toucher** (`touch_spell`: to-hit, threat, a critical's damage up to twice
    more; `bonus`, a point on the roll of every touch cast so, from rank 4, up to 3 at rank 10);
    Sniper and Toucher teach no abilities (#1251),
    **Battle caster** (`armoured_spell`: to-hit, and `armour`, percent off what worn armour adds
    to every spell's mana cost, read by `Actor::armour_spell_cost` whatever is cast, up to half
    at rank 10; rank 4 Spell guard; full marks on every spell cast in such armour). `cells` is
    allowed for `ranged_spell` only and `bonus` for `touch_spell` only among the styles, `speed`
    for no caster grip. `own_styles_in_use` marks Battle caster while the armour worn hinders
    casting (a standing fact), never Sniper or Toucher (decided cast by cast). The
    classes that cast start in one: the wizard and the druid Sniper 1, the witch and the priest
    Toucher 1, the paladin Battle caster 1; a ranger advances by casting.
    Defaults: duelist ↔ sword and board −3, duelist ↔ twin blades −3, great weapon ↔ staff
    fighter −3, marksman ↔ skirmisher −4, sniper ↔ toucher −4, toucher ↔ battle caster −3;
    never to a weapon skill.
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
  - **Two shields** (#611; `world/two_shields.rs`, docs/dev/weapon-skills.md): any shield fits
    either hand (`Content::fits_properly`: an object with `bash`, like a light weapon; a shield
    is put in its own off hand first, `equipment::free_slot`). With a shield in each hand every
    plain melee attack (the player's move into a hostile, a creature's blow) is a shield bash with
    the first shield in hand order, made `with` it, drawing on and training shield bashing and
    Two Shields (`Actor::melee_skill` is shield bashing in the grip), priced as a blow with it
    (`Actor::blow_item`); after one that hits, the style's `bash` chance brings a bash with the
    other shield (`World::second_blow`, one draw only when the chance is above 0). Banes
    (`[two_shields]`: −6 to hit on every bash in the grip, the Shield bash ability's too; every
    attack 50 % dearer, after the stance; a spell the player casts paid in their own mana fails
    75 % of the time, decided before it goes off, its mana and its time spent, "Your shields get
    in the way, and the spell slips from your grasp.", one draw only when the chance is above 0;
    a scroll or a wand never fails so) shrink with the style's `ease_bash`, `ease_slow`, and
    `ease_spell` to none at rank 9 (`Actor::shield_banes`). Boons: the style's Shield, damage,
    and the follow-up chance; Staggering block (rank 4) names it beside Sword and Board. The
    style is `hidden` (§7): no class or race gives it, and it is not
    listed (the weapon skills screen, the `weapon_skills` dump layer and its JSON, the Talents
    page for a talent that asks for it, Stylist's choices) until its owner has marks in it
    (`WeaponSkills::shows`).
  - **Abilities**: a style's ranks teach abilities as weapon skills' do ("Weapon abilities"
    below), working only in the style's grip.
  - **What the player perceives**: their own ranks and marks (as weapon skills), and which
    styles they fight in now (`PerceptionView::own_styles_in_use`), on the weapon skills screen
    (§6) and in the `weapon_skills` dump layer (`Duelist 1 (0%), in use`; JSON `styles_in_use`).
    Never a benefit, a penalty, a chance, or a creature's ranks.
- **Weapon abilities** (#561; the combat progression design §4, §6;
  `content/weapon_ability.rs`, `world/weapon_ability.rs`, docs/dev/weapon-abilities.md): what a
  rank in a weapon skill or a style teaches, one `[[weapon_ability]]` a file in
  `data/weapon_abilities/` (§7; a campaign's may add more), naming its skills (weapon skills or
  styles, never both kinds) and the rank (1 to 10, per file, so each group and style sets its own).
  - **Who has one**: an actor whose rank in one of its skills, as abilities read it
    (`Actor::ability_rank`: a weapon talent's ranks, Stylist and Fencing, count, and Early
    mastery's one more), reaches it: the player and every creature with the rank alike
    (creatures' ranks come from their types). It works only through what the actor fights with:
    a triggered ability fires for the skill and style of the attack concerned (the attack's for
    the attacker, what it holds for the one attacked); a style's only in the style's grip.
  - **Triggered**: free and instant, with no stamina, time, or cooldown (refused in data),
    each on one combat event, raised at one line that already marks it (`World::weapon_trigger`,
    the one place they fire): `hit` and `critical` (a blow, shot, or throw of the actor's that
    lands or is a confirmed critical), `big_hit` (a third of the target's hit points, the target
    alive), `kill`, `sneak_attack` (a sneak attack of the actor's that hits, its sneak dice
    landing: "Surprise and sneak attacks"), `struck`; a melee attack on the actor missed by its Parry (`parried`, at
    the attack's end), its Shield (`blocked`), or its Dexterity or Dodge (`dodged`);
    `both_hands_hit` (both blows of a two-weapon attack hit one living target, once an attack;
    a shield bash is no second hand); `approaches` (a hostile steps up beside the actor, the
    player's step too); `spell_blocked` (a hostile's spell that rolls an attack against the
    actor's Defence, a bolt, a ray, or a touch, a scroll's or a wand's too, missed by its
    Shield). The school abilities' `spell_lands` trigger was removed with them (#1263). Deciding
    never draws: a fight with no qualifying rank keeps its draws.
  - **Effects** (closed): `free_blow` (switches a free blow on while the actor wields a weapon
    of the skill or fights in the style: Brace's gatekeeper, Riposte, Cleaving sweep, Great
    cleave; `World::free_blows_of` folds them in, so a riposte from the style and one from a
    talent are one riposte), `status` (on the other end or the actor; degrees fixed or drawn
    once), `extra_damage` (Rend: the main hand's dice once more, no bonus, resisted, never
    soaked, after the blows), `free_shot` (Point-blank: a free shot with the missile weapon and
    the ammunition ready, else a throw of what is ready to throw; nothing without, no draw),
    `parry_missiles` (Measure: Parry counts against shots and throws, which it otherwise does
    only for a shot from beside its target, "Combat" §8c), `reflect_spell` (Spell reflection: the blocked spell is cast again
    from the actor's cell at its caster, as the caster cast it, its roll, rerolls,
    to-hit, and riders, rolling its own attack against the caster's Defence; no mana spent
    again, nothing for the actor; at most `per_turn` (1) a turn of the actor's, and a reflected
    spell is never reflected again. Bursts, cones, lines, and hexes roll no attack and are never
    reflected, nor a miss by Parry or Dexterity. A caster's rays and bolts are turned back at it
    the same way, §8 "Creature behavior": "Casters").
  - **The catalogue** (data; ranks first guesses): Deep cut (blades 5: a big hit bleeds a
    degree more), Crushing blow (axes, maces, polearms 5: a big hit staggers), Brace (spears,
    staves 5), Point-blank (bows, crossbows, slings, thrown 5), Stunning bash (shield bashing 5:
    a bash that hits staggers), Riposte (Duelist 4), Measure (Duelist 10), Rend (Twin Blades 4),
    Cleaving sweep (Great Weapon 4) and Great cleave (10), Staggering block (Sword and Board 4: a
    parried or blocked melee attack staggers its striker) and Spell reflection (10), Pinning shot (Marksman 4: a critical
    shot pins 1d3 actions), Stunning fist (Brawler 4: a critical staggers), Hit and run
    (Skirmisher 4: a throw that hits makes the thrower Light-footed), Trip (Staff Fighter 4: a
    critical pins an action), Knife in the dark (Daggers 5: a sneak attack that hits with a
    dagger leaves the foe bleeding a degree) and Hunter's shot (Marksman 6: a sneak attack that
    hits with a shot pins the foe 1d2 actions); the caster styles' (#571), fired by a spell that rolls to hit:
    Spell guard alone (Battle caster 4: a spell that lands wards its caster, Spell ward, a Dodge point until
    their next action). Staggered, Pinned, Spell ward, and Light-footed ease by the sufferer's actions
    (§8 "Status effects": `per_action`), so a staggered foe's next action is the slowed one,
    however many blows staggered it.
  - **What the player perceives**: a rank that teaches one says so with its own line ("You feel
    more skilled with long blades. You have learned the deep cut."); the abilities menu (§6)
    lists each under *Weapon skills* after the skill it is listed under ("Long blades: Deep
    cut"), greyed "fires by itself", never usable (`Ability::Weapon`,
    `Unavailable::FiresByItself`); Rend says "Your blades rend the orc.", Staggering block "The
    orc staggers from your shield.", Spell reflection "Your shield turns the scorching ray back
    at the shaman." ("The champion's shield turns your scorching ray back at you."), a free blow or shot as one is. Never a rank threshold, a
    chance, or a creature's abilities. The dev action `rank:SKILL:N` sets a rank (§6, replays).
  - **Actives** (#564; the design §4.5, §4.6; `world/weapon_active.rs`): used from the abilities
    menu (`Ability::Weapon`, `Source::WeaponSkill`; never a key of their own), aimed as `aim`
    says (`nothing`, `direction`, `cell`, the existing `Aim`s), as `use:lunge:E`
    (`Action::Act`; the direction in upper case, as a stack's letter is in lower case) or
    `use:shield_wall`, `use:piercing_shot:12,5` (`Action::Cast` of an id that is no spell's:
    resolved after the class actives, the birth sign gifts, and the spells; a loader check keeps
    an active's id off theirs). Usable only while what the actor holds is for it
    (`Actor::holds_for`: the abilities its blow's, or a shot's, weapon skill and style teach),
    else greyed `Unavailable::NotSoHeld` ("your hands are
    not so held"); short of its stamina, `Unavailable::Short` ("you are short of breath", the
    resource's `lacking` words); a status that blocks the action it is, `While`. **Costs**, both
    paid only after its own refusals (nobody hostile there, no room to lunge, nothing to shoot):
    its `stamina` through `World::spend_resource`, less Drillmaster's point and Armour-breaker's
    2 for Sunder and Gap shot (`TalentMods::weapon_stamina`, `ability_stamina`; at least 1);
    and its `energy`, `percent` of the price of the action it is, priced before it
    is made: `attack_price` for a blow (weapon speed, style, stance, statuses: Light-footed
    halves it, Walled adds its quarter), `shot_price` for a shot, `wait` for Shield wall;
    Practised technique cuts a quarter (`weapon_energy_percent`, never below 50 % of the
    action). An optional per-ability `cooldown` status exists in data; none is used. Effects:
    `reach_blow` (Lunge: the first hostile within `reach` along the direction, the cells between
    empty and walkable, nothing sprung; a foe beyond the first cell brings no free blows),
    `sure_critical` (Vital strike), `ignore_defence` (False lead: rolled without the target's Parry and
    Shield, read in `resolve` through the transient `Technique`), `ignore_protection` (Sunder,
    Gap shot: a penetration above any Protection), `sweep` (Whirl, Staff sweep: a blow at each
    hostile beside, clockwise from the north or the direction given, −2 each, the blades taking
    turns with two), `pierce_shot` (Piercing shot: a hit flies on to the next on the line, once),
    `knockback` (Shield rush, a bash; Hip throw: a foe hit is moved a cell along the blow if it
    is free, as a teleport's check, no draw, then Staggered; into a wall it stays, unstaggered,
    and the blow lands), `buff` (Shield wall: Walled 5 degrees on the actor, Shield +3 on top of
    the shield's, attacks at 125 %, a degree a turn), `double_dice` (Mighty swing: the damage dice
    twice, bonus and riders once, −2 to hit). The catalogue (data; first guesses): Vital strike
    (blades 9; 4 stamina, a blow's time), Sunder (axes, maces, polearms 9; 4, 125 %), Lunge
    (spears, staves 9; 3, 100 %), Piercing shot (bows, crossbows, slings 9; 4, a shot's), Shield
    wall (Sword and Board, Two Shields 9; 5, a wait's), False lead (Duelist 7; 2, 100 %), Whirl (Twin Blades 7; 6,
    150 %), Mighty swing (Great Weapon 7; 5, 150 %), Shield rush (Sword and Board 7; 4, 125 %),
    Gap shot (Marksman 7; 4, 150 % of a shot), Hip throw (Brawler 7; 4, 125 %), Staff sweep
    (Staff Fighter 7; 5, 150 %). Each is its own action (no stacking of actives on one blow);
    regeneration bounds loops. The first rank that teaches an active grants stamina to a
    character without it (`World::ranks_news`: "You have learned the lunge. You learn to pace
    your breath."). Creatures use no actives. The menu row shows the stamina cost ("3 stamina,
    in a direction"), never the time; an active's blow says its blow's own message, a knockback
    "The orc is driven back."

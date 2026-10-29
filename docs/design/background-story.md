# Warlock — Background story: a youth told in sentences, and what it left you

Design document for the owner's review (2026-10-28), revised with the owner's decisions the next day (§18). Written against `origin/main` at 6e7c9e6b (Berserker stance, #690), then against 6082d382 (the heritage talents Family wealth and Father's sword, #697 merged as #700; #671's optional class skills before it). Nothing here is implemented; §16 cuts it into issues. It builds on what is there: `CharacterSpec::start` and `World::start_run` (`docs/dev/character-creation.md`: the rolls, the kit, the names, the sign), the creation picks and granted talents (`docs/dev/progression.md`, `world/creation.rs`, `Actor::boons`), skills and their starts (`docs/dev/skills.md`), scores and potentials (`docs/dev/stats.md`, `Stats::gift`), the starting alignment (`docs/dev/alignment.md`), the screen stack and the info pages (`docs/dev/screens.md`), saves and replays (`docs/dev/saves.md`, `Replay`), and a run's campaign (`docs/dev/campaign.md`). Every number is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-10-28, verbatim): *"feature: add adom type of randomly generated background story. Basically we have list of different sentence pools that the story is build from. Each sentence rolled might have effect on stats, or starting gear. There should be chances to roll rare events to the background, that give more possible boons. The random sentences need to take the player choises in chracter creation into account. the story is presented after player has started teh run, and cannot go back to make choices."*

## Contents

1. Summary
2. The owner's request, ADOM's background, and what exists today
3. The shape: eight chapters, one line each, tags between them
4. The data: `data/background/`
5. The roll: streams, tiers, the floor, determinism
6. Effects: what they touch, and the order they apply
7. Rare events and banes: the tiers and their numbers
8. What the player sees: the story screen, page 8, dumps, what leaks
9. Who gets a story
10. Talents and class abilities: the `background` system
11. The pools: chapters, sizes, and real lines
12. Three sample stories
13. Determinism, saves, replays
14. The wiki, the spec, and the docs
15. Choices made along the way, and what is rejected
16. Implementation plan
17. Open questions, each with a recommended default (answered: §18)
18. Decisions (owner, 2026-10-29)

## 1. Summary

| Piece | Proposal |
|---|---|
| **The story** | eight short chapters, each one sentence drawn from a pool: **Parents**, **Birth**, **Childhood**, **Youth**, **The trial**, **Fortune**, **Leaving**, **The road** (the last is the campaign's own). Together, four paragraphs in the second person, past tense: "You were born to farming folk …". §3, §11 |
| **A line** | an entry in a chapter's file with `text`, a `tier` (common, uncommon, rare, fateful), a `weight`, `when` (conditions on the race, gender, class, birth animal and element, a lucky day, the starting alignment band, the creation talents, the trained skills, the age, and the **tags** earlier lines set), `effects` (stat points, gold, an item as a kit line, a skill start, weapon-skill marks, an alignment nudge, a talent or a flaw), `tags` it sets, and `set` (words later lines use: `{guardian}`, `{home}`, `{trade}`). §4 |
| **The roll** | one named stream per chapter, `background/<race>,<gender>,<class>/<chapter>`; per chapter: draw the tier, keep the lines of it that fit, draw one by weight, roll its dice. No existing creation stream moves; the same seed and spec always tell the same story; the same spec on another seed tells another. §5 |
| **Rare events** | per chapter: **uncommon 12 %** (a boon, perhaps with a cost), **rare 3 %** (net +3 to +5 points), **fateful 0.5 %** (net +5 to +8: a talent, a magic heirloom, a second stat). Over a story of eight: about one character in four has a rare line or better, **one in twenty-five a fateful one**. Rare and fateful lines are always net-positive (checked at load); banes live in common lines and are capped. §7 |
| **No caps, one floor** | nothing limits what a story gives or takes in all (decision 2): the tiers' odds are the only rarity, and a line's worth is bounded only by its tier's band. The one floor is the game's own: a score is never below 1 (`Stats::gift` keeps 1..=40). Points per effect kind (a stat point 2, a skill start 1, a magic item 3, a talent 4 …) exist only to check a line's tier at load and to show the wiki. §5.3 |
| **Order** | the story is told **after** the kit, the class's level-1 grants, and the creation picks (Random picks included), so it can read every choice, and no pick can be met by a gift fate gives later. Its stat changes move the score and the potential together (`Stats::gift`, the class gift's rule); its items join the pack as kit lines do (`Pack::put`/`equip`); a magic heirloom is identified. §6 |
| **The heritage talents (#700)** | Family wealth **forces** the Parents chapter to its "old money" lines, which tell the wealth and give no gold (the talent gave it); Father's sword **excludes** the heirloom lines and tells of the blade instead. The talents bias the pools; nothing is paid twice. §6.2 |
| **#671** | a skill the class's optional roll gave is a condition a line may read (`skills = ["heal"]`: "you had the healer's hands already"); a story skill start on a skill already trained raises it (`Skills::add`), as the optional roll does. §6.3 |
| **Presentation** | a read-only screen **"Your story"** over the run's first turn, opened by the app the moment a run begins from the summary (or `--character`/`--new` at a window), never on Continue, never under `--capture`/`--replay`; then **page 8** of the info pages (`Page(8)` exists already: no new `KeyCommand`), beside Talents; `--screen story`. Words only: never a number the game hides. §8 |
| **Who** | every run that starts with the kit (New run, `--character`, `--name`, `--talents`, a replay with `kit`): a chosen character has a life. `--scenario` alone (the scenario's own character, tests, golden walks) has none, so those hashes hold. The web build needs nothing: all of it is core and content. §9 |
| **Saved state** | `Character::story: Vec<Told { chapter, line }>` (ids; the text is re-told from the content), left out while empty: old saves and replays load; no new replay field; `FORMAT_VERSION` untouched. A story talent is an `Actor::boons` entry (the altar's machinery). §13 |
| **Talents** | the `background` system with **Favoured child** (Heritage, creation-only: the luckier tiers twice as likely), **Hard-won** (Heritage, creation-only: a stat your story cost you comes back sooner: three training marks per point lost), and **Storyteller** (Scholar, Presence 12: shop favour a little better, more for a tale with rare turns). No class ability (`min_class` is 0; §10 names where one would go). §10 |
| **Slices** | five issues in three waves: B1 the data and the roll (save), then B2 the screen and B3 the pools and the wiki together, then B4 the talents and B5 the fateful effects. One PR adds saved state; none adds an action. The implementers write the pools (decision 13). §16 |

**Key choices, and why**

1. *One sentence per chapter, chapters in a fixed order, tags between them.* ADOM's stories read well because each sentence answers the one before; a bag of unordered sentences reads like a horoscope column. Tags (`orphan`, `smith`, `bully`) and three word slots (`{guardian}`, `{home}`, `{trade}`) are the whole memory; nothing else is needed for sentences to agree, and a loader check keeps the vocabulary closed.
2. *Tiers are drawn first, then the line.* The owner asked for rare events with numbers; drawing the tier as its own step makes the rarity a number in one place (`[tiers]`), the same in every chapter, and visible in the wiki, instead of an accident of weights across pools of different sizes.
3. *No caps on a story, and no protection from a bad one* (decisions 2 and 7). The first draft capped a story's boons and banes and bounded each stat; the owner struck the caps: a story is the sum of its lines, however they fall, and the only floor is that a score is never below 1. What stays is per line: a tier's band (a rare line is net-positive and carries no bane) is checked at load, so rarity still means good news, and a grim story is simply a grim story.
4. *The story comes after every choice, and after the picks.* Fate speaks last. That is the owner's "take the player choices into account" and "cannot go back" in one rule: the story reads the race, the sign, the class, the talents (the Random picks too), the optional skills; and because prerequisites are checked at the pick and never again, a bane cannot un-take a talent and a boon cannot have been used to take one.
5. *Effects reuse what exists.* Items are kit lines (`content/kit.rs`, alternatives and `magic` included), stat changes are `Stats::gift`, skill starts are `Skills::add` (#671's path), talents are boons (`Actor::boons`, the forgotten altar's), the alignment nudge is the start's fourth term beside the animal's. No new effect machinery; one application function per kind.
6. *The text is the game's narration, in the plain English of the march's day.* The story is the game's voice, not a commoner's, so it is not Cockney; quoted speech in a line takes its speaker's register (a posh aunt, a Cockney pedlar); old places keep their Old English names, and only the campaign's own chapter names them. "Fate", never "the seed".

## 2. The owner's request, ADOM's background, and what exists today

**ADOM's background** (Ancient Domains of Mystery; the "Background" shown on the character sheet, generated at creation). ADOM composes a few paragraphs of the character's youth from sentence pools, one sentence a part: the **parents** and their station, the **birth** (the star sign's month, and a circumstance), the **childhood** (what kind of child), the **youth** (a trade, a teacher, a temple), and **why they left** (ambition, a quarrel, a calling). Some sentences carry a small effect: a childhood line moves a stat a point or two ("as a child you were very weak" or "the strongest child in the village"), a youth line starts a skill (the temple years give Healing, a scholar's youth Literacy), the parents' wealth sets the starting money, and the sign sentence is the horoscope's. Players remember three things about it: that it read like a life and not a table; that a few lines mattered (the Literacy and the money) and most did not, which made the ones that did feel like luck; and that nothing in it could be chosen, only re-rolled by starting over. The community's own proposals for new backgrounds (the Chaos Knight thread on the ADOM blog, 2012) all reach for the same shape: a life in stages, each line with a stat, a purse, or a skill in brackets, and one closing line that hands the character to the road.

What this design takes from it: the stages, one sentence each, in order; effects small and in words; the sign's place in the birth paragraph; "nothing chosen". What it adds, because the owner asked: **rare events with real boons** and explicit odds; sentences that **read the choices** (ADOM's did not read the class beyond the youth line); the story told **after the run starts**, so it is fate, not a reroll; and the house rules (the wiki exact, the game vague; no caps and no protection: unbalanced is fine).

**What exists today** (the pieces the story is built from):

| Part | Today |
|---|---|
| The creation order | `CharacterSpec::start(content, seed, today)`: the birth (`character/birth/<race>`), the sign, the stat dice (`character/rolls/<spec>`: race, class, animal, element, race potentials), the skills, the optional skills (#671, `character/optional-skills/<spec>`), the weapon skills, the name, the kit (`character/kit/<spec>`). Then `World::begin`: the pack (the kit and the scenario's `carry`), the known scrolls and potions of the kit, the actor, `stars::grant` (the race's and the sign's granted talents), `class::grant(1)`, hit points and mana to their maxima, the welcome lines, `open_creation` (the picks due); then `start_run`: `take_creation` (the spec's picks, else the Random picks on `character/talents/<spec>`), mana again, perception |
| Streams | every creation draw has its own stream (`rng.rs`'s table); adding one moves nothing. The convention is `character/<what>/<spec>` |
| Stats | `Stats::gift(stat, by)` raises the score and the potential together, kept in 1..=40 (a class ability's `stat` gift; the altar's Gift of the forgotten); `effective_stats` is what rules read; the character screen shows final scores only |
| Items | `content/kit.rs`: a kit line is a string or a table (`carry`/`wield`/`wear`, alternatives `"club\|quarterstaff"`, `trained`, `gender`, `magic = "any"\|"magic"\|"rare"`), checked at load; `kit::roll` draws `one_of` and `magic::loot::make` at `kit::ITEM_LEVEL`; `Pack::put` joins stackables, `Pack::equip` finds a free slot |
| Skills | `Skills::add(id, by)`: trained at the roll if untrained, raised if trained, capped at `MAX_VALUE`; #671's optional skills use it. Weapon skills carry `marks` toward the next rank |
| Talents given, not picked | `World::bestow_talent` (the altar's Chosen) gives a talent at rank 1 with what it hands out and records it in `Actor::boons`, which `stars::grant` puts back on every restore; `bestow_flaw` the same for a sign's `flaw` talent |
| Creation-only talents (#700) | `needs = { creation = true }` (`Need::Creation`: offered at the creation picks, refused at a level in words); the `heritage` tree (`no_free`) holds Family wealth (`starting_gold`, `starting_gems`) and Father's sword (`heirloom`: the race's weapon, 2 to 4 affixes, identified), handed out once as they are taken (`world/heirloom.rs`) |
| Alignment | `Start::alignment` = the race's + the class's + the animal's nudge, set on the actor in `begin`; `shift_alignment` is the one place a score changes **in play** |
| Granted talents and the class | `class::grant` gives what the table names and the player lacks; a talent already held (a creation pick, a boon) is simply not given again |
| Screens | one stack (`ui/screens.rs`); the info pages `PAGES` (Character … Talents, seven) switched by `KeyCommand::Page(1..9)`, which exist for all nine numbers; a read-only text screen exists in `ui/sign_screen.rs` (the reveal: a panel of wrapped paragraphs, Enter and Esc from the menu context, no rows) |
| Saves | `Character` is saved and hashed inside `WorldState`; a field with `#[serde(default, skip_serializing_if)]` loads from old saves and leaves old hashes alone while unset |
| Replays | `{scenario, seed, character {race, gender, class, talents}, name, kit, steps}`: a run is a function of these |
| The campaign's own data | `assets/campaigns/<id>/data/` holds the campaign's dialogue and quests; shared data is `assets/data/`; `WorldState::campaign` names a run's |
| Names | `data/names.toml`, by race and gender; the register memo: Old English for the old places, Middle for the guild's age, the English of the day for the people's speech, Cockney for the commons and posh for the better off |

**The gap**: a character has a name, a birthday, a sign, scores, and a pack, and no life between the birth and the pass. The summary says what was chosen; nothing says what happened.

## 3. The shape: eight chapters, one line each, tags between them

A story is eight **chapters** in a fixed order, each contributing **one line** (a sentence or two), shown as **four paragraphs**:

| # | Chapter | id | Paragraph | What it tells | What it tends to give |
|---|---|---|---|---|---|
| 1 | Parents | `parents` | 1 | who raised you and how they lived | `{guardian}`, `{home}`, `{trade}`; tags `orphan`, `poor`, `wealthy`, `smith`, `trader`, `hunter` …; a little gold, a trade's marks |
| 2 | Birth | `birth` | 1 | the circumstances of the birth (the sign already said *when*; this says *how*) | a stat point traded for another; a lucky day acknowledged |
| 3 | Childhood | `childhood` | 2 | what kind of child | a stat point (the chapter where most stat points live) |
| 4 | Youth | `youth` | 2 | a trade, a teacher, a levy, a temple, a wood | a skill start, weapon-skill marks (the class-conditioned chapter) |
| 5 | The trial | `trial` | 3 | the thing that cost you | banes with something kept; the grim rare lines |
| 6 | Fortune | `fortune` | 3 | the thing that was given you | the boon chapter: gold, an heirloom, a blessing, a talent |
| 7 | Leaving | `leaving` | 4 | why you left | an alignment nudge, a last gift from home |
| 8 | The road | `road` | 4 | how you came to the run's first level | the **campaign's own** chapter (`assets/campaigns/<id>/data/background/`): the Ascmearc's tells the pass; a campaign without one has a seven-chapter story |

**Tags** are the story's memory: a line may `tags = ["orphan"]`, and a later line may ask `when = { tags = ["orphan"] }` or `not_tags = ["orphan"]`. The vocabulary is closed: the loader refuses a `when.tags` id no earlier chapter's line sets (a tag set in Youth cannot be asked in Birth). **Word slots** are the other half: a line may `set = { guardian = "the sisters", home = "the minster" }`, and a later line's text may use `{guardian}`, `{home}`, `{trade}`; the chapter settings give every slot a default (`your family`, `home`, `your trade`), so an unset slot still reads. The fixed slots are `{name}`, `{they}`, `{them}`, `{their}` (the gender's pronouns; "other" is they), `{guardian}`, `{home}`, `{trade}`; the loader refuses any other `{…}` in a text. Capitalised forms (`{They}`) for a sentence start.

A chapter always yields a line: every chapter has at least one **plain line** (common, no conditions, no effects: "You worked, as everyone worked, and nothing of it is worth the telling."), checked at load, so a pool emptied by conditions still tells something. The plain lines are weighted so that a common story has two or three of them: a life is mostly ordinary.

## 4. The data: `data/background/`

`data/background.toml` holds the settings; `data/background/NN-<chapter>.toml`, one file per chapter in file-name order (as `data/classes/`); a campaign's `data/background/NN-<chapter>.toml` adds its chapters after the shared ones (a campaign may not redefine a shared chapter's id). Ids are global; a line's id is unique within its chapter and written `chapter/line` where one is named (saves, dumps, the wiki).

```toml
# data/background.toml
# The tiers, per mille, drawn first for every chapter (§5). A talent may scale the three
# luckier ones (Favoured child); common takes what is left.
[tiers]
uncommon = 120
rare = 30
fateful = 5

# What an effect is worth, in points: only for the tier check of each line at load (§7) and
# the wiki's tables. No story total is capped (decision 2).
[points]
stat = 2            # per point of score, boon or bane
skill = 1           # a skill start or raise, whatever the dice
weapon_marks = 1    # marks in a weapon skill, whatever the count
gold_per = 25       # one point per 25 coins of the dice's mean, rounded up (a bane when taken)
item = 1            # a plain item line
magic_any = 2       # a kit line with magic = "any"
magic = 3           # magic = "magic"
magic_rare = 5      # magic = "rare"
talent = 4          # a talent granted
flaw = 3            # a flaw granted (a bane)
alignment = 0       # nudges are free (they are the story's opinion of you)
```

```toml
# data/background/03-childhood.toml
[chapter]
id = "childhood"
name = "Childhood"
paragraph = 2
# Slot defaults for texts of this chapter and later ones that use a slot nothing set.
defaults = { guardian = "your family", home = "home", trade = "your trade" }

[[line]]
id = "strong_child"
tier = "common"
weight = 3
text = "You were a strong child, broad for your age, and carried what the others could not."
effects = [{ stat = { strength = 1 } }]
tags = ["strong"]

[[line]]
id = "sickly_child"
tier = "common"
weight = 2
text = "You were sickly as a child, and {guardian} sat up many a night with you; you lived, and learned to wait."
when = { not_tags = ["orphan"] }
effects = [{ stat = { toughness = -1 } }, { stat = { willpower = 1 } }]
tags = ["sickly"]

[[line]]
id = "wolf_in_the_yard"
tier = "rare"
weight = 1
text = "A winter wolf came into the yard when you were six, and you faced it with a stick until the grown folk came. The story grew; so did you."
when = { age = { min = 10 } }
effects = [{ stat = { toughness = 1 } }, { stat = { willpower = 1 } }]
tags = ["brave"]

[[line]]
id = "ordinary_child"
tier = "common"
weight = 5
text = "You were a child like any other, and nobody wrote down what you did."
```

**A line's fields**:

| Field | Meaning |
|---|---|
| `id`, `tier`, `weight` | the id; `common`, `uncommon`, `rare`, or `fateful`; the weight among the fitting lines of its tier (1 to 20) |
| `text` | the sentence(s), with slots; second person, past tense |
| `when` | all must hold: `race`, `class`, `gender`, `animal`, `element` (lists of ids; any of them), `lucky = true`, `band = { min, max }` (the **starting** band, before the story's own nudges: `alignment::Requirement`), `talents` (every one held: the creation picks, Random ones included, and the class's level-1 grants), `not_talents`, `skills` (every one trained, #671's included), `tags`, `not_tags`, `age = { min, max }` (the age at the start: a kobold of seven had no seven years' apprenticeship; an elf of two hundred had), `campaign` (ids; for a shared chapter's line that names a world) |
| `only` | `true`: when this line fits, the chapter's pool is the fitting `only` lines alone (the forcing the heritage talents use, §6.2). Among several, the weights decide |
| `effects` | a list, each one of: `stat = { strength = 1 }` (−2 to 2 per stat per line); `gold = "4d6"` or `gold = -20` (dice or a number; negative takes, never below 0); `item = <kit line>` (the kit syntax exactly, alternatives and `magic` included; a `magic` item is **identified**); `skill = { heal = "2d4" }`; `weapon_marks = { spears = 3 }` (marks toward the next rank, never a rank; `best = N` for the skill held highest, ties by data order); `alignment = 25` (−100 to 100); `talent = "hardy"` (rank 1, with what it hands out; never a `granted`, class-tree, or `flaw` talent, and never one the character holds; a talent with a `choice` names it, `talent = "weapon_focus:long_blades"`); `flaw = "slight"` (a sign's flaw talent, as the altar's Forsaken) |
| `tags`, `set` | what later lines may read |

**Loader checks** (`content/background.rs`, after races, classes, skills, talents, objects, and the horoscope load): ids; every `when` id exists (a talent in `when.talents` must exist, so a line that names #697's ids waits for #697); every tag asked is set by an earlier chapter; every slot used is known; every kit line parses as kits do (`object::parse_stacks`, a free slot); stat changes within ±2 a line; a tier's net points within its band (§7) and a rare or fateful line net-positive; every chapter has a plain line; `paragraph` 1 to 4 and non-decreasing in chapter order; a campaign's chapter ids are not shared ones. Content without `background.toml` tells no story and draws nothing (fixtures).

## 5. The roll: streams, tiers, the floor, determinism

### 5.1 Streams

One stream per chapter: `Rng::stream(seed, "background/<race>,<gender>,<class>/<chapter>")` (`Rng::background(seed, spec, chapter)`; `<spec>` is `CharacterSpec`'s `Display`, without the picks, as every creation stream). Nothing else draws from it; nothing existing draws from it, so **no roll of today's characters moves**: the birth, the sign, the scores, the skills, the kit, the name, and the Random picks are what they were. The spec is in the label so that a human warrior and a human thief of one seed, whose Youth pools differ, do not share a weight position either.

### 5.2 One chapter's draw

`background::tell(content, seed, &Witness) -> Story` is pure (`warlock-core/src/background.rs`, no `World`): `Witness` is what the conditions may read (the spec with its picks, the sign and the birth day, the age, the band, the trained skills, the talents held, the campaign). For each chapter in order:

1. **The tier**: one draw, `rng.below(1000)` against `[tiers]` (scaled by `TalentMods::story_luck`, §10): fateful, rare, uncommon, else common.
2. **The pool**: the chapter's lines of that tier whose `when` holds against the witness and the tags and slots so far. If an `only` line is among them, the pool is the `only` lines. Empty: the tier below, down to common, whose plain line always fits.
3. **The line**: one draw, `rng.below(sum of weights)`.
4. **Its dice**: gold dice, skill dice, `one_of` alternatives, a `magic` item's loot rolls, in effect order, from the same stream.
5. Its tags and slots are recorded; its text is not rendered until shown.

Two draws for a chapter without dice; a story is sixteen draws plus dice. The result is `Story { told: Vec<Told { chapter, line }>, effects: Vec<Rolled> }`: the ids (saved) and the rolled effects (applied once, §6, never saved: they are in the actor once applied).

### 5.3 No caps, one floor

The owner's decision (§18, 2): **a story is not capped**. The roller counts nothing across chapters: every line that fits is in the pool, whatever the lines before it gave or took, and a story's total is the sum of its eight lines. So eight kind lines may give six stat points and a purse, and eight grim ones may take four and the kit's coins; both are rare by the tiers and the weights, not by a rule. The one floor is the game's own: `Stats::gift` keeps a score in 1..=40, so a stat a story lowers stops at 1 (and its potential never below the score). Gold stops at 0 (nothing is owed). Nothing else bounds a story, and nothing protects a character from a bad one (decision 7): no re-roll, no pity line, no second draw.

What the first draft's `[points]` are still for: the per-line tier bands (§7), checked at load, so that a rare or fateful line is always net-positive, and the wiki's tables, which state each line's worth.

### 5.4 Determinism and "the same spec on another seed"

Given the content, the seed, and the witness, the story is fixed. The witness is itself a function of the seed and the spec (the age and sign from the birth stream, the band from the rolls, the skills from theirs, the picks from `character/talents/<spec>` when Random), so a replay's `{seed, character, name, kit}` reproduces the story without a new field. **The same spec on a different seed tells a different story**: yes, by design; the player who makes "a dwarf warrior" twice gets two lives. Two players with the same seed and spec get the same life, which they will never notice (a release build rolls the seed, #256).

## 6. Effects: what they touch, and the order they apply

### 6.1 Where in the start

`World::start_run` today: `begin` (the kit, the grants, the picks due) → `take_creation` → mana → perception. The story goes in **after `take_creation`**: `World::tell_background()` (`world/background.rs`) builds the witness from the finished character, calls `background::tell`, stores the ids in `Character::story`, and applies the rolled effects to the player, one function per kind:

| Effect | Applied by | Note |
|---|---|---|
| `stat` | `Stats::gift(stat, by)` on the player's stats | **the score and the potential move together** (decision 8): a +1 raises both by 1, so the ceiling moves with the gift; a −1 lowers both by 1, the potential never below the score. Kept in 1..=40: a score a story lowers stops at **1**, the one floor (decision 2). Hit points and mana are set to their maxima afterwards (a Toughness or mental point counts at once; the run has not begun) |
| `gold` | `Pack::put` of `gold_coin` stacks, or taking from them | never below 0: "lost everything" takes what the kit gave |
| `item` | the kit path: `KitItem` → `Pack::put`/`Pack::equip` on the player's inventory and equipment | a `wield`/`wear` line takes the first free slot, else is carried (as kits); a `magic` item is made by `magic::loot::make` at `kit::ITEM_LEVEL` from the chapter's stream and **identified** (`Magic::identify`): you grew up with it. The known scrolls and potions (`WorldState::known_scrolls`) gain the story's, as the kit's |
| `skill` | `Skills::add` | trained at the roll if untrained, raised if trained (#671's rule); a skill the class trains is simply higher |
| `weapon_marks` | `WeaponSkills::mark` by the count | never a rank; the bar on the weapon skills screen shows them |
| `alignment` | `player.alignment = Score::new(score + nudge)` | the start's fourth term after the race, the class, and the animal (`docs/dev/alignment.md` "set at the start" widens by one line); never `shift_alignment` (no diminishing, no band news: nothing has happened yet); the band the summary showed may differ from the one the character screen shows, which is the story's point |
| `talent` | `World::bestow_talent`'s body with the id fixed (`world/boons.rs`) | rank 1 with what it hands out, recorded in `Actor::boons`; the class table later gives nothing in its place if it names the same talent (as Evasion today); the Talents page marks it "(story)" |
| `flaw` | `World::bestow_flaw`'s body with the id fixed | a sign's flaw talent; "(story, flaw)" |

Then mana and hit points to their maxima, then the perception, as today. The welcome lines were pushed in `begin`; the story adds no message (the screen is its telling).

**Why after the picks**: the picks are the player's last choice and the owner's "take the player choices into account" includes them (and the Random picks, which are picks); prerequisites are checked at the pick and never again, so a story bane cannot un-take a talent, and a story boon cannot be used to meet one (Hardy wants Toughness 11; a story's +1 comes after the pick, so a character with Toughness 10 at the pick did not get Hardy by fate). The one consequence: a creation pick's prerequisites read the pre-story scores, which the summary shows; the character screen afterwards shows the post-story scores. The summary therefore says, in one dim line, "Fate will have its say when you set out." (§8.1; question 3).

**`World::draft`** (the Talents step's drafted run) tells no story: `draft` calls `begin`, not `start_run`, and `tell_background` is `start_run`'s. The step's offers are those of a character without a story, which is right: the story has not happened.

### 6.2 The heritage talents: Family wealth and Father's sword (#700)

The two creation-only talents (`family_wealth`, `fathers_sword`; the `heritage` tree, `needs = { creation = true }`) overlap with the Parents and Fortune chapters ("inheritance" lines). The rule: **the talents bias the pools; nothing is given twice.**

| Talent held | Parents | Fortune |
|---|---|---|
| Family wealth | the `only` lines `old_money`, `house_with_a_name`, `merchant_house` (common; `when = { talents = ["family_wealth"] }`): they tell the wealth ("You were born to money, and money was born to you …") and give **no gold** (the talent gave it); they set `wealthy` and `{home}` | the lines with `gold` are not excluded (fortune is fortune), but `the_bequest`'s `when` has `not_talents = ["family_wealth"]`: no second inheritance |
| Father's sword | — | the heirloom lines (`the_heirloom`, `the_named_blade`) have `not_talents = ["fathers_sword"]`; the `only` line `fathers_blade_told` (common, no effect) tells of the blade already in the pack: "The blade you carry was your father's, and his father's; the name on the hilt is older than the house." |
| Neither | the wealthy lines without the talent (`comfortable`, uncommon) give a modest purse (`gold = "4d6+10"`) | the heirloom lines may come (rare, §11): a magic weapon by fate for one character in thirty-odd, where the talent gives one to anyone who pays two picks |

So the talent is the sure way and the story the lucky way, and a character with both the talent and luck gets the talent's wealth and a story that says so, not two purses. The heritage talents are merged, so the lines' `talents` conditions name real ids from B1 on.

### 6.3 #671, the sign, the horoscope, the class grants

| Piece | Interaction |
|---|---|
| Optional class skills (#671) | a condition (`skills = ["heal"]`, rolled before the story) and a target (`skill` effects raise a trained skill); the optional roll keeps its own stream |
| The birth sign | conditions `animal`, `element`, `lucky`; the Birth chapter's `lucky_day` line acknowledges a lucky day ("born on a day the stars had marked"); a sign's gifts are granted before the story and may be read as talents (`talents = ["fleet"]`). The story never changes the sign, the birth date, or the age |
| The gender | `gender` conditions and the pronoun slots; the "other" stat is raised before the story (a condition could read it: not proposed) |
| The race | `race` conditions; the age condition does the rest (an elf's Youth is long, a kobold's short) |
| The class | `class` conditions (the Youth chapter is mostly these); the class's level-1 grants are held before the story and readable as `talents` |
| The kit | the story's items land on the kit's pack: a second dagger stacks, a second cloak is carried. `kit::tests::nobody_starts_strained_and_only_wealth_burdens` (which #700 widened to the heritage at its heaviest) gains the story's heaviest item lines: a content test, as the kits' is, not a rule of the roller |
| Mana | nothing: no story line touches mana (a stat point in a mental stat moves the maximum through the existing rule). Said here as the mana policy asks |
| Alignment | the start's fourth term (§6.1); the bands' widths (150 to a band at the middle) make a −25 nudge a quarter of a band: a story of three cold lines may drop a Neutral start to Callous, which is subtle enough to be a surprise and loud enough to matter |

## 7. Rare events and banes: the tiers and their numbers

| Tier | Odds per chapter | Net points (checked at load) | What it is | Per story of eight: at least one |
|---|---|---|---|---|
| **common** | 84.5 % | −4 to +2 (a line's banes at most 4, boons at most 2) | the plain lines, the small turns, the griefs with something kept | — |
| **uncommon** | 12 % | +1 to +3, banes allowed | a boon, perhaps at a price: a bequest, a stranger's ring, a hard winter that made you | 64 % |
| **rare** | 3 % | +3 to +5, no bane | a thing people tell: the wolf in the yard, the old soldier's winter, the heirloom under the bed | 22 % |
| **fateful** | 0.5 % | +5 to +8, no bane | once in a life: a talent and a stat, a named blade, a blessing that stuck | 3.9 % (one character in twenty-five) |

Expected lines per story: uncommon 0.96, rare 0.24, fateful 0.04; a rare line or better, 25 %. With Favoured child (the three luckier tiers doubled: 24 %, 6 %, 1 %): uncommon 87 %, rare 39 %, fateful 7.7 % (one in thirteen).

**"The rarer the better"**: the net bands rise by tier and a rare or fateful line carries no bane, so a rare event is always good news (the owner's "give more possible boons"), while a common line may be grim. **Banes** are real and **uncapped** (decision 2): a common line may cost a stat point, a purse, or a flaw, and the Trial chapter is mostly such lines; a story of eight such lines takes what it takes, down to the floor of 1 on a score and 0 coins. Nothing protects against it (decision 7). The Trial's lines are written to keep a skill or a point of Willpower from what they take, because a life that only takes is not a story: a writing rule (§11.9), not one of the roller.

The numbers are first guesses in data; the wiki states them, and `background::tests::tier_odds_hold_over_many_seeds` pins them within a margin so a pool edit cannot quietly change the rarity.

## 8. What the player sees

### 8.1 The story screen, and the summary's line

**"Your story"** (`ScreenName::Story`, `ui/story_screen.rs`): a read-only panel of the four paragraphs, 72 wide like the reveal, titled with the name ("Brakka's story"), wrapped with the panel's `wrap`, scrolled with ↑↓ when the console is short (the messages screen's way); footer `Enter continue` (the menu context's Confirm; Esc closes too; no new `KeyCommand`). It is opened by the app at the moment a run **begins**: `Game::new_run` (the summary's Start) and a windowed start that makes a new run (`--character`, `--name`, `--talents`, `--new`, with or without `--scenario`), over the run's first turn before any input; never on Continue or Resume, never under `--capture` or `--replay` (captures of the screen use `--screen story`), and never for a run without a story. It is the first thing the player reads after the summary's Start, and nothing on it can be chosen: there is no way back from it but into the run.

The summary (before the run) gains one dim line under the kit: **"Fate will have its say when you set out."** It leaks nothing (every chosen character has a story) and tells the player why the character screen's scores may differ from the summary's (§6.1). Question 3 offers the alternative.

```
┌ Brakka's story ────────────────────────────────────────────────────┐
│                                                                    │
│  You were born in the deep halls, where your father cut stone     │
│  and your mother counted it. You came in a hard winter, when      │
│  the river froze to its bed, and you came early and small.        │
│                                                                    │
│  You were stubborn as a mule and twice as hard to lead. At         │
│  fifteen you stood in the hold's levy with a borrowed spear,       │
│  and found you had a taste for it.                                 │
│                                                                    │
│  Your family had a feud, as families do, and you have a scar      │
│  across the ribs from the night it was settled. In the chest      │
│  under the bed, when your mother died, you found what no one      │
│  in the family had spoken of: a blade older than the hall, with   │
│  a name on the hilt.                                               │
│                                                                    │
│  You left because the stories were all somewhere else. At the     │
│  top of the pass an old woman sat by a fire that should not have  │
│  burned in that wind, and told you the ash march would make you   │
│  or unmake you. She gave you bread for the road.                   │
│                                                                    │
│  Enter continue                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### 8.2 Afterwards

| Where | What |
|---|---|
| **Page 8, "Story"** | the same screen as an info page: one entry in `PAGES` after Talents (`(ScreenName::Story, "Story", "Story")`), reached by `Page(8)` (bound to `8` by default, as the other numbers) and the tab line; the panel sized with `panel_width`. Esc returns to the run. For a character without a story (a scenario's own, an old save) the page says "Your story is not written." |
| The character screen | nothing new: the scores it shows are the post-story scores, as every rule reads them; the Stars line stays the sign's |
| The Talents page, the `talents` dump | a story talent is marked "(story)", a flaw "(story, flaw)", beside "(sign)" and "(race)" (`OwnTalent::source`) |
| The death screen | nothing new (the epitaph is the death's). Question 6 offers the alternative |
| The Characters screen | nothing: the summary file reads no save, and a story is not a line |
| Dumps | the layer **`story`**: the paragraphs as the screen shows them; the dev `actors` layer adds `story parents/deep_halls birth/hard_winter …` (the ids); JSON `story: [{ chapter, line, text }]` and `character.story` (ids); `--screen story` over a run (`--character` or a `--replay` with `kit`) |
| Messages | none: the story is told once, on its screen, and read again on its page. Its items are in the pack; a stat is a score |
| The wiki | the exact tables (§14) |

### 8.3 What leaks, and the words

The house rule: hidden numbers hidden in the game, exact in the wiki. A line's text says what happened and lets the player feel the effect where the game already shows it:

| Effect | The text may say | Never |
|---|---|---|
| a stat point | "strong", "sickly", "stubborn", "quick, and hard to catch" | "+1 Strength", "Toughness" by name |
| gold | "a purse", "her savings", "paid for the ride" | the count (the pack shows it) |
| an item | its name, as the pack will ("a knife", "a blade with a name on the hilt") | its affixes ("of accuracy"); the pack and the describe screen say those, identified |
| a skill | "the healer's hands", "learned your letters", "the price of everything" | the skill's value or name as a mechanic |
| weapon marks | "a taste for it" | "marks", a rank |
| a talent | what it does in the talent's own words, folded into the sentence ("you have never been sick a day since") | the talent's name (the Talents page says it) |
| alignment | the deed ("kept it, and have been half ashamed and half glad of it since") | the band, the number |
| a tier | nothing: a rare line reads as a story, not as a prize | "rare", "lucky" (the stars' lucky day is the one exception, already in the game's words) |
| the seed | nothing; "fate", "fortune", "the stars", "chance" | "seed", "rolled", "generated" |

`content::background::tests::texts_hide_the_numbers` greps every text for stat names, digits beside a `+`/`−`, and the words `seed`, `rare`, `tier`, `talent`, `bonus`, as the sign screen's test keeps "lucky" off the screen.

## 9. Who gets a story

| Run | Story | Why |
|---|---|---|
| New run (the summary's Start) | yes | a chosen character with the kit (`WorldState::kit`) |
| `--character`, `--name`, `--talents` (the game and the CLI) | yes | the same: they start with the kit |
| a replay with `kit = true` | yes, the same one | the story is a function of its fields |
| `--scenario` alone, `World::new`, `with_character`, tests, golden walks, old replays without `kit` | **no** | the scenario's own character with the scenario's pack: as kits and creation picks, so every existing hash and snapshot of such runs holds |
| a scenario's own character started with the kit (`--character` naming the scenario's own) | yes | it is a chosen character then |
| a run without a character (a scenario with `player_stats`) | no | no life to tell |
| the web build | as New run | all core and content; the page-hide save carries `Character::story` like any field; `cargo xtask web-smoke` presses Enter once more (the story screen) between the summary and the run, and `web/index.html` needs nothing |
| a release build without the test world | as New run | the Ascmearc's `road` chapter is the campaign's; the showcase's test chapter is `showcase` content and never ships |

A saved character from before this lands has no story (`Character::story` empty): page 8 says so; nothing is retold or rolled for it (a story told after the fact would move scores on a run in progress).

## 10. Talents and class abilities: the `background` system

A new `[[system]]` `background` ("Background story: the tiers, what a story costs you, telling it") in `data/talents.toml` (placed after `experience`, before `speed`, by topic: the character's making). Talents, one file each under `data/talents/`, one hook each. Two are **creation-only** (#700's `needs = { creation = true }`, in the `heritage` tree beside Family wealth and Father's sword): they must be, since the story is told at creation and nothing later can change it. The third is an ordinary pick that reads the story in play.

| Talent (tree, needs) | Effect, in the game's words | The rule | Hook (one read) |
|---|---|---|---|
| **Favoured child** (Heritage; creation-only) | "Fortune smiled on your youth: the luckier turns of your story come oftener." | the uncommon, rare, and fateful odds doubled (`[tiers]` × 2: 24 %, 6 %, 1 %) | `background::tell` reads `TalentMods::story_luck` (a percent, 200) when it scales the tier table: the one read, before the first chapter's draw |
| **Hard-won** (Heritage; creation-only) | "What your youth cost you, you win back: a score your story lowered comes back sooner." | every stat point a story line took is three **stat training marks** on that stat (`Actor::stat_marks`, the level and daily checks' reservoir: the point is likely back by level 2 or 3; the potential is where `gift` left it, so the rise is real) | `World::tell_background`'s `stat` arm, `by < 0`: `stat_marks[stat] += 3 × |by| × TalentMods::hard_won` |
| **Storyteller** (Scholar; Presence 12) | "You tell your tale wherever you go, and tell it well: shopkeepers warm to you, the more for a tale worth hearing." | shop favour +5 %, and +5 % more for each rare line of your story, +10 % for a fateful one (a story with no such line: +5 %) | `world::shop::favour` reads `TalentMods::storyteller` and `Character::story`'s tiers (`Story::tiers(content)`, the only read of the story in play) |

Creation-only talents are worth the two picks they cost to a player who likes a gamble (Favoured child) or a grim start (Hard-won: with it, the Trial chapter's banes become a plan); neither is a safe pick, which is the owner's taste ("garbage choices fine"). The system's bar: three talents, above `min_talents` 2. The owner approved them as designed while finding their text weak (decision 11: "they sound kinda shit but let's implement them"): the rules stand; the implementer of B4 may polish the names and descriptions, keeping the ids, and the names here are the design's own choice (decision 10). **No class ability**: `min_class` is 0 until every class has its table; if one is wanted later, the bard's table is the place (a "Ballad of yourself" at 12: Inspire's song is the bard's own story, +1 to its effect per rare line), noted for the class-tables pass and not proposed here. The wiki's by-system table gains the column by itself.

## 11. The pools: chapters, sizes, and real lines

**The implementers write the pools** (decision 13): the lines below are the register guide and a first draft, not finished data, and the counts are targets. Sizes to aim for in B3: **78 lines over eight chapters** (Parents 12, Birth 8, Childhood 10, Youth 12, Trial 10, Fortune 10, Leaving 10, The road 6), of which about 14 uncommon, 9 rare, 4 fateful, and more after; the Ascmearc's `road` chapter is the campaign's six; the showcase gets a two-line test chapter so the tests cover the campaign hook. The owner reviews and refines the texts once the whole wave is in, not line by line in the PRs. Below, the chapter's purpose, its conditions' spread, and real lines (tier, weight, `when`, effects, tags/set). Writing rules for the rest: §11.9.

### 11.1 Parents (12)

Who raised you. Sets `{guardian}`, `{home}`, `{trade}` and the tags later chapters lean on: `orphan`, `poor`, `wealthy`, `smith`, `trader`, `hunter`, `deep`, `warren`. Race- and class-conditioned so that a drow and a halfling do not share a childhood.

| id | Tier (w) | `when` | Text | Effects; tags; set |
|---|---|---|---|---|
| `farm_folk` | common (4) | — | "You were born to farming folk, who had little but gave what they had." | —; `poor`; home "the farm" |
| `smiths_child` | common (2) | not class aristocrat | "Your father was a smith, and the ring of his hammer was the first music you knew." | weapon_marks maces 3; `smith`; guardian "your father", trade "the forge" |
| `market_stall` | common (2) | — | "Your mother kept a stall on market days, and you learned early the price of everything." | skill appraise 2d4; `trader`; guardian "your mother", trade "the stall" |
| `foundling` | common (1) | — | "You were a foundling, left at the minster door in a basket of rushes; the sisters raised you, and never let you forget it." | skill heal 1d4, alignment +25; `orphan`; guardian "the sisters", home "the minster" |
| `woods_edge` | common (2) | race elf, half_elf, or class ranger, druid | "Your people kept to the wood's edge; your mother hunted, and your father cured what she brought home." | weapon_marks bows 3; `hunter`; home "the wood's edge" |
| `deep_halls` | common (3) | race dwarf, duergar | "You were born in the deep halls, where your father cut stone and your mother counted it." | —; `deep`; home "the deep halls" |
| `low_house` | common (3) | race drow | "You were born to a house of the under-dark, low in standing and lower in favour, where a child learned to listen before it learned to speak." | skill listen 2d4, alignment −25; `deep`; home "the house" |
| `the_warren` | common (3) | race kobold, goblin, ratfolk | "You were one of a litter of nine in a warren under the hill, and not the biggest." | stat dexterity +1, strength −1; `warren`, `runt`; home "the warren" |
| `comfortable` | uncommon (2) | not talents family_wealth | "Your parents were comfortable, as the town counted it: a house with two floors and a girl to do the fires." | gold 4d6+10; `wealthy`; home "the house" |
| `old_money` | common (1), **only** | talents family_wealth | "You were born to money, and money was born to you: the house had a name, and the name had a seat at the reeve's table." | —; `wealthy`; home "the house" |
| `nobodys_child` | common (1) | — | "Who your parents were, nobody told you, and you stopped asking." | —; `orphan`; guardian "nobody" |
| `plain_folk` | common (5) | — | "Your parents were plain folk who worked, ate, slept, and did not talk about it." | — (the plain line); home "home" |

### 11.2 Birth (8)

How, not when. Small stat trades; the lucky day acknowledged; the one rare line.

| id | Tier (w) | `when` | Text | Effects; tags |
|---|---|---|---|---|
| `hard_winter` | common (3) | — | "You came in a hard winter, when the river froze to its bed, and you came early and small." | stat toughness −1, willpower +1; `small` |
| `on_the_road` | common (2) | not tags deep | "You were born on the road, in the back of a cart between one town and the next, and have never since been easy in one place." | skill survival 1d4; `wanderer` |
| `the_twin` | common (1) | — | "You were born one of twins. The other did not live out the week, and {guardian} never spoke of it." | stat willpower +1, presence −1; `twin` |
| `with_the_caul` | uncommon (2) | — | "The midwife swore you were born with the caul, and would never drown; she sold it, after, to a bargeman." | skill swim 2d4; `caul` |
| `lucky_day` | common (6) | lucky = true | "You were born on a day the stars had marked, and the midwife, who knew such things, would take no fee." | —; `marked` |
| `storm_born` | rare (1) | — | "You were born in a night of thunder, and the old folk said the storm had come for you and been cheated." | stat willpower +1, toughness +1; `storm` |
| `named_for` | common (2) | not tags orphan | "You were named for a grandparent you never met, and told every year how little you resembled them." | —; — |
| `quiet_birth` | common (5) | — | "You were born without sign or omen, which {guardian} counted a mercy." | — (plain) |

### 11.3 Childhood (10)

The stat chapter. `strong_child`, `sickly_child`, `wolf_in_the_yard`, `ordinary_child` are in §4; the rest: `quick_child` (Dex +1: "You were quick, and hard to catch, and oftener up a tree than at the table."), `watchful_child` (Int +1: "You were a watchful, inward child, who asked why until the grown folk stopped answering."), `wilful_child` (Wi +1, Pr −1: "You were stubborn as a mule and twice as hard to lead."), `winning_child` (Pr +1, `not_tags = ["runt", "orphan"]`: "You were a winning child; the neighbours spoiled you and you let them."), `the_bully` (St +1, alignment −50, tag `bully`: "You were the biggest of the children, and let them know it."), `the_bullied` (Wi +1, To −1, tag `bullied`, `not_tags = ["strong", "bully"]`: "The bigger children had you for sport, and you learned to run before you learned to fight.").

### 11.4 Youth (12)

The class chapter: a trade, a teacher, a levy. Age-gated (`age = { min = 16 }` on apprenticeships; the warren's and the kobold's lines are short). Skill starts and weapon marks.

| id | Tier (w) | `when` | Text | Effects; tags |
|---|---|---|---|---|
| `the_levy` | common (3) | class warrior, paladin, barbarian, commoner; age ≥ 14 | "At fourteen you stood in the levy with a borrowed spear, and found you had a taste for it." | weapon_marks spears 3, skill athletics 1d4; `soldier` |
| `temple_years` | common (3) | class priest, paladin, or tags orphan | "You served your years at the minster: swept, sang, and sat with the dying." | skill heal 2d4, alignment +25; `temple` |
| `the_copyist` | common (3) | class wizard, witch, bard; age ≥ 14 | "A hedge-wizard took you as a copyist; you learned your letters off his spellbooks, and a little more besides." | skill decipher_script 2d4, item `scroll_of_light`; `lettered` |
| `light_fingers` | common (3) | class thief, bard, or tags poor, orphan | "You ran with the market children, and your fingers grew light before your conscience grew heavy." | skill steal 1d4, alignment −25; `light_fingers` |
| `bound_apprentice` | common (3) | tags smith, trader; age ≥ 18 | "You were bound apprentice at {trade}, seven years, and served every one of them." | skill metalcraft 2d4+2 (tags smith) or negotiation 2d4 (tags trader): two lines, `apprentice_smith` and `apprentice_stall`; `apprenticed` |
| `the_woods` | common (3) | class ranger, druid, or tags hunter | "You spent more of your youth in the wood than out of it, and it fed you better than the house did." | skill survival 2d4; `woodsman` |
| `the_barges` | common (2) | not tags deep | "You worked the river barges two summers, and swim better than most who were born to the water." | skill swim 2d4; `bargeman` |
| `idle_youth` | common (2) | tags wealthy, or class aristocrat | "You spent your youth as the sons and daughters of good houses do: at the hunt, at the table, and at nothing." | skill handle_animal 2d4; `idle` |
| `the_old_soldier` | rare (1) | age ≥ 16 | "An old soldier wintered at {home} the year you were sixteen, and taught you, for the price of your listening, everything he knew." | weapon_marks best 6, stat dexterity +1, skill intimidate 2d4; `taught` |
| `plain_work` | common (5) | — | "You worked, as everyone worked, and nothing of it is worth the telling." | — (plain) |

### 11.5 The trial (10)

The grim chapter: what it cost, and what you kept. Mostly common lines with a bane and a smaller boon; the rare line is a terrible thing survived.

| id | Tier (w) | `when` | Text | Effects; tags |
|---|---|---|---|---|
| `the_fever` | common (2) | not tags orphan | "The fever came through {home} the year you were grown, and took {guardian}. You nursed the rest, and it did not take you." | stat toughness −1, willpower +1, skill heal 1d4; `orphan`; guardian "the neighbours" |
| `the_fire` | common (2) | — | "The house burned, one dry autumn, with everything in it. You went into the ash for what could be saved, and came out with a ring that had been your grandmother's." | gold −20, item `{ wear = "iron_ring" }`; `burnt_out` |
| `the_feud` | common (1) | not tags temple | "Your family had a feud, as families do, and you have a scar across the ribs from the night it was settled." | stat toughness +1, alignment −25; `blood` |
| `the_war` | common (1) | tags soldier, or class warrior, barbarian | "The lord's war took you for two years and gave you back older, with a limp in wet weather and a way of waking at every sound." | weapon_marks best 3, stat presence −1, skill listen 1d4; `veteran` |
| `promise_broken` | common (2) | — | "You were promised once, and the promise was broken, and you have not made another." | stat willpower +1, presence −1; `heartsore` |
| `the_debt` | common (2) | not tags wealthy | "{Guardian} died owing, and you worked the debt off a shilling at a time, and learned what every shilling was worth." | gold −15, skill negotiation 1d4; `indebted` |
| `the_mill` | common (1) | — | "You lost two fingers of your left hand to a mill-wheel, and learned to do without them." | stat dexterity −1, willpower +1; `maimed` |
| `the_barrow` | rare (1) | age ≥ 14 | "Something came out of the barrow one winter and took the sheep, and then a child. You went down after it with a lantern and a hayfork. You do not speak of what you found; but you came back, and the child did not." | stat strength +1, willpower +1; `barrow` |
| `the_flux` | fateful (1) | — | "You were dying of the flux at nine, and a hedge-priest laid hands on you, and you got up the next morning. You have not been sick a day since." | talent `hardy`, stat toughness +1; `blessed` |
| `quiet_years` | common (5) | — | "No great grief found you, and you count that luck, as you should." | — (plain) |

### 11.6 Fortune (10)

The boon chapter: what was given. The plain line is heavy (fortune is rare by nature), and the uncommon and rare lines are where purses and heirlooms live.

| id | Tier (w) | `when` | Text | Effects; tags |
|---|---|---|---|---|
| `purse_on_the_road` | common (2) | — | "You found a purse on the road once, and kept it, and have been half ashamed and half glad of it since." | gold 4d6, alignment −25; `found` |
| `the_bequest` | uncommon (2) | not talents family_wealth | "An aunt you had met twice left you her savings, with a letter that said she had always thought you the only one worth anything." | gold 6d6+10; `bequest` |
| `strangers_ring` | uncommon (2) | — | "A stranger you pulled out of a ditch pressed a ring on you, and would not take it back, and was gone by morning." | item `{ wear = "silver_ring\|iron_ring", magic = "any" }`; `ring` |
| `the_dog_that_stayed` | common (1) | not tags deep | "A dog followed you home one spring and stayed three years, and taught you more about patience than any person did." | skill handle_animal 2d4; `dog` |
| `the_heirloom` | rare (1) | not talents fathers_sword; age ≥ 14 | "In the chest under the bed, when {guardian} died, you found what no one in the family had spoken of: a blade older than the house, with a name on the hilt." | item `{ carry = "long_sword\|battle_axe\|war_hammer\|spear\|short_bow", magic = "magic", prefer_trained = true }` (identified); `orphan` if not already; `heirloom` |
| `fathers_blade_told` | common (1), **only** | talents fathers_sword | "The blade you carry was your father's, and his father's; the name on the hilt is older than the house." | —; `heirloom` |
| `the_named_blade` | fateful (1) | not talents fathers_sword | "On the night you came of age a stranger left a wrapped thing at the door and no name; inside was a blade, and on the blade a word you could not read, and you have never been able to put it down for long." | item `{ carry = "long_sword\|rapier\|scimitar\|battle_axe\|war_hammer\|quarterstaff", magic = "rare", prefer_trained = true }` (identified); `named_blade` |
| `the_hermits_word` | rare (1) | — | "The hermit of the hill told you, when you were twelve, exactly how you would die; you have forgotten it, and it has made you very hard to frighten." | stat willpower +2; `fearless` |
| `the_teachers_book` | uncommon (2) | tags lettered, temple, or class wizard, priest | "Your teacher gave you, on parting, a book they said was worth more than you knew." | item `{ carry = "book" }`, skill knowledge 2d4; `book` |
| `nothing_given` | common (6) | — | "Fortune never found your door; whatever you have, you got yourself." | — (plain) |

`prefer_trained` is the one extension to the kit syntax this design asks for: among a line's alternatives, the roll takes one whose weapon skill the character trains (`KitItem::trained`'s rule per alternative), else any; so a warrior's heirloom is a long sword and an archer's a short bow.

### 11.7 Leaving (10)

Why you left. Tag-conditioned so the reason follows the life; alignment nudges.

| id | Tier (w) | `when` | Text | Effects; tags |
|---|---|---|---|---|
| `nothing_held_you` | common (3) | tags orphan | "There was nothing to hold you, and nobody to say goodbye to, and so you did not." | —; — |
| `third_child` | common (2) | tags poor, not tags orphan | "You were the third child of a house that could feed two, and you went so the others could stay." | gold 1d6; — |
| `the_letter` | uncommon (2) | — | "A letter came, in a hand you did not know, with a little money in it and three words: *Come. There's work.*" | gold 2d6+10, item `{ wear = "travelling_cloak" }`; `summoned` |
| `one_step_ahead` | common (1) | tags light_fingers, blood, or band max callous | "You left a step ahead of the reeve's men, and have not written home." | alignment −25; `fugitive` |
| `sent` | common (2) | tags temple, or class priest, paladin | "You left because you were sent: the minster had need of someone who could walk, and you could walk." | alignment +25; `sent` |
| `for_the_stories` | common (3) | class warrior, barbarian, bard, aristocrat, ranger | "You left because the stories were all somewhere else." | —; — |
| `the_quarrel` | common (2) | not tags orphan | "You left after a quarrel with {guardian} that neither of you has yet forgiven." | stat willpower +1, presence −1; `quarrel` |
| `the_dream` | rare (1) | — | "You dreamed, three nights running, of a door under a hill, and woke the third morning knowing the way; you have not dreamed since." | skill search 2d4, stat intelligence +1; `dreamer` |
| `grown_too_big` | common (2) | tags strong, bully, warren | "You grew too big for {home}, and {home} said so." | —; — |
| `plain_leaving` | common (4) | — | "You left, as the young do, because staying was the one thing you could not bear." | — (plain) |

### 11.8 The road (the Ascmearc's 6)

`assets/campaigns/ascmearc/data/background/08-road.toml`: how you came over Beorggeat. The one chapter that names the world (Grægweall, Fyrbeorg); it sits last, so its lines may read anything.

| id | Tier (w) | `when` | Text | Effects; tags |
|---|---|---|---|---|
| `the_last_cart` | common (4) | — | "You came over the Grægweall with the last cart before the snow, and paid for the ride with work." | — (plain) |
| `alone_on_foot` | common (3) | — | "You came over the pass alone and on foot, with the snow a day behind you." | skill survival 1d4; — |
| `the_pedlar` | common (2) | — | "You walked up with a pedlar who talked the whole way and sold you a knife at the top for twice its worth. 'Yer'll want that, where yer goin',' he said, and he was right." | item `{ carry = "dagger" }`, gold −5; — |
| `lost_to_the_snow` | common (2) | — | "The snow caught you on the high road and took your pack; you came down the far side with what you stood in and a new respect for weather." | gold −10, stat toughness +1; — |
| `the_fire_that_should_not_burn` | uncommon (2) | — | "At the top of the pass an old woman sat by a fire that should not have burned in that wind, and told you the ash march would make you or unmake you. She gave you bread for the road." | item `waybread*2`, stat willpower +1; `foretold` |
| `fyrbeorg_lit` | rare (1) | element fire | "Fyrbeorg lit the whole sky the night you crossed, and you walked down into the march by its light, and have felt since that it was looking for you." | stat toughness +1, willpower +1; `fire_marked` |

The showcase campaign's test chapter: `the_hall` ("You came to the Hall by the road everyone comes by.", plain) and `the_arena` (a line conditioned on the arena scenario's class, with one item, so a test sees an item land).

### 11.9 Writing rules

- **The game's voice**: second person, past tense, the plain English of the march's own day (the "Victorian" register of the memo for narration: plain, dry, with the odd long sentence; never archaic "thee", never modern slang). The register memo's Cockney and posh are for **quoted speech** inside a line, by the speaker's station (the pedlar: "Yer'll want that"; the aunt's letter: "the only one worth anything"). Old English names only in a campaign's chapter, never in shared ones.
- **One line, one thing**: a sentence or two; never a list, never a second event. Effects follow from what is said (the fever teaches nursing; the feud leaves a scar and a grudge).
- **Grim is allowed, cruel is not**: a line may take and may grieve; it never mocks the character, and the plain line is never a put-down.
- **No numbers, no mechanics** (§8.3); a test keeps it so.
- **Every race's dignity**: a drow's, a kobold's, a duergar's childhood is its own, not a human's with the nouns changed; their lines are conditioned, and the plain lines are written so they fit any.
- **Ids are plain English** in `snake_case`; texts are what change, ids never (saves name them).

## 12. Three sample stories

Each as the screen shows it, then the lines behind it. The tiers and effects are §11's; the draws are illustrative.

### 12.1 Common: Edwin, a human man, a commoner (Earth Griffin)

> You were born to farming folk, who had little but gave what they had. You were born without sign or omen, which your family counted a mercy.
>
> You were a strong child, broad for your age, and carried what the others could not. You worked the river barges two summers, and swim better than most who were born to the water.
>
> No great grief found you, and you count that luck, as you should. You found a purse on the road once, and kept it, and have been half ashamed and half glad of it since.
>
> You were the third child of a house that could feed two, and you went so the others could stay. You came over the Grægweall with the last cart before the snow, and paid for the ride with work.

| Chapter | Line | Tier | Effect |
|---|---|---|---|
| Parents | `farm_folk` | common | tag `poor` |
| Birth | `quiet_birth` | common | — |
| Childhood | `strong_child` | common | Strength +1 |
| Youth | `the_barges` | common | Swim 2d4 (5) |
| Trial | `quiet_years` | common | — |
| Fortune | `purse_on_the_road` | common | 4d6 gold (13), alignment −25 |
| Leaving | `third_child` | common | 1d6 gold (4) |
| Road | `the_last_cart` | common | — |

Boons 2 + 1 + 1 + 1 = 5 points, banes 0; Neutral stays Neutral (−25 is a sixth of the band). A commoner with Strength 11 instead of 10, a swimmer, and 20 coins instead of 3: a good common story, and ordinary to read.

### 12.2 With a rare event: Brakka, a dwarf woman, a warrior (Metal Manticore)

The screen in §8.1. The lines:

| Chapter | Line | Tier | Effect |
|---|---|---|---|
| Parents | `deep_halls` | common | tag `deep`, home "the deep halls" |
| Birth | `hard_winter` | common | Toughness −1, Willpower +1 |
| Childhood | `wilful_child` | common | Willpower +1, Presence −1 |
| Youth | `the_levy` | common | spears marks 3, Athletics 1d4 (3) |
| Trial | `the_feud` | common | Toughness +1 (back to where it was), alignment −25 |
| Fortune | **`the_heirloom`** | **rare** | a magic long sword (the trained alternative), identified: *a long sword of biting* (two low affixes); tag `heirloom`; `orphan` (her mother died) |
| Leaving | `for_the_stories` | common | — |
| Road | `the_fire_that_should_not_burn` | uncommon | waybread ×2, Willpower +1 |

Boons: 2 + 2 + 2 + 2 + 3 + 3 = 14; banes: 2 + 2 = 4. Net: Willpower +3 (and its potential +3 with it), Presence −1, Toughness 0, a magic sword, bread for the road, a few marks and a small skill, a sixth of a band colder. A warrior who starts with a named blade and a grudge: the rare story the owner asked for, and a story that gives what its lines give, uncapped (decision 2).

### 12.3 Grim: Tamsin, a halfling woman, a thief (Water Basilisk)

> You were a foundling, left at the minster door in a basket of rushes; the sisters raised you, and never let you forget it. You were born one of twins. The other did not live out the week, and the sisters never spoke of it.
>
> The bigger children had you for sport, and you learned to run before you learned to fight. You ran with the market children, and your fingers grew light before your conscience grew heavy.
>
> The sisters died owing, and you worked the debt off a shilling at a time, and learned what every shilling was worth. Fortune never found your door; whatever you have, you got yourself.
>
> You left a step ahead of the reeve's men, and have not written home. You came over the pass alone and on foot, with the snow a day behind you.

| Chapter | Line | Tier | Effect |
|---|---|---|---|
| Parents | `foundling` | common | Heal 1d4 (2), alignment +25; `orphan`, guardian "the sisters" |
| Birth | `the_twin` | common | Willpower +1, Presence −1 (the slot: `{guardian}` is "the sisters") |
| Childhood | `the_bullied` | common | Willpower +1, Toughness −1 |
| Youth | `light_fingers` | common | Steal 1d4 (raising the thief's trained Steal by 3), alignment −25 |
| Trial | `the_debt` | common | gold −15 (the thief's 25 become 10), Negotiation 1d4 (raising it) |
| Fortune | `nothing_given` | common | — |
| Leaving | `one_step_ahead` | common | alignment −25; `fugitive` |
| Road | `alone_on_foot` | common | Survival 1d4 (3) |

Boons 1 + 2 + 2 + 1 + 1 + 1 = 8; banes 2 + 2 + 1 = 5 (the −15 gold is 1 point). Net: Willpower +2, Presence −1, Toughness −1 (each potential moved with its score), 15 coins gone, a thief's Neutral start 25 colder, and three small skills. Eight common lines, not one of them kind; had the Trial given `the_mill` as well, two fingers would have gone with the coins, and nothing would have stopped it (decision 7). A character with something anyway, because the lines are written that way (§11.9).

## 13. Determinism, saves, replays

| Part | Rule |
|---|---|
| Draws | the chapters' own streams (§5.1), plus a magic item's loot draws from the same chapter stream (never the loot stream: no level changes); the gameplay, training, levels, and creation streams are untouched. Runs without a story draw nothing new |
| Saved state | `Character::story: Vec<Told { chapter: String, line: String }>` (ids, not text: an edited sentence reaches a saved character; a line removed from the content is skipped with a log warning and the paragraph closes up), `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, hashed as `Character` is. A story talent or flaw is an `Actor::boons` entry (saved already). The effects are in the actor (scores, skills, marks, the pack, the score): nothing else is stored, and nothing is re-applied on load |
| `FORMAT_VERSION` | untouched (the integrator's): the field is optional and old saves load with no story |
| Replays | no new field: `{seed, character (with picks), name, kit}` determine the story. **Hashes change for every run with the kit** (New run's and `--character`'s), from the first step; `--scenario`-only runs and every committed snapshot, golden walk, and test replay that does not start with the kit hash as before. The screen snapshots that start with the kit (`kit-inventory`, `kit-equipment`, `kit-summary`: the summary is pre-story and unchanged; the other two change where the story gives an item), `character-screen` cases with `kit`, and any `--talents` capture are re-read with their diffs. The golden manifest's `kit = true` entries, if any walk changes (an item in the pack moves nothing visible), are checked by the finisher |
| The witness | reads only what `start_run` has settled: scores before the story, the band, skills, talents, the sign, the age at the start date, the campaign. A later change to any of those (a new optional skill in a class) changes stories of that class on old seeds, as #671 changed their skills: accepted, said in the PR |
| Save compatibility test | `save::tests::a_save_without_a_story_loads` and `a_story_round_trips` |
| The web build | nothing platform-specific; `localStorage` saves carry the field |

## 14. The wiki, the spec, and the docs

**The wiki** (the game vague, the wiki exact): a new page **Your story** (`wiki/src/background.md`) under "Your character" after Horoscope in `SUMMARY.md`: what a story is, when it is told, that nothing in it is chosen, that nothing caps it and a score stops at 1, the talents; and generated by `cargo xtask wiki` (`xtask/src/wiki/background.rs`, one function a table, registered in `generate()` beside the character tables):

| Table | Content |
|---|---|
| `background-tiers.md` | the four tiers: odds per chapter, odds per story of eight (at least one; expected count), the net-point bands, with and without Favoured child |
| `background-points.md` | the `[points]` table and the tier bands in words ("a stat point is 2; a rare line is worth 3 to 5 and never costs"), and the one floor |
| `background-<chapter>.md` × 8 (the Ascmearc's road included; the showcase's never) | every line: id, tier, weight, conditions in words ("dwarves and duergar"; "with Family wealth"; "orphaned"), the text, the effects exactly ("Strength +1", "4d6 gold", "a magic long sword, identified"), the tags it sets |
| `background-talents.md` | the three talents with their exact rules |

**Names** (decision 10, "on names": left to the design): the talents' names (Favoured child, Hard-won, Storyteller), the chapters' and lines' ids, the screen's title "Your story", the page's name "Story", the dump layer `story`, and the field `Character::story` are the design's; none clashes with an id in the data (`story` is no talent, skill, or status). The implementers may polish the talents' names and descriptions (decision 11), keeping the ids.

**The spec**: `spec/08i-character.md` gains the bullet **Background story** after "What a sign gives" (the chapters, the streams, the tiers, the floor, the order of effects, who has one, the saved ids); `spec/07e-characters.md` the data bullet (`data/background.toml`, `data/background/`, a campaign's chapter); `spec/06-input-and-ui.md` the Story screen and page 8; `spec/07f-scenarios-and-saves.md` the `story` dump layer and JSON; `spec/09-development-interface.md` `--screen story`; `spec/03b-campaign.md` one line: a campaign may add a chapter. CLAUDE.md: the dump layer in the layers list and `story` among `--screen`'s names.

**The docs**: `docs/dev/background.md`, new (the modules `content/background.rs`, `background.rs`, `world/background.rs`, `ui/story_screen.rs`, `xtask/src/wiki/background.rs`; the data; the witness; the streams; the floor; the order in `start_run`; what is saved; the tests), listed in CLAUDE.md's topics after `character-creation.md`; one line each in `character-creation.md` (the order), `progression.md` (the `background` system, `Actor::boons` from a story, the creation-only talents), `skills.md` (a story's skill starts), `stats.md` (`Stats::gift` from a story), `alignment.md` (the fourth term), `screens.md` (the screen, page 8, `--screen story`), `saves.md` (the field), `campaign.md` (a campaign's chapter), `content.md` (the directory and the campaign's).

## 15. Choices made along the way, and what is rejected

| Choice | Taken | Rejected, and why |
|---|---|---|
| When the story is told | after every choice and the picks, in `start_run` | before the picks (so picks could read tags): then a story boon could meet a prerequisite and a bane could un-take one, and the summary would have to show the story to explain the offers, which breaks "shown after the run starts" |
| Where it is shown first | a screen over the first turn, opened by the app once | a message in the log (four paragraphs in the message line is unreadable); the summary (the owner: after the run starts, no going back); a modal the player must scroll (it is short) |
| Where it is read again | page 8 of the info pages | a sub-screen of the character screen (a new key or a row to Enter on: the character screen is a page without rows); the Characters list (reads no save); the death screen (the epitaph is the death's: question 6) |
| Rarity | a tier drawn first, per chapter, in per mille | weights alone (rarity would be an accident of pool sizes and unreadable in the wiki); one "fate" roll per story (then rare events cannot land where they fit the chapter; and eight small chances read better than one) |
| Caps | **none** (decision 2): a story is the sum of its lines; the floor is a score of 1 | the first draft's caps (boons 12, banes 5, a stat ±2, counted by the roller: the owner struck them, "no caps! just a floor"); a cap on the data (uncheckable across paths); re-rolling a chapter (a second draw changes the stream's shape) |
| Banes | real, uncapped, in common lines; rare and fateful lines net-positive; no protection (decision 7) | banes in rare lines (the owner: rare events give boons); no banes (ADOM's and the owner's "sad event may cost something"); a tragic tier (a "cursed" story, 1 in 200): not rejected on principle any more (the owner wants no protection from bad luck), only not needed: eight uncapped common lines can already be cruel |
| Stat changes | `Stats::gift`: score and potential together (decision 8) | the score alone (then a bane is permanent damage with no way back: Hard-won would have nothing to win); the potential alone (invisible) |
| Items | kit lines, identified when magic | a story item table of its own (the kit syntax has everything, alternatives and `magic` included); unidentified heirlooms (you grew up with it; and Father's sword is identified, #697) |
| Talents from a story | `Actor::boons`, the altar's machinery; never `granted`, class, or flaw talents as boons (flaws through `flaw`) | a new `Taken::source`; a story tree |
| The heritage talents' overlap (#700) | the talents bias and force pools; no double gift | the story's heirloom lines removed (then Father's sword is the only way to a named blade, and a fateful line has nothing of legend to give); the talents reading the story (they are picked before it) |
| Tags and slots | a closed vocabulary, checked at load | free-form text references (no agreement, no check); a parse of earlier texts |
| The campaign's chapter | one, last, in the campaign's `data/background/` | the campaign overriding shared chapters (then the Ascmearc's and a later world's stories diverge for nothing); the pass in a shared chapter (names a world) |
| Who | runs with the kit | every run with a character (then every test and snapshot of the scenarios' own characters changes, and a `--scenario` dev launch is slower to read); only New run (then `--character` captures and the CLI could never show a story) |
| Companions, parents named, a hometown on the map | not proposed | companions are no system of the game yet (#196 holds pets); named parents want `names.toml` by age and the drow's and the kobold's naming, a content task of its own; a hometown wants the overworld's locations to be data a story can point at (it is), and a quest hook ("go home"), which is a design of its own |
| A story re-rolled by a shrine or a potion of forgetting | no | the owner: no going back; and the effects are in the actor |

## 16. Implementation plan

Each item is one issue, one branch, one PR, updating the spec, `docs/dev/background.md` (created in B1), the other topic files it touches, and the wiki in the same PR. "Save" marks a PR that adds saved state; none adds an action or a `KeyCommand`. Sizes: S about a day, M two to three, L a week.

| # | Issue title | Sections | Adds | Depends on | Size |
|---|---|---|---|---|---|
| **B1** | Background story: the data (`background.toml`, `data/background/`, a campaign's chapter), the loader and its checks, `Rng::background`, `background::tell` (tiers, pools, slots; no caps), `World::tell_background` in `start_run` with the `stat` (score and potential together, floor 1), `gold`, `item` (plain and `magic`, identified, `prefer_trained`), `skill`, `weapon_marks`, and `alignment` effects, `Character::story`, the `story` dump layer and JSON, `--screen` not yet; a first pool of three lines a chapter (the plain line and two others each) and the Ascmearc's `road` with two; `docs/dev/background.md`; the summary's dim line; tests (§13, the floor, the odds, the streams, replays, saves, the hidden-number grep) | §3–§7, §9, §13 | **save** | — | L |
| **B2** | The story screen: `ScreenName::Story`, `ui/story_screen.rs`, opened once at a run's start by the app (never on resume, capture, or replay), page 8 in `PAGES`, `--screen story`, text snapshots (`story-screen`, `story-page`, `story-page-none`), the web smoke's extra Enter | §8 | — | B1 | M |
| **B3** | The pools, **written by the implementer** (decision 13): every chapter to its target size (78 lines in all, §11's lines as the register guide and first draft, more welcome), the showcase's test chapter, three sample stories as tests (`background::tests::the_samples_read_as_written`: fixed seeds found by a helper and pinned), the wiki page and its generated tables (`xtask/src/wiki/background.rs`); the owner reviews the texts after the wave, not in the PR | §11, §12, §14 | — | B1 | M |
| **B4** | The `background` system and its talents: Favoured child and Hard-won (creation-only, `needs = { creation = true }`, the `heritage` tree) and Storyteller; the `story_luck`, `hard_won`, and `storyteller` reads; the `talents` dump's "(story)"; the wording polished at the implementer's discretion, ids kept (decision 11) | §10 | — | B1 | S–M |
| **B5** | The fateful effects: `talent` and `flaw` lines through `Actor::boons`, the lines that use them (`the_flux`, `the_named_blade`'s `magic = "rare"`), the class table's "nothing in its place" test, the Talents page's "(story)" source | §6.1 (talent, flaw), §11.5–11.6 | — (boons are saved already) | B1, B3 | S |

**Waves** (at most about three PRs at once that add actions or save state; here one):

1. **B1** alone (it adds saved state and changes every kit run's hashes: the snapshot diffs are read once, in one PR).
2. **B2** and **B3** together (B2 is `crates/warlock/src/ui/`, B3 is content and `xtask/`: no file in common but `docs/dev/background.md`, where each adds its own section).
3. **B4** and **B5** together (B4 is `data/talents/`, `data/talents.toml`, `TalentMods`, and three reads; B5 is `world/background.rs`'s two effect arms, `boons.rs`, and content lines: no file in common but `docs/dev/background.md`). The heritage talents (#700) are merged, so nothing waits on them.

**Tests each item carries**: `content/background.rs` (parsing; every refusal: a bad tier, a tag no earlier chapter sets, an unknown slot, a stat change of 3, a rare line with a bane, a chapter without a plain line, a campaign chapter reusing a shared id, a kit line that would Strain); `background.rs` (the tier odds over 20 000 seeds within 10 % of the table; no score below 1 and no gold below 0 over every race × gender × class × 1 000 seeds, however grim the lines; the fallback to the plain line when a tier is empty; `only` lines; slots and defaults; the witness's conditions one by one; the same spec on two seeds differs, the same seed and spec never); `world/background_tests.rs` (each effect kind lands where §6.1 says: the score and potential, hit points at the new maximum, the coins, an item in a free slot or carried, a magic heirloom identified and known, a skill trained or raised, marks, the band moved; told after the picks: a pick's prerequisite is not met by a story point; no story for a `--scenario` run and the hash of one unchanged; a replay with `kit` reproduces the story; a save round-trips the ids and re-tells the text; an edited text reaches a save; a removed line is skipped); `ui/story_screen.rs` and the screen snapshots (the first-turn screen, page 8, the page without a story, the tab line's eighth name); `xtask`'s wiki tables generated; the samples pinned. **No golden**: nothing here is the GPU's; the screen is a text snapshot.

## 17. Open questions, each with a recommended default (answered: §18)

As put to the owner; the answers are §18, and the body above is revised to them (the caps of the first draft are gone from §1, §4, §5.3, §7, §12, §15, and §16).

1. **The tier odds** (12 / 3 / 0.5 % per chapter; a fateful line for one character in twenty-five). Default: as proposed; they are one table in data and the wiki shows the per-story odds, so the owner can turn the dial after reading a few stories.
2. **The caps** (boons 12, banes 5, a stat ±2). Default: as proposed. A looser pair (16 / 7) makes a fateful story a level and a half's worth; the owner may prefer that ("unbalanced is fine"), in which case only `[caps]` changes.
3. **The summary's line** ("Fate will have its say when you set out."). Default: keep it; it leaks nothing and explains the post-story scores. The alternative is silence and a surprise.
4. **The screen's opening**: once, at the moment the run begins, before any input. Default: yes. The alternative, opening it on the first `@`, loses the moment.
5. **Scenario-only dev runs** (`--scenario` alone): no story. Default: keep; it is what keeps every test and golden walk's hash. A dev who wants one passes `--character`.
6. **The death screen**: nothing new. Default: keep. The alternative, one dimmed line from the story's Parents chapter under "Here lies …" ("born to farming folk"), is one line in `death_screen.rs` if the owner likes the epitaph fuller.
7. **A tragic tier** (a story with a bane cap of 8 for one character in 200, no boon). Default: no; a player who draws it can only start over, and the owner's rule is that there is no going back. The data could carry it later as a fifth tier if the owner wants cruelty.
8. **Potentials**: a story's stat point moves the potential with the score (`Stats::gift`). Default: yes. The alternative, the score only, makes a bane permanent and Hard-won pointless.
9. **Identified heirlooms**: a story's magic item is identified. Default: yes (you grew up with it; #697's sword is). The alternative keeps the describe screen's mystery for a stranger's ring: `strangers_ring` could be the one unidentified exception; not proposed.
10. **Named parents and places**: no names in the shared chapters; `{home}` is a kind of place, never a map cell. Default: keep; names and hometowns are a content design of their own (§15).
11. **The three talents**: Favoured child, Hard-won, Storyteller. Default: as proposed; the first two use #700's creation-only field (`needs = { creation = true }`).
12. **Where `road` lives**: the campaign's data. Default: yes; a campaign without one has seven chapters, and the showcase's test chapter never ships.
13. **Who the first wave's lines are written by**: B1 ships three lines a chapter so the machinery is testable; B3 fills the pools. Default: the implementer writes to §11's tables and §11.9's rules, and the owner reviews the texts in B3's PR, which is content only.

## 18. Decisions (owner, 2026-10-29)

The owner's answers to §17, verbatim, with the reading taken where an answer needed one. The body above is revised to them.

| # | Question | Answer | Taken as |
|---|---|---|---|
| 1 | The tier odds | "sounds good" | 12 / 3 / 0.5 % per chapter as proposed (§7); the data is the dial |
| 2 | The caps | "no caps! just a floor. ability score is always at least 1 :)" | **no caps at all**: the boon cap, the bane cap, the per-stat ±2, the roller's cap arithmetic, and the "dropped line" tuning are gone (§1, §4, §5.3, §7, §12, §15, §16); the one floor is the game's own, a score never below 1 (`Stats::gift` keeps 1..=40), with gold never below 0. The per-line tier bands stay as a content check (a rare line is net-positive); they bound a line, not a story |
| 3 | The summary's "Fate will have its say when you set out." | "sure" | kept (§8.1) |
| 4 | The screen opens once, as the run begins | "yes" | kept (§8.1) |
| 5 | Scenario-only dev runs have no story | "sure, you know what's best for dev process" | kept (§9): every existing hash and snapshot holds |
| 6 | Nothing new on the death screen | "sure" | kept (§8.2) |
| 7 | A tragic tier | "no need to overthink about it. no need for any protection for any bad luck" | no tragic tier is added, and **no protection of any kind** against a bad story: no re-roll, no pity line, no floor on what a story takes but the score's 1 (§5.3, §7, §15). The first draft's reasons against a tragic tier (a player's only recourse would be to start over) are withdrawn; it is simply not needed |
| 8 | A story's stat point moves the potential with the score | "sure, they also move potential" | kept and said exactly (§6.1): `Stats::gift` raises or lowers the score and the potential by the same amount, the potential never below the score |
| 9 | A story's magic item is identified | "yes" | kept (§6.1, §11.6) |
| 10 | Named parents and places | "on names" | read as "decide on the names yourself": no named parents or map places (§15); the design's own names stand: the three talents, the chapters' and lines' ids, "Your story", page "Story", the `story` layer, `Character::story` (§14, "Names"), none clashing with an id in the data |
| 11 | The three talents | "sure, they sound kinda shit but let's implement them" | implemented as designed (§10, B4); the implementer may polish the names and descriptions, keeping the ids and the rules |
| 12 | The road chapter is the campaign's | "sure" | kept (§3, §11.8) |
| 13 | Who writes the pools | "implementers can write everything. we will review and refine later when the whole wave is done" | the implementers write every pool (B1's first lines, B3's full pools, and more); §11's lines are the register guide and a first draft, its counts targets; the owner's review of the texts comes after the wave, not line by line in the PRs (§11, §16) |

Also folded in: #697 merged as #700 while the questions were out, so the heritage talents' real ids (`family_wealth`, `fathers_sword`), their field (`needs = { creation = true }`), and their tree (`heritage`) are named throughout, Favoured child and Hard-won join that tree, and nothing in §16 waits on them.

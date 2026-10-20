# Warlock — Horoscope revamp: sign abilities, lucky years, the sign revealed after the race

Design document for the owner's review (2026-10-20). Written against `origin/main` at 247d48cf and the approved progression design (`docs/design/progression.md`). Nothing here is implemented; §11 cuts it into issues.

## Contents

1. Summary
2. Research: ADOM's star signs and the others, tied to the decisions
3. The model
4. The twelve animals
5. The five elements
6. The five lucky years
7. Moons and calendar flavour (optional)
8. Character creation
9. Interactions: talents, classes, mana, resources, alignment, coverage, hidden numbers
10. Data and code sketch
11. Implementation plan
12. Open questions, each with a recommendation

## 1. Summary

**What the owner asked for** (2026-10-20): "gameplay effects to horoscopes. propose a revamp that adds significant abilities to horoscopes and elements (also pick 5 lucky combinations with an extra ability just to be more unfair). These should be on par with ADOM horoscopes, which were a significant part of character creation. Revealing the sign is the first step of the character creation." Amended the same day: "oh, my bad. so we then reveal the sign after race selection." And the wording rule, handled by #256 but binding here: nothing player-facing says randomness comes from the seed; in the game it is fate, the stars, or chance.

**The proposal in one table:**

| Piece | Decision |
|---|---|
| What a sign is | unchanged: the animal and the element of the birth year, 12 × 5 = 60 signs in a great cycle, the birth year counted back from the start date by an age rolled in the race's range (the stream `character/birth/<race>`, as today). Existing seeds keep their birth dates, signs, and scores. §3, §8 |
| What a sign gives | **two gifts and one flaw per animal**, **two gifts per element**, and for **five of the sixty signs** (one animal per element) a named **lucky year** with an extra, unfair gift. The small stat adjustments stay. §3–6 |
| How a gift is built | a **granted talent**: a file in `data/talents/` (the progression design's model, #229), in a new tree **Stars**, never offered on a pick list, granted by the sign as a race grants `versatile`. Passives read through the one `TalentMods` aggregation at the hooks that exist; actives are entries in the one abilities menu under a new group **Birth sign** (`Source::Sign`), with unbound rebindable keys like the shield bash. The coverage test counts them. §3, §9, §10 |
| What a sign unlocks later | **sign-limited talents**: picks in the Stars tree with a new prerequisite kind (`animal`, `element`, `lucky`), one per animal and element, so a sign matters at level 30 as at level 1. §9.1 |
| The reveal | a new step **after the race**: "Born under the stars" shows the birth date, the age, the sign, the Great Race line, the element's line, the gifts and the flaw in words, the lucky year if any, and the stat adjustments as data; Enter goes on to the gender, Esc back to the race. The summary gets a Sign row that returns to it. `--screen sign`. No choice is made on it: the stars decide. §8 |
| Flavour | optional, subtle, deterministic: a birthday, the birth month, and the full moon strengthen a few gifts; a data table, off by leaving it out. §7 |
| Hidden numbers | speed, hearing, marks, and checks stay in words; what the game shows (Defence, Protection, carrying, favour, XP) says its number. §9.5 |
| Plan | eight issues after the talents core (#229) lands; the data, the reveal, and the first passives in one PR, the rest in parallel with the progression waves. §11 |

**Key decisions, and why**

1. *Gifts are talents, granted.* The progression design already has granted talents (`versatile`), a `TalentMods` aggregation at every hook, an abilities registry, a Talents page, a coverage test, and a wiki generator. A second model for signs would be the same code twice. The price is a dependency on #229, which is the next wave anyway.
2. *Significant means a changed rule, not a number*: regeneration, speed, cheating death, a free blow, a cure, a stare, loot odds, immunity. ADOM's loved signs (Candle, Raven) changed how a character played; its forgettable ones (Falcon's Survival) did not (§2).
3. *Every animal has a flaw*, as ADOM's Dragon had (−3 Willpower): signs are fate, and fate is uneven; the owner wants unfair. Elements have none: they are the "good" half of the sign.
4. *The lucky years are one per element*, so every element has a legend, and the Great Race's own year (the Wood Salamander, 1441) is one of them.
5. *Nothing moves for existing seeds.* The birth stream, the stat streams, and the sign of a year are untouched; only what a sign gives changes.
6. *The sign is revealed, never chosen*, and never re-rolled on its screen: going back to the race and choosing it again shows the same stars. Changing the seed on New run is allowed (no anti-farming), as ADOM's endless re-rolls were.

## 2. Research: what players praise, tied to the decisions

ADOM assigns a birthday (a 360-day year of twelve 30-day months, as Warlock's) and the month's star sign at creation; the sign is shown on the first screen of creation and cannot be chosen. What each gave, and what the community made of it (ADOM Guidebook §0.2 and §1.1 "Starsigns", ancardia.fandom.com "Star sign"):

| ADOM sign | What it gave | Reputation | What it drives here |
|---|---|---|---|
| Raven | **+10 speed**, +2 Pe, messengers faster, companions stronger, harder to trick | "good for all PCs. It can save the life of low level PCs who need to run away" | the Salamander's speed and cheaper escape (§4); speed as the most-loved kind of gift |
| Book | +3 Le, one free skill increase per level, learns spells more easily, lawful tendencies | a caster's sign | the Sphinx's XP and lore; the skill-roll sign-limited talent (§9.1) |
| Wand | +2 Ch, +2 Ma and 10 % cheaper spells if neutral, lawful tendencies | "the sign for a neutral wizard" | elements as the caster-facing half: Water's mana (§5) |
| Unicorn | +2 Ap, harder to corrupt, lawful alignment hard to leave | "generic sign with resistance to corruption … attractive for an ultra ending" | the Unicorn's purity (disease and poison) and alignment nudges (§9.2) |
| Salamander | +1 Ch, **+3 Ma**, +20 % power points, fire magic 20 % cheaper | "very attractive for elementalists" | Fire as the aggressive element (Blaze) |
| Dragon | +2 St, +1 To, **−3 Wi**, weapon skills 10 % cheaper, tactics 10 % stronger | "the −3 to willpower is significant and makes this unattractive" | flaws are real and named (§4); Metal's weapon-skill marks |
| Sword | +1 Le, **melee weapon skills 20 % cheaper**, tactics bonuses +10 % | "attractive for a PC that really wants to become super-proficient with a weapon" | Metal's Forge-born and Keen edge (§5) |
| Falcon | +2 Wi, +1 Ch, Survival skill, one free talent | "Survival … generally considered as being of little use" | a skill grant is not significant; nothing here is only a skill |
| Cup | +2 Le, **10 % less XP to level**, a free skill advance every two levels, spells learned 20 % better | "they advance rapidly, which is very helpful in keeping them alive at low level" | the Sphinx's XP +10 % (§4) |
| Candle | **faster HP regeneration** (about 2 HP per 12 turns, stacking with race and Healing), gods more forgiving, one free talent | "an unbelievable sign for trolls … they seem to have natural regeneration" | the Hydra's Regrowth (§4): regeneration is the gift players remember |
| Wolf | +3 Pe, +3 Wi, food 25 % more nutritious | "significant benefits" for casters and low-Pe races | Wood's Evergreen (hunger) |
| Tree | +5 Wi, +2 To, **+1 PV**, alignment hard to change | "made for mindcrafters" | Earth's Stone hide: Protection (#192 is in) |

Other games, briefly (UESP "Oblivion: Birthsigns"; Skyrim's standing stones; Morrowind): The Elder Scrolls' thirteen signs mix flat bonuses (Warrior +10 St/End, Steed +20 Speed) with **once-a-day powers** (Ritual's heal, Shadow's invisibility, Serpent's poison touch, Tower's open-lock) and a few **gift-with-a-curse** signs (Atronach: +150 magicka, 50 % absorption, no regeneration; Apprentice: +100 magicka, 100 % weakness to magic; Lord: healing power, 25 % weakness to fire). Players remember the once-a-day powers and the Atronach's bargain, not the +10s.

**Findings tied to decisions**

| Finding | Decision |
|---|---|
| The loved signs changed a rule (speed, regeneration, XP, cheaper skills); the forgettable ones added a stat or a weak skill | every gift here is a rule change (§4–6); the stat adjustments stay but are not the point |
| ADOM's signs had flaws and players still took the game as it came; Oblivion's Atronach is the most discussed sign because of its curse | every animal has a flaw; the Phoenix's glow is a gift and a curse in one |
| Once-a-day powers give a sign a verb; ADOM's signs had none | four actives (Horn's grace, Transfixing stare, Blaze, and the Phoenix's rising, which triggers itself), limited by cooldown statuses, "time is the penalty" |
| "A sign for X": Wand for neutral wizards, Candle for trolls, Tree for mindcrafters: players enjoyed signs that made a race or class combination sing | the element gifts lean caster/finesse (Water, Wood) or fighter (Metal, Earth, Fire), so a sign–class match is a thing to hope for |
| ADOM shows the sign on the first screen, before any choice; a player could re-roll until they liked it | the reveal is a screen of its own, early (after the race); re-rolling is by changing the seed, not on the screen |
| ADOM's signs were by birth month, so every race could have every sign | Warlock's are by year; with the race's age range some signs are out of a race's reach at a given start date (§6.1). The owner's call keeps the year; the month scheme is the rejected option in §12 |

## 3. The model

### 3.1 What a sign gives

| Part | Gives | Count |
|---|---|---|
| The animal | two gifts and one flaw | 12 × 3 = 36 granted talents |
| The element | two gifts | 5 × 2 = 10 |
| A lucky year (five animal–element pairs) | one extra gift, unfair | 5 |
| Every sign | the stat adjustments as today (small, fixed) | unchanged |
| Later, by choice | sign-limited talents in the Stars tree, one per animal and element | 17 picks (§9.1) |

In all, 51 granted talents and 17 pickable ones. A character has 5 granted (6 in a lucky year).

### 3.2 Do the stat adjustments stay?

**Yes.** They are small, fixed numbers (no dice: the stat rolls of every existing seed are unmoved), they are already shown as data on the step screens and the wiki, and ADOM's signs had both stats and effects. Removing them would change every character's scores for no gain. They are no longer what a sign is about: the reveal screen lists them last.

### 3.3 How a gift is represented

The progression design (§7, §10 there; issue #229 "Talents core") gives everything needed:

| Need | Progression piece | Used here as |
|---|---|---|
| a passive modifier at an existing hook | a talent with a typed `effect`, summed into `TalentMods`, read where the affixes already are | every passive gift and flaw is a talent file in `data/talents/` with `tree = "stars"` |
| something given, not picked | `versatile`: "granted at creation" from `races.toml`'s `talents = [...]` | `grants = [...]` on an animal, an element, and a lucky year in `horoscope.toml`; `granted = true` on the talent keeps it off every pick list |
| an active | `A` talents in the abilities registry under `Source::Talent`, cooldowns as statuses (B1, #227) | the four actives register under a new `Source::Sign` ("Birth sign"), between Racial and Class |
| a prerequisite | `needs = { race = "dwarf", eyes = "darkvision", ... }` | three new kinds: `animal = "roc"`, `element = "fire"`, `lucky = true` (§9.1) |
| coverage | `systems = [...]` on every talent, the test | sign talents carry tags; the wiki's by-system table shows them in their own column (§9.4) |
| the page | the Talents info page (page 7) lists the character's talents by tree | the Stars tree's granted talents show first, marked "birth sign", flaws marked "flaw" |
| a flaw | — (talents are never negative) | a talent with `flaw = true`: the same effect kinds with negative values; shown with "but" on the reveal, dimmed on the page; never a prerequisite |

A flaw is a talent so that it rides the same hooks and the same aggregation; `flaw = true` only changes how it is shown and forbids it on pick lists and in prerequisites.

Rules that need the sign itself (alignment's starting nudge, §7's flavour, the `lucky` prerequisite) read `Character::sign` and `Content::lucky(sign)` as the flags `born_of_element`/`born_in_year_of` are read today.

## 4. The twelve animals

Notation: **Hook** is the one rule that reads it (names from CLAUDE.md and the progression design's §10.3); **A/P** active or passive; **Needs** what the gift waits for (blank: everything exists on `main`). Numbers are first guesses in data. In-game text is the talent's `description`, in words; the wiki shows the rule.

| Animal | id | Name | Effect | Hook | A/P | Needs |
|---|---|---|---|---|---|---|
| **Salamander** (first through the gate: quick and clever, if slight) | `fleet` | Fleet | you move and act a little faster (speed +5 %; the number is hidden, as speed is) | the scheduler's speed (`world/schedule.rs`, as `quick` reads it) | P | |
| | `slip_away` | Slip away | a step that takes you out of reach of every adjacent hostile costs a quarter less | `step_price`, with `World::flees`' adjacency test | P | |
| | `slight` | Slight *(flaw)* | carrying limit −10 % | `burden::load` | P | |
| **Hydra** (the strongest swimmer, tireless, of many minds) | `regrowth` | Regrowth | hit points come back half again as fast; Bleeding eases twice as fast | `World::healing_rate`, `Effect::until_eases` | P | |
| | `strong_swimmer` | Strong swimmer | wading costs you as a plain step, and wading never gives you a disease | `step_price`, `World::wade` | P | |
| | `many_minds` | Many minds *(flaw)* | −2 on Willpower checks | `check` callers that use Willpower (diseases today) | P | |
| **Manticore** (ran without rest, fierce, relentless, little loved) | `relentless` | Relentless | each blow at the same creature as your last gives +1 to hit, up to +3; anything else resets it | `Profile::melee` from a new `Actor::streak { target, count }` (saved, hashed) | P | |
| | `barbed_hide` | Barbed hide | a creature that hits you in melee takes 1 damage (not a blow: no roll, no XP, blood as usual) | `World::after_blow` on the player's side, `hurt` with a `Harm::Barbs` | P | |
| | `little_loved` | Little loved *(flaw)* | shop favour −10 % | `world::favour` | P | |
| **Unicorn** (graceful and fair, easily hurt) | `pure_blood` | Pure blood | +4 on disease checks; Poisoned eases twice as fast and never lowers your scores | `check` in `disease.rs`, `until_eases`, `Effects::penalty` | P | |
| | `horns_grace` | Horn's grace | **active**: your Bleeding and Poisoned end, and one disease that shows is shaken off; then **Spent** for a day, during which no grace | `World::cure(player, Means::Horn)`, a cooldown status; cost `wait` | **A** | B1 (#227) |
| | `thin_skinned` | Thin-skinned *(flaw)* | a big hit on you bleeds one degree more | `World::after_blow` (`wounded`) | P | |
| **Wyvern** (burned a path through the Mistwood: bold and generous, clumsy on the ground) | `trailblazer` | Trailblazer | overworld journeys a fifth quicker; forest, hills, and swamp cost you as plains do | `step_price` (`travel_minutes`) | P | |
| | `flame_hardened` | Flame-hardened | bursts of fire (fireball traps, a burning missile, Blaze) do you half damage; with #151, 25 % fire resistance instead | `cast_burst` victim hook; later `resistances` | P | |
| | `heavy_footed` | Heavy-footed *(flaw)* | footsteps a quarter louder | `noise::noise` | P | |
| **Basilisk** (patient and unblinking, trusted by no one) | `unblinking` | Unblinking | you notice hidden things in passing (as if trained in Spot), +2 | `search.rs::notices` | P | |
| | `transfixing_stare` | Transfixing stare | **active**: aim at a hostile within 6 cells in your line of sight that can see you (`sight`): it is **Transfixed**, its next two actions costing twice as much; then **Spent-eyed** for two hours | a creature status with `cost` (B1), aimed with the targeting mode (`InputMode::Aim`); silent; cost `wait` | **A** | B1 (#227) |
| | `trusted_by_none` | Trusted by none *(flaw)* | no creature makes way for you: you walk around | `World::refusal` | P | |
| **Roc** (the mightiest flier, powerful and proud, not one for details) | `rocs_strength` | Roc's strength | carrying limit +25 %; door kicks as if 4 stronger | `burden::load`, `kick_door` | P | |
| | `eyes_of_the_heights` | Eyes of the heights | you see two cells farther on the overworld by day, one by night | `daylight::overworld_sight` | P | |
| | `heedless` | Heedless *(flaw)* | searching −4 | `search_modifier` | P | |
| **Griffin** (watched over its gold from every hilltop: noble and watchful, stiff in its ways) | `hoard_sense` | Hoard sense | loot you find and creatures' drops roll magic half again as often (rare too); you find gems twice as often | `magic::loot::roll` weights, `gem::find` (the loot and drop streams: maps never change) | P | |
| | `watchful` | Watchful | you hear fainter sounds, and footsteps are placed exactly rather than roughly | `Actor::hearing` (−3 to the threshold; hidden), `PerceptionView::sounds` (`rough_place` → the cell) | P | |
| | `set_in_ways` | Set in its ways *(flaw)* | changing your stance costs a normal action (it is free today) | `price_of(Action::Stance)` | P | |
| **Sphinx** (set every racer a riddle: wise and cunning, slow to move) | `riddlers_mind` | Riddler's mind | XP +10 % | `grant_xp` | P | |
| | `old_lore` | Old lore | a magic item you have carried for a day identifies itself; reading a scroll costs half | `World::identify` from `perceive` (needs `Stack::carried_since`, as the `lore` talent does), the `read` price | P | |
| | `ponderous` | Ponderous *(flaw)* | you are a little slow (speed −5 %, hidden) | the scheduler's speed | P | |
| **Phoenix** (burned to ash at noon, rose at dusk: hard to keep down, frail between fires) | `rise_from_ashes` | Rise from the ashes | a blow, a trap, or a status that would kill you does not: you rise with a third of your hit points, flames burst from you (radius 1: 1d6 fire to every creature next to you, ice melts, `Event::Spell` with the `phoenix_fire` effect), and you are **Ashen** for a day (−2 to every score, hit points back half as fast, no second rising). Starving and choking still kill | `dies` (the one place an actor dies) checks it before `Death`; `World::shock(Heat)`; a status | P (triggers itself) | B1 (#227) for Ashen |
| | `inner_light` | Inner light | you shed a faint light (radius 1, a dim ember preset) wherever you are: you see the cell around you in the dark, and so does everything else | `LightCache::sources` adds a source at the player's cell (as `fires` do); `sight.rs` sees it like any light | P | |
| | `frail` | Frail *(flaw)* | maximum hit points −10 % (never below 1) | `Actor::max_hp` | P | |
| **Kraken** (swam the long way round: strong and deep, ungainly ashore) | `grasping_arms` | Grasping arms | both two-weapon penalties eased by 3; after a hit, Sword and Board's free shield bash is 20 more likely | `TwoWeapons::eased`, `World::second_blow` | P | |
| | `of_the_deep` | Of the deep | ice holds you as if you carried nothing; a plunge never hurts you; climbing out onto ice costs half; with #151, 25 % cold resistance | `World::tread`, `after_tread`, `tread_price`; later `resistances` | P | |
| | `ungainly_ashore` | Ungainly ashore *(flaw)* | overworld journeys a tenth slower | `step_price` | P | |
| **Barghest** (the black hound who ran last to see every racer through; guards the gate between the cycles) | `hounds_nose` | Hound's nose | you smell blood: bleeding creatures within 6 cells show even in the dark; you smell rot: the eat screen says whether a corpse is fresh, turning, or rotten | perception (statuses with `seen` through darkness, as `blood_scent`); `Stack::died` becomes a word for presentation (`Freshness`), never an age | P | |
| | `gatekeeper` | Gatekeeper | a hostile that steps next to you takes a free blow from you | `World::second_blow`'s triggers (C, #228) gain `Approach`, raised in `creature_steps` | P | C (#228) |
| | `grim_company` | Grim company *(flaw)* | shop favour −10 % | `world::favour` | P | |

Why these: each animal's gifts come from its line in the Great Race (the wiki's legend), so the reveal screen's story and its rules are one thing. Two flaws are the same rule (favour −10 %: the Manticore and the Barghest, the two the legend calls unloved); the rest are distinct.

## 5. The five elements

| Element | id | Name | Effect | Hook | A/P | Needs |
|---|---|---|---|---|---|---|
| **Wood** (bends and does not break: supple) | `supple` | Supple | Dodge +1; the burden's cap on Dexterity in Defence is 1 higher at every level | `Actor::defence`, `Burden::max_dex` | P | |
| | `evergreen` | Evergreen | hunger burns 15 % slower; rotten food harms you half as much | `World::digest`, `eat.rs::finish` | P | |
| **Fire** (warmth, light, passion) | `blaze` | Blaze | **active**: a burst of flame around you (radius 1): 2d6 fire to every creature next to you, ice melts, a burning missile's fire where it lands; then **Drained** for twelve hours | the scrolls' shared cast code (`cast_burst`) with a `blaze` effect in `effects.toml`, `World::shock(Heat)`; a cooldown status; cost `cast` | **A** | B1 (#227) |
| | `kindled` | Kindled | your lantern reaches one cell farther (a brighter preset); with #151, 25 % fire resistance | the lantern's preset (`lamplighter`'s hook); later `resistances` | P | |
| **Earth** (stone and soil: enduring) | `stone_hide` | Stone hide | Protection +1 | the protection sum (`combat::soak`'s caller) | P | |
| | `stonewise` | Stonewise | you notice what is hidden in stone in passing (as a dwarf does), and dig a quarter faster | `search.rs::notices` (`passive_search = stone`), `dig_work` | P | |
| **Metal** (iron and the forge: strong) | `keen_edge` | Keen edge | threat +1 with every weapon you wield (criticals come more often) | `Profile` (all kinds) | P | |
| | `forge_born` | Forge-born | weapon skills and fighting styles earn marks half again as fast (in words: you learn weapons quickly) | `fight.rs::resolve` → `practise` | P | |
| **Water** (always finds a way: clever) | `always_a_way` | Always a way | secret doors and hidden traps are 3 easier for you to find; searching reaches one cell farther | `search.rs::difficulty_of`, the search reach | P | |
| | `tidal` | Tidal | mana comes back a quarter faster | `World::mana_rate` | P | #223 |

An element's pair is one defensive or utility gift and one that fits a kind of character: Wood and Water for the light-footed and the clever, Earth and Metal for the fighter, Fire for anyone who wants a verb. Elements have no flaw.

## 6. The five lucky years

One animal per element, so every element has its legend, and the Great Race's own year is among them. A lucky year is `[[lucky]]` in the data: the pair, a name the game uses ("born in a lucky year: **the Racer**"), a line of legend, and one extra granted talent.

| Element | Animal | Name | Years (anchor 1; 60 apart) | id | The extra gift | Hook | Needs |
|---|---|---|---|---|---|---|---|
| Wood | Salamander | **the Racer** (the year of the Great Race itself) | 1, …, 1381, 1441 | `winners_luck` | a natural 2 on your attack roll is rolled again, once; and you are quicker still (speed +5 % more, 10 % with Fleet) | `combat::strike` (one gameplay draw only on a 2), the scheduler | |
| Fire | Phoenix | **the Firebird** | 22, …, 1342, 1402, 1462 | `reborn_whole` | rising from the ashes brings you back whole: full hit points, every status effect and disease burned away, and Ashen lasts half a day | `dies` (the rising), `World::cure` | B1 |
| Earth | Barghest | **the Gate Warden** | 48, …, 1368, 1428 | `warden_of_the_gate` | a hostile that steps away from beside you takes a free blow too (Opportunist, granted); Protection +1 more; status effects' penalties to your scores are halved | `second_blow` (`Flee` trigger, C), the protection sum, `Effects::penalty` | C |
| Metal | Manticore | **the Iron Scourge** | 39, …, 1359, 1419, 1479 | `scourge` | a melee blow that kills gives you a free blow at another adjacent hostile (Cleave, granted); your barbs do 1d4 | `second_blow` (`Kill` trigger, C), `barbed_hide`'s damage | C |
| Water | Kraken | **the Deep One** | 35, …, 1355, 1415, 1475 | `drowning_grip` | a creature you hit in melee is **Held**: it cannot step away from you until you miss it, strike another, or move | a creature status with `blocks = ["step"]` (B1), `creature_steps` refuses, the AI attacks instead | B1 |

### 6.1 How rare

Five of sixty signs: one character in twelve over a long enough run of births. But a sign is a year, and a race's age range picks the years, so at a given start date the chance differs by race. At the default start (11 Darkening 1444):

| Race (age) | Birth years | Lucky years in reach | Share of ages |
|---|---|---|---|
| Human (16–35) | 1409–1428 | Iron Scourge 1419, Deep One 1415, Gate Warden 1428 (a birthday on or before the start day) | about 3 in 20 |
| Half-orc (14–30) | 1414–1430 | 1415, 1419, 1428 | about 3 in 17 |
| Half-elf (20–60) | 1384–1424 | 1402 Firebird, 1415, 1419 | about 3 in 41 |
| Halfling (20–50) | 1394–1424 | 1402, 1415, 1419 | about 3 in 31 |
| Dwarf (40–120), gnome (40–150) | 1324–1404, 1294–1404 | all five (1342, 1355, 1359, 1368, 1381, 1402; the gnome also 1295, 1299, 1308, 1321) | about 1 in 13 (dwarf), 1 in 11 (gnome) |
| Elf (90–300) | 1144–1354 | all five, 16 years | about 1 in 13 |
| #255's kobold (6–20) | 1424–1438 | Gate Warden 1428 only | 1 in 15 |
| #255's goblin (10–30) | 1414–1434 | 1415, 1419, 1428 | about 3 in 21 |

So a human or a half-orc is likelier to be born lucky than an elf, and the Racer is for the long-lived races only, at this start date. That is the owner's "unfair", and the wiki says so in a sentence. Changing a scenario's `start_date` moves the table; a test pins the five years' signs, not who reaches them. (Had the owner wanted every race to reach every sign, the birth-month scheme in §12 would do it; it was not asked for.)

## 7. Moons and calendar flavour (optional)

Subtle, deterministic, pure functions of `GameTime` and `Character::born`; a `[flavour]` table in `horoscope.toml`, and none of it without the table. Recommended for a second PR (S7 in §11), after the gifts have been played.

| Flavour | Rule | Hook | Shown |
|---|---|---|---|
| Your birthday | hit points come back a quarter faster all day; at dawn, or on arrival, a note: "It is your birthday." | `World::healing_rate` reads `born.month, born.day == now`; `perceive`'s weather-news place for the note | the note; the character screen's age ticks over as today |
| Your birth month | your sign's active gifts recover in half the time (Spent, Spent-eyed, Drained, Ashen last half as long) | the one place a sign active applies its cooldown status | the status's duration, as statuses show |
| The full moon | on the night the moon is full (`MoonDef::phase_at` within a thirtieth of full, and the moon up: `Sky::moonlight > 0`), gifts with a reach reach two cells farther: Hound's nose 8, Transfixing stare 8, Eyes of the heights +2 more, Inner light radius 2 | each reach reads one `flavour::reach_bonus(time, sky)` | nothing new: the status bar shows the phase already |
| Your element's season (deferred) | Wood in Seedtide–Greenleaf, Fire in Highsun–Emberheat, Earth in Harvestide, Metal in Leaffall–Mistmoon, Water in Darkening–Thawmonth: the element's first gift is a little stronger | one read per element gift | the wiki's calendar page |

Why little: the owner's standing rule that effects stay subtle, and every one of these must be exact however time is split (the healing rate is already an event in `World::live`; a birthday boundary is one more moment, as a hunger state change is).

## 8. Character creation

### 8.1 The steps

New run (scenario, seed) → **Race** → **Born under the stars** (the reveal) → Gender → Class → Talents (#236, when it lands) → Name (#218) → Summary.

The reveal is a screen on the one stack (`ScreenName::Sign`, `ui/chargen_screen.rs::SignScreen`), opened by the race step's Enter (`Step::Race.next()`), and it opens Gender on Enter. It is not a `ChoiceScreen`: it has no rows, no Random, and no choice; `Picks` and `Draft::spec` are unchanged. The birth is the race's (`character/birth/<race>`, as today), so gender and class chosen afterwards never move it, and going back to Race and choosing the same race shows the same screen. `Step` gains `Sign` between `Race` and `Gender` so the titles count the choices only ("Choose your gender (2 of 3)" stays).

### 8.2 The screen

72 wide like the others, about 20 rows, so it fits 960×540.

```
┌ Born under the stars ──────────────────────────────────────────────┐
│                                                                    │
│  You were born on 3 Seedtide 1419, in the year of the Metal        │
│  Manticore. You are 25 years old.                                  │
│                                                                    │
│  The Manticore ran the whole course without rest while every       │
│  other racer fled its path: fierce and relentless, and little      │
│  loved. Metal is iron and the forge, hard and keen: strong.        │
│                                                                    │
│  A lucky year: the Iron Scourge. Once in sixty years the           │
│  Manticore is born in iron, and nothing stands before it.          │
│                                                                    │
│  Relentless     Every blow at the same foe lands more surely.      │
│  Barbed hide    Those who strike you are cut by your barbs.        │
│  but Little loved   Shopkeepers think less of you.                 │
│  Keen edge      Your weapons bite deep: criticals come oftener.    │
│  Forge-born     You learn weapons quickly.                         │
│  Iron Scourge   A killing blow carries on into the next foe, and   │
│                 your barbs cut deeper.                             │
│  St +2  To +1  Pr −1                                               │
│                                                                    │
│  Enter continue   Esc back                                         │
└────────────────────────────────────────────────────────────────────┘
```

| Line | Source |
|---|---|
| the date, the sign, the age | `Start::character` as the summary computes it (`CharacterSpec::start` with the draft's race and any gender and class: the birth does not depend on them), `birth_parts` |
| the animal's and the element's lines | `SignPart::description` (the Great Race lines already in the data) |
| the lucky year | `[[lucky]]`'s `name` and `description`; in the accent colour |
| the gifts | the granted talents' `name` and `description`, flaws prefixed "but" and dimmed; the lucky gift last, in the accent colour |
| the adjustments | `adjustments()` as the race and class steps print them: data, not modifiers |
| the footer | `Hints::of(bindings)`: the menu context's Confirm and Back, rebindable as every key is; no new command |

Never on it: the word "seed", an age range, a year's odds, a modifier, a speed. The sentence "You do not choose your sign; the stars did" is the screen's subtitle if the panel has a spare row, else left to the wiki.

### 8.3 After the reveal

| Where | What |
|---|---|
| The summary | a `Sign  Metal Manticore` row between Race and Gender (`SUMMARY_ROWS`), Enter on it `ScreenAction::Return(ScreenName::Sign)` to read the gifts again; the birth line stays. A lucky year is said in the row's value: `Metal Manticore (the Iron Scourge)` |
| The character screen (`@`) | the birth line as today plus `(a lucky year: the Iron Scourge)` when it is one; the gifts are on the Talents page (page 7, #236) under **Stars**, first, each marked "birth sign", flaws dimmed; until #236 lands, a `Sign gifts` line on the character screen names them |
| The abilities menu | the actives under **Birth sign** (`Source::Sign`), greyed with their cooldown's reason ("you are spent") |
| Messages | "You feel the stars' favour." is not said: the reveal already said it. Gifts speak through their rules' messages (Rise from the ashes: "You burst into flame and rise from your ashes!") |
| Dumps | the `stats` layer gains `Sign Metal Manticore (lucky: the Iron Scourge)` after the class; the `talents` layer (#229) marks granted sign talents `(sign)`; JSON gets `sign` and `lucky` beside `character` |
| `--screen sign` | opens the reveal over New run and Race on the race preselected (the scenario's character's), like `race` and `summary`; a golden manifest entry `chargen-sign` beside `race` and `summary` |
| `--character` | unchanged: `RACE,GENDER,CLASS`; the sign follows |

### 8.4 Wording

With #256: the Random rows say "Let fate pick your race."; the gender "other" names no stat. Here: the reveal, the wiki's Horoscope page ("the stars decide your sign"), and the summary never mention the seed; the wiki's sentence "Both come from the run's seed, so the same seed and race always give the same age and birthday" becomes "The same race always has the same stars in the same world" or goes. The New run screen keeps its seed field: it is a world's name, not an explanation of randomness.

## 9. Interactions

### 9.1 Talents

**Prerequisite kinds** (added to the progression design's §7.3 table):

| Kind | Written as | Read from | Shown |
|---|---|---|---|
| the animal | `animal = "roc"` | `Character::sign` | yes ("born in the year of the Roc") |
| the element | `element = "fire"` | `Character::sign` | yes ("Fire-born") |
| a lucky year | `lucky = true` | `Content::lucky(sign)` | yes ("born in a lucky year") |

**The Stars tree** holds the 51 granted talents (never offered) and these **sign-limited picks**, offered on the level-up screen and the creation Talents step like any talent whose prerequisites hold; the tree needs no prerequisite-free talent since it is never a bonus-pick tree (the coverage test's `free_per_tree` exempts `granted`-only trees, or the Stars tree is listed as `no_free = true`):

| id | Name | Needs | Effect | Hook |
|---|---|---|---|---|
| `hare_footed` | Hare-footed | Salamander, Lv 12 | quicker still (speed +5 % more) | the scheduler |
| `another_head` | Another head | Hydra, Lv 9 | Regrowth works while Starving, and twice as fast while resting | `healing_rate` |
| `relentless_fury` | Relentless fury | Manticore, Lv 9 | the streak goes to +5, and every two steps add +1 damage | `Profile::melee` |
| `horn_of_plenty` | Horn of plenty | Unicorn, Lv 9 | Horn's grace also heals 1 HP per level, and Spent lasts eight hours | the active |
| `fire_breath` | Fire breath | Wyvern, Lv 15 | **active**: a bolt of fire 4 cells long (2d6), then Drained | `cast_bolt`, the `fire_breath` effect |
| `petrifying_gaze` | Petrifying gaze | Basilisk, Lv 12 | the stare holds for four actions and Spent-eyed lasts an hour | the active |
| `rocs_grip` | Roc's grip | Roc, St 14 | carrying limit +25 % more; Burdened costs you nothing | `burden::load`, `Load::penalty` |
| `dragon_hoard` | Dragon's hoard | Griffin, Lv 12 | Hoard sense's odds half again, and shop favour +10 % | the loot weights, `favour` |
| `riddle_of_the_sphinx` | Riddle of the Sphinx | Sphinx, In 14 | +1 skill roll every level | §5 of the progression design (E, #230) |
| `twice_risen` | Twice risen | Phoenix, Lv 18 | Ashen no longer stops a second rising (it still weakens you) | `dies` |
| `tentacles` | Tentacles | Kraken, twin_blades 3 | off-hand blows +2 damage | `Actor::off_hand_blow` |
| `hounds_bay` | Hound's bay | Barghest, Lv 12 | **active**: a howl; hostiles within hearing whose level is below yours flee for three turns, then Hoarse | a creature status `frightened` the AI reads as the coward stance (B1); `noise::Act::Howl` |
| `deep_roots` | Deep roots | Wood, Lv 9 | Dodge +1 more; rotten food never harms you | `defence`, `eat.rs` |
| `wildfire` | Wildfire | Fire, Lv 15 | Blaze does 3d6 out to radius 2 | the active |
| `bedrock` | Bedrock | Earth, Lv 12 | Protection +1 more; a pit never catches you | the protection sum, `spring_trap` |
| `tempered` | Tempered | Metal, a weapon skill at 5 | threat +1 more with your highest weapon skill | `Profile` |
| `undertow` | Undertow | Water, Lv 6 | wading costs you as a plain step; a plunge through ice never hurts you | `step_price`, `after_tread` |

Signs also work as prerequisites on talents outside the tree where it reads naturally (the progression's `sun_blessed` could ask `element = "fire"`; not proposed now: keep the first list clean).

### 9.2 Classes, mana, resources, alignment, names, evil races

| System | Interaction |
|---|---|
| Class tables (#243–#246) | none: classes carry identity through their tables; a sign is fate on top. No class reads a sign |
| Mana (#223) | Water's `tidal` (+25 % regeneration); the Sphinx's `riddle_of_the_sphinx` could instead be mana if the owner prefers (§12). Nothing else; said so in #223's terms |
| Alternative resources (#225) | none now. Later, a resource's `max` could read a sign ("a Fire-born barbarian's rage starts 2 higher"); not proposed |
| Alignment (#254) | a sign **nudges the starting score** (`alignment` on an animal, in data; elements none), added to the race's and class's before the band is shown on the summary: Unicorn +100, Griffin +50, Phoenix +50, Hydra 0, Salamander 0, Wyvern +50 (generous), Roc 0, Sphinx 0, Kraken −50, Basilisk −100, Manticore −100, Barghest 0 (grim, but loyal). Small against the bands' width; never shown as a number |
| Names (#218) | the Name step comes after Class; the reveal says "You were born on …" without a name, and the summary and death screen gain the lucky year's title after the name if the owner likes it ("Brakka, the Iron Scourge") |
| Evil races (#255) | nothing but §6.1's reach: short-lived races see fewer signs at a start date. The kobold's one lucky year is the Gate Warden |
| Protection (#192, in) | Earth's `stone_hide`, the Gate Warden; the wiki's armour table already explains Protection |
| Damage types (#151) | `flame_hardened`, `kindled`, `of_the_deep` switch from their burst-only form to resistances when #151 lands (the same talent file gains a `resistances` effect; the burst halving goes) |

### 9.3 The coverage matrix

Sign talents carry `systems` tags like every talent, and the wiki's by-system table (progression §8.2) gains a **Birth sign** column. The matrix rows this design adds to:

| System | Sign gifts |
|---|---|
| Combat: to-hit, damage, criticals | relentless, keen_edge, scourge, winners_luck, tentacles, tempered |
| Free blows | gatekeeper, scourge, warden_of_the_gate |
| Defence categories | supple, deep_roots |
| Protection | stone_hide, bedrock, warden_of_the_gate |
| Weapon skills and marks | forge_born |
| Fighting styles | grasping_arms, tentacles |
| Stances | set_in_ways |
| Hit points, healing | regrowth, another_head, rise_from_ashes, reborn_whole, frail |
| Status effects | pure_blood, horns_grace, thin_skinned, transfixing_stare, drowning_grip, hounds_bay, the cooldowns |
| Diseases and checks | pure_blood, strong_swimmer, many_minds, horns_grace |
| Hunger, food | evergreen, hounds_nose (rot), deep_roots |
| Burden | slight, rocs_strength, rocs_grip |
| Stealth and noise | heavy_footed, inner_light (you are seen), hounds_bay |
| Hearing | watchful |
| Light | inner_light, kindled |
| Vision, overworld sight | eyes_of_the_heights, hounds_nose |
| Traps | always_a_way, bedrock |
| Searching | unblinking, stonewise, always_a_way, heedless |
| Doors | rocs_strength |
| Digging | stonewise |
| Ice | of_the_deep, undertow |
| Wading | strong_swimmer, undertow |
| Overworld travel | trailblazer, ungainly_ashore |
| Weather, sun, moon | the §7 flavour |
| Shops and favour | little_loved, grim_company, dragon_hoard |
| NPCs, displacement | trusted_by_none |
| Magic items, identification | old_lore, hoard_sense |
| Scrolls and the cast code | old_lore, blaze, fire_breath, flame_hardened |
| Gems | hoard_sense |
| Skills | riddle_of_the_sphinx |
| Experience | riddlers_mind |
| Speed (hidden) | fleet, ponderous, hare_footed, winners_luck |
| Mana (#223) | tidal |
| Damage types (#151) | flame_hardened, kindled, of_the_deep |

**The horoscope is not a system tag of its own**: the test wants two talents and a class ability per tag, no class touches the stars, and the §7 flavour is three reads, not a system. The by-system table shows the sign column so a gap is visible, which is the policy's point.

### 9.4 Wiki

The Horoscope page keeps the legend and the cycle and gains, generated by `cargo xtask wiki` (`xtask/src/wiki/horoscope.rs`): `horoscope-animals.md` (animal, stats, gifts and flaw as rules), `horoscope-elements.md`, `horoscope-lucky.md` (the five, their years in the current cycles, who can reach them at the default start), `horoscope-talents.md` (the sign-limited picks), and the flavour table. The Talents page's by-tree table includes Stars. As everywhere, hidden rules stay in words (speed, hearing, marks).

### 9.5 Hidden numbers

| Gift | In the game | In the wiki |
|---|---|---|
| fleet, ponderous, hare_footed, winners_luck | "a little faster" / "a little slow" | the same words (speed is never shown) |
| watchful | "you hear fainter sounds" | the same (hearing thresholds are hidden) |
| forge_born | "you learn weapons quickly" | the same (marks are hidden) |
| pure_blood, many_minds | "you shake off disease more easily" / "your will is divided" | +4 / −2 (the wiki shows check modifiers in words today: "the chance an average score resists") |
| supple, stone_hide, rocs_strength, riddlers_mind, favour flaws | the number ("Dodge +1", "Protection +1", "+25 % carrying", "XP +10 %", "favour −10 %") | the number |
| relentless | "lands more surely" | +1 per blow to +3 |
| keen_edge | "criticals come more often" | threat +1 (threat ranges are on the combat page) |

Never a creature's anything: the Transfixing stare says "The kobold freezes under your gaze." and nothing of its Willpower; Hound's nose shows a bleeding creature, not its hit points.

## 10. Data and code sketch

### 10.1 Data

`data/horoscope.toml` stays one file (the cycle is settings and twelve plus five rows; the gifts churn, and they live in the talents directory):

```toml
anchor_year = 1

# Optional (§7). Without it, no flavour.
[flavour]
birthday_healing = 125      # percent of the healing pace on your birthday
birth_month_cooldown = 50   # percent of the sign actives' cooldowns in your birth month
full_moon_reach = 2         # cells added to gifts with a reach under a full moon

[[animal]]
id = "manticore"
name = "Manticore"
description = "Ran the whole course without rest ..."
stats = { strength = 1, toughness = 1, presence = -1 }
grants = ["relentless", "barbed_hide", "little_loved"]   # talents in the stars tree, granted = true
alignment = -100                                         # #254: added to the starting score

[[element]]
id = "metal"
name = "Metal"
description = "Iron and the forge, hard and keen: strong."
stats = { strength = 1 }
grants = ["keen_edge", "forge_born"]

[[lucky]]
animal = "manticore"
element = "metal"
name = "the Iron Scourge"
description = "Once in sixty years the Manticore is born in iron, and nothing stands before it."
grants = ["scourge"]
```

`data/talents/<id>.toml` (the directory #229 creates), one per gift:

```toml
[[talent]]
id = "barbed_hide"
name = "Barbed hide"
tree = "stars"
granted = true                    # never on a pick list; given by what names it in grants
description = "Those who strike you in melee are cut by your barbs."
effect = { kind = "barbs", damage = "1" }
systems = ["combat", "status"]

[[talent]]
id = "little_loved"
name = "Little loved"
tree = "stars"
granted = true
flaw = true                       # shown with "but", dimmed; never a prerequisite
description = "Shopkeepers think less of you."
effect = { kind = "favour_percent", value = -10 }
systems = ["shops"]

[[talent]]
id = "relentless_fury"
name = "Relentless fury"
tree = "stars"
needs = { animal = "manticore", level = 9 }
description = "Your streak runs to five, and every second blow bites harder."
effect = { kind = "streak", max = 5, damage_every = 2 }
systems = ["combat"]
```

`data/talents.toml`: the tree `[[tree]] id = "stars", name = "Stars", description = "What you were born under."`, listed first. `data/status.toml`: `spent`, `spent_eyed`, `drained` (if #227 has not added it), `ashen` (penalty −2 all, `regen` 50), `transfixed` (creature, `cost` 200), `held` (creature, `blocks = ["step"]`), `frightened` (creature), `hoarse`. `data/effects.toml`: `phoenix_fire`, `blaze`, `fire_breath` (looks only). `data/lights.toml`: an `ember` preset for Inner light.

**Validation** (`content/horoscope.rs`, after talents load): every `grants` id exists, is in the `stars` tree, and is `granted`; a `flaw` is only among an animal's grants; `[[lucky]]` pairs name an existing animal and element, no pair twice; `alignment` within the band scale; `[flavour]` percents 1 to 400. `Content::lucky(&Sign) -> Option<&Lucky>`; `Horoscope::grants(&Sign) -> Vec<TalentId>` (animal, element, lucky, in that order: the order on the reveal and the page).

### 10.2 State

| Where | Change |
|---|---|
| `Character` | none: `sign` stays; lucky is `Content::lucky(sign)` |
| `Actor::talents` (#229) | the sign's grants are **not stored**: computed on load and at creation from `Character::sign`, as #229 computes the class's entries, so adding a gift to the data gives it to a saved character and no save breaks |
| `Actor::streak: Option<Streak { target: ActorId, count: u8 }>` | new, for Relentless; saved and hashed; `FORMAT_VERSION` +1 (left out of the JSON while `None`, so hashes of runs without it hold) |
| `Stack::carried_since: Option<GameTime>` | #229's `lore` needs it too; whichever lands first adds it |
| cooldowns, Ashen, Transfixed, Held | statuses (B1), saved as statuses are |
| Inner light | derived: a source at the player's cell each perception; no state |
| `Progress::once` | not needed: the Phoenix's limit is Ashen |

### 10.3 Hooks

One read each, at the hooks the progression design already names (§10.3 there); new in this design are marked **new**:

| Hook | Reads |
|---|---|
| the scheduler's speed | fleet, ponderous, hare_footed, winners_luck |
| `step_price` | slip_away (**new**: an "out of reach" test next to `flees`), strong_swimmer, trailblazer, ungainly_ashore, undertow |
| `burden::load`, `Load::penalty` | slight, rocs_strength, rocs_grip |
| `World::healing_rate` | regrowth, another_head, Ashen's `regen`, the birthday |
| `Effect::until_eases`, `Effects::penalty` | regrowth (bleeding), pure_blood, warden_of_the_gate |
| `Profile` builders | relentless (+ `Actor::streak`), keen_edge, tempered, tentacles |
| `World::resolve` / `after_blow` | **new** barbs on the player's side (`Harm::Barbs`); thin_skinned; the streak's update; forge_born in `practise`; drowning_grip applies Held |
| `World::second_blow` (C) | gatekeeper (**new** `Approach` trigger from `creature_steps`), scourge (`Kill`), warden_of_the_gate (`Flee`) |
| `combat::strike` | winners_luck (reroll a natural 2 once: one gameplay draw only then) |
| `dies` | **new**: rise_from_ashes before `Death` is set (not for `Cause::Starvation`/`Choked`): hp to a third, `World::shock(Heat)` radius 1, `Event::Spell`, afflict Ashen; reborn_whole and twice_risen read here |
| `World::cure` | horns_grace (`Means::Horn`), reborn_whole |
| `check` (disease) | pure_blood, many_minds |
| `World::wade`, `tread`, `tread_price`, `after_tread`, `spring_trap` | strong_swimmer, of_the_deep, undertow, bedrock |
| `search.rs` (`notices`, `search_modifier`, `difficulty_of`, reach) | unblinking, stonewise, heedless, always_a_way |
| `kick_door`, `dig_work` | rocs_strength, stonewise |
| `daylight::overworld_sight` | eyes_of_the_heights (+ the full moon) |
| `Actor::hearing`, `PerceptionView::sounds` | watchful |
| `noise::noise` | heavy_footed, hounds_bay's howl |
| `price_of(Action::Stance)` | set_in_ways |
| `grant_xp` | riddlers_mind |
| `World::identify` from `perceive`, the `read` price | old_lore |
| `magic::loot::roll`, `gem::find` | hoard_sense, dragon_hoard |
| `World::refusal` | trusted_by_none |
| `world::favour` | little_loved, grim_company, dragon_hoard |
| `World::digest`, `eat.rs::finish` | evergreen, deep_roots; **new** `Freshness` word for hounds_nose on the eat screen |
| perception (`seen` statuses in the dark) | hounds_nose (as `blood_scent`) |
| `LightCache::sources`, `sight.rs` | **new**: inner_light's source at the player's cell (the creature-sight invariant test covers it: both read the same cache) |
| `cast_burst` / `cast_bolt` (the scrolls' shared code) | blaze, fire_breath (casts), flame_hardened (victim) |
| `Actor::defence`, `Burden::max_dex`, the protection sum, `Actor::max_hp` | supple, deep_roots, stone_hide, bedrock, warden_of_the_gate, frail |
| the lantern's preset | kindled |
| `World::mana_rate` (#223) | tidal |
| the abilities registry | `Source::Sign`; `Ability::Talent(id)` for horns_grace, transfixing_stare, blaze, fire_breath, hounds_bay; `Unavailable::While(status)` from B1 |
| alignment's start (#254) | the animal's `alignment` |

New modules: `content/horoscope.rs` grows (grants, lucky, flavour, validation); `world/stars.rs` (the rising, the barbs, the streak, the flavour's reach and cooldown reads: the sign-specific rules that are not one line at a hook); `ui/chargen_screen.rs::SignScreen`; `xtask/src/wiki/horoscope.rs`.

### 10.4 Determinism

| Draw | Stream | Note |
|---|---|---|
| the birth (age, month, day) | `character/birth/<race>` | **unchanged**: existing seeds keep their dates and signs |
| the stat adjustments | `character/rolls/<spec>` | unchanged: signs stay fixed numbers and draw nothing |
| nothing else at creation | — | grants are a function of the sign; the reveal draws nothing; a sign is never re-rolled |
| winners_luck's reroll | gameplay, only on a natural 2 | a run without it draws nothing extra (the `lucky` talent's rule) |
| the Phoenix's burst damage, Blaze, Fire breath | gameplay, through the cast code as a scroll's burst | only when they happen |
| drowning_grip, transfixing_stare, hounds_bay | no draw (no check; the owner's unfair) | |
| hoard_sense, dragon_hoard | the `loot/<level>` and `drop/<actor>` streams | the map never changes; a level generated before is kept |
| the flavour (birthday, month, moon) | none: functions of `GameTime`, `Sky` | exact however time is cut: the birthday boundary is a moment in `World::live` |
| cooldowns | game time (statuses) | |

Replays record the spec only, as today; a run is reproduced by scenario, seed, and character. Content changes to the gifts change hashes of runs that use them, as any rule change does; a run of a character whose gifts are all untouched hashes as before, which a test checks for a sign with no `grants` (fixture content).

## 11. Implementation plan

Issue-ready items; each one branch and PR with spec (`spec/08i-character.md` "Birth sign", `spec/07e-characters.md` the data), `docs/dev/character-creation.md`, CLAUDE.md, and the wiki updated in the same PR. Sizes S (a day), M (two to three). Dependencies on the progression issues: D = #229 (talents core), B = #227 (statuses as buffs and cooldowns), C = #228 (free blows), E = #230 (skill rolls), G = #236 (screens and the Talents step).

| # | Issue title | Implements | Depends on | Size | Parallel with |
|---|---|---|---|---|---|
| **S1** | Horoscope: gifts as granted talents in the Stars tree, lucky years, the `animal`/`element`/`lucky` prerequisites, the reveal step after the race, `--screen sign`; the first passives that need only existing hooks (fleet, slight, regrowth, strong_swimmer, pure_blood, rocs_strength, heedless, riddlers_mind, ponderous, little_loved, grim_company, supple, evergreen, stone_hide, stonewise, keen_edge, always_a_way, winners_luck) | §3, §6, §8, §10 | D | M | #256 (wording), H1–H2 |
| **S2** | Sign gifts: Salamander to Basilisk (slip_away, many_minds, relentless with `Actor::streak`, barbed_hide, horns_grace, thin_skinned, trailblazer, flame_hardened, heavy_footed, unblinking, transfixing_stare, trusted_by_none) | §4 | S1; B for the two actives | M | S3, S4 |
| **S3** | Sign gifts: Roc to Barghest (eyes_of_the_heights, hoard_sense, watchful, set_in_ways, old_lore, rise_from_ashes, inner_light, frail, grasping_arms, of_the_deep, ungainly_ashore, hounds_nose, gatekeeper) | §4 | S1; B for Ashen; C for gatekeeper | M | S2, S4 |
| **S4** | Element gifts: blaze, kindled, forge_born, tidal (with #223) | §5 | S1; B for Blaze | S | S2, S3 |
| **S5** | Lucky gifts: reborn_whole, scourge, warden_of_the_gate, drowning_grip (Held) | §6 | S2, S3; B, C | S | S6, S7 |
| **S6** | Sign-limited talents: the seventeen Stars picks | §9.1 | S1–S4; G (the pick lists); E for riddle_of_the_sphinx | M | S5, S7 |
| **S7** | Calendar and moon flavour: the birthday, the birth month, the full moon | §7 | S2, S3 | S | S5, S6 |
| **S8** | Alignment nudges by sign | §9.2 | S1, #254 | S | any |
| **S9** | Wiki and docs: the Horoscope page's generated tables, the by-system Birth sign column, the reach-by-race table | §9.4 | S1 (grows with each) | S | with each PR; a closing pass |

**Waves**: S1 right after D (#229) merges; then S2, S3, S4 together (three agents; each owns its hook files: S2 `fight.rs`'s profile and `after_blow`, `talk.rs`, `search.rs`; S3 `dies`, light, perception, `loot.rs`, `second_blow`; S4 `spell.rs`, the lantern preset, `practise`); then S5, S6, S7, S8 together. S1 is the one PR that touches `chargen_screen.rs`, so it runs beside H1/H2 of the progression waves, not beside G (#236), which also edits that file; S6 waits for G.

**Tests each item carries**: `character.rs` (grants per sign, the lucky five pinned by year, nothing moved for existing seeds: a fixture of ten seeds' `Start`s before and after), `content/horoscope.rs` (validation), the rule tests beside each hook (`world/stars_tests.rs`), screen tests for `SignScreen` (text, Enter, Esc, the summary's Return), a replay per active (`replay --create` then `--verify`), the creature-sight invariant with Inner light, a golden entry `chargen-sign` (advisory under #166), `cargo xtask wiki --check`.

## 12. Open questions, each with a recommendation

| # | Question | Recommendation |
|---|---|---|
| 1 | **Keep the stat adjustments?** | Yes (§3.2): small, fixed, already shown; removing them changes every character for nothing. |
| 2 | **Flaws on animals?** The owner asked for abilities, not drawbacks. | Yes: one per animal, none per element. ADOM's Dragon had one and the Atronach is TES's most-remembered sign; fate should be uneven, and the owner wants unfair. If no, the twelve flaws are twelve files to delete and the reveal loses its "but" line. |
| 3 | **Which five are lucky?** | One per element, the Great Race's year among them (§6). If the owner would rather every race could be born lucky at the default start, swap the Firebird (1402) and the Racer (1381/1441) for years inside 1409–1428 (a human's), e.g. the Fire Griffin (1412) and the Wood Roc (1411); the names and extra gifts move with them. |
| 4 | **Signs out of a race's reach** at a start date (a kobold has one lucky year, an elf all five, a human never the Racer). | Accept and say so on the wiki; it is the kind of unfair asked for. The alternative, animals by birth month and elements by year (the rejected option of the first brief), gives every race every sign but changes every existing character's sign and makes the animal depend on a month that has no legend yet. Not recommended unless the owner wants every race to reach every sign. |
| 5 | **Re-rolling a sign by retyping the seed on New run.** | Allowed, unhinted: the no-anti-farming rule, and ADOM allowed it. The reveal screen itself never re-rolls. |
| 6 | **Actives' limits: cooldown statuses (B1, #227) or a per-day flag?** | Statuses, as the progression decided ("cooldowns are statuses"); S2–S5 wait for #227, which is wave 1 there. A per-day flag would be a second mechanism. |
| 7 | **Should the Phoenix's rising be once a day or once a run?** | Once a day (Ashen), with Twice risen at level 18 lifting it. Once a run (ADOM's amulet of life saving) would make the sign a one-use item; a day keeps "time is the penalty". |
| 8 | **The Sphinx's XP +10 %**: it is the one gift that shortens the game. | Keep; ADOM's Cup had exactly this and was liked for keeping low-level casters alive. If the owner prefers, swap with +1 skill roll every level (then `riddle_of_the_sphinx` becomes mana). |
| 9 | **Hidden-number flaws** (Ponderous: speed −5 %). | Keep, in words ("a little slow"); the game hides speed both ways, and the wiki says the same words. |
| 10 | **Trusted by none** (no one makes way for the Basilisk-born) in the crowded hub. | Keep: it is friction, not harm, and it is the first flaw a player will feel and remember. If it annoys in play, soften to "only peaceful creatures of your size or smaller make way". |
| 11 | **Where the gifts show in the game before the Talents page (#236) exists.** | A `Sign gifts` line on the character screen (names only) until #236; then the Stars section of the Talents page, and the line goes. |
| 12 | **The §7 flavour at all?** | Ship S7 after the gifts have been played a week; it is three small reads and a data table, and it can wait. |
| 13 | **Does the lucky title go on the character and death screens with the name (#218)?** | Yes, after the name: "Brakka, the Iron Scourge". Cheap and it feels like ADOM's titles. |

# Warlock — The bard: a jack of all trades, mediocre at everything

Design document for the owner's review (issue #819; first draft 2026-11-02, **second round
2026-11-02** after the owner's answers on PR #835; the third round, 2026-11-02, changed only
`prestige-classes.md`: the calling is now the oath said to the Strange Spirit, and the Hunter's
first level comes by a level-up at total 31). Written against `origin/main` at 38c5085d
(Allies A1–A3, the summon spell kind, saves pinned to the game version, #840; the #776 review's
cull, #798, emptied the bard's old 40 and 50; Lore is gone from the game, #827). Nothing here is
implemented; §11 cuts it into issues. It builds on what is there: the bard's class file
(`assets/data/classes/02-bard.toml`), its class abilities (`data/talents/`), the Inspiration pool
(`docs/dev/resources.md`), skills and their level-up rolls (`docs/dev/skills.md`), weapon skill
ranks (`docs/dev/weapon-skills.md`), statuses (`docs/dev/status-effects.md`), hearing
(`docs/dev/hearing.md`), the one cast path (`docs/dev/spells.md`), class tables and the coverage
policy (`docs/dev/progression.md`, rules 1–9 of #776, which bind every proposal below), and the
level curve (`docs/dev/experience.md`). Every number is data and hidden in the game; the wiki
states it (the owner's rule: the game is vague, the wiki is accurate).

**The second round changes the shape of the class.** The owner (PR #835, verbatim): *"Bard gains
all these abilities by level 30. Bard does not gain levels above 30. With certain conditions, a
character can start advancing in another class. this is not going to be general multiclass thing.
More like prestige classes achievable by finding a "hidden" mechanism. Also implement one
prestige class for bard. condition: lvl30, certain event has happened. Call this class "hunter".
it will be a martial class with 30 levels. bard retains all the abilities when it starts
progressing in that class. XP requirements will be based on total level. Extend the XP tables all
the way to 100 - we are not going there yet, but who knows, maybe we will. This might make "bard"
pretty op, but who cares."* The prestige system, the Hunter, its calling in the Ascmearc, and the
curve to 100 are **`docs/design/prestige-classes.md`** (the same PR); this document keeps the
bard: its table compressed into levels 1 to 30, its cap, and the first round's answers applied.

**The owner's first request** (2026-11-02, verbatim): *"bard: create a design issue to come up
with class fantasy for the class. I mean leaning towards mediocre in everything."*

## Contents

1. Summary
2. The fantasy in one paragraph, and how the data already says it
3. Research: bards and jacks of all trades elsewhere, tied to decisions
4. What exists today
5. The table proposed: levels 1 to 30, and the cap
6. Songs: Music, Inspire reworked, Lullaby, Virtuoso
7. Jack of all trades in two ranks, and class tables that name a rank
8. Travelled hands
9. What the player sees; determinism, saves, replays
10. The wiki, the spec, and the docs
11. Implementation plan
12. Rejected ideas, with the reason
13. Decisions: the owner's answers, applied
14. Open questions left, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **The fantasy** | the bard knows a little of everything and masters nothing: a rank in every weapon and style and none above the middle, small spells, a lock picked badly, a scratch closed, and the one thing that is theirs alone, the song. Breadth is the power; every single thing a bard does, a specialist does better. §2 |
| **The table** (levels 1 to 30, one facet each; **the bard caps at 30**) | 1 Mend, Trained wind (stay; Lore is gone, #827); 3 **Travelled hands** (new; Blessing goes); 6 Fast healer (stays); 9 Inspire (stays, reworked as a song); 12 **Jack of all trades rank 1** (every skill taken up); 16 Throw your voice (stays); 20 **Lullaby** (new); 25 **Jack of all trades rank 2** (every skill +5); 30 Virtuoso (stays, widened). Bonus picks at 5, 10, 15, 23, 28 **from any tree**. §5 |
| **The cap** | `max_level = 30` in the bard's class file (`ClassDef::max_level`, default the game's 60): a bard gains no level past 30, as no one gains one past 60 today; XP keeps counting and buys nothing until a **calling** opens a prestige class (the Hunter), when the levels banked come at once (`prestige-classes.md` §4.3). §5 |
| **Songs** | the Music skill, which no rule reads today, becomes the bard's craft: a `song` talent effect (Inspire's `inspire` kind generalised), **each song paid in one thing** (Inspire 3 Inspiration, the Lullaby 6 mana; no "X else Y", Q6), in a chat's time, **heard** as a voice (`SONG_NOISE` 80: those who hear it come to look; who hears is the world's, not the bard's perception, Q15); Music's value adds degrees to what a song gives (+1 at 50, +1 at 100, +1 per 100 after) and, for the Lullaby, makes the hearers' check harder (+4 at 100, +8 at 200). **No instrument bonus: the bard always sings** (Q8). Music's `kind` becomes `rules`. §6 |
| **Lullaby** (20) | a song over everyone who hears it: each hostile hearer checks Willpower against 13 plus the Music craft (**no level gate**, Q5); failed, it is **Drowsy** (the status the Sleep spell gives: everything three times as slow), one degree plus Music's, at most three; made, it only knows where the bard is. 6 mana, a chat's time, no cooldown; sung to an empty room all the same. §6.3 |
| **Jack of all trades** (12, 25) | rank 1 at 12: **you take up every skill you lack**, generically, whatever skills the content has now or later (`1d4+1` each from the levels stream), **but Sneak attack** (the thief's: the surprise design's decision 7, kept, Q3). Rank 2 at 25: **every skill counts 5 higher** (today's `trained_skills 5`, moved, Q4). The class table names the rank: `{ level = 25, talent = "jack_of_all_trades", rank = 2 }` (Q9). §7 |
| **Travelled hands** (3) | every weapon skill **and every fighting style** the content has (Q10) that the bard holds at rank 0 rises to rank 1, handed out once (`weapon_ranks { to = 1 }` with no list: everything but the schools of magic and hidden styles): +1 to hit with everything, and the defaults rise as any rank gained raises them. §8 |
| **Goes** | Blessing (the old 6, Q1): a priest's abjuration shared with the paladin, off the bard's schools, and the second +1 to hit beside Inspired; the talent file stays (the priest's and paladin's tables name it). Lore (the old 1): gone from the game (#827); nothing is minted in its place. §5 |
| **Mediocre by data, not by cap on breadth** | nothing stops a bard training a weapon to rank 10 or a skill to 200; the class only starts low (hit points 2, Strength −1, Toughness −1, casting 75, mana 5) and spreads wide (4 rolls a level, 1 creation pick more, 8 trained skills, 3 optional). The one cap is the **level** cap, 30, which the Hunter lifts. §2, §5 |
| Coverage policy | no new `[[system]]`: songs extend `status`, `resources`, `mana`, and `hearing`; the skill grants extend `skills` and `weapon_skills`. No pickable talent is minted (the bar is one, and a count is never a reason). Rules 1–9 checked per entry in §5. |
| Slices | B1 songs, B2 Travelled hands and Blessing out, B3 Jack of all trades in two ranks and class tables naming a rank, B4 Lullaby (the one action), B5 the bard's table compressed and capped (needs P1 of `prestige-classes.md`), B6 the wiki, the spec, and the docs. §11 |

**Key choices, and why**

1. *Mediocre is breadth, written as grants.* ADOM's bard gets random weapon skills, random spells, random skills, and the two D&D bards get "a little to everything you lack" (5e's Jack of All Trades, Pathfinder's use-any-skill). Warlock's version hands out the breadth as data (every weapon skill and style to 1, every skill taken up) rather than drawing six at random, because the wiki states rules exactly and "six random" is a lottery the game would have to be vague about. The grants are **generic**: they read the content's lists, never a list of their own, so a skill or a style added later is the bard's too (the owner: "make sure it pull all skills, no matter how many we add later").
2. *The song is the one thing the bard owns.* Inspire exists, Music exists and reads nothing. One `song` effect kind serves Inspire and the Lullaby, and Music's value is what makes a bard's song better than anyone else's. The bard sings; there is no instrument bonus (the owner, Q8), and the lute stays what it is, a thing for the inn.
3. *The Lullaby is a check, not a level gate.* Hound's bay (a birth sign's) frightens everything of a lower level; a bard should be able to fail. A Willpower check per hearer against 13 plus the Music craft lets the weak-willed drop and the strong shrug it off, and the wiki states the odds (`check::ways`). The owner: "check, no level gates."
4. *One cost a song.* Inspire spends Inspiration (the pool is named for it and built by kills), the Lullaby spends mana (a level-6 spell's price near enough, the Sleep spell's cousin): the fight song is paid by the fight, the buff by the kills in it. No fallback from one to the other (the owner: "no dual resource costs").
5. *Two ranks, one id.* The level-12 and level-25 halves of Jack of all trades are one talent in two ranks; the class table gains a `rank` field, which costs one line in the parser and one in `class::grant` and is the shape #776's rule 4 asks for.
6. *The whole class fits in 30 levels.* Every ability by 30, the cadence halved (1, 3, 6, 9, 12, 16, 20, 25, 30 for the old 1, 6, 12, 18, 25, 32, 40, 50, 60), and the class stops there. What comes after 30 is another class's, found, not given (`prestige-classes.md`).
7. *One action, no state of its own.* Only the Lullaby is a new action; skills, weapon ranks, and talent ranks are saved already. The cap and the calling add theirs in the prestige design.

## 2. The fantasy in one paragraph, and how the data already says it

A bard has been everywhere and done a little of everything: fought with whatever came to hand, read scrolls not meant for them, picked up a lockpick once, bound a wound, sung for supper. They are not the warrior, the wizard, the thief, the priest, or the scholar; they are a little of each, and the one thing nobody else does is sing. In play: a bard tries everything, fails more than a specialist would, and is never stuck; their fights are won by a buff, a lullaby, and a thrown voice more than by the blade. And a bard is done learning at 30: there is nothing more the road can teach them, until something in the world calls them to another trade.

What the class data says already (the "mediocre" is mostly written; this design completes it):

| Facet | The bard today | The specialist | Reading |
|---|---|---|---|
| Body | `hp = 2`, Strength −1, Toughness −1, Dexterity +1 | berserker 8, warrior 7 (Strength 1d3+1, Toughness 1d3); wizard 0 | a weak body, not the weakest |
| Weapons | Short blades 1, Daggers 1d2, Duelist 1; everything else 0 | warrior Long blades 2 and a shield; thief Daggers 2, Thrown 1 | one blade, a little; §8 gives one rank in all |
| Magic | `casting = 75` (caster level three quarters of the character's), `mana = 5`, Restoration 1, affinity enchantment 2 and divination 1; Mend at 1 | wizard 100 and `mana = 12`, Evocation 2; warrior and thief 25 and `mana = 1` | a half-caster who learns from any scroll (study, Decipher script) |
| Skills | **4 rolls a level** (the most of any class), known for Bluff and Music (`2d6+10`), Appraise, Negotiation, Decipher script, Initiative, Knowledge, Tumble (`2d4+4`), optional Listen 30 %, Spot 30 %, Steal 15 % | thief 3 rolls and 11 skills, 6 known for; most classes 2 | breadth of learning is the class's identity (design `progression.md` §5: "one class with many rolls: the bard") |
| Talents | **one creation pick more** (3; a human bard 4); bonus picks at 10, 20, 30, 45, 55 from Scholar and Delver | everyone 2; the commoner's bonus picks from any tree | the lucky class (ADOM gives bards a bonus talent); §5 gives the bard the commoner's "any tree" too (Q13) |
| Talk | Bluff, Negotiation, Knowledge; Throw your voice at 32 | — | the quick tongue |
| Song | Inspire at 18 (+1 to hit, Dodge +1, 5 minutes; 3 Inspiration else 3 mana), Virtuoso at 60 (+2/+2, 10 minutes, free); Inspiration built by kills, max 5, a point lost every 30 base turns; **Music reads nothing**; the lute "waits for the Music skill's use" | — | the one thing nobody else has, half built |
| Experience | 100 % | wizard 125, thief 95 | middling; and, after this design, the one class that stops at 30 |

## 3. Research: bards and jacks of all trades elsewhere, tied to decisions

| Source | What it does | What this design takes, and what it leaves |
|---|---|---|
| **ADOM's bard** (Ancardia wiki, *Bard*, *Class power*): class powers at **6** +1 skill increase every level-up; **12** 1d3 levels in six randomly chosen weapon skills; **18** four random spells; **25** six random skills received or improved; **32** hit point regeneration doubled; **40** power point regeneration doubled; **50** +6 to all attributes; bards get more skill increases and a bonus talent; the wiki notes the gains "can be different in different games" | the one roguelike bard, and the model for the cadence (design `progression.md` §4.1 took ADOM's seven levels; this design halves them into 30). **Taken**: 12 → Travelled hands (§8), every weapon skill and style rather than six random; 25 → Jack of all trades rank 1 (§7), every skill rather than six random (the wiki states a rule; a random six is what the wiki could only call chance). **Left**: 6 (+1 roll a level) was the bard's Wide learning until #780 cut it as Quick learner's clone (`skill_rolls`); 18 (random spells) is what Warlock's study does already, with Decipher script and the affinities, and no slot is spent on it; 32 and 40 (regeneration doubled) are Saint's and Arcane mastery's effects (`healing_percent 100`, `mana_regen_percent 100`) and a bard is neither healer nor mage; 50 (+6 all stats) is Unlikely hero (+1 all) writ large, and rule 3 forbids the clone |
| **D&D 3.5 bard** (SRD): bardic knowledge; bardic music, once a day per level: countersong 1, fascinate 1, inspire courage +1 at 1 (+2 at 8, +3 at 14, +4 at 20), inspire competence 3, suggestion 6, inspire greatness 9, song of freedom 12, inspire heroics 15, mass suggestion 18; d6 hit die, three-quarter attack, 6 + Int skill points, arcane spells to 6th | **Taken**: fascinate → the Lullaby (§6.3; Warlock has Drowsy and the Sleep spell, and a song that holds hearers in place is a song that puts them half to sleep); inspire courage's growth → Music's degrees and Virtuoso (§6); bardic knowledge → the Knowledge skill, which exists (Lore, the talent, is gone, #827). **Left**: countersong (hexes on the player come from a few caster types; niche by rule 6's spirit); suggestion and mass suggestion (a pacify mechanic: a faction change at talk, a far larger design, §12); inspire competence (the skill facet is Jack of all trades) |
| **D&D 5e bard** (roll20 compendium): Jack of All Trades at 2 (half proficiency to every check without it); Song of Rest at 2 (d6 → d12); Expertise 3 and 10; Font of Inspiration 5; Countercharm 6; Magical Secrets 10 (spells from any class); Superior Inspiration 20; the inspiration die d6 → d12 | **Taken**: Jack of All Trades's shape, "a little to everything you lack", is rank 1 of §7; Song of Rest is Fast healer, which stays at 6. **Left**: Magical Secrets (any scroll is already the bard's to study); Superior Inspiration (a free point at a fight's start: Inspiration is built by kills and decays, which is the design's shape, `docs/dev/resources.md`); Expertise (a doubled skill is the specialist's, not the bard's) |
| **Pathfinder bard** (Core Rulebook, aonprd): versatile performance at 2, 6, 10, 14, 18 (Perform in place of other skills); lore master 5; well-versed 2; **jack-of-all-trades: 10 use any skill untrained, 16 every skill a class skill, 19 take 10 on any skill**; performances: distraction and fascinate 1, inspire courage, inspire competence 3, suggestion 6, dirge of doom 8, inspire greatness 9, soothing performance 12, frightening tune 14, inspire heroics 15, mass suggestion 18, deadly performance 20 | **Taken**: the three-step jack-of-all-trades is the model for the two ranks (§7): 10 → rank 1 at 12 (every skill taken up: in Warlock an untrained skill does not exist, so "use untrained" is "trained a little"); 19 → rank 2 at 25 (every skill +5). Pathfinder's bard is a **20-level** class with every trick by 20: the compressed table (§5) is nearer that than ADOM's 50. **Left**: 16 (every skill a class skill: the +1 a level-up roll gets on a class's skill) folded away, the bard's four rolls are its learning; versatile performance (one skill standing in for another: information-free but a bookkeeping rule the game would have to explain); dirge and frightening tune (Hound's bay, a birth sign's pick, is the one fear source; a second would be its clone in all but kind, §12); soothing performance (Mend and Fast healer are the bard's healing); deadly performance |
| **DCSS's Wanderer** (CrawlWiki): a background of random skills (ten levels split over a "good" item set, two "decent" ones, and the rest at random), random items, random stats summing to 12; "the possibility for some awful starts" | breadth as a lottery at creation. Warlock's bard already has its small lottery (the optional Listen, Spot, Steal) and the owner's rule that the UI never speaks of chance's source (#256). **Left**: more randomness at creation; the grants of §7 and §8 are fixed, so a bard is never an awful start |
| **Tales of Maj'Eyal's Adventurer** (te4 wiki): talent trees from any unlocked class, extra category points, a kit of every weapon kind, "by no means balanced" | breadth as picks. **Taken** (the owner, Q13: "any"): the bard's bonus picks come from any tree, the commoner's gift shared, since a jack of all trades picks from everywhere |
| **Elona's Performer** (Elona wiki): play an instrument to an audience for tips and thrown items; listeners of a level above the performer's throw stones | a song for gold is economy, which rule 1 bars from the capstones and rule 9's spirit keeps out of the tables. The "audience of a higher level resists" idea becomes the Lullaby's Willpower check rather than a level gate (key choice 3). The instrument as the thing that makes the music better was in the first draft and is dropped: the bard sings (Q8) |

## 4. What exists today

| Part | Today |
|---|---|
| The table | 1 Lore (gone from the game, #827), Mend (spell), Trained wind; 6 Blessing (spell); 12 Fast healer (Vigour, given; #790 cut the clone Song of rest); 18 Inspire; 25 Jack of all trades (`trained_skills 5`); 32 Throw your voice; **40 empty** (Honeyed words cut by #798: `favour_percent 15`, `quest_gold_percent 25`, rules 1 and 9); **50 empty** (Legend cut by #798: `quest_xp_percent 50`, `stat presence 1`, rule 1); 60 Virtuoso. Wide learning (6, `skill_rolls 1`) was cut by #780 as Quick learner's clone |
| Levels | `max_level = 60` for everyone (`data/experience.toml`; `Experience::level_for` stops there; XP beyond it counts and buys nothing); `hp_level = 50`, `mana_level = 60`; the curve is `[kills]` with the `[early]` discount, a threshold table at load (level 30 costs 545,660 XP in all at 100 %, level 60 2,845,856; `docs/dev/experience.md`) |
| Inspire | `inspire { status = "inspired", mana = 3, resource = "inspiration", cost = 3 }` (`talent::Inspire`), `World::inspire` (`world/class_ability.rs`): spends the resource when held, else the mana; afflicts `inspired` (`max = 1`, `eases = 300`, `combat = { to_hit = 1, dodge = 1 }`); a Presence training mark (`StatUse::Inspired`); priced as a chat (`chat = 500`, half a base turn). **No noise is made.** Virtuoso: `virtuoso { status = "inspired_greatly" }` (`eases = 600`, +2/+2) and `Inspire::instead` cleared, so it costs nothing |
| Inspiration | `resources.toml`: `max = 5`, `start = 0`, `decays = 180`, `builds = [{ on = "kill", amount = 1 }]`; granted by Inspire's `grants_resource` |
| Music | `skills.toml`: `kind = "action"`, no `mark`, no `benefit`: **nothing reads it**; the abilities menu lists it under the skills as "not usable yet" (`Ability::Skill`, `Unavailable::NotYet`), beside Bluff, Knowledge, Decipher script, Handle animal, Metalcraft, Woodcraft (Intimidate has its action since #844). The bard is known for it (`2d6+10`) |
| The lute | `objects/lute.toml`: class `miscellaneous`, 1.5 kg, value 40, in the bard's kit; "Playing it waits for the Music skill's use; until then it is carried for the tunes to come" |
| Skills | an untrained skill does not exist for the actor (never shown, gives nothing; `Skills`); "trained at all" is `Skills::trains_effect` (Spot, Detect traps, Steal, Heal, Pick lock, Disarm, Forage, Sneak attack's first die); `skill { skill, value }` on talents and items adds to a skill "an untrained one from 0" without training it; `train_skill { dice }` (Apprentice) trains the chosen skill from the levels stream (`World::train_chosen`); `trained_skills { value }` (Jack of all trades) adds to every trained skill (`Actor::skill_effect`). Level-up rolls go on trained skills only; the Skills screen lists them |
| Weapon skills | ranks 0 to 10 per skill (`WeaponSkills`), raised by marks (`train` → `raise`: the defaults rise, marks reset); a talent's `weapon { ranks }` reads a **held** rank higher and leaves rank 0 at 0 (`TalentMods::raised`); weapon abilities start at rank 4 (`data/weapon_abilities/`: ranks 4 to 10); Daggers rank 1: +1 to hit, nothing else. Styles are rows of `Role::Style` (Duelist, Twin Blades, Great Weapon, Sword and Board, Marksman, Skirmisher, Brawler, Staff Fighter; the caster styles Sniper, Toucher, Battle Caster; Two Shields is `hidden`); the schools of magic are rows of `Role::School` |
| Sleep and Drowsy | the Sleep spell (`spells/sleep.toml`: level 6, enchantment, a burst of radius 1 at range 8, `drowsy` severity 2, Presence lengthens it); `drowsy` (`max = 3`, `eases = 12`, `cost = 300`: every action three times as dear, a degree each 12 s) |
| Hearing | `noise::noise(act)` → `noise::hearers` (who hears it, by loudness less `STEP_LOSS` 10 a cell through what passes sound; nobody on the overworld; **ground truth**: an invisible or unseen creature hears as any other); a voice is `VOICE_NOISE` 50, a howl `HOWL_NOISE` 150; Hound's bay (`world/sign_picks.rs`) is the one ability that acts on hearers: every hostile that heard it, of a lower level than the player, Frightened |
| Checks | `check::check(rng, modifier, difficulty)`: 2d12 plus a stat modifier, natural 2 fails, 24 succeeds; `check::ways` the odds the wiki prints; the surprise design's initiative check is "the target rolls Dexterity against 13 plus the attacker's craft" |
| Class tables | `ClassDef::abilities: Vec<(u8, String)>` (`{ level, talent }`); `class::grant` gives what the table names up to the level **at rank 1** (`hand_out` for what a rank gives once); `check_table` at load; a talent held already brings a pick in its place (pickables only) |

## 5. The table proposed: levels 1 to 30, and the cap

The old nine levels (1, 6, 12, 18, 25, 32, 40, 50, 60) halve into 1, 3, 6, 9, 12, 16, 20, 25, 30: the same growing cadence (gaps 2, 3, 3, 3, 4, 4, 5, 5 for the old 5, 6, 6, 7, 7, 8, 10, 10), every ability by 30, and the class ends there.

| Level (old) | Today | After | Facet | Rules 1–9 |
|---:|---|---|---|---|
| 1 (1) | Lore (gone, #827), Mend (spell), Trained wind | **Mend and Trained wind stay**; nothing replaces Lore (no talent is minted to fill a slot) | a little healing, a little fighting (stamina) | shared with the priest and every fighting class |
| 3 (6) | Blessing (spell) | **goes** (Q1); **Travelled hands** (§8): rank 1 in every weapon skill and style held at 0 | a little of every weapon and every way of fighting | a hand-out, not a clone of `weapon { ranks }` (which never raises 0); not information; not niche |
| 6 (12) | Fast healer (Vigour, given) | **stays** | rest | Song of rest's replacement by #790; a bard's "song of rest" |
| 9 (18) | Inspire | **stays, reworked** (§6.2): a `song`, heard, longer with Music, paid in Inspiration alone | the song | `status`, `resources`, `hearing` |
| 12 (25) | Jack of all trades (`trained_skills 5`) | **reworked** (§7): rank 1, you take up every skill you lack (`1d4+1` each), Sneak attack excepted | a little of every skill | changes what the character can do in a dungeon (locks, traps, wounds, theft, forage) |
| 16 (32) | Throw your voice | **stays** | trickery | `hearing`, `stealth` |
| **20** (40) | empty (Honeyed words cut) | **Lullaby** (§6.3): hostile hearers who fail a Willpower check are Drowsy | the song, against foes | rule 1 (read at the class's capstone end, 20 and up of 30): changes a fight; no gold, no XP; paid in mana, no cooldown (policy step 4) |
| **25** (50) | empty (Legend cut) | **Jack of all trades rank 2**: every skill counts 5 higher (today's effect, moved, Q4) | every skill, better | rule 1: a dungeon-changer across every check (locks, traps, Stealth, Tumble's Dodge, Heal, Spot, Knowledge's resist, Concentration's mana, Decipher's study, Swim, Balance's burden); rule 4: a rank, not a second id |
| 30 (60) | Virtuoso | **stays, widened** (§6.5): every song greater and free | the song, mastered | `status`, `mana`, `resources` |

**Bonus picks** move with the cadence, 5, 10, 15, 23, 28 (the old 10, 20, 30, 45, 55 halved), and come **from any tree** (Q13: `bonus_talents = { levels = [5, 10, 15, 23, 28] }`, no `trees`, the commoner's gift shared). The class file's `skills`, `optional_skills`, `rolls = 4`, `creation_talents = 1`, `weapon_skills`, `styles`, `affinity`, `casting = 75`, and the kit are unchanged: they are the breadth already. Talent picks stay every third level (3, 6, ..., 30: ten in all); a bard's 30 levels bring ten picks and five bonus picks where sixty brought twenty and five.

**The cap.** `max_level = 30` in `02-bard.toml` (`ClassDef::max_level`, a new field, default `[levels] class_max` of `data/experience.toml`, 60: no other class changes). `World::player_level` reads the curve to the character's cap (`Character::level_cap`: the base class's, plus a prestige class's levels once called; `prestige-classes.md` §4). A bard at 30 keeps earning XP (as anyone at 60 does today: "XP beyond it still counts but buys nothing"); the XP row shows no next level, as at the top today; nothing in the game says why. When the Hunter's calling comes (the oath said to the Strange Spirit, `prestige-classes.md` §7), the character is given the XP level 31 costs if the bank is short, and the level-up to Hunter 1 and every level banked come at once. The class's level-up news keeps its words; the character screen's Class row reads `Bard 30` and later `Bard 30, Hunter 4` (the prestige design's §10).

**Rule 1 under a 30-level class.** The policy reads "from class level 40 on" and "no XP or quest-reward effect at or above level 30" against a 60-level table. For the bard the capstone end is its last third (20 and up) and the XP bar its last half (16 and up): the Lullaby, Jack 2, and Virtuoso are judged as capstones above, and no entry of the table pays in gold, favour, quests, or XP at any level. `progression.md`'s wording should say "the last third of a class's table" when the bard's lands (B6).

**Why Blessing goes and Fast healer stays.** Both are shared entries, and one slot had to give way to the weapons facet, which nothing in the table covers. Blessing is abjuration (the priest's at 3, the paladin's at 6), off the bard's schools, and gives the same +1 to hit the bard's own Inspired gives; a bard who wants it studies a scroll of it like any other. Fast healer is the bard's healing by rest (D&D's Song of Rest) and nothing else in the table heals the bard between fights. The `blessing` talent file stays for the two tables that name it. Saves are pinned to the game version that wrote them (#840): no shim for a bard holding Blessing; a new run never gets it.

## 6. Songs: Music, Inspire reworked, Lullaby, Virtuoso

### 6.1 Music, the skill

```toml
[[skill]]
id = "music"
name = "Music"
description = "Singing: how far your songs carry into a heart, and how long they stay there."
kind = "rules"
mark = [{ on = "song", chance = 100 }]
# A song's degrees (spec §8 "Class abilities", the bard's songs): one more at 50 and at 100,
# then one more for every 100 past it, up to what the status takes. And how hard those who hear a
# song against them find it to resist: +4 to the check's difficulty at 100, +8 at 200.
benefit = [
    { effect = "song_degrees", steps = [50, 100], every = 100 },
    { effect = "song_craft", curve = [[0, 0], [100, 4], [200, 8]] },
]
```

`kind` changes from `action` to `rules` (the songs are class actives and read it; the "not usable yet" row for Music goes from the abilities menu, Q11). `skills::Effect::SongDegrees`, `Effect::SongCraft`; `Use::Song` (a song sung, whatever it does). The bard is known for Music, so a level-up roll on it throws a bigger die: a bard who spends rolls on Music has it at 100 around level 15 to 20 (four rolls a level over 30 levels is 120 rolls; the thief's three over sixty bring Sneak attack to 100 around 20 to 25 in the surprise design's reckoning).

### 6.2 The `song` effect, and Inspire on it

Inspire's `inspire { status, mana, resource, cost }` becomes the general kind (a rename of a content effect kind; saves name talents by id and are untouched):

```toml
# talents/inspire.toml, after
effects = [
    { kind = "grants_resource", resource = "inspiration" },
    { kind = "song", id = "inspire", on = "self", status = "inspired", resource = "inspiration", cost = 3 },
]
```

| Field | What |
|---|---|
| `id` | the active's id (`use:inspire`, `use:lullaby`); the abilities menu row under the class source, as today |
| `on` | `self` (Inspire: the singer suffers the status) or `hearers` (the Lullaby: every hostile that hears the song, §6.3) |
| `status`, `severity` | the status given and its base degrees (default 1); the song's degrees are `severity` + Music's `song_degrees`, capped by the status's `max` |
| `resource`, `cost` **or** `mana` | **one of the two** (the loader refuses both, Q6): `cost` of `resource` (Inspire: 3 Inspiration; unavailable while short, `Unavailable::NoMana`'s words from the resource's `lacking`), or `mana` (the Lullaby: 6) |
| `check`, `difficulty` | `hearers` only: the stat each hearer checks (`willpower`) against `difficulty` (13) plus Music's `song_craft` |

`World::sing(song)` (`world/song.rs`, the one place a song is sung; `World::inspire` becomes its `on = self` arm): pay (the resource, or the mana; refused with the resource's `lacking` words or "You have too little mana."), **make the noise** (`Act::Sing`, `SoundKind::Voice`, `SONG_NOISE` 80: it carries 8 cells, farther than a chat's 50 and well short of a howl's 150; creatures that hear it come to look, as they do for a thrown voice), then the statuses, then the Presence mark (`StatUse::Inspired`, as today) and the Music mark (`Use::Song`), priced as a chat. Refused with no time on the overworld (silent, as a thrown voice is) and while Silenced (`silenced` gains `song` in its `blocks`).

**Inspire after**: Inspired for 5 minutes a degree (`inspired` gains `max = 3`): one degree at Music below 50, two at 50, three at 100. Virtuoso's `inspired_greatly` (`max = 3` too) the same, at 10 minutes a degree. The combat bonus is the status's and does not grow with degrees (Inspired is `max = 3`, `stacking = "refresh"`: degrees are time, as the witch's hexes' are; Q14).

**What changes for a bard today**: Inspire is heard (Q7), and it no longer falls back on mana (Q6): a bard with fewer than three Inspiration cannot sing it, and Inspiration is built by kills (one a kill, five at most, a point lost every 30 base turns). So Inspire is the song of a fight going well, sung as the third foe falls to lift the bard for the rest; the first song of a fight is the Lullaby, paid in mana. A bard who sings in a quiet corridor tells the level where they are, which is the fantasy (a bard is loud) and a real cost that Inspire lacks today.

### 6.3 Lullaby (20)

```toml
[[talent]]
id = "lullaby"
name = "Lullaby"
tree = "class"
granted = true
description = "A slow song for those who mean you harm (an ability, paid in mana): every foe that hears it and cannot shake it off grows drowsy, slow in all it does, a while."
effects = [{ kind = "song", id = "lullaby", on = "hearers", status = "drowsy", mana = 6, check = "willpower", difficulty = 13 }]
systems = ["status", "mana", "hearing"]
```

| Rule | Decision | Where |
|---|---|---|
| Who | every creature that **hears** the song (`noise::hearers` of the Sing noise, before it is made, as Hound's bay does: **ground truth**, so an invisible or unseen foe in earshot is checked like any other; what the bard perceives decides nothing, Q15) and is hostile to the bard (`World::stance`); friends, allies, and the peaceful hear it and nothing more; a sleeper (Asleep) hears as a noise wakes it and is checked like anyone (a lullaby does not deepen a sleep: the Asleep status is the den's, not this song's) | `song.rs` |
| The check | each such hearer: `check::check(rng, its Willpower modifier, 13 + the bard's song_craft)`, two gameplay draws apiece, in actor id order (deterministic). Made: nothing (a message for one seen, §9). Failed: `afflict(hearer, drowsy, degrees, Some(bard))` with the song's degrees (1 + Music's, at most Drowsy's 3). A creature already Drowsy is refreshed (the status's `stacking`) | `song.rs` |
| What Drowsy does | everything the creature does costs 300 % (`cost = 300`, the scheduler's one price hook, `World::burdened`), a degree easing every 12 s: one degree is two base turns at a third of its speed, three degrees six. It still acts, slowly; it is not asleep and not unaware (the surprise design's `unaware` is untouched: a drowsy foe that saw the bard is aware) | `data/status.toml` (exists) |
| Cost | 6 mana (Q6: one cost; the Sleep spell's cousin at a level-6 spell's price); a chat's time (500); **no cooldown**: mana is the price (policy step 4) and the noise is the rest of it | `song` data |
| Refusals | no hostile hears it: **sung anyway** (Q15: the mana is spent, the noise made, the marks earned; refusing would need the game to say nobody hostile hears, which is a leak); the overworld and Silenced refuse with no time, as above | `song.rs` |
| Odds (the wiki states them) | a hearer at Willpower +0 against 13 (Music below 100): resists 78 of 144 (54 %); against 17 (Music 100): 36 of 144 (25 %); against 21 (Music 200): 10 of 144 (7 %); a hearer at Willpower +4 against 17: 78 of 144 (54 %). The strong-willed (the drow, the undead casters, the fey lords) shrug it off; the kobolds, rats, and goblins drop | `check::ways` |

**Why Drowsy and not Asleep.** Asleep is the den's state (no turn, unaware, woken by noise): a song that put a room to sleep would be the thief's Lurk and Sneak attack for everyone, and it would let a bard walk through Cealdhlāw's dens untouched. Drowsy slows; the bard still has to win the fight or leave it. The Sleep spell gives the same status and the name says what the song is.

**Why hearers, not a burst.** The Sleep spell is an aimed burst of radius 1 at range 8 (two or three creatures); a song fills the hall. A song also goes through a door and round a corner, which a spell does not, and it tells everyone in earshot where the singer is. The two are different tools, and a bard who studies the scroll has both.

### 6.4 No instrument

The first draft gave a song one degree more with the lute carried (`ObjectDef::instrument`). The owner: "no. bard always sings." Dropped whole: no `instrument` field, no `Actor::carries_instrument`, no read in `sing`. The lute stays in the kit as it is (a thing for the inn, 1.5 kg, 40 gold); its data comment changes to say so ("A bard sings; the lute is for the inn, and for selling when the road is long").

### 6.5 Virtuoso (30), widened

```toml
effects = [{ kind = "virtuoso", status = "inspired_greatly" }]
```

stays, read in `World::sing`: **every** song costs nothing (the song's `cost` and `mana` set aside, as Inspire's are today) and has one degree more, and Inspire gives `inspired_greatly` in place of `inspired`. The description becomes "Your songs lift higher and last longer, and cost you nothing." A Virtuoso's Lullaby at Music 200: 1 + 3 + 1, capped at Drowsy's 3; the cap is the data's to raise.

## 7. Jack of all trades in two ranks, and class tables that name a rank

```toml
[[talent]]
id = "jack_of_all_trades"
name = "Jack of all trades"
tree = "class"
granted = true
ranks = 2
description = "You have picked up a little of everything on the road."

[[talent.rank]]
# Level 12: every skill you lack, taken up a little (the levels stream), whatever skills the game
# has; never Sneak attack, the thief's. Said once: "You find you know a little of everything."
effects = [{ kind = "take_up_skills", dice = "1d4+1", except = ["sneak_attack"] }]

[[talent.rank]]
# Level 25: every skill you have counts a little higher.
effects = [{ kind = "trained_skills", value = 5 }]
```

| Rule | Decision | Where |
|---|---|---|
| **Rank 1** (12): take up every skill | a hand-out as `train_skill` is (`World::train_chosen`): for **every skill in `skills.toml`**, in file order, that the player is not trained in and that `except` does not name, roll `dice` from the **levels stream** (`levels_rng`, made on the first draw) and `Skills::add` it. **Generic by construction** (the owner, Q2: "make sure it pull all skills, no matter how many we add later"): the effect walks `Content::skills()`, never a list of ids; a skill added next month is a bard's at 12 with no change to the talent. About twenty skills for a bard today (eight trained, up to three optional, Sneak attack skipped). From then on they are trained: shown on the Skills screen (it pages; `ui/skills_screen.rs`), marked by use, rollable at a level (`advance:SKILL`, at the ladder's lowest die and no +1, since the class does not train them), and **trained at all** for every rule that asks (`Skills::trains_effect`): the Heal ability, Pick lock, Disarm, Steal, Forage, Intimidate (#844), noticing hidden features and traps in passing, Concentration's mana, Spellcraft, Survival, Swim, Woodcraft, Handle animal, Escape artist | `world/jack.rs::take_up_skills`, from `class::hand_out` |
| **Sneak attack** skipped | the surprise design's decision 7 (the owner: "only thief gets sneak attack guaranteed"), **kept** (the owner, Q3: "nah, no sneak attack"); `except` is data, so the list stays honest and short (one id) | data |
| **Rank 2** (25): every skill +5 | today's `trained_skills 5`, moved (Q4): with every skill trained, it is +5 to all of them, as Pathfinder's "take 10 on any skill" at 19 | `Actor::skill_effect` (exists) |
| **The class table names a rank** | `{ level = 25, talent = "jack_of_all_trades", rank = 2 }` (Q9): `AbilitySpec` gains `rank` (default 1); `ClassDef::abilities: Vec<(u8, String, u8)>`; `check_table` checks the rank is within the talent's `ranks` and that a higher rank comes after a lower one of the same id; `class::grant` gives rank `r` when the level comes to one holding `r − 1` (and every missing rank below it on a restore, each handed out); the news "You gain Jack of all trades." / "Your Jack of all trades grows." A table that names a rank of a pickable (none does) would follow `held_already` as today. The prestige design's tables use the same field (Quarry 1/3, 2/3, 3/3) | `content/class.rs`, `world/class.rs` |
| Why not two ids | #776's rule 4: "one effect, two ids, chained, is a rank"; the class tree has no `needs` to trip `talent_alike`, but the shape is the same, and ranks are what the Talents page and the wiki already show ("Jack of all trades 1/2") | — |
| Numbers | a level-12 bard's new skills are 2 to 5 each (a thief's Open lock starts at 12 to 22); at 25 they count 7 to 10 plus whatever rolls went there. Picking a locked door of difficulty 14 at Open lock 3 is a Dexterity check at +0 or so against 14: about 45 %; a thief at 60 makes it nearly always. That is the fantasy: the bard can, badly, and time is the penalty (the owner's no-anti-farming rule: a lock may be tried again) | — |

**What the hand-out draws.** Twenty-odd rolls of `1d4+1` from the levels stream at level 12: every later levels-stream draw of that run (stat checks, skill rolls) moves, as any talent's `train_skill` does today; the gameplay stream is untouched. Fixtures without `skills.toml` entries hand out nothing.

## 8. Travelled hands

```toml
[[talent]]
id = "travelled_hands"
name = "Travelled hands"
tree = "class"
granted = true
description = "You have fought with whatever came to hand, every way there is: you know every weapon and every style a little."
effects = [{ kind = "weapon_ranks", to = 1 }]
systems = ["weapon_skills", "combat"]
```

| Rule | Decision | Where |
|---|---|---|
| What | a hand-out once, at level 3: **every weapon skill and every fighting style** in `weapon_skills.toml` that the bard holds at rank 0 is **raised to rank 1** (`WeaponSkills::raise_to` per skill, in file order, so its defaults rise as any rank gained raises them; marks reset as `raise` does). **Generic** (Q10, as Q2): without `skills` or `roles` the effect walks every row but the schools of magic (`Role::School`: a study, not a weapon) and hidden styles (`hidden = true`, Two Shields: a hidden rank would show the row, `WeaponSkills::shows`). So: the fourteen weapon skills (daggers to thrown), shield blocking and bashing, the eight fighting styles, and the three caster styles (a bard casts a little, in armour or afar: Sniper, Toucher, Battle Caster at 1 fit the fantasy). The prestige design's Hunter's trade uses the same kind with `skills` named and `to = 3` | `world/class.rs::hand_out` → `WeaponSkills::raise_to` |
| What rank 1 gives | by the rank tables (`weapon_skills.toml`): Daggers +1 to hit; the light blades, heavy, middling, launchers, thrown, unarmed likewise around +1 to hit at rank 1 and nothing else; the styles' rank 1 their small first benefit (Marksman's, Skirmisher's, Great Weapon's); weapon abilities begin at rank 4, so no menu row appears. A bard who picks up a found long sword, a bow, or a sling hits with it a point surer than an untrained anyone, and fights two-handed, with a shield, or with two blades a little better than one who never tried: mediocre, everywhere | data (exists) |
| A rank already held | untouched (short blades, daggers, Duelist); `to` is a floor, never a raise of a held rank | — |
| Why not `weapon { role, ranks = 1 }` | that effect reads a **held** rank higher and leaves 0 at 0 (`TalentMods::raised`), so it would give the bard nothing on the weapons it lacks, which is the point. A real rank is also trainable on: marks on a rank-1 skill carry it to 2 | — |
| Why level 3 | ADOM's bard gets its random weapon skills at 12 of 50; the compressed table's second slot is 3, and it was Blessing's (§5). A level-3 bard with a rank in everything is a bard who stops hoarding daggers | — |
| Draws | none (a flat raise); the news "Your hands know every weapon a little." (one line, not one per skill: `WeaponSkills::raise_to` returns what rose and `say_news` folds them) | — |

## 9. What the player sees; determinism, saves, replays

| Where | What |
|---|---|
| Messages | Inspire as today ("Your song lifts your heart.", the status's `start`); the Lullaby: "You sing a lullaby." then, for those seen, "The kobold's eyes grow heavy." / "The orc shakes its head." (made), as Hound's bay names those it saw flee; nothing for hearers out of sight (they are not shown; the noise is theirs to answer; an unseen one that failed is Drowsy all the same and shows it when seen). Travelled hands: "Your hands know every weapon a little." Jack of all trades rank 1: "You find you know a little of everything." Rank 2: "Your Jack of all trades grows." No number, no roll, no degree count |
| Look mode | "drowsy" on a creature that is (the status's `seen`, as today); nothing says who sang it |
| The abilities menu | Inspire and Lullaby under the class source (`Ability::Class`), greyed with the resource's `lacking` words ("not enough inspiration") or "you have too little mana"; Music's "not usable yet" row goes (`kind = "rules"`) |
| The Skills screen | after 12, every skill (it pages); the character screen's skill count grows |
| The status bar | `Inspiration 3` as today; Inspired and Drowsy as today |
| The level and XP | level 30 is the bard's last: the XP row shows no next, as the top level's does today; the character screen's Class row `Bard 30` (and the prestige design's `Bard 30, Hunter 4` later) |
| Dumps | `abilities` lists the two songs; `skills` lists the taken-up skills; `talents` shows `Jack of all trades 1/2`; `actors` shows a creature's Drowsy with its degrees; JSON `abilities` gains `lullaby` |
| Determinism | the Lullaby's checks draw two gameplay draws per hostile hearer, in actor id order, only when sung; Inspire draws nothing (as today); Travelled hands draws nothing; Jack of all trades rank 1 draws from the levels stream. Runs that never sing or never reach a bard's 3 or 12 hash as before |
| Saves | **no new saved state in the bard's slices**: skills, weapon ranks, talent ranks, statuses, and resources are saved already. Saves are pinned to the game version that wrote them (#840): no shim for a save holding Blessing or an old-table bard; `ClassDef::max_level` is content. The cap and the calling add their state in the prestige design (`Character::prestige`) |
| Replays | a replay holding `use:inspire` plays (the id is kept); a bard replay past level 3 or 12 hashes differently from that level's grant (skills and ranks are hashed), as any class-table change does. The built-in scenarios' characters are a thief and rangers; no golden entry runs a bard |
| Content fingerprint | the `song`, `weapon_ranks`, and `take_up_skills` kinds are talent content (not fingerprinted); `Use::Song` and the two Music effects are skill ids (fingerprinted as their kind is) |

## 10. The wiki, the spec, and the docs

| Where | What |
|---|---|
| `wiki/src/classes.md` (the bard section) and the generated class table | the table's nine rows at 1 to 30; a paragraph on the fantasy ("knows a little of everything") and the cap ("a bard learns nothing new past 30; what comes after is found, not given": a link to the prestige page); Travelled hands and Jack of all trades stated exactly (rank 1 to every weapon skill and style; `1d4+1` to every skill but Sneak attack; +5) |
| `wiki/src/abilities.md` | a **Songs** section: how a song is paid (one thing each), heard (80), and lengthened (Music's degrees table from `skills.toml`), Inspire's and the Lullaby's rows, the Lullaby's odds by Willpower edge and Music (`check::ways`, as `surprise.md`'s table), Virtuoso |
| `wiki/src/skills.md` | Music's row gains its two ladders (generated) |
| `wiki/src/resources.md` | Inspiration spent by Inspire alone |
| `wiki/src/status-effects.md` | Drowsy's causes gain the Lullaby; Inspired's degrees |
| `spec/08i-character.md` | the bard's table bullet rewritten (the nine rows at 1 to 30, the two ranks, the class-table `rank` field, `max_level`); Skills: Music's benefits and mark; Talents: `take_up_skills`, `weapon_ranks`, `song`, `virtuoso` widened |
| `spec/07b-data.md` | `classes/README.toml`'s `abilities` row gains `rank`, a `max_level` row; `talents/README.toml` the three kinds |
| `spec/08c-combat.md` or `08e-blood-and-status.md` | songs beside the class abilities' statuses |
| `docs/dev/progression.md` | the hook table rows (`song` → `world/song.rs`; `weapon_ranks`, `take_up_skills` → handed out, not read; `virtuoso` widened), "Class tables" gains ranks and the per-class cap; rule 1's wording ("the last third of a class's table") |
| `docs/dev/experience.md` | `ClassDef::max_level` and `Character::level_cap` (with the prestige design's P1) |
| `docs/dev/resources.md`, `hearing.md`, `skills.md`, `weapon-skills.md`, `status-effects.md` | one line each: Inspire's spending, `Act::Sing` and `SONG_NOISE`, Music's effects and `Use::Song`, `raise_to`, Inspired's `max` |
| Tests | `world/song_tests.rs` (Inspire paid by the resource and refused without it, its degrees by Music, the noise and who comes to look, Silenced and the overworld refused; the Lullaby paid in mana, its hearers including one the bard cannot see, the check's two outcomes, friends and allies untouched, Drowsy's degrees and cap, the empty room sung to, the messages, a replay, a save), `world/class_rogue_tests.rs` (Travelled hands' ranks and defaults over every row but schools and hidden styles, a fixture style added and taken up, Blessing gone, Jack of all trades' two ranks and the skills taken up, a fixture skill added and taken up, Sneak attack skipped, rank 2 on a restore, the news, the bard's cap at 30 with XP counting on), `content/class.rs` (the `rank` and `max_level` fields and their checks), `content/skill.rs` (Music), the abilities-menu screen snapshot with the two songs, the wiki test |

## 11. Implementation plan

Six issues; at most one adds an action. The false-conflict trick (`docs/dev/merging.md`) for the hot files: `talents/README.toml`, `skills.toml`, `status.toml`, `02-bard.toml`, `ability.rs`, `class.rs`. The prestige system's slices (P1–P5) are in `prestige-classes.md` §10; B5 needs P1.

| Issue | What | Adds | Depends on |
|---|---|---|---|
| **B1 Songs** (§6.1, 6.2, 6.5) | Music's `kind`, `mark`, and two benefits with `Effect::SongDegrees`/`SongCraft` and `Use::Song`; the `song` kind (Inspire migrated onto it, the `inspire` kind retired, the mana fallback gone), `world/song.rs::sing` with `Act::Sing` and `SONG_NOISE`; `inspired`/`inspired_greatly` `max = 3`; `silenced` blocks `song`; Virtuoso on every song; the lute's comment; tests; the README rows | none (the hearers arm is data-ready but no talent names it yet) | — |
| **B2 Travelled hands** (§5, §8) | the generic `weapon_ranks` hand-out (`to`; optional `skills`, which the Hunter's trade uses), `WeaponSkills::raise_to`, the talent, Blessing out of the bard's file, the news; `class_rogue_tests` | none | — |
| **B3 Jack of all trades** (§7) | the generic `take_up_skills` hand-out from the levels stream; the two-rank talent; `AbilitySpec::rank` with its checks and `class::grant`'s rank step; the bard's file naming rank 2; tests | none | — |
| **B4 Lullaby** (§6.3) | the talent; `ClassActive` `lullaby` (`use:lullaby`, aim nothing); the hearers arm of `sing` (the checks, Drowsy, the messages); the abilities-menu snapshot; action text in CLAUDE.md | **Action** | B1 |
| **B5 The table at 1 to 30, and the cap** (§5) | the bard's levels moved (1, 3, 6, 9, 12, 16, 20, 25, 30), bonus picks 5, 10, 15, 23, 28 from any tree, `max_level = 30`; tests of the cap | none (the cap's field is P1's) | P1, B2–B4 (so the moved levels name what exists) |
| **B6 Wiki, spec, docs** (§10) | the Songs section and its generated odds, the class table, Music's ladders, the spec bullets, the docs' lines, rule 1's wording, a cross-check of the spec against what landed | none | B1–B5 |

**Why this order**: B1 first because B4 reads it; B2 and B3 touch only the class file, `class.rs`, and their hand-outs, so they run beside B1 (their shared lines are `02-bard.toml`'s table and `talents/README.toml`: the trick). B4 is the one action and waits for B1. B5 is a data move once the field exists and everything it names has landed. Each issue answers the feature template's "Talents and class abilities": **none new**: songs extend `status`, `resources`, `mana`, and `hearing` (Patient blade, Clear mind, Keen ears touch them); the grants extend `skills` and `weapon_skills` (Apprentice, Weapon focus); no new `[[system]]`, no pickable minted (the bar is one and a count is never a reason).

## 12. Rejected ideas, with the reason

| Idea | Why not |
|---|---|
| **A cap on the bard's ranks or skills** ("master of none": no weapon past rank 5, no skill past 100) | the owner's "leaning towards mediocre", not "capped"; a flaw that bites every build the same is no choice (rule 5's spirit), and the player's own choices and the class's low start are enough. Unbalanced choices are fine (the owner). The one cap is the level's, 30, and it opens |
| **Six random weapon skills, six random skills** (ADOM's 12 and 25) | the wiki states rules exactly and the game stays vague about chance; "every" is a rule, "six at random" a lottery the wiki could only describe as odds; and ADOM's own wiki notes the bard's gains "can be different in different games" |
| **A list of skills or weapon skills in the grants** | the owner: "make sure it pull all skills, no matter how many we add later"; a list rots. Both hand-outs walk the content |
| **Wide learning** (+1 skill roll a level, the old 6) | cut by #780 as Quick learner's clone (`skill_rolls`); the bard's four rolls are the identity already |
| **Honeyed words, Legend** (the old 40 and 50) | rules 1 and 9 (#798): gold, favour, and XP are not capstones |
| **A replacement for Lore at level 1** | Lore is gone (#827) and nothing in the bard's facets is missing at 1 (a little healing, a little fighting, and the breadth the class file gives); a talent minted to fill a slot is the creep the policy forbids |
| **+1 (or +6) to every stat** (ADOM's 50) | Unlikely hero's effects exactly (the commoner's): `talent_alike` would refuse the load |
| **Doubled regeneration** (ADOM's 32 and 40) | Saint (`healing_percent 100`) and Arcane mastery (`mana_regen_percent 100`) are those; a bard is neither, and Fast healer is its rest |
| **A dirge or a frightening tune** (Pathfinder 8 and 14) | Hound's bay (a Barghest's birth sign pick) is the one fear source: a hostile hearer of a lower level flees; a bard's dirge would be it under another kind. One song against foes is enough, and the Lullaby is the bard's (fascinate is the 3.5 bard's first trick) |
| **Suggestion, mass suggestion, a pacifying song** (a hostile made peaceful) | a faction change at talk is a design of its own (what breaks it, what the AI does with a half-friend, what it is worth); deferred to #196 if the owner wants it |
| **A lullaby that puts creatures Asleep** | Asleep is the den's state (no turn, unaware): a room put to sleep is a thief's dream and a bard's free dungeon; Drowsy slows and the fight still has to be won |
| **The Lullaby as the Sleep spell, granted** (`spell = "sleep"`, no new code) | honest and cheap, but it leaves Music unread and the bard's one craft a wizard's spell; a song over hearers is the bard's own and costs one `song` arm |
| **A level gate for the Lullaby** (lower level than the bard, Hound's bay's rule) | the owner: "check, no level gates"; the check is what Warlock uses, and the wiki states its odds |
| **A song paid in Inspiration else mana** (the first draft, Inspire's rule today) | the owner: "no dual resource costs"; each song names one price |
| **Songs on a cooldown** | policy step 4: a cast that costs mana or a pool has no cooldown status besides |
| **Inspire silent** (as today) | a song is sung aloud; the noise is the honest price of a buff with no cooldown (the owner, Q7: "songs are heard") |
| **The Lullaby refused when no hostile hears** | the owner, Q15: "there might be invisible enemies to hear or something. character perception does not determine who might hear"; the refusal would leak, and the hearers are the world's |
| **An instrument carried adds a degree** (the first draft's §6.4) | the owner: "no. bard always sings" |
| **Versatile performance** (Music standing in for Bluff, Negotiation, Intimidate) | a bookkeeping rule the game would have to explain; the bard trains those skills already |
| **Magical secrets** (spells from any class) | the bard studies any scroll already (casting 75, Decipher script, the affinities); no slot is spent on what the rules give |
| **Bonus picks from Scholar and Delver only** (the first draft) | the owner, Q13: "any"; the commoner's gift shared |
| **A song for gold** (Elona's Performer) | economy (rules 1 and 9); the tavern is not in the game |
| **Countersong** (hexes on the bard end sooner) | hexes on the player come from a handful of caster types: niche by rule 6's spirit; `status_eases` exists if a later wave wants it on a pickable |
| **A new `songs` system tag and a pickable to go with it** | songs are class actives on statuses, resources, mana, and hearing: a sub-feature of those, not a system (policy: systems are broad); a pickable would be minted to fill a count |
| **Two talent ids for the two halves of Jack of all trades** | rule 4: a rank; the class table learns to name one (one field) |
| **Levels 31 to 60 as a bard** (the table thinned over sixty) | the owner: "Bard does not gain levels above 30"; the class ends, and the prestige class begins |
| **Banked XP thrown away at the cap** (the level-31 threshold counted from the calling) | needs saved state (the XP at the calling) and takes from the player what the game told them they earned; banking is no state and makes the hidden mechanism worth finding late (`prestige-classes.md` §4.3) |

## 13. Decisions: the owner's answers, applied

The owner's answers on PR #835 (2026-11-02, verbatim: *"Q1: blessing goes / q2: sure. make sure it pull all skills, no matter how many we add later. / q3: nah, no sneak attack / q4: sure / q5: check, no level gates / q6: no dual resource costs / q7: songs are heard / q8: no. bard always sings / q9: sure / q10: it covers styles too / q11: yes / q12: yes / q13: any / q14: yes / q15: yes. there might be invisible enemies to hear or something. character perception does not determine who might hear."*), each read and applied:

| # | Question (first draft) | Answer | Applied |
|---|---|---|---|
| Q1 | Blessing goes from the bard's table? | "blessing goes" | §5: gone; Travelled hands takes the slot (now 3) |
| Q2 | Jack of all trades rank 1 takes up every skill the bard lacks? | "sure. make sure it pull all skills, no matter how many we add later" | §7: the hand-out walks `Content::skills()`; no list; a test adds a fixture skill and sees it taken up |
| Q3 | Sneak attack skipped? | "nah, no sneak attack" | §7: `except = ["sneak_attack"]` stays (decision 7 of the surprise design holds) |
| Q4 | The +5 moves to rank 2? | "sure" | §7: rank 1 the skills, rank 2 the +5 (now 12 and 25) |
| Q5 | The Lullaby is a Willpower check, not a level gate? | "check, no level gates" | §6.3 |
| Q6 | The Lullaby's cost 3 Inspiration else 6 mana? | "no dual resource costs" | §6.2, §6.3: one price a song; Inspire 3 Inspiration, the Lullaby 6 mana (key choice 4; the mapping is one line of data each if the owner prefers it the other way round, §14) |
| Q7 | Songs are heard? | "songs are heard" | §6.2: `Act::Sing`, `SONG_NOISE` 80 |
| Q8 | An instrument adds a degree? | "no. bard always sings" | §6.4: dropped whole |
| Q9 | Class tables name a rank? | "sure" | §7: `rank` on `AbilitySpec` |
| Q10 | Travelled hands covers melee, launchers, thrown, unarmed only? | "it covers styles too" | §8: every weapon skill and every style but the schools and hidden styles; shields too, since the walk is generic |
| Q11 | Music becomes `kind = "rules"`? | "yes" | §6.1 |
| Q12 | Virtuoso widens to every song? | "yes" | §6.5 |
| Q13 | Bonus trees stay Scholar and Delver? | "any" | §5: `bonus_talents` without `trees` |
| Q14 | Degrees are time, not strength? | "yes" | §6.2 |
| Q15 | The Lullaby sung to an empty room spends its cost and makes its noise? | "yes. there might be invisible enemies to hear or something. character perception does not determine who might hear" | §6.3: no refusal; `noise::hearers` on ground truth decides who is checked |

The second round's direction (the bard at 30, the prestige classes, the Hunter, the curve to 100) is §5 here and `prestige-classes.md` whole. Lore's removal (#827) is applied in §5.

## 14. Open questions left, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| Q16 | **Which song takes the pool**: Inspire 3 Inspiration and the Lullaby 6 mana (as proposed), or the other way round? | **as proposed**: the pool is named for Inspire and built by kills, so the buff rises mid-fight and the Lullaby opens the fight on mana; both are one line of data |
| Q17 | **The compressed cadence** 1, 3, 6, 9, 12, 16, 20, 25, 30 (the old halved), or an even one (1, 4, 8, 12, 16, 20, 24, 27, 30)? | **halved**: it keeps the early levels busy and the capstones spaced, as the sixty-level tables do |
| Q18 | **Bonus picks at 5, 10, 15, 23, 28** (the old halved) from any tree? | **yes**; five picks as every class has |
| Q19 | **Rule 1's wording** becomes "the last third of a class's table" (and "the last half" for the XP bar), so a 30-level class is judged as a 60-level one is? | **yes**, in B6, one sentence in `progression.md` |

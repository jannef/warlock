# Warlock — The bard: a jack of all trades, mediocre at everything

Design document for the owner's review (2026-11-02; issue #819). Written against `origin/main` at f3846261 (after the #776 review's cull, #798, which emptied the bard's levels 40 and 50; the class ability gaps report is draft PR #802). Nothing here is implemented; §11 cuts it into issues. It builds on what is there: the bard's class file (`assets/data/classes/02-bard.toml`), its class abilities (`data/talents/`), the Inspiration pool (`docs/dev/resources.md`), skills and their level-up rolls (`docs/dev/skills.md`), weapon skill ranks (`docs/dev/weapon-skills.md`), statuses (`docs/dev/status-effects.md`), hearing (`docs/dev/hearing.md`), the one cast path (`docs/dev/spells.md`), class tables and the coverage policy (`docs/dev/progression.md`, rules 1–9 of #776, which bind every proposal below). Every number is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-11-02, verbatim): *"bard: create a design issue to come up with class fantasy for the class. I mean leaning towards mediocre in everything."*

## Contents

1. Summary
2. The fantasy in one paragraph, and how the data already says it
3. Research: bards and jacks of all trades elsewhere, tied to decisions
4. What exists today
5. The table proposed: stay, change, go, and what fills 40 and 50
6. Songs: Music, Inspire reworked, Lullaby, the instrument, Virtuoso
7. Jack of all trades in two ranks, and class tables that name a rank
8. Travelled hands
9. What the player sees; determinism, saves, replays
10. The wiki, the spec, and the docs
11. Implementation plan
12. Rejected ideas, with the reason
13. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **The fantasy** | the bard knows a little of everything and masters nothing: a rank in every weapon and none above the middle, small spells, a lock picked badly, a scratch closed, a thing known for what it is, and the one thing that is theirs alone, the song. Breadth is the power; every single thing a bard does, a specialist does better. §2 |
| **The table** (nine levels, one facet each) | 1 Lore, Mend, Trained wind (stay); 6 **Travelled hands** (new; Blessing goes); 12 Fast healer (stays); 18 Inspire (stays, reworked as a song); 25 **Jack of all trades rank 1** (reworked: every skill you lack, taken up); 32 Throw your voice (stays); 40 **Lullaby** (new, the gap); 50 **Jack of all trades rank 2** (the gap: every skill counts 5 higher, today's effect moved); 60 Virtuoso (stays, widened to every song). §5 |
| **Songs** | the Music skill, which no rule reads today, becomes the bard's craft: a `song` talent effect (Inspire's `inspire` kind generalised) sung for 3 Inspiration while the bard holds that much, else for mana, in a chat's time, **heard** as a voice (`SONG_NOISE` 80: those who hear it come to look); Music's value adds degrees to what a song gives (+1 at 50, +1 at 100, +1 per 100 after) and, for the Lullaby, makes the hearers' check harder (+4 at 100, +8 at 200); an **instrument** carried (the lute, `instrument = true`) adds one degree. Music's `kind` becomes `rules`. §6 |
| **Lullaby** (40) | a song over everyone who hears it: each hostile hearer checks Willpower against 13 plus the Music craft; failed, it is **Drowsy** (the status the Sleep spell gives: everything three times as slow), one degree plus Music's and the instrument's, at most three; made, it only knows where the bard is. 3 Inspiration else 6 mana, a chat's time, no cooldown (time and the noise are the price). §6.3 |
| **Jack of all trades** (25, 50) | rank 1 at 25: **you take up every skill you lack** (handed out once, `1d4+1` each from the levels stream, skipping Sneak attack: the thief's, by the surprise design's decision 7): the bard can now pick a lock, disarm a trap, treat a wound, steal, forage, and notice traps in passing, all badly, as Pathfinder's bard may "use any skill untrained" at 10. Rank 2 at 50: **every skill counts 5 higher** (today's `trained_skills 5`, moved), as Pathfinder's "take 10 on any skill" at 19. The class table names the rank: `{ level = 50, talent = "jack_of_all_trades", rank = 2 }` (the rule of #776 rule 4: one effect in two steps is a rank, never two ids). §7 |
| **Travelled hands** (6) | every weapon skill of a weapon (melee, launcher, thrown, unarmed; not styles or schools) the bard holds no rank in rises to rank 1, handed out once (`weapon_ranks { roles, to = 1 }`): +1 to hit with everything, and the defaults rise as any rank gained raises them. ADOM's bard gets 1d3 levels in six random weapon skills at 12; this gives one in all of them, so the wiki can state it. §8 |
| **Goes** | Blessing (level 6): a priest's abjuration shared with the paladin, off the bard's schools (enchantment 2, divination 1), and the second +1 to hit beside Inspired; its slot takes the weapons facet. The talent file stays (the priest's and paladin's tables name it). §5 |
| **Mediocre by data, not by cap** | nothing stops a bard training a weapon to rank 10 or a skill to 200; the class only starts low (hit points 2, Strength −1, Toughness −1, casting 75, mana 5) and spreads wide (4 rolls a level, 1 creation pick more, 8 trained skills, 3 optional). No "master of none" cap (§12): the player chooses, and unbalanced choices are fine (the owner). §2 |
| Coverage policy | no new `[[system]]`: songs extend `status`, `resources`, `mana`, and `hearing`; the skill grants extend `skills` and `weapon_skills`. No pickable talent is minted (the bar is one, and a count is never a reason). Rules 1–9 checked per entry in §5. |
| Slices | five issues: B1 songs (Music, the `song` effect, the instrument, the noise, Virtuoso widened), B2 Travelled hands and Blessing out, B3 Jack of all trades in two ranks and class tables naming a rank, B4 Lullaby (the one action), B5 the wiki, the spec, and the docs. No saved state is added; one action. §11 |

**Key choices, and why**

1. *Mediocre is breadth, written as grants.* ADOM's bard gets random weapon skills, random spells, random skills, and the two D&D bards get "a little to everything you lack" (5e's Jack of All Trades, Pathfinder's use-any-skill). Warlock's version hands out the breadth as data (every weapon skill to 1, every skill taken up) rather than drawing six at random, because the wiki states rules exactly and "six random" is a lottery the game would have to be vague about.
2. *The song is the one thing the bard owns.* Inspire exists, Music exists and reads nothing, the lute exists "for the tunes to come". One `song` effect kind serves Inspire and the Lullaby, Music's value is what makes a bard's song better than anyone else's, and the instrument is the lute's reason to be carried.
3. *The Lullaby is a check, not a level gate.* Hound's bay (a birth sign's) frightens everything of a lower level; a bard should be able to fail. A Willpower check per hearer against 13 plus the Music craft lets the weak-willed drop and the strong shrug it off, and the wiki states the odds (`check::ways`).
4. *Two ranks, one id.* The level-25 and level-50 halves of Jack of all trades are one talent in two ranks; the class table gains a `rank` field, which costs one line in the parser and one in `class::grant` and is the shape #776's rule 4 asks for.
5. *Nothing at 40 or 50 pays in gold or XP.* Honeyed words (40: shop favour and quest gold) and Legend (50: quest XP, Presence +1) were cut by #798 for rules 1 and 9; their replacements change a fight (the Lullaby) and a dungeon (every skill +5).
6. *One action, no state.* Only the Lullaby is a new action; skills, weapon ranks, and talent ranks are saved already.

## 2. The fantasy in one paragraph, and how the data already says it

A bard has been everywhere and done a little of everything: fought with whatever came to hand, read scrolls not meant for them, picked up a lockpick once, bound a wound, sung for supper. They are not the warrior, the wizard, the thief, the priest, or the scholar; they are a little of each, and the one thing nobody else does is sing. In play: a bard tries everything, fails more than a specialist would, and is never stuck; their fights are won by a buff, a lullaby, and a thrown voice more than by the blade.

What the class data says already (the "mediocre" is mostly written; this design completes it):

| Facet | The bard today | The specialist | Reading |
|---|---|---|---|
| Body | `hp = 2`, Strength −1, Toughness −1, Dexterity +1 | barbarian 8, warrior 7 (Strength 1d3+1, Toughness 1d3); wizard 0 | a weak body, not the weakest |
| Weapons | Short blades 1, Daggers 1d2, Duelist 1; everything else 0 | warrior Long blades 2 and a shield; thief Daggers 2, Thrown 1 | one blade, a little; §8 gives one rank in all |
| Magic | `casting = 75` (caster level three quarters of the character's), `mana = 5`, Restoration 1, affinity enchantment 2 and divination 1; Mend at 1, Blessing at 6 | wizard 100 and `mana = 12`, Evocation 2; warrior and thief 25 and `mana = 1` | a half-caster who learns from any scroll (study, Decipher script) |
| Skills | **4 rolls a level** (the most of any class), known for Bluff and Music (`2d6+10`), Appraise, Negotiation, Decipher script, Initiative, Knowledge, Tumble (`2d4+4`), optional Listen 30 %, Spot 30 %, Steal 15 % | thief 3 rolls and 11 skills, 6 known for; most classes 2 | breadth of learning is the class's identity (design `progression.md` §5: "one class with many rolls: the bard") |
| Talents | **one creation pick more** (3; a human bard 4); bonus picks from Scholar and Delver at 10, 20, 30, 45, 55 | everyone 2; the commoner's bonus picks from any tree | the lucky class (ADOM gives bards a bonus talent) |
| Talk | Bluff, Negotiation, Knowledge; Throw your voice at 32 | — | the quick tongue; Bluff and Intimidate have no action yet (`Ability::Skill`, "not usable yet") |
| Song | Inspire at 18 (+1 to hit, Dodge +1, 5 minutes; 3 Inspiration else 3 mana), Virtuoso at 60 (+2/+2, 10 minutes, free); Inspiration built by kills, max 5, a point lost every 30 base turns; **Music reads nothing**; the lute "waits for the Music skill's use" | — | the one thing nobody else has, half built |
| Experience | 100 % | wizard 125, thief 95 | middling |

## 3. Research: bards and jacks of all trades elsewhere, tied to decisions

| Source | What it does | What this design takes, and what it leaves |
|---|---|---|
| **ADOM's bard** (Ancardia wiki, *Bard*, *Class power*): class powers at **6** +1 skill increase every level-up; **12** 1d3 levels in six randomly chosen weapon skills; **18** four random spells; **25** six random skills received or improved; **32** hit point regeneration doubled; **40** power point regeneration doubled; **50** +6 to all attributes; bards get more skill increases and a bonus talent; the wiki notes the gains "can be different in different games" | the one roguelike bard, and the model for the cadence (design `progression.md` §4.1 took ADOM's seven levels). **Taken**: 12 → Travelled hands (§8), every weapon skill rather than six random; 25 → Jack of all trades rank 1 (§7), every skill rather than six random (the wiki states a rule; a random six is what the wiki could only call chance). **Left**: 6 (+1 roll a level) was the bard's Wide learning until #780 cut it as Quick learner's clone (`skill_rolls`); 18 (random spells) is what Warlock's study does already, with Decipher script and the affinities, and no slot is spent on it; 32 and 40 (regeneration doubled) are Saint's and Arcane mastery's effects (`healing_percent 100`, `mana_regen_percent 100`) and a bard is neither healer nor mage; 50 (+6 all stats) is Unlikely hero (+1 all) writ large, and rule 3 forbids the clone |
| **D&D 3.5 bard** (SRD): bardic knowledge; bardic music, once a day per level: countersong 1, fascinate 1, inspire courage +1 at 1 (+2 at 8, +3 at 14, +4 at 20), inspire competence 3, suggestion 6, inspire greatness 9, song of freedom 12, inspire heroics 15, mass suggestion 18; d6 hit die, three-quarter attack, 6 + Int skill points, arcane spells to 6th | **Taken**: fascinate → the Lullaby (§6.3; Warlock has Drowsy and the Sleep spell, and a song that holds hearers in place is a song that puts them half to sleep); inspire courage's growth → Music's degrees and Virtuoso (§6); bardic knowledge → Lore (given at 1) and the Knowledge skill, which exist. **Left**: countersong (hexes on the player come from a few caster types; niche by rule 6's spirit); suggestion and mass suggestion (a pacify mechanic: a faction change at talk, a far larger design, §12); inspire competence (the skill facet is Jack of all trades) |
| **D&D 5e bard** (roll20 compendium): Jack of All Trades at 2 (half proficiency to every check without it); Song of Rest at 2 (d6 → d12); Expertise 3 and 10; Font of Inspiration 5; Countercharm 6; Magical Secrets 10 (spells from any class); Superior Inspiration 20; the inspiration die d6 → d12 | **Taken**: Jack of All Trades's shape, "a little to everything you lack", is rank 1 of §7; Song of Rest is Fast healer, which stays at 12. **Left**: Magical Secrets (any scroll is already the bard's to study); Superior Inspiration (a free point at a fight's start: Inspiration is built by kills and decays, which is the design's shape, `docs/dev/resources.md`); Expertise (a doubled skill is the specialist's, not the bard's) |
| **Pathfinder bard** (Core Rulebook, aonprd): versatile performance at 2, 6, 10, 14, 18 (Perform in place of other skills); lore master 5; well-versed 2; **jack-of-all-trades: 10 use any skill untrained, 16 every skill a class skill, 19 take 10 on any skill**; performances: distraction and fascinate 1, inspire courage, inspire competence 3, suggestion 6, dirge of doom 8, inspire greatness 9, soothing performance 12, frightening tune 14, inspire heroics 15, mass suggestion 18, deadly performance 20 | **Taken**: the three-step jack-of-all-trades is the model for the two ranks (§7): 10 → rank 1 at 25 (every skill taken up: in Warlock an untrained skill does not exist, so "use untrained" is "trained a little"); 19 → rank 2 at 50 (every skill +5). **Left**: 16 (every skill a class skill: the +1 a level-up roll gets on a class's skill) folded away, the bard's four rolls are its learning; versatile performance (one skill standing in for another: information-free but a bookkeeping rule the game would have to explain); dirge and frightening tune (Hound's bay, a birth sign's pick, is the one fear source; a second would be its clone in all but kind, §12); soothing performance (Mend and Fast healer are the bard's healing); deadly performance |
| **DCSS's Wanderer** (CrawlWiki): a background of random skills (ten levels split over a "good" item set, two "decent" ones, and the rest at random), random items, random stats summing to 12; "the possibility for some awful starts" | breadth as a lottery at creation. Warlock's bard already has its small lottery (the optional Listen, Spot, Steal) and the owner's rule that the UI never speaks of chance's source (#256). **Left**: more randomness at creation; the grants of §7 and §8 are fixed, so a bard is never an awful start |
| **Tales of Maj'Eyal's Adventurer** (te4 wiki): talent trees from any unlocked class, extra category points, a kit of every weapon kind, "by no means balanced" | breadth as picks. Warlock's bard has one creation pick more and bonus picks from two trees; the commoner owns "any tree" ("the commoner's gift", `classes/README.toml`). **Left**: bonus picks from any tree for the bard (Q13: Scholar and Delver stay) |
| **Elona's Performer** (Elona wiki): play an instrument to an audience for tips and thrown items; listeners of a level above the performer's throw stones | a song for gold is economy, which rule 1 bars from 40 up and rule 9's spirit keeps out of the tables; the instrument as the thing that makes the music better is taken (§6.4). The "audience of a higher level resists" idea becomes the Lullaby's Willpower check rather than a level gate (key choice 3) |

## 4. What exists today

| Part | Today |
|---|---|
| The table | 1 Lore (Scholar, given), Mend (spell), Trained wind; 6 Blessing (spell); 12 Fast healer (Vigour, given; #790 cut the clone Song of rest); 18 Inspire; 25 Jack of all trades (`trained_skills 5`); 32 Throw your voice; **40 empty** (Honeyed words cut by #798: `favour_percent 15`, `quest_gold_percent 25`, rules 1 and 9); **50 empty** (Legend cut by #798: `quest_xp_percent 50`, `stat presence 1`, rule 1); 60 Virtuoso. Wide learning (6, `skill_rolls 1`) was cut by #780 as Quick learner's clone, and Blessing holds 6 since (#802's report) |
| Inspire | `inspire { status = "inspired", mana = 3, resource = "inspiration", cost = 3 }` (`talent::Inspire`), `World::inspire` (`world/class_ability.rs`): spends the resource when held, else the mana; afflicts `inspired` (`max = 1`, `eases = 300`, `combat = { to_hit = 1, dodge = 1 }`); a Presence training mark (`StatUse::Inspired`); priced as a chat (`chat = 500`, half a base turn). **No noise is made.** Virtuoso: `virtuoso { status = "inspired_greatly" }` (`eases = 600`, +2/+2) and `Inspire::instead` cleared, so it costs nothing |
| Inspiration | `resources.toml`: `max = 5`, `start = 0`, `decays = 180`, `builds = [{ on = "kill", amount = 1 }]`; granted by Inspire's `grants_resource` |
| Music | `skills.toml`: `kind = "action"`, no `mark`, no `benefit`: **nothing reads it**; the abilities menu lists it under the skills as "not usable yet" (`Ability::Skill`, `Unavailable::NotYet`), beside Bluff, Intimidate, Knowledge, Decipher script, Handle animal, Metalcraft, Woodcraft. The bard is known for it (`2d6+10`) |
| The lute | `objects/lute.toml`: class `miscellaneous`, 1.5 kg, value 40, in the bard's kit; "Playing it waits for the Music skill's use; until then it is carried for the tunes to come" |
| Skills | an untrained skill does not exist for the actor (never shown, gives nothing; `Skills`); "trained at all" is `Skills::trains_effect` (Spot, Detect traps, Steal, Heal, Pick lock, Disarm, Forage, Sneak attack's first die); `skill { skill, value }` on talents and items adds to a skill "an untrained one from 0" without training it; `train_skill { dice }` (Apprentice) trains the chosen skill from the levels stream (`World::train_chosen`); `trained_skills { value }` (Jack of all trades) adds to every trained skill (`Actor::skill_effect`). Level-up rolls go on trained skills only; the Skills screen lists them |
| Weapon skills | ranks 0 to 10 per skill (`WeaponSkills`), raised by marks (`train` → `raise`: the defaults rise, marks reset); a talent's `weapon { ranks }` reads a **held** rank higher and leaves rank 0 at 0 (`TalentMods::raised`); weapon abilities start at rank 4 (`data/weapon_abilities/`: ranks 4 to 10); Daggers rank 1: +1 to hit, nothing else |
| Sleep and Drowsy | the Sleep spell (`spells/sleep.toml`: level 6, enchantment, a burst of radius 1 at range 8, `drowsy` severity 2, Presence lengthens it); `drowsy` (`max = 3`, `eases = 12`, `cost = 300`: every action three times as dear, a degree each 12 s) |
| Hearing | `noise::noise(act)` → `noise::hearers` (who hears it, by loudness less `STEP_LOSS` 10 a cell through what passes sound; nobody on the overworld); a voice is `VOICE_NOISE` 50, a howl `HOWL_NOISE` 150; Hound's bay (`world/sign_picks.rs`) is the one ability that acts on hearers: every hostile that heard it, of a lower level than the player, Frightened |
| Checks | `check::check(rng, modifier, difficulty)`: 2d12 plus a stat modifier, natural 2 fails, 24 succeeds; `check::ways` the odds the wiki prints; the surprise design's initiative check is "the target rolls Dexterity against 13 plus the attacker's craft" |
| Class tables | `ClassDef::abilities: Vec<(u8, String)>` (`{ level, talent }`); `class::grant` gives what the table names up to the level **at rank 1** (`hand_out` for what a rank gives once); `check_table` at load; a talent held already brings a pick in its place (pickables only) |

## 5. The table proposed: stay, change, go, and what fills 40 and 50

| Level | Today | After | Facet | Rules 1–9 |
|---:|---|---|---|---|
| 1 | Lore (Scholar, given) | **stays** | lore | a pickable given: no clone (rule 3 is for clones, not hand-outs); identification is the item game's rule, not a readout (rule 8) |
| 1 | Mend (spell) | **stays** | a little healing | shared with the priest |
| 1 | Trained wind | **stays** | a little fighting (stamina) | every fighting class's |
| 6 | Blessing (spell) | **goes**; **Travelled hands** (§8): rank 1 in every weapon skill of a weapon held at 0 | a little of every weapon | a hand-out, not a clone of `weapon { ranks }` (which never raises 0); not information; not niche |
| 12 | Fast healer (Vigour, given) | **stays** | rest | Song of rest's replacement by #790; a bard's "song of rest" |
| 18 | Inspire | **stays, reworked** (§6.2): a `song`, heard, longer with Music and an instrument | the song | `status`, `resources`, `mana`, `hearing` |
| 25 | Jack of all trades (`trained_skills 5`) | **reworked** (§7): rank 1, you take up every skill you lack (`1d4+1` each) | a little of every skill | changes what the character can do in a dungeon (locks, traps, wounds, theft, forage); Sneak attack skipped (decision 7 of the surprise design) |
| 32 | Throw your voice | **stays** | trickery | `hearing`, `stealth` |
| **40** | empty (Honeyed words cut) | **Lullaby** (§6.3): hostile hearers who fail a Willpower check are Drowsy | the song, against foes | rule 1: changes a fight; no gold, no XP; paid in Inspiration or mana, no cooldown (policy step 4) |
| **50** | empty (Legend cut) | **Jack of all trades rank 2**: every skill counts 5 higher (today's effect, moved) | every skill, better | rule 1: a dungeon-changer across every check (locks, traps, Stealth, Tumble's Dodge, Heal, Spot, Knowledge's resist, Concentration's mana, Decipher's study, Swim, Balance's burden); rule 4: a rank, not a second id |
| 60 | Virtuoso | **stays, widened** (§6.5): every song greater and free | the song, mastered | `status`, `mana`, `resources` |

Bonus picks stay Scholar and Delver at 10, 20, 30, 45, 55 (Q13). The class file's `skills`, `optional_skills`, `rolls = 4`, `creation_talents = 1`, `weapon_skills`, `styles`, `affinity`, `casting = 75`, and the kit are unchanged: they are the breadth already.

**Why Blessing goes and Fast healer stays.** Both are shared entries, and one slot had to give way to the weapons facet, which nothing in the table covers. Blessing is abjuration (the priest's at 3, the paladin's at 6), off the bard's schools, and gives the same +1 to hit the bard's own Inspired gives; a bard who wants it studies a scroll of it like any other. Fast healer is the bard's healing by rest (D&D's Song of Rest) and nothing else in the table heals the bard between fights. The `blessing` talent file stays for the two tables that name it; a saved bard holding `blessing` keeps it (`class::grant` takes nothing away; a restore of a save below 6 simply does not get it).

## 6. Songs: Music, Inspire reworked, Lullaby, the instrument, Virtuoso

### 6.1 Music, the skill

```toml
[[skill]]
id = "music"
name = "Music"
description = "Singing and playing an instrument: how far your songs carry into a heart, and how long they stay there."
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

`kind` changes from `action` to `rules` (the songs are class actives and read it; the "not usable yet" row for Music goes from the abilities menu, Q11). `skills::Effect::SongDegrees`, `Effect::SongCraft`; `Use::Song` (a song sung, whatever it does). The bard is known for Music, so a level-up roll on it throws a bigger die: a bard who spends rolls on Music has it at 100 around level 20 to 25 (as the surprise design reckons for the thief's Sneak attack at three rolls; the bard has four).

### 6.2 The `song` effect, and Inspire on it

Inspire's `inspire { status, mana, resource, cost }` becomes the general kind (a rename of a content effect kind; saves name talents by id and are untouched):

```toml
# talents/inspire.toml, after
effects = [
    { kind = "grants_resource", resource = "inspiration" },
    { kind = "song", id = "inspire", on = "self", status = "inspired", mana = 3, resource = "inspiration", cost = 3 },
]
```

| Field | What |
|---|---|
| `id` | the active's id (`use:inspire`, `use:lullaby`); the abilities menu row under the class source, as today |
| `on` | `self` (Inspire: the singer suffers the status) or `hearers` (the Lullaby: every hostile that hears the song, §6.3) |
| `status`, `severity` | the status given and its base degrees (default 1); the song's degrees are `severity` + Music's `song_degrees` + the instrument's 1 (§6.4), capped by the status's `max` |
| `mana`, `resource`, `cost` | as Inspire's today: `cost` of `resource` while that much is held, else `mana` |
| `check`, `difficulty` | `hearers` only: the stat each hearer checks (`willpower`) against `difficulty` (13) plus Music's `song_craft` |

`World::sing(song)` (`world/song.rs`, the one place a song is sung; `World::inspire` becomes its `on = self` arm): pay (resource else mana, refused with "You have too little mana." as today), **make the noise** (`Act::Sing`, `SoundKind::Voice`, `SONG_NOISE` 80: it carries 8 cells, farther than a chat's 50 and well short of a howl's 150; creatures that hear it come to look, as they do for a thrown voice), then the statuses, then the Presence mark (`StatUse::Inspired`, as today), priced as a chat. Refused with no time on the overworld (silent, as a thrown voice is) and while Silenced (`silenced` gains `song` in its `blocks`).

**Inspire after**: Inspired for 5 minutes a degree (`inspired` gains `max = 3`): one degree at Music below 50, two at 50, three at 100 (and at any Music with the lute carried, one more, to the cap). Virtuoso's `inspired_greatly` (`max = 3` too) the same, at 10 minutes a degree. The combat bonus is the status's and does not grow with degrees (Inspired is `max = 3`, `stacking = "refresh"`: degrees are time, as the witch's hexes' are).

**What changes for a bard today**: Inspire is heard (Q7). A bard who sings in a quiet corridor tells the level where they are, which is the fantasy (a bard is loud) and a real cost that Inspire lacks today.

### 6.3 Lullaby (40)

```toml
[[talent]]
id = "lullaby"
name = "Lullaby"
tree = "class"
granted = true
description = "A slow song for those who mean you harm (an ability): every foe that hears it and cannot shake it off grows drowsy, slow in all it does, a while. Sung for inspiration when you have enough, else for mana."
effects = [{ kind = "song", id = "lullaby", on = "hearers", status = "drowsy", mana = 6, resource = "inspiration", cost = 3, check = "willpower", difficulty = 13 }]
systems = ["status", "resources", "mana", "hearing"]
```

| Rule | Decision | Where |
|---|---|---|
| Who | every creature that **hears** the song (`noise::hearers` of the Sing noise, before it is made, as Hound's bay does) and is hostile to the bard (`World::stance`); friends and the peaceful hear it and nothing more; a sleeper (Asleep) hears as a noise wakes it and is checked like anyone (a lullaby does not deepen a sleep: the Asleep status is the den's, not this song's) | `song.rs` |
| The check | each such hearer: `check::check(rng, its Willpower modifier, 13 + the bard's song_craft)`, two gameplay draws apiece, in actor id order (deterministic). Made: nothing (a message, §9). Failed: `afflict(hearer, drowsy, degrees, Some(bard))` with the song's degrees (1 + Music's + the instrument's, at most Drowsy's 3). A creature already Drowsy is refreshed (the status's `stacking`) | `song.rs` |
| What Drowsy does | everything the creature does costs 300 % (`cost = 300`, the scheduler's one price hook, `World::burdened`), a degree easing every 12 s: one degree is two base turns at a third of its speed, three degrees six. It still acts, slowly; it is not asleep and not unaware (the surprise design's `unaware` is untouched: a drowsy foe that saw the bard is aware) | `data/status.toml` (exists) |
| Cost | 3 Inspiration while the bard holds 3, else 6 mana (Q6); a chat's time (500); **no cooldown**: a pool or mana is the price (policy step 4) and the noise is the rest of it | `song` data |
| Refusals | no hostile hears it: sung anyway (a bard may sing to an empty room; the mana is spent, the noise made, the mark earned); the overworld and Silenced refuse with no time, as above | `song.rs` |
| Odds (the wiki states them) | a hearer at Willpower +0 against 13 (Music below 100): resists 78 of 144 (54 %); against 17 (Music 100): 36 of 144 (25 %); against 21 (Music 200): 10 of 144 (7 %); a hearer at Willpower +4 against 17: 78 of 144 (54 %). The strong-willed (the drow, the undead casters, the fey lords) shrug it off; the kobolds, rats, and goblins drop | `check::ways` |

**Why Drowsy and not Asleep.** Asleep is the den's state (no turn, unaware, woken by noise): a song that put a room to sleep would be the thief's Lurk and Sneak attack for everyone, and it would let a bard walk through Cealdhlāw's dens untouched. Drowsy slows; the bard still has to win the fight or leave it. The Sleep spell gives the same status and the name says what the song is.

**Why hearers, not a burst.** The Sleep spell is an aimed burst of radius 1 at range 8 (two or three creatures); a song fills the hall. A song also goes through a door and round a corner, which a spell does not, and it tells everyone in earshot where the singer is. The two are different tools, and a bard who studies the scroll has both.

### 6.4 The instrument

`objects/lute.toml` gains `instrument = true` (`ObjectDef::instrument`; the only one so far; a horn or a drum would be one line each). A song sung with an instrument **carried** (anywhere in the pack: the bard takes it out and plays; no slot, no readying) has one degree more (Q8). Read in `World::sing` only, through `Actor::carries_instrument`. The lute weighs 1.5 kg and is worth 40 gold: a bard who sells it sings a little shorter. The object's comment ("Playing it waits for the Music skill's use") is paid.

### 6.5 Virtuoso (60), widened

```toml
effects = [{ kind = "virtuoso", status = "inspired_greatly" }]
```

stays, read in `World::sing`: **every** song costs nothing (`Song::instead` and `mana` set aside, as Inspire's are today) and has one degree more, and Inspire gives `inspired_greatly` in place of `inspired`. The description becomes "Your songs lift higher and last longer, and cost you nothing." A Virtuoso's Lullaby at Music 200 with the lute: 1 + 3 + 1 + 1, capped at Drowsy's 3; the cap is the data's to raise.

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
# Level 25: every skill you lack, taken up a little (the levels stream; never Sneak attack, the
# thief's). Said once: "You find you know a little of everything."
effects = [{ kind = "take_up_skills", dice = "1d4+1", except = ["sneak_attack"] }]

[[talent.rank]]
# Level 50: every skill you have counts a little higher.
effects = [{ kind = "trained_skills", value = 5 }]
```

| Rule | Decision | Where |
|---|---|---|
| **Rank 1** (25): take up every skill | a hand-out as `train_skill` is (`World::train_chosen`): for every skill in `skills.toml`, in file order, that the player is not trained in and that `except` does not name, roll `dice` from the **levels stream** (`levels_rng`, made on the first draw) and `Skills::add` it. About twenty skills for a bard (eight trained, up to three optional, Sneak attack skipped). From then on they are trained: shown on the Skills screen (it pages; `ui/skills_screen.rs`), marked by use, rollable at a level (`advance:SKILL`, at the ladder's lowest die and no +1, since the class does not train them), and **trained at all** for every rule that asks (`Skills::trains_effect`): the Heal ability, Pick lock, Disarm, Steal, Forage, noticing hidden features and traps in passing, Concentration's mana, Spellcraft, Survival, Swim, Woodcraft, Handle animal, Escape artist, Intimidate (when it has an action) | `world/jack.rs::take_up_skills`, from `class::hand_out` |
| **Sneak attack** skipped | the surprise design's decision 7 (the owner: "only thief gets sneak attack guaranteed"); `except` is data, so the owner may empty it (Q3). Initiative the bard trains already | data |
| **Rank 2** (50): every skill +5 | today's `trained_skills 5`, moved: with every skill trained, it is +5 to all of them, as Pathfinder's "take 10 on any skill" at 19 | `Actor::skill_effect` (exists) |
| **The class table names a rank** | `{ level = 50, talent = "jack_of_all_trades", rank = 2 }`: `AbilitySpec` gains `rank` (default 1); `ClassDef::abilities: Vec<(u8, String, u8)>`; `check_table` checks the rank is within the talent's `ranks` and that a higher rank comes after a lower one of the same id; `class::grant` gives rank `r` when the level comes to one holding `r − 1` (and every missing rank below it on a restore, each handed out, so a save below 50 that loads at 50 gets both); the news "You gain Jack of all trades." / "Your Jack of all trades grows." A table that names a rank of a pickable (none does) would follow `held_already` as today | `content/class.rs`, `world/class.rs` |
| Why not two ids | #776's rule 4: "one effect, two ids, chained, is a rank"; the class tree has no `needs` to trip `talent_alike`, but the shape is the same, and ranks are what the Talents page and the wiki already show ("Jack of all trades 1/2") | — |
| Numbers | a level-25 bard's new skills are 2 to 5 each (a thief's Open lock starts at 12 to 22); at 50 they count 7 to 10 plus whatever rolls went there. Picking a locked door of difficulty 14 at Open lock 3 is a Dexterity check at +0 or so against 14: about 45 %; a thief at 60 makes it nearly always. That is the fantasy: the bard can, badly, and time is the penalty (the owner's no-anti-farming rule: a lock may be tried again) | — |

**What the hand-out draws.** Twenty-odd rolls of `1d4+1` from the levels stream at level 25: every later levels-stream draw of that run (stat checks, skill rolls) moves, as any talent's `train_skill` does today; the gameplay stream is untouched. Fixtures without `skills.toml` entries hand out nothing.

## 8. Travelled hands

```toml
[[talent]]
id = "travelled_hands"
name = "Travelled hands"
tree = "class"
granted = true
description = "You have fought with whatever came to hand: you know every weapon a little."
effects = [{ kind = "weapon_ranks", roles = ["melee", "launcher", "thrown", "unarmed"], to = 1 }]
systems = ["weapon_skills", "combat"]
```

| Rule | Decision | Where |
|---|---|---|
| What | a hand-out once, at level 6: every weapon skill of those roles the bard holds at rank 0 is **raised to rank 1** (`WeaponSkills::raise` per skill, in file order, so its defaults rise as any rank gained raises them; marks reset as `raise` does). Styles (`style`), schools (`school`), shield blocking and bashing are left alone: a style is a way of fighting, not a weapon handled; the bard's Duelist stays its one | `world/class.rs::hand_out` → `WeaponSkills::raise_to` |
| What rank 1 gives | by the rank tables (`weapon_skills.toml`): Daggers +1 to hit; the light blades, heavy, middling, launchers, thrown, unarmed likewise around +1 to hit at rank 1 and nothing else; weapon abilities begin at rank 4, so no menu row appears. A bard who picks up a found long sword, a bow, or a sling hits with it a point surer than an untrained anyone: mediocre, everywhere | data (exists) |
| A rank already held | untouched (short blades, daggers); `to` is a floor, never a raise of a held rank | — |
| Why not `weapon { role, ranks = 1 }` | that effect reads a **held** rank higher and leaves 0 at 0 (`TalentMods::raised`), so it would give the bard nothing on the weapons it lacks, which is the point. A real rank is also trainable on: marks on a rank-1 skill carry it to 2 | — |
| Why level 6 | ADOM's bard gets its random weapon skills at 12, but 12 is Fast healer's, which stays, and 6 was Blessing's, which goes (§5). A level-6 bard with a rank in everything is a bard who stops hoarding daggers | — |
| Draws | none (a flat raise); the news "Your hands know every weapon a little." (one line, not one per skill: `WeaponSkills::raise_to` returns what rose and `say_news` folds them) | — |

## 9. What the player sees; determinism, saves, replays

| Where | What |
|---|---|
| Messages | Inspire as today ("Your song lifts your heart.", the status's `start`); the Lullaby: "You sing a lullaby." then, for those seen, "The kobold's eyes grow heavy." / "The orc shakes its head." (made), as Hound's bay names those it saw flee; nothing for hearers out of sight (they are not shown; the noise is theirs to answer). Travelled hands: "Your hands know every weapon a little." Jack of all trades rank 1: "You find you know a little of everything." Rank 2: "Your Jack of all trades grows." No number, no roll, no degree count |
| Look mode | "drowsy" on a creature that is (the status's `seen`, as today); nothing says who sang it |
| The abilities menu | Inspire and Lullaby under the class source (`Ability::Class`), greyed "not enough inspiration" / "you have too little mana" as Inspire is today (`Unavailable::NoMana` only when the resource falls short); Music's "not usable yet" row goes (`kind = "rules"`) |
| The Skills screen | after 25, every skill (it pages); the character screen's skill count grows |
| The status bar | `Inspiration 3` as today; Inspired and Drowsy as today |
| Dumps | `abilities` lists the two songs; `skills` lists the taken-up skills; `talents` shows `Jack of all trades 1/2`; `actors` shows a creature's Drowsy with its degrees; JSON `abilities` gains `lullaby` |
| Determinism | the Lullaby's checks draw two gameplay draws per hostile hearer, in actor id order, only when sung; Inspire draws nothing (as today); Travelled hands draws nothing; Jack of all trades rank 1 draws from the levels stream. Runs that never sing or never reach a bard's 6 or 25 hash as before |
| Saves | **no new saved state**: skills, weapon ranks, talent ranks, statuses, and resources are saved already; `ObjectDef::instrument` is content. Old saves load: a bard below 6 gets Travelled hands when 6 comes; one at 6 or above gets it on load (`class::grant` on `from_state`: "a grown table reaches saves") and keeps Blessing; one holding `jack_of_all_trades` at rank 1 gets rank 2 at 50 |
| Replays | a replay holding `use:inspire` plays (the id is kept); a bard replay past level 6 or 25 hashes differently from that level's grant (skills and ranks are hashed), as any class-table change does. The built-in scenarios' characters are a thief and rangers; no golden entry runs a bard |
| Content fingerprint | the `song` and `weapon_ranks` and `take_up_skills` kinds are talent content (not fingerprinted); `Use::Song` and the two Music effects are skill ids (fingerprinted as their kind is) |

## 10. The wiki, the spec, and the docs

| Where | What |
|---|---|
| `wiki/src/classes.md` (the bard section) and the generated class table | the table's nine rows; a paragraph on the fantasy ("knows a little of everything"); Travelled hands and Jack of all trades stated exactly (rank 1 to every weapon; `1d4+1` to every skill but Sneak attack; +5) |
| `wiki/src/abilities.md` | a **Songs** section: how a song is paid, heard (80), and lengthened (Music's degrees table from `skills.toml`, the instrument), Inspire's and the Lullaby's rows, the Lullaby's odds by Willpower edge and Music (`check::ways`, as `surprise.md`'s table), Virtuoso |
| `wiki/src/skills.md` | Music's row gains its two ladders (generated) |
| `wiki/src/resources.md` | Inspiration spent by two songs |
| `wiki/src/status-effects.md` | Drowsy's causes gain the Lullaby; Inspired's degrees |
| `wiki/src/objects.md` or the kit page | the lute as an instrument |
| `spec/08i-character.md` | the bard's table bullet rewritten (the nine rows, the two ranks, the class-table `rank` field); Skills: Music's benefits and mark; Talents: `take_up_skills`, `weapon_ranks`, `song`, `virtuoso` widened |
| `spec/07b-data.md` | `classes/README.toml`'s `abilities` row gains `rank`; `talents/README.toml` the three kinds; `objects` `instrument` |
| `spec/08c-combat.md` or `08e-blood-and-status.md` | songs beside the class abilities' statuses |
| `docs/dev/progression.md` | the hook table rows (`song` → `world/song.rs`; `weapon_ranks`, `take_up_skills` → handed out, not read; `virtuoso` widened), "Class tables" gains ranks |
| `docs/dev/resources.md`, `hearing.md`, `skills.md`, `weapon-skills.md`, `status-effects.md` | one line each: the songs' spending, `Act::Sing` and `SONG_NOISE`, Music's effects and `Use::Song`, `raise_to`, Inspired's `max` |
| Tests | `world/song_tests.rs` (Inspire paid by resource then mana, its degrees by Music and the lute, the noise and who comes to look, Silenced and the overworld refused; the Lullaby's hearers, the check's two outcomes, friends untouched, Drowsy's degrees and cap, the messages, a replay, a save), `world/class_rogue_tests.rs` (Travelled hands' ranks and defaults, Blessing gone, Jack of all trades' two ranks and the skills taken up, Sneak attack skipped, rank 2 on a restore, the news), `content/class.rs` (the `rank` field and its checks), `content/skill.rs` (Music), `kit.rs` (the lute flagged), the abilities-menu screen snapshot with the two songs, the wiki test |

## 11. Implementation plan

Five issues; at most one adds an action and none adds saved state. The false-conflict trick (`docs/dev/merging.md`) for the hot files: `talents/README.toml`, `skills.toml`, `status.toml`, `02-bard.toml`, `ability.rs`, `class.rs`.

| Issue | What | Adds | Depends on |
|---|---|---|---|
| **B1 Songs** (§6.1, 6.2, 6.4, 6.5) | Music's `kind`, `mark`, and two benefits with `Effect::SongDegrees`/`SongCraft` and `Use::Song`; the `song` kind (Inspire migrated onto it, the `inspire` kind retired), `world/song.rs::sing` with `Act::Sing` and `SONG_NOISE`; `inspired`/`inspired_greatly` `max = 3`; `silenced` blocks `song`; `ObjectDef::instrument` and the lute; Virtuoso on every song; tests; the README rows | none (the hearers arm is data-ready but no talent names it yet) | — |
| **B2 Travelled hands** (§5, §8) | `weapon_ranks` hand-out, `WeaponSkills::raise_to`, the talent, Blessing out of the bard's file, the news; `class_rogue_tests` | none | — |
| **B3 Jack of all trades** (§7) | `take_up_skills` hand-out from the levels stream; the two-rank talent; `AbilitySpec::rank` with its checks and `class::grant`'s rank step; the bard's file naming rank 2 at 50; tests | none | — |
| **B4 Lullaby** (§6.3) | the talent at 40; `ClassActive` `lullaby` (`use:lullaby`, aim nothing); the hearers arm of `sing` (the checks, Drowsy, the messages); the abilities-menu snapshot; action text in CLAUDE.md | **Action** | B1 |
| **B5 Wiki, spec, docs** (§10) | the Songs section and its generated odds, the class table, Music's ladders, the spec bullets, the docs' lines, a cross-check of the spec against what landed | none | B1–B4 |

**Why this order**: B1 first because B4 reads it; B2 and B3 touch only the class file, `class.rs`, and their hand-outs, so they run beside B1 (their shared lines are `02-bard.toml`'s table and `talents/README.toml`: the trick). B4 is the one action and waits for B1. Each issue answers the feature template's "Talents and class abilities": **none new**: songs extend `status`, `resources`, `mana`, and `hearing` (Patient blade, Clear mind, Keen ears touch them); the grants extend `skills` and `weapon_skills` (Apprentice, Weapon focus); no new `[[system]]`, no pickable minted (the bar is one and a count is never a reason).

## 12. Rejected ideas, with the reason

| Idea | Why not |
|---|---|
| **A cap on the bard's ranks or skills** ("master of none": no weapon past rank 5, no skill past 100) | the owner's "leaning towards mediocre", not "capped"; a flaw that bites every build the same is no choice (rule 5's spirit), and the player's own choices and the class's low start are enough. Unbalanced choices are fine (the owner) |
| **Six random weapon skills, six random skills** (ADOM's 12 and 25) | the wiki states rules exactly and the game stays vague about chance; "every" is a rule, "six at random" a lottery the wiki could only describe as odds; and ADOM's own wiki notes the bard's gains "can be different in different games" |
| **Wide learning** (+1 skill roll a level, the old 6) | cut by #780 as Quick learner's clone (`skill_rolls`); the bard's four rolls are the identity already, and a table may hand out Quick learner itself if the owner wants a fifth (it needs Intelligence 14, which a hand-out ignores) |
| **Honeyed words, Legend** (the old 40 and 50) | rules 1 and 9 (#798): gold, favour, and XP are not capstones |
| **+1 (or +6) to every stat** at 50 (ADOM's 50) | Unlikely hero's effects exactly (the commoner's 60): `talent_alike` would refuse the load |
| **Doubled regeneration** at 32 and 40 (ADOM's) | Saint (`healing_percent 100`) and Arcane mastery (`mana_regen_percent 100`) are those; a bard is neither, and Fast healer is its rest |
| **A dirge or a frightening tune** (Pathfinder 8 and 14) | Hound's bay (a Barghest's birth sign pick) is the one fear source: a hostile hearer of a lower level flees; a bard's dirge would be it under another kind. One song against foes is enough, and the Lullaby is the bard's (fascinate is the 3.5 bard's first trick) |
| **Suggestion, mass suggestion, a pacifying song** (a hostile made peaceful) | a faction change at talk is a design of its own (what breaks it, what the AI does with a half-friend, what it is worth); deferred to #196 if the owner wants it |
| **A lullaby that puts creatures Asleep** | Asleep is the den's state (no turn, unaware): a room put to sleep is a thief's dream and a bard's free dungeon; Drowsy slows and the fight still has to be won |
| **The Lullaby as the Sleep spell, granted** (`spell = "sleep"`, no new code) | honest and cheap, but it leaves Music unread, the lute unused, and the bard's one craft a wizard's spell; a song over hearers is the bard's own and costs one `song` arm |
| **A level gate for the Lullaby** (lower level than the bard, Hound's bay's rule) | a bard should be able to fail; the check is what Warlock uses, and the wiki states its odds |
| **Songs on a cooldown** | policy step 4: a cast that costs mana or a pool has no cooldown status besides |
| **Inspire silent** (as today) | a song is sung aloud; the noise is the honest price of a buff that costs 3 mana and no cooldown (Q7) |
| **Versatile performance** (Music standing in for Bluff, Negotiation, Intimidate) | a bookkeeping rule the game would have to explain, and Bluff and Intimidate have no action yet; when they do, the bard trains them already |
| **Magical secrets** (spells from any class) | the bard studies any scroll already (casting 75, Decipher script, the affinities); no slot is spent on what the rules give |
| **Bonus picks from any tree** (ToME's Adventurer) | the commoner's gift; the bard's Scholar and Delver say what kind of jack it is (Q13) |
| **A song for gold** (Elona's Performer) | economy (rules 1 and 9); the tavern is not in the game |
| **Countersong** (hexes on the bard end sooner) | hexes on the player come from a handful of caster types: niche by rule 6's spirit; `status_eases` exists if a later wave wants it on a pickable |
| **A new `songs` system tag and a pickable to go with it** | songs are class actives on statuses, resources, mana, and hearing: a sub-feature of those, not a system (policy: systems are broad); a pickable would be minted to fill a count |
| **Two talent ids for the two halves of Jack of all trades** | rule 4: a rank; the class table learns to name one (one field) |

## 13. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| Q1 | **Blessing goes** from the bard's 6 (Travelled hands takes the slot), or stays beside it (two entries at 6)? | **goes**: nine entries, one facet each; it is off the bard's schools and doubles Inspired's +1 to hit; a bard studies the scroll if they want it |
| Q2 | **Jack of all trades rank 1 takes up every skill** the bard lacks (not six at random), at `1d4+1`? | **yes**, every skill; `1d4+1` keeps them shallow (2 to 5; the thief's Open lock starts at 12 to 22) |
| Q3 | **Sneak attack skipped** by rank 1 (`except = ["sneak_attack"]`)? | **yes**: the surprise design's decision 7 ("only thief gets sneak attack guaranteed"); the list is data if the owner relents |
| Q4 | **The +5 moves from 25 to 50** as rank 2 (so level 25 is the skills taken up alone)? Or rank 1 keeps +5 too and rank 2 adds 5 more (+10 at 50)? | **moves**: taking up twenty skills is enough for 25, and +5 to every skill is the right size for 50; +10 would make the bard the best at everything middling |
| Q5 | **The Lullaby is a Willpower check** per hearer against 13 plus Music's craft (+4 at 100, +8 at 200), not a level gate? | **a check**: the bard can fail, the strong-willed resist, and the wiki prints the odds |
| Q6 | **The Lullaby's cost**: 3 Inspiration else 6 mana (Inspire's 3 and 3)? | **as proposed**: the pool pays for one song or the other, and 6 mana is a level-6 spell's price near enough |
| Q7 | **Songs are heard** (a voice at 80, Inspire too), so creatures come to look? | **yes**: a bard is loud; it is the one cost of a cooldown-free buff and the Lullaby's reach in one number |
| Q8 | **An instrument carried adds a degree** (the lute, `instrument = true`)? | **yes**: one hook, and the lute's comment is paid |
| Q9 | **Class tables name a rank** (`rank = 2`) rather than two ids? | **yes**: rule 4's shape; one field, two lines of code |
| Q10 | **Travelled hands covers melee, launchers, thrown, and unarmed** (not styles, schools, shields)? | **yes**: weapons handled; a style is a way of fighting, a school a study |
| Q11 | **Music becomes `kind = "rules"`** (its "not usable yet" row goes; the songs are its use)? | **yes** |
| Q12 | **Virtuoso widens to every song** (free, a degree more; Inspire greater as today)? | **yes**: the capstone of the one craft |
| Q13 | **Bonus trees stay Scholar and Delver** (not any tree)? | **stay**: "any" is the commoner's gift |
| Q14 | **Degrees are time, not strength**: Inspired's +1/+1 does not grow with degrees (only its length), the Lullaby's Drowsy likewise? | **yes**: the statuses' `combat` and `cost` are fixed as the witch's hexes are; Virtuoso's `inspired_greatly` is where the song gets stronger |
| Q15 | **The Lullaby sung to an empty room** spends its cost and makes its noise (no refusal)? | **yes**: a song is a song; refusing it would need the game to say nobody hostile hears, which is a leak |

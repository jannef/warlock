# Warlock — The adventurers' guild: rolled quests in a chain, tribes, wanted criminals, unique items

Design document for the owner's review (2026-10-26). Written against `origin/main` at 1c0ce120, the approved campaign design (`docs/design/campaign.md`), and the code it produced (#327 hidden locations, #336 Cēaptūn, #340 its quests). Nothing here is implemented; §13 cuts it into issues. §14 records the owner's answers to the first draft's open questions (2026-10-26); §15 lists the follow-ups. Names and dialogue follow the world's language policy (§14, decision 13; the pass over existing text is #378).

## Contents

1. Summary
2. Research, tied to the decisions
3. What exists, and what this builds on
4. The guild in Cēaptūn
5. Rolled quests: the data model
6. Tribes
7. The chain, link by link
8. The wanted: four criminals and their lairs
9. Unique items
10. What the player sees
11. Talents and class abilities
12. Spec, wiki, and docs
13. Implementation plan
14. Decisions (owner, 2026-10-26)
15. Follow-ups

## 1. Summary

**What the owner asked for**: an adventurers' guild in Cēaptūn that hands out **semi-random quests in a chain**, designed as data so later links are added without code: (1) kill a random creature that can spawn in the first three dungeons; (2) kill 100 of a random **tribe**; (3) bring a random scroll; (4) kill a random **wanted criminal**, whose quest **reveals a new dungeon** with the criminal at its bottom, each criminal dropping a **unique item** that fits them.

| Piece | Design |
|---|---|
| Where | the **Gildhalle** (Middle English "guildhall"), a small timber hall in Cēaptūn's south-east corner beside Factor Wulfrun's office; the guild is **the Weyfarers' Gilde** (Middle English: a new organisation). §4 |
| Who | **Wat One-Eye**, the guildmaster, a commoner who speaks Cockney: one npc on a new campaign type `guildmaster`; the chain's four quests are his topics, one at a time, as Leofric's are. §4 |
| Rolled quests | an ordinary `[[quest]]` whose target is a **`pick`** (a pool drawn by a rule from data that already exists: branch populations, tribes, scrolls) or a list of authored **`[[quest.variant]]`**s (the four criminals). The result, a **`Roll`** (target slug, variant id, count), is stored in the quest's `QuestRecord` when the quest is **given**; texts use placeholders (`{target}`, `{count}`, `{members}`, `{where}`). No generated `QuestDef`s: the template is the content, the roll is the save. §5 |
| Giving a quest | **every quest** (not only the guild's): asking the giver about it gives it. There is no accepting and no declining; the quest goes on the log at once with its first breadcrumb, "Wat wants you to kill a goblin chief." The `accept-ID` and `decline-ID` topics go. §5.3 |
| Randomness | a keyed stream per roll, **`quest/<quest id>`**, used once when the quest is given: nothing to save but the result, no draw on `gameplay`. **No rerolls**: the roll stands until the quest is done. §5.3 |
| Tribes | new shared data `data/tribes/<id>.toml`: a name and its members, by faction with additions and exceptions (`factions = ["kobolds"]`). Seven tribes from today's creatures, the ogre and troll out of the goblins, vermin in. A tribe kill is any kill of the player's of a member type, anywhere, after the quest was given; the quest's texts list the members. §6 |
| Named monsters | **every quest** whose kill target is a unique creature is met by its **death**, at any hand, whenever it happened: no softlocks. §8.4 |
| The chain | four quests linked by the existing `requires` and nothing else (no level, alignment, or other quest gate). Rank titles are a data file: seven ranks named by allomantic metals of the game's materials, **Tin, Pewter, Iron, Steel, Duralumin, Electrum, Gold** (a veiled F-to-S of isekai adventurers' guilds); the chain gives Tin on joining and two promotions, to Pewter and Iron. The chain is continued later and is likely a requirement of an ultimate ending; nothing is repeatable. §7 |
| The wanted | **Mother Sallow, the Fennhām Poisoner** (Wyrtholm, Micelfenn), **Black Wystan, the Gentleman of the Road** (Wulfeshol, Lindwudu), **Brother Osrick, the Resurrection Man** (Bānfeld, Hlāwdūn), **Sergeant Herrick, the Deserter** (Brocenburg, Holegnwudu): level-12 uniques of a new `outlaws` faction, each at the bottom of a hidden four-level branch (danger 6 to 9) on a reserved site; the rolled one's lair is revealed as the quest is given, the other three stay hidden for that run. §8 |
| Unique items | **Widowmaker** (stiletto: poison that builds), **High Toby** (long bow: shots that frighten), **the Unhalwed Bedes** (bone amulet: a caster who needs no light), **the Turncoat** (banded mail: a wall that knows no fear). A new `uniques` table of fixed rares (base object, name, affixes at chosen tiers, a **power**); each plays differently. §9 |
| Talents | a new system `bounties` with **Quarry sense** (delver) and **Sworn foe** (weaponcraft, a tribe as its choice); quest rewards already have five talents under `talk`. No class ability (`min_class = 0`). §11 |
| Slicing | six issues: tribes, rolled quests and the giving rules (+ the system and its talents), unique items, outlaws and the four criminals, the guild and links 1–3, the four lairs and link 4. §13 |

## 2. Research, tied to the decisions

| Finding | Source | Used in |
|---|---|---|
| Adventurers' guilds and job boards hand out procedurally parameterised jobs ("kill N of X", "bring Y"); the best remembered are those whose chain ends in a named target | Elona (the Adventurers' Guild, town boards), Daggerfall (the Fighters Guild's rank-gated jobs), Monster Hunter (rank-gated hunts) | §5 template + roll; §7 the chain |
| Isekai adventurers' guilds rank their members F, E, D, C, B, A, S, or by metals and stones (Goblin Slayer's Porcelain to Platinum); the rank is a title the town calls you by | genre convention | §7.6 seven ranks named by metals, veiled |
| Daggerfall's guild ranks matter mostly as the title the player gets | Daggerfall, UESP "Fighters Guild" | §7.6 ranks as titles |
| ADOM's quests are fixed per run but some parameters are rolled per character (the wanted bandit's location, the item a quest-giver wants) | ADOM Guidebook, "Quests" | §5.3 the roll stored per character, never rerolled |
| ADOM's black torc and golden gladius: one fixed item, relatively easy to get and very strong, which shapes the run that finds it | ADOM | §9 strength kept, four very different items |
| Angband's and DCSS's uniques carry or guard a fixed artifact ("randart" vs "unrandart"): a fixed, named item with a theme is what players remember | Angband `artifact.txt`, DCSS unrandarts | §9 fixed uniques, one per criminal |
| Elona's hunt quests that require the kill *by the player* frustrate when a pet or a trap gets it | Elona community wiki, "Hunting quest" | §8.4 a named monster's death counts, whoever killed it |

## 3. What exists, and what this builds on

| Piece | Today (`docs/dev/quests.md`, `npcs.md`, `overworld.md`, `magic-items.md`, `scrolls.md`) | What the guild needs |
|---|---|---|
| Quest data | `QuestDef` (`content/quest.rs`): `Goal::{Kill{creature, count, level}, Bring{object, count, to}, Reach{level}, Deliver{object, to}}`, `Reward`, `QuestTexts` (offer, accept, decline, reminder, done), `requires` (one or a list), `reveals = { location, when }`, `repeatable`, `where` | a target that is **rolled** (§5), a kill target that is a **tribe** (§6); `decline` retired (§5.3) |
| Giving | the quest topic offers; `accept-ID` or `decline-ID` then answers; `withdraw` forgets a declined offer | the quest topic **gives**: one topic per quest (§5.3) |
| Quest state | `Quests` in `WorldState`: `records` by id (`stage`, `completed`, `counted`, `reached`), the kill tally by level and creature type, `prey` | `QuestRecord::roll` (§5.4); `Quests::slain`, the unique types that died (§8.4) |
| Kill counting | `fight.rs::dies` → `tally_kill` (the player's own kills, bleeding included; never traps or others); "those before you took the quest up too" | rolled kills count from the giving (the `counted` baseline that repeats already use); a unique's death counts at any hand (§8.4) |
| Reveals | `World::reveal_location(id)`; `reveal_at` at the stage the quest's `reveals.when` names; the dev action `reveal:ID` | a reveal per **variant** (§5.2) |
| Chains | `requires`: Leofric, Hild, and Wulfrun each offer one quest at a time | unchanged: the guild's chain is `requires` (§7) |
| Streams | `Rng::stream(seed, label)`; the lazily kept `levels_rng`; keyed per-level streams (`spawn/<level>`, `gear/<key>`) | one keyed stream per roll, never kept (§5.3) |
| Placed magic items | a level's `[[item]]` places a rare with a `name` and `affixes = ["sharp:2"]` | the same item from a table, placeable anywhere objects are (§9) |
| Creatures | `unique = true` (a population draws it at most once; `progress.uniques` holds those met), pinned branch vaults (`[[dungeon.levels]] vaults`, the drake's hoard) | four uniques placed by four pinned vaults (§8) |

## 4. The guild in Cēaptūn

### 4.1 The Gildhalle

A timber hall, 9×7 with its walls, in the free grass of the town's south-east corner (about `[77, 22, 85, 28]` in `levels/ceaptun.toml`, east of Wulfrun's office and west of the mill race; the two trees there go). Its door opens north on the lane, which is extended a cell east to meet it. Inside: plank floor, a lamp, a table, Wat at a post by the far wall, and one **named place** (#332's `name` and `note` on a legend cell, no new terrain): *"the bounty board"*, note *"Bills nailed up three deep, most of them years old."* The town comment block and `docs/dev/campaign.md` "Cēaptūn" name it.

| Field | Value | Register (decision 13) |
|---|---|---|
| Building | **the Gildhalle** (Middle English *gild* + *halle*, "guildhall") | Middle English: the guild is a new organisation and its hall is new; the Gæsthūs beside it is older and keeps its Old English |
| Guild | **the Weyfarers' Gilde** (Middle English *weyfarere*, "wayfarer, traveller") | Middle English, as an organisation; read aloud it is "the Wayfarers' Guild", so the player understands it at once |
| NPC | **Wat One-Eye** ("Old Wat"), guildmaster: a one-eyed old delver who has stopped going down | a commoner: Cockney in speech and in name (*Wat*, the short Walter of the commons, and a byname instead of a surname) |
| Ranks | **Tin** to **Gold** (§7.6) | Victorian: the ranks are a concept of the game's own day |

### 4.2 Wat One-Eye

| Data | Content |
|---|---|
| Creature type | `campaigns/ascmearc/data/creatures/guildmaster.toml`: peaceful, `budge = "off_post"`, `@`, the townsfolk's stats with Strength and Toughness of a retired fighter; never fights (as the reeve) |
| Npc | legend `"M" = { terrain = "plank_floor", creature = "guildmaster", npc = "wat" }` |
| Dialogue | `campaigns/ascmearc/data/dialogue/ceaptun.toml` gains `wat`: `name = "Wat One-Eye"`, `greet`, five rotating `lines` (the old days, the eye, the board, the town's ingratitude, the barrow), `greet_evil` (he still gives the chain: none of its quests is good-aligned, §7) |
| Topics | `guild` ("The Gilde": who they are, how ranks work) and `board` ("The board": a reply that points at the chain's next job). With the one quest topic, three of nine |

Sample lines, in Wat's Cockney (one line each, ≤280 characters, as every text):

- greet: *"'Nother one off the pass, is it? Sit down if you like. Standin' costs the same."*
- greet_evil: *"I know your sort. Work's work, mind, an' the board don't care whose 'ands are dirty."*
- `guild`: *"We does the work the reeve can't pay for an' the sisters won't touch. Do a job, come back, get paid. Everyone starts Tin. Do enough an' you're Pewter, then Iron, an' folk stop callin' you 'oi, you'."*
- a rotating line: *"Lost this eye in the barrow, forty year back. Don't go lookin' for it, it's 'ad a long day."*

## 5. Rolled quests: the data model

### 5.1 Why templates and a stored roll, not generated quests

| | A `QuestDef` generated at run time, saved whole | **A template in data + a roll in the record** (chosen) |
|---|---|---|
| Content | texts built in code from fragments | texts in data with placeholders; the owner edits TOML as for Leofric's |
| Saves | every generated def in `WorldState` (texts, goal, reward): a large new save type | `QuestRecord::roll`: three short fields (§5.4) |
| Topics, log, hand-in, `requires`, `reveals` | a second path beside `content.quest(q)` everywhere | unchanged paths; `def` plus `record.roll` resolve the goal (`QuestDef::goal_for(roll)`) |
| Replays | `ask:E:quest-ID` with a generated id (unstable across content) | `ask:E:quest-guild_quarry`: the id is the template's |
| Later links | code | a quest file |

### 5.2 The data

Two ways to roll a target, both on an ordinary `[[quest]]` (the field reference in `data/quests.toml` grows by these):

```toml
[[quest]]
id = "guild_quarry"
name = "A bounty"                 # topic names stay fixed: they show before the roll
giver = "wat"
type = "kill"
pick = { creatures = ["cealdhlaw", "fennmynster", "fyrdelf"] }
count = 1
reward = { gold = 40, xp = 60 }
offer = "There's a price on {a_target}, see. Bring the beast down, wherever you find it, an' the Gilde pays. Counts from today, mind: what you done before's your own business. {Where}"
reminder = "{A_target}. {Where}"
done = "That's the one. 'Ere's the Gilde's coin."
where = "{Targets} are known in {branch}."
```

```toml
[[quest]]
id = "guild_wanted"
name = "Wanted"
giver = "wat"
requires = "guild_scroll"
type = "kill"
reward = { gold = 500, xp = 800 }
alignment = 10
offer = "{offer}"                 # a variant's own texts replace these
reminder = "{reminder}"
done = "{done}"

[[quest.variant]]
id = "wystan"
target = "wystan"
reveals = { location = "wulfeshol", when = "given" }
where = "His den, Wulfeshol, lies in Deorcholt, deep in Lindwudu."
offer = "Black Wystan. Worked the Fennhām road six year, all 'stand an' deliver' an' lovely manners, till 'e done a carter for 'is boots. Reeve's lads won't go into the Deorcholt after 'im. Will you?"
# reminder, done ...
```

| Field | Meaning | Checked at load |
|---|---|---|
| `pick.creatures = [branches]` | any creature type from those branches' `population` that is not `unique`, uniformly, of the branches the player can enter when the quest is given (§5.5); no level band (decision 1) | branches exist and are the quest's campaign's |
| `pick.tribes = [ids]` or `"all"` | a tribe (§6) with a member in a branch the player can enter | tribes exist |
| `pick.objects = { class = "scrolls", loot = true }` | an object of that item class; `loot = true` only those loot can draw (`frequency > 0`) | class exists; the pool is not empty |
| `[[quest.variant]]` (`id`, `target`, optional `weight`, `reveals`, `where`, and any of the texts) | an authored option; what it gives replaces the quest's own | a variant's target fits the `type`; its `reveals` names a hidden location of the campaign |
| `count` | as today; on a tribe pick the tribe's own `count` wins if it has one (§6.2) | |

A quest has `target`, `pick`, or variants, exactly one. **Placeholders**, filled from the roll in every text, the `name` excepted: `{target}` / `{targets}` (the type's, tribe's, or object's name and plural), `{a_target}` (with its article; the unique names of §8 take none), `{count}`, `{members}` (a tribe's member types, plurals joined: "kobolds, kobold archers, and kobold shamans"; empty for other picks), `{branch}` (the first branch, in pick order, whose population holds it: the place the guild "knows" it from), `{where}` (the rolled `where`), and capitalised forms (`{Targets}`, `{A_target}`, `{Where}`). A text over 280 characters after filling, with the longest name and member list the pool can give, is a content error naming the quest.

### 5.3 Giving a quest, and when the roll is made

The owner's rule (decision 4) for **every quest**, the guild's and the existing ones alike: this is a roguelike, not an MMO, so there is no point in declining.

| Question | Design |
|---|---|
| Asking about a quest | the quest topic (`quest-ID`) **gives** the quest: the giver says the offer (then the quest's `accept` line, if it has one, as a second line of the same reply), and the record is created **active**. `World::offers` still decides whether the topic shows (`requires`, `requires_alignment`; since #692 a giver's scorn withholds nothing) |
| The log | the quest goes on the log at once with its **first breadcrumb**: "<Giver> wants you to <objective>." ("Wat wants you to kill a goblin chief.", "Sister Hild wants you to find the barrow's heart."), then progress lines as today, then "Return to <giver>." |
| Topics retired | `accept-ID` and `decline-ID`, and with them the `decline` text and `withdraw`. `RevealWhen::Accepted` becomes `given` (the same moment). Each existing quest's `accept` line stays, said after its offer; its `decline` line is dropped (Hild's three, Wulfrun's, Leofric's). A replay step `ask:E:accept-ID` or `ask:E:decline-ID` is refused: replays and tests that use them are rewritten in the PR |
| When the roll is made | as the quest is given, so the offer can name the target; the record holds its `roll` from then on |
| Stream | `Rng::stream(seed, "quest/<id>")`: keyed, used once, never kept. Nothing about the stream is saved; the roll is |
| Rerolls | **none** (decision 4): the roll is fixed until the quest is done. A kill goal's `counted` baseline is the tally at the giving (§5.6); a variant's `reveals` fires |
| Inputs | the revealed locations and the content: all in the state, so a replay rolls the same |
| Order | candidates in content order (creature, object, and tribe indices; variants in file order), uniform or by `weight`: never `HashMap` order |
| Dev | `quest_roll:ID:TARGET` (a dev action, no time; refused unless the quest is active, has no progress yet, and TARGET is in its pool or a variant id): it replaces the roll and resets the baseline, and a variant's reveal fires (an earlier variant's reveal stays: dev only). Replays, tests, and captures pick a criminal without hunting a seed |

### 5.4 Saves

```rust
pub struct QuestRecord {               // world/quest.rs, existing fields unchanged
    ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roll: Option<Roll>,
}
pub struct Roll {
    pub target: String,                // a creature, tribe, or object slug
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    pub count: u32,
}
// Quests gains, left out while empty (§8.4):
//     pub slain: BTreeSet<String>,    // unique creature types that have died, at any hand
```

Slugs, not indices (as affixes are saved), so adding creatures keeps saves. `Quests::unknown_to` also checks a roll's target, variant, and tribe. Runs without rolled quests or dead uniques hash as before (the fields are left out). A save with an offered-but-not-accepted record (today's stage `offered`) loads as active. `FORMAT_VERSION` is the integrator's, once per release.

### 5.5 What the player can reach

A branch "the player can enter" is one whose overworld entrance is not hidden, or is in `WorldState::revealed` (`Content::hidden_location` and the state, nothing new). Link 1 draws uniformly from the non-unique population of Cealdhlāw, Fennmynster, and Fyrdelf (decision 1), less the branches still hidden when Wat gives it, so the bounty never names a fen leech before Sister Hild reveals Fennmynster, and a roll that is never rerolled can never point at a place the player cannot go. No level band: a level-1 character may draw a cinder elemental (unbalanced choices are fine; the player may come back for it later). The pool is never empty while one branch is open.

### 5.6 Counting a rolled kill

From the giving only (decision 3): giving a rolled kill quest sets `counted` to the tally's count at that moment (today only repeats do). The rest is today's `progress`: tally minus `counted`, capped at the count. An authored kill quest of a non-unique type keeps today's rule (earlier kills count). A unique target is §8.4's rule, whatever the quest.

## 6. Tribes

### 6.1 Is there something to reuse?

Creatures have one grouping today: `faction` (`data/factions.toml`), which drives stance and the AI. It is close to "tribe" but not the same: the `goblins` faction holds the ogre and the troll; `vermin` holds the basilisk; a keeper's faction changes when outraged (`faction_override`). Creatures have no tags. So a tribe is **its own small table**, defined *from* factions so it stays in step with them, with additions and exceptions where the faction is wrong for the word.

### 6.2 The data (`assets/data/tribes/<id>.toml`, shared engine data; one file each)

```toml
[[tribe]]
id = "goblins"
name = "goblin"          # "a goblin" is how the guild says one
plural = "goblins"       # "Kill 100 goblins."
factions = ["goblins"]   # every type of these factions...
except = ["ogre", "troll"]   # ...but these
creatures = []           # ...and these besides
# count = 100            # the guild's count for it (optional; the quest's otherwise)
```

A campaign may add tribes (`campaigns/<id>/data/tribes/`) and its types join shared tribes through `factions`. Membership is by **type**: an actor's `faction_override` does not move it out of or into a tribe. A tribe with no member is a content error.

### 6.3 The seven tribes (decision 5: as listed)

| Tribe | Members | Where they live (reachable from) |
|---|---|---|
| kobolds | kobold, kobold archer, kobold shaman | Cealdhlāw, Fyrdelf |
| goblins | goblin, goblin archer, goblin chief (the ogre and the troll excepted) | Cealdhlāw |
| the undead | skeleton, zombie, barrow ghoul, drowned, ghost, ash wraith, barrow wight | Cealdhlāw, Fennmynster, Fyrdelf |
| vermin | rat, grave worm, barrow spider, giant centipede (the basilisk excepted) | Cealdhlāw, Fyrdelf |
| fen things (`marsh`) | giant leech, giant frog, crocodile, will-o'-wisp, mud golem, marsh troll, bog hag | Fennmynster |
| duergar | duergar miner, duergar warrior, duergar smith (the trader excepted) | Fyrdelf |
| fire things | ember beetle, fire salamander, magma crawler, cinder elemental, fire drake | Fyrdelf |

The cult (two types) is left out: 100 cultists is a grind of one floor type. The outlaws (§8) are not a tribe (decision 6).

**The texts make the membership clear** (decision 5): link 2's offer names every member through `{members}` ("Goblins, any sort: goblins, goblin archers, and goblin chiefs. Not the ogres, mind, nor the trolls."; a tribe file may give an `except_note` for the second sentence), and the quest log's objective repeats the list ("Kill 100 goblins: goblins, goblin archers, and goblin chiefs."). The wiki has the exact table.

### 6.4 Counting rules

| Case | Counts? | Why |
|---|---|---|
| a member killed by the player, anywhere (any branch, the overworld, a local area, a hand-made level) | yes | the tally is per level; a tribe sums every level |
| bled to death of the player's wound, shot, thrown at, burnt by the player's scroll or spell | yes | all are `tally_kill` today |
| killed by a trap, another creature, or the player's absence | no | not the player's kill; spawns refill floors, so the count stays reachable |
| killed before the quest was given | no | §5.6, decision 3 |
| a member that is unique (the barrow wight, the bog hag, the drake) | yes, one each, when the player kills it | a type is a type; a count of 100 never hangs on one unique |
| an outraged shopkeeper, a peaceful villager | no | not members |

## 7. The chain, link by link

The chain is four `[[quest]]`s in `campaigns/ascmearc/data/quests/wat.toml`, each `requires` the one before and **nothing else** (decision 8): no level, alignment, or other quest gates any of the first four links, so Wat shows one topic at a time (Leofric's pattern) from the player's first visit. Nothing is repeatable (decision 10). The chain will be continued later by more quest files (the first with `requires = "guild_wanted"`); reaching its end will probably be a requirement of an ultimate ending (§15).

| # | Id, topic | Goal | Pool | Reward | Alignment | Rank once done |
|---|---|---|---|---|---|---|
| 1 | `guild_quarry`, "A bounty" | kill 1 of a rolled type (decision 2) | any non-unique of Cealdhlāw, Fennmynster, Fyrdelf, those open when given (§5.5) | 40 gold, 60 XP | 0 | (Tin from the giving) |
| 2 | `guild_hundred`, "The hundred" | kill 100 of a rolled tribe | `tribes = "all"`, reachable | 250 gold, 500 XP, 2 red potions | 5 | **Pewter** |
| 3 | `guild_scroll`, "A scroll for the Gilde" | bring 1 rolled scroll to Wat | `objects = { class = "scrolls", loot = true }`: fireball, lightning, frost, identify (the scroll of light, sold in town and never loot, is out) | 200 gold, 150 XP | 0 | — |
| 4 | `guild_wanted`, "Wanted" | kill the rolled criminal (a named monster: any death counts, §8.4) | four variants, §8 | 500 gold, 800 XP (the unique item is the criminal's, §9) | 10 | **Iron** |

The XP sits between Leofric's (30 to 250) and Wulfrun's drake (900); the kills themselves pay their own XP. Being given link 1 makes the player a **Tin** of the Gilde; links 2 and 4 promote (decision 9: two promotions in the chain).

### 7.1 Link 1: a bounty

The target comes from the normal population and spawner: the guild places nothing. It **spawns out of the player's sight** like any creature (the spawner's rules) or is already on a floor they have not reached; the log names the branch it is known in (`{branch}`), never a depth (the danger is hidden). Quarry sense (§11) tells a player on its level. A target another creature kills does not count; the next one will. Count 1.

### 7.2 Link 2: the hundred

100 is long by design: about four floors' populations. The log counts ("37 / 100 killed."), as kill quests already do; which types belong is told in the offer and the log (§6.3) and exactly in the wiki.

### 7.3 Link 3: a scroll

**Identified or not**: scrolls have no unidentified state in this game (they are never magic and are named as they are, `docs/dev/magic-items.md`), so the question does not arise today. Should scroll appearances come, the hand-in matches the object type whatever the player knows ("the Gilde knows its scrolls"). **Consumed**: yes, as every bring (it leaves the pack). A scroll picked up in a shop and handed in unpaid stays on the keeper's bill (today's rule for anything used up).

### 7.4 Link 4: wanted

§8. The variant's `reveals` marks the lair on the map as the quest is given ("Wat marks Wulfeshol on your map.").

### 7.5 Failure and stalls

There is nothing to fail: no timers, no abandoning, no declining, no cap on active quests (the owner's decisions). With no rerolls, every roll must stay finishable; the ways a link could stall, and why they don't:

| Risk | Answer |
|---|---|
| a rolled type or tribe never appears | the pool takes only branches the player can enter when the quest is given, and floors refill |
| a rolled type is far too strong for the player | it waits: no timer; the player comes back stronger (decision 1 accepts this) |
| the criminal dies by another hand or a trap | it counts (§8.4) |
| the scroll is read or sold | another can be found or bought; nothing is lost |
| the player dies | permadeath, as everywhere |

### 7.6 Ranks

The ranks are seven, after the F, E, D, C, B, A, S of isekai adventurers' guilds, veiled: the game never says the letters, only the metals. The metals are the **allomantic** metals among the game's materials (`assets/data/materials/`). Of the sixteen there (iron, steel, tin, pewter, zinc, brass, copper, bronze, cadmium, bendalloy, gold, electrum, chromium, nicrosil, aluminium, duralumin; atium, malatium, lerasium, and harmonium are absent), seven are picked and **ordered by the game's own `value`**, so each rank is a dearer metal than the last and the two lowest pairs are a metal and its alloy:

| Rank | Material (value, % of steel) | The veiled letter | Reached by |
|---|---|---|---|
| **Tin** | tin (40) | F | being given link 1 (joining) |
| **Pewter** | pewter (50) | E | link 2 done |
| **Iron** | iron (60) | D | link 4 done |
| **Steel** | steel (100) | C | a later link |
| **Duralumin** | duralumin (1000) | B | a later link |
| **Electrum** | electrum (1200) | A | a later link |
| **Gold** | gold (2000) | S | the chain's end |

`campaigns/ascmearc/data/guilds/weyfarers.toml` (the rank titles are data, decision 9):

```toml
[[guild]]
id = "weyfarers"
name = "the Weyfarers' Gilde"
master = "wat"
ranks = [
  { title = "Tin",       material = "tin",       given = "guild_quarry" },
  { title = "Pewter",    material = "pewter",    done = "guild_hundred" },
  { title = "Iron",      material = "iron",      done = "guild_wanted" },
  { title = "Steel",     material = "steel" },      # later links: a rank with no quest is not
  { title = "Duralumin", material = "duralumin" },  # reached yet (allowed only after the ranks
  { title = "Electrum",  material = "electrum" },   # that have one)
  { title = "Gold",      material = "gold" },
]
```

`material` must name a material (a load check); the character screen shows the title in its colour. The rank is **derived** from the quest records (the last line whose quest is at that stage), so it is not saved. It is a title: the character screen and the `stats` dump show it; nothing reads it as a rule. A later link adds `done = "..."` to a reserved line.

## 8. The wanted: four criminals and their lairs

### 8.1 Shape

Each criminal is a **campaign creature** (`unique = true`, in no population, placed only by a pinned vault on the bottom floor of their own branch), at **level 12** (the ogre's level, a step above the bog hag), in the new shared faction **`outlaws`** (hostile to the player only; no faction lists it as hostile, so the lair's other inhabitants never fight its master). Each branch is four floors at danger 6 to 9 (`danger = 6`, one a floor), hidden on a **reserved site** of the overworld (design §5.5) until the variant reveals it. All four exist in every run; three stay hidden. Their creatures already exist, plus two new campaign types: the **footpad** (level 6, short sword and jerkin, `short_blades = 2`) and the **footpad archer** (level 7, short bow, `bows = 2`); *footpad*, the Victorian word for a robber on foot, as the commons would say it.

**Names** (decision 13): the criminals are people of the game's own day, so their names are the evolved forms of the old ones (Edith from Ealdgyth, Wystan from Wulfstan, Herrick from Hereric) and their bynames are what the broadsheets call them, in Victorian English; the lairs are old sites and keep their Old English names.

### 8.2 The four

| | Mother Sallow, the Fennhām Poisoner | Black Wystan, the Gentleman of the Road | Brother Osrick, the Resurrection Man | Sergeant Herrick, the Deserter |
|---|---|---|---|---|
| Crime | herb-wife of Fennhām who sold death to heirs; three dead at one wedding feast | highwayman of the Fennhām road, famous for his manners; killed a carter for his boots | a brother of the Mynster, cast out for robbing graves and selling the dead to anatomists, then for raising them (Sister Hild knows the name; one line of hers) | a sergeant of the fort who killed his captain and ran; now hires out to goblins |
| Class and speech | a commoner (her name a byname; were she to speak, Cockney) | a commoner who apes the gentry | a brother of a posh order | a commoner |
| Id, glyph | `sallow`, `@` pale green | `wystan`, `@` russet | `osrick`, `@` bone white | `herrick`, `@` iron grey |
| Stats (St De To In Wi Pr) | 11 17 13 16 14 12 | 15 18 15 11 12 14 | 13 11 15 17 19 15 | 20 12 19 10 13 13 |
| Base hp, speed | 24, 1100 | 30, 1000 | 32, 1000 | 40, 1000 |
| Fights | melee, quick; wields Widowmaker, whose every hit poisons (§9) | `ranged`, range 8, sight 14; wields High Toby, 40 arrows; his hits may frighten | melee with a mace (`maces = 2`), wears the Bedes (darkvision through them); spells later (§15) | melee; a great axe (`axes = 3`, `great_weapon = 2`, 50 % magic), wears the Turncoat |
| Skills, weapon skills | stealth 60, heal 50; `daggers = 3` | stealth 40, spot 50; `bows = 3`, `marksman = 1`, `short_blades = 2` | concentration 50, spellcraft 50; `maces = 2` | intimidate 40; `axes = 3`, `great_weapon = 2` |
| Resists, other | poison 100; footsteps 10 (quiet); vision `low_light` | vision normal | cold 50; poison 50 | Frightened never takes him (his mail) |
| Lair (Old English meaning), region | **Wyrtholm** ("herb islet"), Micelfenn | **Wulfeshol** ("wolf's hole"), Deorcholt in Lindwudu | **Bānfeld** ("bone field"), a barrow field in Hlāwdūn | **Brocenburg** ("the broken fort"), the ruined tower in Holegnwudu |
| Generator, size | caves, 100×40 | caves, 100×40 | rooms, 80×21 | rooms, 80×21 |
| Population | giant leech, giant frog, crocodile, will-o'-wisp, mud golem, marsh troll, giant centipede, footpad | footpad, footpad archer, bat, giant centipede, barrow spider, goblin archer | skeleton, zombie, barrow ghoul, ghost, drowned, grave worm, barrow spider | goblin, goblin archer, goblin chief, footpad, footpad archer, ogre, bat |
| Traps, other | pit, alarm; temperature 9 | pit, arrow, alarm | arrow, spear, pit, alarm; `room_effects` shadowed, hushed; key `rusty_key` | arrow, spear, alarm; 2 locks a floor; key `iron_key` |
| Bottom vault (pinned, floor 4) | "the still-room": her bench, jars (red potions, angelica), her | "the robbers' hoard": a cave mouth of loot (`$`, one `loot = "rare"`), him | "the unhallowed chapel": a defiled altar, bones, him | "the captain's hall": a broken table, the captain's grave, him |

Ogres (12) appear in Brocenburg only out of depth; that is the point of hiring them. A caster's spells are only announced today (`docs/dev/creatures.md`), so Osrick fights with a mace until creature spellcasters exist (decision 11): his creature file (`campaigns/ascmearc/data/creatures/osrick.toml`) carries a note at its top, written by G4 as the issue specifies:

```toml
# TODO(creature spells): Brother Osrick is a necromancer who fights with a mace only because
# creatures cannot cast yet. Once creature spellcasters exist, give him spells (raising the dead,
# a curse, cold): docs/design/adventurers-guild.md §15.
```

**Variant texts** (Wat's Cockney; each ≤280 characters):

| Variant | Offer |
|---|---|
| `sallow` | *"Mother Sallow. Sold 'erbs in Fennhām twenty year, an' a little somethin' extra to anyone with a rich uncle. Three dead at one weddin'. She's gone to ground on Wyrtholm, out in the Micelfenn."* |
| `wystan` | *"Black Wystan. Worked the Fennhām road six year, all 'stand an' deliver' an' lovely manners, till 'e done a carter for 'is boots. Reeve's lads won't go into the Deorcholt after 'im. Will you?"* |
| `osrick` | *"Brother Osrick, what the Mynster threw out. Resurrection man: dug up the dead an' sold 'em to the sawbones, then started gettin' 'em up again. 'E's at Bānfeld, on the Hlāwdūn."* |
| `herrick` | *"Sergeant 'Errick. Done 'is captain at the fort an' legged it, an' now 'e sells 'is axe to goblins out at Brocenburg in the Holegnwudu. Fort wants 'im back. In a box'll do."* |

Sister Hild, of the better-off and a posh speaker, on Osrick (one rotating line): *"Brother Osrick? We do not speak that name in the Mynster. He sold the faithful to anatomists, and then he found worse uses for them."*

### 8.3 Where a criminal is placed, and when

The bottom floor is generated on its first visit (today's rule), so the criminal is always placed out of sight, in the vault, asleep like the drake: it wakes to what it sees or hears. Nothing is spawned for the quest elsewhere, and nothing moves on a floor the player is not on.

### 8.4 A named monster's death

The owner's rule (decision 7), for **every quest** whose kill target is a unique creature type (a named monster: the four criminals, Wulfrun's fire drake, any later one): **only the death counts**, whoever or whatever caused it and whenever it happened, so no bug, trap, or stray monster can softlock a quest. There is no field for it: a kill goal on a type with `unique = true` is met by its death.

- `creature_dies` (the one place every death passes, the player's kills and all others) adds a dying unique's type to `Quests::slain`.
- `progress` of a kill goal on a unique type is 1 when `slain` holds it, else 0. A unique that died before its quest was given still counts: the quest is met at once and the log says "Return to <giver>." (this overrides decision 3's baseline, which is for counts of types and tribes, §5.6).
- Since floors the player is not on are frozen and the criminal's own lair does not fight it, in practice the death is the player's, a trap the player led it into, or the player's scroll. This is the one place the log may know something the player did not see (a trap's kill behind a door); the alternative is a softlock.

## 9. Unique items

### 9.1 The data (`campaigns/ascmearc/data/uniques/<id>.toml`; a shared `data/uniques/` too)

```toml
[[unique]]
id = "widowmaker"
name = "Widowmaker"
object = "stiletto"
affixes = ["venomous:2", "of_celerity:3", "of_the_lynx:2"]   # the [[item]] syntax
parts = [{ of = "steel" }]     # optional, as [[item]]
level = 12
power = { inflict = "poisoned", chance = 100, severity = 1 }
lore = "Thin as a hatpin, and never quite clean."   # the item description's last line
```

| Question | Design |
|---|---|
| Rarity | **rare** with a set name and set properties (decision 12): exactly what a level's `[[item]]` with a `name` makes today, plus `Magic::unique: Option<String>` (the slug, saved like affixes) so its power and lore are found again. No new rarity tier or colour |
| Where it may be placed | anywhere objects are named: `unique:ID` in a gear line's `one_of`, a legend's `wield`/`wear`/`carry`/`objects`, a pile, a quest reward's `items` |
| Once a run | each unique is named by **one** placement in all content (a load check): with pinned vaults and unique criminals it is made at most once, with no save state |
| Identification | as any magic item: unidentified it is "a stiletto" with a rare's unknown look; identified, its name, affixes, power, and lore. Presentation reads only `Stack::known_magic`, as today |
| Loot | never drawn by `loot::roll` |
| Strength | kept (decision 14): high tiers plus a power, at item level 12, above what loot rolls there. Like ADOM's black torc or golden gladius, one item, relatively easy to get, and very strong; each of the four shapes a different way to play |

### 9.2 The four

Each plays differently (decision 14): a quick poisoner's blade, an archer's bow that drives foes off, a caster's amulet for the dark, and armour for standing in a doorway. The affixes serve the item's way to play and nothing else: no item has the generic "more damage, more to hit" set.

| Item (name's register) | Base | Affixes (tier) | Power | How it plays | New rule? |
|---|---|---|---|---|---|
| **Widowmaker**, Mother Sallow's (Victorian slang) | stiletto | **venomous** 2 (a new affix, data only: `elemental`, `damage_type = "poison"`, weapon and ammo), of celerity 3 (attack speed), of the lynx 2 | **every** hit poisons: one degree of Poisoned, stacking (chance 100, severity 1) | many quick, light strikes; step back and let the poison work. Weak against the poison-immune (the undead, golems), which is its price | **yes**: `Power::Inflict { status, chance, severity }`, read where a natural attack's `on_hit` is applied (`fight.rs`), for the weapon that hit; the target's poison resistance applies as it does to `on_hit` |
| **High Toby**, Black Wystan's (Victorian thieves' cant, *the high toby*: robbery on the highway) | long bow | hunter's 3 (range), steady 3 (draw), of precision 2 | a hit may **frighten**: 25 %, one degree of Frightened | shoot, and the foe breaks and runs; shoot it again as it comes back. Control at range, not damage | no new rule: the same `Power::Inflict` with another status |
| **The Unhalwed Bedes**, Brother Osrick's (Middle English *unhalwed bedes*, "unhallowed prayer beads": the Mynster's order is older than the guild, its rosary not as old as the Mynster) | bone amulet | mystic 3 (max mana), of clarity 3 (mana regeneration), of the owl 2 (Intelligence), of shadows 2 (Stealth) | its wearer sees with **darkvision** | a caster who puts the lantern out: unseen, unlit, spells from the dark | **yes**: `Power::Vision(profile)`, read where an actor's vision profile is chosen (the best of race and power; the dev F3 profile still wins) |
| **The Turncoat**, Sergeant Herrick's (Victorian: a deserter's coat, and his crime) | banded mail | reinforced 3, hardened 3, hale 3, of the ox 2 | **Frightened never takes** its wearer | stand in the doorway and do not move: heavy, hard to hurt, unshakable | **yes**: `Power::Ward(status)`, read in the one place a status is applied (`World::afflict`): refused, with a line ("Fear finds no hold on you.") |

Each is a different slot and a different way to fight: weapon, launcher, amulet, armour. The powers are a `power` table, not an affix: an affix is a value with tiers that loot rolls, and a power is a rule only a unique carries. A new power is a variant of `Power`, its one read, a phrase in `describe.rs`, and a wiki word, as an `Effect` is today.

## 10. What the player sees

| Where | What (the game is vague; the wiki is exact) |
|---|---|
| Dialogue | the filled texts; topics by their fixed names; one topic per quest |
| Messages | "Wat marks Wulfeshol on your map."; progress as today ("37 / 100 goblins.", `quest_news`); "Quest complete: Wanted."; a rank: "Wat names you a Pewter of the Weyfarers' Gilde." (said once, when the rank changes) |
| Quest log | the first breadcrumb as the quest is given ("Wat wants you to kill a goblin chief.", "Wat wants you to kill 100 goblins: goblins, goblin archers, and goblin chiefs.", "Wat wants you to bring him a scroll of frost.", "Wat wants you to kill Black Wystan."), the progress line as today, the rolled `where` ("Goblin chiefs are known in Cealdhlāw."), then "Return to Wat." Never a depth, a danger, a pool, or a chance. The same first breadcrumb for every quest of every giver |
| Character screen | a line under the alignment: "Weyfarers' Gilde: Tin", the title in its metal's colour (none before joining) |
| Look mode | the board's note; Wat by name |
| Item description | an identified unique's power in words ("Its every cut poisons."; "Its arrows put fear in what they strike."; "Its wearer sees in the dark."; "Its wearer knows no fear.") and its lore |
| Dumps | `quests` adds the breadcrumb objective; `stats` adds `Guild Weyfarers' Gilde, Tin`; JSON adds `guild` (`name`, `rank`) and per quest `target` (the rolled slug); `progress` (dev) adds each roll with its pool size, and the slain uniques; `truth` (dev) shows the four lairs' entrances by glyph, as it does Fennmynster's |

## 11. Talents and class abilities

**The system**: `bounties` (the guild's rolled quests and the tribes they count), placed in `data/talents.toml` beside `talk`. Quest rewards already have five talents under `talk` (Folk hero, Magnate, Well liked, Diplomat, Honeyed words: `quest_gold_percent`, `quest_xp_percent`); they apply to the guild through `complete` with no change. The new talents are about what is new: finding a rolled target, and the tribes.

| Talent | Tree | Ranks | Effect | The one read |
|---|---|---|---|---|
| **Quarry sense** | delver | 1 | arriving on a level that holds a living creature an active kill quest of yours asks for (a type, a tribe member, a criminal), a note: "Your quarry is here." | on arrival, where Sixth sense says the danger in words |
| **Sworn foe** (choice: a tribe, as Weapon focus takes a skill) | weaponcraft | 1 | +1 to hit and +2 damage against members of the chosen tribe | the attack profile, beside the ranger's Favoured prey |

Two talents meet the bar (`min_talents = 2`). **Class abilities**: none (`min_class = 0`); the ranger's Favoured prey could later read a tribe instead of a type, which is the ranger's design to reopen, not this one's. Unique items' powers extend `magic_items` and need no tag (the policy's point 3).

## 12. Spec, wiki, and docs

| File | Change |
|---|---|
| `spec/08b-creatures.md` §8 "Quests" | **giving**: the quest topic gives the quest, no accepting or declining, the first breadcrumb "X wants you to Y."; rolled quests (pick, variants, the roll at the giving, no rerolls, counting from the giving); a named monster's death counts at any hand and any time; tribes and their counting |
| `spec/03b-campaign.md` | Cēaptūn's Gildhalle and Wat; the chain, standing alone, and its ranks; the four lairs among the locations; the chain as the road to an ultimate ending (to come) |
| `spec/07a-levels.md` "Hidden locations" | the four lairs, revealed by a variant |
| `spec/08g-magic-items.md` | unique items and powers |
| `spec/07b-data.md` | `tribes`, `uniques`, `guilds` (the rank titles); the quest fields (`pick`, variants, `decline` retired, `reveals.when = "given"`) |
| `spec/06-input-and-ui.md` | the quest log's breadcrumb and rolled objectives; the character screen's guild line; one quest topic per quest |
| `spec/09-development-interface.md`, CLAUDE.md | the `quest_roll` dev action; the dump fields; `accept-ID`/`decline-ID` gone from the action text syntax |
| `docs/dev/quests.md` | giving, rolls, the stream, tribes, `slain`; a new `docs/dev/guild.md` only if the code earns a module of its own (`world/guild.rs` for ranks) |
| `docs/dev/magic-items.md` | uniques and powers |
| `docs/dev/campaign.md`, `overworld.md`, `mapgen.md` | the Gildhalle; the lairs' hidden entries; the four branches |
| Wiki | the quests page: quests are given by talking, never declined, and a named monster's death counts however it died. A page **"The Weyfarers' Gilde"** under the Ascmearc's quests: the chain, each link's pool exactly (generated: the link-1 types by level, the tribes and their members, the scrolls), the four criminals with their stats, lairs, and items, the seven ranks and what reaches each (the letters left unsaid there too). Generated tables: `xtask/src/wiki/tribes.rs`, `uniques.rs`, and a `guild.rs` for the pools and ranks; registered in `generate()` by topic. Names in the Ascmearc names table with their registers (Old English, Middle English, Victorian) |

## 13. Implementation plan

Each issue lands rules first, then its UI, then its content, and updates its spec section, its `docs/dev/` topic, and the wiki in the same PR. "State" marks what adds save state; "action" what adds or changes an action (the overlap rule: about three at once). New names and texts follow the language policy from the start (#378 reviews the existing ones).

| # | Issue | Contents | Needs | State / action |
|---|---|---|---|---|
| G1 | **Tribes** | `data/tribes/` (the seven, `except_note`), `content/tribe.rs` (members, checks), `Goal::Kill` takes a tribe target (`target = "tribe:kobolds"` in an authored quest), counting across levels; wiki table | — | — |
| G2 | **Giving and rolled quests** | for every quest: the quest topic gives it, `accept-ID`/`decline-ID` and `withdraw` removed, `decline` texts dropped from Hild's, Wulfrun's, and Leofric's files, `accept` said after the offer, `reveals.when = "given"`, the first breadcrumb in the log, old `offered` records load as active; a named monster's death (`Quests::slain`, any hand, any time, for every quest on a unique type); `pick`, `[[quest.variant]]`, placeholders (`{members}` too) and their length check, `Roll` in `QuestRecord`, the keyed stream, no rerolls, counting from the giving, reveals per variant; the `quest_roll` dev action; dumps; the system `bounties` with Quarry sense and Sworn foe (the policy: in the PR that adds the system) | G1 | state, action |
| G3 | **Unique items** | the `uniques` table, `unique:ID` placements, `Magic::unique`, the once-a-run check, the description and lore, the `venomous` affix; the three powers (`Inflict`, `Vision`, `Ward`) and their reads; wiki table | — | state |
| G4 | **Outlaws and the four criminals** | the `outlaws` faction, the footpad and footpad archer, the four criminal types with their gear (`unique:` lines); in `osrick.toml` the TODO note of §8.2 that he gets spells once creature spellcasters exist (verbatim in the issue); creature tests | G3 | — |
| G5 | **The guild** | the Gildhalle on the map, the board's note, `guildmaster` and Wat's dialogue (Cockney), the `guilds` data file with the seven ranks and the derived rank (character screen in the metal's colour, `stats`, the promotion message), links 1–3 in `wat.toml`; text snapshots; the wiki page | G2 | — |
| G6 | **The four lairs and link 4** | four branches, their hidden entries on reserved sites of `ascmearc.toml`, the four pinned vaults, `guild_wanted` with its variants and texts; Hild's line on Osrick; per-lair mapgen tests and a replay through the reveal | G4, G5 | — |

Order: G1 and G3 in parallel; then G2 and G4; then G5; then G6. At most two PRs with state or actions are open at once (G2 and G3).

**Tests** (the verification list per issue):

| Issue | Tests |
|---|---|
| G1 | membership from factions, `except`, `creatures`; unknown ids and empty tribes refused; tribe kills on two levels sum; a bled-out member counts, a trap kill does not; `faction_override` changes nothing |
| G2 | asking a quest topic gives the quest (active, logged with "Hild wants you to ..."), with no accept or decline topic left, for Leofric's, Hild's, and Wulfrun's quests too (their tests and replays rewritten); an `ask:E:accept-ID` step is refused; a reveal `when = "given"` fires on the first ask; a roll is the same for the same seed and state, survives a save and a replay (`replay --create`/`--verify`), and never changes on asking again; reachability (no Fennmynster type before its reveal, one after); kills before the giving do not count; a variant reveals its own lair and no other (view and `truth` dumps); a unique target's death counts by a trap, by another creature, and before the quest was given (Wulfrun's drake killed first, then the quest given: met at once); placeholder overflow refused at load; a text snapshot of the dialogue screen with a filled offer and of the quest log's breadcrumb; Quarry sense and Sworn foe each read once |
| G3 | a unique is made from its table with its affixes; unidentified it hides name, affixes, and power (`known_magic`); a second placement is a content error; Inflict poisons on every hit (Widowmaker) and frightens at its chance (High Toby, fixed stream), Vision gives darkvision while worn and not after, Ward refuses Frightened; the item description text snapshot |
| G4 | each criminal spawns with its unique wielded or worn; footpads and lair populations are not hostile to each other (`ai_tests` style); `osrick.toml` carries the spells note |
| G5 | Wat's topics in order; links 1–3 given one at a time with no level or quest gate besides the chain's own; Tin on being given link 1, Pewter on link 2 done; the scroll handed in leaves the pack, an unpaid one stays billed; rank lines on the character screen (text snapshot) and in `stats`; a `material` that names no material refused at load; `ceaptun` view snapshot updated |
| G6 | `gen --level wulfeshol:4` shows the vault with Wystan for each lair (insta snapshots of the generated floors); the reveal by `reveal:ID` and by the quest; a replay of link 4 with `quest_roll:guild_wanted:wystan` that kills him (the dev xp and a dev `afflict` where needed); his bow on the floor; the quest completes and Iron is reached. No golden screenshot: nothing here is visual beyond what text snapshots and dumps see |

## 14. Decisions (owner, 2026-10-26)

The owner's answers to the first draft's fourteen questions, and where each changed the design.

| # | Question | Decision | Changed |
|---|---|---|---|
| 1 | Link 1's pool | **Any non-unique of the three dungeons**, uniformly; no level band. (Of the three, those the player can enter when the quest is given, since the roll is never rerolled: §5.5) | §5.2, §5.5, §7 |
| 2 | Link 1's count | **One kill** | §7 |
| 3 | Earlier kills | **Do not count** for rolled quests (the baseline at the giving); a named monster is decision 7's rule | §5.6, §6.4 |
| 4 | Rerolling | **No rerolls.** Quests **cannot be declined** (a roguelike, not an MMO): speaking to the giver about a quest gives it and puts its first breadcrumb on the log ("X wants you to Y"). For every quest, the existing ones too | §1, §3, §5.2, §5.3, §5.4, §7.5, §10, §12, §13 |
| 5 | The tribes | **The seven as listed** (the ogre and troll out of the goblins, vermin in); the quest text makes the membership clear | §6.3 (`{members}`, `except_note`), §10 |
| 6 | Outlaws as a tribe | Not a tribe | §6.3 |
| 7 | A criminal's death | **Only the death counts**, at any hand; made the rule for killing any named monster (a unique), so no bug can softlock a quest. `slain_by` is gone | §5.2, §8.4, §13 |
| 8 | Link 4's gate | **The chain stands alone**, at least its first four links: no level or other quest gate | §7 |
| 9 | Ranks | **Titles only**, an improving title but not after every quest: two promotions in the chain. A veiled nod to isekai guilds' F-to-S ranks: seven ranks named by allomantic metals that exist in `assets/data/materials/`, ordered by value: Tin, Pewter, Iron, Steel, Duralumin, Electrum, Gold. The titles live in a data file | §7.6, §10 |
| 10 | After the chain | **No repeatable quests**; the repeatable-contract proposal is removed. The chain will be continued later; reaching its end will probably be a requirement of an ultimate ending | §5.3, §7, §15 |
| 11 | Osric's spells | A mace for now; a note in the follow-ups and a TODO note in his creature file, specified in G4, to add spells once creature spellcasters exist | §8.2, §13, §15 |
| 12 | Unique items' look | **A rare with set properties and a different name**; no new rarity or colour | §9.1 |
| 13 | Names | **Language policy**: the world's language evolved as ours did. Old English for the oldest places and things; Middle English for less old places and organisations; Victorian English for concepts contemporary to the game world. Commoners speak Cockney, the better off posh. The guild is new and an organisation (Middle English: the Weyfarers' Gilde, the Gildhalle); Wigmund became Wat One-Eye (Cockney); the criminals became Mother Sallow, Black Wystan, Brother Osrick, and Sergeant Herrick, with Victorian bynames; the outlaws footpads; the items Widowmaker, High Toby, the Unhalwed Bedes, and the Turncoat; the lairs keep their Old English names (old, reserved sites). Sample dialogue is in each speaker's register. The review of all existing names and dialogue is #378 | §4, §8, §9, §10 |
| 14 | The items' strength | **Kept**: one item, like ADOM's black torc or golden gladius, relatively easy to get and very strong. The four made very distinct: each has a power and affixes for its own way to play (poison that builds, fear at range, a caster in the dark, an unshakable wall) | §9 |

## 15. Follow-ups

| Follow-up | Where it is tracked |
|---|---|
| **Brother Osrick's spells**: once creature spellcasters exist, give him spells (raising the dead, a curse, cold) in place of fighting with a mace only | the TODO note at the top of `campaigns/ascmearc/data/creatures/osrick.toml` (G4); the creature-magic design, when it is written |
| **The chain continues**: links 5 on, each a quest file with `requires` the last, reaching the reserved ranks Steel, Duralumin, Electrum, and Gold; the chain's end probably a requirement of an ultimate ending (the story-driven victory) | a later design; the reserved rank lines in `weyfarers.toml` |
| **Names and dialogue review** against the language policy (decision 13), for everything written before this design | #378 (after this design merges) |
| The ranger's Favoured prey reading a tribe instead of a type | the ranger's design, if reopened |

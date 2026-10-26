# Warlock — The adventurers' guild: rolled quests in a chain, tribes, wanted criminals, unique items

Design document for the owner's review (2026-10-26). Written against `origin/main` at 1c0ce120, the approved campaign design (`docs/design/campaign.md`), and the code it produced (#327 hidden locations, #336 Cēaptūn, #340 its quests). Nothing here is implemented; §13 cuts it into issues. §14 lists the open questions, each with a recommendation.

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
14. Open questions for the owner

## 1. Summary

**What the owner asked for**: an adventurers' guild in Cēaptūn that hands out **semi-random quests in a chain**, designed as data so later links are added without code: (1) kill a random creature that can spawn in the first three dungeons; (2) kill 100 of a random **tribe**; (3) bring a random scroll; (4) kill a random **wanted criminal**, whose quest **reveals a new dungeon** with the criminal at its bottom, each criminal dropping a **unique item** that fits them.

| Piece | Proposal |
|---|---|
| Where | the **Gildhūs** ("guild house"), a small timber hall in Cēaptūn's south-east corner beside Factor Wulfrun's office; the **Wanderers' Guild**. §4 |
| Who | **Wigmund**, the guildmaster, one npc on a new campaign type `guildmaster`; the chain's four quests are his topics, one at a time, as Leofric's are. §4 |
| Rolled quests | an ordinary `[[quest]]` whose target is a **`pick`** (a pool drawn by a rule from data that already exists: branch populations, tribes, scrolls) or a list of authored **`[[quest.variant]]`**s (the four criminals). The result, a **`Roll`** (target slug, variant id, count), is stored in the quest's `QuestRecord` when the quest is **offered**; texts use placeholders (`{target}`, `{count}`, `{where}`). No generated `QuestDef`s: the template is the content, the roll is the save. §5 |
| Randomness | a keyed stream per roll, **`quest/<quest id>/<times done>/<game second>`**: nothing to save but the result, no draw on `gameplay`, and a declined offer asked again later rolls anew (time is the penalty, no lock-out). §5.3 |
| Tribes | new shared data `data/tribes/<id>.toml`: a name and its members, by faction with additions and exceptions (`factions = ["kobolds"]`). Seven tribes from today's creatures. A tribe kill is any kill of the player's (the existing tally) of a member type, anywhere, after the quest was accepted. §6 |
| The chain | four quests linked by the existing `requires`; a guild file names the ranks they give (**Recruit, Hunter, Slayer, Finder, Thief-taker**). Link 5 is a new quest file with `requires = "guild_wanted"`. §7 |
| The wanted | **Ealdgyth the Poisoner** (Wyrtholm, Micelfenn), **Wulfstan Bloodhand** (Wulfeshol, Lindwudu), **Brother Osric the Unhallowed** (Bānfeld, Hlāwdūn), **Hereric Oathbreaker** (Brocenburg, Holegnwudu): level-12 uniques of a new `outlaws` faction, each at the bottom of a hidden four-level branch (danger 6 to 9) on a reserved site; the rolled one's lair is revealed as the quest is accepted, the other three stay hidden for that run. §8 |
| Unique items | **Ætterthorn** (stiletto), **Feorrsceota** (long bow), **the Unhallowed Rosary** (bone amulet), **Oathbreaker** (banded mail): a new `uniques` table of fixed rare items (base object, name, affixes at chosen tiers, an optional **power**). Three powers are new rules: a status on hit, a vision profile while worn, a status ward. The bow needs none. §9 |
| Talents | a new system `bounties` with **Quarry sense** (delver) and **Sworn foe** (weaponcraft, a tribe as its choice); quest rewards already have five talents under `talk`. No class ability (`min_class = 0`). §11 |
| Slicing | six issues: tribes, rolled quests (+ the system and its talents), unique items, outlaws and the four criminals, the guild and links 1–3, the four lairs and link 4. §13 |

## 2. Research, tied to the decisions

| Finding | Source | Used in |
|---|---|---|
| Adventurers' guilds and job boards hand out repeatable, procedurally parameterised jobs ("kill N of X", "bring Y"); the best remembered are those whose chain ends in a named target | Elona (the Adventurers' Guild, town boards), Daggerfall (the Fighters Guild's rank-gated jobs), Monster Hunter (rank-gated hunts) | §5 template + roll; §7 ranks |
| Daggerfall's guild ranks unlock tougher jobs and nothing else matters as much as the title the player gets | Daggerfall, UESP "Fighters Guild" | §7.6 ranks as titles, gates by `requires` |
| ADOM's quests are fixed per run but some parameters are rolled per character (the wanted bandit's location, the item a quest-giver wants) | ADOM Guidebook, "Quests" | §5.3 the roll stored per character |
| Angband's and DCSS's uniques carry or guard a fixed artifact ("randart" vs "unrandart"): a fixed, named item with a theme is what players remember | Angband `artifact.txt`, DCSS unrandarts | §9 fixed uniques, one per criminal |
| Elona's hunt quests that require the kill *by the player* frustrate when a pet or a trap gets it; the board pays for the death | Elona community wiki, "Hunting quest" | §8.4 a wanted's death by any hand counts |
| Bounty boards that reroll for free invite save-scumming the board; those that never reroll lock players out of the chain | Elona, Moonring discussions | §5.3 a declined offer rerolls only once time has passed |

## 3. What exists, and what this builds on

| Piece | Today (`docs/dev/quests.md`, `npcs.md`, `overworld.md`, `magic-items.md`, `scrolls.md`) | What the guild needs |
|---|---|---|
| Quest data | `QuestDef` (`content/quest.rs`): `Goal::{Kill{creature, count, level}, Bring{object, count, to}, Reach{level}, Deliver{object, to}}`, `Reward`, `QuestTexts`, `requires` (one or a list), `reveals = { location, when }`, `repeatable`, `where` | a target that is **rolled** (§5), a kill target that is a **tribe** (§6) |
| Quest state | `Quests` in `WorldState`: `records` by id (`stage`, `completed`, `counted`, `reached`), the kill tally by level and creature type, `prey` | `QuestRecord::roll` (§5.4); nothing else new |
| Kill counting | `fight.rs::dies` → `tally_kill` (the player's own kills, bleeding included; never traps or others); "those before you took the quest up too" | rolled kills count from acceptance (the `counted` baseline that repeats already use) |
| Reveals | `World::reveal_location(id)`; `reveal_at` at the stage the quest's `reveals.when` names; the dev action `reveal:ID` | a reveal per **variant** (§5.2) |
| Chains | `requires`: Leofric, Hild, and Wulfrun each offer one quest at a time | unchanged: the guild's chain is `requires` (§7) |
| Streams | `Rng::stream(seed, label)`; the lazily kept `levels_rng`; keyed per-level streams (`spawn/<level>`, `gear/<key>`) | one keyed stream per roll, never kept (§5.3) |
| Placed magic items | a level's `[[item]]` places a rare with a `name` and `affixes = ["sharp:2"]` | the same item from a table, placeable anywhere objects are (§9) |
| Creatures | `unique = true` (a population draws it at most once), pinned branch vaults (`[[dungeon.levels]] vaults`, the drake's hoard) | four uniques placed by four pinned vaults (§8) |

## 4. The guild in Cēaptūn

### 4.1 The Gildhūs

A timber hall, 9×7 with its walls, in the free grass of the town's south-east corner (about `[77, 22, 85, 28]` in `levels/ceaptun.toml`, east of Wulfrun's office and west of the mill race; the two trees there go). Its door opens north on the lane, which is extended a cell east to meet it. Inside: plank floor, a lamp, a table, Wigmund at a post by the far wall, and one **named place** (#332's `name` and `note` on a legend cell, no new terrain): *"the guild's board"*, note *"Bounties nailed up three deep, most of them years old."* The town comment block and `docs/dev/campaign.md` "Cēaptūn" name it.

| Field | Value |
|---|---|
| Building | the Gildhūs (Old English *gild* + *hūs*, "guild house", beside the Gæsthūs in the same register; design §5.8's rule) |
| Guild | the Wanderers' Guild (English: an institution, named as the player would hear it; *wrecca*, "wanderer, exile", is the word under it, given in the wiki's names table) |
| NPC | **Wigmund** (*wīg* "war" + *mund* "protection"), guildmaster: a one-eyed old delver who has stopped going down |

### 4.2 Wigmund

| Data | Content |
|---|---|
| Creature type | `campaigns/ascmearc/data/creatures/guildmaster.toml`: peaceful, `budge = "off_post"`, `@`, the townsfolk's stats with Strength and Toughness of a retired fighter; never fights (as the reeve) |
| Npc | legend `"M" = { terrain = "plank_floor", creature = "guildmaster", npc = "wigmund" }` |
| Dialogue | `campaigns/ascmearc/data/dialogue/ceaptun.toml` gains `wigmund`: `name = "Wigmund"`, `greet`, five rotating `lines` (the old days, the eye, the board, the town's ingratitude, the barrow), `greet_evil` ("We don't take your kind of work."; he still offers the chain: none of its quests is good-aligned, §7) |
| Topics | `guild` ("The guild": who they are, how ranks work) and `board` ("The board": a reply that points at the chain's next job). With the quest topic and its accept and decline, four of nine |

Sample lines (one line each, ≤280 characters, as every text): greet *"Another one off the pass. Sit, if you like; standing costs the same."*; `guild` *"We take the work the reeve can't pay for and the sisters won't touch. Do a job, come back, get paid. Do enough of them and the town starts calling you by a title instead of 'you there'."*

## 5. Rolled quests: the data model

### 5.1 Why templates and a stored roll, not generated quests

| | A `QuestDef` generated at run time, saved whole | **A template in data + a roll in the record** (proposed) |
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
giver = "wigmund"
type = "kill"
pick = { creatures = ["cealdhlaw", "fennmynster", "fyrdelf"], band = [-1, 2] }
count = 1
reward = { gold = 40, xp = 60 }
offer = "There's a price on {a_target}. Bring the beast down, anywhere you find it, and the guild pays. {Where}"
accept = "Good hunting. It counts from today: what you killed before is your own business."
reminder = "{A_target}. {Where}"
done = "That's the one. Here's the guild's coin, and welcome: you're a Hunter now."
where = "{Targets} are known in {branch}."
```

```toml
[[quest]]
id = "guild_wanted"
name = "Wanted"
giver = "wigmund"
requires = "guild_scroll"
type = "kill"
slain_by = "anyone"               # §8.4
reward = { gold = 500, xp = 800 }
alignment = 10
offer = "{offer}"                 # a variant's own texts replace these
accept = "{accept}"
reminder = "{reminder}"
done = "{done}"

[[quest.variant]]
id = "wulfstan"
target = "wulfstan"
reveals = { location = "wulfeshol", when = "accepted" }
where = "His den, Wulfeshol, lies in Deorcholt, deep in Lindwudu."
offer = "Wulfstan Bloodhand. Robbed the Fennhām road for six years, then killed a carter for his boots. The reeve's men won't go into Deorcholt after him. Will you?"
# accept, reminder, done ...
```

| Field | Meaning | Checked at load |
|---|---|---|
| `pick.creatures = [branches]`, `band = [lo, hi]` | a creature type from those branches' `population`, not `unique`, whose level is within the player's level + `lo` to + `hi`; only branches the player can enter (§5.5) | branches exist and are the quest's campaign's |
| `pick.tribes = [ids]` or `"all"` | a tribe (§6) with a member in a branch the player can enter | tribes exist |
| `pick.objects = { class = "scrolls", loot = true }` | an object of that item class; `loot = true` only those loot can draw (`frequency > 0`) | class exists; the pool is not empty |
| `[[quest.variant]]` (`id`, `target`, optional `weight`, `reveals`, `where`, and any of the five texts) | an authored option; what it gives replaces the quest's own | a variant's target fits the `type`; its `reveals` names a hidden location of the campaign |
| `slain_by = "anyone"` | a kill goal met by the target's death at any hand (§8.4); default `"player"`, today's rule | only on `kill`, only with a unique target or variants of unique targets |
| `count` | as today; on a tribe pick the tribe's own `count` wins if it has one (§6.3) | |

A quest has `target`, `pick`, or variants, exactly one. **Placeholders**, filled from the roll in every text, the `name` excepted: `{target}` / `{targets}` (the type's, tribe's, or object's name and plural), `{a_target}` (with its article; the unique names of §8 take none), `{count}`, `{branch}` (the first branch, in pick order, whose population holds it: the place the guild "knows" it from), `{where}` (the rolled `where`), and capitalised forms (`{Targets}`, `{A_target}`, `{Where}`). A text over 280 characters after filling, with the longest name the pool can give, is a content error naming the quest.

### 5.3 When and how the roll is made

| Question | Proposal |
|---|---|
| When | when the quest is **offered** (`World::offer`, the first `ask:E:quest-ID`), so the offer can name the target; the record is created then, as today, now with its `roll` |
| Stream | `Rng::stream(seed, "quest/<id>/<completed>/<game second>")`: keyed, used once, never kept. Nothing about the stream is saved; the roll is |
| Asking again while offered | the stored roll: same target |
| Declining | `withdraw` forgets the record (today's rule), roll included. Asked again **in the same game second** (talk about quests takes no time), the key is the same and so is the target; asked after anything that passes time (a step, a wait), a new roll. No lock-out and no free reroll: time is the price (the owner's rule) |
| Accepting | the roll is fixed until done; a kill goal's `counted` baseline is the tally then (§5.6); a variant's `reveals` fires |
| Repeatable templates | `<completed>` in the key gives each repeat its own roll |
| Inputs | the player's level, the revealed locations, the content: all in the state, so a replay rolls the same |
| Order | candidates in content order (creature, object, and tribe indices; variants in file order), uniform or by `weight`: never `HashMap` order |
| Dev | `quest_roll:ID:TARGET` (a dev action, no time; refused unless the quest is offered and TARGET is in its pool or a variant id): replays, tests, and captures pick a criminal without hunting a seed |

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
```

Slugs, not indices (as affixes are saved), so adding creatures keeps saves. `Quests::unknown_to` also checks a roll's target, variant, and tribe. Runs without rolled quests hash as before (the field is left out). `FORMAT_VERSION` is the integrator's, once per release.

### 5.5 What the player can reach

A branch "the player can enter" is one whose overworld entrance is not hidden, or is in `WorldState::revealed` (`Content::hidden_location` and the state, nothing new). So the first bounty never names a fen leech before Sister Hild reveals Fennmynster. If the band is empty (a level-20 character with only Cealdhlāw open), the pool takes the types nearest the band instead (the highest levels on offer). The pool is never empty while one branch is open.

### 5.6 Counting a rolled kill

From acceptance only: `accept` sets `counted` to the tally's count at that moment for every rolled kill quest (today it does so only for repeats). The rest is today's `progress`: tally minus `counted`, capped at the count. An authored quest keeps today's rule (earlier kills count).

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

### 6.3 The seven tribes from today's creatures

| Tribe | Members | Where they live (reachable from) |
|---|---|---|
| kobolds | kobold, kobold archer, kobold shaman | Cealdhlāw, Fyrdelf |
| goblins | goblin, goblin archer, goblin chief | Cealdhlāw |
| the undead | skeleton, zombie, barrow ghoul, drowned, ghost, ash wraith, barrow wight | Cealdhlāw, Fennmynster, Fyrdelf |
| vermin | rat, grave worm, barrow spider, giant centipede (the basilisk excepted) | Cealdhlāw, Fyrdelf |
| fen things (`marsh`) | giant leech, giant frog, crocodile, will-o'-wisp, mud golem, marsh troll, bog hag | Fennmynster |
| duergar | duergar miner, duergar warrior, duergar smith (the trader excepted) | Fyrdelf |
| fire things | ember beetle, fire salamander, magma crawler, cinder elemental, fire drake | Fyrdelf |

The cult (two types) is left out: 100 cultists is a grind of one floor type. Outlaws (§8.2) become a tribe only if the owner wants (question 6).

### 6.4 Counting rules

| Case | Counts? | Why |
|---|---|---|
| a member killed by the player, anywhere (any branch, the overworld, a local area, a hand-made level) | yes | the tally is per level; a tribe sums every level |
| bled to death of the player's wound, shot, thrown at, burnt by the player's scroll or spell | yes | all are `tally_kill` today |
| killed by a trap, another creature, or the player's absence | no | not the player's kill; spawns refill floors, so the count stays reachable |
| killed before the quest was accepted | no | §5.6 |
| a member that is unique (the barrow wight, the bog hag, the drake) | yes, one each | a type is a type |
| an outraged shopkeeper, a peaceful villager | no | not members |

## 7. The chain, link by link

The chain is four `[[quest]]`s in `campaigns/ascmearc/data/quests/wigmund.toml`, each `requires` the one before, so Wigmund shows one topic at a time (Leofric's pattern). A link added later is one more quest with `requires = "guild_wanted"`; a link made repeatable is `repeatable = true` (its roll keyed by `<completed>`).

| # | Id, topic | Goal | Pool | Reward | Alignment | Rank once done |
|---|---|---|---|---|---|---|
| 1 | `guild_quarry`, "A bounty" | kill 1 of a rolled type | `creatures` of Cealdhlāw, Fennmynster, Fyrdelf, level within −1 to +2 of the player's | 40 gold, 60 XP | 0 | Hunter |
| 2 | `guild_hundred`, "The hundred" | kill 100 of a rolled tribe | `tribes = "all"`, reachable | 250 gold, 500 XP, 2 red potions | 5 | Slayer |
| 3 | `guild_scroll`, "A scroll for the guild" | bring 1 rolled scroll to Wigmund | `objects = { class = "scrolls", loot = true }`: fireball, lightning, frost, identify (the scroll of light, sold in town and never loot, is out) | 200 gold, 150 XP | 0 | Finder |
| 4 | `guild_wanted`, "Wanted" | kill the rolled criminal (`slain_by = "anyone"`) | four variants, §8 | 500 gold, 800 XP (the unique item is the criminal's, §9) | 10 | Thief-taker |

The XP sits between Leofric's (30 to 250) and Wulfrun's drake (900); the kills themselves pay their own XP. Accepting link 1 makes the player a **Recruit**.

### 7.1 Link 1: a bounty

The target comes from the normal population and spawner: the guild places nothing. It **spawns out of the player's sight** like any creature (the spawner's rules) or is already on a floor they have not reached; the log names the branch it is known in (`{branch}`), never a depth (the danger is hidden). Quarry sense (§11) tells a player on its level. A target another creature kills does not count; the next one will. Count 1 (question 2).

### 7.2 Link 2: the hundred

100 is long by design: about four floors' populations. The log counts ("37 / 100 killed."), as kill quests already do; which types belong is told in words in the offer ("kobolds of any kind: archers, shamans, the lot") and exactly in the wiki.

### 7.3 Link 3: a scroll

**Identified or not**: scrolls have no unidentified state in this game (they are never magic and are named as they are, `docs/dev/magic-items.md`), so the question does not arise today. Should scroll appearances come, the hand-in matches the object type whatever the player knows ("the guild knows its scrolls"). **Consumed**: yes, as every bring (it leaves the pack). A scroll picked up in a shop and handed in unpaid stays on the keeper's bill (today's rule for anything used up).

### 7.4 Link 4: wanted

§8. The variant's `reveals` marks the lair on the map as the quest is accepted ("Wigmund marks Wulfeshol on your map.").

### 7.5 Failure and retry

There is nothing to fail: no timers, no abandoning, no cap on active quests (the owner's decisions). The only ways a link could stall, and why they don't:

| Risk | Answer |
|---|---|
| a rolled type or tribe never appears | the pool takes only branches the player can enter, and floors refill |
| the criminal dies by another hand or a trap | it counts (§8.4) |
| the player declines a target they dislike | asking again after time passes rolls again (§5.3) |
| the scroll is read or sold | another can be found or bought; nothing is lost |
| the player dies | permadeath, as everywhere |

### 7.6 Ranks

`campaigns/ascmearc/data/guilds/wanderers.toml`:

```toml
[[guild]]
id = "wanderers"
name = "the Wanderers' Guild"
master = "wigmund"
ranks = [
  { title = "Recruit", accepted = "guild_quarry" },
  { title = "Hunter", done = "guild_quarry" },
  { title = "Slayer", done = "guild_hundred" },
  { title = "Finder", done = "guild_scroll" },
  { title = "Thief-taker", done = "guild_wanted" },
]
```

The rank is **derived** from the quest records (the last line whose quest is at that stage), so it is not saved. It is a title: the character screen and the `stats` dump show it; nothing reads it as a rule (question 9). Later links add rank lines.

## 8. The wanted: four criminals and their lairs

### 8.1 Shape

Each criminal is a **campaign creature** (`unique = true`, in no population, placed only by a pinned vault on the bottom floor of their own branch), at **level 12** (the ogre's level, a step above the bog hag), in the new shared faction **`outlaws`** (hostile to the player only; no faction lists it as hostile, so the lair's other inhabitants never fight its master). Each branch is four floors at danger 6 to 9 (`danger = 6`, one a floor), hidden on a **reserved site** of the overworld (design §5.5) until the variant reveals it. All four exist in every run; three stay hidden. Their creatures already exist, plus two new campaign types: the **outlaw** (level 6, short sword and jerkin, `short_blades = 2`) and the **outlaw archer** (level 7, short bow, `bows = 2`).

### 8.2 The four

| | Ealdgyth the Poisoner | Wulfstan Bloodhand | Brother Osric the Unhallowed | Hereric Oathbreaker |
|---|---|---|---|---|
| Crime | herb-wife of Fennhām who sold death to heirs; three dead at one wedding feast | highwayman of the Fennhām road; killed a carter for his boots | a brother of the Mynster, cast out for robbing graves and raising what he robbed (Sister Hild knows the name; one line of hers) | a soldier of the fort who killed his captain and ran; now hires out to goblins |
| Id, glyph | `ealdgyth`, `@` pale green | `wulfstan`, `@` russet | `osric`, `@` bone white | `hereric`, `@` iron grey |
| Stats (St De To In Wi Pr) | 11 17 13 16 14 12 | 15 18 15 11 12 14 | 13 11 15 17 19 15 | 20 12 19 10 13 13 |
| Base hp, speed | 24, 1100 | 30, 1000 | 32, 1000 | 40, 1000 |
| Fights | melee; wields Ætterthorn, whose venom poisons (§9) | `ranged`, range 8, sight 14; wields Feorrsceota, 40 arrows | melee with a mace (`maces = 2`), wears the Rosary (darkvision through it) | melee; a great axe (`axes = 3`, `great_weapon = 2`, 50 % magic), wears Oathbreaker |
| Skills, weapon skills | stealth 60, heal 50; `daggers = 3` | stealth 40, spot 50; `bows = 3`, `marksman = 1`, `short_blades = 2` | concentration 50, spellcraft 50; `maces = 2` | intimidate 40; `axes = 3`, `great_weapon = 2` |
| Resists, other | poison 100; footsteps 10 (quiet); vision `low_light` | vision normal | cold 50; poison 50 | Frightened never takes him (his mail) |
| Lair (Old English meaning), region | **Wyrtholm** ("herb islet"), Micelfenn | **Wulfeshol** ("wolf's hole"), Deorcholt in Lindwudu | **Bānfeld** ("bone field"), a barrow field in Hlāwdūn | **Brocenburg** ("the broken fort"), the ruined tower in Holegnwudu |
| Generator, size | caves, 100×40 | caves, 100×40 | rooms, 80×21 | rooms, 80×21 |
| Population | giant leech, giant frog, crocodile, will-o'-wisp, mud golem, marsh troll, giant centipede, outlaw | outlaw, outlaw archer, bat, giant centipede, barrow spider, goblin archer | skeleton, zombie, barrow ghoul, ghost, drowned, grave worm, barrow spider | goblin, goblin archer, goblin chief, outlaw, outlaw archer, ogre, bat |
| Traps, other | pit, alarm; temperature 9 | pit, arrow, alarm | arrow, spear, pit, alarm; `room_effects` shadowed, hushed; key `rusty_key` | arrow, spear, alarm; 2 locks a floor; key `iron_key` |
| Bottom vault (pinned, floor 4) | "the still-room": her bench, jars (red potions, angelica), her | "the robbers' hoard": a cave mouth of loot (`$`, one `loot = "rare"`), him | "the unhallowed chapel": a defiled altar, bones, him | "the captain's hall": a broken table, the captain's grave, him |

Ogres (12) appear in Brocenburg only out of depth; that is the point of hiring them. A caster's spells are only announced today (`docs/dev/creatures.md`), so Osric fights with a mace until creature magic is designed (question 11).

### 8.3 Where a criminal is placed, and when

The bottom floor is generated on its first visit (today's rule), so the criminal is always placed out of sight, in the vault, asleep like the drake: it wakes to what it sees or hears. Nothing is spawned for the quest elsewhere, and nothing moves on a floor the player is not on.

### 8.4 A death at any hand

`slain_by = "anyone"`: `creature_dies` marks the active quest whose rolled unique target died (`QuestRecord::reached = true`, the reach flag, reused: "the goal happened"), whoever killed it. Since floors the player is not on are frozen and the criminal's own lair does not fight it, in practice that means the player, a trap the player led it into, or the player's scroll. The log then says "Return to Wigmund." This is the one place the log may know something the player did not see (a trap's kill behind a door); the alternative, a bounty no one can finish, is a lock-out (question 7).

## 9. Unique items

### 9.1 The data (`campaigns/ascmearc/data/uniques/<id>.toml`; a shared `data/uniques/` too)

```toml
[[unique]]
id = "aetterthorn"
name = "Ætterthorn"            # Old English "poison thorn"
object = "stiletto"
affixes = ["venomous:2", "keen:2", "of_precision:3", "of_the_lynx:2"]   # the [[item]] syntax
parts = [{ of = "bronze" }]    # optional, as [[item]]
level = 12
power = { inflict = "poisoned", chance = 25, severity = 2 }
lore = "A dark, thin blade that weeps a little at the point."   # the item description's last line
```

| Question | Proposal |
|---|---|
| Rarity | **rare** with the fixed `name` (exactly what a level's `[[item]]` with a `name` makes today), plus `Magic::unique: Option<String>` (the slug, saved like affixes) so its power and lore are found again. No new rarity tier or colour (question 12) |
| Where it may be placed | anywhere objects are named: `unique:ID` in a gear line's `one_of`, a legend's `wield`/`wear`/`carry`/`objects`, a pile, a quest reward's `items` |
| Once a run | each unique is named by **one** placement in all content (a load check): with pinned vaults and unique criminals it is made at most once, with no save state |
| Identification | as any magic item: unidentified it is "a stiletto" with a rare's unknown look; identified, its name, affixes, power, and lore. Presentation reads only `Stack::known_magic`, as today |
| Loot | never drawn by `loot::roll` |

### 9.2 The four

| Item | Base | Affixes (tier) | Power | New rule? |
|---|---|---|---|---|
| **Ætterthorn**, Ealdgyth's | stiletto | **venomous** 2 (new affix, data only: `elemental`, `damage_type = "poison"`, weapon and ammo), keen 2, of precision 3, of the lynx 2 | a hit may poison: 25 %, 2 degrees of Poisoned | **yes**: `Power::Inflict { status, chance, severity }`, read where a natural attack's `on_hit` is applied (`fight.rs`), for the weapon that hit; the target's poison resistance applies as it does to `on_hit` |
| **Feorrsceota** ("far-shooter"), Wulfstan's | long bow | hunter's 3 (range), keen 2, steady 2, of precision 3, of the lynx 2 | none | **no**: affixes only |
| **The Unhallowed Rosary**, Osric's | bone amulet | mystic 3 (max mana), of clarity 3 (mana regeneration), of the sage 2 (Willpower), of warmth 2 (cold resistance) | its wearer sees with `darkvision` | **yes**: `Power::Vision(profile)`, read where an actor's vision profile is chosen (the best of race and power; the dev F3 profile still wins) |
| **Oathbreaker**, Hereric's | banded mail | reinforced 3, hardened 3, of the ox 2, hale 3 | Frightened never takes its wearer | **yes**: `Power::Ward(status)`, read in the one place a status is applied (`World::afflict`): refused, with a line ("Fear finds no hold on you.") |

Each is weapon, launcher, amulet, armour: one for each way to fight. The powers are a `power` table, not an affix: an affix is a value with tiers that loot rolls, and a power is a rule only a unique carries. A new power is a variant of `Power`, its one read, a phrase in `describe.rs`, and a wiki word, as an `Effect` is today.

## 10. What the player sees

| Where | What (the game is vague; the wiki is exact) |
|---|---|
| Dialogue | the filled texts; topics by their fixed names |
| Messages | "Wigmund marks Wulfeshol on your map."; progress as today ("37 / 100 goblins.", `quest_news`); "Quest complete: Wanted."; a rank: "You are a Hunter of the Wanderers' Guild now." (said once, when the rank changes) |
| Quest log | the objective from the roll ("Kill a goblin chief.", "Kill 100 goblins.", "Bring a scroll of frost to Wigmund.", "Kill Wulfstan Bloodhand."), the progress line as today, the rolled `where` ("Goblin chiefs are known in Cealdhlāw."). Never a depth, a danger, a pool, or a chance |
| Character screen | a line under the alignment: "Wanderers' Guild: Hunter" (none before Recruit) |
| Look mode | the board's note; Wigmund by name |
| Item description | an identified unique's power in words ("Its point may poison."; "Its wearer sees in the dark."; "Its wearer knows no fear.") and its lore |
| Dumps | `quests` adds the objective as above; `stats` adds `Guild Wanderers' Guild, Hunter`; JSON adds `guild` (`name`, `rank`) and per quest `target` (the rolled slug, once offered); `progress` (dev) adds each roll with its pool size; `truth` (dev) shows the four lairs' entrances by glyph, as it does Fennmynster's |

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
| `spec/08b-creatures.md` §8 "Quests" | rolled quests (pick, variants, the roll, when, decline, counting from acceptance), the `slain_by` rule, tribes and their counting |
| `spec/03b-campaign.md` | Cēaptūn's Gildhūs and Wigmund; the chain and its ranks; the four lairs among the locations |
| `spec/07a-levels.md` "Hidden locations" | the four lairs, revealed by a variant |
| `spec/08g-magic-items.md` | unique items and powers |
| `spec/07b-data.md` | `tribes`, `uniques`, `guilds`; the quest fields |
| `spec/06-input-and-ui.md` | the quest log's rolled objectives; the character screen's guild line |
| `spec/09-development-interface.md`, CLAUDE.md | the `quest_roll` dev action; the dump fields |
| `docs/dev/quests.md` | rolls, the stream, tribes; a new `docs/dev/guild.md` only if the code earns a module of its own (`world/guild.rs` for ranks) |
| `docs/dev/magic-items.md` | uniques and powers |
| `docs/dev/campaign.md`, `overworld.md`, `mapgen.md` | the Gildhūs; the lairs' hidden entries; the four branches |
| Wiki | a page **"The Wanderers' Guild"** under the Ascmearc's quests: the chain, each link's pool exactly (generated: the link-1 types by level, the tribes and their members, the scrolls), the four criminals with their stats, lairs, and items, the ranks. Generated tables: `xtask/src/wiki/tribes.rs`, `uniques.rs`, and a `guild.rs` for the pools; registered in `generate()` by topic. Names in the Ascmearc names table |

## 13. Implementation plan

Each issue lands rules first, then its UI, then its content, and updates its spec section, its `docs/dev/` topic, and the wiki in the same PR. "State" marks what adds save state; "action" what adds an action (the overlap rule: about three at once).

| # | Issue | Contents | Needs | State / action |
|---|---|---|---|---|
| G1 | **Tribes** | `data/tribes/` (the seven), `content/tribe.rs` (members, checks), `Goal::Kill` takes a tribe target (`target = "tribe:kobolds"` in an authored quest), counting across levels; wiki table | — | — |
| G2 | **Rolled quests** | `pick`, `[[quest.variant]]`, placeholders and their length check, `Roll` in `QuestRecord`, the keyed stream, offer/decline/accept rules, counting from acceptance, `slain_by`, reveals per variant; the `quest_roll` dev action; dumps; the system `bounties` with Quarry sense and Sworn foe (the policy: in the PR that adds the system) | G1 | state, action |
| G3 | **Unique items** | the `uniques` table, `unique:ID` placements, `Magic::unique`, the once-a-run check, the description and lore, the `venomous` affix; the three powers and their reads; wiki table | — | state |
| G4 | **Outlaws and the four criminals** | the `outlaws` faction, the outlaw and outlaw archer, the four criminal types with their gear (`unique:` lines); creature tests | G3 | — |
| G5 | **The guild** | the Gildhūs on the map, the board's note, `guildmaster` and Wigmund's dialogue, the `guilds` table and the derived rank (character screen, `stats`), links 1–3 in `wigmund.toml`; text snapshots; the wiki page | G2 | — |
| G6 | **The four lairs and link 4** | four branches, their hidden entries on reserved sites of `ascmearc.toml`, the four pinned vaults, `guild_wanted` with its variants; per-lair mapgen tests and a replay through the reveal | G4, G5 | — |

Order: G1 and G3 in parallel; then G2 and G4; then G5; then G6. At most two PRs with state or actions are open at once (G2 and G3).

**Tests** (the verification list per issue):

| Issue | Tests |
|---|---|
| G1 | membership from factions, `except`, `creatures`; unknown ids and empty tribes refused; tribe kills on two levels sum; a bled-out member counts, a trap kill does not; `faction_override` changes nothing |
| G2 | a roll is the same for the same seed, time, and state, and survives a save and a replay (`replay --create`/`--verify`); declining and asking at once gives the same roll, after a wait a new one; reachability (no Fennmynster type before its reveal, one after); the band and its fallback; kills before acceptance do not count; a variant reveals its own lair and no other (view and `truth` dumps); `slain_by` counts a trap's kill; placeholder overflow refused at load; a text snapshot of the dialogue screen with a filled offer; Quarry sense and Sworn foe each read once |
| G3 | a unique is made from its table with its affixes; unidentified it hides name, affixes, and power (`known_magic`); a second placement is a content error; Inflict poisons at its chance (fixed stream), Vision gives darkvision while worn and not after, Ward refuses Frightened; the item description text snapshot |
| G4 | each criminal spawns with its unique wielded or worn; outlaws and lair populations are not hostile to each other (`ai_tests` style) |
| G5 | Wigmund's topics in order; links 1–3 offered one at a time; the scroll handed in leaves the pack, an unpaid one stays billed; rank lines on the character screen (text snapshot) and in `stats`; `ceaptun` view snapshot updated |
| G6 | `gen --level wulfeshol:4` shows the vault with Wulfstan for each lair (insta snapshots of the generated floors); the reveal by `reveal:ID` and by the quest; a replay of link 4 with `quest_roll:guild_wanted:wulfstan` that kills him (the dev xp and a dev `afflict` where needed); his bow on the floor; the quest completes. No golden screenshot: nothing here is visual beyond what text snapshots and dumps see |

## 14. Open questions for the owner

1. **Link 1's pool**: a band around the player's level (−1 to +2, recommended), or any non-unique type of the three dungeons uniformly (a level-1 character may draw a cinder elemental)? *Recommendation: the band, with only reachable branches.*
2. **Link 1's count**: one kill (recommended, as the brief says), or a count by the target's level (3 rats, 1 ogre)? *Recommendation: one; a count can come later as data.*
3. **Earlier kills**: should kills from before the quest was accepted count for rolled quests? *Recommendation: no (the `counted` baseline); otherwise link 2 is often done the moment it is accepted. Authored quests keep today's rule.*
4. **Rerolling**: a declined offer rolls again once time has passed (recommended); or never rerolls until done; or rerolls on every ask? *Recommendation: after time passes: no lock-out, no free reroll.*
5. **The tribes**: the seven of §6.3? Should the goblins include the ogre and the troll (their faction does), and should vermin be one (rats make 100 quick)? *Recommendation: the seven as listed, ogre and troll out, vermin in (time is the cost either way).*
6. **Outlaws as a tribe**: make the outlaws (and the four criminals' bands) the eighth tribe? *Recommendation: not now; two types make a thin tribe.*
7. **A criminal's death at any hand** counts (recommended), or only the player's kill (a trap's kill would then strand the chain)? *Recommendation: any hand.*
8. **Link 4's gate**: only link 3 done (recommended), or also a level or Leofric's "The barrow's heart", as Wulfrun gates Fyrdelf? *Recommendation: link 3 only; the lair's danger speaks for itself.*
9. **Ranks**: titles only (recommended), or perks (a discount at the town's shops, a free service from Hild)? *Recommendation: titles; perks belong to talents.*
10. **After the chain**: should links 1 and 3 become repeatable contracts once link 4 is done (gold and XP for time, which the no-anti-farming rule allows)? *Recommendation: yes, as data in G5 (`repeatable = true` with `requires = "guild_wanted"` copies), unless the owner wants new links first.*
11. **Osric's spells**: he fights with a mace until creature spells exist (recommended), or should G4 give creatures real spells through the casters' shared code (`world/caster.rs`)? *Recommendation: the mace; creature magic is its own design.*
12. **Unique items' look**: rare with a fixed name (recommended), or a new "unique" rarity with its own colour in lists? *Recommendation: rare now; a colour is a one-line change later.*
13. **Names**: the Gildhūs, the Wanderers' Guild, Wigmund, the four criminals, the four lairs, and the four items (§4, §8, §9)? *Recommendation: as written; the owner's ear decides.*
14. **The unique items' strength**: four affixes at high tiers plus a power, at item level 12, is above any rare loot rolls at that level. Too much? *Recommendation: keep; they are the chain's prize and there is one per run.*

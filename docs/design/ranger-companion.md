# Warlock — The ranger, the pet class: the animal companion

Design document for the owner's review (#825; first draft 2026-11-02, **revised 2026-11-03 for
the owner's second round**, §18). Written against `origin/main` at 34e2d008, on the **merged**
ally core of the druid's design (`docs/design/druid-and-summons.md` §4 and §14; A1 #836 the core,
A2 #837 summons, A3 #838 the UI, D1 #839 the druid's calls; `docs/dev/allies.md` is its topic
file, `crates/warlock-core/src/world/ally.rs` its code, `assets/data/allies.toml` its numbers).
Nothing of this document is implemented; §17 cuts it into issues. It builds on what is there:
the class tables and their actives (`docs/dev/progression.md` "Class tables", `world/class.rs`,
`world/class_ability.rs`), Focus (`world/focus.rs`), factions and stance (`World::stance`,
`stance_between`, `world/beast_friend.rs`), the creature AI (`world/ai.rs`), making creatures
(`travel::make_creature`, `World::make_ally`), level changes (`World::travel`, `follow_along`),
the abilities registry (`warlock_core::ability`), the kill credit (`fight.rs::dies`), checks
(`check.rs`), skills and their benefits (`docs/dev/skills.md`), the Heal ability (`world/heal.rs`,
`heal::treatable`), the background story (`docs/dev/background.md`), perception
(`perception.rs::Perception::compute`, `sight.rs`), equipment (`docs/dev/equipment.md`), and the
RNG streams (`rng.rs`). Every number is data and hidden in the game; the wiki states it (the
owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-11-02, #825): "Ranger: let's create design issue for this too. I
think this is our pet class." The ranger becomes the pet class: an animal companion as the
class's core (how it is gained, grows, is commanded, dies, is saved; its AI), filling the table's
gaps 18, 25, and 40 (the gaps report, PR #802) and reworking the table, under the coverage policy
(`docs/dev/progression.md`, rules 1–9 of #776), with no new rebindable keys (CLAUDE.md's hard
rules). **The second round** (2026-11-03, PR #832) opened taming to everyone trained in Handle
animal, split the experience share by class, gave every race its own starting beasts with a
rare jackpot and a line in the story, made the companion always come along, gave it a neck slot
and (for the ranger) shared sight, scent, and shared buffs, and kept skill use ungated. §18 has
the answers verbatim and what each changed.

**What the merged core gives, and what is new here.** The core (`Actor::ally`, the side, the
ally arm of the AI, the three orders, following and the retinue, the clock, the kill credit,
Beast-bond, the dumps, the ally mark, `wiki/src/allies.md`) is used as it is. `AllySource::Companion`
exists as a variant **with no data**, reserved for this design. What this document adds is the
companion's own: its data (`CompanionData`), how it is gained (the starting beast by race, Tame),
how it grows, that it always comes along, its nose, its collar, how allies may be tended, the
experience share by class, and the ranger's three class abilities. The witch's familiar (#830)
defers to the same core; nothing here is particular to a hound that a familiar could not reuse.

## Contents

1. Summary
2. Research, tied to the decisions
3. What exists, and what this builds on
4. The merged ally core: what the companion uses, and what this design changes in it
5. Gaining a companion: the starting beast, the story's line, Tame, the name
6. Orders and the companion's mind: the arm as merged, and the nose
7. Following: a companion always comes along
8. Growth: the companion's level is its master's
9. Fighting together: the share by class, friendly fire, death, tending
10. The collar: the neck slot
11. The reworked table: what stays, changes, goes; the three new abilities
12. The data
13. Determinism, saves, replays
14. What the player sees: messages, the status bar, look mode, dumps, the CLI, the wiki
15. Talents and class abilities
16. Rejected ideas
17. Implementation plan (slices)
18. Decisions: the owner's second round, applied
19. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **One companion** | a character has **one companion** at a time: an ally of the merged core (`Actor::ally`) with `source = Companion(CompanionData { name, grade })`, `until = None` (it stays until slain); `World::companion()` is the player's one ally of that source. The ranger has one from the first turn; anyone may tame one. §5 |
| **The starting beast** (the ranger's level 1 **Animal companion**) | drawn as the run begins from the ranger's **race's list** (one to three beasts a race, `data/companions.toml [start]`: a human's hound or boar, an elf's lynx or hound, a dwarf's badger or hound, a goblin's large spider, rat, or bat, …; §5.1), or, at **5 %**, from the **rare list** shared by every race: a wolf, a black hound, a great boar, a panther. The draw **tells a line of the story**: a new background chapter, "The companion", told only to a character with a starting beast (`docs/dev/background.md`; §5.2) |
| **Tame** (everyone's) | a **skill ability** of Handle animal (`Source::Skill`, the abilities menu; `tame:DIR`), for anyone trained in it: an adjacent `tameable` beast that is **not hostile to you or has not noticed you** (`World::unaware`), ten turns beside it, then **one check with no level gate**: Willpower modifier + Handle animal's `tame` benefit + 2 with food offered + the ranger's **small bonus** (Animal companion's `tame = 2`) against 10 + 2 × the beast's level. Made: it is yours, named. Failed: it is roused, and nothing stops you trying again (time is the penalty). Refused with a companion already: a companion is for good. §5.3 |
| **No letting go** | the owner: "pets stay until they are killed." There is no Let go, no Send away (dropped from the core by the owner's Q2 in #828), no stable. §5.4 |
| **Orders, the mind** | the core's three orders (Come to heel, Hold here, Set on) in the Allies group, and its arm, as merged. The companion adds its **nose** to the arm: a beast with `nose` smells a hostile within that many cells in its line of sight, however dark, and goes for it. §6 |
| **Following** | the owner: "it always comes along if it is alive." A companion comes along on every level change **wherever it stands on the level** (not the core's `comes_along` 3, which stays the summons'), held fast or not, holding or not; on the overworld it rides in the retinue. It is never left behind. §7 |
| **Growth** | the companion's level **is its master's level** (`grade_percent` 100: the owner's Q1), kept in `CompanionData::grade`, re-set at every level-up ("Snel grows stronger."). Six hooks read the grade above the type's level with `[growth]`: hit points +2 a level, +1 to hit per 4, +1 damage per 6 and one more die per 12, +1 Defence per 6, Protection by the body formula at its grade, St/Dx/To +1 per 5. Summons do not grow. §8 |
| **The share** | an ally's kill pays its master **50 %** of the master's own kill (`[xp] ally_percent` 50, everyone's); the **pet classes get 100 %**: `ClassDef::ally_share = 100` on the ranger and the druid, read where `ally_percent` is read now. The quest tally, Favoured prey, the named monsters, and the builds are credited in full as today. §9 |
| **Friendly fire** | the core's (the owner's Q3 in #828 and Q7 here): "there is no friendly fire protection in the game." §9 |
| **Death** | "Snel dies." A corpse (a companion is no summon), its collar in the pile, and nothing else happens to the master. Tame another. §9 |
| **Tending** | the Heal ability treats an **adjacent ally's** bleeding, poison, or disease from the start, for anyone with the skill (`heal:ID:DIR`): "skill use should not be gated." §9 |
| **The collar** | the companion has a **neck slot**: **Give** (the Allies group, a stack of yours that fits the neck; `give:LETTER`) puts an amulet on it, and it drops the one it wore on the floor. Its magic works on it as on anyone who wears it (`Mods::worn`). §10 |
| **The table** | 1 **Animal companion** (new: the starting beast, the tame bonus), Favoured prey, Trained wind; 2 Stalker's eye; 6 Tracker, Barkskin (stays: the owner's Q11); 12 Rapid shot; **18 Pack tactics** (new: a creature beside both of you is flanked, +2 to hit for both; the companion shares your Favoured prey); **25 Hunter's bond** (new: **shared sight**, the companion's eyes and nose drawn into your view all the time; **shared buffs**: a buff on you is on it and one on it is on you, never a hex); 32 Evasion; **40 Sic 'em** (new, 5 Focus, aimed at a creature you see: the companion is set on it at half again its speed and its next three hits are criticals); 50 Volley; 60 Master hunter. Nothing goes. §11 |
| **Content** | ten new beasts, shared, `wildlife`, cheap (a file each on the beasts' conventions): `hound`, `lynx`, `badger`, `boar`, `large_spider`, `cave_lizard`, `polecat` (the common starts) and `black_hound`, `great_boar`, `panther` (the rares, with the existing `wolf`); `tameable` and `nose` on them and on `wolf`, `bear`, `rat`, `bat`, `beetle`, `deer`, `warg`, `dire_warg`; the boar and lynx in the forest local areas and the Ascmearc's wildlife encounters. §12 |
| **Talents** | **none new**: the `allies` system and its pickable Beast-bond are merged; the ranger's three new class abilities carry the tag where they read the core. §15 |
| **Slices** | C0 the share and the healer's hand (core, small, first), C1 the companion (the starting beasts, the story's chapter, growth, always along, the nose), C2 Tame, C3 the collar, C4 Pack tactics, C5 Hunter's bond, C6 Sic 'em, C7 the wild beasts in the woods, C8 the probe. §17 |

**Key choices, and why**

1. *The companion is the level 1 signature.* "Our pet class" means the class is felt from the first turn as a hunter and a beast; a companion that arrives at 18 would make the ranger an archer for seventeen levels. NetHack's role pet and WoW's level 10 Tame Beast are the model: the pet is there from the start, and the later abilities make it better.
2. *One beast, grown, not a pack.* One companion keeps the fiction clean (a ranger and a hound) and the two classes distinct: the druid calls many for a while, the ranger keeps one for good.
3. *The companion's level is its master's.* A pet that stays level 1 is dead weight by the fourth floor; a second experience curve is bookkeeping the player never sees. One number, read live, a growth table in data (§8).
4. *Taming is a skill, the ranger is better at it.* The owner opened Tame to everyone with Handle animal and gave the ranger a small bonus (Q2, Q10): so Tame is a skill ability (as Pick lock is Open lock's), the check has no gate, only odds, and what makes the ranger the pet class is the beast from turn one, the full share, the bonus, and the three class abilities.
5. *The core is the druid's, used as merged.* Orders, the arm, the retinue, the kill credit: built for a pack, and a hound is a pack of one. What this design changes in the core is small and listed (§4.2): a companion's own following rule, the share by class, the healer's hand, the data on the variant.
6. *No tameness, no feeding, no happiness, no letting go.* Fiddly meters their own designers removed; the owner: a pet stays until it is killed. The companion is loyal; the cost of having it is keeping it alive.

## 2. Research, tied to the decisions

| Source | What it does | Taken | Left, and why |
|---|---|---|---|
| ADOM (pets by Handle Animal and food, the puppy; pets follow when adjacent on stairs; pets level by their own kills) | taming by the skill and by food, for anyone | Tame as everyone's skill ability (§5.3), food as +2 on the check; the retinue crosses the overworld | a pet's own XP; feeding; the adjacency rule (the owner: the pet always comes along) |
| NetHack (the role's starting pet; follow when adjacent; the magic whistle; pets steal from shops) | the starting pet from turn one | the starting beast at level 1 (§5.1); a pet that comes wherever it is (the owner's Q5 does what the whistle did) | tameness decay; shoplifting pets |
| DCSS (allies' orders free of time; allies' kills give XP) | the orders and the kill credit, which the core took | — here | the retreat order (Come to heel does it) |
| Diablo 2 (hirelings: one at a time, level with you, equipped) | one companion that levels with its master and wears gear | the grade (§8); the neck slot (§10) | resurrection for gold |
| WoW hunter (Tame Beast; the pet's level follows the hunter's; Mend Pet; Kill Command, Bestial Wrath, Eyes of the Beast) | the pet's level bound to the master's, an unleash active, seeing through the beast, tending it | `grade_percent` 100, Sic 'em (§11.3), Hunter's bond's shared sight (§11.2), tending by the skill (§9) | the stable, Revive Pet, happiness, the level gate on taming (the owner: odds, no gates) |
| D&D 3.5 ranger (animal companion; tricks by Handle Animal) and 5e Beast Master (the pet attacks only when the ranger spends their action) | Handle Animal as the skill | the `tame` benefit on Handle animal | half progression; a pet that costs the master's action |
| Pokémon, by the owner's own comparison ("this is like releasing a pokemon? let's not") | — | — | releasing a pet: there is no letting go |

## 3. What exists, and what this builds on

| Part | Today (merged) | Used for |
|---|---|---|
| The ally core (#836–#839) | `Actor::ally: Option<Ally { master, source, order, retinue, until }>`, `AllySource::{Summoned { spell }, Companion}` (the latter "its data is that design's"), `World::make_ally`, `allies_of`, `allies_here`, `ally_plan`, `order`, `follow_along`, `allies_lapse`, `ally_mods` (`ally_blow`, `ally_protection`), `data/allies.toml`, `AllyView`, the Allies group, the ally mark, look mode's "Your wolf" | everything the companion is, but for `CompanionData`, how it is gained, growth, its own following rule, the nose, the collar, the share, and the three abilities |
| The class tables | `ClassDef::abilities`; `class::grant` at `start_run` (level 1), `level_up`, `from_state`; `hand_out` applies a granted talent's effects (hit points, stat gifts, `grants_resource`) | Animal companion at 1 (one more arm of `hand_out`: the gift is noted; the beast is made by `start_run`, §5.1); Pack tactics, Hunter's bond, Sic 'em at 18, 25, 40 |
| Class actives | `ClassActive`, `use_class` / `use_class_at`, `Ability::Class`, costs on a resource | Sic 'em (`ClassActive::Sic`, `Aim::Cell`, 5 Focus) |
| Skill abilities | `Ability::PickLock` (Open lock), `Disarm` (Disable device), `Heal` (Heal) under `Source::Skill`, `had_by` the skill trained, `Ability::use_of` | **Tame** (Handle animal), the same way |
| Focus | granted by Rapid shot (12), max 10, regenerates 1 a base turn; Volley spends 6 | Sic 'em's cost |
| Checks | `check::check(rng, modifier, difficulty)`, 2d12, two gameplay draws | the tame check |
| Skills | Handle animal (`kind = "action"`, trained by the ranger 2d4+4, the barbarian, druid, paladin, witch, aristocrat, commoner, merchant) marks on `handle` (an order, an ally's kill; 50 %); benefits are read in one place each | the `tame` benefit, read in `world/tame.rs`; Tame marks `Use::Handle` too |
| Unaware, surprise | `World::unaware(a, b)` (the one definition), `Mind::noticed`, Stalker's eye's `sees_unaware` | who may be tamed |
| Natural Protection by body and level | the data rule (`combat-progression.md` §8; `Body::of`): hide (L − 10) ÷ 8, bony L ÷ 6, armoured L ÷ 4 + 1 | the companion's Protection at its grade |
| Perception | `Perception::compute(state, content, lights)`: the player's line of sight (`compute_fov`), the light on it, `sight`/`visible_cells`/`seen`, then **Blood scent's pass** (creatures with a status in line of sight within a radius, "however dark their cells", marked `Sight::Seen`), wall completion, the seen actors; `sight::sees_by(state, content, lights, &los, eye, p)` is any viewer's eye; `allies_of(state, content) -> Vec<AllyView>` the player's knowledge of their allies | Hunter's bond's shared sight and the nose add a pass in the same place, the same way (§11.2) |
| Buffs | `cast.rs::cast_buff`: `afflict(player, status, severity, Some(player))`; `content/circles.rs` knows which statuses are a buff spell's and which a hex's (every buff status is `seen`) | shared buffs: one hook in `World::afflict` (§11.2) |
| The Heal ability | `heal[:ID]`: `heal::treatable(actor, content)` lists what may be treated on an actor; the player's own today | an adjacent ally's ailments join the list (§9) |
| Equipment | `Actor::equipment: Equipment` on every actor, `Mods::worn(inventory, equipment)` on every actor (a creature's worn amulet works on it); `data/slots.toml` has `neck`; four amulets fit it (`gold_amulet`, `bone_amulet`, `jade_amulet`, `amber_amulet`), with the affixes | the collar (§10) |
| The kill credit | `fight.rs::dies`: a kill by an ally of the player's is credited to the player at `[xp] ally_percent` (`earn_kill_at`), with `tally_kill`, `judge_kill`, `gain_resource(Build::Kill)`, a Handle animal mark | the share by class (§9) |
| The background story | chapters in `data/background/NN-<chapter>.toml`, `Witness`, `tell` per chapter on its own stream, `only` lines force a chapter, every chapter has a plain line; `World::tell_background` runs in `start_run` **after** `take_creation` and the class's level 1 grants | the companion's chapter (§5.2) |
| The local areas' deer, the encounters' wildlife, D1's wolf and bear | `deer`, `wolf` (4), `bear` (12), faction `wildlife`, tag `beast`; `rat`, `bat`, `beetle` (vermin) | the beasts to tame; the new ones join them |

## 4. The merged ally core: what the companion uses, and what this design changes in it

### 4.1 Used as it is

| Core piece (`docs/dev/allies.md`) | The companion's use |
|---|---|
| `Actor::ally`, `Ally { master, source, order, retinue, until }` | the companion is an ally with `master = player`, `source = Companion(CompanionData)`, `until = None`, saved and hashed with the actor as every ally is |
| the side: made with the player's faction, `disturbs_rest`'s exception, the quick leave, the spawn cap, coarse turns, displacement | a hound beside a sleeping ranger disturbs nothing; monsters hunt it as they hunt the player; a Beast-friend's ranger has a hound that leaves rats be |
| the arm (`ally_plan`): target by the master's stance, fight, the order, the leash; doors by `opens_doors`, traps and water as any creature's | the companion's whole mind, plus the nose (§6) |
| the three orders, `Source::Ally`, `Action::Order`, no time, a voice's noise, the Handle animal mark, `Unavailable::NoAllies` | the companion obeys them as the one member of the pack; there is no fourth order (§5.4) |
| the retinue on the overworld, stepping out, the encounter placement | the companion crosses the map in the retinue and comes into an encounter's area beside the player |
| the kill credit: `earn_kill_at`, `tally_kill`, Favoured prey, `judge_kill`, the builds | the hunt is one; only the percent changes (§9) |
| friendly fire | the ranger's bursts, bolts, and a Volley's missiles hurt the companion in their way |
| death leaves a corpse (a summon leaves nothing: `creature_dies` returns early only for `Summoned`) | "Snel dies.", a hound corpse, its collar in the pile |
| `World::ally_mods(id)` read in `ally_blow` and `ally_protection` | the growth hooks sit beside those reads (§8) |
| `AllyView`, the `allies` JSON, the header's `allies N`, the `actors` layer's `; ally of #0 (…)`, the ally mark, look mode's "Your wolf" | the companion's rows and words are added to them (§14) |
| the `allies` system, the pickable Beast-bond (Handle animal 20) | the ranger's class abilities carry the tag; this document mints no pickable (§15) |

### 4.2 What this design changes in the core (small, listed, each in its slice)

| # | Change | Where | Slice |
|---|---|---|---|
| A | `AllySource::Companion` → `Companion(CompanionData { name: String, grade: u8 })` | `world/ally.rs` (the variant), `AllySource::id` unchanged (`"companion"`) | C1 |
| B | **A companion always comes along**: `follow_along` takes every ally of the player's on `from` whose source is `Companion`, wherever it stands and held fast or not, beside the `comes_along` rule for the rest | one filter in `follow_along` | C1 |
| C | **The share by class**: `[xp] ally_percent` becomes **50** (everyone's); `ClassDef::ally_share: Option<u8>` (`ally_share = 100` on the ranger and the druid) is read first in `fight.rs::dies` where `ally_percent` is read now. The druid's hash is unchanged (100 before, 100 after) | `allies.toml`, `content/class.rs`, one line in `dies` | C0 |
| D | **The healer's hand**: `heal::treatable` candidates include an adjacent ally's; `heal:ID:DIR` | `heal.rs`, `world/heal.rs`, `Aim::Ailment`'s list | C0 |
| E | **Give** joins the Allies group (a fourth row, not an order: it takes a wear's time and is aimed at a stack) | `ability.rs`, `world/collar.rs` | C3 |

Nothing else of the core is touched: `heel`, `leash`, `stay.reach`, `hunt.reach`, `comes_along` (the summons'), `most`, the orders' noise, and the clock stay as they are.

### 4.3 The design's own

| The core left to #825 (druid design §4.9) | This document says |
|---|---|
| what `Companion` carries | `CompanionData { name, grade }`: the name (§5.5) and the growth level (§8); no hunger (§16); the bond is the variant itself: at most one ally of source `Companion`, `World::companion()` |
| a companion's own behaviours | the nose (§6): a beast with `nose` finds hostiles it cannot see |
| what Send away does to a companion | there is no Send away (the owner's Q2 in #828) and no Let go (Q3 here) |
| whether a companion left behind does more than wait | it is never left behind (Q5) |
| the ranger's class abilities | Pack tactics and Sic 'em on hooks of their own, Hunter's bond on perception's pass and `afflict` |
| how a companion is gained and grows | §5, §8 |
| the companion's own rows and words | §14 |

A witch's familiar (#830) would be another `AllySource` variant with data of its own; nothing in
`CompanionData`, Tame, growth, the collar, or the following rule assumes a hound.

## 5. Gaining a companion: the starting beast, the story's line, Tame, the name

### 5.1 The starting beast (the ranger's Animal companion)

The class ability **Animal companion** (`animal_companion`, tree `class`, `granted = true`,
effects `[{ kind = "companion" }, { kind = "tame", bonus = 2 }]`, systems `allies`) is given at
`start_run` as every level 1 entry is. Its `companion` effect is noted by `class::hand_out`
(`TalentMods::companion`), and **`World::start_run` makes the beast** once the level is laid and
the player stands on it, after `take_creation` and **before `tell_background`** (so the story
knows it, §5.2): `World::bring_companion()`:

1. **The draw**, on the stream `companion/<race>,<gender>,<class>` (`Rng::stream`; two draws,
   nothing else moves): `below(1000)` against `[start] rare_per_mille` (50: the owner's "like 5%")
   picks the **rare list** (`[start] rare`, shared by every race) or the **race's list**
   (`[start.by_race] <race>`); then `below(list.len())` picks the beast.
2. **The actor**: `make_ally(creature, free_cell_near(player), Companion(CompanionData { name,
   grade: the player's level }), None)`, the core's way (the player's faction, no gear).
3. **The name** (§5.5) and the line: "Snel, your hound, pads at your heel."

On `from_state` nothing is made (a saved run has its own, or lost it); with `World::companion()`
already `Some`, nothing is made. So every ranger run has a beast from the first turn, the
scenarios' rangers too (the showcase's Osric on the overworld, whose beast is in the retinue, and
Maren in the forest glade, whose beast stands beside her: their replays and `sky.toml`'s forest
golden change once, deliberately, §13).

**The racial lists** (`data/companions.toml [start.by_race]`; every race of `data/races.toml` has
one to three; the loader refuses a race without a list, a beast not `tameable`, or one of level
above `[start] most_level` 2 outside the rare list):

| Race | Beasts | The fiction |
|---|---|---|
| human | `hound`, `boar` | the farmstead's dog, the forest's pig |
| elf | `lynx`, `hound` | the wood's cat |
| half-elf | `hound`, `lynx`, `polecat` | of two peoples |
| dwarf | `badger`, `hound` | the deep halls' digger |
| halfling | `hound`, `polecat` | the burrow's ratter |
| gnome | `badger`, `polecat` | the burrow's |
| half-orc | `boar`, `hound` | the raiders' pig |
| goblin | `large_spider`, `rat`, `bat` | the warren's |
| drow | `large_spider`, `bat` | the under-dark's |
| kobold | `cave_lizard`, `large_spider`, `rat` | the warren's and the caves' |
| lizardfolk | `cave_lizard`, `boar` | the fen's |
| duergar | `beetle`, `large_spider`, `badger` | the deepest dark's |
| ratfolk | `rat`, `polecat`, `hound` | the sewers' |

**The rare list** (`[start] rare`, four, 5 % for every race): `wolf` (4, exists), `black_hound`
(5, new: the great black dog of the lanes, with the best nose), `great_boar` (6, new), `panther`
(6, new: the quietest feet). A rare is a jackpot the player sees at once (the story's line names
it, §5.2) and never a bear (12): a level 12 beast at a level 1 ranger's side would carry the
first five floors alone. The beasts' numbers: §12.2.

### 5.2 The story's line

The owner: "the starting beast choice should drop a line to the characters story." A new shared
chapter, **The companion** (`assets/data/background/08-companion.toml`, `paragraph = 4`, after
Leaving; the Ascmearc's road stays the campaign's last), told **only to a character with a
starting beast**, through two small additions to the background loader (`docs/dev/background.md`):

- **a chapter-level `when`** (`[chapter] when = { companion = true }`, `Chapter::when`): a chapter
  whose `when` does not fit the witness is **skipped before its stream is drawn** (its draws do
  not happen, so a story without a companion hashes as before; a chapter's stream is its own
  anyway, so no other chapter moves);
- **a `companion` condition** on lines (`when = { companion = ["hound"] }`, creature ids checked at
  load) and `Witness::companion: Option<CreatureId>`, filled by `tell_background` from
  `World::companion()`'s type (made before the story: §5.1).

Every beast of the lists has one **`only` line** (common, worth 0, no effects: the beast itself is
Animal companion's gift) with `when = { companion = ["<id>"] }`, so it is forced whatever tier
the chapter drew ("if a line with `only` fits, the fitting `only` lines"); the chapter's required
plain line is the net for a beast without a line ("A beast of the wild had taken to you, and came
when you left."), and a load check in `content/companion.rs` says every listed beast has its
line. The texts, in the pools' voice (second person, past tense, no digit, no stat, no word
about the draw; `texts_hide_the_numbers` holds):

| Beast | Line |
|---|---|
| hound | "A hound followed you home one winter and would not be sent back, and it has not left your heel since." |
| boar | "You raised a boar from a piglet the sounder had abandoned, and it grew into something the dogs give room to." |
| lynx | "A lynx kitten, found mewling under a dead mother, learned to hunt at your side and never learned to leave." |
| badger | "A badger dug under the wall of {home} and stayed, and in time it was yours, as much as a badger is anyone's." |
| polecat | "A polecat you took from a trapper's sack rides your shoulder and kills what moves in the straw." |
| large_spider | "A spider the size of a dog hatched in the dark corner you slept in, and grew up thinking you its kin." |
| rat | "A rat too clever for the traps took your bread for a year, then your side, and keeps both." |
| bat | "A bat that roosted over your bed came to your whistle, and now comes to your hand." |
| cave_lizard | "A cave lizard warmed itself on your hearth-stone so often that it forgot it was wild." |
| beetle | "A beetle of the deep roads, armoured like a shield, took to following your lantern, and then to following you." |
| wolf (rare) | "A wolf cub, the last of a litter the hunters took, grew up at your fire, and the village never quite forgave you for it." |
| black_hound (rare) | "A great black dog came out of the fog one night and sat at your door, and the old folk crossed themselves; it is still there." |
| great_boar (rare) | "The boar that killed two hounds and a huntsman let you walk up to it, nobody knows why, and has let nobody else near since." |
| panther (rare) | "A black cat the length of a man came down from the hills in a hard winter and chose your hearth over every other; it moves without a sound." |

The chapter's wiki table is generated as the others' are (`background-companion.md`); the owner's
review of the texts comes after the wave, as for the pools.

### 5.3 Tame (everyone's skill ability)

`Ability::Tame`, `Source::Skill` ("Handle animal"), `Aim::Direction`, `tame:DIR`; had by anyone
with Handle animal trained (`had_by`, as Pick lock is Open lock's). The one place is
`world/tame.rs`.

| Step | Rule |
|---|---|
| Who | an actor adjacent in that direction whose type is `tameable = true`; the player has no companion (`World::companion()` is `None`, else "You already have a companion."); the beast is **not hostile to you** (`World::stance(player, it)` neutral: wildlife, or an animal a Beast-friend calmed) **or has not noticed you** (`World::unaware(it, player)`). Else refused, no time: "The warg has seen you: it will not be tamed now." **No level gate** (the owner's Q10): a level 1 commoner may try a bear |
| The channel | `[tame] turns` (10) actions beside it, as a dig's turn loop (`dig_turn`): each a normal action's price; interrupted when the player is hurt, moves, or the beast moves away, with nothing lost but the time |
| Food | if the player carries food, one item of the first food stack is eaten by the beast (`food` + 2 on the check; "You hold out some dried meat."); none, no bonus |
| The check | `check::check(gameplay, Willpower modifier + Handle animal's `tame` benefit + food + TalentMods::tame (the ranger's 2), difficulty = [tame] difficulty (10) + per_level (2) × the beast's level)`. The benefit is the skill's curve, `skills.toml` `handle_animal`: `benefit = [{ effect = "tame", curve = [[0, 0], [100, 10], [200, 20]] }]` (so the skill, not the level, is what the odds answer to: the owner's "success chance per skill"). Made: it is yours. Failed: "The boar snaps at you and backs away." — its `Mind::noticed` is set (it knows you now), nothing more: try again when it is calm or unaware again (no cooldown: time is the penalty). A natural 2 fails and a 24 succeeds, as every check: so the odds are never none and never sure |
| Marks | `Use::Handle` (the skill's one mark), made or failed |

The odds, so the owner sees them (2d12 + the modifier against the difficulty, `combat::hits`): a
level 1 ranger (Willpower 12, +1; Handle animal about 10, +1; food +2; the bonus +2: **+6**)
against a hound (1, difficulty 12): about nine in ten; against a wolf (4, 18): about three in
five; a commoner with the skill at 10 and food (+3) against a wolf: about three in eight; anyone
at level 1 against a bear (12, 34): the natural 24 alone, 1 in 144, ten turns a try; a master of
the skill (80, +8; Willpower 16, +3; food; the ranger's bonus: +15) against the bear: about one
in seven. The wiki prints the exact table, and the probe (C8) says whether the wolf comes too
easily.

A beast tamed keeps its actor (hit points, statuses, its place), is given the player's faction as
its `faction_override` (as `make_ally` gives a made one) and `ally` with `source = Companion
(CompanionData { name, grade: the player's level })` (its hit points rise with its maximum), the
name, and `Mind` cleared. "The wolf comes to your hand. You call it Græg."

### 5.4 No letting go

The owner: "pets stay until they are killed." The core has no Send away (the owner's Q2 in
#828), and this design adds no Let go: a companion is for good, and a second is had when the
first is dead. Tame with a companion is refused ("You already have a companion."). A companion
on another level can no longer happen (§7), so nothing is ever lost but by death.

### 5.5 The name

Drawn from `data/companions.toml` `names` (Old English, the world's register for old things:
Snel, Brūn, Græg, Blæc, Hwīt, Lēof, Hræfn, Wulf, Fæger, Beald, Hraþe, Fugol; the owner's #378
review may rename) on the stream `companion/name/<actor id>` (one draw), so no other roll moves.
`World::called` says the name without an article, as a unique character's (`docs/dev/npcs.md`):
"Snel bites the kobold." The core's "Your wolf …" (`fight::End::Yours`, `pack_words`) says the
name for a companion; a dead companion's corpse is "a hound corpse" (the type's), not the name.

## 6. Orders and the companion's mind: the arm as merged, and the nose

The orders are the core's three (`docs/dev/allies.md` "Orders"), the mind the core's arm, the
constants `data/allies.toml`'s. A hound at heel keeps within 2 cells; set on, it goes and fights
what is within 3 of the cell, then looks back and heels; holding, it fights what comes within 1.

**The nose** (the owner's Q6: "give 'scent' ability to animals that you expect to have it. it
reveals hidden foes within 6 squares"): `CreatureDef::nose: Option<i32>` (`nose = 6` on the
hound, badger, boar, polecat, wolf, bear, warg, dire warg, rat; 10 on the black hound; none on
the cats, the spider, the lizard, the bat, the beetle, the deer). Two reads, one field:

1. **the arm's target** (step 1 of `ally_plan`): a hostile of its master's within `nose` cells
   (Chebyshev) in the beast's **line of sight**, whatever the light, counts as seen
   (`nearest_in_sight`'s predicate gains the nose: the creature's own line of sight is already
   computed there; the light check is skipped within the nose). A hound in the dark goes for the
   rat it cannot see;
2. **the player's view**, through Hunter's bond (§11.2).

Line of sight, not through walls: a smell is honest about what is around the corner only if the
corner is open (the owner may want it through doors: §19 Q3). A disguised mimic is a thing, not a
beast, and smells of nothing (`hides`, never a target, as today).

Everything a creature's turn does, the companion's does: footsteps, statuses, Surprised, the
coarse turns beside a resting ranger. The companion adds no draw to any stream.

## 7. Following: a companion always comes along

The owner: "the pet is important. it always comes along if it is alive. we do not want to make
this tedium." So the core's `comes_along` (3, the owner's Q5 in #828) stays the **summons'**
rule, and a companion has its own (change B, §4.2): in `follow_along`, every ally of the
player's on the level left whose source is `Companion` comes along **wherever it stands**, held
fast or not (a webbed hound is pulled free by the move), holding or not (its order becomes
*Follow*, as the core resets every follower's). It arrives beside the player (`free_cell_near`),
"Snel comes with you."; onto the overworld it joins the **retinue** (carried, never drawn, no
turns) and steps out at the next level, "Snel follows you in."; into an encounter's area it
stands beside the player before the party is placed (the core). A companion is **never left
behind** and never on another level than its master (or the retinue): `World::companion()` is
always at hand, so no whistle, no "away" state, and no return trip for a forgotten dog. A
companion whose master falls through a floor or is pulled through a collapse comes the same way:
`follow_along` is called by every `travel`.

The one thing that keeps a companion from a level is death.

## 8. Growth: the companion's level is its master's

`CompanionData::grade` is the companion's own level: `grade_percent` (100: the owner's Q1) of its
master's level, at least the type's level, set when the companion is gained and at every
`level_up` (one line in `World::level_up`: "Snel grows stronger." when it rises). Summons have no
grade (their power is the spell's ladder), so the hooks below read nothing for them: they sit
beside the core's `ally_mods` reads and cost an ally without a grade nothing. The rule is the
companion's, whoever its master is (a commoner's tamed hound grows with the commoner: §19 Q1).

| Hook | Where | Rule (`data/companions.toml [growth]`; `above` = grade − the type's level, never below 0) |
|---|---|---|
| hit points | `Actor::max_hp` | `+ hp_per_level × above` (2); the current hit points rise with the maximum |
| to hit | the attack profile, beside `ally_blow` | `+ above ÷ to_hit_every` (4) |
| damage | the attack profile | `+ above ÷ damage_bonus_every` (6) flat, and `+ above ÷ damage_dice_every` (12) more dice of the attack's die |
| Defence | `Actor::defence` | `+ above ÷ defence_every` (6) |
| Protection | `World::protection_of`, beside `ally_protection` | the body formula at the grade (hide: (grade − 10) ÷ 8, never below the type's own) |
| stats | `Actor::stats` reads for checks and burden | St, Dx, To `+ above ÷ stat_every` (5) |

Speed, senses (the eyes, the nose), size, and resistances are the type's. So the hound (level 1,
1d4 bite, 12 hit points) beside a level 30 ranger is a level 30 hound: 70 hit points, +7 to hit,
3d4+4, Defence +4, Protection 2; a wolf (4, 16 hit points, 1d6) tamed at 4 and carried to 30 has
68, +6, 3d6+4; a panther (6, 24, 1d8) 72, +6, 3d8+4. The wiki prints this table for every
tameable beast at the cadence levels, and `cargo test balance_probe` gains a row: the ranger and
their hound against the monsters of their level (`docs/balance.md`, C8).

Nothing of the companion's is an XP curve: no kills counted for it, no level of its own to show.

## 9. Fighting together: the share by class, friendly fire, death, tending

| Topic | Rule |
|---|---|
| **An ally's kill: the share** (the owner's Q2) | `fight.rs::dies` credits the master as today; the **percent** is `ClassDef::ally_share` when the master's class has one (`ally_share = 100` on the ranger and the druid: "pet classes get 100% of xp share"), else `[xp] ally_percent`, now **50** ("others get 50%"). The percent scales `earn_kill_at` only: **`tally_kill` (the quest log, Favoured prey's record), the named monsters, `judge_kill`, and the builds are credited in full**, as today ("Credit is still shared fully for quests"). A summon's kill and a companion's kill share alike: the rule is the master's class, not the ally's source |
| **Pack tactics (18)** | §11.1 |
| **Friendly fire** (the owner's Q7: "there is no friendly fire protection in the game") | the core's: a burst or bolt the ranger casts or reads hurts the companion in its radius or line as it hurts anything; a shot or thrown thing strikes the first creature in its path, the companion too; a Volley's three shots the same. "Snel yelps." Walking into it swaps places (never a blow); `fire:X,Y` at its cell is allowed, with no dialog |
| **The companion struck** | as any creature: hit points, statuses (bleeding, poisoned), Surprised; it regenerates by its Toughness as creatures do, and rests when the player rests |
| **Tending** (the owner's Q12: "skill use should not be gated … let the heal skill heal from the start") | the **Heal ability** treats an **adjacent ally's** ailment for anyone with the skill, from level 1: `heal::treatable` lists the player's own first, then each adjacent ally's (`Ailment` gains the actor it is on); `Aim::Ailment`'s list says "Snel's bleeding"; `heal:bleeding` the player's, `heal:bleeding:E` the ally's that way; the player's Heal skill rolls, the time and the marks as today. A druid tends their wolf the same way (an ally is an ally) |
| **Death** (the owner's Q8: "only the loss of pet") | "Snel dies." A corpse (a companion is no summon) with its collar in the pile, and nothing else: `World::companion()` is `None`; no grief status, no XP lost, no cooldown. Tame another when you find one: the forest glades, the hills, the dungeon's vermin (a rat companion is a rat companion). Time is the penalty |
| **Traps** | the core's: the companion springs a trap as any creature does, known to the player or not. A ranger who leads a hound over a trapped corridor holds it first |
| **Encounters** | the core's placement: the companion steps out of the retinue beside the player before the party is placed; "Lie in wait" sets `Mind::missed` on the party, not on it |
| **Stealth** | the companion's footsteps are heard as any creature's (a panther's 10, a boar's 60); Hold here lets a ranger scout alone and call it (Come to heel) when the fight starts |

## 10. The collar: the neck slot

The owner (Q9): "let's give the pet a neck slot. the pet will equip any neck slot item given to it,
and drop any previous on the floor." Every actor already has `equipment` by slot and
`Mods::worn` over it, and `data/slots.toml` has `neck`; what is new is the hand-over.

| Piece | Rule |
|---|---|
| **Give** | `Ability::Give`, the Allies group (`Source::Ally`, after Set on; not an order: it takes time), `Aim::Stack` (`Ability::action_on`: the stacks the player carries whose object's `slot` is `neck`), `give:LETTER`. Refused, no time: with no companion adjacent ("Snel is not beside you."), with a stack that does not fit the neck ("Snel has no use for that."). The one place is `world/collar.rs` |
| **The hand-over** | one item of the stack leaves the player's inventory and is put in the companion's `neck` slot (`Equipment`, the one slot a companion uses); the amulet it wore, if any, is **dropped on the companion's cell** (a pile, the ordinary rules). Priced as a `wear` (`actions.toml`). "You fasten the amber amulet about Snel's neck." / "Snel's jade amulet falls to the floor." |
| **What it does** | the amulet's magic works on the companion as on any wearer: `Mods::worn` (natural armour, resistances), `worn_vision` (an amulet that gives eyes gives the hound them), the regeneration affixes. Identification is the player's: wearing it on the beast teaches nothing. There is no taking it back but by giving another (the old one falls to the floor) or by its death: the owner's rule, and no "take off" is added |
| **Shown** | look mode: "Snel, your hound (unhurt), wearing an amber amulet." (worn things are seen, as any creature's); the `companion` dump and the `actors` layer list it; the status bar line does not |
| **Death** | the collar is in the corpse's pile (the ordinary rules: a companion is no summon) |
| **Other slots** | none: a beast wears nothing else (Give refuses), and `rolls_gear = false` stays |

## 11. The reworked table: what stays, changes, goes; the three new abilities

| Lv | Entry | Status | Notes |
|---:|---|---|---|
| 1 | **Animal companion** | **new** | the starting beast by race, the tame bonus; §5 |
| 1 | Favoured prey | stays | the companion's kills feed its record |
| 1 | Trained wind | stays | the martial classes' stamina |
| 2 | Stalker's eye | stays | also the tamer's tool: who has not noticed you may be tamed |
| 6 | Tracker | stays | |
| 6 | Barkskin | stays | the owner's Q11: "stays"; the druid has it at 3 too |
| 12 | Rapid shot | stays | grants Focus, which Sic 'em spends |
| 18 | **Pack tactics** | **new** | §11.1 |
| 25 | **Hunter's bond** | **new** | §11.2 (not "Beast-bond", the merged pickable) |
| 32 | Evasion | stays | |
| 40 | **Sic 'em** | **new** | §11.3 |
| 50 | Volley | stays | |
| 60 | Master hunter | stays | its Favoured prey half reaches the companion through Pack tactics |

Nothing goes. The gaps report's "Direction: stealth" for 40 is not followed: the owner's
direction for the class is the pet.

### 11.1 Pack tactics (18, passive)

`pack_tactics`, effects `[{ kind = "pack_tactics", to_hit = 2 }, { kind = "shares_favoured_prey" }]`,
systems `combat`, `allies`. Read in `World::resolve` → the attack profile, one hook (`world/pack.rs`):

- a creature adjacent to **both** the ranger and their companion is **flanked**: the ranger's
  melee and the companion's blows on it get `+to_hit` (2). Missiles not (flanking is about
  footing);
- the companion's blows carry the ranger's **Favoured prey** bonus against the ranger's favoured
  type (+1 to hit, +2 damage; +2/+4 with Master hunter at 60): `shares_favoured_prey` reads the
  ranger's `TalentMods::favoured_prey` for the companion's profile when the target is the type.

It is no clone of Beast-bond (`ally_mod.to_hit` on every blow): flanking is conditional and
two-sided, and the alike check compares effect kinds, which differ.

### 11.2 Hunter's bond (25, passive)

`hunters_bond`, effects `[{ kind = "shared_sight" }, { kind = "shared_buffs" }]`, systems `vision`,
`allies`, `status`. Two things the bond does, each one hook:

**Shared sight** (the owner's Q6: "draw los from both characters all the time after gaining the
ability"). In `Perception::compute`, after the player's own cells are settled and **in the place
of Blood scent's pass** (the pattern it copies: cells marked `Sight::Seen` and pushed to
`visible_cells` from a second viewpoint), with `talents.shared_sight` and the player's companion
on the level (not the retinue), **a second viewpoint**:

1. the companion's line of sight: `LineOfSight::compute(companion.pos, sight_radius(state,
   content, its sight), is_opaque)`;
2. **its eyes**: for every transparent cell in it, `sight::sees_by(state, content, lights, &los,
   &its VisionProfile, p)` (its own eyes: a hound's low-light, a panther's darkvision; the light
   as it reaches that cell); seen, the cell is `Sight::Seen` and in `visible_cells` if it was
   `Hidden`, and in `seen` (so wall completion runs for its walls too);
3. **its nose** (§6): each hostile creature within `nose` cells of the companion in its line of
   sight, whatever the light: its cell `Sight::Seen` ("it reveals hidden foes within 6 squares");
4. the light grid covers the new cells (a second `lit_in_sight` over the companion's line of
   sight for the same sources, perceived by the player's eye: what decides *seeing* is the
   companion's eye; the colours the player is shown are their own eye's).

**Why nothing leaks**: the snapshot is the one place ground truth becomes what the player knows,
and it already reads every actor and every light there; the companion's pass adds cells to the
same `sight`, `visible_cells`, and seen actors, so every reader (the renderer, look mode,
`is_visible`, the dumps, memory, Set on's "a cell you see", the targeting mode) reads the one
snapshot and nothing else, as today. What the pass does not do: it finds no hidden trap and no
secret door (the search rules are the player's, and eyes are not a search), reveals no disguised
mimic (a thing, §6), and sees nothing the companion's eyes and nose would not (a cell beyond its
sight radius stays dark). A dazzled or blinded player keeps the companion's view. The dumps' `fov`
and `view` layers show it as any seen cell; the `light` layer the perceived light.

**Shared buffs** (the owner's Q11: "buffs cast on ranger or the pet affect the other. but not
hexes"). One hook in `World::afflict`: when `talents.shared_buffs` and the status is a **buff**
(`content/circles.rs` knows the buff spells' statuses from the hex spells'; cooldowns, harms,
Winded, Raging, a mimic's disguise are none) and the target is the player with a companion on the
level or in the retinue, the companion is afflicted with the **same status and severity** (its
own instance: it eases by its own clock, as every status does); and the other way, a buff landing
on the companion lands on the player. A **hex never** crosses (`SpellPurpose::Hex`'s statuses are
excluded by kind, not by list). The sources are whatever lays a buff on the ranger: a spell
(`cast_buff`), a scroll of one, a potion, a boon (`afflict` is the one door). Today nothing buffs
a beast but this hook, so the reverse arm has no source yet; it is written once, for the day a
creature or an item does. "Barkskin hardens Snel's hide too."

Shared sight is the capability players remember of a pet class (WoW's Eyes of the Beast). It is a
change to what the character *can do* (scout with the hound), not a readout, so rule 8 is not
touched; it reads `vision`, which has its talents. Shared buffs read `status`, which has its own.

### 11.3 Sic 'em (40, active)

`sic`, `ClassActive::Sic`, `Aim::Cell` (a creature you see, within `range` 10), `use:sic:X,Y`,
`resource = { id = "focus", cost = 5 }`, effects `[{ kind = "sic", blows = 3, speed_percent = 50 }]`,
systems `combat`, `resources`, `allies`. Refused with no companion on the level, a cell with no
creature the player sees, or the target not hostile to the player; the cost is spent after the
checks. It does two things (`world/sic.rs`):

1. the companion takes `Hunt { at: the target's cell }` and keeps hunting that creature while it
   sees it (the hunt cell follows the target: one line in the arm for a hunt begun by Sic 'em);
2. the companion gets the status **Savage** (`data/status.toml`: `savage`, degrees 3, no tick,
   eased by hits: each blow it lands spends a degree; eases 60 s a degree otherwise;
   `speed_percent` +50 while it lasts): each of its next three blows that hit is a **critical
   without the confirmation roll** (`combat::strike_with`: the status's `auto_critical`, read where
   the threat is rolled). Savage is a buff on the companion in the fiction but **not** a buff
   spell's status, so shared buffs never carry it to the ranger (and the circles' table does not
   list it).

The cost is 5 of 10 Focus (Volley 6, Rapid shot 2 a shot), so a ranger at 40 chooses: a volley,
or the hound's throat-hold, not both in one turn.

## 12. The data

### 12.1 `data/companions.toml` (shared, one file; parsed by `content/companion.rs`)

The allies' constants are `data/allies.toml`; this file holds only the companion's own.

```toml
# The animal companion (spec §8 "Class abilities", "Allies"); the allies' rules are allies.toml.
[start]
rare_per_mille = 50          # the jackpot: a beast of `rare` instead of the race's list
rare = ["wolf", "black_hound", "great_boar", "panther"]
most_level = 2               # a race's list holds beasts of at most this level; the rares are free of it

[start.by_race]
human = ["hound", "boar"]
elf = ["lynx", "hound"]
half_elf = ["hound", "lynx", "polecat"]
dwarf = ["badger", "hound"]
halfling = ["hound", "polecat"]
gnome = ["badger", "polecat"]
half_orc = ["boar", "hound"]
goblin = ["large_spider", "rat", "bat"]
drow = ["large_spider", "bat"]
kobold = ["cave_lizard", "large_spider", "rat"]
lizardfolk = ["cave_lizard", "boar"]
duergar = ["beetle", "large_spider", "badger"]
ratfolk = ["rat", "polecat", "hound"]

[tame]
turns = 10            # actions beside the beast before the check
difficulty = 10       # plus per_level × the beast's level; no level gate
per_level = 2
food = 2              # the bonus for food offered (one item of the first food stack)

[growth]
grade_percent = 100   # of the master's level
hp_per_level = 2
to_hit_every = 4
damage_bonus_every = 6
damage_dice_every = 12
defence_every = 6
stat_every = 5

names = ["Snel", "Brūn", "Græg", "Blæc", "Hwīt", "Lēof", "Hræfn", "Wulf", "Fæger", "Beald", "Hraþe", "Fugol"]
```

Checks at load: every race in `races.toml` has a list of 1 to 3 known `tameable` beasts of level
at most `most_level`; `rare` has 1 to 8 known `tameable` beasts; every listed beast has an `only`
line in the companion chapter (when the content tells stories); every number in range (`turns` 1
to 100, `rare_per_mille` 0 to 1000, `grade_percent` 1 to 300, the `every`s 1 to 60); at least one
name; the `savage` status present with degrees ≥ 3 and no tick. Without the file no companion can
be made (fixtures without it hash as before); without a list for a race, Animal companion makes
nothing and says so in the log (never a panic).

### 12.2 Creature types

| Field | What |
|---|---|
| `tameable = true` (new, `CreatureDef::tameable`, default false) | the type may be tamed; refused at load on a unique, an animated object, a type with `summon`, a type that `opens_doors`, or one of a faction with no `animal = true` |
| `nose = N` (new, `CreatureDef::nose: Option<i32>`, 1 to 20) | it smells hostiles within N cells in its line of sight, however dark (§6); refused on a type that is not `tameable` (only companions read it today; the field is a creature's, so a wild boar's nose may feed the creature AI another day) |

The **new beasts** (shared, `assets/data/creatures/`, faction `wildlife`, tag `beast`,
`alignment = "neutral"`, `tameable`, a `depart` line, `on_eat` as the roster asks, an XP by the
guideline; `docs/dev/creatures/beasts.md` rows and `beast_roster_tests` rows; max HP = Toughness +
`base_hp` as the roster states it). Being neutral they never stray and no population names them
(as the wolf and bear): a starting beast comes with Animal companion, and the others are met in
the woods (C7).

| Type | Level | Speed | Max HP | Attack | Eyes | Footsteps | Nose | Notes |
|---|---:|---:|---:|---|---|---:|---:|---|
| `hound` | 1 | 1200 | 12 | bite 1d4 piercing | low-light | 40 | 6 | the dog; hearing 20 |
| `polecat` | 1 | 1300 | 10 | bite 1d4 piercing | low-light | 20 | 6 | small and quick |
| `lynx` | 2 | 1300 | 13 | claws 1d6 slashing | darkvision | 20 | — | hearing 25 |
| `badger` | 2 | 900 | 19 | bite 1d6 piercing | low-light | 40 | 6 | a thick hide (Defence 1) |
| `boar` | 2 | 1100 | 20 | gore 1d8 piercing | low-light | 60 | 6 | loud |
| `large_spider` | 2 | 1100 | 13 | bite 1d4 piercing, poison as the barrow spider's | darkvision | 20 | — | |
| `cave_lizard` | 2 | 1000 | 16 | bite 1d6 piercing | darkvision | 30 | — | a scaly hide |
| `black_hound` (rare) | 5 | 1250 | 24 | bite 1d8 piercing | darkvision | 40 | 10 | the best nose |
| `great_boar` (rare) | 6 | 1100 | 30 | gore 2d6 piercing | low-light | 70 | 6 | a thick hide (Defence 2) |
| `panther` (rare) | 6 | 1400 | 24 | claws 1d8 slashing | darkvision | 10 | — | the quietest feet |

`tameable` (and a `nose` where it fits) also goes on D1's `wolf` (4; nose 8) and `bear` (12; nose
6) and on `rat` (1; nose 4), `bat` (1), `beetle` (2), `deer` (1), `warg` (7; nose 6),
`dire_warg` (22; nose 6): a level 1 ranger may try a wolf; one who stalks the orcs' kennels, a
warg (it must be unaware). Not the undead hounds, the hell hound, the blink dog (fey, 14), the
basilisk, the marsh things (a crocodile is nobody's dog).

C7 places the `boar` and `lynx` in the forest and hill local areas as the deer are, and as
`"wildlife"` entries in the Ascmearc's encounter tables (a boar by day in the woods; a lynx at
dusk in the deep woods); the `badger` in the hills.

Adding creature types invalidates development saves (`docs/dev/saves.md`: saves are pinned to
the version, and a reordered creature list changes the fingerprint); the PR says so. **No
aliases, no shims** (the owner, #834).

### 12.3 The class files and the allies' share

`assets/data/classes/07-ranger.toml` and `04-druid.toml` gain `ally_share = 100` (`ClassDef::
ally_share: Option<u8>`, 0 to 100); `allies.toml`'s `[xp] ally_percent` becomes 50 with its
comment ("what an ally's kill pays a master whose class has no `ally_share`"). The ranger's
table gains its four rows.

### 12.4 The background chapter

`assets/data/background/08-companion.toml` (§5.2): `[chapter] id = "companion"`, `name = "The
companion"`, `paragraph = 4`, `when = { companion = true }`; fourteen `only` lines and the plain
one. `data/background.toml`'s `[points]` needs nothing (the lines have no effects).

## 13. Determinism, saves, replays

- **Streams**: the starting beast on `companion/<race>,<gender>,<class>` (two draws, `below(1000)`
  and `below(n)`); the name on `companion/name/<actor id>` (one draw); the companion chapter on
  its own `background/<spec>/companion` stream (the tier and the line, as every chapter; skipped
  entirely for a witness without a companion); the tame check on the gameplay stream (two draws,
  only when a check is rolled); growth, the nose, the collar, shared sight, and shared buffs draw
  nothing.
- **Saved**: `CompanionData { name, grade }` inside `Actor::ally` (the core's field); the collar
  in the companion's `equipment` (an actor's, saved as every actor's); nothing on `WorldState`
  (`World::companion()` is derived from the actors). A non-ranger run without a tamed beast saves
  byte for byte as before. Saves are pinned to the version (`docs/dev/saves.md`): the new
  creature types and the variant's data invalidate development saves, said in the PRs; no former
  ids, no shims.
- **Hashes**: a run without a companion hashes as before, but for C0: a kill by a **non-druid's**
  summon now pays 50 % (only the dev `ally:` action and a learned scroll of a call make one; no
  shipped replay has one). Every **ranger** run gains an actor at the start and a story chapter,
  so the ranger runs' replays are re-created (`replay --create`), and the two showcase scenarios
  with a ranger (`world`, Osric; `forest`, Maren) change: Osric's beast rides in the retinue (not
  drawn: `overworld.toml`'s captures may not change at all), Maren's stands beside her
  (`sky.toml`'s forest entry changes once and is updated deliberately in C1's finisher pass,
  "Golden screenshots"). No new golden: the companion is a glyph a core test and a dump can see.
- **Replays**: `tame:DIR`, `give:LETTER`, `heal:ID:DIR`, and `use:sic:X,Y` in the action text
  (CLAUDE.md's list, `action.rs`); the orders are the core's.
- **Invariants** (the tests): a companion's grade is its master's level and rises with it; a
  companion always comes along (held, holding, far off; into the retinue and out); Tame is refused
  with a companion, on a beast that has noticed the player and is hostile, and on a type not
  `tameable`, and never for the beast's level; the check's modifier is the Willpower modifier plus
  the skill's benefit plus the food plus the ranger's bonus; the name is drawn once per
  companion; every race draws only its list or the rare list, at the rare odds over many seeds;
  the story's chapter is told to a ranger and skipped for everyone else (hashes unchanged); a
  ranger's share is 100 and a commoner's 50, the quest tally full for both; Heal treats an
  adjacent ally's bleeding and not a far one's; Give puts the amulet on and drops the old one,
  refuses a non-neck stack, and the amulet's mods reach the companion; shared sight equals the
  companion's `sees_by` and the nose's cells, remembers them, and survives a dazzle; a hex never
  crosses and a buff does both ways; a save round trip keeps `CompanionData` and the collar; a
  replay with a tame, three orders, a level change in and out of the retinue, a Give, a Heal on
  the beast, and a death verifies.

## 14. What the player sees

### 14.1 Messages (words, never a number; the core's with the name in place of "your wolf")

| Moment | Text |
|---|---|
| the start | "Snel, your hound, pads at your heel." / "Blæc, your panther, pads at your heel." |
| taming | "You crouch by the wolf and hold out some dried meat." (the first turn) … "The wolf comes to your hand. You call it Græg." / "The wolf snaps at you and backs away." |
| the stairs | "Snel comes with you." |
| the retinue | "Snel follows you in." |
| growth | "Snel grows stronger." |
| a blow | "Snel bites the kobold." / "The kobold bites Snel." |
| hurt by you | "Snel yelps." |
| the nose (the companion's own turn, seen) | "Snel growls at the dark." (once, when the nose finds what the player does not see) |
| the collar | "You fasten the amber amulet about Snel's neck." / "Snel's jade amulet falls to the floor." |
| tending | "You bind Snel's wound." |
| a shared buff | "Barkskin hardens Snel's hide too." (the spell's own line, with "too") |
| death | "Snel dies." |
| Sic 'em | "Snel goes for the goblin's throat." |

### 14.2 The status bar, look mode, the screens

- **The status bar** gains a short line while the player has a companion: its name and health
  in words ("Snel: unhurt", "Snel: badly hurt"; the `health` words of any seen actor; read from
  `PerceptionView::allies()`), "Snel: with you" in the retinue. A text snapshot
  (`screen_snapshots.rs`).
- **Look mode**: the core's "Your wolf (unhurt)" becomes "Snel, your hound (unhurt), wearing an
  amber amulet."; a tameable wild beast: "a wolf (neutral)" as today, with "It has not noticed
  you." from Stalker's eye as today.
- **The Character screen** (page 1) gains a line under the class: "Companion: Snel, a hound".
- **The story screen** tells the companion's paragraph (page 9).
- **The abilities menu**: the Allies group gains Give ("a thing for its neck"); Tame in the Skill
  group with its reason when refused ("you already have a companion", "no beast there", "it has
  seen you"); Heal's list names an ally's ailment; Sic 'em in the Class group from 40 ("no
  companion here", "no focus").

### 14.3 The CLI and dumps

- `dump --layers companion`: `Snel, a hound (grade 12) at (12, 5): hp 30/30, order follow,
  wearing an amber amulet`; `Snel, a hound (grade 12): with you` in the retinue; `no companion`
  without one; JSON `companion { name, creature, grade, hp, hp_max, order, pos, retinue, wears }`
  (null without one).
- The core's `allies` JSON and the `actors` layer show the companion as any ally, with `named
  Snel, grade 12` appended; the `actors` layer's worn things list the collar as any creature's.
- `abilities` lists Tame `(skill)`, Give `(ally)`, and Sic 'em `(class)`; the orders are the
  core's `(ally)`.
- Action text: §13.

### 14.4 The wiki (the owner's Q13: "allies.md with a section listing starting companions")

`wiki/src/allies.md` loses its "companions that stay for good are still to come" and gains the
section **"Companions"**: what a companion is, that it stays until slain and always comes along,
the collar, growth (the generated table at the cadence levels for every tameable beast,
`xtask/src/wiki/companions.rs`), the share by class, tending; under it **"Starting companions"**:
the generated table of every race's beasts and the rare list with the odds
(`companion-starts.md`), and **"Taming"**: the exact check and the odds table by beast
(`companion-taming.md`). The Classes page's ranger row links it; the creatures table marks
tameable types and the nose; the background page gains the chapter's table; `TERMS.md` gains
"companion", "tame", "grade", "collar", "nose". The spec gains "The animal companion" in
`spec/08i-character.md` under "Class abilities" and the companion's lines in `spec/08b-creatures.md`
"Allies"; `docs/dev/companion.md` is the topic file (`docs/dev/allies.md` the core's, with
changes B–E noted in it).

## 15. Talents and class abilities

**No new system and no new pickable talent.** The system is the merged `allies`, with its
pickable Beast-bond (every ally +1 to hit, +1 Protection: a real pick for a ranger too) and the
druid's class abilities Pack leader and Lord of the wild. The companion extends that system; by
the coverage policy a sub-feature of a system adds no tag and no talent. The ranger's class
abilities carry the tag where they read the core:

| Class ability | Systems | Why it earns its place (rules 1–9) |
|---|---|---|
| Animal companion (1) | `allies` | the class's signature: a new actor on your side from turn one, and the tamer's bonus (rule: it changes what the character can do; no information, no niche) |
| Pack tactics (18) | `combat`, `allies` | conditional, two-sided to-hit on every dungeon floor; no clone of Beast-bond (a different effect kind) |
| Hunter's bond (25) | `vision`, `allies`, `status` | shared sight with the nose, shared buffs: two changes to what the ranger can do; rule 8 untouched (sight is a capability, not a readout) |
| Sic 'em (40) | `combat`, `resources`, `allies` | an active on Focus at the capstone tier that changes a fight (rule 1); no cooldown (Focus and time are the price) |

Nothing else is minted: no "faster taming", no "second companion", no "companion hit points"
talent (the growth table is where those numbers live), no talent for the collar or the nose (a
sub-feature's readouts and gimmicks extend their parent). Tame is a **skill ability**, not a
talent: Handle animal gets its first use without a talent of its own (Beast-bond asks Handle
animal 20, so the skill has a use and a talent). Beast-friend stays the Wayfarer pick and the
druid's level 1: a ranger who wants calm rats takes it; the companion respects it through the
master's stance. Tags match hooks: each reads the system it names.

## 16. Rejected ideas

| Idea | Why not |
|---|---|
| The companion at 18 (the first gap), an archer until then | the owner's "pet class" is felt from turn one, or it is not one |
| A choice of beast at character creation | the owner's: by race, randomised, with a line in the story; a creation step for one class is more UI than a line of text |
| One hound for everyone | the owner: racial beasts, a jackpot; the fiction of a drow with a farm dog was thin anyway |
| Two or more companions, a pack (D2's druid) | that is player summons, the druid's design; one beast keeps the fiction and the classes apart |
| Let go, Send away, a stable, a swap (release a pet to tame a better one) | the owner: "this is like releasing a pokemon? let's not implement this for now. pets stay until they are killed." |
| Tame as the ranger's class active, or a level gate (`over` 3) on the beast | the owner: open to everyone with the skill, no gates, odds only, a small bonus for the ranger |
| A per-source share (companions 100, summons 50) | the owner: by class ("druid and ranger get 100% xp share from pets and summons, others get 50%") |
| A second set of orders for the companion (fetch, guard a cell, track) | the core's three are the pack's and the hound's alike (the owner's Q2 in #828: "helpful without much ordering") |
| Tameness, happiness, feeding (NetHack, classic WoW) | fiddly meters their own designers removed; food is a one-time bonus on taming instead |
| The companion's own XP and levels (NetHack, ADOM) | a second curve the player never sees, and a level 1 pet at depth 10 |
| Growth by swapping the type (hound → great hound) | continuity (its name, its hit points, its collar) is lost |
| Growth for summons too | their power is the spell's ladder: a caster level that both grew the count and the grade would double-count |
| Revive, resurrection for gold (D2, WoW) | permadeath world: tame another |
| A grief status on the ranger when the companion dies | the owner: "only the loss of pet" |
| A companion left behind that waits, and a whistle to call it (the first draft's Hunter's bond) | the owner: it always comes along; nothing to whistle for |
| The companion walks around the traps the player knows | the core decided traps are the player's knowledge, not the wolf's |
| Friendly fire protection for the companion | the owner: "there is no friendly fire protection in the game" |
| Tending the companion only at 25 (the first draft) | the owner: skill use is not gated; the Heal skill tends an ally from the start, for anyone |
| Full gear on the beast (body armour, a helm) | the owner: a neck slot; a hound in a hauberk is a different game |
| A second slot for the collar's own item class ("collars") | the neck slot and the amulets exist; a new item class for one wearer is a registry for nothing |
| Hexes shared with the companion (a symmetric bond) | the owner: "but not hexes" |
| Shared sight only in a stance or while still | the owner: "all the time after gaining the ability" |
| A nose through walls | a smell that finds a foe behind solid rock makes every wall a window; line of sight, and the owner may want doors (§19 Q3) |
| The pet attacks only when the ranger spends an action (5e Beast Master) | the subclass that taught everyone not to; the core's ally acts on its own turn |
| A tameness roll on each order (disobedience) | a disobedient pet is a worse AI, not a mechanic |
| New rebindable keys for the orders or Give, a pets screen | the hard rule: the one abilities menu, fixed letters; the status bar line is the screen |
| The companion picks things up, steals from shops (NetHack) | charming, out of scope; no inventory on the companion but its neck |
| Stealth at 40 (Camouflage's twin underground) | the class's direction is the pet; Sic 'em is the fight changer a capstone tier asks for |
| Cooldowns on Tame or Sic 'em | Focus and time are the prices; no anti-farming limits (the owner) |
| A pickable talent of this document's own for `allies` | Beast-bond is merged; the bar is one, and a count is never a reason to add a talent |
| Mounts | out of scope; a line on #196 |

## 17. Implementation plan

The core is merged; nothing waits on the druid. C0 is small and goes first (it changes a number
the druid's tests pin). C1 is the companion; C2–C6 may run in parallel after it. Each PR runs
the full check; the snapshots are the dumps' insta files, core tests, and text snapshots; no
golden is added, and C1 updates the one the beast appears in.

| Slice | Issue | What it touches | Depends on |
|---|---|---|---|
| **C0 The share and the healer's hand** | an ally's kill pays by the master's class; Heal treats an adjacent ally | `allies.toml` (`ally_percent` 50), `ClassDef::ally_share` and its check, the two class files, one line in `fight.rs::dies`; `heal::treatable` with the actor, `Aim::Ailment`'s list, `heal:ID:DIR`, `world/heal.rs`; `ally_tests` (a druid's 100, a commoner's 50, the tally full), heal tests; `docs/dev/allies.md`, `docs/dev/skills.md`, the wiki's allies page and Heal row, spec | — |
| **C1 The companion** | Animal companion: the starting beasts by race and the rares, the story's chapter, the name, growth, always along, the nose, the table's level 1 | `animal_companion` talent and the ranger's table (level 1), `TalentEffect::{Companion, Tame}`, `hand_out`'s note, `AllySource::Companion(CompanionData)` (change A), `world/companion.rs` (`bring_companion`, the draw, `World::companion`, the level-up grade, `called`), the six growth hooks (§8), `CreatureDef::{tameable, nose}` and their load checks, the ten `creatures/*.toml` and `tameable`/`nose` on the eight existing (`beasts.md` rows, `beast_roster_tests`), `follow_along`'s companion filter (change B), the nose in `ally_plan`'s predicate, `content/companion.rs` (`data/companions.toml`), `background/08-companion.toml` with `Chapter::when`, `When::companion`, `Witness::companion` (`content/background.rs`, `background.rs`, `world/background.rs`, the chapter's wiki table), the `companion` dump layer and JSON, the status bar line, look mode's name, the Character screen's line, the story screen (text snapshots); tests (§13); `docs/dev/companion.md`, `docs/dev/background.md`, `docs/dev/allies.md`; spec; the wiki's Companions and Starting companions sections and the growth table; the ranger replays re-created; `sky.toml`'s forest golden updated in the finisher's pass | C0 |
| **C2 Tame** | everyone's skill ability, the ranger's bonus | `Ability::Tame` (`Source::Skill`), `world/tame.rs` (the channel, the check, the food, the mark), the `tame` benefit on Handle animal (`skills.toml`), `TalentMods::tame` read, `tame:DIR`, the menu row and its reasons, tests, the wiki's Taming table, `docs/dev/skills.md` | C1 |
| **C3 The collar** | the neck slot | `Ability::Give` (`Source::Ally`, `Aim::Stack`), `world/collar.rs`, `give:LETTER`, the pile on the companion's cell, look mode's "wearing", the `companion` dump's `wears`, tests (the mods reach it; the old one on the floor; the pile on death), wiki, `docs/dev/equipment.md` (one line) | C1 |
| **C4 Pack tactics** | level 18 | `pack_tactics` talent, `TalentEffect::{PackTactics, SharesFavouredPrey}`, `world/pack.rs`, the table row, tests, wiki | C1 |
| **C5 Hunter's bond** | level 25 | `hunters_bond` talent, `TalentEffect::{SharedSight, SharedBuffs}`, the second viewpoint in `Perception::compute` (§11.2) with the nose's cells and the light over them, the hook in `World::afflict` with `circles`' buff test, the table row, tests (shared sight equals the companion's `sees_by` plus the nose; memory; dazzle; a buff both ways and a hex never; the retinue), a `--capture` looked at, wiki | C1 |
| **C6 Sic 'em** | level 40 | `sic` talent, `ClassActive::Sic`, `world/sic.rs` (the hunt that follows its target), the `savage` status (`auto_critical`, eased by hits, `speed_percent`) and its read in `strike_with`, the table row, tests, wiki | C1 |
| **C7 The wild beasts** | content | the `boar`, `lynx`, and `badger` in `mapgen/local/` (the glade's boar, the deep wood's lynx, the hills' badger, as the deer) and the Ascmearc's wildlife encounter rows, wiki | C1 |
| **C8 The probe** | the ranger-and-hound row in `balance_probe`; a companion beside the player on `bench-turns`' `delf` row (an ally in the coarse turns) | `balance_probe`, `xtask/src/bench.rs`, `docs/balance.md`, `docs/perf.md` | C1 |

C1 adds saved state (inside the core's field and the creature list) and no action; C2 and C3 add
one action each: the integrator's "about three at once" holds. Each creature-list PR says it
invalidates development saves.

## 18. Decisions: the owner's second round, applied

The owner's answers (PR #832, 2026-11-03), verbatim, against the first draft's §17 questions, and
what each changed in this revision.

| # | The owner | What changed |
|---|---|---|
| Q1 | "100" | `grade_percent = 100` stands (§8); the probe (C8) watches it |
| Q2 | "open to everyone. Let's make pet classes get 100% of xp share, but others get 50%. Ie, druid and ranger get 100% xp share from pets and summons, others get 50%. Credit is still shared fully for quests." | **Tame is a skill ability of Handle animal for anyone trained** (§5.3, `Source::Skill`), no longer the ranger's class active; the ranger keeps a small bonus (Q10). **The share by class**: `[xp] ally_percent` 50 for everyone, `ClassDef::ally_share = 100` on the ranger and the druid, read in `fight.rs::dies`; `tally_kill`, Favoured prey, the named monsters, `judge_kill`, and the builds stay full (§9, change C, slice C0) |
| Q3 | "this is like releasing a pokemon? let's not implement this for now. pets stay until they are killed." | **No Let go**; the first draft's request B (Send away as Let go) is gone with the core's Send away (the owner's Q2 in #828). Tame is refused with a companion (§5.4). The Pokémon row in §2 |
| Q4 | "racial starting beasts per race: hound, lynx, badger, boar, large spider etc. Give each race 1-3 options, and randomize the start beast out of those (the starting beast choice should drop a line to the characters story). There should be some rare jackpot starting beasts too (like 5% chance to roll from a list of 4 rares or something, shared by all races)." | **`[start.by_race]`**, one to three beasts for each of the thirteen races (§5.1), seven new common beasts and three new rares beside the wolf (§12.2), **`rare_per_mille = 50`** and a rare list of four shared by every race; the draw on its own stream before the story; **the story's chapter** "The companion", told only with a starting beast, one `only` line a beast, through a chapter-level `when` and a `companion` condition on the Witness (§5.2, §12.4) |
| Q5 | "the pet is important. it always comes along if it is alive. we do not want to make this tedium." | **A companion always comes along**, wherever it stands, held or holding (§7, change B); the first draft's request A (`comes_along` on `ally_mod`), request D (the default 2), the whistle, the "away" state, and the "left behind" rows are gone |
| Q6 | "yes. shared sight. draw los from both characters all the time after gaining the ability. give 'scent' ability to animals that you expect to have it. it reveals hidden foes within 6 squares." | **Shared sight stays at Hunter's bond (25)**, all the time, as a second viewpoint in `Perception::compute` with the exact pass and why nothing leaks (§11.2); **the nose** (`CreatureDef::nose`, 6 on the beasts that smell, 10 on the black hound, 8 on the wolf) reveals hostiles within that many cells in the beast's line of sight whatever the light, read by the companion's own targeting from the start (§6) and drawn into the player's view through the bond |
| Q7 | "yes, there is no friendly fire protection in the game." | as written (§9) |
| Q8 | "per suggestion, only the loss of pet." | as written (§9): no grief status, no cooldown, no revive |
| Q9 | "let's give the pet a neck slot. teh pet will equip any neck slot item given to it, and drop any previous on the floor." | **The collar** (§10): Give in the Allies group (`Aim::Stack`, `give:LETTER`), the companion's `neck` slot, the old amulet dropped on its cell, the amulet's mods on the beast through `Mods::worn`, the pile on death; slice C3 |
| Q10 | "no level gates, only success chance per skill. ranger should have small bonus compared to everyone else." | **No `over`**: the difficulty is 10 + 2 × the beast's level, the modifier the Willpower modifier plus Handle animal's `tame` benefit (a curve of the skill, +10 at 100) plus food (+2) plus **the ranger's +2** (Animal companion's `tame` effect, `TalentMods::tame`); the odds in §5.3 and the wiki's table |
| Q11 | "stays. also, give ranger the ability to share buffs starting from some level: buffs cast on ranger or the pet affect the other. but not hexes." | Barkskin stays at 6. **Shared buffs** join Hunter's bond (25) as the owner suggested in Q12: one hook in `World::afflict`, a buff spell's status on the ranger lands on the companion and the reverse, a hex never (§11.2) |
| Q12 | "skill use should not be gated. the abilities can give extra benefits thou. perhaps put the q11 spell share to bond, and let the heal skill heal from the start?" | **Heal tends an adjacent ally from the start**, for anyone with the skill (§9, change D, slice C0); the first draft's `tends_ally` at 25 is gone and shared buffs took its place in the bond |
| Q13 | "allies.md with a section listing starting companions." | `wiki/src/allies.md` gains "Companions" with "Starting companions" (the generated table of every race's beasts and the rares with the odds) and "Taming" (§14.4) |

What the merge of the core changed besides: the first draft's §4.2 "asked of A1" is replaced by
§4.2 "what this design changes in the core", since A1–A3 and D1 are merged (the variant exists
without data; `comes_along` is 3; Send away is gone; Handle animal marks on orders and kills;
Beast-bond needs 20); the slices are renumbered C0–C8 and no longer wait on anything.

## 19. Open questions, each with a recommended default

Only what genuinely needs the owner; everything else above is decided with a reason.

| # | Question | Recommended default |
|---|---|---|
| Q1 | **Growth for every master**: does a non-ranger's tamed beast grow with its master's level too (grade = the master's level), or only the ranger's companion (others' beasts stay their type's level)? | **every master's**: one rule, the hooks read `CompanionData::grade` with no class in them, and a commoner's hound at depth 10 is otherwise dead weight; the ranger's edge is the beast from turn one, the full share, the tame bonus, and the three abilities. If the owner wants the gap wider, `grade_percent` could become the class's (`ClassDef::companion_grade`, default 50, the pet classes 100) at the cost of one field |
| Q2 | **The rares**: a wolf (4), a black hound (5), a great boar (6), a panther (6), at 50 per mille for every race | **as listed**: all between 4 and 6, so a jackpot is felt (two to three levels of hit points and dice over a common start) without carrying the first floors alone; a bear (12) is left out on purpose |
| Q3 | **The nose through doors**: a beast smells only along its line of sight (as Blood scent does), or also through a closed door (what is behind a door is the classic use of a dog)? | **line of sight**: one rule, the pass the code has, and a closed door is opened by the ranger in a turn; through doors is one more arm (the cells beyond each door adjacent to the beast) if the owner wants it |
| Q4 | **Shared buffs' sources**: every buff status the ranger gains (spells, scrolls, potions, boons), or buff spells only? | **every buff status**: the hook in `afflict` is one door and sees them all; "buffs cast on" in the owner's words was about the kind, not the source. A potion of haste shared with the hound is the fun of it |

## 20. Decisions: the owner's third round (2026-11-03, approving)

Recorded verbatim from PR #832; the §19 questions not named take their defaults.

| # | The owner | What it means |
|---|---|---|
| §19 q1 | "Growth for every master: yes," | a tamed beast grows with its master's level, whoever the master |
| §19 q2 | (not named) | the default: wolf, black hound, great boar, panther at 50 ‰ |
| §19 q3 | "The nose through doors: through closed door too." | **not the default**: the nose finds hostiles within its reach through closed doors as well, not only in the beast's line of sight (C1's nose predicate and C5's shared cells read it so) |
| §19 q4 | "Shared buffs' sources: every buff status" | the default: any buff status, whatever laid it; never a hex |

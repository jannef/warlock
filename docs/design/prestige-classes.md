# Warlock — Prestige classes: the calling, the Hunter, and the curve to 100

Design document for the owner's review (2026-11-02; issue #819, the bard's second round on PR
#835). Written against `origin/main` at 38c5085d (Allies A1–A3 and the summon spell kind, #837;
saves pinned to the game version that wrote them, #840; the Monk renamed Ascetic and the
barbarian Berserker). Nothing here is implemented; §10 cuts it into issues. It is the second half
of `docs/design/bard-jack-of-all-trades.md` (the bard's table compressed into levels 1 to 30 and
capped there): this document is the system that lets a capped class go on, the one prestige class
the owner asked for, and the level curve extended to 100. It builds on the level curve and
`Factor` (`docs/dev/experience.md`), class tables and the coverage policy (`docs/dev/progression.md`,
rules 1–9 of #776, binding), quests and the kill tally (`docs/dev/quests.md`), hidden locations
(`docs/dev/overworld.md`), corpses and eating (`docs/dev/hunger.md`), traps (`docs/dev/traps.md`),
statuses (`docs/dev/status-effects.md`), allies and summons (`docs/dev/allies.md`), and saves
(`docs/dev/saves.md`). Every number is data and hidden in the game; the wiki states it.

**The owner's direction** (PR #835, 2026-11-02, verbatim): *"Also, let's do something funky. Bard
gains all these abilities by level 30. Bard does not gain levels above 30. With certain conditions,
a character can start advancing in another class. this is not going to be general multiclass
thing. More like prestige classes achievable by finding a "hidden" mechanism. Also implement one
prestige class for bard. condition: lvl30, certain event has happened. Call this class "hunter".
it will be a martial class with 30 levels. bard retains all the abilities when it starts
progressing in that class. XP requirements will be based on total level. Extend the XP tables all
the way to 100 - we are not going there yet, but who knows, maybe we will. This might make "bard"
pretty op, but who cares."*

## Contents

1. Summary
2. What the direction rules in and out
3. Research: prestige classes, class changes, and hunters elsewhere, tied to decisions
4. The system: total level, class caps, the calling, what carries across
5. The level curve to 100
6. The Hunter
7. The Hunter's calling in the Ascmearc: the wyrm's heart
8. What the player sees; determinism, saves, replays
9. The wiki, the spec, and the docs
10. Implementation plan
11. Rejected ideas, with the reason
12. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **One level, two classes** | a character has one **total level**, read off the one curve from their XP as today (`Experience::level_for`), capped by `Character::level_cap`: the base class's `max_level` (the bard 30; every other class 60, the default), plus a prestige class's `levels` once called. Levels up to the base cap are the base class's; each level past it is the prestige class's. §4.1 |
| **The calling** | a prestige class is never offered or chosen: the world **calls** the character when a campaign's `[[calling]]` holds: the base class is one it names (`from`), the total level is at least its `level` (the base cap, 30), and its hidden fact `when` has happened (`ate`, `killed`, `slain`, `quest`, `reached`, or a `once` flag). The character becomes level 1 of the prestige class at once, keeps everything, and every level after is the prestige class's. Nothing in the game states the condition; one hint line in the world and the wiki do. §4.2 |
| **XP at the cap** | banked: XP keeps counting past the cap and buys nothing (the rule at 60 today) until the calling, when the banked levels come at once (`level_up(cap, level_for(xp))`): no new state, and a mechanism found late is worth the more. §4.3 |
| **What a level brings** | hit points (`hp` of the class the level falls in, to `hp_level` 50), skill rolls (that class's `rolls`), a talent pick every third total level, the class table's abilities at the class's own level, its bonus picks at its levels, the stat checks: all as today, each read through "the class of this level". A prestige class has no stats, kit, alignment, casting, affinity, or experience percent: those are the character's base, for life. §4.4 |
| **The Hunter** (martial, 30 levels, from the bard) | one quarry at a time: mark a creature you see, hunt it by scent and then anywhere on the level, hit it surer and harder (Quarry 1/3 to 3/3: +1/+2 to +3/+6 and two marks), hamstring it with a critical, lay snares that hold what steps in them, finish it when it is nearly down (Killing stroke), sound the horn for a hound that fights beside you (the summon spell kind), and at 30 hunt its whole kind at once with two hounds (Master of the hunt). `hp = 6`, 2 rolls, known for Survival, Stealth, and Listen; bonus picks from Weaponcraft and Shadow. Distinct from the ranger (the kill tally's favoured prey, Focus, the bow): the Hunter chooses one quarry and ends it, with any weapon. §6 |
| **The calling in the Ascmearc** | **eat the fire drake's heart**: the player has eaten of the corpse of the fire drake under Fyrbeorg (`when = { ate = "fire_drake" }`, recorded in `Progress::once` as `ate:fire_drake` when a unique type's corpse is eaten) and is a bard of 30. Sigemund's wyrm in Beowulf and Sigurd's taste of Fafnir's heart's blood in the Völsunga saga: the campaign's own well. A rotation line of Wat One-Eye's hints at it; eating it below 30 says an omen; the wiki says it plain. §7 |
| **The curve to 100** | `max_level = 100` (`MAX_LEVEL` 99 → 100): the same `[kills]` formula carried on (level 100 costs 485 kills of one's own level; 9,966,934 XP in all), `[levels] class_max = 60` the default class cap, `[[zero_difference]]` one more every five levels to 100 (25), `hp_level` and `mana_level` unchanged. Nobody reaches past 60 (the bard-hunter's 59) until a class or a prestige says so in data. §5 |
| Coverage policy | no new `[[system]]`: a prestige class is a class (its abilities are class abilities under `class = true`); the Hunter's entries extend `combat`, `awareness`, `traps`, `status`, `allies`, `weapon_skills`. No pickable minted. Rules 1–9 checked per entry in §6.4. |
| Slices | P1 the system (the cap, the calling, `Character::prestige`, the per-class level reads, the dev `call`), P2 the curve to 100, P3 the Hunter's table in three issues (H1 Quarry and Hunter's trade and Scent of the quarry; H2 Hamstring, Killing stroke, Master of the hunt's kind; H3 Snare; H4 Hound of the hunt), P4 the calling content (the rite recorded, the Ascmearc's calling, the hint, the showcase's calling), P5 the wiki, the spec, and the docs. §10 |

**Key choices, and why**

1. *Not multiclassing: a second table on the same level.* The owner: "not going to be a general multiclass thing." A character never splits XP, never chooses which class a level goes to, never has two of anything to track: the total level is the level, and the class of a level follows from where it falls. A prestige class is a class table that starts where the base's ends.
2. *The calling is a fact of the world, read from state the game already keeps.* `Quests::slain` and the kill tally, quest records, visited levels, and `Progress::once` exist; the calling reads them. The one new record is `ate:<id>` for a unique type's corpse eaten (one line in `eat.rs`). No dialogue, no accept, no cost: the character is called, as a quest's stage is set.
3. *The calling is campaign data; the class is shared data.* The Hunter (`data/prestige/hunter.toml`) is any campaign's to call; **how** a world calls it is that world's (`campaigns/ascmearc/data/callings/hunter.toml`): a campaign without a calling has no Hunter, and the test world's calling is whatever its tests need.
4. *Found, not given.* The owner: "achievable by finding a hidden mechanism." Nothing in the game names the condition; a hint line in the world is the only breadcrumb, and the wiki states the truth (the owner's rule). A hidden mechanism the player can never learn of is a mechanism nobody finds; one line is enough.
5. *The calling is the first level.* The Hunter's level 1 comes with the calling (total 30), so finding the mechanism does something at once; Hunter 30 is total 59. The alternative (Hunter 1 at total 31, nothing at the calling but words) is §12 Q4.
6. *Banked XP, no state.* The rule "XP beyond the cap counts but buys nothing" exists; the calling spends what was banked. The alternative, counting from the calling, needs the XP at the calling saved and takes from the player what the game told them they had.
7. *The Hunter is the quarry, not the bow.* The ranger owns the bow, Focus, and the kill tally's favoured prey; the Hunter owns **one chosen quarry**: Complete Adventurer's Bloodhound (mark, nowhere to hide), 4e's Hunter's Quarry, Pathfinder's studied target, WoW's Hunter's Mark, Monster Hunter's one monster a hunt. Every Hunter ability reads the mark, with any weapon.

## 2. What the direction rules in and out

| The owner said | Read as |
|---|---|
| "Bard gains all these abilities by level 30. Bard does not gain levels above 30." | the bard's table compressed (the bard design §5) and `max_level = 30` on the class |
| "With certain conditions, a character can start advancing in another class." | a second class table the character's levels go into once a condition holds |
| "this is not going to be general multiclass thing" | no choosing a class per level, no XP split, no second progression bar, no two base classes; the base class is for life and its table is kept whole |
| "More like prestige classes achievable by finding a 'hidden' mechanism" | D&D's prestige classes (entered by prerequisites, some of them deeds) crossed with a secret: the condition is a thing done in the world that nothing announces |
| "Also implement one prestige class for bard. condition: lvl30, certain event has happened. Call this class 'hunter'. it will be a martial class with 30 levels." | the Hunter (§6), from the bard only for now (`from = ["bard"]`; the field is a list so the owner may open it to others later), 30 levels, fighting abilities |
| "bard retains all the abilities when it starts progressing in that class" | nothing is taken away; the bard's talents, skills, songs, mana, casting, and kit stay; the Hunter's table adds |
| "XP requirements will be based on total level" | one curve, one XP total, the next level's threshold read at the total level; a prestige class has no experience percent of its own |
| "Extend the XP tables all the way to 100" | `max_level = 100` and the formula carried on (§5); no class reaches it yet |
| "This might make 'bard' pretty op, but who cares." | balance is not a goal (CLAUDE.md); the bard-hunter is the strongest thing a run can be, on purpose, and the only one with 59 levels |

## 3. Research: prestige classes, class changes, and hunters elsewhere, tied to decisions

| Source | What it does | Taken, and left |
|---|---|---|
| **D&D 3.0/3.5 prestige classes** (DMG; Complete Adventurer, Masters of the Wild): 5- and 10-level classes entered by prerequisites (base attack, skills, feats) and often a **special** one (the Assassin must kill for no other reason than to join; the Dragon Disciple must have draconic blood; the Shadowdancer must have Hide 10 and Tumble 5); levels add to the character level for XP and hit dice; the base class is kept | **Taken**: the shape (a table entered by a condition, on top of the base), total level for XP, the "special" prerequisite as a deed. **Left**: free switching between base and prestige levels (multiclassing proper: ruled out), the 5-level size (the owner: 30), the many tiny prestige classes (one, well made) |
| **Complete Adventurer's Bloodhound**: *mark* (a quarry chosen by name or sight, bonuses to track and strike it), *ready and waiting*, *nowhere to hide*, *dead or alive* (nonlethal), *bring 'em back alive*, *tenacious pursuit* | the closest hunter prestige class to the one proposed. **Taken**: the mark as the class's spine (Quarry, §6.2), tracking the marked (Scent of the quarry), one quarry at a time. **Left**: nonlethal and the bounty economy (rules 1 and 9) |
| **Masters of the Wild's Foe Hunter** (hated enemy: rancor attack, damage reduction against it) and **Deepwood Sniper** (range increments, concealment reduction, keen arrows); **Order of the Bow Initiate** (a precise ranged shot) | **Taken**: the Foe Hunter's "one enemy, struck harder" is the mark again; the Deepwood Sniper's "the arrow finds the spot" is Killing stroke's threat and penetration (§6.2). **Left**: the sniper's range increments (the bow is the ranger's, and Warlock prices range per launcher); nothing is named "aimed shot" (the owner: all shots are aimed) |
| **D&D 4e**: *Hunter's Quarry* (the ranger marks the nearest enemy once a turn: +1d6 damage, 1d8 at 11, 1d10 at 21); **paragon paths** at 11 and epic destinies at 21 (a second table on top of the class, chosen, no prerequisites but class) | **Taken**: the quarry's damage that grows with level → Quarry's three ranks (+2, +4, +6 damage). **Left**: a chosen path (the owner: found), one quarry a turn re-marked for free (Warlock's mark costs a turn and lasts: the hunt is slow) |
| **Pathfinder**: the Slayer's *studied target* (+1 to +3 to hit, damage, and skills against one target, more targets at higher level), the Ranger's *quarry* (+2 to hit, criticals confirmed, against one target), the Hunter class (an animal companion as the class) | **Taken**: more marks at higher level (Quarry 3/3 holds two), a companion at the table's end (Hound of the hunt, §6.2). **Left**: the companion as the whole class (the hound is a summon, an hour at a time) |
| **World of Warcraft's Hunter**: *Hunter's Mark* (the target shown, more damage to it), *Freezing Trap* (the first thing to step on it is held), *Kill Shot* (a heavy shot at a target below 20 %), pets, *Track Beasts* | **Taken**: the mark seen through walls (Scent 2/2), a trap that holds the first to step in it (Snare, §6.2), the finisher against a low target (Killing stroke's "below a quarter"), the pet as the hound. **Left**: aspects, feign death, disengage (mobility is Fast movement's and Fleet's; the hunter waits) |
| **Monster Hunter** (Capcom): one monster a hunt; pitfall and shock traps hold it; carve its corpse for parts | **Taken**: one quarry, traps that hold, and the corpse that matters (the calling eats it, §7). **Left**: crafting from parts (no crafting in Warlock) |
| **The Witcher**: a bestiary; oils and bombs by the monster's kind | **Taken**: "know the beast" as the whole kind being your quarry at 30 (Master of the hunt). **Left**: consumables by kind |
| **Dragon Quest III**: the class change at Dhama (Alltrades Abbey) at level 20, the character back to level 1 of the new class keeping half its stats and all its spells; the Sage needs the *Book of Satori*, a hidden item | **Taken**: the character keeps what it was (the owner: "retains all the abilities"), and the hidden item as the way into a class is the calling's spirit (a hidden rite, §7). **Left**: the level reset (the total level is one and never falls) and the temple (nothing is asked for at a place) |
| **Final Fantasy Tactics**, **Ragnarok Online**, **Fire Emblem**, **Wizardry**: job unlocks by job levels, job changes at a job level, promotion items, class changes at a cost of levels | **Taken**: a second class reached by the first's levels (the cap). **Left**: a chosen change (found, not given), a level reset |
| **Path of Exile's Ascendancy** (the Labyrinth: a hidden dungeon run once to ascend) | **Taken**: a deed in the world as the door. **Left**: a choice of three at the end |
| **ADOM**, **DCSS**, **Elona**: no class change or prestige class | the roguelike precedent is none; Warlock's is one class, one secret, no reset |
| **Beowulf** 874–897 (Sigemund slays the dragon and takes its hoard) and the **Völsunga saga** (Sigurd roasts Fafnir's heart, tastes the blood, and understands the birds; eats it and gains its wisdom) | the campaign's names are Beowulf's (Brosinga mene, Grendles grāpe); the Ascmearc has one wyrm, the fire drake under Fyrbeorg, and eating a corpse is a thing a Warlock character does. **Taken**: the calling is the drake's heart eaten (§7). **Left**: the birds' speech (no rule to hang it on) |

## 4. The system: total level, class caps, the calling, what carries across

### 4.1 One level, two classes

| Rule | Decision | Where |
|---|---|---|
| The total level | as today: `Experience::level_for(xp, factor, cap)` off the one curve (to 100, §5), `Factor` the race's and the **base** class's percents; now **capped at `cap`**, `Character::level_cap(content)` = the base class's `max_level` + the prestige class's `levels` if called | `content/experience.rs`, `world/experience.rs::player_level`, `character.rs` |
| The class cap | `ClassDef::max_level` (new; `classes/README.toml`), default `[levels] class_max` of `data/experience.toml` (60, so no other class changes); the bard's file says `max_level = 30`. Checked at load: 2 to `max_level`, and every `abilities` row and `bonus_talents` level within it | `content/class.rs` |
| Which class a level is | `Character::class_of(level) -> (class id, class level)`: `level <= base.max_level` → the base at `level`; else the prestige at `level - base.max_level + 1`. So the calling at total 30 is Hunter 1, total 31 Hunter 2, total 59 Hunter 30 (key choice 5) | `character.rs` |
| A prestige class | `PrestigeDef` from `data/prestige/*.toml` (`[[prestige]]`: `id`, `name`, `description`, `levels` (1 to 60), `hp`, `mana` (default 0), `rolls`, `known_for`, `bonus_talents`, `abilities` at its own levels 1 to `levels`): a class in everything a level reads, and nothing a creation reads (no `stats`, `kit`, `skills`, `weapon_skills`, `styles`, `casting`, `affinity`, `experience`, `alignment`, `creation_talents`, `optional_skills`). Its ids are its own: a prestige and a class may not share one. Its abilities are talents of the `class` tree, `granted`, checked by `check_table` like a class's (the coverage check counts them as class abilities) | `content/prestige.rs` (new, after classes and talents load) |
| The character's | `Character::prestige: Option<String>` (saved, hashed, left out while `None`): set by the calling, never cleared (nothing takes a class away; Unmade's lost levels, `Progress::peak`, lower the total and so the prestige level, and give back as they return) | `character.rs`, `save` |
| Only one | a character is called once; a second calling whose `from` names the base is not checked once `prestige` is set (a prestige class is never a `from`). The owner asked for one; `from` is a list so another base class may be let in later by data | `world/calling.rs` |

### 4.2 The calling

```toml
# campaigns/ascmearc/data/callings/hunter.toml (one `[[calling]]` a file; content/calling.rs)
[[calling]]
class = "hunter"                 # a prestige class (data/prestige/)
from = ["bard"]                  # the base classes it calls
level = 30                       # the total level asked: at least each base's max_level
when = { ate = "fire_drake" }    # the hidden fact (one of ate, killed, slain, quest, reached, once)
omen = "You taste the wyrm's blood: bitter, and hot, and not done with you."
message = "The wyrm's blood wakes in you. You know now how a thing is hunted: you are a hunter."
```

| Rule | Decision | Where |
|---|---|---|
| `when`, the fact | one of: `ate = "ID"` (the player has eaten of the corpse of unique creature type `ID`: `Progress::once` holds `ate:ID`, recorded by `eat.rs::finish` for a corpse whose type is `unique`, one line, so the set stays a few ids), `killed = "ID"` (the player's own kill of type `ID` on any level: `Quests::kills`), `slain = "ID"` (a unique type dead at any hand: `Quests::slain`), `quest = "ID"` (a quest completed: `Quests::records`), `reached = "LEVEL"` (a level the player has been on: `WorldState::levels` has its state), `once = "FLAG"` (`Progress::once`: content yet to come, and the dev action). Checked at load against the campaign's creatures (unique for `ate` and `slain`), quests, and levels. Exactly one | `content/calling.rs::When`, `world/calling.rs::holds` |
| `level` | at least every `from` class's `max_level` (the loader refuses less: a prestige level before the base is full would give two classes a level) | `content/calling.rs` |
| When it is checked | `World::call()` at the end of `World::apply` (after the action and the turn, where quest news is said): no prestige yet, the character's class in `from`, total level at least `level`, `when` holds → **called**. So it fires at the deed if the level is there, or at the level-up if the deed came first. Not on `from_state` (a restore changes nothing) | `world/calling.rs`, `world/mod.rs::apply` (one line) |
| What it does | `Character::prestige = Some(class)`; `message` said; the prestige's level-1 abilities granted (`class::grant` with the prestige table at class level 1: "You gain Quarry."); then the banked XP: `level_up(base.max_level, level_for(xp, factor, new cap))`, which brings every level banked with its hit points, rolls, picks, and abilities, each in the class it falls in (§4.4). No draw of its own (`level_up`'s stat checks draw from the levels stream as any level-up does) | `world/calling.rs::called` |
| The omen | when the fact is recorded (`ate:` written) and a calling with that `when` names the character's class but the level is short, `omen` is said once (the hint that something has happened; nothing names what). Said only for the character's own class's calling; a warrior eating the heart hears nothing (there is no calling for them) | `eat.rs::finish` → `World::omen(fact)` (one line) |
| Refusals | none: a calling is not an action. The dev action `call:ID` (`Action::CallDev`, no time; dev only) calls prestige class `ID` now, whatever its conditions, refused with no prestige of that id, one already held, or a base not in any calling's `from` of the run's campaign | `action.rs`, `world/calling.rs` |

### 4.3 XP at the cap

A bard at 30 has 545,660 XP (at 100 %) and keeps earning. Today's rule at 60 holds at any cap: the total counts on and buys nothing; the XP row shows no next level. The calling reads the total: a bard called at 700,000 XP becomes Hunter 1 and at once Hunter 2, 3, and 4 (level 33 is 683,000 and 34 at 732,000), with every level's hit points and rolls and the picks for 33. Nothing is saved for this. A long-capped bard who finds the heart late jumps far: that is the hidden mechanism's reward, and the owner's "who cares".

### 4.4 What a level brings, by the class it falls in

| Each total level | Today | After |
|---|---|---|
| Hit points | `max(1, class hp ÷ 2 + Toughness modifier)` to `hp_level` 50 (`World::level_hp`) | the same with **the class of the level** (`class_of`): a Hunter level brings the Hunter's `hp = 6` → 3 + Toughness; 51 and up Hardened's only (unchanged) |
| Mana | `max(0, class mana ÷ 2 + the better of Intelligence and Willpower)` to `mana_level` 60 | the class of the level: the Hunter's `mana = 0` → the modifier alone, so a bard-hunter's mana grows a little still |
| Skill rolls | class `rolls` + race + Intelligence | the class of the level's `rolls` (the Hunter's 2); `known_for` for the ladder step is the **union** of both classes' (a bard-hunter who took up Survival at 12 rolls it at the known-for die); the +1 for a class's other skills reads either class's `skills` (the Hunter has none: its `known_for` alone) |
| Talent picks | every `talent_every` total level (3) | unchanged: the total level's |
| Bonus picks | the class's `bonus_talents` at its levels | the class of the level's, at its **class** level (the Hunter's 5, 10, 15, 23, 28 are total 34, 39, 44, 52, 57, from Weaponcraft and Shadow); the bard's 5, 10, 15, 23, 28 as before |
| Class abilities | `class::grant` up to the level, at rank 1 or the named rank | per class: the base's table to `min(total, max_level)`, the prestige's to its class level; on a restore both (a grown table reaches saves) |
| Stat checks | each stat once a level (`check_stats`) | unchanged |
| Casting, caster level | `casting` percent of the character level | unchanged: the **base** class's percent of the **total** level (a bard-hunter casts at three quarters of 59; the owner's "pretty op") |
| Experience percent | race × base class | unchanged (a prestige class has none) |
| The news | "Welcome to level 31." | unchanged; the class's name is the character screen's business |

## 5. The level curve to 100

`data/experience.toml`: `max_level = 100` (`content::experience::MAX_LEVEL` 99 → 100). The `[kills]` formula is unchanged and simply carried on: k(n) = 213 × m ÷ (m + 5) + 2.85 × m kills of one's own level, each worth 5n + 10 XP; the threshold table at load grows to 100 entries and its pinned hash updates (a deliberate change). The totals at 100 % (`level_xp`, exact integer math as today):

| Level | Kills of one's own level | That level's XP | Total XP |
|---:|---:|---:|---:|
| 30 | 264 | 42,292 | 545,660 |
| 31 | 268 | 44,232 | 589,892 |
| 40 | 300 | 62,989 | 1,080,032 |
| 50 | 333 | 86,561 | 1,837,200 |
| 60 | 365 | 112,998 | 2,845,856 |
| 70 | 395 | 142,293 | 4,134,599 |
| 80 | 425 | 174,443 | 5,732,001 |
| 90 | 455 | 209,447 | 7,666,603 |
| 100 | 485 | 247,304 | 9,966,934 |

Beside it: `[levels] class_max = 60` (the default `ClassDef::max_level`, so every class but the bard is where it was; nothing reaches 61 until a class or a prestige says so in data); `hp_level = 50` and `mana_level = 60` unchanged (levels 61 to 100 add nothing but Hardened's; the owner's call when a class gets there); `[[zero_difference]]` carried on at one more every five levels: 65 → 18, 70 → 19, 75 → 20, 80 → 21, 85 → 22, 90 → 23, 95 → 24, 100 → 25 (the table's shape from 30 on). `Experience::level_for` takes the cap (§4.1); `threshold`, `xp_for`, and `kills_for` read to 100 for the wiki. The wiki's experience table (`xtask/src/wiki/experience.rs`) prints all 100 and says which classes stop where. Tests: the pinned hash, `level_for` at a cap below the curve's end, the discount untouched (levels 2 to 11), the ZD rows.

## 6. The Hunter

### 6.1 The fantasy in one paragraph

A hunter chooses. Where the ranger fights whatever the borderlands send and comes to know the kind they have killed most, the hunter looks at one creature and says *that one*, and from then on the fight is about it: they smell it in the dark, find it through walls, slow it with a well-placed blow, snare the ground it must cross, and when it is nearly down they end it. Their hound runs with them. At the top of their craft the quarry is a whole kind, and two hounds run. A bard who has eaten a wyrm's heart hunts like this with a lute on their back and every song still in them: that is the owner's "pretty op", and it is the point.

```toml
# data/prestige/hunter.toml
[[prestige]]
id = "hunter"
name = "Hunter"
description = "One who chooses a quarry and ends it: by scent, by snare, by the killing stroke, with a hound at heel."
levels = 30
hp = 6
rolls = 2
known_for = ["survival", "stealth", "listen"]
bonus_talents = { levels = [5, 10, 15, 23, 28], trees = ["weaponcraft", "shadow"] }
abilities = [
    { level = 1, talent = "quarry" },
    { level = 1, talent = "hunters_trade" },
    { level = 3, talent = "scent_of_the_quarry" },
    { level = 6, talent = "hamstring" },
    { level = 9, talent = "snare" },
    { level = 12, talent = "quarry", rank = 2 },
    { level = 15, talent = "scent_of_the_quarry", rank = 2 },
    { level = 18, talent = "killing_stroke" },
    { level = 21, talent = "hound_of_the_hunt" },
    { level = 24, talent = "quarry", rank = 3 },
    { level = 27, talent = "snare", rank = 2 },
    { level = 30, talent = "master_of_the_hunt" },
]
```

Eleven rows on the bard's cadence (1, 3, 6, 9, 12, 15, 18, 21, 24, 27, 30: every third level, as a martial class's picks come), three of them ranks of Quarry and two of Scent and Snare (rule 4). `hp = 6` sits between the warrior's 7 and the ranger's 5; two rolls as most classes; Survival, Stealth, and Listen (not the ranger's Spot, Survival, Woodcraft): a hunter waits and listens.

### 6.2 The table, entry by entry

| Class level (total) | Ability | Rule | Data and hook |
|---:|---|---|---|
| 1 (30) | **Quarry** 1/3 | an ability (`use:quarry:X,Y`, aimed at a cell with a creature the hunter sees; the UI aims it as a shot): the creature becomes the hunter's quarry: it suffers **Marked** (`marked`, a new status: `max = 1`, no `eases` (it lasts until death), `seen = "marked"`, caused `by` the hunter) and the hunter's attacks against it are **+1 to hit and +2 damage**. One quarry at a time (`marks = 1`): marking another frees the first (the oldest Marked by the hunter loses the status). Priced as a wait (a base turn); no mana, no pool, no cooldown (one quarry is its own limit). Refused with no time: no creature there, one the hunter does not see, an ally, the hunter's own quarry already, the overworld (map scale). Ranks 2 and 3 (12 and 24) add +1/+2 each (summed +3/+6), rank 3 a second mark (`marks` 2) | `quarry { to_hit, damage, marks }` per rank (r1 `{1, 2, 1}`, r2 `{1, 2, 0}`, r3 `{1, 2, 1}`, summed by `TalentMods`); `ClassActive::Quarry` (`Aim::Cell`, a creature); `World::mark_quarry` (`world/hunter.rs`); read in `World::resolve` → `World::quarried(attacker, target)` beside `favoured` (blows, shots, throws; the target suffers `marked` by the attacker) |
| 1 (30) | **Hunter's trade** | a hand-out once: bows, crossbows, slings, thrown, and spears each raised to **rank 3** where lower (the defaults rise as any rank gained raises them); a bard's Travelled hands had them at 1. Rank 3 is one short of the first weapon ability (rank 4), which the Hunter earns by use | `weapon_ranks { skills = ["bows", "crossbows", "slings", "thrown", "spears"], to = 3 }` (the bard design's kind with `skills` named) |
| 3 (32) | **Scent of the quarry** 1/2 | the hunter sees their quarry in line of sight within 20 cells however dark (Blood scent's rule for Marked). Rank 2 (15, total 44): **anywhere on the level**: a creature Marked by the hunter is seen at its cell wherever it is (its glyph; not the cell's surroundings), through walls and doors, not on the overworld | r1 `scent { status = "marked", radius = 20 }` (exists: `Perception::compute`); r2 `track { status = "marked" }` (new, one read in `Perception::compute` beside `scent`: every creature on the level suffering the status caused by the viewer) |
| 6 (35) | **Hamstring** | a confirmed critical of the hunter's on their quarry, with any attack (a blow, a shot, a throw), leaves it **Sluggish** one degree (`sluggish`: everything half again as dear, a degree every 60 s). No draw beyond the critical's own | `quarry_critical { status = "sluggish" }`; read in `World::after_blow` beside `critical_status` (which is melee only; this one takes every attack kind, against a Marked target by the attacker) |
| 9 (38) | **Snare** 1/2 | an ability (`use:snare:E`, aimed in a direction): the hunter lays a snare on the floor cell that way, a **minute's** work: a trap of kind `hold` (`traps.toml`: `snare`, `kind = "hold"` (new), `status = "held"`, `severity = 6`, `charges = 1`, `difficulty = 20`, silent), **known** to the hunter from the start (the game asks "Really step onto the snare?" as for any known trap) and hidden to everyone else, so a creature steps on it as on any hidden trap (`creature_steps` → `step_onto_trap`) and is **Held** six degrees (36 s at `eases = 6`; it may struggle free by `held`'s Strength check as today), the snare spent and gone. At most **2 standing** on the level (`standing`): laying a third takes up the oldest, silently. Refused with no time: the cell is not plain floor (a wall, a door, water or ice, a chest, a trap, a creature, the stairs), the overworld. Rank 2 (27, total 56): **3 standing**, and the snared creature also **Bleeds** one degree (the snare bites) | `snares { standing }` per rank (r1 2, r2 1, summed) and r2 `snare_status { status = "bleeding", severity = 1 }`; `ClassActive::Snare` (`Aim::Direction`); `World::lay_snare` (`world/hunter.rs`); `LevelState` trap entries gain `laid: bool` (the player's, for the count and the taking up; saved as traps are); the `hold` trap kind in `world/trap.rs` (afflict `status` at `severity`, then `snare_status` if the layer has it) |
| 12 (41) | Quarry 2/3 | +2 to hit, +4 damage against the quarry | the rank (above) |
| 15 (44) | Scent of the quarry 2/2 | the quarry seen anywhere on the level | the rank (above) |
| 18 (47) | **Killing stroke** | against a quarry **below a quarter of its hit points**, the hunter's attacks threaten a critical **3 points lower** and **penetrate 2** more (through armour) | `quarry_low { below_percent = 25, threat = 3, penetration = 2 }`; read in `World::quarried` (the profile's threat and penetration, as `ferocity` reads the attacker's own hit points) |
| 21 (50) | **Hound of the hunt** | an ability, a spell of the summon kind paid in mana (`use:hound_of_the_hunt`): the hunter sounds a horn and a **hound** runs to their side for an hour and fights beside them (an ally of the allies system: follows, attacks what the hunter attacks, departs when its time is up or it dies). One at a time (`most = 1`); the horn is heard (`[spell.cast] noise`, a horn at 120). `spells/hound_of_the_hunt.toml`: `kind = "summon"`, spell level 24, `summon = { creatures = ["hound"], count = 1, most = 1, minutes = 60 }`; `creatures/hound.toml` (shared): a hunting hound, level 18, quick (speed 130), a bite of 1d8+2, `tags = ["beast", "hound"]`, animal faction, `sight 10`, keen-eared. Priced in mana as every summon is (the bard's mana; the Hunter has no pool: §12 Q6 for a cooldown instead) | `{ kind = "spell", spell = "hound_of_the_hunt" }` (exists: `Ability::Spell`, `cast_summon`); the spell and the creature are content |
| 24 (53) | Quarry 3/3 | +3 to hit, +6 damage; **two quarries** at once | the rank (above) |
| 27 (56) | Snare 2/2 | three snares standing; a snared creature Bleeds | the rank (above) |
| 30 (59) | **Master of the hunt** | **the quarry's whole kind**: every creature of a Marked creature's type on the level counts as the hunter's quarry for Quarry's bonus, Hamstring, and Killing stroke (not for Scent: the mark alone is smelt); and the horn calls **two hounds** (`count` +1, `most` +1) | `quarry_kind` (read where `quarried`, `quarry_critical`, and `quarry_low` ask "is this the quarry": the target suffers `marked` by the attacker, or a creature of its type on the level does); `ally_mod { count = 1, most = 1 }` (the allies design's kind, read by `cast_summon`) |

### 6.3 Why these, and not the ranger's

| The ranger has | The Hunter has instead | Why the two are not clones (rule 3) |
|---|---|---|
| Favoured prey (the type killed most, from the kill tally; +1/+2, +2/+4 with Master hunter) | Quarry (one creature, chosen, Marked; +1/+2 to +3/+6, two marks) | `favoured_prey` reads `Quests::prey` (a statistic the player grows into); `quarry` reads a status the player put there (a choice made in the fight). Different kinds, different conditions; the numbers overlap at rank 1 and the owner's "op" bard may hold both (a ranger is never a Hunter: `from = ["bard"]`) |
| Tracker (hearing +3, exact sounds), Stalker's eye (who is unaware) | Scent of the quarry (the quarry seen in the dark, then anywhere) | a sense of one creature, not of all sound |
| Rapid shot and Volley on Focus (the bow) | Hunter's trade (ranks in five missile and spear skills), Hamstring and Killing stroke with any weapon | the Hunter has no pool and no bow rule; its tricks are the quarry's, whatever is in hand |
| Barkskin (a spell) | Snare (a trap laid) | the Hunter is the first class to lay traps: the trap system's one new kind (`hold`) and its first player-laid trap |
| Evasion | Hound of the hunt (an ally) | the Hunter is the first non-caster with a summon |
| Master hunter (bows and crossbows threat +1, Favoured prey again) | Master of the hunt (the kind is the quarry, two hounds) | the ranger's capstone is the bow; the Hunter's is the hunt |

### 6.4 Rules 1–9, entry by entry

| Entry | 1 (capstones change a fight or a dungeon; no gold, favour, quest, XP) | 3 (no clone) | 4 (ranks) | 5–7 | 8 (not information only) | 9 (no favour) | Tags |
|---|---|---|---|---|---|---|---|
| Quarry 1–3 | changes every fight against the quarry | `quarry` is its own kind (§6.3) | three ranks, one id | — | the mark does something (the bonus); the status's `seen` is the garnish | — | `combat`, `status` |
| Hunter's trade | — (level 1) | a hand-out (`weapon_ranks`), not `weapon { ranks }` | — | — | — | — | `weapon_skills`, `ranged` |
| Scent of the quarry 1–2 | at 44 (rank 2): the quarry found through walls is the dungeon changed (a sense, as #791 kept Blood scent) | `scent` with another status and radius; `track` new | two ranks | — | a sense, not a readout: it shows a creature, as Blood scent does | — | `awareness` |
| Hamstring | — | `quarry_critical` ≠ `critical_status` (melee only, any target) | — | works on every floor | — | — | `combat`, `status` |
| Snare 1–2 | at 56 (rank 2): a trap laid changes a fight | the first player-laid trap | two ranks | underground as outdoors | — | — | `traps`, `status` |
| Killing stroke | at 47: the finisher | `quarry_low` ≠ `ferocity` (the attacker's hit points) | — | — | — | — | `combat` |
| Hound of the hunt | at 50: an ally in the fight | a `spell` of the summon kind (the druid's calls are other spells) | — | — | — | — | `allies`, `spells`, `mana` |
| Master of the hunt | at 59: the kind hunted, two hounds | `quarry_kind` new; `ally_mod` as the allies design's talents use it | — | — | — | — | `combat`, `allies` |

No entry pays in gold, favour, quests, or XP; none works only outdoors or in one season (rule 6); no hidden marks (rule 7); no flaw (rule 5).

### 6.5 Numbers, for the wiki

A Hunter 1 (total 30) with a long bow at Bows 3 against a Marked warg: +1 to hit and +2 damage on every shot; at Hunter 24 +3/+6, and two wargs Marked. Hamstring on a critical: the warg at 150 % cost for a minute. A snare: the first warg through the doorway Held six base turns (a Strength check against 13 to break free sooner), the hunter shooting all the while. Killing stroke on a warg under a quarter: criticals from 3 lower on the roll and +2 penetration. The hound: a level-18 beast for an hour, at the summon table's mana cost for spell level 24 (about 40; the bard-hunter at 50 has 40 to 60 mana). Master of the hunt: every warg on the level is the quarry.

## 7. The Hunter's calling in the Ascmearc: the wyrm's heart

| Piece | Proposal |
|---|---|
| **The fact** | `when = { ate = "fire_drake" }`: the player has eaten of the fire drake's corpse. The drake (`campaigns/ascmearc/data/creatures/fire_drake.toml`: level 19, unique, the bottom of Fyrdelf, Factor Wulfrun's "Fire in the deep") is the Ascmearc's one wyrm, "the last of the volcano's wyrms", and its hoard's lair is the deepest place in the campaign. Its corpse is a corpse like any creature's (nothing in its file says otherwise; the slice checks and, if a drake leaves none today, gives it one): fresh for a while, then old, then rotten, then gone (`docs/dev/hunger.md`), so the heart is eaten in the lair or carried out at once |
| **Why this** | Beowulf names Sigemund's wyrm and the Völsunga saga has Sigurd taste Fafnir's heart's blood and understand the birds: the campaign draws its names from that well (Brosinga mene, Grendles grāpe, the Eorcanstān). A hunter's calling should be a hunter's deed, on the greatest quarry the world has; eating the kill is a thing the game already lets a character do, and nobody does it to a drake without a reason. A bard who reads (Knowledge, Decipher script) might guess it; one who does not finds the hint |
| **The level** | 30, the bard's cap (§4.2 refuses less). The drake is level 19 and the quest brings a character of about 20 to it; most bards eat the heart below 30 and hear the omen, then are called at 30. One who comes to the drake late is called in its lair |
| **The hint** | one rotation line of Wat One-Eye's (`campaigns/ascmearc/data/dialogue/ceaptun.toml`, `lines`; Cockney, decision 13): *"Old 'unter I knew swore blind: eat the 'eart o' the wyrm under the mountain an' you'll never lose a trail after. Never met a soul as tried it, mind."* Said in rotation like his others; nothing more. The pedlar's and Wulfrun's lines are left alone (one breadcrumb is a hint; three is a quest) |
| **The omen** | eating it below 30: "You taste the wyrm's blood: bitter, and hot, and not done with you." (the calling's `omen`); said once |
| **The calling** | "The wyrm's blood wakes in you. You know now how a thing is hunted: you are a hunter." then "You gain Quarry." and "Your hands know the hunter's weapons." and the banked levels' news |
| **The wiki** | `wiki/src/classes.md`, a **Prestige classes** section: the rule, the Hunter's table, and the Ascmearc's calling stated plain ("a bard of level 30 who has eaten of the fire drake's corpse becomes a Hunter"): the game is vague, the wiki is accurate |
| **The test world** | `campaigns/showcase/data/callings/hunter.toml`: `from = ["bard"]`, `level = 30`, `when = { quest = "depths" }` (the hermit's quest, which the reveal tests use), so an integration test plays the whole way; unit tests use the dev `call:hunter` and `xp:N` |
| **Other callings** | none proposed. `when` has six shapes so a later class (a warrior's, a thief's) is data and a line in a campaign |

## 8. What the player sees; determinism, saves, replays

| Where | What |
|---|---|
| The character screen | the Class row: `Bard` until the cap, `Bard 30` at it (the level shown beside the class once the class is full, so a capped bard sees the class is done without being told why), `Bard 30, Hunter 4` after; the Level row the total. The death screen's "Here lies Brakka, a bard" becomes "a bard and hunter" |
| The Characters list | the summary (`saves/<id>.toml`) gains the prestige: "Bard 30 / Hunter 4, level 33" |
| The status bar | the level as today |
| The XP row | no next level at a cap, as at the top today |
| Messages | the omen; the calling and its grants; "The warg is your quarry." (Quarry) / "You lay a snare." / "The kobold is caught in your snare." (seen) / "Your snare is sprung." (heard: the status's start on an unseen creature says nothing; a snare is silent) / "You sound your horn, and a hound comes running." (the spell's `message`) / "Your hound departs." (the allies' words) |
| Look mode | "marked" on the quarry (`seen`), "held" on a snared creature; a snare as a known trap ("a snare, yours") |
| The abilities menu | Quarry, Snare, and Hound of the hunt under the class source, greyed "no quarry in sight", "nowhere to lay it", or the summon's refusals |
| Dumps | `stats`: `Class Bard 30, Hunter 4` (and `Level 33`); `talents`: `Quarry 2/3 (class)`; `traps`: `snare at (12, 5), yours`; `actors`: `marked (by player)`; `progress` (dev): `calling hunter: level 30 met, ate fire_drake not yet` / `called hunter at 30`; JSON `character.prestige`, `character.classes` (`[{ class, level }]`), `calling` (dev) |
| Determinism | the calling draws nothing of its own (`level_up`'s stat checks from the levels stream); Quarry, Snare, Hamstring, Killing stroke draw nothing; the hound's cast draws as any summon; a snare's spring is a trap's (no roll for a `hold`). Runs without a calling or a Hunter hash as before but for the content |
| Saves | new: `Character::prestige` (left out while `None`), `Progress::once` `ate:<id>`, trap entries' `laid` (left out while false), the `marked` status (statuses are saved by index: a new status is a `FORMAT_VERSION` bump by the integrator, as any is). Saves are pinned to the game version (#840): **no shim** for a save from before; `ClassDef::max_level` and the prestige are content |
| Replays | the calling is a consequence of the actions, never an action; `use:quarry:12,5`, `use:snare:E`, `use:hound_of_the_hunt`, and the dev `call:hunter` are action text (CLAUDE.md's list) |
| Content fingerprint | the `marked` status and the `snare` trap (statuses and traps are fingerprinted); the prestige and the calling are not (saves name them by id) |

## 9. The wiki, the spec, and the docs

| Where | What |
|---|---|
| `wiki/src/classes.md` | a **Prestige classes** section: the rule (one level, the cap, the calling, what carries across, banked XP), the Hunter (fantasy, the generated table with every effect's numbers), and each campaign's calling stated plain (generated from the callings: `xtask/src/wiki/classes.rs`) |
| `wiki/src/experience.md` | the curve to 100 (generated), the class caps |
| `wiki/src/abilities.md` | Quarry, Snare, Hound of the hunt rows |
| `wiki/src/status-effects.md`, `traps.md`, `creatures.md` | Marked; the snare (a `hold` trap); the hound |
| `spec/08i-character.md` | "Prestige classes" after "Class abilities": the total level, `max_level`, the calling and its facts, `class_of`, what a level brings by class; the Hunter's table bullet; "Experience": the curve to 100 |
| `spec/07b-data.md` | `classes/README.toml` `max_level`; `prestige/README.toml`; a campaign's `data/callings/`; `traps` `hold`; `talents/README.toml` the Hunter's kinds |
| `spec/08b-creatures.md` | the hound; a player-laid trap |
| `docs/dev/experience.md` | the cap, `class_of`, `level_for` with a cap, the curve to 100, the banked XP |
| `docs/dev/progression.md` | "Class tables": prestige tables and the calling; the hook rows (`quarry`, `quarry_critical`, `quarry_low`, `quarry_kind`, `snares`, `snare_status`, `track`); rule 1's wording for a 30-level table (the bard design's Q19) |
| `docs/dev/campaign.md` | `data/callings/` is a campaign's; the cross-campaign check (a calling names its own campaign's creatures, quests, and levels) |
| `docs/dev/traps.md`, `hunger.md`, `allies.md`, `saves.md`, `character-creation.md` | the `hold` kind and `laid`; `ate:` recorded; the hound as a non-caster's summon; the new saved fields; `Character::prestige` |
| CLAUDE.md | the action text (`use:quarry:X,Y`, `use:snare:E`, `call:ID`), the `stats` and `progress` dump lines |
| Tests | `content/prestige.rs` and `content/calling.rs` (loading, every refusal: an unknown class, a `from` that is a prestige, a level below the cap, a fact naming nothing of the campaign, two `when`s); `world/calling_tests.rs` (the fact recorded by eating a unique's corpse and not a common one, the omen below the level and silence for a class with no calling, the calling at the deed and at the level-up, the banked levels coming at once with the right class's hit points and rolls, both tables on a restore, the dev `call`, Unmade's lost level and its return, a replay, a save); `world/hunter_tests.rs` (Quarry's refusals and bonus, one mark then two, the oldest freed, Scent in the dark and Track through walls, Hamstring on a critical of each attack kind and never on a non-critical, Snare's placing refusals, the count and the taking up, a creature held and bleeding at rank 2, the player asked before stepping on their own, Killing stroke's threshold, the hound called and departing, Master of the hunt's kind, a replay); `content/experience.rs` (the hash to 100, `level_for` capped, the ZD rows); the character-screen, death-screen, and abilities-menu text snapshots (`character-screen-prestige`, `abilities-menu-hunter`) |

## 10. Implementation plan

Rules first, then content, then the wiki. The false-conflict trick (`docs/dev/merging.md`) for the hot files: `experience.toml`, `status.toml`, `traps.toml`, `talents/README.toml`, `ability.rs`, `action.rs`, `class.rs`, `perception.rs`.

| Issue | What | Adds | Depends on |
|---|---|---|---|
| **P1 The system** (§4) | `ClassDef::max_level` and `[levels] class_max`; `Experience::level_for` with a cap; `PrestigeDef` (`content/prestige.rs`) and `CallingDef` (`content/calling.rs`, a campaign's `data/callings/`, the six `when` shapes and their load checks); `Character::prestige`, `class_of`, `level_cap`; every per-level read through the class of the level (hit points, mana, rolls, known-for union, bonus picks, `class::grant` per table); `World::call` at the end of `apply`, the omen hook, `ate:` recorded in `eat.rs::finish` for a unique's corpse; the dev `call:ID`; the character screen's Class row, the summary, the dumps; a fixture prestige and calling for the tests | **saved state** (`Character::prestige`, `once` flags), a dev action | — |
| **P2 The curve to 100** (§5) | `max_level = 100`, `MAX_LEVEL` 100, the pinned hash, the ZD rows, `class_max = 60`; the wiki's experience table | none | P1 (the cap, so nobody reaches 61) |
| **H1 Quarry, Hunter's trade, Scent of the quarry** (§6.2) | the `marked` status; `quarry` (three ranks) and `ClassActive::Quarry` with `mark_quarry` and `quarried`; `weapon_ranks { skills }` (the bard's B2 gives the kind); `scent` on Marked and the `track` read; the Hunter's file with these rows and the rest commented as "to come"; `hunter_tests` | **Action** (`use:quarry`), a status | P1, the bard's B2, B3 (`rank` on `AbilitySpec`) |
| **H2 Hamstring, Killing stroke, Master of the hunt's kind** (§6.2) | `quarry_critical`, `quarry_low`, `quarry_kind` and their reads in `after_blow` and `quarried` | none | H1 |
| **H3 Snare** (§6.2) | the `hold` trap kind and the `snare` trap; `laid` on trap entries; `snares` and `snare_status`; `ClassActive::Snare` with `lay_snare`; the player asked before their own snare; the `traps` dump line | **Action** (`use:snare`), saved state (`laid`) | P1 |
| **H4 Hound of the hunt** (§6.2) | the `hound` creature, the `hound_of_the_hunt` spell, the talent; `ally_mod` on Master of the hunt | none (content on the summon kind) | P1, Allies A2 (merged) |
| **P4 The calling content** (§7) | the Ascmearc's `callings/hunter.toml`, Wat's line, the drake's corpse checked, the showcase's calling; the integration test | none | P1, H1–H4 |
| **P5 Wiki, spec, docs** (§9) | the Prestige classes section and its generated tables, the curve, the spec bullets, the docs' lines, a cross-check of the spec against what landed | none | all |

**Why this order**: P1 is the foundation and ships with a fixture prestige so it is tested whole before the Hunter exists; P2 is a data change gated on P1's cap; H1–H4 run beside each other after P1 (H2 after H1 for `quarried`; H3 and H4 touch neither). P4 is the last content, so a run can play from Beorggeat to a Hunter. Each issue answers the feature template's "Talents and class abilities": **none new**: a prestige class is a class, its abilities class abilities; the Hunter's entries extend `combat`, `awareness`, `traps`, `status`, `allies`, `weapon_skills`; no new `[[system]]`, no pickable minted.

## 11. Rejected ideas, with the reason

| Idea | Why not |
|---|---|
| **Multiclassing** (a class chosen per level, XP split) | the owner: "not going to be a general multiclass thing"; the total level is one and the class of a level follows from the cap |
| **The prestige class offered** (an NPC asks, a dialog to accept) | the owner: "achievable by finding a hidden mechanism"; a calling is set as a quest's stage is, and a hint line is the one breadcrumb |
| **A level reset** (Dragon Quest's level 1 again) | the owner: "bard retains all the abilities"; and the total level never falls but by Unmade |
| **Hunter 1 at total 31, the calling giving words alone** | finding the mechanism should do something at once; the calling is the first level (Q4 keeps the alternative) |
| **XP counted from the calling** (the level-31 threshold measured from the XP at the calling) | needs saved state and takes from the player what the game said they had; banking is no state and rewards a late find |
| **The condition stated in the game** (a quest log line, a screen) | the owner: hidden; the wiki states it (the game is vague, the wiki is accurate) |
| **A new hidden location as the door** (a hunters' camp in Holegnwudu, reserved in the campaign design) | a hand-made level, a legend entry, a reveal rule, and an NPC for one calling; the drake exists, is the Ascmearc's greatest quarry, and eating its corpse is one line to record. The camp stays reserved for a later design |
| **Slaying the drake as the fact** (`slain` or `killed`) | the quest sends everyone to slay it: a reward for the quest, not a secret. Eating the heart is the step nobody is told to take (Sigurd's) |
| **A hunter's pool** (Patience, Breath) | a resource is a system with talents on its hooks (policy step 4); the Hunter's actives are one-at-a-time (the quarry), capped (snares), or a spell (the hound): none needs a pool, and the owner ruled out dual costs |
| **The Hunter's own kit, stats, or starting skills at the calling** | a prestige class is reached at 30 with a character already made; what it hands out is its table's (Hunter's trade's ranks) |
| **Sneak attack or Lurk for the Hunter** | the thief's (the surprise design's decision 7); the Hunter strikes from the open and ends what it marked |
| **A fear effect** (the quarry's death frightens its kind) | Hound's bay is the one fear source (the bard design's reasoning) |
| **Favoured prey for the Hunter** | the ranger's; a clone (rule 3). Quarry reads a mark, not the tally |
| **A permanent animal companion** | the allies system has summons (an hour) and no lifelong companion yet; the hound as a summon is one spell file. A companion is a design of its own |
| **Trophies** (a lasting bonus per quarry kind killed) | a growing table of bonuses is bookkeeping the game would have to explain, and Master of the hunt gives the kind at 30 instead |
| **Information abilities** (the quarry's exact hit points, its level) | rule 8; Scent and Track are senses that show a creature, as Blood scent does, and Killing stroke reads the quarter without telling it |
| **Levels 61 to 100 given hit points and mana** | the owner: "we are not going there yet"; `hp_level` and `mana_level` stay, and the owner decides when a class gets there |

## 12. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| Q1 | **The calling is a campaign's** (`campaigns/<id>/data/callings/`), the class shared (`data/prestige/`)? | **yes**: the world decides how it calls; the class is the game's |
| Q2 | **The fact is `ate = "fire_drake"`** (the heart eaten), not the drake slain? | **the heart**: the quest already asks for the slaying; the secret is the step after |
| Q3 | **Banked XP** turns into levels at the calling (no state)? | **yes** (§4.3) |
| Q4 | **Hunter 1 comes with the calling** (total 30; Hunter 30 at total 59), rather than Hunter 1 at total 31 (Hunter 30 at 60)? | **with the calling**: the find does something at once; if the owner prefers 60 as the bard-hunter's last level, `levels = 31`'s first row is the calling's and the table shifts by one |
| Q5 | **Hit points, rolls, and bonus picks by the class of the level**; talent picks by the total level; casting and the experience percent the base's for life? | **yes** (§4.4) |
| Q6 | **The hound is a summon spell paid in mana** (the bard's), not a free active on a cooldown status? | **mana**: the summon kind exists and prices itself; a martial Hunter from another base class would need a cooldown (`horn_spent`, an hour), one line of data then |
| Q7 | **Hunter's trade to rank 3** in bows, crossbows, slings, thrown, and spears? | **yes**: one short of the first weapon ability, earned by use |
| Q8 | **A snare holds six degrees** (36 s), two standing, three at rank 2 with a degree of Bleeding? | **yes**; all data |
| Q9 | **Master of the hunt's kind** reads the type of any Marked creature on the level (not Scent)? | **yes**: the mark is smelt, the kind is known |
| Q10 | **One hint line** (Wat's) and nothing else in the game? | **yes**; the wiki states the rest |
| Q11 | **`from` stays `["bard"]`** for now? | **yes**: the owner asked for the bard's; the list is data |
| Q12 | **The curve to 100 with `hp_level` 50 and `mana_level` 60 unchanged**? | **yes**: "not going there yet" |
| Q13 | **The death screen says "a bard and hunter"**? | **yes** |

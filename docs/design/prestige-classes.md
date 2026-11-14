# Warlock — Prestige classes: the oath, the Strange Spirit, the Hunter, and the curve to 100

**Built; this file is history.** Current behaviour is in spec §7 "Prestige classes and callings" and `docs/dev/experience.md`.

Design document for the owner's review (2026-11-02; issue #819, the bard's second and **third**
rounds on PR #835). Written against `origin/main` at 38c5085d (Allies A1–A3 and the summon spell
kind, #837; saves pinned to the game version that wrote them, #840; the Monk renamed Ascetic and
the barbarian Berserker). Nothing here is implemented; §12 cuts it into issues. It is the second
half of `docs/design/bard-jack-of-all-trades.md` (the bard's table compressed into levels 1 to 30
and capped there): this document is the system that lets a capped class go on, the one prestige
class the owner asked for, the typed-words dialog its calling needs, and the level curve extended
to 100. It builds on the level curve and `Factor` (`docs/dev/experience.md`), class tables and the
coverage policy (`docs/dev/progression.md`, rules 1–9 of #776, binding), dialogue and the dialogue
screen (`docs/dev/npcs.md`, `docs/dev/screens.md`), overworld encounters (`docs/dev/encounters.md`),
alignment (`docs/dev/alignment.md`), equipment (`docs/dev/equipment.md`), the strike and Protection
(`docs/dev/combat.md`), weapon ranks (`docs/dev/weapon-skills.md`), magic items and the uniques'
powers (`docs/dev/magic-items.md`), vaults (`docs/dev/mapgen.md`), and saves (`docs/dev/saves.md`).
Every number is data and hidden in the game; the wiki states it.

**The owner's second-round direction** (PR #835, 2026-11-02, verbatim): *"Also, let's do something
funky. Bard gains all these abilities by level 30. Bard does not gain levels above 30. With certain
conditions, a character can start advancing in another class. this is not going to be general
multiclass thing. More like prestige classes achievable by finding a "hidden" mechanism. Also
implement one prestige class for bard. condition: lvl30, certain event has happened. Call this
class "hunter". it will be a martial class with 30 levels. bard retains all the abilities when it
starts progressing in that class. XP requirements will be based on total level. Extend the XP
tables all the way to 100 - we are not going there yet, but who knows, maybe we will. This might
make "bard" pretty op, but who cares."*

**The owner's third round** (PR #835, 2026-11-02, verbatim), which replaces the second draft's
calling (the fire drake's heart eaten) and its Hunter (Quarry, Snare, the hound) whole: *"you
misunderstood the hunter class. For the calling, we need a special kind of dialog, where player
can type words (kinda like ADOM has when talking with some npc). Calling will be completed with
following steps and requirements: Bard level 30. This adds special encounter "Strange Spirit"
into over world encounter pool. Good characters can talk with the encounter spirit (not hostile).
They need to say following phrase to the NPC: "Life before death" (ignores capitalization and
punctuation) Spirit will echo the full oath: Life before death. Strength before weakness.
Journey before destination. I will protect those who stand in darkness. I will strive to mend
what is broken. And when I cannot see the light, I will take the next step. then the spirit
disappears and applies the class to player. Player receives enough experience to immediately
receive level up to the first level of the new class. Fire drake dungeon 2nd to last level should
contain a hint on some statue inscription or something. Class should just have regular martial
progression with no abilities above level 1 for now. Level 1 ability: ability that summons a
magical melee weapon of chosen base type -> use the ability -> open dialog to pick base -> the
weapon gets equipped. if the weapon becomes unequipped, it is removed instead of placed somewhere
other than the equipment slot. can be resummoned. ability weapon: decent +hit bonus, ignores PV
of targets, better crit range."*

## Contents

1. Summary
2. What the direction rules in and out
3. Research: typed words, oaths, summoned blades, and prestige classes elsewhere
4. The system: total level, class caps, the calling, what carries across
5. The level curve to 100
6. Saying words: the typed-words dialog
7. The Strange Spirit and the oath: the Hunter's calling in the Ascmearc
8. The hint: a statue on Fyrdelf's eleventh floor
9. The Hunter and the spirit weapon
10. What the player sees; determinism, saves, replays
11. The wiki, the spec, and the docs
12. Implementation plan
13. Rejected ideas, with the reason
14. Decisions: the owner's third round, applied
15. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **One level, two classes** | a character has one **total level**, read off the one curve from their XP as today (`Experience::level_for`), capped by `Character::level_cap`: the base class's `max_level` (the bard 30; every other class 60, the default), plus a prestige class's `levels` once called. Levels up to the base cap are the base class's; each level past it is the prestige class's: the calling at total 30, **Hunter 1 at total 31**, Hunter 30 at total 60. §4.1 |
| **The calling** | a prestige class is never offered or chosen: a character is **called** by saying the right words to the right creature (§6, §7). A campaign's `[[calling]]` names the class, the base classes it is open to (`from`), and the total level asked (`level`, the base cap); a dialogue's `[[dialogue.word]]` whose `does = { calls = "hunter" }` makes it when the phrase is said and the calling is **open** (the base class in `from`, the level met, no prestige yet). The character keeps everything and is given **exactly the XP the next level costs** (the owner): the level-up to Hunter 1 comes at once, and every level banked past the cap with it. §4.2, §4.3 |
| **Saying words** (new, a system part any NPC may use) | the dialogue screen gains a fixed `s` row, **Say something**, where a dialogue has `word`s: a text field (the Name step's, made shared) takes up to 40 characters; Enter sends `say:DIR:WORDS`. Matching **ignores case and punctuation** (`words::normalise`: lower-cased, letters and digits kept, the rest to single spaces); a match says the word's `reply` lines and does its `does`; anything else the dialogue's `unheard` line. A chat's time either way (time is the penalty; no lock-out). No new `KeyCommand`. §6 |
| **The Strange Spirit** | a peaceful overworld encounter entry of the Ascmearc's road, vale, downs, and woods tables, `when = { calling = "hunter" }` (a new `when` field: the calling is open to this character; so a bard of 30 not yet called, and nobody else), `group` one `strange_spirit` (a new creature of the `spirits` faction, hostile to no one, no corpse, no XP). Its dialogue `speaks_to = { min = "kind" }`: a character whose band is Kind or better hears its greeting and may say words; anyone else hears "The spirit looks through you as if you were not there." and nothing more. Said **"life before death"**, it echoes the six lines of the oath, departs ("The spirit fades, and is gone."), and the calling is made. §7 |
| **The hint** | Fyrdelf's **floor 11** (of 12; the drake's hoard is 12) pins a small vault, `hunters_statue`, with a statue cell whose inscription hints the phrase in the world's register. The vault's note survives stamping onto a generated floor (today a vault's `name`/`note` are dropped: `mapgen/vault.rs::stamp` copies no places), the statue a new `statue` terrain with `announce = true`, so arriving beside it says "There is a worn statue here." and the inscription. §8 |
| **The Hunter** (martial, 30 levels, from the bard) | `hp = 7` (the warrior's), 2 rolls, known for Athletics and Survival, bonus picks at 5, 10, 15, 23, 28 from Weaponcraft and Guard; **one ability, at level 1**, nothing above it for now (the owner): **Spirit weapon**, `use:spirit_weapon:BASE`: the one choice list of the 22 melee weapon bases, grouped by weapon skill; the weapon appears in the hand (what was wielded goes to the pack, the ordinary swap), priced as a wield. It is **+3 to hit, ignores Protection** (`ALL_PROTECTION`, Sunder's rule), and **threatens a critical 2 points lower**: numbers on the talent's effect, data. Whenever it leaves the hand, by any path (take off, swap, drop, sell, a tithe, the forge's ruin), it **fades** instead of going anywhere; it cannot be thrown. Resummoned at will, a wield's time, no cost, no cooldown. §9 |
| **The curve to 100** | `max_level = 100` (`MAX_LEVEL` 99 → 100): the same `[kills]` formula carried on (level 100 costs 485 kills of one's own level; 9,966,934 XP in all), `[levels] class_max = 60` the default class cap, `[[zero_difference]]` one more every five levels to 100 (25), `hp_level` and `mana_level` unchanged. Nobody reaches past 60 until a class or a prestige says so in data. §5 |
| Coverage policy | no new `[[system]]`: a prestige class is a class (its abilities are class abilities under `class = true`); saying words extends `talk`; the spirit weapon extends `combat` and `magic_items`; the calling's encounter extends `travel`. No pickable minted. Rules 1–9 checked in §9.3. |
| Slices | P1 the system (the cap, `Character::prestige`, the per-class level reads, `World::call`, the dev `call`), P2 the curve to 100, W1 saying words (the dialog, the action, `word`s and `speaks_to`), C1 the calling content (the spirit, its encounter entries, the oath; the `calling` encounter condition), C2 the hint (the `statue` terrain, vault notes stamped, the vault and its pin), H1 the Hunter and the spirit weapon, P5 the wiki, the spec, and the docs. §12 |

**Key choices, and why**

1. *Not multiclassing: a second table on the same level.* The owner: "not going to be a general multiclass thing." A character never splits XP, never chooses which class a level goes to, never has two of anything to track: the total level is the level, and the class of a level follows from where it falls. A prestige class is a class table that starts where the base's ends.
2. *The calling is said, not found on the ground.* The owner's third round: the player types the phrase to a spirit that only comes for a bard of 30 who is good. Saying words is a general piece (any dialogue may have `word`s: a password at a gate, a riddle's answer, a name to a ghost), and the calling is one thing a word may `do`. No facts tracked, no `once` flags, no eating: the second draft's six `when` shapes are gone.
3. *The calling is campaign data; the class is shared data.* The Hunter (`data/prestige/hunter.toml`) is any campaign's to call; **how** a world calls it is that world's (`campaigns/ascmearc/data/callings/hunter.toml`, the spirit's dialogue and encounter entries): a campaign without a calling has no Hunter, and the test world's calling is a fixture creature that says the same words.
4. *Hunter 1 comes by a level-up, at total 31.* The owner: "receives enough experience to immediately receive level up to the first level of the new class." So the calling grants the XP the next level costs (if the bank does not cover it already) and the level-up follows at once, with its hit points, rolls, and the new class's level 1: the character sees "Welcome to level 31." and "You gain Spirit weapon." in the same breath. Hunter 30 is total 60, the same last level as everyone's.
5. *Banked XP, no state.* The rule "XP beyond the cap counts but buys nothing" exists; the calling spends what was banked (a bard called at 700,000 XP goes to 33 at once) and tops up only a bank too small for 31. Nothing is saved for this.
6. *The weapon is the class, for now.* The owner: "regular martial progression with no abilities above level 1 for now." The Hunter's table is one row; the rest is hit points, rolls, and bonus picks like a martial class's. A table of further abilities is the owner's to ask for later; nothing is minted to fill levels.
7. *The spirit weapon is a stack with one flag and a talent's numbers.* It is an ordinary `Stack` of its base (so every rule that reads a wielded weapon reads it: skill, style, verbs, damage type, weight) with `spirit = true`, and its three gifts are the Hunter's talent effect read where the wielded weapon's profile is built. No affixes (they roll and have tiers and names), no unique (uniques are placed once), no new object files (22 bases would be 22 copies).
8. *Found, not given.* The owner: "achievable by finding a hidden mechanism." Nothing in the game names the condition; the statue is the one breadcrumb, and the wiki states the truth (the owner's rule).

## 2. What the direction rules in and out

| The owner said | Read as |
|---|---|
| "Bard gains all these abilities by level 30. Bard does not gain levels above 30." | the bard's table compressed (the bard design §5) and `max_level = 30` on the class |
| "With certain conditions, a character can start advancing in another class." | a second class table the character's levels go into once a condition holds |
| "this is not going to be general multiclass thing" | no choosing a class per level, no XP split, no second progression bar, no two base classes; the base class is for life and its table is kept whole |
| "More like prestige classes achievable by finding a 'hidden' mechanism" | D&D's prestige classes (entered by prerequisites) crossed with a secret: the condition is a thing done in the world that nothing announces |
| "Call this class 'hunter'. it will be a martial class with 30 levels." | the Hunter (§9), from the bard only for now (`from = ["bard"]`; a list so the owner may open it later), 30 levels, a fighter's hit points and picks |
| "bard retains all the abilities when it starts progressing in that class" | nothing is taken away; the bard's talents, skills, songs, mana, casting, and kit stay; the Hunter's table adds |
| "XP requirements will be based on total level" | one curve, one XP total, the next level's threshold read at the total level; a prestige class has no experience percent of its own |
| "Extend the XP tables all the way to 100" | `max_level = 100` and the formula carried on (§5); no class reaches it yet |
| "a special kind of dialog, where player can type words (kinda like ADOM has when talking with some npc)" | a text field inside the conversation: the player types, the NPC answers what it knows or does not (§6); ADOM's free-text prompt with some NPCs is the model |
| "Bard level 30. This adds special encounter 'Strange Spirit' into over world encounter pool." | an encounter entry whose condition is the calling being open to this character (§7); the pool is the Ascmearc's tables |
| "Good characters can talk with the encounter spirit (not hostile)." | a peaceful entry (anyone may hail it; it fights nobody); its dialogue answers only a character of a good band, the rest see it and are not seen (§7) |
| "They need to say following phrase ... (ignores capitalization and punctuation)" | one `word` on the spirit's dialogue with `phrase = "Life before death"`, matched normalised (§6.2) |
| "Spirit will echo the full oath ... then the spirit disappears and applies the class to player." | the word's six `reply` lines, then `does = { calls = "hunter", departs = true }` (§7) |
| "Player receives enough experience to immediately receive level up to the first level of the new class." | the calling grants the XP the next level costs; Hunter 1 is total 31 (§4.3) |
| "Fire drake dungeon 2nd to last level should contain a hint on some statue inscription or something." | Fyrdelf 11 pins a vault with an inscribed statue (§8) |
| "regular martial progression with no abilities above level 1 for now" | `hp`, `rolls`, `bonus_talents` as a martial class's; one table row (§9.1) |
| "summons a magical melee weapon of chosen base type -> use the ability -> open dialog to pick base -> the weapon gets equipped" | the Spirit weapon active with the choice list of the melee bases, then equipped by the ordinary swap (§9.2) |
| "if the weapon becomes unequipped, it is removed instead of placed somewhere other than the equipment slot. can be resummoned." | one `fade` on every path a stack leaves a slot; the ability again brings it back (§9.2) |
| "decent +hit bonus, ignores PV of targets, better crit range" | `+3` to hit, `penetration = ALL_PROTECTION`, threat lowered by 2, as data (§9.2) |

## 3. Research: typed words, oaths, summoned blades, and prestige classes elsewhere

| Source | What it does | Taken, and left |
|---|---|---|
| **ADOM**: chatting with some NPCs opens "What do you want to say?" and a typed line; the right word gets an answer, the wrong one a shrug; the Oracle's riddles and a few named NPCs work this way | the owner's model. **Taken**: a free-text line in the conversation, exact knowledge asked for, nothing offered as a menu (a menu would be the hint). **Left**: a bare prompt on every chat (here the field opens from a row the dialogue has only when it has words, so a shopkeeper is never asked for a password) |
| **NetHack** `#chat`; **Ultima IV–VII**: keyword conversations ("name", "job", and words learnt elsewhere typed to NPCs; a password at a gate); **King's Quest**, the text adventures: the typed password | the keyword-conversation tradition. **Taken**: the game never lists the words; the world teaches them (the statue). **Left**: typing as the way to ask ordinary topics (Warlock's are numbered rows and stay so) |
| **The Stormlight Archive** (Sanderson): the Knights Radiant's First Ideal, the oath the owner quotes word for word; a Shardblade summoned to the hand in ten heartbeats, cutting through armour as if it were not there, dismissed when it leaves the hand and called again; the spren that bonds a knight who lives the oath | the owner's source for the oath and the weapon. **Taken**: the spirit that hears the oath, the weapon called to the hand in a chosen shape, armour ignored, the vanish when it leaves the grip, resummoned at will. **Left**: the heartbeats (a wield's time), the second oath and beyond (one ability "for now"), Shardplate |
| **D&D 5e Warlock, Pact of the Blade**: a pact weapon of any melee form created in the hand as an action, dismissed at will, vanishing if more than 5 feet away for a minute; counts as magical | the nearest tabletop rule to the owner's weapon. **Taken**: any melee form chosen each time, created in the hand, vanishes when away. **Left**: the distance-and-minute rule (Warlock's is "leaves the hand") |
| **D&D 3.5 Soulknife**: a mind blade formed at will, +1 and growing, shaped as short sword, long sword, or bastard sword; dropped, it dissipates; **Hexblade**; **Spellsword** | the summoned-weapon class. **Taken**: the blade dissipates when dropped; reshaped on each summoning. **Left**: the growing enhancement (one rank for now; the owner may add) |
| **D&D 3.0/3.5 prestige classes** (DMG; Complete Adventurer): 5- and 10-level classes entered by prerequisites and often a **special** deed; levels add to the character level for XP and hit dice; the base class is kept | **Taken**: the shape (a table entered by a condition, on top of the base), total level for XP, the deed. **Left**: free switching between base and prestige levels (ruled out), the 5-level size (the owner: 30) |
| **D&D 4e paragon paths and epic destinies** (a second table at 11 and 21) | **Taken**: a second table on the same level. **Left**: a chosen path (the owner: found) |
| **Dragon Quest III** (Dhama's class change; the Sage needs the hidden *Book of Satori*); **Path of Exile's Ascendancy** (the Labyrinth) | **Taken**: a hidden thing in the world as the door; the character keeps what it was. **Left**: the level reset; a choice of three |
| **ADOM, DCSS, Elona**: no class change or prestige class | the roguelike precedent is none; Warlock's is one class, one secret, no reset |

## 4. The system: total level, class caps, the calling, what carries across

### 4.1 One level, two classes

| Rule | Decision | Where |
|---|---|---|
| The total level | as today: `Experience::level_for(xp, factor, cap)` off the one curve (to 100, §5), `Factor` the race's and the **base** class's percents; now **capped at `cap`**, `Character::level_cap(content)` = the base class's `max_level` + the prestige class's `levels` if called | `content/experience.rs`, `world/character/experience.rs::player_level`, `character.rs` |
| The class cap | `ClassDef::max_level` (new; `classes/README.toml`), default `[levels] class_max` of `data/experience.toml` (60, so no other class changes); the bard's file says `max_level = 30`. Checked at load: 2 to `max_level`, and every `abilities` row and `bonus_talents` level within it | `content/class.rs` |
| Which class a level is | `Character::class_of(level) -> (class id, class level)`: `level <= base.max_level` → the base at `level`; else the prestige at `level - base.max_level`. So total 31 is Hunter 1, total 60 Hunter 30 (key choice 4; the second draft had Hunter 1 at the calling, total 30: the owner's XP rule moves it) | `character.rs` |
| A prestige class | `PrestigeDef` from `data/prestige/*.toml` (`[[prestige]]`: `id`, `name`, `description`, `levels` (1 to 60), `hp`, `mana` (default 0), `rolls`, `known_for`, `bonus_talents`, `abilities` at its own levels 1 to `levels`): a class in everything a level reads, and nothing a creation reads (no `stats`, `kit`, `skills`, `weapon_skills`, `styles`, `casting`, `affinity`, `experience`, `alignment`, `creation_talents`, `optional_skills`). Its ids are its own: a prestige and a class may not share one. Its abilities are talents of the `class` tree, `granted`, checked by `check_table` like a class's (the coverage check counts them as class abilities) | `content/prestige.rs` (new, after classes and talents load) |
| The character's | `Character::prestige: Option<String>` (saved, hashed, left out while `None`): set by the calling, never cleared (nothing takes a class away; Unmade's lost levels, `Progress::peak`, lower the total and so the prestige level, and give back as they return) | `character.rs`, `save` |
| Only one | a character is called once; `World::calling_open` is false once `prestige` is set (a prestige class is never a `from`). `from` is a list so another base class may be let in later by data | `world/classes/calling.rs` |

### 4.2 The calling

```toml
# campaigns/ascmearc/data/callings/hunter.toml (one `[[calling]]` a file; content/calling.rs)
[[calling]]
class = "hunter"      # a prestige class (data/prestige/)
from = ["bard"]       # the base classes it is open to
level = 30            # the total level asked: at least each base's max_level
message = "The oath is in you now, as a blade is in its sheath. You are a hunter."
```

| Rule | Decision | Where |
|---|---|---|
| What a calling is | the **door**, not the deed: which class, who may walk through it, at what level, and what is said. The deed is a dialogue word's `does = { calls = "hunter" }` (§6.2), and there may be several ways to the same door in a world (the test world's fixture says it to a hermit) | `content/calling.rs::CallingDef` |
| `level` | at least every `from` class's `max_level` (the loader refuses less: a prestige level before the base is full would give two classes a level); `class` a prestige id; `from` base class ids, each named by at most one calling of a campaign (two doors to two classes for one base would need a choice, which is not this design) | `content/calling.rs` |
| Open | `World::calling_open(id) -> bool`: no prestige yet, the character's class in `from`, the total level at least `level`. Read by the encounter condition `calling = "ID"` (§7) and by the word's `does` | `world/classes/calling.rs` |
| `World::call(id)` | **the one place a prestige class is given**: `Character::prestige = Some(class)`; `message` said; then the XP (§4.3) and `level_up(base.max_level, level_for(xp, factor, new cap))`, which brings every level the XP buys with its hit points, rolls, picks, and the class table's abilities, each in the class it falls in (§4.4). Refused (nothing happens, the caller says why) when not open. No draw of its own (`level_up`'s stat checks draw from the levels stream as any level-up does). Not on `from_state` | `world/classes/calling.rs` |
| Callers | the dialogue word (`words.rs::do_word`, §6.2); the dev action `call:ID` (`Action::CallDev`, no time; dev only): calls prestige class `ID` now whatever the level, refused with no prestige of that id, one already held, or a base no calling of the run's campaign names | `action.rs`, `world/classes/calling.rs` |
| The showcase | `campaigns/showcase/data/callings/hunter.toml` (`from = ["bard"]`, `level = 30`) and a `word` on the hermit's dialogue (`phrase = "life before death"`, `does = { calls = "hunter" }`, no `departs`): the integration test plays a bard to 30 with `xp:N`, walks to the hermit, says the words, and is a Hunter; the spirit and its encounter are the Ascmearc's alone | content |

### 4.3 The XP: exactly enough for the first level

The owner: "Player receives enough experience to immediately receive level up to the first level of the new class." A bard at 30 has at least 545,660 XP (at 100 %) and keeps earning; level 31 costs 589,892 in all. `World::call`: `let need = Experience::threshold_for(cap + 1, factor); if xp < need { add_xp(need − xp) }` through `World::add_xp` exactly (never `grant_xp`: no `xp_percent` on a gift), then the level-up. So:

- a bard called at 560,000 XP is given 29,892 and goes to 31 (Hunter 1) at once;
- a bard called at 700,000 XP (capped long, the bank full) is given nothing and goes to 33 at once (33 is 683,000; 34 is 732,000): the bank was never taken away (key choice 5), and nothing is saved for either case;
- the XP row showed no next level at the cap, as the top level's does today; after the calling it shows level 32's.

`World::gain_level` (`boons.rs`, Exalted's) sets the XP **to** the next threshold; it is not used here because it would lower a bank above it.

### 4.4 What a level brings, by the class it falls in

| Each total level | Today | After |
|---|---|---|
| Hit points | `max(1, class hp ÷ 2 + Toughness modifier)` to `hp_level` 50 (`World::level_hp`) | the same with **the class of the level** (`class_of`): a Hunter level brings the Hunter's `hp = 7` → 3 + Toughness; 51 and up Hardened's only (unchanged) |
| Mana | `max(0, class mana ÷ 2 + the better of Intelligence and Willpower)` to `mana_level` 60 | the class of the level: the Hunter's `mana = 0` → the modifier alone, so a bard-hunter's mana grows a little still |
| Skill rolls | class `rolls` + race + Intelligence | the class of the level's `rolls` (the Hunter's 2); `known_for` for the ladder step is the **union** of both classes' (a bard-hunter who took up Survival at 12 rolls it at the known-for die); the +1 for a class's other skills reads either class's `skills` (the Hunter has none: its `known_for` alone) |
| Talent picks | every `talent_every` total level (3) | unchanged: the total level's |
| Bonus picks | the class's `bonus_talents` at its levels | the class of the level's, at its **class** level (the Hunter's 5, 10, 15, 23, 28 are total 35, 40, 45, 53, 58, from Weaponcraft and Guard); the bard's 5, 10, 15, 23, 28 as before |
| Class abilities | `class::grant` up to the level, at rank 1 or the named rank | per class: the base's table to `min(total, max_level)`, the prestige's to its class level; on a restore both (a grown table reaches saves) |
| Stat checks | each stat once a level (`check_stats`) | unchanged |
| Casting, caster level | `casting` percent of the character level | unchanged: the **base** class's percent of the **total** level (a bard-hunter casts at three quarters of 60; the owner's "pretty op") |
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

Beside it: `[levels] class_max = 60` (the default `ClassDef::max_level`, so every class but the bard is where it was; a bard-hunter's last level is 60 too); `hp_level = 50` and `mana_level = 60` unchanged (levels 61 to 100 add nothing but Hardened's; the owner's call when a class gets there); `[[zero_difference]]` carried on at one more every five levels: 65 → 18, 70 → 19, 75 → 20, 80 → 21, 85 → 22, 90 → 23, 95 → 24, 100 → 25 (the table's shape from 30 on). `Experience::level_for` takes the cap (§4.1); `threshold`, `xp_for`, and `kills_for` read to 100 for the wiki. The wiki's experience table (`xtask/src/wiki/experience.rs`) prints all 100 and says which classes stop where. Tests: the pinned hash, `level_for` at a cap below the curve's end, the discount untouched (levels 2 to 11), the ZD rows.

## 6. Saying words: the typed-words dialog

A general piece of the talk system: an NPC may know words. The player types them; the NPC answers what it knows. The calling is its first use; a gate's password, a riddle, a ghost's name are the same mechanism with other `does`.

### 6.1 The data

```toml
# a [[dialogue]] of any campaign (data/dialogue/*.toml); content/dialogue.rs
[[dialogue]]
creature = "strange_spirit"
name = "Strange Spirit"
greet = "The spirit regards you, and waits."
lines = ["It waits.", "The air about it is very still."]
speaks_to = { min = "kind" }                       # an alignment Requirement (optional)
silent = "The spirit looks through you as if you were not there."  # outside speaks_to
unheard = "The spirit waits, and does not answer."  # words it does not know

[[dialogue.word]]
phrase = "Life before death"
reply = [
    "Life before death.",
    "Strength before weakness.",
    "Journey before destination.",
    "I will protect those who stand in darkness.",
    "I will strive to mend what is broken.",
    "And when I cannot see the light, I will take the next step.",
]
does = { calls = "hunter", departs = true }
```

| Field | What | Checks at load |
|---|---|---|
| `[[dialogue.word]]` | a word the speaker knows: `phrase` (1 to 40 characters as written; matched normalised), `reply` (1 to 9 lines of at most 72 characters, said in order, each a message and the last the speaker's line on the dialogue screen), `does` (optional) | the normalised phrase is not empty and no two words of one dialogue normalise alike; at most 9 words |
| `does` | what a match does, all optional, done in this order: `calls = "ID"` (a calling of the dialogue's campaign, §4.2; the word is **unheard** when the calling is not open, so a warrior saying the oath to a fixture gets the `unheard` line and nothing else), `departs = true` (the speaker leaves the level at once after the reply: `World::departs(id)`, the removal step of the allies' `World::ally_departs` (`world/allies/ally.rs`, a summon whose time is up) factored out so both use it: the actor taken off the level with no corpse, no pile, no XP, no kill judged, and an encounter party whose members are all gone forgotten), `quest = "ID"` (the quest given, as asking its topic would: `World::give_quest`; a riddle's prize) | `calls` a calling of the campaign whose `from` the dialogue could meet; `quest` a quest of the campaign; `departs` only on a creature's dialogue, not an npc id's unique (an npc that departs is a level edit) |
| `unheard` | the line for words the speaker does not know (default "NAME says nothing."); at most 72 characters | — |
| `speaks_to` | an alignment `Requirement` (`{ min, max }` bands, `alignment::Requirement`, the quests' `requires_alignment`): outside it a chat says `silent` (required with `speaks_to`) and offers **no topics and no Say row**; the lines, greeting, and mutters are not said. **This is an NPC's reaction, as `greet_evil`/`greet_good` are** (`docs/dev/alignment.md`: "alignment bars nothing" is about the player's own abilities, which stay unbarred; a creature choosing whom to talk to is the dialogue's) | `silent` present; the bands valid |

### 6.2 The rules

| Rule | Decision | Where |
|---|---|---|
| The action | `Action::Say { dir, words: String }`, text `say:DIR:WORDS` with the **normalised** words joined by `_` (`say:E:life_before_death`): normalisation leaves only letters, digits, and spaces, so `_` is free to join them; the parser splits on `_` and normalises again (idempotent). The hash takes the words, so a replay says exactly what was said | `action.rs` |
| Normalising | `words::normalise(s)`: Unicode-lower-cased, every char that is not alphanumeric (`char::is_alphanumeric`, so `ǣ` and `þ` survive) becomes a space, runs of spaces to one, trimmed. "Life before death!" and "LIFE, BEFORE... DEATH" and "life  before death" are one | `words.rs` (pure, tested) |
| `World::say_to(dir, words)` | a creature there (`World::partner`, the chat's), with a dialogue (`World::dialogue_of`), that talks (`chat`'s refusals: not on the overworld at map scale, not Silenced, not to a hostile); outside its `speaks_to`: the `silent` line, a chat's time. Else the dialogue's `word`s in order: the first whose normalised `phrase` equals the normalised words **and** whose `does` can be done (an unopen `calls` fails it) is the match: its `reply` lines said (each `World::say`; the last also `Event::Talked`'s line, so the dialogue screen reopens on it, as a topic's reply does), then `does` in order (`calls` → `World::call`; `quest`; `departs` last, after the reply, so the farewell is heard). No match: `unheard`. A chat's time (`chat`, 500) either way: "time is the penalty", no count of tries, no lock-out (the owner's no-anti-farming rule). Marks: none (Bluff and Music have nothing to do with knowing a word) | `world/words.rs` (the one place words are said; one line in `World::apply`) |
| The dialogue screen | `DialogueScreen` lists, under the numbered topics, a fixed row **`s Say something…`** when the speaker's dialogue has `word`s and the player is inside its `speaks_to` (`TopicView` gains nothing: `Conversation::words: bool`, set by `World::answer`); `s` opens the field in the same panel: the question "What do you say to the Strange Spirit?" and the text field below it; Enter sends `ScreenAction::Act(Action::Say { dir, words })` (the screen closes, as asking does; the reply reopens it); Esc goes back to the topics. The hint line names it: `1–9 ask · s say · Esc back`. **A fixed key, no `KeyCommand`** (CLAUDE.md: menus and screens are never rebindable). The digits still ask topics; `s` is free in the screen today | `ui/dialogue_screen.rs` |
| The text field | **shared**: `ui/text_field.rs::TextField { text, max }` extracted from `NameScreen` (`takes(ch)`: a drawable, non-control character, no leading space, under `max`; `Edit::Erase`; `Edit::Confirm`; `Edit::Back`), drawn with the cursor as the name field is; the name screen keeps its Random key and `MAX_NAME` 20 on top of it. The say field's `max` is `words::MAX_SAID` 40 (the longest phrase a dialogue may have; a longer line cannot match anything). `Context::Text` bindings as today (Enter confirms, Backspace erases, Esc backs; every other key types), so `--keys "s L i f e Enter"` types in a capture | `ui/text_field.rs` (new), `ui/name_screen.rs` |
| Refusals | no creature that way, one without a dialogue, a hostile, the overworld at map scale, Silenced: `chat`'s words, no time. An empty line (Enter on nothing) closes the field and sends nothing | `words.rs` |
| Who hears | nothing is heard by others: saying words is a chat (`Act::Chat`'s noise, as a chat makes); the reply lines are the speaker's and shown to the player only | — |

### 6.3 Why a row and a field, not a bare prompt

ADOM asks on every chat with the NPCs that take words; Warlock's chat opens a screen of topics, and a password field on every shopkeeper would be a tell (and a nuisance). The row shows only where there are words to say, which is itself a small hint ("this one listens"), and that is right: the player who has read the statue knows what to do with it, and the one who has not learns that the spirit is waiting for something. The field is the Name step's, so the game has one way to type.

## 7. The Strange Spirit and the oath: the Hunter's calling in the Ascmearc

| Piece | Proposal |
|---|---|
| **The creature** | `campaigns/ascmearc/data/creatures/strange_spirit.toml`: "Strange Spirit", glyph `S` in a pale blue-white, level 1, `xp = 0`, `base_hp` 1, faction `spirits` (new in the campaign's factions: hostile to nobody, nobody hostile to it; so the encounter loader's "a peaceful entry of types not hostile to the player" holds), `alignment = "good"`, `corpse = false` (the creature field that exists: it leaves no corpse), speed 100, no attack, `budge = never`. Not incorporeal: it stands on the road and waits. It never fights; struck, it dies as any peaceful creature dies, leaving no corpse and no XP, and the killer's alignment pays `kill_peaceful` (`judge_kill`); its party is then forgotten and the entry may be met again (the owner's no-lock-out rule). No flag is minted to make it unkillable (§13) |
| **The dialogue** | §6.1's example, in `campaigns/ascmearc/data/dialogue/spirit.toml`: `speaks_to = { min = "kind" }` (the good side of the scale: Kind, Good, Saintly, `Band::side()`), `silent`, `unheard`, the one `word` with the oath's six lines as `reply` and `does = { calls = "hunter", departs = true }`. No topics (nothing to ask; the Say row is the whole conversation). `greet` "The spirit regards you, and waits."; two `lines` for the second and later chats |
| **The encounter entries** | one `[[table.entry]]` in each of `roads.toml`, `vale.toml`, `downs.toml`, and `woods.toml` (the day tables of the open country; not the deep woods, the fen, or the Ascdun's hostile ground: a spirit of the road and the open sky): `id = "strange_spirit_<table>"`, `name = "a strange spirit"`, `weight = 2` (rare: the pedlar is 3 among 10-odd rows; a bard of 30 crosses the Ascmearc often), `stance = "peaceful"`, `group = [{ creature = "strange_spirit", count = "1" }]`, **`when = { calling = "hunter" }`** |
| **The `calling` condition** | `When::calling: Option<String>` (encounters' `when`; checked at load as a calling of the campaign): holds when `World::calling_open(id)`. One field carries class, level, and "not yet called": a bard below 30 never meets it, a warrior never, a Hunter never again. Future prestige classes: an entry with their calling's id, nothing else |
| **What the player sees** | the encounter dialog as for any peaceful entry: "You meet a strange spirit." with Hail and Pass by (`Choice::{Hail, Pass}`); hailing opens the area with the spirit a few cells off, standing still (its `Mind::post` where it stands; `budge = never`: nobody swaps places with it). Look: "a strange spirit (unhurt), who waits" (its dialogue's name; `look.rs` adds "who waits" for a dialogue with words and no services: the one new word in look). Chatting (`C`): a good character hears the greeting and the dialogue screen shows no topics and the `s Say something…` row; a neutral or evil one hears `silent`, and the screen does not open (no topics, no words: as a chat with no topics today) |
| **The oath** | `s`, "life before death" in any case and punctuation, Enter: the six lines, one message each, the dialogue screen reopening on the last; then the calling's `message` ("The oath is in you now, as a blade is in its sheath. You are a hunter."), the XP, "Welcome to level 31." (and more, with a bank), "You gain Spirit weapon.", and the spirit's departure, "The spirit fades, and is gone." (`departs` after everything). Its party is forgotten; the entry's `when` is false from now on |
| **Wrong words** | `unheard`: "The spirit waits, and does not answer." A chat's time. The spirit stays for the visit; leaving the area and coming back meets the same spirit (a known party, `KNOWN_LINE`) until it is called or killed |
| **Why good only** | the owner. In the rules it is the dialogue's `speaks_to`, so a bard who has fallen to Neutral sees the spirit and is not seen; a good deed later (the alignment system's) brings the band back and the spirit speaks: nothing is lost by a drop in band (the alignment doc's principle) |
| **Other callings** | none proposed. A later class is a `[[prestige]]`, a `[[calling]]`, a creature or an npc with a `word`, and an encounter entry or a legend cell |

## 8. The hint: a statue on Fyrdelf's eleventh floor

Fyrdelf (`campaigns/ascmearc/data/dungeons/fyrdelf.toml`) is 12 floors: rooms 1–4, caves 5–12, the drake's hoard pinned on 12 (`vaults = ["drakes_hoard"]`). Floor 11 is caves, 120×60, nothing pinned. **No readable fixture exists today**: a terrain's `description` is per type and look mode shows it; a hand-made level's legend cell may carry a `name` and `note` (`Place`), which look mode shows, but `mapgen/vault.rs::stamp` copies no places, so a vault's note is lost on a generated floor. The cheapest honest hint, three small pieces:

| Piece | Proposal | Where |
|---|---|---|
| **A `statue` terrain** (shared) | `[[terrain]] id = "statue"`, name "worn statue", glyph `Ω` (or `&`; the owner's), stone, not walkable, see-through (a statue is a figure, not a wall), `announce = true` with `description = "Stone, old, and worn smooth where hands have touched it."`. Any vault or level may use it from now on; it says nothing of hunters | `assets/data/terrain.toml` |
| **Vault notes stamped** | `Vault`'s template keeps its legend's `name`/`note` per cell (`Place`s, as a level's does); `stamp` writes them into the generated level's `places` at their stamped positions (today `places: BTreeMap::new()` in `mapgen/mod.rs`). `feature_line` (`world/fixtures/feature.rs`), which says "There is a worn statue here." for an `announce`d terrain the player arrives **on**, learns one thing: for an announced terrain that is not walkable, it speaks when the player arrives **beside** it (every announced terrain today, the altars, the anvil, the spring, is stood on; the statue is the first one cannot stand on), and appends the cell's `note` after the description. So stepping up to the statue says: "There is a worn statue here. Stone, old, and worn smooth where hands have touched it. Letters are cut into the plinth: …" | `content/vault.rs`, `mapgen/vault.rs`, `world/fixtures/feature.rs`, `perception.rs::place` (already reads `places`) |
| **The vault** | `campaigns/ascmearc/vaults/hunters_statue.toml`: a 5×5 chamber off a cave passage, the statue in the middle on a raised floor, two wall sconces (`*`), nothing else (no loot: the hint is the treasure), `dungeons = ["fyrdelf:11"]`; pinned by `[[dungeon.levels]] from = 11 ... vaults = ["hunters_statue"]` so it is there for certain, as the hoard is on 12. The statue cell's `note` is the inscription | content |
| **The inscription** (the world's register: Fyrdelf is old, its dead older; an inscription in the Old English layer, given with its sense, as the campaign's names are) | *"Letters are cut deep into the plinth, in the old tongue: LĪF ǢR DĒAÞE. Below, in a later hand: 'Life before death. So the hunters swore, to the one who waited for them on the road; and the road gave them each a blade.'"* The statue itself: a hooded figure, hands empty, held as if about a hilt that is not there (the `note` may carry both: at most 280 characters, a topic reply's limit, else two cells) |
| **Why floor 11** | the owner: "2nd to last level". A bard of about 20 reaches it on the drake's quest and reads it before 30; by the time the spirit comes, the words are an old note in the player's head (the message history keeps them; the wiki states them) |
| **Why not** a scroll, a legend line, a rumour | a scroll is carried off and sold; a rotation line of an NPC is said to everyone and leaks the secret to a level-3 warrior at the inn; a `reveal` or quest line would make it a quest. A stone in the deep says it once to whoever goes down there, which is the owner's "hidden" |

## 9. The Hunter and the spirit weapon

### 9.1 The class

```toml
# data/prestige/hunter.toml
[[prestige]]
id = "hunter"
name = "Hunter"
description = "One who has sworn the old oath on the road and been answered: a fighter whose weapon comes to the hand when called."
levels = 30
hp = 7
rolls = 2
known_for = ["athletics", "survival"]
bonus_talents = { levels = [5, 10, 15, 23, 28], trees = ["weaponcraft", "guard"] }
abilities = [
    { level = 1, talent = "spirit_weapon" },
]
```

The owner: "regular martial progression with no abilities above level 1 for now." So: the warrior's hit points (`hp = 7`: 3 + Toughness a level), two rolls, bonus picks on the bard's compressed cadence from the warrior's trees (Weaponcraft and Guard), and **one** table row. No `[[prestige]]` field is minted for weapon ranks: a bard's Travelled hands (the bard design §8) has every weapon at rank 1 already, and ranks rise by use as any fighter's do. The table is the owner's to grow; nothing is written "to come".

### 9.2 Spirit weapon (level 1)

```toml
# talents/spirit_weapon.toml
[[talent]]
id = "spirit_weapon"
name = "Spirit weapon"
tree = "class"
granted = true
description = "Call a weapon of spirit-stuff into your hand in the shape you choose (an ability). It strikes surer and finds the gaps in any armour, and it goes back whence it came the moment it leaves your grip; call it again."
effects = [{ kind = "spirit_weapon", to_hit = 3, threat = 2, ignores_protection = true, phrase = "A weapon of spirit-stuff: it finds the gaps in any armour, and fades when it leaves your hand." }]
systems = ["combat", "magic_items"]
```

| Rule | Decision | Where |
|---|---|---|
| The ability | `ClassActive::SpiritWeapon`, `Ability::Class`, **`Aim::Shape`** (new, beside `Ailment` and `Item`: a pick from a list the view gives): the menu's row "Spirit weapon: a weapon's shape". Using it opens the one choice list (`ScreenName::Shape`, a `PagedList` like the equip choice and the Heal ability's ailments, grouped by weapon skill: Daggers, Short blades, Long blades, Great blades, Axes, Maces, Polearms, Spears, Staves), the rows `PerceptionView::weapon_shapes()`: every shared object of class `melee_weapons` that is no unique's base, by skill then id (22 today), each row its name and numbers as the player would hold it (`Lore::listed` of a plain one: `long sword (+0 hit, 1d9)`; the gifts are the ability's, said in its description, not the row's). A letter sends `use:spirit_weapon:ID`. **Why a list, not the one dialog**: the owner said "open dialog to pick base"; the one `Dialog` holds a handful of options in a column (quit, buy, cast anyway) and 22 rows in 9 groups is what the lettered lists are for (`docs/dev/screens.md`: every lettered list is the one `PagedList`); it is a dialog in the owner's sense, a question with answers | `ability.rs`, `ui/shape_screen.rs` (a thin `ObjectScreen` kind), `perception.rs` |
| The action | `Action::Shape(ClassActive, ObjectId)`, text `use:spirit_weapon:long_sword`. Parse rule for `use:ID:X`: `X` a direction (upper case) → `Use`; `X,Y` → `UseAt`; a lower-case letter or digits → `CastOn` (a stack); **else an object id** → `Shape`. Refused with no time: the id is not a melee weapon shape (ammunition, a launcher, a unique's base, a thrown-only weapon, unknown), the player has no `spirit_weapon` effect, both hands are bound by something that cannot be taken off (nothing today; the check is `Equipment`'s swap's) | `action.rs`, `world/classes/spirit_weapon.rs` |
| What it does | `World::call_weapon(shape)`: makes `Stack::spirit(shape)` (a `Stack` of the base object, count 1, `spirit = true`, no `magic`; **identified** in every sense: the maker knows it, `Lore::knows` true for a spirit stack), puts it into the pack and **equips it in the main hand** through the ordinary `World::equip` path: what was in the main hand (and the off hand, for a two-handed shape) is taken off into the pack as any swap does, with the swap's messages ("You put away the long sword." / "A blade of pale light is in your hand." the `start`); a spirit weapon already held **fades first** (so the ability reshapes: a dagger for the corridor, a great axe for the door). Priced as **`wield`** (1000, a base turn: the slot action a weapon costs to wield; `World::price_for`), nothing else: no mana, no pool, no cooldown (policy step 4; a free reshape every turn is a wield every turn, which is what it would cost to swap steel). A Dexterity mark as a wield gives? none: wielding marks nothing today | `world/classes/spirit_weapon.rs` |
| The gifts | read where the wielded weapon's profile is built, `Actor::melee` and `off_hand_blow` (a light spirit weapon in the off hand is one too): when the stack is `spirit` and the actor's `TalentMods::spirit_weapon` is set: `to_hit += 3`; `threat −= 2` (a long sword's 20 → 18: 2d12 of 18 or more threatens, 28 of 144 rolls where 20 gives 15; a dagger's 19 → 17); `penetration = ALL_PROTECTION` (1000; `WeaponDoes::IgnoreProtection`'s constant, `world/combat/weapon_active.rs`: the soak draws nothing, as against an unprotected target, so fights elsewhere keep their hashes). Thrown? it cannot be thrown (below). Kicks and bashes are not the weapon's. A creature never holds one (no creature has the talent; `gear` cannot name a spirit stack: `Stack::spirit` is set by `call_weapon` alone) | `world/turn/actor.rs` (`melee`, `off_hand_blow`), `TalentMods::spirit_weapon: Option<SpiritWeapon { to_hit, threat, ignores_protection }>` |
| Leaving the hand | **the invariant**: a spirit stack exists only in a hand slot. `Equipment::take_off` and `Equipment::after_removal` are the two primitives every path uses (`docs/dev/equipment.md`; there is no `World::unequip`): `remove:SLOT`, the swap in `equip`, `take_off_before_dropping` (drop, sell, trade), the outcomes' `remove_stacks` (the forgotten altar's tithe, the forge's ruin), a quest hand-in's `take_from_pack`. They become `World::take_off(actor, slot)` and `World::after_removal(actor, positions)` (thin wrappers, every caller moved: a mechanical change, one grep), and each ends with `World::fade_spirit(actor)`: any `spirit` stack in the pack and not in a hand slot is removed from the inventory ("The blade fades from your hand." / for a tithe or ruin, the outcome's own line and then "…and what was taken was never there."), positions followed. So a dropped spirit weapon is taken off (fades) and the drop drops nothing; a sold one the same (the shop never sees it); a swap for steel fades it; a tithe that takes it takes spirit-stuff (the altar's loss, not the player's: it is called again). **Thrown**: `World::throw` refuses a spirit stack with no time ("It will not leave your hand.") rather than fade it mid-flight. The player's death ends the run (nothing to pile). A save holds it in the hand (`Stack::spirit`, `skip_serializing_if` false, hashed); a run resumed holds it still | `world/items/equip.rs`, `world/items/carry.rs`, `world/people/shop.rs`, `world/fixtures/outcome.rs`, `world/people/quest.rs`, `world/combat/shoot.rs`; `world/classes/spirit_weapon.rs::fade_spirit` |
| Weight, value, material, temper | its base's, all of them (a spirit long sword weighs 1.8 kg and is steel for every rule that asks; Q6 offers weightless). Value: the shop never sees it (it fades on the sale) and the tithe may take it; no hook. Temper: as any weapon (the ruin fades it; an improvement improves the stack it has, until the next call makes a fresh one: the smith's work is lost with the shape, which is the price of a weapon that is not there) | — |
| Description | `describe::lines` adds a `LineKind::Power` with the effect's `phrase` for a spirit stack (the uniques' power line, reused), and its numbers show the gifts folded in (`magic::numbers`: `(+3 hit, 1d9)`; "Pierces armour" as a penetrating weapon's description says; threat is never shown, as today). The character screen's attacks row reads the profile the strikes use, so it shows +3 and "pierces armour" | `magic/describe.rs`, `attacks.rs` |
| Rank | one, for now. The effect's three numbers are where a second rank would raise them (`ranks = 2` and a `[[talent.rank]]`, the bard design's Q9 shape), should the owner give the Hunter more |

### 9.3 Rules 1–9

| Entry | 1 (capstones) | 3 (no clone) | 4 (ranks) | 5–7 | 8 (not information) | 9 (favour) | Tags |
|---|---|---|---|---|---|---|---|
| Spirit weapon | level 1: not a capstone; it pays in no gold, favour, quest, or XP | `spirit_weapon` is its own kind: no talent, class ability, or weapon ability makes a weapon; Sunder's `ignore_protection` is a weapon ability on a blow, not a weapon, and its constant is borrowed, not its effect | one rank | works on every floor; no flaw; no hidden marks | a weapon in the hand | — | `combat`, `magic_items` |
| The calling | — (no talent) | — | — | — | — | — | — |
| Saying words | — (a talk mechanic, no talent) | — | — | — | — | — | `talk` (the system's tag; no talent reads words) |

No new `[[system]]`: the owner's rule that a new broad system ships with a talent is met by extension: saying words is `talk`'s (Diplomat and the talk talents touch chat and prices; a talent on words would be information or a shortcut to a secret, which rule 8 and the "hidden" ask forbid); the spirit weapon is `combat` and `magic_items`; the encounter entry `travel`.

### 9.4 Numbers, for the wiki

A Hunter 1 (total 31), a bard with Travelled hands' Long blades 1 and whatever rolls went there, calls a long sword: 1d9 slashing or piercing, +3 to hit over the bard's own, criticals on an 18 or better (19 % of attacks threaten against 10 % with steel), and every point of the target's Protection ignored: an orc chieftain's mail (Pv 5, soaking 3 to 5 a hit) soaks nothing; an iron golem's hide the same. Against an unarmoured kobold the sword is +3 and keener and nothing more. A great axe shape for a door: 2d6 (its base's), two hands, +3, threat 18. Reshaping costs a turn each time.

## 10. What the player sees; determinism, saves, replays

| Where | What |
|---|---|
| The character screen | the Class row: `Bard` until the cap, `Bard 30` at it (the level shown beside the class once the class is full, so a capped bard sees the class is done without being told why), `Bard 30, Hunter 4` after; the Level row the total. The death screen's "Here lies Brakka, a bard" becomes "a bard and hunter" |
| The Characters list | the summary (`saves/<id>.toml`) gains the prestige: "Bard 30 / Hunter 4, level 34" |
| The XP row | no next level at a cap, as at the top today; after the calling the next as usual |
| The dialogue screen | the `s Say something…` row and the field (§6.2); text snapshots `dialogue-say-row` (a fixture creature with words), `dialogue-say-field` (`--keys s`), `dialogue-say-field-typed` (`--keys "s l i f e"`) |
| The encounter dialog | "You meet a strange spirit." Hail / Pass by (the peaceful dialog as today; snapshot `encounter-spirit` on a fixture table) |
| Messages | `silent` / the greeting / the six lines of the oath / the calling's `message` / "Welcome to level 31." / "You gain Spirit weapon." / "The spirit fades, and is gone." / `unheard`; "A long sword of pale light is in your hand." / "You put away the long sword." / "The blade fades from your hand." / "It will not leave your hand." / "There is a worn statue here. … Letters are cut deep into the plinth: …" |
| Look mode | "a strange spirit (unhurt), who waits"; the statue's description and note; the spirit weapon in a seen hand? only the player's: "a long sword" (gear names the base; nothing says spirit to onlookers) |
| The abilities menu | Spirit weapon under the class source, "a weapon's shape"; the shape list (snapshot `shape-list`) |
| Dumps | `stats`: `Class Bard 30, Hunter 4` (and `Level 34`); `talents`: `Spirit weapon (class)`; `inventory`/`equipment`: `a long sword (+3 hit, 1d9) (spirit)` (the mark as "(being worn)" is); `attacks`: `Main hand long sword: +7 to hit, 1d9+1 slashing, pierces armour`; `abilities`: `Spirit weapon (class): a weapon's shape`; `progress` (dev): `calling hunter: open` / `calling hunter: level 30 not met` / `called hunter at 30`; `actors` (dev): the spirit's `dialogue strange_spirit, speaks to kind+`; JSON `character.prestige`, `character.classes` (`[{ class, level }]`), `calling` (dev), `abilities[].aim = "shape"`, a stack's `spirit: true`; the `truth` layer marks the statue's cell with the terrain's glyph |
| Determinism | the calling draws nothing of its own (`level_up`'s stat checks from the levels stream); saying words draws nothing; the spirit weapon's call and fade draw nothing; the encounter entry rolls as any entry (the `encounters` stream; a run that never qualifies draws the same rolls, the entry's weight simply absent: **a bard of 30 changes the entry set of four tables**, so the merged-weight draw of those tables differs from a bard of 29's at the same roll, which is the condition working, not a leak). Runs without a bard hash as before but for the content |
| Saves | new: `Character::prestige` (left out while `None`), `Stack::spirit` (left out while false), `Conversation::words` is an event (not saved). The `strange_spirit` creature and the `spirits` faction and the `statue` terrain are content (terrain and factions are fingerprinted; a new terrain is a `FORMAT_VERSION` bump by the integrator, as any is). Saves are pinned to the game version (#840): **no shim** for a save from before |
| Replays | `say:E:life_before_death`, `use:spirit_weapon:long_sword`, the dev `call:hunter` are action text (CLAUDE.md's list); the calling is a consequence of the say |
| Content fingerprint | the `statue` terrain and the `spirits` faction; the prestige, the calling, dialogue words, and talents are not |

## 11. The wiki, the spec, and the docs

| Where | What |
|---|---|
| `wiki/src/classes.md` | a **Prestige classes** section: the rule (one level, the cap, the calling, what carries across, the XP at the calling), the Hunter (fantasy, the one-row table, the spirit weapon's numbers), and each campaign's calling stated plain, generated from the callings and the words that `calls` them ("a bard of level 30 of a good alignment who says *life before death* to the Strange Spirit met on the Ascmearc's roads, vale, downs, or woods becomes a Hunter": `xtask/src/wiki/classes.rs`) |
| `wiki/src/npcs.md` | a **Saying words** section: how it works, that case and punctuation are ignored, who knows words (generated: every dialogue with words, its speaker, and whom it speaks to; the phrases themselves stated, the owner's rule) |
| `wiki/src/experience.md` | the curve to 100 (generated), the class caps |
| `wiki/src/abilities.md`, `encounters.md`, `creatures.md`, `terrain.md` | Spirit weapon's row; the spirit's entries with their condition; the spirit; the statue |
| `spec/08i-character.md` | "Prestige classes" after "Class abilities": the total level, `max_level`, the calling and `World::call`, the XP given, `class_of`, what a level brings by class; the Hunter's bullet; "Experience": the curve to 100 |
| `spec/08b-creatures.md` | "Dialogue": words, `speaks_to`, `silent`, `unheard`, `does`; the spirit |
| `spec/07b-data.md` | `classes/README.toml` `max_level`; `prestige/README.toml`; a campaign's `data/callings/`; dialogue words; encounters' `calling`; terrain `statue`; vault notes; `talents/README.toml` `spirit_weapon` |
| `spec/06-input-and-ui.md` | the dialogue screen's Say row and field; the shape list |
| `docs/dev/experience.md` | the cap, `class_of`, `level_for` with a cap, the curve to 100, the XP at the calling |
| `docs/dev/progression.md` | "Class tables": prestige tables and the calling; the hook row (`spirit_weapon` → `Actor::melee`/`off_hand_blow`); rule 1's wording for a 30-level table (the bard design's Q19) |
| `docs/dev/npcs.md` | words: `words.rs`, `normalise`, the action, `speaks_to`, `silent`, `unheard`, `does`, the screen's row and field |
| `docs/dev/encounters.md` | `when.calling`; the spirit's entries |
| `docs/dev/equipment.md` | `World::take_off`/`after_removal` as the wrappers every path uses, `fade_spirit`; the invariant |
| `docs/dev/combat.md`, `magic-items.md` | the spirit weapon's three reads; `Stack::spirit` and its power line |
| `docs/dev/mapgen.md`, `content.md` | vault notes stamped; `statue`; the pin on Fyrdelf 11 |
| `docs/dev/campaign.md` | `data/callings/` is a campaign's; the cross-campaign check (a calling's `from` are shared classes; a word's `calls` a calling of its own campaign; an entry's `calling` the same) |
| `docs/dev/saves.md`, `character-creation.md`, `screens.md` | the new saved fields; `Character::prestige`; the shared `TextField`, the Say row, the shape list |
| CLAUDE.md | the action text (`say:DIR:WORDS`, `use:spirit_weapon:ID`, `call:ID`), the `stats`, `progress`, and `inventory` dump lines, `--screen shape` |
| Tests | `words.rs` (normalise: case, punctuation, runs of spaces, non-ASCII letters, idempotent); `content/dialogue.rs` (words' checks: empty, alike, too many, `does` naming nothing of the campaign, `speaks_to` without `silent`, `departs` on an npc); `content/prestige.rs` and `content/calling.rs` (loading and every refusal); `world/words_tests.rs` (a match in any case and punctuation, unheard, the time either way, `silent` outside the band and the row gone, `departs`, `quest`, `calls` unopen is unheard, no draws, a replay, a save); `world/classes/calling_tests.rs` (open and not, the XP given exactly and not when banked, the levels coming at once with the right class's hit points and rolls, "You gain Spirit weapon." at 31, both tables on a restore, the dev `call`, Unmade's lost level and its return, a replay, a save); `world/classes/spirit_weapon_tests.rs` (every shape and the refusals, the swap and the two-handed swap, the price, the three gifts in the profile and the soak drawing nothing, the fade on remove, swap, drop, sell, tithe, ruin, and quest hand-in, the throw refused, the reshape, a save holding it, a replay); `world/campaigns/ascmearc_encounter_tests.rs` (the spirit's entries only for an open calling, the good-only chat, the oath end to end from `xp:N` to Hunter 1 by replay); `mapgen/vault_tests` (notes stamped; Fyrdelf 11 has the statue in every seed's snapshot: the generated level's `places`); `content/experience.rs` (the hash to 100, `level_for` capped, the ZD rows); the text snapshots of §10 |

## 12. Implementation plan

Rules first, then content, then the wiki. The false-conflict trick (`docs/dev/merging.md`) for the hot files: `experience.toml`, `terrain.toml`, `talents/README.toml`, `ability.rs`, `action.rs`, `class.rs`, `content/dialogue.rs`, `content/encounter.rs`, `ui/dialogue_screen.rs`.

| Issue | What | Adds | Depends on |
|---|---|---|---|
| **P1 The system** (§4) | `ClassDef::max_level` and `[levels] class_max`; `Experience::level_for` with a cap; `PrestigeDef` (`content/prestige.rs`) and `CallingDef` (`content/calling.rs`, a campaign's `data/callings/`); `Character::prestige`, `class_of`, `level_cap`; every per-level read through the class of the level (hit points, mana, rolls, known-for union, bonus picks, `class::grant` per table); `World::calling_open`, `World::call` with the XP rule; the dev `call:ID`; the character screen's Class row, the summary, the dumps; a fixture prestige and calling for the tests | **saved state** (`Character::prestige`), a dev action | — |
| **P2 The curve to 100** (§5) | `max_level = 100`, `MAX_LEVEL` 100, the pinned hash, the ZD rows, `class_max = 60`; the wiki's experience table | none | P1 (the cap, so nobody reaches 61) |
| **W1 Saying words** (§6) | `words::normalise`; `Action::Say`; dialogue `word`s with `reply` and `does` (`calls`, `quest`, `departs`), `unheard`, `speaks_to` and `silent`, their load checks; `World::say_to` and `do_word`; `Conversation::words`; the shared `TextField` out of `NameScreen`; the dialogue screen's `s` row and field; `--keys` typing into it; snapshots; CLAUDE.md's action text | **Action** (`say`) | P1 (for `calls`; `quest` and `departs` stand alone, so W1 may land first with `calls` refused until P1) |
| **C1 The Strange Spirit** (§7) | the `strange_spirit` creature and the `spirits` faction; its dialogue with the oath; `When::calling` and its load check; the four entries; `World::departs` factored out of `ally_departs` for `departs`; look's "who waits"; the Ascmearc's end-to-end replay test; the showcase's calling and the hermit's word | none (content on W1 and P1) | P1, W1 |
| **C2 The hint** (§8) | the `statue` terrain (`announce`, see-through, not walkable); `feature_line` for an announced terrain beside the player and the cell's note; vault `name`/`note` kept and stamped into `places`; `hunters_statue.toml` and its pin on Fyrdelf 11; the inscription; the mapgen snapshot | none (terrain is fingerprinted: a format bump by the integrator) | — (stands alone) |
| **H1 The Hunter and the spirit weapon** (§9) | `data/prestige/hunter.toml`; the `spirit_weapon` talent and effect, `TalentMods::spirit_weapon`; `ClassActive::SpiritWeapon`, `Aim::Shape`, `Action::Shape`, `PerceptionView::weapon_shapes`, the shape list screen; `Stack::spirit`; `World::call_weapon` on the equip path, priced as `wield`; the three reads in `Actor::melee`/`off_hand_blow`; `World::take_off`/`after_removal` wrappers and `fade_spirit`, the throw refusal; the description's power line and numbers; the dumps; `spirit_weapon_tests` | **Action** (`use:spirit_weapon:ID`), **saved state** (`Stack::spirit`) | P1 (the table is read by the system); the bard's B3 (`rank` on `AbilitySpec`) not needed: one row at rank 1 |
| **P5 Wiki, spec, docs** (§11) | the Prestige classes and Saying words sections and their generated tables, the curve, the spec bullets, the docs' lines, a cross-check of the spec against what landed | none | all |

**Why this order**: P1 is the foundation and ships with a fixture prestige so it is tested whole before the Hunter exists; P2 is a data change gated on P1's cap; W1 is the one new talk action and stands nearly alone; C2 stands wholly alone and may go first; C1 needs the door (P1) and the words (W1); H1 needs P1. At most three of these add actions or save state (P1, W1, H1), the integrator's limit. Each issue answers the feature template's "Talents and class abilities": **none new**: a prestige class is a class, its ability a class ability; saying words extends `talk`, the spirit weapon `combat` and `magic_items`, the encounter `travel`; no new `[[system]]`, no pickable minted.

## 13. Rejected ideas, with the reason

| Idea | Why not |
|---|---|
| **Multiclassing** (a class chosen per level, XP split) | the owner: "not going to be a general multiclass thing"; the total level is one and the class of a level follows from the cap |
| **The prestige class offered** (an NPC asks, a dialog to accept) | the owner: "achievable by finding a hidden mechanism"; the spirit offers nothing and lists nothing: the player must know the words |
| **A level reset** (Dragon Quest's level 1 again) | the owner: "bard retains all the abilities"; and the total level never falls but by Unmade |
| **The second draft's calling: the fire drake's heart eaten** (`when = { ate = "fire_drake" }`, `Progress::once`, six `when` shapes, an omen line, Wat's rotation line) | replaced whole by the owner's third round: the oath said to the Strange Spirit. The six fact shapes go with it (nothing reads them); the drake's quest and corpse are untouched |
| **The second draft's Hunter table** (Quarry 1–3, Hunter's trade, Scent of the quarry, Hamstring, Snare, Killing stroke, Hound of the hunt, Master of the hunt; the `marked` status, the `hold` trap kind, the hound) | the owner: "you misunderstood the hunter class ... regular martial progression with no abilities above level 1 for now ... summons a magical melee weapon." Dropped whole; nothing of it is kept in reserve, and no "to come" rows are written |
| **Hunter 1 at the calling (total 30)** (the second draft's key choice 5) | the owner: the calling gives "enough experience to immediately receive level up to the first level": the first Hunter level is a level-up, total 31 |
| **XP counted from the calling** (the level-31 threshold measured from the XP at the calling) | needs saved state and takes from the player what the game said they had; the owner's rule tops up a short bank and keeps a full one |
| **A bare "What do you want to say?" prompt on every chat** (ADOM's) | a password field on every shopkeeper is a tell and a nuisance; the row shows where there are words |
| **Words as a menu** (the phrase listed as a topic) | the menu is the hint; the owner: typed |
| **Fuzzy matching** (a word off, Levenshtein, "life before death" as "life, death, before") | the owner: ignores capitalisation and punctuation, nothing more; the wiki states the phrase and the statue gives it |
| **A try counted or a lock-out after wrong words** | the owner's no-anti-farming rule; a chat's time is the penalty |
| **The spirit's condition on the entry as `level = { min = 30 }` and `class = "bard"`** | encounters have no level minimum (the owner's Q5 of the encounter design) and no class condition; `calling = "ID"` is one field that says exactly "the door is open to this character", and future callings reuse it |
| **An alignment condition on the encounter entry** (`alignment = { min = "kind" }`) | the owner: good characters can *talk* with it; the rest should see the spirit that will not see them (and a fallen bard sees what a good deed would open). `speaks_to` on the dialogue |
| **The spirit a unique npc placed on a level** (a shrine in Holegnwudu) | the owner: "into over world encounter pool"; and an encounter is met on the road, as the oath's third line says |
| **The spirit unkillable** (`departs_when_struck`, immunity) | a flag and a hook for one creature; 1 hit point, no corpse, no XP, and a peaceful kill's alignment cost do the same, and the entry comes again |
| **A spirit that stays (a party remembered after the calling)** | `departs`: it came for the oath; and its `when` is false once called |
| **The hint as a rotation line of an NPC, a scroll, a quest line** | said to everyone or carried off or made a quest; a stone in Fyrdelf's deep says it to whoever goes there (§8) |
| **The inscription as a per-type terrain description** (a `hunters_statue` terrain with the hint in `terrain.toml`) | the cheapest of all, but one campaign's secret in shared data, and the next inscription would be another terrain; a `statue` terrain plus vault notes stamped gives every vault a readable cell |
| **The shape list as the one `Dialog`** | 22 rows in 9 groups is a lettered list's job; the dialog holds a handful of options (§9.2) |
| **The spirit weapon as a unique or an affixed item** | a unique is placed once and named; affixes roll with tiers and names and show as "of the X"; the gifts are the class's and sit on its talent as data (key choice 7) |
| **22 spirit object files** (`spirit_long_sword`, …) | a flag on the stack reads every base as it is, and a new base weapon is a shape at once |
| **A mana or pool cost, a cooldown, or heartbeats for the summoning** | the owner: "can be resummoned"; policy step 4 (no cooldown besides a cost); a wield's time is the honest price of a weapon in the hand |
| **The weapon weightless, or worth nothing in a shop** | one hook each for a flavour the fade already gives (it is never sold); Q6 keeps weightless as the owner's option |
| **A fade on a timer or a distance** (5e's minute away) | the owner: "if the weapon becomes unequipped, it is removed"; the hand is the rule, and there is no timer on a carried stack to reuse |
| **Letting it be thrown and fade in flight** | a thrown weapon that vanishes is a free miss with no missile to pick up; refused instead |
| **A second rank now, or abilities at 10, 20, 30** | the owner: "no abilities above level 1 for now" |
| **Levels 61 to 100 given hit points and mana** | the owner: "we are not going there yet" |

## 14. Decisions: the owner's third round, applied

| The owner (PR #835, 2026-11-02) | Applied |
|---|---|
| "you misunderstood the hunter class" | §9 rewritten: one ability, the spirit weapon; the quarry table and the heart's calling dropped (§13) |
| "a special kind of dialog, where player can type words (kinda like ADOM has when talking with some npc)" | §6: the `s Say something…` row and the shared text field in the dialogue screen; `say:DIR:WORDS`; dialogue `word`s |
| "Bard level 30. This adds special encounter 'Strange Spirit' into over world encounter pool." | §7: the `strange_spirit` entries with `when = { calling = "hunter" }`, whose calling asks `from = ["bard"]`, `level = 30` |
| "Good characters can talk with the encounter spirit (not hostile)." | §7: a peaceful entry; `speaks_to = { min = "kind" }` on its dialogue; others hear `silent` |
| "They need to say following phrase to the NPC: 'Life before death' (ignores capitalization and punctuation)" | §6.2: `words::normalise` on both sides |
| "Spirit will echo the full oath: [six lines]" | §6.1: the word's `reply`, the six lines verbatim |
| "then the spirit disappears and applies the class to player" | `does = { calls = "hunter", departs = true }`; `World::call`, then `World::departs` |
| "Player receives enough experience to immediately receive level up to the first level of the new class." | §4.3: `add_xp(threshold(31) − xp)` when short, then the level-up; Hunter 1 is total 31 |
| "Fire drake dungeon 2nd to last level should contain a hint on some statue inscription or something." | §8: the `statue` terrain, vault notes stamped, `hunters_statue` pinned on Fyrdelf 11, the inscription |
| "Class should just have regular martial progression with no abilities above level 1 for now." | §9.1: `hp = 7`, 2 rolls, bonus picks from Weaponcraft and Guard, one table row |
| "Level 1 ability: ability that summons a magical melee weapon of chosen base type -> use the ability -> open dialog to pick base -> the weapon gets equipped." | §9.2: `use:spirit_weapon:ID`, the shape list, the ordinary swap into the main hand |
| "if the weapon becomes unequipped, it is removed instead of placed somewhere other than the equipment slot. can be resummoned." | §9.2: `fade_spirit` on the two primitives every path uses; the ability again |
| "ability weapon: decent +hit bonus, ignores PV of targets, better crit range." | §9.2: `to_hit = 3`, `ignores_protection = true` (`ALL_PROTECTION`), `threat = 2` lower, data on the talent |

The second round's parts the owner did not reject stand: the total level and `class_of` (§4.1), the cap (`max_level`), banked XP (§4.3, now with the top-up), what a level brings by class (§4.4), the curve to 100 (§5), the bard capped at 30 (the bard design §5).

## 15. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| Q1 | **The calling is a campaign's** (`campaigns/<id>/data/callings/`), the class shared (`data/prestige/`)? | **yes**: the world decides how it calls; the class is the game's |
| Q2 | **The Say row shows only where the dialogue has words** (and the player is inside `speaks_to`)? | **yes** (§6.3); a bare prompt everywhere is a tell |
| Q3 | **The field takes 40 characters**, Enter sends, a chat's time whether heard or not? | **yes**; the longest phrase a dialogue may hold is 40 |
| Q4 | **Hunter 1 at total 31** (Hunter 30 at 60), by the XP top-up? | **yes**: the owner's words; and the bard-hunter's last level is everyone's |
| Q5 | **The spirit on the road, vale, downs, and woods day tables at weight 2**; not the fen, the deep woods, the Ascdun, or by night? | **yes**: a spirit of the open road; the owner may widen it to every table with one row each |
| Q6 | **The spirit weapon weighs and is made as its base**; or weightless (one read in `Stack::weight`)? | **as its base**: fewer hooks; weightless is one line if the owner likes the flavour |
| Q7 | **+3 to hit, threat 2 lower, Protection ignored** as the three gifts? | **yes**; all data on the talent, a rank away from growing |
| Q8 | **Priced as a wield (a base turn), no other cost**, reshaping included? | **yes** (§9.2) |
| Q9 | **The shape list offers every shared melee base** (22) grouped by skill, uniques' bases included as plain bases? | **yes**; a unique is its own item, its base a plain shape |
| Q10 | **The spirit has 1 hit point, no corpse, no XP**, and a peaceful kill's alignment cost, rather than being unkillable? | **yes**: no flag for one creature; the entry comes again |
| Q11 | **The hint through a `statue` terrain and vault notes stamped**, rather than a hunters' terrain with the text in `terrain.toml`? | **the statue and notes**: the next inscription costs a vault cell, not a terrain |
| Q12 | **The inscription's words** (§8) | as proposed; the owner edits the stone |
| Q13 | **`from` stays `["bard"]`** for now? | **yes**: the owner asked for the bard's; the list is data |
| Q14 | **The curve to 100 with `hp_level` 50 and `mana_level` 60 unchanged**? | **yes**: "not going there yet" |
| Q15 | **The death screen says "a bard and hunter"**? | **yes** |
| Q16 | **Thrown spirit weapons refused** ("It will not leave your hand.") rather than fading in flight? | **refused** |

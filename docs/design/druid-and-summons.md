# Warlock — The druid, and allies: player-castable summons on a shared ally core

Design document for the owner's review (2026-11-02, #821). Written against `origin/main` at
f3846261 (encounters E5). Nothing here is implemented; §13 cuts it into issues. It builds on what
is there: enemy summoning (`docs/dev/summoning.md`, `world/summon.rs`), the one cast path and
the spell data (`docs/dev/spells.md`, `data/spells/*.toml`, the ladders), the creature AI and
factions (`docs/dev/creatures.md`, `world/ai.rs`, `data/factions.toml`), statuses as the house
rule for anything that lasts (`data/status.toml`), the energy scheduler (`world/schedule.rs`),
level travel (`world/travel.rs::travel`), displacement (`world/talk.rs`), the one abilities menu
(`ability.rs`), the class tables (`docs/dev/progression.md` "Class tables") and the coverage
policy (rules 1–9 of #776, binding here), and the class ability gaps report (draft PR #802).
Every number is data and hidden in the game; the wiki states it (the owner's rule: the game is
vague, the wiki is accurate).

**The owner's request** (2026-11-02, #821, verbatim): "druid: create a design issue. We will do
player castable summons as part of this issue." The druid's class design, including
player-castable summons (today summoning is enemy-only): how a player's summons work (control,
duration, upkeep, AI, saves, limits) as well as the druid's table. Druid gaps: 6, 25, 32, 40, 60.

**The sibling** (#825): the ranger is the pet class and reuses the ally machinery. §4 is therefore
written as a general **ally core** (allies: summons and companions); §4.9 says exactly which parts
are shared and which are the ranger's to design. Nothing in this document decides how a companion
is gained, grows, eats, or dies: that is #825's.

## Contents

1. Summary
2. Research, tied to the decisions
3. What exists, and what this builds on
4. The ally core (shared): what an ally is, its side, its mind, orders, following, saves, experience
5. Player summons: the spell kind, duration, the cap, departing, what is left behind
6. The druid's table
7. The data
8. Determinism, saves, replays
9. What the player sees: messages, the menu, look mode, dumps, the wiki
10. Talents and class abilities (the coverage policy)
11. Rejected ideas
12. Implementation plan (slices)
13. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **An ally** | an ordinary `Actor` on the level with `Actor::ally: Option<Ally { master, source, order }>` (§4.1). Its **side is its master's**: `faction_override = player`, so every rule that reads stances (enemy AI, spawn caps, the quick leave, Beast-friend) reads it as the player's without a new branch. `source` is `Summoned { spell }` or `Companion` (#825). §4 |
| **Its mind** | one arm in `ai.rs::decide` for an actor with `ally`: it attacks what its **master** is hostile to (`World::stance(master, other)`, so a Beast-friend's wolf leaves the rats alone as the druid does), else obeys its **order**: *Follow* (heel within `heel` cells of the master), *Stay* (hold the cell, fight what comes beside), *Hunt* (go to a cell the master pointed at and fight there, then follow). Deciding draws nothing. §4.3 |
| **Orders** | four abilities in the one abilities menu under a new group **Allies** (`Source::Ally`): Come to heel, Hold here, Set on (aimed at a cell), Send away. No key of their own (the owner's rule); `Action::Order(Order)`, replay text `order:follow|stay|hunt:X,Y|dismiss`. An order takes **no time** and makes a **voice's noise** (`[orders] noise = 60`): a shout costs stealth, not turns (DCSS's `t`). §4.4 |
| **Following** | an ally **beside** the player when they take stairs or leave an area comes along (NetHack and DCSS's adjacency rule), placed by `free_cell_near` at the arrival; the rest stay where they are (a summon left behind departs when its time runs out, a companion waits, #825). Onto the **overworld** (which has no creatures) adjacent allies join the **retinue** (`Ally::retinue`: not placed, frozen as other levels are) and step out beside the player at the next level. §4.5 |
| **Experience** | a kill by an ally pays its **master** as their own kill would (`earn_kill`, the first-kill bonus, the quest tally, Favoured prey): DCSS's rule, `[xp] ally_percent = 100` in data. §4.6 |
| **Summons** | a spell `kind = "summon"`: `[spell.effect] summon = { creatures, count, most, status, severity }`; cast as a `self` spell; the summoned appear in the caster's `summon_cells` (the enemy summoner's, shared), take the `player` side and the **`summoned` status whose degrees are their minutes** (`eases = 60`): when it ends they depart ("The wolf slips back into the dark."). Two ladders: `count` (a roll ladder, new) and `degrees` (the duration, Presence). No upkeep, no cooldown: mana paid once, time the penalty. A spell's `most` caps what it keeps alive: past it the **oldest departs** (DCSS). The slain leave no corpse and no loot. §5 |
| **The druid's table** | 1 Beast-friend, Spark; 3 Barkskin; **6 Call of the wild** (wolves); 12 Call lightning; 18 Nature's ward; **25 Call of the forest** (a bear); **32 Pack leader** (one more beast every call, calls last half again as long); **40 Call the elements** (a greater elemental, the element the wild chooses); 50 Flame and frost; **60 Lord of the wild** (calls no longer lapse: they stay until slain or sent away; two more allies at once). Three spells, two passives, a capstone that changes every fight. §6 |
| **Talents** | a new broad system **`allies`** with its one pickable, **Beast-bond** (Wayfarer): your allies strike +1 and take Protection +1. Pack leader and Lord of the wild carry the tag as class abilities. The spells are `spells`, `mana`, `allies`. §10 |
| **New content** | two shared beasts, `wolf` (level 4) and `bear` (level 12), faction `wildlife`, tag `beast` (never strays: wildlife is not hostile to the player; the ranger may add more); three spells; the status `summoned`; `data/allies.toml` (every constant). §7 |
| **Slices** | **A1** the ally core (rules, orders, following, XP, saves, dumps, the dev `ally:` action, the `allies` system and Beast-bond); **A2** summons (the spell kind, the status, the cap, departing); **A3** the UI (the Allies group, Set on's aim, look mode, the ally mark, text snapshots); **D1** the druid's content (the beasts, the three spells, Pack leader, Lord of the wild, the wiki, the spec). The ranger's companion is R1 of #825, on A1. §12 |

**Key choices, and why**

1. *An ally is an actor on the player's side, not a new kind of thing.* `faction_override = player` makes every existing stance read (enemy AI, the spawn cap's hostile count, the quick leave, coarse turns, `disturbs_rest`'s cousins) treat it as the player's without a branch each; one field, `Actor::ally`, says whose it is and what it was told. The ally's own target choice is the one new rule, and it reads the **master's** stance, so Beast-friend, the fey courts, and shopkeepers all behave for a pack as they do for the druid.
2. *Orders are abilities in the one menu, free of time, loud.* The owner's rule puts every ability in the abilities menu with no key of its own; a shout that cost a turn would make a pet class fight with half its actions; a shout that cost nothing at all would be the one silent thing in a game where everything is heard. So: no time, a voice's noise.
3. *Duration is a status's degrees.* The house rule (`ability.rs`): anything that lasts is a status with `eases`. A summon's minutes are degrees of `summoned` on the summoned creature, so the `degrees` ladder, Lingering (`SpellDuration`), and the talents' `degrees` hook all lengthen a call with no new code, and the status loop ends it where every status ends.
4. *The enemy summoner's code is the player's.* `summon_cells`, the pool draw, `make_creature`, `Actor::summoner`, the `summoner` roster tests: all of it stays, and the spell kind calls the same functions with the caster as the summoner. One summoning, two sides.
5. *Full experience for an ally's kill.* NetHack's "no XP for pet kills" makes a pet a liability to the player's growth; DCSS tried half and went to full because a summoner felt punished for playing their class. The summon costs mana and the beast can die; that is the price. The owner's no-anti-farming rule points the same way.
6. *The capstone is lasting calls, not a bigger beast.* A fourth summon spell at 60 would be a bigger number. A druid whose pack never lapses fights every fight with a standing pack and manages its losses: it changes how the character plays, which is rule 1's test.

## 2. Research, tied to the decisions

| Source | What it does | Taken | Left, and why |
|---|---|---|---|
| **DCSS** (Summonings, allies) | summons are temporary; each summoning spell keeps a cap of its own summons, the oldest expiring past it; allies' kills pay the player full experience (half in old versions, changed because summoners felt punished); allies on adjacent cells follow through stairs; the `t` shout orders (follow, wait, attack, retreat) are free; abjuration shortens duration; summons leave no corpse | the per-spell cap with the oldest departing (§5.3), full XP (§4.6), adjacency following (§4.5), free orders (§4.4), no corpse (§5.4) | a separate Summoning skill (our schools are the skills: the spells are enchantment); abjuration (no enemy casts spells that resolve yet) |
| **NetHack** (pets) | a pet follows only if adjacent (or leashed) when the player changes level; the player gains **no** experience for pet kills ("not responsible"); no direct commands (a whistle calls them, a leash binds); swapping places with a pet; pets gain levels by kills; a pet left behind long enough goes feral | adjacency following, swapping places (displacement already exists: an ally is not hostile, so the player swaps with it) | no XP (choice 5); the leash and the whistle as items (orders are abilities); feral pets (a summon lapses, a companion is #825's) |
| **ADOM** (druid, companions, Summon Monsters) | the druid is the weather-and-beast caster: animals are peaceful to him, lightning and the elements come with level; companions follow when adjacent and may be ordered by talking ("Follow me", "Wait here"); its *Summon Monsters* spell calls **hostile** monsters | the druid's shape (Beast-friend, Call lightning, Flame and frost stay; the calls fill the gaps); adjacency following; orders in words | hostile summons: a call that may kill its caller is a curse, not a class |
| **Diablo II** (druid, necromancer) | summons are permanent until slain, follow everywhere, scale with skill level; the druid may keep only **one kind** of animal at a time (wolves or a bear), one vine, one spirit; the necromancer's skeleton count rises with the skill | permanence as the **capstone** (Lord of the wild, §6), not the baseline; count growing with the caster (the `count` ladder and Pack leader) | the one-kind rule (a per-spell cap is simpler and lets a druid keep a wolf and a bear; §11); permanent from level 6 (a pack from the first dungeon with no cost but mana would be the whole game) |
| **D&D 3.5** (Summon Nature's Ally, the animal companion) | a summon lasts 1 round a level, calls 1 creature of its list or 1d3 of the list below; the companion is the ranger's and the druid's, permanent, improving with level | the ladder of count by stat (one at the base, more as the caster grows); the companion is the ranger's (#825) | rounds-per-level duration (ours is minutes by Presence, the degrees convention) |
| **Warlock's own enemy summoning** (#520) | `[creature.summon]`: a pool, `count`, `cooldown` as a status, `mana`, `range`, `spread`, `most`; `Actor::summoner`; a summoned kill pays XP as any | the whole mechanism, called by the spell kind (choice 4); `most` and `spread` as the spell's | the cooldown status (a mana cast has no cooldown, `docs/dev/progression.md` "Coverage policy" step 4: time is the penalty) |

## 3. What exists, and what this builds on

| Part | Today | Used for |
|---|---|---|
| `world/summon.rs` | `summons_at`, `live_summons`, `summon_cells` (free cells within `spread`, in line of sight, nearest first), `creature_summons` (pay, status, noise, `count.roll` on the gameplay stream, `below(pool.len())` a creature, `make_creature` with the summoner's faction, `Actor::summoner`) | the spell kind's resolution (§5.2) calls `summon_cells` and the making with the player as summoner |
| `Actor::summoner`, `Actor::faction_override` | who called it; the faction it was placed in when not its type's (`Actor::faction`) | an ally's `faction_override` is `player` (§4.2); `summoner` stays what it is |
| `ai.rs::decide` | look (`nearest_hostile_in_sight`: hostiles by `content.stance_of(me.faction, a.faction)`, Beast-friend, mimics) → `fight` → `search_or_return` → patrol; `Mind { last_seen, noise, post, noticed, missed }`; `route` (A*, fixed tie-breaks) | the ally arm goes first in `decide` (§4.3); `route` moves it; `fight` attacks for it |
| `World::stance(a, b)` | Beast-friend, then the factions' stance | an ally's target test is `stance(master, other) == Hostile` (§4.3) |
| `World::travel` | moves the player alone; `free_cell_near`, `arrive_spawner`, arrival lines | the adjacent allies move after the player (§4.5) |
| Displacement (`world/talk.rs`) | the player swaps places with a creature not hostile to them | the player walks through their pack; nothing new |
| The scheduler | actors on the current level act by speed; other levels frozen | allies act as creatures do; the retinue is frozen as another level is |
| Statuses | `eases` a degree, `max`, `start`/`end` lines; the status loop in `World::live` | `summoned`: degrees are minutes (§5.3); its end is the departure |
| The one cast path | `kind` buff, heal, cleanse, hex, utility, damage; `[spell.effect]` by kind; the ladders (`damage`, `heal`, `range`, `length`, `radius`, `degrees`); `spell_degrees`; Lingering | a new kind `summon` with its effect table and the `count` ladder (§5.1, §7.2) |
| `ability.rs` | the one registry: `Ability` variants with `Source`, `Aim`, `had_by`, `unavailable`; the menu's groups by source | `Source::Ally` and four abilities (§4.4) |
| `earn_kill`, `tally_kill`, `favoured` | pay the player's kills only (`experience.md`: "keep it that way by awarding only where the attacker is the player") | the one exception, in words: the attacker is the player **or their ally** (§4.6) |
| The dialog, `refuse_due` | — | not needed: no order is a question |
| `data/factions.toml` | `player`; `wildlife` (`animal`, neutral to the player) | the beasts' home faction; the ally's side |
| Dumps | `actors` (dev): `; summoned by #N` | `; ally of #0 (summoned, 4 min; follow)` (§9.3) |

## 4. The ally core (shared with the ranger)

### 4.1 What an ally is

```rust
pub struct Ally {
    pub master: ActorId,          // the player, today; the field makes no assumption
    pub source: AllySource,       // Summoned { spell: SpellId } | Companion (#825's data)
    pub order: Order,             // Follow (default) | Stay { at: Pos } | Hunt { at: Pos }
    pub retinue: bool,            // travelling with the master off any level (§4.5)
}
```

`Actor::ally: Option<Ally>`, `#[serde(default, skip_serializing_if = "Option::is_none")]`: every
other actor saves byte for byte as before, old saves load, and runs without allies hash as before
(`World::state_hash` covers it as it covers every field). An ally is `ActorKind::Creature`, with
its type's stats, speed, hit points, eyes, and attacks; it is in `WorldState::actors` on its
level like any creature, so the scheduler, sight, hearing, blood, statuses, and death need
nothing new. `World::allies_of(master)` lists them in actor id order (deterministic).

### 4.2 Its side

An ally is made with `faction_override = Some(player)`. That one assignment does the work:

| Rule that reads a stance | What it now does for an ally, with no change |
|---|---|
| enemy AI, `nearest_hostile_in_sight` (`stance_of(me.faction, a.faction)`) | hostiles attack the pack as they attack the player (the kobolds list `player` hostile) |
| the spawn cap (`spawn.rs::attempt`, hostiles by `World::stance`) | allies are not counted as monsters |
| the quick leave (`quick_leave.rs`: a hostile sees the player or stands within 10) | allies never block `<` |
| coarse turns (`stride.rs::away`, `idle`: hostiles about) | allies rest when the player rests |
| displacement (`talk.rs`: not hostile) | the player swaps places with an ally; `budge` is the type's (always, for a beast) |
| Beast-friend (`befriended(state, content, me, a)`) | reads the *player's* talents when `me` is the player; for an ally see §4.3 |

Two reads need one line each: `talk.rs::disturbs_rest` (a creature whose faction has enemies
disturbs a rest: an ally must not, or the druid never sleeps) and `sight.rs`'s "compute creature
sight only where hostiles stand" (allies are not hostiles, so nothing is computed for them;
their own looking is the AI's, §4.3). The ally's `Mind` works as any creature's (`noticed`,
`missed` set by perception and the slip, unused for an ally since the slip is the player's).

### 4.3 Its mind: the ally arm

One arm at the top of `ai.rs::decide`, for an actor with `ally`, in a new `world/ally.rs`
(`World::ally_plan(id) -> Plan`), before the creature's own look. In order:

| Step | Rule | Draws |
|---|---|---|
| 1 target | the nearest actor on the level within its sight that it **sees** (its own eyes and light, `sight::sees_by`, line of sight as `nearest_hostile_in_sight` computes it) with `World::stance(master, other) == Hostile`, not `hides` (a mimic), ties by lower id; with *Stay*, only one within `[stay] reach = 1` of its post (it holds the line); with *Hunt*, one within `[hunt] reach = 3` of the hunt cell, else the nearest anywhere in sight | none |
| 2 fight | `ai.rs::fight(id, target, at)` as a creature would: melee closes in; a type with `combat = ranged` and a launcher shoots; a `caster` type announces (creature spells are intents today), so summon pools name none | the attack's own rolls |
| 3 order | *Follow*: if farther than `[follow] heel = 2` from the master, `route` toward the master's cell (goal: within `heel`), else `Plan::Wait`; *Stay*: `route` back to its post if off it, else wait; *Hunt*: `route` to the hunt cell; arrived with nothing to fight within `reach`, the order falls back to *Follow* (said once: "Your wolf looks back at you.") | none |
| 4 leash | a *Follow* ally fighting farther than `[follow] leash = 8` from the master breaks off and heels (it was drawn away); *Hunt* and *Stay* ignore the leash | none |

The master's stance, not the ally's: a druid with Beast-friend has a pack that leaves rats and
deer alone, as they do; attacking a shopkeeper or a quest-giver is never an ally's idea (the
player is not hostile to them). An ally never provokes a court or a faction on its own; a blow of
the player's does, as today, and the pack follows the master's new enemies at once.

Doors: an ally opens a door in its way only if its type `opens_doors` (wolves and bears do not);
it waits before a closed door and comes through when the player opens it. Traps: an ally steps
onto a trap as any creature does (`step_onto_trap`; known traps are the player's knowledge, not
the wolf's); the player who leads a pack over a trapped corridor pays for it. Water and ice: as
any creature (`tread`, `wade`). Noise: its footsteps are its type's (`footsteps`); a pack is loud,
and a stealthy druid is a contradiction the data states.

Nothing above draws from any stream: the ally's plan is as deterministic as a creature's, and a
run with no allies calls none of it.

### 4.4 Orders

| Ability (the Allies group, `Source::Ally`) | `Order` | Aim | Replay text | Does |
|---|---|---|---|---|
| **Come to heel** | `Follow` | nothing | `order:follow` | every ally of the player on the level (not the retinue) follows |
| **Hold here** | `Stay { at: its cell }` | nothing | `order:stay` | every ally holds where it stands |
| **Set on** | `Hunt { at }` | a cell (the aiming mode, as a shot: `Aim::Cell`) | `order:hunt:X,Y` | every ally goes there and fights what is within `reach` of it |
| **Send away** | — | nothing | `order:dismiss` | every summoned ally departs now (§5.4: "You send the wolves away."); a companion is refused ("Your hound is no beast to send off.": #825 may change this) |

`Action::Order(Order)`: **no time** (`Spent::NoTime`), a noise of `[orders] noise = 60` at the
player's cell (`noise::Act::Voice`, the throw-voice loudness class: creatures that hear it look
into it, as for any voice), and the message. Refused, with no time and no noise, with no ally on
the level ("You have no one to call."); the menu greys the four out then
(`Unavailable::NoAllies`). The rules of `ability.rs` hold: no `KeyCommand`, no `Context`
entry, no key; the menu's repeat key repeats the last order as it repeats any ability.

Why one order for all: a per-ally order needs a way to pick an ally (a list, a cursor), a screen
more and a question the owner's one-dialog rule does not want asked. A pack obeys as a pack; a
ranger's one hound is the whole pack. `Set on` at a cell with an enemy on it is "attack that";
at an empty cell it is "go there": one aim covers both.

### 4.5 Following across levels, and the retinue

In `World::travel`, after the player's cell is set and before `arrive_spawner`: every ally of the
player on the level left whose cell was **beside** the player's old cell (Chebyshev 1, `[follow]
comes_along = 1`) and that is not held fast (Webbed, a grip) moves with them: `level = to`, `pos =
free_cell_near(arrival)` in actor id order, energy kept. The rest stay. Said: "Your wolves come
with you." / "Your bear is left behind." (only for one the player saw). A summon left behind
keeps its minutes and departs when they run out, wherever it is (§5.3). A companion left behind
waits where it was (its post) and comes to heel when the player returns beside it: #825 says
whether it does more.

**The overworld has no creatures** (campaign design §9.1, encounters design §6.2), and a journey
with a wolf glyph on the map beside the player would break that. So when the level entered is
`LevelKind::Overworld`, the allies that come along join the **retinue**: `ally.retinue = true`,
`level` the overworld's, `pos` the player's (never drawn, never acted: `run_turn` skips retinue
actors as it skips other levels' actors; perception lists them from `PerceptionView::allies()`,
not from the cell). Statuses on a retinue actor do not tick (as on another level's actor: `live`
runs the current level's acting actors), so a timed summon spends none of its minutes on the
road; the owner may prefer that a summon lapses on reaching the overworld (§13 Q4). Entering any
other level, the retinue steps out: each placed by `free_cell_near` around the player's arrival
in id order, `retinue = false`, said "Your wolves follow you in."

Resting on the overworld with a retinue is a rest as today (the retinue takes no turns); an
**encounter** (`world/encounter.rs`) that opens a local area places the retinue beside the player
as any entry does, before the party's positions are taken (the party's cells avoid them). An
ambush's cover cell holds one actor: the retinue stands in the cells about it.

### 4.6 Experience, quests, and the first kill

`fight.rs::dies` pays `earn_kill` when the killer is the player **or an ally of the player**
(`Actor::ally.master == player`), at `[xp] ally_percent = 100` of what the player's own blow
would have paid (first-kill bonus and the level scaling included; `kill_xp` is the type's). The
quest tally (`tally_kill`), Favoured prey's record (`Quests::prey`), the named monsters slain,
and the rage/openings builds (`gain_resource(Kill)`) go to the master too; the ally gets nothing
(creatures hold no resources). `docs/dev/experience.md`'s "award only where the attacker is the
player" gains its one exception in words. A death caused by an ally's status (`Effect::by` an
ally) pays the same way. An ally's own death pays no one.

### 4.7 Healing, buffing, and friendly fire

The druid's touch heals (Mend, Cure wounds: "the creature at the adjacent cell") already heal an
ally: nothing new. A buff is `self` only (Barkskin on the caster): no change. Area spells hit
everyone in their cells but the caster (`spares_self` spares the caster alone): **Call lightning's
line and Flame's burst hurt the pack** if it stands in them. Kept: it is the tactical cost DCSS
and D&D charge, and a druid who calls wolves and then calls lightning learns to aim (§13 Q3). An
ally hurt by its master's spell stays an ally (no provoking of one's own side).

### 4.8 Death, departure, and what is left

An ally slain is a creature slain: blood, a corpse, and its gear by the ordinary rules, **unless it
is a summon** (§5.4: nothing left). "Your wolf dies." is said when seen. A companion's death is
#825's (the fiction of losing it, whether a new one may be gained).

### 4.9 What is shared, and what is the ranger's

| Shared (A1, this design) | The ranger's (#825, on A1) |
|---|---|
| `Actor::ally`, `Ally { master, source, order, retinue }`, `AllySource::Companion` as a variant with no data of its own yet | what `Companion` carries (a name, a level, hunger, a bond) and how it is saved |
| the side (`faction_override = player`), `World::allies_of`, `disturbs_rest`'s exception | — |
| the ally arm (§4.3), `data/allies.toml`'s `[follow]`, `[stay]`, `[hunt]`, the leash | a companion's own behaviours, if any (fetching, tracking, growling at the unseen) |
| the four orders, `Source::Ally`, the Allies group, `Action::Order`, the aiming for Set on | what Send away does to a companion |
| following across levels, the retinue, the encounter placement | whether a companion left behind does more than wait |
| experience, the quest tally, Favoured prey for an ally's kill | — |
| the `allies` system, Beast-bond, the `ally_mod` hooks (`to_hit`, `protection`, `count`, `duration_percent`, `lasting`, `most`) | the ranger's class abilities on these hooks or its own |
| messages, look mode's "(your wolf)", the ally mark, dumps, the wiki page `allies.md` | the companion's own rows and words |
| summons (§5): the ranger needs none of it | how a companion is gained (a taming, a level's gift) and grows |

## 5. Player summons

### 5.1 The spell kind

`kind = "summon"`, cast on `self` with `resolution = "auto"` (a call needs no target: the
summoned appear about the caster), priced as any self spell of its level (`SpellRules::mana`:
`max(1, level)` × the self shape's percent × auto's). The effect table:

```toml
[spell.effect]
summon = { creatures = ["wolf"], count = "1d2", most = 4, status = "summoned", severity = 5 }
```

| Field | Meaning | Load checks |
|---|---|---|
| `creatures` | the pool: each summoned is one of these, `below(pool.len())` on the gameplay stream (the enemy summoner's draw) | known types the campaign may use; not the player, not `unique`, not `animated`, not a `caster` (its spells are intents only); at least one |
| `count` | how many come, a roll | a `Roll` of at least 1 at its least |
| `most` | how many of this spell's summons the caster keeps alive at once; the oldest departs when a new one would exceed it | 1 to `MAX_COUNT` (12) |
| `status`, `severity` | the status each summoned wears, whose degrees are its minutes; the base degrees | a status with `eases` and no tick (`summon::STATUS`'s checks today); severity 1 to its `max` |
| `spread` | how far from the caster they appear (default 2, as the enemy's) | 1 to 5 |

The two ladders (`[spell.grows]`): `count` (**new**, a roll ladder, `Unit::Roll`, allowed only
for a summon spell: `"willpower 14 1d2+1, 18 2d2"`) and `degrees` (the duration, the buffs' and
hexes' convention: `"presence 14 8, 18 12, 22 16"`). The hooks act as on any spell: `degrees`
(Lingering, a talent's `degrees`) lengthen the call; `dice`/`bonus`/`sides` act on the **count**
roll (there is no damage roll: Brimming's die more is one wolf more; Empower's rung climbs both
ladders). `cost` and the mana percents as today. `SpellData::aimed` is false (the menu uses it
at once); `trained_stats` are its ladders' (Willpower, Presence).

### 5.2 The cast

`World::cast_summon` in `cast.rs`, beside `cast_area_hex`:

1. Refusals, no time and no mana: casting blocked (Silenced: already the cast path's); no free
   cell in `summon_cells(caster, spread)` ("There is no room for them beside you."); the
   player's allies on the level already at the global cap `[allies] most = 8` ("You cannot
   lead more than this.").
2. Pay (`pay_cast`), the noise (`[spell.cast] noise`, chanting as any spell), the message
   (`[spell.cast] message`: "You call, and the wild answers.").
3. Roll the count at the caster's rung (`Cast::roll` as a heal's), cap it by `most − live` of this
   spell's summons alive and by the cells free and by the global cap; **if the cap is 0, the
   oldest of this spell's departs first** (§5.3) so a call always brings at least one.
4. Each: one `below(pool.len())` draw, `make_creature` on the next free cell with
   `faction = player`, `rolls_gear = false` (a beast carries nothing; a pool type with gear
   lines gets none), `summoner = caster`, `ally = Ally { master, source: Summoned { spell },
   order: Follow, retinue: false }`, the status at the cast's degrees (`spell_degrees`) with the
   caster as its giver. `World::spawned` bumped (a rest perceives after this turn).
5. Said: "Two wolves come to your call." / "A bear lumbers out of the shadows." (the enemy's
   `counted`/`and_list` words, the first person); unseen appearances as the enemy's "A wolf
   appears!" are impossible here (they appear beside the caster) but the code path is shared.

The marks: a cast is practised as any (`practise_cast`: enchantment; the caster style is
Toucher/Sniper by where it is cast from: a `self` spell takes none, as Barkskin takes none).

### 5.3 Duration and the cap

The status `summoned` (`data/status.toml`, placed by the other durations' statuses): `max = 60`,
`eases = 60` (a degree a minute), `stacking = "refresh"`, no tick, no `cost`, `seen = "called"`
(look mode: "a wolf, called"), `end` said by the departure (below), no `start` (the call's own
message says it). Base severity 5: five minutes, the length of a fight and the walk to the next
room; the ladder to 16 at Presence 22; Pack leader +50 % (`duration_percent`, on the degrees,
rounded down, at least 1); Lingering as on any status.

When the status ends on an ally (`World::live`, the one place statuses end), **the ally departs**
(`World::ally_departs(id, Why::Lapsed)`): removed from `actors` with nothing left behind, said
"The wolf slips back into the dark." (its type's `depart` line, default "… fades away."; a
`[creature] depart` field, optional). On another level or in the retinue the status does not
tick, so a summon left behind on the first floor is still there, with its minutes, when the
player returns; the owner may want it gone (§13 Q5).

`most` per spell (§5.1) and `[allies] most = 8` over all: at the per-spell cap the oldest of that
spell's summons departs ("The old wolf gives way to the new."); at the global cap the cast is
refused before it costs anything. **Lord of the wild** (§6): `lasting = true` makes the status a
permanent one (`severity` ignored: the status is not given; the summon departs only when slain or
sent away) and raises the global cap by its `most` (+2).

No upkeep: no mana a turn, no hunger (`satiation: None`, as creatures have none), no cooldown
status (the policy's step 4: a mana cast has no cooldown besides). Time is the penalty: the
minutes pass, the pack thins, the druid pays mana again.

### 5.4 Departing, dying, and what is left

| Event | What happens |
|---|---|
| lapsed, sent away, gives way | `ally_departs`: removed, its cell freed, no pile, no corpse, its message; `Event::Departed` for presentation (a fade in the glyph layer under the effects switch, nothing under reduced effects) |
| slain | `creature_dies` as any, **but** a `Summoned` ally leaves **no corpse and no pile** (the fewer rules the better: a summoned wolf is not food, and gear it had none); blood as any; "Your wolf dies." |
| its master dies | the run ends; nothing to do |
| the caster dismisses while the pack fights | the pack vanishes at once; the enemies turn to the druid (their `last_seen` is the player's cell if they saw them) |

## 6. The druid's table

Druid data today (`assets/data/classes/04-druid.toml`): Toughness +1d2, Willpower +2d2, Presence
−1; hp 4; mana 9; casting 100 (caster level = character level); affinity restoration 2,
transmutation 2, evocation 1; XP 115; Handle animal and Survival known for; staves 2, slings 1,
evocation 1; Staff fighter 1, Sniper 1; bonus talents from Wayfarer and Vigour at 10, 20, 30, 45,
55. All of it stays; `affinity` gains `enchantment = 1` (the calls' school: the druid studies a
scroll of a call as readily as of a flame).

| Lv | Entry | Kind | Status | What it does | Hook |
|---:|---|---|---|---|---|
| 1 | Beast-friend (`beast_friend`, the Wayfarer pickable) | passive | **stays** | animals are neutral until you attack one of their kind | `World::stance` |
| 1 | Spark (`spark`) | spell, level 1 | stays | a bolt, 1d4 electricity, 1 mana | the cast path |
| 3 | Barkskin (`barkskin`) | spell, level 3 | stays | Dodge +2 a minute a degree | — |
| **6** | **Call of the wild** (`call_of_the_wild`) | spell, level 5 | **new** | 1d2 wolves (level 4) for 5 minutes about you; Willpower 14: 1d2+1, 18: 2d2; Presence 14: 8 min, 18: 12, 22: 16; `most` 4; 5 mana | §5 |
| 12 | Call lightning (`call_lightning`) | spell, level 11 | stays | a line under the open sky, 2d6 to 5d6+2 | — |
| 18 | Nature's ward (`natures_ward`) | passive | stays | wading never sickens; poison eases twice as fast | — |
| **25** | **Call of the forest** (`call_of_the_forest`) | spell, level 20 | **new** | a bear (level 12) for 5 minutes; Willpower 18: 1d2 bears (the base count `1`, the ladder `"willpower 18 1d2"`); Presence as above; `most` 2; 20 mana | §5 |
| **32** | **Pack leader** (`pack_leader`) | passive | **new** | every call brings one more (`count + 1`, after the roll and before the caps) and your calls last half again as long (`duration_percent = 50`) | `ally_mod { count = 1, duration_percent = 50 }` read in `cast_summon` |
| **40** | **Call the elements** (`call_the_elements`) | spell, level 35 | **new** | a greater elemental (level 24) for 5 minutes, the element the wild chooses: the pool `greater_earth_elemental`, `greater_fire_elemental`, `greater_water_elemental`, `greater_air_elemental`, one drawn (the enemy summoner's draw; "The earth heaves and answers."); `most` 2; 35 mana | §5 |
| 50 | Flame and frost (`flame_and_frost`) | spells, levels 12 and 6 | stays | the fireball's burst; the frost burst | — |
| **60** | **Lord of the wild** (`lord_of_the_wild`) | passive | **new** | your calls no longer lapse: what you call stays until it is slain or you send it away; and you may lead two more (`most + 2`) | `ally_mod { lasting = true, most = 2 }` read in `cast_summon` and the global cap |

The caster-level check (`docs/dev/spells.md` "Guaranteed spells": a given spell's level at most
the class's caster level at the level it comes): 5 ≤ 6, 20 ≤ 25, 35 ≤ 40. The pools: the druid's
mana is about 47 at level 6 (nine calls of 5), about 180 at 25 (nine of 20), about 290 at 40
(eight of 35) (`docs/dev/spells.md` "Balance" 1: a full caster's pool is about ten casts of a
spell of its level). The beasts' levels against the druid's: wolves of 4 at 6 (two to three
at Willpower 14), a bear of 12 at 25, an elemental of 24 at 40: about half the character's level,
the summoner's usual bargain (DCSS's summons at their power, D&D's list a tier down), with
numbers in them (two wolves of 16 hit points each beside a druid of about 40). Nothing here
checks it: the strength is the author's call (the spell scaling design's decision 16); the
balance probe gains a row (§12).

The cadence (1, 6, 12, 18, 25, 32, 40, 50, 60) is filled: nine levels, the table's off-cadence 3
kept. Three of the nine are calls, as the gaps report expects of "the druid's early entries" that
may be memes (the progression design §16 decision 5): the druid is the class that fights with
a pack beside its lightning.

What goes: nothing. What changes: nothing but the affinity line. The tables of other classes:
none (the ranger's is #825's).

## 7. The data

### 7.1 `assets/data/allies.toml` (shared; every constant of §4 and §5)

```toml
[follow]
heel = 2            # a following ally stays within this many cells of its master
leash = 8           # farther than this, a following ally breaks off a fight and heels
comes_along = 1     # allies within this many cells of the player come along to another level

[stay]
reach = 1           # a holding ally fights only what comes within this of its post

[hunt]
reach = 3           # a set-on ally fights what is within this of the cell it was sent to

[orders]
noise = 60          # an order's loudness (a voice)

[allies]
most = 8            # the player's allies on one level at most, every source

[xp]
ally_percent = 100  # what an ally's kill pays its master, of the master's own kill
```

`content/ally.rs` loads it into `AllyRules`, checked at load (each 1 to 20, `noise` 1 to
`MAX_LOUDNESS`, `most` 1 to 24, the percent 0 to 100). Without the file no ally can be made
(fixtures without it hash as before).

### 7.2 The spells (`assets/data/spells/`, one file each, path order: new files at the end of the
list, as every new spell)

| Spell | level | school | pool | count | degrees | most | message |
|---|---:|---|---|---|---|---:|---|
| `call_of_the_wild` | 5 | enchantment | `wolf` | `1d2`; `willpower 14 1d2+1, 18 2d2` | 5; `presence 14 8, 18 12, 22 16` | 4 | "You call, and the wild answers." |
| `call_of_the_forest` | 20 | enchantment | `bear` | `1`; `willpower 18 1d2` | 5; the same | 2 | "You call, and the forest answers." |
| `call_the_elements` | 35 | enchantment | the four greater elementals | `1`; `willpower 22 1d2` | 5; the same | 2 | "You call, and the elements answer." |

Each has a scroll (`scroll_of_ID`, the convention: every spell has one; a wizard may study a call
at the enchantment affinity the wizard has), at the object README's level-band values (level 5:
the 3–4 band; 20 and 35: new bands the README gains, say 8/10/320 and 10/5/500; §13 Q7). Their
`[spell.cast]`: `noise = { sound = "chanting", loudness = 20 }`, no `effect` (nothing flies).

### 7.3 The beasts (`assets/data/creatures/`, shared; `docs/dev/creatures/beasts.md` their file)

| Type | level | xp | speed | hp | attack | faction | tags | notes |
|---|---:|---:|---:|---:|---|---|---|---|
| `wolf` | 4 | 30 | 1250 | 16 | bite 1d6 piercing | `wildlife` | `beast`, `animal` | darkvision, hearing 20, footsteps 60; `depart = "slips back into the dark"` |
| `bear` | 12 | 70 | 1000 | 60 | claws 2d6 slashing | `wildlife` | `beast`, `animal` | a hide (the body rule gives Protection `(12 − 10) ÷ 8` = 0, so its hide carries a Defence instead, as `Hide::check` asks); `depart = "lumbers back into the shadows"` |

Both on the `xp = 5 × level + 10` guideline. Neither strays (wildlife is not hostile to the
player: `may_stray` leaves them out) and neither is in a population; the Ascmearc's `woods` and
`deep_woods` encounter tables may gain a wolf-pack row as wildlife in D1 ("Hunt them" / "Let
them be"), which is content, not a rule. The greater elementals exist (level 24, `elemental`,
speed 750): a greater elemental ally is slow and hits hard, as it should.

### 7.4 The talents

| Talent | Tree | Needs | Effects | Systems |
|---|---|---|---|---|
| **Beast-bond** (`beast_bond`, pickable) | Wayfarer | Handle animal 40 | `ally_mod { to_hit = 1, protection = 1 }`: your allies strike +1 and take Protection +1 (read in `Actor::stanced` and `Actor::protection` of an ally through its master's `TalentMods`: `World::ally_mods(id)`) | `allies` |
| **Pack leader** (`pack_leader`, class) | class, granted | — | `ally_mod { count = 1, duration_percent = 50 }` | `allies`, `spells` |
| **Lord of the wild** (`lord_of_the_wild`, class) | class, granted | — | `ally_mod { lasting = true, most = 2 }` | `allies`, `spells` |
| the three calls (class) | class, granted | — | `{ kind = "spell", spell = "call_of_the_wild" }` etc. | `spells`, `mana`, `allies` |

`TalentEffect::AllyMod(AllyModSpec { to_hit, protection, count, duration_percent, lasting, most
})` → `TalentMods::ally` (summed; `lasting` any), one read each: `to_hit` and `protection` in the
ally's profile and Protection (through the master), `count` and `duration_percent` and `lasting`
in `cast_summon`, `most` in the global cap. The alike check (`talent_alike.rs`): Beast-bond,
Pack leader, and Lord of the wild share the effect **kind** but none needs another, and their
effects differ at every rank: no clone, no chain.

## 8. Determinism, saves, replays

| | |
|---|---|
| Streams | the count roll and the pool draws on the **gameplay** stream, only when a call is cast (as the enemy summoner's); the ally's plan draws nothing; the marks on the training stream as any cast's |
| Saved | `Actor::ally` (left out while `None`); the summoned creature is an `Actor` as any; its `summoned` status in `effects`; `faction_override`, `summoner` as today. No `WorldState` field: the retinue is the allies whose `level` is the overworld's with `retinue` set |
| Hash | covers `ally` as every field; runs without allies hash as before; the showcase's and every golden's replays are untouched (no scenario casts a call) |
| Replays | `order:follow|stay|hunt:X,Y|dismiss`; `use:call_of_the_wild` as any spell; the dev `ally:CREATURE[:N]` (N allies of the type beside the player, followers, no time: captures and tests) |
| Format | no `FORMAT_VERSION` bump in the feature PRs (the integrator's, per release); the new spells, status, and creatures reorder nothing (new files at the end of their path orders) but enter the fingerprint as every content id does |
| Invariants (A1's and A2's tests) | an ally attacks only what its master is hostile to (a Beast-friend's wolf and a rat; a shopkeeper); an ally never counts toward the spawn cap nor blocks `<` or a rest; an order takes no time and is heard at `noise`; adjacency following and the retinue round-trip through a save; a call brings `count` capped by `most`, the cells, and the global cap, the oldest departing at the cap; the status's minutes equal the degrees and the departure leaves nothing; a slain summon leaves no corpse; an ally's kill pays the master the same XP as their own blow and tallies the quest; `levelup:auto` on a druid to 60 holds every entry; no draws without a call (`no_draws_without_allies`); load refusals: a unique, a caster, an animated thing, the player in a pool; `most` 0; a `count` ladder on a non-summon; `severity` past the status's `max` |

## 9. What the player sees

### 9.1 Messages (words, never a number)

| Moment | Line |
|---|---|
| the call | the spell's `message`, then "Two wolves come to your call." / "A bear lumbers out of the shadows." |
| no room | "There is no room for them beside you." (no time, no mana) |
| at the cap | "The old wolf gives way to the new." / "You cannot lead more than this." |
| the ally fights | "Your wolf bites the kobold." / "The kobold claws your wolf." (`fight::attack_message` with "your" for an ally the player sees; an unseen end is "something" as today) |
| lapsed | "The wolf slips back into the dark." (the type's `depart`) |
| sent away | "You send the wolves away." then each departure |
| slain | "Your wolf dies." |
| orders | "Your pack comes to heel." / "Your pack holds." / "You set your pack on the kobold." (or "… on that spot.") / none for dismiss beyond the departures |
| following | "Your wolves come with you." / "Your bear is left behind." / "Your wolves follow you in." |
| look mode | "your wolf (ally, called)" through `PerceptionView::stance` gaining an `Ally` word; its health in words as any seen actor's |
| the status bar | nothing new |
| the glyph | the **ally mark**, drawn as the unaware mark is (`ui/glyphs.rs`, the same mechanism: a mark on the glyph, from `SeenActor::ally`, perception only); under the ordinary switches |

### 9.2 The menu

The abilities menu gains the group **Allies** (`Source::Ally`, after Place): Come to heel, Hold
here, Set on (aimed), Send away; greyed "no allies here" without one on the level. The three
calls are in the **Spells** group with the roll shown as the owner's exception allows for spells
(`1d2 wolves, 5 mana`: `SpellRoll` gains the summon's words, `spell_words.rs`). Text snapshots:
`abilities-menu-allies`, `abilities-menu-allies-none`, `abilities-menu-druid-calls`. No golden.

### 9.3 The CLI and dumps

| Where | What |
|---|---|
| the header line | `allies 2` while the player has any on the level or in the retinue |
| `actors` (dev) | `; ally of #0 (summoned by call_of_the_wild, 4 min; follow)` / `(companion; stay at (12, 5))` / `(retinue)`; `summoned by #N` as today |
| `view` | an ally's glyph as any creature's (the mark is the renderer's) |
| `abilities` | the Allies rows with their reasons (`Come to heel (ally): not usable yet` without allies) |
| `spells` | `Call of the wild (level 5, 5 mana): 1d2 wolves, 5 min` |
| JSON | `allies: [{ id, creature, name, hp (words), order, source, lasts (minutes, or null for lasting), retinue }]`; each seen actor's `ally: true`; `spells[].summon: { count, minutes }` |

### 9.4 The wiki (`wiki/src/allies.md`, under Characters after Classes in `SUMMARY.md`;
`xtask/src/wiki/allies.rs`)

| Table | Rows |
|---|---|
| `ally-rules.md` | the constants of `allies.toml` in words and numbers |
| `summon-spells.md` | spell, level, school, pool (types with their levels), count and its ladder, minutes and its ladder, `most`, mana |
| `beasts.md` | the beasts' numbers as `creature-numbers.md` gives every type's (they join those tables by themselves) |
| `class-spells.md` | the druid's three calls "(level 5, from 6)" as every class spell |

Prose: what an ally is, the side, the orders, following and the retinue, experience, that area
spells hurt them, that summons leave nothing. The spec gains "Allies" in spec/08b-creatures.md
after "Summoning", and "Summon spells" in spec/08e-blood-and-status.md's "Spells";
`docs/dev/allies.md` is the topic file; `summoning.md` loses its "enemy-only" sentence and
points at it; CLAUDE.md's action text and dump layers get their lines.

## 10. Talents and class abilities

**A new system, `allies`** (`[[system]] id = "allies"`, "Allies: summons and companions, their
orders and their fights", placed by topic after `danger` (never at the end) in
`data/talents.toml`), and the policy's test: it is broad (a new kind of actor on the player's
side, an AI arm, four actions, a cap, a following rule: rules a talent can change), it ships
with **one pickable that earns its place**, Beast-bond (a flat bonus on every ally's blows and
hide: a real pick for a druid or a ranger, nothing for anyone else, as Deep well is nothing for a
warrior), and two class abilities carry the tag (Pack leader, Lord of the wild). The calls
themselves are `spells`, `mana`, `allies` (their effect reads the ally rules: the cap, the
status).

The rules of #776, checked:

| Rule | Check |
|---|---|
| 1 capstones change a fight | Lord of the wild: a standing pack in every fight, underground too |
| 2 no fixed-pool scaling | no XP or quest effect anywhere here (an ally's kill XP is the kill's, not a percent on a pool) |
| 3 no clones | Beast-bond's `to_hit`/`protection` on allies is no other talent's effect (Weapon focus is the player's own); Pack leader's count and duration no other's |
| 4 no chains | no talent here needs another |
| 5 flaws bite in a dungeon | no flaw |
| 6 outdoors only is niche | every call and every order works on a dungeon floor |
| 7 hidden marks are large or visible | no mark effect |
| 8 information is a garnish | nothing here only tells; the ally mark and "(ally)" are the rules' own perception |
| 9 shop favour | none |
| tags match hooks | `allies` on what reads `allies.toml` or `Ally`; `spells` and `mana` on what is cast and paid |

The ranger's class abilities on these hooks are #825's; none is pre-empted here.

## 11. Rejected ideas

| Idea | Why not |
|---|---|
| Per-ally orders (pick a wolf, tell it) | a list and a cursor more, and the one-dialog rule; a pack obeys as a pack, a hound is a pack of one |
| A rebindable "order" key, or keys per order | the owner's rule: abilities have no keys; the menu's repeat key repeats the last |
| A "companions" screen | nothing to show that look mode and the dumps do not; an info-only screen |
| Orders that cost a turn | a pet class fighting at half its actions; DCSS's shouts are free. The noise is the price |
| Mana upkeep per turn, or hunger for summons | the policy: a mana cast has no cooldown besides, time is the penalty; upkeep is a timer by another name |
| The cooldown status of the enemy summoner | the same |
| Permanent summons from level 6 | a free pack from the first dungeon; permanence is the capstone's to give |
| Diablo's one-kind-at-a-time | a per-spell cap is one rule and lets a druid keep wolves beside a bear |
| Summons that pay no XP (NetHack) or half (old DCSS) | choice 5; the owner's no-anti-farming rule |
| Summons that level up by kills (NetHack's pets) | that is the companion's story (#825); a summon is a call answered for a while |
| Hostile summons (ADOM) | a class ability that may kill its user is a curse |
| An elemental spell per element (four talents at 40) | four spells for one level; the wild choosing the element is the fiction and one draw |
| A bigger summon at 60 (elder elementals) | a bigger number, not a changed fight (rule 1); Lord of the wild instead |
| Wild shape (shapeshifting) | a system of its own (the player's body, attacks, speed, eyes, carry, speech all changing); not in this design; a line on the deferred list #196 if the owner wants it |
| Weather sense, Sun and moon, Timeless body, Winter's friend back | each was cut for a rule (information only; outdoors only; hunger and disease halves that are two other systems' talents); none returns |
| Allies as actors on the overworld | the overworld has no creatures (campaign design §9.1); the retinue carries them |
| A new `Stance::Ally` in the factions | the player's own faction is the ally's: the stance tables need no row |
| An abjuration spell against enemy summons | enemy summons are creatures; killing them is the counter today; a later spell list may add one |
| A `KeyCommand` for "swap with ally" | displacement does it |

## 12. Implementation plan

| Slice | Issue | What it touches | Depends on |
|---|---|---|---|
| **A1 The ally core** | allies: the actor field, the side, the mind, the orders, following and the retinue, experience | `warlock-core`: `world/actor.rs` (`Ally`, `AllySource`, `Order`, `Actor::ally`), `content/ally.rs` (`data/allies.toml`), `world/ally.rs` (`ally_plan`, `allies_of`, `order`, `ally_departs`, `follow_along` from `travel`, the retinue, `ally_mods`), one arm in `ai.rs::decide`, one line in `talk.rs::disturbs_rest`, `Action::Order` and the replay text, the dev `ally:`, `Source::Ally` and the four `Ability` variants, `fight.rs::dies` → `earn_kill` for the master (and `tally_kill`, `favoured`, `gain_resource`), `PerceptionView::allies`/`stance`/`SeenActor::ally`, the dumps (§9.3), `TalentEffect::AllyMod` with `to_hit`/`protection`/`most` read, `[[system]] allies`, Beast-bond; `world/ally_tests.rs` (§8); `docs/dev/allies.md`, CLAUDE.md's lines | — |
| **A2 Summons** | the spell kind | `content/spell_def.rs` (`kind = "summon"`, the effect table, the `count` ladder in `spell_ladder.rs`, the checks), `cast.rs::cast_summon` with `summon.rs`'s cells and making shared (`summon_for`), `data/status.toml` `summoned`, the departure on the status's end in `live`, no corpse for a slain summon in `creature_dies`, `count`/`duration_percent`/`lasting` read, `spell_words.rs`'s summon words, `summoning.md`'s sentence; `world/summon_spell_tests.rs` on a fixture pool | A1 |
| **A3 UI** | the Allies group, Set on's aim, the words, the mark | `crates/warlock`: `ui/abilities_screen.rs` (the group), `ui/target.rs` (`Aim::Cell` for Set on: nothing new if the aim reuses the shot's mode), `ui/glyphs.rs` (the ally mark as the unaware mark), `fight::attack_message`'s "your"; `screen_snapshots.rs` cases (§9.2) | A1 (A2 for the calls' rows) |
| **D1 The druid** | the content | `data/creatures/wolf.toml`, `bear.toml` (+ `docs/dev/creatures/beasts.md`, the roster tests' rows), the three spells and their scrolls, `pack_leader.toml`, `lord_of_the_wild.toml`, the three spell talents, `04-druid.toml`'s table and affinity, `data/talents.toml`'s system (if A1 shipped it pending-free, nothing), the Ascmearc's wildlife row (optional), `caster_tests.rs` (the table to 60, `levelup:auto`), `balance_probe` a row (the druid's calls: pool over cost, the summons' hit points against the floor's), the wiki (§9.4), the spec, `spells.md` "The first list" gains them | A1, A2, A3 |
| **R1** (#825) | the ranger's companion | on A1: `AllySource::Companion`'s data and rules | A1 |

A1 and A2 add saved state and actions: with the ranger's R1 that is the integrator's "about three
at once"; A1 goes first, R1 and A2 may run beside each other after it. Each PR runs the full
check; A1's and A2's snapshots are dumps' insta files and core tests, A3's text snapshots; no
golden anywhere (the ally mark is drawn as the unaware mark, whose guard already exists; a
departure's fade is an effect under the switches and is looked at in a `--capture` noted in the
PR, not kept).

## 13. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| Q1 | **Experience for an ally's kill** (§4.6): full, half, or none? | full (`ally_percent = 100`): DCSS's lesson, the owner's no-anti-farming rule, and the summon costs mana and can die |
| Q2 | **Orders free of time** but loud (§4.4), or a turn each? | free, a voice's noise (60): a pet class needs its actions; the noise is real |
| Q3 | **Friendly fire**: do Call lightning and Flame hurt the pack? | yes: aim is the druid's craft, and sparing allies would need every area spell to know sides |
| Q4 | **The retinue** (§4.5): allies come onto the overworld with the player, frozen, and step out at the next level; or summons lapse at the overworld's edge and only companions travel? | the retinue for all: one rule, and a Lord of the wild's standing pack should cross the map; a timed summon spends no minutes on the road (frozen as another level is) |
| Q5 | **A summon left behind** keeps its minutes (frozen) until the player returns, or departs when the player leaves the level? | keeps them: one less rule, and the player who comes back finds the wolf they left; the leash of the next question stops it mattering much |
| Q6 | **Lord of the wild as lasting calls** (§6), or a fourth call (elder elementals)? | lasting: rule 1's "changes what the character can do in a fight"; a bigger beast is a number |
| Q7 | **Scrolls of the calls** (every spell has one): a wizard may learn Call of the wild from a scroll. Keep, or make the calls the druid's alone (`no_scroll`, a new field)? | keep: the spell list's rule is one scroll a spell, and a wizard with wolves is a build |
| Q8 | **The global cap** of 8 allies at once (10 with Lord of the wild) and the per-spell `most` (4, 2, 2) | as written; tune in data |
| Q9 | **The element at 40** chosen by the wild (one draw) or by the caster (four spells, or an aim)? | the wild's draw: the fiction, one talent, one draw |
| Q10 | **Beast-bond's needs**: Handle animal 40 (the druid's and ranger's skill), or Willpower 13 as Beast-friend's? | Handle animal 40: the skill has few readers and this is its home |
| Q11 | **The pack's noise**: a wolf's footsteps 60 wake the floor. Quieter beasts (`footsteps` 30), or the loud pack as the price of a summoner? | loud: a stealthy druid with wolves is the data saying no |
| Q12 | **The ally mark** in the glyph layer (as the unaware mark) or only look mode's "(ally)"? | the mark: two wolves and a warg in a dark corridor must read at a glance; the unaware mark's guard covers it |

## 14. The owner's decisions (2026-11-02, PR #828)

These override the sections above wherever they differ.

| # | Decision | What changes |
|---|---|---|
| Q1 | **Full experience**: summons are spells cast by the summoner, so their kills are the summoner's | `ally_percent = 100` as written |
| Q2 | **Summons should be helpful without much ordering.** Only basic orders in the abilities menu: **Stay**, **Attack a target**, and **default behaviour** | three orders: *Hold here* (`order:stay`), *Set on* (`order:hunt:X,Y`), *Come to heel* (`order:follow`, the default behaviour). **Send away is dropped** (a summon departs when its time runs out or it is slain). The default AI (§4.3) must be good on its own: it follows, fights what threatens the master, and returns to heel |
| Q3 | **Friendly fire: yes**, every area spell hits everything in it | as written |
| Q4 | **Allies may come onto the overworld** and step out at the next location **if the spell's duration is still running** | the retinue stays, but a summon's minutes **keep running** while it is in the retinue (it is not frozen): one that runs out on the road departs and does not step out at the next location |
| Q5 | **Summons follow the player when within 3 cells**; otherwise they stay, frozen, on the level. When the player returns, the summon's duration is checked; an expired summon is removed from the level | `[follow] comes_along = 3`. A summon left behind does not act, but its end is by the clock: its departure time is stored (absolute game time), and on the player's return (or when the level is next run) any summon past it is removed silently. A companion left behind (#825) is not timed |
| Q6 | (the owner asked what this meant; the default stands until the owner says otherwise) | Lord of the wild (level 60) makes the druid's calls **last until slain**, instead of adding a fourth, bigger call |
| Q7 | **Scrolls of the calls stay, but are very hard to learn** | a new per-spell **learning difficulty modifier** (`study_difficulty`, added to the study check's difficulty; `docs/dev/spells.md` "Learning"), set high on the three calls. Any spell may use it |
| Q8 | **Start with a cap of 1 per spell; the druid's abilities raise it as the druid levels** | each call's `most = 1`; the druid's table gains its raises through `ally_mod { most }` (Pack leader at 32 raises every call's `most`, Lord of the wild at 60 again; the D1 slice sets the numbers). The global cap stays as data |
| Q9 | **The element is drawn**, and the draw can be affected (e.g. an item that guarantees fire) | the draw stays; A2 adds one hook for weighting a summon pool's draw (`TalentMods`/item affix reading `summon_bias`), with no item yet |
| Q10 | **Handle animal 40 is too hard to reach** (the skill has no training marks today) | Beast-bond needs **Handle animal 20**; and A1 gives Handle animal its first mark: an order given to an ally, and an ally's kill, each mark it (`Use::Handle`, a chance in `skills.toml`) |
| Q11 | Loud pack: yes | as written |
| Q12 | **A tasteful way to identify friendlies** | the A3 slice designs the mark (subtle, under the effect switches' taste rule; e.g. a faint underline or tint on the glyph), and look mode's "(your wolf)" |

### Revised slices

| Slice | What | Depends on |
|---|---|---|
| **A1 The ally core** | §12's A1 with Q2 (three orders, no Send away), Q4/Q5 (comes along within 3; summons' clock-based expiry, ticking in the retinue), Q10 (Handle animal mark, Beast-bond at 20) | — |
| **A2 Summons** | §12's A2 with Q7 (`study_difficulty`), Q9 (the draw's bias hook) | A1 |
| **A3 UI** | §12's A3 with Q12 (the friendly mark) and the three orders | A1 |
| **D1 The druid** | §12's D1 with Q8 (caps 1, raised by the druid's abilities) and Q7's difficulties on the calls | A1, A2, A3 |

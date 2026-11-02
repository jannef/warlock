# Warlock — The ranger, the pet class: the animal companion

Design document for the owner's review (2026-11-02, #825). Written against `origin/main` at
f3846261 (encounters E5) and **on the ally core of the druid's design** (draft PR #828,
`docs/design/druid-and-summons.md`, its §4: the shared core; §4.9: what is shared and what is
the ranger's; §7.1: `data/allies.toml`; §12: slice A1). Nothing here is implemented; §16 cuts it
into issues. It builds on what is there: the class tables and their actives
(`docs/dev/progression.md` "Class tables", `world/class.rs`, `world/class_ability.rs`), Focus
(`world/focus.rs`, `docs/dev/resources.md`), factions and stance (`docs/dev/creatures.md`,
`World::stance`, `world/beast_friend.rs`), the creature AI and `Mind` (`world/ai.rs`), making
creatures (`travel::make_creature`), level changes (`World::travel`, `free_cell_near`, frozen
floors), displacement (`swap_places`, `docs/dev/npcs.md`), the abilities registry
(`warlock_core::ability`, `docs/dev/screens.md`), the one dialog, the kill tally
(`Quests::kills`, B6), checks (`check.rs`), skills and their benefits (`docs/dev/skills.md`), the
Heal ability (`world/heal.rs`), and the RNG streams (`rng.rs`). Every number is data and hidden in
the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-11-02, #825): "Ranger: let's create design issue for this too. I
think this is our pet class." The ranger becomes the pet class: an animal companion as the
class's core (how it is gained, grows, is commanded, dies, is saved; its AI), filling the table's
gaps 18, 25, and 40 (the gaps report, PR #802) and reworking the table, under the coverage policy
(`docs/dev/progression.md`, rules 1–9 of #776), with no new rebindable keys (CLAUDE.md's hard
rules).

**The dependency.** The druid's design (#828) owns the **ally core** (its §4): what an ally is
(`Actor::ally`), its side, its mind, the four orders, following across levels and the retinue,
experience for its kills, death, the `allies` system and its talents, and `data/allies.toml`.
This document **does not redesign any of it**: §4 here cites what the companion uses and lists
the few things it asks of A1. Everything else here is the ranger's: how the companion is gained,
what the `Companion` source carries, how it grows, and the three class abilities. The witch's
familiar (#830) defers to the same core; nothing here is particular to a hound that a familiar
could not reuse (§4.3).

## Contents

1. Summary
2. Research, tied to the decisions
3. What exists, and what this builds on
4. The ally core (#828 §4): what the companion uses, and what it asks of A1
5. Gaining a companion: the hound, Tame, Let go, the name
6. Orders and the companion's mind: A1's, with the companion's two differences
7. Following, the retinue, left behind
8. Growth: the companion's level is the ranger's
9. Fighting together: kills, Favoured prey, friendly fire, death and replacement
10. The reworked table: what stays, changes, goes; the three new abilities
11. The data
12. Determinism, saves, replays
13. What the player sees: messages, the status bar, look mode, dumps, the CLI, the wiki
14. Talents and class abilities
15. Rejected ideas
16. Implementation plan (slices)
17. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **One companion** | the ranger has one beast at a time, from the first turn: a **hound** comes with the level 1 class ability **Animal companion** (placed beside the player as the run begins, by the class grant's `hand_out`). It is an ally of A1's core (`Actor::ally`, #828 §4.1) with `source = Companion(CompanionData { name, grade })`; `World::companion()` is the player's one ally of that source. §5 |
| **Tame** (the ability, part of Animal companion) | `tame:DIR`: an adjacent `tameable` beast of level at most the ranger's + 3 that is **not hostile to you or has not noticed you** (`World::unaware`; Stalker's eye at 2 shows which): ten turns beside it (interrupted when the ranger is hurt or moves), then one check (`check::check`): Willpower modifier + Handle animal's new `tame` benefit (+ 4 with a food stack offered, one eaten from the pack) against 13 + 2 × (its level above yours, if any). Made: it is yours, named. Failed: it is roused ("The wolf snaps at you."), and nothing stops you trying again (time is the penalty). Refused with a companion already: **Let go** first. §5 |
| **Orders** | A1's four, in the Allies group of the abilities menu (#828 §4.4; no `KeyCommand`): Come to heel, Hold here, Set on (a cell), Send away. For a companion **Send away is Let go**: the one dialog, then it is a wild thing again (A1's refusal line is replaced by R1). An order takes no time and makes a voice's noise. §6 |
| **The mind** | A1's ally arm (#828 §4.3): the master's stance, heel 2, leash 8, Hold's reach 1, Set on's reach 3, doors and traps as any creature's. The companion adds nothing to it in R1; Hunter's bond (25) adds the whistle (§10.2). §6 |
| **Following** | A1's rule (#828 §4.5): allies beside the player (`comes_along` 1) come along; on the overworld they ride in the retinue. The companion asks A1 for one hook, `comes_along` on `ally_mod`, so Hunter's bond can widen it to 6. Left behind, it waits where it was (A1) and answers Hunter's bond's whistle when the player is back on its level. §7 |
| **Growth** | the companion's level **is the ranger's class level** (`grade_percent` 100, re-set at every level-up: "Snel grows stronger."), kept in `CompanionData::grade`. Six hooks read a grade above the type's level with `data/companions.toml [growth]`: hit points +2 a level, +1 to hit per 4, +1 damage per 6 and one more damage die per 12, +1 Defence per 6, natural Protection by the body formula at its grade (`docs/design/combat-progression.md` §8), St/Dx/To +1 per 5. Read in the places A1's `ally_mods` are read, beside them. Summons do not grow (their ladder is the spell's). §8 |
| **Kills** | A1's (#828 §4.6): a companion's kill pays the ranger as their own (XP at `ally_percent`, the first kill, the quest tally, **Favoured prey's record**, the builds). Nothing added. §9 |
| **Friendly fire** | A1's (#828 §4.7): the ranger's bursts and bolts, and a Volley's missiles, hurt the companion in their way ("Snel yelps."); walking into it swaps places. §9 |
| **Death** | "Snel dies." A corpse (A1 §4.8: a companion is no summon), the bond ends, nothing else happens to the ranger (no grief status); the ranger tames another when they find one. No revive, no stable, no cooldown. §9 |
| **The table** | 1 **Animal companion** (new), Favoured prey, Trained wind; 2 Stalker's eye; 6 Tracker, Barkskin; 12 Rapid shot; **18 Pack tactics** (new: a creature beside both of you is flanked, +2 to hit for both; the companion shares your Favoured prey); **25 Hunter's bond** (new: you see what your companion sees; Come to heel is a whistle it answers from anywhere on the level; it comes along from 6 cells; your Heal tends it); 32 Evasion; **40 Sic 'em** (new, an active on 5 Focus, aimed at a creature you see: the companion is set on it at half again its speed and its next three hits are criticals); 50 Volley; 60 Master hunter (its Favoured prey half reaches the companion through Pack tactics). Nothing goes; nothing else changes. §10 |
| **Content** | the `hound` (the starting beast, `stray = false`), `tameable = true` on A1/D1's `wolf` and `bear` and on `rat`, `bat`, `deer`, `beetle`, `warg`, `dire_warg`; two more wild beasts, `boar` and `lynx`, in the forest local areas and the Ascmearc's wildlife encounters. §11 |
| **Talents** | **none new**: the `allies` system and its pickable **Beast-bond** are A1's (#828 §7.4, §10); the ranger's three new class abilities carry the tag where they read the core. The druid's class ability is named **Pack leader**; this document's 25 is **Hunter's bond**, so no name collides. §14 |
| **Slices** | R1 the companion (on A1: Animal companion, Tame, Let go, the hound, `CompanionData`, growth, the data, the dumps, the table's level 1, the wiki), R2 Pack tactics, R3 Hunter's bond, R4 Sic 'em, R5 the boar and lynx and `tameable`, R6 the balance probe row. §16 |

**Key choices, and why**

1. *The companion is the level 1 signature, not the level 18 filler.* "Our pet class" means the class is felt from the first turn as a hunter and a hound; a companion that arrives at 18 would make the ranger an archer for seventeen levels and then something else. NetHack's role pet and WoW's level 10 Tame Beast are the model: the pet is there from the start, and the later abilities make it better.
2. *One beast, grown, not a pack.* Diablo 2's mercenary and WoW's pet level with their master and are the player's one companion; the D2 druid's wolves are *summons*, the druid's design. One companion keeps the fiction clean (a ranger and a hound) and the two classes distinct: the druid calls many for a while, the ranger keeps one for good.
3. *The companion's level is the ranger's.* A pet that stays level 1 (ADOM's, until it kills enough on its own) is dead weight by the fourth floor; a second experience curve (NetHack's) is bookkeeping the player never sees. One number, read live, and a growth table in data (§8), as a hireling in D2 "levels with you". Summons need none of it: their power is the spell's ladder (#828 §5.1), so growth is the one piece of actor state that is the ranger's alone.
4. *Tame reuses surprise.* WoW tames a hostile beast that fights back during the channel; here the beast must be neutral or unaware (`World::unaware`, the one definition, surprise design), so Stalker's eye at level 2 (you see who has not noticed you) is the tamer's tool, and no new "calmed" state is needed.
5. *The core is the druid's, used as it is.* Orders, the mind, following, the retinue, the kill credit: A1 builds them for a pack, and a hound is a pack of one. The companion asks A1 for one hook (`comes_along` on `ally_mod`) and one change of words (Send away is Let go); everything else it adds is read in R1's own modules.
6. *No tameness, no feeding, no happiness.* NetHack's tameness decay and classic WoW's pet happiness were removed by their own designers or hated; a hunger the creatures do not have anyway is not simulated for the companion. The companion is loyal; the cost of having it is keeping it alive.

## 2. Research, tied to the decisions

| Source | What it does | Taken | Left, and why |
|---|---|---|---|
| ADOM (pets by Handle Animal and food, the puppy; pets follow when adjacent on stairs; pets level by their own kills; pets eat corpses) | taming by giving food, companions that walk the wilderness map with you | food as a +4 on the tame check (§5); A1's retinue crosses the overworld (§7) | a pet's own XP (a level 1 dog at dungeon level 10); feeding (not simulated for creatures) |
| NetHack (the role's starting pet; tameness kept by feeding; follow when adjacent; the magic whistle; pets steal from shops; pets refuse to attack much stronger foes) | the starting pet from turn one; the whistle that brings the pet to you | the hound at level 1 (§5); Hunter's bond's whistle at 25 (§10.2) | tameness decay; shoplifting pets; a pet's fear of strong foes (the companion fights what its master fights; its grade keeps it able) |
| DCSS (allies from summons, necromancy, Beogh's orcs; `t` orders free of time; allies' kills give XP) | the orders and the kill credit, which A1 took (#828 §2) | — here; this document adds no order | the retreat order (Come to heel does it) |
| Diablo 2 (hirelings: one at a time, level with you, equipped, resurrected for gold) | one companion that levels with its master | the grade following the ranger's level (§8) | resurrection for gold (permadeath world: tame another); equipping the companion (no slots on a hound; §17 Q9) |
| WoW hunter (Tame Beast: a channel on a beast at or below your level; one active pet, a stable; the pet's level follows the hunter's; Mend Pet; Kill Command, Bestial Wrath, Eyes of the Beast; happiness and feeding, later removed) | the level gate on taming, the pet's level bound to the master's, an unleash active, seeing through the beast, tending it | `over` 3 levels (§5), `grade_percent` 100 (§8), Sic 'em (§10.3), Hunter's bond's shared sight and tending (§10.2) | the stable (one beast; let go and tame anew), Revive Pet, happiness, pet talents (the companion's growth is the ranger's table) |
| D&D 3.5 ranger (animal companion at level 4, half the druid's progression, tricks by Handle Animal) and 5e Beast Master (the pet attacks only when the ranger spends their action; the subclass widely judged the weakest) | Handle Animal as the skill | the `tame` benefit on Handle animal (§5), which gets its first use | half progression (the data allows it, §17 Q1; the default is full), a pet that costs the master's action (A1's ally acts on its own turn, always) |

## 3. What exists, and what this builds on

A1 (#828 §3, §4) is assumed built; this table is what the ranger's own parts stand on.

| Part | Today | Used for |
|---|---|---|
| The class tables | `ClassDef::abilities` (level, talent id); `class::grant` gives level 1's at `start_run`, each level's at `level_up`, and on `from_state`; a given talent's effects through `hand_out` (hit points, stat gifts, `grants_resource`) | Animal companion at 1 brings the hound through `hand_out` (one more arm: `companion`); Pack tactics, Hunter's bond, Sic 'em at 18, 25, 40 |
| Class actives | `ClassActive` (`content/class.rs`), `use_class` / `use_class_at` (`world/class_ability.rs`, `world/volley.rs`), `Ability::Class`, costs on a resource (`TalentDef::resource`) | Tame (`ClassActive::Tame`, `Aim::Direction`), Sic 'em (`ClassActive::Sic`, `Aim::Cell`, 5 Focus) |
| Focus | granted by Rapid shot (12), max 10, regenerates 1 a base turn, not while resting; Volley spends 6 | Sic 'em's cost |
| The ally core (A1) | `Actor::ally: Option<Ally { master, source, order, retinue }>`, `AllySource::Companion` "a variant with no data of its own yet" (#828 §4.9), `World::allies_of`, `ally_plan`, `follow_along`, the retinue, `ally_mods`, `Action::Order`, `Source::Ally`, `data/allies.toml` | everything the companion is, but for `CompanionData`, Tame, growth, and the three abilities |
| Making a creature | `travel::make_creature` (the one way; `CreatureSpawn`, gear rolled or not, uniques remembered, `World::spawned`) with `faction_override` | the hound at the start (an ally as A1 makes a summon: `summon_for`'s making shared, #828 §12 A2) |
| Checks | `check::check(rng, modifier, difficulty)`, 2d12, two gameplay draws | the tame check |
| Skills | Handle animal exists (`kind = "action"`, trained by the ranger 2d4+4, the barbarian, druid, paladin, witch, aristocrat, commoner, merchant), with no use yet; benefits are read in one place each; `Ability::use_of` lists a skill's use in its place | the `tame` benefit (Handle animal), read in `world/tame.rs`; marks on `tame`. A1's Beast-bond needs Handle animal 40: the skill now has a use and a talent |
| Unaware, surprise | `World::unaware(a, b)` (the one definition), `Mind::noticed`, Stalker's eye's `sees_unaware` | who may be tamed |
| Natural Protection by body and level | the data rule (`combat-progression.md` §8; `content/creature_body_tests.rs::Body::of`): hide (L − 10) ÷ 8, bony L ÷ 6, armoured L ÷ 4 + 1 | the companion's Protection at its grade |
| Perception | the snapshot built from the player's eyes (`perception.rs`, `sight.rs`); creature sight computed per creature with the same rules; `PerceptionView::allies()` (A1) | Hunter's bond adds the companion's seen cells (§10.2); the status bar's line |
| The Heal ability | `heal[:ID]`: the player treats their own bleeding, a poison, a disease, with the Heal skill (`heal::treatable`) | Hunter's bond's tending (§10.2) |
| Statuses that ease by actions | `docs/dev/weapon-abilities.md`: a status spent a degree per action of a kind | `savage` (Sic 'em) spent a degree per hit |
| The kill tally | `Quests::kills`, `Quests::prey` (Favoured prey's record), credited to the master for an ally's kill by A1 (#828 §4.6) | Favoured prey learns from the hunt together: nothing to add |
| The local areas' deer, the encounters' wildlife entries, D1's wolf and bear | `deer` (faction `wildlife`); the Ascmearc's `"wildlife"` entries; `wolf` (4) and `bear` (12) from #828 §7.3, `wildlife`, tags `beast`, `animal` | the beasts to tame; the boar and the lynx join them |

## 4. The ally core (#828 §4): what the companion uses, and what it asks of A1

### 4.1 Used as it is

| A1 piece (#828) | The companion's use |
|---|---|
| `Actor::ally`, `Ally { master, source, order, retinue }` (§4.1) | the hound is an ally with `master = player`, `source = Companion(CompanionData)`, saved and hashed with the actor as A1 saves every ally |
| the side: `faction_override = player`, the master's stance, `disturbs_rest`'s exception, the quick leave, coarse turns (§4.2) | a hound beside a sleeping ranger disturbs nothing; monsters hunt it as they hunt the player; a Beast-friend's ranger has a hound that leaves rats be |
| the ally arm: target by the master's stance, fight, the order, the leash; doors by `opens_doors`, traps and water as any creature's (§4.3) | the companion's whole mind; R1 adds no step. A hound opens no door and waits for the ranger to |
| the four orders, `Source::Ally`, the Allies group, `Action::Order`, `order:follow|stay|hunt:X,Y|dismiss`, no time, a voice's noise, `Unavailable::NoAllies` (§4.4) | the companion obeys them as the one member of the pack; Send away: §4.2 below |
| following beside the player (`comes_along` 1), the retinue on the overworld, stepping out, the encounter placement (§4.5) | the companion crosses the map in the retinue and comes into an encounter's area beside the player |
| experience, the quest tally, Favoured prey, the builds for an ally's kill at `ally_percent` (§4.6) | the hunt is one; nothing added |
| friendly fire and healing (§4.7) | the ranger's Barkskin is `self`; their bursts and shots hurt the hound in the way |
| death leaves a corpse (§4.8) | "Snel dies.", a hound corpse |
| `World::ally_mods(id)` read in the ally's profile and Protection (§7.4) | the growth hooks sit beside those reads (§8) |
| `SeenActor::ally`, the ally mark, `PerceptionView::allies()`, the `allies` dump layer, `wiki/src/allies.md` (§9) | the companion's rows and words are added to them (§13) |
| the `allies` system, the pickable Beast-bond (§7.4, §10) | the ranger's class abilities carry the tag; this document mints no pickable (§14) |

### 4.2 Asked of A1 (requests, not redesigns)

| # | Request | Why | If A1 declines |
|---|---|---|---|
| A | `comes_along` as a field of `AllyModSpec` (`ally_mod { comes_along = 5 }`), summed into the master's `TalentMods::ally` and added to `[follow] comes_along` in `follow_along` | Hunter's bond (25) widens the companion's following to 6 cells; one hook on the data A1 already reads | R3 adds the read itself, one line in `follow_along` |
| B | Send away on a companion: A1 plans the refusal "Your hound is no beast to send off." and says #825 may change it. **It does**: Send away on the player's companion is **Let go**, through the one dialog (§5.3); on a pack of summons it departs them as A1 says; with both, the summons depart and the companion is asked about | one order, as A1 wants; a companion that cannot be released blocks Tame forever | R1 replaces the refusal arm |
| C | `AllySource::Companion(CompanionData)` where A1 has `Companion` with no data: `CompanionData { name: String, grade: u8 }` | A1 names this as the ranger's (§4.9); listed so the variant's shape is agreed before A1 lands | R1 changes the variant; no save of A1's holds a companion yet |
| D | The `comes_along` default: this document would set **2**, not 1 (adjacency is the classic complaint of ADOM and NetHack players, and a two-cell corridor dance is the usual way a pet is lost) | the owner's call in #828's Q-table; the companion works at either | nothing: Hunter's bond widens it anyway |

### 4.3 The ranger's own (A1 §4.9's right column, answered)

| A1 asks | This document says |
|---|---|
| what `Companion` carries (a name, a level, hunger, a bond) and how it is saved | `CompanionData { name, grade }`: the name (§5.4) and the growth level (§8); no hunger (§15); the bond is the variant itself: at most one ally of source `Companion`, `World::companion()` |
| a companion's own behaviours, if any | none in R1; Hunter's bond's whistle (§10.2) is the one addition to the arm, read by it as a `Follow` whose master is the goal wherever they are |
| what Send away does to a companion | Let go (§4.2 B, §5.3) |
| whether a companion left behind does more than wait | it waits (A1); with Hunter's bond the player back on its level whistles it (Come to heel), and nothing crosses levels on its own |
| the ranger's class abilities on these hooks or its own | Pack tactics and Sic 'em on hooks of their own (`world/pack.rs`, `world/sic.rs`), Hunter's bond on `ally_mod.comes_along` (A) and hooks of its own |
| how a companion is gained and grows | §5, §8 |
| the companion's own rows and words | §13 |

A witch's familiar (#830) would be another `AllySource` variant with data of its own; nothing in
`CompanionData`, Tame, or growth assumes a hound, and the hooks of §8 could serve a familiar
that grows with the witch if that design wants them (it would reuse `[growth]` or carry its own).

## 5. Gaining a companion: the hound, Tame, Let go, the name

### 5.1 The hound at level 1

The class ability **Animal companion** (`animal_companion`, tree `class`, `granted = true`,
effects `[{ kind = "companion", creature = "hound" }, { kind = "class_active", active = "tame" }]`,
systems `allies`) is given at `start_run` as every level 1 entry is. Its `companion` effect is
one more arm of `class::hand_out`: `World::bring_companion(creature)` makes an actor of that
type beside the player as A1 makes a summon (`make_creature` with `faction_override = player`,
no gear; `free_cell_near` the player), sets `ally = Ally { master: player, source:
Companion(CompanionData { name, grade: the player's level }), order: Follow, retinue: false }`,
and names it (§5.4). On `from_state` nothing is made (a saved run has its own, or lost it); the
arm does nothing when `World::companion()` is `Some`. So every ranger run has a hound from the
first turn, the scenarios' rangers too (the showcase's Osric on the overworld, whose hound is in
the retinue, and Maren in the forest glade, whose hound stands beside her; their replays and the
goldens that show them, `overworld.toml`'s and `sky.toml`'s forest entries, change once,
deliberately: §12).

### 5.2 Tame

`Ability::Class(ClassActive::Tame)`, "Tame" in the menu's Class group, `Aim::Direction`,
`tame:DIR`. The one place is `world/tame.rs`.

| Step | Rule |
|---|---|
| Who | an actor adjacent in that direction whose type is `tameable = true` and whose level is at most the ranger's class level + `over` (3); the ranger has no companion (`World::companion()` is `None`, else "You already have a companion." — Let go first); the beast is **not hostile to you** (`World::stance(player, it)` neutral: wildlife, or an animal a Beast-friend calmed) **or has not noticed you** (`World::unaware(it, player)`). Else refused, no time: "The warg has seen you: it will not be tamed now." |
| The channel | `turns` (10) actions of the ranger beside it, as a dig's turn loop (`dig_turn`): each a normal action's price; interrupted when the ranger is hurt, moves, or the beast moves away, with nothing lost but the time |
| Food | if the ranger carries food, one item of the first food stack is eaten by the beast (`food` + 4 on the check; "You hold out some dried meat."); none, no bonus |
| The check | `check::check(gameplay, Willpower modifier + Handle animal's tame benefit + food, difficulty 13 + per_level_above 2 × max(0, its level − the ranger's level))`. Made: it is yours (§5.3's reverse). Failed: "The boar snaps at you and backs away." — its `Mind::noticed` is set (it knows you now; a neutral one stays neutral, an unaware hostile is aware), and nothing more: you may try again when it is calm or unaware again (no cooldown: time is the penalty). A natural 2 fails and a 24 succeeds, as every check |
| Marks | Handle animal marks on `tame` (`mark = [{ on = "tame", chance = 30 }]`), made or failed |

A beast tamed keeps its actor (hit points, statuses, its place), takes `faction_override =
player` and `ally` as the hound does (grade = the ranger's level: its hit points rise with its
maximum), the name, and `Mind` cleared. "The wolf comes to your hand. You call it Græg."

### 5.3 Let go (Send away on a companion)

A1's Send away (`order:dismiss`) on the player's companion opens the one dialog: "Let Græg go?" /
"It will be a wild thing again." / "Let it go" (the `Order::Dismiss` action) and "Keep it"
(nothing). Let go: `ally` is cleared, `faction_override` returns to the type's, its hit points
are capped at the type's maximum, the name is forgotten; it is a creature of its kind again
("Græg looks back once, and goes."). A `wildlife` wolf is neutral; a warg's type is `orcs`,
hostile: a let-go warg is a warg again, which is the player's own doing (the dialog said so).
With summons on the level too, they depart as A1 says and the dialog asks about the companion.

### 5.4 The name

Drawn from `data/companions.toml` `names` (Old English, the world's register for old things:
Snel, Brūn, Græg, Blæc, Hwīt, Lēof, Hræfn, Wulf, Fæger, Beald, Hraþe, Fugol; the owner's #378
review may rename) on the stream `companion/<actor id>` (one draw), so no other roll moves.
`World::called` says the name without an article, as a unique character's (`docs/dev/npcs.md`):
"Snel bites the kobold." A1's "your wolf" becomes the name for a companion; a dead companion's
corpse is "a hound corpse" (the type's), not the name.

## 6. Orders and the companion's mind: A1's, with the companion's two differences

The orders are A1's (#828 §4.4), the mind A1's arm (§4.3), the constants `data/allies.toml`'s
(`[follow] heel = 2`, `leash = 8`, `[stay] reach = 1`, `[hunt] reach = 3`, `[orders] noise = 60`).
A hound at heel keeps within 2 cells; set on, it goes and fights what is within 3 of the cell,
then looks back and heels; holding, it fights what comes within 1. The two differences:

1. **Send away is Let go** (§5.3).
2. **Hunter's bond's whistle** (25, §10.2): a Come to heel order reaches the companion wherever it
   is on the player's level: its `Follow` goal is the master's cell even out of sight (today a
   `Follow` ally routes to the master it is near; out of reach it would search as a creature does
   toward a noise). One line in `ally_plan`'s step 3 for an ally whose master has `whistle`.

Everything a creature's turn does, the companion's does: footsteps (40 for the hound), statuses,
Surprised, the coarse turns beside a resting ranger (A1 §4.2: an ally rests when the player
rests). The companion adds no draw to any stream.

## 7. Following, the retinue, left behind

A1's rule (#828 §4.5), used as it is: an ally beside the player (`comes_along` 1; request D asks
for 2; Hunter's bond adds 5 through request A) when they take stairs, an edge, the quick leave, a
collapse, or an encounter's entry comes along; onto the overworld it joins the **retinue**
(carried, never drawn, no turns) and steps out at the next level; "Snel comes with you." /
"Snel is left behind." Hold here means stay: a companion holding is not beside the player and
stays. Left behind, it waits where it was, frozen with its level; the player back on that level
has it come to heel when beside it (A1), or whistles it from anywhere on the level with Hunter's
bond. A companion on a level the player can never return to is lost: tame another.

## 8. Growth: the companion's level is the ranger's

`CompanionData::grade` is the companion's own level. It is `grade_percent` (100) of the ranger's
class level, at least the type's level, set when the companion is gained and at every `level_up`
(one line in `World::level_up`: "Snel grows stronger." when it rises). Summons have no grade
(#828 §5.1: their count and kind are the spell's ladder), so the hooks below read nothing for
them: they sit beside A1's `ally_mods` reads and cost an ally without a grade nothing.

| Hook | Where | Rule (the defaults, `data/companions.toml [growth]`, `above` = grade − the type's level, never below 0) |
|---|---|---|
| hit points | `Actor::max_hp` | `base_hp + hp_per_level × above` (`hp_per_level` 2); the current hit points rise with the maximum |
| to hit | the attack profile (`Actor::stanced`), beside `ally_mods.to_hit` | `+ above ÷ to_hit_every` (4) |
| damage | the attack profile | `+ above ÷ damage_bonus_every` (6) flat, and `+ above ÷ damage_dice_every` (12) more dice of the attack's die |
| Defence | `Actor::defence` | `+ above ÷ defence_every` (6) |
| Protection | `Actor::protection`, beside `ally_mods.protection` | the body formula at the grade (hide: (grade − 10) ÷ 8, never below the type's own) |
| stats | `Actor::stats` reads for checks and burden | St, Dx, To `+ above ÷ stat_every` (5) |

Speed, senses, size, and resistances are the type's. So the hound (level 1, 1d6 bite, 6 hit
points, Defence 1 by its hide) beside a level 30 ranger is a level 30 hound: 64 hit points, +7 to
hit, 3d6+4, Defence 5, Protection 2; a wolf (4, 16 hit points, 1d6) tamed at 4 and carried to 30
has 68, +6, 3d6+4. The wiki's companion page prints this table for every tameable beast at the
cadence levels, and `cargo test balance_probe` gains a row: the ranger and their hound against
the monsters of their level (`docs/balance.md`, R6).

Nothing of the companion's is an XP curve: no kills counted for it, no level of its own to show.

## 9. Fighting together: kills, Favoured prey, friendly fire, death and replacement

| Topic | Rule |
|---|---|
| **A companion's kill** | A1's (#828 §4.6): paid to the ranger as their own at `ally_percent` (XP with the scaling and the first kill, the quest tally, **Favoured prey's record**: the hound's goblin is your goblin, the named monsters, the builds). Nothing added, nothing excluded |
| **Pack tactics (18)** | §10.1 |
| **Friendly fire** | A1's (#828 §4.7): a burst or bolt the ranger casts or reads hurts the companion in its radius or line as it hurts anything; a shot or thrown thing strikes the first creature in its path, the companion too; a Volley's three shots the same. "Snel yelps." Walking into it swaps places (never a blow); `fire:X,Y` at its cell is allowed, with no dialog |
| **The companion struck** | as any creature: hit points, statuses (bleeding, poisoned), Surprised; it regenerates by its Toughness as creatures do, and rests when the ranger rests. The Heal ability keeps treating the player alone until **Hunter's bond** (25, §10.2) lets it tend the companion: one talent, not a general rule |
| **Death** | "Snel dies." A corpse (A1 §4.8) and nothing else: `World::companion()` is `None`; no grief status (a flaw that bites nowhere but in morale is none, rule 5), no XP lost, no cooldown. The ranger tames another when they find one: the forest glades, the hills, the dungeon's vermin (a rat companion is a rat companion). Time is the penalty |
| **Traps** | A1's: the companion springs a trap as any creature does, known to the player or not ("known traps are the player's knowledge, not the wolf's", #828 §4.3). A ranger who leads a hound over a trapped corridor holds it first |
| **Encounters** | A1's placement: the companion steps out of the retinue beside the player before the party is placed; "Lie in wait" sets `Mind::missed` on the party, not on it |
| **Stealth** | the companion's footsteps are heard as any creature's; Hold here lets a ranger scout alone and call it (Come to heel) when the fight starts |

## 10. The reworked table: what stays, changes, goes; the three new abilities

| Lv | Entry | Status | Notes |
|---:|---|---|---|
| 1 | **Animal companion** | **new** | the hound, Tame; §5 |
| 1 | Favoured prey | stays | the companion's kills feed its record (A1 §4.6) |
| 1 | Trained wind | stays | the martial classes' stamina |
| 2 | Stalker's eye | stays | now also the tamer's tool: who has not noticed you may be tamed |
| 6 | Tracker | stays | |
| 6 | Barkskin | stays | the ranger's one spell; the druid has it at 3 too (a table may give a spell two classes have) |
| 12 | Rapid shot | stays | grants Focus, which Sic 'em spends |
| 18 | **Pack tactics** | **new** | §10.1 |
| 25 | **Hunter's bond** | **new** | §10.2 (not "Beast-bond", which is A1's pickable) |
| 32 | Evasion | stays | |
| 40 | **Sic 'em** | **new** | §10.3; Focus from Rapid shot (12) is its price |
| 50 | Volley | stays | |
| 60 | Master hunter | stays | its Favoured prey half (+1/+2 more) reaches the companion through Pack tactics; nothing is edited |

Nothing goes. The gaps report's "Direction: stealth" for 40 (Camouflage, cut as outdoors-only)
is not followed: the owner's direction for the class is the pet, and a stealth passive would be
Camouflage's twin underground.

### 10.1 Pack tactics (18, passive)

`pack_tactics`, effects `[{ kind = "pack_tactics", to_hit = 2 }, { kind = "shares_favoured_prey" }]`,
systems `combat`, `allies`. Read in `World::resolve` → the attack profile, one hook (`world/pack.rs`):

- a creature adjacent to **both** the ranger and their companion is **flanked**: the ranger's
  melee and the companion's blows on it get `+to_hit` (2). Missiles not (flanking is about
  footing; a bow from across the room is not in the pack);
- the companion's blows carry the ranger's **Favoured prey** bonus against the ranger's favoured
  type (+1 to hit, +2 damage; +2/+4 with Master hunter at 60): `shares_favoured_prey` reads the
  ranger's `TalentMods::favoured_prey` for the companion's profile when the target is the type.

"A hunter and a hound work a foe between them": D&D's pack tactics, D2's merc beside you. It
changes every fight on a dungeon floor from 18 on; it reads combat, which has its talents. It is
no clone of A1's Beast-bond (`ally_mod.to_hit` on every blow): flanking is conditional and
two-sided, and the alike check compares effect kinds, which differ.

### 10.2 Hunter's bond (25, passive)

`hunters_bond`, effects `[{ kind = "shared_sight" }, { kind = "whistle" }, { kind = "ally_mod", comes_along = 5 }, { kind = "tends_ally" }]`,
systems `vision`, `allies`, `hit_points`. Four things the bond does, each one hook:

| Effect | Hook | What |
|---|---|---|
| `shared_sight` | `perception.rs`: after the player's field of view, the companion's cells (`sight::sees_by` with its eyes and `sight`, the light as it perceives it) are added as **seen**; what stands on them is seen; memory remembers them (`world/bond.rs`) | you see what your companion sees: set it on a cell round the corner and look. Light the companion sees by is the light that matters there (its eyes: a hound's low-light, a bat's darkvision). Hidden traps and secret doors are not found by it (the search rules are the player's; the companion's eyes are eyes, not a search). A dazzle or blindness of the player leaves the companion's view. Not for summons: the bond is the ranger's |
| `whistle` | `ally_plan` step 3 (§6): a Come to heel order gives the companion the master's cell as its goal wherever it is on the level | NetHack's magic whistle, without the item: the hound you left at the stairs comes when called |
| `ally_mod.comes_along` (request A) | `follow_along`: `comes_along` + 5 (6 cells) | it need not be at your heel when you take the stairs |
| `tends_ally` | the Heal ability gains the companion adjacent as a target when it has a treatable status (`heal::treatable` lists it after the player's own; `heal:bleeding` treats the player's first, the companion's when the player has none of that kind; the player's Heal skill rolls) | WoW's Mend Pet, through the skill that exists. A1's druid heals by touch spells (#828 §4.7); the ranger has no such spell, so the skill is the way |

Shared sight is the capability players remember of a pet class (WoW's Eyes of the Beast). It is
a change to what the character *can do* (scout with the hound), not a readout, so rule 8 is not
touched; it reads `vision`, which has its talents.

### 10.3 Sic 'em (40, active)

`sic`, `ClassActive::Sic`, `Aim::Cell` (a creature you see, within `range` 10), `use:sic:X,Y`,
`resource = { id = "focus", cost = 5 }`, effects `[{ kind = "sic", blows = 3, speed_percent = 50 }]`,
systems `combat`, `resources`, `allies`. Refused with no companion on the level, a cell with no
creature the player sees, or the target not hostile to the player; the cost is spent after the
checks. It does two things (`world/sic.rs`):

1. the companion takes A1's `Hunt { at: the target's cell }` (and keeps hunting that creature
   while it sees it: the hunt cell follows the target, one line in the arm for a hunt begun by
   Sic 'em);
2. the companion gets the status **Savage** (`data/status.toml`: `savage`, degrees 3, no tick,
   eased by hits: each blow it lands spends a degree; eases 60 s a degree otherwise; `speed_percent`
   +50 while it lasts): each of its next three blows that hit is a **critical without the
   confirmation roll** (`combat::strike_with`: the status's `auto_critical`, read where the
   threat is rolled).

The cost is 5 of 10 Focus (Volley 6, Rapid shot 2 a shot), so a ranger at 40 chooses: a volley,
or the hound's throat-hold, not both in one turn. It is WoW's Bestial Wrath and Kill Command in
one, and it is a fight changer at a capstone tier (rule 1: never economy, quests, or XP).

## 11. The data

### 11.1 `data/companions.toml` (shared, one file; parsed by `content/companion.rs`)

The allies' constants are `data/allies.toml` (A1, #828 §7.1); this file holds only the
companion's own.

```toml
# The ranger's animal companion (spec §8 "Class abilities"); the allies' rules are allies.toml.
[tame]
turns = 10            # actions beside the beast before the check
over = 3              # a beast's level may exceed the ranger's by this much
difficulty = 13
per_level_above = 2
food = 4              # the bonus for food offered (one item of the first food stack)

[growth]
grade_percent = 100   # of the ranger's class level
hp_per_level = 2
to_hit_every = 4
damage_bonus_every = 6
damage_dice_every = 12
defence_every = 6
stat_every = 5

names = ["Snel", "Brūn", "Græg", "Blæc", "Hwīt", "Lēof", "Hræfn", "Wulf", "Fæger", "Beald", "Hraþe", "Fugol"]
```

Checks at load: every number in range (`turns` 1 to 100, `over` 0 to 20, `grade_percent` 1 to
300, the `every`s 1 to 60), at least one name, the `savage` status present with degrees ≥ 3 and
no tick. Without the file no companion can be made (fixtures without it hash as before).

### 11.2 Creature types

| Field | What |
|---|---|
| `tameable = true` (new, `CreatureDef::tameable`, default false) | the type may be tamed; refused at load on a unique, an animated object, a type with `summon`, a type that `opens_doors` (a beast), or one of a faction with no `animal = true` |
| `hound` (new, shared; level 1, xp 15 by the guideline, `d`, speed 1200, faction `wildlife`, tags `beast`, `animal`, `stray = false`, hearing 20, footsteps 40, low-light vision, 6 hit points, 1d6 bite, hide; `tameable`) | the starting beast; in no population and no table |
| `tameable` on D1's `wolf` (4) and `bear` (12) (#828 §7.3), and on `rat`, `bat`, `deer`, `beetle`, `warg`, `dire_warg` | a level 1 ranger may tame a wolf; a level 9 one a bear; one in the barrows a giant rat; one who stalks the orcs' kennels, a warg (7, at a ranger's 4 or more; it must be unaware). Not the undead hounds, the hell hound, the blink dog (fey), the basilisk |
| `boar` (3, forests and hills; a 1d8 gore), `lynx` (5, forests; quiet, footsteps 20) (new, shared, `wildlife`, `beast`, `animal`, `tameable`; `docs/dev/creatures/beasts.md`, D1's file) | two more wild beasts: placed by the forest and hill local generators as the deer are, and as `"wildlife"` entries in the Ascmearc's encounter tables (a boar by day in the woods; a lynx at dusk in the deep woods) |

Adding creature types invalidates saves (`docs/dev/saves.md`): the PR says so and leaves
`FORMAT_VERSION` alone.

## 12. Determinism, saves, replays

- **Streams**: the tame check on the gameplay stream (two draws, only when a check is rolled);
  the name on `companion/<actor id>` (one draw, `Rng::stream`); the growth draws nothing; the
  orders and the arm draw nothing (A1).
- **Saved**: `CompanionData { name, grade }` inside `Actor::ally` (A1's field; nothing more on
  the actor); nothing on `WorldState` (`World::companion()` is derived from the actors). A
  non-ranger run saves byte for byte as before (A1's guarantee holds). No `FORMAT_VERSION` bump
  for the fields; the new creature types bump nothing either (the integrator's rule), but saves
  made before them are invalid as `docs/dev/saves.md` says.
- **Hashes**: a run without a ranger hashes as before. Every **ranger** run gains an actor at the
  start, so the replays of ranger runs are re-created (`replay --create`), and the two showcase
  scenarios with a ranger (`world`, Osric; `forest`, Maren) change: Osric's hound rides in the
  retinue (not drawn: `overworld.toml`'s captures may not change at all), Maren's stands beside
  her (`sky.toml`'s forest entry changes once and is updated deliberately in R1's finisher pass,
  "Golden screenshots"). No new golden: the companion is a glyph a core test and a dump can see.
- **Replays**: `tame:DIR` and `use:sic:X,Y` in the action text (CLAUDE.md's list, `action.rs`);
  the orders are A1's.
- **Invariants** (the tests of R1): a tamed beast's grade is the ranger's level and rises with
  it; a let-go beast has no `ally` and its type's faction; Tame is refused with a companion, on a
  beast that has noticed the ranger and is hostile, on one too strong, and on a type not
  `tameable`; the check's modifier is the Willpower modifier plus the skill's benefit plus the
  food; the name is drawn once per companion; a non-ranger run's hash is unchanged by the PR; a
  save round trip keeps `CompanionData`; a replay with a tame, three orders, a level change in
  and out of the retinue, and a death verifies.

## 13. What the player sees

### 13.1 Messages (words, never a number; A1's with the name in place of "your wolf")

| Moment | Text |
|---|---|
| the start | "Snel, your hound, pads at your heel." |
| taming | "You crouch by the wolf and hold out some dried meat." (the first turn) … "The wolf comes to your hand. You call it Græg." / "The wolf snaps at you and backs away." |
| the stairs (A1) | "Snel comes with you." / "Snel is left behind." |
| the retinue (A1) | "Snel follows you in." |
| growth | "Snel grows stronger." |
| a blow (A1) | "Snel bites the kobold." / "The kobold bites Snel." |
| hurt by you | "Snel yelps." |
| death (A1) | "Snel dies." |
| let go | "Græg looks back once, and goes." |
| Sic 'em | "Snel goes for the goblin's throat." |

### 13.2 The status bar, look mode, the screens

- **The status bar** gains a short line while the player has a companion: its name and health
  in words ("Snel: unhurt", "Snel: badly hurt"; the `health` words of any seen actor; read from
  `PerceptionView::allies()`), "Snel: with you" in the retinue, and "Snel: away" when it is on
  another level. A text snapshot (`screen_snapshots.rs`).
- **Look mode**: A1's "(your wolf)" becomes "Snel, your hound (companion)"; a tameable wild
  beast: "a wolf (neutral)" as today, with "It has not noticed you." from Stalker's eye as today.
- **The Character screen** (page 1) gains a line under the class: "Companion: Snel, a hound".
- **The abilities menu**: A1's Allies group; Tame in the Class group with its reason when
  refused ("you already have a companion", "no beast there", "it has seen you", "too strong for
  you"); Sic 'em there from 40 ("no companion here", "no focus").
- **The one dialog** for Let go.

### 13.3 The CLI and dumps

- `dump --layers companion`: `Snel, a hound (grade 12) at (12, 5): hp 30/30, order follow`;
  `Snel, a hound (grade 12): with you` in the retinue; `no companion` without one; JSON
  `companion { name, creature, grade, hp, hp_max, order, pos, level, retinue }` (null without
  one).
- A1's `allies` layer and the `actors` additions show the companion as any ally, with `named
  Snel, grade 12` appended.
- `abilities` lists Tame `(class)` and Sic 'em `(class)`; the orders are A1's `(ally)`.
- Action text: §12.

### 13.4 The wiki

A1's `wiki/src/allies.md` gains a section "The ranger's companion" (or a page of its own,
`companions.md`, beside it: the owner's taste; `TERMS.md` gains "companion", "tame", "grade"):
the hound, how a beast is tamed (the exact check and the level gate), Let go, the growth table
at the cadence levels for every tameable beast (generated by `xtask/src/wiki/companions.rs`
from the data), and what Pack tactics, Hunter's bond, and Sic 'em do exactly. The Classes page's
ranger row links it. The creatures table marks tameable types. The spec gains "The animal
companion" in spec/08i-character.md under "Class abilities"; `docs/dev/companion.md` is the
topic file (A1's `docs/dev/allies.md` the core's).

## 14. Talents and class abilities

**No new system and no new pickable talent.** The system is A1's `allies` (#828 §10), with its
pickable Beast-bond (every ally +1 to hit, +1 Protection: a real pick for a ranger too) and the
druid's class abilities Pack leader and Lord of the wild. The companion extends that system; by
the coverage policy a sub-feature of a system adds no tag and no talent. The ranger's class
abilities carry the tag where they read the core:

| Class ability | Systems | Why it earns its place (rules 1–9) |
|---|---|---|
| Animal companion (1) | `allies` | the class's signature: a new actor on your side from turn one, and Tame, a new action (rule: it changes what the character can do; no information, no niche) |
| Pack tactics (18) | `combat`, `allies` | conditional, two-sided to-hit on every dungeon floor; no clone of Beast-bond (a different effect kind, §10.1) |
| Hunter's bond (25) | `vision`, `allies`, `hit_points` | shared sight, the whistle, the wider following, tending: four changes to what the ranger can do; rule 8 untouched (sight is a capability, not a readout) |
| Sic 'em (40) | `combat`, `resources`, `allies` | an active on Focus at the capstone tier that changes a fight (rule 1); no cooldown (Focus and time are the price) |

Nothing else is minted: no "faster taming", no "second companion", no "companion hit points"
talent (a small percentage on one class's pet is rule-bar filler; the growth table is where
those numbers live). Handle animal gets its first use (Tame) without a talent of its own; A1's
Beast-bond asks Handle animal 40, so the skill now has a use and a talent. Beast-friend stays
the Wayfarer pick and the druid's level 1: a ranger who wants calm rats takes it; the companion
respects it through the master's stance (A1 §4.3). Tags match hooks: each reads the system it
names.

## 15. Rejected ideas

| Idea | Why not |
|---|---|
| The companion at 18 (the first gap), an archer until then | the owner's "pet class" is felt from turn one, or it is not one; NetHack's and WoW's pets are there from the start |
| A choice of beast at character creation | one more creation step for one class; the hound and Tame do it, and the wild beasts are the dungeon's reward |
| Two or more companions, a pack (D2's druid) | that is player summons, the druid's design; one hound keeps the fiction and the classes apart |
| A second set of orders for the companion (fetch, guard a cell, track) | A1's four are the pack's and the hound's alike (#828 §4.4: one order for all); a per-ally order needs a picker the one-dialog rule does not want |
| Tameness, happiness, feeding (NetHack, classic WoW) | fiddly meters their own designers removed; creatures have no hunger to simulate; food is a one-time bonus on taming instead |
| The companion's own XP and levels (NetHack, ADOM) | a second curve the player never sees, and a level 1 pet at depth 10; the grade follows the ranger's level |
| Growth by swapping the type (hound → great hound at 18 and 40) | three steps over sixty levels leaves the pet weak for most of them; continuity (its name, its hit points) is lost |
| Growth for summons too | their power is the spell's ladder (#828 §5.1): a caster level that both grew the count and the grade would double-count; the grade is the companion's |
| Revive, a stable, resurrection for gold (D2, WoW) | permadeath world: tame another; a stable is a second companion in disguise |
| A grief status on the ranger when the companion dies | a flaw must bite in a dungeon (rule 5), and this one would only sting; the loss is the hound |
| The companion walks around the traps the player knows | A1 decided traps are the player's knowledge, not the wolf's (#828 §4.3); Hold here before a trapped corridor is the ranger's craft |
| The pet attacks only when the ranger spends an action (5e Beast Master) | the subclass that taught everyone not to; A1's ally acts on its own turn |
| A tameness roll on each order (disobedience) | WoW and DCSS allies obey; a disobedient pet is a worse AI, not a mechanic |
| New rebindable keys for the orders, a pets screen | the hard rule: the one abilities menu, fixed letters; the status bar line is the screen |
| Taming a hostile aware beast while it fights you (WoW's channel) | it must be neutral or unaware: Stalker's eye's purpose, and no "calmed" state to invent |
| A general "anyone trained in Handle animal tames" (ADOM, NetHack) | the owner's: the ranger is the pet class (§17 Q2 asks) |
| The companion picks things up, steals from shops (NetHack) | charming, out of scope; no inventory on the companion |
| Companion gear: a collar, armour | no slots on a beast; later, perhaps a collar slot (§17 Q9) |
| A revive-on-death capstone | a death undone is the one thing this world does not do |
| Stealth at 40 (Camouflage's twin underground) | the class's direction is the pet; Sic 'em is the fight changer a capstone tier asks for |
| Cooldowns on Tame or Sic 'em | Focus and time are the prices; no anti-farming limits (the owner) |
| A pickable talent of this document's own for `allies` | A1 ships Beast-bond; the bar is one, and a count is never a reason to add a talent |
| Mounts | out of scope; a line on #196 |

## 16. Implementation plan

A1 (#828 §12) goes first; R1 is the "R1 (#825)" row of its plan. R2–R4 are one class ability
each and may run in parallel after R1.

| Slice | Issue | What it touches | Depends on |
|---|---|---|---|
| **R1 The companion** | the ranger's Animal companion: the hound, Tame, Let go, the name, growth, the table | `animal_companion` talent and the ranger's table (level 1), `TalentEffect::Companion`, `hand_out`'s arm, `AllySource::Companion(CompanionData)` (request C), `world/companion.rs` (`bring_companion`, `World::companion`, the level-up grade, Let go in place of A1's refusal: request B), `world/tame.rs` (`ClassActive::Tame`, the channel, the check, the food, marks), the `tame` benefit on Handle animal, the six growth hooks (§8), `CreatureDef::tameable` and its load checks, `creatures/hound.toml`, `tameable` on the eight types, `World::called` for the name, `content/companion.rs` (`data/companions.toml`), the `companion` dump layer and JSON, the one dialog for Let go (the app, from the view), the status bar line, look mode's name, the Character screen's line (text snapshots), `tame:DIR`; tests (§12); `docs/dev/companion.md`; spec; the wiki section and its generated growth table; the ranger replays re-created; `sky.toml`'s forest golden updated in the finisher's pass | A1 |
| **R2 Pack tactics** | level 18 | `pack_tactics` talent, `TalentEffect::{PackTactics, SharesFavouredPrey}`, `world/pack.rs` (the flanking read in the profile), the table row, tests, wiki | R1 |
| **R3 Hunter's bond** | level 25 | `hunters_bond` talent, `TalentEffect::{SharedSight, Whistle, TendsAlly}` and `ally_mod.comes_along` (request A, else its own read), `perception.rs`'s added cells (`world/bond.rs`), the whistle line in `ally_plan`, `heal::treatable`'s ally candidate, the table row, tests (shared sight equals the companion's `sight::sees_by`; memory; dazzle; the whistle across a level; following from 6), a `--capture` looked at, wiki | R1 |
| **R4 Sic 'em** | level 40 | `sic` talent, `ClassActive::Sic`, `world/sic.rs` (the hunt that follows its target), the `savage` status (`auto_critical`, eased by hits, `speed_percent`) and its read in `strike_with`, the table row, tests, wiki | R1 |
| **R5 The boar and the lynx** | content | `creatures/{boar,lynx}.toml` (+ `docs/dev/creatures/beasts.md` rows, the roster tests), `mapgen/local/` (the glade's boar, the deep wood's lynx, as the deer), the Ascmearc's wildlife rows, wiki | R1 (`tameable`); D1's `beasts.md` |
| **R6 The probe** | the ranger-and-hound row in `balance_probe`; a companion beside the player on `bench-turns`' `delf` row (an ally in the coarse turns) | `balance_probe`, `xtask/src/bench.rs`, `docs/balance.md`, `docs/perf.md` | R1 |

R1 adds saved state (inside A1's field) and two actions: with A1 and A2 that is the integrator's
"about three at once" (#828 §12 says the same). Each PR runs the full check; R1's snapshots are
the dumps' insta files, core tests, and text snapshots; no golden is added, and R1 updates the
one the hound appears in.

## 17. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| Q1 | **The grade**: the ranger's full level (100 %, WoW, D2) or half (D&D 3.5)? | 100: the hound is the class's damage and its tank; the balance probe (R6) says if it needs trimming, and `grade_percent` is one number |
| Q2 | **Tame ranger-only**, or open to anyone trained in Handle animal (ADOM, NetHack)? | ranger-only: "our pet class"; Handle animal's `tame` benefit is where the skill matters for everyone who has it, and a druid's beasts are the druid's summons |
| Q3 | **Send away on a companion is Let go** through the one dialog (request B), or A1's refusal stands and Let go is a fifth ability? | Let go through Send away: one order, as A1 wants, and a dialog is the owner's way to ask |
| Q4 | **The starting beast**: always a hound, or by race (an elf's lynx, a dwarf's badger)? | a hound for everyone; the wild beasts give the variety, and a race table is one more thing to tune |
| Q5 | **`comes_along`** (request D): A1's 1, or 2? Hunter's bond makes it 6 either way | 2 is this document's recommendation; the owner decides in #828 |
| Q6 | **Shared sight at 25** (Hunter's bond): too strong, or exactly the fun? | yes: it is what a pet class is remembered for, and it comes at 25, when scouting rooms with a hound is the ranger's way |
| Q7 | **Friendly fire** from the ranger's own bursts and volleys hurts the companion (A1's rule, #828 Q3) | yes: honest, and the reason to order Hold here before a volley |
| Q8 | **Death**: nothing but the loss | yes: no grief status, no cooldown, no revive |
| Q9 | **Companion gear** later (a collar slot for one magic item)? | not now; a line on #196 if the owner likes it |
| Q10 | **Tame's gate** `over` 3 levels above the ranger's | 3: a level 1 ranger may tame a wolf (4), a level 4 one a warg (7) it catches unaware, a level 9 one a bear (12) |
| Q11 | **Barkskin stays** the ranger's level 6 spell beside the druid's level 3 | stays: a table may give a spell two classes have; the druid's design may move theirs |
| Q12 | **Tending through the Heal skill** at 25 (Hunter's bond), or from level 1 for anyone with an ally and the skill? | at 25, the ranger's: A1 heals allies by the druid's touch spells; a general rule would want its own issue |
| Q13 | **The wiki**: a section on A1's `allies.md`, or a page of its own? | a section: one place for allies, the companion's rows under it |

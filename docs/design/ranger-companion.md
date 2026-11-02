# Warlock — The ranger, the pet class: the animal companion

Design document for the owner's review (2026-11-02, #825). Written against `origin/main` at
f3846261 (encounters E5). Nothing here is implemented; §16 cuts it into issues. It builds on what
is there: the class tables and their actives (`docs/dev/progression.md` "Class tables",
`world/class.rs`, `world/class_ability.rs`), Focus (`world/focus.rs`, `docs/dev/resources.md`),
factions and stance (`docs/dev/creatures.md`, `World::stance`, `world/beast_friend.rs`), the
creature AI and `Mind` (`world/ai.rs`), making creatures (`travel::make_creature`), level changes
(`World::travel`, `free_cell_near`, frozen floors), displacement (`swap_places`, `docs/dev/npcs.md`),
the abilities registry (`warlock_core::ability`, `docs/dev/screens.md`), the one dialog, the kill
tally (`Quests::kills`, B6), checks (`check.rs`), skills and their benefits (`docs/dev/skills.md`),
and the RNG streams (`rng.rs`). Every number is data and hidden in the game; the wiki states it
(the owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-11-02, #825): "Ranger: let's create design issue for this too. I
think this is our pet class." The ranger becomes the pet class: an animal companion as the
class's core (how it is gained, grows, is commanded, dies, is saved; its AI), filling the table's
gaps 18, 25, and 40 (the gaps report, PR #802) and reworking the table, under the coverage policy
(`docs/dev/progression.md`, rules 1–9 of #776), with no new rebindable keys (CLAUDE.md's hard
rules), coordinated with the druid's design (#821), which designs player summons and the general
ally machinery.

**The dependency.** The druid's design (#821, a sibling document) owns *player summons* and the
*shared ally core*. This document says what the companion needs from that core (§4) so the two
documents name the same things, and what is the ranger's alone. Whichever implementation lands
first builds the core (slice C0 here, or the druid's equivalent); the other reuses it. Where the
two documents differ on the core, the owner decides in the druid's PR, and this one follows.

## Contents

1. Summary
2. Research, tied to the decisions
3. What exists, and what this builds on
4. The shared ally core: what the ranger needs from it, and what is ranger-only
5. Gaining a companion: the hound, Tame, Let go, the name
6. Orders, and the companion's AI
7. Following: across levels, on the overworld, left behind
8. Growth: the companion's level is the ranger's
9. Fighting together: kills and XP, Favoured prey, friendly fire, death and replacement
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
| **One companion** | the ranger has one beast at a time, from the first turn: a **hound** comes with the level 1 class ability **Animal companion** (placed beside the player as the run begins, by the class grant's `hand_out`). It is an ordinary actor of a `tameable` creature type with faction `player` and `Actor::leader = player`; `WorldState::companion` names it. §5 |
| **Tame** (the ability, part of Animal companion) | `tame:DIR`: an adjacent `tameable` beast of level at most the ranger's + 3 that is **not hostile to you or has not noticed you** (`World::unaware`; Stalker's eye at 2 shows which): ten turns beside it (interrupted when the ranger is hurt or moves), then one check (`check::check`): Willpower modifier + Handle animal's new `tame` benefit (+ 4 with a food stack offered, one eaten from the pack) against 13 + 2 × (its level above yours, if any). Made: it is yours, named. Failed: it is roused ("The wolf snaps at you."), and nothing stops you trying again (time is the penalty). Refused with a companion already: **Let go** first (`order:free`, the one dialog). §5 |
| **Orders** | in the abilities menu, a new **Allies** group (`Source::Ally`, between Class and Talents; shared with the druid's summons; fixed menu letters, no `KeyCommand`): **Heel** (follow me; the default), **Stay** (hold this cell; fight what comes beside you), **At** (a cell, in the aiming mode: go there, attack what is there, then stay), **Let go**. An order takes no time and makes a whistle's noise (`Act::Whistle`, loudness 90). §6 |
| **The AI** (`world/ally.rs`, shared) | an actor with a `leader`: its stance toward anyone is **its leader's** (`World::stance(leader, x)`: Beast-friend, outraged shopkeepers, the fey courts all follow), so it never bites the shopkeeper nor a Beast-friend's neutral rats. Heel: fight a hostile it sees within `engage` 7 cells of the leader, else keep within `heel_near` 2 cells of them, else wait. Stay/At: fight only what comes adjacent. It walks around the traps the player knows of, through doors the player leaves open (a hound opens none), swaps places with the player as a neutral does (`budge = "always"`). §6 |
| **Following** | when the player changes level (stairs, an edge, the quick leave, an encounter's area), every ally with `leader = player` within `follow_reach` 2 cells and not under Stay comes along, placed by `free_cell_near` the arrival cell. On the overworld it is an actor beside the player as on any level (nothing hostile lives there; it only follows). Left behind, it freezes with its level and waits; the ranger goes back for it. §7 |
| **Growth** (`Actor::grade`, shared) | the companion's level **is the ranger's class level** (`grade_percent` 100, re-set at every level-up: "Snel grows stronger."). Five hooks read a `grade` above the type's level with `data/companions.toml [growth]`: hit points +2 a level, +1 to hit per 4, +1 damage per 6 and one more damage die per 12, +1 Defence per 6 and natural Protection by the body formula at its grade (`docs/design/combat-progression.md` §8), St/Dx/To +1 per 5. A summon takes a fixed grade at making; the companion's follows the ranger. §8 |
| **Kills** | a companion's kill pays the ranger as their own: XP (`kill_xp_percent` 100, the scaling and the first-kill bonus as for a kill of theirs) and the kill tally, so **Favoured prey learns from the hunt** together. Not Scavenger, not a resource's `Build::Kill` (those are the hands that struck). §9 |
| **Friendly fire** | honest: the ranger's bursts, bolts, and a Volley's missiles hurt the companion as they hurt anything in the way ("Snel yelps."); walking into it swaps places, never strikes. §9 |
| **Death** | "Snel dies." The bond ends, a corpse lies there, nothing else happens to the ranger (no grief status: a flaw that bites nothing but morale is none); the ranger tames another when they find one. No revive, no stable, no cooldown. §9 |
| **The table** | 1 **Animal companion** (new), Favoured prey, Trained wind; 2 Stalker's eye; 6 Tracker, Barkskin; 12 Rapid shot; **18 Pack tactics** (new: a creature beside both of you is flanked, +2 to hit for both; the companion shares your Favoured prey); **25 Beast bond** (new: you see what your companion sees; Heel is a whistle it answers from anywhere on the level; it follows you across levels from 6 cells); 32 Evasion; **40 Sic 'em** (new, an active on 5 Focus, aimed at a creature you see: the companion goes for it at half again its speed and its next three blows on it are criticals); 50 Volley; 60 Master hunter (its Favoured prey half reaches the companion through Pack tactics). Nothing goes; nothing else changes. §10 |
| **Content** | the `hound` (the starting beast, `stray = false`), and the first wild beasts to tame: `wolf`, `boar`, `lynx` in the forest local areas and the Ascmearc's wildlife encounters, `bear` in the hills; `tameable = true` on `rat`, `bat`, `deer`, `beetle`, `warg`, `dire_warg`. §11 |
| **Talents** | one new system, **`allies`** (companions and player summons; whichever of this and the druid's PR lands first adds the tag), with one pickable talent that earns its place for every pet build: **Pack leader** (Wayfarer): your allies strike +1 to hit, have +1 Defence, and follow you across levels from 2 cells farther. Beast-friend stays a Wayfarer pick (and the druid's level 1), not the ranger's. §14 |
| **Slices** | C0 the shared ally core (or the druid's), C1 the companion (Animal companion, Tame, Let go, the hound, the data, the dumps, the table's level 1, the wiki), C2 Pack tactics, C3 Beast bond, C4 Sic 'em, C5 the wild beasts, C6 the UI (status bar line, look mode, text snapshots). §16 |

**Key choices, and why**

1. *The companion is the level 1 signature, not the level 18 filler.* "Our pet class" means the class is felt from the first turn as a hunter and a hound; a companion that arrives at 18 would make the ranger an archer for seventeen levels and then something else. NetHack's role pet and WoW's level 10 Tame Beast are the model: the pet is there from the start, and the later abilities make it better.
2. *One beast, grown, not a pack.* Diablo 2's mercenary and WoW's pet level with their master and are the player's one companion; the D2 druid's wolves and necromancer's skeletons are *summons*, which are the druid's design. One companion keeps the AI's cost at one actor, the fiction clean (a ranger and a hound), and the two classes distinct.
3. *The companion's level is the ranger's.* A pet that stays level 1 (ADOM's, until it kills enough on its own) is dead weight by the fourth floor; a second experience curve (NetHack's) is bookkeeping the player never sees. One number, read live, and a growth table in data (§8), as a hireling in D2 "levels with you".
4. *Tame reuses surprise.* WoW tames a hostile beast that fights back during the channel; here the beast must be neutral or unaware (`World::unaware`, the one definition, surprise design), so Stalker's eye at level 2 (you see who has not noticed you) is the tamer's tool, and no new "calmed" state is needed.
5. *Orders are abilities, and stance is the leader's.* DCSS's ally orders (attack, follow, stay, retreat) are the four anyone wants; here they live in the one abilities menu (hard rule: no new rebindable keys) and take no time. The ally's stance toward everyone is its leader's, one read, so every existing rule about who fights whom (Beast-friend, outraged shopkeepers, the fey's `provoked`) holds for the companion with no second table.
6. *The stairs rule is ADOM's and NetHack's, widened.* Adjacency is the classic and the classic annoyance; two cells (data) is the compromise, six with Beast bond, two more with Pack leader; Stay means stay. Everything else about moving between levels is `World::travel` as it is.
7. *No tameness, no feeding, no happiness.* NetHack's tameness decay and classic WoW's pet happiness were removed by their own designers or hated; a hunger the creatures do not have anyway is not simulated for the companion. The companion is loyal; the cost of having it is keeping it alive.

## 2. Research, tied to the decisions

| Source | What it does | Taken | Left, and why |
|---|---|---|---|
| ADOM (pets by Handle Animal and food, the puppy; pets follow when adjacent on stairs; pets level by their own kills; pets eat corpses) | taming by giving food, companions that walk the wilderness map with you, the adjacency rule | food as a +4 on the tame check (§5), companions on the overworld (§7), the stairs rule widened to 2 cells (§7) | a pet's own XP (a level 1 dog at dungeon level 10); feeding (not simulated for creatures) |
| NetHack (the role's starting pet; tameness kept by feeding; follow when adjacent; the magic whistle; pets steal from shops; pets refuse to attack much stronger foes) | the starting pet from turn one; the whistle that brings the pet to you | the hound at level 1 (§5); Beast bond's whistle at 25 (§10) | tameness decay; shoplifting pets; a pet's fear of strong foes (the companion fights what its leader fights; its grade keeps it able) |
| DCSS (no pets by design; allies from summons, necromancy, and Beogh's orcs; `t` orders: attack, follow, stay, retreat, which take no time; allies within reach follow on stairs; allies' kills give the player XP, Beogh's orcs level by kills in sight) | the four orders as a menu, orders free of time, ally kills paying XP, allies following from a short distance | Heel, Stay, At, Let go (§6), no time (§6), `kill_xp_percent` 100 (§9), `follow_reach` 2 (§7) | the retreat order (Heel does it: it comes to you), many allies at once (the druid's) |
| Diablo 2 (hirelings: one at a time, level with you, equipped, resurrected for gold; the druid's summons: wolves, a grizzly, one kind at a time) | one companion that levels with its master, different beasts for different builds | the grade following the ranger's level (§8), beasts with different bases to tame (§11) | resurrection for gold (permadeath world: tame another); equipping the companion (no slots on a hound; later, perhaps a collar: §17 Q11) |
| WoW hunter (Tame Beast: a channel on a beast at or below your level; one active pet, a stable; pet families with roles; the pet's level follows the hunter's; stances passive/defensive/aggressive; Mend Pet, Revive Pet; Kill Command, Bestial Wrath, Eyes of the Beast; happiness and feeding, later removed) | the level gate on taming, the pet's level bound to the master's, an unleash active, seeing through the beast | `over` 3 levels (§5), `grade_percent` 100 (§8), Sic 'em (§10, Bestial Wrath and Kill Command in one), Beast bond's shared sight (§10, Eyes of the Beast) | the stable (one beast; let go and tame anew), Revive Pet, happiness, pet talents (the companion's growth is the ranger's table) |
| D&D 3.5 ranger (animal companion at level 4, half the druid's progression, tricks by Handle Animal) and 5e Beast Master (the pet attacks only when the ranger spends their action; the subclass widely judged the weakest) | Handle Animal as the skill | the `tame` benefit on Handle animal (§5), which gets its first use | half progression (the data allows it, §17 Q1; the default is full), a pet that costs the master's action (the companion acts on its own turn, always) |

## 3. What exists, and what this builds on

| Part | Today | Used for |
|---|---|---|
| The class tables | `ClassDef::abilities` (level, talent id); `class::grant` gives level 1's at `start_run`, each level's at `level_up`, and on `from_state`; a given talent's effects through `hand_out` (hit points, stat gifts, `grants_resource`) | Animal companion at 1 brings the hound through `hand_out` (one more arm: `companion`); Pack tactics, Beast bond, Sic 'em at 18, 25, 40 |
| Class actives | `ClassActive` (`content/class.rs`), `use_class` / `use_class_at` (`world/class_ability.rs`, `world/volley.rs`), `Ability::Class`, costs on a resource (`TalentDef::resource`) | Tame (`ClassActive::Tame`, `Aim::Direction`), Sic 'em (`ClassActive::Sic`, `Aim::Cell`, 5 Focus) |
| Focus | granted by Rapid shot (12), max 10, regenerates 1 a base turn, not while resting; Volley spends 6 | Sic 'em's cost |
| Factions, stance | `Content::stance_of(faction, faction)` (symmetric hostility; a faction and itself neutral), `World::stance(a, b)` with Beast-friend's `befriended`, `Actor::faction(&content)` with a per-actor override (`make_creature(.., faction)`, as a summon takes its summoner's) | the companion is made with faction `player`: every monster hostile to the player is hostile to it and it to them; shopkeepers are neutral to both |
| Making a creature | `travel::make_creature` (the one way; `CreatureSpawn`, gear rolled or not, uniques remembered, `World::spawned`) | the hound at the start, a tamed beast re-made as an ally (same actor, fields set) |
| The AI | `ai.rs`: look → fight → the last seen spot → the newest noise → back to the post → patrol; `nearest_hostile_in_sight` by `stance_of`; `Mind { last_seen, noise, post, noticed, missed }`; `path.rs` A* with fixed tie-breaks | `world/ally.rs` is one branch before the look: an actor with a `leader` plans by its order (§6), then falls into `fight` with the target it chose |
| Displacement | `swap_places`: moving into a non-hostile swaps; `budge` (`always`, `off_post`, `never`); sizes | walking into the companion swaps (a hound is `always`) |
| Level changes | `World::travel(to, at)`: `visit`, the arrival cell, `free_cell_near`, the player's `level` and `pos`, `arrive_spawner` (floors the player is not on stay frozen), `settle_in`; `leave_by_edge`, the quick leave, encounters' `open_area` | the followers' move is one call after the player's (§7) |
| The one dialog | `Dialog::new(title, text).plain(label, Action)` from the view | Let go's confirmation |
| The abilities menu | `Ability::every` in menu order by `Source`; `PerceptionView::abilities` (`OwnAbility`, `unavailable` reasons) | `Source::Ally` and `Ability::Order(Order)` |
| Checks | `check::check(rng, modifier, difficulty)`, 2d12, two gameplay draws | the tame check |
| Skills | Handle animal exists (`kind = "action"`, trained by the ranger 2d4+4, the barbarian, druid, paladin, witch, aristocrat, commoner, merchant), with no use yet; benefits are read in one place each (`docs/dev/skills.md`); `Ability::use_of` lists a skill's use in its place | the `tame` benefit (Handle animal), read in `world/tame.rs`; marks on `tame` |
| Unaware, surprise | `World::unaware(a, b)` (the one definition), `Mind::noticed`, Stalker's eye's `sees_unaware` | who may be tamed |
| The kill tally and XP | `World::dies(id, by)`: `gain_resource(by, Kill)`, `creature_dies(id, by_player)` (XP with the scaling and the first-kill bonus, the tally `Quests::kills`, Scavenger, the curse) | the companion's kill paying the ranger (§9) |
| Natural Protection by body and level | the data rule (`combat-progression.md` §8; `content/creature_body_tests.rs::Body::of`): hide (L − 10) ÷ 8, bony L ÷ 6, armoured L ÷ 4 + 1 | the companion's Protection at its grade |
| Perception | the snapshot built from the player's eyes (`perception.rs`, `sight.rs`); creature sight computed per creature with the same rules | Beast bond adds the companion's seen cells (§10) |
| Statuses that ease by actions | `docs/dev/weapon-abilities.md`: a status spent a degree per action of a kind | `savage` (Sic 'em) spent a degree per hit |
| Coarse turns | `stride.rs`: idle creatures stride while the player rests | an ally beside its resting leader is idle and strides nowhere (its route is empty); one actor, no budget risk |
| The local areas' deer, the encounters' wildlife entries | `deer` (faction `wildlife`, `animal = true`), placed by the local generators and the Ascmearc's `"wildlife"` entries ("Hunt them" / "Let them be") | the first beasts to tame: wolves and boars join the deer |

## 4. The shared ally core: what the ranger needs from it, and what is ranger-only

The druid's design (#821) owns this core. This section is the ranger's order from it, so both
documents name the same fields and modules.

### 4.1 Needed from the core (shared with player summons)

| Piece | What the companion needs | Notes for the druid's design |
|---|---|---|
| `Actor::leader: Option<ActorId>` | whose side it is on and whom it follows; saved and hashed, left out while `None` (old saves load; no `FORMAT_VERSION` bump) | a summon's leader is its caster; `Actor::summoner` (enemy summons, `docs/dev/summoning.md`) may stay as it is or fold into this: the druid's call |
| **Stance by the leader** | `World::stance(ally, x)` and the AI's target choice answer with `World::stance(leader, x)` for an actor with a leader | one definition; makes Beast-friend, `provoked`, outraged shopkeepers hold for allies |
| **Faction `player`** on allies | monsters' `nearest_hostile_in_sight` sees the ally as a foe by `stance_of` alone; no AI change on their side | a summon the same |
| `Actor::order: Option<Order>` | `Heel` (`None`), `Stay(pos)`, `At(pos)`; saved and hashed | summons take the same orders (to all of them at once, or by proximity: the druid's call) |
| **`world/ally.rs`** | the plan by order (§6.2), one branch at the top of `ai.rs`'s plan for an actor with a leader; `route` avoiding the leader's known traps | the same module serves summons; a summon with a duration adds its own ending elsewhere |
| **Following across levels** | `World::travel` → `bring_followers(from, to, arrival)`: every actor with `leader = player` on `from` within `follow_reach` (Chebyshev) of the player's cell before the move and not under `Stay` moves to `free_cell_near(to, arrival)` in id order | summons follow by the same rule |
| **Orders as abilities** | `Source::Ally` (the menu's Allies group, between Class and Talents), `Ability::Order(Order)`: Heel and Stay `Aim::Nothing`, At `Aim::Cell`, Let go `Aim::Nothing` (with the dialog); shown only while the player has an ally; `order:heel`, `order:stay`, `order:at:X,Y`, `order:free` | the druid's "dismiss" is Let go |
| **An order's cost** | `Spent::NoTime`, one `make_noise(Act::Whistle)` at the player (loudness `whistle` 90, `data/companions.toml [orders]`) | the same |
| `Actor::grade: Option<u8>` and the growth hooks | an actor's own level when it differs from its type's; read by `Actor::max_hp`, the attack profile's to-hit and damage, `Actor::defence`, `Actor::protection`, and the stat reads, with `[growth]` (§8) | a summon's grade is the caster level at making, fixed; the companion's is re-set at the ranger's level-up |
| **Kill credit** | `World::dies(id, by)`: when `by` is an ally of the player, `creature_dies(id, true)` pays the player (XP at `kill_xp_percent`, the tally, the first kill); `gain_resource(by, Kill)` and Scavenger stay the striker's | the same for a summon's kill (the druid may set its own percent) |
| **Perception** | `SeenActor::ally: bool` (look mode "(your companion)" / "(your ally)"); the ally's `health` words as any seen actor's | the same |
| **Dumps** | the dev `actors` layer adds `; leader #N, order heel, grade 12`; a new `allies` layer (every ally of the player: name, type, grade, hit points, order, position) | the same |
| **The `allies` system** | `[[system]] id = "allies"` in `data/talents.toml` (by topic beside `factions`), with one talent, **Pack leader** (§14) | whichever PR lands first adds both; the druid's doc may name its own talent for the tag, and then Pack leader is this document's second, judged on its own |

### 4.2 Ranger-only

| Piece | Where |
|---|---|
| `WorldState::companion: Option<ActorId>` (the bond: at most one), `Actor::given_name: Option<String>` (read by `World::called`) | `world/companion.rs` |
| Animal companion's grant (the hound through `hand_out`), **Tame** (`world/tame.rs`: the channel, the check, the food), **Let go** | `world/companion.rs`, `world/tame.rs` |
| The grade that follows the ranger's level (`level_up` hook) | `world/companion.rs` |
| Pack tactics, Beast bond (shared sight, the whistle, the reach), Sic 'em and `savage` | `world/pack.rs`, `world/bond.rs`, `world/sic.rs` |
| The hound, the wild beasts, `tameable`, `data/companions.toml` | content |
| The status bar's companion line, the `companion` dump layer and JSON | UI, `dump.rs` |

## 5. Gaining a companion: the hound, Tame, Let go, the name

### 5.1 The hound at level 1

The class ability **Animal companion** (`animal_companion`, tree `class`, `granted = true`,
effects `[{ kind = "companion", creature = "hound" }, { kind = "class_active", active = "tame" }]`)
is given at `start_run` as every level 1 entry is. Its `companion` effect is one more arm of
`class::hand_out`: `World::bring_companion(creature)` makes an actor of that type beside the
player (`make_creature` with faction `player`, `leader = player`, `grade` = the player's level, no
gear; `free_cell_near` the player), names it (§5.4), and sets `WorldState::companion`. On
`from_state` nothing is made (a saved run has its own, or lost it); a talent whose `companion`
arm finds `WorldState::companion` already set does nothing. So every ranger run has a hound from
the first turn, the scenarios' rangers too (the showcase's Osric on the overworld and Maren in the
forest glade; their replays and the goldens that show them, `overworld.toml`'s and `sky.toml`'s
forest entries, change once, deliberately: §12).

### 5.2 Tame

`Ability::Class(ClassActive::Tame)`, "Tame" in the menu's Class group, `Aim::Direction`,
`tame:DIR`. The one place is `world/tame.rs`.

| Step | Rule |
|---|---|
| Who | an actor adjacent in that direction whose type is `tameable = true` and whose level is at most the ranger's class level + `over` (3); the ranger has no companion (`WorldState::companion` is `None`, else "You already have a companion." — Let go first); the beast is **not hostile to you** (`World::stance(player, it)` neutral: wildlife, or an animal a Beast-friend calmed) **or has not noticed you** (`World::unaware(it, player)`). Else refused, no time: "The warg has seen you: it will not be tamed now." |
| The channel | `turns` (10) actions of the ranger beside it, as a dig's turn loop (`dig_turn`): each a normal action's price; interrupted when the ranger is hurt, moves, or the beast moves away, with nothing lost but the time. The beast is left alone by the AI meanwhile only in that it is neutral or unaware; a wandering beast ends the channel |
| Food | if the ranger carries food, one item of the first food stack is eaten by the beast (`+food` 4 on the check; "You hold out some dried meat."); none, no bonus |
| The check | `check::check(gameplay, Willpower modifier + Handle animal's `tame` benefit + food, 13 + 2 × max(0, its level − the ranger's level))`. Made: it is yours (§5.3). Failed: "The boar snaps at you and backs away." — its `Mind::noticed` is set (it knows you now; a neutral one stays neutral, an unaware hostile is aware), and nothing more: you may try again when it is calm or unaware again (no cooldown: time is the penalty). A natural 2 fails and a 24 succeeds, as every check |
| Marks | Handle animal marks on `tame` (`mark = [{ on = "tame", chance = 30 }]`), made or failed |

A beast tamed keeps its actor (hit points, statuses, its place), changes faction to `player`,
takes `leader = player`, `grade` = the ranger's level (its hit points rise with its maximum), the
name, and `Mind` cleared. "The wolf comes to your hand. You call it Græg."

### 5.3 Let go

`Ability::Order(Order::Free)`, "Let go", in the Allies group; the one dialog, "Let Græg go?" /
"It will not come back." / "Let it go" (`order:free`) and "Keep it" (nothing). The companion's
faction returns to its type's, `leader` and `given_name` and `grade` are cleared (its hit points
capped at the type's maximum), `WorldState::companion` is `None`; it wanders off as a neutral
creature of its kind ("Græg looks back once, and goes."). A `wildlife` wolf is neutral; a warg's
type is `orcs`, hostile: a let-go warg is a warg again, which is the player's own doing (the
dialog's text says "It will be a wild thing again.").

### 5.4 The name

Drawn from `data/companions.toml [names]` (Old English, the world's register for old things:
Snel, Brūn, Græg, Blæc, Hwīt, Lēof, Hræfn, Wulf, Fæger, Beald, Hraþe, Fugol; the owner's #378
review may rename) on the stream `companion/<actor id>` (one draw), so no other roll moves.
`World::called` says the name without an article, as a unique character's (`docs/dev/npcs.md`):
"Snel bites the kobold." A dead companion's corpse is "a hound corpse" (the type's), not the name.

## 6. Orders, and the companion's AI

### 6.1 The orders

| Order | Menu | Aim | Action text | What it does |
|---|---|---|---|---|
| Heel | "Heel" | nothing | `order:heel` | `Actor::order = None`: follow me (the default) |
| Stay | "Stay" | nothing | `order:stay` | `Stay(its cell)`: hold there; fight only what comes adjacent; does not follow across levels |
| At | "Go there" | a cell (the aiming mode) | `order:at:X,Y` | `At(cell)`: go there by the shortest route; attack what stands there or any hostile met adjacent on the way; arrived, it becomes `Stay(cell)` |
| Let go | "Let go" | nothing (the dialog) | `order:free` | §5.3 |

An order is `Spent::NoTime` with one `make_noise(Act::Whistle, player's cell)` at loudness
`whistle` 90 (data), so a whistle in a quiet corridor is heard as a kick is (`docs/dev/hearing.md`:
the `sound` layer shows it as `w`). Beast bond (25) makes Heel a whistle the companion answers
from anywhere on its level (§10.3).

### 6.2 The plan (`world/ally.rs`, one branch before `ai.rs`'s look)

For an actor with `leader` on the leader's level, in order:

1. **A foe to fight.** The nearest hostile *by the leader's stance* it sees (`sight::sees` with its own eyes), within `engage` 7 cells of the leader under Heel, or adjacent to itself under Stay or At (and on the way under At): the usual `fight` plan with that target (approach, attack, the creature's own abilities if its type has any: a warg's bite's bleeding, a bat's nothing). A fight it is in continues until the foe is dead or out of sight (then `Mind::last_seen` as any creature's, but it gives up the search at `engage` from the leader and comes back).
2. **At:** not yet at the cell: one route step toward it (A*, `path.rs`; the leader's known traps cost `TRAP` like a shunned cell, so it walks around them; an unknown trap it springs as anyone); at the cell: `Stay(cell)`.
3. **Stay:** nothing to fight: wait.
4. **Heel:** farther than `heel_near` 2 from the leader: one route step toward a free cell adjacent to the leader (toward the leader when none is free); within it: wait. A closed door in the way it cannot open (`opens_doors = false` on a hound): it waits before the door. The player opening the door frees it.
5. On another level than its leader: an ordinary creature's plan with `Stay(its cell)` (it holds where it was left; the level is frozen anyway).

It never attacks its leader, nor anyone its leader is not hostile to. It makes step noise as any
creature (footsteps 40 for the hound: a hostile may hear it coming, which is the price of a dog).
Its `Mind::noticed` and `missed` work as any creature's: it can be surprised. Coarse turns
(`stride.rs`): an ally beside a resting leader has no route and no foe, so it is `idle` and
takes the cheap turn as any calm creature; one actor, no budget change (`bench-turns` gets a
companion on the `delf` row to prove it).

The druid's summons share this plan; what differs (a duration, a summon's own spells) is theirs.

## 7. Following: across levels, on the overworld, left behind

- **Across levels.** `World::travel(to, at)` calls `bring_followers(from, player's cell before
  the move, to, arrival)` after the player is placed: every actor with `leader = player` on
  `from` within `follow_reach` (2; 6 with Beast bond; +2 with Pack leader) of that cell and not
  under `Stay` is moved to `free_cell_near(to, arrival)` (in actor id order), its `energy` set
  as the player's, its `Mind` cleared. This covers stairs, `leave_by_edge`, the quick leave, an
  encounter's area (the party's positions are the encounter's, the companion is placed beside the
  player after them), and a collapse. "Snel follows you down." / "Snel stays behind." (when it
  was too far or told to stay; said only when the player had a companion on the level left).
- **On the overworld** it is an actor beside the player as on any level, under Heel: an overworld
  step is long (20 min on a road, 150 in a swamp) and the scheduler gives it that time, so it
  keeps up by walking. Nothing hostile lives on the overworld (danger 0), so it only follows.
  Resting or foraging there (encounters E4), it is with you, and comes into the encounter's area.
- **Left behind**, it is a creature on a frozen level with `Stay` at its cell: nothing happens to
  it until the player returns, and then Heel (or Beast bond's whistle) gathers it. A companion
  on a level the player can never return to is lost: tame another.
- **In the water, on ice, in a pit**: the creature rules (`World::tread`, `wade`, the traps) as
  for anyone; a hound swims as its type says.

## 8. Growth: the companion's level is the ranger's

`Actor::grade: Option<u8>` is an actor's own level when it differs from its type's. The
companion's is `grade_percent` (100) of the ranger's class level, at least the type's level, set
when it is gained and at every `level_up` (one line in `World::level_up`: "Snel grows stronger."
when it rises). A summon's is set at making and fixed.

The hooks, each reading `grade − type level` (never below 0) with `data/companions.toml
[growth]`, one read each:

| Hook | Where | Rule (the defaults) |
|---|---|---|
| hit points | `Actor::max_hp` | `base_hp + hp_per_level × above` (`hp_per_level` 2); the current hit points rise with the maximum |
| to hit | the attack profile (`Actor::stanced`) | `+ above ÷ to_hit_every` (4) |
| damage | the attack profile | `+ above ÷ damage_bonus_every` (6) flat, and `+ above ÷ damage_dice_every` (12) more dice of the attack's die |
| Defence | `Actor::defence` | `+ above ÷ defence_every` (6) |
| Protection | `Actor::protection` | the body formula at the grade (hide: (grade − 10) ÷ 8, never below the type's own) |
| stats | `Actor::stats` reads for checks and burden | St, Dx, To `+ above ÷ stat_every` (5) |

Speed, senses, size, and resistances are the type's. So the hound (level 1, 1d6 bite, 6 hit
points, Defence 1 by its hide) beside a level 30 ranger is a level 30 hound: 64 hit points, +7 to
hit, 3d6+4, Defence 5, Protection 2. The wiki's companions page prints this table for every
tameable beast at the cadence levels, and `cargo test balance_probe` gains a row: the ranger and
their hound against the monsters of their level (`docs/balance.md`).

Nothing of the companion's is an XP curve: no kills counted for it, no level of its own to show.

## 9. Fighting together: kills and XP, Favoured prey, friendly fire, death and replacement

| Topic | Rule |
|---|---|
| **A companion's kill** | `World::dies(id, by)`: `by` an ally of the player → `creature_dies(id, by_player = true)`: XP as a kill of the player's (`kill_xp_percent` 100 of what the player's own kill would pay, the level scaling and the first-kill bonus included), the kill tally (`Quests::kills`: **Favoured prey learns from the hunt together**; kill quests count it: a hound's goblin is your goblin), the curse of a cursed corpse. Not Scavenger (the player's hands know the drop), not `gain_resource(by, Kill)` for the player (Rage builds on your own kills; the companion has no pools) |
| **Pack tactics (18)** | §10.2 |
| **Friendly fire** | honest and unchanged: a burst or bolt the ranger casts or reads hurts the companion in its radius or line as it hurts anything; a shot or thrown thing strikes the first creature in its path, the companion too; a Volley's three shots the same. "Snel yelps." (its `health` words after). Walking into it swaps places (never a blow); `fire:X,Y` at its cell is allowed (ADOM allows it), with no dialog |
| **The companion struck** | as any creature: hit points, statuses (bleeding, poisoned), Surprised; it regenerates by its Toughness as creatures do, and rests when the ranger rests. The Heal ability keeps treating the player alone until **Beast bond** (25, §10.2) lets it tend the companion: one talent, not a general rule |
| **Death** | "Snel dies." `WorldState::companion = None`; its corpse (the type's, edible as the type says) and nothing else: no grief status (a flaw that bites nowhere but in morale is none, rule 5), no XP lost, no cooldown. The ranger tames another when they find one: the forest glades, the hills, the dungeon's vermin (a rat companion is a rat companion). Time is the penalty |
| **The companion and traps** | it walks around the traps its leader knows; springs unknown ones; a trap it springs the player sees is known then (as a creature's springing is today) |
| **The companion and the encounter's choices** | none: the companion is with the player in the area; "Lie in wait" places it beside them (`Mind::missed` is set on the party, not on it) |
| **Stealth** | the companion's footsteps (40) are heard as any creature's; Stay lets a ranger scout alone and whistle it up (Heel) when the fight starts |

## 10. The reworked table: what stays, changes, goes; the three new abilities

| Lv | Entry | Status | Notes |
|---:|---|---|---|
| 1 | **Animal companion** | **new** | the hound, Tame; §5 |
| 1 | Favoured prey | stays | the companion's kills feed its tally (§9) |
| 1 | Trained wind | stays | the martial classes' stamina |
| 2 | Stalker's eye | stays | now also the tamer's tool: who has not noticed you may be tamed |
| 6 | Tracker | stays | |
| 6 | Barkskin | stays | the ranger's one spell; the druid's too (a table may give a pickable or a spell both have) |
| 12 | Rapid shot | stays | grants Focus, which Sic 'em spends |
| 18 | **Pack tactics** | **new** | §10.1 |
| 25 | **Beast bond** | **new** | §10.2 |
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
changes every fight on a dungeon floor from 18 on; it reads combat, which has its talents.

### 10.2 Beast bond (25, passive)

`beast_bond`, effects `[{ kind = "shared_sight" }, { kind = "whistle" }, { kind = "follow_reach", value = 4 }, { kind = "tends_ally" }]`,
systems `vision`, `allies`, `hit_points`. Four things the bond does, each one hook:

| Effect | Hook | What |
|---|---|---|
| `shared_sight` | `perception.rs`: after the player's field of view, the companion's cells (`sight::sees` with its eyes and `sight`, the light as it perceives it) are added as **seen**; what stands on them is seen; memory remembers them | you see what your companion sees: send it ahead with "Go there" and look round the corner. Light the companion sees by is the light that matters there (its eyes: a hound's low-light, a bat's darkvision). Hidden traps and secret doors are not found by it (the search rules are the player's; the companion's eyes are eyes, not a search). A dazzle or blindness of the player leaves the companion's view |
| `whistle` | `world/ally.rs`: a Heel order sets the companion's goal to the player wherever it is on the level (it needs no path at once; it searches as a creature does toward `Mind::noise` at the player's cell) | NetHack's magic whistle, without the item: the hound you left at the stairs comes when called |
| `follow_reach` | `bring_followers`: `follow_reach` + 4 (6 cells) | it need not be at your heel when you take the stairs |
| `tends_ally` | the Heal ability (`heal:ID`) gains the companion adjacent as a target when it has a treatable status and the player has none of that kind; the player's Heal skill rolls, the companion is treated (`world/heal.rs`, one more candidate in `treatable`) | WoW's Mend Pet, through the skill that exists |

Shared sight is the capability players remember of a pet class (DCSS's players asked for it
with Beogh; WoW's Eyes of the Beast). It is a change to what the character *can do* (scout with
the hound), not a readout, so rule 8 is not touched; it reads `vision`, which has its talents.

### 10.3 Sic 'em (40, active)

`sic`, `ClassActive::Sic`, `Aim::Cell` (a creature you see, within `range` 10), `use:sic:X,Y`,
`resource = { id = "focus", cost = 5 }`, effects `[{ kind = "sic", blows = 3, speed_percent = 50 }]`,
systems `combat`, `resources`, `allies`. Refused with no companion on the level, a cell with no
creature the player sees, or the target not hostile to the player; the cost is spent after the
checks. It does two things:

1. the companion takes `At(target's cell)` with the target as its chosen foe (it follows the
   creature, not the cell, while it sees it);
2. the companion gets the status **Savage** (`data/status.toml`: `savage`, degrees 3, no tick,
   eased by hits: each blow it lands spends a degree; eases 60 s a degree otherwise; +50 % speed
   while it lasts): each of its next three blows that hit is a **critical without the
   confirmation roll** (`combat::strike_with`: the status's `auto_critical`, read where the
   threat is rolled).

The cost is 5 of 10 Focus (Volley 6, Rapid shot 2 a shot), so a ranger at 40 chooses: a volley,
or the hound's throat-hold, not both in one turn. It is WoW's Bestial Wrath and Kill Command in
one, and it is a fight changer at a capstone tier (rule 1: never economy, quests, or XP).

## 11. The data

### 11.1 `data/companions.toml` (shared, one file; parsed by `content/companion.rs`)

```toml
# The animal companion (spec §8 "Class abilities": the ranger) and the allies' rules.
[follow]
reach = 2          # cells from the player when they change level, within which an ally comes along
heel_near = 2      # under Heel it keeps within this many cells of its leader
engage = 7         # under Heel it fights a foe it sees within this many cells of its leader

[orders]
whistle = 90       # the loudness of an order at the player's cell (a sound loses 10 a cell)

[tame]
turns = 10         # actions beside the beast before the check
over = 3           # a beast's level may exceed the ranger's by this much
difficulty = 13
per_level_above = 2
food = 4           # the bonus for food offered (one item of the first food stack)

[growth]
grade_percent = 100
hp_per_level = 2
to_hit_every = 4
damage_bonus_every = 6
damage_dice_every = 12
defence_every = 6
stat_every = 5

[kills]
xp_percent = 100   # of what the player's own kill would pay

names = ["Snel", "Brūn", "Græg", "Blæc", "Hwīt", "Lēof", "Hræfn", "Wulf", "Fæger", "Beald", "Hraþe", "Fugol"]
```

Checks at load: every number in range (reach 1 to 10, turns 1 to 100, percents 1 to 300, the
`every`s 1 to 60), at least one name, the `savage` status present with degrees ≥ 3 and no tick,
the `whistle` within the hearing bounds.

### 11.2 Creature types

| Field | What |
|---|---|
| `tameable = true` (new, `CreatureDef::tameable`, default false) | the type may be tamed; refused on a type with a `faction` that is `player`, a unique, an animated object, a type with `summon`, or one that `opens_doors` (a beast) |
| `hound` (new, shared; level 1, xp 15 by the guideline, `d`, speed 1200, faction `wildlife`, `stray = false`, hearing 20, footsteps 40, low-light vision, 6 hit points, 1d6 bite, hide; `tameable`) | the starting beast; in no population |
| `wolf` (3, forests), `boar` (3, forests and hills; a 1d8 gore that may knock down), `lynx` (5, forests; quiet, footsteps 20), `bear` (8, hills; 2d6 claws, 30 hit points) (new, shared, `wildlife`, `tameable`) | the first wild beasts: placed by the forest and hill local generators as the deer are, and as `"wildlife"` entries in the Ascmearc's encounter tables (a pack of 2d3 wolves at night in the dense forest; a boar by day; a bear in the hills) |
| `tameable` on `rat`, `bat`, `deer`, `beetle`, `warg`, `dire_warg` | a ranger in the barrows may tame a giant rat; one who stalks the orcs' kennels, a warg (level 7 at a ranger's 4 or more; it must be unaware). Not the undead hounds, the hell hound, the blink dog (fey), the basilisk |

Adding creature types invalidates saves (`docs/dev/saves.md`): the PR says so and leaves
`FORMAT_VERSION` alone.

## 12. Determinism, saves, replays

- **Streams**: the tame check on the gameplay stream (two draws, only when a check is rolled);
  the name on `companion/<actor id>` (one draw, `Rng::stream`); the growth draws nothing; the
  orders draw nothing; the AI draws nothing (its one random draw stays `slip::looks_at`'s).
- **Saved**: `Actor::leader`, `order`, `grade`, `given_name` (each `serde(default,
  skip_serializing_if)`, hashed when set), `WorldState::companion` (the same). Old saves load;
  a non-ranger run saves byte for byte as before. No `FORMAT_VERSION` bump for the fields; the
  new creature types bump nothing either (the integrator's rule), but saves made before them are
  invalid as `docs/dev/saves.md` says.
- **Hashes**: a run without a ranger hashes as before (the companion's code runs only for actors
  with a `leader`). Every **ranger** run gains an actor at the start, so the replays of ranger
  runs are re-created (`replay --create`), and the two showcase scenarios with a ranger (`world`,
  Osric; `forest`, Maren) show a hound beside the player: the golden entries on those scenarios
  (the overworld's sun and weather captures, the forest's dawn) change once and are updated
  deliberately in C1's finisher pass ("Golden screenshots"). No new golden: the companion is a
  glyph a core test and a dump can see.
- **Replays**: `tame:DIR`, `order:heel`, `order:stay`, `order:at:X,Y`, `order:free`,
  `use:sic:X,Y` in the action text (CLAUDE.md's list, `action.rs`).
- **Invariants** (the tests of C0 and C1): an ally never attacks its leader nor anyone the leader
  is neutral to; it follows within `reach` and not from farther nor under Stay; a tamed beast's
  grade is the ranger's level and rises with it; a let-go beast has no leader and its type's
  faction; a companion's kill pays the player exactly once (not twice through the tally); a
  non-ranger run's hash is unchanged by the PR; a save round trip keeps every new field; a
  replay with a tame, three orders, a level change, and a death verifies.

## 13. What the player sees

### 13.1 Messages (words, never a number)

| Moment | Text |
|---|---|
| the start | "Snel, your hound, pads at your heel." |
| an order | "Snel comes to heel." / "Snel stays." / "Snel goes." (no text when nothing changes) |
| taming | "You crouch by the wolf and hold out some dried meat." (the first turn) … "The wolf comes to your hand. You call it Græg." / "The wolf snaps at you and backs away." |
| the stairs | "Snel follows you down." / "Snel stays behind." |
| growth | "Snel grows stronger." |
| a blow | "Snel bites the kobold." / "The kobold bites Snel." (the usual attack messages with `World::called`) |
| hurt by you | "Snel yelps." |
| death | "Snel dies." |
| let go | "Græg looks back once, and goes." |
| Sic 'em | "Snel goes for the goblin's throat." |

### 13.2 The status bar, look mode, the screens

- **The status bar** gains a short line while the player has a companion on their level: its
  name and health in words ("Snel: unhurt", "Snel: badly hurt"; the `health` words of any seen
  actor), and "Snel: away" when it is on another level. A text snapshot (`screen_snapshots.rs`).
- **Look mode**: "Snel, your hound (companion)"; a tameable wild beast: "a wolf (neutral)" as
  today, with "It has not noticed you." from Stalker's eye as today.
- **The Character screen** (page 1) gains a line under the class: "Companion: Snel, a hound".
- **The abilities menu**: the Allies group (Heel, Stay, Go there, Let go) while an ally exists;
  Tame in the Class group with its reason when refused ("you already have a companion", "no
  beast there", "it has seen you", "too strong for you").
- **The one dialog** for Let go.

### 13.3 The CLI and dumps

- `dump --layers companion`: `Snel, a hound (grade 12) at (12, 5): hp 30/30, order heel`; `no
  companion` without one; JSON `companion { name, creature, grade, hp, hp_max, order, pos, level }`
  (null without one).
- `allies` (shared): every ally of the player, one a line, the same fields.
- `actors` (dev) adds `; leader #0, order stay, grade 12, named Snel`.
- `abilities` lists the orders with their source `(ally)` and Tame `(class)`.
- The `sound` layer shows a whistle as `w`.
- Action text: §12.

### 13.4 The wiki

`wiki/src/companions.md` (under The character, after Classes, in `SUMMARY.md`; `TERMS.md` gains
"companion", "tame", "orders", "grade"): the hound, how a beast is tamed (the exact check and the
level gate), the orders, the following rule, the growth table at the cadence levels for every
tameable beast (generated by `xtask/src/wiki/companions.rs` from the data), the kill credit,
friendly fire, what Pack tactics, Beast bond, and Sic 'em do exactly. The Classes page's ranger
row links it. The creatures table marks tameable types. The spec gains "The animal companion"
in spec/08i-character.md under "Class abilities" and "Allies" in spec/08b-creatures.md;
`docs/dev/companion.md` and `docs/dev/allies.md` are the topic files.

## 14. Talents and class abilities

**One new system, `allies`**, and one talent that earns its place: allies are a broad rule set a
talent can change (who follows, how far, how they fight), not a sub-feature of factions; the
companion and the druid's summons both read it. Whichever PR lands first adds `[[system]] id =
"allies"` (beside `factions`) and its talent.

| Talent | Tree | Needs | Effects | Why it earns its place |
|---|---|---|---|---|
| **Pack leader** | Wayfarer | Presence 12, or Handle animal 15 (`either`) | `ally_to_hit = 1`, `ally_defence = 1`, `follow_reach = 2` | a real choice for every build with allies (a ranger's hound, a druid's summons): it changes how the allies fight and how freely you move with them; nothing else gives it; it is not information, not niche (every dungeon floor), not a clone (no other talent touches allies) |

Read in one place each: `ally_to_hit` and `ally_defence` in the growth hooks (§8: an ally's
profile and Defence add its leader's `TalentMods`), `follow_reach` in `bring_followers`.

**The class abilities** (§10): Animal companion (`allies`, `factions`), Pack tactics (`combat`,
`allies`), Beast bond (`vision`, `allies`, `hit_points`), Sic 'em (`combat`, `resources`,
`allies`). Tags match hooks: each reads the system it names. Nothing else is minted: no "faster
taming", no "second companion", no "companion hit points" talent (a small percentage on one
class's pet is rule-bar filler; the growth table is where those numbers live). Handle animal gets
its first use (Tame) without a talent of its own. Beast-friend stays the Wayfarer pick and the
druid's level 1: a ranger who wants calm rats takes it; the companion respects it through the
leader's stance.

What this does **not** do for the coverage policy: `handle_animal` is a skill, not a system; the
companion's growth is read by the `allies` hooks and needs no tag of its own.

## 15. Rejected ideas

| Idea | Why not |
|---|---|
| The companion at 18 (the first gap), an archer until then | the owner's "pet class" is felt from turn one, or it is not one; NetHack's and WoW's pets are there from the start |
| A choice of beast at character creation | one more creation step for one class; the hound and Tame do it, and the wild beasts are the dungeon's reward |
| Two or more companions, a pack (D2's druid) | that is player summons, the druid's design; one hound keeps the fiction and the AI's cost |
| Tameness, happiness, feeding (NetHack, classic WoW) | fiddly meters their own designers removed; creatures have no hunger to simulate; food is a one-time bonus on taming instead |
| The companion's own XP and levels (NetHack, ADOM) | a second curve the player never sees, and a level 1 pet at depth 10; the grade follows the ranger's level |
| Growth by swapping the type (hound → great hound at 18 and 40) | three steps over sixty levels leaves the pet weak for most of them; continuity (its name, its hit points) is lost |
| Revive, a stable, resurrection for gold (D2, WoW) | permadeath world: tame another; a stable is a second companion in disguise |
| A grief status on the ranger when the companion dies | a flaw must bite in a dungeon (rule 5), and this one would only sting; the loss is the hound |
| The pet attacks only when the ranger spends an action (5e Beast Master) | the subclass that taught everyone not to; the companion acts on its own turn |
| A tameness roll on each order (disobedience) | WoW and DCSS allies obey; a disobedient pet is a worse AI, not a mechanic |
| New rebindable keys for the orders, a pets screen | the hard rule: the one abilities menu, fixed letters; the status bar line is the screen |
| Taming a hostile aware beast while it fights you (WoW's channel) | it must be neutral or unaware: Stalker's eye's purpose, and no "calmed" state to invent |
| A general "anyone trained in Handle animal tames" (ADOM, NetHack) | the owner's: the ranger is the pet class (§17 Q2 asks) |
| The companion picks things up, steals from shops (NetHack) | charming, out of scope; no inventory on the companion |
| Companion gear: a collar, armour | no slots on a beast; later, perhaps a collar slot (§17 Q11) |
| A revive-on-death "loyal beyond death" capstone | a death undone is the one thing this world does not do |
| Stealth at 40 (Camouflage's twin underground) | the class's direction is the pet; Sic 'em is the fight changer a capstone tier asks for |
| Cooldowns on Tame or Sic 'em | Focus and time are the prices; no anti-farming limits (the owner) |
| Mounts | out of scope; a line on #196 |

## 16. Implementation plan

| Slice | Issue | What it touches | Depends on |
|---|---|---|---|
| **C0 The ally core** (or the druid's equivalent slice: whichever lands first) | allies: leader, orders, the AI, following, grade, kill credit, the `allies` system | `warlock-core`: `Actor::{leader, order, grade}`, `world/ally.rs` (the plan, `bring_followers`, stance by leader), one branch in `ai.rs`, one call in `travel.rs::travel`, `Source::Ally` and `Ability::Order` in `ability.rs`, `Action::Order`, the action text, `Act::Whistle`, the growth hooks (`Actor::max_hp`, `stanced`, `defence`, `protection`, stats), `World::dies`'s credit, `SeenActor::ally`, the `allies` dump layer and the `actors` additions, `data/companions.toml`'s `[follow]`, `[orders]`, `[growth]`, `[kills]`, the `allies` system tag and Pack leader (`data/talents/pack_leader.toml`, `TalentEffect::{AllyToHit, AllyDefence, FollowReach}`); `world/ally_tests.rs`; `docs/dev/allies.md`, CLAUDE.md's action text and layers; the wiki's talent rows (by itself) | — (coordinated with #821) |
| **C1 The companion** | the ranger's Animal companion: the hound, Tame, Let go, the name, the table | `animal_companion` talent and the ranger's table (level 1), `TalentEffect::Companion`, `hand_out`'s arm, `world/companion.rs` (`bring_companion`, the level-up grade, Let go), `world/tame.rs` (`ClassActive::Tame`, the channel, the check, the food, marks), the `tame` benefit on Handle animal, `CreatureDef::tameable`, `creatures/hound.toml`, `tameable` on the six types, `WorldState::companion`, `Actor::given_name` and `World::called`, the `companion` dump layer and JSON, the one dialog for Let go (the app, from the view), the status bar line and look mode (text snapshots), `data/companions.toml`'s `[tame]` and `names`; tests (§12); `docs/dev/companion.md`; spec; the wiki page and its generated growth table; the ranger replays re-created; the `world` and `forest` goldens updated in the finisher's pass | C0 |
| **C2 Pack tactics** | level 18 | `pack_tactics` talent, `TalentEffect::{PackTactics, SharesFavouredPrey}`, `world/pack.rs` (the flanking read in the profile), the table row, tests, wiki | C1 |
| **C3 Beast bond** | level 25 | `beast_bond` talent, `TalentEffect::{SharedSight, Whistle, FollowReach, TendsAlly}`, `perception.rs`'s added cells (`world/bond.rs`), the whistle in `ally.rs`, `heal.rs`'s ally candidate, the table row, tests (shared sight equals the companion's `sight::sees`; memory; dazzle), a `--capture` looked at, wiki | C1 |
| **C4 Sic 'em** | level 40 | `sic` talent, `ClassActive::Sic`, `world/sic.rs`, the `savage` status (`auto_critical`, eased by hits) and its read in `strike_with`, the table row, tests, wiki | C1 |
| **C5 The wild beasts** | wolf, boar, lynx, bear; the local generators and the Ascmearc's wildlife entries | `creatures/{wolf,boar,lynx,bear}.toml`, `mapgen/local/` (the glade's wolves and boars, the hills' bear, as the deer), `campaigns/ascmearc/data/encounters/*.toml` wildlife rows, bestiary and roster tests, wiki | C1 (tameable) |
| **C6 The balance probe and the bench** | the ranger-and-hound row in `balance_probe`, a companion on `bench-turns`' `delf` row | `balance_probe`, `xtask/src/bench.rs`, `docs/balance.md`, `docs/perf.md` | C1 |

C0 and C1 add saved state and actions: two PRs at once at most with the druid's (the integrator's
rule). C2–C4 are one talent each and may run in parallel after C1. No golden is added; C1 updates
the two the hound appears in.

## 17. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| Q1 | **The grade**: the ranger's full level (100 %, WoW, D2) or half (D&D 3.5)? | 100: the hound is the class's damage and its tank; the balance probe (C6) says if it needs trimming, and `grade_percent` is one number |
| Q2 | **Tame ranger-only**, or open to anyone trained in Handle animal (ADOM, NetHack)? | ranger-only: "our pet class"; Handle animal's `tame` benefit is where the skill matters, and a druid's beasts are the druid's summons |
| Q3 | **A companion's kill pays full XP and the tally** (DCSS), or half, or none (ADOM, NetHack)? | full and the tally: the hunt is one; `kill_xp_percent` is data |
| Q4 | **The starting beast**: always a hound, or by race (an elf's lynx, a dwarf's badger)? | a hound for everyone; the wild beasts give the variety, and a race table is one more thing to tune |
| Q5 | **Follow reach** 2 at the start, 6 with Beast bond? ADOM's and NetHack's 1 is the classic | 2 and 6: adjacency is the classic complaint |
| Q6 | **Shared sight at 25** (Beast bond): too strong, or exactly the fun? | yes: it is what a pet class is remembered for, and it comes at 25, when scouting rooms with a hound is the ranger's way |
| Q7 | **Orders free of time** with a whistle's noise, or a fifth of an action? | free, with noise: DCSS's rule; stealth pays in the noise |
| Q8 | **Friendly fire** from the ranger's own bursts and volleys hurts the companion | yes: honest, and the reason to order Stay before a fireball |
| Q9 | **Stay means not following across levels** (DCSS) | yes: an order is an order |
| Q10 | **Death**: nothing but the loss | yes: no grief status, no cooldown, no revive |
| Q11 | **Companion gear** later (a collar slot for one magic item)? | not now; a line on #196 if the owner likes it |
| Q12 | **The companion on the overworld** as an actor beside the player, or carried "in the party" and placed on arrival? | an actor: everything exists for it; only following happens there |
| Q13 | **Tame's gate** `over` 3 levels above the ranger's | 3: a level 4 ranger may tame a warg (7) it catches unaware; a bear (8) at 5 |
| Q14 | **Barkskin stays** the ranger's level 6 spell beside the druid's level 3 | stays: a table may give a spell two classes have; the druid's design may move theirs |
| Q15 | **Who adds the `allies` tag and its talent**: this document's Pack leader, or the druid's own | whichever lands first adds the tag; if the druid's doc names a talent for it, Pack leader is judged on its own as a second (it is not a clone of a summons talent: its effects are to-hit, Defence, and following) |

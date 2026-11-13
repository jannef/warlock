# Warlock — The thief's Openings: the mark, the kill that carries, Feint, and Exploit

Design document (2026-11-13), **approved by the owner 2026-11-13** with the answers in §13. Written against `origin/main` at f5eae7338 (version 0.16.x). Nothing here is implemented; §12 maps it onto implementation issues, one PR each. It builds on what is there: the Openings pool and the resource machinery (`docs/dev/resources.md`, `data/resources.toml`, `world/resource.rs`), sneak attacks and surprise (`docs/dev/surprise.md`, `world/ambush.rs`, `world/surprise.rs`, `docs/design/surprise-and-sneak-attacks.md`), the thief's actives (`world/thief.rs`, `world/openings.rs`), Feint (`world/feint.rs`, `docs/dev/skills.md`), the class tables (`docs/dev/progression.md` "Class tables"), and the balance probe (`docs/balance.md` "Sneak attacks").

Every number below is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate). The aim: a thief's fight has a rhythm of its own from level 1, **work a foe, then cash in**, and the class's level-ups make the cashing-in bigger rather than adding unrelated buttons.

## Contents

1. Summary
2. The owner's brief, and what exists today
3. The mark: whose guard the openings are in
4. What builds an opening: sneak attacks, Feint, and the kill that carries
5. Exploit, from level 1, and the thrown exploit
6. The ladder: what each class talent adds to Exploit
7. The level table, the thrown exploit's level, and Trained wind
8. The other talents: Patient blade, Vanishing act, of Guile, Peerless
9. What the player sees; the dumps
10. Balance
11. Determinism, saves, replays, docs, tests
12. Implementation plan: the phases
13. Owner's answers (2026-11-13)
14. Choices made along the way

## 1. Summary

| Piece | Proposal |
|---|---|
| The mark | The pool remembers **whose guard the openings are in**: `Actor::mark: Option<ActorId>`, saved and hashed. Every attack the thief **chooses** (a blow, a shot, a throw, a single-target attack spell, a feint, an exploit) at creature X: no mark → X becomes the mark, the pool kept; mark X → nothing; mark Y, alive → **the pool is emptied** and X becomes the mark. A free blow (riposte, opportunist, the lurk's gatekeeper blow, hair trigger) never costs a mark: it sets one when there is none, else builds nothing on a stranger. §3 |
| The kill that carries | When the mark dies, by any hand, the mark is **cleared and the pool stays** (it carries to the next target); a death by the thief's own hand (a kill credited to them, bleeding included) also **gives one opening** (`Build::MarkKill`, replacing `sneak_kill`); a death by another hand gives none. The next attack, on anyone, adopts the carried pool. The mark is also cleared, the pool kept, when the mark leaves the game otherwise (a summons that lapses) and when the thief changes level. §3, §4 |
| Feint builds | A successful Feint (`world/feint.rs`: the foe falls for it) is a chosen action at a creature: it applies the mark rule and **builds one opening** (`Build::Feint`, `builds = [{ on = "feint", amount = 1 }]` on Openings). §4 |
| Exploit | A new class active at **level 1**, `use:exploit:E`, aimed in a direction at the hostile beside the thief, in the abilities menu only. Refused, no time, nothing spent: no hostile there, no openings, a hostile that is not the mark while the pool is in someone else's guard, a thing with no vitals. It **spends every opening held** (n) for a melee blow at a melee attack's price that is **a sneak attack whatever the foe knows** (Low blow's old effect) and does **+1d6 damage per opening** on top of the sneak dice (after Protection, never multiplied, as sneak dice are). §5 |
| The thrown exploit | A second class active at **level 18** (beside Evasion), **Thrown exploit**, `use:thrown_exploit:c:12,5`: the throw list, then the aiming mode; Exploit's blow as a throw of the chosen item at the creature on that cell, for the mark out of reach (it blinked, it stepped back, or it died and the carried pool adopts whoever is next). Exploit's refusals and its spend of every opening, a throw's price, within the item's throw range and the sneak dice's reach (6, Long reach's farther), the line clear as the player knows it. The ladder applies unchanged. §5.1 |
| The ladder | The class talents add per-opening increments to Exploit (and the thrown exploit) instead of their own actives: **12 Low blow** — the foe is Stunned (flat-footed: the next blow is a sneak attack too) and the exploit is **+1 to hit per opening**; **32 Improved sneak attack** — keeps its +2d6 on every sneak attack and makes Exploit's dice **2d6 per opening**; **60 Master thief** — keeps its threat −2 on the unaware and lowers Exploit's threat **4 per opening** (never below 2); **five or more: a critical whenever it hits, no confirmation** (the finisher, folded in). §6 |
| Trained wind | **Off the thief's table**, nothing in its place (the owner: "the thief already gets Exploit at level 1"). Nothing the thief does before rank 7 costs stamina, and the game already grants the pool when the first active weapon rank arrives (`fight.rs::grant_stamina`, "You learn to pace your breath."). Exploit takes the level-1 slot. The other six classes keep it. §7 |
| The rest | Lurk, Evasion, Pack mule and Porter, Vanishing act, Hide in plain sight stay. Patient blade: decay −50 % as now, and the mark's kill gives **two**. The affix of Guile grows the pool and so the ladder; no cap beyond the pool's maximum. Peerless is the aristocrat's, not the thief's: untouched. §8 |
| Phases | P1 the mark and the kill that carries (Low blow and the finisher gain the mark rule meanwhile), P2 Feint builds, P3 Exploit at level 1, Trained wind out, Low blow reshaped, P4 Improved sneak attack's and Master thief's increments, the finisher removed, the balance probe's table, P5 the thrown exploit at 18. §12 |

## 2. The owner's brief, and what exists today

The brief (2026-11-13): (1) openings **reset when the target changes**, unless the previous target **died**, which **gives** an opening; (2) killing the target the openings were built on **carries them over**: the next attack on a new target neither resets nor consumes them, they count as built on it, plus the kill's opening; switching away from a living target resets; (3) a **successful Feint builds** an opening, under the target rule; (4) thieves get an ability to **exploit an opening from level 1**, **targeted at a monster**: the same target the openings were built on, the openings **consumed for an effect**; (5) the effect is an **escalating progression**: the improved abilities of the level-up talents, **times the openings consumed**; (6) "perhaps Trained wind is not something the thief should get."

| Part | Today |
|---|---|
| The pool | `data/resources.toml` `openings`: `max = 5`, `start = 0`, `decays = 120` (a point every two minutes, resting too), `builds = [{ on = "sneak_attack", amount = 1 }]`, never ends. Granted by the class passive Sneak attack at level 1 (`grants_resource`). `Pool { resource, value, progress }` in `Actor::resources`, saved and hashed. Nothing remembers **whom** the openings were made on |
| Builds | `Build::{Kill, Hit, Critical, Struck, SneakAttack, SneakKill}` (`content/resource.rs`), each raised at one place: `SneakAttack` in `ambush.rs::ambushed` when the sneak dice land (a blow at any reach; a shot, a throw, or an attack spell within 6 cells), `SneakKill` in `fight.rs::resolve` when a sneak attack's blow kills (Patient blade's second opening; `vanishing_act` beside it) |
| Spenders | **Low blow** (12, `use:low_blow:E`, 3 openings): a melee blow that is a sneak attack whatever the foe knows (`Technique::sneak`), the foe keeps its Defence. **Master thief's finisher** (60, `use:master_thief:E`, 5 openings): a melee blow with `Profile::sure_critical` (a critical whenever it hits, no confirmation), **not** a sneak attack unless the target is unaware, so at level 60 it is worth about ×2 of a plain blow (≈30) against the unaware blow's 51. Both aimed `ClassActive`s, refused with nobody hostile that way before the spend (`use_class`, `thief_may`) |
| Feint | `world/feint.rs` (#1016): Bluff's use at a hostile seen beside the player with a mind; its Willpower check against 10 + Presence modifier + Bluff's points; failed, it is Surprised on `World::fresh` (the next blow finds it flat-footed and lands the sneak dice). It builds nothing itself; the blow after it builds one through `SneakAttack` |
| The table | `data/classes/08-thief.toml`: 1 Sneak attack + **Trained wind**, 6 Lurk, 12 Low blow, 18 Evasion (Finesse, handed out), 25 Pack mule + Porter, 32 Improved sneak attack (+2d6), 40 Vanishing act (a sneak kill leaves the thief Light-footed), 50 Hide in plain sight, 60 Master thief (threat −2 on the unaware, the finisher); bonus picks from Shadow, Finesse, Delver at 4, 10, 16, 22, 28, 34, 46, 58 |
| Stamina | `trained_wind` grants it at level 1 on seven tables. The thief's styles spend it only from rank 7 (Duelist's False lead 2, Twin Blades' Whirl 6) and rank 9 (Vital strike 4, daggers); `fight.rs::grant_stamina` holds the pool for anyone whose first active rank arrives without it |
| Related talents | Patient blade (Shadow, thief 32): Openings `resource_rate` −50, `resource_build` +1 on `sneak_kill`. Ambusher (Shadow, Stealth 30, level 9): +1d6 sneak dice. Of Guile (affix, body/ring/amulet): Openings `resource_max` +1 to +3. Peerless (`critical_confirm` +2) is **the aristocrat's level 50**, not the thief's |
| Presentation | The status bar `Openings 3/5` (`PerceptionView::own_resources`), the character screen's row, the dumps' header `openings 3/5`, JSON `resources[]`, the dev `actors` layer's `(progress)`. No message when a point is built or lost |
| Balance | `docs/balance.md` "Sneak attacks": the thief's plain blow against the first blow on the unaware, by level (plain : unaware = 2.6 : 6.0 at 1, 8.0 : 15.3 at 10, 14.0 : 31.4 at 30, 16.2 : 44.0 at 40, 17.2 : 51.2 at 60). The thief's sustained damage is about half the warrior's at every level |

## 3. The mark: whose guard the openings are in

**Decision: one field on the holder, `Actor::mark: Option<ActorId>`, read and written in one module, `world/mark.rs`.** An opening is a gap in one creature's guard; the pool is worth something only against that creature. `ActorId`s are never reused in a run (`actor.rs`), so a dead mark's id is safe to hold until it is cleared.

### 3.1 What moves the mark

| The thief's action at creature X | No mark | Mark is X | Mark is Y, alive |
|---|---|---|---|
| A **chosen attack**: a melee blow by moving or attacking, a shot (`fire`), a throw, a single-target spell that rolls to hit (a scroll's bolt at X; never a burst), a weapon active's blow (Lunge, Vital strike, Whirl's aimed direction), Feint, Exploit (Lurk's wait is no attack; its blow is a free blow, below) | mark := X; the pool kept | nothing | **pool := 0** (said, §9); mark := X |
| A **free blow** at X (riposte, opportunist, the gatekeeper's blow from a lurk, hair trigger, flurry's second blow) | mark := X (there is nothing to lose) | nothing | nothing: the blow lands as any blow, its sneak dice too, but **builds nothing** and moves nothing |
| An area effect (a fireball scroll, Whirl's other victims) | nothing | nothing | nothing |

The rule sits where the player's attacks are gathered, as the surprise hooks do (`docs/dev/surprise.md` "The hooks"): `World::aim_at(target)` called from `fight.rs::player_blow_with` (every chosen melee blow), `shoot.rs::fire` and the throw, `cast.rs::spell_attack` for a single-target spell cast by the player, `feint.rs::feint` (after its refusals, before the check), and `World::exploit`; `World::free_blow_at(target)` from `free_blow.rs` where the player's free blows are struck. `World::aim_at` runs **before** the attack, so its build (a sneak attack's opening) lands in the new mark's pool, never in the old one's.

Why chosen attacks only: a thief with four openings on an ogre must not lose them because a kobold stepped away and Opportunist fired. The thief controls their mark exactly as they control their actions; nothing a creature does moves it. Why a free blow may **set** a mark: the lurk's blow is the classic opener, and it would otherwise build nothing when the pool is empty.

### 3.2 What clears the mark (the pool kept)

| Event | Mark | Pool |
|---|---|---|
| The mark dies, by any hand | cleared | kept, so it carries to the next target either way; **+1 if the kill is credited to the thief**, nothing more by another hand (§4.3; the owner's answer to Q3) |
| The mark leaves the game otherwise (`fight.rs` where `state.actors.remove` is called: a summons that lapses, a creature that departs) | cleared | kept |
| The thief changes level (`World::travel`, the one place the player's level changes) | cleared | kept (it decays as ever) |
| The pool reaches 0 (decay in `World::live`, or Exploit's spend) | cleared | — |
| A chosen attack at another living creature | := the new creature | **emptied** (§3.1) |

A mark that leaves the thief's sight is **still the mark**: losing sight of a foe is not forgetting its guard; the decay (a point every two minutes) is the clock. The level change clears it because a mark on another floor would make the first fight downstairs empty the pool for no reason the player can see; the openings carried are free, and the two trips the cheese would need cost more time than the pool survives (the owner's rule: no anti-farming locks, time is the price).

### 3.3 Creatures

Creatures hold no Openings (`docs/dev/resources.md`) and never will here: no creature has a mark, and nothing in `ai.rs` changes. The drow assassin's fantasy stays its dice and its speed.

## 4. What builds an opening

**Decision: three builds, each one line where it happens, all through `World::gain_resource`.** `Build` gains `Feint` and `MarkKill`; `SneakKill` goes (its two readers, Patient blade and Vanishing act, are re-pointed; §8).

| Build | Where | Amount | The mark rule |
|---|---|---|---|
| `sneak_attack` | `ambush.rs::ambushed`, when the sneak dice land (unchanged: a blow at any reach; a shot, a throw, or an attack spell within `missile_reach` + Long reach) | 1 | the attack already ran `aim_at` (a chosen attack) or `free_blow_at` (a free one); a free blow at a stranger while a mark stands skips the build (`gain_resource` is not called for Openings; the other pools, none for a thief, are unaffected) |
| `feint` (new) | `feint.rs::feint`, on "falls for it" | 1 | `aim_at` ran before the check: a feint at a new foe empties the pool first, then builds one |
| `mark_kill` (new) | `fight.rs::dies`, beside `Build::Kill`, when the creature that died is the killer's mark and the kill is credited to them (`by`; a wound they dealt that bled it to death counts, as `Kill` does) | 1 (Patient blade +1) | the mark is cleared **after** the build, so the point lands |

### 4.1 Missiles count

A thrown dagger or a sling stone at an unaware foe within 6 cells builds, as today, and a shot at a stranger moves the mark (a chosen attack). The thief who softens a foe with thrown knives and closes to exploit is a fine play, and one definition of "attack" is one rule. Exploit itself is a melee blow (§5); the thrown exploit (§5.1) is the same blow as a throw, for the mark the thief cannot reach.

### 4.2 Feint

A feint that fails ("is not fooled") builds nothing and still moves the mark: the thief chose that foe. A feint is a turn spent, so a thief feinting instead of striking trades damage now for a bigger exploit; with the foe Surprised the next blow lands the sneak dice and builds a second opening. Two openings for two turns, against one blow's damage: worth it before an exploit, not instead of fighting.

### 4.3 The kill that carries

The brief's rule, exactly: the mark's death clears the mark and leaves the pool; the thief's own kill adds one. The next attack, on anyone, finds no mark and adopts the pool (§3.1, first column). So a thief who opens on a kobold (1), kills it with the second blow (2), turns to its fellow with a sneak attack from the kill's Light-footed step (3), kills it (4) walks into the third with four openings in hand, and Exploit cashes them. A kill by an ally, a trap, or a creature's own fellow clears the mark and gives no opening, but the pool **still carries**, exactly as after the thief's own kill less the +1 (the owner's answer to Q3): the opening is the thief's moment, not the corpse's, and nothing the thief built is lost because a wolf got there first. A carried pool is often wanted where the next foe is not beside the thief: that is what the thrown exploit is for (§5.1).

## 5. Exploit, from level 1, and the thrown exploit

**Decision: one new class active, `ClassActive::Exploit`, carried by the level-1 class talent `exploit` (class tree, granted), aimed in a direction, in the abilities menu only.** Low blow's and Master thief's actives go (§6); the thief has one blow to spend openings on at every level (from 18 also as a throw, §5.1), and the level-ups make it bigger.

| Aspect | Decision | Why |
|---|---|---|
| Aim | `Aim::Direction`: the hostile beside the thief that way (`hostile_toward`), as Low blow and the finisher are aimed today | the effect is a melee blow; the throw at range is its own ability with its own aim (§5.1), so this one stays one keypress and a direction |
| Cost | **every opening held** (n, 1 to the pool's maximum, no cap beyond it), spent after the refusals; no count prompt; no stamina, no cooldown, no mana | the owner's answers to Q1 and Q9. The brief's "times the number consumed" wants a count the player controls, and they control it by timing: strike at 2 or wait for 5 while the clock runs |
| Time | a melee attack's price (`attack_price` with what is wielded), its exertion (`exert(Melee)`), as the two actives today | it is a blow |
| The blow | `player_blow_with` with the thief's melee profile and `Technique::sneak` set: **a sneak attack whatever the foe knows** (the sneak dice land on a hit; the foe is not Surprised by it and keeps its Defence, as Low blow's foe does today); **+1d6 damage per opening spent** on a hit, rolled from the gameplay stream after the sneak dice, added after Protection and never multiplied by a critical (`ambush.rs::ambushed`'s rule for sneak dice: one path, one place, `Exploit::dice(n)`) | the openings are gaps in the guard: the knife goes into them. Level 1 gets Low blow's fantasy ("make the aware foe pay") and a reason to count |
| Refusals (no time, nothing spent, said) | "Which way?" (no direction); "There is no one there." (no hostile that way); greyed "you have no opening" (`lacking` on the resource, `Unavailable::Short`); **"Your openings are in the kobold's guard."** (a mark stands and it is not this creature; the mark named as the player knows it, `called`); "It has no vital spot." (`Content::has_vitals` false: the bones, the spirits, the animated; the player sees the type; the owner's answer to Q4) | Feint refuses a mindless foe the same way; wasting five openings on a skeleton for nothing is a trap, not a choice, and vitals are by type, so no hidden fact leaks |
| On a free pool | allowed at anyone: `aim_at` sets the mark, the exploit spends | the brief's carry-over |
| After the blow | the pool is 0; the mark is cleared at the end of `World::exploit` unless the blow killed it (then the kill rule cleared it and gave its opening). Vanishing act fires on a kill as on any sneak kill | the thief lands on 1 opening and no mark after a killing exploit: the loop restarts cleanly |
| Who | the thief only (the talent is on its table); creatures never (actives are the player's) | — |
| Action text | `use:exploit:E` | the one grammar for class actives |

The exploit's own sneak dice are the thief's as ever (the Sneak attack skill's d6s, Ambusher's, Improved sneak attack's): Exploit(n) at level L does the plain blow, plus the sneak dice, plus n × the ladder's dice (§6).

### 5.1 The thrown exploit (level 18)

**Decision: a second class active, `ClassActive::ThrownExploit`, carried by the class talent `thrown_exploit` (class tree, granted at level 18), the same blow as Exploit delivered by a throw, in the abilities menu only** (its `KeyCommand` unbound, as every ability's). It answers the owner's answer to Q8: the openings are worth something when the mark is out of melee reach, because it blinked, stepped back, or fled, or because it died and the carried pool would adopt a foe across the room.

| Aspect | Decision | Why |
|---|---|---|
| Name, id | **Thrown exploit**, `thrown_exploit`; the menu row `Thrown exploit (class): something to throw, then a target` | it names the one thing that differs from Exploit |
| Aim | a new `Aim::Throw`: the **throw list** (the `T` chooser's list of what the player may throw), less flasks, then the **aiming mode** (`InputMode::Aim`, `AimKind::Ability`) at the chosen item's reach (below), the cursor starting on the mark when it is seen, else as a throw's does. One variant, read by the abilities menu as the item list and then the cell | the owner's words: "the throw item selection followed by target selection". A flask shatters on its cell (`world/flask.rs`) and has no point to put into a gap |
| Action text | **`use:thrown_exploit:c:12,5`**: stack `c` thrown at level cell (12, 5). The `use:ID` grammar of class actives, with the stack position of `use:identify:c` and the cell of `use:magic_missile:12,5`, in the order `throw:c:12,5` gives them (what, then where) | abilities are `use:`; `throw` stays the plain throw and keeps its two fields |
| Cost | **every opening held** (n), as Exploit, spent after the refusals; one of the stack leaves the hand as any throw does (`shoot.rs::loose`: it flies, comes down on the pile or breaks); a throw's price (`attack_price` of `throw`) and its exertion; no stamina, no cooldown | the brief's price is the openings; the throw's own is the knife lying beside the foe |
| Reach | the item's throw range (`Actor::throw_range`), **capped at the sneak dice's reach** (`missile_reach` + `sneak_reach`: 6 cells, Long reach's farther; `ambush.rs`) | the exploit is a sneak attack whatever the foe knows, and the sneak dice land on a missile only within that reach; an exploit beyond it would spend the openings on dice that cannot land |
| Refusals (no time, nothing spent, said) | Exploit's, in its words: greyed "you have no opening"; "There is no one there." (no hostile the player sees on that cell); **"Your openings are in the kobold's guard."** (a mark stands and it is not this creature; a carried, unmarked pool adopts anyone); "It has no vital spot." Plus throwing's: "That is out of reach." (beyond the reach above); "Something is in the way." (a wall or a creature **the player sees** on the line before the target, as the aiming mode draws it) | the owner's "same refusals as Exploit, plus range/line rules of throwing". The line is judged by what the player knows, so no hidden creature leaks |
| The flight | the throw's own (`World::fly`): a creature **the player did not see** in the line takes it as a plain throw (the openings spent, no sneak dice of the exploit's, no ladder dice, the mark unmoved: the player did not choose it); a miss spends them as Exploit's miss does | the risk any throw has; refusing on an unseen creature would tell the player it is there |
| The hit | the item's thrown profile (`Actor::thrown`, the Thrown rank) with `Technique::sneak` set, the sneak dice, and **the ladder exactly as Exploit's** (§6): +n × 1d6 (2d6 from 32), Low blow's stun and +n to hit, Master thief's threat −4n (never below 2) and the sure critical at five or more. One resolution for both, `openings.rs::land_exploit(n, target, hit)`, called from Exploit's blow and from the thrown exploit's missile hit | the owner's "resolves as Exploit does but with the thrown item's attack at range". Range already has its price: a thrown dagger's profile is weaker than the wielded one, the thief's Thrown rank is lower than their Daggers, the knife is left on the floor, and the reach is six cells |
| The mark | `aim_at(target)` before the throw (a chosen attack); after it the pool is 0 and the mark cleared, unless the throw killed it (the kill rule cleared it and gave its opening); Vanishing act on a kill as on any sneak kill | Exploit's rules, one path |
| Words | Exploit's three lines (§9) before the throw's own flight and hit lines | one voice for one blow |
| Who | the thief only, from 18 | §7 |

Why a second ability and not an aim on Exploit: Exploit stays one keypress and a direction for the case that comes every fight; the thrown one asks two questions (what, where) and is wanted only when the mark is away. The abilities menu lists both.


## 6. The ladder: what each class talent adds to Exploit

**Decision: each class talent on the table keeps its passive, loses its active, and adds one increment to Exploit, every increment × n.** The brief's "the improved abilities of the level-up talents, times the openings consumed", read as one blow that grows twice: in n, and in what the thief has learned. Data: a new talent effect `exploit { ... }` per increment, collected into `TalentMods::exploit: ExploitMods { dice: Dice, to_hit: i32, threat: u8, stun: Option<StatusId> }`, read in one place, `openings.rs::land_exploit`, which Exploit's blow and the thrown exploit's missile hit both call (§5.1): every increment applies to both.

| Level | Talent | Keeps | Adds to Exploit, per opening spent (n) | Effect kind |
|---|---|---|---|---|
| 1 | **Exploit** (new, `exploit`) | — | the blow is a sneak attack whatever the foe knows; **+1d6** × n | `exploit_dice { dice = "1d6" }` |
| 1 | Sneak attack | grants Openings; `surprise_difficulty` 2 | — | — |
| 12 | **Low blow** (reshaped) | — (its active goes) | the foe struck is **Stunned** (a degree, `stunned`: flat-footed, so the thief's next blow finds it without Dexterity and Dodge and lands the sneak dice; its next action costs half again as much); **+1 to hit** × n | `exploit_status { status = "stunned" }`, `exploit_to_hit { value = 1 }` |
| 32 | Improved sneak attack | +2d6 on every sneak attack (`ambush 2d6`) | Exploit's dice become **2d6** × n (a second `exploit_dice { dice = "1d6" }`; `Dice::and` sums dice of one size) | `exploit_dice` |
| 60 | Master thief | threat −2 on the unaware (`ambush_threat 2`) | **threat 4 lower** × n, never below 2 (every hit threatens); **with five or more spent, a critical whenever it hits, no confirmation** (`Profile::sure_critical`: the finisher) | `exploit_threat { value = 4, sure_at = 5 }` |

Why these: Low blow's sand is the setup move (the stun hands the thief a second sneak attack and a dear action for the foe: the brief's "improved abilities" at 12 are *control*); Improved sneak attack is the class's damage talent and makes Exploit the biggest blow in the game at 32 and after; Master thief's finisher is kept exactly at five and made reachable below it. The three stack: a level-60 exploit of five is Stunned, +5 to hit, +10d6 on the sneak dice, and a sure critical on the weapon's part.

The sneak dice (the skill's, Ambusher's, Improved sneak attack's passive) and the ladder's dice are both "after Protection, never multiplied" (spec §8 "Surprise and sneak attacks"): the critical doubles the weapon's roll alone. On a target without vitals Exploit is refused (§5), so there is no "dice that do not land" case.

Why not scale the stun: Stunned is `per_action`, `max = 1`, and ends with the foe's next action; statuses that ease by actions have no degrees today (`docs/dev/weapon-abilities.md`). A Sluggish degree per opening (a minute of dear actions each) was the alternative in Q2, and the owner took the stun (Stunned + to-hit per opening); the stun's one flat-footed action is the cleaner promise ("your next blow is a sneak attack too") and is what Low blow's sand was always for.

## 7. The level table, the thrown exploit's level, and Trained wind

**Decision: Trained wind leaves the thief's table and nothing takes its place (the owner's answer to Q6: "the thief already gets Exploit at level 1"); Exploit joins Sneak attack at level 1; Thrown exploit joins Evasion at level 18. Nothing else moves.**

| Level | Today | Proposed | Note |
|---|---|---|---|
| 1 | Sneak attack, Trained wind | **Sneak attack, Exploit** | §5 |
| 6 | Lurk | Lurk | unchanged; its blow sets a mark when none stands (§3.1) |
| 12 | Low blow (active, 3 openings) | **Low blow** (Exploit's stun and to-hit) | §6 |
| 18 | Evasion | Evasion, **Thrown exploit** | Evasion unchanged (Finesse, handed out; a pick in its place if held); Thrown exploit §5.1, below |
| 25 | Pack mule, Porter | Pack mule, Porter | unchanged (#1198) |
| 32 | Improved sneak attack | Improved sneak attack (+ Exploit's second die) | §6 |
| 40 | Vanishing act | Vanishing act | unchanged; exploits are sneak attacks, so an exploit that kills leaves the thief Light-footed |
| 50 | Hide in plain sight | Hide in plain sight | unchanged |
| 60 | Master thief (threat −2, the finisher) | Master thief (threat −2, Exploit's threat and the sure critical at five) | §6 |
| Bonus picks | 4, 10, 16, 22, 28, 34, 46, 58 from Shadow, Finesse, Delver | unchanged | — |

**Why Thrown exploit at 18.** It is the table's thinnest row: Evasion is a Finesse talent that a thief who took Finesse picks at 4, 10, or 16 already holds, so the row is often only a pick in its place. It keeps the table's six-level rhythm (1, 6, 12, 18, 25, ...) and gives the eighteen-level gap between Low blow and Improved sneak attack a step. It comes after Low blow on purpose: the thief learns the melee rhythm with Exploit first, and by 18 an exploit carries Low blow's stun and to-hit, so a throw of four openings across a room is worth taking rather than a novelty. And by 18 the carry-over runs through packs, where the next foe is as often across the room as beside the thief. The thief starts trained in Thrown (`weapon_skills = { thrown = 1 }`) and carries daggers, so nothing new is needed to use it. Earlier (6, beside Lurk) would put a second button on the table before the first has grown; later (25, 32) would leave the carried pool with no answer to a foe out of reach through most of the game.

**Trained wind**: the thief has no stamina use before rank 7 of a style (False lead 2, Whirl 6) or rank 9 of daggers (Vital strike 4), and those ranks come at level 20 to 36 by the probe (`docs/balance.md` "Thief": rank 7 at 12–13, rank 9 at 26–28). When the first active rank arrives, `fight.rs::grant_stamina` already holds the pool for a player without it ("You learn to pace your breath."), as it does for the wizard who learns a staff. So the thief loses a bar that read `Sta 10/10` for twenty levels and did nothing, and gains the class's own button at level 1. Deep lungs and Steady breathing stay takeable once the pool exists (their prerequisites are the stamina system's, not the talent's). The other six tables (warrior, berserker, paladin, ranger, ascetic, bard) are untouched.

A save holding `trained_wind` on a thief keeps the talent and the pool (a dropped table row is not a dropped talent: `class::grant` gives what is missing and takes nothing away; `docs/dev/progression.md`); a new thief has neither until a rank grants the pool. The level-up screen's preview of the table and the wiki's class table follow the data.

## 8. The other talents

| Talent | Today | Proposed | Why |
|---|---|---|---|
| **Patient blade** (Shadow, thief 32) | Openings `resource_rate` −50; `resource_build` +1 on `sneak_kill` | `resource_rate` −50 (unchanged); `resource_build` +1 on **`mark_kill`**: the mark's kill gives two. Description: "Your openings fade more slowly, and a foe you have been working on gives you two openings when it falls." | the build it rode on goes; the mark's kill is its natural successor and strictly better for the thief (any kill of the mark, sneak or not) |
| **Vanishing act** (40) | `sneak_kill_status light_footed`, read where `Build::SneakKill` was raised | unchanged in data; its hook moves to the kill path beside `MarkKill` and still asks "was this a sneak attack that killed" (`unseen`), not "was it the mark" | a sneak kill of a stranger by a free blow still vanishes the thief; nothing the player knows changes |
| **Of Guile** (affix) | Openings `resource_max` +1 to +3 | unchanged; the ladder has **no cap** beyond the pool's maximum, so eight openings are +8d6 (or +16d6 at 32) and threat 2 at 60 | a rare suffix that makes the thief's button bigger is what an affix should do |
| **Ambusher** (Shadow) | +1d6 sneak dice | unchanged; its die lands on every exploit (a sneak attack) | — |
| **Peerless** (the aristocrat's 50) | `critical_confirm` +2 | untouched: it is not the thief's | the brief listed it among the thief's; it is in `12-aristocrat.toml` only |
| Knife in the dark (daggers 5, `sneak_attack` trigger) | a sneak attack that hits bleeds the foe | unchanged; it fires on every exploit too (`sneak_trigger`) | — |

## 9. What the player sees; the dumps

**Decision: the bar counts, the words name the mark, nothing says a number but the pool's own.**

| Where | Today | Proposed |
|---|---|---|
| Status bar | `Openings 3/5` | unchanged |
| Character screen | the resource row `Openings 3/5` | `Openings 3/5, in the kobold's guard` while a mark stands and is seen or remembered by name (`called`); plain while none |
| Look mode | statuses, "unaware of you" (Stalker's eye) | adds **"your mark"** after the seen statuses on the creature the openings are in (`SeenActor::marked`, computed from the player's own `mark`, never a creature's secret) |
| Messages | none on a build or a decay | **"Your openings on the kobold are lost."** when a chosen attack at another living creature empties a pool above 0 (`say`, before the attack's own line). Nothing on a build (the bar ticks) or on the carry (the kill line is enough). Exploit: **"You strike at the opening."** (n = 1), **"You press your openings."** (2 to 4), **"You tear into every opening!"** (5 or more), before the blow's line; the blow's own words follow (its hit, its critical), and Low blow's stun shows as the status does on any creature: "stunned" in look mode (its `seen` word), the foe's dear action unremarked |
| Abilities menu | `Low blow (class): a direction, 3 openings`, `Master thief (class): a direction, 5 openings` | **`Exploit (class): a direction`** and, from 18, **`Thrown exploit (class): something to throw, then a target`**, both greyed "you have no opening" while empty; the rows say what they spend in words, "spends every opening", never n |
| Dumps, text | header `openings 3/5` | unchanged header; the `stats` layer gains `Mark the kobold` after `Effects` while one stands; the dev `actors` layer's player line gains `mark #12`; `abilities` lists `Exploit (class): a direction` and `Thrown exploit (class): something to throw, then a target` |
| Dumps, JSON | `resources: [{id, current, max}]` | each entry gains `"on": "the kobold"` (the mark as the player would call it) or `null`; seen actors gain `"marked": true` on the mark; `abilities[]` gains `exploit` and `thrown_exploit` (its `aim` `throw`) |
| Wiki | `resources.md` "Openings", `classes.md`'s thief paragraph, `abilities.md` Low blow and Master thief, `surprise.md`'s build line, the generated class table and talent rows | rewritten for the mark, the three builds, Exploit, the thrown exploit (its reach, its refusals) and the ladder with exact numbers (`xtask/src/wiki/resources.rs` gains the ladder table from `ExploitMods` by level, read from the thief's table and the talents' data) |

## 10. Balance

The probe's "Sneak attacks" table (`docs/balance.md`) gives the thief's plain blow and the first blow on the unaware; Exploit(n) is the unaware blow's damage (the sneak dice land) plus n × 3.5 (1d6) below 32 and n × 7 (2d6) from 32, the to-hit the plain one's (the foe keeps its Defence) plus n from 12, and at 60 the weapon's part doubled at n ≥ 5 (a sure critical) and nearly so at n = 3 (threat 10). A first pass, to be replaced by the probe's own table (P4):

| L | Plain blow | Unaware blow | Exploit 1 | Exploit 3 | Exploit 5 | Of the monsters' hp at that level |
|---|---|---|---|---|---|---|
| 1 | 2.6 | 6.0 | 9.5 | 16.5 | 23.5 | a kobold's 6 to 8: Exploit 2 is a kill |
| 10 | 8.0 | 15.3 | 18.8 | 25.8 | 32.8 | — |
| 30 | 14.0 | 31.4 | 34.9 | 41.9 | 48.9 | a gorgon's 176: a quarter |
| 40 | 16.2 | 44.0 | 51 | 65 | 79 | Hrīmwyrm's 252: a third |
| 60 | 17.2 | 51.2 | 58 | ≈ 85 | ≈ 103 | Nīðdraca's 337: a third; today's finisher ≈ 30 |

How fast five openings come, in the fights the probe plays: the first blow of a fight is a sneak attack (1); a kill (1); a feint lands about half the time against a foe of the thief's level (Willpower modifier against 10 + Presence + Bluff) and costs a turn; a lurk's blow (1). A fight against one foe yields two or three openings; against a pack with kills in it the pool rides at three to five; against a boss the thief feints and strikes alternately for five in six to eight turns. At level 30 that is an exploit of three worth 42 every third or fourth turn on top of blows of 14: about ×1.7 the thief's sustained damage against a single foe, which brings them from half the warrior's to near it **when they play the rhythm**, and leaves the plain thief where they are. Against packs with kills the carry-over makes it better than that, which is the point of the brief's second rule: the thief is the class that gets stronger as the bodies drop.

What the probe will measure (P4, `balance_probe/tables.rs::exploits`): the thief's exploit at n = 1, 3, 5 on an aware monster of their level with vitals, by level, beside the plain and unaware blows; and the mean openings a fight of the "Fights" table yields with the three builds (the probe's fights already feint nothing; a feint-when-the-foe-is-aware policy is added for the thief alone). `docs/balance.md` gains the table and the "Sneak attacks" paragraph says Exploit.

The thrown exploit is the same sum on the thrown profile: a thrown dagger's weapon part and the Thrown rank's to-hit in place of the wielded blow's, the sneak dice and the ladder's dice unchanged. Its numbers are a little below Exploit's at the same n; its worth is the turns not spent closing on a mark that blinked or a foe across the room. The probe adds a thrown column at n = 3 (P5).

Hidden numbers stay hidden: the game says "You tear into every opening!", the wiki says +10d6.

## 11. Determinism, saves, replays, docs, tests

| Concern | Decision |
|---|---|
| Draws | Exploit's blow draws as any blow; its ladder dice are drawn from the gameplay stream after the sneak dice, on a hit only. The mark rule draws nothing. A thief run's hash moves from its first chosen attack (`Actor::mark` is hashed) |
| Saves | `Actor::mark: Option<ActorId>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`: absent from every other actor and from a thief with no mark. Saves are pinned to the version (no shim): a save from before this is a save from another version. `ClassActive::LowBlow` and `MasterThief` go from the enum; `Exploit` and `ThrownExploit` are added in their place in `ALL` (by group, not at the end) |
| Replays | `use:exploit:E` and `use:thrown_exploit:c:12,5` (the parser takes `:LETTER:X,Y` for this id); `use:low_blow` and `use:master_thief` are no longer parsed. The golden manifest names neither |
| Spec | `spec/08i-character.md` "Class abilities" (the thief's rows), `spec/08e-blood-and-status.md` "Resources" (the builds, the mark), `spec/08c-combat.md` "Surprise and sneak attacks" (the build line) |
| Dev docs | a new topic file **`docs/dev/openings.md`** (the mark, the three builds, Exploit, the thrown exploit, and the ladder, the hooks table, the messages, the tests) and one line for it in CLAUDE.md's topic list; `docs/dev/resources.md` "Openings" points at it; `docs/dev/surprise.md`'s hooks table gains the `aim_at`/`free_blow_at` rows and loses `sneak_kill`; `docs/dev/skills.md` Feint gains its build; `docs/dev/progression.md` the table |
| Tests | `world/mark_tests.rs`: a chosen attack sets, keeps, or empties (the message once, only above 0); a free blow sets when none and never empties; a shot and a throw count, a burst does not; the mark's death by the thief carries and adds one, by an ally carries and adds none; a lapsed summons clears; a level change clears and keeps the pool; decay to 0 clears; a save and a replay. `world/openings_tests.rs`: Exploit's refusals (each, no time, nothing spent), its spend of all, the dice by n at 1 and 32, Low blow's stun and to-hit, Master thief's threat by n and the sure critical at five, the no-vitals refusal, the kill in an exploit (one opening, no mark, Light-footed with Vanishing act), the hash. `world/thrown_exploit_tests.rs`: each refusal (no openings, not the mark, no vitals, a flask, out of the item's range, beyond the sneak reach, a seen wall or creature in the line), the spend of all and one of the stack thrown, the ladder at range (dice by n, the stun, the sure critical at five at 60), a carried pool adopting a foe across the room, an unseen creature in the line taking a plain throw with the openings spent and the mark unmoved, a kill at range, the action text's round trip. `world/feint_tests.rs`: a feint made builds one and a feint at a new foe empties first. `world/class_rogue_tests.rs`: the table (Exploit at 1, no Trained wind, no pool until a rank grants it), Patient blade's two. `content` tests: the new `Build`s parse, the `exploit_*` effects parse and refuse a bad die. UI: the screen snapshots `abilities` (Exploit's and Thrown exploit's rows), the thrown exploit's item list, `character` (the mark), `look` ("your mark"); `ui/mod.rs` the status bar unchanged |

## 12. Implementation plan: the phases

One PR each, each green on its own, each with its spec, dev-doc, and wiki changes.

| Phase | Builds | Touches | Depends on |
|---|---|---|---|
| **P1 The mark and the kill that carries** (§3, §4.3, §8 Patient blade, §9's mark words) | `Actor::mark`, `world/mark.rs` (`aim_at`, `free_blow_at`, the clears), the hooks in `fight.rs`, `shoot.rs`, `cast.rs`, `free_blow.rs`, `travel`, `live`; `Build::MarkKill` replacing `SneakKill` (Patient blade re-pointed, Vanishing act's hook moved); the lost-openings message; **Low blow and the finisher gain the mark refusal** ("Your openings are in the kobold's guard.") so the rule is whole before Exploit exists; the character screen, look mode, the dumps; `docs/dev/openings.md`; the wiki's Openings section | `actor.rs`, `fight.rs`, `ambush.rs`, `thief.rs`, `openings.rs`, `content/resource.rs`, `patient_blade.toml`, `perception.rs`, `dump.rs`, `ui/character_screen.rs`, `ui/look.rs` | — |
| **P2 Feint builds** (§4.2) | `Build::Feint`, `builds` on Openings, `aim_at` in `feint.rs`; tests; the wiki's Feint and Openings lines | `feint.rs`, `resources.toml`, `content/resource.rs` | P1 |
| **P3 Exploit at level 1, Trained wind out, Low blow reshaped** (§5, §6's rows 1 and 12, §7) | `ClassActive::Exploit`, the talent `exploit`, `World::exploit` in `openings.rs` (the refusals, the spend of all, `Technique::sneak`, the dice by n, the words), `TalentEffect::{ExploitDice, ExploitToHit, ExploitStatus}` and `ExploitMods`; Low blow's data becomes the stun and the to-hit, `ClassActive::LowBlow` removed; `trained_wind` off the thief's table; the abilities menu row; the screen snapshots; the balance paragraph | `08-thief.toml`, `exploit.toml` (new), `low_blow.toml`, `content/class.rs`, `content/talent.rs`, `talent.rs`, `ability.rs`, `class_ability.rs`, `openings.rs`, `thief.rs`, `README.toml` | P2 |
| **P4 The ladder's top, the finisher folded in, the probe** (§6's rows 32 and 60, §10) | `TalentEffect::ExploitThreat { value, sure_at }`, Improved sneak attack's second die, Master thief's threat and `sure_critical` at five, `ClassActive::MasterThief` removed; `balance_probe/tables.rs::exploits` and `docs/balance.md`; the wiki's ladder table in `xtask/src/wiki/resources.rs` | `improved_sneak_attack.toml`, `master_thief.toml`, `openings.rs`, `balance_probe/`, `xtask/src/wiki/` | P3 |
| **P5 The thrown exploit at 18** (§5.1, §7's row 18) | `ClassActive::ThrownExploit`, the talent `thrown_exploit` on the thief's table at 18, `Aim::Throw` (the throw list less flasks, then the aiming mode at the capped reach), `World::thrown_exploit` in `openings.rs` (Exploit's refusals plus the reach and the seen line, the spend of all, the throw through `shoot.rs`, `land_exploit` on the missile's hit), the action text `use:thrown_exploit:LETTER:X,Y`; the abilities menu row and the item list's snapshot; `world/thrown_exploit_tests.rs`; the probe's thrown column; the wiki's Thrown exploit section | `08-thief.toml`, `thrown_exploit.toml` (new), `ability.rs`, `class_ability.rs`, `openings.rs`, `shoot.rs`, `action.rs` (the parser), `ui/target.rs`, `ui/` (the abilities menu's aim), `balance_probe/`, `xtask/src/wiki/` | P4 |

P1 is the foundation and the only phase that changes a thief's play without adding to it; it ships first so the rule is seen in the wild (the integrator's call whether P2 rides with it: it is one build and one hook). The phases run in order, each from `main` after the one before merges: P2 to P5 all touch `openings.rs` and the Openings data, so a chain costs less than the conflicts. P5 comes last because its hit is `land_exploit` with every increment in it; it needs only P3 in principle and comes after P4 so that its tests cover the whole ladder at range.

## 13. Owner's answers (2026-11-13)

The owner approved the design with these answers to the draft's questions; the sections they touch are updated.

| # | Question | The owner's answer | Where |
|---|---|---|---|
| Q1 | Exploit spends every opening held, or a prompt for how many? | **Every opening held**, no count prompt | §5 |
| Q2 | Low blow's increment at 12: Stunned and +1 to hit per opening, or Sluggish a degree per opening? | **As recommended**: Stunned + 1 to hit per opening | §6 |
| Q3 | The mark's death by another hand: nothing, or the opening too? | **No bonus opening, but the pool still carries** to the next target, as after the thief's own kill less the +1 | §3.2, §4.3 |
| Q4 | Exploit at a target with no vitals: refused, or a plain blow with the openings spent? | **Refused**: no time, nothing spent | §5, §5.1 |
| Q5 | The thief changes level: mark cleared and pool kept, or the mark kept? | **As recommended**: the mark cleared, the pool kept | §3.2 |
| Q6 | Trained wind off the thief's table, Exploit in its slot, nothing else? | **Yes**, nothing else in its place: "the thief already gets Exploit at level 1" | §7 |
| Q7 | The name: "Exploit" or "Seize the opening"? | **Exploit** | §5 |
| Q8 | A thrown exploit? | **Yes**: a separate ability granted at a thief level, the throw list then the aiming mode, resolved as Exploit with the thrown item's attack at range, for a mark out of melee reach (the carried pool after a death, or a mark that blinked or walked away); Exploit's refusals plus throwing's range and line. Its level, name, action text, increments, and cost were left to the design: **Thrown exploit**, `thrown_exploit`, level 18, `use:thrown_exploit:c:12,5`, every opening held and a throw's price, the ladder unchanged | §5.1, §7, §12 P5 |
| Q9 | No cap on n beyond the pool's maximum? | **No cap** beyond the pool's maximum | §5, §8 |
| Q10 | The ladder's unit: dice or a flat bonus? | **Dice** | §6 |
| Q11 | Phasing? | **The design's call**: the phases kept, the thrown exploit added as P5 | §12 |

## 14. Choices made along the way

- **Chosen attacks only move the mark** (§3.1). The alternative, every attack including free blows, made Opportunist and Riposte empty a pool the player had built on someone else, for a blow they did not choose. A free blow may set a mark when none stands so the lurk opener builds.
- **A mark out of sight is still the mark**; only the decay forgets. Clearing on lost sight would punish a thief who steps back around a corner to let a foe come to them, which is how thieves fight.
- **The level change clears the mark but keeps the pool** rather than emptying it: emptying would punish taking the stairs mid-chase, and keeping the mark would surprise the player on the next floor.
- **One blow, not three.** The brief's "escalating progression" could have been three actives at three costs (as today's 3 and 5); then the level-12 and level-60 thief would have more buttons, not a better one, and the level-1 thief none. One blow that the talents grow is the brief's fifth rule read literally. The thrown exploit (the owner's answer to Q8) is the same blow delivered another way, not a fourth rung: every increment applies to it through the one `land_exploit`.
- **The thrown exploit's reach is capped at the sneak dice's** (6 cells) rather than the item's full throw range: an exploit is a sneak attack whatever the foe knows, and past that reach the dice would not land.
- **The thrown exploit's line is judged by what the player sees**: a seen wall or creature in the way refuses it; an unseen one takes a plain throw and the openings are spent. Refusing on the unseen would reveal it.
- **The finisher is kept exactly at five** so nothing the level-60 thief had is lost; it is also reachable at a threat below five, which it never was.
- **Stamina is not Exploit's price.** The openings are the price and the dial; stamina would make a thief without a rank-7 style unable to use their level-1 ability, which is why Trained wind existed at all and why it goes.
- **No message on a build.** The bar ticking is the feedback; a line a turn would drown the fight's own words. The lost-openings line is the one that prevents a "where did they go?".
- **The mark is the player's own knowledge**, so showing it in look mode and the character screen leaks nothing; `SeenActor::marked` is computed from the player's actor, never from a creature's state, as Stalker's eye's `unaware` is.

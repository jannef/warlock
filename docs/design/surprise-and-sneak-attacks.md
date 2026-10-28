# Warlock — Surprise and sneak attacks: the Sneak attack and Initiative skills, the rogue's class fantasy

Design document for the owner's review (2026-10-28), revised with the owner's decisions the same day (§14). Written against `origin/main` at 0c7d7c0e, then against 96f59672 (Detect traps split from Spot, #673; Negotiation split from Appraise, #676; optional starting skills for classes, #671; Knowledge, #674). Nothing here is implemented; §12 cuts it into issues. It builds on what is there: the thief's sneak attack and Ambusher (`world/ambush.rs`, `docs/dev/progression.md`), creature sight and the AI's mind (`docs/dev/creatures.md`, `sight.rs`), the energy scheduler and action costs (`docs/dev/time.md`), statuses that ease by actions (`docs/dev/status-effects.md`, `docs/dev/weapon-abilities.md`), the 2d12 rules and Defence categories (`docs/dev/combat.md`), skills and their marks (`docs/dev/skills.md`), checks (`check.rs`), the sleepers of teeming dens (`docs/dev/special-rooms.md`), weapon abilities and their triggers (`docs/dev/weapon-abilities.md`), resources (`docs/dev/resources.md`), and the renderer's creature layer (`docs/dev/rendering.md`). Every number is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-10-28, verbatim): *"feature: surprise and sneak attacks. we need a system for rogue-types to have some way express their class fantasy. this should include at least a new skill "sneak attack". We can look into dnd 3.5 for inspiration on the effects of a suprise. Perhaps the first action after getting surprised is also more expensive. This might tie also into concept of initiative, which might be a skill the reduce effects of surprise on oneself"*

What is taken from D&D 3.5, because the owner named it or it follows directly: being **surprised** (the surprise round, in which the unaware do not act) and being **flat-footed** (no Dexterity bonus to Defence, no attacks of opportunity, until one's first action); the **sneak attack** as precision dice against a target denied its Dexterity bonus, which never land on what is immune to criticals; **initiative** as what decides who is caught out, and how badly. What is not taken, with the reason, is in §11: flanking (decision 6), attacks of opportunity as such, a whole lost action, an initiative *order*. A hiding contest was rejected in the first draft and is **brought back in its smallest form** (§3.4, decision 15) because the audit of creature eyes shows the melee sneak attack does not happen without one.

## Contents

1. Summary
2. The owner's request, and what exists today
3. Surprise: the model (3.4: getting there, and the smallest contest)
4. The Sneak attack skill
5. The Initiative skill
6. Class fantasy: classes, the thief's table reworked, talents, weapon abilities
7. Creatures
8. What the player sees (8.1: the unaware mark)
9. Determinism, saves, replays
10. The wiki, the spec, and the docs
11. Choices made along the way, and what is rejected
12. Implementation plan
13. Open questions, each with a recommended default (answered: §14)
14. Decisions (owner, 2026-10-28)

## 1. Summary

| Piece | Proposal |
|---|---|
| **Unaware** (exists, widened) | an actor is unaware of another when it has not seen it: for a creature, today's rule (`World::unaware`: it hunts no one, `Mind::last_seen` unset, and its eyes miss the attacker's cell now; a sleeper and a disguised mimic always), **plus one the player has slipped past (§3.4)**; **for the player, new and symmetrical**: the attacker is not among the creatures their last perception saw, and they have not seen it within the last `memory_turns` (20) turns (`Mind::noticed`, the turn the player last saw that creature, set by `perceive`). The check is asked of **every** attack now, not only of an attacker with ambush dice. §3.1 |
| **Surprise** | an attack (a blow, a shot, a throw, a spell that rolls to hit) that finds its target unaware **surprises** it, and so does a hidden trap springing under it (the one it springs on, nobody beside: decision 2). A surprised actor gets one **initiative check** (Dexterity, plus the Initiative skill, against 13 plus the attacker's Sneak attack craft; none for a sleeper or a disguised mimic): failed, it suffers the status **Surprised** (`per_action`, until its next action): **flat-footed** (Dexterity and Dodge count nothing toward its Defence), **no free blows** of its own (riposte, opportunist, gatekeeper, Point-blank), and **its next action costs 150 %** (decision 1; the status's `cost`, as Staggered's), **less what Initiative takes off** (decision 4: the skill shrinks the extra cost, to half of it at 100, three quarters at 200). Every attack on it while it lasts is a sneak attack for an attacker with the dice. The attack that surprised it is already rolled against the flat-footed Defence. The player and creatures alike. §3 |
| **Slipping past** (new, §3.4) | the smallest hiding contest, because 238 of the 294 creature types see five cells in any darkness and would notice a thief beside them whatever the light: a creature that is hunting no one and would see the player **by darkvision alone** (their cell below its eyes' light threshold) rolls, once per stalk, the player's Dexterity + Stealth against its Willpower + Spot (`check::contest`, four gameplay draws); made, it does not notice them until they step into its light, attack, or it is hurt (`Mind::missed`). Nothing else about creature sight changes; nothing draws when the player is lit, seen, or already hunted |
| **Sneak attack**, the skill | `sneak_attack` (`kind = "rules"`): its value buys **d6s added after Protection** to a hit on a target that is unaware or Surprised: 1d6 trained at all, +1d6 at 25, 50, 75, and 100, then +1d6 every 50 (5d6 at 100, 7d6 at 200); and a second benefit, **`surprise`** (+8 at 100, +12 at 200), added to the difficulty of the target's initiative check. Blows at any reach, shots and throws and attack spells **within 6 cells** (`missile_reach`, decision 3). No weapon restriction (the daggers get their hook as a weapon ability). **No vitals** = **immune to criticals** (decision 5): one flag, `vitals = false` on a type, with the incorporeal, the animated, the elementals, the bones and the spirits by rule; such a target is never critically hit and takes no sneak dice; still surprised. The talents' ambush dice (Ambusher, Improved sneak attack) add to the skill's, as today. Marks on a sneak attack that hits (50 %); a Dexterity training mark too. §4 |
| **Initiative**, the skill | `initiative` (`kind = "passive"`): +8 at 100, +12 at 200 on the initiative check, **and** the dear action's extra cost cut by half at 100, three quarters at 200 (`surprise_ease`, decision 4). Marks on every check made. **No initiative order** (§11). §5 |
| Classes | **only the thief** trains Sneak attack, guaranteed (`2d6+10`, known for it); **the drow** may start with it by chance (40 %, `2d4`: `optional_skills` on races, the field #671 gave classes; decision 7); nobody else. Initiative: the thief `2d4+4`, the ranger and the monk `2d6+10`, the bard, warrior, and barbarian `2d4+4`. §6.1 |
| The thief's table reworked (decision 10) | the three class abilities that only replicated a trained skill go: **Trapfinding** (Detect traps and Disable device, trained from level 1 and known for), **Pick lock** (Open lock, trained and known for since always), **Light fingers** (Steal, likewise). Their slots take identity abilities: 6 **Lurk** (`use:lurk`), 12 **Low blow** (`use:low_blow:E`, 3 openings), 40 **Vanishing act** (a sneak attack that kills leaves the thief Light-footed). Level 1 **Sneak attack** keeps its id and becomes the class passive that goes with the skill (Openings from level 1; those caught unawares find it 2 harder to recover); Trained wind, Evasion, Uncanny dodge, Improved sneak attack, Hide in plain sight, Master thief stay. §6.2 |
| Talents | the `awareness` system comes off `pending` with **Wary** (Delver: +4 on the check), **Cool head** (Grit: Surprised slows no action of yours), **Stalker's eye** (Shadow: the unaware mark, §8.1, and look mode's word), **Hair trigger** (Finesse: a check made against a foe beside you is answered with a blow at once); a new system **`sneak_attack`** with **Anatomist** (Scholar: the dice land on bony things too), **Long reach** (Weaponcraft: shots and throws sneak-attack 3 cells farther), and the retagged Ambusher, Improved sneak attack, Master thief, Patient blade. §6.3 |
| Weapon abilities | one new trigger, **`sneak_attack`** (raised where the dice land): **Knife in the dark** (Daggers 5: a sneak attack with a dagger leaves the foe bleeding) and **Hunter's shot** (Marksman 6: a sneak attack with a shot pins the foe a turn or two). §6.4 |
| Creatures | equal treatment through data: a type's `skills = { sneak_attack = N, initiative = M }` gives it the dice and the check as the player's do (eight types); one AI hook, **`lurks = true`** (the meenlock, the barrow spider). Creatures slip past nobody (the contest is the player's Stealth; a creature's approach is its speed and the dark, as today). §7 |
| **The unaware mark** (decision 11) | a character with **Stalker's eye** sees, on each visible creature that has not noticed them, a gentle tint of its glyph toward slate with a slow breathing; under reduced effects the tint stands still. Drawn by `ui/glyphs.rs` on the free-glyph layer from `SeenActor::unaware`, which the snapshot carries only for such a character; no shader change; no golden (a `--capture` looked at, and the tint a unit-tested CPU function). §8.1 |
| Messages | told after the fact, never before: "You catch the kobold unawares." / "The kobold whirls to face you." (the check made) before the blow's own line; "You are taken by surprise!" / "You keep your wits about you." for the player; look mode "(hostile, unhurt, surprised)" while the status lasts and "asleep" as today; "unaware of you" only with Stalker's eye. No number, no roll, no creature's mind otherwise. §8 |
| Determinism | the initiative check is two draws of the **gameplay** stream, made only when an attack finds its target unaware or a hidden trap springs on someone awake, before the strike's draws; the slip contest four draws, only when a creature would first see an unlit player by darkvision; the dice draw as today; marks on the **training** stream. Saved state: the statuses (as any), `Mind::noticed: Option<u64>` and `Mind::missed: Option<u64>` on creatures (left out while `None`). Old saves and replays load; replays whose first attack finds a creature unaware, or whose thief walks unlit up to a creature, hash differently from that step. §9 |
| Slices | nine issues in four waves: S1 surprise (rules, state, the first talents), S2 the two skills, class data, and crit immunity, S8 slipping past (state), then S3 the thief's rework with its actives (actions), S4 the weapon abilities, S5 the creatures, then S6 the unaware mark and Hair trigger, S7 the wiki page and the balance row. At most two at a time add saved state, one adds actions. §12 |

**Key choices, and why**

1. *Surprise is a status, and the first action after it is dear.* The game already has statuses that ease by their sufferer's actions and make that action dearer (Staggered: `per_action`, `cost = 150`): Surprised is one, so the owner's "first action more expensive" costs no new mechanism, the scheduler is untouched, and creatures suffer it exactly as the player does (`World::burdened` prices everyone's actions). Initiative softens that price at the one place it is applied (decision 4).
2. *Flat-footed rides Feint's code.* Feint already rolls an attack as if the foe had no Parry or Shield (`Technique::ignores`, `World::feinted`); a surprised target is rolled as if it had no Dexterity and no Dodge, through the same `Target::without(categories)`. One function, two callers.
3. *The player is surprised by the same rule as a creature.* "Has not seen the attacker" is the creature's `Mind::last_seen` plus its eyes; for the player it is their perception snapshot plus a short memory (`Mind::noticed` on the creature: the turn the player last saw it). Nothing is new in principle; the asymmetry today (creatures never surprise the player) was only that nobody wrote the player's half.
4. *Sneak attack is a skill, so it grows by the skill ladder, and the class talent stays a passive.* The thief's level-1 talent keeps its id (saves name it) and loses its dice to the skill; the skill's ladder is where the dice live, and anyone who trains it (the thief, a lucky drow, a creature) has them. Ambusher and Improved sneak attack keep adding dice on top, as today's `Dice::and` already sums them.
5. *Initiative reduces surprise; it does not order the fight.* The owner's phrase is "reduce effects of surprise on oneself", read twice over (decision 4): the check that may shrug the status off, and the benefit that shrinks the dear action when it does not. The energy scheduler is exact and hidden, and a second order on top of it would be either redundant or a leak.
6. *One contest, as small as it can be, where the eyes make it necessary.* §3.4 shows that without it the melee sneak attack lands only on sleepers, mimics, the six creature types without darkvision, and by luck of speed. The contest is one roll per stalk, rolled only when a creature would first see an unlit player by darkvision, stored in its mind so the AI stays a pure function of state, and over the moment the player is lit, attacks, or hurts it.
7. *The thief's abilities are what no skill does.* Three of the ten replicated a skill the class already trains (decision 10); the skills take their place in the class data, and the slots take Lurk, Low blow, and Vanishing act, which are the thief's and nobody's skill.
8. *Creatures get it through data and one AI hook.* `skills` on a creature type exists and is read by the same `Actor::skill_effect`; the only new behaviour, lurking, is one deterministic branch in `ai.rs::fight`.

## 2. The owner's request, and what exists today

| Part | Today |
|---|---|
| The sneak attack | `world/ambush.rs`: `World::sneaking(attacker, target)` is true when the attacker holds a talent with the `ambush` effect (the thief's Sneak attack 1d6 at level 1, Improved sneak attack +2d6 at 32, Ambusher 1d6 in Shadow at Stealth 30 and level 9; `Dice::and` sums dice of one size) **and** `World::unaware(target, attacker)`. `fight.rs::resolve` asks it before the strike (no draw), lowers the threat for Master thief (`sneak_threat`, −2, never below 3), and `ambushed` rolls the dice on a hit, **after Protection** (never soaked, never multiplied by a critical), from the gameplay stream, then raises `Build::SneakAttack` (the Openings pool, R3) and `resolve` raises `sneak_kill` for Patient blade. Shots, throws, and blows alike; spells never (`profile` is `None` for an intent, and a spell that rolls to hit is resolved in `cast.rs::cast_single`, which never asks `sneaking`) |
| Unaware | `World::unaware(id, other)`: **creatures only**; true for a sleeper (`den.rs`, the Asleep status that `sleeps`) and a disguised mimic (`mimic.rs`) whatever they see; else when `Mind::last_seen` is unset (it hunts no one) and `World::sees_now(id, other)` is false (creature sight, `sight.rs`: radius, line of sight, light enough for its eyes or within its darkvision). **Never for the player**: a creature coming out of the dark gets nothing on them |
| Creature eyes | `vision` profiles: normal (threshold 40 of 255), low-light (12), **darkvision (40, and everything within 5 cells whatever the light)**, UV. Of the 294 creature types, **238 have darkvision**, 25 low-light, 6 normal (the deer, the mud golem, the dark wizard, the necromancer, the shopkeeper, the player); `sight` 10 by default. A creature's sight is a pure function of light and lines, kept equal to the player's perception by a property test, and the AI draws nothing |
| The creature's mind | `Mind { last_seen, noise, post }` on `Actor` (saved, hashed): `ai.rs::decide` sets `last_seen` the moment `nearest_hostile_in_sight` sees a hostile (deterministic, no draw), clears it when the search is given up (`search_or_return` walks back to the post and the mind goes calm). A noise sets `noise` only: a creature sent to look by a thrown voice (`voice.rs`) stays unaware |
| Awareness and the player | the perception snapshot's `actors()` (what the player sees now) and `World::seen_creature_at` (talk, steal); nothing remembers which creatures the player has seen |
| Sleep | Asleep (`sleeps = true`): no turn, unaware of everyone, woken by noise at its threshold × the den's `wake`, by harm, by a neighbour's cry; "The goblin wakes up."; look mode "asleep" |
| The scheduler | `schedule.rs::run_turn`: energy per actor, the player's price first, creatures in time order; `World::burdened` → `slowed` is the one place every price passes, and `Effects::slow` multiplies in each status's `cost` (Staggered 150, Sluggish 150, Drowsy 300, Transfixed 200). `per_action` statuses (Staggered, Pinned, Light-footed) ease a degree per action of their sufferer (`World::spend_actions`, called where each action's price is paid; a dose given during the sufferer's own action is not eased by it: `World::acting`, `World::fresh`) |
| Defence | `combat::Category`: Dexterity, Armour, Shield, Parry, Natural, Dodge, Deflection, Wall; `Defence` keeps the highest per category (Dodge stacks). `World::target_against(id, missile)` builds the `Target` (Deflect missiles' Dodge, Measure's Parry); `World::feinted` strips the categories a Feint ignores (positive bonuses only) |
| Criticals | a natural roll in the threat range, confirmed by a second roll (`combat::strike`); `Profile::sure_critical` (Vital strike, Master thief's finisher). **No creature is immune to criticals today**: no data field, no hook, no word in the spec (decision 5 adds it, §4) |
| Free blows | `World::second_blow` on a `Cue` (`fight.rs`); `World::free_blows_of(actor)` folds the actor's switches, its talents', and the weapon abilities' (`ability_switches`); the out-of-turn ones are riposte, opportunist (`step_away`), gatekeeper (`step_up`), and Point-blank's free shot (`approached`) |
| Checks | `check::check(rng, modifier, difficulty)`: 2d12 + a stat modifier, natural 2 fails, 24 succeeds, two gameplay draws; `check::ways` the odds; `contest` four draws (stealing: the thief's Dexterity against the victim's Willpower) |
| Skills | `data/skills.toml`: trained skills only (a missing one gives nothing); `benefit` by `steps`/`every` or a `curve`; `mark` by `Use` with a chance (training stream, player only); each effect read in one place through `Actor::skill_effect`; creature types may list `skills` with fixed values (114 of the types do, nearly all Concentration and Spellcraft); "trained at all" is `Skills::trains_effect` (Spot, Detect traps, Steal, Heal, Pick lock, Disarm). **Optional starting skills** (#671): a class's `optional_skills = { listen = { chance = 30, dice = "2d4" } }`, each rolled once at creation from `character/optional-skills/<spec>`; `ClassDef::optional_skills: Vec<OptionalSkill>`, `character.rs::optional_skills`; **races have no such field** |
| The thief | class skills Open lock, Steal, Stealth, Tumble (known for), Appraise, Detect traps, Negotiation, Disable device, Escape artist, Search, Spot; optional Balance, Bluff, Listen; Daggers 2, Thrown 1; a dagger style; the table: 1 Sneak attack + Trained wind, 6 Trapfinding (`searching +4`, `trap_ease 2`, the `disarm` ability), 12 Pick lock (the `pick_lock` ability), 18 Evasion, 25 Uncanny dodge, 32 Improved sneak attack (grants Openings), 40 Light fingers (the `steal` ability), 50 Hide in plain sight, 60 Master thief (threat −2 and the finisher, `use:master_thief:E`, 5 openings); bonus picks from Shadow, Finesse, Delver at 4, 10, 16, 22, 28, 34, 46, 58. **Three of the ten only unlock what a trained skill unlocks** (`Ability::had_by`: `trains_effect(Disarm) || talents.disarm`, and the same for `Pick` and `Steal`), and the thief trains all three skills from level 1 |
| The other rogue-types | the ranger (Favoured prey, Tracker, Camouflage, Rapid shot and Volley on Focus; Bows 2, Short blades 1, Marksman); the bard (Lore, Inspire, Throw your voice; Short blades 1, Daggers 1d2, Duelist); the monk (Flurry, Still mind, Quivering palm). No assassin class, and none is added here |
| Creatures that ought to ambush | the drow assassin ("the houses' knife in the dark ... a sure eye for the opening that kills"), the drow stalker ("opens a throat with a dagger"), the quickling ("cutting with a little knife before it is seen"), the meenlock ("holds it with paralysing claws"): their descriptions promise what no rule gives them; speed and venom are all they have |
| Talents | the `awareness` system ("Creature sight and awareness") is in `talents.toml` **`pending`**, with Ambusher alone in it; `stealth` holds Ambusher, Hush-foot, Silent feet, Camouflage, Hide in plain sight and the rest; the Shadow tree is Stealth, hearing, eyes, and light; the coverage bar is 2 talents a system |
| Traps | `trap.rs::step_onto_trap` for the player's and creatures' steps; a hidden trap springs with no warning; nothing of it stays on the victim but the harm |
| The renderer | creatures are free glyphs (`render/free_glyph.rs`), coloured and lit on the CPU in `ui/glyphs.rs` (`glyph(...)`, `palette(fg)`, `VisualLight::sample` + `lighting::shade`), banded for composites with a slow sway that is 0 under reduced effects; `ui/` reads the perception snapshot only; `Settings::effective_effects(force)` and `--reduced-effects` turn every animation off |

**The gap**, in one line each: creatures never surprise the player; surprise is not a state, so a creature caught unawares fights back at full Defence and full speed from its next action; the sneak attack is three flat talents, not a skill that grows; no class but the thief has any of it; no creature has any of it; nothing about surprise is told, so the player cannot tell a lucky big roll from an ambush that worked; and, with 238 types seeing five cells in the dark, nobody awake is ever caught unawares in melee by anyone slower than a quickling.

## 3. Surprise: the model

### 3.1 Unaware

`World::unaware(target, attacker)` becomes the one definition for everyone and is asked of **every** attack (today only of an attacker with ambush dice; it draws nothing, so the wider ask changes no hash):

| Target | Unaware of the attacker when |
|---|---|
| A creature | as today: asleep or disguised, whatever it sees; else it hunts no one (`Mind::last_seen` unset) **and** its eyes do not see the attacker's cell now (`World::sees_now`), **or** the attacker is the player and it has been slipped past (`Mind::missed` set, §3.4: it does not see them though its darkvision would). A creature that has lost sight of the player but is still searching (`last_seen` set, until it gives up and its mind goes calm) is **aware**: it knows something is about, and D&D's "aware but cannot see" is not surprise either |
| The player | the attacker is not among the actors of the player's **current perception** (the snapshot of their last turn's end, `World::perception`), **and** `Mind::noticed` on the attacker is `None` or more than `memory_turns` (20) turns ago (decision 8). `World::perceive` sets `noticed = Some(state.turn)` on every creature the snapshot sees (in the turn pipeline only, never on `from_state`'s recomputation, which must not touch state). A fast creature that walks into view and strikes in the same creature turn, a shot out of the dark from something never seen, a sleeper the player walks past that wakes and bites: surprises. A creature seen a few turns ago that steps behind a pillar and out again: not |

### 3.2 What happens

In `fight.rs::resolve`, before the strike (the order of draws is fixed, §9):

1. `unaware = self.unaware(target, attacker)`. A spell that rolls to hit (`cast.rs::cast_single`, an own cast; a bolt, a ray, a touch) asks it too, through the same function, so a wizard's bolt out of the dark surprises as an arrow does.
2. If unaware: **`World::startle(target, by, craft)`** (`world/surprise.rs`), the one place anyone is surprised:
   - a sleeper wakes (as harm wakes it today) and a disguised mimic is revealed (as today) and **both are Surprised with no check**: they could not react;
   - else the **initiative check**: `check::check(rng, modifier, difficulty)`, two gameplay draws, with `modifier` = the target's Dexterity modifier (effective score) + its Initiative benefit (`Actor::skill_effect(Effect::Initiative)`) + its talents' `initiative_bonus` (Wary), and `difficulty` = `SurpriseRules::difficulty` (13) + `craft`, the attacker's Sneak attack `surprise` benefit (0 for one untrained) + the attacker's talents' `surprise_difficulty` (the thief's class passive, +2). Made: not surprised (a message, §8); failed: **`afflict(target, surprised, 1, by)`**.
3. The strike is rolled against `target_against(target, missile)` **with Dexterity and Dodge stripped** when the target is unaware or Surprised (`Target::without(&[Dexterity, Dodge])`, the function Feint's strip becomes; positive bonuses only, as Feint). So the surprising attack itself lands on a flat-footed target, as D&D's first attack in the surprise round does, and so does every attack while the status lasts.
4. On a hit, the sneak attack dice (§4), raised for an attacker with dice when the target was unaware **or Surprised** (today: unaware only; Surprised is the new second door, which is how a trap, a companion of a faster creature's ambush, or the thief's second blow in the same turn as its first all count).

**The status `surprised`** (`data/status.toml`, placed by the weapon abilities' statuses): `name = "Surprised"`, `seen = "surprised"`, `max = 1`, `stacking = "refresh"`, `per_action = true`, `cost = 150`, `start = "You are taken by surprise!"`, `end = "You recover your wits."` (a `note`: it does not stop a rest; nothing rests while Surprised anyway). Its readers, one each:

| Effect of being Surprised | The one hook |
|---|---|
| Flat-footed: Dexterity and Dodge count nothing toward Defence | `World::target_against`: `Target::without` when the target is Surprised (`Effects::has(surprised)`) or `unaware` (passed in by `resolve`) |
| No free blows: riposte, opportunist, gatekeeper, Point-blank's free shot | `World::free_blows_of` returns none while the actor is Surprised; `World::approached` skips a Surprised actor |
| The next action costs 150 %, **less what Initiative and Cool head take off the extra** (decision 4) | the status's `cost`, read by `Effects::slow` in `World::burdened` as every status's is, **through `World::surprise_cost(id)`**: `100 + (cost − 100) × (100 − ease) / 100`, `ease` = the sufferer's `surprise_ease` benefit (Initiative: 50 at 100, 75 at 200) + the talents' (Cool head: 100), capped at 100. At Initiative 100 the dear action is 125 %; with Cool head, 100 %. The one status whose `cost` is eased this way; the hook is one match arm beside the generic read |
| Every attack on it is a sneak attack for an attacker with dice | `resolve` step 4 |
| It ends with the sufferer's next action | `per_action`, `World::spend_actions` (no code) |

**How the two halves of Initiative fit** (decision 4): the check is the chance to shrug the surprise off entirely (no status: full Defence from the next attack on, free blows kept, no dear action); the `surprise_ease` benefit is what softens the dear action when the check fails. The surprising attack's own roll and dice are never touched by either: the craft is the attacker's. A master of Initiative (200) makes the check 143 times in 144 against an untrained attacker (only a natural 2 fails) and, failing it, pays 112 %.

**How long, and stacking.** Until the surprised actor's next action, which is the dear one; a second surprise before it acts refreshes the one degree (one dear action however many foes fell on it); Staggered and Surprised multiply as any two slowing statuses do (rare, and the stagger was earned). A creature of speed 2000 pays its dear action and is itself again on the second of its two turns.

**Traps as a source** (decision 2): `trap.rs::step_onto_trap` (the player's and creatures' steps alike) calls `startle(victim, None, 0)` when a **hidden** trap springs under its victim (a known trap stepped onto on purpose surprises nobody): the check against 13 with no craft, then the status. **Only the one it springs on**: a burst trap's bystanders, the one who watches a pit open under a goblin, are not surprised (they are hurt or not by the trap's own rules; the surprise is the stepper's). The sense of it: a pit opening under a thief with a drow assassin beside them is the assassin's opening; and a goblin that springs a trap beside the player is the player's.

**Symmetry.** Nothing above asks who is the player. A creature with Sneak attack in its `skills` (§7) rolls its dice on a surprised player; a creature caught by a trap is flat-footed to the player's blow; the player's Initiative skill and a creature's are read by the same line.

### 3.3 Data: `data/surprise.toml`

```toml
# Surprise (spec §8 "Surprise and sneak attacks"). Optional: without it nobody is surprised, no
# sneak attack dice land, and nobody slips past anyone (fixtures).
[surprise]
status = "surprised"      # the status an actor caught unawares suffers (status.toml)
difficulty = 13           # the initiative check's difficulty before the attacker's craft
missile_reach = 6         # how far a shot, a throw, or an attack spell is still a sneak attack
memory_turns = 20         # how many turns the player stays aware of a creature they have seen
# What has no vitals to find (decision 5: the same as being immune to criticals): no critical
# is ever confirmed on, and no sneak attack dice or lowered threat land on, an incorporeal
# creature, an animated object, a type carrying one of these tags, or one that says
# `vitals = false`; it is surprised like anyone.
no_vitals = ["elemental", "skeleton", "spirit", "construct"]

# Slipping past (§3.4): the player's Dexterity plus Stealth against a creature's Willpower plus
# Spot, once per stalk, when the creature would first see them by darkvision alone.
[slip]
enabled = true
```

`content/surprise.rs`: `SurpriseRules`, `Content::surprise()`; the status checked to be `per_action` with `max = 1`; the tags checked to exist on some type (as a quest's tag is). A creature type gains `vitals` (default true; `CreatureDef::vitals`), and `Content::has_vitals(type)` is the one reader of the three rules; `combat::Target::no_criticals` is set from it by `World::target_of` (§4).

### 3.4 Getting there: can anyone be sneak-attacked without a contest? (decision 15)

The owner's question, answered first by an audit of what the rules of sight allow **today**, then by the smallest rule that fills the gap.

**The rule of creature sight.** A creature sees the player's cell when it is in line of sight within its `sight` (10 by default), and either lit at least to its eyes' threshold (normal 40 of 255; low-light 12) **or within its darkvision** (5 cells, whatever the light). It notices at the start of its own turn (`decide`), and from then on it hunts (`last_seen` set): aware until it gives up the search. There is no facing, no cover but line of sight, and no roll.

**Who sees how.** 238 of 294 creature types have darkvision (every kobold, goblin, orc, drow, duergar, undead, fiend, troll, most fey); 25 have low-light eyes (the quickling, the deer-like wildlife of the courts, a few fey); 6 have normal eyes (the deer, the mud golem, the dark wizard, the necromancer, the shopkeeper, the player).

**Where a sneak attack lands with no contest at all** (today's `unaware`, widened to every attacker by §3.1):

| Situation | Melee | Shot, throw, or attack spell (within 6) |
|---|---|---|
| A sleeper (a teeming den's; every den has a dozen) | yes, every blow until it wakes | yes |
| A disguised mimic | yes (the first blow) | yes |
| A creature with normal or low-light eyes, the player unlit (lantern out or hooded, no fire, no sky) | yes: it cannot see the cell beside it | yes |
| A darkvision creature, the player **beyond 5 cells** in the dark and hunting no one | no (not beside it) | **yes**: the thief's thrown dagger or the ranger's arrow from the dark, six cells out, is unseen |
| A darkvision creature looking into a thrown voice, hunting no one | no: once the thief stands within 5 cells, it sees them | yes, from outside the five |
| A darkvision creature beside the player | **no**, unless the player acts twice before it acts once (speed: Quick, Fleet, Hasted; a thief at 1000 against a kobold at 900 gets one extra action in ten) | — |
| Any creature Surprised by a hidden trap it sprang | yes, that one dear action | yes |
| Low blow (3 openings) | yes, by fiat, no Surprised | — |

**A worked example, today's rules.** A human thief, lantern hooded, in a dark corridor of Cealdhlāw; a kobold (darkvision 5, sight 10, hearing 15, patrolling, hunting no one) comes up the corridor. At 7 cells the kobold does not see the thief (dark, beyond its darkvision). The thief throws a dagger: a sneak attack (1d6 at level 1, 2d6 at Sneak attack 25), the kobold is Surprised or whirls to face them; either way it now hunts them. Had the thief instead waited for a melee blow: at 5 cells the kobold's darkvision reaches the thief's cell; on its next turn it sees them, hunts, and closes in; the thief's dagger meets an aware kobold at full Defence. The thief's footsteps (60, less 10 a cell) reach the kobold's hearing (15) at four cells on stone, so creeping up unheard needs Stealth too (60 % quieter at 100: one cell), but hearing only sends the kobold to look; it is the eyes that end the ambush, at five cells, every time.

**So, without a contest**: the ranged sneak attack works against everything (and is the ranger's and the thrower's fantasy); the melee sneak attack works against sleepers, mimics, the six types without darkvision, trap victims, and otherwise only by speed or by spending openings. The thief with a dagger, the owner's rogue, would make nearly every sneak attack of their life on something asleep. That is not enough: the design needs a contest.

**The smallest contest: slipping past** (`world/slip.rs`, data `[slip]`, decision 15). One roll, at one moment, stored in the creature's mind:

| Rule | Decision |
|---|---|
| **When** | in `ai.rs::nearest_hostile_in_sight`, as a creature that is hunting no one (`last_seen` unset) finds the **player** seen **by darkvision alone** (the player's cell is below its eyes' light threshold: `sight::sees` says so because `in_darkvision` was what settled it; a one-bit return beside the bool) and `Mind::missed` is unset: `check::contest(rng, player's Dexterity modifier + Stealth benefit + the talents' `slip_bonus`, creature's Willpower modifier + its Spot benefit)`, four gameplay draws. The player's cell lit enough for its eyes (a lantern, a fire, the sky): no roll, seen as today. Any other hostile (a creature of another faction): no roll, as today |
| **Made** | `Mind::missed = Some(state.turn)`: the creature does not see the player (the loop goes on to the next hostile, as if the cell were dark to it). It stays blind to them while `missed` is set: it patrols, looks into noises (the thief's footsteps send it to look, and it still does not see them when it arrives), walks past. `unaware(creature, player)` is true (§3.1) |
| **Failed** | seen as today: `last_seen` set, it hunts. `missed` stays unset, and **no second roll this stalk**: it hunts until it gives up (its mind goes calm), and only a fresh approach, from darkness, after that rolls again. No anti-farming rule beyond what the mind already does: time and distance are the price |
| **Over** | `missed` is cleared (and the creature sees as today from its next look) when: the player's cell is lit enough for its eyes at any look (they stepped into the light, lit the lantern); the player attacks it or anything within its sight (an attack it hears or sees: `resolve`'s noise reaches it, or `last_seen` is set by its next look at a lit fight); it is hurt by anything; or `memory_turns` × 3 turns have passed since `missed` (a long stalk ends by itself: 60 turns, data `[slip] lasts_turns`). A thrown voice does not clear it (it goes to look, still blind) |
| **Creatures slipping past the player** | no (decision: symmetry is kept where both sides have the same means; the player sees by light as creatures do, and a creature's approach from the dark is already unseen when it is unlit beyond the player's own darkvision, which is the surprise of §3.1). A creature type with Stealth in its `skills` is quieter, as today, and nothing more |
| **Draws** | four, only at that moment: a lit player, a creature already hunting, a creature that sees by light, a creature with no darkvision, a stalk already decided: none. Runs where the player walks unlit within five cells of a darkvision creature that is not hunting them change hashes from that creature turn |
| **Odds** (the wiki states them) | Dexterity +2 and Stealth 30 (+2, through the same `quiet` ladder? no: a new `slip` benefit, below) against a kobold at Willpower 10 and no Spot: about 69 % (the contest's even case is 52 %, each point of edge about 4 %); a thief at Stealth 100 (+8) against a drow (Spot 1d6+4, Willpower +2): about 70 %; a master at 200 (+12) against a kobold: 90 % |

Stealth gains a second benefit for it: `{ effect = "slip", curve = [[0, 0], [100, 8], [200, 12]] }` (the ladder every other check benefit uses), read in `slip.rs` only; Spot's existing `spot` benefit is the creature's side (it already means "noticing what is hiding", and creatures with `skills = { spot = N }` get it: the drow's keen eyes). Hush-foot, Silent feet, and Hide in plain sight stay what they are (noise); a new talent is not needed for the contest to ship (the `awareness` system's four cover it; **Shadow's Stalker's eye** marks exactly the creatures slipped past or otherwise unaware, §8.1).

**The worked example, with the contest.** The same corridor. At 5 cells the kobold's darkvision reaches the thief; it is hunting no one and the thief's cell is dark to it: the contest, once. Made: it walks on, or, hearing the thief's steps, comes to look, and arrives beside them none the wiser; the thief's dagger is a sneak attack (2d6 at Sneak attack 25, after Protection), the kobold is Surprised (or whirls), and the thief's second blow in the same breath, if they are faster, lands on a flat-footed foe too. Failed: it sees them, hunts, and the fight is an ordinary one, until the thief breaks away into the dark, it gives up, and the next stalk rolls again. With Lurk (§6.2), the thief waits in the dark and lets it come.

**Recommendation**: ship the contest (S8, its own slice, after S1), because the audit shows the melee sneak attack does not happen without it; keep it to this one roll and this one moment. The first draft's rejection of a *hiding mechanic* (where to hide, what breaks it, creatures that search) stands: this is not that.

## 4. The Sneak attack skill

```toml
[[skill]]
id = "sneak_attack"
name = "Sneak attack"
description = "Finding the telling spot in a foe that has not seen you coming."
kind = "rules"
mark = [{ on = "sneak_attack", chance = 50 }]
# The dice added, after Protection, to a hit on a target unaware of you or surprised (spec §8
# "Surprise and sneak attacks"): 1d6 trained at all, +1d6 at 25, 50, 75, and 100, then +1d6 for
# every 50 past 100 (5d6 at 100, 7d6 at 200). And how hard those you catch unawares find it to
# recover their wits: +8 to the check's difficulty at 100, +12 at 200.
benefit = [
    { effect = "sneak_dice", steps = [1, 25, 50, 75, 100], every = 50 },
    { effect = "surprise", curve = [[0, 0], [100, 8], [200, 12]] },
]
```

It sits in the skills list after Search. `skills::Effect::SneakDice` and `Effect::Surprise`; `Use::SneakAttack` (a sneak attack of the player's that hits, reported in `ambushed` where the dice land, beside the `Build`).

| Rule | Decision | Where |
|---|---|---|
| **The dice** | `World::sneak_dice(attacker)` = `Dice::of(skill_effect(SneakDice), 6)` `and` the talents' `ambush` (Ambusher 1d6, Improved sneak attack 2d6: `TalentMods::ambush` as today). `World::sneaking` becomes "`sneak_dice` is some, and the target is unaware or Surprised". `ambushed` rolls them on a hit as today: after Protection, never soaked, never multiplied by a critical (D&D's precision damage is not multiplied, and the code already does this), of the blow's own kind | `ambush.rs` (`sneaking`, `ambushed`), one read of `SneakDice` |
| **The craft** | the `surprise` benefit, the attacker's part of the check's difficulty (§3.2) | `startle`'s `craft`, one read of `Surprise` |
| **What counts** | a melee blow at any reach (a lunge, a reach blow), the off hand's blow, a shield bash, a kick, a free blow; a shot, a throw, or a spell that rolls to hit **within `missile_reach` cells** (Chebyshev distance from attacker to target, 6; D&D's 30 ft; decision 3); a burst, a bolt that does not roll to hit, a trap's missile, a status tick: never. The dice ride the attack's `Profile` path, so every blow path gets them from the one hook | `resolve` (the distance check in `sneaking` from `from`/`to`), `cast_single` |
| **Which weapons** | any; bare hands too. The thief's daggers get their flavour from the Daggers weapon ability (§6.4) and the two thief styles, not from a restriction (a restriction would make a sneak attack with a found long sword feel like a bug) | — |
| **Who is immune** (decision 5) | **no vitals = immune to criticals**, one flag: incorporeal creatures (`Content::incorporeal`), animated objects (`CreatureDef::animated`), the `no_vitals` tags (elementals; `skeleton`, the walking bones; `spirit`, the incorporeal dead; `construct`, the mud golem and every animated object), a type with `vitals = false`. On such a target **no critical is ever confirmed** (`Target::no_criticals`: `combat::strike` treats a threat as a plain hit, no confirmation roll drawn), **no sneak dice land, no Master thief threat**; it is Surprised like anyone. The game had no critical immunity before; this is it, and it is the same list, so a thing that cannot be stabbed in the heart cannot be run through either. The fleshy undead (the draugr, the ghouls, the drowned, the vampires, the mummies) **have** vitals: the barrow and the minster are the Ascmearc's first dungeons and a thief needs a game there. Vital strike (a sure critical) on a no-vitals target is a plain blow, said so in its line ("It has no vital spot.") and still priced and paid (the player can see what it is; the abilities menu greys it out against such a target only once it is known, which it is not: no leak, the blow simply does what it can) | `Content::has_vitals`, read in `World::target_of` (the `Target`), `sneaking`, `sneak_threat` |
| **Criticals and the 2d12** | the attack roll is the ordinary one against the stripped Defence; Master thief's threat −2 as today; a confirmed critical multiplies the weapon's dice and bonus as today and adds the sneak dice once after. Vital strike on an unaware foe with vitals: both | `combat::strike_with`, one `no_criticals` branch |
| **Stealth** | the way there (quiet steps and doors, and now the slip contest, §3.4); Sneak attack is what the blow does once there. Ambusher keeps its Stealth 30 prerequisite. Hide in plain sight's quieter attacks mean a sneak attack is less likely to wake the sleeper's neighbours, as today | — |
| **Weapon skills and styles** | the dice are added after `Profile::with(benefit)` and the strike, so ranks' to-hit, damage, threat, and speed apply to the attack and never to the dice; the sneak attack trigger (§6.4) fires for the attack's skills | — |
| **Openings and Patient blade** | unchanged: `Build::SneakAttack` on a hit, `sneak_kill` on a kill | — |
| **Marks** | `Use::SneakAttack` (50 %) when the dice land; a Dexterity stat mark too (`[[training.use]] stat = "dexterity", on = "sneak_attack", chance = 20`, `StatUse::SneakAttack`, `data/stats.toml`) | `ambushed`, `World::train` |
| **The character screen** | the value only, as any skill; the attacks list (`OwnAttack`) does not show the dice (situational changes are left out of it already: ambush is named there) | — |

**Numbers.** A level-1 thief (2d6+10: 12 to 22) has 1d6; at 25, 2d6; the thief is known for it (its rolls throw a bigger die), with 3 rolls a level, so about 100 by level 25 to 30 (5d6), then Improved sneak attack's 2d6 at 32 (7d6), Ambusher's 1d6 (8d6), and 200 around level 50 (10d6 in all: D&D's rogue at 20). Against the probe's monsters (`docs/balance.md`): a level-30 thief's dagger blow does about 12 after Protection; 5d6 (17.5) more on an unaware foe roughly triples the first blow of a fight, which is the fantasy and no more, since the second blow meets an aware foe at full Defence (or a Surprised one, once). The owner accepts unbalanced choices; S7 adds a row to the balance probe so the number is watched.

## 5. The Initiative skill

```toml
[[skill]]
id = "initiative"
name = "Initiative"
description = "Keeping your wits when something falls on you unseen."
kind = "passive"
mark = [{ on = "surprise", chance = 100 }]
# The initiative check (spec §8 "Surprise and sneak attacks"): a Dexterity check against 13 plus
# the attacker's craft, made when an attack finds you unaware or a hidden trap springs under you;
# made, you are not surprised: +8 at 100, +12 at 200. Failed, your first action afterwards costs
# half again as much, less this much of the extra: half of it at 100, three quarters at 200.
benefit = [
    { effect = "initiative", curve = [[0, 0], [100, 8], [200, 12]] },
    { effect = "surprise_ease", curve = [[0, 0], [100, 50], [200, 75]] },
]
```

In the list after Heal. `Effect::Initiative`, read in `startle`; `Effect::SurpriseEase`, read in `World::surprise_cost` (§3.2); nowhere else. `Use::Surprise` (a check made, whatever comes of it; the player's only, as every mark).

| Question | Decision |
|---|---|
| The stat | **Dexterity** (D&D's; reflexes). Wary (Willpower 12) is the wits' way in: +4 flat |
| What a made check buys | **no Surprised status**: full Defence from the next attack on, no dear action, free blows kept. The surprising attack itself was still rolled flat-footed and its dice still land (the attacker's craft is the attacker's) |
| What the benefit buys when the check fails (decision 4) | the dear action's extra cost shrinks: 150 % becomes 125 % at Initiative 100, 112 % at 200; with Cool head, 100 % |
| Odds (the wiki states them) | 2d12 + modifier against 13 + craft: Dexterity +0 against an untrained attacker, 78 of 144 (54 %); +2, 69 %; +0 against a master thief's +8 (21), 10 of 144 (7 %); Initiative 100 (+8) against 13: 138 of 144 (96 %). A sleeper gets no check |
| Who trains it | §6.1: the quick classes much (ranger, monk), the thief, the fighters, and the bard a little; nobody else. Creatures: §7 |
| An initiative **order** | **rejected** (§11): the energy scheduler decides who acts when, exactly and hidden; ties go to the player then to ids; speed is the one number. Initiative is the check and the ease, and only those |

## 6. Class fantasy: classes, the thief's table reworked, talents, weapon abilities

### 6.1 Classes and the drow (data; decision 7)

| Class or race | Sneak attack | Initiative | Why |
|---|---|---|---|
| **Thief** | `2d6+10`, **known for**; the only guaranteed start | `2d4+4` | the whole fantasy |
| **Drow** (race) | **by chance**: `optional_skills = { sneak_attack = { chance = 40, dice = "2d4" } }` | — | "the knife in the dark" is the drow's roster; a drow of any class may come with it, as a thief's Listen comes |
| Ranger | — | `2d6+10` | the one who is never caught out in the wild; its ambush is Ambusher (a Shadow pick), Camouflage, and Marksman's Hunter's shot, which fire for its arrow from the dark with or without the skill (no dice without it) |
| Bard | — | `2d4+4` | a quick tongue |
| Monk | — | `2d6+10` | Still mind's class |
| Warrior, barbarian | — | `2d4+4` | drilled or wild reflexes; a little |
| The rest | — | — | untrained: the check at Dexterity alone |

**Optional skills on races** (the smallest change): `races.toml` gains `optional_skills`, the same table and the same `OptionalSkill` type as the classes' (`content/class.rs`'s parser moves to `content/skill.rs` and both read it); `character.rs::optional_skills` rolls the class's list first, then the race's, from the **same stream** `character/optional-skills/<spec>`, so every class's existing optional rolls are unchanged and a race's draws come after them. A race without the field draws nothing. The alternative, hanging the drow's chance on the class roll, would make it a property of every class file; a field on the race is one line in one place.

### 6.2 The thief's table reworked (decision 10)

The owner: *"just rework all the thief abilities that replicate skill use... and give those skills to the class"*. Every class ability and talent of the thief was read against the skills the class trains (§2). The table keeps its ten slots:

| Level | Today | Replicates a skill? | After |
|---|---|---|---|
| 1 | Sneak attack (`ambush 1d6`) | its dice are now the **Sneak attack skill's** (trained `2d6+10`, known for) | **kept, reworked**: the class passive that goes with the skill, `effects = [{ kind = "grants_resource", resource = "openings" }, { kind = "surprise_difficulty", value = 2 }]`; "You find the telling spot in a foe that has not seen you coming (Sneak attack), and those you catch unawares are slower to gather their wits; each sneak attack that hits gives you an opening (Openings: they slip away with time)." The Openings pool is held from level 1 (decision 13) |
| 1 | Trained wind | no (a resource grant) | kept |
| 6 | **Trapfinding** (`searching +4`, `trap_ease 2`, the Disarm ability) | **yes**: Detect traps (#673: finding hidden traps, searching and in passing) and Disable device (the Disarm ability, had by whoever is trained) are both class skills already; `searching +4` is the Search skill's job | **removed**; the class data takes it: `detect_traps = "2d6+10"`, `disable_device = "2d6+10"` (from `2d4+4`), both added to `known_for` (with Open lock, Steal, Stealth, Tumble: six known-for skills, a bigger die on their rolls); `search` raised to `2d6+4`. Slot 6 → **Lurk** |
| 12 | **Pick lock** (the Pick lock ability) | **yes**: Open lock is trained `2d6+10` and known for from level 1; the talent was a second key to a door already open (`Ability::had_by`: `trains_effect(Pick) || talents.pick_lock`) | **removed**. Slot 12 → **Low blow** |
| 18 | Evasion (Finesse, given) | no | kept |
| 25 | Uncanny dodge (the burden's Dexterity cap) | no (no skill eases the cap) | kept |
| 32 | Improved sneak attack (`ambush 2d6`, grants Openings) | no (identity dice) | kept, loses the grant (moved to 1; `class::grant` leaves an old save's pool as it is): "Your sneak attacks hurt far more." |
| 40 | **Light fingers** (the Steal ability) | **yes**: Steal is trained `2d6+10` and known for; `had_by` is `trains_effect(Steal) || talents.steal` | **removed**. Slot 40 → **Vanishing act** |
| 50 | Hide in plain sight (no footsteps Unburdened, attacks half as loud) | no: Stealth quiets steps by a percent and never silences them, and no skill touches attack noise | kept |
| 60 | Master thief (threat −2, the finisher) | no | kept |

What the removed three leave behind: the `TalentEffect` kinds `disarm`, `pick_lock`, `steal` and their `had_by` arms **stay** (Tinkerer, the gnome's, and the craftsperson's Tinker use `disarm`; the arms cost nothing); the three files under `data/talents/` are **deleted**, and `Talents` drops an id the content no longer has on load (one line in `Talents::check`, said in the log; saves change, as the policy allows before 1.0). The wiki's class table and the talents-by-system matrix update themselves; `searching`'s and `doors`' coverage stays above the bar (Keen searcher, Stone sense, Trap sense; Lockbreaker, Knock, Mighty kick). `world/class_rogue_tests.rs` loses three cases and gains three.

**The new class abilities** (all `ClassActive`s or class passives in the one registry; rows of the abilities menu under the class source; no key of their own; `kind_of` prices the actives as `wait` and `melee` for the statuses that block):

| Level | Ability | What |
|---|---|---|
| 6 | **Lurk** (`lurk`, active, `use:lurk`, aim nothing) | a wait in ambush: `afflict(player, lurking, 1)`, priced as a wait (`price_of(Wait)`), no pool, no cooldown (time is the price). **Lurking** (`status.toml`: `max = 1`, `refresh`, `per_action`, no `seen`; `start = "You wait in ambush."`): while it lasts, a hostile that steps up beside the player takes the player's blow at once (the gatekeeper's free blow, `Cue::Approaches` in `World::step_up`, switched on by the status: one line in `World::gatekeeper`), and **is caught unawares by it unless it was already hunting the player** (`Mind::last_seen` set): `startle` is called for it as for any unaware target, so the blow is a sneak attack and the creature may be Surprised. With slipping past (§3.4) a creature that comes to look into the thief's footsteps and arrives beside them is exactly such a one. The status ends with the player's next action (the free blow is not an action of theirs). Refused with no time when a hostile already stands beside them ("Too late to lurk.") |
| 12 | **Low blow** (`low_blow`, active, `use:low_blow:E`, aim direction, `resource = { id = "openings", cost = 3 }`) | sand in the eyes and the knife where it hurts: a melee blow at the hostile that way (beside the player), priced as the blow is (`attack_price`), that **is a sneak attack whatever the foe knows** (the dice land on a hit; the foe is not Surprised: it saw the sand coming, and it keeps its Defence). `World::low_blow` sets a transient `Technique::sneak` (as Feint sets `ignores`) that `sneaking` takes. Greyed out "not enough openings" while short; refused with no time without a hostile there ("There is no one there to blind."). A no-vitals foe: the blow lands, no dice, the openings still spent (the thief can see what it is) |
| 40 | **Vanishing act** (`vanishing_act`, passive) | "The body drops, and you are already elsewhere: a sneak attack of yours that kills leaves you light on your feet." `effects = [{ kind = "sneak_kill_status", status = "light_footed" }]`, read where `Build::SneakKill` is raised in `resolve` (`afflict(player, light_footed, 1)`: the next action at half price, the weapon abilities' status). Identity: the burglar gone before the alarm |

**The ranger, the bard, the monk** get Initiative (§6.1) and no table change: the ranger's Ambusher-by-pick, Camouflage, Tracker, and Marksman's Hunter's shot (§6.4) are its ambush; the bard's Throw your voice is now the tool it was always meant to be (the creature it sends to look is unaware when the bard's knife finds it, though without the skill the knife has no dice: a bard who wants them takes a drow's luck or nothing). Keeping their tables as they are keeps S2 small.

### 6.3 Talents

Two systems. **`awareness`** exists `pending` in `talents.toml` ("Creature sight and awareness"): S1 takes the flag off with the first two. **`sneak_attack`** is new: `[[system]] id = "sneak_attack", name = "Sneak attacks: dice, reach, vitals"`, placed after `stealth`; S2 adds it with its talents and retags the existing four.

| Talent | Tree | Needs | Effect (new `TalentEffect` where marked) | Its one hook | Systems |
|---|---|---|---|---|---|
| **Wary** | Delver | Willpower 12 | `initiative_bonus = 4` (new) | `startle`'s modifier | `awareness` |
| **Cool head** | Grit | Willpower 14, level 9 | `surprise_ease = 100` (new: Surprised slows no action of yours) | `World::surprise_cost` (§3.2), summed with Initiative's and capped at 100 | `awareness`, `status` |
| **Stalker's eye** | Shadow | Stealth 20, Spot 10 | `sees_unaware` (new) | `Perception::compute`: `SeenActor::unaware` set from `World::unaware(creature, player)` for the seen creatures when the player holds it, else always false; look mode's word and the unaware mark (§8.1) | `awareness`, `sneak_attack` |
| **Hair trigger** | Finesse | Dexterity 14, Initiative 20, level 12 | `startled_blow` (new) | `startle`: a check **made** by the player against a hostile attacker beside them strikes it back at once, a free blow (`Cue::Startled` in `World::second_blow`, `Free::Startled`, held as the out-of-turn ones are); one per surprise; it brings no free blows of its own | `awareness`, `free_blows` |
| **Anatomist** | Scholar | Intelligence 13, Knowledge 20 | `vitals_in = ["skeleton", "construct"]` (new: tags) | `Content::has_vitals` takes the attacker's talents: those tags count as having vitals for them (never the incorporeal, never the elementals): their blows may critically hit and sneak-attack the bones | `sneak_attack`, `combat` |
| **Long reach** | Weaponcraft | any launcher or thrown skill at 4 | `sneak_reach = 3` (new) | `sneaking`'s distance check: `missile_reach` + the attacker's | `sneak_attack`, `ranged` |
| Ambusher | Shadow | (as today) | `ambush 1d6` | as today | `stealth`, `awareness`, + `sneak_attack` |
| Improved sneak attack, Master thief, Patient blade, the level-1 Sneak attack, Vanishing act | class | — | as §6.2 | — | + `sneak_attack` |

Coverage: `awareness` four talents and two class abilities (Sneak attack's `surprise_difficulty`, Lurk); `sneak_attack` three talents (Anatomist, Long reach, Ambusher) and five class abilities. Both well over the bar of two. `wiki/src/generated/talents-by-system.md` updates itself.

### 6.4 Weapon abilities

One new trigger, **`sneak_attack`**, raised in `ambushed` where the dice land (beside `Build::SneakAttack`), with the attack's skills; `Trigger::ALL` grows by one (the wiki's order: after `kill`). Two abilities on it, both on the catalogue's `status` effect kind (no new effect):

| Ability | Skill, rank | Effect |
|---|---|---|
| **Knife in the dark** | Daggers 5 | a sneak attack that hits with a dagger leaves the foe **bleeding** (`status = "bleeding"`, severity 1, on the target): "the knife finds the vein". The dagger is the thief's weapon by data, not by rule |
| **Hunter's shot** | Marksman 6 | a sneak attack that hits with a shot **pins** the foe (`status = "pinned"`, `turns = "1d2"`): the ranger's arrow through the foot before the quarry knew it was hunted |

A creature with the rank (the drow assassin fights with its envenomed natural attack, not a dagger, so neither fires for it; a drow with a rank in Daggers and a dagger in hand would) has them by the same rule as every weapon ability.

## 7. Creatures

**Skills on types** (data; `skills` is read for creatures already): the dice and the check through the same `Actor::skill_effect`. Eight types, the number read from the roster descriptions:

| Type | Level | `sneak_attack` | `initiative` | Note |
|---|---|---|---|---|
| drow assassin | 32 | 160 (6d6) | 100 | "the knife in the dark"; speed 1350 |
| drow stalker | 19 | 75 (4d6) | 60 | "opens a throat" |
| quickling | 6 | 40 (2d6) | 150 | "cutting before it is seen"; speed 2000: it will surprise often, and is nearly never surprised |
| darkling | 9 | 30 (2d6) | 50 | a shadow of the Unseelie |
| meenlock | 16 | 30 (2d6) | 30 | the lurker (below) |
| redcap | 12 | 40 (2d6) | 40 | the murderous little man |
| sluagh | 43 | 120 (5d6) | 60 | the restless host, out of the night |
| barrow spider | 2 | 10 (1d6) | 20 | the lurker in the barrow's dark |

Everyone else: no dice, the check at Dexterity alone (a kobold at Dexterity 10 shrugs off half the thief's surprises; the thief's craft lowers that as it grows). Numbers are the roster PR's to tune; the wiki table (§10) shows them. Per the creatures rule, these go in the per-tribe roster notes (`docs/dev/creatures/*.md`), not `creatures.md`. A creature's `skills = { spot = N }` is its side of the slip contest (§3.4): the drow (Spot `1d6+4` as a race) and the sharp-eyed get theirs there.

**Lurking** (one AI hook, `ai.rs::fight`): a type with `lurks = true` (`CreatureDef::lurks`, default false) that sees a hostile **does not close in**: it waits where it stands (`Plan::Wait`; its mind still records `last_seen`, so it is aware) until the target is beside it (then it attacks as usual), or until it has been harmed (`Actor::hp < max_hp`: then it hunts as any creature). A ranged lurker shoots from where it stands if it can. The meenlock and the barrow spider lurk. It is deterministic and draws nothing; it makes a corridor's dark alcove a place where a thing waits, which is what the two descriptions say. A lurker that sees the player is **aware** (it is hunting): walking into it is not a surprise for the player; being bitten by one they never saw is. Whether a lurker's wait is coarse while the player rests follows `stride.rs` as any wait does.

**No creature uses Lurk or Low blow** (actives are the player's, as the weapon actives are; creatures hold no Openings), and **no creature slips past** (§3.4); the drow assassin's fantasy is its 6d6 and speed.

## 8. What the player sees

Told **after the fact**, never before, so nothing of a creature's mind leaks that the player could not see with their own eyes:

| Moment | Text | Where |
|---|---|---|
| The player's attack finds a creature unaware and the creature fails its check | "You catch the kobold unawares." then the blow's own line | `startle`, said at once (`player_attacks` says the player's own attacks at once) |
| ... and the creature makes its check | "The kobold whirls to face you." then the blow's line (the dice still landed: the damage is in the blow's words as always; the surprise did not stick) | `startle` |
| ... on a sleeper or a mimic | the waking line ("The goblin wakes up.") or the reveal line as today; no second line (it was asleep: unawares goes without saying) | `den.rs`, `mimic.rs` |
| A creature's attack finds the player unaware and they fail | "You are taken by surprise!" (the status's `start`, a `say`: it stops a rest), before the creature's attack line | `Event::Status` |
| ... and they make the check | "You keep your wits about you." before the attack line | `startle` (`say`) |
| The dear action is taken | "You recover your wits." (`end`, a `note`) | `Event::Status` |
| A hidden trap springs | the trap's line, then the surprise line (or the wits line) | `step_onto_trap` → `startle` |
| Slipping past | **nothing**: a creature that did not notice the player gives no sign, and one that did hunts them as today (its approach is its own tell). With Stalker's eye the unaware mark and look mode's word say it (§8.1) | — |
| Hair trigger | the free blow's own line ("You stab the drow stalker.") after "You keep your wits about you." | `strike_free` |
| Lurk | "You wait in ambush." (`lurking`'s `start`; it ends silently); the free blow's lines as it comes | `use_class` |
| Low blow | the blow's line; refused: "There is no one there to blind." | `World::low_blow` |
| Vital strike or Low blow on a thing without vitals | "It has no vital spot." after the blow's line, the first time against that type in a run (`Progress::once` is not for this; a plain line every time is simpler and honest: the player can see it is a skeleton) | `active_blow`, `low_blow` |
| Look mode | "A kobold (hostile, unhurt, surprised), on stone floor." while the status lasts (`seen = "surprised"`: the player saw it flinch); "asleep" as today; **with Stalker's eye**, "A kobold (hostile, unhurt, unaware of you), ..." from `SeenActor::unaware`; without it, never a word about awareness | `ui/look.rs` |
| The status bar | `Surprised` among the player's effects while it lasts; `Lurking` likewise | `own_effects` |
| The Skills screen | Sneak attack and Initiative with their values, as any skill; never the dice or the bonus | `own_skills` |
| The abilities menu | Lurk and Low blow under the class source; Sneak attack and Initiative are not listed (neither is of `kind = "action"`) | `Ability::every` |
| The character screen's attacks | unchanged (the ambush is a situational change, left out by rule) | — |
| The death screen | unchanged ("Killed by a drow assassin"); the surprise is in the log before it | — |
| Dumps | `actors` (dev): `unaware of player` / `aware` / `slipped past` per creature, `noticed turn N`, `missed turn N`, and `lurks`; the `effects` of the header and of seen actors carry `surprised` and `lurking` as any status; JSON `effects`; `skills` layer the two values; `abilities` the two actives | `dump.rs` |

**What never leaks**: the check's roll, modifier, and difficulty; the contest's; the dice; a creature's `last_seen`, `noticed`, `missed`, or `sees_now` outside the dev layer; which creatures have vitals (the wiki says; the game says it only by the damage and the one line). The perception snapshot gains one field, `SeenActor::unaware: bool`, always false without Stalker's eye, computed in `Perception::compute` from the rules (as Mimic-wise's reveal is), so the UI reads the snapshot and nothing else.

### 8.1 The unaware mark (decision 11)

The owner asked for a visual indicator on unsuspecting enemies when the character has the talent. **Stalker's eye** is the gate (a talent, as asked; not a skill threshold: a threshold would make the knowledge a side effect of training, and a pick is the player choosing to read their marks).

| Part | Decision |
|---|---|
| **What is known** | `SeenActor::unaware: bool` in the snapshot (§8), `World::unaware(creature, player)` for each visible creature, computed in `Perception::compute` **only when the player holds `sees_unaware`**; false for everyone otherwise. The renderer never asks the world: the talent grants the knowledge, perception carries it, `ui/` reads perception. A sleeper and a mimic revealed to Mimic-wise are unaware too and marked the same |
| **The look** | `ui/glyphs.rs`, where each creature's free glyph is coloured: an unaware creature's `fg` is mixed toward a cool slate (`UNAWARE_TINT = #8a96a8`, a palette entry so themes can set it) by `UNAWARE_MIX` 0.25 before lighting, so the glyph reads a shade cooler and quieter than its neighbours: the kobold is still red, just not quite as red. With glyph animation on, the mix **breathes** between 0.15 and 0.35 over 6 s (`anim::breathe(VisualTime, phase)`, the creature's own phase as the bands' sway uses), so the eye finds it without a glyph or an outline. No shader change: free glyphs are coloured on the CPU, and the tint is a pure function (`glyphs::unaware_tint(fg, t)`) with a unit test for both ends |
| **Reduced effects** (`--reduced-effects`, `glyph_animation` off) | the mix stands still at 0.25: still information, no motion (the bands' sway is 0 the same way). Not nothing: the talent's whole worth is the knowledge, and the owner's bar is "no animation, no particles", not "no information" |
| **Where it does not draw** | a creature sliding (motion) is marked the same (the glyph is the same free glyph); remembered creatures are not drawn at all; the map grid's base colours never carry it (creatures are not on the grid); look mode says "unaware of you" beside the glyph's tint |
| **Golden?** | **no**: the tint is a CPU colour function (unit-tested at both ends of the breath and under reduced effects), the gate is perception (a core test: `unaware` set only with the talent), the wording is a text snapshot (look mode). A `--capture` of the arena with `--talents stalkers_eye` is looked at and noted in S6's PR, at `--freeze-time 0` and `3`, and not kept. The goldens are for what only the GPU produces; this is not that |
| **Subtlety** | the bar from `docs/dev/rendering.md`: barely perceptible; a quarter mix toward a near-grey is less than the darkvision fade. Tune with captures beside an aware creature of the same type |

## 9. Determinism, saves, replays

| Concern | Decision |
|---|---|
| **Streams** | the initiative check: two draws of the **gameplay** stream (`check::check`), in `resolve` **after** `favoured`/`bane`/`sneak_threat` shape the profile and **before** `combat::strike_with`'s draws; for a spell, in `cast_single` before its roll; for a trap, in `step_onto_trap` after the trap's own draws. The slip contest: four gameplay draws in `nearest_hostile_in_sight`, at the moment §3.4 names, inside the creature's turn (a rule's draw, as a trap's check under a creature's step is; the AI's decision is then a pure function of the mind). The sneak dice: gameplay, after the strike, as today. Marks (`Use::SneakAttack`, `Use::Surprise`, the Dexterity mark): the **training** stream, the player only. Lurk's and Low blow's blows draw as blows. `unaware`, `has_vitals`, lurking, the Defence strip, `no_criticals` (one draw *fewer*: no confirmation): no draws of their own |
| **When draws happen** | only when an attack finds its target unaware, a hidden trap springs on an awake actor, or an unlit player comes within darkvision of a creature hunting no one. A fight that begins in plain sight, or in the light, draws exactly as today; `no_draws_while_nothing_is_unaware` compares every hash of such a fight with and without `surprise.toml` |
| **Hashes that move** | every run whose first attack found a creature unaware (the showcase's thief against the depths' patrol, the arena's dark corner, the golden replays that open with an ambush): two draws more from that step on; every run whose unlit player walked within five cells of a darkvision creature at its post: four more from that creature turn; every fight that confirmed a critical on a thing now without vitals: one fewer. The snapshot tests and replays of `ambush`'s and `den.rs`'s tests are re-made in S1, S2, and S8 and reviewed; the golden manifests' replays are rebuilt by `cargo xtask golden` as always (no new golden entry: nothing here is a GPU effect) |
| **Saved state** | the statuses (`surprised`, `lurking`: `StatusId` indexes join the fingerprint, as every status); **`Mind::noticed: Option<u64>`** and **`Mind::missed: Option<u64>`** on creatures (serde-skipped while `None`; hashed): old saves load with `None` (every creature a stranger again, nobody slipped past: harmless). The skills are saved by id (new ids, nothing shifts); three thief talents gone from saves on load (§6.2). `CreatureDef::vitals` and `lurks` are content; `RaceDef::optional_skills` is content. `Technique::sneak` is transient. `FORMAT_VERSION` is the integrator's; the PRs say "saves change" |
| **Replays** | `use:lurk`, `use:low_blow:E` are `Action::Use` as the other class actives; old replays carry none and load; a replay of the menu reads back the same text. A drow's optional Sneak attack rolls from `character/optional-skills/<spec>` after the class's draws, so every existing character's skills are unchanged but a drow's |
| **Perception and state** | `perceive` writes `noticed` in the turn pipeline only; `from_state` recomputes the snapshot without writing it, so a save loaded and a run continued agree. `SeenActor::unaware` is computed from state, never stored |
| **Content fingerprint** | the two statuses and the two skills' use/effect ids are in it as their kinds are; the talents are not (named by id) |

## 10. The wiki, the spec, and the docs

| Where | What |
|---|---|
| `wiki/src/surprise.md` (new, under Combat after Fighting styles in `SUMMARY.md`) | "Surprise and sneak attacks": the rule in words (unaware, slipping past, surprised, flat-footed, the dear action and what eases it, the check, what counts, what has no vitals and so takes no critical), with generated tables from `xtask/src/wiki/surprise.rs`: the Sneak attack ladder (value → dice, from `skills.toml`), the Initiative ladder (bonus and ease), Stealth's slip ladder, the check's odds by Dexterity edge and craft (`check::ways`, as `stealing.md`'s table), the contest's odds by edge (`check::contest_ways`), the creatures that sneak-attack, recover, or spot (from creature data: type, level, dice, bonus; and which lurk), what has no vitals (the tags and the types; "immune to criticals and sneak attacks"), the statuses' numbers (150 %), `missile_reach`, `memory_turns`, the stalk's length |
| `wiki/src/skills.md`, `classes.md`, `races.md`, `talents.md`, `weapon-skills.md`, `fighting-styles.md`, `abilities.md`, `creatures.md`, `combat.md` | the existing generated tables pick up the two skills, the class dice and the drow's chance (the races' optional column, new), the talents, the two weapon abilities (by their rows), the thief's actives and its reworked table, the creature skills; prose lines where a page explains (combat.md: criticals and what is immune) |
| `spec/08c-combat.md` | a new bullet **Surprise and sneak attacks** after Free blows: the model of §3 (slipping past included), the skill of §4, the check and ease of §5, what counts, vitals and critical immunity, the messages, the unaware mark's gate |
| `spec/08i-character.md` | Skills: the two entries and Stealth's second benefit; Character creation: races' optional skills; the thief's table bullet: the three removed, the level-1 change, Lurk, Low blow, Vanishing act; Talents: the six; Weapon abilities: the trigger and the two |
| `spec/08b-creatures.md` | Creature behavior: `lurks`, slipping past in `nearest_hostile_in_sight`; the `skills` note gains the two skills; `vitals` |
| `spec/04-rendering.md` | the unaware mark (the tint, the breath, reduced effects) |
| `spec/07b-data.md` | `surprise.toml`, the creature fields, the race field, the status fields in use |
| `docs/dev/surprise.md` (new topic; listed in CLAUDE.md after `stealing.md`) | modules (`world/surprise.rs`: `startle`, `surprise_cost`, `unaware` moved there from `ambush.rs` with `sees_now`; `world/slip.rs`: the contest and `Mind::missed`; `ambush.rs` keeps the dice), data, invariants (one definition of unaware, one place anyone is surprised, one moment anyone slips past, the draw order, the no-draw rule), tests |
| `docs/dev/combat.md`, `creatures.md`, `skills.md`, `progression.md`, `weapon-abilities.md`, `status-effects.md`, `traps.md`, `rendering.md`, `character-creation.md` | one line each at the hook: the strip and `no_criticals` in `target_against`/`target_of`, `lurks` and the slip call in the AI overview, the effects and uses, the talent hooks table rows, the trigger row, the two statuses, `startle` in `step_onto_trap`, the unaware tint in `ui/glyphs.rs`, the races' optional skills |
| Tests | `world/surprise_tests.rs` (both ways round: the player surprised by a creature from the dark and not by one seen lately, the check's two outcomes, the strip, no free blows, the dear action and its ease, traps and their bystanders, sleepers without a check, mimics, the messages, saves and replays, the no-draw hash test), `world/slip_tests.rs` (the one moment, lit/unlit, hunting/not, made/failed, the endings, a creature of another faction never rolls, the dark corridor example as a replay), `ambush_tests` reworked for the skill, `world/vitals_tests.rs` (no critical confirmed, no dice, Anatomist, Vital strike's line), `class_rogue_tests` for the reworked table, Lurk, Low blow, Vanishing act, and the dropped ids on load, `weapon_ability_tests` for the trigger, `character.rs` for the drow's optional skill and the unchanged class rolls, `content::surprise` parse tests, the screen snapshot of the abilities menu with the thief's actives, `ui/glyphs.rs` for the tint, the wiki test |

## 11. Choices made along the way, and what is rejected

| Choice | Decision and why |
|---|---|
| **A whole lost action** (D&D's surprise round) | rejected: the owner's "more expensive first action" is the Warlock reading of it, and 150 % on the energy scheduler is exactly half an action lost (decision 1); a full turn lost (300 %, Drowsy's) would make every ambush by a fast creature a near-certain second free blow, and the player's death screen would often read "surprised, then dead" |
| **Flat-footed loses Parry and Shield too** | rejected: D&D keeps armour and shield against the unseen blow; a shield held up stays held up. Dexterity and Dodge go (the categories that are *reacting*); Armour, Shield, Parry, Natural, Deflection, Wall stay. Feint takes Parry and Shield, so the two together strip nearly everything, which is what a feint on a surprised foe deserves |
| **Attacks of opportunity** | not added: the free blows are Warlock's reactions already (riposte, opportunist, gatekeeper); surprise turns them off, which is the one thing 3.5's flat-footed says about them |
| **Flanking** | rejected (decision 6, "not for now"): the player has no allies (summons are enemy-only), so flanking would be a rule that only ever works against the player, and being surrounded already costs enough. If allies come, "flanked" is one more door into `sneaking` beside unaware and Surprised, nothing else changes |
| **An initiative order** | rejected (§5): the scheduler is the order |
| **A hiding mechanic** (where to hide, what breaks it, creatures that search) | still rejected; **the slip contest is not it** (§3.4, decision 15): one roll at one moment, stored in the mind, no new state of "hidden", no change to what creatures do; it is the thief's Stealth answering the one question the eyes ask. The first draft rejected a contest outright and was wrong about what that left the thief: the audit (238 of 294 with darkvision) is in §3.4 |
| **Critical immunity as its own field** | rejected: one flag (decision 5 read onto the vitals list): a thing with no telling spot takes neither a critical nor a sneak attack, and the data says it once; the game had no critical immunity to tie to, so this adds it |
| **Which creatures have vitals** | the fleshy undead do (§4): the Ascmearc's first two dungeons are theirs; the bones and the bodiless do not. Anatomist buys the bones back |
| **Sneak attack dice multiplied by a critical** | no (3.5's precision rule; the code already adds them after) |
| **A separate "flat-footed" status and "surprised" status** | one status: Surprised is flat-footed until the next action, and the next action is dear; two statuses would be two timers for one moment |
| **The thief's level-1 talent renamed** | kept its id `sneak_attack` (saves name talents by id; a rename is a migration for nothing) and its name; only its effects move (decision 10). The skill shares the name, which reads as intended: the class ability "Sneak attack" is what the thief makes of the skill |
| **The three replicating talents kept as empty shells** rather than deleted | rejected: dead talents in the wiki and the Talents page for the sake of old saves, before 1.0, when the policy says saves change; one tolerant line on load does it |
| **Openings from level 1** | yes (decision 13): the pool was granted at 32 because nothing before Master thief spent it; Low blow at 12 spends it, and a level-1 thief watching openings tick up from their first ambush is the fantasy at level 1 |
| **Sneak attack for the ranger and the bard** | rejected (decision 7: "only thief gets sneak attack guaranteed"); the drow's chance is the one other way in |
| **The unaware mark on a skill threshold** | rejected: the owner said "when character has this talent"; Stalker's eye it is, and a Shadow pick is cheap |
| **The unaware mark as a glyph or an outline** | rejected: an outline needs a shader path for free glyphs that does not exist, and a second glyph on the cell is a thing the player would read as an object; a cooler colour and a slow breath are the quietest sign that still reads |
| **Creatures lurking by a draw** ("a meenlock waits 60 % of the time") | no: no creature behaviour draws; `lurks` is a rule of the type |
| **The player's awareness as a per-creature turn stamp** rather than a set on the player | the creature's `Mind` is where the creature's half lives; the player's half about that creature lives beside it, serde-skipped while `None`; a set on the player would need pruning as creatures die and levels change. `missed` follows the same reasoning |
| **Memory of 20 turns** | decision 8; data |
| **Initiative on Willpower** | Dexterity (3.5); Wary is the Willpower character's way to the same place, and Cool head's prerequisite is Willpower: the wits' classes are covered by talents, the reflexes' by the skill |
| **The spell path** | an own cast that rolls to hit surprises and sneak-attacks (a wizard's bolt from the dark); a burst never (no roll, no telling spot); a scroll's or a wand's attack spell rolls to hit too and goes through `cast_single`, so it counts as well: a thief reading a scroll of lightning from the dark is still a thief |
| **Optional skills on races by a new field** rather than hanging the drow's chance on the class roll | the field (§6.1): one line in `races.toml`, the parser shared, the class rolls untouched |

## 12. Implementation plan

Nine issues in four waves; each links to its section. "**Action**" marks a PR that adds an action, "**State**" one that adds saved state; at most two State PRs run beside each other and one Action PR at a time. Nothing here overlaps the affixes and chests waves in flight beyond `fight.rs::resolve`'s few lines; the false-conflict trick (`docs/dev/merging.md`) for the hot files: `status.toml`, `skills.toml`, `talents.toml`, the thief's class file, `ambush.rs`, `Trigger::ALL`, `ability.rs`, `perception.rs`.

| Wave | Issue | What | Adds | Depends on |
|---|---|---|---|---|
| 1 | **S1 Surprise** (§3.1–3.3, §8, §9) | `data/surprise.toml` and `content/surprise.rs` (`no_vitals` parsed, nothing reads it yet; no `[slip]` yet); `world/surprise.rs`: `unaware` for the player (`Mind::noticed`, set in `perceive`), `startle` with the check at Dexterity alone (no skills yet), the status `surprised`, `Target::without` shared with Feint, the strip in `target_against`, free blows off, `surprise_cost` with the talents' ease only, the messages, traps as a source (the stepper only), sleepers and mimics without a check, dumps; `awareness` off `pending` with **Wary** and **Cool head**; `docs/dev/surprise.md`; the spec bullet; tests and the hash test; the re-made snapshots listed in the PR. `ambush.rs` keeps the dice and is asked for them as today | **State** (`Mind::noticed`, a status) | — |
| 1 | **S2 The skills, the class data, and vitals** (§4, §5, §6.1, §6.3's `sneak_attack` system) | `sneak_attack` and `initiative` in `skills.toml` with `Effect::SneakDice`, `Surprise`, `Initiative`, `SurpriseEase` and `Use::SneakAttack`, `Surprise`; `sneak_dice` from the skill and the talents; `missile_reach` and the spell path; `vitals`, `has_vitals`, `Target::no_criticals` in `combat::strike`; the Dexterity stat use; the thief's dice; `RaceDef::optional_skills` and the drow's chance (the parser shared, the stream order kept); the thief's level-1 talent reworked (openings from 1, `surprise_difficulty`), Improved sneak attack's grant moved; the `sneak_attack` system with **Anatomist** and **Long reach**, the four retagged; `creatures/README.toml`, `races.toml`'s notes; the skills' and races' wiki rows; `class_rogue_tests`, `vitals_tests`, `character.rs` tests | none (content ids; the class rolls unchanged) | S1 |
| 2 | **S8 Slipping past** (§3.4) | `[slip]` in the data; `world/slip.rs`: the contest in `nearest_hostile_in_sight`, `Mind::missed`, its endings, Stealth's `slip` benefit, `unaware` reading `missed`; the dumps' words; `slip_tests` with the corridor replay; the spec and the wiki's contest table | **State** (`Mind::missed`) | S1, S2 |
| 2 | **S3 The thief's rework: Lurk, Low blow, Vanishing act** (§6.2) | the three replicating talents removed and the class skills raised and known for; the tolerant load; the two `ClassActive`s, `use:lurk` and `use:low_blow:E`, the `lurking` status and its gatekeeper line, `Technique::sneak`, Vanishing act's hook on `sneak_kill`; the menu rows and the screen snapshot; the action text; replays; the class wiki table | **Action** (two) | S2 (Lurk's "caught unawares" reads S8's `missed` when it is there, and works without it) |
| 2 | **S4 The weapon abilities** (§6.4) | the `sneak_attack` trigger in `ambushed`, **Knife in the dark**, **Hunter's shot**, the wiki rows, `weapon_ability_tests` | none | S2 |
| 3 | **S5 The creatures** (§7) | `skills` on the eight types (and `spot` where a roster wants a sharp eye), `lurks` on two with the AI branch, the roster notes per tribe, `bestiary_tests`, the wiki's creature table | none | S2, S8 (the AI branch sits beside the slip call in `ai.rs`: after it, not beside it) |
| 3 | **S6 The unaware mark, Stalker's eye, Hair trigger** (§6.3, §8.1) | `SeenActor::unaware` and its gate in `Perception::compute`; look mode's word; `glyphs::unaware_tint` and the breath, reduced effects' still mix, the captures noted; `Cue::Startled`/`Free::Startled`; the two talents and their tests; `spec/04-rendering.md`, `docs/dev/rendering.md` | none (a snapshot field, transient) | S1, S2 (Hair trigger's Initiative need), S8 (the mark is most useful once a creature can be slipped past; it works without it) |
| 4 | **S7 The wiki page and the balance row** (§10) | `wiki/src/surprise.md`, `xtask/src/wiki/surprise.rs` (the ladders, the odds, the contest's odds, the creatures, the vitals), the balance probe's sneak-attack row in `docs/balance.md`, a cross-check of the spec against what landed | none | S1–S6, S8 |

**Why this order**: S1 and S2 first because every later slice reads `startle`, `unaware`, and the skill effects; S2 after S1 rather than beside it because both touch `ambush.rs` and `fight.rs::resolve`. Wave 2 puts the two remaining PRs that add state or actions (S8, S3) beside one that adds neither (S4): S8 lives in `ai.rs` and `slip.rs`, S3 in the thief's class file, `class_ability.rs`, and `ability.rs`, S4 in `weapon_ability.rs` and the trigger's one line in `ambushed`; their shared lines are the `status.toml` entries (S3's `lurking`) and `docs/dev/surprise.md`'s hook list, the false-conflict trick's territory. S5 after S8 because both edit `ai.rs::nearest_hostile_in_sight`'s neighbourhood; S6 after S8 so the mark can be tested on a slipped-past creature. S7 last.

Each issue answers the feature template's "Talents and class abilities": S1 (`awareness`: Wary, Cool head; the class passive), S2 (`sneak_attack`: Anatomist, Long reach, Ambusher; the thief's Sneak attack, Improved sneak attack, Master thief), S8 (extends `stealth` and `awareness`: Stalker's eye reads it; no new system), S3 (three class abilities, carrying `sneak_attack` and `awareness`), S4 (the abilities carry `weapon_abilities`, `sneak_attack`, `status`), S5 and S7 (content and docs: none, extending the systems above), S6 (the remaining two talents).

## 13. Open questions, each with a recommended default (answered: §14)

The first draft's questions, as the owner answered them (§14 quotes the answers; the body above applies them).

| # | Question | Recommended default |
|---|---|---|
| Q1 | **The dear action's price**: 150 % (half an action lost), or 200 %? | **150 %** (§11): data in `status.toml`, one number to change |
| Q2 | **A hidden trap surprises its victim** (the player and creatures)? | **yes** (§3.2): it is the one surprise that needs no attacker, and it makes a trap beside a creature a moment for both sides |
| Q3 | **Shots, throws, and attack spells sneak-attack within 6 cells** (Long reach +3)? | **yes, 6**: D&D's 30 ft; a thief's thrown dagger from the dark is the fantasy, a sniper's from across the hall is the ranger's (Marksman's Hunter's shot still fires inside the reach) |
| Q4 | **Does a made initiative check also keep the sneak attack's dice off?** | **no** (§5): the craft is the attacker's, the recovery the victim's; the other reading makes Initiative an anti-thief skill and leaves the thief's first blow a coin toss |
| Q5 | **Vitals**: are the fleshy undead (draugr, ghouls, drowned, vampires, mummies) sneak-attackable? | **yes** (§4, §11): the first two dungeons; the bones, the spirits, the elementals, and the constructs are not, and Anatomist buys the bones |
| Q6 | **Flanking**? | **no** (§11); one more door into `sneaking` if allies ever come |
| Q7 | **The classes that train the skills** (§6.1): the ranger's Sneak attack, the monk's Initiative, the fighters' little Initiative? | **as §6.1**; each is one line of class data |
| Q8 | **The player's memory of a seen creature**: 20 turns? | **20** (§11); data |
| Q9 | **The creature's check draws from the gameplay stream** (so ambush-opening replays change hashes), or a draw-free rule for creatures (a fixed threshold)? | **it draws** (§9): equal treatment, and a threshold would make every creature of a type recover always or never |
| Q10 | **The thief's level-1 talent keeps the id `sneak_attack`** and the skill takes the same name? | **yes** (§11) |
| Q11 | **Stalker's eye** tells the player which creatures have not noticed them: is that too much information for the vague game? | **keep it, as a talent** (§8): it is the thief's craft to read a mark, bought with a pick, and the snapshot carries it only then; without the talent nothing is said before the blow |
| Q12 | **Initiative's name**: "Initiative" (the owner's word), or "Reflexes" / "Wits"? | **Initiative**: the owner named it, and the wiki's table "the initiative check" reads naturally |
| Q13 | **Openings from level 1** (so Low blow has a pool), moving the grant off Improved sneak attack? | **yes** (§11) |
| Q14 | **Lurking creatures**: the meenlock and the barrow spider only, or the mimics' kin and more? | **those two** in S5; a roster PR may mark more, one line each |
| Q15 | **A hiding and spotting contest** (Stealth against Spot when a creature first sees the player) as a later design? | **defer** (the first draft; §3.4 now answers the owner's question and changes the recommendation: the smallest contest ships) |

## 14. Decisions (owner, 2026-10-28)

The owner answered the first draft's questions q1 to q15, numbered as its §13 was; the numbering lines up one to one. Each answer is recorded verbatim with how it was read where reading was needed. (The coordinator's note took q6 for the hiding contest; q6 was flanking, and the hiding contest is q15.)

| # | Question | Decision | Sections changed |
|---|---|---|---|
| 1 | The dear action's price? | "150 is fine" | — |
| 2 | A hidden trap surprises its victim? | "sure, but it surprises only those affected". Read as: the one the trap springs on, never a bystander (a burst's neighbours are hurt by the trap's rules, not surprised) | §3.2 |
| 3 | Missiles and attack spells within 6 cells? | "sure" | — |
| 4 | Does a made check keep the dice off? | "initiative can mitigate the cost of the first attack after being surprised". Read as the coordinator did: the Initiative skill shrinks the **extra cost of the dear action** (a second benefit, `surprise_ease`: half the extra at 100, three quarters at 200; Cool head all of it), beside the check. The check stays what it was (made: no status at all); the dice are never touched by either: the two halves are "shrug it off" and "soften it when you cannot" | §1, §3.2, §5, §6.3, §9, §10 |
| 5 | Vitals: the fleshy undead sneak-attackable? | "yes. but anything immune to crits is also immune to sneak attack extra damage". The game has no critical immunity today (none in the data, `combat.rs`, or the spec); read as: **the no-vitals list is the critical immunity**, one flag: no critical is confirmed on such a target and no sneak dice land; the fleshy undead keep both | §1, §2, §3.3, §4, §6.3, §9, §10, §11 |
| 6 | Flanking? | "not for now" | §11 |
| 7 | Which classes train the skills? | "only thief gets sneak attack guaranteed, also give it to drow as a chance to have". Read as: Sneak attack guaranteed for the thief alone (the ranger's and bard's dice dropped); the drow race gets it as an optional starting skill, 40 % for `2d4`, through a new `optional_skills` field on races that reuses #671's class field, parser, and stream (the class's draws first, so nothing else moves). Initiative's spread (the answer names Sneak attack only) stays as proposed | §1, §6.1, §9, §10, §11, §12 |
| 8 | Memory 20 turns? | "sure" | — |
| 9 | The creature's check draws? | "sure" | — |
| 10 | The level-1 talent keeps its id? | "your talent changes in docs sound fine. while at it just rework all the thief abilities that replicate skill use... and give those skills to the class". Done in §6.2: every thief class ability read against the class skills; Trapfinding, Pick lock, and Light fingers only unlocked what Detect traps, Disable device, Open lock, and Steal unlock when trained, which the thief trains from level 1: removed, the skills raised and known for; the slots take Lurk (6), Low blow (12), Vanishing act (40); the identity abilities stay | §1, §6.2, §8, §9, §10, §11, §12 |
| 11 | Stalker's eye's information? | "sounds good. can we make a visual indicator appear on unsuspecting enemies with shaders or something when character has this talent?" Yes: §8.1, the unaware mark, gated on the talent, drawn from the snapshot's `SeenActor::unaware` on the free-glyph layer as a cool tint with a slow breath (still under reduced effects), no shader change needed, no golden | §1, §6.3, §8.1, §10, §11, §12 |
| 12 | The name? | "initiative" | — |
| 13 | Openings from level 1? | "yes, design was good" | — |
| 14 | Lurking creatures? | "sure" | — |
| 15 | A hiding and spotting contest later? | "we don't need it? has player chance to actually sneak attack anyone without that skill contest??" Answered in §3.4: without a contest, the ranged sneak attack works against everything and the melee one only against sleepers, mimics, the six creature types without darkvision, trap victims, and by luck of speed, because 238 of 294 types see five cells in any dark; so the design **ships the smallest contest**, slipping past (one roll of Dexterity + Stealth against Willpower + Spot, at the moment a creature hunting no one would first see an unlit player by darkvision alone, stored in its mind, over when they are lit, attack, or hurt it), its own slice S8. A hiding *mechanic* stays out | §1, §2, §3.1, §3.4, §6.2, §7, §8, §9, §10, §11, §12 |

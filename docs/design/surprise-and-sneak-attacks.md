# Warlock — Surprise and sneak attacks: the Sneak attack and Initiative skills, the rogue's class fantasy

Design document for the owner's review (2026-10-28). Written against `origin/main` at 0c7d7c0e. Nothing here is implemented; §12 cuts it into issues. It builds on what is there: the thief's sneak attack and Ambusher (`world/ambush.rs`, `docs/dev/progression.md`), creature sight and the AI's mind (`docs/dev/creatures.md`, `sight.rs`), the energy scheduler and action costs (`docs/dev/time.md`), statuses that ease by actions (`docs/dev/status-effects.md`, `docs/dev/weapon-abilities.md`), the 2d12 rules and Defence categories (`docs/dev/combat.md`), skills and their marks (`docs/dev/skills.md`), checks (`check.rs`), the sleepers of teeming dens (`docs/dev/special-rooms.md`), weapon abilities and their triggers (`docs/dev/weapon-abilities.md`), and resources (`docs/dev/resources.md`). Every number is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-10-28, verbatim): *"feature: surprise and sneak attacks. we need a system for rogue-types to have some way express their class fantasy. this should include at least a new skill "sneak attack". We can look into dnd 3.5 for inspiration on the effects of a suprise. Perhaps the first action after getting surprised is also more expensive. This might tie also into concept of initiative, which might be a skill the reduce effects of surprise on oneself"*

What is taken from D&D 3.5, because the owner named it or it follows directly: being **surprised** (the surprise round, in which the unaware do not act) and being **flat-footed** (no Dexterity bonus to Defence, no attacks of opportunity, until one's first action); the **sneak attack** as precision dice against a target denied its Dexterity bonus; **initiative** as what decides who is caught out. What is not taken, with the reason, is in §11: flanking, attacks of opportunity as such, a whole lost action, an initiative *order*, a hiding contest.

## Contents

1. Summary
2. The owner's request, and what exists today
3. Surprise: the model
4. The Sneak attack skill
5. The Initiative skill
6. Class fantasy: classes, class abilities, talents, weapon abilities
7. Creatures
8. What the player sees
9. Determinism, saves, replays
10. The wiki, the spec, and the docs
11. Choices made along the way, and what is rejected
12. Implementation plan
13. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **Unaware** (exists, widened) | an actor is unaware of another when it has not seen it: for a creature, today's rule (`World::unaware`: it hunts no one, `Mind::last_seen` unset, and its eyes miss the attacker's cell now; a sleeper and a disguised mimic always); **for the player, new and symmetrical**: the attacker is not among the creatures their last perception saw, and they have not seen it within the last `memory_turns` (20) turns (`Mind::noticed`, the turn the player last saw that creature, set by `perceive`). The check is asked of **every** attack now, not only of an attacker with ambush dice. §3.1 |
| **Surprise** | an attack (a blow, a shot, a throw, a spell that rolls to hit) that finds its target unaware **surprises** it, and so does a hidden trap springing under it; a thrown voice and the dark are the ways to arrange one. A surprised actor gets one **initiative check** (Dexterity, plus the Initiative skill, against 13 plus the attacker's Sneak attack craft; none for a sleeper or a disguised mimic): failed, it suffers the status **Surprised** (`per_action`, until its next action): **flat-footed** (Dexterity and Dodge count nothing toward its Defence), **no free blows** of its own (riposte, opportunist, gatekeeper, Point-blank), and **its next action costs 150 %** (the owner's "first action more expensive"; the status's `cost`, as Staggered's). Every attack on it while it lasts is a sneak attack for an attacker with the dice. The attack that surprised it is already rolled against the flat-footed Defence. The player and creatures alike. §3 |
| **Sneak attack**, the skill | `sneak_attack` (`kind = "rules"`): its value buys **d6s added after Protection** to a hit on a target that is unaware or Surprised: 1d6 trained at all, +1d6 at 25, 50, 75, and 100, then +1d6 every 50 (5d6 at 100, 7d6 at 200); and a second benefit, **`surprise`** (+8 at 100, +12 at 200), added to the difficulty of the target's initiative check. Blows at any reach, shots and throws and attack spells **within 6 cells** (`missile_reach`). No weapon restriction (the daggers get their hook as a weapon ability). **No vitals** (incorporeal, animated, elementals, the bone-and-spirit undead by tag, a type that says `vitals = false`): no dice, no lowered threat, still surprised. The talents' ambush dice (Ambusher, Improved sneak attack) add to the skill's, as today. Marks on a sneak attack that hits (50 %); a Dexterity training mark too. §4 |
| **Initiative**, the skill | `initiative` (`kind = "passive"`): +8 at 100, +12 at 200 on the initiative check; trained at all it changes nothing else. Marks on every check made (100 %: a rare use). **No initiative order**: the energy scheduler stays the one rule of who acts when (rejected: §11). §5 |
| Classes | the thief trains Sneak attack (`2d6+10`, known for it) and Initiative (`2d4+4`); the ranger Sneak attack `2d4+4` and Initiative `2d6+10`; the bard Sneak attack `2d4+4`, Initiative `2d4+4`; the monk Initiative `2d6+10`; the warrior and barbarian Initiative `2d4+4`. Races unchanged. §6.1 |
| The thief's table | level 1 **Sneak attack** keeps its id and becomes the class passive that goes with the skill (the Openings pool granted from level 1, and those the thief catches unawares find it 2 harder to recover their wits); 6 **Lurk** (`use:lurk`, a wait in ambush: a hostile that steps up beside the thief takes a blow at once, caught unawares unless it was already hunting them); 25 **Low blow** (`use:low_blow:E`, 3 openings: a blow beside you that is a sneak attack whatever the foe knows); 32 Improved sneak attack and 60 Master thief as they are. No new key: the abilities menu, as every active. §6.2 |
| Talents | the `awareness` system comes off `pending` with **Wary** (Delver: +4 on the check), **Cool head** (Grit: Surprised no longer slows the next action), **Stalker's eye** (Shadow: look mode says which creatures have not noticed you), **Hair trigger** (Finesse: a check made against a foe beside you is answered with a blow at once); a new system **`sneak_attack`** with **Anatomist** (Scholar: the dice land on bony things too), **Long reach** (Weaponcraft: shots and throws sneak-attack 3 cells farther), and the retagged Ambusher, Improved sneak attack, Master thief, Patient blade. §6.3 |
| Weapon abilities | one new trigger, **`sneak_attack`** (raised where the dice land): **Knife in the dark** (Daggers 5: a sneak attack with a dagger leaves the foe bleeding) and **Hunter's shot** (Marksman 6: a sneak attack with a shot pins the foe a turn or two). §6.4 |
| Creatures | equal treatment through data: a type's `skills = { sneak_attack = N, initiative = M }` gives it the dice and the check as the player's do (the drow assassin and stalker, the quickling, the darkling, the meenlock, the redcap, the sluagh, the barrow spider: eight types); one AI hook, **`lurks = true`** (the meenlock, the barrow spider): it waits where it stands for prey to come beside it instead of closing in, until harmed. §7 |
| Messages | told after the fact, never before: "You catch the kobold unawares." / "The kobold whirls to face you." (the check made) before the blow's own line; "You are taken by surprise!" / "You keep your wits about you." for the player; look mode "(hostile, unhurt, surprised)" while the status lasts and "asleep" as today; "unaware of you" only with Stalker's eye. No number, no roll, no creature's mind otherwise. §8 |
| Determinism | the initiative check is two draws of the **gameplay** stream, made only when an attack finds its target unaware or a hidden trap springs on someone awake, before the strike's draws; the dice draw as today; marks on the **training** stream. Saved state: the status (as any), and `Mind::noticed: Option<u64>` on creatures (left out while `None`). Old saves and replays load; replays whose first attack finds a creature unaware hash differently from that step. §9 |
| Slices | seven issues in three waves: S1 surprise (rules, state, the first talents), S2 the two skills and the class data, then S3 the thief's actives (actions), S4 the weapon abilities, S5 the creatures, then S6 the perception talents and Hair trigger, S7 the wiki page and the balance row. At most one at a time adds actions and one adds saved state. §12 |

**Key choices, and why**

1. *Surprise is a status, and the first action after it is dear.* The game already has statuses that ease by their sufferer's actions and make that action dearer (Staggered: `per_action`, `cost = 150`): Surprised is one, so the owner's "first action more expensive" costs no new mechanism, the scheduler is untouched, and creatures suffer it exactly as the player does (`World::burdened` prices everyone's actions).
2. *Flat-footed rides Feint's code.* Feint already rolls an attack as if the foe had no Parry or Shield (`Technique::ignores`, `World::feinted`); a surprised target is rolled as if it had no Dexterity and no Dodge, through the same `Target::without(categories)`. One function, two callers.
3. *The player is surprised by the same rule as a creature.* "Has not seen the attacker" is the creature's `Mind::last_seen` plus its eyes; for the player it is their perception snapshot plus a short memory (`Mind::noticed` on the creature: the turn the player last saw it). Nothing is new in principle; the asymmetry today (creatures never surprise the player) was only that nobody wrote the player's half.
4. *Sneak attack is a skill, so it grows by the skill ladder, and the class talent stays a passive.* The thief's level-1 talent keeps its id (saves name it) and loses its dice to the skill; the skill's ladder is where the dice live, and anyone who trains it (a ranger, a bard, a creature) has them. Ambusher and Improved sneak attack keep adding dice on top, as today's `Dice::and` already sums them.
5. *Initiative reduces surprise; it does not order the fight.* The owner's phrase is "reduce effects of surprise on oneself". The energy scheduler is exact and hidden, and a second order on top of it would be either redundant or a leak; the skill is the check's bonus and nothing else.
6. *Creatures get it through data and one AI hook.* `skills` on a creature type exists and is read by the same `Actor::skill_effect`; the only new behaviour, lurking, is one deterministic branch in `ai.rs::fight`.

## 2. The owner's request, and what exists today

| Part | Today |
|---|---|
| The sneak attack | `world/ambush.rs`: `World::sneaking(attacker, target)` is true when the attacker holds a talent with the `ambush` effect (the thief's Sneak attack 1d6 at level 1, Improved sneak attack +2d6 at 32, Ambusher 1d6 in Shadow at Stealth 30 and level 9; `Dice::and` sums dice of one size) **and** `World::unaware(target, attacker)`. `fight.rs::resolve` asks it before the strike (no draw), lowers the threat for Master thief (`sneak_threat`, −2, never below 3), and `ambushed` rolls the dice on a hit, **after Protection** (never soaked, never multiplied by a critical), from the gameplay stream, then raises `Build::SneakAttack` (the Openings pool, R3) and `resolve` raises `sneak_kill` for Patient blade. Shots, throws, and blows alike; spells never (`profile` is `None` for an intent, and a spell that rolls to hit is resolved in `cast.rs::cast_single`, which never asks `sneaking`) |
| Unaware | `World::unaware(id, other)`: **creatures only**; true for a sleeper (`den.rs`, the Asleep status that `sleeps`) and a disguised mimic (`mimic.rs`) whatever they see; else when `Mind::last_seen` is unset (it hunts no one) and `World::sees_now(id, other)` is false (creature sight, `sight.rs`: radius, line of sight, light enough for its eyes). **Never for the player**: a creature coming out of the dark gets nothing on them |
| The creature's mind | `Mind { last_seen, noise, post }` on `Actor` (saved, hashed): `ai.rs::decide` sets `last_seen` the moment `nearest_hostile_in_sight` sees a hostile (deterministic, no draw: "no creature behavior may use randomness"), clears it when the search is given up (`search_or_return` walks back to the post and the mind goes calm). A noise sets `noise` only: a creature sent to look by a thrown voice (`voice.rs`) stays unaware |
| Awareness and the player | the perception snapshot's `actors()` (what the player sees now) and `World::seen_creature_at` (talk, steal); nothing remembers which creatures the player has seen |
| Sleep | Asleep (`sleeps = true`): no turn, unaware of everyone, woken by noise at its threshold × the den's `wake`, by harm, by a neighbour's cry; "The goblin wakes up."; look mode "asleep" |
| The scheduler | `schedule.rs::run_turn`: energy per actor, the player's price first, creatures in time order; `World::burdened` → `slowed` is the one place every price passes, and `Effects::slow` multiplies in each status's `cost` (Staggered 150, Sluggish 150, Drowsy 300, Transfixed 200). `per_action` statuses (Staggered, Pinned, Light-footed) ease a degree per action of their sufferer (`World::spend_actions`, called where each action's price is paid; a dose given during the sufferer's own action is not eased by it: `World::acting`, `World::fresh`) |
| Defence | `combat::Category`: Dexterity, Armour, Shield, Parry, Natural, Dodge, Deflection, Wall; `Defence` keeps the highest per category (Dodge stacks). `World::target_against(id, missile)` builds the `Target` (Deflect missiles' Dodge, Measure's Parry); `World::feinted` strips the categories a Feint ignores (positive bonuses only) |
| Free blows | `World::second_blow` on a `Cue` (`fight.rs`); `World::free_blows_of(actor)` folds the actor's switches, its talents', and the weapon abilities' (`ability_switches`); the out-of-turn ones are riposte, opportunist (`step_away`), gatekeeper (`step_up`), and Point-blank's free shot (`approached`) |
| Checks | `check::check(rng, modifier, difficulty)`: 2d12 + a stat modifier, natural 2 fails, 24 succeeds, two gameplay draws; `check::ways` the odds; `contest` four draws |
| Skills | `data/skills.toml`: trained skills only (a missing one gives nothing); `benefit` by `steps`/`every` or a `curve`; `mark` by `Use` with a chance (training stream, player only); each effect read in one place through `Actor::skill_effect`; creature types may list `skills` with fixed values (114 of 273 do, nearly all Concentration and Spellcraft; one Listen); "trained at all" is `Skills::trains_effect` (Spot, Steal, Heal, Pick lock) |
| The thief | class skills Open lock, Steal, Stealth, Tumble (known for), Appraise, Disable device, Escape artist, Search, Spot; Daggers 2, Thrown 1; a dagger style; the table: 1 Sneak attack + Trained wind, 6 Trapfinding, 12 Pick lock, 18 Evasion, 25 Uncanny dodge, 32 Improved sneak attack (grants Openings), 40 Light fingers, 50 Hide in plain sight, 60 Master thief (threat −2 and the finisher, `use:master_thief:E`, 5 openings); bonus picks from Shadow, Finesse, Delver at 4, 10, 16, 22, 28, 34, 46, 58 |
| The other rogue-types | the ranger (Favoured prey, Tracker, Camouflage, Rapid shot and Volley on Focus; Bows 2, Short blades 1, Marksman); the bard (Lore, Inspire, Throw your voice; Short blades 1, Daggers 1d2, Duelist); the monk (Flurry, Still mind, Quivering palm). No assassin class, and none is added here |
| Creatures that ought to ambush | the drow assassin ("the houses' knife in the dark ... a sure eye for the opening that kills"), the drow stalker ("opens a throat with a dagger"), the quickling ("cutting with a little knife before it is seen"), the meenlock ("holds it with paralysing claws"): their descriptions promise what no rule gives them; speed and venom are all they have |
| Talents | the `awareness` system ("Creature sight and awareness") is in `talents.toml` **`pending`**, with Ambusher alone in it; `stealth` holds Ambusher, Hush-foot, Silent feet, Camouflage, Hide in plain sight and the rest; the Shadow tree is Stealth, hearing, eyes, and light; the coverage bar is 2 talents a system |
| Traps | `trap.rs::step_onto_trap` for the player's and creatures' steps; a hidden trap springs with no warning; nothing of it stays on the victim but the harm |

**The gap**, in one line each: creatures never surprise the player; surprise is not a state, so a creature caught unawares fights back at full Defence and full speed from its next action; the sneak attack is three flat talents, not a skill that grows; no class but the thief has any of it; no creature has any of it; nothing about surprise is told, so the player cannot tell a lucky big roll from an ambush that worked.

## 3. Surprise: the model

### 3.1 Unaware

`World::unaware(target, attacker)` becomes the one definition for everyone and is asked of **every** attack (today only of an attacker with ambush dice; it draws nothing, so the wider ask changes no hash):

| Target | Unaware of the attacker when |
|---|---|
| A creature | as today: asleep or disguised, whatever it sees; else it hunts no one (`Mind::last_seen` unset) **and** its eyes do not see the attacker's cell now (`World::sees_now`). A creature that has lost sight of the player but is still searching (`last_seen` set, until it gives up and its mind goes calm) is **aware**: it knows something is about, and D&D's "aware but cannot see" is not surprise either |
| The player | the attacker is not among the actors of the player's **current perception** (the snapshot of their last turn's end, `World::perception`), **and** `Mind::noticed` on the attacker is `None` or more than `memory_turns` (20) turns ago. `World::perceive` sets `noticed = Some(state.turn)` on every creature the snapshot sees (in the turn pipeline only, never on `from_state`'s recomputation, which must not touch state). A fast creature that walks into view and strikes in the same creature turn, a shot out of the dark from something never seen, a sleeper the player walks past that wakes and bites: surprises. A creature seen a few turns ago that steps behind a pillar and out again: not |

Nothing else is new here: the dark, a corner, a closed door, a thrown voice (the creature goes to look and stays unaware), a sleeper, and a mimic are all ways an attack finds its target unaware, by the rules that exist.

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
| The next action costs 150 % | the status's `cost`, read by `Effects::slow` in `World::burdened` as every status's is (no code) |
| Every attack on it is a sneak attack for an attacker with dice | `resolve` step 4 |
| It ends with the sufferer's next action | `per_action`, `World::spend_actions` (no code) |

**How long, and stacking.** Until the surprised actor's next action, which is the dear one; a second surprise before it acts refreshes the one degree (one dear action however many foes fell on it); Staggered and Surprised multiply as any two slowing statuses do (225 %: rare, and the stagger was earned). A creature of speed 2000 pays its dear action and is itself again on the second of its two turns.

**Traps as a source**: `trap.rs::step_onto_trap` (the player's and creatures' steps alike) calls `startle(victim, None, 0)` when a **hidden** trap springs (a known trap stepped onto on purpose surprises nobody): the check against 13 with no craft, then the status. The sense of it: a pit opening under a thief with a drow assassin beside them is the assassin's opening; and a goblin that springs a trap beside the player is the player's.

**Symmetry.** Nothing above asks who is the player. A creature with Sneak attack in its `skills` (§7) rolls its dice on a surprised player; a creature caught by a trap is flat-footed to the player's blow; the player's Initiative skill and a creature's are read by the same line.

### 3.3 Data: `data/surprise.toml`

```toml
# Surprise (spec §8 "Surprise and sneak attacks"). Optional: without it nobody is surprised and
# no sneak attack dice land (fixtures).
[surprise]
status = "surprised"      # the status an actor caught unawares suffers (status.toml)
difficulty = 13           # the initiative check's difficulty before the attacker's craft
missile_reach = 6         # how far a shot, a throw, or an attack spell is still a sneak attack
memory_turns = 20         # how many turns the player stays aware of a creature they have seen
# What has no vitals to find: no sneak attack dice and no lowered threat land on an incorporeal
# creature, an animated object, or a type carrying one of these tags, or one that says
# `vitals = false`; it is surprised like anyone.
no_vitals = ["elemental", "skeleton", "spirit", "construct"]
```

`content/surprise.rs`: `SurpriseRules`, `Content::surprise()`; the status checked to be `per_action` with `max = 1`; the tags checked to exist on some type (as a quest's tag is). A creature type gains `vitals` (default true; `CreatureDef::vitals`), and `Content::has_vitals(type)` is the one reader of the three rules.

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

It sits in the skills list after Search (the list is the Skills screen's order; "S" after "Search" keeps the alphabet the file already follows). `skills::Effect::SneakDice` and `Effect::Surprise`; `Use::SneakAttack` (a sneak attack of the player's that hits, reported in `ambushed` where the dice land, beside the `Build`).

| Rule | Decision | Where |
|---|---|---|
| **The dice** | `World::sneak_dice(attacker)` = `Dice::of(skill_effect(SneakDice), 6)` `and` the talents' `ambush` (Ambusher 1d6, Improved sneak attack 2d6: `TalentMods::ambush` as today). `World::sneaking` becomes "`sneak_dice` is some, and the target is unaware or Surprised". `ambushed` rolls them on a hit as today: after Protection, never soaked, never multiplied by a critical (D&D's precision damage is not multiplied, and the code already does this), of the blow's own kind | `ambush.rs` (`sneaking`, `ambushed`), one read of `SneakDice` |
| **The craft** | the `surprise` benefit, the attacker's part of the check's difficulty (§3.2) | `startle`'s `craft`, one read of `Surprise` |
| **What counts** | a melee blow at any reach (a lunge, a reach blow), the off hand's blow, a shield bash, a kick, a free blow; a shot, a throw, or a spell that rolls to hit **within `missile_reach` cells** (Chebyshev distance from attacker to target, 6; D&D's 30 ft); a burst, a bolt that does not roll to hit, a trap's missile, a status tick: never. The dice ride the attack's `Profile` path, so every blow path gets them from the one hook | `resolve` (the distance check in `sneaking` from `from`/`to`), `cast_single` |
| **Which weapons** | any; bare hands too. The thief's daggers get their flavour from the Daggers weapon ability (§6.4) and the two thief styles, not from a restriction (a restriction would make a sneak attack with a found long sword feel like a bug) | — |
| **Who is immune** | **no vitals** (§3.3): incorporeal creatures (`Content::incorporeal`), animated objects (`CreatureDef::animated`), the `no_vitals` tags (elementals; `skeleton`, the walking bones; `spirit`, the incorporeal dead, already incorporeal; `construct`, the mud golem and every animated object), a type with `vitals = false`. The fleshy undead (the draugr, the ghouls, the drowned, the vampires, the mummies) **have** vitals: the barrow and the minster are the Ascmearc's first dungeons and a thief needs a game there. No dice, no Master thief threat; still Surprised | `Content::has_vitals`, read in `sneaking` and `sneak_threat` |
| **Criticals and the 2d12** | the attack roll is the ordinary one against the stripped Defence; Master thief's threat −2 as today; a confirmed critical multiplies the weapon's dice and bonus as today and adds the sneak dice once after. Vital strike (a sure critical) on an unaware foe: both, as today with the talents' dice | `combat::strike_with` unchanged |
| **Stealth** | unchanged: Stealth is quiet steps and doors (how one gets there); Sneak attack is what the blow does once there. Ambusher keeps its Stealth 30 prerequisite. Hide in plain sight's quieter attacks mean a sneak attack is less likely to wake the sleeper's neighbours, as today | — |
| **Weapon skills and styles** | the dice are added after `Profile::with(benefit)` and the strike, so ranks' to-hit, damage, threat, and speed apply to the attack and never to the dice; the sneak attack trigger (§6.4) fires for the attack's skills | — |
| **Openings and Patient blade** | unchanged: `Build::SneakAttack` on a hit, `sneak_kill` on a kill | — |
| **Marks** | `Use::SneakAttack` (50 %) when the dice land; a Dexterity stat mark too (`[[training.use]] stat = "dexterity", on = "sneak_attack", chance = 20`, `StatUse::SneakAttack`, `data/stats.toml`) | `ambushed`, `World::train` |
| **The character screen** | the value only, as any skill; the attacks list (`OwnAttack`) does not show the dice (situational changes are left out of it already: ambush is named there) | — |

**Numbers.** A level-1 thief (2d6+10: 12 to 22) has 1d6; at 25, 2d6; the thief is known for it (its rolls throw a bigger die), with 3 rolls a level, so about 100 by level 25 to 30 (5d6), then Improved sneak attack's 2d6 at 32 (7d6), Ambusher's 1d6 (8d6), and 200 around level 50 (10d6 in all: D&D's rogue at 20). Against the probe's monsters (`docs/balance.md`): a level-30 thief's dagger blow does about 12 after Protection; 5d6 (17.5) more on an unaware foe roughly triples the first blow of a fight, which is the fantasy and no more, since the second blow meets an aware foe at full Defence. The owner accepts unbalanced choices; S7 adds a row to the balance probe so the number is watched.

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
# made, you are not surprised. +8 at 100, +12 at 200.
benefit = [{ effect = "initiative", curve = [[0, 0], [100, 8], [200, 12]] }]
```

In the list after Heal. `Effect::Initiative`, read in `startle` and nowhere else; `Use::Surprise` (a check made, whatever comes of it; the player's only, as every mark).

| Question | Decision |
|---|---|
| The stat | **Dexterity** (D&D's; reflexes). Wary (Willpower 12) is the wits' way in: +4 flat |
| What a made check buys | **no Surprised status**: full Defence from the next attack on, no dear action, free blows kept. The surprising attack itself was still rolled flat-footed and its dice still land (the attacker's craft is the attacker's; Initiative is the victim's recovery, which is what the owner asked for). §13 Q4 offers the other reading |
| Odds (the wiki states them) | 2d12 + modifier against 13 + craft: Dexterity +0 against an untrained attacker, 78 of 144 (54 %); +2, 69 %; +0 against a master thief's +8 (21), 10 of 144 (7 %); Initiative 100 (+8) against 13: 138 of 144 (96 %). A sleeper gets no check |
| Who trains it | §6.1: the quick classes much (thief, ranger, monk), the fighters a little (warrior, barbarian), the bard a little; nobody else. Creatures: §7 |
| An initiative **order** | **rejected** (§11): the energy scheduler decides who acts when, exactly and hidden; ties go to the player then to ids; speed is the one number. A "who acts first when a fight starts" rule would either restate the scheduler or add a second clock. Initiative is the check and only the check |
| Beyond surprise | nothing: a passive skill with one hook. If a second use ever wants it (a reaction to a trap's missile, a dodge of a falling block), it is a benefit added here, not a new skill |

## 6. Class fantasy: classes, class abilities, talents, weapon abilities

### 6.1 Classes (data)

| Class | Sneak attack | Initiative | Why |
|---|---|---|---|
| Thief | `2d6+10`, **known for** | `2d4+4` | the whole fantasy |
| Ranger | `2d4+4` | `2d6+10` | a hunter's first arrow from cover, and the one who is never caught out in the wild; `known_for` unchanged (spot, survival, woodcraft: #662 may revisit) |
| Bard | `2d4+4` | `2d4+4` | a quick knife and a quick tongue; lesser |
| Monk | — | `2d6+10` | Still mind's class; no sneak attack (the monk's fantasy is the open fight) |
| Warrior, barbarian | — | `2d4+4` | drilled or wild reflexes; a little |
| Paladin, priest, druid, wizard, witch, aristocrat, commoner, craftsperson, merchant | — | — | untrained: the check at Dexterity alone |

Races unchanged (no race trains either; the halfling's and drow's Stealth stays their edge). Creature types: §7.

### 6.2 The thief's class abilities

The table keeps its ten slots and its talent ids (saves name them; `class::grant` gives a grown table's new entries to old saves):

| Level | Talent | Change |
|---|---|---|
| 1 | **Sneak attack** (`sneak_attack`, granted, class tree) | its `ambush 1d6` **moves into the skill** (§4); the talent becomes the class passive that goes with it: `effects = [{ kind = "grants_resource", resource = "openings" }, { kind = "surprise_difficulty", value = 2 }]`. Description: "You find the telling spot in a foe that has not seen you coming (Sneak attack), and those you catch unawares are slower to gather their wits; each sneak attack that hits gives you an opening (Openings: they slip away with time)." The Openings pool is held from level 1 (today from 32), so Low blow at 25 has something to spend |
| 1 | Trained wind | as today |
| 6 | Trapfinding | as today; **Lurk** joins the level (a second entry at 6, as the warrior has two at 25) |
| 6 | **Lurk** (`lurk`, class active, `use:lurk`, aim nothing) | a wait in ambush: `afflict(player, lurking, 1)`, priced as a wait (`price_of(Wait)`), no pool, no cooldown (time is the price). **Lurking** (`status.toml`: `max = 1`, `refresh`, `per_action`, no `seen`): while it lasts, a hostile that steps up beside the player takes the player's blow at once (the gatekeeper's free blow, `Cue::Approaches` in `World::step_up`, switched on by the status: one line in `World::gatekeeper`), and **is caught unawares by it unless it was already hunting the player** (`Mind::last_seen` set): `startle` is called for it as for any unaware target, so the blow is a sneak attack and the creature may be Surprised. The status ends with the player's next action (the free blow is not an action of theirs). Refused with no time when a hostile already stands beside them ("Too late to lurk.") |
| 12 | Pick lock | as today |
| 18 | Evasion | as today |
| 25 | Uncanny dodge | as today; **Low blow** joins the level |
| 25 | **Low blow** (`low_blow`, class active, `use:low_blow:E`, aim direction, `resource = { id = "openings", cost = 3 }`) | sand in the eyes and the knife where it hurts: a melee blow at the hostile that way (beside the player), priced as the blow is (`attack_price`), that **is a sneak attack whatever the foe knows** (the dice land on a hit; the foe is not Surprised: it saw the sand coming, and it keeps its Defence). `World::low_blow` sets a transient `Technique::sneak` (as Feint sets `ignores`) that `sneaking` takes. Greyed out "not enough openings" while short; refused with no time without a hostile there |
| 32 | Improved sneak attack | keeps `ambush 2d6`; loses `grants_resource` (moved to level 1; `Talents::regrant`/`class::grant` leave an old save's pool as it is). Description: "Your sneak attacks hurt far more." |
| 40 | Light fingers, 50 Hide in plain sight, 60 Master thief | as today |

Both actives are `ClassActive`s in the one registry (`Ability::Class`, `Source::Class`), rows of the abilities menu ("Lurk (class)", "Low blow (class): 3 openings, a direction"); no key of their own; the action text is `use:lurk` and `use:low_blow:E` (`Action::Use`), `kind_of` prices them as `wait` and `melee` for the statuses that block.

**The ranger, the bard, the monk** get the skills (§6.1) and no table change: the ranger's Ambusher-by-pick, Camouflage, Tracker, and Marksman's Hunter's shot (§6.4) are its ambush; the bard's Throw your voice is now the tool it was always meant to be (the creature it sends to look is unaware when the bard's knife finds it). Keeping their tables as they are keeps S2 small.

### 6.3 Talents

Two systems. **`awareness`** exists `pending` in `talents.toml` ("Creature sight and awareness"): S1 takes the flag off with the first two. **`sneak_attack`** is new: `[[system]] id = "sneak_attack", name = "Sneak attacks: dice, reach, vitals"`, placed after `stealth`; S2 adds it with its talents and retags the existing four.

| Talent | Tree | Needs | Effect (new `TalentEffect` where marked) | Its one hook | Systems |
|---|---|---|---|---|---|
| **Wary** | Delver | Willpower 12 | `initiative_bonus = 4` (new) | `startle`'s modifier | `awareness` |
| **Cool head** | Grit | Willpower 14, level 9 | `unshaken` (new: Surprised slows no action of yours) | `Effects::slow` through `TalentMods::penalty`-style exemption, `TalentMods::ignores_cost(status)`, read once where `cost` is multiplied in | `awareness`, `status` |
| **Stalker's eye** | Shadow | Stealth 20, Spot 10 | `sees_unaware` (new) | `Perception::compute`: `SeenActor::unaware` set from `World::unaware(creature, player)` for the seen creatures when the player holds it, else always false; look mode's word (§8) | `awareness`, `sneak_attack` |
| **Hair trigger** | Finesse | Dexterity 14, Initiative 20, level 12 | `startled_blow` (new) | `startle`: a check **made** by the player against a hostile attacker beside them strikes it back at once, a free blow (`Cue::Startled` in `World::second_blow`, `Free::Startled`, held as the out-of-turn ones are); one per surprise; it brings no free blows of its own | `awareness`, `free_blows` |
| **Anatomist** | Scholar | Intelligence 13, Knowledge 20 | `vitals_in = ["skeleton", "construct"]` (new: tags) | `Content::has_vitals` takes the attacker's talents: those tags count as having vitals for them (never incorporeal, never elementals) | `sneak_attack` |
| **Long reach** | Weaponcraft | any launcher or thrown skill at 4 | `sneak_reach = 3` (new) | `sneaking`'s distance check: `missile_reach` + the attacker's | `sneak_attack`, `ranged` |
| Ambusher | Shadow | (as today) | `ambush 1d6` | as today | `stealth`, `awareness`, + `sneak_attack` |
| Improved sneak attack, Master thief, Patient blade, the level-1 Sneak attack | class | — | as §6.2 | — | + `sneak_attack` |

Coverage: `awareness` four talents and two class abilities (Sneak attack's `surprise_difficulty`, Lurk); `sneak_attack` three talents (Anatomist, Long reach, Ambusher) and four class abilities. Both well over the bar of two. `wiki/src/generated/talents-by-system.md` updates itself.

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

Everyone else: no dice, the check at Dexterity alone (a kobold at Dexterity 10 shrugs off half the thief's surprises; the thief's craft lowers that as it grows). Numbers are the roster PR's to tune; the wiki table (§10) shows them. Per the creatures rule, these go in the per-tribe roster notes (`docs/dev/creatures/*.md`), not `creatures.md`.

**Lurking** (one AI hook, `ai.rs::fight`): a type with `lurks = true` (`CreatureDef::lurks`, default false) that sees a hostile **does not close in**: it waits where it stands (`Plan::Wait`; its mind still records `last_seen`, so it is aware) until the target is beside it (then it attacks as usual), or until it has been harmed (`Actor::hp < max_hp`: then it hunts as any creature). A ranged lurker shoots from where it stands if it can. The meenlock and the barrow spider lurk. It is deterministic and draws nothing; it makes a corridor's dark alcove a place where a thing waits, which is what the two descriptions say. A lurker that sees the player is **aware** (it is hunting): walking into it is not a surprise for the player; being bitten by one they never saw is. Whether a lurker's wait is coarse while the player rests follows `stride.rs` as any wait does.

**No creature uses Lurk or Low blow** (actives are the player's, as the weapon actives are; creatures hold no Openings); the drow assassin's fantasy is its 6d6 and speed.

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
| Hair trigger | the free blow's own line ("You stab the drow stalker.") after "You keep your wits about you." | `strike_free` |
| Lurk | "You wait in ambush." (`lurking`'s `start`; it ends silently); the free blow's lines as it comes | `use_class` |
| Low blow | the blow's line; refused: "There is no one there to blind." | `World::low_blow` |
| Look mode | "A kobold (hostile, unhurt, surprised), on stone floor." while the status lasts (`seen = "surprised"`: the player saw it flinch); "asleep" as today; **with Stalker's eye**, "A kobold (hostile, unhurt, unaware of you), ..." from `SeenActor::unaware`; without it, never a word about awareness | `ui/look.rs` |
| The status bar | `Surprised` among the player's effects while it lasts; `Lurking` likewise | `own_effects` |
| The Skills screen | Sneak attack and Initiative with their values, as any skill; never the dice or the bonus | `own_skills` |
| The abilities menu | Lurk and Low blow under the class source; Sneak attack and Initiative are not listed (neither is of `kind = "action"`) | `Ability::every` |
| The character screen's attacks | unchanged (the ambush is a situational change, left out by rule) | — |
| The death screen | unchanged ("Killed by a drow assassin"); the surprise is in the log before it | — |
| Dumps | `actors` (dev): `unaware of player` / `aware` per creature, `noticed turn N`, and `lurks`; the `effects` of the header and of seen actors carry `surprised` and `lurking` as any status; JSON `effects`; `skills` layer the two values; `abilities` the two actives | `dump.rs` |

**What never leaks**: the check's roll, modifier, and difficulty; the dice; a creature's `last_seen`, `noticed`, or `sees_now` outside the dev layer; which creatures have vitals (the wiki says; the game says it only by the damage). The perception snapshot gains one field, `SeenActor::unaware: bool`, always false without Stalker's eye, computed in `Perception::compute` from the rules (as Mimic-wise's reveal is), so the UI reads the snapshot and nothing else.

## 9. Determinism, saves, replays

| Concern | Decision |
|---|---|
| **Streams** | the initiative check: two draws of the **gameplay** stream (`check::check`), in `resolve` **after** `favoured`/`bane`/`sneak_threat` shape the profile and **before** `combat::strike_with`'s draws; for a spell, in `cast_single` before its roll; for a trap, in `step_onto_trap` after the trap's own draws. The sneak dice: gameplay, after the strike, as today. Marks (`Use::SneakAttack`, `Use::Surprise`, the Dexterity mark): the **training** stream, the player only. Lurk's and Low blow's blows draw as blows. `unaware`, `has_vitals`, lurking, the Defence strip: no draws |
| **When draws happen** | only when an attack finds its target unaware, or a hidden trap springs on an awake actor. A fight that begins in plain sight draws exactly as today; `no_draws_while_nothing_is_unaware` compares every hash of such a fight with and without `surprise.toml` |
| **Hashes that move** | every run whose first attack found a creature unaware (the showcase's thief against the depths' patrol, the arena's dark corner, the golden replays that open with an ambush): two draws more from that step on. The snapshot tests and replays of `ambush`'s and `den.rs`'s tests are re-made in S1 and reviewed; the golden manifests' replays are rebuilt by `cargo xtask golden` as always (no new golden entry: nothing here is a GPU effect) |
| **Saved state** | the statuses (`surprised`, `lurking`: `StatusId` indexes join the fingerprint, as every status); **`Mind::noticed: Option<u64>`** on creatures (serde-skipped while `None`; hashed): old saves load with `None` (every creature a stranger again: harmless). The skills are saved by id (new ids, nothing shifts). `CreatureDef::vitals` and `lurks` are content. `Technique::sneak` is transient. `FORMAT_VERSION` is the integrator's; the PRs say "saves change" |
| **Replays** | `use:lurk`, `use:low_blow:E` are `Action::Use` as the other class actives; old replays carry none and load; a replay of the menu reads back the same text |
| **Perception and state** | `perceive` writes `noticed` in the turn pipeline only; `from_state` recomputes the snapshot without writing it, so a save loaded and a run continued agree |
| **Content fingerprint** | the two statuses and the two skills' use/effect ids are in it as their kinds are; the talents are not (named by id) |

## 10. The wiki, the spec, and the docs

| Where | What |
|---|---|
| `wiki/src/surprise.md` (new, under Combat after Fighting styles in `SUMMARY.md`) | "Surprise and sneak attacks": the rule in words (unaware, surprised, flat-footed, the dear action, the check, what counts, what has no vitals), with generated tables from `xtask/src/wiki/surprise.rs`: the Sneak attack ladder (value → dice, from `skills.toml`), the Initiative ladder, the check's odds by Dexterity edge and craft (`check::ways`, as `stealing.md`'s table), the creatures that sneak-attack or recover (from creature data: type, level, dice, bonus; and which lurk), what has no vitals (the tags and the types), the statuses' numbers (150 %), `missile_reach` and `memory_turns` |
| `wiki/src/skills.md`, `classes.md`, `talents.md`, `weapon-skills.md`, `fighting-styles.md`, `abilities.md`, `creatures.md` | the existing generated tables pick up the two skills, the class dice, the talents, the two weapon abilities (by their rows), the thief's actives, and the creature skills; prose lines where a page explains |
| `spec/08c-combat.md` | a new bullet **Surprise and sneak attacks** after Free blows: the model of §3, the skill of §4, the check of §5, what counts, vitals, the messages |
| `spec/08i-character.md` | Skills: the two entries; the thief's table bullet: the level-1 change, Lurk, Low blow; Talents: the six; Weapon abilities: the trigger and the two |
| `spec/08b-creatures.md` | Creature behavior: `lurks`; the `skills` note gains the two skills; `vitals` |
| `spec/07b-data.md` | `surprise.toml`, the creature fields, the status fields in use |
| `docs/dev/surprise.md` (new topic; listed in CLAUDE.md after `stealing.md`) | modules (`world/surprise.rs`: `startle`, `unaware` moved there from `ambush.rs` with `sees_now`; `ambush.rs` keeps the dice), data, invariants (one definition of unaware, one place anyone is surprised, the draw order, the no-draw rule), tests |
| `docs/dev/combat.md`, `creatures.md`, `skills.md`, `progression.md`, `weapon-abilities.md`, `status-effects.md`, `traps.md` | one line each at the hook: the strip in `target_against`, `lurks` in the AI overview, the two effects and uses, the talent hooks table rows, the trigger row, the two statuses, `startle` in `step_onto_trap` |
| Tests | `world/surprise_tests.rs` (both ways round: the player surprised by a creature from the dark and not by one seen lately, the check's two outcomes, the strip, no free blows, the dear action, traps, sleepers without a check, mimics, the messages, saves and replays, the no-draw hash test), `ambush_tests` reworked for the skill, `class_rogue_tests` for Lurk and Low blow, `weapon_ability_tests` for the trigger, `content::surprise` parse tests, the screen snapshot of the abilities menu with the thief's actives, the wiki test |

## 11. Choices made along the way, and what is rejected

| Choice | Decision and why |
|---|---|
| **A whole lost action** (D&D's surprise round) | rejected: the owner's "more expensive first action" is the Warlock reading of it, and 150 % on the energy scheduler is exactly half an action lost; a full turn lost (300 %, Drowsy's) would make every ambush by a fast creature a near-certain second free blow, and the player's death screen would often read "surprised, then dead". Data, if the owner wants 200 |
| **Flat-footed loses Parry and Shield too** | rejected: D&D keeps armour and shield against the unseen blow; a shield held up stays held up. Dexterity and Dodge go (the categories that are *reacting*); Armour, Shield, Parry, Natural, Deflection, Wall stay. Feint takes Parry and Shield, so the two together strip nearly everything, which is what a feint on a surprised foe deserves |
| **Attacks of opportunity** | not added: the free blows are Warlock's reactions already (riposte, opportunist, gatekeeper); surprise turns them off, which is the one thing 3.5's flat-footed says about them |
| **Flanking** | rejected: the player has no allies (summons are enemy-only), so flanking would be a rule that only ever works against the player, and being surrounded already costs enough. If allies come, "flanked" is one more door into `sneaking` beside unaware and Surprised, nothing else changes |
| **An initiative order** | rejected (§5): the scheduler is the order |
| **A hiding or noticing contest** (Stealth against a creature's Spot when it first sees you; "noticed late") | rejected here: creature sight is a pure function of light and lines (`sight.rs`, kept equal to the player's perception by a property test), and the AI draws nothing; a hiding mechanic is its own design (where to hide, what breaks it, how creatures search) and would change every creature's first move. The dark, the corner, the door, the sleeper, and the thrown voice are the ambush tools this design builds on. Recommended for the deferred list (#196) as "hiding and spotting", not decided here |
| **Which creatures have vitals** | the fleshy undead do (§4): the Ascmearc's first two dungeons are theirs; the bones and the bodiless do not. Anatomist buys the bones back |
| **Sneak attack dice multiplied by a critical** | no (3.5's precision rule; the code already adds them after) |
| **A separate "flat-footed" status and "surprised" status** | one status: Surprised is flat-footed until the next action, and the next action is dear; two statuses would be two timers for one moment |
| **The thief's level-1 talent renamed** | kept its id `sneak_attack` (saves name talents by id; a rename is a migration for nothing) and its name; only its effects move. The skill shares the name, which reads as intended: the class ability "Sneak attack" is what the thief makes of the skill |
| **Openings from level 1** | yes: the pool was granted at 32 because nothing before Master thief spent it; Low blow at 25 spends it, and a level-1 thief watching openings tick up from their first ambush is the fantasy at level 1 |
| **Creatures lurking by a draw** ("a meenlock waits 60 % of the time") | no: no creature behaviour draws; `lurks` is a rule of the type |
| **The player's awareness as a per-creature turn stamp** rather than a set on the player | the creature's `Mind` is where the creature's half lives; the player's half about that creature lives beside it, serde-skipped while `None`; a set on the player would need pruning as creatures die and levels change |
| **Memory of 20 turns** | long enough that a creature seen and lost behind a pillar is not a surprise when it comes round; short enough that the thing seen an hour's rest ago out of the corner of an eye can surprise again. Data |
| **Initiative on Willpower** | Dexterity (3.5); Wary is the Willpower character's way to the same place, and Cool head's prerequisite is Willpower: the wits' classes are covered by talents, the reflexes' by the skill |
| **The spell path** | an own cast that rolls to hit surprises and sneak-attacks (a wizard's bolt from the dark); a burst never (no roll, no telling spot); a scroll's or a wand's attack spell rolls to hit too and goes through `cast_single`, so it counts as well: a thief reading a scroll of lightning from the dark is still a thief |

## 12. Implementation plan

Seven issues in three waves; each links to its section. "**Action**" marks a PR that adds an action, "**State**" one that adds saved state; at most one of each at a time, and nothing here overlaps the affixes and chests waves in flight beyond `fight.rs::resolve`'s few lines (the false-conflict trick in `docs/dev/merging.md` for the hot files: `status.toml`, `skills.toml`, `talents.toml`, `Trigger::ALL`).

| Wave | Issue | What | Depends on |
|---|---|---|---|
| 1 | **S1 Surprise** (§3, §8, §9) — **State** | `data/surprise.toml` and `content/surprise.rs` (without `no_vitals` yet, nothing reads it); `world/surprise.rs`: `unaware` for the player (`Mind::noticed`, set in `perceive`), `startle` with the check at Dexterity alone (no skills yet), the status `surprised`, `Target::without` shared with Feint, the strip in `target_against`, free blows off, the messages, traps as a source, sleepers and mimics without a check, dumps; `awareness` off `pending` with **Wary** and **Cool head**; the talent hooks; `docs/dev/surprise.md`; the spec bullet; tests and the hash test; the re-made snapshots listed in the PR. `ambush.rs` keeps the dice and is asked for them as today | — |
| 1 | **S2 The skills** (§4, §5, §6.1, §6.2 data only) | `sneak_attack` and `initiative` in `skills.toml` with `Effect::SneakDice`, `Surprise`, `Initiative` and `Use::SneakAttack`, `Surprise`; `sneak_dice` from the skill and the talents; `missile_reach` and the spell path; `no_vitals`, `vitals`, `has_vitals`; the Dexterity stat use; the classes' dice; the thief's level-1 talent reworked (openings from 1, `surprise_difficulty`), Improved sneak attack's grant moved; the `sneak_attack` system with **Anatomist** and **Long reach**, the four retagged; `creatures/README.toml`; the skills' wiki rows; `class_rogue_tests` | S1 |
| 2 | **S3 Lurk and Low blow** (§6.2) — **Action** | the two `ClassActive`s, `use:lurk` and `use:low_blow:E`, the `lurking` status and its gatekeeper line, `Technique::sneak`, the menu rows and the screen snapshot, the action text, replays | S2 |
| 2 | **S4 The weapon abilities** (§6.4) | the `sneak_attack` trigger in `ambushed`, **Knife in the dark**, **Hunter's shot**, the wiki rows, `weapon_ability_tests` | S2 |
| 2 | **S5 The creatures** (§7) | `skills` on the eight types, `lurks` on two with the AI branch, the roster notes per tribe, `bestiary_tests`, the wiki's creature table | S2 (S5 may merge beside S4; neither adds an action or state) |
| 3 | **S6 Stalker's eye and Hair trigger** (§6.3) | `SeenActor::unaware` and look mode's word; `Cue::Startled`/`Free::Startled`; the two talents and their tests | S1 (Hair trigger needs Initiative's prerequisite from S2) |
| 3 | **S7 The wiki page and the balance row** (§10) | `wiki/src/surprise.md`, `xtask/src/wiki/surprise.rs` (the ladders, the odds, the creatures, the vitals), the balance probe's sneak-attack row in `docs/balance.md`, a cross-check of the spec against what landed | S1–S5 |

Each issue answers the feature template's "Talents and class abilities": S1 (`awareness`: Wary, Cool head; the class passive), S2 (`sneak_attack`: Anatomist, Long reach, Ambusher; the thief's Sneak attack, Improved sneak attack, Master thief), S3 (two class actives, carrying `sneak_attack` and `awareness`), S4 (the abilities carry `weapon_abilities`, `sneak_attack`, `status`), S5 and S7 (content and docs: none, extending the systems above), S6 (the remaining two talents).

## 13. Open questions, each with a recommended default

None blocks approval: each has a default the plan is written against.

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
| Q13 | **Openings from level 1** (so Low blow at 25 has a pool), moving the grant off Improved sneak attack? | **yes** (§11) |
| Q14 | **Lurking creatures**: the meenlock and the barrow spider only, or the mimics' kin and more? | **those two** in S5; a roster PR may mark more, one line each |
| Q15 | **A hiding and spotting contest** (Stealth against Spot when a creature first sees the player) as a later design? | **defer to #196** as "hiding and spotting" (§11); this design does not need it |

# Warlock — Combat progression: weapon skill and style ranks and their abilities, spell scaling by stats, monster Protection, stat growth between levels, potions and elixirs

Design document for the owner's review (2026-10-27). Written against `origin/main` at 3eb489d7, the balance investigation of the same day (the probe over all 400 creature types and four classes at levels 1 to 60; its findings are summarised in §2), the approved progression design (`docs/design/progression.md`, §6 for stat training) and magic design (`docs/design/magic.md`, §5 for the caps), and the systems as built (`docs/dev/weapon-skills.md`, `combat.md`, `spells.md`, `stats.md`, `status-effects.md`, `resources.md`, `magic-items.md`, `scrolls.md`, `hunger.md`). Nothing here is implemented; §15 cuts it into slices. **Every number is a first guess, kept in data**, with its reasoning beside it; none is a balance decision. The owner's scope note: **none of this goes into 0.7.0**.

**The owner's request** (2026-10-27, verbatim): "feature: address balance issues discusses earlier - make higher ranks in weapon skill more powerful. biggen numbers and perhaps some abilities. - same for fighting styles. They should also give some abilities. - the abilities weapon skill and fighting styles could be something like riposte (free attack when parrying enemy attack), rend (extra attack worth of damage when both weapons of two-weapon fighting hit) etc. sword and board mayby give enemy penalty to next attack speed after they get parried etc. Triggered abilities, and active abilities activetable from ability menu. - spells perhaps should perhaps pick up benefits from multiple mental stats to allow better scaling. this is just something that needs to be authored per-spell. - increase PV of higher level monsters (just hand tweak per monster) - make sure stat training marks are plentiful, and give stats change to grow even between level ups - introduce potions. healing, cures, temp stat boosts, visiontypes (ultravision) Each potion gives also training marks. Also introduce elixirs that increase stats by 1 (also increase potential by 1 or more? 1d2?) this does not go into 0.7.0".

## Contents

1. Summary: the decisions needed
2. What exists today, and what this builds on (the balance findings)
3. Weapon skills: the new per-rank tables
4. Rank abilities: triggered and active (the system)
5. Fighting styles: tables and abilities
6. The catalogue, interactions, and abuse cases
7. Spell scaling from the mental stats
8. Monster Protection by level and body
9. Stat training: plentiful marks, growth between levels
10. Potions and elixirs
11. Talents and class abilities (the coverage bar)
12. Saves, ids, determinism
13. Tests, text snapshots, and the wiki
14. What is deliberately not in this design
15. Implementation plan
16. Open questions, each with a recommended default
17. Owner decisions

## 1. Summary: the decisions needed

| # | Piece | Proposal (the default if the owner only nods) | Section |
|---|---|---|---|
| 1 | Weapon skill ranks | ranks 0 to 5 as today; ranks 6 to 10 grow faster. Rank 10 to-hit +7 / +6 / +5 by family (today +5 / +4), damage +5 / +8 / +12 (today +3 / +4 / +6), threat up to 3 wider (today 2), speed up to 30 % (today 20), Parry up to +6 (today +4). The to-hit stays moderate so that trained stats (§9) do not saturate the 2d12 curve; the growth goes into damage, where monster Protection (§8) can answer it | §3 |
| 2 | Rank abilities | a new registry, `data/weapon_abilities/<id>.toml`, one file per ability: **triggered** ones fire on one of ten combat events through one hook (`World::weapon_trigger`), **active** ones are rows of the abilities menu's existing *Weapon skills* group, aimed as class actives are, paid with a **cooldown status** (the martial actives' cost model) and never a key of their own. Unlocked at ranks 5 and 9 of a weapon skill, 4 and 7 of a style | §4 |
| 3 | Styles | their numbers grow at ranks 6 to 10 as the weapon skills' do (Duelist Parry to +7, Great Weapon damage to +10 and a ×4 critical at 10, Twin Blades penalty-free at 9, Sword and Board bash 35 %); each style has one or two abilities, the owner's three among them: **Riposte** (Duelist 4, the existing free blow switched on by the style), **Rend** (Twin Blades 4), **Staggering block** (Sword and Board 4) | §5, §6 |
| 4 | The catalogue | 18 abilities: 10 on the weapon skill families, 8 on the styles; every one on an existing mechanism (free blows, statuses, sure criticals, the one blow path). No chains: a free blow brings none, Rend is once per attack, Staggered refreshes instead of stacking | §6 |
| 5 | Spell scaling | an authored `[spell.scales]` table per spell: four dimensions (`damage`, `to_hit`, `reach`, `duration`), each a map of stat → weight in percent of the modifier. Damage scaling **lifts the cap**: the tier ceiling grows by the scaled percent up to a `max_lift` per tier (40 % at tier 0 to 80 % from tier 4), and the dice follow the lifted cap. The strict cap stays a rule: nothing passes the *lifted* ceiling, and the lift is bounded in data | §7 |
| 6 | Monster Protection | a rule of thumb by level and body (none, hide, bony, armoured: 0, (L−10)/8, L/6, L/4+1) applied as a data pass to `natural.protection`, with a list of exceptions to hand-tune; shown against the player's damage after §3 | §8 |
| 7 | Stat training | chances raised (a blow 5 → 15 %, hurt 10 → 20 %, ...) and six uses added (parry, block, critical, killed, drank, elixir); a **daily check** of every stat at the same hour (the `training` stream, six draws) beside the level-up's, with `need × daily_factor` (3) so a trained stat rises between levels about every two to three weeks of game time early and rarely past its potential; potentials stay the soft cap | §9 |
| 8 | Potions | a `[object.drink]` table (like `read`, `wand`, `rod`): heal, cure, boost, vision, haste, resistance, mana, and elixir kinds; **Drink** (`q`, `Action::Drink`, `drink:b`), one turn; unidentified by colour labels like scrolls', identified by drinking or the scroll of identify; every potion gives training marks by its nature; sold by alchemists and general stores, found as loot | §10 |
| 9 | Elixirs | rare potions: +1 to a stat for good and **+1d2 potential** (recommended over +1: see §10.5); no price in shops (quest rewards, altars, deep loot; a very rare alchemist's stock) | §10.5 |
| 10 | Talents | a new system `weapon_abilities` (four talents, one class ability), a new system `potions` (three talents), the `stat_training` channel promoted to a system (two talents: the daily check is now a rule a talent can change); spells and monster Protection extend systems already in the list | §11 |
| 11 | Slices | fourteen issues in five waves, never more than three at once that add an action or save state | §15 |

**Key choices, and why**

1. *Numbers first, abilities second.* The balance report's finding is arithmetic (a rank cap of +5 against monsters that reach +18); the per-rank tables fix the arithmetic in data, and the abilities give ranks 4 to 10 something to look forward to beyond "+1 more". Both are needed: abilities alone would not move the 2d12 curve, numbers alone would make rank 10 a bigger rank 5.
2. *Triggered abilities ride the free-blow and status machinery.* Riposte is already a free blow (`FreeBlows::riposte`, the Riposte talent); a style's rank switches the same bit on through the one reader, `World::free_blows_of`. Staggered is already a status. The new code is the registry, one trigger hook, and a handful of effect kinds.
3. *Cooldown statuses, not a new pool.* The martial class actives pay with cooldown statuses (Spent, Winded, Drained), casters with mana, and the five resource pools are class identities (a thief's Openings). A shared "stamina" would be a sixth bar on the status bar and a new system to cover with talents, for no rule the cooldowns do not already give: "time is the penalty" (`docs/dev/progression.md`, coverage policy item 4). §16 q2 keeps the pool as the alternative.
4. *Stats lift spell caps; they do not add damage past them.* The magic design's "strict cap" is kept as a rule, made a function of the caster's stats with a bounded lift per tier, authored per spell so a cantrip stays a cantrip.
5. *Protection is data.* The report shows monsters with Pv 0 to 5 against player damage that the new ranks double; the rule of thumb makes a pass over `assets/data/creatures/` reviewable at a glance, and the roster formulas' tests pin it.
6. *Growth between levels needs no new channel.* The marks exist; one more moment at which they are checked, at a fixed hour, is the smallest rule that gives stats a life between levels and keeps the owner's no-anti-farming rule (nothing is lost, nothing is capped, time is the only price).
7. *Potions are the scrolls' twin.* Labels, identification, the list screen, the loot and shop draws, and the "a read trains Intelligence" hook all exist for scrolls; potions take each with one new table on the object and one action.

## 2. What exists today, and what this builds on

**The balance findings** (the probe of 2026-10-27, a `#[ignore]` test reading every number through the game's own code, averaged over seeds; a player blow per 1000 energy, monsters' blows scaled by speed; `scratchpad/balance-report.md` holds the tables):

| Finding | Numbers |
|---|---|
| The player's unaided to-hit is Strength modifier + rank to-hit, capped at +5 at rank 10 | a plain warrior goes +4 → +8 from level 1 to 60; monsters go −1 → +17 (Strength up to 40, plus `to_hit = L/10` in the rosters) |
| Hit chances cross over | the plain warrior's hit chance falls 85 % → 46 % (monster Dv 12 → 21); the monster's rises 19 % → 75 % against plain armour |
| Magic armour decides whether the player is ever hit | a full magic set is Dv 25 to 39 from level 8; monsters need a natural 24 from level 5 to 40 (a fight of 500+ rounds for the monster) |
| Hit points stop at level 20 (`hp_level`), monster damage and hit points do not | monster damage per blow 14 → 32 and HP 71 → 221 from level 20 to 60; player damage per hit 10 → 15 |
| Protection is lopsided | monsters: Pv median 0 to 2, so the player's soak barely matters; a plain warrior kills a same-level monster in 16 rounds at level 30 and 36 at level 60 |
| Casters fall behind | wizard spells top out at 3d6 (about 10.5) at every level, against 56 HP at level 15 and 221 at 60; no spell above tier 3 exists yet |
| Not measured | stat training (the probe's stats stay at creation values), talent choice (`levelup:auto` takes near-useless picks), stances, styles' extra blows, resistances |

The report's tuning options A to F map onto this design as: **A** (a growth term for player to-hit) → §3's ranks and §9's trained Strength; **E** (monster Protection, a damage term) → §8 and §3's damage; **F** (spell dice past 3) → §7; **D** (narrow the magic-armour gap) and **C** (hit points past level 20) are *not* designed here (§14) but named in §16 for the owner.

**What this builds on** (each the one place; nothing here adds a second):

| Part | Today |
|---|---|
| Weapon skills and styles | `data/weapon_skills.toml`: `[Benefit; 11]` per skill (`to_hit`, `damage`, `threat`, `crit`, `speed`, `parry`, `reload`, `shield`; styles add `ease_main`, `ease_off`, `bash`), ranks 0 to `MAX_RANK` 10, marks on the `melee`/`missile` curves, defaults between skills. `Actor::attack_benefit(kind, with)` is the sum a profile reads; `Actor::defence` adds Parry and Shield; `World::attack_price` takes `speed` off. The file's own comment: "Rank 10 is where later active or triggered abilities will hang" |
| Free blows | `World::second_blow(striker, target, Cue)` is the one place every blow beyond an attack's first is decided (`Cue::First`, `Attacked { parried }`, `Cleaved`, `Leaves`, `Approaches`); `FreeBlows { cleave, great_cleave, riposte, opportunist, gatekeeper }` on `Actor::free_blows`, read only through `World::free_blows_of`; "a free blow brings none of its own"; the Riposte and Opportunist talents switch them on |
| Statuses | `data/status.toml`: `combat`, `cost`, `blocks`, `follows`, `penalty` (a raise too), `eases`; a cooldown is a status with `eases` and no tick (Spent, Winded, Drained); `World::afflict` the one way to give one; Staggered (`cost = 150`, one turn, `stacking = "refresh"`) exists |
| Abilities | `warlock_core::ability`: `Ability` variants, `Source` groups (Racial, Birth sign, Class, Spells, Talents, Skills, **Weapon skills** (Shield bash is there), Items, Here), `Aim::{Nothing, Direction, Cell}`, `Unavailable::{NoShield, NotYet, Strayed, While, Short, NoMana}`; `use:ID[:DIR|:X,Y]` is the action text of every class active and spell |
| Resources | `data/resources.toml`: Rage, Focus, Openings, Ki, Inspiration; `Build` events (`kill`, `hit`, `critical`, `struck`, `sneak_attack`, `sneak_kill`) raised at one place each in `fight.rs::resolve` and `dies` |
| Spells | `data/spells.toml` tiers (`comfortable`, `cost`, `ceiling`), `reach_step` (Willpower) and `presence_step` (Presence); a spell's `damage = { dice, per_levels, max_dice, bonus }` held to `ceiling × resolution.damage %`; attack spells aim with the Dexterity modifier; the load-time cap check |
| Monster Protection | `natural = { defence, protection, name }` per creature type; `Hide::protection` summed by `Actor::protection`; `combat::soak` draws ⌈Pv/2⌉..Pv once per physical hit; rosters' formulas pin Defence from level, nothing pins Protection |
| Stat training | `data/stats.toml` `[training]` (`need_base` 20, `need_per_point` 3, `over_potential_divisor` 4) and 25 `[[training.use]]` entries with chances; `Actor::stat_marks`; `train_stat(StatUse)` draws the `training` stream; `check_stats` at every level-up draws the `levels` stream six times, p = marks / (marks + need); a rise says "You feel stronger." |
| Potions | the `potions` item class (`!`); `red_potion` and `blue_potion`, the witch's kit, with no effect; `shops.toml` lists potions among the general store's and the alchemist's wares ("alchemists wait for potions that do something", spec §7); corpses' `on_eat stat` raises a score with "You feel stronger!" (`eat.rs::rise_news`); the angelica root `cures`; `Stats::train` lifts a potential with a score; the scroll labels (`scroll_label.rs`, `Labels::draw` from the `identification` stream, `WorldState::known_scrolls`, `Lore`) |

## 3. Weapon skills: the new per-rank tables

**The rule stays**: 11 numbers per benefit per skill, rank 0 nothing, never less at a higher rank, never shown. Ranks 1 to 5 keep today's values (the early game is right: a level-5 warrior hits 85 % and is hit 19 %), and the growth is moved into ranks 6 to 10, where today a rank is worth "+1 every other rank". Skills fall into families for the tables (a family is a pattern, not a data concept; each skill's arrays are still written out in the file):

| Family | Skills | Rank 10 today (to-hit / damage / threat / speed / parry) | Rank 10 proposed |
|---|---|---|---|
| **Light blades** | daggers, short blades | +5 / +3 / 2 / 20 / +2 (daggers); +5 / +3 / 1 / 10 / +3 (short) | +7 / +5 / 3 / 30 / +4; +7 / +5 / 3 / 25 / +5 |
| **Precise** | long blades, spears, unarmed | +5 / +4 / 1 / 0 / +4 (long); +5 / +4 / 0 / 0 / +2 (spears); +5 / +5 / 0 / 15 / 0 (unarmed) | +7 / +8 / 2 / 0 / +6; +7 / +8 / 2 / 0 / +4; +7 / +8 / 2 / 30 / 0 |
| **Heavy** | great blades, axes, polearms | +4 / +6 / 1 / 0 / +2 (great); +4 / +5 / 0 / 0 / 0 (axes, polearms) | +5 / +12 / 2 / 0 / +4; +5 / +12 / 2 / 0 / 0 |
| **Middling** | maces, staves, shield bashing | +4 / +5 / 0 / 0 / 0 (maces); +4 / +3 / 0 / 0 / +4 (staves); +5 / +4 (bash) | +6 / +8 / 1 / 0 / 0; +6 / +5 / 1 / 0 / +6; +6 / +5 / 1 |
| **Launchers and thrown** | bows, crossbows, slings, thrown | +5 / +3 / 1 / 15 (bows); +4 / +3 / 0 / reload 50 (crossbows); +5 / +4 / 0 / 10 (slings); +5 / +3 / 0 / 15 (thrown) | +7 / +5 / 2 / 25; +6 / +5 / 1 / reload 60; +7 / +5 / 1 / 25; +7 / +5 / 2 / 30 |
| **Shield blocking** | shield_block | shield +4 | shield +6 |

The full arrays for the four skills the balance report's players use, before → after (ranks 0 to 10):

| Skill | Benefit | Today | Proposed |
|---|---|---|---|
| long blades | to_hit | `[0,1,1,2,2,2,3,3,4,4,5]` | `[0,1,1,2,2,2,3,4,5,6,7]` |
| | damage | `[0,0,1,1,2,2,2,3,3,4,4]` | `[0,0,1,1,2,2,3,4,5,6,8]` |
| | threat | `[0,0,0,0,0,0,0,0,1,1,1]` | `[0,0,0,0,0,0,0,1,1,2,2]` |
| | parry | `[0,0,1,1,1,2,2,2,3,3,4]` | `[0,0,1,1,1,2,2,3,4,5,6]` |
| great blades | to_hit | `[0,1,1,1,2,2,2,3,3,3,4]` | `[0,1,1,1,2,2,2,3,4,4,5]` |
| | damage | `[0,1,1,2,2,3,3,4,4,5,6]` | `[0,1,1,2,2,3,4,5,7,9,12]` |
| | threat | `[0,0,0,0,0,0,0,0,0,1,1]` | `[0,0,0,0,0,0,0,0,1,1,2]` |
| | parry | `[0,0,0,1,1,1,1,2,2,2,2]` | `[0,0,0,1,1,1,2,2,3,3,4]` |
| daggers | to_hit | `[0,1,1,2,2,3,3,4,4,5,5]` | `[0,1,1,2,2,3,3,4,5,6,7]` |
| | damage | `[0,0,0,1,1,1,1,2,2,2,3]` | `[0,0,0,1,1,1,2,2,3,4,5]` |
| | threat | `[0,0,0,0,0,0,1,1,1,2,2]` | `[0,0,0,0,0,0,1,1,2,2,3]` |
| | speed | `[0,0,0,0,0,5,5,10,10,15,20]` | `[0,0,0,0,0,5,5,10,15,20,30]` |
| bows | to_hit | `[0,1,1,2,2,3,3,4,4,5,5]` | `[0,1,1,2,2,3,3,4,5,6,7]` |
| | damage | `[0,0,0,1,1,1,2,2,2,3,3]` | `[0,0,0,1,1,1,2,2,3,4,5]` |
| | threat | `[0,0,0,0,0,0,0,0,1,1,1]` | `[0,0,0,0,0,0,0,1,1,2,2]` |
| | speed | `[0,0,0,0,0,0,5,5,10,10,15]` | `[0,0,0,0,0,0,5,10,15,20,25]` |

The other skills follow their family's pattern (the implementer writes them out; a content test holds every array to its family's rank-10 value in this table and to "never less at a higher rank"). `MAX_THREAT_WIDENING` (10) and the data checks already allow these values; `MAX_SAVING` (50) holds the speeds.

**Why moderate to-hit.** With rank 10 at +7, a plain warrior at level 60 (Strength modifier +3 untrained) is at +10 against Dv 21: 69 %. With Strength trained to 26 (§9) they are at +15: 93 %. The report's lesson is that each point near the middle is worth 7.5 % and a side 6 points ahead saturates; +7 leaves the trained stats their room and keeps monster Defence meaningful. Damage can grow further because Protection (§8) and hit points answer it, where nothing answers to-hit but Defence.

**The effect on the report's matchups** (a plain-geared warrior with a long sword against the median monster of their level; "+training" is Strength trained by §9 to 16 / 18 / 20 / 22 / 24 / 26 at levels 10 to 60; "+Pv" is §8's Protection for a hide / bony / armoured monster of that level; rounds are expected rounds for the player to kill):

| L | Today: to-hit / dmg, hit %, dmg per round, rounds | New ranks only | New ranks + training | + new Pv: rounds hide / bony / armoured |
|---|---|---|---|---|
| 10 | +5 / 8.5, 90 %, 8.0, **5** | +5 / 9.5, 90 %, 8.9, **4** | +6 / 10.5, 93 %, 10.2, **4** | 4 / 4 / 5 |
| 20 | +6 / 9.5, 90 %, 9.2, **8** | +7 / 11.5, 93 %, 11.5, **6** | +9 / 13.5, 98 %, 14.2, **5** | 5 / 6 / 7 |
| 30 | +6 / 10.5, 69 %, 6.9, **16** | +8 / 12.5, 81 %, 10.2, **11** | +11 / 15.5, 93 %, 14.6, **8** | 8 / 9 / 11 |
| 40 | +8 / 11.5, 75 %, 8.3, **16** | +10 / 15.5, 85 %, 13.6, **10** | +13 / 18.5, 96 %, 18.2, **7** | 8 / 9 / 12 |
| 50 | +8 / 11.5, 69 %, 7.7, **25** | +10 / 15.5, 81 %, 12.9, **15** | +14 / 19.5, 96 %, 19.3, **10** | 11 / 13 / 17 |
| 60 | +8 / 11.5, 54 %, 6.2, **36** | +10 / 15.5, 69 %, 11.2, **20** | +15 / 20.5, 93 %, 19.8, **11** | 13 / 16 / 22 |

Read with the report's monster side (the plain warrior dies in 18 rounds at level 30 and 4 at 60; Parry +2 more lifts those to 27 and 5): the fight at level 60 goes from "36 rounds to kill, dead in 4" to "11 to 22 rounds to kill, dead in 5". Still lost without hit points past level 20 (§16 q9), which is the report's option C and outside this design. The thief with magic gear at level 60 goes from 20 rounds to 15 (short blades +7 / +5 and the wider threat). The wizard's melee is not meant to be fixed here (§7).

**What the player sees.** Nothing new: ranks and the marks bar on the weapon skills screen (`W`), as today; the ready attacks on the character screen show the new to-hit and damage as they show today's (`PerceptionView::own_attacks`). The wiki's `weapon-skill-benefits.md` tables regenerate from the data.

## 4. Rank abilities: triggered and active (the system)

### 4.1 The two kinds

| Kind | Fires | Cost | Where it lives |
|---|---|---|---|
| **Triggered** | on one combat event (§4.3) while the actor has the rank and, for a style's, fights in that configuration | none: a free thing, like a free blow (decided without a draw; its blow or status draws as that blow or status would) | nowhere the player chooses: it is listed in the abilities menu greyed with "fires by itself" (so the player can read what they have), never usable |
| **Active** | when the player picks it from the abilities menu's *Weapon skills* group (`Source::WeaponSkill`, where Shield bash is) and aims it (`Aim::Nothing`, `Direction`, or `Cell`, per ability) | its action's time (a melee blow's, a shot's) and a **cooldown status** (a status with `eases` and no tick, named per ability, `Unavailable::While` until it eases); never a key of its own (CLAUDE.md "Hard rules") | `Ability::Weapon(WeaponAbilityId)`; the action text is the existing `use:ID[:DIR|:X,Y]`, resolved after the class actives, the sign gifts, and the spells |

**Who has it**: `had_by` = the actor's rank in the ability's skill is at least `rank` (a `Weapon` talent's `ranks` bonus counts, as it does for benefits: Stylist and Fencing make an ability come sooner), and for a style, the actor is in its grip now (`Actor::style(kind, with)`), else `Unavailable::NotSoHeld` ("your hands are not so held"). Creatures have ranks too (`Actor::weapon_skills` from the type) and so **have the triggered abilities** by the same rule, read through the same hook, which is why every triggered effect is written as "the actor", never "the player"; actives are the player's (creatures use no menu; the AI may pick them up later).

### 4.2 The data: `data/weapon_abilities/<id>.toml`

One `[[weapon_ability]]` per file, read through `ContentFiles::table("weapon_abilities")` (campaigns may add their own), ids `WeaponAbilityId` indices in path order, in the content fingerprint (saves name the cooldown statuses, not the abilities; the index is for the menu and the tests). **Why a directory and not `weapon_skills.toml`**: the skills file is one hot file holding every skill's arrays, and the merging playbook keeps churning content tables as directories, one file per entity (talents, spells, mishaps, rod effects, uniques). An ability names its skill; the skill does not list its abilities. The loader checks: the skill exists, `rank` is 1 to 10, a style's ability names a style, a weapon skill's a weapon skill, and the effect's fields fit its kind (as `RodDoes` is checked).

```toml
# data/weapon_abilities/rend.toml: Twin Blades rank 4, triggered
[[weapon_ability]]
id = "rend"
name = "Rend"
description = "When both your blades bite in one attack, the wound they open between them bleeds a third blow's worth."
skills = ["twin_blades"]        # one or more skills or styles (a family shares one file)
rank = 4
kind = "triggered"
trigger = "both_hands_hit"       # §4.3
systems = ["weapon_abilities", "styles"]   # for the wiki's by-system table
[weapon_ability.effect]
kind = "extra_damage"            # §4.4
dice = "main_hand"               # the main hand's damage dice, rolled once more, no bonus
```

```toml
# data/weapon_abilities/lunge.toml: spears and staves rank 9, active
[[weapon_ability]]
id = "lunge"
name = "Lunge"
description = "A long thrust at a foe two paces off, over an empty pace between."
skills = ["spears", "staves"]
rank = 9
kind = "active"
aim = "direction"
cost = "melee"                   # the ActionKind priced (data/actions.toml); default melee
cooldown = "overextended"        # a status of status.toml with eases and no tick
systems = ["weapon_abilities", "weapon_skills"]
[weapon_ability.effect]
kind = "reach_blow"
reach = 2
```

Fields: `id`, `name`, `description`, `skills` (1 or more), `rank`, `kind` (`triggered` | `active`), `trigger` (triggered only), `aim` and `cost` and `cooldown` (active only; `cooldown` optional: an active without one is limited by its time alone, as a plain blow is), `systems`, `[weapon_ability.effect]` with only its kind's fields. An ability is listed in the abilities menu by its skill's name as a sub-heading ("Long blades: Deep cut (fires by itself)", "Spears: Lunge"), the group ordered by skill file order then rank.

### 4.3 Triggers: one hook, ten events

`World::weapon_trigger(actor, Trigger, Context)` is **the one place** triggered abilities fire: it reads `Content::weapon_abilities_for(skill, rank)` for the skill (and style) of the attack concerned, applies each effect in path order, and is called from the lines that already raise the matching event. No new place in combat learns about abilities; the `Build` events of the resources (`docs/dev/resources.md`) are the model, and three of the triggers are raised on the same lines:

| Trigger | Fires when | Where it is raised (an existing line) | Who fires |
|---|---|---|---|
| `hit` | a blow, shot, or throw of the actor's hits (soaked or resisted included: it landed) | `fight.rs::resolve`, where `gain_resource(Hit)` is | the attacker |
| `critical` | a confirmed critical of the actor's | the same line as `Build::Critical` | the attacker |
| `big_hit` | a blow of the actor's is a big hit (`combat::big_hit`) | `World::after_blow`, where wounds are decided | the attacker |
| `kill` | a blow of the actor's kills | `fight.rs::dies`, the `Build::Kill` line | the attacker |
| `struck` | the actor is hit | the `Build::Struck` line | the target |
| `parried` | an attack on the actor missed by its Parry | `second_blow(Cue::Attacked { parried: true })`, where `World::riposte` is decided | the target |
| `blocked` | an attack on the actor missed by its Shield (`Strike::Missed { by: Shield }`, `Defence::why_missed`) | `fight.rs::resolve`, the miss attribution, beside the shield-blocking marks | the target |
| `dodged` | an attack on the actor missed by its Dexterity or Dodge | the same attribution, beside `StatUse::Dodge` | the target |
| `both_hands_hit` | both blows of a two-weapon attack hit the same target, which lives | `World::after_attack`, after the second blow | the attacker |
| `approaches` | a hostile steps up beside the actor | `World::step_up`, the gatekeeper's cue | the one approached |

(`both_hands_hit` and `approaches` are specific cases the catalogue needs; the enum is closed, `Trigger::ALL` in wiki order.) The skill concerned is the attack's for the attacker's triggers (`AttackSkills` is already on the resolve path) and the actor's wielded weapon's and style's for the target's (what they hold decides what they can do with it: a riposte with a staff is the staff's). A trigger whose effect makes a blow goes through `World::second_blow` as a `Cue` or through `strike_free` as a `Free` kind, so the rule "a free blow brings none of its own" holds for everything here.

### 4.4 Effects: a closed list, each on one existing mechanism

| `kind` | Fields | What it does, and through what |
|---|---|---|
| `free_blow` | `trigger = "riposte" \| "cleave" \| "great_cleave" \| "opportunist" \| "gatekeeper"` | switches that `FreeBlows` bit on for the holder: folded into `World::free_blows_of` beside the talents' (the design's one reader). A triggered ability of this kind has no `trigger` of its own: the free blow's is its trigger |
| `status` | `status`, `severity` (default 1), `on = "target" \| "self"` | `World::afflict` on the target of the blow or on the actor; a hostile with the status's `immune` is refused as any status |
| `extra_damage` | `dice = "main_hand" \| "off_hand" \| "1d6"`, optional `kind` | one more roll of those dice (no bonus, no riders) hurt as a rider of the blow's kind would be, after the blow's own damage: `hurt` with `Harm::Actor`, never soaked twice (the blow's soak was drawn once) |
| `reach_blow` | `reach` 2 | a melee blow at the first hostile along the direction within `reach`, the cells between empty and passable: an attack `with` the wielded weapon through `attack_with` |
| `sure_critical` | — | the next blow is a `Profile::sure_critical` (Master thief's, `combat::strike_with`: every hit a critical, no confirmation draw) |
| `ignore_defence` | `categories = ["parry", "shield"]` | the blow is rolled against the target's Defence without those categories (`Defence` without them: a `Target` built by `World::target_of` minus the categories) |
| `ignore_protection` | — | the profile's `penetration` is set to the target's full Pv |
| `sweep` | `to_hit = -2` | one blow at each hostile beside the actor, clockwise from the direction aimed (no draw to decide; each blow draws as a blow), each at the penalty |
| `pierce_shot` | — | a shot that, on a hit, flies on to the next actor on its line as a missed shot does (`World::fly` already flies on past a miss) |
| `knockback` | `cells` 1 | the target is moved one cell away along the blow's direction if that cell is free and walkable (`World::teleport`'s cell check, no draw), then `status` Staggered |
| `buff` | `status`, `severity` | `World::afflict` on the actor (an active's self-buff: Shield wall) |

Every kind is a variant of `WeaponDoes` with its one read; a kind with no catalogue entry is not written (no speculative effects).

### 4.5 The cost model, and why cooldowns

An active ability costs **its action's time** (priced by its `cost` kind: a melee blow's `attack_price`, a shot's `shot_price`, so the weapon's speed and the stance still count) **and a cooldown status**, afflicted at one degree as the ability is used (`afflict(player, cooldown, 1, player)`, after its own refusals, so a refused use costs nothing). While the status lasts the menu greys the row with `Unavailable::While(cooldown)`: "you are overextended". Cooldowns are 1 to 5 minutes of game time (ten to fifty turns), in data per status (`eases`), first guesses in §6.

Alternatives weighed:

| Model | For | Against |
|---|---|---|
| **Cooldown statuses** (chosen) | the martial class actives' model (Smite's Spent, Second wind's Winded); no new bar; "time is the penalty" (the coverage policy, item 4: a cast that costs a pool has no cooldown besides, and a timer is what a martial active pays); creatures could use them later with no pool to grant | several actives mean several statuses in `status.toml` (one group, by topic) and a status bar that may show two or three cooldowns at once (they give way last, as cooldowns do today) |
| A shared stamina pool (`data/resources.toml`) | one bar, costs tunable per ability, builds on hits | a sixth pool beside five that are class identities; a new system to cover with talents; a pool that regenerates is spent on the strongest active in a loop, so each would still need a cooldown or a very high cost; the owner's resource pools were designed as class features (progression design §4.18) |
| Mana | exists for everyone | martial characters have 1 to 60 mana; the design keeps mana magic's |

§16 q2 leaves the pool as the owner's alternative.

### 4.6 What the player sees

- The abilities menu's *Weapon skills* group lists Shield bash and, under a sub-heading per skill, the abilities the player's ranks give: an active as a usable row ("Lunge — a long thrust at a foe two paces off"), greyed with its reason while its cooldown lasts or the hands are not so held; a triggered one greyed with "fires by itself". A weapon skill's rank that unlocks one says, with the rank's own line, "You feel more skilled with spears. You have learned the lunge." (the ability's `name` in lower case).
- Messages: a triggered blow is said as a free blow is ("You parry the kobold's spear. You strike back at the kobold."); Rend: "Your blades rend the orc."; Staggering block: "The orc staggers from your shield."; an active: its blow's own message. Cooldowns begin and end with their status words ("You are overextended." / "You recover your footing.").
- The status bar shows a cooldown as it shows Winded. Nothing shows a rank threshold or a chance.
- The character screen's ready attacks are unchanged (situational blows are not part of a profile, `docs/dev/combat.md`).

## 5. Fighting styles: tables and abilities

The styles' benefits grow at ranks 6 to 10 as the weapon skills' do, within what each grip may use (`content/weapon_skill.rs` checks it):

| Style | Benefit | Today (ranks 0 to 10) | Proposed |
|---|---|---|---|
| Duelist | to_hit | `[0,1,1,1,2,2,2,3,3,3,4]` | `[0,1,1,1,2,2,2,3,3,4,5]` |
| | parry | `[0,1,1,2,2,3,3,3,4,4,5]` | `[0,1,1,2,2,3,3,4,5,6,7]` |
| Twin Blades | ease_main | `[0,1,1,2,2,2,3,3,3,4,4]` | `[0,1,1,2,2,2,3,3,4,4,4]` |
| | ease_off | `[0,1,2,2,3,4,4,5,6,7,8]` | `[0,1,2,2,3,4,5,6,7,8,8]` (penalty-free at 9) |
| | to_hit | none | `[0,0,0,0,0,0,0,1,1,2,2]` |
| Great Weapon | damage | `[0,1,1,2,2,3,3,4,4,5,6]` | `[0,1,1,2,2,3,4,5,6,8,10]` |
| | crit | `[0,0,0,0,0,0,0,0,1,1,1]` | `[0,0,0,0,0,0,0,0,1,1,2]` (×4 at 10; `CRITS` allows up to ×5) |
| | speed | `[0,0,0,0,5,5,5,10,10,10,15]` | `[0,0,0,0,5,5,5,10,15,15,20]` |
| Sword and Board | shield | `[0,1,1,1,2,2,2,3,3,3,4]` | `[0,1,1,1,2,2,2,3,4,5,6]` |
| | bash | `[0,0,0,5,5,10,10,15,15,20,25]` | `[0,0,0,5,5,10,10,15,20,25,35]` |
| Marksman | to_hit | `[0,1,1,2,2,2,3,3,3,4,4]` | `[0,1,1,2,2,2,3,3,4,5,6]` |
| | speed | `[0,0,0,5,5,5,10,10,10,15,15]` | `[0,0,0,5,5,5,10,10,15,20,25]` |
| | reload | `[0,0,10,10,10,20,20,20,30,30,30]` | `[0,0,10,10,10,20,20,30,30,40,40]` |
| Skirmisher | to_hit / damage / speed | `[..,3,3]` / `[..,3,3]` / `[..,15,20]` | to_hit `[0,1,1,1,1,2,2,2,3,3,4]`, damage `[0,0,1,1,1,2,2,3,3,4,5]`, speed `[0,0,0,5,5,10,10,15,15,20,30]` |
| Brawler | to_hit / damage | `[..,3,4]` / `[..,5,6]` | to_hit `[0,1,1,1,2,2,2,3,3,4,5]`, damage `[0,1,1,2,2,3,4,5,6,8,10]` |
| Staff Fighter | to_hit / parry | `[..,3,4]` / `[..,4,4]` | to_hit `[0,1,1,1,2,2,2,3,3,4,5]`, parry `[0,1,1,2,2,2,3,4,5,6,7]` |

A style's to-hit at +5 with the weapon's +7 and trained Strength lands a rank-10 duelist at about +20 at level 60 (98 % against Dv 21): the price of a rank-10 style, which took 15 345 marks on top of the weapon's, and the reason the weapon tables stay moderate. Styles' abilities come at **rank 4 and rank 7** (the owner's "4/7/10"; rank 10 is the numbers' capstone, and §16 q3 asks whether it should carry a third ability).

## 6. The catalogue, interactions, and abuse cases

Eighteen abilities, each on §4.4's kinds. Cooldown statuses are new entries in `data/status.toml` in a "Weapon abilities' cooldowns" group (`eases` in seconds; `stacking = "refresh"`; no tick). Names in the game's register; descriptions are the menu's line. **All first guesses.**

| # | Ability | Skills (rank) | Kind: trigger or aim | Effect | Cost / cooldown |
|---|---|---|---|---|---|
| 1 | **Deep cut** | daggers, short blades, long blades, great blades (5) | triggered: `big_hit` | `status` bleeding +1 degree on the target (a big hit with a blade wounds as a critical does) | — |
| 2 | **Vital strike** | daggers, short blades, long blades, great blades (9) | active: direction | `sure_critical`: the next blow at that foe is a critical if it hits | a blow's time; *Off balance*, 3 min |
| 3 | **Crushing blow** | axes, maces, polearms (5) | triggered: `big_hit` | `status` staggered on the target (its next action costs half again) | — |
| 4 | **Sunder** | axes, maces, polearms (9) | active: direction | `ignore_protection`: one blow that armour does not soak | a blow's time; *Arm-weary*, 3 min |
| 5 | **Brace** | spears, staves (5) | triggered (`free_blow`: gatekeeper) | a hostile that steps up beside the actor takes a blow as it comes (the existing gatekeeper) | — |
| 6 | **Lunge** | spears, staves (9) | active: direction | `reach_blow` 2: a blow at a foe two cells off over an empty cell | a blow's time; *Overextended*, 2 min |
| 7 | **Point-blank** | bows, crossbows, slings, thrown (5) | triggered: `approaches` | a shot or throw at a hostile that steps up beside the actor (the gatekeeper's geometry, a shot instead of a blow; needs ammunition ready) | — |
| 8 | **Piercing shot** | bows, crossbows, slings (9) | active: cell | `pierce_shot`: a shot that flies on through the first creature it hits to the next on its line | a shot's time; *Strained bow*, 2 min |
| 9 | **Stunning bash** | shield_bash (5) | triggered: `hit` (a bash that hits) | `status` staggered on the target | — |
| 10 | **Shield wall** | shield_block (9) | active: nothing | `buff` *Walled* 5 degrees: Shield +3 to Defence, attacks cost 125 %, a degree a turn | an action's time (`wait`); *Shield-worn*, 5 min |
| 11 | **Riposte** | duelist (4) | triggered (`free_blow`: riposte) | the existing riposte: a parried attack on the actor earns a blow back once it is over | — |
| 12 | **Feint** | duelist (7) | active: direction | `ignore_defence` parry and shield: the blow is rolled as if the foe had neither | a blow's time; *Shown your hand*, 1 min |
| 13 | **Rend** | twin_blades (4) | triggered: `both_hands_hit` | `extra_damage` main_hand: the main hand's dice once more on the target | — |
| 14 | **Cleaving sweep** | great_weapon (4; `great_cleave` at 10) | triggered (`free_blow`: cleave) | the existing cleave: a kill carries the blow on to the next foe beside; at rank 10 it goes on while the blows kill | — |
| 15 | **Mighty swing** | great_weapon (7) | active: direction | the blow's own damage dice rolled twice (not a critical: bonuses and riders once), at −2 to hit | 150 % of a blow's time; *Winded arm*, 5 min |
| 16 | **Staggering block** | sword_and_board (4) | triggered: `blocked` or `parried` | `status` staggered on the attacker: its next action costs half again (the owner's "penalty to its next attack's speed") | — |
| 17 | **Pinning shot** | marksman (4) | triggered: `critical` (a critical shot) | `status` *Pinned* 1d3 turns on the target (a hold-like status: blocks `step`, eases a degree a turn, `stacking = "highest"`; no struggle check: it is a missile through the foot, not a grip) | — |
| 18 | **Stunning fist** | brawler (4) | triggered: `critical` | `status` staggered on the target | — |

Not every style gets two: Twin Blades, Sword and Board, Marksman, Brawler, Skirmisher, and Staff Fighter have one each at 4 (Skirmisher's and Staff Fighter's are their weapon families' Point-blank and Brace, so their style rows are their numbers; §16 q4 asks for a second). Shield bash, the one weapon-skill ability that exists, stays as it is.

**Interactions and abuse cases, decided here**

| Case | Rule |
|---|---|
| Riposte on a riposte, cleave off a riposte, a gatekeeper's blow that kills cleaving | the free-blow rule stands: **a free blow brings none of its own** but a cleave after a cleave. Nothing in this catalogue adds a cue; Brace, Point-blank, Riposte, Cleaving sweep are the existing switches |
| Rend twice in one attack | once per attack: `both_hands_hit` is raised once, in `after_attack`, and only when both blows were `Strike::Hit` on the same living target; a Sword and Board bash is not a second hand (the trigger is Twin Blades' only) |
| Staggered stacking (Crushing blow, Stunning fist, Stunning bash, Staggering block on one foe) | `staggered` is `max = 1`, `stacking = "refresh"`: a staggered foe cannot be staggered further, only kept so while blows land, which costs the blows |
| Staggering block against a foe that attacks twice a turn | each blocked blow refreshes the status once; the foe's *next* action is dearer, so two blocked blows in a turn cost it one slowed action, not two |
| Pinned forever by repeated criticals | `stacking = "highest"`: a new pin keeps the longer of the two, never adds; at most 3 turns from any one shot |
| Deep cut on a bleeding foe | bleeding `stacking = "add"` to `max = 5`, as any wound: a bleeding foe can be bled more, which is what blades do |
| Vital strike then Mighty swing then Feint in three turns | each is its own action with its own cooldown; nothing stops a sequence, which costs three turns of cooldowns and is the point of a rank-9 fighter |
| Feint plus Vital strike in one blow | Feint is a blow of its own, not a status: no stacking of actives on one blow. (A later "the next blow" status form is possible; not written) |
| Sunder on a monster with Pv 0 | allowed, pointless, costs the cooldown: the menu does not know the target's Protection (it must not: hidden information) |
| Lunge over a trap, a door, water | the cell between must be empty of actors and passable for the actor; a lunge is not a step (nothing is sprung); a closed door blocks |
| Point-blank with an unloaded crossbow | needs ammunition ready; a reload is not free: the trigger fires only when a shot could be loosed now, else nothing (no draw) |
| Piercing shot through an ally or a shopkeeper | it hits whoever is next, as a missed shot that flies on does; provoking as any shot |
| Shield wall while Raging, Hasted | statuses add: Walled's cost 125 and Hasted's 75 multiply as the data says they do |
| Creatures with triggered abilities | by the same rule: a rank-5 orc spearman (creature types have ranks) braces; a rank-4 duelist bandit ripostes. The rosters set creature ranks, so this is a content review item (§15 slice W2 lists which shipped types cross a threshold); `Trigger` for a creature draws exactly as the blow it makes, so fights without a qualifying creature keep their draws |
| Determinism | deciding never draws: `weapon_abilities_for` is a lookup, the free-blow switches are bits, Staggered and Pinned are afflicted with their severity (Pinned's 1d3 is one draw, as a hold's `turns` is), Rend's extra dice are rolled in the blow's draw order after the riders |

## 7. Spell scaling from the mental stats

### 7.1 Today

A spell's dice grow with caster level (`per_levels`, `max_dice`) under `ceiling[tier] × resolution.damage %`; Willpower widens reach (`reach_step` 2), Presence lengthens buffs and hexes (`presence_step` 2), Intelligence eases study; attack spells aim with the Dexterity modifier. The magic design's rule: "stats never lift damage over its cap". A level-60 wizard with tier-3 spells does 3d6 (10.5) to monsters of 221 HP; the first list stops at tier 3.

### 7.2 The proposal: an authored `[spell.scales]` table, and a lifted cap

```toml
[spell.scales]                          # all optional; absent dimensions keep today's rule
damage   = { intelligence = 10, willpower = 5 }   # percent of the cap per modifier point
to_hit   = { intelligence = 100 }                  # percent of the modifier added to the attack roll
reach    = { willpower = 50 }                      # a cell per 2 points (today's reach_step)
duration = { presence = 50, willpower = 25 }       # a degree per 2 points, plus one per 4 of Willpower
```

Weights are **percent of the stat's modifier** (`Stats::modifier`, the D&D floor((score − 10)/2)), summed over the stats named, rounded to the nearest, never below 0. The defaults when a table or a dimension is absent reproduce today exactly: `reach = { willpower = 50 }`, `duration = { presence = 50 }`, `to_hit = { dexterity = 100 }`, `damage = {}` (no lift). `reach_step` and `presence_step` in `spells.toml` are retired in favour of these defaults (a data change, one constant each).

**Damage: the cap is lifted, never passed.** `lift = Σ weight × modifier` percent, at most the tier's `max_lift` (a new `[tiers]` array, tier 0 first, first guess `[40, 50, 60, 70, 80, 80, 80, 80, 80, 80]`). The spell's cap for this caster is `cap_of(spell) × (100 + lift) / 100`, rounded down; the dice at a caster level are `SpellDice::at` as today, and then **as many more dice as the lifted cap holds** (`max_dice` becomes `floor(lifted cap / sides)` when that is larger; the file's `max_dice` is the unlifted ceiling's). The load-time check keeps refusing a spell written over its tier's unlifted cap, so the data cannot write a 10d6 fireball; the lift is the caster's, bounded by `max_lift`. Criticals, Empower, and Archmage are held to the lifted cap as they are held to the cap today (`spell::capped`).

Why a lift rather than "stats add dice": the strict cap is the magic design's load-bearing decision (§5 of it, "caps, not curves"); making the cap a bounded function of the caster keeps the ceiling table the one authority and the per-spell authoring small (a cantrip with `damage = {}` stays a cantrip; a bolt that "scales with wit" names Intelligence).

**To-hit**: an `attack` or `touch` spell with `to_hit` scales aims with the named stats instead of Dexterity (the one place: `Profile::bare(Attack { to_hit })` in `cast.rs`). Wands keep Dexterity (the wand's level, the user's hand).

### 7.3 Numbers: what a level 10 / 30 / 60 wizard deals, against the report's monsters

Wizard caster level 12 / 36 / 72 (casting 100, Arcane training +1 per 5); Intelligence trained (§9) to 18 / 22 / 26 (modifiers +4 / +6 / +8), Willpower 14 / 16 / 18 (+2 / +3 / +4); evocation damage spells authored `damage = { intelligence = 10, willpower = 5 }`; the median monster's HP from the report (38 / 112 / 221). "Casts" are expected casts to kill at an average roll (a burst hits several; this is per target).

| L | Spell (tier, resolution) | Lift | Cap today → lifted | Dice, average | Casts to kill (today) |
|---|---|---|---|---|---|
| 10 | Scorching ray (2, attack 100 %) | 50 % (max 60) | 18 → 27 | 4d6, 14.0 (3d6, 10.5) | 2.7 (3.6) |
| 10 | Fireball (3, area 80 %) | 50 % | 19 → 28 | 4d6, 14.0 (3d6, 10.5) | 2.7 (3.6) |
| 30 | Fireball (3, area) | 75 % (max 70) → 70 | 19 → 32 | 5d6, 17.5 (10.5) | 6.4 (10.7) |
| 30 | a tier-5 burst (5, area), not yet written | 70 % | 32 → 54 | 9d6, 31.5 | 3.6 |
| 60 | Fireball (3, area) | 110 % → 70 | 19 → 32 | 5d6, 17.5 (10.5) | 12.6 (21.0) |
| 60 | a tier-7 burst (7, area), not yet written | 110 % → 80 | 49 → 88 | 14d6, 49 | 4.5 |
| 60 | a tier-9 burst (9, area), not yet written | 80 % | 72 → 129 | 21d6, 73.5 | 3.0 |

The table says two things: the lift keeps a known spell useful two tiers longer (fireball at level 30 kills in 6 casts rather than 11), and **a level-60 caster needs the tier 4 to 9 spells to exist**, which is content the magic design planned and no scaling replaces (§14; §16 q6). Mana: a tier-7 burst of radius 2 costs 37 × 2.4 = 89 of a 502 pool, five casts; the tier-9 one 134, three; "area is expensive" holds.

### 7.4 What the player sees

The abilities menu's Spells group and the aiming mode show cost and reach as today (`own_spell_cost`, `own_spell_reach`); nothing shows a lift or a cap. The spell's description may say, in words, what it grows with ("It burns the hotter for a keen mind." is the author's line, not generated). The wiki's `spell-list.md` gains a "Grows with" column from the `scales` table, and the Magic page states the lift rule and the `max_lift` table.

## 8. Monster Protection by level and body

### 8.1 Today

Of 400 types, 7 to 16 per level band have any Protection; the median is 0 to level 27 and 2 from level 28; the mean is 0.3 at level 1 to 3.3 at level 60. The rosters' formulas pin Defence from level (`level ÷ 10 + 2` for spirits) and say nothing of Protection; the troll roster's "a hide of Protection 2 or more" is the only rule. Against §3's damage (15 to 20 a hit at level 40 to 60) a soak of 0 to 2 is nothing.

### 8.2 The rule of thumb

`natural.protection` is set per type by **body** and **level**, a data pass over `assets/data/creatures/` and the campaigns' (a creature file is one entity: no hot file), with the hide's `name` deciding the body (the 114 types with no `natural` get one where the body is not *none*):

| Body | Hides (by `natural.name` today) | Pv at level L (rounded down, at least 0) | L 10 / 20 / 30 / 40 / 50 / 60 / 70 |
|---|---|---|---|
| **none** | insubstantial form, rushing wind, crackling form, flickering flame, flowing body, fey grace, mud, and no body (oozes, swarms) | 0 | 0 |
| **hide** | hide (and its adjectives), skin, fur, feathers, dead flesh, wrappings, rotting scales | (L − 10) ÷ 8 | 0 / 1 / 2 / 3 / 5 / 6 / 7 |
| **bony** | bones, dry bones, bone, skull, scales, chitin, shell, carapace, plates | L ÷ 6 | 1 / 3 / 5 / 6 / 8 / 10 / 11 |
| **armoured** | stone body, stony hide, iron-hard skin, frost-hard skin, ice, slag hide, cinder hide; the animated objects' metals (`docs/design/animated-objects.md` §6.5 keeps its own table) | L ÷ 4 + 1 | 3 / 6 / 8 / 11 / 13 / 16 / 18 |

**Armed types**: a humanoid that rolls gear (`gear::roll`) keeps a hide of 0 or its race's (the player's races' `natural`: lizardfolk 1) and lets the armour supply the rest (gear adds up to +8 Pv today); the pass does not give an armed type both. **Penetration** stays the player's answer (the war hammer, the stiletto, crossbow bolts, Sunder at rank 9), and bludgeoning against the bony body should be revisited when per-kind `protects` on creatures exist (today only worn pieces have `protects`; the stone troll's "cracks under blunt blows" is written as a resistance).

### 8.3 Exceptions to hand-tune (the implementer's list, reviewed in the PR)

| Type (level) | Why | Suggested |
|---|---|---|
| giant rat, bat, deer, the level-1 vermin | should stay harmless to armour and soft to anything | 0 |
| cave beetle (3), giant beetle | the one early "shell" the spec cites (Dv +1 / Pv 3): keep | 3 |
| stone troll (27) | the roster gives 5 and a resistance; keep its own | 5 |
| mummy lord (36) | wrappings 3 by the roster | 3 (hide 3 is the formula's anyway) |
| elder troll (42) | "ancient hide" 4 today; hide gives 4 | 4 |
| the balor (62), the pit fiend, the dragons when they come | bosses: the armoured line (16) plus their resistances would make them walls to a plain fighter (22 rounds in §3's table); hold them to **bony** (10) and let hit points carry them | 10 |
| iron golem, the animated panoply | the animated design's table rules | as there |
| the hags (hide, casters) | casters should not also be hard to cut | hide ÷ 2 |
| spirits with `wall_defence` | none, as the body says; their Defence inside stone is their armour | 0 |

### 8.4 The effect on the player's damage per round (from §3's table)

At level 30, the plain warrior with trained Strength kills a hide monster in 8 rounds, a bony one in 9, an armoured one in 11 (16 today, with Pv 2); at level 60, 13 / 16 / 22 (36 today). The soak takes 20 to 45 % off a rank-10 fighter's blow against the armoured body, which is what Sunder, penetration, and criticals (soaked once after the multiplier) are for. Against today's ranks and untrained stats the armoured body at level 60 would take 50+ rounds: **§8 is not shipped before §3 and §9** (the plan, §15, orders them so).

### 8.5 What the player sees

Nothing new: a soaked hit "glances off the stone troll's stony hide" as today (`fight::Guard`), and the message is the hint. The wiki's `creature-numbers.md` already carries Pv; the Creatures page gains a paragraph on bodies in words ("bony things turn blades; the hard-skinned turn most blows") and no formula, as the game keeps it vague and the wiki states the table.

## 9. Stat training: plentiful marks, growth between levels

### 9.1 More uses, higher chances

`data/stats.toml`'s `[[training.use]]` chances raised, and six uses added at hooks that exist (one line each; `StatUse` gains six variants, `describe()` their words for the wiki):

| Stat | Use | Today | Proposed | Hook |
|---|---|---|---|---|
| Strength | `blow` | 5 | **15** | `fight.rs::resolve` |
| | `burdened` | 10 | 15 | `World::apply` |
| | `critical` (new): a confirmed critical of yours in melee | — | **40** | the `Build::Critical` line |
| | `block` (new): a blow your shield blocked | — | **20** | the shield-block marks' line |
| Dexterity | `dodge` | 20 | **30** | the miss attribution |
| | `shot` | 10 | **20** | `missile_hits` |
| | `parry` (new): a blow your Parry turned | — | **25** | the same attribution, the `parried` cue |
| | `sneak` | 5 | 10 | `make_noise` |
| Toughness | `hurt` | 10 | **20** | `hurt` |
| | `big_hit` | 50 | 60 | `after_blow` |
| | `killed` (new): a kill of yours in melee against a foe of your level or above | — | **30** | `earn_kill` (beside `discovered`) |
| Willpower | `hungry_fight` | 10 | 20 | `resolve` |
| | `cast` | 20 | **30** | `spend_mana` |
| Intelligence | `read` | 30 | 40 | `read.rs` |
| Presence | `chat` | 20 | 30 | `talk.rs` |
| any | `drank` (new): a potion drunk, the stat its nature names | — | **100** × its `marks` (§10.3) | `World::drink` |
| any | `elixir` (new): an elixir drunk | — | 100 × its `marks` | the same |

Unchanged uses keep their chances. The arithmetic: a fighting character lands about 30 blows a game day in play (the curve's 150 to 300 blows a level, a level in two to five days early); at 15 % that is 4 to 5 Strength marks a day from blows alone, 2 to 3 more from criticals and burden, 3 to 4 Toughness from being hit, 2 to 3 Dexterity from dodges and parries; a caster 3 to 6 Willpower a day from casting. Call it **8 to 15 marks a day in a trained stat**, 0 to 2 in an untrained one.

### 9.2 Growth between levels: the daily check

Today the marks are checked only at a level-up (`check_stats`, six `levels`-stream draws). **Proposed**: once a game day, at a fixed hour (06:00, the hour the sky turns and rests to dawn end), every stat gets the same check with a stiffer need, drawn from the **`training`** stream (the stream of marks, so a fight's rolls still never move, and the `levels` stream stays the level-up's):

`p = marks / (marks + need × daily_factor)`, `need = need_base + need_per_point × score` as today, `daily_factor` 3 (a new `[training]` key), the `over_potential_divisor` (4) on top at or above the potential.

A rise consumes the marks and says what it says today ("You feel stronger."); a failed check keeps them. The moment is a `World::live` moment like a disease's turn or a hunger change (`until_stat_check` ends a step at it, so a rest of a week makes seven checks at their exact hours, in order, whatever the batching), and `WorldState::next_stat_check: GameTime` is saved (left out of the JSON while the training rules are absent, so test content without `stats.toml` hashes as before). The level-up check stands beside it, unchanged.

**Expected pace** (a simulation of the rule with 8 to 15 marks a day, Strength 14, potential 17, over 120 game days with a level every two days): the daily check adds **4 to 8 rises** to the level-ups' over such a career, most of them before the potential; past it a stat rises between levels about once in 25 days. `daily_factor` is the knob: 2 doubles the between-level rises, 5 halves them. The owner's "plentiful" and "grow even between level-ups" are met without a new channel; the no-anti-farming rule holds (nothing decays, nothing is capped: a hundred kicks are a hundred chances, and a hundred bangs).

Why a fixed daily hour rather than "checked when the marks reach a threshold": a threshold check fires in the middle of a fight (a rise mid-fight is a message in the wrong place and a hit-point change the status loop must then handle), and it makes a rise a function of play volume alone, so a day of grinding rats is a day of rises; the daily hour bounds the rate in game time, which is the game's one currency. §16 q7 keeps the threshold variant.

### 9.3 What the player sees

As today: "You feel stronger." (quicker, tougher, sharper, more resolute, more confident) at dawn or at a level, the character screen's new score, no marks, no chance, no potential. A rise at the daily hour interrupts a rest as a message does (so the player notices; a rest to dawn ends then anyway). The wiki's Stats page states the daily rule and its arithmetic, and `stat-training.md` regenerates with the new uses and chances.

## 10. Potions and elixirs

### 10.1 Objects

A potion is an object of class `potions` with a `[object.drink]` table (`DrinkSpec` → `Drink` on `ObjectDef::drink`, parsed as `read`, `wand`, and `rod` are; a potion has no `food`, no `read`, no attack; `stackable`; `materials = ["glass"]`; weight 250; `level` and `frequency` make it loot, never magic (as scrolls: `loot::roll` draws among objects with a `frequency` and skips those without a `kind` for magic). Its kind and only its kind's fields:

| `kind` | Fields | What drinking does (through the existing one place) | Marks (`marks`, §10.3) |
|---|---|---|---|
| `heal` | `dice` ("2d8", "4d8", "8d8"), optional `cures` | hit points back at once, to the maximum (`Actor::hp` up; the one `heal` path); `cures` as a cure's | toughness |
| `cure` | `cures = [...]` (status and disease cure ids: `bandage`, `remove_disease`, a poison's), `diseases = true` | `World::cure(actor, means)` for each, as the angelica root's and the remedy's | toughness |
| `boost` | `status` (one of six boost statuses), `severity` 1 to 2 | `World::afflict` on the drinker: Fortified (St), Nimble (Dx), Hale (To), Keen (In), Steadfast (Wi), Imposing (Pr): `penalty = { from = 1, <stat> = 2 }` per degree, `eases = 600`, `max = 2`, `stacking = "highest"` | the stat |
| `vision` | `status` | a status that carries a **vision profile** (a new optional `vision` field on `StatusDef`, an id of `vision.toml`; read in `Actor::eyes` beside `worn_vision`, bettering part by part as a unique item's Vision power does): Cat-eyed (`low_light`), Owl-eyed (`darkvision`), Moth-eyed (`uv_sight`: the owner's ultravision); `eases = 1800` | willpower |
| `haste` | `severity` | `World::afflict` Hasted (the spell's status, cost 75) at the severity | dexterity |
| `resistance` | `status` | a status with a new optional `resist` map (`{ fire = 50 }`, read in `Actor::resistances` with what is worn: gear-like, never immune): Warmed (fire), Chilled-proof (cold), Earthed (electricity), Proof against poison; `eases = 1800` | toughness |
| `mana` | `dice` | mana back at once, to the maximum (`mana::regain`'s add; never a death) | willpower |
| `elixir` | `stat`, `points` 1, `potential` dice "1d2" | §10.5 | the stat, 20 marks |
| `harm` | `dice`, optional `status` | a bad potion (sickness: poisoned degrees, as rotten food), so unknown potions are a gamble as unknown scrolls are | — |

The first list (names in the game's register; prices by the scrolls' tiers; **first guesses**):

| Potion | Kind | Effect | Level / frequency | Value |
|---|---|---|---|---|
| potion of healing | heal | 2d8 | 1 / 60 | 40 |
| potion of greater healing | heal | 4d8, cures bleeding | 4 / 30 | 120 |
| potion of full healing | heal | 8d8, cures bleeding and poison | 10 / 10 | 300 |
| potion of antidote | cure | poisoned | 2 / 40 | 60 |
| potion of cure disease | cure | every disease (`diseases = true`) | 4 / 25 | 150 |
| potion of mana | mana | 3d8 | 3 / 30 | 100 |
| potion of might / of grace / of vigour / of wit / of will / of presence | boost | +2 to the stat for 10 minutes (severity 1; a greater one at severity 2, +4, level 8) | 3 / 20 each | 150 |
| potion of cat's eyes / of owl's eyes / of moth's eyes | vision | low-light / darkvision / UV sight for 30 minutes | 3 / 15 each | 100 |
| potion of quickness | haste | Hasted 1 degree (a minute, everything a quarter quicker) | 6 / 10 | 250 |
| potion of warmth / of frost / of earthing / of proofing | resistance | 50 % against fire / cold / electricity / poison for 30 minutes | 4 / 15 each | 120 |
| murky draught | harm | poisoned 2 degrees | 1 / 20 | 10 |
| elixir of might, grace, vigour, wit, will, presence | elixir | +1 to the stat for good, potential +1d2 | 8 / 2 each | 2000 |

The witch's `red_potion` and `blue_potion` become the potion of healing and the potion of mana (their files renamed; the kit line keeps "one of the two"); `scroll_of_light`-like placeholders are not kept.

### 10.2 Drinking

`Action::Drink { item }` (`drink:b`), `ActionKind::Drink` (`drink` in `data/actions.toml`, a turn), `KeyCommand::Drink` default **`q`** (quaff; unbound today, next to Eat and Read in `settings/keys.rs`, rebindable), opening `ScreenName::Drink` (`ObjectList::Drink`: the potions carried, the paged list as Eat's). `World::drink` (`world/drink.rs`): refused with no time when there is no such stack or it has no `drink` ("You cannot drink a long sword."); otherwise priced, one taken from the stack (positions shift as `use_up` handles), the type learned (`World::learn_scroll(object)`: "You drink the murky green potion. It is a potion of healing."), the effect applied through the table above, the marks given (`train_stat(StatUse::Drank(stat))` per mark), then the kind's own message ("You feel your wounds close.", "You feel hale.", "The dark is less dark to you."). Drinking while Raging is allowed (a barbarian can drink); a status that `blocks = ["drink"]` may be written later. Creatures do not drink (no AI for it; a later wave). Floor potions are picked up first (no `drink:floor`), unlike food.

**Unidentified potions** are labelled as scrolls are: `data/potion_labels.toml` in the `ScrollLabelDef` format ("a {label} potion", labels drawn from a colour list: murky green, bubbling red, smoky, pearly, viscous black, ...; the one `fg` of every unknown potion is its colour's, so the glyph's colour **is** the label and two labels never share a colour), drawn by `Labels::draw` after the rods' from the same `identification` stream (so scrolls, wands, and rods keep their labels); `WorldState::known_scrolls` holds known potion types as it holds wands' and rods'; the scroll of identify identifies a potion stack's type (`World::identify` already learns a labelled type). Prices are true (a shop's price hints, as ADOM's). The witch's kit potions are known (a chosen character knows their kit, as scrolls).

### 10.3 Marks by nature

Every potion names `marks = { <stat> = N }` (default by kind as the table says: a healing potion marks Toughness, a vision potion Willpower; N 5 for a common potion, 10 for a greater, 20 for an elixir), given as **sure marks** through the `drank` use (`chance = 100` in `stats.toml`, the potion's N as the count: `train_stat` gains a count argument, 1 for every use today). So a potion of might is 5 Strength marks and ten minutes of +2: the body remembers the effort. A harm potion gives none.

### 10.4 Where they come from

Loot by `level` and `frequency` as scrolls (`magic::loot::roll`), shops by the loot frequency (`mapgen/shop.rs`): the **alchemist** kind (exists in `shops.toml`, waiting) stocks potions first and buys them; general stores and provisioners a few healing and antidote potions among their wares; the temple's services unchanged. Creature packs: a hedge witch, a cultist, an apothecary carry one (gear lines in their files, the rosters' review). Quest rewards and the forgotten altar's "a healing draught" outcome may name one.

### 10.5 Elixirs

Rare (level 8, frequency 2: about one in a long game's loot, more in deep places and as rewards; never in a general store, at a very rare alchemist's stock at 2000 gold). Drinking one: `Stats::train(stat)` once (the score +1, and the potential lifted to the score if it was below), then the potential raised by a roll of `potential` ("1d2", one gameplay draw), "You feel stronger, and that you could be stronger yet." (the second clause only when the potential rose by 2). Hit points follow a Toughness rise through `raised_stats`, mana an Intelligence or Willpower one; carrying limits a Strength one; all as a trained rise does.

**+1d2 potential rather than +1 (the recommendation)**: with +1 the elixir moves the score and the potential together, so the gap the training rule works in is unchanged and an elixir past the potential is one point and nothing more; with +1d2 (average 1.5) half of elixirs widen the room for training by one, so an elixir has a lasting value beyond its point and a stat the player lives by can be pushed by finding elixirs *and* training, which is the ADOM rhythm the owner's design leans on (potions of potential there are the rare good find). It costs one draw and one line of data. The alternative (+1 fixed) is q10 in §16.

Elixirs are not drunk for marks alone: 20 marks of the stat are given too, because the owner asked that every potion marks.

### 10.6 What the player sees

- `q` opens the Drink list (the potions carried, by colour label or name, with their count); Esc closes it; a drink takes a turn and says its message; an unknown potion's name is learned as a scroll's is.
- The status bar and the character screen show a boost or a vision status by name (Fortified, Owl-eyed) as they show Blessed; no numbers or timers; the character screen's score is raised and marked as magic raises it today.
- The item description (`describe::appraised`) says what a known potion does in words ("Heals wounds at once", "Sharpens the eyes in the dark for a while", "Makes you stronger for good"), never dice or minutes; an unknown potion says "You do not know what this potion does."
- Look mode's pile names use the label. Prices in shops are true.
- Snapshots: `drink-list`, `drink-list-none` ("You carry nothing to drink."), `potion-description`, `potion-description-unidentified`, the character screen with Fortified (`potion-boost`), the status bar row.

## 11. Talents and class abilities (the coverage bar)

Every new system ships with its `[[system]]` tag and at least `min_talents` (2) talents in the PR that adds it (`docs/dev/progression.md`, coverage policy); every new ability names its systems. **Names and numbers are first guesses**, each on a hook this design adds or one that exists.

| System | Status | Talents (tree) | Hook |
|---|---|---|---|
| `weapon_skills`, `styles`, `combat`, `defence` | exist | the new rank tables are data; Weapon focus, Stylist, Fencing, Weapon mastery, Tempered keep their meanings and read the new arrays | — |
| **`weapon_abilities`** (new) | ships in W2 | **Quick recovery** (Weaponcraft, needs a weapon skill at 5): weapon abilities' cooldowns ease a third faster (`TalentMods::weapon_cooldown_percent` −33, read where the cooldown is afflicted: the severity-seconds are scaled as `study_percent` scales study) · **Early mastery** (Weaponcraft, needs level 9): rank abilities come one rank sooner (`had_by` reads rank + 1) · **Second blade's rend** (Finesse, needs Twin Blades 6): Rend rolls the off hand's dice too · **Brace of steel** (Guard, needs spears or staves 5): Brace's blow is at +2 and a braced foe is Staggered · class ability **Drillmaster** (warrior, at 25): the warrior's active abilities' cooldowns are halved |
| `spells` | exists | **Keen mind** (Spellcraft): the damage lift's `max_lift` is 10 points higher for the chosen school (`school_focus` gains a `lift` field) · the existing Empower, Archmage, School focus read the lifted cap | §7 |
| `protection` | exists | **Armour-breaker** (Might, needs axes, maces, or polearms 7): Sunder's cooldown is a minute shorter and a critical of yours ignores 2 Protection | §8 |
| **`stat_training`** (promoted: the daily check is a rule a talent can change, so the coverage policy's "channel, not a system" no longer holds) | ships in T2 | **Hard training** (Vigour): the daily check's `daily_factor` is 2 for you · **Quick study of the body** (Scholar): potions' marks are doubled · the existing Good learner stays XP's | §9 |
| **`potions`** (new) | ships in Q1 | **Thrifty drinker** (Delver): one potion in four is not used up (as Spell recall for scrolls; a draw of the gameplay stream only when the talent is held) · **Strong constitution** (Vigour, needs Toughness 13): boost and vision potions last half again (the afflicted severity-seconds × 1.5) · **Nose for draughts** (Scholar): an unknown potion's kind is told by its smell on pickup ("It smells of herbs: a cure of some kind."): the kind, never the type · class ability **Cauldron-wise** (witch, at 12; the existing Cauldron talent file is reviewed for overlap): potions she drinks heal or boost a fifth more |
| `hit_points`, `mana` | exist | unchanged; the heal and mana potions are items, not rules | — |

## 12. Saves, ids, determinism

| What | Saved / id | Note |
|---|---|---|
| Weapon ability ids | `WeaponAbilityId` indices in the content fingerprint; **not** saved (nothing on an actor names one: cooldowns are statuses by id, free blows are bits) | adding an ability changes the fingerprint, not the save format |
| Cooldown and effect statuses (Off balance, Overextended, Walled, Pinned, the six boosts, three visions, four resistances) | `StatusId` indices; new entries at their groups' ends in `status.toml` (saves change: the integrator's `FORMAT_VERSION` bump once per release) | a status with `vision` or `resist` is read by `Actor::eyes` / `Actor::resistances` only |
| `FreeBlows` from styles | not saved separately: `free_blows_of` folds the style's switch in at read time, as the talents' | an actor's saved bits stay what talents set |
| `next_stat_check` | `WorldState`, a `GameTime`, left out while no training rules | one new field |
| Stat marks' counts | `Actor::stat_marks` as today (u16 saturating; a potion's 20 marks fit) | — |
| Potion labels | derived (`World::labels`, never saved); known types in `WorldState::known_scrolls` (exists) | a new object file changes the fingerprint |
| Potions in packs | plain stacks; a `drink` table is data on the type | nothing per instance |
| Spell scales | derived at cast time; nothing saved | old replays that cast verify only if the caster's modifiers give lift 0 (a run at creation stats with Intelligence ≤ 11): the regenerated costs are unchanged, the damage is not |
| Monster Protection | data per type; nothing saved | replays through fights with changed types diverge (as any content change) |
| Determinism | deciding an ability never draws; Rend's dice, Pinned's turns, a harm potion's sickness, Thrifty drinker's keep, an elixir's potential are gameplay draws only when they arise; the daily check draws the `training` stream six times at its hour (always six, so the stream's position is a function of days passed); labels from `identification` after the rods' | fights without a qualifying rank keep their draws and hashes |

## 13. Tests, text snapshots, and the wiki

- **Core tests** per slice: `content/weapon_skill.rs` (the family rank-10 table and monotonic arrays; a style's `crit` to ×4), `content/weapon_ability.rs` (parse and refusals: unknown skill, a style's ability on a weapon skill, a trigger on an active, an effect with another kind's field, a cooldown status with a tick), `world/weapon_ability_tests.rs` (each catalogue entry on a hand-made fight: Rend once per attack and not on a bash, Staggering block refreshes, Pinned keeps the longer, Deep cut adds to bleeding to its max, Riposte through the style and the talent together once, a creature with the rank braces, Lunge over an empty cell and refused over a closed door, Piercing shot on to the next, Sunder against Pv 8, Shield wall's cost and Shield, every cooldown greys the row and eases; no draws while no rank qualifies; a save and a replay per active), `world/cast_tests.rs` (the lift by stat, `max_lift` holds, dice follow the lifted cap, criticals held to it, a cantrip with no scales unchanged, `to_hit` by Intelligence, the defaults equal today's reach and duration), `content/spell_def.rs` (a spell over its unlifted cap still refused; `max_lift` ascending or flat), the rosters' tests (`undead_roster_tests` and the tribes': Protection by body and level within the rule or in the exceptions list), `world/stat_training_tests.rs` (the new uses at their hooks, the daily check at 06:00 and only then, a week's rest makes seven checks in order, batching invariance, `daily_factor`, the potential's divisor, the message interrupts a rest, saves), `world/drink_tests.rs` (every kind, the labels and `known_scrolls`, identification by the scroll, marks given by count, Thrifty drinker's draw only when held, an elixir's score and potential and the hit points that follow, a harm potion's sickness, refusals, the witch's kit known, a replay).
- **Text snapshots** (`crates/warlock/src/ui/screen_snapshots.rs`; no goldens: nothing here is GPU-only): the abilities menu with the weapon skills group populated (`abilities-menu-weapon-skills`), a greyed cooldown row, `drink-list`, `drink-list-none`, `potion-description`, `potion-description-unidentified`, the character screen with a boost (`potion-boost`), the status bar with a cooldown and a boost.
- **Wiki** (`cargo xtask wiki`): `weapon-skill-benefits.md` and `style-benefits.md` regenerate; new `weapon-abilities.md` (`xtask/src/wiki/weapon_abilities.rs`: ability, skill, rank, kind, trigger or aim, what it does, cooldown in minutes) on the Weapon skills and Fighting styles pages; `spell-list.md` gains "Grows with" and the Magic page the lift table; `stat-training.md` regenerates and the Stats page states the daily rule; a new page **Potions** (`wiki/src/potions.md`, under Objects after Scrolls in `SUMMARY.md`) with `potions.md` (potion, kind, effect in numbers, duration, marks, level, frequency, value) and the elixir rule; the Creatures page's bodies paragraph.

## 14. What is deliberately not in this design

- **Hit points past level 20** (the report's option C) and **the magic-armour gap** (option D): both are owner decisions about the shape of the whole curve, named in §16 (q9, q11) and not designed here. Without C a plain fighter at level 60 still dies in 5 rounds; §3 and §9 make the fight winnable in 11 to 22.
- **The tier 4 to 9 spell list**: content the magic design planned (`docs/design/magic.md` §7 stops at tier 3); §7 shows why a level-60 caster needs it. An issue of its own.
- **Monster stat growth** (option B): the rosters' Strength formulas are a content design; §8 gives monsters Protection instead of taking their to-hit.
- **A stamina pool**, **creatures drinking potions**, **potions thrown as weapons**, **a potion of restoration for drains** (`docs/dev/stats.md` names it as not yet; it fits the `cure` kind with a `restores` field later), **brewing** (the witch's Cauldron), **rank-10 third abilities** (§16 q3).

## 15. Implementation plan

Each slice is one issue, one branch, one PR, with the spec section, its `docs/dev/` topic (`weapon-skills.md`, a new `docs/dev/weapon-abilities.md` created in W2 and listed in CLAUDE.md there, `spells.md`, `creatures.md`, `stats.md`, a new `docs/dev/potions.md` created in Q1), and the wiki in the same PR. Sizes: S a day, M two to three, L a week. "Needs" is the slice it builds on; "Ids / state / action" says what it adds that the overlap rule counts (at most about three PRs at once that add actions or save state).

| # | Slice | Sections | Needs | Ids / state / action | Size |
|---|---|---|---|---|---|
| **W1** | Weapon skill and style rank tables: the new arrays, the family content test, `crit` to ×4 for styles, wiki tables regenerated | §3, §5 | — | none (data) | S |
| **W2** | Weapon abilities, the system: `data/weapon_abilities/`, `WeaponAbilityId`, the loader and checks, `Trigger` and `World::weapon_trigger` at the ten lines, `WeaponDoes` kinds `free_blow`, `status`, `extra_damage` (the triggered half of the catalogue: Deep cut, Crushing blow, Brace, Point-blank, Stunning bash, Riposte, Rend, Cleaving sweep, Staggering block, Pinning shot, Stunning fist), Pinned in `status.toml`, the menu's greyed "fires by itself" rows, the creature-rank review, `docs/dev/weapon-abilities.md`, the `weapon_abilities` system with Early mastery, Second blade's rend, and Brace of steel (three on W2's own hooks: the bar is met without a cooldown) | §4.1–4.4, §6, §11 | W1 | ids (fingerprint); 1 status | M |
| **W3** | Weapon abilities, the actives: `Ability::Weapon`, `use:ID` resolution, `Unavailable::NotSoHeld`, cooldown statuses (8), the kinds `reach_blow`, `sure_critical`, `ignore_defence`, `ignore_protection`, `pierce_shot`, `buff` (Vital strike, Sunder, Lunge, Piercing shot, Shield wall, Feint, Mighty swing), Quick recovery and Drillmaster live | §4.5, §4.6, §6 | W2 | **action** (an arm of `use:`); 9 statuses | M |
| **S1** | Spell scales: `[spell.scales]`, the defaults, `max_lift`, the lifted cap in `SpellDice`/`cap_of`, `to_hit` by stat, `reach_step`/`presence_step` retired, the first list authored (evocation damage by Intelligence and Willpower, hexes' duration by Presence and Willpower, heals by Willpower), Keen mind, the wiki column | §7, §11 | — | none (derived) | S–M |
| **P1** | Monster Protection: the body rule as a content test with the exceptions list, the data pass over every creature file (shared and campaigns), the Creatures page paragraph, Armour-breaker | §8, §11 | W1 (and T1 landed, by the plan's waves) | none (data) | M (many files, one line each) |
| **T1** | Stat training uses and chances: the six uses at their hooks, `train_stat` with a count, the chances, wiki | §9.1 | — | none | S |
| **T2** | The daily check: `daily_factor`, `next_stat_check`, the `live` moment, the message, Hard training, the `stat_training` system | §9.2, §11 | T1 | **state** (one field) | S–M |
| **Q1** | Potions: `[object.drink]` with `heal`, `cure`, `mana`, `harm`, `marks`; `Action::Drink`, `q`, the Drink list, `world/drink.rs`, labels and identification, the first six potions and the murky draught, the witch's kit renamed, shops and loot, `docs/dev/potions.md`, the `potions` system with Thrifty drinker and Nose for draughts, the Potions wiki page | §10.1–10.4, §11 | T1 (the `drank` use) | **action**; labels (derived) | M |
| **Q2** | Potions: `boost`, `vision` (`StatusDef::vision` in `Actor::eyes`), `haste`, `resistance` (`StatusDef::resist` in `Actor::resistances`); the thirteen boost, vision, and resistance statuses; Strong constitution, Cauldron-wise | §10.1 | Q1 | 13 statuses | M |
| **Q3** | Elixirs: the `elixir` kind, the six elixirs, their loot and rewards, the altar outcome's reference, Quick study of the body | §10.5, §11 | Q1, T2 | none | S |
| **D1** | The cross-checking pass: the balance probe re-run as a documented `#[ignore]` test (`world/balance_probe.rs`, the report's method, kept this time) with its tables in `docs/perf.md`'s neighbour `docs/balance.md`; the spec's §8 sections and CLAUDE.md's topic index for the two new topics | all | everything above | none | S |

**Waves** (never more than three at once that add an action or save state):

1. **W1, T1, S1** (data and derived rules; no overlap: `weapon_skills.toml`, `stats.toml` and five hook lines, `spells/` and `spell_def.rs`).
2. **W2** (ids, one status), **T2** (state), **Q1** (action): the three counted ones; **P1** with them (data only, many creature files, nothing else touches them).
3. **W3** (action, statuses), **Q2** (statuses): `status.toml` is a hot file, each adds its own group.
4. **Q3**.
5. **D1**, last.

## 16. Open questions, each with a recommended default

Each stays listed with the design's recommendation as the default; the owner decides in §17, and the implementing PR states what it chose.

| # | Question | Recommended default | Alternative |
|---|---|---|---|
| q1 | **Rank thresholds**: weapon skills at 5 and 9, styles at 4 and 7? | yes: a weapon skill's 9 is 7 665 marks in, a style's 7 half that; both come in the middle game for a fighter who sticks to one weapon | the owner's "4/7/10" for both, with a third ability at 10 (q3) |
| q2 | **Cost model for actives**: cooldown statuses? | yes (§4.5) | a shared Stamina pool in `resources.toml` (max 10, a point back every two turns, costs 2 to 5), which W3 would add instead of the cooldown statuses, plus its two talents |
| q3 | **A third ability at rank 10**? | no: rank 10 is the numbers' capstone (×4 criticals, +12 damage, penalty-free twin blades); 18 abilities already | one per style at 10 (a Duelist's "Measure": Parry counts against missiles; Great Weapon's great cleave is already its 10) |
| q4 | **One ability for Twin Blades, Sword and Board, Marksman, Brawler**, none of their own for Skirmisher and Staff Fighter? | yes for the first draft: the weapon families cover the last two (Point-blank, Brace, Lunge) | a second each at 7: Twin Blades "Whirl" (a sweep with both hands), Sword and Board "Shield rush" (a bash that knocks back), Marksman "Aimed shot" (threat 14–24, double time), Brawler "Throw" (knockback), Skirmisher "Hit and run" (a throw that hits makes the next step cost half), Staff Fighter "Trip" (a critical pins) |
| q5 | **Creatures use triggered abilities** by their ranks? | yes, at once in W2 (the hook reads the actor, not the player; the rosters' ranks are reviewed in the PR: today few shipped types reach 4 or 5) | player only, with a `player_only` flag on the ability |
| q6 | **Spell lift maxima** `[40, 50, 60, 70, 80, 80, …]` and weights of 10 % Intelligence, 5 % Willpower per point? | yes: a +8 Intelligence caster lifts a tier-3 spell 70 %, so a found spell stays useful two tiers on, and the tier 4–9 list (an issue of its own) remains the way up | lower maxima (50 % flat) and a dice-adding rule instead of a cap lift |
| q7 | **The daily check** at 06:00 with `daily_factor` 3? | yes (§9.2) | a check whenever a stat's marks reach `need` ("ripe"), fired at the end of the player's turn; or a check at every rest's end |
| q8 | **Protection bodies** and the four lines? | yes, with the exceptions list (bosses held to the bony line) | three bodies (merge bony into hide at L ÷ 7), or Protection only from level 20 |
| q9 | **Hit points past level 20** (the report's option C)? | not in this design; the owner's call. If wanted: `hp_level` 40 with half the class hit points per level past 20, in T2's neighbourhood (one constant and the formula's one line) | leave it: the design's fights at level 60 are won in 11 to 22 rounds and lost in 5 |
| q10 | **Elixir potential +1d2** or +1? | +1d2 (§10.5) | +1 |
| q11 | **The magic-armour gap** (option D: cap what Defence affixes add by item level, or count the best only)? | not in this design; a magic-items pass of its own. The monster to-hit of +18 at level 60 already needs a natural 24 against Dv 39 | — |
| q12 | **Drink key `q`** and no floor drinking? | yes: `q` is unbound, the list is the pack's | `D`, or Eat's list taking potions too |
| q13 | **Potion labels by colour** with the glyph's colour the label? | yes: it reads at a glance and costs no name | the scroll scheme's nonsense words ("a potion labelled ZUNBAR") |

## 17. Owner decisions

*(left for the owner; one line per question of §16, with the sections changed by each)*

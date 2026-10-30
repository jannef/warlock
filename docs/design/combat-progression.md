# Warlock — Combat progression: weapon skill and style ranks and their abilities, spell scaling by stats, monster Protection, stat growth between levels, potions and elixirs

Design document for the owner's review (2026-10-27). Written against `origin/main` at 3eb489d7, the balance investigation of the same day (the probe over all 400 creature types and four classes at levels 1 to 60; its findings are summarised in §2), the approved progression design (`docs/design/progression.md`, §6 for stat training) and magic design (`docs/design/magic.md`, §5 for the caps), and the systems as built (`docs/dev/weapon-skills.md`, `combat.md`, `spells.md`, `stats.md`, `status-effects.md`, `resources.md`, `magic-items.md`, `scrolls.md`, `hunger.md`). Nothing here is implemented; §15 cuts it into slices. **Every number is a first guess, kept in data**, with its reasoning beside it; none is a balance decision. The owner's scope note: **none of this goes into 0.7.0**. **Revised the same day with the owner's answers to the thirteen open questions** (§17 quotes them; the body below applies them: a stamina pool and an energy cost for actives, new style abilities and rank-10 capstones with Spell reflection, creatures treated as the player, hit points past level 20 by class data only, elixirs at +1d2 potential, one Consume command for food and drink, potion colours by effect hue).

**The owner's request** (2026-10-27, verbatim): "feature: address balance issues discusses earlier - make higher ranks in weapon skill more powerful. biggen numbers and perhaps some abilities. - same for fighting styles. They should also give some abilities. - the abilities weapon skill and fighting styles could be something like riposte (free attack when parrying enemy attack), rend (extra attack worth of damage when both weapons of two-weapon fighting hit) etc. sword and board mayby give enemy penalty to next attack speed after they get parried etc. Triggered abilities, and active abilities activetable from ability menu. - spells perhaps should perhaps pick up benefits from multiple mental stats to allow better scaling. this is just something that needs to be authored per-spell. - increase PV of higher level monsters (just hand tweak per monster) - make sure stat training marks are plentiful, and give stats change to grow even between level ups - introduce potions. healing, cures, temp stat boosts, visiontypes (ultravision) Each potion gives also training marks. Also introduce elixirs that increase stats by 1 (also increase potential by 1 or more? 1d2?) this does not go into 0.7.0".

## Contents

1. Summary: the decisions needed
2. What exists today, and what this builds on (the balance findings)
3. Weapon skills: the new per-rank tables
4. Rank abilities: triggered and active (the system), the stamina pool, energy costs
5. Fighting styles: tables, abilities, and rank-10 capstones
6. The catalogue, interactions, and abuse cases
7. Spell scaling from the mental stats
8. Monster Protection by level and body
9. Stat training: plentiful marks, growth between levels; hit points past level 20 (class data)
10. Potions and elixirs (Consume, colours by hue)
11. Talents and class abilities (the coverage bar)
12. Saves, ids, determinism
13. Tests, text snapshots, and the wiki
14. What is deliberately not in this design
15. Implementation plan
16. Open questions, each with a recommended default (answered: §17)
17. Owner decisions (2026-10-27)

## 1. Summary: the decisions needed

| # | Piece | Proposal (the default if the owner only nods) | Section |
|---|---|---|---|
| 1 | Weapon skill ranks | ranks 0 to 5 as today; ranks 6 to 10 grow faster. Rank 10 to-hit +7 / +6 / +5 by family (today +5 / +4), damage +5 / +8 / +12 (today +3 / +4 / +6), threat up to 3 wider (today 2), speed up to 30 % (today 20), Parry up to +6 (today +4). The to-hit stays moderate so that trained stats (§9) do not saturate the 2d12 curve; the growth goes into damage, where monster Protection (§8) can answer it | §3 |
| 2 | Rank abilities | a new registry, `data/weapon_abilities/<id>.toml`, one file per ability: **triggered** ones fire on one of eleven combat events through one hook (`World::weapon_trigger`), are instant, and cost no time and no stamina; **active** ones are rows of the abilities menu's existing *Weapon skills* group, aimed as class actives are, paid with **stamina** and an **energy cost** (time: a percent of the action they are, so never instant), and never a key of their own. Unlocked at ranks the implementer chooses **per weapon group and per style** (they need not be the same for every group; the first guess is 5 and 9 for a weapon skill, 4 and 7 for a style, 10 for a style's capstone) | §4 |
| 3 | Stamina | a new alternative resource (`data/resources.toml`, the existing system): maximum 10 + the Toughness modifier + 1 per 10 levels, a point back every two base turns (resting too), held by every character of a fighting class from level 1 and by anyone else once a rank unlocks an active; `Sta 7/12` on the status bar after the other pools. Two talents and a class ability touch it | §4.5 |
| 4 | Styles | their numbers grow at ranks 6 to 10 as the weapon skills' do (Duelist Parry to +7, Great Weapon damage to +10 and a ×4 critical at 10, Twin Blades penalty-free at 9, Sword and Board bash 35 %); every style has at least one ability of its own and most two, the owner's three among them: **Riposte** (Duelist), **Rend** (Twin Blades), **Staggering block** (Sword and Board); and capstones at 10 where one fits: **Spell reflection** (Sword and Board), **Measure** (Duelist), the existing great cleave (Great Weapon) | §5, §6 |
| 5 | The catalogue | 27 abilities: 10 on the weapon skill families, 17 on the styles; every one on an existing mechanism (free blows, statuses, sure criticals, the one blow path) but Spell reflection's one new trigger and effect. No chains: a free blow brings none, Rend is once per attack, Staggered refreshes instead of stacking, a reflected spell is never reflected again | §6 |
| 6 | Creatures | equal treatment: the same data and the same triggers fire for a creature with the rank as for the player; creatures hold no stamina, so they use no actives (an AI slice may later give them actives under a cooldown status; not in this plan) | §4.1, §6 |
| 7 | Spell scaling | an authored `[spell.scales]` table per spell: four dimensions (`damage`, `to_hit`, `reach`, `duration`), each a map of stat → weight in percent of the modifier. Damage scaling **lifts the cap**: the tier ceiling grows by the scaled percent up to a `max_lift` per tier (40 % at tier 0 to 80 % from tier 4), and the dice follow the lifted cap. The strict cap stays a rule: nothing passes the *lifted* ceiling, and the lift is bounded in data | §7 |
| 8 | Monster Protection | a rule of thumb by level and body (none, hide, bony, armoured: 0, (L−10)/8, L/6, L/4+1) applied as a data pass to `natural.protection`, with a list of exceptions to hand-tune; shown against the player's damage after §3 | §8 |
| 9 | Stat training | chances raised (a blow 5 → 15 %, hurt 10 → 20 %, ...) and six uses added (parry, block, critical, killed, drank, elixir); a **daily check** of every stat at 06:00 by the game clock (the `training` stream, six draws) beside the level-up's, with `need × daily_factor` (3) so a trained stat rises between levels about every two to three weeks of game time early and rarely past its potential; potentials stay the soft cap | §9 |
| 10 | Hit points past level 20 | no new mechanic: data only. `hp_level` 20 → 50 in `experience.toml` and the thief's class `hp` 3 → 4; a level-60 warrior goes from about 88 to about 187 hit points | §9.4 |
| 11 | Potions | a `[object.drink]` table (like `read`, `wand`, `rod`): heal, cure, boost, vision, haste, resistance, mana, and elixir kinds; drunk through the existing **Eat** command, renamed **Consume** (its list shows food and drinks; no Drink key, no second list), one turn; unidentified by a drawn word and a colour by the effect's hue ("a fizzy red potion"), identified by drinking or the scroll of identify; every potion gives training marks by its nature; sold by alchemists and general stores, found as loot | §10 |
| 12 | Elixirs | rare potions: +1 to a stat for good and **+1d2 potential**; no price in shops (quest rewards, altars, deep loot; a very rare alchemist's stock) | §10.5 |
| 13 | Talents | a new system `weapon_abilities` (four talents, one class ability), a new system `stamina` (two talents), a new system `potions` (three talents), the `stat_training` channel promoted to a system (two talents: the daily check is now a rule a talent can change); spells and monster Protection extend systems already in the list | §11 |
| 14 | Slices | fifteen issues in five waves (the stamina pool, the Consume rename, and the hit-point data pass among them), never more than three at once that add an action or save state | §15 |

**Key choices, and why**

1. *Numbers first, abilities second.* The balance report's finding is arithmetic (a rank cap of +5 against monsters that reach +18); the per-rank tables fix the arithmetic in data, and the abilities give ranks 4 to 10 something to look forward to beyond "+1 more". Both are needed: abilities alone would not move the 2d12 curve, numbers alone would make rank 10 a bigger rank 5.
2. *Triggered abilities ride the free-blow and status machinery.* Riposte is already a free blow (`FreeBlows::riposte`, the Riposte talent); a style's rank switches the same bit on through the one reader, `World::free_blows_of`. Staggered is already a status. The new code is the registry, one trigger hook, and a handful of effect kinds.
3. *Stamina and time pay for actives; triggers are free.* The owner chose a pool (§17 q2): one small stamina bar, on the existing resources system, that every fighter holds. An active also costs time, a percent of the action it is, so no active is instant; a triggered ability is instant and costs nothing, as a free blow does. Cooldown statuses are kept only as a per-ability limiter if one ability needs it; none in the catalogue does.
4. *Stats lift spell caps; they do not add damage past them.* The magic design's "strict cap" is kept as a rule, made a function of the caster's stats with a bounded lift per tier, authored per spell so a cantrip stays a cantrip.
5. *Protection is data.* The report shows monsters with Pv 0 to 5 against player damage that the new ranks double; the rule of thumb makes a pass over `assets/data/creatures/` reviewable at a glance, and the roster formulas' tests pin it.
6. *Growth between levels needs no new channel.* The marks exist; one more moment at which they are checked, at a fixed hour, is the smallest rule that gives stats a life between levels and keeps the owner's no-anti-farming rule (nothing is lost, nothing is capped, time is the only price).
7. *Potions are the scrolls' twin, and drunk as food is eaten.* Labels, identification, the loot and shop draws, and the "a read trains Intelligence" hook all exist for scrolls; the list and the command exist for food. Potions take each with one new table on the object and no new action or key: Eat becomes Consume.

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

The report's tuning options A to F map onto this design as: **A** (a growth term for player to-hit) → §3's ranks and §9's trained Strength; **E** (monster Protection, a damage term) → §8 and §3's damage; **F** (spell dice past 3) → §7; **C** (hit points past level 20) → §9.4, as class data only (the owner: no new mechanics); **D** (narrow the magic-armour gap) is out of scope: the owner chose not to go there (§17 q11).

**What this builds on** (each the one place; nothing here adds a second):

| Part | Today |
|---|---|
| Weapon skills and styles | `data/weapon_skills.toml`: `[Benefit; 11]` per skill (`to_hit`, `damage`, `threat`, `crit`, `speed`, `parry`, `reload`, `shield`; styles add `ease_main`, `ease_off`, `bash`), ranks 0 to `MAX_RANK` 10, marks on the `melee`/`missile` curves, defaults between skills. `Actor::attack_benefit(kind, with)` is the sum a profile reads; `Actor::defence` adds Parry and Shield; `World::attack_price` takes `speed` off. The file's own comment: "Rank 10 is where later active or triggered abilities will hang" |
| Free blows | `World::second_blow(striker, target, Cue)` is the one place every blow beyond an attack's first is decided (`Cue::First`, `Attacked { parried }`, `Cleaved`, `Leaves`, `Approaches`); `FreeBlows { cleave, great_cleave, riposte, opportunist, gatekeeper }` on `Actor::free_blows`, read only through `World::free_blows_of`; "a free blow brings none of its own"; the Riposte and Opportunist talents switch them on |
| Statuses | `data/status.toml`: `combat`, `cost`, `blocks`, `follows`, `penalty` (a raise too), `eases`; a cooldown is a status with `eases` and no tick (Spent, Winded, Drained); `World::afflict` the one way to give one; Staggered (`cost = 150`, one turn, `stacking = "refresh"`) exists |
| Abilities | `warlock_core::ability`: `Ability` variants, `Source` groups (Racial, Birth sign, Class, Spells, Talents, Skills, **Weapon skills** (Shield bash is there), Items, Here), `Aim::{Nothing, Direction, Cell}`, `Unavailable::{NoShield, NotYet, While, Short, NoMana}`; `use:ID[:DIR|:X,Y]` is the action text of every class active and spell |
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

**Rank thresholds are the implementer's, per weapon group** (§17 q1): the ranks at which a group's abilities unlock, and the shape of its rows, need not be the same for every group (a light blade may come into its own sooner than a great blade); the families above and the 5 / 9 thresholds of §4 are first guesses, kept in data. The other skills follow their family's pattern (the implementer writes them out; a content test holds every array to its family's rank-10 value in this table and to "never less at a higher rank"). `MAX_THREAT_WIDENING` (10) and the data checks already allow these values; `MAX_SAVING` (50) holds the speeds.

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

Read with the report's monster side (the plain warrior dies in 18 rounds at level 30 and 4 at 60; Parry +2 more lifts those to 27 and 5): the fight at level 60 goes from "36 rounds to kill, dead in 4" to "11 to 22 rounds to kill, dead in 5". Still lost without hit points past level 20: §9.4's class data (the owner's answer to q9) lifts the warrior from about 88 to about 187 hit points at level 60, so "dead in 5" becomes about 10. The thief with magic gear at level 60 goes from 20 rounds to 15 (short blades +7 / +5 and the wider threat). The wizard's melee is not meant to be fixed here (§7).

**What the player sees.** Nothing new: ranks and the marks bar on the weapon skills screen (`W`), as today; the ready attacks on the character screen show the new to-hit and damage as they show today's (`PerceptionView::own_attacks`). The wiki's `weapon-skill-benefits.md` tables regenerate from the data.

## 4. Rank abilities: triggered and active (the system), the stamina pool, energy costs

### 4.1 The two kinds

| Kind | Fires | Cost | Where it lives |
|---|---|---|---|
| **Triggered** | on one combat event (§4.3) while the actor has the rank and, for a style's, fights in that configuration | **none, and instant**: no energy, no stamina, like a free blow (decided without a draw; its blow or status draws as that blow or status would). The owner: "triggers can be instant and take no energy" | nowhere the player chooses: it is listed in the abilities menu greyed with "fires by itself" (so the player can read what they have), never usable |
| **Active** | when the player picks it from the abilities menu's *Weapon skills* group (`Source::WeaponSkill`, where Shield bash is) and aims it (`Aim::Nothing`, `Direction`, or `Cell`, per ability) | **stamina** (§4.5, `Unavailable::Short` while the pool holds less) and an **energy cost**: a percent of the action it is (a melee blow's `attack_price`, a shot's `shot_price`, a wait's), so an active is never instant; never a key of its own (CLAUDE.md "Hard rules") | `Ability::Weapon(WeaponAbilityId)`; the action text is the existing `use:ID[:DIR|:X,Y]`, resolved after the class actives, the sign gifts, and the spells |

**Who has it**: `had_by` = the actor's rank in the ability's skill is at least `rank` (a `Weapon` talent's `ranks` bonus counts, as it does for benefits: Stylist and Fencing make an ability come sooner), and for a style, the actor is in its grip now (`Actor::style(kind, with)`), else `Unavailable::NotSoHeld` ("your hands are not so held"). The `rank` is per ability file, so each weapon group and style sets its own thresholds (§17 q1).

**Creatures, equal treatment** (§17 q5): creatures have ranks (`Actor::weapon_skills` from the type) and so **have the triggered abilities** by the same rule, the same data, and the same hook, which is why every triggered effect is written as "the actor", never "the player". Creatures hold no stamina (resources are never granted to them, `docs/dev/resources.md`), so they use **no actives** in this design; when an AI slice teaches them to, the simple rule is one active, then a cooldown status of 30 seconds (`breathless`, the martial actives' existing model) instead of a pool. That slice is not in §15.

### 4.2 The data: `data/weapon_abilities/<id>.toml`

One `[[weapon_ability]]` per file, read through `ContentFiles::table("weapon_abilities")` (campaigns may add their own), ids `WeaponAbilityId` indices in path order, in the content fingerprint (nothing saved names an ability; the index is for the menu and the tests). **Why a directory and not `weapon_skills.toml`**: the skills file is one hot file holding every skill's arrays, and the merging playbook keeps churning content tables as directories, one file per entity (talents, spells, mishaps, rod effects, uniques). An ability names its skill; the skill does not list its abilities. The loader checks: the skill exists, `rank` is 1 to 10 (any rank: the thresholds are per file, so per group), a style's ability names a style, a weapon skill's a weapon skill, and the effect's fields fit its kind (as `RodDoes` is checked).

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
rank = 9                         # the spear and staff group's own threshold (§17 q1)
kind = "active"
aim = "direction"
stamina = 3                      # points of the stamina pool (§4.5)
energy = { action = "melee", percent = 100 }   # the ActionKind priced (data/actions.toml), and the percent of its price
systems = ["weapon_abilities", "weapon_skills"]
[weapon_ability.effect]
kind = "reach_blow"
reach = 2
```

Fields: `id`, `name`, `description`, `skills` (1 or more), `rank`, `kind` (`triggered` | `active`), `trigger` (triggered only), `aim`, `stamina`, and `energy` (active only; `energy` is required, `percent` 50 to 300, so no active is free of time; a triggered ability has neither: the loader refuses a trigger with a cost), an optional `cooldown` (active only: a status with `eases` and no tick, the per-ability limiter if one ability ever needs one beside its stamina; none in §6 does), `systems`, `[weapon_ability.effect]` with only its kind's fields. An ability is listed in the abilities menu by its skill's name as a sub-heading ("Long blades: Deep cut (fires by itself)", "Spears: Lunge"), the group ordered by skill file order then rank.

### 4.3 Triggers: one hook, eleven events

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
| `spell_blocked` | a hostile's spell that rolls an attack against the actor's Defence (an `attack` or `touch` resolution: bolts, rays, touches; a wand's too) missed by its Shield | `cast.rs`, where the spell's `Profile::bare` roll is attributed (`Defence::why_missed`, as a blow's) | the target |

(`both_hands_hit`, `approaches`, and `spell_blocked` are specific cases the catalogue needs; the enum is closed, `Trigger::ALL` in wiki order.) The skill concerned is the attack's for the attacker's triggers (`AttackSkills` is already on the resolve path) and the actor's wielded weapon's and style's for the target's (what they hold decides what they can do with it: a riposte with a staff is the staff's). A trigger whose effect makes a blow goes through `World::second_blow` as a `Cue` or through `strike_free` as a `Free` kind, so the rule "a free blow brings none of its own" holds for everything here.

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
| `parry_missiles` | — | a switch, as `free_blow` is (no trigger of its own): the holder's Parry counts against shots and throws as it does against blows (`Actor::defence` for a missile `Target`, today Parry's one exclusion) |
| `reflect_spell` | `per_turn` 1 | on `spell_blocked`: the spell is cast back at its caster from the actor's cell, as the caster cast it (its caster level, its lifted cap, §7; its own attack roll against the caster's Defence; its mana is not spent again), at most `per_turn` times a turn of the actor's; a reflected spell is marked and never reflected again (the free-blow rule's twin) |

Every kind is a variant of `WeaponDoes` with its one read; a kind with no catalogue entry is not written (no speculative effects).

### 4.5 The cost model: stamina and time

The owner's answer (§17 q2): a stamina pool, and an energy cost on every active so none is instant; triggers stay instant and free. An active ability therefore pays **two things**, both after its own refusals (a refused use costs nothing):

1. **Stamina**, its `stamina` points, through `World::spend_resource` (the one place pools are spent). With less in the pool the menu greys the row with `Unavailable::Short` ("you are short of breath").
2. **Time**, its `energy`: the price of the action it is (`attack_price` for a melee blow, so the weapon's speed, the style's, and the stance still count; `shot_price` for a shot; `wait` for a stance like Shield wall), times `percent` ÷ 100 (`percent_of`, at least 1). A Mighty swing at 150 % of a blow takes half a blow longer than a blow; nothing is free of time.

**The pool** (a new entry at the end of `data/resources.toml`: the file order is the save order, so it is appended, never inserted; the existing system, `docs/dev/resources.md`):

| Field | Value (first guess, kept in data) | Why |
|---|---|---|
| `id`, `name`, `short` | `stamina`, "Stamina", `Sta` | the status bar's label |
| `max` | 10, plus **`max_from = { stat = "toughness", per_levels = 10 }`**: the Toughness modifier and a point per 10 levels (at least 5) | the one new field of the system, read in the one hook, `Actor::resource_max`; a Toughness 14 fighter holds 12 at level 1 and about 20 at level 60 (Toughness 18). The talents' `resource_max` add on top as today |
| `regenerates` | 12 (a point every two base turns) | a full pool is three or four actives; then one active of 4 every eight turns, so the strongest active cannot be looped and needs no cooldown |
| `resting` | true | a rest fills it; time is the price (the no-anti-farming rule) |
| `show_max` | true | `Sta 7/12` on the status bar, after MP and the class pools, and on the character screen, as every pool shows; never the regeneration |
| `builds`, `ends_at_zero`, `status` | none, false, none | it sits at 0 and comes back; no class identity rides on it |

**Who holds it**: a new granted talent `stamina` ("Trained wind", `grants_resource { resource = "stamina" }`, the existing grant path) in the level-1 row of the fighting classes' level tables: **warrior, barbarian, paladin, ranger, monk, thief, bard**. Anyone else (a wizard who trains staves to their group's active rank, a commoner with a spear) is granted it the moment a weapon rank first unlocks an active: `World::rank_up` calls `focus::hold` for it, the **third grant place** (`docs/dev/resources.md` lists two today, `class::hand_out` and `World::grant_class`; the topic file gains this one with its test). A barbarian holds Rage and Stamina at once, as pools allow; Raging blocks casting, not weapon actives. Creatures never hold it (§4.1).

**Cooldown statuses** are kept only as an optional per-ability limiter (`cooldown`, §4.2) if a later ability needs one beside its stamina; no ability in §6 has one, so W3 adds no cooldown statuses.

Alternatives weighed (the owner chose the pool):

| Model | For | Against |
|---|---|---|
| **Stamina pool and energy cost** (chosen) | one bar, costs tunable per ability, regeneration bounds the loop; on the existing resources system (no new state kind); time keeps every active from being instant | a sixth pool and a new system to cover with talents (§11 does) |
| Cooldown statuses (the first draft) | the martial class actives' model; no new bar | several statuses on the status bar at once; one timer per ability to tune |
| Mana | exists for everyone | martial characters have 1 to 60 mana; the design keeps mana magic's |

### 4.6 What the player sees

- The abilities menu's *Weapon skills* group lists Shield bash and, under a sub-heading per skill, the abilities the player's ranks give: an active as a usable row with its stamina cost ("Lunge (3 stamina) — a long thrust at a foe two paces off"), greyed with its reason while the pool is short or the hands are not so held; a triggered one greyed with "fires by itself". A weapon skill's rank that unlocks one says, with the rank's own line, "You feel more skilled with spears. You have learned the lunge." (the ability's `name` in lower case); the first that grants the pool adds "You learn to pace your breath." The stamina cost is shown, as a spell's mana cost is; the energy cost is not (time costs are never shown).
- Messages: a triggered blow is said as a free blow is ("You parry the kobold's spear. You strike back at the kobold."); Rend: "Your blades rend the orc."; Staggering block: "The orc staggers from your shield."; Spell reflection: "Your shield turns the bolt back at the shaman."; an active: its blow's own message.
- The status bar shows `Sta 7/12` while the player holds the pool. Nothing shows a rank threshold or a chance.
- The character screen's ready attacks are unchanged (situational blows are not part of a profile, `docs/dev/combat.md`).

## 5. Fighting styles: tables, abilities, and rank-10 capstones

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

A style's to-hit at +5 with the weapon's +7 and trained Strength lands a rank-10 duelist at about +20 at level 60 (98 % against Dv 21): the price of a rank-10 style, which took 15 345 marks on top of the weapon's, and the reason the weapon tables stay moderate. Styles' abilities come, as a first guess, at **rank 4 and rank 7**, and a **capstone at rank 10** where one fits (§17 q3): **Spell reflection** for Sword and Board, **Measure** for Duelist, and Great Weapon's existing great cleave; the other styles' rank 10 stays the numbers' capstone (×4 criticals, penalty-free twin blades, +6 to-hit and 25 % speed for the marksman). The ranks are per style and the implementer's (§17 q1).

**Spell reflection** (Sword and Board 10, triggered, `spell_blocked` → `reflect_spell`): when a hostile's spell that rolls an attack against the actor's Defence (a bolt, a ray, a touch; a wand's bolt too) misses **by the Shield** (the miss attribution, as a blocked blow's), the spell is turned back at its caster: it is cast again from the actor's cell at the caster, with the caster's own power (caster level, lifted cap, riders), and rolls its own attack against the caster's Defence; it costs the caster no more mana and the actor nothing. **Cap**: once a turn of the actor's (`per_turn` 1), and a reflected spell is never reflected again (two shield-bearing casters cannot bat a bolt back and forth). **What it does not touch**: area bursts, cones, and walls (no attack roll: nothing for a shield to block), hexes and charms that ask a check, and a spell the Shield did not cause to miss (a miss by Parry or Dexterity is a plain miss). The Shield needed is the one the style already needs: a shield held and the grip of Sword and Board.

## 6. The catalogue, interactions, and abuse cases

Twenty-seven abilities, each on §4.4's kinds. Ranks are first guesses per group and style (§17 q1). Triggered abilities cost nothing and take no time; an active's cost is its stamina and its energy, a percent of the action it is (§4.5). Names in the game's register; descriptions are the menu's line. No ability is called, or implies, an "aimed" shot: every shot in the game is aimed (the owner's rule, §17 q4). **All first guesses, kept in data.**

| # | Ability | Skills (rank) | Kind: trigger or aim | Effect | Stamina / energy |
|---|---|---|---|---|---|
| 1 | **Deep cut** | daggers, short blades, long blades, great blades (5) | triggered: `big_hit` | `status` bleeding +1 degree on the target (a big hit with a blade wounds as a critical does) | — |
| 2 | **Vital strike** | daggers, short blades, long blades, great blades (9) | active: direction | `sure_critical`: the next blow at that foe is a critical if it hits | 4 / melee 100 % |
| 3 | **Crushing blow** | axes, maces, polearms (5) | triggered: `big_hit` | `status` staggered on the target (its next action costs half again) | — |
| 4 | **Sunder** | axes, maces, polearms (9) | active: direction | `ignore_protection`: one blow that armour does not soak | 4 / melee 125 % |
| 5 | **Brace** | spears, staves (5) | triggered (`free_blow`: gatekeeper) | a hostile that steps up beside the actor takes a blow as it comes (the existing gatekeeper) | — |
| 6 | **Lunge** | spears, staves (9) | active: direction | `reach_blow` 2: a blow at a foe two cells off over an empty cell | 3 / melee 100 % |
| 7 | **Point-blank** | bows, crossbows, slings, thrown (5) | triggered: `approaches` | a shot or throw at a hostile that steps up beside the actor (the gatekeeper's geometry, a shot instead of a blow; needs ammunition ready) | — |
| 8 | **Piercing shot** | bows, crossbows, slings (9) | active: cell | `pierce_shot`: a shot that flies on through the first creature it hits to the next on its line | 4 / shoot 100 % |
| 9 | **Stunning bash** | shield_bash (5) | triggered: `hit` (a bash that hits) | `status` staggered on the target | — |
| 10 | **Shield wall** | shield_block (9) | active: nothing | `buff` *Walled* 5 degrees: Shield +3 to Defence, attacks cost 125 %, a degree a turn | 5 / wait 100 % |
| 11 | **Riposte** | duelist (4) | triggered (`free_blow`: riposte) | the existing riposte: a parried attack on the actor earns a blow back once it is over | — |
| 12 | **Feint** | duelist (7) | active: direction | `ignore_defence` parry and shield: the blow is rolled as if the foe had neither | 2 / melee 100 % |
| 13 | **Measure** | duelist (10, capstone) | triggered (`parry_missiles`) | the duelist's Parry counts against shots and throws as against blows: a blade that has learned the distance turns arrows | — |
| 14 | **Rend** | twin_blades (4) | triggered: `both_hands_hit` | `extra_damage` main_hand: the main hand's dice once more on the target | — |
| 15 | **Whirl** (new) | twin_blades (7) | active: nothing | `sweep` −2: a blow with each blade in turn at every hostile beside the actor, clockwise from the north | 6 / melee 150 % |
| 16 | **Cleaving sweep** | great_weapon (4; `great_cleave` at 10, the capstone) | triggered (`free_blow`: cleave) | the existing cleave: a kill carries the blow on to the next foe beside; at rank 10 it goes on while the blows kill | — |
| 17 | **Mighty swing** | great_weapon (7) | active: direction | the blow's own damage dice rolled twice (not a critical: bonuses and riders once), at −2 to hit | 5 / melee 150 % |
| 18 | **Staggering block** | sword_and_board (4) | triggered: `blocked` or `parried` | `status` staggered on the attacker: its next action costs half again (the owner's "penalty to its next attack's speed") | — |
| 19 | **Shield rush** (new) | sword_and_board (7) | active: direction | a bash (the shield-bash blow) with `knockback` 1: the foe is driven a cell back and Staggered | 4 / melee 125 % |
| 20 | **Spell reflection** (new) | sword_and_board (10, capstone) | triggered: `spell_blocked` | `reflect_spell`: a spell attack the shield turns goes back at its caster, once a turn (§5) | — |
| 21 | **Pinning shot** | marksman (4) | triggered: `critical` (a critical shot) | `status` *Pinned* 1d3 turns on the target (a hold-like status: blocks `step`, eases a degree a turn, `stacking = "highest"`; no struggle check: it is a missile through the foot, not a grip) | — |
| 22 | **Gap shot** (new) | marksman (7) | active: cell | `ignore_protection` on a shot: the arrow finds the gap between plates, so Protection does not soak it (the missile twin of Sunder; slow, as finding the gap is) | 4 / shoot 150 % |
| 23 | **Stunning fist** | brawler (4) | triggered: `critical` | `status` staggered on the target | — |
| 24 | **Hip throw** (new) | brawler (7) | active: direction | an unarmed blow with `knockback` 1: on a hit the foe is thrown a cell back and Staggered (empty hands: the brawler's grip) | 4 / melee 125 % |
| 25 | **Hit and run** (new) | skirmisher (4) | triggered: `hit` (a throw or a blow of the skirmisher's) | `status` *Light-footed* on the actor: its next action costs half (`cost = 50`, one action, `stacking = "refresh"`), the step away after the strike | — |
| 26 | **Trip** (new) | staff_fighter (4) | triggered: `critical` | `status` Pinned, 1 turn, on the target: the staff takes its legs | — |
| 27 | **Staff sweep** (new) | staff_fighter (7) | active: nothing | `sweep` −2 with the staff at every hostile beside the actor | 5 / melee 150 % |

Every style now has at least one ability of its own and most have two; the weapon families add theirs on top (a skirmisher with javelins has Point-blank, a staff fighter Brace and Lunge). Shield bash, the one weapon-skill ability that exists, stays as it is.

**Interactions and abuse cases, decided here**

| Case | Rule |
|---|---|
| Riposte on a riposte, cleave off a riposte, a gatekeeper's blow that kills cleaving | the free-blow rule stands: **a free blow brings none of its own** but a cleave after a cleave. Nothing in this catalogue adds a cue; Brace, Point-blank, Riposte, Cleaving sweep are the existing switches |
| Rend twice in one attack | once per attack: `both_hands_hit` is raised once, in `after_attack`, and only when both blows were `Strike::Hit` on the same living target; a Sword and Board bash is not a second hand (the trigger is Twin Blades' only) |
| Staggered stacking (Crushing blow, Stunning fist, Stunning bash, Staggering block on one foe) | `staggered` is `max = 1`, `stacking = "refresh"`: a staggered foe cannot be staggered further, only kept so while blows land, which costs the blows |
| Staggering block against a foe that attacks twice a turn | each blocked blow refreshes the status once; the foe's *next* action is dearer, so two blocked blows in a turn cost it one slowed action, not two |
| Pinned forever by repeated criticals | `stacking = "highest"`: a new pin keeps the longer of the two, never adds; at most 3 turns from any one shot |
| Deep cut on a bleeding foe | bleeding `stacking = "add"` to `max = 5`, as any wound: a bleeding foe can be bled more, which is what blades do |
| Vital strike then Mighty swing then Feint in three turns | each is its own action with its own stamina and energy: 11 points of a pool of 12 to 20 and three and a half blows' time; nothing stops a sequence, the empty pool after it is the price, and it is the point of a rank-9 fighter |
| Looping the strongest active | the pool regenerates a point every two base turns: a 5-point active once every ten turns at best, between plain blows; no cooldown is needed (one may be added per ability in data, §4.2, if play shows otherwise) |
| Hit and run into an active | Light-footed halves the *next* action's energy, an active's too (a Whirl at 150 % costs 75 %): one action, refreshed, never stacked; the stamina is paid in full |
| Spell reflection ping-pong, or two reflectors | a reflected spell carries a mark and is never reflected again; at most one reflection a turn of the reflector's; area spells and checks are never reflected (§5) |
| Measure and Spell reflection on one character | they cannot meet: Measure is the duelist's grip (one weapon, the off hand empty), Spell reflection Sword and Board's (a shield held) |
| Feint plus Vital strike in one blow | Feint is a blow of its own, not a status: no stacking of actives on one blow. (A later "the next blow" status form is possible; not written) |
| Sunder or Gap shot on a monster with Pv 0 | allowed, pointless, costs the stamina and the time: the menu does not know the target's Protection (it must not: hidden information) |
| Lunge over a trap, a door, water | the cell between must be empty of actors and passable for the actor; a lunge is not a step (nothing is sprung); a closed door blocks |
| Point-blank with an unloaded crossbow | needs ammunition ready; a reload is not free: the trigger fires only when a shot could be loosed now, else nothing (no draw) |
| Piercing shot through an ally or a shopkeeper | it hits whoever is next, as a missed shot that flies on does; provoking as any shot |
| Shield wall while Raging, Hasted | statuses add: Walled's cost 125 and Hasted's 75 multiply as the data says they do |
| Creatures with triggered abilities | equal treatment (§17 q5): by the same rule and the same data, Spell reflection included (a shield-bearing champion turns the player's bolt): a rank-5 orc spearman (creature types have ranks) braces; a rank-4 duelist bandit ripostes. The rosters set creature ranks, so this is a content review item (§15 slice W2 lists which shipped types cross a threshold); creatures use no actives (no stamina, §4.1); `Trigger` for a creature draws exactly as the blow it makes, so fights without a qualifying creature keep their draws |
| Determinism | deciding never draws: `weapon_abilities_for` is a lookup, the free-blow switches are bits, Staggered and Pinned are afflicted with their severity (Pinned's 1d3 is one draw, as a hold's `turns` is), Rend's extra dice are rolled in the blow's draw order after the riders |

## 7. Spell scaling from the mental stats

### 7.1 Today

A spell's dice grow with caster level (`per_levels`, `max_dice`) under `ceiling[tier] × resolution.damage %`; Willpower widens reach (`reach_step` 2), Presence lengthens buffs and hexes (`presence_step` 2), Intelligence eases study; attack spells aim with the Dexterity modifier. The magic design's rule: "stats never lift damage over its cap". A level-60 wizard with tier-3 spells does 3d6 (10.5) to monsters of 221 HP; the first list stops at tier 3.

### 7.2 The proposal: an authored `[spell.scales]` table, and a lifted cap

**In plain words: what the lift maxima are** (the owner asked, §17 q6). Every spell belongs to a tier (0 to 9), and every tier has a **damage ceiling**: the most one cast of a spell of that tier may deal, whoever casts it (`spells.toml`'s `ceiling`, e.g. 18 for a tier-2 bolt, 19 for a tier-3 burst after the area's discount). Today a spell's dice grow with the caster's level until they reach that ceiling, and then stop for good: a level-60 wizard's fireball is a level-12 wizard's fireball. The proposal lets the ceiling itself rise **for a caster with the stats the spell names**: each point of the named stats' modifiers raises it by a few percent (the spell's weights, e.g. 10 % per point of Intelligence modifier), and the dice follow it up. The **lift maximum** is how far that rise may go, per tier, as a percent of the tier's own ceiling: 40 % at tier 0 (a cantrip can grow by at most two fifths), 80 % from tier 4 on (a great spell can almost double). So a tier-3 fireball (ceiling 19) in the hands of a sharp-witted, strong-willed caster can reach at most 19 × 1.7 = 32, never more, whatever the stats; the ceilings stay the one authority, and higher-tier spells remain the way up. Caster level still grows the dice up to the (lifted) ceiling, as today; stats decide how high that ceiling sits; the maximum caps the stats' share.

```toml
[spell.scales]                          # all optional; absent dimensions keep today's rule
damage   = { intelligence = 10, willpower = 5 }   # percent of the cap per modifier point
to_hit   = { intelligence = 100 }                  # percent of the modifier added to the attack roll
reach    = { willpower = 50 }                      # a cell per 2 points (today's reach_step)
duration = { presence = 50, willpower = 25 }       # a degree per 2 points, plus one per 4 of Willpower
```

Weights are **percent of the stat's modifier** (`Stats::modifier`, the D&D floor((score − 10)/2)), summed over the stats named, rounded to the nearest, never below 0. The defaults when a table or a dimension is absent reproduce today exactly: `reach = { willpower = 50 }`, `duration = { presence = 50 }`, `to_hit = { dexterity = 100 }`, `damage = {}` (no lift). `reach_step` and `presence_step` in `spells.toml` are retired in favour of these defaults (a data change, one constant each).

**Damage: the cap is lifted, never passed.** `lift = Σ weight × modifier` percent, at most the tier's `max_lift` (a new `[tiers]` array, tier 0 first, first guess `[40, 50, 60, 70, 80, 80, 80, 80, 80, 80]`). The spell's cap for this caster is `cap_of(spell) × (100 + lift) / 100`, rounded down; the dice at a caster level are `SpellDice::at` as today, and then **as many more dice as the lifted cap holds** (`max_dice` becomes `floor(lifted cap / sides)` when that is larger; the file's `max_dice` is the unlifted ceiling's). The load-time check keeps refusing a spell written over its tier's unlifted cap, so the data cannot write a spell over its cap; the lift is the caster's, bounded by `max_lift`. Criticals, Empower, and Archmage are held to the lifted cap as they are held to the cap today (`spell::capped`).

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

The table says two things: the lift keeps a known spell useful two tiers longer (fireball at level 30 kills in 6 casts rather than 11), and **a level-60 caster needs the tier 4 to 9 spells to exist**, which is content the magic design planned and no scaling replaces (§14; §17 q6). Mana: a tier-7 burst of radius 2 costs 37 × 2.4 = 89 of a 502 pool, five casts; the tier-9 one 134, three; "area is expensive" holds.

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

## 9. Stat training: plentiful marks, growth between levels; hit points past level 20 (class data)

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

Why a fixed daily hour rather than "checked when the marks reach a threshold": a threshold check fires in the middle of a fight (a rise mid-fight is a message in the wrong place and a hit-point change the status loop must then handle), and it makes a rise a function of play volume alone, so a day of grinding rats is a day of rises; the daily hour bounds the rate in game time, which is the game's one currency. The owner kept the clock (§17 q7: "clock feels like a good place as any").

### 9.3 What the player sees

As today: "You feel stronger." (quicker, tougher, sharper, more resolute, more confident) at dawn or at a level, the character screen's new score, no marks, no chance, no potential. A rise at the daily hour interrupts a rest as a message does (so the player notices; a rest to dawn ends then anyway). The wiki's Stats page states the daily rule and its arithmetic, and `stat-training.md` regenerates with the new uses and chances.

### 9.4 Hit points past level 20: data in the class tables

The owner's answer (§17 q9): no new mechanic; more hit points in the data where the game needs them. Two existing knobs do it, both data:

- **`hp_level` 20 → 50** (`assets/data/experience.toml`): every level up to 50, not 20, adds `max(1, class hp ÷ 2 + the Toughness modifier)` as levels 2 to 20 do today. The formula is unchanged; only the last level that pays moves. 50 rather than 60 keeps the comment's "no runaway pools" true and leaves the last ten levels to the capstone talents, and keeps **Hardened** (the barbarian's level-25 class talent, "every level past `hp_level` adds a level's hit points again, and a little more") meaningful for levels 51 to 60; its row in the barbarian's level table is reviewed in the slice (data: its `bonus` 1 → 3, or the row moved to level 45), since from 26 to 50 it now pays nothing extra.
- **The thief's class `hp` 3 → 4** (`assets/data/classes/08-thief.toml`): the report's thief gains 1 a level (3 ÷ 2 = 1), as a wizard does; at 4 they gain 2, a priest's pace, and start a point higher. The wizard stays at 0 (a point a level, the floor): the caster's answer is §7 and range, not hit points. Warrior (7) and priest (4) keep their numbers: `hp_level` alone gives them the growth.

**Before → after** (the balance report's bare human male characters, maximum hit points, no magic items; "before" is the report's measurement, "after" adds the levels that now pay at the report's per-level gains: warrior about 3.3, priest 2, wizard 1, thief 1 → 2; **first guess, kept in data**):

| Level | Warrior | Thief | Wizard | Priest |
|---|---|---|---|---|
| 20 | 85 → 85 | 36 → 56 | 32 → 32 | 58 → 58 |
| 30 | 88 → 121 | 38 → 78 | 32 → 42 | 58 → 78 |
| 40 | 89 → 155 | 39 → 99 | 32 → 52 | 59 → 99 |
| 50 | 89 → 188 | 40 → 120 | 32 → 62 | 60 → 120 |
| 60 | 88 → 187 | 40 → 120 | 32 → 62 | 60 → 120 |

Against the report's level-60 monster (32 a blow, three hits in four against plain armour) the plain warrior goes from "dead in 4 to 5 rounds" to about 8 to 10, the report's own aim for option C ("about 8"), while §3 and §9 bring their kills to 11 to 22 rounds: still a hard fight, now one a rank-10 fighter with stamina and a potion can win. The other eleven classes follow the same `hp_level`; the slice lists their before and after in the wiki's class table (`cargo xtask wiki` regenerates it) and the PR reviews them. Early levels do not move but the thief's (+1 at level 1, +1 a level to 20).

## 10. Potions and elixirs (Consume, colours by hue)

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

### 10.2 Drinking: the Eat command becomes Consume

The owner's answer (§17 q12): no Drink key and no second list; Eat lists drinks too, and is labelled **Consume**. So:

- **The command**: `KeyCommand::Eat` keeps its default key and its place in `settings/keys.rs` (and its id `eat` in `settings.toml` and exported keymaps, so no player's binding breaks); its description becomes "Consume" (`input.rs`), and the help (`?`), Controls, and every hint that names it say "consume". The `q` Drink key of the first draft is dropped: nothing new is bound.
- **The list**: the Eat screen (`ScreenName::Eat`, `ObjectList::Eat`) is titled "Consume what?" and lists what can be eaten **and** what can be drunk, food and potions together, from the floor and the pack as today, grouped by item class as every list is (Food, then Potions). Its empty line becomes "You have nothing to consume." Floor potions are listed and drunk as floor food is eaten (one rule for both; no "pick it up first").
- **The action**: the simplest is to keep one: `Action::Eat` and its text `eat:b` / `eat:floor:a` stay, and `World::eat` hands a stack with a `drink` table to `World::drink` (`world/drink.rs`) instead of starting a meal. No new `Action` variant, no new action text, no new replay syntax; `drink:` and `consume:` are not added (a replay reads `eat:c` for a potion, which the dump's action text documents). The price is a new `drink` in `data/actions.toml` (1000, a turn; a meal's turns stay `eat`'s).
- **`World::drink`**: refused with no time when the stack has neither `food` nor `drink` ("You cannot consume a long sword."); otherwise priced, one taken from the stack (positions shift as `use_up` handles), the type learned (`World::learn_scroll(object)`: "You drink the fizzy red potion. It is a potion of healing."), the effect applied through §10.1's table, the marks given (`train_stat(StatUse::Drank(stat))` per mark), then the kind's own message ("You feel your wounds close.", "You feel hale.", "The dark is less dark to you."). Drinking while Raging is allowed (a barbarian can drink); a status that `blocks = ["drink"]` may be written later. Creatures do not drink (no AI for it; a later wave).

**Unidentified potions: a drawn word and a colour by hue** (the owner's answer, §17 q13). An unknown potion is called "a {word} {colour} potion" ("a fizzy red potion", "a cloudy teal potion"):

- The **colour** is the potion type's own, fixed in data (`colour = "red"` on `[object.drink]`, a name from the colour table below with its `fg`), chosen by **one rule: each effect family has a hue, and a potion with several effects mixes their hues**. Colours may be close, and potions of one family may share one exactly (the six boosts are all orange); the colour hints at the family, never at the type. The glyph `!` shows in that colour.
- The **word** is drawn per type at the start of a run from `data/potion_labels.toml`, in the format of `scroll_labels.toml` and `wand_labels.toml` (`name = "{label} {colour} potion"`, `syllables = [1, 1]`, `parts` the words: bubbling, cloudy, fizzy, foaming, glittering, milky, misty, oily, pearly, shimmering, smoky, sparkling, swirling, syrupy, thick, thin, viscous, ...; at least as many as there are potion types, so no two types share a word), by `Labels::draw` after the rods' from the same `identification` stream (so scrolls, wands, and rods keep their labels and runs their rolls). The one change to the label machinery: a potion's `fg` comes from its type's colour, not the file's single `fg`.

| Family (effect) | Hue | Colour name, `fg` (first guesses, kept in data) |
|---|---|---|
| healing | red | red `#c83c3c` |
| cures (poison, disease) | green | green `#3ca050`; pale green `#78c080` (disease) |
| healing and cures (greater, full healing) | red mixed with green | russet `#a8503a`; deep russet `#985038` |
| mana | blue | blue `#3c6ec8` |
| stat boosts | orange | orange `#d89a30`; deep orange `#c87820` (greater) |
| elixirs (a stat for good) | gold | gold `#e8c860` |
| vision | purple | violet `#8a50c0` (all three) |
| speed | yellow | yellow `#e0d040` |
| resistance | teal | teal `#30a8a0` (all four) |
| harm | brown | brown `#5a4030` |

A later potion with two families' effects mixes them by the rule (a heal-and-mana draught purple-red, "plum"); the author names the colour and picks its `fg` between the two hues.

`WorldState::known_scrolls` holds known potion types as it holds wands' and rods'; the scroll of identify identifies a potion stack's type (`World::identify` already learns a labelled type). Prices are true (a shop's price hints, as ADOM's). The witch's kit potions are known (a chosen character knows their kit, as scrolls).

### 10.3 Marks by nature

Every potion names `marks = { <stat> = N }` (default by kind as the table says: a healing potion marks Toughness, a vision potion Willpower; N 5 for a common potion, 10 for a greater, 20 for an elixir), given as **sure marks** through the `drank` use (`chance = 100` in `stats.toml`, the potion's N as the count: `train_stat` gains a count argument, 1 for every use today). So a potion of might is 5 Strength marks and ten minutes of +2: the body remembers the effort. A harm potion gives none.

### 10.4 Where they come from

Loot by `level` and `frequency` as scrolls (`magic::loot::roll`), shops by the loot frequency (`mapgen/shop.rs`): the **alchemist** kind (exists in `shops.toml`, waiting) stocks potions first and buys them; general stores and provisioners a few healing and antidote potions among their wares; the temple's services unchanged. Creature packs: a hedge witch, a cultist, an apothecary carry one (gear lines in their files, the rosters' review). Quest rewards and the forgotten altar's "a healing draught" outcome may name one.

### 10.5 Elixirs

Rare (level 8, frequency 2: about one in a long game's loot, more in deep places and as rewards; never in a general store, at a very rare alchemist's stock at 2000 gold). Drinking one: `Stats::train(stat)` once (the score +1, and the potential lifted to the score if it was below), then the potential raised by a roll of `potential` ("1d2", one gameplay draw), "You feel stronger, and that you could be stronger yet." (the second clause only when the potential rose by 2). Hit points follow a Toughness rise through `raised_stats`, mana an Intelligence or Willpower one; carrying limits a Strength one; all as a trained rise does.

**+1d2 potential** (the owner's answer, §17 q10: "1d2"): with +1 the elixir moves the score and the potential together, so the gap the training rule works in is unchanged and an elixir past the potential is one point and nothing more; with +1d2 (average 1.5) half of elixirs widen the room for training by one, so an elixir has a lasting value beyond its point and a stat the player lives by can be pushed by finding elixirs *and* training, which is the ADOM rhythm the owner's design leans on (potions of potential there are the rare good find). It costs one draw and one line of data.

Elixirs are not drunk for marks alone: 20 marks of the stat are given too, because the owner asked that every potion marks.

### 10.6 What the player sees

- The Consume command (Eat's key, renamed) opens the Consume list: food and potions, the floor's and the pack's, an unknown potion by its word and colour ("a fizzy red potion", the `!` in red), a known one by name, with their counts; Esc closes it; a drink takes a turn and says its message; an unknown potion's name is learned as a scroll's is.
- The status bar and the character screen show a boost or a vision status by name (Fortified, Owl-eyed) as they show Blessed; no numbers or timers; the character screen's score is raised and marked as magic raises it today.
- The item description (`describe::appraised`) says what a known potion does in words ("Heals wounds at once", "Sharpens the eyes in the dark for a while", "Makes you stronger for good"), never dice or minutes; an unknown potion says "You do not know what this potion does."
- Look mode's pile names use the label. Prices in shops are true.
- Snapshots: `consume-list` (food and potions together), `consume-list-none` ("You have nothing to consume."), the existing eat-list cases renamed with their new title and hints, `potion-description`, `potion-description-unidentified`, the character screen with Fortified (`potion-boost`), the status bar row.

## 11. Talents and class abilities (the coverage bar)

Every new system ships with its `[[system]]` tag and at least `min_talents` (2) talents in the PR that adds it (`docs/dev/progression.md`, coverage policy); every new ability names its systems. **Names and numbers are first guesses**, each on a hook this design adds or one that exists.

| System | Status | Talents (tree) | Hook |
|---|---|---|---|
| `weapon_skills`, `styles`, `combat`, `defence` | exist | the new rank tables are data; Weapon focus, Stylist, Fencing, Weapon mastery, Tempered keep their meanings and read the new arrays | — |
| **`weapon_abilities`** (new) | ships in W2 | **Early mastery** (Weaponcraft, needs level 9): rank abilities come one rank sooner (`had_by` reads rank + 1) · **Second blade's rend** (Finesse, needs Twin Blades 6): Rend rolls the off hand's dice too · **Brace of steel** (Guard, needs spears or staves 5): Brace's blow is at +2 and a braced foe is Staggered · **Practised technique** (Weaponcraft, needs a weapon skill at 7; ships in W3): weapon actives take a quarter less time (`TalentMods::weapon_energy_percent` −25, read in the one place an active is priced, never below 50 % of the action) · class ability **Drillmaster** (warrior, at 25; W3): the warrior's weapon actives cost a point less stamina, at least 1 (`TalentMods::weapon_stamina_discount`) |
| **`stamina`** (new: a resource is a system, the coverage policy) | ships in SP1 | the granted class talent **Trained wind** (`grants_resource`, the fighting classes' level 1, §4.5) · **Deep lungs** (Vigour): the stamina maximum +4 (the existing `resource_max` effect) · **Steady breathing** (Weaponcraft, needs a weapon skill at 5): stamina comes back half again as fast (the existing `resource_rate` +50) · **Second breath** (Might): a kill of yours gives a point of stamina back (the existing `resource_build` on `kill`) | §4.5 |
| `spells` | exists | **Keen mind** (Spellcraft): the damage lift's `max_lift` is 10 points higher for the chosen school (`school_focus` gains a `lift` field) · the existing Empower, Archmage, School focus read the lifted cap | §7 |
| `protection` | exists | **Armour-breaker** (Might, needs axes, maces, or polearms 7): Sunder and Gap shot cost 2 stamina less and a critical of yours ignores 2 Protection | §8 |
| **`stat_training`** (promoted: the daily check is a rule a talent can change, so the coverage policy's "channel, not a system" no longer holds) | ships in T2 | **Hard training** (Vigour): the daily check's `daily_factor` is 2 for you · **Quick study of the body** (Scholar): potions' marks are doubled · the existing Good learner stays XP's | §9 |
| **`potions`** (new) | ships in Q1 | **Thrifty drinker** (Delver): one potion in four is not used up (as Spell recall for scrolls; a draw of the gameplay stream only when the talent is held) · **Strong constitution** (Vigour, needs Toughness 13): boost and vision potions last half again (the afflicted severity-seconds × 1.5) · **Nose for draughts** (Scholar): an unknown potion's kind is told by its smell on pickup ("It smells of herbs: a cure of some kind."): the kind, never the type · class ability **Cauldron-wise** (witch, at 12; the existing Cauldron talent file is reviewed for overlap): potions she drinks heal or boost a fifth more |
| `hit_points`, `mana` | exist | unchanged talents; §9.4's `hp_level` and the thief's `hp` are data; Hardened (barbarian) is reviewed with them; the heal and mana potions are items, not rules | §9.4 |

## 12. Saves, ids, determinism

| What | Saved / id | Note |
|---|---|---|
| Weapon ability ids | `WeaponAbilityId` indices in the content fingerprint; **not** saved (nothing on an actor names one: free blows are bits, stamina is a pool) | adding an ability changes the fingerprint, not the save format |
| Effect statuses (Walled, Pinned, Light-footed, the six boosts, three visions, four resistances; no cooldown statuses) | `StatusId` indices; new entries at their groups' ends in `status.toml` (saves change: the integrator's `FORMAT_VERSION` bump once per release) | a status with `vision` or `resist` is read by `Actor::eyes` / `Actor::resistances` only |
| `FreeBlows` from styles | not saved separately: `free_blows_of` folds the style's switch in at read time, as the talents' | an actor's saved bits stay what talents set |
| Stamina | the pool lives in `Actor::resources` (exists, saved and hashed); `stamina` is a new `ResourceId`, appended at the end of `resources.toml` (file order is save order); `max_from` and the maximum are derived; the granted talent Trained wind is a `TalentId` (one file) | old saves load without the pool; `World::grant_class` on load grants it to a fighting class past level 1, as it grants any pool a talent gives |
| `next_stat_check` | `WorldState`, a `GameTime`, left out while no training rules | one new field |
| Stat marks' counts | `Actor::stat_marks` as today (u16 saturating; a potion's 20 marks fit) | — |
| Potion labels | derived (`World::labels`, never saved); known types in `WorldState::known_scrolls` (exists) | a new object file changes the fingerprint |
| Potions in packs | plain stacks; a `drink` table and its `colour` are data on the type | nothing per instance |
| Consume | no new `Action` and no new action text: `eat:b` drinks a potion; `KeyCommand::Eat` keeps its id `eat` (settings and exported keymaps unchanged); one new price, `drink`, in `actions.toml` | replays made before keep verifying (an `eat:` of food is unchanged) |
| Hit points (`hp_level`, the thief's `hp`) | data; `hp_base` is saved, so a saved character keeps what their past levels gave and gains at the new rule from their next level | replays past level 20, and any thief's, diverge in hit points (as any content change) |
| Spell scales | derived at cast time; nothing saved | old replays that cast verify only if the caster's modifiers give lift 0 (a run at creation stats with Intelligence ≤ 11): the regenerated costs are unchanged, the damage is not |
| Monster Protection | data per type; nothing saved | replays through fights with changed types diverge (as any content change) |
| Determinism | deciding an ability never draws; Rend's dice, Pinned's turns, a reflected spell's own attack roll and dice (drawn as the caster's cast would, after the blocked roll), a harm potion's sickness, Thrifty drinker's keep, an elixir's potential are gameplay draws only when they arise; the daily check draws the `training` stream six times at its hour (always six, so the stream's position is a function of days passed); labels from `identification` after the rods' | fights without a qualifying rank keep their draws and hashes |

## 13. Tests, text snapshots, and the wiki

- **Core tests** per slice: `content/weapon_skill.rs` (the family rank-10 table and monotonic arrays; a style's `crit` to ×4), `content/weapon_ability.rs` (parse and refusals: unknown skill, a style's ability on a weapon skill, a trigger on an active, an effect with another kind's field, a triggered ability with a stamina or energy cost, an active without `energy`, a cooldown status with a tick), `world/weapon_ability_tests.rs` (each catalogue entry on a hand-made fight: Rend once per attack and not on a bash, Staggering block refreshes, Pinned keeps the longer, Deep cut adds to bleeding to its max, Riposte through the style and the talent together once, a creature with the rank braces, Lunge over an empty cell and refused over a closed door, Piercing shot on to the next, Sunder against Pv 8, Shield wall's cost and Shield, Whirl and Staff sweep round the actor, Shield rush and Hip throw knock back and refuse into a wall, Gap shot unsoaked, Hit and run halves the next action once, Trip pins a turn, Measure parries a shot, Spell reflection turns a blocked bolt once a turn and never a reflected one and never a burst, a creature with the rank reflects; every active spends its stamina after its refusals and is priced at its energy percent, a short pool greys the row; no draws while no rank qualifies; a save and a replay per active), `world/stamina_tests.rs` (the maximum from Toughness and level, the grant at level 1 by the class table and on the first active's rank for anyone else, regeneration and rest, Deep lungs, Steady breathing, Second breath, an old save of a fighter granted the pool on load), `world/cast_tests.rs` (the lift by stat, `max_lift` holds, dice follow the lifted cap, criticals held to it, a cantrip with no scales unchanged, `to_hit` by Intelligence, the defaults equal today's reach and duration), `content/spell_def.rs` (a spell over its unlifted cap still refused; `max_lift` ascending or flat), the rosters' tests (`undead_roster_tests` and the tribes': Protection by body and level within the rule or in the exceptions list), `world/stat_training_tests.rs` (the new uses at their hooks, the daily check at 06:00 and only then, a week's rest makes seven checks in order, batching invariance, `daily_factor`, the potential's divisor, the message interrupts a rest, saves), `world/drink_tests.rs` (every kind through `eat:`, a floor potion, food still eaten as before, the labels' words and the types' colours and `known_scrolls`, identification by the scroll, marks given by count, Thrifty drinker's draw only when held, an elixir's score and potential and the hit points that follow, a harm potion's sickness, refusals, the witch's kit known, a replay), the experience content test (`hp_level` 50; the hit points a level-60 character of each class has, pinned).
- **Text snapshots** (`crates/warlock/src/ui/screen_snapshots.rs`; no goldens: nothing here is GPU-only): the abilities menu with the weapon skills group populated (`abilities-menu-weapon-skills`), a row greyed short of stamina, the stamina cost on a row, `consume-list`, `consume-list-none`, the renamed eat cases, the help and Controls rows saying Consume, `potion-description`, `potion-description-unidentified`, the character screen with a boost (`potion-boost`) and with stamina, the status bar with `Sta` and a boost.
- **Wiki** (`cargo xtask wiki`): `weapon-skill-benefits.md` and `style-benefits.md` regenerate; new `weapon-abilities.md` (`xtask/src/wiki/weapon_abilities.rs`: ability, skill, rank, kind, trigger or aim, what it does, stamina, energy in percent) on the Weapon skills and Fighting styles pages; `spell-list.md` gains "Grows with" and the Magic page the lift table; `stat-training.md` regenerates and the Stats page states the daily rule; the resources table gains Stamina and the Combat page the stamina rule; the classes' hit points table regenerates with `hp_level` 50; a new page **Potions** (`wiki/src/potions.md`, under Objects after Scrolls in `SUMMARY.md`) with `potions.md` (potion, kind, colour, effect in numbers, duration, marks, level, frequency, value), the colour-by-hue rule, and the elixir rule; the Hunger page's Eat becomes Consume; the Creatures page's bodies paragraph.

## 14. What is deliberately not in this design

- **The magic-armour gap** (the report's option D): out of scope; the owner chose not to go there (§17 q11).
- **A new hit-point mechanic** (the report's option C as a formula): the owner wants none; §9.4 changes data only.
- **The tier 4 to 9 spell list**: content the magic design planned (`docs/design/magic.md` §7 stops at tier 3); §7 shows why a level-60 caster needs it. An issue of its own.
- **Monster stat growth** (option B): the rosters' Strength formulas are a content design; §8 gives monsters Protection instead of taking their to-hit.
- **Creatures using actives** (no stamina; a later AI slice under a cooldown status, §4.1), **creatures drinking potions**, **potions thrown as weapons**, **a potion of restoration for drains** (`docs/dev/stats.md` names it as not yet; it fits the `cure` kind with a `restores` field later), **brewing** (the witch's Cauldron), **a Drink key or list** (Consume covers it, §10.2).

## 15. Implementation plan

Each slice is one issue, one branch, one PR, with the spec section, its `docs/dev/` topic (`weapon-skills.md`, a new `docs/dev/weapon-abilities.md` created in W2 and listed in CLAUDE.md there, `spells.md`, `creatures.md`, `stats.md`, `resources.md` (SP1), `experience.md` (H1), `screens.md` (C1), a new `docs/dev/potions.md` created in Q1), and the wiki in the same PR. Sizes: S a day, M two to three, L a week. "Needs" is the slice it builds on; "Ids / state / action" says what it adds that the overlap rule counts (at most about three PRs at once that add actions or save state).

| # | Slice | Sections | Needs | Ids / state / action | Size |
|---|---|---|---|---|---|
| **W1** | Weapon skill and style rank tables: the new arrays (each group's own, §17 q1), the family content test, `crit` to ×4 for styles, wiki tables regenerated | §3, §5 | — | none (data) | S |
| **H1** | Hit points past level 20: `hp_level` 20 → 50, the thief's `hp` 3 → 4, Hardened's row reviewed, the experience content test, the wiki's class table | §9.4 | — | none (data) | S |
| **C1** | Consume: Eat's description, the list's title, its empty line, the help, Controls, and hints renamed to Consume (the key, its id `eat`, and `eat:` unchanged); the eat-list snapshots updated | §10.2 | — | none (UI text) | S |
| **SP1** | The stamina pool: the `stamina` entry in `resources.toml`, `max_from` in `ResourceDef` and `Actor::resource_max`, the granted talent Trained wind in the fighting classes' level-1 rows, the grant on a rank that unlocks an active (the third grant place), the status bar's `Sta`, the `stamina` system with Deep lungs, Steady breathing, Second breath, `docs/dev/resources.md` | §4.5, §11 | — | **state** (a new `ResourceId`; the pool in saves) | S–M |
| **W2** | Weapon abilities, the system: `data/weapon_abilities/`, `WeaponAbilityId`, the loader and checks, `Trigger` and `World::weapon_trigger` at the ten combat lines, `WeaponDoes` kinds `free_blow`, `status`, `extra_damage`, `parry_missiles` (the triggered abilities but Spell reflection: Deep cut, Crushing blow, Brace, Point-blank, Stunning bash, Riposte, Measure, Rend, Cleaving sweep, Staggering block, Pinning shot, Stunning fist, Hit and run, Trip), Pinned and Light-footed in `status.toml`, the menu's greyed "fires by itself" rows, creatures' triggers and the creature-rank review, `docs/dev/weapon-abilities.md`, the `weapon_abilities` system with Early mastery, Second blade's rend, and Brace of steel | §4.1–4.4, §6, §11 | W1 | ids (fingerprint); 2 statuses | M |
| **W3** | Weapon abilities, the actives: `Ability::Weapon`, `use:ID` resolution, `Unavailable::NotSoHeld`, stamina spent and the `energy` price, the kinds `reach_blow`, `sure_critical`, `ignore_defence`, `ignore_protection`, `pierce_shot`, `buff`, `sweep`, `knockback` (Vital strike, Sunder, Lunge, Piercing shot, Shield wall, Feint, Mighty swing, Whirl, Shield rush, Gap shot, Hip throw, Staff sweep), Walled, Practised technique and Drillmaster | §4.5, §4.6, §6 | W2, SP1 | **action** (an arm of `use:`); 1 status | M |
| **W4** | Spell reflection: the `spell_blocked` trigger in `cast.rs`'s attribution, `reflect_spell` with its once-a-turn cap and the reflected mark, creatures reflecting | §4.3, §4.4, §5 | W2, S1 | none (a transient mark) | S–M |
| **S1** | Spell scales: `[spell.scales]`, the defaults, `max_lift`, the lifted cap in `SpellDice`/`cap_of`, `to_hit` by stat, `reach_step`/`presence_step` retired, the first list authored (evocation damage by Intelligence and Willpower, hexes' duration by Presence and Willpower, heals by Willpower), Keen mind, the wiki column and the Magic page's plain-words paragraph | §7, §11 | — | none (derived) | S–M |
| **P1** | Monster Protection: the body rule as a content test with the exceptions list, the data pass over every creature file (shared and campaigns), the Creatures page paragraph, Armour-breaker | §8, §11 | W1 (and T1 landed, by the plan's waves) | none (data) | M (many files, one line each) |
| **T1** | Stat training uses and chances: the six uses at their hooks, `train_stat` with a count, the chances, wiki | §9.1 | — | none | S |
| **T2** | The daily check at 06:00: `daily_factor`, `next_stat_check`, the `live` moment, the message, Hard training, the `stat_training` system | §9.2, §11 | T1 | **state** (one field) | S–M |
| **Q1** | Potions: `[object.drink]` with `heal`, `cure`, `mana`, `harm`, `marks`, `colour`; `World::drink` reached from `eat:` (the Consume list showing potions), the `drink` price, `potion_labels.toml` and the colour per type, identification, the first six potions and the murky draught, the witch's kit renamed, shops and loot, `docs/dev/potions.md`, the `potions` system with Thrifty drinker and Nose for draughts, the Potions wiki page | §10.1–10.4, §11 | T1 (the `drank` use), C1 | none counted (no new action; labels derived) | M |
| **Q2** | Potions: `boost`, `vision` (`StatusDef::vision` in `Actor::eyes`), `haste`, `resistance` (`StatusDef::resist` in `Actor::resistances`); the thirteen boost, vision, and resistance statuses; Strong constitution, Cauldron-wise | §10.1 | Q1 | 13 statuses | M |
| **Q3** | Elixirs: the `elixir` kind (+1 and potential +1d2), the six elixirs, their loot and rewards, the altar outcome's reference, Quick study of the body | §10.5, §11 | Q1, T2 | none | S |
| **D1** | The cross-checking pass: the balance probe re-run as a documented `#[ignore]` test (`world/balance_probe.rs`, the report's method, kept this time) with its tables in `docs/perf.md`'s neighbour `docs/balance.md`; the spec's §8 sections and CLAUDE.md's topic index for the two new topics | all | everything above | none | S |

**Waves** (never more than three at once that add an action or save state):

1. **W1, T1, S1, H1, C1** (data, derived rules, and UI text; none counted; no overlap: `weapon_skills.toml`, `stats.toml` and five hook lines, `spells/` and `spell_def.rs`, `experience.toml` and one class file, the eat screen's strings).
2. **SP1** (state), **W2** (ids, statuses), **T2** (state): the three counted ones; **P1** with them (data only, many creature files, nothing else touches them).
3. **W3** (action, a status), **Q1** (not counted), **W4** (not counted): `status.toml` is a hot file, each adds its own group.
4. **Q2** (statuses), **Q3**.
5. **D1**, last.

## 16. Open questions, each with a recommended default (answered: §17)

Kept as the owner reviewed them; every one is answered in §17, and the body above applies the answers (where an answer differs from the default here, the body and §17 win).

| # | Question | Recommended default | Alternative |
|---|---|---|---|
| q1 | **Rank thresholds**: weapon skills at 5 and 9, styles at 4 and 7? | yes: a weapon skill's 9 is 7 665 marks in, a style's 7 half that; both come in the middle game for a fighter who sticks to one weapon | the owner's "4/7/10" for both, with a third ability at 10 (q3) |
| q2 | **Cost model for actives**: cooldown statuses? | yes (§4.5) | a shared Stamina pool in `resources.toml` (max 10, a point back every two turns, costs 2 to 5), which W3 would add instead of the cooldown statuses, plus its two talents |
| q3 | **A third ability at rank 10**? | no: rank 10 is the numbers' capstone (×4 criticals, +12 damage, penalty-free twin blades); 18 abilities already | one per style at 10 (a Duelist's "Measure": Parry counts against missiles; Great Weapon's great cleave is already its 10) |
| q4 | **One ability for Twin Blades, Sword and Board, Marksman, Brawler**, none of their own for Skirmisher and Staff Fighter? | yes for the first draft: the weapon families cover the last two (Point-blank, Brace, Lunge) | a second each at 7: Twin Blades "Whirl" (a sweep with both hands), Sword and Board "Shield rush" (a bash that knocks back), Marksman "Gap shot" (a shot Protection does not soak, slower than a shot), Brawler "Throw" (knockback), Skirmisher "Hit and run" (a throw that hits makes the next step cost half), Staff Fighter "Trip" (a critical pins) |
| q5 | **Creatures use triggered abilities** by their ranks? | yes, at once in W2 (the hook reads the actor, not the player; the rosters' ranks are reviewed in the PR: today few shipped types reach 4 or 5) | player only, with a `player_only` flag on the ability |
| q6 | **Spell lift maxima** `[40, 50, 60, 70, 80, 80, …]` and weights of 10 % Intelligence, 5 % Willpower per point? | yes: a +8 Intelligence caster lifts a tier-3 spell 70 %, so a found spell stays useful two tiers on, and the tier 4–9 list (an issue of its own) remains the way up | lower maxima (50 % flat) and a dice-adding rule instead of a cap lift |
| q7 | **The daily check** at 06:00 with `daily_factor` 3? | yes (§9.2) | a check whenever a stat's marks reach `need` ("ripe"), fired at the end of the player's turn; or a check at every rest's end |
| q8 | **Protection bodies** and the four lines? | yes, with the exceptions list (bosses held to the bony line) | three bodies (merge bony into hide at L ÷ 7), or Protection only from level 20 |
| q9 | **Hit points past level 20** (the report's option C)? | not in this design; the owner's call. If wanted: `hp_level` 40 with half the class hit points per level past 20, in T2's neighbourhood (one constant and the formula's one line) | leave it: the design's fights at level 60 are won in 11 to 22 rounds and lost in 5 |
| q10 | **Elixir potential +1d2** or +1? | +1d2 (§10.5) | +1 |
| q11 | **The magic-armour gap** (the report's option D)? | not in this design | — (out of scope: §17) |
| q12 | **How potions are drunk**? | (the first draft proposed a key and a list of their own; dropped) | Eat's list taking potions too: **chosen**, as Consume (§10.2) |
| q13 | **Potion labels by colour** with the glyph's colour the label? | yes: it reads at a glance and costs no name | the scroll scheme's nonsense words ("a potion labelled ZUNBAR") |

## 17. Owner decisions (2026-10-27)

The owner's answers, verbatim (2026-10-27): "q1: implementer can decide. they do not need to be the same for every weapon group / q2: stamina pool sounds fine. also, make sure there is 'energy cost' appropriate for activatable actions so they are not instant speed. triggers can be instant and take no energy / q3: if you feel like you have a cool ability for that style not yet in the tree, go for it. if you want to give spell reflection to maxed out style go for it, / q4: new fucking rule. there is no ability called 'aimed shot' ever in this game. all shots are aimed. figure something else. other than that ok. / q5: sure, equal treatment / q6: sure, i am not sure what this is about but let's try that / q7: sure, clock feels like a good place as any / q8: ok? / q9: just sprinkle more hp into class tables if that is what you think the game needs. no new mechanics. / q10: 1d2 / q11: let's not go there / q12: eat can list drinks too. no need for two keys. Just label the menu 'consume' / q13: assing some color, but don't worry if they are close to eachother. some potions can even have same color if it makes sense to you. Generally have a rule, that certain type of effect have a certain hue, and if there is many effects it might be a mix of those hues."

| # | Answer | What the design does with it | Sections changed |
|---|---|---|---|
| q1 | the implementer decides the rank thresholds; they may differ by weapon group | thresholds are per ability file, so per group and style; 5 / 9 and 4 / 7 / 10 are first guesses only; the one-size claim is gone | §1, §3, §4.1, §4.2, §5, §6, §15 (W1) |
| q2 | a stamina pool; actives cost energy (not instant); triggers instant and free | actives pay stamina and an energy cost (a percent of their action, every catalogue entry priced); triggers cost nothing; the pool designed on the resources system (maximum from Toughness and level, a point every two turns, the fighting classes from level 1, `Sta` on the status bar); cooldown statuses only as an optional per-ability limiter, none used | §1, §4.1, §4.2, §4.5, §4.6, §6, §11, §12, §13, §15 (SP1, W3) |
| q3 | new style abilities welcome; Spell reflection for a maxed style | nine new abilities (Whirl, Shield rush, Gap shot, Hip throw, Hit and run, Trip, Staff sweep, Measure, Spell reflection); capstones at 10: Spell reflection (Sword and Board), Measure (Duelist), great cleave (Great Weapon); a new trigger `spell_blocked` and effects `reflect_spell`, `parry_missiles` | §1, §4.3, §4.4, §5, §6, §13, §15 (W2, W3, W4) |
| q4 | no ability ever named for aiming a shot: all shots are aimed | the marksman's second ability is **Gap shot** (a shot Protection does not soak); no name in the catalogue implies an unaimed shot; the rest of q4's list taken | §6, §16 |
| q5 | equal treatment for creatures | creatures fire the same triggers from the same data (Spell reflection too); with no stamina they use no actives; a later AI slice would give them actives under a cooldown status | §1, §4.1, §6 |
| q6 | try the lift maxima | kept as proposed; a plain-words paragraph explains the ceilings and the lift | §7.2 |
| q7 | the daily check by the clock | kept: every stat at 06:00 game time | §1, §9.2 |
| q8 | bodies for Protection accepted | kept as proposed | — |
| q9 | more hit points in the class tables; no new mechanics | `hp_level` 20 → 50 and the thief's `hp` 3 → 4, data only, with a before/after table for four classes; Hardened reviewed | §1, §2, §3, §9.4, §11, §12, §14, §15 (H1) |
| q10 | elixir potential +1d2 | kept | §10.5 |
| q11 | the magic-armour gap: don't go there | out of scope; the option text is removed | §2, §14, §16 |
| q12 | no Drink key; Eat lists drinks, labelled Consume | Eat becomes Consume (description, list title, hints; key and `eat:` unchanged), the list shows food and potions, `World::drink` reached from `eat:`; no new action, no `q`, no Drink screen | §1, §10.2, §10.6, §12, §13, §14, §15 (C1, Q1) |
| q13 | colours by effect hue; close or equal colours fine; mixed effects mix hues | each potion type has a colour fixed in data by family hue (healing red, mana blue, cures green, boosts orange, elixirs gold, vision purple, speed yellow, resistance teal, harm brown; mixes between), and a drawn word hides the type: "a fizzy red potion" | §10.2, §10.6, §13 |

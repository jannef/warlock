# Warlock — The paladin: Zeal, Smite, and the modes of smiting

**Built; this file is history.** Current behaviour is in spec §8 "The adventurers' class tables, and the class actives" and `docs/dev/progression.md` and `docs/dev/resources.md`.

Design document for the owner's review (2026-11-02; #823). Written against `origin/main` at f3846261. Nothing here is implemented; §12 cuts it into issues. It builds on what is there: the class tables and the class actives (`world/class_ability.rs`, `content/class.rs::ClassActive`, `docs/dev/progression.md`), alternative resources (`docs/dev/resources.md`: Rage, Focus, Openings, Ki, Inspiration, Stamina), alignment and the holy gifts' bands (`docs/dev/alignment.md`: `World::answered`), free blows (`world/free_blow.rs`: `Cue`), statuses that shed light (`status.toml` `light`, read by `light/cache.rs`), the Frightened status that `flees` (Hound's bay's), summoning's record of who summoned whom (`docs/dev/summoning.md`), and the talent coverage policy (`docs/dev/progression.md`, "Coverage policy", rules 1–9 of #776, binding here). Every number is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-11-02, verbatim): *"Paladin: create design issue to bring the paladin class fantasy together. I think we want to lean into smites and alternative modes of smiting."* The gaps report (#802) lists the paladin's empty levels: **25** (Shield of faith, cut by #798) and **32** (Steadfast, cut by #798).

**The fantasy, in a line.** The paladin is the knight who stands in the doorway and takes the blows, and answers each with holy wrath. Zeal is the rhythm of that fight: it rises as blows are given and taken, and the paladin spends it on wrath (a smite, of one kind or another) or on mercy (a hand laid on their own wounds). The unholy fear them; the dark cannot hide from them; what was summoned is unmade by them.

## Contents

1. Summary
2. The owner's request, and what exists today
3. Research, tied to decisions
4. Zeal: the pool
5. Smite: the one blow
6. The modes of smiting
7. Lay on hands, Avatar, and the rest of the table
8. The table reworked: what stays, changes, and goes
9. Talents (the coverage policy)
10. What the player sees; determinism, saves, replays; the wiki and the docs
11. Choices made along the way, and what is rejected
12. Implementation plan (slices)
13. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **Zeal** (new resource, `resources.toml`) | the paladin's pool, granted by the Smite talent at level 1 (`grants_resource`, as the thief's Sneak attack grants Openings). Maximum 10 plus the Presence modifier plus a point per 10 levels (`max_from`), at least 6; granted at 5; **built by the fight**: a point each time a blow of the holder's hits (`hit`) and each time they are hit (`struck`), the data's `builds`; and a slow trickle with time, a point every ten base turns (`regenerates = 60`, resting too), so an empty pool is full again in ten minutes of game time, the span Lay on hands' cooldown is today. Sits at 0, never ends. **One pool, two uses**: every smite and Lay on hands spend it, so wrath and mercy compete, which is the paladin's choice to make. §4 |
| **Smite** (exists; reworked, **moved to level 1**: the signature) | `use:smite:E`: a melee blow in a direction with the player's weapon, **+1d6 per 10 levels (at least 1d6)** as today, now **for 3 zeal instead of a 5-minute cooldown** (the Spent status retired), with the paladin's **Presence modifier added to hit** (D&D 3.5 and Pathfinder: Charisma to the smite's attack), and **one die more against the unholy** (creature types carrying a tag of `data/smite.toml`'s `unholy` list: the undead and the fiends; D&D 5e's extra d8). A holy gift, as today: works for Kind or better; outside the bands "none answers", a cast's time, no zeal spent. §5 |
| **The modes** (new; each its own ability row, a class ability at its level, all on the one smite code in `world/smite.rs`) | **Branding smite** (12, 3 zeal): the foe it hits is **Branded** (a new status: it sheds the item glow's light for five minutes, `light` on the status as Lit's), so it is seen wherever it goes in line of sight, cannot come at the paladin out of the dark, and lights its fellows. **Wrathful smite** (32, 4 zeal): the foe it hits makes a Willpower check against 13 plus the paladin's Presence modifier or is **Frightened** (the existing status that `flees`) for `2d3` turns. **Banishing smite** (50, 6 zeal): a summoned foe it hits is **unmade outright** (its death earned as a kill), and an unholy foe that was not summoned takes the smite's dice **twice**; anyone else takes a plain smite. **Retribution** (25, passive, 4 zeal a blow): a hostile beside the paladin whose attack **hit** them takes a smite at once when the attack is over (a free blow on `Cue::Attacked`, as a riposte is on a parry), while the zeal lasts. **Holy wrath** (40, passive, free): every blow of the paladin's that hits an unholy foe carries one smite die (D&D 5e's Improved Divine Smite). §6 |
| **Lay on hands** (exists; onto Zeal) | 2 hit points a level as today, **for 6 zeal** in place of its two degrees of Drained (the status retired). **Avatar** (60) keeps its full heal and its cures and now costs **half the zeal** (3) instead of half the cooldown, and widens Holy wrath's die to **every** foe. §7 |
| **Alignment interplay** | unchanged in principle: *alignment bars nothing*; every smite and Lay on hands is a holy gift that works for Kind or better (`alignment` on its talent), and outside the bands is used, takes a cast's time, and spends nothing. Smites land on any hostile, read **tags**, never the target's hidden alignment (nothing leaks). One new hook: **a deed that moves the paladin toward evil empties their zeal** ("Your zeal deserts you.", one line in `shift_alignment`; data `sin_empties_zeal`). §4.3 |
| The table | 1 **Smite**, Lay on hands, Trained wind; 6 Divine grace, Blessing; 12 **Branding smite**, Consecrate; 18 Purity of body; 25 **Retribution**; 32 **Wrathful smite**; 40 Holy radiance, **Holy wrath**; 50 **Banishing smite**, Bulwark; 60 Avatar (reworked). Nothing is cut; Smite moves from 12 to 1; the two gaps are filled and the class is smites from the first turn. The priest's Smite at 40 keeps working: the Smite talent grants Zeal to whoever is given it. §8 |
| Talents | no new `[[system]]`: smiting is combat, its pool a resource, its riders statuses and light; the talents tag what their effects read. Zeal, a resource, gets its one pickable talent under `resources` as every pool has: **Zealot** (Grit; needs Smite and level 6): a point more zeal each time you are struck (`resource_build`, an existing effect kind: no hook). §9 |
| Determinism, saves | the zeal pool is an `Actor::resources` entry (saved and hashed as Openings is); Branded is a status; the smites draw only what a blow draws, plus Wrathful's check (two gameplay draws) and its turns (one); Retribution's free blow draws as a riposte's does. Old saves load (a resource or status missing from a save is simply not held); replays with a `use:smite:E` from level 1 on, or a paladin fight (zeal built), hash differently from that step. §10 |
| Slices | five issues: **P1** Zeal, Smite onto it (Presence, the unholy die, level 1), Lay on hands onto it, Avatar's half cost, the retired statuses, dumps, tests, the balance probe's row; **P2** Branding and Wrathful smite; **P3** Retribution; **P4** Holy wrath and Banishing smite, Avatar's widened wrath; **P5** Zealot, the wiki's paladin page. §12 |

**Key choices, and why**

1. *A pool, not a timer.* The coverage policy's step 4 says it: "a cast that costs mana or a pool has no cooldown status besides (time refills it, 'time is the penalty')". Smite's Spent and Lay on hands' Drained were the stand-ins the progression design wrote before resources existed ("Lay on hands waits for its mana"); the resource system (#225, R1–R5) is built, and every other class's signature runs on one (Rage, Focus, Openings, Ki, Inspiration). Zeal is the paladin's.
2. *Zeal is built by the fight, and trickles back with time.* Diablo III's Crusader (Wrath: generated by the primary blows, spent by the big ones, a slow regeneration besides) is the model that plays best in a roguelike: the paladin opens with a few ordinary blows, is hit, and then smites; out of a fight the pool refills slowly, so a smite is still there for the next door. Openings (built, decaying) is the thief's tempo; Ki (regenerating) the monk's; Zeal is both halves, because the paladin both opens the fight and ends it.
3. *One pool for wrath and mercy.* D&D 5e's paladin spends spell slots on smites or on spells, and the choice is the class's best moment. Lay on hands on the same pool makes every zeal point a choice: smite the ghoul, or close the wound it gave you. Each resource's one pickable talent (§9) and the paladin's Presence then matter to both.
4. *Modes are rows in the abilities menu, not a toggle.* Each smite is a tactical choice made at the moment (brand the one that will flee into the dark, frighten the ogre off, unmake the summoned imp), so each is its own row (`use:branding_smite:E`), as Low blow and the Master thief's finisher are, on one code path with a `SmiteMode` table. A stance-like toggle (choose a mode, then `use:smite:E`) would hide the choice behind a setting and cost a state field; rejected (§11).
5. *Reactive smiting is a free blow.* Retribution rides `Cue::Attacked`, where the riposte already lives: a hostile's attack is over, it hit, the paladin holds the zeal: a smite answers. One branch beside the riposte's, no scheduler change, and the paladin's "stands and takes it" half of the fantasy becomes a rule.
6. *The unholy are tags, not alignment.* 206 of the 265 creature types are evil, so "smite evil" would be "smite nearly everything", and reading a creature's hidden alignment would leak it (`docs/dev/alignment.md`: never shown in play). The undead (93 types) and the fiends (the `fiend`, `demon`, `devil`, `yugoloth` families) are the unholy the fiction names, their natures are told by their names and descriptions, and a data list of tags says who they are. DCSS's holy wrath brand (undead and demons) and D&D 5e's smite (undead and fiends) draw the same line.
7. *Nothing is cut from the table.* The rules 1–4 audits (#789, #790) passed every entry the paladin has; Holy radiance is the paladin's light and Bulwark their wall. The two gaps take smite modes, Smite moves to 1 so the class is felt at once (the progression design's rule for the signature), and the passives Holy wrath and the reworked Avatar lean the top of the table into smiting without emptying what was there.

## 2. The owner's request, and what exists today

| Part | Today |
|---|---|
| The class | `data/classes/06-paladin.toml`: St `1d3`, Dx −1, In −1, Wi +1, Pr `1d3`; hp 6; mana 3, casting 50; affinity restoration 2, abjuration 1, necromancy −3; experience 110; alignment 550 (Good under any birth animal unless the race is evil, #317); Intimidate known for, Concentration, Handle animal, Heal; Long blades 2, Shield block `1d2`, Shield bash 1, Abjuration 1; Sword and board 1, Battle caster 1; the kit (long sword, small shield, hardened jerkin, iron helm, gauntlets, an amber amulet perhaps magic); bonus picks from Guard and Vigour at 10, 20, 30, 45, 55; skill rolls 2 |
| The table | 1 Lay on hands, Trained wind; 6 Divine grace, Blessing (spell); 12 Smite, Consecrate; 18 Purity of body; **25 empty**; **32 empty**; 40 Holy radiance; 50 Bulwark; 60 Avatar |
| Smite | `talents/smite.toml`: `{ kind = "smite", per_levels = 10, sides = 6, cooldown = "spent", degrees = 1 }`, `alignment = { min = "kind" }`; `world/class_ability.rs::smite`: a melee blow with what the player wields at the hostile in `dir`, `Profile::extra` the smite's dice (a magic weapon's own extra die folded into the bonus at its average), priced as the attack, exertion, then **Spent** (5 minutes a degree, `status.toml`). Refused, no time, with nobody hostile there; unanswered outside the bands ("You call for holy wrath, but none answers.", a cast's time). **Shared with the priest at 40** (`03-priest.toml`). `talent.rs::Smite::dice_at`: `level / per_levels` dice, at least one |
| Lay on hands | `{ kind = "lay_on_hands", per_level = 2, cooldown = "drained", degrees = 2 }`: 2 hit points a level, then two degrees of **Drained** (10 minutes); a cast's time; Kind or better. The class file's comment: "Lay on hands waits for its mana (the mana PR) with a cooldown". Mana exists (`docs/dev/mana.md`); the wait was never resolved |
| Avatar (60) | `{ kind = "avatar" }`: Lay on hands heals fully, cures every status that ticks and every disease, one degree of Drained |
| Divine grace (6) | disease checks +2, Poisoned eases twice as fast, drain checks +2 |
| Blessing (6) | the spell (`spells/blessing.toml`, level 3, abjuration, Blessed: +1 to hit, Willpower +1, ten minutes a degree) |
| Consecrate (12) | a minute's prayer for 15 mana that lifts an unholy room's effect for good (`world/consecrate.rs`); Kind or better |
| Purity of body (18) | no disease is ever caught (shared with the priest at 32 and the monk at 32) |
| Holy radiance (40) | a toggle: Radiant (no end of its own) sheds `item_glow` (radius 3) from the player; everyone sees it; Kind or better to light, always works to put out |
| Bulwark (50) | Protection +2 (kept by the rules 3–4 audit: "Bulwark's plain Protection +2") |
| Resources | `resources.toml` (ordered; a new one is a new entry at the end, before none): Rage, Focus, Openings, Ki, Inspiration, Stamina. `builds` events: `kill`, `hit`, `critical`, `struck`, `sneak_attack`, `sneak_kill`. A talent's `resource = { id, cost }` is a class active's price, spent by `use_class` after its refusals; `Unavailable::Short` greys the row out with the resource's `lacking` words. `max_from = { stat, per_levels, least }`. Talents: `grants_resource`, `resource_max`, `resource_start`, `resource_rate`, `resource_build`. Creatures hold none |
| Alignment | bands Wicked to Saintly; the paladin starts at 550 (Good). *Alignment bars nothing* (#642): a holy gift (`ClassActive::holy`: Lay on hands, Smite, Holy radiance, Consecrate) asks `World::answered` after its free refusals and outside its bands does nothing for a cast's time. `World::shift_alignment` is the one place a score moves; `Deed` the causes. Never a creature's alignment in play |
| Free blows | `World::second_blow(cue)`: `Cue::First`, `Attacked { parried }` (the riposte, the cleave), `Cleaved`, `Leaves`, `Approaches`, `Startled`; `World::free_blows_of` folds the actor's switches, its talents', and the weapon abilities' |
| Light from a status | `status.toml` `light = "spell_light"` on Lit (the Light spell's); `light/cache.rs` reads every actor's statuses for a `light` (line 294), so a lit creature is a light source already |
| Fear | Frightened (`flees = true`, `by`, a degree a turn, max 3): Hound's bay's, a creature of a lower level that heard the howl; `World::frightened` steps it away from `by` |
| Summons | `docs/dev/summoning.md`: the summoned record who summoned them (the dev `actors` layer's "summoned by #N"); a summoned creature's kill pays as any |
| Creatures | 265 shared types: 206 evil, 55 neutral, 4 good (the shopkeeper, the dryad, the blink dog, the nymph); tags `undead` 93, `fiend` 32, `spirit` 22, `skeleton` 13, `demon` 10, `devil` 10, `yugoloth` 5, `elemental` 36, `fey` 13 |
| The balance probe | `docs/balance.md`: the warrior (the paladin's nearest measured kin) does 8.2 a hit at level 1, 11.5 at 10, 15.2 at 20, 17.3 at 30, 20.7 at 40, 24.2 at 60, about one blow a round; the priest with a mace 5.8 to 23 |

**The gap**, in one line each: the paladin's signature is a heal with a ten-minute timer; its one blow comes at level 12 and once in five minutes, so a paladin fights like a warrior with worse numbers and a cure; the two timers are the stand-ins the progression design wrote before resources existed; levels 25 and 32 are empty; nothing in the class reads Presence, the stat it is given `1d3` of; and nothing the paladin does is different against a ghoul than against a goblin.

## 3. Research, tied to decisions

| Source | What it does | Taken, or not, and where |
|---|---|---|
| **D&D 3.5** (PHB 2003): Smite evil, once a day at level 1 (more with levels): Charisma bonus to the attack, a point of damage a level, **only against evil** (wasted on anyone else); Lay on hands, a daily pool of level × Charisma; Divine grace (Charisma to saves), Aura of courage, Divine health, Turn undead, Remove disease, a mount; a code of conduct, and the fall | **Charisma to hit** → Presence to hit on every smite (§5). Smite evil's "only evil" → **not taken**: 78 % of the types are evil, and the target's alignment is hidden (choice 6). "Once a day" → not taken: uses a day are a timer in a longer coat; Zeal replaces them (choice 1). Divine health, Remove disease → Purity of body, Divine grace, kept |
| **D&D 5e** (2014): Divine Smite: on a melee hit, spend a spell slot for +2d8 radiant, +1d8 a slot level (to 5d8), **+1d8 against undead and fiends**; Improved Divine Smite (11): +1d8 radiant on **every** melee hit; Lay on Hands, a pool of 5 × level; Channel Divinity (Sacred Weapon: Charisma to attack; Turn the Unholy; Vow of Enmity; Abjure Enemy); Aura of Protection and Courage; Cleansing Touch; the **smite spells**: Searing (fire, burning), Thunderous (pushed, prone), **Wrathful** (frightened), **Branding** (sheds light, cannot turn invisible), Blinding, Staggering, **Banishing** (sent away at 50 hit points or fewer). The 2024 rules make Divine Smite a spell cast once a turn as a bonus action, and the smite spells bonus actions | **The spell-slot tension** (smite or spell) → one pool for smites and Lay on hands (choice 3). **+1d8 against undead and fiends** → the unholy die (§5). **Improved Divine Smite** → Holy wrath at 40 (§6.5). **Wrathful, Branding, Banishing** → the three modes (§6), each turned to a rule Warlock has: fear that `flees`, a status that sheds light, the summoned unmade. Thunderous → not taken (Shield rush, a weapon ability of the paladin's own style, is the knockback); Blinding → not taken (no blindness status, and Dazzled is light-blind eyes only); Staggering → not taken (Stunning bash and Stunning fist are Staggered). Auras → not taken (no allies to shelter until the pet design, #825) |
| **Pathfinder 1e**: Smite evil picks a target: Charisma to attack, a point of damage a level (doubled on the first blow against an evil outsider, dragon, or undead), bypasses damage reduction, a Charisma deflection bonus to AC against that target, lasting until it dies; uses a day. Mercies: riders on lay on hands (fatigued, shaken, diseased, poisoned, ...). Divine bond. Aura of justice (shared smite). Holy Champion (20): a smite banishes an evil outsider | The **doubled dice against the unholy** → Banishing smite's second half (§6.3). "A sworn foe" (the lasting mark) → **not taken**, with regret (§11: a mark on the target needs a new Defence and to-hit hook reading `Effect::by`, and the fight it suits, one big foe, is rare below the depths). Mercies → Avatar's cures, kept as they are |
| **Pathfinder 2e** (the Champion): Retributive Strike, a reaction when a foe damages an ally in reach: the ally suffers less and the champion strikes; Divine Smite (persistent good damage on it); Smite Evil (extra good damage against one foe); Lay on Hands as a focus spell (a point, refocused by ten minutes' rest) | **Retributive Strike** → Retribution (§6.4): the paladin has no ally to defend, so the blow it answers is the one that hit them. Focus points refilled by rest → Zeal's slow trickle and `resting = true` (§4) |
| **Diablo II** (Paladin): Smite (a shield blow that never misses, stuns, knocks back), Zeal (several swings), Sacrifice (more damage at the cost of one's own blood), Vengeance (elemental damage), Holy Bolt, Blessed Hammer, Fist of the Heavens; the auras (Fanaticism, Conviction, Sanctuary: the undead thrown back, Redemption, Holy Shield) | The **name Zeal** for the pool (the fervour that mounts in the fight). Sacrifice → a pickable "smite for blood" considered and **not taken** (Blood magic is that trade for mana, and one such trade is enough). Auras → not taken (as above) |
| **Diablo III** (Crusader): **Wrath**, a resource generated by the primary blows (Punish, Slash, Smite, Justice) and spent by the big ones, regenerating slowly besides; Laws; Akarat's Champion | **The tempo of Zeal** (choice 2): built by the blows given and taken, a slow refill with time. Diablo IV's paladin (announced for its second expansion) is too new to draw on |
| **DCSS**: the Shining One (Cleansing Flame: a holy burst that hurts only the evil, the undead, and demons; Divine Shield; Summon Divine Warrior; the **halo**, a light about the worshipper that reveals the hidden and ends stealth; a conduct against attacking the unaware or the fleeing); the **holy wrath** brand (more damage against the undead and demons, none against the rest; not for the undead or demonic to wield); Zin (Recite, Sanctuary); Elyvilon (healing). The old Paladin background (a TSO fighter) was removed | **Holy wrath** by name and rule (§6.5): one die against the undead and the fiends, nothing extra against the rest. The halo → Holy radiance already (kept at 40; Branding smite is a halo put on the foe). TSO's conduct → **not taken**: no system of conducts, and the owner's "no anti-farming limits"; the alignment score and the bands are Warlock's conduct, and a sin empties zeal (§4.3) |
| **ADOM**: the class powers at 6, 12, 18, 25, 32, 40, 50 that differ in kind (the progression design §4.1 took the cadence) | the cadence 1, 6, 12, 18, 25, 32, 40, 50, 60, kept |
| **Angband** (4.2): the paladin is the fighter with the priest's first book (Call Light, Bless, Minor Healing, Protection from Evil) | Blessing kept at 6; Holy radiance is Call Light |

## 4. Zeal: the pool

### 4.1 Data

```toml
# The paladin's zeal (the Smite talent grants it; spec §8 "Class abilities": "Smite"): the
# fervour that mounts in a fight. It is built by blows given and taken, and comes back slowly
# with time; smites and Lay on hands spend it. It sits at 0 when spent.
[[resource]]
id = "zeal"
name = "Zeal"
short = "Zeal"
max = 10
max_from = { stat = "presence", per_levels = 10, least = 6 }
start = 5
regenerates = 60
builds = [{ on = "hit", amount = 1 }, { on = "struck", amount = 1 }]
lacking = "your zeal is not enough"
```

The last entry of `resources.toml` (file order is save order). `ResourceId` gains an index; the fingerprint changes as any content change does. Nothing in `content/resource.rs` is new: every field exists.

| Rule | Decision | Why |
|---|---|---|
| **Maximum** | 10 + the Presence modifier + a point per 10 levels, at least 6 (`max_from`, Stamina's shape). A level 1 paladin with Presence 12 holds 11; at level 30 with Presence 16, 16; the pickable Zealot adds nothing to it (§9) | Presence is the paladin's stat (`1d3`) and read nowhere in a fight today; D&D's Charisma |
| **Granted** | at 5 (`start`), by the Smite talent at level 1 (`grants_resource`; `focus::hold` from `class::hand_out` and `World::grant_class`, so a save of a paladin made before this holds it on load) | a smite at the first door, not a whole fight of waiting |
| **Builds** | +1 when a blow, shot, or throw of the holder's hits (`Build::Hit`, raised in `fight.rs::resolve` before the damage, so a killing blow counts); +1 when the holder is hit (`Build::Struck`, the same place) | the two events the fight already raises; a paladin who hits six times in ten and is hit four builds about a point a round. Kills build nothing: Inspiration's event, and a smite should not feed itself |
| **Time** | a point every 60 seconds (`regenerates = 60`: ten base turns), resting too (`resting` default). Empty to full (10) in ten minutes; a rest to dawn fills it | the span Lay on hands' Drained covers today, so the out-of-fight heal keeps its pace; "time is the penalty" |
| **Decay** | none. Zeal sits where it is; it never ends | a resource that both builds and decays would be two timers; Rage's decay is Rage's |
| **Who** | whoever holds the Smite talent: the paladin from level 1, the priest from 40 (§8.3). Creatures never |  |

**Costs** (each the `resource = { id = "zeal", cost }` of its talent, or the effect's own `cost` for a passive; all data):

| Spend | Zeal | Of a pool of 10 |
|---|---|---|
| Smite | 3 | a third |
| Branding smite | 3 | a third |
| Wrathful smite | 4 | |
| Retribution (each answering blow) | 4 | |
| Banishing smite | 6 | most of it |
| Lay on hands | 6 (3 with Avatar) | most of it |
| Holy wrath | 0 | free |

**A worked fight.** A level 10 paladin (Presence 13: maximum 12, holding 5 after a quiet walk) meets three orcs in a corridor. Round 1: a blow hits (6), an orc hits back (7). Round 2: Smite (−3: 4; its +2d6 lands: 7 more on 11.5), hit again (5). Round 3: a blow hits (6), hit (7). Round 4: Smite (4). The second orc dies to it; the third hits twice over two rounds (6) while two blows land (8): Lay on hands (−6: 2) closes 20 hit points, and the fight ends with the pool near empty, filling a point every ten turns on the walk to the next door. Against a ghoul pack the same rhythm, each smite a die bigger.

### 4.2 Where it is read

Nothing new in the resource code: `World::spend_resource` from `use_class` (the actives) and from `world/smite.rs::retribution` (the free blow); `gain_resource` where `Hit` and `Struck` already fire; `Ability::unavailable`'s `short_of` greys the rows out ("your zeal is not enough"). The status bar shows `Zeal 7/12` after MP (`show_max` true), the character screen its row, the `resources` dump its line.

### 4.3 Alignment and zeal: one hook

`data/smite.toml` (§5.1) carries `sin_empties_zeal = true`: in `World::shift_alignment`, when the **player's** score moves by a negative amount (a deed toward evil: killing the peaceful, theft, attacking a keeper, eating kin, an evil altar's prayer, a quest's shift), their zeal, if held, is set to 0, and "Your zeal deserts you." is said with the deed's own lines. One `if` after the score moves; a diminished deed that moves 0 empties nothing. It bars nothing (the pool fills again; the gifts answer or not by the bands as today); it is the moment the fiction asks for, and the only new reading of alignment in this design.

The bands stay as they are: every smite and Lay on hands names `alignment = { min = "kind" }`. A paladin who has fallen to Neutral calls for wrath and none answers (a cast's time, no zeal spent, the line each mode has); their zeal still builds, and Holy wrath's free die still lands (a passive: the paladin's passives never ask for Good, as today; §7).

## 5. Smite: the one blow

### 5.1 Data

```toml
# data/smite.toml: the paladin's smiting (spec §8 "Class abilities": "Smite"). Optional:
# without it nothing is unholy and no sin empties zeal (fixtures).
[smite]
# What holy wrath falls hardest on: a creature type carrying any of these tags takes the
# smite's extra die, Holy wrath's die, and the Banishing smite's doubled dice. Checked on load:
# each tag is carried by some type.
unholy = ["undead", "fiend", "demon", "devil", "yugoloth"]
# A deed that moves the player toward evil empties their zeal (§4.3).
sin_empties_zeal = true
```

```toml
# talents/smite.toml (the paladin's at 1, the priest's at 40)
[[talent]]
id = "smite"
name = "Smite"
tree = "class"
granted = true
description = "A blow of holy wrath in a direction (an ability, paid in zeal): surer for your presence, harder as you grow, and harder still on the unholy. Your zeal rises as you give and take blows."
effects = [
    { kind = "smite", per_levels = 10, sides = 6, aim = "presence", unholy = 1 },
    { kind = "grants_resource", resource = "zeal" },
]
resource = { id = "zeal", cost = 3 }
systems = ["combat", "resources"]
alignment = { min = "kind" }
```

### 5.2 Rules

| Rule | Decision | Where |
|---|---|---|
| **The blow** | as today: a melee blow with what the player wields at the hostile in `dir`, priced as the attack (`attack_price`), one exertion, the free blows an attack's end brings (`after_attack`). Refused, no time, with nobody hostile there ("There is nobody there to strike."), without a direction ("Which way?") | `world/smite.rs::smite(mode, dir)`, moved out of `class_ability.rs` (one line there: the dispatch) |
| **The dice** | `level / per_levels` dice of `sides`, at least one (`Smite::dice_at`, as today): 1d6 to level 9, 2d6 at 10, 3d6 at 20, ..., 7d6 at 60. **One die more** (`unholy`) against a target whose type carries an unholy tag. `Profile::extra` the dice; a magic weapon's own extra die folded into the bonus at its average, as today. The dice are of the blow's own kind of damage (slashing for a long sword), resisted as the blow is: no holy damage type (§11) | `smite.rs`, `Content::unholy(creature)` the one reader of the tags |
| **To hit** | the paladin's **Presence modifier** (`aim = "presence"`, the effective score's modifier, `Actor::modifier`) added to the profile's `to_hit` | `smite.rs`; `aim` is the stat id, checked on load |
| **The cost** | 3 zeal, spent by `use_class` after the free refusals and the bands (so a refused or unanswered smite spends nothing), as every class active's `resource` is | `use_class` (exists) |
| **The bands** | Kind or better; outside them `World::unanswered`: "You call for holy wrath, but none answers.", a cast's time, nothing spent. **Order**: the free refusals (nobody there), then the bands, then the spend, then the blow: the same order the thief's actives keep | `use_class`, `answered` (exist) |
| **Criticals** | the smite's dice ride `Profile::extra`, so a confirmed critical multiplies them with the rest, as a magic weapon's extra die is multiplied today. Kept: a smite that crits is the paladin's best moment | no change |
| **Who** | the paladin from level 1; the priest from 40 (§8.3) | the tables |
| **Numbers** (the wiki states them) | level 1: 1d6 (3.5) on a long sword's 8.2, +1 to hit from Presence 12 or 13: a smite does 1.4 times a blow; level 20: 3d6 (10.5) on 15.2, 1.7 times; level 60: 7d6 (24.5) on 24.2, twice. Against a ghoul at level 10: 3d6 (10.5) on 11.5. With a smite every third blow the paladin's damage over a fight is about a quarter to a third above the warrior's at the same gear, less the warrior's own actives (Whirlwind, Cleaving strike) and ranks; the balance probe gets a paladin column (P1) so the number is watched. The owner accepts unbalanced choices; the dice are data |

## 6. The modes of smiting

Every mode is a class ability of the `class` tree at its level, `granted`, `alignment = { min = "kind" }`, with a `smite_mode` effect and its `resource` cost; each is a `ClassActive` (`BrandingSmite`, `WrathfulSmite`, `BanishingSmite`: aimed, `strikes()`), a row of the abilities menu under Class, and `use:branding_smite:E` in action text. All resolve through `smite.rs::smite(mode, dir)`: the blow of §5 (the dice, Presence, the unholy die), then the mode's rider **on a hit that did damage** (as Quivering palm's bleeding asks). A mode's rider on a target with nothing for it to do (Banishing on a goblin) still spends its zeal: the paladin can see what they strike.

```toml
# The effect kind the modes share (the effects README of data/talents/):
#   smite_mode { mode, status, severity, turns, check, difficulty }
#     mode       brand, fear, or banish
#     status, severity   brand: the status the foe hit suffers, this many degrees
#     status, turns, check, difficulty   fear: the status (one that `flees`), its degrees drawn
#                from `turns`, after a check of `check` against `difficulty` plus the smiter's
#                aim modifier that the foe may make
#     banish takes nothing more: what it does is the rule's (§6.3)
```

### 6.1 Branding smite (level 12, 3 zeal)

```toml
[[talent]]
id = "branding_smite"
name = "Branding smite"
tree = "class"
granted = true
description = "A smite that leaves holy light clinging to the foe (an ability, paid in zeal): it shines for all to see, and the dark hides it from you no more."
effects = [{ kind = "smite_mode", mode = "brand", status = "branded", severity = 1 }]
resource = { id = "zeal", cost = 3 }
systems = ["combat", "resources", "status", "light"]
alignment = { min = "kind" }
```

```toml
# status.toml, after Radiant: Branding smite's (the paladin's). Holy light clings to the foe
# for five minutes: it sheds the faint item glow wherever it goes.
[[status]]
id = "branded"
name = "Branded"
seen = "shining"
max = 1
stacking = "refresh"
eases = 300
light = "item_glow"
start = "Holy light clings to you."
end = "The holy light about you fades."
```

| What it does | How, and the one hook |
|---|---|
| The foe sheds `item_glow` (radius 3) for five minutes | `light` on the status: `light/cache.rs` reads every actor's statuses for a light already (Lit); **no new reader**. Gameplay light: the player sees the branded foe in line of sight whatever the dark (`Perception::compute`), creatures see it and each other by it, the paladin's own cell is lit when it stands beside them |
| It cannot come at the paladin out of the dark | follows: a lit creature in line of sight is in the perception snapshot, so `World::unaware(player, it)` is false (`docs/dev/surprise.md`); nothing to code |
| It lights its fellows | follows from the light; a branded orc in a dark den shows the paladin the den |
| A wounded foe that flees into the dark is followed | the fantasy's point: the paladin's fights are in corridors and doorways, and the thing that breaks and runs is the one that comes back |
| Look mode | "shining" (its `seen`); the player is told "Holy light clings to the orc." as the rider lands |
| Message | "You smite, and holy light clings to the orc." |

### 6.2 Wrathful smite (level 32, 4 zeal)

```toml
[[talent]]
id = "wrathful_smite"
name = "Wrathful smite"
tree = "class"
granted = true
description = "A smite that breaks the foe's nerve (an ability, paid in zeal): unless its will holds, it runs from you a while."
effects = [{ kind = "smite_mode", mode = "fear", status = "frightened", turns = "2d3", check = "willpower", difficulty = 13 }]
resource = { id = "zeal", cost = 4 }
systems = ["combat", "resources", "status"]
alignment = { min = "kind" }
```

| Rule | Decision |
|---|---|
| **The check** | the foe hit makes `check::check(rng, its Willpower modifier, 13 + the smiter's Presence modifier)`: two gameplay draws, drawn only on a hit that did damage. Made: "The ogre's will holds." Failed: Frightened, `2d3` degrees (one draw), `by` the paladin: it steps away from them while a step takes it farther (`World::frightened`, exists), a degree a turn |
| **Odds** (the wiki states them) | Willpower 10 against 13 + 2 (Presence 14): 78 of 144 fail (54 %); a drow priest at Willpower 16 (+3) against the same: 36 % |
| **Who fears** | everyone: a frightened ghoul is turned undead by another name (D&D's Turn Undead is this). No immunity list: a creature with `vitals = false` (the surprise design's crit immunity) is not an exception either, since fear is not a wound |
| **Why 32** | the Steadfast slot: a mode for the fight that is going badly, which is a level-30 paladin's fight with a troll or an ogre; it buys the turns a Lay on hands needs. Frightened's `max = 3` holds the degrees (a 6 is three) |
| **Message** | "You smite, and the ogre quails!" / "You smite, but the ogre's will holds." |

### 6.3 Banishing smite (level 50, 6 zeal)

```toml
[[talent]]
id = "banishing_smite"
name = "Banishing smite"
tree = "class"
granted = true
description = "A smite that sends the summoned back whence they came (an ability, paid in zeal): what was called is unmade outright, and the unholy take your wrath twice over."
effects = [{ kind = "smite_mode", mode = "banish" }]
resource = { id = "zeal", cost = 6 }
systems = ["combat", "resources"]
alignment = { min = "kind" }
```

| Target hit | What happens |
|---|---|
| A **summoned** creature (one with a summoner recorded, `docs/dev/summoning.md`), whatever its hit points | it is **unmade**: it dies there (`World::creature_dies` with the smite as the killer, so the kill pays as any kill of a summon does today, and quests count it); "You smite, and the imp is unmade!" |
| An **unholy** creature not summoned | the smite's dice are rolled **twice** (the extra dice doubled, `Profile::extra` rolled again, the bonus once, as Mighty swing's `DoubleDice` doubles dice and not bonus); "You smite, and the draugr reels under holy wrath!" |
| Anyone else | a plain smite's dice; the zeal spent |

Why summons: a summoner behind a door with a stream of fiends is the fight a level-50 paladin has (the summoning hall, `docs/dev/summoning.md`), and a blow that ends a summon outright for most of the pool is a tactic, not a number. D&D's "banished at 50 hit points or fewer" is a kill one blow early and was rejected as that (§11).

### 6.4 Retribution (level 25, passive, 4 zeal a blow)

```toml
[[talent]]
id = "retribution"
name = "Retribution"
tree = "class"
granted = true
description = "A foe beside you that lands a blow on you takes your smite at once, while your zeal lasts."
effects = [{ kind = "retribution", cost = 4 }]
systems = ["combat", "resources", "free_blows"]
alignment = { min = "kind" }
```

| Rule | Decision | Where |
|---|---|---|
| **When** | a hostile's melee attack on the player is over (`Cue::Attacked`, where the riposte lives), **at least one of its blows hit** (`parried` is the riposte's cue; `hit` is Retribution's: `Cue::Attacked { parried, hit }`, one bool more), the attacker stands beside the player and lives, the player holds Retribution, holds 4 zeal, and the gift is answered (the bands). Then: `spend_resource(zeal, 4)` and a smite blow (§5's profile: the dice, Presence, the unholy die; no mode rider) at the attacker, free (no action, no price), one exertion | `world/smite.rs::retribution`, one branch in `World::second_blow`'s `Attacked` arm, **before** the riposte's: an attack answered by Retribution is not also riposted (one free blow per attack, as today) |
| **Zeal's order** | the hit that triggers it has already built a point (`Struck` fires in `resolve`, before the attack's end), so a paladin at 3 can be struck to 4 and answer | follows |
| **Bands** | unanswered: nothing happens, no line (a passive that says "none answers" every time the paladin is hit would be noise); the paladin learns it from the character screen's band as today | `answered(ClassActive::Smite)` reused: Retribution is a smite |
| **Creatures** | never: creatures hold no resources |  |
| **Why 25** | the Shield of faith slot (the report's "direction: shield"): the paladin who stands in the doorway and takes the blows is the shield, and this is what the standing buys. The level-25 fight has several foes beside the paladin; Retribution is a smite a round against the one that lands, for as long as the zeal they build by being hit lasts: a paladin surrounded is a paladin smiting |
| **Message** | "You answer the blow with holy wrath!" then the blow's own line |
| **Determinism** | the free blow's draws at the attack's end, as the riposte's; a replay in which a level-25 paladin is hit hashes differently from that creature turn |

### 6.5 Holy wrath (level 40, passive, free)

```toml
[[talent]]
id = "holy_wrath"
name = "Holy wrath"
tree = "class"
granted = true
description = "Every blow of yours that lands on the unholy carries a die of your god's wrath."
effects = [{ kind = "holy_wrath", dice = "1d6" }]
systems = ["combat"]
```

Every melee blow, shot, or throw of the paladin's that hits an unholy target (the tags of §5.1) rolls `dice` more, after the blow's own damage, of the blow's kind, resisted but never soaked (the `ExtraDamage` rule weapon abilities already have: "one more roll of dice on the target, after the blow's own damage"), no zeal. One die, not the smite's dice: D&D's Improved Divine Smite is a flat d8, and DCSS's holy wrath a multiplier on the blow; a scaling free rider on every blow would make the smite itself pointless against the undead. A smite on an unholy foe carries both (the smite's dice plus one, and Holy wrath's die). No bands: a passive, as every paladin passive (§7). Read in `World::resolve` after the strike, beside `ambushed` (the sneak dice land there), through `TalentMods::holy_wrath: Option<Dice>` and `Content::unholy`.

## 7. Lay on hands, Avatar, and the rest of the table

| Ability | Today | After |
|---|---|---|
| **Lay on hands** (1) | 2 hit points a level, then two degrees of Drained (10 minutes), a cast's time, Kind or better | `effects = [{ kind = "lay_on_hands", per_level = 2 }]`, `resource = { id = "zeal", cost = 6 }`; the cooldown fields gone (`LayOnHands { per_level }`); spent by `use_class` as any active's cost; the heal and the line as today ("Warmth flows from your hands."). Greyed out "your zeal is not enough" under 6. **Out of a fight** the pool refills a point in ten turns, so a heal every sixty turns (six minutes): near the ten-minute cooldown it replaces, a little kinder; **in a fight** it competes with the smites, which is the design |
| **Avatar** (60) | Lay on hands heals fully, cures every status that ticks and every disease, one degree of Drained | `{ kind = "avatar", hands_cost_percent = 50 }`: the full heal and the cures as today, for **half the zeal** (3, `percent_of`, at least 1); **and Holy wrath's die lands on every foe**, not only the unholy (`TalentMods::avatar` read in `resolve`'s Holy wrath branch). A capstone that changes a fight: every blow of the paladin carries the wrath, and mercy is cheap |
| Trained wind (1), Divine grace (6), Blessing (6), Consecrate (12), Purity of body (18), Holy radiance (40), Bulwark (50) | | **kept as they are**. Holy radiance stays the paladin's light (the rules 1–2 audit passed it: a light below ground changes a dungeon); Bulwark their wall (the rules 3–4 audit kept it). The `holy` passives never ask for Good; the gifts used from the menu do, as the spec says |

The **Spent** and **Drained** statuses lose their last users (Quivering palm lost Spent to Ki in R4; the weapon actives have no cooldowns "in the catalogue"). They are **deleted** from `status.toml` in P1. `StatusId` is an index in file order, so a save made before P1 reads its statuses off by two for every id after Spent's: the save format policy (`docs/dev/merging.md`: `FORMAT_VERSION` bumped once a release by the integrator) covers it, and the dev `actors` layer is where it would show. If the integrator would rather not pay a bump for this, the alternative is to leave the two entries in place, unused, until the next bump (open question 9).

## 8. The table reworked: what stays, changes, and goes

### 8.1 The paladin

| Level | Today | After | Stay, change, go |
|---:|---|---|---|
| 1 | Lay on hands, Trained wind | **Smite** (grants Zeal), Lay on hands, Trained wind | Smite **moved here from 12** and onto zeal (§5); Lay on hands onto zeal (§7); Trained wind stays |
| 6 | Divine grace, Blessing | Divine grace, Blessing | stay |
| 12 | Smite, Consecrate | **Branding smite**, Consecrate | Smite's slot takes the first mode (§6.1); Consecrate stays |
| 18 | Purity of body | Purity of body | stays (shared) |
| 25 | — | **Retribution** | the gap filled (§6.4) |
| 32 | — | **Wrathful smite** | the gap filled (§6.2) |
| 40 | Holy radiance | Holy radiance, **Holy wrath** | Holy radiance stays; Holy wrath added (§6.5) |
| 50 | Bulwark | **Banishing smite**, Bulwark | Banishing smite added (§6.3); Bulwark stays |
| 60 | Avatar | Avatar (reworked) | half the zeal, wrath on every foe (§7) |

Nine levels, fourteen entries, nine of them smiting or zeal: Smite, Lay on hands (the pool's other spend), Branding, Retribution, Wrathful, Holy wrath, Banishing, Avatar, and Zealot the pick. Three rows at level 1: the thief has two; all three are small, and the class is felt from the first turn (Smite's the signature; a heal and a breath beside it).

```toml
abilities = [
    { level = 1, talent = "smite" },
    { level = 1, talent = "lay_on_hands" },
    { level = 1, talent = "trained_wind" },
    { level = 6, talent = "divine_grace" },
    { level = 6, talent = "blessing" },
    { level = 12, talent = "branding_smite" },
    { level = 12, talent = "consecrate" },
    { level = 18, talent = "purity_of_body" },
    { level = 25, talent = "retribution" },
    { level = 32, talent = "wrathful_smite" },
    { level = 40, talent = "holy_radiance" },
    { level = 40, talent = "holy_wrath" },
    { level = 50, talent = "banishing_smite" },
    { level = 50, talent = "bulwark" },
    { level = 60, talent = "avatar" },
]
```

The class description keeps its words. The kit is unchanged. Bonus picks (Guard, Vigour at 10, 20, 30, 45, 55) unchanged.

### 8.2 Old saves

`World::grant_class` on `from_state` gives a saved paladin the talents its table now names up to its level (a grown table reaches saves; nothing is taken away), so a level-12 paladin saved today loads with Smite (already held), Branding smite, and Zeal (held at its start of 5, through `focus::hold`). A Spent or Drained degree in a save is the status-index question of §7.

### 8.3 The priest's Smite

The priest's table names `smite` at 40 (`03-priest.toml`). The Smite talent grants Zeal to whoever is given it, so the priest at 40 holds the pool and smites for 3 zeal (built by their mace and by being struck; refilled with time), with Presence to hit and the unholy die, as the paladin does. The priest's design (#820) owns the priest's table and may move or replace it; nothing here decides the priest's table beyond keeping its Smite working on the day P1 lands. (If #820 removes the priest's Smite, Zeal becomes the paladin's alone and nothing in this design changes.)

## 9. Talents (the coverage policy)

| Question | Answer |
|---|---|
| A new `[[system]]`? | **No.** Smiting is the paladin's family of combat actives: its blows read `combat`, its pool `resources`, its riders `status` and `light`, its free blow `free_blows`. A class's ability family is not a broad system of the game (the policy: "a monster family's gimmick, a chest kind, or an info readout is never a system of its own: it extends its parent system"); the thief's `sneak_attack` is a system because it is a skill anyone (and creatures) may have. The talents above tag what their effects read ("Tags match hooks"): Smite `combat`, `resources`; Branding `combat`, `resources`, `status`, `light`; Wrathful `combat`, `resources`, `status`; Banishing `combat`, `resources`; Retribution `combat`, `resources`, `free_blows`; Holy wrath `combat`; Lay on hands `hit_points`, `resources`; Avatar `hit_points`, `status`, `diseases`, `combat` |
| The resource's talent (step 4: "each joins the list with talents on its own hooks") | **Zealot**, the one pickable, under `resources`, as Battle trance (rage), Inner calm (ki), Steady hand (focus), and Patient blade (openings) are each a pool's: |

```toml
[[talent]]
id = "zealot"
name = "Zealot"
tree = "grit"
description = "Every blow that lands on you stokes your zeal the more."
needs = { talents = { smite = 1 }, level = 6 }
effects = [{ kind = "resource_build", resource = "zeal", on = "struck", amount = 1 }]
systems = ["resources"]
```

| The bar, line by line | Zealot |
|---|---|
| A real choice at its tier | it doubles the zeal a paladin builds by being hit (two a blow taken), so a surrounded paladin smites every other round and Retribution (25) fires as often as it is struck: a build that stands in the doorway, against one that picks its fights (Guard's Dodge, Vigour's hit points at the same picks). Grit is the will's tree |
| Not information, not a small percent on a niche mechanic, not a duplicate | a rule change on the class's pool; no other talent, class ability, or skill rank builds zeal |
| Rule 4 (one effect, two ids, chained, is a rank) | it needs Smite, whose effect kinds are `smite` and `grants_resource`; Zealot's is `resource_build`: no kind in common, so no chain |
| Who may take it | whoever holds Smite from level 6: the paladin; the priest from 40 (§8.3). Class-limited by its prerequisite, as Patient blade is by `class` |
| New hooks | **none**: `resource_build` exists (`World::gain_resource` sums the talents' builds) |

No second pickable. The policy: "a count is never a reason to add a talent". The spec's proposed alignment talents (Righteous fury, Hardened heart, The middle way; `docs/dev/alignment.md`) are not taken here: there is no `alignment` system in the list, and a talent reading the player's band at the moment of use is a question for an alignment design, not this one.

## 10. What the player sees; determinism, saves, replays; the wiki and the docs

| Surface | What |
|---|---|
| The abilities menu (Class group) | Smite, Branding smite, Wrathful smite, Banishing smite, Lay on hands, Holy radiance, Consecrate (and Blessing under Spells), each with its description; greyed "your zeal is not enough" while short; the character's band never named by a row (as today) |
| The status bar | `Zeal 7/12` after MP (priority 1, gives way before it) |
| The character screen | the Zeal row under the stance, as every pool |
| Messages | "You smite!" (plain); "You smite, and holy light clings to the orc." / "You smite, and the ogre quails!" / "You smite, but the ogre's will holds." / "You smite, and the imp is unmade!" / "You smite, and the draugr reels under holy wrath!"; "You answer the blow with holy wrath!"; "Warmth flows from your hands."; "Your zeal deserts you."; unanswered: "You call for holy wrath, but none answers." (every smite), "You lay on your hands, but no warmth flows from them."; a Branded or Frightened creature's lines in the third person as today ("The orc is frightened." by the status's words). **No number**: the dice, the check, the pool's pace stay in the wiki |
| Look mode | "shining" (Branded), "frightened" (exists) |
| Dumps | the header's `zeal 7/12`; `abilities` ("Smite (class): a direction", "Banishing smite (class): not enough zeal" through `Unavailable::Short`'s words); `talents`; `actors` (the dev layer: `resources zeal 7/12 (0)`, statuses `branded 1`); JSON `resources`, `abilities`, `effects` |
| Action text | `use:smite:E` (unchanged), `use:branding_smite:E`, `use:wrathful_smite:E`, `use:banishing_smite:E`, `use:lay_on_hands` (unchanged); Retribution and Holy wrath have none (passives) |
| Determinism | Zeal's builds and trickle draw nothing (`resource::pass`); a smite's draws are a blow's (the strike, the dice) plus, for Wrathful, the check (two gameplay draws) and the turns (one), only on a hit that did damage; Branding and Banishing none beyond the blow; Retribution's free blow at `Cue::Attacked`, before the riposte's would be; marks on the training stream as any blow's. Replays with a smite before level 12, a paladin fight (the pool), or a level-25 paladin struck hash differently from that step |
| Saves | the pool in `Actor::resources` (saved and hashed; left out while none); Branded a status as any; `Mind` untouched. A save without the pool holds it on load (§8.2) |
| Wiki | the classes page's paladin row and table (generated: `xtask/src/wiki/character.rs` reads the class tables); `resources.md`'s table gains Zeal's row by itself; the status table Branded; a **Paladin** section on the classes page (prose: the pool, each smite's numbers, Retribution's rule, Holy wrath's die, the bands, the sin that empties zeal; every term linked, `wiki/TERMS.md`); the balance page's new paladin column (P1) |
| Docs | spec §8 "Class abilities" (the actives' and passives' lists), §8 "Resources" (Zeal), §8 "Alignment" (the sin hook, one line); `docs/dev/progression.md` (the hooks table: `smite` widened, `smite_mode`, `retribution`, `holy_wrath`, `avatar`'s fields; the modules table: `world/smite.rs`), `docs/dev/resources.md` (Zeal, its builds), `docs/dev/alignment.md` (the hook), `docs/dev/status-effects.md` (Branded), `docs/dev/summoning.md` (the banishing); this document stays as the record |

## 11. Choices made along the way, and what is rejected

| Idea | Verdict | Why |
|---|---|---|
| **A mode toggle** (`smite_mode:brand`, then `use:smite:E`) | rejected | the choice is tactical and per blow; a toggle hides it behind a setting, adds a state field, and makes the plain smite a fourth setting. Rows are what the menu is for (Low blow, Master thief) |
| **Smite works only on evil; nothing on the good** (D&D 3.5) | rejected | 206 of 265 types are evil (the rule would nearly always say yes), the four good types are peaceful by nature (killing them is `kill_peaceful` already), and a smite that fizzles on a creature would tell the player its hidden alignment (`docs/dev/alignment.md`: never in play). The unholy by **tag** is the line that reads nothing hidden |
| **A holy damage type** (`radiant`), the unholy vulnerable to it | rejected | it needs a `resist` entry on about 130 creature files or a rule that reads tags anyway; the smite's dice of the blow's own kind plus an unholy die reach the same place with one list. Reconsider if a holy *spell* school arrives |
| **Uses a day / a long cooldown** (3.5, PF1) | rejected | a timer in a longer coat; the policy's step 4 and "time is the penalty" put every class active on a pool or mana |
| **Lay on hands on mana** (the class file's old note) | not taken | the paladin's mana (3 a level at casting 50 %) is Blessing's and Consecrate's; a heal on it would compete with nothing. On zeal it competes with the smites, which is the design (choice 3). Open question 3 |
| **A sworn foe** (PF1's lasting smite: Charisma to hit and to Defence against one target until it dies) | not taken, with regret | a mark on the target read by `to_hit` and `Actor::defence` through `Effect::by` is a new hook in two hot places; its fight (one big foe) is the depths' and not the first dungeon's. A candidate for a later pick if the owner wants a sixth mode |
| **Thunderous, Staggering, Blinding smites** | not taken | Shield rush (knockback), Stunning bash and Stunning fist (Staggered) are weapon abilities of the paladin's own style and skills ("an effect another ability already gives"); there is no blindness, and Dazzled is light-blind eyes only |
| **Banishing at half hit points** (5e) | rejected for the summoned-outright rule | a kill a blow early is a number; unmaking a summon whatever its hit points is a tactic against the summoners the deep levels have |
| **Auras** (Protection, Courage; Diablo's) | not taken | the player has no allies to shelter until pets (#825); a self-aura is a passive by another name |
| **Smite for blood** (Diablo II's Sacrifice; a smite paid in hit points when the zeal is dry) | not taken | Blood magic is the game's one such trade (hit points for mana), and the thesis of one pool for wrath and mercy is weaker if wrath has a second purse |
| **Oaths / subclasses** (5e) | not taken | the modes are the oaths' flavour without a second class axis; the owner's talent policy prefers few, real choices |
| **A conduct** (TSO's: no attacking the unaware or fleeing) | rejected | no system of conducts; the alignment score and bands are Warlock's conduct; the owner's "no anti-farming limits". The sin hook (§4.3) is the one nod |
| **Kills build zeal** | rejected | a smite that kills would refund itself; Inspiration is the kill-built pool |
| **Zeal decays** (as Rage and Openings) | rejected | a pool that both builds and decays is two timers; the slow refill is enough, and it makes the out-of-fight heal possible |
| **Restoring Shield of faith and Steadfast** | rejected | #798 cut them as filler (a flat Shield +1 and a burden rule the dwarf has); the owner asked for smites in those slots |
| **Holy wrath's die scaling with the smite's** | rejected | a free, scaling rider on every blow against 43 % of the types would make the smite itself pointless against them; D&D's is a flat d8 |
| **Smite as a spell** (5e 2024) | rejected | Warlock's spells are the cast path's (mana, schools, ladders); a smite is a blow with the weapon, and its school would be none |

## 12. Implementation plan

Five issues, each a PR; rules first, then content. At most one at a time adds actions (P2, P4), one adds a status (P2). The full check list runs once, before each PR opens.

| Slice | What | Files (the hot ones: one line each) | Tests |
|---|---|---|---|
| **P1** Zeal and Smite | `resources.toml` Zeal; `data/smite.toml` (`content/smite.rs`: `SmiteRules`, `Content::unholy`, the tag check); the Smite talent reworked (`aim`, `unholy`, `grants_resource`, `resource`; `cooldown`/`degrees` gone), moved to level 1; `world/smite.rs` (Smite moved out of `class_ability.rs`; Presence to hit; the unholy die; `SmiteMode::Plain` only); Lay on hands onto zeal (`resource`, the cooldown fields gone); Avatar's `hands_cost_percent`; Spent and Drained deleted (or left: question 9); the sin hook in `shift_alignment`; the balance probe's paladin column; the wiki rows; spec and docs | `talent.rs` (`Smite`, `LayOnHands`), `class_ability.rs` (dispatch), `ability.rs::class_cooldown` (two arms gone), `world/alignment.rs` (one `if`), `balance_probe.rs` | `class_active_tests.rs`: a smite spends 3 and lands its dice with Presence to hit; the unholy die on a ghoul and not on a goblin; refused with nobody there spends nothing; unanswered for a Neutral paladin spends nothing; Lay on hands for 6, greyed under 6; Avatar's 3; zeal built by hit and struck, trickling by time, resting; a sin empties it; the priest at 40 holds it; a level-1 paladin holds it at 5; a save of a level-12 paladin loads with the pool; a replay |
| **P2** Branding and Wrathful smite | the two talents (12, 32), `ClassActive::BrandingSmite`/`WrathfulSmite`, `smite_mode` (`content/talent.rs`, `TalentMods::smite_modes`), the `branded` status, the fear check, the riders in `smite.rs`, action text, the menu rows, dumps, wiki | `content/class.rs` (`ClassActive` arms: `ALL`, `id`, `aimed`, `strikes`, `holy`), `action.rs` (parse by id, no new variant: `use:ID:DIR` resolves the active), `class_ability.rs` (two dispatch lines) | a branded orc sheds light the player sees in the dark and is not `unaware`-able; the brand lasts five minutes; Wrathful's check made and failed (fixed seeds), the flee, the turns; both spend on a hit that wounded, nothing on a miss; unanswered |
| **P3** Retribution | the talent (25), `retribution { cost }`, `Cue::Attacked { parried, hit }`, the branch in `second_blow` before the riposte's, `smite.rs::retribution` | `free_blow.rs` (the cue's field, one branch) | a hostile's hit answered when zeal is 4, not at 3; a parried attack ripostes and does not retribute; one free blow per attack; not for a Neutral paladin (no line); the blow's draws in order (a replay) |
| **P4** Holy wrath, Banishing smite, Avatar's wrath | the talents (40, 50), `holy_wrath { dice }` read in `resolve` beside `ambushed`, `ClassActive::BanishingSmite`, the unmaking (`creature_dies`) and the doubled dice, `avatar`'s wrath on every foe | `fight.rs::resolve` (one call), `content/class.rs`, `class_ability.rs` | Holy wrath's die on a ghoul, none on a goblin, both with Avatar; a summoned imp unmade and its kill paid; a draugr's doubled dice; a goblin's plain smite for 6 |
| **P5** Zealot and the wiki | the pickable; the Paladin section of the classes page; `TERMS.md`; the balance page's tuning note | `data/talents/zealot.toml` | `resource_talent_tests.rs`: two a blow taken; the prerequisite |

P1 first, alone (it moves the signature and retires two statuses). P2 and P3 may run in parallel after it (one adds a status and actions, the other a free blow). P4 after P2 (it shares `smite.rs`'s mode table). P5 any time after P1.

## 13. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| 1 | Zeal's tempo: built by `hit` and `struck` with a slow trickle (as proposed), or regenerating only (Ki's shape), or built only (Openings')? | **as proposed**: the fight builds it, time refills it slowly. The numbers (10, 1 a point, 60 s) are first guesses in data; the balance probe's column watches them |
| 2 | Smite on a hostile that is not evil (a bear, a fey of the courts): lands as on anyone (proposed), or does nothing and spends nothing? | **lands**. Reading the target's alignment leaks it; the unholy by tag is the only distinction |
| 3 | Lay on hands onto zeal (proposed), or onto mana as the old note said? | **zeal**: one pool, wrath against mercy |
| 4 | A sin empties zeal (§4.3)? | **yes**: one line, the moment the fiction asks for; it bars nothing |
| 5 | The priest's Smite at 40 grants Zeal (proposed), or #820 decides the priest's Smite first? | **grants**, so nothing breaks the day P1 lands; #820 owns the priest's table and may move it |
| 6 | Smite moves to level 1 (proposed), or stays at 12 with Branding smite taking level 1? | **level 1**: the signature is the smite; a brand before a smite is backwards |
| 7 | Bulwark and Holy radiance stay (proposed), or give way to smiting at 40 and 50? | **stay**: the audits passed them, the wall and the light are the paladin's, and the smites join them at their levels |
| 8 | Wrathful smite: a Willpower check for everyone (proposed), or Hound's bay's rule (a lower level flees, a higher never)? | **the check**: Presence matters to it, and a troll can be frightened by a saint |
| 9 | Spent and Drained: deleted in P1 (an index shift, covered by the release's format bump), or left unused until the bump? | **deleted**, if the integrator's next bump is near; else left until it |
| 10 | Retribution answers a hit only (proposed), or any attack (a miss too)? | **a hit**: the fantasy is the blow taken and returned; a miss is the riposte's cue when it was parried |
| 11 | A sixth mode later (the sworn foe, §11)? | **not now**: five ways to smite are plenty; revisit after the depths have their big foes |

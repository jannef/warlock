# Warlock — The monk renamed, a class of ki tricks

**Built; this file is history.** Current behaviour is in spec §8 "The adventurers' class tables, and the class actives" and `docs/dev/progression.md`.

Design document for the owner's review (2026-11-02), for #822. Written against `origin/main` at f3846261. Nothing here is implemented; §9 cuts it into issues. It builds on what is there: the class tables and their actives (`docs/dev/progression.md` "Class actives", `world/classes/class_ability.rs`), the alternative resources and the Ki pool (`docs/dev/resources.md`, `data/resources.toml`), the statuses that ease by actions and what Surprised does (`docs/dev/status-effects.md`, `docs/dev/weapon-abilities.md`, `world/combat/surprise.rs`), the free blows and their switches (`world/combat/free_blow.rs`), the unarmed skill and the Brawler and Staff Fighter styles (`data/weapon_skills.toml`), the weapon abilities those styles teach (`data/weapon_abilities/`), and the talent coverage policy (`docs/dev/progression.md`, rules 1 to 9 of #776). Every number is data and hidden in the game; the wiki states it.

**The owner's request** (2026-11-02, verbatim): *"Monk: create design issue to lean into ki tricks. Let's rename this to something else (Martial artist? idk)"*. The gaps report (#802) finds the monk's table full: this design reworks it rather than fills it.

## Contents

1. Summary
2. Research, tied to the decisions
3. What exists today
4. The name
5. Ki: the pool from level 1
6. The table reworked: the tricks
7. The rename: ids, saves, replays, data, wiki, spec
8. Rejected ideas
9. Slices
10. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **The name** | **Ascetic** (recommended; §4): Greek *askētēs*, "one who trains", an athlete's word before it was a hermit's; it says the vow of poverty and the inward turn, it is a word every English reader knows, and it ends the clash with the world's own monks (Fennmynster's drowned brothers, the ghost of a monk). Four more are weighed: Mystic, Pugilist, Wrestler, Martial artist |
| **Ki from level 1** | Martial arts (level 1) grants the pool, not Flurry (18). The pool grows with the character: `max = 4`, `max_from = { stat = "willpower", per_levels = 6, least = 3 }` (5 at level 1 with Willpower 12, 10 at level 25, 17 at 60, Inner calm +2), a point back every 18 s as today, resting too, granted full. §5 |
| **The table** | 1 Martial arts (bare hands 1d6, **grants Ki**), **Flurry** (moved from 18: a second bare-handed blow for 1 ki), Trained wind; 6 **Step of the wind** (Fleet's quicker steps, and for 1 ki your next action is quick: Light-footed, no time); 12 **Stunning strike** (1 ki and a blow's price: a bare-handed blow that stuns, flat-footed and slow, where it wounds); 18 **Patient defence** (1 ki and a wait: Poised until your next action, Dodge +4 and every blow you dodge or parry answered at once); 25 **Still mind** (kept, and gains **Stillness**: 2 ki, no time, the statuses that cloud the mind or hold the body end); 32 Purity of body (kept, shared); 40 **Diamond soul** (Deflection +2 kept, and a shot or throw that would strike your bare-handed self is plucked from the air for 1 ki, spent by itself); 50 Quivering palm (kept, 5 ki); 60 Perfect self (kept, and ki returns twice as fast). Fleet is folded into Step of the wind; Deflect missiles leaves the table (it stays a Finesse pick). §6 |
| **Who may use what** | the blows (Flurry, Stunning strike, Quivering palm) and the catch ask for empty hands, as today; Step of the wind, Patient defence, and Stillness work with a staff in hand too. The staff-fighting ascetic keeps its style's Trip and Staff sweep |
| **Hooks** | one new read each: `grants_resource` and `resource_rate` exist; `wind_step`, `stunning_strike`, `patient_defence`, `stillness`, `catch_missiles` are five `TalentEffect` kinds with one function each in `world/classes/ascetic.rs` (as the thief's live in `world/classes/thief.rs`); Stunning strike shares Quivering palm's blow (`palm_blow(dir, status, severity)`: one function, two callers). Two new statuses, **Stunned** and **Poised**; one new status field, **`flat_footed`**, which Surprised takes too so the one hook reads the field, not the id. No new system: ki is `resources` |
| **The rename** | class id `ascetic`, file `05-ascetic.toml` (the same number: the offered order and the path-order test stand); `ClassDef::was = ["monk"]`, resolved in the one lookup `Content::class(id)`, so old saves, replays, scenarios, and `--character dwarf,female,monk` keep working; the stored spec string stays as written (it keys streams), every word the player sees comes from the def. Content edits: Inner calm's `needs`, the youth chapter's `when` and text, two comments. The wiki's `classes.md#monk` anchors (12 links on 6 pages, TERMS.md) and the spec's and `docs/dev/`'s "monk" become "ascetic", except the world's monks. §7 |
| **Determinism, saves** | a ki trick draws only where it rolls a blow (the gameplay stream, as any blow); Stillness and Step draw nothing. Saves: `fleet` is dropped on load (`Talents::drop_unknown`, as #684's three); a loaded ascetic below 18 is granted ki by `World::grant_class` → `hold`, as a restored run gets what its talents grant. Replays of runs that fought bare-handed below level 18 hash differently from the first flurry; the Characters screen shows an old summary's "Monk" until that run is saved again |
| **Slices** | seven issues in three waves: S1 the rename (no rules), S2 the pool at 1 and Flurry's move (saves change, no actions); then S3 Step of the wind, S4 Stunning strike and Stunned, S5 Patient defence and Poised (each one action); then S6 Stillness, S7 Diamond soul's catch. At most two at a time add actions. §9 |

**Key choices, and why**

1. *The pool is the signature, so it comes at level 1.* A class "leaning into ki tricks" cannot wait until level 18 for its first point; D&D's monk has ki at 2 and ADOM's monk is a monk from the first kick. Martial arts (the level-1 passive the table already gives) takes the grant, and Flurry, the first trick, comes with it. The level-18 slot it leaves is where the first defensive trick goes.
2. *Every trick is a class active on the one registry, paid in ki, never a cooldown.* The coverage policy's rule for resources ("a cast that costs a pool has no cooldown besides; time refills it") and the owner's "time is the penalty" decide it. Quivering palm already lost its cooldown for 5 ki (#252); the new tricks follow.
3. *Tricks ride the statuses that already ease by actions.* Light-footed (next action at half price), Staggered's shape (`per_action`, `cost = 150`), Surprised's flat-footedness, and the Lurking pattern (a status that switches a free blow on in `free_blows_of`) give Step of the wind, Stunned, and Poised with one new field and two new entries in `status.toml`; no scheduler change.
4. *Passives stay where they are the body.* ADOM's monk is all passives and reads as a monk anyway; D&D's keeps Purity of body, Still mind, Perfect self beside the ki. The design keeps the four that define the body and mind (Martial arts, Still mind, Purity of body, Perfect self) and gives two of them a ki half, folds Fleet into a trick, and turns the two slots that were only numbers (Deflect missiles' Dodge, Diamond soul's Deflection) into one that is a number and a trick.
5. *The name says what the character is in this world's English.* The classes are plain English nouns (Barbarian, Bard, Priest, ...), so the new one is a plain English noun too, not an Old English coinage (the register memory: "no pure-Old-English naming for new things"); "Martial artist" is a 20th-century phrase that points east; "Monk" names the minster's brothers the player meets as ghosts. Ascetic is what the kit, the vow, and the discipline say.

## 2. Research, tied to the decisions

| Finding | Source | Used in |
|---|---|---|
| D&D 3.5 monk: flurry of blows, stunning fist (uses **per day**), evasion, still mind, ki strike, slow fall, purity of body, wholeness of body, diamond body, abundant step, diamond soul (spell resistance), quivering palm (once a week), timeless body, empty body, perfect self | SRD, "Monk" | §6: the names Still mind, Purity of body, Diamond soul, Quivering palm, Perfect self stay; per-day uses rejected (§8: no cooldowns on a pool) |
| D&D 5e (2014) monk: **ki points equal to the level**, back on a short rest; Flurry of Blows 1, Patient Defense 1 (Dodge), Step of the Wind 1 (Disengage or Dash), Stunning Strike 1 (a hit stuns), Deflect Missiles (reduce, 1 ki to throw back), Slow Fall, Stillness of Mind (end a charm or fear), Diamond Soul (1 ki to reroll a save), Empty Body 4, Perfect Self (4 ki back when empty as a fight begins) | Player's Handbook (2014), "Monk" | §5: a pool that grows with the level; §6: the five tricks at 1 ki each, Stunning strike, Patient defence, Step of the wind, Stillness; Perfect self's ki half |
| D&D 2024 renamed **ki to Focus Points**, raised the Martial Arts die (d6 to d12), let Flurry strike three times at 10, gave Patient Defense temporary hit points, and widened Deflect Missiles to melee | [Roll20, 2024 vs 2014 monk](https://pages.roll20.net/dnd/monk-class-2024-vs-2014), [Wargamer, all changes to the 2024 monk](https://www.wargamer.com/dnd/2024-monk), [RPGBOT, 2024 monk guide](https://rpgbot.net/updated-dnd-5-5-monk-class-guide/) | §4 and §10: "Focus" is the ranger's pool here, so the pool keeps "Ki" (or Mettle, question 2); the Martial arts die growing is Perfect self's d8 already |
| ADOM's monk fights with hands and feet, gains speed and Defence as it levels while lightly armoured and unburdened, and its class powers are passive: no pool, nothing to spend | ADOM Guidebook, "Monk" (the fan wiki and the guidebook mirror were unreachable at writing: paywalled and an expired certificate; stated at the level the author is sure of) | §6: Fleet's quicker steps kept as the passive half of Step of the wind; the body passives kept (choice 4) |
| DCSS's Monk is a **background**, not a path: Fighting 3, Dodging 3, a plain weapon at 3, a robe, an orb of light, a potion of ambrosia and one of enlightenment, and extra piety with the first god; nothing comes later | [crawl `jobs/monk.yaml`](https://raw.githubusercontent.com/crawl/crawl/master/crawl-ref/source/dat/jobs/monk.yaml), [`descript/backgrounds.txt`](https://raw.githubusercontent.com/crawl/crawl/master/crawl-ref/source/dat/descript/backgrounds.txt) | the lesson in reverse: a start with no path fades into its skills by the mid-game; Warlock's has a table of nine, so each level must bring a thing to do (§6) |
| Warlock's own thief rework (#684, `docs/design/surprise-and-sneak-attacks.md` §6.2): abilities that only replicate a trained skill go; the slots take identity actives on the pool | the approved design | §6: Deflect missiles (a Finesse pick handed out) leaves the table; the slots take tricks on the pool |
| The owner's rules: no cooldown beside a pool cost; a talent changes a cost in data, never adds a timer; capstones change a fight or a dungeon; no clones; information is a garnish | `docs/dev/progression.md` "Coverage policy", #776 | §6 throughout: every trick costs ki and time or ki alone; Quivering palm and Perfect self at 50 and 60 change fights; Stunning strike and Stunning fist are different hooks (§6.3) |

## 3. What exists today

| Part | Today |
|---|---|
| The class (`data/classes/05-monk.toml`) | "A disciple of a cloister's discipline, body and mind trained as one: quick, enduring, inward-turned." Dexterity 1d3, Toughness 1, Willpower 1d3, Presence −1; hp 5, mana 4, casting 50, experience 120; Balance, Initiative, Tumble `2d6+10` (known for Balance, Tumble), Athletics, Concentration, Escape artist, Listen `2d4+4`; optional Heal, Stealth, Swim; Unarmed 2, Staves 1; Brawler or Staff Fighter; kit a quarterstaff (if Staff Fighter) or leather gloves (if Brawler), a wool cloak, three apples, seven copper coins; bonus picks Finesse or Vigour at 10, 20, 30, 45, 55; rolls 2 |
| The table | 1 Martial arts (`unarmed_die 6`), Trained wind (stamina); 6 Fleet (`step_percent −10`, unarmoured and shieldless); 12 Deflect missiles (the Finesse pick, handed out: `missile_dodge 2`); 18 Flurry (`grants_resource ki`, `free_blow flurry`, `flurry_cost ki 1`); 25 Still mind (`checks willpower 2`, `no_penalty_to intelligence`, `no_penalty_to willpower`); 32 Purity of body (`disease_immune`; shared with the paladin at 18 and the priest at 32); 40 Diamond soul (`deflection 2`); 50 Quivering palm (`quivering_palm { status = "bleeding", severity = 3 }`, `resource = { id = "ki", cost = 5 }`); 60 Perfect self (`unarmed_die 8`, `weapon { role = "unarmed", threat = 1, crit = 1 }`) |
| Ki (`data/resources.toml`) | `max = 5`, `regenerates = 18` (a point every three base turns), resting too, granted full, never ends; `show_max` true (`Ki 5/5`). Inner calm (Finesse; class monk, level 18): `resource_max +2`, `resource_rate +50`. The affix of Stillness raises its maximum (`Mods::resource`). Spent in `World::flurry` (`free_blow.rs`: the second blow is decided only once its cost is spent; a weapon in hand, no flurry) and by `use_class` for Quivering palm (refused with a weapon in hand before the spend) |
| The actives' machinery | `ClassActive` (`content/class.rs`), `TalentEffect::active` maps an effect to one; `Ability::Class` in the one registry, the abilities menu's Class group, `use:ID` and `use:ID:DIR`; `may_use_class` and `Unavailable::Short(resource)` ("not enough ki"); `kind_of` prices an active as the action it is (`melee`, `wait`) for the statuses that block; the thief's three live in `world/classes/thief.rs` with one line each in `use_class` |
| Statuses that ease by actions | Staggered (`per_action`, `cost = 150`), Pinned (`blocks = ["step"]`), Light-footed (`cost = 50`, ends as that action is taken), Surprised (`per_action`, `cost = 150`; flat-footed and no free blows, both read **by id** in `World::target_against` and `World::free_blows_of`), Lurking (switches the gatekeeper free blow on: one line in `free_blows_of`) |
| What the styles teach | Brawler 4 Stunning fist (a critical with bare hands staggers), 7 Hip throw (4 stamina: a pace of knockback and Staggered); Staff Fighter 4 Trip (a critical pins), 7 Staff sweep (5 stamina: a blow at every foe beside) |
| Where "monk" is written | the class file; `inner_calm.toml` (`needs = { class = "monk" }`); `background/04-youth.toml` `the_cloister` (`when = { class = ["monk"] }`, "given to the cloister young"); comments in `resources.toml` and `03-priest.toml`; 12 core test files (`character.rs`, `content/scenario.rs`, `replay.rs`, `content/mod.rs`'s class-order list, `ki_tests.rs`, `class_active_tests.rs`, ...); the spec (§7 "Characters", §8 "Combat", "Blood and status", "Stats and burden", "Character": about 20 lines); `docs/dev/progression.md`, `resources.md`; the wiki's `classes.md#monk` anchor in `classes.md`, `combat.md`, `fighting-styles.md`, `magic-items.md`, `resources.md`, `surprise.md`, and TERMS.md. **The world's monks** stay monks: Fennmynster's (`dungeons.md`, `fennmynster.toml`, the sunken crypt vault) and the ghost of a monk (`creatures/ghost.toml`) |

**The gap**, in one line each: the pool arrives at 18, so for seventeen levels the class is a Fleet barbarian with gloves; two of nine slots are a number on a Defence category and nothing to do; the one trick before 50 is Flurry, which is automatic; and the class is named for the brothers whose ghosts it fights in the fen.

## 4. The name

The classes are plain English nouns a reader knows without a glossary (Barbarian, Bard, Priest, Druid, Paladin, Ranger, Thief, Wizard, Witch, Warrior, Aristocrat, Commoner, Craftsperson, Merchant). The register memory (owner, 2026-10-26) puts Old English on the oldest places and things, Middle English on organisations, and Victorian English on the world's contemporary concepts, and says "no pure-Old-English naming for new things"; a class name is the player's word for themselves, so it is a plain noun in that list's register. Five candidates:

| Name | Where the word is from | For | Against |
|---|---|---|---|
| **Ascetic** (recommended) | Greek *askētēs*, "one who trains, an athlete"; in English from the 1600s, Victorian prose's word for the self-denying | says the vow of poverty (the kit: a staff, bound hands, alms in copper, three apples), the discipline, and the inward turn; the training sense is the class's; one word, one meaning; no clash with the world's monks; "an ascetic" reads at once in the creation list | says nothing of fists on its own (nor does Monk); learned rather than homely |
| Mystic | Greek *mystikos*, through Victorian occultism | the ki side, the still mind | reads as a seer or a caster beside Witch and Druid; promises visions it does not give |
| Pugilist | Latin *pugil*, the prize ring's Victorian word | the fists, exactly; a period flavour | a boxer, nothing of the mind or the vow; faintly comic |
| Wrestler | Old English *wrǣstlere*, attested | the one truly Anglo-Saxon fighting word; plain | promises grappling the rules barely have (Hip throw at Brawler 7 and no more); old-register for a new thing |
| Martial artist | 20th century, from "martial art" (1900s), East Asian cinema | the owner's suggestion; says fighting | points east in an Anglo-Saxon march; two words in a list of one; "artist" reads oddly on a status screen ("Martial artist, level 12") |

Also weighed and dropped: Disciple and Mendicant (religious again), Adept (a caster's word here: Scroll adept), Pilgrim (a Pious rank), Hermit (the showcase's npc), Brawler (the style), Champion (the warrior's capstone), Fistfighter (a thug).

**Recommendation**: Ascetic. The description becomes: *"A wandering ascetic of the old discipline, body and breath trained as one: quick, enduring, inward-turned."* The youth chapter's line `the_cloister` becomes "You were given young to a house of the discipline, and spent your youth in silence, in hard work, and in learning to fall without hurting." (its id stays: ids are plain). The rest of the document says "ascetic" and reads the same under any name the owner picks; §7 gives the mechanics for whichever.

**Ki's name** (question 2): the owner's word is "ki", the data id is `ki` whatever the player reads, and D&D 2024's own rename (Focus) is taken by the ranger here. If the owner wants the pool in the register too, **Mettle** ("spirit, courage, stuff": Victorian English's word for exactly this; `Mettle 5/5` on the status bar) is the one that reads; "Breath" clashes with Stamina's words (Trained wind, "short of breath"). Default: keep Ki.

## 5. Ki: the pool from level 1

```toml
# The ascetic's ki (spec §8 "Resources"; the ki tricks design): granted full by Martial arts at
# level 1, a point back every three base turns, resting too; it never ends. Its maximum is 4,
# plus the Willpower modifier, plus a point per 6 character levels, at least 3; talents add on
# top (Inner calm), and so does the affix of Stillness.
[[resource]]
id = "ki"
name = "Ki"
short = "Ki"
max = 4
max_from = { stat = "willpower", per_levels = 6, least = 3 }
regenerates = 18
lacking = "your ki is spent"
```

| Rule | Decision | Why |
|---|---|---|
| Granted by | **Martial arts** (level 1): `effects = [{ kind = "unarmed_die", sides = 6 }, { kind = "grants_resource", resource = "ki" }]`; Flurry loses its grant | the pool is the signature (choice 1); the same two kinds the thief's level-1 Sneak attack uses |
| Maximum | `4 + Willpower modifier + level / 6`, at least 3: Willpower 12 at level 1 gives **5** (today's 5); level 25, Willpower 14, **10**; level 60, Willpower 16, **17**; Inner calm +2 on top | D&D's pool grows with the level; `max_from` is the field Stamina already uses (`{ stat = "toughness", per_levels = 10, least = 5 }`), so no code; Willpower is the ascetic's rolled stat (`willpower = "1d3"`) and the mind's |
| Pace | 18 s a point (three base turns), resting too, as today; Inner calm +50 %; Perfect self +100 % (§6); Stillness affix on the maximum as today | a fight of ten turns refills about three points, so the tricks are spent, not spammed; the bigger late pool means a longer opening burst, not a faster one |
| Start | full (no `start`: `starts_full`, `max_from` included) | as today |
| `show_max` | true: `Ki 5/5` | as today |
| Creatures | none | as every pool |
| The hash | a loaded run below level 18 gains a pool on load (`World::grant_class` → `hold`), so its hash changes once at load, as #560's stamina grant did; new runs hash as any run with the pool | — |

`Actor::resource_max` already sums the data's maximum, `max_from` at the holder's level, the talents' `resource_max`, and the affixes' (`docs/dev/resources.md` "Hooks"); nothing is added for the pool but the two data lines.

## 6. The table reworked: the tricks

### 6.1 The table

| Level | Today | After | Kind | Cost | Hands |
|---:|---|---|---|---|---|
| 1 | Martial arts | **Martial arts** reworked: bare hands and kicks 1d6, **and you have ki** | passive, grants | — | — |
| 1 | — | **Flurry** (from 18): a second bare-handed blow, a little wilder, for 1 ki | the free blow, as today | 1 ki a second blow | empty |
| 1 | Trained wind | kept | passive, grants | — | — |
| 6 | Fleet | **Step of the wind** (`step_of_the_wind`): Fleet's quicker steps (unarmoured, no shield), **and** for 1 ki your next action is quick (Light-footed); no time | passive + active (`use:step_of_the_wind`) | 1 ki, no time | any |
| 12 | Deflect missiles | **Stunning strike** (`stunning_strike`): a bare-handed blow that way; where it wounds, the foe is **Stunned** | active (`use:stunning_strike:E`) | 1 ki + a blow's price | empty |
| 18 | Flurry | **Patient defence** (`patient_defence`): a breath taken in the guard: **Poised** until your next action: Dodge +4, and a blow you dodge or parry is answered at once | active (`use:patient_defence`) | 1 ki + a wait's price | any |
| 25 | Still mind | **Still mind** kept (Willpower checks +2; nothing dulls Intelligence or Willpower), **and Stillness**: for 2 ki, what clouds the mind or holds the body ends at once | passive + active (`use:stillness`) | 2 ki, no time | any |
| 32 | Purity of body | kept (shared with the paladin and the priest) | passive | — | — |
| 40 | Diamond soul | **Diamond soul** kept (Deflection +2), **and** a shot or throw that would strike you is plucked from the air for 1 ki, spent by itself, while your hands are empty | passive + a spend by itself | 1 ki a missile | empty |
| 50 | Quivering palm | kept: a bare-handed blow that leaves a wounded foe bleeding badly, for 5 ki | active (`use:quivering_palm:E`) | 5 ki + a blow's price | empty |
| 60 | Perfect self | kept (bare hands 1d8, unarmed threat +1 and critical +1), **and ki returns twice as fast** | passive | — | — |

Bonus picks (Finesse or Vigour at 10, 20, 30, 45, 55), skills, kit, and the styles are unchanged. **Fleet** is folded into Step of the wind and its file deleted (`fleet` dropped on load; nothing else names it). **Deflect missiles** leaves the table and stays the Finesse pick it is (Dodge 1, Dexterity 15, level 12: an ascetic takes it with the bonus pick at 20 if they want it); a save holding it keeps it (nothing is taken away). The table names eleven ids, three at level 1 and one at each of the other eight levels: `martial_arts`, `flurry`, `trained_wind`, `step_of_the_wind`, `stunning_strike`, `patient_defence`, `still_mind`, `purity_of_body`, `diamond_soul`, `quivering_palm`, `perfect_self`; two of them (`trained_wind`, `purity_of_body`) are shared with other tables, and the three new ones (`step_of_the_wind`, `stunning_strike`, `patient_defence`) are class-tree talents, `granted`, like the rest.

### 6.2 The tricks, each on its hook

| Trick | Data (`data/talents/`) | Rule | Hook (one read) |
|---|---|---|---|
| **Step of the wind** | `effects = [{ kind = "step_percent", value = -10, unarmoured = true, no_shield = true }, { kind = "wind_step", resource = "ki", cost = 1, status = "light_footed" }]`, `systems = ["speed", "resources"]` | `use:step_of_the_wind`: refused, no time, with no ki ("your ki is spent": the menu greys it); else 1 ki is spent and the ascetic is **Light-footed** (the status Hit and run gives: `cost = 50`, `per_action`, `max = 1`, `refresh`): whatever they do next, a step out of reach, a blow, a shot, costs half. **No time**: the trick is the breath between actions, and the ki is its price (the policy: time or a pool). A second use before acting only refreshes the one degree (one half-price action, however many points spent: the menu says so, "You are already light on your feet.", and spends nothing) | `World::wind_step` (`world/classes/ascetic.rs`), from `use_class`; `TalentMods::wind_step: Option<WindStep>`; `afflict(player, status, 1)` |
| **Stunning strike** | `effects = [{ kind = "stunning_strike", status = "stunned", severity = 1 }]`, `resource = { id = "ki", cost = 1 }`, `systems = ["kicks", "status", "resources"]` | `use:stunning_strike:E`: refused, no time, with a weapon in hand ("Your hands are not free.") or nobody hostile that way, before the spend; else 1 ki, a bare-handed blow at the blow's price (`attack_price`, the unarmed skill's and the style's speed counted), and a hit that does damage leaves the foe **Stunned** (below). The blow is an ordinary unarmed blow: the brawler's Stunning fist may fire on its critical too (a stagger on top of a stun: both `per_action`, they multiply as Staggered and Surprised do) | `World::stunning_strike(dir)` = `palm_blow(dir, status, severity)`, the body of today's `quivering_palm` with the status passed in: **one function, two callers** |
| **Patient defence** | `effects = [{ kind = "patient_defence", status = "poised" }]`, `resource = { id = "ki", cost = 1 }`, `systems = ["defence", "free_blows", "resources"]` | `use:patient_defence`: 1 ki and a wait's price (`price_of(Wait)`), then **Poised** (below) until the ascetic's next action: the creatures' turns in between meet Dodge +4, and every melee blow they dodge or parry is answered with a blow at once (the riposte free blow, `Cue::Dodged`/`Parried`, switched on by the status as Lurking switches the gatekeeper's on). Refused, no time, with no ki; never refused for "nobody near": a breath taken alone is the player's to waste | `World::patient_defence`; `World::free_blows_of`: `riposte` on while `Effects::has(poised)` (one line beside `lurking`'s); the Dodge is the status's own `combat = { dodge = 4 }`, read where every status's `combat` is |
| **Stillness** | on Still mind: `effects` gain `{ kind = "stillness", resource = "ki", cost = 2, statuses = ["held", "pinned", "staggered", "stunned", "surprised", "transfixed", "frightened", "drowsy", "webbed"] }`; `systems` gain `"resources"` | `use:stillness`: refused, no time, when the ascetic suffers none of the listed statuses ("Your mind is already still.") or has less than 2 ki; else 2 ki and every listed status they suffer ends at once (`World::cure` by id: `Effects::remove` with its `end` words), **no time**: the point is to act *this* turn, held fast or stunned. The list is data: a status the content lacks is a load error (as `second_wind`'s `cooldown` is checked) | `World::stillness`; `TalentMods::stillness: Option<Stillness>` |
| **Diamond soul's catch** | `effects = [{ kind = "deflection", value = 2 }, { kind = "catch_missiles", resource = "ki", cost = 1 }]`, `systems = ["defence", "ranged", "resources"]` | a shot or a throw (a creature's, or a trap's missile: `trap_strikes`) that **hits** the ascetic while their hands are empty and they hold 1 ki: the point is spent by itself (as Rapid shot spends focus on a shot) and the hit is a miss: "You pluck the arrow from the air." The missile falls on the ascetic's cell as a dodged one does (no catching into the pack: the stack rules stay as they are). Spells never (a bolt is not plucked); a burst never (nothing to catch). With no ki, or a weapon or shield in hand, the hit stands | `World::caught_missile(target, events) -> bool` in `resolve` after the strike, for `AttackKind::Missile`, and in `trap_strikes`: two callers, one function |
| **Perfect self's pace** | `effects` gain `{ kind = "resource_rate", resource = "ki", value = 100 }`; `systems` gain `"resources"` | ki returns twice as fast (9 s a point; with Inner calm 7.2 s) | `World::resource_rate` (exists) |
| **Flurry** | `effects = [{ kind = "free_blow", trigger = "flurry" }, { kind = "flurry_cost", resource = "ki", cost = 1 }]` (the grant moved to Martial arts); `systems = ["free_blows", "kicks", "resources"]` | as today | `World::flurry` (exists) |
| **Quivering palm** | unchanged: `resource = { id = "ki", cost = 5 }` | as today; "for five ki" in its description now that the pool is larger than five | exists |

**The new statuses** (`data/status.toml`, placed beside Staggered and Lurking):

```toml
# Stunned (the ascetic's Stunning strike): reeling from a blow. Flat-footed (Dexterity and Dodge
# count nothing toward Defence), no free blows, and the next action costs half again as much;
# that action ends it. A second stun before it only keeps it.
[[status]]
id = "stunned"
name = "Stunned"
seen = "stunned"
max = 1
stacking = "refresh"
per_action = true
cost = 150
flat_footed = true
start = "You are stunned."
end = "Your head clears."

# Poised (the ascetic's Patient defence): a breath taken in the guard, until the next action.
# Dodge +4, and a blow dodged or parried is answered at once (the riposte free blow).
[[status]]
id = "poised"
name = "Poised"
seen = "poised"
max = 1
stacking = "refresh"
per_action = true
combat = { dodge = 4 }
start = "You settle into your guard."
```

**`flat_footed`**, the one new status field (default false): while it lasts, Dexterity and Dodge count nothing toward the sufferer's Defence and it gets no free blows. Surprised takes `flat_footed = true` in data, and `World::target_against` and `World::free_blows_of` read `Effects::flat_footed()` where they read the id `surprised` today: the hook moves from an id to a field, the behaviour of Surprised is unchanged, and Stunned is the field's second user. A Stunned foe is also, for an attacker with sneak dice, a target "unaware or Surprised" (`World::sneaks`): the design proposes **yes**, the dice land on a Stunned foe too (the thief's dice are "a target denied its Dexterity", which is what flat-footed means; `sneaks` asks the field). A creature Stunned has no sneak dice of its own to worry about.

### 6.3 Coverage, clones, and the rules of #776

| Check | Answer |
|---|---|
| New system? | none: ki is the `resources` system (held to the bar since R5); the tricks carry `resources` and the tags of the rules they read (`speed`, `kicks`, `status`, `defence`, `free_blows`, `ranged`) |
| A talent must earn its place | Step of the wind: on-demand quickness, a new choice every turn; Stunning strike: a stun on demand, not on a critical; Patient defence: a counter-turn the game has nowhere else; Stillness: the ascetic breaks a hold or a stun on its own turn; the catch: a hit undone; none is information, a niche percent, or a rare-room effect |
| No clones (rule 3) | Stunning strike beside Brawler's **Stunning fist**: different hooks (a class active spending ki against a weapon ability's trigger on a critical) and different statuses (Stunned is flat-footed; Staggered only slows); a brawler-ascetic may have both, as a thief has Sneak attack beside Knife in the dark. Step of the wind beside **Hit and run**: the same status, the skirmisher's on a throw's hit by trigger, the ascetic's for ki by choice (and Fleet's passive half is its own). Patient defence beside the **defensive stance** (Dodge +2, Parry +1, to-hit −2, free, lasting): a burst of +4 and ripostes for a turn and a point. The catch beside **Deflect missiles** (Dodge +2 against missiles, pickable): a hit undone against a chance of one missing; the design's reading of #790's "the stronger class version beside the pickable" (Improved sneak attack beside Ambusher). The alike check compares talents' effect kinds: none of the new kinds matches another's |
| One effect, two ids, chained (rule 4) | no chain: no new talent needs another |
| Capstones (rule 1) | Diamond soul 40, Quivering palm 50, Perfect self 60 change a fight |
| Flaws, outdoors, hidden marks, information, shop favour (rules 5 to 9) | no flaw, nothing outdoors-only, no marks, no readout, no favour |
| Inner calm | unchanged in effect; `needs = { class = "ascetic", level = 18 }` |

### 6.4 What the player sees

| Where | What |
|---|---|
| The abilities menu's Class group | Step of the wind, Stunning strike, Patient defence, Stillness, Quivering palm, in table order, each with "not enough ki" (`lacking`: "your ki is spent") greyed, Stillness greyed "your mind is already still" with nothing to end, the blows "your hands are not free" with a weapon |
| The status bar | `Ki 5/5` after MP as today; Stunned, Poised, Light-footed as any status |
| Messages | "You stun the kobold." / "The kobold reels." (a creature seen; its `seen` word in look mode, "stunned"); "You settle into your guard." then the ripostes' own lines; "You still your mind." then each status's `end` line; "You pluck the arrow from the air."; Step of the wind says nothing but Light-footed's own silence (it has no `start`): the next action's quickness is felt, not told |
| The character screen | the pool's row under the stance as today; no number of the pace |
| Dumps | the header's `ki 5/5`; `abilities` rows `Step of the wind (class): nothing`, `Stunning strike (class): a direction`; `actors`'s `resources ki 5/5 (progress)`; the `talents` layer's `(class)` marks |
| The wiki | `classes.md` (the class, the table, its description), `resources.md` (ki's maximum, pace, the tricks' costs), `abilities.md` (the five actives), `status-effects.md` (Stunned, Poised rows, generated), `surprise.md` (flat-footed: Surprised and Stunned), `combat.md` (free blows: the Poised riposte), TERMS.md; every number exact |

## 7. The rename: ids, saves, replays, data, wiki, spec

| Piece | Decision |
|---|---|
| The id | `ascetic`; `data/classes/05-ascetic.toml` (the number is the offered order: the class stays fifth, `content::tests::classes_in_path_order` changes one string) |
| Old ids | `ClassDef::was: Vec<String>` (data `was = ["monk"]`, default empty): `Content::class(id)` finds a class by `id` **or by a `was` entry**; it is the one lookup every caller uses (`kit.rs`, `perception.rs` ×2, `character.rs`, `world/character/creation.rs`, `world/character/talent.rs`, `world/classes/class.rs` ×3, `content/talent.rs` ×2), so saves, replays, scenario files, the golden manifest, and `--character dwarf,female,monk` keep working with no other change. Load check: a `was` id is no class's `id` and no other class's `was` (a content error naming both files). The README of `data/classes/` documents the field |
| The stored spec string | **left as written** (`Character.spec.class` in saves, `character.class` in replays): it keys the character's streams (`character/kit/dwarf,female,monk`, `character/talents/...`, the name draws), so rewriting it would change an old replay's creation draws. Every word the player sees comes from the def through the lookup (the character screen, the summary, the dumps' `stats` layer print `ClassDef::name`), so a loaded monk reads "Ascetic" everywhere; `Display` of the spec is the key, not a word. A new run started by `--character dwarf,female,monk` records `monk` too and is the same character as `dwarf,female,ascetic` **only by lookup**: its streams differ, as a different spec string's do. The wiki's and CLAUDE.md's examples say `ascetic` |
| The saves list | `saves/<id>.toml` (`save::Summary`) holds the class **name**: an old summary reads "Monk" on the Characters screen until that run is saved again (its next autosave rewrites it). Accepted; no migration |
| Talents | `inner_calm.toml`: `needs = { class = "ascetic", level = 18 }`; the class tree's granted talents are unchanged by the rename (the table names them) |
| The background | `04-youth.toml` `the_cloister`: `when = { class = ["ascetic"] }` and the text of §4; the id stays |
| Comments | `resources.toml` ("The ascetic's ki"), `03-priest.toml` ("the ascetic's too") |
| Tests | the twelve files that spell `monk` (§3); `content/mod.rs`'s class list; a new test: `a_save_naming_the_old_class_id_loads_as_the_class` (`class_tests.rs`), and `content::class::tests` for the `was` check |
| The spec | the ~20 lines (§7 "Characters": casting 50, experience 120, hp 5, the skills, the styles; §8 "Combat": the free blows, the experience factors; "Blood and status": Resources (Ki, Inner calm, Trained wind's row); "Stats and burden": the Discipline note; "Character": the class tables, the actives, the bonus picks, Initiative); **the world's monks stay** |
| `docs/dev/` | `progression.md` (Class actives, the trees so far), `resources.md` (Ki), `character-creation.md` where it names the class; the new tricks' notes go in `progression.md`'s hook table and `resources.md`'s Ki entry, as today's do |
| The wiki | the anchor `classes.md#monk` is generated from the id (`anchored(Term::Class, &c.id, &c.name)` in `xtask/src/wiki/character.rs`) and becomes `classes.md#ascetic`: 12 links on 6 pages and TERMS.md's entry change, and `cargo xtask wiki --check-links` fails on any missed; prose "monk" → "ascetic" on those pages; `dungeons.md`'s monks of Fennmynster stay |
| The fingerprint | unchanged: classes are found by string, not by index; the content fingerprint has no class group |
| Reversibility | a later rename adds one entry to `was`; nothing else |

## 8. Rejected ideas

| Idea | Why not |
|---|---|
| **Wholeness of body** (D&D: hit points back for ki) | Second wind (Vigour) and Veteran's wind (warrior 12) are "hit points back at once"; a ki heal makes ki a mana, and the policy's "an effect another ability already gives". The ascetic's sustain is Purity of body, Still mind, and not being hit |
| **Abundant step**, **Empty body** (a short teleport; invisibility) | no teleport or invisibility rule exists for the player; a trick is not the place to found one |
| **Slow fall** | pits are Light step's (Finesse: `pit_proof` at rank 2); a second source is a clone |
| **Diamond soul's reroll** (1 ki to roll a failed save again) | checks are rolled at many sites (diseases, searching, locks, traps, holds, the initiative check); a reroll hook at each is the opposite of one read. Stillness ends what a failed check brought instead |
| **Timeless body**, **Tongue of the sun and moon** | ageing and tongues are not rules |
| **Per-day or cooldown uses** (D&D 3.5's stunning fist, quivering palm once a week) | the owner's rule: a pool cost has no cooldown beside it; time refills the pool |
| **Ki for mana**, or the reverse | the pools stay separate (the progression design §4.18: "a monk's mana stays theirs, separate"); Inspiration is the one place a pool touches mana, by decision |
| **Ki on creatures** | no creature holds a pool; a creature that fights like an ascetic gets its tricks as `on_hit` statuses and weapon abilities, as the roster does today |
| **Flurry striking three times at level 10 or later** (D&D 2024's Heightened Focus) | the second blow is already a free blow for ki; a third is more of the same and a bigger damage multiplier than any other class gets |
| **Catching the missile into the pack** | the stack and pile rules are untouched; the missile falls as a dodged one does |
| **Keeping "Monk"** | the player meets monks: Fennmynster's brothers, the ghost of a monk, a cloister's ghosts in the sunken crypt; a class named for them confuses what the word means in this world |
| **"Martial artist"** | §4: a 20th-century phrase that points east, two words in a list of one |
| **An Old English class name** (*cempa*, *wrǣstlere*) | the classes are plain English nouns, and the register memory forbids pure Old English on new things |
| **Renaming the pool to "Breath"** | Stamina owns the breath words (Trained wind, "you are short of breath") |

## 9. Slices

| Slice | What | Adds | Depends on |
|---|---|---|---|
| **S1** | **The rename**: `05-ascetic.toml`, `ClassDef::was` and its check, the content edits (§7), the tests, the spec, `docs/dev/`, the wiki's anchors and prose, TERMS.md; `cargo xtask wiki --check-links` green | no rules, no saved state, no actions | — |
| **S2** | **The pool at 1**: Martial arts grants ki, Flurry moves to 1 and loses its grant, `max_from` on ki, Perfect self's `resource_rate`, Quivering palm's words; `ki_tests.rs` (a level-1 ascetic flurries; the maximum by level and Willpower; a loaded save below 18 holds ki after load); the balance probe re-run for the class (`docs/balance.md`) | saved state (the pool held from 1); no actions | S1 |
| **S3** | **Step of the wind**: the talent (Fleet folded, `fleet` deleted and dropped on load), `wind_step`, `use:step_of_the_wind`, the menu row, the wiki | one action | S2 |
| **S4** | **Stunning strike** and **Stunned**: the status, the `flat_footed` field (Surprised reads it), `palm_blow` shared with Quivering palm, `sneaks` asking the field, the menu row, the wiki (`status-effects.md`, `surprise.md`) | one action, one status | S2 |
| **S5** | **Patient defence** and **Poised**: the status, the riposte switch in `free_blows_of`, the menu row, the wiki | one action, one status | S2 |
| **S6** | **Stillness** on Still mind: the effect, its status list checked at load, `use:stillness`, the wiki | one action | S4 (Stunned is in its list) |
| **S7** | **Diamond soul's catch**: `catch_missiles`, `caught_missile` in `resolve` and `trap_strikes`, the wiki | no action | S2 |

Three waves: S1 and S2 in order; then S3, S4, S5 (each one action: at most two at once, the integrator's rule); then S6 and S7. Each slice updates `docs/dev/progression.md`'s hook table, `docs/dev/resources.md`'s Ki entry, the spec's "Class abilities" and "Resources", and the wiki with it, as CLAUDE.md asks. Tests: `world/classes/ascetic_tests.rs` (the tricks: refusals before the spend, the spend, the statuses' course, the riposte, the catch on a shot and a trap missile, a replay of each), `class_active_tests.rs` (the table by level), `content/class.rs` (`was`). No golden: nothing here is only the GPU's.

## 10. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| 1 | The class's name | **Ascetic** (§4); the owner's pick decides the id and the description's wording, nothing else in the design |
| 2 | The pool's name | **keep Ki** (the owner's word; the id is `ki` either way); Mettle if the owner wants the register |
| 3 | Fleet: folded into Step of the wind, or cut | **folded**: the quicker steps stay the class's, and a slot holds a passive and a trick as Still mind and Diamond soul do |
| 4 | Deflect missiles: off the table, or kept at 12 with Stunning strike elsewhere | **off the table**: it stays a Finesse pick an ascetic may take at 20, and the catch at 40 is the class's own answer to missiles |
| 5 | Stunned flat-footed through a `flat_footed` field Surprised shares, or Staggered's shape only (slow, not flat-footed) | **the field**: one hook reads it, Surprised is unchanged, and a stun that opens the foe to the flurry is the trick's point |
| 6 | The sneak dice landing on a Stunned foe | **yes**: flat-footed is "denied its Dexterity", the dice's door; a thief beside an ascetic is a rare party anyway |
| 7 | Stillness as Still mind's active, or a slot of its own | **Still mind's**: the slot's passive is the mind's stillness already; the active is its other half |
| 8 | Step of the wind's price: no time, or a wait's price | **no time**: with a wait's price the next action's half is a wash; the ki is the price |
| 9 | The pool's numbers (`max = 4`, Willpower, a point per 6 levels, at least 3) | **as proposed**: 5 at level 1 as today, 17 at 60; the balance probe row in S2 watches it |
| 10 | The stored spec string left as written, or rewritten to the new id on load | **left as written**: it keys streams; every word shown comes from the def |
| 11 | The catch on trap missiles too | **yes**: a dart trap's dart is a missile; the one function has two callers either way |
| 12 | Stunning strike and Quivering palm with a staff in hand | **no**: the ki runs through the hands, as today's palm; the staff ascetic has Trip and Staff sweep, and Step, Patient defence, and Stillness with any grip |

# Warlock — The Berserker: the Barbarian renamed, a class built on rage

Design document for the owner's review (2026-11-02), for #818. Written against `origin/main` at f3846261 (Encounters E5, #815). Nothing here is implemented; §12 cuts it into issues. It builds on what is there: the Rage resource and the Raging status (`docs/dev/resources.md`, `world/rage.rs`), the berserker stance (`world/berserker.rs`, spec §8 "Stances"), class tables and class actives (`docs/dev/progression.md` "Class actives", `content/class.rs`), statuses (`docs/dev/status-effects.md`), hearing and noise (`docs/dev/hearing.md`), checks (`check.rs`), skills (`docs/dev/skills.md`), and the talent coverage policy (`docs/dev/progression.md`, rules 1–9 of #776, which bind every row below). Every number is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (2026-11-02, verbatim): *"barbarian: create a design issue to lean more on the rage as class fantasy. I think we should rename this to Berserker"*

The gaps report (`docs/design/class-ability-gaps.md`, draft PR #802) finds the barbarian's table full (10 abilities, 0 gaps) on the cadence 1, 6, 12, 18, 25, 32, 40, 50, 60. So this is not a gap-filling design: it asks of every row whether it serves the fantasy the owner named, and of the pool whether a level-1 berserker feels the rage from the first fight. Today it does not: rage is a twelve-turn timer that nothing feeds until level 40, and of the nine rows only 1, 40, 50, and 60 read it.

## Contents

1. Summary
2. The owner's request, and what exists today
3. Research: rage in other games, tied to decisions
4. The pool reworked: fighting feeds the rage
5. The level table, row by row (stay, change, go)
6. The new abilities, and the `while` condition
7. The rename: class, stance, ids, saves, replays, data, docs
8. What the player sees
9. Determinism, saves, replays
10. Talents and coverage (the policy's answer)
11. Rejected ideas
12. Implementation plan
13. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **The class** | `barbarian` → **`berserker`**, "Berserker"; the file `data/classes/01-berserker.toml`; stats, hit points, skills, kit, picks unchanged. A former id is remembered in data (`was = ["barbarian"]`): saves and replays naming it load as berserkers. §7 |
| **The pool** (`data/resources.toml`) | Rage keeps its shape (20 at most, 12 granted, a point lost a base turn, ends at zero, Raging while it lasts) and gains **builds from level 1**: a blow of yours that hits +1, a blow that hits you +1, a kill +3. A rage now lasts as long as the fight does and burns out when it is over (DCSS and D&D 5e, §3); nothing new in code: `builds` exist on resources. Exhausted after it is **5 minutes** (300 s; today 10). §4 |
| **The table** | 1 **Rage** (fed by fighting) + Trained wind; 6 Thick hide (stays); 12 **Headlong** (new: raging, your steps cost a fifth less in no body armour; replaces Fast movement); 18 Brutal critical (stays); 25 **War cry** (new active: a roar, for 4 rage and a wait, that frightens the hostiles who hear and see you unless they keep their nerve, the first use of the **Intimidate** skill); 32 Mighty thews (stays); 40 **Blood frenzy** (reworked: raging, your melee attacks cost a fifth less, and a kill feeds 5 rage as today); 50 Mighty rage (stays) and **Hardened** (moved from 25: it pays only past level 50 anyway); 60 Unstoppable (stays). Six of nine rows read the rage (1, 12, 25, 40, 50, 60), three are the body (6, 18, 32). §5 |
| **One mechanism** | `while = "raging"` on any talent effect: it counts only while the holder suffers that status (a status id, checked at load). Summed in the one place talents are summed (`Actor::talent_mods`, which knows the actor's statuses), so every hook stays one read. Headlong and Blood frenzy use it; Mighty rage, Unstoppable, Battle trance, Wild guard keep their own kinds. §6.1 |
| **War cry** | `use:war_cry`, aim nothing, 4 rage (the class active's `resource` cost: greyed "not enough rage", so only a raging berserker roars), a wait's time, as loud as a howl (`Act::Howl`: it is heard across the floor and wakes dens, which is its price). Every hostile that hears it and sees you, and has a heart to quail (`Content::has_vitals`, the surprise design's flag), rolls Willpower against **10 + your Strength modifier + Intimidate**; failed, it is **Frightened** three degrees (it runs from you three turns). Intimidate gains its first use: a `benefit` (+8 at 100, +12 at 200) and marks on every cry. §6.3 |
| **The stance** | `berserker` → **`frenzy`**, "Frenzy": "You fight in a frenzy." / "Only rage brings the frenzy." / "Your rage holds you in the frenzy." The word *berserker* then means one thing in the game, the class, as *thief* and *monk* do; the stance's numbers are unchanged. The enum variant loads the old name (`#[serde(alias)]`), Wild guard says `stance = "frenzy"`. Alternative (open question 1): keep the stance "Berserker" and change nothing. §7.2 |
| Saves and replays | no new state. Old saves and replays load (the former id); a save is written back with the live id. Replays whose berserker fights while raging hash differently from the first blow that lands (the pool's value moves); War cry draws two gameplay draws a hearer; Intimidate's marks are the training stream's. §9 |
| Slices | five issues: S1 the class rename (data, code names, docs, wiki; no rule changes), S2 the stance rename, S3 the pool (builds, Exhausted, Hardened's row), S4 the `while` condition with Headlong and Blood frenzy, S5 War cry and Intimidate. S1–S2 and S3–S5 are independent; at most one adds an action (S5). §12 |

**Key choices, and why**

1. *The rage is a fire, and fighting is its fuel.* Today the pool is a timer; the one thing that feeds it comes at level 40. Every game studied (§3) ties the rage's life to the fight: D&D 5e ends it the turn you neither attacked nor were hit, DCSS extends it on kills and ends it in a few turns otherwise, Path of Exile pauses its decay while you are hit and builds it on hits, Diablo III builds fury by blows and bleeds it out of combat. The data already has the events (`hit`, `struck`, `kill` are `Build`s): three numbers on the resource make a rage last the fight and no longer, from level 1.
2. *The rows that stay are the ones ADOM and D&D agree on.* ADOM's barbarian gets +3 Strength and Toughness at 32 (our Mighty thews, 32), hit points per level at 25 (Hardened), and binds its 40 and 50 to the berserk tactic (our 50 and 60 are rage-bound already). D&D's Brutal Critical and Unarmored Defense are the two-handed critical and Thick hide. Those stay. What goes is what neither tradition makes the barbarian's: a flat movement bonus that reads nothing of rage (ADOM's level-12 movement power becomes Headlong, the same idea while raging).
3. *One condition, not a kind a row.* Mighty rage and Unstoppable each needed a kind of their own to say "while raging"; the design adds the condition once (`while`), so Headlong is `step_percent` with it, Blood frenzy's speed is an attack-price percent with it, and a later buff-bound talent (Blessed, Inspired) costs no new mechanism. The alike check (#790) treats the condition as part of the effect, so a conditional effect is never a clone of a plain one.
4. *An active beyond the on-switch, paid in rage.* The class's only button is Rage itself; the thief spends openings on Low blow, the monk ki on Quivering palm, the ranger focus on Volley. War cry spends the rage that is also the rage's duration, so it is a real trade (four turns of fury for a crowd scattered), and it gives Intimidate, a skill the barbarian trains and nothing reads, its first use. D&D 5e's Berserker path (Intimidating Presence), Pathfinder's rage powers (Terrifying howl, Intimidating glare), Diablo II's Howl and War Cry, Path of Exile's War Bringer all make the roar the berserker's second move.
5. *Stay inside the policy.* No new system (`resources`, `stances`, `status`, `hearing`, `skills` exist); no clone (Hound's bay is a horoscope gift of another kind, kept whole by the owner's rule); no information-only ability (Mindless rage was dropped for exactly that, §11); nothing that pays in gold or XP; a capstone unchanged.
6. *Rename cheaply, and once.* Saves and replays carry the class id as a string. A former id in the class's own data and one resolving lookup (`Content::class`) load everything written so far; the file, the comments, the docs, and the wiki are a search-and-replace with a review of each line's sense.

## 2. The owner's request, and what exists today

| Part | Today |
|---|---|
| The class | `data/classes/01-barbarian.toml`: "A warrior of the wild lands, fueled by fury: mighty and hardy, with no patience for books or manners." St `1d3+1`, Dx +1, To `1d3`, In `−1d2`, Pr −1; 8 hit points, 1 mana, casting 25, experience 90 %; Athletics and Survival known for, Initiative, Intimidate, Listen, Swim a little; optional Handle animal, Spot; Unarmed 1 and Axes 2 or Great blades 2; Great Weapon 1; a great axe or great sword, a fur cloak, a bone amulet (perhaps magic), dried meat, 4 gold; bonus picks from Might or Vigour at 10, 20, 30, 45, 55 |
| The table | 1 Rage (active) and Trained wind; 6 Thick hide (natural armour +2 in no body armour); 12 Fast movement (steps 10 % cheaper, unarmoured and Unburdened); 18 Brutal critical (critical multiplier +1 with a two-handed weapon); 25 Hardened (each level past `hp_level`, 50, adds the level's hit points again and 3 more: **it pays nothing from 25 to 50**, which `docs/design/combat-progression.md` §9.4 flagged when `hp_level` rose from 20); 32 Mighty thews (St +2, To +2 for good); 40 Blood frenzy (a kill builds 5 rage); 50 Mighty rage (Raging's raises doubled, 10 more rage at most, Exhausted eases twice as fast); 60 Unstoppable (bleeding and poison tick nothing while raging; a blow that would take the last hit point leaves it for 10 rage) |
| The pool | `data/resources.toml`: Rage, `max = 20`, `start = 12`, `decays = 6` (a point a base turn, resting too), `ends_at_zero`, `status = "raging"`, **no `builds`**. Granted by the Rage ability (a wait's time, `world/rage.rs`); Blood frenzy's `resource_build` on `kill` is the one thing that feeds it, at level 40. A rage lasts twelve turns however the fight goes; a long fight outlives it |
| Raging | `status.toml`: St +2, To +2, In −2 (`penalty` positive), `blocks = ["read", "study", "cast"]` (no reading, no mana), `follows = "exhausted"`; Exhausted St −1, `eases = 600` (ten minutes), and no rage while either lasts (`ability.rs::class_cooldown`) |
| The stance | `CombatStance::Berserker` (`stances.toml`: to-hit +2, damage +4, Dodge −6, fleeing steps 150 %, marks 125 %), never chosen: `berserker::settle` puts an actor raging with its own Rage in it and holds it there, and its end leaves them aggressive; "You fight as a berserker.", "Only rage makes a berserker.", "Your rage holds you in the berserker's stance." (#680). Wild guard (Might, barbarian 6) halves its Dodge penalty |
| The picks that read rage | Battle trance (Might, barbarian 6: rage starts 6 higher, decays at 67 %), Wild guard (Might, barbarian 6); the affix of Fury (`resource_max` for rage) |
| Builds | `Build`: `kill`, `hit`, `critical`, `struck`, `sneak_attack`, `sneak_kill`, raised at one place each in `fight.rs::resolve` and `dies`; a resource's `builds` and a talent's `resource_build` both feed through `World::gain_resource`. Focus, Openings, Ki use them; Rage does not |
| Intimidate | `skills.toml`: `kind = "action"`, no `benefit`, no `mark`, **read nowhere**; the barbarian trains it `2d4+4`, the half-orc `1d6+4` |
| Fear | `frightened` (`flees = true`, `max = 3`, `eases = 6`): a creature suffering it runs from the one who gave it; Hound's bay (the Barghest sign's gift, `world/sign_picks.rs`) gives it to every hostile of a lower level that hears the howl; eleven creature types give it `on_hit`. On the player it does nothing but its message (`flees` is read by the AI only) |
| Class actives | `ClassActive` (Rage, Veteran's wind, Whirlwind, Lay on hands, Smite, Holy radiance, Quivering palm, Lurk, Low blow, Master thief, Inspire, Consecrate, Knock, Volley); a talent's `resource = { id, cost }` is its cost, `Unavailable::Short` greys it, `use_class` spends after the refusals; no key of its own (the abilities menu, `a`) |
| Where the name lives | the class file and `README.toml`; `needs = { class = "barbarian" }` in `battle_trance.toml` and `wild_guard.toml`; `when = { class = [...] }` in `background/04-youth.toml` and `07-leaving.toml`; comments in `experience.toml`, `resources.toml`, `stances.toml`, `status.toml`; code comments in `ability.rs`, `burden.rs`, `combat.rs`, `kit.rs`, `content/{class,kit,mod,resource,stance,status}.rs`, `world/{berserker,buff,mana,rage}.rs`; tests in `character.rs`, `kit.rs`, `content/mod.rs` (the class order), `world/*_tests.rs`; the UI's `chargen_screen.rs` ("elf,male,barbarian") and `screen_snapshots.rs` ("human,male,barbarian", the `berserker` and `class-screen` snapshots); spec `07e`, `08c`, `08e`, `08f`, `08i`; `docs/dev/{character-creation,progression,resources,spells,surprise}.md`; wiki prose in `classes`, `resources`, `combat`, `features`, `character`, `mana`, `status-effects`, `talents`, `surprise`, `skills`, `starting-kit`, `weapon-skills`, `fighting-styles`, `magic-items`; `wiki/TERMS.md` (`classes.md#barbarian`, `combat.md#stance-berserker` as examples); `xtask/src/wiki/mod.rs` (a test for `| Barbarian |`). The design documents under `docs/design/` stay as written (records) |

**The gap**, in one line each: the rage is a timer nothing feeds until level 40; between 6 and 32 the class is a D&D 3.5 brute whose rows never read its rage; Hardened sits at 25 and pays at 51; the class has one button; Intimidate is trained and unread; and the class and its stance share a name that will mean two things once the class is the Berserker.

## 3. Research: rage in other games, tied to decisions

Sources are the ones the web turned up for this document (listed at the end); where a page could not be fetched the figures are the search summaries' and marked so.

| Game | What its rage is | Taken, or not, and where |
|---|---|---|
| **ADOM** (class powers; the ancardia wiki's class power table, the 2007 barbarian guide) | 6 mighty blow (double damage at 2500 EP, two and a half actions), 12 base movement 750 EP (a quarter quicker), 18 tremendous blow (triple damage at 4000 EP), 25 +1d4 hit points a level, 32 +3 Strength and Toughness, **40 +10 melee damage while in the berserk tactic**, **50 melee at 800 EP while in the berserk tactic**. The ADOM II dev blog (2012) calls the 40 and 50 powers "the least-utilized of the bunch", because players rarely fought in the berserk tactic | the cadence itself is ADOM's (`docs/design/progression.md` §4.1). 32 → Mighty thews stays; 25 → Hardened stays (moved to where it pays); 12 → Headlong keeps the movement idea *while raging* (§5); 50 → Blood frenzy's attack speed is exactly ADOM's "800 EP while berserk", at 40. The blog's lesson shapes the whole table: our berserker is **held** in the frenzy stance while raging (#680), so rewards bound to the stance are always collected; and the builds (§4) make raging the normal way to fight, not a rare mode. Mighty and tremendous blow are **not** taken: Mighty swing (Great Weapon rank 7, double dice at 150 %) is already that, and a class copy would be a clone (§11) |
| **D&D 3.5** (SRD) | Rage once a day: St +4, Con +4, Will +2, AC −2, then fatigued; cannot do anything needing patience or concentration (no spells, no skills of the mind); Illiteracy; Fast movement 1; Uncanny dodge 2; Damage reduction 7; Greater rage 11 (+6); Indomitable will 14; Tireless rage 17 (no fatigue); Mighty rage 20 (+8) | the shape Warlock has: Raging's raises, `blocks` read and cast, Exhausted after. Mighty rage (50) is Greater/Mighty rage; Exhausted stays (Tireless rage rejected, §11: without the exhaustion a berserker would re-rage every wait and never leave the frenzy) |
| **D&D 5e** (the class table and the Berserker path, dndbeyond) | Rage a bonus action, one minute, **ends early if your turn ends and you have neither attacked a hostile nor taken damage since your last turn**; +2 to +4 damage, resistance to weapon damage, advantage on Strength; Reckless Attack 2 (advantage for advantage against you); Danger Sense 2; Feral Instinct 7 (act when surprised if you rage first); Brutal Critical 9/13/17; Relentless Rage 11 (Con save at 0 hp, DC up by 5 a use); Persistent Rage 15; Primal Champion 20 (St and Con +4). Path of the Berserker: Frenzy 3 (an extra attack each turn while raging, exhaustion after), Mindless Rage 6 (no charm or fear while raging), Intimidating Presence 10 (a Wisdom save or frightened), Retaliation 14 (a free blow at whoever damages you) | "ends if you neither attacked nor were hit" → the builds: a point a blow landed either way, against a point a turn lost (§4). Reckless Attack is the frenzy stance (to-hit up, Dodge down). Brutal critical, Relentless rage (Unstoppable), Primal Champion (Mighty thews) stay. Frenzy → Blood frenzy's attack speed (40); Intimidating Presence → War cry (25). Mindless Rage rejected (Frightened does nothing to the player: information only, §11). Retaliation deferred (§11). Feral Instinct not taken: the barbarian trains Initiative, and Hair trigger and Cool head are picks |
| **Pathfinder 1e** (rage powers, aonprd) | a power every two levels, usable only while raging: Intimidating glare, Terrifying howl (nearby shaken foes panic), Swift foot (+5 ft while raging), Renewed vigor, Strength surge, Reckless abandon, Come and get me, No escape, Fearless rage, Mighty swing | "only while raging" is the model for the `while` condition (§6.1): the class's later rows are rage powers. Terrifying howl → War cry; Swift foot → Headlong. Mighty swing exists as a weapon ability |
| **DCSS** (CrawlWiki *Berserk*, *Slow*; search summaries) | Berserk: hasted, might (more melee damage), +50 % hit points, only move and melee; **(19 + 1d19) / 2 turns, 10 to 19, weighted to 15**; for Trog's followers **a kill may extend it**; an action other than a melee attack halves what is left; after it, Slow and −Berserk for about twice as long; Trog's Hand (regeneration), Brothers in Arms (summons) | DCSS's rage is a fight's length, extended by killing: the model for a pool that builds on kills (§4). Its exhaustion is twice the rage's length, which argues for a shorter Exhausted now that a rage lasts a fight rather than a fixed twelve turns (open question 2: 300 s). "Only move and melee" is harsher than ours (`blocks` read, study, cast); not widened: a berserker who cannot open a door or drink is not fun, and the owner's "no anti-farming limits" rule leans the same way. Brothers in Arms is the druid's territory (#821) and not touched |
| **Path of Exile** (the Berserker ascendancy; wiki and build guides, search summaries) | Rage 0 to 50: each point more attack damage, attack speed, and movement speed; gained on hit (Crave the Slaughter) or kill; **lost a point every 0.5 s when you have neither gained rage nor been hit recently**; Berserk spends rage for a buff; Rite of Ruin (rage's effects stronger, life lost per rage), Aspect of Carnage (+40 % damage, +10 % damage taken), War Bringer (warcries grant rage), Defy Pain, Flawless Savagery, Blitz | decay that pauses while you are hit → the `struck` build (§4): being hit feeds the fury. A rage-scaled damage meter (a point of damage per 5 rage) was considered and rejected (§11): the pool is the rage's duration, and a meter rewards hoarding what War cry asks you to spend. Aspect of Carnage is the frenzy stance; Rite of Ruin is Mighty rage |
| **Diablo II / III** (diablowiki, icy-veins, search summaries) | II: Frenzy (attack speed stacking with hits), Berserk (damage and attack rating up, defence to nothing), Whirlwind, Battle Orders, Howl, Taunt, War Cry (stun), Natural Resistance, Iron Skin. III: **Fury built by generators (hits) and bleeding away out of combat**, spent on Wrath of the Berserker and the rest; Battle Rage (more fury gained, less lost) | Frenzy → Blood frenzy's attack speed; Berserk → the frenzy stance; Howl and War Cry → War cry; Fury's build-and-bleed → the builds and the decay. Battle Rage is Battle trance (a pick that exists). Whirlwind is the warrior's (50) and not duplicated |

What is **not** taken from anywhere, with the reason, is in §11.

## 4. The pool reworked: fighting feeds the rage

```toml
# The berserker's rage (the Rage ability): granted at 12, it counts down a point a base turn,
# resting too, and ends at zero, taking the Raging status with it (then Exhausted follows).
# Fighting feeds it: a blow of the holder's that hits, a blow that hits them, and a kill; so a
# rage lasts the fight and burns out after. Blood frenzy feeds it more by kills, Mighty rage
# raises its maximum, War cry and Unstoppable spend it (talents/).
[[resource]]
id = "rage"
name = "Rage"
short = "Rage"
max = 20
start = 12
show_max = false
decays = 6
ends_at_zero = true
status = "raging"
lacking = "you have not the rage for it"
builds = [{ on = "hit", amount = 1 }, { on = "struck", amount = 1 }, { on = "kill", amount = 3 }]
```

| Rule | Decision | Why |
|---|---|---|
| **Builds** | `hit` +1 (a blow, shot, or throw of the holder's that hits; a critical is a hit too, so it builds once), `struck` +1 (a blow that hits them), `kill` +3; on the resource, so whoever holds Rage has them (the berserker; the dev `resource:rage`; nobody else: the affix of Fury grants no pool) | D&D 5e's "attacked or was hit" rule and Path of Exile's paused decay, as numbers: at one action a base turn, a berserker in melee who lands seven blows in ten and is hit back half the time gains about 1.2 a turn against the 1 lost, so the rage **holds or grows while they fight** and **runs down in about twelve turns once nothing is near**. A kill tops it up (DCSS). The pool is capped (20; 30 with Mighty rage), so a long fight never banks a long calm |
| **Start** | 12, as today | the first turns of the rage matter before the first blow lands; Battle trance's +6 stays worth its pick |
| **Decay** | a point a base turn, resting too, as today | the rhythm is unchanged; the builds, not a slower decay, are what lengthen a rage, so a berserker who stands still still cools |
| **Exhausted** | `eases = 300` (five minutes), from 600 | DCSS makes the slowed spell about twice the rage's length; a fight-long rage of two to four minutes makes ten minutes of Exhausted a long wait between fights on a floor where fights come every few minutes. Mighty rage's `status_eases` 100 % still halves it (150 s). Open question 2 |
| **`lacking`** | "you have not the rage for it": how the abilities menu greys War cry while the pool is short or not held (`Unavailable::Short`'s words for this pool, as stamina's "you are short of breath") | the menu's one place for it |
| **Blood frenzy's build** | stays a talent `resource_build` (+5 a kill at 40), on top of the data's 3 | a kill at 40 is +8: the frenzy of the name |

**A worked rage, level 1.** A human berserker (St 13, To 12) steps into a room of three kobolds and rages (a wait; 12 rage, Raging: St 15, To 14; the frenzy stance: +2 to hit, +4 damage, −6 Dodge). Turn 1: the rage ticks to 11; a blow lands, 12. Turn 2: a kobold hits them, 13; their blow misses, 12. Turn 3: a kill, 14 (+3, −1). Three kobolds take about eight turns; the rage ends the fight at 12 to 14, and burns out over the next twelve or so turns of searching the room ("Your rage subsides.", then "You are exhausted." for five minutes: St −1, and no rage). Today the same fight would see the rage end on turn 12 mid-fight if a fourth kobold came, and the berserker fall back to aggressive, able to read a scroll but no longer the berserker.

**Creatures** hold no resources (the resource rule); the orc berserker stays a creature with a name, its frenzy a stat block.

## 5. The level table, row by row

| Level | Today | Verdict | After |
|---|---|---|---|
| 1 | **Rage** (active) | **stays, fed** | the ability is unchanged; the pool it grants builds by fighting (§4). Description: "Fly into a rage (an ability): stronger and tougher, too wild to read or cast, and fighting keeps the fire fed; exhausted after, and no rage till then." |
| 1 | Trained wind | stays | the martial classes' shared row |
| 6 | **Thick hide** (natural armour +2 in no body armour) | **stays** | the unarmoured brute of D&D and ADOM; it answers the frenzy's lost Dodge in the slot where Wild guard also unlocks; the one `natural_unarmoured`, no clone. Its row does not read rage, by choice: a berserker's hide is theirs cold or hot |
| 12 | **Fast movement** (steps −10 %, unarmoured and Unburdened) | **goes**; the slot takes **Headlong** | "Raging, you go headlong: your steps cost a fifth less while you wear no body armour." `{ kind = "step_percent", value = −20, unarmoured = true, while = "raging" }`. ADOM's 12 (750 EP) and DCSS's haste, bound to the rage; twice today's figure because it lasts only the rage. The Unburdened condition is dropped (a raging berserker hauling loot still charges). The id `fast_movement` goes (a save holding it loads without it, `Talents::drop_unknown`; `class::grant` on load gives Headlong to a berserker of 12 or more). Fleet (the monk's 6) is no longer paired with it in the "kept near misses" list of `docs/dev/progression.md` |
| 18 | **Brutal critical** (two-handed critical multiplier +1) | **stays** | D&D 5e's 9; the great axe and the great sword are the kit. It does not read rage, by choice: the frenzy stance's +2 to hit makes the threat more often confirmed anyway |
| 25 | **Hardened** | **moves to 50** | it pays only past `hp_level` (50): at 25 it is a promise for 25 levels (flagged in `docs/design/combat-progression.md` §9.4). At 50 it pays from the next level, exactly as it does today. The slot takes **War cry** (§6.3) |
| 32 | **Mighty thews** (St +2, To +2) | **stays** | ADOM's 32 (+3 St and To) and 5e's Primal Champion |
| 40 | **Blood frenzy** (a kill +5 rage) | **reworked** | "Raging, you fight in a blood frenzy: your blows come a fifth faster, and every kill feeds the fire." `effects = [{ kind = "attack_price_percent", value = −20, while = "raging" }, { kind = "resource_build", resource = "rage", on = "kill", amount = 5 }]`. ADOM's 50 (melee at 800 EP while berserk), 5e's Frenzy, Diablo's Frenzy; with the Great Weapon style's speed row (20 % at rank 10) a master berserker's blows come at 64 % of the price. The id and the build stay |
| 50 | **Mighty rage** | **stays**, Hardened beside it | ADOM's 40 and 50, D&D's Greater and Mighty rage. Two rows at one level, as level 1 has two and the ranger's 6 has two |
| 60 | **Unstoppable** | **stays** | 5e's Relentless Rage; the capstone changes a fight (rule 1) |

The class-limited picks keep their place: Battle trance (Might, berserker 6) and Wild guard (Might, berserker 6; `stance = "frenzy"`). Nothing new joins the picks (§10).

## 6. The new abilities, and the `while` condition

### 6.1 `while`: an effect that counts only under a status

`data/talents/README.toml` gains one field every effect may carry:

```
#   while = "raging"              the effect counts only while its holder suffers that status
#                                 (../status.toml); left out, always
```

| Rule | Decision | Where |
|---|---|---|
| **Parse** | `while: Option<StatusId>` on `TalentEffectSpec`, resolved after the statuses load (as `second_wind`'s `cooldown` is); a `Rank` keeps its conditional effects apart from its plain ones (`Rank::effects_while: Vec<(StatusId, TalentEffect)>`) | `content/talent.rs` |
| **Sum** | `Talents::mods(content, effects: &Effects)`: the plain effects as today, then each conditional effect whose status `effects.has(status)`. `Actor::talent_mods(content)` passes `&self.effects`; it is the one summing place, so **every hook keeps its one read** and no reader knows the condition exists. The borrowed `Cow` for actors without talents is unchanged | `talent.rs`, `world/actor.rs` |
| **The alike check** (#790) | an effect's condition is part of its identity: `step_percent −20 while raging` is not alike `step_percent −10` (Fleet), so the check needs no exception; a listed `alike` pair stays honest | `content/talent_alike.rs` |
| **Prerequisites, the Talents page, the dump** | unchanged: they read the talents held, not the sum | — |
| **The wiki** | `effect_text` appends "while raging" (the status's name, lower case) | `xtask/src/wiki/talents.rs` |
| **Why a status, not "while raging" only** | the status id makes it one mechanism for any buff a class or a spell brings (a later paladin or bard row may say `while = "blessed"` or `while = "inspired"` without touching this code); the Raging status is on an actor only through their own Rage (and the dev `resource:rage`), so `while = "raging"` is exactly `berserker::raging` for the player | — |

The two kinds that already say "while raging" in their own way (Mighty rage's `raise_percent { status = "raging" }` reads the status's raise, Unstoppable's `unstoppable` is read in `hurt` with a `raging` check) stay as they are: a rewrite would change nothing the player sees.

### 6.2 Headlong (level 12)

`data/talents/headlong.toml`: tree `class`, `granted`, `effects = [{ kind = "step_percent", value = -20, unarmoured = true, while = "raging" }]`, `systems = ["speed", "resources"]`. No new kind: `step_percent` with its `unarmoured` condition exists (`World::step_price`, `TalentMods::step_percent`). A raging berserker in no body armour steps at 80 %; in mail, or calm, as anyone. With Fast movement gone, a berserker between rages walks at the pace of their race: the fantasy the owner asked for.

### 6.3 War cry (level 25): the roar, and Intimidate's first use

```toml
[[talent]]
id = "war_cry"
name = "War cry"
tree = "class"
granted = true
description = "Roar your fury (an ability, while you rage): the foes who hear and see you flee from you a few moments, unless they keep their nerve; it is heard far."
resource = { id = "rage", cost = 4 }
effects = [{ kind = "war_cry", status = "frightened", severity = 3, difficulty = 10 }]
systems = ["status", "hearing", "skills", "resources"]
```

| Rule | Decision | Where |
|---|---|---|
| **The active** | `ClassActive::WarCry` (`use:war_cry`, `Aim::None`), a row of the abilities menu's Class group, no key of its own; priced as a wait (`price_of(Wait)`; `kind_of` → `Wait`, which Raging does not block) | `content/class.rs`, `world/class_ability.rs`, `world/war_cry.rs` |
| **The cost** | 4 rage, the talent's `resource`: spent after the refusals (`use_class`), so only a raging berserker roars (the pool is held only while raging) and the menu greys it "you have not the rage for it" otherwise. Four rage is four turns of the fury traded for the crowd's fear; no cooldown (a resource pays, "time is the penalty") | `ability.rs::short_of` |
| **The noise** | `make_noise(player, Act::Howl)`: a howl's loudness and marker (`h`, the hearing dump), heard across the floor and through doors by the rules of every noise; it wakes a den's sleepers at their threshold and sends every patrol to look. That is the cry's real price, told nowhere and learned by play | `noise.rs` |
| **Who quails** | every hostile (`World::stance == Hostile`) that **hears** it (`noise::hearers`, the howl's hearers before it is made, as Hound's bay does) and **sees** the player now (`World::sees_now(id, player)`: fear needs the sight of the roaring berserker) and **has vitals** (`Content::has_vitals`, the surprise design's flag: the bones, the spirits, the animated, the elementals, and a type that says `vitals = false` have no heart to quail; the fleshy undead do, as they bleed) | `war_cry.rs`, one loop in actor-id order |
| **The check** | each such creature rolls `check::check(rng, its Willpower modifier, difficulty + the player's Strength modifier + the player's Intimidate benefit)`, two gameplay draws apiece, in actor-id order; natural 2 fails, 24 holds. **Made**: it stands ("The orc does not flinch."). **Failed**: `afflict(id, frightened, 3, Some(player))`: three degrees, a degree easing every 6 s, so it runs from the berserker for about three turns and fights only when cornered (the status's `flees`, read by `World::frightened` in the AI as today). Strength, not Presence: a berserker's menace is their size and their fury (their Presence is −1 by class), and the stat the class lives by trains by the cry's use too | `war_cry.rs`, `check.rs` |
| **Intimidate** | gains `benefit = [{ effect = "intimidate", curve = [[0, 0], [100, 8], [200, 12]] }]` (the ladder every check benefit uses: Initiative's, Stealth's slip) and `mark = [{ on = "intimidate", chance = 100 }]` (a mark per cry, whatever comes of it, the training stream; the player's only, as every mark). `skills::Effect::Intimidate`, read in `war_cry.rs` and nowhere else; `Use::Intimidate`. The skill keeps `kind = "action"`; the Skills page's "who trains what" and the abilities menu's Skills group are unchanged (the cry is the class's ability, listed under Class; Intimidate has no row of its own, as Initiative has none) | `data/skills.toml`, `skills.rs` |
| **Odds** (the wiki states them) | a creature at Willpower 10 against difficulty 10 + St +2 (14): 2d12 ≥ 14 in 66 of 144, so it **flees 54 %** of the time; at Intimidate 100 (+8; difficulty 22) it stands only on 22 or more, 6 of 144: flees 96 %; a Willpower 16 foe (+3) against the level-25 berserker (St 18, +4; Intimidate about 60, +5; difficulty 19): stands on 16 or more, 45 of 144: flees 69 % | — |
| **No level cut** | Hound's bay frightens only lesser levels and rolls nothing; War cry rolls and cuts nothing: a troll chief with Willpower 18 stands more often than a kobold, and a berserker of 60 with Intimidate 150 may send a troll running. Open question 5 asks whether to add the bay's level cut as well (default: no) | — |
| **Refusals, no time** | not raging (the pool is not held: the menu says why, and `use_class` refuses "You have not the rage for it."); nobody hostile hears and sees you ("No one is near to hear it." — the roar is still not made: a cry at an empty room is a wasted turn and a woken den, and the game is kinder than that; open question 6) | `war_cry.rs` |
| **Not a clone** | Hound's bay (`hounds_bay { status, severity, cooldown }`, the Barghest's gift, a horoscope entry the owner keeps whole) frightens lesser levels on a cooldown with no check; War cry (`war_cry { status, severity, difficulty }`) checks, cuts no level, costs rage, and trains a skill. Different kinds, so the alike check passes without a listing; the design says so here because the two are cousins | `content/talent_alike.rs` |

**Why 25.** The ability spends rage and wants a pool worth spending: by 25 the berserker has Headlong, the rage holds through fights (§4), and the bonus picks of 10 and 20 may have brought Battle trance and Wild guard. D&D's Berserker gets Intimidating Presence at 10 of 20; 25 of 60 is the same share.

### 6.4 Blood frenzy (level 40), reworked

`attack_price_percent { value }` is one new kind: "its melee attacks cost that percent more (negative: less)", read in `World::attack_price` beside `two_handed_price` (the one place melee attacks are priced; shots and throws are `shot_price` and untouched). With `while = "raging"` on it, Blood frenzy's blows come at 80 % while the rage lasts; the kill build stays. Great Weapon's speed row (20 % at rank 10) multiplies in as today, so a rank-10 raging berserker strikes at 64 %. ADOM's level-50 power, at 40.

## 7. The rename: class, stance, ids, saves, replays, data, docs

### 7.1 The class

| Where | Change |
|---|---|
| `data/classes/01-barbarian.toml` → **`01-berserker.toml`** | `id = "berserker"`, `name = "Berserker"`, `description = "A warrior of the wild lands who fights in a fury: mighty and hardy, quicker to rage than to read."`; **`was = ["barbarian"]`** (new optional field: the class's former ids, each checked at load to be no live class id; the classes `README.toml` documents it). Everything else as it is |
| `Content::class(id)` | finds a class by `id` **or by one of its `was`**: the one resolving lookup, so `CharacterSpec`s, kits, `class::grant`, `creation_talents`, perception, and the Talents page all follow. `Candidate::player`'s class (`Need::Class`) and `same_choice` compare the **resolved** id, so a save's `"barbarian"` meets `needs = { class = "berserker" }` |
| Saves | `World::from_state` rewrites `character.spec.class` to the live id when it is a former one (one line beside `Talents::drop_unknown`, logged in `retiered`), so a save is written back modern and the Characters screen groups it with new berserkers. No stream is drawn from the spec after creation, so a rewritten save plays on exactly. `save::Summary` (name, race and class **names**) is recomputed at the next save |
| Replays | the spec string a replay carries is **kept** as written and resolved at every lookup; `CharacterSpec::start` names its creation streams by the string (`character/kit/human,male,barbarian`), so **an old replay draws exactly what it drew** and its hashes hold through S1. A new run writes `berserker`; a replay recorded from it is a new replay. `--character dwarf,female,barbarian` is accepted (a former id) and the run is a berserker |
| `needs = { class = "barbarian" }` | → `"berserker"` in `battle_trance.toml`, `wild_guard.toml`; `Need::Class`'s words ("to be a berserker") follow the class's name |
| `background/04-youth.toml`, `07-leaving.toml` | `when = { class = [...] }` → `"berserker"` (the raid line: "You rode on your first raid before your voice had broken..." fits the berserker as it did) |
| Comments in data and code | `experience.toml`, `resources.toml`, `stances.toml`, `status.toml`, `classes/README.toml`; `ability.rs`, `burden.rs`, `combat.rs`, `kit.rs`, `content/{class,kit,mod,resource,stance,status}.rs`, `world/{berserker,buff,mana,rage}.rs`: "barbarian" → "berserker" where the class is meant (a reviewer reads each: `combat.rs`'s "a dwarf barbarian (6 + 8) with To 16 has 30" is a worked number and stays true) |
| Tests | `content/mod.rs` (the class order), `character.rs`, `kit.rs`, `burden.rs`, `world/{class_active,berserker,buff,character,fight,racial,resource,resource_talent,stamina,style,weapon_skill,affix_stamina}_tests.rs`: the spec strings and the names; one new test: a save and a replay naming `barbarian` load as berserkers with their talents and picks |
| The UI | `chargen_screen.rs` ("elf,male,barbarian" → berserker), `screen_snapshots.rs` ("human,male,barbarian"; the `class-screen` snapshot's row reads "Berserker"; the `berserker` snapshot is the stance's, §7.2) |
| Spec | `07e` (the class list, hit points, skills, weapon skills, styles), `08c` (Stances, §7.2), `08e` (Resources: Rage; status examples), `08f` (the burden example), `08i` (the class tables, the actives and passives; Trained wind's row), `06` (the abilities menu's Class list gains War cry in S5) |
| `docs/dev/` | `character-creation.md`, `progression.md` (the module table's `world/rage.rs` and `world/berserker.rs` lines, the class tables, the resources paragraph), `resources.md`, `spells.md`, `surprise.md`, `creatures.md` "Stances" |
| The wiki | prose in `classes`, `resources`, `combat`, `features`, `character`, `mana`, `status-effects`, `talents`, `surprise`, `skills`, `starting-kit`, `weapon-skills`, `fighting-styles`, `magic-items`; anchors `classes.md#barbarian` → `#berserker` everywhere (`cargo xtask wiki --check-links` finds a miss); `wiki/TERMS.md`'s examples; `xtask/src/wiki/mod.rs`'s test (`| Berserker |`). The generated tables (`classes.md`, `class-levels.md`, `class-kits.md`, `mana.md`, skills) follow the data |
| `docs/design/*.md` | **unchanged**: they are the record of what was decided under the old name (the README says so) |
| The orc berserker | a creature, unchanged: a tribe's berserker is a creature named for what it does, as the drow warrior and the orc shaman are |

### 7.2 The stance (open question 1)

Once the class is the Berserker, "Stance Berserker" on a Berserker's character screen, "You fight as a berserker." as a Berserker's rage begins, and `combat.md#stance-berserker` beside `classes.md#berserker` make one word carry two meanings. ADOM keeps them apart (a *berserk* tactic, no class of the name); so does this design:

| Where | Change |
|---|---|
| `CombatStance::Berserker` → **`CombatStance::Frenzy`**, `#[serde(alias = "berserker")]` | saves holding `"berserker"` load; a save is written back as `frenzy`. The dump's `stance` id is `frenzy`; `stance:berserker` and `stance:frenzy` are both refused as today ("Only rage brings the frenzy.") |
| `data/stances.toml` | `id = "frenzy"`, `name = "Frenzy"`; the numbers unchanged (to-hit +2, damage +4, Dodge −6, fleeing steps 150 %, marks 125 %) |
| `world/berserker.rs` → `world/frenzy.rs` (`frenzy::settle`, `frenzy::raging`), `berserker_tests.rs` → `frenzy_tests.rs` | the words: "You fight in a frenzy." (the rage begins), "You fall back into an aggressive stance." (unchanged), "Only rage brings the frenzy.", "Your rage holds you in the frenzy." |
| `wild_guard.toml` | `effects = [{ kind = "stance_dodge", stance = "frenzy", percent = 50 }]`; description "Even in the frenzy of your rage your guard keeps half of what it would give up." |
| The `berserker` screen snapshot | renamed `frenzy` (the status bar's "Stance Frenzy" and the character screen's row) |
| Spec `08c` "Stances", `docs/dev/{resources,creatures,progression}.md`, wiki `combat.md` (`#stance-frenzy`), `wiki/TERMS.md` | the name and the anchor |

The alternative, keeping "Berserker" for both, changes nothing and reads "a berserker fights as a berserker": true, and the owner may prefer the word kept where it is. The recommendation is Frenzy, for the one-meaning rule the other class names keep, and because "Blood frenzy" (40) then names what it deepens.

## 8. What the player sees

| Moment | Words (told after the fact; never a number) |
|---|---|
| The rage begins | "You fly into a rage!" then "You fight in a frenzy." (as today, the stance's new name) |
| A blow lands or is taken while raging | nothing new: the status bar's `Rage 13` moves (the pool is shown without its maximum, as today) |
| The rage burns out | "Your rage subsides." / "You are exhausted." / "You fall back into an aggressive stance." (as today) |
| War cry | "You roar your fury!" then, for each hostile seen: "The kobold quails and flees from you." or "The orc does not flinch."; look mode "(hostile, wounded, frightened)" while the fear lasts (the status's `seen`) |
| War cry refused | "You have not the rage for it." (not raging; also the menu's grey) / "No one is near to hear it." |
| Headlong, Blood frenzy | nothing: the pace is felt, not told (speed is hidden; the wiki states the percents) |
| The abilities menu | under Class: Rage (greyed "you are raging" / "you are exhausted" as today), War cry (greyed "you have not the rage for it" until the rage is up) |
| The character screen | `Class Berserker`; `Stance Frenzy` while raging; `Rage 13` under the stance |
| Level-ups | "You gain Headlong." (12), "You gain War cry." (25), "You gain Blood frenzy." (40), "You gain Mighty rage." and "You gain Hardened." (50) |
| The wiki | the Classes page's Berserker heading and table, Levels (the generated class table shows every row's rule: "while raging" appended by `effect_text`), Resources (Rage's builds in the table's column), Abilities (War cry with its check and odds), Skills (Intimidate's use and benefit), Combat (the Frenzy stance's row), Status effects (Exhausted's five minutes) |

## 9. Determinism, saves, replays

| Part | Decision |
|---|---|
| Builds | `gain_resource` draws nothing; the pool's value is in the hash (`Actor::resources`), so **a replay whose berserker fights while raging hashes differently from the first blow that lands or is taken** (S3). Replays that never rage are unchanged |
| War cry | two gameplay draws per creature that hears and sees the berserker and has vitals, in actor-id order, before the fear is given; the noise as any howl's; Intimidate's mark on the training stream (the player's only) |
| `while` | no draws; a sum that reads statuses the actor already has |
| Saves | no new state: the pool, the statuses (Raging, Exhausted, Frightened), the stance exist. The class id and the stance id load their former names (§7); a save is written back with the live ones. The content fingerprint does not count classes (`fingerprint_for`'s groups: skills, weapon skills, statuses, resources, ...), so no save is refused for the rename; a new talent (`war_cry`, `headlong`) is named by id in saves (talents are not in the fingerprint) |
| Old saves at 12 or more | lose `fast_movement` (`Talents::drop_unknown`, logged) and are given `headlong` by `class::grant` on load (a grown table gives what is missing); at 25 or more `war_cry` likewise; Hardened at 50: a berserker of 25 to 49 who held it keeps it (nothing is taken away, and it pays nothing before 51 either way); one of 50 or more holds it already |
| Replays in the repo | none names a barbarian (`runs/`, `tests/golden/manifest/`); the screen snapshots and tests that do are code and move with S1 |

## 10. Talents and coverage (the policy's answer)

The feature template's "Talents and class abilities" section, answered:

- **No new system.** Everything here extends systems in the list: `resources` (the builds, War cry's cost), `stances` (the Frenzy rename), `status` (Frightened, Exhausted), `hearing` (the roar), `skills` (Intimidate's use and benefit), `speed` (Headlong). No `[[system]]` is added, no `pending` touched.
- **No new pickable talent**, by the policy's own rule ("a count is never a reason to add a talent"): `resources` has Battle trance, Inner calm, Steady hand, Patient blade; `stances` has Wild guard and the warrior's Iron stance; `skills` its own. The class abilities are the deliverable.
- **Rules 1–9**: no capstone touched (Unstoppable stays; Mighty rage and Hardened at 50 change a fight and a dungeon); no XP or gold effect; no clone (§6.3's War cry against Hound's bay, Headlong against Fleet: different kinds or a condition); no chain (no new talent needs one talent); no flaw; nothing outdoors-only or seasonal; no hidden-marks talent (Intimidate's mark is a skill's, shown as its progress); no information-only ability (Mindless rage was dropped for it, §11); no shop favour. Tags match hooks: Headlong `speed`, `resources`; War cry `status`, `hearing`, `skills`, `resources`; Blood frenzy `combat`, `resources`, `status`.
- **The `while` condition** is a mechanism, not a system: it reads a status the actor has, and adds no tag.

## 11. Rejected ideas

| Idea | From | Why not |
|---|---|---|
| **Mindless rage**: no fear while raging | D&D 5e Berserker 6 | Frightened does nothing to the player but its message (`flees` is the AI's): an immunity to it is information only (rule 8). If fear ever bites the player, revisit |
| **A rage-scaled damage meter**: +1 damage per 5 rage held | Path of Exile | the pool is the rage's duration; a meter rewards hoarding exactly what War cry asks the berserker to spend, and the frenzy stance's +4 and Raging's St +2 are the damage already. Mighty rage doubles them |
| **Savage blow**: a rage-paid double-damage blow | ADOM 6 (mighty blow), 18 (tremendous blow) | Mighty swing (Great Weapon 7: double dice, 150 %, 5 stamina) is that blow; a class copy is a clone (rule 3). The berserker's kit trains the style from level 1 |
| **Tireless rage**: no Exhausted after | D&D 3.5 (17), 5e Persistent Rage | the rage costs a wait and nothing else; without Exhausted a berserker would re-rage every time it ended and never leave the frenzy (no reading, no casting, Dodge −6 for ever). Mighty rage halving it is as far as it goes |
| **Rage as a pool always held** (sits at 0, builds by fighting, the Rage ability or a threshold turns the frenzy on) | Diablo III's Fury, Path of Exile | the "ends at zero, Exhausted follows" rhythm and the stance lock need a clear on and off, and `ends_at_zero` with `status` gives it in data today; the always-held model would add a second state (held but not raging) to every reader of the stance. The builds give the fight-long rage without it |
| **Berserkitis**: a rage that starts by itself when struck or surprised | DCSS's mutation | the owner's agency: raging is the player's choice (and an unwanted rage blocks reading at the wrong moment); DCSS treats it as a flaw |
| **Retaliation**: while raging, a foe whose blow hurts you takes your blow at once | D&D 5e Berserker 14 | strong and good, and it needs a new free-blow cue (`Cue::Hurt`), which belongs with the free-blows system's next pass rather than a class rename; the table has no empty row for it. **Deferred** to #196 as a Might pick candidate ("Fury's answer") |
| **Reckless attack** as an ability | D&D 5e 2 | it is the frenzy stance (to-hit up, Dodge down), which the rage already forces |
| **Feral instinct**: act freely when surprised if you rage first | D&D 5e 7 | the barbarian trains Initiative, and Cool head and Hair trigger are picks of the awareness system (#686); a class copy is a clone |
| **Only move and melee** while raging | DCSS | harsher than `blocks` read, study, cast; a berserker who cannot open a door, drink a potion, or pick up loot is not fun, and the owner's no-lockout instinct leans the same way |
| **Brothers in Arms**: a summoned ally | DCSS (Trog) | the druid's design (#821) owns player summons |
| **A rage-bound Brutal critical or Thick hide** | this design's first draft | binding the body rows to the rage would nerf the calm berserker to nothing but a timer's worth of class; three body rows (6, 18, 32) and six rage rows is the balance the owner's "lean more" asks for, not "only" |
| **Renaming the Rage resource** "Berserk" or "Fury" | DCSS, Diablo | Rage is the word in every data file, save, affix (of Fury is taken), and wiki anchor; the class is renamed, not its pool |
| **A whole Intimidate system** (a stare-down action for everyone trained, shop haggling by threat) | Pathfinder's Intimidate skill | War cry is the skill's first use and enough for the berserker; a general action is a later skills pass, and the merchant's favour is rule 9's closed door |

## 12. Implementation plan

| Slice | What | Needs | Changes hashes? | Adds |
|---|---|---|---|---|
| **S1 The class rename** | `01-berserker.toml` with `was`; `Content::class` resolves former ids; `from_state` rewrites the spec; `Candidate` and `same_choice` compare resolved ids; data, comments, tests, snapshots, spec, dev docs, wiki prose and anchors, TERMS.md, the xtask test; a test for an old save and an old replay. **No rule changes** | — | no (replays keep their strings) | no state, no action |
| **S2 The stance rename** | `CombatStance::Frenzy` with the serde alias, `stances.toml`, `world/frenzy.rs`, the words, Wild guard, the `frenzy` snapshot, spec §8 "Stances", dev docs, wiki `combat.md`, TERMS.md | — (S1 first for the words' sense; may ride in S1 if the owner prefers) | no | no |
| **S3 The pool** | `builds` and `lacking` on Rage; Exhausted `eases = 300`; Hardened's row to 50; the Rage talent's description; `resources.md` and `status-effects.md` tables regenerated; spec `08e`, `08i`; `docs/dev/resources.md`; tests (`resource_tests.rs`: a fight feeds the rage; `class_active_tests.rs`: Hardened at 50 pays from 51 as before) and a balance probe row (a raging berserker against the monsters of its level, `docs/balance.md`) | — | yes, raging replays | no state, no action |
| **S4 `while`, Headlong, Blood frenzy** | the `while` field, `Talents::mods(content, effects)`, the alike check's identity, `effect_text`; `headlong.toml` replacing `fast_movement.toml`; `attack_price_percent` and its one read in `attack_price`; Blood frenzy's new effects; tests (`class_active_tests.rs`: steps and blows priced while raging and not; `talent_alike` with a conditional pair) | S3 (the rage lasts long enough to matter) | no (no draws; prices change only while raging, which the hash sees in time) | no state, no action |
| **S5 War cry and Intimidate** | `ClassActive::WarCry`, `use:war_cry`, `world/war_cry.rs`, the `war_cry` effect kind, Intimidate's benefit and mark, `Effect::Intimidate`, `Use::Intimidate`, the words, the refusals, the menu row; spec `06`, `08i`; wiki `abilities.md`, `skills.md`; tests (`class_active_tests.rs`: the cry's cost, who quails and who stands by seed, the fearless, the noise, the refusals; `skills` tests: the mark) | S3 | from the first cry | **one action** (`Action::Use(war_cry)` is the existing `use:ID` path, so no replay-text change) |

Order: S1 and S2 at once (one PR or two small ones); S3; then S4 and S5 in parallel (they touch different files: S4 the talent loader and prices, S5 a new module). Nothing waits on the sibling designs (#819–#826), and none of them touches the barbarian's table, the Rage resource, or Intimidate.

## 13. Open questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| 1 | The stance's name: rename to **Frenzy** (§7.2), or keep **Berserker** for both the class and the stance? | **Frenzy**: one word, one meaning, as the other class names; "Blood frenzy" then names what it deepens. Keeping it costs nothing and is a fair owner's call |
| 2 | Exhausted after a rage: **300 s** (five minutes), or the present 600? | **300**: DCSS's "twice the rage" with a fight-long rage; Mighty rage still halves it |
| 3 | The builds: **hit +1, struck +1, kill +3**, or kills only (DCSS's Trog)? | as proposed: a duel of two should hold the rage, not only a slaughter; `struck` is Path of Exile's "not while you are being hit" |
| 4 | Headlong's condition: **no body armour** (as Fast movement), or unconditional while raging? | **no body armour**: the unarmoured brute is the class's thread through Thick hide and Headlong; a berserker in plate is a choice with a cost |
| 5 | War cry's level cut: the check alone, or **also** Hound's bay's "lesser levels only"? | **the check alone**: Willpower is the creature's nerve, and a berserker of 60 ought to make a troll think |
| 6 | War cry at an empty room: refused with no time ("No one is near to hear it."), or roared anyway (a turn, 4 rage, the noise)? | **refused**: the one-step-back kindness every aimed active has (Low blow's "There is no one there to blind."). A berserker who wants to wake a den can kick a door |
| 7 | War cry's stat: **Strength**, or Presence? | **Strength**: the class's stat, which the cry then trains; Presence −1 by class would make the berserker the worst at its own roar |
| 8 | Frightened's degrees from the cry: **3** (three turns, as Hound's bay) or 2? | **3**: four rage and a turn should buy more than a step |
| 9 | Hardened: to **50** beside Mighty rage, to 45 (off-cadence, as the wizard's Magister), or left at 25? | **50**: it pays from 51 either way, and two rows at 50 read as the body and the fury peaking together; 45 would be a lone off-cadence row |
| 10 | Former ids in data (`was`) with the resolving lookup, or no alias (old saves and replays lose the class; "saves change" as the pre-1.0 policy allows)? | **`was`**: one field, one lookup, and every save and replay written so far keeps its class; it serves the next rename too |
| 11 | Rage's start: **12**, or lower now that fighting feeds it? | **12**: the first turns of a fight are before the first blow lands, and Battle trance's +6 stays a pick worth taking |

### Sources

ADOM: the ancardia wiki's [Class power](https://ancardia.fandom.com/wiki/Class_power) and [Barbarian (class)](https://ancardia.fandom.com/wiki/Barbarian_(class)) pages (the level table; fetched as search summaries), the [2007 barbarian guide](http://adomguides.blogspot.com/2007/11/guide-to-being-barbarian.html), and the [ADOM II dev blog on barbarians (2012)](https://www.ancientdomainsofmystery.com/2012/02/barbarians-in-adom-ii.html). D&D 5e: [Barbarian Class Details](https://www.dndbeyond.com/classes/9-barbarian) (the legacy class table and the Path of the Berserker). D&D 3.5: the [SRD barbarian](https://www.d20srd.org/srd/classes/barbarian.htm) (not fetchable at writing; from memory of the SRD). Pathfinder: [Barbarian rage powers](https://aonprd.com/BarbarianRagePowers.aspx). DCSS: CrawlWiki [Berserk](http://crawl.chaosforge.org/Berserk) and [Slow](http://crawl.chaosforge.org/Slow), and a [new-player guide](https://mitjafelicijan.com/dcss-new-player-guide.html) (the wiki was unreachable at writing; figures from the search summaries). Path of Exile: the wiki's [Rage](https://pathofexile.fandom.com/wiki/Rage), [Berserker](https://pathofexile.fandom.com/wiki/Berserker), and [Berserk](https://pathofexile.fandom.com/wiki/Berserk) pages, and [Rite of Ruin](https://www.poewiki.net/wiki/Rite_of_Ruin) (search summaries; the pages block fetching). Diablo: the [Diablo II barbarian patches](https://www.purediablo.com/d2wiki/Barbarian_Patches), the [Berserker skill tree](https://diablowiki.net/Berserker_Skill_Tree) and [Wrath of the Berserker](https://www.diablowiki.net/Wrath_of_the_Berserker), [Battle Rage](https://diablo.fandom.com/wiki/Battle_Rage), and the icy-veins [Barbarian class](https://icy-veins.com/d3/barbarian-class) guide.

# Warlock — The witch and the wizard: how they differ beyond their spell lists

**Built; this file is history.** Current behaviour is in spec §8 "The casters' class tables, and their spells" and `docs/dev/progression.md`.

Design document for the owner's review (2026-11-02, #826). Written against `origin/main` at
f3846261 (Encounters E5). Nothing here is implemented; §12 cuts it into issues. It builds on
what is there: the class tables (`assets/data/classes/09-wizard.toml`, `10-witch.toml`,
`docs/dev/progression.md` "Class actives", "Caster spells"), the one cast path and the spell data
(`docs/dev/spells.md`), the schools of magic and the caster styles (`docs/dev/magic-skills.md`),
mana (`docs/dev/mana.md`), statuses and their givers (`docs/dev/status-effects.md`,
`status.rs::Effect::by`), scrolls and study (`docs/dev/scrolls.md`, `world/magic/learn.rs`), the
abilities menu and the one registry (`warlock_core::ability`), noise (`noise.rs`), and the talent
coverage policy with rules 1 to 9 of #776 (`docs/dev/progression.md` "Coverage policy"). Every
number is data and hidden in the game; the wiki states it (the owner's rule: the game is vague,
the wiki is accurate).

**The owner's request** (2026-11-02, verbatim): *"Witch/Wizard: Create a design issue to figure
out how these are different except for the different guaranteed spell list."* The gaps report
(#802) gives the cadence 1, 6, 12, 18, 25, 32, 40, 50, 60; the witch's gaps are 6 and 18, the
wizard's 40 (Magister sits at 45). The druid design (#821, draft PR #828: `docs/design/druid-and-summons.md`, its §4 the shared ally
core) owns player summons and allies, and the priest design (#820) healing and housekeeping:
nothing here touches either.

## Contents

1. Summary
2. Research, tied to the decisions
3. What exists today: the two classes side by side
4. The two fantasies
5. The witch: hexes that stick and come home (Cackle at 6, Malison at 18)
6. The wizard: the prepared scholar (Scribe at 12, Magister to 40)
7. Every entry of both tables: stays, changes, goes
8. Determinism, saves, replays
9. What the player sees; the wiki, the spec, the docs
10. Talents and the coverage policy
11. Rejected ideas
12. Implementation plan (slices)
13. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **The difference, in one line each** | The **witch** is the hedge-cunning woman of Anglo-Saxon *wiccecræft*: she makes a foe weaker and outlasts it, pays in blood and herbs, and takes from what dies under her hex. Her mechanic is **the hex**: a status she lays and tends. The **wizard** is the Latin-schooled scholar: he knows more spells than anyone, reads and now **writes** scrolls, knows what things are, and bends every school. His mechanic is **preparation**: mana banked as scrolls between fights, and the ladders and schools lifted by study. §4 |
| **Witch 6: Cackle** (new active, `use:cackle`, aims nothing) | every hex of hers on a creature she sees lasts **a degree longer** (`afflict` at severity + 1, the status's `max` holding it; a hex at its max is refreshed instead). A turn (`cast`'s price), no mana, and **loud**: a voice at loudness 60 (a shout is 50), heard by whatever is near. Refused, with no time, when no creature in sight is under a hex of hers. No draw. §5.1 |
| **Witch 18: Malison** (new passive) | two effects on one hook each: a creature under **any hex** (Hexed, Weakened, Sluggish, Drowsy, Webbed: the statuses the hex spells lay) takes **+3** from her blows **and her spells** (the existing `status_damage` kind, its read widened from blows to `spell_strikes`; Poison lore gains the same reach); and when a creature **dies under a hex of hers**, of anything, anywhere she can see or not, the hex comes home: **mana back, half the dead creature's level, at least 1** (`hex_tithe`, read in `World::dies`). No draw. §5.2 |
| **Wizard 12: Scribe** (new active, `scribe:SPELL`) | writes a scroll of a spell he knows onto a **blank scroll** (a new object, loot and shop stock): **three times the spell's mana**, paid at the end, and the time of a study of that spell (`study.toml`'s turns), interruptible like a study. The scroll made is the ordinary `scroll_of_ID` at its own rung, known. Mana banked as scrolls is mana spent at leisure and loosed at once, with Scroll adept's half-time read and extra rung; a scroll is also a spell for a friend, a sale, or a study. Level 12 is where Spell recall sat until #782 and where a wizard's pool (106) pays for a Fireball scroll (78). §6.1 |
| **Wizard 40: Magister** (moved from 45) | one line of data: the level. 45 is a bonus-pick level (10, 20, 30, 45, 55); 40 is the cadence's. A save at level 40 to 44 gains it on load, as every grown table does (`class::grant`). §6.2 |
| **Everything else stays** | all 23 existing entries of the two tables keep their ids, levels (but Magister's), and effects; two change their reach by one line (Poison lore and Malison read `status_damage` for spells too). §7 |
| **Talents** | no new system and no new pickable: Cackle and Malison extend `status` and `spells`, Scribe extends `scrolls` (a sub-feature adds no tag and no talent). Tags: `cackle` `status`, `spells`; `malison` `status`, `combat`, `spells`, `mana`; `scribe` `scrolls`, `spells`, `mana`. Every entry passes rules 1 to 9. §10 |
| **Determinism** | none of the three draws. Saved state: nothing new (a scribing is one action, run through `keep_at` as a study is). Content: three talent ids (not fingerprinted), one object id (`blank_scroll`), one action id (`scribe`). Old saves load; a witch of 6 or 18 and a wizard of 12 gain the new entry on load. §8 |
| **Slices** | three issues: **W1** Cackle, **W2** Malison, **Z1** Scribe with Magister's move (the move alone is one line and may go first in any PR). No slice adds saved state; Z1 alone adds an action. §12 |

**Key choices, and why**

1. *The witch's mechanic is the hex, because the hex is already hers.* Five of the game's six hexes are the witch's (Evil eye, Hex of weakness, Curse of sloth; Sleep and Web are learned) and no other class casts one by right. Hexes are statuses with a giver (`Effect::by`), degrees, and a `max`: everything Cackle and Malison need exists, and each is one read. Nothing is added to the spell system.
2. *Cackle costs time and noise, not mana.* A hex is cheap (0 to 3 mana); what it lacks is staying power against a fight's length. Cackle trades a turn and a shout for every hex in sight lasting longer: the witch who cackles is heard. "Time is the penalty"; no cooldown, no pool.
3. *Malison closes the loop.* A hexed foe is hurt more by her, and its death pays her hex back. The witch spends on the hex, not on the kill: her pool is smaller than the wizard's and her spells are not the ones that kill from afar, so her economy runs through what she has marked. Siphoning (the affix) pays only a kill within three cells; the tithe pays by the tether of the hex, at any range, by any cause.
4. *The wizard's mechanic is the scroll, because scrolls are already his.* Scroll adept (1), Arcane training (1), Arcane lore (6), and the Scholar and Spellcraft trees make a scholar of books and items; D&D 3.5 gives every wizard Scribe Scroll at level 1, ADOM its magical writing set, NetHack its magic marker. Writing closes the circle: the wizard reads, learns, and now makes the thing he learns from. Prepared spells, the 3.5 wizard's whole fantasy, appear as scrolls in the pack without a slot system.
5. *Three times the mana, at leisure.* A scroll must cost more than a cast, or casting is never the right choice; it must cost less than a rest's whole pool, or no one writes. ×3 makes one Fireball scroll a level-12 pool's work and three a level-30 pool's: a wizard who rests writes, and opens the next fight with a burst he could not otherwise afford.
6. *Magister moves; nothing is added at 40.* The spell scaling design placed Magister at 45 before the cadence was audited; 45 is a bonus-pick level, so the wizard got two things at 45 and nothing at 40. The move is one number and leaves the wizard's table the fullest in the game, with no gap.
7. *No familiar, no brewing, no metamagic.* A familiar is an ally: the druid design (draft PR #828, `docs/design/druid-and-summons.md` §4, slice A1) builds the shared ally core (`Actor::ally`, `AllySource::Summoned | Companion`, orders under `Source::Ally`, following, the retinue, `data/allies.toml`), and its §4.9 lists what is shared and what a class's own ally adds; the witch's familiar is a slice on that core after A1 lands, with its own issue (§11), and asks nothing of A1 now; brewing is a crafting system the game does not have and the hex loop does not need; metamagic is a fourth source on the menu's one number, which the owner's decision 11 (spell scaling) ruled out. §11 has the rest.

## 2. Research, tied to the decisions

Sources: Pathfinder's witch (the d20pfsrd pages, from memory), D&D 3.5's wizard (SRD), 5e's wizard and warlock (the Basic Rules), ADOM (magical writing sets, the Necromancer, the Wizard's spellbooks; ancardia wiki, from memory), NetHack (the magic marker; nethackwiki, from memory), DCSS (Hexes and Necromancy; the crawl wiki, from memory), and Old English: Bosworth–Toller for *wicce*, *hægtesse*, *wyrtgælstre*, *galdor*, *lyblāc*; the Lacnunga's *Wið færstice* and the Nine Herbs Charm.

| Source | What it does | What it drives here |
|---|---|---|
| **Pathfinder witch** | hexes at will (no cost), chosen from a list; **Cackle** (a move action: every Agony, Charm, Evil Eye, Fortune, or Misfortune hex within 30 ft lasts a round longer); Evil Eye (−2 to a thing); Slumber; major hexes at 10, grand at 18; a familiar holds her spells; a patron | **Cackle** (§5.1) is taken by name and shape: a short action that keeps hexes going. Hexes stay spells paid in mana (the owner: cantrips cost mana). The familiar is deferred to the ally core of the druid design (#828 §4; §11 here). Her early-level hexes already follow the "hex at 1, more at 3" rhythm |
| **D&D 3.5 wizard** | prepared spells; **Scribe Scroll** as the free bonus feat at level 1; bonus feats (metamagic or item creation) at 5, 10, 15, 20; a specialist school with prohibited schools | **Scribe** (§6.1): the wizard's own craft since 3.0. Preparation appears as scrolls in the pack, not slots (the owner: no slots). Metamagic rejected (§11). Specialization is School focus, which exists |
| **5e wizard** | Arcane Recovery (mana back once a day), Spell Mastery (a chosen 1st and 2nd level spell cast free), Signature Spells (two 3rd-level spells free once a day) | rejected: Arcane mastery (50) already doubles regeneration; free casts contradict "cantrips cost mana"; a "once a day" is a cooldown, which a mana ability never gets (coverage policy, step 4) |
| **5e warlock** | a pact and a patron; Eldritch Blast at will; invocations | the game is named for the Old English *wǣrloga*, the oath-breaker; the witch is the class nearest it (Hag's bargain is her pact). Not taken: the blast is Magic missile's, the invocations are talents |
| **ADOM** | wizards are guaranteed every spellbook they find readable; **magical writing sets** write a scroll of a spell you know for PP; the Necromancer raises undead; no witch | the writing set is Scribe's cost model: a known spell, mana, and a consumable. ADOM's loop of writing scrolls to sell is the one §13 q3 asks about |
| **NetHack** | the **magic marker** writes scrolls and spellbooks; Wizards have the best chance to write an unknown scroll; a failed write wastes the marker's ink | Scribe needs **no roll**: a failure that wastes a blank is busywork (as the magic design rejected NetHack's spell forgetting). The blank and the time are the price |
| **DCSS** | Hexes (Slow, Confuse, Petrify, Enslavement) and Necromancy (Pain, Vampiric Draining, Agony) as schools; the Necromancer start; no witch, the Wizard start a generalist | **Malison** (§5.2) gives hexes a bite that DCSS's Hexes school lacks (its hexes set up kills; ours also pay for them). A drain spell is a spell-list matter, deferred (§11) |
| **Anglo-Saxon wiccecræft** | *wicce* (f.) and *wicca* (m.), the practitioner; *hægtesse*, the hag, the hedge-rider; *wyrtgælstre*, the herb-charmer; *galdor*, the sung charm; *lyblāc*, sorcery by potions and poisons; the evil eye; *Wið færstice* against the hags' shot and elf-shot; the Nine Herbs Charm | the witch's names and shape are already hers: Herb lore (*wyrtgælstre*), Poison lore (*lyblāc*), Evil eye, Blight (the hags' shot), Hag's bargain. Cackle is her *galdor*: a charm **sung aloud**, which is why it is loud (§5.1). The early-modern witch (cauldron, familiar, pact) is kept where it is: Cauldron at 50, the bargain at 60, the familiar deferred |
| **The scholar-magician** | the *magister* of the schools, Faust and the grimoire; magic as reading, writing, and knowing | the wizard's names already fit (Magister, Arcane lore, Arcane training, Scroll adept); Scribe is the missing verb |

## 3. What exists today: the two classes side by side

Read from `assets/data/classes/09-wizard.toml` and `10-witch.toml` and their talents (`assets/data/talents/`):

| | Wizard | Witch |
|---|---|---|
| Description | "A scholar of the arcane who traded sunlight for candlelight: brilliant of mind, frail of body." | "A hedge-magician of herbs, hexes, and old bargains: cunning, wise in strange ways, and feared." |
| Stats | St −2, To −1d2, **In 2d2+1**, Wi +1 | St −2, To −1, In 1d3, Wi 1d3, Pr 1d3−2 |
| Hit points, mana | 0, **12** | 1, 10 |
| Casting, experience | 100 %, 125 | 100 %, 115 |
| Alignment | 0 | **−200 (Callous)** |
| Affinity (study) | evocation 2; abjuration, transmutation, enchantment, divination, necromancy 1; restoration −2 | **necromancy 3, enchantment 2**, transmutation 1; restoration −1 |
| Skills | Concentration, Knowledge, Spellcraft (known for), Decipher script; by chance Appraise, Search | **Heal**, Spellcraft (known for), Concentration, **Intimidate** (read by nothing yet); by chance Bluff, Handle animal, Knowledge |
| Weapon skills, style | staves 1, **evocation 2**; **Sniper** 1 | daggers 1, staves 1, **necromancy 1**; **Toucher** 1 |
| Kit | quarterstaff, cloak, scrolls of light and identify, one of fireball, frost, or lightning, 4 candles, bread, 15 gold | dagger, cloak, a **bone amulet (magic, unidentified)**, 2 angelica roots, a potion of healing or mana, 2 candles, 2 apples, 6 gold |
| Bonus picks | 10, 20, 30, 45, 55 from **Scholar, Spellcraft** | 10, 20, 30, 45, 55 from **Scholar, Grit** |
| Table | 1 Scroll adept, Ray of frost, Arcane training; 3 Magic missile; 4 Knock; 6 Arcane lore; 12 Burning hands; 18 Arcane bolt; 25 Mage armour; 32 Empower; **45 Magister**; 50 Arcane mastery; 60 Archmage (13 entries, 8 of 9 cadence levels) | 1 Herb lore, Evil eye; 3 Hex of weakness; 12 Poison lore, Cause wounds; 25 Second sight; 32 Curse of sloth; 40 Blight; 50 Cauldron; 60 Hag's bargain (10 entries, 7 of 9) |
| Guaranteed spells | Ray of frost (1), Magic missile (3), Burning hands (4), Arcane bolt (6): all **evocation**, all harm, two rolls to hit (Sniper's) | Evil eye (0, enchantment), Hex of weakness (3, necromancy), Curse of sloth (3, transmutation), Cause wounds (4, necromancy, a touch), Blight (7, necromancy, a burst): **three hexes**, a touch, a poison burst |
| Non-spell entries | Scroll adept (scrolls: −50 % time, +1 rung), Arcane training (caster level +1 per 5), Knock (locks, 5 mana), Arcane lore (identify what is carried an hour), Mage armour (Deflection +2 unarmoured), Empower and Archmage (rungs, mana off), Magister (every school a rank up), Arcane mastery (regeneration ×2) | Herb lore (rot −50 %, corpses' own harm never, the angelica root heals), Poison lore (Poisoned eases twice as fast, +2 on blows at the poisoned), Second sight (notice +2, searching +2), Cauldron (corpses: never harm, marks ×2, a fresh one heals), Hag's bargain (a reprieve once, Presence −2) |
| Pools (`docs/balance.md`) | 16 / 102 / 337 / 752 at levels 1 / 10 / 30 / 60 | 13 / 89 / 308 / 703 |
| School pace | evocation 5–6 at level 10, 8 at 30, 10 from 50 | necromancy 6–7 at 10, 9–10 at 30, 10 from 40 |

**So what already differs**: the wizard is Intelligence, evocation, the Sniper, the biggest pool, the Spellcraft tree, and scrolls and items; the witch is three mental stats, necromancy and enchantment, the Toucher, Heal and Intimidate, the Grit tree, corpses and herbs, Callous. The *shapes* differ well; what is missing is a **mechanic each class plays by**: the wizard's non-spell entries are all modifiers on casting, and the witch's are mostly about what she eats. Neither table has a verb of its own besides Knock.

**The hexes today**: `kind = "hex"` spells lay a status on the hostile seen at the aimed cell (`hex_target`) or on everyone in a burst (`cast_area_hex`), through `afflict(target, status, severity, Some(player))`: **the giver is recorded** (`Effect::by`), the severity is the spell's degrees (Presence's ladder, plus the school's and the hooks'), clamped to the status's `max` (3 for Hexed, Weakened, Sluggish, Drowsy; 6 for Webbed), stacking `refresh` (a second hex takes the higher severity and starts its wear again). Each degree is one span of `eases` (60 s for Hexed and Sluggish, 120 for Weakened, 12 for Drowsy, 6 for Webbed). `data/circles.toml` already lists every buff's and hex's status with its school, checked at load against the spells; the hex statuses are a known, closed set. Nothing reads "under a hex" as a condition today; Poison lore's `status_damage` reads "under Poisoned" for blows only (`World::lore_blow`, from `resolve`).

**Scrolls today**: every spell has one `scroll_of_ID` object (`spell_list_tests`: one scroll a spell), read through the cast path at the scroll's own `rungs` plus the reader's `scroll_rungs` (Scroll adept's 1), identified by reading, studied to learn the spell (`world/magic/learn.rs`: a multi-turn action through `World::keep_at`, interrupted by a creature coming into view or a blow, the scroll used up at the end). The scroll's rung is fixed data (the three old ones 1, 2, 0; the rest 0). There is no blank scroll and no way to make one.

## 4. The two fantasies

**The witch.** She is not a lesser wizard. She has fewer spells and they do not kill from afar: they *mark*. A fight for the witch opens with the evil eye and a hex of weakness on the thing in front of her, and is then won by the dagger, the touch, and time, while the hexed thing misses, weakens, and slows. Her two new entries make that a loop instead of an opening: Cackle keeps the marks on as long as she is willing to be heard (and being heard is the hedge-witch's own risk: what she calls to her comes), and Malison makes everything she does to a marked thing hurt more, and pays her hex back when the thing dies. She feeds on the fight. Herbs and corpses (Herb lore, Cauldron) feed her between fights; the bargain at 60 is the last price. The classes nearest her in play are the thief (a setup, then the payoff) and the barbarian (a resource that builds in the fight), not the wizard.

**The wizard.** He is the one who *knows*: more spells than any other class (Arcane training brings every level of scroll within certain reach sooner), what every item is (Arcane lore), what every lock wants (Knock), and every school a rank better than his practice (Magister). His two changes make preparation his verb: Scribe lets him turn a quiet hour and a full pool into scrolls, and open the next fight with more than his pool holds, read in half the time at a rung more (Scroll adept, his level-1 entry, finally has a supply it does not depend on luck for). The wizard who rests is working. The classes nearest him in play are the ranger (missiles, a quiver filled between fights) and the merchant (what he makes is also worth something).

**What a player feels at level 12.** A witch: Evil eye, Hex of weakness, Cause wounds, Cackle; a pool of about 106; she hexes two orcs, cackles once to hold it, and touches the weakened one, her Toucher ranks landing it. A wizard: Ray of frost, Magic missile, Burning hands, Scribe; the same pool; he has written a Fireball scroll after the last rest and opens on the pack with it for no mana, then Magic missile from the Sniper's distance. At 18 and 40 the loops deepen (Malison's tithe; Magister's rank).

## 5. The witch: hexes that stick and come home

Both entries read one new notion, **"under a hex"**: an actor has an `Effect` whose status is one a `kind = "hex"` spell lays. `Content::hex_statuses()` (a `Vec<StatusId>` computed at load from the spells, in status order; the same set `circles.toml`'s hex rows check against) is the one definition; "under a hex **of hers**" adds `Effect::by == Some(player)`. Both are pure reads of state; neither draws.

### 5.1 Cackle (level 6, active)

```toml
# assets/data/talents/cackle.toml
[[talent]]
id = "cackle"
name = "Cackle"
tree = "class"
granted = true
description = "You cackle, loud enough to be heard: every hex you have laid on a creature you see holds a while longer."
effects = [{ kind = "cackle", degrees = 1, noise = 60 }]
systems = ["status", "spells"]
```

| Rule | Decision | Where |
|---|---|---|
| **The action** | `Action::UseClass(ClassActive::Cackle)`, action text `use:cackle`, aim nothing; a row of the abilities menu under the class source ("Cackle (class): nothing to aim"), listed usable whenever held (the rules refuse, as Blood magic's row does) | `ability.rs`, `world/classes/class_ability.rs` → `world/classes/witch.rs::cackle` |
| **What it does** | for every creature the player's **current perception** sees, for every `Effect` on it whose status is in `hex_statuses()` and whose `by` is the player: `afflict(creature, status, severity + degrees, Some(player))`. `Effects::add` with `refresh` takes the higher severity, clamps it to the status's `max`, and sets `wear` to 0: so a hex below its max gains a degree **and** starts the new degree fresh, and one at its max is refreshed (its current degree's time starts again). Hexed 2 → 3 (a minute more, and the minute restarts); Webbed 6 stays 6 (six more seconds) | `world/classes/witch.rs` |
| **What it costs** | a turn: priced as `cast` (`price_of(ActionKind::Cast)`, 1000), no mana, no pool, no cooldown | `World::use_class` |
| **Noise** | `Act::Cackle` → `(SoundKind::Voice, the witch's cell, noise)`: 60, louder than a shout (`VOICE_NOISE` 50), far short of a howl (150). Creatures that hear it get `Mind::noise` and come to look, as at any voice; the sound layer shows `V` | `noise.rs` (one arm beside `Mutter`) |
| **Refused, no time** | without the ability ("You have no such ability."); when no creature in sight is under a hex of hers ("No one you see is under your hex."); while a status blocks `cast` (as a spell is) | `World::may_use_class` |
| **Marks** | none: no cast (no school marks: nothing lands), no skill use. §13 q5 asks whether it should mark Intimidate, which the witch trains and nothing reads | — |
| **Messages** | "You cackle." then, if any hex grew, "Your hexes hold." (one line, not one per creature); the creatures' lines are the statuses' own (a status refreshed says nothing; one that grew worse says nothing either: `Added::Worse` is already silent for creatures, their look-mode word unchanged: "hexed") | `perceive_events` |
| **Determinism** | no draws | — |

**Why a degree and not a span of time**: hexes are already in degrees (`Effects::add`, the Presence ladder, the schools' `degrees`); "a degree longer" is the only unit the system has, costs no new field, and means a different number of seconds for each hex by that hex's own `eases`, which is right (Web's six-second degrees are short because Web is strong).

**Why loud**: the *galdor* is sung. A silent Cackle would be a free action in all but time; a loud one is a choice, every time, between holding the hexes and calling the next room. The number (60) is data on the effect.

**Numbers**: Evil eye at level 6 (Presence about 11: 1 degree, 1 minute) on an orc; two Cackles over the next twenty turns hold it to the end of a thirty-turn fight for two turns and no mana. At Presence 14 (2 degrees) and necromancy 3 (+1 degree) her hexes start at 3 and Cackle only refreshes: she cackles once every ten turns to hold three hexes on three orcs for the whole fight. The abuse case, cackling every other turn, is three casts' worth of hexes held forever at the cost of half her turns and a shout every twelve seconds: the dungeon answers that.

### 5.2 Malison (level 18, passive)

```toml
# assets/data/talents/malison.toml
[[talent]]
id = "malison"
name = "Malison"
tree = "class"
granted = true
description = "A creature under a hex suffers more from your blows and your spells, and when one dies under your hex the hex comes home to you as mana."
effects = [
    { kind = "status_damage", status = "ill_wished", value = 3 },
    { kind = "status_damage", status = "weakened", value = 3 },
    { kind = "status_damage", status = "sluggish", value = 3 },
    { kind = "status_damage", status = "drowsy", value = 3 },
    { kind = "status_damage", status = "webbed", value = 3 },
    { kind = "hex_tithe", per_levels = 2 },
]
systems = ["status", "combat", "spells", "mana"]
```

| Rule | Decision | Where |
|---|---|---|
| **The bite** | the existing `status_damage` kind, one row per hex status (the five of `circles.toml`; a hex spell added later adds its row to this file: the statuses are named, not derived, so the talent's text is honest and the load check holds). `World::lore_blow` sums every row whose status the target has: a Hexed and Weakened orc takes +6. **Widened to spells**: `spell_strikes` calls `lore_blow` on its blow before Protection and resistance, as `resolve` does for a blow, a shot, or a throw. Poison lore (the other holder) gains the same: its +2 lands with Blight on the poisoned too, one line of description changed ("your blows and spells") | `caster.rs::lore_blow`, one call added in `spell.rs::spell_strikes` |
| **The tithe** | a new kind `hex_tithe { per_levels }`: when an actor dies (`World::dies`, every cause: a blow, a spell, poison's tick, a trap, another creature, starving) while under a hex whose `by` is the player, and the player holds the effect, the player's mana rises by `max(1, level / per_levels)` of the dead creature's level (`Content::level_of` the type), never past `max_mana`. No range, no line of sight: the hex is the tether. A kill by the witch or by anyone. Never for the player's own death. | `world/classes/witch.rs::hex_tithe`, one line in `World::dies` beside `siphon`'s callers (not inside `siphon`: that one is the kill point of an attack within three cells, this one is any death) |
| **Messages** | nothing for the bite (the damage is the blow's); "The hex comes home." when the tithe pays at least a point (said after the death's lines) | `perceive_events` |
| **Determinism** | no draws | — |

**Why +3 and not Poison lore's +2**: level 18 against level 12, and a hex is a cast the witch pays for where poison comes free on a blade; the two holders differ in status set, so neither the clone rule nor the chain rule is touched (`talent_alike`: different effects per rank). The owner accepts unbalanced choices; the balance probe's witch rows (`docs/balance.md` "Spells") gain a "hexed" column in W2.

**Why the tithe is by level**: a fixed point is nothing against a pool of 300 and everything against a pool of 13; half the level tracks the pool (a level-L witch's pool is about 10 L, §3) so that killing one hexed creature of her own level returns about a twentieth of her pool, two hexes' worth. The number is `per_levels` on the effect.

**What it is not**: not Siphoning (a worn affix, a kill within three cells, by the slayer alone), and not an XP or gold effect (rule 1). It stacks with Siphoning as two sources do.

## 6. The wizard: the prepared scholar

### 6.1 Scribe (level 12, active)

```toml
# assets/data/talents/scribe.toml
[[talent]]
id = "scribe"
name = "Scribe"
tree = "class"
granted = true
description = "Given a blank scroll, time, and three casts' worth of mana, you write a scroll of a spell you know."
effects = [{ kind = "scribe", mana_times = 3 }]
systems = ["scrolls", "spells", "mana"]
```

```toml
# assets/data/objects/blank_scroll.toml
# A blank scroll (spec §8 "Scrolls"): paper and nothing on it, until a wizard writes a spell on
# it (the Scribe ability). Read it (`r`) and nothing happens: there is nothing to read.
[[object]]
id = "blank_scroll"
name = "blank scroll"
plural = "blank scrolls"
glyph = "?"
fg = "#d8d0c0"
class = "scrolls"
materials = ["paper"]
weight = 50
value = 15
stackable = true
level = 1
frequency = 30
```

```toml
# assets/data/actions.toml
# A turn of writing a scroll (the wizard's Scribe ability; spec §8 "Scrolls"): a scroll takes as
# many turns as a study of its spell would (study.toml's turns_base and turns_per_level).
scribe = 1000
```

| Rule | Decision | Where |
|---|---|---|
| **The action** | `Action::Scribe { spell }`, text `scribe:SPELL` (`scribe:fireball`), a `ClassActive::Scribe` with `Aim::Spell`, a new aim kind: in the game the abilities menu row ("Scribe (class): a spell you know") opens a list of the spells known with each one's price and time (`ui/object_screen.rs`'s list style, as the Study list; `ObjectList::Scribe`; snapshot `scribe-list`), and the first blank scroll in the pack is the one written on | `ability.rs`, `action.rs`, `world/magic/scribe.rs`, `ui/` |
| **Refused, no time, no mana** | without the ability; a spell the player does not know (`Actor::knows_spell`); a spell no scroll object holds (`Content::scroll_of(spell)`: the object whose `read.spell` is it; every shipped spell has one); no blank scroll carried ("You have nothing to write on."); a status that blocks `cast`; mana short of the price ("You lack the mana to write it."); nothing about light or place (a scroll is read in the dark today, and written the same) | `World::start_scribe` |
| **The price** | mana: `Actor::scribe_cost(spell)` = `Actor::spell_base(spell)` (the cost table's price: level, shape, resolution, the armour worn; Empower's and Archmage's percents and the school's `mana` apply, as to any cast) × `mana_times`, then through `spend_mana` (Frugal caster's cut, the Willpower and Concentration marks, as every spend). Fireball 26 → 78; Magic missile 3 → 9; Identify 60 → 180. Time: `study_turns(rules, spell.level, 100)` turns of `scribe` (`study.toml`: 100 + 20 × level; Magic missile 160 turns, 16 minutes; Fireball 320, 32 minutes), Quick study's percent not applied (it is a study talent; §13 q4). The one blank is used up and the mana spent **at the end**, as a study's scroll is | `World::start_scribe` → `Spent::Scribe(Writing)` → `World::scribe` through `World::keep_at` |
| **Interrupted** | a creature coming into view, a message, a blow (`Ended::Came`/`Said`, as a study): "…you stop writing." Time lost; the blank kept; no mana spent | `World::scribe` |
| **The scroll** | the ordinary `scroll_of_ID` object, one, into the pack (the equipment's letters followed, as `use_scroll` does), at **its own `rungs`** (the magic is the item's: a scribed Fireball casts at rung 1 like a found one; Scroll adept adds his rung at the read, as to any scroll); its type **known** (`learn_scroll`: the writer knows what he wrote, "You write a scroll of fireball."); never a stack's charges or a wand | `World::finish_scribe` |
| **Marks** | the spend's own (Willpower, Concentration); an Intelligence mark (`StatUse::Scribe`, 100 %, as a study's); a Decipher script mark (`Use::Scribe`) if the skill gains a `mark` row for it (its `study` benefit is not read: the time is the spell's) | `World::finish_scribe` |
| **Who** | the wizard at 12. Nothing else gives `scribe`; a pickable is not minted (§10) | — |
| **Determinism** | no draws: the price is derived, the turns are counted, the scroll is made from a known id | — |

**Why a known spell only**: writing is the wizard's knowledge made portable. A scroll he can study is already a scroll; writing a spell he cannot cast would be a copy, which is NetHack's and rejected (§2).

**Why the scroll's own rung**: "the magic is the item's" (the spell scaling design §7.4, decision 12): a scroll's rung is its object's data, a wand's is drawn when it is made. A scribed scroll at the writer's own rung would be a fifth kind of roll on the menu's number and would make one object two things. Scroll adept's rung at the read is where the wizard's skill shows.

**Numbers**. Pools (§3): 106 at level 12, 214 at 20, 337 at 30, 752 at 60. A rest to full is about 3 hours at 12 and 7 at 30 (`docs/dev/spells.md` "Balance" 2). So a level-12 wizard writes one Fireball (78) or eight Magic missiles (72) per rest, a level-30 wizard four Fireballs (312), a level-60 wizard nine (Archmage's half: 13 × 3 = 39 each) in a night, and reads each in half a turn at a rung more. The burst a fight opens with roughly doubles at every level, bought with time outside the fight, which is the wizard's own currency. What it does to the gold economy is §13 q3.

### 6.2 Magister to 40

`{ level = 45, talent = "magister" }` becomes `{ level = 40, ... }` in `09-wizard.toml`. The talent is unchanged (`weapon { role = "school", ranks = 1 }`). `class::grant` gives a grown table's missing entries on `from_state`, so a saved wizard of level 40 to 44 gains Magister on load with the usual news. The spell scaling design's §8.6 wrote "level 45"; this document revises it, and `docs/design/README.md`'s note on that design says so when this one merges.

## 7. Every entry of both tables: stays, changes, goes

**Witch** (12 entries after; every cadence level filled):

| Level | Entry | After | Why |
|---|---|---|---|
| 1 | Herb lore | stays | *wyrtgælstre* |
| 1 | Evil eye (spell) | stays | her signature |
| 3 | Hex of weakness (spell) | stays | |
| **6** | — | **Cackle** (§5.1) | the gap; the hex loop's verb |
| 12 | Poison lore | **changes by one line**: its `status_damage` lands with her spells too (Blight on the poisoned) | the one read widened for Malison reaches it; its description says "blows and spells" |
| 12 | Cause wounds (spell) | stays | |
| **18** | — | **Malison** (§5.2) | the gap; the loop closes |
| 25 | Second sight | stays | witchy enough (`searching`; passes rule 8: it finds, it does not tell) |
| 32 | Curse of sloth (spell) | stays | |
| 40 | Blight (spell) | stays | |
| 50 | Cauldron | stays | kept by the owner in #787 |
| 60 | Hag's bargain | stays | the pact |

**Wizard** (14 entries after; every cadence level filled, 3 and 4 beside):

| Level | Entry | After | Why |
|---|---|---|---|
| 1 | Scroll adept, Ray of frost (spell), Arcane training | stay | the scholar's three |
| 3 | Magic missile (spell) | stays | |
| 4 | Knock | stays | off the cadence, as the gaps report accepts |
| 6 | Arcane lore | stays | |
| 12 | Burning hands (spell) | stays | |
| **12** | — | **Scribe** (§6.1) | Spell recall's old slot (#782), a scroll ability where a scroll ability was; the pool pays for it from here |
| 18 | Arcane bolt (spell) | stays | |
| 25 | Mage armour | stays | |
| 32 | Empower | stays | |
| **40** | — | **Magister** (from 45) | the gap, filled by the move |
| 45 | Magister | **moves to 40** | a bonus-pick level; the cadence's 40 was empty |
| 50 | Arcane mastery | stays | |
| 60 | Archmage | stays | |

Nothing goes. No id changes.

## 8. Determinism, saves, replays

| | Cackle | Malison | Scribe |
|---|---|---|---|
| Draws | none | none | none |
| Saved state | none new (statuses as any) | none (mana as any) | none: a scribing runs inside one action through `keep_at`, as a study does; nothing is in flight between actions |
| Content ids | talent `cackle` (not fingerprinted; saves name talents by id) | talent `malison` | talent `scribe`; object `blank_scroll` (objects are fingerprinted: **the content fingerprint changes**, saves do not, as every object added); action `scribe` |
| Old saves | a witch of 6+ gains Cackle on load (`class::grant`) | a witch of 18+ gains Malison | a wizard of 12+ gains Scribe; one of 40 to 44 gains Magister |
| Replays | unchanged unless they `use:cackle` | unchanged: the bite changes damage only where a hexed creature is hit after the talent is held, which no existing replay does (no replay reaches witch 18); the tithe likewise | unchanged unless they `scribe:` |
| `FORMAT_VERSION` | untouched (the integrator's) | | |

Hash order: Cackle's `afflict`s run in the perception's actor order (ascending id), the one order every area effect uses. The tithe is applied inside `dies` after the death's own bookkeeping and before `siphon`'s callers return, so a death that pays both pays the tithe first (the order is fixed, not drawn).

## 9. What the player sees; the wiki, the spec, the docs

| Where | What |
|---|---|
| The abilities menu | Cackle under the class source ("nothing to aim"); Scribe ("a spell you know") opening the scribe list; Malison nowhere (a passive, on the Talents page) |
| The scribe list (new) | the spells known, each with its mana price and its time in words ("about half an hour"), the blank scrolls carried in the header; empty: "You know no spell to write." (text snapshots `scribe-list`, `scribe-list-none`) |
| Messages | "You cackle." / "Your hexes hold." / "No one you see is under your hex."; "The hex comes home."; "You begin to write." / "…you stop writing." / "You write a scroll of fireball." / "You have nothing to write on." / "You lack the mana to write it." No number anywhere |
| Look mode | unchanged: "hexed", "weakened", "sluggish" as today |
| The sound layer | a cackle is a voice (`v`/`V`) at the witch's cell |
| Dumps | `abilities` gains the rows ("Cackle (class): nothing to aim", "Scribe (class): a spell you know"); `talents` lists Malison as any class passive; the `inventory` layer shows blank scrolls as scrolls; the header's `mp` shows the tithe |
| The wiki | `classes.md`'s witch and wizard prose rewritten to §4's two fantasies (linking hexes, statuses, scrolls, mana, schools per `wiki/TERMS.md`); the generated class tables (`class-levels.md`) pick the entries up; `scrolls.md` gains the blank scroll and a "Writing scrolls" section with the price rule (three times the spell's mana, the study's time); `status-effects.md`'s hexes note Cackle and Malison; `mana.md`'s sources gain the tithe |
| The spec | spec §8 "Class abilities" (witch: Cackle, Malison; wizard: Scribe, Magister at 40), "Scrolls" (blank scrolls, writing), "Status effects" (under a hex; the giver read by Cackle and the tithe) |
| `docs/dev/` | `progression.md`'s hook table (`cackle`, `hex_tithe`, `scribe`; `status_damage`'s read widened), `scrolls.md` (writing), `status-effects.md` (`hex_statuses`), `docs/design/README.md` (this document under "Approved, implementing"; the spell-scaling note on Magister's level) |

## 10. Talents and the coverage policy

No new system: the hex loop extends `status` and `spells` (its hooks read statuses and the spells' kinds), the tithe `mana`, the bite `combat`; writing extends `scrolls` (a sub-feature of scrolls adds no tag and no talent: CLAUDE.md's hard rule). No pickable is minted: a "writes cheaper" or "cackles quieter" talent is a small percentage on a niche mechanic (the policy's bar), and a pickable Cackle would be a clone of a class ability (rule 3). Scroll savant and Scroll adept already touch scrolls; Numbing cold and the school abilities already touch what spells lay.

Rules 1 to 9 of #776, each entry:

| Entry | 1 capstones | 2 fixed pool | 3 clones | 4 chains | 5 flaws | 6 outdoors | 7 hidden marks | 8 information | 9 favour |
|---|---|---|---|---|---|---|---|---|---|
| Cackle (6) | below 40; a fight | — | no other talent lengthens hexes | no prerequisite | — | works underground | no marks | acts, tells nothing | — |
| Malison (18) | below 40 | mana, not gold or XP | `status_damage` shared with Poison lore on **other** statuses (as Numbing cold's rider and Wasting's status differ): not alike at any rank | — | — | underground | the tithe is shown (the pool) | — | — |
| Scribe (12) | below 40; its pay is spells in a fight (the scrolls' sale is a side, §13 q3) | — | nothing else writes | — | — | underground | the marks are a study's | — | — |
| Magister (40) | changes every spell in a fight | — | kept distinct from Schooled (every school against one; #790 kept it) | — | — | — | — | — | — |

Tags match hooks: `cackle` reads statuses and the spells' kinds (`status`, `spells`); `malison` reads statuses (`status`), the blow's damage (`combat`), the spells' kinds (`spells`), and mana (`mana`); `scribe` reads scrolls (`scrolls`), spells (`spells`), and mana (`mana`). None is tagged `hearing` (Cackle's noise is the system it borrows, not the one it is for: the policy's Blaze example).

## 11. Rejected ideas

| Idea | Why not |
|---|---|
| **A familiar** (the witch's companion, holding her spells or fighting beside her) | an ally: the druid design (draft PR #828, §4 "The ally core", slice A1) builds the shared machinery (`Actor::ally` with `AllySource::Summoned | Companion`, the ally arm of the AI, the four orders under `Source::Ally`, following across levels and the retinue, XP from an ally's kill, `data/allies.toml`), and its §4.9 says what a class's own ally adds on top (the ranger's companion, #825, is the first). A witch's familiar is a third `AllySource` variant with its own data, a slice **after A1 lands**, with its own issue: how it is gained, what it holds, what its death costs. Nothing is asked of A1 now, and nothing shared is redesigned here. The witch's loop stands without it |
| **Brewing** (a Cauldron active: herbs or corpses into potions) | a crafting system with recipes, a fixture or a vessel, and a new item flow, for a class whose loop is the hex; Cauldron (50) keeps its name and its stomach. If brewing is ever wanted it is a system of its own (`potions`), not a witch entry |
| **Hexes at will** (Pathfinder's: no cost) | the owner's decision: cantrips cost mana; Evil eye at 1 mana is as near as the game goes |
| **Metamagic modes** (a casting stance: wider, longer, quicker, for more mana) | a fourth source on the menu's one number, which the spell scaling design's decision 11 held to the roll as cast; the six hooks are the talents' and items'; a mode is also a UI state across every cast. The 5e sorcerer's, not the wizard's |
| **Signature spell / spell mastery** (a chosen spell free) | a free cast contradicts "cantrips cost mana"; a once-a-day is a cooldown on a mana ability (coverage policy, step 4); School focus (−10 %) is the game's version |
| **Arcane recovery** (mana back once a rest) | Arcane mastery (50) doubles regeneration; a once-a-day is a cooldown |
| **Scribe at the writer's rung** | one object two things (§6.1) |
| **Scribe as a pickable or at level 1** (3.5's bonus feat) | a level-1 wizard's pool (16) writes nothing worth having; 12 is where it pays, and where a scroll ability sat |
| **Copying a scroll** (writing a spell from a scroll one cannot cast) | NetHack's unknown-scroll writing with a failure roll: busywork; and it makes study pointless |
| **A cackle that frightens** | Hound's bay (the Barghest's gift) is the game's fright; a second source shares one effect (rule 3's spirit) |
| **A drain spell** (necrotic harm that heals the caster: DCSS's Vampiric Draining) | a spell with a new `[spell.effect] drains` hook belongs to the spell list and its design, not to the class table; noted for the spells' tracker |
| **Herb lore on foraging** (the witch finds herbs oftener) | outdoors only (rule 6) and information-adjacent; the forage table is data a campaign tunes |
| **Intimidate as a witch mechanic** (a stare that cows) | the skill is read by nothing yet; its uses are an issue of the skills, not of this class (§13 q5 asks only whether Cackle marks it) |
| **Filling 40 with a new ability and keeping Magister at 45** | the wizard's table would be the fullest by two; the move costs one line and leaves no gap (§13 q1 keeps the alternative) |
| **Anything from the druid's or the priest's briefs** (summons; heal, cure, food, light, curses lifted) | theirs (#821, #820) |

## 12. Implementation plan (slices)

| Slice | What | Adds actions / saved state | Needs |
|---|---|---|---|
| **W1 Cackle** | `Content::hex_statuses()`; `cackle` talent and `ClassActive::Cackle`; `Act::Cackle` and its noise; `world/classes/witch.rs::cackle`, the refusals, the messages; the menu row; the `abilities` dump; tests (`world/classes/witch_tests.rs`: a hex grown a degree and refreshed at its max, another's hex untouched, a creature out of sight untouched, the refusals, the noise heard, no draws, the news on load of a level-6 save); the wiki and spec lines; the witch's class file | one action (`use:cackle`, the existing `UseClass` arm) / none | — |
| **W2 Malison** | `lore_blow` called from `spell_strikes` (Poison lore's description updated); `hex_tithe` kind, `TalentMods::hex_tithe`, `World::hex_tithe` in `dies`; the `malison` talent; tests (the bite on a hexed creature by blow and by Blight, none on an unhexed one, the tithe by every cause and never for the player's death, the cap at `max_mana`, no draws); the probe's hexed column; the wiki and spec | none / none | W1's `hex_statuses` (shared; W2 may land first with it) |
| **Z1 Scribe, Magister to 40** | `blank_scroll`; `scribe` in `actions.toml`; `Action::Scribe`, `ClassActive::Scribe`, `Aim::Spell`; `world/magic/scribe.rs` (`start_scribe`, `scribe`, `finish_scribe`, `Writing`); `Content::scroll_of(spell)`; the scribe list screen and its snapshots; the `abilities` dump; tests (`world/magic/scribe_tests.rs`: the price by spell and by Archmage, the time, interruption keeps the blank and the mana, the scroll made known at its own rung, every shipped spell writable, the refusals, no draws, a replay); the wizard's class file (Scribe at 12, Magister at 40); `read.rs`'s refusal for a blank ("There is nothing written on it."); the wiki's "Writing scrolls" and the spec | one action (`scribe:SPELL`) / none | — |

Order: W1 and Z1 in parallel (they share nothing), W2 after W1. One slice adds an action at a time per the integrator's rule (W1's rides the existing `UseClass`; Z1's is new). Magister's move may be cherry-picked into any earlier PR as one line.

## 13. Open questions, each with a recommended default

| # | Question | Default | The alternative |
|---|---|---|---|
| 1 | Magister to 40 and nothing new there, or a new wizard ability at 40 with Magister kept at 45? | **move it** (§6.2): one line, no gap, and the wizard's new mechanic (Scribe) sits at 12 where the pool pays for it | Scribe at 40 and Magister at 45: the mechanic arrives when few runs reach it |
| 2 | Scribe's price: three times the spell's mana? | **×3** (`mana_times`): one Fireball per rest at 12, four at 30 (§6.1) | ×2 makes writing nearly always better than casting; ×4 makes it a once-a-rest luxury |
| 3 | Shops buy scribed scrolls at the usual offer (a Fireball scroll is worth 200 gold; a level-30 wizard writes four a rest)? | **yes, no guard**: the owner's rule (no anti-farming limits; time is the penalty), and ADOM's writing sets run the same loop. The rate is stated here so it is a decision: about 600 gold a day at level 30 if every scroll is sold, against the Ascmearc's 2,200 quest gold | a scribed scroll carries no mark, so the only guard would be a flag on the stack ("a scroll in a fresh hand") and a tenth of the offer: a new saved field for an economy rule the owner has not asked for |
| 4 | Scribe's time: a study's (`study.toml`'s 100 + 20 × level turns), with or without Quick study's cut? | **a study's, without the cut**: no new knob, and Quick study is a learning talent | its own `turns` on the effect; or half a study's |
| 5 | Does Cackle mark Intimidate (the witch trains it; nothing reads it)? | **no**: Intimidate's uses are the skills' issue; a mark with no effect to train toward is noise | a `mark = [{ on = "cackle", chance = 100 }]` row on the skill, ready for its uses |
| 6 | Cackle's loudness 60 (a shout is 50, a howl 150)? | **60** | 50 (a voice) hides it among the dungeon's talk; 100 makes every cackle a call to the floor |
| 7 | Malison's bite +3 (Poison lore's is +2)? | **+3** | +2 for symmetry; or a die (`1d4`), which `status_damage` does not take |
| 8 | The tithe: half the dead creature's level, at least 1? | **half** (`per_levels = 2`) | a third; or the hex spell's level, which does not grow |
| 9 | `status_damage` widened to spells, so Poison lore's +2 lands with Blight too? | **yes**: one function from two places, and the witch's two lores agree | leave Poison lore as blows only and give Malison its own kind (`hex_damage`) read in both places |
| 10 | Cackle refuses with no time when no hex of hers is in sight, or goes off (loud, for nothing) as an area hex goes off on an empty room? | **refuses**: the player knows their own hexes (statuses are `seen`), so nothing leaks, and a wasted shout is a trap for a mis-key | goes off: simpler rules, a worse feel |
| 11 | A familiar for the witch once the ally core (#828 §4, slice A1) lands? | **yes, as its own issue then** (a third `AllySource` on A1's core, per #828 §4.9), not here | never |
| 12 | A drain spell (necrotic harm heals the caster) for the necromancy list? | **note it for the spells' tracker** (#759): a `[spell.effect] drains` percent on a damage spell, a new hook in `spell_strikes`; the witch's affinity (necromancy 3) makes it hers to learn | add it here as a sixth witch spell: it would be a spell-list change, which this issue is not about |

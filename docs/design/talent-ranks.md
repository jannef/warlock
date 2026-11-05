# Warlock — One rank per talent

Design document for the owner's review (#952). Second pass, 2026-11-05, after the owner's review of the first (2026-11-04). Written against `origin/main` at 4c7e06f2. Nothing here is implemented; §6 cuts it into issues. Every number below was read from the files named; anything not checked is marked *(unchecked)*. The balance probe was not run: the power notes reason from `docs/balance.md`.

**The owner's request** (#952, verbatim): *"get rid of multi rank talents. picking same thing twice is lame. additional ranks should be converted into talents that take the previous talent as prereq, and build on top without just doing the same same. If you can't figure out an interesting replacement, just drop the extra rank."* And on the first pass: *"I want the mechanic of having multiple ranks per talent gone from the game. Effects either need to be folded into 1 rank, or additional ranks need to be made independently named talents with prereqs if their existence is justified."*

## Second pass: what changed

The owner's comments on the first pass, each with what this pass does. Where the words left room, the reading taken is marked *(interpreted)*.

| Comment | Done |
|---|---|
| "get rid of apprentice talent … replace with skill focus that gives effective +10 or something, that can push the total skill above 100" | Apprentice is removed, with its `train_skill` effect kind. **Skill focus** (Scholar, choice: a skill you are trained in, the chosen skill counts 25 higher, past 100 too) takes its place: §2 "Skill focus", §3.7 for everything that named Apprentice. *(interpreted)*: the number is 25, not 10, because of how the skill curves round (§7, the one open question); and the choice is a trained skill only (§2 says why) |
| "double effect of armour training, seems too shit at the moment" | Armour training is one talent giving worn armour's Dexterity cap +2: §2 "Armour training", with what it does to Heavy armour mastery and the balance tables |
| "cleave: great cleave should be separate follow up talent" | kept as the first pass had it: Cleave, then Great cleave needing it |
| "split to 2 talents with dependency and different name" | read as the same instruction, which Cleave → Great cleave, Mighty kick → Breaker, and Old delver → Old hand already follow; nothing else was split *(interpreted)* |
| "just get rid of the 2nd rank effect, reducing gamba on the altars is not something i want after all" | Penitent is dropped. Pious is one talent (`devout`, `pilgrim`). The `penitent` effect kind, its hook, and its test go with it: §3.7 |
| "weapon focus: go with second tier being called weapon specialization" | Weapon focus → Weapon specialization stands, as `weapon { ranks = 1 }` on the chosen weapon skill (a weapon skill's only, never a style's or a school's) |
| "from the weapon style talents, just remove the follow-up talents - we will rework and expand weapon/style feats later" | **Anvil and hammer** and **Second blade's rend** are removed, nothing in their place: §2 "Fighting styles", a table the owner can check line by line. *(interpreted)*: no style talent names another talent in its `needs`, so "follow-up" is read as the second tier of a style's line (a style at rank 6 above an entry talent of the same style at rank 2). The rework of weapon and style feats comes later and is not designed here |
| §7's five questions | decided, no longer questions: Weapon specialization is `weapon { ranks = 1 }`; Breaker stays, with the kick-back; the chain rule is retired; the rank machinery is removed now (T5); Apprentice's dice are moot |
| (not a comment) | brought up to `main`: #954 (Silent feet folded into Light step, one rank) and #953 (Weapon finesse) merged, so the counts are 17 talents and 21 extra ranks, and the file and line references were re-read |

**Merged since the first pass, nothing here touches them**: Light step is one talent (#954: Finesse, Dexterity 12; `sure_step`, `pit_proof`, `footsteps_percent` −40, `silent_doors`; Silent feet deleted), and Weapon finesse is a base weapon's `finesse` flag (#953: `Attack::finesse`, Dexterity instead of Strength to hit in melee, docs/dev/combat.md 35), not a talent.

## Contents

1. Summary: every multi-rank talent and its verdict
2. The replacements, talent by talent (and the owner's cuts)
3. Inventory: everything else that depends on ranks, and what the cuts touch
4. What it does to total power and the bonus-pick budgets
5. The rank machinery: remove it
6. Implementation issues
7. Open questions

## 1. Summary

17 talents have `ranks` above 1 (`git grep '^ranks = [2-9]' origin/main -- assets/data/talents/`): 13 with two, 4 with three, **21 extra ranks in all: 6 become follow-up talents, 1 is folded into the one rank, 14 are dropped.** Three talents are removed whole (Apprentice, with its two extra ranks among the 14; Anvil and hammer; Second blade's rend) and one new talent that is no follow-up comes in (Skill focus). After this design no talent has more than one rank, and no `ranks`, `[[talent.rank]]`, `max_ranks`, or class-table `rank` remains (§5).

| Talent (tree) | Ranks | The one talent | Extra rank(s) → | New effect kinds |
|---|---|---|---|---|
| Hardy (Vigour) | 3 | hit points: 1 per 4 levels, at least 3 | **dropped** ×2; Unbowed now needs Hardy | — |
| Thick skin (Guard) | 3 | natural armour +1 | **dropped** ×2; Immune to pain now needs Thick skin | — |
| Deep well (Scholar) | 3 | mana: 1 per 4 levels, at least 2 | **dropped** ×2 | — |
| Apprentice (Scholar, choice: skill) | 3 per skill | **removed**; **Skill focus** (choice: a trained skill; it counts 25 higher) in its place | **dropped** ×2 (with the talent) | none (`skill`, its `skill` field now optional: the chosen one) |
| Brawn (Might) | 2 | melee damage +1 | **dropped** | — |
| Quick (Finesse) | 2 | speed +5 % | **dropped** | — |
| Keen ears (Shadow) | 2 | hearing +2 | **dropped** | — |
| Pack mule (Grit) | 2 | carrying +10 % | **dropped**; Porter now needs Pack mule | — |
| Armour training (Guard) | 2 | armour's Dexterity cap **+2** (the owner: doubled) | **folded** into the one rank; Heavy armour mastery now needs Armour training | — |
| Elemental ward (Guard, choice: element) | 2 per element | resist 25 % | **dropped** | — |
| Cleave (Might) | 2 | a free blow at a foe beside when a blow kills | **Great cleave** (new talent, needs Cleave, level 18): cleaving goes on, once at each foe beside | none (`free_blow great_cleave`) |
| Mighty kick (Might) | 2 | kicks roll 1d6, door kicks as if 4 stronger | **Breaker** (new talent, needs Mighty kick, Strength 14): a kick that lands drives the foe back a cell; digs a quarter faster | `kick_back` (new, small: calls the existing `World::knock_back`) |
| Old delver (Delver) | 2 | never falls through a cracked floor, Ebbing never takes you, nothing you kill rises in a cursed room | **Old hand** (new talent, needs Old delver, level 10): a room that quickens healing or mana quickens it half again more; one thing lifted from a tempting hoard before it springs | none (`room_pace_percent`, `hoard_grace`) |
| Pious (Wayfarer) | 2 | `devout`, `pilgrim` (the tree's free talent) | **dropped** (the owner: no Penitent); the `penitent` kind removed | — |
| Weapon focus (Weaponcraft, choice: weapon skill) | 2 per skill | +1 to hit with the chosen skill | **Weapon specialization** (new talent, same choice, needs Weapon focus for the same skill, chosen skill at 5): you fight with the chosen weapon as one a rank more practised | none (`weapon { ranks = 1 }`) |
| Scroll savant (Scholar) | 2 | reads twice as fast, own bursts never catch you | **Spell recall** (new talent, needs Scroll savant, Intelligence 15, level 12): a scroll read is not used up, one time in four | none (`scroll_kept`) |
| Jack of all trades (bard, class, granted) | 2 | every skill lacked taken up at 1d4+1 (bard 12) | **Master of none** (new class talent, granted at bard 25): every trained skill counts 5 higher | none (`trained_skills`) |

Three of the six follow-ups (Great cleave, Spell recall, Master of none) are the second ranks' effects under the names they had before #790 merged them (`docs/dev/progression.md` 387–389, 497–498; `spec/08i-character.md` 714, 748, 1430–1436). The drops are where the extra rank was the same number again: hit points, mana, +1, +5 %, +10 %, +25 %, and Penitent by the owner's word. Nothing new is invented to avoid a drop; the two kinds of talent the house rules forbid (information only, a niche percentage) are not proposed.

**The owner's cuts beyond the ranks** (§2, at the end): Anvil and hammer and Second blade's rend, the second-tier style talents, removed with nothing in their place.

## 2. The replacements, talent by talent

Each entry: what the file does now rank by rank, what the single talent keeps, and each extra rank's verdict. For a follow-up: name, tree, needs, effect in plain words, the effect kinds, and whether they exist (`assets/data/talents/README.toml` lists every kind; `docs/dev/progression.md` "The hooks" names each one's read).

### Hardy — `assets/data/talents/hardy.toml`

| | Now |
|---|---|
| Effect per rank | `max_hp_per_levels { per_levels = 4, at_least = 3 }`: hit points, one per 4 character levels, at least 3, handed out at the pick and grown each level (`talent.rs` 2346–2350 `per_levels_at`; `world/talent.rs` 292–296) |
| Rank needs | Toughness 11 / 14 / 17 |

**Keeps**: rank 1 (Toughness 11). **Ranks 2 and 3: dropped.** More of the same number; Vigour already has the different things that build on Hardy (Unbowed needs it; Second wind, Fast healer, Clotting are the tree's other hit-point talents). A follow-up that is not "more hit points" would be a new hit-point mechanic, and none earns its place beside those.

### Thick skin — `thick_skin.toml`

| | Now |
|---|---|
| Effect per rank | `natural { value = 1 }`: natural armour +1 (read in `Actor::defence`, on top of an amulet's) |
| Rank needs | Toughness 12 / 15 / 18 |

**Keeps**: rank 1 (Toughness 12). **Ranks 2 and 3: dropped.** Immune to pain (`pain 1`) is already the "builds on it" talent and moves to needing Thick skin.

### Deep well — `deep_well.toml`

| | Now |
|---|---|
| Effect per rank | `max_mana_per_levels { per_levels = 4, at_least = 2 }` (handed out as Hardy's hit points; `class::hand_out`) |
| Rank needs | Intelligence 12 or Willpower 12 / level 10 / level 25 |

**Keeps**: rank 1. **Ranks 2 and 3: dropped.** The Scholar tree's other mana talents (Clear mind, Meditation, Frugal caster, Blood magic) are the different things already.

### Apprentice — `apprentice.toml` (choice: any skill): removed

| | Now |
|---|---|
| Effect per rank | `train_skill { dice = "1d6+4" }`: the chosen skill trained at the roll, or raised by it, from the `levels` stream (`World::train_chosen`, `world/talent.rs` 315–332) |
| Rank needs | none; three ranks per skill (`docs/dev/progression.md` 642–643); the Scholar tree's one talent without prerequisites |

**Removed whole** (the owner), with `train_skill`, the effect kind it alone uses (`grep train_skill assets/data`: Apprentice and the README). Taking skills up by talent is gone; a skill is trained by a class, a race, a background, and the level-up rolls (docs/dev/skills.md). What replaces it:

### Skill focus — new, `skill_focus.toml`

*"You have a knack for one trade you know: your chosen skill counts higher than its training alone, however far that training goes."*

| | Skill focus |
|---|---|
| Tree | Scholar, Apprentice's place: the tree's one talent without prerequisites, so the coverage bar's `free_per_tree = 1` still holds (`TalentDef::is_free` reads `needs` only, `content/talent.rs` 2771; `coverage_gap` 2924–2931). Offered at creation as Apprentice was (every talent without needs is) |
| Choice | `choice = "skill"`, kept, with its meaning narrowed to **a skill you are trained in**: Apprentice was the choice's only user, so `Choice::Skill`'s words change ("a skill" → "a skill you are trained in", `content/talent.rs` 124) and `talent::may_take` (2506–2527) refuses an untrained one as a `Refusal::BadChoice` ("`stealth` is not a skill you are trained in."), the way a hidden style is a bad choice for Stylist today (`docs/dev/weapon-skills.md` 91–92). The screens list what `may_take` allows. One pick per skill, as every talent with a choice (`Talents::rank_of(id, choice)`) |
| Needs | none |
| Effect | `skill { value = 25 }`: the **existing** kind `skill { skill, value }` ("that skill counts that much higher", README 238; `TalentEffect::Skill`, `content/talent.rs` 562), which no talent uses yet, with `skill` made optional: left out, it is the chosen skill, as `weapon` without `skill` or `role` reads the choice (README 244–248). `TalentMods::add` already takes the choice (`talent.rs` 1323) and the Skill arm (1505–1512) resolves `skill.or(choice)`; the loader (`content/talent.rs` 1703–1707) asks for `skill` or a `skill` choice |
| The one hook | `Actor::skill_effect_above` (`world/actor.rs` 466–490), which every rule that reads a skill's benefit goes through (docs/dev/skills.md 73–75): `TalentMods::skills` is already put in the bonus list that `Skills::effect_above` (`skills.rs` 829–860) adds to a skill's value before its benefits are read. Nothing new is read anywhere |
| Past 100 | 100 is the **soft cap on growth**, not a ceiling on the value: a level-up roll past it adds half (`Skills::roll`, `skills.rs` 13, 792, `soft_cap`); a value runs 0 to `MAX_VALUE` 999 (47), and `effect_above` holds the value plus bonuses to 999, not 100. So a skill at 100 with Skill focus is read at 125, and the benefits keep paying there: every curve in `data/skills.toml` runs to 150 or 200 (`[[0, 0], [100, 8], [200, 12]]` for most uses; quiet `[100, 60], [150, 75]`; identify `[100, 12], [200, 20]`; mana regen `[100, 40], [200, 60]`), level only after the last point (`Benefit::at`, 558–583), and Tumble's Dodge steps go on every 25 past 100. A skill the affixes already push past 100 (the same bonus list) proves the path |
| With Master of none | both are bonuses in the one list, and `effect_above` sums every bonus a skill has: a bard at 25 with Skill focus (Stealth) reads Stealth 30 higher, a magic affix on top |
| Shown | as an affix's bonus is: the Skills screen shows the trained value (`own_skills`, `perception.rs` 1707–1716), the Talents page the talent with its choice, the wiki the number |
| Why trained only | an untrained skill does not exist for the actor: never shown, no marks, no rolls, and the "trained at all" gates read the trained set alone (`Skills::trains_effect`, 812–815: Spot's passive notice, Appraise's identify, the Steal, Heal, Disarm, and Pick lock abilities). A bonus from 0 (`effect_with` allows it, 821) would give the value benefits at 25 and nothing else: a pick that looks like Apprentice and is not |
| Systems | `skills` (Quick learner, War cry, and the bard's Jack of all trades are its others: the bar holds) |
| Alike | the `skill` kind against `trained_skills` (Master of none): different variants, no clone; nothing else raises a skill. The chain rule is retired (§5) |

**The number: 25, from the owner's "+10 or something".** The curves are straight lines rounded down between their points (`Benefit::at`), so a bonus pays in whole points of effect or not at all:

| Benefit shape (`data/skills.toml`) | +10 below 100 | +10 past 100 | +25 below 100 | +25 past 100 |
|---|---|---|---|---|
| most uses, `[0, 0], [100, 8], [200, 12]` (kick, search, spot, steal, heal, disarm, pick, intimidate, temper, initiative, surprise, slip, evade, detect traps) | +0.8: +1 or nothing | +0.4: nothing as often as not | +2, always | +1, always |
| Stealth's quiet, `[100, 60], [150, 75]` | +6 % | +3 % | +15 % | +7 % |
| Tumble's Dodge, steps 20…100, then every 25 | +1 or nothing | +1 or nothing | +1 or +2 | +1, always |
| Concentration's mana regen, `[100, 40], [200, 60]` | +4 % | +2 % | +10 % | +5 % |
| Negotiation's bargain, `[100, 20], [200, 30]` | +2 % | +1 % | +5 % | +2 % |

For scale: a skill affix gives 5–10 at item level 1 and 20–30 at 12 (`affixes/of_shadows.toml`), and a level-up roll on a skill adds +1 to 4d5 by its marks, halved past 100. +10 is a level-1 affix's worth and on most skills less than a point of effect; +25 is a top affix's worth, chosen, permanent, and one point of effect past the cap on every shape. Not more: it stacks with affixes and Master of none, and a pick should not outdo the best found item. §7.1 asks the owner to confirm 25.

### Brawn — `brawn.toml`

| | Now |
|---|---|
| Effect per rank | `melee_damage { value = 1 }` (`Actor::melee`, `off_hand_blow`) |
| Rank needs | Strength 12 / 15 |

**Keeps**: rank 1. **Rank 2: dropped.** The Might tree's different things exist (Crusher's penetration, Staggering blow, Iron grip, Armour breaker).

### Quick — `quick.toml`

| | Now |
|---|---|
| Effect per rank | `speed_percent { value = 5 }` (hidden in the game; `World::burdened`) |
| Rank needs | Dexterity 14 and level 9 / Dexterity 16 and level 24 |

**Keeps**: rank 1. **Rank 2: dropped.** Quick's rank 1 is the Fleet-footed birth gift's twin (`talents.toml` 24, `alike`); a second rank was only +5 % more.

### Keen ears — `keen_ears.toml`

| | Now |
|---|---|
| Effect per rank | `hearing { value = 2 }` (`Actor::hearing`, with Listen) |
| Rank needs | none / Listen 20 |

**Keeps**: rank 1 (no needs: the Shadow tree's free talent stays). **Rank 2: dropped.** The different thing on hearing, exact placement, is the ranger's Tracker (`exact_sounds`), a class ability.

### Pack mule — `pack_mule.toml`

| | Now |
|---|---|
| Effect per rank | `carry_percent { value = 10 }` (`burden::load`) |
| Rank needs | Strength 11 / 14 |

**Keeps**: rank 1. **Rank 2: dropped.** Porter (`ignore_burdened`) is the follow-up already and moves to needing Pack mule.

### Armour training — `armour_training.toml`: doubled

| | Now |
|---|---|
| Effect per rank | `max_dex { value = 1 }`: worn armour's Dexterity cap +1 (`Actor::defence`) |
| Rank needs | none / Strength 13 |

**One talent, `max_dex { value = 2 }`, no needs** (the owner: "double effect"; it stays the Guard tree's free talent). The description loses "less again at the second rank".

| | Armour training +2 |
|---|---|
| Read | `Actor::defence` (`world/actor.rs` 617–623): each worn piece's `max_dex`, plus its affix's and the talents', then the burden's cap (Unburdened none, Burdened 3, Strained 1, Overloaded 0: `Burden::max_dex`, `burden.rs` 112) chained after them, and the lowest wins; the Dexterity modifier (`floor((score − 10) / 2)`) is held to it |
| What +2 buys | the armours' caps: padded coat 8, leather jerkin 6, studded leather 5, chain mail 4, scale mail 3, tower shield 2, banded mail 1, plate 1 (`data/objects/*.toml`). Plate or banded mail 1 → 3: a Dexterity 16 fighter (+3) keeps the whole modifier in plate, where +1 kept Dexterity 14's; scale 3 → 5, chain 4 → 6. Nothing for Dexterity 13 or less, whose +1 fits every cap but a tower shield's over banded mail or plate, nor against the Overloaded cap of 0 |
| Heavy armour mastery | needs Armour training (one pick; rank 2's Strength 13 goes) and level 12 (§3.1): the pair is two picks for cap +2 and Protection +1 in heavy armour, where rank 1, rank 2, and the mastery were three. Stalwart needs Armour training as before |
| Alike | no other talent has `max_dex` (Nimble is `burden_dex`, the burden's cap); the affix table's row naming it (`xtask/src/wiki/magic.rs` 359) stays |
| Balance tables | `docs/balance.md` never names it. The probe's players take Random's creation picks and what `levelup:auto` offers first (787–790 on `main`), so a probe warrior may hold it; where one wears heavy armour with a Dexterity modifier above its cap, Defence moves a point. The rerun in T1 settles it |
| Tests | `talent_arms_tests.rs` 203–213 (two gives raise plate's cap 1 → 3; one give does after), `class_active_tests.rs` 199–200 ("You learn Armour training (1)." loses its number), the three snapshots naming it (§3.5) |

### Elemental ward — `elemental_ward.toml` (choice: element)

| | Now |
|---|---|
| Effect per rank | `resist { value = 25 }` to the chosen element (`Actor::resistances`, capped as gear is) |
| Rank needs | Toughness 12 / level 20 |

**Keeps**: rank 1, 25 % per element chosen. **Rank 2: dropped.** A follow-up that is not "more resistance" (a status of the element that never takes you, a spell of the element cast harder) would be niche for most builds or a caster's talent in the Guard tree; neither earns its place. Of the deep and Flame-hardened (birth gifts, `resist`) are untouched.

### Cleave — `cleave.toml`

| | Now |
|---|---|
| Rank 1 | `free_blow { trigger = "cleave" }`: when a blow kills, a free blow at once at another foe beside; needs Strength 13 and a melee or unarmed skill at 3 |
| Rank 2 | `free_blow { trigger = "great_cleave" }`, level 18: cleaving goes on while the blows kill, once at each foe beside in a turn (`World::second_blow`; two cleave sources make a great cleave, `TalentMods::add`) |

**Keeps**: rank 1 as Cleave. **Rank 2 → Great cleave** (Might; needs Cleave, level 18; `free_blow great_cleave`, an existing kind, no new hook). It does a different thing (the chain of blows, not a bigger number), which is why it was a talent of its own before #790 made it a rank. The `alike` pair Cleave / Cleaving strike (`talents.toml` 20) stays; the chain rule this reverses is retired (§5).

### Mighty kick — `mighty_kick.toml`

| | Now |
|---|---|
| Rank 1 | `kick_die { sides = 6 }`, `door_kick { value = 4 }`; no needs (the Might tree's free talent) |
| Rank 2 | `door_kick { value = 4 }` again, `dig_percent { value = 25 }`; Strength 14 |

**Keeps**: rank 1 as Mighty kick, the free talent. **Rank 2 → Breaker** (Might; needs Mighty kick, Strength 14; decided, with the kick-back): *"Your kicks send a foe reeling: one your kick lands on is driven a cell back, when there is room; and you dig a quarter faster."* Effects: `dig_percent 25` (exists) and **`kick_back` (new)**: a kick at a creature that hits drives it one cell along the kick's direction when that cell is free. The code exists: `World::knock_back` (`world/weapon_active.rs` 376–405, Hip throw's and Shield rush's `knockback` weapon abilities) moves a hit foe one cell when the cell is walkable, untrapped, unfrozen, and empty, no draw. The new kind is a `TalentEffect` variant, a `TalentMods` bool, and one read where a kick's blow resolves (`world/fight.rs`, the `Action::Kick` arm, after the strike), calling `knock_back` with no status. Its second `door_kick 4` is dropped: a bigger number of rank 1's thing. Why not drop Breaker whole: a "dig faster" talent alone is Tunneler (Delver, `dig_percent 50`, no needs) under another name; the kick-back is what makes it a different talent, and it reuses a mechanic the game already has.

### Old delver — `old_delver.toml`

| | Now |
|---|---|
| Rank 1 | `collapse_proof`, `room_status_immune ebbing`, `grave_warden`; Willpower 11 or Dexterity 12 |
| Rank 2 | `room_pace_percent 50`, `hoard_grace 1`; level 10 |

**Keeps**: rank 1 as Old delver. **Rank 2 → Old hand** (Delver; needs Old delver, level 10): *"You take what a strange place gives, gladly: a room that quickens healing or mana quickens it half again more for you, and you lift one thing from a tempting hoard before it springs."* Both kinds exist (`World::healing_rate`/`mana_rate` for the pace, `World::lift_in_hoard` for the grace). Rank 1 is immunities, rank 2 is gains: different things, and #775 built them as one talent only because of rule 4. The name is the owner's to change.

### Pious — `pious.toml`

| | Now |
|---|---|
| Rank 1 | `devout`, `pilgrim`; no needs (the Wayfarer tree's free talent) |
| Rank 2 | `penitent`; Willpower 12, level 10: a great bane drawn at a forgotten altar becomes nothing |

**Keeps**: rank 1 as Pious, the free talent; its description loses the "At the second rank you kneel as one who knows their debts" sentence. **Rank 2: dropped** (the owner: "reducing gamba on the altars is not something i want after all"). No Penitent. The `penitent` effect kind has no user left and goes with it: §3.7.

### Weapon focus — `weapon_focus.toml` (choice: weapon skill)

| | Now |
|---|---|
| Rank 1 | `weapon { to_hit = 1 }` with the chosen skill; the chosen skill at 2 |
| Rank 2 | `weapon { damage = 2 }`; the chosen skill at 5 (Weapon specialization before #790) |

**Keeps**: rank 1 as Weapon focus, per skill. **Rank 2 → Weapon specialization** (Weaponcraft; `choice = "weapon_skill"`; needs Weapon focus **taken for the same skill** and `chosen = 5`; decided): *"You fight with your chosen weapon as one a rank more practised than you are."* Effect `weapon { ranks = 1 }` on the chosen skill: its benefits (to-hit, damage, threat, speed, whatever the rank table gives, `weapon_skills.toml`) read one rank higher, never past 10 (`TalentMods::weapon_benefit`, `raised`; `talent.rs` 2223–2244). The kind exists and keys on the choice already (README 244–248: "else without `role` the chosen weapon skill"): it is what Stylist does for a style and Schooled for a school. Why not "+2 damage": that is a bigger flat number of rank 1's `weapon` effect; a rank higher is a different thing (it scales with the weapon's rank table and gives what the table gives at that rank) and keeps the choice, as the issue asks. The prerequisite on the same choice works today: "a talent with a choice may be needed only by one of the same choice, taken for the same" (README 27–29; `talent.rs` 2405–2411).

**A weapon skill's only.** The `weapon_skill` choice lists the skills that attack (`Choice::attacks`, `content/talent.rs` 151–153), never a style or a school, so Weapon specialization cannot be taken for a style and never doubles Stylist (`weapon { ranks = 1 }`, chosen style at 3) or Schooled. Those two stay as they are and get no follow-up (the owner: the style feats are reworked later).

### Scroll savant — `scroll_savant.toml`

| | Now |
|---|---|
| Rank 1 | `read_percent -50`, `spares_self`; Intelligence 13 |
| Rank 2 | `scroll_kept { chance = 25 }`; Intelligence 15, level 12 (Spell recall, the wizard's 12, before #775) |

**Keeps**: rank 1 as Scroll savant. **Rank 2 → Spell recall** (Scholar; needs Scroll savant, Intelligence 15, level 12; `scroll_kept 25`, exists: `read.rs::use_up`, one gameplay draw a read only with the talent). Reading faster and keeping the scroll are different things.

### Jack of all trades — `jack_of_all_trades.toml` (bard class ability, granted)

| | Now |
|---|---|
| Rank 1 | `take_up_skills { dice = "1d4+1", except = ["sneak_attack"] }`; bard 12 (`02-bard.toml` 44) |
| Rank 2 | `trained_skills { value = 5 }`; bard 25 (`02-bard.toml` 47, `rank = 2`: the one class-table row naming a rank, #872) |

**Keeps**: rank 1 as Jack of all trades at 12. **Rank 2 → Master of none** (tree `class`, `granted`, named by the bard's table at 25 in the rank-2 row's place; `trained_skills 5`, exists: `Actor::skill_effect_above`). *"Master of none, yet every trade you know comes a little easier: every skill you are trained in counts higher."* Taking up every skill and raising every skill are different things. ("Many trades" if the proverb's second half reads as a flaw; the bard design used "master of none" only for a cap it rejected, `docs/design/bard-jack-of-all-trades.md` 338, so the name is free.) With it, no class or prestige table names a rank (§3.2), which is what lets the `rank` field go.

### Fighting styles: the follow-up talents removed

**This set is an interpretation of the owner's words** ("from the weapon style talents, just remove the follow-up talents"), to be checked line by line; strike a line to keep the talent. No style talent names another talent in its `needs` (`grep 'talents = ' assets/data/talents/`: the twelve talent prerequisites in the game are Deflect missiles', Fast healer's, Fever-proof's, Father's sword's, Heavy armour mastery's, Immune to pain's, Meditation's, Porter's, Stalwart's, Trap sense's, Unbowed's, and Zealot's, none of the `styles` system), so by the letter the set is empty. Read as the **second tier of a style's line** (a talent asking for a style at rank 6, above the entry talent of the same style at rank 2; the wiki calls one "a talent for those deep in the style", `wiki/src/fighting-styles.md` 77), it is these two:

| Talent | Tree; needs | What it did | What references it | Replaced by |
|---|---|---|---|---|
| **Anvil and hammer** | Weaponcraft; Two Shields 6 (above Shield expert, Shield block 2, the entry: Shield +1 and every style's bash 10 likelier) | `weapon { skill = "two_shields", bash = 15 }`: the second shield's follow-up blow 15 points likelier (`World::other_shield`); `systems = ["styles"]` | `wiki/src/fighting-styles.md` 77; `docs/dev/weapon-skills.md` 92–93; `spec/08i-character.md` 1650–1651; `world/two_shield_tests.rs` 228–236 (`the_styles_talents_and_the_stylists_choice_wait_for_its_first_marks`, `ours = ["anvil_and_hammer"]`: the hidden style's talents stay unlisted until marks; the test keeps its Stylist half and loses its talent half); the generated wiki rows. No class, race, sign, or campaign names it | nothing |
| **Second blade's rend** | Finesse; Twin Blades 6 (above Two-weapon training, Twin Blades 2, the entry: both two-weapon penalties eased by 2) | `rend_off_hand`: Rend, Twin Blades' rank ability, rolls the off hand's dice too (`world/weapon_ability.rs` 279); `systems = ["weapon_abilities", "styles"]` | the kind `rend_off_hand`, its only user: `content/talent.rs` 940, `talent.rs` 522, README; `world/weapon_ability_tests.rs` 326 (`second_blades_rend_rolls_the_off_hand_too`); `world/body_talent_tests.rs` 459 (a comment); `xtask/src/wiki/weapon_abilities.rs` 184 (the Rend row's words); `docs/dev/weapon-abilities.md` 97, 205; `assets/data/talents.toml` 173 (the system's comment); `docs/balance.md` 910 (the `weapon_abilities` count, 5 → 4: Early mastery, Brace of steel, Practised technique, Drillmaster); `docs/design/combat-progression.md` 605, 655 (the record, untouched). No class, race, sign, or campaign names it | nothing |

**Kept**: Stylist, Two-weapon training, Shield expert (the entry talents), Fencing (the class's), Forge-born, Grasping arms, Tentacles (the stars'; the horoscope is kept whole, #776). The `styles` system keeps its talents, so the coverage bar holds. The rework and expansion of weapon and style feats comes later (the owner) and is not designed here.

## 3. Inventory: everything else that depends on ranks, and what the cuts touch

### 3.1 Talents whose `needs` name a rank above 1

| Talent | File:line | Now needs | New prerequisite |
|---|---|---|---|
| Heavy armour mastery | `assets/data/talents/heavy_armour_mastery.toml:6` | `armour_training = 2`, level 12 | **Armour training**, level 12 |
| Porter | `porter.toml:6` | `pack_mule = 2`, Toughness 12 | **Pack mule**, Toughness 12 |
| Unbowed | `unbowed.toml:6` | `hardy = 2`, level 20 | **Hardy**, level 20 |
| Immune to pain | `immune_to_pain.toml:6` | `thick_skin = 2`, Toughness 16, level 20 | **Thick skin**, Toughness 16, level 20 |

No other talent needs a rank above 1 (`grep -E 'talents = \{[^}]*= [2-9]' assets/`: these four). The `chosen = N`, `weapon_skills = { x = N }`, and `any_weapon_skill { rank }` needs are weapon-skill ranks, not talent ranks, and stay. No effect reads another talent's rank: `TalentMods` sums effects and never asks who holds what rank (`talent.rs` 137–165); the only per-rank readers are the two `1..=taken.rank` loops (§3.4).

### 3.2 Who grants, counts, or compares ranks

| Where | File:line | What | Change |
|---|---|---|---|
| The bard's table | `assets/data/classes/02-bard.toml:47` | `{ level = 25, talent = "jack_of_all_trades", rank = 2 }` | the row names `master_of_none` (§2); comment at 36–37 |
| Class tables' `rank` field | `assets/data/classes/README.toml:97–101`; `content/class.rs:110–117` (`rank: u8`), 155–167 (rank 0, twice, after a higher rank), 219–237 (`check_table`: a rank past the talent's), 557–560 (`ranks_to`) | the one row above is the only user | field removed (§5) |
| Prestige tables | `assets/data/prestige/README.toml:26`; `content/prestige.rs:75, 253–254, 281–289` (`Table::abilities` `(level, talent, rank)`, `ranks_to`) | the Hunter's one row names no rank (`hunter.toml`) | with the class field |
| `class::grant`, `held_already`, the news | `world/class.rs:5–8, 20–25 (hand_out(rank)), 61, 73–92 (every missing rank below), 106–122 (held at the rank named or higher), 142–153 ("Your Jack of all trades grows.")` | | collapse to one rank; the "grows" line goes |
| Races | `assets/data/races.toml` | grant `versatile` only, rank 1 (`world/racial.rs`) | nothing |
| Birth signs | `assets/data/horoscope.toml` `grants`; `world/stars.rs`; `Talents::regrant` (`talent.rs` 117–131, each at rank 1) | none of the 17 | nothing |
| Background stories | `assets/data/background/05-trial.toml:131–135` (`the_flux`: `{ talent = "hardy" }`, a boon at rank 1, `world/boons.rs` 171–185); `03-childhood.toml:27` `tags = ["quick"]` is a line tag, not the talent; `04-youth.toml` 105–201 "apprentice" ids and the `apprenticed` tag are story lines, not the talent | Hardy at rank 1 stays Hardy; the comment at 131 ("never told to one who holds it already") still holds | nothing |
| Campaign data | `assets/campaigns/` | names none of the 17, nor the three removed (grep) | nothing |
| Creation picks, `--talents` | `world/creation.rs`; `CharacterSpec::talents` | a pick is `id` or `id:choice`, never a rank | nothing (Skill focus, no needs, is offered as Apprentice was) |
| `Actor::boons`, altars | `world/boons.rs:132, 154, 171–185` | rank 1 | nothing |
| The story's conditions | `background.rs:94–95, 155` (`has(t, 1)`, `rank_of > 0`) | held or not | nothing (the `1` goes with the field) |

### 3.3 Talents a follow-up or a new talent would crowd

| Talent | Neighbours checked | Verdict |
|---|---|---|
| Great cleave | Cleaving strike (warrior, `free_blow cleave`; two sources make a great cleave, `talents.toml` 18–20) | as today: Cleaving strike plus Cleave already great-cleaves; Great cleave is for everyone else |
| Breaker | Tunneler (Delver, `dig_percent 50`), Stone sense / Stonewise (`dig_percent`), Lockbreaker (Delver: `door_kick`, `kick_noise_percent`, `smash_destroy_percent`), Roc's strength (`door_kick`), Hip throw and Shield rush (weapon abilities, `knockback`) | the kick-back is new to talents and reads a kick; the digging quarter is small beside Tunneler and stays as the second half of the talent |
| Weapon specialization | Stylist (`weapon { ranks = 1 }`, a style), Schooled (a school), Improved critical (`weapon { threat = 1 }`, chosen 7), Deadly aim (`weapon { role = "launcher", damage = 2 }`), Early mastery (`ability_ranks 1`: the *abilities* a rank sooner) | the same mechanism as Stylist on the other kind of row, and the choice kinds never overlap (§2); Improved critical and Deadly aim stay distinct (threat; launcher damage) |
| Skill focus | Master of none (`trained_skills 5`, every trained skill), the skill affixes (`affixes/of_*.toml`, items), Quick learner (`skill_rolls`), the cut Apprentice (`train_skill`) | one skill, a flat bonus, a pick: a different kind from each; the alike check's clone rule passes (different variants) |
| Old hand, Spell recall, Master of none | their old selves, cut by #775/#790 | nothing else does what they do (the alike check's clone rule will say so at load) |

### 3.4 The rank machinery (file:line)

| Piece | Where |
|---|---|
| Data: `ranks`, `[[talent.rank]]`, top-level `effects` applied per rank | `assets/data/talents/README.toml:17, 25–29, 50–56`; `assets/data/talents.toml:9–10` (`max_ranks = 3`) |
| Loader | `content/talent.rs:3` (doc), `44` (`DEFAULT_MAX_RANKS`), `2686–2714` (`Rank { needs, effects, effects_while }`, `TalentDef::ranks: Vec<Rank>`), `2741–2768` (`rank_count`, `needs_at`, `effects_at`, `effects_while_at`), `2771` (`is_free`: "its first rank"), `2793–2794`, `2826`, `2942` (`TalentFile::max_ranks`), `2984, 2992` (`TalentSpec::ranks`, `rank: Vec<RankSpec>`), `3169–3179` ("needs `x` at rank N, which it does not have"), `3329–3357`, `3432–3433` ("a talent has 1 to {max_ranks} ranks") |
| `Need::Talent(id, rank)` and its words | `content/talent.rs:2514, 2551–2556` ("{name} {rank}"); `talent.rs:2405–2411` |
| The alike check | `content/talent_alike.rs:23–28` (effects per rank), `30, 40–53` (clone per rank), `65–75` (`needed`: every rank's needs), `105–122` (**the chain rule**: "make it that talent's next rank") |
| Runtime | `talent.rs:36–41` (`Taken { id, rank, choice }`, saved and hashed), `80–89` (`rank_of`, `has`), `92–104` (`take`: one rank more), `108–115` (`hold`), `154–161` (mods summed `1..=taken.rank`), `223, 233, 244` (per-rank hp/mana/caster-level entries), `2452, 2465` (`Refusal::AtTop`, "You already have all of {name}."), `2491–2538` (`unmet`, `may_take`: the rank it would reach) |
| The pick | `world/talent.rs:288–311` (`take_talent`: `hand_out(rank)`, `effects_at(rank)`, "You learn {name} ({rank})."), `315–332` (`train_chosen`: goes with Apprentice) |
| Other per-rank readers | `world/spell_mods.rs:63–66` (`1..=taken.rank`); `ability.rs:369–372` and `content/weapon_ability.rs:497` (`effects_at(rank_count())`: the top rank's effects) |
| Perception | `perception.rs:165–176` (`OwnTalent { rank, ranks }`), `220–236` (`TalentView { ranks, held, needs }`: the *next* rank's needs), `2144–2152, 2187–2202` |
| Dumps | `dump.rs:79–80, 1012–1019` (`Hardy 2/3`), `2023–2024` (JSON `rank`, `ranks`); `crates/warlock-cli/src/main.rs:93` (help) |
| Screens | `crates/warlock/src/ui/talents_screen.rs:3–5, 33–34, 101, 108–112, 130–132, 215–229, 239–243` ("2/3"); `level_up_screen.rs:67, 201–204, 356` ("2/3" beside the name); `chargen_talents_screen.rs:152` ("listed even when their last rank is taken") |
| Wiki generator | `xtask/src/wiki/talents.rs:12–13, 33, 168–199` (class tables rank by rank, "Jack of all trades 2/2"), `262–306` (needs "Toughness 11 / 14 / 17"), `315–345` ("per rank", "rank N: …"), `352–357` (`rank_text`) |
| Coverage test | `content/talent.rs::tests` (3616 `max_ranks = 3` fixture, 3658–3662 `[[talent.rank]]`, 3674 `talents_load_with_ranks_needs_and_effects`, 3861) |

### 3.5 Tests and snapshots that take a talent twice, read a rank, or name a cut talent

| Test | File:line | What it does | After |
|---|---|---|---|
| `ranks_come_one_at_a_time_and_hardy_grows_with_the_levels` | `world/talent_tests.rs:157` | takes Hardy rank by rank | one Hardy, grown with the levels |
| (the perception test) `(hardy.held, hardy.ranks) == (1, 3)` | `world/talent_tests.rs:543` | reads `TalentView` | held or not |
| `each_talent_changes_the_rule_it_names`'s fixture | `world/talent_tests.rs:232–245` | gates Apprentice to elves (its three ranks per skill would keep a pick open to the last level) by rewriting its file text | the gate moves to Skill focus (one pick per trained skill keeps picks open the same way) |
| `weapon_focus_and_its_kind_sharpen_only_the_chosen_skill_rank_by_rank` | `world/talent_arms_tests.rs:94` | rank 2's damage | Weapon specialization's rank |
| `guard_talents_change_defence_and_protection` | `world/talent_arms_tests.rs:203–213` | two Armour training gives raise plate's cap 1 → 3 | one give does |
| `elemental_ward_resists_the_chosen_element_by_rank` | `world/talent_arms_tests.rs:283` | rank 2's 50 % | gone |
| `mighty_kick_and_crusher_change_kicks_and_its_second_rank_digs_faster` | `world/body_talent_tests.rs:222` | rank 2 | Breaker |
| `old_delver_lifts_one_stack_before_it_seals_at_its_second_rank` | `world/hoard_tests.rs:212` | rank 2 | Old hand |
| `old_delver_quickens_a_room_s_quickening_half_again_at_its_second_rank` | `world/room_tests.rs:581` | rank 2 | Old hand |
| `pious_at_its_second_rank_turns_a_great_bane_into_nothing_but_not_the_explosion` | `world/boons_tests.rs:348` | rank 2 | **deleted** with the `penitent` kind |
| `scroll_savant_now_and_then_keeps_the_scroll_at_its_second_rank` | `world/talent_wits_tests.rs:98` | rank 2 | Spell recall |
| `apprentice_trains_the_chosen_skill_at_the_pick` | `world/talent_wits_tests.rs:2, 194–220` | the pick trains Appraise | a Skill focus test: the chosen skill's benefit read 25 higher, past 100 too; an untrained choice refused; the sum with Master of none |
| `a_restored_bard_past_both_ranks_is_given_each_and_a_skill_added_later_is_taken_up` | `world/class_rogue_tests.rs:740, 760` | the bard's two ranks | Master of none at 25 |
| the Stalwart pick | `world/class_active_tests.rs:199–200` | "You learn Armour training (1)." | without the number |
| `second_blades_rend_rolls_the_off_hand_too` | `world/weapon_ability_tests.rs:326` | the cut talent | **deleted** with `rend_off_hand` |
| `the_styles_talents_and_the_stylists_choice_wait_for_its_first_marks` | `world/two_shield_tests.rs:228–236` | `ours = ["anvil_and_hammer"]` | the Stylist half stays, the talent half goes |
| `talent.rs` unit tests | `talent.rs:2646–2658` (`take("hardy") == 2`, the JSON `"rank":2`) | the serialized form | with the field |
| `content/class.rs` table checks | `content/class.rs:1166–1175` ("fencing rank 2, but it has 1", "rank 0") | | with the field |
| `talent_alike.rs` tests | `content/talent_alike.rs:207, 234` (the chain rule's cases) | | with the rule |
| The level-up screen's choice test | `crates/warlock/src/ui/level_up_screen.rs:568–580` (picks Apprentice, the choice list opens); `screen_snapshots.rs:812` (Apprentice's letter) | | Skill focus (the list: trained skills) |
| The dependents' tests | `talent_wits_tests.rs:228` (Porter), `talent_arms_tests.rs:221–236` (Heavy armour mastery), `:255` (Immune to pain), `body_talent_tests.rs:145` (Unbowed): each `give`s the talent straight onto `Actor::talents` (`talent_arms_tests.rs:50–52`: `talents.take`), no prerequisite taken | | nothing |
| Screen snapshots | `ui/snapshots/…talents-screen.snap:15` ("Hardy 1/3"), `…talents-screen-story.snap:18`, `…talents-screen-spellcraft.snap:15, 17` ("Pious 1/2", "Scroll savant 1/2"), `…level-up-talent.snap:11, 12, 19, 21, 25, 28, 37` (Brawn, Mighty kick, Weapon focus, Armour training, Pious, Apprentice "1/3", Pack mule), `…level-up-choice.snap:9` (Apprentice's choice list), `…chargen-talents-marked.snap:37` and `…chargen-talents-screen.snap:36` (Apprentice among the creation picks), `…scribe-list.snap:5–6` and `…abilities-menu-merchants-eye.snap:5–6` (`levelup:auto`'s "You learn Apprentice (appraise) (1)…(3)": the auto-picks move on) | | re-accepted: no fraction, Skill focus in Apprentice's place |
| The balance probe | `world/balance_probe.rs:66–67` (`levelup:auto` "takes the first it may") | its auto-picks change (Apprentice (appraise) was the first it took; a Hardy at rank 1 is no longer offered again; Armour training is worth +2), so the probe's tables move a little; rerun and update `docs/balance.md` (the issue asks it) | |

### 3.6 Docs, spec, and wiki prose

`spec/08i-character.md` 636–640 (the data), 697–698 ("Ranks come one at a time ('You learn Hardy (2).')"), 714, 720, 725, 745–748, 756–757 ("a rank" in the effect list; Armour training "+1 a rank" → +2), 873–879 (Scroll savant), 883–885 (Apprentice "three ranks" → Skill focus), 912 (Elemental ward), 960–974 (Old delver, Pious and Penitent), 1038–1045 (rule 4), 1068, 1340, 1430–1436 (the bard's two ranks), 1650–1651 (Anvil and hammer); `spec/07d-shops-and-rooms.md` 742 (Pious "two ranks", Penitent); `spec/06-input-and-ui.md` 537–538 (`talent:apprentice:stealth` → `talent:skill_focus:stealth`); `docs/dev/progression.md` 12, 14, 22, 387–399 (rule 4, the chain check), 475–479 (Pious), 497–498, 609–617 (a table row's rank), 642–643 (Apprentice); `docs/dev/special-rooms.md` 404, 777–780 (Penitent's hook); `docs/dev/weapon-skills.md` 92–93 and `docs/dev/weapon-abilities.md` 97, 205 (the cut style talents); `docs/balance.md` 787–790, 910; `wiki/src/talents.md` 5, 36, 49–57, 113 ("up to three", "Hardy (2)", Apprentice "any skill, trained or not", its ranks), `classes.md` 219–223, `combat.md` 228, `fighting-styles.md` 77, `special-rooms.md` 175, 325, `background.md` 53 ("at its first rank"); `assets/data/talents/README.toml` and `classes/README.toml` as above; `assets/data/talents.toml` 173.

### 3.7 The three talents and three effect kinds removed

| Removed | Its only user | Where it lives | What happens |
|---|---|---|---|
| Apprentice | — | `assets/data/talents/apprentice.toml`; §3.5's tests and snapshots; `spec/08i-character.md` 883–885, `spec/06-input-and-ui.md` 537–538; `docs/dev/progression.md` 642–643; `wiki/src/talents.md` 36, 57, 113; `docs/design/progression.md` 767, 869, 991, 1043 (the record, untouched) | deleted; Skill focus in its place; prose names Skill focus |
| `train_skill` kind | Apprentice | `README.toml` 177–178; `content/talent.rs` 425–428 (the variant), 1758–1763 (its checks: a `skill` choice, dice 1 to 100); `world/talent.rs` 315–332 (`train_chosen`, the `levels` draw at the pick) and its call in `take_talent`; `xtask/src/wiki/talents.rs` 853 (its words) | removed |
| Penitent (Pious's rank 2) | — | `pious.toml` 13–16; `docs/dev/progression.md` 475–479; `docs/dev/special-rooms.md` 404, 777–780; `spec/08i-character.md` 970–974; `spec/07d-shops-and-rooms.md` 742; `wiki/src/special-rooms.md` 175, 325; `docs/design/special-rooms.md` 416, 610, 628 (the record, untouched) | the rank goes; prose says Pious has no second rank |
| `penitent` kind | Pious's rank 2 | `README.toml` 218; `content/talent.rs` 545, 2439; `talent.rs` 418 (`TalentMods::penitent`), 1494; `world/pray.rs` 117–121 (the one read, in `prayer_answered`); `xtask/src/wiki/talents.rs` 979; `world/boons_tests.rs` 348 | removed, with the test |
| Anvil and hammer | — | §2 "Fighting styles" | deleted; the `weapon { bash }` field stays (Shield expert's) |
| Second blade's rend | — | §2 "Fighting styles" | deleted |
| `rend_off_hand` kind | Second blade's rend | `README.toml`; `content/talent.rs` 940; `talent.rs` 522 (`TalentMods::rend_off_hand`); `world/weapon_ability.rs` 279 (the one read, Rend's extra damage); `xtask/src/wiki/weapon_abilities.rs` 184; `world/weapon_ability_tests.rs` 326 | removed, with the test |

Saves: a talent removed is one a saved character may hold by id; saves are pinned to the game version (`docs/dev/saves.md`, #840; the memory rule), so no shim, as §5 says for `Taken::rank`.

## 4. What it does to power, and the bonus-pick budgets

**Totals** (the numbers from the files; the level-60 hit points from `docs/balance.md` 806, 883–884):

| Talent | All ranks, at level 60 | After | Against the untrained warrior's 215 hp at 60 (trained: 400) |
|---|---|---|---|
| Hardy | 3 × 15 = 45 hp (at level 12: 9; at 30: 21) | 15 (3; 7) | 21 % → 7 % (trained 11 % → 4 %) |
| Deep well | 45 mana | 15 | (the wizard's pool: *unchecked*) |
| Thick skin | natural armour +3 | +1 | on 2d12, each point of Defence is one step of the hit table (`combat::hit_ways`) |
| Brawn | melee damage +2 | +1 | |
| Quick | speed +10 % | +5 % | |
| Keen ears | hearing +4 | +2 | |
| Pack mule | carrying +20 % | +10 % | |
| Armour training | Dexterity cap +2, two picks (the second at Strength 13) | +2, one pick, no needs: **kept whole, half the price** | |
| Elemental ward | 50 % per element | 25 % | |
| Apprentice | 3 × (1d6+4) ≈ 22 points in one skill, from nothing | Skill focus: +25 on a trained skill, no dice, no cap | |
| Pious | Penitent: a great bane at a forgotten altar passed over | gone | |
| Anvil and hammer, Second blade's rend | the second shield's bash 15 likelier; Rend with both blades' dice | gone | |

Three Hardy picks were the cheapest hit points a build could buy (Toughness 11/14/17, no level); losing two of them is the one change a player will feel, and it is the owner's intent: the picks go to talents that do something. Everything else was a second helping of a small number. The six follow-ups cost what the second ranks cost today (the same needs), so no build loses a thing it could do but Penitent and the two style talents, which the owner cut; Armour training's build gains a pick.

**Bonus-pick budgets.** A class's bonus picks (`bonus_talents` in `assets/data/classes/*.toml`: five at 10, 20, 30, 45, 55; the thief's and warrior's eight) must be spent in the class's trees; a pick nothing fits is dropped (`settle_due`, `docs/dev/progression.md` 15). Pickable talents per tree on `main` (the files of each `tree`, `granted` ones left out), counting the ranks that go and the talents that come and go:

| Tree | Talents today | Extra ranks lost | Talents removed | Talents gained | After | Classes whose bonus picks draw on it |
|---|---|---|---|---|---|---|
| Vigour | 12 | Hardy 2 | — | — | 12 | berserker, druid, priest, paladin, ascetic |
| Guard | 9 | Thick skin 2, Armour training 1 (folded), Elemental ward 1 per element | — | — | 9 | paladin, warrior, craftsperson, the Hunter |
| Might | 9 | Brawn 1, Cleave 1, Mighty kick 1 | — | Great cleave, Breaker | 11 | berserker |
| Finesse | 11 | Quick 1 | Second blade's rend | — | 10 | ascetic, thief |
| Scholar | 10 | Deep well 2, Apprentice 2 per skill, Scroll savant 1 | Apprentice | Skill focus, Spell recall | 11 | witch, wizard, merchant |
| Grit | 10 | Pack mule 1 | — | — | 10 | priest, witch |
| Delver | 10 | Old delver 1 | — | Old hand | 11 | thief, aristocrat, craftsperson, merchant |
| Wayfarer | 4 | Pious 1 | — | — | 4 | druid, ranger |
| Shadow | 6 (Silent feet gone, #954) | Keen ears 1 | — | — | 6 | thief |
| Weaponcraft | 17 | Weapon focus 1 per skill | Anvil and hammer | Weapon specialization (per skill) | 17 | ranger, warrior, aristocrat, craftsperson, the Hunter |
| Spellcraft | 10 | — | — | — | 10 | wizard |

No budget gets tight enough to leave a pick unspendable: the smallest pair of bonus trees is the druid's Wayfarer + Vigour (4 + 12 talents for 5 picks) and the ranger's Wayfarer + Weaponcraft (4 + 17); the thief's eight draw on Finesse + Shadow + Delver (10 + 6 + 11); and Weapon focus, Weapon specialization, Elemental ward, Skill focus, and Sworn foe are one pick per choice besides. What tightens is the *sink*: today Vigour's five classes could put three bonus picks into Hardy and never choose; after, each bonus pick is a different talent. The Might tree loosens (two follow-ups) and the berserker is its only bonus class.

## 5. The rank machinery: remove it

**Decided: remove it, in its own PR right after the data lands (T5).** Once no talent has `ranks` above 1 and no table names a rank, the machinery is dead weight that every reader, screen, dump, and wiki line still carries: `Taken::rank` is saved and hashed, `TalentView::ranks`/`held`, `OwnTalent::rank`, the "2/3" in three screens and the dump, "You learn Hardy (2).", `Refusal::AtTop`, the per-rank loops in `talent.rs` 154 and `spell_mods.rs` 63, `effects_at(rank)` at nine call sites, the loader's two structs and three checks, the class table's `rank` field and its three checks, and the wiki's "per rank" and "rank N:" text (§3.4). Keeping it "for later" keeps a feature the owner has rejected one data line away from coming back. What the removal looks like:

| Now | After |
|---|---|
| `ranks = N`, `[[talent.rank]]` with its own `needs` and `effects`, `max_ranks` | gone: a talent is `needs` and `effects`, once. The loader refuses the old fields (`TalentSpec` is `deny_unknown_fields`, `content/talent.rs` 2978) |
| `needs = { talents = { hardy = 2 } }` | `needs = { talents = ["hardy"] }` (a list of ids); `Need::Talent(id)`; its words "Hardy" |
| `{ level = 25, talent = "x", rank = 2 }` | `{ level = 25, talent = "x" }`; `Table::abilities` `(level, talent)`; `ranks_to` → the ids |
| `Taken { id, rank, choice }` | `Taken { id, choice }`; `rank_of` → `holds(id, choice) -> bool`; `take` refuses one held (`Refusal::Held`, "You have {name} already.") |
| `effects_at(rank)`, `rank_count()`, `needs_at(rank)` | `effects`, `needs` |
| "Hardy 2/3", "1/2", `rank`/`ranks` in JSON and `TalentView` | gone |
| "You learn Hardy (2).", "Your Jack of all trades grows." | "You learn Hardy." only |
| the chain rule (`talent_alike.rs` 105–122) | **retired** (decided): it exists only to force a chain into ranks, the thing this design abolishes. The clone rule (two talents alike) stays, with the `alike` list, and still catches a follow-up that only repeats its parent. Listing the follow-up pairs in `alike` instead would make it carry exemptions to a rule nobody wants |
| wiki "per rank", "rank N:", "Toughness 11 / 14 / 17", "2/2" rows | one line per talent |

Saves and hashes: `Taken` is saved and hashed (`talent.rs` 5–7), so every run with a talent hashes differently and no old save loads; saves are pinned to the game version (`docs/dev/saves.md`, #840; the memory rule), so this is the ordinary cost of a release and needs no shim. The golden manifests build their replays from action text at run time (`cargo xtask golden`), so they need nothing, and the repository holds no `runs/` replay files (none to re-create).

Between T1 and T5 everything works: every talent at `ranks = 1`, which the loader accepts today, and the machinery shows nothing for a single-rank talent (`talents_screen.rs` 239–243 prints nothing for `ranks == 1`; `world/talent.rs` 309 says "You learn X." without the number).

## 6. Implementation issues

Small PRs, rules first (CLAUDE.md "Keep PRs small"). T2 and T4 touch other files than T1 but share the level-up and talents-screen snapshots and the wiki and spec prose, so they go in one at a time; the integrator orders them.

| # | PR | Contents | Depends on |
|---|---|---|---|
| T1 | **The ranks: one per talent, and the follow-ups** | the 16 files (the 17 less Apprentice, T2) to `ranks` 1 (or the field dropped) with their rank-1 needs and effects hoisted; Armour training's `max_dex` 2; Pious without its second rank, and the `penitent` kind, its hook in `pray.rs`, its wiki words, and its test removed (§3.7); the six new talent files (Great cleave, Breaker without `kick_back`, Old hand, Weapon specialization, Spell recall, Master of none); the four `needs` to the talent alone (§3.1); the bard's 25 row; the chain rule retired in `talent_alike.rs`; the tests of §3.5 rewritten for the follow-ups and the one Armour training; the talents-screen and level-up snapshots re-accepted; `spec/08i-character.md`, `spec/07d-shops-and-rooms.md`, `docs/dev/progression.md` (rule 4 struck, the class-table rank paragraph), `docs/dev/special-rooms.md`, the wiki prose of §3.6, `cargo xtask wiki`; the balance probe rerun and `docs/balance.md`'s tables | nothing (the Light step merge and Weapon finesse are in) |
| T2 | **Skill focus replaces Apprentice** | `skill_focus.toml`; `apprentice.toml` and the `train_skill` kind deleted (§3.7); `Choice::Skill`'s words and `may_take`'s trained-only refusal; the `skill` kind's `skill` field optional (the chosen skill) in the loader and `TalentMods::add`; the tests and snapshots of §3.5 naming Apprentice; `spec/08i-character.md` 883–885, `spec/06-input-and-ui.md` 537–538, `docs/dev/progression.md` 642–643, `docs/dev/skills.md` (the hook's one line), README 19–20 and 238, the wiki prose (`talents.md` 36, 57, 113), `cargo xtask wiki`; the probe rerun if T1's has not landed | T1 (the one-rank file shape), or beside it on the integrator's word |
| T3 | **Breaker's kick-back** | `kick_back` (`TalentEffect`, `TalentMods`, README line), the one read in `fight.rs`'s kick arm calling `World::knock_back` (made `pub(super)`), its test, the wiki row | T1 (or folded into T1 if small enough; it is one hook) |
| T4 | **The style follow-ups cut** | `anvil_and_hammer.toml` and `second_blades_rend.toml` deleted; the `rend_off_hand` kind, its read in `weapon_ability.rs`, its wiki words, and its test removed; `two_shield_tests.rs` 228–236 trimmed to Stylist's choice; `talents.toml` 173; `docs/dev/weapon-skills.md` 92–93, `docs/dev/weapon-abilities.md` 97, 205, `docs/balance.md` 910, `spec/08i-character.md` 1650–1651, `wiki/src/fighting-styles.md` 77, `cargo xtask wiki` | nothing; one PR with T1 if the integrator prefers |
| T5 | **Remove the rank machinery** | §5's table: loader, `Taken`, `Need::Talent`, the class and prestige tables' `rank`, the pick's words, perception, dumps, the three screens, the CLI help, the wiki generator, the READMEs, `talents.toml`'s `max_ranks`, `docs/dev/progression.md`'s module table, the spec's data paragraph | T1 and T2 (the content must be rank-free first) |

T1 and T2 are what the owner sees; T5 is mechanical. The balance probe (minutes) runs once, in whichever of T1 and T2 lands last.

## 7. Open questions

One. Everything else the first pass asked is decided above (§"Second pass").

1. **Skill focus: +25, or the +10 the owner named?** Recommended: 25 (§2 "Skill focus", the table). The skill curves are lines rounded down, so +10 is worth less than one point of effect on most skills and nothing past 100 as often as not; +25 is one point past the cap on every curve, two below it, and the worth of a top-tier skill affix. +20 would be the floor at which the curves below 100 still pay something every time, but past 100 it rounds to nothing on the common `[100, 8], [200, 12]` shape. If the owner wants 10, the design is the same with the number changed and the talent weaker than a level-1 affix.

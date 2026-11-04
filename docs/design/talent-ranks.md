# Warlock — One rank per talent

Design document for the owner's review (2026-11-04; #952). Written against `origin/main` at 340d45db. Nothing here is implemented; §6 cuts it into issues. Every number below was read from the files named; anything not checked is marked *(unchecked)*. The balance probe was not run: the power notes reason from `docs/balance.md`.

**The owner's request** (#952, verbatim): *"get rid of multi rank talents. picking same thing twice is lame. additional ranks should be converted into talents that take the previous talent as prereq, and build on top without just doing the same same. If you can't figure out an interesting replacement, just drop the extra rank."* And for this document: *"i want proposal for replacements and inventory of other talents affected"*.

**Already handled elsewhere** (assume both land; nothing here touches them): Light step and Silent feet are being merged into one single-rank Light step (Finesse, Dexterity 12: `sure_step`, `pit_proof`, footsteps 40 % quieter, `silent_doors`), Silent feet deleted; Weapon finesse (Dexterity for Strength on melee to-hit) is a `finesse = true` base-weapon flag in its own PR.

## Contents

1. Summary: every multi-rank talent and its verdict
2. The replacements, talent by talent
3. Inventory: everything else that depends on ranks
4. What it does to total power and the bonus-pick budgets
5. The rank machinery: remove it (recommendation)
6. Implementation issues
7. Open questions

## 1. Summary

19 talents have `ranks` above 1 (`grep '^ranks = [2-9]' assets/data/talents/`): 15 with two, 4 with three, 23 extra ranks in all. Two (Light step, Silent feet) are handled elsewhere, leaving **21 extra ranks: 7 become follow-up talents, 14 are dropped.**

| Talent (tree) | Ranks | Rank 1 keeps | Extra rank(s) → | New effect kinds |
|---|---|---|---|---|
| Hardy (Vigour) | 3 | hit points: 1 per 4 levels, at least 3 | **dropped** ×2; Unbowed now needs Hardy 1 | — |
| Thick skin (Guard) | 3 | natural armour +1 | **dropped** ×2; Immune to pain now needs Thick skin 1 | — |
| Deep well (Scholar) | 3 | mana: 1 per 4 levels, at least 2 | **dropped** ×2 | — |
| Apprentice (Scholar, choice: skill) | 3 per skill | the chosen skill trained or raised by 1d6+4 | **dropped** ×2 (one pick per skill) | — |
| Brawn (Might) | 2 | melee damage +1 | **dropped** | — |
| Quick (Finesse) | 2 | speed +5 % | **dropped** | — |
| Keen ears (Shadow) | 2 | hearing +2 | **dropped** | — |
| Pack mule (Grit) | 2 | carrying +10 % | **dropped**; Porter now needs Pack mule 1 | — |
| Armour training (Guard) | 2 | armour's Dexterity cap +1 | **dropped**; Heavy armour mastery now needs Armour training 1 | — |
| Elemental ward (Guard, choice: element) | 2 per element | resist 25 % | **dropped** | — |
| Cleave (Might) | 2 | a free blow at a foe beside when a blow kills | **Great cleave** (new talent, needs Cleave, level 18): cleaving goes on, once at each foe beside | none (`free_blow great_cleave`) |
| Mighty kick (Might) | 2 | kicks roll 1d6, door kicks as if 4 stronger | **Breaker** (new talent, needs Mighty kick, Strength 14): a kick that lands drives the foe back a cell; digs a quarter faster | `kick_back` (new, small: calls the existing `World::knock_back`) |
| Old delver (Delver) | 2 | never falls through a cracked floor, Ebbing never takes you, nothing you kill rises in a cursed room | **Old hand** (new talent, needs Old delver, level 10): a room that quickens healing or mana quickens it half again more; one thing lifted from a tempting hoard before it springs | none (`room_pace_percent`, `hoard_grace`) |
| Pious (Wayfarer) | 2 | `devout`, `pilgrim` (the tree's free talent) | **Penitent** (new talent, needs Pious, Willpower 12, level 10): a great bane at a forgotten altar passes over you | none (`penitent`) |
| Weapon focus (Weaponcraft, choice: weapon skill) | 2 per skill | +1 to hit with the chosen skill | **Weapon specialization** (new talent, same choice, needs Weapon focus for the same skill, chosen skill at 5): you fight with the chosen weapon as one a rank more practised | none (`weapon { ranks = 1 }`, Stylist's for a style) |
| Scroll savant (Scholar) | 2 | reads twice as fast, own bursts never catch you | **Spell recall** (new talent, needs Scroll savant, Intelligence 15, level 12): a scroll read is not used up, one time in four | none (`scroll_kept`) |
| Jack of all trades (bard, class, granted) | 2 | every skill lacked taken up at 1d4+1 (bard 12) | **Master of none** (new class talent, granted at bard 25): every trained skill counts 5 higher | none (`trained_skills`) |
| Light step (Finesse) | 2 | handled elsewhere (one talent) | — | — |
| Silent feet (Shadow) | 2 | handled elsewhere (merged) | — | — |

Four of the seven follow-ups (Great cleave, Penitent, Spell recall, Master of none) are the second ranks' effects under the names they had before #790 merged them (`docs/dev/progression.md` 387–389, 475–479, 497–498; `spec/08i-character.md` 1429–1433). The drops are where the extra rank was the same number again: hit points, mana, +1, +5 %, +10 %, +25 %. Nothing new is invented to avoid a drop; the two kinds of talent the owner's house rules forbid (information only, a niche percentage) are not proposed.

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

**Keeps**: rank 1 (Toughness 12). **Ranks 2 and 3: dropped.** Immune to pain (`pain 1`) is already the "builds on it" talent and moves to needing Thick skin 1.

### Deep well — `deep_well.toml`

| | Now |
|---|---|
| Effect per rank | `max_mana_per_levels { per_levels = 4, at_least = 2 }` (handed out as Hardy's hit points; `class::hand_out`) |
| Rank needs | Intelligence 12 or Willpower 12 / level 10 / level 25 |

**Keeps**: rank 1. **Ranks 2 and 3: dropped.** The Scholar tree's other mana talents (Clear mind, Meditation, Frugal caster, Blood magic) are the different things already.

### Apprentice — `apprentice.toml` (choice: any skill)

| | Now |
|---|---|
| Effect per rank | `train_skill { dice = "1d6+4" }`: the chosen skill trained at the roll, or raised by it, from the levels stream (`World::train_chosen`) |
| Rank needs | none; three ranks per skill (`docs/dev/progression.md` 642–643) |

**Keeps**: one pick per skill (no needs). **Ranks 2 and 3: dropped.** Taking it again for another skill stays, as every talent with a choice does. The dice stay 1d6+4 (open question 5).

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

**Keeps**: rank 1. **Rank 2: dropped.** Porter (`ignore_burdened`) is the follow-up already and moves to needing Pack mule 1.

### Armour training — `armour_training.toml`

| | Now |
|---|---|
| Effect per rank | `max_dex { value = 1 }`: worn armour's Dexterity cap +1 (`Actor::defence`) |
| Rank needs | none / Strength 13 |

**Keeps**: rank 1 (no needs: the Guard tree's free talent stays). **Rank 2: dropped.** Stalwart (needs Armour training 1 already) and Heavy armour mastery (now Armour training 1) are its follow-ups.

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

**Keeps**: rank 1 as Cleave. **Rank 2 → Great cleave** (Might; needs Cleave, level 18; `free_blow great_cleave`, an existing kind, no new hook). It does a different thing (the chain of blows, not a bigger number), which is why it was a talent of its own before #790 made it a rank. The `alike` pair Cleave / Cleaving strike (`talents.toml` 20) stays; see §3.5 for the chain rule this reverses.

### Mighty kick — `mighty_kick.toml`

| | Now |
|---|---|
| Rank 1 | `kick_die { sides = 6 }`, `door_kick { value = 4 }`; no needs (the Might tree's free talent) |
| Rank 2 | `door_kick { value = 4 }` again, `dig_percent { value = 25 }`; Strength 14 |

**Keeps**: rank 1 as Mighty kick, the free talent. **Rank 2 → Breaker** (Might; needs Mighty kick, Strength 14): *"Your kicks send a foe reeling: one your kick lands on is driven a cell back, when there is room; and you dig a quarter faster."* Effects: `dig_percent 25` (exists) and **`kick_back` (new)**: a kick at a creature that hits drives it one cell along the kick's direction when that cell is free. The code exists: `World::knock_back` (`world/weapon_active.rs` 376–405, Hip throw's and Shield rush's `knockback` weapon abilities) moves a hit foe one cell when the cell is walkable, untrapped, unfrozen, and empty, no draw. The new kind is a `TalentEffect` variant, a `TalentMods` bool, and one read where a kick's blow resolves (`world/fight.rs`, the `Action::Kick` arm, after the strike), calling `knock_back` with no status. Its second `door_kick 4` is dropped: a bigger number of rank 1's thing. Why not drop Breaker whole: a "dig faster" talent alone is Tunneler (Delver, `dig_percent 50`, no needs) under another name; the kick-back is what makes it a different talent, and it reuses a mechanic the game already has. Open question 2 if the owner would rather drop it.

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
| Rank 2 | `penitent`; Willpower 12, level 10 |

**Keeps**: rank 1 as Pious, the free talent. **Rank 2 → Penitent** (Wayfarer; needs Pious, Willpower 12, level 10; `penitent`, exists: a great bane drawn at a forgotten altar becomes nothing, `boons_tests.rs` 348). Its own name before #788 (`docs/dev/progression.md` 475–479).

### Weapon focus — `weapon_focus.toml` (choice: weapon skill)

| | Now |
|---|---|
| Rank 1 | `weapon { to_hit = 1 }` with the chosen skill; the chosen skill at 2 |
| Rank 2 | `weapon { damage = 2 }`; the chosen skill at 5 (Weapon specialization before #790) |

**Keeps**: rank 1 as Weapon focus, per skill. **Rank 2 → Weapon specialization** (Weaponcraft; `choice = "weapon_skill"`; needs Weapon focus **taken for the same skill** and `chosen = 5`): *"You fight with your chosen weapon as one a rank more practised than you are."* Effect `weapon { ranks = 1 }` on the chosen skill: its benefits (to-hit, damage, threat, speed, whatever the rank table gives, `weapon_skills.toml`) read one rank higher, never past 10 (`TalentMods::weapon_benefit`, `raised`; `talent.rs` 2223–2244). This is Stylist's effect for a style (`stylist.toml`: `weapon { ranks = 1 }`, chosen 3) and Schooled's for a school, so the kind exists and keys on the choice already (README 244–248: "else without `role` the chosen weapon skill"). Why not keep "+2 damage": that is a bigger flat number of rank 1's `weapon` effect; a rank higher is a different thing (it scales with the weapon's rank table and gives what the table gives at that rank) and keeps the choice, as the issue asks. The prerequisite on the same choice works today: "a talent with a choice may be needed only by one of the same choice, taken for the same" (README 27–29; `talent.rs` 2405–2411). Open question 1 keeps the +2 damage as the alternative.

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

**Keeps**: rank 1 as Jack of all trades at 12. **Rank 2 → Master of none** (tree `class`, `granted`, named by the bard's table at 25 in the rank-2 row's place; `trained_skills 5`, exists: `Actor::skill_effect`). *"Master of none, yet every trade you know comes a little easier: every skill you are trained in counts higher."* Taking up every skill and raising every skill are different things. ("Many trades" if the proverb's second half reads as a flaw.) With it, no class or prestige table names a rank (§3.2), which is what lets the `rank` field go.

## 3. Inventory: everything else that depends on ranks

### 3.1 Talents whose `needs` name a rank above 1

| Talent | File:line | Now needs | New prerequisite |
|---|---|---|---|
| Heavy armour mastery | `assets/data/talents/heavy_armour_mastery.toml:6` | `armour_training = 2`, level 12 | **Armour training** (1), level 12 |
| Porter | `porter.toml:6` | `pack_mule = 2`, Toughness 12 | **Pack mule** (1), Toughness 12 |
| Unbowed | `unbowed.toml:6` | `hardy = 2`, level 20 | **Hardy** (1), level 20 |
| Immune to pain | `immune_to_pain.toml:6` | `thick_skin = 2`, Toughness 16, level 20 | **Thick skin** (1), Toughness 16, level 20 |

No other talent needs a rank above 1 (`grep -E 'talents = \{[^}]*= [2-9]' assets/`: these four). The `chosen = N`, `weapon_skills = { x = N }`, and `any_weapon_skill { rank }` needs are weapon-skill ranks, not talent ranks, and stay. No effect reads another talent's rank: `TalentMods` sums effects and never asks who holds what rank (`talent.rs` 137–165); the only per-rank readers are the two `1..=taken.rank` loops (§3.4).

### 3.2 Who grants, counts, or compares ranks

| Where | File:line | What | Change |
|---|---|---|---|
| The bard's table | `assets/data/classes/02-bard.toml:47` | `{ level = 25, talent = "jack_of_all_trades", rank = 2 }` | the row names `master_of_none` (§2); comment at 36–37 |
| Class tables' `rank` field | `assets/data/classes/README.toml:97–101`; `content/class.rs:110–117` (`rank: u8`), 155–167 (rank 0, twice, after a higher rank), 219–237 (`check_table`: a rank past the talent's), 557–560 (`ranks_to`) | the one row above is the only user | field removed (§5) |
| Prestige tables | `assets/data/prestige/README.toml:26`; `content/prestige.rs:75, 253–254, 281–289` (`Table::abilities` `(level, talent, rank)`, `ranks_to`) | the Hunter's one row names no rank (`hunter.toml`) | with the class field |
| `class::grant`, `held_already`, the news | `world/class.rs:5–8, 20–25 (hand_out(rank)), 61, 73–92 (every missing rank below), 106–122 (held at the rank named or higher), 142–153 ("Your Jack of all trades grows.")` | | collapse to one rank; the "grows" line goes |
| Races | `assets/data/races.toml` | grant `versatile` only, rank 1 (`world/racial.rs`) | nothing |
| Birth signs | `assets/data/horoscope.toml` `grants`; `world/stars.rs`; `Talents::regrant` (`talent.rs` 117–131, each at rank 1) | none of the 19 | nothing |
| Background stories | `assets/data/background/05-trial.toml:131–135` (`the_flux`: `{ talent = "hardy" }`, a boon at rank 1, `world/boons.rs` 171–185); `03-childhood.toml:27` `tags = ["quick"]` is a line tag, not the talent | Hardy at rank 1 stays Hardy; the comment at 131 ("never told to one who holds it already") still holds | nothing |
| Campaign data | `assets/campaigns/` | names none of the 19 (grep) | nothing |
| Creation picks, `--talents` | `world/creation.rs`; `CharacterSpec::talents` | a pick is `id` or `id:choice`, never a rank | nothing |
| `Actor::boons`, altars | `world/boons.rs:132, 154, 171–185` | rank 1 | nothing |
| The story's conditions | `background.rs:94–95, 155` (`has(t, 1)`, `rank_of > 0`) | held or not | nothing (the `1` goes with the field) |

### 3.3 Talents a follow-up would crowd

| Follow-up | Neighbours checked | Verdict |
|---|---|---|
| Great cleave | Cleaving strike (warrior, `free_blow cleave`; two sources make a great cleave, `talents.toml` 18–20) | as today: Cleaving strike plus Cleave already great-cleaves; Great cleave is for everyone else |
| Breaker | Tunneler (Delver, `dig_percent 50`), Stone sense / Stonewise (`dig_percent`), Lockbreaker (Delver: `door_kick`, `kick_noise_percent`, `smash_destroy_percent`), Roc's strength (`door_kick`), Hip throw and Shield rush (weapon abilities, `knockback`) | the kick-back is new to talents and reads a kick; the digging quarter is small beside Tunneler and may be cut from Breaker if the owner prefers the kick alone |
| Weapon specialization | Stylist (`weapon { ranks = 1 }`, a style), Schooled (a school), Improved critical (`weapon { threat = 1 }`, chosen 7), Deadly aim (`weapon { role = "launcher", damage = 2 }`), Early mastery (`ability_ranks 1`: the *abilities* a rank sooner) | the same mechanism as Stylist on the other kind of row; Improved critical and Deadly aim stay distinct (threat; launcher damage) |
| Old hand, Penitent, Spell recall, Master of none | their old selves, cut by #775/#788/#790 | nothing else does what they do (the alike check's clone rule will say so at load) |

### 3.4 The rank machinery (file:line)

| Piece | Where |
|---|---|
| Data: `ranks`, `[[talent.rank]]`, top-level `effects` applied per rank | `assets/data/talents/README.toml:17, 25–29, 50–56`; `assets/data/talents.toml:9–10` (`max_ranks = 3`) |
| Loader | `content/talent.rs:3` (doc), `44` (`DEFAULT_MAX_RANKS`), `2686–2714` (`Rank { needs, effects, effects_while }`, `TalentDef::ranks: Vec<Rank>`), `2741–2768` (`rank_count`, `needs_at`, `effects_at`, `effects_while_at`), `2793–2794`, `2826`, `2942` (`TalentFile::max_ranks`), `2984, 2992` (`TalentSpec::ranks`, `rank: Vec<RankSpec>`), `3169–3179` ("needs `x` at rank N, which it does not have"), `3329–3357`, `3432–3433` ("a talent has 1 to {max_ranks} ranks") |
| `Need::Talent(id, rank)` and its words | `content/talent.rs:2514, 2551–2556` ("{name} {rank}"); `talent.rs:2405–2411` |
| The alike check | `content/talent_alike.rs:23–28` (effects per rank), `30, 40–53` (clone per rank), `65–75` (`needed`: every rank's needs), `105–122` (**the chain rule**: "make it that talent's next rank") |
| Runtime | `talent.rs:36–41` (`Taken { id, rank, choice }`, saved and hashed), `80–89` (`rank_of`, `has`), `92–104` (`take`: one rank more), `108–115` (`hold`), `154–161` (mods summed `1..=taken.rank`), `223, 233, 244` (per-rank hp/mana/caster-level entries), `2452, 2465` (`Refusal::AtTop`, "You already have all of {name}."), `2491–2538` (`unmet`, `may_take`: the rank it would reach) |
| The pick | `world/talent.rs:288–311` (`take_talent`: `hand_out(rank)`, `effects_at(rank)`, "You learn {name} ({rank}).") |
| Other per-rank readers | `world/spell_mods.rs:63–66` (`1..=taken.rank`); `ability.rs:369–372` and `content/weapon_ability.rs:497` (`effects_at(rank_count())`: the top rank's effects) |
| Perception | `perception.rs:165–176` (`OwnTalent { rank, ranks }`), `220–236` (`TalentView { ranks, held, needs }`: the *next* rank's needs), `2144–2152, 2187–2202` |
| Dumps | `dump.rs:79–80, 1012–1019` (`Hardy 2/3`), `2023–2024` (JSON `rank`, `ranks`); `crates/warlock-cli/src/main.rs:93` (help) |
| Screens | `crates/warlock/src/ui/talents_screen.rs:3–5, 33–34, 101, 108–112, 130–132, 215–229, 239–243` ("2/3"); `level_up_screen.rs:67, 201–204, 356` ("2/3" beside the name); `chargen_talents_screen.rs:152` ("listed even when their last rank is taken") |
| Wiki generator | `xtask/src/wiki/talents.rs:12–13, 33, 168–199` (class tables rank by rank, "Jack of all trades 2/2"), `262–306` (needs "Toughness 11 / 14 / 17"), `315–345` ("per rank", "rank N: …"), `352–357` (`rank_text`) |
| Coverage test | `content/talent.rs::tests` (3616 `max_ranks = 3` fixture, 3658–3662 `[[talent.rank]]`, 3674 `talents_load_with_ranks_needs_and_effects`, 3861) |

### 3.5 Tests and snapshots that take a talent twice or read a rank

| Test | File:line | What it does |
|---|---|---|
| `ranks_come_one_at_a_time_and_hardy_grows_with_the_levels` | `world/talent_tests.rs:157` | takes Hardy rank by rank |
| (the perception test) `(hardy.held, hardy.ranks) == (1, 3)` | `world/talent_tests.rs:553` | reads `TalentView` |
| `weapon_focus_and_its_kind_sharpen_only_the_chosen_skill_rank_by_rank` | `world/talent_arms_tests.rs:94` | rank 2's damage → Weapon specialization's rank |
| `elemental_ward_resists_the_chosen_element_by_rank` | `world/talent_arms_tests.rs:283` | rank 2's 50 % → gone |
| `mighty_kick_and_crusher_change_kicks_and_its_second_rank_digs_faster` | `world/body_talent_tests.rs:222` | → Breaker |
| `old_delver_lifts_one_stack_before_it_seals_at_its_second_rank` | `world/hoard_tests.rs:212` | → Old hand |
| `old_delver_quickens_a_room_s_quickening_half_again_at_its_second_rank` | `world/room_tests.rs:581` | → Old hand |
| `pious_at_its_second_rank_turns_a_great_bane_into_nothing_but_not_the_explosion` | `world/boons_tests.rs:348` | → Penitent |
| `scroll_savant_now_and_then_keeps_the_scroll_at_its_second_rank` | `world/talent_wits_tests.rs:98` | → Spell recall |
| `a_restored_bard_past_both_ranks_is_given_each_and_a_skill_added_later_is_taken_up` | `world/class_rogue_tests.rs:740, 760` | → Master of none at 25 |
| `talent.rs` unit tests | `talent.rs:2646–2658` (`take("hardy") == 2`, the JSON `"rank":2`) | the serialized form |
| `content/class.rs` table checks | `content/class.rs:1166–1175` ("fencing rank 2, but it has 1", "rank 0") | go with the field |
| `talent_alike.rs` tests | `content/talent_alike.rs:207, 234` (the chain rule's cases) | go with the rule |
| The dependents' tests | `talent_wits_tests.rs:228` (Porter), `talent_arms_tests.rs:221–236` (Heavy armour mastery), `:255` (Immune to pain), `body_talent_tests.rs:145` (Unbowed): each `give`s the talent straight onto `Actor::talents` (`talent_arms_tests.rs:50–52`: `talents.take`), no prerequisite taken | nothing |
| Screen snapshots | `ui/snapshots/…talents-screen.snap:15` ("Hardy 1/3"), `…talents-screen-story.snap:18`, `…talents-screen-spellcraft.snap:15, 17` ("Pious 1/2", "Scroll savant 1/2"), `…level-up-talent.snap:11, 12, 19, 21, 25, 37` (Brawn, Mighty kick, Weapon focus, Armour training, Pious, Pack mule "1/2") | re-accepted without the fraction |
| The balance probe | `world/balance_probe.rs:66–67` (`levelup:auto` "takes the first it may") | its auto-picks change (a Hardy at rank 1 is no longer offered again), so the probe's tables move a little; rerun and update `docs/balance.md` (the issue asks it) |

### 3.6 Docs, spec, and wiki prose

`spec/08i-character.md` 636–640 (the data), 697–698 ("Ranks come one at a time ('You learn Hardy (2).')"), 714, 720, 725, 745–748, 755–756 ("a rank" in the effect list), 872–878 (Scroll savant), 882–884 (Apprentice "three ranks"), 911 (Elemental ward), 959–969 (Old delver, Pious), 1037–1044 (rule 4), 1067, 1339, 1429–1433 (the bard's two ranks); `docs/dev/progression.md` 12, 14, 22, 387–399 (rule 4, the chain check), 609–617 (a table row's rank), 642–643; `docs/balance.md` 783–786; `wiki/src/talents.md` 5, 49–57 ("up to three", "Hardy (2)", Apprentice's ranks), `classes.md` 219–223, `combat.md` 221, `special-rooms.md` 175, 325, `background.md` 53 ("at its first rank"); `assets/data/talents/README.toml` and `classes/README.toml` as above.

## 4. What it does to power, and the bonus-pick budgets

**Totals** (the numbers from the files; the level-60 hit points from `docs/balance.md` 802, 879–880):

| Talent | All ranks, at level 60 | One rank | Against the untrained warrior's 215 hp at 60 (trained: 400) |
|---|---|---|---|
| Hardy | 3 × 15 = 45 hp (at level 12: 9; at 30: 21) | 15 (3; 7) | 21 % → 7 % (trained 11 % → 4 %) |
| Deep well | 45 mana | 15 | (the wizard's pool: *unchecked*) |
| Thick skin | natural armour +3 | +1 | on 2d12, each point of Defence is one step of the hit table (`combat::hit_ways`) |
| Brawn | melee damage +2 | +1 | |
| Quick | speed +10 % | +5 % | |
| Keen ears | hearing +4 | +2 | |
| Pack mule | carrying +20 % | +10 % | |
| Armour training | Dexterity cap +2 | +1 | |
| Elemental ward | 50 % per element | 25 % | |
| Apprentice | 3 × (1d6+4) ≈ 22 points in one skill | ≈ 7.5 | |

Three Hardy picks were the cheapest hit points a build could buy (Toughness 11/14/17, no level); losing two of them is the one change a player will feel, and it is the owner's intent: the picks go to talents that do something. Everything else was a second helping of a small number. The seven follow-ups cost what the second ranks cost today (the same needs), so no build loses a thing it could do: only repeated numbers go.

**Bonus-pick budgets.** A class's bonus picks (`bonus_talents` in `assets/data/classes/*.toml`: five at 10, 20, 30, 45, 55; the thief's and warrior's eight) must be spent in the class's trees; a pick nothing fits is dropped (`settle_due`, `docs/dev/progression.md` 15). Pickable talents per tree today, counting the ranks that go and the follow-ups that come (the listing in §3; class-limited picks counted for their class):

| Tree | Talents today | Extra ranks lost | Follow-ups gained | After | Classes whose bonus picks draw on it |
|---|---|---|---|---|---|
| Vigour | 12 | Hardy 2 | — | 12 | berserker, druid, priest, paladin, ascetic |
| Guard | 9 | Thick skin 2, Armour training 1, Elemental ward 1 per element | — | 9 | paladin, warrior, craftsperson, the Hunter |
| Might | 9 | Brawn 1, Cleave 1, Mighty kick 1 | Great cleave, Breaker | 11 | berserker |
| Finesse | 11 (Silent feet merged into Light step elsewhere) | Quick 1 | — | 11 | ascetic, thief |
| Scholar | 10 | Deep well 2, Apprentice 2 per skill, Scroll savant 1 | Spell recall | 11 | witch, wizard, merchant |
| Grit | 10 | Pack mule 1 | — | 10 | priest, witch |
| Delver | 10 | Old delver 1 | Old hand | 11 | thief, aristocrat, craftsperson, merchant |
| Wayfarer | 4 | Pious 1 | Penitent | 5 | druid, ranger |
| Shadow | 7 → 6 (Silent feet) | Keen ears 1 | — | 6 | thief |
| Weaponcraft | 17 | Weapon focus 1 per skill | Weapon specialization (per skill) | 18 | ranger, warrior, aristocrat, craftsperson, the Hunter |
| Spellcraft | 10 | — | — | 10 | wizard |

No budget gets tight enough to leave a pick unspendable: the smallest pair of bonus trees is the druid's and ranger's Wayfarer + Vigour (5 + 12 talents for 5 picks), and Weapon focus, Weapon specialization, Elemental ward, Apprentice, and Sworn foe are one pick per choice besides. What tightens is the *sink*: today Vigour's five classes could put three bonus picks into Hardy and never choose; after, each bonus pick is a different talent. The Might tree loosens (two follow-ups) and the berserker is its only bonus class.

## 5. The rank machinery: remove it

**Recommendation: remove it, in its own PR right after the data lands** (open question 4). Once no talent has `ranks` above 1 and no table names a rank, the machinery is dead weight that every reader, screen, dump, and wiki line still carries: `Taken::rank` is saved and hashed, `TalentView::ranks`/`held`, `OwnTalent::rank`, the "2/3" in three screens and the dump, "You learn Hardy (2).", `Refusal::AtTop`, the per-rank loops in `talent.rs` 154 and `spell_mods.rs` 63, `effects_at(rank)` at nine call sites, the loader's two structs and three checks, the class table's `rank` field and its three checks, and the wiki's "per rank" and "rank N:" text (§3.4). Keeping it "for later" keeps a feature the owner has rejected one data line away from coming back. What the removal looks like:

| Now | After |
|---|---|
| `ranks = N`, `[[talent.rank]]` with its own `needs` and `effects`, `max_ranks` | gone: a talent is `needs` and `effects`, once. The loader refuses the old fields (`TalentSpec` is `deny_unknown_fields`, `content/talent.rs` 2978) |
| `needs = { talents = { hardy = 2 } }` | `needs = { talents = ["hardy"] }` (a list of ids); `Need::Talent(id)`; its words "Hardy" |
| `{ level = 25, talent = "x", rank = 2 }` | `{ level = 25, talent = "x" }`; `Table::abilities` `(level, talent)`; `ranks_to` → the ids |
| `Taken { id, rank, choice }` | `Taken { id, choice }`; `rank_of` → `holds(id, choice) -> bool`; `take` refuses one held (`Refusal::Held`, "You have {name} already.") |
| `effects_at(rank)`, `rank_count()`, `needs_at(rank)` | `effects`, `needs` |
| "Hardy 2/3", "1/2", `rank`/`ranks` in JSON and `TalentView` | gone |
| "You learn Hardy (2).", "Your Jack of all trades grows." | "You learn Hardy." only |
| the chain rule (`talent_alike.rs` 105–122) | **retired** (open question 3): it exists only to force a chain into ranks. The clone rule (two talents alike) stays, with the `alike` list |
| wiki "per rank", "rank N:", "Toughness 11 / 14 / 17", "2/2" rows | one line per talent |

Saves and hashes: `Taken` is saved and hashed (`talent.rs` 5–7), so every run with a talent hashes differently and no old save loads; saves are pinned to the game version (`docs/dev/saves.md`, #840; the memory rule), so this is the ordinary cost of a release and needs no shim. The golden manifests build their replays from action text at run time (`cargo xtask golden`), so they need nothing, and the repository holds no `runs/` replay files (none to re-create).

If the owner would rather not touch the code now: the data PR alone leaves everything working (every talent `ranks = 1`, which the loader accepts today), and the machinery shows nothing for a single-rank talent (`talents_screen.rs` 239–243 prints nothing for `ranks == 1`; `world/talent.rs` 309 says "You learn X." without the number). The cost is the dead code above and the one-line road back.

## 6. Implementation issues

Small PRs, rules first (CLAUDE.md "Keep PRs small"):

| # | PR | Contents | Depends on |
|---|---|---|---|
| T1 | **The data: one rank per talent, and the follow-ups** | the 17 files to `ranks` 1 (or the field dropped) with their rank-1 needs and effects hoisted; the seven new talent files (Great cleave, Breaker without `kick_back`, Old hand, Penitent, Weapon specialization, Spell recall, Master of none); the four `needs` to rank 1 (§3.1); the bard's 25 row; the chain rule retired in `talent_alike.rs` (or the six pairs listed in `alike`, per open question 3); the tests of §3.5 rewritten for the follow-ups; the three talents-screen snapshots and the level-up snapshot re-accepted; `spec/08i-character.md`, `docs/dev/progression.md` (rule 4 struck, the class-table rank paragraph), the wiki prose of §3.6, `cargo xtask wiki`; the balance probe rerun and `docs/balance.md`'s tables | the Light step merge (to avoid a conflict on `light_step.toml`); Weapon finesse touches no talent file |
| T2 | **Breaker's kick-back** | `kick_back` (`TalentEffect`, `TalentMods`, README line), the one read in `fight.rs`'s kick arm calling `World::knock_back` (made `pub(super)`), its test, the wiki row | T1 (or folded into T1 if small enough; it is one hook) |
| T3 | **Remove the rank machinery** | §5's table: loader, `Taken`, `Need::Talent`, the class and prestige tables' `rank`, the pick's words, perception, dumps, the three screens, the CLI help, the wiki generator, the READMEs, `docs/dev/progression.md`'s module table, the spec's data paragraph | T1 (the content must be rank-free first) |

T1 is the one the owner sees; T3 is mechanical. The balance probe (minutes) runs once, in T1.

## 7. Open questions

1. **Weapon specialization: a rank higher, or +2 damage?** Recommended: `weapon { ranks = 1 }` on the chosen skill (§2): a different mechanism from Weapon focus's +1 to hit, it scales, and it is what Stylist does for styles. The alternative keeps today's `weapon { damage = 2 }` under the old name, which the chain rule would call a rank and the owner's brief "a bigger number of the same thing".
2. **Breaker: keep it (with the kick-back), or drop Mighty kick's second rank?** Recommended: keep, with the kick-back and the digging quarter; the mechanic exists (`World::knock_back`) and a kick that sends a foe reeling is a thing to build a Might character around. If dropped, Tunneler and Lockbreaker cover what it did.
3. **Retire the chain rule (rule 4), or keep it and list the follow-up pairs in `alike`?** Recommended: retire it. It says "make it that talent's next rank", the thing this design abolishes; the clone rule (no two talents alike) stays and still catches a follow-up that only repeats its parent. Listing pairs instead would make `alike` carry six exemptions to a rule nobody wants.
4. **Remove the machinery (T3) now, or leave it?** Recommended: now, right after T1, as §5 says: dead code in nine readers and three screens, and a one-line road back to ranks.
5. **Apprentice's dice: keep 1d6+4 for its one pick per skill, or raise them?** Recommended: keep. One pick trains a skill from nothing to about 7, enough to use it; a build that wants a skill high trains it by using it (the skills system), and "unbalanced choices are fine".

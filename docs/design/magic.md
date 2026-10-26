# Warlock — Magic: spells learned from scrolls, tiers 0 to 9, wands and rods

Design document for the owner's review (2026-10-26). Written against `origin/main` at 88dbb0e5, the approved progression design (`docs/design/progression.md`), and the mana system as built (`docs/dev/mana.md`). Nothing here is implemented; §13 cuts it into issues. Numbers are first guesses, kept in data, to be tuned by play (§12 lists what to check first).

## Contents

1. Summary
2. Research: how the genre does magic, tied to the decisions
3. The model: spells, tiers, caster level
4. Learning: scrolls, study, overreach, mishaps
5. Casting: mana, resolution, damage, caps
6. Tags: schools, damage types, shapes
7. The first spell list
8. Classes: what is guaranteed, who casts
9. Items: wands and rods
10. Talents and class abilities (the coverage bar)
11. Data and code sketch
12. Balance checks
13. Implementation plan
14. Decisions left to the implementer
15. Decisions (the owner's, from the design conversation)

## 1. Summary

**What the owner asked for** (2026-10-26, in conversation): a magic system for a roguelike that limits resources without ADOM's spellbooks: "the lottery in ADOM was fine, it's just depleting very common spells (that you need to spend time refarming with no real danger), and keeping the books in bags was busywork." Spells are **learned from scrolls, like D&D**, so there is no separate book resource. Schools are tags. Classes get a very small guaranteed list (a cantrip, about four spells in all), so a caster who loses the lottery struggles but is never crippled, and does not shine without finding more spells. Spells have **levels** (cantrip and 1 to 9, as in D&D): learning above your level is hard and can ruin the scroll; casting above it costs far more mana. Wands and rods are the item forms. And the limits the owner set: damage is rolled (never 5e-style fixed), static bonuses are rare, **damage is strictly capped**, **area spells are expensive**, and the game goes to level 60, not 20.

**The proposal in one table:**

| Piece | Decision |
|---|---|
| What a spell is | a data entry: id, tier (0 to 9), tags (school, damage type, shape, kind), a mana cost, a resolution (`auto`, `attack`, `area`, `touch`), its effect and a strict cap. The casters' placeholder actives (`[[talent.spell]]`) are replaced by it. §3, §11 |
| How spells are had | **learned**, never prepared or memorised: a character knows a set of spells for good (saved); no slots, no forgetting, no spellbooks. The class gives a few; scrolls give the rest. §4, §8 |
| Scrolls | **two uses**, both existing code's neighbours: **read** casts it once, free of mana, aimed as today (no cancelling once the target is known); **study** (`Action::Study`) commits to learning it, takes time, cannot be cancelled, and uses the scroll up. §4 |
| Learning risk | at or below your level in tiers: certain. Above it: a check that gets harder per tier of overreach and per tier-sum of the spells already known; a failure **ruins the scroll and causes a mishap** scaled to the overreach, some of them very good. Study needs an identified scroll (identified by reading any copy or by a scroll of identify; the scroll of identify becomes a common drop), and any identified scroll whose spell is not yet learned can be studied: identification and learning are separate. §4 |
| Casting | costs mana by tier, **much more above your level** (×1.5 per tier of overreach), **area costs a multiple** (cone ×1.6 up to a radius-3 burst ×3). Cantrips cost 1. §5 |
| Resolution | **per spell**: `auto` (always hits, less damage), `attack` (rolls to hit like a missile, full damage, can critical), `area` (everyone in the shape, Evasion halves, no roll, dearer), `touch` (adjacent, cheap, stronger). §5 |
| Damage | **rolled dice**; a static `+n` is allowed but rare and small; **every spell has a hard cap** on dice and on the maximum possible damage, and nothing (level, talents, items, stats) pushes a spell over it. §5 |
| Power | **caster level**: a class's casting fraction of its level, plus talents, racials, items, and wands' set level; it sets learning, cost, and dice. Stats do not add damage; they widen reach, area, and duration, one stat per dimension. §3, §5 |
| Schools | **tags**, not trees: talents, class abilities, and items interact with spells through tags, which share themes with the damage types. §6 |
| Items | **wands** (a spell, finite charges, a set caster level, in the Missile weapon slot, no recharge), **rods** (the user's mana and level, effects no spell has). §9 |
| Everyone has mana | no class or race multiplier is 0 by default; the class table's casting fraction makes warriors poor casters, not non-casters. §3, §8 |
| Talents | a new `spells` system with a Spellcraft tree, class abilities for casters, and wand and rod talents: the coverage bar is met in the first PR that ships the system. §10 |

**Key decisions, and why**

1. *Scrolls teach, so nothing is depleted.* A learned spell is permanent; the limit is mana and the lottery of finding it, not use. A duplicate scroll is a one-shot cast, a sale, or a gift, so no scroll is dead loot.
2. *Risk is the player's choice.* The study check is shown in words ("this is far beyond you"), the scroll can be identified first, and a ruined scroll costs one optional spell, never a path to victory (owner, 2026-10-26: "sucks to suck").
3. *Caps, not curves.* A 10d6 fireball at level 20 is fine in D&D; at 60 levels it breaks a roguelike. Every spell has a cap; a tier has a ceiling; area is paid for in mana (§5).
4. *Schools are tags* so every system that already reads tags or damage types (affixes, talents, resistances) gets magic for free.
5. *One abilities menu.* Known spells are entries in the abilities menu (Source `Spell`), keys unbound by default and rebindable; no spell book screen (CLAUDE.md "Hard rules").

## 2. Research: how the genre does magic

Sources checked in this session: the ADOM wiki clone at ancardia.fandom.com ("Spellbook", "PP"), nethackwiki.com ("Spelbook"), brogue.fandom.com (staffs), wiki.cavesofqud.com ("Mutations"), the Moria spoilers (beej.us). Where a fetch failed, the entry below says "from memory".

| Game | Model | What it limits | What it drives here |
|---|---|---|---|
| ADOM | spellbooks found as loot; reading teaches a spell, a failed read can stun, confuse, blind, teleport, cost Toughness, or destroy the book; casting from the book costs 3× the time and PP; HP can pay for PP (2 PP per HP, risking the Mana stat); wizards and necromancers are guaranteed every book | acquisition, and the books' bookkeeping | learning from an item with a failure that **ruins it** (§4); Blood magic stays the HP-for-mana talent; the class guarantee (§8) |
| NetHack | books, a failure rate from the stat, level, skill, role, and armour; spells forgotten after 20 000 turns | memory, failure | no forgetting (busywork); armour matters to casting through cost, not a roll (§5) |
| DCSS (from memory) | known spells, spell levels, slots by level and skill, MP, a failure rate that skill and armour lower | slots and failure | spell levels as a ladder; no slots |
| Angband and Moria | class spell lists in books; a failure rate from the spell's level against yours; **missing mana adds failure**; a first cast gives XP (Moria spoilers) | mana and failure | the cost of overreach is mana, not luck (§5) |
| Brogue | staffs hold charges equal to their enchantment and refill over about 500 turns; no mana | charges and time | wands without recharge: the player's own owner decision (§9) |
| Caves of Qud | mental mutations on cooldowns scaled by Willpower and Ego | time | rods scale with the user, wands do not (§9) |
| ToME (from memory) | several resources by flavour | each resource | already covered by #225's resources, not repeated |
| D&D 3.5 | spell levels 0 to 9; `damage = Nd6 per caster level` up to a cap (fireball 10d6, magic missile 5 missiles); scrolls cast once or are **copied into a spellbook**; wands with charges and a set caster level; staffs and rods | slots per day | tiers, caps by dice, per-spell attack rolls (ray of frost rolls, magic missile does not), wands with a set level; **not** its slots, and not 5e's fixed damage (owner: "no deterministic die rules") |

## 3. The model: spells, tiers, caster level

**Tiers.** A spell has a tier 0 (a *cantrip*) to 9. Each tier has a **comfortable caster level**, the level at which learning it is certain and casting it is at base cost (in data, `data/spells.toml`):

| Tier | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|---|
| Comfortable caster level | 1 | 3 | 6 | 12 | 18 | 25 | 32 | 40 | 50 | 60 |

These are the class tables' own cadence (1, 6, 12, 18, 25, 32, 40, 50, 60 in the progression design), with tier 1 at 3.

**Caster level.** A derived number, not a stored one: `class level × the class's casting percent`, plus talents (`caster_level`), the race's and the birth sign's gifts, and worn items' affixes (a new `caster_level` effect, like `max_mana`), rounded down, at least 1 for anyone with mana. Wands carry their own set caster level (§9); a rod uses the user's. It is used for: the learning check (§4), the cost of overreach (§5), a spell's dice (§5), and nothing else.

**Casting percent** (the class table's `casting`; first guess):

| Percent | Classes |
|---|---|
| 100 | wizard, witch, priest, druid |
| 75 | bard, aristocrat |
| 50 | paladin, ranger, monk |
| 25 | thief, merchant, craftsperson, commoner, and the fighting classes |

So a level-30 warrior has caster level 7 or so: tier 2 spells are within reach, tier 5 are a gamble. The owner's "perhaps an ability in the tree that grows caster level" is the talent route on top (§10).

**Mana is innate.** Every class gives at least 1, and no multiplier is 0; a race may opt out (the data says so, as today). Mana growth currently stops at level 30 (`mana_level = 30`); with tier 9 at caster level 60 this becomes 60 in the first PR (§12).

**The ability stats' jobs** (one stat per dimension, the same for every spell; a spell says which dimensions it has):

| Stat | Dimension |
|---|---|
| Intelligence | learning: the study check (§4); how many spells' worth of upkeep the mind carries (§4 penalty) |
| Willpower | reach and area: range, a radius or length, one step per few modifier points, never more than the spell's own cap (§5); also mana regeneration, as today |
| Presence | duration and potency of buffs, hexes, and charms; the Inspire-like boons |

Damage dice come from caster level and nothing else; stats never lift damage over its cap.

## 4. Learning: scrolls, study, overreach, mishaps

**Scrolls.** A spell scroll (an object whose `read` table names a spell) has two uses:

- **Read** (`r`, `Action::Read`, as today): casts the spell once, costs no mana (the scroll *is* the cost), at the scroll's own caster level (its tier's comfortable level by default, a data field). It is aimed first: once the player has seen that it is targetable, they cannot cancel (the owner's rule), so reading an unidentified scroll is a gamble too.
- **Study** (a new action, `study:b`, a rebindable key command next to Read): the player commits to learning the spell. It takes time (§4 "Time"), cannot be cancelled once begun, and uses the scroll up. At the end: a check (below). Identification and learning are separate states. **Any scroll whose type is identified** (by reading any copy of it, or by a scroll of identify) **can be studied by a character who has not learned that spell**, whether or not they have read this very scroll; study is refused only for an **unidentified** scroll ("You do not know what this says.": identify it first) and for a spell already learned (below). **Scroll of identify becomes a common drop** and a regular shop item (the owner's rule: "better ID those scrolls before if you do not want to get fucked").
- A scroll of a spell the player already knows can still be **read** (a one-shot), or sold, or given. Studying it is refused with no time ("You know this spell already.").

**Unidentified scrolls** need ADOM-style labels (the scrolls spec has said "may come later"): a scroll type is unknown until read or identified, shown as "a scroll labelled XYZZY" (one label per spell per game, from the seed's identification stream). This is its own issue (§13) and a prerequisite of study (the scroll types are known on sight today; #393 changes that). Once a type is identified, it stays identified for the whole game, and every scroll of it can be studied by anyone who has not learned the spell.

**Overreach.** `overreach = tier(spell) − tier of the caster's level`, where a caster level's tier is the highest tier whose comfortable level it has reached (a level-2 character is at tier 0). At overreach 0 or less: learning is certain, no check, no risk. At 1 or more: a **check** (the game's 2d12, as skills use) against a difficulty:

`difficulty = 12 + 4 × overreach + known/10 − bonuses`

where `known` is the sum of the tiers (cantrip counts 1) of the spells the player knows (the owner's small cumulative penalty: ten tier-3 spells known make the next one about 3 points harder), and the bonuses are Intelligence's modifier, Spellcraft and Decipher script (each its value ÷ 10), a class's affinity with the school (§6), and the Quick study talent. All first guesses. The wiki shows the exact chance; the game says "this will be easy / a struggle / far beyond you".

**Time.** Study takes `10 minutes × (tier + 1)` of game time as one action, with the world acting around the player (a danger of its own); it is **interrupted**, as rest is, when a hostile comes into view or the player is hurt, and an interrupted study loses the time but keeps the scroll (a decision left to the implementer, §14: the alternative is to ruin it).

**Failure.** The scroll is destroyed and a **mishap** happens, drawn from `data/mishaps/` (one file per entry, like traps) by the overreach's band; the table is data so content can grow it:

| Band | Overreach | Examples (each uses an effect the game has) |
|---|---|---|
| Slight | 1 | the words fizzle: nothing; a status for a few turns (dazzled, sluggish); a puff of the spell's own school, small, at the reader |
| Bad | 2 to 3 | the spell goes off uncontrolled around the reader (a burst of the scroll's own effect at its own caster level, the reader included); mana drained; blinded; teleported on the level; a stat training mark lost |
| Terrible | 4 and more | the spell goes off at the scroll's tier as a burst on the reader; a stat damaged for a long time (the drain code); a summoned hostile of the school; a curse status |
| Lucky (any band, rare) | any | a wild surge casts a different good spell for free; a lasting buff; a free stat mark; the spell is learned anyway, one tier lower in power |

A mishap never kills outright by itself beyond what its own damage does: it is the spell's damage, and the caps (§5) apply to it.

**No anti-farming rule applies.** A failed study loses only that scroll; the spell can come again (the lottery is the point), and nothing here blocks the way to winning (owner, 2026-10-26).

## 5. Casting: mana, resolution, damage, caps

**Mana cost.** A spell's base cost by tier is data, scaled to the mana pools that exist (§12): a wizard with a +3 modifier has about 30 mana at level 3, 110 at level 12, and 270 at level 30; a tier's comfortable-level pool divided by about 10 is the cost of a tier's single-target spell (first guess):

| Tier | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|---|
| Base cost | 1 | 3 | 6 | 11 | 16 | 22 | 29 | 37 | 46 | 56 |

Modifiers, applied in this order and rounded to the nearest, at least 1 (the one `Actor::spell_cost` hook, then `spend_mana` and the talents' cut):

- **Overreach**: ×(1 + 0.5 × overreach), so one tier over costs half again, two tiers double (the owner: "mana cost increases significantly for learned spells above character level").
- **Shape**: the area multiplier (below).
- **Resolution** (§5): `auto` ×1, `attack` ×1, `touch` ×0.7, `area` by shape.
- **Armour**: worn armour above a limit adds a percent, the NetHack lesson as cost not luck; the Armour data names its `spell_cost_percent`.

**Area is expensive.** The owner: "we need to make AOE rather expensive in mana." The multiplier by shape, on the tier's base cost:

| Shape | Multiplier |
|---|---|
| single target (`ray`, `bolt`, `touch`, `self`) | ×1 |
| `cone` (length 3 to 5) | ×1.6 |
| `line` (a long bolt that hits all in its path) | ×1.8 |
| `burst` radius 1 | ×1.8 |
| `burst` radius 2 | ×2.4 |
| `burst` radius 3 | ×3 |

and an area spell's damage is also lower per target than the single-target spell of the tier (below).

**Resolution**, per spell, in data (`resolution`):

| Resolution | Roll | Damage vs. the tier's single-target `attack` spell | Shapes |
|---|---|---|---|
| `attack` | the 2d12 attack roll against Defence with the spell's attack skill, threats and critical confirmation as a blow; a miss does nothing | 100 % | ray, bolt |
| `auto` | none; always hits | 70 to 75 % | bolt (magic missile) |
| `area` | none; everyone in the shape; **Evasion** halves (as `cast_burst` already does) and Protection and resistances soak (the one `World::resisted`) | 75 to 80 % per target | cone, line, burst |
| `touch` | an attack roll against Defence at adjacency only | 120 % at ×0.7 cost | touch |

The attack roll's skill, for a first version, is the wielder's **Missile** skill for rays and **Spellcraft** for touch (left to the implementer, §14). Spell attack rolls can **critical**; `auto` and `area` never do, which is part of their price.

**Damage is rolled.** A spell's damage is its `dice` (rolled one draw per die, in actor-id order for areas, as the casts do today), plus a **static** `+n` that is allowed and rare: at most a quarter of the spell's ceiling (below), none on most spells. The dice grow with caster level and **stop at the spell's cap**:

```toml
[spell.damage]
dice = "1d6"          # at the tier's comfortable level
per_levels = 3        # one die more per 3 caster levels above it
max_dice = 3          # the cap: never more than 3d6
bonus = 0             # static; rare
```

**The strict cap.** A tier has a **ceiling**: the greatest damage a single-target spell of that tier may do in one cast at its maximum roll, counted the way hit points are (a level-70 creature has about 90 base hit points plus Toughness; a weapon blow is 1d12 plus bonuses):

| Tier | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|---|
| Single-target ceiling (max roll) | 6 | 12 | 18 | 24 | 32 | 40 | 50 | 62 | 76 | 90 |

and an `area` spell's per-target maximum is its ratio of that (a tier-3 burst of radius 2: at most 18). The data check refuses a spell whose `max_dice × sides + bonus` passes its tier's ceiling, so **no spell can be written over its cap**. Nothing adds over it: talents, items, and stats that "add dice" or "add percent" **raise the spell toward** its cap, or make it come earlier, never past it. (A talent that lifts the cap itself is not written; this answers "10d6 fireballs".) Cantrips never pass 6 total, so a wizard's cantrip is a poor weapon beside a crossbow's 1d10+1, which is the intent.

**Reach and area** grow with Willpower (§3) only to the spell's own `max_range` and `max_radius`, set by the shape; a ball never grows past radius 3.

## 6. Tags: schools, damage types, shapes

Every spell has **tags**; talents, class abilities, items (affixes), resistances, and the wiki's tables key on them:

| Kind | Tags (first set) |
|---|---|
| School | `evocation` (damage), `necromancy` (wounds, the dead), `abjuration` (wards), `restoration` (healing and cures), `transmutation` (changes to bodies and things), `enchantment` (minds), `divination` (seeing) |
| Damage type | `fire`, `cold`, `electricity`, `acid`, `poison` (existing), and two new: `force` (magic missile) and `necrotic` (cause wounds); holy and unholy damage stay deferred (spec §8 "Alignment") |
| Shape | `ray`, `bolt`, `cone`, `line`, `burst`, `touch`, `self` |
| Kind | `damage`, `heal`, `buff`, `hex`, `utility` |

**Schools are themes, not trees**: the damage type fits the school (fire and cold are evocation; necrotic is necromancy). A class has an **affinity** by school in its table (a bonus or malus to the study check and to the cost, in percent), so a priest finds restoration easy and a wizard everything but restoration a bit easier. Talents with a `choice` of a school or a damage type (the existing `element` choice already exists for damage types) give per-school gains: Pyromancer (fire spells cost less and reach farther, never more damage over the cap).

**New damage types** are `[[type]]` entries at the end of `data/damage_types.toml` (docs/dev/combat.md), with their resist and soak defaults.

## 7. The first spell list

Inspired by D&D 3.5, balanced against each other by §5's ratios, with names left for the world-language review (#378). **Every entry is a first guess** for the doc's review. Damage is the dice at the tier's comfortable level, and its cap; every cap obeys the tier's ceiling times the resolution's ratio (§5: `auto` at most 75 %, `area` at most 80 % per target, `touch` at most 120 %, `attack` 100 %), which the data check enforces.

| Spell | Tier | School | Shape, resolution | Effect (dice, cap) | Mana (base) |
|---|---|---|---|---|---|
| Ray of frost | 0 | evocation | ray, `attack` | 1d4 cold (cap 1d6), range 6 | 1 |
| Spark | 0 | evocation | bolt, `auto` | 1d4 electricity (cap 1d4), range 5 | 1 |
| Mend | 0 | restoration | touch | heals 1d3 | 1 |
| Magic missile | 1 | evocation | bolt, `auto` | 1d4 force, one die more per 3 caster levels (cap 2d4), range 8; no flat bonus | 3 |
| Burning hands | 1 | evocation | cone 3, `area` | 1d4 fire, one die more per 3 caster levels (cap 2d4), sets alight rarely | 5 (3 × 1.6) |
| Cause wounds | 1 | necromancy | touch | 1d8 necrotic, one die more per 4 caster levels (cap 2d6), adjacent | 2 (3 × 0.7) |
| Cure wounds | 1 | restoration | touch or self | heals 2d6 (cap 2d6) | 3 |
| Bless | 1 | abjuration | self | a boon (the existing Blessed) | 3 |
| Sleep | 2 | enchantment | `area` burst r1 | creatures of low level sleep for a time | 11 (6 × 1.8) |
| Scorching ray | 2 | evocation | ray, `attack`, one ray, then two and three as caster level grows | 1d6 fire per ray (at most 3 rays, so a cast's maximum is 18, the tier's ceiling) | 6 |
| Web | 2 | transmutation | burst r1 | holds creatures a while | 11 |
| Fireball | 3 | evocation | burst r2, `area` | 2d6 fire, one die more per 4 caster levels (cap 3d6, 18 at most), blasts earth as today, melts ice | 26 (11 × 2.4) |
| Lightning bolt | 3 | evocation | line, `area` | 2d6 electricity, one die more per 4 caster levels (cap 3d6), pierces all in the line | 20 (11 × 1.8) |
| Haste | 3 | transmutation | self | speed for a time (the existing Hasted if any) | 11 |

The old placeholder classes (arcane bolt 2d6, fire burst 3d6 r2, call lightning 4d6, flame 3d6 r2 and frost 2d6 r2) fold into this list in the migration PR (§13); the priest's blessing, cure wounds, remove disease, and the witch's hexes become entries with the matching kinds (`buff`, `heal`, `cleanse`, `hex`) the placeholder code already has.

## 8. Classes: what is guaranteed, who casts

**Guaranteed spells**: a class table lists its spells with the level each comes at; a full caster (casting 100) has **a cantrip at 1 and about four spells in all**, then finds the rest:

| Class | Guaranteed (first guess) |
|---|---|
| Wizard | ray of frost (1); magic missile (3); burning hands (12); the rest from scrolls |
| Priest | mend (1); cure wounds (3); bless (3); remove disease (12) |
| Druid | spark (1); a nature buff (3); call lightning, under the open sky (12); one more |
| Witch | evil eye (1); hex of weakness (3); cause wounds (12) |
| Bard | the bard's songs stay abilities; one cantrip and a buff |
| Paladin, ranger | a cantrip or a buff at the half-casting levels |
| Others | none, or a cantrip by a story reason; **a class that would read as a meme gets none** (the owner) |

The shared placeholders (`blessing`, `cure_wounds`, `remove_disease`, `call_lightning`, `flame`, `frost`, `arcane_bolt`, `fire_burst`, `evil_eye`, `hex_of_weakness`, `curse_of_sloth`, `blight`) become class spell entries: the migration PR reads the old `[[talent.spell]]` tables into spell data and the class tables' `spells` list.

Hybrids' (paladin, ranger) are low-tier boons and self buffs. A class with no story for magic has none, by the owner's rule: "if the class sounds like a meme, do not give them spells to save them."

## 9. Items: wands and rods

Both are objects with a `use` table, in the **Missile weapon slot** (so a wand and a crossbow compete for it), used through the same targeting key (`t`, sent as `fire`/`throw` today) with the targeting rules of the spell or effect they hold; none are cast in the abilities menu.

| Item | What it holds | Resource | Caster level | Recharge |
|---|---|---|---|---|
| **Wand** | a *spell* (any in the list) | charges (a data count by tier, for example tier 1: 12, tier 3: 7, tier 6: 3) | **set** in the item (its tier's comfortable level by default; magic wands carry more) | **none** (the owner; "we might get such ability introduced later") |
| **Rod** | an *effect no spell has* (dig, summon, a sleeping cloud...) | **the user's mana** (a cost in the item) | **the user's** | none needed |

- **A wand's resolution is its spell's**: a wand of ray of frost rolls to hit (with the wielder's Missile skill), a wand of magic missile does not. An empty wand stays an item (sold, or kept for a later recharge); using it says "nothing happens" and costs no time.
- **Wands let a martial class cast**: a warrior with no caster level fires fireball wands at the wand's level. That is the point; finite charges keep it scarce.
- **Identification** is the scroll of identify's (charges and spell: unidentified items show neither) and Lore's. **Use magic device**, a trained skill that exists (`assets/data/skills.toml`, "Making wands, staves, and scrolls work, even without the gift"), adds to a wand's effective caster level and to a rod's cost cut; the exact is left to the implementer (§14).
- **No magic staffs** (owner, 2026-10-26: "get rid of magic staffs if it's an issue"; it was one: "staff" is also the quarterstaff's weapon skill and a class's wielded item). Effects no spell has belong to rods; the idea is deferred (#196).

## 10. Talents and class abilities (the coverage bar)

The coverage policy (`docs/dev/progression.md`): a new system ships with its `[[system]]` tag and its talents. The system tags are `spells` (new) and the existing `mana` and `scrolls`. The first set (names and numbers are first guesses; every one is a data file in `assets/data/talents/` on an existing or listed hook):

| Talent | Tree | What it does | Hook |
|---|---|---|---|
| Quick study | Spellcraft (new tree) | studying is easier and takes half the time | study check and time |
| Overreach | Spellcraft | the mana overcost per tier is cut by a third | `spell_cost` overreach |
| Spell recall | Scholar (exists) | one scroll read in four is not used up (as today) | read |
| School focus (choice: a school or damage type) | Spellcraft | those spells cost 10 % less, reach one step further | the tag hooks |
| Arcane training | class: wizard | caster level +1 per 5 character levels (the "ability in the tree that grows caster level") | `caster_level` effect |
| Channeler | Spellcraft | rods cost less of the user's mana | rod cost |
| Wand mastery | Weaponcraft | wands' effective caster level +2 | wand |
| Empower and Archmage | class | **reworked**: they lower costs and reach the cap sooner; they cannot lift it | `spell_damage` (the cap clamps) |
| Scroll adept | exists | reading costs half the time; keep | exists |

Class abilities for the casters come with the migration (§13) as a level-1 cantrip and the class tables' spells; Blood magic stays. Every spell-related talent names the `spells` system (and `mana` where it touches it).

## 11. Data and code sketch

- **Data**: `assets/data/spells.toml` (the tier table: comfortable levels, base costs, ceilings, area multipliers; `[mana_level]` raised in `experience.toml`) and `assets/data/spells/` one file per spell (`[[spell]]`: id, name, tier, tags, resolution, shape, `[spell.damage]`, `[spell.effect]` for non-damage kinds, range, a `scroll` block for the object it comes on, noise); `assets/data/mishaps/` one file per mishap; a class table's `casting` and `spells`; objects' `use` for wands and rods; the Armour data's `spell_cost_percent`.
- **Core**: `content/spell_def.rs` (not the existing `spell.rs`, which holds the bursts and bolts the scrolls and traps share), `world/learn.rs` (study, the check, the mishaps), `world/cast.rs` (the one cast path: cost, resolution, dice with the cap, then the existing `cast_burst`, `cast_bolt`, and new `cast_cone` and `cast_line`), `Actor::spells` (the learned set, saved), `World::caster_level`, `Action::Study` and `Action::Cast` generalised from the placeholders, `ActionKind::Study`.
- **Saves change** (a spell set on the actor and, for wands, charges on item instances), so the integrator bumps `FORMAT_VERSION` once per release as usual; the `spells` list is left out of the JSON while empty, so an actor without spells hashes as before.
- **UI**: a study command (`KeyCommand::Study`, bound next to Read like the other basic commands, rebindable), spells in the abilities menu (a group *Spells* with a column for tier and cost), the existing aiming mode with the spell's shape (a cone's and a line's lines), the unidentified scroll labels' item names, and text snapshots for each screen (CLAUDE.md "Golden screenshots": no golden entries).
- **Wiki**: generated tables for tiers (comfortable level, base cost, ceiling), shapes, and the spell list; a page "Magic" for the rules (the wiki states the numbers the game keeps vague).
- **Determinism**: the study check and the mishap draw come from the gameplay stream only when they arise; a wand's rolls are the spell's; no draws for a certain learning.

## 12. Balance checks (first, before tuning by play)

1. **Mana pools against the cost table**: the pools in §5 are my arithmetic from `docs/dev/mana.md` and `experience.toml`, not a run's result; verify with `warlock-cli dump --layers stats` at levels 3, 12, 30, 60 per class, and set `mana_level` to 60.
2. **Regeneration**: `regen_seconds` is Willpower's, and the pools above are far larger than at the placeholders' time; check that a rest to full is a few hours, not a day, or tier-3 casters never need to rest.
3. **Ceilings against creature hit points** (the roster to level 70; `base_hp` up to 90 plus Toughness): a tier's ceiling should kill the same tier's common creature in a few casts, never one.
4. **Weapon and bow parity**: a level-12 wizard's best attack should cost real mana to match a warrior's blow; compare tier 3's 4d6 cap (24) to a level-12 fighter's blow (a great axe at 1d12 plus bonuses).
5. **A test**: every spell file passes the cap check; the cost and ceiling tables are monotonic; `area` is dearer and weaker than the matching single-target spell.

## 13. Implementation plan

Waves are ordered; the first and second can start at once. Every PR names its talents (§10), a spec section, its `docs/dev/` topic, and its wiki page.

| # | Issue | Depends on |
|---|---|---|
| #393 | **Unidentified scrolls**: labels per game, identified by reading or the scroll of identify; identify scrolls made common in loot and shops | none |
| #394 | **Spell data and the tier tables**: `spells.toml`, the `[[spell]]` file format with the cap check, `casting` percent in class tables, `World::caster_level`, `mana_level` 60, the wiki's tier tables | none |
| #395 | **The one cast path**: resolution (`auto`, `attack`, `area`, `touch`), the cost with overreach, shape and armour, the new shapes (`cone`, `line`), the new damage types, and the placeholder spells migrated onto it | #394 |
| #396 | **Learned spells and study** (the study list is the identified scrolls whose spell is not learned; unidentified refused with "You do not know what this says.", a known spell with no time): `Actor::spells`, `Action::Study`, the check, the time, the interruptions, the mishaps (data), the scroll objects' two uses, the abilities menu's Spells group | #393, #395 |
| #397 | **Class spell lists**: the guaranteed spells per class, the first spell list (§7), the scroll objects, loot and shop weights by tier | #395, #396 |
| #398 | **Wands**: the Missile-slot items, charges, the set caster level, the spell's resolution, identification of charges | #395 |
| #399 | **Rods**: effects no spell has, the user's mana, a first handful of effects | #398 |
| #400 | **Talents**: the Spellcraft tree and the first set (§10), the `spells` system tag, the `caster_level` and school-choice effects, Empower and Archmage reworked | #395, #396 |
| #401 | **Magic on the wiki and in the spec**: the "Magic" page, spec §8's section, the `docs/dev/` topic `spells.md` (folded into each PR's docs; this is the cross-checking pass) | #397 |

## 14. Decisions left to the implementer

Owner, 2026-10-26: "open questions left for the implementer to decide." Each question below stays listed with the design's recommendation as the **default**; the PR that implements the issue concerned decides, and **states what it chose** in its description (and in its `docs/dev/` topic). Nothing here blocks the design's approval.

1. **Interrupted study**: lose the time and keep the scroll (the default), or ruin the scroll?
2. **The attack roll's skill**: Missile for rays and Spellcraft for touch (the default), or one Spellcraft-based skill for every spell attack?
3. **Use magic device**: what it does (caster level for wands, rod cost, scroll misfires, or all of them)?
4. **Spell cap lifts**: the default is that nothing lifts a cap; does the owner want a rare, named exception (a unique item)?
5. **Casting percents** (§3's table) and the guaranteed lists (§8): the first guess is mine.
6. **New damage types** `force` and `necrotic`: or untyped force, and necrotic as the existing poison?
7. **The mishap table's tone**: how nasty may *Terrible* be (a stat damaged for a long time, a hostile summoned)?
8. **Spell recall's old meaning**: one scroll in four kept when read; does it also apply to study (a failed study that keeps the scroll)?

## 15. Decisions (the owner's, from the design conversation, 2026-10-26)

- Learned from scrolls, no spellbooks or bags.
- Spells have levels: cantrip and 1 to 9; comfortable levels in data; learning above one's level is hard and can ruin the scroll.
- Mana cost grows a lot for spells above the caster's level.
- A small learning penalty by the cumulative level of the spells already known.
- Reading casts it and cannot be cancelled once the target is known; studying commits and cannot be cancelled; identify scrolls are common.
- Owner correction (2026-10-26): "Unidentified scrolls are a prerequisite for study. The spec says scroll types are known on sight today, so #393 changes that. Identified scrolls not yet learned should also be able to be studied." Identification and learning are separate: any identified scroll whose spell is not learned can be studied; only unidentified scrolls and already-learned spells are refused.
- Owner (2026-10-26): no magic staffs (rods and wands stay); the open questions of §14 are left for the implementer of each issue to decide, the design's recommendation being the default.
- A failed study loses the scroll and causes a mishap by overreach; some mishaps are very good.
- Every living thing has some mana; no class's multiplier is 0.
- Caster level combines the class's levels, talents, racials, items, skill; ability stats modify range, area, and similar per spell, consistently.
- Schools are tags; talents, class abilities, and items interact through them; tags share themes with damage types.
- Classes have a tiny guaranteed list, a cantrip and about four spells; no spells for classes that would be a meme.
- Resolution is per spell, 3.5-inspired, balanced against each other.
- Cantrips cost mana. Reading a scroll costs none.
- Wands: Missile slot, charges, a set caster level, the spell's own targeting, **no recharge**. Rods: the user's mana and level, effects no spell has.
- Spell damage is **rolled**, not 5e's deterministic; a static `+n` is allowed, sparingly.
- **Strict damage caps**; the game runs to level 60; **area spells are expensive**.

# Warlock — Affixes for the game's systems: spellcasting, stamina, and the rest

Design document for the owner's review (2026-10-28). Written against `origin/main` at 99b15e62, with the itemisation change (#621: rarity 85/13/2, the affix counts, affix categories) assumed as the base even though it has not merged. Nothing here is implemented; §13 cuts it into issues. It builds on what is there: the affix system (`docs/dev/magic-items.md`), mana (`docs/dev/mana.md`), spells (`docs/dev/spells.md`), resources and stamina (`docs/dev/resources.md`), weapon abilities (`docs/dev/weapon-abilities.md`), potions (`docs/dev/potions.md`), and the systems surveyed in §6. Every number is data and hidden in the game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

## Contents

1. Summary
2. The owner's request, and what exists today
3. What a new affix is: the shape every candidate takes
4. Spellcasting affixes
5. Stamina affixes, and the resource effects
6. The other systems, surveyed
7. The data-only batch: skill suffixes and typed resistances
8. What could break
9. Determinism
10. Talents
11. What the player sees; the wiki, the spec, and the docs
12. Choices made along the way
13. Implementation plan
14. Open questions, each with its default

## 1. Summary

| Piece | Proposal |
|---|---|
| The shape | an affix is one data file under `data/affixes/` with an `effect`, a `category` (#621), its `bases` (the item kinds it rolls on), a `weight`, and `tiers` by item level; each new effect is one `Effect` variant read in **one** place, the place the matching talent is already read (`TalentMods` style: Frugal caster's hook takes of Thrift, Deep lungs' takes Tireless). No new saved state anywhere: affixes are saved by id, and every hook is a pure read. §3 |
| Spellcasting | eight affixes on seven new effects and one that exists unused: **of the Adept** (caster level, the effect that has no file yet), **of Thrift** (mana costs, every spend), **Mage-wrought** (local: a piece of armour hinders casting less — Warlock has no spell failure; the armour's mana surcharge is the thing to ease), **of the Magus** (spell to-hit), **Baleful** (spell criticals), **Potent** (the damage lift), **of Farcasting** (reach), **of Lingering** (buffs and hexes last longer); later **of Siphoning** (mana on a kill) and **of Wands**. §4 |
| Stamina | three affixes on three new **resource effects** that name their pool in data (`resource = "stamina"`, as skill affixes name a skill): **Tireless** (the pool), **of Breath** (its pace), **of Ease** (every weapon active a point cheaper); later **of the Hunt** (a point on a kill). An affix never grants a pool: one who holds no stamina gets nothing from it, which is why the other class pools get no files yet. §5 |
| The rest | surveyed system by system (§6): five more hooks worth an affix each (**Hobnailed** boots kick doors and chests in, **Delving** gloves dig faster, **of Sustenance** slows hunger, **of Wariness** softens traps, **of Health** resists disease), one data-only batch (ten skill suffixes and five typed resistances: of the Adder, of the Storm, of Glass, of the Anvil, of the Living, §7), and a list of what gets **no** affix and why (temperature feels nothing yet, speed of every action breaks the rest batching, alignment is not for sale, light and luck are covered or unwanted, vision and the class pools wait). |
| Categories | #621's rule (an item never has two affixes of one category) is the one that lets two effects of one family be kept apart: of Thrift and Frugal-like cuts share `mana_cost`; Tireless and of Breath do not (`stamina`, `stamina_pace`). Every new affix names its category here; a category spans a prefix and a suffix only for one effect family. §3 |
| Talents | no new `[[system]]`: every affix joins a system whose talents exist, read at the very hook. Two talents that make the new affixes matter: **Attuned** (scholar: the caster level your items give counts half again) and **Bellows** (vigour: the stamina your gear gives is doubled). §10 |
| Slices | one foundation PR for the hot files (the `Effect` list, `Mods`, the phrases, the wiki words), then three in parallel that add only hooks, files, and tests; then the utility hooks, the on-kill pair, the talents, and a balance pass. **None adds saved state**; none adds an action or a key. §13 |

## 2. The owner's request, and what exists today

The request (2026-10-28), verbatim: *"implement affixes that interact with spellcasting and stamina. also research what other systems we have that made sense we had affixes for."* The issue (#622) lists the systems to read and the examples to consider: mana pool, mana regeneration, spell cost, spell failure, caster level, spell to-hit and crit, stamina pool and regeneration, ability stamina cost; and the rules: hidden numbers hidden in the game and exact in the wiki, one hook per modifier, talents may interact.

**The base assumed from #621** (not merged; read from its issue): loot is normal 85 %, magic 13 %, rare 2 %; a magic item has a prefix only 40 %, a suffix only 40 %, both 20 %; a rare item has 3 affixes 50 %, 4 30 %, 5 15 %, 6 5 %, at most 3 of each kind; **every affix names a `category`** in its file (a loader check), **an item never has two affixes of one category** (the draw skips categories already on the item), prefix and suffix lists keep their own categories, and one category may appear on both kinds only when it is the same effect family. The categories of today's 50 affixes are #621's to name; this document names the new ones' and, where a new affix must share one with an old, says which (§3). Nothing below depends on #621's bastard sword or longsword change.

| Part | Today |
|---|---|
| The affix system | `data/affixes.toml` (settings) and `data/affixes/`, one file per prefix or suffix, read in path order; `content/affix.rs`: `Effect` (30 variants plus the six stats), `AffixDef { id, name, kind, effect, skill, damage_type, bases, weight, tiers }`, `Group = (Effect, Option<SkillId>, Option<DamageTypeId>)` (an item has at most one affix of each group); `magic/mods.rs`: `Mods::local` and `Mods::worn`, values summed (Deflection, natural armour, and light: the best); `describe.rs::phrase` the words of each effect; `Numbers::of` the item lists' numbers (to-hit, damage, Dv, Pv, range: none of the effects below show there) |
| Local and global | attack effects on what attacks are local, on anything else global; the item's own numbers (Armour, Protection, Parry, Shield, max Dex, weight) are local; everything else global. `Effect::allowed_on` keeps an affix off an item that lacks the means (range without a range, Durability on what never breaks) |
| Saves | an instance is `Affix { id, tier, value }` on `Magic` on a `Stack`, **saved by id**: adding affixes keeps saves and old replays load; values and tiers are checked on load |
| Mana | `Actor::max_mana` reads `MaxMana` (Mystic), `World::mana_rate` reads `ManaRegeneration` (of Clarity) beside `TalentMods::mana_regen_percent`; `Actor::mana_cost` applies `TalentMods::mana_cost` (Frugal caster, −20 %), the one cut every spend goes through (`World::spend_mana`: spells, rods, Inspire, summons, the dev drain) |
| Spells | `Actor::caster_level` already reads `Effect::CasterLevel` ("no affix file grants it yet"); `Actor::spell_base` (the tier price × armour × the talents' percents), `Actor::spell_cost` = `spell_base` through `mana_cost`; `Actor::armour_spell_cost` sums the worn pieces' `spell_cost_percent` (chain 10, scale 20, banded 30, plate 40, great helm 10, large shield 10, tower shield 20; eight objects) less Battle caster's ease; `Actor::spell_to_hit`, `spell_lift` (Keen mind's `more`), `spell_reach` (School focus's cell), `spell_degrees_more` (`world/spell_scale.rs`, `world/cast.rs`); an own cast's attack is `Profile::bare(Attack { to_hit, threat 22, crit ×2 }).with(benefit)`, the roll held to the cap; **there is no spell failure**: the magic design made armour a cost, never a chance |
| Stamina | the last resource in `data/resources.toml`: max 10 + the Toughness modifier + a point per 10 levels (at least 5), back a point every 12 s, resting too; granted by Trained wind (seven classes) or by the first active rank learned. `Actor::resource_max` adds `TalentMods::resource_max` (Deep lungs +4); `World::resource_rate` adds `resource_rate` (Steady breathing +50 %); `Actor::weapon_stamina` takes `weapon_stamina` (Drillmaster −1) and `ability_stamina` (Armour-breaker) off an active's cost, at least 1; `World::gain_resource` adds the data's `builds` and the talents' `resource_builds` (Second breath); `resource::pass` clamps a pool to its maximum as time moves |
| Creatures | mana affixes on creature gear work (#525: a goblin wizard's Mystic amulet); creatures never hold resources; creature spells are announced only |
| Talents | the systems a talent may name (`data/talents.toml`): `mana`, `spells`, `resources`, `stamina`, `magic_items`, and the rest; every system below the bar has at least two talents already |

## 3. What a new affix is: the shape every candidate takes

Every candidate in §4 to §7 is described by the same eight things, so the implementer has nothing to decide but the code:

| Field | Meaning |
|---|---|
| **Name, kind** | the prefix ("Baleful") or suffix ("of Thrift"), in the register the affixes already use: one plain word, or "of the Animal", or "of a Quality" |
| **Effect** | an id in `Effect` (`content/affix.rs`), new ones marked; a new effect is one variant, one `Mods::add` arm (sum unless said), one phrase in `describe.rs`, one word in `xtask/src/wiki/magic.rs` (`effect_words`, `tier_value`), one `allowed_on` arm where a means is needed |
| **Category** | #621's; named here for every new affix. Rule of thumb used: one category per effect family; two effects share one only when stacking them would be the same bonus twice |
| **Bases** | the item kinds (`weapon`, `launcher`, `ammo`, `body`, `helmet`, `cloak`, `gloves`, `boots`, `shield`, `ring`, `amulet`) and so the slots: Main or Off hand, Missile weapon, Missiles, Body, Head, Cloak, Hands, Feet, a hand (shield), Left or Right ring, Neck |
| **Weight** | how often it rolls among those that may (100 the default; 60 for a secondary one, 40 for a rare and strong one, as Brutal and of Ruin) |
| **Tiers** | by item level: the Ascmearc's floors run 1 to 12 (Cealdhlāw 1–6, Fennmynster to 8, Fyrdelf to 12), so the bands are *early* (1–3), *middle* (4–7), *deep* (8–12), and one top tier at 10–14 for deeper places, as the existing affixes have (most top out at 7–8) |
| **The one hook** | the function that reads it, always the one that reads the matching talent; the affix's value joins the talent's term there and nowhere else |
| **Local or global** | global unless said: none of the new effects is an attack effect; Mage-wrought is the one local one (it changes its own piece) |

Values are positive numbers meaning "more" or "less" as the effect says (as Featherweight's `weight` is "N % lighter" and of Celerity's "N % faster"), never signed. Percent effects that stack with a talent's are capped where speeds are (50 off in all); flat ones add. Everything applies unidentified, as every affix does.

## 4. Spellcasting affixes

Mystic (maximum mana) and of Clarity (mana regeneration) stay as they are; the issue's "mana pool" and "mana regeneration" are them. The rest of the issue's list, with the one candidate each, and what else the spell system offers:

| Affix | Effect (id) | Category | Bases | Weight | Tiers (level: value) | The one hook | Notes |
|---|---|---|---|---|---|---|---|
| **of the Adept** (suffix) | `caster_level` (exists, unused) | `caster_level` | weapon, helmet, ring, amulet | 50 | 2: +1; 7: +2; 12: +3 | `Actor::caster_level` (reads it already) | +1 is about a wizard's level (1.2 caster levels a level); the tiers are spaced so a +2 waits for Fennmynster's floor. On a weapon it is the staff in the caster's hand; a warrior's "long sword of the Adept" gives a caster level to one who casts nothing (`caster_level` is at least 1 only while `max_mana` > 0, so a mana-less wearer reads 1 or more and nothing cares) |
| **of Thrift** (suffix) | `mana_cost` (new: abilities cost N % less mana) | `mana_cost` | helmet, cloak, ring, amulet | 60 | 2: 5–8; 6: 8–12; 10: 12–16 | `Actor::mana_cost`: the items' percent joins `TalentMods::mana_cost`'s (Frugal caster's −20) before the one rounding; the sum capped at 50 off; at least 1 for a cost above 0 | every spend goes through it: spells, rods (`rod_base` → `spend_mana`), Inspire, a summoner's cost, the dev drain. So one affix covers "spell cost" and rods both; no `rod_cost` affix. Archmage's and Empower's percents are `spell_base`'s and multiply with it, as today |
| **Mage-wrought** (prefix, **local**) | `spell_ease` (new: this piece hinders casting N points less) | `spell_armour` | body, helmet, shield | 50 | 2: 5; 5: 10; 9: 15–20 | `Actor::worn_spell_cost`: each worn piece's `spell_cost_percent` less its own local `spell_ease`, never below 0 | the "spell failure" answer: Warlock has none, armour's surcharge is the thing. `loot::may_roll` needs the object's `spell_cost_percent` > 0 (the per-object means check Range and Durability use), so it rolls only on the eight hindering pieces. A chain shirt fully eased (10 − 10) no longer counts as armour for the grip: `Actor::spell_skills` picks the ranged or touch style instead of Battle caster, which is right (there is nothing left to ease) |
| **of the Magus** (suffix) | `spell_to_hit` (new) | `spell_to_hit` | weapon, gloves, ring, amulet | 60 | 1: +1; 4: +1–2; 8: +2–3 | `Actor::spell_to_hit`: the scales' value plus the items' | an own cast only (`attack` and `touch` resolutions); a scroll read or a wand zapped keeps the Dexterity modifier, as the scales do. The existing global to-hit (Accurate, of Precision on gloves) never reached spells: `Profile::bare` carries no worn mods, and that stays so |
| **Baleful** (prefix) | `spell_threat` (new: spells threaten a critical on N more rolls) | `spell_critical` | weapon, gloves, amulet | 40 | 4: +1; 9: +2 | `cast_single`'s `Attack { threat }` through a new `Actor::spell_threat` (22 less the items', never below the blows' floor, `THREATS`) for an own cast; scrolls and wands keep 22 | the Spell attacks skill widened spell criticals in #571, so the system has them; a critical's roll is still held to the cap (design §5's strict cap), which is why no critical *multiplier* affix: it would hit the ceiling at once |
| **Potent** (prefix) | `spell_lift` (new: the damage lift N points more) | `spell_power` | weapon, ring, amulet | 50 | 3: 5–8; 7: 8–12; 11: 12–15 | `Actor::spell_lift`: `more` = Keen mind's focus lift + the items', which `SpellRules::lift` adds to the tier's `max_lift` | percent of the spell's cap, so a +10 is a tenth more damage at most, only on spells with `damage` scales, and only for casts paid in mana (the lift is the caster's own); Empower's dice and Archmage's per-die stay what they are |
| **of Farcasting** (suffix) | `spell_reach` (new: cells) | `spell_reach` | cloak, ring, amulet | 50 | 2: +1; 7: +2 | `Actor::spell_reach`: School focus's `focus` + the items' on the range (a cone's or a line's length too), never the radius, never past the spell's cap | a burst still costs by its own radius; a touch stays 1 |
| **of Lingering** (suffix) | `spell_duration` (new: degrees) | `spell_duration` | helmet, ring, amulet | 50 | 3: +1; 8: +2 | `Actor::spell_degrees_more`: the scales' steps + the items' | clamped by the status's `max` in `Effects::add` (3 for Blessed, Hexed, Weakened, Sluggish), so the second tier matters only on long statuses; the wiki says so |
| **of Siphoning** (suffix; wave 3) | `mana_on_kill` (new) | `mana_on_kill` | weapon, launcher, amulet | 40 | 3: 1; 7: 1–2; 11: 2–3 | one line in `fight.rs::dies` beside `gain_resource(by, Kill)`: `World::siphon_mana(by)` raises `mana` by the worn sum, never past `max_mana` | the killer's, creature or player (a creature with a pool refills it: harmless and consistent). Mana is not a resource pool, so this is its own line, not a `Build` |
| **of Wands** (suffix; wave 3) | `wand_level` (new) | `wand_level` | gloves, ring | 40 | 4: +1; 10: +2 | `Actor::wand_caster_level`: the wand's own + Wand mastery's + the items', at least 1 | wands themselves have no `kind` and are never magic; the hand that holds them is |
| **of the Scholar** (suffix; §7) | `skill` = `spellcraft` | `skill:spellcraft` (#621's form for skill affixes) | helmet, ring, amulet | 60 | 1: 10–20; 4: 20–35; 7: 35–50 | `Skills::effect_with` already: the study check's `skill_effect(Effect::Study)` | the issue's "study" candidate needs no code: Spellcraft's benefit is the study check's points |

**Considered and left out.** *Mana resting pace* (Meditation's hook): too narrow beside of Clarity. *Overreach cost*: a talent's niche. *Spell cost by school* (School focus): the per-tag machinery would need a tag in the `Group`; one of Thrift covers it. *A critical multiplier for spells*: see Baleful. *Casting while Raging*: a status block is a rule, not a number. *Spell failure chance*: none exists and the magic design chose not to have one; Mage-wrought is the affix of that decision.

**Interactions with the talents** (each at the same hook, so they add and the cap holds): Frugal caster + of Thrift (−20 and −16 is −36; two rings and the talent hit the 50 cap); Arcane training + of the Adept; Keen mind + Potent (both widen `max_lift`); School focus + of Farcasting (two cells past the base, never the cap); Archmage's −50 % multiplies with of Thrift's cut, each applied where it is today.

## 5. Stamina affixes, and the resource effects

Stamina is a resource (`docs/dev/resources.md`), so its effects are written **per resource** and name their pool in data, exactly as a skill affix names its skill: `effect = "resource_max"`, `resource = "stamina"`. `Group` grows a fourth member, the resource (`Group { effect, skill, damage_type, resource }`, a struct now), so "one affix per effect" means one per (effect, resource). The loader refuses an unknown resource and a resource effect without one.

| Affix | Effect (id) | Category | Bases | Weight | Tiers (level: value) | The one hook | Notes |
|---|---|---|---|---|---|---|---|
| **Tireless** (prefix) | `resource_max` (new) | `stamina` | body, gloves, boots, ring, amulet | 60 | 1: +1–2; 4: +2–3; 8: +3–5 | `Actor::resource_max`: `more` = the talents' (Deep lungs +4) + the worn items' for that resource | a pool of 10 to 14 at the Ascmearc's levels; +3 is a lunge more. The pool is clamped as time moves (`resource::pass`); a new `World::keep_resources_within_max` beside `keep_mana_within_max` makes taking the item off exact at once (one call in `World::apply`) |
| **of Breath** (suffix) | `resource_rate` (new: N % faster) | `stamina_pace` | body, helmet, cloak, amulet | 60 | 1: 15–25; 4: 25–40; 7: 40–60 | `World::resource_rate`: 100 + the talents' (Steady breathing +50) + the items'; never below 0. The function's early return on `talents.is_empty()` becomes "no talents and nothing worn" | resting's 0 for a pool that does not refill while resting stands (stamina refills resting, so it is moot for it) |
| **of Ease** (suffix) | `active_stamina` (new: every weapon active costs N stamina less) | `stamina_cost` | weapon, launcher, shield, gloves | 40 | 3: 1; 9: 2 | `Actor::weapon_stamina`: `own − weapon_stamina − named − items`, at least 1; the early return on `talents.is_empty()` goes the same way | **global**, not local (decision: it is no attack effect; the actives one may use are those of what one holds anyway, so a local reading would give the same answer with more code). Drillmaster's −1 and of Ease's −2 bring a 3-stamina active to the floor of 1 |
| **of the Hunt** (suffix; wave 3) | `resource_on_kill` (new) | `stamina_on_kill` | weapon, launcher, boots | 40 | 3: 1; 8: 1–2 | `World::gain_resource(on = Kill)`: the data's `builds` + the talents' `resource_builds` (Second breath) + the worn items' for that resource | the other events (`hit`, `critical`, `struck`, `sneak_attack`) are not offered as affixes: a kill is the one a fighter cannot farm by waiting |

**An affix never grants a pool.** One who holds no stamina (a wizard, a priest) gets nothing from Tireless; the description still says what it does, and the wiki says who holds stamina. The alternative, an item that grants the pool while worn, would need an un-grant on removal (a fourth grant place, and a pool that comes and goes with a ring), and the mana affixes set the precedent the other way: Mystic raises a maximum, it never makes a caster. Creatures never hold pools, so stamina affixes on creature gear do nothing for the creature and everything for the one who picks them up; the dev `actors` layer shows the gear as it is.

**The other pools** (rage, focus, openings, ki, inspiration) get the same three effects for free once the files exist (`resource = "rage"`), but **no files now**: each is held by one class, so an "of the Berserker" ring would be dead on nine characters in ten, and a dead affix is the worst affix (Path of Exile's lesson). They return with a loot rule that weighs affixes by what the player holds, if the owner wants one (§14 q3).

## 6. The other systems, surveyed

The issue's list, each system with the hook it has, the candidate it would take, and the recommendation. "Yes" means an issue in §13; "data" means a file and nothing else; "no" means not now, with the reason.

| System | The hook today | Candidate | Recommendation |
|---|---|---|---|
| Mana | `max_mana`, `mana_regeneration` (exist); `Actor::mana_cost` | of Thrift (§4); mana on a kill (of Siphoning) | **yes** (A1, E1) |
| Spells, study, wands, rods | §4 | §4 | **yes** (A1, E2); rods through of Thrift |
| Stamina | §5 | §5 | **yes** (B1, E1) |
| Resources (the class pools) | the same hooks as stamina's | one file per pool | **no** for now: dead on most characters (§5) |
| Potions | `take_draught` reads `TalentMods::potion_kept` (Thrifty drinker) | **of the Alchemist** (suffix): a potion is not used up N % of the time; `potion_kept` (new), category `potions`, ring, amulet, gloves; 3: 10–15; 8: 15–25; the one draw only when the percent is above 0 (as the talent's today) | **yes, small** (E2). "Potions heal more" would be a second hook in `drink_potion`'s Heal arm: left out, healing has its affixes |
| Burden | Burly, of the Mule (`carry`), Featherweight (`weight`) exist | the dwarves' `ignores_burdened` as a valueless affix | **no**: carry covers it; a binary affix on the bar's thresholds would surprise |
| Stealth and noise | of Stealth (`stealth`), of Shadows (`skill = stealth`) exist; Hide in plain sight's `silent_unburdened`, `attack_noise_percent` | quieter attacks (`attack_noise`) | **no**: a second stealth category for a rare case; the two affixes stack already |
| Skills | `skill` affixes exist (five) | ten more skills (§7) | **data** (C1) |
| Searching | `skill = search` (value); Keen searcher's `search_reach`; the passive gate `Spot` *trained* | of Seeking (§7); a `search_reach` affix | **data** for the skill; **no** for the reach (a cell of reach on a ring is Keen searcher's whole talent) |
| Doors and locks | `door_kick` (Lockbreaker) in `kick_door`; `skill = open_lock` (Pick lock's check) | **Hobnailed** (prefix): kicks break doors (and #609's chests) easier; `door_kick` (new: + to the Strength the kick is rolled against), category `kick`, boots only; 1: +2; 4: +3; 8: +4; hook `World::kick_door`'s `mighty` term (and the chests' smash roll, which reads the same mods) | **yes** (D1). Picking is a skill suffix (of the Picklock, §7) |
| Digging | `dig_percent` (Tunneler) in `dig_turn`; the pick-axe has no `kind` (tools are not magic bases) | **Delving** (prefix): digs N % faster; `dig_percent` (new), category `digging`, gloves, body; 2: 15–25; 6: 25–40; 10: 40–50; hook `dig_turn`'s `faster` term | **yes** (D1) on gloves. A `tool` item kind (magic pick-axes) is deferred: it is a new `ItemKind`, rare-name words, and a bases list, for one object (§14 q9) |
| Hunger | `World::hunger_percent` (the race's, then Lean's `hunger_percent`) | **of Sustenance** (suffix): hunger comes N % slower; `hunger` (new), category `hunger`, body, cloak, ring, amulet; 2: 10–15; 6: 15–25; 10: 25–35; hook `hunger_percent`'s talent term (the sum capped so the pace stays at least 1) | **yes** (D1): the ring of slow digestion of every roguelike; a cursed "of Hunger" waits for negative affixes (none exist) |
| Temperature | nothing feels the cold yet (`docs/dev/temperature.md`) | — | **no**: no hook; of Warmth (cold resistance) exists for the frost that hurts. When cold matters, a `warmth` effect joins the one place it is felt |
| Light | Radiant, of Light (`light`) exist; the lantern's preset (Lamplighter's `lantern_light`) | a lantern affix | **no**: the lantern is no base item; the carried light is covered |
| Status effects | `World::afflict` (resistance by damage type: poison), `status_eases` (Clotting: bleeding eases faster), `treat` | typed resistance to poison as data (of the Adder, §7); a `status_ease { status }` affix (of Clotting) | **data** for poison (C1); **no** for the ease now: it needs a status in the `Group` for one affix; return when a second status wants it |
| Diseases | `World::disease_modifier` (the stat, `disease_checks` talents, the race's `disease_check`) | **of Health** (suffix): + to every disease check; `disease_check` (new), category `disease`, body, cloak, ring, amulet; 1: +1; 4: +2; 8: +3; hook `disease_modifier`'s talents term | **yes** (D1): exposure and progression alike |
| Drains | `drain_checks` (Unbroken soul) | a sustaining affix | **no**: `docs/dev/stats.md` lists it with restoration as "not yet"; it is a resistance to a rare attack, and the temple mends drains |
| Hit points | Hale, of Vitality, of Mending exist | life on a kill | **no**: not asked; of Siphoning's twin would be one more line in `dies` if ever wanted |
| Traps | `trap_damage_percent` (Trap sense −50) in `trap.rs` | **of Wariness** (suffix): a trap that catches you hurts N % less; `trap_damage` (new), category `traps`, boots, cloak; 2: 15–20; 6: 20–30; 10: 30–40; hook the same line | **yes** (D1); with Trap sense the sum is capped at 75 (a trap always hurts) |
| Alignment | `shift_alignment` (one place) | an item that moves the score, or requires a band | **no**: alignment is earned by deeds, not worn; items with a `requires_alignment` are an equipment rule, not an affix (deferred, a line on #196) |
| Speed | Fleet, of the Wind (`move_speed`), of Celerity (`attack_speed`) exist | every action faster | **no**: a price factor below 100 is allowed only on a spell's buff on its caster so that creatures are never hastened and `World::out_of_view_for` stays exact; a worn affix would reach creatures' gear |
| Hearing | of the Fox (`skill = listen`) exists | — | nothing more |
| Vision and awareness | a unique's `Power::Vision` (`Actor::eyes`); creature sight reads the light | a darkvision affix (valueless tiers naming a vision profile, as light's name a preset) | **deferred**: the uniques' power is the one reader and darkvision on a common ring would cheapen the Bedes; revisit when the vision system has more profiles to give (§14 q10) |
| Luck and loot | `Luck::of(&talents)` (Treasure hunter, Hoard sense) | of Fortune: loot magic more often | **no**: floor loot is rolled as a level is made, so the affix would read the player's gear at that moment and reward swapping rings on the stairs; magic-find gear is a loop the owner has not asked for |
| Experience | `grant_xp` | learn faster | **no**: not asked; a note on #196 |
| Shops and favour | `appraise` (the one trade skill: `bargain` in `shop.rs`) | of Appraisal (§7) | **data** |
| Stealing | `skill = steal` (trained gates it) | of the Cutpurse (§7) | **data** (helps the trained only) |
| Weather, sun, moon; wading and ice; travel; special rooms; prayer; services; bounties; talk; summoning; creature gear; danger; gems | no number of theirs is the wearer's | — | **no** |

## 7. The data-only batch: skill suffixes and typed resistances

Files only; no code, no new effect. Two existing effects take them: `skill` (five suffixes today) and `resistance` (two). Each follows of Shadows' or of Warmth's tiers.

**Skill suffixes** (`effect = "skill"`, category `skill:<id>` in #621's form, weight 60, tiers 1: 10–20; 4: 20–35; 7: 35–50, but the gated ones 1: 5–10; 4: 10–20; 7: 20–30 as of Shadows): **of Seeking** (search; helmet, ring, amulet), **of the Eagle** (spot; helmet, amulet), **of the Stag** (athletics: kicks and speed; boots, body), **of the Leech** (heal, the physician's old name; gloves, ring), **of the Picklock** (open_lock; gloves, ring), **of Disarming** (disable_device; gloves), **of Appraisal** (appraise: the shops' bargain; helmet, ring, amulet), **of the Scholar** (spellcraft; helmet, ring, amulet), **of Letters** (decipher_script; helmet, ring), **of the Cutpurse** (steal; gloves, cloak): ten. Skills whose benefit nothing reads yet (swim, concentration, escape artist, intimidate, knowledge, music, metalcraft, woodcraft, bluff, handle animal) get none until they do; C1's test holds every shipped skill suffix to a skill with a benefit in `data/skills.toml`.

**The gate.** A skill affix adds to the skill's value "for what that skill gives, trained or not", but Spot's noticing in passing, Open lock's Pick lock, Disable device's Disarm, Heal's treating, Steal's stealing, and Survival's foraging are gated on the skill being **trained at all** (`Skills::trains_effect`). So of the Eagle, of the Picklock, of Disarming, of the Leech, and of the Cutpurse help the trained only (a better check), and a thief finds them; the untrained get a value nothing reads. The wiki's skills table marks the gated skills, and the suffix row says "for those trained in it". No code changes this; it is the skill system's rule.

**Typed resistances** (`effect = "resistance"` with a `damage_type`, category `resistance:<type>`, of Warmth's tiers 1: 10–15; 3: 15–25; 5: 25–35; 7: 35–45, capped at 75 with everything worn): **of the Adder** (poison; body, cloak, ring, amulet: the poisoned status meets it as a dose, so a Venomous blade's rider and a centipede's bite are both softened), **of the Storm** (electricity; cloak, shield, ring, amulet: lightning traps and Call lightning), **of Glass** (acid; body, gloves, boots, amulet), **of the Anvil** (force; shield, ring, amulet: Magic missile, Arcane bolt), **of the Living** (necrotic; body, amulet: Cause wounds, Blight). The physical kinds get none: Protection is their resistance. Elemental weapon prefixes for electricity and acid (a Shocking blade, a Corrosive one) are one file each too, and belong to a weapons pass, not this one (§14 q11).

## 8. What could break

| Risk | Where | Answer |
|---|---|---|
| **The affix list shifts loot** | `loot::roll` draws an affix by weight among those that may roll: a new file changes which affix a seed rolls, so every insta snapshot and test that names a rolled affix, and any replay whose run rolled a magic item, changes | inherent to adding content (#621 does the same); each wave's files land in **one** PR per wave so the churn is reviewed once; the `levels` and `loot` streams themselves do not move. Saves load (affixes by id); a replay re-rolled with new files diverges at its first magic item and says so, as the policy allows |
| **The `Effect` list is a hot file** | `Effect`'s variants, `COUNT` (30 + the stats) and the hand-numbered `index()`, `all()` (the wiki's order), `Mods::values: [i32; Effect::COUNT]`, `describe::phrase`, `wiki/magic.rs::effect_words` and `tier_value`: three PRs adding variants at once would conflict in all six | **one foundation PR** (F1) adds every variant of §4 to §6 with its parse, `Mods::add` arm, phrase, and wiki word, and no hook; the hook PRs then touch only their own modules. `index()` is in-memory, never saved, so renumbering is free; `all()` inserts by group (mana, spells, stamina, utility), never at the end |
| **`Group` changes shape** | from a tuple to a struct with `resource`; `AffixDef::group`, `Mods::add`, the loot draw's "not yet on the item" filter, the one-of-each check on load | F1 too; the struct's `Default` keeps the three old readers one-line changes |
| **Early returns skip items** | `World::resource_rate` and `Actor::weapon_stamina` return before reading mods when the actor has no talents | the condition becomes "no talents **and** `Mods::worn` empty" (the `is_empty` check exists); a test wears the item on a talent-less actor |
| **A pool above its maximum** | Tireless taken off: `resource::pass` clamps at the next step, the status bar clamps (`own_resources`), but the rules see 12/10 for the rest of the action | `World::keep_resources_within_max` after every player action (one call beside `keep_mana_within_max`; one line in `apply`); a test takes the ring off and reads the pool |
| **Mana costs below 1** | of Thrift ×2 + Frugal caster | the sum of percents is capped at 50 off before rounding; `mana_cost` keeps "at least 1 for a cost above 0"; a test stacks three |
| **The armoured grip flips** | a Mage-wrought piece eased to 0 makes `worn_spell_cost` 0, so `Actor::spell_skills` chooses Toucher or Sniper over Battle caster, and Battle caster's marks stop | intended: nothing is left to ease. The wiki's Fighting styles page says a caster in eased armour fights in the plain grip; a test checks the switch both ways |
| **Baleful below the floor** | threat 22 − 2 = 20; the blows' `THREATS` range | `spell_threat` clamps to `THREATS.start()` as a weapon's `Threat` does; a test at the floor |
| **Lingering past a status's `max`** | +2 on a `max = 3` status with Presence's +1 | `Effects::add` clamps already; the wiki says it. No change |
| **Hidden numbers** | the item description prints an identified affix's value ("+2 to caster level", "abilities cost 12% less mana"), as Mystic's "+6 to maximum mana" does today | unchanged convention: an identified affix's own number is the item's, not a modifier, a potential, or an odds; the abilities menu already prints a spell's mana and an active's stamina, and shows the discounted figure, as it shows Frugal caster's |
| **Unidentified effects leak** | a worn of the Magus raises the spell to-hit; nothing shows it | nothing new shows: caster level is not on the character screen (`docs/dev/spells.md`), spell to-hit and lift are never shown, the menu's costs are the one visible change (an attentive player may notice a cheaper spell, as they may notice Mystic's larger pool: the spec's rule) |
| **Creature gear** | creatures read `caster_level`, `mana_cost` (summoners), `spell_*` (unused: their spells are announced only), resource effects (dead: no pools), `mana_on_kill` (a kill refills), the utility effects (`door_kick`: creatures never kick; `dig_percent`: never dig; `hunger`: never hunger; `trap_damage`: `spring_trap`'s victim may be a creature, so of Wariness works on a kobold's boots, fine; `disease_check`: a creature is exposed only by a disease-bearing bite (`status.rs`'s `on_hit`), and then its gear counts) | each hook is where a creature's reading already is or is not; no hook is made player-only by hand |
| **A means check on the object** | Mage-wrought must roll only on a piece with a `spell_cost_percent`, a field of the base object, not of its kind | `loot::may_roll(affix, kind, def)` is the per-object check today (a range for Range, a Strength for Draw, a breakable missile for Durability); one more arm reads `spell_cost_percent`; `Effect::allowed_on` (by kind, at load) keeps it to body, helmet, and shield |
| **No draws added** | every hook is a read; of the Alchemist's draw happens only when the percent is above 0, as Thrifty drinker's | the "no draw without it" tests gain the affix cases |
| **The wiki's affix tables** | `magic-prefixes.md` and `magic-suffixes.md` print every file's effect, tiers, bases, and (after #621) category by themselves | each new effect needs its `effect_words` and `tier_value` arm (F1), else the table test fails |

Nothing here needs `FORMAT_VERSION`: no struct that is saved changes, no index that is saved moves (the content fingerprint does not include affixes; resources and statuses are untouched).

## 9. Determinism

| Draw | Stream |
|---|---|
| which affix an item rolls, its tier and value | the level's `loot/<level id>` or the kill's `drop/<actor id>` stream, as today; new files change the pick, not the stream |
| of the Alchemist's keep | the **gameplay** stream, one draw, only while the percent is above 0 (Thrifty drinker's rule) |
| everything else in this document | none: every hook is a pure read of `Mods::worn` or `Mods::local` |

Runs that wear none of the new affixes keep every hash they have; a fight with and without the affixes' data (worn by nobody) hashes the same (the pattern of `no_draws_while_no_rank_qualifies`).

## 10. Talents

No new `[[system]]`: every affix joins a system in the list at a hook its talents already use (`mana`: Frugal caster; `spells`: Arcane training, Keen mind, School focus; `stamina`: Deep lungs, Steady breathing, Drillmaster, Second breath; `doors`, `digging`, `hunger`, `traps`, `diseases`, `potions`, `skills`, `damage_types`: one talent each at the very line). The coverage bar asks nothing. Two talents still, so the affixes have something that cares about them, each one hook, each a file under `data/talents/`, `systems = ["magic_items", ...]`:

| Talent (tree) | Needs | Effect | Hook (one read) | Issue |
|---|---|---|---|---|
| **Attuned** (scholar) | Intelligence 12, level 6 | the caster level your magic items give you counts half again, rounded up (+1 → +2, +2 → +3, +3 → +5) | `Actor::caster_level`'s `items` term: `items + TalentMods::attuned_percent × items / 100`, rounded up | T1 |
| **Bellows** (vigour) | Toughness 12 | the stamina your gear gives you is doubled | `Actor::resource_max`'s items term for a pool (`TalentMods::geared_resource_percent`, 100) | T1 |

Names in the register: *attuned* is the old sense of brought into tune; *bellows* the smith's, for a fighter's breath. Both are picks, not class abilities; neither needs a choice.

## 11. What the player sees; the wiki, the spec, and the docs

| Where | What |
|---|---|
| Item names | "a Tireless breastplate of Breath", "a Mage-wrought chain mail", "Grim Charm, an amulet" (rare names unchanged); unidentified "(magic)" as today |
| Item description | one phrase per affix once identified (`describe::phrase`): "+2 to caster level", "abilities cost 12% less mana", "hinders casting 10 points less", "+2 to hit with spells", "spells threaten a critical on 1 more roll", "spells do up to 8% more", "spells reach 1 cell farther", "buffs and hexes last 1 degree longer", "+3 to maximum stamina", "stamina comes back 25% faster", "weapon feats cost 1 stamina less", "+1 stamina on a kill", "+2 mana on a kill", "+1 to the level of wands", "kicks break doors 3 easier" (wording: "your kicks are stronger"), "digs 25% faster", "hunger comes 15% slower", "traps hurt you 20% less", "+2 to resist disease", "a potion is not used up 15% of the time" |
| Never shown | caster level (not on the character screen yet), spell to-hit, the lift, the paces, the kick roll, the check modifiers: all as today |
| The abilities menu | a spell's mana and an active's stamina as the hooks now price them (the menu prints the discounted cost today for Frugal caster and Drillmaster; nothing new) |
| Dumps | `inventory`'s names; `actors` (dev) shows the truth as it does; JSON `magic` per stack once identified |
| Wiki | the Magic items page's generated prefix and suffix tables gain the rows by themselves (effect words, tiers, bases, category); a new generated table **`magic-systems.md`** (`xtask/src/wiki/magic.rs::systems`, one line in `generate()`): affix → system → what it changes → the talent beside it, the owner's "what systems have affixes" answered in one place; the Mana page's "what changes the pace" and a new "what changes costs" row (of Thrift); the Magic page's casting table rows (of the Adept, of the Magus, Baleful, Potent, of Farcasting, of Lingering); the Resources page's sentence on stamina gear; the Skills page's gate marks; the Fighting styles page's line on eased armour; Doors, Digging, Hunger, Traps, Diseases, Potions: one sentence each naming their affix |
| Spec | `spec/08g-magic-items.md` "Effects" (the new effects, local or global, and their one reader each) and "Affixes" (the counts and names); `08e` "Mana" (the cost hook), "Spells" (the six reads), "Resources" (the three resource effects); `08d` (the kick and dig terms), `08h` (hunger's), `08e` "Diseases" (the check) |
| docs/dev | `magic-items.md` (the effect list and the hook list), `mana.md`, `spells.md`, `resources.md`, `weapon-abilities.md`, `doors.md`, `digging.md`, `hunger.md`, `traps.md`, `status-effects.md`, `potions.md`, `skills.md`: one sentence each at the hook they describe |

## 12. Choices made along the way

| Choice | Why |
|---|---|
| Mana cost is one global cut (`mana_cost`), not a spell cost and a rod cost | every spend goes through `Actor::mana_cost` already; one hook, one affix, one category |
| Mage-wrought is local and rolls only on hindering pieces | the surcharge is the piece's; an affix that could roll on a cloth robe would be nothing |
| Spell to-hit and threat reach own casts only | scrolls and wands cast with the reader's Dexterity by the magic design's choice; the scales set the pattern |
| No spell critical multiplier | the cap holds a critical's roll; the multiplier would be a lie on the label |
| Resource effects name their pool in data | the skill affixes' pattern; stamina today, any pool later without code |
| No affix grants a pool | Mystic's precedent; a grant on wear needs an un-grant on removal |
| No files for the class pools | dead on most characters |
| of Ease is global | no attack effect; a local reading would answer the same |
| Skill suffixes for the gated skills too | a thief with Open lock trained is exactly who finds of the Picklock useful; the wiki says who it helps |
| No magic-find affix | floor loot is rolled at level creation; a ring swapped on the stairs is a loop the owner did not ask for |
| No all-action speed | the rest batching's exactness rests on creatures never being hastened |
| A foundation PR for the hot files, then parallel hook PRs | the `Effect` list and its five readers are one file each; three PRs inserting at once would conflict in all of them |
| One PR per wave of data files | each file shifts which affix a seed rolls; the snapshot churn is reviewed once a wave |
| Two talents under existing systems, no new tag | affixes extend `magic_items`; the bar is met; the talents give the gear a reason to be sought |

## 13. Implementation plan

Each item is one issue, one branch, one PR, updating the spec, the `docs/dev/` topic at the hook, and the wiki in the same PR. **None adds saved state, an action, or a key**; the "Save" column is here because the template asks for it. Sizes: S about a day, M two to three, L a week.

| # | Issue title | Sections | Save | Depends on | Size |
|---|---|---|---|---|---|
| **F1** | Affix effects for the systems: the `Effect` variants of §4–§6 (`mana_cost`, `spell_ease`, `spell_to_hit`, `spell_threat`, `spell_lift`, `spell_reach`, `spell_duration`, `mana_on_kill`, `wand_level`, `resource_max`, `resource_rate`, `active_stamina`, `resource_on_kill`, `door_kick`, `dig_percent`, `hunger`, `trap_damage`, `disease_check`, `potion_kept`) with their parse (`resource`, the means check for `spell_ease`), `Group` as a struct with `resource`, the `Mods::add` arms, the `describe` phrases, the wiki words and tier forms, `all()` by group; **no hook reads them yet** (as `caster_level` sat unread), **no affix file**; the loader refuses a resource effect without a known resource | §3, §8 | — | #621 merged (the `category` field and its check) | M |
| **A1** | Spellcasting affixes: the hooks in `cast.rs` (`mana_cost`, `worn_spell_cost`), `spell_scale.rs` (to-hit, lift, degrees), `spell_reach`, `cast_single`'s threat (`Actor::spell_threat`); the files of the Adept, of Thrift, Mage-wrought, of the Magus, Baleful, Potent, of Farcasting, of Lingering; the Magic and Mana wiki rows; `world/affix_spell_tests.rs` (each hook with and without the item, unidentified applies, taken off stops, the 50 cap, the armoured grip flip, the threat floor, scrolls and wands untouched, creature gear) | §4 | — | F1 | M |
| **B1** | Stamina affixes: the hooks in `resource.rs` (`resource_max`, `resource_rate`, the early returns), `weapon_active.rs` (`weapon_stamina`), `keep_resources_within_max`; the files Tireless, of Breath, of Ease; the Resources wiki sentence; `world/affix_stamina_tests.rs` (a larger pool, the clamp on removal, the pace, the floor of 1, a talent-less wearer, nothing for one without the pool, a creature's gear doing nothing) | §5 | — | F1 | M |
| **C1** | The data-only batch: the ten skill suffixes and five typed resistances of §7; the Skills page's gate marks; a test that every shipped skill suffix names a skill with a read hook, and that every resistance names a non-physical type | §7 | — | #621 (categories) | S |
| **D1** | Utility affixes: the hooks in `door.rs` (`mighty`), `dig.rs` (`faster`), `hunger.rs` (`hunger_percent`), `trap.rs` (the damage percent, capped at 75 with Trap sense), `disease.rs` (`disease_modifier`); the files Hobnailed, Delving, of Sustenance, of Wariness, of Health; the Doors, Digging, Hunger, Traps, Diseases wiki sentences; `world/affix_utility_tests.rs` | §6 | — | F1 | M |
| **E1** | On a kill: `World::siphon_mana` in `dies`, `resource_on_kill` in `gain_resource`; the files of Siphoning and of the Hunt; tests (a kill refills, never past the maximum, a creature's kill too, nothing on a kill by a trap) | §4, §5 | — | A1, B1 | S |
| **E2** | Potions and wands: `potion_kept` in `take_draught` (the one draw only above 0), `wand_level` in `wand_caster_level`; the files of the Alchemist and of Wands; tests | §4, §6 | — | A1 | S |
| **T1** | Talents Attuned and Bellows: `TalentEffect`s, the two reads, the files, `world/affix_talent_tests.rs`, the Talents wiki rows | §10 | — | A1, B1 | S |
| **X1** | Balance pass and the `magic-systems.md` table: roll 10 000 items at levels 1, 4, 8, and 12 with `warlock-cli` (a dev `roll` subcommand or a test printing the shares), check no new affix takes more than its weight's share of its bases and no category starves another, tune weights in data; `xtask/src/wiki/magic.rs::systems`; a note in the PR of what was measured | §11 | — | E1, E2, T1 | S |

**Waves** (at most about three PRs at once that add actions or save state: here none does; the limit that binds is the hot files):

1. **F1** alone (the hot files: `Effect`, `Group`, `Mods`, `describe`, the wiki words). **C1** may run beside it: data only, no file in common, though its loot shift and F1's wiki test both touch `xtask/src/wiki/mod.rs`'s rows (insert by group).
2. **A1, B1, D1** in parallel: each adds hooks in its own modules and files under `data/affixes/` (new files never conflict), and its own test file; the wiki pages they touch differ (Magic and Mana; Resources; Doors, Digging, Hunger, Traps, Diseases). The one shared line is `docs/dev/magic-items.md`'s hook list: each adds its own sentence at its own hook (the false-conflict trick).
3. **E1, E2, T1** in parallel (E1 touches `fight.rs` and `resource.rs`, E2 `potion.rs` and `zap.rs`, T1 `talent.rs` and two hooks already changed in wave 2).
4. **X1**.

**Tests each item carries**: `content/affix.rs` (the new effects parse; a resource effect without a resource, with an unknown one, and `spell_ease` on a cloak are refused; `Group` equality; the local/global table); `magic/mods` (the sums, two Tireless rings add, a resource effect keyed by its pool); the `world/affix_*_tests.rs` files above, each with the item worn, unidentified, taken off, and on a creature; the hash test (a run wearing none of them hashes as before the files existed); a text snapshot of one item description per wave (`item-description-spell-affix`, `-stamina-affix`, `-utility-affix` in `screen_snapshots.rs`); a replay per wave (`replay --create`, `--verify`); `cargo xtask wiki` with the tables' rows. **No golden**: nothing here is produced by the GPU.

## 14. Open questions, each with its default

Each has a default so that nothing blocks approval; the owner's line comments change what they change.

| # | Question | Default |
|---|---|---|
| 1 | Should Mystic (maximum mana) also roll on weapons, as of the Adept does (a caster's staff)? | **No**: Mystic's bases stay; of the Adept is the weapon affix for casters |
| 2 | of Ease: global (any weapon active a point cheaper while worn) or local (only the actives of the piece that carries it)? | **Global** (§5): the same answer with less code |
| 3 | Files for the class pools (rage, focus, ki, openings, inspiration) now, since the effects are generic? | **No**: dead on most characters; return with a loot rule that weighs affixes by what the player holds, if ever |
| 4 | Should a stamina affix grant the pool to one who lacks it? | **No** (§5) |
| 5 | A spell critical multiplier affix beside Baleful? | **No**: the cap holds the roll |
| 6 | Potent: raise the tier's `max_lift` as Keen mind does (so it is never wasted), or only the lift within it? | **As Keen mind**: it joins `more` |
| 7 | of Siphoning for creatures' kills too? | **Yes**: the hook is `dies`, whoever killed; a creature with a pool refills it |
| 8 | Negative affixes (of Hunger, Cursed) now that categories exist? | **No**: none exist; a separate design if wanted (it touches identification and the item lists' colours) |
| 9 | A `tool` item kind so pick-axes roll Delving themselves? | **No** for now: gloves carry it; the kind is one `ItemKind`, rare-name words, and a bases list when a second tool wants magic |
| 10 | A darkvision affix (valueless tiers naming a vision profile)? | **Deferred**: it would cheapen the Bedes' power; revisit with more profiles |
| 11 | Elemental weapon prefixes for electricity and acid (Shocking, Corrosive) in C1? | **Not in C1**: a weapons pass; one file each when wanted |
| 12 | Tier levels above 12 (of the Adept's +3 at 12, Potent's third at 11): keep them, or stop at the Ascmearc's floors? | **Keep one top tier** at 10–14 per affix for deeper places, as the existing affixes do (Flaming's 8, most 7) |
| 13 | Names: of the Adept, of Thrift, Mage-wrought, of the Magus, Baleful, Potent, of Farcasting, of Lingering, of Siphoning, of Wands, Tireless, of Breath, of Ease, of the Hunt, Hobnailed, Delving, of Sustenance, of Wariness, of Health, of the Alchemist, and §7's | **As written**; a rename is a file rename before the wave merges (ids are the file names) |
| 14 | F1 as a separate foundation PR, or folded into A1 with B1 and D1 waiting? | **Separate** (§8, §12): it is the one PR that touches the six hot sites |
| 15 | Attuned's and Bellows' trees and needs | **scholar (Intelligence 12, level 6)** and **vigour (Toughness 12)** |
| 16 | The categories of today's 50 affixes where a new one shares a family (mana: Mystic `mana`, of Clarity `mana_regeneration`; stealth: of Stealth `stealth`, of Shadows `skill:stealth`) | **#621's names stand**; this document's new categories are listed per affix and chosen not to collide |

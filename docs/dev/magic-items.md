# Magic items

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08g-magic-items.md. Keep it current with the code, as CLAUDE.md says.

Magic items (spec §8 "Magic items", #149) live in `warlock-core`: `content/affix.rs`
(`data/affixes.toml`, the settings file with the rarity weights and rare names, and
`data/affixes/`, one file per prefix or suffix named by its id, read in path order; see
"Content directories" in `docs/dev/content.md`: `ItemKind`, `Effect`, `AffixDef` with tiers,
`Rarity`), base item fields in `content/object.rs` (`kind`, `level`, `frequency`,
`attack_cost`; the weapon skill `category` names the item in descriptions, `weapon_noun`; loot
also draws scrolls, which have no `kind` and are never magic), and
`magic/` (`mod.rs`: `Magic`/`Affix`, the instance on a `Stack` (saved, hashed), `check`;
`mods.rs`: `Mods::local` (an item's own and its attacks') and `Mods::worn` (what an actor wears
and wields, global), the one aggregation every rule reads; `loot.rs`: `roll`, its draw order;
`name.rs`: the names around `Stack::names`, the item as it is made (its gems, #191, and its
material, #193), so the order is [prefix] [ornament] [material] base [suffix] and a rare item's
base shows its gem; `describe.rs`: base
stats, gems with their size and cut, and affixes in words for the item description screen
(never the item level, a hidden number; #637);
`numbers.rs`: `Numbers::of(stack)`, what an item gives in numbers (#552: a weapon's to-hit and
damage with its material and local affixes through the very `Mods::sharpen` a strike uses and
`Profile::bare`, a launcher's or thrower's range, ammunition's additions, a worn thing's global
to-hit and damage (`Mods::global`, one stack's global effects), Dv as armour + shield + parry +
the local Armour/Shield/Parry + the global Dodge/Deflection/Natural, Pv as `Stack::protection`),
figured on `Stack::as_known` so an unidentified affix moves no number; `text` is the item lists'
form (`Lore::listed` = `Lore::describe` + ` (numbers)`, which the object screens, the equipment
screen and its choice, look mode's piles, and the `inventory` dump layer use; messages keep
`describe`), `lines` the description's (`describe::lines` adds them after the base stats);
`dice_text` is the one damage-dice form, which `OwnAttack::damage_text` uses too).
`loot::roll` first lets a find of grade `any` be a loose gem (`gem::find`), then sets an item
of a class with an `[[ornament]]` with a gem (`gem::ornament`), and last draws its material
(`variant::draw`, step 8: by the materials' `rarity` at the item level; `loot::roll_from` takes
a `variant::Source`, and creature drops pass `Source::Gear(creature level)`, at the item level
`world/loot.rs::drop_level` gives them, below); its own parts are
the material first, then the gem. Creature drops try `gem::find` before the magic item
(`docs/dev/objects.md`, "Gems" and "Materials and variants").
`world/loot.rs` has `World::identify` (the API the scroll of identify calls from its read action,
`docs/dev/scrolls.md`; it identifies a labelled scroll's or wand's type too, `World::learn_scroll`,
and a wand's charges, `Stack::identify_charges`; wands and rods are docs/dev/spells.md "Wands" and "Rods"),
creature drops (`drop/<actor id>` stream), and the hit-point clamp after equipment changes
(to `Actor::lasting_max_hp`: a penalty's lower maximum passes). Stats raised by magic are
`Actor::effective_stats` (the stored scores, raised by `Mods::worn`, then lowered by status
effects, Weak, and stat damage; `raised_stats` is before those, for the character screen's marks
and level-up hit points), the one source every rule reads; an attack profile is built as those stats, the
weapon skill's benefit, the fighting style's (with the two-weapon penalty it eases), the
affixes, then the combat stance. The regeneration affix goes
through `World::healing_rate` like every other pace change, the mana affixes (Mystic,
`max_mana`; of Clarity, `mana_regeneration`) through `Actor::max_mana` and `World::mana_rate`
(`docs/dev/mana.md`; mana follows the maximum down with `World::keep_mana_within_max`); the
caster level effect (`caster_level`, no built-in affix yet) through `Actor::caster_level`,
its items term lifted by Attuned (`TalentMods::attuned`: `attuned_percent`, rounded up; T1, #634)
(`docs/dev/spells.md`); the Protection affix (Hardened, an
item effect, local, on armour kinds: `ItemKind::is_armour`) through `Stack::protection`, which
`Actor::protection` sums (docs/dev/combat.md); the typed effects (docs/dev/combat.md "Damage
types"): `Effect::Elemental` (an attack effect, local on what attacks: `Mods::elemental` is one
rider per kind, `sharpen` adds them to the profile) and `Effect::Resistance` (global:
`Mods::resistances`, which `Actor::resistances` adds to what is worn and what the creature is),
each with the affix's `damage_type` (`AffixDef::damage_type`, part of its `Group`, so an item
may hold one of each kind). **Categories** (#621): `AffixDef::category` (required in data) is the
rule loot and `Magic::check` read: an item never has two affixes of one category
(`loot::roll_affix` leaves out the categories already rolled; `Magic::check` refuses a repeat in a
save, a legend's `[[item]]`, or a unique). `content::affix::check_category` makes every affix of
one `Group` name one category (so one per category is one per group, which `Mods` relies on) and
a category on both prefixes and suffixes one group. Tests: `content/affix.rs`
(`affix_categories_are_checked`), `magic/tests.rs` (`no_rolled_item_repeats_an_affix_category`,
over many rare rolls at four item levels). Hard
rule: presentation reads an item's affixes only through `Stack::known_magic` (none until
identified; `Stack::magic` is crate-private), names only through `Stack::describe` /
`title`, and numbers only through `magic::Numbers` (on the known stack), so unidentified affixes
cannot leak; dev dumps use `describe_truth`. Tests: `world/numbers_tests.rs` (a +1 sword, an
unidentified armour's base numbers and its affixes once identified, a shield, a launcher's
range, ammunition, a worn thing's to-hit). A new effect is a
variant of `Effect` (id, value range, local/global, `allowed_on`; `index()` and `all()` insert
it by group, never at the end), its reader in one rule, a
phrase in `describe.rs`, and a wiki word and tier form in `xtask/src/wiki/magic.rs`
(`effect_words`, `tier_value`). Affixes are saved by id, so
adding affixes keeps saves; their values and tiers are checked on load, leniently (#637): a
ladder regraded since a save was made may move the range a saved tier holds, so
`World::from_state` first re-tiers every magic stack (`world/retier.rs`: what actors carry, the
levels' piles, remembered piles, and chests) with `Magic::retier`, which moves an affix whose value
its tier no longer holds to the first tier that holds it, or the nearest when none does, the value
kept; `Magic::check` then accepts a value no tier holds on its nearest tier. Each move is a line in
`World::retiered`, which the game logs (`crates/warlock/src/save.rs`); nothing is refused and the
content fingerprint (which leaves affixes out) is unchanged (`world/retier_tests.rs`).

**The system effects** (#628, docs/design/affixes-systems.md §4–§6): `Effect` holds, by group,
the spell ones (`mana_cost`, `spell_ease`, `spell_to_hit`, `spell_threat`, `spell_lift`,
`spell_reach`, `spell_duration`, `mana_on_kill`, `wand_level`), the resource ones
(`resource_max`, `resource_rate`, `active_stamina`, `resource_on_kill`), and the utility ones
(`door_kick`, `dig_percent`, `hunger`, `trap_damage`, `disease_check`, `potion_kept`), each with
its parse, `Mods::add` arm (a sum), `describe::phrase`, and wiki words. The hooks came first,
each in its own PR (A1 spells, B1 resources, D1 utility, E1 on a kill, E2 potions and wands),
tested on fixture affixes; **the shipped files** came after (P1, #635), one under
`data/affixes/` each, on the ladder: of the Adept, of Thrift, Mage-wrought, of the Magus,
Baleful, Potent, of Farcasting, of Lingering, of Siphoning, of Wands, Tireless, of Breath, of
Ease, of the Hunt, the five pool maxima (of Fury, of Steadiness, of Stillness, of Guile, of the
Muse: dead loot on a character without the pool, by the owner's choice), Hobnailed, Delving, of
Sustenance, of Wariness, of Health, and of the Alchemist; Mystic gained `weapon`, Featherweight
and of the Bear `tool`. `magic/tests.rs`, `every_shipped_affix_effect_is_read_by_its_hook`,
names each effect's reading module (an exhaustive match) and fails on a shipped affix whose
effect that module never reads (a dead effect). **The skill suffixes** (C1, #636): of Seeking,
of the Eagle, of the Stag, of the Leech, of the Picklock, of Disarming, of Appraisal, of the
Scholar, of Letters, of the Cutpurse, twelve tiers each (levels 1 to 12; 8–12 to 45–50 for the
value skills, 4–6 to 28–32 for the five gated on `Skills::trains_effect`, which help the
trained only), weight 60, category the skill's id. `every_shipped_skill_suffix_raises_a_read_benefit`
holds each to a skill with benefits, every one read through `Actor::skill_effect` (so the
items' bonus counts) in the module `skill_reader` names; the shops' bargain read
`Skills::effect` past the items until C1 (`shop::favour`). **Damage is more than one category** (owner,
#635), so a weapon may roll more than one damage affix: `physical_damage` (+damage: Sharp,
Vicious), `extra_damage` (an extra die: Heavy and the suffix of Wounding), and one per element
(`fire_damage`, ...); the one-per-category rule stands. **The spell hooks** (A1, #629; `world/affix_spell_tests.rs`): `mana_cost` in
`Actor::mana_cost` (joins Frugal caster's percent, 50 off at most), `spell_ease` in
`Actor::worn_spell_cost` (each piece less its own, never below 0: a fully eased piece leaves the
armoured grip), `spell_to_hit`, `spell_lift` (beside Keen mind's), and `spell_duration` in
`world/spell_scale.rs`, `spell_reach` in `Actor::spell_reach` (beside School focus's, the range or
length only), `spell_threat` in `Actor::spell_threat` (22 less it, never below `THREATS`' floor;
an own cast only, `reflect::Shot::threat`). **The potion and wand hooks** (E2, #633;
`world/affix_draught_tests.rs`): `potion_kept` in `Actor::potion_kept` (Thrifty drinker's chance
plus the worn, held to `potion::MAX_KEPT` 50 unless the talent alone is above it; `take_draught`
draws only above 0), `wand_level` in `Actor::wand_caster_level` (beside Wand mastery's). Each hook's tests bring fixture affix files
(`ContentFiles::insert("data/affixes/zz_fixture.toml", ...)`, as `magic/tests.rs` does).
`spell_ease` is an item effect (local: the piece's own `spell_cost_percent`), allowed on body
armour, helmets, and shields, and `may_roll` keeps it to objects with a `spell_cost_percent`.
**Pooled effects** (`Effect::is_pooled`: the four resource ones) name their pool in data
(`resource = "stamina"`, `AffixDef::resource`), refused without one or with an unknown one, and
left out when content has no resources (as skill affixes without skills; `content::affix::Lookups`
holds the id lookups, `None` for a table content lacks; the table is read after resources). They
sum by (effect, resource) in `Mods::pools`, read with `Mods::resource(effect, id)`, never into
`Mods::get`. Their hooks (B1, #630; docs/dev/resources.md): `resource_max` in
`Actor::resource_max` (the worn term lifted by Bellows: `TalentMods::geared_resource`, by pool;
T1, #634; `world/affix_talent_tests.rs`), `resource_rate` in `World::resource_rate`, `active_stamina` (keyed on
stamina, global) in `Actor::weapon_stamina`. **On a kill** (E1, #632; `world/siphon.rs`,
`world/siphon_tests.rs`): `mana_on_kill` and `resource_on_kill` in `World::siphon`, called at
the kill point of an attack (`fight.rs::resolve`, `spell.rs::spell_strikes`), the slain within
`SIPHON_RANGE` (3, a radius); never from `dies`, so a status tick's or a trap's death gives
nothing. **`Group`** is a struct `{ effect, skill, damage_type, resource }` (`Group::of` for an
effect that names nothing else): one group per (effect, resource), so `check_category` keeps
Tireless (stamina) and of Fury (rage) apart. **`ItemKind::Tool`** (`ALL` is a slice; the `tools`
item class holds the kind; rare names' second words `tool = [...]` in `data/affixes.toml`): a
kind for digging tools, which the pick-axe takes in D1. **Tiers**: at most `MAX_TIERS` (12) per
affix, a loader check. Every shipped ladder (X1, #637, regraded the old affixes but Flaming,
Freezing, and Venomous, which W1 does; the two light affixes keep their three glow presets, and
Brutal and of Ruin their one tier) runs from its first level to item level 12, a tier every level
or two where the values can step (a flat +1 to +3 has five tiers: 1, 1–2, 2, 2–3, 3), its first
and last values as before. The hand-made items that name tiers (the uniques, the showcase's
`[[item]]`s) were moved to the tier of the same top value. **The balance pass**
(`magic/shares_tests.rs`): 10 000 magic items at item levels 1, 4, 8, and 12; every affix's picks
lie within four standard deviations and a tenth of its weight's share of the picks it could have
been (none over, none starved), and every rolled tier is in the window (`--nocapture` prints the
shares). **The tier window** (`loot::tier_window`, `TIER_WINDOW` = 4): loot draws a
tier evenly among those found at the item level whose level is within 3 below it, the top one
alone once the item level is past the last tier (or when none is that close), one draw as before
(`magic/tests.rs`, `tiers_roll_from_a_window_below_the_item_level`). **A drop's item level**
(`world/loot.rs::drop_level`): the creature type's level, at least the floor's item level, at
most `MAX_ITEM_LEVEL`; floor loot keeps the floor's (`world/magic_tests.rs`,
`a_drop_rolls_at_the_creatures_level_at_least_the_floors`).

**The utility hooks** (D1, #631): each worn effect joins its talent's term at the one line that
reads it, through `Actor::mods` (so unidentified gear works, and a creature's gear wherever the
rule reads a creature): `door_kick` in `World::kick_force` (`world/door.rs`, a door's kick and a
chest's), `dig_percent` in `dig_turn`, `hunger` in `World::hunger_percent` (subtracted: hunger
comes slower; the pace at least 1), `trap_damage` in `trap_deals` (subtracted from Trap sense's
percent, the sum at most `TRAP_DAMAGE_CUT` = 75 off), `disease_check` in
`World::disease_modifier`. Tests on fixture affixes: `world/affix_utility_tests.rs`. The
pick-axe is the one `tool` base item (`level = 3`, `frequency = 10`); `Numbers::of` shows a
tool nothing, and `describe`'s base stats give any digging tool's power ("Digs at 10"). Delving,
Featherweight, and of the Bear roll on tools (P1, #635), so loot finds a magic pick-axe; loot
that must be magic draws only bases some affix rolls on (`loot::can_be_magic`), and a magic
item's shape only among those its base can take (`loot::rolls_any`).

## Unique items

Unique items (spec §8 "Magic items", #423, docs/design/adventurers-guild.md §9) are data in
`content/unique.rs` (`data/uniques/` and each campaign's `data/uniques/`, one file per item,
read through `ContentFiles::table("uniques")` after objects, affixes, and gems and before
everything that places objects): `UniqueDef` with its `name`, base `object`, the template
`stack` (one, rare, unidentified, made by `level::magic_of` and `item_part`, the code an
`[[item]]` uses), its `power` (`Power`), `phrase`, `lore`, and `campaign`. `Magic::unique` is
the item's id (saved, hashed; `skip_serializing_if` none, so other items' saves are unchanged);
`Magic::check` refuses an unknown id, another base, or another rarity than rare.

- **Placement**: `parse_stacks` (`content/object.rs`) makes `unique:ID` through
  `Content::place_unique` (`Uniques::place`), which marks it placed (an `AtomicBool` per unique,
  set while `Content::load` runs: every placement is parsed then, once per legend cell that
  names it) and refuses a second placement and a count. `level::equip_ids` takes `unique:ID` in
  `wield`/`wear` for the carried stack of that unique (not a placement). Gear (`gear::make`)
  and kits keep a unique as it is, and `kit::check_kept` refuses a `magic` chance on one. The
  Ascmearc's four are placed by their owners' gear lines (the criminals, #424,
  `docs/dev/creatures/outlaws.md`), so no other content may name them: a test that needs a
  placement defines a unique of its own (`world/unique_tests.rs`), and a test that drops the
  uniques table puts the base items in their place (`character.rs`'s `skill_free_content`). The
  wiki's table names who carries each (`xtask/src/wiki/uniques.rs`, from the gear lines).
- **Powers**, a variant each, read in one place each: `Power::Inflict(OnHit)` rides
  `Mods::local` of the weapon into `Profile::on_hit` (`sharpen`), which `World::resolve` hands
  to `after_blow` after the natural attack's `on_hit` (`Mods::worn` never carries it: it is the
  weapon's that hit; a kick and a bash build no local mods, so none). `Power::Vision(index)`:
  `Actor::eyes` betters a base profile part by part (`worn_vision`); `WorldState::eyes` uses it
  for the player unless `WorldState::vision_dev` (set by the dev `vision:` action) and the AI's
  and ambush's sight checks for creatures. `Power::Ward { status, message }`: `World::afflict`
  returns before anything else when `Actor::ward` finds one, saying `message` to the player.
  `magic::worn_powers` lists the powers of what an actor has equipped.
- **Description**: `describe::lines` adds `LineKind::Power` (the `phrase`) and
  `LineKind::Lore` after the affixes, only through `known_magic`; the UI labels the first
  "Power" and dims the lore in a section of its own.
- **Adding a power**: a `Power` variant, its parse in `unique::power`, its one read, and its
  words in `xtask/src/wiki/uniques.rs` (`power_text`); the item's own words are its `phrase`.
- **Tests**: `world/unique_tests.rs` (the table, hiding until identified, one placement,
  Widowmaker's poison on every hit, High Toby's fear by its chance on the gameplay stream, the
  Bedes' darkvision on and off and under the dev override, the Turncoat's ward, a save); the
  item description's text snapshots `unique-description` and `unique-description-unidentified`
  (the quarry's Stonebiter, the test world's unique). Tests that strip statuses or skills
  remove `campaigns/ascmearc/data/uniques` with them.

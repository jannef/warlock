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
stats, gems with their size and cut, and affixes in words for the item description screen).
`loot::roll` first lets a find of grade `any` be a loose gem (`gem::find`), then sets an item
of a class with an `[[ornament]]` with a gem (`gem::ornament`), and last draws its material
(`variant::draw`, step 8: by the materials' `rarity` at the item level; `loot::roll_from` takes
a `variant::Source`, and creature drops pass `Source::Gear(creature level)`); its own parts are
the material first, then the gem. Creature drops try `gem::find` before the magic item
(`docs/dev/objects.md`, "Gems" and "Materials and variants").
`world/loot.rs` has `World::identify` (the API the scroll of identify calls from its read action,
`docs/dev/scrolls.md`; it identifies a labelled scroll's type too, `World::learn_scroll`),
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
caster level effect (`caster_level`, no built-in affix yet) through `Actor::caster_level`
(`docs/dev/spells.md`); the Protection affix (Hardened, an
item effect, local, on armour kinds: `ItemKind::is_armour`) through `Stack::protection`, which
`Actor::protection` sums (docs/dev/combat.md); the typed effects (docs/dev/combat.md "Damage
types"): `Effect::Elemental` (an attack effect, local on what attacks: `Mods::elemental` is one
rider per kind, `sharpen` adds them to the profile) and `Effect::Resistance` (global:
`Mods::resistances`, which `Actor::resistances` adds to what is worn and what the creature is),
each with the affix's `damage_type` (`AffixDef::damage_type`, part of its `Group`, so an item
may hold one of each kind). Hard
rule: presentation reads an item's affixes only through `Stack::known_magic` (none until
identified; `Stack::magic` is crate-private), and names only through `Stack::describe` /
`title`, so unidentified affixes cannot leak; dev dumps use `describe_truth`. A new effect is a
variant of `Effect` (id, value range, local/global, `allowed_on`), its reader in one rule, a
phrase in `describe.rs`, and a wiki word in `xtask/src/wiki/magic.rs`. Affixes are saved by id, so
adding affixes keeps saves; their values and tiers are checked on load.

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

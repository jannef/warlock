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
(`docs/dev/mana.md`; mana follows the maximum down with `World::keep_mana_within_max`); the Protection affix (Hardened, an
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

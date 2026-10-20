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
`name.rs`; `describe.rs`: base stats and affixes in words for the item description screen).
`world/loot.rs` has `World::identify` (the API the scroll of identify calls from its read action,
`docs/dev/scrolls.md`),
creature drops (`drop/<actor id>` stream), and the hit-point clamp after equipment changes
(to `Actor::lasting_max_hp`: a penalty's lower maximum passes). Stats raised by magic are
`Actor::effective_stats` (the stored scores, raised by `Mods::worn`, then lowered by status
effects, Weak, and stat damage; `raised_stats` is before those, for the character screen's marks
and level-up hit points), the one source every rule reads; an attack profile is built as those stats, the
weapon skill's benefit, the fighting style's (with the two-weapon penalty it eases), the
affixes, then the combat stance. The regeneration affix goes
through `World::healing_rate` like every other pace change. Hard
rule: presentation reads an item's affixes only through `Stack::known_magic` (none until
identified; `Stack::magic` is crate-private), and names only through `Stack::describe` /
`title`, so unidentified affixes cannot leak; dev dumps use `describe_truth`. A new effect is a
variant of `Effect` (id, value range, local/global, `allowed_on`), its reader in one rule, a
phrase in `describe.rs`, and a wiki word in `xtask/src/wiki/magic.rs`. Affixes are saved by id, so
adding affixes keeps saves; their values and tiers are checked on load.

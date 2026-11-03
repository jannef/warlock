# Equipment

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/07b-data.md and spec/06-input-and-ui.md. Keep it current with the code, as CLAUDE.md says.

Equipment (spec §6 "Equipment", §7; M16): slots are data, `data/slots.toml`
(`content/slot.rs`: `SlotDef` with `id`, `name`, `fits`, `action`, `mark`, `hand`; `SlotId` is an
index in screen order, in the content fingerprint), so **a new slot is one entry there** (saves
change). Objects say where they go (`slot` = a slot's `fits`,
`two_handed`, `light_weapon` (any hand; a shield, anything with `bash`, too: two-shield
fighting, `docs/dev/weapon-skills.md`), `ammo`, `launches`; `Content::fits`, `Content::fires`, `Content::hand_slots`).
`equipment.rs` holds the runtime side: `Equipment` (slot → inventory position, on
`Actor::equipment`, saved and hashed; equipped stacks stay in the inventory and keep weighing),
`slots_taken` (every hand for a two-handed object), `free_slot` (spawn data), `mark` ("being
worn"), `verbs` (the message verbs per action). `world/equip.rs` is the rules: `Action::Equip {
slot, item }` / `Action::Remove(slot)` take off what is in the way (a swap) and cost the slot
action (`wield`, `wear`, `body_armor`, `ready` in `data/actions.toml`) of everything put on or
taken off (`World::price_for`); dropping takes off first (`take_off_before_dropping`) and
`Equipment::after_removal` follows the positions (past the 26th too: the pack has no count limit).
**Spirit weapons** (the Hunter's Spirit weapon, #881, `world/spirit_weapon.rs`): a `Stack::spirit`
exists only in a hand slot. `World::equip_saying` (what `World::equip` calls, with the call's own
line, "A long sword of pale light is in your hand.") and `World::remove` end with
`World::fade_spirit`: any spirit stack in the pack and out of the hands (taken off, swapped for
steel, moved to the missile weapon slot) is removed, its positions followed. A drop and a shop's
trade first pass their positions through `World::release_spirit` (the spirit stacks taken off and
faded, the rest renumbered), so neither the floor nor a shop ever holds one; `World::throw`
refuses one; `shop::count_of` and the quests' counts skip it. What takes a stack out of the pack
(an outcome's `remove_stacks`, the forge's ruin) needs nothing: the stack is gone either way.
A companion's collar (`world/collar.rs`, `docs/dev/companion.md`) is the one thing handed over to
a creature's `equipment` in play: the `neck` slot, by the Give ability.
Creatures wield and wear from their spawn
(`wield`/`wear` in a level legend, resolved in `content/level.rs::equip`). Presentation reads
`PerceptionView::equipment` (the player's own) and `PerceptionView::gear(actor)` (what a seen
creature wears and wields, all of what it carries that is seen; look mode, `ui/look.rs::gear`).
The equipment screen (`ui/equipment_screen.rs`) names each slot's stack with its numbers
(`Lore::listed`, `magic::numbers`, docs/dev/magic-items.md; #552) and shows the player's totals
under the slots from `PerceptionView::own_defence` / `own_protection` (the character screen's;
the choice of what goes in a slot is a row taller to cover it).
Digging reads it too: `Actor::dig_tool` (the first equipped thing with `dig` power: the Tool slot's
pick-axe). Combat reads equipment (`docs/dev/combat.md`): `Actor::weapon` (the first hand holding a weapon),
`launcher` and `ammunition` (the `ready` slots), and `defence(content, burden)` (armour, shield,
parry, Dexterity capped by worn `max_dex` and the burden's `max_dex`, the lower winning; the rules
ask `World::defence_of(id)`, which passes the actor's `Load::penalty`). Spells read it too:
`Actor::armour_spell_cost` (the summed `spell_cost_percent` of everything worn or held; the
cost's armour factor, `docs/dev/spells.md`).

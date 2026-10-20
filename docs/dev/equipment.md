# Equipment

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/07b-data.md and spec/06-input-and-ui.md. Keep it current with the code, as CLAUDE.md says.

Equipment (spec §6 "Equipment", §7; M16): slots are data, `data/slots.toml`
(`content/slot.rs`: `SlotDef` with `id`, `name`, `fits`, `action`, `mark`, `hand`; `SlotId` is an
index in screen order, in the content fingerprint), so **a new slot is one entry there** (saves
change). Objects say where they go (`slot` = a slot's `fits`,
`two_handed`, `light_weapon` (any hand), `ammo`, `launches`; `Content::fits`, `Content::fires`, `Content::hand_slots`).
`equipment.rs` holds the runtime side: `Equipment` (slot → inventory position, on
`Actor::equipment`, saved and hashed; equipped stacks stay in the inventory and keep weighing),
`slots_taken` (every hand for a two-handed object), `free_slot` (spawn data), `mark` ("being
worn"), `verbs` (the message verbs per action). `world/equip.rs` is the rules: `Action::Equip {
slot, item }` / `Action::Remove(slot)` take off what is in the way (a swap) and cost the slot
action (`wield`, `wear`, `body_armor`, `ready` in `data/actions.toml`) of everything put on or
taken off (`World::price_for`); dropping takes off first (`take_off_before_dropping`) and
`Equipment::after_removal` follows the positions (past the 26th too: the pack has no count limit). Creatures wield and wear from their spawn
(`wield`/`wear` in a level legend, resolved in `content/level.rs::equip`). Presentation reads
`PerceptionView::equipment` (the player's own) and `PerceptionView::gear(actor)` (what a seen
creature wears and wields, all of what it carries that is seen; look mode, `ui/look.rs::gear`).
Digging reads it too: `Actor::dig_tool` (the first equipped thing with `dig` power: the Tool slot's
pick-axe). Combat reads equipment (`docs/dev/combat.md`): `Actor::weapon` (the first hand holding a weapon),
`launcher` and `ammunition` (the `ready` slots), and `defence(content, burden)` (armour, shield,
parry, Dexterity capped by worn `max_dex` and the burden's `max_dex`, the lower winning; the rules
ask `World::defence_of(id)`, which passes the actor's `Load::penalty`).

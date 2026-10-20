# Combat

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08c-combat.md. Keep it current with the code, as CLAUDE.md says.

Combat (spec §8 "Combat"; M16, #116): the pure rules are `warlock-core/src/combat.rs` (2d12
`roll_natural`, `hits`, `hit_ways`, `ways`; `Category` (Dexterity, Armour, Shield, Parry,
Natural, Dodge, Deflection: one entry each in its `categories!` list) and `Defence` (highest per
category, Dodge stacks; `why_missed` weighs a miss by the positive bonuses); `Attack` (what data
gives: dice, to-hit, threat, crit, a bow's Strength) and `Profile` (an attack with the attacker's
modifiers: `melee`, `ranged`, `thrown`); `strike` resolves one with a fixed order of draws from
the gameplay stream; `max_hp`, `regen_seconds`, `Health` words). `world/fight.rs` applies them:
`World::attack` (picks the profile: melee weapon / natural attack / bare hands, the readied
launcher for a shot, none for a spell) → `attack_with(attacker, target, kind, Option<Profile>)`
(**the entry point for ranged combat, #117**) → `Event::Attacked { .., strike }`, noise, damage
(`hurt`), `Event::Died`, then for a target left alive `World::after_blow` (status effects:
wounds and `on_hit`, `world/status.rs`). **`hurt(id, damage, Harm)` is the one way hit points go
down**: blows (`Harm::Actor`), traps (`Harm::Trap`), status ticks and diseases (`Harm::Status {
of, by, when }`), starving (`Harm::Starving(when)`, `die_of`), and what never kills
(`Harm::Spared`: the plunge through the ice, a disease that does not kill; it leaves the last hit
point); the `Harm` says what a death by it is put down to. A new source of damage is a `Harm`
variant, never a direct `hp -=`. `dies` is the one way an actor dies (a blow or a status,
`Event::Died { of }`): `creature_dies` removes the actor and piles its inventory on its cell,
the player's death sets `WorldState::death` (`Death`: turn, time, place, `by` the creature type
if seen, `of` the status that killed) and `run_turn` stops; `World::apply` refuses everything
after it. Healing and status ticks run as the scheduler moves the clock (`World::pass_time`,
`docs/dev/status-effects.md`). Messages: the player's own attacks are said at once
(`player_attacks`, by what they see as they strike); a creature's in `World::perceive_events`
(`fight::attack_message`, `death_message`; actors that died this action are in the unsaved
`World::fallen`, so their names and words still resolve). Data: objects `damage`, `threat`,
`crit`, `to_hit`, `strength`, `parry`, `armour`, `max_dex`, `shield` (`content/object.rs`);
creatures `base_hp`, `attack` (`NaturalAttack`), `natural` (`Hide`); races and classes `hp`; a
scenario without a character `hp`. Hit points live on `Actor` (`hp`, `hp_base`, `regen`;
`max_hp(content)`, `health(content)`: the effective Toughness), saved and hashed. Presentation reads only
`PerceptionView::own_hp`, `own_defence` (the total: its Dexterity part would give a modifier
away), `SeenActor::health`, and `death`: **never a creature's hit points, a to-hit, a roll, or a
modifier**. Spells stay intents (`strike: None`). Tests: `combat.rs` (exhaustive 2d12
enumeration for hit chances, criticals, miss attribution), `world/fight_tests.rs`.

Protection (spec §8 "Combat", #192; ADOM's PV): `combat::strike(profile, defence, protection,
rng)` takes the target's Protection and soaks a hit with `combat::soak` (one draw from
`soaks(pv)` = ⌈Pv/2⌉..=Pv, **only when Pv less the profile's `penetration` is above 0**, so
fights against unprotected targets keep their draws and hashes), after the damage dice (the
critical multiplied first) and the at-least-1; a hit soaked to 0 is `Strike::Soaked` (damage
0; `is_hit()` stays true). The rules ask `World::protection_of(id)` (`Actor::protection`: every
equipped stack's `Stack::protection`, its object's `protection` plus the `Effect::Protection`
affix, the one reader of that effect, plus the creature type's `Hide::protection`). `resolve`
and `trap_strikes` pass it; `trap_hurts` (blasts, bolts, falls), status ticks, and diseases do
not. Everything a blow does to its target is gated on `blow.damage > 0` in `resolve` (bleed,
`hurt`, `after_blow`) and `trap_deals`, so a soaked hit has no side effects; marks
(`practise`, by `is_hit`), the provoked shopkeeper, noise, the second blow, and a missile's
breaking still happen. Messages go through `fight::Guard` (`Guard::of(target, content)`: the
hide's name for a Natural miss, `Actor::soaked_by` for a soak: the worn piece with the most
Protection, else the hide) into `attack_message`: "Your sword glances off the cave beetle's
shell.", "Something hits you but does no harm." when an end is unseen. Penetration is
`ObjectDef::penetration` (weapons and ammunition), set on the profile by `Actor::melee`,
`off_hand_blow`, `ranged` (launcher plus ammunition), and `thrown`; natural attacks, bashes,
kicks, and traps have none. Data: objects `protection` (armour worn, never a hand slot or a
shield; `check_protection`) and `penetration`; creatures `natural = { defence, protection,
name }`. Presentation: `PerceptionView::own_protection` (the character screen's `Pv` row, the
JSON dump's `protection`), the item description ("Protection 3", "Pierces armour": never a
penetration number); the dev `actors` layer shows every actor's `pv`. **Damage types (#151)**
split it later: Protection soaks physical damage only, per-kind adjustments on a piece,
elements ignore it, resistances after the soak; the hook is `combat::soak` and its comment.
Tests: `combat.rs` (soak distribution, draw order, criticals before the soak, penetration),
`world/protection_tests.rs` (sums, soaked hits, messages, penetration, replay).

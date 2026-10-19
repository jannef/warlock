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
wounds and `on_hit`, `world/status.rs`); `dies` is the one way an actor dies (a blow or a status,
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

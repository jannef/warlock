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
creatures `base_hp`, `attack` (`NaturalAttack`), `natural` (`Hide`; a race's `natural` is the
player's instead, `Actor::hide(content, race)`, and a race's `resist` adds to the creature
type's, `docs/dev/character-creation.md`); races and classes `hp`; a
scenario without a character `hp`. Hit points live on `Actor` (`hp`, `hp_base`, `regen`;
`max_hp(content)`, `health(content)`: the effective Toughness), saved and hashed. Presentation reads only
`PerceptionView::own_hp`, `own_defence` (the total: its Dexterity part would give a modifier
away), `SeenActor::health`, and `death`: **never a creature's hit points, a to-hit, a roll, or a
modifier**. Spells stay intents (`strike: None`). Tests: `combat.rs` (exhaustive 2d12
enumeration for hit chances, criticals, miss attribution), `world/fight_tests.rs`.

Protection (spec §8 "Combat", #192; ADOM's PV): `combat::strike(profile, &Target, rng)` takes
the target's Defence, Protection, and resistances (`combat::Target`, `World::target_of(id)`)
and soaks a hit with `combat::soak` (one draw from
`soaks(pv)` = ⌈Pv/2⌉..=Pv, **only when the Pv against the blow's kind, less the profile's
`penetration`, is above 0**, so fights against unprotected targets, and every element, keep
their draws and hashes), after the damage dice (the
critical multiplied first) and the at-least-1; a hit soaked to 0 is `Strike::Soaked` (damage
0; `is_hit()` stays true). The rules ask `World::protection_of(id)` (`Actor::protection` →
`damage::Protection`: the `base` is every
equipped stack's `Stack::protection`, its object's `protection` plus the `Effect::Protection`
affix, the one reader of that effect, plus the creature type's `Hide::protection`; `against(kind)`
is the base for a physical kind adjusted by the pieces' `protects`, 0 for an element, the base
for untyped harm). `resolve`
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
name }`. Presentation: `PerceptionView::own_protection` (the character screen's `Pv` beside `Dv`, the
JSON dump's `protection`), the item description ("Protection 3", "Pierces armour": never a
penetration number); the dev `actors` layer shows every actor's `pv`.
Tests: `combat.rs` (soak distribution, draw order, criticals before the soak, penetration),
`world/protection_tests.rs` (sums, soaked hits, messages, penetration, replay).

Damage types (spec §8 "Damage types", #151): the data is `data/damage_types.toml`
(`content/damage_type.rs`: `DamageTypeDef { id, name, physical, verb, you, unarmed }`,
`DamageTypeId` an index in file order, in the content fingerprint; `Content::damage_types`,
`damage_type`, `damage_type_id`, `unarmed_damage`, `is_physical`; the shared resolvers
`Content::resolve_kinds` (`damage_type`, one or two names → `Kinds`) and `resolve_per_kind`
(`resist`/`protects` maps) are what objects, creatures, affixes, traps, scrolls, statuses, and
terrain parse with). The model is `warlock-core/src/damage.rs`: `Damage { amount, kind:
Option<DamageTypeId> }` (`None` untyped), `Kinds` (at most `MAX_KINDS` 2; `best(resistance,
protection)` picks the kind a blow does, no draw), `Rider { kind, dice }` (added damage),
`Resistances` (one percent per type; `sum(count, innate, gear)` caps at `MAX_RESISTANCE` 75 /
`MIN_RESISTANCE` −100 and keeps an innate `IMMUNE` 100), `resist(amount, percent)` (nearest,
half up), `Protection { base, against }`, and `ResistanceWord` (the player's words). **Every
damage path goes through one of two doors**: a blow rolled against Defence through
`combat::strike` (its `Profile` carries `kinds` and `riders`, set by `Actor::melee`,
`off_hand_blow`, `ranged`, `thrown`, `bash`, `kick` from `Attack::kinds` (a weapon's
`damage_type`, a natural attack's, `Attack::unarmed(content.unarmed_damage())`) and
`ObjectDef::added` plus `Mods::elemental` (the `elemental` affix, `sharpen` adds one rider per
kind); the blow's `kind` is in `Blow` and `Event::Attacked { damage }`, and the soak and
resistances happen inside `strike`), and everything else through `World::resisted(id, Damage)`
(`trap_hurts` for bursts, bolts, and pits, `spell_hurts` for a scroll's, `ice.rs::fall_in` for
the plunge, `Freeze::kind`), which pushes `Strike::Resisted` when nothing is left of damage
there was. Untyped on purpose: starving and choking (`die_of`), a disease's hit points, and
every status tick (`live`); the poisoned status is typed through `StatusDef::damage_type`,
which `World::afflict` reads as a dose (immune: refused; else `resist` on the severity).
`World::wounded` asks `Wound::caused_by(blow.kind)` (`wound.from`, the kinds that bleed).
Messages: `fight::Words::of(actor, kind, with, damage, content)` takes the kind the blow did
(`Words::blow`: the weapon's `verb`, else the kind's `verb`/`you`, else "hits";
`second_person` derives "cleave" from "cleaves" for a weapon's own verb); traps and scrolls
default their `verb` to the kind's (`content::spell::default_verb`); `attack_message` says a
`Resisted` hit "does {target} no harm". A new damage type is a `[[type]]` at the end of the
file (saves change); a new source of typed harm calls `World::resisted` (or builds a `Profile`
with `kinds`) and names its `damage_type` in data. Presentation reads
`PerceptionView::own_resistances` (words; the character screen's `resistance_rows`, the `stats`
dump layer, JSON `resistances`), `Protection::base` for the Pv total, and the item description
(`magic::describe`: "Slashing or piercing damage", "Adds 1d4 fire damage", "Protection 3 (+1
against slashing, −1 against piercing)", "Resists cold 20%", affix phrases); never a creature's
resistances (the dev `actors` layer's `resists` only). Wiki: `xtask/src/wiki/damage.rs`
(`damage-types.md`, `damage-weapons.md`, `damage-armour.md`; the traps' and scrolls' damage
cells carry the kind). Tests: `damage.rs`, `content/damage_type.rs`, `combat.rs` (the better
kind, elements unsoaked, riders' draw order, resisted hits), `world/damage_tests.rs` (the
built-in data names a kind everywhere, verbs, per-kind Protection, resistance sums and words,
riders, wounds by kind, poison doses, spells, the plunge, descriptions, data checks, replay).

Free blows (spec §8 "Free blows", #228; design `docs/design/progression.md` §4.1 B3) live in
`world/free_blow.rs`. **`World::second_blow(striker, target, Cue)` (`fight.rs`) is the one place
every blow beyond an attack's first is decided**; a new kind of extra blow is a `Cue` (or a case
of one) there and a `Free` kind, never a second mechanism. Cues: `First(blow)` (the off hand's or
Sword and Board's bash, `off_hand_or_bash`, unchanged), `Attacked { parried }` (the end of a
melee attack: `World::cleave` if the target is gone, else `World::riposte`), `Cleaved` (great
cleave's next), `Leaves` (`World::opportunist`), `Approaches` (`World::gatekeeper`, #267: a
hostile that steps up beside a gatekeeper, from `World::step_up(id, from)` at the end of
`ai.rs::creature_steps`, after the trap; each gatekeeper it was not beside before strikes).
Callers: `World::attack` (creatures) and
`player_attacks` run the first and second blows, then `World::after_attack(attacker, target,
parried)`, which strikes the cleave or riposte and follows cleaves while `Cleaved` gives more;
`player_kicks` and `bash` call `after_attack` after their one blow; `World::step_away(id, to)`
runs before a step in `travel.rs::player_moves` and `ai.rs::creature_steps` (after the step is
decided, before `tread`: the mover may die, and then never steps), using `World::leaves` (the
flee geometry `World::flees` also uses). `World::strike_free` strikes one: `resolve` for a
creature, `player_blow` for the player (said at once); the player's out-of-turn ones (`Free::
Riposte`, `Opportunist`, `Gatekeeper`) run with `World::held` set, so `World::say` holds their messages, which
join `World::news` after what was pending and are said at the end of `perceive`, after that
turn's reports. The switches are `FreeBlows` (`cleave`, `great_cleave`, `riposte`, `opportunist`)
on `Actor::free_blows` (saved and hashed, skipped while none is on, so hashes stand), read only
through `World::free_blows_of(actor)`: **talents and class abilities (the progression design's
items D, H1, I1, I4) switch them on there**, by folding their aggregation in or by setting the
field; nothing does yet. A free blow is an ordinary blow: its kinds, riders, resistances, and
verbs come with its `Profile` and `Words::of` like any other's. Rules: deciding never draws (cleave's target is the next hostile clockwise from where
the target fell); a free blow draws only as the blow it is; free blows give none of their own
but cleave after cleave; noise only for the opportunist's and the gatekeeper's. Tests: `world/free_blow_tests.rs`
(cleave order and count, great cleave, kicks, ripostes and their message order, no answer to a
riposte, opportunists both ways, in the dark, no draws while triggers do not fire, saves).

# Scrolls

Developer notes for working on this part of Warlock (#150); the rules it implements are in
spec/08g-magic-items.md ("Scrolls") and spec/06-input-and-ui.md. Keep it current with the code,
as CLAUDE.md says.

Scrolls (spec §8 "Scrolls") are objects with a `read` table (`content/scroll.rs`: `ReadSpec` →
`Reading::{Spell(ScrollSpell), Identify}`; `ScrollSpell { cast: Cast::{Burst, Bolt}, range,
noun, verb, message, noise }`), one file each in `data/objects/` (`scroll_of_fireball`,
`scroll_of_lightning`, `scroll_of_frost`, `scroll_of_identify`; `scroll_of_light` has no `read`
yet). Their spells are the traps' own data types (`content/spell.rs`: `Burst`, `Bolt`,
`BurstIce`, `SpellFields`, with `Burst::new`/`Bolt::new` the one place their fields are
checked, and `check_effect`, the effect-shape check, run for scrolls once `data/effects.toml`
is loaded, `scroll::check_spells`); `TrapEffect::Burst(Burst)`/`Bolt(Bolt)` hold the same
structs. A scroll's `level` and `frequency` make it loot (`ObjectDef::frequency` > 0 for base
items and scrolls; `magic::loot::roll` draws among both, leaves objects without a `kind` out of
loot that must be magic, and draws no rarity for them).

The rules are `world/read.rs`: `Action::Read { item, target: Option<ReadTarget> }` (`read:b`,
`read:b:12,5` a cell, `read:b:c` a stack; `ReadTarget::{Cell, Item}`), one arm in
`player_acts`, priced `ActionKind::Read` (`read` in `data/actions.toml`) before the scroll is
used up (`use_up`, which follows the equipment's letters). Refusals take no time and keep the
scroll. A spell checks `can_loose_here` and `within_reach` (`shoot.rs`, shared with missiles),
then casts through **the one shared spell code** in `world/spell.rs`: `World::cast_burst(caster,
(origin, center), burst)` and `World::cast_bolt(caster, origin, path, bolt)` (the effect's
`Event::Spell`, one damage roll for everyone reached in actor-id order, then a burst's
`World::blast`, `World::shock`, `World::frost`), with `Caster::Trap(TrapId)` (blows are
`Event::TrapStruck` via `World::trap_hurts`, `Harm::Trap`: no one's kill) or
`Caster::Actor(ActorId)` (blows are `Event::Attacked` of `AttackKind::Spell` with a hit, blood
by `spill`, `hurt` with `Harm::Actor`, so `dies` → `earn_kill` and `tally_kill`, then
`wounded`; a shopkeeper it reaches is `provoke`d). `World::sound_at` makes a trap's or a spell's
noise (no maker). A burst read flies along `Pos::ray` to the cell aimed at and goes off on the
first actor, before the first opaque cell, or there (`burst_center`); a bolt runs `length` cells
along the ray after the reader (`bolt_line`). `say_spell` says the reader's blows at once, as
`player_looses` does for shots (`End::Trap(noun)`: "The fireball burns the kobold."), keeps the
seen ones' events, and leaves the reader's own death to `perceive_events`; a reader killed by
their own scroll gets `Death::cause = Cause::Scroll(object)` ("Killed by their own scroll of
fireball"). The scroll of identify calls `World::identify` on the stack named before using the
scroll up (positions shift after).

Presentation: `r` is `KeyCommand::Read` (id `read`, next to Eat), opening `ScreenName::Read`
(`ObjectList::Read`, the paged list of readable stacks). A spell scroll answers
`ScreenAction::Aim(AimKind::Read(item))`: the aiming mode (`ui/target.rs`) with the scroll's
`range`, its line (`Aim::line`: a bolt's line is blocked only by what is solid) and the read
key confirming; `Aim::catches_reader` predicts, from perception only, whether a burst may catch
the player, and `input::burst_dialog` then asks in the one dialog (Read it / Aim again / Don't
read it). The scroll of identify answers `ScreenAction::Replace(ScreenName::Identify)` after
noting the scroll (`ObjectScreen::looked`; `Screens::replace` reads it before closing), and the
identify list (`ObjectList::Identify(scroll)`, the unidentified stacks) acts
`Action::Read { item: scroll, target: Item(i) }`. Spell visuals are the traps' (`ui/spells.rs`
draws `Event::Spell`; a burst's origin is the reader, so it flies there first).

Content and places: the trap hall (`levels/trap_hall.toml`) has a pile of scrolls and an
unidentified dagger beside the arrival; general stores stock and buy the `scrolls` class. The
wiki's table is `xtask/src/wiki/scrolls.rs` (`scrolls.md`), sharing `ice_text`/`sound_text`
with the traps' table. Tests: `world/read_tests.rs` (the shared casts match the traps', bursts
and bolts, kills and experience, refusals, identify, death, saves and replays, loot), the read
and identify lists in `ui/object_screen.rs`, aiming in `ui/target.rs`.

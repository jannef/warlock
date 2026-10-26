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
`Event::Spell`, one damage roll for everyone reached in actor-id order, typed by the burst's or
bolt's `kind` (`damage_type` in data) and resisted by each target through `World::resisted`
(docs/dev/combat.md "Damage types"), then a burst's
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

**Unidentified scrolls** (#393, `scroll_label.rs`): every readable object type (`read`) has a
label for the run, "a scroll labelled ZUNBAR". `content/scroll_label.rs` reads
`data/scroll_labels.toml` (`ScrollLabelDef`: `name`/`plural` with `{label}`, the one `fg` of
every unknown scroll, `syllables` least and most, `parts`); without the file nothing is
labelled. `Labels::draw(content, seed)` draws them all at once from the stream
`identification`, types sorted by object id string, each label different (after 100 equal
draws a number tells it apart): derived, held in `World::labels` (made in `World::assemble`,
so new and loaded worlds alike), never saved. What the player knows is
`WorldState::known_scrolls` (`BTreeSet<ObjectId>`, saved and hashed, left out while empty, so
runs that never identify a scroll hash as before; saves change). `World::learn_scroll(object)`
adds a labelled type (true if it was news); it is called by `say_read` (every read, spell or
identify: "You read the scroll labelled ZUNBAR." then "It is a scroll of fireball."), by
`World::identify` (a labelled scroll stack: "This is 2 scrolls of lightning!"; so the dev
`identify` and F8 too), and `World::new` puts the kit's readable scrolls in the set (a chosen
character knows their kit; a scenario's own pack is not a kit).

`Lore` (`World::lore()`, `PerceptionView::lore()`) is how everything the player is shown names
a stack: `describe`, `title`, `names`, `fg`, `describe_object(object, count)`, `knows(stack)`
(type and magic both), `is_identified(object)` (the type: what the study list, #396, asks;
identified is not learned, so an identified spell scroll may be studied by one who has not
learned its spell, a labelled one by nobody), `label(object)`. Use it, never
`Stack::describe`, wherever a name reaches the player: messages (`object::phrase` and
`phrase_with` take a `Lore`), screens (`object_screen`, `describe_screen` through
`describe::appraised`, the map's pile colour, look), and player-facing dumps (`inventory`,
`equipment`, the JSON's `name` and `identified`). `Lore::plain(content)` names everything as it
is (tests, starting-kit summaries). Dev dumps (`actors`) keep `describe_truth`. Prices are true
(a shop's price can hint at a label, as in ADOM). The read list still dispatches on the true
`read` (a spell is aimed, identify opens the identify list), so the prompt that follows tells a
spell from identify; the design's "no cancelling once the target is known" belongs to the spells
work (#395/#396). The identify list (`ObjectList::Identify`) lists the stacks `Lore::knows`
is false for. Tests: `world/scroll_label_tests.rs`.

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

The casters' strikes (#245, `world/caster.rs`) are a scroll's spell without the scroll: a
class ability's `[[talent.spell]]` names a `cast` table that `ReadSpec` reads, and `World::strike`
casts it through `burst_center`, `bolt_line`, `cast_burst`/`cast_bolt`, and `say_spell`, as
`read_spell` does (`Cause::Spell` for a caster killed by their own). Both apply the reader's
`TalentMods::spell_damage` (Scroll adept, Empower, Archmage) to the dice first; `check_spells`
checks the talents' spell effects too.

Content and places: the trap hall (`levels/trap_hall.toml`) has a pile of scrolls (two of each
spell, three of identify) and an unidentified dagger beside the arrival; general stores stock and
buy the `scrolls` class, and a generated one always lays out scrolls of identify (`always` in
`data/shops.toml`; Cēaptūn's has three by hand). The scroll of identify's `frequency` is 300,
the spell scrolls' 50 to 60. The
wiki's table is `xtask/src/wiki/scrolls.rs` (`scrolls.md`), sharing `ice_text`/`sound_text`
with the traps' table. Tests: `world/read_tests.rs` (the shared casts match the traps', bursts
and bolts, kills and experience, refusals, identify, death, saves and replays, loot), the read
and identify lists in `ui/object_screen.rs`, aiming in `ui/target.rs`.

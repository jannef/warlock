# Scrolls

Developer notes for working on this part of Warlock (#150); the rules it implements are in
spec/08g-magic-items.md ("Scrolls") and spec/06-input-and-ui.md. Keep it current with the code,
as CLAUDE.md says.

Scrolls (spec §8 "Scrolls") are objects with a `read` table (`content/scroll.rs`: `ReadSpec` →
`Reading::{Spell(ScrollSpell), Cast(SpellScroll), Identify}`; `ScrollSpell { cast:
Cast::{Burst, Bolt}, range, noun, verb, message, noise, teaches }`), one file each in
`data/objects/`: a scroll of every spell of `data/spells/` (`scroll_of_ID`, kind `spell`,
`Reading::Cast`, "Scrolls of spells" below), the three older ones whose readings are the traps'
bursts and bolts (`scroll_of_fireball`, `scroll_of_lightning`, `scroll_of_frost`,
`Reading::Spell`), and `scroll_of_identify`; `scroll_of_light` has no `read` yet. The older
three's spells are the traps' own data types (`content/spell.rs`: `Burst`, `Bolt`,
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
character knows their kit; a scenario's own pack is not a kit). Wand types (#398) are labelled
the same way from `data/wand_labels.toml` ("an oak wand"), drawn after the scrolls from the
same stream, and learned by `World::use_charge` (docs/dev/spells.md "Wands").

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

**Scrolls of spells** (#397): a `read` table of kind `spell` names a spell of `data/spells/`
(`Reading::Cast(SpellScroll { spell, level, reach })`, resolved as objects load, spells loading
first; only `spell` may be given). Reading one is `World::read_cast` (`world/cast.rs`): the one
cast path with `Pay::Scroll(item)` instead of mana, at `level` (the spell's `[spell.scroll]
level`, else its tier's comfortable level), never asking whether the reader knows the spell or
has the mana; the scroll is used up (`use_up`, Spell recall's chance) and said read
(`say_read`, which identifies its type) only once the cast is sure to go off, so a refusal
keeps it and takes no time. An aimed spell needs its cell (`read:b:12,5`); a spell on oneself
is read with none (`read:b`; the read list acts at once, `ObjectScreen`'s `identify` flag
telling it from the scroll of identify). A death by it is `Cause::Scroll`. The aiming mode
aims it as the spell (`ui/target.rs::spell_of`, the player's own reach).

**Study** (#396, docs/dev/spells.md "Learning"): a scroll teaches what `Reading::teaches` says:
a `spell` reading its own spell; a burst's or bolt's the spell its `teaches` names
(`ScrollSpell::teaches`: fireball Fireball, lightning Lightning bolt, frost Frost); the scroll of
identify nothing. `Action::Study` uses the scroll
up its own way (`World::use_scroll` in `world/learn.rs`: no Spell recall, no Read training), and
a mishap's backfire casts the scroll's own `Cast` on its reader with `say_spell` (a `spell`
reading's damage spell as a burst around the reader, its own radius or 1, at the scroll's
level, held to its cap; any other kind fizzles).

Spells (the one cast path, `world/cast.rs`, docs/dev/spells.md) share this code: a burst spell
goes through `burst_center` and `cast_burst`, a line through `bolt_line` and `cast_bolt`, and
both are said with `say_spell`, as `read_spell` does (`Cause::Spell` for a caster killed by their
own). Both apply the caster's `TalentMods::spell_damage` (Scroll adept, Empower, Archmage) to the
dice first; a spell's roll is then held to its cap (`Burst::cap`, `Bolt::cap`), a scroll's is
not (`None`). `spell_def::check_effects` checks spells' effects the way `check_spells` checks
scrolls'.

Content and places: the trap hall (`levels/trap_hall.toml`) has a pile of scrolls (two each of
fireball, lightning, and frost, three of identify) and an unidentified dagger beside the arrival; general stores stock and
buy the `scrolls` class, and a generated one always lays out scrolls of identify (`always` in
`data/shops.toml`; Cēaptūn's has three by hand). The scroll of identify's `frequency` is 300,
the spell scrolls' 20 to 60 by tier (level, frequency, and value by tier, the older three
too: docs/dev/spells.md "The first list"). The
wiki's table is `xtask/src/wiki/scrolls.rs` (`scrolls.md`), sharing `ice_text`/`sound_text`
with the traps' table. Tests: `world/read_tests.rs` (the shared casts match the traps', bursts
and bolts, kills and experience, refusals, identify, death, saves and replays, loot), the read
and identify lists in `ui/object_screen.rs`, aiming in `ui/target.rs`.

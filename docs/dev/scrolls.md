# Scrolls

Developer notes for working on this part of Warlock (#150); the rules it implements are in
spec/08g-magic-items.md ("Scrolls") and spec/06-input-and-ui.md. Keep it current with the code,
as CLAUDE.md says.

Scrolls (spec §8 "Scrolls") are objects with a `read` table (`content/scroll.rs`: `ReadSpec` →
`Reading::Cast(SpellScroll { spell, rungs, reach, on_item })`), one file each in
`data/objects/`: a scroll of every spell of `data/spells/` (`scroll_of_ID`, kind `spell`,
"Scrolls of spells" below). The three older ones
(`scroll_of_fireball`, `scroll_of_lightning`, `scroll_of_frost`) are spell scrolls too (S2, the
spell scaling design §7.4), at a rung of their spells' ladders (1, 2, and 0: 3d6, 4d6, 2d6);
there are no `burst` or `bolt` readings (a `read` of either kind is refused, naming the `spell`
kind and `rungs`). **Every scroll is a real spell** (#592), but the scroll of restoration (below, #887): each casts its spell
(`read_tests.rs::every_builtin_scroll_can_be_read` holds every `scroll_of_*` to it), and none
teaches it: its grimoire does (#1260, "Grimoires" below); the
`identify` reading kind is gone, and `scroll_of_identify` is a `spell` scroll of Identify
(below). `scroll_of_light` casts the cantrip Light (#589: until then it had no `read` and could
not be read at all): a buff whose status, Lit, has a `light` (`StatusDef::light`), which the
light cache sheds about its bearer while it lasts (`light/cache.rs`, the brightest of several, as
Holy radiance's). `read_tests.rs` `every_builtin_scroll_can_be_read` reads every `scroll_of_*`
in the content, so a scroll left without a reading fails it. The traps' bursts and bolts
(`content/spell.rs`: `Burst`, `Bolt`, `BurstIce`, `SpellFields`, `Burst::new`/`Bolt::new`) are
no scroll's any more; the cast path shares their structs for its bursts and lines. A scroll's
`level` and `frequency` make it loot (`ObjectDef::frequency` > 0 for base items and scrolls;
`magic::loot::roll` draws among both, leaves objects without a `kind` out of loot that must be
magic, and draws no rarity for them).

The rules are `world/read.rs`: `Action::Read { item, target: Option<ReadTarget> }` (`read:b`,
`read:b:12,5` a cell, `read:b:c` a stack; `ReadTarget::{Cell, Item}`), one arm in
`player_acts`, priced `ActionKind::Read` (`read` in `data/actions.toml`; `read_price` scales it
by `MagicMods::read_percent`, Scroll savant's and Scroll adept's halves, never below
`[read] least_percent` of `data/tuning.toml`, a quarter) before the scroll is
used up (`use_up`, which follows the equipment's letters; it reports the Read stat use and
`Use::Read`, Decipher script's 50 % mark, #1270: all reading trains it). Refusals take no time
and keep the scroll. A read is a cast (`World::read_cast`, below): a damage spell checks `can_loose_here`
and `within_reach` (`shoot.rs`, shared with missiles), then casts its shape through **the one
shared spell code** in `world/spell.rs`: `World::cast_burst(caster,
(origin, center), burst)` and `World::cast_bolt(caster, origin, path, bolt)` (the effect's
`Event::Spell`, one damage roll for everyone reached in actor-id order, typed by the burst's or
bolt's `kind` (`damage_type` in data) and resisted by each target through `World::resisted`
(docs/dev/combat.md "Damage types"), then a burst's
`World::blast`, `World::shock`, `World::frost`), with `Caster::Trap(TrapId)` (blows are
`Event::TrapStruck` via `World::trap_hurts`, `Harm::Trap`: no one's kill) or
`Caster::Actor(ActorId)` (blows are `Event::Attacked` of `AttackKind::Spell` with a hit, blood
by `spill`, `hurt` with `Harm::Actor`, so `dies` → `earn_kill` and `tally_kill`, then
`wounded`; a shopkeeper it reaches is `provoke`d). `World::sound_at` makes a trap's or a spell's
noise (no maker). A burst flies along `Pos::ray` to the cell aimed at and goes off on the first
actor, before the first opaque cell, or there (`burst_center`); a line runs its length (the
reader's, the spell's reach ladder climbed) along the ray after the reader (`bolt_line`).
`say_spell` says the reader's blows at once, as
`player_looses` does for shots (`End::Trap(noun)`: "The fireball burns the kobold."), keeps the
seen ones' events, and leaves the reader's own death to `perceive_events`; a reader killed by
their own scroll gets `Death::cause = Cause::Scroll(object)` ("Killed by their own scroll of
fireball"). The scroll of identify casts Identify through the cast path
(`World::cast_identify`: refused without the stack or for one already known; then paid, the
scroll used up, and `World::identify` on the stack named, one place up if the used-up scroll
stack was before it).

**The Decipher ability** (#1025; `world/decipher.rs`): `Ability::Decipher` (Decipher script's use,
skill source, `Aim::Item`; `Unavailable::NoUnknownScroll` from the perception when no stack
passes `undeciphered`) sends `Action::Decipher { item }` (`decipher:b`). `start_decipher` refuses a
stack that is no unidentified scroll (no time) and answers `Spent::Decipher`; `decipher` keeps turns
of `ActionKind::Decipher` for `DecipherRules::seconds` as a forage does (`keep_at`), then `deciphered`
calls `deciphers` with the scroll's spell and level (`Reading::Cast`, else none and 0), and on success
`learn_scroll`s the type without using the scroll up. The UI list is `ObjectList::Decipher`.

**Making out a scroll** (#1024, `world/decipher.rs`, `content/decipher.rs`,
`data/decipher.toml`; the skill uses design §6.6): `World::deciphers(spell, level)` stands between a
reading's refusals and its `use_up`. A spell the player knows (`Actor::knows_spell`, a scroll they
wrote too): true, no draw, and `Use::Decipher` still marks (a trained skill only). Else trained in Decipher script (the skill of learning, #1270) with `Actor::learning` at
least `level`: true, no draw, no mark, no words. Otherwise a `check::check` of the Intelligence modifier +
`VigourMods::check(Intelligence)` + `skill_effect(Effect::Decipher)` (0 unless trained in Decipher script) against
`DecipherRules::difficulty` (`base` + level ÷ `levels_per_point`), then `Use::Decipher` (100 %,
whatever came of it; none for Spellcraft), and on a failure
"The writing swims before your eyes." An untrained reader's points are 0, whatever
they wear that raises the skill (a suffix of the Scholar does nothing without the skill). The hook is
**`World::pay_cast`**, where every kind of cast has finished refusing: it returns false for a scroll
not made out, and its eight callers end the cast with `return true`, so `read` still answers
`Spent::Turn(price)` and nothing has been used up, said read, or cast (`Cast::read` stays `None`).
`read_cleanse` asks the same way, as level 0. The check is the player's alone: only they read. A
failure saves nothing new; the labels stay as they were, because `say_read` never ran.

Under this rule **the scroll of identify** still lists and refuses as it did (`cast_identify`'s
refusals come before `pay_cast`, so a stack with nothing to tell draws nothing); read and failed, it
is kept and identifies nothing. It is a spell of level 6, so a character trained in Decipher script below
learning 6 makes the check. **Scribe** (`world/scribe.rs`, below) reads nothing and asks no check:
it needs the spell known, the blank, and the mana, as before, and its scroll is an ordinary scroll,
which reads by this rule for its writer too (the spell is known, so it reads at once, whatever their learning).
The dev `identify` and F8 are not reads. Wands, rods, and spells cast for mana never ask. The file
is optional (`Content::decipher`): without it every scroll simply reads; fixtures whose tests are
about what a read does remove it (`files.remove("data/decipher.toml")`) rather than draw a check.
Tests: `world/decipher_tests.rs`.

**The scroll of restoration** (#887, the priest design §11) is the one scroll with no spell
behind it: `read` of kind `cleanse` (`Reading::Cleanse(Cleansing { mends, restores })`; one of
the two at least, neither with `spell` or `rungs`, nor they on a `spell` reading). `spell()`
is `None`, `scroll()` (the `SpellScroll`) is `None`, `range()` and `on_item()` are a cast's
only, so the read list reads it at once, it has no grimoire, and the aiming mode never sees it. `World::read` sends it
to `World::read_cleanse` (`world/restore.rs`): refused, no time, with a target or with nothing to
mend (no stat damage while `mends`, no drained point while `restores`, and no status
`cured_by` `restore::RESTORATION`, Stiffening: "There is nothing to restore."); otherwise `deciphers(None, 0)` (above), `use_up` (Scroll savant's keep, the Read stat training), `say_read`, then
`World::restore` (shared with the potion of restoration, docs/dev/potions.md):
`StatDamage::mend_all` and `World::restore_drained` (the temple's `World::mend`, split by the two
switches), `World::cure(player, RESTORATION)` (Stiffening ends, its end line said; #1126), and "You
feel your strength return." for each stat that came back. No disease check
(the temple's "Be cured first." is not a rule here), no stream drawn but the check's and Scroll savant's, nothing
new saved. Adding it moved every label drawn after the scrolls (wands', rods', potions': the
`identification` stream draws types in id order), hence the screen snapshots' new potion and
wand words. `read_tests.rs::the_scroll_of_restoration_mends_and_restores`;
`every_builtin_scroll_can_be_read` expects its refusal on a hale reader.

**Blank scrolls and writing** (#906, the witch and wizard design §6.1): `blank_scroll`
(`data/objects/blank_scroll.toml`, `Content::blank_scroll`) is of the scrolls class with no
`read`, so it is never labelled (labels are drawn for readable types only: adding it moved no
label) and known on sight; `content/object.rs` lets a scrolls-class object without a `read` be
loot (`level`, `frequency`: a `frequency` given makes it loot, so the sealed letter, which has
none, stays out). `World::read` refuses it before anything else ("There is nothing written on
it."). **Writing scrolls** is `world/scribe.rs`, the
wizard's Scribe (`scribe` talent at 12, `MagicEffect::Scribe { mana_times }`,
`MagicMods::scribe`): `Action::Scribe { spell }` (`scribe:fireball`, its own action, as the
active asks for a spell: `ClassActive::aims_spell`, `Aim::Spell`), `World::start_scribe`
refusing with no time (`may_use_class`; an unknown id, a spell not known, `Content::scroll_of`
none, no blank, `scribe_cost` above the mana), then `Spent::Scribe(Writing)` → `World::scribe`
through `keep_at` (`scribe_turns`: `study_turns` at 0 %, Quick study left out), interrupted as
a study ("You stop writing.", nothing spent), and `finish_scribe`: `spend_mana(scribe_base)`
(three times `spell_base`, Frugal caster's cut on the whole), one blank taken (the equipment's
letters followed), `Stack::one(scroll_of)` added, `learn_scroll`, `StatUse::Scribe`. Nothing
new is saved (the writing runs inside one action); no draws but the spend's training. The
scribe list is `ui/scribe_screen.rs` (`ScreenName::Scribe`, `--screen scribe`), from
`PerceptionView::scribes` (each spell's price and nominal time, `OwnScribe`) and
`blank_scrolls`. Tests: `world/scribe_tests.rs`; the list's snapshots `scribe-list` and
`scribe-list-none`.

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
`identify` and F8 too), and `World::begin` puts every labelled type of the starting pack in the set
(`kit::known_types`, #1173: a chosen character of any class knows their kit, the casters' random
spell scrolls too, #1104, a scenario's own pack is known the same way, and a background story's
things, `world/background.rs::story_item`; `kit.rs::every_starting_scroll_and_potion_is_known`,
`a_scenarios_own_pack_is_known_by_type_too`). Wand types (#398) are labelled
the same way from `data/wand_labels.toml` ("an oak wand"), drawn after the scrolls from the
same stream, and learned by `World::use_charge` (docs/dev/spells.md "Wands"). Potion types
(#565) come last, from `data/potion_labels.toml`: a word and the type's own colour ("a fizzy red
potion", docs/dev/potions.md), learned by drinking.

`Lore` (`World::lore()`, `PerceptionView::lore()`) is how everything the player is shown names
a stack: `describe`, `title`, `names`, `fg`, `describe_object(object, count)`, `knows(stack)`
(type and magic both), `is_identified(object)` (the type; identified is not learned: no scroll
teaches), `label(object)`. Use it, never
`Stack::describe`, wherever a name reaches the player: messages (`object::phrase` and
`phrase_with` take a `Lore`), screens (`object_screen`, `describe_screen` through
`describe::appraised`, the map's pile colour, look), and player-facing dumps (`inventory`,
`equipment`, the JSON's `name` and `identified`). `Lore::plain(content)` names everything as it
is (tests, starting-kit summaries). Dev dumps (`actors`) keep `describe_truth`. Prices are true
(a shop's price can hint at a label, as in ADOM). The read list still dispatches on the true
`read` (a spell is aimed, identify's opens the identify list: `Reading::on_item`), so the prompt that follows tells a
spell from identify; the design's "no cancelling once the target is known" belongs to the spells
work (#395/#396). The identify list (`ObjectList::Identify`) lists the stacks `Lore::knows`
is false for. Tests: `world/scroll_label_tests.rs`.

Presentation: `r` is `KeyCommand::Read` (id `read`, next to Eat), opening `ScreenName::Read`
(`ObjectList::Read`, the paged list of readable stacks). A spell scroll answers
`ScreenAction::Aim(AimKind::Read(item))`: the aiming mode (`ui/target.rs`) with the scroll's
`range`, its line (`Aim::line`: a bolt's line is blocked only by what is solid) and the read
key confirming; `Aim::catches_reader` predicts, from perception only, whether a burst may catch
the player, and `ui::step_dialogs::burst_dialog` then asks in the one dialog (Read it / Aim again / Don't
read it). The scroll of identify answers `ScreenAction::Replace(ScreenName::Identify)` after
noting the scroll (`ObjectScreen::looked`; `Screens::replace` reads it before closing), and the
identify list (`ObjectList::Identify(IdentifyBy::Scroll(scroll))`, the unidentified stacks) acts
`Action::Read { item: scroll, target: Item(i) }`; cast from the abilities menu, Identify opens the
same list (`IdentifyBy::Spell`, `Screens::open_identify(IdentifyBy::Spell)`), which acts
`Action::CastOn("identify", i)`. When the cast told something new (`World::identify` gave `Identified::Now`) `cast_identify` also sends `Event::Identified { item }` (the stack's position after the scroll was used up), and `screens::show_identified` opens that stack's description screen once the action is done. Appraise, Merchant's eye, decipher, and the dev `identify` send none. Spell visuals are the traps' (`ui/spells.rs`
draws `Event::Spell`; a burst's origin is the reader, so it flies there first).

**Scrolls of spells** (#397): a `read` table of kind `spell` names a spell of `data/spells/`
(`Reading::Cast(SpellScroll { spell, rungs, reach, on_item })`, resolved as objects load, spells
loading first; only `spell` and `rungs` may be given, `rungs` 0 to the spell's roll ladder's
length, default 0). Reading one is `World::read_cast` (`world/cast.rs`): the one cast path with
`Pay::Scroll(item, rungs)` instead of mana, its roll at the scroll's `rungs` plus the reader's
`scroll_rungs` (Scroll adept; `CastBy::Scroll`, no stats), the reader's other hooks (a reroll,
a rider, a die more) applying, and the reader's reach and degrees, never asking whether the
reader knows the spell or
has the mana; the scroll is used up (`use_up`, Spell recall's chance) and said read
(`say_read`, which identifies its type) only once the cast is sure to go off, so a refusal
keeps it and takes no time. An aimed spell needs its cell (`read:b:12,5`), a spell on a stack
(Identify, `SpellScroll::on_item`) its stack (`read:b:c`); a spell on oneself
is read with none (`read:b`; the read list acts at once, `ObjectScreen`'s `identify` flag
telling it from the scroll of identify). A death by it is `Cause::Scroll`. The aiming mode
aims it as the spell (`ui/target.rs::spell_of`, the player's own reach).

**Grimoires** (#1260, docs/dev/spells.md "Learning"): a scroll teaches nothing; the spell is
learned from its grimoire (`grimoire_of_X`, generated from `scroll_of_X` by
`content/grimoire.rs`; `ObjectDef::grimoire`, no `read`). `World::read` sends a grimoire, read
with no target, to `World::start_study` before anything about `read` (a target: "A grimoire is
read, not aimed: `read:a`."); the study keeps the grimoire, and a mishap's backfire casts the
grimoire's damage spell as a burst around the reader (its own radius or 1) at the spell's base,
no rung and no hooks; any other kind fizzles. A grimoire is never labelled (known on sight: the
title is plain words), so `Lore`, `kit::is_labelled`, decipher, and the identify list never see
it. The read list (`ObjectList::Read`) lists it with the scrolls (`ObjectDef::grimoire` as well
as `read`).

Spells (the one cast path, `world/cast.rs`, docs/dev/spells.md) share this code: a burst spell
goes through `burst_center` and `cast_burst`, a line through `bolt_line` and `cast_bolt`, and
both are said with `say_spell` (`Cause::Spell` for a caster killed by their own, `Cause::Scroll`
for a reader). Their roll is the cast's (`Burst::damage`, a `Roll`, with `Burst::reroll`, a
trap's 0), nothing holding it (decision 16). `spell_def::check_effects` checks spells' effects.

Content and places: the trap hall (`levels/trap_hall.toml`) has a pile of scrolls (two each of
fireball, lightning, and frost, three of identify) and an unidentified dagger beside the arrival; general stores stock and
buy the `scrolls` class, and a generated one always lays out scrolls of identify (`always` in
`data/shops.toml`; Cēaptūn's has three by hand). The scroll of identify's `frequency` is 300,
the spell scrolls' 20 to 60 by spell level (level, frequency, and value each the object's own,
by the objects' README convention, the older three too: docs/dev/spells.md "The first list"). The
wiki's table is `xtask/src/wiki/scrolls.rs` (`scrolls.md`), sharing `ice_text`/`sound_text`
with the traps' table; its "Casts at" column is the roll at the scroll's rung
(`SpellData::roll_at`). Tests: `world/read_tests.rs` (the three older scrolls at their rungs,
bursts and bolts, kills and experience, refusals, identify, death, saves and replays, loot,
the `burst` and `bolt` readings refused), `world/spell_talent_tests.rs` (Scroll adept's rung), the read
and identify lists in `ui/object_screen.rs`, aiming in `ui/target.rs`.

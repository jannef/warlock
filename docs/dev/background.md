# Background story

Developer notes for a chosen character's background story (spec §8 "Background story"; the
design is `docs/design/background-story.md`, tracked in #708). B1 (#702) is the data, the roll,
the effects, and the saved ids; the story screen and page 8 are B2 (#703), the full pools and
the wiki's tables B3 (#704), the `background` system's talents B4 (#705), and talent and flaw
effects B5 (#707). Keep it current with the code, as CLAUDE.md says.

## Modules

| Module | What |
|---|---|
| `content/background.rs` | The data: `Background { tiers, points, chapters }`, `Chapter`, `Line` (`StoryLine` outside), `When`, `Effect` (`StoryEffect`), `Marks`, `Tier` (`StoryTier`), `Tiers::of(roll, percent)`, `slots_of`; `Content::load_background` (after the kits: races, classes, skills, weapon skills, talents, objects, slots, the horoscope, and the campaigns are known) and every check. Tests: `content/background_tests.rs`. |
| `background.rs` | The roll, pure: `Witness` (what conditions read) and `Witness::fits`, `tell(content, seed, &witness) -> Story { told, effects }`, `Rolled` (an effect with its dice rolled), `Told { chapter, line }` (saved), `retell` and `lines` (the text again from the content, slots filled), `missing` (log lines for ids the content lost). Tests: `background_tests.rs`. |
| `world/background.rs` | `World::tell_background` (from `start_run`), `apply_story` (one arm per effect kind), `story_gold`, `story_item`. Tests: `world/background_tests.rs`. |
| `rng.rs` | `Rng::background(seed, spec, chapter)`: `background/<race>,<gender>,<class>/<chapter>`. |
| `dump.rs` | The `story` layer, JSON `story` and `character.story`, the `actors` layer's `story …`. |
| `crates/warlock/src/ui/chargen_screen.rs` | The summary's dim line under the kit, `FATE`: "Fate will have its say when you set out." (only with content that tells stories). |

## The data

`assets/data/background.toml` holds `[tiers]` (per mille: uncommon 120, rare 30, fateful 5;
common the rest) and `[points]` (what each effect kind is worth: only for the tier check at load
and the wiki; nothing counts a story's total). Its comments document every field.
`assets/data/background/NN-<chapter>.toml` is one chapter a file in file-name order (`[chapter]`
`id`, `name`, `paragraph` 1 to 4, slot `defaults`; `[[line]]`s), then a campaign's own in
`assets/campaigns/<id>/data/background/` (read by `ContentFiles::table("background")` after the
shared ones; `Chapter::campaign` from the path). A story told in campaign `c` is the shared
chapters, then `c`'s (`Background::chapters_for`). Two campaigns may each have a `road`.

**Checks** (each refused with the file, chapter, and line): the settings' tiers at most 1000;
a chapter's `paragraph` 1 to 4 and never less than an earlier chapter's; ids unique (a line's in
its chapter, a chapter's in each story's order, so a campaign never redefines a shared one);
`weight` 1 to 20; every id a `when` names exists (races, classes, animals, elements, talents,
skills, campaigns; a `band` well formed); a tag in `when.tags`/`not_tags` is set by a line of an
earlier chapter of that story; slots are `{name}`, `{they}`, `{them}`, `{their}`, `{guardian}`,
`{home}`, `{trade}` (and capitalised), `set` and `defaults` fill only the last three, and a slot
used has a default in this chapter or an earlier one; an `only` line is common; an effect names
one kind; kit lines resolve as kits do (`content::kit::resolve`: objects, alternatives, `magic`
on base items, a free slot for `wear`/`wield`; `prefer_trained` is the one addition to the kit
syntax, honoured by kits too, `KitItem::pick`); a stat 1 or 2 either way, at most 2 a stat a
line; gold always given or always taken; skill dice at least 1; marks at least 1; alignment
1 to 100 either way; `talent` and `flaw` effects are refused until B5; a line's net worth within
its tier's band (`Tier::band`: common −4 to 2, uncommon 1 to 3, rare 3 to 5, fateful 5 to 8;
rare and fateful never cost anything); every chapter has a plain line (common, no `when`, no
effects). Content without `data/background.toml` tells no story and checks nothing (test
fixtures that strip talents, skills, races, or classes remove it; `background::storyless()` is
the built-in content without it, for tests of what a story would add to).

## The roll

`World::tell_background` builds the `Witness` from the finished character (after `begin` and
`take_creation`): the character (spec, sign, birth date), the age at the start date, a lucky day,
the starting band (before the story's nudges), the trained skills, the talents held (creation
picks, Random ones too, the sign's gifts, the race's and the class's level 1 grants), the weapon
skills with a rank (what `prefer_trained` prefers), the campaign, and `luck` (100; B4's
Favoured child scales it). `background::tell` then, per chapter in order, from its own stream:

1. the tier: `rng.below(1000)` against `Tiers::of(roll, luck)`;
2. the pool: if a line with `only` fits, the fitting `only` lines (they are common, so they
   force the chapter whatever the tier: Family wealth's `old_money`, Father's sword's
   `fathers_blade_told`); else the lines of the tier drawn that fit, falling a tier at a time to
   common (whose plain line always fits);
3. the line: `rng.below(total weight)`;
4. its effects' dice in order: gold and skill dice, an item's alternative (`KitItem::pick`) and a
   magic item's loot draws (`loot::make` at `kit::ITEM_LEVEL`, identified).

The tags it sets are known to the chapters after it. Nothing is counted across chapters, capped,
dropped, or drawn again (the owner, 2026-10-29: no caps, no protection). The `only` rule is a
little stronger than the issue's wording ("an `only` line fitting narrows the pool"): it applies
whatever tier was drawn, so the heritage talents always force their lines.

## The effects (`world/background.rs`)

| Kind | Applied by |
|---|---|
| stat | `Stats::gift(stat, by)`: score and potential together, within 1..=40 (the one floor) |
| gold | coins joining the pack (`object::add_to_inventory`), or taken from the last stack back, down to none (emptied stacks removed, `Equipment::after_removal`) |
| item | the kit's way: a `Pack` of the player's inventory and equipment, `Pack::put`, `Pack::equip` for `wear`/`wield` (a free slot, else carried); food made now (`rot::made_at`); a scroll or potion joins `known_scrolls` |
| skill | `Skills::add` (trained at the roll, or raised) |
| weapon marks | `WeaponSkills::add_marks` (never a rank: stops one short of the next); `best` is the weapon skill (not a style) at the highest rank, ties by data order, none without a rank |
| alignment | `Score::new(score + by)`: the start's fourth term, never `shift_alignment` |

Then hit points and mana to their maxima. `start_run` sets mana and the perception after it, as
before.

## Who, and what is saved

Only a run that starts with the kit (`WorldState::kit`: New run, `--character`, `--name`,
`--talents`, a replay with `kit`) is told a story; `World::new`, `with_character`, `World::draft`
(the Talents step), and every `--scenario`-only run, test, and golden walk are not, and draw and
save nothing new, so their hashes hold. `Character::story: Vec<Told>` (ids) is saved and hashed
with the character, left out while empty (saves change: old saves load without a story;
`FORMAT_VERSION` untouched). The text is never saved: `retell` tells it from the content, so an
edited sentence reaches a saved character; a line the content lost is skipped (its `set` too)
and `World::from_state` adds a log line for it to `World::retiered`. The effects are in the actor
and are never applied again. Every kit run's hash changed once with B1 (the screen snapshots
`kit-*`, `character-*`, `level-up-*`, `abilities-menu-*`, and others built on kitted runs).

## Dumps

`story` layer: the paragraphs as the story screen will tell them, a blank line between (`no
story` without one). JSON: `story: [{chapter, line, text}]` (empty without), and
`character.story` (`["parents/plain_folk", …]`, left out while empty). The dev `actors` layer
adds `story parents/plain_folk birth/hard_winter …` to the player's line.

## Tests

`content/background_tests.rs`: the built-in set (every effect kind and tier, the forced lines),
`texts_hide_the_numbers` (no stat's name, no digit or sign, none of seed, rare, tier, talent,
bonus, rolled, generated), every refusal. `background_tests.rs`: the tier odds over 20 000 seeds
(every shared chapter's stream, within a tenth), no score below 1 over every race, gender, and
class on a thousand stories, the fall to the plain line, the heritage talents' forcing, slots,
defaults, pronouns, a missing line, each condition, determinism. `world/background_tests.rs`:
each effect lands (fixture chapters of one forced line), gold down to none, the story after the
picks (Hardy's Toughness), who has one (and unchanged hashes), a replay and a save keep it, an
edited text reaches a save, a lost line is skipped and logged. `kit::tests::
nobody_starts_strained_and_only_wealth_or_armour_burdens` adds the heaviest story line (items and
gold) for every class but the aristocrat's and merchant's.

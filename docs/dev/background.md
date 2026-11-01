# Background story

Developer notes for a chosen character's background story (spec §8 "Background story"; the
design is `docs/design/background-story.md`, tracked in #708). B1 (#702) is the data, the roll,
the effects, and the saved ids; the story screen and its page (now 9) are B2 (#703), the full pools and
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
1 to 100 either way; a `talent` a known one, never `granted`, `flaw`, or of a class tree, its
choice (`"id:choice"`) given exactly when it takes one and one of `Choice::options`; a `flaw` a
known `flaw` talent (B5); a line's net worth within
its tier's band (`Tier::band`: common −4 to 2, uncommon 1 to 3, rare 3 to 5, fateful 5 to 8;
rare and fateful never cost anything); every chapter has a plain line (common, no `when`, no
effects). Content without `data/background.toml` tells no story and checks nothing (test
fixtures that strip talents, skills, races, or classes remove it; `background::storyless()` is
the built-in content without it, for tests of what a story would add to).

## The pools (B3, #704)

The shared chapters (`assets/data/background/01-parents.toml` to `07-leaving.toml`) and the
Ascmearc's `road` hold 126 lines, written by the implementers (decision 13 of the design; the
owner reviews the texts after the wave): Parents 18, Birth 12, Childhood 17, Youth 27, The trial
15, Fortune 15, Leaving 15, the road 7. Of them 17 are uncommon, 11 rare, and 5
fateful (`the_comet`, `the_master`, `the_flux`, `the_named_blade`, `the_hollow_tree`). How
they are spread:

- **Parents** sets the words and the tags the rest lean on (`poor`, `wealthy`, `orphan`,
  `smith`, `trader`, `hunter`, `deep`, `warren`, `outsider`, `herbs`; `{guardian}`, `{home}`,
  `{trade}`). Every race has a line of its own, weighted above the shared ones (6 to 12 against
  the plain line's 5) so most of a race's stories start in its own world: the deep halls
  (dwarf, duergar), a low house of the under-dark (drow), the warren (kobold, goblin, ratfolk),
  the fen-clutch (lizardfolk), the burrow (halfling, gnome), the wood's edge (elf, half-elf),
  two peoples (half-orc, half-elf); `farm_folk` is not told to the under-dark's races.
- **Youth** is the class chapter: nearly every class has a line of its own, weighted as the
  plain line (the levy, the raids,
  the minster, the cloister, the copyist, the cunning woman, the alehouses, the market children,
  the workshop, the counting house, the idle youth, the woods), plus tag lines (`smith` and
  `trader` apprenticeships, `hunter`, `orphan`, `poor`) and age gates (a drow's half-lifetime as
  a page from 50, the long apprenticeship of the long-lived from 80, the old soldier and the
  master from 16).
- **The trial** is mostly common lines that take and keep (a stat traded, gold lost with a skill
  kept); its rare line is a terrible thing survived. **Fortune** holds the purses and the
  heirlooms. **Leaving** follows the tags (`orphan`, `poor`, `indebted`, `wanderer`, `warren`,
  `strong`) and nudges the alignment.
- **The road** (the Ascmearc's) is the one chapter naming the march's places; the showcase's
  two-line `road` stays the tests' chapter.

`when.tags` holds when **every** tag is set (and the loader refuses a tag no earlier chapter
sets, so `not_tags` on a tag of the same chapter is refused too): a line meant for "an orphan or
a priest" is two lines. A line that kills `{guardian}` (`the_fever`, `the_debt`, `the_heirloom`) sets `orphan`, so
no later line has them alive (the quarrel, the parting gift). `{guardian}` may be any of the
values the Parents lines set ("the
sisters", "the woman who took you in", "the elders of the clutch", …), so a line using it reads
with each; `{home}` defaults to "home", so a line using it reads after "at", "around", or "for",
never "through".

`trial/the_flux` gives `{ talent = "hardy" }` and Toughness +1 (B5, #707; worth 6), and
`fortune/the_named_blade` a `magic = "rare"` weapon, identified (B1's item path). No pool line
gives a flaw yet: a flaw is a bane, so only a common or uncommon line may carry one.

**The writing rules** (design §11.9): second person, past tense, the plain English of the
march's day; Cockney and posh only inside quoted speech, by the speaker's station (the pedlar's
"Yer'll want that"); Old English names only in a campaign's chapter; one line, one thing; grim
allowed, cruel never; no digit, sign, stat name, or word about how a story is drawn
(`texts_hide_the_numbers`: a stat's name anywhere in a text, even inside a longer word, fails
it); every race's own childhood; ids never change (saves name them).

**The samples** (`background::tests::the_samples_read_as_written`): three kitted runs on
Beorggeat (`World::start_run`, so the witness is the real one, picks and all), their stories
pinned as text: Edwin (human commoner, seed 85, eight common lines; 72 until B4's talents moved his Random pick), Brakka (dwarf warrior,
seed 24, the rare heirloom), Tamsin (halfling thief, seed 21, a grim one). A pool edit that
moves a sample's lines fails it: pick new seeds from `find_samples` (ignored; `cargo test -p
warlock-core find_samples -- --ignored --nocapture` prints 300 stories of each with every
line's kind and worth) and read the new stories before pinning them.

## The wiki (B3, #704)

`xtask/src/wiki/background.rs`, one function a table, registered in `generate()` after the
horoscope's: `background-tiers.md` (each kind's odds per chapter, at least one and the expected
count in a story of the Ascmearc's eight chapters, its worth band, whether it may take),
`background-points.md` (`[points]` in words), and `background-<chapter>.md` for the eight
chapters of a story in the Ascmearc (never the showcase's road): every line, commonest first
then by weight, with its kind (and `only`), weight, conditions in words, text (slots as
written), effects exactly with the line's worth, and the tags and words it sets. The page is
`wiki/src/background.md`.

## The roll

`World::tell_background` builds the `Witness` from the finished character (after `begin` and
`take_creation`): the character (spec, sign, birth date), the age at the start date, a lucky day,
the starting band (before the story's nudges), the trained skills, the talents held (creation
picks, Random ones too, the sign's gifts, the race's and the class's level 1 grants), the weapon
skills with a rank (what `prefer_trained` prefers), and the campaign. `background::tell` then, per chapter in order, from its own stream:

1. the tier: `rng.below(1000)` against `Tiers::of(roll)`;
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
| talent, flaw | `World::bestow(id, choice)` (`world/boons.rs`, the body of the altar's Chosen and Forsaken with the id fixed, now `pub(super)`): `Talents::take`, `class::hand_out` at rank 1 (Hardy's hit points), `Actor::boons`, `stars::grant`'s order, `train_chosen`; no message (the story is shown, never told) |

## Talents and flaws (B5, #707)

A line that gives a talent or a flaw is never drawn for one who holds it (any rank; a talent
with a choice for that choice) or whose earlier line gave it: `tell` filters the pool by
`gifts_of(line)` against `Witness::talents` and the gifts drawn so far, beside `Witness::fits`,
so a line's `when` need not say `not_talents`. A boon is saved already (`Actor::boons`), so
nothing new is saved and every restore puts it back (`stars::grant`); a class table that names
it later skips it (`class::grant`: held). `background::gave(content, campaign, character, id,
choice)` says whether a told line gave it: `OwnTalent::story`, so the Talents page shows
"(story)" / "(story, flaw)" (`ui/talents_screen.rs`), the `talents` dump the same, and the
character screen's sign list leaves a story flaw out. A story boon is not `OwnTalent::boon`
(the altar's). The wiki's chapter tables name it, "the talent [Hardy](…)", "the flaw […]".

Then hit points and mana to their maxima. `start_run` sets mana and the perception after it, as
before.

## Talents

None. The `background` system (B4, #705) had Favoured child, a heritage talent that doubled the
luckier tiers' odds (`story_luck`); #787 cut it and dropped the system (Hard-won and Storyteller
had gone in #774). A save naming it loads without it (`Talents::drop_unknown`).

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

## Shown (B2, #703)

The story is shown, never told in messages: `PerceptionView::story()` (the paragraphs of
`retell`, from the perception's character and campaign; empty without a story) feeds the
story screen (`crates/warlock/src/ui/story_screen.rs`), the `story` dump layer, and nothing
else. The app opens it once as a run begins (`screens::on_run_start` on `Game::story_due`: New
run's Start, a windowed `--character`/`--name`/`--talents`/`--new` start; never Continue, Resume,
`--capture`, `--replay`, beneath a `--screen`, or for a run without a story), 72 wide; afterwards
it is info page 9, "Story" ("Your story is not written." without one). No saved "seen" flag and
no new `KeyCommand` (`Page(9)` exists). `--screen story` captures the opening screen. The
character screen, the death screen, and the Characters screen show nothing of it (decision 6).
Details: `docs/dev/screens.md`, "The story".

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
edited text reaches a save, a lost line is skipped and logged; B5's: a story talent at rank 1
with its hit points, through a save and a restore, "(story)"; a held talent (picked, or given
by an earlier line) never given again; Evasion from a story and the thief's table at 18
(nothing in its place); a flaw "(story, flaw)" (a sign's own flaw is skipped: the warrior of
seed 7 was born Frail); `the_fateful_lines_at_pinned_seeds` (a human warrior on Beorggeat:
seed 107 the flux, 398 the named blade). The screen snapshot `talents-screen-story` is seed
107's Talents page. `kit::tests::
nobody_starts_strained_and_only_wealth_or_armour_burdens` adds the heaviest story line (items and
gold) for every class but the aristocrat's and merchant's.

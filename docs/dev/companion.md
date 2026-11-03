# The animal companion

Developer notes for the companion, a beast that is the player's for good (spec §8 "Allies": "The
animal companion"; the design is `docs/design/ranger-companion.md`, §18 and §20 the owner's
decisions, which override the rest). Slice **C1** (#916, this file: the starting beast, the
story's chapter, growth, always along, the nose) and **C4** Pack tactics (#919, below); C2 Tame, C3 the collar, C5
Hunter's bond, C6 Sic 'em, C7 the wild beasts, C8 the probe build on it. The ally core it uses as
it is (the side, the arm, the orders, the retinue, the kill credit) is `docs/dev/allies.md`.
The witch's familiar (`docs/dev/familiar.md`) is the same **kept** beast: what here is the name,
growth, always along, the words, the status bar, and the Character screen's row reads
`AllySource::kept()` (`Actor::kept()`), and only what is the ranger's reads `Actor::companion()`.
Keep it current with the code, as CLAUDE.md says.

## What a companion is

An ally of the core (`Actor::ally`, `world/ally.rs`) with `source =
AllySource::Companion(CompanionData)` and `until = None`:

```rust
pub struct CompanionData {
    pub name: String,            // "Snel", said without an article
    pub grade: u8,               // its own level: its master's, never below its type's
    pub scent: Option<ActorId>,  // what its nose is on (the growl is said once per thing)
}
```

`World::companion()` is the player's one ally of that source, on any level or in the retinue
(`Actor::companion()` its data). A character has at most one: Animal companion makes none with
one at hand, and Tame (C2) is refused with one. Nothing is kept on `WorldState`: the companion is
derived from the actors, saved and hashed with its actor as every ally is. The code is
`world/companion.rs`; the data `assets/data/companions.toml` → `content/companion.rs::CompanionRules`
(`Content::companions()`, `None` without the file: then no companion can be made, and fixtures
without it hash as before).

## Gaining one: the starting beast

**Animal companion** (`assets/data/talents/animal_companion.toml`: tree `class`, `granted`,
effects `companion` and `tame = 2`, system `allies`) is the ranger's level 1
(`data/classes/07-ranger.toml`). `TalentEffect::Companion` → `TalentMods::companion`;
`TalentEffect::Tame { bonus }` → `TalentMods::tame`, read by the Tame skill ability (C2). The
check: a `tame` bonus of 1 to 10.

`World::bring_companion()` runs in `World::begin` once the level is laid and the player stands on
it (after `class::grant`'s level 1 and `assemble`, so **every** run with a ranger character has a
beast from the first turn: New run, `--character`, the showcase's Osric (`world`) and Maren
(`forest`); and before `tell_background`, so the story knows it). Nothing without the talent, a
character, `allies.toml`, `companions.toml`, a list for the race, or with a companion already;
nothing on `from_state` (a saved run has its own, or lost it).

1. **The draw** (`companion::draw`, pure): the stream `companion/<race>,<gender>,<class>`
   (`Rng::stream`; `CharacterSpec`'s `Display`, never the picks), two draws and nothing else:
   `below(1000) < [start] rare_per_mille` (50) picks the **rare list** (`[start] rare`, shared by
   every race), else the **race's list** (`[start.by_race] <race>`); then `below(len)` picks the
   beast.
2. **The actor**: `make_ally(creature, free_cell_near(player), Companion(..), None)`, the
   core's way (the player's faction, no gear); on the overworld, made at the player's cell in the
   retinue at once (as the dev `ally:` does). Its `grade` is `Growth::grade(player level, type
   level)`; its hit points are set to the grown maximum.
3. **The name** (`companion::name_for`): one draw on `companion/name/<actor id>` from `names`.
4. The line: "Snel, your hound, pads at your heel."

**The data** (`companions.toml`): `[start] rare_per_mille` 0 to 1000, `rare` 1 to 8 known
`tameable` beasts, `most_level` (2); `[start.by_race]` a list of 1 to 3 known `tameable` beasts
of level at most `most_level` per race (a race the content lacks is refused; a shipped race
without a list is a test's failure, `content/companion.rs::tests`, not a load error, so fixtures
that add races load); `[tame]` (C2's numbers: `turns` 1 to 100, the rest 0 or more);
`[growth]` (`grade_percent` 1 to 300, `hp_per_level` 0 or more, every `every` 1 to 60); `names`
at least one, none empty. When the content tells stories and has the `companion` chapter, every
beast a start may draw has an `only` line in it with `when = { companion = ["<id>"] }`.

## Creature types: `tameable` and `nose`

`CreatureDef::tameable` (default false) marks a type a companion may be; the loader refuses it on
a unique, an animated object, a summoner, one that `opens_doors`, one of the peoples (a `race`),
one that fights from afar, or one that carries gear (`racial_weapon` or `gear` lines). The
faction is not what decides: the orcs' wargs are beasts of a people's faction, and tameable (the
design's §12.2 lists them; its "animal faction" clause would have refused them, so this is the
check instead). `CreatureDef::nose: Option<i32>` (1 to 20, only on a `tameable` type) is how far
it smells.

The ten new beasts (`hound`, `polecat`, `lynx`, `badger`, `boar`, `large_spider`, `cave_lizard`;
the rares `black_hound`, `great_boar`, `panther`) are shared types in `assets/data/creatures/`
on the beast roster's conventions (`docs/dev/creatures/beasts.md`: wildlife, tag `beast`, a
`depart` line, XP by the guideline); `wolf`, `bear`, `rat`, `bat`, `beetle`, `deer`, `warg`, and
`dire_warg` gained `tameable` (and a `nose` where the design gives one). Adding creature types
reorders the creature list: development saves are invalidated (saves are pinned to the version,
`docs/dev/saves.md`; no aliases, no shims).

## Growth

`CompanionData::grade` is set when the companion is gained and re-set by `World::grow_companion`
at the end of `World::level_up` (`[growth] grade_percent` of the master's level, rounded down,
never below the type's level, never down): "Snel grows stronger." among the level's news, the
current hit points rising with the maximum. Summons have no grade, so every hook below costs
them nothing. The rule is the companion's, whoever its master (the owner's §20: growth for every
master).

The hooks read `Actor::grade_above(content)` (`above` = grade − the type's level, never below 0;
0 for anyone that is not a companion) in `world/companion.rs`'s `impl Actor`, each called from
one place in `actor.rs`:

| Hook | Where | Rule |
|---|---|---|
| hit points | `Actor::max_hp` | `+ hp_per_level × above` (2) |
| to hit, damage | `Actor::melee` (`Actor::grown`) | `+ above ÷ to_hit_every` (4); `+ above ÷ damage_bonus_every` (6) flat and `above ÷ damage_dice_every` (12) more dice of the attack's first group (`Roll::more`) |
| Defence | `Actor::defence` | `+ above ÷ defence_every` (6) in the Natural category, with the hide's or alone |
| Protection | `Actor::protection_plus` | the body rule at the grade (`companion::body_protection`: by what the hide is called, as `creature_body_tests` groups them: a hide (grade − 10) ÷ 8, bony grade ÷ 6, armoured grade ÷ 4 + 1, a body with none 0; no hide: a hide), never below the type's own |
| stats | `Actor::stats_with` | St, Dx, To `+ above ÷ stat_every` (5), so Toughness feeds the hit points and Strength the blow too |

Speed, senses, size, and resistances are the type's. The wiki prints the table at the cadence
levels through the real `Actor` (`xtask/src/wiki/companions.rs::growth`).

## Always along

`World::follow_along` (`ally.rs`) takes every ally of the player's on the level left whose source
is `Companion`, **wherever it stands and held fast or not**, beside the core's `comes_along` rule
for the rest (a webbed hound is pulled free by the move; a holding one's order becomes *Follow*,
as every follower's). It arrives beside the player ("Snel comes with you."), joins the retinue
onto the overworld and steps out at the next level ("Snel follows you in."), and so is never on
another level than its master. The one thing that keeps it from a level is death.

## The nose

`World::smelled(id, wanted)` (`companion.rs`): for an ally whose type has a `nose`, the nearest
`wanted` actor within `nose` cells (Chebyshev; ties by lower id) that a smell reaches: a line of
sight from the beast with **closed doors counted open** (`LevelState::is_closed_door`), whatever
the light (the owner's §20: through closed doors too; never through walls). A disguised mimic
smells of nothing (`hides`). `ally.rs::ally_plan`'s target is `World::senses`: the nearest seen
(`nearest_in_sight`), else the nearest smelled, under the order's reach as before. Found by the
nose, `World::scented` remembers it in `CompanionData::scent` and says "Snel growls at the dark."
once per thing scented, when the player sees the beast and not what it smells (a summon with a
nose, the dev `ally:hound`, hunts in silence: it has no memory for the growl). A door it cannot
open keeps it from what it smells: it waits for the ranger to open it. Nothing here draws from
any stream.

## The words

`World::called(actor)` says a companion's name first ("Snel bites the kobold."); `fight::End::Named`
is the companion's end in `perceive.rs` and the player's own blow at it ("Snel dies.", "You hit
Snel."); `ally.rs::pack_phrase` names it before the summons ("Snel comes to heel.", "Snel and your
wolves come with you.", "You set Snel on the kobold."); a hunt arrived says "Snel looks back at
you." (`World::ally_subject`). A dead companion's corpse is "a hound corpse", the type's.

## The story's chapter

`assets/data/background/08-companion.toml` (shared, `paragraph = 4`, after Leaving; a campaign's
chapter stays last): `[chapter] when = { companion = true }` (`ChapterWhen`, `Chapter::when`: a
chapter whose `when` does not fit the witness is skipped by `background::tell` **before its
stream is drawn**, so a story without a companion hashes as before) and fourteen `only` lines
(common, worth 0, no effects) with `when = { companion = ["<id>"] }` (`When::companion`, creature
ids checked at load: known and `tameable`; `Witness::companion` is the companion's type, filled by
`World::tell_background` from `World::companion()`), plus the plain line, the net for a beast
without one. Details in `docs/dev/background.md`.

## Pack tactics (C4, #919)

The ranger's level 18 (`assets/data/talents/pack_tactics.toml`: tree `class`, `granted`, effects
`pack_tactics` with `to_hit = 2` and `shares_favoured_prey`, systems `combat` and `allies`).
`TalentEffect::PackTactics { to_hit }` (checked 1 to 10) → `TalentMods::pack_tactics`;
`TalentEffect::SharesFavouredPrey` → `TalentMods::shares_favoured_prey`. One hook,
`World::pack_blow` in `world/pack.rs`, called in `fight.rs::resolve` after `ally_blow` (Beast-bond):

- **flanked**: the target stands beside (Chebyshev 1) both the player and `World::companion()`,
  the companion on the current level and not in the retinue. The player's `Melee` attacks and the
  companion's at it get `+pack_tactics` to hit; `Ranged`, `Thrown`, and `Spell` never;
- **the shared prey**: the companion's attacks (any kind) at a creature of `Quests::prey()` (by
  `family_key`, as `ambush.rs::favoured`) carry the player's `TalentMods::favoured_prey`, to hit
  and damage (Favoured prey's +1/+2, Master hunter's adding +1/+2). The player's own prey bonus
  stays `favoured`'s.

Only the player's talents are read (a companion is the player's). Nothing is said and nothing is
saved: the flank is read from where everyone stands at the blow. It is no clone of Beast-bond
(`ally_mod`): the alike check compares effect kinds. Tests: `world/pack_tests.rs` (the table's
row and the mods; flanking both ways, diagonals, not for shots, throws, spells, or the kobold's
blows; one side only or the retinue, nothing; the shared prey with and without Master hunter,
flanked too, and another type nothing).

## What the player sees

- **Messages**: above; the start's "Snel, your hound, pads at your heel.", growth's "Snel grows
  stronger.", the nose's growl.
- **The status bar** (`ui/mod.rs::status_segments`): "Snel: unhurt" (`Health::word`), "Snel: with
  you" in the retinue; priority 2, after the stance.
- **Look mode** (`ui/look.rs`): "Snel, your hound (unhurt), on grass." (`SeenActor::companion`
  marks it; the name comes from `PerceptionView::companion()`).
- **The Character screen** (`ui/character_screen.rs`): the row `With    Snel, a hound` under the
  identity rows (the label column is eight wide, so not "Companion:").
- **The story screen**: the chapter's paragraph, as any.
- **Dumps**: the `companion` layer (`Snel, a hound (grade 12) at (12, 5): unhurt, order follow`,
  the cell only where the player sees it; `Snel, a hound (grade 12): with you` in the retinue;
  `no companion`); JSON `companion` (`id`, `name`, `creature`, `grade`, `health`, `order`, `at`,
  `pos` (seen, else null), `retinue`; null without one); the `allies` JSON's `source` is
  `"companion"`; the `actors` layer appends `; named Snel, grade 12`; the header's `allies N`
  counts it. `PerceptionView::companion()` is the one read for the UI and the dumps; the
  snapshot's `AllyView::source` carries the data.

## Determinism, saves, replays

Streams: the beast on `companion/<spec>` (two draws), the name on `companion/name/<id>` (one),
the chapter on its own `background/<spec>/companion` stream (skipped entirely for a witness
without a companion); growth, the following rule, and the nose draw nothing. Saved:
`CompanionData` inside `Actor::ally`. Every ranger run gained an actor and a chapter, so the
ranger scenarios changed (Maren's beast beside her in `forest`: the `forest-heavy-rain` golden,
updated in the finisher's pass; Osric's in the retinue on `world`, never drawn), and so did every
screen snapshot built on a ranger run; no non-ranger run's hash moved.

## Tests

`world/companion_tests.rs`: the draw over every shipped race and 400 seeds (its list or the
rares, the rare odds, determinism, the names); a kitted ranger's beast (named, whole, for good,
on the player's side, said, its `only` line told, the dumps) and a warrior's none; the
scenarios' rangers (Maren's beside her, Osric's in the retinue), a restored run making no
second; growth at level 4 and far up the curve (every hook against an ungraded twin, the news);
the body rule; a companion coming along from across the level while a summon is left behind;
the retinue in and out (and a replay of the road); the nose through a closed door that stops the
eyes (the lynx without one; beyond its reach, nothing), the growl once; the words (the orders,
the kill, the dumps, the mark); a save round trip. `content/companion.rs::tests`: the shipped
file and every race's list, the refusals, loading without the file, `tameable` and `nose`
refusals. `content/beast_roster_tests.rs`: the twelve beasts' rows. `background_tests` pins the
chapter list; the samples (`the_samples_read_as_written`) prove a non-ranger's story unchanged.
Screen snapshots (`crates/warlock/src/ui/screen_snapshots.rs`): every ranger case shows the
status bar's segment, the Character screen's row, and the Allies group.

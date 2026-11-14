# The animal companion

Developer notes for the companion, a beast that is the player's for good (spec §8 "Allies": "The
animal companion"; the design is `docs/design/ranger-companion.md`, §18 and §20 the owner's
decisions, which override the rest). Slices **C1** (#916: the starting beast, the story's chapter,
growth, always along, the nose), **C2** Tame (#917, "Taming" below), **C3** the collar (#918,
below), **C4** Pack tactics (#919, below), **C5** Hunter's bond (#920, below), and **C6** Sic 'em
(#921, below) are this file; the design is complete, C7 the wild beasts and C8 the probe aside. The ally core it uses as
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
    pub quarry: Option<ActorId>, // what Sic 'em set it on, until a new order
}
```

`World::companion()` is the player's one ally of that source, on any level or in the retinue
(`Actor::companion()` its data). A character has at most one: Animal companion makes none with
one at hand, and Tame is refused with one. Nothing is kept on `WorldState`: the companion is
derived from the actors, saved and hashed with its actor as every ally is. The code is
`world/companion.rs`; the data `assets/data/companions.toml` → `content/companion.rs::CompanionRules`
(`Content::companions()`, `None` without the file: then no companion can be made, and fixtures
without it hash as before). Its `[start]` (the lists by race and the rares) is
`content/race_beasts.rs::RaceBeasts` (`CompanionRules::start`), shared with the familiar's
file, which checks it and the names the same way (#1567).

## Gaining one: the starting beast

**Animal companion** (`assets/data/talents/animal_companion.toml`: tree `class`, `granted`,
effects `companion` and `tame = 2`, system `allies`) is the ranger's level 1
(`data/classes/07-ranger.toml`). `BeastEffect::Companion` → `BeastMods::companion`;
`BeastEffect::Tame { bonus }` → `BeastMods::tame`, read by the Tame skill ability ("Taming" below). The
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

**The data** (`companions.toml`): `[start] rare_per_mille`, `rare` (known creatures, never the
player, at least one: the draw indexes it), `most_level` (2); `[start.by_race]` a list of known
creatures per race, at least one (a race the content lacks is refused; a shipped race without a
list is a test's failure, `content/companion.rs::tests`, not a load error, so fixtures that add
races load); `[tame]` ("Taming" below); `[growth]` (every `every` at least 1: they divide);
`names` at least one. The rest is the author's (#1205): the numbers, whether a beast listed is
`tameable`, the beasts' levels against `most_level`, and an `only` line in the `companion`
chapter for every beast a start may draw
(`when = { companion = ["<id>"] }`; a beast without one is told the chapter's other lines).

## Taming

`Ability::Tame` (`Source::Skill`, `Aim::Direction`; Handle animal's use, listed in its place,
`Ability::use_of`, while `companions.toml` and `allies.toml` exist; had by whoever is trained in
a skill with the `tame` benefit) → `Action::Tame(dir)` (`tame:DIR`) → `world/tame.rs`, whose
module docs are the rules. `start_tame` refuses without time or a draw (untrained, a companion
already, no beast seen there, one that is an ally or not `tameable`, one hostile that has seen
the player); the menu greys the row with `Unavailable::HaveCompanion` (the perception's
`companion()`), the only refusal it can know up front. `Spent::Tame(Taming { beast })` runs
`World::tame` on `World::keep_at`, as a dig does: `[tame] turns` of `ActionKind::Tame`, stopped by
keep_at's own interruptions, hit points falling (compared turn by turn, so regeneration hides no
blow), or `taming_holds` going false. The last turn's action is `tame_check` (before that turn's
creatures and perception, so both know what came of it): the food (one of the first food
stack, `first_food`; an interrupted try eats none), `tame_odds`, `check::check` on the gameplay
stream (two draws), the mark (`Use::Handle`, made or failed), `wake`, and `World::adopt`
(`companion.rs`: the player's faction as `faction_override`, `Mind` cleared, `Ally` with
`Companion(CompanionData)`, the grade and the risen hit points, the name's one draw on its own
stream) or the snap. Failed and hostile, the beast's `Mind::last_seen` is set, which is what
`creature_unaware` reads (the design said `noticed`; that field is when the player last perceived the creature, and
does not refuse a try), so it may not be tried again until it loses the player. No state beyond
the actors; no cooldown (time is the price). Tests: `world/tame_tests.rs`; the odds in the wiki,
`xtask/src/wiki/companions.rs::taming` (`companion-taming.md`).

## Creature types: `tameable` and `nose`

`CreatureDef::tameable` (default false) marks a type Tame may make a companion of. The shipped
tameable types are plain beasts (no unique, animated object, summoner, door-opener, one of the
peoples, ranged fighter, or gear-carrier), but that is the data author's choice, not a load check
(#1205). The faction is not what decides: the orcs' wargs are beasts of a people's faction, and
tameable (the design's §12.2). `CreatureDef::nose: Option<i32>` is how far it smells (read for a
companion or a familiar).

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
| Protection | `Actor::protection_plus` | the body rule at the grade (`companion::body_protection`: by what the hide is called, as `docs/dev/creatures.md` "Natural Protection by body and level" groups them: a hide (grade − 10) ÷ 8, bony grade ÷ 6, armoured grade ÷ 4 + 1, a body with none 0; no hide: a hide), never below the type's own |
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
any stream. The same nose shows the player the unseen (`world/unseen.rs::unseen_by_player`, by
`smelt_within`): an invisible or shadowy creature within the companion's `nose` is in the
player's perception, wherever the companion stands on the level (not in the retinue), with or
without Hunter's bond.

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

## The collar (C3, #918)

(An escort, `docs/dev/allies.md` "Escorts", takes Give too: anything that fits one of its slots.)

`world/collar.rs`, `Action::Give { item, ally }`, `give:LETTER[:DIR]`, `Ability::Give`
(`Source::Ally`, `Aim::Item`; the last row of the Allies group, listed while the player keeps a
beast, a companion or a familiar, `Ability::orders(any, kept)`; unavailable `NotBeside`, from the
snapshot's `kept_beside`). One of the stack goes from the player's pack to the end of the beast's
`Actor::inventory` and into its `neck` slot; a last one worn by the player comes off first, paying
for that too (`take_off_before_dropping`). Priced as a `wear`. The old one leaves the beast's
inventory for the pile on its cell, with the ordinary rules (standing in an open shop, that makes
it the keeper's goods, as anything lying there is). `world::fits_neck` is the Give list's filter
and the rule's. Unpaid goods are refused (`may_loose`).

Nothing new is saved: the collar is an ordinary stack in the beast's equipment, so `Mods::worn`,
`Actor::eyes`, and the regeneration affixes read it, `SeenGear` shows it (look mode's `wearing`),
`World::creature_dies` drops it, and the `actors` layer lists it. `AllyView::wears` carries it to
the `companion` and `familiar` dumps wherever the beast is (`, wearing a jade amulet`; JSON `wears`, null with
none): the player fastened it. No identification: nothing reads the beast's wearing for lore.

**Kept beasts.** `World::collar_wearer` is the one place that says which ally wears a collar
(`Actor::kept()`: a companion or a familiar), and `ally` picks the way. With more than one kept
beast beside the player (`PerceptionView::kept_beside()`; the owner's #907 Q4) the game asks: the
Give list's choice reaches `Gameplay::act`, which enters `InputMode::GiveWay(item)` instead of
sending it, "Give it to which side?" in the message line, and a direction sends `ally: Some(dir)`
(any other key cancels, nothing spent). The list comes first, unlike Heal's way
(`input.rs::tend_way`), so a player with nothing for a neck is never asked. A way with no kept beast
is the rules' to refuse, no time: "No beast of yours stands that way."

## Pack tactics (C4, #919)

The ranger's level 18 (`assets/data/talents/pack_tactics.toml`: tree `class`, `granted`, effects
`pack_tactics` with `to_hit = 2` and `shares_favoured_prey`, systems `combat` and `allies`).
`BeastEffect::PackTactics { to_hit }` (checked 1 to 10) → `BeastMods::pack_tactics`;
`BeastEffect::SharesFavouredPrey` → `BeastMods::shares_favoured_prey`. One hook,
`World::pack_blow` in `world/pack.rs`, called in `fight.rs::resolve` after `ally_blow` (Beast-bond):

- **flanked**: the target stands beside (Chebyshev 1) both the player and `World::companion()`,
  the companion on the current level and not in the retinue. The player's `Melee` attacks and the
  companion's at it get `+pack_tactics` to hit; `Ranged`, `Thrown`, and `Spell` never;
- **the shared prey**: the companion's attacks (any kind) at a creature of `Quests::prey()` (by
  `family_key`, as `ambush.rs::favoured`) carry the player's `TalentMods::favoured_prey`, to hit
  and damage (Favoured prey's +1/+2, Master hunter's adding +1/+2). The player's own prey bonus
  stays `favoured`'s.

Only the player's talents are read (a companion is the player's). Nothing is said and nothing is
saved: the flank is read from where everyone stands at the blow. Tests: `world/pack_tests.rs` (the table's
row and the mods; flanking both ways, diagonals, not for shots, throws, spells, or the kobold's
blows; one side only or the retinue, nothing; the shared prey with and without Master hunter,
flanked too, and another type nothing).

## Hunter's bond (C5, #920)

The ranger's level 25 (`assets/data/talents/hunters_bond.toml`: tree `class`, `granted`, effects
`shared_sight` and `shared_buffs`, systems `vision`, `allies`, `status`). Two separate effects, so
the witch's familiar takes the sight and the nose without the buffs (`BeastEffect::FamiliarSight`,
the same viewpoint, `docs/dev/familiar.md` "Familiar's sight"; `shared_sight` itself reads only the
companion).

**Shared sight** (`BeastEffect::SharedSight` → `BeastMods::shared_sight`), a second viewpoint in
`Perception::compute` (`perception.rs::BondView`, `bond_view`), only while the companion stands on
the player's level (nothing from the retinue):

- `sight::LineOfSight::cells` of the companion's line of sight within its type's sight
  (`sight::sight_radius`), each seen iff `sight::sees_by` says so with its own eyes
  (`Actor::eyes`: creature sight's rule, so a dark cell beyond its darkvision stays hidden);
- `world::smelt` (`companion.rs`, what `World::smelled` takes the first of): the cell of every
  hostile its nose finds, by the player's stance, through the dark and closed doors, never a wall
  and never a disguised mimic (`hides`). A scented cell is seen; no wall completion round it;
- wall completion around the cells seen by its eyes (no reach limit from the player; `Seen` in
  line of sight, else `Edge`);
- the light grid covers the union of both lines of sight (`lit_los`, `lit_cells`; `near` reaches
  far enough, `light_reach`), perceived by the **player's** eye, so what the screen shows is in
  the player's colours; without the bond the two are the player's own and nothing changes.

They are ordinary `Sight::Seen` cells: memory, look mode, targeting, Set on, the seen actors, and
the dumps read them with no other change. Hidden traps and secret doors stay hidden (searching is
the player's own), the pass runs after the dazzle cut so a dazzled player keeps the companion's
view, and it draws nothing and changes nothing (perception is a pure read of the state).

**Shared buffs** (`BeastEffect::SharedBuffs` → `BeastMods::shared_buffs`), `world/bond.rs`, one
hook: `World::afflict`, the one door of every status (`status-effects.md`), calls `afflict_one`
and then `share_buff`. With the talent, a **buff** (`bond::is_buff`: a status of a buff spell, a
`circles` row that is not a hex, or one a potion lends, `Draught::lends`) on the player lands on
`World::companion()` (on the level or in the retinue) and one on the companion on the player, by
`afflict_one`, so at the same severity, with no echo; each is its own `Effect`, easing by its own
clock. A hex, a harm, a cooldown, Winded, Raging never cross. "Snel is bark-skinned too." when it
begins on the companion (the status's `seen` word). Nothing drawn.

Tests: `world/bond_tests.rs` (the sight is exactly the eyes plus the nose; a hostile behind a
closed door, and not behind a wall; memory; dazzle; the retinue lends no eyes; a buff both ways
and a hex never; a buff reaches a companion in the retinue). `class_rogue_tests.rs` pins the
table's row.

## Sic 'em (C6, #921)

The ranger's level 40 (`assets/data/talents/sic.toml`: tree `class`, `granted`, effect `sic` with
`status = "savage"`, `blows = 3`, `range = 10`, `resource = { id = "focus", cost = 5 }`, systems
`combat`, `resources`, `allies`). `TalentEffect::Sic` (checked: a known status with
`auto_critical` and no tick, `blows` 1 to its `max`, `range` 1 to 20) → `TalentMods::sic`;
`ClassActive::Sic` (`aims_cell`, so `Aim::Cell` and `use:sic:12,5`, through `World::use_class_at`;
the cost a wait's). `Content::sic_range` is `Ability::reach` for the aiming mode. The code is
`world/sic.rs`:

- **Refused**, no time, nothing spent: short of focus (`may_use_class`); no companion on the level
  or it is in the retinue ("You have no companion here."; the menu's `Unavailable::NoCompanion`,
  "no companion here", read from `AllyView` in `PerceptionView::abilities`); out of reach; no
  creature the player sees on the cell (a disguised mimic is none; nor is the beast); one not
  hostile to the player.
- **The hunt**: `set_order(Hunt { at })` then `CompanionData::quarry`. `ally_plan`'s Hunt arm asks
  `World::quarry_sensed` first: the quarry by `senses` (sight, then the nose), so it goes for that
  creature before a nearer one. The hunt's cell follows it **where the player sees that cell**:
  the dumps print the order's cell (`allies`, `companion`), and hidden information cannot leak, so
  an unseen quarry's cell stays the last the player saw. (A ranger of this level has Hunter's bond,
  so they see what the beast senses and the gate rarely bites.) Lost, it is the ordinary hunt: what
  is near the last cell, then "looks back at you". `set_order` clears the quarry, so any order, and
  the hunt's own end, end it.
- **Savage** (`data/status/`): degrees 3, `refresh`, `eases` 60, `cost` 67 (half again as fast;
  allies never disturb a rest, so `World::out_of_view_for` stays exact), `auto_critical`
  (`StatusDef::auto_critical`). `World::savage_blow`, a hook in `fight.rs::resolve` after
  `pack_blow`, sets `Profile::sure_critical` while the attacker suffers one, the read
  `combat::strike_with` has for an exploit of five at 60: no confirmation roll, and a target with no vitals
  still takes a plain hit. `World::savage_spent`, after the strike, spends a degree for each blow
  that hits (`Strike::is_hit`: soaked or resisted too), the status gone at none.

**Hunter's bond** never carries Savage to the ranger: its door (`bond.rs::is_buff`) passes only
what a buff spell's circle row or a potion lends, and Savage is neither (nor does `circles.toml`
list it). It reads only on the companion's blows and is the price of Focus.

Tests: `world/sic_tests.rs` (the table's row and data; the set-on, its focus and time; the
refusals; a neutral creature; the menu; every hit a critical and a degree, gone at three; speed;
easing by the minute; the quarry before a nearer foe, the cell following only where seen, the nose
through a door; a new order and a dead quarry; no crossing; a save and a replay).

## What the player sees

- **Messages**: above; the start's "Snel, your hound, pads at your heel.", growth's "Snel grows
  stronger.", the nose's growl, Sic 'em's "Snel goes for the kobold's throat."
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
  `pos` (seen, else null), `retinue`, `wears`; null without one); the `allies` JSON's `source` is
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

Taming's are `world/tame_tests.rs`. `world/companion_tests.rs`: the draw over every shipped race and 400 seeds (its list or the
rares, the rare odds, determinism, the names); a kitted ranger's beast (named, whole, for good,
on the player's side, said, its `only` line told, the dumps) and a warrior's none; the
scenarios' rangers (Maren's beside her, Osric's in the retinue), a restored run making no
second; growth at level 4 and far up the curve (every hook against an ungraded twin, the news);
the body rule; a companion coming along from across the level while a summon is left behind;
the retinue in and out (and a replay of the road); the nose through a closed door that stops the
eyes (the lynx without one; beyond its reach, nothing), the growl once; the words (the orders,
the kill, the dumps, the mark); a save round trip. `content/companion.rs::tests`: the shipped
file and every race's list, the refusals, loading without the file, the wolf's `tameable` and
`nose`. `background_tests` pins the
chapter list; the samples (`the_samples_read_as_written`) prove a non-ranger's story unchanged.
Screen snapshots (`crates/warlock/src/ui/screen_snapshots.rs`): every ranger case shows the
status bar's segment, the Character screen's row, and the Allies group.

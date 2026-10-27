# Animated objects

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08b-creatures.md ("Animated objects"), the approved design in
`docs/design/animated-objects.md` (its §15 cuts it into issues A1 to A6). Keep it current with
the code, as CLAUDE.md says.

A1 (#483) is the household tier: brooms, stools, candlesticks, teapots, the rug of smothering;
A2 (#484) breaks what does not fall whole into its parts; A3 (#485) adds families and the first
of them, blades. Hollow armour and panoplies (A4, #486), their homes (A5, #487), and the
chromatic look of composites (A6, #463) come after.

## Generated creature types (`content/animated.rs`)

An animated thing is an **ordinary creature type**, generated when content loads: every rule
reads it as it reads a kobold, and the guideline and roster tests cover it with no change.

- **Data**: `data/animated.toml` (`animated::RULES`: `[drops]`, `[creature]` (faction, tags,
  senses, `immune`), the material `[[group]]`s, the `[[tier]]`s) and `data/animated/<id>.toml`,
  one household body each (`BodySpec`; `README.toml` there lists the fields). Read by
  `Animated::parse` (`Content::animated`), after the objects (`load_objects` now runs before
  `load_creatures`) and before the creatures. Without the rules file there are none, and a body
  file without it is an error. Bodies are shared: a campaign's `data/animated/` is refused.
- **Checks**: a body's object exists and is of class `household`; its level lies within its
  tier's `levels`; `count` only on a stackable object; `plural` only with `name`; its light
  exists. Every other check is the creature table's own, since the generated spec goes through
  `load_creatures` like a file's.
- **Generation** (`Animated::specs`): one `CreatureSpec` per body, `animated_<id>` (`ID_PREFIX`),
  built as a TOML table and deserialized, so a generated type is checked exactly as a written
  one. Its name is "animated " and the object's (or the body's `name`), glyph and colour the
  object's, weight 1 g (it weighs what its body weighs, which it carries), size and footsteps
  the tier's, faction, tags, senses, and `immune` the `[creature]` table's, `corpse = false`,
  `blood = "none"`. Numbers by `Numbers::at(level, tier)`: the undead roster's formulas (#383),
  rounded down part by part, in integers (`hp_factor` to a thousandth); the dice from `LADDER`
  (the roster's, with 1d2 below it). Natural Protection (a hide named by the material),
  resistances, and the death verb from the body object's **base** material's group; a drawn
  variant changes none of them. XP by the guideline.
- **Order and the fingerprint**: `load_creatures` splices the generated specs in after the last
  shared `data/creatures/` file and before the first campaign's, in body path order, each named
  by its body's file (errors name it). A generated id a creature file uses is the duplicate-id
  error. Generation draws nothing, so the ids and `Content::fingerprint_for` are the same on
  every load; adding a body shifts every later id, as adding a creature file does (saves change:
  the integrator's format bump).
- **Families** (A3): a tier with `bodies = false` generates one type per level of its
  `levels`, `animated_<tier>_<level>` (`Animated::member`), least first, after the bodies'
  types (tiers in file order). Its name, glyph, colour, rarity, speed and Defence by level
  (`Ramp`: nothing below `from`, else `base + per_level × (L − from)` floored, at most `most`;
  a hair over before the floor so 0.1 × 30 is 3), resistances, and death verb are the tier's;
  no attack (it wields). What every generated type shares is `Animated::common`. The tier's
  slot lines are resolved at parse (`Animated::lines`, by tier; slot and classes must exist,
  `shown_by` must be one of the lines' slots). `Tier::family` is its id, `animated_<tier>`;
  `Content::name_families` registers it in `creature_ids` for the first member (a creature
  file of that id is an error), so populations, quests, and legends can name it.
- **`AnimatedDef`** on `CreatureDef::animated` (`#[serde(skip)]` on the spec): its tier, its
  family (the first type of its run, set at numbering as `id − member`; a household type is its
  own), `member` (its place in the run), and its body (`None` in a family;
  `Content::animated_body(creature)`).

Two creature fields came with it, for any creature: `immune` (status ids; `diseased` is every
disease) and `dies` (the death line's verb, `CreatureDef::death_verb`).

## Composition (`animated.rs`)

`animated::compose(content, rng, creature)` makes a household creature's body: one draw for a
body of several pieces (`count`, the cutlery's 3 to 6), then its material by `variant::draw` with
`Source::Gear(level)` (the materials' `gear` weights: a silver candlestick is rare).
`animated::with_body` puts it first in the pack (last if something is worn or wielded, whose
places must not move), in no slot, unless the creature carries its body already.

- A generated level: `animated::arm(template, content, seed)` after `gear::arm` (both generators,
  `mapgen/mod.rs` and `mapgen/local/mod.rs`), from `animated/<level id>` in spawn order.
- Every other spawn (a legend, the danger spawner, the dev `spawn`): `travel::make_creature`,
  after the gear, from `animated/<actor id>`.
- A legend's `animated = { material = "silver" }` (`content/level.rs::animated_body`) gives it
  its body at the least count of that material (else its object's own), a fixed kit: nothing is
  drawn.

A family's creature: `animated::compose_slots` goes through its tier's lines in order: one
draw for `chance`; one for the base item (`loot::base_in`: the loot objects of the line's
classes with `level ≤ L` and a `frequency`, by frequency, ordered by object id); one `below(100)`
for the grade (under `rare` rare, under `max(rare, magic)` at least magic, else plain); then
`loot::make_as` at item level L with `Source::Gear(L)` (or, plain, one `variant::draw` for the
material), put in the pack and in the line's slot. `animated::kit` is the one entry (household
body or family pieces; a family creature that carries anything already is not rolled), called
by `arm` and `travel::make_creature`. `animated::weapon_skills` (no draws) gives it the tier's
`weapon_rank` at L in the skill of each wielded weapon (`WeaponSkills::from_ranks`), set as
`Actor::weapon_skills` in `make_creature`.

No new save state: the body and the pieces are ordinary inventory stacks, the ranks ordinary
weapon skills.

## Families: the four reads

- **The danger draw**: `Content::candidates` gives a family one `danger::Candidate`, `(first id,
  (from, to), factor)` (a member's id in a population stands for its family); `danger::pick`
  weighs it at `clamp(D, from, to)` and returns `danger::member`: `first + clamp − from`.
- **First kills**: `World::earn_kill` keys `discovered.kills` by `Content::family_key` (the
  family's id, else the type's own).
- **Kills and quests**: `World::tally_kill` tallies by `family_key` (so Favoured prey,
  `ambush.rs`, compares `family_key` too); `KillTarget::has` compares `Content::family`;
  `QuestDef::members` (a `tag:` objective) and a rolled pick of creatures list a family once
  (`content.family(c) == c`), the pick's target the family's id; `Content::lives_in` asks for
  the family.
- **The wiki**: `order::creatures`, the experience table, and the tag table show a family once
  (its first member's row); `animated.rs` prints its per-level numbers and odds.

## In play (`world/animated.rs`)

- **Drops** (`World::remains`, called by `creature_dies` for what a creature leaves): an
  animated creature's body falls whole by `[drops] intact` plus the player's
  `TalentMods::salvage_percent` (Salvager), one draw on `drop/<actor id>` (its `loot` is 0, so
  `loot_of` draws nothing there); else it breaks into its parts, `object::break_up` at
  `[drops] pieces` percent (loose gems, pieces of its grouped materials, nothing of paper; no
  magic; `docs/dev/objects.md` "Pieces and breaking"). One draw per body stack, in inventory
  order, so the stream is A1's. Anything else it carried falls as anyone's does.
- **The break line** (`World::break_lines`): "The silver candlestick breaks apart." ("The 4
  clatters of cutlery break apart."), one per broken stack, only when the dead creature was on
  the current level at a cell whose actors the player's perception saw. `creature_dies` returns
  the lines and `World::dies` splices them into `news` where they stood before the kill's
  earnings, so they come right after the death line (said by each death path) and before "You
  learn something of ..." and level-ups.
- **Immunities**: `World::afflict` refuses a status in the type's `immune` before anything else;
  `World::expose` refuses a disease when `immune` holds the diseases' status, nothing rolled.
- **Stealing**: `World::steal` refuses an animated creature before the hostile check: "It is all
  of a piece.", no time.
- **Unbinder**: `TalentEffect::Bane { tag, to_hit, damage }` sums into `TalentMods::banes`;
  `World::bane` adds them to an attack's profile against a creature whose type carries the tag,
  beside Favoured prey in `World::resolve`.
- **Death line**: `fight::death_message(end, dies)`, its callers passing the dead type's
  `death_verb`.

## Presentation

A composite's colours (#463): `Shape::bands()` lists the distinct `Stack::fg` colours of what
it wears and wields, in slot order (helm, body armour, hands, feet), at most four; a one-item
thing has none and shows `fg` alone (still the `shown_by` piece's colour). The map draws them as
horizontal bands (`docs/dev/rendering.md`, "Colour bands"), and a JSON dump's seen actor carries
them as `bands`, top first.

`SeenActor::shape` (`perception.rs::shape`): an animated creature's body's glyph, its colour
(`Stack::fg`: the drawn material's), and its main material. `SeenActor::glyph`/`fg` read it,
else the type's; the map (`ui/glyphs.rs`), the look cursor, and the `view` dump use them, and a
JSON dump's seen actors carry `glyph` and `fg`. Look mode (`ui/look.rs::made_of`) adds the
material after the name and the body's `moves` after how hurt it looks: "An animated
candlestick of silver (hostile, unhurt), rocking, on stone floor." A family's creature is
shaped by the piece in its tier's `shown_by` slot: its colour, and with `item_glyph` its glyph
(a blade shows as its weapon, `)`); without that piece, the type's own. Look mode adds no
material word for it: the gear line names what it wields ("An animated blade (hostile,
unhurt), wielding an iron great sword, on stone floor.").

## Content

The household objects are `data/objects/<body>.toml`, class `household` (the general store buys
them). Populations: Cealdhlāw (candlestick, cutlery, teapot, cooking pot), Fennmynster
(candlestick, book, lantern, rug), and `animated_blade` in Cealdhlāw and Fyrdelf; the stray list
follows. Talents: system `animated` with
Salvager (delver) and Unbinder (weaponcraft).

## Tests

`content/animated_tests.rs` (ids, order, numbers, group effects, identity, every load the same,
collisions and bad bodies), `world/animated_tests.rs` (the legend's body, composition and its
stream, material weights, the cutlery's count, immunities, stealing, the drop rule and
Salvager, broken bodies as their parts with the break line, a loose gem and a book's leather,
Unbinder, saves and a replay), `world/animated_blade_tests.rs` (the family's ids and numbers, a
blade's weapon, rank, stream, odds, and fight, first kills, quests, and the tag objective
counting the family once, saves), `content/danger.rs` (a family candidate's weight and member,
out of depth too), `ui/look.rs` (the look lines, a blade's as an insta snapshot). The wiki's
tables are `xtask/src/wiki/animated.rs`.

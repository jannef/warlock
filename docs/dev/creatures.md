# Creature subsystem

**Rule: a roster PR adds or edits ONLY its own `docs/dev/creatures/<tribe>.md`; it never edits
this file.** This file is the stable overview of the creature subsystem (factions, the AI,
sight, spawning, gear, incorporeal creatures, coarse turns); every tribe's or wave's roster
notes, per-creature facts, and test-row lists live in the per-tribe files below. The spec's
creature and faction lists are likewise not edited by a roster (`spec/07b-data.md` points at
`assets/data/creatures/` and `assets/data/factions.toml`). A new tribe adds a new file and one
line here only if it truly is a new family, in the same PR as nothing else shares.

Per-tribe notes:
- `docs/dev/creatures/undead.md`: the undead roster (#383; batches, population lines, holds, drains)
- `docs/dev/creatures/campaign-waves.md`: the barrow, fen, and delf waves (the non-undead notes)
- `docs/dev/creatures/duergar.md`: the duergar (#518; the delf wave's three and the roster to level 40, formulas, Fyrdelf)
- `docs/dev/creatures/yugoloth.md`: the yugoloths (#446; faction, tags, riders)
- `docs/dev/creatures/hags.md`: the hags (#451; faction, tag, riders)
- `docs/dev/creatures/mimics.md`: the mimics (#616; the generated family, the disguise in perception, revealing, Chest-wise)
- `docs/dev/creatures/hobgoblins.md`: the hobgoblins (#454; the goblin tribe's second tier)
- `docs/dev/creatures/fey.md`: the fey (#452; the courts, `provoked`, the roster)
- `docs/dev/creatures/orcs.md`: the orc tribe and its wargs (#455; faction, tags, formulas, populations)
- `docs/dev/creatures/outlaws.md`: the outlaws (#424; faction, tag, the four criminals and their uniques, proper names)
- `docs/dev/creatures/demons.md`: the demons (#449; faction, tags, riders, attack costs)
- `docs/dev/creatures/trolls.md`: the troll tribe (#456; faction, tag, formulas, attack cost, population)
- `docs/dev/creatures/layer-bosses.md`: the layer bosses of Grundlēas (#1118; faction, the five bosses and their lairs)
- `docs/dev/creatures/second-wave.md`: the dread design's second wave (#1138; the black pudding, cockatrice, hydra and its heads, rust monster, gorgon)
- `docs/dev/creatures/summoners.md`: the summoners (#520; the `summoner` tag, the three types; the mechanism is `docs/dev/summoning.md`)

## Overview

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08b-creatures.md. Keep it current with the code, as CLAUDE.md says.

Stair followers (spec §8 "Stair followers", #985) live in `world/stair_followers.rs`, whose
module docs are the rules: `take_stairs` asks `stair_followers` before `travel` (awareness reads
the level being left) and `followers_arrive` after. A follower crosses through
`travel.rs::cross_to`, the allies' way, and keeps its `Mind` with `last_seen` set to the player's
arrival and `post`, `noise`, and `patrol` cleared, since those name the old level's cells. What
reads a creature's old place: `Actor::home` (the level it first left) is what `fight.rs::dies`
tallies a kill under, so `level`-scoped kill quests stay fair; `summon.rs::live_summons` counts a
summoner's summons on its own level only; a hold (`hold.rs`) ends by itself if the holder stayed
behind and goes on if it came. A den's sleepers don't follow until woken, a shop's keeper never;
`hostiles_here` reads the level, so the spawn cap moves with the follower. Hoards, curses, and
hauntings keep no ids, so losing a guardian to the stairs changes nothing there.

Auras, innate statuses, and cries (#1128, design `docs/design/monster-dread.md` §5.4, §9) live in
`world/aura.rs`, whose module docs are the rules, with the data in `content/aura.rs`
(`[creature.aura]`, `innate`, `cry`; `CreatureDef::{aura, innate, cry}`). Two lines in
`ai.rs::creature_turn`: `aura_turn` as the turn starts (after the slumber and coarse-turn
returns, before `decide`; it puts the innate statuses back first) and `cry` beside `mutter` once
the plan is done; `travel.rs::make_creature` calls `aura::wear_innate`, and `perception.rs`
leaves a type's innate statuses out of the seen effects. The aura's cooldown is the `aura`
status; its draws (a check's two per target) and the cry's one are the gameplay stream's, on
the bearer's turn. `content::aura_roster_tests` holds the bearers' rows; the tribe files say
whose is whose.

Grades (spec §8 "Grades", #1135; `docs/design/monster-dread.md` §7.4) live in `grade.rs`, whose
module docs are the rules: `grade::draw` is one line in `travel::make_creature`, after the court,
on its own stream `grade/<actor id>` (`Rng::grade`), so adding or removing it moves no other
draw. Per type: `grades` (default true; `CreatureSpec::resolve` forces it false for a unique, a
mimic, and a generated type; the people's files, the peaceful and merchants factions, say
`grades = false`) and `grade_words` (resolved at load: the file's, else `grade::TAG_WORDS` by tag,
else `grade::DEFAULT_WORDS`). A spawn with an `npc` is never graded; `Actor::graded` is 0 for a
kept beast. Perception copies it to `SeenActor::grade`; look mode puts the word before the name,
the `actors` dump says `; grade WORD`, the JSON `grade`. The tests are `grade_tests.rs` (the
draw's odds and determinism, the reads, the roster of the ungraded, saves and hash, the dumps).

The unseen (#1132; `docs/design/monster-dread.md` §5.10, §5.11) live in `world/unseen.rs`, whose
module docs are the rules, and `content/unseen.rs` (the data). Per type: `shadowy` (unseen on a
cell dim to the player's eyes, `VisionProfile::dim`: under twice its threshold, darkvision or
not, unless beside them), `invisible` (`invisible = true`, innate: the invisible stalker; or
`[creature.invisible] cooldown = 300`, resolved to `Invisible::Fades` with the `invisible` status
and the `invisibility` cooldown, a degree per 6 s as a breath's), and `hunts` (with no hostile in
sight it goes to the player's cell as to a noise: `World::hunted`, one line in `ai.rs::decide`).
The status flags are `unseen` (Invisible: `Effects::unseen`, read by `Actor::invisible`, and by
`light/cache.rs` so a creature's own light goes out with it) and `sees_unseen` (Seeing: See
invisible's and the potion of seeing's). **Perception**: `Perception::compute` asks
`world::unseen_by_player` once its light is final, with the dim test and Blood scent's finds; the
answer (`Concealed`, by id and cell) is left out of the actors, `sees_actors_at` is false on
their cells, and `hides` (and the sound pass) counts their cells as hidden, so what they make is
heard there (the `x` of an unseen blow at its cell, footsteps, a heartbeat to Quick-sensed). What
shows them: Seeing, Blood scent, the player's companion's `nose` (`smelt_within`), and
`Mind::glimpsed`, the turn a blow of the player's found one (shown that turn and the one its
perception computes after). `perceive.rs` says "You feel a cold draught." as an unseen shadowy
creature first comes within 2 (`Perception::draught`), and names no unseen attacker
(`conceals`). **The self-buff step**: `World::fades` is one line in `ai.rs::creature_turn`,
before `decide`: a fading type that holds neither status and is not blocked from casting gives
itself both, free of time and mana, heard as chanting and said where seen ("The duergar warrior
fades from sight."), and then takes its turn as ever; `World::unveiled` (from `buff.rs::ended`)
says "You can see the duergar warrior again." where the player sees the cell. **Striking**:
`World::at_the_unseen` (one line in `fight.rs::resolve`) puts `UNSEEN_TO_HIT` (−4) on the
player's blow at a concealed creature, `World::glimpse` marks a hit, and `travel.rs::strike_at`
says "You strike at something." first. Creature sight is unchanged (creatures see the unseen;
the player's own Invisible, from the dev `afflict:invisible`, hides them from no one yet). The
tests are `unseen_tests.rs`.

Factions and creature behavior (spec §7 "Factions", §5 "Creature sight", §8 "Creature behavior"):
`data/factions.toml` (`content/faction.rs`: `Factions`, symmetric hostility, `Stance`; optional,
without it everyone is neutral, but then no creature may name a faction; monsters fight only the
player but for the two story feuds, Seelie and wrathful fey against the Unseelie and demons
against devils, #612, the tribes' old feuds kept as comments; a level that wants a brawl gives
one side a faction of its own by a legend override, as the arena's rats are `arena_rats`) and per creature type
`faction`, `vision`, `sight`, `combat` (`melee`/`ranged`/`caster`/`dragon`), `range`, `opens_doors`, `lurks`
(unharmed, it waits where it stands for a hostile it sees to come beside it; `world/lurk.rs`, one
branch on `ai.rs::fight`'s plan; the meenlock and the barrow spider),
`alignment` and `race` (`docs/dev/alignment.md`; read a creature's with `Actor::nature`, never the
type's) (`content/creature.rs`, checked in `content/mod.rs`). Its `tags` (`content/tag.rs`, #441) are plain singular
words for quests and rules (`tag:goblin`, `docs/dev/quests.md` "Creature tags"), never the
faction: **tag every new creature** of a group or role listed in the README (an undead of the
`undead` faction must carry `undead`: `content::tag`'s test). Creature types live in `data/creatures/`,
one file per creature named by its id (`kobold.toml`; the field documentation is in its
`README.toml`), a table directory read in path order: see "Content directories" in
`docs/dev/content.md`. Stance comes from types:
`Content::stance(creature, creature)`, `World::stance(actor, actor)`, and for presentation only
`PerceptionView::stance(seen actor)` (look mode's "(hostile)"). Between the player and a creature,
the Beast-friend talent makes `animal` factions neutral until the player attacks one of the type
(`world/beast_friend.rs::befriended`, read by `World::stance`, the AI's `nearest_hostile_in_sight`,
and `PerceptionView::stance`; `docs/dev/progression.md`). The AI is `world/ai.rs`
(look → fight → search the last seen spot → look into the newest unseen noise → back to the post →
patrol), paths `world/path.rs` (A*, fixed tie-breaks), and per-creature memory `Mind` on `Actor`
(saved, hashed; its `noticed` is the player's half of awareness, set by perception, not the AI:
docs/dev/surprise.md; its `missed`, the turn the player slipped past it, is set at the look by
`slip::looks_at`, the AI's one random draw, docs/dev/surprise.md). Creature sight is `sight.rs`: the player's rules computed per creature, only for
cells where hostiles stand (nearest first, line of sight only out to that distance, darkvision
before light, bounce light only when needed); it must stay equal to the player's perception —
`light_tests::check_creature_sight` compares `sight::light` with the perceived light on every
visible cell in the light property tests (and under each weather in `weather_tests`). Both take
the sky from `WorldState::sky` (the weather's clouds included) and overworld sight from
`daylight::sight_under` (rain included). A creature's attack is `World::attack` (`docs/dev/combat.md`):
`Event::Attacked` (kept only when the player sees both ends), a noise (`noise::Act::Attack`), and
messages built in `World::perceive_events` (unseen ends are "something"). The app shows a lunge
(`ui/motion.rs`, `motion` switch), a shot glyph, and a creature's `on = "attack"` particle burst
(`ui/effects.rs`, `particles` switch); `Event::Died` has no effect of its own. No creature
behavior may use randomness (combat rolls are the rules', from the gameplay stream); tie-breaks
are by distance, direction order, and actor id.

Dragons and several natural attacks (spec §8 "Creature behavior": "Dragons", "Several natural
attacks", #1099). **Data**: `Combat::Dragon` (`combat = "dragon"`) and `[creature.breath]`
(`content/breath.rs`: `BreathSpec` resolved in the creature loop by `BreathSpec::resolve` into
`CreatureDef::breath: Option<Breath>`, the damage type and the `breath` status's degrees,
`ceil(cooldown / eases)`; refused with an unknown type, without the status, or with a cooldown
the status cannot hold; `check_combat` refuses a breath on any type but a dragon (it would never
be read); a dragon without one has no breath to use;
`breath::check_effects` runs once effects are loaded: a burst effect; its optional `riders`,
`on_hit` entries resolved as a gaze's, #1138). Natural attacks are one
table or a list (`creature::NaturalAttacks`, its own `Deserialize`: a table or an array of
tables, so parse errors stay the field's): `CreatureDef::attack` is the first,
`CreatureDef::more_attacks` the rest, `CreatureDef::natural_attacks()` all in order. **Rules**:
`world/breath.rs` (`World::breathes_at`, one branch in `ai.rs::fight` after the summoning and
the throw, `Plan::Breathe(at)`; `World::creature_breathes`: the status, `noise::Act::Roar`, the
words, `World::cast_cone` in `spell.rs` (the one place a cone strikes, shared with Burning
hands; reuse it for any creature's cone), `say_spell`, `spell_type_lands`, then each rider by
`World::rider` on each target harmed and alive: the gorgon's stiffening); the cooldown is a
status in the actor's `effects`, saved as every status is. `world/natural.rs`
(`World::more_natural_blows`, one line in `fight.rs::attack` after the second blow:
`Actor::more_natural_blows` gives each later attack's profile, built by `Actor::melee_with`;
each is `World::attack_with`, and its `Event::Attacked` gets `natural: Some(index)`, which
`perceive.rs` says with that attack's words). A natural attack's `on_hit` now rides on its
blow's profile (`Actor::melee_with` sets `Profile::on_hit` first, before the affixes'), so
`status.rs::on_hit_of` reads only throws. **Wiki**: `creature_attacks` lists a row per natural
attack, `creature_breaths` (`creature-breaths.md`, "Breath" in `creature-numbers.md`); the
roar in the hearing table. **Probe**: the monsters' sides sum every natural attack's damage
(`balance_probe.rs::Side::more`), and `balance_probe/dragon.rs` measures the fire drake against
the players of its level (docs/balance.md "Dragons"). **Tests**: `world/breath_tests.rs`.
Adding a dragon: `combat = "dragon"`, a `[creature.breath]` of its element with a burst effect
of that element in `data/effects.toml`, `[[creature.attack]]`s, and the tag `dragon`.

Warning signs (#1133, design `docs/design/monster-dread.md` §9 channels 3 and 4). **Data**:
`leavings = ["statue", "web", "acid_pit", "torn_cloth"]` (`content/leaving.rs`: `Leaving`, each
laid as a fixed terrain or object, `Content::leaving`; the loader refuses a kind the content
lacks) and `first_sight = "A basilisk. Do not meet its eyes."` on a type. **Rules**: the
generator lays leavings (`mapgen/leavings.rs`, docs/dev/mapgen.md); `world/first_sight.rs`
(`World::first_sight`, the last line of `perceive.rs::perceive`) says a type's line once a run
when one is first among the perception's actors, remembering the type's id in
`Progress::sighted` (saved). **Content**: every dread type has a `first_sight` and nothing else
does (`dread_roster_tests.rs`); statues on the basilisk, webs on the barrow and large spiders,
acid pits on the hezrou and ember beetle, torn cloth on the imp and nymph. **Dumps**: `truth`
`leavings web: (12, 5), ...` a line per kind; JSON `leavings` (`pos`, `kind`) the ones the player
sees or remembers (`PerceptionView::known_leavings`). **Wiki**: `creature_signs`
(`creature-signs.md`, "Warning signs" in `creature-numbers.md`). **Tests**:
`mapgen/leavings_tests.rs`, `world/first_sight_tests.rs`, `ui/look.rs`.

Gazes and averted eyes (spec §8 "Creature behavior": "Gazes", #1125, design
`docs/design/monster-dread.md` §5.1). **Data**: `[creature.gaze]` (`content/gaze.rs`:
`GazeSpec` resolved in the creature loop into `CreatureDef::gaze: Option<Gaze>`; `range` 1 to 8,
`chance` (default 100), `cooldown` 6 to 600 s (default 12, the `gaze` status's
`ceil(cooldown / eases)` degrees), `averted`, `glance` (only with `averted`, default 25), and
`riders`, `on_hit` entries resolved by `OnHitSpec::resolve`, a status or a drain only).
**Rules**: `world/gaze.rs` (`World::gazes_at`, one branch in `ai.rs::fight` after the summoning
and before the throw, `Plan::Gaze(target)`: in range, the cooldown off, the target with vitals
(its type's `vitals` and `Content::has_vitals`); `World::creature_gazes`: the cooldown status,
a `cast`'s price, then the eyes must meet: the player's by `Perception::sees_actors_at` on the
gazer's cell (the snapshot's sight, no disguise), a creature's by `World::sees_now`; unmet, the
player is told "You feel a cold stare on you."; met, one draw at `Gaze::chance_against` (below
100 only) and each rider by `World::rider` in `status.rs`, the one place a blow's riders fall
too). No noise. `world/avert.rs`: Averted is `Perception::averted` (a seen hostile whose gaze is
`averted`), derived in `Perception::compute`, never saved and out of the hash; read by
`World::averted_blow` in `fight.rs::resolve` (`AVERTED_TO_HIT`, −4), `World::averted_defence`
in `fight.rs::defence_of` (a positive Dodge halved), and `free_blow.rs::free_blows_of` (none at
all); `World::avert_news` notes it beginning and ending. Status bar "Averted"; dump header
`averted`, JSON `averted`; the cooldown shows among `actors`' effects. Nothing is `averted` until
D2's basilisk. **Tests**: `world/gaze_tests.rs`, and `content/gaze.rs`'s roster of the moved
riders (`MOVED`). Keep the rider generic: D2's stiffening is a status rider like any.
Bursts: vortices and death throes (the dread design `docs/design/monster-dread.md` §5.3, #1127).
**Data**: `[creature.burst]` (`content/burst.rs`: `BurstSpec` resolved in the creature loop into
`CreatureDef::burst: Option<CreatureBurst>`: `damage`, `damage_type`, `radius`, `when`
(`BurstWhen::Touches`, a vortex; `Dies`, death throes), `effect` (a burst effect, checked by
`burst::check_effects` once effects load), optional `noun`/`verb`, `rider` (the spell types'
rider form, `spell_type::rider`), `douses`, `gust`); `check_combat` refuses a `touches` burst on
a type with an `attack` or a `combat` other than melee (the burst is struck instead of every
melee blow, so the attack would never be). **Rules**: `world/creature_burst.rs`:
`ai.rs::creature_turn` sends every melee plan of a vortex (`World::bursts_on_touch`) to
`World::creature_bursts` (a melee attack's price; the burst, then `dies` with no killer);
`fight.rs::dies` calls `World::death_throes` after `creature_dies` (current level only; the
words held and joined to the news, so they follow the death line). `World::go_off` is the
burst: the words, the traps' `explosion` sound (`sound_at`), and
`spell.rs::cast_burst_reaching` with `Reach { spare: its type, full: true }` (no halving at the
edge; `cast_burst_at` is the same with the usual `Reach`), so the ruin, the spell type's cells,
and `spell_strikes` are the spells'; then the wave (`quench` on everyone on its cells,
`spell_type.rs::douse`), the gust (piles moved a cell outward), `say_spell`, and the riders by
their chance on those harmed (no `spell_type_lands`). A dead burster's blows are its own: it is
in `fallen`. **Wiki**: `creature_bursts` (`creature-bursts.md`, "Bursts" in
`creature-numbers.md`), the creatures table's "bursts beside you" / "bursts when it dies", the
attacks table's vortex row. **Tests**: `world/creature_burst_tests.rs`; the elementals' roster
test pins every line's burst and the earth line's throw.
Theft and the blink (docs/design/monster-dread.md §5.6, #1130). **Data**: `steals = { what,
chance, cooldown }` and `blinks = { range, cooldown }` on a type (`content/theft.rs`:
`StealsSpec` and `BlinksSpec`, resolved in the creature loop into `CreatureDef::steals:
Option<Steals>` and `CreatureDef::blinks: Option<Blinks>` with their cooldown statuses' degrees,
`theft` and `blink` in `data/status.toml`, a degree per 6 s; `what` is `small`, the classes of
`theft::SMALL`, or `any`). **Rules**: `world/snatch.rs` (`World::snatches_at`, a guard in `ai.rs::fight` before a cast: a thief
beside the player, its cooldown off, strikes in melee; `World::snatch`, one line in `fight.rs`
after `barbed`, so a thief cut down by barbs takes nothing): a melee hit on the
player that leaves them alive, the cooldown off, one `chance` draw, then one stack of the loose
ones its `what` allows, `index(n)` with two or more (the Steal ability's pick); the stack into
the thief's pack, the cooldown on it; it fights on, and its pack falls where it dies
(`fight.rs::dies`). With nothing to take, no cooldown. `world/blink.rs` (`World::blinks_at`, a
guard in `ai.rs::fight`'s `close_in` on `Route::Wait | Route::Unreachable`, `Plan::Blink(at)`;
`World::creature_blinks`): a cell beside the target within `range` of the blinker and in its
line of sight that the teleport trap would land on (`trap.rs::may_land`), not shunned, drawn
with `index(n)` in row order; `teleport_to`, the cooldown, a step's price. The two draws come
after the AI decided, on the gameplay stream. **Words**: "The pixie snatches 40 gold coins
from you!" ("Something" unseen); "The blink dog is suddenly beside you." where the player sees
it land, nothing unseen. **Wiki**: `creature_thieves` and `creature_blinks`
(`creature-thieves.md`, `creature-blinks.md`, "Thieves" and "Blinkers" in
`creature-numbers.md`). **Tests**: `world/theft_tests.rs`. No talent (design §11).
The disarming blow (#1166). **Data**: `disarms = { chance, difficulty }` on a type
(`CreatureSpec::disarms`, `CreatureDef::disarms: Option<Disarms>`, `content/creature.rs`).
**Rules**:
`world/knock_away.rs` (`World::knock_away`, one line in `fight.rs` after `snatch`): a melee hit
on the player that leaves them alive, a weapon in the first hand slot; one `chance` draw, then
the open cells beside the player in row order (`is_walkable`, not `swim`, not a closed door, no
actor; none: nothing more drawn), then the Dexterity check (`check::check`, two draws) with
`knock_away::hold` (Dexterity modifier and talent checks, the weapon's skill rank, the grip:
`TWO_HANDED_HOLD`, `SHIELD_HOLD`, or the `OneWeapon` style's rank), then `index(n)` for the cell
with two or more. The stack leaves the pack (`after_removal`) onto the cell's pile; a spirit
stack is taken off and `fade_spirit` says it goes. No noise of its own (the fight's is heard).
**Words**: "The ultroloth knocks your long sword from your hand!", "It lands beside you.".
**Wiki**: `creature_disarmers` (`creature-disarmers.md`, "Disarmers" in `creature-numbers.md`).
**Tests**: `world/knock_away_tests.rs` (the roster pinned there). No talent (a gimmick).

Casters (spec §8 "Creature behavior": "Casters", #1102). **Data**: `spells = [...]` on a
`combat = "caster"` type (`CreatureSpec::spells`, resolved in the creature loop by
`content/creature_spells.rs::resolve` into `CreatureDef::spells: Vec<SpellId>`, in preference
order; spells load before creatures). Refused: a caster without a list, a list on any other
type (never read), an unknown id, and a spell no creature casts (`creature_spells::castable`:
zones, corpses, the open sky, pushes, charms, ground- or tag-bound hexes, heals not by touch,
summons, cleansings, utilities). A caster's `mana` is its pool (summoners' summons share it).
**Rules**: `world/creature_cast.rs`: `World::creature_spell` (one branch in `ai.rs::fight`
after the breath, `Plan::Cast { spell, on }`), `World::spell_reach_at` (how far the AI closes
in; `None` sends it to a launcher or to melee: never idle), `World::creature_casts` (mana
through `World::spend_mana`, a spell's noise, the words, then by kind: a buff or a hex through
`World::afflict`, a heal, or harm through the shared `cast_burst_at`, `cast_bolt`, `cast_cone`,
and `cast.rs::cast_single`, which now takes its caster's id, then `spell_type_spreads` and
`spell_lands`). `Cast::by_creature` settles reach, roll, and degrees at the creature's stats
(`Pay::Creature`: nothing paid there, nothing practised);
`World::creature_aim` adds the level's quarter to an `attack` spell's to-hit. Spell reflection
(`reflect.rs`) turns a caster's rays and bolts back at it. **Wiki**: `creature_spells`
(`creature-spells.md`, "Spells" in `creature-numbers.md`). **Probe**:
`balance_probe/casters.rs` (docs/balance.md "Casters"). **Tests**:
`world/creature_cast_tests.rs`, the silent vault's in `silent_tests.rs`. Adding a caster:
`combat = "caster"`, a `range`, `spells` (up to level 10 only level-1 harm and level ≤ 3 hexes,
heals, and buffs, and `rarity = "very_rare"`), and `mana` for a
handful of its main spell.

Natural Protection by body and level (spec §8 "Natural Protection by body and level", #563,
`docs/design/combat-progression.md` §8): data only. Every type's `natural.protection` is the
body's at its level, the hide's `name` picking the body: none 0, hide (L − 10) ÷ 8, bony L ÷ 6,
armoured L ÷ 4 + 1. The shipped types were written to the rule, but for the hand-tuned (beetle,
ember beetle, stone troll, mud golem, balor, pit fiend), the peoples (who wear their Protection:
0), armed types (gear that may roll a piece with Protection or armour: 0), the hags who cast
(half a hide's), and the living trolls (a floor of 2); no test holds them to it (#1205: the data
author decides). Animated objects (tag `animated`) keep their own table, which their generator
computes. A new creature type: give it the hide its body asks at its level.

Incorporeal creatures (spec §8 "Incorporeal creatures", #381): the data is a creature type's
`incorporeal = { wall_defence }` (`content/creature.rs::Incorporeal`, read in `Content::load`
beside `natural`; `Content::incorporeal(type)`). The rules are `world/incorporeal.rs`:
`LevelState::phaseable` (solid terrain, neither walkable nor transparent, off the level's edge:
walls, rock, closed doors), `World::in_wall`, `World::wall_defence` (read by
`World::defence_of`: the Wall category, `combat::Category::Wall`), and `World::landing` (where
a dying creature's things fall: `creature_dies` moves the dead actor's cell there before its
pile and corpse). The hooks, one each: `ai.rs::route` (a solid cell costs `1 + WALL` and may be
a goal), `ai.rs::creature_steps` (through a solid cell, a closed door left shut; no `tread`, no
`step_onto_trap`), `travel.rs::player_moves` (a seen hostile inside a solid cell is struck at,
`World::strike_at`), and `shoot.rs` (a missile reaching an opaque cell attacks whoever stands
inside it). Sight is unchanged: the player's perception already sees a wall cell (and so an
actor in it) by its lit face or darkvision; `sight::sees` sees opaque cells only within
darkvision. The message for a miss the wall turned is `fight::attack_message`'s `Category::Wall`
arm. The example type is `stone_haunt` (placed by no level). Tests:
`world/incorporeal_tests.rs`, `shoot_tests::a_shade_in_a_wall_is_shot_there_...`,
`spawn_tests::an_incorporeal_creature_is_never_placed_inside_a_wall`.

Danger levels and spawning (spec §7 "Danger levels", #326): the data is `content/danger.rs`
(`DangerRule`, `Spawning`, `level_weight`, `weight` and `draw`, the field parsing shared by
`data/dungeons.toml` and level files), reached through `Content::spawning(level id)` (a hand-made level's own, else
its branch's at the depth; `None`: danger 0). The rules are `world/spawn.rs`: `World::danger`
(computed, never stored), `LevelState::spawner` (the next attempt's time and the stream
`spawn/<level id>`, `Rng::spawn`; made with a level that has a danger rule, else absent so other
levels hash as before), `spawn_tick` (one call at the end of `run_turn`: every attempt due on the
current level), `arrive_spawner` (one line in `travel`: frozen floors), `attempt` (the cap by `World::stance`, the chance, the draw, the
cell out of a freshly computed `Perception`'s sight and `AWAY` cells off, the route by
`mapgen::out_and_back`, then `travel::make_creature`, the one way a creature is made from a
`CreatureSpawn`), and the dev `spawn` action. A spawn bumps `World::spawned`, so a rest perceives
after that turn. Uniques are remembered in `Progress::uniques` by type id, in `make_creature`
(however one came: a spawn, a population, a level file). A spawn sets `rolls_gear`, so
`make_creature` rolls its gear from `gear/<actor id>`. The initial population of a generated
floor is `mapgen/population.rs` (`populate`, one call in `mapgen::generate_for` after
`place_loot` and before `gear::arm`; stream `population/<level id>`, `Rng::population`): the
count by walkable cells, the draw by `danger::draw`, a cell from the free plain-floor cells in
row order (`swap_remove` of a drawn index), a route by `out_and_back`. Whom a level is made for
is `mapgen::Visit` (the luck, the player's level for a scaled danger, the uniques met;
`travel::level_template` builds it; `generate_with` is `generate_for` with only the luck).
A vault's monster pools (#958, `content/monster_pool.rs`): `populate` draws them first, group by
group, from `Rng::vault_monsters` with the same `content.candidates` and `danger::draw`
(`MonsterPool::Level`: the floor's population and generic list; `MonsterPool::Tags`: the tag's
types of the floor's campaign as both lists, so a stray stays in the tribe and an out-of-depth
draw reaches deeper in it). Each draw is placed with the pool's `status`, walks its `route` if it
has one (a walk is drawn either way, so the stream is the same), and the first placed takes its
`carry` (`gear::arm` puts rolled gear first and keeps what a spawn was given after it). A
generated level holds its vaults' pools in `LevelTemplate::monsters` until they are drawn, so
`connect::validate` sees a pool's key; `population::leave_unborne` then lays a `carry` no draw
took on the pool's first cell. Their cells stand in `LevelTemplate::outside_cap`, which
`travel::instantiate_level` turns into `Spawner::outside` (actor ids): `World::hostiles_here`
leaves those out, so the spawner's cap and the dumps' `population 2/6` do not count them. Without
this a floor laid down full would see nothing come until as many had died as the vault holds, and
an avoided vault would stop the floor's respawning for good; a teeming den's sleepers, by contrast,
do count (spec §7 "Special rooms"). Tests: `mapgen/vault_monsters_tests.rs`,
`world::spawn_tests::a_vault_s_monster_pool_stands_outside_the_cap`.
Rarity (#373): `content/creature_rarity.rs` parses `data/creature_rarities.toml` (tiers in file
order, each a `factor` in percent of a common type's weight, 1 to 1000; `common` required, the
only tier without the file); `CreatureDef::rarity` is an index into `Content::creature_rarities`
(a type's `rarity`, default `common`; an unknown tier is a load error). A draw's candidates are
`danger::Candidate`s `(type, levels, factor, dread)`, built by `Content::candidates` (the run's met
uniques left out) for `World::draw_spawn` and `mapgen::population::populate`, and
`danger::weight(level, factor, danger)` is `level_weight × factor`.
Strays and exploding out-of-depth draws (#375): `danger::draw(rng, own, generic, danger, odds)`
rolls, in order, the stray chance (`odds.stray`: the pick goes to `generic`), the out-of-depth
chance, and each further step (`danger::deeper`: `D + 1`, then `odds.deeper` % again and again
one more, at most `odds.deeper_cap`), then one `below(total)` per pick that finds something
(`danger::pick`): at the deeper danger the chosen pool, else the generic list (the own folk all
too weak), then at `D` the chosen pool, else the own folk. `danger::Odds` is
`Spawning::odds`: each of `stray`, `out_of_depth`, `deeper`, `deeper_cap` as a branch or level
gives it, else `data/dungeons.toml`'s `[defaults]` (`danger::DefaultsFile`, read into
`Content::draw_odds` before the levels; a `danger` without it is a load error, and `[defaults]`
in any other dungeons file is refused). The generic creature list is derived
(the owner, 2026-10-27: anything the campaign knows may come, the exploding out-of-depth draw
bringing even the deepest to a first floor at astronomical odds): `Content::fill_generic`
(after `load_dungeons`) gives every `Spawning::generic` the union of the populations of the
areas with a danger of its campaign and of no campaign, uniques left out, and, unless the area's
`Spawning::pool` is `danger::StrayPool::Populations` (`stray_pool = "populations"`: the guild's
four lairs, whose folk are the design's table), every type of its campaign or of none that
`Content::may_stray`: hostile to the player (`Content::stance` with `player`), no unique, no
animated object (`CreatureDef::animated`: those come by family through the populations that
name them), and not marked `stray = false` (`CreatureDef::stray`, default true, for a type only
placed by hand or by its own rules; no shipped type uses it yet). So a roster type in no
population is met as a stray, and people, keepers, hand-placed guardians, and quest creatures
(neutral or unique) never stray. Tests: `content::tests::the_generic_list_is_every_hostile_type_without_uniques_or_people`,
`a_lair_draws_its_strays_from_the_populations_only`,
`a_first_floor_draw_gone_far_out_of_depth_brings_a_type_of_no_population`, and
`world::spawn_tests::strays_come_from_every_hostile_type_but_those_kept_out`; `populate` builds the generic candidates once a floor, the own ones per creature (a
unique leaves them). Hand-placed creatures (level legends, vaults,
scenario people, shopkeepers) are never drawn, whatever their tier.
Neutral animals (#1195; the owner: "it's a game about killing stuff, not Dwarf Fortress"):
`Odds::neutral` (`neutral`, percent 0 to 100, `[defaults]`'s, overridable by a branch or a
hand-made level; 100 without the field) scales the rarity factor of every candidate not hostile
to the player by its data stance (`Content::stance` with `player`; the `wildlife` faction and
anything else neutral by default; never Beast-friend's empathy, which is the player's, not the
type's) in `Content::draw_candidates`, at least 1 unless 0 %. The floor's population
(`mapgen/population.rs`, the count's draws) and the spawner (`world/spawn.rs::draw_spawn`) draw
by it; the count is unchanged, so the freed weight goes to the hostile types. Untouched, as
places that mean what they draw: a vault's monster pools, teeming dens (a beast den's
`swarm.factions` is animals by design), hoards' guardians (hostile anyway), outcomes' summons,
and overworld encounters (wildlife belongs outdoors). Tests:
`mapgen::neutral_share_tests` (the share over every branch floor and many seeds; the ignored
`print_neutral_shares` prints the per-branch table), `content::danger::tests` (the bounds).
Dread (docs/design/monster-dread.md §8, #1124): `CreatureDef::dread` (`dread = true`, default
false) marks the types to dread, flagged for every type §6 sorts as dread (the gimmicks come in
later slices); a rarity tier is still the type's own. `danger::draw` rolls first, when
`Odds::dread_one_in` is above 0, one `below(dread_one_in)`: 0 makes the draw among the dread
candidates alone (`danger::Among::Dread(true)`, a filter inside `pick`, no allocation), anything
else among the rest; the same stray and out-of-depth draw is then made, and if it finds nothing,
again among the other side. `Odds::without_dread` (`dread_one_in` 0, never in data) rolls nothing
and reads no flag: a vault's monster pool (`mapgen/population.rs`). A den's warren or tribe starts
from a non-dread type drawn without the roll (`mapgen/den.rs::Theme::roll`). The odds of a draw on
a level are `Content::draw_odds_at(id)`: its spawning rules' odds with a branch depth's own
`dread_one_in` (`DungeonDef::dread_one_in`: an endless branch's `content/dread.rs::DreadCurve` at
the floor's true depth, else its `[[dungeon.levels]]` entry's `LevelPlan::dread_one_in`); the
spawner, the population (`mapgen::generate_for` patches the `Spawning` it hands `populate` and
`den::fill`), hoards' guardians, and outcomes' summons read it; an encounter's draw takes its
branch's odds. Tests: `content::danger::tests` (the roll, the fallbacks, determinism),
`content::dungeon::tests::a_depths_dread_one_in_wins_over_the_branchs_over_the_defaults`,
`an_endless_branch_reads_its_dread_curve_at_the_true_depth`, `content::dread::tests`,
`content::dread_roster_tests` (the sort's flags and tiers, no unique dread), and
`world::spawn_tests::the_spawner_rolls_for_dread_at_the_levels_own_odds`,
`draw_odds_take_the_most_specific_dread_one_in`.
Splitting and dying into (docs/design/monster-dread.md §5.5, #1129; spec §8 "Splitting and dying
into"): `CreatureDef::splits` (`content::Splits`: damage types, `least`) and `dies_into`
(`content::DiesInto`: pool, count, spread), resolved in `Content::load_splits`
(`content/split.rs`, after `load_summons`). The rules are `world/split.rs`: `World::split`, one
line in `fight.rs::resolve` after a blow that left the target alive; `World::dies_into`, one
line in `fight.rs::dies` before `creature_dies` (its line is said after the death's break
lines); `World::offspring_share` in `dies` for the credit; `Actor::split_from` (`Split::Half`,
`Split::Corpse`; saved, left out while `None`). Look mode's ", writhing" is the UI's
(`ui/look.rs`). Tests: `world/split_tests.rs`, `ui::look::tests::what_lives_in_a_wormhost_shows_writhing`.
Where the rosters are met (#531): the showcase's Depths has no danger (its patrol of rats, bats,
and kobolds only; there is no general candidate pool). In the Ascmearc a type of level 17 or less
(level 11 or less in Fennmynster, whose danger runs 3 to 10) is placed in a population where it
fits (Fennmynster 3–10, Fyrdelf 5–16, #534); every other hostile type comes only as a stray
(Strays, above). Each tribe's placements are in its file below.
`cargo xtask bench-turns` has a `delf` row: a fixture 120×60 cave branch of danger 8 at full
population (`xtask/src/bench.rs`); its night's rest is held to the 500 ms budget thanks to coarse
turns.

Coarse turns while the player rests (#349, spec §8 "Coarse turns while resting"): `world/stride.rs`,
one call at the top of `World::creature_turn` (`coarse_turn` → `Coarse::Strode(price)` returns at
once, `Slow` multiplies the usual action's price by `STRIDE`, `Usual` is the usual turn). `away`
(the player rests, a local level, no dialogue, and not hostile to the player within its sight
radius, nor its mind's `last_seen`/`noise`), `idle` (calm, nothing heard worth a look, no
statuses, able to move, no hostile within its sight radius; returns the hostiles within its
sight plus a stride), `stride_path` (up to `STRIDE` = 20 route steps onto plain cells: walkable,
no door, ice, trap, actor, or shunned cell, not a cell the player's last perception saw creatures
on, not within its step noise's reach of the player, none within its sight of a hostile; prices
and step noise cached per terrain), and a stride's one `make_step_noise` at its end. An idle
creature whose next route cell is taken waits `STRIDE` waits; one whose first step is not plain
takes its turn as usual. Outside a rest nothing here runs. Tests: `world/stride_tests.rs` (pace
kept, busy ones slowed, turn by turn near the player, replays, the rest's sight equals a
perception's); the rest-versus-waiting tests in `time_tests`, `cost_tests`, `hearing_tests`, and
`talk_tests` still hold (their creatures cross into view or earshot turn by turn), and
`speed_tests::resting_accounts_for_fast_creatures` compares a rest with resting turn by turn. Tests: `world/spawn_tests.rs` (the property test of the
design's §9.6, the population of a generated floor and that nothing else moves, uniques in a
population, a spawn's gear stream, frozen floors, the talents, determinism),
`content/danger.rs` (rules, counts, weights, draws, a rare type drawn less often, out-of-depth
draws by rarity), `content/creature_rarity.rs` (the tier file and its refusals), and
`content::tests::every_drawn_creature_has_a_rarity` (shipped content).

Creature gear (spec §8 "Creature gear", #328): the data is `content/gear.rs` (`GearLineSpec`
as written, kept on `CreatureDef::gear_spec` until objects load, then `Content::load_gear`
resolves `CreatureDef::gear`: the racial weapon's line first, `Keep` defaults by the slot an
object fits; what fits no slot as kept is carried, `Pack::equip`).
The rolls are `gear.rs`: `gear::roll(content, rng, creature, item_level) -> kit::Pack` (the
kit's `Pack::put`/`equip`, `loot::make_as` for a magic piece, `variant::draw` with
`Source::Gear { level, faction }` (the type's level and faction: a faction's `gear`
favourites, `data/factions.toml`, #1191, `docs/dev/objects.md` "Materials and variants") for a
plain one, the burden guard against `burden::limit` of the
type's stats and size). Who rolls is `CreatureSpawn::rolls_gear` (true for a legend or vault
entry without `carry`/`wield`/`wear`, `Canvas::add_patrol`, the local areas' deer; false for a
keeper): a generated level clears it in `gear::arm` (stream `gear/<level id>`, after
`place_loot`, in both `mapgen::generate_with` and `mapgen::local::generate`), and
`world/travel.rs::placed_gear` rolls the rest as `instantiate_level` places them (stream
`gear/<actor id>`). The danger levels' paths set `rolls_gear`: the initial population of a
generated floor (rolled by `gear::arm` after it) and a spawn in play (rolled by
`make_creature`). What falls is unidentified (Scavenger, which told it, was cut by #816). Tests: `world/gear_tests.rs`; the mapgen
snapshots print each spawn's gear.

Gear condition (#1193): right after each `gear::roll` (one line in `gear::arm`, one in
`placed_gear`), `world/gear_condition.rs::apply` draws each piece's state ([`Worked`], the
smith's: damaged with `by` for its word, or one to three improvements, each an attempt) from its
own stream, `condition/<key>` with the gear stream's key, so the gear's draws and the rest of the
level do not move. One draw below 1000 per piece against the creature's tribe's odds
(`data/gear_condition.toml`, `content/gear_condition.rs`: `[default]`, `[[tribe]]` by creature
tag, first match in file order), then one per improvement for its aspect. Only single pieces of
arms or armour an anvil or knife could work (`temper::arms`, aspects), never protected ones
(`item_damage::protected`, with every object any quest may ask for, fixed or rolled: gear is made
before a quest is given). The kind is by the main part's material group (`[kinds]`, metal: rust
and acid): the one the material withstands least, never one at 100; else `ItemHarm::Work`
("damaged"). `World::loot_of` (a death's extra drop) is treasure, not gear, and takes no
condition. Tribes chosen: undead (95 % damaged), duergar, hobgoblin, drow, devil (disciplined:
8–10 % damaged, Fine 10–15 %), kobold, goblin, orc, troll (scavengers: 45–55 % damaged); the
rest the default (30 %, Fine 4 %, Marvellous 0.8 %, Masterwork 0.2 %). Tests:
`world/gear_condition_tests.rs`.

## Variance by role

The variance pass (#1134, `docs/design/monster-dread.md` §7.2) spread a level's monsters by
role on #1097's `to_hit = L/4`. The shipped types were written to these rules; since #1205 no
test holds the data to them (the data author decides):
- **to-hit** of the natural attack: a striker (quick, assassin) `L/4 + L/8` (at most the data's
  20), a brute, shambler, or caster `L/4 − L/8`, everyone else `L/4`, each part rounded down. An
  armed type (a wielded weapon in its gear, the racial weapon too) keeps `L/4`: its spread is
  the gear's and the weapon skill's. **Armed types aim by their ranks, never a flat to-hit**
  (#1169): the weapon's
  skill (and Duelist, for a one-handed weapon fought with no shield in the gear) is raised to
  the fewest ranks whose to-hit reaches `L/4` (a caster, a type with spells, `L/4 − L/8`), no
  rank past what a trained warrior of the level holds (the balance probe's `rank`: 2 at level
  1, 7 at 15–22, 10 from 42); a heavy weapon (great blades, axes, polearms: their ranks give
  damage first) may stay below and holds the most the cap gives. A new armed type takes its
  ranks by the same rule.
- **threat and crit** of the natural attack: a striker `threat = 19` (crit ×2), a brute
  `threat = 24`, `crit = 3`, everyone else the defaults (22, ×2). The mean of the dice is the
  rosters' as before.
- **hit points**: the role factor on `base_hp = 2.5 × L × factor` runs **0.5** (quick, small,
  caster) to **2.0** (brute; the giants and ogres too), the shambler 1.5, the spirit 0.6, the
  standard 1.0 (archer 0.9, officer 1.1), ±35 % kept; the brutes stand in its top part (every one at 1.25 to 1.35 of its formula, the
  dread brutes at the top, so the probe's slowest tenth reaches twice the median). The
  mimics' `hp_factor` is the brute's.
- **the starters** (#1200, `content/starter.rs`): a type of level 1 to 5 has
  `starter::hp_percent` of its formula's hit points (40 % at level 1, 55, 70, 80, 90 % at 5),
  Toughness and `base_hp` alike: the generated families' `Numbers::of` cut both, and each
  hand-written file states its cut numbers (Toughness at most 20 × the share). Their riders were
  written rare and mild: at most 1 % at level 1, 3 % at 2 and 3, 4 % at 4, 5 % at 5, one degree,
  no drain; a rider that defines its creature (the spiders' and the centipede's venom, the
  leech's latch and bleeding) up to 12 % from level 2. No poison at level 1.
- **speed** (every faction but the beasts, the people, the player, and the layer keepers;
  uniques and the generated families left out): the bulk between
  800 and 1250 by the tribe's temperament (each tribe file names it), the dread at 600 or 700
  (the slow and awful) or 1400 and up (the quick dread), no tribe with every type at 1000, and a
  people's rank-and-file (the orcs', hobgoblins', drow's, and duergar's standards and archers)
  at 950 to 1050.

The probe's "Spread" table (`balance_probe/spread.rs`, `docs/balance.md`) measures what it does:
rounds to kill and to die over a level's monsters as a distribution.

## Creature waves

The campaign's creature waves (docs/design/campaign.md §10.1; #333 the barrow wave, #334 the fen
wave, #335 the delf wave): content only, on the fields above. A wave adds
- one file per type: the common ones in `data/creatures/` (shared), the campaign's own people and
  bosses in `campaigns/<id>/data/creatures/` (numbered after the shared block, only that
  campaign's files may name them; `docs/dev/campaign.md`), the Ascmearc's in
  `campaigns/ascmearc/data/creatures/`. Each file starts with a comment naming its wave; `xp` follows
  `5 × level + 10`; a humanoid has `racial_weapon` with a rank in its skill and `[[creature.gear]]`
  lines (a second weapon of another skill wants a rank there too, so it benefits);
- its factions in `data/factions.toml`, inserted by group (after the kobolds), never at the end.
  Four test fixtures write their own neutral factions file over the shipped one
  (`world/{equip,magic,object,read}_tests.rs`): every shipped faction id must be listed there
  too, or the shipped types naming it fail to load;
- its rows in `world/bestiary_tests.rs`'s `WAVES` (the type and its owning campaign), and its
  stances in that file's faction test.

**`hostile_to_all`** (`content/faction.rs`): a faction with it (`berserk`, Rouse the dead's,
`docs/dev/allies.md` "The risen") fills its row, column, and own diagonal of the hostility matrix
(a `hostile` list beside it adds nothing). So `World::stance` of an actor and itself is `Neutral` by id
(`stance_between`), and `Factions::has_enemies` counts a faction's own quarrels only: being on the
berserk's list makes no one a fighter (`disturbs_rest`, hearing, `stride.rs`, the wiki's `fights`).
`fixture_factions.rs` strips the line with the other grudges, so a fixture's berserk are at peace;
the fixtures that write their own `data/factions.toml` have no such faction, which is why the
spell's `berserk_faction` is read at the cast (none: it rises serving) and checked on the shipped
data by a test rather than at load.

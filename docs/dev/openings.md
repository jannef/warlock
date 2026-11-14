# The thief's openings: the mark and its builds

The Openings pool (`data/resources.toml`, `docs/dev/resources.md` "Openings") remembers whose
guard its points are in: the **mark**. Design: `docs/design/thief-openings.md` (#1449); this
file covers phases O1 (#1461: the mark, the kill that carries, and the builds), O2 (#1462:
a feint builds), and O3 (#1463: Exploit at level 1, Low blow its first increment, Trained wind
off the thief's table). Spec §8 "Resources", "Class abilities", and "Surprise and sneak
attacks".

## Data

- `marked = true` on a `[[resource]]` (`ResourceDef::marked`): its points are in one creature's
  guard. Only Openings has it. Every rule below reads the flag, never the id.
- `builds` on Openings, one each: `sneak_attack`, `feint` (O2, #1462), and `mark_kill`.
  `Build::MarkKill` (`mark_kill`) replaced `Build::SneakKill` (`sneak_kill`, gone).

| Build | Where | The mark rule |
|---|---|---|
| `sneak_attack` | `ambush.rs`, when the sneak dice land | the attack ran `choose_mark` or `free_blow_at` first; `builds_on` skips a free blow's at a stranger |
| `feint` | `feint.rs::feint`, on "falls for it" | `choose_mark(foe)` after the refusals, before the check: a feint at a new foe empties the pool first, then builds; a failed feint builds nothing and still moves the mark |
| `mark_kill` | `mark.rs::mark_dies`, from `fight.rs::dies` | the mark is cleared after the build |
- Patient blade (`talents/patient_blade.toml`): `resource_build` +1 on `mark_kill` (the mark's
  kill gives two) beside its `resource_rate` −50.

## State

`Actor::mark: Option<ActorId>` (`world/actor.rs`), `#[serde(default, skip_serializing_if =
"Option::is_none")]`: saved and hashed, absent while `None`, so runs without a thief save and
hash as before. Only the player ever has one; it is set and cleared only in `world/mark.rs`.
`World::mark_of(id)` reads it while the creature is still in the game.

## The rules (`world/mark.rs`)

| What | Function | Effect |
|---|---|---|
| A chosen attack at X | `choose_mark(X)` | no mark: mark := X, the pool kept; mark X: nothing; mark Y still in the game: every marked pool := 0 ("Your openings on the kobold are lost.", only when it held anything), mark := X. Nothing without a marked pool |
| A chosen shot or throw at a cell | `choose_at_cell(at, false)` | `choose_mark` of the creature standing on the aimed cell, if any; who the missile meets on its way is not chosen |
| A free blow at X | `free_blow_at(X)` (and `choose_at_cell(at, true)` for a free shot) | mark := X when none stands; else nothing |
| A sneak attack's build | `builds_on(holder, target)` → `World::build_resources(id, on, marked)` | a marked pool builds only on the mark, or with none standing; other pools as ever |
| A death | `mark_dies(id, by)`, in `fight.rs::dies` beside `Build::Kill` | `by`'s mark was `id` (its own kill; a wound it dealt bleeding it out counts): `gain_resource(by, MarkKill)`; then the mark is cleared, the pool kept, whoever killed it |
| A departure | `unmark(id)`, in `ally.rs::departs` | cleared, the pool kept (a lapsed summons) |
| A level change | `unmark_on_travel()`, in `travel.rs::travel` | cleared, the pool kept |
| Time passing | `mark::faded(actor, content)`, in `World::live` after `resource::pass` (both paths) | cleared once every marked pool held is at 0 (decayed, or never built): no mark stands over an empty pool after a turn's time |
| An active paid from a marked pool | `off_mark(active, dir)`, in `class_ability.rs::use_class` after the hostile check | refused, no time, nothing spent: "Your openings are in the kobold's guard." while a mark stands and the creature that way is another (Exploit) |

A mark out of sight stays the mark. The mark is named as the player knows it: `World::called`
while they see it, "an unseen foe" while they do not (`mark_called`; `content::possessive` for
"the kobold's").

### The hooks (where `choose_mark` runs)

Before the attack, so its sneak attack builds in the new mark's pool:

- `fight.rs::player_attacks` (a blow by moving or attacking), `player_blow_with` (every class
  or weapon active's single blow: Exploit, Smite, Quivering palm, Lunge, ...),
  `player_kicks`, `bash`;
- `weapon_active.rs::rush` (Shield rush), `sweep` (Whirl, Staff sweep: only the one in the aimed
  direction; the rest are struck through `strike_with` and swept up, an area);
- `shoot.rs::fire` and `throw`, `weapon_active.rs::active_shot`, `volley.rs` (`choose_at_cell`);
- `cast.rs::cast_single`, the player's own single-target spell that rolls to hit (a wand's and a
  scroll's bolt too). Bursts, cones, and other area spells touch nothing;
- `feint.rs::feint` (O2), after its refusals and before its check: a refused feint moves nothing.

`free_blow_at`: `free_blow.rs::strike_free` (riposte, opportunist, a lurk's gatekeeper blow,
hair trigger, retribution, a cleave) and `weapon_ability.rs`'s free shot (Point-blank shot).
The class active Whirlwind strikes everyone around through `player_blow`: an area, untouched.

Vanishing act stays where a sneak attack's blow kills, in `fight.rs::resolve` (`unseen` is
known only there); it no longer rides a build.

## Exploit (`world/openings.rs`)

The thief's one active for openings from level 1 (`ClassActive::Exploit`, the class talent
`exploit`, `use:exploit:E`, `Aim::Direction`, the abilities menu only; its `KeyCommand` unbound).

| Step | Where | What |
|---|---|---|
| Refusals, no time, nothing spent | `use_class`: `may_use_class` (greyed `Unavailable::Short`, "you have no opening": Openings' `lacking`), "Which way?", then `thief.rs::thief_may` → `openings.rs::exploit_may` | "There is no one there." (no hostile that way); `off_mark` ("Your openings are in the kobold's guard."); "It has no vital spot." for a foe the player **sees** without vitals (`Content::has_vitals`; an unseen one is struck and told so after, as any blow, so nothing leaks) |
| The price | `ability::class_cost` | the talent's `resource` cost (1) is the least it needs; for Exploit the cost is **every opening held** (n), spent by `use_class` and handed to `World::exploit` |
| The blow | `World::exploit` | a melee attack's price and exertion; the words by n ("You strike at the opening." 1, "You press your openings." 2–4, "You tear into every opening!" 5+); `exploit_profile(n, melee)` (`ExploitMods::profile`: +n × `to_hit`, the threat n × `threat` lower, never below 2, `Profile::sure_critical` from `sure_at` spent; the sneak attack's own lower threat, `ambush.rs::sneak_threat`, never raises it back); `Technique::sneak` (a sneak attack whatever the foe knows: the sneak dice land on a hit, the foe keeps its Defence and is not Surprised) and `Technique::exploit = n`; `player_blow_with` (which runs `choose_mark`) |
| The ladder's dice | `fight.rs::resolve` → `openings.rs::exploited` → `land_exploit(n, blow)` | after `ambush.rs::ambushed`, on a hit whose sneak dice landed: n × `ExploitMods::dice` from the gameplay stream, after Protection, never multiplied |
| No build | `ambush.rs::ambushed` asks `exploiting(attacker)` | an exploit's sneak attack builds no opening (it spends them) |
| After | `World::exploit` | `unmark(target)`: the mark goes with the pool unless the blow killed it (then `mark_dies` cleared it and `mark_kill` gave one); on a hit at a foe still standing, a degree of `ExploitMods::status` (Low blow's Stunned); then `after_attack`. Vanishing act fires on a kill as on any sneak kill |

### The ladder

`TalentEffect::{Exploit, ExploitDice { dice }, ExploitToHit { value }, ExploitStatus { status },
ExploitThreat { value, sure_at }}` (`exploit`, `exploit_dice`, `exploit_to_hit`,
`exploit_status`, `exploit_threat`), collected into `TalentMods::exploit_active` and
`TalentMods::exploit: ExploitMods { dice, to_hit, status, threat, sure_at }` (`exploit_dice`
summed with `Dice::and`, `exploit_threat`'s values summed and the lowest `sure_at` kept;
`exploit_threat`'s two fields are at least 1). Two places read it: `ExploitMods::profile`
(through `World::exploit_profile`) before the roll, and `land_exploit` after a hit. There is no
cap on n beyond the pool's maximum (Of Guile's eight openings are 8 × the dice and a threat of 2).

| Level | Talent | Adds per opening spent |
|---|---|---|
| 1 | Exploit | the blow; +1d6 (`exploit_dice`) |
| 12 | Low blow (no active since O3) | +1 to hit (`exploit_to_hit`); the foe Stunned on a hit (`exploit_status`: flat-footed, its next action dear) |
| 32 | Improved sneak attack (keeps `ambush 2d6`) | a second `exploit_dice` 1d6: 2d6 an opening |
| 60 | Master thief (keeps `ambush_threat 2`) | threat 4 lower, never below 2 (`exploit_threat`); with five or more spent a critical whenever it hits, no confirmation (`sure_at = 5`: the finisher, folded in; `ClassActive::MasterThief` and `use:master_thief` are gone, O4 #1464) |

Trained wind is off the thief's table (a save holding it keeps it); the pool comes with the first
active weapon rank (`fight.rs::grant_stamina`).

## What the player sees

- Character screen: the pool's row `Openings 3/5, in the kobold's guard` (`OwnResource::marked`,
  `PerceptionView::own_mark`).
- Look mode: "your mark" after the seen statuses (`SeenActor::marked`, from the player's own
  mark).
- Abilities menu: `Exploit (class): a direction`, greyed "you have no opening" while empty.
- Messages: "Your openings on the kobold are lost." (a chosen attack elsewhere, the pool above
  0); "Your openings are in the kobold's guard." (a refusal). Nothing on a build or a carry.
- Dumps: `stats` `Mark the kobold`; the dev `actors` layer's `; mark #12` on the player's line;
  JSON `resources[].on` (the mark's name for a marked pool, else `null`), seen actors'
  `"marked": true`.

## Determinism

Nothing here draws. The mark is hashed: a thief's run hashes differently from its first chosen
attack. Saves are pinned to the version (no shim).

## Tests

`world/mark_tests.rs` (a chosen attack sets, keeps, or empties, said once and only above 0; a
free blow sets and never moves or builds on a stranger; a throw counts and a burst does not;
the mark's death by the thief carries and adds one, by another carries; a departure, a level
change, and the decay to 0 clear; Exploit's refusal off the mark; the dumps, a save, and a replay);
`world/feint_tests.rs` (a feint made builds one, at the mark too; a failed one builds none and
moves the mark; one at a new foe empties first, then builds; a refused one moves nothing);
`world/openings_tests.rs` (Exploit at level 1, each refusal with no time and nothing spent, the
spend of all with the words by n and no build, the ladder's dice by n, Low blow's stun and
to-hit, Improved sneak attack's 2d6 at 32, Master thief's threat by n and the sure critical from
five at 60 (every hit of five a critical; no finisher row, no `use:master_thief`), a pool of eight
spent whole, a killing exploit's one opening and no mark, the replay); `world/class_rogue_tests.rs`
(the table: Exploit at 1, no Trained wind, no stamina until an active rank; Vanishing act on a
killing exploit); `world/resource_talent_tests.rs` (Patient blade's two); `content/resource.rs` (`marked`,
`mark_kill`, and `feint` parse, `sneak_kill` is refused); `content/talent.rs` (the `exploit_*`
effects refuse a bad die or status, `exploit_threat` a value or `sure_at` of 0); the balance
probe's "Exploits" tables (`balance_probe/exploits.rs`, `docs/balance.md`); `ui/look.rs`'s
`the_creature_your_openings_are_in_is_your_mark` (look mode's "your mark", `own_mark`).

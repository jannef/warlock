# Warlock specification — 9. Development interface

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

## 9. Development interface

```
warlock --scenario showcase --seed 42                     # play a named scenario
warlock --scenario depths --character dwarf,female,warrior --name Brakka # ... as a chosen character
warlock --replay runs/doors.json \
        --freeze-time 1.5 --window 1280x720 --scale 1 \
        --capture out.png                                 # reproducible screenshot, then exit
warlock --record runs/new.json                            # record actions for bug reproduction
warlock-cli dump --scenario showcase --actions "move:N move:N" \
        --format ascii --layers view,fov,memory           # text view of the simulation
warlock-cli dump --replay runs/doors.json --format json --state   # structured dump
warlock-cli replay --create --scenario showcase --actions "move:N wait" -o runs/x.json
warlock-cli replay --verify runs/doors.json               # check per-turn state hashes
warlock-cli gen --level depths:3 --seed warlock           # print a generated level (dev-only;
                                                          # also local areas: local:49,23;
                                                          # `*` a placed light, `&` a lock,
                                                          # `S` a secret door,
                                                          # special rooms by their letter)
cargo xtask golden [--update | --local]                   # compare golden screenshots
```

- Replays are JSON: `{game_version, scenario, seed, character, steps: [{action, hash}]}`, the
  character `{race, gender, class}` (left out for a run without one; a replay without it starts
  as the scenario's character, so older replays still load). Actions use a text
  syntax (`move:E wait open:E close:W ascend descend lantern rest:1h rest:dawn rest:dusk
  pickup pickup:ac drop drop:b sell:b pay pay:W equip:body:b remove:body bash:E fire:12,5
  throw:c:12,5 eat:b
  eat:floor:a dig:E search search:10 chat:E ask:E:rooms ask:E:quest-rats ask:E:accept-rats vision:darkvision
  skip:1h weather:rain stance:coward effect:fireball@12,5 effect:lightning_bolt@3,4>12,4
  identify:a afflict:bleeding infect:filth_fever freeze:12,5,3 thaw:12,5,3`; `bash:DIR` bashes
  with the held shield that way, `dig:DIR` digs toward a direction until the wall gives way or
  something interrupts; `search` searches the cells around the player once, `search:N` for up to
  N turns (1 to 100), stopping when something is found; `fire:X,Y` shoots at level cell (X, Y),
  `throw:LETTER:X,Y` throws one of inventory stack LETTER there; `eat:LETTER` eats one of
  inventory stack LETTER, `eat:floor:LETTER` one of stack LETTER of the pile underfoot;
  `chat:DIR` talks to the creature that way, `ask:DIR:TOPIC` asks it about a topic by id (a
  quest's: `quest-ID`, `accept-ID`, `decline-ID`); the dev
  action `effect:ID@X,Y[>X,Y]` shows spell effect ID (`data/effects.toml`) as the player perceives it, a
  burst around X,Y (around the second cell, flying there from the first) or a bolt from the first
  cell toward the second (cut short by the first opaque cell); it hurts no one, takes no time,
  and only says so in the message log; the dev bursts `freeze:X,Y,R` (an instant frost: the
  water within radius R, 0 to 20, of cell (X, Y), in its line of sight, freezes to 10 cm of ice)
  and `thaw:X,Y,R` (a burst of heat: the ice there melts, and whoever stood on it falls in) take
  no time (§5 "Ice");
  durations are a whole number with `s`, `m`, or `h`, a rest at most `24h`; letters name stacks
  of the pile or the inventory by position, `a` first, `z` the 26th, and any position may be a
  number instead (`drop:0,2,30`, `equip:body:26`, `throw:27:12,5`), as positions past `z` must;
  without letters `pickup`, `drop`, `sell`, and the
  dev `identify` (§8 "Magic items") take everything; `sell` sells to the shop the player stands
  in, `pay` pays it, and `pay:DIR` pays and steps out that way (§7 "Shops"); `equip:SLOT:LETTER` puts inventory stack LETTER
  on in slot SLOT (a slot id of `data/slots.toml`) and `remove:SLOT` takes off what is there;
  the dev weather override takes `clear`, `overcast`, `rain`, or `heavy`; the dev `afflict:ID`
  gives the player a degree more of status ID (`data/status.toml`), and `infect:ID` makes them
  catch disease ID (`data/diseases.toml`); in scripts, `ACTION*N` repeats an
  action, e.g. `wait*10`); `hash` is the
  FNV-1a 64 hash of the
  canonical JSON encoding of `WorldState` after the step. Verification reports the first
  diverging step. `--replay` takes its scenario, seed, character, and name from the file;
  `--record` rewrites its file after every action (including replayed steps). `--character
  RACE,GENDER,CLASS`, like `--seed`, shapes the run that `--scenario`, `--new`, `--record`, or
  `--capture` starts (and `warlock-cli dump` and `replay --create`); without it, the scenario's
  character. `--name NAME` names that character (up to 20 characters; without it the scenario's
  name for its own character, else one of the race's names from the seed, §8 "Character
  creation"). Replays record the name (`"name"`, beside `"character"`).
- Dump layers: `view`, `fov`, `memory` (terrain, the top object of each known pile, and known
  traps over them), `light`
  (perceived brightness of each visible cell as a digit, `0` dark to `9` fully lit or brighter),
  `sound` (the view with each heard sound at its rough place: `f` footsteps, `o` a door opening,
  `c` a door closing, `x` fighting, `b` a bowstring, `m` chanting, `k` banging (a door kicked),
  `d` digging, `v` a voice, `a` an alarm bell, `e` an explosion, `t` a thunderclap, upper case
  when loud),
  `traps` (the traps the player knows, one a line, `(12, 5) fireball trap`, `(remembered)` when
  out of sight; `no known traps`),
  `inventory` (what the player carries, grouped by item class as the screens group it, a
  `Melee weapons:` line before each class, each stack by its position as action text names it
  with weights and what is worn or wielded marked, `b - a leather jerkin (being worn) (3.5 kg)`, and in a shop what is unpaid, `c - a
  dagger (unpaid, 45 gold) (450 g)`, then what they carry, their burden and
  its thresholds, and what they weigh, `carried 4.2 kg (unburdened: burdened over 14 kg,
  strained over 28 kg, overloaded over 42 kg), total weight 74.2 kg`),
  `equipment` (the player's slots by letter, `c Body: a leather jerkin (b)`, `-` when empty),
  `stats` (the player's name, race, gender, and class, `Name Brakka`, `Race Dwarf`, their age, birthday, and
  sign, `Age 27`, `Born 3 Seedtide 1417`, `Sign Fire Manticore`, then their own scores as
  the rules read them, `Strength 12`, then their status effects by name, `Effects Bleeding,
  Diseased`, then the diseases they know they have, `Diseases filth fever`), `skills` (the player's trained skills in skill order, `Stealth 17`),
  `weapon_skills` (the player's weapon skills and fighting styles in file order with their rank
  and how far their marks are toward the next, `Long blades 2 (40%)`; at rank 10 the rank alone;
  the styles they fight in now marked, `Duelist 1 (0%), in use`), and `quests`
  (the quest log, active first: `Rats in the arena (active, from the old hermit): Kill 3 giant
  rats in the Arena. 0 / 3 killed.`) are built from the perception snapshot only;
  `truth` (terrain, objects, traps hidden or known, secret doors as `S`, and creatures) and `actors` (every actor on
  the level with its hit
  points and Defence, `hp 8/12 dv 13`, its
  weight, base plus carried, what it carries and of that wears and wields, `equipped main_hand a
  spear` (magic items as they truly are, `an iron ring of the Bear (unidentified)`), what its
  magic items do to it by effect id, `magic strength +1 light item_glow`, its burden and carrying
  limit, `unburdened (limit 31.5 kg)`, its weapon skills with rank and marks, `weapon skills
  spears 1 (0/30)`, its stats as score/potential, `St 12/15`, its trained skills with their
  marks toward the next point, `skills stealth 17 (2/5)`, its status effects with their
  severities, `effects bleeding 2`, its diseases with their progressions (checks won in a row)
  or `incubating` and the time to their next turn, `diseases filth_fever 2 (1 won) 0d17h`, and
  its stat damage, `damage Dx 2 To 1`, the player's nutrition and hunger, `nutrition 3540
  (not_hungry)`, and the unique character it is and how far its talk has got, `npc elder; talk
  said 3, muttered 1, met`, if any; then every corpse on the level and in the player's pack with
  when it died and how it has rotted, `corpse rat_corpse at (2, 1): died 1444-11-11 08:01:12, 2
  hours ago, fresh`), `rooms` (the level's special rooms: the terrain with each room's plain
  ground drawn as its effect's `mark`, then a line per room, `s shadowed room at (34, 8)-(49,
  13), entered`), and `ice` (a line with the temperature on the level and when its ice was last
  brought up to date, then the map with each iced cell's thickness in whole centimetres, `0`–`9`,
  `a`–`z` for 10–35, `#` beyond, over the terrain) show
  ground truth and are dev-only. JSON dumps
  carry turn, game time (seconds and calendar fields), level, viewer, the special room the
  player stands in if they know it (`room`, its effect id; else null), the open shop they stand
  in (`shop: {sign, keeper, owed}`; else null; §7 "Shops"), the weather (under the open
  sky; else null), the temperature where the player is (`temperature`, °C to a tenth), seen
  actors (each with how hurt it looks, `health`: `unhurt`,
  `lightly_wounded`, `wounded`, `badly_wounded`, `nearly_dead`, and the status effects that show
  on it, `effects`, by id), heard sounds, known traps (`known_traps: [{pos, trap, seen}]`), the
  player's
  inventory (each stack's letter, object, count, weight, `name` as the player knows it, `rarity`,
  `identified`, and `magic`: `{level, name, affixes: [{id, tier, value}]}` once identified, else
  null; an unpaid stack with its price, `unpaid`), equipment (by slot id: letter, object, count), weight, burden (`burden: {level,
  limit}`, the limit in grams), character
  (`{name, race, gender, class, raised, age, born: {year, month, day}, sign: {animal, element}}` or
  null), scores (`stats`, by stat id), trained skills (`skills`, by skill id with their values),
  status effects (`effects`, by id), the diseases they know they have (`diseases`, by id),
  weapon skills and fighting styles (`weapon_skills`, by skill id: `{rank, progress}`, the
  progress a percent or null at rank 10), the styles in use (`styles_in_use`, style ids), the
  quest log (`quests`: `[{id, name,
  kind, active, completed, giver, objective, status}]`), hit points (`hp: {current, max}`),
  Defence (`defence`, the total), hunger (`hunger`: `bloated` … `starving`, or null when nobody
  hungers), how the run ended (`death: {turn, time, place, by, of, cause, trap}` or null;
  `cause` "Starved to death", "Killed by a fireball trap", and the like when neither a creature
  nor a status effect killed them; `trap` the id of the trap that did, else null),
  recent messages,
  and optionally the full world state; the ASCII header line has the turn, the time
  (`0001-01-01 08:00:00`), level, viewer, the player's hit points (`hp 17/17`, then `dead` once
  they have died), their hunger unless Not hungry (`hunger hungry`), the weather under the open
  sky (`weather rain`), the temperature (`temperature -3.5 °C`), and hash.
- Freezing visual time and fixing window size and scale make screenshots reproducible. With frozen
  time, glyph animations show that moment and motion (slides, camera) completes instantly.
- `--fire` (dev) starts the run aiming the readied missile weapon, as `f` would, so captures show
  the targeting line (golden manifest entries pass it with `args`); `--look X,Y` (dev) starts in
  look mode with the cursor on level cell (X, Y), so captures show what look mode says there;
  `--keys "KEYS"` (dev) presses keys at startup, after any replay, as if typed, so captures show
  what they open (the shop's dialogs: `--keys h`, `--keys "d b Enter"`); `--screen dialog`
  opens the quit dialog.
- Dev builds add in-game commands: switch vision profile (`F3`), reveal map (`F2`, ground truth,
  hidden traps included, in view too; marked as dev-only on screen), toggle debug overlays (light values `F4`; line of sight and memory
  later), save and load named snapshots (`F5`/`F9`), skip an hour (`F6`, recorded as `skip:1h`),
  set the next weather for three hours (`F7`, recorded as `weather:overcast` and so on), identify
  everything carried (`F8`, recorded as `identify`; §8 "Magic items").
  They are commands of their own context in the bindings table, read (and listed by Controls
  and the `?` help, which can rebind them) only where they work. Dev builds (debug, or `--dev`)
  also give New run its seed field (§6 "Main menu"); elsewhere a new run's seed is random.
- CLAUDE.md documents build commands, architecture notes, pinned documentation references, and
  verification procedures, including the agent workflow guidance from the original brief.
  Verification uses tests, dumps, logs, and screenshots — a successful compile alone does not
  verify rendering.

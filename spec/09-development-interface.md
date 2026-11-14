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

- The flags, the replay format, the action text syntax, and the dump layers are listed in one
  place, [`docs/dev/cli.md`](../docs/dev/cli.md), kept with the code; this section states what
  they must do.
- Replays are JSON: the game version, scenario, seed, character (left out for a run without one;
  a replay without it starts as the scenario's character, so older replays still load), the
  character's name, and the steps, each an action in the text syntax and the FNV-1a 64 hash of
  the canonical JSON encoding of `WorldState` after the step. Verification reports the first
  diverging step. `--replay` takes its scenario, seed, character, and name from the file;
  `--record` rewrites its file after every action (including replayed steps). `--character
  RACE,GENDER,CLASS`, like `--seed`, shapes the run that `--scenario`, `--new`, `--record`, or
  `--capture` starts (and `warlock-cli dump` and `replay --create`); without it, the scenario's
  character. `--name NAME` names that character (§8 "Character creation").
- Every player action has a text form (`move:E`, `pickup:ac`, `fire:12,5`, `rest:dawn`; in
  scripts `ACTION*N` repeats one), so any state can be reached from a script and recorded. Dev
  actions (experience, status effects, weather, spell effects for captures, and the like) take no
  game time unless they say so and exist to set up states for tests and captures.
- Dump layers show the perception snapshot only (what the player sees, remembers, hears, and
  knows of themselves), except the dev-only layers (`truth`, `actors`, `progress`, `rooms`,
  `ice`), which show ground truth. JSON dumps carry the same and optionally the full world state.
- Freezing visual time and fixing window size and scale make screenshots reproducible. With frozen
  time, glyph animations show that moment and motion (slides, camera) completes instantly.
- `--fire` (dev) starts the run aiming the readied missile weapon, as the Fire command would, so captures show
  the targeting line (golden manifest entries pass it with `args`); `--look X,Y` (dev) starts in
  look mode with the cursor on level cell (X, Y), so captures show what look mode says there;
  `--keys "KEYS"` (dev) presses keys at startup, after any replay, as if typed, so captures show
  what they open (the shop's dialogs: `--keys h`, `--keys "d b Enter"`); `--screen dialog`
  opens the quit dialog; `--screen story` opens the background story as a run begins shows it
  (over a `--character` run or a `--replay` with `kit`; §6 "Your story"), which the app never
  opens by itself under `--capture` or `--replay`.
- Dev builds add in-game commands: switch vision profile (`F3`), reveal map (`F2`, ground truth,
  hidden traps included, in view too; marked as dev-only on screen), toggle debug overlays (light values `F4`; line of sight and memory
  later), save and load named snapshots (`F5`/`F9`), skip an hour (`F6`, recorded as `skip:1h`),
  set the next weather for three hours (`F7`, recorded as `weather:overcast` and so on), identify
  everything carried (`F8`, recorded as `identify`; §8 "Magic items").
  They are commands of their own context in the bindings table, read (and listed by Controls
  and the `?` help, which can rebind them) only where they work. Dev builds (debug, or `--dev`)
  also give New run its seed field (§6 "Main menu"); elsewhere a new run's seed is random.
- CLAUDE.md documents build commands, architecture notes, pinned documentation references, and
  verification procedures, including the agent workflow guidance from the original brief; the
  CLI reference is `docs/dev/cli.md`.
  Verification uses tests, dumps, logs, and screenshots — a successful compile alone does not
  verify rendering.

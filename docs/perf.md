# Performance budgets (spec §10)

Proposed budgets, to be confirmed on reference hardware: 60 fps at 1440p on an integrated GPU;
turn processing at most 5 ms in the hub and 16 ms in the vast level.

## Methodology

- **Turn time**: `cargo xtask bench-turns` (release build). Scripted walk of 400 turns per level
  through `warlock-core` only (no rendering); each `World::apply` that takes a turn is timed. The
  verdict compares the **max** turn time with the budget; the command fails when over budget.
  After the walk it skips (dev `skip`) to 18:00 and times one `rest:dawn`: 7200 six-second turns
  through the normal pipeline unless a creature interrupts it (budget 500 ms).
- **Frame time**: `cargo xtask bench-frames [WxH]` (release build, default `2560x1440`, scale 1).
  Runs the game offscreen with `WARLOCK_UNPACED=1` (removes the 60 Hz capture-mode pacing) and
  `--capture-after-frames` 10 and 2010, 5 runs each, takes the best run of each, and divides the
  wall-clock difference by 2000. Startup, asset loading and the screenshot cancel out. The
  command fails if a level exceeds 1000/60 ms.
- Caveats: this is offscreen rendering with no presentation or vsync, so it is a throughput proxy
  for CPU frame cost plus submitted GPU work, not a measured on-screen frame rate. Frames can be
  pipelined by the driver, so a GPU-bound machine is under-reported. The 1440p integrated-GPU
  claim can only be confirmed on such hardware.

## Results

Machine: Apple M5 Pro (integrated GPU, Metal), macOS 26.6.2, release profile, 2026-10-17, measured
on `main` with lighting (#39), effects (#37), procgen (#40) and the status line (#41) merged.

| Measurement | Hub | Vast | Depths | Budget |
|---|---|---|---|---|
| Turn time, avg | 0.020 ms | 0.051 ms | 0.007 ms | |
| Turn time, max (400 turns) | 0.042 ms | 0.123 ms | 0.016 ms | 5 ms / 16 ms / 5 ms |
| Rest dusk to dawn, 7200 turns (2026-10-17, game clock #52) | 121 ms | 17 ms | 50 ms | 500 ms |
| Rest dusk to dawn, 7200 turns (2026-10-18, speed #75 and hearing #78; 232 / 53 / 82 ms without hearing) | 249 ms | 57 ms | 81 ms | 500 ms |
| Turn time, max (400 turns; 2026-10-18, creature AI #77, after objects #98) | 0.129 ms | 0.293 ms | 0.026 ms | 5 ms / 16 ms / 5 ms |
| Rest dusk to dawn, 7200 turns (2026-10-18, creature AI #77; the depths rest is cut short by an attack) | 269 ms | 66 ms | n/a | 500 ms |
| Frame time, 2560x1440 | 0.89 ms (about 1120 fps) | 0.89 ms (about 1120 fps) | n/a | 16.7 ms |
| Frame time, 1280x720 | 0.79 ms | 0.84 ms | n/a | 16.7 ms |

Moonlight (#82, 2026-10-18, same machine): the courtyard and a forest area at midnight under the
full moon (moon shadows instead of sun shadows) take at most 0.19 ms and 0.16 ms a turn; a rest
from dusk to dawn under the full moon in the forest area, with deer in view range so that every
turn is perceived, takes 218 ms (500 ms budget).

A rest perceives after every turn while a creature is within the view range plus one; farther
away, turns pass in batches between perceptions (no creature can come into view in them), which is
why the vast level rests fastest.

Creature AI (#77) is measured in the arena too, where a dozen creatures see, aim, and attack every
turn: 0.088 ms a turn on average, 0.202 ms at most (budget 5 ms). Its worst rest is a night in the
kobolds' camp behind a closed door while the fight in the pit goes on unseen and unheard, every
turn perceived because the fighters are within the view range: 353 ms for 7200 turns (budget
500 ms; the `fight` row of `bench-turns`). Creature sight is the costly part; it is computed only
for cells where a hostile stands, nearest first, with line of sight only out to that distance,
darkvision before light, and bounce light only when the rest of the light falls short (before
those, the same rest took about 1 s).

Full population (#326, 2026-10-25, same machine under load from other builds, load average
4 to 6): the `delf` row is a generated 120×60 cave floor of a fixture branch of danger 8 laid
down at its full population (80 creatures, the ceiling of the campaign design's §9.3), the
player with hit points enough to walk among them. Its turns take 0.59 ms on average and 1.36 ms
at most (budget 16 ms, the vast level's), so the ceiling of 80 stays. A night's rest there took
about 7.7 s for 7200 six-second turns (the hub's and the vast level's took 0.53 s and 0.59 s in
the same run): with 80 creatures some are always within the view range, so every turn was
perceived, and all 80 acted every turn (about 1 ms a turn, as the walk's).

Coarse turns while resting (#349, 2026-10-25, same machine, load average 6 to 7): far creatures
take their turns in strides of up to 20 steps while the player rests (spec §8 "Coarse turns
while resting", `world/stride.rs`), busy ones far away at 20 times the price, and a rest's turn
past the out-of-view stretch is looked at by the rules of sight for the watched creatures' cells
alone instead of a whole perception when nothing but movement happened. With them, and with
three changes that make every turn cheaper (`Actor::talent_mods` borrows a shared empty
`TalentMods` for an actor without talents instead of building one, a creature with nothing to
live through skips working out its healing paces in `World::live`, and a step away from beside
creatures checks adjacency before free-blow talents), the `delf` row's rest is judged again:
210 ms for 4962 turns (eight rests, each but the first cut short by a cave beetle wandering into
view; about 300 ms a night at that pace), against 7.7 s. The same run's other rests: hub 88 ms,
vast 60 ms, depths 52 ms, courtyard 94 ms, overworld 41 ms, forest 48 ms, the moonlit courtyard
and forest 92 ms and 46 ms, the arena fight 64 ms. Counted in instructions (`/usr/bin/time -l`,
which load does not skew), the `delf` rest went from about 123 billion to about 5 billion; a
profile of what is left shows ordinary creature turns near the player, `World::live` for every
actor at every moment the clock stops, and the rest's sight checks, in about equal parts.

Panoplies (#486, 2026-10-27, same machine): the `panop` row is the same 120×60 cave floor at
danger 40 whose population is `animated_panoply` alone, at full population: creatures that wear
and wield up to ten things each, whose magic every rule reads through `Mods::worn` (the animated
objects design's open performance risk, §12). Its turns take 1.0 ms on average and 2.2 ms at
most (budget 16 ms; `delf` 0.4 ms and 1.2 ms in the same run), so `Mods::worn` stays uncached.
Its rest is cut short at once (the panoplies see the player), so the row times the walk only.

Spell circles (#779, 2026-11-01, same machine): `bench-frames` has two more rows, `circle`, a
hall of 48 peaceful kobolds each buffed, hexed, or both and the player with a buffs' and a
hexes' circle, fifty circles in view (`xtask/src/bench_circles.rs`, the design's stress case,
docs/design/buff-circles.md §10.3), and `c-off`, the same hall with the Spell circles switch off;
their difference is the circles' own cost, printed after the table and not judged (it is the
difference of two best-of-five runs, whose noise is of the budget's size).

| Measurement | hub | vast | circle | c-off | fifty circles |
|---|---|---|---|---|---|
| Frame time, 2560x1440 | 0.893 ms | 0.902 ms | 0.990 ms | 0.940 ms | 0.050 ms (budget 0.2 ms) |
| Frame time, 2560x1440, an earlier run | 0.901 ms | 0.901 ms | 0.986 ms | 0.944 ms | 0.042 ms |
| Frame time, 1280x720 | | | 0.888 ms | 0.782 ms | 0.106 ms |

The circles' CPU part is an `O(circles)` loop a frame plus one mesh of a quad each; the GPU part
is one draw of fifty small quads. Within the budget at both sizes.

Frame time is nearly identical across levels and resolutions, which means the loop is CPU or
submission bound on this machine; the offscreen run at the default 60 Hz pacing is bounded by the
sleep granularity, not by rendering. All budgets are met with a large margin. Before lighting and
effects landed the same measurement gave 0.48 ms per frame; the cost roughly doubled. Re-measure on
a real Windows/Linux integrated GPU before confirming the budgets in the spec.

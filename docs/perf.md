# Performance budgets (spec §10)

Proposed budgets, to be confirmed on reference hardware: 60 fps at 1440p on an integrated GPU;
turn processing at most 5 ms in the hub and 16 ms in the vast level.

## Methodology

- **Turn time**: `cargo xtask bench-turns` (release build). Scripted walk of 400 turns per level
  through `warlock-core` only (no rendering); each `World::apply` that takes a turn is timed. The
  verdict compares the **max** turn time with the budget; the command fails when over budget.
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

Machine: Apple M5 Pro (integrated GPU, Metal), macOS 26.6.2, release profile, 2026-10-17.

| Measurement | Hub | Vast | Budget |
|---|---|---|---|
| Turn time, avg | 0.004 ms | 0.076 ms | |
| Turn time, max (400 turns) | 0.013 ms | 0.130 ms | 5 ms / 16 ms |
| Frame time, 2560x1440 | 0.48 ms (about 2080 fps) | 0.48 ms (about 2090 fps) | 16.7 ms |
| Frame time, 1280x720 | 0.48 ms | 0.48 ms | 16.7 ms |

Frame time is identical across levels and resolutions, which means the loop is CPU/submission
bound on this machine (the scene is a few full-window quads plus a small free-glyph mesh); the
offscreen run at the default 60 Hz pacing is bounded by the sleep granularity, not by rendering.
All budgets are met with a large margin. Re-measure after the lighting (#29/#19) and effects (#21)
work lands, and on a real Windows/Linux integrated GPU before confirming the budgets in the spec.

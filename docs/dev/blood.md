# Blood

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08e-blood-and-status.md. Keep it current with the code, as CLAUDE.md says.

Blood (spec §8 "Blood", §4.8; M17): the pure pattern is `warlock-core/src/blood.rs` (`Wound`:
`volume` by kill or big hit (a third of the maximum, or a critical), size `blood` percent,
overkill, critical, missile; `spatter`: pool, fan away from the attacker (±45° melee, ±30°
missiles), kill flecks, stopped by opaque cells, walls next to the victim stained on their face;
`Stain`: amount to `CAP` 100, colour and spill time mixed by amount, `faces` bits; `seed` hashes
turn, time, and actor ids with SplitMix: **never the gameplay stream**). `world/bleed.rs`
(`World::bleed`, called from `fight.rs::resolve` before `hurt`) writes `LevelState::blood` via
`LevelState::stain` (terrain `stains`, data; never on the overworld). No rule reads blood;
`blood_tests.rs` checks a fight is identical with and without it. Perception: `PerceptionView::blood`
(open cells in sight; walls only from a stained face; else `blood_memory` as last seen, updated
in `Perception::remember`). Data: creature `blood` (`"#rrggbb"` or `"none"`, default
`blood::RED`), size `blood`, terrain `stains`. App: `ui/blood.rs::stained` shifts a cell's fg/bg
toward the blood (dried by age) in `draw_map` before the shader lights it, so darkvision, memory
dimming, and palettes apply as for terrain; look mode prefixes the terrain with `Stain::word`.
The `blood` switch hides both. Tune by capturing the arena at `wait*8` (first kills) and `wait*40`
(after the brawl, the `arena-blood` golden), and on walls with the walk of the former
`blood-wall` golden (#301): `move:N*3 open:N move:N*2 move:E*36 open:S move:S*3 move:SW move:W*9
move:NW lantern kick:N*18` at `--scale 2`.

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
The `blood` switch hides both.

**By damage type** (#1255): `data/damage_types.toml` `blood` (default true) says whether a blow of
the kind spills any; `World::spill` checks `Blow::kind` (the kind that struck, the better of a
sword's two), so one hook covers melee, missiles, spells, and traps. False: fire, cold,
electricity, poison, necrotic.

**Char** (#1255, spec §8 "Char"): fire's sibling of blood. The pure pattern is
`warlock-core/src/scorch.rs` (`area`: each open cell its jittered share, walls beside on the face
toward it at `WALL` percent, half across a corner; `blow`: a struck creature's cell; `add`: amount
to the cap, faces joined, colour from the type's `char.light` to `char.heavy` by amount; `word`;
`seed` hashes turn, time, and the centre cell with a salt apart from blood's: **never the gameplay
stream**). It reuses `blood::Stain` for storage. `world/sear.rs` applies it: `sear_burst` (from
`spell.rs::cast_burst_reaching`, every caster: spell, scroll, wand, flask, trap, place),
`sear_area` (from `cast_cone`, and `zone.rs::lay_cells` for a fire zone), `sear_blow` (from
`bleed.rs::spill`, before the blood check); all write `LevelState::scorch` via
`LevelState::char` (terrain `stains`; never on the overworld). A damage type chars when it has a
`char = { light, heavy }` (fire). Perception: `PerceptionView::scorch` by blood's exact rules (one
`stained` closure in `perception.rs` computes both lists), `scorch_memory` in `remember`; the
`scorch` dump layer. App: `ui/scorch.rs::charred` shifts fg toward the char and the ground toward
a darker shade of it (`GROUND`: never lighter than the floor) before `blood::stained`, so blood
lies over char; look mode puts `scorch::word` before the blood's word. No rule reads it;
`sear_tests.rs` checks a fight with fire zones is identical with the char washed away. Tune on
the sunlit courtyard (`zone:fire@2,13,2 wait*11` repeated) and the trap hall's scrolls of
fireball (`move:E pickup` ×4, `identify`, `read:c:45,17`).

Tune by capturing the arena at `wait*8` (first kills) and `wait*40`
(after the brawl, the `arena-blood` golden), and on walls with the walk of the former
`blood-wall` golden (#301): `move:N*3 open:N move:N*2 move:E*36 open:S move:S*3 move:SW move:W*9
move:NW use_tool kick:N*18` at `--scale 2`.

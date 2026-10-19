# Ranged combat

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08c-combat.md. Keep it current with the code, as CLAUDE.md says.

Ranged combat (spec §8 "Ranged combat"; keys: `t` shoots or throws the Missile weapon slot's contents, `T` is the throw chooser, `f` unbound but rebindable; the slot `takes_weapons`) lives in `warlock-core/src/world/shoot.rs`: `fire`,
`throw`, and `creature_shoots` loose one missile (`loose`): it leaves its stack, makes its noise
once, flies along `Pos::ray` (the line the AI's `clear_shot` checks) over transparent cells as far
as the weapon's `range`, attacks whoever stands in its way (`World::missile_hits`, no noise) until
one is hit, and comes down (`come_down`) on top of the pile, unless it broke (`breaks`, one
gameplay draw on a hit) or the cell is not walkable. What is `lit` flies as its burning object and
starts a fire (`LevelState::fires`, saved): a light source in `LightCache::sources` while the clock
is before its end, cleaned up by `World::burn_out` at the start of `perceive`. Prices go through
`price_of`/`price_for` (shoot, plus `reload` for a launcher that `reloads`; throw), so burden
slows them too. `Event::Missile` carries the path; `perceive_events` cuts it to what the player
saw (a burning missile: everything in their line of sight). In the app, `ui/target.rs` is the
aiming input mode (`InputMode::Aim`: cursor start, Tab cycling, the line of fire as the player
knows it, `LastTarget`), `ui/missiles.rs` flies missiles on the free-glyph layer (glyphs turned
with `FreeGlyph::turn`, trail, embers via `particles::trail`, impact puff) and writes this frame's
`render::lighting::Transients` (the moving light of a burning missile, and landed lights held back
until it arrives; `draw_map`, object glow, and object particles hold back what is `arriving` too).

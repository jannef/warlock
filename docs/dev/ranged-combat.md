# Ranged combat

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08c-combat.md. Keep it current with the code, as CLAUDE.md says.

Ranged combat (spec §8 "Ranged combat"; keys: `t` shoots or throws the Missile weapon slot's contents, `T` is the throw chooser; the slot `takes_weapons`) lives in `warlock-core/src/world/shoot.rs`: `fire`,
`throw`, and `creature_shoots` loose one missile (`loose`): it leaves its stack, makes its noise
once, flies along `Pos::ray` (the line the AI's `clear_shot` checks) over transparent cells as far
as the weapon's `range`, attacks whoever stands in its way (`World::missile_hits`, no noise) until
one is hit, and comes down (`come_down`) on top of the pile, unless it broke (`breaks`, one
gameplay draw on a hit) or the cell is not walkable. What is `lit` flies as its burning object and
starts a fire (`LevelState::fires`, saved): a light source in `LightCache::sources` while the clock
is before its end, cleaned up by `World::burn_out` at the start of `perceive`. Prices go through
`attack_price` (shoot with the missile weapon, throw with nothing; priced as the shot begins, so
the missile leaving and the marks it earns change only the next) plus `reload_price` for a
launcher that `reloads`, so the stance and burden change them too. `Event::Missile` carries the path; `perceive_events` cuts it to what the player
saw (a burning missile: everything in their line of sight). A wand in the missile weapon slot is not
loosed: `fire` hands it to `World::zap` (`world/zap.rs`, docs/dev/spells.md "Wands"), which
casts its spell for a charge. In the app, `ui/target.rs` is the
aiming input mode (`InputMode::Aim`: cursor start, Tab cycling, the line of fire as the player
knows it, `LastTarget`; `AimKind::Read` aims a spell scroll too, `docs/dev/scrolls.md`; `AimKind::Zap` a wand; a spell, `AimKind::Ability`
of `Ability::Spell`, is aimed by its shape at the player's own reach, `docs/dev/spells.md`), `ui/missiles.rs` flies missiles on the free-glyph layer (glyphs turned
with `FreeGlyph::turn`, trail, embers via `particles::trail`, impact puff) and writes this frame's
`render::lighting::Transients` (the moving light of a burning missile, and landed lights held back
until it arrives; `draw_map`, object glow, and object particles hold back what is `arriving` too).

Creatures' throws (spec §8 "Ranged combat": "Creatures' throws", #457) live in
`warlock-core/src/world/hurl.rs`: a creature type's `[creature.throw]` (`content/throw.rs`:
`ThrowSpec` resolved like a natural attack, its `object` once objects are known,
`Content::load_throws`; `Throw` on `CreatureDef::throw`). `World::hurls_at` is the AI's check (not
next to the target, in range, `clear_shot`), `World::creature_hurls` prices the throw
(`attack_price` of `throw`, which `Content::action_cost` gives the throw's `cost`), makes the
noise, and flies a `shoot::Flight` with `lands: None` and `breaks: 0` through `World::fly`, the
flight `loose` uses (so no gameplay draw beyond the attacks' and nothing comes down).
`Actor::hurled` is its profile (`Profile::thrown`, the `thrown` rank, affixes, stance);
`status.rs::on_hit_of` gives a creature's thrown attacks its throw's `on_hit`. The flight is an
`Event::Missile` of the throw's object (the `boulder`, which nothing else places), shown like an
arrow; `perceive_events` says the throw through `hurl::throw_messages` (the thrower's announcement,
then the blow with `End::Trap(noun)` as subject). Tests: `world/hurl_tests.rs`.

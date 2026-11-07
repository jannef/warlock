# Ranged combat

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08c-combat.md. Keep it current with the code, as CLAUDE.md says.

Ranged combat (spec §8 "Ranged combat"; keys: `t` shoots or throws the Missile weapon slot's contents, `T` is the throw chooser; the slot `takes_weapons`) lives in `warlock-core/src/world/shoot.rs`: `fire`,
`throw` (a potion it hands to `World::throw_flask`, `world/flask.rs`: it flies and shatters, docs/dev/potions.md "Flasks"), and `creature_shoots` loose one missile (`loose`): it leaves its stack, makes its noise
once, flies along `Pos::ray` (the line the AI's `clear_shot` checks) over transparent cells as far
as the weapon's `range`, attacks whoever stands in its way (`World::missile_hits`, no noise) until
one is hit (or caught: Diamond soul's catch sets `World::caught` and the flight ends on that
cell, a miss; `world/ascetic.rs`, a trap's `trap_shot` alike), and comes down (`come_down`) on
top of the pile, unless it broke (`breaks`, one gameplay draw on a hit) or the cell is not
walkable. What is `lit` flies as its burning object and
starts a fire (`LevelState::fires`, saved): a light source in `LightCache::sources` while the clock
is before its end, cleaned up by `World::burn_out` at the start of `perceive`. Prices go through
`attack_price` (shoot with the missile weapon, throw with nothing; priced as the shot begins, so
the missile leaving and the marks it earns change only the next) plus `reload_price` for a
launcher that `reloads`, so the stance and burden change them too. `Event::Missile` carries the path; `perceive_events` cuts it to what the player
saw (a burning missile: everything in their line of sight). A wand in the missile weapon slot is not
loosed: `fire` hands it to `World::zap` (`world/zap.rs`, docs/dev/spells.md "Wands"), which
casts its spell for a charge; a rod there goes to `World::use_rod` (`world/rod.rs`, "Rods"), which does its effect for the user's mana. In the app, `ui/target.rs` is the
aiming input mode (`InputMode::Aim`: cursor start, Tab cycling, the line of fire as the player
knows it, `LastTarget` (creatures only, never an empty cell; `Aim::start` skips candidates beyond `Aim::range`, Tab does not); `AimKind::Read` aims a spell scroll too, `docs/dev/scrolls.md`; `AimKind::Zap` a wand or a rod; a spell, `AimKind::Ability`
of `Ability::Spell`, is aimed by its shape at the player's own reach, `docs/dev/spells.md`), `ui/missiles.rs` flies missiles on the free-glyph layer (glyphs turned
with `FreeGlyph::turn`, trail, embers via `particles::trail`, impact puff) and writes this frame's
`render::lighting::Transients` (the moving light of a burning missile, and landed lights held back
until it arrives; `draw_map`, object glow, and object particles hold back what is `arriving` too).

A target dummy (`world/dummy.rs`, docs/dev/weapon-skills.md "Target dummies") stops a missile in
`World::fly`, hit or miss (pierce or not): the flight ends on its cell and
`World::dummy_catches` takes the place of `breaks` and `come_down`: destroyed by the dummy's
`breaks` (one gameplay draw, only when the missile's own chance and the dummy's are both above
0), else down on a walkable cell beside it holding no actor (one gameplay draw), or on its own
cell when there is none. Tests: `world/dummy_tests.rs`.

Shots by distance (spec §8 "Ranged combat": "Distance", #956): `data/ranged.toml` (optional;
`content/ranged.rs`, `RangedRules`, `Content::ranged`) holds the `adjacent` penalty and the
`[[band]]`s. `World::fly` asks `flight_profile` for the profile it attacks each creature with, by
that creature's cell, so bystanders are judged by their own distance; only `AttackKind::Ranged`
pays. `RangedRules::to_hit_at` is the one measure: adjacent (Chebyshev 1) wins, else the farthest
band whose share of `radius2(range)` the squared distance exceeds. Parry: `World::resolve` reads
`from` and `to` as it rolls and passes `close` to `target_shot`, so a shot from beside its target
keeps Parry (`target_against`, every other caller, passes `false`). No state, no new draws, no AI
change (a `Combat::Ranged` creature keeps shooting when adjacent). A creature's AI range is its
`range` capped by its launcher's (`ai.rs`), while the bands are shares of the launcher's range
alone, so an archer whose own `range` is far below its bow's seldom reaches the far band (a
hobgoblin archer, `range` 8 with a long bow of 15, only ever pays the first band, at 8 cells).
Tests: `world/shot_range_tests.rs` (with the same draws, a shot at a band strikes as one at a clean
distance with the penalty added to its to-hit), `content/ranged.rs`.

Creatures' throws (spec §8 "Ranged combat": "Creatures' throws", #457) live in
`warlock-core/src/world/hurl.rs` (the earth elementals' stones too, the dread design §5.3): a
creature type's `[creature.throw]` (`content/throw.rs`:
`ThrowSpec` resolved like a natural attack, its `object` once objects are known,
`Content::load_throws`; `Throw` on `CreatureDef::throw`). `World::hurls_at` is the AI's check (not
next to the target, not inside an opaque cell (an incorporeal thrower comes out of the wall
first), in range, `clear_shot`), `World::creature_hurls` prices the throw
(`attack_price` of `throw`, which `Content::action_cost` gives the throw's `cost`), makes the
noise, and flies a `shoot::Flight` with `lands: None` and `breaks: 0` through `World::fly`, the
flight `loose` uses (so no gameplay draw beyond the attacks' and nothing comes down).
`Actor::hurled` is its profile (`Profile::thrown`, the `thrown` rank, affixes, stance);
`status.rs::on_hit_of` gives a creature's thrown attacks its throw's `on_hit`. The flight is an
`Event::Missile` of the throw's object (the `boulder`, which nothing else places), shown like an
arrow; `perceive_events` says the throw through `hurl::throw_messages` (the thrower's announcement,
then the blow with `End::Trap(noun)` as subject). A throw whose object is a flask (a bomber,
#1230; `ThrowSpec::damage` optional then, `load_throws` refusing a damage-less throw of anything
else) takes the one branch in `creature_hurls`: after pricing, noise, and `aim_at`, "The kobold
alchemist hurls a flask." (`gaze.rs::end_of`'s subject) when the player sees the thrower, then
`World::shatter_flask(id, object, (pos, at, range))` instead of `fly` (`docs/dev/potions.md`
"Flasks"). Tests: `world/hurl_tests.rs`, `world/creature_flask_tests.rs`.

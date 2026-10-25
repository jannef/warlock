# Hearing

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/05-visibility.md. Keep it current with the code, as CLAUDE.md says.

Hearing (spec §5 "Hearing") lives in `warlock-core/src/noise.rs`. `noise::noise(content, level,
actor, burden, Act)` is the one place an action's loudness is decided (creature `footsteps` ×
terrain `step_noise` percent × the burden's `footsteps()` percent for `Act::Step`, the door's
`toggle_noise` for `Act::OpenDoor`/`CloseDoor`, `VOICE_NOISE` for `Act::Mutter` and for
`Act::ThrowVoice(at)`, which is made on that cell, not the actor's, `HOWL_NOISE` for `Act::Howl`
(Hound's bay, #270: `SoundKind::Howl`, "a howl", `h` in the sound layer; `noise::hearers` tells
the howler who hears it); `make_noise` passes the actor's
`Load::penalty`):
a new noisy action gets an `Act` variant there, and its call site in the feature's module (the makers are `world/hearing.rs`) is one
`self.make_noise(actor, Act::…)` where the action happens. `noise::make` delivers it at once to
every other actor on the level whose `hearing` threshold it reaches (`propagate`: integer Dijkstra,
10 a step, 14 diagonally, plus terrain `muffle`; solid terrain blocks unless it has a muffle),
pushing a `Heard { kind, at, loudness }` onto `Actor::heard`. A creature's list is what it heard
since it last acted and is cleared as it acts in `World::creature_turn` (the AI reads it there
before clearing); the player's is cleared in `World::run_turn` once the player has acted, and
becomes `PerceptionView::sounds()` (sources not visible only; kind, `Volume`, and `rough_place`,
never the actor; a walker's several steps within one turn are one sound; a listener with
Watchful's `exact_footsteps`, #267, places footsteps at their own cell, and the ranger's Tracker,
`exact_sounds` (#244), every sound: `noise::perceive`'s `exact`, a `noise::Exact`). In
`noise::noise` the thief's and ranger's talents (#244) quiet the player: Camouflage's
`sky_footsteps_percent` (a step onto an unroofed cell of a sky-exposed level, summed with
`footsteps_percent`), Hide in plain sight's `silent_unburdened` (no footsteps while the burden is
Unburdened) and `attack_noise_percent` (attacks, the one percent they take). `World::hear` logs news
(`noise::news` against `WorldState::recent_sounds`) as messages, which stop a rest (voices are said
with their words by `World::say_mutters` instead, as notes). The app draws
`PerceptionView::sounds()` in `ui/sounds.rs` (a still `◌` marker, always: it is information; the
ripple only with `glyph_animation`), and there too where the player's own thrown voice sounded
(`Event::VoiceThrown` in the last outcome, as a loud sound). Throwing one's voice (#235) is
`world/voice.rs` (`World::throw_voice`, `VOICE_RANGE`; tests `world/voice_tests.rs`): refused
on the overworld, at the player, out of reach, or at a cell not `sees_actors_at` or solid; else
one `make_noise(player, Act::ThrowVoice(at))`, so the AI's listening step sends creatures with
enemies that do not see the cell there. Sounds carry at most `noise::MAX_REACH` (30) cells, under
`VIEW_RADIUS`, which keeps rest batching (`out_of_view_for`) exact (a const assert guards it).

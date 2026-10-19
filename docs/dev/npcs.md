# NPCs, dialogue, and making way

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08b-creatures.md. Keep it current with the code, as CLAUDE.md says.

NPCs, dialogue, and making way (spec §8 "Dialogue" and "Displacement", #146, #147):
`content/dialogue.rs` loads `data/dialogue.toml` (per creature type, or per unique `npc` id a level
legend entry gives a creature, `Actor::npc`; `Content::dialogue_of` prefers the npc's): rotating
`lines`, `greet`, `[[dialogue.topic]]`s (`id`, `name`, `reply`), and `mutter` (chance per action,
verb, lines). `world/talk.rs` is the one place talk happens: `World::chat` (`Action::Chat`, cost
`chat`), `World::ask` (`Action::Ask`, no time), `World::topics` (the one source of what an NPC can
be asked about: quests, #148, add their topics there and answer them in `ask`), `rotation` (each
line once a round in a per-speaker shuffled order, never twice in a row; `Actor::talk` counts what
was said; with `Actor::npc` saved and hashed), mutters (`World::mutter` after each `creature_turn`, a hash roll that never
draws on the gameplay stream; `Act::Mutter` noise; `say_mutters` logs what the player heard, with
the words, only as notes that never stop a rest), and making way: moving into a creature that is
not hostile swaps places (`swap_places`; the creature takes its old cell as its `Mind::post` and
`ai.rs` walks it back, waiting beside the post while someone stands on it, then on with its
route) unless `World::refusal` says why not (the type's `budge`: `never`, or `off_post` while its
mind is calm, ready for shopkeepers, #145; it cannot move; or it is two sizes or more larger:
sizes are listed smallest first). What an actor is called in messages comes from
`World::called` ("the gardener"): named shopkeepers go there. Creatures that talk and fight no one
never stop a rest by coming into view (`World::disturbs_rest`).

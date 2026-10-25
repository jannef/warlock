# NPCs, dialogue, and making way

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08b-creatures.md. Keep it current with the code, as CLAUDE.md says.

NPCs, dialogue, and making way (spec §8 "Dialogue" and "Displacement", #146, #147):
`content/dialogue.rs` loads the `dialogue` table (`data/dialogue.toml`, the shopkeeper's, and each
campaign's `data/dialogue/`: the showcase's hermit, gardener, and bard) (per creature type, or per unique `npc` id a level
legend entry gives a creature, `Actor::npc`; `Content::dialogue_of` prefers the npc's): rotating
`lines`, `greet`, `greet_evil`/`greet_good` (cold lines for a character the speaker scorns for
their alignment, said in turn instead of the greeting and lines: `World::cold_greeting`,
`docs/dev/alignment.md`), `[[dialogue.topic]]`s (`id`, `name`, `reply`), `[[dialogue.service]]`s
(`id`, `name`, `price`, `does`, `reply`, `nothing`; topic and service ids share one namespace, at
most nine of them together, and a service needs `data/shops.toml` for its currency), and `mutter`
(chance per action, verb, lines). `world/talk.rs` is the one place talk happens: `World::chat` (`Action::Chat`, cost
`chat`), `World::ask` (`Action::Ask`, no time), `World::topics` (the one source of what an NPC can
be asked about: quests, #148, add their topics there and answer them in `ask`; services follow
the topics with `TopicView::price`), `rotation` (each
line once a round in a per-speaker shuffled order, never twice in a row; `Actor::talk` counts what
was said; with `Actor::npc` saved and hashed), mutters (`World::mutter` after each `creature_turn`, a hash roll that never
draws on the gameplay stream; `Act::Mutter` noise; `say_mutters` logs what the player heard, with
the words, only as notes that never stop a rest), and making way: moving into a creature that is
not hostile swaps places (`swap_places`; the creature lands on the player's old cell as any
step lands: `World::tread` on ice and `World::spring_trap`, what that costs it on top taken off
its energy; it takes its old cell as its `Mind::post` and
`ai.rs` walks it back, waiting beside the post while someone stands on it, then on with its
route) unless `World::refusal` says why not (the type's `budge`: `never`, or `off_post` while its
mind is calm, ready for shopkeepers, #145; it cannot move; or it is two sizes or more larger:
sizes are listed smallest first). What an actor is called in messages comes from
`World::called` ("the gardener"): named shopkeepers go there, and so do unique characters with a
proper name (an npc dialogue's `name`, `Dialogue::name`, #336: "Sister Hild", no article; also
`Content::speaker_name`/`speaker_called` for quest texts and the wiki). Presentation knows a seen
creature's dialogue from `SeenActor::dialogue` (`Content::dialogue_of`, not serialized): look
mode names an npc by its proper name with the services it sells in words ("Sister Hild
(unhurt), who cures and mends for gold, on plank floor."), and the dialogue screen's title is
the name. Creatures that talk and fight no one
never stop a rest by coming into view (`World::disturbs_rest`).

**Services** (#329, the campaign design §6.1; spec §8 "Dialogue"): `world/service.rs`.
`World::ask` hands a topic id it is not a quest's to `World::ask_service` before the dialogue's
topics; it checks there is something to do (`prayer_cures`: a status or disease, showing or not,
whose `cured_by` lists `service::PRAYER`; or stat damage, with no disease showing for a mend),
then the gold (`service_price`: the data's price, halved rounded up with `TalentMods::services`,
Pilgrim), pays through `shop.rs`'s `hand_over` (the purse code `pay` uses; the coins go to the
speaker), and does it: `World::cure(player, "prayer")` (every disease in `data/diseases.toml`
lists `prayer`; with Pilgrim a point of each damaged stat mends after, unless a disease still
shows) or `World::mend` (`StatDamage::mend_all`). The reply goes through `World::answer`, so the
dialogue screen reopens on the service. Refusals take no time; a service done takes `chat`'s.
Nothing is saved that was not before, and no stream is drawn. Tests: `world/service_tests.rs`
(a fixture sister on the built-in content; prices, refusals, Pilgrim, the replay and the save),
the screen's row in `ui/dialogue_screen.rs`; the wiki's table is `xtask/src/wiki/services.rs`.

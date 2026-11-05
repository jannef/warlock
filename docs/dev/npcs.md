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
sizes are listed smallest first). A player trained in Escape artist is not refused for the
`budge` (`World::wriggle`, `world/wriggle.rs`, #1032): an automatic check instead, Dexterity plus
the `escape` points against 11 plus the creature's Willpower modifier; never a shop's keeper (`Shop::keeper`, the refusal stands untouched); tests `wriggle_tests.rs`. What an actor is called in messages comes from
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
whose `cured_by` lists `service::PRAYER`; or stat damage or drained points, with no disease showing for a mend),
then the gold (`service_price`: the data's price, halved rounded up with `TalentMods::services`,
Pious), pays through `shop.rs`'s `hand_over` (the purse code `pay` uses; the coins go to the
speaker), and does it: `World::cure(player, "prayer")` (every disease in `data/diseases.toml`
lists `prayer`; with Pious a point of each damaged stat mends after, unless a disease still
shows) or `World::mend` (`StatDamage::mend_all`, then `World::restore_drained`: drained points back, `docs/dev/stats.md` "Drain"),
or, for `identify` (#432), `World::identify` on each of `World::unidentified` (the stacks
`Lore::knows` does not), priced per stack: `service_price` (Pious only for `cure`/`mend`) times
`service_items`, which `World::topics` also puts in `TopicView::items`; the dialogue screen
lists such a service "(N gold each)" and, with things to pay for, asks first in the one dialog
with the total (`DialogueScreen::pay_dialog`). The reply goes through `World::answer`, so the
dialogue screen reopens on the service. Refusals take no time; a service done takes `chat`'s.
Nothing is saved that was not before, and no stream is drawn. Tests: `world/service_tests.rs`
(a fixture sister on the built-in content; prices, refusals, Pious, the replay and the save),
`world/identify_service_tests.rs` (a fixture appraiser: the count, the refusals, magic and
scroll types identified, the replay and the save), Wulfrun's in `world/ceaptun_tests.rs`,
the screen's row in `ui/dialogue_screen.rs`; the wiki's table is `xtask/src/wiki/services.rs`.

**A trader's offer** (#808, docs/design/overworld-encounters.md §15 Q7; spec §8 "Dialogue"):
`content/offer.rs` (`TradeOffer`, a dialogue's `[dialogue.offer]`: `id`, `name`, `reply` said on
a sale, `sold` said after, `level = { bonus, least }`, `affixes` as `[count, weight]` pairs,
`price = { affix, unique }`, `uniques = { chance, items }` with the chance in basis points),
`world/wares.rs`. Its id shares the topic namespace (the nine counts it); load checks: an id,
not a topic's or service's, the shops' currency, lines, `least` 1 to 99, counts 1 to 6 with some
weight, `price.affix` ≥ 1, the chance ≤ 10 000, `price.unique` ≥ 1 with uniques, each unique
known, of the dialogue's campaign or shared, once; and, after everything that places objects
loaded, `offer::check_placements`: a unique an offer names is placed by no other content
(`Uniques::is_placed`), so it comes only through the offer. `World::stock_wares` (one line in
`chat` and in `ask`, after `listens`) makes a trader's `Wares` once, from `Rng::stream(seed,
"wares/<actor id>")`: one `below(10_000)` against the chance (only with uniques); made, one
`below` among its uniques not in `WorldState::uniques_offered` (then added); else
`magic::loot::favoured` at the player's level + `bonus`, clamped to `least`..99 (a base that can
be magic, not ammunition or a stack, by frequency in id order; the count by weight;
`loot::heirloom`, so rare). The item is identified (a trader names his wares). Price: a unique
`price.unique`; else `Stack::value` + `price.affix` × affixes × item level; no favour.
`World::topics` lists it after the services (`World::offer_topic`) with `TopicView::price` and
`TopicView::ware` (the item listed, then its affix and power lines from `describe::lines`,
joined by ". ") while for sale. `World::ask_offer` (in `ask` after `ask_service`): sold, the
`sold` line (no time); without the gold "You cannot pay." (no time); else `hand_over`, "You pay
the pedlar N gold for X.", the item into the pack, `Wares::item` cleared, `train_deal`, the
`reply`, `chat`'s time. Saved: `WorldState::wares` (`BTreeMap<ActorId, Wares>`) and
`uniques_offered`, both left out while empty (old saves and replays unchanged). The `actors`
dump layer shows `wares ITEM for N gold` / `wares sold (N gold)` and the player's `uniques
offered`. The app: `DialogueScreen` opens the one dialog for a topic with a `ware`
(`offer_dialog`: the speaker's name, "ITEM. N gold, take it or leave it.", "Buy it" sending the
ask and "Leave it" closing it; short of gold, " You have only N." and "Leave it" alone).
Leaving it is nothing. Tests: `world/wares_tests.rs` (determinism, the favoured roll's rarity,
count, level, and price, buy, sold, can't pay, the uniques by a certain chance each once then a
find, the three's slots and the placement refusal, the load checks, a replay and a save),
`ui/dialogue_screen.rs::an_offer_asks_buy_it_or_leave_it` (snapshots `offer-buy`,
`offer-short`); the wiki's `npcs.md` "The pedlar's offer" and the uniques table's offer column
(`xtask/src/wiki/uniques.rs::offered_by`).

**Rumoured ways** (the owner's request of 2026-10-27; spec §8 "Dialogue"): `world/rumour.rs`.
A topic's optional `route` (`Topic::route`, waypoints `[x, y]` on the campaign's overworld;
`Topic::route_cells` joins them with `Pos::line_between`) is checked when content loads
(`Content::check_topic_routes`, one line in `Content::load`: a campaign's dialogue only, two
waypoints or more, every cell walkable on its overworld). `World::ask` calls
`World::mark_rumoured_way` after a dialogue topic's reply: it makes the overworld if the run has
none yet (`World::visit`), runs `fov::compute_fov` with `daylight::SIGHT_DAY` from every route
cell over the overworld's opacity, and writes each cell seen into `LevelState::memory` from the
terrain grid, saying `rumour::MARKED` if anything was new. The terrain grid holds a hidden
location's plain terrain, so nothing hidden leaks (`docs/dev/overworld.md`); nothing else is
read or written (no objects, traps, or stairs links; no time; no stream; no new saved state).
Why waypoints and not a computed path: the way is the speaker's words, drawn by hand along the
roads, and checking it is a load-time walk over its cells; a pathfinder would need its own costs
and could drift off the road with any map edit. Godgifu's five topics carry routes from the town
cell (18, 28). Tests: `world/rumour_tests.rs` (marks after the ask and not before, a corridor and
not the map, idempotent, no time, hidden locations stay hidden while Cealdhlāw shows, the
`memory` and `locations` layers back on the map, a replay, the validation).

**Saying words** (#878, docs/design/prestige-classes.md §6; spec §8 "Dialogue", "Saying
words"): `words.rs` (`normalise`, pure: Unicode lower case, non-alphanumerics to spaces, runs
collapsed, trimmed; idempotent; `MAX_SAID` 40), `content/word.rs` (`[[dialogue.word]]`: `phrase`,
its normalised `said`, `reply` lines, `does` = `calls`/`quest`/`departs`; the dialogue's
`unheard`, `speaks_to`, `silent`; `resolve` checks them inside `dialogue.rs`'s parse, one call,
and `Content::check_words`, one line in `Content::load` after the quests, checks a `quest` is
the dialogue's campaign's), `world/words.rs` (`World::say_to`, `Action::Say`'s one line in
`World::apply`: refusals as `chat`'s through `listens`; the first word whose `said` equals the
normalised words and whose `does` can be done; `do_word` says the reply lines, the last through
`World::answer` (so the dialogue screen reopens on it) unless the word departs, then
`word_quest` (`World::give`, only when the quest topic would be listed for its giver now, not
active, and the player's level is met) and `World::departs` (factored out of
`World::ally_departs`, `world/ally.rs`); no match: `unheard` through `answer`; `chat`'s price
either way). `speaks_to` lives in `talk.rs`: `World::speaks_to(dialogue)` (the player's band
against the range); outside it `listens` says `silent` (so `chat`, `ask`, and `say_to` all
refuse alike) and `World::topics` is empty. `Conversation::words` (an event, not saved) is set
by `answer` when the dialogue has words and speaks to the player; `screens::converse` opens the
dialogue screen on it too. `calls` (#879): `Content::check_words` checks it names a calling of
the dialogue's campaign (or, for a dialogue outside every campaign, one outside every campaign);
`World::can_do` asks `World::calling_open`, so a word whose calling is not open is unheard (a
warrior saying the oath gets the `unheard` line); `do_word` makes the call (`World::call`) after
the reply, before `quest` and `departs`. The level-up's news ("Welcome to level 31.") is said as
the action ends, after the departure line. The words in content: the Strange Spirit's oath
(`campaigns/ascmearc/data/dialogue/spirit.toml`, `does = { calls = "hunter", departs = true }`,
`speaks_to = { min = "kind" }`) and the test world's hermit (`calls = "hunter"`, no `departs`).
Look mode says ", who waits" after a creature whose dialogue has words and no topic, service, or
offer (`ui/look.rs::waits`: "a strange spirit (unhurt), who waits"). Nothing
new is saved; nothing is drawn; no marks. The app: `ui/text_field.rs` (`TextField`, out of the
Name step), `DialogueScreen`'s fixed `s` row and field (`SAY_KEY`; no `KeyCommand`). Tests:
`words.rs`, `world/words_tests.rs` (fixture content: a match in any case, unheard and no
lock-out, a quest given once, departs, `silent` with no topics or row, the refusals, no draws,
a replay and a save, the load checks), `action.rs` (the text), `ui/text_field.rs`, and the
screen snapshots `dialogue-say-row`, `dialogue-say-field`, `dialogue-say-field-typed`,
`dialogue-say-reply` (`screen_snapshots.rs::saying_words`, the hermit given a word).

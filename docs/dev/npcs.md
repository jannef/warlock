# NPCs, dialogue, and making way

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08b-creatures.md. Keep it current with the code, as CLAUDE.md says.

NPCs, dialogue, and making way (spec §8 "Dialogue" and "Displacement", #146, #147):
`content/dialogue.rs` loads the `dialogue` table (`data/dialogue.toml`, the shopkeeper's, and each
campaign's `data/dialogue/`: the showcase's hermit, gardener, and bard; the Ascmearc's people
by place in `docs/dev/campaign.md`, among them Līctūn's sexton, Old Dudda (`dudda`, #1258), who
has only rotating lines and mutters) (per creature type, or per unique `npc` id a level
legend entry gives a creature, `Actor::npc`; `Content::dialogue_of` prefers the npc's): rotating
`lines`, `greet`, `greet_evil`/`greet_good` (cold lines for a character the speaker scorns for
their alignment, said in turn instead of the greeting and lines: `World::cold_greeting`,
`docs/dev/alignment.md`), `[[dialogue.topic]]`s (`id`, `name`, `reply`), `[[dialogue.service]]`s
(`id`, `name`, `price`, `does`, `reply`, `nothing`; topic and service ids share one namespace,
and a service needs `data/shops.toml` for its currency), and `mutter` (chance per action, verb,
lines). The loader checks only what resolves and what the code needs (#1205): ids once and
without `:` or whitespace (`ask:E:ID`), at least one line to rotate and to mutter, `silent` with
`speaks_to`; no length, count, or style limits (the dialogue screen asks topics past nine by the
arrow keys and cuts what overflows). `world/talk.rs` is the one place talk happens: `World::chat` (`Action::Chat`, cost
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
the name. An escort quest's character, chatted with while the quest is active, joins the player instead of
greeting them (`World::escort_joins`, the one line in `chat`; `docs/dev/quests.md`). Creatures that talk and fight no one
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
`Lore::knows` does not), priced per stack: `service_price` (Pious only for `cure`/`mend`; moved by `shop::haggling` unless the service is `temple = true`, #1031) times
`service_items`, which `World::topics` also puts in `TopicView::items`; the dialogue screen
lists such a service "(N gold each)" and, with things to pay for, asks first in the one dialog
with the total (`DialogueScreen::pay_dialog`). The reply goes through `World::answer`, so the
dialogue screen reopens on the service. Refusals take no time; a service done takes `chat`'s.
Nothing is saved that was not before, and no stream is drawn. Tests: `world/service_tests.rs`
(a fixture sister on the built-in content; prices, refusals, Pious, the replay and the save),
`world/identify_service_tests.rs` (a fixture appraiser: the count, the refusals, magic and
scroll types identified, the replay and the save), Wulfrun's in `world/ceaptun_tests.rs`,
the screen's row in `ui/dialogue_screen.rs`; the wiki's table is `xtask/src/wiki/services.rs`.

**Goodwill** (#1247; spec §8 "Dialogue": "Goodwill"): `content/town.rs` (`TownDef`, a
campaign's `data/towns/`, one file per town: `id`, `name`, `people`, `goodwill` in percent,
`told`; well-formedness only, #1205: a name in `people` that speaks no dialogue just matches
nobody) and `world/goodwill.rs`. `Content::town_of(dialogue)` is the town whose `people` name
the speaker (an npc id, or a creature id for a type's dialogue). `World::goodwill(town)` sums
`QuestRecord::completed` over the quests whose `giver` is one of its people: derived, never
saved, so the save and the state hash are unchanged. `World::goodwill_off(dialogue)` is that
times the town's `goodwill`, at most 100; `service_price(dialogue, service)` takes it off the
haggled price last (`goodwill::discounted`: rounded up, so free only at 100%). `ask_service`
with a price of 0 hands nothing over and says and trains nothing; the dialogue screen lists
such a service "(free)" and asks no pay dialog. `World::complete_saying` calls
`tell_goodwill(giver)` after "Quest complete" (the one line in `quest.rs`). Tests:
`world/goodwill_tests.rs` (Cēaptūn: the rounding, a quest done for the reeve, every completion
and the Gilde's counting but Holtwīc's not, the cap and a free cure), the screen's row in
`ui/dialogue_screen.rs`; the wiki's table is `xtask/src/wiki/services.rs::towns`.

**A trader's offer** (#808, docs/design/overworld-encounters.md §15 Q7; spec §8 "Dialogue"):
`content/offer.rs` (`TradeOffer`, a dialogue's `[dialogue.offer]`: `id`, `name`, `reply` said on
a sale, `sold` said after, `level = { bonus, least }`, `affixes` as `[count, weight]` pairs,
`price = { affix, unique }`, `uniques = { chance, items }` with the chance in basis points),
`world/wares.rs`. Its id shares the topic namespace; load checks (only what resolves and what
the code needs, #1205): an id without `:` or whitespace, not a topic's or service's, the shops'
currency, `least` at most `MAX_ITEM_LEVEL` (the draw clamps from it to the top), counts at most
6 (an item's affixes), each unique known, of the dialogue's campaign or shared (no other content places them: the
data author's promise). `World::stock_wares` (one line in
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
find, the three's slots, the load checks, a replay and a save),
`ui/dialogue_screen.rs::an_offer_asks_buy_it_or_leave_it` (snapshots `offer-buy`,
`offer-short`); the wiki's `npcs.md` "The pedlar's offer" and the uniques table's offer column
(`xtask/src/wiki/uniques.rs::offered_by`).

**Rumoured ways** (the owner's request of 2026-10-27; spec §8 "Dialogue"): `world/rumour.rs`.
A topic's optional `route` (`Topic::route`, waypoints `[x, y]` on the campaign's overworld;
`Topic::route_cells` joins them with `Pos::line_between`) is checked when content loads
(`Content::check_topic_routes`, one line in `Content::load`: a campaign's dialogue with an
overworld only; the waypoints are the author's, #1205). `World::ask` calls
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

**Callings by topic** (#1174, which removed #878's saying words; spec §8 "Dialogue",
"Callings"): a `[[dialogue.topic]]` may have `calls` (a prestige class's calling of the
dialogue's campaign, or, outside every campaign, one outside every campaign:
`Content::check_topic_callings`, one line in `Content::load` after the callings) and `departs`
(a creature type's dialogue only, checked in `dialogue.rs`'s parse). `World::topics` lists such
a topic only while `World::calling_open`; `World::ask` hands it to `World::ask_calling`
(`world/calling.rs`), which says the reply (through `World::answer`, or as a message only when
it departs, so the screen does not reopen on one gone), makes the call (`World::call`), then
`World::departs` (`world/ally.rs`), and takes a chat's time; not open, it is "knows nothing
about that" and no time. The level-up's news ("Welcome to level 31.") is said as the action
ends, after the departure line. In content: the Strange Spirit's oath
(`campaigns/ascmearc/data/dialogue/spirit.toml`, `calls = "hunter"`, `departs = true`, with
`speaks_to = { min = "kind" }`) and the test world's hermit (`calls = "hunter"`, no `departs`).
`speaks_to` lives in `talk.rs`: `World::speaks_to(dialogue)` (the player's band against the
range); outside it `listens` says `silent` (so `chat` and `ask` refuse alike) and
`World::topics` is empty. Look mode says ", who waits" after a creature whose dialogue's only
topics are callings', with no service or offer (`ui/look.rs::waits`: "a strange spirit
(unhurt), who waits"). Nothing new is saved; nothing is drawn. Tests:
`world/strange_spirit_tests.rs` (the spirit's silence and its topic, the oath asked, a replay
and a save, the hermit's topic offered to a full bard only), `world/calling_tests.rs`.

## Speech by race and station (#1392)

Spoken text (topics, replies, services, rotation lines, mutters, quest text, arrival lines, cries
with words) is written in the speaker's own register: commoners Cockney, the rich posh, dwarves
Geordie (#1391), and **elves Brummie** (the Birmingham accent and dialect). Readability comes
first and the accent is hinted: "bab", "ar" (yes), "tara-a-bit" (goodbye), "yow" (you), "summat",
"gambol" (a forward roll), "cob" (a bread roll), "bostin" (great), "babby", "ow am ya?", and the
vowels only lightly ("loike", "noice"); **halflings and gnomes Scouse** (the Liverpool accent
and dialect), hinted the same way: "la", "lad", "der" (there), "made up" (delighted), "boss"
(great), "sound", "me" for my, "ye" for you, "giz" (give us), "kecks" (trousers), "scran" (food),
"arl feller" (old man); and **goblins, kobolds, and ratfolk a very heavy Irish brogue**, the one
accent laid on thick: "ye", "yer", "meself", "tis", "sure an'", "begorrah", "eejit", "feckin'",
"grand", "craic", "wee", "me oul' da", "would ye ever", "th'" for the, "t'" for to, "ould" for old,
"tink" for think, "dat" for that; and **drow Middle English**, Chaucer's register made readable:
"thou", "thee", "ye", "hast", "dost", "nat" (not), "eek" (also), "ywis" (truly), "anon", "certes",
"sooth", "-eth" verbs ("he cometh"), "y-" past participles ("yslain"), "quod she" (said she). Ids, topic names, and placeholders never change with the
accent. The rule binds speakers whose data says they are of that race (`race = "elf"`, `"halfling"`,
`"gnome"`, `"goblin"`, `"kobold"`, `"ratfolk"`, `"drow"`, or a creature that is one: a goblin, a kobold, a
rat king, a drow who speaks). As of #1392 none speaks: the campaigns have no elf, halfling, gnome, goblin,
kobold, ratfolk, or drow NPC, and their creatures have no words, and Ælfrun of Holtwīc is a
human cottar; **half-elves and half-orcs speak as humans do** (Cockney or posh by their station, never
the Brummie of elves); the fey of Ælfhām are not elves by this rule. A new speaker
of one of these races is written in its accent from the start.

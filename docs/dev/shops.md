# Shops

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/07d-shops-and-rooms.md. Keep it current with the code, as CLAUDE.md says.

Shops (spec §7 "Shops", M27, #145) are data: `data/shops.toml` (`content/shop.rs`: `Shops` with
the currency, the outraged faction, `presence`, the generation `chance`, keeper `names`, the
`[keeper]` template, and the kinds, `ShopDef`: `always` (objects a generated shop lays out
first, a stack each, before its drawn goods: the general store's scrolls of identify, #393),
`never` (objects of a stocked class a generated shop never lays out, read in `Shops::stocks`:
the general store's and the provisioner's elixirs, #568; none may also be in `always`),
`stock`/`buys` by item class (`ItemClassId`, ids
in `data/item_classes.toml`, `docs/dev/objects.md`), `items`, `markup`, `pays`; `Shops::price`/`offer` are the one place
prices are made, from `Stack::value`, the value of what the stack is made of: its parts, a gem
by its size and cut, `docs/dev/objects.md`; `Shops::trades`/`buys` take the stack, so a loose gem,
whose object is worth 0, is traded by its gem; the general store buys gems and no shop stocks
them). Every object has a `value` (required; test fixtures with their own objects need
`value = 0`; fixtures that replace `factions.toml` but keep the built-in creatures need the
`merchants` and `outraged` factions, and fixtures that replace creatures or objects start from
`ContentFiles::default()` or drop `data/shops.toml`). A shop is a `ShopPlan` in the template
(level files: `[[shop]] kind keeper area post`; generated: `mapgen/shop.rs`, last after special
rooms, rooms levels only, from the map stream, by the branch's `ShopOdds`: its own `shops` table
or the shared odds, docs/dev/mapgen.md; each stack after `always` drawn from what the kind
stocks by its loot `frequency`, 100 for what is never loot, `magic::loot::pick_weighted`, one
draw, so spell scrolls come by spell level, #397) whose keeper is spawned at its post; at
instantiation it becomes a `world::Shop` in `LevelState::shops` (saved, hashed; format 30): the
keeper's actor id, its `exits` (the cells around the floor that could be walked onto or were
doors when the level was made: the door), `unpaid` (what the player owes, counted by goods: an
object and its own parts, `Stack::same_goods`/`goods`, so a gem is owed at its gem's price;
`unpaid_parts` assigns the counts to the last stacks of the same goods; only `pay`, `put_back`,
`outrage`, and `keeper_died` take anything off it, so whatever uses unpaid goods up (eating,
a quest hand-in, rotting away) leaves their price on the bill with no hook of its own; it may
hold more of an object than the pack does; its lines are counts by object, not stacks lying
anywhere, so `Shop::is_valid_for` checks them as such: any count, no magic, no food age), and
`ShopStatus` (Open, Outraged, Abandoned). Rotten food (spec §8 "Hunger", #195) is worth
nothing: `World::spoilt` (by `rot::stage`) makes `asking_price` and `picked_up_goods` free and
`deal` refuse it (`Deal::spoilt`: "Grimbold sniffs at ... and says: \"Not at any price.\"");
`ShopView::price`/`offer` in perception do the same for look mode, the object screens, and the
sale dialog, while the bill (`Shop::owed`, `PerceptionView::unpaid`) stays priced by object. The
rules are `world/shop.rs`: `turned_back` from `player_moves` (a step onto an exit owing
anything: the keeper steps into the way and the step lands one cell back into the shop, priced
as a step; walking out of the door never steals), `shop_step` (greeting, theft: leaving owing
any other way, such as a wall dug open) from `player_moves`, `picked_up_goods` from `pick_up`,
`ate_goods` from `eat.rs::finish` (food eaten off the shop floor is owed for; unpaid food from
the pack stays owed, and the keeper says so), `trade` (drop and `Action::Sell`) from
`drop_objects`, `pay` (`Action::Pay(None)`, no time; `Action::Pay(Some(dir))`, `pay:W`, pays and
steps out at once),
`provoke` from `fight.rs::resolve` (the player attacking a keeper; `provoke`, theft in
`shop_step`, and `pay` each make one alignment deed, `docs/dev/alignment.md`), `caught_stealing`
from `world/steal.rs` (a keeper that catches the player stealing, `docs/dev/stealing.md`; a theft
it does not notice takes its purse and leaves the shop open), `keeper_died` from
`creature_dies`, `may_loose` in `shoot.rs` (unpaid goods are not shot or thrown). An outraged
keeper gets the `outraged` faction as its `faction_override`, so the AI and stance need nothing
new. Keepers are NPCs: `budge = "off_post"` (no swapping at the post), one-liners in
`data/dialogue.toml`, and `World::called` names them by their shop ("Grimbold says: ...").
Prices are by the goods, never their magic (`Stack::magic` is kept on what is sold and put back, never
priced; keys and quest letters are worth 0). Generated shops avoid locked rooms and rooms with
something lying in them, and `hide_traps`/`place_loot` leave shop floors alone. The player's `favour` (Presence modifier × `presence` + the Negotiation `Effect::Bargain` (Appraise's until #666), through `Actor::skill_effect` so of Haggling and skill talents count, + what
a good keeper makes of their alignment band, `world/alignment.rs::keeper_favour`, at most
`MAX_FAVOUR`) is never shown: presentation reads only prices
(`PerceptionView::shop_at`/`shop_here`, `price_tag`, `unpaid`, `owed_on_leaving`, `sale`,
`own_gold`; look mode, the object screens, and the shop's dialogs in `input.rs`).

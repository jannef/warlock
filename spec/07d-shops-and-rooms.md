# Warlock specification — 7. Levels and data formats (continued): shops and special rooms

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Shops** (`data/shops.toml`, optional; without it there are none and level files may mark
  none; M27, #145), ADOM-like: a room with goods on its floor and a named shopkeeper by its
  door. A **general store** sells a bit of everything, **specialty shops** one kind of thing.
  - **Money and worth**: gold coins (`$`, stackable) are the currency (`currency`). Every object
    has a `value` in gold (§7 "Objects and materials"); what is worth 0 (junk: rocks, sling
    stones, copper coins, a burning arrow) and the currency are never traded.
  - **The file**: `currency` (a stackable object), `outraged` (the faction a robbed or attacked
    keeper joins; it must be hostile to the player), `presence` (percent better prices per point
    of the Presence modifier, 0 to 10; 3), `chance` (percent of generated rooms levels with a
    shop; 70), `names` (keepers' names for generated shops), `bundle` (`[least, most]` of a
    stackable object in one stack of generated stock; `[6, 15]`), `[keeper]` (every keeper:
    `creature`, `carry`, `wield`, `wear`, as in level legends; the `shopkeeper` creature, level
    18, quick (1200), tough, in the `merchants` faction, carrying a spear, a small shield, a chain
    mail shirt, an iron helm, and a purse of 300 gold coins), and `[[shop]]`s (at most 32; ids are
    indices, in the save fingerprint): `id`, `name` ("general store"), `title` (its sign, with
    `{keeper}` for the keeper's name: "{keeper}'s General Store"), `stock` and `buys` (item
    classes), `items` (`[least, most]` stacks of generated stock), `markup` (what it asks, percent
    of the value, 100 to 1000), and `pays` (what it pays for what it buys, 0 to 100). Objects
    are traded by **item class** (`data/item_classes.toml`, §7 "Objects and materials":
    `melee_weapons`, `missile_weapons`, `missiles`, `shields`, `body_armour`, `helmets`,
    `cloaks`, `gloves`, `boots`, `rings`, `amulets`, `light_sources`, `tools`, `food`,
    `potions`, `scrolls`, `gems`, `coins`, `miscellaneous`). The kinds: a general store (melee
    weapons, missile weapons, missiles, armour of every kind, light sources, tools, food, potions,
    scrolls; buys gems too, §7 "Gems", and sells none; asks 150 %, pays 30 %), a weaponsmith ("{keeper}'s Forge": melee weapons and
    missiles; 130 %, 40 %), an armourer ("{keeper}'s Armoury": shields, body armour, helmets,
    cloaks, gloves, and boots), a provisioner ("{keeper}'s Provisions": light sources, tools,
    and food; 120 %, 30 %), and a fletcher ("{keeper}'s Fletchery": missile weapons and missiles;
    125 %, 40 %). General stores stock and buy scrolls (§8 "Scrolls"); alchemists wait for potions that do
    something, scribes for more scrolls.
  - **Prices** (`Shops::price`, `Shops::offer`; integers): a stack's value (each one's times the
    count) times the markup, less the player's **favour**, rounded up and at least 1; an offer is
    the value times `pays`, plus the favour, rounded down, never more than the shop asks for the
    same. The favour (`world::favour`) is `presence` percent for each point of the Presence
    modifier plus the Appraise skill's `bargain` benefit (§8 "Skills": 20 % at 100), plus, at a
    good shopkeeper's (every keeper is good), what their alignment band gives (`[shops] favour`
    in `data/alignment.toml`, §8 "Alignment": Saintly 10, Evil −10, Wicked −20), together at
    most 50 % either way. Only the prices are ever shown, and they include all of it.
  - **Buying**: stepping onto the shop floor from outside the keeper greets the player
    ("Welcome to Grimbold's General Store!"). While the keeper trades (`ShopStatus::Open`),
    everything traded lying on the shop floor is for sale: stepping onto it says the price ("You
    see here a dagger (for sale, 45 gold)."), and so do look mode and the pick-up screen.
    Picking it up adds it to what the player owes (`Shop::unpaid`, counted by goods: an object
    and what it is made of, so a gem is priced by its gem, size, and cut) and says
    the price ("You pick up a dagger (unpaid, 45 gold).", with "You owe Grimbold 63 gold." when
    more is owed). **Paying** (`p`, `Action::Pay`, replay text `pay`) pays everything owed, from
    the gold the player carries into the keeper's purse, and takes no time; refused outside a
    shop, owing nothing, or without enough gold ("You owe Grimbold 75 gold, but you have only
    60."). Paying with a direction (`pay:W`) pays and then steps that way, out of the shop, in
    one action that takes the step's time; refused, with no step, if the player cannot pay. Without enough gold the player **puts it back**: dropping unpaid goods in the shop
    returns them to the floor (and off the debt), taking a drop's time. Which stacks are unpaid
    follows from the counts: of an object's stacks, the last ones in the pack.
  - **Using up goods** is buying them. Only paying, putting back, theft, and the keeper's death
    take anything off what the player owes, so unpaid goods eaten (or used up any other way)
    stay owed, and leaving still asks first, is turned back at the door, and is theft any other
    way ("Grimbold says: \"Eaten or not, the angelica root is still 47 gold.\"", said when what
    is owed for an object is more than the pack holds of it). Food eaten off the shop floor
    (`eat:floor:a`) is owed for as it is eaten ("Grimbold says: \"You eat it, you buy it: 47
    gold for the angelica root.\"", with "You owe Grimbold 63 gold." when more is owed); a meal
    stopped early eats nothing and owes nothing.
  - **Selling**: dropping the player's own things in the shop keeps them and says the offer
    ("Grimbold offers 15 gold for a scroll of light."), or that the shop has no use for them;
    **selling** (`Action::Sell`, replay text `sell:b`, `sell` for everything) takes the offer
    from the keeper's purse ("You sell a scroll of light to Grimbold for 15 gold."; "Grimbold
    cannot afford ..." when the purse is short) and leaves the thing on the floor, for sale.
    Presentation asks first, in the dialog (§6): dropping in a shop what it buys asks
    `Grimbold offers 15 gold for a scroll of light.` with `Sell for 15 gold` (accept) and `Keep
    it` (cancel: nothing is dropped).
  - **Rotten food** (§8 "Hunger": food past its keeping, by its age) is worth nothing: a shop
    neither sells it (lying on its floor it shows no price, and picking it up or eating it there
    owes nothing) nor buys it (no offer; dropping or selling it keeps it, and the keeper says
    "Grimbold sniffs at a loaf of bread and says: \"Not at any price.\"", the one in-world hint of
    rot besides eating it). What is already on the bill stays priced as it was (the bill counts
    by object, whatever the age). `World::spoilt` in `world/shop.rs` and `ShopView` in
    perception decide it with the same `rot::stage`.
  - **The way out**: a shop's `exits` are the cells around its floor that could be walked onto,
    or were doors, when the level was made: its door. A step onto one while owing anything is
    turned back (`World::turned_back`, in the rules, so no step out of the door ever steals):
    "Grimbold steps into your way: you haven't paid.", and the step lands one cell the other way,
    back into the shop, or, if that cell is not free shop floor, on the nearest free cell of the
    shop floor (the first in row order among the nearest), or nowhere if none is free; it takes
    a step's time (`step_price`) either way. Presentation asks first, in the dialog: with the
    gold, `You owe Grimbold 47 gold.` with `Pay 47 gold and leave` (accept: `pay:W`, paid and out
    in one action) and `Don't pay` (decline: the step, turned back; so is Esc); without it, `You
    owe Grimbold 125 gold and have only 60. Put something back, or bring more gold.` with only
    `Go back` (the step, turned back). A held movement key stops at the shop's edge instead of
    asking.
  - **Theft**: leaving the shop floor owing anything any other way (a step through a wall dug
    open, or anything else that puts the player outside but a way out) is theft ("You
    leave without paying. \"Stop, thief!\" shouts Grimbold."): the keeper joins the `outraged`
    faction (its `faction_override`, hostile to the player; ADOM-like, it comes after them), the
    shop trades no more (`ShopStatus::Outraged`), and what was owed is the player's, stolen. A
    keeper the player attacks (any attack, melee, shot, or throw, hit or miss) turns the same way
    ("\"You will pay for that!\" shouts Grimbold."). A step out that is not a way out asks first,
    in the dialog: `You owe Grimbold 47 gold. Leave this way without paying, and Grimbold will
    call it theft.` with `Pay 47 gold and leave` (accept, with the gold), `Leave without paying`
    (plain: the step, theft), and `Stay` (cancel), so theft is always the player's choice.
    Unpaid goods cannot be shot or thrown ("You have not paid
    for a dagger (c)."). A keeper's death abandons its shop (`ShopStatus::Abandoned`): what lies
    there is free to take, and its purse falls with it. Witnesses, pacifying a keeper, and the
    steal skill come later.
  - **The keeper** stays at its post (a merchant has no enemies, so it neither hunts nor looks
    into noises) until outraged; then it fights and chases like any hostile creature and goes back
    to its post when it has lost the player. It does not swap places with the player at its post
    (`budge = "off_post"`, §8 "Displacement"). Chatting with it (`C`) it answers in one-liners
    about trade (`data/dialogue.toml`), and messages call it by its name ("Grimbold says: ...";
    `World::called`); an outraged keeper is in no mood to talk.
  - **Magic items** (§8 "Magic items") are priced as their base item, identified or not, and
    sold to a shop they lie on its floor, magic and all, for sale at that price. Keys and quest
    letters are worth nothing and never traded.
  - **Generation** (`mapgen/shop.rs`): after special rooms, last of all, with `chance` a rooms
    level picks one of its rooms (not the arrival room, not a vault, not a locked room) with a
    single door, no stairs, nothing lying in it, no special room, and no patrol in it,
    uniformly, and makes it a shop of a kind drawn
    uniformly, kept by a name drawn from `names`: the keeper stands beside the cell inside the
    door (the way in stays free), a lamp hangs over the middle of the room, and the floor is laid
    with `items` stacks (as many as there are free cells, one a cell), each drawn uniformly from
    what the kind stocks (traded, not burning), a stackable one a `bundle`. All from the level's
    map stream, so a level without a shop is the map it would be without them. Caverns get none.
    Hidden traps and floor loot (§8 "Traps", "Magic items") never lie on a shop's floor.
    About one level of the Depths in six gets one (a test keeps it between 11 and 20 % over
    1200 levels; 16 % now: rooms levels lock some of their single-door rooms, which never become
    shops, and neither does a room where a key was left).
  - **Level files** mark shops with `[[shop]] kind = "general_store" keeper = "Grimbold" area =
    [x0, y0, x1, y1] post = [x, y]` (the shop floor, inclusive, inside the map, not overlapping
    another shop; the post walkable, on it, and where no creature or the player starts); the
    keeper is spawned there from `[keeper]`, and the level places the goods like any objects.
    Vaults have none.
  - **State**: each level state keeps its shops (`LevelState::shops`: the kind, the keeper's
    actor id and name, the floor, the post, the ways out, what the player owes, and the status; saved and
    hashed). Prices are not saved: they follow from the data and the player.
  - **What the player perceives** (`PerceptionView::shop_at`, `shop_here`, `price_tag`,
    `unpaid`, `owed_on_leaving`, `sale`, `own_gold`): an open shop's sign, its keeper's name, its
    prices for them, what they owe, and their own gold; never a value, a markup, or how their
    Presence and skills move the prices.
- **Special rooms** (`data/room_effects.toml`, optional; without it there are none; M25),
  ADOM-like: a room of a generated level, or an area a hand-made level marks, carries an
  **effect** that changes the rules while one is inside it. Top level: `chance` (percent, 0 to
  100: how likely each candidate room is to be special) and `per_level` (at most this many special
  rooms a generated level, default 1). Each `[[effect]]` (at most 32, in file order; ids are
  indices, in the save fingerprint): `id`, `name` (without an article: "echoing hall"), `mark` (one
  character, unique: the `rooms` dev layer), `enter` (the message the first time the player is
  inside one), and what it does, each optional: `light` (percent of all light kept inside, 0 to
  100, default 100; §5 "Special rooms"), `glow` (`[r, g, b, uv]` added over the room like ambient
  light), `noise` (percent of the loudness of every noise made inside, 1 to 1000, default 100; §5
  "Hearing"), `regen` (percent of the usual pace of regeneration inside, 1 to 1000, default 100;
  §8 "Combat"), `shunned` (creatures hostile to the player do not step in; §8 "Creature
  behavior"), and `floor` (terrain generators lay over the room's plain floor: walkable,
  transparent, not stairs, a door, or running water).
  - **The effects**: a *shadowed room* keeps 30 % of the light ("The light seems to drain away
    here."); an *echoing hall* doubles noise ("Your footsteps echo loudly."); a *hushed chamber*
    leaves two fifths of it ("A deep hush fills this room."); a *sanctuary* heals three times as
    fast, glows faintly warm (`[60, 44, 22]`, about enough for normal eyes to see by), and keeps
    hostile creatures out ("You feel at peace here."); a *flooded room* has a floor of shallow
    water, slower (150) and loud (200) to wade ("The floor here is flooded."). Blood washing away
    in a flooded room waits for blood (#127).
  - **Generation** (`mapgen/special.rs`): last of all, each room of a rooms level but the arrival
    room and vaults, and each of eight chambers a caves level tries, is special with `chance`
    (7 %) until the level has `per_level` (1); the effect is drawn uniformly, from the level's
    map stream. A rooms level's special room is the room's rectangle, walls and doors included;
    a cave's is a blob: the floor 4-connected to a random free floor cell more than 10 cells
    (Chebyshev) from the arrival, within radius 5 of it (at least 12 cells; vaults left out), plus
    the rock around it. The effect's `floor` replaces plain floor in it (doorways too). A shunned
    effect never goes where a creature patrols. About 40 % of generated levels get one (a test
    keeps it between a third and a half over 1200 levels).
  - **Level files** mark rooms with `[[room]] effect = "sanctuary" area = [x0, y0, x1, y1]`
    (inclusive, walls and doors included; inside the map; rooms never overlap; vaults mark none).
    A hand-made level draws its own floor (a flooded room's water). The hub's stairs nook is a
    sanctuary.
  - **State**: each level state keeps its special rooms (`LevelState::rooms`: the effect, its
    box, a blob's cells, and whether the player has been inside; saved and hashed, format
    version 15). The rules ask one question, `World::room_effect_at(level, pos)` (inside the core
    the level state's `room_effect_at`): light and creature sight, noise, regeneration, and
    creatures that shun a sanctuary. The first time the player's cell is in a special room
    (after any action, or arriving there), the room is marked entered and its `enter` message is
    said; never again for that room. Perception knows a room only once entered
    (`PerceptionView::room`, on cells seen or remembered): look mode names it (§6). The light it
    changes is seen like any light, before and after.

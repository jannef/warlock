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
    125 %, 40 %). General stores stock and buy scrolls (§8 "Scrolls"), and a generated one always lays out a
    stack of scrolls of identify first (the kind's `always`, at most its least `items`); alchemists wait for potions that do
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
    ("\"You will pay for that!\" shouts Grimbold."), and so does a keeper that catches the
    player stealing from it (§8 "Stealing": "Grimbold catches you stealing. \"Stop, thief!\"");
    a theft it does not notice leaves the shop open, the keeper's purse gone (a sale it then
    cannot pay for does not happen). A step out that is not a way out asks first,
    in the dialog: `You owe Grimbold 47 gold. Leave this way without paying, and Grimbold will
    call it theft.` with `Pay 47 gold and leave` (accept, with the gold), `Leave without paying`
    (plain: the step, theft), and `Stay` (cancel), so theft is always the player's choice.
    Unpaid goods cannot be shot or thrown ("You have not paid
    for a dagger (c)."). A keeper's death abandons its shop (`ShopStatus::Abandoned`): what lies
    there is free to take, and its purse falls with it. Witnesses and pacifying a keeper come
    later; the Steal skill takes from a keeper's pack, never the shop floor (§8 "Stealing").
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
    with `items` stacks (as many as there are free cells, one a cell), each drawn from what the
    kind stocks (traded, not burning) by its loot `frequency` (100 for an object that is never
    loot; #397: a spell scroll of a low tier lies there more often than one of a high tier), a
    stackable one a `bundle`. All from the level's
    map stream, so a level without a shop is the map it would be without them. Caverns get none.
    Hidden traps and floor loot (§8 "Traps", "Magic items") never lie on a shop's floor.
    About one level of the Depths in six gets one (a test keeps it between 11 and 20 % over
    1200 levels; 16 % now: rooms levels lock some of their single-door rooms, which never become
    shops, and neither does a room where a key was left).
  - **A branch's own shops** (`shops` on a branch in `data/dungeons.toml`, or on a range of its
    depths in `[[dungeon.levels]]`, which overrides the branch's table field by field): `chance`
    (0 to 100), `kinds` (ids of kinds, the draw uniform among them), `names`, and `keeper` (a
    creature type, the campaign's own allowed; it carries, wields, and wears the shared
    `[keeper]`'s kit), each defaulting to `shops.toml`'s. `kinds` and `names` may not be empty
    while `chance` is above 0. The draws are the same ones from the same stream, so a branch
    without the table generates what it would without the setting; `shops = { chance = 0 }`
    means none.
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
- **Special rooms** (`data/room_effects.toml`, optional; without it there are none; M25, #402;
  developer notes `docs/dev/special-rooms.md`, the second wave's design
  `docs/design/special-rooms.md`), ADOM-like: a room of a generated level, or an area a
  hand-made level marks, carries an **effect** (a *feature*) that changes the rules while one is
  inside it; rarely one covers a whole level. Top level: `chance` (percent, 0 to 100: how likely
  each candidate room is to be special; 12) and `per_level` (at most this many special rooms a
  generated level, default 1). Each `[[effect]]` (at most 32, in file order; ids are indices, in
  the save fingerprint): `id`, `name` (without an article: "echoing hall"), `mark` (one
  character, unique: the `rooms` dev layer), `enter` (the message the first time the player is
  inside one), `weight` (1 to 100, default 10: how often it is drawn), and, each optional: `near`
  (said the first time a cell of such a room comes into the player's view from outside it),
  `level_chance` (percent, default 0) with `pervade` (said on every arrival on a level it covers),
  the eligibility `needs_danger` (only on a level with a danger), `needs_below` (only on a branch
  level with another below it), `min_cells` and `max_cells` (only for a room of at least / at
  most that many floor cells), and what it does: `light` (percent of all light kept inside, 0 to
  100, default 100; §5 "Special rooms"), `glow` (`[r, g, b, uv]` added over the room like ambient
  light), `noise` (percent of the loudness of every noise made inside, 1 to 1000, default 100; §5
  "Hearing"), `regen` (percent of the usual pace of regeneration inside, 1 to 1000, default 100;
  §8 "Combat"), `mana_regen` (the same for mana, §8 "Mana"), `shunned` (creatures hostile to the
  player do not step in; §8 "Creature behavior"), `floor` (terrain generators lay over the
  room's plain floor: walkable, transparent, not stairs, a door, or running water), and `status`
  (a status everyone inside holds while they stay, below; it must ease, at least 10 seconds a
  degree, the longest single action, so it never lapses while one stays), `swarm` (a
  teeming den's numbers, below; needs `needs_danger`), `temperature` (degrees Celsius added
  to the level's inside, −40 to 40, default 0; below), `burden` (percent of everyone's
  carrying limit inside, 10 to 1000, default 100; below), `mirrors` (`{ images = N }`, 1
  to 8: a hall of mirrors' reflections, below), `warp` (percent chance a step inside moves
  the stepper elsewhere, 0 to 100, default 0; below), `collapse` (`{ chance, damage }`: a
  collapsed floor, below; needs `needs_below`), `rise` (`{ faction, after }`: a cursed
  room's risings, below; needs `needs_danger`), `unholy` (Consecrate lifts it, below), `feature` (a
  terrain, transparent and not walkable, generators place in the room; below), `sides` (the room
  has a hidden side, and its feature a light per side: `{ good = LIGHT, evil = LIGHT }`), and
  `offers` (what the player may do beside the feature: `{ pray = TABLE }`, an outcome table in
  `data/outcomes/`). An effect with a `level_chance` needs its `pervade` line and is neither
  `shunned`, nor lays a `floor`, nor is a den, nor holds `mirrors`, nor collapses, nor has a
  `feature`.
  - **The effects** (weights in brackets): a *shadowed room* (10) keeps 30 % of the light ("The
    light seems to drain away here."); an *echoing hall* (8) doubles noise ("Your footsteps echo
    loudly."); a *hushed chamber* (8) leaves two fifths of it ("A deep hush fills this room."); a
    *sanctuary* (6) heals three times as fast and brings mana back three times as fast, glows
    faintly warm (`[60, 44, 22]`, about enough for normal eyes to see by), and keeps hostile
    creatures out ("You feel at peace here."); a *flooded room* (8) has a floor of shallow water,
    slower (150) and loud (200) to wade ("The floor here is flooded."); a *black chamber* (5)
    keeps no light at all, carried light too, so only darkvision sees in it ("The dark here
    swallows your light."); a *draining room* (6, #403) gives everyone inside **Ebbing** (§8
    "Status effects": a point of mana drained every 20 seconds and none coming back), glows a
    faint cold violet (`[10, 6, 24, 8]`), and covers a whole level with `level_chance` 5 ("You
    feel your power seeping out of you."; over a level: "Something here drinks at your
    power."); a *teeming den* (8, #405; only on a level with a danger, in a room of at least 16
    floor cells) is full of sleeping creatures and their hoard (below; "A warm, rank smell
    drifts out of the dark ahead." when glimpsed, "You hear slow breathing all around you." on
    entering); a *sweltering chamber* (5, #404) is 25 °C warmer than the level and glows a faint
    red (`[30, 10, 0]`; "A wave of heat meets you."); a *frigid hall* (5) is 25 °C colder and
    glows a faint blue (`[0, 8, 22]`; "Your breath mists in the bitter cold."); a *silent
    vault* (5, #409) keeps a hundredth of every noise made inside (`noise = 1`) and gives
    everyone inside **Silenced** (§8 "Status effects": no casting, reading, or throwing one's
    voice; "Your footsteps make no sound at all."); a *heavy hall*
    (4, #412) halves everyone's carrying limit ("Your pack drags at you, as if the stones pulled
    it down."); a *hall of mirrors* (3, #419; only in a room of 80 floor cells or fewer) shows
    everyone inside with two reflections ("Your reflection stares back at you from every side.";
    below); an *unstable room* (4, #411, only in a room of at most 60 floor cells) warps
    one step in ten ("The floor shimmers under your feet."); a *beast den* (5, #410; as a
    teeming den) is a den of beasts with a hoard of bones and fresh corpses (below; "You smell
    musk and old blood." when glimpsed, "Heavy shapes breathe in the dark around you." on
    entering); a *collapsed floor* (3, #420; only
    on a level with another below it) has a floor of cracked stone that may give way under the
    player ("The floor ahead is cracked and sagging." when glimpsed, "The floor creaks under your
    weight." on entering; below); a *treasure room* (5, #415; only on a level with a danger) is a
    tempting hoard (below; "Something glitters ahead." when glimpsed, "Treasure lies here,
    unguarded." on entering); a *cursed room* (6, #408; only on a level with a danger; unholy, as is the
    draining room) raises the dead killed inside (below; "A cold draught smells of the grave."
    when glimpsed, "The air is cold and the dead are near." on entering) and covers a whole
    level with `level_chance` 5 ("The dead do not rest on this level."). Fennmynster favours it
    at 15; an *altar room* (8, #406; only in a room of at least 9 floor cells) holds a
    forgotten altar to pray at ("A faint glow rises from an altar ahead." when glimpsed, "An old
    altar stands here, its god long forgotten." on entering; below). Blood washing
    away in a flooded room waits for blood (#127).
  - **Hot and cold rooms** (`temperature`, #404): a room sets its cells' temperature and
    nothing else: no status, no damage, nothing done to items or food. The temperature at a cell
    is decided in one place, `WorldState::temperature_at` (the level's, §5 "Temperature", plus
    what the effects there add, the room's and the whole level's), and whatever the temperature
    does is the environment's, so a hot room is like any hot place. Today that is two things:
    the status bar shows the temperature of the player's cell, so stepping in shows the change,
    and the ice follows it (§5 "Ice": a cold room's water freezes on a 10 °C level, ice melts in
    a hot room in a frost). A level with a hot or a cold room has had the same 30 days of
    weather as a level under the open sky when it is first made, so a cold room's pool is
    frozen when one first comes upon it. Neither covers a whole level (a hot level is its
    `temperature`). Fennmynster bans the sweltering chamber.
  - **A heavy hall** (`burden`, #412): inside, every actor's carrying limit (§8 "Burden") is
    `burden` percent of what it is elsewhere, kept a multiple of 3 grams; the player, creatures
    alike. It is read in one place, the load (`WorldState::load`, from the effects at the actor's
    cell, the room's and the whole level's multiplied), so every penalty of a burden follows:
    slower actions, louder steps, Dexterity capped in Defence, the status bar. Stepping in, the
    player is told the change of their burden ("You are burdened.", right after the step) and,
    the first time, the room's line;
    stepping out, that it is gone. A step is refused when the player would be Overloaded on the
    cell they step onto ("You carry too much to go in there.", no time), so one cannot walk in
    and be stuck, and one overloaded inside (who picked something up there) can still step out
    to where they are not. It is saved as nothing new: the burden follows from where one stands.
    It never covers a whole level.
  - **Halls of mirrors** (`mirrors`, #419; `world/mirrors.rs`): every actor inside the room has
    `images` (2) **images**, reflections on free floor of the room (walkable, no door, no one on
    it, no other image). Where they stand is no draw: a hash (FNV-1a) of the level, the turn, the
    actor, and the image's index picks among the free cells, hall by hall and actor by actor in
    id order (`WorldState::images`), so the rules and perception compute the same images and no
    random stream moves; they move every turn, and vanish once the actor leaves. Perception
    shows the images of the actors the player sees, on cells the player sees, as those actors
    (kind, type, faction, how hurt, what they wear and wield, the statuses that show) under ids
    of their own, above every real actor's: nothing on the screen tells them from the real one,
    the player's own reflections too (look mode calls those "Your reflection"). Mirror-wise
    (§8 "Talents") sees those within 2 cells of the player for what they are: they are not
    shown. A step or a blow into an image the player sees meets only glass ("Your blow meets
    only glass.", a melee blow's time, no step); a missile the player aims at an image they see
    (a creature at any image) flies on through its cell instead of coming down there; bolts
    always fly to the first real creature. A creature outside the hall shooting or throwing at
    the player inside aims at the player or one of their images, each alike (a gameplay draw);
    creatures inside know the hall and aim true, and creatures never see images at all (their
    sight and AI read ground truth). Never over a whole level. The `actors` dev layer lists each
    actor's images (`; images (x, y) (x, y)`); `view` shows them as the player does.
  - **An unstable room** (`warp`, #411): every step onto a cell inside, the player's or a
    creature's (a move, a step made way for, a known trap sidestepped; never an incorporeal
    creature passing through), has `warp` percent (one draw of the gameplay stream, made only on
    such a cell, after any trap there has gone off and only if it left the stepper there) to
    move the stepper to a free cell of the level exactly as a teleport trap does (§8 "Traps":
    walkable, no one there, no trap, no ice, picked by a hash). The trap's lines say it: "You
    are wrenched elsewhere." for the player, "The giant rat vanishes!" for a creature they saw
    go. Where it lands is never a step, so it is never warped again on landing. Nothing is
    saved. Its `max_cells` keeps it to small rooms; it never covers a whole level.
  - **A collapsed floor** (`collapse`, #420; `world/collapse.rs`): the room's floor is
    `cracked_floor` (a walkable terrain, "cracked floor", `:`), laid by the generator over the
    room's middle only: a cell cracks only when its eight neighbours are all walkable cells of
    the room and none is a door, so no cell next to a wall or a door cracks, nor a door, and a
    ring of sound floor runs from every door round the room (#523; a room too narrow to have a
    middle gets no cracks at all). A hand-made level draws its own. Every step of the **player** onto a cracked cell of it (a
    move; the room's cells without cracks never give way) has `collapse.chance` percent (15,
    doubled while Burdened or worse; one draw of the gameplay stream, made only on such a cell,
    after any trap and warp there, and only while the level has a way down that leads
    somewhere) to give way: "The floor gives way beneath you!" The player falls to the level
    that way leads to (generated as usual on a first visit, as by the stairs; the step's time
    is all it costs), "You fall to <level>.", landing on a random cell of it (a second gameplay
    draw) that is walkable, no door, stairs, trap, or ice, where no one stands, outside every
    shop, and reachable from where the level is entered without passing a locked door (so never
    inside a vault behind one, nor behind a secret door, which is wall until found). They take
    `collapse.damage` (2d6), which can kill them (#523: there is always a sound way round, so
    the last hit point is not spared): the death is put down to the fall ("Fell through a
    collapsed floor" on the death screen, the dumps' `cause`). A survivor is Staggered. The cracked floor
    stays as it was. Creatures never fall (no draw for them). Nothing new is saved: the fall
    is a level change like the stairs'. Light tread (§8 "Talents") never falls. It never
    covers a whole level.
  - **A tempting hoard** (`hoard = { items = 3, loot_level = 3, guardians = [2, 4], danger = 1
    }`, #415; `mapgen/hoard.rs`, `world/hoard.rs`). A generated floor's treasure room gets
    `items` items (1 to 16), each rolled at the floor's item level + `loot_level` with the
    finder's luck and laid on a plain floor cell of the room drawn anew (no stairs, light, or
    trap), from the level's stream `swarm/<level id>` after its population; the floor's
    population and the generator's patrols keep out of it, so it lies unguarded. The first
    stack the player lifts inside it (`pick_up`; every stack of a pick-up counts) **seals** it:
    every door of it shuts (an open one unless someone or something is in the doorway; a broken
    one stays broken) and locks with a lock of the floor's level (no id: only a skeleton key
    fits; picked against the usual difficulty plus the branch's per depth, or kicked, §8
    "Doors"), each slamming as loud as a kick (banging, 100), and "The doors slam shut!" is
    said when any did; then `guardians` creatures (least to most, one gameplay draw) come, each
    drawn by the level's danger + `danger` from the branch's population and strays by the draw
    of §7 "Danger levels" (hostile to the player, never unique), awake, with gear rolled as a
    creature that comes, on a walkable free cell of the room on its **edge** (next to a cell
    outside it or not walkable): first those the player does not see, then those they see,
    then any free cell of the room. A level without a danger rule gets no guardians. A cave
    chamber has no doors: only the guardians come. A room seals once; the stacks lifted and
    whether it sealed are saved with the room (`SpecialRoom::lifted`, `sealed`; left out while
    nothing was lifted, so other rooms save and hash as before), and the `rooms` dev layer
    lists them (`, lifted 1, sealed`). Hoard-wise (§8 "Talents") lets one stack go first.
    Never over a whole level.
  - **Cursed rooms** (`rise = { faction = "undead", after = "2m" }`, #408; `world/curse.rs`):
    a creature that dies on a cell whose effects raise the dead (the room's, else the whole
    level's), is not of the rise's `faction`, and leaves a corpse is marked to rise, whoever
    killed it (a creature's kill too; never the player, who leaves no corpse to rise): at the
    death one gameplay draw picks what it rises as, alike among the types of the faction that
    are not unique and are shared or the run's campaign's own, of the dead's level, else of the
    nearest level that has one (the lower on a tie; with a single candidate nothing is drawn).
    The rising (`LevelState::risings`: when, where, what died, what it rises as; saved, hashed,
    left out while empty) is due `after` the death. At the end of every turn, after the
    spawns, the risings due on the **current** level rise in the order they died
    (`World::rise_tick`): if a corpse of the dead's type still lies where it died, one is taken
    and the risen creature stands there, or on the nearest free cell, carrying nothing (what
    the dead carried lies there already) and giving its own kill experience; else nothing
    happens. So a corpse picked up, eaten, or destroyed meanwhile never rises, and one carried
    out and dropped is not cursed again. A level the player is not on stays frozen: what fell
    due while they were away rises on their first turn back. The player is told "The goblin's
    corpse twitches." at a death they see and "The goblin's corpse rises as a zombie!" at a
    rising they see (it stops a rest, as any newcomer does); unseen, the risen groans (a voice
    at its cell). Grave warden (§8 "Talents"): what the player kills never rises. The `actors`
    dev layer lists the pending risings (`rising rat at (11, 2) as skeleton at 08:15:00`).
  - **Consecrate** (`unholy`, #408; `world/consecrate.rs`; the priest's at 9 and the paladin's
    at 12, §8 "Class abilities"): standing inside a special room whose effect is unholy (cursed,
    draining), or anywhere on a level an unholy effect covers whole, the player spends 15 mana
    and prays for a minute, turn after turn as a rest goes (anything that stops a rest stops it;
    an interrupted prayer lifts nothing, its mana and time spent). Done, the room leaves the
    level's special rooms for good, a plain room (look mode no longer names it), or the level's
    effect is gone ("A stillness settles over the room: it is consecrated.", "... over the whole
    level ..."); the room's first, when the player stands in one and the level is covered too.
    Risings no longer under a curse are dropped, and it is a good deed (`consecrate` in
    `data/alignment.toml`, 20). Elsewhere it is refused with no time ("There is nothing here
    to consecrate.").
  - **Forgotten altars and prayer** (`feature`, `sides`, `offers`, #406; `world/pray.rs`,
    `world/outcome.rs`; the design's §7): ADOM's pool with another flavour, not its gods: no god
    to name or serve, no sacrifice, no conversion. A room whose effect has a `feature` gets that
    terrain at the free plain floor cell nearest the middle of its box (ties by cell order) that
    holds nothing else (no stamp, light, creature or patrol, trap, or object), is no door's
    neighbour, and without which the room's walkable cells stay joined (4-connected); a room
    with no such cell is not made special. A room whose effect has `sides` draws its side from
    the map stream as it is made (one more draw, only for such an effect): good one time in
    four, neutral two, evil one; a hand-made level gives it (`[[room]] side = "evil"`, default
    neutral). The side is kept in the room (`SpecialRoom::side`, saved and hashed only when set)
    and never said in words; the feature shows it by its light, a placed light on its cell (the
    `sides` light; a side without one keeps the terrain's own): gold good, grey neutral, red
    evil. The altar is its own terrain, `forgotten_altar` ("forgotten altar"; look mode adds its
    description), never the showcase's or a temple's altar.
    - **Pray** (`Action::Pray`, `pray` in action text) is offered in the abilities menu's *Here*
      group (`Source::Place`, §6) while the player stands next to a cell whose terrain an
      effect's `feature` names and that `offers` pray (`Content::offer_at`; the first such cell
      in direction order), and nowhere else; refused there, taking no time ("There is no altar
      here to pray at."). A prayer is the outcome table's `seconds` (60) of turns of the `pray`
      action (the last cut short to end on time), through the rest's machinery: whatever stops a
      rest breaks it, and a broken prayer ("Your prayer is broken.") draws nothing, moves no
      alignment, and never spends the altar; the time is spent.
    - **The answer**: the deed first (`Deed::Prayed(side)`: `[shift] prayed_good` 5 at a good
      altar, `prayed_evil` −10 at an evil one, nothing at a neutral one; §8 "Alignment"), then one
      gameplay draw of the table's `kinds` (boon 35, nothing 30, bane 35), leaned by the side
      against the player's band's side: agreement (good and good, evil and evil) moves `lean`
      (15) from bane to boon, opposition from boon to bane, a neutral altar or a neutral player
      nothing (Devout, §8 "Talents": an agreeing lean doubled, an opposed one none); then one
      draw by weight among that kind's outcomes, its line said (one of its `messages`, drawn),
      and its hooks (`OutcomeEffect`, one existing system each: healing, cures, mending, mana,
      statuses (told by the outcome's line, not the status's own), hunger by states, objects
      and loot at the altar's foot (the walkable cell beside it nearest the player, not their
      own), known traps, identifying and forgetting (`Stack::forget`), harm that spares the
      last hit point and harm that can kill (`Harm::Place`: a place's kill earns no one
      anything; a death by it is "Struck down at a forgotten altar"), coins lost, the lantern
      snuffed, hostile creatures drawn as the spawner draws them (by the floor's danger, plus
      `danger`; none on a level without a danger), a random disease, stat damage, experience,
      the most valuable stack gone, the teleport trap's move, the burst spell code, terrain).
      A hook that finds nothing to do says the outcome's `otherwise` line instead (Glimpse with
      nothing unidentified: "You feel you understand a little."). The table
      (`data/outcomes/forgotten_altar.toml`, the wiki's "Forgotten altars") has six "nothing
      happens" lines and the minor and moderate boons and banes; the great ones come with #407.
    - **The altar explodes** (the bane `burst`, 5 of the banes): "The altar bursts apart in
      fire!", then a fireball (3d6 fire, radius 2, `Caster::Place`) centred on the altar through
      the burst spell code: everyone in it is hurt, the player too, and it can kill ("The altar's
      fire burns you."); the explosion noise (150); ice melts. Then the altar's cell becomes
      `rubble` and its light is gone.
    - **Exhaustion**: after a completed prayer, unless the altar is gone, one gameplay draw of
      the table's `exhaust` (50 %, the same whatever the prayer gave): the altar is **spent**, its
      cell `spent_altar` (dark, no light, no glow) and its placed light gone ("The altar's glow
      gutters and dies."); Pray is offered there no more. Nothing new is saved: the terrain
      grid is the record, and the `rooms` dev layer marks such a room `spent` (or `destroyed`).
      No other place offers prayer (the design's decision 14).
  - **A status while inside** (`status`, #403): as time passes, at the start of each stretch of
    it (`World::pass_time`: so after every action, the player's and each creature's, at every
    turn of a rest or another long action, and through a creature's coarse turns while the
    player rests), every actor on the current level whose cell has an effect with a `status`
    (the room's, the whole level's, or both) holds it at one degree, its easing begun anew
    (`World::dwell`; `stacking = "refresh"`), so it lasts while they stay and eases by its own
    `eases` once they leave. Everyone inside, creatures too: a kobold shaman's mana ebbs as the
    wizard's does. Not one immune to the status's kind of damage (the usual rule of
    `World::afflict`), not one with no mana when the status only touches mana (drains it or
    slows its return, `StatusDef::touches_only_mana`), and not one whose talents spare them
    (Anchored mind). The player is told as it begins and ends (the status's `start` and
    `end`), and it shows on the status bar; look mode shows it on creatures by its `seen`
    word.
  - **Teeming dens** (`swarm = { per_cells = 4, min = 4, max = 16, wake = 200, cry = 60,
    loot_level = 1, themes = { kind = 10, faction = 20 } }`, #405; `mapgen/den.rs`,
    `world/den.rs`). Nothing about a den is hand-made: a generated floor's den is filled after
    its population, from the level's own stream `swarm/<level id>` (so the rest of the level is
    the same with or without one). **Sleepers**: one per `per_cells` free plain floor cells of
    the room (no creature, stairs, light, or trap), `min` to `max` (1 to 64; never more than the
    cells), each drawn by the floor's danger from the branch's population by the draw that lays
    down its creatures (§7 "Danger levels": level weights, rarity, strays, out of depth), of the
    types hostile to the player and never unique, on a free cell drawn among the rest, with no
    post or route, its gear rolled with the population's (from the gear stream, after theirs),
    placed **asleep**. **Themes**: one draw in 0..100: under `kind` the den is a *warren*, every
    sleeper of one type (drawn first by the same draw); under `kind` + `faction` a *tribe*, every
    sleeper drawn from the population and strays of one faction (that of a type drawn first; it
    again should none weigh anything at the danger); else mixed. **The hoard**: one item per four
    sleepers (at least one), each rolled at the floor's item level + `loot_level` with the
    finder's luck and laid on a plain floor cell of the room drawn anew. The floor's population
    and the generator's patrols keep out of a den (a den is never made where a creature
    patrols); the sleepers come on top of the population and **count** toward the spawning cap
    (§7 "Danger levels"), so a floor with a den brings nothing new until enough of it is dead.
    A hand-made level may mark a den (`[[room]] effect = "teeming"`) but fills it itself.
    **Factions** (`swarm.factions`, faction ids): the sleepers are drawn the same way but only
    among the types of those factions, hostile to the player or not, and the den kind is
    eligible only on a branch whose own population holds a type of one of them (not unique);
    elsewhere its favoured share passes to the rest. **A corpse hoard** (`swarm.hoard =
    "corpses"`; default `items`, above): per four sleepers (at least one) a creature is drawn
    from the branch's population and strays (any faction, no uniques) by the same draw, and its
    corpse (if its type leaves one) and a pile of bones (junk) each lie on a plain floor cell of
    the room drawn anew; both are made when the level is first entered, so the corpses are
    fresh then. The **beast den** (`beast_den`, #410) is the one such kind: `factions =
    ["wildlife", "vermin", "marsh"]`, `hoard = "corpses"`, otherwise a teeming den's numbers.
  - **Sleep** (the status *Asleep*, §8 "Status effects": `sleeps = true`, no easing): a sleeper
    takes no turn (its turn is a wait, a stride's waits at once while the player rests; what it
    heard is forgotten) and is unaware of everyone (§8 "Class abilities": a sneak attack lands
    on it). Nothing wakes it with time. It **wakes** (the status ends) when (1) a noise reaches
    it at least at its hearing threshold × its den's `wake` percent (100 to 1000; 100 outside a
    den) where it lies (`World::spread_noise`, where every noise is heard; a noise the player
    makes needs Hush-foot's percent more); (2) a blow or any harm leaves it alive
    (`World::hurt`); or (3) a neighbour **cries**: a sleeper that wakes cries out at once, a
    voice of its den's `cry` loudness where it lies (0 to 300; none outside a den), which may
    wake the next, and so on: a cascade, deterministic, each sleeper waking once. Light and
    sight never wake one. A plain step (60) beside a kobold (hearing 15) wakes it; one three
    cells off, 30 there, too; the stealthy cross. The player is told "The kobold wakes up." when
    they see it; look mode shows *asleep* on a sleeper; Sixth sense adds "Something sleeps on
    this level." to its arrival note. Killing a sleeper is a hostile kill like any other (no
    alignment of its own).
  - **Generation** (`mapgen/special.rs`): last of all, from the level's map stream, each room of
    a rooms level but the arrival room and vaults, and each of eight chambers a caves level
    tries, is special with `chance` until the level has `per_level`. A rooms level's special room
    is the room's rectangle, walls and doors included; a cave's is a blob: the floor 4-connected
    to a random free floor cell more than 10 cells (Chebyshev) from the arrival, within radius 5
    of it (at least 12 cells; vaults left out), plus the rock around it. Its effect is then
    drawn among the effects the room is **eligible** for (its floor cells, the level's danger
    and whether a level lies below) and the dungeon does not ban: if any is favoured, one draw in
    0..100 walks the favoured ones in effect order by their shares; past them, or with none
    favoured, a weighted draw by `weight` over the others (or, when every eligible effect is
    favoured, over the favoured ones by their shares). For an effect with a `level_chance`, one
    more draw: with that chance it covers the **whole level** and the room stays plain (it
    counts toward `per_level`; at most one a level). The effect's `floor` replaces plain floor in
    the room (doorways too). A shunned effect, a den, or a hoard never goes where a creature patrols. About 60 % of
    generated levels get a special room (a test keeps it between half and seven tenths over 1200
    levels).
  - **A dungeon's own special rooms** (`room_effects` on a branch in a campaign's
    `data/dungeons/`, or on a range of its depths in `[[dungeon.levels]]`, overriding the
    branch's table field by field): `chance` (0 to 100), `per_level` (1 to 8), `favoured = { id
    = share }` (fixed shares of its draws in percent, 1 to 100 each, at most 100 together: a
    favoured effect keeps its share however many effects are added) and `banned = [ids]` (never
    drawn), each defaulting to `room_effects.toml`'s (no favoured, none banned). Unknown ids, an
    effect both favoured and banned, shares over 100, and every effect banned while `chance` is
    above 0 are content errors; the old `effects` list is refused, pointing to `favoured` and
    `banned`. A new effect therefore turns up in every dungeon that does not ban it.
    Fennmynster: `chance` 25, `per_level` 2, favoured flooded 35, hushed 15, shadowed 15,
    banned hot.
  - **Level files** mark rooms with `[[room]] effect = "sanctuary" area = [x0, y0, x1, y1]`
    (inclusive, walls and doors included; inside the map; rooms never overlap; vaults mark none),
    and may name an effect covering the whole level with a top-level `effect = "..."` (one that
    can cover a level; never on the overworld). A hand-made level draws its own floor (a flooded
    room's water). The hub's stairs nook is a sanctuary.
  - **State**: each level state keeps its special rooms (`LevelState::rooms`: the effect, its
    box, a blob's cells, whether the player has seen a cell of it (`glimpsed`) and been inside
    (`entered`)) and the effect covering it (`LevelState::pervading`); saved and hashed, each
    left out while empty. The rules ask one question, `World::effects_at(level, pos)` (inside the
    core the level state's `effects_at`), which gives the room's effect and the level's, folded
    the same way by every reader (percents multiply, glows add, flags hold if either does): light
    and creature sight, noise, regeneration and mana, and creatures that shun a sanctuary (by the
    room alone). The first time a cell of a room comes into the player's view it is glimpsed and
    its `near` line said (not when they first see it from inside); the first time the player's
    cell is in a special room (after any action, or arriving there), the room is marked entered
    and its `enter` message is said; never again for that room. On every arrival on a level an
    effect covers, its `pervade` line is said (with Sense of place, followed by "The whole level
    is like a gloomy room."). Perception knows a room only once entered (with Sense of place, once
    glimpsed; `PerceptionView::room`, on cells seen or remembered): look mode names it (§6); it
    never names the whole level's effect (`PerceptionView::pervading`, JSON dumps' `pervading`).
    The light it changes is seen like any light, before and after.
  - **Talents** (system `special_rooms`, §8 "Talents"): Sense of place (a room known from a
    glimpse; a whole level's effect named on arrival), Hallowed rest (a room's quickening of
    healing and mana half again more), Anchored mind (a room's or a level's Ebbing
    never takes them; #403), Hush-foot (shadow; sleepers need twice the noise to wake from
    anything the player does; #405), Mirror-wise (a hall of mirrors' images within 2 cells
    seen for what they are; #419), Light tread (a collapsed floor never gives way under
    them; #420), and Grave warden (guard; nothing the player kills rises; #408). The class ability Consecrate (above) lifts an unholy room or level. System `prayer`: Devout (wayfarer; an altar of the player's
    side leans twice as far, an opposed one not at all; #406).

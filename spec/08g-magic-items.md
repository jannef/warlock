# Warlock specification — 8. Architecture (continued): magic items

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Magic items** (`warlock-core/src/magic/`, `content/affix.rs`; M29, #149), after Path of
  Exile. A first pass: the lists and numbers are to be tuned after playtesting.
  - **Base items** are objects with a `kind` (§7): weapon, missile weapon (`launcher`),
    ammunition, body armour, helmet, cloak, gloves, boots, shield, ring, amulet. Only they are
    magic; they and scrolls (§8 "Scrolls", never magic) are loot.
  - **Rarity**: *normal* (no affixes), *magic* (a prefix, a suffix, or one of each; most have
    one), *rare* (up to three prefixes and three suffixes, usually three or four affixes in all,
    and a two-word name: a first word and a second for its kind, "Grim Bite"). Unique items
    later. An item never has two affixes of the same effect.
  - **Instances**: a magic or rare stack records what it rolled (`magic::Magic` on
    `object::Stack`): its rarity, its **item level**, each affix by id with its tier (1 first)
    and rolled value, a rare item's name, and whether the player has identified it, for an
    enchanting system later. It goes with the stack everywhere: piles, inventories, missiles in
    flight (an arrow shot from a magic stack lands as one of it, and joins it again when picked
    up: stacks join only with the very same magic; a flaming arrow is never magic), saves.
  - **Names** (`Stack::describe`, the one name presentation uses): normal "a long sword"; magic
    its prefix and suffix around the base, "a Sharp long sword of the Bear" ("12 Sharp arrows of
    Durability"); rare its name then the base, "Grim Bite, a long sword"; unidentified the base
    and its rarity, "a long sword (magic)", "12 arrows (rare)". The base is the object's name
    with its gems and precious metals (§7 "Objects and materials": "a gold and ruby amulet
    (magic)"), which show identified or not. Rarity colors the name (§6).
    Combat messages name the base ("The kobold's spear glances off your armour.").
  - **Unidentified**: magic and rare items are found unidentified. The rarity shows, the affixes
    and a rare name do not, and their effects apply all the same while worn or wielded (as in
    ADOM and Path of Exile; so an attentive player may notice them: a stat on the character
    screen, a glow). Wearing or wielding an item for a while does not identify it. The core API
    `World::identify(item)` (`world/loot.rs`; crate-internal, for actions: the scroll of identify,
    #150, calls it from its read action) identifies the player's inventory stack `item` and says
    "This is a Sharp long sword of the Bear!", returning `Identified::Now(name)`, `Known`
    (already identified, or normal), or `NoSuchItem`. Presentation reaches affixes only through
    `Stack::known_magic` (nothing until identified), so neither perception, the UI, nor
    player-facing dumps can leak them; dev dumps show the truth.
  - **Effects** (`magic::mods`) apply while the item is worn or wielded (equipped), never while
    merely carried. Attack effects (to-hit, damage, extra damage die, critical threat and
    multiplier, attack speed, range, a bow's draw, durability) on a weapon, missile weapon, or
    ammunition are **local**: they change attacks made with that item (a shot: the missile
    weapon's and the ammunition's together; a throw: the thrown weapon's); on anything else
    (gloves, rings, amulets, helmets) they are **global**: every attack. Effects on the item itself
    (Armour, Protection, Shield, Parry, `max_dex`, weight) are local (the Hardened prefix adds
    to an armour's Protection, which adds up with every other piece's). The rest are global: the six stats,
    maximum hit points, regeneration, Dodge (stacks), Deflection and natural armour (the best
    counts, with the creature's own), moving speed, carrying limit, stealth (quieter footsteps,
    §5 "Hearing"), **light** (the wearer carries the brightest light among its glowing items,
    a steady preset; `LightCache` adds it as a carried source, so creatures see by it too), and
    **skills** (§8 "Skills": a skill affix adds to that skill's value for what it gives,
    `Actor::skill_effect`, trained or not, an untrained one from 0; the Skills screen still lists
    trained values). Each rule reads them in one place: `Actor::effective_stats`, `max_hp`,
    `defence`, `melee`, `ranged`, `thrown`, `launcher_range`, `throw_range`, `skill_effect`;
    `World::cost_of` (speed); `burden::load`; `noise::noise`; `World::healing_rate`;
    `Stack::weight`. Speeds are at most 50 % off in all; an attack's weapon skill takes its
    `speed` off after them (`World::attack_price`, §8 "Weapon skills"). Pending hooks, added
    with the systems they need: affixes that raise a weapon skill's rank (#134's ranks exist;
    none roll yet), hunger (#137), and value (#145, a base `value` times rarity and affixes).
  - **Affixes** (`data/affixes.toml`, §7): 21 prefixes (mostly offensive, flat Defence, and
    life: Sharp, Heavy, Glinting, Keen, Brutal, Guarding, Reinforced, Sturdy, Supple,
    Featherweight, Nimble, Shimmering, Oaken, Hale, Vicious, Accurate, Fleet, Hunter's, Steady,
    Radiant, Burly) and 26 suffixes (stats, speed, criticals, utility, skills: of the Bear, Lynx,
    Ox, Owl, Sage, Lion; of Celerity, Precision, Slaying, Ruin, Mending, Stealth, Light, the
    Mule, Evasion, Warding, the Wind, Reach, Durability, the Tortoise, Vitality; of Shadows
    (stealth), the Hare (tumble), the Fox (listen), the Wanderer (survival), Balance (balance)),
    each with the kinds of item it rolls on and up to four tiers by item level; an item has at
    most one affix of each effect (of each skill, for skill affixes).
  - **Loot** (`magic::loot::roll`): the item level is the depth (a branch level's depth, a
    hand-made level's `item_level`, 1 elsewhere). One roll draws, in order: the base (among
    the loot objects with `level` at most the item level, by `frequency`, sorted by id; scrolls
    left out of loot that must be magic or better, and drawing no rarity: they are normal); a stack's count (ammunition 8 to
    20); the rarity (by weight, those below the grade asked for left out); how many affixes
    (magic: both by `magic.both`, else prefix or suffix evenly; rare: the count by weight, then
    how many are prefixes, evenly within at most three of each); each affix, prefixes first (by
    weight among those that may roll: the item's kind in its bases, something for its effect to
    change, a tier at the item level, an effect not yet on the item), its tier (evenly among
    those at the item level), and its value (evenly in the tier's range); a rare item's name (two
    draws). With nothing left to roll an item has fewer affixes, with none it is normal. Loot is
    found unidentified.
  - **Where**: generated branch levels' floors hold `loot` items (the Depths 2 to 4) on plain
    floor where nothing lies, stands, shines, or leads away, and vault `loot` cells an item at
    least of their grade; outdoor ruins' loot cells too (local areas have no floor loot); all
    from the level's `loot/<level id>` stream once its map is valid (the map is the same as
    without loot). Creatures with `loot` drop a magic or rare item (grade magic) where they die,
    besides what they carried, by their chance, from the stream `drop/<actor id>`, at their
    level's item level.
  - **Dev**: replay text `identify` / `identify:LETTERS` (`Action::Identify`, `F8` in dev
    builds: everything carried) identifies without time passing, recorded in replays.

- **Scrolls** (`warlock-core/src/world/read.rs`, data `read` in `data/objects/` §7; M29, #150):
  objects (`?`, the scrolls class, stackable, with a value) whose `read` says what reading one
  does. They are **read** (`Action::Read { item, target }`, replay text `read:LETTER:X,Y` at a
  cell or `read:LETTER:LETTER` on an inventory stack; `r`, §6 "Read"), which takes the `read`
  action's time and uses one up; a reading that cannot be done is refused, takes no time, and
  keeps the scroll: nothing to read on it ("You cannot read a dagger."), no target or the wrong
  kind ("Read it at what?", "Read it on what?"), the player's own cell ("You cannot aim at
  yourself."), out of reach ("That is out of range."), a spell on the overworld, or a stack
  there is nothing to learn about ("You already know what a dagger is."). Scroll types are
  known on sight; ADOM-style unknown labels ("a scroll labelled XYZZY") may come later.
  - **Spell scrolls** cast a trap's spell (§8 "Traps") with the same rules, data, and code
    (`World::cast_burst`, `World::cast_bolt`), aimed by the reader (§6 "Targeting"): the scroll
    of **fireball** and of **frost** a burst (the fireball and frost traps': 3d6 and 2d6 in
    radius 2, the fireball breaking earth and stone and melting ice, the frost freezing water to
    10 cm), which flies from the reader along the line through the cell aimed at (the line a
    missile flies, `Pos::ray`) as far as its `range` (10) and goes off on the first cell an actor
    stands on, before the first opaque cell (the reader's own cell when that is the next), or
    on the cell aimed at, so it may catch the reader; the scroll of **lightning** a bolt (4d6),
    which runs from the reader along that line and on, `length` (12) cells or to the first
    opaque cell, striking everyone on it but the reader. Its effect is `Event::Spell` from the
    reader (a burst flies to its center first, §4.8), its noise (`noise` in data: an explosion,
    thunder) is made where it goes off (a burst's center, the reader for a bolt) and heard by
    everyone in earshot, like a trap's. Unlike a trap's, its blows are the reader's: an
    `Event::Attacked` (a spell, a hit) for each actor reached, blood, damage (no roll to hit),
    and the wound's statuses, caused by the reader; so its kills earn the reader their
    experience (§8 "Experience") and count for their quests, and a shopkeeper it reaches turns on
    them (§7 "Shops"). A scroll that kills its reader is what killed them (`Death::cause`,
    "Killed by their own scroll of fireball"). Messages, said at once by what the player sees:
    "You read the scroll of fireball.", its `message` ("A fireball streaks from the scroll and
    explodes!"), then its blows in its `noun` and `verb` and combat's words ("The fireball burns
    the kobold.") and deaths ("The kobold dies.").
  - **The scroll of identify** identifies one unidentified stack the player carries
    (`World::identify`: "This is a Sharp dagger of the Lynx!"); a stronger "identify all" can
    come later.
  - **Where**: loot (by `level` and `frequency`, never magic), general stores' stock (they sell
    and buy the scrolls class, §7 "Shops"), and the trap hall beside its arrival (two of each
    spell, a scroll of identify, and an unidentified dagger). The showcase has none. Later:
    Spellcraft, Decipher Script, and Use Magic Device (§8 "Skills") may change reading.

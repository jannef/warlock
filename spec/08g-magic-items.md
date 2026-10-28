# Warlock specification — 8. Architecture (continued): magic items

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Magic items** (`warlock-core/src/magic/`, `content/affix.rs`; M29, #149), after Path of
  Exile. A first pass: the lists and numbers are to be tuned after playtesting.
  - **Base items** are objects with a `kind` (§7): weapon, missile weapon (`launcher`),
    ammunition, body armour, helmet, cloak, gloves, boots, shield, ring, amulet, tool (a digging
    tool in the Tool slot: the pick-axe, loot from item level 3 and rare among it (`frequency`
    10; Delving, Featherweight, and of the Bear roll on it); the pick-axes placed in levels, worn
    by creatures, and in starting kits are plain; a tool shows no numbers in the lists, its description its digging power, "Digs at 10"). Only they are
    magic; they and scrolls (§8 "Scrolls", never magic) are loot.
  - **Rarity**: *normal* (no affixes; 85 % of loot), *magic* (13 %: a prefix only 40 %, a
    suffix only 40 %, one of each 20 %), *rare* (2 %: three affixes 50 %, four 30 %, five 15 %,
    six 5 %, at most three prefixes and three suffixes, and a two-word name: a first word and a second for its kind, "Grim Bite"); a *unique*
    item is a rare one with a set name and a power (below). An item never has two affixes of
    one category (below).
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
    (magic)"), which show identified or not, its own gems included (§7 "Gems"), and its material
    when it is made of another than its base item's (§7 "Materials"), so the order is [prefix]
    [ornament] [material] base [suffix]: "a Sharp emerald-pommelled bronze long sword of the
    Bear"; a rare item keeps its two-word name and its gem and material follow in the base, "Grim
    Bite, an emerald-pommelled long sword", and in its description. Rarity colors the name (§6).
    Combat messages name the base ("The kobold's spear glances off your armour."). The item
    lists add what it gives in numbers after the name (§6 "Item lists": "a Sharp long sword
    (+0 hit, 1d8+2)"), counting its affixes only once it is identified.
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
    to an armour's Protection, which adds up with every other piece's). An `elemental` affix
    (Flaming, Freezing, Venomous; §8 "Damage types") is an attack effect too: its die of its kind rides on
    every roll of the damage, after the weapon's own added damage, one die per kind (the sides
    add up); a `resistance` affix (of the Salamander, of Warmth) is global: its percentage adds
    to the wearer's, capped with the rest at 75. The rest are global: the six stats,
    maximum hit points, regeneration, maximum mana, mana regeneration (§8 "Mana"), Dodge (stacks), Deflection and natural armour (the best
    counts, with the creature's own), moving speed, carrying limit, stealth (quieter footsteps,
    §5 "Hearing"), **light** (the wearer carries the brightest light among its glowing items,
    a steady preset; `LightCache` adds it as a carried source, so creatures see by it too), and
    **skills** (§8 "Skills": a skill affix adds to that skill's value for what it gives,
    `Actor::skill_effect`, trained or not, an untrained one from 0; the Skills screen still lists
    trained values). Each rule reads them in one place: `Actor::effective_stats`, `max_hp`,
    `max_mana`, `World::mana_rate`,
    `defence`, `melee`, `ranged`, `thrown`, `launcher_range`, `throw_range`, `skill_effect`;
    `World::cost_of` (speed); `burden::load`; `noise::noise`; `World::healing_rate`;
    `Stack::weight`. Speeds are at most 50 % off in all; an attack's weapon skill takes its
    `speed` off after them (`World::attack_price`, §8 "Weapon skills"). **System effects** (docs/design/affixes-systems.md §4–§6): mana costs (`mana_cost`), a
    piece's spell cost surcharge (`spell_ease`, local, on body armour, helmets, and shields that
    hinder casting), spell to-hit, criticals, lift, reach, and duration (`spell_to_hit`,
    `spell_threat`, `spell_lift`, `spell_reach`, `spell_duration`), mana on a kill
    (`mana_on_kill`), the level of wands (`wand_level`), a resource's maximum, pace, weapon feat
    costs, and gain on a kill (`resource_max`, `resource_rate`, `active_stamina`,
    `resource_on_kill`: each names its resource in data, `resource = "stamina"`), kicking doors in
    (`door_kick`), digging (`dig_percent`), hunger (`hunger`), trap damage (`trap_damage`),
    disease checks (`disease_check`), and potions kept (`potion_kept`) are effects with words, but
    no affix has them yet, and each is read once its hook lands. The two on a kill
    (`mana_on_kill`, `resource_on_kill`) count only a kill by the wearer's own blow, shot, or
    spell, the slain within three cells (§8e "Mana", "On a kill").
    The spell ones are read (#629), each where its talent's is, unidentified too, on creatures
    as on the player: `mana_cost` joins Frugal caster's cut (§8 "Mana", 50 % off at most),
    `spell_ease` eases its own piece (§8 "Spells": "Cost"), `spell_to_hit` and `spell_threat`
    aim an own cast only (never a scroll's or a wand's), `spell_lift` widens the lift with Keen
    mind's, `spell_reach` adds to the range or length with School focus's, and `spell_duration`
    adds degrees. `caster_level` (of the Adept) is read in `Actor::caster_level`; **Attuned**
    (Scholar, Intelligence 12, level 6) counts the caster level magic items give half again,
    rounded up (+1 → +2, +3 → +5), and **Bellows** (Vigour, Toughness 12) the stamina they add
    to the maximum twice (affixes design §10).
    The utility hooks
    are in (#631), worn by the player or a creature, identified or not: `door_kick` adds to a
    kick's Strength as Lockbreaker's does (a door's and a chest's), `dig_percent` is that much
    more work a turn of digging, `hunger` makes hunger come that percent slower (the pace at
    least 1 % of a human's), `trap_damage` takes that percent off a trap's blow (with Trap sense,
    at most 75 % in all: a trap always hurts, at least 1), and `disease_check` adds to every
    check against a disease, catching and progression alike.
    `potion_kept` and `wand_level` are read (#633): `potion_kept` adds to Thrifty drinker's
    chance (§8 "Potions", 50 % at most unless the talent alone is above it), `wand_level` adds
    to the level a wand casts at with Wand mastery's (§8 "Magic items": "Wands").
    Pending hooks, added
    with the systems they need: affixes that raise a weapon skill's rank (#134's ranks exist;
    none roll yet) and value (#145, a base `value` times rarity and affixes).
  - **Affixes** (`data/affixes/`, §7): 32 prefixes (mostly offensive, flat Defence, and
    life: Sharp, Heavy, Glinting, Keen, Brutal, Guarding, Reinforced, Sturdy, Supple,
    Featherweight, Nimble, Shimmering, Oaken, Hale, Vicious, Accurate, Fleet, Hunter's, Steady,
    Radiant, Burly, Hardened, Mystic (maximum mana; on weapons too), Venomous (poison), Flaming,
    Freezing; the systems' (#635): Mage-wrought (eased casting), Baleful (spell criticals),
    Potent (spell damage), Tireless (stamina), Hobnailed (kicks), Delving (digging)) and 59
    suffixes (stats, speed, criticals, utility, skills: of the Bear, Lynx, Ox, Owl, Sage, Lion;
    of Celerity, Precision, Slaying, Ruin, Mending, Clarity (mana regeneration), Stealth, Light,
    the Mule, Evasion, Warding, the Wind, Reach, Durability, the Tortoise, Vitality, the
    Salamander, Warmth; of Shadows (stealth), the Hare (tumble), the Fox (listen), the Wanderer
    (survival), Balance (balance); the skills' (#636): of Seeking (search), the Eagle (spot), the
    Stag (athletics), the Leech (heal), the Picklock (open lock), Disarming (disable device),
    Appraisal (appraise), Haggling (negotiation, #666), the Scholar (spellcraft), Letters (decipher script), the Cutpurse
    (steal), on the ladder (Spot, Open lock, Disable device, Heal, Steal, and Appraise give nothing to the
    untrained, so their suffixes help the trained only); of Wounding (an extra damage die); the systems' (#635): of the
    Adept (caster level), of Thrift (mana costs), of the Magus (spell to-hit), of Farcasting
    (spell reach), of Lingering (spell degrees), of Siphoning (mana on a kill), of Wands (wand
    level), of Breath (stamina pace), of Ease (weapon feats' stamina), of the Hunt (stamina on a
    kill), of Fury, of Steadiness, of Stillness, of Guile, of the Muse (the maxima of rage,
    focus, ki, openings, inspiration: of no use to one without the pool), of Sustenance
    (hunger), of Wariness (traps), of Health (disease checks), of the Alchemist (potions kept)),
    each with the kinds of item it rolls on and its tiers by item level (at most twelve,
    `MAX_TIERS`; the systems' affixes on a ladder, a tier every item level or two).
  - **Categories** (#621, after Path of Exile's mod groups): every affix
    names a `category`, and an item never has two affixes of one category. Every affix of one
    effect (of one skill, for skill affixes; of one damage type, for elemental and resistance
    affixes; of one resource, for the resource effects) is of one category, so an item also has at most one affix of each effect; a few
    categories hold more than one effect of a family: `stealth` (of Stealth, quieter footsteps,
    and of Shadows, the Stealth skill). Damage is more than one category, so a weapon may roll
    more than one damage affix (#635): `physical_damage` (Sharp and Vicious, +damage),
    `extra_damage` (Heavy and of Wounding, an extra die), and one per element (`fire_damage`:
    Flaming, ...). A category on both prefixes and suffixes is one effect either way
    (`accuracy`: Accurate, Glinting, of Precision; `critical_chance`, `critical_multiplier`,
    `range`, `dodge`, `deflection`, `natural_armour`, `life`, `movement_speed`, `carrying`,
    `light`); the rest are one effect each, named for it (`fire_damage`, `fire_resistance`,
    `strength`, `listen`, ...). The wiki's affix tables name each affix's category.
  - **Loot** (`magic::loot::roll`): the item level is the depth (a branch level's depth, a
    hand-made level's `item_level`, 1 elsewhere). One roll draws, in order: for a find of grade
    `any`, whether it is a loose gem (§7 "Gems": one draw against `chance`, then the gem by
    `rarity` among the gem materials by id, its size and its cut by their weights at the item
    level; `gem::find`), which ends the roll; the base (among the loot objects with `level` at
    most the item level, by `frequency`, sorted by id; scrolls left out of loot that must be
    magic or better, and drawing no rarity: they are normal); a stack's count (ammunition 8 to
    20); the rarity (by weight, those below the grade asked for left out); how many affixes
    (magic: a prefix, a suffix, or both by the weights of `[magic]`; rare: the count by weight,
    then how many are prefixes, evenly among the splits that keep at most three of each, so
    prefixes and suffixes are alike: four affixes are one, two, or three prefixes, a third
    each); each affix, prefixes first (by
    weight among those that may roll: the item's kind in its bases, something for its effect to
    change, a tier at the item level, a category not yet on the item), its tier (evenly among
    those at the item level whose level is within three below it, `TIER_WINDOW` = 4: an item of
    level 12 rolls the tiers of levels 9 to 12; the highest it may have alone once the item level
    is past the affix's last tier, or when none is that close), and its value (evenly in the tier's range); a rare item's name (two
    draws); for an item of a class with an `[[ornament]]`, whether it is set with a gem (one
    draw against its chance, then the gem, size, and cut, never a rough cut; `gem::ornament`);
    and for an item whose construction list lets it be made of other materials, its material (one
    draw by the materials' `rarity` at the item level, or for a creature's drop their `gear` at
    the creature's level; §7 "Materials", `variant::draw`). With nothing left to roll an item has
    fewer affixes, with none it is normal. Loot is found unidentified (its gems and material
    show).
  - **Where**: generated branch levels' floors hold `loot` items (the Depths 2 to 4) on plain
    floor where nothing lies, stands, shines, or leads away, and vault `loot` cells an item at
    least of their grade; outdoor ruins' loot cells too (local areas have no floor loot); all
    from the level's `loot/<level id>` stream once its map is valid (the map is the same as
    without loot). Creatures with `loot` drop a magic or rare item (grade magic) where they die,
    besides what they carried, by their chance, from the stream `drop/<actor id>`, at the
    creature's level as the item level, or the level's item level where that is higher (an
    out-of-depth creature drops a deeper item), made of a material by the creature's level (its gear: `roll_from` with
    `Source::Gear`); by the gems' `chance` (one draw first) the drop is a loose gem instead.
  - **Chest loot** (`chest::roll_contents`, §8 "Chests"; docs/design/chests-and-treasure.md
    §4): a generated chest holds what its band's `loot` table rolls, from the level's chest
    stream: `items` dice (one roll), then per item one `below(100)` for its grade (under `rare`
    rare, under `magic` at least magic, else any: loot's own chance of a gem first) and the loot
    roll at the floor's item level + `loot_level` with the finder's luck (Treasure hunter, Hoard
    sense), nothing more when nothing is found; then the purse, `gold.dice` × `gold.each` coins
    of the shops' currency (one roll; none at 0); then `gems` dice (0 or less: none), each a
    loose gem rolled at the same level (`gem::roll`). Like stacks join. A coffer (`1d3`, 10 %
    / 1 %, `2d6` gold, `1d4-3` gems) holds on average 2 items, 7 gold, and a gem one time in
    four; a banded chest (`2d3+1` at +1, 25 % / 3 %, `3d6` × 3, `1d2`) 5 items, 31.5 gold, and
    1.5 gems. A mimic standing in for a chest carries the same stacks from the same draws (§8 "Mimics").
  - **Unique items** (`content/unique.rs`, data `data/uniques/` and each campaign's
    `data/uniques/`, §7; #423, docs/design/adventurers-guild.md §9): one of a kind, like ADOM's
    artifacts: a **rare** item with a set name, set affixes and parts, an item level, a
    **power**, and a line of **lore**; no new rarity or color. Its `Magic` adds `unique` (its id,
    saved like the affixes), by which its power and lore are found. Content places one as
    `unique:ID` wherever it names objects (a pile, a legend's `objects` and `carry`, and `wield`
    or `wear` by the same name, a gear line, a kit, a shop's or a scenario's `carry`, a quest
    reward's `items`); each is named by **one placement in all content** (a load error
    otherwise), so with pinned vaults and unique creatures a run holds it at most once, with no
    save state. Loot never makes one; a gear line's `magic` never rerolls one. Identification is
    as for any magic item: unidentified it is "a stiletto (rare)"; identified its name, affixes,
    power in words, and lore (the item description's last line), all through
    `Stack::known_magic`. A power works while the item is worn or wielded, identified or not,
    and is read in one place each:
    - **Inflict** (`{ inflict = STATUS, chance, severity }`): a hit with it that leaves its
      target alive causes `severity` degrees of the status by `chance` percent (one draw of the
      gameplay stream below 100, none at 100), read where a natural attack's `on_hit` is
      (`World::after_blow`), through the attack's profile (`Profile::on_hit`, from
      `Mods::local` of the weapon that hit: the main hand's, the off hand's for its blows, a
      launcher's for its shots, a thrown weapon's; never a kick or a shield bash). The target's
      resistance to the status's kind applies, as to any `on_hit`.
    - **Vision** (`{ vision = PROFILE }`): its wearer sees with that profile where it is better
      than their own eyes, part by part (the lower threshold, the farther darkvision, ultraviolet
      if either sees it), the name staying their own: read where eyes are chosen
      (`WorldState::eyes` for the player, before talents; `Actor::eyes` for creatures' sight).
      The dev `vision:` override is shown as it is.
    - **Ward** (`{ ward = STATUS, message }`): its wearer never suffers the status, read in
      `World::afflict`, which refuses it and tells the player `message` ("Fear finds no hold on
      you.").
    The four of the Ascmearc (the Weyfarers' Gilde's criminals carry them): Widowmaker
    (stiletto: Venomous, of Celerity, of the Lynx; every hit poisons), High Toby (long bow:
    Hunter's, Steady, of Precision; a hit frightens at 25 %), the Unhalwed Bedes (bone amulet:
    Mystic, of Clarity, of the Owl, of Shadows; darkvision), and the Turncoat (banded mail:
    Reinforced, Hardened, Hale, of the Ox; never frightened), each at item level 12, placed by
    its owner's gear line (Mother Sallow, Black Wystan, Brother Osrick, Sergeant Herrick); the test
    world's Stonebiter lies in the quarry.
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
  there is nothing to learn about ("You already know what a dagger is.").
  - **Unidentified scrolls** (`scroll_label.rs`, data `data/scroll_labels.toml`; #393): a
    readable scroll type (an object with `read`) is unknown until the player identifies it,
    and meanwhile shows by a **label**, "a scroll labelled ZUNBAR" (`name`/`plural` with
    `{label}`), in one plain colour for every unknown scroll (`fg`), everywhere the player is
    shown it: inventory, piles, shops, look, the item description, messages, and player-facing
    dumps (all through `Lore::describe`). One label per type for the whole run, all different,
    made of 2 to 3 of the file's syllables, drawn when the world is made or loaded from the
    run's `identification` stream (§8 "Determinism"), types in object id order: derived, never
    saved, never drawn per read. What the player knows is the set of **identified scroll
    types** (`WorldState::known_scrolls`), saved and hashed. A type is identified when a scroll
    of it is read ("You read the scroll labelled ZUNBAR." then "It is a scroll of fireball."),
    when the scroll of identify is read on a stack of it ("This is 2 scrolls of lightning!"), the
    scroll of identify's own type by its first read, when it is in the character's starting kit
    (a chosen character knows their kit's scrolls), or by the dev `identify` (and F8). Reading an
    unknown scroll works as reading a known one does (a gamble: what it does is not known until
    it is done); a label changes no outcome, only names. A scroll that cannot be read (a
    sealed letter) has no label and is known on sight. **Identified is not learned**: a spell
    scroll whose type is identified shows by its name and can be studied (§8 "Spells":
    "Learning") by anyone who has not learned its spell; study is refused only for a scroll still under its label and for
    a spell already learned. `Lore::is_identified(object)` is what the study list asks.
  - **Scrolls of spells** (#397): every spell of `data/spells/` comes on a scroll of its own
    (`read` of kind `spell`), which casts it through the one cast path (§8 "Spells") at its
    scroll level, free of mana: aimed as the spell is (`read:b:12,5`), or at once for a spell on
    oneself (`read:b`; refused, no time, for an aimed one without its cell). Its blows are the
    reader's; one that kills them is the scroll's (`Cause::Scroll`); Spell recall may keep it.
  - **The three older spell scrolls** (fireball, frost, lightning; M29) cast a trap's spell
    (§8 "Traps") rather than their spell's, with the same rules, data, and code
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
  - **The scroll of identify** identifies one unidentified stack the player carries: a magic
    item's affixes, a labelled scroll's, wand's, or rod's type, or a wand's charges
    (`World::identify`: "This is a Sharp dagger of the Lynx!"); a stronger "identify all" can
    come later. It is a scroll of the spell Identify (#592, §8 "Spells"): read, it casts it free
    of mana through the one cast path (`read:b:c`); studied, it teaches it. It is the commonest scroll in loot (`frequency` 300 against the spell scrolls'
    20 to 60, by tier) and a regular general-store item.
  - **Where**: loot (by `level` and `frequency`, never magic), general stores' stock (they sell
    and buy the scrolls class, and a generated one always lays out scrolls of identify, §7
    "Shops"; Cēaptūn's has three), and the trap hall beside its arrival (two each of fireball,
    lightning, and frost, three scrolls of identify, and an unidentified dagger). The showcase
    has none. Spellcraft and Decipher script (§8 "Skills") help studying a scroll (§8 "Spells":
    "Learning"), not reading one.

- **Wands** (`warlock-core/src/world/zap.rs`, data `wand` in `data/objects/` §7, `content/wand.rs`;
  the magic design §9; #398): objects (`-`, the wands class, one at a time) whose `[object.wand]`
  holds a spell of `data/spells/` (an aimed one), its `charges` (default its tier's
  `wand_charges` in `data/spells.toml`: 15, 12, 9, 7, 5, 4, 3, 3, 2, 2 for tiers 0 to 9), and the
  `caster_level` it casts at (default its tier's comfortable level). A wand goes in the
  **missile weapon slot** (`slot = "missile_weapon"`), so a wand and a launcher compete for it
  (the swap is the `ready` action, as for any missile weapon), and is used as a missile weapon
  is: `t` aims it (`AimKind::Zap`, the spell's own aiming by its shape and the player's reach
  once the wand's type is known, a plain line before), `fire:X,Y` in replay text.
  - **Casting**: through the one cast path (§8 "Spells", `World::wand_cast`), with the spell's
    own resolution (an `attack` spell rolls to hit with the user's Dexterity, as every spell
    attack does; `auto` and `area` ones do not), reach (the user's Willpower), dice at the
    **wand's caster level**, never the user's (plus Wand mastery's +2 and worn `wand_level`
    affixes'), held to the spell's cap.
    A charge pays instead of mana; a cast refused (at oneself, out of reach, no one to touch)
    uses none and takes no time; one that goes off takes the `cast` action's time (times the
    spell's `casts`). Its kills are the user's; one that kills its user is the spell's
    (`Cause::Spell`). "You zap the oak wand.", then the spell's own words.
  - **Empty**: a wand with no charges left does nothing ("Nothing happens.", no time) and stays
    an item (to sell, or keep); the player then knows it is empty. Wands are **never
    recharged** (the owner); there is no Use magic device skill and there are no magic staffs.
  - **Identification**: a wand type is unknown on sight, as a scroll's is: "an oak wand" in one
    plain colour (`data/wand_labels.toml`, the scroll labels' format; one label per type per
    run, drawn after the scrolls' from the `identification` stream, so no scroll's label
    changes). It is identified when one is used ("It is a wand of fireball.") or identified;
    every wand of it then shows by its name. Its **charges** (`Stack`'s `charges`: the charges
    used, saved, left out of the save while the wand is as made) stay unknown until the wand
    itself is identified, by the scroll of identify or by Connoisseur's carrying ("a wand of
    fireball (7 charges)", "(empty)"). The item description says the spell in words ("Holds
    Fireball (tier 3): ...") once the type is known, and the charges left once identified.
  - **The wands**: magic missile and burning hands (tier 1, item level 3, frequency 10, 250
    gold), scorching ray, sleep, and web (tier 2, level 5, frequency 7, 400 gold), fireball and
    lightning bolt (tier 3, level 8, frequency 5, 600 gold). **Where**: loot (by `level` and
    `frequency`, never magic), general stores (they sell and buy the wands class), and the trap
    hall's pile beside its arrival (a wand of magic missile and one of fireball).
  - **Talents**: Wand mastery (Weaponcraft, system `spells`): wands cast at caster level +2.
- **Rods** (`warlock-core/src/world/rod.rs`, data `rod` in `data/objects/` §7 and
  `data/rod_effects/`, `content/rod.rs`; the magic design §9; #399): objects (`-`, the rods
  class, one at a time) whose `[object.rod]` holds an **effect no spell has** (`effect`, an id of
  `data/rod_effects/`) and the `mana` a use costs (1 to 100). A rod goes in the **missile weapon
  slot** and is used as a wand is (`t`, `AimKind::Zap`; `fire:X,Y`); the aiming mode shows its
  reach once its type is known.
  - **Mana, not charges**: a use costs the user the rod's `mana`, less Channeler's quarter
    (rounded to the nearest, at least 1), spent through the one mana spend (Frugal caster's cut,
    a Willpower training mark). Without it the rod does nothing ("You have not the mana to use
    it.", no time), nor while a status blocks casting (Raging: "You cannot use a rod while
    raging."). A rod never runs out and has nothing of its own to save.
  - **Caster level**: the **user's** (§8 "Spells"), never the rod's: an effect's reach is its
    `range` plus a cell per `per_levels` caster levels, at most its `max_range`; a status
    effect with `up_to_caster_level` takes only creatures whose level is at most it ("The ogre
    shrugs it off."). No skill applies: there is no Use magic device skill (the owner).
  - **Effects** (`data/rod_effects/`, one `[[rod_effect]]` a file: `shape` line, bolt, or burst,
    its reach, and a `kind` on existing code): `dig` (a line toward the cell aimed at: every wall
    of hardness below its `power` breaks as if dug through, `World::break_wall`; the first that
    does not, a door, or the level's edge stops it), `teleport` (a bolt: the first creature on it
    is moved as by a teleport trap, an attack on it as a blow is: a shopkeeper is provoked),
    `status` (a burst at the cell aimed at, within reach: its status on everyone in it but the
    user, as an area hex's). The first four: **Digging** (line, power 1000: stone and packed
    earth, never granite), **Teleport other** (bolt), **Sleep** (burst radius 1, Asleep, up to
    the caster level: a true sleep that a noise or a blow ends, unlike the Sleep spell's
    drowsiness), **Terror** (burst radius 1, Frightened 3, up to the caster level).
  - **Use**: refused, taking no time and no mana, at oneself, a burst out of reach, without the
    mana; otherwise "You zap the pewter rod.", the effect, and the `cast` action's time. A bolt
    that meets no one still costs its mana and time ("The bolt strikes nothing.").
  - **Identification**: a rod type is unknown on sight, "a pewter rod" (`data/rod_labels.toml`,
    the scroll labels' format, drawn after the wands' from the same stream, so no scroll's or
    wand's label changes), until one is used ("It is a rod of digging.") or identified. The item
    description then says the effect in words and its cost before the user's talents ("Holds
    Digging: ...", "Costs 6 mana a use, before your talents; it never runs out").
  - **The rods**: terror (item level 3, frequency 6, 5 mana, 450 gold), digging (4, 6, 6 mana,
    500 gold), sleep (6, 5, 9 mana, 700 gold), teleport other (10, 3, 12 mana, 900 gold).
    **Where**: loot (by `level` and `frequency`, never magic), general stores (they sell and buy
    the rods class), and the trap hall's pile beside the wands (a rod of digging and one of
    sleep).
  - **Talents**: Channeler (Spellcraft, from level 5, systems `spells` and `mana`): rods cost a
    quarter less mana, before Frugal caster.
- **Potions** (`warlock-core/src/world/potion.rs`, data `drink` in `data/objects/` §7,
  `content/potion.rs`; the combat progression design §10; #565): objects (`!`, the potions
  class, stackable) whose `[object.drink]` says what drinking one does by its `kind`, each
  through the one path that does it: `heal` (`dice` hit points back at once, never past the
  maximum, and what its `cures` names cured), `cure` (`cures`, means of the statuses' and
  diseases' `cured_by`: `bandage` stops bleeding, `antidote` ends poison; `diseases = true` ends
  every disease), `mana` (`dice` mana back, never past the maximum), `elixir` (#568: its `stat`
  +`points` (1) for good, each a trained rise that lifts the potential with the score where it
  was at it, then the potential +a roll of `potential`, "1d2", one gameplay draw; hit points
  follow a Toughness rise at once, the maximum mana and the carrying limits follow the score;
  "You feel stronger.", with ", and that you could be stronger yet." when the potential rose by
  2), `harm` (`dice` degrees of its `status`, Poisoned: sickness, as rotten food gives), and,
  for a while (#567), `boost`
  (`severity` degrees of its `status`, one that raises a stat: Fortified, Nimble, Hale, Keen,
  Steadfast, Imposing, +2 a degree for ten minutes a degree, the stronger of two doses kept),
  `vision` (its `status` lends eyes for half an hour: Cat-eyed low-light, Owl-eyed darkvision,
  Moth-eyed the ultraviolet sight), `haste` (`severity` degrees of Hasted, the Haste spell's
  status: a minute a degree, everything a quarter quicker), and `resistance` (its `status`
  resists as gear does for half an hour, 50 %: Warmed fire, Chill-proof cold, Earthed
  electricity, Proof against poison). A status lent says its own line as it begins ("You feel
  stronger.", "Your eyes open wide to the dark."); a stronger dose of one already felt says it
  again, and one that changes nothing "Nothing more comes of it." The status bar and the
  character screen name it as they name Blessed, the raised score shown raised, never a number
  or a timer. Every potion names its training
  `marks` by stat (5 for a common one, 10 for a greater, 20 for an elixir; none for a harm),
  given as sure marks through the `drank` use (an elixir's through the `elixir` use; §8
  "Stats").
  - **Elixirs** (#568): the elixirs of might, grace, vigour, wit, will, and presence (Strength,
    Dexterity, Toughness, Intelligence, Willpower, Presence), gold `#e8c860`, item level 8,
    frequency 2, value 2000: deep loot, the forgotten altar's great boon `elixir` (§7 "Special
    rooms"), and very rarely an alchemist's stock; never a general store's or a provisioner's
    (their `never` list, `data/shops.toml`). The description says "Makes you stronger for
    good"; never a number, never the potential.
  - **Drinking: Consume** (§6 "Consume"): no action of its own. `Action::Eat` (`eat:b`,
    `eat:floor:a`) hands a stack with a `drink` table to `World::drink_potion` instead of
    starting a meal; it needs no hunger. It takes the `drink` action's time (one turn) and one of
    the stack, from the pack or the floor (in a shop, a potion drunk is bought: "You drink it,
    you buy it: 60 gold for the fizzy red potion."), says "You drink the fizzy red potion." and,
    for an unknown type, "It is a potion of healing.", does what it does ("You feel your wounds
    close.", "Your mind clears, and power wells up in you.", "It tastes foul!"), and gives its
    marks. Drinking while Raging is allowed. Creatures do not drink.
  - **Unknown potions**: a potion type is unknown on sight, "a fizzy red potion": a word drawn
    per type for the run (`data/potion_labels.toml`, the scroll labels' format with `{colour}`,
    drawn after the rods' from the same stream, so no scroll's, wand's, or rod's label changes;
    a word a type, at least as many words as types, checked as content loads) and the type's own
    **colour**, fixed in data (`colour`, its `fg` the object's; potions of one colour share
    one `fg`, checked). The colour follows one rule: each effect family has a hue, and a potion
    of several effects mixes theirs: healing red, cures green, healing and cures russet, mana
    blue, stat boosts orange (greater ones deep orange), vision violet, speed yellow, resistance
    teal, harm brown. The colour hints at the family, never the type. The `!` shows in it,
    known or not. A type is known (`WorldState::known_scrolls`) once one is drunk, identified
    (the scroll of identify, an appraisal), or in the character's starting kit (the witch's
    potion). The item description says what a known one does in words ("Heals wounds at once,
    and cures bleeding"), never dice; an unknown one "You do not know what this potion does."
    Prices are true.
  - **The potions**: potion of healing (heal 2d8; item level 1, frequency 60; 40 gold; red),
    greater healing (4d8, cures bleeding; 4, 30; 120; russet), full healing (8d8, cures bleeding
    and poison; 10, 10; 300; deep russet), antidote (cure poison; 2, 40; 60; green), cure
    disease (every disease; 4, 25; 150; pale green), mana (3d8; 3, 30; 100; blue), and the
    murky draught (harm, Poisoned 2; 1, 20; 10; brown); the potions of might, grace, vigour,
    wit, will, and presence (boost, severity 1; 3, 20 each; 150; orange) and their greater
    potions (severity 2; 8, 8 each; 400; deep orange; 10 marks), of cat's eyes, owl's eyes, and
    moth's eyes (vision; 3, 15 each; 100; violet; Willpower), of quickness (haste 1; 6, 10;
    250; yellow; Dexterity), and of warmth, frost, earthing, and proofing (resistance; 4, 15
    each; 120; teal; Toughness). **Where**: loot (by `level` and
    `frequency`, never magic), general stores and provisioners (they stock and buy potions), the
    alchemist (§7 "Shops": potions only, healing always first), quest rewards, and the hub's
    pile.
  - **Talents** (system `potions`): Thrifty drinker (Delver, from level 6): a potion is not used
    up one time in four (one gameplay draw a drink, none while the chance is 0); worn
    `potion_kept` affixes add their percent to it, the sum at most 50 (a talent's own chance
    above 50 stands); Nose for draughts
    (Scholar, Intelligence 12): an unknown potion picked up tells its kind by its smell, never
    its type ("The fizzy red potion smells of herbs: a cure of some kind."); Lasting draughts
    (Vigour, Toughness 13; #567): boost and vision statuses last half again
    (`status_lasts`, each degree half again as long to ease); the witch's class ability
    Cauldron-wise (at 12): a healing potion heals a fifth more (`potion_heals`) and the boosts
    last a fifth longer.

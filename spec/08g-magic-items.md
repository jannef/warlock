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
    and a two-word name: a first word and a second for its kind, "Grim Bite"); a *unique*
    item is a rare one with a set name and a power (below). An item never has two affixes of the same effect.
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
    `speed` off after them (`World::attack_price`, §8 "Weapon skills"). Pending hooks, added
    with the systems they need: affixes that raise a weapon skill's rank (#134's ranks exist;
    none roll yet), hunger (#137), and value (#145, a base `value` times rarity and affixes).
  - **Affixes** (`data/affixes.toml`, §7): 23 prefixes (mostly offensive, flat Defence, and
    life: Sharp, Heavy, Glinting, Keen, Brutal, Guarding, Reinforced, Sturdy, Supple,
    Featherweight, Nimble, Shimmering, Oaken, Hale, Vicious, Accurate, Fleet, Hunter's, Steady,
    Radiant, Burly, Hardened, Mystic (maximum mana), Venomous (poison)) and 27 suffixes (stats, speed, criticals,
    utility, skills: of the Bear, Lynx, Ox, Owl, Sage, Lion; of Celerity, Precision, Slaying,
    Ruin, Mending, Clarity (mana regeneration), Stealth, Light, the
    Mule, Evasion, Warding, the Wind, Reach, Durability, the Tortoise, Vitality; of Shadows
    (stealth), the Hare (tumble), the Fox (listen), the Wanderer (survival), Balance (balance)),
    each with the kinds of item it rolls on and up to four tiers by item level; an item has at
    most one affix of each effect (of each skill, for skill affixes).
  - **Loot** (`magic::loot::roll`): the item level is the depth (a branch level's depth, a
    hand-made level's `item_level`, 1 elsewhere). One roll draws, in order: for a find of grade
    `any`, whether it is a loose gem (§7 "Gems": one draw against `chance`, then the gem by
    `rarity` among the gem materials by id, its size and its cut by their weights at the item
    level; `gem::find`), which ends the roll; the base (among the loot objects with `level` at
    most the item level, by `frequency`, sorted by id; scrolls left out of loot that must be
    magic or better, and drawing no rarity: they are normal); a stack's count (ammunition 8 to
    20); the rarity (by weight, those below the grade asked for left out); how many affixes
    (magic: both by `magic.both`, else prefix or suffix evenly; rare: the count by weight, then
    how many are prefixes, evenly within at most three of each); each affix, prefixes first (by
    weight among those that may roll: the item's kind in its bases, something for its effect to
    change, a tier at the item level, an effect not yet on the item), its tier (evenly among
    those at the item level), and its value (evenly in the tier's range); a rare item's name (two
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
    besides what they carried, by their chance, from the stream `drop/<actor id>`, at their
    level's item level, made of a material by the creature's level (its gear: `roll_from` with
    `Source::Gear`); by the gems' `chance` (one draw first) the drop is a loose gem instead.
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
    it is done); a label changes no outcome, only names. A scroll that cannot be read (the
    scroll of light) has no label and is known on sight. **Identified is not learned**: a spell
    scroll whose type is identified shows by its name and can be studied (§8 "Spells":
    "Learning") by anyone who has not learned its spell; study is refused only for a scroll still under its label and for
    a spell already learned. `Lore::is_identified(object)` is what the study list asks.
  - **Scrolls of spells** (#397): every spell of `data/spells/` comes on a scroll of its own
    (`read` of kind `spell`), which casts it through the one cast path (§8 "Spells") at its
    scroll level, free of mana: aimed as the spell is (`read:b:12,5`), or at once for a spell on
    oneself (`read:b`; refused, no time, for an aimed one without its cell). Its blows are the
    reader's; one that kills them is the scroll's (`Cause::Scroll`); Spell recall may keep it.
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
  - **The scroll of identify** identifies one unidentified stack the player carries: a magic
    item's affixes, or a labelled scroll's type (`World::identify`: "This is a Sharp dagger of
    the Lynx!"); a stronger "identify all" can come later. It is the commonest scroll in loot
    (`frequency` 300 against the spell scrolls' 50 to 60) and a regular general-store item.
  - **Where**: loot (by `level` and `frequency`, never magic), general stores' stock (they sell
    and buy the scrolls class, and a generated one always lays out scrolls of identify, §7
    "Shops"; Cēaptūn's has three), and the trap hall beside its arrival (two of each spell, three
    scrolls of identify, and an unidentified dagger). The showcase has none. Later:
    Spellcraft and Decipher Script (§8 "Skills") may change reading.

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
    **wand's caster level**, never the user's (plus Wand mastery's +2), held to the spell's cap.
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

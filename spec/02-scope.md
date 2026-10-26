# Warlock specification — 2. Scope of the first phase

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

## 2. Scope of the first phase

Build the engine and framework, not the game. Everything built is production quality (§10).

**In scope**
- GPU glyph renderer with lighting, glyph animation, and effects
- Hand-authored levels from text files; multiple levels joined by stairs, each persistent
- Player movement (8 directions), doors (open/close), stairs
- Doors (M19): locked doors opened with keys (a key for one lock, or a skeleton key for every
  lock), and kicking doors in, by Strength against the door's sturdiness (§8 "Doors")
- Line of sight, terrain memory, gameplay lighting, vision types
- Test creatures walking fixed patrol routes; from M10 they act at their own hidden speed (§8
  "Speed"), belong to factions (§7 "Factions") that are hostile or neutral to the player and to
  each other, hear noise, see by the player's rules (§5 "Creature sight"), and fight with
  deterministic AI (§8 "Creature behavior")
- Pickable objects (glyph, color, material, weight), inventory, and creature weight (M11)
- Equipment (M16): ADOM-like slots in which the player and creatures wear and wield what they
  carry, an equipment screen, and the time it takes to put things on and off (§6 "Equipment")
- Combat (M16): melee blows and shots rolled with 2d12 against Defence (D&D-like, with threat
  ranges and confirmed criticals), weapon and armour data, ADOM-like hit points from Toughness,
  race, and class, regeneration with game time, and death: creatures die and drop what they
  carry, the player's death ends the run for good (§8 "Combat"). Ranged combat (M16): bows, a
  crossbow, a sling, and throwing weapons, ammunition spent and picked up again, ADOM-like
  targeting, missiles flying on screen, and flaming arrows that light the dark where they land
  (§8 "Ranged combat"). Blood (M17): big hits and kills stain floors and walls by the victim's
  size and how the blow landed, subtly, adding up where many die (§8 "Blood"). Spells stay
  announced only
- Status effects and passive healing (M24): a data-driven framework of effects on actors that
  tick and wear off in game time, stack, can be cured, lower stats, slow healing, and can kill;
  bleeding from big hits and confirmed criticals, poison from a giant rat's bite; hit points come
  back twice as fast while resting (§8 "Status effects", "Healing"). Diseases (M24, #139): one
  umbrella status, Diseased, over six pathologies in the D&D 3.5 spirit that are caught by a
  failed check, incubate unseen, progress daily with a check each (a loss costs stat damage),
  are shaken off by two checks won in a row (grave rot never, and it can kill), and slow healing;
  stat damage mends a point a day once they are gone (§8 "Checks", "Diseases")
- Moons with phases and moonlight, weather starting with rain (M12)
- Temperature and ice (M26): a fixed temperature underground, under the open sky one that follows
  the month, the time of day, and the weather, always shown in the status bar; water that freezes
  slowly in the cold and thaws in the warmth, ice of tracked thickness that holds actors by their
  weight (what they carry included) and gives way under the heavy, falling in and climbing out,
  and bursts of frost, heat, and blasts as hooks for spells and explosions (§5 "Temperature",
  "Ice"). Nothing feels the cold yet (no hypothermia, warmth, or clothing)
- Action costs: what an actor does and its speed decide how long it takes (M13)
- Stats (M14): Strength, Dexterity, Toughness, Intelligence, Willpower, and Presence, inspired by
  D&D, with D&D 3.5 modifiers and ADOM-like potentials that the player never sees; every actor
  has them, and the player sees their own scores on a character screen (§8 "Stats"). Combat uses
  Strength, Dexterity, and Toughness (§8 "Combat"), and so do burden and kicking doors in
  (Strength, §8 "Doors"); status effects and stat damage may lower them (§8 "Status effects",
  "Diseases"); diseases are resisted with checks of Toughness or Willpower (§8 "Checks"); no
  other rule uses stats yet
- Experience (M21, M30): creatures have a level and an XP value; the player's own kills (melee,
  shots, throws) earn XP, scaled down for monsters below the player's level as in World of
  Warcraft, and so do the first kill of each creature type and the first arrival on a level;
  levels (to 60, on a curve written in kills, times the race's and the class's experience
  factors) grow the hit points up to level 20 (§8 "Experience")
- Burden (M16): a carrying limit from Strength, Toughness, and size; carrying more than a third,
  two thirds, or all of it slows every action, caps Dexterity in Defence, makes steps louder, and
  overloaded, keeps the actor from moving (§8 "Burden")
- Traps (M18): arrow, spear, fireball, lightning bolt, pit, alarm, and teleport traps (data),
  hidden until the player sets one off or sees a creature do it, then known and remembered;
  creatures walk onto them too; a few hidden on every generated Depths level, and a trap hall to
  try them (§8 "Traps"). No saving throws
- Searching (#140): secret doors (level legends, vaults, a door or two on each Depths rooms level)
  and hidden traps, found with one check: searching (`s`, `S` for a while; Intelligence and the
  Search skill) or in passing (Willpower and Spot; elves, dwarves on stone, anyone trained in
  Spot), rolled afresh every turn (§8 "Searching")
- Magic items (M29, #149), after Path of Exile: base items (weapons, missile weapons,
  ammunition, armour, helmets, cloaks, gloves, boots, shields, rings, amulets) roll prefixes and
  suffixes by rarity (normal, magic: one of each at most; rare: up to three of each and a name),
  in tiers by item level (the depth); each item records its affixes (tier and rolled value) for
  enchanting later. Their effects map onto rules that exist (to-hit, damage, criticals, Defence,
  hit points, regeneration, stats, attack and moving speed, carrying, stealth, light). Loot on
  generated levels' floors, in vaults, and dropped by creatures; found unidentified (rarity
  known, affixes not, effects applying); an item description screen (§8 "Magic items")
- Damage types (#151): every blow, burst, bolt, fall, and plunge is slashing, piercing,
  bludgeoning, fire, cold, electricity, acid, or poison (a data list); Protection soaks the
  physical kinds only, by kind; creatures and gear resist kinds in percent; a flaming arrow and
  magic weapons add damage of another kind; only cuts and stabs bleed (§8 "Damage types")
- Digging (M19): walls of terrain that breaks give way to a digging tool (a pick-axe, a mattock)
  ready in a new Tool equipment slot, turn after turn like a rest, by the tool's power and the
  digger's Strength against the wall's hardness, and to explosions stronger than they are hard;
  they become rubble (§8 "Digging")
- NPCs and dialogue (M28): creatures who talk (data, `data/dialogue.toml`): chatting (`C`) gets
  a rotating one-liner, or opens a small dialogue screen of topics; some mutter to themselves,
  overheard through hearing. Moving into a creature that is not hostile swaps places with it,
  unless it will not make way (§8 "Dialogue", "Displacement")
- Quests (M28): simple, ADOM-like errands people give through their dialogue topics (data,
  `data/quests.toml`): kill (creatures of a type, optionally on one level), bring (objects from
  the pack), reach (a level), and deliver (an item the giver hands over, to someone else); offered,
  accepted or declined, followed in a quest log (`q`), and rewarded with gold, items, and
  experience (§8 "Quests")
- Character creation (M15): every new run chooses a race, a gender, and a class, which set the
  player's starting stats (partly rolled with dice, ADOM-like) and eyes; a random age by race
  and a random birthday give the birth year, whose horoscope sign (12 mythological animals × 5
  elements) adjusts the stats too (§8 "Character creation"); race and class also give hit points
  (§8 "Combat"), trained skills (§8 "Skills"), and starting ranks in weapon skills and fighting
  styles
- Skills (M20): 26 skills in data, ADOM-like; race and class train the player in some (dice),
  creatures may list theirs, and every other skill is untrained (never shown, gives nothing).
  Values grow without a hard cap (slowly past 100) by skill rolls at each level-up, whose dice
  are set by the training marks meaningful use earns (ADOM's ladder), and the passive ones help: tumble (Dodge in Defence),
  stealth (quieter steps and doors), listen (hearing fainter sounds), balance (less burden
  slowdown), survival (quicker journeys); a Skills screen lists the player's (§8 "Skills")
- Weapon skills (M22): ADOM-like marks and ranks 0 to 10 in sixteen weapon categories (shield
  blocking and shield bashing among them) with per-category passive benefits, and *The Riddle of
  Steel*'s defaults between related categories; a weapon skills screen and a shield bash
  (§8 "Weapon skills")
- Fighting styles (M22): eight styles after AD&D 2e's *Complete Fighter's Handbook* (Duelist,
  Twin Blades, Great Weapon, Sword and Board, Marksman, Skirmisher, Brawler, Staff Fighter), each
  a skill with the weapon skills' ranks, marks, and per-rank passive benefits, trained by fighting
  in its configuration; fighting with a light weapon in the off hand: two blows an attack, at
  penalties Twin Blades eases (§8 "Fighting styles")
- Hunger (M23), ADOM-like: a nutrition counter that game time burns (resting and overworld
  journeys too) by the race's need; Hungry, Weak, Fainting, and Starving, and Satiated and
  Bloated, with their effects; food to eat from the pack or the floor in meals of several turns;
  creatures leave corpses that weigh what they did, rot, and may do something when eaten (§8
  "Hunger")
- Save and resume an unfinished run
- Keyboard controls with an in-game controls reference
- Development interface (§9)
- Procedural generation with reproducible seeds (M6): dungeon branches of generated levels
  (rooms and corridors, caves) with hand-made vaults, joined to the hand-authored levels by stairs
- An overworld with travel time, locations, local areas, and day and night (M9)
- Special rooms (M25), ADOM-like: now and then a room of a generated level (or an area a
  hand-made level marks) carries an effect that changes the rules inside it — a shadowed room, an
  echoing hall, a hushed chamber, a sanctuary, a flooded room — told the first time the player
  enters (§7 "Special rooms")
- Shops (M27), ADOM-like: a general store and specialty shops with named shopkeepers, buying,
  paying, selling, and theft; one in the courtyard, now and then one in the Depths (§7 "Shops")

**Out of scope for now**: magic (spells are announced, nothing is damaged; only traps burst into
fireballs and lightning, and scrolls cast them), rules that use stats beyond combat, burden, and kicking doors in (ability checks,
saving throws, training stats), levels and experience, digging down or up, creatures that dig,
what a class gives beyond starting stats, hit points, skills, weapon skills, and fighting
styles (equipment, spells), active uses of skills (picking locks, healing, and the like), item effects
beyond magic items' affixes, food's, and scrolls' (§8 "Hunger", "Scrolls"; no potions to use
yet, no unique items, no enchanting), audio, mouse input, infravision, localization.

Vision types are in scope even though there are no other mechanics yet: they are part of perception,
and the light data model must carry them from the start.

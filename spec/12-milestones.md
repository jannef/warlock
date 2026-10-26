# Warlock specification — 12. Milestones

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

## 12. Milestones

Each milestone ships a playable build.

- **M0 Foundation** — Rust toolchain, cargo workspace, Linux CI plus wasm check, CLAUDE.md with
  pinned docs, lints, cargo-deny, xtask; a window opens and `--capture` works in CI.
- **M1 Walking skeleton (first playable)** — grid renderer v1, TTF atlas, level format and loader,
  hub v0, 8-direction movement, walls, doors, turn pipeline, bindings and `?` overlay, message line,
  scenario/seed/capture/freeze-time flags, downloadable builds.
- **M2 Perception and dev loop** — symmetric shadowcasting, memory, perception snapshot, patrol
  creatures, stairs and level persistence, vast level v0, replay/record/dump/state hash, save and
  resume v1 with dev snapshots, property and snapshot tests.
- **M3 Light and vision** — gameplay light grid (RGB + UV), wall-face rule, vision profiles, visual
  light compositing with flicker, interpolated lighting, darkvision fade, debug overlays; light hall
  and darkness chamber.
- **M4 Renderer quality and effects** — free-glyph layer, glyph animations (wave, pulse, shimmer,
  cycle), glyph particles, bloom, fades, distortion, movement slide, camera, HiDPI and resize,
  golden screenshots per room; water garden and effects room.
- **M5 Settings and accessibility** — settings persistence, reduced effects, font and scale, color
  palettes, rebinding, main menu with new run and continue.
- **M6 Procedural generation** — RNG streams, generators, vault embedding, connectivity validation,
  seed UX, snapshot tests.
- **M7 Ship** — packaging per OS, tagged releases, web build, performance budgets verified, crash logs.
  Released as v0.1.0.
- **M8 Lighting polish** — creature shadows, weak bounce light, walls lit from their own distance,
  fungus glow, still and running water, per-source light stability (hue and intensity drift), and
  along the way the screen stack (one way to open, route keys to, and draw modal screens), look
  mode, and a smaller web build.
- **M9 Time, sunlight and overworld** — game clock and 12×30 calendar, time costs per level and
  terrain, resting; sunlight, sky light and moving shadows on sky-exposed levels over a 24-hour
  day (moonlight followed in M12); an ADOM-like overworld with per-terrain travel time,
  locations, and sight by daylight; outdoor local areas generated from overworld terrain.
- **M10 Creatures** — an ADOM-like speed system (base speed 1000, never shown to the player),
  factions with hostile or neutral relations, deterministic AI that approaches and announces melee,
  ranged, and spell attacks (no combat resolution), and hearing: actions make noise that the
  player and creatures hear.
- **M11 Objects and inventory** — pickable objects that each define glyph, color, material, and
  weight; piles on the floor, pick up and drop, the inventory screen; creature weight (base plus
  carried).
- **M12 Moons and weather** — configurable moons (one by default: 30-day period, faint purple
  light) with phases, paths, and shadows; weather starting with rain; the full calendar date and
  moon phase in the status bar.
- **M13 Action costs** — every action has a cost in energy (data, 1000 a normal action; creature
  types may give their attacks their own), and the cost and the actor's speed decide how long it
  takes over one 6-second base (overworld steps: their travel time); levels lose their own turn
  length; exact over any run of actions and at any speed of the player.
- **M14 Stats** — D&D-inspired stats (Strength, Dexterity, Toughness, Intelligence, Willpower,
  Presence) on every actor, with hidden D&D 3.5 modifiers and hidden ADOM-like potentials; the
  player's own scores on a character screen (`@`).
- **M15 Character generation** — character creation on New run: race (with its eyes), gender,
  and class set the starting stats, partly rolled with dice; a random age and birthday give the
  horoscope sign (12 animals × 5 elements, a 60-year great cycle), which adjusts them too; a
  summary before the run starts; the character saved, replayed, and given on the command line.
- **M16 Combat** — equipment: ADOM-like slots (data), an equipment screen (`e`),
  wearing and wielding with their action costs, creatures that wield and wear what they carry;
  the combat core (#113, #116): 2d12 against Defence with categories and miss attribution,
  threat ranges with confirmed criticals, weapon and armour data, hit points from Toughness,
  race, and class, regeneration, death with drops and permadeath, health in words; ranged
  combat (#117): bows, a crossbow, a sling, and throwing weapons, ammunition spent and picked up
  again, ADOM-like targeting, missiles on screen, and flaming arrows that light the dark.
- **M17 Blood** — big hits and kills stain floors and the walls next to the victim (#127): how
  much by the victim's size, overkill, criticals, and missiles; spray away from the blow; blood
  colour per creature type; accumulating, drying, remembered; subtle on the map, with a switch.
- **M18 Traps and spell effects** — traps (#128): arrow, spear, fireball, lightning bolt, pit,
  alarm, and teleport traps in data, hidden until sprung in the player's sight and then known and
  remembered, set off by creatures too, an ADOM-like confirmation before stepping onto a known
  one, a few hidden on every generated Depths level, and a trap hall to try and watch them; spell
  effect rendering (#129) draws their fireballs and lightning.
- **M19 Doors and walls** — destructible walls (#131): terrain hardness, digging through walls
  with a pick-axe or a mattock in a new Tool slot, turn after turn by tool power and Strength,
  explosions that break the softer walls, rubble, and the quarry to dig in. Locked doors and keys
  (a key for its lock, a skeleton key for every lock; creatures that open doors need the key
  too), kicking doors in by Strength against sturdiness and kicking creatures (#130); locked rooms
  with reachable keys in the generated Depths, a locked store room in a vault, a locked pantry in
  the hub.
- **M20 Skills** — 25 skills in data (#132): trained or untrained (never shown, gives nothing),
  values from race and class dice and creature data, training marks on meaningful use with a
  provisional increase rule (gentle to 100, slow past it), passive benefits for tumble, stealth,
  listen, balance, and survival, a Skills screen (`S`), and the starting skills on the character
  creation summary.
- **M21 Experience** — creatures have levels and XP values; the player's own kills earn XP, with
  World of Warcraft scaling for monsters below (down to grey, worth nothing) and above the
  player's level, a quadratic level curve to 50 in data, and hit point growth up to level 20;
  level and XP on the status bar, the character screen, and the death screen (#133).
- **M22 Weapon skills, styles, stances** — weapon skills (#134): ADOM-like marks and ranks 0 to
  10 in sixteen categories with per-category passive benefits, The Riddle of Steel's defaults
  between related categories, shield blocking and a shield bash, starting ranks from race and
  class, and a weapon skills screen (`W`); fighting styles (#135): eight styles with the same
  ranks, marks, and per-rank benefits, trained in their configuration, starting ranks from
  class, light weapons in the off hand and two blows an attack with two weapons, a shield bash
  after a hit, and a Fighting styles section on the weapon skills screen.
- **M29 Magic items and scrolls** — magic items after Path of Exile (#149): a broad set of base
  items with stats, 21 prefixes and 26 suffixes in tiers by item level, normal, magic, and rare
  items (up to three prefixes and three suffixes, generated names), each recording its affixes;
  loot on generated levels, in vaults, and dropped by creatures; unidentified until identified
  (the `identify` API), effects applying all the same; rarity colors and an item description
  screen. Scrolls, the scroll of identify among them, follow (#150).


- **M23 Hunger and food** (#137) — ADOM-like hunger: a nutrition counter that game time burns by
  the race's need (data, `data/hunger.toml`), the states from Bloated to Starving with their
  effects, food eaten from the pack or the floor in meals of several turns (`E`), overeating,
  corpses dropped by every creature that dies, rotting, and doing what their creature's
  `on_eat` says; starting food and a pantry in the showcase.

- **M25 Special rooms** — ADOM-like special rooms (data, `data/room_effects.toml`): a
  shadowed room, an echoing hall, a hushed chamber, a sanctuary, and a flooded room, in about one
  generated level in two or three and in areas hand-made levels mark (the hub's stairs nook is a
  sanctuary); one query for the rules (`World::room_effect_at`), a first-entry message, and look
  mode naming rooms the player has been in.
- **M27 Merchants and shops** — ADOM-like shops (#145): a value in gold on every object, a
  general store and four specialty shops in data (`data/shops.toml`), named shopkeepers in a
  merchants faction, buying (prices in look mode and on pick-up, a debt, paying with `p` or when
  leaving), selling (offers for dropped goods), prices moved by Presence and Appraise (never
  shown), theft that turns the keeper hostile, free goods once it is dead; Grimbold's General
  Store in the courtyard and a shop in about one generated level in six.

- **M24 Status effects and diseases** — a data-driven status-effect framework (#138): effects
  with severities that tick, ease, stack, and can be cured in game time, lower stats, slow
  healing, and can kill; bleeding from big hits and criticals, poison from a giant rat's bite;
  healing twice as fast while resting; shown by name in the status bar, on the character
  screen, and on creatures in look mode. Diseases (#139): one umbrella status, Diseased, over six
  pathologies (filth fever, red ache, the shakes, marsh ague, grave rot, brain fever) with
  incubation, daily progression, checks (2d12 + a stat modifier against a difficulty, hidden),
  stat damage that mends a point a day once they are gone (Toughness damage lowers the most hit
  points), recovery by two checks won in a row (grave rot never; it can kill); caught from the
  giant rat's bite, the marshes and swamps, and rotten food; named on the character screen
  after their first progression.

- **M26 Temperature and ice** (#142) — a temperature on every level, fixed underground and
  following the month, the time of day, and the weather outdoors, always in the status bar (°C
  or °F); water that freezes and thaws lazily with game time, ice of tracked thickness that
  holds by weight and gives way under the heavy, falling in and climbing out, the Frost Grotto,
  and bursts of frost and heat for spells and explosions to come.

- **M28 NPCs and quests** — NPCs and dialogue (#146): creatures who talk, by type or as unique
  characters, with rotating one-liners, a greeting, topics on a dialogue screen (`C`), and
  mutters overheard through hearing; a gardener and a wandering bard in the courtyard and an old
  hermit by the hub's stairs. Displacement (#147): swapping places with anyone who is not
  hostile, unless they will not make way. Quests (#148): kill, bring, reach, and deliver quests
  offered through dialogue topics, accepted or declined, followed in a quest log (`q`, info page
  5), handed in for gold, items, and XP, optionally requiring another or repeatable; the hermit's
  rats and then the Depths, the gardener's apples, the bard's letter.

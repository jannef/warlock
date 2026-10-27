# The outlaws

The Weyfarers' Gilde's wanted criminals and their hired blades (#424, guild G4;
`docs/design/adventurers-guild.md` §8). Content only, on the fields of `docs/dev/creatures.md`,
plus one naming rule (below).

- **Faction** `outlaws` (`data/factions.toml`, after the cult): hostile to the player only, and
  listed as hostile by no faction, so a lair's inhabitants never fight their master or the
  footpads. Every type of it carries the `outlaw` tag (`data/creatures/README.toml`).
- **Types**, all the Ascmearc's own (`campaigns/ascmearc/data/creatures/`, human, evil, open
  doors, `xp = 5 × level + 10`):
  - `footpad` (level 6, common): short sword (racial weapon, `short_blades = 2`), a leather
    jerkin three times in four, stealth 25.
  - `footpad_archer` (level 7, uncommon; tags `outlaw`, `archer`): short bow (racial weapon,
    `bows = 2`), 15 or 25 arrows, a dagger and a jerkin each half the time.
  - The four criminals, `unique = true`, level 12, `very_rare`, loot 50, numbers as in design
    §8.2: `sallow` (Mother Sallow; speed 1100, footsteps 10, `low_light`, poison 100; wields
    `unique:widowmaker`), `wystan` (Black Wystan; `ranged`, range 8, sight 14; wields
    `unique:high_toby` with 40 arrows, a short sword and a jerkin), `osrick` (Brother Osrick; a
    mace, wears `unique:unhalwed_bedes`, which gives his normal eyes darkvision; cold and poison
    50), `herrick` (Sergeant Herrick; a great axe, magic at 50 %, wears `unique:turncoat`, whose
    Ward keeps Frightened off him). Each unique is placed by its owner's gear line at chance 100
    (the one placement in all content: a test that needs a placement of its own defines a test
    unique, `world/unique_tests.rs`). `osrick.toml` opens with the design's TODO note on his
    spells (§8.2, §15); `outlaw_tests` checks it is there.
- **Where**: the four lairs (#426, `docs/dev/guild.md`): pinned vaults on each lair branch's
  fourth floor place the criminals (and two footpads in Wulfeshol's hoard); the lairs'
  populations hold the footpads (Wyrtholm), the footpads and their archers (Wulfeshol,
  Brocenburg). The criminals are in no population.
- **Proper names**: a creature type whose `name` starts upper-case is a proper name
  (`content::is_proper`), said without an article: `content::the` ("the kobold", "Black
  Wystan") and `content::the_start` ("The kobold") build every message that names a seen
  creature (`fight.rs`'s `End`, deaths, falls, pits, waking, a creature's throw, the "no
  quarrel" refusals, `World::called`, `World::creature_called`), and look mode names it as it
  names an npc ("Black Wystan (hostile, unhurt), ..."). The quest texts already did
  (`content::one_of`). A corpse still takes the type's name ("Black Wystan corpse").
- **Tests**: `world/outlaw_tests.rs` (faction and tags, the gear rolls, stances against every
  lair population, a lit den where outlaws and lair types wait 40 turns unhurt, the powers
  working for their owners, the proper names, the note); `world/bestiary_tests.rs`'s `WAVES`.

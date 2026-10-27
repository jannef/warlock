# Warlock specification — 3b. The first campaign: the Ascmearc

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

## 3b. The Ascmearc

The design is `docs/design/campaign.md` (M32); campaigns as such, and the showcase as the test
campaign, are §3 "The showcase is a test campaign". The Ascmearc (`assets/campaigns/ascmearc/`,
kind `campaign`) is **the world a release plays**: a release build holds no other campaign, so
every new run starts in it.

- **The release start** (#341, design §3.5): New run lists the start scenario of every campaign
  of kind `campaign` and, in developer builds (debug, or `--dev`), every scenario of every
  campaign under its name, the test world's too (§6 "Main menu"). A release build therefore has
  one row, the Ascmearc's start, and **skips the step**: New run opens on character creation's
  race (§6 "Character creation"), and Esc there goes back to the main menu. The run's seed is
  drawn as for any release run (#256). The run starts on Beorggeat at 08:00 on 11 Darkening 1444
  with the arrival line "The pass opens on the Ascmearc. Far to the east, Fyrbeorg smokes."
  `--scenario ascmearc` starts it from the command line; the dev scenarios `ceaptun` and
  `fyrdelf` (in the town, on the branch's first level) are listed in developer builds only.
- **The Characters screen** (§6 "Characters"): each saved character's summary keeps its
  campaign's name, shown beside the place only when more than one campaign is installed (a
  developer build: "In Cēaptūn (The Ascmearc)."). A release build has one, and says just "In
  Cēaptūn."
- **The overworld** (#332, design §5): 120×60 cells, about 5 000 of them walkable, three to four
  days across by road. The player comes in by **Beorggeat**, a pass in the western wall, a
  closed way back (no stairs, no link; look mode: "The way you came; snow has closed it behind
  you."). **Cēaptūn**, the market town, lies five road cells east. Roads run east over the
  barrow downs to the ford over Blæcwæter, south over the bridge on Beorhtburna, and north to
  the holly wood. The volcano Fyrbeorg smokes on the south face of the north-eastern range. The
  status bar names the region (the level file's `[[region]]`s, first match wins; every walkable
  cell is in one).
- **Names** (design §5.8): a place is named by a plain English description put into Old
  English, in spellings the glyph atlas draws (æ for ǣ, y for ȳ; no wynn or yogh); ids are ASCII
  slugs without the marks (`ceaptun`, `cealdhlaw`). People are named in the same register
  (Hild, Leofric, Godgifu, Wulfrun); creature names stay plain English.
- **Locations**: the pass, the town (§3 "Cēaptūn"), and three dungeon branches of rising
  danger: **Cealdhlāw** (the cold barrow, visible from the start, in the downs east of the
  town), **Fennmynster** (the fen minster, hidden in Micelfenn), and **Fyrdelf** (the fire
  diggings, hidden in Ascdūn, §7 "Fyrdelf"). A hidden one is its plain terrain until a quest
  reveals it (§7 "Hidden locations"). The rest of the design's forty sites are reserved: plain
  cells today, spaced so a later level drops in without moving anything.
- **Quests** (#340, design §6.2; `campaigns/ascmearc/data/quests/`): three people in Cēaptūn
  give a chain of three each, every quest offered once the one before it is done:
  - **Reeve Leofric**: "Rats in the barrows" (kill 5 giant rats in Cealdhlāw; 25 gold, 30 XP),
    "The barrow's heart" (reach Cealdhlāw 4; 60 gold, 100 XP), "The wight of the barrow" (kill
    the barrow wight in Cealdhlāw; 150 gold, a chain mail shirt, 250 XP).
  - **Sister Hild**: "Grave goods" (bring the silver chalice from the barrow chamber, Cealdhlāw
    2; 40 gold, 60 XP), "The fen minster" (reach Fennmynster 3; it **reveals Fennmynster** as it
    is given; 80 gold, 2 angelica roots, 160 XP), "What the water keeps" (bring the minster
    reliquary from the sunken crypt, Fennmynster 8; 250 gold, 450 XP).
  - **Factor Wulfrun**: "The foreman's ledger" (deliver her ledger to Godgifu at the Gæsthūs; 10
    gold, 20 XP), "The lost shaft" (reach Fyrdelf 2; offered once the ledger and "The barrow's
    heart" are both done, since Fyrdelf starts at danger 5, deeper than Cealdhlāw 4; it **reveals Fyrdelf** as it is
    given; 100 gold, 220 XP), "Fire in the deep" (kill the fire drake in Fyrdelf, a named
    monster: its death counts however it came; 500 gold, 900 XP).
  - **Wat One-Eye** (#425, the guild design §4, §7), guildmaster of **the Weyfarers' Gilde**
    (Middle English: a new organisation) in the **Gildhalle**, a timber hall in the town's
    south-east corner with its bounty board by the door: a chain of rolled quests, each drawn as
    it is given and never drawn again, each offered once the one before it is done and with no
    other gate (level, alignment, or another quest): "A bounty" (kill one of a creature type
    drawn from the non-unique populations of Cealdhlāw, Fennmynster, and Fyrdelf, those the
    player can enter; 40 gold, 60 XP), "The hundred" (kill 100 of a creature tag drawn from
    kobold, goblin, undead, vermin, marsh-thing, duergar, and fire-thing, those living in a
    branch the player can enter; 250 gold, 2 red potions, 500 XP), "A scroll for the Gilde"
    (bring a scroll drawn from those loot can hold; 200 gold, 150 XP), "Wanted" (kill one of
    four wanted criminals, drawn from the quest's four variants, each revealing its own lair as
    it is given: the other three stay hidden for the run; a named monster, met by its death at
    any hand; 500 gold, 800 XP, alignment 10). Being given the first makes the player a **Tin**
    of the Gilde, the second done a **Pewter**, the fourth done an **Iron**; the ranks are
    seven, titles named by metals of the game's materials ordered by their value (Tin, Pewter,
    Iron, Steel, Duralumin, Electrum, Gold; `campaigns/ascmearc/data/guilds/weyfarers.toml`),
    the later ones reserved for the chain's later links.
  - **The four lairs** (#426; the guild design §8): four hidden branches of four floors each,
    danger 6 to 9, on reserved sites of the overworld, each with its criminal's pinned vault on
    the fourth floor and no shops: **Wyrtholm** ("herb islet", Mother Sallow; caves 100×40 under
    an islet in Micelfenn at (88, 46), 9 °C; the still-room), **Wulfeshol** ("wolf's hole",
    Black Wystan; caves 100×40 in Deorcholt at (18, 49); the robbers' hoard), **Bānfeld** ("bone
    field", Brother Osrick; rooms 80×21 under a barrow field on Hlāwdūn at (42, 31), 8 °C,
    shadowed and hushed rooms, rusty keys; the unhallowed chapel), **Brocenburg** ("broken
    fort", Sergeant Herrick; rooms 80×21 under the ruined tower in Holegnwudu at (66, 12), iron
    keys, ogres only out of depth; the captain's hall). Their populations are the design's
    §8.2 with the footpads. Sister Hild has a line on Osrick.
    The chain is likely a requirement of the story's ending, to come.
  Every quest of the three helps someone and is a little good (`alignment` 5 to 30); of Wat's,
  only "The hundred" is (5). Each reach and bring
  quest and the delivery say in the log where their goal lies (`where`). The chalice, the
  reliquary, and the ledger are objects of their own, worth nothing to a shop. The three
  branches' first levels each have an `arrival` line: "Cold air breathes up from the mound.",
  "The water is up to the third step.", "The galleries smell of sulphur."
- **The player wiki** describes the Ascmearc and never the test world: a page "The Ascmearc"
  with the names and their meanings, the regions, and the overworld map drawn from the level
  file with the hidden locations on it (the wiki tells the truth, design §11.4), plus the places
  table; the scenarios, creatures, and dialogue tables list only campaigns of kind `campaign`.

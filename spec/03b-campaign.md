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
- **The player wiki** describes the Ascmearc and never the test world: a page "The Ascmearc"
  with the names and their meanings, the regions, and the overworld map drawn from the level
  file with the hidden locations on it (the wiki tells the truth, design §11.4), plus the places
  table; the scenarios, creatures, and dialogue tables list only campaigns of kind `campaign`.

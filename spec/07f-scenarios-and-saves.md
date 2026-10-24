# Warlock specification — 7. Levels and data formats (continued): scenarios, saves, settings, replays, dumps

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Scenarios** (`assets/scenarios/*.toml`): level set, start position, lantern, options, when the
  run starts: `start_date = [year, month, day]` (all from 1; default `[1444, 11, 11]`, 11 Darkening,
  Year 1444) and `start_time = "hh:mm"` (default `"08:00"`), and the player's character when the run
  does not choose one: `character = { race = "human", gender = "female", class = "thief" }` (ids in
  `races.toml` and `classes.toml`; gender `male`, `female`, or `other`), from which the player's
  eyes, stats, and hit points follow. A scenario without a character (test fixtures, content without
  races or classes) gives them itself: `vision` (default `normal`), `stats` and `potential` as for
  creature types (default 10 each, the potential the score), and `hp` (the hit points beyond
  Toughness a race and a class would give, 0 to 1000, default 0); with a character these four are
  refused; such a player is trained in no skill (§8 "Skills"). A scenario may give a starting
  pack, `carry = ["ration*3"]` (as level files place objects; no corpses): on top of the
  character's starting kit when the player chose the character (§8 "Character creation":
  New run, `--character`, `--name`; stackable things join the kit's), else alone (a launch on
  the scenario's own character: `--scenario` alone, tests, golden walks). No built-in scenario
  gives one: the kits bring food, and more on top would burden the smallest. Every
  built-in scenario has a
  character with normal eyes: the showcase a human female thief, the arena a halfling male thief,
  the courtyard a human female druid, the depths and the vast level a human male warrior, the forest
  a human female ranger, the world a human male ranger. The `character` table may name them
  (`name = "Tamsin"`, up to 20 characters): the name a run of the scenario told none gives its own
  character (§8 "Character creation"); every built-in scenario names its own (Tamsin, Pip, Rowena,
  Aldric and Roderick, Maren, Osric, Garrick, Bram).
- **Saves**: suspend-only. One save per character, written as a run starts, on quit, and by
  periodic autosave; loading continues the run. Dev builds add named snapshots. Format: a versioned envelope (magic, format
  version, game version) around a JSON payload, gzip-compressed with a pure-Rust backend, written
  atomically (temp file + rename). Before 1.0, incompatible saves are rejected with a clear message;
  from 1.0 on, saves are migrated. The format version is bumped once per release by the
  integrator, never by a feature change (`docs/dev/merging.md`): saves are not promised
  across development builds, and a save a build cannot read is kept as `run.sav.bad`. The web build keeps the same bytes in the browser's
  `localStorage` (below). The envelope also records a content fingerprint (hash of the ordered
  terrain, creature, level, light, dungeon-branch, object, equipment slot, skill, room effect,
  weapon skill, status, disease, trap, and shop
  ids), because ids are indices; a save made with different content is rejected. Generated levels
  are saved with the run like visited hand-made ones (terrain, memory, stairs, lights, kind,
  arrival point,
  edge exit, piles of objects and the piles as remembered), together with the gameplay random
  stream, the game clock, every actor's scheduler energy (§8 "Speed"), the direction of the last
  overworld step (saves without the step load with none), what each actor heard and has not yet
  acted on, the sounds the player perceived lately (§5 "Hearing"; saves without them load with
  none), what every actor carries, and what each creature has in mind (where it last saw a
  hostile or heard a noise to look into, and the post it left; §8 "Creature behavior"; saves
  without it load with every creature calm), every actor's stats, scores and
  potentials (§8 "Stats"), the player's character (name, race, gender, and class,
  the stat the gender raised, birth date, and birth sign; §8 "Character creation"), the player's
  alignment score and the alignment a level gave a creature, if any (§8 "Alignment"; a score off
  the scale does not load),
  what every actor wears and wields (which inventory stack is in which slot, §6
  "Equipment"), every actor's hit points (what it has left, what its race and
  class or its type add to its Toughness, and the seconds banked toward the next one back; §8
  "Combat"), the fires burning on each level (§8 "Ranged combat"), the player's
  experience total (§8 "Experience"; absent while
  0, so a run without kills hashes as before) and the creature types whose first kill paid
  (absent while none), every actor's trained skills (value and training
  marks) and the training random stream (§8 "Skills"), each level's special
  rooms and whether the player has been in each (§7 "Special rooms"), every
  actor's weapon skills (its rank and marks in each, §8 "Weapon skills"), the
  work done on walls dug into and still standing (§8 "Digging"), every
  actor's combat stance (§8 "Stances"), the free blows each actor is given (absent while none,
  §8 "Free blows"), the blood on each level and the blood
  as last seen (§8 "Blood"), every actor's status effects (each status with
  its severity, its progress toward its next tick and easing, and who caused it), the progress
  toward the next hit point back counted in hundredths of a second at the normal pace, and the
  status effect that killed the player (§8 "Status effects", "Healing"), the
  player's satiation (their nutrition, the burn banked toward the next point, and the seconds
  toward the next hit point lost while starving), every corpse's death time, and what killed the
  player when neither a creature nor a status effect did (§8 "Hunger"), the
  locks on each level's doors (§8 "Doors"), how far each actor's talk has got
  and the unique character a level made it (§8 "Dialogue"; left out while
  nothing has been said, or none), each level's temperature underground, the ice on its water
  (thickness and the water beneath), and when its ice was last brought up to date (§5
  "Temperature", "Ice"), every actor's diseases (each with the seconds to
  its next turn, whether it shows, its progressions so far, its checks won in a row, and who gave
  it) and stat damage (with the seconds banked toward mending it; §8 "Diseases"),
  every magic or rare item as it is (§8 "Magic items": on its stack wherever it lies or is
  carried, its rarity, item level, each affix by id with its tier and rolled value, a rare item's
  name, and whether it is identified; absent on normal stacks; an affix this
  content does not have, a tier or value out of range, or more affixes than the rarity allows
  makes the save inconsistent; affixes are recorded by id, so adding affixes does not change the
  content fingerprint), the player's quests and the kills they count (§8 "Quests";
  left out while the player has heard of none and killed nothing a kill quest asks
  for; quests and creature types are recorded by id, so a save naming one this content does not
  have is inconsistent), the traps on each level (kind, whether the player knows it, charges
  left, a bolt's direction), the known traps as the player remembers them, and a trap as what
  killed the player (§8 "Traps"), every actor's fighting styles (after its
  weapon skills, §8 "Fighting styles") and a light weapon in the off hand,
  each level's shops (the keeper, its ways out, what the player owes, and how the keeper stands;
  §7 "Shops"), and a dev weather override while
  it lasts (§5 "Weather"; none otherwise, so saves without one load as they are). The weather
  itself is not saved, nor the temperature under the open sky: they follow from the seed and the
  clock. **One save per character**: each character's save is `saves/<id>.sav` in the platform
  data directory, its id a slug of the name and the lowest free number (`brakka-1`; a second
  Brakka is `brakka-2`), and beside it `saves/<id>.toml`, what the list of saved characters shows
  (name, race, class, level, place, and when last played, wall-clock time, shown in UTC), so the
  list reads no save. The run in progress knows its id, and every way of saving writes that
  character's files only: a run starting (so a new character is listed at once), quitting from the
  pause menu (closing the window saves too), autosave every 50 turns and on level change, the web
  page-hide save, and the dev snapshots. A new run never touches another character's save. The
  main menu's Continue resumes the character played last, and starting without the menu resumes it
  unless `--new` is given; the Characters screen lists them all (§6) to resume one or delete it.
  A save that cannot be read is renamed `<id>.sav.bad` when resuming it fails, never deleted, and
  the list no longer shows it. **Permadeath**: when the player dies that character's save (its
  summary and snapshot too) is deleted, and no other character's; a run that has ended is never
  saved again (every trigger that would save it deletes the save instead; on the web nothing is
  kept for the page being hidden). Dev snapshots: `F5` saves and `F9` loads `snapshots/<id>.sav`,
  the character's own (debug builds or `--dev`). A save of the single-save layout before this
  (`run.sav`) is left where it is, never read: the next release's format version rejects it
  anyway.
  **File locations and portable mode** (`paths.rs` decides them all): per folder, an explicit
  `--config-dir` / `--save-dir` wins, then `--portable DIR`, then a `warlock-data` folder next to
  the game (the executable's folder; on macOS, the folder holding `Warlock.app`, never inside the
  bundle), then the platform's standard folders. A portable folder is laid out `config/`
  (`settings.toml`, key bindings; `warlock-keys.toml`, exported keys), `data/` (`saves/`, `snapshots/`), `logs/` (logs, crash reports).
  `--portable` alone uses (and creates) `warlock-data` next to the game. A portable folder that
  cannot be written is reported on stderr and in Settings > Files, and the game runs on: what is
  there is read, writes fail with their usual messages. `warlock --paths` prints each location with
  the reason it was chosen, then exits; Settings > Files shows the same with an Open folder action
  per location (the platform's opener: `open`, `xdg-open`, `explorer`; a failure is shown, never
  fatal) and, on the web, says everything is in the browser's storage. `warlock --move-data native`
  moves the portable folder's files into the standard folders and renames the emptied folder to
  `warlock-data.moved-<date>`; `--move-data portable [DIR]` does the reverse (DIR defaults to
  `warlock-data` next to the game). Each file is copied beside its destination, compared, renamed
  into place, then removed from the source; a file already at the destination is kept as
  `NAME.bak-<time>`, never overwritten; an error stops the move with every file complete in one
  place; `--dry-run` prints the plan only. It refuses while another Warlock runs (every running game
  holds a shared lock on `logs/.in-use`).
  On the web the files are `localStorage` keys `warlock/saves/<id>.sav`, `warlock/saves/<id>.toml`,
  `warlock/saves/<id>.sav.bad`, `warlock/snapshots/<id>.sav`, and `warlock/settings.toml`, the bytes
  base64-encoded, with the same envelope and checks; the list of saved characters comes from the
  keys under `warlock/saves/`. The same triggers save (autosave, level change, quit), and so does the page
  being hidden or closed (`visibilitychange` to hidden, `pagehide`); main menu Continue works after a
  reload. Storage that is unavailable (some privacy modes) turns saving off with a warning in the log;
  a write that fails (quota) shows a message in the message line; neither panics.
- **Settings**: a separate file in the platform config directory.
- **Replays**: JSON action logs with a header (game version, scenario, seed, character, the
  character's name) and per-turn state hashes; a replay without a name gives the character the
  run's default name, so older replays load.
- **Dumps**: JSON simulation state plus ASCII layers.

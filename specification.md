# Warlock — Specification

Status: refined draft v2 (2026-10-17)
Repository: https://github.com/jannef/warlock — used for code, documentation, issues, and pull requests.

**This specification is the source of truth for what to build.** It is split into one file
per section under `spec/` (a large section into several, in reading order); this file is
the index. Code follows the spec, and the spec, the developer notes (CLAUDE.md and
`docs/dev/`), and the player wiki change together with behavior. Section numbers are
stable: a reference like §8 "Combat" means the bullet **Combat** in section 8, whichever
file holds it, and "below" or "above" beside such a name points at that bullet, in this file
or the next of its section. A feature adds its bullets to the files of the sections it touches,
next to the related ones, never at the end.

## Sections

- [`spec/01-vision.md`](spec/01-vision.md) — **1. Vision**: what Warlock is and the targets
- [`spec/02-scope.md`](spec/02-scope.md) — **2. Scope of the first phase**: the engine and framework phase: everything in scope, feature by feature
- [`spec/03-showcase-world.md`](spec/03-showcase-world.md) — **3. Showcase world**: the hub's rooms, the arena, the courtyard, the vast level, the depths, the overworld and its areas, scenarios
- [`spec/04-rendering.md`](spec/04-rendering.md) — **4. Rendering**: glyphs, layers, glyph animation, visual lighting, perception states, motion and time, readability and reduced effects, effects (particles, glow, post, rain, blood, spell effects)
- [`spec/05-visibility.md`](spec/05-visibility.md) — **5. Visibility, light, and vision**: line of sight, light (sources, falloff, bounce, wall faces, special rooms), sunlight, moonlight, weather, temperature, ice, brightness, visibility, overworld sight, vision profiles, memory, hearing, creature sight, the perception API
- [`spec/06-input-and-ui.md`](spec/06-input-and-ui.md) — **6. Input and UI**: screens and the dialog, menus, settings and rebinding, the inventory, equipment, character, skills, weapon skills, quest log, and death screens, chat, rest, eat, dig, messages, attacks, targeting, look mode
- [`spec/07a-levels.md`](spec/07a-levels.md) — **7. Levels and data formats: level files**: the TOML/JSON rule, level files and legends (objects, locks, traps, water, temperature and ice in level files)
- [`spec/07b-data.md`](spec/07b-data.md) — **7. Levels and data formats (continued): data definitions**: definitions (terrain, creatures, lights, vision, ...), action costs, the sky, objects and materials, magic items, weapon skills, equipment slots, factions, dialogue, quests, the weather, spell effects
- [`spec/07c-generation.md`](spec/07c-generation.md) — **7. Levels and data formats (continued): generation**: dungeon branches, the overworld, the generators, vaults
- [`spec/07d-shops-and-rooms.md`](spec/07d-shops-and-rooms.md) — **7. Levels and data formats (continued): shops and special rooms**: shop data and levels, special room effects
- [`spec/07e-characters.md`](spec/07e-characters.md) — **7. Levels and data formats (continued): races, classes, and the rules' data**: stat adjustments, races, status effects, diseases, sizes, hunger, classes, alignment, skills, the horoscope
- [`spec/07f-scenarios-and-saves.md`](spec/07f-scenarios-and-saves.md) — **7. Levels and data formats (continued): scenarios, saves, settings, replays, dumps**: scenario files, the save format and what is saved, settings, replays, dumps
- [`spec/08a-architecture.md`](spec/08a-architecture.md) — **8. Architecture**: the pure core, data layout, the perception snapshot, turn phases, time (the calendar, action costs, the scheduler, rest)
- [`spec/08b-creatures.md`](spec/08b-creatures.md) — **8. Architecture (continued): creatures**: creature behavior, dialogue, quests, displacement
- [`spec/08c-combat.md`](spec/08c-combat.md) — **8. Architecture (continued): combat**: combat, experience, stances, ranged combat
- [`spec/08d-traps-doors-digging.md`](spec/08d-traps-doors-digging.md) — **8. Architecture (continued): traps, doors, digging**: traps, doors, searching, digging
- [`spec/08e-blood-and-status.md`](spec/08e-blood-and-status.md) — **8. Architecture (continued): blood, status effects, healing, checks, diseases**: blood, status effects, healing, checks, diseases
- [`spec/08f-stats-and-burden.md`](spec/08f-stats-and-burden.md) — **8. Architecture (continued): stats and burden**: stats, burden
- [`spec/08g-magic-items.md`](spec/08g-magic-items.md) — **8. Architecture (continued): magic items**: magic items: affixes, rarities, loot, identification
- [`spec/08h-hunger.md`](spec/08h-hunger.md) — **8. Architecture (continued): hunger**: hunger, eating, corpses
- [`spec/08i-character.md`](spec/08i-character.md) — **8. Architecture (continued): the character**: character creation, alignment, skills, weapon skills, fighting styles
- [`spec/08j-determinism.md`](spec/08j-determinism.md) — **8. Architecture (continued): dice, events, determinism**: dice, events to effects, determinism
- [`spec/09-development-interface.md`](spec/09-development-interface.md) — **9. Development interface**: command-line flags, the headless CLI, replays, dumps, dev keys
- [`spec/10-quality-bar.md`](spec/10-quality-bar.md) — **10. Quality bar (Definition of Done)**: CI, tests, golden screenshots, performance budgets
- [`spec/11-technology.md`](spec/11-technology.md) — **11. Technology**: Rust, Bevy, the crates
- [`spec/12-milestones.md`](spec/12-milestones.md) — **12. Milestones**: M0 onward
- [`spec/13-github-workflow.md`](spec/13-github-workflow.md) — **13. GitHub workflow**: issues, branches, PRs, CI
- [`spec/14-deferred.md`](spec/14-deferred.md) — **14. Deferred and open**: what is deferred and still open

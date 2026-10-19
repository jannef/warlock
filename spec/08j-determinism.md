# Warlock specification — 8. Architecture (continued): dice, events, determinism

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Dice** (`warlock-core/src/dice.rs`): one type for every roll the rules make (character
  creation, weapon and natural attack damage). Text form: a fixed number (`+2`, `-1`, `0`) or `NdS`
  (`N` dice of `S` sides: 1 to 100 dice, 2 to 1000 sides), optionally negated (`-1d2`) and
  with a fixed number after it (`2d3+1`, `1d4-1`; within ±1000); no spaces, lower-case `d`.
  `Dice` parses, prints its canonical text, gives its `min` and `max`, and rolls from a
  deterministic stream: one uniform draw per die, none for a fixed number. Data files may give
  a fixed number as a TOML integer.
- **Events to effects**: the core emits events (door opened, door kicked or broken, actor moved,
  attacked, died); the
  presentation layer turns them into animations: slides, a short lunge toward the target for a
  melee attack (motion), missiles flying (§4.8) and a creature's `attack`
  particles where its attack lands, e.g. the shaman's violet sparkle (particles); dust where a
  door swings or a wall gives way (the `toggle` particles of what the cell became); a death shows
  as the creature gone and what it dropped, and in the message line; a trap going off
  (`TrapSprung`, `TrapStruck`) shows as the trap now known and in the message line, its arrow as
  a missile, and its fireball or lightning (`Event::Spell`) by the spell effect renderer.
  Gameplay never waits for them.
- **Determinism**: ChaCha8 RNG (`rand_chacha`) with separate streams, each seeded with FNV-1a 64 of
  the run seed (little-endian) and a label: map generation per level (`map/<level id>`, retry *k*
  `map/<level id>/<k>`), so a level depends only on the seed and its id, never on visiting order
  (its special room and then its shop are drawn last, so a level without them is the map it
  would be without them);
  the traps hidden on a generated level (`traps/<level id>`, apart from its map's); a generated
  level's loot (`loot/<level id>`, drawn once its map is valid and its traps hidden, so the map
  is the same with or without it); what a creature drops when it dies (`drop/<actor id>`, so a
  kill never changes a later roll of the gameplay stream; actor ids are never reused);
  gameplay (`gameplay`, kept in the world state and saved; a teleport trap's destination is a
  hash, never this stream, §8 "Traps"); character creation (derived on
  demand: `character/gender` for the stat "other" raises, `character/birth/<race>` for the age
  and birthday, `character/rolls/<race>,<gender>,<class>` for the dice,
  `character/weapon-skills/<race>,<gender>,<class>` for the starting weapon skills,
  `character/styles/<race>,<gender>,<class>` for the starting fighting styles; New run's
  Random rows roll `new-run/<step>/<n>` streams, which never touch a run); training marks
  (`training`, kept in the world state and saved; §8 "Skills"); the weather, which keeps no
  stream at
  all: each weather block hashes the seed, a label, and its number (FNV-1a, then SplitMix64's
  finalizer; §5 "Weather"), nor the temperature and the ice, which follow from the weather, the
  clock, and the levels (only ice giving way under a weight between what it holds and twice that
  draws from the gameplay stream; §5 "Ice"); and visual-only randomness in the app, which never uses either. No hash-order iteration and no floating point in the core's decisions.
  The same seed and actions produce the same result within a game version; seeds are not
  guaranteed stable across versions, and snapshot tests flag when generator output changes. A seed
  is a number or a word (`--seed warlock`, hashed with FNV-1a 64).
- Small modules with clear responsibilities; no speculative abstractions or premature optimization.

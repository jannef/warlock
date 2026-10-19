# Content files and levels

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/07a-levels.md and spec/07b-data.md. Keep it current with the code, as CLAUDE.md says.

Content (`assets/data`, `assets/levels`, `assets/scenarios`, `assets/vaults`) is TOML, embedded into
`warlock-core` at build time by `build.rs`. Level maps are ASCII with a legend; see
`assets/levels/hub.toml` and spec §7. A legend entry with a `creature` may set `faction = "peaceful"` (any id in
`data/factions.toml`) to override its type's faction for that spawn (`Actor::faction_override`; read it with
`Actor::faction`, never the creature type's, when computing stance). The showcase (hub, vast, courtyard) is safe
this way: `xtask/src/vast.rs` emits it for its rats and kobolds; fighting is the arena's and the Depths'. A level
file's optional `arrival = "..."` line is said when the player arrives by stairs. Water: `~` is still water; running water (terrain with a
`flow` anim) needs a flow on every cell from its legend entry (`flow = "S"`, optional
`flow_speed` percent), and the shared legend draws it as arrows (`→` normal, `⇒` double speed).
Flow is presentation data carried by `LevelTemplate::flow` / `LevelState::flow` and read by the
renderer through `PerceptionView::flow` (visible cells only); no rule reads it.

---
name: Feature
about: A new mechanic, system, screen, or content for the game
labels: ["type:feature"]
---

<!-- One issue per PR (or a tight group). The body is the approved plan of the change; where it
leaves a detail open, the PR decides in its spirit and says so. -->

## What

<!-- The change in a few sentences, from the player's side. Link the design document
(docs/design/<topic>.md) if there is one. -->

## Rules

<!-- What warlock-core does: data, state, actions, hooks, determinism, saves. Hidden numbers
stay hidden in the game (the wiki states them). -->

## What the player sees

<!-- Messages, screens, keys (only play-context commands are rebindable KeyCommands, menu/screen/prompt/targeting keys are fixed; abilities only in the one
abilities menu), dumps. -->

## Tests

<!-- Unit, property, or snapshot tests in warlock-core; a replay created and verified; text
snapshots for new screens (`crates/warlock/src/ui/screen_snapshots.rs`; no golden entries unless
only the GPU can show it, CLAUDE.md "Golden screenshots"); `cargo xtask wiki`. -->

## Docs

<!-- The spec section, the docs/dev/ topic, CLAUDE.md if commands change, and the wiki page
(a generated table when the fact lives in data). -->

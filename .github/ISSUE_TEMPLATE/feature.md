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

<!-- Messages, screens, keys (every key a rebindable KeyCommand; abilities only in the one
abilities menu), dumps. -->

## Talents and class abilities

<!-- The coverage policy (spec §8 "Talents", docs/dev/progression.md "Coverage policy"): a new
system of the game ships with its talents.
Which talents and class abilities does this system ship with (at least two talents, naming
their trees; at least one class ability once the class tables have landed), and which system
tag do they carry (a new [[system]] in assets/data/talents.toml)? If it is a resource (mana, a
pool like Rage or Focus), which talents change its maximum, its refill, or its costs?
Or why none: it is presentation only (no rule reads it, as blood stains), or it extends a
system already in the list (name the tag and the talents that already touch it). -->

## Tests

<!-- Unit, property, or snapshot tests in warlock-core; a replay created and verified; golden
entries for new screens; `cargo xtask wiki`. -->

## Docs

<!-- The spec section, the docs/dev/ topic, CLAUDE.md if commands change, and the wiki page
(a generated table when the fact lives in data). -->

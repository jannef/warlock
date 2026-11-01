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

## Talents and class abilities

<!-- The coverage policy (spec §8 "Talents", docs/dev/progression.md "Coverage policy"): a new
broad system of the game (combat, stealth, magic, light, hunger, ...) ships with at least one
talent, under a new [[system]] tag in assets/data/talents.toml (and a class ability once the
class tables have landed). Which talents and trees, which tag? If it is a resource (mana, a pool
like Rage or Focus), which talent changes its maximum, its refill, or its costs?
A talent must earn its place: a real choice at its tier that changes how a build plays or what
it can do. Not information only; not a small percentage on a niche mechanic; not an effect
another talent, class ability, or skill rank already gives (extend that one instead); not
something that matters only in a rare room or against one rare creature.
Capstones are not economy, quests, or XP: from class level 40 on, a class ability changes what
the character can do in a fight or a dungeon (not gold, favour, quest rewards, or XP alone, nor
only outside the dungeon); no XP or quest-reward effect at or above level 30; a quest-reward
effect is judged by its total over the campaign's fixed pool, not by its percent.
A class table names the pickable's id, never a clone of it (two talents with the same effects
fail the load), and a talent whose one talent prerequisite shares an effect kind with it is
that talent's next rank, not a talent of its own (also a load check; deliberate pairs are
listed in `alike`, assets/data/talents.toml).
Or none: a sub-feature (a room effect, a creature family's gimmick, a chest kind, an info
readout) extends its parent system: name the tag and the talents that already touch it. Or it is
presentation only (no rule reads it, as blood stains). -->

## Tests

<!-- Unit, property, or snapshot tests in warlock-core; a replay created and verified; text
snapshots for new screens (`crates/warlock/src/ui/screen_snapshots.rs`; no golden entries unless
only the GPU can show it, CLAUDE.md "Golden screenshots"); `cargo xtask wiki`. -->

## Docs

<!-- The spec section, the docs/dev/ topic, CLAUDE.md if commands change, and the wiki page
(a generated table when the fact lives in data). -->

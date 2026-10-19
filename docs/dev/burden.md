# Burden

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08f-stats-and-burden.md. Keep it current with the code, as CLAUDE.md says.

Burden (spec §8 "Burden"; M16, #115) lives in `warlock-core/src/burden.rs`: `Load` (`carried`,
`limit`, `ignores_burdened`; `burden()` the level by thirds, `penalty()` what the actor suffers:
Unburdened for a Burdened dwarf, `thresholds()`/`over()`), `Burden` (Unburdened, Burdened,
Strained, Overloaded; its effects: `slow(price)` ×1.25/×1.5 rounded up, `max_dex()` +3/+1/0,
`footsteps()` 125/150 %, `can_move()`, `news()`), `limit(st, to, carry)` = 1500 g × (2 St + To)
× carry % (always a multiple of 3, so thirds are exact), and `load(actor, race, content)`.
Sizes are data, `data/sizes.toml` (`content/size.rs`, `SizeDef { id, name, carry }`;
`Content::sizes`/`size`/`size_index`; `medium` required, the default; without the file only
medium); races have `size` and `ignores_burdened` (dwarves), creature types `size` (the
kobolds, the rat, the bat, and the beetle are small). The player's size is their race's when the
run has a character (`WorldState::load(content, actor)` picks it; `World::actor_load(id)` for
dev tools). Nothing is stored (saves are unchanged): the burden follows from the inventory,
stats, and content. Every penalty goes by `Load::penalty`: prices in `schedule.rs`
(`World::burdened`, inside `price_of` and `step_price`: the one place, so every action and
overworld journey is slowed), `World::defence_of`, `make_noise`, and moving (`World::can_move`
refuses steps, edge exits, stairs, and entering a location or local area: "You carry too much to
move."; `creature_steps` makes an overloaded creature wait). `World::apply` says the change
(`burden_news`) right after the player's action. Presentation reads `PerceptionView::own_load`:
the status bar (`Burdened`/`Strained`/`Overloaded` after the HP, priority 1), the inventory
(72 wide; `Carrying 28.4 kg (burdened)` and the thresholds line, `character_screen::thresholds`)
and the character screen.

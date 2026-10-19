# Hunger

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08h-hunger.md. Keep it current with the code, as CLAUDE.md says.

Hunger (spec §8 "Hunger"; M23, #137): the rules are `warlock-core/src/hunger.rs` (`Hunger`, the
seven states from `Starving` up to `Bloated`, ordered so `<` is hungrier; `news(from, to)`,
`stops_rest`; `Satiation { nutrition, banked, starving }` on `Actor::satiation`, the player's
only, with `burn`/`starve`/`feed`), data `data/hunger.toml` (`content/hunger.rs`, `HungerDef`:
`burn_seconds`, `start`, `max`, `vomit_leaves`, `[states]` least nutrition, `bloated_slow`,
`weak` stats, `faint`, `starve`, `exertion` (only `EXERTIONS`: melee, shoot, throw),
`overeating`, optional `corpses` (material, rot hours, rotten vomit/harm, `sickness` (a status
id: harm from food is degrees of it), `Food` per size);
`Content::hunger()`; without the file nobody hungers and there are no corpses), races' `hunger`
percent, objects' `food = { nutrition, turns }`, creatures' `corpse` (default true) and `on_eat`
(`OnEat`), scenarios' `carry` (starting pack). Corpses are objects made at load
(`Content::load_hunger`, after the file's objects, id `<creature>_corpse`, `ObjectDef::corpse`
names the creature, `Content::corpse_of`); their stacks carry `Stack::died` (crate-private:
presentation cannot read a corpse's age), `Stack::is_valid_for` checks it, and level files
cannot place corpses. Hunger joins the status framework (`world/status.rs`) rather than running
beside it: the player's nutrition burns inside `World::live` (status effects' event loop, called
through `pass_time` by `run_turn`), each change of state and each hit point lost to starving a
moment of its own (`World::until_hungrier`, `World::digest(id, seconds)` in `world/hunger.rs`),
so splitting time never changes the result and the player starves to death at the exact moment;
`World::healing_rate` (the one pace) gives a Starving player none; harm from food (a rotten
corpse, `on_eat` `harm`) is `World::afflict` of the `sickness` status (poisoned: "You feel
sick.", said at once by `sicken`). `run_turn` adds `faint()` to the player's price;
`World::bloated` is inside `burdened`, so every price is slowed while Bloated; `exert(kind)`
after the player's blow, shot, or throw (priced before it); `die_of(cause, when)` kills without
an `Event::Died` (`Death::cause`: `Cause::Starvation`, `Choked`; a death by food poisoning is
the status's, `Death::of`). Weak lowers `Actor::effective_stats(content)` (adding to the status
effects' `penalty`, `Stats::adjusted_by`), which combat profiles, `Actor::defence`,
`burden::load`, and `own_scores` read; never `actor.stats` directly in a rule that hunger
should touch. Eating (`world/eat.rs`): `Action::Eat(Meal::Pack|Floor)` → `Spent::Meal(Dish)` →
`dine` (turns of `eat` through `World::keep_at`, stopping like a rest; the food stays until the
last turn) → `finish` (take one, then rotten / fresh corpse `on_eat` / plain food, `feed` with
overeating, `vomit`, hunger and burden news). Saved and hashed (`Actor::satiation`,
`Stack::died`, `Death::cause`).
`creature_dies` calls `leave_corpse` (corpse on top); `perceive` calls `rot_away` (current
level's piles and the pack). Presentation: `PerceptionView::own_hunger` (the status bar segment
after the HP, priority 1, hidden while Not hungry), `E` (`KeyCommand::Eat`, id `eat`) opens
`ScreenName::Eat` (`ObjectList::Eat`). Long tests that rest or travel for days remove
`data/hunger.toml` (or the player starves).

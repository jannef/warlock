# Hunger

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08h-hunger.md. Keep it current with the code, as CLAUDE.md says.

Hunger (spec §8 "Hunger"; M23, #137): the rules are `warlock-core/src/hunger.rs` (`Hunger`, the
seven states from `Starving` up to `Bloated`, ordered so `<` is hungrier; `news(from, to)`,
`stops_rest`; `Satiation { nutrition, banked, starving }` on `Actor::satiation`, the player's
only, with `burn`/`starve`/`feed`), data `data/hunger.toml` (`content/hunger.rs`, `HungerDef`:
`burn_seconds`, `start`, `max`, `vomit_leaves`, `[states]` least nutrition, `bloated_slow`,
`weak` stats, `faint`, `starve`, `exertion` (only `EXERTIONS`: melee, shoot, throw),
`overeating`, `sickness` (a status id: harm from food is degrees of it), `rotten` (`Rotten`:
what eating rotten food of any kind does, `vomit` and `harm`), optional `corpses` (material,
`Food` per size);
`Content::hunger()`; without the file nobody hungers, food does not rot, and there are no
corpses), races' `hunger` percent, objects' `food = { nutrition, turns, keeps, lasts }` (`Food`;
the shelf life is whole hours or days, `"8h"`, `"30d"`) and `cures` (a remedy: the means it applies with
`World::cure` as it is eaten, `World::remedy`, checked against the statuses' and diseases'
`cured_by` at load; the angelica root cures diseases), creatures' `corpse` (default true) and `on_eat`
(`OnEat`), scenarios' `carry` (a starting pack on top of the character's kit, docs/dev/character-creation.md). Corpses are objects made at load
(`Content::load_hunger`, after the file's objects, id `<creature>_corpse`, `ObjectDef::corpse`
names the creature, `Content::corpse_of`), feeding and keeping by their creature's size, and
level files cannot place corpses. **Rot** (#195): every food stack carries `Stack::died` (when
it was made; a corpse, when its creature died; crate-private: presentation cannot read any
food's age), and `Stack::is_valid_for` checks food knows it and nothing else does. Food that
comes into the world from content is made then (`rot::made_at`: in `instantiate_level` for
piles and creatures' packs, in `World::with_character` for the starting pack, and for quest
gifts); a test stack made with `Stack::new` knows no age and never rots. The stage is pure
(`crate::rot`: `Stage` Fresh/Old/Rotten/Gone, `Food::stage(age)`, `rot::stage`, `gone_at`; all
`None` without hunger.toml) and is shown only in dev dumps (`actors`: food on the level and in
the packs of those on it, with its stage; `--state`: the raw `died`); food never shows its stage
to the player (the owner's decision, #196 for later reveals). Joining stacks keep the oldest
age (`object::join`), so neither packs nor piles split by age. `world/rot.rs` applies it:
`World::rot_pack(id, now)` (pack food gone by `now` leaves, the equipment following, the player
told by a `note`) runs in `World::live` at its start (food that rotted while the actor was
frozen) and as the last thing at each moment, `rot::until_rots` ending a step at the next
rotting, so splitting time never changes the result; `World::rot_away` (from `perceive`) drops
rotted food from the current level's piles. Rotten food is worth nothing in shops
(`World::spoilt` in `world/shop.rs`, `ShopView::spoilt` in perception; `docs/dev/shops.md`).
Rot tests (`world/rot_tests.rs`) burn a point of nutrition a day (`burn_seconds = 86400`) so
rests of weeks do not starve the player. Hunger joins the status framework (`world/status.rs`) rather than running
beside it: the player's nutrition burns inside `World::live` (status effects' event loop, called
through `pass_time` by `run_turn`), each change of state and each hit point lost to starving a
moment of its own (`World::until_hungrier`, `World::digest(id, seconds)` in `world/hunger.rs`),
so splitting time never changes the result and the player starves to death at the exact moment;
`World::healing_rate` (the one pace) gives a Starving player none, and so does
`World::mana_rate` (`docs/dev/mana.md`); a race that `eats_rotten`
(ratfolk) skips everything rotten food does but feed (the top of `World::eat_rotten`); harm from
food (rotten food,
`on_eat` `harm`) is `World::afflict` of the `sickness` status (poisoned: "You feel
sick.", said at once by `sicken`). `run_turn` adds `faint()` to the player's price;
`World::bloated` is inside `burdened`, so every price is slowed while Bloated; `exert(kind)`
after the player's blow, shot, or throw (priced before it); `die_of(cause, when)` kills without
an `Event::Died` (`Death::cause`: `Cause::Starvation`, `Choked`; a death by food poisoning is
the status's, `Death::of`). Weak lowers `Actor::effective_stats(content)` (adding to the status
effects' `penalty`, `Stats::adjusted_by`), which combat profiles, `Actor::defence`,
`burden::load`, and `own_scores` read; never `actor.stats` directly in a rule that hunger
should touch. Eating (`world/eat.rs`): `Action::Eat(Meal::Pack|Floor)` → `Spent::Meal(Dish)` →
`dine` (turns of `eat` through `World::keep_at`, stopping like a rest; the food stays until the
last turn) → `finish` (take one, with its age; then rotten food of any kind (`eat_rotten`:
`[rotten]`, sickness, `expose_to(player, "rotten_food")`) / fresh corpse `on_eat` / plain food,
`feed` with overeating, `vomit`, hunger and burden news). Saved and hashed (`Actor::satiation`,
`Stack::died`, `Death::cause`).
`creature_dies` calls `leave_corpse` (corpse on top); `perceive` calls `rot_away` (the current
level's piles). Presentation: `PerceptionView::own_hunger` (the status bar segment
after the HP, priority 1, hidden while Not hungry), `e` (`KeyCommand::Eat`, id `eat`) opens
`ScreenName::Eat` (`ObjectList::Eat`). Long tests that rest or travel for days remove
`data/hunger.toml` (or the player starves).

# Hunger

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08h-hunger.md. Keep it current with the code, as CLAUDE.md says.

Hunger (spec §8 "Hunger"; M23, #137): the rules are `warlock-core/src/hunger.rs` (`Hunger`, the
seven states from `Starving` up to `Bloated`, ordered so `<` is hungrier; `news(from, to)`,
`worsens`; `Satiation { nutrition, banked, starving }` on `Actor::satiation`, the player's
only, with `burn`/`starve`/`feed`), data `data/hunger.toml` (`content/hunger.rs`, `HungerDef`:
`burn_seconds`, `start`, `max`, `vomit_leaves`, `[states]` least nutrition, `bloated_slow`,
`weak` stats, `faint`, `starve`, `exertion` (only `EXERTIONS`: melee, shoot, throw),
`overeating`, `sickness` (a status id: harm from food is degrees of it), `rotten` (`Rotten`:
what eating rotten food of any kind does, `vomit` and `harm`), optional `corpses` (material,
`Food` per size);
`Content::hunger()`; without the file nobody hungers, food does not rot, and there are no
corpses), races' `hunger` percent (`World::hunger_percent`: then the talents' signed
`hunger_percent` (Lean) less the worn `hunger` affix effect, at least 1), objects' `food = { nutrition, turns, keeps, lasts }` (`Food`;
the shelf life is whole hours or days, `"8h"`, `"30d"`) and `cures` (a remedy: the means it applies with
`World::cure` as it is eaten, `World::remedy`, checked against the statuses' and diseases'
`cured_by` at load; the angelica root cures diseases), creatures' `corpse` (default true) and `on_eat`
(`OnEat`: `message`, `vomit`, `train` and `widen` per stat, `heal`, `harm`, `satiation`; the
old `stat` refused by name), hunger.toml's `[corpses.by_tag]` (`CorpseGifts` per creature tag:
`train` 1 to 200 marks, `widen` 1 to `MAX_TAG_WIDEN` (5) %, a `widen` needing a `train`; a tag
no creature carries refused), resolved into each corpse-leaving creature's `on_eat` by
`Content::resolve_corpse_gifts` (from `load_hunger`; `OnEat::resolve`: per stat the largest of
its tags', `Corpses::gifts_of`, then its own per stat, zeros dropped; `MAX_TRAIN` 200,
`MAX_WIDEN` 20), so `eat.rs` and the wiki's `corpse-effects.md` read one resolved gift (the
undead's files switch their caster, troll, and marsh-thing gifts off with zeros), scenarios' `carry` (a starting pack on top of the character's kit, docs/dev/character-creation.md). Corpses are objects made at load
(`Content::load_hunger`, after the file's objects, id `<creature>_corpse`, `ObjectDef::corpse`
names the creature, `Content::corpse_of`), feeding and keeping by their creature's size, and
level files cannot place corpses. **Rot** (#195): every food stack carries `Stack::died` (when
it was made; a corpse, when its creature died; crate-private: presentation cannot read any
food's age), and `Stack::is_valid_for` checks food knows it and nothing else does. Food that
comes into the world from content is made then (`rot::made_at`: in `instantiate_level` for
piles and creatures' packs, in `World::with_character` for the starting pack, and for quest
gifts, and for foraged food, `world/forage.rs`); a test stack made with `Stack::new` knows no age and never rots. The stage is pure
(`crate::rot`: `Stage` Fresh/Old/Rotten/Gone, `Food::stage(age)`, `rot::stage`, `gone_at`; all
`None` without hunger.toml) and is shown only in dev dumps (`actors`: food on the level and in
the packs of those on it, with its stage; `--state`: the raw `died`); food never shows its stage
to the player by default (the owner's decision; #196 for later reveals), except in words to a trained eye
(#1031): `Effect::Freshness` on Appraise and Survival, summed, a percent chance per hour and food
(`world/freshness.rs::judged`, a pure function of the seed, the player, the food, and the hour, its own stream
`freshness/<object>/<made>/<hour>`, so no gameplay stream moves and looking is read-only), plus 25 points
once rotten, at most 95; `PerceptionView::judge` gives `fresh`/`old`/`rotten` (gone food as rotten) for
the eat list's marks and look mode's pile (visible cells only); eating food so told reports `Use::Judged`
(Appraise, 10 %; `World::train_judged`). Tests: `world/freshness_tests.rs`. Joining stacks keep the oldest
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
should touch. Long fast (`TalentMods::no_hunger_weakness`, #775's rework) is read in those two
places only: `Actor::penalty` leaves Weak's stats out, and `World::faint` returns before its
draw. Eating (`world/eat.rs`): `Action::Eat(Meal::Pack|Floor)` (a potion goes to
`World::drink_potion` instead, before any hunger check: docs/dev/potions.md) → `Spent::Meal(Dish)` →
`dine` (turns of `eat` through `World::keep_at`, stopping like a rest; the food stays until the
last turn) → `finish` (take one, with its age; then rotten food of any kind (`eat_rotten`:
`[rotten]`, sickness, `expose_to(player, "rotten_food")`) / fresh corpse `on_eat` (`eat_fresh`: message, vomit, then `corpse_gifts`: per stat in
display order the `train` marks added to `Actor::stat_marks` (sure, no draw; "Something of its
strength passes into you." at `LOUD_MARKS`, 30, or more) and a gameplay draw per `widen` stat
for `Stats::widen(stat, 1)` ("You feel you could grow stronger yet." when it rose), both scaled
by `corpse_stats_percent`; then `corpse_skills`: per `on_eat.skills` entry (`SkillGift
{ gives, up_to }`, `SkillGift::apply`, no draw, not scaled by talents) an untrained skill is
learned at `gives` and a trained one rises by it up to `up_to`, through `Skills::add`, told
in `[corpses.skills]`'s `learn`/`rise` words (`SkillNews`), nothing past the cap; the loader
(`OnEat::check`: `gives` 1 to `up_to`, `up_to` at most `MAX_VALUE`; `resolve_corpse_gifts`:
`[corpses.skills]` names known skills, a creature only skills with words there) checks it;
then heal, harm, feed; tests in `world/eat_tests.rs`) / plain food,
`feed` with overeating, `vomit`, hunger and burden news). Saved and hashed (`Actor::satiation`,
`Stack::died`, `Death::cause`).
`creature_dies` calls `leave_corpse` (corpse on top); `perceive` calls `rot_away` (the current
level's piles). Presentation: `PerceptionView::own_hunger` (the status bar segment
after the HP, priority 1 for Satiated and Bloated and 0 from Hungry down, so it outlasts even the
level's name, hidden while Not hungry; `ui::hunger_tint` colours the word from Hungry down), `e` (`KeyCommand::Eat`, id `eat`, described "Consume") opens
`ScreenName::Eat` (`ObjectList::Eat`). Long tests that rest or travel for days remove
`data/hunger.toml` (or the player starves).

**Stopping the player** (#972; no rule but one changed, no save state). Every long action is
`World::keep_at`, which stops on any `say`, and `hunger_news` says a worsening: rest, eat, dig,
search, forage, study, scribe, pray, drink, temper, tame, the sanctuary and the familiar's sitting
all stop on it (`hunger_tests.rs`). Chat and ask are a single turn's time, with nothing to stop.
The one rule: `World::starving_stops_rest`, called from the status loop as starving costs a hit
point, says "You are wasting away from hunger." while `World::resting`, so a rest (up to 24 hours,
enough to kill) stops at each hit point; the bounded long actions go on, and a step on the overworld
says nothing, or a journey would log a line per ten minutes. In the game: `Game::apply` sets
`Game::hunger_stop` when `Hunger::worsens(before, now)`; `ui::more` then waits at the last page too
(`more::stops`; `pages`' `last_waits` leaves it room for `--more--`), and `game::apply_queued` drops
the actions queued behind it that frame (as it drops a step behind another that would ask first
from where the first left the player, `input::asks_first`, #1538: the frame's presses are all read
before any is applied, so input asked about it from the cell before) (it takes the `ResMut` so a frame without actions leaves the
`Game` unchanged: `on_level_up` and `refresh` run on change). A replay's last step sets no stop.
`input::hunger_dialog` asks before an overworld step while Fainting or Starving
(`PerceptionView::level_kind`, the state alone); text cases `hunger-*` in `screen_snapshots.rs`.
There is no multi-step movement to end (no run or travel command, on the overworld or anywhere): a
held key's repeats are the only walking.

**A corpse's other uses**: the spells aimed at one (`world/corpse_spell.rs`, `docs/dev/spells.md`
"Spells aimed at a corpse") use up the top-most corpse of a pile the player sees, and read neither
`Stack::died` nor its stage. A cursed room's rising looks for its corpse when it is due
(`world/curse.rs`): one used by a spell does not rise.

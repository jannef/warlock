# Warlock specification — 8. Architecture (continued): hunger

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

- **Hunger** (`warlock-core/src/hunger.rs`, the rules; `world/hunger.rs` and `world/eat.rs`,
  applying them; `data/hunger.toml`, §7; M23, #137), ADOM-like:
  - **Nutrition**: the player has a counter of nutrition points (`Actor::satiation`; creatures
    do not hunger). Game time burns it as the scheduler moves the clock, so long actions,
    rests, and overworld journeys burn it too: a point every `burn_seconds` (60) at the race's
    `hunger` percent (§7; a human 1440 a day, a halfling 1800), the seconds that do not make a
    whole point banked (`Satiation::banked`, saved). The dev time skip burns nothing (nobody
    acts). The player's exertions burn extra points at their race's percent: a blow in melee 2,
    a shot 1, a throw 1 (`[exertion]`; `World::exert`). A run starts at `start` (3600: Not
    hungry, Hungry after 36 hours for a human); the counter never rises above `max` (6500) and
    may fall below 0. The burn is one of the things that happen while the player lives through
    the time (`World::live`, "Status effects" above: `World::digest`), each change of state and
    each hit point lost to starving a moment of its own (`World::until_hungrier`), so it makes no
    difference how the time is cut up.
  - **States** (`Hunger`), by the least nutrition of each (`[states]`): Bloated (5500),
    Satiated (4000), Not hungry (1440), Hungry (720), Weak (240), Fainting (0), and below that
    Starving. **Hungry**: a warning only. **Weak** (and Fainting and Starving): Strength and
    Dexterity −1 (`[weak] stats`) in the effective scores combat and burden read (§8 "Stats";
    hit points go by the actor's own Toughness). **Fainting** (and Starving): as each turn
    begins (`World::run_turn`), one draw of the gameplay stream with `[fainting] chance` (5 %;
    none drawn otherwise): "You faint from hunger." and the player loses `turns` (5) turns, a
    wait's price each added to their action's, while the others act (`World::faint`).
    **Starving**: no hit points come back (`World::healing_rate`, "Healing" above), and
    `[starving] damage` (1) are lost every `seconds` (600) of game time (banked,
    `Satiation::starving`), until the player starves to death at that moment ("You starve to
    death...", `Cause::Starvation`; no creature killed them, so no `Event::Died`). **Bloated**: every price the player pays is raised by `[bloated] slow`
    (×1.25), after the burden ("Costs" above). Satiated and Not hungry do nothing.
  - **Messages** on each change (`Hunger::news`): getting hungrier, "You no longer feel
    bloated.", "You no longer feel satiated.", "You are getting hungry.", "You feel weak from
    hunger.", "You feel faint from hunger.", "You are starving!"; fed, "You are no longer
    starving.", "You feel less faint.", "You feel less weak.", "You are no longer hungry.", "You
    feel satiated.", "You feel bloated." Getting hungrier into Hungry or worse is a message that
    stops a rest (§6 "Rest"); the rest are only logged.
  - **Meals** (`Action::Eat(Meal)`: one of an inventory stack, `Meal::Pack`, or of a stack of
    the pile on the player's cell, `Meal::Floor`; §6 "Eat"): refused, taking no time, without
    `data/hunger.toml` ("You feel no need to eat."), for a stack that is not there ("You carry
    no f.", "There is no c here."), or for what has no `food` ("You cannot eat a dagger."). A
    meal passes the food's `turns` turns through the normal pipeline like a rest (`World::dine`
    through `World::keep_at`; each the `eat` action's price, so speed, burden, and Bloated
    count), and stops early after a turn in which a creature came into view or a message was logged that
    stops a rest (hunger news and fainting included), or the player died: "You stop eating." or
    "A kobold comes into view; you stop eating.", and nothing is eaten. Only when the last turn
    is done is one of the food taken (from where it lay, or the first stack like it there if a
    missile landed on the pile meanwhile; "There is nothing left to eat." if it is gone) and does
    it feed: "You eat the food ration.", then what it does, then the hunger news ("You feel
    satiated.") and the burden news if its weight changed that.
  - **Food** feeds its `nutrition`. A meal that would bring the counter past `max` is
    **overeating**: one draw with `[overeating] choke` (0, off by default: none drawn) to choke
    to death ("You choke to death on the food ration...", `Cause::Choked`), else one with
    `vomit` (50 %) to vomit ("You vomit!": it feeds nothing, and the counter falls to at most
    `vomit_leaves`, 1000, Hungry), else "You have a hard time getting it all down." and what
    does not fit is lost.
  - **Food rots** (`crate::rot`, the stages; `world/rot.rs`, applying them; #195): every food
    has a shelf life in data, `keeps` and `lasts` (§7 "Objects": a loaf of bread keeps 5 days
    and lasts 10, an apple 14 and 28, a food ration 30 and 60, dried meat and the angelica root
    longer; corpses by their creature's size, 8 and 36 hours), and every stack of food knows
    when it was made (`Stack::died`, saved; a corpse's death; food is made as it comes into the
    world: the level it lies on or the creature carrying it as the level is made, the starting
    pack, a quest giver's gift, `rot::made_at`). Its **stage** follows from its age alone
    (`Food::stage`, `rot::stage`): fresh, **old** past half of `keeps`, **rotten** past `keeps`,
    **gone** past `lasts`. Food of one kind joins one stack whatever its age, and the stack keeps
    the oldest (`object.rs`), so neither the pack nor a pile splits by age (which would show it).
    Without `data/hunger.toml` food does not rot.
    - **In a pack** (the player's and creatures'): food rots away at its exact moment as the
      carrier lives through the time (`World::rot_pack`, the last thing at a moment of
      `World::live`, "Status effects" above, which ends a step at it), so however the time is cut
      (a long rest or many waits) it goes at the same moment. The player is told ("Your loaf of
      bread has rotted away.", "Your 3 apples have rotted away."), logged without stopping a rest;
      a creature's goes silently; the equipment follows the letters. Creatures on other levels
      are frozen, so theirs goes as they next live through time (the turn the player arrives).
    - **In piles**: decided from the age alone: food that has rotted away leaves the piles of the
      level the player is on as perception runs (`World::rot_away`, like burnt-out fires), and
      those of another level when the player arrives there.
  - **Corpses**: a creature that dies leaves its corpse (§7 "Objects": the creature's
    `<creature>_corpse` object, weighing the creature's weight, so a heavy one burdens) on top of
    the pile where it fell, after what it carried, whatever killed it (`World::leave_corpse`, in
    `creature_dies`), bloodless creatures too unless their type says `corpse = false`. It rots
    like any food from the moment of the death ("Your giant rat corpse has rotted away.").
  - **Eating rotten food** of any kind (by its age when the last turn of the meal is done; gone
    food not yet removed counts as rotten): "Ugh! The loaf of bread was rotten.", one draw with
    `[rotten] vomit` (50 %) to vomit, else it makes them sick, exposes them to the diseases
    rotten food carries (grave rot, §8 "Diseases": `World::expose_to(player, "rotten_food")`), and
    feeds them; a rotten corpse's creature's `on_eat` does nothing. Old food is eaten as fresh.
    A race that `eats_rotten` (ratfolk, #255) eats it unharmed: "The loaf of bread was rotten,
    but your stomach does not mind.", no draw, no sickness, no exposure; it feeds them, no more.
    **Sickness** is a status effect ("Status effects" above), given through `World::afflict`:
    `[rotten] harm` (1d4) degrees of `sickness` (poisoned), "You feel sick." said at once (when
    it begins); it costs hit points as time passes and may kill (`Death::of`, "Died of poison").
  - **Eating a corpse** feeds by its creature's size (`[corpses.size]`; small 400 in 3 turns,
    medium 1000 in 6). A fresh (or old) one does what
    its creature's `on_eat` says (§7), in this order, each draw from the gameplay stream and none
    for a chance of 0: its `message`; a draw with `vomit` (it comes back up: no food, no more
    effects); per stat in display order, a draw with its chance to raise it by 1 up to its
    potential ("You feel tougher!"; nothing at the potential; raising Toughness raises the most
    hit points); `heal` ("You feel better." when it healed any); `harm` (degrees of sickness, as
    rotten food's); then it feeds its size's nutrition plus `satiation` (at least 0), overeating as food
    does. The effects are never shown in the game before the player tries; the wiki states them
    exactly (the game is vague, the wiki is accurate).
  - **What the player perceives** (`PerceptionView::own_hunger`): their state, never the counter
    or any food's age or stage (the owner's decision: no "(old)" or "(rotten)" in names, look
    mode, or lists, for corpses and all other food alike, so eating stays a gamble; revealing
    freshness by a skill or spell is deferred, #196). What they learn is the message when food in
    their pack rots away, what eating it does, and a shopkeeper refusing rotten food (§7
    "Shops"). The status bar shows the state unless it is Not hungry (`Hungry`, `Weak`,
    `Fainting`, `Starving`, `Satiated`, `Bloated`; after the hit points and before the burden,
    giving way after the burden), and the character screen shows the effective scores. Dev
    dumps: the ASCII header has `hunger hungry` unless Not hungry, JSON dumps `hunger` (the state
    id, or null without hunger), the `actors` layer the player's nutrition and all food on the
    level and in the packs of those on it with when it was made and its stage (`food bread*2
    carried by 0: made ..., 70 hours ago, old`), and `--state` the raw `died` of each stack.
    Not now: cold slowing rot, salting or drying.
  - Showcase: the hub's pantry keeps food rations, loaves of bread, dried meat, and apples; the
    depths, world, and forest scenarios give starting food (§7 "Scenarios"). The thresholds are
    generous, so the showcase is no race against the clock.

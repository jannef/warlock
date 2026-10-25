# Progression: talents and level-up choices

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08i-character.md ("Talents") and the design is `docs/design/progression.md` (§7, §10).
Keep it current with the code, as CLAUDE.md says. Experience itself (the curve, kill XP, the
hit points a level gives) is `docs/dev/experience.md`.

## Modules

| Where | What |
|---|---|
| `content/talent.rs` | `TalentBook` from `data/talents.toml` (settings: `max_ranks`, `[coverage]`, `[[tree]]`, `[[system]]`, each set in one file) and `data/talents/*.toml` (one `[[talent]]` a file, `ContentFiles::table("talents")` order; `TalentId` an index, saves name talents by id so it is **not** in the content fingerprint). `TalentDef` (tree, `ranks: Vec<Rank>`, `needs` for every rank plus each rank's own, `effects` per rank held plus each rank's own, `choice`, `systems`, `granted`), `Needs`/`Need` (the prerequisite kinds and their words: talents, stat scores, skill values, weapon-skill ranks, any skill of a role (`any_weapon_skill`), `chosen`, level, race, class, eyes, `requires_alignment`, an `alignment::Requirement` read against `Actor::band`, and `either`, any one of several, `Need::Either`), `TalentEffect` (the typed effects), `Choice::options`/`name`, and the **coverage check** (`TalentBook::coverage_gap`, run at load: a `ContentError`). |
| `talent.rs` | The runtime: `Talents` (`Vec<Taken { id, rank, choice }>` on `Actor::talents`, saved and hashed, left out of the JSON while empty), `TalentMods` (every effect summed; `Talents::mods(content)` / `Actor::talent_mods`), `Candidate` (what prerequisites read: `Candidate::player`), `unmet`, `may_take` → `Refusal` (in words), `openings`, `offers` (the pick list, tree by tree). |
| `world/talent.rs` | `Due { level, talents }` in `Progress::due` (lowest level first); `World::level_up(from, to)` from `grant_xp` (a pick every `Experience::talent_every` levels, and the per-level hit points of talents held); `refuse_due` (the first line of `player_acts`: everything but `Talent`, `Advance`, `LevelUpAuto` is refused with "You have skill rolls to spend." or "You must choose a talent first."); `take_talent` (the first due with a pick, so a talent may come before the rolls), `level_up_auto` (dev); `settle_due` drops talent picks nothing can be spent on (`offers` empty and no roll left) and rolls when no skill is trained, asked again after each pick. `Due { level, talents, rolls, bonus }`: a level with rolls but no pick is due too; `bonus` are the class's bonus picks (a talent of the class's trees spends one before an ordinary pick; "You must choose a talent of Weaponcraft or Delver."), dropped like the others when nothing fits. |
| `world/creation.rs` | Creation picks (#236; below): `creation_count`, `World::open_creation` (a level 1 `Due` from `begin`, for a run with the kit), `take_creation` (from `start_run`: the spec's picks, else `take_random` on the `character/talents/<spec>` stream; recorded in `CharacterSpec::talents`), `World::draft` / `creation_offers` / `random_creation` (character creation's Talents step), `chosen_with_talents` and `parse_talents` (`--talents`). `World::learn` (`world/talent.rs`) is `take_talent` without its words. |
| `world/class.rs` | Class abilities (#246): `grant` gives the talents a class's level table names up to a level that the player lacks (rank 1, `hand_out` for hit points by level and stat gifts), at creation (`start_run`, level 1), as each level comes (`level_up`, with the news "You gain Fencing."), and on `from_state` (a grown table reaches saves; nothing is taken away); `bonus_trees`; `appraise_carried` (Connoisseur, from `World::live` for the player). |
| `content/class.rs` | `ClassDef::abilities` (`[(level, talent id)]`), `bonus_talents` (`levels`, `trees`; none: any), `abilities_to`, `bonus_at`, `bonus_trees`, `check_table` (after the talents load, from `Content::check_class_tables`, which then runs the coverage check with the class abilities counted). A book without a class tree (fixtures) gives nothing and checks nothing. |
| `world/skill_roll.rs` | The skill rolls (#230, `docs/dev/skills.md`): `level_rolls`, `advance_skill` (`advance:SKILL`), `advance_auto` (`levelup:auto`'s rolls). `World::due()` and `candidate()`. |
| `world/experience.rs` | `grant_xp` scales every gain by `TalentMods::xp_percent` and calls `level_up`; the dev `xp:N` (`grant_xp_dev`). |
| `perception.rs` | `due()`, `dues()` (every level's, for the level-up screen's sequence), `bonus_trees()`, `own_talents()` (`OwnTalent`), `talents()` (`TalentView`: held rank, the next rank's needs with `met`, `openings`): what the level-up screen (`ui/level_up_screen.rs`), the Talents page (`ui/talents_screen.rs`), and the creation step (`ui/chargen_talents_screen.rs`) read (`docs/dev/screens.md`). |
| `dump.rs` | layers `talents` (held, `Hardy 2/3`) and `progress` (dev: dues, `due level 3: 3 skill rolls, 1 talent`, takeable ids with choices, discovered kills, `empathy broken rat`); JSON `talents`, `due` (`level`, `talents`, `rolls`), `forecast` (Weather-wise only); the header's `weather clear then rain`. |
| `action.rs` | `Talent { id, choice }` (`talent:hardy`, `talent:weapon_focus:long_blades`; ids keep their case), `Advance(skill)`, dev `LevelUpAuto` (`levelup:auto`), `GrantXp` (`xp:N`); all `Spent::NoTime` or refused. |
| `rng.rs` | `Rng::levels(seed)`, the `levels` stream, kept in `WorldState::levels_rng: Option<Rng>` **from the first draw** (the stat checks of each level, `world/stat_training.rs`; skill rolls, `world/skill_roll.rs`), so runs that never drew hash as before. |
| `xtask/src/wiki/talents.rs` | `talents.md` (trees with the rules), `talents-by-system.md` (the coverage matrix, class abilities beside the talents); `effect_text`, the rule of one effect, shared with the class tables. |
| `content/class.rs` (#243) | `ClassActive`, the actives (id = the ability's id, aim, cost); `strip_tables` (tests that load talents of their own). |
| `world/class_ability.rs` | `use_class` (`Action::Use`, `use:rage`, `use:smite:E`), `may_use_class`, and the actives but Volley (Inspire, #244, among them), `frenzy` (from `dies`), `unstoppable` (from `hurt`); Hardened's hit points are `level_up`'s. |
| `world/volley.rs` | `use_class_at` (`Action::UseAt`, a class active aimed at a cell: `use:volley:12,5`) and the ranger's Volley (#244). |
| `world/quest.rs` | `tally_kill` (every kill of the player's, by level and type; `Quests::prey`, `killed`): the design's B6. |

## The hooks

`TalentMods` is read where the rules already read what magic items do (`Mods::worn`), one read
each; a new effect is a `TalentEffect` variant, a `TalentMods` field, and its one read:

| Effect | Read in |
|---|---|
| `dodge`, `natural` | `Actor::defence` (Dodge adds to the skills' Dodge; natural armour is added on top of the creature type's own and a magic item's, or stands alone: Thick skin stacks with an amulet of natural armour, by decision) |
| `max_dex` | `Actor::defence`: the burden's cap on Dexterity (only while burdened) |
| `protection` | `Actor::protection`, with the creature type's natural Protection |
| `threat`, `reroll_fumble` | `Actor::stanced` (every attack profile; never below 3), `World::resolve` → `combat::strike_with` |
| `streak` | `World::resolve` → `World::relentless` (`world/gifts.rs`): +1 to hit a blow in the streak, to `max`, from `Actor::streak` (saved and hashed, left out while `None`) |
| `ease`, `poison_spares_scores` | `World::live` (`Effect::until_eases_at`/`advance_at`), `Actor::penalty` (`Effects::penalty_unless`) |
| `disease_checks` | `World::expose` and `disease_turn` |
| `swimmer` | `World::step_price` (wading terrain: `world/stars.rs::wading`), `World::wade` |
| `speed_percent` | `Actor::speed`, read by the scheduler, `price_until`, rest, and wading time |
| `rot_harm_percent` | `eat.rs::eat_rotten` (rounded down; no sickness at 0) |
| `stone_sense`, `search`, `search_reach` | `search.rs::notices`, `search_modifier` (both looks), `search_turn` |
| `kick`, `dig_percent`, `favour_percent` | `kick_door`, `dig_turn`, `world::favour` |
| `melee_damage` | `Actor::melee`, `off_hand_blow` (after the affixes, before the stance; not kicks or bashes) |
| `max_hp_per_levels` | handed out, not read: `take_talent` adds the rank's `hp_at(level)` to `hp_base` and `hp`, `level_up` each later level's growth, as levels' own hit points are (so `Actor::max_hp` needs no level) |
| `healing_percent` | `World::healing_rate`, summed with the regeneration affixes |
| `mana_regen_percent` | `World::mana_rate`, summed with the mana regeneration affixes (Tidal) |
| `footsteps_percent`, `silent_doors` | `noise::noise` (after the stealth skill and affixes; a silent door is no noise at all) |
| `hearing` | `Actor::hearing`, with the listen skill |
| `threshold_percent`, `darkvision`, `sees_uv` | `WorldState::eyes` (`world/eyes.rs`, the design's B4): the base profile (`WorldState::vision`) as the talents change it; `Perception::compute` reads it, so the view's `vision()` is the effective eye (id and name the base's). Creatures keep their types' profiles |
| `scent` | `Perception::compute`: a creature suffering the status, in line of sight within the radius (not on the overworld), has its cell seen as darkvision would |
| `night_sight` | `Perception::compute`: the overworld's sight radius at night (`daylight::phase`) |
| `lantern_light` | `LightCache::sources`: the lit lantern's preset (gameplay light: creatures see it too) |
| `ambush` | `World::resolve` → `World::ambushed` (`world/ambush.rs`): the dice added after the strike when it hit a creature that `World::unaware` says has not seen the attacker (`Mind::last_seen` unset, and creature sight from it misses the attacker's cell); drawn only then |
| `carry_percent` | `burden::load` (the limit before the affixes' kilograms, kept a multiple of 3) |
| `travel_percent` | `World::step_price` on the overworld, after the survival skill |
| `terrain_as` | `World::step_price`: the terrain stepped onto is priced as the one it names (local and overworld alike; its ids checked once the terrain loads, `TalentBook::check_terrain`) |
| `plunge_proof` | `World::fall_in`: no plunge damage and no draw for it |
| `light_on_ice`, `climb_percent` | `World::tread` (the load on the ice is the body's weight alone), `World::tread_price` (the climb's price) |
| `wading_proof` | `World::wade`: no exposure, no draw |
| `beast_friend` | `beast_friend::befriended` (`world/beast_friend.rs`), read by `World::stance`, `ai::nearest_hostile_in_sight`, and `PerceptionView::stance`; `World::resolve` (an attack by the player) records the type in `Progress::empathy_broken` |
| `forecast` | `Perception::compute` → `weather::forecast` (`PerceptionView::forecast`; the status bar, the dump header's `then`, JSON `forecast`) |
| `sun_healing` | `World::healing_rate`, with the regeneration affixes, while `World::under_open_sun` |
| `hunger_percent`, `no_exertion` | `World::hunger_percent` (the race's pace scaled), `exert` |
| `xp_percent` | `grant_xp` (every gain: kills, discovery, quests, dev) |
| `notice` | `search.rs`: `notices` (as if trained in Spot) and `search_modifier(Look::Passive)` |
| `free_blow` | `World::free_blows_of`: `FreeBlows::either` of the actor's own switches and the talents' |
| `weapon` | `Actor::benefit` (`TalentMods::weapon_benefit`): the chosen skill's (`WeaponBonus::skill`, the choice) or a role's benefits; `ranks` raise a held rank (`TalentMods::raised`); `bash` only where the base has a chance. Every profile builder, Defence's parry and shield, `attack_price`, and `reload_price` read benefits, so they need no read of their own |
| `launcher_range`, `throw_range` | `Actor::launcher_range`, `Actor::throw_range` |
| `max_dex`, `deflection` | `Actor::defence` (each worn armour's cap; on top of magic items' Deflection) |
| `protection` | `Actor::protection` through `talent_protection` (`TalentMods::protection_with`: the piece worn with the most Protection counts as body armour; a shield in a hand slot) |
| `stance_to_hit` | `Actor::stanced` (`TalentMods::stance_to_hit`: a penalty only, rounded toward 0) |
| `flee_percent` | `World::step_price` while fleeing, after the stance's `flee_cost` |
| `slip_percent` | `World::slipped` (`world/gifts.rs`), on the player's step price in `travel.rs`: some hostile beside, none beside the cell stepped to; whatever the stance |
| `pain` | `World::hurt` for `Harm::Actor` and `Harm::Trap`, damage over 1 |
| `barbs` | `World::resolve` → `World::barbed` (`world/gifts.rs`) after a melee blow that hurt and left the target alive: `hurt(attacker, barbs, Harm::Barbs)`, no roll, no noise, a death of it earns nothing |
| `resist` | `Actor::resistances`, as gear (`Resistances::sum` caps it); `damage` names the kind (Flame-hardened's fire), else the choice |
| `resting_percent` | `World::healing_rate`, on resting's pace (Fast healer) |
| `second_wind` | `world/second_wind.rs` (`Action::SecondWind`, `Ability::SecondWind` under `Source::Talent`): hit points back, then its `cooldown` status |
| `horns_grace`, `stare` | `world/gifts.rs` (`Action::Gift`, `Ability::Sign(SignActive)` under `Source::Sign`): the birth sign's actives, then their `cooldown` statuses (`TalentMods::sign_cooldown`) |
| `iron_stomach` | `eat.rs`: `eat_rotten` (no vomiting, no draw for it; sickness halved) and `feed` (overeating never vomits) |
| `status_eases` | `World::live`: the sufferer's easing pace, `TalentMods::eases_every` into `Effect::until_eases_every`/`advance_every` |
| `no_penalty`, `penalty_percent` | `Actor::penalty` through `Effects::penalty_for` (`TalentMods::penalty`: lowerings only, toward 0) |
| `clotting` | `World::wounded` |
| `wound_more` | `World::wounded`: a big hit that wounds at all, that many degrees more |
| `disease_checks`, `disease_recovery` | `World::expose`, `World::disease_turn` |
| `kick_die` | `Actor::kick` |
| `door_kick` | `World::kick_door` (added to Strength and athletics) |
| `dig_percent` | `World::dig_turn` |
| `two_handed_price` | `World::attack_price` (melee with a `two_handed` object) |
| `critical_status` | `World::after_blow` (a confirmed critical in melee) |
| `penetration` | `Actor::melee`, `off_hand_blow`, `bash`, `kick` (`TalentMods::penetration_for` the profile's kinds) |
| `burden_dex` | `Actor::defence` (the burden's cap) |
| `sure_step`, `pit_proof` | `World::step_onto_trap` → `steps_lightly` (the player's steps only) |
| `speed_percent` | `World::burdened`, the one place every price passes (`quicker`) |
| `natural_unarmoured` | `Actor::defence` (no body armour: nothing in a `body_armor` slot, `Actor::wears_body_armour`) |
| `weapon` `crit`, `at_rank` | `TalentMods::weapon_benefit` (the held rank passed in) |
| `weapon_marks` | `World::practise` (`TalentMods::marks`; no `role`: every skill, Forge-born) |
| `blaze` | `world/element_gifts.rs` (`Action::Gift`, `Ability::Sign` under `Source::Sign`): the burst, the fires, then its `cooldown` status |
| `two_handed_crit`, `unarmed_die` | `Actor::melee` (and `kick` for the die, the bigger of it and `kick_die`) |
| `stance_dodge` | `Actor::defence`, the stance's Dodge (`TalentMods::stance_dodge`) |
| `step_percent` | `World::step_price` (`TalentMods::step_percent`: body armour, burden, shield) |
| `steadfast` | `burden::load` (`ignores_burdened`) |
| `disease_immune`, `checks` | `World::expose` (refused, nothing rolled), `disease_turn`, `search_modifier(Look::Passive)` (Willpower) |
| `no_penalty_to`, `raise_percent` | `TalentMods::penalty(status, stat, by)` in `Effects::penalty_for` |
| `free_blow` `flurry` | `World::second_blow` on `Cue::First` (`World::flurry`), after the off hand and the bash; two `cleave` sources make `great_cleave` |
| `hardened` | `World::level_up`, as the level comes (a level's hit points again, and `bonus` more, past `hp_level`) |
| `blood_frenzy`, `unstoppable` | `World::frenzy` (a player's kill), `World::unstoppable` (`hurt`, `Harm::Actor`), `World::live` (the numb statuses tick nothing while the rage lasts) |
| `radiance` | `LightCache::sources` (a carried source, `RADIANCE` tag, while its status lasts) |
| `inner_light` | `LightCache::sources` (a carried source, `INNER` tag, always: the player and the creatures see by it alike) |
| the actives (`rage`, `veterans_wind`, `whirlwind`, `lay_on_hands`, `avatar`, `smite`, `quivering_palm`) | `world/class_ability.rs`; `Ability::Class` in `ability.rs` (`class_cooldown`: Rage's status and what it `follows`) |
| `inspire`, `virtuoso` | `world/class_ability.rs` (`World::inspire`: `TalentMods::inspiring`, `World::spend_mana`); `Ability::unavailable` (`Unavailable::NoMana`) |
| `volley` | `world/volley.rs` (`Action::UseAt`, `World::use_class_at`; `ClassActive::aims_cell`, `Aim::Cell`; the UI aims it as a shot, `ui/target.rs`) |
| `favoured_prey` | `World::resolve` → `World::favoured` (`world/ambush.rs`), against `Quests::prey` |
| `ambush` (summed), `ambush_threat` | `World::resolve` → `World::sneaking` (before the strike), `sneak_threat`, `ambushed`; `Dice::and` adds the dice of the same sides |
| `exact_sounds` | `Perception::compute` → `noise::perceive` (`noise::Exact::All`) |
| `sky_footsteps_percent`, `silent_unburdened`, `attack_noise_percent` | `noise::noise` (steps onto an unroofed cell of a sky-exposed level; steps while Unburdened; attacks) |
| `trap_ease` | `search.rs::finding` (searching and in passing; `difficulty_of`, which disarming reads, is unchanged) |
| `pick_lock`, `steal`, `throw_voice` | `Ability::had_by` (and the actions' refusals: `lockpick.rs`, `steal.rs`, `voice.rs` ask it) |
| `trained_skills` | `Actor::skill_effect` (a bonus for every trained skill, as `skill`'s) |
| `missile_dodge` | `World::target_against` (from `resolve` for shots and throws, `trap_strikes` for trap missiles) |
| `spell_damage_percent` | `World::evaded` in `spell_hurts` (bursts and bolts, before resistances) |
| `read_percent`, `scroll_kept` | `read.rs`: `read_price`; `use_up` (one gameplay draw a read, only with the talent; "The writing does not fade from the scroll.") |
| `spares_self` | `World::cast_burst` (the caster is never caught); `PerceptionView::bursts_spare_me`, so `Aim::catches_reader` asks nothing |
| `loot_magic_percent` | `magic::loot::roll_with` (`roll_rarity`: the weights of the rarities above the find's least grade); `mapgen::generate_with` from `level_template` (the player's luck as a level is first made), `World::loot_of` (drops). The same draws, so maps never change |
| `skill_rolls` | `World::level_rolls` |
| `train_skill` | handed out, not read: `take_talent` → `train_chosen` rolls the dice from `levels_rng` into the chosen skill (`Skills::add`: trained at the roll, or raised by it) |
| `ignore_burdened` | `burden::load` (`Load::ignores_burdened`, as a dwarf's) |
| `no_bloating`, `corpse_stats_percent` | `eat.rs`: `feed` (nutrition stops one below Bloated, nothing overeats) and `eat_fresh` |
| `step_slowdown_percent` | `World::step_price` → `slowed` (the burden's ease and `bloated`'s slowdown, steps only) |
| `searching` | `search.rs::search_modifier` (searching only; Keen searcher; `search`, in both looks, is above) |
| `trap_damage_percent` | `World::trap_deals` (before the blood and the hurt; at least 1) |
| `kick_noise_percent` | `noise::noise` (`Act::Kick`) |
| `stat` | handed out, not read: `class::hand_out` raises score and potential (`Stats::gift`) at the pick or the grant; a Toughness gift raises `hp` with the maximum |
| `carry_kg`, `weightless_coins` | `burden::load` (with the affixes' kilograms; coins, item class `coins`, left out of the load) |
| `quest_xp_percent`, `quest_gold_percent` | `World::complete` (`quest.rs`), before `xp_percent` |
| `favour_percent`, `favour_cap` | `world::shop::favour` (before the cap; the cap `MAX_FAVOUR` + `favour_cap`, at most `MOST_FAVOUR`, which `Shops::price`/`offer` clamp to) |
| `sell_percent` | `Shops::offer_more` from `World::deal` and `ShopView::offer` |
| `credit` | `World::on_credit` in `turned_back` and `shop_step`, and `PerceptionView::owed_on_leaving` (no dialog): owing at most that, the player leaves, the debt stays on the bill |
| `identify_trades` | `World::pay` (what was unpaid) and `World::trade` (what is sold, "That was …!") |
| `item_worth`, `gem_worth` | `PerceptionView::appraisal` → `magic::describe::appraised` (the describe screen) |
| `identify_carried` | `World::appraise_carried` from `live`: `Magic::carried` counts the seconds (saved, left out while 0) |
| `gem_find_percent` | `gem::find_with` (the gems' chance scaled, at most 100) through `magic::loot::Luck` (`Luck::of(talents)`: with `loot_magic_percent`, from `level_template` and `World::loot_of`) |
| `gem_cut` | `gem::cut_finer` in `instantiate_level` (loose gems of a level made from then on) and `World::loot_of` (a drop) |
| `skill` | `Actor::skill_effect`, as magic items' skill bonuses |
| `make_way` | `World::refusal`: everyone budges (sizes and overloading still stop a swap) |
| `distrusted` | `World::refusal`: no one budges, `make_way` or not (Trusted by none) |
| `missile_breaks` | `World::loose` (the break chance scaled) |
| `checks` | the stat checks: `disease.rs` (both), `search_modifier`, `lockpick.rs`, `disarm_modifier`, `steps_lightly` |
| `critical_confirm` | `Actor::stanced` → `Profile::confirm`, added to the confirmation roll in `combat::strike` |
| `burden_ease` | `World::burdened` (added to Balance's ease) |
| `disarm` | `Ability::Disarm`'s `had_by` |
| `weapon { skill }`, `protection { per_piece }` | `TalentMods::weapon_benefit` (a fixed skill or style: Fencing's Duelist, Big hands' maces and unarmed); `talent_protection` counts the pieces worn with Protection |

Birth sign gifts, the Roc to the Barghest (#267), and their hooks:

| Effect | Read by |
|---|---|
| `day_sight` | `perception.rs`, the overworld range by day (`night_sight` at night) |
| `exact_footsteps` | `noise::perceive` (footsteps at their own cell, never `rough_place`) |
| `stance_costs` | `World::set_stance`: a wait's price instead of no time |
| `max_hp_percent` | `Actor::max_hp`, `lasting_max_hp` (`Actor::frail`, at least 1) |
| `rise_from_ashes` | `World::hurt` → `World::rise_from_ashes` (`world/stars.rs`): the player at 0 hit points of anything but `Harm::Starving` (choking never passes `hurt`), while `Progress::once` lacks `risen`; sets it, afflicts the status, hit points to the share (of the maximum with it), `Event::Spell` over `burst_cells`, one gameplay roll of the damage for every creature reached (`spell_hurts`), `shock(Heat)` |
| `smell_rot` | `PerceptionView::smell(stack)` (a `Freshness` word from `rot::stage`, never an age), the eat list's mark |
| `free_blow` `gatekeeper` | `World::second_blow` on `Cue::Approaches` (`World::step_up` from `creature_steps`) |

`Progress::once` (saved, left out while empty) holds what may happen once in a life; the
`talents` dump and `OwnTalent::spent` mark Rise from the ashes "(spent)" once it has.

Effects that name a status (`second_wind`'s `cooldown`, `status_eases`, `no_penalty`,
`critical_status`) or a damage type (`penetration`) are checked at load (statuses load before
talents) and resolved by `TalentMods::of(content, ...)`; `TalentEffect` is `Clone`, not `Copy`.
The `any_weapon_skill` need (`Need::AnyWeaponSkill`: `roles` and `of`, resolved to skill ids
with its words at load) is what "a melee skill at 3" asks.

## Class actives

The warrior, barbarian, paladin, and monk (#243) have their tables the way the everyday folk do:
class-tree talents (`data/talents/`, `granted`) named by their class files' `abilities`, so
they are held like any talent, saved with them, and read through `TalentMods`. Their effects are
the talents' kinds and the class's own (the README of `data/talents/` lists them). The seven
actives (`rage`, `veterans_wind`, `whirlwind`, `lay_on_hands`, `smite`, `radiance`,
`quivering_palm`) are effects that `TalentEffect::active` maps to a `ClassActive`;
`Content::class_active` finds the talent that has one, whose name and description the abilities
menu shows and whose `requires_alignment` need the paladin's gifts ask. Shared entries are a
talent each (`divine_health`, `purity_of_body`); the monk's Deflect missiles is the ordinary
talent, handed out. Keys: the actives have none of their own (as Second wind; the abilities menu
and its repeat key use them).

## Coverage policy

`data/talents.toml` holds the closed `[[system]]` list and `[coverage]` (`min_talents = 2`,
`min_class = 0` until every class has its table, `free_per_tree = 1`); class abilities are
counted by `Content::check_class_tables`. A talent must name systems from
the list; a system or tree marked `pending = true` is counted and shown in the wiki but not yet
held to the bar. **A wave that brings a system's talents removes its `pending`** (and a tree's
once it has a talent without prerequisites); `min_class` becomes 1 with the class tables. A new
system of the game joins the list in the PR that ships its talents. The check runs at load, so
`Content::builtin()` (and the game) fail on a gap; `content::talent::tests` covers the gaps. Talents of a `class = true` tree are class abilities:
not counted as talents, nor asked for a free one; the class abilities a system has are the
talents the class tables give that name it (each once). `min_class` stays 0 until every class
has its table (I1–I3).

**Every new system ships with talents** (the owner, 2026-10-20; design §8.2), so no system is
left with nothing that cares about it:

1. **The issue** says which (`.github/ISSUE_TEMPLATE/feature.md`, "Talents and class
   abilities"): at least `min_talents` talents and their trees, at least `min_class` class
   abilities, the system tag; or why none.
2. **The PR** that adds the system adds, in the same PR: its `[[system]]` entry in
   `data/talents.toml` (placed by topic beside its neighbours, never at the end; no `pending`),
   its talents under `data/talents/` with the tag in their `systems`, the `TalentEffect`
   variants and their one read each (above), and the wiki rows (the by-system table updates by
   itself). Adding a tag without its talents fails the load; adding a system without a tag
   passes, so the reviewer checks the template's answer.
3. **What counts as a system**: a rule a talent could change (a check, a price, a pace, a
   limit, a new action). Not one: presentation state no rule reads (blood stains), and channels
   of existing systems (stat training is what the level-up reads, not a row). Extending a
   system already in the list (a new trap kind) needs no new tag, only its talents' `systems`
   kept honest.
4. **Resources** (mana, the mana PR; the pools of #225: Rage, Focus, Openings, Ki,
   Inspiration) are systems like any other: each joins the list with talents on its own hooks
   (its maximum, its refill or decay, what it costs), and every ability that spends it carries
   its tag. A cast that costs mana or a pool has no cooldown status besides (time refills it,
   "time is the penalty"); a talent that changes a cost changes the data's cost, never adds a
   timer. Mana and Resources are already named in the design's matrix (§8.1) with their first
   talents (`deep_well`, … for mana; `battle_trance`, … for #225) and join the list when those
   PRs land.
5. **The bar is data** (`[coverage]`): raise `min_talents` or `min_class` as the list grows,
   in its own PR, with the talents that meet it.

## Creation picks

A run that starts with the character's kit (`WorldState::kit`: New run, `--character`,
`--name`, `--talents`) takes `[levels] creation_talents` (2, `data/experience.toml`) plus the
class's `creation_talents` (the bard's 1, `data/classes/`). `World::begin` (the old body of
`start_run`) opens them as a `Due { level: 1, talents }`; `start_run` then spends it with
`take_creation`: `CharacterSpec::talents` in order (each `World::learn`: an unknown, refused,
missing, or extra pick is `WorldError::Character`), else `take_random` (the stream
`character/talents/<race>,<gender>,<class>`: an index into `talent::offers` then into the talent's
ways, until none is due or none is offered), writes what was taken into the character's spec,
recomputes perception, and says nothing. Not the kit (the scenario's own character in tests,
walks, `--scenario` alone): no picks, so those replays keep their hashes. The humans' third pick
(design §7.2) is H5's Versatile: it should add to `creation_count`.

`CharacterSpec::talents` is left out of the JSON while empty (old saves and replays load and
start as before); `Display` stays `race,gender,class`, so the character's other streams are
unchanged, and `same_choice` compares without the picks (`default_name`).

The creation step reads a drafted run (`World::draft`: `begin` with the kit, the picks left
due), cloned per question: `creation_offers(picks)` takes `picks` on a clone and returns what is
left and what may come next; `random_creation` runs `take_random` on a clone. Tests:
`world/creation_tests.rs`.

## Determinism and saves

Picks are replayed actions; prerequisites read stored scores. Talents draw from the gameplay
stream only where their condition arises (Light step's check, Ambusher's die on a hit at an
unaware creature), and some draw less (Swimmer's plunge, Marsh walker's wading, Iron stomach),
so a run without them draws as before. The hash covers `Actor::talents`, `Progress::due`,
`Progress::empathy_broken`, and `levels_rng`, each left out of the JSON while unused, so runs
without talents hash as before this change. Saves change (`Actor::talents`, `Progress::due`,
`Progress::empathy_broken`, `levels_rng`, all with serde defaults; the new light preset shifts the
light ids after it). A run without talents content (fixtures) never has a pick due.

## Tests

`world/class_active_tests.rs` (the adventurers' tables: abilities by level, bonus picks, every
passive's rule, the actives and their cooldowns, strayed paladins, replays);
`content/talent.rs` (loading, validation, the coverage gaps, choice chains), `talent.rs`
(summing, ranks per choice, serialization), `world/talent_arms_tests.rs` (the Weaponcraft and Guard hooks, the role need's words), `world/damage_talent_tests.rs` (Elemental ward and Crusher, the damage-types bar), `world/land_talent_tests.rs` (the Shadow and Wayfarer hooks), `world/talent_tests.rs` (dues and refusals,
prerequisites in words, ranks and Hardy's growth, several levels at once and `levelup:auto`,
dropped picks, every hook, a free-blow talent, saves and replays, perception and dumps).

## Class tables

The everyday folk (#246: aristocrat, commoner, craftsperson, merchant) have their tables: nine
abilities each at 1, 6, 12, 18, 25, 32, 40, 50, 60 (34 class-tree talents, plus Riposte and
Second wind, which their tables hand out), and bonus picks at 10, 20, 30, 45, 55. The
warrior, barbarian, paladin, and monk (I1) have theirs, in `world/class_active_tests.rs`; the
ranger, thief, and bard (I2, #244) theirs, in `world/class_rogue_tests.rs` (the kill tally and
Favoured prey, every passive, Volley and Inspire, the abilities the thief and bard get
untrained); the casters come with I3. The I2 tables hand out two ordinary talents (Evasion,
Lore), so a creation pick or a bonus pick may already have taken one; the table then gives
nothing in its place (as the monk's Deflect missiles). The showcase's and the arena's own
characters are thieves, the forest's and the world's rangers: their runs start with Sneak attack
or Favoured prey. `world/class_tests.rs` covers the grants, bonus picks,
restores, replays, and every hook; `content::class::tests` the table checks.

## The trees so far

Vigour, Might, and Finesse (#237) ship whole but for Battle trance and Inner calm (the class
resources, #225); `body_talent_tests.rs` covers each one's rule, `hunger_tests.rs` Iron stomach.
The statuses Winded (Second wind's cooldown) and Staggered (`cost = 150`, 6 s) are in
`data/status.toml`.

Scholar, Grit, and Delver (#240) ship whole but for the five mana talents (Deep well, Clear
mind, Meditation, Frugal caster, Blood magic: the mana PR, H6); `talent_wits_tests.rs` covers
each rule, `quest_tests.rs` Well liked. Apprentice is the first talent with a `skill` choice (any
skill, trained or not; `talent:apprentice:stealth`, three ranks per skill), Treasure hunter the
first with an `either` prerequisite (`Need::Either`, "Intelligence 12 or Appraise 20"). The
`talk` and `gems` systems stay pending until the racial talents (Diplomat, Gem lore) land.

The damage-type talents (#248), Elemental ward (H2) and Crusher (H1), are the `damage_types`
system's two, so it is no longer `pending`; `world/damage_talent_tests.rs` covers the ward
against damage and status doses, its words on the character screen, Crusher by weapon kind, and
the bar. The witch's Blight (#245) needs no read of its own for its type: its burst names
`damage_type = "poison"`, and its Poisoned goes through `World::afflict`, which meets poison
resistance.

Shadow and Wayfarer (#239) ship whole but for Hooded light (it waits for the lantern's hood,
#235: the talent narrows who has the hood ability) and Patient blade (the thief's openings,
#225); `land_talent_tests.rs` covers each one's rule. `data/lights.toml` has the Lamplighter's
`lantern_bright`; `data/factions.toml` marks the vermin and the wildlife `animal`. Blood scent's
radius is 9 (the design's "4 cells" read as four beyond darkvision's five, which it needs: at 4 it
would see nothing darkvision does not). The design's B5 (Defence by attack kind) is
`World::target_against`, which came with Deflect missiles (#237).

## Later waves

Skill rolls (#230) add `Due::rolls` and draw from `levels_rng`; stat training (#231, done:
`docs/dev/stats.md` "Training") checks every stat on it once a level, from `level_up`; the level-up screen, the Talents page, creation picks, and `--talents` (#236, done) read
`PerceptionView::due`/`talents`; the talent waves add files under `data/talents/` and one read
each; `granted` talents (races' `versatile`, the birth signs') are computed into `Actor::talents`
by whoever grants them and never offered (`may_take` refuses them). The birth signs' are granted
by `world/stars.rs` at the start and on every restore (`Talents::regrant`; #265,
`docs/dev/character-creation.md`), in the first tree, Stars (`no_free`).

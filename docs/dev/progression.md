# Progression: talents and level-up choices

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08i-character.md ("Talents") and the design is `docs/design/progression.md` (§7, §10).
Keep it current with the code, as CLAUDE.md says. Experience itself (the curve, kill XP, the
hit points a level gives) is `docs/dev/experience.md`.

## Modules

| Where | What |
|---|---|
| `content/talent.rs` | `TalentBook` from `data/talents.toml` (settings: `max_ranks`, `[coverage]`, `[[tree]]`, `[[system]]`, each set in one file) and `data/talents/*.toml` (one `[[talent]]` a file, `ContentFiles::table("talents")` order; `TalentId` an index, saves name talents by id so it is **not** in the content fingerprint). `TalentDef` (tree, `ranks: Vec<Rank>`, `needs` for every rank plus each rank's own, `effects` per rank held plus each rank's own, `choice`, `systems`, `granted`), `Needs`/`Need` (the prerequisite kinds and their words: talents, stat scores, skill values, weapon-skill ranks, any skill of a role (`any_weapon_skill`), `chosen`, level, race, class, eyes (never an alignment band: alignment bars nothing, `docs/dev/alignment.md`), and `either`, any one of several, `Need::Either`), `TalentEffect` (the typed effects), `Choice::options`/`name` (`school`: a spell school or a non-physical damage type, #400; `stat`: one of the six, Gifted, #726), and the **coverage check** (`TalentBook::coverage_gap`, run at load: a `ContentError`). |
| `talent.rs` | The runtime: `Talents` (`Vec<Taken { id, rank, choice }>` on `Actor::talents`, saved and hashed, left out of the JSON while empty), `TalentMods` (every effect summed; `Talents::mods(content)` / `Actor::talent_mods`, a `Cow`: borrowed and shared for an actor without talents, which most creatures are, so the many reads a turn copy nothing), `Candidate` (what prerequisites read: `Candidate::player`), `unmet`, `may_take` → `Refusal` (in words), `openings`, `offers` (the pick list, tree by tree). |
| `world/talent.rs` | `Due { level, talents }` in `Progress::due` (lowest level first); `World::level_up(from, to)` from `grant_xp` (a pick every `Experience::talent_every` levels, and the per-level hit points of talents held); `refuse_due` (the first line of `player_acts`: everything but `Talent`, `Advance`, `LevelUpAuto` is refused with "You have skill rolls to spend." or "You must choose a talent first."); `take_talent` (the first due with a pick, so a talent may come before the rolls), `level_up_auto` (dev); `settle_due` drops talent picks nothing can be spent on (`offers` empty and no roll left) and rolls when no skill is trained, asked again after each pick. `Due { level, talents, rolls, bonus }`: a level with rolls but no pick is due too; `bonus` are the class's bonus picks (a talent of the class's trees spends one before an ordinary pick; "You must choose a talent of Weaponcraft or Delver."), dropped like the others when nothing fits. |
| `world/creation.rs` | Creation picks (#236; below): `creation_count`, `World::open_creation` (a level 1 `Due` from `begin`, for a run with the kit), `take_creation` (from `start_run`: the spec's picks, else `take_random` on the `character/talents/<spec>` stream; recorded in `CharacterSpec::talents`), `World::draft` / `creation_offers` / `random_creation` (character creation's Talents step), `chosen_with_talents` and `parse_talents` (`--talents`). `World::learn` (`world/talent.rs`) is `take_talent` without its words. |
| `world/class.rs` | Class abilities (#246): `grant` gives the talents a class's level table names up to a level that the player lacks (rank 1, `hand_out` for hit points by level and stat gifts), at creation (`start_run`, level 1), as each level comes (`level_up`, with the news "You gain Fencing."), and on `from_state` (a grown table reaches saves; nothing is taken away); `bonus_trees`; `appraise_carried` (Connoisseur, from `World::live` for the player). |
| `content/class.rs` | `ClassDef::abilities` (`[(level, talent id)]`), `bonus_talents` (`levels`, `trees`; none: any), `abilities_to`, `bonus_at`, `bonus_trees`, `check_table` (after the talents load, from `Content::check_class_tables`, which then runs the coverage check with the class abilities counted). A book without a class tree (fixtures) gives nothing and checks nothing. |
| `world/skill_roll.rs` | The skill rolls (#230, `docs/dev/skills.md`): `roll_count`/`skill_roll` (the count's parts and a roll's dice, shared with the level-up screen, #543), `level_rolls`, `advance_skill` (`advance:SKILL`), `advance_auto` (`levelup:auto`'s rolls). `World::due()` and `candidate()`. |
| `world/experience.rs` | `grant_xp` scales every gain by `TalentMods::xp_percent` and calls `level_up`; the dev `xp:N` (`grant_xp_dev`). |
| `perception.rs` | `due()`, `dues()` (every level's, for the level-up screen's sequence), `roll_count()`, `skill_roll(skill)` (the skill-roll step's numbers, #543), `bonus_trees()`, `own_talents()` (`OwnTalent`), `talents()` (`TalentView`: held rank, the next rank's needs with `met`, `openings`): what the level-up screen (`ui/level_up_screen.rs`), the Talents page (`ui/talents_screen.rs`), and the creation step (`ui/chargen_talents_screen.rs`) read (`docs/dev/screens.md`). |
| `dump.rs` | layers `talents` (held, `Hardy 2/3`) and `progress` (dev: dues, `due level 3: 3 skill rolls, 1 talent`, `rolls a level 3 (class 2, race 1, intelligence 0, talents 0)`, `roll dice athletics 1d3, balance +2`, takeable ids with choices, discovered kills, `empathy broken rat`); JSON `talents`, `due` (`level`, `talents`, `rolls`), `forecast` (Weather-wise only); the header's `weather clear then rain`. |
| `action.rs` | `Talent { id, choice }` (`talent:hardy`, `talent:weapon_focus:long_blades`; ids keep their case), `Advance(skill)`, dev `LevelUpAuto` (`levelup:auto`), `GrantXp` (`xp:N`); all `Spent::NoTime` or refused. |
| `rng.rs` | `Rng::levels(seed)`, the `levels` stream, kept in `WorldState::levels_rng: Option<Rng>` **from the first draw** (the stat checks of each level, `world/stat_training.rs`; skill rolls, `world/skill_roll.rs`), so runs that never drew hash as before. |
| `xtask/src/wiki/talents.rs` | `talents.md` (trees with the rules), `talents-by-system.md` (the coverage matrix, class abilities beside the talents); `effect_text`, the rule of one effect, shared with the class tables. |
| `content/class.rs` (#243) | `ClassActive`, the actives (id = the ability's id, aim, cost); `strip_tables` (tests that load talents of their own). |
| `world/caster.rs` | `cast_spell` (`Action::Cast`, `use:blessing`, `use:arcane_bolt:12,5`), `Actor::spell_cost`, the strike on the scrolls' shared code, `reprieve` (Miracle, Hag's bargain, from `hurt`), `lore_blow` (Poison lore, from `resolve`). |
| `world/racial.rs` | The racial talents (#241): `grants` (a race's `talents`, `data/races.toml`, put in by `stars::grant` before the sign's gifts, through `Talents::regrant`, which now replaces every granted talent outside a class tree), `World::small_target` (Small target's Dodge, from `resolve` after `target_against`, when the attacker's `size_of` is larger). `content/race.rs::check_talents` (at load): a race grants only `granted` talents of the `racial` tree. |
| `world/class_ability.rs` | `use_class` (`Action::Use`, `use:rage`, `use:smite:E`), `may_use_class`, and the actives but Volley and Rage (Inspire, #244, among them); Hardened's hit points are `level_up`'s. |
| `world/rage.rs` | The barbarian's Rage on the Rage resource (R1, #249): `World::rage` (`grant_resource_with_status`), `World::unstoppable` (from `hurt`). |
| `world/volley.rs` | `use_class_at` (`Action::UseAt`, a class active aimed at a cell: `use:volley:12,5`) and the ranger's Volley (#244), on 6 Focus (#250). |
| `world/focus.rs` | The ranger's Focus (R2, #250): `hold` (what a talent's `grants_resource` grants, from `class::hand_out` and `World::grant_class`), `World::rapid_shot` (from `shoot.rs::fire`). `docs/dev/resources.md`. |
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
| `mana_regen_percent` | `World::mana_rate`, summed with the mana regeneration affixes (Tidal, Clear mind) |
| `max_mana_per_levels` | handed out, not read: `class::hand_out` adds the rank's `mana_at(level)` to `mana_base` and `mana`, `level_up` each later level's growth (Deep well; past `mana_level` too) |
| `mana_resting_percent` | `World::mana_rate`, on resting's pace (Meditation) |
| `mana_cost_percent` | `World::spend_mana` through `Actor::mana_cost` (`TalentMods::mana_cost`: rounded to the nearest, at least 1; Frugal caster) |
| `blood_magic` | `world/mana.rs::blood_magic` (`Action::UseTalent`, `use:blood_magic`, `Ability::BloodMagic` under `Source::Talent`): hit points for mana, no cooldown |
| `footsteps_percent`, `silent_doors` | `noise::noise` (after the stealth skill and affixes; a silent door is no noise at all) |
| `hearing` | `Actor::hearing`, with the listen skill |
| `threshold_percent`, `darkvision`, `sees_uv` | `WorldState::eyes` (`world/eyes.rs`, the design's B4): the base profile (`WorldState::vision`) as the talents change it; `Perception::compute` reads it, so the view's `vision()` is the effective eye (id and name the base's). Creatures keep their types' profiles |
| `scent` | `Perception::compute`: a creature suffering the status, in line of sight within the radius (not on the overworld), has its cell seen as darkvision would |
| `night_sight` | `Perception::compute`: the overworld's sight radius at night (`daylight::phase`) |
| `lantern_light` | `WorldState::carried_light` (`world/lamp.rs`, read by `LightCache::sources`): the preset of a lit (not hooded) lantern ready in the Tool slot (gameplay light: creatures see it too) |
| `ambush` | `World::sneak_dice` (`world/ambush.rs`): added to the Sneak attack skill's dice; `World::resolve` → `World::ambushed` rolls them after the strike when it hit a target that `World::unaware` (`world/surprise.rs`) says has not seen the attacker, or one that is Surprised; drawn only then |
| `initiative_bonus` | `World::startle` (`world/surprise.rs`): added to the initiative check (Wary) |
| `surprise_ease` | `World::status_cost` (`world/surprise.rs`): the percent of Surprised's extra cost spared, with Initiative's capped at 100 (Cool head) |
| `surprise_difficulty` | `World::startle`'s `craft` (`world/surprise.rs`): added to the difficulty of the check of those the holder catches unawares (the thief's Sneak attack) |
| `sneak_reach` | `World::sneaks` (`world/ambush.rs`): cells beyond `missile_reach` a missile or attack spell still sneak-attacks (Long reach) |
| `sees_unaware` | `Perception::compute` (`perception.rs`): `SeenActor::unaware` from `surprise::creature_unaware` for each creature in view, false for all without it; read by look mode ("unaware of you") and `ui/glyphs.rs` (the unaware mark) (Stalker's eye) |
| `startled_blow` | `World::startle` → `World::hair_trigger` (`world/free_blow.rs`): a check made against a hostile attacker beside the holder is answered by `Cue::Startled`'s free blow before the attack is rolled; `resolve` (and `cast.rs::spell_attack`) stop when it fells the attacker (Hair trigger) |
| `vitals_in` | `World::has_vitals_for` → `Content::has_vitals` (`world/ambush.rs`, `content/surprise.rs`): types with those tags have vitals for the holder's attacks, criticals and sneak dice (Anatomist) |
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
| `resist` | `Actor::resistances`, as gear (`Resistances::sum` caps it); `damage` names the kind (Flame-hardened's and Kindled's fire, Of the deep's cold), else the choice |
| `resting_percent` | `World::healing_rate`, on resting's pace (Fast healer) |
| `second_wind` | `world/second_wind.rs` (`Action::UseTalent`, `use:second_wind`, `Ability::SecondWind` under `Source::Talent`): hit points back, then its `cooldown` status |
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
| `flurry_cost` | `World::flurry`: the second blow spends its cost (Ki) first, else none (`TalentMods::flurry_cost`; `docs/dev/resources.md`) |
| `hardened` | `World::level_up`, as the level comes (a level's hit points again, and `bonus` more, past `hp_level`) |
| `resource_build` (Blood frenzy, Patient blade), `resource_max` (Mighty rage, Inner calm), `resource_start` (Battle trance), `resource_rate` (Battle trance, Inner calm, Steady hand, Patient blade) | `World::gain_resource` (beside the resource's own `builds`), `Actor::resource_max`, `Actor::resource_start` (`World::grant_resource`, `focus::hold`), `World::resource_rate` |
| `geared_resource_percent` (Bellows: stamina, 100) | `Actor::resource_max`'s worn term for that pool (`TalentMods::geared_resource`, rounded up; a term below 0 as it is) |
| `unstoppable` | `World::unstoppable` (`hurt`, `Harm::Actor`: `cost` of the rage via `spend_resource`), `World::live` (the numb statuses tick nothing while Raging lasts) |
| `radiance` | `LightCache::sources` (a carried source, `RADIANCE` tag, while its status lasts) |
| `inner_light` | `LightCache::sources` (a carried source, `INNER` tag, always: the player and the creatures see by it alike) |
| the actives (`rage`, `veterans_wind`, `whirlwind`, `lay_on_hands`, `avatar`, `smite`, `quivering_palm`) | `world/class_ability.rs` (Rage: `world/rage.rs`; `rage { resource }`, `TalentMods::rage_status`); `Ability::Class` in `ability.rs` (`class_cooldown`: Rage's status and what it `follows`) |
| `spell` (`{ kind = "spell", spell = "ID" }`, checked against `data/spells/`) | `TalentMods::spells` → `Ability::Spell`'s `had_by`; `world/cast.rs` |
| `weapon` on a `choice = "school_skill"` talent (Schooled) or `role = "school"` (Magister); `school_marks_percent` (Devotee) | `TalentMods::weapon_benefit` through `Actor::benefit` (`Actor::school_benefit`); `TalentMods::marks(Role::School, _)` in `World::practise` (docs/dev/magic-skills.md) |
| `spell_mod` (`SpellModSpec`: `rungs`, `scroll_rungs`, `wand_rungs`, `reach_rungs`, `degree_rungs`, `sides`, `dice`, `bonus`, `reroll`, `rider`, `cells`, `radius_cells`, `degrees`, `cost`, `when`; a talent chosen for a school or an element keys on the choice) | `TalentMods::spell` → `Actor::spell_mods` (`world/spell_mods.rs`), read by `world/spell_rung.rs` and the cast path (the spell scaling design §7.1: Empower, Archmage, Keen mind, School focus, Scroll adept, Wand mastery, Sure hand, Brimming, Numbing cold); `cost` in `Actor::spell_base` |
| `spell_mana_percent` | `Actor::spell_cost` (the cast and `Unavailable::NoMana`) |
| `caster_level` (`per_levels`: Arcane training) | `Actor::caster_level` (`TalentMods::caster_level_at`, at the character level) |
| `attuned_percent` (Attuned) | `Actor::caster_level`'s items term (`TalentMods::attuned`: the worn `caster_level` this percent more, rounded up; a term below 0 as it is) |
| `study` | `World::start_study` (time), `study_odds` (bonus; `overreach_percent`: Overreach's third off the overreach the check counts) |
| `deflection_unarmoured` | `Actor::defence` (Deflection, no body armour: Mage armour) |
| `purify`, `corpse_harmless`, `meal_heals` | `eat.rs`: `eat_rotten` (no sickness, no exposure, no draw), `eat_fresh` (no `on_eat` harm or vomiting, no draw), `meal_heals` from `finish` (an object, or a fresh corpse) |
| `disease_still` | `World::disease_turn`: a failed check does nothing and counts no progression |
| `status_damage` | `World::resolve` → `World::lore_blow` (a hit at a creature suffering the status) |
| `reprieve` | `World::hurt` → `World::reprieve` (after Rise from the ashes; `Progress::once`) |
| `inspire`, `virtuoso` | `world/class_ability.rs` (`World::inspire`: `TalentMods::inspiring`, its `instead` resource via `World::spend_resource`, else `World::spend_mana`); `Ability::unavailable` (`Unavailable::NoMana` when the resource falls short too) |
| `volley` | `world/volley.rs` (`Action::UseAt`, `World::use_class_at`; `ClassActive::aims_cell`, `Aim::Cell`; the UI aims it as a shot, `ui/target.rs`) |
| `favoured_prey` | `World::resolve` → `World::favoured` (`world/ambush.rs`), against `Quests::prey` |
| `ambush` (summed), `ambush_threat` | `World::resolve` → `World::sneaks` (before the strike), `sneak_threat`, `ambushed`; `Dice::and` adds the dice of the same sides |
| `exact_sounds` | `Perception::compute` → `noise::perceive` (`noise::Exact::All`) |
| `sky_footsteps_percent`, `silent_unburdened`, `attack_noise_percent` | `noise::noise` (steps onto an unroofed cell of a sky-exposed level; steps while Unburdened; attacks) |
| `trap_ease` | `search.rs::finding` (searching and in passing; `difficulty_of`, which disarming reads, is unchanged) |
| `pick_lock`, `steal`, `throw_voice` | `Ability::had_by` (and the actions' refusals: `lockpick.rs`, `steal.rs`, `voice.rs` ask it); no talent carries `pick_lock`, `steal`, or `trap_ease` since #684 took the thief's Trapfinding, Pick lock, and Light fingers (the kinds stay) |
| `lurk` | `world/thief.rs` (`World::lurk`, `use_class`; `thief_may` refuses "Too late to lurk."); the status switches the gatekeeper's free blow on in `World::free_blows_of`; `World::step_up` → `lie_in_wait` flags the blow (`Technique::lurk`), `resolve` → `lurked` makes its target unaware unless `Mind::last_seen` is set |
| `low_blow` | `world/thief.rs` (`World::low_blow`, `use_class`; `thief_may` refuses "There is no one there to blind."); `Technique::sneak`, taken in `resolve` by `low_blow_taken`, passes `sneaks` as unaware (no `startle`, no flat-footed) |
| `sneak_kill_status` | `World::resolve` → `World::vanishing_act` (`world/thief.rs`), beside `Build::SneakKill` |
| `trained_skills` | `Actor::skill_effect` (a bonus for every trained skill, as `skill`'s) |
| `missile_dodge` | `World::target_against` (from `resolve` for shots and throws, `trap_strikes` for trap missiles) |
| `spell_damage_percent` | `World::evaded` in `spell_hurts` (bursts and bolts, before resistances) |
| `read_percent`, `scroll_kept` | `read.rs`: `read_price`; `use_up` (one gameplay draw a read, only with the talent; "The writing does not fade from the scroll.") |
| `spares_self` | `World::cast_burst` (the caster is never caught); `PerceptionView::bursts_spare_me`, so `Aim::catches_reader` asks nothing |
| `loot_magic_percent` | `magic::loot::roll_with` (`roll_rarity`: the weights of the rarities above the find's least grade); `mapgen::generate_with` from `level_template` (the player's luck as a level is first made), `World::loot_of` (drops). The same draws, so maps never change |
| `skill_rolls` | `World::level_rolls` |
| `extra_pick` | `creation_count` (level 1: the race's granted talents' picks, from data alone) and `World::level_up` (`TalentMods::extra_picks_at`, added to the level's talent pick) |
| `ferocity` | `Actor::stanced` (every attack profile), while `hp × 3 < max_hp` |
| `stone_find` | `search.rs`: `World::stone_find(at)` added to the modifier of both looks for a hidden feature on a `stone` terrain cell |
| `disease_interval_percent` | `World::disease_turn`: `Infection::left` set to the interval scaled (`TalentMods::scaled`, at least 1 s) |
| `stout` | `burden::load` → `Load::stout`; `Load::penalty` gives Burdened while Strained (left out of the JSON while false) |
| `small_target` | `World::small_target` (`world/racial.rs`) |
| `launcher_range { skill }` | `Actor::launcher_range` through `TalentMods::launcher_range_for` (the launcher's `skill` id) |
| `train_skill` | handed out, not read: `take_talent` → `train_chosen` rolls the dice from `levels_rng` into the chosen skill (`Skills::add`: trained at the roll, or raised by it) |
| `ignore_burdened` | `burden::load` (`Load::ignores_burdened`, as a dwarf's) |
| `no_bloating`, `corpse_stats_percent` | `eat.rs`: `feed` (nutrition stops one below Bloated, nothing overeats) and `corpse_gifts` (a corpse's `train` marks and `widen` chances scaled, the chance at most 100) |
| `step_slowdown_percent` | `World::step_price` → `slowed` (the burden's ease and `bloated`'s slowdown, steps only) |
| `searching` | `search.rs::search_modifier` (searching only; Keen searcher; `search`, in both looks, is above) |
| `trap_damage_percent` | `World::trap_deals` (before the blood and the hurt; at least 1) |
| `kick_noise_percent` | `noise::noise` (`Act::Kick`) |
| `stat` | handed out, not read: `class::hand_out` raises score and potential (`Stats::gift`) at the pick or the grant; a Toughness gift raises `hp` with the maximum |
| `carry_kg`, `weightless_coins` | `burden::load` (with the affixes' kilograms; coins, item class `coins`, left out of the load) |
| `quest_xp_percent`, `quest_gold_percent` | `World::complete` (`quest.rs`), before `xp_percent` |
| `favour_percent`, `favour_cap` | `world::shop::favour` (before the cap; the cap `MAX_FAVOUR` + `favour_cap`, at most `MOST_FAVOUR`, which `Shops::price`/`offer` clamp to) |
| `storyteller` | `world::shop::favour` (before the cap, with `favour_percent`): `Storyteller::favour(rare, fateful)`, counted from the tiers of the story's lines (`background::tiers`, passed in by `World::player_favour` and `PerceptionView::shop_at`; `&[]` for anyone else) |
| `story_luck` | `World::tell_background`: the witness's `luck` is 100 + it, which `background::tell` scales `[tiers]`' luckier odds by (`Tiers::of`) |
| `hard_won` | `World::story_stat` (the story's `stat` arm): each point a line took (after the floor) is that many `stat_marks` on the stat |
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
| `critical_penetration` | `Actor::stanced` → `Profile::critical_penetration`, added to the penetration of a confirmed critical in `combat::strike` |
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

Sign-limited picks (#270; `world/sign_picks.rs`), and their hooks:

| Effect | Read by |
|---|---|
| `streak { damage_every }` | `World::relentless` (`gifts.rs`): +1 to `Profile::bonus` per that many blows before it, as far as the streak counts |
| `starving_heals` | `World::healing_rate`: Starving no longer stops healing |
| `horn_of_plenty { per_level, degrees }` | `World::horns_grace` via `World::horn_of_plenty`: hit points back (never past the maximum, and cause enough alone), the cooldown's degrees |
| `petrifying_gaze { severity, degrees }` | `World::transfixing_stare`: the status's and the cooldown's degrees |
| `fire_breath` | `World::fire_breath` (`SignActive::FireBreath`) |
| `hounds_bay` | `World::hounds_bay` (`SignActive::HoundsBay`); the status's `flees` read by `World::frightened` in `creature_turn` |
| `wildfire { radius, damage }` | `World::blaze` (`element_gifts.rs`): radius and damage instead |
| `second_rising` | `World::rise_from_ashes` (`stars.rs`): `risen_twice` after `risen` |
| `off_hand_damage` | `Actor::off_hand_blow` |
| `best_skill_threat` | `Actor::benefit` for a skill `Actor::knows_best` (a melee, unarmed, launcher, or thrown skill held at the top rank) |
| `eats_rotten` | `eat.rs::eat_rotten`: as a race that `eats_rotten` |
| `wades_free` | `World::step_price`, beside `swimmer` (no disease immunity) |

The rest (Hare-footed, Roc's grip, Dragon's hoard, Riddle of the Sphinx, Deep roots' Dodge,
Bedrock, Undertow's plunge) are existing kinds.

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
menu shows; a holy gift's talent may name the bands it works for (`alignment`; outside them it
is used and does nothing, `docs/dev/alignment.md`). Shared entries are a
talent each (`divine_health`, `purity_of_body`); the monk's Deflect missiles is the ordinary
talent, handed out. A save holding a talent the content no longer has (the thief's three removed
by #684) loads without it: `Talents::drop_unknown` in `World::from_state`, each said in
`World::retiered` (logged on load). Keys: the actives have none of their own (as Second wind; the abilities menu
and its repeat key use them).

## Caster spells

The priest, druid, wizard, and witch (#245) have their tables as the others do; their actives
are **spells** (`SpellId`, `Ability::Spell`, `Action::Cast` by the spell's id). The placeholders
of #245 were migrated onto the spell system by #395, and #397 gave each caster its guaranteed
list (a cantrip at 1, about four in all; the bard, paladin, and ranger one or two boons): a class
ability's talent effect `{ kind = "spell", spell = "ID" }` gives the spell (`TalentMods::spells`
says who has it; an id is given by one talent, which several tables may share), and the spell
itself is `data/spells/ID.toml`, cast by the one cast path
(docs/dev/spells.md: its tier, cost, reach, dice, resolution; the migrated tiers and costs are
listed there). They cost mana (`Actor::spell_cost`, `World::spend_mana`) and never a cooldown;
`Unavailable::NoMana` greys them out. A refusal (no cell, out of range, no hostile seen to
hex, no sky, nothing to cure, not hurt) spends neither time nor mana. Shared entries: the
priest names the paladin's `smite` (so it asks Good) and `divine_health`, the druid the
ranger's `woodland_stride` (#244 writes the same file), the wizard the Scholar `spell_recall`.
The design's priest "Shield of faith" is `ward_of_faith` (Deflection +1): the id is the
paladin's. New statuses: Blessed, Hexed (`ill_wished`), Weakened, Sluggish (`sluggish`, cost
150), Worn thin (the reprieves'); new effects `arcane_bolt` and `blight` in `data/effects.toml`.
Tests: `world/caster_tests.rs`.

## Coverage policy

`data/talents.toml` holds the closed `[[system]]` list and `[coverage]` (`min_talents = 1`
since #773, `min_class = 0` until every class has its table, `free_per_tree = 1`); class
abilities are counted by `Content::check_class_tables`. A `[[system]]` may still set its own
`min_talents` in place of the bar's (`SystemDef::min_talents`); none does since the bar is one.
`spells` (#394, docs/dev/spells.md) joined pending, with the
`caster_level` talent effect and nothing taking it; #396 added the Spellcraft tree with its
first talent, Quick study (`TalentEffect::Study`), and #400 the rest (Overreach, School focus
with the `school` choice: a school or an element), the wizard's Arcane training, Empower and
Archmage reworked (they also lower costs; their dice are held to the cap, a scroll's too), and
took `spells` off `pending` (`world/spell_talent_tests.rs`). Wand mastery comes with wands
(#398), Channeler with rods (#399): their hooks do not exist yet. A talent must name systems from
the list; a system or tree marked `pending = true` is counted and shown in the wiki but not yet
held to the bar. **A wave that brings a system's talents removes its `pending`** (and a tree's
once it has a talent without prerequisites); `min_class` becomes 1 with the class tables. A new
system of the game joins the list in the PR that ships its talents. The check runs at load, so
`Content::builtin()` (and the game) fail on a gap; `content::talent::tests` covers the gaps. Talents of a `class = true` tree are class abilities:
not counted as talents, nor asked for a free one; the class abilities a system has are the
talents the class tables give that name it (each once). `min_class` stays 0 until every class
has its table (I1–I3).

**Every new system ships with a talent** (the owner, 2026-10-20; design §8.2), so no system is
left with nothing that cares about it. The bar was two talents a system until #773: that
produced talent creep (every room effect, monster gimmick, chest variant, or info readout minted
a talent or two of its own, most of them filler; the 2026-11-01 audit found about 40, and #774
cut them). The owner: "Modify the talent rule to prevent this type of talent creep." So:

- **Systems are broad.** A `[[system]]` names a system of the game (combat, stealth, magic,
  light, hunger, ...), not a sub-feature. A single special room effect, a monster family's
  gimmick, a chest kind, or an info readout is never a system of its own: it extends its parent
  system, whose talents may cover it. #773 folded the narrow tags into their parents: `mimics`
  into `chests`, `prayer` into `special_rooms`, `bounties` and `services` into `talk`, `drain`
  into `diseases`, and `animated` dropped (its talents were cut; animated objects are creatures,
  their remains `creature_gear`).
- **The bar is one talent.** `min_talents = 1`, and a feature issue may answer "none: it extends
  X, whose talents already touch it".
- **A talent must earn its place.** It is a real choice at its tier: it changes how a build plays
  or what it can do. It may not be
  - information only (a readout, a worth shown, a direction told);
  - a small percentage on a niche mechanic;
  - an effect another talent, a class ability, or a skill rank already gives (extend that one or
    add a rank to it instead; #774 kept one of each duplicate group);
  - something that only matters in a rare room or against one rare creature.
- **Tags match hooks.** A talent names a system in its `systems` only if its effect reads that
  system's rules. A general bonus is tagged by the system it is for, not by every rule that
  happens to read it (Iron will's +2 to Willpower checks is `diseases`, not also `searching` for
  the passive search that rolls Willpower); the code an effect borrows is no system (a birth
  sign's Blaze uses the scrolls' spell code but is `combat`, not `scrolls`). There is no load
  check tying effect kinds to tags (about 150 kinds, many reading several systems): the reviewer
  checks it against the effect's one read in the hook table above.

The steps:

1. **The issue** says which (`.github/ISSUE_TEMPLATE/feature.md`, "Talents and class
   abilities"): at least `min_talents` talents and their trees, each meeting the bar above, at
   least `min_class` class abilities, the system tag; or why none.
2. **The PR** that adds the system adds, in the same PR: its `[[system]]` entry in
   `data/talents.toml` (placed by topic beside its neighbours, never at the end; no `pending`),
   its talents under `data/talents/` with the tag in their `systems`, the `TalentEffect`
   variants and their one read each (above), and the wiki rows (the by-system table updates by
   itself). Adding a tag without its talents fails the load; adding a system without a tag
   passes, so the reviewer checks the template's answer.
3. **What counts as a system**: a broad rule set a talent could change (a check, a price, a
   pace, a limit, a new action). Not one: presentation state no rule reads (blood stains),
   channels of existing systems, and sub-features of one (a room effect, a creature family, a
   chest kind). (Stat training was such a channel, what the level-up reads, until its
   daily check made it a rule a talent can change: `stat_training` since #562, with Hard
   training and Quick study of the body.) Extending a
   system already in the list (a new trap kind, a new room effect) needs no new tag and no new
   talent, only its talents' `systems` kept honest.
4. **Resources** (mana, the mana PR; the pools of #225: Rage, Focus, Openings, Ki,
   Inspiration) are systems like any other: each joins the list with talents on its own hooks
   (its maximum, its refill or decay, what it costs), and every ability that spends it carries
   its tag. A cast that costs mana or a pool has no cooldown status besides (time refills it,
   "time is the penalty"); a talent that changes a cost changes the data's cost, never adds a
   timer. Mana and Resources are already named in the design's matrix (§8.1) with their first
   talents (`deep_well`, … for mana; `battle_trance`, … for #225) and join the list when those
   PRs land.
5. **The bar is data** (`[coverage]`): `min_class` rises to 1 with the class tables, in its
   own PR. `min_talents` stays at one: a count is never a reason to add a talent.

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
(design §7.2) is Versatile's (#241): `creation_count` adds the `extra_pick` at level 1 of the
race's granted talents, so a human picks 3 and a human bard 4.

`CharacterSpec::talents` is left out of the JSON while empty (old saves and replays load and
start as before); `Display` stays `race,gender,class`, so the character's other streams are
unchanged, and `same_choice` compares without the picks (`default_name`).

**Heritage** (#697): a talent with `needs = { creation = true }` (`Need::Creation`, "to be a
new character"; the wiki says "only at character creation") is met only at character level 1,
so only the creation picks offer it (the creation step, `--talents`, Random) and a level's pick
refuses it in words. Family wealth and Father's sword (tree `heritage`, `no_free`) are the
first; their effects (`starting_gold`, `starting_gems`, `heirloom`) are hand-outs, not
`TalentMods`: `World::hand_down` (`world/heirloom.rs`, one line in `learn`) gives them once, as
the rank is taken, each from its own stream (`character/family-wealth/<spec>`,
`character/fathers-sword/<spec>`), so the kit and every other roll stay as they were. The
heirloom is `loot::heirloom` (2 affixes: magic, prefix and suffix; more: rare, split and named
as a rare item's), identified, and readied in its own slot only if that is free. Its weapons
are checked once objects load (`talent::check_heirlooms`). `kit::tests::
nobody_starts_strained_and_only_wealth_burdens` adds the heritage at its heaviest: never
Strained, and Burdened only for the weakest of a small race. Tests: `world/heirloom_tests.rs`.
The background story (docs/dev/background.md) reads the talents held after the creation picks:
its `only` lines force Family wealth's and Father's sword's chapters, so nothing is paid twice.
Favoured child and Hard-won (#705, the `background` system) are heritage talents too, and
`TalentMods` (`story_luck`, `hard_won`), read once, as the story is told; the system's third,
Storyteller (Scholar, Presence 12), is an ordinary pick that reads the story in play (the hooks
above; `world/background_talent_tests.rs`).

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
passive's rule, the actives and their cooldowns, the holy gifts unanswered outside their bands, replays);
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
Favoured prey, every passive, Volley and Inspire, the abilities the thief's skills give, Lurk,
Low blow, and Vanishing act (#684), and a save holding a talent the content dropped); the priest, druid, wizard, and witch (I3, #245) theirs, in `world/caster_tests.rs`. The I2 tables hand out two ordinary talents (Evasion,
Lore), so a creation pick or a bonus pick may already have taken one; the table then gives
nothing in its place (as the monk's Deflect missiles). The showcase's and the arena's own
characters are thieves, the forest's and the world's rangers: their runs start with Sneak attack
or Favoured prey. `world/class_tests.rs` covers the grants, bonus picks,
restores, replays, and every hook; `content::class::tests` the table checks.

## The trees so far

Vigour, Might, and Finesse (#237) ship whole; Battle trance and Inner calm (the class
resources) came with R5 (#253); `body_talent_tests.rs` covers each one's rule, `hunger_tests.rs` Iron stomach.
The statuses Winded (Second wind's cooldown) and Staggered (`cost = 150`, until the next action:
`per_action`, docs/dev/weapon-abilities.md) are in `data/status.toml`.

Scholar, Grit, and Delver (#240) ship whole; their five mana talents (Deep well, Clear mind,
Meditation, Frugal caster, Blood magic) came with #242 (H6; `docs/dev/mana.md`,
`world/mana_talent_tests.rs`), which took the `mana` system's `pending` off. `talent_wits_tests.rs` covers
each rule, `quest_tests.rs` Well liked. Apprentice is the first talent with a `skill` choice (any
skill, trained or not; `talent:apprentice:stealth`, three ranks per skill), Treasure hunter the
first with an `either` prerequisite (`Need::Either`, "Intelligence 12 or Appraise 20").

The racial talents (#241) are the `racial` tree (`no_free`: each asks for its race): Elven
accuracy, Ageless, Dual heritage, Diplomat, Stone sense, Dwarven resilience, Stout, Lucky,
Slinger, Small target, Tinkerer, Gem lore, Ferocity, Orcish hide, and the humans' granted
Versatile. The gnome's disarming talent is `tinkerer` ("Tinkerer"), since `tinker` is the
craftsperson's class ability of the same effect. With Diplomat and Gem lore the `talk` and `gems`
systems are held to the bar. Granting Versatile changes every human's talents, so human runs
hash differently from before. The dump's `talents` layer marks it `(race)`, the Talents page
"race", and the character screen's birth-sign list leaves it out (`OwnTalent::race`).
`racial_tests.rs` covers each rule, the prerequisites, the picks, and a replay.

The `resources` system (#225, `docs/dev/resources.md`) is in the list: its four talents (R5,
#253) are Battle trance (Might, barbarian 6: rage starts 6 higher, decays at 67 %), Inner calm
(Finesse, monk 18: ki +2 at most, back at 150 %), Steady hand (Weaponcraft, ranger 12: focus back
at 150 %), and Patient blade (Shadow, thief 32: openings decay at 50 %, and a sneak attack that
kills builds one more, the `sneak_kill` event); the only class-limited picks
(`needs = { class, level }`). `world/resource_talent_tests.rs` covers each rule and the
prerequisites. A talent of a resource the class has not been granted yet (Inner calm before
Flurry grants ki, R4) holds the change until it is. Wild guard (Might, barbarian 6; #680) halves
the berserker stance's Dodge penalty (`stance_dodge`), the stance a barbarian's rage holds them
in (`world/berserker.rs`); `world/berserker_tests.rs` covers it.

The damage-type talents (#248), Elemental ward (H2) and Crusher (H1), are the `damage_types`
system's two, so it is no longer `pending`; `world/damage_talent_tests.rs` covers the ward
against damage and status doses, its words on the character screen, Crusher by weapon kind, and
the bar. The witch's Blight (#245) needs no read of its own for its type: its burst names
`damage_type = "poison"`, and its Poisoned goes through `World::afflict`, which meets poison
resistance.

Shadow and Wayfarer (#239) ship whole but for Hooded light (it waited for the lantern's hood,
#235; the hood is now a lantern's mode, `u`, #643, so the talent needs a new rule) and Patient blade (the thief's openings,
#225); `land_talent_tests.rs` covers each one's rule. `data/lights.toml` has the Lamplighter's
`lantern_bright`; `data/factions.toml` marks the vermin and the wildlife `animal`. Blood scent's
radius is 9 (the design's "4 cells" read as four beyond darkvision's five, which it needs: at 4 it
would see nothing darkvision does not). The design's B5 (Defence by attack kind) is
`World::target_against`, which came with Deflect missiles (#237).

## Later waves

Skill rolls (#230) add `Due::rolls` and draw from `levels_rng`; stat training (#231, done:
`docs/dev/stats.md` "Training") checks every stat on it once a level, from `level_up` (the
daily check at 06:00, #562, draws the training stream instead); the level-up screen, the Talents page, creation picks, and `--talents` (#236, done) read
`PerceptionView::due`/`talents`; the talent waves add files under `data/talents/` and one read
each; `granted` talents (races' `versatile`, the birth signs') are computed into `Actor::talents`
by whoever grants them and never offered (`may_take` refuses them); a race's and the sign's
together, by `stars::grant`. The birth signs' are granted
by `world/stars.rs` at the start and on every restore (`Talents::regrant`; #265,
`docs/dev/character-creation.md`), in the first tree, Stars (`no_free`).

**`Actor::boons`** holds the talents given for life outside every pick, at rank 1 with what
they hand out (`World::bestow`, `world/boons.rs`): a forgotten altar's Chosen and Forsaken, and
a background story's `talent` and `flaw` lines (#707, `world/background.rs`;
`docs/dev/background.md`). `stars::grant` puts them back on every restore beside the race's and
the sign's grants. `OwnTalent::story` (a boon a told line names, `background::gave`) marks the
story's "(story)" / "(story, flaw)"; `OwnTalent::boon` is the altar's (a boon the story did not
give). A class table naming a held boon later gives nothing in its place (`class::grant`
skips what is held).

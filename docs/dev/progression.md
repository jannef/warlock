# Progression: talents and level-up choices

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08i-character.md ("Talents") and the design is `docs/design/progression.md` (§7, §10).
Keep it current with the code, as CLAUDE.md says. Experience itself (the curve, kill XP, the
hit points a level gives) is `docs/dev/experience.md`.

## Modules

| Where | What |
|---|---|
| `content/talent.rs` | `TalentBook` from `data/talents.toml` (settings: `[[tree]]`, `[[system]]`) and `data/talents/*.toml` (one `[[talent]]` a file, `ContentFiles::table("talents")` order; `TalentId` an index, saves name talents by id so it is **not** in the content fingerprint). `TalentDef` (tree, `needs`, `effects`, `choice`, `systems`, `granted`), `Needs`/`Need` (the prerequisite kinds and their words: talents, stat scores, skill values, weapon-skill ranks, any skill of a role (`any_weapon_skill`), `chosen`, level, race, class, eyes (never an alignment band: alignment bars nothing, `docs/dev/alignment.md`), and `either`, any one of several, `Need::Either`), `TalentEffect` (the typed effects), `Choice::options`/`name` (`school`: a spell school or a non-physical damage type, #400). The load checks only that the table is well formed ("Loading" below). |
| `talent.rs` | The runtime: `Talents` (`Vec<Taken { id, choice }>` on `Actor::talents`, `holds(id, choice)`, `has(id)`, saved and hashed, left out of the JSON while empty), `TalentMods` (every effect summed; `Talents::mods(content, effects)`, the effects under a status counted while the holder suffers it, / `Actor::talent_mods`, a `Cow`: borrowed and shared for an actor without talents, which most creatures are, so the many reads a turn copy nothing), `Candidate` (what prerequisites read: `Candidate::player`), `unmet`, `may_take` → `Refusal` (in words; `Held`, "You have Hardy already."), `openings`, `offers` (the pick list, tree by tree). |
| `world/talent.rs` | `Due { level, talents }` in `Progress::due` (lowest level first); `World::level_up(from, to)` from `grant_xp` (a pick every `Experience::talent_every` levels, and the per-level hit points of talents held); `refuse_due` (the first line of `player_acts`: everything but `Talent`, `Advance`, `LevelUpAuto` is refused with "You have skill rolls to spend." or "You must choose a talent first."); `take_talent` (the first due with a pick, so a talent may come before the rolls), `level_up_auto` (dev); `settle_due` drops talent picks nothing can be spent on (`offers` empty and no roll left) and rolls when no skill is trained, asked again after each pick. `Due { level, talents, rolls, bonus }`: a level with rolls but no pick is due too; `bonus` are the class's bonus picks (a talent of the class's trees spends one before an ordinary pick; "You must choose a talent of Weaponcraft or Delver."), dropped like the others when nothing fits. |
| `world/creation.rs` | Creation picks (#236; below): `creation_count`, `World::open_creation` (a level 1 `Due` from `begin`, for a run with the kit), `take_creation` (from `start_run`: the spec's picks, else `take_random` on the `character/talents/<spec>` stream; recorded in `CharacterSpec::talents`), `World::draft` / `creation_offers` / `random_creation` (character creation's Talents step), `chosen_with_talents` and `parse_talents` (`--talents`). `World::learn` (`world/talent.rs`) is `take_talent` without its words. |
| `world/class.rs` | Class abilities (#246): `grant` gives the talents a class's level table names up to a level that the player lacks (`hand_out` for hit points by level, stat gifts, and `weapon_ranks`), at creation (`start_run`, level 1), as each level comes (`level_up`, with the news "You gain Fencing."), and on `from_state` (a grown table reaches saves; nothing is taken away); `bonus_trees`; `appraise_carried` (Lore, Arcane lore, Old lore, from `World::live` for the player). |
| `content/class.rs` | `ClassDef::abilities` (`[(level, talent id)]`), `bonus_talents` (`levels`, `trees`; none: any), `abilities_to`, `bonus_at`, `bonus_trees`, `check_table` (after the talents load, from `Content::check_class_tables`: the talents and trees named resolve). Rows are read by level, lowest first (`table_rows` sorts them; `abilities_to` takes them while their level is reached). A book without a class tree (fixtures) gives nothing and checks nothing. |
| `world/skill_roll.rs` | The skill rolls (#230, `docs/dev/skills.md`): `roll_count`/`skill_roll` (the count's parts and a roll's dice, shared with the level-up screen, #543), `level_rolls`, `advance_skill` (`advance:SKILL`), `advance_auto` (`levelup:auto`'s rolls), `roll_skill` (the one roll both use), `mark_skill` (every mark added; the automatic roll at `[rolls] auto_at`, #1328, outside any `Due`). `World::due()` and `candidate()`. |
| `world/experience.rs` | `grant_xp` scales every gain by `TalentMods::xp_percent` and calls `level_up`; the dev `xp:N` (`grant_xp_dev`). |
| `perception/view.rs`, `perception/truth.rs` | `due()`, `dues()` (every level's, for the level-up screen's sequence), `roll_count()`, `skill_roll(skill)` (the skill-roll step's numbers, #543), `bonus_trees()`, `own_talents()` (`OwnTalent`), `talents()` (`TalentView`: `held`, the needs with `met`, `openings`): what the level-up screen (`ui/level_up_screen.rs`), the Talents page (`ui/talents_screen.rs`), and the creation step (`ui/chargen_talents_screen.rs`) read (`docs/dev/screens.md`). |
| `dump.rs` | layers `talents` (held, `Skill focus (stealth)`) and `progress` (dev: dues, `due level 3: 3 skill rolls, 1 talent`, `rolls a level 3 (class 2, race 1, intelligence 0, talents 0)`, `roll dice athletics 1d3, balance +2`, takeable ids with choices, discovered kills, `empathy broken rat`); JSON `talents`, `due` (`level`, `talents`, `rolls`) |
| `action.rs` | `Talent { id, choice }` (`talent:hardy`, `talent:weapon_focus:long_blades`; ids keep their case), `Advance(skill)`, dev `LevelUpAuto` (`levelup:auto`), `GrantXp` (`xp:N`); all `Spent::NoTime` or refused. |
| `rng.rs` | `Rng::levels(seed)`, the `levels` stream, kept in `WorldState::levels_rng: Option<Rng>` **from the first draw** (the stat checks of each level, `world/stat_training.rs`; skill rolls, `world/skill_roll.rs`), so runs that never drew hash as before. |
| `xtask/src/wiki/talents.rs` | `talents.md` (trees with the rules), `talents-by-system.md` (each system's talents, class abilities beside them); `effect_text`, the rule of one effect, shared with the class tables. |
| `content/class.rs` (#243) | `ClassActive`, the actives (id = the ability's id, aim, cost); `strip_tables` (tests that load talents of their own). |
| `world/caster.rs` | `cast_spell` (`Action::Cast`, `use:blessing`, `use:arcane_bolt:12,5`), `Actor::spell_cost`, the strike on the scrolls' shared code, `reprieve` (Hag's bargain, from `hurt`), `lore_blow` (Poison lore, Malison; from `resolve` and `spell_strikes`). |
| `world/racial.rs` | The racial talents (#241): `grants` (a race's `talents`, `data/races.toml`, put in by `stars::grant` before the sign's gifts, through `Talents::regrant`, which now replaces every granted talent outside a class tree), `World::small_target` (Small target's Dodge, from `resolve` after `target_against`, when the attacker's `size_of` is larger). `content/race.rs::check_talents` (at load): a race grants only `granted` talents of the `racial` tree. |
| `world/class_ability.rs` | `use_class` (`Action::Use`, `use:rage`, `use:smite:E`), `may_use_class`, and the actives but Volley and Rage (the songs are `world/song.rs`, #870; Smite is `world/smite.rs`, #910); Hardened's hit points are `level_up`'s. |
| `world/rage.rs` | The berserker's Rage on the Rage resource (R1, #249): `World::rage` (`grant_resource_with_status`), `World::unstoppable` (from `hurt`). |
| `world/volley.rs` | `use_class_at` (`Action::UseAt`, a class active aimed at a cell: `use:volley:12,5`) and the ranger's Volley (#244), on 6 Focus (#250). |
| `world/charge.rs` | The warrior's Charge (#1466, `use:charge:12,5`): the way to a hostile the player sees checked whole before a step (`charge_path`: walkable, no deep water, ice, known trap, or seen creature; at least a cell; no way out of a shop's door owing, `World::owing_exit`, the test `turned_back` makes, #1488), then the steps without `step_away`/`step_up` (no free blows provoked), each priced by `step_price`, an unknown trap or someone unseen stopping it short, then `player_attacks`, free; 1 stamina (the talent's `resource`) + `per_cell` a cell, checked and spent by the charge itself (`Content::charge_cost` for the menu row). Whirling charge (`whirling_charge`, the warrior's at 50, #1467; `TalentMods::whirling_charge`): `World::whirl` after each step strikes every hostile beside the new cell but the target, each once a charge (`strike_free` with `Free::Cleave`: no time, no exertion, the mark untouched). Tests: `world/charge_tests.rs`. |
| `world/spirit_weapon.rs` | `use_class_shape` (`Action::Shape`, a class active in a weapon's shape: `use:spirit_weapon:long_sword`) and the Hunter's Spirit weapon (#881): `shapes` (every `melee_weapons` object with an attack that a hand holds, by weapon skill then id), `World::call_weapon` (a `Stack::spirit` into the pack, wielded through `World::equip_saying`, one held faded first; priced by the swap: a wield plus what the hands held), `gifts` (read in `Actor::melee`/`off_hand_blow`), `World::fade_spirit` and `release_spirit` (the invariant, `docs/dev/equipment.md`), `gift` (the content's numbers and phrase, for the description). |
| `world/scribe.rs` | `start_scribe` (`Action::Scribe`, `scribe:fireball`, its own action: a class active on a spell known) and the wizard's Scribe (#906): `Actor::scribe_base`/`scribe_cost` (`spell_base` × `mana_times`, then `mana_cost`), `scribe_turns` (a study's, no Quick study), `World::scribe` through `keep_at` (`Spent::Scribe(Writing)`), `finish_scribe` (the mana through `spend_mana`, one blank used up, `Content::scroll_of` made and `learn_scroll`, the `scribe` stat use); `docs/dev/scrolls.md` "Writing scrolls". |
| `world/merchants_eye.rs` | `use_class_on` (`Action::UseOn`, a class active on a stack carried: `use:merchants_eye:c`) and the merchant's Merchant's eye (#816): `World::identify` on the stack for the time of a `read`; no mana, cooldown, check, or limit; refused, no time, for a stack not carried or already known. |
| `world/focus.rs` | The ranger's Focus (R2, #250): `hold` (what a talent's `grants_resource` grants, from `class::hand_out` and `World::grant_class`), `World::rapid_shot` (from `shoot.rs::fire`). `docs/dev/resources.md`. |
| `world/quest.rs` | `tally_kill` (every kill of the player's, by level and type; `Quests::prey`, `killed`): the design's B6. |

## The hooks

`TalentMods` is read where the rules already read what magic items do (`Mods::worn`), one read
each; a new effect is a `TalentEffect` variant, a `TalentMods` field, and its one read.

**`while = "STATUS"`** (#843, the Berserker design §6.1): any effect may carry it in data, a
status id resolved at load (`content/talent.rs::split_effects`; an unknown one is refused). Such
an effect is kept apart from the plain ones (`TalentDef::effects_while`) and only `Talents::mods(content, effects)` reads it: it
adds the effect while `effects` (the holder's statuses; `Actor::talent_mods` passes
`&self.effects`) has the status, so no rule below knows the condition exists. `TalentDef::effects` and
every reader of a talent's effects other than the sum see only the plain ones, so a kind they
read is refused a condition (`TalentEffect::may_be_conditional`: actives, Second wind, Blood
magic, `spell`, `spell_mod`, `weapon_stamina`, what is rolled or handed down when taken,
`terrain_as`, and the gifts' `scent` and `night_sight`). Battle-scarred (`protection 2`), Headlong
(`step_percent −20`), and Blood frenzy (`attack_price_percent −20`) use it with `raging`; the
wiki's effect text appends "while raging".

| Effect | Read in |
|---|---|
| `dodge`, `natural` | `Actor::defence` (Dodge adds to the skills' Dodge; natural armour is added on top of the creature type's own and a magic item's, or stands alone: Thick skin stacks with an amulet of natural armour, by decision) |
| `max_dex` | `Actor::defence`: the burden's cap on Dexterity (only while burdened) |
| `protection` | `Actor::protection`, with the creature type's natural Protection |
| `threat`, `reroll_fumble` | `Actor::stanced` (every attack profile; never below 3), `World::resolve` → `combat::strike_with` |
| `streak` | `World::resolve` → `World::relentless` (`world/gifts.rs`): +1 to hit a blow in the streak, to `max`, from `Actor::streak` (saved and hashed, left out while `None`) |
| `ease`, `poison_spares_scores` | `World::live` (`Effect::until_eases`/`until_eases_every`), `Actor::penalty` (`Effects::penalty_for`) |
| `disease_checks` | `World::expose` and `disease_turn` |
| `swimmer` | `World::step_price` (wading terrain: `world/stars.rs::wading`), `World::wade` |
| `speed_percent` | `Actor::speed`, read by the scheduler, `price_until`, rest, and wading time |
| `rot_harm_percent` | `eat.rs::eat_rotten` (rounded down; no sickness at 0) |
| `stone_sense`, `search`, `search_reach` | `search.rs::notices`, `search_modifier` (both looks), `search_turn` |
| `kick`, `dig_percent`, `favour_percent` | `kick_door`, `dig_turn`, `world::favour` |
| `melee_damage` | `Actor::melee`, `off_hand_blow` (after the affixes, before the stance; not kicks or bashes) |
| `max_hp_per_levels` | handed out, not read: `take_talent` adds its `hp_at(level)` to `hp_base` and `hp`, `level_up` each later level's growth, as levels' own hit points are (so `Actor::max_hp` needs no level) |
| `healing_percent` | `World::healing_rate`, summed with the regeneration affixes |
| `mana_regen_percent` | `World::mana_rate`, summed with the mana regeneration affixes (Tidal, Clear mind) |
| `max_mana_per_levels` | handed out, not read: `class::hand_out` adds its `mana_at(level)` to `mana_base` and `mana`, `level_up` each later level's growth (Deep well; past `mana_level` too) |
| `mana_resting_percent` | `World::mana_rate`, on resting's pace (Meditation) |
| `mana_cost_percent` | `World::spend_mana` through `Actor::mana_cost` (`TalentMods::mana_cost`: rounded to the nearest, at least 1; Frugal caster) |
| `blood_magic` | `world/mana.rs::blood_magic` (`Action::UseTalent`, `use:blood_magic`, `Ability::BloodMagic` under `Source::Talent`): hit points for mana, no cooldown |
| `footsteps_percent`, `silent_doors` | `noise::noise` (after the stealth skill and affixes; a silent door is no noise at all) |
| `hearing` | `Actor::hearing`, with the listen skill |
| `threshold_percent`, `darkvision` | `WorldState::eyes` (`world/eyes.rs`, the design's B4): the base profile (`WorldState::vision`) as the talents change it; `Perception::compute` reads it, so the view's `vision()` is the effective eye (id and name the base's). Creatures keep their types' profiles |
| `scent` | `Perception::compute`: a creature suffering the status, in line of sight within the radius (not on the overworld), has its cell seen as darkvision would |
| `night_sight` | `Perception::compute`: the overworld's sight radius at night (`daylight::phase`) |
| `lantern_light` | `WorldState::carried_light` (`world/lamp.rs`, read by `LightCache::sources`): the preset of a lit (not hooded) lantern ready in the Tool slot (gameplay light: creatures see it too) |
| `ambush` | `World::sneak_dice` (`world/ambush.rs`): added to the Sneak attack skill's dice; `World::resolve` → `World::ambushed` rolls them after the strike when it hit a target that `World::unaware` (`world/surprise.rs`) says has not seen the attacker, or one that is Surprised; drawn only then |
| `initiative_bonus` | `World::startle` (`world/surprise.rs`): added to the initiative check (Wary) |
| `surprise_ease` | `World::status_cost` (`world/surprise.rs`): the percent of Surprised's extra cost spared, with Initiative's capped at 100 (Cool head) |
| `surprise_difficulty` | `World::startle`'s `craft` (`world/surprise.rs`): added to the difficulty of the check of those the holder catches unawares (the thief's Sneak attack) |
| `sneak_reach` | `World::sneaks` (`world/ambush.rs`): cells beyond `missile_reach` a missile or attack spell still sneak-attacks (Long reach) |
| `ally_mod` (`to_hit`, `protection`) | `World::ally_blow` (`fight.rs::resolve`) and `World::ally_protection` (`World::protection_of`), through an ally's *master's* mods (`world/ally.rs::ally_mods`; Beast-bond, docs/dev/allies.md) |
| `ally_mod` (`count`, `duration_percent`, `lasting`, `most`) | `World::cast_summon` (`world/summon_spell.rs`): more come, a longer `Ally::until`, none, each spell's `most` raised (docs/dev/allies.md "Summons") |
| `summon_bias` (`tag`, `weight`) | `World::summon_draw` (`world/summon.rs`), from `cast_summon`: the pool's weights (no item or talent yet; the owner's Q9 of #828) |
| `familiar` | `World::bring_familiar` (`world/familiar.rs`, from `World::begin`): a familiar as the run begins; `has_active` for Call familiar, `World::start_calling` (`docs/dev/familiar.md`) |
| `shared_sight` | `Perception::compute` (`perception/seeing.rs`): the companion's second viewpoint, its eyes and nose (Hunter's bond, `docs/dev/companion.md`) |
| `familiar_sight` | `Perception::compute` (`perception/seeing.rs`): the same viewpoint on the familiar, its nose at least `familiars.toml`'s `nose`; no buffs (Familiar's sight, `docs/dev/familiar.md`) |
| `shared_buffs` | `World::afflict` → `World::share_buff` (`world/bond.rs`): a buff lands on the player and their companion both (Hunter's bond) |
| `fletcher` | `world/fletch.rs`: a `count = "fletch"` recipe's time, its check, and its count after the margin (Fletcher, the ranger's at level 1, #1021; "never brittle" gone with #1336) |
| `sees_unaware` | `Perception::compute` (`perception/actors.rs`): `SeenActor::unaware` from `surprise::creature_unaware` for each creature in view, false for all without it; read by look mode ("unaware of you") and `ui/glyphs.rs` (the unaware mark) (Stalker's eye, the ranger's at level 2) |
| `startled_blow` | `World::startle` → `World::hair_trigger` (`world/free_blow.rs`): a check made against a hostile attacker beside the holder is answered by `Cue::Startled`'s free blow before the attack is rolled; `resolve` (and `cast.rs::spell_attack`) stop when it fells the attacker (Hair trigger) |
| `carry_percent` | `burden::load` (the limit before the affixes' kilograms, kept a multiple of 3) |
| `travel_percent` | `World::step_price` on the overworld, after the survival skill |
| `terrain_as` | `World::step_price`: the terrain stepped onto is priced as the one it names (local and overworld alike; its ids checked once the terrain loads, `TalentBook::check_terrain`) |
| `plunge_proof` | `World::fall_in`: no plunge damage and no draw for it (Water walking's status has `ice_holds` instead: the ice never gives way, `docs/dev/status-effects.md`) |
| `light_on_ice`, `climb_percent` | `World::tread` (the load on the ice is the body's weight alone), `World::tread_price` (the climb's price) |
| `wading_proof` | `World::wade`: no exposure, no draw |
| `beast_friend` | `beast_friend::befriended` (`world/beast_friend.rs`), read by `World::stance`, `ai::nearest_hostile_in_sight`, and `PerceptionView::stance`; `World::resolve` (an attack by the player) records the type in `Progress::empathy_broken` |
| `hunger_percent`, `no_exertion` | `World::hunger_percent` (the race's pace scaled), `exert` |
| `xp_percent` | `grant_xp` (every gain: kills, discovery, quests, dev) |
| `notice` | `search.rs`: `notices` (as if trained in Spot) and `search_modifier(Look::Passive)` |
| `free_blow` | `World::free_blows_of`: `FreeBlows::either` of the actor's own switches and the talents' |
| `weapon` | `Actor::benefit` (`TalentMods::weapon_benefit`): the chosen skill's (`WeaponBonus::skill`, the choice) or a role's benefits; `ranks` raise a held rank (`TalentMods::raised`); `bash` only where the base has a chance. Every profile builder, Defence's parry and shield, `attack_price`, and `reload_price` read benefits, so they need no read of their own |
| `launcher_range`, `throw_range` | `Actor::launcher_range`, `Actor::throw_range` |
| `max_dex`, `deflection` | `Actor::defence` (each worn armour's cap; on top of magic items' Deflection) |
| `protection` | `Actor::protection` through `talent_protection` (`TalentMods::protection_with`: the piece worn with the most Protection counts as body armour; a shield in a hand slot) |
| `stance_to_hit` | `Actor::stanced` (`TalentMods::stance_to_hit`: a penalty only, rounded toward 0) |
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
| `kick_back` | `World::player_kicks` (`World::knock_back` after the blow and its free blows, no status, no draw) |
| `dig_percent` | `World::dig_turn` |
| `two_handed_price` | `World::attack_price` (melee with a `two_handed` object) |
| `attack_price_percent` | `World::attack_price` (every melee attack; shots and throws are `shot_price`'s) |
| `critical_status` | `World::after_blow` (a confirmed critical in melee) |
| `penetration` | `Actor::melee`, `off_hand_blow`, `bash`, `kick` (`TalentMods::penetration_for` the profile's kinds) |
| `burden_dex` | `Actor::defence` (the burden's cap) |
| `sure_step`, `pit_proof` | `World::step_onto_trap` → `steps_lightly` (the player's steps only) |
| `speed_percent` | `World::burdened`, the one place every price passes (`quicker`) |
| `weapon` `crit`, `at_rank` | `TalentMods::weapon_benefit` (the held rank passed in) |
| `weapon_marks` | `World::practise` (`TalentMods::marks`; no `role`: every skill, Forge-born) |
| `blaze` | `world/element_gifts.rs` (`Action::Gift`, `Ability::Sign` under `Source::Sign`): the burst, the fires, then its `cooldown` status |
| `two_handed_crit`, `unarmed_die` | `Actor::melee` (and `kick` for the die, the bigger of it and `kick_die`) |
| `stance_dodge` (Wild guard) | `Actor::defence`, the stance's Dodge (`TalentMods::stance_dodge`) |
| `step_percent` | `World::step_price` (`TalentMods::step_percent`: body armour, burden, shield) |
| `disease_immune`, `checks` | `World::expose` (refused, nothing rolled), `disease_turn`, `search_modifier(Look::Passive)` (Willpower) |
| `no_penalty_to`, `raise_percent` | `TalentMods::penalty(status, stat, by)` in `Effects::penalty_for` |
| `free_blow` `flurry` | `World::second_blow` on `Cue::First` (`World::flurry`), after the off hand and the bash; two `cleave` sources make `great_cleave` |
| `flurry_cost` | `World::flurry`: the second blow spends its cost (Ki) first, else none (`TalentMods::flurry_cost`; `docs/dev/resources.md`) |
| `retribution { resource, cost }` (the paladin's at 25, #912; band-gated) | `World::retribution` (`world/smite.rs`), from `World::second_blow`'s `Cue::Attacked { hit }` arm before the riposte: a hostile beside the player whose melee attack hit them takes their smite (`smite_profile`) as `Free::Retribution`, its cost (zeal) spent first, else none (`TalentMods::retribution`; `docs/dev/combat.md`, `docs/dev/resources.md`) |
| `hardened` | `World::level_up`, as the level comes (a level's hit points again, and `bonus` more, past `hp_level`) |
| `resource_build` (Blood frenzy, Patient blade, Zealot, Battle rhythm: `melee_hit` and `melee_kill`, raised in `fight.rs::resolve` for a melee blow), `resource_max` (Mighty rage, Inner calm), `resource_start` (Battle trance), `resource_rate` (Battle trance, Inner calm, Steady hand, Patient blade) | `World::gain_resource` (beside the resource's own `builds`), `Actor::resource_max`, `Actor::resource_start` (`World::grant_resource`, `focus::hold`), `World::resource_rate` |
| `unstoppable` | `World::unstoppable` (`hurt`, `Harm::Actor`: `cost` of the rage via `spend_resource`), `World::live` (the numb statuses tick nothing while Raging lasts) |
| `radiance` | `LightCache::sources` (a carried source, `RADIANCE` tag, while its status lasts) |
| `inner_light` | `LightCache::sources` (a carried source, `INNER` tag, always: the player and the creatures see by it alike) |
| `patient_defence` (Patient defence, the ascetic's at 18, #849) | `World::patient_defence` (`world/ascetic.rs`), from `use_class`: a degree of its status (Poised) for a wait's price, the ki its talent's `resource`; `World::poised` switches the riposte on in `World::free_blows_of` and lets `World::riposte` answer a dodged blow (`Cue::Attacked { dodged }`, raised by `World::attack`); the Dodge is the status's `combat` |
| the actives (`rage`, `veterans_wind`, `lay_on_hands`, `avatar`, `smite`, `quivering_palm`) | `world/class_ability.rs` (Rage: `world/rage.rs`; `rage { resource }`, `TalentMods::rage_status`); `Ability::Class` in `ability.rs` (`class_cooldown`: Rage's status and what it `follows`; `class_cost`: a talent's `resource`, Avatar's `hands_cost_percent` of Lay on hands') |
| `smite { per_levels, sides, aim, unholy }` (the paladin's at 1, #910) | `world/smite.rs`: `World::smite(SmiteMode, dir)` (the one smite code: `SmiteMode::Plain`, `Branding`, `Wrathful`, `Banishing`; the rider by `World::smite_rider`) and `World::smite_profile` (the melee profile, `Smite::dice_at(level, unholy)` as its extra dice, `Content::unholy` reading `data/smite.toml`'s tags, the `aim` stat's modifier to hit); paid in zeal (its talent's `resource`), no cooldown |
| `smite_mode { mode, status, severity, turns, check, difficulty }` (the paladin's Branding smite at 12 and Wrathful smite at 32, #911; Banishing smite at 50, #913) | `TalentMods::smite_modes` (`SmiteRider::Brand`, `SmiteRider::Fear`, `SmiteRider::Banish`; `TalentMods::smite_mode(active)`), each its own `ClassActive` (`BrandingSmite`, `WrathfulSmite`, `BanishingSmite`: aimed, `strikes`, `holy`); `world/smite.rs`: Smite's blow (the mode needs the Smite talent's `smite`), then `World::smite_rider` on a hit that did damage: `brand` afflicts its status (Branded, which sheds `item_glow` by its `light`: no new reader); `fear` asks the foe `check::check` of its `check` stat's modifier (and its talents' checks) against `difficulty` + the smiter's Smite `aim` modifier, and failing afflicts its status (one that `flees`, checked on load) at `turns` degrees, `by` the player; `banish` takes nothing more: `World::mode_profile` doubles the extra dice on an unholy foe with no `Actor::summoner` (no line unless it wounds and leaves it alive: "reels under holy wrath"), and on one with a summoner the rider says "is unmade" and `World::unmake` kills it (`World::dies`, then what `resolve` does for a kill: `train_kill`, `siphon`, `Trigger::Kill`), past `unstoppable` and whatever its hit points; paid in zeal (its talent's `resource`), spent by `use_class` after the refusals and the bands |
| `holy_wrath { dice }` (Holy wrath, the paladin's at 40, #913; no bands) | `TalentMods::holy_wrath`; `World::holy_wrathed` (`world/smite.rs`), one line in `World::resolve` after `lore_blow`, so every melee blow, shot, throw, and free blow: on a hit, when `Content::unholy` says so of the target or the holder has `avatar` (`TalentMods::avatar`, read there too: Avatar alone is no wrath), one draw of the dice, `World::resisted` by the blow's kind, added to the blow's damage after its soak. No line |
| `catch_missiles` (Diamond soul's catch, the ascetic's at 40, #851) | `World::caught_missile` (`world/ascetic.rs`), from `resolve` (`caught_shot`, a shot or throw) and `trap_strikes` (`caught_trap_shot`, a trap's missile): a hit on one with no weapon and no shield in hand spends its cost (Ki) by itself and is a miss; `World::caught` ends the flight on that cell (`fly`, `trap_shot`), where the missile falls (`TalentMods::catch_missiles`) |
| `wind_step` (Step of the wind, the ascetic's at 6, #847) | `World::wind_step` (`world/ascetic.rs`), from `use_class`: a degree of its status (Light-footed), `Spent::NoTime`, the ki its talent's `resource`; `World::may_wind_step` refuses it before the spend while that status lasts |
| `spell` (`{ kind = "spell", spell = "ID" }`, checked against `data/spells/`) | `TalentMods::spells` → `Ability::Spell`'s `had_by`; `world/cast.rs` |
| `spell_mod` (`SpellModSpec`: `rungs`, `scroll_rungs`, `wand_rungs`, `reach_rungs`, `degree_rungs`, `sides`, `dice`, `bonus`, `reroll`, `rider`, `cells`, `radius_cells`, `degrees`, `cost`, `when`; a talent chosen for a school or an element keys on the choice) | `TalentMods::spell` → `Actor::spell_mods` (`world/spell_mods.rs`), read by `world/spell_rung.rs` and the cast path (the spell scaling design §7.1: Empower, Archmage, Keen mind, School focus, Scroll adept, Wand mastery, Sure hand, Brimming; `rider` has no shipped user since the spell types took Numbing cold's, `docs/dev/spells.md`); `cost` in `Actor::spell_base` |
| `spell_type` (`arc_chance`, `when`: damage types; Conductor) | `TalentMods::arc_chance`, read in `world/spell_arc.rs::spell_type_spreads` (`docs/dev/spells.md` "Spell types") |
| `spell_mana_percent` | `Actor::spell_cost` (the cast and `Unavailable::NoMana`) |
| `study` | `World::start_study` (time: reading a grimoire), `study_odds` (bonus; `overreach_percent`: Overreach's third off the overreach the check counts) |
| `deflection_unarmoured` | `Actor::defence` (Deflection, no body armour: Mage armour) |
| `purify`, `corpse_harmless`, `meal_heals` | `eat.rs`: `eat_rotten` (no sickness, no exposure, no draw), `eat_fresh` (no `on_eat` harm or vomiting, no draw), `meal_heals` from `finish` (an object, or a fresh corpse) |
| `status_damage` | `World::resolve` and `World::spell_strikes` (before resistance) → `World::lore_blow` (a hit at a creature suffering the status: Poison lore, Malison) |
| `hex_tithe` | `world/malison.rs` (`World::hex_tithe`, from `World::dies` before the creature leaves: a dying creature under a hex of the player's, `Content::hex_statuses()` with `Effect::by` the player, pays its level / `per_levels` in mana, at least 1, to `max_mana`; "The hex comes home." in `news`) |
| `reprieve` | `World::hurt` → `World::reprieve` (after Rise from the ashes; `Progress::once`) |
| `song`, `virtuoso` | `world/song.rs` (`World::sing`: `TalentMods::song`, Virtuoso's free and a degree greater; its one cost via `World::spend_resource` or `World::spend_mana`; `Act::Sing`; the status at `severity` plus Music's `Effect::SongDegrees`, on the singer or, `on = "hearers"`, on each hostile hearer who fails `check` against `difficulty` plus Music's `Effect::SongCraft`: `World::lull`, the Lullaby's); `Ability::unavailable` (`Unavailable::Short` or `NoMana` by its cost) |
| `cackle` | `world/witch.rs` (`World::cackle`, from `use_class`: the time of a cast; `World::own_hexes_in_sight`, the player's hexes, `Content::hex_statuses()` with `Effect::by` the player, on the creatures `perception` sees; each re-`afflict`ed at its severity plus `degrees`; refused, no time, with none; `Act::Cackle` at the talent's `noise`) |
| `charge` | `world/charge.rs` (`Action::UseAt`, `World::use_class_at`; `ClassActive::aims_cell`, `Aim::Cell`; `ClassActive::cost` is `Step`, so what blocks a step blocks it; the UI aims it on the line to the foe, `ui/target.rs`, and the menu row says its stamina, `Content::charge_cost`) |
| `volley` | `world/volley.rs` (`Action::UseAt`, `World::use_class_at`; `ClassActive::aims_cell`, `Aim::Cell`; the UI aims it as a shot, `ui/target.rs`) |
| `sic` | `world/sic.rs` (`Action::UseAt`, `World::use_class_at`; `ClassActive::aims_cell`, `Aim::Cell`; `Content::sic_range` the aiming mode's reach; `docs/dev/companion.md`) |
| `spirit_weapon` | `world/spirit_weapon.rs` (`Action::Shape`, `World::use_class_shape`; `ClassActive::aims_shape`, `Aim::Shape`; the UI opens the shape list, `ui/shape_screen.rs`); the gifts in `Actor::melee` and `Actor::off_hand_blow` (`spirit_weapon::gifts`: `to_hit`, `threat` lower, `penetration = ALL_PROTECTION`); `magic::Numbers::of` and `describe::lines` (`spirit_gift`) |
| `scribe` | `world/scribe.rs` (`Action::Scribe`, `World::start_scribe`; `ClassActive::aims_spell`, `Aim::Spell`; the UI opens the scribe list, `ui/scribe_screen.rs`, from `PerceptionView::scribes` and `blank_scrolls`); `Ability::unavailable` (a status blocking `cast` greys it) |
| `merchants_eye` | `world/merchants_eye.rs` (`Action::UseOn`, `World::use_class_on`; `ClassActive::aims_item`, `Aim::Item`, `Ability::action_on`; the UI opens the identify list, `IdentifyBy::Class`) |
| `favoured_prey` | `World::resolve` → `World::favoured` (`world/ambush.rs`), against `Quests::prey` |
| `ambush` (summed), `ambush_threat` | `World::resolve` → `World::sneaks` (before the strike), `sneak_threat`, `ambushed`; `Dice::and` adds the dice of the same sides |
| `exact_sounds` | `Perception::compute` → `noise::perceive` (`noise::Exact::All`) |
| `silent_unburdened`, `attack_noise_percent` | `noise::noise` (steps while Unburdened; attacks) |
| `trap_ease` | `search.rs::finding` (searching and in passing; `difficulty_of`, which disarming reads, is unchanged) |
| `pick_lock`, `steal`, `throw_voice` | `Ability::had_by` (and the actions' refusals: `lockpick.rs`, `steal.rs`, `voice.rs` ask it); no talent carries `pick_lock`, `steal`, or `trap_ease` since #684 took the thief's Trapfinding, Pick lock, and Light fingers (the kinds stay) |
| `lurk` | `world/thief.rs` (`World::lurk`, `use_class`; `thief_may` refuses "Too late to lurk."); the status switches the gatekeeper's free blow on in `World::free_blows_of`; `World::step_up` → `lie_in_wait` flags the blow (`Technique::lurk`), `resolve` → `lurked` makes its target unaware unless `Mind::last_seen` is set |
| `stillness { name, statuses }` (Stillness, on the ascetic's Still mind at 25, #850) | `World::stillness` (`world/ascetic.rs`, its own section), from `use_class`: every listed status the player suffers is removed and `World::ended` says its `end` words, `Spent::NoTime`, two ki its talent's `resource`; `TalentMods::stillness: Option<Vec<StatusId>>`; `Unavailable::Still` (`ability.rs::still`) greys it and refuses it before the spend with none of them to end; `name` is the menu's (`TalentEffect::active_name`), as the talent's own is Still mind; each status id is checked at load |
| `stunning_strike`, `quivering_palm` | `world/ascetic.rs` (`World::stunning_strike`; `World::palm_blow`, the one body of both blows, Quivering palm's caller in `class_ability.rs`); `use_class` refuses either with a weapon in hand (`ClassActive::bare_handed`) or nobody hostile there before the ki is spent; `TalentMods::stunning_strike`/`quivering_palm: Option<PalmBlow>` |
| `exploit` | `world/openings.rs` (`World::exploit`, `use_class`; `thief_may` → `exploit_may` refuses "There is no one there.", off the mark, and "It has no vital spot." at a seen foe without vitals; `ability::class_cost` makes its cost every opening held); `Technique::sneak`, taken in `resolve` by `sneak_taken`, passes `sneaks` as unaware (no `startle`, no flat-footed); `docs/dev/openings.md` |
| `thrown_exploit` | `world/openings.rs` (`World::use_class_throw`, `Action::UseThrow`, `use:thrown_exploit:c:12,5`; the thief's at 18): Exploit's blow as a throw, its ladder on the creature aimed at only (`Technique::at`); `docs/dev/openings.md` "The thrown exploit" |
| `exploit_dice`, `exploit_to_hit`, `exploit_status` | `TalentMods::exploit: ExploitMods` (dice summed by `Dice::and`): `World::exploit` adds n × `to_hit` to the profile and leaves the foe a degree of `status` on a hit; `openings.rs::land_exploit` rolls n × `dice` after the sneak dice (Exploit 1d6; Low blow +1 and Stunned) |
| `sneak_kill_status` | `World::resolve` → `World::vanishing_act` (`world/thief.rs`), where a sneak attack's blow kills |
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
| `weapon_ranks` | handed out, not read: `class::hand_out` (`raise_ranks`) raises every row of `skills` (none named: every weapon skill and style of the content but hidden styles, read from the table, never a list) held below `to` to it, `WeaponSkills::raise_to`; its `news` said by `grant_class` (Travelled hands, the bard's) |
| `take_up_skills` | handed out, not read: `World::take_up_skills` (`world/jack.rs`, from `grant_class`, `take_talent`, and `bestow`) trains every skill of the content the player lacks, but `except`, at `dice` from `levels_rng` in file order; its `news` said by `grant_class` (Jack of all trades, the bard's) |
| `ignore_burdened` | `burden::load` (`Load::ignores_burdened`, as a dwarf's) |
| `no_bloating`, `corpse_stats_percent` | `eat.rs`: `feed` (nutrition stops one below Bloated, nothing overeats) and `corpse_gifts` (a corpse's `train` marks and `widen` chances scaled, the chance at most 100) |
| `searching` | `search.rs::search_modifier` (searching only; Keen searcher; `search`, in both looks, is above) |
| `trap_damage_percent` | `World::trap_deals` (before the blood and the hurt; at least 1) |
| `kick_noise_percent` | `noise::noise` (`Act::Kick`) |
| `stat` | handed out, not read: `class::hand_out` raises score and potential (`Stats::gift`) at the pick or the grant; a Toughness gift raises `hp` with the maximum (Stubborn's +2 since #1406, its +1 on Toughness checks gone) |
| `weightless_coins` | `burden::load` (coins, item class `coins`, left out of the load) |
| `quest_gold_percent` | `World::complete` (`quest.rs`) |
| `favour_percent` | `world::shop::favour` (before the cap, `MAX_FAVOUR`) |
| `gem_worth` | `PerceptionView::appraisal` → `magic::describe::appraised` (the describe screen) |
| `identify_carried` | `World::appraise_carried` from `live`: `Magic::carried` counts the seconds (saved, left out while 0) |
| `gem_find_percent` | `gem::find_with` (the gems' chance scaled, at most 100) through `magic::loot::Luck` (`Luck::of(talents)`: with `loot_magic_percent`, from `level_template` and `World::loot_of`) |
| `skill` (its `skill` left out: the chosen one, Skill focus) | `Actor::skill_effect`, as magic items' skill bonuses |
| `distrusted` | `World::refusal`: no one budges (Trusted by none) |
| `checks` | the stat checks: `disease.rs` (both), `search_modifier`, `lockpick.rs`, `disarm_modifier`, `steps_lightly` |
| `critical_confirm` | `Actor::stanced` → `Profile::confirm`, added to the confirmation roll in `combat::strike` |
| `critical_penetration` | `Actor::stanced` → `Profile::critical_penetration`, added to the penetration of a confirmed critical in `combat::strike` |
| `burden_ease` | `World::burdened` (added to Balance's ease) |
| `disarm` | `Ability::Disarm`'s `had_by` |
| `weapon { skill }`, `protection { per_piece }` | `TalentMods::weapon_benefit` (a fixed skill or style: Fencing's Duelist, Big hands' maces and unarmed); `talent_protection` counts the pieces worn with Protection |

Birth sign gifts, the Roc to the Barghest (#267), and their hooks:

| Effect | Read by |
|---|---|
| `day_sight` | `perception/seeing.rs`, the overworld range by day (`night_sight` at night) |
| `exact_footsteps` | `noise::perceive` (footsteps at their own cell, never `rough_place`) |
| `stance_costs` | `World::set_stance`: a wait's price instead of no time |
| `max_hp_percent` | `Actor::max_hp`, `lasting_max_hp` (`Actor::frail`, at least 1) |
| `rise_from_ashes` | `World::hurt` → `World::rise_from_ashes` (`world/stars.rs`): the player at 0 hit points of anything but `Harm::Starving` (choking never passes `hurt`), while `Progress::once` lacks `risen`; sets it, afflicts the status, hit points to the share (of the maximum with it), `Event::Spell` over `burst_cells`, one gameplay roll of the damage for every creature reached (`spell_hurts`), `shock(Heat)` |
| `smell_rot` | `PerceptionView::smell(stack)` (a `Freshness` word from `rot::stage`, never an age), the eat list's mark |
| `free_blow` `gatekeeper` | `World::second_blow` on `Cue::Approaches` (`World::step_up` from `creature_steps`); with `one_in` (the Gatekeeper gift's 4) `TalentMods::gatekeeper_one_in`, a chance the stance scales (`World::gatekeeper_odds`, `gatekeeper_draw`), else `FreeBlows::gatekeeper`, every time |

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
| `wades_free` | `World::step_price` through `Actor::wades_free` (a status's too), beside `swimmer` (no disease immunity) |

The rest (Dragon's hoard, Deep roots' Dodge, Bedrock, Undertow's plunge) are
existing kinds.

Effects that name a status (`second_wind`'s `cooldown`, `status_eases`, `no_penalty`,
`critical_status`) or a damage type (`penetration`) are checked at load (statuses load before
talents) and resolved by `TalentMods::of(content, ...)`; `TalentEffect` is `Clone`, not `Copy`.
The `any_weapon_skill` need (`Need::AnyWeaponSkill`: `roles` and `of`, resolved to skill ids
with its words at load) is what "a melee skill at 3" asks.

## Class actives

The warrior, berserker, paladin, and ascetic (#243) have their tables the way the everyday folk do:
class-tree talents (`data/talents/`, `granted`) named by their class files' `abilities`, so
they are held like any talent, saved with them, and read through `TalentMods`. Their effects are
the talents' kinds and the class's own (the README of `data/talents/` lists them). The seven
actives (`rage`, `veterans_wind`, `lay_on_hands`, `smite`, `radiance`,
`quivering_palm`; since then the others in `ClassActive::ALL`, among them the berserker's
`war_cry { status, severity, difficulty }`, #844, `world/war_cry.rs`, paid in rage by its
talent's `resource`) are effects that `TalentEffect::active` maps to a `ClassActive`;
`Content::class_active` finds the talent that has one, whose name and description the abilities
menu shows; a holy gift's talent may name the bands it works for (`alignment`; outside them it
is used and does nothing, `docs/dev/alignment.md`). The priest's prayer (`prayer`, #883,
`docs/dev/prayer.md`) is one active, `orison`, carried by its tier-1 talent (`active` maps only
`tier = 1`); the later prayer talents add uses, answers, and titles to `TalentMods::prayer`, and
the menu names it by its highest tier and the face that answers (`PerceptionView::ability_name`). Communion with the dark (`communion`, #884) is a band-gated passive beside Consecrate: its
`alignment` (`max = "callous"`) is read where it acts (`World::communes`), as a holy gift's is.
A shared entry is one talent the tables
name alike (`purity_of_body`, the paladin's and the ascetic's, the priest's too until #883: #790 cut its copy
`divine_health`); Deflect missiles left the ascetic's table for Stunning strike (#848) and stays
the ordinary Finesse talent. A talent cut,
renamed, or merged into another needs nothing for saves: saves are pinned to the version that
wrote them (#840, docs/dev/saves.md), and `World::from_state` refuses a state holding a talent
the content does not have (`Talents::unknown_to`; `class_rogue_tests.rs`
`a_save_holding_a_talent_the_content_no_longer_has_is_refused`). Keys: the actives have none of their own (as Second wind; the abilities menu
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
priest named the paladin's `smite` and `purity_of_body` until the prayer (#883) took their
places (the druid named the ranger's `woodland_stride` until #791 cut it). The priest's guaranteed
spells are three (Mend, Blessing, Cure wounds): their rite is the prayer. The wizard's level 12 handed out the
Scholar `spell_recall` until #775 merged it into Scroll savant; it is a pickable of its own
again since #952, which a wizard picks from the Scholar tree.
New statuses: Blessed, Hexed (`ill_wished`), Weakened, Sluggish (`sluggish`, cost
150), Worn thin (the reprieves'); new effects `arcane_bolt` and `blight` in `data/effects.toml`.
Tests: `world/caster_tests.rs`.

## Loading

The loader trusts the data author (#1205): it checks only that the talent table is well formed,
so the game can build it and run without panicking. Kept: TOML that parses into the types
(`deny_unknown_fields`); unique ids, and ids free of `:` and spaces (action text reads
`talent:ID:CHOICE`); every id named resolves (talents, trees, systems, statuses, resources,
skills, weapon skills, races, classes, eyes, damage types, lights, spells and their effects,
terrain, heirloom weapons); the forms keyed on a choice have it (`skill` without a skill,
`resist` without a damage type, `weapon` without a role or skill, `chosen`'s own resolution);
a `while` only on an effect the sum reads (`may_be_conditional`: elsewhere it would be lost);
durations that parse; a die's sides and a divisor (`per_levels`, `one_in`) not 0; at most 255
trees or systems and 65,535 talents (the ids' widths). Everything else is the author's: numbers
in any range, a talent with no effects or no systems, two talents alike, a chain of choices of
different kinds (`Candidate::meets` reads a need of another kind as held at all), a flaw that
is needed, a resource cost or alignment on a talent that has no active. `data/talents.toml`'s
`[[system]]` list is read only by the wiki's talents-by-system table.

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
first; their effects (`starting_gold`, `starting_gems`, `heirloom`; `TalentEffect::is_handed_down`)
are hand-outs, not `TalentMods`: `World::hand_down` (`world/heirloom.rs`, one line in `learn`, and
one in `World::begin` for a class's level-1 talents: the aristocrat's Family wealth, #1404, so
`may_take` refuses it as held and Father's sword is open to their first pick) gives them once, as
the talent is taken or given, each from its own stream (`character/family-wealth/<spec>`,
`character/fathers-sword/<spec>`), so the kit and every other roll stay as they were. The
heirloom is `loot::heirloom` (2 affixes: magic, prefix and suffix; more: rare, split and named
as a rare item's), identified, and readied in its own slot only if that is free. Its weapons
are checked once objects load (`talent::check_heirlooms`). `kit::tests::
nobody_starts_strained_and_only_wealth_burdens` adds the heritage at its heaviest: never
Strained, and Burdened only for the weakest of a small race. Tests: `world/heirloom_tests.rs`.
The background story (docs/dev/background.md) reads the talents held after the creation picks:
its `only` lines force Family wealth's and Father's sword's chapters, so nothing is paid twice.
Landed noble (#1405; `heritage`, `needs = { creation = true, class = "aristocrat" }`) is no
hand-down: its `stipend_more` (`TalentMods::stipend_gold_percent`, `stipend_chance`,
`stipend_magic_percent`) is read by `world/stipend.rs::of_level`, as the aristocrat's class
ability Stipend's `stipend` effect (`TalentMods::stipend`, level 1 of the class table) is read by
`grow_stipend`, one line in `experience.rs::rise_from` after `level_up` (every level reached,
a prestige class's too). The rule and its numbers: `docs/dev/npcs.md` "The aristocrat's servant",
`data/stipend.toml`. The merchant's Invest (`invest`, `TalentMods::invest`, #1407) gates the
business partner's deals (`world/invest.rs`); the partner is placed by the talent id, as the
servant is: `docs/dev/npcs.md` "The merchant's business partner", `data/invest.toml`.

**Crafts** (#1418; the craftsperson design `docs/design/craftsperson.md` §3, §4): the `crafts`
tree, after `heritage`, holds the seven trades (Smith, Wright, Gemcutter, Brewster, Pitman,
Wortcunner, Spellwright; one file each in `data/talents/`), one per profession skill, each
`needs = { creation = true, class = "craftsperson", race_lacks = "<skill>" }`: a craftsperson's
creation pick alone (`Need::Class`, `Need::Creation`), never a level's or a bonus tree's.
`race_lacks` is `Need::RaceLacks(SkillId)`: met unless the character's race's guaranteed
`skills` name the skill (`RaceDef::teaches`; never `optional_skills` or a human's
`class_skill`); a run without a character meets it. Its words are derived from `races.toml`
(`Races::teaching`, file order): "not to be a dwarf or a duergar"; a `race_lacks` on a skill no
race teaches everyone is dropped at load (`resolve_needs`), so it never shows empty. The effect
is `trade { skill, dice, tools }` (`TalentEffect::Trade`, `is_handed_down`): `World::hand_down`
calls `World::learn_trade` (`world/trade.rs`), which rolls `dice` plus `TalentMods::trade_bonus`
(`trade_bonus { value }`, summed; Time served's, the craftsperson's at level 1) from the stream
`character/trade/<spec>/<skill>` into `Skills::add`, then makes each tool line (`KitLineSpec`,
`Content::kit_lines`; checked once objects load, `talent::check_trades`) from the same stream,
as a kit line is made, and puts it in the pack via `World::story_item` (readied or worn in a free
slot, its labelled type known). The sum keeps `TalentMods::trades: Vec<SkillId>`, read once in
`world::skill_roll` (its `trades` argument): a roll on a trade is one step up the ladder, as a
known-for skill's (`SkillRoll::known`; `SkillRoll::trade` when the step is the trade's, not the
class's; never two steps). Tests: `world/trade_tests.rs`,
`content::talent::tests::a_trade_names_its_skill_and_tools_and_the_races_it_is_closed_to`.

**The craftsperson's two** (#1419; the design §4, §5): the class file's `creation_bonus = {
count = 2, trees = ["crafts"] }` (`ClassDef::creation_bonus`, `creation_trees`;
`world/creation.rs::creation_bonus`) opens level 1's `Due` with `bonus: 2` beside
`talents: creation_count` (`open_creation`), and `World::bonus_trees_at(1)` answers the class's
creation trees (the level table's for every other level). The bonus machinery does the rest: a
Crafts talent spends a bonus pick before an ordinary one (`learn`), `settle_due` drops a bonus
pick no trade fits. `creation_offers` counts the bonus picks in what is left and offers the
creation trees' talents alone while one is due (`creation_choices`, also what `take_random`
draws from, so a random craftsperson gets two random trades its race allows, then its ordinary
picks); `creation_bonus_left` is the Talents step's title ("Choose a trade"). A third trade is an
ordinary pick. The class's level 1 abilities, granted in `begin` before the picks are taken,
are **Time served** (`trade_bonus { value = 6 }`: each trade `3d6+6`) and **Eye for the make**
(`eye_for_make`, `TalentMods::eye_for_make`, read once, in `PerceptionView::describe_item`,
which adds `magic::describe::make_lines`: what salvage would give at `salvage.toml`'s yield
(`piece::break_up`, the materials a profession works), or that the item is protected
(`world::protected_in`, the rules' `item_protected` over perception's quests, a unique or Proof only once identified, so nothing hidden leaks) or holds nothing
worth saving; for arms and armour `temperable` or `whittleable`, the improvements left to
`temper.toml`'s `most` or that it is damaged, never `attempts`; the main material's `work` in
words. No draw, no state). A scenario's own character may name its picks (`talents` in its
`character` table; docs/dev/character-creation.md). Tests: `world/craftsperson_tests.rs`.
No talent reads the story's draw: Hard-won and Storyteller were cut in #774, Favoured child
(#705) in #787, and the `background` system with it.

The creation step reads a drafted run (`World::draft`: `begin` with the kit, the picks left
due), cloned per question: `creation_offers(picks)` takes `picks` on a clone and returns what is
left and what may come next; `random_creation` runs `take_random` on a clone. Tests:
`world/creation_tests.rs`.

## Determinism and saves

Picks are replayed actions; prerequisites read stored scores. Talents draw from the gameplay
stream only where their condition arises (Light step's check, Ambusher's die on a hit at an
unaware creature), and some draw less (Swimmer's plunge, Nature's ward's wading, Iron stomach),
so a run without them draws as before. The hash covers `Actor::talents`, `Progress::due`,
`Progress::empathy_broken`, and `levels_rng`, each left out of the JSON while unused, so runs
without talents hash as before this change. Saves change (`Actor::talents`, `Progress::due`,
`Progress::empathy_broken`, `levels_rng`, all with serde defaults; the new light preset shifts the
light ids after it). A run without talents content (fixtures) never has a pick due.

## Tests

`world/class_active_tests.rs` (the adventurers' tables: abilities by level, bonus picks, every
passive's rule, the actives and their cooldowns, the holy gifts unanswered outside their bands, replays);
`content/talent.rs` (loading, the well-formedness refusals, choices' options), `talent.rs`
(summing, ranks per choice, serialization), `world/talent_arms_tests.rs` (the Weaponcraft and Guard hooks, the role need's words), `world/damage_talent_tests.rs` (Elemental ward and Crusher), `world/land_talent_tests.rs` (the Shadow and Wayfarer hooks), `world/talent_tests.rs` (dues and refusals,
prerequisites in words, Hardy's growth, several levels at once and `levelup:auto`,
dropped picks, every hook, a free-blow talent, saves and replays, perception and dumps).

**A talent is taken once** (#952, `docs/design/talent-ranks.md`; the machinery that let one be
taken again went in #966): a talent is its `needs` and `effects`, `Taken` is an id and a choice, and
`may_take` refuses one held (`Refusal::Held`); a talent with a `choice` is taken once per choice.
`Talents::take` of one held does nothing. Skill focus took Apprentice's place in the Scholar tree (#963). What used to be taken again and
repeated a number was dropped (Hardy, Thick skin, Deep well, Brawn, Quick, Keen ears,
Pack mule, Elemental ward); Armour training's two steps were folded into `max_dex` 2. Those that did a
different thing are follow-ups needing the first: Great cleave (Might; level 18), Breaker (Might;
Strength 14; `kick_back`, `dig_percent` 25), Old hand (Delver; level 10), Weapon specialization
(Weaponcraft; the same skill's Weapon focus, chosen at 5; `weapon { ranks = 1 }`, a weapon skill's
only), Spell recall (Scholar; Intelligence 15, level 12), and the bard's Master of none (granted
at 25). `needs = { talents = ["hardy"] }` is a list of ids (`Need::Talent(id)`, its words "Hardy"). Anvil and hammer and Second blade's rend, the styles' second tier, were cut with
`rend_off_hand`, until weapon and style feats are reworked.

## The burden line

The talents that eased burden were one line by #1198 (the owner: "combine talents that ease
effects of burden into one line. There are too many."): three Grit talents, each needing the one
before and adding to it (talents sum, `TalentMods::add`), so a rank is a talent held:

| Step | Needs | Effects |
|---|---|---|
| Pack mule | Strength 11 | `carry_percent` 10, `burden_ease` 60, `burden_dex` 1 |
| Porter | Pack mule, Toughness 12 | `carry_percent` 10, `burden_dex` 1, `ignore_burdened` |
| Beast of burden | Porter, level 12 | `carry_percent` 10, `stout` |

It replaced Roc's grip (the Roc's sign pick: the Roc has none now, its born take the line),
Stout (the dwarf's racial), Hard work (the commoner's 1, now Pack mule), Sure-footed (its
Bloated half went with it), Nimble, Uncanny dodge (the thief's 25, now Pack mule and Porter),
and Deep pockets' 10 kg (the merchant's 6 now gives Deep pockets, coins weightless, and Pack
mule). `carry_kg` and `step_slowdown_percent` went with their only talents. Kept: Roc's
strength and Supple, birth-sign gifts (a sign grants only `granted` talents of the `stars`
tree, `horoscope::check`, so they cannot name the line; the horoscope is kept whole). A table
that gives a step the character already took gives a pick in its place (`class::held_already`).
Tests: `talent_wits_tests.rs` (each step's limit, penalty, price, and Defence; the chain).

## Class tables

The everyday folk (#246: aristocrat, commoner, craftsperson, merchant) have their tables: nine
abilities each at 1, 6, 12, 18, 25, 32, 40, 50, 60 (34 class-tree talents, plus Riposte and
Second wind, which their tables hand out), and bonus picks at 10, 20, 30, 45, 55; the
craftsperson's table is being rebuilt (#1415: Time served and Eye for the make at 1 since #1419,
the rest phase by phase; Sturdy work, Masterwork, and Artisan deleted). The
warrior, berserker, paladin, and ascetic (I1) have theirs, in `world/class_active_tests.rs`; the
ranger, thief, and bard (I2, #244) theirs, in `world/class_rogue_tests.rs` (the kill tally and
Favoured prey, every passive, Volley and Inspire, the abilities the thief's skills give, Lurk,
Vanishing act (#684), the thief's table without Trained wind (#1463; Exploit is `world/openings_tests.rs`'s), and a save holding a talent the content dropped); the priest, druid, wizard, and witch (I3, #245) theirs, in `world/caster_tests.rs`. The I2 tables hand out two ordinary talents (Evasion,
Lore; and since #790 the bard's Fast healer and the druid's Beast-friend), so a creation pick
or a bonus pick may already have taken one; the table then gives a talent pick in its place at
that level instead (as for the ascetic's Deflect missiles before #848; `class::held_already`, counted into the
level's `Due` by `level_up`, with the news "You know Second wind already: choose a talent in its
place."; not on a restore, where a grown table only gives what is missing).
`class_tests.rs::a_talent_already_picked_is_not_given_again_but_brings_a_pick_in_its_place`. The showcase's and the arena's own
characters are thieves, the forest's and the world's rangers: their runs start with Sneak attack
or Favoured prey. `world/class_tests.rs` covers the grants, bonus picks,
restores, replays, and every hook; `content::class::tests` the table checks.

A table row is `{ level = 25, talent = "x" }` (`ClassDef::abilities` and
`PrestigeDef::abilities` hold `(level, talent)`; the shared `class::table_rows` and `check_rows`).
The loader refuses a talent named twice in a table. `class::grant` gives each talent the table
names up to the level that the player does not hold (a restore too), with its hand-out; the news
is "You gain Jack of all trades.", then the hand-out's own `news`. `held_already` counts a
pickable held. `class_rogue_tests.rs` covers the bard's Jack of all trades and Master of none, a restore past
both, and a skill the content adds being taken up.

A class's table stops at its `max_level` (`ClassDef::settle_cap`: never past the curve's top; a
row past it never comes), and a **prestige class's** table (`data/prestige/`, `PrestigeDef`, #876) is a second
one at its own levels, read once the character is called (`docs/dev/experience.md` "Prestige
classes"): `class::grant` walks `Character::class_levels` (every table with the level reached in
it), `held_already` and the bonus pick of a level read `Character::class_of`, and each `Due`'s
bonus trees are the class of its level's (`World::bonus_trees_at`). A prestige class's abilities
are class abilities too (`Content::check_class_tables` checks both tables) and are marked `(class)` in the dumps like a class's. One ships, the Hunter (`data/prestige/hunter.toml`,
#881): the warrior's hit points, two rolls, Athletics and Survival known for, bonus picks at its 5,
10, 15, 23, and 28 from Weaponcraft and Guard, and one row, Spirit weapon at its level 1; its door
is a campaign's calling. The tests' Warden (`world/calling_tests.rs`) gives Fencing at its level 1
and Riposte at 2.

## The trees so far

Vigour, Might, and Finesse (#237) ship whole; Battle trance and Inner calm (the class
resources) came with R5 (#253); `body_talent_tests.rs` covers each one's rule, `hunger_tests.rs` Iron stomach.
The statuses Winded (Second wind's cooldown) and Staggered (`cost = 150`, until the next action:
`per_action`, docs/dev/weapon-abilities.md) are in `data/status.toml`.

Scholar, Grit, and Delver (#240) ship whole; their five mana talents (Deep well, Clear mind,
Meditation, Frugal caster, Blood magic) came with #242 (H6; `docs/dev/mana.md`,
`world/mana_talent_tests.rs`), which took the `mana` system's `pending` off. `talent_wits_tests.rs` covers
each rule. Skill focus is the talent with a `skill` choice (a skill the
taker is trained in, `talent::may_take` refuses another as a `BadChoice`;
`talent:skill_focus:athletics`, one pick per skill; `skill { value = 25 }`), Treasure hunter the
first with an `either` prerequisite (`Need::Either`, "Intelligence 12 or Appraise 20").

The racial talents (#241) are the `racial` tree (`no_free`: each asks for its race): Elven
accuracy, Ageless, Diplomat, Stone sense, Dwarven resilience, Lucky,
Slinger, Small target, Tinkerer, Gem lore, Ferocity, Orcish hide, and the humans' granted
Versatile. The gnome's disarming talent is `tinkerer` ("Tinkerer"; the craftsperson's `tinker`,
the same effect, was cut in #787). With Diplomat and Gem lore the `talk` and `gems`
systems are held to the bar. Granting Versatile changes every human's talents, so human runs
hash differently from before. The dump's `talents` layer marks it `(race)`, the Talents page
"race", and the character screen's birth-sign list leaves it out (`OwnTalent::race`).
`racial_tests.rs` covers each rule, the prerequisites, the picks, and a replay.

The `resources` system (#225, `docs/dev/resources.md`) is in the list: its five talents (R5,
#253; Zealot, #914) are Battle trance (Might, berserker 6: rage starts 6 higher, decays at 67 %), Inner calm
(Finesse, ascetic 18: ki +2 at most, back at 150 %), Steady hand (Weaponcraft, ranger 12: focus back
at 150 %), and Patient blade (Shadow, thief 32: openings decay at 50 %, and the thief's kill of
their mark builds one more, the `mark_kill` event, docs/dev/openings.md), and Zealot (Grit, who holds Smite from level 6: a
blow taken builds a second zeal); the only class-limited picks (`needs = { class, level }`, Zealot's
by `needs = { talents = { smite = 1 }, level = 6 }`, the paladin alone holding Smite since #891). `world/resource_talent_tests.rs` covers each rule and the
prerequisites. A talent of a resource the class has not been granted yet holds the change until
it is (Martial arts grants ki at level 1, so Inner calm, at 18, always finds it). Wild guard (Might, berserker 6; #680) halves
the frenzy stance's Dodge penalty (`stance_dodge`), the stance a berserker's rage holds them
in (`world/frenzy.rs`); `world/frenzy_tests.rs` covers it.

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

**`Actor::boons`** holds the talents given for life outside every pick, with what
they hand out (`World::bestow`, `world/boons.rs`): a forgotten altar's Chosen and Forsaken, and
a background story's `talent` and `flaw` lines (#707, `world/background.rs`;
`docs/dev/background.md`). `stars::grant` puts them back on every restore beside the race's and
the sign's grants. `OwnTalent::story` (a boon a told line names, `background::gave`) marks the
story's "(story)" / "(story, flaw)"; `OwnTalent::boon` is the altar's (a boon the story did not
give). A class table naming a held boon later gives a talent pick in its place
(`class::grant` skips what is held, `class::held_already` counts it into the level's `Due`).

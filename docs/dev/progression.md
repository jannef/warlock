# Progression: talents and level-up choices

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08i-character.md ("Talents") and the design is `docs/design/progression.md` (§7, §10).
Keep it current with the code, as CLAUDE.md says. Experience itself (the curve, kill XP, the
hit points a level gives) is `docs/dev/experience.md`.

## Modules

| Where | What |
|---|---|
| `content/talent.rs` | `TalentBook` from `data/talents.toml` (settings: `max_ranks`, `[coverage]`, `[[tree]]`, `[[system]]`, each set in one file) and `data/talents/*.toml` (one `[[talent]]` a file, `ContentFiles::table("talents")` order; `TalentId` an index, saves name talents by id so it is **not** in the content fingerprint). `TalentDef` (tree, `ranks: Vec<Rank>`, `needs` for every rank plus each rank's own, `effects` per rank held plus each rank's own, `choice`, `systems`, `granted`), `Needs`/`Need` (the prerequisite kinds and their words: talents, stat scores, skill values, weapon-skill ranks, any skill of a role (`any_weapon_skill`), `chosen`, level, race, class, eyes, and `requires_alignment`, an `alignment::Requirement` read against `Actor::band`), `TalentEffect` (the typed effects), `Choice::options`/`name`, and the **coverage check** (`TalentBook::coverage_gap`, run at load: a `ContentError`). |
| `talent.rs` | The runtime: `Talents` (`Vec<Taken { id, rank, choice }>` on `Actor::talents`, saved and hashed, left out of the JSON while empty), `TalentMods` (every effect summed; `Talents::mods(content)` / `Actor::talent_mods`), `Candidate` (what prerequisites read: `Candidate::player`), `unmet`, `may_take` → `Refusal` (in words), `openings`, `offers` (the pick list, tree by tree). |
| `world/talent.rs` | `Due { level, talents }` in `Progress::due` (lowest level first); `World::level_up(from, to)` from `grant_xp` (a pick every `Experience::talent_every` levels, and the per-level hit points of talents held); `refuse_due` (the first line of `player_acts`: everything but `Talent`, `Advance`, `LevelUpAuto` is refused with "You have skill rolls to spend." or "You must choose a talent first."); `take_talent` (the first due with a pick, so a talent may come before the rolls), `level_up_auto` (dev); `settle_due` drops talent picks nothing can be spent on (`offers` empty and no roll left) and rolls when no skill is trained, asked again after each pick. `Due { level, talents, rolls }`: a level with rolls but no pick is due too. |
| `world/skill_roll.rs` | The skill rolls (#230, `docs/dev/skills.md`): `level_rolls`, `advance_skill` (`advance:SKILL`), `advance_auto` (`levelup:auto`'s rolls). `World::due()` and `candidate()`. |
| `world/experience.rs` | `grant_xp` scales every gain by `TalentMods::xp_percent` and calls `level_up`; the dev `xp:N` (`grant_xp_dev`). |
| `perception.rs` | `due()`, `own_talents()` (`OwnTalent`), `talents()` (`TalentView`: held rank, the next rank's needs with `met`, `openings`): what the level-up screen and the Talents page (#236) read. |
| `dump.rs` | layers `talents` (held, `Hardy 2/3`) and `progress` (dev: dues, `due level 3: 3 skill rolls, 1 talent`, takeable ids with choices, discovered kills); JSON `talents`, `due` (`level`, `talents`, `rolls`). |
| `action.rs` | `Talent { id, choice }` (`talent:hardy`, `talent:weapon_focus:long_blades`; ids keep their case), `Advance(skill)`, dev `LevelUpAuto` (`levelup:auto`), `GrantXp` (`xp:N`); all `Spent::NoTime` or refused. |
| `rng.rs` | `Rng::levels(seed)`, the `levels` stream, kept in `WorldState::levels_rng: Option<Rng>` **from the first draw** (the stat checks of each level, `world/stat_training.rs`; skill rolls, `world/skill_roll.rs`), so runs that never drew hash as before. |
| `xtask/src/wiki/talents.rs` | `talents.md` (trees with the rules), `talents-by-system.md` (the coverage matrix). |

## The hooks

`TalentMods` is read where the rules already read what magic items do (`Mods::worn`), one read
each; a new effect is a `TalentEffect` variant, a `TalentMods` field, and its one read:

| Effect | Read in |
|---|---|
| `dodge`, `natural` | `Actor::defence` (Dodge adds to the skills' Dodge; natural armour is added on top of the creature type's own and a magic item's, or stands alone: Thick skin stacks with an amulet of natural armour, by decision) |
| `melee_damage` | `Actor::melee`, `off_hand_blow` (after the affixes, before the stance; not kicks or bashes) |
| `max_hp_per_levels` | handed out, not read: `take_talent` adds the rank's `hp_at(level)` to `hp_base` and `hp`, `level_up` each later level's growth, as levels' own hit points are (so `Actor::max_hp` needs no level) |
| `healing_percent` | `World::healing_rate`, summed with the regeneration affixes |
| `footsteps_percent`, `silent_doors` | `noise::noise` (after the stealth skill and affixes; a silent door is no noise at all) |
| `hearing` | `Actor::hearing`, with the listen skill |
| `carry_percent` | `burden::load` (the limit before the affixes' kilograms, kept a multiple of 3) |
| `travel_percent` | `World::step_price` on the overworld, after the survival skill |
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
| `pain` | `World::hurt` for `Harm::Actor` and `Harm::Trap`, damage over 1 |
| `resist` | `Actor::resistances`, as gear (`Resistances::sum` caps it) |
| `resting_percent` | `World::healing_rate`, on resting's pace (Fast healer) |
| `second_wind` | `world/second_wind.rs` (`Action::SecondWind`, `Ability::SecondWind` under `Source::Talent`): hit points back, then its `cooldown` status |
| `iron_stomach` | `eat.rs`: `eat_rotten` (no vomiting, no draw for it; sickness halved) and `feed` (overeating never vomits) |
| `status_eases` | `World::live`: the sufferer's easing pace, `TalentMods::eases_every` into `Effect::until_eases_every`/`advance_every` |
| `no_penalty`, `penalty_percent` | `Actor::penalty` through `Effects::penalty_for` (`TalentMods::penalty`: lowerings only, toward 0) |
| `clotting` | `World::wounded` |
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
| `missile_dodge` | `World::target_against` (from `resolve` for shots and throws, `trap_strikes` for trap missiles) |
| `spell_damage_percent` | `World::evaded` in `spell_hurts` (bursts and bolts, before resistances) |

Effects that name a status (`second_wind`'s `cooldown`, `status_eases`, `no_penalty`,
`critical_status`) or a damage type (`penetration`) are checked at load (statuses load before
talents) and resolved by `TalentMods::of(content, ...)`; `TalentEffect` is `Clone`, not `Copy`.
The `any_weapon_skill` need (`Need::AnyWeaponSkill`: `roles` and `of`, resolved to skill ids
with its words at load) is what "a melee skill at 3" asks.

## Coverage policy

`data/talents.toml` holds the closed `[[system]]` list and `[coverage]` (`min_talents = 2`,
`min_class = 0` until the class tables land, `free_per_tree = 1`). A talent must name systems from
the list; a system or tree marked `pending = true` is counted and shown in the wiki but not yet
held to the bar. **A wave that brings a system's talents removes its `pending`** (and a tree's
once it has a talent without prerequisites); `min_class` becomes 1 with the class tables. A new
system of the game joins the list in the PR that ships its talents. The check runs at load, so
`Content::builtin()` (and the game) fail on a gap; `content::talent::tests` covers the gaps.

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

## Determinism and saves

Picks are replayed actions; prerequisites read stored scores. Nothing in talents draws from the
gameplay stream. The hash covers `Actor::talents`, `Progress::due`, and `levels_rng`, each left
out of the JSON while unused, so runs without talents hash as before this change. Saves change
(`Actor::talents`, `Progress::due`, `levels_rng`, all with serde defaults). A run without talents
content (fixtures) never has a pick due.

## Tests

`content/talent.rs` (loading, validation, the coverage gaps, choice chains), `talent.rs`
(summing, ranks per choice, serialization), `world/talent_arms_tests.rs` (the Weaponcraft and Guard hooks, the role need's words), `world/damage_talent_tests.rs` (Elemental ward and Crusher, the damage-types bar), `world/talent_tests.rs` (dues and refusals,
prerequisites in words, ranks and Hardy's growth, several levels at once and `levelup:auto`,
dropped picks, every hook, a free-blow talent, saves and replays, perception and dumps).

## The trees so far

Vigour, Might, and Finesse (#237) ship whole but for Battle trance and Inner calm (the class
resources, #225); `body_talent_tests.rs` covers each one's rule, `hunger_tests.rs` Iron stomach.
The statuses Winded (Second wind's cooldown) and Staggered (`cost = 150`, 6 s) are in
`data/status.toml`.

The damage-type talents (#248), Elemental ward (H2) and Crusher (H1), are the `damage_types`
system's two, so it is no longer `pending`; `world/damage_talent_tests.rs` covers the ward
against damage and status doses, its words on the character screen, Crusher by weapon kind, and
the bar. The witch's Blight (#245) needs no read of its own for its type: its burst names
`damage_type = "poison"`, and its Poisoned goes through `World::afflict`, which meets poison
resistance.

## Later waves

Skill rolls (#230) add `Due::rolls` and draw from `levels_rng`; stat training (#231, done:
`docs/dev/stats.md` "Training") checks every stat on it once a level, from `level_up`; the level-up screen, the Talents page, creation picks, and `--talents` (#236) read
`PerceptionView::due`/`talents`; the talent waves add files under `data/talents/` and one read
each; `granted` talents (races' `versatile`, the birth signs') are computed into `Actor::talents`
by whoever grants them and never offered (`may_take` refuses them).

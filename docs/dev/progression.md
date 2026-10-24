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
| `world/talent.rs` | `Due { level, talents }` in `Progress::due` (lowest level first); `World::level_up(from, to)` from `grant_xp` (a pick every `Experience::talent_every` levels, and the per-level hit points of talents held); `refuse_due` (the first line of `player_acts`: everything but `Talent`, `Advance`, `LevelUpAuto` is refused with "You must choose a talent first."); `take_talent`, `advance_skill` (refused until the skill ladder, #230), `level_up_auto` (dev); `settle_due` drops picks nothing can be spent on (`offers` empty), asked again after each pick. `World::due()` and `candidate()`. |
| `world/experience.rs` | `grant_xp` scales every gain by `TalentMods::xp_percent` and calls `level_up`; the dev `xp:N` (`grant_xp_dev`). |
| `perception.rs` | `due()`, `own_talents()` (`OwnTalent`), `talents()` (`TalentView`: held rank, the next rank's needs with `met`, `openings`): what the level-up screen and the Talents page (#236) read. |
| `dump.rs` | layers `talents` (held, `Hardy 2/3`) and `progress` (dev: dues, takeable ids with choices, discovered kills); JSON `talents`, `due`. |
| `action.rs` | `Talent { id, choice }` (`talent:hardy`, `talent:weapon_focus:long_blades`; ids keep their case), `Advance(skill)`, dev `LevelUpAuto` (`levelup:auto`), `GrantXp` (`xp:N`); all `Spent::NoTime` or refused. |
| `rng.rs` | `Rng::levels(seed)`, the `levels` stream, kept in `WorldState::levels_rng: Option<Rng>` **from the first draw** (nothing draws from it yet: skill rolls and stat checks will), so runs that never drew hash as before. |
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

## Coverage policy

`data/talents.toml` holds the closed `[[system]]` list and `[coverage]` (`min_talents = 2`,
`min_class = 0` until the class tables land, `free_per_tree = 1`). A talent must name systems from
the list; a system or tree marked `pending = true` is counted and shown in the wiki but not yet
held to the bar. **A wave that brings a system's talents removes its `pending`** (and a tree's
once it has a talent without prerequisites); `min_class` becomes 1 with the class tables. A new
system of the game joins the list in the PR that ships its talents. The check runs at load, so
`Content::builtin()` (and the game) fail on a gap; `content::talent::tests` covers the gaps.

## Determinism and saves

Picks are replayed actions; prerequisites read stored scores. Nothing in talents draws from the
gameplay stream. The hash covers `Actor::talents`, `Progress::due`, and `levels_rng`, each left
out of the JSON while unused, so runs without talents hash as before this change. Saves change
(`Actor::talents`, `Progress::due`, `levels_rng`, all with serde defaults). A run without talents
content (fixtures) never has a pick due.

## Tests

`content/talent.rs` (loading, validation, the coverage gaps, choice chains), `talent.rs`
(summing, ranks per choice, serialization), `world/talent_arms_tests.rs` (the Weaponcraft and Guard hooks, the role need's words), `world/talent_tests.rs` (dues and refusals,
prerequisites in words, ranks and Hardy's growth, several levels at once and `levelup:auto`,
dropped picks, every hook, a free-blow talent, saves and replays, perception and dumps).

## Later waves

Skill rolls (#230) add `Due::rolls` and draw from `levels_rng`; stat training (#231) the stat
checks on it; the level-up screen, the Talents page, creation picks, and `--talents` (#236) read
`PerceptionView::due`/`talents`; the talent waves add files under `data/talents/` and one read
each; `granted` talents (races' `versatile`, the birth signs') are computed into `Actor::talents`
by whoever grants them and never offered (`may_take` refuses them).

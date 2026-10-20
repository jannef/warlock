# Skills

Developer notes for working on this part of Warlock (moved from CLAUDE.md, #185); the
rules it implements are in spec/08i-character.md. Keep it current with the code, as CLAUDE.md says.

Skills (spec §8 "Skills", #132) live in `warlock-core`: `skills.rs` (`Skills` per actor: trained
skills only, a missing one is untrained and gives nothing; `Effect` and `Use`, the typed benefit
and training-mark hooks; `Benefit` growth by `steps` or a `curve`; the **provisional**
`Increase` curve), `content/skill.rs` (`data/skills.toml`; adding a skill is one entry), and
`world/train.rs` (marks for the player only, drawn from the `training` stream in `WorldState`,
never the gameplay stream). Race and class grants are dice rolled in `character.rs` after every
other roll, so they never move a score. Rules read a benefit through `Actor::skill_effect`
(the actor's skills plus what magic items' skill affixes add, `Skills::effect_with`), never
`Skills::effect` directly. Each effect is read in one place: Dodge in
`Actor::defence`, quiet in `noise::noise`, hearing in `Actor::hearing`, burden in
`World::burdened` (`Burden::slow_eased`), travel in `World::step_price`, bargain in
`world/shop.rs`, kick (athletics: Strength for kicking a door in) in `World::kick_door`, pick
(Open lock: picking a lock, and trained at all, the Pick lock ability) in `World::pick_lock`,
search, search reach, and spot in `world/search.rs` (`search_modifier`, `search_turn`; Spot
trained at all lets the player notice in passing, `Skills::trains_effect`), disarm in
`world/disarm.rs` (`disarm_modifier`; trained at all, Disable device gives the Disarm ability,
`Ability::had_by`); each use
is reported where it happens (`make_noise`, `hear`, `fight.rs::resolve`, `apply`, `player_moves`,
`kick_door`, `search_turn` for `search`, `notice_hidden` for `spot`, `World::disarm` for
`disarm`, `pick_lock` for `pick`). A new
effect or use is a variant there plus its one hook. The player sees values only
(`PerceptionView::own_skills`, the Skills screen `ui/skills_screen.rs`, `S`); never marks or
benefit numbers. Skills get no keys of their own: active uses are chosen from the one abilities
menu (#156, `docs/dev/screens.md`): each trained skill of kind `action` is an `Ability::Skill`
there (`warlock_core::ability`), unavailable ("not usable yet") until its use exists, when it
becomes an ability of its own in the skill's place (`Ability::use_of`): Disable device is
`Ability::Disarm` (`docs/dev/traps.md`), Open lock `Ability::PickLock` (`docs/dev/doors.md`).
Pending hooks:
athletics (climbing out of pits, #128) and escape artist (pits, #128).

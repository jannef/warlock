# Stealing

Developer notes for working on this part of Warlock; the rules it implements are in
spec/08i-character.md ("Stealing"). Keep it current with the code, as CLAUDE.md says.

Stealing (spec §8 "Stealing", #234; the progression design's B7, `docs/design/progression.md`)
is the Steal skill's use: `Action::Steal(Dir)` (replay text `steal:DIR`, cost `steal` in
`data/actions.toml`, `ActionKind::Steal`), dispatched by one arm in `World::player_acts` to
`World::steal` in `world/steal.rs`; tests in `world/steal_tests.rs`.

- **Who**: anyone trained in a skill with the `steal` benefit (`skills::Effect::Steal`, read
  through `Actor::skill_effect`; `Skills::trains_effect` for "trained at all", as Spot is), or
  holding a talent with the `steal` effect (the thief's Light fingers, #244; `Ability::had_by`), while
  the content has `data/steal.toml` (`content/steal.rs`, `StealRules`, `Content::steal`;
  optional: without it nobody steals). The thief's class trains Steal.
- **The ability**: `Ability::Steal` in `warlock_core::ability` (`Source::Skill`, `Aim::Direction`,
  cost `ActionKind::Steal`, no `Unavailable` of its own beyond a blocking status).
  `Ability::every` lists it in the Steal skill's place (`Ability::use_of`: a skill whose benefit
  is read by an ability's action is listed as that ability, not as `Ability::Skill`). The app's
  `KeyCommand::Steal` ("Steal", id `steal`) is in `REBINDABLE` after Bash and in `UNBOUND`; it
  answers `ScreenAction::UseAbility(Ability::Steal)` like the menu. No item list: the player
  never sees a creature's pack (`PerceptionView::gear` is what it wears and wields), so the rules
  pick the stack.
- **The rule** (`World::steal`): refused with no time when untrained, when no creature is seen
  on the cell (`World::seen_creature_at`, shared with talk), or when it is hostile
  (`World::stance`). Else priced first (`price_of(player, ActionKind::Steal)`, before the load
  changes), then the contest `check::contest(rng, Dexterity modifier + Steal benefit, victim's
  Willpower modifier)` (effective scores; four gameplay draws, the victim's roll first; ties go
  to the thief). Unnoticed: `lift` takes one stack the victim neither wears nor wields
  (`Equipment::holds`), uniformly (`rng.index(n)`, a draw only when there are two or more),
  `Equipment::after_removal` follows the victim's letters, and the stack joins the player's pack.
  Caught: `World::caught_stealing` (`shop.rs`) outrages a keeper of an open shop on the level as
  a robbery does (`outrage`), else the victim's `faction_override` becomes `StealRules::caught`
  (hostile to the player, checked on load), so the AI and every stance follow. Then
  `train(Use::Steal)` (training stream). Nothing is stored beyond what changed hands and the
  override (saved and hashed as they already are), so saves change only by content (the new
  `ActionKind`, no new indexed id).
- **The hook for alignment (#254)**: `World::got_away_with(victim, stack)` in `world/steal.rs`
  is the one place a theft that took something has consequences beyond the goods: it says "You
  steal ... from ..." and calls `player_did(Deed::Theft)` (`world/alignment.rs`; `[shift] theft`
  in `data/alignment.toml`, −100 like leaving a shop owing). A caught attempt and empty pockets
  do not reach it.
- **Shop goods** are never stolen with Steal: the shop's own rules (`docs/dev/shops.md`) cover
  taking from its floor. Stealing from a keeper takes from its pack (its purse); a sale it then
  cannot pay for does not happen (`deal`).
- **Wiki**: the Steal benefit's table comes from `data/skills.toml` (`xtask/src/wiki/skills.rs`),
  the odds by edge from `check::contest_ways` (`stealing()`, `generated/stealing.md`), the
  ability's row from `Ability::every`, the cost from `data/actions.toml`; the page text is
  `wiki/src/skills.md` ("Stealing").

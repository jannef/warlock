# Alignment

Developer notes for working on this part of Warlock (#254); the rules it implements are in
spec/08i-character.md ("Alignment"). Keep it current with the code, as CLAUDE.md says.

Alignment is good against evil, ADOM's law and chaos turned. Three modules:

- `crates/warlock-core/src/alignment.rs`, pure: `Score` (−1000 to 1000, `MAX`; serde through
  `i32` with `try_from`, so a save with a score off the scale does not load), `Score::shifted`
  (the one diminishing rule: a deed in the direction the score leans counts its worth times
  `(MAX − |s|) / MAX`, rounded toward 0; the other way, or from 0, in full), `Band` (Wicked up to
  Saintly, ordered so `>=` is "as good or better"; `ALL` best first; `side`, `scorned_by`),
  `Alignment` (a creature's fixed good, neutral, or evil; `band` for where a band is asked),
  `Requirement` (`{ min, max }` bands, `allows`, `check`, `words`: what data's
  `requires_alignment` deserializes to), and `check_start` (a race's or class's part of the start).
- `content/alignment.rs`: `AlignmentRules` from `data/alignment.toml` (optional;
  `Content::alignment()`): the seven `[[band]]`s with `from`, `description` (the wiki), and the
  `rise`/`fall` lines (`news(from, to)`), `[shift]` (`Shifts`, `ByAlignment` for the kills), and
  `[shops] favour` by band. Without the file the player has no score, everyone counts as Neutral,
  and nothing moves.
- `world/alignment.rs`, the rules: `Deed` (what moved a score; the hook the talents will read),
  `World::shift_alignment(actor, amount, cause)` (**the one place a score changes**; an actor
  without a score, every creature, is left alone; a band crossed queues its line in
  `World::notes`, said by `say_news` after the deed's own messages and the news, as notes that
  never stop a rest), `player_did(deed)` (deeds whose worth the rules fix: `Deed::worth`),
  `judge_kill` (stance first, then nature: hostile by `kill_hostile`; a beast, neutral and
  without a dialogue, by `kill_beast`; anyone else not hostile by `kill_peaceful`),
  `ate_corpse`, `scorns(actor)`, `cold_greeting`, `offers(quest, scorned)`, and
  `keeper_favour` (free function: a good `[keeper]` type's favour by band).

**State.** `Actor::alignment: Option<Score>` is the player's score (set at the start when the
content has the rules: `Start::alignment`, the race's plus the class's `alignment`, or 0 without a
character); `Actor::alignment_override: Option<Alignment>` is a level legend entry's `alignment`
(`CreatureSpawn::alignment`). Read the one that counts with `Actor::nature(content)` (a score's
band's side, else the override, else the type's `CreatureDef::alignment`) and
`Actor::band(content)` (a score's band, else the nature's band, Neutral without rules). Both
saved and hashed only when set. Creature types also have `race` (an id in `data/races.toml`,
checked only when races exist): eating such a corpse is kin to a character of that race.

**Hooks, one line each.** `fight.rs::dies` (`judge_kill`, beside `earn_kill` and `tally_kill`:
only the player's kills, a status they caused included), `shop.rs` (`provoke`: attacking a keeper
who trades; `shop_step`: theft; `pay`: paying a debt; `favour`: `keeper_favour`), `eat.rs::finish`
(`ate_corpse`, rotten or not), `quest.rs` (`complete`: the quest's `alignment`; `quest_topics`
takes `scorned` and asks `offers`), `talk.rs` (`chat`: `cold_greeting` before the greeting and the
lines; `topics` passes `scorns`). `steal.rs::got_away_with` (a theft that took something): `player_did(Deed::Theft)`. Nothing else
moves a score: a new deed is a `Deed` variant, a `[shift]` field, and one call where it happens.

**Readers.** Presentation reads `PerceptionView::own_alignment()` (the band, `None` without
rules): the character screen's `Alignment` row (after Stance), the chargen summary's
`Alignment` line (from `Start::alignment`), the `stats` dump layer (`Alignment Kind`) and the JSON
dump's `alignment` (the band id). The dev `actors` layer shows the score (`alignment 450 (good)`)
and every creature's nature (`alignment evil`). Never show the score or a creature's alignment in
play. Abilities: `ability::strayed(actor, content, requirement)` gives `Unavailable::Strayed`
("you have strayed") while the band falls short.

**For the class tables (#243, I1) and talents (#220).** A class ability or talent that asks for an
alignment carries `requires_alignment: Option<Requirement>` in its data (`{ min = "good" }` for
the paladin's gifts; parse it with serde and `Requirement::check` it on load); its arm of
`Ability::unavailable` returns `strayed(actor, content, def.requires_alignment)` first, then its
own reasons. The action it makes must refuse the same way (the menu greys it out, but a replay or
a bound key could still send it). The alignment talents proposed in the spec (Righteous fury,
Hardened heart, The middle way) hook into `attack_with` (the target's `nature`),
`shift_alignment` (by `cause`), and `Actor::defence`/`check`. Mana reads nothing of alignment.

**Tests.** `alignment.rs` (diminishing, bands, scorn, requirements, loading), `content/alignment.rs`
(the built-in bands cover the scale, news, bad files), `ability.rs` (a test ability's
requirement), `world/alignment_tests.rs` (starting bands, each deed, diminishing in play, notes
after the deed that do not count as said, cold greetings, quest gating and shifts, a good keeper's
prices, eating kin, a paladin straying, save and replay). The wiki's tables are
`xtask/src/wiki/alignment.rs`.

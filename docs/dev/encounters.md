# Overworld encounters

Developer notes for working on this part of Warlock; the design is
`docs/design/overworld-encounters.md` (its §15, the owner's decisions, overrides the rest), the
slices #803 (E0, the edge rule: `world/edge_leave.rs`, `docs/dev/overworld.md`), #804 (E1, the
rules, this file), #805 (E2, the dialog), E3 (the Ascmearc's tables, the wiki page, the spec
section), #807 (E4, resting and foraging roll too: "Camping and foraging" below), E5 (the
pedlar's offer). Keep it current with the code, as CLAUDE.md says.

## Modules and data

- `assets/data/encounters.toml`, `content/encounter.rs`: `EncounterRules` (`edge_reach`, E0's;
  `meetings: Option<Meetings>`), `Content::encounters()`, `Content::meetings()`. `Meetings`:
  `[rate]` (basis points an hour by landscape, every land landscape and the road named),
  `[phase]` (percent by day, twilight, night), `[checks]` (`difficulty`, `cover` by landscape,
  `night`, `twilight`, `rain`, `heavy_rain`, `light`), `[positions]` (`player = { ambush, fight,
  close, ambushed }` in percent of the way from the arrival edge to the area's middle; `fight`,
  `ambush`, `ambushed`, `close` in cells from the player), `[evade]` (`difficulty`,
  `detour_percent`, `aware`, `pursuit_per`), `[ambush] wait_seconds`, `[rest] rate_percent` (25: a
  quarter of the rate an hour of rest). The seven tables are all given or none (none: nobody meets anybody).
- `campaigns/<id>/data/encounters/*.toml`: `[[table]]` (`id`, `when`, `rate_percent`,
  `replaces`, `[[table.entry]]`: `id`, `name`, `weight`, `when`, `group = [{ creature, count }]`,
  `lead`, `stance = hostile|peaceful|wildlife`), read by `content/encounter.rs::parse` from
  `ContentFiles::table("encounters")` less the rules file, after the quests, sorted by campaign
  and id. A table outside every campaign is refused. `when`: `regions`, `landscapes`,
  `terrains`, `phase`, `weather` (`clear`, `overcast`, `rain`, `heavy_rain`), `level = { max }`
  (no minimum: the owner, Q5), `quest_active`, `quest_done`, `quest_not`, `revealed`, `hidden`,
  `after_days`, `before_days`. Load checks (`content/encounter_tests.rs`): regions of the
  campaign's overworld, landscapes, terrain ids, quests of the campaign, hidden locations of its
  overworld, creature types of the campaign or shared, no unique, a hostile entry of hostile
  types and a peaceful or wildlife one of types not hostile to the player, the first row's dice
  at least 1, at most 12 a row, `lead` one of the group, `replaces` another table, entry ids
  unique in the campaign. Not yet: the design's `draw` (a branch's population) and `npc` rows
  (E3, with their first use).
- `assets/data/skills.toml`: Spot's and Survival's `lookout` (and `Use::Lookout` marks on both),
  Stealth's `stalk`, Survival's `evade` (`skills::Effect::{Lookout, Stalk, Evade}`, read only in
  `world/encounter.rs`). Alert (`talents/alert.toml`) is read once more there (its `notice`) and
  its `systems` gain `awareness`.
- `world/encounter.rs`: the rules (below), `Pending`, `Party`, `Choice`, `Outcome`, `Forced`,
  `EncounterView`; `world/encounter_area.rs`: opening the area and the positions.

## The flow

1. `player_moves` (`travel.rs`) records an overworld step's price (`World::encounter_step`).
2. `World::apply` calls `encounter_tick` after the action's turn has run: the step's time has
   passed on the overworld. `encounter_roll`: nothing without the meetings, while one is
   pending, off a cell with a local area (`Content::local_area`), or in a campaign without
   tables. The matching tables (`when` holds; less those a kept one `replaces`), the chance
   `rate × seconds × phase% × Π rate_percent / (3600 × 100)` basis points, one `below(10_000)`
   of the `encounters` stream (`WorldState::encounters`, made on the first draw; none drawn when
   the chance is 0), then the merged entries whose own `when` holds by weight (one `below`), then
   `meet`.
3. `meet`: parties whose members are all gone are forgotten; a party of the entry still in the
   cell's area makes this a **known** encounter (no dice; the same actors are placed again;
   `KNOWN_LINE`, the owner's Q10); else each group row's dice (the `encounters` stream), the lead
   first. Peaceful and wildlife entries pend at once. A hostile one: the lookouts (or the dev
   `meet`'s forced outcome): the drop and met pend (`WorldState::encounter`); ambushed opens the
   area at once (no question); pass by does nothing and says nothing.
4. While pending, `refuse_pending` (after `refuse_due` in `player_acts`) refuses everything but
   `encounter:CHOICE`, the level's picks, the stance, and the dev switches that take no time
   ("You must decide what to do about them."). The dialog cannot be closed (the owner, Q9; E2).
5. `answer_encounter`: `pass` (no time); `evade` (half the step's price again whatever comes of
   it; a check; failed, the area opens `Close`); `ambush` (`wait_seconds` of waits); `fight`,
   `hail`, `hunt` (the `stairs` action, as `>`). The area opens after that turn, in
   `encounter_tick`, so no wait is ever spent in the area.

## Camping and foraging (#807)

The design's §14 Q3 default, which the owner widened to foraging (§15 Q3).

- **The roll**: each turn of a rest (`rest.rs::rest_until`) and of a forage (`forage.rs::forage`)
  calls `World::watch_roll(Watch, price)` from its `keep_at` turn, before the turn runs, for the
  turn's time (`price` / the player's speed). Where it rolls and what it draws is the step's roll
  (`World::odds`: the rules, the overworld, a cell with a local area, a campaign with tables,
  nothing pending; and nothing while an area is to open), but drawn exactly: one
  `below(10_000 × 3600 × 100)` of the `encounters` stream against rate (bp an hour) × seconds ×
  the percent (phase × tables) × the share, so turns of a few seconds add up to the rate an hour.
  The share is `[rest] rate_percent` (25) for a rest, 100 for a forage (a forager is a
  traveller). A chance of 0 draws nothing. Then `draw_entry` (the step's: the entries by weight,
  `meet`), the detour's `step` the price of a plain step onto the cell's terrain (as the dev
  `meet`'s), the side the opposite of the last overworld step, as ever.
- **The sleeper's lookout** (`World::sleepers_lookout`, a hostile entry met during a rest): one
  check, Willpower modifier + talents' Willpower checks + the player's hearing (Listen's `hearing`
  plus Keen ears' `hearing`, at most `MAX_ENCOUNTER_BONUS`; 0 for a deaf player) + Alert's
  `notice`, against `difficulty` + the lead's `stalk` + the rain (no cover, no dark: those hide
  from eyes). The party always finds the camp: made, **met** (woken in time); failed,
  **ambushed**. Never the drop, never pass by. Marks `Use::Hear` (Listen), not `Use::Lookout`.
  A forager's lookouts are the traveller's two.
- **What is said**: `Pending::woken` (saved, left out while false) words the dialog for a rest:
  "You wake to the sound of NAME close by your camp." (met), "You wake to find NAME at your
  camp." (peaceful), "... near your camp." (wildlife); ambushed: "You wake with NAME upon you!".
  The line is a `say`, so `keep_at` stops: "You stop resting after 2 hours." / "You stop
  foraging." (a forage cut short finds nothing). An ambushed sleeper's area opens in
  `encounter_tick` after the rest's action, as a step's does.
- Off the overworld (in the cell's local area too) a rest and a forage roll nothing; so does a
  campaign without tables (no replay moves).

## The checks (gameplay stream, two draws each)

- The player's lookout: Willpower modifier + talents' Willpower checks + `lookout` + Alert's
  `notice`, against `difficulty` + the lead type's `stalk` + cover + the dark (night or twilight,
  halved for a player with darkvision) + rain. Willpower, not the design's Intelligence: the
  passive notice (`search.rs::notice_hidden`) and the slip contest are Willpower in the code,
  and Alert needs Willpower 12.
- The party's: the lead type's Willpower modifier + its `lookout` (Spot's), against
  `difficulty` + the player's `stalk` + cover + the dark only for a lead without darkvision +
  rain − `light` while the player carries a lit light (`WorldState::carried_light`) at night or
  twilight.
- Evade: Dexterity modifier + talents' Dexterity checks + `evade` (+ `stalk` from the drop)
  against `[evade] difficulty` + pursuit (`(lead speed − 1000) / pursuit_per`) + `aware` when met.
- Marks (training stream): `Use::Lookout` on every lookout; `Use::Sneak` when the party missed
  the player; an evade made marks `Use::Travel` (and `Use::Sneak` from the drop).

## Where everyone stands (`encounter_area.rs`)

The area is visited as `>` visits it; **A** is `mapgen::local::arrival` for `Pending::side` (the
step's opposite; the dev `meet` without a step: the south). The way is the straight line from A
to the middle. The player stands `player.<outcome>` percent along it: lying in wait 10 (the
nearest cell within two beside opaque terrain, cover, if any), a fight 40, a failed evade 70,
ambushed 100 (the middle). The party, from the player: a fight `fight` cells further in, spread
across the way; lying in wait `ambush` further in with a looping route back past the player;
ambushed `ambushed` around them, the lead toward the edge; a failed evade `close` back toward the
edge. Every cell is the nearest free one to its mark among those reachable on foot from A
(walkable, not water). The party is made by `make_creature` (`rolls_gear`: `gear/<actor id>`)
and recorded as a `Party` (`WorldState::parties`). Awareness: a fight, a failed evade, ambushed:
`Mind::last_seen` the player's cell; lying in wait: `Mind::missed` the turn (slipped past: unaware
until hurt, the slip lapses, or their eyes find the player lit, `slip.rs`) and one normal
action of energy debt; every member's `Mind::noticed` the turn, so the player is never caught
unawares by one of them later. Ambushed: `World::startle(player, lead)` once, then `run_turn`
of one normal action (the party's head start; the Surprised status is fresh, so it is spent by
the player's next action, not by this one), then perception.

## Saved state and hashes

`WorldState::encounters` (`Option<Rng>`), `encounter` (`Option<Pending>`), `parties`
(`Vec<Party>`), each left out while empty, so runs without encounters hash as before. A campaign
without tables (the showcase, the Ascmearc until E3) draws nothing: no replay moves. The
`Underway` field on `World` (the step's price and the area to open) is resolved within the
action and never saved.

## What is shown

`PerceptionView::encounter()` (`EncounterView`: `name`, `title`, `text`, `choices` as labels and
`Choice`s) and `encounter_pending()`; the dump header `encounter pending: a band of kobolds
(ambush, evade, fight)`; JSON `encounter` (`entry`, `name`, `outcome`, `title`, `text`,
`choices`) or null, and with `--state` the dev `encounter_tables` (the ids matching the player's
cell); the `truth` layer's `encounter rate here N bp/h, tables: ...` on the overworld of a
campaign with tables; the `actors` layer's `of encounter ID`.

The app (`crates/warlock/src/ui/encounter.rs`, #805): `on_encounter` opens the one dialog
(`Dialog`, `docs/dev/screens.md`) from the view whenever the run changed with an encounter
pending and no such dialog open: after the step that met them and on load. Its title and text
are the view's (the known line included), a plain row per choice sending `Action::Encounter`;
`Dialog::required`: Esc does nothing and no info page opens over it (the owner, Q9; the
design's §5.3 Esc and info screens first are overridden). It runs before `on_level_up`, so a
level's picks, which `refuse_due` asks first, show over it. The messages are the core's `say`s
(the dialog's text is also the message line's). No new `KeyCommand`.

## Tests

`content/encounter_tests.rs` (the shipped file, the all-or-none tables, ranges, a fixture table,
each refusal); `world/encounter_tests.rs` (no draws without tables or while pending, a certain
roll, the choices by outcome, pass by makes nothing, the positions and awareness by outcome and
their order from the edge, the ambushing party unaware and in debt, the ambushed player
startled once, evade both ways, the known party, a save mid-question, replays through every
outcome, the dumps; #807: no draws resting without tables, a share, or the overworld, a rest stopped
and never the drop for a sleeper, met and ambushed, a forage cut short, camping replays); `magic/tests.rs::skill_reader` names `encounter.rs` for the three benefits. The app:
`ui/screen_snapshots.rs::encounters_as_text` (`encounter-drop`, with Esc and `1` pressed over it,
`encounter-met`, `encounter-peaceful`, `encounter-wildlife`, on a fixture table, the dialog as a
load opens it) and `an_encounter_row_answers_it`; `dialog_screen.rs`'s required dialog.

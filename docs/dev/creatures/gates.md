# Gates (the miscast's fiend portals)

General conventions: `docs/dev/creatures.md`; the summoning: `docs/dev/summoning.md`; the
miscast table: `docs/dev/spells.md` "Miscasts". Design: `docs/design/miscasts-review.md` §6.

The gates (#1295): nine shared creature types in `assets/data/creatures/`, one file each, made
only by a miscast. Each is opened by the miscast row of its own id (`assets/data/miscasts/<id>.toml`,
`effect = { kind = "summon", creature = "<id>" }`, all above the protected half), which puts it
on the first free cell around the caster (`world/miscast.rs::miscast_summon`, of its own
faction). They replace the `harm` stand-ins #1288 laid under the same ids.

| L | id | name | hp (base + To) | Prot. | lets through | count | every | most | row |
|---|---|---|---|---|---|---|---|---|---|
| 10 | `rift` | rift | 13 + 17 | 4 | dretch, quasit | 1d2 | 60 s | 4 | 75 |
| 10 | `hellmouth` | hellmouth | 13 + 17 | 4 | lemure, imp | 1d2 | 60 s | 4 | 77 |
| 16 | `gloom_gate` | gate of gloom | 20 + 20 | 5 | hell hound, nightmare, mezzoloth | 1 | 60 s | 4 | 79 |
| 30 | `abyssal_gate` | abyssal gate | 63 + 27 | 8 | balgura, chasme, hezrou, vrock | 1 | 90 s | 4 | 87 |
| 30 | `iron_gate` | iron gate | 63 + 27 | 8 | bearded, barbed, bone devil, erinyes | 1 | 90 s | 4 | 91 |
| 32 | `black_gate` | black gate | 62 + 28 | 8 | incubus, succubus, canoloth, night hag, nycaloth | 1 | 90 s | 4 | 93 |
| 42 | `dire_gate` | dire gate | 97 + 33 | 10 | nycaloth, erinyes, cambion, marilith, chain devil, glabrezu | 1 | 120 s | 3 | 101 |
| 52 | `arch_gate` | arch-gate | 122 + 38 | 12 | arcanaloth, horned devil, nalfeshnee, rakshasa, ice devil, ultroloth | 1 | 180 s | 3 | 111 |
| 64 | `lords_gate` | lords' gate | 160 + 40 | 14 | balor, pit fiend | 1 | 300 s | 2 | 123 |

Common to all:
- **Faction `loosed`** (#1301, `hostile_to_others`): hostile to every faction but its own, the
  player's, the kobolds', and the ordered fiends' alike. What it lets through comes `loosed` too
  (`creature_summons` passes the summoner's faction when it differs from the type's), with
  `summoner` the gate, so Banishing smite unmakes it. A kobold shaman's own rift eats the kobolds.
- **Rooted** (#1294, `world/rooted.rs`): it never moves or is moved. A rooted type with **no
  natural attack and no weapon in hand strikes no blow** (`World::rooted_plan`: a melee
  `Plan::Attack` is a wait; this PR's one line of code beside the data), so a gate's turn is a
  summoning when `summons_at` allows, else a wait. `world/rooted_tests.rs` checks it.
- **The summoning**: an id pool (shared types only), `mana` left out (no cost: a breeder's rule,
  so `base_mana` is 0; `content/summoner_roster_tests.rs` lets a rooted summoner have none), the
  default range 8 and spread 2, verb "lets through" ("The rift lets through a dretch!"). The
  `summoning` status is its cooldown; none at birth, so the first comes on its first turn with a
  hostile in sight.
- **Out of every draw**: `stray = false` and in no population, so only a miscast makes one;
  `rarity = "very_rare"` besides. `grades = false`.
- **Body**: glyph `0` in a colour each, darkvision, sight 8, no footsteps, a ton's weight, a
  "warded rim" of Defence 0 and the table's Protection, resist fire 100, poison 100, cold 50,
  immune to asleep, bleeding, diseased, frightened, held, poisoned, stunned, and transfixed;
  St 10, Dx 3 (an easy target), In 3, To and Wp `12 + L/2` (at most 40), Pr `10 + L/3`; `base_hp` the
  design's hit points less that Toughness. No corpse, no blood, no loot; its death line "The rift
  shudders and closes." (`dies`). XP `5L + 10`.
- **Its voice**: a `cry` every 90 s on average, loudness `40 + L` (at most 100): a wail, a howl,
  a scream, or (the iron gate) a banging, so it is heard through the walls.
- **Never timed out**: a gate stands until it is killed. What it called stays when it dies.
- **Tags** `summoner`, `fiend` (so a paladin's smite finds it unholy, `data/smite.toml`), and
  `gate`.

The miscast rows' words are the design's §8 lines.

Tests: `content/gate_roster_tests.rs` (every gate rooted, `loosed`, with no attack, no stray, no
corpse, no mana, its pool as above; each opened by its one row, of its id, above the protected
half); `world/gate_tests.rs` (a `rift` row on the built-in content opens the rift beside the
caster, hostile to the player and the kobold; it lets dretches and quasits through, `loosed`, at
no cost, never more than four, and never moves); `world/rooted_tests.rs` (a gate with no attack
strikes no blow beside the player).

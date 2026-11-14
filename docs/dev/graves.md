# Graves and burial grounds

The run's dead (the graveyard, `docs/dev/saves.md` "The graveyard") laid to rest in a chain of
hand-made levels: Līctūn, Cēaptūn's churchyard, then crypt after crypt below it (spec §7
"Graves"; #1258, #1682). The petrified are statues instead (`world/graveyard.rs`, spec §8
"Statues").

## Modules

| Module | What it does |
|---|---|
| `content/burial.rs` | `[burial]` and the legend's marks parsed into `LevelTemplate::burial` (`Burial`), the terrain check, the chains linked (`link_chains`), `Content::burial_chain`, `Content::burial_of` |
| `world/graves.rs` | `bury`: a level's places filled and its way down opened as the level is made and again on `World::set_graveyard`; `LevelState::unburied`, the level as the state hash sees it |
| `world/hash.rs::levels` | serializes `WorldState::levels` through `unburied` while hashing |
| `statue.rs::Fallen` | one of the dead: `name`, `race`, `class`, `petrified`, `death`; `grave_words` ("Brakka the dwarf warrior") |
| `perception/truth.rs::epitaph` | what look mode reads on a grave the player sees or remembers |

## Writing a burial level

A burial level is an ordinary hand-made level file (`assets/campaigns/<id>/levels/<level>.toml`,
`docs/dev/content.md`) with a `[burial]` table and a few legend marks. Nothing else is needed: the
chain is found from the files, so a new crypt is one new file.

```toml
[burial]
chain = "lictun"        # the chain's id: the id of its depth-0 level, the churchyard
depth = 3               # its place: 0 the churchyard, 1 the first crypt, 2 the second, ...
epitaph = "Here lies {name}, {death}."   # optional; this is the default
```

`{name}` is the dead's name, race, and class ("Brakka the dwarf warrior", either word left out
where unknown); `{death}` how and where they died, in the Graveyard screen's words with a lower
case first letter ("killed by a kobold in Fyrdelf 3, The Ascmearc"; "taken before their time"
when unknown). The epitaph must have `{name}`.

| Mark | Legend entry | Rules |
|---|---|---|
| A **place** for one of the dead | any cell of a terrain with `burial`: `grave_plot` (yard: a kerb of stones, becomes a `gravestone`), `empty_niche` (in a wall, opaque, becomes `sealed_niche`), `open_sarcophagus` (free-standing, becomes `sarcophagus`) | at least one per level; filled in **row order** (top row first, left to right) |
| A **fixed grave** (the town's own dead) | `{ terrain = "gravestone", epitaph = "Here lies ..." }` (or `sarcophagus`, `sealed_niche`: any `grave` terrain), one legend character each | its words exactly as written; no place for the run's dead. A `grave` terrain cell without `epitaph` is refused |
| The **way down** | `{ terrain = "floor", burial_stairs = "stairs_down" }` | one per level; its cell's own terrain (walkable, seen past, plain ground: no stairs, door, chest, or light) until every place is filled, then those stairs to the next level of the chain. Required on every level but the chain's deepest |
| The **stairs up** | `{ terrain = "stairs_up", to = "<the level above>" }` | required on every level but the churchyard, `to` the level at `depth - 1` |
| A **statue place** | `{ terrain = "floor", statue_place = true }` | plain ground; kept free for statues, nothing placed yet (`Burial::statue_places`) |

The burial terrain pairs (`burial = "<grave terrain>"` in `assets/data/terrain/`) are checked at
load to be the same in every rule (walkable, transparent, height, roofed, stone, bounce, muffle,
step noise, cost) and to be neither a light, stairs, door, chest, statue, station, nor
breakable: a place filled changes only what is drawn and read. A new pair needs the same.

Content checks (`ContentError` at load): places, `epitaph`, `statue_place`, or `burial_stairs`
without `[burial]`; `[burial]` in a vault; no place; an epitaph without `{name}`; two levels at
one depth of a chain, a gap in the depths, or a depth-0 level whose id is not the chain's; a
missing way down (but on the deepest) or stairs up.

Conventions for the crypts (the levels are the owner's to review): no `danger`, no creatures;
lit sparsely (`candles`, the `candle` light; `sconce`), a faint `ambient` (`[6, 6, 8, 0]`), a
cool `temperature`; 10 to 40 places; a few statue places; each level its own shape and story
(family vaults, an ossuary, a flooded crypt, a collapsed one, a sexton's store, a saint's
shrine). Ids `lictun_crypt_NN`, two digits.

## The chain

`content::burial::link_chains` runs once every level is read: it groups the burial levels by
`chain`, orders them by `depth`, checks the depths run 0, 1, 2, ... without a gap, and points
each way down at the next level (`BurialStairs::to`); the deepest's leads nowhere and never
opens, so the dead past its last place are not shown. Līctūn's chain: `lictun` (40 plots, ten
fixed graves by the chapel), `lictun_crypt_01` (The Undercroft, 26 places), `lictun_crypt_02`
(Bānhūs, 30 places).

## Burial

`world/graves.rs::bury(state, content, level)`, from `travel.rs::instantiate_level` and
`World::set_graveyard` (for the levels made already):

1. Undo what an earlier burial did (`buried`: each changed cell's former terrain; the opened
   stairs link).
2. Cut the fixed graves' words.
3. The dead: the graveyard (most recent first) reversed, the petrified left out: the **oldest
   death first**. The level's share is those after the places of the levels above it
   (`Content::burial_of`), up to its own places; each place takes its grave terrain and the
   level's epitaph.
4. When the dead fill every place (as many or more), the way down becomes its stairs and its
   link is pushed.

No draw: the same graveyard lays the same dead in the same places, whenever it is given.

## Out of the state hash

`LevelState::graves` (the inscriptions) and `LevelState::buried` (the changed cells) are skipped
while hashing (`unhashed_map`); `hash.rs::levels` hashes each level as `LevelState::unburied`
gives it: the places empty, the way down closed and unlinked, the player's memory of them so.
A place and its grave are the same in every rule, so a replay verifies under any graveyard, unless
it steps on an opened way down (the message log says so) or takes it. Saves keep everything.

## Tests

`world/lictun_tests.rs`: the way in and out, the sexton, the chain and the crypts' shape (quiet,
10 to 40 places, statue places, stairs up), the ten fixed graves and forty plots, the dead laid
oldest first without the petrified, the way down opening when full (not one short), the crypts
filled in turn down to the deepest (whose way down stays shut), the hash under any graveyard, a
graveyard given late and taken away, a save, the view snapshot `lictun_arrival`, and a replay
verified without the graveyard. `content/tests.rs`: the loader's refusals. `ui/look.rs`:
`a_gravestone_reads_its_inscription`.

Dev: `warlock-cli dump --fallen Name:race:class,...` (docs/dev/cli.md) lays a long list; the
`truth` layer lists every grave's words.

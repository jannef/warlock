# Warlock — The witch's familiar: her evil eye on four legs

**Built; this file is history.** Current behaviour is in spec §8 "The casters' class tables, and their spells" and `docs/dev/familiar.md`.

Design document for the owner's review (2026-11-03, #907). Written against `origin/main` at
39aad1ac (Cackle, W1 #908, merged). Nothing here is implemented; §13 cuts it into issues. It
builds on what is there: the **merged ally core** (`docs/design/druid-and-summons.md` §4 and
§14; `docs/dev/allies.md`; `crates/warlock-core/src/world/ally.rs`: `Actor::ally`,
`Ally { master, source, order, retinue, until }`, `AllySource::{Summoned, Companion}`, the ally
arm `ally_plan`, the three orders, `follow_along` and the retinue, `make_ally`, `ally_mods`,
the kill credit, the ally mark), the **approved ranger design** (`docs/design/ranger-companion.md`,
#832: `CompanionData { name, grade }`, growth, "always comes along", the collar, tending, the
story's chapter; its slices C0–C8 are issues, #916 C1 and #918 C3 among them, none built yet),
the **approved witch design** (`docs/design/witch-and-wizard.md`, #830: the hex loop; Cackle at 6
is merged, `world/witch.rs::{cackle, own_hexes_in_sight}`, `Content::hex_statuses()`; Malison at
18 is #905), the hexes as statuses with a giver (`Effect::by`, `World::afflict` in
`world/status.rs`, `data/status.toml`), the spells' degree ladders (`spell_degrees`,
`data/spells/evil_eye.toml`), the class tables and actives (`docs/dev/progression.md` "Class
tables", "Class actives"; `content/class.rs::ClassActive`), the multi-turn action loop a study
uses (`World::keep_at`, `world/learn.rs`), the abilities registry (`warlock_core::ability`), the
background story (`docs/dev/background.md`), and the talent coverage policy, rules 1 to 9 of #776
(`docs/dev/progression.md` "Coverage policy", binding). Every number is data and hidden in the
game; the wiki states it (the owner's rule: the game is vague, the wiki is accurate).

**The owner's request** (#907, from the witch design's Q11, default taken: "yes, as its own issue
once the ally core landed"): a witch's familiar as a third `AllySource` on the ally core. "Needs a
short design pass first (which creature, when it comes, what it does for her, the level, how it
returns when lost)." The brief adds: it should feed the witch's fantasy, hexes that stick and
come home, and not be a second combat pet; pets stay until killed, no releasing; no anti-farming
limits, time is the penalty; saves pinned to the game version, no shims or former-id aliases;
abilities in the one abilities menu; effects subtle.

## Contents

1. Summary
2. Research, tied to the decisions
3. What exists, and what this builds on
4. The familiar: what it is, and what it is not
5. What it does for her: the touch, the harrying, the loop
6. Which creature: the kinds, the names, the register
7. When it comes, and how it returns: level 1, and Call familiar
8. What it shares with the ranger's companion, and what is its own
9. The data
10. Determinism, saves, replays
11. What the player sees: messages, the status bar, look mode, dumps, the wiki, the spec
12. Talents and the coverage policy
13. Implementation plan (slices)
14. Rejected ideas
15. Open questions, each with a recommended default

## 1. Summary

| Piece | Proposal |
|---|---|
| **The familiar** | a third `AllySource`, **`Familiar(CompanionData)`**: an ally of the merged core that stays until slain, carrying the ranger design's `CompanionData { name, grade }` (its name, and its grade: its master's level). A witch has **one familiar** at a time; `World::familiar()` is the player's one ally of that source. It is a small, quiet beast on her side, not a hound: its blows are a scratch. §4 |
| **What it does for her** | **its touch is her evil eye.** When the familiar's blow lands on a creature, the creature is hexed as if she had cast Evil eye on it: `afflict(target, ill_wished, her degrees of Evil eye, Some(witch))`, **the witch as the giver**, for no mana and none of her turns (`[familiar] touch = "evil_eye"`, any hex spell's id). So her hexes **stick** (a blow of the familiar's on a creature already under her hexes refreshes every one of them, the way a second cast does: `stacking = "refresh"`) and **come home** (a hex the familiar laid is hers: Cackle lengthens it, Malison's bite and tithe read it, the kill credit sees her as the giver). The familiar **harries what she has hexed**: its target choice in the ally arm takes the nearest creature under a hex of hers first, the nearest hostile only when none is. §5 |
| **Which creature** | one of five small beasts, drawn as the run begins: `cat`, `toad`, `raven`, `hare` (new, shared, level 1, `wildlife`, `beast`, cheap) and the existing `bat`. The English witch's familiars, each with an Old English name for the thing (*catt*, *tādige*, *hræfn*, *hara*, *hrēremūs*). They differ in eyes, speed, and feet (a cat's dark sight and silent paws, a raven's and a bat's flight, a hare's speed), never in the hex. Its **name** is drawn from the names the English witch trials recorded (Tibb, Fancie, Pyewacket, Greedigut, …). §6 |
| **When it comes** | at **level 1**, the witch's third level-1 entry, **Familiar** (`familiar`, class, granted): "the witch is the one with the cat" from the first turn, as the ranger is the one with the hound. Made by `start_run` after the creation picks and before the story is told, so the story can name it. §7.1 |
| **How it returns when lost** | the Familiar entry also gives the active **Call familiar** (`use:call_familiar`, the abilities menu's Class group, nothing to aim): a **ritual of an hour** (`[familiar] call_turns = 600`, through `World::keep_at` as a study is, interrupted by a creature coming into view), **no mana, no cost but the time**, refused while her familiar lives ("Your familiar lives yet.") or a hostile is in sight. At its end a new kind is drawn and a new familiar sits at her feet, named, at her grade. No cooldown, no count: time is the penalty. No releasing: a familiar leaves only by death. §7.2 |
| **Shared with the companion** | `CompanionData` and its growth (the familiar's hit points, to hit, Defence, Protection, and stats rise with her level: the ranger design §8, "growth for every master"), **always comes along** (never left behind, held fast or not; the retinue on the overworld), the **collar** (Give puts an amulet on it: the witch's kit has a bone amulet), **tending** (Heal treats it), the story's chapter (one `only` line a kind), the status bar line, the Character screen's line. One generalisation: these rules read "a **kept** beast" (`AllySource::kept()`: `Companion` or `Familiar`), not `Companion` alone. A witch may have both a familiar and a tamed companion: they are different things. §8 |
| **Saved state** | `AllySource::Familiar(CompanionData)` inside `Actor::ally`; nothing on `WorldState`; a calling in progress is one action through `keep_at`, nothing in flight between actions. Four new creature types enter the content fingerprint (development saves invalidated, said in the PR; no aliases, no shims). No scenario or shipped replay has a witch: no replay, snapshot, or golden changes. §10 |
| **Talents** | **none new**: the `allies` system and its pickable Beast-bond are merged; Familiar is a class ability of the witch's table, tagged `allies`, `status`, `spells`. A sub-feature of allies and of the hex loop adds no tag and no talent. §12 |
| **Slices** | **F1** the familiar (the variant, the data, the four beasts, the start, the touch, the harrying, the shared rules read as `kept`, the words, the dumps, the wiki, the spec); **F2** Call familiar; **F3** the story's lines. F1 is on the ranger's C1 (`CompanionData`, growth, always along) when C1 lands first, and builds those shared pieces itself when the integrator wants the witch first. §13 |

**Key choices, and why**

1. *The familiar is the instrument of her hex, because that is what a familiar was.* In the
   English trial records (Chelmsford 1566, St Osyth 1582, Pendle 1612, Hopkins 1645) the familiar
   is not a bodyguard: the witch *sends* it, and it does the harm; it is fed on her blood, and it
   is the proof of her craft. The game's witch is "hexes that stick and come home" (#830 §4); a
   familiar whose touch is her Evil eye, with her as the giver, is that fantasy on four legs, and
   it adds **nothing** to the spell or status systems: one `afflict` call at a hit, the giver she
   already is, the degrees she already has.
2. *Not a second combat pet.* Its blow is 1d2; what the blow carries is the hex. The ranger's
   hound grows into a 3d4+4 killer by level 30 (ranger design §8); the familiar grows the same
   way (one rule) but from 1d2, so at 30 it bites for about 1d2+4 three times a minute while a
   hexed orc's blows land worse and die into her mana. The druid calls many for a while, the
   ranger keeps one that kills, the witch keeps one that **marks**.
3. *It harries what she hexed.* The one change to the ally arm is a sort key: a creature under a
   hex of hers before any other. So the loop needs no orders: she hexes the thing in front of
   her, the cat goes for it, the hex is refreshed at every scratch, Cackle holds the room's, and
   the kill pays Malison's tithe. The owner's "helpful without much ordering" (#828 Q2).
4. *Level 1, like the hound, because a familiar at 25 is a wizard's cat.* The witch's table has
   every cadence level filled (#830 §7); a familiar at any later level would displace an entry
   the owner kept. Level 1 is where a signature lives (the wizard has three entries there, the
   ranger three), and a level-1 Evil eye on a cat is one degree of Hexed (−2 to hit for a minute),
   a fair thing for a level-1 witch to have for free.
5. *It returns by an hour's ritual, for nothing but the hour.* The owner's rules: pets stay
   until killed; no anti-farming limits; time is the penalty. D&D 5e's Find Familiar is a one-hour
   ritual and the shape the players know. An hour underground is a real cost (hunger, the spawn
   clock, the fight she cannot start in the meantime) without being a gate; a night-long ritual
   or a mana price are the alternatives (§15 Q5). Pathfinder's lost spells and 3.5's lost XP are
   rejected (rule 1; the owner's "only the loss of pet").
6. *The companion's machinery, not a copy of it.* `CompanionData`, growth, always along, the
   collar, tending, the story's chapter, the status bar line: each already decided for the
   hound (#832), each reads one predicate. The familiar changes that predicate from `Companion`
   to `kept()` and adds its own two rules (the touch, the harrying) and its own calling. Nothing
   in the ranger's design assumed a hound (its §4.3 says so); this document holds it to that.

## 2. Research, tied to the decisions

| Source | What it does | Taken | Left, and why |
|---|---|---|---|
| **The English witch trials** (Chelmsford 1566: Elizabeth Francis's cat *Sathan*, fed on her blood, passed to Agnes Waterhouse and turned to a toad; St Osyth 1582: Ursula Kemp's *Tyffin*, *Tittey*, *Pigin*, *Jack*; Pendle 1612: *Tibb*, *Fancie*, *Dandy*; Hopkins's *Discovery of Witches* 1647: *Holt*, *Jarmara*, *Vinegar Tom*, *Sack and Sugar*, *Newes*, *Elemauzer*, *Pyewacket*, *Peck in the Crown*, *Grizzel Greedigut*) | the familiar is a small beast (cat, toad, dog, ferret, hare, rat, bird) the witch keeps, feeds, names, and **sends to do the harm**; "imp" is the Essex word for one | the kinds (§6.1), the names (§6.2), and the whole mechanic: the familiar carries the hex (§5) | the dog (the ranger's hound; a witch with a dog is a ranger), feeding on blood as a rule (a one-line fiction in the calling's words; a blood price is §15 Q5's alternative) |
| **Anglo-Saxon *wiccecræft*** (the witch design §2) | *wicce*, *hægtesse*, *galdor*, *lyblāc*; no familiar: the familiar is early-modern | the kinds' Old English names in the creature files' comments and the wiki (*catt*, *tādige*, *hræfn*, *hara*, *hrēremūs*); the witch design kept the early-modern witch (Cauldron 50, the bargain 60) and deferred the familiar: here it is | — |
| **Pathfinder witch** (the familiar holds her spells: she learns from it; a dead familiar costs the spells beyond her class list, replaced by a 24-hour ritual at 500 gp a level) | the familiar as the witch's own, from level 1 | level 1 (§7.1); a ritual to replace it (§7.2) | spells held by the familiar (losing it would lock her out of her craft: the owner's "never lock out content"; and the hex loop stands without it, #830 §11); the gold price |
| **D&D 3.5** (the wizard's and sorcerer's familiar: Alertness, empathic link, share spells, **deliver touch spells** at 3rd, speak with master, scry on it; its death costs 200 XP a level; a year and a day before the next) | "deliver touch spells" is the ancestor of the touch (§5.1): the familiar brings the caster's magic to the target | the touch, made hers (the giver) | XP loss (rule 1, and the owner's "only the loss of pet"); the year's wait (time is the penalty, but an hour, not a calendar) |
| **D&D 5e** (*Find Familiar*: a one-hour ritual, 10 gp; a spirit in a beast's shape (cat, raven, toad, bat, hawk, owl, rat, spider, …); it **cannot attack** but delivers touch spells; shared senses as an action; the warlock's Pact of the Chain gives an imp, quasit, pseudodragon, or sprite that fights) | the one-hour ritual (§7.2); the list of shapes (§6.1); a familiar whose worth is not its damage | the hour; the kinds; 1d2 blows | a familiar that cannot attack at all (the touch needs a hit; a non-attacking pet is a ghost); shared senses (the ranger's Hunter's bond; rule 3; §14); the imp (§15 Q7 offers it to a witch of 60 through Hag's bargain, as Pact of the Chain's) |
| **NetHack** (the Wizard starts with a kitten; pets are pets) | the role's pet from turn one | level 1 | nothing magical about the kitten |
| **ADOM** (no familiars; pets by Handle Animal; Necromancers' undead) | — | — | the ranger design took its taming; nothing for a familiar |
| **DCSS** (no familiars; summons; the Hexes school sets up kills) | the hex-and-payoff shape (#830 §2) | the harrying (§5.2) is the setup done by the pet | — |
| **ToME 4** (the Alchemist's golem: one permanent pet the class is built around, grown with the class, rebuilt when destroyed) | one kept pet that returns by the class's own means | Call familiar as the class's own return (§7.2) | the golem's gear slots (the collar is the one slot, the ranger's Q9) |
| **Warlock's own ally core and ranger design** (#836–#839, #832) | `Actor::ally`, the side, the arm, the orders, the retinue, the clock, the kill credit; `CompanionData`, growth, always along, the collar, tending, the chapter | all of it (§8) | the companion's `Tame` (a familiar is called, not tamed); the ranger's share (the witch's class has no `ally_share`: a familiar's rare kill pays 50 % as any non-pet class's ally does, C0; the hex's tithe is where the familiar pays her) |

## 3. What exists, and what this builds on

| Part | Today (merged) | Used for |
|---|---|---|
| `Actor::ally`, `Ally { master, source, order, retinue, until }`, `AllySource::{Summoned { spell }, Companion}` | the ally core; `Companion` is a variant with no data, reserved for #825 | `Familiar(CompanionData)` beside it (§4); `until = None` |
| `World::make_ally(creature, at, source, until)` | makes an ally on the player's side with no gear | the familiar at the start and at a calling |
| `ally_plan` (`world/ally.rs`): step 1 the nearest actor in sight with `stance_between(master, other) == Hostile` | the target choice, ties by lower id | the familiar's sort key: her hexed first (§5.2) |
| `fight.rs::resolve` → `World::ally_blow` (Beast-bond's `to_hit` on an ally's profile); the hit's resolution; `OnHit` statuses a natural attack lays | where a blow of an ally's is made and lands | the touch, one hook after an ally's hit lands (§5.1); not an `on_hit` row (its giver would be the familiar and its severity fixed) |
| `World::afflict(target, status, severity, by)` (`world/status.rs`); `Effects::add` with `refresh` (the higher severity, `wear` reset); `Effect::by` | how a hex is laid and by whom | the touch's one call |
| `Content::hex_statuses()`; `world/witch.rs::own_hexes_in_sight` (W1) | "under a hex of hers" | the harrying's predicate (the same read, on one creature) |
| `spell_degrees` / `SpellData::base_degrees()`, Evil eye's ladder `presence 14 2, 18 3` | the degrees she would cast a hex at (Presence, the school's and the talents' `degrees` hooks) | the touch's severity (§5.1) |
| `class::grant`, `hand_out`, `ClassActive`, `use_class`, `Ability::Class` | the class tables and their actives | Familiar at 1; `ClassActive::CallFamiliar` (§7.2) |
| `World::keep_at` (`world/learn.rs`: a study's turn loop, interrupted by a creature coming into view, a message, a blow) | multi-turn actions | the calling's hour |
| `World::start_run`: `take_creation`, the level 1 grants, `tell_background` | where a ranger's beast is made (C1: `bring_companion`, before the story) | `bring_familiar` in the same place (§7.1) |
| The ranger design (#832): `CompanionData { name, grade }`, growth's six hooks, `follow_along`'s companion filter, Give, `heal:ID:DIR`, the companion chapter with `Chapter::when` and the `companion` line condition, the status bar line, `World::companion()` | approved, slices C0–C8 as issues (C1 #916, C3 #918) | §8: read as `kept()`; `World::familiar()` beside `companion()` |
| `data/creatures/bat.toml` (level 1, `wildlife`, silent, darkvision, speed 2000) | a small flying beast | the fifth kind (§6.1) |
| `data/creatures/imp.toml` ("the familiar of those who bargain with hell", level 9, `devil`) | exists, in no population | §15 Q7: Hag's bargain's familiar |

## 4. The familiar: what it is, and what it is not

```rust
pub enum AllySource {
    Summoned { spell: Option<SpellId> },
    Companion(CompanionData),   // the ranger's (#832 change A)
    Familiar(CompanionData),    // this design: the witch's; id "familiar"
}

impl AllySource {
    /// A beast kept for good (a companion, a familiar): it grows, always comes along, wears a
    /// collar, is tended, and leaves a corpse. A summon is none of these.
    pub fn kept(&self) -> bool { matches!(self, Companion(_) | Familiar(_)) }
    pub fn data(&self) -> Option<&CompanionData> { ... }   // the kept beast's name and grade
}
```

| | The familiar is | It is not |
|---|---|---|
| **An ally** | an ordinary creature `Actor` with `ally = Ally { master: the witch, source: Familiar(data), order: Follow, retinue, until: None }`, on her side (`faction_override = player`), scheduled, hurt, and killed as any creature; it obeys the three orders (Come to heel, Hold here, Set on) as the one member of her pack; it counts one toward `[allies] most` (8) | a summon: it has no clock, leaves a corpse, is never dismissed |
| **Hers** | made by her class ability at the start and by her calling; one at a time (`World::familiar()`); its blow lays **her** hex (§5.1) | a companion: Tame does not make one, and a witch with Handle animal may also tame a hound (both: §8); nothing of its is read by the ranger's abilities (Pack tactics, Hunter's bond, Sic 'em read `companion()`) |
| **Small and quiet** | level 1 beasts, 1d2 to 1d3 blows, footsteps 0 to 20 (§6.1): a witch and her cat are quiet where a druid's pack wakes the floor (#828 Q11) | a fighter: its damage grows by the shared growth table (ranger design §8) from a scratch, and it gets none of Malison's bite (that is her blows and spells) |
| **Grown** | `CompanionData::grade` is her level, re-set at every level-up, read by the six growth hooks: hit points +2 a level above the type's, +1 to hit per 4, +1 damage per 6 and a die per 12, Defence +1 per 6, Protection by its body at the grade, St/Dx/To +1 per 5 (the owner's "growth for every master", #832 §20) | a second experience curve |
| **Always along** | wherever it stands on the level she leaves, held fast or not, holding or not (#832 §7, change B, read as `kept()`); the retinue on the overworld | left behind, ever |

## 5. What it does for her: the touch, the harrying, the loop

### 5.1 The touch

One hook, `World::familiar_touch(attacker, target)` in `world/familiar.rs`, called from
`fight.rs::resolve` where a **hit** of an actor's natural attack has landed and its `on_hit`
statuses are applied (the same point, after the damage), when the attacker's `ally` is
`Familiar` and the target is a creature (not the master; the arm never attacks her):

| Rule | Decision |
|---|---|
| **The status** | `[familiar] touch` names a spell of `kind = "hex"` (`evil_eye`); the status is its `effect.status` (`ill_wished`, "Hexed": `to_hit −2`, `max 3`, `eases 60`, `refresh`). Load check: the spell exists, is a hex, has a status |
| **The degrees** | the master's degrees of that spell **as they would cast it now**: `spell_degrees(master, spell)` (the base 1, Evil eye's ladder `presence 14 2, 18 3`, the school's `degrees`, Lingering, every `degrees` hook: the one function the cast uses). Not a fixed severity: the familiar's hex is as strong as hers, and grows as hers does. If the master does not know the spell (cannot happen for a witch; a fixture's familiar), the base degrees |
| **The giver** | `afflict(target, status, degrees, Some(master))`: **the witch**. Everything that reads "under a hex of hers" (`Effect::by == player` with `hex_statuses()`) sees it: Cackle (`own_hexes_in_sight`) lengthens it, Malison's tithe pays when the creature dies, Malison's `status_damage` bites on her blows and spells at it, a death of the hex credits the kill as the hex's giver's. The familiar's blow itself credits the master as any ally's (`fight.rs::dies`, at the class's share) |
| **Sticking** | `Effects::add` with `refresh`: a hex already there at the same or a higher severity is **refreshed** (its current degree's time starts again), a lower one is raised. A familiar scratching a hexed orc every turn holds Hexed on it for the whole fight for nothing. **Her other hexes on the target** (Weakened, Sluggish, Drowsy, Webbed) are refreshed too, each at its current severity (`afflict(target, s, sev, Some(master))` for each `Effect` on the target whose `by` is the master and whose status is in `hex_statuses()`): the familiar's touch keeps **every** hex of hers on its target going, the way a cast of Cackle does for one creature at no noise. That is "hexes that stick" |
| **Cost** | none: no mana, no turn of hers, no mark (the familiar's blow is not a cast: no school mark, as Cackle gives none, #830 §5.1); its blow's own time is the creature's attack cost as any |
| **Messages** | the hit is said as any ally's ("Tibb scratches the kobold."); the status's own lines for a creature are silent (`Added::Worse` and a refresh say nothing for creatures; look mode says "hexed"). Nothing new |
| **Determinism** | no draws beyond the blow's own |

**Why not an `on_hit` row on the creature type** (`on_hit = [{ status = "ill_wished", chance =
100, severity = 1 }]`): the giver would be the familiar, so neither Cackle nor the tithe would see
the hex as hers, the severity would be fixed where hers grows, and a wild cat (a tamed one, a
day's) would hex too. The hook reads the master; the creature file stays a cat.

### 5.2 The harrying

In `ally_plan` step 1 (the target: the nearest in sight the master is hostile to, ties by lower
id), a `Familiar` ally sorts **creatures under a hex of its master's first** (`Effect::by ==
master`, status in `hex_statuses()`; the same read as `own_hexes_in_sight`, on one creature),
then by distance and id as any ally; *Stay* and *Hunt* keep their reach rules and the leash keeps
its (`[follow] leash` 8). Nothing else of the arm changes: it fights with `ai.rs::fight`, heels
within `heel`, obeys the three orders.

So: she casts Evil eye or Hex of weakness on the thing in front of her, and the familiar goes for
**that**, not the nearest kobold; every scratch refreshes her hexes on it; she daggers it with
Malison's +3 (18) and it dies into her mana. With nothing hexed in sight the familiar fights as
any ally does, and its first hit hexes what it bit, so her next blow is already Malison's. The
owner's "helpful without much ordering" (#828 Q2): the cat's default is her loop.

### 5.3 The loop at four levels

| Level | Her pool | What happens |
|---:|---:|---|
| 1 | 13 | Evil eye (1 mana) on an orc; Tibb goes for it and refreshes Hexed at every scratch; the orc swings at −2 for the fight; her dagger finishes it. Or she keeps her mana: the cat's first scratch hexes for her |
| 6 | 47 | Cackle holds the room's hexes, hers and the cat's alike (they are hers). The cat holds one creature's for nothing |
| 18 | 180 | Malison: every hexed creature takes +3 from her dagger, touch, and Cause wounds; the cat's hex is hers, so a creature it scratched pays the tithe when it dies: half its level in mana. The familiar is now worth about a hex a kill to her pool |
| 40 | 400 | Blight (a poison burst) on a pack; the cat picks the hexed one; Cackle; the kills come home. A familiar of grade 40 has about 90 hit points and a 1d2+6 scratch: it lives through the fight and kills nothing by itself |

Nothing above adds a number to the game: the hex's degrees, Hexed's −2, and the tithe are the
data they are.

## 6. Which creature: the kinds, the names, the register

### 6.1 The kinds

Five small beasts, `[familiar] kinds` in `data/familiars.toml`, one drawn at the start and at
every calling (the five equal; §15 Q2 asks about race lists and a jackpot). Four are new, shared
(`assets/data/creatures/`, faction `wildlife`, tag `beast`, `alignment = "neutral"`, level 1, in
no population and no local area: they come only as familiars until someone places them;
`docs/dev/creatures/beasts.md` rows, the roster tests' rows). The glyphs follow the roster's
conventions (`docs/dev/creatures.md`); those below are proposals.

| Kind | Old English | Glyph | Speed | Max HP | Blow | Eyes, sight | Footsteps | Hearing | Weight | What it is for |
|---|---|---|---:|---:|---|---|---:|---:|---:|---|
| `cat` | *catt* | `f` | 1300 | 10 | claws 1d2 slashing | darkvision, 10 | 10 | 6 | 4 kg | the witch's cat: sees in the dark, silent, quick |
| `toad` | *tādige* | `t` | 900 | 10 | bite 1d2 piercing | low-light, 6 | 0 | 15 | 0.3 kg | silent and slow; its flesh poisons (`on_eat harm = "1d4"`, "Bitter and foul: your mouth goes numb.") |
| `raven` | *hræfn* | `B` | 1300 | 9 | beak 1d3 piercing | normal, 12 | 0 (it flies) | 8 | 1.2 kg | the far-seeing one; the keenest in daylight |
| `hare` | *hara* | `h` | 1600 | 10 | kick 1d2 bludgeoning | low-light, 9 | 20 | 4 | 3.5 kg | the fast one, the shape the old tales give the witch herself |
| `bat` (exists) | *hrēremūs* | `b` | 2000 | 10 | bite 1d3 piercing | darkvision, 8 | 0 | 3 | 0.5 kg | the fastest, the keenest ears |

Stats on the files' conventions (St 2 to 4, Dx 15 to 18, To 9 to 10, In 2, Wi 10 to 14, Pr 4 to
8; max HP = Toughness + `base_hp`), XP by the guideline (`5 × level + 10`: 15), `corpse = true`
(a kept beast leaves one), a `dies` line each ("The cat goes limp."). None of them is
`tameable` unless the ranger's lists want one later (the hare could be a halfling's start): a
familiar is called, not tamed. All five are quiet and quick enough to heel (`heel` 2 at speed
900 and up); none `opens_doors` (it waits before a door and comes through when she opens it, as
the core says). The familiar's own lines (`familiars.toml`, by kind): how it arrives ("Tibb, your
cat, winds about your ankles." / "Pigin, your toad, squats in your shadow." / "Newes, your raven,
settles on your shoulder." / "Fancie, your hare, crouches at your heel." / "Holt, your bat, hangs
from your hood."), said at the start and at a calling's end.

### 6.2 The names

`[familiar] names`, drawn once a familiar on the stream `companion/name/<actor id>` (the
companion's, one draw), said without an article as a unique character's (`World::called`): Tibb,
Fancie, Dandy, Tyffin, Pigin, Tittey, Suckin, Robbin, Jarmara, Holt, Newes, Elemauzer, Pyewacket,
Greedigut, Sathan. These are the names the English trials recorded (§2), Middle and early-modern
English nicknames: the world's register allows them (the owner's rule: Old, Middle, and Victorian
English by age; a familiar's name is an old woman's pet name, not a chronicle's), and the witch
design kept the early-modern witch for her top entries. #378 (the names review) may rename any.
§15 Q6 offers Old English names like the companions' (Blæc, Græg, Scead) instead.

### 6.3 The register

The creature files' comments and the wiki name the thing in Old English and the folklore in a
line ("The witch's cat, *catt*: it sees in the dark, walks without a sound, and in the old trials
it was fed on a drop of its mistress's blood and sent to do her harm."). In the game the familiar
is "your cat" and its name; no word of pacts or devils at level 1 (those are Hag's bargain's, at
60: §15 Q7).

## 7. When it comes, and how it returns

### 7.1 Level 1: the Familiar entry

```toml
# assets/data/talents/familiar.toml
[[talent]]
id = "familiar"
name = "Familiar"
tree = "class"
granted = true
description = "A small beast of the old craft keeps to your heel. Its touch is your evil eye, and it goes for what you have hexed. Lost, it is called again in an hour's sitting."
effects = [{ kind = "familiar" }]
systems = ["allies", "status", "spells"]
```

`{ level = 1, talent = "familiar" }` joins `10-witch.toml`'s table (its third level-1 entry,
after Evil eye). `TalentMods::familiar` is noted by `class::hand_out`; **`World::start_run`
makes the beast** once the level is laid and the player stands on it, after `take_creation` and
before `tell_background` (so the story knows it: §8), beside the ranger's `bring_companion`:
`World::bring_familiar()`:

1. **The draw**, on the stream `familiar/<race>,<gender>,<class>` (`Rng::stream`; one
   `below(kinds.len())` draw; nothing else moves).
2. **The actor**: `make_ally(kind, free_cell_near(player), Familiar(CompanionData { name, grade:
   the player's level }), None)`: the player's faction, no gear, following. On the overworld
   (a scenario starting there) it is made in the retinue, as the dev `ally:` is.
3. **The name** (§6.2) and the kind's arrival line.

On `from_state` nothing is made (a saved run has its own, or lost it and may call); with
`World::familiar()` already `Some`, nothing is made. A `--character ...,witch` run and New run's
witch have one from the first turn; no shipped scenario's character is a witch, so no scenario
changes.

The table after (12 entries; every cadence level filled, as #830 §7 left it, with Malison
pending in #905):

| Level | Entry |
|---:|---|
| 1 | Herb lore, Evil eye, **Familiar** |
| 3 | Hex of weakness |
| 6 | Cackle |
| 12 | Poison lore, Cause wounds |
| 18 | Malison |
| 25 | Second sight |
| 32 | Curse of sloth |
| 40 | Blight |
| 50 | Cauldron |
| 60 | Hag's bargain |

Nothing goes, nothing moves.

### 7.2 Call familiar: the return

The Familiar entry gives the active **Call familiar**: `ClassActive::CallFamiliar`, `Aim::Nothing`,
`use:call_familiar`, a row of the abilities menu's Class group ("Call familiar (class): nothing to
aim"), had by whoever holds `familiar`. The one place is `world/familiar.rs`.

| Step | Rule |
|---|---|
| **Refused, no time** | without the ability; with a familiar alive anywhere (`World::familiar()` is `Some`, the retinue included: "Your familiar lives yet."); with a hostile the player sees ("Not with that watching."); while a status blocks `cast` (as a study is refused); without `familiars.toml` or `allies.toml` ("Nothing comes.") |
| **The ritual** | `[familiar] call_turns` (600: an hour) turns of the action `call_familiar` (`actions.toml`, 1000 each: a turn) through `World::keep_at` as a study runs (`Spent::Calling`), **interrupted** by a creature coming into view, a message, or a blow (`Ended::Came`/`Said`: "…you break off your calling."): the time spent is spent, nothing else is lost. "You sit down in the dark and call, under your breath." at the start |
| **The end** | one `below(kinds.len())` draw on the **gameplay** stream (a player action's draw, so replays verify it), `make_ally` on `free_cell_near(player)` with `Familiar(CompanionData { name, grade: the player's level })`, the name drawn, the kind's arrival line after "Something comes out of the dark and sits at your feet." No free cell: the ritual ends with "Nothing comes." and the hour is spent (the cells about her are what she chose) |
| **The price** | the hour. **No mana, no gold, no blood, no cooldown, no count**: the owner's rules (time is the penalty; no anti-farming limits). A witch who loses a cat a floor spends an hour a floor; the dungeon's clock (hunger, the spawn tick, a wandering pack finding a sitting witch) is the cost. §15 Q5 offers a night or a blood price |
| **On the overworld** | allowed: the new familiar is made in the retinue (nothing is drawn on the map) and steps out at the next level |
| **Where** | anywhere: a familiar is called, not found; a toad comes out of a dungeon's dark as readily as a wood's |
| **Marks** | none (not a cast, not a skill) |

Why an active and not a return by itself (the familiar reappearing at the next dawn): an
active is the player's choice of when (not mid-fight, not when a floor is being cleared), and the
one abilities menu is where the owner wants every ability; an automatic return would need a
clock on `WorldState` and a rule for where it appears. §15 Q5 keeps the automatic return as the
alternative.

Why no releasing, no second familiar, no swap: the owner (#832 Q3): "this is like releasing a
pokemon? let's not… pets stay until they are killed." Call familiar is refused while one lives.

## 8. What it shares with the ranger's companion, and what is its own

The ranger design (#832) decided these for the hound; each reads one predicate, which this design
makes `AllySource::kept()` (Companion or Familiar). Where C1–C3 (#916, #918, …) land before F1,
F1 changes the predicate; where F1 lands first, it builds the shared piece as the ranger design
wrote it and C1 finds it there. Nothing is designed twice.

| Shared (the ranger design's §) | Read as | For the familiar |
|---|---|---|
| `CompanionData { name, grade }` (§4.3) | the kept beast's data, `AllySource::data()` | its name and grade |
| **Growth** (§8): the six hooks, `[growth]` in `companions.toml`, the grade re-set at `level_up` ("Tibb grows stronger.") | `data().grade` | the same table: a level 30 cat has 68 hit points and a 1d2+4 scratch |
| **Always comes along** (§7, change B): `follow_along` takes every kept beast wherever it stands, held or holding; the retinue | `kept()` | never left behind; "Tibb comes with you." / "Tibb follows you in." |
| **The collar** (§10, C3): Give in the Allies group, the neck slot, the old amulet dropped | the kept beast adjacent (`companion()` or `familiar()`; with both adjacent, the familiar: §15 Q4) | her bone amulet about the cat's neck; its magic on the cat |
| **Tending** (§9, C0): Heal treats an adjacent ally's bleeding, poison, disease | any ally (the core's rule) | nothing new |
| **Death** (§9): "X dies.", a corpse, the collar in the pile, nothing else happens to the master | `kept()` leaves a corpse (`creature_dies` returns early only for `Summoned`) | "Tibb dies."; an hour's calling for the next |
| **The story's chapter** (§5.2, C1): "The companion", `Chapter::when = { companion = true }`, `only` lines by creature id | the chapter's `when` fits a witness with **a kept beast at the start** (`Witness::companion` holds the starting beast's kind and source); a `familiar` line condition beside `companion`, so a goblin ranger's bat and a witch's bat have each their line | five `only` lines (§11.1, F3) |
| **The status bar line** (§14.2): "Snel: unhurt" | one line per kept beast, in actor id order ("Tibb: unhurt"; with a hound too, two lines) | the player sees the cat's health at a glance |
| **The Character screen's line** (§14.2): "Companion: Snel, a hound" | "Familiar: Tibb, a cat" below it | |
| **The kill credit** (§9, C0): the master's class's `ally_share`, else `[xp] ally_percent` 50 | the witch's class has no `ally_share` | a familiar's kill (rare) pays half; the hex's tithe is how it pays her (§5) |
| `World::companion()` (§4.3) | `World::familiar()` beside it, each the one ally of its source | both may exist at once: a witch with Handle animal may tame a hound (Tame checks `companion()`, not `kept()`); the familiar is never what Tame makes |

**What is the familiar's own** (this design, `world/familiar.rs`): the touch (§5.1), the
harrying (§5.2), `bring_familiar` and Call familiar (§7), `data/familiars.toml` (§9), the four
beasts and the names, the `familiar` dump (§11.3).

**What the familiar does not get**: the ranger's Pack tactics, Hunter's bond (shared sight and
buffs), and Sic 'em read `companion()` and stay the ranger's (rule 3); Malison's bite is hers,
not its; Beast-bond (+1 to hit, +1 Protection on every ally) reaches it through `ally_mods` as
the core wrote, for a witch who takes the Wayfarer pick.

## 9. The data

### 9.1 `assets/data/familiars.toml` (new; parsed by `content/familiar.rs`)

```toml
# The witch's familiar (spec §8 "Class abilities", "Allies"; docs/design/witch-familiar.md).
# The allies' rules are allies.toml, the kept beasts' growth companions.toml.
[familiar]
kinds = ["cat", "toad", "raven", "hare", "bat"]   # one drawn at the start and at a calling
touch = "evil_eye"       # the hex spell whose status its blow lays, at its master's degrees
call_turns = 600         # the calling: an hour of sitting
names = ["Tibb", "Fancie", "Dandy", "Tyffin", "Pigin", "Tittey", "Suckin", "Robbin", "Jarmara",
         "Holt", "Newes", "Elemauzer", "Pyewacket", "Greedigut", "Sathan"]

[familiar.arrives]       # said with the name: "{name}, your cat, winds about your ankles."
cat = "winds about your ankles"
toad = "squats in your shadow"
raven = "settles on your shoulder"
hare = "crouches at your heel"
bat = "hangs from your hood"
```

Checks at load: 1 to 12 known kinds, none `unique`, animated, a `caster`, a summoner, or the
player, each of level at most `[familiar] most_level` (2; the imp of §15 Q7 would be listed
apart); `touch` a `kind = "hex"` spell with a status; `call_turns` 1 to 6000; at least one name
(no name longer than a character's 20); an `arrives` line for every kind. Without the file the
Familiar entry makes nothing and says so in the log (never a panic), and Call familiar is refused.

### 9.2 `assets/data/actions.toml`

```toml
# A turn of calling a familiar (the witch's Call familiar; spec §8 "Class abilities"): the
# ritual takes familiars.toml's call_turns of these.
call_familiar = 1000
```

### 9.3 The creature files

Four files on the beasts' conventions (§6.1); `bat.toml` unchanged. Adding creature types changes
the content fingerprint and invalidates development saves (`docs/dev/saves.md`; the owner, #834:
no aliases, no shims); the PR says so.

### 9.4 The class file and the talent

`10-witch.toml`: `{ level = 1, talent = "familiar" }`; `assets/data/talents/familiar.toml` (§7.1).
`TalentEffect::Familiar` → `TalentMods::familiar: bool`; `ClassActive::CallFamiliar` had by its
holders (`Ability::Class`'s `had_by`).

## 10. Determinism, saves, replays

| | |
|---|---|
| **Streams** | the starting kind on `familiar/<race>,<gender>,<class>` (one draw); the name on `companion/name/<actor id>` (one); a calling's kind on the **gameplay** stream (one, at the ritual's end); the story's chapter on its own stream as every chapter's (skipped entirely for a witness without a kept beast: the ranger design §5.2). The touch, the harrying, and growth draw nothing |
| **Saved** | `AllySource::Familiar(CompanionData)` inside `Actor::ally`; the collar in the familiar's `equipment`; nothing on `WorldState` (`World::familiar()` is derived from the actors). A calling in progress is one action through `keep_at`: nothing is in flight between actions. Saves are pinned to the version: the new creature types and the new variant invalidate development saves, said in the PR; **no former ids, no shims** |
| **Hashes** | a run without a witch hashes as before (the touch and the harrying run only for a `Familiar` ally; `kept()` is true for no existing actor). Every **witch** run gains an actor at the start and a story chapter: no shipped scenario, replay, dump snapshot, or golden has a witch (`assets/campaigns/` names none), so none changes; `caster_tests`' witch table (`levelup:auto` to 60) gains the familiar's presence and is re-pinned |
| **Replays** | `use:call_familiar` (the existing `UseClass` arm); the orders, Give, and Heal are the core's and the ranger's |
| **Invariants** (F1's and F2's tests, `world/familiar_tests.rs`) | a witch starts with one familiar of the five kinds, named, at her grade, on her side; its hit lays Hexed on the target with **her** as the giver at her Evil eye degrees (base 1; 2 at Presence 14 with a fixture), and refreshes her Hex of weakness on it (the `wear` reset, the severity kept); a cast of Cackle lengthens the familiar's hex; a hexed creature's death pays Malison's tithe when she holds it (with #905); it sorts a hexed creature before a nearer unhexed one, and the nearest hostile with none hexed; a wild cat's (a dev `ally:cat`'s) blow hexes nothing; its blows get no Malison bite; growth, always along, Give, Heal, and the corpse hold for it as for a companion; Call familiar is refused with one alive (on the level, elsewhere, in the retinue), with a hostile in sight, without the ability, and costs nothing then; the ritual takes `call_turns` turns, is interrupted by a creature coming into view with the time spent and no familiar made, and ends with a new familiar named and placed, or "Nothing comes." with no free cell; a save round trip keeps the variant and the collar; a replay through the start, a fight with the touch, a death, a calling, a level change, and the retinue verifies; `levelup:auto` on a witch to 60 holds every entry; the load checks (an unknown kind, a non-hex `touch`, a unique) |

## 11. What the player sees

### 11.1 Messages (words, never a number)

| Moment | Text |
|---|---|
| the start | "Tibb, your cat, winds about your ankles." (the kind's `arrives` line with the name) |
| a blow | "Tibb scratches the kobold." / "The kobold claws Tibb." (the core's `End::Yours`, the name in place of "your cat", as a companion's) |
| the hex laid | nothing of its own: a creature's statuses are silent, look mode says "hexed" |
| hurt by her | "Tibb hisses." (the ranger's "Snel yelps." by kind: a `hurt` word on the kind, optional) |
| growth | "Tibb grows stronger." |
| the stairs, the retinue | "Tibb comes with you." / "Tibb follows you in." |
| the collar | "You fasten the bone amulet about Tibb's neck." |
| death | "Tibb dies." |
| Call familiar | "You sit down in the dark and call, under your breath." … "…you break off your calling." / "Something comes out of the dark and sits at your feet." then "Pigin, your toad, squats in your shadow." / "Nothing comes." |
| refused | "Your familiar lives yet." / "Not with that watching." |
| the story (F3) | one `only` line a kind in the companion chapter, in the pools' voice: cat "A cat came to your door one winter and would not be put out; it sat and watched you at your work, and you came to understand each other." / toad "A toad lived under the step of {home}, and you fed it crumbs and then better things, and it took to following you." / raven "A raven with a broken wing mended on your sill and never flew far again; it knows your voice from the dark." / hare "A hare sat in your path one dusk and did not run, and the next dusk, and the next, until it came in." / bat "A bat hung in the rafters above your bed and came down to your hand when you whispered." |

### 11.2 The status bar, look mode, the menu, the screens

- **The status bar**: the kept beast's line (the ranger design §14.2) says "Tibb: unhurt" for the
  familiar as for a hound; both, two lines in actor id order. A text snapshot
  (`screen_snapshots.rs`).
- **Look mode**: "Tibb, your cat (unhurt), wearing a bone amulet." (the core's and the ranger's
  words; a hexed creature it scratched: "hexed" as today).
- **The abilities menu**: Call familiar in the Class group with its reason when refused ("your
  familiar lives", "something is watching"); the orders in the Allies group as the core's. No
  key, no `KeyCommand`, no `Context` entry (the hard rule). Text snapshot `abilities-menu-witch`.
- **The Character screen**: "Familiar: Tibb, a cat" (the companion's line's sibling).
- **The glyph**: the core's ally mark (the faint sage underline), nothing more; no effect, no
  glow: the owner's taste rule.

### 11.3 The CLI and dumps

- `dump --layers familiar`: `Tibb, a cat (grade 12) at (12, 5): hp 30/30, order follow, wearing
  a bone amulet`; `Tibb, a cat (grade 12): with you` in the retinue; `no familiar` without one;
  JSON `familiar { name, creature, grade, hp, hp_max, order, pos, retinue, wears }` (null without
  one). The `companion` layer's twin, one function with the source as its argument.
- The core's `allies` JSON and the `actors` layer show it as any ally: `; ally of #0 (familiar,
  named Tibb, grade 12; follow)`.
- `abilities` lists `Call familiar (class): nothing to aim` with its reason; the header's `allies
  N` counts it.
- Action text: `use:call_familiar`; `ally:cat` (the dev action) makes a wild cat ally, not a
  familiar (its source is `Summoned { spell: None }` as today), so a test may tell them apart.

### 11.4 The wiki, the spec, the docs

| Where | What |
|---|---|
| `wiki/src/allies.md` | a section **"Familiars"** after "Companions": what a familiar is, the touch (the hex, the giver, that every hex of hers on the target is refreshed), the harrying, Call familiar and its hour, that it grows and comes along as a companion does, the collar; the generated table `familiars.md` (`xtask/src/wiki/familiars.rs`: kind, Old English name, level, speed, hit points, blow, eyes, footsteps, hearing) and the names list; `TERMS.md` gains "familiar", "touch" |
| `wiki/src/classes.md` | the witch's prose gains the familiar (linking allies, hexes, statuses per `TERMS.md`); the generated class table picks Familiar up |
| `wiki/src/status-effects.md` | the Hexed row notes the familiar's touch beside Cackle and Malison |
| The spec | spec §8 "Class abilities" (the witch: Familiar, Call familiar); spec/08b-creatures.md "Allies" (a familiar: kept, the touch, the calling) |
| `docs/dev/` | `familiar.md` (the topic file: the module, the data, the hooks, the invariants, the tests); `allies.md` (the third variant, `kept()`); `companion.md` (the shared rules' predicate); `progression.md`'s hook table (`familiar`); `status-effects.md` (the touch as a giver of Hexed); CLAUDE.md's action text and dump layer lines; `docs/design/README.md` (this document under "Approved, implementing") |

## 12. Talents and the coverage policy

**No new system and no new pickable talent.** The system is the merged `allies` (its pickable
Beast-bond reaches the familiar) and the hex loop's `status` and `spells` (Cackle's and
Malison's). A familiar is a sub-feature of allies and of the hex loop: by the policy it extends
its parents and adds no tag and no talent (CLAUDE.md's hard rule). Familiar is a class ability,
not a pickable; Call familiar is its active, had with it.

| Rule (#776) | Familiar (level 1) |
|---|---|
| 1 capstones | level 1; it changes every fight (a hex laid and held for her) |
| 2 fixed pool | no XP, gold, favour, or quest effect (an ally's kill credit is the core's rule, not this talent's) |
| 3 clones | no other talent or class ability lays a hex by a pet's blow, or sorts an ally's targets by the master's hexes; it shares no effect kind with Animal companion (`companion`/`tame`) or Beast-bond (`ally_mod`); the alike check sees `familiar` alone |
| 4 chains | no prerequisite |
| 5 flaws | none |
| 6 outdoors | works on every dungeon floor |
| 7 hidden marks | no mark effect |
| 8 information | it acts; it tells nothing (a hexed creature says "hexed" in look mode as today) |
| 9 shop favour | none |
| tags match hooks | `allies` (it is an ally, reads `allies.toml` and the arm), `status` (it lays and refreshes statuses), `spells` (the degrees of a spell, `hex_statuses()`) |

What is **not** minted, and why: a "second familiar" (one is the fiction; the owner's one pet), a
"familiar's bite" or "hardier familiar" (growth is where those numbers live), a "quicker calling"
(a small percentage on a niche mechanic), a "familiar's eyes" (the ranger's Hunter's bond; rule
3), a pickable Familiar for other classes (a class signature; a wizard with a cat is NetHack's,
not ours: §14).

## 13. Implementation plan (slices)

| Slice | Issue | What it touches | Depends on |
|---|---|---|---|
| **F1 The familiar** | the variant, the data, the beasts, the start, the touch, the harrying, the shared rules as `kept()`, the words, the dumps, the wiki, the spec | `world/ally.rs` (`AllySource::Familiar(CompanionData)`, `kept()`, `data()`, `id` "familiar"); `content/familiar.rs` (`data/familiars.toml`, the checks); `data/creatures/{cat,toad,raven,hare}.toml` (+ `beasts.md` rows, `beast_roster_tests`); `talents/familiar.toml`, `TalentEffect::Familiar`, `TalentMods::familiar`, `10-witch.toml`; `world/familiar.rs` (`bring_familiar` from `start_run`, `World::familiar`, `familiar_touch` from `fight.rs::resolve`, the harrying's sort key in `ally_plan`); the ranger's growth hooks, `follow_along`'s filter, Give's target, the status bar line, the Character screen's line, and the companion chapter's `when` read as `kept()` (or built as the ranger design wrote them, if F1 is first); the `familiar` dump layer and JSON; `world/familiar_tests.rs` (§10); `docs/dev/familiar.md`, `allies.md`, `companion.md`, `progression.md`, CLAUDE.md; the wiki's "Familiars" and `familiars.rs`; the spec | the ranger's C1 (#916) for `CompanionData`, growth, always along, when it lands first; else F1 builds them |
| **F2 Call familiar** | the active | `ClassActive::CallFamiliar`, `Ability::Class`'s row and reasons, `actions.toml` `call_familiar`, `world/familiar.rs::{start_calling, calling, finish_calling}` through `keep_at` (`Spent::Calling`), the refusals, the draw, the words; tests (§10); the `abilities` dump; the text snapshot `abilities-menu-witch`; the wiki's "Call familiar"; `docs/dev/familiar.md` | F1 |
| **F3 The story's lines** | the companion chapter's familiar lines | `background/08-companion.toml` gains the five `only` lines with `when = { familiar = ["cat"] }` etc.; `When::familiar` and `Witness::companion`'s source (`content/background.rs`, `world/background.rs`); the chapter's `when` fits a kept beast; the chapter's wiki table; tests (a witch is told her line; a ranger with a bat his; no draw moves for anyone else) | F1, C1's chapter (#916) |
| (**F4** the probe) | a witch-and-cat row in `balance_probe` beside the ranger's (C8): the hex held by the familiar against the monsters of her level; `docs/balance.md` | F1, Malison (#905) |

F1 adds saved state (inside the core's field and the creature list) and no action; F2 adds one
action (`use:call_familiar` on the existing `UseClass` arm). With the ranger's C1–C3 and the
wizard's Z1 (#909) in flight, the integrator's "about three at once" decides the order; F1 may
wait for C1 or carry its shared pieces, as the brief above says. Each creature-list PR says it
invalidates development saves. No golden anywhere: the familiar is a glyph a core test and a
dump can see; the ally mark's guard is the core's.

## 14. Rejected ideas

| Idea | Why not |
|---|---|
| **A familiar that holds her spells** (Pathfinder) | losing it would lock her out of her craft: the owner's "never lock out content by one failed roll"; the hex loop stands without it (#830 §11) |
| **XP or stat loss at its death** (3.5), **a grief status** | rule 1 (no XP effects); the owner (#832 Q8): "only the loss of pet" |
| **A familiar that cannot attack** (5e) | the touch needs a hit; a pet that only follows is a readout with legs |
| **A fighting familiar** (a hound by another name, Malison's bite on its blows) | the brief: not a second combat pet; the druid calls, the ranger kills, the witch marks |
| **Shared sight through the familiar** (5e's senses, 3.5's scrying; a witch's cat in the dark is the obvious one) | the ranger's Hunter's bond (25) is shared sight; a second source of the same effect kind is rule 3's spirit; the witch has Second sight at 25 (searching); if the owner wants a witch's scouting cat it is one hook already written (`shared_sight` read for `kept()`), §15 Q3 |
| **Cackle through the familiar's eyes** (its hexes held at a distance) | needs the familiar's sight computed into her perception, which is shared sight by another door; Cackle is her voice and her eyes |
| **The familiar's touch lays her last hex, or a chosen one** | a state to carry and a UI to choose; Evil eye is the one hex every witch has from 1, and the touch's work is to hold the others |
| **A hex degree from every scratch** (the touch growing the hex, as Cackle does) | a cat scratching three times a turn would hold every hex at its max from the first hit; Cackle's degree costs her a turn and a shout. Refresh is the right strength |
| **Race lists and a 5 % jackpot** (the ranger's Q4) | the kinds differ in flavour, not in worth (the hex is the same); a race's familiar is a thin fiction (a dwarf's toad?); §15 Q2 offers it |
| **The imp from level 1** (Pact of the Chain) | level 9, a poisoned 1d10 sting, hell's: not a level-1 witch's; §15 Q7 gives it to Hag's bargain |
| **The familiar as a draw at creation** (a step: "choose your familiar") | a creation step for one class; the owner chose a random start with a story line for the ranger (#832 Q4) |
| **A familiar for the wizard** (NetHack's kitten, 3.5's) | the wizard's mechanic is the scroll (#830 §6); a cat is the witch's and tells the classes apart, which was #826's whole question |
| **Return by itself at the next dawn or midnight** | a clock on `WorldState`, a rule for where it appears, no choice of when; the active is the owner's one-menu way. §15 Q5 keeps it as the alternative |
| **A mana, gold, or blood price for the calling** | the owner: time is the penalty; a blood price (a drop: Bleeding 1) is pretty and a death trap at 2 hit points. §15 Q5 |
| **A cooldown or a count on callings** | no anti-farming limits (the owner) |
| **Releasing, swapping, a second familiar** | the owner: pets stay until killed; one is the fiction |
| **A familiar left behind that waits** | the ranger's Q5: a kept beast always comes along; nothing to whistle for |
| **Tameable familiar kinds** (a ranger's cat) | nothing against it later; here they come only by calling, and a wild cat would hex no one (the touch reads the master) |
| **A `KeyCommand` for Call familiar**, a familiar screen | the hard rules: the one menu, fixed letters; the status bar line and look mode are the screen |

## 15. Open questions, each with a recommended default

Only what genuinely needs the owner; everything else above is decided with a reason.

| # | Question | Recommended default | The alternative |
|---|---|---|---|
| Q1 | **The level**: 1 (a third level-1 entry), or a non-cadence level (3 beside Hex of weakness, or 4 as the wizard's Knock)? | **1**: the signature from the first turn, as the ranger's hound; the wizard and the ranger have three at 1; a level-1 Evil eye on a cat is one degree of −2 to hit | 3: the table's first three levels are then 1, 3, 3, 6; the witch is a cat-less hexer for two levels, for no gain |
| Q2 | **The kinds**: one shared list of five (cat, toad, raven, hare, bat), equal odds, or race lists and a 5 % jackpot as the ranger's? | **one list of five**: the kinds differ in eyes, speed, and feet, never in the hex, so a jackpot has nothing to be a jackpot of; and a drow's toad is as good a fiction as a human's | `[familiar.by_race]` and a rare list (an imp, a black cat with darkvision 12) as the ranger's `[start]`: the same loader, two more tables |
| Q3 | **The familiar's eyes**: none for her (as written), or shared sight through the familiar at some level (the witch's cat in the dark is the classic)? | **none**: the ranger's Hunter's bond is shared sight (rule 3), and the witch's table has no free level; she has Second sight (25) | read the ranger's `shared_sight` hook for `kept()` on a witch of 25 (Second sight widened by a line): one hook, two classes, and the ranger's distinctness thinned |
| Q4 | **Give with both a hound and a familiar adjacent**: the familiar first, or a choice? | **the familiar first** (the witch's bone amulet is for her cat); the hound gets it when the cat is not beside her | `Aim::Stack` then a second aim (a direction): a question the one-dialog rule does not want asked |
| Q5 | **Call familiar's price**: an hour of sitting and nothing else (as written), or more? | **the hour** (`call_turns` 600): the owner's rules; an hour underground is real | a night (`rest:dusk` first: a calendar gate the game otherwise avoids); a mana price (her pool, for a hex she then cannot cast); a drop of blood (Bleeding 1: a death trap at low hit points); or no active: the familiar returns by itself at the next dawn (a `WorldState` clock) |
| Q6 | **The names**: the trial records' (Tibb, Pyewacket, Greedigut, …: Middle and early-modern English), or Old English like the companions' (Blæc, Græg, Scead, Deorc)? | **the trial names**: the one place the game's witch meets the English witch of record, and a pet name is an old woman's, not a chronicle's; #378 may rename | the companions' register throughout: safer, and the familiar sounds like a hound |
| Q7 | **Hag's bargain (60) and the imp**: from the bargain, may a calling answer with an **imp** (`imp.toml`: "the familiar of those who bargain with hell", level 9, a poisoned sting), one line of data (`[familiar] bargain_kinds = ["imp"]`, drawn with the five from level 60)? | **yes**: one line, the imp exists in no population, and the capstone's pact gains a face; at grade 60 its sting is still a small thing beside Blight, and the touch is the same evil eye | no: the bargain stays a reprieve, the imp waits for the devil roster's dungeon |
| Q8 | **The familiar beside a companion**: both at once (a witch with Handle animal tames a hound too), or one kept beast of any kind? | **both**: they are different things, each read by its own `companion()`/`familiar()`; a witch with a hound and a cat is a rare build, not a problem | one kept beast: Tame refuses with a familiar ("You have your familiar.") |

## 16. Decisions: the owner's round (2026-11-03, approving)

Recorded verbatim from PR #924, with what each changes. Where an answer departs from §15's default, the implementation issues carry the change; the sections above stand otherwise.

| # | The owner | What it means |
|---|---|---|
| Q1 | "level 1" | as proposed |
| Q2 | "race lists + jackpot" | **not the default**: `data/familiars.toml` holds 1–3 kinds per race (from cat, toad, raven, hare, bat, and the race-fitting beasts the ranger's list already has where they fit a witch) and a shared rare list drawn at the ranger's 50 ‰, as `companions.toml` does; the draw is the companion's draw on its own stream |
| Q3 | "shared sight + nose at some level" | **not the default**: the witch shares her familiar's sight and nose from **level 25**, beside Second sight, reusing the ranger's Hunter's bond reads (C5, #920: the second viewpoint and the nose's cells; buffs not shared); a granted rider, no new pickable |
| Q4 | "need to select direction if both adjacent" | **not the default**: Give (and Heal on an ally) asks for a direction when more than one kept beast is adjacent; one adjacent needs none |
| Q5 | "hour" | as proposed: the calling is an hour, nothing else |
| Q6 | "trial records" | as proposed |
| Q7 | "sure, you can put the imp in - but perhaps a little earlier? 60 capstone for a shitty imp seems weak." | the imp joins the calling's draw **from witch level 30** (`[familiar] bargain_kinds = ["imp"]`, `bargain_level = 30`), not tied to Hag's bargain; Hag's bargain stays as it is |
| Q8 | "can have both" | as proposed: a familiar and a tamed companion at once |

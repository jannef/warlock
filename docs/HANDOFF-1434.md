# Handoff #1434 (delete before the PR)

## Done (uncommitted work committed as WIP; nothing run but `cargo build -p warlock-cli` and dumps)
- Code: `DungeonDef::bottom` (content/dungeon.rs: parse, `leads_down`, `down_to`; exclusive with
  `below`/`endless`/`portal`); validation in content/mod.rs (bottom must be a hand-made level of the
  same campaign with a `<` back to the deepest floor). Level `[[shop]] creature` (content/level.rs:
  keeper creature type, default shops.toml `[keeper]`).
- Dungeon `assets/campaigns/ascmearc/data/dungeons/deopgang.toml`: Dēopgang, depth 11, danger 14
  (14→24), rooms 1–4 then caves 120x60, `bottom = "gimmwic"`, ores silver+gold only (gems ~23% of
  non-stone finds vs 9% shared table), chests 40%, deep-gnome trader shops on rooms floors.
  `gen --level deopgang:11` shows `stairs Down -> gimmwic`.
- Overworld: `D` at (18, 57), south-west corner against Grægweall, visible `dungeon_entrance`, no
  quest (owner: not quest-gated). Header comment in levels/ascmearc.toml updated.
- Town `levels/gimmwic.toml` (Gimmwīc): moot hall with Headman Peran (npc `peran`, leader for the
  later letter quest), the Glimmering temple (sanctuary room; Mother Senara `senara`: cure 60,
  mend 40, temple), Kerensa's Stones (`gem_dealer`, npc `kerensa`, also `identify` 100), Jory's
  Outfitters (`outfitter`: ingots, billets, herbs, essences, tools, food), the cutting floor (3
  anvils, braziers, workbenches; Old Tregear `tregear` teaches Jewelcraft 1500, `train_skill`).
  Loads; `dump --content-dir assets --scenario gimmwic --layers truth,actors` fine.
- Creatures (campaign-owned): deep_gnome, deep_gnome_trader, deep_gnome_headman,
  deep_gnome_priest, deep_gnome_jeweller; tag `deep-gnome`; NO `race` field and no races.toml row
  (races.toml is New run's list and needs kit/companion/familiar/name rows); "race not playable".
- Shop kinds `gem_dealer`, `outfitter` (generated = false) in data/shops.toml.
- Dialogue `data/dialogue/gimmwic.toml`: speech register = Cornish (my 'ansum, dreckly, proper
  job, emmet, wasson, 'ee); lines turn to craft. Fix: header lists "dreckly" twice. Check lines
  ≤ 72 chars (loader validates).
- Scenarios `deopgang` (deopgang:1) and `gimmwic` (2000 gold, dwarf warrior Brakka); added to the
  list in content/campaign.rs test.

## Remains
- Run fmt, clippy, `cargo test --workspace` (insta snapshots may change: scenario lists, wiki).
- Tests: core test that deopgang:11 `>` leads to gimmwic and its `<` back (model:
  world/way_out_tests.rs), content test for `bottom` validation errors, replay create/verify.
- Docs: data/dungeons.toml field reference (`bottom`), shops.toml header (level `creature`),
  docs/dev/mapgen.md / campaign.md / shops.md / npcs.md (speech rule: deep gnomes Cornish, bound
  by tag `deep-gnome`), new docs/dev/creatures/deep-gnomes.md, spec §7 (dungeons, shops), CLAUDE.md
  scenario list (`deopgang`, `gimmwic`), wiki ascmearc.md (place + names table: Dēopgang "the deep
  way", Gimmwīc "the gem dwelling"), npcs.md, dungeon-guide.md; `cargo xtask wiki`.
- Captures of the town at --scale 1 (`--scenario gimmwic`), looked at.
- Golden entries likely unaffected (overworld cell change may touch an overworld golden: check).

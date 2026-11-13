# Handoff #1427 (E1: affix capacity and the trim) — delete before the PR

## Done
- `MaterialDef::binding` (−2..=2, loader check), 21 material files, materials README.
- `magic/capacity.rs`: `capacity`, `trim`, `trimmed`; `magic/capacity_tests.rs` (design examples,
  damaged Masterwork, trim sequences, draws only when trimming, every loot path + creature gear
  within capacity, `hand_made_items_are_never_trimmed` reading asset TOML from disk). All pass.
- `magic::Quality` (worked.rs) + `Worked::improvements()`; `quality` key on uniques, `[[item]]`
  (`content/level.rs::hand_made`, trims at load from `trim/<key>` seed 0), heirloom effect
  (`TalentEffect::Heirloom.quality`, only talent-code touch).
- `loot::finished` last in make/make_as/roll*/roll_pool/heirloom/favoured; gear uses
  `loot::make_as_unfinished` and `gear::finish` after condition (arm, travel::placed_gear,
  balance probe). `gem::ornament_part` moved from world/set.rs (re-exported there).
- Data of design §4.1: 13 items + Stonebiter + heirloom = Masterwork (`fathers_sword.toml`,
  description updated). Chose Masterwork (not measured vs Marvellous max 3 yet).
- Capacity line in `describe::appraised_for` (`Appraisal::capacity`, Appraise > 0 in
  `PerceptionView::appraisal`); test in `world/appraise_tests.rs`.
- Dumps: inventory row ends with ` (holds N/C)`; actors carried list too (`Stack::holds_text`).
- Shares test `trim_shares_hold_every_find_to_its_capacity` (prints shares). Result: ~89% of
  loot bases have capacity 1; rares untrimmed only ~2.5%, ~89% of rares keep 1 affix; magic
  items ~72% untrimmed (2-affix magic → 1). Far below design §12's "a third keep 3+": flag to owner.
  The pedlar's favoured offer is now trimmed too (wares test relaxed to <= capacity): flag.
- Docs: docs/dev/enchanting.md (new), CLAUDE.md index + inventory layer, magic-items.md pointer,
  spec 08g/07a/07b; wiki magic-items.md "Capacity", materials.md binding bullet + generated column
  (xtask/src/wiki/materials.rs), affixes.md, talents.md, TERMS.md.
- Test fixes for the new names (Masterwork helm, Marvellous bow/stiletto, Fine dagger), wiki
  test rows, magic/tests.rs distribution tests now on `roll_untrimmed`.

## Remains
- Last failing test: `ui::describe_screen::tests::identified_items_list_their_affixes_and_others_do_not`
  line ~279: bow now "} Marvellous short bow" → update the assertion.
- Re-run full `cargo test --workspace` (last run: only that + 2 fixed since); screen snapshot
  `magic-inventory` passed in last run; no mapgen snapshot changed.
- fmt, clippy -D warnings, `cargo xtask wiki` and read pages, `--check-links`.
- Heirloom measurement (attacks dump of `--talents family_wealth,fathers_sword`, Masterwork vs
  Marvellous max 3) and state the choice in the PR.
- Replay create/verify + dump showing `(holds …)`; PR (Closes #1427), list goldens possibly
  affected: `ranged` (hub's Storm Song now Marvellous) and `arena` (kobold gear trimmed).
- Delete this file; disk hygiene.

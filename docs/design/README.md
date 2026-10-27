# Design documents

Designs approved by the owner, one file per topic. Each arrived as a draft PR that added only
its document. It was reviewed in the PR, and merging the PR approved it. Implementation issues
link to the section they build.

What is built is described by the spec (`specification.md`, `spec/`) and `docs/dev/`. These
documents record what was decided and why. The process is in `docs/dev/merging.md`, "Design
documents".

## Built

- `magic.md` (#392): built by #393 to #401 (unidentified scrolls, the spell data and tiers, the
  one cast path, learning, the first spell list, wands, rods, the Spellcraft talents, and the
  cross-checking pass). The spec's §8 "Spells", "Scrolls", "Wands", and "Rods" and
  `docs/dev/spells.md` describe what was built; its §14 questions were answered with the
  design's defaults (`docs/dev/spells.md`, "Choices"), and its §12 checks were run in #401
  (`docs/dev/spells.md`, "Balance"). The document itself stays as approved.

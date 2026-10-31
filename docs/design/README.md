# Design documents

Designs approved by the owner, one file per topic. Each arrived as a draft PR that added only
its document. It was reviewed in the PR, and merging the PR approved it. Implementation issues
link to the section they build.

What is built is described by the spec (`specification.md`, `spec/`) and `docs/dev/`. These
documents record what was decided and why. The process is in `docs/dev/merging.md`, "Design
documents".

## Approved, implementing

- `spell-scaling.md` (#730): spell levels 0 to 70 for tiers, a ladder of the caster's stats in
  every spell's own file, six named hooks for talents and items, caster level for learning
  only, school skills, and a Magic page. The owner's decisions are its §16; its plan is §15,
  twelve slices S0 to S11 tracked in #759 (S0 #747, S1 #748, S2 #749 built; the rest in
  progress). Where it revises `magic.md`, it wins.

## Built

- `magic.md` (#392): built by #393 to #401 (unidentified scrolls, the spell data and tiers, the
  one cast path, learning, the first spell list, wands, rods, the Spellcraft talents, and the
  cross-checking pass). The spec's §8 "Spells", "Scrolls", "Wands", and "Rods" and
  `docs/dev/spells.md` describe what was built; its §14 questions were answered with the
  design's defaults (`docs/dev/spells.md`, "Choices"), and its §12 checks were run in #401
  (`docs/dev/spells.md`, "Balance"). The document itself stays as approved.

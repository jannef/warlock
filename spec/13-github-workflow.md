# Warlock specification — 13. GitHub workflow

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

## 13. GitHub workflow

All through the `gh` CLI:
- Milestones M0–M7 as GitHub milestones; one issue per task.
- Labels: `area:core`, `area:render`, `area:input`, `area:tools`, `area:ci`, `area:docs`;
  `type:feature`, `type:bug`, `type:chore`.
- Feature issues follow `.github/ISSUE_TEMPLATE/feature.md` (What, Rules, What the player
  sees, **Talents and class abilities**, Tests, Docs). The talents section is the coverage
  policy (§8 "Talents"): a new system names the talents and class abilities it ships with, or
  why none.
- One pull request per issue, referencing it; CI required; squash merge into `main`.
- The player wiki (`wiki/`, mdBook, published at `/wiki/` on GitHub Pages with each release) is
  updated in the same PR as any change to gameplay mechanics or world facts.

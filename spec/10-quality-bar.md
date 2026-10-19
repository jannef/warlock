# Warlock specification — 10. Quality bar (Definition of Done)

Part of the specification, the source of truth for what to build; the index is
[`specification.md`](../specification.md). Update it together with the code and the
developer notes (CLAUDE.md, `docs/dev/`).

## 10. Quality bar (Definition of Done)

- CI (Linux) is green: `cargo fmt`, `clippy -D warnings`, tests, headless screenshots, a
  `wasm32` compile check, `cargo-deny`. macOS is verified locally before every PR (development
  happens there); macOS does not run in CI.
- Core: unit tests; property tests for line-of-sight symmetry and for sight and light never passing
  through walls; snapshot tests of ASCII dumps; replay determinism tests.
- Renderer: golden screenshots per showcase room, compared on Linux CI with software Vulkan.
  Reference images are updated deliberately, never automatically.
- No panics on player paths; errors are shown to the player gracefully; `tracing` logs go to a file
  in release builds.
- Every milestone ends with a downloadable, playable build for each OS, published by the tagged
  release workflow (Linux, Windows, macOS).
- Performance budgets (the frame budget is not yet measured on an integrated GPU, #74): 60 fps at 1440p on an
  integrated GPU; turn processing ≤ 5 ms in the hub and ≤ 16 ms in the vast level.
- The spec and CLAUDE.md are updated together with behavior changes.

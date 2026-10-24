# Managed Agents: cloud agents for Warlock

Claude Managed Agents run in a cloud sandbox and are billed to the API usage credits, not the
subscription. Each one does a job that a local subagent would otherwise do, with the same brief and
the same stopping point, so choosing the cloud never adds a step: it only moves where the work
runs and which quota pays for it.

| Agent | Model | Replaces | Launcher |
|---|---|---|---|
| `finisher/agent.md` | Sonnet 5.5 | the local Sonnet finisher (`local/finisher.md`): CI fixes, golden review, the one merge of main, light re-verify, push | `finish.sh <PR> <BRANCH> [GOLDEN_RUN] [NOTES]` |
| `engineer/agent.md` | Opus 5.5 (Fable 5.1 with `MODEL=claude-fable-5-1`) | the local Opus feature agent (`local/feature.md`), the integrator's escalation helper, a design-doc agent, issue filing | `engineer.sh build\|fix\|design\|issues ...` |

Both stop at a pushed branch or an open PR and never merge; the integrator merges as usual.

## Choosing local or cloud (the integrator decides)

For each job, pick **one** place to run it; never both, and never a cloud run "to double-check"
a local one.

- **Subscription quota comfortable, API credits low:** local subagents, as before.
- **Subscription quota tight** (near the weekly limit, the quota-hold pattern): send new jobs to
  the cloud instead of pausing them. Jobs already running locally stay local.
- **Otherwise** prefer the cloud for jobs that run long without needing the integrator's context:
  finishers, `build` from well-specified issues, `fix`, `design`, `issues`.
- **Stay local** when the job needs macOS (a platform bug, the app bundle), the owner's machine,
  or an agent's existing worktree (the cloud starts from what is pushed).
- Fable (`MODEL=claude-fable-5-1`) costs 2.5x Opus at list price: only for what the local rules
  would give Fable (foundations, design docs, a `fix` Opus already failed).

## Jobs

    ops/agents/finish.sh 224 wp-218-names "" "merge main now; #260 and #258 landed"
    ops/agents/engineer.sh build  wp-237-h1 "#237" "after #236 merged"
    ops/agents/engineer.sh fix    wp-237-h1 284 "clippy: ... (error text)" "finisher re-ran tests"
    MODEL=claude-fable-5-1 ops/agents/engineer.sh design design-resources resources "issue #225"
    ops/agents/engineer.sh issues docs/design/resources.md M9

- `build`: issues → an open PR, verified on Linux (fmt, clippy, tests, wiki, replays, captures,
  and `cargo xtask golden`, which is meaningful here: the sandbox has the same lavapipe as CI). It
  triggers the golden update run when captures change and reports the run id; a finisher reviews
  it later, exactly as for a local feature agent.
- `fix`: the escalation path (memory: integrator-escalates-automatically). Pass the error and what
  was tried. Opus first; `MODEL=claude-fable-5-1` if Opus fails.
- `design`: a draft PR adding only `docs/design/<topic>.md` (or a revision of one after review).
- `issues`: splits a merged design into issues.

Each launcher prints the session id. Check on it with `ant beta:sessions retrieve --session-id <id>`
(status `idle` with `stop_reason` `end_turn` = done; read the last agent message with `ant
beta:sessions:events list --session-id <id>`). Don't stream or poll in a loop; look when the
PR or branch should be there.

## Budgets

`ops/agents/spend.sh [YYYY-MM-DD]` lists every session since that date (default: the 1st of the
month) with its list-price cost and the total; check it before choosing the cloud for a big job.
The monthly credit allowance itself is not visible through the API: ask the owner if unsure.

Per-session caps at list price (`BUDGET_CENTS` overrides): finisher $5, build $20, fix $10,
design $10, issues $3. At the cap the session pauses (`stop_reason: budget_reached`); raise it
with a session update to resume, or let a fresh job pick up the pushed branch. Session time costs
$0.08/hour on top of tokens.

## Credentials

- **Golden update runs:** the cloud token gets HTTP 403 on `gh workflow run` (seen 2026-10-24), so
  cloud agents report that a branch needs `update_golden=true` and the integrator dispatches it
  (`gh workflow run ci.yml --ref <branch> -f update_golden=true`); the finisher then downloads
  and reviews the artifact as usual.

- `git push` goes through Anthropic's git proxy, which uses only the session's
  `github_repository` resource token: `session.sh` reads the PAT from the Keychain (`security
  find-generic-password -a jannef -s warlock-github-pat -w`) and sends it on stdin, never on a
  command line or in a file. The repo is mounted at `/workspace/warlock` with the branch checked
  out.
- The vault token is for `gh` (API, `gh run download`) only. It lives in vault
  `vlt_011CgLo82BAUBMmoHUVRfyRA` (`GH_TOKEN`, a fine-grained PAT on this repo only), managed
  outside these files. The PAT's permissions on this repo: contents, pull requests, actions,
  and issues, read/write.

## Environment

`finisher/environment.yaml` is the one environment for both agents: the Linux CI job's packages
plus Mesa lavapipe (`mesa-vulkan-drivers`, `libvulkan1`), so captures and `cargo xtask golden`
run with `WGPU_BACKEND=vulkan`; limited networking (GitHub, crates.io, Rust hosts).

## Apply

From the repo root (this writes and reads `claude-lock.json`, commit it):

    ant apply --dry-run -v ops/agents/finisher/agent.md ops/agents/finisher/environment.yaml ops/agents/engineer/agent.md
    ant apply ops/agents/finisher/agent.md ops/agents/finisher/environment.yaml ops/agents/engineer/agent.md

Name the files; do not pass `.`. A changed prompt makes a new agent version; sessions already
running keep theirs.

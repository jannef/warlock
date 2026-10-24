#!/usr/bin/env bash
# Start an engineer session (Opus; MODEL=claude-fable-5-1 for a hard one) from the repo root:
#   ops/agents/engineer.sh build  <BRANCH> <ISSUES> [NOTES]         issues -> an open PR
#   ops/agents/engineer.sh fix    <BRANCH> <PR> <PROBLEM> [TRIED] [NOTES]
#   ops/agents/engineer.sh design <BRANCH> <TOPIC> <ASK> [NOTES]    a draft docs/design PR
#   ops/agents/engineer.sh issues <DESIGN_PATH> <MILESTONE> [NOTES] (needs Issues write on the PAT)
# BUDGET_CENTS caps the session at list price (defaults: build 2000, fix 1000, design 1000, issues 300).
set -euo pipefail
set +x
. "$(dirname "$0")/session.sh"
usage() { sed -n '2,7p' "$0" >&2; exit 2; }
[ $# -ge 3 ] || usage
JOB=$1; shift
case $JOB in
  build)
    BRANCH=$1 CHECKOUT=main BUDGET=${BUDGET_CENTS:-2000} TITLE="build $2"
    MSG="JOB: BUILD
ISSUES: $2
BRANCH: $1
NOTES: ${3:-none}" ;;
  fix)
    [ $# -ge 3 ] || usage
    BRANCH=$1 CHECKOUT=$1 BUDGET=${BUDGET_CENTS:-1000} TITLE="fix PR $2"
    MSG="JOB: FIX
PR: $2
BRANCH: $1
PROBLEM: $3
TRIED: ${4:-nothing}
NOTES: ${5:-none}" ;;
  design)
    [ $# -ge 3 ] || usage
    BRANCH=$1 CHECKOUT=main BUDGET=${BUDGET_CENTS:-1000} TITLE="design $2"
    MSG="JOB: DESIGN
TOPIC: $2
BRANCH: $1
ASK: $3
NOTES: ${4:-none}" ;;
  issues)
    CHECKOUT=main BUDGET=${BUDGET_CENTS:-300} TITLE="issues from $1"
    MSG="JOB: ISSUES
DESIGN: $1
MILESTONE: $2
NOTES: ${3:-none}" ;;
  *) usage ;;
esac
start_session ./ops/agents/engineer/agent.md "$TITLE" "$CHECKOUT" "$BUDGET" "$MSG" "${MODEL:-}"

#!/usr/bin/env bash
# Start a finisher session for a PR: ops/agents/finish.sh <PR> <BRANCH> [GOLDEN_RUN] [NOTES]
# Run from the repo root. Overrides for a smoke test: KICKOFF="message" (replaces the finisher job),
# CHECKOUT=main. BUDGET_CENTS (default 500) caps the session at list price.
set -euo pipefail
set +x
[ $# -ge 2 ] || { echo "usage: $0 <PR> <BRANCH> [GOLDEN_RUN] [NOTES]" >&2; exit 2; }
. "$(dirname "$0")/session.sh"
PR=$1 BRANCH=$2 GOLDEN=${3:-} NOTES=${4:-"do not merge main"}
MSG=${KICKOFF:-"PR: $PR
BRANCH: $BRANCH
GOLDEN_RUN: ${GOLDEN:-none}
NOTES: $NOTES"}
start_session ./ops/agents/finisher/agent.md "finish PR $PR" "${CHECKOUT:-$BRANCH}" \
  "${BUDGET_CENTS:-500}" "$MSG"

#!/usr/bin/env bash
# List-price spend of this workspace's cloud sessions since a date (default: the 1st of this month):
#   ops/agents/spend.sh [YYYY-MM-DD]
# Prints each session (title, agent model, status, cost) and the total, in USD. List price, so the
# billed amount may be lower; the monthly credit allowance is not visible through the API.
set -euo pipefail
SINCE=${1:-$(date -u +%Y-%m-01)}
ant beta:sessions list --created-at-gte "${SINCE}T00:00:00Z" --include-archived --max-items -1 \
    --format jsonl |
  jq -rs '
    (map({t: .title, m: .agent.model.id, s: .status,
          c: ((.usage.list_cost.amount // "0") | tonumber)})) as $r
    | ($r[] | "\(.c / 100 | tostring | .[0:6])\t\(.s)\t\(.m)\t\(.t)"),
      "total USD \(($r | map(.c) | add // 0) / 100)"'

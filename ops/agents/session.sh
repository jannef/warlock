# Sourced by finish.sh and engineer.sh: start_session AGENT_FILE TITLE CHECKOUT BUDGET_CENTS MESSAGE [MODEL]
# prints the new session's id. Run from the repo root (claude-lock.json holds the agent and
# environment ids). The git token comes from the macOS Keychain (never echoed, never on a command
# line or in a file); it goes to the API on stdin as the github_repository resource's
# authorization_token. MODEL (optional) overrides the agent's model for this session only.
VAULT=vlt_011CgLo82BAUBMmoHUVRfyRA   # warlock-github (GH_TOKEN for gh), managed outside these files
LOCK=claude-lock.json
ENV_FILE=./ops/agents/finisher/environment.yaml   # one environment for every agent

start_session() {
  set +x   # never trace: the token is in a variable
  local agent_file=$1 title=$2 checkout=$3 budget=$4 msg=$5 model=${6:-}
  local agent_id env_id token sid
  agent_id=$(jq -r --arg f "$agent_file" '.resources[$f].id' "$LOCK")
  env_id=$(jq -r --arg f "$ENV_FILE" '.resources[$f].id' "$LOCK")
  [ "$agent_id" != null ] || { echo "no $agent_file in $LOCK (run ant apply)" >&2; return 1; }
  token=$(security find-generic-password -a jannef -s warlock-github-pat -w)
  [ -n "$token" ] || { echo "no token in the Keychain (warlock-github-pat)" >&2; return 1; }
  # jq builds the body (the token via its environment, not an argument); ant reads it from stdin.
  sid=$(BODY_TOKEN=$token jq -n --arg msg "$msg" --arg co "$checkout" --arg agent "$agent_id" \
      --arg model "$model" '{
      agent: (if $model == "" then $agent
              else {type: "agent_with_overrides", id: $agent, model: {id: $model, effort: "high"}} end),
      resources: [{type: "github_repository", url: "https://github.com/jannef/warlock",
                   authorization_token: env.BODY_TOKEN, checkout: {type: "branch", name: $co}}],
      initial_events: [{type: "user.message", content: [{type: "text", text: $msg}]}]
    }' | ant beta:sessions create \
    --environment-id "$env_id" \
    --vault-id "$VAULT" \
    --budget "{type: limit, max_list_cost: {amount: \"$budget\", currency: USD}}" \
    --title "$title" \
    --transform id -r)
  unset token
  echo "session $sid"
  echo "check: ant beta:sessions retrieve --session-id $sid"
}

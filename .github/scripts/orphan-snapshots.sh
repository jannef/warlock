#!/usr/bin/env bash
# Fails on a committed insta snapshot file that no test read (#1537): what
# `cargo insta test --unreferenced=reject` checks, without cargo-insta. The test run lists every
# snapshot file it reads in the file named by INSTA_SNAPSHOT_REFERENCES_FILE (insta does that
# itself); pass that file. Run from the repository root, after `cargo test --workspace`.
set -euo pipefail

refs=${1:?usage: orphan-snapshots.sh REFERENCES_FILE}
if [ ! -s "$refs" ]; then
  echo "::error::No snapshot references were recorded in $refs: was INSTA_SNAPSHOT_REFERENCES_FILE set for the test run?"
  exit 1
fi

root=$(pwd -P)
# insta records absolute paths; make them relative to the repository like git's.
referenced=$(while IFS= read -r path; do
  dir=$(cd "$(dirname "$path")" && pwd -P)
  echo "${dir#"$root"/}/$(basename "$path")"
done < "$refs" | sort -u)
committed=$(git ls-files '*.snap' | sort)

orphans=$(comm -23 <(echo "$committed") <(echo "$referenced"))
if [ -n "$orphans" ]; then
  echo "::error::Snapshot files that no test reads (a renamed or removed test left them behind; delete them):"
  echo "$orphans"
  exit 1
fi
echo "Every one of the $(echo "$committed" | wc -l | tr -d ' ') snapshot files is read by a test."

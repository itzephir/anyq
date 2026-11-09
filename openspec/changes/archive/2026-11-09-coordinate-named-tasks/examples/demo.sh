#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
anyq_bin="${ANYQ_BIN:-$root/target/debug/anyq}"
demo_dir="$(mktemp -d)"
trap 'rm -rf "$demo_dir"' EXIT
export ANYQ_DB="$demo_dir/queue.db"

"$anyq_bin" topic create large-builds
"$anyq_bin" topic create emulator-use
"$anyq_bin" group create large-builds builders
"$anyq_bin" group create emulator-use emulator-tests
"$anyq_bin" publish large-builds --payload '{"product":"demo","revision":"abc123"}' --key demo-abc123
"$anyq_bin" worker large-builds --group builders --consumer bash-build --once \
  --next-topic emulator-use --next-payload '{"artifact":"demo.apk","test":"smoke"}' \
  -- sh -c 'cat; printf "\nSimulated build succeeded\n"'
"$anyq_bin" worker emulator-use --group emulator-tests --consumer emulator-1 --once \
  -- sh -c 'cat; printf "\nSimulated emulator test succeeded\n"'
"$anyq_bin" jobs list

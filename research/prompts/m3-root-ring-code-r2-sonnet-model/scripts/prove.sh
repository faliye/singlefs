#!/usr/bin/env bash
# 用法：prove.sh <crate> <行号…>：按 work/tree/crates/mutations.tsv 的行号取变异名，交 prove-red.sh
set -uo pipefail
crate="$1"; shift
names=()
for n in "$@"; do
  name="$(awk -F'\t' -v n="$n" 'NR==n{print $1}' /tmp/claude-1000/m3-root-ring-code-r2-sonnet/work/tree/crates/mutations.tsv)"
  names+=("$name")
done
cd /home/fy5090/code/singlefs
PROVE_RED_LOG_DIRECTORY=/tmp/claude-1000/m3-root-ring-code-r2-sonnet/prove-logs \
CARGO_TARGET_DIR=/tmp/claude-1000/m3-root-ring-code-r2-sonnet/work/tree/target \
nice -n 19 bash research/scripts/capped.sh 15 bash research/scripts/prove-red.sh --copy /tmp/claude-1000/m3-root-ring-code-r2-sonnet/work/tree --memory 8G "$crate" "${names[@]}"

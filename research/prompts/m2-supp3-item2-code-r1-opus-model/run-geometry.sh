#!/usr/bin/env bash
# Usage: run-geometry.sh <id> : big tier (env-driven, attack switches off) over a fixed list of geometries on copies/<id> (already built by run-mutant.sh).
# One progress line per geometry.
set -u
D=/tmp/claude-1000/m2-supp3-item2-code-r1-opus
id=$1; L=$D/logs/$id; BIN=$(cat "$L/bin.txt")
# name weights first_seed seeds ops
while read -r g w f n o; do
  SINGLEFS_RANDOM_HISTORY_WEIGHTS=$w SINGLEFS_RANDOM_HISTORY_FIRST_SEED=$f SINGLEFS_RANDOM_HISTORY_SEEDS=$n SINGLEFS_RANDOM_HISTORY_OPERATIONS=$o SINGLEFS_RANDOM_HISTORY_SHRINK=none SINGLEFS_RANDOM_HISTORY_THREADS=16 \
    nice -n 19 "$BIN" --ignored --exact random_histories_large_tier_seeds_and_length_from_the_environment --nocapture > "$L/geo-$g.log" 2>&1
  echo "$id $g ($w [$f,$((f+n))) x$o) exit=$? $(grep -m1 -E '^历史 ' "$L/geo-$g.log")"
  grep -E '^新发现 ' "$L/geo-$g.log" | sed -E 's/同签名的种子 \[([^]]*)\]/n=\1/' | awk -v p="    " '{print p $0}' | cut -c1-220
done <<'GEO'
G1 broad 96 96 30
G2 broad 0 96 20
G3 broad 0 96 40
G4 reuse 0 96 30
G5 rollback 0 96 30
G6 rollback 48 48 30
G7 rollback 0 48 20
G8 reuse 0 48 20
G9 broad 0 96 12
GEO

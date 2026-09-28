#!/usr/bin/env bash
# 攻方副本批 2：每个候选各跑一遍 E9、E10。
set -u
cd /tmp/claude-1000/abandoned-floor-r1/opus/repo
export CARGO_TARGET_DIR=/tmp/claude-1000/abandoned-floor-r1/opus/target
runs=/tmp/claude-1000/abandoned-floor-r1/opus/runs
rm -f "$runs"/b2-*.rc
index=0
for spec in today:not-counted jia:not-counted yi_adjust:not-counted yi_refuse:not-counted bing:not-counted bing_all:not-counted ding_isolated:not-counted ding_isolated_released:not-counted ding_defer_below_floor:not-counted today:count jia:count; do
  index=$((index + 1))
  candidate=${spec%%:*}
  floor_mode=${spec##*:}
  started=$(date +%s)
  AF_CANDIDATE=$candidate AF_ABANDONED_F=$floor_mode nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/capped.sh 16 \
    "$CARGO_TARGET_DIR"/release/deps/abandoned_floor_candidates-"$BINARY_HASH" --nocapture --test-threads 16 e9_ e10_ \
    > "$runs/b2-$index-$candidate-$floor_mode.log" 2>&1
  echo "$?" > "$runs/b2-$index.rc"
  echo "b1 件 $index $candidate/$floor_mode 退出码 $(cat "$runs/b2-$index.rc") 用时 $(( $(date +%s) - started )) 秒；下一件预计同量" >> /tmp/claude-1000/abandoned-floor-r1/opus/progress.md
done

#!/usr/bin/env bash
# 复现 W1（真单元）：watch 观察到在跑的 opusr2-w1-<pid>，它的登记目录被删，单元照跑 ⇒ watch 退 0、报「都结束了」「单元已不在」。
# 用法：bash repro-w1-watch-record-deleted.sh <仓根>
set -u
repo="$1"; here="$(cd "$(dirname "$0")" && pwd)"; source "$here/repro-lib.sh"
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr2-rw1.XXXXXX")"; script="$(opusr2_tree "$repo" "$work" "$repo/research/scripts/detached-run.sh")"
label="opusr2-w1-$$"; unit="singlefs-detached-$label.service"; export DETACHED_RUN_STATE_DIRECTORY="$work/state"
bash "$script" start --label "$label" -- sleep 300 > "$work/start.out" 2>&1; echo "start 退 $?"
timeout 60 bash "$script" watch --every 1 > "$work/watch.out" 2>&1 &
w=$!
for _ in $(seq 100); do grep -q "还在跑  $label" "$work/watch.out" 2>/dev/null && break; sleep 0.1; done
rm -rf -- "$DETACHED_RUN_STATE_DIRECTORY/$label"
wait "$w"; echo "watch 退 $?；此刻单元 ActiveState=$(systemctl --user show -p ActiveState --value "$unit")"
grep -E "登记里没有  单元 $unit|$label 没留退出码|都结束了" "$work/watch.out" | cut -c1-160
opusr2_stop_unit "$unit"; opusr2_residue; rm -rf -- "$work"

#!/usr/bin/env bash
# 模型 V4-g 的后果（只跑 watch，替身 systemctl）：watch 开跑时用户级 systemd 连不上（查不到），这期间那条跑完、写了 exit；
#   连上之后 watch 退出。对的实现报「opusr2g 退出码 0」；g 那种错实现（不记查不到的）报「观察到 0 条」。
# 用法：bash model-v4g-watch-unknown-then-ended.sh <detached-run.sh 的路径>
set -u
script="$1"; here="$(cd "$(dirname "$0")" && pwd)"; source "$here/shim-lib.sh"
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr2-v4g.XXXXXX")"; trap 'rm -rf -- "$work"' EXIT
opusr2_shim "$work/shim"; state="$work/state"; mkdir -p "$state"
opusr2_record "$state" opusr2g; touch "$work/shim/units/bus-down"
PATH="$work/shim/bin:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state" timeout 30 bash "$script" watch --every 1 > "$work/w.out" 2>&1 &
w=$!
for _ in $(seq 50); do grep -q '查不到  opusr2g' "$work/w.out" 2>/dev/null && break; sleep 0.1; done
echo 0 > "$state/opusr2g/exit"; date +%s > "$state/opusr2g/ended"; rm -f "$work/shim/units/bus-down"
wait "$w"; echo "watch 退 $?"
grep -E '^  · opusr2g|观察到' "$work/w.out"

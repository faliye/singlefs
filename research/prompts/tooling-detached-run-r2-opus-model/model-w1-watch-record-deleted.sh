#!/usr/bin/env bash
# 模型 W1（V2 ②）：watch 观察到一条在跑的，它的登记目录随后被删、单元还在跑 ⇒ watch 退 0、报「都结束了」。
# 不起单元：替身 systemctl（shim-lib.sh）说单元 active、list-units 也列它。只跑 detached-run.sh 的 watch（只读子命令）。
# 用法：bash model-w1-watch-record-deleted.sh <detached-run.sh 的路径>
set -u
script="$1"; here="$(cd "$(dirname "$0")" && pwd)"; source "$here/shim-lib.sh"
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr2-w1.XXXXXX")"; trap 'rm -rf -- "$work"' EXIT
opusr2_shim "$work/shim"; state="$work/state"; mkdir -p "$state"
opusr2_record "$state" opusr2w1
touch "$work/shim/units/active-singlefs-detached-opusr2w1.service"
PATH="$work/shim/bin:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state" timeout 30 bash "$script" watch --every 1 > "$work/watch.out" 2>&1 &
w=$!
for _ in $(seq 50); do grep -q '还在跑  opusr2w1' "$work/watch.out" 2>/dev/null && break; sleep 0.1; done
rm -rf -- "$state/opusr2w1"          # 登记被删；单元（替身）照旧 active
wait "$w"; rc=$?
echo "watch 退出码 $rc；替身单元仍 active：$(PATH="$work/shim/bin:$PATH" systemctl --user show -p ActiveState --value singlefs-detached-opusr2w1.service)"
grep -E '登记里没有|opusr2w1 没留退出码|都结束了' "$work/watch.out"

#!/usr/bin/env bash
# 复现：用户级 systemd 的查询中途失败（systemctl --user show 退 1、什么都不打）时，detached-run.sh 的 list / watch 把还在跑的单元报成「没留退出码」，
# watch 因此退 0 并宣布「观察到的 1 条都结束了」。不起任何单元：systemctl 用同目录的 systemctl-shim 顶替，登记目录是临时的、只放一条手造的登记。
#   用法：bash repro-bus-flake.sh <detached-run.sh 的路径>（它要在 research/scripts/ 下，preflight 从 ../../.claude/scripts 找）
# 只读：不 --selftest、不 start、不 stop。
set -uo pipefail
script="$(cd "$(dirname "${1:?要 detached-run.sh 的路径}")" && pwd)/$(basename "$1")"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
work="$(mktemp -d "${TMPDIR:-/tmp}/detached-run-bus-flake.XXXXXX")"
mkdir -p "$work/shim" "$work/state/fake"
cp "$here/systemctl-shim" "$work/shim/systemctl"; chmod +x "$work/shim/systemctl"
cd "$work/state/fake" || exit 1
echo singlefs-detached-fake.service > unit; echo 'sleep 999' > command; date +%s > started; echo 2026-10-01 > started-date
echo "$work/state/fake/log" > log; echo 1111 > pid; : > log
cd "$work" || exit 1
SHIM_COUNT="$work/count" PATH="$work/shim:$PATH" DETACHED_RUN_STATE_DIRECTORY="$work/state" timeout 30 bash "$script" watch --every 1 > "$work/watch.out" 2>&1
echo "watch 退出码 $?"
grep -n '第 .* 次\|还在跑、没结束\|观察到\|没留退出码' "$work/watch.out"
echo "全文：$work/watch.out（用完自己删 $work）"

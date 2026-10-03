#!/usr/bin/env bash
# 模型 F4（V2 ③）：不跑 --selftest，自己在临时登记目录里放 .selftest-token、把同一个值放进 DETACHED_RUN_SELFTEST_TOKEN，
# 弄坏开关就在自证之外生效。只跑 list（只读）；替身 systemctl 让 show 失败（用户级 systemd 连不上），看 query-failure-as-gone 生不生效。
# 用法：bash model-f4-forged-token.sh <detached-run.sh 的路径>
set -u
script="$1"; here="$(cd "$(dirname "$0")" && pwd)"; source "$here/shim-lib.sh"
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr2-f4.XXXXXX")"; trap 'rm -rf -- "$work"' EXIT
opusr2_shim "$work/shim"; state="$work/state"; mkdir -p "$state"
opusr2_record "$state" opusr2f4
touch "$work/shim/units/bus-down"
run() { PATH="$work/shim/bin:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state" "$@" bash "$script" list 2>&1 | grep -E 'opusr2f4|✗|✓' | cut -c1-110; echo "   退 ${PIPESTATUS[0]}"; }
echo "== 1 不设开关（对照）"; run env
echo "== 2 设开关、不带令牌（F4 的拒）"; run env DETACHED_RUN_BREAK=query-failure-as-gone
echo "== 3 设开关、自己造令牌"; echo forged > "$state/.selftest-token"
run env DETACHED_RUN_BREAK=query-failure-as-gone DETACHED_RUN_SELFTEST_TOKEN=forged

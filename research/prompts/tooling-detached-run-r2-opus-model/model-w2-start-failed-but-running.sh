#!/usr/bin/env bash
# 模型 W2（V2 ②）：登记里有 start-failed（systemd-run 退了非 0），而单元其实起来了、在跑
#   （systemd-run 的 D-Bus 调用超时而 systemd 照样起了单元，是推的触发；真起单元的复现见 repro-w2-start-failed-but-running.sh）。
#   list 先看 start-failed、不问单元状态 ⇒ 列成「没起成…命令一行都没跑」；单元在登记里，也不进「登记里没有」；watch 一轮就退 0。
# 用法：bash model-w2-start-failed-but-running.sh <detached-run.sh 的路径>
set -u
script="$1"; here="$(cd "$(dirname "$0")" && pwd)"; source "$here/shim-lib.sh"
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr2-w2.XXXXXX")"; trap 'rm -rf -- "$work"' EXIT
opusr2_shim "$work/shim"; state="$work/state"; mkdir -p "$state"
opusr2_record "$state" opusr2w2; echo "2026-10-01 systemd-run 退 1" > "$state/opusr2w2/start-failed"
touch "$work/shim/units/active-singlefs-detached-opusr2w2.service"
PATH="$work/shim/bin:$PATH" DETACHED_RUN_STATE_DIRECTORY="$state" timeout 30 bash "$script" watch --every 1 > "$work/w.out" 2>&1; rc=$?
echo "watch 退 $rc；替身单元：$(PATH="$work/shim/bin:$PATH" systemctl --user show -p ActiveState --value singlefs-detached-opusr2w2.service)"
grep -E '没起成  |登记里没有  |观察到' "$work/w.out" | cut -c1-140

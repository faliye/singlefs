#!/usr/bin/env bash
# 复现 V3 乙（真单元）：--log 指到写不进的目录里 ⇒ start 退 0、打「✓ 起了…（已经跑完）」，命令一行都没跑（marker 不在），list 报「已结束 退出码 1」。
# 用法：bash repro-v3-log-unwritable.sh <仓根>
set -u
repo="$1"; here="$(cd "$(dirname "$0")" && pwd)"; source "$here/repro-lib.sh"
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr2-rv3.XXXXXX")"; script="$(opusr2_tree "$repo" "$work" "$repo/research/scripts/detached-run.sh")"
label="opusr2-log-$$"; unit="singlefs-detached-$label.service"; export DETACHED_RUN_STATE_DIRECTORY="$work/state"
mkdir -p "$work/ro"; chmod 555 "$work/ro"
bash "$script" start --label "$label" --log "$work/ro/run.log" -- touch "$work/marker" > "$work/start.out" 2>&1; echo "start 退 $?"
grep -E '✓ 起了|Permission denied' "$work/start.out" | cut -c1-160
sleep 1; echo "marker：$([[ -e "$work/marker" ]] && echo 在 || echo 不在)"
bash "$script" list 2>&1 | grep -E "已结束  $label" | cut -c1-120
opusr2_stop_unit "$unit"; opusr2_residue; chmod 755 "$work/ro"; rm -rf -- "$work"

#!/usr/bin/env bash
# 复现 B1（真单元，只跑 bash -c 'echo …'）：start -- 之后第一个词写 --force，preflight 把它摘掉，单元里跑的是后面那条命令；
#   钩子的 detached_run_start_command 不摘，把 --force 当命令词（重型闸与 ③ 看不到后面那条，见 hook-probes-v2.txt 的 B1–B3、C1–C2）。
# 用法：bash repro-b1-force-inner.sh <仓根>
set -u
repo="$1"; here="$(cd "$(dirname "$0")" && pwd)"; source "$here/repro-lib.sh"
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr2-rb1.XXXXXX")"; script="$(opusr2_tree "$repo" "$work" "$repo/research/scripts/detached-run.sh")"
label="opusr2-force-$$"; unit="singlefs-detached-$label.service"; export DETACHED_RUN_STATE_DIRECTORY="$work/state"
bash "$script" start --label "$label" -- --force bash -c 'echo ran > "$0"' "$work/marker" > "$work/start.out" 2>&1; echo "start 退 $?"
sleep 1; echo "marker：$(cat "$work/marker" 2>/dev/null || echo 不在)；登记里的命令：$(head -n 1 "$DETACHED_RUN_STATE_DIRECTORY/$label/command")"
opusr2_stop_unit "$unit"; opusr2_residue; rm -rf -- "$work"

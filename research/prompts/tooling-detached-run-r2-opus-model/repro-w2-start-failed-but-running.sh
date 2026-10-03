#!/usr/bin/env bash
# 复现 W2（真单元）：替身 systemd-run 先转给真的（单元起来了）、再退 1（模拟 D-Bus 调用超时而单元照起）
#   ⇒ start 退 1、登记写 start-failed；list 列「没起成…命令一行都没跑」；watch 一轮退 0「观察到 0 条」；单元其实 active。
# 用法：bash repro-w2-start-failed-but-running.sh <仓根>
set -u
repo="$1"; here="$(cd "$(dirname "$0")" && pwd)"; source "$here/repro-lib.sh"
work="$(mktemp -d "${TMPDIR:-/tmp}/opusr2-rw2.XXXXXX")"; script="$(opusr2_tree "$repo" "$work" "$repo/research/scripts/detached-run.sh")"
label="opusr2-w2-$$"; unit="singlefs-detached-$label.service"; export DETACHED_RUN_STATE_DIRECTORY="$work/state"
mkdir -p "$work/shim"; real="$(command -v systemd-run)"
printf '#!/usr/bin/env bash\n%q "$@"\necho "替身：真的 systemd-run 退 $?，这里报 1" >&2\nexit 1\n' "$real" > "$work/shim/systemd-run"; chmod +x "$work/shim/systemd-run"
PATH="$work/shim:$PATH" bash "$script" start --label "$label" -- sleep 300 > "$work/start.out" 2>&1; echo "start 退 $?"
sleep 1
echo "单元 ActiveState=$(systemctl --user show -p ActiveState --value "$unit")"
bash "$script" list 2>&1 | grep -E "没起成  $label|登记里没有  单元 $unit" | cut -c1-160
timeout 20 bash "$script" watch --every 1 > "$work/watch.out" 2>&1; echo "watch 退 $?"; grep -E '观察到' "$work/watch.out"
opusr2_stop_unit "$unit"; opusr2_residue; rm -rf -- "$work"

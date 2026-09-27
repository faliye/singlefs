#!/usr/bin/env bash
# admission: always 演示的是 run-with-memory-cap.sh 今天的 scope 属性（OOMPolicy=stop），随时可能改
# run-condition: command systemctl
# defs-m2-closeout-r2 云端攻方 F9：整条命令经包装时，里面一步撞了上限，systemd 停的是整个 scope——后面的步骤一步都不跑。
# 在自己开的临时 slice（总上限 600M）里：外层上限 150M，跑「第 1 步正常、第 2 步分配 400 MiB、第 3 步正常」三步的 bash，看第 3 步（紧接着）与第 4 步（隔 2 秒）打没打出来。
# 对照：同一条三步不经包装时，第 2 步被 ulimit -v 拒掉之后第 3 步照跑（模拟门禁不经包装时一个阶段失败、后面的阶段照跑）。
# 用法（仓根下）：bash research/prompts/defs-m2-closeout-r2-opus-model/oom-demo.sh ；跑完停掉并 revert 自己的 slice，删临时目录。
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
wrapper="$root/research/scripts/run-with-memory-cap.sh"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/oom-demo.XXXXXX")"
slice="singlefs_r2opus_oom_$$.slice"
export RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=600M
export RUN_WITH_MEMORY_CAP_STATE_DIR="$scratch/state" RUN_WITH_MEMORY_CAP_PEAKS="$scratch/peaks.tsv"
steps='echo "  step1 ran"; python3 -c "a = bytearray(400 * 1024 * 1024); a[::4096] = b\"x\" * len(a[::4096])" && echo "  step2 ok" || echo "  step2 exit=$?"; echo "  step3 ran"; sleep 2; echo "  step4 ran (2 s after the failing step)"'
echo "wrapped (cap 150M):"
RUN_WITH_MEMORY_CAP_KEY=oom-demo-wrapped bash "$wrapper" 150M bash -c "$steps" 2>"$scratch/wrapped.err"
echo "  wrapper_exit=$?"
echo "unwrapped (ulimit -v 300000 KiB stands in for one failing stage):"
bash -c "ulimit -v 300000; $steps" 2>/dev/null
echo "  exit=$?"
systemctl --user stop "$slice" >/dev/null 2>&1
systemctl --user revert "$slice" >/dev/null 2>&1
rm -rf -- "${scratch:?}"

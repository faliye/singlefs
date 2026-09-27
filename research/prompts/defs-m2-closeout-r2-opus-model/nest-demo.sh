#!/usr/bin/env bash
# admission: always 演示的是 run-with-memory-cap.sh 今天的排队算法，随时可能改
# run-condition: command systemctl
# defs-m2-closeout-r2 云端攻方 F9：包装里再经包装跑时，外层按「它要的量」占着账，里层要等外层让出位置。
# 在一个自己开的临时 slice（总上限 600M）里跑：外层上限 200M / 400M，里层上限 300M、只跑 true，里层排队至多等 6 秒。
# 外层、里层每次各用一个新键（RUN_WITH_MEMORY_CAP_KEY），峰值表是这一次的临时文件，于是「要的量」= 上限（包装文件头「排队」一段）；
# 第四次让外层复用第三次的键：峰值表已记下外层实测峰值，外层「要的量」变成那个峰值。跑完停掉并 revert 自己的 slice，删临时目录。
# 用法（仓根下）：bash research/prompts/defs-m2-closeout-r2-opus-model/nest-demo.sh
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
wrapper="$root/research/scripts/run-with-memory-cap.sh"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/nest-demo.XXXXXX")"
slice="singlefs_r2opus_nest_$$.slice"
export RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=600M
export RUN_WITH_MEMORY_CAP_STATE_DIR="$scratch/state" RUN_WITH_MEMORY_CAP_PEAKS="$scratch/peaks.tsv"
inner='RUN_WITH_MEMORY_CAP_KEY="$1" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=6 bash "$0" 300M true 2>"$2"; status=$?; echo "  inner_cap=300M inner_key=$1 inner_exit=$status"; grep -m1 -o "内存不够排不上[^；]*" "$2" | sed "s/^/  /"'
run_outer() { # run_outer <外层上限> <外层键> <里层键>
  RUN_WITH_MEMORY_CAP_KEY="$2" bash "$wrapper" "$1" bash -c "$inner" "$wrapper" "$3" "$scratch/inner-$3.err"
  echo "outer_cap=$1 outer_key=$2 outer_exit=$?"
}
run_outer 200M nest-outer-a nest-inner-a
run_outer 400M nest-outer-b nest-inner-b
run_outer 300M nest-outer-c nest-inner-c
run_outer 300M nest-outer-c nest-inner-d
echo "peaks recorded (MiB, cap, key):"
grep -v '^#' "$scratch/peaks.tsv" | awk -F'\t' '{printf "  %.0f\t%s\t%s\n", $1/1048576, $3, $4}'
systemctl --user stop "$slice" >/dev/null 2>&1
systemctl --user revert "$slice" >/dev/null 2>&1
rm -rf -- "${scratch:?}"

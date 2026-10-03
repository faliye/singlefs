#!/usr/bin/env bash
# 攻方 J2-c 续：--memory-cap 的 1.5 倍超过 slice 总上限时（本机约 26.7G 以上），升档那趟退几。临时 slice 总上限 256M、slice 里没有别的：
# 头一趟上限 200M 撞顶（峰值表记成 200M），升档那趟上限 300M（200M 的 1.5 倍）。用法：j2c-over-slice.sh <树根> <草稿目录>
set -uo pipefail
tree="$1"; work="$2"; wrap="$tree/research/scripts/run-with-memory-cap.sh"
rm -rf "${work:?}/j2c-over"; mkdir -p "$work/j2c-over"; cd "$work/j2c-over"
export RUN_WITH_MEMORY_CAP_SLICE=singlefs-r3attack.slice RUN_WITH_MEMORY_CAP_SLICE_TOTAL=256M RUN_WITH_MEMORY_CAP_STATE_DIR="$work/j2c-over/state" \
       RUN_WITH_MEMORY_CAP_PEAKS="$work/j2c-over/peaks.tsv" RUN_WITH_MEMORY_CAP_KEY=r3-over RUN_WITH_MEMORY_CAP_WAIT_SECONDS=30
hog='c=[]
while True:
    b=bytearray(8<<20); b[::4096]=b"\x01"*(len(b)//4096); c.append(b)'
for cap in 200M 300M; do
  bash "$wrap" "$cap" python3 -c "$hog" 2> "$cap.err"; rc=$?
  echo "上限 $cap exit=$rc"; grep -o 'run-with-memory-cap: [^（]*' "$cap.err" | head -2
  grep -P '\tr3-over' peaks.tsv | cut -f1,3,4
done

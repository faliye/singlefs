#!/bin/bash
# 复跑：rerun-all.sh <草稿目录> <段>。段：build | sb | nrb | rh | ss4 | ss5 | ss6a | ss6b（每段前台跑得完，ss5 约 3 分钟）。
# 输出进 <草稿目录>/out/，汇总用 analyze-judges.py 与 pair-writers.py（见报告开头的复跑命令）。
set -u
scratch=$1; part=$2
model=$(cd "$(dirname "$0")/.." && pwd)
run() { bash "$model/scripts/run.sh" "$scratch" "$@"; }
case "$part" in
  build) bash "$model/scripts/build.sh" "$scratch" ;;
  sb)
    # 根槽读坏（同一实例）：S = 4，实例 1 写到 L ∈ 12..20，翻坏 t ∈ L−6..L−1 那一槽；正确写者与坏写者 m4a 各一件。8 件并行。
    rm -f "$scratch"/out/bsb-*.rc; i=0
    for L in $(seq 12 20); do for t in $(seq $((L - 6)) $((L - 1))); do for w in ok m4a; do
      bw=""; [ "$w" = m4a ] && bw=m4a
      { HOLE_BAD_WRITER=$bw run "sb-l$L-t$t-$w" slot_bad 4 1048576 "$L" "$t"; echo "$?" > "$scratch/out/bsb-$i.rc"; } &
      i=$((i + 1)); if (( i % 8 == 0 )); then wait; fi
    done; done; done
    wait; echo "rc_files=$(ls "$scratch"/out/bsb-*.rc | wc -l) dispatched=$i" ;;
  nrb)
    # 最新实例写行那一条根读坏：1 MiB journal、3000 字节内容，翻坏 1 / 2 条；短环那一形（2 MiB 顺序写）翻坏 1 条、挂载之后写 3..6 次。
    rm -f "$scratch"/out/bnrb-*.rc; i=0
    for b in 1 2; do { run "nrb-l10-k3-b$b" newest_row_bad 4 1048576 10 3 "$b"; echo "$?" > "$scratch/out/bnrb-$i.rc"; } & i=$((i + 1)); done
    for k in 3 4 5 6; do { PROTO_SEQ=1 PROTO_CONTENT_BYTES=2097152 run "nrbs-l4-k$k-b1" newest_row_bad 4 1048576 4 "$k" 1; echo "$?" > "$scratch/out/bnrb-$i.rc"; } & i=$((i + 1)); done
    wait; echo "rc_files=$(ls "$scratch"/out/bnrb-*.rc | wc -l) dispatched=$i" ;;
  rh) bash "$model/scripts/rollback-sweep.sh" "$scratch" 1 ;;
  ss4) bash "$model/scripts/short-ring-sweep.sh" "$scratch" 2 4 ;;
  ss5) bash "$model/scripts/short-ring-sweep.sh" "$scratch" 3 5 ;;
  ss6a) bash "$model/scripts/short-ring-sweep.sh" "$scratch" 4 6 "4 6" ;;
  ss6b) bash "$model/scripts/short-ring-sweep.sh" "$scratch" 5 6 "8" ;;
  *) echo "不认识的段：$part" >&2; exit 2 ;;
esac

#!/usr/bin/env bash
# e171-conclusion-r3 攻方：E171 判「未触达」的行逐行打进 s4 树副本，编返回值探针、跑、还原。只写 <输出目录>。
# 用法：bash run_return_batch.sh <副本根> <行表（行号\t文件\t行名）> <起始行号> <结束行号> <输出目录> <线程数>
set -uo pipefail
MODEL=$(cd "$(dirname "$0")" && pwd)
WS=$1; TABLE=$2; FROM=$3; TO=$4; OUT=$5; THREADS=$6
REPO=/home/fy5090/code/singlefs
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='e171-conclusion-r3 攻方：同一探针换变异 core 重编重跑'
mkdir -p "$OUT"
while IFS=$'\t' read -r row file name; do
  [ "$row" -ge "$FROM" ] && [ "$row" -le "$TO" ] || continue
  t0=$(date +%s)
  changed=$(python3 "$MODEL/mutate_by_row.py" "$WS" apply "$name" 2>"$OUT/row-$row.mutate.err") || { echo -e "$row\tmutate_failed"; continue; }
  (cd "$WS" && nice -n 19 bash $REPO/research/scripts/run-with-memory-cap.sh 3G bash $REPO/research/scripts/capped.sh "$THREADS" \
     cargo test --offline --release -p singlefs-harness --test opus_r3_return_probe --no-run > "$OUT/row-$row.build.log" 2>&1)
  brc=$?
  if [ $brc -eq 0 ]; then
    exe=$(grep -o 'target/release/deps/opus_r3_return_probe-[0-9a-f]*' "$OUT/row-$row.build.log" | tail -n 1)
    OPUS_DRIVE_OUT="$OUT/row-$row.tsv" OPUS_DRIVE_LABEL="row$row" nice -n 19 bash $REPO/research/scripts/run-with-memory-cap.sh 256M "$WS/$exe" --test-threads 1 > /dev/null 2>"$OUT/row-$row.run.err"
    rrc=$?
  else
    rrc=-
  fi
  python3 "$MODEL/mutate_by_row.py" "$WS" restore "$changed"; touch "$WS/$changed"
  echo -e "$row\tbuild=$brc\trun=$rrc\tseconds=$(( $(date +%s) - t0 ))"
done < "$TABLE"

#!/bin/bash
# 用法：fuzz-batch.sh <草稿目录> <批号> <S> <起始种子> <件数> <每件种子数>：并行起 <件数> 件 appearance_fuzz（每件经 run.sh：内存包装 + capped.sh 1），
# 每件退出码写 <草稿目录>/b<批号>-<件号>.rc，不带参数的 wait 等齐之后数 .rc，对不上整批作废。
scratch=$1; batch=$2; s=$3; seed0=$4; pieces=$5; per=$6
here=$(cd "$(dirname "$0")" && pwd)
rm -f "$scratch"/b"$batch"-*.rc
for piece in $(seq 1 "$pieces"); do
  start=$((seed0 + (piece - 1) * per))
  { bash "$here/run.sh" "$scratch" "b$batch-$piece" appearance_fuzz "$start" "$s" "$per"; echo "$?" > "$scratch/b$batch-$piece.rc"; } &
done
wait
got=$(ls "$scratch"/b"$batch"-*.rc 2>/dev/null | wc -l)
if [ "$got" != "$pieces" ]; then echo "BATCH b$batch 作废：派出 $pieces 件，收回 $got 个 .rc"; exit 3; fi
for piece in $(seq 1 "$pieces"); do echo "b$batch-$piece rc=$(cat "$scratch/b$batch-$piece.rc") $(tail -1 "$scratch/out/b$batch-$piece.out")"; done

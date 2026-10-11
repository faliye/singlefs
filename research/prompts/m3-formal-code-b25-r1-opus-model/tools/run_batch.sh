#!/usr/bin/env bash
# 用法：run_batch.sh <测试二进制> <tsv 目录> <批号> <件号…>：每件一个进程并行跑，退出码写 <tsv 目录>/b<批号>-<件号>.rc。
set -u
bin=$1; dir=$2; batch=$3; shift 3
rm -f "$dir"/b"$batch"-*.rc
n=0
for part in "$@"; do
  if [ "${K1_BATCH_NO_INNER_CAP:-}" = 1 ]; then
    # 整条 rerun.sh 已经经内存包装跑：这里不再套一层。
    { K1_HISTORIES="$dir/$part.tsv" nice -n 19 "$bin" --nocapture > "$dir/b$batch-$part.log" 2>&1; echo "$?" > "$dir/b$batch-$part.rc"; } &
  else
    { K1_HISTORIES="$dir/$part.tsv" RUN_WITH_MEMORY_CAP_PARALLEL_REASON="攻方腿原型并行分件跑历史" \
        nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 120M \
        "$bin" --nocapture > "$dir/b$batch-$part.log" 2>&1; echo "$?" > "$dir/b$batch-$part.rc"; } &
  fi
  n=$((n+1))
done
wait
got=$(ls "$dir"/b"$batch"-*.rc 2>/dev/null | wc -l)
if [ "$got" -ne "$n" ]; then echo "批 $batch 作废：rc 文件 $got 个，派出 $n 件"; exit 9; fi
for part in "$@"; do echo "b$batch-$part rc=$(cat "$dir/b$batch-$part.rc")"; done

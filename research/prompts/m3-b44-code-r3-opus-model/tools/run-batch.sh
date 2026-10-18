#!/usr/bin/env bash
# 一批并行跑原型：run-batch.sh <批号> <二进制> <工作目录> <输出目录> <任务表>
# 任务表一行一件：<件号>\t<测试名>\t<空格隔开的 环境变量=值>；每件的输出写 <输出目录>/b<批号>-<件号>.log，退出码写 .rc。
# 全部起完之后一个不带参数的 wait 等齐，数 .rc 个数与件数对不上就整批作废（退 3）；对得上按件号次序逐个读，有非零的退 1。
set -euo pipefail
batch="$1" binary="$2" workdir="$3" outdir="$4" jobs="$5"
rm -f "$outdir"/b"$batch"-*.rc
count=0
while IFS=$'\t' read -r item test assignments; do
  [[ -z "$item" ]] && continue
  count=$((count + 1))
  # shellcheck disable=SC2086
  { (cd "$workdir" && env $assignments PROBE_VERBOSE=1 nice -n 19 "$binary" --nocapture --exact "$test") > "$outdir/b$batch-$item.log" 2>&1; echo "$?" > "$outdir/b$batch-$item.rc"; } &
done < "$jobs"
wait
found=$(ls "$outdir"/b"$batch"-*.rc 2>/dev/null | wc -l)
if [[ "$found" -ne "$count" ]]; then
  echo "batch $batch: rc files $found != jobs $count, whole batch void"
  exit 3
fi
status=0
while IFS=$'\t' read -r item test assignments; do
  [[ -z "$item" ]] && continue
  rc=$(cat "$outdir/b$batch-$item.rc")
  echo "batch $batch item $item rc=$rc"
  [[ "$rc" == 0 ]] || status=1
done < "$jobs"
exit "$status"

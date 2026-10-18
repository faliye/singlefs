#!/usr/bin/env bash
# 跑一批件：run-batch.sh <二进制> <件表> <批号> <输出目录>。件表每行「R 的种类 k或前缀 起点 终点 读故障组表 批号」，只跑批号等于 <批号> 的那几行。
# 每件把退出码写进 <输出目录>/b<批号>-<件号>.rc（开跑前先删这一批的 .rc），最后不带参数 wait 等齐，数 .rc 个数，再按起的次序逐个读。
set -euo pipefail
binary="$1" pieces="$2" batch="$3" out="$4"
rm -f "$out"/b"$batch"-*.rc
mapfile -t lines < <(awk -v b="$batch" '$6 == b' "$pieces")
index=0
for line in "${lines[@]}"; do
  read -r kind k start end faults _batch <<<"$line"
  { nice -n 19 "$binary" "$kind" "$k" run "$start" "$end" "$faults" > "$out/b$batch-$index.out" 2> "$out/b$batch-$index.err"; echo "$?" > "$out/b$batch-$index.rc"; } &
  index=$((index + 1))
done
wait
rc_files=$(ls "$out"/b"$batch"-*.rc 2>/dev/null | wc -l)
echo "batch=$batch launched=$index rc_files=$rc_files"
if [[ "$rc_files" -ne "$index" ]]; then
  echo "batch=$batch void: rc_files != launched"
  exit 3
fi
bad=0
for ((i = 0; i < index; i++)); do rc=$(cat "$out/b$batch-$i.rc"); [[ "$rc" == 0 ]] || bad=$((bad + 1)); echo "piece=$i rc=$rc"; done
echo "batch=$batch nonzero=$bad"

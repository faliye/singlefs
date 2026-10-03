#!/bin/bash
# 用法：short-ring-sweep.sh <草稿目录> <批号> <S 列表，如 "4 5 6"> [last 列表，缺省 "4 6 8"]。短 journal 环（1 MiB，mkfs 收的最小一档）上记录被盖掉那一形：S ∈ {4,5,6}、
# 实例 1 写到 last ∈ {4,6,8}、每次顺序写 c × 512 KiB，c ∈ {2,3,4}（一次发布的记录数上限 85 条，2 MiB 是 65 条），挂载之后写 k = 3S 次；正确写者与坏写者 m4a 各跑一件。
# 8 件并行，每件的退出码写进 <草稿目录>/out/b<批号>-<件号>.rc；每 8 件一个 wait，最后单独一个 wait 等齐。
set -u
scratch=$1; batch=$2; s_list=$3; last_list=${4:-4 6 8}
model=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$scratch/out"
rm -f "$scratch"/out/b"$batch"-*.rc
i=0
for s in $s_list; do for last in $last_list; do for c in 2 3 4; do for writer in ok m4a; do
  name="ss$batch-s$s-l$last-c$c-$writer"
  bw=""; [ "$writer" = m4a ] && bw=m4a
  { HOLE_BAD_WRITER=$bw PROTO_SEQ=1 PROTO_CONTENT_BYTES=$((c * 524288)) bash "$model/scripts/run.sh" "$scratch" "$name" short_ring "$s" 1048576 "$last" $((3 * s)); echo "$?" > "$scratch/out/b$batch-$i.rc"; } &
  i=$((i + 1))
  if (( i % 8 == 0 )); then wait; fi
done; done; done; done
wait
echo "rc_files=$(ls "$scratch"/out/b"$batch"-*.rc | wc -l) dispatched=$i"

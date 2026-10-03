#!/bin/bash
# 用法：rollback-sweep.sh <草稿目录> <批号>。回退之后的用户动作放开扫（长度 0..2，每步取 over | seq | inodes | empty），
# 回退目标取 txg 5..9，洞右边第一条读得出的根 R 取回退那一版到最新那一版之间每一条（upto:R），每件经 run.sh 跑。
# 8 件并行，每件的退出码写进 <草稿目录>/out/b<批号>-<件号>.rc；最后单独一个 wait 等齐。
set -u
scratch=$1; batch=$2
model=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$scratch/out"
rm -f "$scratch"/out/b"$batch"-*.rc
acts="over seq inodes empty"
seqs="-"
for x in $acts; do seqs="$seqs $x"; for y in $acts; do seqs="$seqs $x,$y"; done; done
jobs=()
for target in 5 6 7 8 9; do
  for ops in $seqs; do
    n=0; [ "$ops" != "-" ] && n=$(( $(tr -cd ',' <<<"$ops" | wc -c) + 1 ))
    # 回退那一版是 txg 14（实例 2 挂载写行 11、挂载里另发 12、13），之后每个动作一版；R 取 14..14+n。
    for r in $(seq 14 $((14 + n))); do jobs+=("$target $ops upto:$r"); done
  done
done
echo "jobs=${#jobs[@]}"
i=0
for job in "${jobs[@]}"; do
  set -- $job
  name="rs$batch-t$1-$(tr ',' '+' <<<"$2")-$3"
  { bash "$model/scripts/run.sh" "$scratch" "${name/:/}" rollback_hole 4 1048576 10 "$1" "$2" "$3"; echo "$?" > "$scratch/out/b$batch-$i.rc"; } &
  i=$((i + 1))
  if (( i % 8 == 0 )); then wait; fi
done
wait
echo "rc_files=$(ls "$scratch"/out/b"$batch"-*.rc | wc -l) dispatched=$i"

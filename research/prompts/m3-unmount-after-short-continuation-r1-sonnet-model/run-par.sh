#!/usr/bin/env bash
# 用法: run-par.sh <批号> <环境变量前缀串，如 PROBE_UNMOUNT_RESERVE=24> <行文件>
# 行文件每行: 标签 盘 O 列表 U 列表 B 列表 K 列表（空格分隔，列表逗号分隔）；每行起一个进程，输出 out/<批号>-<标签>.out，退出码 out/<批号>-<标签>.rc
set -uo pipefail
batch="$1"; prefix="$2"; lines="$3"
cd /tmp/claude-1000/m3-unmount-forward
mkdir -p out
rm -f out/"$batch"-*.rc
count=0
while read -r tag disks o u b k; do
  [ -z "$tag" ] && continue
  count=$((count+1))
  {
    env $prefix PROBE_GRID=custom PROBE_DISKS="$disks" PROBE_O="$o" PROBE_U="$u" PROBE_B="$b" PROBE_K="$k" \
      bash /tmp/claude-1000/m3-unmount-forward/run-probe.sh "/tmp/claude-1000/m3-unmount-forward/out/$batch-$tag.out"
    echo "$?" > "out/$batch-$tag.rc"
  } &
done < "$lines"
wait
echo "launched=$count rc-files=$(ls out/"$batch"-*.rc | wc -l)"
cat out/"$batch"-*.rc | sort | uniq -c

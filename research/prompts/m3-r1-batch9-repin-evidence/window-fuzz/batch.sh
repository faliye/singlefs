#!/bin/bash
# 用法：batch.sh <批名> <二进制> <件说明文件> <small|default>；每件一个退出码文件，最后单独 wait、数文件。
D=/tmp/claude-1000/m3-r1/batch9-repin/fz
b=$1; bin=$2; spec=$3; disks=$4
cd $D
rm -f rc-$b-*.txt
n=0
while read -r name envs seed s count; do
  [ -z "$name" ] && continue
  n=$((n+1))
  if [ "$envs" = "-" ]; then envs=""; fi
  if [ "$disks" = small ]; then envs="PROTO_JOURNAL_BYTES=1048576,PROTO_IMAGE_BYTES=268435456${envs:+,$envs}"; fi
  { env $(echo "$envs" | tr ',' ' ') RUN_WITH_MEMORY_CAP_PARALLEL_REASON="改钉实现员复跑随机历史，各件各跑" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="复跑攻方原型，峰值随种子不同" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=120 RUN_WITH_MEMORY_CAP_TIME_LIMIT=220 nice -n 19 bash /home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh 1500M bash /home/fy5090/code/singlefs/scripts/capped.sh 1 $bin "$seed" "$s" "$count" window > out-$b-$name.out 2> out-$b-$name.err; echo "$?" > rc-$b-$n.txt; } &
done < "$spec"
wait
got=$(ls rc-$b-*.txt 2>/dev/null | wc -l)
echo "batch $b pieces=$n rc_files=$got"
for i in $(seq 1 $n); do printf '%s ' "$(cat rc-$b-$i.txt)"; done; echo

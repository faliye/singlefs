#!/bin/bash
# 用法：fz-batch.sh <批号> <件说明文件>；件说明每行：<输出名> <环境变量串（用逗号分隔 K=V，无则 ->）> <起始种子> <S> <段数> <臂>
# 每件经 run.sh（run-with-memory-cap 1G + capped.sh 1）；退出码写 out/b<批>-<件号>.rc；最后单独 wait。
cd /tmp/claude-1000/m3-ring-r3-opus
b=$1; spec=$2
rm -f out/b$b-*.rc
n=0
while read -r name envs seed s count arm; do
  [ -z "$name" ] && continue
  n=$((n+1))
  if [ "$envs" = "-" ]; then envs=""; fi
  { env $(echo "$envs" | tr ',' ' ') ./run.sh "$name" window_fuzz "$seed" "$s" "$count" "$arm"; echo "$?" > out/b$b-$n.rc; } &
done < "$spec"
wait
got=$(ls out/b$b-*.rc 2>/dev/null | wc -l)
echo "batch $b pieces=$n rc_files=$got"
for i in $(seq 1 $n); do printf '%s ' "$(cat out/b$b-$i.rc)"; done; echo

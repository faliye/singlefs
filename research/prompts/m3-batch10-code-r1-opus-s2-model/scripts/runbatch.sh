#!/usr/bin/env bash
# 用法：W=<工作目录> runbatch.sh <批号> <件脚本…>：每件各写自己的 .rc（文件名带批号与件号），最后单独一个 wait 等齐，数 .rc 再按次序读。
set -u
: "${W:?给 W}"
B=$1; shift
rm -f "$W"/b"$B"-*.rc
i=0
for piece in "$@"; do
  i=$((i+1))
  { bash "$piece"; echo "$?" > "$W/b$B-$i.rc"; } &
done
wait
n=$(ls "$W"/b"$B"-*.rc 2>/dev/null | wc -l)
if [ "$n" != "$i" ]; then echo "BATCH $B 作废：派出 $i 件、rc $n 份"; exit 3; fi
for k in $(seq 1 "$i"); do echo "b$B-$k rc=$(cat "$W/b$B-$k.rc")"; done

#!/usr/bin/env bash
# 攻方原型的跑批：batch.sh <批号> <规格文件>；规格每行「件名<TAB>二进制<TAB>环境变量（空格分开）<TAB>测试参数」，各件并行、单线程跑，
# 每件的退出码写 <草稿>/rc/<批号>-<件号>.rc，输出写 <草稿>/out/<件名>.log。最后不带参数的 wait 等齐。
set -u
D=$HOME/.local/share/singlefs/work/m3-batch10z-b-code-r1-opus
batch=$1; spec=$2
mkdir -p "$D/rc" "$D/out"
rm -f "$D/rc/$batch"-*.rc
i=0
while IFS=$'\t' read -r name bin envs args; do
  [[ -z "$name" ]] && continue
  i=$((i+1))
  { env $envs nice -n 19 "$D/target/debug/deps/$bin" $args > "$D/out/$name.log" 2>&1; echo "$?" > "$D/rc/$batch-$i.rc"; } &
done < "$spec"
wait
n=$(ls "$D/rc/$batch"-*.rc 2>/dev/null | wc -l)
echo "batch=$batch pieces=$i rc_files=$n"
for j in $(seq 1 $i); do echo "piece $j rc=$(cat "$D/rc/$batch-$j.rc")"; done

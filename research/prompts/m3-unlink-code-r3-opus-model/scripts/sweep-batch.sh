#!/usr/bin/env bash
# 用法：sweep-batch.sh <批号> <二进制> <日志前缀> <变量名> <取值1> <取值2> …
# 每个取值一件并行跑（同批其余环境变量照传），每件退出码写进 <批号>-<件号>.rc；最后 wait 等齐、数 .rc。
set -u
batch=$1; binary=$2; prefix=$3; var=$4; shift 4
dir=${ATTACK_DIR:-/tmp/claude-1000/m3-unlink-code-r3-opus}
rm -f "$dir"/b"$batch"-*.rc
piece=0
for value in "$@"; do
  piece=$((piece + 1))
  { env "$var=$value" "$dir/run.sh" "$binary" "$dir/out/$prefix-$value.log"; echo "$?" > "$dir/b$batch-$piece.rc"; } &
done
wait
count=$(ls "$dir"/b"$batch"-*.rc 2>/dev/null | wc -l)
echo "batch=$batch pieces=$piece rc_files=$count"
for i in $(seq 1 "$piece"); do echo "piece=$i rc=$(cat "$dir/b$batch-$i.rc")"; done

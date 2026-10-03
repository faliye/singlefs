#!/usr/bin/env bash
# 用法：sweep-batch.sh <批号> <臂名> <日志前缀> <变量名> <取值1> <取值2> …
# 每个取值一件并行跑（同批其余环境变量照传），每件退出码写进 $ATTACK_DIR/b<批号>-<件号>.rc；最后不带参数的 wait 等齐、数 .rc，对不上整批作废。
set -u
batch=$1; arm=$2; prefix=$3; var=$4; shift 4
dir=$ATTACK_DIR
here=$(cd "$(dirname "$0")" && pwd)
rm -f "$dir"/b"$batch"-*.rc
piece=0
for value in "$@"; do
  piece=$((piece + 1))
  { env "$var=$value" bash "$here/run.sh" "$arm" "$dir/out/$prefix-$value.log"; echo "$?" > "$dir/b$batch-$piece.rc"; } &
done
wait
count=$(ls "$dir"/b"$batch"-*.rc 2>/dev/null | wc -l)
echo "batch=$batch arm=$arm pieces=$piece rc_files=$count"
if [ "$count" -ne "$piece" ]; then echo "batch=$batch 作废：.rc 份数对不上"; exit 1; fi
for i in $(seq 1 "$piece"); do echo "piece=$i rc=$(cat "$dir/b$batch-$i.rc")"; done

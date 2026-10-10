#!/usr/bin/env bash
# 用法：grid-batch.sh <批号> <臂名> <日志前缀> <变量甲> <取值,…> <变量乙> <取值,…>
# 每一对 (甲, 乙) 一件并行跑（其余环境变量照传），退出码写进 $ATTACK_DIR/b<批号>-<件号>.rc；不带参数的 wait 等齐、数 .rc，对不上整批作废。
set -u
batch=$1; arm=$2; prefix=$3; var1=$4; vals1=$5; var2=$6; vals2=$7
dir=$ATTACK_DIR
here=$(cd "$(dirname "$0")" && pwd)
rm -f "$dir"/b"$batch"-*.rc
piece=0
for v1 in ${vals1//,/ }; do
  for v2 in ${vals2//,/ }; do
    piece=$((piece + 1))
    { env "$var1=$v1" "$var2=$v2" bash "$here/run.sh" "$arm" "$dir/out/$prefix-$v1-$v2.log"; echo "$?" > "$dir/b$batch-$piece.rc"; } &
  done
done
wait
count=$(ls "$dir"/b"$batch"-*.rc 2>/dev/null | wc -l)
echo "batch=$batch arm=$arm pieces=$piece rc_files=$count"
if [ "$count" -ne "$piece" ]; then echo "batch=$batch 作废：.rc 份数对不上"; exit 1; fi
for i in $(seq 1 "$piece"); do printf 'piece=%s rc=%s\n' "$i" "$(cat "$dir/b$batch-$i.rc")"; done | sort | uniq -c -f1 | awk '{print "rc_summary", $0}'

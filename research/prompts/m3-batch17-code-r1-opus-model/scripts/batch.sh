#!/usr/bin/env bash
# 用法：batch.sh <批号> <件表文件>   件表每行「臂 日志名 环境变量…」（# 起头的行跳过），每行一件并行跑（件数不超过线程上限），
# 每件退出码写进 $ATTACK_DIR/b<批号>-<件号>.rc；最后不带参数的 wait 等齐、数 .rc，对不上整批作废。
set -u
batch=$1; table=$2
dir=$ATTACK_DIR
here=$(cd "$(dirname "$0")" && pwd)
rm -f "$dir"/b"$batch"-*.rc
piece=0
while read -r arm log rest; do
  case "$arm" in ''|'#'*) continue ;; esac
  piece=$((piece + 1))
  # shellcheck disable=SC2086
  { env $rest bash "$here/run.sh" "$arm" "$dir/out/$log.log"; echo "$?" > "$dir/b$batch-$piece.rc"; } &
done < "$table"
wait
count=$(ls "$dir"/b"$batch"-*.rc 2>/dev/null | wc -l)
echo "batch=$batch pieces=$piece rc_files=$count"
if [ "$count" -ne "$piece" ]; then echo "batch=$batch 作废：.rc 份数对不上"; exit 1; fi
for i in $(seq 1 "$piece"); do printf 'rc=%s\n' "$(cat "$dir/b$batch-$i.rc")"; done | sort | uniq -c | sed "s/^/batch=$batch /"

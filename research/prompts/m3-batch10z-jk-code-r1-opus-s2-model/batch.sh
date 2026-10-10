#!/usr/bin/env bash
# batch.sh <批号> <清单>：清单每行「件名 环境变量…」，每件并行起，各写 runs/<批号>/<件名>.rc；最后 wait 等齐、数 .rc。
set -u
S=$HOME/.local/share/singlefs/work/m3-batch10z-jk-code-r1-opus-s2
batch=$1; list=$2
mkdir -p "$S/runs/$batch"; rm -f "$S/runs/$batch"/*.rc
n=0
while read -r name rest; do
  [ -z "$name" ] && continue
  # shellcheck disable=SC2086
  { bash "$S/piece.sh" "$batch" "$name" $rest; } &
  n=$((n+1))
done < "$list"
wait
got=$(ls "$S/runs/$batch"/*.rc 2>/dev/null | wc -l)
echo "BATCH $batch pieces=$n rc_files=$got"
for f in "$S/runs/$batch"/*.rc; do echo "$(basename "$f") $(cat "$f")"; done

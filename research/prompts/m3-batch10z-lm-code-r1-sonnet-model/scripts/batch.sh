#!/usr/bin/env bash
# batch.sh <批号> <清单>：清单每行「件名 探针 环境变量…」，至多 PARALLEL 件并行，每件写自己的 .rc，最后 wait 等齐再数。
set -u
W=$HOME/.local/share/singlefs/work/m3-batch10z-lm-code-r1-sonnet
batch=$1; list=$2; parallel=${PARALLEL:-12}
mkdir -p "$W/runs/$batch"; rm -f "$W/runs/$batch"/*.rc
n=0
while read -r name rest; do
  [ -z "$name" ] && continue
  { bash "$W/scripts/piece.sh" "$batch" "$name" $rest; } &
  n=$((n+1))
  if (( n % parallel == 0 )); then wait; fi
done < "$list"
wait
got=$(ls "$W/runs/$batch"/*.rc 2>/dev/null | wc -l)
bad=$(cat "$W/runs/$batch"/*.rc | grep -vc '^0$')
echo "BATCH $batch pieces=$n rc_files=$got nonzero_rc=$bad"

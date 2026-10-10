#!/usr/bin/env bash
# par7.sh <规格文件> <起> <止> ：规格文件第 起..止 行（1 起），各经 piece7.sh 并行起（timeout 225），wait，报每件的 rc 与行数
W=$HOME/.local/share/singlefs/work/m3-batch10z-xy-code-r1-sonnet
cd "$W"
spec=$1; from=$2; to=$3
names=()
while IFS= read -r line; do
  [ -z "$line" ] && continue
  eval "set -- $line"
  name=$1; shift
  names+=("$name")
  rm -f "runs/$name.rcx"
  { timeout 225 ./scripts/piece7.sh "$name" "$@"; echo "$?" > "runs/$name.rcx"; } &
done < <(sed -n "${from},${to}p" "$spec")
wait
for n in "${names[@]}"; do echo "$n rcx=$(cat runs/$n.rcx) rc=$(cat runs/$n.rc 2>/dev/null) S2lines=$(grep -c '^S2 label' runs/$n.log) DIAG=$(grep -c 'DIAG label' runs/$n.log)"; done

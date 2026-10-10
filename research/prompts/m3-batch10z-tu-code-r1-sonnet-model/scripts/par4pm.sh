#!/usr/bin/env bash
W=$HOME/.local/share/singlefs/work/m3-batch10z-tu-code-r1-sonnet
cd "$W"
names=()
while IFS= read -r line; do
  [ -z "$line" ] && continue
  eval "set -- $line"
  name=$1; shift
  names+=("$name")
  { timeout 230 ./scripts/piece-pm.sh "$name" "$@"; echo "$?" > "runs/pm-$name.rcx"; } &
done
wait
for n in "${names[@]}"; do echo "pm-$n rcx=$(cat runs/pm-$n.rcx) rc=$(cat runs/pm-$n.rc) S2=$(grep -c '^S2 label' runs/pm-$n.log)"; done

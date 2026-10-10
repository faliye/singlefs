#!/usr/bin/env bash
# par4.sh <批前缀> : 读 stdin 里最多 4 行，每行 "<log名> <环境变量…>"；各经 piece.sh 起（timeout 230），最后 wait，再逐个报 rc 与日志行数
W=$HOME/.local/share/singlefs/work/m3-batch10z-tu-code-r1-sonnet
cd "$W"
names=()
i=0
while IFS= read -r line; do
  [ -z "$line" ] && continue
  eval "set -- $line"
  name=$1; shift
  names+=("$name")
  rm -f "runs/$name.rcx"
  { timeout 230 ./scripts/piece.sh "$name" "$@"; echo "$?" > "runs/$name.rcx"; } &
done
wait
for n in "${names[@]}"; do echo "$n rcx=$(cat runs/$n.rcx) rc=$(cat runs/$n.rc) S2lines=$(grep -c '^S2 label' runs/$n.log) DIAG=$(grep -c 'DIAG label' runs/$n.log)"; done

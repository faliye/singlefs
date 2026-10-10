#!/usr/bin/env bash
# par_atk.sh：stdin 每行 "<pqrs|tu> <log名> <测试名> <环境变量…>"，最多 4 行并行（每件 3G），timeout 230，最后 wait，报 rc 与行数
W=$HOME/.local/share/singlefs/work/m3-batch10z-vw-code-r1-sonnet
cd "$W"
names=()
while IFS= read -r line; do
  [ -z "$line" ] && continue
  eval "set -- $line"
  which=$1; name=$2; test=$3; shift 3
  names+=("$name"); rm -f "runs/$name.rcx"
  { timeout 230 ./scripts/arun.sh "$which" "$name" "$test" "$@"; echo "$?" > "runs/$name.rcx"; } &
done
wait
for n in "${names[@]}"; do echo "$n rcx=$(cat runs/$n.rcx) rc=$(cat runs/$n.rc) lines=$(wc -l < runs/$n.log) ATK=$(grep -c '^ATK' runs/$n.log)"; done

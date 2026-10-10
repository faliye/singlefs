#!/usr/bin/env bash
# runq2.sh <规格文件> [路数]：每行 "<树> <目标> <log名> <测试名> <环境变量…>"，没有 rc=0 的件并行（默认 5 路），总期限 200 秒：期限前剩不到 40 秒不再起新件，每件限时不超过剩余时间
W=$HOME/.local/share/singlefs/work/m3-batch10z-xy-code-r1-sonnet
cd "$W"
export DEADLINE=$(( $(date +%s) + 200 ))
: > "$1.todo"
while IFS= read -r line; do [ -z "$line" ] && continue; n=$(echo "$line" | awk '{print $3}'); [ "$(cat runs/$n.rc 2>/dev/null)" = 0 ] || echo "$line" >> "$1.todo"; done < "$1"
echo "待跑 $(wc -l < "$1.todo")"
runone() { line="$1"; left=$(( DEADLINE - $(date +%s) )); [ $left -lt 40 ] && return 0; eval "set -- $line"; rm -f "runs/$3.rc"; timeout $left ./scripts/arun2.sh "$@"; }
export -f runone
xargs -P${2:-5} -d '\n' -I{} bash -c 'runone "$1"' _ {} < "$1.todo"
echo "done: $(for n in $(awk '{print $3}' "$1.todo"); do [ "$(cat runs/$n.rc 2>/dev/null)" = 0 ] && echo $n; done | wc -l) / $(wc -l < "$1.todo")"

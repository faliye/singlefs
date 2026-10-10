#!/usr/bin/env bash
# runq.sh <规格文件>：探针件，7 路并行；总期限 200 秒：剩不到 40 秒不再起新件
W=$HOME/.local/share/singlefs/work/m3-batch10z-xy-code-r1-sonnet
cd "$W"
export DEADLINE=$(( $(date +%s) + 200 ))
python3 scripts/split_spec.py "$1" > "$1.todo"
echo "待跑 $(wc -l < "$1.todo") 件"
runone() { line="$1"; left=$(( DEADLINE - $(date +%s) )); [ $left -lt 40 ] && return 0; eval "set -- $line"; name=$1; shift; rm -f "runs/$name.rc"; timeout $left ./scripts/piece7.sh "$name" "$@"; }
export -f runone
xargs -P7 -d '\n' -I{} bash -c 'runone "$1"' _ {} < "$1.todo"
echo "done: $(for n in $(cut -d' ' -f1 "$1.todo"); do [ "$(cat runs/$n.rc 2>/dev/null)" = 0 ] && echo $n; done | wc -l) / $(wc -l < "$1.todo")"

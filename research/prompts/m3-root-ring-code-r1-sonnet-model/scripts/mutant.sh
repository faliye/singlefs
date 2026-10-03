#!/usr/bin/env bash
# 用法：mutant.sh <名> <文件(相对 crates/)> <行号> <原文> <替换> <测试目标...>
set -u
name="$1"; file="$2"; line="$3"; orig="$4"; repl="$5"; shift 5
work=/tmp/claude-1000/m3-root-ring-code-r1-sonnet/work
frozen=/tmp/claude-1000/m3-root-ring-code-r1/tree
target="$work/crates/$file"
cp "$frozen/crates/$file" "$target"
actual=$(awk -v n="$line" 'NR==n' "$target")
case "$actual" in *"$orig"*) ;; *) echo "MUTANT $name: 锚点不在第 $line 行：$actual"; exit 3;; esac
python3 - "$target" "$line" "$orig" "$repl" <<'PY'
import sys
p,line,orig,repl=sys.argv[1],int(sys.argv[2]),sys.argv[3],sys.argv[4]
lines=open(p).read().split('\n')
assert orig in lines[line-1]
lines[line-1]=lines[line-1].replace(orig,repl,1)
open(p,'w').write('\n'.join(lines))
PY
args=()
for t in "$@"; do args+=(--test "$t"); done
cd "$work"
RUN_WITH_MEMORY_CAP_PARALLEL_REASON="主 agent 同时派了正推与攻方两条腿，各跑各的探针" nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 3G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 15 cargo test --offline --release -p singlefs-harness "${args[@]}" > "/tmp/claude-1000/m3-root-ring-code-r1-sonnet/mut-$name.log" 2>&1
code=$?
cp "$frozen/crates/$file" "$target"
echo "MUTANT $name: exit=$code"; grep -E '^test .* FAILED|test result' "/tmp/claude-1000/m3-root-ring-code-r1-sonnet/mut-$name.log" | head -20

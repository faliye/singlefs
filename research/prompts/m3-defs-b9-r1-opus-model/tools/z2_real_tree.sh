#!/usr/bin/env bash
# Z2 真树喂法：从冻结副本硬链接拷一棵（格扫 scripts、.claude/gate.d、.claude/hooks、crates、登记与排除表），
# 按变体改一处，再用 z2_htd_cell.py 照那一版自己的格判。只改拷贝，冻结副本只读。
# 用法：z2_real_tree.sh <名>=<冻结副本根> <工作目录>
set -euo pipefail
trim() { python3 -c 'import sys; n = int(sys.argv[1]); sys.stdout.write("".join(line.rstrip("\n")[:n] + "\n" for line in sys.stdin))' "$1"; }   # 按字截（cut -c 按字节截，会把中文切坏）
label="${1%%=*}" frozen="${1#*=}" work="$2"
tools="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$work"
make_tree() {   # 硬链接拷整棵（格只读），要改的文件由 replace_once 先换成新 inode
  local tree="$1"
  rm -rf "$tree"; cp -al "$frozen" "$tree"
}
replace_once() {   # 文件 旧串 新串：旧串要恰好命中一次；先断开硬链接再写
  cp --remove-destination "$1" "$1.new-inode" && mv "$1.new-inode" "$1"
  python3 -B - "$@" <<'PY'
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
assert text.count(old) == 1, (path, text.count(old))
open(path, "w", encoding="utf-8").write(text.replace(old, new, 1))
PY
}
judge() {
  local name="$1" tree="$2"
  printf '%s\t%s\t' "$label" "$name"
  nice -n 19 python3 -B "$tools/z2_htd_cell.py" "$frozen" "$tree" > "$work/$label-$name.txt" 2>&1 || true
  { grep -E '^  (✓|✗) ' "$work/$label-$name.txt" | head -1 | trim 120 || true; grep -E 'random_histories|crates-mutation-rows|new_sampling|new_rows' "$work/$label-$name.txt" | head -3 | trim 200 || true; tail -1 "$work/$label-$name.txt"; } | tr '\n' ' '
  echo
}
tree="$work/$label-tree"
make_tree "$tree"
judge base "$tree"
# 甲：已有「代码批」行（锚串 available_parallelism）的文件里新加一个默认线程数函数，不设上限
replace_once "$tree/crates/singlefs-harness/tests/random_histories.rs" $'fn worker_threads_by_default() -> usize {\n' $'fn sampling_workers_by_default() -> usize {\n    std::thread::available_parallelism().map_or(1, usize::from)\n}\n\nfn worker_threads_by_default() -> usize {\n'
judge insert-fn-exempted-file "$tree"
make_tree "$tree"
printf '%s\n' 'fn sampling_workers_by_default() -> usize {' '    std::thread::available_parallelism().map_or(1, usize::from)' '}' > "$tree/crates/singlefs-harness/tests/new_sampling.rs"
judge insert-fn-new-file "$tree"
# 乙：排除表钉住的那一行改几个字，读进一个 capped.sh 不设、也不回退的新线程变量
make_tree "$tree"
replace_once "$tree/scripts/mutation/crates-mutation-rows.py" '        processor_count = os.cpu_count() or 4' '        processor_count = int(os.environ.get("GATE_MUTATION_TOTAL_WORKERS") or 0) or os.cpu_count() or 4'
judge edit-exempted-line "$tree"
make_tree "$tree"
printf '%s\n' 'import os' '' 'def worker_count():' '    processor_count = int(os.environ.get("GATE_MUTATION_TOTAL_WORKERS") or 0) or os.cpu_count() or 4' '    return processor_count' > "$tree/scripts/mutation/new_rows.py"
judge edit-line-new-file "$tree"
rm -rf "$tree"

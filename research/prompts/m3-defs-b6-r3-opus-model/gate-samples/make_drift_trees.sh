#!/usr/bin/env bash
# 门禁 doc-process-records 的 abandoned-rounds 格用的两棵小树（Y1-丙）：同一张叫停登记表、同一个出处「records/log.md 第 3 行」，
# before 里第 3 行是这一轮的叫停原话；after 在第 2 行之后插了一行小节标题（照常往调度记录里加一节），第 3 行成了标题。轮 dr-r1 发过腿（有 -opus-output.md）、没有判决。
# 用法：make_drift_trees.sh <新目录>
set -euo pipefail
root=$1
[[ ! -e "$root" || -z "$(ls -A "$root")" ]] || { echo "  ✗ $root 已存在且非空" >&2; exit 2; }
for tree in before after; do
  t="$root/$tree"
  mkdir -p "$t/records" "$t/research/prompts/dr-r1-snapshot"
  if [[ $tree == before ]]; then
    printf '# 样本调度\n\n- 2026-10-13 用户叫停 dr-r1：「这一轮先放一放，别再派腿了」\n- dr-r1 之后的活并进别处。\n' > "$t/records/log.md"
  else
    printf '# 样本调度\n\n## 批 9（2026-10-14）\n\n- 2026-10-13 用户叫停 dr-r1：「这一轮先放一放，别再派腿了」\n- dr-r1 之后的活并进别处。\n' > "$t/records/log.md"
  fi
  printf '# body\n' > "$t/research/prompts/_dr-r1-body.md"; printf '# out\n' > "$t/research/prompts/dr-r1-opus-output.md"
  printf '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  crates/foo/src/lib.rs\n' > "$t/research/prompts/dr-r1-snapshot/frozen-sha256.txt"
  printf 'dr-r1\t2026-10-13\t用户叫停；出处 records/log.md 第 3 行\t并进别处\n' > "$t/research/prompts/abandoned-rounds.tsv"
done
echo "  ✓ 造了两棵树：$root/before、$root/after"

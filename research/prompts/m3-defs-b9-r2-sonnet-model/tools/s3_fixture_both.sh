#!/usr/bin/env bash
# S3：门禁 code-tooling 的 heavy-thread-defaults 各份样本（用这一轮冻结副本里的样本目录），分别用 b9 与 b10 的格代码判（z2_htd_cell.py 抽出格的 python 原样跑），
# 打每份样本两边点名的文件与行。装配带进样本的 scripts/hosts/heavy_threads.py（格要读它）两行不算样本，滤掉。
# 用法：s3_fixture_both.sh <b9 树> <b10 树> <工作目录> <z2_htd_cell.py>
set -uo pipefail
t9="$1" t10="$2" work="$3" cell="$4"
mkdir -p "$work"
for fixture in "$t10"/.claude/gate.d/fixtures/code-tooling.sh/heavy-thread-defaults-*; do
  name="$(basename "$fixture")"
  for label in b9 b10; do
    root="$t9"; [[ "$label" == b10 ]] && root="$t10"
    tree="$work/$label-$name"; mkdir -p "$tree"; cp -a "$fixture/." "$tree/"
    [[ -f "$tree/scripts/hosts/heavy_threads.py" ]] || { mkdir -p "$tree/scripts/hosts"; cp "$t10/scripts/hosts/heavy_threads.py" "$tree/scripts/hosts/"; }
    out="$(cd "$tree" && PYTHONDONTWRITEBYTECODE=1 python3 -B "$cell" "$root" "$tree" 2>&1)"
    named="$(grep -E '^     (scripts|crates)/' <<<"$out" | grep -v 'scripts/hosts/heavy_threads.py' | awk '{print $1}' | tr '\n' ' ')"
    printf '%s\t%s\t%s\t点名：%s\n' "$name" "$label" "$(tail -1 <<<"$out")" "$named"
  done
done

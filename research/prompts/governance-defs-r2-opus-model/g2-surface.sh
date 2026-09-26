#!/usr/bin/env bash
# 用法：bash g2-surface.sh <仓根>
# 放开「别的会话改了哪一个文件」这一步：对 crash-verifier 第 1 步那六条路径下今天每一个已跟踪文件，
# 数「它在工作区里被改而没暂存 ⇒ 那条命令退 1 ⇒ 三道都停」时，它是不是 55 / 57 / 59 各自读的输入。
set -uo pipefail
root="$1"; cd "$root" || exit 2
row_paths() { awk -F'\t' -v s="$1" '$1 == s { print $2 }' .claude/gate.d/stage-inputs.tsv; }
read -r -a in55 <<< "$(row_paths 55-qemu-first-transaction.sh)"
read -r -a in59 <<< "$(row_paths 59-crates-mutation-replay.sh)"
in57=(litmus/ crates/)
covered() { local file="$1"; shift; local p; for p in "$@"; do [[ "$file" == "$p"* ]] && return 0; done; return 1; }
total=0; not55=0; not57=0; not59=0; not_any=0
while IFS= read -r file; do
  total=$((total + 1)); a=0; b=0; c=0
  covered "$file" "${in55[@]}" || { not55=$((not55 + 1)); a=1; }
  covered "$file" "${in57[@]}" || { not57=$((not57 + 1)); b=1; }
  covered "$file" "${in59[@]}" || { not59=$((not59 + 1)); c=1; }
  (( a && b && c )) && not_any=$((not_any + 1))
done < <(git ls-files -- crates Cargo.toml Cargo.lock litmus research/scripts research/results)
echo "六条路径下已跟踪文件 $total 个"
echo "  不是 55 号输入（stage-inputs.tsv 那一行）的：$not55"
echo "  不是 57 号输入（litmus/、crates/）的：$not57"
echo "  不是 59 号输入（stage-inputs.tsv 那一行）的：$not59"
echo "  三道都不读、改了照样让三道全停的：$not_any"

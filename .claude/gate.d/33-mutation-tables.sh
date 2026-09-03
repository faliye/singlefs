#!/usr/bin/env bash
# gate-stage: 每个实验二进制都要有变异表
#
# 判据：`research/e7-index-bench/src/bin/` 下每个 `*.rs` 在 `research/mutations/`
# 下都要有**同名** `.tsv` 变异表，且表里至少一条成形的变异（三段制表符分隔）。
#
# 为什么：kb 里每一句「N 条变异全部被抓」都靠这些表复现（C40（变异表没存）——
# 结论留下了而产生结论的装置没留下，正是 2026-08-29 审计实测过的失败形态；
# 2026-09-03 把最后 19 张欠表补齐后，用这条阶段拦住它再欠回去）。
#
# ⚠️ **这条阶段不跑变异**（跑一遍全部表要逐条重编译，量级是小时）。
# 「表今天还会不会红」由每轮改动实验代码时手跑 `research/scripts/mutate.sh` 证明，
# 复跑记录见各实验正文的「口径与复跑」——本阶段只保证装置在、形状对，不冒充跑过。
set -uo pipefail
BIN_DIR=research/e7-index-bench/src/bin
MUT_DIR=research/mutations
[[ -d "$BIN_DIR" && -d "$MUT_DIR" ]] || { echo "  ! 找不到 $BIN_DIR 或 $MUT_DIR，本阶段跳过"; exit 0; }

missing=(); malformed=()
for src in "$BIN_DIR"/*.rs; do
  stem="$(basename "$src" .rs)"
  tsv="$MUT_DIR/$stem.tsv"
  if [[ ! -f "$tsv" ]]; then missing+=("$stem"); continue; fi
  # 至少一条成形的变异行：非注释、非空、恰好三段
  ok_rows=$(awk -F'\t' '!/^#/ && NF==3 && $1!="" && $2!="" {n++} END{print n+0}' "$tsv")
  bad_rows=$(awk -F'\t' '!/^#/ && NF!=3 && $0!="" {n++} END{print n+0}' "$tsv")
  if [[ "$ok_rows" -eq 0 || "$bad_rows" -gt 0 ]]; then malformed+=("$stem(成形 $ok_rows 条/坏 $bad_rows 行)"); fi
done

if ((${#missing[@]} + ${#malformed[@]})); then
  if ((${#missing[@]})); then
    echo "  ✗ 这些实验二进制没有同名变异表："
    printf '      %s\n' "${missing[@]}"
  fi
  if ((${#malformed[@]})); then
    echo "  ✗ 这些变异表不成形（要求每行三段制表符分隔，至少一条）："
    printf '      %s\n' "${malformed[@]}"
  fi
  echo "  → 怎么办：写 research/mutations/<bin名>.tsv（每行：变异名<TAB>原文<TAB>替换文），"
  echo "    跑 bash research/scripts/mutate.sh <bin> <源文件> <表> 证明每条都被抓，再来。"
  exit 1
fi
n=$(ls "$BIN_DIR"/*.rs | wc -l)
echo "  ✓ $n 个实验二进制都有成形的变异表（本阶段不跑变异，只验装置在）"

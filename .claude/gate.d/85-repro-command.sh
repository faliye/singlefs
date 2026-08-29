#!/usr/bin/env bash
# gate-stage: 跑过的实验有没有留下复跑命令
#
# 判据：一个实验正文若点名了 `research/results/` 里的产物，就必须同时写出**怎么把它跑出来**——
# 一条 `cargo run --release --bin X` 或一条 `research/scripts/*.sh`。
#
# ⚠️ **这条是实测出来的**：2026-08-29 的复跑轮里，E9（key 编码对遍历局部性的影响）的入库产物是
# **25 次运行拼起来的**（5 种子 × 5 改名档），而那个循环一个字都没写进 kb。
# 复跑的人只能从产物的 config 行反推参数——反推对了才发现它本来就复跑得出来。
# 「产物在」和「产物跑得出来」是两件事，`40-results-cited.sh` 只查了前一件。
#
# 例外：正文写明「原始输出未留存」的（要真设备 / 虚机的那类），本阶段不管。
set -uo pipefail
EXP_DIR=.claude/kb/experiments
[[ -d "$EXP_DIR" ]] || { echo "  ! 找不到 $EXP_DIR，本阶段跳过"; exit 0; }

bad=()
for f in "$EXP_DIR"/*.md; do
  grep -qE 'research/results/[A-Za-z0-9._-]+\.out' "$f" || continue
  grep -qE 'cargo run --release --bin |research/scripts/[a-z0-9-]+\.sh' "$f" && continue
  bad+=("$(basename "$f")")
done

if ((${#bad[@]})); then
  echo "  ✗ 这些实验点了产物却没写复跑命令，读的人只能靠反推："
  printf '      %s\n' "${bad[@]}"
  echo "  → 怎么办：在正文的口径一节补一句，格式与别处一致——"
  echo "    代码 \`research/e7-index-bench/src/bin/eNN_xxx.rs\`（\`cargo run --release --bin eNN-xxx\`），"
  echo "    原始输出 \`research/results/eNN-xxx-YYYY-MM-DD.out\`。"
  echo "    多次运行拼起来的产物，要把那个循环也写出来（种子、参数各扫了哪些值）。"
  echo "    要真设备 / 虚机因而没留产物的，正文写明「原始输出未留存」，本阶段就不管它。"
  exit 1
fi
echo "  ✓ 点了产物的实验都写了复跑命令（$(grep -lE 'research/results/[A-Za-z0-9._-]+\.out' "$EXP_DIR"/*.md | wc -l) 个）"

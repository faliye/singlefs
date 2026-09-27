#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 跑过的实验有没有留下复跑命令
#
# 判据：一个实验正文若点名了 `research/results/` 里的产物，就必须同时写出**怎么把它跑出来**——
# 一条 `cargo run --release --bin X`，或者点名复跑入口 `research/scripts/replay.sh` / `research/scripts/vm-bench.sh`。
# 提到别的 `research/scripts/*.sh`（`mutate.sh` 这类）不算写了复跑命令：它们跑不出那份产物。
#
# ⚠️ **这条是实测出来的**：2026-08-29 的复跑轮里，E9（key 编码对遍历局部性的影响）的入库产物是
# **25 次运行拼起来的**（5 种子 × 5 改名档），而那个循环一个字都没写进 kb。
# 复跑的人只能从产物的 config 行反推参数——反推对了才发现它本来就复跑得出来。
# 「产物在」和「产物跑得出来」是两件事，`40-results-cited.sh` 只查了前一件。
#
# 例外：正文写明「原始输出未留存」的（要真设备 / 虚机的那类），本阶段不管，成功行逐个列名。
# 读不了的页判红（grep 出错与「没点产物」不是一回事）；一页都没点产物，本次无对象可判，退 77。
set -uo pipefail
EXP_DIR=.claude/kb/experiments
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
[[ -d "$EXP_DIR" ]] || { echo "  ! 找不到 $EXP_DIR，本阶段跳过"; exit 77; }
PRODUCT_RE='research/results/[A-Za-z0-9._-]+\.out'
REPRO_RE='cargo run --release --bin |research/scripts/(replay|vm-bench)\.sh'

bad=(); unreadable=(); exempt=(); pointing=0
for f in "$EXP_DIR"/*.md; do
  grep -qE "$PRODUCT_RE" "$f"; rc=$?
  if ((rc == 2)); then unreadable+=("$(basename "$f")"); continue; fi
  ((rc == 0)) || continue
  pointing=$((pointing + 1))
  if grep -qF '原始输出未留存' "$f"; then exempt+=("$(basename "$f")"); continue; fi
  grep -qE "$REPRO_RE" "$f" && continue
  bad+=("$(basename "$f")")
done

if ((${#unreadable[@]})); then
  echo "  ✗ 这些实验页读不了，点没点产物、写没写复跑命令都没判："
  printf '      %s\n' "${unreadable[@]}"
  echo "  → 怎么办：按 grep 的报错修好（权限、编码），再跑；读不了的页不能当成没点产物放过去。"
  exit 1
fi

if ((${#bad[@]})); then
  echo "  ✗ 这些实验点了产物却没写复跑命令，读的人只能靠反推："
  printf '      %s\n' "${bad[@]}"
  echo "  → 怎么办：在正文的口径一节补一句，格式与别处一致——"
  echo "    代码 \`research/e7-index-bench/src/bin/eNN_xxx.rs\`（\`cargo run --release --bin eNN-xxx\`），"
  echo "    原始输出 \`research/results/eNN-xxx-YYYY-MM-DD.out\`。"
  echo "    多次运行拼起来的产物，要把那个循环也写出来（种子、参数各扫了哪些值）。"
  echo "    要真设备 / 虚机因而没留产物的，正文写明「原始输出未留存」，本阶段就不管它。"
  echo "    复跑入口只认 cargo run --release --bin 与 research/scripts/replay.sh、research/scripts/vm-bench.sh；提到 mutate.sh 这类别的脚本不算。"
  exit 1
fi
if ((pointing == 0)); then
  echo "  ⊘ 本次无对象可判：$EXP_DIR 下没有一页点到 research/results/ 里的产物"
  exit 77
fi
echo "  ✓ 点了产物的实验都写了复跑命令（点了产物的 ${pointing} 页里判了 $((pointing - ${#exempt[@]})) 页）"
echo "    写明「原始输出未留存」不判的 ${#exempt[@]} 页：${exempt[*]:-（没有）}"

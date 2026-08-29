#!/usr/bin/env bash
# gate-stage: 实验产物有没有写回
#
# 判据：`research/results/` 里的实验产物，必须在 `kb/experiments.md` 里被点名。
# 点不到名 = 跑过但结论没写回，或者写回了却无法复核（读的人拿不到那份原始数据）。
#
# ⚠️ **这条是实测出来的，不是想出来的**：2026-08-29 有一次把 E20 从 2 档扩到 6 档、
# 跑了三轮、原始输出 22 KB 落了盘，而 experiments.md 里那一节还是两点对比，
# 决策侧一次都没引——报告只存在于对话里。
set -uo pipefail
EXP=.claude/kb/experiments.md
RES=research/results
[[ -f "$EXP" ]] || { echo "  ! 找不到 $EXP，本阶段跳过"; exit 0; }
[[ -d "$RES" ]] || { echo "  ✓ 没有 $RES 目录，无对象可判"; exit 0; }

missing=()
while IFS= read -r f; do
  b="$(basename "$f")"
  # 三类不算产物：本地腿的问答、逐轮复跑的中间件、确认目录
  case "$b" in *local*|*.r[0-9].out|*.round*|confirm*) continue;; esac
  grep -qF "$b" "$EXP" || missing+=("$b")
done < <(find "$RES" -maxdepth 1 -name '*.out' -type f | sort)

if ((${#missing[@]})); then
  echo "  ✗ 有实验产物没被 experiments.md 点名——跑过但没写回，或写回了却无法复核："
  for m in "${missing[@]}"; do echo "     $RES/$m"; done
  echo "     → 怎么办：把结论写进 experiments.md 对应实验的正文，"
  echo "               并在口径段点名这份原始输出；确实是废弃产物就删掉它。"
  exit 1
fi
echo "  ✓ $RES 下的实验产物全部被 experiments.md 点名（$(find "$RES" -maxdepth 1 -name '*.out'|wc -l) 个文件）"

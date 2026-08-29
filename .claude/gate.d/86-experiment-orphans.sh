#!/usr/bin/env bash
# gate-stage: research 里的实验号在 kb 里有没有正文
#
# 判据：`research/` 下凡是以 `eNN` 命名的东西（提示、产物、源码、变异表），
# `kb/experiments/` 里就必须有对应编号的正文文件。没有 = 干了活但没入库，
# 而**跑过的东西没入库比没跑更危险**——它会以「我们量过」的形式活在对话里，谁也复核不了。
#
# ⚠️ **这条是实测出来的**：2026-08-29 复跑轮现查，`research/prompts/e35-rootring-geometry-local.md`
# 与 `research/results/e35-rootring-local.out` 都在，而 kb 里没有 E35。
# 已有的阶段 `40-results-cited.sh` 抓不到它，因为那个阶段把文件名含 `local` 的一律当本地腿问答排除掉了
# ——**排除规则正好盖住了这一个**。
set -uo pipefail
EXP_DIR=.claude/kb/experiments
[[ -d "$EXP_DIR" ]] || { echo "  ! 找不到 $EXP_DIR，本阶段跳过"; exit 0; }

have=$(ls "$EXP_DIR" | grep -oE '^[0-9]+' | sed 's/^0*//' | sort -un)
want=$(for d in research/prompts research/results research/e7-index-bench/src/bin research/mutations; do
         ls "$d" 2>/dev/null; done | grep -ohE '^e[0-9]+' | sed 's/^e//' | sort -un)

orphan=()
for n in $want; do grep -qx "$n" <<<"$have" || orphan+=("E$n"); done

if ((${#orphan[@]})); then
  echo "  ✗ research 里有这些实验号的东西，kb/experiments/ 里却没有正文："
  for e in "${orphan[@]}"; do
    printf '      %-6s' "$e"
    find research -maxdepth 3 -name "${e,,}[-_]*" -o -maxdepth 3 -name "${e,,}.*" 2>/dev/null | tr '\n' ' '
    echo
  done
  echo "  → 怎么办：给它建正文（测什么 / 判据 / 失败条款 / 口径 / 复跑命令），并登记进 experiments.md 的索引表；"
  echo "    若那轮的结论不打算入库，就把 research 下那些文件删掉——留着等于留一份没人能复核的证据。"
  exit 1
fi
echo "  ✓ research 里的实验号在 kb 里都有正文（$(wc -w <<<"$want") 个）"

#!/usr/bin/env bash
# gate-stage: 每个实验二进制都要有钉绝对值的断言（research/e7-index-bench/src/bin 下全部，别处 src/bin 下以 e<数字>_ 开头的；其余成功行逐个列名）
#
# 还 test-discipline.md 的这一条：
# 「只让多条臂互相比，测不出『所有臂一起错』……每一条互比断言旁边，
#   必须有一条把绝对值钉死的断言。」
#
# **判据只做机器判得了的那一半**：这个实验有没有**任何一条**把某个量与数字字面量比死的断言。
# 判不了的那一半（钉的是不是对的那个量、绝对值是不是独立算出来的）留给人。
#
# ⚠️ **零绝对值断言是一个可判定的、且实测出过问题的形态**（2026-08-29 对抗验证）：
# `e9_keylayout` 与 `e21_cpu` 当时各是 0 条，七个 / 两个单测钉的全是结构性质。
# e9 支撑 D8（核心索引结构） 的 key 布局（1.46×），e21 支撑 D24（后台重活能不能卸给 GPU）
# 的跨语言比值——两个都是承重结论，而它们量出来的那个数没有任何东西钉。
#
# 判别力已证（2026-08-29）：把 e9 的四条绝对值断言注释掉 ⇒ 本阶段判红。
#
# 射程：research/e7-index-bench/src/bin 下的每一份，加上别处 `crates/*/src/bin`、`research/*/src/bin` 下文件名以 `e<数字>_` 开头的
# （实验编号的写法，`.claude/abbreviations` 登记的 `e<数字>`）——按「是不是实验」认，不按住在哪个目录认：
# 住在 crates/ 下的实验与 research/ 下的同规矩。别处不以 `e<数字>_` 开头的是装置工具（例：拿设备日志逐项比 ground truth 的），
# 不判，成功行逐个列名，清单现算；research/prompts/ 下腿的模型是冻结证据，不算实验二进制，不列。
# 三方判决：research/prompts/gate-fix-forks-r1-main-verification.md 的 T4。
# 样本：fixtures/80-absolute-assertions.sh/red 放一份只比相对值的（e7 目录）与一份住在 crates/ 下、以 e<数字>_ 开头、只比相对值的，
# 两份都要点名判红；green 的两份都钉了绝对值、判绿，另放一份不以 e<数字>_ 开头的 crates/demo/src/bin/probe.rs，成功行要把它列成没判的。
#
#   bash .claude/gate.d/80-absolute-assertions.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
BINS=research/e7-index-bench/src/bin
[[ -d "$BINS" ]] || { echo "  ! 没有 $BINS，本阶段无对象可判"; exit 77; }

judged=() uncovered=()
for f in "$BINS"/*.rs; do
  [[ -f "$f" ]] && judged+=("$f")   # 目录是空的时候 glob 原样留着，不是一份实验
done
in_bins=${#judged[@]}
for directory in crates/*/src/bin research/*/src/bin; do
  [[ -d "$directory" && "$directory" != "$BINS" ]] || continue
  for f in "$directory"/*.rs; do
    [[ -f "$f" ]] || continue
    if [[ "$(basename "$f")" =~ ^e[0-9]+_ ]]; then judged+=("$f"); else uncovered+=("$f"); fi
  done
done
bad=0; n=0
for f in "${judged[@]}"; do
  n=$((n+1))
  # 绝对值断言：与数字字面量比死。两种形态都认。
  c=$(grep -cE 'assert_eq!\([^;]*, *-?[0-9][0-9_]*(\.[0-9]+)?\)|assert!\([^;]*[<>=]=? *-?[0-9][0-9_]*(\.[0-9]+)?[,)]' "$f")
  if (( c == 0 )); then
    echo "  ✗ $f 一条绝对值断言都没有"
    bad=$((bad+1))
  fi
done

if ((bad)); then
  echo "     → 怎么办：给它加一条把**被量的那个数**钉死的断言，绝对值要由**独立算术**给出，"
  echo "               不许从代码里读回来。加完用 research/scripts/mutate.sh 证明它会红——"
  echo "               实测教训：先加的断言可能一条变异都拦不住（E9 踩过），只有变异测试分得开。"
  exit 1
fi
((n)) || { echo "  ! $BINS 下一个 .rs 都没有、别处也没有 e<数字>_ 开头的，本阶段无对象可判"; exit 77; }
echo "  ✓ $n 个实验二进制各自至少有一条绝对值断言（$BINS 下 $in_bins 份，别处 src/bin 下以 e<数字>_ 开头的 $(( n - in_bins )) 份）"
echo "    没判的 ${#uncovered[@]} 份（别处 src/bin 下不以 e<数字>_ 开头，按装置工具算）："
for f in "${uncovered[@]}"; do echo "      $f"; done

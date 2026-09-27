#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 上一轮及更早的实验记录已经归档进版本库，工作区里不留
#
# 规则见 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「原样保存的证据不许事后改」之下那三段：
# 冻结只在本轮内有效；这一轮提交之后，下一次提交把上一次留下的那批删掉；本次的照旧留着、本次的验证照旧做得了；
# 要查删掉的那些去版本库历史里找，对早先的结论有疑问就重新验证，不翻旧证据。
#
# ⚠️ **这条是实测出来的**（2026-09-21）：留着的旧记录不只占地方，它让当前工程的命名、路径、口径出现二义性——
# 那天一次全仓术语改名量出 22741 处旧名，其中 19639 处（86%）躺在上一轮及更早的提示与产物里，
# 改它们对今天的工程没有任何用处，不改又让「全仓一致」这件事永远做不完。
#
# 保留三类：三方判决 `*-main-verification.md`（kb 的依据指着它）、`abandoned-rounds.tsv`（门禁 66 号的输入）、
# 还被代码当输入的产物（门禁阶段、脚本或装置源码的非注释行点名它，见 archive-past-rounds.py 的 still_an_input）。
# 「这一轮」的起点与门禁 68 号同一个基准：research/scripts/changed-paths.sh 的 `gate_diff_base gate`
# （GATE_BASE，否则 @{upstream} 的 merge-base，都没有就 HEAD）。
# 判别力：fixtures/91-archive-past-rounds.sh/red 是一个留着上一轮产物的小仓，必须判红；green 只有本轮的，必须判绿。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
SCRIPT="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/archive-past-rounds.py"
[[ -d "$ROOT/research/prompts" || -d "$ROOT/research/results" ]] || { echo "  ! 没有 research/prompts 与 research/results，本阶段无对象可判"; exit 77; }
if python3 "$SCRIPT" --check "$ROOT"; then
  exit 0
fi
echo "  ✗ 工作区里还留着上一轮及更早的实验记录（上面按目录列出份数）"
echo "     → 怎么办：照上面那一行打出来的基准跑 --apply 删掉（不带基准它会按 HEAD 算、把这一次提交要带的记录也删掉），"
echo "               它同时把别处指向这些文件的引用改成只留文件名、不留路径；删完跑共享门禁的「链接指向」阶段确认链接都还到得了。"
exit 1

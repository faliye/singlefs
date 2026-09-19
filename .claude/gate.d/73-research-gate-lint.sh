#!/usr/bin/env bash
# gate-stage: research/scripts/ 与 .claude/hooks/ 里每一条拒绝都带出路
#
# 共享门禁的「门禁自检」只把 SOP 自己的脚本与 .claude/gate.d/ 交给 gate-lint（.claude/singlefs-ai-sop/scripts/gate.sh 第 211–212 行），
# 研究脚本与 hook 不在射程里。2026-09-18 单跑整仓 gate-lint 红 90 处，84 处在这两个目录，没有任何一笔账记着（C382（研究脚本与 hook 的拒绝不在门禁自检的射程里））。
# research/prompts/ 下的脚本是冻结证据，不扫（同 .claude/doc-lint-exclude 那一行的理由）。
# 判别力：fixtures/73-research-gate-lint.sh/red 放一个只带一句话的 die，必须判红；green 带上第二个参数，必须判绿。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
LINT="$(cd "$(dirname "$0")/../.." && pwd)/.claude/singlefs-ai-sop/scripts/gate-lint.sh"
[[ -f "$LINT" ]] || { echo "  ✗ 找不到共享脚本 $LINT"; echo "     → 怎么办：规范副本没装，把 singlefs-ai-sop-<语言> 仓拷进 .claude/singlefs-ai-sop/ 再跑它的 install.sh"; exit 1; }
TARGETS=()
for directory in "$ROOT/research/scripts" "$ROOT/.claude/hooks"; do
  [[ -d "$directory" ]] && TARGETS+=("$directory")
done
((${#TARGETS[@]})) || { echo "  ! 没有 research/scripts/ 也没有 .claude/hooks/，本阶段无对象可判"; exit 77; }
# GATE_LINT_DIR 指到第一个目标目录：不设它，共享脚本默认还会扫 SOP 自己的包，样本目录里红绿就靠那个包碰巧干净来分。
if GATE_LINT_DIR="${TARGETS[0]}" bash "$LINT" "${TARGETS[@]:1}"; then
  exit 0
fi
echo "  ✗ research/scripts/ 或 .claude/hooks/ 里有不带出路的拒绝（上面逐处列出）"
echo "     → 怎么办：照 .claude/singlefs-ai-sop/rules/sop-first.md「每一条拒绝都必须给出下一步」补出路：die 加第二个参数，bad 后五行内写 howto，直接打印的拒绝之后跟一行以箭头开头的出路"
exit 1

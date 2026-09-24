#!/usr/bin/env bash
# gate-stage: research/scripts/、.claude/hooks/ 与 .claude/scripts/ 里每一条拒绝都带出路、守 shell 纪律；前两个目录的执行位不丢
#
# 共享门禁的「门禁自检」只把 SOP 自己的脚本与 .claude/gate.d/ 交给 gate-lint（.claude/singlefs-ai-sop/scripts/gate.sh 第 211–212 行），
# 研究脚本与 hook 不在射程里。2026-09-18 单跑整仓 gate-lint 红 90 处，84 处在这两个目录，没有任何一笔账记着（C382（研究脚本与 hook 的拒绝不在门禁自检的射程里））。
# research/prompts/ 下的脚本是冻结证据，不扫（同 .claude/doc-lint-exclude 那一行的理由）。
# shell-lint 同一批目录一起跑（上游 show-me-test.md「射程只到 .claude/gate.d/」：只接 gate-lint 等于只补了一半，
# 2026-09-19 单跑 shell-lint 报 14 处，全是它认不出一行写完的函数造成的假红，上游已修、同步副本之后才生效）。两个 lint 都跑完再判，不在第一个红就停。
# .claude/scripts/ 此前同样不在这两个 lint 的射程里（共享门禁只把它交给「脚本执行位」），而 lkmm.sh 一份就有二十多处拒绝，
# 所以一并交给 gate-lint 与 shell-lint；执行位仍归共享那一道，这里不重复扫。
# 判别力：fixtures/73-research-gate-lint.sh/red 放一个只带一句话的 die 和一个靠子 shell 赋值往外带值的函数，两样都必须报出来；
# 它的 .claude/scripts/ 里再放一个不带出路的拒绝和一处 pkill -f，只有这个目录进了射程才报得出来。
# green 带上第二个参数、不靠子 shell 带值，必须判绿；它的 .claude/scripts/ 里放一个干净的包装，成功行的脚本数要把它数进去。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
LINT="$(cd "$(dirname "$0")/../.." && pwd)/.claude/singlefs-ai-sop/scripts/gate-lint.sh"
SHELL_LINT="$(dirname "$LINT")/shell-lint.sh"
for script in "$LINT" "$SHELL_LINT"; do
  [[ -f "$script" ]] || { echo "  ✗ 找不到共享脚本 $script"; echo "     → 怎么办：规范副本没装，把 singlefs-ai-sop-<语言> 仓拷进 .claude/singlefs-ai-sop/ 再跑它的 install.sh"; exit 1; }
done
TARGETS=()
MODE_TARGETS=()
for directory in "$ROOT/research/scripts" "$ROOT/.claude/hooks" "$ROOT/.claude/scripts"; do
  [[ -d "$directory" ]] || continue
  TARGETS+=("$directory")
  [[ "$directory" == "$ROOT/.claude/scripts" ]] || MODE_TARGETS+=("$directory")
done
((${#TARGETS[@]})) || { echo "  ! 没有 research/scripts/、.claude/hooks/ 也没有 .claude/scripts/，本阶段无对象可判"; exit 77; }
# GATE_LINT_DIR 指到第一个目标目录：不设它，共享脚本默认还会扫 SOP 自己的包，样本目录里红绿就靠那个包碰巧干净来分。
failed=0
if ! GATE_LINT_DIR="${TARGETS[0]}" bash "$LINT" "${TARGETS[@]:1}"; then
  echo "  ✗ research/scripts/、.claude/hooks/ 或 .claude/scripts/ 里有不带出路的拒绝（上面逐处列出）"
  echo "     → 怎么办：照 .claude/singlefs-ai-sop/rules/sop-first.md「每一条拒绝都必须给出下一步」补出路：die 加第二个参数，bad 后五行内写 howto，直接打印的拒绝之后跟一行以箭头开头的出路"
  failed=1
fi
# 执行位也一起：共享门禁的「脚本执行位」阶段只扫 .claude/gate.d 与 .claude/scripts，
# research/scripts 与 .claude/hooks 不在它的射程里（同 gate-lint / shell-lint 那两条的理由）；.claude/scripts 已在那一道里，不重复扫。
MODES="$(dirname "$LINT")/script-modes.sh"
if [[ -f "$MODES" ]] && ((${#MODE_TARGETS[@]})); then
  modes_rc=0
  bash "$MODES" "${MODE_TARGETS[@]}" || modes_rc=$?
  if (( modes_rc != 0 && modes_rc != 77 )); then
    echo "  ✗ research/scripts/ 或 .claude/hooks/ 里的脚本执行位在暂存区里不对（上面逐处列出）"
    echo "     → 怎么办：用 git update-index --chmod=+x <路径>（或 -x）改暂存区里的模式，让它与工作区一致"
    failed=1
  fi
fi
# shell-lint 一次只扫一个目录（SHELL_LINT_DIR），逐个目录跑
for directory in "${TARGETS[@]}"; do
  if ! SHELL_LINT_DIR="$directory" bash "$SHELL_LINT"; then
    echo "  ✗ ${directory#"$ROOT"/} 里有违反 shell 纪律的写法（上面逐处列出）"
    echo "     → 怎么办：照每一处给的改法改，规则在 .claude/singlefs-ai-sop/rules/command-safety.md；是 lint 认错了的，到上游 singlefs-ai-sop 修 shell-lint 并加样本，不在本仓绕开"
    failed=1
  fi
done
exit "$failed"

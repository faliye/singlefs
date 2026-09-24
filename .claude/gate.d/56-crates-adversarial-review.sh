#!/usr/bin/env bash
# gate-stage: crates 里的实现改动有没有走过三方正反对抗推理
#
# 本工程的实现流程是三步：写代码 → 多 agent 正反对抗推理（.claude/rules/three-way-inference.md）→ checker / 层 0 / 变异证明会红
# （.claude/rules/implementation-workflow.md）。第二步没有任何东西盯着：代码写完、单测绿、门禁绿，就直接提交了。
# 这一道判的是形式：这次改动里每个 crates/*/src/*.rs 文件，都要在这一次改动新写的某份三方判决文件
# （research/prompts/*-main-verification.md）里被按路径点名。点名了判得对不对，要人看。
# 新写的只认三种：相对基准新增（`git diff --diff-filter=A`）、暂存区新增、未跟踪；被改过的旧判决（路径改写、补一句）不算点名。
#
# 改动范围：GATE_BASE 给了就与它比，否则与 @{upstream} 的 merge-base 比，都没有就与 HEAD 比；工作区、暂存区与未跟踪文件都算
# （取法用共用库 research/scripts/changed-paths.sh 的 gate 取法，与 68、69、97 号同一份代码；它的 git 调用带 core.quotepath=false——
# 不带的话中文文件名的判决被转成带引号的八进制串，对不上 research/prompts/*-main-verification.md，点名了也算没点）。
# 没有 crates 改动 ⇒ 无对象可判，退 77（show-me-test.md：本次未跑，不记通过）。
#
#   bash .claude/gate.d/56-crates-adversarial-review.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $ROOT 不是 git 仓，本阶段跳过"; exit 77; }
LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
# shellcheck source=../../research/scripts/changed-paths.sh
source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里再抄一份。"; exit 1; }
base="$(gate_diff_base gate)"
changed="$(gate_changed_paths "$base" untracked)" || {
  echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
  echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一阶段什么都没比，不是通过。"
  exit 1
}
sources="$(grep -E '^crates/[^/]+/src/.*\.rs$' <<<"$changed" || true)"
if [[ -z "$sources" ]]; then
  echo "  ! 这次改动没碰 crates/*/src，本阶段无对象可判（基准 $base）"
  exit 77
fi
added="$(gate_changed_paths "$base" untracked A)" || {
  echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
  echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一阶段什么都没比，不是通过。"
  exit 1
}
verdicts="$(grep -E '^research/prompts/.*-main-verification\.md$' <<<"$added" || true)"
missing=()
count=0
while IFS= read -r source; do
  [[ -z "$source" ]] && continue
  count=$((count + 1))
  named=0
  while IFS= read -r verdict; do
    [[ -z "$verdict" || ! -f "$verdict" ]] && continue
    if grep -qF -- "$source" "$verdict"; then named=1; break; fi
  done <<<"$verdicts"
  ((named)) || missing+=("$source")
done <<<"$sources"
if ((${#missing[@]})); then
  echo "  ✗ crates 改动里 ${#missing[@]} 个文件没有三方判决点名（这次改动新写的 research/prompts/*-main-verification.md 里一份都没按路径提到它；被改过的旧判决不算）："   # gate-lint:summary
  printf '      %s\n' "${missing[@]}"   # gate-lint:detail
  echo "     → 怎么办：写代码之后、提交之前走一轮三方（.claude/rules/three-way-inference.md）：材料带上 diff，正推 / 反推 / 找反例三条腿各攻一遍，"
  echo "               主 agent 的判决写进 research/prompts/<题>-r<n>-main-verification.md，逐个按路径点名改过的 crates 文件（打中的写回代码再改一轮）；"
  echo "               点名写进新的判决，不往旧判决里补。"
  exit 1
fi
verdict_count="$(grep -c . <<<"$verdicts" || true)"
echo "  ✓ crates 改动 ${count} 个文件都有三方判决点名（新写的判决文件 ${verdict_count} 份，基准 $base）"

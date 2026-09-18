#!/usr/bin/env bash
# gate-stage: 改过的 agent 定义与共用约束有没有走过三方审核
#
# 规则在 `.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」：改 `.claude/agents/`、
# `.claude/agent-common.md`、`.claude/main-agent.md` 与改 `crates/` 同规矩，写完要走一轮三方（`.claude/rules/three-way-inference.md`），
# 判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份。形态照 56-crates-adversarial-review.sh。
# 这一道判的是形式：改动范围里每个被管文件，都要在这一次改动新写的某份判决文件里被按路径点名。点名了判得对不对，要人看；
# 新写的只认三种：相对基准新增（`git diff --diff-filter=A`）、暂存区新增、未跟踪；被改过的旧判决（路径改写、补一句）不算点名。
# 与 56 号同一个盲区——路径出现在「没攻它」那句里也算点名（`records/2026-09-16-subagent拆分提案.md` 第十一节第二行）。
#
# 改动范围：GATE_BASE 给了就与它比，否则与 @{upstream} 的 merge-base 比，都没有就与 HEAD 比；工作区、暂存区与未跟踪文件都算
# （删掉的定义也算改动）。没有被管文件的改动 ⇒ 无对象可判，退 77（本次未跑，不记通过）。
#
#   bash .claude/gate.d/72-agent-def-adversarial-review.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $ROOT 不是 git 仓，本阶段跳过"; exit 77; }
base="HEAD"
if [[ -n "${GATE_BASE:-}" ]] && git rev-parse --verify -q "${GATE_BASE}^{commit}" >/dev/null 2>&1; then
  base="$GATE_BASE"
elif git rev-parse --verify -q '@{upstream}' >/dev/null 2>&1; then
  base="$(git merge-base HEAD '@{upstream}')"
fi
changed="$( { git -c core.quotepath=false diff --name-only "$base" -- ; git -c core.quotepath=false diff --name-only --cached -- ; git -c core.quotepath=false ls-files --others --exclude-standard -- ; } | sort -u )"
definitions="$(grep -E '^\.claude/(agents/[^/]+\.md|agent-common\.md|main-agent\.md)$' <<<"$changed" || true)"
if [[ -z "$definitions" ]]; then
  echo "  ! 这次改动没碰 .claude/agents/、.claude/agent-common.md、.claude/main-agent.md，本阶段无对象可判（基准 $base）"
  exit 77
fi
added="$( { git -c core.quotepath=false diff --name-only --diff-filter=A "$base" -- ; git -c core.quotepath=false diff --name-only --cached --diff-filter=A -- ; git -c core.quotepath=false ls-files --others --exclude-standard -- ; } | sort -u )"
verdicts="$(grep -E '^research/prompts/.*-main-verification\.md$' <<<"$added" || true)"
missing=()
count=0
while IFS= read -r definition; do
  [[ -z "$definition" ]] && continue
  count=$((count + 1))
  named=0
  while IFS= read -r verdict; do
    [[ -z "$verdict" || ! -f "$verdict" ]] && continue
    if grep -qF -- "$definition" "$verdict"; then named=1; break; fi
  done <<<"$verdicts"
  ((named)) || missing+=("$definition")
done <<<"$definitions"
if ((${#missing[@]})); then
  echo "  ✗ 改过的定义与共用约束里 ${#missing[@]} 份没有三方判决点名（这次改动新写的 research/prompts/*-main-verification.md 里一份都没按路径提到它；被改过的旧判决不算）："   # gate-lint:summary
  printf '      %s\n' "${missing[@]}"   # gate-lint:detail
  echo "     → 怎么办：走一轮三方（.claude/rules/three-way-inference.md），判决落 research/prompts/<轮>-main-verification.md，"
  echo "               正文里按路径点名改过的每一份定义（打中的写回定义再改一轮）；点名写进新的判决，不往旧判决里补。"
  exit 1
fi
verdict_count="$(grep -c . <<<"$verdicts" || true)"
echo "  ✓ 改过的定义与共用约束 ${count} 份都有三方判决点名（新写的判决文件 ${verdict_count} 份，基准 $base）"

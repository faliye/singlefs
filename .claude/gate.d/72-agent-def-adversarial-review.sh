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
# 改动范围的取法用共用库（与 56、68、69、97 号同一份代码，git 调用带 core.quotepath=false、git 失败退非 0）
LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
# shellcheck source=../../research/scripts/changed-paths.sh
source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里另抄一份"; exit 1; }
base="$(gate_diff_base gate)"
changed="$(gate_changed_paths "$base" untracked)" || {
  echo "  ✗ 取不到这次改动碰了哪些路径（基准 $base）"
  echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一阶段什么都没比，不是通过。"
  exit 1
}
definitions="$(grep -E '^\.claude/(agents/[^/]+\.md|agent-common\.md|main-agent\.md)$' <<<"$changed" || true)"
if [[ -z "$definitions" ]]; then
  echo "  ! 这次改动没碰 .claude/agents/、.claude/agent-common.md、.claude/main-agent.md，本阶段无对象可判（基准 $base）"
  exit 77
fi
added="$(gate_changed_paths "$base" untracked A)" || {
  echo "  ✗ 取不到这次改动新增了哪些路径（基准 $base）"
  echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到新增的判决文件时这一阶段什么都没比，不是通过。"
  exit 1
}
verdicts="$(grep -E '^research/prompts/.*-main-verification\.md$' <<<"$added" || true)"
# 用户逐次豁免的那几份：`.claude/agent-def-review-exempt` 一行三段（路径、豁免时那一版的 sha256、为什么）。
# ⚠️ 按**内容哈希**认，不按文件名认：那份定义再改一个字，哈希就对不上、这道闸照常红。
# 只有用户明说「这次不走三方」时才加一行，理由里写明是哪一次、他说了什么。
EXEMPT=".claude/agent-def-review-exempt"
exempted=()
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
  if ((named)); then continue; fi
  if [[ -f "$EXEMPT" && -f "$definition" ]]; then
    now="$(sha256sum -- "$definition" | cut -d' ' -f1)"
    why="$(awk -F'\t' -v path="$definition" -v hash="$now" \
      '$1 !~ /^#/ && NF >= 3 && $1 == path && $2 == hash { print $3; exit }' "$EXEMPT")"
    if [[ -n "$why" ]]; then exempted+=("$definition：$why"); continue; fi
  fi
  missing+=("$definition")
done <<<"$definitions"
if ((${#missing[@]})); then
  echo "  ✗ 改过的定义与共用约束里 ${#missing[@]} 份没有三方判决点名（这次改动新写的 research/prompts/*-main-verification.md 里一份都没按路径提到它；被改过的旧判决不算）："   # gate-lint:summary
  printf '      %s\n' "${missing[@]}"   # gate-lint:detail
  echo "     → 怎么办：走一轮三方（.claude/rules/three-way-inference.md），判决落 research/prompts/<轮>-main-verification.md，"
  echo "               正文里按路径点名改过的每一份定义（打中的写回定义再改一轮）；点名写进新的判决，不往旧判决里补。"
  exit 1
fi
verdict_count="$(grep -c . <<<"$verdicts" || true)"
echo "  ✓ 改过的定义与共用约束 ${count} 份都有去向（新写的判决文件 ${verdict_count} 份，另有 ${#exempted[@]} 份按 $EXEMPT 豁免，基准 $base）"
if ((${#exempted[@]})); then
  echo "    豁免的 ${#exempted[@]} 份（按内容哈希认，那份文件再改一个字就不再豁免）："
  printf '      %s\n' "${exempted[@]}"
fi

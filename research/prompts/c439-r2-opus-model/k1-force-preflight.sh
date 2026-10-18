#!/usr/bin/env bash
# K1：外壳第 19–20 行的 preflight 先把 --force 摘掉，第 33 行转给里层的 "$@" 里就没有它。
# 不起外壳本体（重型闸拒「整轮门禁」）：只拿一份文件头与冻结外壳第 1–3 行相同的探针脚本，照第 19–20 行同样 source 项目 shim、调 preflight，
# 打出摘完之后的参数与 PREFLIGHT_FORCE_GIVEN。用法同 k1-binary.sh
source "$(dirname "$0")/lib.sh" "$1" "$2" "$3"
new_repo "$WORK/repo-force"
mkdir -p probe/scripts/gate probe/.claude/scripts probe/.claude/singlefs-ai-sop/scripts
cp "$TOOLS/.claude/scripts/preflight.sh" probe/.claude/scripts/
cp "$TOOLS/.claude/singlefs-ai-sop/scripts/preflight.sh" "$TOOLS/.claude/singlefs-ai-sop/scripts/preflight.py" probe/.claude/singlefs-ai-sop/scripts/
{ head -3 "$FROZEN/scripts/gate/gate-staged.sh"
  echo 'source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"'
  echo 'preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}'
  echo 'printf "after_preflight_args=[%s] PREFLIGHT_FORCE_GIVEN=%s\n" "$*" "$PREFLIGHT_FORCE_GIVEN"'
  echo 'ROOT="${1:-x}"; shift 2>/dev/null || true; printf "forwarded_to_inner=[%s]\n" "$*"'
} > probe/scripts/gate/probe.sh
echo "header_and_preflight_lines_match_frozen=$(diff <(sed -n '1,3p;19,20p' "$FROZEN/scripts/gate/gate-staged.sh") <(sed -n 1,5p probe/scripts/gate/probe.sh) >/dev/null && echo yes || echo no)"
echo "=== probe <根> --force"; bash probe/scripts/gate/probe.sh "$PWD" --force 2>&1 | sed 's/^/  /'
echo "=== probe <根> --force --other"; bash probe/scripts/gate/probe.sh "$PWD" --force --other 2>&1 | sed 's/^/  /'
cd "$WORK"; rm -rf "$WORK/repo-force"

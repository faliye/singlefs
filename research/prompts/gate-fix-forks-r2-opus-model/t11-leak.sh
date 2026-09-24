#!/usr/bin/env bash
# T11 第二问：共用脚本改成「先认 GATE_DIFF_BASE」时，gate.sh 导出的基准经样本自检（stage-selftest 只清 GATE_BASE、GATE_STAGED_FROM）
# 漏进样本的临时仓。两种认法：甲 解析得到提交就认；乙 只认 40 位十六进制、且在这个仓里是个提交。
# 用法：bash t11-leak.sh <真仓根> <草稿目录>   只读真仓；把 40、61、88 三道阶段与样本、补丁后的共用脚本拷进草稿目录跑上游 stage-selftest。
set -uo pipefail
REPO="$(cd "$1" && pwd)"; D="$2"; mkdir -p "$D"
unset GATE_BASE GATE_DIFF_BASE GATE_STAGED_FROM
REAL_HASH="$(git -C "$REPO" rev-parse HEAD)"
build() {  # $1 = 目录名，$2 = 甲 / 乙
  local R="$D/$1"; rm -rf "$R"; mkdir -p "$R/.claude/gate.d/fixtures" "$R/research/scripts"
  for s in 40-results-cited 61-settled-same-file 88-quoted-result-lines; do
    cp "$REPO/.claude/gate.d/$s.sh" "$R/.claude/gate.d/"; cp -r "$REPO/.claude/gate.d/fixtures/$s.sh" "$R/.claude/gate.d/fixtures/"
  done
  python3 - "$REPO/research/scripts/changed-paths.sh" "$R/research/scripts/changed-paths.sh" "$2" <<'PY'
import sys
src, dst, arm = sys.argv[1:4]
text = open(src, encoding='utf-8').read()
cond = '[[ -n "${GATE_DIFF_BASE:-}" ]]'
if arm == '乙':
    cond += ' && [[ "$GATE_DIFF_BASE" =~ ^[0-9a-f]{40}$ ]]'
patch = ('    gate)\n      if ' + cond + ' && git rev-parse --verify -q "${GATE_DIFF_BASE}^{commit}" >/dev/null 2>&1; then\n'
         '        echo "$GATE_DIFF_BASE"\n      elif [[ "${LIB_CHANGED_PATHS_BREAK:-}" != fallback && -n "${GATE_BASE:-}" ]] \\\n')
old = '    gate)\n      if [[ "${LIB_CHANGED_PATHS_BREAK:-}" != fallback && -n "${GATE_BASE:-}" ]] \\\n'
assert text.count(old) == 1
open(dst, 'w', encoding='utf-8').write(text.replace(old, patch))
PY
  echo "$R"
}
for arm in 甲 乙; do
  R="$(build "leak-$arm" "$arm")"
  for value in "" "$REAL_HASH" master HEAD; do
    out="$(GATE_DIFF_BASE="$value" nice -n 19 bash "$REPO/.claude/singlefs-ai-sop/scripts/stage-selftest.sh" "$R/.claude/gate.d" 2>&1)"; rc=$?
    printf '  认法%s  GATE_DIFF_BASE=%-40s  stage-selftest 退出码 %s；判错：%s\n' "$arm" "${value:-（不设）}" "$rc" \
      "$(grep -E '✗ .*(期望|找不到)' <<<"$out" | sed 's/^ *✗ *//' | tr -s ' ' | tr '\n' ';')"
  done
done
echo "  （GATE_DIFF_BASE=master 就是有人跑「GATE_BASE=master bash .claude/scripts/gate.sh」时 gate.sh 导出的值：lib.sh 的 diff_base 原样交回 GATE_BASE）"
echo "  同一漏法的第二条路：47 号跑的「bash research/scripts/changed-paths.sh --selftest」只清 GATE_BASE"
for arm in 甲 乙; do for value in master HEAD; do
  out="$(GATE_DIFF_BASE="$value" bash "$D/leak-$arm/research/scripts/changed-paths.sh" --selftest 2>&1)"; rc=$?
  printf '  认法%s  GATE_DIFF_BASE=%-6s 共用脚本 --selftest 退 %s；%s\n' "$arm" "$value" "$rc" "$(grep -E '期望' <<<"$out" | head -1 | sed 's/^ *//' | cut -c1-80)"
done; done

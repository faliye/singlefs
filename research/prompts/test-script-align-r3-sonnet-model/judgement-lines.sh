#!/usr/bin/env bash
# V3 复现：两份判决里的 cite-check 总数、快照漂移、现查行号。只读。用法：judgement-lines.sh <草稿目录>
S="${1:?用法：judgement-lines.sh <草稿目录>}"; mkdir -p "$S"
cd "$(git rev-parse --show-toplevel)" || exit 2
for r in 1 2; do
  echo "== r$r cite-check（对着今天的树）"
  python3 research/scripts/cite-check.py research/prompts/test-script-align-r$r-sonnet-output.md research/prompts/test-script-align-r$r-opus-output.md --root . --background research/prompts/_test-script-align-r$r-background.md > "$S/cite-r$r.txt" 2>&1
  grep -E '核了 [0-9]+ 处引文' "$S/cite-r$r.txt" | cut -c1-120
  echo "对不上的引文按被引文件数："; grep '✗ test-script' "$S/cite-r$r.txt" | sed -E 's/.* 引 ([^ ]+):[0-9]+「.*/\1/' | sort | uniq -c | sort -rn
  echo "快照 judged / kb 今天 sha256sum -c 的 FAILED 行数（总行数）："
  echo "judged $(sha256sum -c research/prompts/test-script-align-r$r-snapshot/judged-sha256.txt 2>&1 | grep -c ': FAILED') / $(wc -l < research/prompts/test-script-align-r$r-snapshot/judged-sha256.txt)"
  echo "kb $(sha256sum -c research/prompts/test-script-align-r$r-snapshot/kb-sha256.txt 2>&1 | grep -c ': FAILED') / $(wc -l < research/prompts/test-script-align-r$r-snapshot/kb-sha256.txt)"
done
echo "== r3 快照"
echo "judged $(sha256sum -c research/prompts/test-script-align-r3-snapshot/judged-sha256.txt 2>&1 | grep -c ': FAILED') / $(wc -l < research/prompts/test-script-align-r3-snapshot/judged-sha256.txt)"
echo "kb $(sha256sum -c research/prompts/test-script-align-r3-snapshot/kb-sha256.txt 2>&1 | grep -c ': FAILED') / $(wc -l < research/prompts/test-script-align-r3-snapshot/kb-sha256.txt)"
echo "== 第一轮判决里点名的行号，今天的内容"
for spec in ".claude/hooks/lib_heavy_tests.py:141" ".claude/hooks/lib_heavy_tests.py:1595" ".claude/hooks/lib_heavy_tests.py:2142" "research/scripts/crash-amplification.sh:658" "research/scripts/crash-amplification.sh:715" "research/scripts/crash-amplification.sh:721" "research/scripts/mutate.sh:84" "research/scripts/mutate.sh:90" "research/scripts/crash-case-check.py:190" "research/scripts/crash-case-check.py:191" ".claude/rules/verification.md:18"; do
  f=${spec%%:*}; n=${spec##*:}; echo "$spec => $(awk -v n="$n" 'NR==n' "$f" | cut -c1-110)"
done
echo -n "T3-3 ignored_points / states_skipped_by_ignore 在 crash-amplification.sh 里的行数: "; grep -c 'ignored_points\|states_skipped_by_ignore' research/scripts/crash-amplification.sh
echo -n "mutate.sh 里 BROKEN_JUDGEMENT=\"\${MUTATE_BREAK:-}\" 所在行: "; grep -n 'BROKEN_JUDGEMENT="${MUTATE_BREAK:-}"' research/scripts/mutate.sh | head -1 | cut -d: -f1

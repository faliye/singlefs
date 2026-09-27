#!/usr/bin/env bash
# K2 逐格核对：G2、G5、G7 改后字面 vs r2 判决第三节；G7 十道扩法的可核部分。
# 用法：bash rerun.sh <仓根>
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../../.." && pwd)}"
cd "$ROOT" || { echo "找不到仓根 $ROOT"; exit 2; }

echo "== 1. G2/G5/G7 改后字面：grep -nF 逐句核（命中即证明这句确实在文件里、行号现取） =="
grep -nF "这一次的新文件也不删：重跑已有实验时，第 5 步把 \`research/scripts/replay.sh\` 里这个实验的登记行改指新文件，旧的留着" .claude/agents/experiment-runner.md
grep -nF "取值是布尔 \`true\` 的、取值是 \`zero\` 的（例 \`control_violations_ok=true\`、\`layer0_violations=zero\`）与名字不表示「没过」的计数（例 \`journal_differing_states\`）不点名" .claude/agents/experiment-runner.md
grep -nF "读实验页或实验索引的那几道除外（27、34、40、69、75、84、85、86、88、99 号）" .claude/agents/experiment-runner.md
grep -nF "产物照第 4 步一个不删（86 号出路里「把 research 下那些文件删掉」那一句不照做，写进报告交主 agent）" .claude/agents/experiment-runner.md

echo
echo "== 2. r2 判决第三节 G2/G5/G7 原文行号 =="
grep -n "^- \*\*G2\*\*\|^- \*\*G5\*\*\|^- \*\*G7\*\*" research/prompts/defs-m2-closeout-r2-main-verification.md

echo
echo "== 3. 登记给 experiment-runner 的阶段（stage-owners.tsv 现算） =="
awk -F'\t' -v me="experiment-runner" '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv | sort

echo
echo "== 4. 十四道逐道 grep 'kb/experiments|experiments\\.md|\\.claude/kb|KB=|kb_dir'（复现 report.md 的现查） =="
for f in 27-format-constants.sh 33-mutation-tables.sh 34-experiment-index-sync.sh 40-results-cited.sh \
         52-segment-registry.sh 69-evidence-in-repo.sh 75-decision-experiment-links.sh 80-absolute-assertions.sh \
         84-verdict-false-named.sh 85-repro-command.sh 86-experiment-orphans.sh 88-quoted-result-lines.sh \
         96-experiment-source-discipline.sh 99-multipath-registry.sh; do
  echo "--- $f ---"
  grep -n 'kb/experiments\|experiments\.md\|\.claude/kb\|KB=\|kb_dir' ".claude/gate.d/$f" || echo "(零命中)"
done

echo
echo "== 5. G2 与 40 号 *.r[0-9].out 跳过规则交叉：40 号的 case 分支原文 =="
grep -n 'r\[0-9\]\.out' .claude/gate.d/40-results-cited.sh

echo
echo "== 6. 今天全部判决行（name=verdict），供 G5 逐格分类核对 =="
grep -rn 'name=verdict' research/results/*.out 2>/dev/null | wc -l
grep -rn 'name=verdict' research/results/*.out 2>/dev/null

echo
echo "== 7. 86 号出路文本里的『删掉』一句 =="
grep -n '删' .claude/gate.d/86-experiment-orphans.sh

#!/bin/bash
# 辩方腿 K1 的取证命令：只读，从仓根起；用法 bash evidence.sh <仓根>
cd "${1:?仓根}" || exit 2
S=records/2026-09-16-subagent拆分提案.md
R=records/2026-10-22-子agent结果落地审计.md
echo "== C1 九十节(第1466行起)与调度记录里这些词的命中行数"
for k in admission decision-evidence-lag 子目录 long-run-lines-dropped live-words-narrowed 起点 three-way-materials summary findings --until 修红 用量撞顶 subagent_transcript 常设; do
  a=$(awk 'NR>=1466' "$S" | grep -c -F -- "$k"); b=$(grep -c -F -- "$k" "$R")
  echo "$k: 九十节 $a, 调度记录 $b"
done
echo "== C2 九十节 19 行里写了红样本(自证…应当 或 样本)的"
python3 - "$S" <<'PY'
import re,sys
rows=[l for l in open(sys.argv[1],encoding='utf-8').read().split('\n')[1465:] if re.match(r'^\| [①-⑳] \|',l)]
c=[[x.strip() for x in l.strip('|').split('|')] for l in rows]
print('行数',len(c))
print('写了红样本的',[x[0] for x in c if ('自证' in x[2] and '应当' in x[2]) or '样本' in x[2]])
print('去向列是「下一批工具改动」的',sum(1 for x in c if x[3]=='下一批工具改动'))
PY
echo "== C3 ⑪ 点名的两处是否已改调库"
grep -n -F 'baseline_rows(base, pending_rel' .claude/gate.d/doc-experiments.sh | cut -c1-120
grep -n -F 'lag_baseline.baseline_text(diff_base' .claude/gate.d/doc-decisions.sh | cut -c1-120
echo "== C4 ⑪ 写的 gate-overlap 路径在不在；⑯ 要靠它报的最短相同段"
ls scripts/gate-overlap.py 2>&1 | head -1
grep -n -F 'CLONE_MINIMUM_LINES = 8' .claude/singlefs-ai-sop/scripts/gate-overlap.py
echo "== C5 三方类型表现有几处(不含自证样本)"
grep -n -E '^(THREE_WAY_REPORT_TYPES|CITATION_CHECKED_AGENTS) =' .claude/hooks/runner-dispatch-guard.sh scripts/agents/dispatch-prompt.py .claude/hooks/handback-guard.sh | cut -c1-170
grep -n -E 'startswith\("three-way"\)' scripts/agents/agent-cost.py | cut -c1-120
echo "== C6 共用约束 108 行三前缀与派发闸"
awk 'NR==108' .claude/agent-common.md | grep -o '以 REPORT、SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）'
grep -c -i -E 'summary\.md|findings|analysis' .claude/hooks/runner-dispatch-guard.sh
echo "== C7 复跑表里有没有 subagent_transcript / dispatch-prompt / quote-kb / runner-dispatch-guard"
grep -c -E 'subagent_transcript|dispatch-prompt|quote-kb|runner-dispatch-guard' .claude/gate.d/break-switch-replay-files.tsv
echo "== C8 rework.md 记下的缺口在定义里的痕迹(0=没有)"
echo "K12 static-check 在 implementation-writer.md: $(grep -c -F 'static-check' .claude/agents/implementation-writer.md)"
echo "K5 自证 在 experiment-runner.md: $(grep -c -F '自证' .claude/agents/experiment-runner.md)"
echo "K2 clippy 在 experiment-runner.md: $(grep -c -F 'clippy' .claude/agents/experiment-runner.md)"
echo "K10 观测不到|跑整格|重登记 在 main-agent.md: $(grep -c -E '观测不到|跑整格|重登记' .claude/main-agent.md)"
echo "K13 连带红 在 implementation-writer.md: $(grep -c -F '连带红' .claude/agents/implementation-writer.md)"
echo "结局账 agent-cost.py outcome 在 main-agent.md/agent-common.md/hooks 里的调用点: $(grep -rn -F 'agent-cost.py outcome' .claude/main-agent.md .claude/agent-common.md .claude/hooks 2>/dev/null | wc -l)"
echo "== C9 判决 r2 K1 K3-4 引的「主 agent 建的工作树」在定义里"
grep -rn -F '主 agent 建的工作树' .claude/agent-common.md .claude/agents .claude/main-agent.md | wc -l
echo "== C10 decision-evidence-lag.tsv 第9行"
awk 'NR==9' .claude/gate.d/decision-evidence-lag.tsv | cut -c1-120
echo "== C11 ⑥ 与 agent-common 命名检查"
awk 'NR==1477' "$S" | grep -o '没跑就拒交回'
awk 'NR==97' .claude/agent-common.md | grep -o '在 rsync 拷的副本里改的，不在副本上跑'
echo "== C12 门禁格头注自陈管不到"
awk 'NR==2406' .claude/gate.d/doc-experiments.sh | cut -c1-200
echo "== C13 admission.py 里复跑表的第五份读法"
grep -n -F 'REPLAY_TABLE_START =' scripts/gate/admission.py | cut -c1-80
echo "== C14 看门狗检出前两条：调度记录里的原话"
awk 'NR==121||NR==122' "$R" | cut -c1-120
echo "== C15 ⑰ 写的出处「调度记录第七批、一个工具员 20 分钟里报了五次以上」在调度记录里找得到吗"
echo "含「20 分钟」的行数: $(grep -c -F '20 分钟' "$R")；第七批一节(第183行起)含 mktemp 的行数: $(awk 'NR>=183' "$R" | grep -c -F 'mktemp')；含「五次」的行数: $(grep -c -F '五次' "$R")"
echo "== C16 ③ 的词在调度记录里的命中(粗算)"
echo "命中延后|下轮开|下一批的行: $(grep -c -E '延后|下轮开|下一批' "$R")；其中同行带 C 号、拆分提案、九十节或 checks-owed 的行: $(grep -E '延后|下轮开|下一批' "$R" | grep -c -E 'C[0-9]{3}|拆分提案|九十节|checks-owed')"

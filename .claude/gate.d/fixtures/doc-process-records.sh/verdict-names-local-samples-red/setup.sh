#!/usr/bin/env bash
# 两份新写的判决各漏点名一份作废副本（-output-void<n>.md）⇒ 本该判红，两份都要逐个列出。
# demo-r1 的判决没跟踪：点名了两份干净样本（一份带目录、一份只写文件名）与拆题那一份，漏了 void1。
# two-r1 的判决暂存新增：点名了 s1，漏了本地辩方的 void2。
# 漏的只有作废副本：VERDICT_LOCAL_SAMPLES_BREAK=skip-void 下这一例转绿，门禁判别力自证因此判红。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p research/prompts
printf '# 样本仓\n' > README.md
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
mkdir -p .claude/gate.d
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf 'Q1 Row D: yes, no\n' > research/prompts/demo-r1-local-attack-output-s1.md
printf 'Q1 Row D: threads=LAYER0; yes\n' > research/prompts/demo-r1-local-attack-output-s2.md
printf 'Q1 Row D: yes, **no\n' > research/prompts/demo-r1-local-attack-output-void1.md
printf 'Part 1 Q2: 6649413746\n' > research/prompts/demo-r1-local-attack-part1-output-s1.md
cat > research/prompts/demo-r1-main-verification.md <<'MD'
# demo 第一轮：主 agent 核实

本地腿：research/prompts/demo-r1-local-attack-output-s1.md（干净）、demo-r1-local-attack-output-s2.md（干净）、
demo-r1-local-attack-part1-output-s1.md（干净）；另有一份作废，不采。
MD
printf 'Q3: no counterexample\n' > research/prompts/two-r1-local-defense-output-s1.md
printf 'Q3: counterexample at step 5 of\n' > research/prompts/two-r1-local-defense-output-void2.md
cat > research/prompts/two-r1-main-verification.md <<'MD'
# two 第一轮：主 agent 核实

本地辩方：two-r1-local-defense-output-s1.md（干净）。
MD
git add research/prompts/two-r1-main-verification.md

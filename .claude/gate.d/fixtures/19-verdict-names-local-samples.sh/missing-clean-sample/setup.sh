#!/usr/bin/env bash
# 两份干净样本排版不同，新写的判决只点名了 s1、把 s2 写成「不稳定不采」而没给路径 ⇒ 本该判红，列出 s2。
# 漏的是干净样本，不是作废副本：VERDICT_LOCAL_SAMPLES_BREAK=skip-void 下这一例照样红。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p research/prompts
printf '# 样本仓\n' > README.md
git add -A && git commit -qm base
printf 'Row D: yes, no, enumerate_layer0_in_state_slices, yes\n' > research/prompts/three-r1-local-attack-output-s1.md
printf 'Row D: threads=LAYER0; yes; yes\n' > research/prompts/three-r1-local-attack-output-s2.md
cat > research/prompts/three-r1-main-verification.md <<'MD'
# three 第一轮：主 agent 核实

本地腿：three-r1-local-attack-output-s1.md；第二份样本 Row D 填法不同，不稳定不采。
MD

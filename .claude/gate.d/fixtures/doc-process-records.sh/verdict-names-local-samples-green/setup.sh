#!/usr/bin/env bash
# 本该判绿的几种：
#   demo-r1 新写的判决点名了 s1（带目录）、作废副本 void1 与拆题的 part1-s1（只写文件名）；s2 是 0 字节（退 5 时重定向建出的空文件）、
#     没点名 ⇒ 不判，成功句报跳过 1 份（VERDICT_LOCAL_SAMPLES_BREAK=require-empty 下它转红）；
#     demo-r1-local-attack-output.md（没有 -s<n>）、子目录里的样本、demo-r1-extra- 开头的别一轮的样本都不归 demo-r1 这份判决点名；
#   nolocal-r1 新写的判决，那一轮没有本地腿样本 ⇒ 照样计数报出；
#   old-r1 的判决在基准里就有、这次只补了一句，它漏点名的样本不追溯。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p research/prompts/sub
printf '# 样本仓\n' > README.md
printf 'Q1: yes\n' > research/prompts/old-r1-local-attack-output-s1.md
printf '# old 第一轮：主 agent 核实\n\n三条腿都没打中。\n' > research/prompts/old-r1-main-verification.md
# 同一个阶段的 batch-scope 与 knowledge-sync 两格读触发文件清单（读不到就红）：这份只放一条碰不到这份样本改动的正则、随基准提交，
# 那两格在这里判「没有触发文件」——batch-scope 绿、knowledge-sync 无对象可判
mkdir -p .claude/gate.d
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '\n路径改写：旧路径换成新路径。\n' >> research/prompts/old-r1-main-verification.md
printf 'Q1 Row D: yes, no\n' > research/prompts/demo-r1-local-attack-output-s1.md
: > research/prompts/demo-r1-local-attack-output-s2.md
printf 'Q1 Row D: yes, **no\n' > research/prompts/demo-r1-local-attack-output-void1.md
printf 'Part 1 Q2: 6649413746\n' > research/prompts/demo-r1-local-attack-part1-output-s1.md
printf 'summary\n' > research/prompts/demo-r1-local-attack-output.md
printf 'Q9: yes\n' > research/prompts/sub/demo-r1-local-attack-output-s9.md
printf 'Q1: no\n' > research/prompts/demo-r1-extra-local-attack-output-s1.md
cat > research/prompts/demo-r1-main-verification.md <<'MD'
# demo 第一轮：主 agent 核实

本地腿：research/prompts/demo-r1-local-attack-output-s1.md（干净）、demo-r1-local-attack-part1-output-s1.md（干净）、
demo-r1-local-attack-output-void1.md（参考：第 1 行成对标记落单，其余一行说 Row D 为 no，现查坐实而采）。
MD
printf '# nolocal 第一轮：主 agent 核实\n\n本地腿缺席：网关不通。\n' > research/prompts/nolocal-r1-main-verification.md

#!/usr/bin/env bash
# code-round-checklist 格的样本现场：只判八格的名单（样本根里的 .claude/gate.d/code-round-checklist-eight-cells.tsv）两行指的 _c994-code-r1-body.md、_c995-code-r1-body.md 都在、八格齐；
# 另有一份标题日期晚于第九格截止日的代码轮正文九格齐（这一格有对象可判）。名单只缩不长要一个基准：建 git 仓，提交一版只有第一行的名单，
# 再把第二行加回来（工作区比基准多一行）。晚的那个日期在跑样本时现算（取今天往后 30 天）。只起 git、grep、date。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
sample_date="$(TZ=Asia/Tokyo date -d '+30 days' +%F)"
mkdir -p research/prompts
eight_cells=(走不到的理由二次查实 类型与名字 不兼容旧代码与旧格式 不新建判定库 所有任务多机可用 测试优先多线程 "带 --item 的命令与项" 新崩溃流多罩了什么)
{
  printf '# c998-code-r1：样本代码轮第一轮正文（%s）\n\n' "$sample_date"
  for label in "${eight_cells[@]}" 放的位置; do
    printf '### 检查单：%s\n\n成立：样本里的答案与证据，crates/demo/src/lib.rs:1。\n\n' "$label"
  done
} > research/prompts/_c998-code-r1-body.md
{
  printf '# 样本里标题没有日期、基准那一版名单里就有的代码轮正文\n\n'
  for label in "${eight_cells[@]}"; do
    printf '### 检查单：%s\n\n成立：样本里的答案与证据，crates/demo/src/lib.rs:1。\n\n' "$label"
  done
} > research/prompts/_c994-code-r1-body.md
{
  printf '# 样本里标题没有日期、基准之后才加进名单的代码轮正文\n\n'
  for label in "${eight_cells[@]}"; do
    printf '### 检查单：%s\n\n成立：样本里的答案与证据，crates/demo/src/lib.rs:1。\n\n' "$label"
  done
} > research/prompts/_c995-code-r1-body.md
git init -q -b master .
cp .claude/gate.d/code-round-checklist-eight-cells.tsv .grown-list
grep -v '_c995-code-r1-body.md' .grown-list > .claude/gate.d/code-round-checklist-eight-cells.tsv
git add -A .claude research
git commit -qm base
mv .grown-list .claude/gate.d/code-round-checklist-eight-cells.tsv

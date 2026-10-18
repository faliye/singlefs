#!/usr/bin/env bash
# 点名的三类各一个在树里；另一个产物 e9-gone.out 先随基准提交、再从工作区删掉（提交过、树里没有）→ 只在历史里 1 个，绿。
# 它由 extra-notes/history.md 点名，那个目录只登记在 .claude/kb-directories 里：不读登记就看不到这一个。
# 弄坏开关 EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=history-lookup-skipped 不查历史，e9-gone.out 算缺，这份样本判红，判错。
# notes.md 末两行点名的 e9-never-kept.out 树里与历史里都没有，其中一行写明了原件未留存，两处都不判；开关 unretained-note-ignored 打开时它算缺，判错。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
git add -A
git commit -qm '基准：产物与 kb 页'
git rm -q research/results/e9-gone.out
git commit -qm '删掉一份产物（留在历史里）'

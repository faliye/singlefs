#!/usr/bin/env bash
# 造一段可控历史：先提交两条决策，再单独提交「乙样本新增了一个已定小节」。
# 于是被点名那条的状态行比点名它的未定项**更新** ⇒ 本该判红。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
GIT_COMMITTER_DATE="2026-08-31T00:00:00" GIT_AUTHOR_DATE="2026-08-31T00:00:00" sh -c 'git add -A && git commit -qm base'
printf '\n### 已定项 1 —— 已定（2026-09-01）：取甲\n\n依据。\n' >> .claude/kb/decisions/96-样本.md
GIT_COMMITTER_DATE="2026-09-01T00:00:00" GIT_AUTHOR_DATE="2026-09-01T00:00:00" sh -c 'git add -A && git commit -qm settle'
# 第三次提交改了那条未定项，但一句都没点名 D96 ⇒ 这不是复核，仍然本该判红（2026-09-12 改前这一步会把红消掉）。
sed -i 's/那边不定它就定不了/那边不定它就定不了（补一句与乙样本无关的话）/' .claude/kb/decisions/95-样本.md
# D94 只在第 1 条自己的 #### 论证里补一句点名 D96，第 2 条没动 ⇒ 第 2 条仍本该判红（条目块越过 #### 时这一句会把红消掉）。
sed -i 's/丙一的论证。/丙一的论证。顺带一提 D96（乙样本） 的口径在这里用不上。/' .claude/kb/decisions/94-样本.md
GIT_COMMITTER_DATE="2026-09-02T00:00:00" GIT_AUTHOR_DATE="2026-09-02T00:00:00" sh -c 'git add -A && git commit -qm unrelated-edit'

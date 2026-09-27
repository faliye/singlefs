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
# 缺了旧版本对象的两份（git 取不到时间，不许读成「从没定过」或「复核过了」）：
#   D93 在 base 之后改过一次、从没加过「—— 已定」小节 ⇒ git log -1 -G 要一路读到 base 那一版，而那一版的对象删掉了 ⇒ 读不到；
#   D990 的未定项点名 D96，条目块在 base 之后改过一次，最后一次提交加了「—— 已定」小节 ⇒ git log -1 -G 在最后那次就停、读得到，
#   而 git log -L 要一路追到条目块诞生的 base 那一版 ⇒ 读不到。
base_commit="$(git rev-list --max-parents=0 HEAD)"
sed -i 's/这份决策的正文：/这份决策的正文（改过一次）：/' .claude/kb/decisions/93-样本.md
sed -i 's/要等 D96（乙样本） 那边/要等 D96（乙样本） 那边，另补一句与它无关的话/' .claude/kb/decisions/990-样本.md
GIT_COMMITTER_DATE="2026-09-03T00:00:00" GIT_AUTHOR_DATE="2026-09-03T00:00:00" sh -c 'git add -A && git commit -qm touch-93-990'
printf '\n### 已定项 2 —— 已定（2026-09-04）：取丙\n\n依据。\n' >> .claude/kb/decisions/990-样本.md
GIT_COMMITTER_DATE="2026-09-04T00:00:00" GIT_AUTHOR_DATE="2026-09-04T00:00:00" sh -c 'git add -A && git commit -qm settle-990'
for decision_file in .claude/kb/decisions/93-样本.md .claude/kb/decisions/990-样本.md; do
  base_blob="$(git rev-parse "$base_commit:$decision_file")"
  rm -f ".git/objects/${base_blob:0:2}/${base_blob:2}"
done

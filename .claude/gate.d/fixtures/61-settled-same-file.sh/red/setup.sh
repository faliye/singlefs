#!/usr/bin/env bash
# 先提交一份带未定项的决策、把上游设在这个提交，再**另提交一次、不推**：加一个「已定」小节而不碰那条未定项；
# 工作区另改一个无关文件 ⇒ 本该判红：定了新东西却没回头看同文件还开着的那条。
# 定案已经提交在本地而没推，正是「在默认分支上 merge-base 就是 HEAD」那种基准会漏掉的形状（三方判决 gate-fix-forks-r1 的 T5）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master . && git add -A && git commit -qm base
git branch 上游 && git branch -q --set-upstream-to=上游 master
printf '\n### 已定项 2 —— 已定（2026-09-01）：取甲\n\n依据。\n' >> .claude/kb/decisions/97-样本.md
# D980 同一次定案只改了表格第 2 行，第 1 行没碰：表格行一行就是一条，第 1 行要报（条目块延到下一个 ### 时第 2 行的改动会替它交差）
printf '\n### 已定项 3 —— 已定（2026-09-01）：取乙\n\n依据。\n' >> .claude/kb/decisions/980-样本.md
sed -i 's/| 2 | \*\*丁二问题\*\* | 还没选/| 2 | **丁二问题** | 还没选（2026-09-01 复核过，仍然开着）/' .claude/kb/decisions/980-样本.md
# 索引页同一次只改了索引表 D980 那一行，「待议」那一节一个字没动：要报（整份 decisions.md 动过不算回头看过那一节）
sed -i 's/| D980（表格两行样本） | 半定（两项未定） | 样本，供门禁判别力自检/| D980（表格两行样本） | 半定（两项未定） | 样本，供门禁判别力自检（定了一项）/' .claude/kb/decisions.md
git add -A && git commit -qm '定案，没推'
printf '无关的改动\n' > 无关.md

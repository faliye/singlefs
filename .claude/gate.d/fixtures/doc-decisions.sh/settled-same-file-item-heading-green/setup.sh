#!/usr/bin/env bash
# 先提交一份带未定项的决策、把上游设在这个提交，再另提交一次、不推：在未定项那一节之前加一节 `#### 已定项 2：丙问题`，
# 同一次在那条未定项上补了复核注、索引页的待议节也补了复核注 ⇒ 本该判绿：新增了分项定案小节，同文件的未定项与待议节都被回头看过。
# 工作区另改一个无关文件。与 settled-same-file-item-heading-red 只差两处复核注。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master . && git add -A && git commit -qm base
git branch 上游 && git branch -q --set-upstream-to=上游 master
decision=.claude/kb/decisions/982-分项定案标题样本.md
awk '/^### 未定项$/ { print "#### 已定项 2：丙问题"; print ""; print "取丙。"; print "" } { print }' "$decision" > "$decision.new" && mv "$decision.new" "$decision"
grep -q '^#### 已定项 2：丙问题$' "$decision" || { echo "setup：没插进已定项 2 的标题" >&2; exit 1; }
sed -i 's/两条出路都还没选 \*\*状态：未定。\*\*/两条出路都还没选（2026-10-13 复核过，仍然开着） **状态：未定。**/' "$decision"
grep -q '2026-10-13 复核过，仍然开着' "$decision" || { echo "setup：没补上未定项的复核注" >&2; exit 1; }
sed -i 's/^悬着的一件事，定案之前就在这里。$/悬着的一件事，2026-10-13 复核过，仍然开着。/' .claude/kb/decisions.md
grep -q '2026-10-13 复核过，仍然开着' .claude/kb/decisions.md || { echo "setup：没补上待议节的复核注" >&2; exit 1; }
git add -A && git commit -qm '定案，没推'
printf '无关的改动\n' > 无关.md

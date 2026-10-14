#!/usr/bin/env bash
# 先提交一份带未定项的决策、把上游设在这个提交，再另提交一次、不推：在未定项那一节之前加一节 `#### 已定项 2：丙问题`（分项定案标题的现行写法），
# 不碰那条未定项；索引页的待议节同一次补了复核注，所以红只落在那条未定项上。工作区另改一个无关文件。
# 本该判红：定了新分项却没回头看同文件还开着的那条。格曾只认 `—— 已定` 与 `已定（` 两种标题，这一份看不见（C112）；
# 弄坏开关 DOC_DECISIONS_BREAK=settled-item-heading-unrecognised 退回老正则时，这一格报「本次 diff 没有新增「已定」小节」退 77。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master . && git add -A && git commit -qm base
git branch 上游 && git branch -q --set-upstream-to=上游 master
decision=.claude/kb/decisions/981-分项定案标题样本.md
awk '/^### 未定项$/ { print "#### 已定项 2：丙问题"; print ""; print "取丙。"; print "" } { print }' "$decision" > "$decision.new" && mv "$decision.new" "$decision"
grep -q '^#### 已定项 2：丙问题$' "$decision" || { echo "setup：没插进已定项 2 的标题" >&2; exit 1; }
sed -i 's/^悬着的一件事，定案之前就在这里。$/悬着的一件事，2026-10-13 复核过，仍然开着。/' .claude/kb/decisions.md
grep -q '2026-10-13 复核过，仍然开着' .claude/kb/decisions.md || { echo "setup：没补上待议节的复核注" >&2; exit 1; }
git add -A && git commit -qm '定案，没推'
printf '无关的改动\n' > 无关.md

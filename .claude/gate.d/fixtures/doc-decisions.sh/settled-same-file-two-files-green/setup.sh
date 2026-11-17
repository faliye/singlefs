#!/usr/bin/env bash
# 先提交两份带未定项的决策、把上游设在这个提交，再另提交一次、不推：D982 新增一节 `### 已定项 2 —— 已定`，同一次在它那条未定项上补了复核注、索引页的待议节也补了复核注；
# D983 同一次只改了「## 历史版本」下的一句话，没有新增已定小节，它那条未定项一个字没动 ⇒ 本该判绿：
# 新增已定小节的只有 D982 一份，D983 不算（弄坏开关 DOC_DECISIONS_BREAK=settled-same-file-added-lines-pooled 把几份改过的决策的新增行并成一堆看，D983 也算定了案，它那条没动的未定项判红）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master . && git add -A && git commit -qm base
git branch 上游 && git branch -q --set-upstream-to=上游 master
settled=.claude/kb/decisions/982-两份样本甲.md
other=.claude/kb/decisions/983-两份样本乙.md
printf '\n### 已定项 2 —— 已定（2026-10-13）：取丁\n\n依据。\n' >> "$settled"
sed -i 's/两条出路都还没选 \*\*状态：未定。\*\*/两条出路都还没选（2026-10-13 复核过，仍然开着） **状态：未定。**/' "$settled"
grep -q '2026-10-13 复核过，仍然开着' "$settled" || { echo "setup：没补上未定项的复核注" >&2; exit 1; }
sed -i 's/^悬着的一件事，定案之前就在这里。$/悬着的一件事，2026-10-13 复核过，仍然开着。/' .claude/kb/decisions.md
grep -q '2026-10-13 复核过，仍然开着' .claude/kb/decisions.md || { echo "setup：没补上待议节的复核注" >&2; exit 1; }
sed -i 's/^无。$/无。（补一句无关的话）/' "$other"
grep -q '补一句无关的话' "$other" || { echo "setup：没改上 D983 的历史版本" >&2; exit 1; }
git add -A && git commit -qm '定案，没推'

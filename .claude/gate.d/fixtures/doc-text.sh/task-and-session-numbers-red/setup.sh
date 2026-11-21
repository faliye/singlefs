#!/usr/bin/env bash
# 基准里一份干净的记录、一份批号在围栏里的记录；这一轮往干净那一份加一行带任务号的，把另一份开头那一行围栏删掉（批号那一行本身没改，出了围栏），
# 另放一份未跟踪的新记录带别的会话的批号、一份未跟踪的 kb 页带任务号：四处都报，判红。
# 射程之外的三份（.py 里的任务号、门禁样本目录下的 .md、research/prompts/ 下的判决）带同样的号，不报。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records scripts .claude/kb research/prompts .claude/gate.d/fixtures/sample/red
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度（2026-09-24）

- 一句干净的话。
X
printf '%s\n' '# 围栏样本' '' '```' '那一路第 7 批交回' '```' > records/fenced.md
git add -A && git commit -qm base
cat >> records/2026-09-24-样本调度.md <<'X'
- 调度那一条跑完（任务 21036）。
X
printf '%s\n' '# 围栏样本' '' '那一路第 7 批交回' '```' > records/fenced.md
printf '%s\n' '# 新记录' '' '- 它的批 105 在修。' > records/2026-09-25-新记录.md
printf '%s\n' '# kb 页' '' '- 复跑任务 21037 退 0。' > .claude/kb/probe.md
printf '%s\n' '# 判决' '' '- 复跑任务 21038 退 0；它的批 106 在修。' > research/prompts/x-main-verification.md
printf '%s\n' '# 任务 21036 跑完' > scripts/probe.py
printf '%s\n' '- 样本里的（任务 21036）' > .claude/gate.d/fixtures/sample/red/notes.md

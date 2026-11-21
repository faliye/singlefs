#!/usr/bin/env bash
# 基准里一份干净的记录；这一轮往里加按出场次序的 mainagent(A)、mainagent(B)、subagent(A)、subagent(B) 一行，「这一路」「那一路径」「一路绿」「按块那一路」一行，
# 反引号里的旧写法「那一路会话」一行，另放一份未跟踪的记录把同样的旧写法写在围栏里：都不是拿描述指别的会话，判绿。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度（2026-09-24）

- 一句干净的话。
X
git add -A && git commit -qm base
cat >> records/2026-09-24-样本调度.md <<'X'
- mainagent(A) 在改 `.claude/hooks/write-guard.sh`，mainagent(B) 认领了检出钩子；subagent(A) 交回了报告，subagent(B) 在草稿里改判据库。
- 这一路改到的文件都绿了；那一路径下的文件照留；门禁一路绿到收尾；按块那一路与变异那一路都不起。
- 旧写法 `改检出钩子那一路会话` 不再用。
X
printf '%s\n' '# 围栏样本' '' '```' '改检出钩子那一路会话' '```' > records/fenced.md

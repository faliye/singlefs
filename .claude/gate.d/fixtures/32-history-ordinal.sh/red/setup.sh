#!/usr/bin/env bash
# 两条路各撞一次：已跟踪的那份里又取了一次「其二」；按月拆出的新文件里（HEAD 里还没有它），
# 搬过去的存量撞号不算，而新写的一条又取了「其二」——算。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/kb
cat > .claude/kb/decisions-history.md <<'X'
# 决策变更史

## 历史版本

### 2026-09-02（其二）：另一个会话先取走的那条
### 2026-09-01（其一）：更早的一条
### 2026-08-30（其二）：存量撞号甲
### 2026-08-30（其二）：存量撞号乙
X
printf '# 实验变更史\n\n## 历史版本\n' > .claude/kb/experiments-history.md
git add -A && git commit -qm base
cat > .claude/kb/decisions-history.md <<'X'
# 决策变更史

## 历史版本

### 2026-09-02（其二）：本会话新写的一条
### 2026-09-02（其二）：另一个会话先取走的那条
### 2026-09-01（其一）：更早的一条
X
cat > .claude/kb/2026-08-decisions-history.md <<'X'
# 决策变更史 · 2026-08

## 历史版本

### 2026-08-30（其二）：新写的一条，又取了其二
### 2026-08-30（其二）：存量撞号甲
### 2026-08-30（其二）：存量撞号乙
X
git add -A

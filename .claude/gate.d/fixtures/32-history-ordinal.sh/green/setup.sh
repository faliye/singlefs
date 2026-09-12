#!/usr/bin/env bash
# 同一个现场，但取号前查过最大号：两处新条目都取「其三」；搬进新文件的存量撞号不算新撞号。
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

### 2026-09-02（其三）：本会话新写的一条
### 2026-09-02（其二）：另一个会话先取走的那条
### 2026-09-01（其一）：更早的一条
X
mkdir -p .claude/kb/decisions-history
cat > .claude/kb/decisions-history/2026-08.md <<'X'
# 决策变更史 · 2026-08

## 历史版本

### 2026-08-30（其三）：新写的一条
### 2026-08-30（其二）：存量撞号甲
### 2026-08-30（其二）：存量撞号乙
X
git add -A

#!/usr/bin/env bash
# 建一个最小 git 仓：变更史里已有「其一」「其二」，本次再取一次「其二」——撞号。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/kb
cat > .claude/kb/decisions-history.md <<'X'
# 决策变更史

## 历史版本

### 2026-09-02（其二）：另一个会话先取走的那条
### 2026-09-01（其一）：更早的一条
X
printf '# 实验变更史\n\n## 历史版本\n' > .claude/kb/experiments-history.md
git add -A && git commit -qm base
# 本次新增：没查最大号，又取了一次「其二」
sed -i 's/^### 2026-09-02（其二）：另一个会话先取走的那条/### 2026-09-02（其二）：本会话新写的一条\n### 2026-09-02（其二）：另一个会话先取走的那条/' .claude/kb/decisions-history.md

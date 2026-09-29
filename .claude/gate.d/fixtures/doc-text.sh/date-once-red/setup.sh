#!/usr/bin/env bash
# 基准里一份记录与一份实验页都干净；这一轮往记录正文加一行带文件名日期的话，往实验页加第二个同日 `### ` 小节：两份都判红。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records .claude/kb/experiments
cat > records/2026-09-24-样本调度.md <<'X'
# 样本调度（2026-09-24）

- 实一交回：跑了 5 轮。
X
cat > .claude/kb/experiments/01-样本实验.md <<'X'
## E1 样本实验 —— 部分已跑

### 2026-09-24：第一段

- 第一段做了甲。

## 历史版本
X
git add -A && git commit -qm base
cat >> records/2026-09-24-样本调度.md <<'X'
- 用户 2026-09-24 定：先收耗时表。
X
python3 - <<'PY'
path = '.claude/kb/experiments/01-样本实验.md'
text = open(path, encoding='utf-8').read()
text = text.replace('## 历史版本', '### 2026-09-24：第二段\n\n- 第二段做了乙。\n\n## 历史版本')
open(path, 'w', encoding='utf-8').write(text)
PY

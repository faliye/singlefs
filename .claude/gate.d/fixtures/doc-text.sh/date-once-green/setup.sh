#!/usr/bin/env bash
# 与 date-once-red 同一个基准；这一轮加的行只带别的日期（事件的锚）、产物名里的日期与引名，加的小节是另一天：判绿。
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
- 用户 2026-09-26 定：先收耗时表；产物 `e142-r18-2026-09-24.out`，见「2026-09-24：第一段」。
X
python3 - <<'PY'
path = '.claude/kb/experiments/01-样本实验.md'
text = open(path, encoding='utf-8').read()
text = text.replace('## 历史版本', '### 2026-09-25：第二段\n\n- 第二段做了乙。\n\n## 历史版本')
open(path, 'w', encoding='utf-8').write(text)
PY

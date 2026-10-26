#!/usr/bin/env bash
# 两份变更史与 INDEX.md 连同英日译文都不判：基准里九份都干净、另有一份干净的实验页；这一轮往九份里各加同一天在另一节的一块（INDEX 是同一天的第二个小节）
# 与一行重复所属日期的话，实验页加另一天的小节。date-once 只判实验页那一份，判绿。
# 弄坏开关 date-once-history-translations-judged（只排除中文那三份）下六份译文被点名，判红。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/kb/experiments
for name in decisions-history decisions-history.en decisions-history.ja experiments-history experiments-history.en experiments-history.ja; do
  cat > ".claude/kb/$name.md" <<'X'
# 样本变更史

## D1 样本决策

### 2026-10-01

- 条 3 改了措辞。

## D2 另一条决策

- 还没有变更。
X
done
for name in INDEX INDEX.en INDEX.ja; do
  cat > ".claude/kb/$name.md" <<'X'
# kb 样本索引

| 文件 | 内容 |
|---|---|
| [decisions.md](decisions.md) | 决策索引 |

### 2026-10-01

- 加了变更史那一行。
X
done
cat > .claude/kb/experiments/01-样本实验.md <<'X'
## E1 样本实验 —— 部分已跑

### 2026-09-24：第一段

- 第一段做了甲。

## 历史版本
X
git add -A && git commit -qm base
python3 - <<'PY'
import glob
for path in glob.glob('.claude/kb/*-history*.md'):
    text = open(path, encoding='utf-8').read()
    text = text.replace('- 还没有变更。', '### 2026-10-01\n\n- 条 5 改了措辞，2026-10-01 一并改的。')
    open(path, 'w', encoding='utf-8').write(text)
for path in glob.glob('.claude/kb/INDEX*.md'):
    with open(path, 'a', encoding='utf-8') as handle:
        handle.write('\n### 2026-10-01\n\n- 2026-10-01 加了实验变更史那一行。\n')
path = '.claude/kb/experiments/01-样本实验.md'
text = open(path, encoding='utf-8').read()
text = text.replace('## 历史版本', '### 2026-09-25：第二段\n\n- 第二段做了乙。\n\n## 历史版本')
open(path, 'w', encoding='utf-8').write(text)
PY

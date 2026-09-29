#!/usr/bin/env bash
# 基准与 shape-red 同一份：a、b、c、d、e 各一处存量，只计数、不判红。
# 这一轮新增的都照新形态写：日期块住进自己的节、同节日期不重复、块下是顶格 `- ` 变更项；
# 文件自己「## 历史版本」里的日期块没点名哪条决策；围栏里的旧形态是示例不算；决策正文里带日期的论证小节不是条目；
# kb 页自己的「## 历史版本」里照旧可以用 `#### `（只有两份变更史不许）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/kb/experiments .claude/kb/decisions
cat > .claude/kb/decisions.md <<'X'
# 设计决策记录

| 决策 | 状态 | 结论（简报） | 正文 |
|---|---|---|---|
| D1（样本决策） | 已定 1 项 / 未定 0 项 | 块取 16 KiB | [01-样本决策.md](decisions/01-样本决策.md) |
| D2（样本二） | 已定 2 项 / 未定 0 项 | 取乙 | [02-样本二.md](decisions/02-样本二.md) |

## 历史版本
X
cat > .claude/kb/experiments.md <<'X'
# 待做实验

| 实验 | 状态 | 结论（简报） | 正文 |
|---|---|---|---|
| E1（样本实验） | 部分已跑 | 一半已答 | [01-样本实验.md](experiments/01-样本实验.md) |

## 历史版本
X
cat > .claude/kb/invariants.md <<'X'
# 不变量

## 历史版本

### 2026-09-01（其一）：存量旧形态
X
cat > .claude/kb/decisions-history.md <<'X'
# 决策变更史

## D1（样本决策）

### 2026-09-10

#### 已有的一条（存量旧形态包装，块下也没有变更项）

### 2026-09-10

- 存量里同一天开了两个日期块

## D2（样本二）

## 历史版本
X
cat > .claude/kb/experiments-history.md <<'X'
# 实验变更史

## E1（样本实验）

## 历史版本

### 2026-09-02：E1（样本实验） 存量条目还在文件自己的历史里

- 存量。
X
git add -A && git commit -qm base

# ── 这一轮新增 ──
cat > .claude/kb/invariants.md <<'X'
# 不变量

## 历史版本

### 2026-09-28

#### 新加了一条（其一）

#### 新加了一条（其二）

### 2026-09-01（其一）：存量旧形态
X
cat > .claude/kb/decisions-history.md <<'X'
# 决策变更史

## D1（样本决策）

### 2026-09-28

- 定了一项
  - **结论**：块取 16 KiB。
  - **依据**：E1（样本实验）。
- 见 D2（样本二） 节同日条目「改了一处」。

### 2026-09-10

#### 已有的一条（存量旧形态包装，块下也没有变更项）

### 2026-09-10

- 存量里同一天开了两个日期块

## D2（样本二）

### 2026-09-27

- 改了一处
  - **改前**：`git show abc1234:.claude/kb/decisions/02-样本二.md`。

## 历史版本

### 2026-09-28

- 全部条目按决策拆进各节
X
cat > .claude/kb/experiments-history.md <<'X'
# 实验变更史

## E1（样本实验）

### 2026-09-26

- 跑了一段

## 历史版本

### 2026-09-02：E1（样本实验） 存量条目还在文件自己的历史里

- 存量。
X
cat > .claude/kb/experiments/01-样本实验.md <<'X'
## E1 样本实验

旧形态长这样，下面是示例：

```
### 2026-09-28（其三）：围栏里的示例
```

## 历史版本

### 2026-09-28

#### 新立
X
cat > .claude/kb/decisions/99-样本.md <<'X'
## D99 样本 —— 已定

### 2026-09-08 三轮对抗论证

正文里带日期的小节标题。

## 历史版本

D99（样本）的历史条目集中在 decisions-history.md。
X

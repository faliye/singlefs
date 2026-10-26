#!/usr/bin/env bash
# 现场同 history-loss-green（基准里一块带两个数与一个编号，工作区整理成一条结论、数都在），另把基准那一版变更史的对象从对象库里删掉：
# git ls-tree 认得出名字、git show 读不出。取不到基准那一版不许当成「基准里没有这份变更史」：改前 cat-file -e 失败就跳过，
# 两份都跳过之后报「与基准无差异」退 77（整理丢没丢其实没比）；改后经 lib-lag-baseline.py show 取，取不到判红、点名那一份。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/kb
cat > .claude/kb/decisions-history.md <<'Y'
# 决策变更史

## D1（样本决策）

### 2026-09-10

- 已定项 1 定案：节点 32 KiB
  - **依据**：E5（乙） 量到 1.7 倍。
- 索引改成 16 KiB 起步

## 历史版本
Y
git add -A && git commit -qm base
history_blob="$(git rev-parse HEAD:.claude/kb/decisions-history.md)"
cat > .claude/kb/decisions-history.md <<'Y'
# 决策变更史

## D1（样本决策）

### 2026-09-10

- 已定项 1 定案：节点 32 KiB，索引 16 KiB 起步；依据 E5（乙） 量到 1.7 倍。

## 历史版本
Y
rm -f ".git/objects/${history_blob:0:2}/${history_blob:2}"

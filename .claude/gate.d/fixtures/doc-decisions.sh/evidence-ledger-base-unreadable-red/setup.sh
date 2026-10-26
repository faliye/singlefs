#!/usr/bin/env bash
# 建一个最小的 git 仓：挂账表与决策正文一起提交成基准，工作区不改；再把基准那一版挂账表的对象从对象库里删掉：git ls-tree 认得出名字、git show 读不出。
# 挂账表的只缩不涨三格（cross-decision-status、inference-needs-verdict、three-way-leg-record）都要判红在「读不出基准 HEAD 里的 .claude/gate.d/decision-evidence-lag.tsv」，
# 不许当成基准里没有这张表、静默不比（改前 base_ledger_text 把取不到交 None，三格都判绿）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
git add -A && git commit -qm base
ledger_blob="$(git rev-parse HEAD:.claude/gate.d/decision-evidence-lag.tsv)"
rm -f ".git/objects/${ledger_blob:0:2}/${ledger_blob:2}"

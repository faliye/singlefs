#!/usr/bin/env bash
# 建一个最小的 git 仓：基准提交里挂账表只有已定项 1 那一行，工作区里挂账表再多一行（已定项 2）——相对基准涨了。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
ledger=.claude/gate.d/decision-evidence-lag.tsv
full="$(mktemp)"
cp "$ledger" "$full"
grep -v '已定项 2' "$full" > "$ledger"
git add -A && git commit -qm base
cp "$full" "$ledger"
rm -f "$full"

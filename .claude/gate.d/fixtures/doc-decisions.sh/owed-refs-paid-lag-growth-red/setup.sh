#!/usr/bin/env bash
# 基准提交里已定项 2 是「无。」、滞后表只登记 C20 那一行；这一轮新写了引 C21 的欠段、往滞后表加了罩它的一行：滞后表比基准多 1 行，判红。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
decision='.claude/kb/decisions/01-样本决策.md'
lag=.claude/gate.d/owed-refs-paid-lag.tsv
kept_decision="$(mktemp)"; kept_lag="$(mktemp)"
cp "$decision" "$kept_decision"; cp "$lag" "$kept_lag"
sed -i 's/^\*\*欠\*\*：C21（样本已还的乙）——这一轮新写的样本欠段。$/**欠**：无。/' "$decision"
grep -v -F 'C21（样本已还的乙）——这一轮新写的样本欠段' "$kept_lag" > "$lag"
git add -A && git commit -qm base
cp "$kept_decision" "$decision"; cp "$kept_lag" "$lag"
rm -f "$kept_decision" "$kept_lag"

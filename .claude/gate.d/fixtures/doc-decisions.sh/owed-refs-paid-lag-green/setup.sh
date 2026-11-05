#!/usr/bin/env bash
# 基准提交里已定项 2 还引着已还清的 C21、滞后表登记了它那一行；这一轮把它改成「无。」、删了滞后表那一行（只缩）：判绿，C20 那一处照登记放过。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
decision='.claude/kb/decisions/01-样本决策.md'
lag=.claude/gate.d/owed-refs-paid-lag.tsv
kept_decision="$(mktemp)"; kept_lag="$(mktemp)"
cp "$decision" "$kept_decision"; cp "$lag" "$kept_lag"
sed -i 's/^\*\*欠\*\*：无。$/**欠**：C21（样本已还的乙）。/' "$decision"
printf '%s\t%s\t%s\t%s\n' "$decision" C21 'C21（样本已还的乙）。' '# 样本：基准里登记的另一处，这一轮改好删了' >> "$lag"
git add -A && git commit -qm base
cp "$kept_decision" "$decision"; cp "$kept_lag" "$lag"
rm -f "$kept_decision" "$kept_lag"

#!/usr/bin/env bash
# 红样本的现场：基准提交里滞后表只有前三行（994、993、991 三页），工作区又加了 995 那一页与一行没写依据的，
# 「只缩不涨」那一条才有基线可比。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
lag=.claude/gate.d/multipath-registry-lag.tsv
current="$(mktemp)"
cp "$lag" "$current"
head -n 4 "$current" > "$lag"
git add -A
git commit -qm '样本基准：滞后表三行'
cp "$current" "$lag"
rm -f "${current:?}"

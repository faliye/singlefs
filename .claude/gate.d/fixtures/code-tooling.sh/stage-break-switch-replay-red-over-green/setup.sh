#!/usr/bin/env bash
# code-tooling 的 stage-break-switch-replay 格记红撤绿样本的现场：git init（判定库放 git common-dir），照这一版逐字拷一份上一版进 .unit-verdict-sample/prior/
# （门禁拿它照判绿把同一把键的两条记进库）。
set -euo pipefail
prior=.unit-verdict-sample/prior
mkdir -p "$prior"
cp -dR --preserve=timestamps,links .claude .red-switch "$prior/"
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude

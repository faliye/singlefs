#!/usr/bin/env bash
# research-build-and-replay.sh 的 research-unit-tests 格记红撤绿样本的现场：git init（判定库放 git common-dir），
# 照这一版逐字拷一份上一版进 .unit-verdict-sample/prior/（门禁拿它照判绿把同一把键记进库）。
# 样本带一份 scripts/experiment-apparatus/Cargo.lock：上一版拷的时候就有，两版的依赖行相同（理由同 research-unit-tests-unit-key）。
set -euo pipefail
prior=.unit-verdict-sample/prior
mkdir -p "$prior/scripts"
cp -dR --preserve=timestamps,links scripts/experiment-apparatus "$prior/scripts/"
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude

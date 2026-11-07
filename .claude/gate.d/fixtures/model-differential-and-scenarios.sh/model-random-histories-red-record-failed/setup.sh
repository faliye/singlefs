#!/usr/bin/env bash
# model-differential-and-scenarios 记红撤绿样本的现场：样本拷进临时目录之后，照这一版逐字拷一份上一版进 .unit-verdict-sample/prior/
# （门禁拿它照判绿把同一把键记进库），git init（判定库放 git common-dir），假工具补上执行位。
set -euo pipefail
chmod +x .model-differential-sample-tools/cargo .model-differential-sample-tools/rustc
prior=.unit-verdict-sample/prior
mkdir -p "$prior"
cp -dR --preserve=timestamps,links Cargo.toml crates "$prior/"
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude

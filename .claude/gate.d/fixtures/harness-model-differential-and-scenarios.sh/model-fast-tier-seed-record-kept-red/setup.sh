#!/usr/bin/env bash
# harness-model-differential-and-scenarios 样本 model-fast-tier-seed-record-kept-red 的现场：样本拷进临时目录之后，照这一版拷一份上一版进
# .unit-verdict-sample/prior/（与这一版相同；这一格走不到查库，只要上一版那一步算得出键），git init（判定库放 git common-dir），假工具补上执行位。
set -euo pipefail
chmod +x .model-differential-sample-tools/cargo .model-differential-sample-tools/rustc .model-differential-sample-tools/python3
prior=.unit-verdict-sample/prior
mkdir -p "$prior"
cp -a Cargo.toml crates "$prior/"
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude

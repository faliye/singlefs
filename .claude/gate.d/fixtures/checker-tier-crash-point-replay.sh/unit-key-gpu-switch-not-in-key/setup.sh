#!/usr/bin/env bash
# checker-tier-crash-point-replay「GPU 开没开不进键」样本的现场：样本拷进临时目录之后，
# 1. 照这一版原样拷一份上一版进 .unit-verdict-sample/prior/（源码逐字相同）；上一版那一趟的多机配置是样本里的 .unit-verdict-sample/prior-multi-host.env（ENABLE_GPU=0），
#    阶段开跑前照它拿上一版算键、照判绿记进这一版样本的判定库；这一版照 .crash-point-replay-sample-tools/multi-host.env（ENABLE_GPU=1）算键、查库。
# 2. git init，这一版连 HEAD 一起提交（判定库放 git common-dir，单元键要 git 列文件）。
set -euo pipefail
chmod +x .crash-point-replay-sample-tools/cargo .crash-point-replay-sample-tools/rustc
prior=.unit-verdict-sample/prior
mkdir -p "$prior"
cp -a Cargo.toml crates "$prior/"
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
git add -A
git -c user.name=sample -c user.email=sample@example.invalid commit -q -m "样本：这一版"

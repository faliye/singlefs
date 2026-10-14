#!/usr/bin/env bash
# 样本现场：format-constants 那一格只扫 git 列出的 scripts/experiments/、research/、crates/ 下的 .rs，所以样本要放进 git 仓；
# 被忽略的编译副本由这里现写（仓里的样本目录本身不带它：真仓的 .gitignore 会把 scripts/experiments/target/ 下的东西忽略掉，克隆下来就没有这份样本了）。
set -e
printf '/scripts/experiments/target/\n' > .gitignore
mkdir -p scripts/experiments/target/copy
printf 'const SAMPLE_HDR: u64 = 83; // 被忽略的编译副本里的旧值：红样本里它也不该被扫到\n' > scripts/experiments/target/copy/sample.rs
git init -q . && git add -A

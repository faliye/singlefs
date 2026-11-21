#!/usr/bin/env bash
# 先提交拆之前的样子、再换上拆之后的样子：基准提交里 e8 还是 scripts/experiment-apparatus/e8_seed_and_readings.rs 一个文件，两张豁免表各一行登记在它名下（before-split/ 里的那一版）；
# 工作区里它拆成 e8_seed_and_readings/main.rs 加 readings.rs，登记行跟着挪到拆出来的文件名下（样本根里摆着的那一版）。
# 只缩不涨按拆分族比之和：两张表在 e8 的族里都是 1 行，与基准相同。别的两格不读 git。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir .git/aside
mv before-split .git/aside/before
mv scripts/experiment-apparatus/e8_seed_and_readings .git/aside/after-e8
mv .claude/gate.d/experiment-seed-fold-lag.tsv .git/aside/after-seed-fold-lag.tsv
mv .claude/gate.d/experiment-constant-reading-lag.tsv .git/aside/after-constant-reading-lag.tsv
cp .git/aside/before/e8_seed_and_readings.rs scripts/experiment-apparatus/e8_seed_and_readings.rs
cp .git/aside/before/experiment-seed-fold-lag.tsv .claude/gate.d/experiment-seed-fold-lag.tsv
cp .git/aside/before/experiment-constant-reading-lag.tsv .claude/gate.d/experiment-constant-reading-lag.tsv
git add -A
git commit -qm '样本基准：e8 还是一个文件，两张豁免表各登记一行'
rm scripts/experiment-apparatus/e8_seed_and_readings.rs
mv .git/aside/after-e8 scripts/experiment-apparatus/e8_seed_and_readings
mv .git/aside/after-seed-fold-lag.tsv .claude/gate.d/experiment-seed-fold-lag.tsv
mv .git/aside/after-constant-reading-lag.tsv .claude/gate.d/experiment-constant-reading-lag.tsv

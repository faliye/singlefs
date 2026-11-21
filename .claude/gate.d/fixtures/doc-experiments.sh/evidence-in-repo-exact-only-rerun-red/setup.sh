#!/usr/bin/env bash
# 本该判红：E917 在复跑表里既有 timing 行又有 exact 行，装置改在只出 timing 产物的那一段；这一次只重跑了 exact 那一条（另存了一份更新的 exact 产物），timing 产物还是旧的。
# 有 timing 行的实验比修改时刻的对象是 timing 行点名的产物，exact 行点名的产物与另存的 exact 产物都不替它作保。
# 修改日子一律按东京的零点写（TZ=Asia/Tokyo touch -d <日期>），格里也按东京的日期报，与跑门禁那台机器的时区无关。
# 样本根的 .gate-cells 只点名 evidence-in-repo。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/experiment-apparatus research/results .claude/kb research/prompts
printf 'E917|e917-exact-only-rerun-timing||e917-exact-only-rerun-timing-2026-09-01.out|timing\nE917|e917-exact-only-rerun-anchors||e917-exact-only-rerun-anchors-2026-09-01.out|exact\n' > scripts/experiment-apparatus/replay.sh
printf 'fn main() { println!("e917"); }\n' > scripts/experiment-apparatus/e917_exact_only_rerun.rs
printf 'E7RESULT name=e917 rows=1\n' > research/results/e917-exact-only-rerun-timing-2026-09-01.out
printf 'E7RESULT name=e917 rows=1\n' > research/results/e917-exact-only-rerun-anchors-2026-09-01.out
git add -A && git commit -qm base
printf 'fn main() { println!("e917 改过了：改的是只出 timing 产物的那一段"); }\n' > scripts/experiment-apparatus/e917_exact_only_rerun.rs
printf 'E7RESULT name=e917 rows=1\n' > research/results/e917-exact-only-rerun-anchors-2026-09-06.out
TZ=Asia/Tokyo touch -d 2026-09-01 research/results/e917-exact-only-rerun-timing-2026-09-01.out research/results/e917-exact-only-rerun-anchors-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-05 scripts/experiment-apparatus/e917_exact_only_rerun.rs
TZ=Asia/Tokyo touch -d 2026-09-06 research/results/e917-exact-only-rerun-anchors-2026-09-06.out

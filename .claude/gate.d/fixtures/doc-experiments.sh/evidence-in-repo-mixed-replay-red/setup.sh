#!/usr/bin/env bash
# 本该判红：E915 在复跑表里既有 exact 行（点名的产物在盘）又有 timing 行，装置改了而两份产物都比它旧。
# 复跑表的行与装置源码没有机读的对应，装置这一次的改动可能落在只出 timing 产物的那一段，所以有 timing 行的实验不按复跑表放过，照旧按修改时刻判红。
# 修改日子一律按东京的零点写（TZ=Asia/Tokyo touch -d <日期>），格里也按东京的日期报，与跑门禁那台机器的时区无关。
# 样本根的 .gate-cells 只点名 evidence-in-repo。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/experiment-apparatus research/results .claude/kb research/prompts
printf 'E915|e915-mixed-replay-timing||e915-mixed-replay-timing-2026-09-01.out|timing\nE915|e915-mixed-replay-anchors||e915-mixed-replay-anchors-2026-09-01.out|exact\n' > scripts/experiment-apparatus/replay.sh
printf 'fn main() { println!("e915"); }\n' > scripts/experiment-apparatus/e915_mixed_replay.rs
printf 'E7RESULT name=e915 rows=1\n' > research/results/e915-mixed-replay-timing-2026-09-01.out
printf 'E7RESULT name=e915 rows=1\n' > research/results/e915-mixed-replay-anchors-2026-09-01.out
git add -A && git commit -qm base
printf 'fn main() { println!("e915 改过了"); }\n' > scripts/experiment-apparatus/e915_mixed_replay.rs
TZ=Asia/Tokyo touch -d 2026-09-01 research/results/e915-mixed-replay-timing-2026-09-01.out research/results/e915-mixed-replay-anchors-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-05 scripts/experiment-apparatus/e915_mixed_replay.rs

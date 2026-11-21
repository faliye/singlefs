#!/usr/bin/env bash
# 本该判红：复跑表里有 timing 行的两个实验，每一条 timing 行点名的产物都要在盘、都不比装置旧。
#   E919：两条 timing 行，第一份产物比装置新、第二份不在盘（没重跑）。
#   E920：两条 timing 行，第一份产物比装置新、第二份比装置旧。
# 只要有一份新产物就放过（比这个实验号在盘最新的那一份，或只比点名的产物里最新的那一份）的写法，两个都判绿。
# 修改日子一律按东京的零点写（TZ=Asia/Tokyo touch -d <日期>），格里也按东京的日期报，与跑门禁那台机器的时区无关。
# 样本根的 .gate-cells 只点名 evidence-in-repo。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/experiment-apparatus research/results .claude/kb research/prompts
{
  printf 'E919|e919-timing-row-missing-a||e919-timing-row-missing-a-2026-09-06.out|timing\n'
  printf 'E919|e919-timing-row-missing-b||e919-timing-row-missing-b-2026-09-01.out|timing\n'
  printf 'E920|e920-timing-row-stale-a||e920-timing-row-stale-a-2026-09-06.out|timing\n'
  printf 'E920|e920-timing-row-stale-b||e920-timing-row-stale-b-2026-09-01.out|timing\n'
} > scripts/experiment-apparatus/replay.sh
printf 'fn main() { println!("e919"); }\n' > scripts/experiment-apparatus/e919_timing_row_missing.rs
printf 'fn main() { println!("e920"); }\n' > scripts/experiment-apparatus/e920_timing_row_stale.rs
git add -A && git commit -qm base
printf 'fn main() { println!("e919 改过了"); }\n' > scripts/experiment-apparatus/e919_timing_row_missing.rs
printf 'fn main() { println!("e920 改过了"); }\n' > scripts/experiment-apparatus/e920_timing_row_stale.rs
printf 'E7RESULT name=e919 rows=1\n' > research/results/e919-timing-row-missing-a-2026-09-06.out
printf 'E7RESULT name=e920 rows=1\n' > research/results/e920-timing-row-stale-a-2026-09-06.out
printf 'E7RESULT name=e920 rows=1\n' > research/results/e920-timing-row-stale-b-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-01 research/results/e920-timing-row-stale-b-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-05 scripts/experiment-apparatus/e919_timing_row_missing.rs scripts/experiment-apparatus/e920_timing_row_stale.rs
TZ=Asia/Tokyo touch -d 2026-09-06 research/results/e919-timing-row-missing-a-2026-09-06.out research/results/e920-timing-row-stale-a-2026-09-06.out

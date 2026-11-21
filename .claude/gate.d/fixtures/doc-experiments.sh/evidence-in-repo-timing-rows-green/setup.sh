#!/usr/bin/env bash
# 本该判绿：E925 在复跑表里有两条 timing 行与一条 exact 行，装置改了；两条 timing 行点名的产物都重跑存回了（都比装置新），exact 行点名的产物还是旧的。
# exact 行的产物不在判的范围里（装置改在 exact 那一段时由 research-build-and-replay.sh 的逐字节复跑判红）。
# 修改日子一律按东京的零点写（TZ=Asia/Tokyo touch -d <日期>），格里也按东京的日期报，与跑门禁那台机器的时区无关。
# 样本根的 .gate-cells 只点名 evidence-in-repo。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/experiment-apparatus research/results .claude/kb research/prompts
{
  printf 'E925|e925-timing-rows-fresh-a||e925-timing-rows-fresh-a-2026-09-06.out|timing\n'
  printf 'E925|e925-timing-rows-fresh-b||e925-timing-rows-fresh-b-2026-09-06.out|timing\n'
  printf 'E925|e925-timing-rows-fresh-anchors||e925-timing-rows-fresh-anchors-2026-09-01.out|exact\n'
} > scripts/experiment-apparatus/replay.sh
printf 'fn main() { println!("e925"); }\n' > scripts/experiment-apparatus/e925_timing_rows_fresh.rs
printf 'E7RESULT name=e925 rows=1\n' > research/results/e925-timing-rows-fresh-anchors-2026-09-01.out
git add -A && git commit -qm base
printf 'fn main() { println!("e925 改过了"); }\n' > scripts/experiment-apparatus/e925_timing_rows_fresh.rs
printf 'E7RESULT name=e925 rows=1\n' > research/results/e925-timing-rows-fresh-a-2026-09-06.out
printf 'E7RESULT name=e925 rows=1\n' > research/results/e925-timing-rows-fresh-b-2026-09-06.out
TZ=Asia/Tokyo touch -d 2026-09-01 research/results/e925-timing-rows-fresh-anchors-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-05 scripts/experiment-apparatus/e925_timing_rows_fresh.rs
TZ=Asia/Tokyo touch -d 2026-09-06 research/results/e925-timing-rows-fresh-a-2026-09-06.out research/results/e925-timing-rows-fresh-b-2026-09-06.out

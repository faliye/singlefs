#!/usr/bin/env bash
# 本该判红：E926 在复跑表里一条 exact 行（行键 E926）、一条 timing 行（行键 E926R2，与仓里 E16R5、E155R2 同一种重跑写法），装置改了；
# timing 行点名的产物比装置旧，exact 行点名的产物刚重跑存回、另放一份 e926 起头的读数笔记，都比装置新。
# 行键带重跑后缀的 timing 行归 E926：比的是那一份 timing 产物，exact 产物与笔记不替它作保。
# 行键去掉 E 整个当号的写法（E926R2 归不到 E926），只剩 exact 行与笔记，判绿。
# 修改日子一律按东京的零点写（TZ=Asia/Tokyo touch -d <日期>），格里也按东京的日期报，与跑门禁那台机器的时区无关。
# 样本根的 .gate-cells 只点名 evidence-in-repo。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/experiment-apparatus research/results .claude/kb research/prompts
{
  printf 'E926|e926-suffix-anchors||e926-suffix-anchors-2026-09-01.out|exact\n'
  printf 'E926R2|e926-suffix-timing||e926-suffix-timing-2026-09-01.out|timing\n'
} > scripts/experiment-apparatus/replay.sh
printf 'fn main() { println!("e926"); }\n' > scripts/experiment-apparatus/e926_suffix_timing_row.rs
printf 'E7RESULT name=e926 rows=1\n' > research/results/e926-suffix-timing-2026-09-01.out
printf 'E7RESULT name=e926 rows=1\n' > research/results/e926-suffix-anchors-2026-09-01.out
git add -A && git commit -qm base
printf 'fn main() { println!("e926 改过了：改的是只出 timing 产物的那一段"); }\n' > scripts/experiment-apparatus/e926_suffix_timing_row.rs
printf '读数笔记，不是复跑产物。\n' > research/results/e926-notes-2026-09-06.txt
TZ=Asia/Tokyo touch -d 2026-09-01 research/results/e926-suffix-timing-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-05 scripts/experiment-apparatus/e926_suffix_timing_row.rs
TZ=Asia/Tokyo touch -d 2026-09-06 research/results/e926-suffix-anchors-2026-09-01.out research/results/e926-notes-2026-09-06.txt

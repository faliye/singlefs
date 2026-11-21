#!/usr/bin/env bash
# 本该判红：E918 在复跑表里既有 timing 行又有 exact 行，装置改在只出 timing 产物的那一段；research/results/ 里只多放了一份 e918 起头的笔记（不是复跑表点名的产物），timing 产物还是旧的。
# 比修改时刻的对象是 timing 行点名的产物，别的 e<号> 起头的文件不替它作保。
# 修改日子一律按东京的零点写（TZ=Asia/Tokyo touch -d <日期>），格里也按东京的日期报，与跑门禁那台机器的时区无关。
# 样本根的 .gate-cells 只点名 evidence-in-repo。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/experiment-apparatus research/results .claude/kb research/prompts
printf 'E918|e918-unrelated-file-timing||e918-unrelated-file-timing-2026-09-01.out|timing\nE918|e918-unrelated-file-anchors||e918-unrelated-file-anchors-2026-09-01.out|exact\n' > scripts/experiment-apparatus/replay.sh
printf 'fn main() { println!("e918"); }\n' > scripts/experiment-apparatus/e918_unrelated_file.rs
printf 'E7RESULT name=e918 rows=1\n' > research/results/e918-unrelated-file-timing-2026-09-01.out
printf 'E7RESULT name=e918 rows=1\n' > research/results/e918-unrelated-file-anchors-2026-09-01.out
git add -A && git commit -qm base
printf 'fn main() { println!("e918 改过了：改的是只出 timing 产物的那一段"); }\n' > scripts/experiment-apparatus/e918_unrelated_file.rs
printf '读数笔记，不是复跑产物。\n' > research/results/e918-notes-2026-09-06.txt
TZ=Asia/Tokyo touch -d 2026-09-01 research/results/e918-unrelated-file-timing-2026-09-01.out research/results/e918-unrelated-file-anchors-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-05 scripts/experiment-apparatus/e918_unrelated_file.rs
TZ=Asia/Tokyo touch -d 2026-09-06 research/results/e918-notes-2026-09-06.txt

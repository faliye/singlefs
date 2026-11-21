#!/usr/bin/env bash
# 本该判绿：E927 在复跑表里两条 timing 行，装置改了，两份 timing 产物都重跑存回、都比装置新：一份存成 .out，
# 另一份超过单文件推送上限、压成 .out.gz 入库（gzip -n -9，原文件删掉），只存着 .out.gz。
# 只存着 .gz 的 timing 产物也算在盘；只认原名的写法把它当不在盘，判红。
# 修改日子一律按东京的零点写（TZ=Asia/Tokyo touch -d <日期>），格里也按东京的日期报，与跑门禁那台机器的时区无关。
# 样本根的 .gate-cells 只点名 evidence-in-repo。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/experiment-apparatus research/results .claude/kb research/prompts
{
  printf 'E927|e927-timing-gz-plain||e927-timing-gz-plain-2026-09-06.out|timing\n'
  printf 'E927|e927-timing-gz-packed||e927-timing-gz-packed-2026-09-06.out|timing\n'
} > scripts/experiment-apparatus/replay.sh
printf 'fn main() { println!("e927"); }\n' > scripts/experiment-apparatus/e927_timing_gz.rs
git add -A && git commit -qm base
printf 'fn main() { println!("e927 改过了"); }\n' > scripts/experiment-apparatus/e927_timing_gz.rs
printf 'E7RESULT name=e927 rows=1\n' > research/results/e927-timing-gz-plain-2026-09-06.out
printf 'E7RESULT name=e927 rows=1\n' > research/results/e927-timing-gz-packed-2026-09-06.out
gzip -n -9 research/results/e927-timing-gz-packed-2026-09-06.out
TZ=Asia/Tokyo touch -d 2026-09-05 scripts/experiment-apparatus/e927_timing_gz.rs
TZ=Asia/Tokyo touch -d 2026-09-06 research/results/e927-timing-gz-plain-2026-09-06.out research/results/e927-timing-gz-packed-2026-09-06.out.gz

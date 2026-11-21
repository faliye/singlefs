#!/usr/bin/env bash
# 本该判红：E928 在复跑表里一条 timing 行，它点名的产物压成 .out.gz 入库（gzip -n -9，原文件删掉）、只存着 .out.gz，比装置旧；装置改了。
# 红句报的是 timing 行点名的产物里最旧的那一份，点名那份 .out.gz。
# 只认原名的写法把它当不在盘（红句换成「不在盘」）；有 timing 行也比这个实验号在盘最新那一份的写法，红句写成「最新的」：两种都对不上这里要的那一句。
# 修改日子一律按东京的零点写（TZ=Asia/Tokyo touch -d <日期>），格里也按东京的日期报，与跑门禁那台机器的时区无关。
# 样本根的 .gate-cells 只点名 evidence-in-repo。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/experiment-apparatus research/results .claude/kb research/prompts
printf 'E928|e928-timing-gz-stale||e928-timing-gz-stale-2026-09-01.out|timing\n' > scripts/experiment-apparatus/replay.sh
printf 'fn main() { println!("e928"); }\n' > scripts/experiment-apparatus/e928_timing_gz_stale.rs
printf 'E7RESULT name=e928 rows=1\n' > research/results/e928-timing-gz-stale-2026-09-01.out
gzip -n -9 research/results/e928-timing-gz-stale-2026-09-01.out
git add -A && git commit -qm base
printf 'fn main() { println!("e928 改过了"); }\n' > scripts/experiment-apparatus/e928_timing_gz_stale.rs
TZ=Asia/Tokyo touch -d 2026-09-01 research/results/e928-timing-gz-stale-2026-09-01.out.gz
TZ=Asia/Tokyo touch -d 2026-09-05 scripts/experiment-apparatus/e928_timing_gz_stale.rs

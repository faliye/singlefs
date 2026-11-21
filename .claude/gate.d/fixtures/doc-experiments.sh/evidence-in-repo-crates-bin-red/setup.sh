#!/usr/bin/env bash
# 本该判红：装置在 crates/*/src/bin/ 下（实验装置不只在 scripts/experiment-apparatus/）。
#   E921：单文件装置 crates/sample/src/bin/e921_apparatus.rs 改了，research/results/ 下一份 E921 的产物都没有。
#   E922：目录形装置 crates/sample/src/bin/e922_apparatus_dir/main.rs 改了，产物比它旧。
# 修改日子一律按东京的零点写（TZ=Asia/Tokyo touch -d <日期>），格里也按东京的日期报，与跑门禁那台机器的时区无关。
# 样本根的 .gate-cells 只点名 evidence-in-repo。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/sample/src/bin/e922_apparatus_dir research/results .claude/kb research/prompts
printf 'fn main() { println!("e921"); }\n' > crates/sample/src/bin/e921_apparatus.rs
printf 'fn main() { println!("e922"); }\n' > crates/sample/src/bin/e922_apparatus_dir/main.rs
printf 'E7RESULT name=e922 rows=1\n' > research/results/e922-apparatus-dir-2026-09-01.out
git add -A && git commit -qm base
printf 'fn main() { println!("e921 改过了"); }\n' > crates/sample/src/bin/e921_apparatus.rs
printf 'fn main() { println!("e922 改过了"); }\n' > crates/sample/src/bin/e922_apparatus_dir/main.rs
TZ=Asia/Tokyo touch -d 2026-09-01 research/results/e922-apparatus-dir-2026-09-01.out
TZ=Asia/Tokyo touch -d 2026-09-05 crates/sample/src/bin/e921_apparatus.rs crates/sample/src/bin/e922_apparatus_dir/main.rs

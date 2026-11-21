#!/usr/bin/env bash
# 本该判绿：装置在 crates/*/src/bin/ 下。
#   E923：装置 crates/sample/src/bin/e923_apparatus.rs 改了，产物比它新。
#   E924：装置 crates/sample/src/bin/e924_comment_only.rs 只改了 // 注释，不判、成功行列名。
#   helper_tool.rs：名字不是 e<号>_ 起头的二进制改了，不在对象里。
# 修改日子一律按东京的零点写（TZ=Asia/Tokyo touch -d <日期>），格里也按东京的日期报，与跑门禁那台机器的时区无关。
# 样本根的 .gate-cells 只点名 evidence-in-repo。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p crates/sample/src/bin research/results .claude/kb research/prompts
printf 'fn main() { println!("e923"); }\n' > crates/sample/src/bin/e923_apparatus.rs
printf '// 字段表见 first-layout.md\nfn main() { println!("e924"); }\n' > crates/sample/src/bin/e924_comment_only.rs
printf 'fn main() { println!("helper"); }\n' > crates/sample/src/bin/helper_tool.rs
git add -A && git commit -qm base
printf 'fn main() { println!("e923 改过了"); }\n' > crates/sample/src/bin/e923_apparatus.rs
printf '// 字段表见 layout/01-first.md\nfn main() { println!("e924"); }\n' > crates/sample/src/bin/e924_comment_only.rs
printf 'fn main() { println!("helper 改过了"); }\n' > crates/sample/src/bin/helper_tool.rs
printf 'E7RESULT name=e923 rows=1\n' > research/results/e923-apparatus-2026-09-06.out
TZ=Asia/Tokyo touch -d 2026-09-05 crates/sample/src/bin/e923_apparatus.rs crates/sample/src/bin/e924_comment_only.rs crates/sample/src/bin/helper_tool.rs
TZ=Asia/Tokyo touch -d 2026-09-06 research/results/e923-apparatus-2026-09-06.out

#!/usr/bin/env bash
# 本该判红：两份钉了装置 sha256 的起跑脚本，认不出哪一份是它自己的跑前登记，装置有没有变都判（改动范围里什么都没有）。
#   E938：注释里点名了一份登记（沿用它的几何），没有一处写成「跑前登记 <路径>」。
#   E939：注释里「跑前登记」后面点名了两份不同的登记（自己的与上一次的）。
# 样本根的 .gate-cells 只点名 pinned-apparatus-revision。
set -euo pipefail
git init -q -b master .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p scripts/experiment-apparatus crates/sample/src/bin research/prompts

for number in 938 939; do
  printf 'fn main() { println!("e%s"); }\n' "$number" > "crates/sample/src/bin/e${number}_apparatus.rs"
  printf '# E%s 第 2 次重跑登记\n\n## 十二、修订\n\n（装置改完之后、第一份产物之前由执行员写。）\n' "$number" > "research/prompts/e${number}-r2-prereg.md"
done
{
  echo '#!/usr/bin/env bash'
  echo '# E938 沿用 research/prompts/e938-r1-prereg.md 的几何与停机条款（只是参照，这一份自己的登记没在这里写成「跑前登记」）：'
  echo 'apparatus_source=crates/sample/src/bin/e938_apparatus.rs'
  printf 'registered_apparatus_sha256=%064d\n' 938
} > scripts/experiment-apparatus/e938-r2-segment-run.sh
{
  echo '#!/usr/bin/env bash'
  echo '# E939 第 2 次重跑一段的跑法（跑前登记 research/prompts/e939-r2-prereg.md 第五节「跑法」；上一次的跑前登记 research/prompts/e939-r1-prereg.md）：'
  echo 'apparatus_source=crates/sample/src/bin/e939_apparatus.rs'
  printf 'registered_apparatus_sha256=%064d\n' 939
} > scripts/experiment-apparatus/e939-r2-segment-run.sh
git add -A
git commit -qm '基准：两份认不出登记的起跑脚本'
